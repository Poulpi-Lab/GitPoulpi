//! The process-wide registry of sequence slugs and their keymap slots. The
//! registry itself lives in `gitpoulpi_app::sequences`, shared by every
//! frontend; this module re-exports it and adds what is the terminal window's
//! own: the active repository's trusted sequences (`repo`) and the hooks.
//!
//! Unit tests give each thread a registry of its own, so a test that
//! refreshes it never disturbs the ones running beside it.

#[cfg(test)]
pub(crate) use gitpoulpi_app::sequences::test_def;
#[allow(unused_imports)]
pub(crate) use gitpoulpi_app::sequences::{
    config_id, generation, info, note_config_loaded, slot_of, visible, Info, Refused,
    MAX_SEQUENCE_SLOTS,
};
#[cfg(test)]
use gitpoulpi_config::sequences::SequenceDef;
use gitpoulpi_config::Config;

/// Brings the registry in line with the sequences visible to `install_id`:
/// the user's own, the machine's, and the active repository's file when the
/// user trusted it exactly as it is (`repo`).
pub(crate) fn refresh(config: &Config, install_id: &str) -> Refused {
    #[cfg(test)]
    gitpoulpi_app::sequences::isolate_thread();
    gitpoulpi_app::sequences::refresh_with(config, install_id, super::repo::trusted_defs())
}

/// Rebuilds the registry and the keymap from the config on disk (a live
/// window: the repository's trust or file changed). Unit tests never read
/// the machine's config.
pub(crate) fn rebuild() {
    crate::app_hooks::ensure();
    gitpoulpi_app::sequences::rebuild();
}

/// [`refresh`] over an explicit list.
#[cfg(test)]
pub(crate) fn refresh_defs(defs: &[SequenceDef]) -> Refused {
    #[cfg(test)]
    gitpoulpi_app::sequences::isolate_thread();
    gitpoulpi_app::sequences::refresh_defs(defs)
}
