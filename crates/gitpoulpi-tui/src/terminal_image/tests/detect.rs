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
