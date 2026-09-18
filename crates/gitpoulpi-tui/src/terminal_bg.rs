//! Terminal background detection for `ui.theme = system`, once at start-up.
//!
//! Asks with OSC 11 (`ESC ] 11 ; ? ST`) followed by the primary device
//! attributes query as a sentinel: every terminal answers the latter, so a
//! terminal that does not know OSC 11 costs no wait and the reply (if any)
//! is fully read, never left for the event loop. A hard deadline
//! ([`PROBE_BUDGET`]) bounds a terminal that answers neither. Unix only
//! (it reads `/dev/tty`); on Windows, or without an answer, `COLORFGBG`,
//! then Dark. Re-detection while running is not done.
//!
//! A hand-written reader rather than the `terminal-colorsaurus` crate: the
//! kitty keyboard probe already needed the same bounded `/dev/tty` read,
//! a dependency would add a second implementation of it, and ConPTY (which
//! is why the crate has a Windows path at all) is simply skipped here.

#[cfg(unix)]
use std::time::Duration;

/// How long start-up may wait for the terminal's answer, in total.
#[cfg(unix)]
pub(crate) const PROBE_BUDGET: Duration = Duration::from_millis(100);

/// OSC 11 background query (ST-terminated), then the DA sentinel.
#[cfg(unix)]
const QUERY: &[u8] = b"\x1b]11;?\x1b\\\x1b[c";

// Windows never sends the query (see `detect_is_light`): the parsing
// helpers below only exist on unix, and in tests everywhere.

/// Parses `rgb:R/G/B` (1 to 4 hex digits per channel), `rgba:R/G/B/A` or
/// `#rrggbb` out of an OSC 11 reply, scaled to 8 bits per channel.
#[cfg(any(unix, test))]
pub(crate) fn parse_osc11(bytes: &[u8]) -> Option<(u8, u8, u8)> {
    let text = String::from_utf8_lossy(bytes);
    let start = text.find("\x1b]11;")? + "\x1b]11;".len();
    let body = &text[start..];
    let end = body.find(['\x07', '\x1b']).unwrap_or(body.len());
    let body = body[..end].trim();
    if let Some(hex) = body.strip_prefix('#') {
        let n = hex.len() / 3;
        if hex.len() % 3 != 0 || !(1..=4).contains(&n) {
            return None;
        }
        return Some((
            scale(&hex[..n])?,
            scale(&hex[n..2 * n])?,
            scale(&hex[2 * n..])?,
        ));
    }
    let rest = body
        .strip_prefix("rgb:")
        .or_else(|| body.strip_prefix("rgba:"))?;
    let mut parts = rest.split('/');
    let (r, g, b) = (parts.next()?, parts.next()?, parts.next()?);
    Some((scale(r)?, scale(g)?, scale(b)?))
}

/// One hex channel of 1 to 4 digits, scaled to 0..=255.
#[cfg(any(unix, test))]
fn scale(digits: &str) -> Option<u8> {
    if digits.is_empty() || digits.len() > 4 || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let v = u32::from_str_radix(digits, 16).ok()?;
    let max = (1u32 << (4 * digits.len())) - 1;
    Some(((v * 255 + max / 2) / max) as u8)
}

/// Perceived brightness above the middle: the background is light.
#[cfg(any(unix, test))]
pub(crate) fn is_light(rgb: (u8, u8, u8)) -> bool {
    let (r, g, b) = (f32::from(rgb.0), f32::from(rgb.1), f32::from(rgb.2));
    0.2126 * r + 0.7152 * g + 0.0722 * b > 127.5
}

/// `COLORFGBG` (`fg;bg` or `fg;default;bg`): backgrounds 7 and 15 are light.
pub(crate) fn colorfgbg_is_light(value: &str) -> Option<bool> {
    let bg: u8 = value.rsplit(';').next()?.trim().parse().ok()?;
    (bg <= 15).then_some(matches!(bg, 7 | 15))
}

/// Light or dark, from what the terminal says; `None` when unknown.
pub(crate) fn detect_is_light() -> Option<bool> {
    #[cfg(unix)]
    if let Some(rgb) = probe(PROBE_BUDGET) {
        return Some(is_light(rgb));
    }
    std::env::var("COLORFGBG")
        .ok()
        .and_then(|v| colorfgbg_is_light(&v))
}

/// Whether the bytes read so far contain the DA reply (`ESC [ ? ... c`).
#[cfg(unix)]
fn has_attributes_reply(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == 0x1b && bytes[i + 1] == b'[' && bytes[i + 2] == b'?' {
            let mut j = i + 3;
            while j < bytes.len() && (bytes[j].is_ascii_digit() || bytes[j] == b';') {
                j += 1;
            }
            if bytes.get(j) == Some(&b'c') {
                return true;
            }
            i = j;
        } else {
            i += 1;
        }
    }
    false
}

/// Asks the controlling terminal; `None` on silence, an error or a bad reply.
/// Must run in raw mode, before anything else reads the keyboard.
#[cfg(unix)]
pub(crate) fn probe(budget: Duration) -> Option<(u8, u8, u8)> {
    use std::fs::OpenOptions;
    use std::io::{Read, Write};
    use std::os::fd::AsRawFd;
    use std::time::Instant;

    let mut tty = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .ok()?;
    tty.write_all(QUERY).and_then(|()| tty.flush()).ok()?;
    let deadline = Instant::now() + budget;
    let mut seen: Vec<u8> = Vec::new();
    // The sentinel closes the read: the color reply, when there is one,
    // precedes it. A terminal that never sends the sentinel is bounded by
    // the deadline, and whatever it said by then is still parsed.
    while !has_attributes_reply(&seen) {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            break;
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
            break;
        }
        if ready == 0 {
            break;
        }
        let mut chunk = [0u8; 128];
        match tty.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => seen.extend_from_slice(&chunk[..n]),
        }
    }
    parse_osc11(&seen)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn osc11_replies_parse_in_every_form() {
        assert_eq!(
            parse_osc11(b"\x1b]11;rgb:ffff/ffff/ffff\x1b\\"),
            Some((255, 255, 255))
        );
        assert_eq!(
            parse_osc11(b"\x1b]11;rgb:0000/0000/0000\x07"),
            Some((0, 0, 0))
        );
        assert_eq!(
            parse_osc11(b"\x1b]11;rgb:ff/80/00\x07"),
            Some((255, 128, 0))
        );
        assert_eq!(parse_osc11(b"\x1b]11;rgb:f/0/8\x07"), Some((255, 0, 136)));
        assert_eq!(
            parse_osc11(b"\x1b]11;rgba:2e2e/3434/4040/ffff\x1b\\"),
            Some((46, 52, 64))
        );
        assert_eq!(parse_osc11(b"\x1b]11;#2e3440\x07"), Some((46, 52, 64)));
        // Noise before the reply, and the DA reply after it.
        assert_eq!(
            parse_osc11(b"x\x1b]11;rgb:1111/2222/3333\x1b\\\x1b[?62;c"),
            Some((17, 34, 51))
        );
        assert_eq!(parse_osc11(b"\x1b[?62;c"), None);
        assert_eq!(parse_osc11(b"\x1b]11;rgb:zz/00/00\x07"), None);
        assert_eq!(parse_osc11(b"\x1b]11;rgb:ff/00\x07"), None);
        assert_eq!(parse_osc11(b""), None);
    }

    #[test]
    fn luminance_splits_light_from_dark() {
        assert!(is_light((255, 255, 255)));
        assert!(is_light((253, 246, 227)));
        assert!(!is_light((0, 0, 0)));
        assert!(!is_light((46, 52, 64)));
    }

    #[test]
    fn colorfgbg_reads_the_background_slot() {
        assert_eq!(colorfgbg_is_light("0;15"), Some(true));
        assert_eq!(colorfgbg_is_light("0;default;15"), Some(true));
        assert_eq!(colorfgbg_is_light("15;0"), Some(false));
        assert_eq!(colorfgbg_is_light("0;7"), Some(true));
        assert_eq!(colorfgbg_is_light("7;8"), Some(false));
        assert_eq!(colorfgbg_is_light("garbage"), None);
        assert_eq!(colorfgbg_is_light(""), None);
    }

    #[cfg(unix)]
    #[test]
    fn the_sentinel_is_recognized() {
        assert!(has_attributes_reply(b"\x1b]11;rgb:0/0/0\x07\x1b[?1;2c"));
        assert!(!has_attributes_reply(b"\x1b]11;rgb:0/0/0\x07"));
        assert!(!has_attributes_reply(b"\x1b[?1;2"));
    }
}
