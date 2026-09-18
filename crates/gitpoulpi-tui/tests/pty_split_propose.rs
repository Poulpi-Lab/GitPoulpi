//! An agent's split proposal (`ui propose split`) end to end: the real binary
//! and its embedded daemon in a pty, and a plain IPC client standing in for
//! the command (`common::ui_show`). The proposal opens the split editor
//! pre-filled and marked "Proposed by"; only the user's own Apply splits.
//!
//! Every wait names the state it waits for.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{Oid, RepoId, SplitPart, SplitPiece};
use gitpoulpi_ipc::ui::{Proposal, ProposalStatus, UiIntent, UiOutcome};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};

const STEP: Duration = Duration::from_secs(30);

/// `base`, then `parser and flag` (edits `a.txt`, adds `b.txt` and the helper's own file), then a tip.
fn fixture() -> FixtureRepo {
    let before: String = (1..=4).map(|n| format!("line {n}\n")).collect();
    let after: String = (1..=4)
        .map(|n| match n {
            1 => "line 1 changed\n".to_string(),
            n => format!("line {n}\n"),
        })
        .collect();
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", &before, "base")
        .unwrap()
        .write_worktree("a.txt", after.as_bytes())
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .write_worktree("b.txt", b"bee\n")
        .unwrap()
        .stage("b.txt")
        .unwrap()
        .commit("parser and flag")
        .unwrap()
        .commit("tip on top")
        .unwrap()
        .build()
}

fn subjects(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["log", "--format=%s"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn file(path: &str) -> SplitPiece {
    SplitPiece::File { path: path.into() }
}

fn status_of(dir: &std::path::Path, session: &str, id: &str) -> Option<ProposalStatus> {
    common::ui_state(dir, session)
        .proposals
        .into_iter()
        .find(|p| p.id == id)
        .map(|p| p.status)
}

fn wait_status(dir: &std::path::Path, session: &str, id: &str, want: ProposalStatus) {
    let deadline = Instant::now() + STEP;
    loop {
        let got = status_of(dir, session, id);
        if got == Some(want) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "{id}: wanted {want:?}, got {got:?}"
        );
        std::thread::sleep(Duration::from_millis(150));
    }
}

#[test]
fn a_split_proposal_opens_the_editor_and_only_the_users_apply_splits() {
    let repo = fixture();
    let session = "pty-propose-split";
    let (mut term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready("tip on top")
        .start();
    common::wait_for_viewer(config.path(), session);
    // Past the "the user just pressed a key" deferral window.
    std::thread::sleep(Duration::from_millis(1700));
    let dir = config.path();
    let target = Oid::from_hex(repo.git(&["rev-parse", "HEAD~1"]).trim()).expect("an object id");
    let proposal = Proposal::SplitCommit {
        commit: target,
        parts: vec![
            SplitPart {
                message: "add the bee file".into(),
                pieces: vec![file("b.txt"), file("parser and flag.txt")],
            },
            SplitPart {
                message: "change the first line".into(),
                pieces: vec![file("a.txt")],
            },
        ],
    };
    let send = |id: &str| {
        common::ui_show(
            dir,
            session,
            UiIntent::Propose {
                repo: RepoId::from_path(repo.root()),
                id: id.into(),
                proposal: proposal.clone(),
            },
        )
    };
    let before = subjects(&repo);

    // Opened pre-filled, with the banner; Esc closes it and nothing changes.
    assert!(matches!(send("p-s1"), UiOutcome::Proposed { .. }));
    term.wait_for("the editor, pre-filled and marked", STEP, |s| {
        let t = s.text();
        t.contains("Proposed by claude-code")
            && t.contains("add the bee file")
            && t.contains("change the first line")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(status_of(dir, session, "p-s1"), Some(ProposalStatus::Open));
    term.send_keys("Esc").unwrap();
    term.wait_for("the editor closed", STEP, |s| {
        !s.text().contains("Proposed by claude-code")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    wait_status(dir, session, "p-s1", ProposalStatus::Rejected);
    assert_eq!(subjects(&repo), before, "nothing was split");

    // Again, applied with the editor's own key: split, accepted.
    std::thread::sleep(Duration::from_millis(1700));
    assert!(matches!(send("p-s2"), UiOutcome::Proposed { .. }));
    term.wait_for("the editor again", STEP, |s| {
        // The banner is up before the editor is: wait for the editor itself.
        let t = s.text();
        t.contains("Proposed by claude-code") && t.contains("Split into 2")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    let wanted = [
        "tip on top",
        "change the first line",
        "add the bee file",
        "base",
    ];
    let deadline = Instant::now() + STEP;
    while subjects(&repo) != wanted {
        assert!(
            Instant::now() < deadline,
            "history is {:?}, wanted {wanted:?}",
            subjects(&repo)
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    wait_status(dir, session, "p-s2", ProposalStatus::Accepted);
}
