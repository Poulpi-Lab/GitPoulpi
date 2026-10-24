//! The new side of the files a review shows, for the whole-file layout of
//! the diff pane (the Hunks toggle off). Same blobs the Diff view reads for
//! it (`diff_content::blob_lines`), here read from the request's head (the
//! Files page) or from a commit (the Commits page), one read in flight at a
//! time on the daemon link: nothing waits for them, a file shows its hunks
//! (with a line saying so) until its text lands.

use std::collections::HashMap;
use std::sync::Arc;

use gitpoulpi_core::domain::{Blob, FileDiff, FileStatus, Oid};

use super::source::{CodeSource, Pending};

/// How many files of one commit are read for the whole-file layout; the rest
/// keep their hunks. A commit is not a place to read a hundred files from.
const MAX_FILES: usize = 40;

/// What a read ended in.
#[derive(Debug, Clone)]
pub(super) enum Whole {
    Ready(Arc<Vec<String>>),
    /// Binary, too large, or the read failed: the hunks stay, with this said.
    Unavailable(String),
}

type Key = (Oid, String);

#[derive(Default)]
pub(super) struct Blobs {
    held: HashMap<Key, Whole>,
    wait: Option<(Key, Pending<Blob>)>,
}

/// Whether the whole-file layout needs `file`'s new side: an added or deleted
/// file's one hunk already is the whole of it.
pub(super) fn needs_content(file: &FileDiff) -> bool {
    !file.hunks.is_empty()
        && !matches!(
            file.status,
            FileStatus::Added | FileStatus::Untracked | FileStatus::Deleted
        )
}

impl Blobs {
    pub(super) fn get(&self, at: Oid, path: &str) -> Option<&Whole> {
        self.held.get(&(at, path.to_string()))
    }

    /// A read that already answered (what the tests put in by hand).
    #[cfg(test)]
    pub(super) fn insert(&mut self, at: Oid, path: &str, lines: Vec<String>) {
        self.held
            .insert((at, path.to_string()), Whole::Ready(Arc::new(lines)));
    }

    pub(super) fn busy(&self) -> bool {
        self.wait.is_some()
    }

    pub(super) fn clear(&mut self) {
        self.held.clear();
        self.wait = None;
    }

    /// Takes an answer that arrived. `true`: the rows must be rebuilt.
    pub(super) fn poll(&mut self) -> bool {
        let Some((key, reply)) = &self.wait else {
            return false;
        };
        let Some(answer) = reply.take() else {
            return false;
        };
        let key = key.clone();
        self.wait = None;
        let whole = match answer {
            Ok(blob) => match crate::diff_content::blob_lines(blob) {
                Ok(lines) => Whole::Ready(lines),
                Err(why) => Whole::Unavailable(why),
            },
            Err(why) => Whole::Unavailable(why),
        };
        self.held.insert(key, whole);
        true
    }

    /// Asks for the first of `wanted` that is neither held nor on its way.
    pub(super) fn request(&mut self, source: &CodeSource, wanted: Vec<(Oid, String)>) {
        if self.wait.is_some() {
            return;
        }
        let Some(key) = wanted
            .into_iter()
            .take(MAX_FILES)
            .find(|k| !self.held.contains_key(k))
        else {
            return;
        };
        let reply = source.blob(key.0, &key.1);
        self.wait = Some((key, reply));
    }
}
