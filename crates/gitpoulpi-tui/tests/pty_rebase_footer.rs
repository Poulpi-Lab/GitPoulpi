//! The footer of the interactive rebase and interactive
//! cherry-pick views ("shortcuts:" with one mnemonic per action, Ctrl+Up /
//! Ctrl+Down moves, Reset / Cancel / Start buttons), and every shortcut
//! changing the focused row's action, end to end through a pty. Same
//! rules as `pty_ui.rs`.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn launch(repo: &FixtureRepo, session: &str, ready: &[&str]) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready_all(ready)
        .start()
}

fn subjects(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["log", "--format=%s"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn wait_subjects(repo: &FixtureRepo, want: &[&str]) {
    let want: Vec<String> = want.iter().map(|s| s.to_string()).collect();
    for _ in 0..300 {
        if subjects(repo) == want && !repo.root().join(".git/rebase-merge").exists() {
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(subjects(repo), want);
}

/// The screen row of the plan row for `summary` (the one with an action
/// dropdown; the commit-details panel repeats the summary elsewhere).
fn plan_row(s: &Screen, summary: &str) -> Option<u16> {
    s.find_all(summary)
        .into_iter()
        .map(|(_, r)| r)
        .find(|r| s.line(*r).contains('\u{25BE}'))
}

/// Does the plan row for `summary` show `label` (its action)?
fn has(s: &Screen, summary: &str, label: &str) -> bool {
    plan_row(s, summary).is_some_and(|r| s.line(r).contains(label))
}

/// Multi-selects the commits whose summaries are `summaries` (top to
/// bottom on screen): click the first, Space, then Down to each next row
/// (as many rows as separate them - a commit of another branch can sit
/// between two of them) and Space. Returns the column of the first row's
/// text and the screen row of the last one.
fn multi_select(term: &mut Terminal, summaries: &[&str]) -> (u16, u16) {
    let (col, mut prev) = term.wait_for_text(summaries[0], STEP).unwrap();
    term.click(col + 1, prev).unwrap();
    term.send_text(" ").unwrap();
    for summary in &summaries[1..] {
        let (_, row) = term.wait_for_text(summary, STEP).unwrap();
        let downs = vec!["Down"; usize::from(row - prev)].join(" ");
        term.send_keys(&downs).unwrap();
        term.send_text(" ").unwrap();
        prev = row;
    }
    (col, prev)
}

fn five_commits() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "c1 base")
        .unwrap()
        .commit_file("b.txt", "b\n", "c2 second")
        .unwrap()
        .commit_file("c.txt", "c\n", "c3 third")
        .unwrap()
        .commit_file("d.txt", "d\n", "c4 fourth")
        .unwrap()
        .commit_file("e.txt", "e\n", "c5 fifth")
        .unwrap()
        .build()
}

#[test]
fn the_rebase_view_has_the_footer_and_each_shortcut_changes_the_action() {
    let repo = five_commits();
    let (mut term, _c) = launch(&repo, "pty-rebase-footer", &["c5 fifth", "c1 base"]);
    let (col, row) = term.wait_for_text("c1 base", STEP).unwrap();
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the commit menu", STEP, |s| {
            s.contains("Interactively rebase from here...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Interactively rebase from here...").unwrap();
    term.click(c + 1, r).unwrap();
    let view = term
        .wait_for("the rebase view", STEP, |s| {
            s.contains("Start rebase") && s.contains("Ctrl+\u{2193} Move down")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    for want in [
        "Interactive rebase",
        "Rebasing main onto",
        "Cancel rebase",
        " Reset ",
        "shortcuts:",
        "ick",
        "quash",
        "eword",
        "rop",
        "Ctrl+\u{2191} Move up",
        "Ctrl+\u{2193} Move down",
    ] {
        assert!(
            view.contains(want),
            "{want:?} missing from:\n{}",
            view.text()
        );
    }

    // Rows newest first: c5 c4 c3 c2. d / s / f / e down the rows.
    term.send_keys("d").unwrap();
    term.send_keys("Down s").unwrap();
    term.send_keys("Down f").unwrap();
    term.send_keys("Down e").unwrap();
    term.wait_for("every row relabeled", STEP, |s| {
        has(s, "c5 fifth", "Drop")
            && has(s, "c4 fourth", "Squash")
            && has(s, "c3 third", "Fixup")
            && has(s, "c2 second", "Edit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // p puts c2 back to Pick; r opens the editor on c5 and Esc keeps
    // the row a Reword.
    term.send_keys("p").unwrap();
    term.wait_for("c2 is Pick", STEP, |s| has(s, "c2 second", "Pick"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Up Up Up r").unwrap();
    term.wait_for("the reword editor", STEP, |s| {
        s.contains("Reword commit message")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("c5 is Reword", STEP, |s| {
        !s.contains("Update message") && has(s, "c5 fifth", "Reword")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Ctrl+R is the Reset button.
    term.send_keys("C-r").unwrap();
    term.wait_for("reset", STEP, |s| {
        ["c5 fifth", "c4 fourth", "c3 third", "c2 second"]
            .iter()
            .all(|m| has(s, m, "Pick"))
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The footer's Cancel rebase (the last one on screen) closes the
    // view and changes nothing.
    let screen = term.screen();
    let (c, r) = screen
        .find_all("Cancel rebase")
        .into_iter()
        .last()
        .expect("the footer's Cancel rebase");
    term.click(c + 2, r).unwrap();
    term.wait_for("the view closed", STEP, |s| !s.contains("Start rebase"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(subjects(&repo).len(), 5);
}

#[test]
fn the_cherry_pick_view_has_the_footer_and_each_shortcut_changes_the_action() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "c1 base")
        .unwrap()
        .branch("topic")
        .unwrap()
        .checkout("topic")
        .unwrap()
        .commit_file("t1.txt", "1\n", "t1 first pick")
        .unwrap()
        .commit_file("t2.txt", "2\n", "t2 second pick")
        .unwrap()
        .checkout("main")
        .unwrap()
        .commit_file("m.txt", "m\n", "m1 on main")
        .unwrap()
        .build();
    let (mut term, _c) = launch(&repo, "pty-cp-footer", &["t2 second pick", "m1 on main"]);
    let (col, row1) = multi_select(&mut term, &["t2 second pick", "t1 first pick"]);
    term.right_click(col + 1, row1).unwrap();
    let menu = term
        .wait_for("the multi cherry-pick entry", STEP, |s| {
            s.contains("Cherry pick 2 commits")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Cherry pick 2 commits").unwrap();
    term.click(c + 1, r).unwrap();
    let view = term
        .wait_for("the view", STEP, |s| {
            s.contains("Start cherry pick") && s.contains("Ctrl+\u{2193} Move down")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    for want in [
        "Interactive cherry pick",
        "Cherry picking 2 commits onto main",
        "Cancel cherry pick",
        " Reset ",
        "shortcuts:",
        "Ctrl+\u{2191} Move up",
        "Ctrl+\u{2193} Move down",
    ] {
        assert!(
            view.contains(want),
            "{want:?} missing from:\n{}",
            view.text()
        );
    }
    // Rows newest first: t2, t1.
    term.send_keys("d").unwrap();
    term.wait_for("t2 dropped", STEP, |s| has(s, "t2 second pick", "Drop"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("s").unwrap();
    term.wait_for("t2 squashed", STEP, |s| has(s, "t2 second pick", "Squash"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down d").unwrap();
    term.wait_for("t1 dropped", STEP, |s| has(s, "t1 first pick", "Drop"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("p").unwrap();
    term.wait_for("t1 is Pick", STEP, |s| has(s, "t1 first pick", "Pick"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-r").unwrap();
    term.wait_for("reset", STEP, |s| {
        has(s, "t2 second pick", "Pick") && has(s, "t1 first pick", "Pick")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Start cherry pick, by mouse.
    let screen = term.screen();
    let (c, r) = screen.find("Start cherry pick").unwrap();
    term.click(c + 2, r).unwrap();
    wait_subjects(
        &repo,
        &["t2 second pick", "t1 first pick", "m1 on main", "c1 base"],
    );
}

/// Squashing c5 into c4 in the interactive view turns c4 into a Reword
/// whose description carries c5's message; the popup edits it, Start
/// runs the plan, and git holds exactly that message.
#[test]
fn squashing_into_a_row_rewords_it_with_the_combined_message() {
    let repo = five_commits();
    let (mut term, _c) = launch(&repo, "pty-rebase-squash-reword", &["c5 fifth", "c1 base"]);
    let (col, row) = term.wait_for_text("c1 base", STEP).unwrap();
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the commit menu", STEP, |s| {
            s.contains("Interactively rebase from here...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Interactively rebase from here...").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the rebase view", STEP, |s| s.contains("Start rebase"))
        .unwrap_or_else(|e| panic!("{e}"));

    // Rows newest first: c5 c4 c3 c2. c5 squashes into c4.
    term.send_keys("s").unwrap();
    term.wait_for("c4 became a Reword", STEP, |s| {
        has(s, "c5 fifth", "Squash") && has(s, "c4 fourth", "Reword")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Enter on c4 opens the popup: summary c4's, description c5's.
    term.send_keys("Down Enter").unwrap();
    term.wait_for("the popup with the combined message", STEP, |s| {
        s.contains("Reword commit message")
            && s.contains("Update message")
            && s.contains("c5 fifth")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(" edited").unwrap();
    term.send_keys("C-s").unwrap();
    term.wait_for("the popup closed", STEP, |s| {
        !s.contains("Update message") && s.contains("c4 fourth edited")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-s").unwrap();
    wait_subjects(
        &repo,
        &["c4 fourth edited", "c3 third", "c2 second", "c1 base"],
    );
    assert_eq!(repo.git(&["log", "-1", "--format=%b"]), "c5 fifth");
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
}
