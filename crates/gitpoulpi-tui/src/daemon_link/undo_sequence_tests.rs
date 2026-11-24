//! "Undo whole sequence" through the real link against an embedded daemon:
//! the origin a ticket carries reaches the journal, the entries of the run
//! are undone newest first, and a refusal in the middle stops with the step
//! named and the older entries left.

use super::ticket_tests::{harness, wait_final, Harness};
use super::*;
use gitpoulpi_core::domain::SequenceOrigin;

const WAIT: Duration = Duration::from_secs(60);

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .write_worktree("one.txt", b"1\n")
        .unwrap()
        .build()
}

/// Submits `action` as step `step` of 3 of run 42 of "Backup" and waits for it.
fn step(h: &Harness, step: u32, action: Action) {
    let ticket = tickets::issue();
    tickets::set_origin(
        ticket,
        SequenceOrigin {
            name: "Backup".into(),
            run_id: 42,
            step,
            of: 3,
        },
    );
    h.link.submit_as(Some(ticket), action);
    assert_eq!(wait_final(ticket), vec![StepOutcome::Ok(None)]);
}

fn commit(message: &str) -> Action {
    Action::Commit {
        message: message.into(),
        amend: false,
        no_verify: false,
        push: false,
    }
}

/// Waits until the link's journal state satisfies `done`.
fn wait_state(h: &Harness, done: impl Fn(&gitpoulpi_core::domain::JournalState) -> bool) {
    let started = std::time::Instant::now();
    while !done(&h.link.journal_state()) {
        assert!(started.elapsed() < WAIT, "the journal state never settled");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// A run of stage, commit, stage again (steps 1 to 3).
fn run_three(h: &Harness, fixture: &gitpoulpi_testkit::FixtureRepo) {
    step(h, 1, Action::StageAll);
    step(h, 2, commit("one"));
    std::fs::write(fixture.root().join("two.txt"), "2\n").unwrap();
    step(h, 3, Action::StageAll);
    wait_state(h, |s| {
        s.sequence.as_ref().is_some_and(|g| g.entries.len() == 3)
    });
}

#[test]
fn the_whole_run_is_undone_newest_first() {
    let fixture = fixture();
    let h = harness("undo-seq-all", &fixture);
    let before = fixture.git(&["rev-parse", "HEAD"]);
    run_three(&h, &fixture);
    assert_ne!(fixture.git(&["rev-parse", "HEAD"]), before);

    let said = h.link.start_undo_sequence();
    assert!(!said.is_empty());
    wait_state(&h, |s| s.undo_count == 0);
    // The status line named the sequence, not a count.
    let message = h.link.action_message_rx.recv_timeout(WAIT).unwrap();
    assert!(message.contains("Backup"), "{message}");
    assert_eq!(fixture.git(&["rev-parse", "HEAD"]), before);
    assert_eq!(
        fixture.git(&["status", "--porcelain"]).lines().count(),
        2,
        "both files are untracked again"
    );
}

#[test]
fn a_refusal_in_the_middle_stops_and_leaves_the_older_entries() {
    // An outside commit must stay an outside change nobody records.
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    let fixture = fixture();
    let h = harness("undo-seq-stop", &fixture);
    run_three(&h, &fixture);
    // Somebody commits behind the run's back: the run's commit cannot be
    // taken back any more (its branch moved), the stage on top still can.
    fixture.git(&["commit", "--allow-empty", "-m", "from the shell"]);

    h.link.start_undo_sequence();
    let line = h.link.action_error_rx.recv_timeout(WAIT).unwrap();
    assert!(line.contains("\"Backup\""), "{line}");
    assert!(line.contains("step 2/3"), "the commit is step 2: {line}");
    assert!(line.contains("Commit"), "{line}");
    assert!(line.ends_with("2"), "two entries are left: {line}");
    wait_state(&h, |s| s.undo_count == 2);
    // The newest stage was undone; what is left is no longer a run's top.
    assert_eq!(
        h.link.journal_state().undo.map(|e| e.kind),
        Some(gitpoulpi_core::domain::OperationKind::Commit)
    );
}
