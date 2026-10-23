//! Where `hosting review suggest` reads the project conventions from
//! (`gitpoulpi_ai::review_context`): the clone's base ref through the daemon,
//! or the host's API at the request's target branch, plus links through the
//! profile that matches their host.

use async_trait::async_trait;
use gitpoulpi_ai::review_context::{fetch_link, ContextSource, ProfileReader};
use gitpoulpi_core::domain::{BlobBody, BlobBytes, BlobSource, Oid, OperationKind};
use gitpoulpi_hosting::{HostingClient, Integration};
use serde_json::json;

use crate::client::Session;

use super::HostingEnv;

/// A conventions file is never read past this size.
const MAX_READ: u64 = 1 << 20;

/// Where the files of the request's target branch come from.
pub(super) enum Base<'a> {
    /// The clone: the request's base ref, read through the daemon.
    Daemon {
        session: tokio::sync::Mutex<&'a mut Session>,
        base: Oid,
    },
    /// The host: the target branch, through `file_content`.
    Api {
        client: &'a dyn HostingClient,
        project: String,
        branch: String,
    },
}

pub(super) struct SuggestSource<'a> {
    pub(super) base: Base<'a>,
    pub(super) env: &'a HostingEnv,
    pub(super) integrations: &'a [Integration],
}

struct HostReader<'a> {
    env: &'a HostingEnv,
    integrations: &'a [Integration],
}

#[async_trait]
impl ProfileReader for HostReader<'_> {
    async fn read(
        &self,
        integration: &Integration,
        project: &str,
        path: &str,
        rev: &str,
    ) -> Result<Option<Vec<u8>>, String> {
        let client = self.env.client(integration, self.integrations)?;
        client
            .file_content(project, path, Some(rev))
            .await
            .map_err(|e| e.to_string())
    }
}

#[async_trait]
impl ContextSource for SuggestSource<'_> {
    async fn repo_file(&self, path: &str) -> Result<Option<Vec<u8>>, String> {
        match &self.base {
            Base::Daemon { session, base } => {
                let mut session = session.lock().await;
                // A file that is not there is an error of the daemon; for a
                // context file that is the ordinary "absent".
                let blob: Result<BlobBytes, String> = session
                    .call(
                        OperationKind::ReadBlobBytes,
                        json!({
                            "path": path,
                            "source": BlobSource::Commit(*base),
                            "max": MAX_READ,
                        }),
                    )
                    .await;
                Ok(match blob.map(|b| b.body) {
                    Ok(BlobBody::Bytes(bytes)) => Some(bytes),
                    Ok(BlobBody::Lfs {
                        object: Some(bytes),
                        ..
                    }) => Some(bytes),
                    _ => None,
                })
            }
            Base::Api {
                client,
                project,
                branch,
            } => client
                .file_content(project, path, Some(branch))
                .await
                .map_err(|e| e.to_string()),
        }
    }

    async fn link_file(&self, url: &str) -> Result<Option<Vec<u8>>, String> {
        let reader = HostReader {
            env: self.env,
            integrations: self.integrations,
        };
        let transport = self.env.transport()?;
        fetch_link(url, self.integrations, &reader, &*transport).await
    }
}
