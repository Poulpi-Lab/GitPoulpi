//! The caret, the selection and the edit operations of one text field,
//! as byte offsets into its text. Pure: a frontend decodes its own key
//! events into [`Op`]s and calls [`Caret::run`], so the terminal UI and the
//! desktop frontend edit text with the same rules.
//!
//! The terminal UI's `text_edit` adds the key decoding, the field editor and
//! the mouse half; see there for the key map.

use std::cell::Cell;

use super::history::{History, Kind, Snap};

/// The caret and selection of one text field. Offsets are bytes into the
/// field's text and are clamped (to its length, then down to a char
/// boundary) on every use, so a caret can never point outside a string
/// that was changed behind its back.
#[derive(Debug, Clone, Default)]
pub struct Caret {
    /// `None` is "at the end", which is where a field's caret starts and
    /// where it stays while the text is only ever appended to - so a
    /// field whose text is set from outside keeps its caret after it.
    cursor: Option<usize>,
    /// The other end of the selection, when there is one.
    anchor: Option<usize>,
    /// First visible char of the caret's line, recorded by the last
    /// render (`text_field::edit_spans`) and read back by clicks so a
    /// click maps through the same horizontal scroll that was drawn.
    pub scroll: Cell<usize>,
    /// First visible line of a multi-line field, likewise.
    pub top: Cell<usize>,
    /// What `Ctrl+Z` / `Ctrl+Y` walk (see `text_history`).
    pub history: History,
}

/// What a key did to a field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyEdit {
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
pub enum Op {
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
pub enum Motion {
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

/// The largest char boundary of `text` at or below `at`.
pub fn floor_boundary(text: &str, at: usize) -> usize {
    let mut at = at.min(text.len());
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

pub fn prev_boundary(text: &str, at: usize) -> usize {
    text[..at].char_indices().next_back().map_or(0, |(i, _)| i)
}

pub fn next_boundary(text: &str, at: usize) -> usize {
    text[at..].chars().next().map_or(at, |c| at + c.len_utf8())
}

pub fn line_start(text: &str, at: usize) -> usize {
    text[..at].rfind('\n').map_or(0, |i| i + 1)
}

pub fn line_end(text: &str, at: usize) -> usize {
    text[at..].find('\n').map_or(text.len(), |i| at + i)
}

/// The three kinds of run a double-click or a word jump sees: word
/// characters, whitespace, and everything else (punctuation, symbols).
pub fn char_class(c: char) -> u8 {
    if c.is_alphanumeric() || c == '_' {
        0
    } else if c.is_whitespace() {
        1
    } else {
        2
    }
}

pub fn word_left(text: &str, at: usize) -> usize {
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

pub fn word_right(text: &str, at: usize) -> usize {
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
pub fn column_of(text: &str, at: usize) -> usize {
    text[line_start(text, at)..at].chars().count()
}

/// The byte offset of char column `col` on the line starting at `start`,
/// or the line's end when it is shorter.
pub fn offset_at_column(text: &str, start: usize, col: usize) -> usize {
    let end = line_end(text, start);
    text[start..end]
        .char_indices()
        .nth(col)
        .map_or(end, |(i, _)| start + i)
}

impl Caret {
    /// The caret's byte offset into `text`.
    pub fn cursor(&self, text: &str) -> usize {
        floor_boundary(text, self.cursor.unwrap_or(text.len()))
    }

    /// The selected byte range, ordered, when it is not empty.
    pub fn selection(&self, text: &str) -> Option<(usize, usize)> {
        let anchor = floor_boundary(text, self.anchor?);
        let cursor = self.cursor(text);
        (anchor != cursor).then(|| (anchor.min(cursor), anchor.max(cursor)))
    }

    /// The selected text, when there is any.
    pub fn selected_text<'a>(&self, text: &'a str) -> Option<&'a str> {
        self.selection(text).map(|(a, b)| &text[a..b])
    }

    /// Moves the caret to `at`, extending the selection from where the
    /// caret was when `extend` is set and dropping it otherwise.
    pub fn set_cursor(&mut self, text: &str, at: usize, extend: bool) {
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
    pub fn masked(&self, text: &str, mask: char) -> (String, Caret) {
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

    pub fn select_all(&mut self, text: &str) {
        self.anchor = Some(0);
        self.cursor = Some(text.len());
    }

    /// Selects the word (or whitespace run, or punctuation run) under
    /// byte offset `at` - the double-click rule. At the very end of the
    /// text the run before it is taken.
    pub fn select_word_at(&mut self, text: &str, at: usize) {
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
    pub fn scroll_lines_to(&mut self, text: &str, top: usize, rows: usize) {
        let rows = rows.max(1);
        let lines = text.split('\n').count();
        let top = top.min(lines.saturating_sub(rows));
        self.top.set(top);
        let cursor = self.cursor(text);
        let caret_line = text[..cursor].matches('\n').count();
        let (motion, times) = if caret_line < top {
            (Motion::Down, top - caret_line)
        } else if caret_line >= top + rows {
            (Motion::Up, caret_line + 1 - (top + rows))
        } else {
            return;
        };
        let mut scratch = text.to_string();
        for _ in 0..times {
            self.run(&mut scratch, Op::Move(motion, false));
        }
    }

    /// Types `s` at the caret, replacing the selection if there is one.
    /// This is also the paste path.
    pub fn insert(&mut self, text: &mut String, s: &str) {
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
    pub fn replace_range(&mut self, text: &mut String, range: std::ops::Range<usize>, with: &str) {
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
    pub fn undo(&mut self, text: &mut String) -> bool {
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
    pub fn redo(&mut self, text: &mut String) -> bool {
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
    pub fn can_undo(&self, text: &str) -> bool {
        self.history.can_undo(text)
    }

    /// Whether [`Caret::redo`] would change `text`.
    pub fn can_redo(&self, text: &str) -> bool {
        self.history.can_redo(text)
    }

    /// Whether the editor would do something with `op` (`None`: the key
    /// decoded to nothing) - asked by callers whose key dispatch is a
    /// `match` with guards, so a key the editor claims never also reaches a
    /// pane shortcut. Copy and Cut are only claimed when there is a
    /// selection to copy.
    pub fn claims_op(&self, text: &str, op: Option<Op>) -> bool {
        match op {
            Some(Op::Copy | Op::Cut) => self.selection(text).is_some(),
            Some(_) => true,
            None => false,
        }
    }

    /// Applies one decoded editing operation to `text`. A frontend turns
    /// its key events into [`Op`]s (the terminal UI's `decode`) and calls
    /// this; the key map itself is described in the terminal UI's module
    /// docs.
    pub fn run(&mut self, text: &mut String, op: Op) -> KeyEdit {
        let kind = match op {
            Op::Undo | Op::Redo => {
                let done = if op == Op::Undo {
                    self.undo(text)
                } else {
                    self.redo(text)
                };
                return if done {
                    KeyEdit::Edited
                } else {
                    KeyEdit::Moved
                };
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
