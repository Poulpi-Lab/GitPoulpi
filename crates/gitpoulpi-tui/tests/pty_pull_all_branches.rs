//! "Pull all branches": the Pull dropdown's row and `Ctrl+G Ctrl+P` fetch,
//! fast-forward every branch that is only behind its upstream and pull the
//! checked-out one; Undo restores every branch; a behind branch checked out in
//! another worktree is asked about first (update, skip or cancel).
//!
//! Falsified: with the dropdown row routed to nothing the topic branch never
//! moves; with the keymap's exact-chord lookup removed `Ctrl+G Ctrl+P` opens
//! the palette (or runs a plain Pull) and topic stays behind; with the
//! dialog's Update answer sending `skip` the worktree branch stays put; with
//! the check removed the dialog never appears.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);

/// `main` (checked out) and `topic`, both tracking origin, both behind it.
fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .branch("topic")
        .unwrap()
        .commit("topic commit")
        .unwrap()
        .push("topic")
        .unwrap()
        .track("topic", "origin")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    repo.push_from_elsewhere("main", "main news");
    repo.push_from_elsewhere("topic", "topic news");
    repo
}

/// `main` and a branch `wtbranch` checked out in a linked worktree, which is
/// behind origin (fetched already, so the window's refs know it).
fn worktree_fixture() -> (FixtureRepo, std::path::PathBuf) {
    let (builder, wt) = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .worktree("orbit-wt", "wtbranch")
        .unwrap();
    let repo = builder
        .push("wtbranch")
        .unwrap()
        .track("wtbranch", "origin")
        .unwrap()
        .build();
    repo.push_from_elsewhere("wtbranch", "wt news");
    repo.git(&["fetch", "origin"]);
    (repo, wt)
}

fn launch(repo: &FixtureRepo, session: &str) -> Terminal {
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready("base commit")
        .start();
    std::mem::forget(config);
    term
}

fn rev(repo: &FixtureRepo, name: &str) -> String {
    repo.git(&["rev-parse", name]).trim().to_string()
}

fn wait_until(what: &str, term: &Terminal, mut done: impl FnMut() -> bool) {
    let deadline = Instant::now() + STEP;
    while !done() {
        assert!(
            Instant::now() < deadline,
            "{what} never happened:\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn open_pull_menu(term: &mut Terminal) -> xtask::pty::Screen {
    let screen = term
        .wait_for("the toolbar's Pull split button", STEP, |s| {
            s.contains("Pull  \u{25BE}")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find("Pull  \u{25BE}").unwrap();
    term.click(c + 5, r).expect("click the Pull dropdown arrow");
    term.wait_for("the Pull menu with Pull all branches", STEP, |s| {
        s.contains("Pull all branches")
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn the_pull_menu_row_brings_a_behind_branch_to_its_upstream() {
    let repo = fixture();
    let before = rev(&repo, "topic");
    let mut term = launch(&repo, "pty-pull-all-menu");
    let menu = open_pull_menu(&mut term);
    let (c, r) = menu
        .find_all("Pull all branches")
        .into_iter()
        .next()
        .expect("the default-mode row");
    term.click(c + 3, r).unwrap();
    // `origin/topic` is stale until the run fetches: topic moving is the proof.
    wait_until("topic moving to its upstream", &term, || {
        rev(&repo, "topic") != before && rev(&repo, "topic") == rev(&repo, "origin/topic")
    });
    term.wait_for("the summary of the run", STEP, |s| {
        s.contains("updated 2 branches")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s", "main"]).trim(),
        "main news",
        "the checked-out branch was pulled too"
    );
}

#[test]
fn ctrl_g_ctrl_p_runs_it_without_opening_the_palette_and_undo_restores_every_branch() {
    let repo = fixture();
    let topic_before = rev(&repo, "topic");
    let main_before = rev(&repo, "main");
    let mut term = launch(&repo, "pty-pull-all-chord");
    term.send_keys("C-g").unwrap();
    term.send_keys("C-p").unwrap();
    wait_until("topic moving to its upstream", &term, || {
        rev(&repo, "topic") != topic_before && rev(&repo, "topic") == rev(&repo, "origin/topic")
    });
    assert!(
        !term.screen().contains("Command Palette"),
        "the chord did not open the palette:\n{}",
        term.text()
    );
    term.wait_for("the summary of the run", STEP, |s| {
        s.contains("updated 2 branches")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // One Undo takes every branch back.
    term.send_keys("C-z").unwrap();
    wait_until("topic and main restored", &term, || {
        rev(&repo, "topic") == topic_before && rev(&repo, "main") == main_before
    });
}

#[test]
fn ctrl_g_p_is_still_a_plain_pull() {
    let repo = fixture();
    let topic_before = rev(&repo, "topic");
    let mut term = launch(&repo, "pty-pull-all-plain-pull");
    term.send_keys("C-g").unwrap();
    term.send_keys("p").unwrap();
    wait_until("main pulled", &term, || {
        repo.git(&["log", "-1", "--format=%s", "main"]).trim() == "main news"
    });
    std::thread::sleep(Duration::from_millis(1500));
    assert_eq!(
        rev(&repo, "topic"),
        topic_before,
        "a plain Pull leaves the other branches alone"
    );
}

#[test]
fn a_behind_branch_in_another_worktree_is_asked_about_and_skip_leaves_it() {
    let (repo, wt) = worktree_fixture();
    let before = rev(&repo, "wtbranch");
    let mut term = launch(&repo, "pty-pull-all-wt-skip");
    term.send_keys("C-g").unwrap();
    term.send_keys("C-p").unwrap();
    term.wait_for("the worktree question", STEP, |s| {
        s.contains("behind and checked out in other worktrees")
            && s.contains("wtbranch")
            && s.contains("orbit-wt")
            && s.contains("Update those worktrees too")
            && s.contains("Skip them")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("2").unwrap();
    term.wait_for(
        "the summary naming the skipped worktree branch",
        STEP,
        |s| s.contains("checked out in another worktree") && !s.contains("Skip them"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(rev(&repo, "wtbranch"), before, "Skip left the branch alone");
    assert_eq!(
        std::process::Command::new("git")
            .args(["log", "-1", "--format=%s"])
            .current_dir(&wt)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap(),
        "base commit"
    );
}

#[test]
fn updating_the_worktrees_fast_forwards_the_branch_in_its_folder() {
    let (repo, wt) = worktree_fixture();
    let mut term = launch(&repo, "pty-pull-all-wt-update");
    term.send_keys("C-g").unwrap();
    term.send_keys("C-p").unwrap();
    term.wait_for("the worktree question", STEP, |s| {
        s.contains("Update those worktrees too")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("1").unwrap();
    wait_until("wtbranch reaching origin/wtbranch", &term, || {
        rev(&repo, "wtbranch") == rev(&repo, "origin/wtbranch")
    });
    let head = std::process::Command::new("git")
        .args(["log", "-1", "--format=%s"])
        .current_dir(&wt)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap();
    assert_eq!(head, "wt news", "the worktree's files followed the branch");
}

#[test]
fn cancel_runs_nothing() {
    let (repo, _wt) = worktree_fixture();
    let before = rev(&repo, "wtbranch");
    let mut term = launch(&repo, "pty-pull-all-wt-cancel");
    term.send_keys("C-g").unwrap();
    term.send_keys("C-p").unwrap();
    term.wait_for("the worktree question", STEP, |s| {
        s.contains("Update those worktrees too")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("the question gone", STEP, |s| {
        !s.contains("Update those worktrees too")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    std::thread::sleep(Duration::from_millis(1500));
    assert_eq!(rev(&repo, "wtbranch"), before);
    assert!(
        !term.screen().contains("updated 1 branch"),
        "nothing ran:\n{}",
        term.text()
    );
}
