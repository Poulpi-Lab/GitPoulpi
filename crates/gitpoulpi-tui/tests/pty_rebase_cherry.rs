//! Multi-select cherry pick (Space toggles, right-click menu, "Start
//! Cherry Pick") and the conflict banner (Continue / Skip / Abort),
//! end to end through a pty. Same rules as `pty_ui.rs`.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

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

/// The screen row of the plan row for `summary` (the one with an action
/// dropdown; the commit-details panel repeats the summary elsewhere).
fn plan_row(s: &xtask::pty::Screen, summary: &str) -> Option<u16> {
    s.find_all(summary)
        .into_iter()
        .map(|(_, r)| r)
        .find(|r| s.line(*r).contains('\u{25BE}'))
}

fn launch(repo: &FixtureRepo, session: &str, ready: &[&str]) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready_all(ready)
        .start()
}

/// (c) multi-select two commits of another branch (click + Space), the
/// menu offers "Cherry pick 2 commits onto main...", Start lands both.
#[test]
fn multi_select_cherry_pick_lands_both_commits() {
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
    let (mut term, _c) = launch(&repo, "pty-cp", &["t1 first pick", "m1 on main"]);
    let (col, row2) = term.wait_for_text("t2 second pick", STEP).unwrap();
    term.click(col + 1, row2).unwrap();
    term.send_text(" ").unwrap();
    let (_, row1) = term.wait_for_text("t1 first pick", STEP).unwrap();
    // Walk to t1 by the number of rows that separate it from t2 on screen,
    // not with a single `Down`: the fixture's commits share a timestamp,
    // so whether t1 sits directly under t2 or under m1 depends on the
    // revwalk's tie-break (it differs between platforms and machine
    // speed). A plain click would clear the multi-selection.
    let key = if row1 > row2 { "Down" } else { "Up" };
    for _ in 0..row1.abs_diff(row2) {
        term.send_keys(key).unwrap();
    }
    term.send_text(" ").unwrap();
    term.right_click(col + 1, row1).unwrap();
    let menu = term
        .wait_for("the multi cherry-pick entry", STEP, |s| {
            s.contains("Cherry pick 2 commits")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Cherry pick 2 commits").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the cherry-pick view", STEP, |s| {
        s.contains("Start cherry pick")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    wait_subjects(
        &repo,
        &["t2 second pick", "t1 first pick", "m1 on main", "c1 base"],
    );
}

/// (e) a conflicting plan stops the rebase; the banner offers Skip, which
/// finishes it.
#[test]
fn a_conflict_shows_the_banner_and_skip_finishes() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("f.txt", "base\n", "c1 base")
        .unwrap()
        .commit_file("f.txt", "one\n", "c2 sets one")
        .unwrap()
        .commit_file("f.txt", "two\n", "c3 sets two")
        .unwrap()
        .commit_file("g.txt", "g\n", "c4 other file")
        .unwrap()
        .build();
    let (mut term, _c) = launch(&repo, "pty-conflict", &["c4 other file", "c1 base"]);
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
    // Rows newest first: c4 c3 c2. Drop c2: c3 then fails to apply.
    term.send_keys("Down Down d C-s").unwrap();
    term.wait_for("the conflict banner", STEP, |s| {
        s.contains(" Continue ") && s.contains(" Skip ") && s.contains(" Abort ")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let screen = term.screen();
    let (c, r) = screen.find(" Skip ").unwrap();
    term.click(c + 2, r).unwrap();
    wait_subjects(&repo, &["c4 other file", "c1 base"]);
}

/// The interactive cherry-pick view end to end with every action: four
/// commits multi-selected, then Reword (keyboard, message editor), Drop
/// (mouse, through the row's dropdown) and Squash (keyboard, Space opens
/// the dropdown) before Start lands the plan.
#[test]
fn interactive_cherry_pick_applies_reword_drop_and_squash() {
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
        .commit_file("t3.txt", "3\n", "t3 third pick")
        .unwrap()
        .commit_file("t4.txt", "4\n", "t4 fourth pick")
        .unwrap()
        .checkout("main")
        .unwrap()
        .commit_file("m.txt", "m\n", "m1 on main")
        .unwrap()
        .build();
    let (mut term, _c) = launch(&repo, "pty-cp-all", &["t4 fourth pick", "m1 on main"]);
    let (col, row1) = multi_select(
        &mut term,
        &[
            "t4 fourth pick",
            "t3 third pick",
            "t2 second pick",
            "t1 first pick",
        ],
    );
    term.right_click(col + 1, row1).unwrap();
    let menu = term
        .wait_for("the multi cherry-pick entry", STEP, |s| {
            s.contains("Cherry pick 4 commits")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Cherry pick 4 commits").unwrap();
    term.click(c + 1, r).unwrap();
    let view = term
        .wait_for("the cherry-pick view", STEP, |s| {
            s.contains("Start cherry pick")
                && s.contains("Cancel cherry pick")
                && s.contains("Ctrl+\u{2193} Move down")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(view.contains("Cherry picking 4 commits onto main"));

    // Row 0 (t4): Reword through the message editor.
    term.send_keys("r").unwrap();
    term.wait_for("the reword editor", STEP, |s| {
        s.contains("Update message") && s.contains("Reword commit message")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(" renamed").unwrap();
    term.send_keys("C-s").unwrap();
    term.wait_for("the editor closed", STEP, |s| {
        !s.contains("Update message") && s.contains("renamed")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Row 1 (t3): Drop, by clicking its dropdown and then "Drop".
    let screen = term.screen();
    let r3 = plan_row(&screen, "t3 third pick").unwrap();
    let (bx, _) = screen
        .find_all("Pick")
        .into_iter()
        .find(|(_, y)| *y == r3)
        .expect("the row's action button");
    term.click(bx + 1, r3).unwrap();
    let open = term
        .wait_for("the action list", STEP, |s| {
            s.find_all("Squash")
                .iter()
                .any(|(_, y)| *y > r3 && *y < r3 + 6)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (dx, dy) = open
        .find_all("Drop")
        .into_iter()
        .find(|(x, y)| *y > r3 && *y < r3 + 6 && *x >= bx && *x < bx + 12)
        .expect("Drop in the list");
    term.click(dx, dy).unwrap();
    term.wait_for("t3 dropped", STEP, |s| {
        s.find_all("Drop").iter().any(|(_, y)| *y == r3)
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Row 2 (t2): Squash into t1, by keyboard.
    term.send_keys("Down").unwrap();
    term.send_text(" ").unwrap();
    term.send_keys("Down Down Enter").unwrap();
    term.wait_for("t2 reads Squash", STEP, |s| {
        plan_row(s, "t2 second pick").is_some_and(|r| s.line(r).contains("Squash"))
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-s").unwrap();
    wait_subjects(
        &repo,
        &[
            "t4 fourth pick renamed",
            "t1 first pick",
            "m1 on main",
            "c1 base",
        ],
    );
    assert!(repo.root().join("t2.txt").exists(), "t2 was squashed in");
    assert!(!repo.root().join("t3.txt").exists(), "t3 was dropped");
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
}

/// The top-right "Cancel cherry pick" button closes the view and changes
/// nothing.
#[test]
fn cancel_cherry_pick_leaves_the_repository_alone() {
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
    let (mut term, _c) = launch(&repo, "pty-cp-cancel", &["t2 second pick", "m1 on main"]);
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
        .wait_for("the view", STEP, |s| s.contains("Cancel cherry pick"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (bx, by) = view.find("Cancel cherry pick").unwrap();
    term.click(bx + 2, by).unwrap();
    term.wait_for("the view closed", STEP, |s| {
        !s.contains("Start cherry pick")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(subjects(&repo), vec!["m1 on main", "c1 base"]);
}
