//! The grouped push dialog, the happy path: the branch has two extra push
//! targets, every remote is a fast-forward (or has no copy yet), Run pushes to
//! all of them and the progress rows turn into check marks.
//!
//! The dialog is entered through `Git: Push to Selected Remotes...` (its
//! selection step, then the plan). The rules are `pty_ui.rs`'s: nothing is
//! located by a hard-coded coordinate and every wait describes the whole
//! state it is waiting for. Falsified: with `PushGroupUi`'s Run submit broken,
//! or the progress rows not fed, the waits below time out.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

/// `feature` tracks `origin`, pushes to `gitlab` and `backup` too, and has a
/// local commit none of them has.
fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
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
        .unwrap()
        .push_targets("feature", &["gitlab", "backup"])
        .unwrap()
        .commit("second commit")
        .unwrap()
        .build()
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

/// The palette command, by its words.
fn open_selection(term: &mut Terminal) {
    term.send_keys("C-p").unwrap();
    term.send_text("Push to Selected Remotes").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Git: Push to Selected Remotes...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
}

/// The plan step with its rows read.
fn wait_plan(term: &Terminal, remotes: usize) -> Screen {
    term.wait_for("the plan step with its rows", STEP, |s| {
        s.contains(&format!("Push feature to {remotes} remotes"))
            && s.contains("Then: ")
            && s.contains("Fetch again")
            && !s.contains("Asking the remotes")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

/// The lowest occurrence of a button label (the key hint line above says
/// the same words).
fn button(screen: &Screen, label: &str) -> (u16, u16) {
    screen
        .find_all(label)
        .into_iter()
        .max_by_key(|(_, r)| *r)
        .unwrap_or_else(|| panic!("no {label} button:\n{}", screen.text()))
}

fn head(repo: &FixtureRepo) -> String {
    repo.git(&["rev-parse", "feature"]).trim().to_string()
}

#[test]
fn run_pushes_to_every_remote_and_the_rows_turn_into_check_marks() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-push-group");
    open_selection(&mut term);
    term.wait_for("the selection step", STEP, |s| {
        s.contains("Push feature to selected remotes")
            && s.contains("[x] origin")
            && s.contains("[x] gitlab")
            && s.contains("[x] backup")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    let plan = wait_plan(&term, 3);
    for remote in ["origin", "gitlab", "backup"] {
        assert!(plan.contains(remote), "{}", plan.text());
    }
    assert!(plan.contains("Then: push to 3 remotes."), "{}", plan.text());
    // Run by mouse.
    let (c, r) = button(&plan, "Run");
    term.click(c + 1, r).unwrap();
    let done = term
        .wait_for("every row sent and Done", STEP, |s| {
            s.contains("Done.")
                && ["origin", "gitlab", "backup"].iter().all(|n| {
                    (0..s.rows()).any(|y| {
                        let line = s.line(y);
                        line.contains('\u{2713}') && line.contains(n)
                    })
                })
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(done.contains("sent"), "{}", done.text());
    let local = head(&repo);
    assert_eq!(repo.git_in_remote(&["rev-parse", "feature"]).trim(), local);
    for name in ["gitlab", "backup"] {
        assert_eq!(
            repo.git_in_named_remote(name, &["rev-parse", "refs/heads/feature"])
                .trim(),
            local,
            "{name} has the new commit"
        );
    }
    // Closing the finished view never touches the run; the dialog is gone.
    let (c, r) = button(&done, "Close");
    term.click(c + 1, r).unwrap();
    term.wait_for("the dialog closed", STEP, |s| {
        !s.contains("Done.") && !s.contains("Push feature to")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The same run by keyboard alone: Enter on the selection, Tab to the Run
/// button, Enter to press it.
#[test]
fn the_keyboard_alone_reaches_run() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-push-group-keys");
    open_selection(&mut term);
    term.wait_for("the selection step", STEP, |s| {
        s.contains("Push feature to selected remotes") && s.contains("[x] backup")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_plan(&term, 3);
    // Fetch again, Skip problems, Cancel, Run: four Tabs.
    for _ in 0..4 {
        term.send_keys("Tab").unwrap();
    }
    term.send_keys("Enter").unwrap();
    term.wait_for("Done", STEP, |s| s.contains("Done."))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let local = head(&repo);
    assert_eq!(
        repo.git_in_named_remote("backup", &["rev-parse", "refs/heads/feature"])
            .trim(),
        local
    );
}
