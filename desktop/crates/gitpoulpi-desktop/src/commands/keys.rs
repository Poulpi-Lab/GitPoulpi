//! Key bindings: GPUI keystrokes become `KeyChord`s, are resolved with the shared keymap in the key
//! context of the focused pane (then `global`), and run through the one dispatcher.
//!
//! The keymap is the terminal UI's: `[keybindings]` of `config.toml` overrides it the same way. One
//! interceptor sees every keystroke before GPUI's own bindings:
//!
//! - while a dialog, a menu or a picker is open it stays out of the way (the popup owns the keyboard);
//! - in a text field it only takes chords that cannot be typing (Ctrl / Alt / function keys), and
//!   leaves the field's own editing keys alone;
//! - a command that the desktop does not implement yet answers "not available yet" when its chord
//!   carries Ctrl or Alt (or is a function key); a bare key bound to such a command is left to the
//!   panel under the pointer, which handles its own navigation keys.

use gitpoulpi_app::keymap::chord::{Key, KeyChord, Mods};
use gitpoulpi_app::keymap::conflict::{install_transport, KeyTransport};
use gitpoulpi_app::keymap::{self, Action, Ctx, Keymap, KeymapIssue};
use gpui_kit::component::WindowExt as _;
use gpui_kit::{App, Global, Keystroke, KeystrokeEvent, Window};
use std::path::Path;

use super::catalog::Command;
use super::context::window_state;
use super::exec;
use super::ops::op_for;
use super::picker;
use super::status;

/// The chord a keystroke spells, normalized like the terminal's. `None` for a key with no chord form
/// (a bare modifier, the Super key off macOS, an AltGr character).
pub fn chord_of(ks: &Keystroke) -> Option<KeyChord> {
    let m = &ks.modifiers;
    if m.platform && !cfg!(target_os = "macos") {
        return None;
    }
    let mut mods = Mods {
        ctrl: m.control || (cfg!(target_os = "macos") && m.platform),
        alt: m.alt,
        shift: m.shift,
    };
    let typed = !mods.ctrl && !mods.alt;
    let key = match ks.key.as_str() {
        "enter" => Key::Enter,
        "escape" => Key::Esc,
        "tab" => Key::Tab,
        "backspace" => Key::Backspace,
        "delete" => Key::Delete,
        "insert" => Key::Insert,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        "up" => Key::Up,
        "down" => Key::Down,
        "left" => Key::Left,
        "right" => Key::Right,
        "space" => Key::Space,
        "menu" => Key::Menu,
        name => {
            if let Some(n) = name
                .strip_prefix('f')
                .and_then(|n| n.parse::<u8>().ok())
                .filter(|n| (1..=24).contains(n))
            {
                Key::F(n)
            } else {
                // What the key types wins when the user is typing (`?` is Shift+/), else the key itself.
                let text = if typed { ks.key_char.as_deref() } else { None };
                let mut chars = text.unwrap_or(name).chars();
                let (Some(c), None) = (chars.next(), chars.next()) else {
                    return None;
                };
                if c.is_alphabetic() {
                    if c.is_uppercase() {
                        mods.shift = true;
                    }
                    Key::Char(c.to_lowercase().next().unwrap_or(c))
                } else {
                    // A symbol carries its own shift; Ctrl+Alt+symbol is how AltGr arrives.
                    mods.shift = false;
                    if mods.ctrl && mods.alt {
                        return None;
                    }
                    Key::Char(c)
                }
            }
        }
    };
    Some(KeyChord::new(mods, key).class())
}

/// A chord that can only be a command, never text: Ctrl or Alt held, or a function key.
fn is_command_chord(chord: KeyChord) -> bool {
    chord.mods.ctrl || chord.mods.alt || matches!(chord.key, Key::F(_))
}

/// The editing keys a text field handles itself: a command bound to one is left to the field.
fn is_field_editing(chord: KeyChord) -> bool {
    match chord.key {
        Key::Char(c) if chord.mods.ctrl && !chord.mods.alt => "acvxzy".contains(c),
        Key::Char(_) | Key::Space => false,
        Key::F(_) => false,
        _ => true,
    }
}

/// The first key of a two-key chord was pressed: the next key is looked up in this context.
#[derive(Default)]
struct PendingPrefix(Option<Ctx>);
impl Global for PendingPrefix {}

/// Starts a two-key chord in `ctx` (`Ctx::GitPrefix` or `Ctx::NavPrefix`).
pub fn begin_prefix(ctx: Ctx, cx: &mut App) {
    cx.set_global(PendingPrefix(Some(ctx)));
    let km = keymap::current();
    let text = match ctx {
        Ctx::GitPrefix => gitpoulpi_i18n::msg::footer::hint_git(km.shown(Action::GlobalGitPrefix)),
        _ => gitpoulpi_i18n::msg::footer::hint_nav(km.shown(Action::GlobalNavPrefix)),
    };
    status::set(cx, text);
}

fn take_prefix(cx: &mut App) -> Option<Ctx> {
    let pending = cx.try_global::<PendingPrefix>().and_then(|p| p.0)?;
    cx.set_global(PendingPrefix(None));
    Some(pending)
}

/// The action the second key of a prefix chord triggers. A prefix table looks at the key, not at the
/// modifiers: `Shift+H` (an uppercase letter) and `Shift+Left` are bound as such, a plain key is
/// looked up bare, and a Ctrl chord bound after the prefix matches exactly first.
pub fn prefix_action(km: &Keymap, prefix: Ctx, chord: KeyChord) -> Option<Action> {
    let shifted = KeyChord::new(
        Mods {
            ctrl: false,
            alt: false,
            shift: chord.mods.shift,
        },
        chord.key,
    );
    [chord, shifted, chord.bare()]
        .into_iter()
        .find_map(|c| km.action_exact(prefix, c))
}

/// What a keystroke resolves to.
#[derive(Debug, PartialEq)]
pub enum Resolved {
    /// A command to run.
    Command(Action),
    /// Nothing is bound.
    Nothing,
}

/// The action `chord` triggers, the pane's own context before `global`. A chord with Ctrl or Alt held
/// needs an exact binding (the keymap's lenient matching would take `Alt+F` for the bare `f`, and
/// `Ctrl+N` for the Log's `n`), except that an extra Shift is tolerated (`Ctrl+Shift+P` fires a
/// `Ctrl+P` binding unless it has its own). Any other chord is matched as the terminal UI does:
/// exact bindings first, then the keymap's lenient match.
pub fn resolve(km: &Keymap, pane: Ctx, chord: KeyChord) -> Resolved {
    let mut contexts = vec![pane];
    if pane != Ctx::Global {
        contexts.push(Ctx::Global);
    }
    let exact = |c: KeyChord| contexts.iter().find_map(|ctx| km.action_exact(*ctx, c));
    let found = if is_command_chord(chord) {
        exact(chord).or_else(|| {
            chord
                .mods
                .shift
                .then(|| {
                    exact(KeyChord::new(
                        Mods {
                            shift: false,
                            ..chord.mods
                        },
                        chord.key,
                    ))
                })
                .flatten()
        })
    } else {
        exact(chord).or_else(|| contexts.iter().find_map(|ctx| km.action_for(*ctx, chord)))
    };
    found.map_or(Resolved::Nothing, Resolved::Command)
}

fn in_popup(event: &KeystrokeEvent, window: &mut Window, cx: &mut App) -> bool {
    window.has_active_dialog(cx)
        || event
            .context_stack
            .iter()
            .any(|c| c.contains("GpPicker") || c.contains("GpMenu") || c.contains("PopupMenu"))
}

fn on_keystroke(event: &KeystrokeEvent, window: &mut Window, cx: &mut App) {
    if event
        .context_stack
        .iter()
        .any(|c| c.contains(picker::CONTEXT))
    {
        let ks = &event.keystroke;
        if picker::navigate(&ks.key, ks.modifiers.control && !ks.modifiers.alt, cx) {
            cx.stop_propagation();
        }
        return;
    }
    if in_popup(event, window, cx) {
        return;
    }
    let Some(chord) = chord_of(&event.keystroke) else {
        return;
    };
    let in_field = event.context_stack.iter().any(|c| c.contains("Input"));

    // The second key of a Ctrl+G / Ctrl+N chord.
    if let Some(prefix) = take_prefix(cx) {
        cx.stop_propagation();
        status::clear(cx);
        if chord.key == Key::Esc {
            return;
        }
        if let Some(action) = prefix_action(&keymap::current(), prefix, chord) {
            exec::run(Command::Action(action), window, cx);
        }
        return;
    }

    if in_field && (!is_command_chord(chord) || is_field_editing(chord)) {
        return;
    }

    let km = keymap::current();
    let pane = window_state(window, cx).pane;
    // F10 and Alt+<letter> reach the menu bar when no action holds the chord.
    let Resolved::Command(action) = resolve(&km, pane, chord) else {
        if crate::menu::handle_chord(chord, window, cx) {
            cx.stop_propagation();
        }
        return;
    };
    let implemented = op_for(Command::Action(action)).is_some();
    if implemented || is_command_chord(chord) {
        cx.stop_propagation();
        exec::run(Command::Action(action), window, cx);
    }
}

/// Builds the keymap from `config.toml` in `config_dir` (the defaults without one) and installs it.
/// The entries that could not be honoured come back as the startup message.
pub fn install_keymap(config_dir: Option<&Path>) -> Vec<KeymapIssue> {
    let config = config_dir
        .and_then(|dir| gitpoulpi_config::Config::load(&dir.join("config.toml")).ok())
        .unwrap_or_default();
    keymap::install_from_config_with(&config, "gitpoulpi-desktop", Vec::new())
}

/// Installs the keymap, how keys reach this frontend and the keystroke interceptor.
pub fn install(config_dir: Option<&Path>, cx: &mut App) {
    install_transport(|| KeyTransport::Desktop);
    let issues = install_keymap(config_dir);
    if let Some(text) = keymap::issues_message(&issues) {
        status::set(cx, text);
    }
    cx.set_global(PendingPrefix(None));
    cx.intercept_keystrokes(on_keystroke).detach();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chord(text: &'static str) -> Option<KeyChord> {
        chord_of(&Keystroke::parse(text).unwrap())
    }

    fn want(text: &str) -> KeyChord {
        KeyChord::parse(text).unwrap().class()
    }

    #[test]
    fn keystrokes_spell_the_terminal_chords() {
        assert_eq!(chord("ctrl-p"), Some(want("Ctrl+P")));
        assert_eq!(chord("ctrl-shift-p"), Some(want("Ctrl+Shift+P")));
        assert_eq!(chord("alt-t"), Some(want("Alt+T")));
        assert_eq!(chord("f5"), Some(want("F5")));
        assert_eq!(chord("shift-f10"), Some(want("Shift+F10")));
        assert_eq!(chord("pagedown"), Some(want("PageDown")));
        assert_eq!(chord("ctrl-,"), Some(want("Ctrl+,")));
        assert_eq!(chord("q"), Some(want("q")));
        assert_eq!(chord("shift-h"), Some(want("Shift+H")));
        assert_eq!(chord("shift-tab"), Some(want("Shift+Tab")));
        assert_eq!(chord("?"), Some(want("?")));
        assert_eq!(chord("space"), Some(want("Space")));
    }

    #[test]
    fn typed_characters_win_over_the_key() {
        let mut ks = Keystroke::parse("shift-/").unwrap();
        ks.key_char = Some("?".into());
        assert_eq!(chord_of(&ks), Some(want("?")));
    }

    #[test]
    fn a_bare_modifier_is_no_chord() {
        assert_eq!(chord("shift"), None);
    }

    #[test]
    fn resolution_looks_in_the_pane_then_global() {
        let km = Keymap::defaults();
        assert_eq!(
            resolve(&km, Ctx::Log, want("Ctrl+P")),
            Resolved::Command(Action::GlobalPalette)
        );
        assert_eq!(
            resolve(&km, Ctx::Global, want("Ctrl+P")),
            Resolved::Command(Action::GlobalPalette)
        );
        assert_eq!(
            resolve(&km, Ctx::Global, want("Ctrl+Alt+Shift+F9")),
            Resolved::Nothing
        );
    }

    #[test]
    fn the_second_key_of_a_prefix_keeps_its_shift() {
        let km = Keymap::defaults();
        assert_eq!(
            prefix_action(&km, Ctx::NavPrefix, want("Shift+H")),
            Some(Action::NavCollapseLeft)
        );
        assert_eq!(
            prefix_action(&km, Ctx::NavPrefix, want("t")),
            Some(Action::NavChangeTheme)
        );
        // Ctrl held through the second key still works for a plain key.
        assert_eq!(
            prefix_action(&km, Ctx::NavPrefix, want("Ctrl+T")),
            Some(Action::NavChangeTheme)
        );
        assert_eq!(prefix_action(&km, Ctx::NavPrefix, want("Ctrl+Alt+Q")), None);
    }

    #[test]
    fn a_pane_letter_does_not_shadow_a_ctrl_chord() {
        let km = Keymap::defaults();
        // `n` is bound in the Log (next compared row); Ctrl+N is the navigation prefix.
        assert_eq!(
            resolve(&km, Ctx::Log, want("Ctrl+N")),
            Resolved::Command(Action::GlobalNavPrefix)
        );
    }

    #[test]
    fn an_alt_letter_is_not_a_bare_letter() {
        let km = Keymap::defaults();
        // `f` searches the Log; Alt+F is free for the File menu.
        assert_eq!(
            resolve(&km, Ctx::Global, want("f")),
            Resolved::Command(Action::GlobalFindInLog)
        );
        assert_eq!(resolve(&km, Ctx::Global, want("Alt+F")), Resolved::Nothing);
        // An extra Shift is tolerated on a Ctrl chord.
        assert_eq!(
            resolve(&km, Ctx::Global, want("Ctrl+Shift+P")),
            Resolved::Command(Action::GlobalPalette)
        );
    }

    #[test]
    fn a_remapped_key_follows_the_config() {
        let km = keymap::test_keymap(&[("global", "quit", &["Ctrl+Q"])]);
        assert_eq!(
            resolve(&km, Ctx::Global, want("Ctrl+Q")),
            Resolved::Command(Action::GlobalQuit)
        );
        assert_eq!(resolve(&km, Ctx::Global, want("q")), Resolved::Nothing);
    }

    #[test]
    fn a_field_keeps_its_editing_keys() {
        assert!(is_field_editing(want("Ctrl+A")));
        assert!(is_field_editing(want("Backspace")));
        assert!(is_field_editing(want("Left")));
        assert!(!is_field_editing(want("Ctrl+P")));
        assert!(!is_field_editing(want("F5")));
        assert!(is_command_chord(want("Ctrl+P")));
        assert!(!is_command_chord(want("q")));
    }
}
