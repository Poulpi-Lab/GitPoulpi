//! Where a copy goes: the policy of the clipboard, as pure functions.
//!
//! Two clipboards can be reached from here. The **system** clipboard of the
//! machine GitPoulpi runs on (arboard), and the clipboard of the **terminal**
//! the user looks at, asked through OSC 52. They are the same machine only
//! when nothing remote is involved.
//!
//! - **Over SSH, or with no display** (a Linux console, a container, a
//!   server): the system clipboard is the remote one or does not exist, so
//!   the copy goes to the terminal through OSC 52 and nothing else.
//! - **Locally**: the system clipboard, plus OSC 52 when the terminal is known
//!   to honour it (harmless there, and it also reaches a terminal window that
//!   sits on another seat). When the system clipboard fails, OSC 52 is the
//!   fallback whatever the terminal is, unless it cannot carry one.
//! - **Inside tmux** the sequence is wrapped in a DCS passthrough as well as
//!   sent bare: bare works with `set-clipboard on`, wrapped with
//!   `allow-passthrough on`, and a tmux with neither discards what it gets.
//! - **"Copied" is only said when the system clipboard took it.** What was
//!   sent to the terminal cannot be confirmed from here, so the sentence says
//!   where it went ([`Delivery::Terminal`]).

use std::io::Write;

use crate::terminal_image::passthrough::{tmux_only, wrap_tmux};

/// The facts of the environment the policy reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Surroundings {
    /// An SSH session (`SSH_CONNECTION` and friends).
    pub remote: bool,
    /// A local display exists for the system clipboard to live on.
    pub display: bool,
    /// The terminal is known to implement OSC 52.
    pub osc52_known: bool,
    /// The terminal could carry an OSC 52 at all (not the Linux console, not
    /// `TERM=dumb`, not GNU screen alone, which drops it).
    pub osc52_possible: bool,
    /// tmux is the only multiplexer around.
    pub tmux: bool,
}

impl Surroundings {
    pub(crate) fn from_env<F: Fn(&str) -> Option<String>>(get: &F) -> Self {
        let set = |name: &str| get(name).is_some_and(|v| !v.is_empty());
        let term = get("TERM").unwrap_or_default().to_ascii_lowercase();
        let program = get("TERM_PROGRAM").unwrap_or_default().to_ascii_lowercase();
        let remote = ["SSH_CONNECTION", "SSH_CLIENT", "SSH_TTY"]
            .iter()
            .any(|v| set(v));
        let display = if cfg!(any(windows, target_os = "macos")) {
            true
        } else {
            set("WAYLAND_DISPLAY") || set("DISPLAY")
        };
        let osc52_known = set("KITTY_WINDOW_ID")
            || set("WT_SESSION")
            || set("WEZTERM_EXECUTABLE")
            || set("GHOSTTY_RESOURCES_DIR")
            || [
                "wezterm",
                "ghostty",
                "rio",
                "tabby",
                "iterm.app",
                "alacritty",
            ]
            .iter()
            .any(|p| program.contains(p))
            || ["xterm-kitty", "xterm-ghostty", "alacritty"]
                .iter()
                .any(|t| term == *t)
            || term.starts_with("foot");
        let tmux = tmux_only(get);
        let screen_alone = set("STY") && !set("TMUX");
        let osc52_possible = term != "linux" && term != "dumb" && !screen_alone;
        Surroundings {
            remote,
            display,
            osc52_known,
            osc52_possible,
            tmux,
        }
    }
}

/// What to try for one copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Plan {
    /// Try the system clipboard first.
    pub system: bool,
    /// Write OSC 52 whatever the system clipboard says.
    pub osc52: bool,
    /// Write OSC 52 only if the system clipboard fails.
    pub osc52_fallback: bool,
}

pub(crate) fn plan(s: &Surroundings) -> Plan {
    if s.remote || !s.display {
        return Plan {
            system: false,
            osc52: s.osc52_possible,
            osc52_fallback: false,
        };
    }
    Plan {
        system: true,
        osc52: s.osc52_known && s.osc52_possible,
        osc52_fallback: s.osc52_possible,
    }
}

/// The bytes of one OSC 52 write, with the tmux passthrough when needed.
pub(crate) fn osc52_bytes(text: &str, tmux: bool) -> String {
    let seq = gitpoulpi_config::osc52::osc52(text);
    if tmux {
        format!("{}{seq}", wrap_tmux(&seq))
    } else {
        seq
    }
}

/// Where a copy ended up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Delivery {
    /// The system clipboard took it.
    Clipboard,
    /// Sent to the terminal through OSC 52: nothing confirms it arrived.
    Terminal,
    /// Neither path could carry it; the reason, for the status line.
    Nowhere(String),
}

/// Runs a copy by the policy. `system` is the system clipboard write, called
/// only when the plan wants it; `out` is the terminal.
pub(crate) fn deliver(
    s: &Surroundings,
    text: &str,
    system: impl FnOnce() -> Result<(), String>,
    out: &mut dyn Write,
) -> Delivery {
    let plan = plan(s);
    let system_result = if plan.system { Some(system()) } else { None };
    let system_ok = matches!(system_result, Some(Ok(())));
    let want_osc52 = plan.osc52 || (plan.osc52_fallback && !system_ok);
    let mut sent = false;
    if want_osc52 {
        sent = out
            .write_all(osc52_bytes(text, s.tmux).as_bytes())
            .and_then(|()| out.flush())
            .is_ok();
    }
    if system_ok {
        Delivery::Clipboard
    } else if sent {
        Delivery::Terminal
    } else {
        Delivery::Nowhere(match system_result {
            Some(Err(why)) => why,
            _ if s.remote => "over SSH and this terminal cannot carry OSC 52".to_string(),
            _ => "no display, and this terminal cannot carry OSC 52".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
        move |k| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| (*v).to_string())
        }
    }

    fn local(known: bool) -> Surroundings {
        Surroundings {
            remote: false,
            display: true,
            osc52_known: known,
            osc52_possible: true,
            tmux: false,
        }
    }

    #[test]
    fn the_bytes_are_the_bare_sequence_or_the_wrapped_one_then_the_bare() {
        assert_eq!(osc52_bytes("foo", false), "\x1b]52;c;Zm9v\x07");
        assert_eq!(
            osc52_bytes("foo", true),
            "\x1bPtmux;\x1b\x1b]52;c;Zm9v\x07\x1b\\\x1b]52;c;Zm9v\x07"
        );
    }

    #[test]
    fn over_ssh_only_the_terminal_is_asked() {
        let s = Surroundings::from_env(&env(&[
            ("SSH_CONNECTION", "a 1 b 22"),
            ("DISPLAY", ":0"),
            ("WAYLAND_DISPLAY", "wayland-1"),
            ("TERM", "xterm-256color"),
        ]));
        assert!(s.remote);
        let p = plan(&s);
        assert!(!p.system && p.osc52, "{p:?}");
    }

    #[test]
    fn without_a_display_only_the_terminal_is_asked() {
        if cfg!(any(windows, target_os = "macos")) {
            return;
        }
        let s = Surroundings::from_env(&env(&[("TERM", "xterm-256color")]));
        assert!(!s.display);
        let p = plan(&s);
        assert!(!p.system && p.osc52, "{p:?}");
    }

    #[test]
    fn locally_the_system_clipboard_comes_first_and_osc52_follows_a_known_terminal() {
        let known = plan(&local(true));
        assert!(known.system && known.osc52);
        let unknown = plan(&local(false));
        assert!(unknown.system && !unknown.osc52 && unknown.osc52_fallback);
    }

    #[test]
    fn terminals_are_recognized_from_their_own_environment() {
        for pairs in [
            &[("KITTY_WINDOW_ID", "1")][..],
            &[("WT_SESSION", "x")][..],
            &[("TERM_PROGRAM", "WezTerm")][..],
            &[("TERM", "foot-direct")][..],
            &[("TERM", "alacritty")][..],
        ] {
            let pairs: &'static [(&'static str, &'static str)] = Box::leak(pairs.to_vec().into());
            assert!(Surroundings::from_env(&env(pairs)).osc52_known, "{pairs:?}");
        }
        assert!(!Surroundings::from_env(&env(&[("TERM", "xterm-256color")])).osc52_known);
    }

    #[test]
    fn the_console_dumb_terminals_and_screen_cannot_carry_osc52() {
        for pairs in [
            &[("TERM", "linux")][..],
            &[("TERM", "dumb")][..],
            &[("STY", "1.pts"), ("TERM", "screen")][..],
        ] {
            let pairs: &'static [(&'static str, &'static str)] = Box::leak(pairs.to_vec().into());
            assert!(
                !Surroundings::from_env(&env(pairs)).osc52_possible,
                "{pairs:?}"
            );
        }
        assert!(
            Surroundings::from_env(&env(&[("TERM", "tmux-256color"), ("TMUX", "x")]))
                .osc52_possible
        );
    }

    #[test]
    fn a_copy_over_ssh_writes_the_sequence_and_says_terminal_not_copied() {
        let s = Surroundings {
            remote: true,
            ..local(false)
        };
        let mut out = Vec::new();
        let d = deliver(
            &s,
            "foo",
            || panic!("no system clipboard over SSH"),
            &mut out,
        );
        assert_eq!(d, Delivery::Terminal);
        assert_eq!(out, b"\x1b]52;c;Zm9v\x07");
    }

    #[test]
    fn a_copy_in_tmux_over_ssh_is_wrapped_and_bare() {
        let s = Surroundings {
            remote: true,
            tmux: true,
            ..local(false)
        };
        let mut out = Vec::new();
        deliver(&s, "foo", || Ok(()), &mut out);
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "\x1bPtmux;\x1b\x1b]52;c;Zm9v\x07\x1b\\\x1b]52;c;Zm9v\x07"
        );
    }

    #[test]
    fn a_local_copy_that_the_system_took_is_a_copy_and_writes_nothing_for_an_unknown_terminal() {
        let mut out = Vec::new();
        let d = deliver(&local(false), "foo", || Ok(()), &mut out);
        assert_eq!(d, Delivery::Clipboard);
        assert!(out.is_empty());
    }

    #[test]
    fn a_local_copy_in_a_known_terminal_also_writes_the_sequence() {
        let mut out = Vec::new();
        let d = deliver(&local(true), "foo", || Ok(()), &mut out);
        assert_eq!(d, Delivery::Clipboard);
        assert_eq!(out, b"\x1b]52;c;Zm9v\x07");
    }

    #[test]
    fn a_failing_system_clipboard_falls_back_to_the_terminal_and_says_so() {
        let mut out = Vec::new();
        let d = deliver(
            &local(false),
            "foo",
            || Err("no data-control".into()),
            &mut out,
        );
        assert_eq!(d, Delivery::Terminal);
        assert_eq!(out, b"\x1b]52;c;Zm9v\x07");
    }

    #[test]
    fn when_nothing_can_carry_it_the_reason_is_the_sentence() {
        let s = Surroundings {
            osc52_possible: false,
            ..local(false)
        };
        let mut out = Vec::new();
        let d = deliver(&s, "foo", || Err("no data-control".into()), &mut out);
        assert_eq!(d, Delivery::Nowhere("no data-control".into()));
        assert!(out.is_empty());
        let s = Surroundings {
            remote: true,
            osc52_possible: false,
            ..local(false)
        };
        assert!(matches!(
            deliver(&s, "foo", || Ok(()), &mut out),
            Delivery::Nowhere(_)
        ));
    }
}
