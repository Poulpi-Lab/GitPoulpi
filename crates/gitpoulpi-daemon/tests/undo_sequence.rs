//! A sequence run's requests carry an `origin`; the daemon copies it into the
//! journal entry, and the state names the newest entries of one run (what
//! "Undo whole sequence" undoes). Untracked requests leave no trace of it.

use std::sync::Arc;

use gitpoulpi_core::domain::{
    CommitOptions, JournalListing, JournalState, OperationKind, RepoId, StageTarget,
};
use gitpoulpi_core::GitBackend;
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
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .build();
        std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
        std::env::set_var("GITPOULPI_GENERAL_UNDO_DEPTH", "0");
        let handle = Daemon::new("undo-sequence-test")
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

    async fn call(&mut self, command: OperationKind, args: Value) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "undo-sequence-test".into(),
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

    /// Stages and commits `name`, both marked as steps `step` and `step + 1`
    /// of run `run` (or unmarked with `run == 0`).
    async fn stage_and_commit(&mut self, name: &str, run: u64, step: u32) {
        std::fs::write(self.repo.root().join(name), name).unwrap();
        let mark = |step: u32| {
            if run == 0 {
                Value::Null
            } else {
                json!({ "name": "Backup", "run_id": run, "step": step, "of": 4 })
            }
        };
        let mut stage = json!({ "target": StageTarget::File(name.into()) });
        let mut commit = json!({ "message": name, "opts": CommitOptions::default() });
        if run != 0 {
            stage["origin"] = mark(step);
            commit["origin"] = mark(step + 1);
        }
        self.ok(OperationKind::StageFile, stage).await;
        self.ok(OperationKind::Commit, commit).await;
    }

    async fn state(&mut self) -> JournalState {
        serde_json::from_value(self.ok(OperationKind::JournalState, json!({})).await).unwrap()
    }

    async fn listing(&mut self) -> Value {
        self.ok(OperationKind::JournalList, json!({})).await
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn the_origin_of_a_request_reaches_its_journal_entry() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.stage_and_commit("a.txt", 77, 1).await;
        let listing: JournalListing = serde_json::from_value(rig.listing().await).unwrap();
        assert_eq!(listing.undo.len(), 2, "{name}");
        let stage = listing.undo[0].summary.origin.as_ref().expect("stage");
        let commit = listing.undo[1].summary.origin.as_ref().expect("commit");
        assert_eq!(
            (stage.name.as_str(), stage.run_id, stage.step),
            ("Backup", 77, 1),
            "{name}"
        );
        assert_eq!(
            (commit.run_id, commit.step, commit.of),
            (77, 2, 4),
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn an_untracked_request_leaves_no_origin_in_the_wire_format() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.stage_and_commit("a.txt", 0, 0).await;
        let listing = rig.listing().await;
        let text = listing.to_string();
        assert!(!text.contains("origin"), "{name}: {text}");
        assert!(rig.state().await.sequence.is_none(), "{name}");
        // The journal file is the old format too.
        let journal = std::fs::read_dir(rig.repo.root().join(".git/gitpoulpi"))
            .ok()
            .into_iter()
            .flatten()
            .flatten()
            .find(|e| e.file_name().to_string_lossy().ends_with("journal.json"))
            .map(|e| std::fs::read_to_string(e.path()).unwrap());
        if let Some(text) = journal {
            assert!(!text.contains("\"origin\""), "{name}");
        }
        rig.stop().await;
    }
}

#[tokio::test]
async fn the_state_names_the_newest_entries_of_one_run_and_stops_at_another() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        // An older run, an untracked commit, then the run that counts.
        rig.stage_and_commit("old.txt", 5, 1).await;
        rig.stage_and_commit("manual.txt", 0, 0).await;
        rig.stage_and_commit("new.txt", 6, 1).await;
        let group = rig.state().await.sequence.expect("a group");
        assert_eq!(
            (group.name.as_str(), group.run_id, group.of),
            ("Backup", 6, 4),
            "{name}"
        );
        let steps: Vec<u32> = group.entries.iter().map(|e| e.step).collect();
        assert_eq!(steps, vec![2, 1], "{name}: newest first");
        assert!(group.entries[0].label.starts_with("Commit"), "{name}");

        // One more untracked entry on top: the group is gone.
        rig.stage_and_commit("later.txt", 0, 0).await;
        assert!(rig.state().await.sequence.is_none(), "{name}");
        rig.stop().await;
    }
}
