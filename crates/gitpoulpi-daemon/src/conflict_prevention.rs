//! Proactive conflict detection (plan `repo-settings.md`, slice 3): would
//! merging HEAD into the branch it started from conflict, and on which files.
//!
//! The answer to `OperationKind::PredictConflicts`. It is built here, never
//! on a client's render thread and never under a repository lock for longer
//! than a read:
//!
//! 1. **Settings** (`general.conflict_detection`, the repository's own switch
//!    and its Base branches globs: its own list, a named list or the global
//!    default, `conflict_prevention.*`) are read per request: a request is rare and
//!    a changed setting applies at once.
//! 2. **The base** is the branch of the pull request HEAD's branch has in the
//!    repository's hosting cache (no network call), else the branch among the
//!    ones the globs allow with the fewest commits since the fork
//!    ([`pick_base`]).
//! 3. **The answer is cached by `(head oid, base oid)`**, so asking again
//!    after a refresh that moved nothing costs the settings read, the two
//!    resolves and the graph walks of step 2, never a merge.
//! 4. **The merge runs once at a time per repository** (`busy`), and a
//!    request that asked to be debounced waits for the repository to be quiet
//!    first: a newer debounced request for the same repository supersedes it
//!    (`Superseded`, nothing computed), so a rebase or a burst of commits
//!    costs one merge at the end. A merge whose request was superseded while
//!    it ran is still cached (it is right for its key) but not answered.
//!
//! Nothing in the work tree, index or any ref is touched; the merge itself is
//! `GitBackend::predict_merge_conflicts`.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use gitpoulpi_core::domain::conflict_prevention::{
    base_allowed, pick_base, BaseCandidate, BaseSource, ConflictPrediction, PredictionStatus,
    MAX_BASE_CANDIDATES,
};
use gitpoulpi_core::domain::{Oid, RefKind, RepoRef};
use gitpoulpi_core::{Error, GitBackend, Result};
use gitpoulpi_ipc::{Request, Response, ServerMessage};
use serde::Deserialize;

/// The quiet time a debounced request waits for.
pub const DEFAULT_DEBOUNCE_MS: u64 = 2000;

/// What the settings say about one repository.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Settings {
    enabled: bool,
    globs: Vec<String>,
}

fn read_settings(root: &Path) -> Settings {
    let config = gitpoulpi_config::Settings::resolve(
        gitpoulpi_config::ConfigPaths::discover().as_ref(),
        &gitpoulpi_config::Overrides::default(),
    )
    .ok();
    let global = config
        .as_ref()
        .map(|s| s.config().general.conflict_detection)
        .unwrap_or(true);
    let state_dir = config
        .as_ref()
        .map(|s| s.config().session.per_repo_state_dir.clone())
        .unwrap_or_else(|| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
    let lists = config
        .as_ref()
        .map(|s| s.config().conflict_prevention.clone())
        .unwrap_or_default();
    let prefs = gitpoulpi_session::RepoState::load(&gitpoulpi_session::RepoState::path_for(
        root, &state_dir,
    ))
    .map(|s| s.preferences)
    .unwrap_or_default();
    Settings {
        enabled: global && prefs.conflict_detection.unwrap_or(true),
        // The lists live in config.toml: read again with every request, so an
        // edited list or default applies to the next check without a restart.
        globs: lists
            .resolve(
                prefs.conflict_base_source.as_deref(),
                prefs.conflict_base_list.as_deref(),
                prefs.conflict_base_branches.as_deref(),
            )
            .patterns,
    }
}

/// The part of the TUI's hosting cache (`hosting-cache.json`) this needs: a
/// lenient mirror, so a cache written by another version is "no pull request".
#[derive(Debug, Default, Deserialize)]
struct CachedHosting {
    #[serde(default)]
    remote: String,
    #[serde(default)]
    prs: Vec<CachedPr>,
}

#[derive(Debug, Default, Deserialize)]
struct CachedPr {
    #[serde(default)]
    source_branch: String,
    #[serde(default)]
    target_branch: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    source_project: Option<String>,
}

/// The target branch of the open pull request whose source is `branch`, as
/// `(remote, branch)`, from the cache only.
fn pr_target(root: &Path, branch: &str) -> Option<(String, String)> {
    let state_dir = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().session.per_repo_state_dir.clone())
        .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
    let path = gitpoulpi_session::RepoState::hosting_cache_path_for(root, &state_dir);
    let cache: CachedHosting = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    let rank = |state: &str| match state {
        "open" => Some(0),
        "draft" => Some(1),
        _ => None,
    };
    cache
        .prs
        .iter()
        .filter(|pr| pr.source_branch == branch && pr.source_project.is_none())
        .filter_map(|pr| Some((rank(&pr.state)?, pr)))
        .min_by_key(|(rank, _)| *rank)
        .map(|(_, pr)| (cache.remote.clone(), pr.target_branch.clone()))
        .filter(|(_, target)| !target.is_empty())
}

/// What a request resolved to before any merge.
enum Resolved {
    Done(ConflictPrediction),
    Merge(Target),
}

struct Target {
    head: Oid,
    base: Oid,
    base_ref: String,
    source: BaseSource,
    /// HEAD already contains the base, or the base contains HEAD: nothing to
    /// merge, nothing can conflict.
    trivial: bool,
}

impl Target {
    fn key(&self) -> Key {
        (self.head, self.base)
    }

    /// The answer for these files: clean when there are none.
    fn outcome(&self, files: Vec<String>) -> ConflictPrediction {
        let status = if files.is_empty() {
            PredictionStatus::Clean
        } else {
            PredictionStatus::Conflicts
        };
        self.prediction(status, files)
    }

    fn prediction(&self, status: PredictionStatus, files: Vec<String>) -> ConflictPrediction {
        ConflictPrediction {
            status,
            reason: None,
            head: Some(self.head.to_hex()),
            base_ref: Some(self.base_ref.clone()),
            base: Some(self.base.to_hex()),
            base_source: Some(self.source),
            files,
        }
    }
}

fn status_only(status: PredictionStatus, reason: Option<&str>) -> ConflictPrediction {
    ConflictPrediction {
        status,
        reason: reason.map(str::to_string),
        ..Default::default()
    }
}

/// Steps 1 and 2: settings and base.
fn resolve_target(backend: &dyn GitBackend, repo: &RepoRef, slot: &Slot) -> Result<Resolved> {
    let distance = |tip: Oid, base: Oid| slot.distance(backend, repo, tip, base);
    let settings = read_settings(&repo.root);
    if !settings.enabled {
        return Ok(Resolved::Done(status_only(
            PredictionStatus::Disabled,
            Some("proactive conflict detection is off"),
        )));
    }
    let Ok(head) = backend.resolve(repo, "HEAD") else {
        // An unborn branch has nothing to compare.
        return Ok(Resolved::Done(status_only(PredictionStatus::NoBase, None)));
    };
    let refs = backend.refs(repo)?;
    let head_ref = refs.iter().find(|r| r.is_head);
    let head_branch = head_ref.and_then(|r| match &r.kind {
        RefKind::LocalBranch { name } => Some(name.clone()),
        _ => None,
    });
    // A branch name as it is spelled in `refs`: local `main`, remote `origin/main`.
    let named = |r: &gitpoulpi_core::domain::Ref| match &r.kind {
        RefKind::LocalBranch { name } => Some((name.clone(), false)),
        RefKind::RemoteBranch { remote, name } if name != "HEAD" => {
            Some((format!("{remote}/{name}"), true))
        }
        _ => None,
    };

    let mut chosen: Option<(String, Oid, BaseSource, u32, u32)> = None;
    if let Some(branch) = &head_branch {
        if let Some((remote, target)) = pr_target(&repo.root, branch) {
            let full = format!("{remote}/{target}");
            let found = [full.as_str(), target.as_str()].iter().find_map(|wanted| {
                refs.iter()
                    .find(|r| named(r).is_some_and(|(n, _)| n == *wanted))
                    .map(|r| ((*wanted).to_string(), r.target))
            });
            if let Some((name, oid)) = found {
                let (ahead, behind) = distance(head, oid)?;
                chosen = Some((name, oid, BaseSource::PullRequest, ahead, behind));
            }
        }
    }
    if chosen.is_none() {
        let upstream = head_ref.and_then(|r| r.upstream.clone());
        let mut allowed: Vec<(String, bool, Oid)> = refs
            .iter()
            .filter_map(|r| {
                let (name, remote) = named(r)?;
                // The branch itself and its own remote copy are not its base.
                let own = head_branch.as_deref().is_some_and(|b| {
                    name == b
                        || name
                            .split_once('/')
                            .is_some_and(|(_, rest)| rest == b && remote)
                }) || upstream.as_deref() == Some(name.as_str());
                (!own && base_allowed(&settings.globs, &name)).then_some((name, remote, r.target))
            })
            .collect();
        allowed.sort_by(|a, b| a.0.cmp(&b.0));
        allowed.truncate(MAX_BASE_CANDIDATES);
        let mut candidates = Vec::with_capacity(allowed.len());
        for (name, remote, oid) in &allowed {
            if let Ok((ahead, behind)) = distance(head, *oid) {
                candidates.push(BaseCandidate {
                    name: name.clone(),
                    remote: *remote,
                    ahead,
                    behind,
                });
            }
        }
        if let Some(best) = pick_base(&candidates) {
            if let Some((_, _, oid)) = allowed.iter().find(|(n, _, _)| *n == best.name) {
                chosen = Some((
                    best.name.clone(),
                    *oid,
                    BaseSource::Globs,
                    best.ahead,
                    best.behind,
                ));
            }
        }
    }
    let Some((base_ref, base, source, ahead, behind)) = chosen else {
        return Ok(Resolved::Done(status_only(PredictionStatus::NoBase, None)));
    };
    Ok(Resolved::Merge(Target {
        head,
        base,
        base_ref,
        source,
        trivial: ahead == 0 || behind == 0,
    }))
}

/// Merges for real (step 4), mapping an old git to "disabled".
fn run_merge(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    target: &Target,
) -> Result<ConflictPrediction> {
    if target.trivial {
        return Ok(target.prediction(PredictionStatus::Clean, Vec::new()));
    }
    match backend.predict_merge_conflicts(repo, target.head, target.base) {
        Ok(files) => Ok(target.outcome(files)),
        Err(Error::Unsupported { operation }) => {
            static LOGGED: AtomicBool = AtomicBool::new(false);
            if !LOGGED.swap(true, Ordering::Relaxed) {
                tracing::info!(
                    target: "daemon",
                    "proactive conflict detection is off: {operation}"
                );
            }
            Ok(status_only(
                PredictionStatus::Disabled,
                Some("needs git 2.38 or later"),
            ))
        }
        Err(e) => Err(e),
    }
}

type Key = (Oid, Oid);

#[derive(Default)]
struct Slot {
    /// Bumped by every debounced request: the highest one is the one still wanted.
    generation: AtomicU64,
    /// The files of the last merge, by key. Only the files: which branch the
    /// base came from is the caller's current answer, not the cache's.
    cache: Mutex<Option<(Key, Vec<String>)>>,
    /// One merge at a time for this repository.
    busy: tokio::sync::Mutex<()>,
    /// `(ahead, behind)` of a tip against a base, by their commits: finding the
    /// base walks the graph once per candidate branch, so a repeat request
    /// (a fetch that moved one branch, Settings closed) re-walks only what moved.
    distances: Mutex<HashMap<(Oid, Oid), (u32, u32)>>,
}

/// The distances kept per repository before the memo starts over.
const MAX_MEMO: usize = 4096;

impl Slot {
    fn distance(
        &self,
        backend: &dyn GitBackend,
        repo: &RepoRef,
        tip: Oid,
        base: Oid,
    ) -> Result<(u32, u32)> {
        let known = self
            .distances
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&(tip, base))
            .copied();
        if let Some(known) = known {
            return Ok(known);
        }
        let found = backend.ahead_behind(repo, tip, base)?;
        let mut memo = self.distances.lock().unwrap_or_else(|e| e.into_inner());
        if memo.len() >= MAX_MEMO {
            memo.clear();
        }
        memo.insert((tip, base), found);
        Ok(found)
    }

    fn cached(&self, target: &Target) -> Option<ConflictPrediction> {
        let cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        cache
            .as_ref()
            .filter(|(k, _)| *k == target.key())
            .map(|(_, files)| target.outcome(files.clone()))
    }

    fn store(&self, target: &Target, prediction: &ConflictPrediction) {
        *self.cache.lock().unwrap_or_else(|e| e.into_inner()) =
            Some((target.key(), prediction.files.clone()));
    }
}

/// The daemon's conflict predictions, one slot per repository.
#[derive(Default)]
pub struct Predictor {
    slots: Mutex<HashMap<String, Arc<Slot>>>,
    /// How many merges were run (not served from the cache): what a test or a
    /// perf measurement reads.
    merges: AtomicU64,
}

impl Predictor {
    fn slot(&self, repo: &RepoRef) -> Arc<Slot> {
        let mut slots = self.slots.lock().unwrap_or_else(|e| e.into_inner());
        Arc::clone(slots.entry(repo.id.0.clone()).or_default())
    }

    /// How many merges ran so far.
    pub fn merges_run(&self) -> u64 {
        self.merges.load(Ordering::Relaxed)
    }

    /// Answers now: no debounce, so no supersession.
    pub fn predict_now(
        &self,
        backend: &dyn GitBackend,
        repo: &RepoRef,
    ) -> Result<ConflictPrediction> {
        let slot = self.slot(repo);
        let target = match resolve_target(backend, repo, &slot)? {
            Resolved::Done(done) => return Ok(done),
            Resolved::Merge(target) => target,
        };
        if let Some(hit) = slot.cached(&target) {
            return Ok(hit);
        }
        // Blocking callers wait for a running merge of this repository.
        let _busy = loop {
            match slot.busy.try_lock() {
                Ok(guard) => break guard,
                Err(_) => std::thread::sleep(Duration::from_millis(5)),
            }
        };
        if let Some(hit) = slot.cached(&target) {
            return Ok(hit);
        }
        let prediction = run_merge(backend, repo, &target)?;
        if !target.trivial {
            self.merges.fetch_add(1, Ordering::Relaxed);
            slot.store(&target, &prediction);
        }
        Ok(prediction)
    }

    /// [`Self::predict_now`] after the repository has been quiet for
    /// `debounce`; a newer debounced request for it supersedes this one.
    pub async fn predict_debounced(
        &self,
        backend: Arc<dyn GitBackend>,
        repo: RepoRef,
        debounce: Duration,
    ) -> Result<ConflictPrediction> {
        let slot = self.slot(&repo);
        let generation = slot.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let current = |slot: &Slot| slot.generation.load(Ordering::SeqCst) == generation;
        let superseded = || status_only(PredictionStatus::Superseded, None);

        let resolve = {
            let (backend, repo, slot) = (Arc::clone(&backend), repo.clone(), Arc::clone(&slot));
            tokio::task::spawn_blocking(move || resolve_target(backend.as_ref(), &repo, &slot))
                .await
        };
        let target = match resolve.map_err(|e| Error::Backend(e.to_string()))?? {
            Resolved::Done(done) => return Ok(done),
            Resolved::Merge(target) => target,
        };
        if target.trivial {
            return Ok(target.outcome(Vec::new()));
        }
        if let Some(hit) = slot.cached(&target) {
            return Ok(hit);
        }
        tokio::time::sleep(debounce).await;
        if !current(&slot) {
            return Ok(superseded());
        }
        let _busy = slot.busy.lock().await;
        if !current(&slot) {
            return Ok(superseded());
        }
        if let Some(hit) = slot.cached(&target) {
            return Ok(hit);
        }
        let target = Arc::new(target);
        let merged = {
            let (backend, repo, target) = (Arc::clone(&backend), repo.clone(), Arc::clone(&target));
            tokio::task::spawn_blocking(move || run_merge(backend.as_ref(), &repo, &target)).await
        };
        let prediction = merged.map_err(|e| Error::Backend(e.to_string()))??;
        // Cached even when superseded meanwhile: it is right for its key.
        if matches!(
            prediction.status,
            PredictionStatus::Clean | PredictionStatus::Conflicts
        ) {
            self.merges.fetch_add(1, Ordering::Relaxed);
            slot.store(&target, &prediction);
        }
        Ok(if current(&slot) {
            prediction
        } else {
            superseded()
        })
    }
}

/// The daemon's one predictor.
pub fn global() -> &'static Predictor {
    static GLOBAL: OnceLock<Predictor> = OnceLock::new();
    GLOBAL.get_or_init(Predictor::default)
}

/// The debounce a request asked for (`args.debounce_ms`), when it wants one.
pub fn debounce_of(request: &Request) -> Option<Duration> {
    let ms = request.args.get("debounce_ms")?.as_u64()?;
    (ms > 0).then(|| Duration::from_millis(ms))
}

/// Answers a debounced request on its own task, so the connection's loop goes
/// straight back to reading and no repository lock is held while it waits.
pub fn spawn_debounced(
    backend: Arc<dyn GitBackend>,
    request: Request,
    debounce: Duration,
    out_tx: tokio::sync::mpsc::Sender<ServerMessage>,
) {
    tokio::spawn(async move {
        let id = request.id;
        let result = match crate::dispatch::repo_ref(&request) {
            Err(message) => Err(message),
            Ok(repo) => global()
                .predict_debounced(backend, repo, debounce)
                .await
                .map_err(|e| e.to_string())
                .and_then(|p| serde_json::to_value(p).map_err(|e| e.to_string())),
        };
        let result = result.map_err(|message| gitpoulpi_ipc::ResponseError {
            message,
            unsupported: false,
            kind: None,
            detail: None,
            auth_required: false,
        });
        let _ = out_tx
            .send(ServerMessage::Response(Response {
                request_id: id,
                result,
            }))
            .await;
    });
}
