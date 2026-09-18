//! `gitpoulpi --add` / `gp -a` against a real running TUI: the binary and its
//! embedded daemon in a pty, and the client half (`gitpoulpi_daemon::join`,
//! the code both `gp` and `gitpoulpi` run) standing in for the second
//! terminal. The CLI's own JSON is covered by `gitpoulpi-cli/tests/add.rs`.
//!
//! Every wait names the state it waits for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_core::domain::{OperationKind, RepoId};
use gitpoulpi_daemon::join::{self, AddArgs, AddOutcome, SessionViewers};
use gitpoulpi_ipc::ui::{DeferReason, UiOutcome, UiQueryArgs, ViewerInfo};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture(file: &str, message: &str) -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(file, "one\n", message)
        .unwrap()
        .build()
}

struct Rig {
    term: Terminal,
    config: tempfile::TempDir,
    session: String,
    rt: tokio::runtime::Runtime,
    _repo: FixtureRepo,
}

fn launch(session: &str, repo: FixtureRepo, ready: &'static str) -> Rig {
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready(ready)
        .start();
    common::wait_for_viewer(config.path(), session);
    Rig {
        term,
        config,
        session: session.into(),
        rt: tokio::runtime::Runtime::new().unwrap(),
        _repo: repo,
    }
}

impl Rig {
    /// What the scan of `sessions/*.port` would find, read from this rig's
    /// own config directory (the scan itself reads the real one).
    fn found(&self) -> Vec<SessionViewers> {
        let port: u16 = std::fs::read_to_string(
            self.config
                .path()
                .join("sessions")
                .join(format!("{}.port", self.session)),
        )
        .unwrap()
        .trim()
        .parse()
        .unwrap();
        let viewers: Vec<ViewerInfo> = serde_json::from_value(common::daemon_call(
            self.config.path(),
            &self.session,
            OperationKind::UiQuery,
            UiQueryArgs::Viewers,
        ))
        .unwrap();
        vec![SessionViewers {
            session: self.session.clone(),
            port,
            viewers,
        }]
    }

    /// `gp -a <path>` from a second terminal: no question asked (one session).
    fn add(&self, path: &Path) -> AddOutcome {
        let args = AddArgs {
            path: Some(path.to_path_buf()),
            session: None,
            no_raise: true,
        };
        self.rt
            .block_on(join::add_with(
                &args,
                self.found(),
                false,
                &mut std::io::empty(),
                &mut Vec::new(),
            ))
            .expect("add")
    }

    fn wait_for(&self, what: &str, f: impl Fn(&str) -> bool) {
        self.term
            .wait_for(what, STEP, |s| f(&s.text()))
            .unwrap_or_else(|e| panic!("{e}"));
    }

    fn tabs(&self) -> Vec<String> {
        common::ui_state(self.config.path(), &self.session)
            .tabs
            .into_iter()
            .map(|t| t.path)
            .collect()
    }
}

fn joined(outcome: AddOutcome) -> join::Joined {
    match outcome {
        AddOutcome::Joined(j) => *j,
        other => panic!("expected Joined, got {other:?}"),
    }
}

#[test]
fn a_folder_added_from_another_terminal_becomes_a_tab_of_the_running_window() {
    let rig = launch(
        "pty-add-tab",
        fixture("a.txt", "first repo commit"),
        "first repo commit",
    );
    let other = fixture("o.txt", "second repo commit");
    assert_eq!(rig.tabs().len(), 1);

    // From a sub-folder, like typing `gp -a` in the middle of a project.
    let sub = other.root().join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    let j = joined(rig.add(&sub));
    assert!(matches!(j.outcome, UiOutcome::Applied { .. }), "{j:?}");
    assert!(!j.already_open);
    assert_eq!(j.session, "pty-add-tab");
    assert_eq!(j.repo, RepoId::from_path(other.root()));

    rig.wait_for("the new tab's Log", |t| t.contains("second repo commit"));
    assert_eq!(rig.tabs().len(), 2, "{:?}", rig.tabs());
    // The user's own command, not an agent's: no "shown by" toast, no mark.
    let text = rig.term.text();
    assert!(!text.contains("shown by"), "{text}");

    // The same folder again shows the open tab instead of adding a second.
    let j = joined(rig.add(other.root()));
    assert!(j.already_open);
    assert_eq!(rig.tabs().len(), 2);
}

#[test]
fn the_added_tab_is_not_held_back_by_a_key_pressed_a_moment_ago() {
    // An agent's view change waits out the user's last key; `gp -a` is the
    // user's own command and applies at once.
    let mut rig = launch(
        "pty-add-recent",
        fixture("a.txt", "first repo commit"),
        "first repo commit",
    );
    let other = fixture("o.txt", "second repo commit");
    rig.term.send_keys("j").unwrap();
    let j = joined(rig.add(other.root()));
    assert!(matches!(j.outcome, UiOutcome::Applied { .. }), "{j:?}");
    rig.wait_for("the new tab's Log", |t| t.contains("second repo commit"));
}

#[test]
fn a_window_with_a_form_open_offers_the_tab_instead_of_yanking_the_screen() {
    let dirty = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .build();
    let mut rig = launch("pty-add-busy", dirty, "// WIP");
    let other = fixture("o.txt", "second repo commit");
    rig.term.send_keys("c").expect("focus the composer");
    rig.term.send_text("subject").expect("type");
    rig.wait_for("the typed summary", |t| t.contains("subject"));

    let j = joined(rig.add(other.root()));
    assert_eq!(
        j.outcome,
        UiOutcome::Deferred {
            reason: DeferReason::InputBusy
        }
    );
    assert_eq!(join::exit_code(&AddOutcome::Joined(Box::new(j))), 3);
    assert_eq!(rig.tabs().len(), 1, "nothing was opened behind the form");
}

#[test]
fn gp_dash_a_in_a_second_terminal_opens_a_tab_of_the_first_and_exits() {
    let rig = launch(
        "pty-add-e2e",
        fixture("a.txt", "first repo commit"),
        "first repo commit",
    );
    let other = fixture("o.txt", "second repo commit");
    // The second terminal: `gp -a` with no session named (the one running is
    // found), from inside the other repository.
    let second = common::launch(other.root())
        .config_dir(rig.config.path())
        .implicit_session()
        .no_exit_pause()
        .arg("-a")
        .arg("--no-raise")
        .detached()
        .spawn();
    rig.wait_for("the new tab's Log", |t| t.contains("second repo commit"));
    assert_eq!(rig.tabs().len(), 2, "{:?}", rig.tabs());
    // What the second terminal said before it exited. Not on Windows: a
    // ConPTY may drop the output of a process that exits at once.
    if !cfg!(windows) {
        assert!(
            second
                .text()
                .contains("as a new tab in session `pty-add-e2e`"),
            "{}",
            second.text()
        );
    }
}

#[test]
fn gp_dash_a_with_nothing_running_starts_normally_on_that_folder() {
    let repo = fixture("o.txt", "plain start commit");
    let config = tempfile::tempdir().unwrap();
    let term = common::launch(repo.root())
        .config_dir(config.path())
        .implicit_session()
        .no_exit_pause()
        .arg("-a")
        .ready("plain start commit")
        .spawn();
    assert!(term.text().contains("plain start commit"));
}
