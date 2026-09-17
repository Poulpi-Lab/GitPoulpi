//! Split commit through the daemon: one journal entry, undo puts the
//! original tip back, a refusal journals nothing. The worktree is never
//! touched, so a dirty one is fine.

use std::sync::Arc;

use gitpoulpi_core::domain::{
    Direction, Oid, OperationKind, SplitOutcome, SplitPart, SplitPiece, SplitRefusal, SplitSource,
    UndoOutcome, UndoRefusal,
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
    next: u64,
    /// The commit that adds `x.txt` and `y.txt`.
    target: Oid,
    /// The tip before any split: one commit on top of `target`.
    tip: Oid,
}

impl Rig {
    async fn new(backend: Arc<dyn GitBackend>) -> Rig {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .build();
        std::fs::write(repo.root().join("x.txt"), "x\n").unwrap();
        std::fs::write(repo.root().join("y.txt"), "y\n").unwrap();
        repo.git(&["add", "x.txt", "y.txt"]);
        repo.git(&["commit", "-q", "-m", "add x and y"]);
        let target = Oid::from_hex(&repo.git(&["rev-parse", "HEAD"])).unwrap();
        std::fs::write(repo.root().join("z.txt"), "z\n").unwrap();
        repo.git(&["add", "z.txt"]);
        repo.git(&["commit", "-q", "-m", "add z"]);
        let tip = Oid::from_hex(&repo.git(&["rev-parse", "HEAD"])).unwrap();
        // Outside changes are not recorded here: only the split's own entry.
        std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
        let handle = Daemon::new("split-undo-test").spawn(backend).await.unwrap();
        let client = Client::connect_port(handle.port).await.unwrap();
        Rig {
            handle: Some(handle),
            client,
            repo,
            next: 1,
            target,
            tip,
        }
    }

    async fn call(&mut self, command: OperationKind, args: Value) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "split-undo-test".into(),
                repo: Some(self.repo.repo_ref().id),
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

    fn rev(&self, spec: &str) -> Oid {
        Oid::from_hex(&self.repo.git(&["rev-parse", spec])).unwrap()
    }

    fn request(&self, parts: Vec<SplitPart>) -> Value {
        json!({ "req": { "commit": self.target, "parts": parts, "sign": false } })
    }

    fn two_parts(&self) -> Vec<SplitPart> {
        let part = |message: &str, path: &str| SplitPart {
            message: message.into(),
            pieces: vec![SplitPiece::File { path: path.into() }],
        };
        vec![part("add x", "x.txt"), part("add y", "y.txt")]
    }

    /// The typed refusal `Undo` ends in.
    async fn undo_refusal(&mut self) -> UndoRefusal {
        let e = self.call(OperationKind::Undo, json!({})).await.unwrap_err();
        match e.kind {
            Some(TypedError::UndoRefused(r)) => r,
            other => panic!("expected an undo refusal, got {other:?} ({})", e.message),
        }
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn split_source_is_a_plain_read() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let answer = rig
            .ok(OperationKind::SplitSource, json!({ "commit": rig.target }))
            .await;
        let source: SplitSource = serde_json::from_value(answer).unwrap();
        assert_eq!(source.commit, rig.target, "{name}");
        assert_eq!(source.files.len(), 2, "{name}");
        assert_eq!(
            rig.undo_refusal().await,
            UndoRefusal::NothingToUndo,
            "{name}: a read records no entry"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn split_is_one_entry_undo_restores_the_tip_and_redo_the_same_new_one() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        // A dirty worktree survives both ways.
        std::fs::write(rig.repo.root().join("dirty.txt"), "d").unwrap();
        std::fs::write(rig.repo.root().join("z.txt"), "z edited\n").unwrap();
        let status = rig.repo.git(&["status", "--porcelain"]);

        let args = rig.request(rig.two_parts());
        let answer = rig.ok(OperationKind::SplitCommit, args).await;
        let outcome: SplitOutcome = serde_json::from_value(answer).unwrap();
        assert_eq!(outcome.created.len(), 2, "{name}");
        assert_eq!(outcome.previous_head, rig.tip, "{name}");
        assert_eq!(outcome.replayed, 1, "{name}");
        let split_tip = rig.rev("HEAD");
        assert_eq!(split_tip, outcome.new_head, "{name}");
        assert_eq!(rig.repo.git(&["status", "--porcelain"]), status, "{name}");
        assert_eq!(
            rig.repo.git(&["rev-list", "--count", "HEAD"]),
            "4",
            "{name}: initial, add x, add y, add z"
        );

        let out: UndoOutcome =
            serde_json::from_value(rig.ok(OperationKind::Undo, json!({})).await).unwrap();
        assert_eq!(out.direction, Direction::Undo, "{name}");
        assert!(out.entry.label.starts_with("Split "), "{name}");
        assert!(out.entry.label.contains("into 2 commits"), "{name}");
        assert_eq!(rig.rev("HEAD"), rig.tip, "{name}: the old tip, same oid");
        assert_eq!(rig.rev("main"), rig.tip, "{name}");
        assert_eq!(rig.rev("HEAD~1"), rig.target, "{name}: the original commit");
        assert_eq!(rig.repo.git(&["status", "--porcelain"]), status, "{name}");

        rig.ok(OperationKind::Redo, json!({})).await;
        assert_eq!(rig.rev("HEAD"), split_tip, "{name}: redo never re-splits");

        // A commit on top since: the entry requires HEAD, so undo refuses.
        rig.repo
            .git(&["commit", "-q", "--allow-empty", "-m", "outside"]);
        let refusal = rig.undo_refusal().await;
        assert!(
            matches!(
                refusal,
                UndoRefusal::HeadMoved { .. } | UndoRefusal::RefMoved { .. }
            ),
            "{name}: {refusal:?}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_refused_split_changes_and_journals_nothing() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let mut parts = rig.two_parts();
        parts[1].pieces.clear();
        let args = rig.request(parts);
        let e = rig
            .call(OperationKind::SplitCommit, args)
            .await
            .unwrap_err();
        assert_eq!(
            e.kind,
            Some(TypedError::SplitRefused(SplitRefusal::EmptyPart)),
            "{name}: {}",
            e.message
        );
        assert_eq!(rig.rev("HEAD"), rig.tip, "{name}");
        assert_eq!(
            rig.undo_refusal().await,
            UndoRefusal::NothingToUndo,
            "{name}: nothing was journaled"
        );
        rig.stop().await;
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
