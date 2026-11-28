//! Git LFS in the window: the toolbar button's menu, fetch, pull, push, check out and prune, tracking
//! patterns and files, locks, and the "LFS only" mode of the working-changes lists.
//!
//! The state is read on demand (`LfsStatus`, `LfsFiles`, and the locks once asked for) and kept in
//! [`LfsView`]; the daemon pushes a fresh snapshot when `.gitattributes` or the store changes
//! (`RefreshOutcome::lfs`, taken in `live.rs`). Each command submits one action, so one undo entry (the
//! network ones are not undoable: the daemon says so itself).

use std::collections::HashSet;

use gitpoulpi_core::domain::{LfsBinary, LfsFile, LfsLock, LfsRepoStatus, Remote, LFS_INSTALL_URL};
use gitpoulpi_i18n::msg;
use gitpoulpi_link::lfs_ui::LfsSnapshot;
use gitpoulpi_link::Action;
use gpui_kit::{Context, Window};

use super::Workspace;
use crate::app::actions::with_workspace;
use crate::commands::file_ops;
use crate::commands::repo_ops::RepoOp;
use crate::daemon::repo_ops as rules;
use crate::dialogs::{self, Choice, Spec};
use crate::form_dialog::{self, Field, FormSpec};
use crate::ref_target::{FileSide, RefTarget};
use crate::repo_menu;
use crate::strings;

/// What the window knows about LFS in the repository.
#[derive(Clone, Debug, Default)]
pub struct LfsView {
    pub status: Option<LfsRepoStatus>,
    pub files: Vec<LfsFile>,
    /// The server's locks, once they were read.
    pub locks: Vec<LfsLock>,
}

impl LfsView {
    pub fn from_snapshot(s: LfsSnapshot, keep_locks: &[LfsLock]) -> Self {
        Self {
            status: s.status,
            files: s.files,
            locks: s.locks.unwrap_or_else(|| keep_locks.to_vec()),
        }
    }

    /// Why git-lfs cannot run, `None` when it can.
    pub fn unusable(&self) -> Option<String> {
        match self.status.as_ref().map(|s| &s.binary) {
            Some(LfsBinary::Installed { .. }) | None => None,
            Some(LfsBinary::TooOld { version }) => Some(msg::lfs_ui::too_old(version)),
            Some(LfsBinary::Missing) => Some(msg::lfs_ui::not_installed(LFS_INSTALL_URL)),
        }
    }

    pub fn is_lfs(&self, path: &str) -> bool {
        self.files
            .iter()
            .any(|f| f.path == std::path::Path::new(path))
    }

    fn lock_on(&self, path: &str) -> Option<&LfsLock> {
        self.locks
            .iter()
            .find(|l| l.path == std::path::Path::new(path))
    }

    /// The LFS rows a file's menu gets: track it, or take, release or break its lock.
    pub fn file_rows(&self, path: &str) -> Vec<RepoOp> {
        if self.status.is_none() || self.unusable().is_some() {
            return Vec::new();
        }
        if !self.is_lfs(path) {
            let mut rows = vec![RepoOp::LfsTrackFile];
            if rules::ext_of(path).is_some() {
                rows.push(RepoOp::LfsTrackExtension);
            }
            return rows;
        }
        match self.lock_on(path) {
            None => vec![RepoOp::LfsLock],
            Some(l) if l.ours => vec![RepoOp::LfsUnlock],
            Some(_) => vec![RepoOp::LfsForceUnlock],
        }
    }
}

impl Workspace {
    /// The LFS state as last read.
    pub fn lfs_view(&self) -> Option<&LfsView> {
        self.repo.lfs.as_ref()
    }

    /// Whether the working-changes lists show only the LFS files.
    pub fn lfs_only(&self) -> bool {
        self.repo.lfs_only
    }

    /// Takes a snapshot the daemon pushed.
    pub(in crate::app) fn set_lfs_snapshot(
        &mut self,
        snapshot: LfsSnapshot,
        cx: &mut Context<Self>,
    ) {
        let keep = self
            .repo
            .lfs
            .as_ref()
            .map(|v| v.locks.clone())
            .unwrap_or_default();
        self.repo.lfs = Some(LfsView::from_snapshot(snapshot, &keep));
        self.apply_lfs_only(cx);
    }

    /// Reads status and files now, then runs `then`.
    pub(in crate::app) fn read_lfs(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        then: impl FnOnce(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) {
        let (status, files) = (self.backend.lfs_status(), self.backend.lfs_files());
        cx.spawn_in(window, async move |this, cx| {
            let status = status.await.ok();
            let files = files.await.unwrap_or_default();
            let _ = this.update_in(cx, |this, window, cx| {
                let locks = this
                    .repo
                    .lfs
                    .as_ref()
                    .map(|v| v.locks.clone())
                    .unwrap_or_default();
                this.repo.lfs = Some(LfsView {
                    status,
                    files,
                    locks,
                });
                this.apply_lfs_only(cx);
                then(this, window, cx);
            });
        })
        .detach();
    }

    /// Reads the LFS state in the background (opening a repository).
    pub(in crate::app) fn refresh_lfs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.read_lfs(window, cx, |_, _, _| {});
    }

    /// The remote LFS transfers use: `origin`, else the only one.
    fn lfs_remote(remotes: &[Remote]) -> Option<String> {
        remotes
            .iter()
            .find(|r| r.name == "origin")
            .or(if remotes.len() == 1 {
                remotes.first()
            } else {
                None
            })
            .map(|r| r.name.clone())
    }

    /// Reads the remotes, then runs `then` with the LFS remote (or says there is none).
    fn with_lfs_remote(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        then: impl FnOnce(&mut Self, String, &mut Window, &mut Context<Self>) + 'static,
    ) {
        let remotes = self.backend.remotes();
        cx.spawn_in(window, async move |this, cx| {
            let remotes = remotes.await.unwrap_or_default();
            let _ = this.update_in(cx, |this, window, cx| match Self::lfs_remote(&remotes) {
                Some(remote) => then(this, remote, window, cx),
                None => this.say(msg::input::NO_DEFAULT_REMOTE.get(), cx),
            });
        })
        .detach();
    }

    fn submit_lfs(&mut self, action: Action, said: impl Into<String>, cx: &mut Context<Self>) {
        self.backend.submit(action);
        self.say(said, cx);
    }

    /// The file a file-level LFS command acts on: the menu's, else the selected one.
    fn lfs_file_target(&self, target: Option<RefTarget>, cx: &gpui_kit::App) -> Option<String> {
        match target {
            Some(RefTarget::File(path, _)) => Some(path),
            _ => self.right.read(cx).current_file().map(|(p, _)| p),
        }
    }

    pub(in crate::app) fn run_lfs_op(
        &mut self,
        op: RepoOp,
        target: Option<RefTarget>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use msg::lfs_ui as m;
        match op {
            RepoOp::LfsMenu => {
                let at = window.mouse_position();
                self.read_lfs(window, cx, move |this, window, cx| {
                    let view = this.repo.lfs.clone().unwrap_or_default();
                    let items = repo_menu::lfs_items(view.status.as_ref(), view.unusable());
                    this.show_items(items, RefTarget::Worktree(String::new()), at, window, cx);
                });
            }
            RepoOp::LfsInit => {
                self.submit_lfs(Action::LfsInstall, msg::input::LFS_SETTING_UP.get(), cx)
            }
            RepoOp::LfsCheckout => self.submit_lfs(Action::LfsCheckout, m::CHECKING_OUT.get(), cx),
            RepoOp::LfsPrune => {
                self.submit_lfs(Action::LfsPrune, msg::input::LFS_PRUNING.get(), cx)
            }
            RepoOp::LfsFetch
            | RepoOp::LfsFetchAll
            | RepoOp::LfsPull
            | RepoOp::LfsPush
            | RepoOp::LfsPushAll => {
                self.with_lfs_remote(window, cx, move |this, remote, _, cx| {
                    let (action, said) = match op {
                        RepoOp::LfsFetch => {
                            (Action::LfsFetch(remote), msg::input::LFS_FETCHING.get())
                        }
                        RepoOp::LfsFetchAll => (Action::LfsFetchAll(remote), m::FETCHING_ALL.get()),
                        RepoOp::LfsPull => (Action::LfsPull(remote), msg::input::LFS_PULLING.get()),
                        RepoOp::LfsPush => (Action::LfsPush(remote), msg::input::LFS_PUSHING.get()),
                        _ => (Action::LfsPushAll(remote), m::PUSHING_ALL.get()),
                    };
                    this.submit_lfs(action, said, cx);
                });
            }
            RepoOp::LfsTrackPattern | RepoOp::LfsUntrackPattern => {
                self.pattern_form(op == RepoOp::LfsTrackPattern, window, cx)
            }
            RepoOp::LfsShowLocks => self.show_locks(window, cx),
            RepoOp::LfsInstallPage => {
                match file_ops::spawn(
                    &file_ops::default_program_argv(std::path::Path::new(LFS_INSTALL_URL)),
                    None,
                ) {
                    Ok(()) => self.say(strings::launched(LFS_INSTALL_URL), cx),
                    Err(why) => crate::commands::status::set_error(
                        cx,
                        strings::launch_failed(LFS_INSTALL_URL, &why),
                    ),
                }
            }
            RepoOp::LfsOnly => self.toggle_lfs_only(window, cx),
            RepoOp::LfsTrackFile | RepoOp::LfsTrackExtension => {
                let Some(path) = self.lfs_file_target(target, cx) else {
                    return self.say(strings::SELECT_FILE, cx);
                };
                let pattern = if op == RepoOp::LfsTrackFile {
                    Some(rules::lfs_file_pattern(std::path::Path::new(&path)))
                } else {
                    rules::lfs_extension_pattern(std::path::Path::new(&path))
                };
                match pattern {
                    Some(p) => {
                        let said = msg::input::lfs_tracking(&p);
                        self.submit_lfs(rules::lfs_track_action(vec![p]), said, cx);
                    }
                    None => self.say(msg::status_list::NO_EXTENSION.get(), cx),
                }
            }
            RepoOp::LfsLock | RepoOp::LfsUnlock | RepoOp::LfsForceUnlock => {
                let Some(path) = self.lfs_file_target(target, cx) else {
                    return self.say(strings::SELECT_FILE, cx);
                };
                if op == RepoOp::LfsForceUnlock {
                    return self.ask_force_unlock(path, window, cx);
                }
                self.lock_file(path, op == RepoOp::LfsLock, false, window, cx);
            }
            _ => {}
        }
    }

    fn lock_file(
        &mut self,
        path: String,
        lock: bool,
        force: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.with_lfs_remote(window, cx, move |this, remote, _, cx| {
            let said = msg::input::LFS_LOCKS.get();
            let file = std::path::PathBuf::from(&path);
            let action = if lock {
                Action::LfsLock { remote, path: file }
            } else {
                Action::LfsUnlock {
                    remote,
                    path: file,
                    force,
                }
            };
            this.submit_lfs(action, said, cx);
            // The locks are read again, so the file's menu shows its new state.
            if let Some(v) = this.repo.lfs.as_mut() {
                v.locks.clear();
            }
        });
    }

    fn ask_force_unlock(&mut self, path: String, window: &mut Window, cx: &mut Context<Self>) {
        let spec = Spec {
            title: msg::status_pane::FA_LFS_FORCE_UNLOCK.get().to_string(),
            lines: vec![path.clone()],
            choices: vec![
                Choice::destructive(msg::status_pane::FA_LFS_FORCE_UNLOCK.get()),
                Choice::new(msg::dialog::CANCEL.get()),
            ],
            hint: msg::diff_actions::CONFIRM_HINT.get().to_string(),
            highlight_first: false,
            yes_no_keys: true,
        };
        dialogs::open(
            window,
            cx,
            spec,
            move |ix, cx| {
                if ix == 0 {
                    with_workspace(cx, move |ws, window, cx| {
                        ws.lock_file(path, false, true, window, cx)
                    });
                }
            },
            || {},
        );
    }

    /// The Track / Untrack pattern forms.
    fn pattern_form(&mut self, track: bool, window: &mut Window, cx: &mut Context<Self>) {
        let patterns: Vec<String> = self
            .repo
            .lfs
            .as_ref()
            .and_then(|v| v.status.as_ref())
            .map(|s| s.patterns.iter().map(|p| p.pattern.clone()).collect())
            .unwrap_or_default();
        let (title, button) = if track {
            (
                msg::palette::LFS_TRACK_PATTERN.get(),
                msg::palette::LFS_TRACK_PATTERN.get(),
            )
        } else {
            (
                msg::palette::LFS_UNTRACK_PATTERN.get(),
                msg::palette::LFS_UNTRACK_PATTERN.get(),
            )
        };
        let spec = FormSpec {
            title: title.trim_end_matches("...").to_string(),
            intro: if track {
                Vec::new()
            } else {
                patterns
                    .iter()
                    .map(|p| strings::lfs_pattern_line(p, false))
                    .collect()
            },
            fields: vec![Field::text(strings::LFS_PATTERN_LABEL, "*.psd", "")],
            submit: button
                .trim_end_matches("...")
                .trim_start_matches("LFS: ")
                .to_string(),
            cancel: msg::dialog::CANCEL.get().to_string(),
        };
        form_dialog::open(
            window,
            cx,
            spec,
            |v| rules::validate_lfs_pattern(v.text(0)).map(|_| ()),
            move |v, _, cx| {
                let pattern = v.text(0).to_string();
                with_workspace(cx, move |ws, _, cx| {
                    if track {
                        let said = msg::input::lfs_tracking(&pattern);
                        ws.submit_lfs(rules::lfs_track_action(vec![pattern]), said, cx);
                    } else {
                        let said = msg::input::lfs_untracking(&pattern);
                        ws.submit_lfs(Action::LfsUntrack(vec![pattern]), said, cx);
                    }
                });
            },
        );
    }

    /// Reads the server's locks and lists them.
    fn show_locks(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.with_lfs_remote(window, cx, |this, remote, window, cx| {
            this.say(msg::input::LFS_LOCKS.get(), cx);
            let locks = this.backend.lfs_locks(&remote);
            cx.spawn_in(window, async move |this, cx| {
                let locks = locks.await;
                let _ = this.update_in(cx, |this, window, cx| {
                    crate::commands::status::clear(cx);
                    match locks {
                        Ok(list) => this.list_locks(list, window, cx),
                        Err(why) => crate::commands::status::set_error(cx, why),
                    }
                });
            })
            .detach();
        });
    }

    fn list_locks(&mut self, list: Vec<LfsLock>, window: &mut Window, cx: &mut Context<Self>) {
        use msg::status_list as m;
        let lines: Vec<String> = list
            .iter()
            .map(|l| {
                let who = if l.ours {
                    m::LOCK_YOU.get().to_string()
                } else {
                    l.owner
                        .clone()
                        .unwrap_or_else(|| m::LOCK_SOMEONE.get().to_string())
                };
                format!("{}{}", l.path.display(), m::locked_by(who))
            })
            .collect();
        if let Some(view) = self.repo.lfs.as_mut() {
            view.locks = list;
        }
        let spec = Spec {
            title: msg::palette::LFS_SHOW_LOCKS.get().to_string(),
            lines,
            choices: vec![Choice::new(strings::CLOSE)],
            hint: String::new(),
            highlight_first: true,
            yes_no_keys: false,
        };
        dialogs::open(window, cx, spec, |_, _| {}, || {});
    }

    // -- LFS only ----------------------------------------------------------------------------------------

    fn toggle_lfs_only(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.repo.lfs_only {
            self.repo.lfs_only = false;
            return self.apply_lfs_only(cx);
        }
        self.read_lfs(window, cx, |this, _, cx| {
            let has = this.repo.lfs.as_ref().is_some_and(|v| !v.files.is_empty());
            if !has {
                return this.say(msg::palette::WHY_THIS_REPOSITORY_HAS_NO_LFS_FILES.get(), cx);
            }
            this.repo.lfs_only = true;
            this.apply_lfs_only(cx);
        });
    }

    /// Hands the right pane the set of LFS files it shows, or lets it show everything.
    pub(in crate::app) fn apply_lfs_only(&mut self, cx: &mut Context<Self>) {
        let only: Option<HashSet<String>> = (self.repo.lfs_only)
            .then(|| {
                self.repo.lfs.as_ref().map(|v| {
                    v.files
                        .iter()
                        .map(|f| f.path.to_string_lossy().replace('\\', "/"))
                        .collect()
                })
            })
            .flatten();
        self.right.update(cx, |r, cx| r.set_lfs_only(only, cx));
    }

    /// The side a file's row is on, for the menu's LFS rows.
    pub(in crate::app) fn lfs_rows_for(&self, path: &str, side: FileSide) -> Vec<RepoOp> {
        if side == FileSide::Commit {
            return Vec::new();
        }
        self.repo
            .lfs
            .as_ref()
            .map(|v| v.file_rows(path))
            .unwrap_or_default()
    }
}
