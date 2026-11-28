//! A tiny stand-in for a frontend's key events, so the document's tests can
//! keep reading as the keys a person presses.

use crate::text::{KeyEdit, Motion, Op};

use super::{Doc, DocKey, Group};

#[derive(Debug, Clone, Copy)]
pub(super) enum KeyCode {
    Char(char),
    Enter,
    Tab,
    BackTab,
    End,
    Down,
    Backspace,
}

pub(super) struct KeyModifiers;

impl KeyModifiers {
    pub(super) const NONE: KeyModifiers = KeyModifiers;
    pub(super) const SHIFT: KeyModifiers = KeyModifiers;
}

pub(super) struct KeyEvent(KeyCode);

impl KeyEvent {
    pub(super) fn new(code: KeyCode, _mods: KeyModifiers) -> KeyEvent {
        KeyEvent(code)
    }
}

pub(super) trait OnKey {
    fn on_key(&mut self, key: &KeyEvent, page: usize) -> KeyEdit;
}

impl OnKey for Doc {
    fn on_key(&mut self, key: &KeyEvent, _page: usize) -> KeyEdit {
        let caret = |op, group| DocKey::Caret {
            op: Some(op),
            group,
        };
        let key = match key.0 {
            KeyCode::Char(c) => caret(Op::Insert(c), Group::Typing),
            KeyCode::Enter => DocKey::Enter,
            KeyCode::Tab => DocKey::Tab,
            KeyCode::BackTab => DocKey::BackTab,
            KeyCode::End => caret(Op::Move(Motion::LineEnd, false), Group::Alone),
            KeyCode::Down => DocKey::Lines(1),
            KeyCode::Backspace => caret(Op::Backspace, Group::Deleting),
        };
        self.apply_key(key, false)
    }
}
