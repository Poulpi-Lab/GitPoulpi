//! The files of a review read in an encoding the reviewer chose (the diff
//! bar's encoding dropdown). The Diff view does the same through the link
//! thread (`diff_recode`); the review, which is not inside the event loop,
//! asks the daemon itself for the exact bytes of both sides of a file
//! (`ReadBlobBytes`, one file in flight) and diffs them on a worker thread,
//! so nothing here blocks the render. The choice itself is
//! `diff_recode::remembered`, shared with the Diff view: a file read as
//! Windows-1252 in one stays so in the other.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{mpsc, Arc};
use std::time::Instant;

use gitpoulpi_core::domain::{BlobBody, BlobBytes, BlobSource, FileStatus, Oid, OperationKind};
use serde_json::json;

use crate::daemon_link::{Reply, Requester};
use crate::diff_recode::{Recoded, FEEDBACK_AFTER};
use crate::editor_charset::Charset;

/// Which two blobs a file is made of: the older side (`None` for a file with
/// no earlier version) and the newer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Sides {
    pub(super) old: Option<Oid>,
    pub(super) new: Oid,
}

/// What one re-read is about.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct Key {
    path: String,
    sides: Sides,
    charset: Charset,
    /// An added file has no old side to read, a deleted one no new side.
    old_wanted: bool,
    new_wanted: bool,
}

impl Key {
    pub(super) fn new(path: &str, status: FileStatus, sides: Sides, charset: Charset) -> Self {
        Self {
            path: path.to_string(),
            sides,
            charset,
            old_wanted: !matches!(status, FileStatus::Added | FileStatus::Untracked),
            new_wanted: status != FileStatus::Deleted,
        }
    }
}

/// How a read ended.
#[derive(Debug, Clone)]
pub(super) enum Done {
    Ready(Arc<Recoded>),
    Failed(String),
}

struct Wait {
    key: Key,
    since: Instant,
    announced: bool,
    old: Option<Reply<BlobBytes>>,
    new: Option<Reply<BlobBytes>>,
    got_old: Option<Option<Vec<u8>>>,
    got_new: Option<Option<Vec<u8>>>,
    /// The diff, once both sides are in and the worker is at it.
    work: Option<mpsc::Receiver<Result<Recoded, String>>>,
}

#[derive(Default)]
pub(super) struct Recodes {
    held: HashMap<Key, Done>,
    wait: Option<Wait>,
}

fn bytes_of(answer: gitpoulpi_core::Result<BlobBytes>) -> Result<Option<Vec<u8>>, String> {
    match answer.map_err(|e| e.to_string())?.body {
        BlobBody::Bytes(raw) => Ok(Some(raw)),
        BlobBody::Lfs {
            object: Some(raw), ..
        } => Ok(Some(raw)),
        BlobBody::TooLarge => Err(gitpoulpi_i18n::msg::diff_view::ENC_TOO_LARGE
            .get()
            .to_string()),
        BlobBody::Lfs { .. } => Err(gitpoulpi_i18n::msg::diff_view::ENC_LFS_MISSING
            .get()
            .to_string()),
    }
}

impl Recodes {
    pub(super) fn get(&self, key: &Key) -> Option<&Done> {
        self.held.get(key)
    }

    #[cfg(test)]
    pub(super) fn insert(&mut self, key: Key, done: Done) {
        self.held.insert(key, done);
    }

    /// A read has been out long enough that the pane says so.
    pub(super) fn slow(&self, key: &Key) -> bool {
        self.wait
            .as_ref()
            .is_some_and(|w| &w.key == key && w.since.elapsed() >= FEEDBACK_AFTER)
    }

    pub(super) fn busy(&self) -> bool {
        self.wait.is_some()
    }

    pub(super) fn clear(&mut self) {
        self.held.clear();
        self.wait = None;
    }

    /// Forgets what was read for `path` (its choice changed).
    pub(super) fn forget(&mut self, path: &str) {
        self.held.retain(|k, _| k.path != path);
        if self.wait.as_ref().is_some_and(|w| w.key.path == path) {
            self.wait = None;
        }
    }

    /// Takes what arrived. `true`: the rows must be rebuilt (an answer, or
    /// the moment the "reading as ..." word is due).
    pub(super) fn poll(&mut self) -> bool {
        let Some(wait) = self.wait.as_mut() else {
            return false;
        };
        let mut changed = false;
        if !wait.announced && wait.since.elapsed() >= FEEDBACK_AFTER {
            wait.announced = true;
            changed = true;
        }
        let mut failure: Option<String> = None;
        if let Some(reply) = &wait.old {
            if let Some(answer) = reply.take() {
                match bytes_of(answer) {
                    Ok(bytes) => wait.got_old = Some(bytes),
                    Err(why) => failure = Some(why),
                }
                wait.old = None;
            }
        }
        if let Some(reply) = &wait.new {
            if let Some(answer) = reply.take() {
                match bytes_of(answer) {
                    Ok(bytes) => wait.got_new = Some(bytes),
                    Err(why) => failure = Some(why),
                }
                wait.new = None;
            }
        }
        if let Some(why) = failure {
            let key = wait.key.clone();
            self.wait = None;
            self.held.insert(key, Done::Failed(why));
            return true;
        }
        // Both sides are in: the diff is the worker's.
        if wait.work.is_none() && wait.old.is_none() && wait.new.is_none() {
            let old = wait.got_old.take().flatten();
            let new = wait.got_new.take().flatten();
            let (path, charset) = (wait.key.path.clone(), wait.key.charset);
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                let done = crate::diff_recode::recode(
                    Path::new(&path),
                    old.as_deref(),
                    new.as_deref(),
                    charset,
                )
                .map(|(diff, new_lines)| Recoded {
                    diff,
                    new_lines: Arc::new(new_lines),
                });
                let _ = tx.send(done);
                crate::redraw::poke();
            });
            wait.work = Some(rx);
        }
        if let Some(rx) = &wait.work {
            match rx.try_recv() {
                Ok(done) => {
                    let key = wait.key.clone();
                    self.wait = None;
                    self.held.insert(
                        key,
                        match done {
                            Ok(r) => Done::Ready(Arc::new(r)),
                            Err(why) => Done::Failed(why),
                        },
                    );
                    return true;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    let key = wait.key.clone();
                    self.wait = None;
                    self.held.insert(
                        key,
                        Done::Failed(gitpoulpi_i18n::msg::review_view::LOAD_STOPPED.get().into()),
                    );
                    return true;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        changed
    }

    /// Asks for the first of `wanted` that is neither held nor on its way.
    pub(super) fn request(&mut self, requester: &Requester, wanted: Vec<Key>) {
        if self.wait.is_some() {
            return;
        }
        let Some(key) = wanted.into_iter().find(|k| !self.held.contains_key(k)) else {
            return;
        };
        let read = |oid: Oid| {
            requester.call::<BlobBytes>(
                OperationKind::ReadBlobBytes,
                json!({
                    "path": key.path,
                    "source": BlobSource::Commit(oid),
                    "max": gitpoulpi_core::domain::MAX_TEXT_BLOB_BYTES,
                }),
            )
        };
        let old = key.sides.old.filter(|_| key.old_wanted).map(read);
        let new = key.new_wanted.then(|| read(key.sides.new));
        self.wait = Some(Wait {
            key,
            since: Instant::now(),
            announced: false,
            old,
            new,
            got_old: None,
            got_new: None,
            work: None,
        });
    }
}
