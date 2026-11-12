//! What tmux does with the sequences this program writes, measured with raw
//! escape sequences (tmux 3.8, November 2026), and what we do about it.
//!
//! - **OSC 22 (pointer shape), kitty graphics (APC `G`), OSC 52** are dropped by
//!   tmux when written bare, and reach the outer terminal when wrapped in a DCS
//!   passthrough (`ESC P tmux ; <sequence, every ESC doubled> ESC \`) with
//!   `allow-passthrough on`. With the option off tmux discards the wrapper, so
//!   wrapping is harmless. The pointer is a window-global property, so a pane
//!   being behind tmux does not make it wrong.
//! - **Sixel** is drawn by tmux itself, re-encoded for a client that supports
//!   it: no wrapper, and the client's capability list says so
//!   (`#{client_termfeatures}` contains `sixel`). Note that tmux's own reply to
//!   a device-attributes query always claims sixel (`CSI ? 1;2;4 c`) whatever
//!   the client is, so that reply proves nothing; the client features do.
//! - iTerm2 inline images (OSC 1337) pass through but tmux cannot position
//!   them, so they stay off inside tmux.
//!
//! Only tmux is handled here. zellij, screen and herdr have no passthrough.

use std::time::Duration;

use super::detect::inside_herdr;

/// The synthetic "variable" detection asks for when it needs the sixel verdict
/// of tmux's client. [`super::detect()`] answers it with
/// [`query_client_features`]; tests answer it from a table, so detection stays
/// a pure function of its environment.
pub(super) const CLIENT_FEATURES_KEY: &str = "\0tmux:client_termfeatures";

/// How long to wait for `tmux display-message`: it answers in a few
/// milliseconds, and a hung server must not hold up the start-up.
const QUERY_BUDGET: Duration = Duration::from_millis(400);

/// Wraps `seq` in a tmux DCS passthrough: the escapes inside are doubled and
/// the whole is a `DCS tmux ;` ... `ST` string.
pub(crate) fn wrap_tmux(seq: &str) -> String {
    let mut out = String::with_capacity(seq.len() + 16);
    out.push_str("\x1bPtmux;");
    for ch in seq.chars() {
        if ch == '\x1b' {
            out.push('\x1b');
        }
        out.push(ch);
    }
    out.push_str("\x1b\\");
    out
}

/// Whether tmux is the **only** multiplexer around this process: `TMUX` is
/// set and nothing else (zellij, screen, herdr) claims the screen. Inside a
/// tmux inside another multiplexer nothing can be promised, so that stays
/// text and no pointer.
pub(super) fn tmux_only<F: Fn(&str) -> Option<String>>(get: &F) -> bool {
    let set = |name: &str| get(name).is_some_and(|v| !v.is_empty());
    set("TMUX") && !set("ZELLIJ") && !set("STY") && !inside_herdr(get)
}

/// Whether tmux's client advertises sixel in its terminal features
/// (`#{client_termfeatures}`, a comma separated list).
pub(super) fn features_include_sixel(features: &str) -> bool {
    features.split(',').any(|f| f.trim() == "sixel")
}

/// Whether the terminal behind tmux takes sixel (tmux re-encodes it).
pub(super) fn tmux_client_has_sixel<F: Fn(&str) -> Option<String>>(get: &F) -> bool {
    tmux_only(get)
        && get(CLIENT_FEATURES_KEY).is_some_and(|features| features_include_sixel(&features))
}

/// Asks tmux for its client's terminal features. `None` when `tmux` is not
/// there, errors, or does not answer within [`QUERY_BUDGET`].
pub(super) fn query_client_features() -> Option<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let spawn = std::thread::Builder::new()
        .name("tmux-features".into())
        .spawn(move || {
            let out = std::process::Command::new("tmux")
                .args(["display-message", "-p", "#{client_termfeatures}"])
                .stdin(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .output();
            let _ = tx.send(out);
        });
    if spawn.is_err() {
        return None;
    }
    let out = rx.recv_timeout(QUERY_BUDGET).ok()?.ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Whether sequences written to the terminal go through a tmux passthrough.
/// Read from the environment once; unit tests always get `false`, so exact
/// bytes never depend on whoever runs them (a developer inside tmux).
#[cfg(not(test))]
pub(crate) fn wrapping() -> bool {
    static WRAP: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *WRAP.get_or_init(|| tmux_only(&|name: &str| std::env::var(name).ok()))
}

#[cfg(test)]
pub(crate) fn wrapping() -> bool {
    false
}

/// `seq` as it must be written: wrapped when [`wrapping`].
pub(crate) fn passthrough(seq: String) -> String {
    if wrapping() {
        wrap_tmux(&seq)
    } else {
        seq
    }
}

/// [`wrap_tmux`] applied to each string of a run of `ST`-terminated strings
/// (a chunked kitty upload): tmux gets many small passthroughs, never one of
/// several hundred kilobytes.
pub(crate) fn wrap_each_string(seq: &str) -> String {
    let mut out = String::with_capacity(seq.len() + seq.len() / 64);
    for piece in seq.split_inclusive("\x1b\\") {
        out.push_str(&wrap_tmux(piece));
    }
    out
}

/// [`passthrough`] for a run of strings, see [`wrap_each_string`].
pub(crate) fn passthrough_each(seq: String) -> String {
    if wrapping() {
        wrap_each_string(&seq)
    } else {
        seq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
        let pairs = pairs.to_vec();
        move |name: &str| {
            pairs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| (*v).to_string())
        }
    }

    #[test]
    fn the_wrapper_doubles_every_escape_and_is_a_dcs_string() {
        assert_eq!(
            wrap_tmux("\x1b]22;pointer\x1b\\"),
            "\x1bPtmux;\x1b\x1b]22;pointer\x1b\x1b\\\x1b\\"
        );
        assert_eq!(wrap_tmux("plain"), "\x1bPtmux;plain\x1b\\");
        assert_eq!(wrap_tmux(""), "\x1bPtmux;\x1b\\");
    }

    #[test]
    fn the_wrapper_keeps_a_kitty_upload_intact() {
        let apc = "\x1b_Ga=T,f=100,m=0;aGVsbG8=\x1b\\";
        let wrapped = wrap_tmux(apc);
        // Undoing the doubling gives the original back.
        let inner = wrapped
            .strip_prefix("\x1bPtmux;")
            .and_then(|s| s.strip_suffix("\x1b\\"))
            .unwrap()
            .replace("\x1b\x1b", "\x1b");
        assert_eq!(inner, apc);
    }

    #[test]
    fn a_chunked_upload_is_wrapped_one_string_at_a_time() {
        let upload = "\x1b_Ga=T,m=1;AAAA\x1b\\\x1b_Gm=0;BBBB\x1b\\";
        assert_eq!(
            wrap_each_string(upload),
            "\x1bPtmux;\x1b\x1b_Ga=T,m=1;AAAA\x1b\x1b\\\x1b\\\x1bPtmux;\x1b\x1b_Gm=0;BBBB\x1b\x1b\\\x1b\\"
        );
        assert_eq!(wrap_each_string(""), "");
    }

    #[test]
    fn tmux_alone_counts_and_a_second_multiplexer_does_not() {
        assert!(tmux_only(&env(&[("TMUX", "/tmp/tmux-1000/default,1,0")])));
        assert!(!tmux_only(&env(&[])));
        assert!(!tmux_only(&env(&[("TMUX", "")])));
        assert!(!tmux_only(&env(&[("TMUX", "x"), ("ZELLIJ", "0")])));
        assert!(!tmux_only(&env(&[("TMUX", "x"), ("STY", "1.pts")])));
        assert!(!tmux_only(&env(&[("TMUX", "x"), ("HERDR_ENV", "1")])));
    }

    #[test]
    fn sixel_is_read_from_the_feature_list_by_name() {
        // Measured: tmux 3.8 in foot 1.28 and in kitty 0.49.
        assert!(features_include_sixel(
            "256,bpaste,ccolour,clipboard,hyperlinks,cstyle,extkeys,focus,mouse,osc7,rectfill,RGB,sixel,strikethrough,sync,title,usstyle"
        ));
        assert!(!features_include_sixel(
            "bpaste,ccolour,clipboard,cstyle,focus,RGB,sync,title"
        ));
        assert!(!features_include_sixel("nosixel,sixelish"));
        assert!(!features_include_sixel(""));
    }

    #[test]
    fn the_client_verdict_needs_tmux_and_its_answer() {
        let features = "RGB,sixel";
        let with = |pairs: &[(&'static str, &'static str)]| {
            let pairs = pairs.to_vec();
            move |name: &str| {
                if name == CLIENT_FEATURES_KEY {
                    Some(features.to_string())
                } else {
                    pairs
                        .iter()
                        .find(|(k, _)| *k == name)
                        .map(|(_, v)| (*v).to_string())
                }
            }
        };
        assert!(tmux_client_has_sixel(&with(&[("TMUX", "x")])));
        // The same answer outside tmux, or behind a second multiplexer, is
        // not a verdict about this process.
        assert!(!tmux_client_has_sixel(&with(&[])));
        assert!(!tmux_client_has_sixel(&with(&[
            ("TMUX", "x"),
            ("STY", "1")
        ])));
    }

    #[test]
    fn passthrough_is_off_unless_asked_so_exact_bytes_stay_testable() {
        assert!(!wrapping());
        assert_eq!(passthrough("abc".into()), "abc");
    }
}
