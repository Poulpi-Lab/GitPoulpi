//! YouTrack (Cloud and Server) issue-tracker client (slice T1).
//!
//! An issue source only. Both editions speak the same REST API
//! (`<url>/api/...`, every call names its `fields=`), authenticated by a
//! permanent token as `Authorization: Bearer`. Projects are keyed by their
//! short name (`ABC`), the references are `ABC-12`. The per-repository
//! filter is a YouTrack query (`#Unresolved for: me`), like JQL for Jira.
//!
//! Mapping: the `State` custom field is the status (and the board's
//! columns, from its bundle), `Assignee` the assignee, tags the labels and
//! `Fix versions` the milestone. A version has no numeric id, so the
//! milestone id the pickers speak is its 1-based position in the project's
//! list of unreleased, unarchived versions (the same rule as the local
//! Markdown source).

mod client;
mod map;
mod query;
pub mod spec;
#[cfg(test)]
mod tests;

pub(crate) use client::YouTrackClient;
