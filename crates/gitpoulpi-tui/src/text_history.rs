//! The type-to-filter box's edit history over a plain `String`. The history
//! itself (grouping, bounds, undo and redo) is `gitpoulpi_app::text::history`,
//! shared with the desktop frontend; this file keeps the part that reads
//! terminal key events.

#[allow(unused_imports)]
pub(crate) use gitpoulpi_app::text::history::{History, Kind, Snap, MAX_BYTES, MAX_ENTRIES, PAUSE};

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
        let class = gitpoulpi_app::text::char_class(c);
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
