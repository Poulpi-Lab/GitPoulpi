//! User-defined action sequences: the run side.
//!
//! Only the registry (which slug owns which keymap slot) and a `start` stub
//! exist so far; the runner arrives with its own slice.

pub(crate) mod registry;

/// Starts the sequence in `slot`. A stub: it only says so on the status line.
pub(crate) fn start(slot: u16, last_message: &mut Option<String>) {
    let name = registry::info(slot)
        .map(|i| i.name)
        .unwrap_or_else(|| format!("#{slot}"));
    // Internal text of the stub; the runner slice replaces it.
    *last_message = Some(format!("{name}: sequences are not wired yet")); // i18n-english
}
