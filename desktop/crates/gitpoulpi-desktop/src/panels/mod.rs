//! The window's panes. Each one is a GPUI entity that talks to the daemon through `daemon::Backend`
//! and reports to the workspace through events; none of them knows about the others.

pub mod activity;
pub mod diff;
pub mod log;
pub mod refs;
pub mod right;
