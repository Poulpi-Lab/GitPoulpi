//! The capability lines of the Activity Log's environment report: what the
//! terminal and the settings gave this run (pictures and why, multiplexer,
//! pointer shapes, keyboard protocol, clipboard path, colors, cell size, text
//! cursor, paste). A report copied from a terminal then says what GitPoulpi
//! decided instead of leaving it to be guessed. Always English.

// i18n-scan: off

use crate::clipboard_route::{plan, Surroundings};
use crate::terminal_image::detect::RendererChoice;
use crate::terminal_image::report::{multiplexers_in, protocol_and_reason};
use gitpoulpi_config::{ImageProtocol, KittyKeyboard, TextCursor};

/// The settings and run-time facts the lines are made from.
pub(super) struct Facts {
    pub renderer: RendererChoice,
    pub image_protocol: ImageProtocol,
    pub kitty_keyboard: KittyKeyboard,
    pub kitty_keyboard_active: bool,
    pub pointer_shapes: bool,
    pub color_depth: &'static str,
    pub cell_px: (u32, u32),
    pub text_cursor: TextCursor,
    pub bracketed_paste: bool,
}

/// The facts of this process.
pub(super) fn current_facts(ui: Option<&gitpoulpi_config::UiConfig>) -> Facts {
    use crate::color_depth::Depth;
    let default_ui = gitpoulpi_config::UiConfig::default();
    let ui = ui.unwrap_or(&default_ui);
    let cell = crate::terminal_image::cell_size();
    Facts {
        renderer: ui.graph_renderer.into(),
        image_protocol: ui.image_protocol,
        kitty_keyboard: ui.kitty_keyboard,
        kitty_keyboard_active: crate::kitty_keyboard::is_active(),
        pointer_shapes: crate::cursor::pointer_shapes_allowed(),
        color_depth: match crate::color_depth::active() {
            Depth::True => "truecolor",
            Depth::X256 => "256",
            Depth::X16 => "16",
        },
        cell_px: (cell.width, cell.height),
        text_cursor: ui.text_cursor,
        bracketed_paste: crate::term_modes::bracketed_paste_requested(),
    }
}

/// The lines, from an environment lookup (injected for tests) and the facts.
pub(super) fn lines<F, R>(get: &F, read: &R, facts: &Facts) -> Vec<String>
where
    F: Fn(&str) -> Option<String>,
    R: Fn(&std::path::Path) -> Option<String>,
{
    let set = |name: &str| get(name).filter(|v| !v.is_empty());
    let mut out = Vec::new();

    let mux = multiplexers_in(get);
    out.push(format!(
        "multiplexer: {}",
        if mux.is_empty() {
            "none".to_string()
        } else {
            mux.join(", ")
        }
    ));

    let (protocol, why) = protocol_and_reason(facts.renderer, facts.image_protocol, get, read);
    out.push(format!("pictures: {protocol} ({why})"));

    out.push(format!(
        "start-up query: {}",
        match (
            set(crate::terminal_image::detect::PROBE_VERSION_KEY),
            set(crate::terminal_image::detect::PROBE_ATTRIBUTES_KEY),
        ) {
            (None, None) => "no answer".to_string(),
            (version, attributes) => format!(
                "XTVERSION {}, device attributes {}",
                version.unwrap_or_else(|| "no answer".to_string()),
                attributes.unwrap_or_else(|| "no answer".to_string()),
            ),
        }
    ));

    out.push(format!(
        "pointer shape (OSC 22): {}",
        if facts.pointer_shapes {
            "sent".to_string()
        } else if !mux.is_empty() {
            format!("not sent (inside {})", mux.join(", "))
        } else {
            "not sent (TERM=linux or dumb)".to_string()
        }
    ));

    out.push(format!(
        "keyboard: {}",
        match (facts.kitty_keyboard, facts.kitty_keyboard_active) {
            (_, true) => "kitty protocol active".to_string(),
            (KittyKeyboard::Off, false) => "legacy (ui.kitty_keyboard = off)".to_string(),
            (KittyKeyboard::Auto, false) =>
                "legacy (ui.kitty_keyboard = auto, the terminal did not answer the query)"
                    .to_string(),
        }
    ));

    let memory = set("GITPOULPI_CLIPBOARD").as_deref() == Some("memory");
    out.push(format!(
        "clipboard: {}",
        if memory {
            "in-process memory (GITPOULPI_CLIPBOARD=memory)".to_string()
        } else {
            clipboard_route(&Surroundings::from_env(get))
        }
    ));

    out.push(format!("color depth: {}", facts.color_depth));
    out.push(format!(
        "cell size: {}x{} px",
        facts.cell_px.0, facts.cell_px.1
    ));
    out.push(format!("text cursor: {}", facts.text_cursor.as_str()));
    out.push(format!(
        "bracketed paste: {}",
        if facts.bracketed_paste {
            "requested"
        } else {
            "not requested on this platform"
        }
    ));
    out
}

/// The clipboard policy for these surroundings, in words
/// (`clipboard_route` has the rules).
fn clipboard_route(around: &Surroundings) -> String {
    let plan = plan(around);
    let tmux = if around.tmux {
        " (wrapped for the tmux passthrough, and bare)"
    } else {
        ""
    };
    if !plan.system {
        let why = if around.remote {
            "over SSH the system clipboard would be the remote one"
        } else {
            "no display for a system clipboard"
        };
        return if plan.osc52 {
            format!("terminal clipboard through OSC 52{tmux}, not confirmable ({why})")
        } else {
            format!("none: {why}, and this terminal cannot carry OSC 52")
        };
    }
    let mut text = "system clipboard of the machine running GitPoulpi (arboard)".to_string();
    if plan.osc52 {
        text.push_str(&format!(
            ", plus OSC 52 to the terminal (known to support it){tmux}"
        ));
    } else if plan.osc52_fallback {
        text.push_str(&format!("; OSC 52 to the terminal if that fails{tmux}"));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> Facts {
        Facts {
            renderer: RendererChoice::Auto,
            image_protocol: ImageProtocol::Auto,
            kitty_keyboard: KittyKeyboard::Auto,
            kitty_keyboard_active: false,
            pointer_shapes: true,
            color_depth: "truecolor",
            cell_px: (9, 21),
            text_cursor: TextCursor::Terminal,
            bracketed_paste: true,
        }
    }

    fn run(pairs: &[(&str, &str)], facts: &Facts) -> Vec<String> {
        let get = |k: &str| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| (*v).to_string())
        };
        lines(&get, &|_| None, facts)
    }

    #[test]
    fn a_plain_kitty_window_reports_every_capability() {
        let text = run(
            &[
                ("KITTY_WINDOW_ID", "3"),
                ("TERM", "xterm-kitty"),
                ("DISPLAY", ":0"),
            ],
            &facts(),
        )
        .join("\n");
        for needle in [
            "multiplexer: none",
            "pictures: kitty (auto: recognized",
            "pointer shape (OSC 22): sent",
            "keyboard: legacy (ui.kitty_keyboard = auto, the terminal did not answer",
            "clipboard: system clipboard",
            "color depth: truecolor",
            "cell size: 9x21 px",
            "text cursor: terminal",
            "bracketed paste: requested",
        ] {
            assert!(text.contains(needle), "missing {needle:?} in:\n{text}");
        }
    }

    #[test]
    fn inside_tmux_the_veto_and_the_silent_pointer_are_explained() {
        let mut f = facts();
        f.pointer_shapes = false;
        f.kitty_keyboard_active = true;
        let text = run(&[("TMUX", "/tmp/t,1,0"), ("TERM_PROGRAM", "WezTerm")], &f).join("\n");
        assert!(text.contains("multiplexer: tmux"), "{text}");
        assert!(
            text.contains("pictures: none (auto: multiplexer veto (tmux; its client does not advertise sixel))"),
            "{text}"
        );
        assert!(
            text.contains("pointer shape (OSC 22): not sent (inside tmux)"),
            "{text}"
        );
        assert!(text.contains("keyboard: kitty protocol active"), "{text}");
    }

    #[test]
    fn ssh_and_the_memory_clipboard_are_named() {
        let text = run(
            &[("SSH_CONNECTION", "a b c d"), ("TERM", "xterm-256color")],
            &facts(),
        )
        .join("\n");
        assert!(
            text.contains("clipboard: terminal clipboard through OSC 52"),
            "{text}"
        );
        assert!(text.contains("over SSH the system clipboard would be the remote one"));
        let text = run(&[("GITPOULPI_CLIPBOARD", "memory")], &facts()).join("\n");
        assert!(text.contains("clipboard: in-process memory"), "{text}");
    }

    #[test]
    fn the_probe_answers_and_the_tmux_wrapping_are_reported() {
        let text = run(&[], &facts()).join("\n");
        assert!(text.contains("start-up query: no answer"), "{text}");
        let text = run(
            &[
                ("\0probe:xtversion", "kitty(0.35.2)"),
                ("\0probe:da1", "62;4"),
                ("SSH_TTY", "/dev/pts/1"),
                ("TMUX", "x"),
                ("TERM", "tmux-256color"),
            ],
            &facts(),
        )
        .join("\n");
        assert!(
            text.contains("start-up query: XTVERSION kitty(0.35.2), device attributes 62;4"),
            "{text}"
        );
        assert!(text.contains("wrapped for the tmux passthrough"), "{text}");
    }

    #[test]
    fn the_drawn_caret_and_an_unrequested_paste_are_reported() {
        let mut f = facts();
        f.text_cursor = TextCursor::Drawn;
        f.bracketed_paste = false;
        let text = run(&[], &f).join("\n");
        assert!(text.contains("text cursor: drawn"));
        assert!(text.contains("bracketed paste: not requested on this platform"));
    }
}
