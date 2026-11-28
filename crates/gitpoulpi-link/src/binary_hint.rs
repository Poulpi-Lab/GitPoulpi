//! Whether git diffs a file as binary, for the file menu's "Treat as
//! Binary" / "Treat as Text" row.
//!
//! Building a menu must not wait on the daemon, so the row reads a small
//! process-wide cache. It is filled by the daemon's answer to
//! `BinaryPaths`, asked for the file the Diff view shows (and the expected
//! value is recorded when a treat-as rule is written), and by a local fallback until then: git's own
//! sniff (a NUL byte in the first 8000 bytes of the file on disk). The
//! answer matters because attributes decide, not content: a `*.dat binary`
//! rule makes a text-looking file binary and the row must offer "Text".

use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use gitpoulpi_core::domain::Oid;

use crate::client::{DaemonLink, FileInsightKind, FileInsightRequest};

/// What is known about whether git diffs a file as binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hint {
    /// The daemon has not answered: only the local sniff exists. A menu row
    /// must not act on it, because attributes can say otherwise.
    Sniffed(bool),
    /// The daemon's answer (or the value a treat-as rule was just written
    /// for).
    FromGit(bool),
}

impl Hint {
    /// The value to act on, `None` while only the sniff is known.
    pub fn git(self) -> Option<bool> {
        match self {
            Hint::FromGit(binary) => Some(binary),
            Hint::Sniffed(_) => None,
        }
    }
}

#[derive(Default)]
struct Cache {
    answers: HashMap<PathBuf, bool>,
    /// Files with a request on its way.
    inflight: HashSet<PathBuf>,
    /// Files whose answer landed since the open menu was last refreshed.
    landed: HashSet<PathBuf>,
}

fn cache() -> &'static Mutex<Cache> {
    static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

fn last_asked() -> &'static Mutex<Option<PathBuf>> {
    static ASKED: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    ASKED.get_or_init(Default::default)
}

/// The daemon's answer for `rel` (repo-relative).
pub fn record(root: &Path, rel: &Path, binary: bool) {
    if let Ok(mut c) = cache().lock() {
        let key = root.join(rel);
        c.inflight.remove(&key);
        c.answers.insert(key.clone(), binary);
        c.landed.insert(key);
    }
}

/// Git's binary sniff on the file on disk. A missing file is text.
pub fn sniff(root: &Path, rel: &Path) -> bool {
    let Ok(mut file) = std::fs::File::open(root.join(rel)) else {
        return false;
    };
    let mut buf = vec![0u8; 8000];
    let n = file.read(&mut buf).unwrap_or(0);
    buf[..n].contains(&0)
}

/// What is known for `rel`: the daemon's answer, else the local sniff.
pub fn state(root: &Path, rel: &Path) -> Hint {
    match cache()
        .lock()
        .ok()
        .and_then(|c| c.answers.get(&root.join(rel)).copied())
    {
        Some(binary) => Hint::FromGit(binary),
        None => Hint::Sniffed(sniff(root, rel)),
    }
}

/// Git's answer for `rel` (`None` while pending): what menu rows read.
pub fn git_answer(root: &Path, rel: &Path) -> Option<bool> {
    state(root, rel).git()
}

/// Whether the answer for `rel` landed since the last call (clears it).
pub fn take_landed(root: &Path, rel: &Path) -> bool {
    cache()
        .lock()
        .is_ok_and(|mut c| c.landed.remove(&root.join(rel)))
}

/// Asks the daemon about `rel` when no answer is known and none is on its
/// way, for a menu opened on a file the Diff view does not show. Never blocks.
pub fn ask_pending(link: &DaemonLink, root: &Path, rel: &Path) {
    {
        let Ok(mut c) = cache().lock() else {
            return;
        };
        let key = root.join(rel);
        if c.answers.contains_key(&key) || !c.inflight.insert(key) {
            return;
        }
    }
    request(link, rel);
}

/// The daemon could not answer: fall back to the sniff so the row does not
/// stay pending forever.
pub fn record_failed(root: &Path, rel: &Path) {
    record(root, rel, sniff(root, rel));
}

fn request(link: &DaemonLink, rel: &Path) {
    link.request_file_insight(FileInsightRequest {
        path: rel.to_path_buf(),
        at: Oid([0u8; 20]),
        kind: FileInsightKind::BinaryState,
        worktree: true,
        staged: false,
    });
}

/// Asks the daemon about `path` (the file the Diff view shows) unless it
/// already did since the last rule write. Never blocks.
pub fn ask(link: &DaemonLink, path: Option<&Path>) {
    let Some(path) = path else {
        return;
    };
    let Ok(mut asked) = last_asked().lock() else {
        return;
    };
    if asked.as_deref() == Some(path) {
        return;
    }
    *asked = Some(path.to_path_buf());
    request(link, path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_nul_in_the_file_sniffs_as_binary_and_the_daemons_answer_wins() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("t.dat"), "plain\n").unwrap();
        std::fs::write(dir.path().join("b.dat"), b"a\0b").unwrap();
        assert_eq!(state(dir.path(), Path::new("t.dat")), Hint::Sniffed(false));
        assert_eq!(state(dir.path(), Path::new("b.dat")), Hint::Sniffed(true));
        assert_eq!(
            state(dir.path(), Path::new("missing")),
            Hint::Sniffed(false)
        );
        // A `*.dat binary` rule: the file reads as text but git says binary.
        record(dir.path(), Path::new("t.dat"), true);
        assert_eq!(state(dir.path(), Path::new("t.dat")), Hint::FromGit(true));
        record(dir.path(), Path::new("t.dat"), false);
        assert_eq!(state(dir.path(), Path::new("t.dat")), Hint::FromGit(false));
    }

    #[test]
    fn a_sniff_is_never_a_git_answer_until_one_lands() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("t.dat"), "plain\n").unwrap();
        let rel = Path::new("t.dat");
        assert_eq!(git_answer(dir.path(), rel), None);
        assert!(!take_landed(dir.path(), rel));
        record(dir.path(), rel, true);
        assert_eq!(git_answer(dir.path(), rel), Some(true));
        // The landing is reported once, so an open menu refreshes once.
        assert!(take_landed(dir.path(), rel));
        assert!(!take_landed(dir.path(), rel));
    }

    #[test]
    fn a_failed_request_falls_back_to_the_sniff_instead_of_staying_pending() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("b.dat"), b"a\0b").unwrap();
        record_failed(dir.path(), Path::new("b.dat"));
        assert_eq!(git_answer(dir.path(), Path::new("b.dat")), Some(true));
    }
}
