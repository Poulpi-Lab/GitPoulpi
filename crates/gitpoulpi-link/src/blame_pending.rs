//! "Show this commit's diff for the file": the Log has to select the
//! commit and load its changed files before the diff can open from them by
//! path, so the wish waits here and the window's tick settles it.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::Oid;
use gitpoulpi_i18n::msg::blame_view as msg;
use gitpoulpi_i18n::M;

use crate::CommitFilesOutcome;

/// How long the commit's file list may take before the wish is dropped.
const PATIENCE: Duration = Duration::from_secs(15);

static WANTED: Mutex<Option<(Oid, PathBuf, Instant)>> = Mutex::new(None);

/// Remembers that the diff of `path` at `commit` should open as soon as the
/// commit's files are loaded.
pub fn want_diff(commit: Oid, path: PathBuf) {
    if let Ok(mut slot) = WANTED.lock() {
        *slot = Some((commit, path, Instant::now()));
    }
}

/// What the tick should do about the wish, given the loaded files.
#[derive(Debug, PartialEq, Eq)]
pub enum Ready {
    /// Open the diff of `path` (index `file_index` of the commit's files).
    Open {
        commit: Oid,
        file_index: usize,
        path: PathBuf,
    },
    /// The commit does not list the file: say why.
    Refused(M),
}

/// Settles the wish when the files of its commit are there (or it timed
/// out). `None`: nothing to do yet.
pub fn settle(files: Option<&CommitFilesOutcome>) -> Option<Ready> {
    let mut slot = WANTED.lock().ok()?;
    let (commit, path, since) = slot.as_ref()?.clone();
    if since.elapsed() > PATIENCE {
        *slot = None;
        return None;
    }
    let outcome = files.filter(|o| o.commit == commit)?;
    *slot = None;
    let Ok(list) = &outcome.files else {
        return Some(Ready::Refused(msg::FILE_NOT_LISTED));
    };
    Some(match list.iter().position(|f| f.path == path) {
        Some(file_index) => Ready::Open {
            commit,
            file_index,
            path,
        },
        None => Ready::Refused(msg::FILE_NOT_LISTED),
    })
}

/// Whether a wish is waiting (the loop keeps ticking while it is).
pub fn waiting() -> bool {
    WANTED.lock().is_ok_and(|s| s.is_some())
}
