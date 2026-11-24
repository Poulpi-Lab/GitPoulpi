//! Coming back to a review in progress.
//!
//! A review is **in progress** while its view is open: not closed by the
//! reviewer (Esc, the cross, "Close review"), not submitted (Finish review,
//! Gerrit submit / abandon), not replaced by "check out". When a window ends
//! with such a view open (a tab switch, quitting) the view's place is written
//! as a [`ReviewResume`] marker next to the pending comments (the repository's
//! state file, or `remote-reviews.toml` for an API review). The window that
//! shows the repository (or the Radar) again reopens the request through the
//! normal entry point and [`ReviewView::restore_position`] puts the page, the
//! file and the scroll back once the data has loaded.
//!
//! A request that is merged or closed meanwhile, or a host that does not
//! answer, does not leave a broken view: the view asks to be closed with a
//! status line (`ReviewEffect::CloseWith`) and the marker is cleared.

use gitpoulpi_hosting::PullRequestState;
use gitpoulpi_i18n::msg::review_view as msg;
use gitpoulpi_session::{RemoteReviews, RepoState, ResumePos, ReviewResume};

use super::model::{DraftStore, Focus};
use super::{ReviewEffect, ReviewView};
use crate::review_cmd::Tab;

fn tab_word(tab: Tab) -> &'static str {
    match tab {
        Tab::Overview => "overview",
        Tab::Files => "files",
        Tab::Commits => "commits",
    }
}

fn tab_of(word: &str) -> Tab {
    match word {
        "files" => Tab::Files,
        "commits" => Tab::Commits,
        _ => Tab::Overview,
    }
}

/// The resumable marker of a request, as the marker files hold it.
pub(crate) fn mark_of(
    integration: &str,
    project: &str,
    number: u64,
    pos: ResumePos,
) -> ReviewResume {
    ReviewResume {
        integration: integration.to_string(),
        project: project.to_string(),
        number,
        pos,
        ..Default::default()
    }
}

/// Where a repository keeps its marker: `repo-state.toml`.
pub(crate) fn repo_mark(root: &std::path::Path) -> Option<ReviewResume> {
    let path = crate::remotes_ui::repo_state_path(root);
    RepoState::load(&path).ok().and_then(|s| s.open_review)
}

/// Clears a repository's marker (the review was closed or finished).
pub(crate) fn clear_repo_mark(root: &std::path::Path) {
    let path = crate::remotes_ui::repo_state_path(root);
    if let Ok(mut state) = RepoState::load(&path) {
        if state.open_review.take().is_some() {
            let _ = state.save(&path);
        }
    }
}

/// The API review's marker in `<config dir>/remote-reviews.toml`.
pub(crate) fn api_mark(config_dir: &std::path::Path) -> Option<ReviewResume> {
    RemoteReviews::load(&RemoteReviews::file_in(config_dir)).open_review
}

pub(crate) fn clear_api_mark(config_dir: &std::path::Path) {
    let path = RemoteReviews::file_in(config_dir);
    let mut file = RemoteReviews::load(&path);
    if file.open_review.take().is_some() {
        let _ = file.save(&path);
    }
}

impl ReviewView {
    /// The place to come back to, `None` when the review is over (submitted).
    pub(crate) fn resume_mark(&self) -> Option<ReviewResume> {
        if self.submitted {
            return None;
        }
        let pane = match self.tab {
            Tab::Commits => &self.commits_pane,
            _ => &self.files_pane,
        };
        let pos = ResumePos {
            tab: tab_word(self.tab).to_string(),
            diff_focus: self.focus == Focus::Diff,
            file: self.current_path().unwrap_or_default(),
            commit: self.commit_sel,
            cursor: pane.cursor,
            offset: pane.offset,
        };
        let t = &self.target;
        Some(ReviewResume {
            label: t.iid_label.clone(),
            title: t.title.clone(),
            web_url: t.web_url.clone(),
            source_branch: t.source_branch.clone(),
            target_branch: t.target_branch.clone(),
            ..mark_of(&t.integration.id, &t.project, t.number, pos)
        })
    }

    /// Writes the marker (the window is ending with the view open). A view
    /// that is over, or one still waiting to restore an earlier position
    /// (the reviewer left before it landed), keeps what is on file.
    pub(crate) fn save_resume_mark(&self) {
        if self.resume.is_some() {
            return;
        }
        let Some(mark) = self.resume_mark() else {
            self.forget_resume();
            return;
        };
        match &self.store {
            DraftStore::Repo(root) => {
                let path = crate::remotes_ui::repo_state_path(root);
                let mut state = RepoState::load(&path).unwrap_or_default();
                state.open_review = Some(mark);
                let _ = state.save(&path);
            }
            DraftStore::Remote(Some(path)) => {
                let mut file = RemoteReviews::load(path);
                file.open_review = Some(mark);
                let _ = file.save(path);
            }
            DraftStore::Remote(None) => {}
        }
    }

    /// The review was submitted: it is not reopened with the tab.
    pub(super) fn over(&mut self) {
        self.submitted = true;
        self.forget_resume();
    }

    /// The review is over (closed, submitted, gone): nothing to reopen.
    pub(crate) fn forget_resume(&self) {
        match &self.store {
            DraftStore::Repo(root) => clear_repo_mark(root),
            DraftStore::Remote(Some(path)) => {
                let mut file = RemoteReviews::load(path);
                if file.open_review.take().is_some() {
                    let _ = file.save(path);
                }
            }
            DraftStore::Remote(None) => {}
        }
    }

    /// Asks the view to put the page, the file and the scroll back once the
    /// request has loaded (a reopened review).
    pub(crate) fn restore_position(&mut self, pos: ResumePos) {
        self.resume = Some(pos);
        self.resume_placed = false;
    }

    /// Per poll: lands the position asked by [`Self::restore_position`], or
    /// gives the view up when the request cannot be reviewed any more.
    pub(super) fn poll_resume(&mut self) -> Option<ReviewEffect> {
        let pos = self.resume.clone()?;
        if let Some(why) = self.host_err.clone() {
            self.resume = None;
            self.forget_resume();
            return Some(ReviewEffect::CloseWith(msg::resume_unreachable(
                self.target.caption(),
                why,
            )));
        }
        let host = self.host.as_ref()?;
        if matches!(
            host.detail.summary.state,
            PullRequestState::Merged | PullRequestState::Closed
        ) {
            self.resume = None;
            self.forget_resume();
            return Some(ReviewEffect::CloseWith(msg::resume_gone(
                self.target.caption(),
            )));
        }
        let tab = tab_of(&pos.tab);
        let file_at = host.files.iter().position(|f| f.path == pos.file);
        if !self.resume_placed {
            self.resume_placed = true;
            self.set_tab(tab);
            match tab {
                Tab::Files => {
                    if let Some(at) = file_at {
                        self.select_file(at);
                    }
                }
                Tab::Commits => self.select_commit(pos.commit),
                Tab::Overview => {}
            }
            if pos.diff_focus && tab != Tab::Overview {
                self.focus = Focus::Diff;
            }
        }
        match tab {
            Tab::Overview => self.resume = None,
            Tab::Files | Tab::Commits => {
                if self.diff_err.is_some() {
                    self.resume = None;
                    return None;
                }
                let pane = match tab {
                    Tab::Commits => &mut self.commits_pane,
                    _ => &mut self.files_pane,
                };
                if !pane.is_empty() {
                    pane.cursor = pos.cursor;
                    pane.offset = pos.offset;
                    pane.clamp_to_rows();
                    self.resume = None;
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_words_round_trip_and_an_unknown_one_is_the_overview() {
        for tab in [Tab::Overview, Tab::Files, Tab::Commits] {
            assert_eq!(tab_of(tab_word(tab)), tab);
        }
        assert_eq!(tab_of("nonsense"), Tab::Overview);
    }

    #[test]
    fn a_marker_is_kept_and_cleared_in_the_repository_state_file() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let path = crate::remotes_ui::repo_state_path(root);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let state = RepoState {
            open_review: Some(mark_of("p", "o/r", 7, ResumePos::default())),
            ..Default::default()
        };
        state.save(&path).unwrap();
        assert_eq!(repo_mark(root).map(|m| m.number), Some(7));
        clear_repo_mark(root);
        assert_eq!(repo_mark(root), None);
    }
}
