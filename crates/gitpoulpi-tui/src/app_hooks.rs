//! What the terminal UI installs into the shared `gitpoulpi-app` registries:
//! how keys reach it, how the sequence registry and keymap are rebuilt from
//! the config, and the palette's generated rows and process-state probes.
//!
//! [`ensure`] is idempotent and cheap; every entry point into the shared
//! registries that needs a hook calls it first, so a unit test and the real
//! window see the same wiring.

use std::sync::Once;

static ONCE: Once = Once::new();

pub(crate) fn ensure() {
    ONCE.call_once(|| {
        gitpoulpi_app::keymap::conflict::install_transport(crate::keymap::term::current_transport);
        gitpoulpi_app::sequences::install_rebuild(rebuild);
        gitpoulpi_app::palette::install_families(crate::palette::tui_families());
    });
}

/// Rebuilds the sequence registry and the keymap from the config on disk (a
/// live window: the repository's trust or file changed). Unit tests never
/// read the machine's config.
fn rebuild() {
    #[cfg(not(test))]
    {
        if let Ok(settings) = gitpoulpi_config::Settings::resolve_ambient() {
            let own = crate::launcher_sync::own_config(settings.config());
            crate::keymap::install_from_config(&own);
        }
    }
}
