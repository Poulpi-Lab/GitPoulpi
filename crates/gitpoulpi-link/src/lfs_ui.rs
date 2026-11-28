//! The pure half of the LFS state: the default switch, the per-repository
//! resolution, [`LfsSnapshot`] and the [`LfsGate`] that decides when a
//! snapshot is taken. No terminal types; re-exported by `lfs_ui`.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{
    LfsFile, LfsLock, LfsProgress, LfsRepoStatus, OperationKind, WorkingTree,
};

/// How often a repository that uses LFS is re-read without an event.
const LFS_POLL: Duration = Duration::from_secs(10);

/// `git.lfs_enabled` from `config.toml`, published like the other live
/// settings (`live_settings::install_auto_prune`).
static LFS_DEFAULT: AtomicBool = AtomicBool::new(true);

pub fn install_default(on: bool) {
    LFS_DEFAULT.store(on && crate::build_features::LFS, Ordering::Relaxed);
}

/// The repository's own `lfs_enabled` (Preferences, repo scope) if it set
/// one, else the global default.
pub fn resolved_enabled(repo_root: &Path) -> bool {
    let repo_choice =
        gitpoulpi_session::RepoState::load(&crate::repo_state_path::repo_state_path(repo_root))
            .ok()
            .and_then(|state| state.preferences.lfs_enabled);
    // A build without the `lfs` feature never shows or reads LFS state
    // (toolbar button, banners, file menus, refresh polling all follow this).
    crate::build_features::LFS && repo_choice.unwrap_or_else(|| LFS_DEFAULT.load(Ordering::Relaxed))
}

/// How often the server's locks are re-read, once the user has asked for
/// them (locking is a network round trip, so it is never polled for a
/// user who never looks).
const LOCKS_POLL: Duration = Duration::from_secs(30);

/// One read of the LFS state of a repository.
#[derive(Debug, Clone, Default)]
pub struct LfsSnapshot {
    /// `None` when the read failed: the UI then shows nothing about LFS
    /// rather than an error on every poll.
    pub status: Option<LfsRepoStatus>,
    pub files: Vec<LfsFile>,
    /// The server's locks, when they were read this time (`None`: not
    /// asked, keep the last list).
    pub locks: Option<Vec<LfsLock>>,
}

#[derive(Default)]
struct GateState {
    /// The very first refresh is left alone (see `LfsGate::due`).
    first_deferred: bool,
    fetched_once: bool,
    uses_lfs: bool,
    last: Option<Instant>,
    attributes_stamp: Vec<(bool, bool)>,
    /// The remote whose locks the user asked about, and whether a read is
    /// wanted now.
    locks_remote: Option<String>,
    locks_wanted: bool,
    locks_last: Option<Instant>,
    /// The latest report of an LFS transfer in flight (or just done).
    progress: Option<(OperationKind, LfsProgress)>,
    /// The daemon installed the LFS filters before a pull or push and the
    /// status line has not said so yet.
    posed_unsaid: bool,
    /// ...and the pull/push's own completion message has not carried the
    /// note yet.
    posed_note_due: bool,
}

/// Decides when the link thread takes an [`LfsSnapshot`].
#[derive(Default)]
pub struct LfsGate {
    dirty: AtomicBool,
    state: Mutex<GateState>,
}

impl LfsGate {
    /// An event or a manual refresh says the repository changed.
    pub fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// Is a snapshot due now? Consumes the dirty flag.
    ///
    /// **The very first refresh does not take one**: what a user waits for on
    /// opening a repository is the log and the status, and an LFS read is
    /// several process spawns (slow on Windows) that would delay them. The
    /// `LFS` tags and the banner appear one poll (2 s) later.
    pub fn due(&self) -> bool {
        let dirty = self.dirty.swap(false, Ordering::Relaxed);
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if !state.first_deferred {
            state.first_deferred = true;
            return dirty;
        }
        dirty
            || !state.fetched_once
            || (state.uses_lfs && state.last.is_none_or(|t| t.elapsed() >= LFS_POLL))
    }

    /// Records that a snapshot was taken.
    pub fn taken(&self, uses_lfs: bool) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.fetched_once = true;
        state.uses_lfs = uses_lfs;
        state.last = Some(Instant::now());
    }

    /// The user asked for (or changed) locks on `remote`: read them on the
    /// next refresh, and keep them fresh from then on.
    pub fn want_locks(&self, remote: String) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.locks_remote = Some(remote);
        state.locks_wanted = true;
    }

    /// The remote to read locks from, if a read is due. Consumes the wanted
    /// flag.
    pub fn locks_due(&self) -> Option<String> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let remote = state.locks_remote.clone()?;
        let due = std::mem::take(&mut state.locks_wanted)
            || state.locks_last.is_none_or(|t| t.elapsed() >= LOCKS_POLL);
        due.then_some(remote)
    }

    pub fn locks_taken(&self) {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .locks_last = Some(Instant::now());
    }

    /// Records a transfer report (called from the link's reader task).
    pub fn set_progress(&self, operation: OperationKind, progress: LfsProgress) {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .progress = Some((operation, progress));
        crate::redraw::poke();
    }

    /// `Event::LfsFiltersPosed` arrived (called from the reader task).
    pub fn set_filters_posed(&self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.posed_unsaid = true;
        state.posed_note_due = true;
        drop(state);
        crate::redraw::poke();
    }

    /// Whether the filters were posed and the status line has yet to say
    /// so (consumed).
    pub fn take_filters_posed(&self) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        std::mem::take(&mut state.posed_unsaid)
    }

    /// Whether the next completion message should carry the note
    /// (consumed).
    pub fn take_filters_posed_note(&self) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        std::mem::take(&mut state.posed_note_due)
    }

    pub fn progress(&self) -> Option<(OperationKind, LfsProgress)> {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .progress
    }

    /// Watches `.gitattributes` in the status: a change there is how a
    /// repository starts (or stops) using LFS, so it makes the next poll
    /// take a snapshot.
    pub fn observe_status(&self, status: &WorkingTree) {
        let stamp: Vec<(bool, bool)> = status
            .files
            .iter()
            .filter(|f| f.path.file_name().is_some_and(|n| n == ".gitattributes"))
            .map(|f| (f.index.is_some(), f.worktree.is_some()))
            .collect();
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.fetched_once && state.attributes_stamp != stamp {
            self.dirty.store(true, Ordering::Relaxed);
        }
        state.attributes_stamp = stamp;
    }
}
