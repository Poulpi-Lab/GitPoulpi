//! The "Compose commits with AI" button of the commit composer shrinks with
//! the right pane instead of vanishing: the full label, a short one, then the
//! sparkle alone, which stays clickable. Driven through a pty with a fake
//! local agent, resizing the terminal.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::fake_agent::{ai_config_toml, write_fake_agent};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

const PLAN: &str =
    r#"[{"message":"feat: a","files":["a.txt"]},{"message":"fix: b","files":["b.txt"]}]"#;

const SPARK: &str = "\u{2726}";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .write_worktree("b.txt", b"bee\n")
        .unwrap()
        .build()
}

/// The row of the commit-options line (the one with the Amend checkbox), or
/// `None` when no such row is on screen.
fn options_row(screen: &Screen) -> Option<(u16, String)> {
    let (_, row) = screen.find("[ ] Amend")?;
    Some((row, screen.line(row)))
}

#[test]
fn compose_shrinks_to_the_icon_and_stays_clickable() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &[PLAN]);
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();
    let mut term: Terminal = common::launch(repo.root())
        .session("pty-compose-narrow")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded, WIP row and all", STARTUP, |s| {
        s.contains("initial commit") && s.contains("// WIP")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Wide: the full label.
    term.wait_for("the full label beside Amend", STEP, |s| {
        options_row(s).is_some_and(|(_, line)| line.contains("Compose commits with AI"))
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Narrow: the label is gone, the sparkle is not.
    let mut seen_icon_only = false;
    for cols in [110u16, 90, 75, 60] {
        term.resize(cols, 40).expect("resize");
        let screen = term
            .wait_for_stable(Duration::from_millis(400), STEP)
            .unwrap_or_else(|e| panic!("{e}"));
        let Some((_, line)) = options_row(&screen) else {
            panic!(
                "no commit options row at {cols} columns:\n{}",
                screen.text()
            );
        };
        assert!(
            line.contains(SPARK),
            "the Compose button vanished at {cols} columns: {line:?}\n{}",
            screen.text()
        );
        if !line.contains("Compose") {
            seen_icon_only = true;
        }
    }
    assert!(seen_icon_only, "the button never shrank to its icon");

    // The icon alone is still the button: clicking it starts the compose.
    let screen = term.screen();
    let (row, line) = options_row(&screen).expect("options row");
    let col = line
        .chars()
        .position(|c| c.to_string() == SPARK)
        .expect("the icon") as u16;
    term.click(col, row).expect("click the icon");
    term.wait_for("the compose review", STEP, |s| {
        s.contains("feat: a") && s.contains("fix: b")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
