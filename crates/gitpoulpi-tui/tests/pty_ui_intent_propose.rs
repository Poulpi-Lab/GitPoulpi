//! An agent's proposals (`ui propose reword|commit-message|branch|tag`, mcp
//! show-ui section 11) end to end: the real binary and its embedded daemon
//! in a pty, and a plain IPC client standing in for the commands
//! (`common::ui_show`). A proposal opens the dialog pre-filled and marked
//! "Proposed by"; only the user's own confirm changes the repository.
//!
//! Every wait names the state it waits for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_core::domain::{Oid, RepoId};
use gitpoulpi_ipc::ui::{DeferReason, Proposal, ProposalStatus, UiIntent, UiOutcome};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .commit_file("b.txt", "two\n", "second commit")
        .unwrap()
        .build()
}

/// A staged edit: a WIP row, so the composer exists.
fn dirty_fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .build()
}

struct Rig {
    term: Terminal,
    config: tempfile::TempDir,
    session: String,
    repo: FixtureRepo,
}

fn launch(session: &str, repo: FixtureRepo, ready: &'static str) -> Rig {
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready(ready)
        .start();
    common::wait_for_viewer(config.path(), session);
    // Past the "the user just pressed a key" deferral window.
    std::thread::sleep(Duration::from_millis(1700));
    Rig {
        term,
        config,
        session: session.into(),
        repo,
    }
}

impl Rig {
    fn dir(&self) -> &Path {
        self.config.path()
    }

    fn repo_id(&self) -> RepoId {
        RepoId::from_path(self.repo.root())
    }

    fn propose(&self, id: &str, proposal: Proposal) -> UiOutcome {
        common::ui_show(
            self.dir(),
            &self.session,
            UiIntent::Propose {
                repo: self.repo_id(),
                id: id.into(),
                proposal,
            },
        )
    }

    fn head(&self) -> Oid {
        Oid::from_hex(self.repo.git(&["rev-parse", "HEAD"]).trim()).expect("an object id")
    }

    fn status_of(&self, id: &str) -> Option<ProposalStatus> {
        common::ui_state(self.dir(), &self.session)
            .proposals
            .into_iter()
            .find(|p| p.id == id)
            .map(|p| p.status)
    }

    fn wait_status(&self, id: &str, want: ProposalStatus) {
        let deadline = std::time::Instant::now() + STEP;
        loop {
            let got = self.status_of(id);
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

    fn wait_for(&self, what: &str, f: impl Fn(&str) -> bool) {
        self.term
            .wait_for(what, STEP, |s| f(&s.text()))
            .unwrap_or_else(|e| panic!("{e}"));
    }

    fn quiet(&self) {
        std::thread::sleep(Duration::from_millis(1700));
    }
}

fn proposed(out: UiOutcome, id: &str) {
    match out {
        UiOutcome::Proposed { id: got, state } => {
            assert_eq!(got, id);
            assert!(
                state.proposals.iter().any(|p| p.id == id),
                "the state lists it: {state:?}"
            );
        }
        other => panic!("expected Proposed, got {other:?}"),
    }
}

#[test]
fn a_reword_proposal_is_rejected_by_esc_then_accepted_by_its_confirm() {
    let mut rig = launch("pty-ui-propose-reword", fixture(), "second commit");
    let head = rig.head();
    let reword = |message: &str| Proposal::Reword {
        commit: head,
        message: message.into(),
    };

    // Opened pre-filled, with the banner; Esc cancels and nothing changes.
    proposed(
        rig.propose("p-r1", reword("better subject\n\nwith a body")),
        "p-r1",
    );
    rig.wait_for("the popup pre-filled and marked", |t| {
        t.contains("Reword commit message")
            && t.contains("Proposed by claude-code")
            && t.contains("better subject")
            && t.contains("with a body")
    });
    assert_eq!(rig.status_of("p-r1"), Some(ProposalStatus::Open));
    rig.term.send_keys("Esc").unwrap();
    rig.wait_for("the popup closed", |t| !t.contains("Reword commit message"));
    rig.wait_status("p-r1", ProposalStatus::Rejected);
    assert_eq!(
        rig.repo.git(&["log", "-1", "--format=%s"]).trim(),
        "second commit",
        "nothing was reworded"
    );

    // Again, confirmed with the popup's own Apply key: reworded, accepted.
    rig.quiet();
    proposed(rig.propose("p-r2", reword("better subject")), "p-r2");
    rig.wait_for("the popup again", |t| {
        t.contains("Proposed by claude-code") && t.contains("better subject")
    });
    rig.term.send_keys("C-s").unwrap();
    rig.wait_for("the Log shows the new message", |t| {
        !t.contains("Reword commit message") && t.contains("better subject")
    });
    rig.wait_status("p-r2", ProposalStatus::Accepted);
    let deadline = std::time::Instant::now() + STEP;
    while rig.repo.git(&["log", "-1", "--format=%s"]).trim() != "better subject" {
        assert!(
            std::time::Instant::now() < deadline,
            "the commit was reworded"
        );
        std::thread::sleep(Duration::from_millis(150));
    }
}

#[test]
fn a_commit_message_proposal_while_typing_waits_on_the_chip() {
    let mut rig = launch("pty-ui-propose-defer", dirty_fixture(), "// WIP");
    rig.term.send_keys("c").expect("focus the composer");
    rig.term.send_text("mine").expect("type");
    rig.wait_for("the typed summary", |t| t.contains("mine"));

    let out = rig.propose(
        "p-c1",
        Proposal::CommitMessage {
            summary: "feat: proposed".into(),
            body: String::new(),
        },
    );
    assert_eq!(
        out,
        UiOutcome::Deferred {
            reason: DeferReason::InputBusy
        }
    );
    rig.wait_for("the chip", |t| {
        t.contains("claude-code proposes a commit message")
    });
    assert_eq!(rig.status_of("p-c1"), Some(ProposalStatus::Offered));
    // The user's text is untouched.
    let t = rig.term.screen().text();
    assert!(t.contains("mine") && !t.contains("feat: proposed"), "{t}");
}

#[test]
fn a_commit_message_proposal_fills_the_composer_and_never_commits() {
    let rig = launch("pty-ui-propose-composer", dirty_fixture(), "// WIP");
    proposed(
        rig.propose(
            "p-c2",
            Proposal::CommitMessage {
                summary: "feat: proposed".into(),
                body: "the body".into(),
            },
        ),
        "p-c2",
    );
    rig.wait_for("the composer filled and the banner", |t| {
        t.contains("feat: proposed") && t.contains("Proposed by claude-code: a commit message")
    });
    std::thread::sleep(Duration::from_millis(800));
    assert_eq!(
        rig.repo.git(&["log", "-1", "--format=%s"]).trim(),
        "initial commit",
        "nothing was committed"
    );
    assert_eq!(rig.status_of("p-c2"), Some(ProposalStatus::Open));
}

#[test]
fn a_branch_proposal_opens_the_box_and_enter_creates_it() {
    let mut rig = launch("pty-ui-propose-branch", fixture(), "second commit");
    let head = rig.head();
    proposed(
        rig.propose(
            "p-b1",
            Proposal::Branch {
                name: "feature/proposed".into(),
                start_point: head,
            },
        ),
        "p-b1",
    );
    rig.wait_for("the box pre-filled and the banner", |t| {
        t.contains("feature/proposed") && t.contains("Proposed by claude-code: branch")
    });
    assert!(
        rig.repo
            .git(&["branch", "--list", "feature/proposed"])
            .trim()
            .is_empty(),
        "not created before the user confirms"
    );
    // Past the Enter guard.
    std::thread::sleep(Duration::from_millis(700));
    rig.term.send_keys("Enter").unwrap();
    rig.wait_status("p-b1", ProposalStatus::Accepted);
    let deadline = std::time::Instant::now() + STEP;
    while rig
        .repo
        .git(&["branch", "--list", "feature/proposed"])
        .trim()
        .is_empty()
    {
        assert!(
            std::time::Instant::now() < deadline,
            "the branch was created"
        );
        std::thread::sleep(Duration::from_millis(150));
    }
}

#[test]
fn an_annotated_tag_proposal_prefills_the_name_box_and_the_message_dialog_and_an_edit_is_reported()
{
    let mut rig = launch("pty-ui-propose-tag", fixture(), "second commit");
    let head = rig.head();
    proposed(
        rig.propose(
            "p-t1",
            Proposal::Tag {
                name: "v1.0".into(),
                target: head,
                message: Some("first release".into()),
            },
        ),
        "p-t1",
    );
    rig.wait_for("the name box and the banner", |t| {
        t.contains("v1.0") && t.contains("Proposed by claude-code: tag v1.0")
    });
    std::thread::sleep(Duration::from_millis(700));
    rig.term.send_keys("Enter").unwrap();
    // The message step is a dialog now, pre-filled with the proposed message.
    let step = rig
        .term
        .wait_for("the message dialog pre-filled", STEP, |s| {
            s.contains("New annotated tag v1.0 at") && s.contains("first release")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !common::status_line_text(&step).contains("first release"),
        "the message is not echoed in the status line"
    );
    rig.term.send_text("!").unwrap();
    // Enter breaks the line in the message area; the apply chord submits.
    rig.term.send_keys("C-s").unwrap();
    rig.wait_status("p-t1", ProposalStatus::AcceptedEdited);
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let message = rig.repo.git(&["tag", "-l", "-n1", "v1.0"]);
        if message.contains("first release!") {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "tag: {message}");
        std::thread::sleep(Duration::from_millis(150));
    }
}
