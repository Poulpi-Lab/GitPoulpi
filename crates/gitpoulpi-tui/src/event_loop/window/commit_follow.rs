//! The commit column's follow step: the Log's selection moved to another
//! commit while a view of a commit's file is open beside the column, so the
//! view goes along (`commit_column::plan` says to what).
//!
//! It waits for the new commit's file list (the same read the right panel
//! shows), which is also what tells whether the commit changed the file, so
//! no read is made twice and nothing here waits on the daemon.

use crate::commit_column::{open_file, plan, viewed_commit, Retarget};
use crate::event_loop::*;
use crate::file_panel::{open_commit_file, CommitFileOpen};
use crate::file_view::FollowNote;
use init::WindowState;

impl WindowState {
    pub(in crate::event_loop) fn follow_commit_column(&mut self) {
        // Not drawn last frame: nothing was picked, the view stays as it is.
        if self.commit_col.area.width == 0 {
            self.commit_col.keys = false;
            return;
        }
        if self.compare.is_some() {
            return;
        }
        let Some(selected) =
            selected_log_commit(&self.log_result, &self.log_state, &self.log_order)
        else {
            return;
        };
        let Some(viewed) = viewed_commit(
            self.central_view,
            self.diff_view_state.as_ref(),
            self.file_insight.as_ref(),
        ) else {
            return;
        };
        // A kind of view the menu asked for is opened even at the commit
        // the view is already on.
        if viewed == selected && self.commit_col.force.is_none() {
            return;
        }
        // The new commit's files, not yet there: the view stays for now.
        let Some(outcome) = self.commit_files.as_ref().filter(|o| o.commit == selected) else {
            return;
        };
        let Some((kind, path)) = open_file(
            self.central_view,
            self.diff_view_state.as_ref(),
            self.file_insight.as_ref(),
        ) else {
            return;
        };
        let path = path.to_path_buf();
        let files = outcome.files.as_ref().ok().map(|f| f.as_slice());
        let kind = self.commit_col.force.take().unwrap_or(kind);
        let retarget = plan(kind, &path, files);
        let changed = files.is_some_and(|f| f.iter().any(|d| d.path == path));
        match retarget {
            Retarget::Diff { file_index, path } => open_commit_file(
                CommitFileOpen::Diff { file_index, path },
                selected,
                &self.link,
                &mut self.file_insight,
                &mut self.diff_view_state,
                &mut self.central_view,
                None,
            ),
            Retarget::Content { path } => {
                open_commit_file(
                    CommitFileOpen::Content { path },
                    selected,
                    &self.link,
                    &mut self.file_insight,
                    &mut self.diff_view_state,
                    &mut self.central_view,
                    None,
                );
                if let (false, Some(insight)) = (changed, self.file_insight.as_mut()) {
                    insight.follow_note = Some(FollowNote::Unchanged);
                }
            }
            Retarget::Blame { path, changed } => {
                self.diff_view_state = changed.map(|file_index| {
                    DiffViewState::for_commit(selected, file_index, path.clone()).resume_last_used()
                });
                open_file_insight_at(
                    FileInsightKind::Blame,
                    path,
                    selected,
                    &self.link,
                    &mut self.file_insight,
                );
                if let (None, Some(insight)) = (changed, self.file_insight.as_mut()) {
                    insight.follow_note = Some(FollowNote::Unchanged);
                }
                self.central_view = CentralView::File;
            }
        }
    }
}
