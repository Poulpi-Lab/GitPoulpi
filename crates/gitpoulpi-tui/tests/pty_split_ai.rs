#![cfg(feature = "ai")]
//! "Split commit with AI" end to end through a pty (`xtask::pty`): the real
//! binary, daemon and git, and a fake local agent standing in for the model
//! (no socket is ever opened). The ✦ row of the commit menu and the palette
//! entry open the split editor prefilled with the agent's plan; a plan that
//! cannot be used still opens the editor, with the reason.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the state it waits for.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn numbered(change_first: bool, change_last: bool) -> String {
    (1..=20)
        .map(|n| match (n, change_first, change_last) {
            (1, true, _) => "line 01 changed\n".to_string(),
            (20, _, true) => "line 20 changed\n".to_string(),
            _ => format!("line {n:02}\n"),
        })
        .collect()
}

/// `base`, then `parser and flag`: two hunks of `a.txt` (H1, H2), a new
/// `b.txt` (F3) and a new file named after the commit (F4); then a tip.
fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", &numbered(false, false), "base")
        .unwrap()
        .write_worktree("a.txt", numbered(true, true).as_bytes())
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .write_worktree("b.txt", b"bee\n")
        .unwrap()
        .stage("b.txt")
        .unwrap()
        .commit("parser and flag")
        .unwrap()
        .commit("tip on top")
        .unwrap()
        .build()
}

fn subjects(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["log", "--format=%s"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn wait_for_subjects(repo: &FixtureRepo, wanted: &[&str]) {
    let deadline = Instant::now() + STEP;
    while subjects(repo) != wanted {
        assert!(
            Instant::now() < deadline,
            "history is {:?}, wanted {wanted:?}",
            subjects(repo)
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn launch(repo: &FixtureRepo, session: &str, agent: &Path) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .config(ai_config_toml(agent))
        .size(200, 50)
        .ready_when("the fixture loaded", |s| {
            s.contains("tip on top") && s.contains("parser and flag")
        })
        .start()
}

/// The agent's plan for the fixture: the first hunk and the commit-named
/// file in one commit, the second hunk and `b.txt` in the other.
fn good_plan(dir: &Path) -> std::path::PathBuf {
    write_fake_agent(
        dir,
        &[
            r#"[{"message": "first part", "pieces": ["H1", "F4"]}, {"message": "second part", "pieces": ["H2", "F3"]}]"#,
        ],
    )
}

fn wait_editor(term: &Terminal) {
    term.wait_for("the split editor with its changes", STEP, |s| {
        s.contains("New commits") && s.contains("b.txt") && s.contains("Split into")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// How many `[ n v]` assignment fields show the number `n`.
fn owned_by(term: &Terminal, n: u8) -> usize {
    term.screen().find_all(&format!(" {n} \u{25be} ")).len()
}

/// Right-click `parser and flag` and click `row` of its menu.
fn menu_click(term: &mut Terminal, row_text: &str) {
    let (col, row) = term.wait_for_text("parser and flag", STEP).unwrap();
    term.right_click(col + 1, row)
        .expect("right-click the commit");
    let menu = term
        .wait_for("the commit menu", STEP, |s| s.contains(row_text))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find(row_text).unwrap();
    term.click(c + 1, r).expect("click the menu row");
}

/// The ✦ menu row asks the agent once, opens the editor on its plan, and
/// the plan splits the history as written; the editor never opened before
/// the click.
#[test]
fn the_ai_menu_row_opens_the_editor_prefilled_and_the_plan_splits() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = good_plan(tools.path());
    let tree = repo.git(&["rev-parse", "HEAD^{tree}"]);
    let (mut term, _config) = launch(&repo, "pty-split-ai-menu", &agent);
    assert_eq!(agent_calls(&agent), 0, "nothing asks the AI on its own");

    menu_click(&mut term, "Split commit (AI)...");
    wait_editor(&term);
    term.wait_for("the plan's messages in the commits pane", STEP, |s| {
        s.contains("1 first part") && s.contains("2 second part")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 1);
    assert_eq!(owned_by(&term, 1), 2, "H1 and F4 are the first commit's");
    assert_eq!(owned_by(&term, 2), 2, "H2 and F3 are the second's");
    assert_eq!(subjects(&repo).len(), 3, "nothing is split before Split");

    term.send_keys("C-s").unwrap();
    term.wait_for("the editor closed", STEP, |s| !s.contains("Split into 2"))
        .unwrap_or_else(|e| panic!("{e}"));
    wait_for_subjects(&repo, &["tip on top", "second part", "first part", "base"]);
    assert_eq!(repo.git(&["rev-parse", "HEAD^{tree}"]), tree);
    // The Log selects the newest commit the split made.
    term.wait_for("the new commit selected in the details", STEP, |s| {
        s.contains("second part")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The palette entry does the same as the menu row.
#[test]
fn the_ai_palette_entry_opens_the_editor_prefilled() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = good_plan(tools.path());
    let (mut term, _config) = launch(&repo, "pty-split-ai-palette", &agent);
    let (col, row) = term.wait_for_text("parser and flag", STEP).unwrap();
    term.click(col + 1, row).expect("select the commit");
    term.send_keys("C-p").unwrap();
    term.send_text("ai: split").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("AI: Split Selected Commit...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    wait_editor(&term);
    term.wait_for("the plan's messages", STEP, |s| {
        s.contains("1 first part") && s.contains("2 second part")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// An answer that is no plan: the editor still opens, on the plain starting
/// plan, with the reason; the manual split works.
#[test]
fn a_useless_answer_still_offers_the_manual_editor_with_the_reason() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["I could not decide"]);
    let (mut term, _config) = launch(&repo, "pty-split-ai-failed", &agent);

    menu_click(&mut term, "Split commit (AI)...");
    wait_editor(&term);
    term.wait_for("the reason in the editor", STEP, |s| {
        s.contains("could not propose a split")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Asked twice: the first answer and the retry.
    assert_eq!(agent_calls(&agent), 2);
    assert_eq!(owned_by(&term, 1), 5, "everything is in the first commit");
    // The manual way still works: the second hunk goes to a second commit.
    term.send_keys("Down").unwrap();
    term.send_keys("Down").unwrap();
    term.send_text("2").unwrap();
    term.wait_for("one change moved by hand", STEP, |s| {
        s.find_all(" 2 \u{25be} ").len() == 1
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// With AI off the ✦ row does not exist.
#[test]
fn without_ai_there_is_no_ai_split_row() {
    let repo = fixture();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-split-ai-off")
        .size(200, 50)
        .ready_when("the fixture loaded", |s| {
            s.contains("tip on top") && s.contains("parser and flag")
        })
        .start();
    let (col, row) = term.wait_for_text("parser and flag", STEP).unwrap();
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the commit menu", STEP, |s| s.contains("Split commit..."))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!menu.contains("Split commit (AI)"));
}
