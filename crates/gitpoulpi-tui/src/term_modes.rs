//! Terminal input modes this UI turns on at start-up and must turn off on
//! every way out.
//!
//! Bracketed paste (`CSI ? 2004 h`): the terminal wraps a paste in
//! `ESC [ 200 ~ ... ESC [ 201 ~`, which crossterm reads as one
//! `Event::Paste`. Without it a multi-line paste arrives as keystrokes (an
//! Enter submits, letters fire shortcuts). Written as raw bytes, and on Unix
//! only: the Windows console input path has no paste brackets, and a
//! terminal that wrapped a paste there would type the markers. Windows
//! recognises a paste by its keys all being queued at once instead
//! (`paste_burst`).

use std::io::Write;

const PASTE_ON: &[u8] = b"\x1b[?2004h";
const PASTE_OFF: &[u8] = b"\x1b[?2004l";

/// Asks the terminal to bracket pastes.
pub(crate) fn enable_bracketed_paste<W: Write>(w: &mut W) {
    if cfg!(unix) {
        let _ = w.write_all(PASTE_ON);
        let _ = w.flush();
    }
}

/// Turns it off again: a no-op for a terminal that never had it on.
pub(crate) fn disable_bracketed_paste<W: Write>(w: &mut W) {
    if cfg!(unix) {
        let _ = w.write_all(PASTE_OFF);
        let _ = w.flush();
    }
}

/// Whether this platform requests bracketed paste (for the diagnostics).
pub(crate) fn bracketed_paste_requested() -> bool {
    cfg!(unix)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn the_enable_and_disable_sequences_are_the_dec_private_mode_2004() {
        let mut out = Vec::new();
        enable_bracketed_paste(&mut out);
        disable_bracketed_paste(&mut out);
        assert_eq!(out, b"\x1b[?2004h\x1b[?2004l");
    }
}
