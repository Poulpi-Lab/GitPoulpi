//! A diff read in an encoding the user chose (the Diff view's and the
//! review's encoding dropdown).
//!
//! **Why the bytes.** Every backend hands the TUI diff lines already
//! decoded to UTF-8 (lossily, or with the `git.default_encoding` fallback),
//! and git calls a UTF-16 file binary and prints no hunks for it. Neither
//! can be undone from the strings. So a chosen encoding is a second read:
//! the exact bytes of the file's two sides (`ReadBlobBytes`, the same
//! sources the whole-file layout and the image view use), decoded with the
//! chosen [`Charset`] and diffed here (`history_view::rename::hunks`, the
//! histogram diff the File History already uses for renames). The link
//! thread does the work (`daemon_link/file_insight.rs`); nothing here
//! blocks the render thread.
//!
//! **Staging is off while an encoding is forced.** The hunks of this diff
//! are numbered by this module, not by the backend, and staging names a
//! `HunkId` the backend resolves against its own diff. `Auto-detect` is the
//! diff staging understands.
//!
//! **Memory.** The choice is remembered per file path for the session
//! (`remembered`), shared by the Diff view and the review: reopening a file
//! comes back in the encoding it was read in. The editor's "Reopen with
//! encoding" is per document and remembers nothing, so there is no
//! per-repository store to share with.
//!
//! The module keeps its state in a static (`image_view::split` does the
//! same): the draw asks [`resolve`] every frame and gets the answer or
//! "loading", the event loop sends [`take_requests`] and hands the link's
//! answers to [`offer`].

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{FileDiff, FileStatus, Oid};

use crate::daemon_link::{
    DiffContentSide, FileInsightKind, FileInsightOutcome, FileInsightPayload, FileInsightRequest,
};
use crate::editor_charset::Charset;

/// How long a re-read may take before the view says it is working.
pub(crate) const FEEDBACK_AFTER: Duration = Duration::from_millis(150);

/// Entries kept: a view and a review in the middle of a few files.
const KEEP: usize = 12;

/// The label of the "decide for me" choice, and its combo value.
pub(crate) const AUTO_VALUE: &str = "auto";

/// Splits decoded text into lines the way the file views do: no terminator,
/// a trailing `\r` dropped, no empty last entry for a final newline.
fn lines_of(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = text
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l).to_string())
        .collect();
    if lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    lines
}

fn decode(bytes: &[u8], charset: Charset) -> Result<Vec<String>, String> {
    charset
        .decode(bytes)
        .map(|(text, _)| lines_of(&text))
        .map_err(|_| gitpoulpi_i18n::msg::diff_view::enc_not_valid(charset.name()))
}

/// The diff between two sides' bytes read as `charset`, and the new side's
/// whole text. `None` is an empty side. The status is a guess (`Modified`,
/// `Added`, `Deleted`) that the caller replaces with the real one.
pub(crate) fn recode(
    path: &Path,
    old: Option<&[u8]>,
    new: Option<&[u8]>,
    charset: Charset,
) -> Result<(FileDiff, Vec<String>), String> {
    let old_lines = old.map(|b| decode(b, charset)).transpose()?;
    let new_lines = new.map(|b| decode(b, charset)).transpose()?;
    let status = match (&old_lines, &new_lines) {
        (None, _) => FileStatus::Added,
        (_, None) => FileStatus::Deleted,
        _ => FileStatus::Modified,
    };
    let (a, b) = (
        old_lines.unwrap_or_default(),
        new_lines.clone().unwrap_or_default(),
    );
    let mut hunks = crate::history_view::rename::hunks(&a, &b);
    // git spells an empty side `N,0` with N the line before it.
    for h in &mut hunks {
        if h.old_lines == 0 {
            h.old_start = h.old_start.saturating_sub(1);
        }
        if h.new_lines == 0 {
            h.new_start = h.new_start.saturating_sub(1);
        }
    }
    Ok((
        FileDiff {
            path: path.to_path_buf(),
            status,
            hunks,
        },
        new_lines.unwrap_or_default(),
    ))
}

// -- what to read ------------------------------------------------------------------

/// Which two blobs one open diff is made of, and in which encoding to read
/// them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Want {
    pub(crate) path: PathBuf,
    pub(crate) old: DiffContentSide,
    pub(crate) new: DiffContentSide,
    /// The request's `at`, for the `Commit` / `Parent` sides.
    pub(crate) at: Oid,
    pub(crate) charset: Charset,
}

impl Want {
    /// The sides of a Diff view: a commit against its parent, one half of
    /// the working tree, or a comparison.
    pub(crate) fn of_state(state: &crate::diff_view::DiffViewState, charset: Charset) -> Self {
        use crate::diff_view::DiffSource;
        use gitpoulpi_core::domain::DiffTarget as T;
        use DiffContentSide as S;
        let (old, new, at) = match state.source {
            DiffSource::Commit => (S::Parent, S::Commit, state.commit),
            DiffSource::Unstaged => (S::Index, S::WorkingTree, state.commit),
            DiffSource::Staged => (S::Head, S::Index, state.commit),
            DiffSource::Compare => match state.compare {
                Some(T::CommitRange(a, b)) => (S::CommitAt(a), S::CommitAt(b), state.commit),
                Some(T::CommitSpan(o, n)) => (S::ParentOf(o), S::CommitAt(n), state.commit),
                Some(T::CommitVsWorkingTree(c)) => (S::CommitAt(c), S::WorkingTree, state.commit),
                Some(T::CommitVsParent(c)) => (S::ParentOf(c), S::CommitAt(c), state.commit),
                Some(T::WorkingTreeVsIndex) => (S::Index, S::WorkingTree, state.commit),
                Some(T::IndexVsHead) => (S::Head, S::Index, state.commit),
                None => (S::Parent, S::Commit, state.commit),
            },
        };
        Self {
            path: state.path.clone(),
            old,
            new,
            at,
            charset,
        }
    }

    /// A commit's file against the commit's first parent.
    #[cfg(test)]
    pub(crate) fn of_commit(commit: Oid, path: PathBuf, charset: Charset) -> Self {
        Self {
            path,
            old: DiffContentSide::ParentOf(commit),
            new: DiffContentSide::CommitAt(commit),
            at: commit,
            charset,
        }
    }

    fn request(&self, status: FileStatus) -> FileInsightRequest {
        let none =
            |side| (!matches!(status, FileStatus::Added | FileStatus::Untracked)).then_some(side);
        let new_none = |side| (status != FileStatus::Deleted).then_some(side);
        FileInsightRequest {
            path: self.path.clone(),
            at: self.at,
            kind: FileInsightKind::DiffRecoded {
                old: none(self.old),
                new: new_none(self.new),
                charset: self.charset,
            },
            worktree: false,
            staged: false,
        }
    }
}

// -- the state ---------------------------------------------------------------------

/// What a re-read produced.
#[derive(Debug)]
pub(crate) struct Recoded {
    pub(crate) diff: FileDiff,
    /// The new side's whole text, for the whole-file layout.
    pub(crate) new_lines: Arc<Vec<String>>,
}

#[derive(Debug, Clone)]
enum State {
    Loading,
    Ready(Arc<Recoded>),
    Failed(String),
}

struct Entry {
    request: FileInsightRequest,
    /// The diff this was read for; a different one arriving means the
    /// file changed under the read.
    for_diff: FileDiff,
    since: Instant,
    state: State,
}

#[derive(Default)]
struct Store {
    remembered: HashMap<PathBuf, Charset>,
    entries: Vec<Entry>,
    outbox: Vec<FileInsightRequest>,
}

#[cfg(not(test))]
static STORE: Mutex<Option<Store>> = Mutex::new(None);

// In tests the store is per thread: unit tests run in parallel in one
// process, and a test that remembers an encoding for `src/a.rs` would
// otherwise change what every other test rendering that path reads (the
// bar saying `Windows-1252` instead of `Auto`, only under load).
#[cfg(test)]
thread_local! {
    static STORE: std::cell::RefCell<Option<Store>> = const { std::cell::RefCell::new(None) };
}

#[cfg(not(test))]
fn with<R>(f: impl FnOnce(&mut Store) -> R) -> R {
    let mut guard = STORE.lock().unwrap_or_else(|p| p.into_inner());
    f(guard.get_or_insert_with(Store::default))
}

#[cfg(test)]
fn with<R>(f: impl FnOnce(&mut Store) -> R) -> R {
    STORE.with(|s| f(s.borrow_mut().get_or_insert_with(Store::default)))
}

/// The encoding last chosen for `path` this session, if any.
pub(crate) fn remembered(path: &Path) -> Option<Charset> {
    with(|s| s.remembered.get(path).copied())
}

/// Remembers (or, with `None`, forgets) the choice for `path`.
pub(crate) fn remember(path: &Path, charset: Option<Charset>) {
    with(|s| match charset {
        Some(c) => {
            s.remembered.insert(path.to_path_buf(), c);
        }
        None => {
            s.remembered.remove(path);
        }
    });
}

/// The answer to "what does this diff look like in this encoding".
#[derive(Debug, Clone)]
pub(crate) enum Status {
    Ready(Arc<Recoded>),
    /// Asked, not back yet; `since` is when, for the feedback after
    /// [`FEEDBACK_AFTER`].
    Loading {
        since: Instant,
    },
    Failed(String),
}

/// Asks for (or finds) the re-read of `original` as `want` says. A diff
/// that changed since the read is read again; the old answer stays
/// available meanwhile, so a refresh does not flash.
pub(crate) fn resolve(want: &Want, original: &FileDiff) -> Status {
    with(|s| {
        let request = want.request(original.status);
        if let Some(entry) = s.entries.iter_mut().find(|e| e.request == request) {
            if entry.for_diff != *original {
                entry.for_diff = original.clone();
                s.outbox.push(request);
            }
            return match &entry.state {
                State::Loading => Status::Loading { since: entry.since },
                State::Ready(r) => Status::Ready(Arc::clone(r)),
                State::Failed(why) => Status::Failed(why.clone()),
            };
        }
        let since = Instant::now();
        if s.entries.len() >= KEEP {
            s.entries.remove(0);
        }
        s.entries.push(Entry {
            request: request.clone(),
            for_diff: original.clone(),
            since,
            state: State::Loading,
        });
        s.outbox.push(request);
        Status::Loading { since }
    })
}

/// The reads the draw asked for since the last call (sent by the event
/// loop, off the draw closure).
pub(crate) fn take_requests() -> Vec<FileInsightRequest> {
    with(|s| std::mem::take(&mut s.outbox))
}

/// Takes the link's answer when it is a re-read's, handing anything else
/// back. A stale answer (the entry is gone) is swallowed.
pub(crate) fn offer(outcome: FileInsightOutcome) -> Option<FileInsightOutcome> {
    if !matches!(outcome.request.kind, FileInsightKind::DiffRecoded { .. }) {
        return Some(outcome);
    }
    with(|s| {
        let Some(entry) = s.entries.iter_mut().find(|e| e.request == outcome.request) else {
            return;
        };
        entry.state = match outcome.result {
            Ok(FileInsightPayload::Recoded {
                mut diff,
                new_lines,
            }) => {
                diff.status = entry.for_diff.status;
                State::Ready(Arc::new(Recoded {
                    diff,
                    new_lines: Arc::new(new_lines),
                }))
            }
            Ok(_) => State::Failed(
                gitpoulpi_i18n::msg::diff_view::ENC_UNEXPECTED
                    .get()
                    .to_string(),
            ),
            Err(e) => State::Failed(e.to_string()),
        };
    });
    None
}

/// When the "reading as ..." word is due for the earliest re-read still
/// out and not yet showing it (the loop wakes for it, see `idle.rs`).
pub(crate) fn next_deadline() -> Option<Instant> {
    with(|s| {
        s.entries
            .iter()
            .filter(|e| matches!(e.state, State::Loading))
            .map(|e| e.since + FEEDBACK_AFTER)
            .filter(|due| *due > Instant::now())
            .min()
    })
}

#[cfg(test)]
pub(crate) fn reset() {
    STORE.with(|s| *s.borrow_mut() = None);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The store is process-wide: the tests that use it take turns.
    fn serial() -> std::sync::MutexGuard<'static, ()> {
        static TURN: Mutex<()> = Mutex::new(());
        TURN.lock().unwrap_or_else(|p| p.into_inner())
    }

    fn added(diff: &FileDiff) -> Vec<&str> {
        diff.hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| l.origin == gitpoulpi_core::domain::LineOrigin::Add)
            .map(|l| l.content.as_str())
            .collect()
    }

    #[test]
    fn a_latin_1_file_reads_right_once_the_encoding_is_chosen() {
        let old = b"caf\xe9\nna\xefve\n";
        let new = b"caf\xe9\nna\xefve\nd\xe9j\xe0 vu\n";
        let (diff, lines) = recode(
            Path::new("f.txt"),
            Some(old),
            Some(new),
            Charset::Windows1252,
        )
        .unwrap();
        assert_eq!(added(&diff), ["d\u{e9}j\u{e0} vu"]);
        assert_eq!(lines, ["caf\u{e9}", "na\u{ef}ve", "d\u{e9}j\u{e0} vu"]);
        assert_eq!(diff.status, FileStatus::Modified);
        // The same bytes are not UTF-8: reading them as such is refused.
        assert!(recode(Path::new("f.txt"), Some(old), Some(new), Charset::Utf8).is_err());
    }

    #[test]
    fn a_utf_16_file_is_diffed_though_git_calls_it_binary() {
        let utf16 = |text: &str| Charset::Utf16Le.encode(text).unwrap();
        let (diff, _) = recode(
            Path::new("w.txt"),
            Some(&utf16("one\ntwo\n")),
            Some(&utf16("one\ntwo\nthr\u{e9}e\n")),
            Charset::Utf16Le,
        )
        .unwrap();
        assert_eq!(added(&diff), ["thr\u{e9}e"]);
        // The BOM is not a character of the first line.
        assert!(diff.hunks[0]
            .lines
            .iter()
            .all(|l| !l.content.contains('\u{feff}')));
    }

    #[test]
    fn an_added_file_has_one_hunk_and_no_old_side() {
        let (diff, lines) =
            recode(Path::new("n"), None, Some(b"a\r\nb\r\n"), Charset::Utf8).unwrap();
        assert_eq!(diff.status, FileStatus::Added);
        assert_eq!(added(&diff), ["a", "b"]);
        assert_eq!(lines, ["a", "b"], "a CRLF is not part of the line");
        assert_eq!(diff.hunks[0].old_lines, 0);
        assert_eq!(diff.hunks[0].old_start, 0);
    }

    #[test]
    fn the_choice_is_remembered_per_path() {
        let _turn = serial();
        reset();
        assert_eq!(remembered(Path::new("a.txt")), None);
        remember(Path::new("a.txt"), Some(Charset::Koi8R));
        assert_eq!(remembered(Path::new("a.txt")), Some(Charset::Koi8R));
        assert_eq!(remembered(Path::new("b.txt")), None);
        remember(Path::new("a.txt"), None);
        assert_eq!(remembered(Path::new("a.txt")), None);
    }

    #[test]
    fn a_read_is_asked_once_answered_and_asked_again_when_the_diff_changes() {
        let _turn = serial();
        reset();
        let want = Want::of_commit(Oid([1; 20]), PathBuf::from("f.txt"), Charset::Windows1252);
        let original = FileDiff {
            path: PathBuf::from("f.txt"),
            status: FileStatus::Modified,
            hunks: Vec::new(),
        };
        assert!(matches!(resolve(&want, &original), Status::Loading { .. }));
        let asked = take_requests();
        assert_eq!(asked.len(), 1);
        assert!(matches!(
            asked[0].kind,
            FileInsightKind::DiffRecoded {
                old: Some(DiffContentSide::ParentOf(_)),
                new: Some(DiffContentSide::CommitAt(_)),
                ..
            }
        ));
        // Asking again while it is out does not ask again.
        let _ = resolve(&want, &original);
        assert!(take_requests().is_empty());
        let (diff, new_lines) = recode(
            Path::new("f.txt"),
            Some(b"a\n"),
            Some(b"b\n"),
            Charset::Utf8,
        )
        .unwrap();
        assert!(offer(FileInsightOutcome {
            request: asked[0].clone(),
            result: Ok(FileInsightPayload::Recoded { diff, new_lines }),
        })
        .is_none());
        assert!(matches!(resolve(&want, &original), Status::Ready(_)));
        // The diff moved under it: read again, the old answer still shown.
        let mut changed = original.clone();
        changed.status = FileStatus::Renamed;
        assert!(matches!(resolve(&want, &changed), Status::Ready(_)));
        assert_eq!(take_requests().len(), 1);
    }

    #[test]
    fn an_added_file_asks_for_no_old_side() {
        let want = Want::of_commit(Oid([2; 20]), PathBuf::from("n.txt"), Charset::Utf8);
        let FileInsightKind::DiffRecoded { old, new, .. } = want.request(FileStatus::Added).kind
        else {
            panic!("a recode");
        };
        assert_eq!((old, new.is_some()), (None, true));
        let FileInsightKind::DiffRecoded { old, new, .. } = want.request(FileStatus::Deleted).kind
        else {
            panic!("a recode");
        };
        assert_eq!((old.is_some(), new), (true, None));
    }
}
