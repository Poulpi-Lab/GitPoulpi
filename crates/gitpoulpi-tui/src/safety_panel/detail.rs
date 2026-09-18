//! The commit shown beside the Reflog and Lost work lists: its message, its
//! author and the files it changed. Two small reads per commit (the log
//! entry and the changed-file list), both off the render thread and cached
//! for the life of the panel, so moving the selection costs nothing twice.

use std::collections::HashMap;

use gitpoulpi_core::domain::{
    CommitGraph, CommitNode, DiffFileList, DiffTarget, FileStatus, LogOptions, Oid, OperationKind,
};

use crate::daemon_link::{Reply, Requester};

/// How many changed files the side pane lists before "and N more".
pub(crate) const FILES_SHOWN: usize = 12;

#[derive(Clone)]
pub(crate) struct Loaded {
    pub(crate) node: Option<CommitNode>,
    pub(crate) files: Vec<(FileStatus, String)>,
    pub(crate) file_count: usize,
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
                    files: Vec::new(),
                    file_count: 0,
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
                    files: Vec::new(),
                    file_count: 0,
                });
                if let Ok(list) = result {
                    entry.file_count = list.files.len();
                    entry.files = list
                        .files
                        .into_iter()
                        .take(FILES_SHOWN)
                        .map(|f| (f.status, f.path.display().to_string()))
                        .collect();
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
