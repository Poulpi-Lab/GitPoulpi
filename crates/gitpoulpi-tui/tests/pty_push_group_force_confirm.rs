//! A rewritten branch: the commit `origin` holds was amended locally, so the
//! plan defaults to a force push (with lease) for it. Run asks first, through
//! the ordinary destructive confirmation, naming the remote and the commits it
//! replaces; Esc cancels and nothing is pushed; asking again and confirming
//! force-pushes.
//!
//! Falsified: with the confirmation skipped (`Effect::Confirm` replaced by the
//! submit), the first test finds origin overwritten; with the confirmed action
//! not starting the run (`on_started`), the second never reaches Done.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(40);

fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .branch("feature")
        .unwrap()
        .commit("feature commit")
        .unwrap()
        .push("feature")
        .unwrap()
        .track("feature", "origin")
        .unwrap()
        .build();
    // The pushed commit is replaced by an amended one.
    repo.git(&["commit", "--amend", "-m", "amended commit"]);
    repo
}

fn button(screen: &Screen, label: &str) -> (u16, u16) {
    screen
        .find_all(label)
        .into_iter()
        .max_by_key(|(_, r)| *r)
        .unwrap_or_else(|| panic!("no {label} button:\n{}", screen.text()))
}

fn plan(repo: &FixtureRepo, session: &str) -> (Terminal, Screen) {
    let (mut term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready("amended commit")
        .start();
    std::mem::forget(config);
    term.send_keys("C-p").unwrap();
    term.send_text("Push to Selected Remotes").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Git: Push to Selected Remotes...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the selection step", STEP, |s| {
        s.contains("Push feature to selected remotes") && s.contains("[x] origin")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    let screen = term
        .wait_for("the plan with the force default", STEP, |s| {
            s.contains("Push feature to 1 remote")
                && s.contains("Force push (lease)")
                && !s.contains("Asking the remotes")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    (term, screen)
}

#[test]
fn a_rewritten_remote_defaults_to_force_and_esc_on_the_confirmation_pushes_nothing() {
    let repo = fixture();
    let before = repo.git_in_remote(&["rev-parse", "feature"]);
    let (mut term, screen) = plan(&repo, "pty-push-group-force-esc");
    assert!(
        screen.contains("Force push replaces 1 commit on origin."),
        "the warning names the remote and the count:\n{}",
        screen.text()
    );
    let (c, r) = button(&screen, "Run");
    term.click(c + 1, r).unwrap();
    let ask = term
        .wait_for("the force confirmation", STEP, |s| {
            s.contains("Force push feature to origin?")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(ask.contains("1 commit"), "{}", ask.text());
    term.send_keys("Escape").unwrap();
    term.wait_for("the confirmation closed, the plan kept", STEP, |s| {
        !s.contains("Force push feature to origin?") && s.contains("Push feature to 1 remote")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Give a wrongly started push time to land.
    std::thread::sleep(Duration::from_millis(1500));
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "feature"]),
        before,
        "nothing was pushed"
    );
}

#[test]
fn confirming_the_force_push_runs_it_and_the_dialog_becomes_the_progress_view() {
    let repo = fixture();
    let (mut term, screen) = plan(&repo, "pty-push-group-force-yes");
    let (c, r) = button(&screen, "Run");
    term.click(c + 1, r).unwrap();
    let ask = term
        .wait_for("the force confirmation", STEP, |s| {
            s.contains("Force push feature to origin?")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = button(&ask, "Force push");
    term.click(c + 1, r).unwrap();
    term.wait_for("the progress view says Done", STEP, |s| {
        s.contains("Done.") && s.contains("origin")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "feature"]).trim(),
        repo.git(&["rev-parse", "feature"]).trim()
    );
}
