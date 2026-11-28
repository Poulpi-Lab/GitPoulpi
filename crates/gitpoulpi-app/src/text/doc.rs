//! The built-in editor's document: the file's text,
//! the caret, undo/redo, and the byte-exact round trip back to disk.
//!
//! # One `String`, the caret from `text_edit`
//!
//! The text is held as one `String` whose line breaks are always `\n`,
//! and the caret and selection are a [`Caret`] from `text_edit.rs`, the same
//! editing core every text field types through, so the editor's
//! `Ctrl+A/U/K/C/X`, word jumps, Backspace and Delete are the fields' own,
//! not a second copy. What the editor adds on top is what a field never
//! needed: vertical moves by *display* column (a tab is
//! several cells), auto-indent on Enter, Tab / Shift+Tab indenting in the
//! file's own style, and undo.
//!
//! A line index (`line_starts`) is rebuilt after every change. That is a
//! scan for `\n` over the whole text -- a few microseconds for a file of
//! several thousand lines, cheaper than keeping an incremental index
//! right, and the only per-keystroke cost that grows with the file.
//!
//! # Undo is computed, not declared
//!
//! Every change goes through [`Doc::edit_with`], which diffs the text
//! before and after (common prefix and suffix) and records the one
//! replaced range. So the `Caret`'s operations need no undo hooks of
//! their own, and anything that edits -- a key, a paste, an indent --
//! is undoable the same way. Consecutive typing (and consecutive
//! Backspace / Delete) merges into one step, broken at a word boundary,
//! the way desktop editors group it.
//!
//! # Byte-exact round trip, and the file's format
//!
//! [`Doc::decode`] reads the bytes in the encoding they announce (a BOM),
//! else UTF-8, else Windows-1252 (shown on the status bar, never silently:
//! the chip says which). Binary (a NUL byte outside UTF-16) is refused. The
//! document's `Format` is the encoding plus the kind of line break
//! (`editor_charset`): a file whose breaks are all CRLF (or all CR) is held
//! with `\n` and written back with that break; a mix stays exactly as it
//! was read (`Eol::Mixed`, a stray `\r` staying in the text) until the user
//! converts it. The final newline, or its absence, is simply the last byte
//! of the text. So [`Doc::encode`] of an unedited document gives back the
//! bytes that were read: through the encoding itself, or, for the rare file
//! whose bytes the decode / encode pair does not reproduce (a duplicate
//! code point in a multi-byte page), by keeping those bytes (`pristine`)
//! until the first edit.
//!
//! Changing the format (convert the encoding, convert the line breaks) is
//! an undo step like any edit: the `Change` carries the format it came
//! from and the one it went to, and the dirty marker compares text and
//! format with what was last saved.

use unicode_width::UnicodeWidthStr;

use super::{Caret, KeyEdit, Op};
use gitpoulpi_core::textconv::{self as charset, Charset, DecodeError, Eol, Unencodable};
use gitpoulpi_i18n::msg::editor as msg;

/// Columns between tab stops when a tab is drawn (`editor.tab_size`).
pub fn tab_width() -> usize {
    crate::editor_prefs::tab_size()
}

/// The largest file the editor opens. Past this a terminal editor is the
/// wrong tool, and the per-keystroke re-index would start to show.
pub const MAX_EDIT_BYTES: u64 = 8 * 1024 * 1024;

/// Why a file cannot be opened in the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadRefusal {
    /// It holds a NUL byte: binary, not text.
    Binary,
    /// Forced to UTF-8 and not valid in it; `at` is the offset of the first bad byte.
    NotUtf8 {
        at: usize,
    },
    /// Forced to an encoding the bytes are not valid in (Reopen with Encoding).
    Undecodable(Charset),
    TooLarge(u64),
}

impl LoadRefusal {
    /// The status-line sentence, naming the file.
    pub fn message(&self, path: &str) -> String {
        match self {
            LoadRefusal::Binary => msg::binary(path),
            LoadRefusal::NotUtf8 { at } => msg::not_utf8(path, at),
            LoadRefusal::Undecodable(cs) => msg::not_encoding(path, cs.name()),
            LoadRefusal::TooLarge(size) => {
                msg::too_large(path, size, MAX_EDIT_BYTES / (1024 * 1024))
            }
        }
    }
}

/// How the file indents, which is what `Tab` inserts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Indent {
    Tabs,
    Spaces(usize),
}

/// How the document is written back: its encoding and its line breaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Format {
    pub charset: Charset,
    /// The kind of break a save writes for each `\n` of the text (and,
    /// for [`Eol::Mixed`], none: the text is written as it is).
    pub eol: Eol,
    /// The user chose `eol` explicitly: it then wins over `editor.eol`.
    pub eol_set: bool,
}

/// A key as the document understands it: a frontend decodes its own key
/// events into these and calls [`Doc::apply_key`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocKey {
    /// Up / Down by one line, PageUp / PageDown by a page: this many lines
    /// (negative is up), keeping the goal column.
    Lines(isize),
    /// Enter: a line break plus the current indentation.
    Enter,
    /// Tab: indent in the file's own style.
    Tab,
    /// Shift+Tab: outdent.
    BackTab,
    /// Home: first non-blank, then column 0.
    SmartHome,
    /// Everything else: a field editing operation (`None`: the key is not
    /// an editing key and does nothing) and how its undo step may merge.
    Caret { op: Option<Op>, group: Group },
}

/// How a change may merge into the undo step before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Typing,
    Deleting,
    Alone,
}

/// One undo step: `removed` was replaced by `inserted` at byte `at`.
#[derive(Debug, Clone)]
struct Change {
    at: usize,
    removed: String,
    inserted: String,
    group: Group,
    /// An opaque tag from the owner of the document (the merge editor's
    /// pick operations): a tagged step never merges with its neighbors,
    /// and undo / redo hand the tag back so the owner can restore what
    /// lives beside the text.
    tag: Option<u64>,
    /// A change of format (encoding, line breaks) made by this step:
    /// `(before, after)`. Undo restores the first, redo the second.
    fmt: Option<(Format, Format)>,
}

/// One replaced range of the text, as the owner of a change-logged
/// document sees it: `removed` bytes at `at` became `inserted` bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextChange {
    pub at: usize,
    pub removed: usize,
    pub inserted: usize,
}

/// The document.
pub struct Doc {
    text: String,
    /// The text as last read or saved, for the dirty marker - compared
    /// only when the text changes, so an edit undone back to the saved
    /// state is clean again.
    saved: String,
    dirty: bool,
    /// How a save writes the document, and how it was when last saved.
    fmt: Format,
    saved_fmt: Format,
    /// The bytes as read, kept only for a file the encoding does not
    /// reproduce exactly (see the module docs); written back while the
    /// document is unchanged.
    pristine: Option<Vec<u8>>,
    indent: Indent,
    /// Columns per tab stop for this file, when set from the status bar;
    /// otherwise `editor.tab_size`.
    tab: Option<usize>,
    line_starts: Vec<usize>,
    pub caret: Caret,
    /// The display column Up/Down aim for, kept across a run of vertical
    /// moves so passing a short line does not lose it.
    goal_col: Option<usize>,
    undo: Vec<Change>,
    redo: Vec<Change>,
    /// The next change starts a new undo step whatever it is.
    sealed: bool,
    /// The first line changed since [`Doc::take_changed_from`] was last
    /// asked - where the highlighter's cache goes stale.
    changed_from: Option<usize>,
    /// Bumped on every change, so a caller can cache what it derives.
    version: u64,
    /// Every change since [`Doc::take_text_changes`], when logging was
    /// switched on by [`Doc::from_text`] (anchors kept beside the text
    /// follow edits through it). `None` for the editor, which needs none.
    log: Option<Vec<TextChange>>,
}

impl Doc {
    /// Reads a file's bytes, or says why it will not be edited. The
    /// encoding is the one a BOM announces, else UTF-8, else Windows-1252.
    pub fn decode(bytes: &[u8]) -> Result<Doc, LoadRefusal> {
        Doc::decode_as(bytes, None)
    }

    /// Like [`Doc::decode`], with the encoding forced when `forced` is set
    /// (Reopen with Encoding).
    pub fn decode_as(bytes: &[u8], forced: Option<Charset>) -> Result<Doc, LoadRefusal> {
        if bytes.len() as u64 > MAX_EDIT_BYTES {
            return Err(LoadRefusal::TooLarge(bytes.len() as u64));
        }
        let announced = Charset::from_bom(bytes);
        let wanted = forced.or(announced);
        if !wanted.is_some_and(Charset::is_utf16) && bytes.contains(&0) {
            return Err(LoadRefusal::Binary);
        }
        let (text, charset) = match wanted {
            Some(cs) => cs.decode(bytes).map_err(|e| match (e, cs) {
                (DecodeError::Utf8 { at }, _) => LoadRefusal::NotUtf8 { at },
                (DecodeError::Invalid, cs) => LoadRefusal::Undecodable(cs),
            })?,
            None => match Charset::Utf8.decode(bytes) {
                Ok(read) => read,
                // Not UTF-8: the legacy default, every byte of which means something.
                Err(_) => Charset::Windows1252
                    .decode(bytes)
                    .map_err(|_| LoadRefusal::Undecodable(Charset::Windows1252))?,
            },
        };
        let eol = charset::detect_eol(&text);
        let text = match eol {
            Eol::Crlf | Eol::Cr => charset::normalize_eol(&text),
            _ => text,
        };
        let fmt = Format {
            charset,
            eol,
            eol_set: false,
        };
        let mut doc = Doc::new(text, fmt);
        // A file the encoding does not reproduce keeps its bytes until edited.
        if doc.try_encode().ok().as_deref() != Some(bytes) {
            doc.pristine = Some(bytes.to_vec());
        }
        Ok(doc)
    }

    /// A document over text already decoded (the merge editor's Output,
    /// composed from a merge): `text` has `\n` line breaks and no BOM,
    /// `crlf` / `bom` say how a save writes it. Every change is logged
    /// for [`Doc::take_text_changes`].
    pub fn from_text(text: String, crlf: bool, bom: bool) -> Doc {
        let mut doc = Doc::new(
            text,
            Format {
                charset: if bom { Charset::Utf8Bom } else { Charset::Utf8 },
                eol: if crlf { Eol::Crlf } else { Eol::Lf },
                eol_set: false,
            },
        );
        doc.log = Some(Vec::new());
        doc
    }

    /// The changes made since the last call, oldest first (empty unless
    /// the document came from [`Doc::from_text`]).
    pub fn take_text_changes(&mut self) -> Vec<TextChange> {
        self.log.as_mut().map(std::mem::take).unwrap_or_default()
    }

    fn new(text: String, fmt: Format) -> Doc {
        let indent = detect_indent(&text);
        let mut doc = Doc {
            saved: text.clone(),
            text,
            dirty: false,
            fmt,
            saved_fmt: fmt,
            pristine: None,
            indent,
            tab: None,
            line_starts: Vec::new(),
            caret: Caret::default(),
            goal_col: None,
            undo: Vec::new(),
            redo: Vec::new(),
            sealed: true,
            changed_from: None,
            version: 0,
            log: None,
        };
        doc.reindex();
        doc.caret.set_cursor(&doc.text, 0, false);
        doc
    }

    /// The bytes to write (tests; a save goes through [`Doc::try_encode`]).
    #[cfg(test)]
    pub fn encode(&self) -> Vec<u8> {
        self.try_encode().expect("the encoding holds the text")
    }

    /// The text as written: each `\n` as the document's break.
    fn body(&self) -> std::borrow::Cow<'_, str> {
        use std::borrow::Cow;
        match self.eol() {
            Eol::Lf | Eol::Mixed => Cow::Borrowed(&self.text),
            Eol::Crlf => Cow::Owned(self.text.replace('\n', "\r\n")),
            Eol::Cr => Cow::Owned(self.text.replace('\n', "\r")),
        }
    }

    /// The bytes to write, or what the encoding cannot hold.
    pub fn try_encode(&self) -> Result<Vec<u8>, Unencodable> {
        if let (Some(bytes), false) = (&self.pristine, self.dirty) {
            return Ok(bytes.clone());
        }
        self.fmt.charset.encode(&self.body()).ok_or_else(|| {
            self.fmt
                .charset
                .unencodable(&self.text)
                .unwrap_or(Unencodable {
                    count: 1,
                    line: 1,
                    ch: '?',
                })
        })
    }

    /// The text now matches what is on disk.
    pub fn mark_saved(&mut self) {
        self.saved = self.text.clone();
        self.saved_fmt = self.fmt;
        self.pristine = None;
        self.dirty = false;
        self.sealed = true;
    }

    /// Makes `text` the saved state the dirty marker compares against
    /// (a document restored from a cache is dirty against the fresh one).
    pub fn set_baseline(&mut self, text: &str) {
        self.saved = text.to_string();
        self.dirty = self.text != self.saved || self.fmt != self.saved_fmt;
    }

    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    /// The kind of line break a save writes: the file's own, unless
    /// `editor.eol` forces one - and the user's explicit choice (the status
    /// bar's menu) wins over that. A mix is only ever written as it is.
    pub fn eol(&self) -> Eol {
        use gitpoulpi_config::EditorEol;
        if self.fmt.eol_set || self.fmt.eol == Eol::Mixed {
            return self.fmt.eol;
        }
        match crate::editor_prefs::eol() {
            EditorEol::Auto => self.fmt.eol,
            EditorEol::Lf => Eol::Lf,
            EditorEol::Crlf => Eol::Crlf,
        }
    }
    pub fn charset(&self) -> Charset {
        self.fmt.charset
    }
    /// Columns per tab stop in this file.
    pub fn tab_cols(&self) -> usize {
        self.tab.unwrap_or_else(crate::editor_prefs::tab_size)
    }
    /// The status bar's indentation chip: what Tab inserts.
    pub fn indent_label(&self) -> String {
        match self.indent {
            Indent::Tabs => msg::indent_tabs_size(self.tab_cols()),
            Indent::Spaces(n) => msg::indent_spaces(n),
        }
    }
    pub fn indent(&self) -> Indent {
        self.indent
    }
    pub fn version(&self) -> u64 {
        self.version
    }
    /// How many undo steps there are (a step merged into the one before
    /// it does not count).
    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }
    /// The tag on the newest undo step, if it has one.
    pub fn last_tag(&self) -> Option<u64> {
        self.undo.last().and_then(|c| c.tag)
    }
    /// Tags the newest undo step. Later typing may still merge into it,
    /// so a tagged typing step stays one step.
    pub fn set_last_tag(&mut self, tag: u64) {
        if let Some(c) = self.undo.last_mut() {
            c.tag = Some(tag);
        }
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    // -- lines ---------------------------------------------------------

    fn reindex(&mut self) {
        self.line_starts.clear();
        self.line_starts.push(0);
        self.line_starts.extend(
            self.text
                .bytes()
                .enumerate()
                .filter(|(_, b)| *b == b'\n')
                .map(|(i, _)| i + 1),
        );
    }

    /// Lines, counting the (empty) one after a final newline, the way
    /// editors show it.
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    pub fn line_start(&self, line: usize) -> usize {
        self.line_starts[line.min(self.line_starts.len() - 1)]
    }

    /// Byte offset of the end of `line`, before its `\n`.
    pub fn line_end(&self, line: usize) -> usize {
        self.line_starts
            .get(line + 1)
            .map_or(self.text.len(), |next| next - 1)
    }

    pub fn line(&self, line: usize) -> &str {
        &self.text[self.line_start(line)..self.line_end(line)]
    }

    /// The line holding byte offset `at`.
    pub fn line_of(&self, at: usize) -> usize {
        match self.line_starts.binary_search(&at) {
            Ok(i) => i,
            Err(i) => i - 1,
        }
    }

    pub fn cursor(&self) -> usize {
        self.caret.cursor(&self.text)
    }

    pub fn selection(&self) -> Option<(usize, usize)> {
        self.caret.selection(&self.text)
    }

    /// The caret's line and display column.
    pub fn cursor_line_col(&self) -> (usize, usize) {
        let at = self.cursor();
        let line = self.line_of(at);
        let start = self.line_start(line);
        (
            line,
            visual_col_in(&self.text[start..at], at - start, self.tab_cols()),
        )
    }

    /// The first line changed since the last call, if any.
    pub fn take_changed_from(&mut self) -> Option<usize> {
        self.changed_from.take()
    }

    // -- editing -------------------------------------------------------

    /// Runs `f` over the caret and the text, and records whatever it
    /// changed as one undo step.
    fn edit_with<R>(&mut self, group: Group, f: impl FnOnce(&mut Caret, &mut String) -> R) -> R {
        let before = self.text.clone();
        let out = f(&mut self.caret, &mut self.text);
        self.record(&before, group);
        out
    }

    /// Replaces `start..end` with `new` as one undo step carrying `tag`,
    /// even when the text comes out the same (the maintainer's own state
    /// changed, and undo must find the step). The caret goes to the end
    /// of the inserted text.
    pub fn replace_tagged(&mut self, start: usize, end: usize, new: &str, tag: u64) {
        let before = self.text.clone();
        let (start, end) = (start.min(before.len()), end.min(before.len()));
        self.text.replace_range(start..end, new);
        self.caret.set_cursor(&self.text, start + new.len(), false);
        self.goal_col = None;
        self.record_tagged(&before, start, Some(tag));
    }

    fn record(&mut self, before: &str, group: Group) {
        self.record_inner(before, group, None, 0, None);
    }

    fn record_tagged(&mut self, before: &str, at: usize, tag: Option<u64>) {
        self.record_inner(before, Group::Alone, tag, at, None);
    }

    fn record_inner(
        &mut self,
        before: &str,
        group: Group,
        tag: Option<u64>,
        tag_at: usize,
        fmt: Option<(Format, Format)>,
    ) {
        let (at, old_end, new_end) = match changed_range(before, &self.text) {
            Some(r) => r,
            None if tag.is_some() || fmt.is_some() => (
                tag_at.min(self.text.len()),
                tag_at.min(self.text.len()),
                tag_at.min(self.text.len()),
            ),
            None => return,
        };
        let change = Change {
            at,
            removed: before[at..old_end].to_string(),
            inserted: self.text[at..new_end].to_string(),
            group,
            tag,
            fmt,
        };
        self.patch_index(at, &change.removed, &change.inserted);
        self.redo.clear();
        let merged = tag.is_none()
            && fmt.is_none()
            && !self.sealed
            && self
                .undo
                .last_mut()
                .is_some_and(|last| merge_into(last, &change));
        if self.log.is_some() {
            let at = self.pinned_at(before, &change);
            if let Some(log) = self.log.as_mut() {
                log.push(TextChange {
                    at,
                    removed: change.removed.len(),
                    inserted: change.inserted.len(),
                });
            }
        }
        if !merged {
            self.undo.push(change);
        }
        self.sealed = group == Group::Alone;
        self.after_change(at);
    }

    /// Where `change` really happened. The diff between the old and new
    /// text cannot tell typing a line break at the end of a line from
    /// typing it at the start of the next, nor `a` before `a1` from after
    /// the first `a`; the caret, left where the edit ended, can. Used for
    /// the change log only (anchors beside the text care which side of a
    /// boundary an edit fell on). Falls back to the diff's position when
    /// the caret's reading would not reproduce the text.
    fn pinned_at(&self, before: &str, change: &Change) -> usize {
        let (rem, ins) = (change.removed.len(), change.inserted.len());
        let cur = self.caret.cursor(&self.text);
        let want = if ins == 0 {
            cur
        } else {
            cur.saturating_sub(ins)
        };
        if want == change.at {
            return want;
        }
        let ok = before.get(want..want + rem) == Some(change.removed.as_str())
            && self.text.get(want..want + ins) == Some(change.inserted.as_str())
            && before.get(..want) == self.text.get(..want)
            && before.get(want + rem..) == self.text.get(want + ins..);
        if ok {
            want
        } else {
            change.at
        }
    }

    /// The line index follows a change (`removed` bytes at `at` became
    /// `inserted`) instead of being rebuilt: a rescan of a multi-megabyte
    /// Output on every keystroke is most of the cost of typing in it.
    fn patch_index(&mut self, at: usize, removed: &str, inserted: &str) {
        let (rem, ins) = (removed.len(), inserted.len());
        // Entries are the starts of lines: a newline at byte `n` makes `n + 1`.
        let lo = self.line_starts.partition_point(|&p| p <= at);
        let hi = self.line_starts.partition_point(|&p| p <= at + rem);
        if ins != rem {
            for p in &mut self.line_starts[hi..] {
                *p = *p + ins - rem;
            }
        }
        let new = inserted
            .bytes()
            .enumerate()
            .filter(|(_, b)| *b == b'\n')
            .map(|(i, _)| at + i + 1);
        self.line_starts.splice(lo..hi, new);
        #[cfg(debug_assertions)]
        {
            let patched = self.line_starts.clone();
            self.reindex();
            debug_assert_eq!(patched, self.line_starts, "line index drifted");
        }
    }

    fn after_change(&mut self, at: usize) {
        let line = self.line_of(at.min(self.text.len()));
        self.changed_from = Some(self.changed_from.map_or(line, |l| l.min(line)));
        self.dirty = self.text != self.saved || self.fmt != self.saved_fmt;
        self.version += 1;
    }

    pub fn undo(&mut self) -> bool {
        self.undo_tagged().is_some()
    }

    /// Undoes one step; `Some(tag)` when it did, the tag being the one
    /// the step was recorded with (`None` inside for plain typing).
    pub fn undo_tagged(&mut self) -> Option<Option<u64>> {
        let c = self.undo.pop()?;
        if let Some(log) = self.log.as_mut() {
            log.push(TextChange {
                at: c.at,
                removed: c.inserted.len(),
                inserted: c.removed.len(),
            });
        }
        if let Some((from, _)) = c.fmt {
            self.fmt = from;
        }
        self.text
            .replace_range(c.at..c.at + c.inserted.len(), &c.removed);
        self.patch_index(c.at, &c.inserted, &c.removed);
        self.caret
            .set_cursor(&self.text, c.at + c.removed.len(), false);
        self.after_change(c.at);
        let tag = c.tag;
        self.redo.push(c);
        self.sealed = true;
        self.goal_col = None;
        Some(tag)
    }

    pub fn redo(&mut self) -> bool {
        self.redo_tagged().is_some()
    }

    /// Redoes one step, like [`Doc::undo_tagged`].
    pub fn redo_tagged(&mut self) -> Option<Option<u64>> {
        let c = self.redo.pop()?;
        if let Some(log) = self.log.as_mut() {
            log.push(TextChange {
                at: c.at,
                removed: c.removed.len(),
                inserted: c.inserted.len(),
            });
        }
        if let Some((_, to)) = c.fmt {
            self.fmt = to;
        }
        self.text
            .replace_range(c.at..c.at + c.removed.len(), &c.inserted);
        self.patch_index(c.at, &c.removed, &c.inserted);
        self.caret
            .set_cursor(&self.text, c.at + c.inserted.len(), false);
        self.after_change(c.at);
        let tag = c.tag;
        self.undo.push(c);
        self.sealed = true;
        self.goal_col = None;
        Some(tag)
    }

    /// Inserts `s` at the caret, replacing the selection - the paste
    /// path. Line breaks are normalized to the document's `\n`.
    pub fn insert_text(&mut self, s: &str) {
        let clean = s.replace("\r\n", "\n").replace('\r', "\n");
        self.goal_col = None;
        self.edit_with(Group::Alone, |caret, text| caret.insert(text, &clean));
    }

    /// Moves the caret to byte `at` (a click), extending the selection
    /// when `extend` is set.
    pub fn set_cursor(&mut self, at: usize, extend: bool) {
        self.caret.set_cursor(&self.text, at, extend);
        self.goal_col = None;
        self.sealed = true;
    }

    /// Selects `a..b` with the caret at `b`.
    pub fn select_range(&mut self, a: usize, b: usize) {
        self.caret.set_cursor(&self.text, a, false);
        self.caret.set_cursor(&self.text, b, true);
        self.goal_col = None;
        self.sealed = true;
    }

    pub fn select_word_at(&mut self, at: usize) {
        self.caret.select_word_at(&self.text, at);
        self.goal_col = None;
        self.sealed = true;
    }

    /// The byte offset shown at display column `col` of `line` - the
    /// char whose cells cover `col`, or the line's end past it.
    pub fn offset_at(&self, line: usize, col: usize) -> usize {
        let line = line.min(self.line_count() - 1);
        self.line_start(line) + byte_at_visual_in(self.line(line), col, self.tab_cols())
    }

    /// Moves the caret `delta` lines up (negative) or down, keeping its
    /// display column. Past the first line it goes to the text's start,
    /// past the last to its end, as editors do.
    pub fn move_lines(&mut self, delta: isize, extend: bool) {
        let at = self.cursor();
        let line = self.line_of(at);
        let start = self.line_start(line);
        let tab = self.tab_cols();
        let goal = *self
            .goal_col
            .get_or_insert_with(|| visual_col_in(&self.text[start..at], at - start, tab));
        let target = line as isize + delta;
        let to = if target < 0 {
            0
        } else if target as usize >= self.line_count() {
            self.text.len()
        } else {
            self.offset_at(target as usize, goal)
        };
        self.caret.set_cursor(&self.text, to, extend);
        self.sealed = true;
    }

    /// Applies one decoded key. `shift` extends the selection of the
    /// motions. Undo, redo, save and the other chords are the editor's, not
    /// the document's.
    pub fn apply_key(&mut self, key: DocKey, shift: bool) -> KeyEdit {
        if let DocKey::Lines(by) = key {
            self.move_lines(by, shift);
            return KeyEdit::Moved;
        }
        self.goal_col = None;
        match key {
            DocKey::Lines(_) => unreachable!("handled above"),
            DocKey::Enter => {
                self.newline();
                KeyEdit::Edited
            }
            DocKey::Tab => {
                self.tab();
                KeyEdit::Edited
            }
            DocKey::BackTab => {
                self.outdent();
                KeyEdit::Edited
            }
            DocKey::SmartHome => {
                self.smart_home(shift);
                KeyEdit::Moved
            }
            DocKey::Caret { op, group } => {
                let group = match group {
                    Group::Deleting if self.selection().is_some() => Group::Alone,
                    other => other,
                };
                let edit = self.edit_with(group, |caret, text| match op {
                    Some(op) => caret.run(text, op),
                    None => KeyEdit::Ignored,
                });
                if edit == KeyEdit::Moved {
                    self.sealed = true;
                }
                edit
            }
        }
    }

    /// Enter: a line break, then the current line's indentation (up to
    /// the caret, so Enter inside the indentation does not double it).
    fn newline(&mut self) {
        let at = self.selection().map_or(self.cursor(), |(a, _)| a);
        let line = self.line_of(at);
        let start = self.line_start(line);
        let indent: String = self.text[start..at]
            .chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .collect();
        self.edit_with(Group::Alone, |caret, text| {
            caret.insert(text, &format!("\n{indent}"))
        });
    }

    /// The lines a selection spans for Tab / Shift+Tab: from the line of
    /// its start to the line of its end, not counting a last line the
    /// selection only reaches the very start of.
    fn selected_lines(&self) -> Option<(usize, usize)> {
        let (a, b) = self.selection()?;
        let first = self.line_of(a);
        let mut last = self.line_of(b);
        if last > first && b == self.line_start(last) {
            last -= 1;
        }
        (last > first).then_some((first, last))
    }

    fn indent_unit(&self, col: usize) -> String {
        match self.indent {
            Indent::Tabs => "\t".to_string(),
            Indent::Spaces(n) => " ".repeat(n - col % n),
        }
    }

    /// Tab: indents every line of a multi-line selection, else inserts
    /// one indent at the caret (a tab, or spaces to the next stop).
    fn tab(&mut self) {
        if let Some((first, last)) = self.selected_lines() {
            let unit = self.indent_unit(0);
            let starts: Vec<usize> = (first..=last).map(|l| self.line_start(l)).collect();
            self.edit_with(Group::Alone, |_, text| {
                for s in starts.iter().rev() {
                    text.insert_str(*s, &unit);
                }
            });
            self.select_lines(first, last);
            return;
        }
        let at = self.selection().map_or(self.cursor(), |(a, _)| a);
        let line = self.line_of(at);
        let start = self.line_start(line);
        let col = visual_col_in(&self.text[start..at], at - start, self.tab_cols());
        let unit = self.indent_unit(col);
        self.edit_with(Group::Alone, |caret, text| caret.insert(text, &unit));
    }

    /// Shift+Tab: removes one indent from the start of the caret's line,
    /// or of every line of the selection.
    fn outdent(&mut self) {
        let (first, last) = self.selected_lines().unwrap_or_else(|| {
            let l = self.line_of(self.cursor());
            (l, l)
        });
        let width = match self.indent {
            Indent::Tabs => self.tab_cols(),
            Indent::Spaces(n) => n,
        };
        let cuts: Vec<(usize, usize)> = (first..=last)
            .map(|l| {
                let line = self.line(l);
                let n = if line.starts_with('\t') {
                    1
                } else {
                    line.bytes().take(width).take_while(|b| *b == b' ').count()
                };
                (self.line_start(l), n)
            })
            .collect();
        let multi = first != last;
        self.edit_with(Group::Alone, |caret, text| {
            let cursor = caret.cursor(text);
            let mut moved = 0;
            for (s, n) in cuts.iter().rev() {
                text.replace_range(*s..*s + *n, "");
                if *s < cursor {
                    moved += (*n).min(cursor - *s);
                }
            }
            caret.set_cursor(text, cursor - moved, false);
        });
        if multi {
            self.select_lines(first, last);
        }
    }

    fn select_lines(&mut self, first: usize, last: usize) {
        let a = self.line_start(first);
        let b = self.line_end(last);
        self.caret.set_cursor(&self.text, a, false);
        self.caret.set_cursor(&self.text, b, true);
    }

    /// Home: to the first non-blank of the line, or to its very start
    /// when already there.
    fn smart_home(&mut self, extend: bool) {
        let at = self.cursor();
        let line = self.line_of(at);
        let start = self.line_start(line);
        let blank = self
            .line(line)
            .bytes()
            .take_while(|b| *b == b' ' || *b == b'\t')
            .count();
        let to = if at == start + blank {
            start
        } else {
            start + blank
        };
        self.caret.set_cursor(&self.text, to, extend);
        self.sealed = true;
    }

    /// The next (or previous) match of `query`, ASCII case-insensitive,
    /// starting from byte `from`, wrapping around the end. Returns the
    /// match's byte range.
    pub fn find(&self, query: &str, from: usize, forward: bool) -> Option<(usize, usize)> {
        if query.is_empty() {
            return None;
        }
        let hay = self.text.to_ascii_lowercase();
        let needle = query.to_ascii_lowercase();
        let from = from.min(hay.len());
        let hit = if forward {
            hay[from..]
                .find(&needle)
                .map(|i| i + from)
                .or_else(|| hay.find(&needle))
        } else {
            hay[..from].rfind(&needle).or_else(|| hay.rfind(&needle))
        }?;
        Some((hit, hit + needle.len()))
    }
}

/// The changed range between `old` and `new`: `(start, old_end,
/// new_end)`, on char boundaries, or `None` when they are equal.
/// How many leading bytes `a` and `b` share: block compares (memcmp) first,
/// a byte walk only inside the first differing block.
fn common_prefix(a: &[u8], b: &[u8]) -> usize {
    const BLOCK: usize = 512;
    let n = a.len().min(b.len());
    let mut i = 0;
    while i + BLOCK <= n && a[i..i + BLOCK] == b[i..i + BLOCK] {
        i += BLOCK;
    }
    i + a[i..n]
        .iter()
        .zip(&b[i..n])
        .take_while(|(x, y)| x == y)
        .count()
}

/// How many trailing bytes `a` and `b` share, like [`common_prefix`].
fn common_suffix(a: &[u8], b: &[u8]) -> usize {
    const BLOCK: usize = 512;
    let n = a.len().min(b.len());
    let mut i = 0;
    while i + BLOCK <= n
        && a[a.len() - i - BLOCK..a.len() - i] == b[b.len() - i - BLOCK..b.len() - i]
    {
        i += BLOCK;
    }
    i + a[..a.len() - i]
        .iter()
        .rev()
        .zip(b[..b.len() - i].iter().rev())
        .take(n - i)
        .take_while(|(x, y)| x == y)
        .count()
}

fn changed_range(old: &str, new: &str) -> Option<(usize, usize, usize)> {
    if old == new {
        return None;
    }
    let (a, b) = (old.as_bytes(), new.as_bytes());
    let mut prefix = common_prefix(a, b);
    while !old.is_char_boundary(prefix) {
        prefix -= 1;
    }
    let max_suffix = a.len().min(b.len()) - prefix;
    let mut suffix = common_suffix(&a[a.len() - max_suffix.min(a.len())..], b);
    while !old.is_char_boundary(a.len() - suffix) {
        suffix -= 1;
    }
    Some((prefix, a.len() - suffix, b.len() - suffix))
}

/// Folds `next` into `last` when they are one run of typing or of
/// deleting. Typing breaks where a word ends, so undo takes a sentence
/// back a word at a time.
fn merge_into(last: &mut Change, next: &Change) -> bool {
    match (last.group, next.group) {
        (Group::Typing, Group::Typing)
            if last.removed.is_empty()
                && next.removed.is_empty()
                && last.at + last.inserted.len() == next.at
                && !next.inserted.contains('\n') =>
        {
            let word_ends = next.inserted.starts_with(char::is_whitespace)
                && !last.inserted.ends_with(char::is_whitespace);
            if word_ends {
                return false;
            }
            last.inserted.push_str(&next.inserted);
            true
        }
        (Group::Deleting, Group::Deleting)
            if last.inserted.is_empty() && next.inserted.is_empty() =>
        {
            if next.at + next.removed.len() == last.at {
                // Backspace: the new removal is just before the last.
                last.removed.insert_str(0, &next.removed);
                last.at = next.at;
                true
            } else if next.at == last.at {
                // Delete: the new removal is just after.
                last.removed.push_str(&next.removed);
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

/// How the text indents: tabs when more lines start with a tab than with
/// spaces, else spaces, as many as the most common step between one
/// indented line and the next deeper one (4 when there is no evidence).
pub fn detect_indent(text: &str) -> Indent {
    let mut tab_lines = 0usize;
    let mut space_lines = 0usize;
    let mut steps = [0usize; 9];
    let mut prev = 0usize;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with('\t') {
            tab_lines += 1;
            continue;
        }
        let n = line.bytes().take_while(|b| *b == b' ').count();
        if n >= 2 {
            space_lines += 1;
        }
        if n > prev && n - prev <= 8 && n - prev >= 2 {
            steps[n - prev] += 1;
        }
        prev = n;
    }
    if tab_lines > space_lines {
        return Indent::Tabs;
    }
    let best = (2..=8).max_by_key(|&n| (steps[n], n == 4)).unwrap_or(4);
    if steps[best] == 0 {
        Indent::Spaces(4)
    } else {
        Indent::Spaces(best)
    }
}

/// Terminal cells `c` takes when drawn starting at column `col`.
pub fn char_cells(c: char, col: usize) -> usize {
    char_cells_in(c, col, tab_width())
}

/// [`char_cells`] with tab stops every `tab` columns (a file's own size).
pub fn char_cells_in(c: char, col: usize, tab: usize) -> usize {
    if c == '\t' {
        tab - col % tab
    } else if c.is_ascii() || c.is_control() {
        1
    } else {
        UnicodeWidthStr::width(c.encode_utf8(&mut [0; 4]) as &str).max(1)
    }
}

/// The display column of byte `at` in `line` (tabs expanded).
pub fn visual_col(line: &str, at: usize) -> usize {
    visual_col_in(line, at, tab_width())
}

pub fn visual_col_in(line: &str, at: usize, tab: usize) -> usize {
    let mut col = 0;
    for c in line[..at.min(line.len())].chars() {
        col += char_cells_in(c, col, tab);
    }
    col
}

/// The byte offset of the char covering display column `target` in
/// `line`, or the line's length when the line is shorter.
#[cfg(test)]
pub fn byte_at_visual(line: &str, target: usize) -> usize {
    byte_at_visual_in(line, target, tab_width())
}

pub fn byte_at_visual_in(line: &str, target: usize, tab: usize) -> usize {
    let mut col = 0;
    for (i, c) in line.char_indices() {
        let w = char_cells_in(c, col, tab);
        if col + w > target {
            return i;
        }
        col += w;
    }
    line.len()
}

/// The display width of `line`.
pub fn line_width(line: &str) -> usize {
    visual_col(line, line.len())
}

/// [`line_width`] with tab stops every `tab` columns.
pub fn line_width_in(line: &str, tab: usize) -> usize {
    visual_col_in(line, line.len(), tab)
}

mod format;
#[cfg(test)]
mod format_tests;
#[cfg(test)]
mod test_keys;

#[cfg(test)]
mod tests {
    use super::test_keys::{KeyCode, KeyEvent, KeyModifiers, OnKey};
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn type_str(doc: &mut Doc, s: &str) {
        for c in s.chars() {
            let code = if c == '\n' {
                KeyCode::Enter
            } else {
                KeyCode::Char(c)
            };
            doc.on_key(&key(code), 10);
        }
    }

    #[test]
    fn an_unedited_document_writes_back_the_exact_bytes() {
        for bytes in [
            &b"one\ntwo\n"[..],
            b"one\ntwo",
            b"one\r\ntwo\r\n",
            b"one\r\ntwo",
            b"mixed\r\nendings\nhere\r\n",
            b"\xEF\xBB\xBFbom\r\nfile\r\n",
            b"\tindented\n\t\ttwice\n",
            b"lone\rcarriage\r\n",
            b"cr before crlf\r\r\n",
            b"",
            b"\n",
            b"old\rmac\rfile\r",
            b"\xFF\xFEa\x00\n\x00",
            b"\xFE\xFF\x00a\x00\n",
            b"caf\xE9 \x80\r\n",
        ] {
            let doc = Doc::decode(bytes).unwrap();
            assert_eq!(doc.encode(), bytes, "{:?}", String::from_utf8_lossy(bytes));
            assert!(!doc.is_dirty());
        }
    }

    #[test]
    fn a_crlf_file_is_edited_as_lines_and_saved_with_crlf() {
        let mut doc = Doc::decode(b"alpha\r\nbeta\r\n").unwrap();
        assert_eq!(doc.eol(), Eol::Crlf);
        assert_eq!(doc.line_count(), 3, "two lines and the empty one after");
        assert_eq!(doc.line(1), "beta");
        doc.on_key(&key(KeyCode::End), 10);
        type_str(&mut doc, "!\nnew");
        assert_eq!(doc.encode(), b"alpha!\r\nnew\r\nbeta\r\n");
        assert!(doc.is_dirty());
    }

    #[test]
    fn no_final_newline_stays_absent_and_a_final_newline_stays() {
        let mut doc = Doc::decode(b"a\nb").unwrap();
        doc.on_key(&key(KeyCode::Char('x')), 10);
        assert_eq!(doc.encode(), b"xa\nb");
        let mut doc = Doc::decode(b"a\nb\n").unwrap();
        doc.on_key(&key(KeyCode::Char('x')), 10);
        assert_eq!(doc.encode(), b"xa\nb\n");
    }

    #[test]
    fn binary_is_refused_and_non_utf8_is_read_as_windows_1252() {
        assert_eq!(
            Doc::decode(b"PNG\0\x01\x02").err(),
            Some(LoadRefusal::Binary)
        );
        let doc = Doc::decode(b"caf\xE9\n").unwrap();
        assert_eq!(doc.charset(), Charset::Windows1252);
        assert_eq!(doc.text(), "caf\u{e9}\n");
        assert_eq!(doc.encode(), b"caf\xE9\n");
        // Forced to UTF-8 it is refused, naming the bad byte.
        assert_eq!(
            Doc::decode_as(b"caf\xE9\n", Some(Charset::Utf8)).err(),
            Some(LoadRefusal::NotUtf8 { at: 3 })
        );
        let message = LoadRefusal::NotUtf8 { at: 3 }.message("a.txt");
        assert!(message.contains("a.txt") && message.contains("not UTF-8"));
    }

    #[test]
    fn tab_follows_the_files_indentation() {
        let mut tabs = Doc::decode(b"fn a() {\n\tx();\n\ty();\n}\n").unwrap();
        assert_eq!(tabs.indent(), Indent::Tabs);
        tabs.on_key(&key(KeyCode::Tab), 10);
        assert!(tabs.text().starts_with("\tfn a()"));

        let mut spaces = Doc::decode(b"def a():\n  x = 1\n  if x:\n    y = 2\n").unwrap();
        assert_eq!(spaces.indent(), Indent::Spaces(2));
        spaces.on_key(&key(KeyCode::Char('z')), 10);
        spaces.on_key(&key(KeyCode::Tab), 10);
        // Column 1 -> the next 2-column stop: one space.
        assert!(spaces.text().starts_with("z def"), "{:?}", spaces.text());

        let plain = Doc::decode(b"no indentation at all\n").unwrap();
        assert_eq!(plain.indent(), Indent::Spaces(4));
    }

    #[test]
    fn existing_tabs_are_preserved_and_enter_keeps_the_indentation() {
        let mut doc = Doc::decode(b"\tfirst\n").unwrap();
        doc.on_key(&key(KeyCode::End), 10);
        type_str(&mut doc, "\nsecond");
        assert_eq!(doc.text(), "\tfirst\n\tsecond\n");
    }

    #[test]
    fn tab_and_shift_tab_indent_every_selected_line() {
        let mut doc = Doc::decode(b"a\nb\nc\n").unwrap();
        doc.select_range(0, 3); // "a\nb"
        doc.on_key(&key(KeyCode::Tab), 10);
        assert_eq!(doc.text(), "    a\n    b\nc\n");
        doc.on_key(&KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT), 10);
        assert_eq!(doc.text(), "a\nb\nc\n");
    }

    #[test]
    fn undo_takes_typing_back_a_word_at_a_time_and_redo_replays_it() {
        let mut doc = Doc::decode(b"\n").unwrap();
        type_str(&mut doc, "hello world");
        assert_eq!(doc.text(), "hello world\n");
        assert!(doc.undo());
        assert_eq!(doc.text(), "hello\n", "the second word goes first");
        assert!(doc.undo());
        assert_eq!(doc.text(), "\n");
        assert!(!doc.is_dirty(), "undone back to the saved text is clean");
        assert!(!doc.undo(), "nothing left");
        assert!(doc.redo());
        assert!(doc.redo());
        assert_eq!(doc.text(), "hello world\n");
        assert!(doc.is_dirty());
        assert!(!doc.redo());
    }

    #[test]
    fn a_new_edit_after_undo_drops_the_redo_history() {
        let mut doc = Doc::decode(b"").unwrap();
        type_str(&mut doc, "ab");
        doc.undo();
        type_str(&mut doc, "c");
        assert!(!doc.redo());
        assert_eq!(doc.text(), "c");
    }

    #[test]
    fn backspaces_merge_into_one_undo_step() {
        let mut doc = Doc::decode(b"abcdef").unwrap();
        doc.on_key(&key(KeyCode::End), 10);
        for _ in 0..3 {
            doc.on_key(&key(KeyCode::Backspace), 10);
        }
        assert_eq!(doc.text(), "abc");
        doc.undo();
        assert_eq!(doc.text(), "abcdef");
        assert_eq!(doc.cursor(), 6);
    }

    #[test]
    fn undo_restores_a_crlf_file_byte_for_byte() {
        let original = b"x\r\ny\r\n";
        let mut doc = Doc::decode(original).unwrap();
        type_str(&mut doc, "a\nb");
        doc.insert_text("pasted\r\nlines");
        while doc.undo() {}
        assert_eq!(doc.encode(), original);
        assert!(!doc.is_dirty());
    }

    #[test]
    fn up_and_down_keep_the_display_column_across_tabs_and_short_lines() {
        // Column 5 on the first line; the tab on the third line is
        // four cells, so column 5 there is after "\tx".
        let mut doc = Doc::decode(b"abcdefgh\nab\n\txyz\n").unwrap();
        doc.set_cursor(5, false);
        doc.on_key(&key(KeyCode::Down), 10);
        assert_eq!(doc.cursor_line_col(), (1, 2), "clamped to the short line");
        doc.on_key(&key(KeyCode::Down), 10);
        assert_eq!(doc.cursor_line_col(), (2, 5), "the goal column survived");
        assert_eq!(&doc.text()[doc.cursor()..doc.cursor() + 2], "yz");
    }

    #[test]
    fn the_changed_line_is_reported_for_the_highlighter() {
        let mut doc = Doc::decode(b"a\nb\nc\n").unwrap();
        assert_eq!(doc.take_changed_from(), None);
        doc.set_cursor(doc.line_start(2), false);
        doc.on_key(&key(KeyCode::Char('x')), 10);
        assert_eq!(doc.take_changed_from(), Some(2));
        assert_eq!(doc.take_changed_from(), None);
    }

    #[test]
    fn find_wraps_and_ignores_ascii_case() {
        let doc = Doc::decode(b"Foo bar foo\n").unwrap();
        assert_eq!(doc.find("foo", 1, true), Some((8, 11)));
        assert_eq!(doc.find("foo", 9, true), Some((0, 3)), "wrapped");
        assert_eq!(doc.find("FOO", 8, false), Some((0, 3)));
        assert_eq!(doc.find("zzz", 0, true), None);
    }

    #[test]
    fn display_columns_expand_tabs_and_map_back() {
        assert_eq!(visual_col("\tab", 1), 4);
        assert_eq!(visual_col("a\tb", 2), 4);
        assert_eq!(byte_at_visual("\tab", 2), 0, "inside the tab");
        assert_eq!(byte_at_visual("\tab", 4), 1);
        assert_eq!(byte_at_visual("\tab", 99), 3);
        assert_eq!(line_width("\u{4e2d}x"), 3, "a wide char is two cells");
    }

    #[test]
    fn from_text_logs_changes_and_tagged_steps_undo_with_their_tag() {
        let mut doc = Doc::from_text("a\nb\n".into(), false, false);
        assert!(!doc.is_dirty());
        doc.replace_tagged(2, 3, "BB", 7);
        assert_eq!(doc.text(), "a\nBB\n");
        // A tagged step that changes nothing is still a step.
        doc.replace_tagged(0, 0, "", 8);
        assert_eq!(doc.undo_len(), 2);
        let log = doc.take_text_changes();
        assert_eq!(
            log[0],
            TextChange {
                at: 2,
                removed: 1,
                inserted: 2
            }
        );
        assert_eq!(log.len(), 2);
        // Typing right after a tagged step is its own step, not merged.
        type_str(&mut doc, "x");
        assert_eq!(doc.undo_len(), 3);
        assert_eq!(doc.undo_tagged(), Some(None));
        assert_eq!(doc.undo_tagged(), Some(Some(8)));
        assert_eq!(doc.undo_tagged(), Some(Some(7)));
        assert_eq!(doc.undo_tagged(), None);
        assert_eq!(doc.text(), "a\nb\n");
        assert_eq!(doc.redo_tagged(), Some(Some(7)));
        assert_eq!(doc.text(), "a\nBB\n");
    }

    #[test]
    fn the_change_log_pins_typing_to_the_caret_not_the_diff() {
        let mut doc = Doc::from_text("a1\nb\n".into(), false, false);
        doc.set_cursor(0, false);
        type_str(&mut doc, "a");
        // The diff alone would say "inserted at 1".
        assert_eq!(doc.take_text_changes()[0].at, 0);
        doc.set_cursor(2, false);
        doc.on_key(&key(KeyCode::Enter), 10);
        assert_eq!(doc.take_text_changes()[0].at, 2);
        // The editor's own documents keep no log.
        let mut plain = Doc::decode(b"x\n").unwrap();
        type_str(&mut plain, "y");
        assert!(plain.take_text_changes().is_empty());
    }
}
