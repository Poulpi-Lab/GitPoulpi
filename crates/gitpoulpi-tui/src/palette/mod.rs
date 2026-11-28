//! The Ctrl+P command palette. The catalogue, the matching and the
//! availability rules are frontend-neutral and live in `gitpoulpi_app::palette`;
//! this module re-exports them at their old paths and adds what is the
//! terminal window's own:
//!
//! - `settings`: the `Settings: ...` rows, generated from the Preferences
//!   field model;
//! - `flow` and `jobs`: the `Flow: Start <Type>...` and `Jobs: Cancel <job>`
//!   rows, built from this window's state;
//! - [`tui_families`], the hook table handed to
//!   `gitpoulpi_app::palette::install_families`.
//!
//! The entry points that read those hooks ([`palette_matches`],
//! [`unavailable_reason`], [`entry_shortcut`]) make sure they are installed
//! first, so a unit test and the real window see the same wiring.

use crate::flow::FlowCmdExt;
use gitpoulpi_app::palette::Families;

mod flow;
mod jobs;
mod settings;

#[allow(unused_imports)]
pub(crate) use gitpoulpi_app::palette::*;

// What the palette's own tests reach through `use super::*`.
#[cfg(test)]
#[allow(unused_imports)]
use {
    crate::diff_view::DiffViewMode,
    crate::repo_menu::RepoMenuAction,
    crate::{
        AmendScope, CommitField, HostingCommand, LogColumn, RefField, SidebarSection, StatusSection,
    },
    gitpoulpi_config::PullButtonMode,
    gitpoulpi_core::domain::{ConflictSide, ResetMode},
    gitpoulpi_i18n::msg,
};

#[cfg(test)]
pub(crate) use flow::entries as flow_entries;
#[cfg(test)]
pub(crate) use jobs::entries as jobs_entries;
pub(crate) use settings::{
    listed_count as settings_listed_count, run as run_setting, sub_query as setting_sub_query,
    Outcome as SettingOutcome,
};

impl crate::ToolbarButton {
    /// The palette command a toolbar button runs: the button is a mouse
    /// twin of it, and clicking one goes through the same `run_command`.
    pub(crate) fn command(self) -> PaletteCommand {
        use crate::ToolbarButton as B;
        match self {
            B::Undo => PaletteCommand::Undo,
            B::Redo => PaletteCommand::Redo,
            B::Pull => PaletteCommand::Pull,
            B::Push => PaletteCommand::Push,
            B::Branch => PaletteCommand::NewBranch,
            B::Tag => PaletteCommand::NewTag,
            B::Stash => PaletteCommand::StashPush,
            // The toolbar pops the latest stash; the palette entry the
            // selected one (see [`StashTarget`]).
            B::Pop => PaletteCommand::StashPop(StashTarget::Latest),
            B::Search => PaletteCommand::SearchLog,
            // The LFS button is the dropdown itself.
            B::Lfs => PaletteCommand::LfsMenu,
            B::Flow => PaletteCommand::Flow(crate::flow::FlowCmd::Menu),
        }
    }
}

/// This window's side of the palette: its generated rows and the process
/// state the availability rules read.
pub(crate) fn tui_families() -> Families {
    Families {
        settings_value_text: settings::value_text,
        flow_entries: flow::entries,
        flow_signature: flow::signature,
        flow_unavailable: |cmd| cmd.unavailable(),
        jobs_entries: jobs::entries,
        jobs_signature: jobs::signature,
        any_job_cancellable: crate::jobs_ui::any_cancellable,
        sequence_running: crate::sequence_run::visible,
        sequence_can_continue: crate::sequence_run::can_continue,
        repo_sequences_pending: || crate::sequence_run::repo::pending().is_some(),
        status_error_present: crate::activity_log::status_error::has_error,
        last_error_has_remedy: crate::activity_log::last_error_has_remedy,
        nothing_to_undo: crate::undo_ui::nothing_to_undo,
    }
}

/// Why `command` cannot run in `ctx`, or `None`.
pub(crate) fn unavailable_reason(
    command: PaletteCommand,
    ctx: PaletteContext,
) -> Option<&'static str> {
    crate::app_hooks::ensure();
    gitpoulpi_app::palette::unavailable_reason(command, ctx)
}

/// The palette's filtered, ranked rows for `query` in the given context.
pub(crate) fn palette_matches(query: &str, ctx: PaletteContext) -> Vec<PaletteMatch> {
    crate::app_hooks::ensure();
    gitpoulpi_app::palette::palette_matches(query, ctx)
}

/// What the palette shows right-aligned for `entry` under `km`.
pub(crate) fn entry_shortcut(km: &crate::keymap::Keymap, entry: &PaletteEntry) -> String {
    crate::app_hooks::ensure();
    gitpoulpi_app::palette::entry_shortcut(km, entry)
}

#[cfg(test)]
mod tests;
