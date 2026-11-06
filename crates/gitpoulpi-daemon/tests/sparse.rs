//! Sparse checkout through the daemon, and its undo story: the three
//! operations are *not journaled* (nothing is lost -- `SparseDisable` brings
//! every file back), so a sparse change never lands on the undo stack and
//! never makes a later Undo touch the working tree.

use std::sync::Arc;

use gitpoulpi_core::domain::{JournalListing, OperationKind, RepoId, SparseState};
use gitpoulpi_core::GitBackend;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request, ResponseError};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};

struct Rig {
    handle: Option<gitpoulpi_daemon::DaemonHandle>,
    client: Client,
    repo: FixtureRepo,
    id: RepoId,
    next: u64,
}

impl Rig {
    async fn new(backend: Arc<dyn GitBackend>) -> Rig {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("README.md", "root\n", "initial")
            .unwrap()
            .commit_file("src/a.rs", "a\n", "add src")
            .unwrap()
            .commit_file("docs/b.md", "b\n", "add docs")
            .unwrap()
            .build();
        let handle = Daemon::new("sparse").spawn(backend).await.unwrap();
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

    async fn call(&mut self, command: OperationKind, args: Value) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "sparse".into(),
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

    async fn sparse(&mut self) -> SparseState {
        serde_json::from_value(self.ok(OperationKind::SparseStatus, json!({})).await).unwrap()
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
async fn sparse_operations_work_and_are_not_on_the_undo_stack() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let root = rig.repo.root().to_path_buf();

        rig.ok(OperationKind::SparseSet, json!({ "dirs": ["src"] }))
            .await;
        assert!(root.join("src/a.rs").is_file(), "{name}");
        assert!(!root.join("docs").exists(), "{name}");
        let state = rig.sparse().await;
        assert!(state.enabled && state.cone, "{name}");
        assert_eq!(state.dirs, ["src"], "{name}");

        // The malformed request is refused and changes nothing.
        assert!(
            rig.call(OperationKind::SparseSet, json!({ "dirs": [] }))
                .await
                .is_err(),
            "{name}"
        );
        assert!(root.join("src/a.rs").is_file(), "{name}");

        rig.ok(OperationKind::SparseDisable, json!({})).await;
        assert!(root.join("docs/b.md").is_file(), "{name}");
        assert!(!rig.sparse().await.enabled, "{name}");

        // Nothing was journaled: the undo list is empty, and Undo refuses.
        let listing: JournalListing =
            serde_json::from_value(rig.ok(OperationKind::JournalList, json!({})).await).unwrap();
        assert!(listing.undo.is_empty(), "{name}: {:?}", listing.undo);
        assert!(
            rig.call(OperationKind::Undo, json!({})).await.is_err(),
            "{name}: nothing to undo"
        );
        rig.handle.take().unwrap().shutdown().await.unwrap();
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
