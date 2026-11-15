//! Gerrit Code Review client (slice T3): the review view's host for
//! patch-set based review. Review only: Gerrit has no issues, and a change
//! is created by pushing to `refs/for/<branch>`, never by an API call.
//!
//! Authentication is HTTP basic with the user name (`Integration::account`)
//! and an HTTP password / token; the REST API lives under `<url>/a/` and
//! every JSON answer starts with the guard line `)]}'`.
//!
//! Mapping: a change is a pull request (`#<number>`), a patch set a
//! revision. The diff comes from git: `refs/changes/NN/<change>/<ps>` is
//! fetched into `refs/gitpoulpi/pr/<change>`. A comment thread is a root
//! comment and everything that answers it; the newest comment decides
//! whether it is resolved (`unresolved`). Votes are labels (`Code-Review`,
//! `Verified`...) with the values the server permits for the change.

mod client;
#[cfg(test)]
mod code_tests;
#[cfg(test)]
mod conversation_tests;
pub(crate) mod map;
pub mod spec;
#[cfg(test)]
mod tests;

pub use map::{change_ref, patch_set_of_ref};

pub(crate) use client::GerritClient;
