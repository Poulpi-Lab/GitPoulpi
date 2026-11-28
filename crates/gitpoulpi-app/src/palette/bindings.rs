//! Palette commands as bindable keymap actions.
//!
//! A command that has no key action of its own (`command_action`, or a
//! `Ctrl+G`/`Ctrl+N` default in its catalogue text) gets one here:
//! `Action::Palette(i)`, `i` indexing `PALETTE_COMMANDS`, config key
//! `[keybindings.palette] <command id> = "chord"`. The id is derived from
//! the variant (and its payload), so every value of `DiffMode(..)` or
//! `ToggleColumn(..)` has its own. Unbound by default; the key runs the same
//! `run_command` the palette runs.

use super::{
    command_action, default_shortcut_actions, PaletteCommand, PaletteEntry, BADGE_SUFFIX,
    PALETTE_COMMANDS,
};
use crate::keymap::action::ActionDef;
use crate::keymap::{Action, Ctx};

/// The stable config name of a command: its `Debug` spelling in snake case
/// (`StashPop(Latest)` is `stash_pop_latest`). Renaming a variant renames it,
/// so a test pins the list.
pub fn command_id(command: &PaletteCommand) -> String {
    let debug = format!("{command:?}");
    let mut out = String::with_capacity(debug.len() + 8);
    let mut prev_lower = false;
    for ch in debug.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && prev_lower {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
            prev_lower = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        } else {
            if !out.ends_with('_') && !out.is_empty() {
                out.push('_');
            }
            prev_lower = false;
        }
    }
    out.trim_end_matches('_').to_string()
}

/// The keymap action a command already has (a registered one), if any.
pub fn static_bind_action(entry: &PaletteEntry) -> Option<Action> {
    if let Some(action) = command_action(entry) {
        return Some(action);
    }
    let defaults = default_shortcut_actions();
    entry.shortcut.split(" / ").find_map(|alt| {
        let alt = alt.trim();
        let alt = alt.strip_suffix(BADGE_SUFFIX).unwrap_or(alt);
        defaults.iter().find(|(l, _)| l == alt).map(|(_, a)| *a)
    })
}

/// The bindable action of a command with no registered one.
pub fn palette_action(entry: &PaletteEntry) -> Option<Action> {
    if static_bind_action(entry).is_some() {
        return None;
    }
    PALETTE_COMMANDS
        .iter()
        .position(|e| e.command == entry.command)
        .map(|i| Action::Palette(i as u16))
}

/// The entry an `Action::Palette(i)` stands for.
pub fn palette_entry(action: Action) -> Option<&'static PaletteEntry> {
    match action {
        Action::Palette(i) => PALETTE_COMMANDS.get(i as usize),
        _ => None,
    }
}

/// One `ActionDef` per command without a registered action (built once by
/// `keymap::action::all_actions`).
pub fn palette_action_defs() -> Vec<ActionDef> {
    PALETTE_COMMANDS
        .iter()
        .enumerate()
        .filter(|(_, e)| static_bind_action(e).is_none())
        .map(|(i, e)| ActionDef {
            action: Action::Palette(i as u16),
            static_id: Box::leak(format!("palette.{}", command_id(&e.command)).into_boxed_str()),
            ctx: Ctx::Palette,
            static_label: e.label,
            defaults: &[],
            rebindable: true,
            shadows_parent: false,
            hint: None,
            requires: e.requires,
            surface: e.surface,
        })
        .collect()
}
