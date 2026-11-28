//! The terminal adapter of the keymap: crossterm key events in, neutral
//! [`KeyChord`]s out. Every terminal spelling quirk lives here (`\n` read as
//! Ctrl+Enter, Ctrl plus a letter arriving as a control character, AltGr, the
//! two spellings of Shift+Tab); `chord.rs`, `mod.rs` and `conflict.rs` know
//! none of it, so a desktop frontend can feed them chords directly.

use super::chord::{Key, KeyChord, Mods};
use super::conflict::KeyTransport;
use super::{Action, Ctx, Keymap};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

fn mods_of(m: KeyModifiers) -> Mods {
    Mods {
        ctrl: m.contains(KeyModifiers::CONTROL),
        alt: m.contains(KeyModifiers::ALT),
        shift: m.contains(KeyModifiers::SHIFT),
    }
}

/// The chord a crossterm key code with modifiers spells, normalized. `None`
/// for a key with no chord form. An uppercase letter is `Shift+<lower>`.
pub(crate) fn chord_of_parts(code: KeyCode, modifiers: KeyModifiers) -> Option<KeyChord> {
    let mut mods = mods_of(modifiers);
    let key = match code {
        KeyCode::Char('\n') => {
            return Some(KeyChord::new(Mods::CTRL, Key::Enter));
        }
        KeyCode::Char('\r') => Key::Enter,
        KeyCode::Char('\t') => Key::Tab,
        // Raw control characters: what some terminals hand over
        // instead of Ctrl+<letter>.
        KeyCode::Char(c @ '\u{1}'..='\u{1a}') => {
            mods.ctrl = true;
            Key::Char((b'a' + (c as u8 - 1)) as char)
        }
        KeyCode::Char(' ') => Key::Space,
        KeyCode::Char(c) => {
            if c.is_uppercase() {
                // Under Ctrl/Alt only the Shift bit says Shift.
                if !mods.ctrl && !mods.alt {
                    mods.shift = true;
                }
            } else if !c.is_alphabetic() {
                mods.shift = false;
            }
            Key::Char(c.to_lowercase().next().unwrap_or(c))
        }
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Esc,
        KeyCode::Tab => Key::Tab,
        KeyCode::BackTab => {
            mods.shift = true;
            Key::BackTab
        }
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Delete => Key::Delete,
        KeyCode::Insert => Key::Insert,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::F(n) => Key::F(n),
        KeyCode::Menu => Key::Menu,
        _ => return None,
    };
    Some(KeyChord::new(mods, key).class())
}

/// The chord a crossterm event spells, normalized. `None` for a release
/// event or a key with no chord form.
pub(crate) fn chord_of(ev: &KeyEvent) -> Option<KeyChord> {
    if ev.kind == KeyEventKind::Release {
        return None;
    }
    chord_of_parts(ev.code, ev.modifiers)
}

/// [`KeyChord`] built from and matched against crossterm input.
pub(crate) trait TermChord: Sized {
    /// The chord a bare key code spells with no modifier information: an
    /// uppercase letter is `Shift+<lower>`. Used by the prefix tables, which
    /// look at `key.code` alone.
    fn from_code(code: KeyCode) -> Option<KeyChord>;
    fn from_event(ev: &KeyEvent) -> Option<KeyChord>;
    /// Whether `ev` triggers this binding ([`KeyChord::matches_chord`] on
    /// the event's chord); only a key press does.
    fn matches(&self, ev: &KeyEvent) -> bool;
    fn matches_parts(&self, code: KeyCode, modifiers: KeyModifiers) -> bool;
    /// Exact (class-wise) equality with the event's own chord.
    fn matches_exact(&self, ev: &KeyEvent) -> bool;
}

impl TermChord for KeyChord {
    fn from_code(code: KeyCode) -> Option<KeyChord> {
        chord_of_parts(code, KeyModifiers::NONE)
    }

    fn from_event(ev: &KeyEvent) -> Option<KeyChord> {
        chord_of(ev)
    }

    fn matches(&self, ev: &KeyEvent) -> bool {
        if ev.kind != KeyEventKind::Press {
            return false;
        }
        self.matches_parts(ev.code, ev.modifiers)
    }

    fn matches_parts(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        chord_of_parts(code, modifiers).is_some_and(|actual| self.matches_chord(actual))
    }

    fn matches_exact(&self, ev: &KeyEvent) -> bool {
        chord_of(ev).is_some_and(|c| c == self.class())
    }
}

/// [`Keymap`] lookups by crossterm event.
pub(crate) trait TermKeymap {
    /// Whether `ev` triggers `action` (any of its chords).
    fn is(&self, action: Action, ev: &KeyEvent) -> bool;
    /// The action `ev` triggers in `ctx`, if any (see [`Keymap::action_for`]).
    fn action_in(&self, ctx: Ctx, ev: &KeyEvent) -> Option<Action>;
    /// [`TermKeymap::action_in`] for a bare key code, the form the prefix
    /// tables (and `chrome::navigation_key_action`) are called with.
    fn action_in_code(&self, ctx: Ctx, code: KeyCode) -> Option<Action>;
    /// [`TermKeymap::action_in_code`] with the modifiers kept (`Shift+Left`
    /// after the navigation prefix).
    fn action_in_code_mods(
        &self,
        ctx: Ctx,
        code: KeyCode,
        modifiers: KeyModifiers,
    ) -> Option<Action>;
}

impl TermKeymap for Keymap {
    fn is(&self, action: Action, ev: &KeyEvent) -> bool {
        if ev.kind != KeyEventKind::Press {
            return false;
        }
        chord_of_parts(ev.code, ev.modifiers).is_some_and(|c| self.chord_is(action, c))
    }

    fn action_in(&self, ctx: Ctx, ev: &KeyEvent) -> Option<Action> {
        if ctx.ignores_modifiers() {
            // A prefix table looks at the key code, but a Ctrl chord bound
            // after the prefix matches exactly first (see `action_for`).
            if ev.modifiers.contains(KeyModifiers::CONTROL) {
                let exact = self.action_in_code_mods(ctx, ev.code, KeyModifiers::CONTROL);
                if exact.is_some() {
                    return exact;
                }
            }
            return self.action_in_code(ctx, ev.code);
        }
        let chord = chord_of(ev)?;
        if ev.kind == KeyEventKind::Press {
            self.action_for(ctx, chord)
        } else {
            // A repeat only fires an exact binding.
            self.action_exact(ctx, chord)
        }
    }

    fn action_in_code(&self, ctx: Ctx, code: KeyCode) -> Option<Action> {
        self.action_in_code_mods(ctx, code, KeyModifiers::NONE)
    }

    fn action_in_code_mods(
        &self,
        ctx: Ctx,
        code: KeyCode,
        modifiers: KeyModifiers,
    ) -> Option<Action> {
        self.action_exact(ctx, chord_of_parts(code, modifiers)?)
    }
}

/// What the running terminal can send: the console API on Windows, and
/// whether the kitty keyboard protocol is on.
pub(crate) fn current_transport() -> KeyTransport {
    KeyTransport::Terminal {
        windows: cfg!(windows),
        enhanced: crate::kitty_keyboard::is_active(),
    }
}

/// The one place AltGr is undone: every terminal event goes through here
/// right after it is read, before keymap resolution and before any handler
/// or text field sees it. No configured chord can be `Ctrl+Alt+<non-letter>`
/// (`ChordError::AltGr` rejects it), so there is no binding for this to
/// shadow; `Ctrl+Alt+<letter or digit>` chords are left alone.
pub(crate) fn normalize_event(event: crossterm::event::Event) -> crossterm::event::Event {
    match event {
        crossterm::event::Event::Key(key) => crossterm::event::Event::Key(drop_altgr(key)),
        other => other,
    }
}

/// A key typed with AltGr, as the plain character it types.
///
/// A Windows console reports AltGr as `CONTROL | ALT`, and on the Swiss and
/// German layouts `[` `]` `{` `}` `\` `@` need it. A column's own keys
/// (`[` `]` `0`) would otherwise never reach a keyboard that types `[` that
/// way. A letter or digit stays a chord: only a real `Ctrl+Alt` makes one.
pub(crate) fn drop_altgr(mut key: KeyEvent) -> KeyEvent {
    use crossterm::event::KeyModifiers;
    let altgr = KeyModifiers::CONTROL | KeyModifiers::ALT;
    if key.modifiers.contains(altgr) {
        if let KeyCode::Char(c) = key.code {
            if !c.is_ascii_alphanumeric() && !c.is_control() {
                key.modifiers.remove(altgr);
            }
        }
    }
    key
}

/// The Ctrl+Enter spelling class on a raw event: `Ctrl+Enter`, `Ctrl+J`
/// (what most Unix terminals send for it) or a bare `'\n'` character.
pub(crate) fn is_ctrl_enter_class(ev: &KeyEvent) -> bool {
    match ev.code {
        KeyCode::Enter => ev.modifiers.contains(KeyModifiers::CONTROL),
        KeyCode::Char('j') | KeyCode::Char('J') => {
            ev.modifiers.contains(KeyModifiers::CONTROL)
                && !ev.modifiers.contains(KeyModifiers::ALT)
        }
        KeyCode::Char('\n') => true,
        _ => false,
    }
}

#[cfg(test)]
mod altgr_tests {
    use super::drop_altgr;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn altgr_brackets_are_plain_but_ctrl_alt_letters_stay_chords() {
        let altgr = KeyModifiers::CONTROL | KeyModifiers::ALT;
        let k = drop_altgr(KeyEvent::new(KeyCode::Char(']'), altgr));
        assert_eq!(k.modifiers, KeyModifiers::NONE);
        let k = drop_altgr(KeyEvent::new(KeyCode::Char('u'), altgr));
        assert_eq!(k.modifiers, altgr);
        let k = drop_altgr(KeyEvent::new(KeyCode::Char(']'), KeyModifiers::ALT));
        assert_eq!(k.modifiers, KeyModifiers::ALT);
    }

    #[test]
    fn normalize_event_only_touches_keys() {
        use crossterm::event::Event;
        let altgr = KeyModifiers::CONTROL | KeyModifiers::ALT;
        for c in ['@', '{', '\\', '[', '~', '|'] {
            let Event::Key(k) =
                super::normalize_event(Event::Key(KeyEvent::new(KeyCode::Char(c), altgr)))
            else {
                panic!("key stays a key")
            };
            assert_eq!(
                (k.code, k.modifiers),
                (KeyCode::Char(c), KeyModifiers::NONE)
            );
        }
        let Event::Key(k) =
            super::normalize_event(Event::Key(KeyEvent::new(KeyCode::Char('o'), altgr)))
        else {
            panic!()
        };
        assert_eq!(k.modifiers, altgr);
        assert!(matches!(
            super::normalize_event(Event::Paste("x".into())),
            Event::Paste(_)
        ));
    }

    #[test]
    fn no_chord_can_be_ctrl_alt_punctuation_so_none_is_shadowed() {
        assert!(super::KeyChord::parse("Ctrl+Alt+[").is_err());
        assert!(super::KeyChord::parse("Ctrl+Alt+@").is_err());
        assert!(super::KeyChord::parse("Ctrl+Alt+O").is_ok());
    }
}
