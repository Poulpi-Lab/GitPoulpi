//! What is kept between two replies about the same repository.
//!
//! - **Built contexts** ([`ContextCache`]): the last eight
//!   [`RepoContext`]s, found again by their [`ContextKey`] (repository, HEAD,
//!   selection, limits) while they are younger than `context_ttl_minutes`.
//! - **Parts** ([`PartCache`]): each part on its own under `(HEAD, what it
//!   is)`, so the next issue in the same repository rebuilds only the parts
//!   it does not share (the map, the files both mention).
//! - **Agent sessions** ([`SessionStore`]): the id of the session a local
//!   agent keeps between replies, per `(repository, HEAD, action)`.
//!
//! A new HEAD drops everything of the old one (the key already differs, the
//! frontends also call [`ContextCache::head_changed`]), and "Refresh context"
//! is [`ContextCache::refresh`] and [`SessionStore::drop_repo`]. In the TUI
//! the caches live in memory for the window; the CLI keeps them in files
//! under its state directory (user-only, `secure_fs`), so a second
//! `gitpoulpi ai hosting-reply --read-repo` reuses what the first built.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use gitpoulpi_core::domain::Oid;
use serde::{Deserialize, Serialize};

use super::repo_context::{ContextKey, PartKind, RepoContext};
use crate::presets::SessionUse;
use gitpoulpi_config::AiAction;

/// Built contexts kept.
const KEPT_CONTEXTS: usize = 8;

/// Seconds since the epoch.
pub fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// One part, as the cache keeps it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedPart {
    pub kind: PartKind,
    pub text: String,
    pub at: u64,
}

/// Parts of repository context, by `(head, id)`.
#[derive(Debug, Default)]
pub struct PartCache {
    parts: BTreeMap<String, CachedPart>,
    ttl: u64,
}

impl PartCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            parts: BTreeMap::new(),
            ttl: ttl_secs,
        }
    }

    fn slot(head: &str, id: &str) -> String {
        format!("{head}\u{1}{id}")
    }

    /// The part, when it is younger than the time to live.
    pub fn get(&self, head: &str, id: &str, now: u64) -> Option<CachedPart> {
        self.parts
            .get(&Self::slot(head, id))
            .filter(|p| now.saturating_sub(p.at) <= self.ttl)
            .cloned()
    }

    pub fn put(&mut self, head: &str, id: &str, kind: &PartKind, text: &str, now: u64) {
        self.parts.insert(
            Self::slot(head, id),
            CachedPart {
                kind: kind.clone(),
                text: text.to_string(),
                at: now,
            },
        );
    }

    /// Drops what belongs to another commit than `head`.
    pub fn keep_only(&mut self, head: &str) {
        let prefix = format!("{head}\u{1}");
        self.parts.retain(|slot, _| slot.starts_with(&prefix));
    }

    pub fn len(&self) -> usize {
        self.parts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }
}

#[derive(Default, Serialize, Deserialize)]
struct Stored {
    #[serde(default)]
    contexts: Vec<RepoContext>,
    #[serde(default)]
    parts: BTreeMap<String, CachedPart>,
}

/// The built contexts and parts of one repository.
pub struct ContextCache {
    ttl: u64,
    /// Newest first.
    contexts: VecDeque<Arc<RepoContext>>,
    parts: PartCache,
    path: Option<PathBuf>,
    clock: fn() -> u64,
}

impl ContextCache {
    /// In memory (the TUI window's).
    pub fn new(ttl_minutes: u32) -> Self {
        let ttl = u64::from(ttl_minutes.max(1)) * 60;
        Self {
            ttl,
            contexts: VecDeque::new(),
            parts: PartCache::new(ttl),
            path: None,
            clock: unix_now,
        }
    }

    /// Kept in `<dir>/<hash of the repository>.json` (the CLI's); what is
    /// there already is read back, an unreadable file is ignored.
    pub fn on_disk(dir: &Path, repo_root: &Path, ttl_minutes: u32) -> Self {
        let mut cache = Self::new(ttl_minutes);
        let path = dir.join(format!(
            "{:016x}.json",
            hash_of(&repo_root.to_string_lossy())
        ));
        if let Some(stored) = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Stored>(&text).ok())
        {
            cache.contexts = stored.contexts.into_iter().map(Arc::new).collect();
            cache.parts.parts = stored.parts;
        }
        cache.path = Some(path);
        cache
    }

    /// A clock for tests.
    pub fn with_clock(mut self, clock: fn() -> u64) -> Self {
        self.clock = clock;
        self
    }

    pub fn now(&self) -> u64 {
        (self.clock)()
    }

    /// The built context for `key`, while it is fresh.
    pub fn get(&mut self, key: &ContextKey) -> Option<Arc<RepoContext>> {
        let now = self.now();
        let at = self
            .contexts
            .iter()
            .position(|c| &c.key == key && now.saturating_sub(c.built_at) <= self.ttl)?;
        // The most recently used first.
        let found = self.contexts.remove(at)?;
        self.contexts.push_front(found.clone());
        Some(found)
    }

    /// Keeps `context` (the oldest of eight is dropped).
    pub fn put(&mut self, context: RepoContext) -> Arc<RepoContext> {
        let context = Arc::new(context);
        self.contexts.retain(|c| c.key != context.key);
        self.contexts.push_front(context.clone());
        self.contexts.truncate(KEPT_CONTEXTS);
        self.save();
        context
    }

    /// The part cache, for [`super::repo_context::build`].
    pub fn parts(&mut self) -> &mut PartCache {
        &mut self.parts
    }

    /// HEAD of the repository is `head` now: what was built for another
    /// commit is dropped.
    pub fn head_changed(&mut self, head: Oid) {
        self.contexts.retain(|c| c.key.head == head);
        self.parts.keep_only(&head.to_hex());
        self.save();
    }

    /// "Refresh context": forget everything.
    pub fn refresh(&mut self) {
        self.contexts.clear();
        self.parts.parts.clear();
        self.save();
    }

    pub fn len(&self) -> usize {
        self.contexts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.contexts.is_empty()
    }

    /// Writes the file (a no-op in memory); a failure is not an error, the
    /// cache is only a saving.
    pub fn save(&self) {
        let Some(path) = &self.path else { return };
        let stored = Stored {
            contexts: self.contexts.iter().map(|c| (**c).clone()).collect(),
            parts: self.parts.parts.clone(),
        };
        let Ok(text) = serde_json::to_string(&stored) else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = gitpoulpi_config::secure_fs::write_private(path, text.as_bytes());
    }
}

fn hash_of(text: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

// -- agent sessions ---------------------------------------------------------------

/// What a session belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionKey {
    pub repo_root: PathBuf,
    pub head: String,
    pub action: AiAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Entry {
    id: String,
    /// The agent ran once with it: the next run resumes it.
    started: bool,
}

#[derive(Default, Serialize, Deserialize)]
struct SessionFile {
    #[serde(default)]
    sessions: BTreeMap<String, Entry>,
}

/// The local agent's session ids.
#[derive(Debug, Default)]
pub struct SessionStore {
    entries: HashMap<SessionKey, Entry>,
    path: Option<PathBuf>,
}

fn flat(key: &SessionKey) -> String {
    format!(
        "{}\u{1}{}\u{1}{}",
        key.repo_root.display(),
        key.head,
        key.action.key()
    )
}

impl SessionStore {
    /// In memory (the TUI's).
    pub fn new() -> Self {
        Self::default()
    }

    /// Kept in `file` (`<state>/ai-sessions.toml`, user-only).
    pub fn on_disk(file: &Path) -> Self {
        let mut store = Self {
            entries: HashMap::new(),
            path: Some(file.to_path_buf()),
        };
        let stored: SessionFile = std::fs::read_to_string(file)
            .ok()
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default();
        for (flat_key, entry) in stored.sessions {
            let mut parts = flat_key.split('\u{1}');
            let (Some(root), Some(head), Some(action)) = (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            let Some(action) = AiAction::from_key(action) else {
                continue;
            };
            store.entries.insert(
                SessionKey {
                    repo_root: PathBuf::from(root),
                    head: head.to_string(),
                    action,
                },
                entry,
            );
        }
        store
    }

    /// How the next run uses the session of `key`: the first starts it, the
    /// later ones resume it.
    pub fn next_use(&mut self, key: &SessionKey) -> SessionUse {
        let entry = self.entries.entry(key.clone()).or_insert_with(|| Entry {
            id: new_session_id(),
            started: false,
        });
        let used = if entry.started {
            SessionUse::Resume(entry.id.clone())
        } else {
            SessionUse::Start(entry.id.clone())
        };
        self.save();
        used
    }

    /// The agent ran: the session exists now.
    pub fn mark_started(&mut self, key: &SessionKey) {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.started = true;
        }
        self.save();
    }

    /// The run failed before the agent kept anything: forget a session that
    /// never started.
    pub fn forget_unstarted(&mut self, key: &SessionKey) {
        if self.entries.get(key).is_some_and(|e| !e.started) {
            self.entries.remove(key);
            self.save();
        }
    }

    /// HEAD moved or the user refreshed: the repository's sessions end.
    pub fn drop_repo(&mut self, root: &Path) {
        self.entries.retain(|k, _| k.repo_root != root);
        self.save();
    }

    /// Drops the sessions of `root` that belong to another commit.
    pub fn keep_head(&mut self, root: &Path, head: &str) {
        self.entries
            .retain(|k, _| k.repo_root != root || k.head == head);
        self.save();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn save(&self) {
        let Some(path) = &self.path else { return };
        let file = SessionFile {
            sessions: self
                .entries
                .iter()
                .map(|(k, v)| (flat(k), v.clone()))
                .collect(),
        };
        let Ok(text) = toml::to_string(&file) else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = gitpoulpi_config::secure_fs::write_private(path, text.as_bytes());
    }
}

/// A random version-4 UUID (the id a session is named by).
pub fn new_session_id() -> String {
    use std::hash::{BuildHasher, Hasher};
    let mut bytes = [0u8; 16];
    for chunk in bytes.chunks_mut(8) {
        // `RandomState` is seeded by the operating system for every value.
        let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
        hasher.write_u64(unix_now());
        hasher.write_usize(std::process::id() as usize);
        chunk.copy_from_slice(&hasher.finish().to_le_bytes());
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}
