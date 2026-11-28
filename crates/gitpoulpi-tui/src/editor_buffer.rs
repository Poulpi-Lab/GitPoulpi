//! The built-in editor's document. The model (text, caret, undo, byte-exact
//! round trip, formats) is `gitpoulpi_app::text::doc`, shared with the other
//! frontend; this file adds the terminal key decoding.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::text_edit::{decode, KeyEdit};

#[allow(unused_imports)]
pub(crate) use gitpoulpi_app::text::doc::*;

/// The terminal key half of [`Doc`]: a `KeyEvent` is decoded into a
/// [`DocKey`] and handed to [`Doc::apply_key`].
pub(crate) trait DocKeys {
    /// Applies one editing key. `page` is how many lines PageUp/PageDown
    /// move. Undo, redo, save and the other chords are the editor's
    /// (`editor.rs`), not the document's.
    fn on_key(&mut self, key: &KeyEvent, page: usize) -> KeyEdit;
}

impl DocKeys for Doc {
    fn on_key(&mut self, key: &KeyEvent, page: usize) -> KeyEdit {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let page = page.max(1) as isize;
        let doc_key = match key.code {
            KeyCode::Up if !ctrl && !alt => DocKey::Lines(-1),
            KeyCode::Down if !ctrl && !alt => DocKey::Lines(1),
            KeyCode::PageUp => DocKey::Lines(-page),
            KeyCode::PageDown => DocKey::Lines(page),
            KeyCode::Enter if !ctrl && !alt => DocKey::Enter,
            KeyCode::Tab if !ctrl && !alt => DocKey::Tab,
            KeyCode::BackTab => DocKey::BackTab,
            KeyCode::Home if !ctrl => DocKey::SmartHome,
            code => DocKey::Caret {
                op: decode(key, true),
                group: match code {
                    KeyCode::Char(_) if !ctrl || alt => Group::Typing,
                    KeyCode::Backspace | KeyCode::Delete => Group::Deleting,
                    _ => Group::Alone,
                },
            },
        };
        self.apply_key(doc_key, shift)
    }
}
