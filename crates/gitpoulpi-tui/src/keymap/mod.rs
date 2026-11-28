//! The keymap: one registry saying which key triggers which action in
//! which context. The registry, the chords and the conflict analysis are
//! shared by every frontend and live in `gitpoulpi_app::keymap`; this module
//! re-exports them at their old paths, adds the terminal adapter ([`term`])
//! and the two entry points that need the terminal window's own facts.

// Registry metadata (labels, flags, `Ctx` helpers) is not all consumed.
#![allow(dead_code)]

pub(crate) mod term;

#[allow(unused_imports)]
pub(crate) use gitpoulpi_app::keymap::*;
pub(crate) use gitpoulpi_app::keymap::{action, chord, conflict};
pub(crate) use term::normalize_event;

/// Builds the keymap from `config` and installs it; returns the issues. The
/// active repository's own sequences, once trusted, are part of the registry.
pub(crate) fn install_from_config(config: &gitpoulpi_config::Config) -> Vec<KeymapIssue> {
    crate::app_hooks::ensure();
    gitpoulpi_app::keymap::install_from_config_with(
        config,
        &install_id(),
        crate::sequence_run::repo::trusted_defs(),
    )
}

/// The installation id `installation`-scope sequences are matched against.
/// Tests use none (reading the identity seeds the real `config.toml`).
pub(crate) fn install_id() -> String {
    #[cfg(test)]
    {
        String::new()
    }
    #[cfg(not(test))]
    {
        gitpoulpi_app::keymap::install_id()
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod drift;
