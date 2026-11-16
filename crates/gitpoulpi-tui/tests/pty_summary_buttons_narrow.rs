//! The Generate and `#` buttons of the commit composer's summary row shrink
//! to glyphs and then fold into one `⋯` menu as the right pane narrows; the
//! menu still runs them. Driven through a pty with a fake local agent,
//! resizing the terminal.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::fake_agent::{ai_config_toml, write_fake_agent};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

const PLAN: &str = r#"[{"message":"feat: a","files":["a.txt"]}]"#;
const SPARK: &str = "\u{2726}";
const MORE: &str = "\u{22EF}";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .build()
}

/// The summary row: two below the commit-options row ("[ ] Am", the Amend label may be clipped), past the box's
/// top border.
fn summary_row(screen: &Screen) -> Option<(u16, String)> {
    let (_, row) = screen.find("[ ] Am")?;
    Some((row + 2, screen.line(row + 2)))
}

#[test]
fn generate_folds_into_the_more_menu_and_stays_reachable() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &[PLAN]);
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();
    let mut term: Terminal = common::launch(repo.root())
        .session("pty-summary-buttons-narrow")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded, WIP row and all", STARTUP, |s| {
        s.contains("initial commit") && s.contains("// WIP")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for("the Generate glyph on the summary row", STEP, |s| {
        summary_row(s).is_some_and(|(_, line)| line.contains(SPARK))
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Narrow the terminal until the glyph folds into the `⋯`; at every width
    // one of the two is on the summary row.
    let mut folded = None;
    for cols in [110u16, 90, 75, 60, 50, 44, 40, 36, 32, 30, 28, 26] {
        term.resize(cols, 40).expect("resize");
        term.wait_for(&format!("a redraw at {cols} columns"), STEP, |s| {
            summary_row(s).is_some_and(|(_, line)| line.contains(SPARK) || line.contains(MORE))
        })
        .unwrap_or_else(|e| panic!("Generate vanished at {cols} columns: {e}"));
        let screen = term
            .wait_for_stable(Duration::from_millis(400), STEP)
            .unwrap_or_else(|e| panic!("{e}"));
        let (row, line) = summary_row(&screen).expect("summary row");
        assert!(
            line.contains(SPARK) || line.contains(MORE),
            "Generate vanished at {cols} columns: {line:?}\n{}",
            screen.text()
        );
        if line.contains(MORE) {
            let col = line.chars().position(|c| c.to_string() == MORE).unwrap() as u16;
            folded = Some((col, row));
            break;
        }
    }
    let (col, row) = folded.expect("the buttons never folded into the more menu");
    term.click(col, row).expect("click the more button");
    term.wait_for("the menu listing Generate", STEP, |s| {
        s.contains("Generate a commit message with AI")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
