//! An agent proposing a sequence run (`ui sequence run <id>`) end to end: the
//! real binary and its embedded daemon in a pty, and a plain IPC client
//! standing in for the command (`common::ui_show`). The banner "Proposed by
//! <client>: run sequence <name>" with Run and Dismiss opens; nothing runs
//! until the user presses Run, and Dismiss runs nothing.
//!
//! Temp config dir and fixture repo only.

mod common;

use std::time::Duration;

use gitpoulpi_core::domain::RepoId;
use gitpoulpi_ipc::ui::{Proposal, ProposalStatus, UiIntent, UiOutcome};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);

const CONFIG: &str = r#"
[[sequences]]
id = "backup"
name = "Backup"
scope = "machine"

[[sequences.steps]]
run = "stage_all"
"#;

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .build()
}

fn staged(repo: &FixtureRepo) -> bool {
    repo.git(&["diff", "--cached", "--name-only"])
        .contains("a.txt")
}

fn wait_status(config: &std::path::Path, session: &str, id: &str, want: ProposalStatus) {
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let got = common::ui_state(config, session)
            .proposals
            .into_iter()
            .find(|p| p.id == id)
            .map(|p| p.status);
        if got == Some(want) {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "{id}: wanted {want:?}, got {got:?}"
        );
        std::thread::sleep(Duration::from_millis(150));
    }
}

/// Where `label` sits on the banner row.
fn banner_button(term: &Terminal, label: &str) -> (u16, u16) {
    let screen = term.screen();
    let row = (0..screen.rows())
        .find(|&r| screen.line(r).contains("run sequence Backup"))
        .expect("the banner row");
    let line = screen.line(row);
    let after = line.find("run sequence Backup").unwrap();
    let at = after + line[after..].find(label).expect("the button");
    (line[..at].chars().count() as u16 + 1, row)
}

#[test]
fn nothing_runs_until_the_user_presses_run_and_dismiss_runs_nothing() {
    let repo = fixture();
    let session = "pty-seq-propose";
    let (mut term, config) = common::launch(repo.root())
        .session(session)
        .config(CONFIG)
        .size(200, 50)
        .no_exit_pause()
        .mouse_capture()
        .ready("initial commit")
        .start();
    common::wait_for_viewer(config.path(), session);
    // Past the "the user just pressed a key" deferral window.
    std::thread::sleep(Duration::from_millis(1700));
    let propose = |id: &str| {
        common::ui_show(
            config.path(),
            session,
            UiIntent::Propose {
                repo: RepoId::from_path(repo.root()),
                id: id.into(),
                proposal: Proposal::SequenceRun {
                    id: "backup".into(),
                },
            },
        )
    };

    // Refused: no such sequence.
    let out = common::ui_show(
        config.path(),
        session,
        UiIntent::Propose {
            repo: RepoId::from_path(repo.root()),
            id: "p-ghost".into(),
            proposal: Proposal::SequenceRun { id: "ghost".into() },
        },
    );
    assert!(matches!(out, UiOutcome::Refused { .. }), "{out:?}");

    // The banner, with its two buttons; nothing ran.
    match propose("p-dismiss") {
        UiOutcome::Proposed { id, .. } => assert_eq!(id, "p-dismiss"),
        other => panic!("expected Proposed, got {other:?}"),
    }
    let screen = term
        .wait_for("the banner", STEP, |s| {
            s.contains("Proposed by") && s.contains("run sequence Backup")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let row = (0..screen.rows())
        .map(|r| screen.line(r))
        .find(|l| l.contains("run sequence Backup"))
        .unwrap();
    assert!(row.contains("Run") && row.contains("Dismiss"), "{row}");
    std::thread::sleep(Duration::from_millis(700));
    assert!(!staged(&repo), "nothing runs before Run is pressed");

    // Dismiss: nothing runs, the proposal is rejected.
    let (col, line) = banner_button(&term, "Dismiss");
    term.click(col, line).unwrap();
    wait_status(
        config.path(),
        session,
        "p-dismiss",
        ProposalStatus::Rejected,
    );
    term.wait_for("the banner gone", STEP, |s| {
        !s.contains("run sequence Backup")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!staged(&repo), "Dismiss runs nothing");

    // Run: the sequence starts and stages the file. (A click is input: the
    // next proposal waits out the deferral window like any other.)
    std::thread::sleep(Duration::from_millis(1700));
    match propose("p-run") {
        UiOutcome::Proposed { .. } => {}
        other => panic!("expected Proposed, got {other:?}"),
    }
    term.wait_for("the banner", STEP, |s| s.contains("run sequence Backup"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, line) = banner_button(&term, "Run");
    term.click(col, line).unwrap();
    wait_status(config.path(), session, "p-run", ProposalStatus::Accepted);
    let deadline = std::time::Instant::now() + STEP;
    while !staged(&repo) {
        assert!(
            std::time::Instant::now() < deadline,
            "Run never started the sequence:\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    term.stop().unwrap();
}
