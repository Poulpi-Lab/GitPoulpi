//! Reading a file's *content* rather than a diff of it: the blob behind a
//! path at some revision, and per-line blame over that same blob.
//!
//! One module because both share the decision of what counts as text and
//! where a line ends: both backends call [`BlobContent::from_bytes`], so
//! `blame`'s line numbers and `read_blob`'s line vector index the same
//! lines.

use std::borrow::Cow;
use std::cell::Cell;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{b64, ImageInfo, LfsPointer, Oid};

thread_local! {
    static FALLBACK_ENCODING: Cell<Option<&'static encoding_rs::Encoding>> =
        const { Cell::new(None) };
}

/// Runs `f` with `label` (an encoding name such as `windows-1252` or
/// `latin1`) as the **fallback encoding for display**: content that is not
/// valid UTF-8 is decoded with it instead of being reported binary (blobs,
/// blame) or mangled (diff lines). `None`, `utf-8` and an unknown label
/// leave everything as it was.
///
/// This widens the read contract of `read_blob`, `blame` and the diffs
/// without a parameter on every `GitBackend` method: the daemon scopes it,
/// per request, around exactly the calls that produce text for a person to
/// read. Staging and patch operations never run inside such a scope, so
/// the bytes they write stay exact.
pub fn with_fallback_encoding<R>(label: Option<&str>, f: impl FnOnce() -> R) -> R {
    let encoding = label
        .map(str::trim)
        .and_then(|l| encoding_rs::Encoding::for_label(l.as_bytes()))
        .filter(|e| *e != encoding_rs::UTF_8);
    let previous = FALLBACK_ENCODING.with(|c| c.replace(encoding));
    struct Restore(Option<&'static encoding_rs::Encoding>);
    impl Drop for Restore {
        fn drop(&mut self) {
            FALLBACK_ENCODING.with(|c| c.set(self.0));
        }
    }
    let _restore = Restore(previous);
    f()
}

/// The fallback encoding in force on this thread, if any.
fn fallback_encoding() -> Option<&'static encoding_rs::Encoding> {
    FALLBACK_ENCODING.with(Cell::get)
}

/// The name of the fallback encoding in force on this thread (the label
/// `encoding_rs` knows it by, e.g. `windows-1252`), if any. Lets a document
/// say which encoding it was actually decoded with.
pub fn fallback_encoding_name() -> Option<&'static str> {
    fallback_encoding().map(|e| e.name())
}

/// `raw` as text: itself when it is valid UTF-8, else decoded with the
/// fallback encoding when one is in force and the bytes are valid in it.
/// `None`: not text as far as we can tell.
pub fn decode_text(raw: &[u8]) -> Option<Cow<'_, str>> {
    if let Ok(text) = std::str::from_utf8(raw) {
        return Some(Cow::Borrowed(text));
    }
    fallback_encoding()?.decode_without_bom_handling_and_without_replacement(raw)
}

/// [`decode_text`], but never failing: bytes the fallback cannot decode
/// (or no fallback at all) become U+FFFD, as `String::from_utf8_lossy`.
pub fn decode_text_lossy(raw: &[u8]) -> Cow<'_, str> {
    decode_text(raw).unwrap_or_else(|| String::from_utf8_lossy(raw))
}

/// A commit's text (message, author name) as git shows it: decoded with the
/// commit's own `encoding` header when it has a known one, otherwise as
/// [`decode_text_lossy`] (valid UTF-8 kept, else the fallback encoding).
/// Display-only: nothing is ever re-encoded on the way back.
pub fn decode_commit_text<'a>(raw: &'a [u8], header_encoding: Option<&str>) -> Cow<'a, str> {
    let declared = header_encoding
        .map(str::trim)
        .and_then(|l| encoding_rs::Encoding::for_label(l.as_bytes()))
        .filter(|e| *e != encoding_rs::UTF_8);
    match declared {
        Some(encoding) => encoding.decode_without_bom_handling(raw).0,
        None => decode_text_lossy(raw),
    }
}

/// git's commit subject (`%s`) of a decoded `message`: its first paragraph
/// with the line breaks folded to single spaces.
pub fn commit_summary(message: &str) -> String {
    let message = message.trim_start_matches('\n');
    let first = message.split("\n\n").next().unwrap_or_default();
    first
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Above this many bytes a file is reported as [`BlobContent::TooLarge`]
/// instead of being read.
///
/// The content crosses the IPC wire as JSON and the TUI turns it into one
/// `String` per line, so both backends learn a file's size *before*
/// reading it (`git cat-file -s`, `odb.read_header`).
pub const MAX_TEXT_BLOB_BYTES: u64 = 8 * 1024 * 1024;

/// How many leading bytes the binary sniff looks at. Matches git's own
/// `buffer_is_binary` heuristic so GitPoulpi calls the same files binary
/// that `git diff` does.
const BINARY_SNIFF_BYTES: usize = 8000;

/// Which of a path's three possible contents to read: the committed one,
/// the staged one or the one on disk. An explicit argument, not inferred
/// from repo state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlobSource {
    /// The file as of a commit.
    Commit(Oid),
    /// The staged content (index stage 0). An unmerged path has no stage
    /// 0 and is an error here, not a silent fallback to another stage.
    Index,
    /// The bytes currently on disk. Read straight from the filesystem by
    /// both backends - never through `git show`, which would apply
    /// smudge filters and hand back something that is not what the editor
    /// would open.
    WorkingTree,
    /// One stage of an unmerged path (1 base, 2 ours, 3 theirs), as the
    /// index holds it. Read by `read_blob_bytes` for the merge editor's
    /// picture previews.
    Stage(u8),
}

/// A file's content at one [`BlobSource`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blob {
    pub path: PathBuf,
    pub source: BlobSource,
    /// The full byte length at `source`, reported even when `content` is
    /// `Binary` or `TooLarge` - a viewer that can't show the file still
    /// wants to say how big it is.
    pub size: u64,
    pub content: BlobContent,
    /// What the header says when `content` is `Binary` and the bytes are an
    /// image (format, pixel size, ...). Read from the header alone, never
    /// decoded. `None` for anything else, and for `TooLarge` (the bytes are
    /// not read).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<ImageInfo>,
}

impl Blob {
    /// The image header of `raw` when `content` is `Binary`: both backends
    /// call this so they agree on which files are images.
    pub fn image_of(content: &BlobContent, raw: &[u8]) -> Option<ImageInfo> {
        match content {
            BlobContent::Binary => super::image_info::sniff(raw),
            _ => None,
        }
    }
}

/// A file's exact bytes at one [`BlobSource`] (see
/// `GitBackend::read_blob_bytes`): no smudge filter, no decoding, no text
/// splitting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobBytes {
    pub path: PathBuf,
    pub source: BlobSource,
    /// The full byte length at `source`, reported whatever `body` holds.
    pub size: u64,
    pub body: BlobBody,
}

/// The bytes, or the reason there are none.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlobBody {
    /// The exact bytes (base64 on the wire).
    Bytes(#[serde(with = "b64")] Vec<u8>),
    /// Larger than the cap asked for; never read.
    TooLarge,
    /// An LFS pointer. `object` is the real content when it is in the
    /// local LFS store and within the cap, else `None`.
    Lfs {
        pointer: LfsPointer,
        #[serde(with = "b64::option", default)]
        object: Option<Vec<u8>>,
    },
}

/// Text, or a reason there is no text to show.
///
/// The two non-text variants are deliberately part of the success type
/// rather than an `Err`: "this is a 4 GiB PNG" is an answer a file viewer
/// renders, not a failure it reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlobContent {
    Text {
        /// One entry per line, **without** its line terminator. See
        /// [`BlobContent::from_bytes`] for how they're split.
        lines: Vec<String>,
        /// At least one line ended `\r\n` in the raw bytes. The `\r` is
        /// stripped from `lines` either way; this records that it was
        /// there so an editor could write the file back unchanged.
        crlf: bool,
        /// The raw content ended with a line terminator. A file whose
        /// last line has no newline (git's "\ No newline at end of file")
        /// has this `false`, and `lines.len()` still counts that line.
        final_newline: bool,
    },
    /// Not showable as text. Either a NUL byte appeared in the first
    /// 8000 bytes (git's own heuristic) or the content is not valid
    /// UTF-8.
    Binary,
    /// Larger than [`MAX_TEXT_BLOB_BYTES`]; the bytes were never read.
    TooLarge,
    /// A Git LFS pointer: the blob is the 3-line stand-in git stores
    /// instead of the real (large) content. Decided by
    /// [`BlobContent::from_bytes`], so both backends agree.
    LfsPointer(LfsPointer),
}

impl BlobContent {
    /// The single place in GitPoulpi where raw bytes become lines.
    ///
    /// Both backends call this rather than splitting themselves: a split on
    /// `\n` in one and `\r\n` in the other would give the same file a
    /// different line count, and blame line N would point at different
    /// text per backend. With `core.autocrlf` a checked-out file is CRLF
    /// while its blob is LF, so `WorkingTree` and `Commit` reads only
    /// yield identical `lines` because `\r` is stripped here.
    ///
    /// Callers must check the size cap themselves *before* reading the
    /// bytes to pass in; this function only decides text vs. binary.
    pub fn from_bytes(raw: &[u8]) -> Self {
        if let Some(pointer) = LfsPointer::parse(raw) {
            return BlobContent::LfsPointer(pointer);
        }
        let sniff = &raw[..raw.len().min(BINARY_SNIFF_BYTES)];
        // A NUL means binary -- except in UTF-16, where every ASCII
        // character carries one.
        let utf16 = fallback_encoding()
            .is_some_and(|e| e == encoding_rs::UTF_16LE || e == encoding_rs::UTF_16BE);
        if sniff.contains(&0) && !utf16 {
            return BlobContent::Binary;
        }
        let decoded = if utf16 && sniff.contains(&0) {
            // NULs in UTF-16 text are valid UTF-8, so only the fallback
            // can read it.
            fallback_encoding()
                .and_then(|e| e.decode_without_bom_handling_and_without_replacement(raw))
        } else {
            decode_text(raw)
        };
        let Some(text) = decoded else {
            return BlobContent::Binary;
        };
        Self::text_of(text.as_ref())
    }

    /// `raw` as text whatever it holds: a file the user told git to treat
    /// as text (`.gitattributes`) shows as text even with a NUL in it. Bytes
    /// that are not valid text become U+FFFD.
    pub fn from_bytes_as_text(raw: &[u8]) -> Self {
        Self::text_of(decode_text_lossy(raw).as_ref())
    }

    fn text_of(text: &str) -> Self {
        let final_newline = text.is_empty() || text.ends_with('\n');
        let mut crlf = false;
        let mut lines: Vec<String> = Vec::new();
        for line in text.split('\n') {
            if let Some(stripped) = line.strip_suffix('\r') {
                crlf = true;
                lines.push(stripped.to_string());
            } else {
                lines.push(line.to_string());
            }
        }
        // `split` always yields one trailing empty element after a final
        // `\n` ("a\n" -> ["a", ""]). That is the *absence* of a next
        // line, not an empty last line, and keeping it would put a phantom
        // row under every file in the viewer and make blame's line count
        // disagree with git's by exactly one.
        if final_newline {
            lines.pop();
        }

        BlobContent::Text {
            lines,
            crlf,
            final_newline,
        }
    }

    /// The lines, or an empty slice for a binary/oversized blob.
    pub fn lines(&self) -> &[String] {
        match self {
            BlobContent::Text { lines, .. } => lines,
            _ => &[],
        }
    }
}

/// Per-line attribution for a file at one commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blame {
    pub path: PathBuf,
    /// The commit whose version of the file was blamed. Blame is always
    /// of a *committed* revision: blaming a dirty working tree (git's
    /// "Not Committed Yet" pseudo-commit) would make half the lines carry
    /// a zero oid that nothing downstream can click through to, so the
    /// caller resolves what it wants blamed and passes it here.
    pub at: Oid,
    pub size: u64,
    pub content: BlameContent,
}

/// Blame lines, or the same two reasons there are none that
/// [`BlobContent`] carries - the decision is made by the same
/// [`BlobContent::from_bytes`] call, so a file the viewer calls binary is
/// never one blame claims to have lines for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlameContent {
    Lines(Vec<BlameLine>),
    Binary,
    TooLarge,
    /// The file is an LFS pointer; there is no text to attribute.
    LfsPointer(LfsPointer),
}

impl BlameContent {
    pub fn lines(&self) -> &[BlameLine] {
        match self {
            BlameContent::Lines(lines) => lines,
            _ => &[],
        }
    }
}

/// One blamed line.
///
/// `content` is carried here rather than left to a second `read_blob`
/// round trip, so a blame pane renders from one response and the text
/// beside an attribution is provably the text that was blamed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlameLine {
    /// The commit that last touched this line - what a click in the
    /// blame gutter selects in the log.
    pub commit: Oid,
    pub author_name: String,
    pub author_email: String,
    /// Author time, seconds since the epoch, same basis as
    /// [`super::CommitNode::timestamp`].
    pub timestamp: i64,
    /// Committer time of `commit`, seconds since the epoch.
    pub committer_time: i64,
    /// First line of `commit`'s message, for the gutter tooltip.
    pub summary: String,
    /// 1-based line number in the file as of [`Blame::at`]. Equals this
    /// entry's index in `lines` plus one; stored anyway so a filtered or
    /// scrolled view can still print the real number.
    pub final_line: u32,
    /// 1-based line number this line had in `commit`'s own version of the
    /// file, which is what "jump to where this line came from" needs.
    pub orig_line: u32,
    /// The file's path in `commit` (not necessarily [`Blame::path`]: blame
    /// follows whole-file renames, so this is the old name for lines that
    /// predate the rename).
    pub orig_path: PathBuf,
    /// Where to re-blame to see what this line looked like before `commit`
    /// touched it ("blame previous revision"): `commit`'s first parent and
    /// the file's path there. `None` when `commit` has no parent or the
    /// file did not exist in it (the line was born in `commit`).
    pub previous: Option<(Oid, PathBuf)>,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_forced_to_text_keep_their_lines_even_with_a_nul() {
        let raw = b"alpha\nbe\0ta\ngamma\n";
        assert_eq!(BlobContent::from_bytes(raw), BlobContent::Binary);
        let forced = BlobContent::from_bytes_as_text(raw);
        assert_eq!(forced.lines(), ["alpha", "be\0ta", "gamma"]);
    }

    #[test]
    fn crlf_and_lf_content_produce_the_same_lines() {
        let lf = BlobContent::from_bytes(b"one\ntwo\n");
        let crlf = BlobContent::from_bytes(b"one\r\ntwo\r\n");

        assert_eq!(lf.lines(), ["one", "two"]);
        assert_eq!(crlf.lines(), ["one", "two"]);
        assert!(matches!(crlf, BlobContent::Text { crlf: true, .. }));
        assert!(matches!(lf, BlobContent::Text { crlf: false, .. }));
    }

    #[test]
    fn a_fallback_encoding_decodes_what_is_not_utf8_and_only_in_its_scope() {
        let raw = b"caf\xe9\n";
        assert_eq!(BlobContent::from_bytes(raw), BlobContent::Binary);
        with_fallback_encoding(Some("windows-1252"), || {
            assert_eq!(BlobContent::from_bytes(raw).lines(), ["caf\u{e9}"]);
            // Valid UTF-8 is never decoded a second time.
            assert_eq!(
                BlobContent::from_bytes("caf\u{e9}\n".as_bytes()).lines(),
                ["caf\u{e9}"]
            );
            // A NUL is still binary ...
            assert_eq!(BlobContent::from_bytes(b"a\0b\xe9"), BlobContent::Binary);
        });
        // ... except in UTF-16, where every ASCII character carries one.
        with_fallback_encoding(Some("utf-16le"), || {
            assert_eq!(BlobContent::from_bytes(b"h\0i\0").lines(), ["hi"]);
        });
        // utf-8 and unknown labels change nothing.
        for label in [Some("utf-8"), Some("klingon"), None] {
            with_fallback_encoding(label, || {
                assert_eq!(BlobContent::from_bytes(raw), BlobContent::Binary);
            });
        }
        assert_eq!(decode_text_lossy(raw), "caf\u{fffd}\n");
    }

    #[test]
    fn a_final_newline_does_not_add_a_phantom_line() {
        assert_eq!(BlobContent::from_bytes(b"one\n").lines().len(), 1);
        assert_eq!(BlobContent::from_bytes(b"one").lines().len(), 1);
        assert_eq!(BlobContent::from_bytes(b"").lines().len(), 0);
        assert_eq!(BlobContent::from_bytes(b"\n").lines(), [""]);
    }

    #[test]
    fn a_missing_final_newline_is_recorded() {
        assert!(matches!(
            BlobContent::from_bytes(b"one"),
            BlobContent::Text {
                final_newline: false,
                ..
            }
        ));
        assert!(matches!(
            BlobContent::from_bytes(b"one\n"),
            BlobContent::Text {
                final_newline: true,
                ..
            }
        ));
    }

    #[test]
    fn a_nul_byte_makes_it_binary() {
        assert_eq!(
            BlobContent::from_bytes(b"png\0\x01\x02"),
            BlobContent::Binary
        );
        // Invalid UTF-8 without a NUL is binary too: it cannot be
        // rendered, and lossy-decoding it would show the user text that
        // is not in the file.
        assert_eq!(
            BlobContent::from_bytes(&[0xff, 0xfe, 0x41]),
            BlobContent::Binary
        );
    }

    #[test]
    fn a_nul_past_the_sniff_window_is_still_text() {
        // git's heuristic looks only at the first 8000 bytes, and
        // GitPoulpi has to call the same files binary that `git diff`
        // does or the diff view and the file view would disagree.
        let mut raw = vec![b'a'; BINARY_SNIFF_BYTES + 10];
        raw[BINARY_SNIFF_BYTES + 5] = 0;
        assert!(matches!(
            BlobContent::from_bytes(&raw),
            BlobContent::Text { .. }
        ));
    }

    #[test]
    fn a_pointer_blob_is_an_lfs_pointer_not_text() {
        let raw = b"version https://git-lfs.github.com/spec/v1\noid sha256:4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393\nsize 12345\n";
        match BlobContent::from_bytes(raw) {
            BlobContent::LfsPointer(p) => assert_eq!(p.size, 12345),
            other => panic!("expected a pointer, got {other:?}"),
        }
        // Text that only looks like one stays text.
        assert!(matches!(
            BlobContent::from_bytes(b"version 2\nsize 3\n"),
            BlobContent::Text { .. }
        ));
    }
}
