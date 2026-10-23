//! Where the review view gets the request's code from: a clone, through the
//! daemon's [`Requester`] (the request's refs are fetched and diffed by git),
//! or the host's API (no clone: the diff, the commits and the files come from
//! `pull_request_diff`, `pull_request_commits`, `commit_diff` and
//! `file_content`).
//!
//! The view works on [`Oid`]s (a diff's head and base, a commit, the key of a
//! file's new side). In API mode they are the commits' real ids when the host
//! sends 40 hex digits, else a stand-in made from the id's bytes; the source
//! remembers which id each one stands for, so a later call asks the host for
//! the right commit.
//!
//! Every call returns a [`Pending`] answer the render loop polls with
//! `take`; nothing here waits on the input thread.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};

use gitpoulpi_core::domain::{
    Blob, BlobContent, BlobSource, CommitNode, DiffTarget, FileDiff, Oid, OperationKind,
    MAX_TEXT_BLOB_BYTES,
};
use gitpoulpi_hosting::review::{ApiCommit, ApiFileDiff};
use serde_json::json;

use super::load::runtime;
use super::model::{GitData, ReviewTarget};
use crate::daemon_link::{Reply, Requester};
use crate::hosting_task::{error_text, ClientFactory};
use crate::redraw::NotifySend as _;

/// An answer on its way, from the daemon or from a worker thread.
pub(super) enum Pending<T> {
    Daemon(Reply<T>),
    Worker(mpsc::Receiver<Result<T, String>>),
}

impl<T> Pending<T> {
    /// An answer that is already there.
    fn ready(result: Result<T, String>) -> Self {
        let (tx, rx) = mpsc::channel();
        let _ = tx.send(result);
        Self::Worker(rx)
    }

    /// The answer, once; `None` while it is still coming.
    pub(super) fn take(&self) -> Option<Result<T, String>> {
        match self {
            Self::Daemon(reply) => reply.take().map(|r| r.map_err(|e| e.to_string())),
            Self::Worker(rx) => match rx.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err(gitpoulpi_i18n::msg::review_view::LOAD_STOPPED
                        .get()
                        .to_string()))
                }
            },
        }
    }
}

fn spawn<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Pending<T> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send_notify(work());
    });
    Pending::Worker(rx)
}

#[derive(Clone)]
pub(super) enum CodeSource {
    /// A clone: the daemon of the window's repository.
    Git(Requester),
    /// The host's API: no repository.
    Api(Box<ApiCode>),
}

/// `(commit, path)`; `None`: a file of the request itself.
type OmittedKey = (Option<Oid>, String);

/// The API half: what a call needs, and what the loads learned.
#[derive(Clone)]
pub(super) struct ApiCode {
    factory: Arc<dyn ClientFactory>,
    target: ReviewTarget,
    /// Which commit id each [`Oid`] of the view stands for.
    shas: Arc<Mutex<HashMap<Oid, String>>>,
    /// The request's files as the host sent them (the Files page and the
    /// ✦ draft read them from here).
    files: Arc<Mutex<Vec<ApiFileDiff>>>,
    /// `(commit, path)` of the files the host sent no patch for (`None`: a
    /// file of the request itself).
    omitted: Arc<Mutex<HashSet<OmittedKey>>>,
}

impl ApiCode {
    pub(super) fn new(factory: Arc<dyn ClientFactory>, target: ReviewTarget) -> Self {
        Self {
            factory,
            target,
            shas: Arc::default(),
            files: Arc::default(),
            omitted: Arc::default(),
        }
    }

    fn oid(&self, sha: &str) -> Oid {
        let oid = oid_of(sha);
        if let Ok(mut known) = self.shas.lock() {
            known.entry(oid).or_insert_with(|| sha.to_string());
        }
        oid
    }

    fn sha(&self, oid: Oid) -> String {
        self.shas
            .lock()
            .ok()
            .and_then(|known| known.get(&oid).cloned())
            .unwrap_or_else(|| oid.to_hex())
    }

    fn files(&self) -> Vec<FileDiff> {
        self.files
            .lock()
            .map(|f| f.iter().map(|f| f.diff.clone()).collect())
            .unwrap_or_default()
    }

    fn note_omitted(&self, at: Option<Oid>, files: &[ApiFileDiff]) {
        if let Ok(mut set) = self.omitted.lock() {
            set.retain(|(c, _)| *c != at);
            set.extend(
                files
                    .iter()
                    .filter(|f| f.omitted.is_some())
                    .map(|f| (at, f.diff.path.display().to_string())),
            );
        }
    }

    fn client(&self) -> Result<Box<dyn gitpoulpi_hosting::HostingClient>, String> {
        self.factory
            .client(&self.target.integration, &self.target.all)
            .map_err(|e| error_text(&e))
    }

    fn load(&self, patch_set: Option<u32>) -> Result<GitData, String> {
        let client = self.client()?;
        let (project, number) = (self.target.project.as_str(), self.target.number);
        let compare = self.target.compare_patch_set;
        runtime()?.block_on(async {
            let (detail, files, commits) = tokio::join!(
                client.pull_request_detail_at(project, number, patch_set),
                client.pull_request_diff(project, number, patch_set, compare),
                client.pull_request_commits(project, number),
            );
            let detail = detail.map_err(|e| error_text(&e))?;
            let files = files.map_err(|e| error_text(&e))?;
            // The Commits page is a nicety: a host that cannot list them
            // still gets the Files page.
            let commits = commits.unwrap_or_default();
            self.note_omitted(None, &files);
            if let Ok(mut held) = self.files.lock() {
                *held = files;
            }
            let head = self.oid(&detail.head_sha);
            let base = self.oid(&detail.base_sha);
            let mut commits: Vec<CommitNode> =
                commits.iter().rev().map(|c| self.node_of(c)).collect();
            if compare.is_some() {
                // As with a clone: the head patch set alone, whose diff is
                // the difference between the two patch sets.
                commits.truncate(1);
            }
            Ok(GitData {
                head,
                diff_base: base,
                comparing: compare.is_some(),
                commits,
            })
        })
    }

    fn node_of(&self, c: &ApiCommit) -> CommitNode {
        CommitNode {
            id: self.oid(&c.sha),
            parents: c.parents.iter().map(|p| self.oid(p)).collect(),
            summary: c.summary.clone(),
            message: c.summary.clone(),
            author_name: c.author.clone(),
            author_email: String::new(),
            timestamp: 0,
            refs: Vec::new(),
        }
    }
}

/// The view's id for a commit id of the host: the id itself when it is 40
/// hex digits, else its bytes folded into 20 (stable, and told apart by the
/// source's table).
fn oid_of(sha: &str) -> Oid {
    if let Some(oid) = Oid::from_hex(&sha.to_ascii_lowercase()) {
        return oid;
    }
    let mut bytes = [0u8; 20];
    for (i, b) in sha.bytes().enumerate() {
        bytes[i % 20] = bytes[i % 20].wrapping_mul(31).wrapping_add(b);
    }
    Oid(bytes)
}

impl CodeSource {
    pub(super) fn is_api(&self) -> bool {
        matches!(self, Self::Api(_))
    }

    /// The daemon link, when there is a clone.
    pub(super) fn requester(&self) -> Option<&Requester> {
        match self {
            Self::Git(r) => Some(r),
            Self::Api(_) => None,
        }
    }

    /// Starts reading the request's code: the head and base, the commits.
    /// `cancel` stops a clone's fetch when the view is gone.
    pub(super) fn start(
        &self,
        target: &ReviewTarget,
        patch_set: Option<u32>,
        cancel: Arc<AtomicBool>,
    ) -> mpsc::Receiver<Result<GitData, String>> {
        match self {
            Self::Git(requester) => {
                super::load::start_git(requester.clone(), target, patch_set, cancel)
            }
            Self::Api(api) => {
                let api = api.clone();
                let (tx, rx) = mpsc::channel();
                std::thread::spawn(move || {
                    let result = api.load(patch_set);
                    if !cancel.load(Ordering::Relaxed) {
                        let _ = tx.send_notify(result);
                    }
                });
                rx
            }
        }
    }

    /// One file of the request, from `base` to `head`.
    pub(super) fn file_diff(&self, base: Oid, head: Oid, path: &str) -> Pending<Vec<FileDiff>> {
        match self {
            Self::Git(requester) => Pending::Daemon(requester.call::<Vec<FileDiff>>(
                OperationKind::Diff,
                json!({
                    "target": DiffTarget::CommitRange(base, head),
                    "paths": [path],
                }),
            )),
            Self::Api(api) => Pending::ready(Ok(api
                .files()
                .into_iter()
                .filter(|f| f.path.as_path() == std::path::Path::new(path))
                .collect())),
        }
    }

    /// Every file of the request (what the ✦ draft reads).
    pub(super) fn all_files(&self, base: Oid, head: Oid) -> Pending<Vec<FileDiff>> {
        match self {
            Self::Git(requester) => Pending::Daemon(requester.call::<Vec<FileDiff>>(
                OperationKind::Diff,
                json!({ "target": DiffTarget::CommitRange(base, head) }),
            )),
            Self::Api(api) => Pending::ready(Ok(api.files())),
        }
    }

    /// What one commit changed. `against` is the older patch set the Files
    /// page is compared with (`comparing`), else the commit's first parent.
    pub(super) fn commit_diff(&self, commit: Oid, against: Option<Oid>) -> Pending<Vec<FileDiff>> {
        match self {
            Self::Git(requester) => Pending::Daemon(requester.call::<Vec<FileDiff>>(
                OperationKind::Diff,
                json!({ "target": match against {
                    Some(old) => DiffTarget::CommitRange(old, commit),
                    None => DiffTarget::CommitVsParent(commit),
                } }),
            )),
            Self::Api(api) => {
                if against.is_some() {
                    return Pending::ready(Ok(api.files()));
                }
                let api = api.clone();
                spawn(move || {
                    let client = api.client()?;
                    let files = runtime()?
                        .block_on(client.commit_diff(&api.target.project, &api.sha(commit)))
                        .map_err(|e| error_text(&e))?;
                    api.note_omitted(Some(commit), &files);
                    Ok(files.into_iter().map(|f| f.diff).collect())
                })
            }
        }
    }

    /// The new side of `path` at `at` (the head or a commit).
    pub(super) fn blob(&self, at: Oid, path: &str) -> Pending<Blob> {
        match self {
            Self::Git(requester) => Pending::Daemon(requester.call::<Blob>(
                OperationKind::ReadBlob,
                json!({ "path": path, "source": BlobSource::Commit(at) }),
            )),
            Self::Api(api) => {
                let (api, path) = (api.clone(), path.to_string());
                spawn(move || {
                    let client = api.client()?;
                    let sha = api.sha(at);
                    let raw = runtime()?
                        .block_on(client.file_content(&api.target.project, &path, Some(&sha)))
                        .map_err(|e| error_text(&e))?
                        .ok_or_else(|| {
                            gitpoulpi_i18n::msg::review_view::API_FILE_MISSING
                                .get()
                                .to_string()
                        })?;
                    let size = raw.len() as u64;
                    Ok(Blob {
                        path: path.clone().into(),
                        source: BlobSource::Commit(at),
                        size,
                        content: if size > MAX_TEXT_BLOB_BYTES {
                            BlobContent::TooLarge
                        } else {
                            BlobContent::from_bytes(&raw)
                        },
                        image: None,
                    })
                })
            }
        }
    }

    /// The host sent no patch for `path` (of the request, or of `commit`).
    pub(super) fn omitted(&self, commit: Option<Oid>, path: &str) -> bool {
        match self {
            Self::Git(_) => false,
            Self::Api(api) => api
                .omitted
                .lock()
                .is_ok_and(|set| set.contains(&(commit, path.to_string()))),
        }
    }

    /// Forgets what the loads learned (the request is read again).
    pub(super) fn reset(&self) {
        if let Self::Api(api) = self {
            if let Ok(mut f) = api.files.lock() {
                f.clear();
            }
            if let Ok(mut o) = api.omitted.lock() {
                o.clear();
            }
        }
    }
}
