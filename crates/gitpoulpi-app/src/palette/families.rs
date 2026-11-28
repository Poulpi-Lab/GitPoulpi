//! What a frontend feeds the palette: the generated rows that depend on its
//! window (`Flow: Start <Type>...`, `Jobs: Cancel <job>`, the sequence run),
//! the value shown beside a `Settings: ...` row and the few facts about
//! process state that the availability rules read.
//!
//! The catalogue, the matching, the availability rules and the `Settings: ...`
//! rows (generated from the shared Settings model,
//! [`crate::settings::palette_rows`]) live here and are shared; the rows and
//! the state that belong to the frontend that owns them are installed once
//! with [`install_families`]. Until it does, every family is empty and every
//! probe says "nothing".

use super::vocab::flow::FlowCmd;
use super::{PaletteEntry, SettingOp};
use std::sync::OnceLock;

/// The frontend's side of the palette. Plain function pointers, so a frontend
/// can build it as a `static` or a literal.
#[derive(Clone, Copy)]
pub struct Families {
    /// The text shown in the shortcut column of a `Settings: ...` row.
    pub settings_value_text: fn(SettingOp) -> String,
    /// The `Flow: Start <Type>...` rows of the repository's workflow.
    pub flow_entries: fn() -> &'static [PaletteEntry],
    /// What the flow rows depend on besides the language (a cache key).
    pub flow_signature: fn() -> u64,
    /// Why a flow command does nothing right now.
    pub flow_unavailable: fn(FlowCmd) -> Option<&'static str>,
    /// The `Jobs: Cancel <job>` rows.
    pub jobs_entries: fn() -> &'static [PaletteEntry],
    pub jobs_signature: fn() -> u64,
    /// Some background job can be canceled right now.
    pub any_job_cancellable: fn() -> bool,
    /// A sequence run is on screen (running, or failed and waiting).
    pub sequence_running: fn() -> bool,
    /// The sequence run can be continued now.
    pub sequence_can_continue: fn() -> bool,
    /// The repository's own sequences file waits for an answer.
    pub repo_sequences_pending: fn() -> bool,
    /// The last operation left an error to show.
    pub status_error_present: fn() -> bool,
    /// ...and it has a remedy that can be run.
    pub last_error_has_remedy: fn() -> bool,
    /// The reason text of "nothing to undo".
    pub nothing_to_undo: fn() -> &'static str,
}

fn no_entries() -> &'static [PaletteEntry] {
    &[]
}
fn no_text(_: SettingOp) -> String {
    String::new()
}
fn zero() -> u64 {
    0
}
fn no_flow(_: FlowCmd) -> Option<&'static str> {
    None
}
fn never() -> bool {
    false
}
fn nothing() -> &'static str {
    gitpoulpi_i18n::msg::undo_ui::NOTHING_TO_UNDO.get()
}

impl Families {
    /// No generated rows and no process state.
    pub const NONE: Families = Families {
        settings_value_text: no_text,
        flow_entries: no_entries,
        flow_signature: zero,
        flow_unavailable: no_flow,
        jobs_entries: no_entries,
        jobs_signature: zero,
        any_job_cancellable: never,
        sequence_running: never,
        sequence_can_continue: never,
        repo_sequences_pending: never,
        status_error_present: never,
        last_error_has_remedy: never,
        nothing_to_undo: nothing,
    };
}

static INSTALLED: OnceLock<Families> = OnceLock::new();

/// Publishes the frontend's families; the first call wins.
pub fn install_families(families: Families) {
    let _ = INSTALLED.set(families);
}

/// The installed families, or [`Families::NONE`].
pub fn families() -> Families {
    INSTALLED.get().copied().unwrap_or(Families::NONE)
}

/// Whether the entry is listed for `query` (trimmed, lower case): a settings
/// choice row only once the query names its setting (`Settings: <label>:`).
pub fn settings_listed(entry: &PaletteEntry, query: &str, prefixes: &[String]) -> bool {
    match entry.command {
        super::PaletteCommand::Setting(SettingOp::Choose(r, _)) => prefixes
            .get(r as usize)
            .is_some_and(|p| !p.is_empty() && query.starts_with(p.as_str())),
        _ => true,
    }
}
