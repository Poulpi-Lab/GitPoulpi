//! `RepoReader` over the daemon: what lets `ai hosting-reply --read-repo`
//! read the repository the way every other command does (an IPC client, never
//! git directly). Read operations only: `Resolve`, `ListTree`,
//! `ReadBlobBytes`, `GrepTree`, `MergeBase` and `Diff`.

use gitpoulpi_ai::hosting_reply::repo_context::RepoReader;
use gitpoulpi_ai::{Error, Result};
use gitpoulpi_core::domain::{
    BlobBody, BlobBytes, BlobSource, DiffTarget, FileDiff, GrepHit, Oid, OperationKind,
    MAX_GREP_FILE_BYTES,
};
use serde_json::json;
use tokio::sync::Mutex;

use crate::client::Session;

/// A [`RepoReader`] that asks the daemon through the command's own session.
pub(crate) struct SessionReader<'a> {
    session: Mutex<&'a mut Session>,
}

impl<'a> SessionReader<'a> {
    pub(crate) fn new(session: &'a mut Session) -> Self {
        Self {
            session: Mutex::new(session),
        }
    }
}

fn failed(what: &str, why: String) -> Error {
    Error::Invalid(format!("could not read the repository ({what}): {why}"))
}

#[async_trait::async_trait]
impl RepoReader for SessionReader<'_> {
    async fn head(&self) -> Result<Oid> {
        self.session
            .lock()
            .await
            .call(OperationKind::Resolve, json!({ "rev": "HEAD" }))
            .await
            .map_err(|e| failed("HEAD", e))
    }

    async fn list_tree(&self, at: Oid) -> Result<Vec<String>> {
        let paths: Vec<std::path::PathBuf> = self
            .session
            .lock()
            .await
            .call(OperationKind::ListTree, json!({ "at": at }))
            .await
            .map_err(|e| failed("the file list", e))?;
        Ok(paths
            .into_iter()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .collect())
    }

    async fn read(&self, path: &str, at: Oid) -> Result<Option<Vec<u8>>> {
        let blob: std::result::Result<BlobBytes, String> = self
            .session
            .lock()
            .await
            .call(
                OperationKind::ReadBlobBytes,
                json!({
                    "path": path,
                    "source": BlobSource::Commit(at),
                    "max": MAX_GREP_FILE_BYTES,
                }),
            )
            .await;
        // A path that does not exist at this commit is an error from the
        // daemon: not worth failing the whole reply for.
        let Ok(blob) = blob else { return Ok(None) };
        Ok(match blob.body {
            BlobBody::Bytes(bytes) if !bytes[..bytes.len().min(8000)].contains(&0) => Some(bytes),
            _ => None,
        })
    }

    async fn grep(&self, terms: &[String], at: Oid, max_hits: usize) -> Result<Vec<GrepHit>> {
        self.session
            .lock()
            .await
            .call(
                OperationKind::GrepTree,
                json!({ "at": at, "terms": terms, "max_hits": max_hits }),
            )
            .await
            .map_err(|e| failed("the search", e))
    }

    async fn diff(&self, base: &str, head: &str) -> Result<Option<String>> {
        let mut session = self.session.lock().await;
        let Some(base) = session_resolve(&mut session, base.to_string()).await else {
            return Ok(None);
        };
        let Some(head) = session_resolve(&mut session, head.to_string()).await else {
            return Ok(None);
        };
        // The pull request's own changes: from the merge base to the head.
        let from: Oid = session
            .call(OperationKind::MergeBase, json!({ "a": base, "b": head }))
            .await
            .unwrap_or(base);
        let files: Vec<FileDiff> = session
            .call(
                OperationKind::Diff,
                json!({ "target": DiffTarget::CommitRange(from, head) }),
            )
            .await
            .map_err(|e| failed("the pull request's diff", e))?;
        Ok(Some(gitpoulpi_ai::assist::render_diff(&files)))
    }
}

async fn session_resolve(session: &mut Session, rev: String) -> Option<Oid> {
    session
        .call(OperationKind::Resolve, json!({ "rev": rev }))
        .await
        .ok()
}
