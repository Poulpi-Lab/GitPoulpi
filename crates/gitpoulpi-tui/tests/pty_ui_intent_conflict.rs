//! An agent and the merge conflict editor (plan
//! the merge-conflict-editor design section 8, slice S8):
//! `ui show conflict` opens the editor on a file at a conflict (view only),
//! `ui propose resolution` opens it with the agent's picks or content filled
//! in, marked "Proposed by", and only the user's Save & mark resolved
//! (Ctrl+S) writes or stages anything. The real binary and its embedded
//! daemon in a pty, a plain IPC client standing in for the commands
//! (`common::ui_show`), on a real conflicted repository.

mod common;

use std::time::Duration;

use gitpoulpi_core::domain::{ConflictPicks, ConflictSide, Oid, Pick, RepoId};
use gitpoulpi_ipc::ui::{Proposal, ProposalStatus, UiIntent, UiOutcome};
use gitpoulpi_testkit::ConflictFixture;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

struct Rig {
    term: Terminal,
    config: tempfile::TempDir,
    session: String,
    fixture: ConflictFixture,
}

fn launch(session: &str, fixture: ConflictFixture) -> Rig {
    let (term, config) = common::launch(fixture.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the conflicted file in the Status pane", |s| {
            s.contains("Unstaged") && s.contains("Merge in progress")
        })
        .mouse_capture()
        .start();
    common::wait_for_viewer(config.path(), session);
    // Past the "the user just pressed a key" deferral window.
    std::thread::sleep(Duration::from_millis(1700));
    Rig {
        term,
        config,
        session: session.into(),
        fixture,
    }
}

fn picks(side: ConflictSide, lines: usize) -> ConflictPicks {
    ConflictPicks {
        picks: (0..lines).map(|line| Pick { side, line }).collect(),
        order: vec![side],
    }
}

impl Rig {
    fn repo(&self) -> RepoId {
        RepoId::from_path(self.fixture.root())
    }

    fn oids(&self, path: &str) -> [Option<Oid>; 3] {
        [1u8, 2, 3].map(|n| {
            self.fixture
                .stage_oid(n, path)
                .map(|h| Oid::from_hex(&h).expect("an object id"))
        })
    }

    fn show(&self, path: &str, conflict: Option<u32>) -> UiOutcome {
        common::ui_show(
            self.config.path(),
            &self.session,
            UiIntent::ShowConflict {
                repo: self.repo(),
                path: path.into(),
                conflict,
            },
        )
    }

    fn propose(&self, id: &str, proposal: Proposal) -> UiOutcome {
        common::ui_show(
            self.config.path(),
            &self.session,
            UiIntent::Propose {
                repo: self.repo(),
                id: id.into(),
                proposal,
            },
        )
    }

    /// a.txt: conflict 1 is two lines (3-4), conflicts 2 and 3 one line
    /// each. A, then B, then A.
    fn a_b_a(&self) -> Proposal {
        Proposal::ConflictResolution {
            path: "a.txt".into(),
            expected: self.oids("a.txt"),
            picks: Some(vec![
                picks(ConflictSide::Ours, 2),
                picks(ConflictSide::Theirs, 1),
                picks(ConflictSide::Ours, 1),
            ]),
            content: None,
        }
    }

    fn wait_for(&self, what: &str, f: impl Fn(&str) -> bool) {
        self.term
            .wait_for(what, STEP, |s| f(&s.text()))
            .unwrap_or_else(|e| panic!("{e}\n{}", self.term.text()));
    }

    fn status_of(&self, id: &str) -> Option<ProposalStatus> {
        common::ui_state(self.config.path(), &self.session)
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

    fn on_disk(&self, path: &str) -> String {
        String::from_utf8(std::fs::read(self.fixture.root().join(path)).unwrap()).unwrap()
    }

    fn unmerged(&self) -> Vec<String> {
        common::git(
            self.fixture.root(),
            &["diff", "--name-only", "--diff-filter=U"],
        )
        .lines()
        .map(str::to_string)
        .collect()
    }
}

fn applied(out: UiOutcome) {
    assert!(matches!(out, UiOutcome::Applied { .. }), "{out:?}");
}

fn proposed(out: UiOutcome, id: &str) {
    match out {
        UiOutcome::Proposed { id: got, .. } => assert_eq!(got, id),
        other => panic!("expected Proposed, got {other:?}"),
    }
}

/// `ui show conflict` opens the editor at the asked conflict and picks
/// nothing: the file and the index are exactly as they were.
#[test]
fn show_conflict_opens_the_editor_at_a_conflict_and_changes_nothing() {
    let rig = launch("pty-ui-conflict-show", ConflictFixture::multi_conflict());
    applied(rig.show("a.txt", Some(2)));
    rig.wait_for("the editor on conflict 2, nothing picked", |t| {
        t.contains("Conflict 2 / 3") && t.contains("3 left")
    });
    let state = common::ui_state(rig.config.path(), &rig.session);
    assert_eq!(state.central.as_deref(), Some("merge"));
    let merge = state.merge.expect("the merge section");
    assert_eq!(merge["path"], "a.txt");
    assert_eq!(merge["conflict"], 2);
    assert_eq!(merge["left"], 3);
    assert_eq!(merge["proposed"], false);
    assert!(rig.on_disk("a.txt").contains("<<<<<<<"));
    assert_eq!(rig.unmerged(), ["a.txt", "b.txt", "docs/c.txt"]);
}

#[test]
fn show_conflict_on_a_file_that_is_not_conflicted_is_not_found() {
    let rig = launch("pty-ui-conflict-nf", ConflictFixture::text());
    let out = rig.show("nope.txt", None);
    assert!(matches!(out, UiOutcome::NotFound { .. }), "{out:?}");
}

/// A proposal opens pre-picked with the banner; Esc then Discard rejects it
/// and the file is untouched; a second one confirmed with Ctrl+S is accepted
/// and written exactly as proposed.
#[test]
fn a_resolution_proposal_is_rejected_by_closing_and_accepted_by_ctrl_s() {
    let mut rig = launch("pty-ui-conflict-propose", ConflictFixture::multi_conflict());
    proposed(rig.propose("p-x1", rig.a_b_a()), "p-x1");
    rig.wait_for("the editor pre-picked, with the banner", |t| {
        t.contains("Conflict 1 / 3")
            && t.contains("0 left")
            && t.contains("Proposed by claude-code: a resolution of a.txt")
    });
    assert_eq!(rig.status_of("p-x1"), Some(ProposalStatus::Open));
    let state = common::ui_state(rig.config.path(), &rig.session);
    assert_eq!(state.merge.expect("merge")["proposed"], true);
    // Nothing is written or staged by a proposal.
    assert!(rig.on_disk("a.txt").contains("<<<<<<<"));
    assert_eq!(rig.unmerged().len(), 3);

    // Esc asks (the picks are unsaved work); Discard closes: rejected.
    rig.term.send_keys("Escape").unwrap();
    rig.wait_for("the unsaved-changes box", |t| t.contains("Unsaved changes"));
    rig.term.send_keys("d").unwrap();
    rig.wait_for("the editor closed", |t| !t.contains("Conflict 1 / 3"));
    rig.wait_status("p-x1", ProposalStatus::Rejected);
    assert!(rig.on_disk("a.txt").contains("<<<<<<<"));
    assert_eq!(rig.unmerged().len(), 3);

    // Again; the user's own Save & mark resolved confirms it.
    std::thread::sleep(Duration::from_millis(1700));
    proposed(rig.propose("p-x2", rig.a_b_a()), "p-x2");
    rig.wait_for("the editor pre-picked again", |t| {
        t.contains("Proposed by claude-code") && t.contains("0 left")
    });
    rig.term.send_keys("C-s").unwrap();
    rig.wait_for("the file resolved, the list back", |t| {
        t.contains("resolved a.txt") && !t.contains("Conflict 1 / 3")
    });
    rig.wait_status("p-x2", ProposalStatus::Accepted);
    let text = rig.on_disk("a.txt");
    assert!(
        text.contains("A3 ours\nA4 ours\n")
            && text.contains("A15 theirs")
            && text.contains("A28 ours")
            && !text.contains("<<<<<<<"),
        "{text}"
    );
    assert_eq!(rig.unmerged(), ["b.txt", "docs/c.txt"]);
}

/// Changing a pick before confirming is reported as `accepted_edited`.
#[test]
fn a_changed_proposal_confirmed_is_reported_as_edited() {
    let mut rig = launch("pty-ui-conflict-edit", ConflictFixture::multi_conflict());
    proposed(rig.propose("p-x3", rig.a_b_a()), "p-x3");
    rig.wait_for("the editor pre-picked", |t| {
        t.contains("Proposed by claude-code") && t.contains("0 left")
    });
    // Take B for conflict 1 instead of the proposed A.
    rig.term.send_keys("b").unwrap();
    rig.wait_for("conflict 1 now from B", |t| {
        t.contains("A3 theirs") && t.contains("0 left")
    });
    rig.term.send_keys("C-s").unwrap();
    rig.wait_status("p-x3", ProposalStatus::AcceptedEdited);
    let text = rig.on_disk("a.txt");
    assert!(
        text.contains("A3 theirs") && !text.contains("A3 ours"),
        "{text}"
    );
}

/// A whole-file proposal fills the Output.
#[test]
fn a_content_proposal_fills_the_output_for_the_user_to_confirm() {
    let mut rig = launch("pty-ui-conflict-content", ConflictFixture::text());
    let path = rig.fixture.path().to_string();
    proposed(
        rig.propose(
            "p-x4",
            Proposal::ConflictResolution {
                path: path.clone(),
                expected: rig.oids(&path),
                picks: None,
                content: Some("agent wrote this\n".into()),
            },
        ),
        "p-x4",
    );
    rig.wait_for("the proposed text in the Output", |t| {
        t.contains("agent wrote this") && t.contains("Proposed by claude-code")
    });
    assert!(rig.on_disk(&path).contains("<<<<<<<"));
    rig.term.send_keys("C-s").unwrap();
    rig.wait_status("p-x4", ProposalStatus::Accepted);
    assert_eq!(rig.on_disk(&path), "agent wrote this\n");
    assert!(rig.unmerged().is_empty());
}

/// A proposal made against stages that have since moved is refused, and a
/// proposal while the user has unsaved picks waits on the chip.
#[test]
fn a_stale_proposal_is_refused_and_one_over_unsaved_work_waits() {
    let mut rig = launch("pty-ui-conflict-stale", ConflictFixture::multi_conflict());
    let stale = Proposal::ConflictResolution {
        path: "a.txt".into(),
        expected: [None, Some(Oid([9; 20])), None],
        picks: Some(vec![
            picks(ConflictSide::Ours, 2),
            picks(ConflictSide::Ours, 1),
            picks(ConflictSide::Ours, 1),
        ]),
        content: None,
    };
    let out = rig.propose("p-x5", stale);
    assert!(matches!(out, UiOutcome::Refused { .. }), "{out:?}");
    assert_eq!(rig.status_of("p-x5"), Some(ProposalStatus::Failed));

    // The user opens the editor and picks: unsaved work.
    let (c, r) = rig
        .term
        .wait_for_text("! a.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.click(c + 2, r).unwrap();
    rig.term.send_keys("Enter").unwrap();
    rig.wait_for("the editor", |t| t.contains("Conflict 1 / 3"));
    rig.term.send_keys("a").unwrap();
    rig.wait_for("a pick", |t| t.contains("2 left"));
    std::thread::sleep(Duration::from_millis(1700));
    let out = rig.propose("p-x6", rig.a_b_a());
    assert!(matches!(out, UiOutcome::Deferred { .. }), "{out:?}");
    assert!(rig.term.text().contains("2 left"), "the user's picks stay");
    rig.term.stop().unwrap();
}
