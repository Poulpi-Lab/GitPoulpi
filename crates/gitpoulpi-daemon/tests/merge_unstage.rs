//! Staging and aborting during a merge, through the daemon:
//! * unstaging a resolved path leaves the other conflicts conflicted and is
//!   undoable (Undo is offered, and it puts the resolution back in the index);
//! * aborting the merge is undoable and brings back the whole stopped
//!   merge (MERGE_HEAD, the conflict stages, the markers in the files);
//!   redo aborts again.

use std::sync::Arc;

use gitpoulpi_core::domain::{ConflictEntry, FileStatus, JournalState, OperationKind, WorkingTree};
use gitpoulpi_core::GitBackend;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request};
use gitpoulpi_testkit::ConflictFixture;
use serde_json::{json, Value};

struct Ctx {
    client: Client,
    id: gitpoulpi_core::domain::RepoId,
    n: u64,
}

impl Ctx {
    async fn call(&mut self, command: OperationKind, args: Value) -> Value {
        self.n += 1;
        let request = Request {
            id: self.n,
            session: "merge-unstage-test".into(),
            repo: Some(self.id.clone()),
            command,
            args,
        };
        let response = self.client.call(&request).await.unwrap();
        response
            .result
            .unwrap_or_else(|e| panic!("{command:?}: {}", e.message))
    }

    async fn journal(&mut self) -> JournalState {
        serde_json::from_value(self.call(OperationKind::JournalState, json!({})).await).unwrap()
    }

    async fn entries(&mut self) -> Vec<ConflictEntry> {
        serde_json::from_value(self.call(OperationKind::ConflictEntries, json!({})).await).unwrap()
    }

    async fn status(&mut self) -> WorkingTree {
        serde_json::from_value(self.call(OperationKind::Status, json!({})).await).unwrap()
    }
}

fn backends() -> Vec<(&'static str, Arc<dyn GitBackend>)> {
    vec![
        (
            "system",
            Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new()),
        ),
        ("git2", Arc::new(gitpoulpi_git2_backend::Git2Backend::new())),
    ]
}

#[tokio::test]
async fn unstaging_a_resolved_path_keeps_the_others_conflicted_and_is_undoable() {
    for (name, backend) in backends() {
        let fx = ConflictFixture::multi_conflict();
        let handle = Daemon::new("merge-unstage-test")
            .spawn(backend)
            .await
            .unwrap();
        let mut c = Ctx {
            client: Client::connect_port(handle.port).await.unwrap(),
            id: fx.repo_ref().id,
            n: 0,
        };
        let paths = fx.paths().to_vec();
        let others: Vec<&String> = paths.iter().filter(|p| *p != "a.txt").collect();
        c.call(
            OperationKind::ResolveConflict,
            json!({"path": "a.txt", "resolution": {"UseContent": "done\n"}}),
        )
        .await;
        c.call(
            OperationKind::UnstageFile,
            json!({"target": {"File": "a.txt"}}),
        )
        .await;
        let tree = c.status().await;
        for p in &others {
            let e = tree
                .files
                .iter()
                .find(|e| e.path.to_str() == Some(p.as_str()))
                .unwrap();
            assert_eq!(
                e.worktree,
                Some(FileStatus::Conflicted),
                "{name} {p}: {e:?}"
            );
        }
        // Undo is offered, and is the unstage.
        let state = c.journal().await;
        let top = state.undo.expect("undo is offered");
        assert!(top.label.contains("Unstage"), "{name}: {}", top.label);
        assert!(state.blocked.is_none(), "{name}: {:?}", state.blocked);
        c.call(OperationKind::Undo, json!({})).await;
        let staged = fx.git(&["diff", "--cached", "--name-only"]);
        assert!(
            staged.contains("a.txt"),
            "{name}: the resolution is staged again"
        );
        assert_eq!(c.entries().await.len(), others.len(), "{name}");
        // And redo unstages again.
        c.call(OperationKind::Redo, json!({})).await;
        assert!(!fx
            .git(&["diff", "--cached", "--name-only"])
            .contains("a.txt"));
        assert_eq!(c.entries().await.len(), others.len(), "{name}");
        handle.shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn undoing_an_abort_brings_back_the_whole_merge() {
    for (name, backend) in backends() {
        let fx = ConflictFixture::multi_conflict();
        let handle = Daemon::new("merge-unstage-test")
            .spawn(backend)
            .await
            .unwrap();
        let mut c = Ctx {
            client: Client::connect_port(handle.port).await.unwrap(),
            id: fx.repo_ref().id,
            n: 0,
        };
        let paths = fx.paths().to_vec();
        let entries_before = c.entries().await;
        assert_eq!(entries_before.len(), paths.len());
        let disk: Vec<Vec<u8>> = paths.iter().map(|p| fx.worktree_bytes(p)).collect();
        let merge_head = fx.git(&["rev-parse", "MERGE_HEAD"]);
        // One resolved and staged, two still conflicted.
        c.call(
            OperationKind::ResolveConflict,
            json!({"path": "a.txt", "resolution": {"UseContent": "done\n"}}),
        )
        .await;

        c.call(OperationKind::OperationAbort, json!({})).await;
        assert!(!fx.root().join(".git/MERGE_HEAD").exists(), "{name}");
        let state = c.journal().await;
        let top = state.undo.expect("undo is offered after an abort");
        assert!(top.label.contains("Abort"), "{name}: {}", top.label);
        assert!(state.blocked.is_none(), "{name}: {:?}", state.blocked);

        c.call(OperationKind::Undo, json!({})).await;
        assert_eq!(
            fx.git(&["rev-parse", "MERGE_HEAD"]),
            merge_head,
            "{name}: MERGE_HEAD is back"
        );
        assert!(fx.root().join(".git/MERGE_MSG").exists(), "{name}");
        let after = c.entries().await;
        // The two unresolved paths are conflicted again with their stages.
        let others: Vec<&ConflictEntry> = entries_before
            .iter()
            .filter(|e| e.path.to_str() != Some("a.txt"))
            .collect();
        assert_eq!(after.len(), others.len(), "{name}: {after:?}");
        for (got, want) in after.iter().zip(&others) {
            assert_eq!(&got, want, "{name}");
        }
        for (p, bytes) in paths.iter().zip(&disk) {
            if p != "a.txt" {
                assert_eq!(&fx.worktree_bytes(p), bytes, "{name} {p}: markers back");
            }
        }
        // The resolved one is back, staged, with its resolution.
        assert_eq!(fx.worktree_bytes("a.txt"), b"done\n".to_vec(), "{name}");
        assert!(
            fx.git(&["diff", "--cached", "--name-only"])
                .contains("a.txt"),
            "{name}: a.txt staged"
        );
        let tree = c.status().await;
        for p in paths.iter().filter(|p| *p != "a.txt") {
            let e = tree
                .files
                .iter()
                .find(|e| e.path.to_str() == Some(p.as_str()))
                .unwrap();
            assert_eq!(e.worktree, Some(FileStatus::Conflicted), "{name} {p}");
        }
        // Redo aborts again.
        let redo = c.journal().await;
        assert!(redo.redo.is_some(), "{name}");
        c.call(OperationKind::Redo, json!({})).await;
        assert!(!fx.root().join(".git/MERGE_HEAD").exists(), "{name}");
        assert!(c.entries().await.is_empty(), "{name}");
        assert_eq!(fx.git(&["status", "--porcelain"]).trim(), "", "{name}");
        handle.shutdown().await.unwrap();
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
