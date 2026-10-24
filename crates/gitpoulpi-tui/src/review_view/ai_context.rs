//! What the ✦ draft run is given besides the diff (`[ai.review]`, slice S5a):
//! the levels that decide it, and where the files come from. The rules and
//! the caps live in `gitpoulpi_ai::review_context`; this file only knows the
//! view's two code sources.
//!
//! Files of the repository come from the request's target branch: the
//! daemon's `ReadBlob` of the base commit for a clone, the host's
//! `file_content` at the base commit without one (both through
//! [`CodeSource::blob`]). A link goes through the profile of its host, else
//! an anonymous GET.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use gitpoulpi_ai::review_context::EffectiveReviewContext;
use gitpoulpi_ai::review_context::{fetch_link, resolve, ContextSource, ProfileReader};
use gitpoulpi_config::{AiReviewContext, Integration, RepoAiReview};
use gitpoulpi_core::domain::{BlobContent, Oid};
use gitpoulpi_session::{RemoteReviews, RepoState};

use super::source::CodeSource;
use crate::hosting_task::{ClientFactory, DefaultFactory};

/// The repository's own values (a clone's state file, or the project's entry
/// of `remote-reviews.toml`), which sit between the global table and a run.
pub(super) fn own_values(
    repo_root: Option<&Path>,
    remote_file: Option<&Path>,
    remote_key: &str,
    state_dir: &str,
) -> RepoAiReview {
    match (repo_root, remote_file) {
        (Some(root), _) => RepoState::load(&RepoState::path_for(root, state_dir))
            .map(|s| s.preferences.ai_review)
            .unwrap_or_default(),
        (None, Some(file)) => RemoteReviews::load(file)
            .ai_review
            .get(remote_key)
            .cloned()
            .unwrap_or_default(),
        (None, None) => RepoAiReview::default(),
    }
}

/// What a draft run is given: the global table, the repository's own values
/// over it and the choices of the options dialog over both.
pub(super) fn effective(
    global: &AiReviewContext,
    own: &RepoAiReview,
    per_run: &RepoAiReview,
) -> EffectiveReviewContext {
    resolve(global, Some(own), per_run)
}

/// The view's code source as a [`ContextSource`].
pub(super) struct ViewContext {
    pub(super) source: Mutex<CodeSource>,
    /// The commit the request merges into.
    pub(super) base: Oid,
    pub(super) cancel: Arc<AtomicBool>,
    pub(super) integrations: Vec<Integration>,
}

struct Profiles<'a>(&'a [Integration]);

#[async_trait]
impl ProfileReader for Profiles<'_> {
    async fn read(
        &self,
        integration: &Integration,
        project: &str,
        path: &str,
        rev: &str,
    ) -> Result<Option<Vec<u8>>, String> {
        let client = DefaultFactory
            .client(integration, self.0)
            .map_err(|e| e.to_string())?;
        client
            .file_content(project, path, Some(rev))
            .await
            .map_err(|e| e.to_string())
    }
}

#[async_trait]
impl ContextSource for ViewContext {
    async fn repo_file(&self, path: &str) -> Result<Option<Vec<u8>>, String> {
        let pending = self
            .source
            .lock()
            .map_err(|_| "the code source is gone".to_string())? // i18n-english
            .blob(self.base, path);
        loop {
            if let Some(answer) = pending.take() {
                // A file that is not there is an error of the source; for a
                // context file that is the ordinary "absent".
                return Ok(match answer.map(|blob| blob.content) {
                    Ok(BlobContent::Text { lines, .. }) => Some(lines.join("\n").into_bytes()),
                    _ => None,
                });
            }
            if self.cancel.load(Ordering::Relaxed) {
                return Err(String::new());
            }
            tokio::time::sleep(Duration::from_millis(4)).await;
        }
    }

    async fn link_file(&self, url: &str) -> Result<Option<Vec<u8>>, String> {
        let transport = gitpoulpi_hosting::shared_network_transport().map_err(|e| e.to_string())?;
        fetch_link(
            url,
            &self.integrations,
            &Profiles(&self.integrations),
            &*transport,
        )
        .await
    }
}
