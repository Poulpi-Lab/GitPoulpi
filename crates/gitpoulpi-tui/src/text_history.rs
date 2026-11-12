//! The edit history of one text field: what `Ctrl+Z` / `Ctrl+Y` walk.
//!
//! It lives inside the field's [`crate::text_edit::Caret`], so every field that
//! types through the shared editing core has it. Each entry is the whole
//! field as it was *before* an edit (text, caret, selection): an undo swaps
//! the current state for the entry, a redo swaps back.
//!
//! - **Grouping.** Typing is recorded per word: consecutive characters of the
//!   same class (word characters, whitespace, punctuation) typed within
//!   [`PAUSE`] of each other, with the caret where the last one left it, are
//!   one entry; so are consecutive Backspaces or Deletes. A paste, a cut, a
//!   kill-to-line-end, a snippet insert are one entry each, never merged.
//! - **Bounded.** At most [`MAX_ENTRIES`] entries and [`MAX_BYTES`] of saved
//!   text; the oldest go first.
//! - **Replaced from outside.** The history remembers the text it left the
//!   field with. When the field holds something else the next time it is
//!   asked (a path completed, a form reloaded), the history is dropped: an
//!   entry would restore a text the field no longer has.

use std::time::{Duration, Instant};

/// Typing pauses longer than this start a new entry.
pub(crate) const PAUSE: Duration = Duration::from_millis(1200);
/// Most entries kept per field.
pub(crate) const MAX_ENTRIES: usize = 200;
/// Most bytes of saved text kept per field.
pub(crate) const MAX_BYTES: usize = 1 << 20;

/// A field as it was: text, caret and the selection's other end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Snap {
    pub(crate) text: String,
    pub(crate) cursor: usize,
    pub(crate) anchor: Option<usize>,
}

/// What kind of edit an entry is, for merging the next one into it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// A typed character of this class (`text_edit::char_class`).
    Typed(u8),
    Backspace,
    Delete,
    /// Never merged: a paste, a cut, a kill, a snippet.
    Block,
}

#[derive(Debug, Clone)]
pub(crate) struct History {
    undo: Vec<Snap>,
    redo: Vec<Snap>,
    /// The text the last recorded edit (or undo / redo) left in the field.
    expected: Option<String>,
    /// The kind, caret and time of the last recorded edit.
    last: Option<(Kind, usize, Instant)>,
    pause: Duration,
}

impl Default for History {
    fn default() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            expected: None,
            last: None,
            pause: PAUSE,
        }
    }
}

impl History {
    /// Drops the history when the field no longer holds what it left.
    fn sync(&mut self, text: &str) {
        if self.expected.as_deref().is_some_and(|e| e != text) {
            self.clear();
        }
    }

    pub(crate) fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.expected = None;
        self.last = None;
    }

    /// Whether an undo has something to restore for a field holding `text`.
    pub(crate) fn can_undo(&self, text: &str) -> bool {
        !self.undo.is_empty() && self.expected.as_deref() == Some(text)
    }

    /// Whether a redo has something to restore for a field holding `text`.
    pub(crate) fn can_redo(&self, text: &str) -> bool {
        !self.redo.is_empty() && self.expected.as_deref() == Some(text)
    }

    /// Records an edit: `before` is the field as it was, `kind` what the
    /// edit was, `after` the text it left, `cursor` where the caret is now.
    pub(crate) fn record(&mut self, before: Snap, kind: Kind, after: &str, cursor: usize) {
        self.record_at(before, kind, after, cursor, Instant::now());
    }

    fn record_at(&mut self, before: Snap, kind: Kind, after: &str, cursor: usize, now: Instant) {
        // A field replaced from outside since the last edit: what the entries
        // hold is not this field any more.
        if self.expected.as_deref().is_some_and(|e| e != before.text) {
            self.clear();
        }
        let merge = kind != Kind::Block
            && !self.undo.is_empty()
            && self.last.is_some_and(|(k, at, when)| {
                k == kind && at == before.cursor && now.duration_since(when) <= self.pause
            });
        if !merge {
            self.undo.push(before);
        }
        self.redo.clear();
        self.expected = Some(after.to_string());
        self.last = Some((kind, cursor, now));
        self.trim();
    }

    fn trim(&mut self) {
        let mut bytes: usize = self.undo.iter().map(|s| s.text.len()).sum();
        while self.undo.len() > MAX_ENTRIES || (bytes > MAX_BYTES && self.undo.len() > 1) {
            bytes -= self.undo.remove(0).text.len();
        }
    }

    /// Steps back: `current` is the field now, the result the state to put
    /// back (`None`: nothing to undo).
    pub(crate) fn undo(&mut self, current: Snap) -> Option<Snap> {
        self.sync(&current.text);
        let back = self.undo.pop()?;
        self.expected = Some(back.text.clone());
        self.redo.push(current);
        self.last = None;
        Some(back)
    }

    /// Steps forward again.
    pub(crate) fn redo(&mut self, current: Snap) -> Option<Snap> {
        self.sync(&current.text);
        let next = self.redo.pop()?;
        self.expected = Some(next.text.clone());
        self.undo.push(current);
        self.last = None;
        Some(next)
    }

    #[cfg(test)]
    pub(crate) fn set_pause(&mut self, pause: Duration) {
        self.pause = pause;
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.undo.len()
    }
}

/// The history of a type-to-filter box that has no caret (the text only ever
/// grows at its end and loses its last character): the same grouping and
/// bounds, over a plain `String`.
#[derive(Debug, Clone, Default)]
pub(crate) struct QueryHistory(History);

impl QueryHistory {
    fn snap(text: &str) -> Snap {
        Snap {
            text: text.to_string(),
            cursor: text.len(),
            anchor: None,
        }
    }

    fn apply(&mut self, text: &mut String, kind: Kind, edit: impl FnOnce(&mut String)) {
        let before = Self::snap(text);
        edit(text);
        if before.text != *text {
            self.0.record(before, kind, text, text.len());
        }
    }

    /// Types `c` at the end.
    pub(crate) fn push(&mut self, text: &mut String, c: char) {
        let class = crate::text_edit::char_class(c);
        self.apply(text, Kind::Typed(class), |t| t.push(c));
    }

    /// Deletes the last character.
    pub(crate) fn pop(&mut self, text: &mut String) {
        self.apply(text, Kind::Backspace, |t| {
            t.pop();
        });
    }

    /// Appends `s` (a paste): one entry.
    pub(crate) fn paste(&mut self, text: &mut String, s: &str) {
        self.apply(text, Kind::Block, |t| t.push_str(s));
    }

    /// Empties the box: one entry.
    pub(crate) fn wipe(&mut self, text: &mut String) {
        self.apply(text, Kind::Block, String::clear);
    }

    /// `Ctrl+Z` / `Ctrl+Y` / `Ctrl+Shift+Z`: whether `key` was one (the text
    /// is changed when there was something to step to).
    pub(crate) fn key(&mut self, text: &mut String, key: &crossterm::event::KeyEvent) -> bool {
        use crossterm::event::{KeyCode, KeyModifiers};
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let undo = match key.code {
            KeyCode::Char('\u{1a}') => true,
            KeyCode::Char('\u{19}') => false,
            KeyCode::Char(c) if ctrl => match c.to_ascii_lowercase() {
                'z' if shift => false,
                'z' => true,
                'y' => false,
                _ => return false,
            },
            _ => return false,
        };
        let now = Self::snap(text);
        let step = if undo {
            self.0.undo(now)
        } else {
            self.0.redo(now)
        };
        if let Some(s) = step {
            *text = s.text;
        }
        true
    }
}
