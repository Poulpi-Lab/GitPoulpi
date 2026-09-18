//! What the TUI knows about Git LFS in the open repository, and how it
//! keeps that fresh without paying for it on every poll.
//!
//! - [`LfsSnapshot`]: one read of `lfs_status` + `lfs_files`, taken on the
//!   link thread and handed to the render loop inside a `RefreshOutcome`.
//! - [`LfsGate`]: decides *when* that read is worth its process spawns.
//!   The 2-second poll would otherwise run `git lfs version` and friends
//!   for every repository forever; instead a snapshot is taken on open, on
//!   any `RepoChanged` (every LFS action broadcasts one), on a manual
//!   refresh, when `.gitattributes` changes in the status, and -- only for
//!   a repository that does use LFS -- every ten seconds (an `lfs pull`
//!   from a terminal changes the "downloaded" marks without any event).
//! - [`LfsUi`]: the render loop's copy, with the lookups the panes need.
//!
//! Everything is gated on the resolved `git.lfs_enabled` preference
//! (per-repository override, then the global default): when off, the TUI
//! never asks about LFS at all.

use gitpoulpi_i18n::msg::lfs_ui as msg;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{
    human_size, LfsBinary, LfsFile, LfsLock, LfsProgress, LfsRepoStatus, OperationKind,
    WorkingTree, LFS_INSTALL_URL,
};

/// How often a repository that uses LFS is re-read without an event.
const LFS_POLL: Duration = Duration::from_secs(10);

/// `git.lfs_enabled` from `config.toml`, published like the other live
/// settings (`live_settings::install_auto_prune`).
static LFS_DEFAULT: AtomicBool = AtomicBool::new(true);

pub(crate) fn install_default(on: bool) {
    LFS_DEFAULT.store(on && crate::build_features::LFS, Ordering::Relaxed);
}

/// The repository's own `lfs_enabled` (Preferences, repo scope) if it set
/// one, else the global default.
pub(crate) fn resolved_enabled(repo_root: &Path) -> bool {
    let repo_choice =
        gitpoulpi_session::RepoState::load(&crate::remotes_ui::repo_state_path(repo_root))
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
pub(crate) struct LfsGate {
    dirty: AtomicBool,
    state: Mutex<GateState>,
}

impl LfsGate {
    /// An event or a manual refresh says the repository changed.
    pub(crate) fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// Is a snapshot due now? Consumes the dirty flag.
    ///
    /// **The very first refresh does not take one**: what a user waits for on
    /// opening a repository is the log and the status, and an LFS read is
    /// several process spawns (slow on Windows) that would delay them. The
    /// `LFS` tags and the banner appear one poll (2 s) later.
    pub(crate) fn due(&self) -> bool {
        let dirty = self.dirty.swap(false, Ordering::Relaxed);
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if !state.first_deferred {
            state.first_deferred = true;
            return dirty;
        }
        dirty
            || !state.fetched_once
            || (state.uses_lfs && state.last.map_or(true, |t| t.elapsed() >= LFS_POLL))
    }

    /// Records that a snapshot was taken.
    pub(crate) fn taken(&self, uses_lfs: bool) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.fetched_once = true;
        state.uses_lfs = uses_lfs;
        state.last = Some(Instant::now());
    }

    /// The user asked for (or changed) locks on `remote`: read them on the
    /// next refresh, and keep them fresh from then on.
    pub(crate) fn want_locks(&self, remote: String) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.locks_remote = Some(remote);
        state.locks_wanted = true;
    }

    /// The remote to read locks from, if a read is due. Consumes the wanted
    /// flag.
    pub(crate) fn locks_due(&self) -> Option<String> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let remote = state.locks_remote.clone()?;
        let due = std::mem::take(&mut state.locks_wanted)
            || state.locks_last.map_or(true, |t| t.elapsed() >= LOCKS_POLL);
        due.then_some(remote)
    }

    pub(crate) fn locks_taken(&self) {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .locks_last = Some(Instant::now());
    }

    /// Records a transfer report (called from the link's reader task).
    pub(crate) fn set_progress(&self, operation: OperationKind, progress: LfsProgress) {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .progress = Some((operation, progress));
        crate::redraw::poke();
    }

    /// `Event::LfsFiltersPosed` arrived (called from the reader task).
    pub(crate) fn set_filters_posed(&self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.posed_unsaid = true;
        state.posed_note_due = true;
        drop(state);
        crate::redraw::poke();
    }

    /// Whether the filters were posed and the status line has yet to say
    /// so (consumed).
    pub(crate) fn take_filters_posed(&self) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        std::mem::take(&mut state.posed_unsaid)
    }

    /// Whether the next completion message should carry the note
    /// (consumed).
    pub(crate) fn take_filters_posed_note(&self) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        std::mem::take(&mut state.posed_note_due)
    }

    pub(crate) fn progress(&self) -> Option<(OperationKind, LfsProgress)> {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .progress
    }

    /// Watches `.gitattributes` in the status: a change there is how a
    /// repository starts (or stops) using LFS, so it makes the next poll
    /// take a snapshot.
    pub(crate) fn observe_status(&self, status: &WorkingTree) {
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

/// One row of the toolbar's `LFS` dropdown. Each is
/// also a palette command and a bindable `Ctrl+G` action, so the dropdown
/// is never the only way in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LfsMenuItem {
    FetchAll,
    Checkout,
    Pull,
    PushAll,
    Prune,
}

impl LfsMenuItem {
    pub(crate) const ALL: [LfsMenuItem; 5] = [
        LfsMenuItem::FetchAll,
        LfsMenuItem::Checkout,
        LfsMenuItem::Pull,
        LfsMenuItem::PushAll,
        LfsMenuItem::Prune,
    ];

    pub(crate) fn label(self) -> gitpoulpi_i18n::M {
        match self {
            LfsMenuItem::FetchAll => msg::MENU_FETCH_ALL,
            LfsMenuItem::Checkout => msg::MENU_CHECKOUT_ALL,
            LfsMenuItem::Pull => msg::MENU_PULL_ALL,
            LfsMenuItem::PushAll => msg::MENU_PUSH_ALL,
            LfsMenuItem::Prune => msg::MENU_PRUNE,
        }
    }

    /// The palette command that runs the same thing.
    #[cfg(test)]
    pub(crate) fn command(self) -> crate::palette::PaletteCommand {
        use crate::palette::PaletteCommand as C;
        match self {
            LfsMenuItem::FetchAll => C::LfsFetchAll,
            LfsMenuItem::Checkout => C::LfsCheckout,
            LfsMenuItem::Pull => C::LfsPull,
            LfsMenuItem::PushAll => C::LfsPushAll,
            LfsMenuItem::Prune => C::LfsPrune,
        }
    }

    /// Submits the action (on the default `remote` when it needs one) and
    /// returns the in-flight status line. The same code serves the
    /// dropdown, the palette and the `Ctrl+G` chords.
    pub(crate) fn run(
        self,
        remote: Option<String>,
        link: &crate::daemon_link::DaemonLink,
    ) -> String {
        use crate::daemon_link::Action;
        let action = match (self, remote) {
            (LfsMenuItem::Checkout, _) => Action::LfsCheckout,
            (LfsMenuItem::Prune, _) => Action::LfsPrune,
            (_, None) => return gitpoulpi_i18n::msg::input::NO_DEFAULT_REMOTE.get().into(),
            (LfsMenuItem::FetchAll, Some(r)) => Action::LfsFetchAll(r),
            (LfsMenuItem::Pull, Some(r)) => Action::LfsPull(r),
            (LfsMenuItem::PushAll, Some(r)) => Action::LfsPushAll(r),
        };
        link.submit_action(action);
        match self {
            LfsMenuItem::FetchAll => msg::FETCHING_ALL,
            LfsMenuItem::Checkout => msg::CHECKING_OUT,
            LfsMenuItem::Pull => gitpoulpi_i18n::msg::input::LFS_PULLING,
            LfsMenuItem::PushAll => msg::PUSHING_ALL,
            LfsMenuItem::Prune => gitpoulpi_i18n::msg::input::LFS_PRUNING,
        }
        .get()
        .into()
    }
}

/// The status line of a pull or push's completion, with the note that the
/// daemon posed the LFS filters first.
pub(crate) fn with_posed_note(message: String) -> String {
    format!("{message} \u{b7} {}", msg::FILTERS_POSED.get())
}

/// `*.psd` for `art/hero.psd`; `None` for a file with no extension.
pub(crate) fn extension_pattern(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?;
    (!ext.is_empty()).then(|| format!("*.{ext}"))
}

/// A path as `.gitattributes` and `git lfs` want it: forward slashes.
pub(crate) fn pattern_for_file(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// The most rows the missing-program banner may take.
const BANNER_MAX_ROWS: usize = 3;

/// The banner text wrapped to `width` columns, at most
/// [`BANNER_MAX_ROWS`] rows (the last one ends in an ellipsis if the text
/// does not fit - the install link is at the very end of the text, so the
/// wrap keeps the *tail* visible by dropping words from the middle).
pub(crate) fn banner_rows(text: &str, width: u16) -> Vec<String> {
    let width = width as usize;
    if width < 8 {
        return Vec::new();
    }
    let mut rows: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        // A word longer than a row (the URL on a very narrow pane) is
        // cut rather than overflowing.
        let mut word = word.to_string();
        while crate::ui_kit::display_width(&word) > width {
            let cut = crate::ui_kit::take_width(&word, width);
            let taken = cut.chars().count().max(1);
            rows.push(cut);
            word = word.chars().skip(taken).collect();
        }
        match rows.last_mut() {
            Some(last)
                if crate::ui_kit::display_width(last) + 1 + crate::ui_kit::display_width(&word)
                    <= width =>
            {
                last.push(' ');
                last.push_str(&word);
            }
            _ => rows.push(word),
        }
    }
    if rows.len() > BANNER_MAX_ROWS {
        // Keep the head's first row and the tail (with the link).
        let tail = rows.split_off(rows.len() - (BANNER_MAX_ROWS - 1));
        rows.truncate(1);
        rows.extend(tail);
        if let Some(first) = rows.first_mut() {
            let keep = width.saturating_sub(1);
            let mut clipped = crate::ui_kit::take_width(first, keep);
            clipped.push('\u{2026}');
            *first = clipped;
        }
    }
    rows
}

/// Can a pane `avail` rows tall spare the banner's rows? Withheld on a
/// short pane, where the file lists matter more.
pub(crate) fn banner_height_ok(avail: u16) -> bool {
    avail >= 14
}

/// The status-line text for an LFS transfer report, e.g.
/// `LFS pull: 2/5 files, 1.2 MB of 4.0 MB (30%)` - plain ASCII.
pub(crate) fn progress_text(operation: OperationKind, p: &LfsProgress) -> String {
    let verb = match operation {
        OperationKind::LfsFetch => msg::VERB_FETCH,
        OperationKind::LfsPull => msg::VERB_PULL,
        OperationKind::LfsPush => msg::VERB_PUSH,
        OperationKind::LfsCheckout => msg::VERB_CHECKOUT,
        _ => msg::VERB_TRANSFER,
    }
    .get();
    if p.done {
        return msg::progress_done(p.files_total, verb);
    }
    msg::progress(
        verb,
        p.files_done,
        p.files_total,
        human_size(p.bytes_done),
        human_size(p.bytes_total),
        p.percent(),
    )
}

/// Git LFS is set up in this repository: its own `git lfs install --local`
/// hooks are in place. The filters alone do not count, they are usually the
/// user's global ones (`git lfs install` at user level), and then every
/// repository would look initialized.
pub(crate) fn initialized(status: &LfsRepoStatus) -> bool {
    status.hooks_installed
}

/// The toolbar's `LFS` button is worth showing: the repository is initialized,
/// or its `.gitattributes` already routes files through LFS (the menu is how
/// its large files get fetched, whatever the hooks say).
pub(crate) fn shows_toolbar_button(status: &LfsRepoStatus) -> bool {
    initialized(status) || status.uses_lfs
}

/// The render loop's copy of the LFS state.
#[derive(Debug, Default)]
pub(crate) struct LfsUi {
    /// The resolved `lfs_enabled` preference.
    pub(crate) enabled: bool,
    pub(crate) status: Option<LfsRepoStatus>,
    files: HashMap<PathBuf, LfsFile>,
    /// Locks on the server (filled by the lock actions).
    pub(crate) locks: Vec<LfsLock>,
}

impl LfsUi {
    /// Nothing read yet, with the resolved preference.
    pub(crate) fn new(enabled: bool) -> Self {
        Self {
            enabled,
            ..Self::default()
        }
    }

    /// Takes a fresh snapshot, if the refresh carried one.
    pub(crate) fn apply(&mut self, snapshot: Option<LfsSnapshot>) {
        if let Some(snapshot) = snapshot {
            if let Some(locks) = snapshot.locks {
                self.locks = locks;
            }
            self.status = snapshot.status;
            self.files = snapshot
                .files
                .into_iter()
                .map(|f| (f.path.clone(), f))
                .collect();
        }
    }

    /// The toolbar button hides until the repository's LFS state is known and
    /// says it is initialized or in use: a plain repository never shows it
    /// (the init command stays in the palette and on the LFS settings page).
    pub(crate) fn toolbar_hidden(&self) -> bool {
        !self.enabled || !self.status.as_ref().is_some_and(shows_toolbar_button)
    }

    /// Everything is off (setting) or unknown: draw nothing LFS.
    fn active(&self) -> bool {
        self.enabled && self.status.is_some()
    }

    /// The pointer file at `path` in the index, if LFS is shown.
    pub(crate) fn file(&self, path: &Path) -> Option<&LfsFile> {
        if self.enabled {
            self.files.get(path)
        } else {
            None
        }
    }

    pub(crate) fn is_lfs(&self, path: &Path) -> bool {
        self.file(path).is_some()
    }

    /// Whether the repository uses LFS but the program cannot be used --
    /// the banner's condition.
    pub(crate) fn program_unavailable(&self) -> bool {
        self.active()
            && self
                .status
                .as_ref()
                .is_some_and(|s| s.uses_lfs && !matches!(s.binary, LfsBinary::Installed { .. }))
    }

    /// The one-line banner for a repository that needs LFS and does not
    /// have it. The link is shown as text and also opened by the button.
    pub(crate) fn banner_text(&self) -> Option<String> {
        self.program_unavailable()
            .then(|| msg::banner(LFS_INSTALL_URL))
    }

    /// Whether the LFS actions are usable, and if not why (for the
    /// palette's disabled entries).
    pub(crate) fn unavailable_reason(&self) -> Option<String> {
        match self.status.as_ref().map(|s| &s.binary) {
            Some(LfsBinary::Installed { .. }) | None => None,
            Some(other) => Some(match other {
                LfsBinary::TooOld { version } => msg::too_old(version),
                _ => msg::not_installed(LFS_INSTALL_URL),
            }),
        }
    }

    /// The LFS rows a file's right-click menu adds (none when LFS is off
    /// or the program is unusable): track it, or - for a file already
    /// stored with LFS - take, release or break its lock.
    pub(crate) fn file_menu_actions(
        &self,
        path: &Path,
    ) -> Vec<crate::status_pane::FileContextAction> {
        use crate::status_pane::FileContextAction as A;
        if !self.enabled || self.status.is_none() || self.unavailable_reason().is_some() {
            return Vec::new();
        }
        if !self.is_lfs(path) {
            let mut actions = vec![A::LfsTrackFile];
            if extension_pattern(path).is_some() {
                actions.push(A::LfsTrackExtension);
            }
            return actions;
        }
        match self.lock_on(path) {
            None => vec![A::LfsLock],
            Some(lock) if lock.ours => vec![A::LfsUnlock],
            Some(_) => vec![A::LfsForceUnlock],
        }
    }

    /// `path` is locked by somebody, for the row's text.
    pub(crate) fn lock_on(&self, path: &Path) -> Option<&LfsLock> {
        if self.enabled {
            self.locks.iter().find(|l| l.path == path)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{FileStatus, LfsPointer, StatusEntry};

    fn repo_status(hooks: bool, uses: bool) -> LfsRepoStatus {
        LfsRepoStatus {
            binary: gitpoulpi_core::domain::LfsBinary::Missing,
            uses_lfs: uses,
            filters_configured: true,
            hooks_installed: hooks,
            patterns: Vec::new(),
        }
    }

    /// "Initialized" is this repository's own hooks; the toolbar button also
    /// shows for a repository whose attributes use LFS, and stays hidden for
    /// a plain one (even with global filters) and while the state is unread.
    #[test]
    fn the_toolbar_button_follows_the_repositorys_lfs_state() {
        let mut ui = LfsUi::new(true);
        assert!(ui.toolbar_hidden(), "unread state: hidden");
        for (hooks, uses, hidden) in [
            (false, false, true),
            (true, false, false),
            (false, true, false),
            (true, true, false),
        ] {
            ui.status = Some(repo_status(hooks, uses));
            assert_eq!(ui.toolbar_hidden(), hidden, "hooks={hooks} uses={uses}");
        }
        assert!(!initialized(&repo_status(false, true)));
        assert!(initialized(&repo_status(true, false)));
        ui.enabled = false;
        assert!(ui.toolbar_hidden(), "turned off: hidden");
    }

    fn file(path: &str, local: bool) -> LfsFile {
        LfsFile {
            path: path.into(),
            pointer: LfsPointer {
                oid: "a".repeat(64),
                size: 1234,
            },
            local,
        }
    }

    fn status(binary: LfsBinary, uses_lfs: bool) -> LfsRepoStatus {
        LfsRepoStatus {
            binary,
            uses_lfs,
            filters_configured: true,
            hooks_installed: true,
            patterns: Vec::new(),
        }
    }

    #[test]
    fn lookups_answer_only_when_enabled() {
        let mut ui = LfsUi {
            enabled: true,
            ..LfsUi::default()
        };
        ui.apply(Some(LfsSnapshot {
            status: Some(status(LfsBinary::Missing, true)),
            files: vec![file("a.bin", false)],
            locks: None,
        }));
        assert!(ui.is_lfs(Path::new("a.bin")));
        assert!(!ui.is_lfs(Path::new("b.txt")));
        ui.enabled = false;
        assert!(!ui.is_lfs(Path::new("a.bin")), "off means nothing shows");
        assert!(ui.banner_text().is_none());
    }

    #[test]
    fn the_banner_needs_lfs_use_and_an_unusable_program() {
        let mut ui = LfsUi {
            enabled: true,
            ..LfsUi::default()
        };
        assert!(ui.banner_text().is_none(), "nothing known yet");
        for (binary, uses, banner) in [
            (LfsBinary::Missing, true, true),
            (LfsBinary::Missing, false, false),
            (
                LfsBinary::TooOld {
                    version: "2.0.0".into(),
                },
                true,
                true,
            ),
            (
                LfsBinary::Installed {
                    version: "3.5.1".into(),
                },
                true,
                false,
            ),
        ] {
            ui.apply(Some(LfsSnapshot {
                status: Some(status(binary, uses)),
                files: vec![],
                locks: None,
            }));
            assert_eq!(ui.banner_text().is_some(), banner);
        }
        ui.apply(Some(LfsSnapshot {
            status: Some(status(LfsBinary::Missing, true)),
            files: vec![],
            locks: None,
        }));
        assert!(ui.banner_text().unwrap().contains("git-lfs.com"));
        assert!(ui.unavailable_reason().unwrap().contains("git-lfs.com"));
    }

    #[test]
    fn a_refresh_without_a_snapshot_keeps_the_last_one() {
        let mut ui = LfsUi {
            enabled: true,
            ..LfsUi::default()
        };
        ui.apply(Some(LfsSnapshot {
            status: Some(status(LfsBinary::Missing, true)),
            files: vec![file("a.bin", true)],
            locks: None,
        }));
        ui.apply(None);
        assert!(ui.is_lfs(Path::new("a.bin")));
    }

    #[test]
    fn the_gate_is_due_once_then_only_on_events_or_the_slow_poll() {
        let gate = LfsGate::default();
        assert!(
            !gate.due(),
            "the very first refresh is left alone (open stays fast)"
        );
        assert!(gate.due(), "the next one reads");
        gate.taken(false);
        assert!(!gate.due(), "a repository without LFS is left alone");
        gate.mark_dirty();
        assert!(gate.due(), "an event makes it due");
        assert!(!gate.due(), "...once");
        gate.taken(true);
        assert!(!gate.due(), "a fresh snapshot is not re-read at once");
    }

    #[test]
    fn a_gitattributes_change_in_the_status_makes_the_gate_due() {
        let gate = LfsGate::default();
        gate.observe_status(&WorkingTree::default());
        assert!(!gate.due(), "deferred");
        assert!(gate.due());
        gate.taken(false);

        let with_attributes = WorkingTree {
            files: vec![StatusEntry {
                path: ".gitattributes".into(),
                index: None,
                worktree: Some(FileStatus::Untracked),
            }],
        };
        gate.observe_status(&with_attributes);
        assert!(gate.due(), "a new .gitattributes may start LFS use");
        gate.taken(true);
        gate.observe_status(&with_attributes);
        assert!(!gate.due(), "an unchanged status is not a reason");
    }

    #[test]
    fn the_banner_wraps_within_the_width_and_keeps_the_link() {
        let text = LfsUi {
            enabled: true,
            status: Some(status(LfsBinary::Missing, true)),
            ..LfsUi::default()
        }
        .banner_text()
        .unwrap();
        for width in [20u16, 30, 40, 80] {
            let rows = banner_rows(&text, width);
            assert!(
                !rows.is_empty() && rows.len() <= BANNER_MAX_ROWS,
                "{width}: {rows:?}"
            );
            assert!(
                rows.iter()
                    .all(|r| crate::ui_kit::display_width(r) <= width as usize),
                "{width}: {rows:?}"
            );
            assert!(
                rows.join(" ").contains("git-lfs.com"),
                "the install link must survive the wrap at width {width}: {rows:?}"
            );
        }
        assert!(!banner_height_ok(10), "no room, no banner");
        assert!(banner_height_ok(30));
    }

    #[test]
    fn the_file_menu_offers_track_for_plain_files_and_the_right_lock_action_for_lfs_ones() {
        use crate::status_pane::FileContextAction as A;
        let mut ui = LfsUi {
            enabled: true,
            ..LfsUi::default()
        };
        ui.apply(Some(LfsSnapshot {
            status: Some(status(
                LfsBinary::Installed {
                    version: "3.5.1".into(),
                },
                true,
            )),
            files: vec![
                file("art/a.psd", true),
                file("art/b.psd", true),
                file("art/c.psd", true),
            ],
            locks: Some(vec![
                LfsLock {
                    id: "1".into(),
                    path: "art/b.psd".into(),
                    owner: Some("me".into()),
                    locked_at: String::new(),
                    ours: true,
                },
                LfsLock {
                    id: "2".into(),
                    path: "art/c.psd".into(),
                    owner: Some("you".into()),
                    locked_at: String::new(),
                    ours: false,
                },
            ]),
        }));
        assert_eq!(
            ui.file_menu_actions(Path::new("notes/readme.md")),
            [A::LfsTrackFile, A::LfsTrackExtension]
        );
        assert_eq!(
            ui.file_menu_actions(Path::new("LICENSE")),
            [A::LfsTrackFile]
        );
        assert_eq!(ui.file_menu_actions(Path::new("art/a.psd")), [A::LfsLock]);
        assert_eq!(ui.file_menu_actions(Path::new("art/b.psd")), [A::LfsUnlock]);
        assert_eq!(
            ui.file_menu_actions(Path::new("art/c.psd")),
            [A::LfsForceUnlock]
        );

        ui.enabled = false;
        assert!(ui.file_menu_actions(Path::new("art/a.psd")).is_empty());
        ui.enabled = true;
        ui.apply(Some(LfsSnapshot {
            status: Some(status(LfsBinary::Missing, true)),
            files: vec![],
            locks: None,
        }));
        assert!(
            ui.file_menu_actions(Path::new("readme.md")).is_empty(),
            "without the program there is nothing to offer"
        );
    }

    #[test]
    fn patterns_are_built_the_way_gitattributes_wants_them() {
        assert_eq!(
            extension_pattern(Path::new("a/b.PSD")).as_deref(),
            Some("*.PSD")
        );
        assert_eq!(extension_pattern(Path::new("Makefile")), None);
        assert_eq!(pattern_for_file(Path::new("a\\b\\c.bin")), "a/b/c.bin");
    }
}
