//! Commit Composer end to end through a pty (`xtask::pty`): the real
//! binary, daemon and git, and a fake local agent that prints a canned plan.
//! Expanding a proposal, moving a file by drag and drop (from the list and
//! from the right pane), the "Move file to commit..." dropdown (keyboard
//! and palette), and the commits that result: each file in exactly one.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::fake_agent::{ai_config_toml, write_fake_agent};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

const PLAN: &str =
    r#"[{"message":"feat: a","files":["a.txt"]},{"message":"fix: bc","files":["b.txt","c.txt"]}]"#;

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .write_worktree("b.txt", b"bee\n")
        .unwrap()
        .write_worktree("c.txt", b"sea\n")
        .unwrap()
        .build()
}

/// Opens the review on the fixture; `extra_config` is appended to the
/// `config.toml` that turns the fake agent on.
fn open_review(
    repo: &FixtureRepo,
    session: &str,
    extra_config: &str,
) -> (Terminal, tempfile::TempDir, tempfile::TempDir) {
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &[PLAN]);
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        format!("{}\n{extra_config}", ai_config_toml(&agent)),
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded, WIP row and all", STARTUP, |s| {
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
    term.wait_for("the review with the plan", STEP, |s| {
        s.contains("feat: a") && s.contains("fix: bc") && s.contains("Create 2 commits")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config, tools)
}

fn wait_for_head_subject(repo: &FixtureRepo, subject: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while repo.git(&["log", "-1", "--format=%s"]) != subject {
        assert!(
            std::time::Instant::now() < deadline,
            "HEAD never became {subject:?}: {}",
            repo.git(&["log", "--format=%s"])
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn files_of(repo: &FixtureRepo, rev: &str) -> Vec<String> {
    let out = repo.git(&["show", "--name-only", "--format=", rev]);
    let mut files: Vec<String> = out.lines().map(str::to_string).collect();
    files.sort();
    files
}

/// `b.txt` moved to the first commit: it leaves `fix: bc`, and every file
/// is in exactly one of the created commits.
fn assert_b_moved_to_first(repo: &FixtureRepo) {
    wait_for_head_subject(repo, "fix: bc");
    assert_eq!(files_of(repo, "HEAD"), ["c.txt"]);
    assert_eq!(files_of(repo, "HEAD~1"), ["a.txt", "b.txt"]);
    assert_eq!(
        repo.git(&["log", "--format=%s", "-3"]),
        "fix: bc\nfeat: a\ninitial commit"
    );
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
}

/// `ui.composer_files = "collapsed"` opens the proposals closed; Right
/// expands the selected one, Left collapses it again.
#[test]
fn collapsed_proposals_expand_and_collapse() {
    let repo = fixture();
    let (mut term, _c, _t) = open_review(
        &repo,
        "pty-compose-collapse",
        "[ui]\ncomposer_files = \"collapsed\"\n",
    );
    assert!(
        !term.screen().contains("\u{251c}\u{2500} b.txt")
            && !term.screen().contains("\u{2514}\u{2500} b.txt"),
        "collapsed: no inline file rows"
    );
    term.send_keys("Down").unwrap(); // the second proposal
    term.wait_for("commit 2 selected", STEP, |s| {
        s.contains("Commit 2 \u{b7} 2 files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Right").unwrap();
    term.wait_for("its files listed inline", STEP, |s| {
        s.contains("\u{251c}\u{2500} b.txt") && s.contains("\u{2514}\u{2500} c.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Left").unwrap();
    term.wait_for("collapsed again", STEP, |s| {
        !s.contains("\u{251c}\u{2500} b.txt") && s.contains("fix: bc")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // A click on the arrow expands too.
    let screen = term
        .wait_for("the second row's arrow", STEP, |s| {
            s.find_all("\u{25b8}")
                .into_iter()
                .any(|(_, r)| s.line(r).contains("fix: bc"))
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen
        .find_all("\u{25b8}")
        .into_iter()
        .find(|&(_, r)| screen.line(r).contains("fix: bc"))
        .expect("the second row's arrow");
    term.click(col, row).unwrap();
    term.wait_for("expanded by the click", STEP, |s| {
        s.contains("\u{2514}\u{2500} c.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Drag a file row onto another proposal (press, motion, release).
#[test]
fn dragging_a_file_to_another_proposal_moves_it() {
    let repo = fixture();
    let (mut term, _c, _t) = open_review(&repo, "pty-compose-dnd", "");
    // Both ends from one screen, once nothing moves any more.
    let screen = term
        .wait_for("the file row and the target proposal", STEP, |s| {
            s.contains("\u{251c}\u{2500} b.txt") && s.contains("feat: a")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (fc, fr) = screen.find("\u{251c}\u{2500} b.txt").unwrap();
    let (tc, tr) = screen.find("feat: a").unwrap();
    term.press(fc + 4, fr).unwrap();
    term.drag_to(tc + 2, tr).unwrap();
    term.wait_for("the drag announced in the footer", STEP, |s| {
        s.contains("Moving b.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.release(tc + 2, tr).unwrap();
    term.wait_for("b.txt now in commit 1", STEP, |s| {
        s.contains("Commit 1 \u{b7} 2 files") && !s.contains("Moving b.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    assert_b_moved_to_first(&repo);
}

/// Drag a file from the right pane (pane 3) onto another proposal.
#[test]
fn dragging_a_file_from_the_right_pane_moves_it() {
    let repo = fixture();
    let (mut term, _c, _t) = open_review(&repo, "pty-compose-pane-dnd", "");
    let (c, r) = term.wait_for_text("fix: bc", STEP).unwrap();
    term.click(c + 1, r).unwrap();
    let screen = term
        .wait_for("commit 2's files in the right pane", STEP, |s| {
            s.contains("Commit 2 \u{b7} 2 files")
                && s.find("feat: a")
                    .is_some_and(|(tc, _)| s.find_all("b.txt").iter().any(|&(c, _)| c > tc + 40))
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // The pane is the right-most place b.txt is drawn.
    let (pc, pr) = screen
        .find_all("b.txt")
        .into_iter()
        .max_by_key(|&(c, _)| c)
        .unwrap();
    let (tc, tr) = screen.find("feat: a").unwrap();
    assert!(pc > tc + 40, "the pane's b.txt is right of the list");
    term.press(pc + 1, pr).unwrap();
    term.drag_to(tc + 2, tr).unwrap();
    term.release(tc + 2, tr).unwrap();
    term.wait_for("b.txt now in commit 1", STEP, |s| {
        s.contains("Commit 1 \u{b7} 2 files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    assert_b_moved_to_first(&repo);
}

/// The keyboard twin: `m` on a file opens the dropdown, Enter picks the
/// first commit.
#[test]
fn the_move_file_combo_moves_a_file_by_keyboard() {
    let repo = fixture();
    let (mut term, _c, _t) = open_review(&repo, "pty-compose-move-key", "");
    for _ in 0..3 {
        term.send_keys("Down").unwrap(); // a.txt, fix: bc, b.txt
    }
    term.wait_for("b.txt selected", STEP, |s| {
        s.contains("Commit 2 \u{b7} 2 files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("m").unwrap();
    term.wait_for("the dropdown", STEP, |s| {
        s.contains("Move file to commit...")
            && s.contains("1. feat: a")
            && s.contains("A new commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("moved", STEP, |s| {
        s.contains("Commit 1 \u{b7} 2 files") && !s.contains("Move file to commit...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    assert_b_moved_to_first(&repo);
}

/// The palette reaches it too (Ctrl+P inside the review; Esc brings the
/// review back unchanged), and "A new commit" splits the file out.
#[test]
fn the_palette_opens_the_move_combo_and_a_new_commit_is_offered() {
    let repo = fixture();
    let (mut term, _c, _t) = open_review(&repo, "pty-compose-move-palette", "");
    for _ in 0..3 {
        term.send_keys("Down").unwrap();
    }
    term.wait_for("b.txt selected", STEP, |s| {
        s.contains("Commit 2 \u{b7} 2 files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.send_text("move file").unwrap();
    term.wait_for("the entry offered", STEP, |s| {
        s.contains("Composer: Move File to Commit...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    // Wait for what only the closed palette shows: a following Ctrl+P
    // read together with the unread Escape would be Alt+Ctrl+P.
    term.wait_for("the review is back, the palette closed", STEP, |s| {
        s.contains("Create 2 commits") && !s.contains("Composer: Move File to Commit...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.send_text("move file").unwrap();
    term.wait_for("the entry offered", STEP, |s| {
        s.contains("Composer: Move File to Commit...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the dropdown", STEP, |s| {
        s.contains("Move file to commit... ")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap(); // "+ A new commit"
    term.send_keys("Enter").unwrap();
    term.wait_for("three commits", STEP, |s| s.contains("Create 3 commits"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    wait_for_head_subject(&repo, "New commit");
    assert_eq!(files_of(&repo, "HEAD"), ["b.txt"]);
    assert_eq!(files_of(&repo, "HEAD~1"), ["c.txt"]);
    assert_eq!(files_of(&repo, "HEAD~2"), ["a.txt"]);
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
}

/// The move dialog's close cross closes it like Esc: the review is back and
/// nothing moved (the commits created afterwards are the proposed ones).
/// Falsified by dropping the cross hit in `on_move_mouse`.
#[test]
fn the_move_dialog_close_cross_closes_it_and_moves_nothing() {
    let repo = fixture();
    let (mut term, _c, _t) = open_review(&repo, "pty-compose-move-cross", "");
    for _ in 0..3 {
        term.send_keys("Down").unwrap();
    }
    term.wait_for("b.txt selected", STEP, |s| {
        s.contains("Commit 2 \u{b7} 2 files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("m").unwrap();
    term.wait_for("the dropdown", STEP, |s| {
        s.contains("Move file to commit...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::retry_until(
        &mut term,
        "the dialog closed by its cross",
        5,
        Duration::from_secs(6),
        |term| {
            let screen = term.screen();
            if let Some((_, row)) = screen.find("Move file to commit...") {
                // The dialog's title row carries its cross.
                if let Some((c, r)) = screen
                    .find_all("\u{2715}")
                    .into_iter()
                    .find(|(_, r)| *r == row)
                {
                    term.click(c, r).unwrap();
                }
            }
        },
        |s| !s.contains("Move file to commit...") && s.contains("Create 2 commits"),
    );
    term.send_keys("C-s").unwrap();
    wait_for_head_subject(&repo, "fix: bc");
    assert_eq!(files_of(&repo, "HEAD"), ["b.txt", "c.txt"]);
    assert_eq!(files_of(&repo, "HEAD~1"), ["a.txt"]);
}
