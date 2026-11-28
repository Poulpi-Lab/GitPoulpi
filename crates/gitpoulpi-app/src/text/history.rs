//! The edit history of one text field: what `Ctrl+Z` / `Ctrl+Y` walk.
//!
//! It lives inside the field's [`super::Caret`], so every field that
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
pub const PAUSE: Duration = Duration::from_millis(1200);
/// Most entries kept per field.
pub const MAX_ENTRIES: usize = 200;
/// Most bytes of saved text kept per field.
pub const MAX_BYTES: usize = 1 << 20;

/// A field as it was: text, caret and the selection's other end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snap {
    pub text: String,
    pub cursor: usize,
    pub anchor: Option<usize>,
}

/// What kind of edit an entry is, for merging the next one into it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A typed character of this class (`super::char_class`).
    Typed(u8),
    Backspace,
    Delete,
    /// Never merged: a paste, a cut, a kill, a snippet.
    Block,
}

#[derive(Debug, Clone)]
pub struct History {
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

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.expected = None;
        self.last = None;
    }

    /// Whether an undo has something to restore for a field holding `text`.
    pub fn can_undo(&self, text: &str) -> bool {
        !self.undo.is_empty() && self.expected.as_deref() == Some(text)
    }

    /// Whether a redo has something to restore for a field holding `text`.
    pub fn can_redo(&self, text: &str) -> bool {
        !self.redo.is_empty() && self.expected.as_deref() == Some(text)
    }

    /// Records an edit: `before` is the field as it was, `kind` what the
    /// edit was, `after` the text it left, `cursor` where the caret is now.
    pub fn record(&mut self, before: Snap, kind: Kind, after: &str, cursor: usize) {
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
    pub fn undo(&mut self, current: Snap) -> Option<Snap> {
        self.sync(&current.text);
        let back = self.undo.pop()?;
        self.expected = Some(back.text.clone());
        self.redo.push(current);
        self.last = None;
        Some(back)
    }

    /// Steps forward again.
    pub fn redo(&mut self, current: Snap) -> Option<Snap> {
        self.sync(&current.text);
        let next = self.redo.pop()?;
        self.expected = Some(next.text.clone());
        self.undo.push(current);
        self.last = None;
        Some(next)
    }

    /// Overrides the typing pause that merges edits (a test hook: tests
    /// should not sleep).
    pub fn set_pause(&mut self, pause: Duration) {
        self.pause = pause;
    }

    /// How many undo entries are kept.
    pub fn len(&self) -> usize {
        self.undo.len()
    }

    /// No undo entry is kept.
    pub fn is_empty(&self) -> bool {
        self.undo.is_empty()
    }
}
