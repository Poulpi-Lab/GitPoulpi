//! Redmine issue-tracker client (slice T2).
//!
//! An issue source only: the REST API (`<url>/issues.json` ...) answers JSON
//! when the administrator turned "Enable REST web service" on, and a user's
//! API key travels as `X-Redmine-API-Key`. Projects are keyed by their
//! identifier (`bench`), the references are `#123`: a bare `#N` only means
//! a Redmine issue when Redmine is the repository's issue source.
//!
//! Mapping: the tracker is the kind (Bug, Feature), the status the status
//! (and the Kanban board's columns, from `/issue_statuses.json`; moving a
//! card is a status change), `assigned_to` the one assignee, `fixed_version`
//! the milestone. Core Redmine has no labels, so none are listed or edited.

mod client;
mod map;
pub mod spec;
#[cfg(test)]
mod tests;

pub(crate) use client::RedmineClient;
