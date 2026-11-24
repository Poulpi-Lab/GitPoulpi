//! Coming back to a review in progress: the window that shows a repository
//! (or the Home window with the Radar) reopens the review the tab was left
//! on. The marker is written by `finish` (`RepoDialogs::save_review_mark`)
//! and cleared when the review is closed or submitted
//! (`review_view::resume`).

use std::time::Duration;

use crate::event_loop::*;
#[cfg(all(feature = "hosting", feature = "review"))]
use gitpoulpi_i18n::msg;
use init::WindowState;

/// How long a reopened review waits for the request list (the sidebar's
/// PULL REQUESTS, read from the host) before it is given up. The marker stays:
/// the next time the tab is shown it tries again.
const WAIT: Duration = Duration::from_secs(60);

impl WindowState {
    /// The window is ending: a review still open (the repository's, or the
    /// Radar's over the Home window) is written down.
    pub(in crate::event_loop) fn save_review_marks(&self) {
        self.repo_dialogs.save_review_mark();
        if let Some(manager) = self.management.as_ref() {
            manager.radar.save_review_mark();
        }
    }

    /// Per frame: reopens the saved review of this repository once the
    /// request list knows the request, and the saved review of the Radar once
    /// the Radar is showing.
    pub(in crate::event_loop) fn review_resume_tick(&mut self) {
        self.api_resume_tick();
        let Some(mark) = self.review_resume.clone() else {
            return;
        };
        if self.no_repository || self.still_loading || self.refs_result.is_err() {
            return;
        }
        // The reviewer opened something else first: it is not taken over.
        if self.repo_dialogs.any_open() || self.review_resume_since.elapsed() > WAIT {
            self.review_resume = None;
            return;
        }
        #[cfg(all(feature = "hosting", feature = "review"))]
        {
            if !self.hosting.review_ready(mark.number) {
                return;
            }
            self.review_resume = None;
            let target = self
                .hosting
                .review_target(mark.number)
                .filter(|t| t.project == mark.project && t.integration.id == mark.integration);
            let Some(target) = target else {
                // Merged, closed or unlinked meanwhile: no broken view.
                crate::review_view::clear_repo_mark(&self.repo.root);
                self.last_message = Some(msg::review_view::resume_gone(format!(
                    "{} {}",
                    mark.label, mark.title
                )));
                return;
            };
            commands::apply_hosting_effect(
                &mut self.command_ctx(),
                crate::hosting_ui::HostingEffect::Review(Box::new(target)),
            );
            self.repo_dialogs.restore_review_position(mark.pos);
        }
        #[cfg(not(all(feature = "hosting", feature = "review")))]
        {
            let _ = mark;
            self.review_resume = None;
        }
    }

    /// The Radar was just shown with a review of the API in progress.
    fn api_resume_tick(&mut self) {
        #[cfg(all(feature = "hosting", feature = "review"))]
        {
            let Some(manager) = self.management.as_mut() else {
                return;
            };
            let Some(radar) = manager.radar.get() else {
                return;
            };
            if !radar.take_resume() || radar.hosts_review() {
                return;
            }
            let Some(dir) =
                gitpoulpi_config::ConfigPaths::discover().map(|p| p.config_dir().to_path_buf())
            else {
                return;
            };
            let Some(mark) = crate::review_view::api_mark(&dir) else {
                return;
            };
            let request = crate::radar::ReviewRequest {
                integration: mark.integration.clone(),
                project: mark.project.clone(),
                number: mark.number,
                label: mark.label.clone(),
                title: mark.title.clone(),
                web_url: mark.web_url.clone(),
                source_branch: mark.source_branch.clone(),
                target_branch: mark.target_branch.clone(),
                clone_url: String::new(),
            };
            commands::open_api_review(&mut self.command_ctx(), request);
            if let Some(radar) = self.management.as_mut().and_then(|m| m.radar.get()) {
                radar.restore_review_position(mark.pos);
            }
            // A profile that is gone says so on the status line and leaves
            // no view: the marker would only fail again.
            if !self
                .management
                .as_mut()
                .and_then(|m| m.radar.get())
                .is_some_and(|r| r.hosts_review())
            {
                crate::review_view::clear_api_mark(&dir);
            }
        }
    }
}

/// The review this repository's tab was left on, read when its window starts.
pub(in crate::event_loop) fn saved_mark(
    root: &std::path::Path,
) -> Option<gitpoulpi_session::ReviewResume> {
    #[cfg(all(feature = "hosting", feature = "review"))]
    {
        crate::review_view::repo_mark(root)
    }
    #[cfg(not(all(feature = "hosting", feature = "review")))]
    {
        let _ = root;
        None
    }
}
