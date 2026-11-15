//! Trello issue-tracker client (slice S6).
//!
//! An issue source only: boards are the projects, cards the issues, lists
//! the "types". The key and token travel in the `Authorization: OAuth ...`
//! header, never in a URL.

mod client;
mod conversation;
#[cfg(test)]
mod conversation_tests;
mod map;
pub mod spec;
#[cfg(test)]
mod tests;

pub(crate) use client::TrelloClient;
