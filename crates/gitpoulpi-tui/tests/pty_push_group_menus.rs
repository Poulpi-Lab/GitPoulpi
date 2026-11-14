//! The Push button, the Push dropdown, the branch context menu and the palette
//! on a branch with extra push targets (`feature` tracks `origin` and pushes to
//! `gitlab` and `backup` too).
//!
//! - The Push button pushes to all three bare remotes with no dialog to answer
//!   (every row is a plain push), the progress view shows the result.
//! - The Push dropdown's "Push to gitlab only" pushes there and nowhere else.
//! - "Push to selected remotes..." is offered on a branch without targets too.
//! - The branch context menu carries the same rows as the toolbar.
//! - The palette's Continue / Abort / Retry are grayed out with their reason
//!   while no grouped push is waiting.
//!
//! Falsified: with the Push button routed back to the single-remote push the
//! first test never sees backup move; with `push_one` pushing to the wrong
//! remote the second goes red; without the always-on "selected remotes" row the
//! third fails; without the group rows in `ref_menu_entries` the fourth fails.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(40);

/// `feature` tracks `origin`, pushes to `gitlab` and `backup` too, and has a
/// local commit none of them has.
fn fixture(targets: bool) -> FixtureRepo {
    let b = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .with_remote("gitlab")
        .unwrap()
        .with_remote("backup")
        .unwrap()
        .push("main")
        .unwrap()
        .branch("feature")
        .unwrap()
        .commit("feature commit")
        .unwrap()
        .push("feature")
        .unwrap()
        .push_to("gitlab", "feature")
        .unwrap()
        .push_to("backup", "feature")
        .unwrap()
        .track("feature", "origin")
        .unwrap();
    let b = if targets {
        b.push_targets("feature", &["gitlab", "backup"]).unwrap()
    } else {
        b
    };
    b.commit("second commit").unwrap().build()
}

fn launch(repo: &FixtureRepo, session: &str) -> Terminal {
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready("second commit")
        .start();
    std::mem::forget(config);
    term
}

fn remote_head(repo: &FixtureRepo, name: &str) -> String {
    if name == "origin" {
        repo.git_in_remote(&["rev-parse", "feature"])
    } else {
        repo.git_in_named_remote(name, &["rev-parse", "refs/heads/feature"])
    }
    .trim()
    .to_string()
}

/// The toolbar's Push button: its glyph and label, the button itself (the
/// dropdown arrow is the cell after it).
fn push_button(screen: &Screen) -> (u16, u16) {
    screen
        .find_all("\u{21E7} Push")
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no Push button:\n{}", screen.text()))
}

/// The ▾ of the Push button: the first arrow right of the button.
fn push_arrow(screen: &Screen) -> (u16, u16) {
    let (x, y) = push_button(screen);
    let line = screen.line(y);
    let chars: Vec<char> = line.chars().collect();
    let arrow = (usize::from(x)..chars.len())
        .find(|i| chars[*i] == '\u{25BE}')
        .unwrap_or_else(|| panic!("no arrow after Push:\n{}", screen.text()));
    (arrow as u16, y)
}

fn item(screen: &Screen, label: &str) -> (u16, u16) {
    screen
        .find_all(label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no {label} row:\n{}", screen.text()))
}

#[test]
fn the_push_button_on_a_branch_with_targets_pushes_to_all_three_remotes() {
    let repo = fixture(true);
    let local = repo.git(&["rev-parse", "feature"]).trim().to_string();
    for r in ["origin", "gitlab", "backup"] {
        assert_ne!(remote_head(&repo, r), local);
    }
    let mut term = launch(&repo, "pty-push-menus-button");
    let screen = term.screen();
    let (c, r) = push_button(&screen);
    term.click(c + 1, r).unwrap();
    let done = term
        .wait_for("the grouped push finished", STEP, |s| s.contains("Done."))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(done.contains("Push feature"), "{}", done.text());
    for r in ["origin", "gitlab", "backup"] {
        assert_eq!(remote_head(&repo, r), local, "{r} got the commit");
    }
}

#[test]
fn the_push_dropdown_pushes_to_one_remote_only() {
    let repo = fixture(true);
    let local = repo.git(&["rev-parse", "feature"]).trim().to_string();
    let origin_before = remote_head(&repo, "origin");
    let mut term = launch(&repo, "pty-push-menus-only");
    let screen = term.screen();
    let (c, r) = push_arrow(&screen);
    term.click(c, r).unwrap();
    let menu = term
        .wait_for("the Push dropdown with the group rows", STEP, |s| {
            s.contains("Push to all (origin, gitlab, backup)")
                && s.contains("Push to gitlab only")
                && s.contains("Push to selected remotes...")
                && s.contains("Remotes of feature...")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = item(&menu, "Push to gitlab only");
    term.click(c + 3, r).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while remote_head(&repo, "gitlab") != local {
        assert!(
            std::time::Instant::now() < deadline,
            "gitlab never got the commit:\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    // Give a wrong fan-out time to show, then check the others stayed.
    std::thread::sleep(Duration::from_millis(800));
    assert_eq!(remote_head(&repo, "origin"), origin_before);
    assert_ne!(remote_head(&repo, "backup"), local);
}

#[test]
fn push_to_selected_remotes_is_offered_on_a_branch_without_targets() {
    let repo = fixture(false);
    let mut term = launch(&repo, "pty-push-menus-selected");
    let screen = term.screen();
    let (c, r) = push_arrow(&screen);
    term.click(c, r).unwrap();
    let menu = term
        .wait_for("the Push dropdown", STEP, |s| {
            s.contains("Push to selected remotes...") && s.contains("Remotes of feature...")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!menu.contains("Push to all ("), "{}", menu.text());
    let (c, r) = item(&menu, "Push to selected remotes...");
    term.click(c + 3, r).unwrap();
    term.wait_for("the selection step", STEP, |s| {
        s.contains("Push feature to selected remotes") && s.contains("[x] origin")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_branch_context_menu_has_the_same_rows_as_the_toolbar() {
    let repo = fixture(true);
    let mut term = launch(&repo, "pty-push-menus-ref");
    let screen = term.screen();
    // The LOCAL section's `feature` row.
    let (c, r) = screen
        .find_all("feature")
        .into_iter()
        .find(|(c, _)| *c < 40)
        .unwrap_or_else(|| panic!("no feature row:\n{}", screen.text()));
    term.right_click(c + 1, r).unwrap();
    let menu = term
        .wait_for("the branch menu with the group rows", STEP, |s| {
            s.contains("Push to all (origin, gitlab, backup)")
                && s.contains("Push to gitlab only")
                && s.contains("Pull from gitlab only")
                && s.contains("Push to selected remotes...")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = item(&menu, "Push to backup only");
    term.click(c + 3, r).unwrap();
    let local = repo.git(&["rev-parse", "feature"]).trim().to_string();
    let deadline = std::time::Instant::now() + STEP;
    while remote_head(&repo, "backup") != local {
        assert!(
            std::time::Instant::now() < deadline,
            "backup never got the commit:\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[test]
fn the_palette_continue_abort_and_retry_are_grayed_out_without_a_waiting_group() {
    let repo = fixture(true);
    let mut term = launch(&repo, "pty-push-menus-palette");
    // One command at a time: the selected row's reason shows when the
    // filter leaves a single row.
    for (words, label, reason) in [
        (
            "Show Grouped Push",
            "Git: Show Grouped Push",
            "there is no grouped push to show",
        ),
        (
            "Continue Grouped Push",
            "Git: Continue Grouped Push",
            "no grouped push is waiting on conflicts",
        ),
        (
            "Abort Grouped Push",
            "Git: Abort Grouped Push",
            "no grouped push is waiting on conflicts",
        ),
        (
            "Retry Failed Pushes",
            "Git: Retry Failed Pushes",
            "no remote failed in the last grouped push",
        ),
    ] {
        term.send_keys("C-p").unwrap();
        term.send_text(words).unwrap();
        term.wait_for(label, STEP, |s| {
            s.contains("Command Palette (1/") && s.contains(label) && s.contains(reason)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        term.send_keys("Escape").unwrap();
        term.wait_for_text_to_go("Command Palette", STEP)
            .unwrap_or_else(|e| panic!("{e}"));
    }
}

/// A one-remote push of a branch with targets that the remote refuses
/// (non-fast-forward) opens the push group dialog with the remote's merge
/// default, not the single-remote "has diverged" menu.
#[test]
fn a_rejected_push_of_a_branch_with_targets_opens_the_push_group_dialog() {
    let repo = fixture(true);
    repo.push_from_elsewhere_to("gitlab", "feature", "gitlab moved");
    let mut term = launch(&repo, "pty-push-menus-rejected");
    let screen = term.screen();
    let (c, r) = push_arrow(&screen);
    term.click(c, r).unwrap();
    let menu = term
        .wait_for("the Push dropdown", STEP, |s| {
            s.contains("Push to gitlab only")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = item(&menu, "Push to gitlab only");
    term.click(c + 3, r).unwrap();
    let plan = term
        .wait_for("the plan with gitlab's merge default", STEP, |s| {
            s.contains("Push feature to 3 remotes")
                && s.contains("Merge from gitlab")
                && !s.contains("Asking the remotes")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !plan.contains("Pull (merge), then push"),
        "the single-remote diverged menu is not shown:\n{}",
        plan.text()
    );
}
