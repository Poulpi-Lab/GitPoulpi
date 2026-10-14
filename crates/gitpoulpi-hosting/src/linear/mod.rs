//! Linear issue-tracker client (slice T1).
//!
//! An issue source only: no pull requests, no SSH keys. A personal API key
//! goes in the `Authorization` header as is (no `Bearer`); everything is one
//! GraphQL endpoint (`https://api.linear.app/graphql`). Teams are the
//! "projects" (`ENG`), the references `ENG-123`. Cursor paging.
//!
//! Choices: the "milestone" of the metadata pickers is the team's **cycle**
//! (its `number` fits the numeric id the pickers speak; a project milestone
//! has only a UUID); the New-issue form's "list" is the workflow **state**.

mod client;
mod map;
mod queries;
pub mod spec;
#[cfg(test)]
mod tests;

pub(crate) use client::LinearClient;
