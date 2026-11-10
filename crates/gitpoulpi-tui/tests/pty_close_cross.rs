//! The shared close cross (` ✕ ` on a framed surface's top border, no "Esc"
//! text beside it): a click on it closes Settings, the Diff view and the
//! Global search overlay, and Esc still closes each of them.

#![cfg(unix)]

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const CROSS: &str = "\u{2715}";

fn start(session: &str) -> (Terminal, gitpoulpi_testkit::FixtureRepo) {
    let before: String = (1..=40).map(|n| format!("row {n:02}\n")).collect();
    let after = before.replace("row 20\n", "row 20 changed\n");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("big.txt", &before, "ten rows")
        .unwrap()
        .commit_file("big.txt", &after, "change row 5")
        .unwrap()
        .build();
    let (term, config) = common::launch(repo.root())
        .no_commit_column()
        .session(session)
        .size(160, 50)
        .no_exit_pause()
        .ready("change row 5")
        .start();
    // Keep the temp config dir alive for the whole test.
    std::mem::forget(config);
    (term, repo)
}

fn open_settings(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn open_diff(term: &mut Terminal) {
    term.wait_for("the commit detail", STEP, |s| s.contains("View all files"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_mouse_capture(STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = term
        .wait_for_text("big.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the diff", STEP, |s| s.contains("row 20 changed"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn cross_without_esc(term: &Terminal) -> (u16, u16) {
    let screen = term
        .wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find(CROSS).expect("the close cross");
    let line = term.text().lines().nth(r as usize).unwrap().to_string();
    assert!(!line.contains("Esc"), "no Esc beside the cross: {line:?}");
    (c, r)
}

#[test]
fn clicking_the_cross_closes_settings_and_esc_still_does() {
    let (mut term, _repo) = start("pty-cross-settings");
    open_settings(&mut term);
    let (c, r) = cross_without_esc(&term);
    term.click(c + 1, r).unwrap();
    term.wait_for_text_to_go("Repo-specific settings", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    open_settings(&mut term);
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Repo-specific settings", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn clicking_the_cross_closes_the_diff() {
    let (mut term, _repo) = start("pty-cross-diff");
    open_diff(&mut term);
    let (c, r) = cross_without_esc(&term);
    term.click(c + 1, r).unwrap();
    term.wait_for_text_to_go("row 20 changed", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn esc_closes_the_diff_without_the_cross() {
    let (mut term, _repo) = start("pty-cross-diff-esc");
    open_diff(&mut term);
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("row 20 changed", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn clicking_the_cross_closes_the_global_search_and_esc_still_does() {
    let (mut term, _repo) = start("pty-cross-search");
    term.send_keys("M-C-f").unwrap();
    term.wait_for("the search popup", STEP, |s| {
        s.contains("Search open repositories")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = cross_without_esc(&term);
    term.click(c + 1, r).unwrap();
    term.wait_for_text_to_go("Search open repositories", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    term.send_keys("M-C-f").unwrap();
    term.wait_for_text("Search open repositories", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Search open repositories", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
