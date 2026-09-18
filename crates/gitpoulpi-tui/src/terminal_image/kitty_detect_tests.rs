//! When the Kitty protocol is picked, and what a Kitty
//! picture does to the ratatui buffer. Detection is a pure function of an
//! environment and a file reader, so every case is a call, never a
//! mutation of this process' environment or a read of the real disk.

use super::*;

fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let map: std::collections::HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect();
    move |name: &str| map.get(name).cloned()
}

/// A real WezTerm pane on Windows.
const WEZTERM: [(&str, &str); 3] = [
    (
        "WEZTERM_EXECUTABLE",
        r"C:\Program Files\WezTerm\wezterm-gui.exe",
    ),
    ("TERM_PROGRAM", "WezTerm"),
    ("TERM", "xterm-256color"),
];

/// A herdr 0.9.1 pane inside that WezTerm, as measured on 2026-09-23.
fn wezterm_inside_herdr() -> Vec<(&'static str, &'static str)> {
    let mut pairs = WEZTERM.to_vec();
    pairs.extend([
        ("HERDR_ENV", "1"),
        ("HERDR_PANE_ID", "w4:p1"),
        (
            "HERDR_SOCKET_PATH",
            r"C:\Users\me\AppData\Roaming\herdr\herdr.sock",
        ),
        ("APPDATA", r"C:\Users\me\AppData\Roaming"),
    ]);
    pairs
}

/// herdr's config as the maintainer's reads (2026-09-25).
const HERDR_KITTY_ON: &str = "onboarding = false\n\n[experimental]\nkitty_graphics = true\n";

/// Detection with herdr's config file answering `config` wherever it is
/// looked for.
fn verdict(
    choice: RendererChoice,
    wanted: ImageProtocol,
    pairs: &[(&str, &str)],
    config: Option<&str>,
) -> Protocol {
    let config = config.map(str::to_string);
    protocol_for(choice, wanted, &env(pairs), &|_: &std::path::Path| {
        config.clone()
    })
}

fn auto(pairs: &[(&str, &str)], config: Option<&str>) -> Protocol {
    verdict(RendererChoice::Auto, ImageProtocol::Auto, pairs, config)
}

#[test]
fn herdr_with_kitty_graphics_on_in_wezterm_gets_the_kitty_protocol() {
    assert_eq!(
        auto(&wezterm_inside_herdr(), Some(HERDR_KITTY_ON)),
        Protocol::Kitty
    );
}

/// Falsified: treated a missing switch as on (the missing-config cases
/// went red); restored.
#[test]
fn herdr_without_kitty_graphics_keeps_the_text_rendering() {
    for config in [
        None,
        Some("onboarding = false\n"),
        Some("[experimental]\nkitty_graphics = false\n"),
        // `[terminal]` wins over the legacy `[experimental]`.
        Some("[terminal]\nkitty_graphics = false\n[experimental]\nkitty_graphics = true\n"),
    ] {
        assert_eq!(
            auto(&wezterm_inside_herdr(), config),
            Protocol::None,
            "{config:?}"
        );
    }
}

#[test]
fn herdrs_switch_is_read_from_either_section_and_as_a_dotted_key() {
    assert_eq!(
        herdr_kitty_graphics("[terminal]\nkitty_graphics = true # on\n"),
        Some(true)
    );
    assert_eq!(
        herdr_kitty_graphics("terminal.kitty_graphics = true\n"),
        Some(true)
    );
    assert_eq!(
        herdr_kitty_graphics("[experimental]\nkitty_graphics = true\n"),
        Some(true)
    );
    assert_eq!(
        herdr_kitty_graphics("[ui]\nkitty_graphics = true\n"),
        None,
        "only herdr's own two sections count"
    );
}

#[test]
fn herdr_in_a_terminal_that_draws_no_kitty_images_keeps_text() {
    // Windows Terminal around herdr: herdr's docs say it does not
    // expose the Kitty path.
    let pairs = [
        ("HERDR_ENV", "1"),
        ("WT_SESSION", "abc"),
        ("TERM", "xterm-256color"),
        ("APPDATA", r"C:\Users\me\AppData\Roaming"),
    ];
    assert_eq!(auto(&pairs, Some(HERDR_KITTY_ON)), Protocol::None);
}

#[test]
fn herdrs_config_is_looked_for_where_herdr_keeps_it() {
    let paths = herdr_config_paths(&env(&[
        ("APPDATA", r"C:\Users\me\AppData\Roaming"),
        ("HOME", "/home/me"),
    ]));
    let paths: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        paths,
        [
            "C:/Users/me/AppData/Roaming/herdr/config.toml",
            "/home/me/.config/herdr/config.toml"
        ]
    );
}

#[test]
fn kitty_and_ghostty_get_the_kitty_protocol_natively() {
    for pairs in [
        vec![("KITTY_WINDOW_ID", "1"), ("TERM", "xterm-kitty")],
        vec![("TERM", "xterm-kitty")],
        vec![("TERM_PROGRAM", "ghostty"), ("TERM", "xterm-ghostty")],
        vec![("GHOSTTY_RESOURCES_DIR", "/usr/share/ghostty")],
    ] {
        assert_eq!(auto(&pairs, None), Protocol::Kitty, "{pairs:?}");
    }
}

/// A kitty window started from a shell that itself ran inside a herdr
/// pane (a launcher, a keybinding, an agent): the HERDR_* variables leak
/// in, but kitty sets `TERM=xterm-kitty` for its own window and herdr
/// sets `TERM` for its panes. Detection used to take the leak for a herdr
/// pane, find no `kitty_graphics` in herdr's config and pick iTerm2 --
/// which kitty does not draw: the Graph column stayed blank.
const KITTY_WITH_LEAKED_HERDR: [(&str, &str); 5] = [
    ("KITTY_WINDOW_ID", "1"),
    ("TERM", "xterm-kitty"),
    ("HERDR_ENV", "1"),
    ("HERDR_PANE_ID", "wF:p2"),
    ("HERDR_SOCKET_PATH", "/home/me/.config/herdr/herdr.sock"),
];

#[test]
fn a_kitty_window_with_leaked_herdr_variables_still_gets_kitty() {
    assert_eq!(auto(&KITTY_WITH_LEAKED_HERDR, None), Protocol::Kitty);
    assert_eq!(
        verdict(
            RendererChoice::Image,
            ImageProtocol::Auto,
            &KITTY_WITH_LEAKED_HERDR,
            None
        ),
        Protocol::Kitty
    );
}

#[test]
fn kitty_around_a_real_herdr_pane_is_still_decided_by_herdrs_config() {
    // A real pane: herdr's own TERM, kitty's variables only leaked in.
    let pairs = [
        ("KITTY_WINDOW_ID", "1"),
        ("TERM", "xterm-256color"),
        ("HERDR_ENV", "1"),
        ("HOME", "/home/me"),
    ];
    assert_eq!(auto(&pairs, None), Protocol::None);
    assert_eq!(auto(&pairs, Some(HERDR_KITTY_ON)), Protocol::Kitty);
}

#[test]
fn bare_wezterm_stays_on_iterm2() {
    assert_eq!(auto(&WEZTERM, None), Protocol::Iterm2Alpha);
}

#[test]
fn kitty_inside_tmux_is_still_denied() {
    let pairs = [("KITTY_WINDOW_ID", "1"), ("TMUX", "/tmp/tmux,1,0")];
    assert_eq!(auto(&pairs, None), Protocol::None);
}

/// Read from `/proc/<pid>/environ` of a gitpoulpi in a herdr 0.9.x pane
/// inside kitty 0.48.2 on Linux: herdr strips `KITTY_WINDOW_ID` and
/// leaves the other kitty variables.
const HERDR_PANE_IN_KITTY: [(&str, &str); 12] = [
    ("TERM", "xterm-256color"),
    ("TERM_PROGRAM", "herdr"),
    ("HERDR_ENV", "1"),
    ("HERDR_PANE_ID", "wF:p2"),
    ("HERDR_SOCKET_PATH", "/home/me/.config/herdr/herdr.sock"),
    ("KITTY_PID", "1234"),
    ("KITTY_LISTEN_ON", "unix:@mykitty"),
    ("KITTY_PUBLIC_KEY", "1:abc"),
    ("KITTY_INSTALLATION_DIR", "/usr/lib/kitty"),
    ("TERMINFO", "/usr/lib/kitty/terminfo"),
    ("HOME", "/home/me"),
    ("XDG_CONFIG_HOME", "/home/me/.config"),
];

const HERDR_TERMINAL_ON: &str = "[terminal]\nnew_cwd = \"follow\"\nkitty_graphics = true\n";

#[test]
fn herdr_pane_in_kitty_without_window_id_gets_kitty() {
    assert_eq!(
        auto(&HERDR_PANE_IN_KITTY, Some(HERDR_TERMINAL_ON)),
        Protocol::Kitty
    );
    // Each leftover alone is enough.
    for name in ["KITTY_PID", "KITTY_INSTALLATION_DIR", "KITTY_LISTEN_ON"] {
        let pairs: Vec<_> = HERDR_PANE_IN_KITTY
            .iter()
            .copied()
            .filter(|(k, _)| !k.starts_with("KITTY_") || *k == name)
            .collect();
        assert_eq!(
            auto(&pairs, Some(HERDR_TERMINAL_ON)),
            Protocol::Kitty,
            "{name}"
        );
    }
}

#[test]
fn herdr_pane_in_kitty_needs_herdrs_switch() {
    for config in [
        None,
        Some("[terminal]\nnew_cwd = \"follow\"\n"),
        Some("[terminal]\nkitty_graphics = false\n"),
    ] {
        assert_eq!(auto(&HERDR_PANE_IN_KITTY, config), Protocol::None);
    }
}

#[test]
fn herdr_pane_without_any_kitty_variable_stays_text() {
    let pairs = [
        ("TERM", "xterm-256color"),
        ("TERM_PROGRAM", "herdr"),
        ("HERDR_ENV", "1"),
        ("HERDR_PANE_ID", "wF:p2"),
        ("HOME", "/home/me"),
    ];
    assert_eq!(auto(&pairs, Some(HERDR_TERMINAL_ON)), Protocol::None);
}

#[test]
fn kitty_leftovers_inside_tmux_stay_text() {
    let pairs = [
        ("KITTY_PID", "1234"),
        ("KITTY_INSTALLATION_DIR", "/usr/lib/kitty"),
        ("TERM", "screen-256color"),
        ("TMUX", "/tmp/tmux,1,0"),
    ];
    assert_eq!(auto(&pairs, None), Protocol::None);
}

#[test]
fn an_explicit_protocol_is_trusted_even_where_detection_says_no() {
    for (wanted, expected) in [
        (ImageProtocol::Kitty, Protocol::Kitty),
        (ImageProtocol::Iterm2, Protocol::Iterm2),
        (ImageProtocol::None, Protocol::None),
    ] {
        // A herdr whose config cannot be found: auto says text.
        assert_eq!(
            verdict(RendererChoice::Auto, wanted, &wezterm_inside_herdr(), None),
            expected
        );
    }
    // Bare WezTerm, told to use Kitty (to compare the two there).
    assert_eq!(
        verdict(RendererChoice::Auto, ImageProtocol::Kitty, &WEZTERM, None),
        Protocol::Kitty
    );
    // `none` wins over a forced picture.
    assert_eq!(
        verdict(RendererChoice::Image, ImageProtocol::None, &WEZTERM, None),
        Protocol::None
    );
}

#[test]
fn text_and_the_kill_switch_win_over_an_explicit_protocol() {
    assert_eq!(
        verdict(RendererChoice::Text, ImageProtocol::Kitty, &WEZTERM, None),
        Protocol::None
    );
    let mut pairs = WEZTERM.to_vec();
    pairs.push(("GITPOULPI_TERMINAL_IMAGE", "off"));
    assert_eq!(
        verdict(RendererChoice::Image, ImageProtocol::Kitty, &pairs, None),
        Protocol::None
    );
}

#[test]
fn forcing_image_picks_kitty_where_kitty_is_what_works() {
    assert_eq!(
        verdict(
            RendererChoice::Image,
            ImageProtocol::Auto,
            &wezterm_inside_herdr(),
            Some(HERDR_KITTY_ON)
        ),
        Protocol::Kitty
    );
    assert_eq!(
        verdict(
            RendererChoice::Image,
            ImageProtocol::Auto,
            &[("TERM", "xterm-kitty")],
            None
        ),
        Protocol::Kitty
    );
    // And iTerm2, as before, everywhere else -- herdr without Kitty too.
    assert_eq!(
        verdict(
            RendererChoice::Image,
            ImageProtocol::Auto,
            &wezterm_inside_herdr(),
            None
        ),
        Protocol::Iterm2
    );
}

/// The buffer test: a Kitty picture's area holds one placeholder per
/// cell, the image id in its foreground, row and column in its
/// diacritics - and, unlike iTerm2's reservation, nothing is `skip`.
#[test]
fn a_kitty_picture_reserves_placeholder_cells_not_skipped_ones() {
    let area = Rect::new(2, 1, 3, 2);
    let mut buf = Buffer::empty(Rect::new(0, 0, 8, 4));
    let encoded = encode_image(Protocol::Kitty, &Bitmap::new(27, 42, (1, 2, 3)), 3, 2)
        .expect("should encode");
    let Encoded::Kitty(image) = &encoded else {
        panic!("not a Kitty picture");
    };
    encoded.reserve(area, &mut buf);
    let color = kitty::id_color(image.id);
    for (row, y) in (area.top()..area.bottom()).enumerate() {
        for (col, x) in (area.left()..area.right()).enumerate() {
            let cell = &buf[(x, y)];
            assert!(!cell.skip, "ratatui must emit placeholder {x},{y}");
            assert_eq!(cell.fg, color, "the id rides in the foreground");
            assert_eq!(
                cell.symbol(),
                kitty::placeholder_symbol(row, col).unwrap(),
                "cell {x},{y}"
            );
        }
    }
    assert_eq!(buf[(5, 1)].symbol(), " ", "outside the area");
    assert!(encoded.sequence().starts_with("\x1b_Ga=T,U=1,f=100,"));
    assert!(encoded
        .sequence()
        .contains(&format!(",i={},c=3,r=2,", image.id)));
    let payload = encoded.sequence().split_once(';').unwrap().1;
    assert!(payload.starts_with("iVBORw0KGgo"), "the payload is a PNG");
}

/// The property placeholders were chosen for: ratatui's own diff puts
/// every placeholder on the wire, one column each, so a popup closing
/// over the picture restores it with no help from this module.
#[test]
fn ratatui_emits_each_placeholder_cell_once_in_its_own_column() {
    let area = Rect::new(1, 0, 4, 3);
    let empty = Buffer::empty(Rect::new(0, 0, 8, 3));
    let mut next = empty.clone();
    let encoded = encode_image(Protocol::Kitty, &Bitmap::new(36, 63, (9, 9, 9)), 4, 3).unwrap();
    encoded.reserve(area, &mut next);
    let emitted: Vec<(u16, u16)> = empty
        .diff(&next)
        .into_iter()
        .map(|(x, y, _)| (x, y))
        .collect();
    let expected: Vec<(u16, u16)> = (0..3).flat_map(|y| (1..5).map(move |x| (x, y))).collect();
    assert_eq!(emitted, expected);
}

/// A pane of `herdr --remote` as measured on 2026-10-06 (herdr 0.9.3):
/// the remote server's panes carry `SSH_CONNECTION`, herdr's own `TERM`,
/// and here also kitty leftovers and a config that turns the switch on --
/// everything that makes a *local* pane pick Kitty. The picture still
/// never showed (the local client had `kitty_graphics` off): a blank Graph
/// column.
fn herdr_remote_pane() -> Vec<(&'static str, &'static str)> {
    vec![
        ("TERM", "xterm-256color"),
        ("TERM_PROGRAM", "herdr"),
        ("HERDR_ENV", "1"),
        ("HERDR_PANE_ID", "w1:p1"),
        ("HERDR_SOCKET_PATH", "/home/me/.config/herdr/herdr.sock"),
        ("KITTY_PID", "4242"),
        ("KITTY_INSTALLATION_DIR", "/usr/lib/kitty"),
        ("SSH_CONNECTION", "10.0.0.2 51000 10.0.0.9 22"),
        ("HOME", "/home/me"),
    ]
}

#[test]
fn herdr_remote_gets_text_under_auto_even_when_everything_looks_kitty() {
    assert_eq!(
        auto(&herdr_remote_pane(), Some(HERDR_KITTY_ON)),
        Protocol::None
    );
    let mut ssh_tty = herdr_remote_pane();
    ssh_tty.retain(|(k, _)| *k != "SSH_CONNECTION");
    ssh_tty.push(("SSH_TTY", "/dev/pts/3"));
    assert_eq!(auto(&ssh_tty, Some(HERDR_KITTY_ON)), Protocol::None);
}

#[test]
fn the_same_pane_off_ssh_still_gets_kitty() {
    let mut local = herdr_remote_pane();
    local.retain(|(k, _)| *k != "SSH_CONNECTION");
    assert_eq!(auto(&local, Some(HERDR_KITTY_ON)), Protocol::Kitty);
}

#[test]
fn herdr_remote_with_explicit_image_choice_gets_kitty_not_iterm2() {
    // herdr carries no OSC 1337; the remote config is unreadable evidence.
    let mut pane = herdr_remote_pane();
    pane.retain(|(k, _)| !k.starts_with("KITTY_"));
    assert_eq!(
        verdict(RendererChoice::Image, ImageProtocol::Auto, &pane, None),
        Protocol::Kitty
    );
    assert_eq!(
        verdict(RendererChoice::Text, ImageProtocol::Auto, &pane, None),
        Protocol::None
    );
}
