//! The transport of a code-host client: swaps the personal token for the one
//! a repository's token command gives, on that repository's calls only, and
//! asks the command again once when the host answers 401.

use std::sync::Arc;

use crate::http::{HttpRequest, HttpResponse, HttpTransport, Progress};
use crate::scope::{self, RepoScopes};
use crate::Result;

pub(crate) struct ScopedTransport {
    inner: Arc<dyn HttpTransport>,
    integration_id: String,
    /// Tests give their own; production reads the installed ones per call.
    scopes: Option<Arc<dyn RepoScopes>>,
}

impl ScopedTransport {
    pub(crate) fn new(inner: Arc<dyn HttpTransport>, integration_id: String) -> Self {
        Self {
            inner,
            integration_id,
            scopes: None,
        }
    }

    #[cfg(test)]
    fn with_scopes(mut self, scopes: Arc<dyn RepoScopes>) -> Self {
        self.scopes = Some(scopes);
        self
    }
}

fn with_bearer(mut request: HttpRequest, token: &str) -> HttpRequest {
    request
        .headers
        .retain(|(name, _)| !name.eq_ignore_ascii_case("authorization"));
    request
        .headers
        .push(("Authorization".into(), format!("Bearer {token}")));
    request
}

#[async_trait::async_trait]
impl HttpTransport for ScopedTransport {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse> {
        let scopes = self.scopes.as_ref().or_else(|| scope::installed());
        let (Some(scopes), Some(project)) = (scopes, scope::github_project_of(&request.url)) else {
            return self.inner.send(request).await;
        };
        let Some(first) = scopes.token(&self.integration_id, &project, false).await else {
            return self.inner.send(request).await;
        };
        let response = self
            .inner
            .send(with_bearer(request.clone(), &first?))
            .await?;
        if response.status != 401 {
            return Ok(response);
        }
        // The token expired (an installation token lives an hour): ask the
        // command again, once.
        let Some(fresh) = scopes.token(&self.integration_id, &project, true).await else {
            return Ok(response);
        };
        self.inner.send(with_bearer(request, &fresh?)).await
    }

    async fn download(&self, request: HttpRequest, progress: &Progress<'_>) -> Result<Vec<u8>> {
        self.inner.download(request, progress).await
    }
}

#[cfg(test)]
pub(crate) mod tests;
