//! The kitty keyboard protocol, opted into when the terminal speaks it.
//!
//! Legacy terminals cannot tell `Ctrl+,`, `Ctrl+Shift+N`, `Ctrl+Enter`
//! or `Ctrl+<digit>` from other keys (the keymap's `reach` table lists
//! them). With the protocol's *disambiguate escape codes* flag those keys
//! arrive as `CSI <code> ; <mods> u`, which crossterm decodes into the
//! key and modifiers that were actually pressed.
//!
//! # Detecting it without delaying start-up
//!
//! The protocol is queried with `CSI ? u`, one of the questions of the
//! merged start-up probe (`startup_probe`, which has the whole story: one
//! write, one bounded read, the primary device attributes query as the
//! sentinel that every terminal answers). A terminal that knows the protocol
//! answers `CSI ? <flags> u` before the attributes reply; one that does not
//! costs no wait, and one that answers nothing keeps the legacy keys after
//! the probe's budget, exactly as before. crossterm's own
//! `supports_keyboard_enhancement` waits up to two seconds for a terminal
//! that answers neither, which would be a visible start-up stall.
//!
//! # Lifetime
//!
//! `enable` pushes the flags, `disable` pops them; `disable` is
//! idempotent and is called from every way out: the normal exit, the
//! error exit (`exit_log::restore_terminal`) and the panic hook. This app
//! never suspends itself (Ctrl+Z is a key like any other in raw mode) and
//! never hands the terminal to a child process (the editor is built in),
//! so there is no other place the keyboard mode must be released.

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags};
use gitpoulpi_config::KittyKeyboard;

/// What is pushed: disambiguate escape codes. `REPORT_ALTERNATE_KEYS` is
/// deliberately left out - it changes which character a shifted key
/// reports and nothing in the keymap needs it.
pub(crate) const FLAGS: KeyboardEnhancementFlags =
    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES;

static ACTIVE: AtomicBool = AtomicBool::new(false);

/// Whether the protocol is on right now (what Settings > Keyboard
/// Shortcuts uses to decide which keys the terminal can send).
pub(crate) fn is_active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

/// Pushes [`FLAGS`] when `mode` allows it and the terminal supports it;
/// returns whether the protocol is now on.
pub(crate) fn enable(mode: KittyKeyboard, out: &mut impl Write) -> bool {
    if mode == KittyKeyboard::Off || !cfg!(unix) || !crate::startup_probe::reply().keyboard {
        return false;
    }
    let pushed = crossterm::execute!(out, crossterm::event::PushKeyboardEnhancementFlags(FLAGS));
    if pushed.is_ok() {
        ACTIVE.store(true, Ordering::Relaxed);
    }
    pushed.is_ok()
}

/// Pops what [`enable`] pushed; a no-op when nothing was pushed, so every
/// teardown path can call it unconditionally.
pub(crate) fn disable(out: &mut impl Write) {
    if ACTIVE.swap(false, Ordering::Relaxed) {
        let _ = crossterm::execute!(out, PopKeyboardEnhancementFlags);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_never_pushes() {
        let mut out = Vec::new();
        assert!(!enable(KittyKeyboard::Off, &mut out));
        assert!(out.is_empty());
        assert!(!is_active());
        disable(&mut out);
        assert!(out.is_empty(), "nothing to pop when nothing was pushed");
    }

    #[test]
    fn a_probe_that_heard_nothing_pushes_nothing() {
        // The probe never ran in a unit test: no answer, legacy keys.
        let mut out = Vec::new();
        assert!(!enable(KittyKeyboard::Auto, &mut out));
        assert!(out.is_empty());
    }
}
