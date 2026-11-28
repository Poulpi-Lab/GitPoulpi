//! Completion tickets through the real `start_at` link against an embedded
//! daemon (explicit discovery file, never `GITPOULPI_CONFIG_DIR`). The
//! dialogs that continue a ticket have their own tests next to them
//! (`remotes_ui/tests/tickets.rs`, `stage_guard_dialog/tests.rs`,
//! `repo_dialogs/close_tests.rs`); the helpers they share live here.

use super::*;

const WAIT: Duration = Duration::from_secs(60);

/// `GITPOULPI_GENERAL_AUTO_STASH` is process-wide: every test that sets it
/// holds this.
pub(crate) static AUTOSTASH_ENV: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// A link to its own embedded daemon, past its first refresh.
pub(crate) struct Harness {
    pub(crate) link: DaemonLink,
    _config: tempfile::TempDir,
}

pub(crate) fn harness(name: &str, fixture: &gitpoulpi_testkit::FixtureRepo) -> Harness {
    let config = tempfile::tempdir().unwrap();
    let port_file = config.path().join("sessions").join(format!("{name}.port"));
    let link = start_at(
        name.into(),
        fixture.repo_ref(),
        Some(port_file),
        crate::daemon_link_host::host(),
    );
    link.outcome_rx
        .recv_timeout(WAIT)
        .expect("the first refresh");
    Harness {
        link,
        _config: config,
    }
}

/// The reports of `ticket` up to and including its final one.
pub(crate) fn wait_final(ticket: Ticket) -> Vec<StepOutcome> {
    wait_until(ticket, |seen| seen.iter().any(StepOutcome::is_final))
}

/// The reports of `ticket` up to the first `NeedsUser`.
pub(crate) fn wait_needs_user(ticket: Ticket) -> Vec<StepOutcome> {
    wait_until(ticket, |seen| {
        seen.iter()
            .any(|o| matches!(o, StepOutcome::NeedsUser(_)) || o.is_final())
    })
}

fn wait_until(ticket: Ticket, done: impl Fn(&[StepOutcome]) -> bool) -> Vec<StepOutcome> {
    let started = std::time::Instant::now();
    let mut seen = Vec::new();
    loop {
        seen.extend(take_for(ticket));
        if done(&seen) {
            return seen;
        }
        assert!(
            started.elapsed() < WAIT,
            "ticket {ticket:?} never reported (saw {seen:?})"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// A repository whose `main` was pushed, then moved on at the remote, then
/// moved on here: a plain push is refused as non-fast-forward, and pulling
/// first (the files differ) merges cleanly.
pub(crate) fn diverged_fixture() -> gitpoulpi_testkit::FixtureRepo {
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("base.txt", "base\n", "base")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    fixture.git(&["push", "-u", "origin", "main"]);
    fixture.push_from_elsewhere("main", "theirs");
    std::fs::write(fixture.root().join("mine.txt"), "mine\n").unwrap();
    fixture.git(&["add", "mine.txt"]);
    fixture.git(&["commit", "-m", "mine"]);
    fixture
}

pub(crate) fn push_main() -> Action {
    Action::Push {
        remote: "origin".into(),
        opts: PushOptions {
            branch: Some("main".into()),
            ..PushOptions::default()
        },
    }
}

#[test]
fn an_action_that_worked_reports_ok() {
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .write_worktree("new.txt", b"x\n")
        .unwrap()
        .build();
    let h = harness("ticket-ok", &fixture);
    let ticket = h.link.submit_tracked(Action::StageAll);
    assert_eq!(wait_final(ticket), vec![StepOutcome::Ok(None)]);
    assert!(
        fixture
            .git(&["status", "--porcelain"])
            .contains("A  new.txt"),
        "the action itself ran"
    );
}

#[test]
fn a_failed_action_carries_the_status_line_text() {
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let h = harness("ticket-failed", &fixture);
    let ticket = h
        .link
        .submit_tracked(Action::CheckoutBranch("no-such-branch".into()));
    let outcome = wait_final(ticket);
    let line = h
        .link
        .action_error_rx
        .recv_timeout(WAIT)
        .expect("the status line got the error");
    assert_eq!(outcome, vec![StepOutcome::Failed { text: line }]);
}

/// A daemon job that was canceled ends its ticket `Canceled`, a refusal
/// `Failed`.
#[test]
fn a_canceled_job_is_canceled_not_failed() {
    use gitpoulpi_core::Error;
    assert_eq!(
        outcome_of_error(&Error::Canceled { job: 7 }, "canceled (job 7)"),
        StepOutcome::Canceled
    );
    assert_eq!(
        outcome_of_error(&Error::Backend("boom".into()), "boom"),
        StepOutcome::Failed {
            text: "boom".into()
        }
    );
}

/// A tab switch rebuilds the window and drops its link: the ticket lives in
/// process statics, so the answer is still there for the next window.
#[test]
fn a_ticket_survives_the_window_that_submitted_it() {
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .write_worktree("new.txt", b"x\n")
        .unwrap()
        .build();
    let h = harness("ticket-rebuild", &fixture);
    let ticket = h.link.submit_tracked(Action::StageAll);
    drop(h);
    // Either the action finished before the window went (Ok), or the link
    // ended with it in flight (the lost answer is a failure): never silence.
    let seen = wait_final(ticket);
    assert_eq!(seen.len(), 1, "{seen:?}");
    assert!(
        matches!(seen[0], StepOutcome::Ok(None) | StepOutcome::Failed { .. }),
        "{seen:?}"
    );
}

/// What a link thread had in flight when it ended fails, it does not hang.
#[test]
fn an_abandoned_link_fails_what_it_had_in_flight() {
    let link = new_link_id();
    let running = issue();
    begin(running, 1, link);
    let waiting = issue();
    begin(waiting, 1, link);
    report_ticket(Some(waiting), StepOutcome::NeedsUser(UserWait::AutoStash));
    let other = issue();
    begin(other, 1, new_link_id());

    abandon_link(link);

    assert!(matches!(
        take_for(running).as_slice(),
        [StepOutcome::Failed { .. }]
    ));
    // A dialog holds this one: the dialog ends it, not the link.
    assert_eq!(
        take_for(waiting),
        vec![StepOutcome::NeedsUser(UserWait::AutoStash)]
    );
    assert!(take_for(other).is_empty());
}

/// A renamed file is two actions for git; one ticket reports once, after
/// both.
#[test]
fn several_parts_of_one_ticket_report_once() {
    let ticket = issue();
    begin(ticket, 2, 0);
    report_ticket(Some(ticket), StepOutcome::Ok(None));
    assert!(take_for(ticket).is_empty(), "not before the last part");
    report_ticket(Some(ticket), StepOutcome::Ok(None));
    assert_eq!(take_for(ticket), vec![StepOutcome::Ok(None)]);

    let failing = issue();
    begin(failing, 2, 0);
    report_ticket(
        Some(failing),
        StepOutcome::Failed {
            text: "first".into(),
        },
    );
    report_ticket(Some(failing), StepOutcome::Ok(None));
    assert_eq!(
        take_for(failing),
        vec![StepOutcome::Failed {
            text: "first".into()
        }],
        "the first failure ends the ticket; the rest is ignored"
    );
}

/// The created worktree's path rides on the ticket's `Ok`.
#[test]
fn a_created_worktree_reports_its_path() {
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let h = harness("ticket-worktree", &fixture);
    let parent = tempfile::tempdir().unwrap();
    let path = parent.path().join("wt-ticket");
    let ticket = h.link.submit_tracked(Action::WorktreeAdd {
        path: path.clone(),
        opts: WorktreeAddOptions {
            start: gitpoulpi_core::domain::WorktreeStart::Branch("main".into()),
            new_branch: Some("wt-ticket".into()),
        },
    });
    match wait_final(ticket).as_slice() {
        [StepOutcome::Ok(Some(StepPayload::Worktree(made)))] => {
            assert_eq!(
                std::fs::canonicalize(made).unwrap(),
                std::fs::canonicalize(&path).unwrap()
            );
        }
        other => panic!("expected Ok with the worktree, got {other:?}"),
    }
}
