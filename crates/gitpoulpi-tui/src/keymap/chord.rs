//! `KeyChord`: one key plus modifiers, parsed from and printed to the
//! text form used by config files and labels (`Ctrl+Shift+N`, `F2`,
//! `PageDown`), and built from a crossterm event with the terminal's
//! spelling quirks normalized away. Pure: no event-loop state.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::fmt;

/// Modifier set. Only the three that terminals report reliably.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) struct Mods {
    pub(crate) ctrl: bool,
    pub(crate) alt: bool,
    pub(crate) shift: bool,
}

impl Mods {
    pub(crate) const NONE: Mods = Mods {
        ctrl: false,
        alt: false,
        shift: false,
    };
    pub(crate) const CTRL: Mods = Mods {
        ctrl: true,
        alt: false,
        shift: false,
    };

    fn from_crossterm(m: KeyModifiers) -> Mods {
        Mods {
            ctrl: m.contains(KeyModifiers::CONTROL),
            alt: m.contains(KeyModifiers::ALT),
            shift: m.contains(KeyModifiers::SHIFT),
        }
    }

    /// Whether every modifier of `self` is also set in `other`.
    fn subset_of(self, other: Mods) -> bool {
        (!self.ctrl || other.ctrl) && (!self.alt || other.alt) && (!self.shift || other.shift)
    }
}

/// The key of a chord. Letters are stored lowercase; an uppercase
/// letter is `Char(lower)` with `mods.shift` set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Key {
    Char(char),
    Enter,
    Esc,
    Tab,
    BackTab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    F(u8),
    Space,
    Menu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct KeyChord {
    pub(crate) mods: Mods,
    pub(crate) key: Key,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ChordError {
    Empty,
    UnknownKey(String),
    /// `Ctrl+Alt+<non-letter>` is how Windows delivers AltGr characters.
    AltGr,
}

impl fmt::Display for ChordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChordError::Empty => write!(f, "empty key"),
            ChordError::UnknownKey(k) => write!(f, "unknown key '{k}'"),
            ChordError::AltGr => write!(f, "AltGr on Windows types this character"),
        }
    }
}

fn named_key(name: &str) -> Option<Key> {
    let lower = name.to_ascii_lowercase();
    Some(match lower.as_str() {
        "enter" | "return" => Key::Enter,
        "esc" | "escape" => Key::Esc,
        "tab" => Key::Tab,
        "backtab" => Key::BackTab,
        "backspace" => Key::Backspace,
        "delete" | "del" => Key::Delete,
        "insert" | "ins" => Key::Insert,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" | "pgup" => Key::PageUp,
        "pagedown" | "pgdn" | "pgdown" => Key::PageDown,
        "up" => Key::Up,
        "down" => Key::Down,
        "left" => Key::Left,
        "right" => Key::Right,
        "space" => Key::Space,
        "menu" => Key::Menu,
        _ => {
            let n = lower.strip_prefix('f')?.parse::<u8>().ok()?;
            if (1..=24).contains(&n) {
                Key::F(n)
            } else {
                return None;
            }
        }
    })
}

impl KeyChord {
    pub(crate) const fn new(mods: Mods, key: Key) -> Self {
        KeyChord { mods, key }
    }

    /// `Ctrl+Shift+N`, `F2`, `+`, `Ctrl+,`, `x`, `X` (= `Shift+x`).
    pub(crate) fn parse(text: &str) -> Result<KeyChord, ChordError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(ChordError::Empty);
        }
        let mut mods = Mods::NONE;
        let mut rest = text;
        // Peel `Name+` modifier prefixes; `Ctrl++` leaves the key `+`.
        while let Some((head, tail)) = rest.split_once('+') {
            if tail.is_empty() {
                break;
            }
            match head.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => mods.ctrl = true,
                "alt" => mods.alt = true,
                "shift" => mods.shift = true,
                _ => break,
            }
            rest = tail;
        }
        let key = if let Some(k) = named_key(rest) {
            k
        } else {
            let mut chars = rest.chars();
            match (chars.next(), chars.next()) {
                (Some(' '), None) => Key::Space,
                (Some(c), None) => {
                    // Under Ctrl/Alt the case is just spelling (`Ctrl+O`);
                    // only the explicit `Shift+` counts.
                    if c.is_uppercase() && !mods.ctrl && !mods.alt {
                        mods.shift = true;
                    }
                    Key::Char(c.to_lowercase().next().unwrap_or(c))
                }
                _ => return Err(ChordError::UnknownKey(rest.to_string())),
            }
        };
        if matches!(key, Key::Char(c) if !c.is_alphabetic()) {
            // Shift is implicit in a symbol's own glyph.
            if mods.shift && !mods.ctrl && !mods.alt {
                mods.shift = false;
            }
            if mods.ctrl && mods.alt {
                return Err(ChordError::AltGr);
            }
        }
        Ok(KeyChord { mods, key })
    }

    /// The chord a bare key code spells with no modifier information: an
    /// uppercase letter is `Shift+<lower>`. Used by the prefix tables, which
    /// look at `key.code` alone.
    pub(crate) fn from_code(code: KeyCode) -> Option<KeyChord> {
        Self::from_parts(code, KeyModifiers::NONE)
    }

    /// The chord a crossterm event spells, normalized. `None` for a release
    /// event or a key with no chord form.
    pub(crate) fn from_event(ev: &KeyEvent) -> Option<KeyChord> {
        if ev.kind == KeyEventKind::Release {
            return None;
        }
        Self::from_parts(ev.code, ev.modifiers)
    }

    pub(super) fn from_parts(code: KeyCode, modifiers: KeyModifiers) -> Option<KeyChord> {
        let mut mods = Mods::from_crossterm(modifiers);
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
        Some(KeyChord { mods, key }.class())
    }

    /// The spelling class this chord belongs to: the canonical
    /// member of the set of chords one terminal byte sequence can mean.
    /// `Ctrl+J` and `Ctrl+Enter` are one class; so are `Ctrl+@` and
    /// `Ctrl+Space`, and `Ctrl+_`, `Ctrl+7` and `Ctrl+/`.
    pub(crate) fn class(self) -> KeyChord {
        let plain_ctrl = self.mods.ctrl && !self.mods.alt && !self.mods.shift;
        match self.key {
            Key::Char('j') if plain_ctrl => KeyChord::new(Mods::CTRL, Key::Enter),
            Key::Char('@') if plain_ctrl => KeyChord::new(Mods::CTRL, Key::Space),
            Key::Char('_' | '7') if plain_ctrl => KeyChord::new(Mods::CTRL, Key::Char('/')),
            _ => self,
        }
    }

    /// Whether `ev` triggers this binding. Deliberately lenient: the
    /// binding's modifiers must all be present, extra ones are tolerated
    /// (`Ctrl+O` still fires for `Ctrl+Shift+O`), and a letter under
    /// Ctrl/Alt ignores case. Callers with a Shift twin (`Ctrl+Shift+N` over
    /// `Ctrl+N`) keep the twin's arm first.
    pub(crate) fn matches(&self, ev: &KeyEvent) -> bool {
        if ev.kind != KeyEventKind::Press {
            return false;
        }
        self.matches_parts(ev.code, ev.modifiers)
    }

    pub(crate) fn matches_parts(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        let Some(actual) = Self::from_parts(code, modifiers) else {
            return false;
        };
        let want = self.class();
        if want.key != actual.key {
            return false;
        }
        let letter = matches!(want.key, Key::Char(c) if c.is_alphabetic());
        if letter && !want.mods.ctrl && !want.mods.alt {
            // A bare letter is case-sensitive (`p` is not `P`) and ignores
            // other modifiers.
            return want.mods.shift == actual.mods.shift;
        }
        want.mods.subset_of(actual.mods)
    }

    /// Exact (class-wise) equality with the event's own chord.
    pub(crate) fn matches_exact(&self, ev: &KeyEvent) -> bool {
        Self::from_event(ev).is_some_and(|c| c == self.class())
    }
}

impl KeyChord {
    /// The compact spelling used on screen: a shifted bare letter is its
    /// capital (`P`); every other chord prints as [`fmt::Display`] does
    /// (`Ctrl+Shift+N`).
    pub(crate) fn short(&self) -> String {
        match self.key {
            Key::Char(c) if self.mods.shift && !self.mods.ctrl && !self.mods.alt => {
                c.to_uppercase().collect()
            }
            _ => self.to_string(),
        }
    }
}

impl fmt::Display for KeyChord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.mods.ctrl {
            write!(f, "Ctrl+")?;
        }
        if self.mods.alt {
            write!(f, "Alt+")?;
        }
        if self.mods.shift {
            write!(f, "Shift+")?;
        }
        match self.key {
            Key::Char(c) => {
                if self.mods.ctrl || self.mods.alt || self.mods.shift {
                    write!(f, "{}", c.to_uppercase())
                } else {
                    write!(f, "{c}")
                }
            }
            Key::Enter => write!(f, "Enter"),
            Key::Esc => write!(f, "Esc"),
            Key::Tab => write!(f, "Tab"),
            Key::BackTab => write!(f, "BackTab"),
            Key::Backspace => write!(f, "Backspace"),
            Key::Delete => write!(f, "Delete"),
            Key::Insert => write!(f, "Insert"),
            Key::Home => write!(f, "Home"),
            Key::End => write!(f, "End"),
            Key::PageUp => write!(f, "PageUp"),
            Key::PageDown => write!(f, "PageDown"),
            Key::Up => write!(f, "Up"),
            Key::Down => write!(f, "Down"),
            Key::Left => write!(f, "Left"),
            Key::Right => write!(f, "Right"),
            Key::F(n) => write!(f, "F{n}"),
            Key::Space => write!(f, "Space"),
            Key::Menu => write!(f, "Menu"),
        }
    }
}

/// Whether a chord can reach the application on a terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reach {
    Ok,
    /// Arrives as the same bytes as this other chord.
    NotDistinct(KeyChord),
    /// The terminal never sends it (it arrives as the bare character, or
    /// not at all).
    Unsendable,
}

impl KeyChord {
    /// Pure table: can this chord arrive? `windows` = the console API
    /// (modifiers intact); `enhanced` = the kitty keyboard protocol is on.
    pub(crate) fn reach(self, windows: bool, enhanced: bool) -> Reach {
        if windows || enhanced {
            return Reach::Ok;
        }
        let m = self.mods;
        let bare = |key| KeyChord::new(Mods::NONE, key);
        if m.ctrl && !m.alt {
            return match self.key {
                Key::Char(c) if c.is_alphabetic() => {
                    if m.shift {
                        return Reach::NotDistinct(KeyChord::new(Mods::CTRL, self.key));
                    }
                    match c {
                        'i' => Reach::NotDistinct(bare(Key::Tab)),
                        'm' => Reach::NotDistinct(bare(Key::Enter)),
                        'h' => Reach::NotDistinct(bare(Key::Backspace)),
                        _ => Reach::Ok,
                    }
                }
                Key::Char('[') => Reach::NotDistinct(bare(Key::Esc)),
                Key::Char('/' | ']' | '\\' | '@' | '_') | Key::Space if !m.shift => Reach::Ok,
                Key::Char(_) => Reach::Unsendable,
                Key::Enter if !m.shift => Reach::Ok,
                Key::Enter => Reach::Unsendable,
                Key::Backspace => Reach::NotDistinct(bare(Key::Backspace)),
                Key::Tab | Key::BackTab => Reach::Unsendable,
                _ => Reach::Ok,
            };
        }
        if m.shift && !m.ctrl && !m.alt && self.key == Key::Enter {
            return Reach::NotDistinct(bare(Key::Enter));
        }
        Reach::Ok
    }
}

/// Tools that commonly take a chord before the application sees it, as
/// an informational hint (never blocking).
pub(crate) fn commonly_taken_by(chord: KeyChord) -> Option<gitpoulpi_i18n::M> {
    use gitpoulpi_i18n::msg::keymap_conflict as msg;
    let c = chord.class();
    let text = c.to_string();
    Some(match text.as_str() {
        "Ctrl+B" => msg::TAKEN_TMUX,
        "Ctrl+A" => msg::TAKEN_SCREEN,
        "Ctrl+G" | "Ctrl+P" | "Ctrl+N" | "Ctrl+T" | "Ctrl+O" | "Ctrl+S" | "Ctrl+H" | "Ctrl+Q" => {
            msg::TAKEN_ZELLIJ
        }
        "Ctrl+," | "Ctrl+Shift+T" | "Ctrl+Shift+W" | "Ctrl+Shift+N" | "Ctrl+Shift+P"
        | "Ctrl+Shift+C" | "Ctrl+Shift+V" | "Alt+Enter" | "Ctrl+Tab" => msg::TAKEN_WINDOWS_TERMINAL,
        _ => return None,
    })
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
