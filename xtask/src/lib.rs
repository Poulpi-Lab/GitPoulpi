//! Repeatable dev tasks, as a library so the workspace's own tests can use
//! them and as a binary (`cargo run -p xtask -- <task>`) so a person or an
//! agent can run them by hand.
//!
//! The library half exists for `pty`: it is how the TUI is verified, both
//! from `crates/gitpoulpi-tui/tests/` and interactively.

pub mod advisory;
pub mod custom_build;
pub mod desktop_lock;
pub mod dist;
pub mod emit;
pub mod features;
pub mod licenses;
pub mod lock_age;
pub mod merge_fixture;
pub mod mock_host;
pub mod perf;
pub mod perf_merge;
pub mod perf_rewrite;
pub mod perf_search;
pub mod perf_tui;
pub mod pty;
pub mod script;
pub mod sign;
pub mod test_failures;
pub mod test_runner;
pub mod test_tiers;
