//! The application menu bar (File, Edit, View, Repository, Branch, Git, Hosting, Tools, Help),
//! generated from the shared registry: `model` lays the palette catalogue out in menus, `bar` draws it.
//!
//! Reachable by keyboard: F10 opens the first menu, Alt + a heading's mnemonic opens that one (when no
//! action holds the chord).

pub mod bar;
pub mod model;

pub use bar::MenuBar;

use gitpoulpi_app::keymap::chord::{Key, KeyChord};
use gpui_kit::{App, Entity, Global, Window};

/// The main window's menu bar, for the keystroke interceptor.
struct MenuHandle(gpui_kit::WeakEntity<MenuBar>);
impl Global for MenuHandle {}

/// Makes `bar` the one F10 and the mnemonics reach.
pub fn register(bar: &Entity<MenuBar>, cx: &mut App) {
    cx.set_global(MenuHandle(bar.downgrade()));
}

fn bar_of(cx: &App) -> Option<Entity<MenuBar>> {
    cx.try_global::<MenuHandle>().and_then(|h| h.0.upgrade())
}

/// What a chord does to the menu bar: F10 opens the first menu, Alt + a mnemonic opens that menu.
/// `true` when the chord was taken.
pub fn handle_chord(chord: KeyChord, window: &mut Window, cx: &mut App) -> bool {
    let Some(bar) = bar_of(cx) else {
        return false;
    };
    let m = chord.mods;
    match chord.key {
        Key::F(10) if !m.ctrl && !m.alt && !m.shift => {
            bar.update(cx, |b, cx| b.open_first(window, cx));
            true
        }
        Key::Char(c) if m.alt && !m.ctrl && !m.shift => {
            bar.update(cx, |b, cx| b.open_mnemonic(c, window, cx))
        }
        _ => false,
    }
}
