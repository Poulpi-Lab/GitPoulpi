//! What the Activity Log's environment report says about pictures: the
//! protocol the setting and the environment give, which rule decided, and the
//! multiplexers the environment names. Pure functions of an environment, like
//! `detect::protocol_for`, whose precedence they mirror.

use super::detect::{
    auto_protocol, inside_a_multiplexer, inside_herdr, over_ssh, protocol_for, RendererChoice,
    MULTIPLEXER_TERM_PREFIXES,
};
use super::Protocol;
use gitpoulpi_config::ImageProtocol;

/// The multiplexers the environment names.
pub(crate) fn multiplexers_in<F: Fn(&str) -> Option<String>>(get: &F) -> Vec<&'static str> {
    let set = |name: &str| get(name).is_some_and(|v| !v.is_empty());
    let mut out = Vec::new();
    if set("TMUX") {
        out.push("tmux");
    }
    if set("ZELLIJ") {
        out.push("zellij");
    }
    if set("STY") {
        out.push("screen");
    }
    if inside_herdr(get) {
        out.push("herdr");
    }
    let term = get("TERM").unwrap_or_default().to_ascii_lowercase();
    if out.is_empty()
        && MULTIPLEXER_TERM_PREFIXES
            .iter()
            .any(|p| term.starts_with(p))
    {
        out.push("screen/tmux (from TERM)");
    }
    out
}

/// The protocol, as a word, and the rule that chose it.
pub(crate) fn protocol_and_reason<F, R>(
    choice: RendererChoice,
    wanted: ImageProtocol,
    get: &F,
    read: &R,
) -> (&'static str, String)
where
    F: Fn(&str) -> Option<String>,
    R: Fn(&std::path::Path) -> Option<String>,
{
    let protocol = match protocol_for(choice, wanted, get, read) {
        Protocol::Iterm2 => "iterm2",
        Protocol::Iterm2Alpha => "iterm2 (alpha)",
        Protocol::Kitty => "kitty",
        Protocol::Sixel => "sixel",
        Protocol::None => "none",
    };
    (protocol, reason(choice, wanted, get, read))
}

fn reason<F, R>(choice: RendererChoice, wanted: ImageProtocol, get: &F, read: &R) -> String
where
    F: Fn(&str) -> Option<String>,
    R: Fn(&std::path::Path) -> Option<String>,
{
    if matches!(
        get("GITPOULPI_TERMINAL_IMAGE").as_deref(),
        Some("off") | Some("0") | Some("text")
    ) {
        return "GITPOULPI_TERMINAL_IMAGE turns pictures off".to_string();
    }
    match (choice, wanted) {
        (RendererChoice::Text, _) => "ui.graph_renderer = text".to_string(),
        (_, ImageProtocol::None) => "ui.image_protocol = none".to_string(),
        (_, ImageProtocol::Iterm2 | ImageProtocol::Kitty | ImageProtocol::Sixel) => {
            format!("ui.image_protocol = {} (forced)", wanted.as_str())
        }
        (RendererChoice::Image, ImageProtocol::Auto) => {
            "ui.graph_renderer = image (guessed, no multiplexer veto)".to_string()
        }
        (RendererChoice::Auto, ImageProtocol::Auto) => {
            let chosen = auto_protocol(get, read);
            if inside_herdr(get) {
                if over_ssh(get) {
                    "auto: herdr over SSH, the drawing terminal is not readable from here"
                        .to_string()
                } else if chosen == Protocol::Kitty {
                    "auto: herdr forwards kitty graphics".to_string()
                } else {
                    "auto: herdr without kitty_graphics, or an outer terminal that does not draw it"
                        .to_string()
                }
            } else if inside_a_multiplexer(get) {
                format!(
                    "auto: multiplexer veto ({})",
                    multiplexers_in(get).join(", ")
                )
            } else if chosen == Protocol::None {
                "auto: terminal not recognized (deny by default)".to_string()
            } else {
                "auto: recognized from the terminal's environment".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |k| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| (*v).to_string())
        }
    }
    fn nothing(_: &std::path::Path) -> Option<String> {
        None
    }

    #[test]
    fn kitty_is_recognized_from_its_environment() {
        let (p, why) = protocol_and_reason(
            RendererChoice::Auto,
            ImageProtocol::Auto,
            &env(&[("KITTY_WINDOW_ID", "1"), ("TERM", "xterm-kitty")]),
            &nothing,
        );
        assert_eq!(p, "kitty");
        assert!(why.contains("recognized"), "{why}");
    }

    #[test]
    fn a_multiplexer_vetoes_and_is_named() {
        let get = env(&[("TMUX", "/tmp/x"), ("TERM_PROGRAM", "WezTerm")]);
        let (p, why) =
            protocol_and_reason(RendererChoice::Auto, ImageProtocol::Auto, &get, &nothing);
        assert_eq!(p, "none");
        assert!(why.contains("veto") && why.contains("tmux"), "{why}");
        assert_eq!(multiplexers_in(&get), vec!["tmux"]);
    }

    #[test]
    fn a_forced_protocol_and_the_off_switch_say_so() {
        let (p, why) = protocol_and_reason(
            RendererChoice::Auto,
            ImageProtocol::Sixel,
            &env(&[]),
            &nothing,
        );
        assert_eq!(
            (p, why.as_str()),
            ("sixel", "ui.image_protocol = sixel (forced)")
        );
        let (p, why) = protocol_and_reason(
            RendererChoice::Auto,
            ImageProtocol::Kitty,
            &env(&[("GITPOULPI_TERMINAL_IMAGE", "off")]),
            &nothing,
        );
        assert_eq!(p, "none");
        assert!(why.contains("GITPOULPI_TERMINAL_IMAGE"), "{why}");
    }

    #[test]
    fn an_unknown_terminal_is_denied_by_default() {
        let (p, why) = protocol_and_reason(
            RendererChoice::Auto,
            ImageProtocol::Auto,
            &env(&[("TERM", "xterm-256color")]),
            &nothing,
        );
        assert_eq!(p, "none");
        assert!(why.contains("not recognized"), "{why}");
        assert!(multiplexers_in(&env(&[("TERM", "xterm-256color")])).is_empty());
    }

    #[test]
    fn a_screen_term_alone_counts_as_a_multiplexer() {
        assert_eq!(
            multiplexers_in(&env(&[("TERM", "screen-256color")])),
            vec!["screen/tmux (from TERM)"]
        );
    }
}
