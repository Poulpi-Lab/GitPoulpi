//! The one editing core every text field in the TUI types through
//!
//! One copy of the caret, selection and key handling, so fields cannot
//! disagree on details such as which one takes `Ctrl+U` or filters AltGr
//! characters out:
//!
//! - [`Caret`] is the caret and the selection of one field, as byte
//!   offsets into its text. It sits *beside* the field's `String` rather
//!   than wrapping it, so every reader of the text (filters, submits,
//!   renders) is untouched, and a field whose text is replaced from outside
//!   (a folder chosen in the picker, a path completed) just has its caret
//!   clamped back into range on the next use.
//! - [`Caret::on_key`] is the key map: characters, `Ctrl+A` select all,
//!   `Ctrl+U` / `Ctrl+K` delete to the start / end of the line, Backspace
//!   and Delete, the arrows (with Shift extending the selection and Ctrl
//!   jumping words), Home/End, and `Ctrl+C` / `Ctrl+X` handing the
//!   selection back for the clipboard. Enter, Esc and Tab are never
//!   claimed: what they do is the field's business, not the editor's.
//! - [`FieldEditor`] is the owner-side glue: one caret for whichever of
//!   a surface's fields has the keyboard, reset whenever that changes,
//!   plus the per-frame record of where each field was drawn so a mouse
//!   press, drag or double-click can be turned into a caret position or a
//!   selection.
//!
//! The drawing half (turning text + caret into a scrolled, highlighted
//! row of spans) is `text_field::edit_spans`.

use std::cell::{Cell, RefCell};
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;

use crate::text_field;
use crate::text_history::{History, Kind, Snap};

/// The caret and selection of one text field. Offsets are bytes into the
/// field's text and are clamped (to its length, then down to a char
/// boundary) on every use, so a caret can never point outside a string
/// that was changed behind its back.
#[derive(Debug, Clone, Default)]
pub(crate) struct Caret {
    /// `None` is "at the end", which is where a field's caret starts and
    /// where it stays while the text is only ever appended to - so a
    /// field whose text is set from outside keeps its caret after it.
    cursor: Option<usize>,
    /// The other end of the selection, when there is one.
    anchor: Option<usize>,
    /// First visible char of the caret's line, recorded by the last
    /// render (`text_field::edit_spans`) and read back by clicks so a
    /// click maps through the same horizontal scroll that was drawn.
    pub(crate) scroll: Cell<usize>,
    /// First visible line of a multi-line field, likewise.
    pub(crate) top: Cell<usize>,
    /// What `Ctrl+Z` / `Ctrl+Y` walk (see `text_history`).
    history: History,
}

/// What a key did to a field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum KeyEdit {
    /// The text changed.
    Edited,
    /// Only the caret or the selection moved.
    Moved,
    /// `Ctrl+C` with a selection: this text wants to go on the clipboard.
    Copy(String),
    /// `Ctrl+X` with a selection: this text was removed from the field
    /// and wants to go on the clipboard.
    Cut(String),
    /// Not an editing key: the caller's own handling applies (Enter,
    /// Esc, Tab, Up/Down in a single-line field, other chords).
    Ignored,
}

/// The editing operations, decoded from a key before touching any text,
/// so "does the editor claim this key?" is answerable without mutating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Insert(char),
    SelectAll,
    KillToLineStart,
    KillToLineEnd,
    Copy,
    Cut,
    Backspace,
    Delete,
    Undo,
    Redo,
    Move(Motion, bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Motion {
    Left,
    Right,
    WordLeft,
    WordRight,
    LineStart,
    LineEnd,
    TextStart,
    TextEnd,
    Up,
    Down,
}

/// Decodes `key` into an editing operation, or `None` when the key is
/// not the editor's.
///
/// Ctrl+letter arrives two ways depending on the terminal: as
/// `Char('a')` with `CONTROL` set, or - some Windows consoles, some
/// Unix ttys - as the bare C0 control character (`\x01` for `Ctrl+A`).
/// Both spellings are recognized. `CONTROL | ALT` together is AltGr on
/// Windows (`@`, `#`, `€` on Swiss and French layouts) and types its
/// character, unless that character is an ASCII letter or digit, which
/// only a real `Ctrl+Alt` chord produces.
fn decode(key: &KeyEvent, multiline: bool) -> Option<Op> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let control_char = |c: char| match c {
        '\u{1}' => Some(Op::SelectAll),
        '\u{15}' => Some(Op::KillToLineStart),
        '\u{b}' => Some(Op::KillToLineEnd),
        '\u{3}' => Some(Op::Copy),
        '\u{18}' => Some(Op::Cut),
        '\u{1a}' => Some(Op::Undo),
        '\u{19}' => Some(Op::Redo),
        _ => None,
    };
    match key.code {
        KeyCode::Char(c) if c.is_control() => control_char(c),
        KeyCode::Char(c) if ctrl && alt => (!c.is_ascii_alphanumeric()).then_some(Op::Insert(c)),
        KeyCode::Char(c) if ctrl => match c.to_ascii_lowercase() {
            'a' => Some(Op::SelectAll),
            'u' => Some(Op::KillToLineStart),
            'k' => Some(Op::KillToLineEnd),
            'c' => Some(Op::Copy),
            'x' => Some(Op::Cut),
            'z' if shift => Some(Op::Redo),
            'z' => Some(Op::Undo),
            'y' => Some(Op::Redo),
            _ => None,
        },
        KeyCode::Char(_) if alt => None,
        KeyCode::Char(c) => Some(Op::Insert(c)),
        KeyCode::Backspace if !alt => Some(Op::Backspace),
        KeyCode::Delete => Some(Op::Delete),
        KeyCode::Left if ctrl => Some(Op::Move(Motion::WordLeft, shift)),
        KeyCode::Right if ctrl => Some(Op::Move(Motion::WordRight, shift)),
        KeyCode::Left => Some(Op::Move(Motion::Left, shift)),
        KeyCode::Right => Some(Op::Move(Motion::Right, shift)),
        KeyCode::Home if ctrl && multiline => Some(Op::Move(Motion::TextStart, shift)),
        KeyCode::End if ctrl && multiline => Some(Op::Move(Motion::TextEnd, shift)),
        KeyCode::Home => Some(Op::Move(Motion::LineStart, shift)),
        KeyCode::End => Some(Op::Move(Motion::LineEnd, shift)),
        KeyCode::Up if multiline => Some(Op::Move(Motion::Up, shift)),
        KeyCode::Down if multiline => Some(Op::Move(Motion::Down, shift)),
        _ => None,
    }
}

/// The largest char boundary of `text` at or below `at`.
fn floor_boundary(text: &str, at: usize) -> usize {
    let mut at = at.min(text.len());
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

fn prev_boundary(text: &str, at: usize) -> usize {
    text[..at].char_indices().next_back().map_or(0, |(i, _)| i)
}

fn next_boundary(text: &str, at: usize) -> usize {
    text[at..].chars().next().map_or(at, |c| at + c.len_utf8())
}

fn line_start(text: &str, at: usize) -> usize {
    text[..at].rfind('\n').map_or(0, |i| i + 1)
}

fn line_end(text: &str, at: usize) -> usize {
    text[at..].find('\n').map_or(text.len(), |i| at + i)
}

/// The three kinds of run a double-click or a word jump sees: word
/// characters, whitespace, and everything else (punctuation, symbols).
pub(crate) fn char_class(c: char) -> u8 {
    if c.is_alphanumeric() || c == '_' {
        0
    } else if c.is_whitespace() {
        1
    } else {
        2
    }
}

fn word_left(text: &str, at: usize) -> usize {
    let mut chars = text[..at].char_indices().rev().peekable();
    // Skip the whitespace just before the caret, then the run before it.
    while chars.peek().is_some_and(|(_, c)| c.is_whitespace()) {
        chars.next();
    }
    let Some(&(mut start, first)) = chars.peek() else {
        return 0;
    };
    let class = char_class(first);
    for (i, c) in chars {
        if char_class(c) != class {
            break;
        }
        start = i;
    }
    start
}

fn word_right(text: &str, at: usize) -> usize {
    let mut chars = text[at..].char_indices().peekable();
    let Some(&(_, first)) = chars.peek() else {
        return at;
    };
    let class = char_class(first);
    let mut end = text.len();
    let mut in_run = true;
    for (i, c) in chars {
        if in_run && char_class(c) == class && !c.is_whitespace() {
            continue;
        }
        in_run = false;
        if !c.is_whitespace() {
            end = at + i;
            break;
        }
    }
    end
}

/// The char column of byte offset `at` within its line.
fn column_of(text: &str, at: usize) -> usize {
    text[line_start(text, at)..at].chars().count()
}

/// The byte offset of char column `col` on the line starting at `start`,
/// or the line's end when it is shorter.
fn offset_at_column(text: &str, start: usize, col: usize) -> usize {
    let end = line_end(text, start);
    text[start..end]
        .char_indices()
        .nth(col)
        .map_or(end, |(i, _)| start + i)
}

impl Caret {
    /// The caret's byte offset into `text`.
    pub(crate) fn cursor(&self, text: &str) -> usize {
        floor_boundary(text, self.cursor.unwrap_or(text.len()))
    }

    /// The selected byte range, ordered, when it is not empty.
    pub(crate) fn selection(&self, text: &str) -> Option<(usize, usize)> {
        let anchor = floor_boundary(text, self.anchor?);
        let cursor = self.cursor(text);
        (anchor != cursor).then(|| (anchor.min(cursor), anchor.max(cursor)))
    }

    /// The selected text, when there is any.
    pub(crate) fn selected_text<'a>(&self, text: &'a str) -> Option<&'a str> {
        self.selection(text).map(|(a, b)| &text[a..b])
    }

    /// Moves the caret to `at`, extending the selection from where the
    /// caret was when `extend` is set and dropping it otherwise.
    pub(crate) fn set_cursor(&mut self, text: &str, at: usize, extend: bool) {
        if extend {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor(text));
            }
        } else {
            self.anchor = None;
        }
        self.cursor = Some(floor_boundary(text, at));
    }

    /// `text` drawn as one `mask` per char (a password field), and this
    /// caret moved onto the masked string, sharing its scroll so a
    /// render of the mask is what a click maps through.
    pub(crate) fn masked(&self, text: &str, mask: char) -> (String, Caret) {
        let to = |at: usize| text[..floor_boundary(text, at)].chars().count() * mask.len_utf8();
        let shown = mask.to_string().repeat(text.chars().count());
        let caret = Caret {
            cursor: Some(to(self.cursor(text))),
            anchor: self.anchor.map(to),
            scroll: Cell::new(self.scroll.get()),
            top: Cell::new(self.top.get()),
            history: History::default(),
        };
        (shown, caret)
    }

    pub(crate) fn select_all(&mut self, text: &str) {
        self.anchor = Some(0);
        self.cursor = Some(text.len());
    }

    /// Selects the word (or whitespace run, or punctuation run) under
    /// byte offset `at` - the double-click rule. At the very end of the
    /// text the run before it is taken.
    pub(crate) fn select_word_at(&mut self, text: &str, at: usize) {
        let at = floor_boundary(text, at);
        let probe = if at == text.len() || text[at..].starts_with('\n') {
            if at == 0 || text[..at].ends_with('\n') {
                self.set_cursor(text, at, false);
                return;
            }
            prev_boundary(text, at)
        } else {
            at
        };
        let class = char_class(text[probe..].chars().next().expect("probe is inside text"));
        let same = |c: char| c != '\n' && char_class(c) == class;
        let start = text[..probe]
            .char_indices()
            .rev()
            .take_while(|(_, c)| same(*c))
            .last()
            .map_or(probe, |(i, _)| i);
        let end = text[probe..]
            .char_indices()
            .find(|(_, c)| !same(*c))
            .map_or(text.len(), |(i, _)| probe + i);
        self.anchor = Some(start);
        self.cursor = Some(end);
    }

    /// Removes the selection from `text`, leaving the caret where it
    /// was. Returns whether there was one.
    fn delete_selection(&mut self, text: &mut String) -> bool {
        match self.selection(text) {
            Some((a, b)) => {
                text.replace_range(a..b, "");
                self.anchor = None;
                self.cursor = Some(a);
                true
            }
            None => {
                self.anchor = None;
                false
            }
        }
    }

    /// Scrolls a multi-line field's view to start at line `top` (clamped)
    /// of `rows` visible, and moves the caret onto the nearest visible
    /// line if the scroll left it outside - so the caret stays visible
    /// and the next render does not scroll straight back to it. Used by
    /// the scrollbar and the wheel.
    pub(crate) fn scroll_lines_to(&mut self, text: &str, top: usize, rows: usize) {
        let rows = rows.max(1);
        let lines = text.split('\n').count();
        let top = top.min(lines.saturating_sub(rows));
        self.top.set(top);
        let cursor = self.cursor(text);
        let caret_line = text[..cursor].matches('\n').count();
        let (code, times) = if caret_line < top {
            (KeyCode::Down, top - caret_line)
        } else if caret_line >= top + rows {
            (KeyCode::Up, caret_line + 1 - (top + rows))
        } else {
            return;
        };
        let mut scratch = text.to_string();
        for _ in 0..times {
            self.on_key(&mut scratch, &KeyEvent::new(code, KeyModifiers::NONE), true);
        }
    }

    /// Types `s` at the caret, replacing the selection if there is one.
    /// This is also the paste path.
    pub(crate) fn insert(&mut self, text: &mut String, s: &str) {
        let before = self.snap(text);
        self.insert_raw(text, s);
        self.record(before, Kind::Block, text);
    }

    fn insert_raw(&mut self, text: &mut String, s: &str) {
        self.delete_selection(text);
        let at = self.cursor(text);
        text.insert_str(at, s);
        self.cursor = Some(at + s.len());
    }

    /// Replaces the bytes `range` of `text` by `with` and puts the caret after
    /// it: one history entry (an autocomplete accepted).
    pub(crate) fn replace_range(
        &mut self,
        text: &mut String,
        range: std::ops::Range<usize>,
        with: &str,
    ) {
        let before = self.snap(text);
        let end = range.start + with.len();
        text.replace_range(range, with);
        self.cursor = Some(end);
        self.anchor = None;
        self.record(before, Kind::Block, text);
    }

    /// The field as it is now, for the history.
    fn snap(&self, text: &str) -> Snap {
        Snap {
            text: text.to_string(),
            cursor: self.cursor(text),
            anchor: self.anchor,
        }
    }

    /// Records an edit that took the field from `before` to `text`, when it
    /// changed anything.
    fn record(&mut self, before: Snap, kind: Kind, text: &str) {
        if before.text != text {
            let cursor = self.cursor(text);
            self.history.record(before, kind, text, cursor);
        }
    }

    fn restore(&mut self, text: &mut String, snap: Snap) {
        *text = snap.text;
        self.cursor = Some(snap.cursor);
        self.anchor = snap.anchor;
    }

    /// Undoes the last edit of `text`. Whether there was one.
    pub(crate) fn undo(&mut self, text: &mut String) -> bool {
        let current = self.snap(text);
        match self.history.undo(current) {
            Some(back) => {
                self.restore(text, back);
                true
            }
            None => false,
        }
    }

    /// Redoes the last undone edit of `text`. Whether there was one.
    pub(crate) fn redo(&mut self, text: &mut String) -> bool {
        let current = self.snap(text);
        match self.history.redo(current) {
            Some(next) => {
                self.restore(text, next);
                true
            }
            None => false,
        }
    }

    /// Whether [`Caret::undo`] would change `text` (the Undo button's state).
    pub(crate) fn can_undo(&self, text: &str) -> bool {
        self.history.can_undo(text)
    }

    /// Whether [`Caret::redo`] would change `text`.
    pub(crate) fn can_redo(&self, text: &str) -> bool {
        self.history.can_redo(text)
    }

    /// Whether [`Caret::on_key`] would do something with `key` - asked
    /// by callers whose key dispatch is a `match` with guards, so a key
    /// the editor claims never also reaches a pane shortcut. `Ctrl+C` and
    /// `Ctrl+X` are only claimed when there is a selection to copy.
    pub(crate) fn claims(&self, text: &str, key: &KeyEvent, multiline: bool) -> bool {
        match decode(key, multiline) {
            Some(Op::Copy | Op::Cut) => self.selection(text).is_some(),
            Some(_) => true,
            None => false,
        }
    }

    /// Applies one key to `text`. See the module docs for the key map.
    pub(crate) fn on_key(&mut self, text: &mut String, key: &KeyEvent, multiline: bool) -> KeyEdit {
        let Some(op) = decode(key, multiline) else {
            return KeyEdit::Ignored;
        };
        let kind = match op {
            Op::Undo | Op::Redo => {
                let done = if op == Op::Undo {
                    self.undo(text)
                } else {
                    self.redo(text)
                };
                return if done { KeyEdit::Edited } else { KeyEdit::Moved };
            }
            Op::Insert(c) if self.selection(text).is_none() => Kind::Typed(char_class(c)),
            Op::Backspace if self.selection(text).is_none() => Kind::Backspace,
            Op::Delete if self.selection(text).is_none() => Kind::Delete,
            Op::Insert(_)
            | Op::Backspace
            | Op::Delete
            | Op::KillToLineStart
            | Op::KillToLineEnd
            | Op::Cut => Kind::Block,
            _ => return self.apply(text, op),
        };
        let before = self.snap(text);
        let result = self.apply(text, op);
        self.record(before, kind, text);
        result
    }

    fn apply(&mut self, text: &mut String, op: Op) -> KeyEdit {
        let at = self.cursor(text);
        match op {
            Op::Undo | Op::Redo => KeyEdit::Ignored,
            Op::Insert(c) => {
                self.insert_raw(text, c.encode_utf8(&mut [0; 4]));
                KeyEdit::Edited
            }
            Op::SelectAll => {
                self.select_all(text);
                KeyEdit::Moved
            }
            Op::KillToLineStart | Op::KillToLineEnd => {
                // With a selection, the selection goes -- the same rule
                // as Backspace -- rather than a range the user can't see.
                if self.delete_selection(text) {
                    return KeyEdit::Edited;
                }
                let (a, b) = if op == Op::KillToLineStart {
                    (line_start(text, at), at)
                } else {
                    (at, line_end(text, at))
                };
                if a == b {
                    return KeyEdit::Moved;
                }
                text.replace_range(a..b, "");
                self.cursor = Some(a);
                KeyEdit::Edited
            }
            Op::Copy => match self.selected_text(text) {
                Some(s) => KeyEdit::Copy(s.to_string()),
                None => KeyEdit::Ignored,
            },
            Op::Cut => match self.selected_text(text).map(str::to_string) {
                Some(s) => {
                    self.delete_selection(text);
                    KeyEdit::Cut(s)
                }
                None => KeyEdit::Ignored,
            },
            Op::Backspace | Op::Delete => {
                if self.delete_selection(text) {
                    return KeyEdit::Edited;
                }
                let (a, b) = if op == Op::Backspace {
                    (prev_boundary(text, at), at)
                } else {
                    (at, next_boundary(text, at))
                };
                if a == b {
                    return KeyEdit::Moved;
                }
                text.replace_range(a..b, "");
                self.cursor = Some(a);
                KeyEdit::Edited
            }
            Op::Move(motion, extend) => {
                // A plain Left/Right with a selection collapses it to
                // that side, the way every desktop text box does.
                if !extend {
                    if let Some((a, b)) = self.selection(text) {
                        match motion {
                            Motion::Left => {
                                self.set_cursor(text, a, false);
                                return KeyEdit::Moved;
                            }
                            Motion::Right => {
                                self.set_cursor(text, b, false);
                                return KeyEdit::Moved;
                            }
                            _ => {}
                        }
                    }
                }
                let to = match motion {
                    Motion::Left => prev_boundary(text, at),
                    Motion::Right => next_boundary(text, at),
                    Motion::WordLeft => word_left(text, at),
                    Motion::WordRight => word_right(text, at),
                    Motion::LineStart => line_start(text, at),
                    Motion::LineEnd => line_end(text, at),
                    Motion::TextStart => 0,
                    Motion::TextEnd => text.len(),
                    Motion::Up | Motion::Down => {
                        let start = line_start(text, at);
                        let col = column_of(text, at);
                        if motion == Motion::Up {
                            if start == 0 {
                                0
                            } else {
                                offset_at_column(text, line_start(text, start - 1), col)
                            }
                        } else {
                            let end = line_end(text, at);
                            if end == text.len() {
                                text.len()
                            } else {
                                offset_at_column(text, end + 1, col)
                            }
                        }
                    }
                };
                self.set_cursor(text, to, extend);
                KeyEdit::Moved
            }
        }
    }
}

/// Where one field was drawn this frame: the rect its *text* occupies
/// (not its label or border), and whether it is multi-line.
#[derive(Debug, Clone, Copy)]
struct Drawn<K> {
    id: K,
    rect: Rect,
    multiline: bool,
}

/// The caret of whichever field of one surface has the keyboard, plus
/// the mouse half of editing.
///
/// A surface (the main window, a dialog) has several fields but only one is
/// ever typed into, so one [`Caret`] is enough: it belongs to `owner` and is
/// reset whenever another field takes over (entering a field starts with the
/// caret at its end and nothing selected).
#[derive(Debug, Clone)]
pub(crate) struct FieldEditor<K> {
    owner: Option<K>,
    caret: Caret,
    /// Filled by the renders of this frame, read by the next mouse event.
    drawn: RefCell<Vec<Drawn<K>>>,
    /// A left press landed in this field and the button is still down:
    /// drags extend the selection from where it went down.
    dragging: bool,
    /// Where the field being dragged in was drawn when the press landed.
    drag_rect: Option<(Rect, bool)>,
    last_click: Option<(Instant, u16, u16)>,
}

/// Where a caret sits is not part of what a form *is*: two forms holding
/// the same values are equal whatever their fields' carets, so a form
/// that derives `PartialEq` can hold an editor without its comparisons
/// (tests asserting on a form's contents) turning on caret state.
impl<K> PartialEq for FieldEditor<K> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<K> Eq for FieldEditor<K> {}

impl<K> Default for FieldEditor<K> {
    fn default() -> Self {
        Self {
            owner: None,
            caret: Caret::default(),
            drawn: RefCell::new(Vec::new()),
            dragging: false,
            drag_rect: None,
            last_click: None,
        }
    }
}

impl<K: Copy + PartialEq> FieldEditor<K> {
    /// Tells the editor which field has the keyboard now (`None`: none
    /// does). A change of owner resets the caret.
    pub(crate) fn sync(&mut self, active: Option<K>) {
        if self.owner != active {
            self.owner = active;
            self.caret = Caret::default();
            self.dragging = false;
        }
    }

    /// Puts the maintainer's caret back at the end with nothing selected --
    /// after its text was replaced wholesale (a completion accepted).
    pub(crate) fn reset(&mut self) {
        self.caret = Caret::default();
        self.dragging = false;
    }

    /// The caret of field `id`, when it is the one with the keyboard.
    pub(crate) fn caret(&self, id: K) -> Option<&Caret> {
        (self.owner == Some(id)).then_some(&self.caret)
    }

    /// The caret of field `id`, taking ownership for it first.
    pub(crate) fn caret_mut(&mut self, id: K) -> &mut Caret {
        self.sync(Some(id));
        &mut self.caret
    }

    /// Applies `key` to field `id`'s `text`.
    pub(crate) fn on_key(
        &mut self,
        id: K,
        text: &mut String,
        key: &KeyEvent,
        multiline: bool,
    ) -> KeyEdit {
        self.caret_mut(id).on_key(text, key, multiline)
    }

    /// A dialog's key for its focused field `id`: `None` when it isn't
    /// an editing key (the dialog's own handling applies), else what it
    /// did, plus the status sentence of a `Ctrl+C`/`Ctrl+X` that put the
    /// selection on the clipboard.
    pub(crate) fn dialog_key(
        &mut self,
        id: K,
        text: &mut String,
        key: &KeyEvent,
    ) -> Option<(KeyEdit, Option<String>)> {
        if !self.claims(id, text, key, false) {
            return None;
        }
        let edit = self.on_key(id, text, key, false);
        let message = match &edit {
            KeyEdit::Copy(s) | KeyEdit::Cut(s) => Some(crate::clipboard::copy_shared(
                gitpoulpi_i18n::msg::input::WHAT_SELECTION.get(),
                s,
            )),
            _ => None,
        };
        Some((edit, message))
    }

    /// A dialog's mouse event for field `id` (the focused one, when a
    /// press landed on it): a press places the caret or on a
    /// double-click selects a word, a drag selects, a release ends the
    /// drag. Returns whether the event was the editor's - a drag in
    /// progress owns every drag and the release that ends it.
    pub(crate) fn dialog_mouse(
        &mut self,
        id: Option<K>,
        text: &str,
        kind: crossterm::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        use crossterm::event::{MouseButton, MouseEventKind};
        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(id) = id.filter(|id| self.field_at(col, row) == Some(*id)) {
                    self.mouse_down(id, text, col, row, Instant::now());
                    return true;
                }
                false
            }
            MouseEventKind::Drag(MouseButton::Left) => self.mouse_drag(text, col, row),
            MouseEventKind::Up(MouseButton::Left) => {
                let was = self.is_dragging();
                self.mouse_up();
                was
            }
            _ => false,
        }
    }

    /// The mouse for a modal multi-line editor holding one field `id`
    /// (the settings paths/hooks editors): press, Shift+press, drag,
    /// release and the wheel. Returns whether the event was the field's.
    pub(crate) fn area_mouse(
        &mut self,
        id: K,
        text: &str,
        kind: crossterm::event::MouseEventKind,
        (col, row): (u16, u16),
        shift: bool,
    ) -> bool {
        use crossterm::event::{MouseButton, MouseEventKind};
        let delta: isize = match kind {
            MouseEventKind::ScrollDown => 3,
            MouseEventKind::ScrollUp => -3,
            _ => 0,
        };
        if delta != 0 {
            let Some(rect) = self.drawn_rect(id, col, row).map(|d| d.0) else {
                return false;
            };
            if !rect.contains((col, row).into()) {
                return false;
            }
            let top = self.caret(id).map_or(0, |c| c.top.get());
            let top = (top as isize + delta).max(0) as usize;
            self.caret_mut(id)
                .scroll_lines_to(text, top, rect.height as usize);
            return true;
        }
        if kind == MouseEventKind::Down(MouseButton::Left) {
            if self.field_at(col, row) != Some(id) {
                return false;
            }
            self.press(id, text, col, row, Instant::now(), shift);
            return true;
        }
        self.dialog_mouse(Some(id), text, kind, col, row)
    }

    /// Whether `key` is an editing key for field `id` - see
    /// [`Caret::claims`].
    pub(crate) fn claims(&self, id: K, text: &str, key: &KeyEvent, multiline: bool) -> bool {
        match self.caret(id) {
            Some(caret) => caret.claims(text, key, multiline),
            None => Caret::default().claims(text, key, multiline),
        }
    }

    /// Forgets last frame's field rects; the draw that follows records
    /// this frame's.
    pub(crate) fn begin_frame(&self) {
        self.drawn.borrow_mut().clear();
    }

    /// Records that field `id`'s text was drawn in `rect` this frame.
    pub(crate) fn drawn(&self, id: K, rect: Rect, multiline: bool) {
        if rect.width > 0 && rect.height > 0 {
            crate::pointer_map::field(rect);
            self.drawn.borrow_mut().push(Drawn {
                id,
                rect,
                multiline,
            });
        }
    }

    /// The field whose text rect holds (`col`, `row`), if any.
    pub(crate) fn field_at(&self, col: u16, row: u16) -> Option<K> {
        self.drawn
            .borrow()
            .iter()
            .find(|d| d.rect.contains((col, row).into()))
            .map(|d| d.id)
    }

    /// Where field `id` was drawn: the drawing under (`col`, `row`) when
    /// there is one (a field drawn twice - the commit summary, in the
    /// composer and in the Log's WIP row - is clicked in one of them),
    /// else its first.
    fn drawn_rect(&self, id: K, col: u16, row: u16) -> Option<(Rect, bool)> {
        let drawn = self.drawn.borrow();
        drawn
            .iter()
            .find(|d| d.id == id && d.rect.contains((col, row).into()))
            .or_else(|| drawn.iter().find(|d| d.id == id))
            .map(|d| (d.rect, d.multiline))
    }

    /// The byte offset a pointer at (`col`, `row`) points at in field
    /// `id` drawn in `rect`, clamped into it so a drag past the edge
    /// selects to the edge.
    fn offset_in(
        &self,
        id: K,
        (rect, multiline): (Rect, bool),
        text: &str,
        col: u16,
        row: u16,
    ) -> usize {
        let col = col.clamp(rect.x, rect.right().saturating_sub(1)) - rect.x;
        let row = row.clamp(rect.y, rect.bottom().saturating_sub(1)) - rect.y;
        let caret = self.caret(id).cloned().unwrap_or_default();
        text_field::offset_at(
            text,
            &caret,
            rect.width as usize,
            multiline,
            col as usize,
            row as usize,
        )
    }

    /// A left press at (`col`, `row`) in field `id` (which the caller
    /// has already given the keyboard): places the caret there, or on a
    /// double-click selects the word under it, and starts a drag.
    pub(crate) fn mouse_down(&mut self, id: K, text: &str, col: u16, row: u16, now: Instant) {
        self.press(id, text, col, row, now, false);
    }

    /// [`Self::mouse_down`], where `extend` (Shift held) grows the
    /// selection from the caret to the pointer instead of moving it.
    pub(crate) fn press(
        &mut self,
        id: K,
        text: &str,
        col: u16,
        row: u16,
        now: Instant,
        extend: bool,
    ) {
        let Some(drawn) = self.drawn_rect(id, col, row) else {
            return;
        };
        let at = self.offset_in(id, drawn, text, col, row);
        self.drag_rect = Some(drawn);
        let double = self.last_click.is_some_and(|(t, c, r)| {
            now.duration_since(t) < crate::DOUBLE_CLICK_WINDOW && c == col && r == row
        });
        self.last_click = Some((now, col, row));
        let caret = self.caret_mut(id);
        if extend {
            caret.set_cursor(text, at, true);
            self.dragging = true;
        } else if double {
            caret.select_word_at(text, at);
            self.dragging = false;
        } else {
            caret.set_cursor(text, at, false);
            self.dragging = true;
        }
    }

    /// A drag with the left button: extends the selection when a press
    /// started in the owning field. Returns whether it did.
    pub(crate) fn mouse_drag(&mut self, text: &str, col: u16, row: u16) -> bool {
        let Some(id) = self.owner.filter(|_| self.dragging) else {
            return false;
        };
        let Some(drawn) = self.drag_rect else {
            return false;
        };
        let at = self.offset_in(id, drawn, text, col, row);
        self.caret.set_cursor(text, at, true);
        true
    }

    /// Whether a press in a field is still being dragged.
    pub(crate) fn is_dragging(&self) -> bool {
        self.dragging && self.owner.is_some()
    }

    /// The owning field, when a drag is in progress.
    pub(crate) fn dragging_field(&self) -> Option<K> {
        self.owner.filter(|_| self.dragging)
    }

    pub(crate) fn mouse_up(&mut self) {
        self.dragging = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }
    fn ctrl(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }
    fn shift(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::SHIFT)
    }

    /// Types `keys` into a fresh field holding `start` (caret at its
    /// end) and returns the text and the caret.
    fn run(start: &str, multiline: bool, keys: &[KeyEvent]) -> (String, Caret) {
        let mut text = start.to_string();
        let mut caret = Caret::default();
        for k in keys {
            caret.on_key(&mut text, k, multiline);
        }
        (text, caret)
    }

    #[test]
    fn typing_appends_at_the_caret_which_starts_at_the_end() {
        let (text, caret) = run("ab", false, &[key(KeyCode::Char('c'))]);
        assert_eq!(text, "abc");
        assert_eq!(caret.cursor(&text), 3);
        let (text, _) = run("ac", false, &[key(KeyCode::Left), key(KeyCode::Char('b'))]);
        assert_eq!(text, "abc");
    }

    #[test]
    fn ctrl_a_selects_all_then_typing_replaces_it() {
        let (text, caret) = run("hello", false, &[ctrl('a')]);
        assert_eq!(caret.selection(&text), Some((0, 5)));
        let (text, caret) = run("hello", false, &[ctrl('a'), key(KeyCode::Char('x'))]);
        assert_eq!(text, "x");
        assert_eq!(caret.selection(&text), None);
    }

    #[test]
    fn backspace_and_delete_remove_the_selection_whole() {
        let (text, _) = run("hello", false, &[ctrl('a'), key(KeyCode::Backspace)]);
        assert_eq!(text, "");
        let (text, _) = run(
            "hello",
            false,
            &[
                key(KeyCode::Home),
                shift(KeyCode::Right),
                shift(KeyCode::Right),
                key(KeyCode::Delete),
            ],
        );
        assert_eq!(text, "llo");
    }

    #[test]
    fn shift_arrows_extend_and_shrink_the_selection() {
        let (text, caret) = run(
            "abcdef",
            false,
            &[
                shift(KeyCode::Left),
                shift(KeyCode::Left),
                shift(KeyCode::Left),
                shift(KeyCode::Right),
            ],
        );
        assert_eq!(caret.selected_text(&text), Some("ef"));
        // A plain arrow collapses the selection to its side.
        let mut caret = caret;
        let mut t = text.clone();
        caret.on_key(&mut t, &key(KeyCode::Left), false);
        assert_eq!(caret.selection(&t), None);
        assert_eq!(caret.cursor(&t), 4);
    }

    #[test]
    fn the_control_character_spellings_of_ctrl_letters_work_too() {
        let (text, caret) = run("hello", false, &[key(KeyCode::Char('\u{1}'))]);
        assert_eq!(caret.selection(&text), Some((0, 5)));
        let (text, _) = run("hello", false, &[key(KeyCode::Char('\u{15}'))]);
        assert_eq!(text, "");
    }

    #[test]
    fn ctrl_u_and_ctrl_k_on_a_single_line() {
        let mid = [key(KeyCode::Left), key(KeyCode::Left)];
        let (text, caret) = run("abcdef", false, &[&mid[..], &[ctrl('u')]].concat());
        assert_eq!(text, "ef");
        assert_eq!(caret.cursor(&text), 0);
        let (text, caret) = run("abcdef", false, &[&mid[..], &[ctrl('k')]].concat());
        assert_eq!(text, "abcd");
        assert_eq!(caret.cursor(&text), 4);
    }

    #[test]
    fn ctrl_u_and_ctrl_k_stop_at_the_current_lines_ends() {
        // Caret in the middle line, after "mid".
        let keys = [key(KeyCode::Up), key(KeyCode::End)];
        let (text, _) = run(
            "one\nmiddle\nlast",
            true,
            &[
                &keys[..],
                &[key(KeyCode::Left)],
                &[key(KeyCode::Left)],
                &[key(KeyCode::Left)],
                &[ctrl('u')],
            ]
            .concat(),
        );
        assert_eq!(text, "one\ndle\nlast");
        let (text, _) = run(
            "one\nmiddle\nlast",
            true,
            &[
                &keys[..],
                &[key(KeyCode::Left)],
                &[key(KeyCode::Left)],
                &[key(KeyCode::Left)],
                &[ctrl('k')],
            ]
            .concat(),
        );
        assert_eq!(text, "one\nmid\nlast");
        // At a line's end, Ctrl+K has nothing to delete -- it does not
        // join the next line.
        let (text, _) = run(
            "one\nmiddle\nlast",
            true,
            &[&keys[..], &[ctrl('k')]].concat(),
        );
        assert_eq!(text, "one\nmiddle\nlast");
    }

    #[test]
    fn up_and_down_keep_the_column_in_multiline_and_are_ignored_in_single_line() {
        let (text, caret) = run("abcdef\nxy", true, &[key(KeyCode::Up)]);
        assert_eq!(caret.cursor(&text), 2);
        let mut t = "abc".to_string();
        assert_eq!(
            Caret::default().on_key(&mut t, &key(KeyCode::Up), false),
            KeyEdit::Ignored
        );
    }

    #[test]
    fn enter_esc_and_tab_are_never_claimed() {
        let caret = Caret::default();
        for code in [KeyCode::Enter, KeyCode::Esc, KeyCode::Tab] {
            assert!(!caret.claims("x", &key(code), true));
        }
        assert!(caret.claims("x", &key(KeyCode::Char('q')), false));
        assert!(!caret.claims("x", &ctrl('p'), false));
    }

    #[test]
    fn copy_and_cut_need_a_selection() {
        let caret = Caret::default();
        assert!(!caret.claims("abc", &ctrl('c'), false));
        let mut text = "abc".to_string();
        let mut caret = Caret::default();
        caret.select_all(&text);
        assert!(caret.claims(&text, &ctrl('c'), false));
        assert_eq!(
            caret.on_key(&mut text, &ctrl('c'), false),
            KeyEdit::Copy("abc".into())
        );
        assert_eq!(text, "abc");
        assert_eq!(
            caret.on_key(&mut text, &ctrl('x'), false),
            KeyEdit::Cut("abc".into())
        );
        assert_eq!(text, "");
    }

    #[test]
    fn altgr_characters_type_but_real_ctrl_alt_chords_do_not() {
        let altgr = KeyModifiers::CONTROL | KeyModifiers::ALT;
        let (text, _) = run("a", false, &[KeyEvent::new(KeyCode::Char('@'), altgr)]);
        assert_eq!(text, "a@");
        let normalized = |c| {
            let crossterm::event::Event::Key(k) = crate::keymap::normalize_event(
                crossterm::event::Event::Key(KeyEvent::new(KeyCode::Char(c), altgr)),
            ) else {
                unreachable!()
            };
            k
        };
        let (text, _) = run(
            "a",
            false,
            &[normalized('@'), normalized('{'), normalized('\\')],
        );
        assert_eq!(text, "a@{\\");
        let (text, _) = run("a", false, &[KeyEvent::new(KeyCode::Char('u'), altgr)]);
        assert_eq!(text, "a");
    }

    #[test]
    fn double_click_word_selection() {
        let text = "fix: the parser_bug  now";
        let mut caret = Caret::default();
        caret.select_word_at(text, 11); // inside "parser_bug"
        assert_eq!(caret.selected_text(text), Some("parser_bug"));
        caret.select_word_at(text, 3); // the colon
        assert_eq!(caret.selected_text(text), Some(":"));
        caret.select_word_at(text, 20); // the double space
        assert_eq!(caret.selected_text(text), Some("  "));
        caret.select_word_at(text, text.len()); // past the end: last word
        assert_eq!(caret.selected_text(text), Some("now"));
        // Multi-line: a word never runs across a newline.
        let text = "ab\ncd";
        caret.select_word_at(text, 3);
        assert_eq!(caret.selected_text(text), Some("cd"));
        caret.select_word_at(text, 2);
        assert_eq!(caret.selected_text(text), Some("ab"));
    }

    #[test]
    fn word_jumps() {
        let text = "foo bar.baz";
        assert_eq!(word_left(text, text.len()), 8);
        assert_eq!(word_left(text, 4), 0);
        assert_eq!(word_right(text, 0), 4);
        assert_eq!(word_right(text, 4), 7);
    }

    #[test]
    fn wide_and_multibyte_chars_are_edited_as_whole_chars() {
        let (text, _) = run(
            "日本語",
            false,
            &[key(KeyCode::Left), key(KeyCode::Backspace)],
        );
        assert_eq!(text, "日語");
        let (text, _) = run("🚀x", false, &[key(KeyCode::Home), key(KeyCode::Delete)]);
        assert_eq!(text, "x");
    }

    #[test]
    fn a_caret_left_behind_by_an_outside_change_is_clamped() {
        let mut caret = Caret::default();
        let mut text = "abcdef".to_string();
        caret.on_key(&mut text, &key(KeyCode::Left), false);
        let mut shorter = "ab".to_string();
        assert_eq!(caret.cursor(&shorter), 2);
        caret.on_key(&mut shorter, &key(KeyCode::Char('c')), false);
        assert_eq!(shorter, "abc");
    }

    #[test]
    fn the_editor_resets_the_caret_when_another_field_takes_over() {
        let mut editor = FieldEditor::<u8>::default();
        let mut text = "abc".to_string();
        editor.on_key(1, &mut text, &ctrl('a'), false);
        assert!(editor.caret(1).unwrap().selection(&text).is_some());
        editor.sync(Some(2));
        assert!(editor.caret(1).is_none());
        editor.sync(Some(1));
        assert!(editor.caret(1).unwrap().selection(&text).is_none());
    }

    #[test]
    fn click_drag_and_double_click_through_the_drawn_rect() {
        let mut editor = FieldEditor::<u8>::default();
        let text = "hello world";
        editor.drawn(7, Rect::new(10, 3, 30, 1), false);
        editor.sync(Some(7));
        let t0 = Instant::now();
        editor.mouse_down(7, text, 12, 3, t0);
        assert_eq!(editor.caret(7).unwrap().cursor(text), 2);
        assert!(editor.mouse_drag(text, 16, 3));
        assert_eq!(editor.caret(7).unwrap().selected_text(text), Some("llo "));
        // Dragging past the right edge selects to the end.
        editor.mouse_drag(text, 200, 3);
        assert_eq!(
            editor.caret(7).unwrap().selected_text(text),
            Some("llo world")
        );
        editor.mouse_up();
        assert!(!editor.mouse_drag(text, 12, 3));
        // Double-click on "world".
        let t1 = t0 + std::time::Duration::from_secs(5);
        editor.mouse_down(7, text, 18, 3, t1);
        editor.mouse_up();
        editor.mouse_down(7, text, 18, 3, t1 + std::time::Duration::from_millis(100));
        assert_eq!(editor.caret(7).unwrap().selected_text(text), Some("world"));
    }

    /// The modal multi-line editors: wide glyphs map to whole chars, Shift
    /// extends, a drag selects across lines, the wheel scrolls.
    #[test]
    fn area_mouse_maps_wide_glyphs_extends_with_shift_and_scrolls() {
        use crossterm::event::{MouseButton as B, MouseEventKind as K};
        let mut editor = FieldEditor::<()>::default();
        let text = "日本語abc\nsecond\nthird\nfourth\nfifth";
        editor.caret_mut(());
        editor.drawn((), Rect::new(10, 2, 20, 2), true);
        // Column 12 is the second wide glyph (cells 12-13): before 本.
        assert!(editor.area_mouse((), text, K::Down(B::Left), (12, 2), false));
        assert_eq!(editor.caret(()).unwrap().cursor(text), "日".len());
        editor.area_mouse((), text, K::Up(B::Left), (12, 2), false);
        // The second half of 本 still lands before 本.
        editor.area_mouse((), text, K::Down(B::Left), (13, 2), false);
        assert_eq!(editor.caret(()).unwrap().cursor(text), "日".len());
        editor.area_mouse((), text, K::Up(B::Left), (13, 2), false);
        // Shift+click on `a` (cell 16) extends from the caret.
        editor.area_mouse((), text, K::Down(B::Left), (16, 2), true);
        assert_eq!(editor.caret(()).unwrap().selected_text(text), Some("本語"));
        // A drag onto the next line selects across the newline.
        editor.area_mouse((), text, K::Up(B::Left), (16, 2), true);
        editor.area_mouse((), text, K::Down(B::Left), (10, 2), false);
        assert!(editor.area_mouse((), text, K::Drag(B::Left), (12, 3), false));
        assert_eq!(
            editor.caret(()).unwrap().selected_text(text),
            Some("日本語abc\nse")
        );
        editor.area_mouse((), text, K::Up(B::Left), (12, 3), false);
        // A press outside the field is not the editor's.
        assert!(!editor.area_mouse((), text, K::Down(B::Left), (2, 0), false));
        // The wheel scrolls the view, only over the field.
        assert!(editor.area_mouse((), text, K::ScrollDown, (12, 2), false));
        assert_eq!(editor.caret(()).unwrap().top.get(), 3);
        assert!(!editor.area_mouse((), text, K::ScrollDown, (2, 0), false));
        assert!(editor.area_mouse((), text, K::ScrollUp, (12, 2), false));
        assert_eq!(editor.caret(()).unwrap().top.get(), 0);
    }

    // ---- the edit history (Ctrl+Z / Ctrl+Y) ----

    fn typed(caret: &mut Caret, text: &mut String, s: &str) {
        for c in s.chars() {
            caret.on_key(text, &key(KeyCode::Char(c)), true);
        }
    }

    /// Typing groups per word: "hello world" is three entries (hello, the
    /// space, world), so three Ctrl+Z walk back to nothing.
    #[test]
    fn typing_is_undone_a_word_at_a_time_and_redone() {
        let mut caret = Caret::default();
        let mut text = String::new();
        typed(&mut caret, &mut text, "hello world");
        assert_eq!(caret.on_key(&mut text, &ctrl('z'), true), KeyEdit::Edited);
        assert_eq!(text, "hello ");
        caret.on_key(&mut text, &ctrl('z'), true);
        assert_eq!(text, "hello");
        caret.on_key(&mut text, &ctrl('z'), true);
        assert_eq!(text, "");
        assert_eq!(caret.on_key(&mut text, &ctrl('z'), true), KeyEdit::Moved);
        // Ctrl+Y and Ctrl+Shift+Z both redo.
        caret.on_key(&mut text, &ctrl('y'), true);
        assert_eq!(text, "hello");
        let shifted = KeyEvent::new(
            KeyCode::Char('Z'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        caret.on_key(&mut text, &shifted, true);
        assert_eq!(text, "hello ");
        assert_eq!(caret.cursor(&text), 6);
        // The bare control characters some terminals send.
        caret.on_key(&mut text, &key(KeyCode::Char('\u{1a}')), true);
        assert_eq!(text, "hello");
        caret.on_key(&mut text, &key(KeyCode::Char('\u{19}')), true);
        assert_eq!(text, "hello ");
    }

    /// A new edit after an undo forgets the redo branch.
    #[test]
    fn a_new_edit_clears_the_redo_branch() {
        let mut caret = Caret::default();
        let mut text = String::new();
        typed(&mut caret, &mut text, "ab");
        caret.on_key(&mut text, &ctrl('z'), true);
        assert!(caret.can_redo(&text));
        typed(&mut caret, &mut text, "x");
        assert!(!caret.can_redo(&text));
        assert_eq!(text, "x");
    }

    #[test]
    fn a_pause_starts_a_new_entry_and_backspaces_group() {
        let mut caret = Caret::default();
        caret.history.set_pause(std::time::Duration::ZERO);
        let mut text = String::new();
        typed(&mut caret, &mut text, "abc");
        assert_eq!(caret.history.len(), 3, "no grouping across pauses");
        let mut caret = Caret::default();
        let mut text = "abcdef".to_string();
        for _ in 0..3 {
            caret.on_key(&mut text, &key(KeyCode::Backspace), true);
        }
        assert_eq!(text, "abc");
        assert_eq!(caret.history.len(), 1, "backspaces are one entry");
        caret.on_key(&mut text, &ctrl('z'), true);
        assert_eq!(text, "abcdef");
    }

    /// Paste, cut and kill are one entry each and restore caret and selection.
    #[test]
    fn paste_cut_and_kill_are_single_entries() {
        let mut caret = Caret::default();
        let mut text = "one two".to_string();
        caret.insert(&mut text, " three four");
        assert_eq!(text, "one two three four");
        caret.on_key(&mut text, &ctrl('a'), true);
        let cut = caret.on_key(&mut text, &ctrl('x'), true);
        assert!(matches!(cut, KeyEdit::Cut(_)));
        assert_eq!(text, "");
        caret.on_key(&mut text, &ctrl('z'), true);
        assert_eq!(text, "one two three four");
        assert_eq!(
            caret.selection(&text),
            Some((0, text.len())),
            "the selection comes back with the text"
        );
        caret.on_key(&mut text, &ctrl('z'), true);
        assert_eq!(text, "one two");
        caret.on_key(&mut text, &ctrl('u'), true);
        assert_eq!(text, "");
        caret.on_key(&mut text, &ctrl('z'), true);
        assert_eq!(text, "one two");
    }

    #[test]
    fn the_history_is_bounded() {
        let mut caret = Caret::default();
        caret.history.set_pause(std::time::Duration::ZERO);
        let mut text = String::new();
        let n = crate::text_history::MAX_ENTRIES;
        typed(&mut caret, &mut text, &"x".repeat(n + 50));
        assert_eq!(caret.history.len(), n);
        // The oldest are gone: the walk back stops short of empty.
        let mut steps = 0;
        while caret.undo(&mut text) {
            steps += 1;
        }
        assert_eq!(steps, n);
        assert_eq!(text.len(), 50);
    }

    /// A field whose text was replaced from outside has no history: an
    /// entry would bring back text it no longer holds.
    #[test]
    fn the_history_is_cleared_when_the_text_is_replaced_from_outside() {
        let mut caret = Caret::default();
        let mut text = String::new();
        typed(&mut caret, &mut text, "abc");
        assert!(caret.can_undo(&text));
        text = "from outside".to_string();
        assert!(!caret.can_undo(&text));
        assert_eq!(caret.on_key(&mut text, &ctrl('z'), true), KeyEdit::Moved);
        assert_eq!(text, "from outside");
        // New typing starts a fresh history.
        typed(&mut caret, &mut text, "!");
        caret.on_key(&mut text, &ctrl('z'), true);
        assert_eq!(text, "from outside");
        // The owner's reset drops it too.
        let mut editor = FieldEditor::<u8>::default();
        let mut text = String::new();
        editor.on_key(1, &mut text, &key(KeyCode::Char('q')), false);
        editor.reset();
        assert_eq!(
            editor.on_key(1, &mut text, &ctrl('z'), false),
            KeyEdit::Moved
        );
        assert_eq!(text, "q");
    }

    /// The editor claims the chords, so a focused field never lets them reach
    /// the global git undo.
    #[test]
    fn a_field_claims_ctrl_z_and_ctrl_y() {
        let caret = Caret::default();
        assert!(caret.claims("", &ctrl('z'), false));
        assert!(caret.claims("", &ctrl('y'), false));
    }
}
