//! The dispatcher's table: every command the desktop implements, as an [`Op`].
//!
//! [`op_for`] is the only place that says "the desktop does this". A command it maps to `None` is not
//! implemented yet: running it answers "not available yet" in the status line (see `exec`). The parity
//! table derives its `done` set from this function ([`implemented_ids`]), so a command cannot be marked
//! done without being handled here.

use std::collections::BTreeSet;

use gitpoulpi_app::keymap::{Action, Ctx};
use gitpoulpi_app::palette::vocab::log_table::LogColumnCmd;
use gitpoulpi_app::palette::{palette_entry, PaletteCommand};
use gitpoulpi_app::registry::{parity_rows, ParityKind};

use super::catalog::{command_for_id, Command};

/// What the desktop does for a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    OpenPalette,
    Quit,
    /// Open the Log's find bar.
    FindInLog,
    /// Reload the refs and the Log.
    Refresh,
    /// The keybindings view.
    Keybindings,
    /// The first key of a two-key chord (Ctrl+G, Ctrl+N): the next key is looked up in that context.
    Prefix(Ctx),
    ToggleRefsPane,
    ToggleStatusPane,
    ChangeTheme,
    ChangeLanguage,
    ChangeGraphColors,
    FocusRefs,
    FocusRefsFilter,
    FocusLog,
    FocusStatus,
    /// The folder chooser of the operating system, then open that repository.
    BrowseRepository,
    FitColumns,
    ResetColumns,
}

/// The op of a key action, if the desktop implements it.
pub fn op_for_action(action: Action) -> Option<Op> {
    use Action as A;
    Some(match action {
        A::GlobalPalette => Op::OpenPalette,
        A::GlobalQuit => Op::Quit,
        A::GlobalFindInLog => Op::FindInLog,
        A::GlobalRefresh => Op::Refresh,
        A::GlobalHelp | A::NavToggleHelp => Op::Keybindings,
        A::GlobalGitPrefix => Op::Prefix(Ctx::GitPrefix),
        A::GlobalNavPrefix => Op::Prefix(Ctx::NavPrefix),
        A::NavCollapseLeft => Op::ToggleRefsPane,
        A::NavCollapseRight => Op::ToggleStatusPane,
        A::NavChangeTheme => Op::ChangeTheme,
        A::NavChangeLanguage => Op::ChangeLanguage,
        A::NavChangeGraphColorTheme => Op::ChangeGraphColors,
        A::LogColumnsFitAll => Op::FitColumns,
        A::LogColumnsResetWidth => Op::ResetColumns,
        // A palette command bound to a key of its own.
        A::Palette(_) => return palette_entry(action).and_then(|e| op_for_palette(e.command)),
        _ => return None,
    })
}

/// The op of a palette command, if the desktop implements it.
pub fn op_for_palette(command: PaletteCommand) -> Option<Op> {
    use PaletteCommand as C;
    Some(match command {
        C::Quit => Op::Quit,
        C::SearchLog => Op::FindInLog,
        C::Refresh => Op::Refresh,
        C::ToggleHelp | C::Keybindings => Op::Keybindings,
        C::ToggleRefsPaneCollapsed => Op::ToggleRefsPane,
        C::ToggleStatusPaneCollapsed => Op::ToggleStatusPane,
        C::ChangeTheme | C::Theme => Op::ChangeTheme,
        C::ChangeLanguage => Op::ChangeLanguage,
        C::ChangeGraphColorTheme => Op::ChangeGraphColors,
        C::FocusRefs => Op::FocusRefs,
        C::FocusRefsFilter => Op::FocusRefsFilter,
        C::FocusLog => Op::FocusLog,
        C::FocusStatus => Op::FocusStatus,
        C::BrowseRepository => Op::BrowseRepository,
        C::LogColumn(LogColumnCmd::FitAllToContent) => Op::FitColumns,
        C::LogColumn(LogColumnCmd::ResetWidth) => Op::ResetColumns,
        _ => return None,
    })
}

/// The op of a command, `None` when the desktop does not implement it yet.
pub fn op_for(command: Command) -> Option<Op> {
    match command {
        Command::Action(a) => op_for_action(a),
        Command::Palette(c) => op_for_palette(c),
    }
}

/// The ids (of the registry's actions and palette commands) the dispatcher implements.
pub fn implemented_ids() -> BTreeSet<String> {
    parity_rows()
        .into_iter()
        .filter(|r| matches!(r.kind, ParityKind::Action | ParityKind::Palette))
        .filter(|r| command_for_id(&r.id).and_then(op_for).is_some())
        .map(|r| r.id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_app::palette::PALETTE_COMMANDS;

    /// A palette command and the key action it is the only twin of must do the same thing, or the merged
    /// parity row would say "done" for one and not the other.
    #[test]
    fn a_command_and_its_key_action_agree() {
        use gitpoulpi_app::palette::bindings::static_bind_action;
        for entry in PALETTE_COMMANDS {
            let Some(action) = static_bind_action(entry) else {
                continue;
            };
            let twins = PALETTE_COMMANDS
                .iter()
                .filter(|e| static_bind_action(e) == Some(action))
                .count();
            if twins != 1 {
                continue;
            }
            assert_eq!(
                op_for_action(action),
                op_for_palette(entry.command),
                "{:?}",
                entry.command
            );
        }
    }

    #[test]
    fn the_known_commands_are_implemented() {
        let ids = implemented_ids();
        for id in [
            "global.palette",
            "global.quit",
            "global.find_in_log",
            "nav_prefix.collapse_refs",
            "nav_prefix.collapse_status",
            "nav_prefix.change_theme",
            "nav_prefix.change_language",
            "nav_prefix.change_graph_color_theme",
            "palette.focus_log",
        ] {
            assert!(ids.contains(id), "{id}");
        }
        assert!(!ids.contains("git_prefix.fetch_all"));
    }
}
