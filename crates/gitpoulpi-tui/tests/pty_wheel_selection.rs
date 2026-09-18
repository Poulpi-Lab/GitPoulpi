//! The wheel scrolls, it never selects: the Settings search popup and the
//! Compose Commits review scroll their window and leave the selection where
//! it was (it may scroll out of view); a key move brings the window back to
//! the selection. The search popup also paints the shared scrollbar.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::fake_agent::{ai_config_toml, write_fake_agent};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};

const STEP: Duration = Duration::from_secs(30);
const POPUP: &str = "Enter: go";
const ARROW_UP: &str = "\u{2191}";

#[test]
fn the_wheel_over_the_settings_search_scrolls_the_window_not_the_selection() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("search wheel probe")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-wheel-settings-search")
        .config_dir(config.path())
        .size(200, 50)
        .no_exit_pause()
        .ready("search wheel probe")
        .mouse_capture()
        .spawn();
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-f").unwrap();
    term.wait_for("the popup", STEP, |s| s.contains(POPUP))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("e").unwrap();
    let s = term
        .wait_for("many results, the scrollbar painted", STEP, |s| {
            s.find(POPUP).is_some_and(|(_, r)| {
                (r + 1..r + 12).any(|row| {
                    s.cell(right_edge(s, r), row)
                        .is_some_and(|cell| cell.display() == ARROW_UP)
                })
            })
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (pc, pr) = s.find(POPUP).unwrap();
    let first = s.line(pr + 1);
    term.scroll(true, pc + 4, pr + 3).unwrap();
    term.wait_for("the window scrolled", STEP, |s| s.line(pr + 1) != first)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The selection stayed on the first hit: one key move brings it back in
    // view, on the second row of the unscrolled window.
    term.send_keys("Down").unwrap();
    term.wait_for("the window follows the key move", STEP, |s| {
        s.line(pr + 1) == first
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn right_edge(s: &xtask::pty::Screen, title_row: u16) -> u16 {
    (0..s.cols())
        .rev()
        .find(|c| {
            s.cell(*c, title_row)
                .is_some_and(|cell| cell.display() == "\u{256e}")
        })
        .expect("the popup's top-right corner")
}

const COUNT: usize = 60;

fn compose_fixture() -> (FixtureRepo, String) {
    let mut b = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("base.txt", "base\n", "initial commit")
        .unwrap();
    let mut plan = Vec::new();
    for i in 0..COUNT {
        let f = format!("f{i:02}.txt");
        b = b.write_worktree(&f, b"x\n").unwrap();
        plan.push(format!(r#"{{"message":"feat: n{i:02}","files":["{f}"]}}"#));
    }
    (b.build(), format!("[{}]", plan.join(",")))
}

#[test]
fn the_wheel_over_the_compose_review_scrolls_the_list_not_the_selection() {
    let (repo, plan) = compose_fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &[&plan]);
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "{}\n[ui]\ncomposer_files = \"collapsed\"\n",
            ai_config_toml(&agent)
        ),
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-wheel-compose")
        .config_dir(config.path())
        .size(200, 50)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    term.wait_for("the fixture loaded", STEP, |s| {
        s.contains("initial commit") && s.contains("// WIP")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.send_text("compose commits").unwrap();
    term.wait_for("the entry offered", STEP, |s| {
        s.contains("AI: Compose Commits from Working Changes")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    let s = term
        .wait_for("the review", STEP, |s| {
            s.contains("feat: n00") && s.contains("Commit 1 \u{b7}")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = s.find("feat: n00").unwrap();
    for _ in 0..3 {
        term.scroll(true, c + 2, r).unwrap();
    }
    term.wait_for("the list scrolled away from the first row", STEP, |s| {
        !s.contains("feat: n00")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        term.screen().contains("Commit 1 \u{b7}"),
        "the selection did not move:\n{}",
        term.text()
    );
    term.send_keys("Down").unwrap();
    term.wait_for("the key move shows the selection again", STEP, |s| {
        s.contains("feat: n00") && s.contains("Commit 2 \u{b7}")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}
