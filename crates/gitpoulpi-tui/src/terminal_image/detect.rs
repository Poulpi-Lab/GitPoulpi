use super::passthrough::{self, tmux_client_has_sixel, tmux_only};
use super::*;

/// Terminals that implement the iTerm2 inline-image protocol. Matched
/// against `TERM_PROGRAM`, case-insensitively.
///
/// A deny-by-default list rather than a sniff: an unknown terminal gets
/// the text rendering, which is correct everywhere, instead of a gamble
/// that paints base64 across the pane when it loses.
pub(super) const ITERM2_TERM_PROGRAMS: [&str; 5] =
    ["wezterm", "iterm.app", "rio", "mintty", "bobcat"];

/// Environment variables a **pane multiplexer** sets, and that therefore
/// mean *the thing that owns the screen is not the terminal those other
/// variables describe*.
///
/// A multiplexer runs its children with its own parent's environment, so
/// inside one `WEZTERM_EXECUTABLE` and `TERM_PROGRAM` still name WezTerm
/// while the process actually drawing the cells is the multiplexer - and
/// it forwards cells, not escape sequences it does not understand. The
/// observed failure was the Graph column filled with chewed-up base64
/// (`fffeffeat`, `ttteettest`), because the payload was partly echoed and
/// partly eaten.
///
/// The `HERDR_*` names are **measured, not guessed**: read out of a live
/// herdr 0.9.1 pane on Windows on 2026-09-23, where the environment
/// carried `HERDR_ENV=1`, `HERDR_PANE_ID`, `HERDR_TAB_ID`,
/// `HERDR_WORKSPACE_ID` and `HERDR_SOCKET_PATH` *alongside* an unchanged
/// `WEZTERM_EXECUTABLE` and `TERM_PROGRAM=WezTerm`. `TERM` was
/// `xterm-256color`, so `TERM` alone would not have caught it.
pub(super) const MULTIPLEXER_VARS: [&str; 6] = [
    "HERDR_ENV",
    "HERDR_PANE_ID",
    "HERDR_SOCKET_PATH",
    "TMUX",
    "ZELLIJ",
    "STY",
];

/// `TERM` prefixes that say a multiplexer is in the way even when it set
/// no variable of its own. `screen` covers GNU screen and tmux's default
/// `screen-256color`.
pub(super) const MULTIPLEXER_TERM_PREFIXES: [&str; 2] = ["screen", "tmux"];

/// Whether something is sitting between this process and the terminal
/// whose name the environment carries.
///
/// Deny by default, the same principle as [`ITERM2_TERM_PROGRAMS`]: a
/// multiplexer that *does* forward the protocol costs its user one
/// `ui.graph_renderer = "image"`; a multiplexer that does not, and is
/// trusted, paints base64 across their pane. The two mistakes are not the
/// same size.
pub(super) fn inside_a_multiplexer<F: Fn(&str) -> Option<String>>(get: &F) -> bool {
    if MULTIPLEXER_VARS.iter().any(|v| {
        // Inherited herdr variables in a window of kitty's own are no
        // multiplexer (see `inside_herdr`).
        let herdr_name = v.starts_with("HERDR_");
        (!herdr_name || inside_herdr(get)) && get(v).is_some_and(|value| !value.is_empty())
    }) {
        return true;
    }
    get("TERM").is_some_and(|term| {
        let term = term.to_ascii_lowercase();
        MULTIPLEXER_TERM_PREFIXES
            .iter()
            .any(|p| term.starts_with(p))
    })
}

/// Whether this process runs in a herdr pane. The variables are the
/// measured ones listed in [`MULTIPLEXER_VARS`].
///
/// `TERM=xterm-kitty` overrules them: kitty sets it for every window it
/// opens and herdr sets its own `TERM` for its panes, so that value means
/// the process talks to kitty directly and the `HERDR_*` names were
/// inherited from the shell that launched the window. Taking the leak for
/// a herdr pane found no `kitty_graphics` in herdr's config, picked
/// iTerm2 - which kitty does not draw - and left the Graph column blank.
pub(super) fn inside_herdr<F: Fn(&str) -> Option<String>>(get: &F) -> bool {
    if get("TERM").is_some_and(|t| t.eq_ignore_ascii_case("xterm-kitty")) {
        return false;
    }
    ["HERDR_ENV", "HERDR_PANE_ID", "HERDR_SOCKET_PATH"]
        .iter()
        .any(|v| get(v).is_some_and(|value| !value.is_empty()))
}

/// Whether this process runs on the far side of an SSH connection:
/// `sshd` sets `SSH_CONNECTION` / `SSH_CLIENT` (and `SSH_TTY` for a pty) and
/// herdr keeps them in its panes' environment (measured on herdr 0.9.3,
/// `herdr --remote`).
///
/// Inside herdr this means **the environment and `config.toml` this
/// process can read are the remote host's, not those of the terminal that
/// draws**. Kitty images are decided by the *local* herdr client - its own
/// `kitty_graphics` and its own outer terminal - and nothing on this side
/// can see either. Guessing wrong leaves the Graph column blank: the
/// placeholder cells are drawn, the picture never arrives (reproduced with
/// a client whose `kitty_graphics = false` against a server whose config
/// said `true`).
pub(super) fn over_ssh<F: Fn(&str) -> Option<String>>(get: &F) -> bool {
    ["SSH_CONNECTION", "SSH_CLIENT", "SSH_TTY"]
        .iter()
        .any(|v| get(v).is_some_and(|value| !value.is_empty()))
}

/// A terminal that implements the Kitty graphics protocol *and* its
/// Unicode placeholders, identified by the variables it sets itself:
/// kitty (`KITTY_WINDOW_ID`, `TERM=xterm-kitty`) and Ghostty
/// (`TERM_PROGRAM=ghostty`, `GHOSTTY_RESOURCES_DIR`, `TERM=xterm-ghostty`).
pub(super) fn speaks_kitty_natively<F: Fn(&str) -> Option<String>>(get: &F) -> bool {
    let set = |name: &str| get(name).is_some_and(|v| !v.is_empty());
    if set("KITTY_WINDOW_ID") || set("GHOSTTY_RESOURCES_DIR") {
        return true;
    }
    let term = get("TERM").unwrap_or_default().to_ascii_lowercase();
    let program = get("TERM_PROGRAM").unwrap_or_default().to_ascii_lowercase();
    term == "xterm-kitty" || term == "xterm-ghostty" || program.contains("ghostty")
}

/// Whether the terminal *around* herdr draws the Kitty images herdr
/// re-emits: kitty, Ghostty, or WezTerm (the maintainer's; yazi's pictures
/// show there through herdr). herdr leaves the outer terminal's variables
/// in the pane's environment, which is what makes this answerable --
/// but it strips `KITTY_WINDOW_ID` (the window id is the outer window's,
/// meaningless in a pane), measured on herdr 0.9.x inside kitty 0.48.2
/// on Linux. What stays is `KITTY_PID`, `KITTY_LISTEN_ON`,
/// `KITTY_PUBLIC_KEY`, `KITTY_INSTALLATION_DIR`, so those count here too,
/// and Ghostty's `GHOSTTY_BIN_DIR`. Only this herdr branch looks at them:
/// a tmux inside kitty inherits the same names and must stay text, which
/// is why [`speaks_kitty_natively`] does not.
pub(super) fn outer_terminal_draws_kitty<F: Fn(&str) -> Option<String>>(get: &F) -> bool {
    const LEFTOVERS: [&str; 5] = [
        "KITTY_PID",
        "KITTY_INSTALLATION_DIR",
        "KITTY_LISTEN_ON",
        "KITTY_PUBLIC_KEY",
        "GHOSTTY_BIN_DIR",
    ];
    speaks_kitty_natively(get)
        || looks_like_wezterm(get)
        || LEFTOVERS
            .iter()
            .any(|v| get(v).is_some_and(|value| !value.is_empty()))
}

pub(super) fn looks_like_wezterm<F: Fn(&str) -> Option<String>>(get: &F) -> bool {
    get("WEZTERM_EXECUTABLE").is_some_and(|v| !v.is_empty())
        || get("TERM_PROGRAM").is_some_and(|v| v.eq_ignore_ascii_case("wezterm"))
}

/// Where herdr keeps its `config.toml`, most likely first:
/// `%APPDATA%\herdr` on Windows, `$XDG_CONFIG_HOME/herdr` or
/// `~/.config/herdr` elsewhere (herdr's configuration docs).
pub(super) fn herdr_config_paths<F: Fn(&str) -> Option<String>>(
    get: &F,
) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    for (var, tail) in [
        ("APPDATA", &["herdr"][..]),
        ("XDG_CONFIG_HOME", &["herdr"][..]),
        ("HOME", &[".config", "herdr"][..]),
    ] {
        if let Some(dir) = get(var).filter(|d| !d.is_empty()) {
            let mut path = std::path::PathBuf::from(dir);
            for part in tail {
                path.push(part);
            }
            path.push("config.toml");
            paths.push(path);
        }
    }
    paths
}

/// herdr's `kitty_graphics` switch as its config file states it:
/// `[terminal] kitty_graphics`, which wins, or the legacy
/// `[experimental] kitty_graphics` (the maintainer's). `None` when the file
/// does not say - which this code treats as "no": herdr's default has
/// changed across releases, and a pane wrongly believed to draw images
/// shows an empty Graph column instead of the text one.
///
/// A line scanner rather than a TOML parser: two keys, and a herdr config
/// this crate's `toml` version might reject must not cost the answer.
pub(super) fn herdr_kitty_graphics(config: &str) -> Option<bool> {
    let (mut terminal, mut experimental) = (None, None);
    let mut section = String::new();
    for line in config.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if let Some(header) = line.strip_prefix('[') {
            section = header.trim_end_matches(']').trim().to_string();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = match value.trim() {
            "true" => true,
            "false" => false,
            _ => continue,
        };
        let key = key.trim();
        let dotted = if section.is_empty() {
            key.to_string()
        } else {
            format!("{section}.{key}")
        };
        match dotted.as_str() {
            "terminal.kitty_graphics" => terminal = Some(value),
            "experimental.kitty_graphics" => experimental = Some(value),
            _ => {}
        }
    }
    terminal.or(experimental)
}

/// Whether the herdr this pane runs in passes Kitty images on to a
/// terminal that draws them.
pub(super) fn herdr_forwards_kitty<F, R>(get: &F, read: &R) -> bool
where
    F: Fn(&str) -> Option<String>,
    R: Fn(&std::path::Path) -> Option<String>,
{
    let enabled = herdr_config_paths(get)
        .iter()
        .find_map(|path| read(path))
        .and_then(|config| herdr_kitty_graphics(&config));
    enabled == Some(true) && outer_terminal_draws_kitty(get)
}

/// What the user asked for, independent of what the terminal can do.
/// `ui.graph_renderer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum RendererChoice {
    #[default]
    Auto,
    Image,
    Text,
}

impl From<gitpoulpi_config::GraphRenderer> for RendererChoice {
    fn from(value: gitpoulpi_config::GraphRenderer) -> RendererChoice {
        match value {
            gitpoulpi_config::GraphRenderer::Auto => RendererChoice::Auto,
            gitpoulpi_config::GraphRenderer::Image => RendererChoice::Image,
            gitpoulpi_config::GraphRenderer::Text => RendererChoice::Text,
        }
    }
}

/// Detects what the terminal can do.
///
/// Cheap and side-effect free - it reads environment variables and, at
/// most, `TIOCGWINSZ`. It writes nothing and reads no input, which is the
/// property that matters: see [`cell_size`].
pub(crate) fn detect(choice: RendererChoice, wanted: ImageProtocol) -> Capability {
    Capability {
        protocol: protocol_for(
            choice,
            wanted,
            &|name| {
                if name == passthrough::CLIENT_FEATURES_KEY {
                    passthrough::query_client_features()
                } else {
                    std::env::var(name).ok()
                }
            },
            &|path| std::fs::read_to_string(path).ok(),
        ),
        cell: cell_size(),
    }
}

/// The whole verdict as a **pure function of an environment**, so every
/// case below is a unit test rather than a thing you can only find out by
/// running the TUI inside four different terminals.
///
/// Precedence, highest first:
///
/// 1. `GITPOULPI_TERMINAL_IMAGE=off|0|text` - the low-level escape hatch,
///    and what the pty harness sets. It wins over an explicit
///    `ui.graph_renderer = "image"` on purpose: it exists for the case
///    where emitting the sequence actively breaks the session.
/// 2. `ui.graph_renderer = "text"` or `ui.image_protocol = "none"`.
/// 3. `ui.image_protocol = "iterm2"` / `"kitty"`: that protocol, trusted.
/// 4. `ui.graph_renderer = "image"`: pictures on, the protocol guessed
///    without the multiplexer veto - Kitty for kitty, Ghostty and a
///    Kitty-enabled herdr, iTerm2 otherwise.
/// 5. Deny-by-default detection ([`auto_protocol`]).
///
/// `read` fetches a file (herdr's config), injected like `get` so no test
/// touches the disk.
pub(super) fn protocol_for<F, R>(
    choice: RendererChoice,
    wanted: ImageProtocol,
    get: &F,
    read: &R,
) -> Protocol
where
    F: Fn(&str) -> Option<String>,
    R: Fn(&std::path::Path) -> Option<String>,
{
    if matches!(
        get("GITPOULPI_TERMINAL_IMAGE").as_deref(),
        Some("off") | Some("0") | Some("text")
    ) {
        return Protocol::None;
    }
    match (choice, wanted) {
        (RendererChoice::Text, _) | (_, ImageProtocol::None) => Protocol::None,
        (_, ImageProtocol::Iterm2) => iterm2_for(get),
        (_, ImageProtocol::Kitty) => Protocol::Kitty,
        (_, ImageProtocol::Sixel) => Protocol::Sixel,
        (RendererChoice::Image, ImageProtocol::Auto) => {
            let kitty = if inside_herdr(get) {
                // Pictures were asked for explicitly. Over SSH the herdr
                // config here says nothing about the client that draws,
                // and the only protocol herdr carries is Kitty's.
                over_ssh(get) || herdr_forwards_kitty(get, read)
            } else {
                !looks_like_wezterm(get) && speaks_kitty_natively(get)
            };
            if kitty {
                Protocol::Kitty
            } else if sixel_only_terminal(get) {
                Protocol::Sixel
            } else {
                iterm2_for(get)
            }
        }
        (RendererChoice::Auto, ImageProtocol::Auto) => auto_protocol(get, read),
    }
}

/// The `auto` verdict, deny by default:
///
/// - inside herdr over SSH (`herdr --remote`, or `ssh` then `herdr`), text:
///   see [`over_ssh`], the answer is not readable from here;
/// - inside herdr, Kitty when herdr's config says it forwards it and the
///   terminal around herdr draws it; text otherwise;
/// - inside any other multiplexer, text;
/// - WezTerm, iTerm2 - it speaks both, and iTerm2 is the proven one;
/// - kitty and Ghostty, Kitty;
/// - the other iTerm2 terminals, iTerm2;
/// - anything else, text.
pub(super) fn auto_protocol<F, R>(get: &F, read: &R) -> Protocol
where
    F: Fn(&str) -> Option<String>,
    R: Fn(&std::path::Path) -> Option<String>,
{
    if inside_herdr(get) {
        return if !over_ssh(get) && herdr_forwards_kitty(get, read) {
            Protocol::Kitty
        } else {
            Protocol::None
        };
    }
    if tmux_only(get) {
        // tmux draws sixel itself for a client that takes it; it carries
        // nothing else (`passthrough` module).
        return if tmux_client_has_sixel(get) {
            Protocol::Sixel
        } else {
            Protocol::None
        };
    }
    if inside_a_multiplexer(get) {
        return Protocol::None;
    }
    if looks_like_wezterm(get) {
        return iterm2_for(get);
    }
    if speaks_kitty_natively(get) {
        return Protocol::Kitty;
    }
    if looks_like_iterm2_terminal(get) {
        iterm2_for(get)
    } else if sixel_only_terminal(get) {
        Protocol::Sixel
    } else {
        Protocol::None
    }
}

/// A terminal that draws sixel and neither of the other two protocols,
/// told from its own environment (preference is kitty, iTerm2, sixel):
/// Windows Terminal (`WT_SESSION`; sixel since 1.22), foot (`TERM=foot*`),
/// Konsole (`KONSOLE_VERSION`), mlterm (`TERM=mlterm*`) and Tabby
/// (`TERM_PROGRAM=Tabby`; 1.0.238 on Linux answers sixel in its device
/// attributes and draws a transparent sixel picture, and shows neither kitty
/// nor iTerm2 images).
///
/// **Not DA1.** The textbook test is parameter 4 in the reply to `CSI c`,
/// but the reply arrives on stdin and detection here never reads stdin
/// (see [`cell_size`]: a blocked read cannot be canceled and eats
/// keystrokes). So the VS Code terminal (`terminal.integrated.enableImages`
/// is off by default) and a sixel-enabled xterm stay text under `auto`;
/// `ui.image_protocol = "sixel"` opts in.
pub(super) fn sixel_only_terminal<F: Fn(&str) -> Option<String>>(get: &F) -> bool {
    let set = |name: &str| get(name).is_some_and(|v| !v.is_empty());
    let term = get("TERM").unwrap_or_default().to_ascii_lowercase();
    set("WT_SESSION")
        || tmux_client_has_sixel(get)
        || set("KONSOLE_VERSION")
        || term.starts_with("foot")
        || term.starts_with("mlterm")
        || get("TERM_PROGRAM").is_some_and(|p| p.eq_ignore_ascii_case("tabby"))
}

/// The iTerm2 flavour for this terminal: [`Protocol::Iterm2Alpha`] when it
/// is known to blend PNG alpha, the opaque-backdrop [`Protocol::Iterm2`]
/// otherwise.
///
/// - **WezTerm: blends.** Its image cells are alpha-blended over the cell
///   background by the GPU renderer (wezterm issue #544, "alpha blending
///   in linear space", settled with transparent PNGs shown over the
///   terminal's own background), and its `imgcat` page documents the
///   protocol. Confirmed here by a real WezTerm window capture
///   (`docs/developer/visual-verification.md`).
/// - **Rio (0.5.28, Linux): blends.** A transparent PNG sent as `OSC 1337`
///   over a bright field shows the field through the transparent corners
///   (raw-sequence probe in a real window, November 2026). Only Linux was
///   looked at; Windows keeps the opaque backdrop until it is.
/// - **iTerm2 itself, mintty, Bobcat: unverified.** mintty's wiki
///   documents the `File=` image controls without any word on alpha
///   (only its OSC 11 background images are said to honour it), and this
///   project has not seen the others on a real window, so they keep the
///   opaque backdrop. A wrong guess there is a visible lighter rectangle;
///   a wrong guess the other way could be a black one.
///
/// Inside herdr/tmux this is never reached: those cannot carry OSC 1337.
pub(super) fn iterm2_for<F: Fn(&str) -> Option<String>>(get: &F) -> Protocol {
    if blends_png_alpha(get) {
        Protocol::Iterm2Alpha
    } else {
        Protocol::Iterm2
    }
}

/// Whether the terminal named by the environment composites inline PNG
/// alpha over the cell background. WezTerm, and Rio off Windows, by
/// verification.
pub(super) fn blends_png_alpha<F: Fn(&str) -> Option<String>>(get: &F) -> bool {
    (looks_like_wezterm(get) || looks_like_rio(get))
        && !inside_a_multiplexer(get)
        && !inside_herdr(get)
}

/// Rio sets `TERM_PROGRAM=rio` (and `TERM=rio` when its terminfo is
/// installed). Only where its alpha blending was seen: not on Windows.
pub(super) fn looks_like_rio<F: Fn(&str) -> Option<String>>(get: &F) -> bool {
    !cfg!(windows) && get("TERM_PROGRAM").is_some_and(|v| v.eq_ignore_ascii_case("rio"))
}

pub(super) fn looks_like_iterm2_terminal<F: Fn(&str) -> Option<String>>(get: &F) -> bool {
    // WezTerm sets `WEZTERM_EXECUTABLE` on every platform, including
    // Windows, where `TERM_PROGRAM` is not always populated.
    if looks_like_wezterm(get) {
        return true;
    }
    get("TERM_PROGRAM").is_some_and(|v| {
        let v = v.to_ascii_lowercase();
        ITERM2_TERM_PROGRAMS.iter().any(|t| v.contains(t))
    })
}
