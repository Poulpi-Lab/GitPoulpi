//! `ui.graph_lane_background`: `off`, `row_tint`, one test each,
//! driven through the real binary in a pty with a temp config dir (the
//! pty has no pixels, so this is the text renderer; the image renderers'
//! transparency is covered by the unit tests on the bitmap alpha and by
//! the real-window captures in `docs/developer/visual-verification.md`).
//!
//! The old spellings must still load: `true` is `row_tint`, `false` is `off`,
//! and `lane` / `subtle` / `strong` migrate to `row_tint`.

mod common;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Color, Screen, Terminal};

const MESSAGE: &str = "lane background probe commit";
/// The older commit's row, which the Log shows in full while the detail
/// pane carries the selected commit's message too.
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

fn launch(repo: &FixtureRepo, session: &str, setting: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .config(format!("[ui]\ngraph_lane_background = {setting}\n"))
        .size(200, 60)
        .ready_when("the fixture loaded", |s| s.contains(MESSAGE))
        .start()
}

/// What a probe run measured on the Log row of `ROW_TEXT`.
struct Row {
    /// The background under the message text.
    message_bg: Color,
    /// The backgrounds of the non-blank cells between the ref column and
    /// the message: the lane markers.
    graph_bgs: Vec<Color>,
}

fn measure(screen: &Screen) -> Row {
    let (msg_col, row) = screen.find(ROW_TEXT).expect("the commit row");
    let graph_bgs = (msg_col.saturating_sub(8)..msg_col)
        .filter_map(|c| screen.cell(c, row))
        .filter(|cell| {
            "\u{2502}\u{25cf}\u{25cb}\u{25c9}\u{2503}\u{253c}".contains(cell.text.as_str())
                && !cell.text.is_empty()
        })
        .map(|cell| cell.bg)
        .collect();
    Row {
        message_bg: screen.cell(msg_col, row).expect("a cell").bg,
        graph_bgs,
    }
}

fn run(setting: &str, session: &str) -> Row {
    let repo = fixture();
    let (term, _config) = launch(&repo, session, setting);
    let screen = term.screen();
    let row = measure(&screen);
    let _ = term.stop();
    row
}

#[test]
fn off_paints_no_background_behind_the_lanes_or_the_row() {
    let row = run("\"off\"", "lane-bg-off");
    assert!(!row.graph_bgs.is_empty(), "no lane marker found");
    assert!(
        row.graph_bgs.iter().all(|bg| *bg == row.message_bg),
        "off: lane cells must share the row's own background: {:?} vs {:?}",
        row.graph_bgs,
        row.message_bg
    );
    assert!(matches!(row.message_bg, Color::Default | Color::Rgb(..)));
}

#[test]
fn row_tint_tints_the_graph_from_the_node_rightwards_and_not_the_message() {
    let off = run("\"off\"", "lane-bg-off3");
    let row = run("\"row_tint\"", "lane-bg-tint");
    assert_eq!(
        row.message_bg, off.message_bg,
        "row_tint: the message column stays untinted"
    );
    assert!(!row.graph_bgs.is_empty(), "no lane marker found");
    assert!(
        row.graph_bgs
            .iter()
            .all(|bg| matches!(bg, Color::Rgb(..)) && *bg != row.message_bg),
        "row_tint: the node's cell carries the commit's tint: {:?}",
        row.graph_bgs
    );
}

#[test]
fn the_old_spellings_still_load() {
    let lane = run("\"row_tint\"", "lane-bg-lane2");
    for (setting, session) in [
        ("true", "lane-bg-true"),
        ("\"lane\"", "lane-bg-lane"),
        ("\"subtle\"", "lane-bg-subtle"),
        ("\"strong\"", "lane-bg-strong"),
    ] {
        let row = run(setting, session);
        assert_eq!(row.graph_bgs, lane.graph_bgs, "{setting}");
    }
    let off = run("\"off\"", "lane-bg-off4");
    assert_eq!(run("false", "lane-bg-false").graph_bgs, off.graph_bgs);
}
