//! The commit shown beside the Reflog and Lost work lists: its message, its
//! author and every file it changed. Two small reads per commit (the log
//! entry and the changed-file list), both off the render thread and cached
//! for the life of the panel, so moving the selection costs nothing twice.

use std::collections::HashMap;
use std::path::PathBuf;

use gitpoulpi_core::domain::{
    CommitGraph, CommitNode, DiffFileList, DiffTarget, FileStatus, LogOptions, Oid, OperationKind,
};

use crate::daemon_link::{Reply, Requester};

#[derive(Clone)]
pub(crate) struct Loaded {
    pub(crate) node: Option<CommitNode>,
    /// Every changed file, in the daemon's order.
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) statuses: HashMap<PathBuf, FileStatus>,
}

#[derive(Default)]
pub(crate) struct Detail {
    cache: HashMap<Oid, Loaded>,
    node_reply: Option<(Oid, Reply<CommitGraph>)>,
    files_reply: Option<(Oid, Reply<DiffFileList>)>,
    partial: HashMap<Oid, Loaded>,
    shown: Option<Oid>,
}

impl Detail {
    /// What is known about the selected commit, if anything yet.
    pub(crate) fn current(&self) -> Option<&Loaded> {
        self.shown.and_then(|oid| self.cache.get(&oid))
    }

    /// Puts a commit's details in the cache, as if the reads had landed.
    #[cfg(test)]
    pub(crate) fn fill(&mut self, oid: Oid, loaded: Loaded) {
        self.cache.insert(oid, loaded);
        self.shown = Some(oid);
    }

    /// Whether the selected commit is still being read.
    pub(crate) fn loading(&self) -> bool {
        self.node_reply.is_some() || self.files_reply.is_some()
    }

    /// Drains answers and starts the reads for `want` (the selected commit).
    pub(crate) fn poll(&mut self, requester: &Requester, want: Option<Oid>) {
        if let Some((oid, reply)) = self.node_reply.as_ref() {
            if let Some(result) = reply.take() {
                let oid = *oid;
                self.node_reply = None;
                let entry = self.partial.entry(oid).or_insert(Loaded {
                    node: None,
                    paths: Vec::new(),
                    statuses: HashMap::new(),
                });
                entry.node = result.ok().and_then(|g| g.nodes.into_iter().next());
                self.seal(oid);
            }
        }
        if let Some((oid, reply)) = self.files_reply.as_ref() {
            if let Some(result) = reply.take() {
                let oid = *oid;
                self.files_reply = None;
                let entry = self.partial.entry(oid).or_insert(Loaded {
                    node: None,
                    paths: Vec::new(),
                    statuses: HashMap::new(),
                });
                if let Ok(list) = result {
                    entry.paths = list.files.iter().map(|f| f.path.clone()).collect();
                    entry.statuses = list.files.into_iter().map(|f| (f.path, f.status)).collect();
                }
                self.seal(oid);
            }
        }
        self.shown = want;
        let Some(oid) = want else {
            return;
        };
        if self.cache.contains_key(&oid) || self.loading() {
            return;
        }
        self.node_reply = Some((
            oid,
            requester.call(
                OperationKind::Log,
                serde_json::json!({ "opts": LogOptions {
                    limit: Some(1),
                    start_ref: Some(oid.to_hex()),
                    ..LogOptions::default()
                } }),
            ),
        ));
        self.files_reply = Some((
            oid,
            requester.call(
                OperationKind::DiffFiles,
                serde_json::json!({ "target": DiffTarget::CommitVsParent(oid) }),
            ),
        ));
    }

    /// Moves a commit to the cache once both of its reads have landed.
    fn seal(&mut self, oid: Oid) {
        let waiting = self.node_reply.as_ref().is_some_and(|(o, _)| *o == oid)
            || self.files_reply.as_ref().is_some_and(|(o, _)| *o == oid);
        if !waiting {
            if let Some(done) = self.partial.remove(&oid) {
                self.cache.insert(oid, done);
            }
        }
    }
}
