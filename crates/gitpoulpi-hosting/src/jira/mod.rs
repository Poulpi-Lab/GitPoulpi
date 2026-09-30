//! Jira (Cloud and Data Center) issue-tracker client (slice S5).
//!
//! An issue source only: no pull requests, no SSH keys. Cloud takes a
//! classic API token (Basic `email:token`) against the site URL; Data Center
//! takes a personal access token (Bearer).

mod client;
mod jql;
mod map;
pub mod spec;
#[cfg(test)]
mod tests;

pub(crate) use client::JiraClient;
