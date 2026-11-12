use super::*;

#[test]
fn a_bare_wezterm_draws_the_graph_as_an_image() {
    assert_eq!(
        protocol_for(RendererChoice::Auto, &env(&WEZTERM)),
        Protocol::Iterm2Alpha
    );
}

#[test]
fn wezterm_with_herdr_in_between_falls_back_to_text() {
    // The defect this whole change exists for: herdr owns the screen,
    // does not forward `OSC 1337`, and the base64 payload came out as
    // `fffeffeat` / `ttteettest` chewed across the Graph column.
    assert_eq!(
        protocol_for(RendererChoice::Auto, &env(&wezterm_inside_herdr())),
        Protocol::None
    );
}

#[test]
fn every_multiplexer_we_know_of_falls_back_to_text() {
    for extra in [
        ("TMUX", "/tmp/tmux-1000/default,1234,0"),
        ("ZELLIJ", "0"),
        ("STY", "1234.pts-0.host"),
    ] {
        let mut pairs = WEZTERM.to_vec();
        pairs.push(extra);
        assert_eq!(
            protocol_for(RendererChoice::Auto, &env(&pairs)),
            Protocol::None,
            "{} in the environment means something is between us and the terminal",
            extra.0
        );
    }
    // And the one that sets no variable of its own but does rewrite
    // `TERM`.
    for term in ["screen-256color", "tmux-256color"] {
        let pairs = [("WEZTERM_EXECUTABLE", "/usr/bin/wezterm"), ("TERM", term)];
        assert_eq!(
            protocol_for(RendererChoice::Auto, &env(&pairs)),
            Protocol::None,
            "TERM={term}"
        );
    }
}

#[test]
fn an_unknown_terminal_gets_text_because_the_default_is_deny() {
    assert_eq!(
        protocol_for(RendererChoice::Auto, &env(&[("TERM", "xterm-256color")])),
        Protocol::None
    );
    assert_eq!(
        protocol_for(
            RendererChoice::Auto,
            &env(&[("TERM_PROGRAM", "Apple_Terminal")])
        ),
        Protocol::None
    );
}

#[test]
fn an_empty_variable_is_not_a_terminal_and_not_a_multiplexer() {
    // `set TMUX=` and an unset `TMUX` must mean the same thing, and an
    // empty `WEZTERM_EXECUTABLE` must not vouch for anything.
    assert_eq!(
        protocol_for(
            RendererChoice::Auto,
            &env(&[("WEZTERM_EXECUTABLE", ""), ("TERM", "xterm-256color")])
        ),
        Protocol::None
    );
    let mut pairs = WEZTERM.to_vec();
    pairs.push(("TMUX", ""));
    assert_eq!(
        protocol_for(RendererChoice::Auto, &env(&pairs)),
        Protocol::Iterm2Alpha
    );
}

#[test]
fn forcing_image_overrides_the_multiplexer_verdict() {
    // The user's escape hatch in the other direction: a multiplexer
    // that *does* pass images through must not be permanently denied
    // by a list this code can only ever be behind on.
    assert_eq!(
        protocol_for(RendererChoice::Image, &env(&wezterm_inside_herdr())),
        Protocol::Iterm2
    );
    assert_eq!(
        protocol_for(RendererChoice::Image, &env(&[])),
        Protocol::Iterm2
    );
}

#[test]
fn forcing_text_overrides_a_perfectly_capable_terminal() {
    assert_eq!(
        protocol_for(RendererChoice::Text, &env(&WEZTERM)),
        Protocol::None
    );
}

#[test]
fn the_env_kill_switch_wins_over_everything_including_forced_image() {
    // `GITPOULPI_TERMINAL_IMAGE=off` is what the pty harness sets, and
    // a config file that says `image` must not be able to break the
    // test suite from a user's home directory.
    for value in ["off", "0", "text"] {
        let mut pairs = WEZTERM.to_vec();
        pairs.push(("GITPOULPI_TERMINAL_IMAGE", value));
        assert_eq!(
            protocol_for(RendererChoice::Image, &env(&pairs)),
            Protocol::None,
            "GITPOULPI_TERMINAL_IMAGE={value}"
        );
    }
}

const FOOT: [(&str, &str); 1] = [("TERM", "foot")];

#[test]
fn sixel_only_terminals_are_recognised_from_their_environment() {
    for pairs in [
        vec![("WT_SESSION", "0a1b2c3d")],
        vec![("TERM", "foot-extra")],
        vec![("TERM", "xterm-256color"), ("KONSOLE_VERSION", "240800")],
        vec![("TERM", "mlterm")],
        vec![("TERM", "xterm-256color"), ("TERM_PROGRAM", "Tabby")],
    ] {
        assert_eq!(
            protocol_for(RendererChoice::Auto, &env(&pairs)),
            Protocol::Sixel,
            "{pairs:?}"
        );
    }
}

#[test]
fn tabby_draws_sixel_and_is_told_from_its_own_variables() {
    // Read out of a Tabby 1.0.238 tab on Windows.
    for pairs in [
        vec![("TERM_PROGRAM", "Tabby"), ("TERM", "xterm-256color")],
        vec![(
            "TABBY_CONFIG_DIRECTORY",
            r"C:\Users\u\AppData\Roaming\tabby",
        )],
    ] {
        assert_eq!(
            protocol_for(RendererChoice::Auto, &env(&pairs)),
            Protocol::Sixel,
            "{pairs:?}"
        );
    }
    // Tabby's variables inherited into a tmux pane: a multiplexer owns the screen.
    let tmux = [
        ("TERM_PROGRAM", "Tabby"),
        ("TMUX", "/tmp/tmux-1000/default,1,0"),
    ];
    assert_eq!(
        protocol_for(RendererChoice::Auto, &env(&tmux)),
        Protocol::None
    );
}

#[test]
fn rio_gets_the_iterm2_protocol_with_alpha_and_not_inside_a_multiplexer() {
    let rio = [("TERM_PROGRAM", "rio"), ("TERM_PROGRAM_VERSION", "0.5.28")];
    assert_eq!(
        protocol_for(RendererChoice::Auto, &env(&rio)),
        Protocol::Iterm2Alpha
    );
    let tmux = [
        ("TERM_PROGRAM", "rio"),
        ("TMUX", "/tmp/tmux-1000/default,1,0"),
    ];
    assert_eq!(
        protocol_for(RendererChoice::Auto, &env(&tmux)),
        Protocol::None
    );
    // mintty keeps the opaque backdrop: its alpha handling is not verified.
    assert_eq!(
        protocol_for(RendererChoice::Auto, &env(&[("TERM_PROGRAM", "mintty")])),
        Protocol::Iterm2
    );
}

#[test]
fn sixel_comes_after_kitty_and_iterm2_in_the_preference() {
    // WezTerm inside Windows Terminal's variables: iTerm2 wins.
    let mut wez = WEZTERM.to_vec();
    wez.push(("WT_SESSION", "x"));
    assert_eq!(
        protocol_for(RendererChoice::Auto, &env(&wez)),
        Protocol::Iterm2Alpha
    );
    // kitty's variables next to a foot TERM: kitty wins.
    let kitty = [("KITTY_WINDOW_ID", "1"), ("TERM", "foot")];
    assert_eq!(
        protocol_for(RendererChoice::Auto, &env(&kitty)),
        Protocol::Kitty
    );
}

#[test]
fn sixel_is_denied_inside_a_multiplexer_and_when_pictures_are_off() {
    let mut tmux = FOOT.to_vec();
    tmux.push(("TMUX", "/tmp/tmux-1000/default,1,0"));
    assert_eq!(
        protocol_for(RendererChoice::Auto, &env(&tmux)),
        Protocol::None
    );
    let mut wt = vec![("WT_SESSION", "x"), ("GITPOULPI_TERMINAL_IMAGE", "off")];
    assert_eq!(
        protocol_for(RendererChoice::Auto, &env(&wt)),
        Protocol::None
    );
    wt.pop();
    assert_eq!(
        protocol_for(RendererChoice::Text, &env(&wt)),
        Protocol::None
    );
    assert_eq!(
        super::super::protocol_for(
            RendererChoice::Auto,
            ImageProtocol::None,
            &env(&wt),
            &|_: &std::path::Path| None
        ),
        Protocol::None
    );
}

#[test]
fn an_unknown_terminal_stays_text_and_the_setting_forces_sixel() {
    assert_eq!(
        protocol_for(RendererChoice::Auto, &env(&[("TERM", "xterm-256color")])),
        Protocol::None
    );
    assert_eq!(
        super::super::protocol_for(
            RendererChoice::Auto,
            ImageProtocol::Sixel,
            &env(&[("TERM", "xterm-256color")]),
            &|_: &std::path::Path| None
        ),
        Protocol::Sixel
    );
    // `ui.graph_renderer = "image"` picks it for a sixel-only terminal.
    assert_eq!(
        protocol_for(RendererChoice::Image, &env(&FOOT)),
        Protocol::Sixel
    );
}

/// tmux 3.8 inside foot 1.28, as measured: `TERM` is tmux's, `TERM_PROGRAM`
/// is `tmux`, and the client's feature list names sixel.
const TMUX_IN_FOOT: [(&str, &str); 3] = [
    ("TMUX", "/tmp/tmux-1000/default,1234,0"),
    ("TERM", "tmux-256color"),
    ("TERM_PROGRAM", "tmux"),
];

const FOOT_CLIENT_FEATURES: &str = "256,bpaste,clipboard,RGB,sixel,sync,title";
const KITTY_CLIENT_FEATURES: &str = "bpaste,clipboard,RGB,sync,title";

/// `pairs` plus tmux's answer to "what does your client support".
fn with_client_features<'a>(
    pairs: &'a [(&'a str, &'a str)],
    features: Option<&'a str>,
) -> impl Fn(&str) -> Option<String> + 'a {
    move |name: &str| {
        if name == super::super::passthrough::CLIENT_FEATURES_KEY {
            return features.map(str::to_string);
        }
        pairs
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| (*v).to_string())
    }
}

#[test]
fn tmux_in_a_sixel_terminal_gets_sixel_and_nothing_else_does() {
    let get = with_client_features(&TMUX_IN_FOOT, Some(FOOT_CLIENT_FEATURES));
    assert_eq!(protocol_for(RendererChoice::Auto, &get), Protocol::Sixel);
    // Asked for pictures explicitly: the same answer.
    assert_eq!(protocol_for(RendererChoice::Image, &get), Protocol::Sixel);
}

#[test]
fn tmux_in_a_terminal_without_sixel_stays_text() {
    let get = with_client_features(&TMUX_IN_FOOT, Some(KITTY_CLIENT_FEATURES));
    assert_eq!(protocol_for(RendererChoice::Auto, &get), Protocol::None);
    // tmux not answering (not installed, hung, no client) is a "no", too.
    let silent = with_client_features(&TMUX_IN_FOOT, None);
    assert_eq!(protocol_for(RendererChoice::Auto, &silent), Protocol::None);
}

#[test]
fn a_sixel_client_behind_a_second_multiplexer_is_still_text() {
    for extra in [("ZELLIJ", "0"), ("STY", "1.pts-0.host"), ("HERDR_ENV", "1")] {
        let mut pairs = TMUX_IN_FOOT.to_vec();
        pairs.push(extra);
        let get = with_client_features(&pairs, Some(FOOT_CLIENT_FEATURES));
        assert_eq!(
            protocol_for(RendererChoice::Auto, &get),
            Protocol::None,
            "{}",
            extra.0
        );
    }
}

#[test]
fn the_client_features_of_tmux_are_not_asked_outside_tmux() {
    // A foot that is not inside tmux keeps its own verdict (`TERM=foot`),
    // whatever a stray answer says.
    let pairs = [("TERM", "foot")];
    let get = with_client_features(&pairs, Some(KITTY_CLIENT_FEATURES));
    assert_eq!(protocol_for(RendererChoice::Auto, &get), Protocol::Sixel);
}

/// Forced pictures in tmux inside kitty: the Kitty protocol, which is now
/// written inside a passthrough instead of bare (a bare APC was dropped and
/// the graph column stayed blank).
#[test]
fn forcing_pictures_in_tmux_inside_kitty_picks_the_kitty_protocol() {
    let mut pairs = TMUX_IN_FOOT.to_vec();
    pairs.extend([("KITTY_WINDOW_ID", "1")]);
    let get = with_client_features(&pairs, Some(KITTY_CLIENT_FEATURES));
    assert_eq!(protocol_for(RendererChoice::Image, &get), Protocol::Kitty);
    // Auto stays conservative: it does not know `allow-passthrough`.
    assert_eq!(protocol_for(RendererChoice::Auto, &get), Protocol::None);
}

/// `pairs` plus what the start-up probe heard (XTVERSION text, device
/// attributes parameters).
fn with_probe<'a>(
    pairs: &'a [(&'a str, &'a str)],
    version: Option<&'a str>,
    attributes: Option<&'a str>,
) -> impl Fn(&str) -> Option<String> + 'a {
    move |name: &str| match name {
        super::super::detect::PROBE_VERSION_KEY => version.map(str::to_string),
        super::super::detect::PROBE_ATTRIBUTES_KEY => attributes.map(str::to_string),
        _ => pairs
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| (*v).to_string()),
    }
}

const XTERM_ENV: [(&str, &str); 1] = [("TERM", "xterm-256color")];

#[test]
fn an_unnamed_terminal_that_reports_sixel_gets_sixel() {
    // xterm started with `-ti vt340`, a VS Code terminal with images on.
    let get = with_probe(&XTERM_ENV, None, Some("63;4;22"));
    assert_eq!(protocol_for(RendererChoice::Auto, &get), Protocol::Sixel);
}

#[test]
fn an_unnamed_terminal_without_sixel_stays_text() {
    let plain = with_probe(&XTERM_ENV, None, Some("1;2"));
    assert_eq!(protocol_for(RendererChoice::Auto, &plain), Protocol::None);
    let silent = with_probe(&XTERM_ENV, None, None);
    assert_eq!(protocol_for(RendererChoice::Auto, &silent), Protocol::None);
}

#[test]
fn a_terminal_reached_over_ssh_is_recognized_by_its_name() {
    // `TERM_PROGRAM` is not forwarded by ssh: only XTVERSION is left.
    let pairs = [("TERM", "xterm-256color"), ("SSH_CONNECTION", "a 1 b 22")];
    let kitty = with_probe(&pairs, Some("kitty(0.35.2)"), Some("62;c"));
    assert_eq!(protocol_for(RendererChoice::Auto, &kitty), Protocol::Kitty);
    let wezterm = with_probe(&pairs, Some("WezTerm 20240203-110809"), Some("1;2"));
    assert_eq!(
        protocol_for(RendererChoice::Auto, &wezterm),
        Protocol::Iterm2
    );
    // A name that says nothing about pictures leaves the attributes to decide.
    let xterm = with_probe(&pairs, Some("XTerm(388)"), Some("64;4"));
    assert_eq!(protocol_for(RendererChoice::Auto, &xterm), Protocol::Sixel);
}

#[test]
fn the_probe_never_overrides_a_multiplexer_veto() {
    // tmux claims sixel in its own attributes reply whatever the client is.
    for extra in [
        ("TMUX", "/tmp/tmux-1000/default,1,0"),
        ("ZELLIJ", "0"),
        ("STY", "1.pts-0.host"),
        ("HERDR_ENV", "1"),
    ] {
        let mut pairs = XTERM_ENV.to_vec();
        pairs.push(extra);
        let get = with_probe(&pairs, Some("kitty(0.35.2)"), Some("1;2;4"));
        assert_eq!(
            protocol_for(RendererChoice::Auto, &get),
            Protocol::None,
            "{}",
            extra.0
        );
    }
}

#[test]
fn the_environment_wins_over_the_probe() {
    // A kitty window keeps the Kitty protocol whatever the attributes say.
    let pairs = [("KITTY_WINDOW_ID", "1"), ("TERM", "xterm-kitty")];
    let get = with_probe(&pairs, Some("foot(1.28)"), Some("1;2;4"));
    assert_eq!(protocol_for(RendererChoice::Auto, &get), Protocol::Kitty);
}

#[test]
fn text_stays_text_when_the_user_says_so() {
    let get = with_probe(&XTERM_ENV, None, Some("63;4;22"));
    assert_eq!(protocol_for(RendererChoice::Text, &get), Protocol::None);
}
