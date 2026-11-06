//! Undo/redo end to end through the daemon (the undo-redo design):
//! the journal, its persistence, the refusals, and the live
//! event. Every test runs against both backends.

use std::sync::Arc;
use std::time::Duration;

use gitpoulpi_core::domain::{
    CheckoutTarget, CommitOptions, Direction, EntryState, JournalState, LogOptions, Oid,
    OperationKind, RepoId, StageTarget, UndoOutcome, UndoRefusal,
};
use gitpoulpi_core::{GitBackend, TypedError};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Event, Request, ResponseError, ServerMessage};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};

fn backends() -> Vec<(&'static str, Arc<dyn GitBackend>)> {
    vec![
        (
            "system",
            Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new()),
        ),
        ("git2", Arc::new(gitpoulpi_git2_backend::Git2Backend::new())),
    ]
}

struct Rig {
    handle: Option<gitpoulpi_daemon::DaemonHandle>,
    client: Client,
    repo: FixtureRepo,
    id: RepoId,
    next: u64,
    backend: Arc<dyn GitBackend>,
}

impl Rig {
    async fn new(backend: Arc<dyn GitBackend>) -> Rig {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .build();
        Self::on(backend, repo).await
    }

    async fn on(backend: Arc<dyn GitBackend>, repo: FixtureRepo) -> Rig {
        // These tests are about a blocked entry: outside changes are not
        // recorded here (outside_journal.rs covers them recorded).
        std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
        // These tests stack several operations: unlimited history (the
        // depth limits are covered by `undo_depth.rs`), and never the
        // developer's own config.
        static ENV: std::sync::Once = std::sync::Once::new();
        ENV.call_once(|| std::env::set_var("GITPOULPI_GENERAL_UNDO_DEPTH", "0"));
        let handle = Daemon::new("undo-test")
            .spawn(Arc::clone(&backend))
            .await
            .unwrap();
        let client = Client::connect(&handle.endpoint()).await.unwrap();
        let id = repo.repo_ref().id;
        Rig {
            handle: Some(handle),
            client,
            repo,
            id,
            next: 1,
            backend,
        }
    }

    async fn call(&mut self, command: OperationKind, args: Value) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "undo-test".into(),
                repo: Some(self.id.clone()),
                command,
                args,
            })
            .await
            .unwrap()
            .result
    }

    async fn ok(&mut self, command: OperationKind, args: Value) -> Value {
        self.call(command, args)
            .await
            .unwrap_or_else(|e| panic!("{command:?} failed: {}", e.message))
    }

    /// Writes `name` (staged) and commits it through the daemon.
    async fn commit_file(&mut self, name: &str, message: &str) -> Oid {
        std::fs::write(self.repo.root().join(name), name).unwrap();
        self.ok(
            OperationKind::StageFile,
            json!({ "target": StageTarget::File(name.into()) }),
        )
        .await;
        let oid = self
            .ok(
                OperationKind::Commit,
                json!({ "message": message, "opts": CommitOptions::default() }),
            )
            .await;
        serde_json::from_value(oid).unwrap()
    }

    async fn undo(&mut self) -> UndoOutcome {
        serde_json::from_value(self.ok(OperationKind::Undo, json!({})).await).unwrap()
    }

    async fn redo(&mut self) -> UndoOutcome {
        serde_json::from_value(self.ok(OperationKind::Redo, json!({})).await).unwrap()
    }

    async fn refused(&mut self, command: OperationKind, args: Value) -> UndoRefusal {
        let e = self.call(command, args).await.unwrap_err();
        match e.kind {
            Some(TypedError::UndoRefused(r)) => r,
            other => panic!("expected an undo refusal, got {other:?} ({})", e.message),
        }
    }

    async fn state(&mut self) -> JournalState {
        serde_json::from_value(self.ok(OperationKind::JournalState, json!({})).await).unwrap()
    }

    fn head(&self) -> Oid {
        Oid::from_hex(&self.repo.git(&["rev-parse", "HEAD"])).unwrap()
    }

    async fn stop(mut self) -> FixtureRepo {
        self.handle.take().unwrap().shutdown().await.unwrap();
        self.repo
    }
}

#[tokio::test]
async fn commit_undo_brings_the_changes_back_staged_and_redo_restores_the_same_oid() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let parent = rig.head();
        let commit = rig.commit_file("feature.txt", "add feature").await;
        assert_eq!(rig.head(), commit);

        let out = rig.undo().await;
        assert!(out.applied, "{name}");
        assert!(
            out.entry.label.starts_with("Commit \"add feature\""),
            "{name}"
        );
        assert_eq!(rig.head(), parent, "{name}: HEAD is the parent again");
        assert_eq!(
            rig.repo.git(&["status", "--porcelain"]),
            "A  feature.txt",
            "{name}: the commit's changes come back staged"
        );

        // The stage that preceded the commit is the next thing to undo.
        let state = rig.state().await;
        assert_eq!(
            state.undo.as_ref().map(|e| e.kind),
            Some(OperationKind::StageFile),
            "{name}"
        );
        assert!(state.redo.is_some(), "{name}");

        rig.redo().await;
        assert_eq!(
            rig.head(),
            commit,
            "{name}: redo restores, it never re-commits"
        );
        assert_eq!(rig.repo.git(&["status", "--porcelain"]), "", "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_commit_made_outside_between_the_op_and_the_undo_is_refused_until_put_back() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let ours = rig.commit_file("a.txt", "ours").await;
        rig.repo
            .git(&["commit", "--allow-empty", "-m", "from the shell"]);
        let theirs = rig.head();
        assert_ne!(theirs, ours);

        let refusal = rig.refused(OperationKind::Undo, json!({})).await;
        match &refusal {
            UndoRefusal::RefMoved {
                name: r,
                expected,
                actual,
                ..
            } => {
                assert_eq!(r, "refs/heads/main", "{name}");
                assert_eq!(*expected, Some(ours), "{name}");
                assert_eq!(*actual, Some(theirs), "{name}");
            }
            other => panic!("{name}: {other:?}"),
        }
        assert!(refusal.message().contains("outside GitPoulpi"));
        assert_eq!(rig.head(), theirs, "{name}: nothing changed");
        assert!(
            rig.state().await.blocked.is_some(),
            "{name}: the button is blocked"
        );

        // The user puts things back; the entry is valid again.
        rig.repo.git(&["reset", "--soft", "HEAD~1"]);
        assert_eq!(rig.head(), ours);
        assert!(rig.state().await.blocked.is_none(), "{name}");
        rig.undo().await;
        rig.stop().await;
    }
}

#[tokio::test]
async fn an_unrelated_outside_change_does_not_block() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let parent = rig.head();
        rig.commit_file("a.txt", "ours").await;
        // Other refs and the index move; nothing the entry touched does.
        rig.repo.git(&["branch", "elsewhere", &parent.to_hex()]);
        rig.repo.git(&["tag", "v9"]);
        std::fs::write(rig.repo.root().join("staged.txt"), "x").unwrap();
        rig.repo.git(&["add", "staged.txt"]);

        assert!(rig.state().await.blocked.is_none(), "{name}");
        rig.undo().await;
        assert_eq!(rig.head(), parent, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn an_in_progress_merge_blocks_with_a_clear_reason() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.commit_file("a.txt", "ours").await;
        let head = rig.head();
        std::fs::write(rig.repo.root().join(".git/MERGE_HEAD"), format!("{head}\n")).unwrap();
        let refusal = rig.refused(OperationKind::Undo, json!({})).await;
        assert_eq!(
            refusal,
            UndoRefusal::InProgress { op: "merge".into() },
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn amend_undo_restores_the_pre_amend_commit_and_redo_the_amended_one() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let original = rig.commit_file("a.txt", "first try").await;
        let amended: Oid = serde_json::from_value(
            rig.ok(
                OperationKind::Commit,
                json!({ "message": "better", "opts": CommitOptions { amend: true, ..Default::default() } }),
            )
            .await,
        )
        .unwrap();
        assert_ne!(original, amended);
        // The amended-away commit is only reachable through its pin.
        assert!(
            !rig.repo
                .git(&["for-each-ref", "refs/gitpoulpi/undo/"])
                .is_empty(),
            "{name}: pinned"
        );

        let out = rig.undo().await;
        assert!(out.entry.label.starts_with("Amend"), "{name}");
        assert_eq!(rig.head(), original, "{name}");
        rig.redo().await;
        assert_eq!(rig.head(), amended, "{name}");
        rig.undo().await;
        rig.undo().await; // the first commit, too: stack of two
        assert_eq!(rig.repo.git(&["rev-parse", "HEAD"]).len(), 40);
        rig.stop().await;
    }
}

#[tokio::test]
async fn branch_create_and_delete_round_trip_with_the_upstream() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let tip = rig.head();
        rig.ok(
            OperationKind::BranchCreate,
            json!({ "name": "feature", "at": tip }),
        )
        .await;
        assert!(!rig.repo.git(&["rev-parse", "feature"]).is_empty());
        rig.undo().await;
        assert!(
            rig.repo
                .git(&["for-each-ref", "refs/heads/feature"])
                .is_empty(),
            "{name}: undo of a create deletes the branch"
        );
        rig.redo().await;
        assert_eq!(
            rig.repo.git(&["rev-parse", "feature"]),
            tip.to_hex(),
            "{name}"
        );

        // Delete a branch that has an upstream.
        rig.repo.git(&["config", "branch.feature.remote", "origin"]);
        rig.repo
            .git(&["config", "branch.feature.merge", "refs/heads/feature"]);
        rig.ok(OperationKind::BranchDelete, json!({ "name": "feature" }))
            .await;
        assert!(rig
            .repo
            .git(&["for-each-ref", "refs/heads/feature"])
            .is_empty());
        assert!(!rig
            .repo
            .git(&["config", "--local", "--list"])
            .contains("branch.feature."));

        rig.undo().await;
        assert_eq!(
            rig.repo.git(&["rev-parse", "feature"]),
            tip.to_hex(),
            "{name}"
        );
        assert_eq!(
            rig.repo.git(&["config", "branch.feature.remote"]),
            "origin",
            "{name}"
        );
        assert_eq!(
            rig.repo.git(&["config", "branch.feature.merge"]),
            "refs/heads/feature",
            "{name}"
        );

        rig.redo().await;
        assert!(rig
            .repo
            .git(&["for-each-ref", "refs/heads/feature"])
            .is_empty());
        rig.undo().await;
        assert_eq!(
            rig.repo.git(&["rev-parse", "feature"]),
            tip.to_hex(),
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn checkout_undo_and_redo_move_head_back_and_forth() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let tip = rig.head();
        rig.ok(
            OperationKind::BranchCreate,
            json!({ "name": "feature", "at": tip }),
        )
        .await;
        rig.ok(
            OperationKind::Checkout,
            json!({ "target": CheckoutTarget::Branch("feature".into()) }),
        )
        .await;
        assert_eq!(rig.repo.git(&["branch", "--show-current"]), "feature");

        let out = rig.undo().await;
        assert_eq!(out.entry.label, "Checkout feature", "{name}");
        assert_eq!(
            rig.repo.git(&["branch", "--show-current"]),
            "main",
            "{name}"
        );
        rig.redo().await;
        assert_eq!(
            rig.repo.git(&["branch", "--show-current"]),
            "feature",
            "{name}"
        );

        // Leaving a detached HEAD pins its commit, so undoing back onto it
        // works even after the branch it came from moved away.
        rig.ok(
            OperationKind::Checkout,
            json!({ "target": CheckoutTarget::Commit(tip) }),
        )
        .await;
        assert!(rig.repo.git(&["branch", "--show-current"]).is_empty());
        rig.undo().await;
        assert_eq!(
            rig.repo.git(&["branch", "--show-current"]),
            "feature",
            "{name}"
        );
        rig.redo().await;
        assert_eq!(rig.head(), tip, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_commit_on_a_detached_head_is_undone_with_head_alone() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let tip = rig.head();
        rig.repo.git(&["checkout", "--detach"]);
        let commit = rig.commit_file("d.txt", "detached").await;
        assert_eq!(rig.head(), commit);
        rig.undo().await;
        assert_eq!(rig.head(), tip, "{name}");
        assert_eq!(
            rig.repo.git(&["status", "--porcelain"]),
            "A  d.txt",
            "{name}"
        );
        rig.redo().await;
        assert_eq!(rig.head(), commit, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn the_journal_survives_a_daemon_restart() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(Arc::clone(&backend)).await;
        let parent = rig.head();
        rig.commit_file("a.txt", "persisted").await;
        let repo = rig.stop().await;

        let mut rig = Rig::on(backend, repo).await;
        let state = rig.state().await;
        assert_eq!(
            state.undo.map(|e| e.kind),
            Some(OperationKind::Commit),
            "{name}"
        );
        rig.undo().await;
        assert_eq!(rig.head(), parent, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_pending_entry_after_a_crash_is_interrupted_and_never_undone() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .build();
        // What a daemon killed between "write-ahead" and "done" leaves.
        let mut journal = gitpoulpi_core::journal::Journal::default();
        let head = Oid::from_hex(&repo.git(&["rev-parse", "HEAD"])).unwrap();
        journal.begin(
            OperationKind::Commit,
            "Commit \"half done\"".into(),
            1,
            Some(gitpoulpi_core::domain::HeadState::Branch("main".into())),
            true,
            vec![("refs/heads/main".into(), Some(head))],
        );
        let dir = repo.root().join(".git/gitpoulpi");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("journal.json"),
            serde_json::to_vec(&journal).unwrap(),
        )
        .unwrap();

        let mut rig = Rig::on(backend, repo).await;
        let state = rig.state().await;
        assert_eq!(
            state.undo.as_ref().map(|e| e.state),
            Some(EntryState::Interrupted),
            "{name}"
        );
        assert!(matches!(
            rig.refused(OperationKind::Undo, json!({})).await,
            UndoRefusal::Interrupted { .. }
        ));
        let listing: gitpoulpi_core::domain::JournalListing =
            serde_json::from_value(rig.ok(OperationKind::JournalList, json!({})).await).unwrap();
        assert_eq!(listing.undo.len(), 1, "{name}");
        assert!(listing.undo[0].changes[0].contains("refs/heads/main"));
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_second_client_receives_journal_changed() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let handle = rig.handle.as_ref().unwrap();
        let mut watcher = Client::connect(&handle.endpoint())
            .await
            .unwrap()
            .into_stream();

        rig.commit_file("a.txt", "watch me").await;
        // Staging is journaled too: its event comes first.
        let state = next_journal_event(&mut watcher, &rig.id).await;
        assert_eq!(state.undo.unwrap().kind, OperationKind::StageFile, "{name}");
        let state = next_journal_event(&mut watcher, &rig.id).await;
        assert_eq!(state.undo.unwrap().kind, OperationKind::Commit, "{name}");

        rig.undo().await;
        let state = next_journal_event(&mut watcher, &rig.id).await;
        assert!(state.redo.is_some(), "{name}");
        assert_eq!(
            state.undo.map(|e| e.kind),
            Some(OperationKind::StageFile),
            "{name}"
        );
        rig.stop().await;
    }
}

async fn next_journal_event(stream: &mut tokio::net::TcpStream, repo: &RepoId) -> JournalState {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let msg: ServerMessage = gitpoulpi_ipc::transport::read_message(stream)
                .await
                .unwrap()
                .expect("connection closed before the event");
            if let ServerMessage::Event(Event::JournalChanged { repo: r, state }) = msg {
                if &r == repo {
                    return state;
                }
            }
        }
    })
    .await
    .expect("timed out waiting for JournalChanged")
}

#[tokio::test]
async fn an_expectation_that_is_not_the_top_entry_is_stale() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let parent = rig.head();
        rig.commit_file("a.txt", "one").await;
        let seen = rig.state().await.undo.unwrap().id;
        rig.commit_file("b.txt", "two").await; // another client moved on
        let refusal = rig
            .refused(OperationKind::Undo, json!({ "expect": seen }))
            .await;
        assert!(
            matches!(refusal, UndoRefusal::StaleEntry { expected_id, .. } if expected_id == seen),
            "{name}"
        );
        let top = rig.state().await.undo.unwrap().id;
        rig.ok(OperationKind::Undo, json!({ "expect": top })).await;
        assert_ne!(rig.head(), parent);
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_dry_run_describes_the_steps_and_changes_nothing() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let commit = rig.commit_file("a.txt", "dry").await;
        let before_journal =
            std::fs::read(rig.repo.root().join(".git/gitpoulpi/journal.json")).unwrap();
        let out: UndoOutcome = serde_json::from_value(
            rig.ok(OperationKind::Undo, json!({ "dry_run": true }))
                .await,
        )
        .unwrap();
        assert!(!out.applied);
        assert_eq!(out.direction, Direction::Undo);
        assert!(
            out.steps.iter().any(|s| s.contains("move main")),
            "{name}: {:?}",
            out.steps
        );
        assert_eq!(rig.head(), commit, "{name}");
        assert_eq!(
            std::fs::read(rig.repo.root().join(".git/gitpoulpi/journal.json")).unwrap(),
            before_journal,
            "{name}: the journal is untouched"
        );
        assert!(rig.state().await.undo.is_some());
        // Nothing to redo yet.
        assert_eq!(
            rig.refused(OperationKind::Redo, json!({ "dry_run": true }))
                .await,
            UndoRefusal::NothingToRedo
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn an_undone_commit_stays_reachable_and_pinned_after_a_new_commit() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let first = rig.commit_file("a.txt", "will be undone").await;
        rig.undo().await;
        // A new operation retires the redo stack, but its data lives on.
        let second = rig.commit_file("b.txt", "replacement").await;
        assert_ne!(first, second);
        rig.repo.git(&["cat-file", "-e", &first.to_hex()]);
        assert!(
            rig.repo
                .git(&["for-each-ref", "refs/gitpoulpi/undo/"])
                .contains(&first.to_hex()),
            "{name}: still pinned"
        );
        // ... and is invisible to the commit graph.
        let log: gitpoulpi_core::domain::CommitGraph = serde_json::from_value(
            rig.ok(
                OperationKind::Log,
                json!({ "opts": LogOptions { all_refs: true, ..Default::default() } }),
            )
            .await,
        )
        .unwrap();
        assert!(log.nodes.iter().all(|n| n.id != first), "{name}");
        assert!(rig.state().await.redo.is_none(), "{name}");
        let _ = &rig.backend;
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_failed_operation_leaves_no_entry_and_no_pins() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        // Nothing staged: git refuses an empty commit.
        let failed = rig
            .call(
                OperationKind::Commit,
                json!({ "message": "empty", "opts": CommitOptions::default() }),
            )
            .await;
        // (libgit2 would make an empty commit; either way no half-entry
        // may be left Pending.)
        let state = rig.state().await;
        if failed.is_err() {
            assert!(state.undo.is_none(), "{name}");
            assert!(rig
                .repo
                .git(&["for-each-ref", "refs/gitpoulpi/undo/"])
                .is_empty());
        } else {
            assert_eq!(state.undo.unwrap().state, EntryState::Done, "{name}");
        }
        rig.stop().await;
    }
}

/// A mutation that records no entry (here a stage) after the ref the top
/// entry depends on moved outside: clients must hear the blocked state.
#[tokio::test]
async fn a_non_journaled_mutation_that_blocks_the_top_entry_broadcasts_it() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let handle = rig.handle.as_ref().unwrap();
        let mut watcher = Client::connect(&handle.endpoint())
            .await
            .unwrap()
            .into_stream();
        let ours = rig.commit_file("a.txt", "ours").await;
        // Staging is journaled too: skip to the commit's event.
        let top = rig.state().await.undo.unwrap().id;
        let state = loop {
            let state = next_journal_event(&mut watcher, &rig.id).await;
            if state.undo.as_ref().map(|u| u.id) == Some(top) {
                break state;
            }
        };
        assert!(state.blocked.is_none(), "{name}");

        // The branch moves outside, then a stage (not journaled) runs.
        let parent = Oid::from_hex(&rig.repo.git(&["rev-parse", "HEAD~1"])).unwrap();
        assert_ne!(parent, ours);
        rig.repo.git(&["reset", "--soft", "HEAD~1"]);
        rig.ok(
            OperationKind::StageAll,
            json!({ "target": StageTarget::All }),
        )
        .await;
        let state = next_journal_event(&mut watcher, &rig.id).await;
        assert!(
            matches!(state.blocked, Some(UndoRefusal::RefMoved { .. })),
            "{name}: {:?}",
            state.blocked
        );
        rig.stop().await;
    }
}

/// A commit already on its upstream: undo asks first (`NeedsConfirmation`,
/// nothing changed); `confirmed` goes ahead; a local-only commit never asks.
#[tokio::test]
async fn undoing_a_pushed_commit_needs_confirmation() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let remote = tempfile::tempdir().unwrap();
        let bare = remote.path().join("origin.git");
        std::process::Command::new("git")
            .args(["init", "--bare", "-q"])
            .arg(&bare)
            .status()
            .unwrap();
        rig.repo
            .git(&["remote", "add", "origin", &bare.to_string_lossy()]);
        rig.repo.git(&["push", "-u", "origin", "main"]);

        let local_only = rig.commit_file("a.txt", "local").await;
        // Not pushed: no confirmation.
        assert!(rig.undo().await.applied, "{name}");
        rig.redo().await;
        assert_eq!(rig.head(), local_only);

        rig.repo.git(&["push", "origin", "main"]);
        let e = rig.call(OperationKind::Undo, json!({})).await.unwrap_err();
        match e.kind {
            Some(TypedError::NeedsConfirmation(p)) => {
                assert!(p.contains("already on origin/main"), "{name}: {p}")
            }
            other => panic!("{name}: {other:?}"),
        }
        assert_eq!(rig.head(), local_only, "{name}: nothing changed");
        let dry: UndoOutcome = serde_json::from_value(
            rig.ok(OperationKind::Undo, json!({ "dry_run": true }))
                .await,
        )
        .unwrap();
        assert!(dry.needs_confirmation.is_some(), "{name}");

        let out: UndoOutcome = serde_json::from_value(
            rig.ok(OperationKind::Undo, json!({ "confirmed": true }))
                .await,
        )
        .unwrap();
        assert!(out.applied, "{name}");
        assert_ne!(rig.head(), local_only, "{name}");
        rig.stop().await;
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
