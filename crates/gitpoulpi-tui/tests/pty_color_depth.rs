//! `ui.color_depth` and `ui.theme = system` detection, against the real
//! binary in a pty: a 256-color setting emits no truecolor SGR at all, and
//! a terminal that answers the OSC 11 background query with a light color
//! gets the Light theme (silence, or a dark answer, keeps Dark), with the
//! reply fully consumed (nothing printed, nothing left in the input).

mod common;

use gitpoulpi_config::theme::{preset, ThemeColor};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Color, Screen};

const MESSAGE: &str = "color depth probe";
const ROW_TEXT: &str = "an older commit";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("an older commit under the probe")
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build()
}

fn has(raw: &[u8], needle: &str) -> bool {
    String::from_utf8_lossy(raw).contains(needle)
}

fn log_bg(screen: &Screen) -> Color {
    let (col, row) = screen.find(ROW_TEXT).expect("the older commit row");
    screen.cell(col, row).expect("a cell").bg
}

#[test]
fn a_256_color_setting_emits_no_truecolor_sgr() {
    let repo = fixture();
    let (term, _config) = common::launch(repo.root())
        .session("depth-256")
        .config("[ui]\ntheme = \"nord\"\ncolor_depth = \"256\"\n")
        .size(200, 60)
        .ready(MESSAGE)
        .start();
    let raw = term.raw_output();
    assert!(!has(&raw, "38;2;"), "truecolor foreground leaked");
    assert!(!has(&raw, "48;2;"), "truecolor background leaked");
    assert!(has(&raw, "38;5;") || has(&raw, "48;5;"), "indexed colors");
}

#[test]
fn truecolor_still_emits_truecolor_sgr() {
    // The control for the test above: the same theme, truecolor.
    let repo = fixture();
    let (term, _config) = common::launch(repo.root())
        .session("depth-true")
        .config("[ui]\ntheme = \"nord\"\ncolor_depth = \"truecolor\"\n")
        .size(200, 60)
        .ready(MESSAGE)
        .start();
    let raw = term.raw_output();
    assert!(has(&raw, "48;2;"), "the Nord background is truecolor");
}

#[cfg_attr(
    windows,
    ignore = "the OSC 11 probe reads /dev/tty: it never queries on Windows (COLORFGBG, then Dark)"
)]
#[test]
fn system_follows_a_light_terminal_background() {
    let repo = fixture();
    let ThemeColor::Rgb(r, g, b) = preset("light").expect("light").ui.bg else {
        panic!("Light's bg is RGB");
    };
    let (term, _config) = common::launch(repo.root())
        .session("system-light")
        .answer_background("rgb:ffff/ffff/ffff")
        .size(200, 60)
        .ready_all(&[MESSAGE, ROW_TEXT])
        .start();
    let screen = term.screen();
    assert_eq!(log_bg(&screen), Color::Rgb(r, g, b), "Light was painted");
    assert!(
        !screen.contains("rgb:ffff") && !screen.contains("11;rgb"),
        "the reply must not reach the screen"
    );
}

#[test]
fn system_stays_dark_for_a_dark_answer_and_for_silence() {
    for (session, answer) in [
        ("system-dark", Some("rgb:0000/0000/0000")),
        ("system-silent", None),
    ] {
        let repo = fixture();
        let mut launch = common::launch(repo.root())
            .session(session)
            .size(200, 60)
            .ready_all(&[MESSAGE, ROW_TEXT]);
        if let Some(rgb) = answer {
            launch = launch.answer_background(rgb);
        }
        let (term, _config) = launch.start();
        assert_eq!(
            log_bg(&term.screen()),
            Color::Default,
            "{session}: Dark leaves the terminal's own background"
        );
    }
}
