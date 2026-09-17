//! `general.undo_enabled` and the journal's caching, through the daemon.
//! Its own test binary: the setting is driven through the process
//! environment, so everything here runs in sequence inside one test.

use std::sync::Arc;

use gitpoulpi_cli_backend::total_spawn_count;
use gitpoulpi_core::domain::{
    JournalListing, JournalState, OperationKind, RepoId, StageTarget, UndoRefusal,
};
use gitpoulpi_core::{GitBackend, TypedError};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request, ResponseError};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder, SpawnBudget};
use serde_json::{json, Value};

const ENV: &str = "GITPOULPI_GENERAL_UNDO_ENABLED";

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
            .commit_file("a.txt", "one\n", "initial")
            .unwrap()
            .build();
        let handle = Daemon::new("undo-off").spawn(backend).await.unwrap();
        let client = Client::connect_port(handle.port).await.unwrap();
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
                session: "undo-off".into(),
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

    async fn state(&mut self) -> JournalState {
        serde_json::from_value(self.ok(OperationKind::JournalState, json!({})).await).unwrap()
    }

    async fn list(&mut self) -> JournalListing {
        serde_json::from_value(self.ok(OperationKind::JournalList, json!({})).await).unwrap()
    }

    async fn stage(&mut self, name: &str) {
        std::fs::write(self.repo.root().join(name), name).unwrap();
        self.ok(
            OperationKind::StageFile,
            json!({ "target": StageTarget::File(name.into()) }),
        )
        .await;
    }

    fn journal_file(&self) -> std::path::PathBuf {
        self.repo.root().join(".git/gitpoulpi/journal.json")
    }

    fn pins(&self) -> String {
        self.repo.git(&["for-each-ref", "refs/gitpoulpi/undo/"])
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
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
async fn undo_off_does_no_undo_work_and_erases_the_history() {
    let sb = SpawnBudget::serial(total_spawn_count);
    for (name, backend) in backends() {
        let system = name == "system";
        std::env::remove_var(ENV);
        let mut rig = Rig::new(backend).await;

        // On: staging is recorded, with its pins.
        rig.stage("on.txt").await;
        gitpoulpi_daemon::flush_undo_journal();
        assert!(rig.journal_file().exists(), "{name}");
        assert!(!rig.pins().is_empty(), "{name}: staging pins its trees");
        assert!(rig.list().await.undo.len() == 1, "{name}");
        // A discard leaves a snapshot behind (write-ahead, synchronous).
        std::fs::write(rig.repo.root().join("a.txt"), "changed\n").unwrap();
        rig.ok(OperationKind::Discard, json!({ "paths": ["a.txt"] }))
            .await;
        assert_eq!(rig.list().await.undo.len(), 2, "{name}");

        // Off: the next request erases the history, everywhere it knows.
        std::env::set_var(ENV, "false");
        let state = rig.state().await;
        assert!(state.disabled && state.undo.is_none(), "{name}");
        assert!(!rig.journal_file().exists(), "{name}: journal deleted");
        assert!(rig.pins().is_empty(), "{name}: pins released");
        assert!(rig.list().await.undo.is_empty(), "{name}");

        // Off: undo and redo refuse in words, and nothing is recorded or
        // paid for: the pre-undo costs (git add only on the system backend).
        for command in [OperationKind::Undo, OperationKind::Redo] {
            let err = rig.call(command, json!({})).await.unwrap_err();
            assert!(
                matches!(
                    err.kind,
                    Some(TypedError::UndoRefused(UndoRefusal::Disabled))
                ),
                "{name}: {err:?}"
            );
            assert_eq!(err.message, "Undo is turned off (Settings > General)");
        }
        let before = total_spawn_count();
        rig.stage("off.txt").await;
        let spawned = total_spawn_count() - before;
        assert_eq!(spawned, u64::from(system), "{name}: staging spawns");
        std::fs::write(rig.repo.root().join("a.txt"), "again\n").unwrap();
        rig.ok(OperationKind::Discard, json!({ "paths": ["a.txt"] }))
            .await;
        gitpoulpi_daemon::flush_undo_journal();
        assert!(!rig.journal_file().exists(), "{name}: no journal I/O");
        assert!(rig.pins().is_empty(), "{name}: no pins");
        assert!(!rig.repo.root().join(".git/gitpoulpi").exists(), "{name}");

        // Back on: recording starts again from an empty history.
        std::env::remove_var(ENV);
        rig.stage("again.txt").await;
        assert_eq!(rig.list().await.undo.len(), 1, "{name}");
        assert!(!rig.state().await.disabled, "{name}");
        rig.stop().await;
    }
    std::env::remove_var(ENV);
    drop(sb);
}

/// The journal is cached in memory, but the file stays the truth: an edit
/// by another process shows on the next read, and a crash between a
/// staging operation and its journal write leaves a usable state.
#[tokio::test]
async fn the_journal_cache_follows_the_file_and_survives_a_lost_write() {
    let sb = SpawnBudget::serial(total_spawn_count);
    std::env::remove_var(ENV);
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.stage("one.txt").await;
        rig.stage("two.txt").await;
        gitpoulpi_daemon::flush_undo_journal();
        assert_eq!(rig.list().await.undo.len(), 2, "{name}");

        // Another process rewrites the journal (a shorter one).
        let text = std::fs::read_to_string(rig.journal_file()).unwrap();
        let mut journal: Value = serde_json::from_str(&text).unwrap();
        journal["undo"].as_array_mut().unwrap().remove(1);
        std::fs::write(rig.journal_file(), serde_json::to_vec(&journal).unwrap()).unwrap();
        assert_eq!(
            rig.list().await.undo.len(),
            1,
            "{name}: the external edit is noticed"
        );

        // A crash after a staging entry's pins but before its journal
        // write: the file never learned the entry, the pins stayed. The
        // id is handed out again and the new entry still records.
        let next = journal["next_id"].as_u64().unwrap_or(0);
        let head = rig
            .repo
            .git(&["rev-parse", "HEAD^{tree}"])
            .trim()
            .to_string();
        rig.repo.git(&[
            "update-ref",
            &format!("refs/gitpoulpi/undo/{next}/0"),
            &head,
        ]);
        rig.stage("three.txt").await;
        gitpoulpi_daemon::flush_undo_journal();
        assert_eq!(
            rig.list().await.undo.len(),
            2,
            "{name}: recorded despite the stale pin"
        );
        rig.stop().await;
    }
    drop(sb);
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
