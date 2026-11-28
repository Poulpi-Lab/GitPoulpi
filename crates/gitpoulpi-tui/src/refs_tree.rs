//! The Refs sidebar's tree model lives in `gitpoulpi_app::refs_tree`, shared
//! with the desktop frontend. This module re-exports it at the old path and
//! keeps the two parts that need this frontend: [`ref_display_rows`] (it
//! hands the process-wide workflow view to the model) and
//! [`submit_ref_visibility`] (it speaks to the daemon link).

use std::collections::HashSet;

pub use gitpoulpi_app::refs_tree::*;

/// [`gitpoulpi_app::refs_tree::ref_display_rows_with_flow`] with the window's
/// workflow view (none while the plain trees are shown).
pub fn ref_display_rows(
    names: &[String],
    kind: RefRowKind,
    hierarchical: bool,
    collapsed_folders: &CollapsedFolders,
    remotes: &[String],
) -> Vec<RefTreeRow> {
    let flow = if crate::flow::view::plain() {
        None
    } else {
        crate::flow::view::configured()
    };
    ref_display_rows_with_flow(
        names,
        kind,
        hierarchical,
        collapsed_folders,
        remotes,
        flow.as_deref(),
    )
}

/// Send the hidden/soloed sets to the daemon, which writes them and tells
/// every other frontend (`Event::RefVisibilityChanged`). Names are sorted so
/// the request is deterministic.
pub fn submit_ref_visibility(
    link: &crate::daemon_link::DaemonLink,
    hidden: &HashSet<String>,
    solo: &HashSet<String>,
) {
    let (hidden, hidden_remotes) = gitpoulpi_app::refs_tree::split_hidden(hidden);
    let mut solo: Vec<String> = solo.iter().cloned().collect();
    solo.sort();
    link.submit_action(crate::daemon_link::Action::SetRefVisibility {
        hidden,
        solo,
        hidden_remotes,
    });
}
