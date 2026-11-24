//! A tag that differs between here and a remote, through the daemon: "use the
//! remote's version" (local, one undo entry that puts the old object back,
//! annotated or not) and "force push mine" (asks first, leased on the tag
//! mirror, one undo entry that pushes the old value back with a lease).
//! The remote is a local bare repository, never a real server. Every test
//! runs against both backends.

use std::sync::Arc;

use gitpoulpi_core::domain::{
    Direction, JournalState, Oid, OperationKind, PushRejectReason, RepoId, UndoOutcome, UndoRefusal,
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
    /// `v1` is on `origin` (annotated when `remote_annotated`, else
    /// lightweight) at the first commit and known to the tag mirror; here it
    /// is re-made at a second commit (annotated when not `remote_annotated`).
    async fn differing(backend: Arc<dyn GitBackend>, remote_annotated: bool) -> Rig {
        std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .with_origin()
            .unwrap()
            .push("main")
            .unwrap()
            .build();
        if remote_annotated {
            repo.git(&["tag", "-a", "v1", "-m", "remote notes"]);
        } else {
            repo.git(&["tag", "v1"]);
        }
        let handle = Daemon::new("undo-tag-differs-test")
            .spawn(backend)
            .await
            .unwrap();
        let client = Client::connect(&handle.endpoint()).await.unwrap();
        let id = repo.repo_ref().id;
        let mut rig = Rig {
            handle: Some(handle),
            client,
            repo,
            id,
            next: 1,
        };
        rig.ok(
            OperationKind::TagPush,
            json!({ "remote": "origin", "selection": { "Named": ["v1"] } }),
        )
        .await;
        rig.repo.git(&["commit", "--allow-empty", "-m", "second"]);
        if remote_annotated {
            rig.repo.git(&["tag", "-f", "v1"]);
        } else {
            rig.repo.git(&["tag", "-f", "-a", "v1", "-m", "my notes"]);
        }
        rig
    }

    async fn call(&mut self, command: OperationKind, args: Value) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "undo-tag-differs-test".into(),
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

    async fn undo_prompt(&mut self) -> String {
        let e = self.call(OperationKind::Undo, json!({})).await.unwrap_err();
        match e.kind {
            Some(TypedError::NeedsConfirmation(prompt)) => prompt,
            other => panic!("expected a confirmation, got {other:?} ({})", e.message),
        }
    }

    async fn undo(&mut self) -> UndoOutcome {
        let out: UndoOutcome = serde_json::from_value(
            self.ok(OperationKind::Undo, json!({ "confirmed": true }))
                .await,
        )
        .unwrap();
        assert_eq!(out.direction, Direction::Undo);
        out
    }

    async fn redo(&mut self) {
        self.ok(OperationKind::Redo, json!({ "confirmed": true }))
            .await;
    }

    async fn journal(&mut self) -> JournalState {
        serde_json::from_value(self.ok(OperationKind::JournalState, json!({})).await).unwrap()
    }

    fn local(&self) -> String {
        self.repo.git(&["rev-parse", "refs/tags/v1"])
    }

    fn remote(&self) -> String {
        self.repo.git_in_remote(&["rev-parse", "refs/tags/v1"])
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

fn take_args() -> Value {
    json!({ "remote": "origin", "name": "v1" })
}

fn push_args(confirmed: bool) -> Value {
    json!({ "remote": "origin", "name": "v1", "confirmed": confirmed })
}

#[tokio::test]
async fn using_the_remotes_tag_is_one_entry_that_undo_puts_back_even_collected() {
    for (name, backend) in backends() {
        for remote_annotated in [true, false] {
            let mut rig = Rig::differing(backend.clone(), remote_annotated).await;
            let (mine, theirs) = (rig.local(), rig.remote());
            let before = rig.journal().await;

            rig.ok(OperationKind::TagTakeRemote, take_args()).await;
            assert_eq!(rig.local(), theirs, "{name}: the remote's object");
            let kind = rig.repo.git(&["cat-file", "-t", "refs/tags/v1"]);
            assert_eq!(kind == "tag", remote_annotated, "{name}: {kind}");
            let state = rig.journal().await;
            let top = state.undo.as_ref().expect("an entry");
            assert_eq!(top.kind, OperationKind::TagTakeRemote, "{name}");
            assert_eq!(top.label, "Use tag v1 from origin", "{name}");
            assert_eq!(state.undo_count, before.undo_count + 1, "{name}: one entry");

            // The old object is unreachable now: only the journal's pin
            // keeps it. Collect everything collectable, then undo.
            rig.repo
                .git(&["reflog", "expire", "--expire-unreachable=now", "--all"]);
            rig.repo.git(&["gc", "--prune=now", "-q"]);
            rig.undo().await;
            assert_eq!(rig.local(), mine, "{name}: the old object is back");
            let kind = rig.repo.git(&["cat-file", "-t", "refs/tags/v1"]);
            assert_eq!(kind == "tag", !remote_annotated, "{name}: {kind}");
            rig.redo().await;
            assert_eq!(rig.local(), theirs, "{name}: redone");
            assert_eq!(rig.remote(), theirs, "{name}: the remote never moved");
            rig.stop().await;
        }
    }
}

#[tokio::test]
async fn a_force_push_asks_first_and_changes_nothing_unconfirmed() {
    for (name, backend) in backends() {
        let mut rig = Rig::differing(backend, false).await;
        let (theirs, count) = (rig.remote(), rig.journal().await.undo_count);
        let e = rig
            .call(OperationKind::TagForcePush, push_args(false))
            .await
            .unwrap_err();
        assert!(
            matches!(e.kind, Some(TypedError::NeedsConfirmation(_))),
            "{name}: {e:?}"
        );
        assert_eq!(rig.remote(), theirs, "{name}");
        assert_eq!(rig.journal().await.undo_count, count, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_force_push_is_one_entry_undone_with_a_lease_and_redone() {
    for (name, backend) in backends() {
        for remote_annotated in [false, true] {
            let mut rig = Rig::differing(backend.clone(), remote_annotated).await;
            let (mine, theirs) = (rig.local(), rig.remote());
            let count = rig.journal().await.undo_count;

            rig.ok(OperationKind::TagForcePush, push_args(true)).await;
            assert_eq!(rig.remote(), mine, "{name}: the remote has ours");
            assert_eq!(rig.local(), mine, "{name}: the local tag is untouched");
            let state = rig.journal().await;
            let top = state.undo.as_ref().expect("an entry");
            assert_eq!(top.kind, OperationKind::TagForcePush, "{name}");
            assert_eq!(top.label, "Force push tag v1 to origin", "{name}");
            assert_eq!(state.undo_count, count + 1, "{name}: one entry");

            let prompt = rig.undo_prompt().await;
            assert!(
                prompt.contains("moves tag v1 on origin"),
                "{name}: {prompt}"
            );
            rig.undo().await;
            assert_eq!(rig.remote(), theirs, "{name}: the old value is pushed back");
            assert_eq!(rig.local(), mine, "{name}");
            rig.redo().await;
            assert_eq!(rig.remote(), mine, "{name}: redone");
            rig.stop().await;
        }
    }
}

#[tokio::test]
async fn an_undo_is_refused_when_the_remote_tag_moved_again() {
    for (name, backend) in backends() {
        let mut rig = Rig::differing(backend, false).await;
        rig.ok(OperationKind::TagForcePush, push_args(true)).await;
        // Someone else moves the tag on the remote to another commit.
        rig.repo.push_from_elsewhere("main", "theirs");
        rig.repo.git_in_remote(&["tag", "-f", "v1", "main"]);
        let moved = rig.remote();
        let e = rig
            .call(OperationKind::Undo, json!({ "confirmed": true }))
            .await
            .unwrap_err();
        match e.kind {
            Some(TypedError::UndoRefused(UndoRefusal::RemoteRefMoved(m))) => {
                assert_eq!(m.actual, Oid::from_hex(&moved), "{name}")
            }
            other => panic!("{name}: expected RemoteRefMoved, got {other:?}"),
        }
        assert_eq!(rig.remote(), moved, "{name}: nothing was overwritten");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_stale_lease_is_refused_typed_records_nothing_and_overwrites_nothing() {
    for (name, backend) in backends() {
        let mut rig = Rig::differing(backend, false).await;
        rig.repo.push_from_elsewhere("main", "theirs");
        rig.repo.git_in_remote(&["tag", "-f", "v1", "main"]);
        let moved = rig.remote();
        let count = rig.journal().await.undo_count;

        let e = rig
            .call(OperationKind::TagForcePush, push_args(true))
            .await
            .unwrap_err();
        match e.kind {
            Some(TypedError::PushRejected(p)) => {
                assert_eq!(p.reason, PushRejectReason::StaleLease, "{name}")
            }
            other => panic!(
                "{name}: expected a stale lease, got {other:?} ({})",
                e.message
            ),
        }
        assert!(e.message.contains("fetch first"), "{name}: {}", e.message);
        assert_eq!(rig.remote(), moved, "{name}");
        assert_eq!(
            rig.journal().await.undo_count,
            count,
            "{name}: not recorded"
        );
        rig.stop().await;
    }
}
