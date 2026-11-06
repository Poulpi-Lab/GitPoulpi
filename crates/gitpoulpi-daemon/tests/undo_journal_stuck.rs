//! Stuck-states slice S6: the undo journal. A resolution made while a
//! conflicted stash pop is in the tree is undoable (#10), a blocked top
//! entry can be forgotten (#13), and a conflicted merge finished outside
//! the app is settled when the state is read (#14). Both backends.

use std::sync::Arc;

use gitpoulpi_core::domain::{
    CommitOptions, ConflictEntry, ConflictResolution, Direction, JournalState, OperationKind,
    RepoId, UndoOutcome, UndoRefusal,
};
use gitpoulpi_core::{GitBackend, TypedError};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request, ResponseError};
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
}

impl Rig {
    async fn new(backend: Arc<dyn GitBackend>) -> Rig {
        // These tests are about a blocked entry: outside changes are not
        // recorded here (outside_journal.rs covers them recorded).
        std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .build();
        let handle = Daemon::new("undo-journal-stuck-test")
            .spawn(backend)
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
        }
    }

    fn commit(&self, file: &str, contents: &str, message: &str) {
        std::fs::write(self.repo.root().join(file), contents).unwrap();
        self.repo.git(&["add", file]);
        self.repo.git(&["commit", "-q", "-m", message]);
    }

    async fn call(&mut self, command: OperationKind, args: Value) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "undo-journal-stuck-test".into(),
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

    async fn journal(&mut self) -> JournalState {
        serde_json::from_value(self.ok(OperationKind::JournalState, json!({})).await).unwrap()
    }

    async fn entries(&mut self) -> Vec<ConflictEntry> {
        serde_json::from_value(self.ok(OperationKind::ConflictEntries, json!({})).await).unwrap()
    }

    async fn forget(&mut self, args: Value) -> Result<UndoOutcome, ResponseError> {
        let mut args = args;
        args["forget"] = json!(true);
        self.call(OperationKind::Undo, args)
            .await
            .map(|v| serde_json::from_value(v).unwrap())
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn a_resolution_of_a_stash_pop_conflict_is_undone_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.commit("f.txt", "base\n", "base");
        std::fs::write(rig.repo.root().join("f.txt"), "stashed\n").unwrap();
        rig.repo.git(&["stash", "push", "-q"]);
        rig.commit("f.txt", "main\n", "main change");
        // The pop conflicts: unmerged paths, no merge/rebase running.
        let _ = rig.call(OperationKind::StashPop, json!({ "id": 0 })).await;
        let before = rig.entries().await;
        assert_eq!(before.len(), 1, "{name}: the pop left a conflict");

        rig.ok(
            OperationKind::ResolveConflict,
            json!({ "path": "f.txt", "resolution": ConflictResolution::UseContent("both\n".into()) }),
        )
        .await;
        assert!(rig.entries().await.is_empty(), "{name}");
        let state = rig.journal().await;
        assert_eq!(
            state.undo.as_ref().unwrap().label,
            "Resolve f.txt",
            "{name}"
        );
        assert_eq!(state.blocked, None, "{name}: undoable");

        let out: UndoOutcome =
            serde_json::from_value(rig.ok(OperationKind::Undo, json!({})).await).unwrap();
        assert_eq!(out.direction, Direction::Undo, "{name}");
        assert_eq!(rig.entries().await, before, "{name}: the conflict is back");
        rig.ok(OperationKind::Redo, json!({})).await;
        assert!(rig.entries().await.is_empty(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_blocked_top_entry_can_be_forgotten_and_the_one_below_undone() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        std::fs::write(rig.repo.root().join("a.txt"), "a\n").unwrap();
        rig.repo.git(&["add", "a.txt"]);
        rig.ok(
            OperationKind::Commit,
            json!({ "message": "one", "opts": CommitOptions::default() }),
        )
        .await;
        let tip = rig.repo.git(&["rev-parse", "HEAD"]);
        rig.ok(
            OperationKind::BranchCreate,
            json!({ "name": "side", "at": tip.trim() }),
        )
        .await;
        // Not blocked yet: forgetting is refused, nothing changes.
        assert!(
            rig.forget(json!({ "confirmed": true })).await.is_err(),
            "{name}"
        );
        // The repository moves on outside: the top entry is stuck.
        rig.repo.git(&["branch", "-D", "side"]);
        let state = rig.journal().await;
        assert!(
            matches!(state.blocked, Some(UndoRefusal::RefMoved { .. })),
            "{name}"
        );
        let head = rig.repo.git(&["rev-parse", "HEAD"]);

        // Without confirmation: asked, nothing dropped.
        let e = rig.forget(json!({})).await.unwrap_err();
        assert!(
            matches!(e.kind, Some(TypedError::NeedsConfirmation(_))),
            "{name}"
        );
        assert!(rig.journal().await.blocked.is_some(), "{name}");
        // A dry run changes nothing either.
        let dry = rig.forget(json!({ "dry_run": true })).await.unwrap();
        assert!(!dry.applied, "{name}");

        let out = rig.forget(json!({ "confirmed": true })).await.unwrap();
        assert!(out.applied, "{name}");
        assert_eq!(
            rig.repo.git(&["rev-parse", "HEAD"]),
            head,
            "{name}: repo untouched"
        );
        let state = rig.journal().await;
        assert_eq!(state.blocked, None, "{name}");
        assert!(state.undo.unwrap().label.starts_with("Commit"), "{name}");
        let out: UndoOutcome =
            serde_json::from_value(rig.ok(OperationKind::Undo, json!({})).await).unwrap();
        assert!(out.applied, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_merge_finished_outside_is_settled_when_the_state_is_read() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.commit("c.txt", "base\n", "base");
        rig.repo.git(&["branch", "feature"]);
        rig.repo.git(&["checkout", "-q", "feature"]);
        rig.commit("c.txt", "theirs\n", "theirs");
        rig.repo.git(&["checkout", "-q", "main"]);
        rig.commit("c.txt", "ours\n", "ours");
        let refs = rig.ok(OperationKind::Refs, json!({})).await;
        let feature = refs
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["kind"]["LocalBranch"]["name"] == "feature")
            .unwrap()
            .clone();
        rig.ok(OperationKind::Merge, json!({ "from": feature }))
            .await;
        // Concluded with plain git, outside the app.
        std::fs::write(rig.repo.root().join("c.txt"), "both\n").unwrap();
        rig.repo.git(&["add", "c.txt"]);
        rig.repo.git(&["commit", "-q", "-m", "merge feature"]);
        // Reading the state alone settles the pending entry.
        let state = rig.journal().await;
        assert_eq!(state.blocked, None, "{name}: {:?}", state.blocked);
        assert!(
            state
                .undo
                .as_ref()
                .unwrap()
                .label
                .starts_with("Merge feature"),
            "{name}: {:?}",
            state.undo
        );
        rig.stop().await;
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
