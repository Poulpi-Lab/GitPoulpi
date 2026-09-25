//! `<config>/radar-account.json`: the last account-wide answer per
//! integration, so the Radar opens on yesterday's rows while a fresh round
//! runs and `hosting radar` does not hammer a host's search API.
//!
//! Read leniently like the workspace search's cache: a missing, corrupt,
//! half-written or foreign-version file is an empty cache, and one bad entry
//! drops only itself. Writes go through a temp file and a rename, merged into
//! what is on disk, so the TUI and the CLI saving at once lose at most one
//! entry, never the file.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::AccountItems;

/// The file name under the config directory.
pub const ACCOUNT_CACHE_FILE: &str = "radar-account.json";

/// An entry newer than this is reused without asking the host.
pub const ACCOUNT_TTL_SECS: u64 = 300;
/// [`ACCOUNT_TTL_SECS`] as a duration.
pub const ACCOUNT_TTL: Duration = Duration::from_secs(ACCOUNT_TTL_SECS);

/// The cache file inside `config_dir`.
pub fn cache_path(config_dir: &Path) -> PathBuf {
    config_dir.join(ACCOUNT_CACHE_FILE)
}

/// One integration's last answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedAccount {
    /// Unix seconds.
    pub fetched_at: u64,
    /// [`super::AccountQuery::fingerprint`] of the question it answers.
    pub query_hash: String,
    pub items: AccountItems,
}

impl CachedAccount {
    /// Younger than [`ACCOUNT_TTL`] at `now` (unix seconds).
    pub fn is_fresh(&self, now: u64) -> bool {
        now.saturating_sub(self.fetched_at) < ACCOUNT_TTL_SECS
    }
}

/// Integration id -> its last answer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountCache {
    pub entries: BTreeMap<String, CachedAccount>,
}

impl AccountCache {
    /// The cache at `path`; anything unreadable is empty.
    pub fn load(path: &Path) -> Self {
        let Ok(raw) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        let Ok(map) = serde_json::from_str::<BTreeMap<String, serde_json::Value>>(&raw) else {
            return Self::default();
        };
        let entries = map
            .into_iter()
            .filter_map(|(id, value)| Some((id, serde_json::from_value(value).ok()?)))
            .collect();
        Self { entries }
    }

    /// The entry of `id` when it answers the same question, whatever its age
    /// (a caller shows it while it refreshes).
    pub fn get(&self, id: &str, query_hash: &str) -> Option<&CachedAccount> {
        self.entries.get(id).filter(|e| e.query_hash == query_hash)
    }

    /// [`Self::get`] restricted to entries younger than [`ACCOUNT_TTL`].
    pub fn fresh(&self, id: &str, query_hash: &str, now: u64) -> Option<&CachedAccount> {
        self.get(id, query_hash).filter(|e| e.is_fresh(now))
    }

    /// Stores `entry` for `id` into the file at `path`, keeping the other
    /// integrations' entries as they are on disk right now.
    pub fn store(path: &Path, id: &str, entry: CachedAccount) -> std::io::Result<()> {
        let mut cache = Self::load(path);
        cache.entries.insert(id.to_string(), entry);
        cache.save(path)
    }

    /// Writes the whole cache (temp file, then rename).
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string(&self.entries).map_err(std::io::Error::other)?;
        let tmp = path.with_extension(format!("json.{}.tmp", std::process::id()));
        std::fs::write(&tmp, text)?;
        std::fs::rename(&tmp, path).inspect_err(|_| {
            let _ = std::fs::remove_file(&tmp);
        })
    }
}
