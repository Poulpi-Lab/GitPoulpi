//! Branching workflows in the TUI (plan `branching-workflows.md`, slice W3).
//!
//! - [`view`]: the daemon's `Workflow` report kept for the whole window, the
//!   grouped sidebar rows and the branch tooltips built from it.
//! - [`source`]: keeps that report fresh (one request in flight).
//! - [`cmd`]: the commands every surface runs (`Flow ▾` dropdown, palette,
//!   keys).
//! - [`form`]: the Start form's pure logic (pattern tokens, suggestions).
//! - [`dialog`]: Start / Finish / Sync / Promote / Set parent.
//! - [`actions`]: the [`FlowActions`] seam to slice W2's daemon requests,
//!   with a recording fake.
//!
//! The Settings page lives with the other pages (`preferences/workflow_page`).

pub(crate) mod actions;
pub(crate) mod cmd;
pub(crate) mod daemon;
pub(crate) mod dialog;
pub(crate) mod form;
pub(crate) mod hook_trust;
#[cfg(feature = "hosting")]
pub(crate) mod issue_flow;
#[cfg(feature = "hosting")]
pub(crate) mod issue_watch;
pub(crate) mod mr;
pub(crate) mod source;
pub(crate) mod view;

pub(crate) use actions::{default_actions, FlowActions};
pub(crate) use cmd::{menu_rows, request, take_request, FlowCmd, FlowCmdExt};
pub(crate) use dialog::{flow_rect, FlowDialog, FlowKind, FlowOutcome, FlowSeed};
pub(crate) use source::FlowSource;

#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod daemon_tests;
