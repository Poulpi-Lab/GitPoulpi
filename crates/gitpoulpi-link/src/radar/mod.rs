//! The Radar's frontend-neutral half: the data and the background worker
//! behind the multi-repository list of pull requests, issues and work in
//! progress. A frontend draws the rows; the logic lives here.
//!
//! - [`model`]: items, groups, pins, snoozes and the rows of a view, folded
//!   from what the repositories say. Pure: the clock is a parameter. Which
//!   actions a row offers is decided by `Item::meta` / `Item::review` (from
//!   `gitpoulpi_app::provider_caps`) together with the `Act` vocabulary of
//!   `gitpoulpi_app::palette::vocab::radar`.
//! - [`account`]: the account-wide rows (what the integrations list across
//!   all repositories) and the notes about repositories that need a link.
//! - [`worker`]: the background protocol. A frontend builds a
//!   [`worker::RadarWorker`] with a [`worker::Prepare`] implementation that
//!   reads the repositories' own state (the terminal UI's lives in its
//!   `radar/worker_fs.rs`), sends `Msg` values and drains `RepoAnswer`s.

use std::path::{Path, PathBuf};

use gitpoulpi_app::palette::vocab::radar::Act;

pub mod account;
pub mod model;
pub mod worker;

/// A repository with something to say (an error, a missing link).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub path: PathBuf,
    pub name: String,
    pub text: String,
    pub act: Option<(Act, Option<String>)>,
}

/// A repository's display name: the last path component (the whole path for
/// a root).
pub fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}
