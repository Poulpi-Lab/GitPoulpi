//! [`Command`]: what a key binding, a palette row or a menu item stands for, and the stable id the
//! parity table uses for it.

use std::collections::HashMap;
use std::sync::OnceLock;

use gitpoulpi_app::keymap::{Action, STATIC_ACTIONS};
use gitpoulpi_app::palette::{command_id, PaletteCommand, PaletteEntry, PALETTE_COMMANDS};

/// One thing the user can ask for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    /// A key action of the registry.
    Action(Action),
    /// A command of the palette catalogue.
    Palette(PaletteCommand),
}

/// The parity id of an action: its stable config id.
pub fn action_id(action: Action) -> Option<&'static str> {
    STATIC_ACTIONS
        .iter()
        .find(|d| d.action == action)
        .map(|d| d.static_id)
}

/// The parity id of a palette command: `palette.<command id>`.
pub fn palette_id(command: &PaletteCommand) -> String {
    format!("palette.{}", command_id(command))
}

/// The id of a command (an action's, or a palette command's).
pub fn id_of(command: Command) -> Option<String> {
    match command {
        Command::Action(a) => action_id(a).map(str::to_string),
        Command::Palette(c) => Some(palette_id(&c)),
    }
}

fn index() -> &'static HashMap<String, Command> {
    static INDEX: OnceLock<HashMap<String, Command>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut map = HashMap::new();
        for def in STATIC_ACTIONS {
            map.insert(def.static_id.to_string(), Command::Action(def.action));
        }
        for entry in PALETTE_COMMANDS {
            map.insert(palette_id(&entry.command), Command::Palette(entry.command));
        }
        map
    })
}

/// The command with this id (`global.quit`, `palette.refresh`, ...).
pub fn command_for_id(id: &str) -> Option<Command> {
    index().get(id).copied()
}

/// The catalogue entry of a palette command.
pub fn entry_of(command: PaletteCommand) -> Option<&'static PaletteEntry> {
    PALETTE_COMMANDS.iter().find(|e| e.command == command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip() {
        for id in ["global.quit", "palette.refresh", "nav_prefix.change_theme"] {
            let cmd = command_for_id(id).unwrap_or_else(|| panic!("{id}"));
            assert_eq!(id_of(cmd).as_deref(), Some(id));
        }
        assert!(command_for_id("no.such_id").is_none());
    }
}
