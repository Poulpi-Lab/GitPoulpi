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
//! The protocol is queried with `CSI ? u` followed by the primary device
//! attributes query `CSI c` as a sentinel: a terminal that knows the
//! protocol answers `CSI ? <flags> u` first, and *every* terminal answers
//! the attributes query, so the absence of the first answer is known as
//! soon as the second arrives. crossterm's own
//! `supports_keyboard_enhancement` does the same dance but waits up to
//! two seconds for a terminal that answers neither (a multiplexer that
//! swallows both, a dumb pipe), which would be a visible start-up stall.
//! This module reads the reply itself, off the controlling terminal,
//! with a hard deadline of [`PROBE_BUDGET`]; anything that does not
//! answer in time keeps the legacy keys, exactly as before. It runs
//! before crossterm's event reader exists, so no one else is reading the
//! tty, and a late reply that lands after the deadline is an unknown
//! escape sequence that crossterm drops.
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
use std::time::Duration;

use crossterm::event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags};
use gitpoulpi_config::KittyKeyboard;

/// How long start-up may wait for the terminal's answer, in total.
pub(crate) const PROBE_BUDGET: Duration = Duration::from_millis(100);

/// What is pushed: disambiguate escape codes. `REPORT_ALTERNATE_KEYS` is
/// deliberately left out - it changes which character a shifted key
/// reports and nothing in the keymap needs it.
pub(crate) const FLAGS: KeyboardEnhancementFlags =
    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES;

/// The flags query, then the attributes query as its sentinel.
#[cfg_attr(not(unix), allow(dead_code))]
pub(crate) const QUERY: &[u8] = b"\x1b[?u\x1b[c";

static ACTIVE: AtomicBool = AtomicBool::new(false);

/// Whether the protocol is on right now (what Settings > Keyboard
/// Shortcuts uses to decide which keys the terminal can send).
pub(crate) fn is_active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

/// What the terminal said to [`QUERY`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(unix), allow(dead_code))]
pub(crate) enum Reply {
    /// Not enough bytes yet to know.
    Pending,
    /// The attributes reply arrived without a flags reply.
    Unsupported,
    /// A flags reply arrived (the attributes reply may still follow).
    Supported,
}

/// Classifies the bytes read so far. Pure.
#[cfg_attr(not(unix), allow(dead_code))]
pub(crate) fn classify(bytes: &[u8]) -> Reply {
    let mut flags = false;
    let mut attributes = false;
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == 0x1b && bytes[i + 1] == b'[' && bytes[i + 2] == b'?' {
            let mut j = i + 3;
            while j < bytes.len() && (bytes[j].is_ascii_digit() || bytes[j] == b';') {
                j += 1;
            }
            match bytes.get(j) {
                Some(b'u') => flags = true,
                Some(b'c') => attributes = true,
                _ => {}
            }
            i = j;
        } else {
            i += 1;
        }
    }
    match (flags, attributes) {
        (true, _) => Reply::Supported,
        (false, true) => Reply::Unsupported,
        (false, false) => Reply::Pending,
    }
}

/// Asks the controlling terminal; `false` on silence, an error, or a
/// platform without a tty to read (Windows' console reports the keys
/// itself).
#[cfg(unix)]
pub(crate) fn probe(budget: Duration) -> bool {
    use std::fs::OpenOptions;
    use std::io::Read;
    use std::os::fd::AsRawFd;
    use std::time::Instant;

    let Ok(mut tty) = OpenOptions::new().read(true).write(true).open("/dev/tty") else {
        return false;
    };
    if tty.write_all(QUERY).and_then(|()| tty.flush()).is_err() {
        return false;
    }
    let deadline = Instant::now() + budget;
    let mut seen: Vec<u8> = Vec::new();
    loop {
        match classify(&seen) {
            // The flags reply is the answer; the attributes reply that
            // follows is read too when it is already here, so it does
            // not reach the event loop, but never waited for.
            Reply::Supported => return true,
            Reply::Unsupported => return false,
            Reply::Pending => {}
        }
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return false;
        }
        let mut pfd = libc::pollfd {
            fd: tty.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let ms = left.as_millis().clamp(1, i32::MAX as u128) as i32;
        // SAFETY: one valid pollfd, count 1.
        let ready = unsafe { libc::poll(&mut pfd, 1, ms) };
        if ready < 0 {
            if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return false;
        }
        if ready == 0 {
            return false;
        }
        let mut chunk = [0u8; 64];
        match tty.read(&mut chunk) {
            Ok(0) | Err(_) => return false,
            Ok(n) => seen.extend_from_slice(&chunk[..n]),
        }
    }
}

#[cfg(not(unix))]
pub(crate) fn probe(_budget: Duration) -> bool {
    false
}

/// Pushes [`FLAGS`] when `mode` allows it and the terminal supports it;
/// returns whether the protocol is now on.
pub(crate) fn enable(mode: KittyKeyboard, out: &mut impl Write) -> bool {
    if mode == KittyKeyboard::Off || !cfg!(unix) || !probe(PROBE_BUDGET) {
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
    fn the_reply_is_classified_by_what_has_arrived() {
        assert_eq!(classify(b""), Reply::Pending);
        assert_eq!(classify(b"\x1b[?"), Reply::Pending);
        assert_eq!(classify(b"\x1b[?1;2c"), Reply::Unsupported);
        assert_eq!(classify(b"\x1b[?0u"), Reply::Supported);
        assert_eq!(classify(b"\x1b[?0u\x1b[?62;c"), Reply::Supported);
        assert_eq!(classify(b"\x1b[?31u\x1b[?1;2c"), Reply::Supported);
        // Unrelated input typed meanwhile does not confuse it.
        assert_eq!(classify(b"ab\x1b[?1;2c"), Reply::Unsupported);
    }

    #[test]
    fn off_never_asks_and_never_pushes() {
        let mut out = Vec::new();
        assert!(!enable(KittyKeyboard::Off, &mut out));
        assert!(out.is_empty());
        assert!(!is_active());
        disable(&mut out);
        assert!(out.is_empty(), "nothing to pop when nothing was pushed");
    }

    #[test]
    fn the_query_pairs_the_flags_question_with_the_attributes_sentinel() {
        assert_eq!(QUERY, b"\x1b[?u\x1b[c");
    }
}
