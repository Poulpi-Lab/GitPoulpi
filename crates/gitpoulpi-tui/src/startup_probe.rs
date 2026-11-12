//! The one question GitPoulpi puts to the terminal at start-up.
//!
//! Four queries go out in a **single write** and their answers come back in a
//! **single bounded read** (at most [`PROBE_BUDGET`], on the controlling
//! terminal, before crossterm's reader exists):
//!
//! | Query | Asks | Used for |
//! |---|---|---|
//! | `CSI ? u` | the kitty keyboard protocol's flags | `kitty_keyboard` |
//! | `OSC 11 ; ?` | the background color | `ui.theme = system` |
//! | `CSI > 0 q` | XTVERSION: the terminal names itself | picture protocol for a terminal env cannot name (over SSH) |
//! | `CSI c` | primary device attributes | parameter `4` = sixel; also the sentinel |
//!
//! The device attributes query goes last and is the sentinel: *every*
//! terminal answers it, and in order, so once its reply is in, the others
//! that will come have come. A terminal that answers nothing (a multiplexer
//! that swallows the lot, a dumb pipe) costs the budget once, and everything
//! stays as before: legacy keys, the `COLORFGBG` theme guess, the
//! environment's verdict on pictures.
//!
//! This is the one place that reads input at start-up, and the exception to
//! "detection never reads stdin" (`terminal_image`'s module header) is
//! exactly what makes it safe: the rule is about *unbounded helper threads*
//! whose blocked `read` cannot be canceled and eats keystrokes. This is a
//! `poll` with a deadline on the calling thread, finished before the event
//! reader starts, so there is nothing left running to eat a key. A late reply
//! after the deadline reaches crossterm as an unknown escape sequence, which
//! it drops. The render thread is not running yet.
//!
//! Unix only. On Windows crossterm reads console input records, where a
//! reply is not even bytes; the result there is empty and behaviour is the
//! environment's, as before.

use std::sync::OnceLock;
use std::time::Duration;

/// How long start-up may wait for the terminal's answers, in total.
pub(crate) const PROBE_BUDGET: Duration = Duration::from_millis(100);

/// The four queries, device attributes last.
#[cfg_attr(not(unix), allow(dead_code))]
pub(crate) const QUERY: &[u8] = b"\x1b[?u\x1b]11;?\x1b\\\x1b[>0q\x1b[c";

/// What the terminal answered. Every field is "did not say" by default.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Reply {
    /// The kitty keyboard protocol's flags reply arrived.
    pub keyboard: bool,
    /// The OSC 11 background color, 8 bits per channel.
    pub background: Option<(u8, u8, u8)>,
    /// The XTVERSION text (`kitty(0.35.2)`, `WezTerm 2024...`).
    pub version: Option<String>,
    /// The primary device attributes parameters (`62;4;22` is `[62, 4, 22]`).
    pub attributes: Option<Vec<u32>>,
}

impl Reply {
    /// Parameter `4` of the device attributes: the terminal draws sixel.
    #[cfg(test)]
    pub(crate) fn sixel(&self) -> bool {
        self.attributes.as_ref().is_some_and(|p| p.contains(&4))
    }

    /// The device attributes as the text of the reply, `62;4;22`.
    pub(crate) fn attributes_text(&self) -> Option<String> {
        self.attributes
            .as_ref()
            .map(|p| p.iter().map(u32::to_string).collect::<Vec<_>>().join(";"))
    }
}

/// Whether the device attributes reply (the sentinel) is in `bytes`.
#[cfg_attr(not(unix), allow(dead_code))]
pub(crate) fn is_complete(bytes: &[u8]) -> bool {
    parse(bytes).attributes.is_some()
}

/// Reads what the terminal answered out of the bytes received so far. Pure,
/// and forgiving: noise typed meanwhile, replies in any order and replies cut
/// short are all fine.
pub(crate) fn parse(bytes: &[u8]) -> Reply {
    let mut reply = Reply::default();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != 0x1b {
            i += 1;
            continue;
        }
        match bytes.get(i + 1) {
            // CSI ? <digits;> u|c : flags reply or device attributes.
            Some(b'[') if bytes.get(i + 2) == Some(&b'?') => {
                let mut j = i + 3;
                while j < bytes.len() && (bytes[j].is_ascii_digit() || bytes[j] == b';') {
                    j += 1;
                }
                match bytes.get(j) {
                    Some(b'u') => reply.keyboard = true,
                    Some(b'c') => {
                        let params = String::from_utf8_lossy(&bytes[i + 3..j]);
                        reply.attributes = Some(
                            params
                                .split(';')
                                .filter(|p| !p.is_empty())
                                .filter_map(|p| p.parse().ok())
                                .collect(),
                        );
                    }
                    _ => {}
                }
                i = j.max(i + 1);
            }
            // OSC 11 ; <color> (BEL | ST). Only a finished string counts: a
            // color cut off halfway is a wrong color.
            Some(b']') if bytes[i..].starts_with(b"\x1b]11;") => {
                let Some((body_end, end)) = string_end(bytes, i + 5) else {
                    break;
                };
                if let Some(rgb) = crate::terminal_bg::parse_osc11(&bytes[i..body_end]) {
                    reply.background = Some(rgb);
                }
                i = end;
            }
            // DCS > | <text> ST : the XTVERSION reply, finished likewise.
            Some(b'P') if bytes[i..].starts_with(b"\x1bP>|") => {
                let Some((body_end, end)) = string_end(bytes, i + 4) else {
                    break;
                };
                let text = String::from_utf8_lossy(&bytes[i + 4..body_end]);
                let text = text.trim();
                if !text.is_empty() {
                    reply.version = Some(text.to_string());
                }
                i = end;
            }
            _ => i += 1,
        }
    }
    reply
}

/// Where the string that starts at `from` ends: the index of its terminator
/// (BEL or `ESC \`) and the index just past it. `None` while it is not
/// terminated.
fn string_end(bytes: &[u8], from: usize) -> Option<(usize, usize)> {
    let mut k = from;
    while k < bytes.len() {
        match bytes[k] {
            0x07 => return Some((k, k + 1)),
            0x1b if bytes.get(k + 1) == Some(&b'\\') => return Some((k, k + 2)),
            _ => k += 1,
        }
    }
    None
}

/// The picture protocol a terminal's XTVERSION name implies, for a terminal
/// the environment could not name (typically one reached over SSH, where
/// `TERM_PROGRAM` and friends are not forwarded). `None` when the name says
/// nothing about pictures; device attributes then decide.
pub(crate) fn protocol_for_version(version: &str) -> Option<crate::terminal_image::Protocol> {
    use crate::terminal_image::Protocol;
    let name = version.to_ascii_lowercase();
    let starts = |p: &str| name.starts_with(p);
    if starts("kitty") || starts("ghostty") {
        Some(Protocol::Kitty)
    } else if starts("wezterm") || starts("iterm2") || starts("mintty") {
        Some(Protocol::Iterm2)
    } else if starts("foot") {
        Some(Protocol::Sixel)
    } else {
        None
    }
}

static REPLY: OnceLock<Reply> = OnceLock::new();

/// What the start-up probe heard; empty when it has not run (a report built
/// by a test, the CLI) or the terminal said nothing.
pub(crate) fn reply() -> &'static Reply {
    static NONE: Reply = Reply {
        keyboard: false,
        background: None,
        version: None,
        attributes: None,
    };
    REPLY.get().unwrap_or(&NONE)
}

/// Runs the probe once and remembers the answers. Must run in raw mode,
/// before anything else reads the keyboard; later calls do nothing.
pub(crate) fn run() {
    REPLY.get_or_init(|| probe(PROBE_BUDGET));
}

#[cfg(unix)]
fn probe(budget: Duration) -> Reply {
    use std::fs::OpenOptions;
    use std::io::{Read, Write};
    use std::os::fd::AsRawFd;
    use std::time::Instant;

    let Ok(mut tty) = OpenOptions::new().read(true).write(true).open("/dev/tty") else {
        return Reply::default();
    };
    if tty.write_all(QUERY).and_then(|()| tty.flush()).is_err() {
        return Reply::default();
    }
    let deadline = Instant::now() + budget;
    let mut seen: Vec<u8> = Vec::new();
    while !is_complete(&seen) {
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
        let mut chunk = [0u8; 256];
        match tty.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => seen.extend_from_slice(&chunk[..n]),
        }
    }
    parse(&seen)
}

#[cfg(not(unix))]
fn probe(_budget: Duration) -> Reply {
    Reply::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_image::Protocol;

    #[test]
    fn one_write_carries_all_four_queries_with_the_attributes_last() {
        assert_eq!(QUERY, b"\x1b[?u\x1b]11;?\x1b\\\x1b[>0q\x1b[c");
        assert!(QUERY.ends_with(b"\x1b[c"));
    }

    #[test]
    fn a_full_reply_is_read_field_by_field() {
        let bytes =
            b"\x1b[?0u\x1b]11;rgb:1111/2222/3333\x1b\\\x1bP>|kitty(0.35.2)\x1b\\\x1b[?62;4;22c";
        let reply = parse(bytes);
        assert!(reply.keyboard);
        assert_eq!(reply.background, Some((17, 34, 51)));
        assert_eq!(reply.version.as_deref(), Some("kitty(0.35.2)"));
        assert_eq!(reply.attributes, Some(vec![62, 4, 22]));
        assert!(reply.sixel());
        assert_eq!(reply.attributes_text().as_deref(), Some("62;4;22"));
    }

    #[test]
    fn only_the_attributes_is_the_minimum_every_terminal_gives() {
        let reply = parse(b"\x1b[?1;2c");
        assert_eq!(reply.attributes, Some(vec![1, 2]));
        assert!(!reply.sixel());
        assert!(!reply.keyboard);
        assert_eq!(reply.version, None);
        assert_eq!(reply.background, None);
    }

    #[test]
    fn sixel_is_parameter_four_not_a_digit_inside_another() {
        assert!(parse(b"\x1b[?1;2;4c").sixel());
        assert!(!parse(b"\x1b[?64;22c").sixel());
        assert!(!parse(b"\x1b[?14c").sixel());
    }

    #[test]
    fn the_sentinel_is_the_attributes_reply_alone() {
        assert!(is_complete(b"\x1b]11;rgb:0/0/0\x07\x1b[?1;2c"));
        assert!(!is_complete(b"\x1b]11;rgb:0/0/0\x07"));
        assert!(!is_complete(b"\x1b[?1;2"));
        assert!(!is_complete(b"\x1b[?0u"));
        assert!(!is_complete(b""));
    }

    #[test]
    fn replies_in_any_order_with_noise_still_parse() {
        let reply = parse(b"ab\x1b[?1;2c\x1bP>|XTerm(388)\x07\x1b[?0u");
        assert_eq!(reply.attributes, Some(vec![1, 2]));
        assert_eq!(reply.version.as_deref(), Some("XTerm(388)"));
        assert!(reply.keyboard);
    }

    #[test]
    fn a_reply_cut_short_says_nothing_instead_of_something_wrong() {
        assert_eq!(parse(b"\x1b[?6"), Reply::default());
        assert_eq!(parse(b"\x1bP>|kit"), Reply::default());
        assert_eq!(parse(b"\x1b]11;rgb:ff"), Reply::default());
        assert_eq!(parse(b""), Reply::default());
    }

    #[test]
    fn version_names_map_to_a_picture_protocol_only_when_they_say_so() {
        assert_eq!(protocol_for_version("kitty(0.35.2)"), Some(Protocol::Kitty));
        assert_eq!(protocol_for_version("ghostty 1.0.1"), Some(Protocol::Kitty));
        assert_eq!(
            protocol_for_version("WezTerm 20240203-110809"),
            Some(Protocol::Iterm2)
        );
        assert_eq!(protocol_for_version("iTerm2 3.5.0"), Some(Protocol::Iterm2));
        assert_eq!(protocol_for_version("foot(1.28.0)"), Some(Protocol::Sixel));
        assert_eq!(protocol_for_version("XTerm(388)"), None);
        assert_eq!(protocol_for_version("tmux 3.8"), None);
        assert_eq!(protocol_for_version(""), None);
    }

    #[test]
    fn before_the_probe_ran_nothing_is_known() {
        assert_eq!(*reply(), Reply::default());
    }
}
