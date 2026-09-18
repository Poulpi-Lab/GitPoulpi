//! `editor.follow_selection`: while the editor is open, selecting another
//! file in the right pane (a click, Enter, the palette - whichever path
//! opens a diff or File View for it) opens that file in the editor instead
//! of leaving the editor over the new file's view.
//!
//! One place, not one per call site: every selection path ends by pointing
//! the view under the editor at the new file, so the pre-frame step notices
//! that the editor's file and the view's file differ. The view underneath is
//! therefore already the new file's normal view - closing the editor lands on
//! it - and a file that cannot be edited simply leaves that view showing.
//!
//! Unsaved changes: the editor raises its Save / Discard / Cancel prompt
//! (`Editor::ask_follow`). Cancel puts the view back to what it showed
//! (`HostSnapshot`, taken every frame the two agreed) and re-selects that
//! file's row, so the pane and the editor never disagree.

use crate::event_loop::*;
use init::WindowState;

use crate::editor::FollowAnswer;
use crate::file_panel::editor_host_path;

/// The view under the editor while it showed the edited file: what Cancel
/// puts back.
#[derive(Debug, Clone)]
pub(in crate::event_loop) enum HostSnapshot {
    Diff(DiffViewState),
    File(FileInsightRequest),
}

impl WindowState {
    pub(in crate::event_loop) fn follow_editor_selection(&mut self) {
        let Some(ed) = self.editor.as_mut() else {
            self.editor_host_snapshot = None;
            return;
        };
        let host = editor_host_path(self.file_insight.as_ref(), self.diff_view_state.as_ref())
            .map(Path::to_path_buf);
        // A pick in a commit's (or a comparison's) file list is not a pick of
        // the working copy the editor edits: the editor is left and that
        // file's view shown, as from a diff.
        let host_is_commit = match self.file_insight.as_ref() {
            Some(f) => !f.request.worktree,
            None => self
                .diff_view_state
                .as_ref()
                .is_some_and(|d| !matches!(d.source, DiffSource::Unstaged | DiffSource::Staged)),
        };
        match ed.take_follow_answer() {
            Some(FollowAnswer::Stay) => {
                self.restore_editor_host();
                return;
            }
            Some(FollowAnswer::Switch) => {
                if let Some(host) = host.filter(|h| *h != ed.path) {
                    if host_is_commit {
                        self.editor = None;
                        self.editor_host_snapshot = None;
                    } else {
                        self.switch_editor_to(&host);
                    }
                }
                return;
            }
            None => {}
        }
        if ed.follow_pending() {
            return;
        }
        match host {
            Some(host) if host != ed.path => {
                let follows = crate::editor_prefs::follow_selection() && !host_is_commit;
                if (!follows && !host_is_commit) || ed.ask_follow() {
                    return;
                }
                if follows {
                    self.switch_editor_to(&host);
                } else {
                    self.editor = None;
                    self.editor_host_snapshot = None;
                }
            }
            Some(_) => {
                self.editor_host_snapshot = match self.file_insight.as_ref() {
                    Some(f) => Some(HostSnapshot::File(f.request.clone())),
                    None => self.diff_view_state.clone().map(HostSnapshot::Diff),
                };
            }
            None => {}
        }
    }

    /// Opens `path` in the editor; when it cannot be edited the editor goes
    /// and the view under it stays, with the reason on the status line.
    fn switch_editor_to(&mut self, path: &Path) {
        match crate::editor::Editor::open(&self.repo_root, path) {
            Ok(opened) => self.editor = Some(opened),
            Err(why) => {
                self.editor = None;
                self.last_message = Some(why);
            }
        }
        self.editor_host_snapshot = None;
    }

    /// Cancel: the view and the pane selection go back to the edited file.
    fn restore_editor_host(&mut self) {
        let Some(snapshot) = self.editor_host_snapshot.clone() else {
            return;
        };
        match snapshot {
            HostSnapshot::Diff(state) => {
                self.file_insight = None;
                self.reselect_diff_row(&state);
                self.diff_view_state = Some(state);
            }
            HostSnapshot::File(request) => {
                self.diff_view_state = None;
                self.link.request_file_insight(request.clone());
                let path = request.path.clone();
                self.file_insight = Some(FileInsight::loading(request));
                if self.wip_all.shown() {
                    let found = self.wip_all.rows.iter().position(|r| {
                        r.file_index()
                            .and_then(|i| self.wip_all.paths.get(i))
                            .is_some_and(|p| *p == path)
                    });
                    if found.is_some() {
                        self.wip_all.state.select(found);
                        self.wip_all.follow = true;
                    }
                }
            }
        }
    }

    /// Puts the right pane's selection back on the row `state` is a diff of.
    fn reselect_diff_row(&mut self, state: &DiffViewState) {
        match state.source {
            DiffSource::Unstaged | DiffSource::Staged => {
                let section = if state.source == DiffSource::Staged {
                    StatusSection::Staged
                } else {
                    StatusSection::Unstaged
                };
                let found = section_lines(&self.status_section_lines, section)
                    .iter()
                    .position(
                        |l| matches!(l, StatusPaneLine::File { path, .. } if *path == state.path),
                    );
                if found.is_some() {
                    self.active_status_section = section;
                    self.status_states.select(section, found);
                }
            }
            _ => {
                let found = self
                    .commit_file_rows
                    .iter()
                    .position(|r| r.file_index() == Some(state.file_index));
                if found.is_some() {
                    self.commit_file_state.select(found);
                    self.commit_files_follow = true;
                }
            }
        }
    }
}
