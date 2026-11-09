//! Undo/redo of deleting a branch or a tag on a remote, through the daemon.
//! The remote is a local bare repository (never a real server). Undo pushes
//! the very same object back under the same name (never forced), redo
//! deletes it again with a lease; a ref that moved on the remote meanwhile
//! refuses both, and a "delete here and on the remote" is one entry.
//! Every test runs against both backends.

use std::sync::Arc;

use gitpoulpi_core::domain::{
    Direction, EntrySummary, JournalState, Oid, OperationKind, RepoId, UndoOutcome, UndoRefusal,
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
    async fn on(backend: Arc<dyn GitBackend>, repo: FixtureRepo) -> Rig {
        std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
        let handle = Daemon::new("undo-remote-delete-test")
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

    /// A repository with `origin` (a bare local repository) holding `main`,
    /// the branch `topic` (one commit ahead of main) and the tags `light`
    /// (lightweight) and `v1` (annotated).
    async fn with_remote_refs(backend: Arc<dyn GitBackend>) -> Rig {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .with_origin()
            .unwrap()
            .push("main")
            .unwrap()
            .build();
        repo.git(&["branch", "topic"]);
        repo.git(&["tag", "light"]);
        repo.git(&["tag", "-a", "v1", "-m", "release one"]);
        repo.git(&["push", "origin", "topic", "refs/tags/light", "refs/tags/v1"]);
        Self::on(backend, repo).await
    }

    async fn call(&mut self, command: OperationKind, args: Value) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "undo-remote-delete-test".into(),
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

    /// Without `confirmed`, undoing a remote deletion asks first: the prompt
    /// is what a frontend shows.
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

    async fn redo(&mut self) -> UndoOutcome {
        serde_json::from_value(
            self.ok(OperationKind::Redo, json!({ "confirmed": true }))
                .await,
        )
        .unwrap()
    }

    /// The typed refusal `command` ends in.
    async fn refused(&mut self, command: OperationKind) -> UndoRefusal {
        let e = self
            .call(command, json!({ "confirmed": true }))
            .await
            .unwrap_err();
        match e.kind {
            Some(TypedError::UndoRefused(r)) => r,
            other => panic!("expected an undo refusal, got {other:?} ({})", e.message),
        }
    }

    async fn journal(&mut self) -> JournalState {
        serde_json::from_value(self.ok(OperationKind::JournalState, json!({})).await).unwrap()
    }

    fn rev(&self, spec: &str) -> Oid {
        Oid::from_hex(&self.repo.git(&["rev-parse", spec])).unwrap()
    }

    fn remote_rev(&self, spec: &str) -> String {
        self.repo.git_in_remote(&["rev-parse", spec])
    }

    fn remote_has(&self, full_ref: &str) -> bool {
        !self
            .repo
            .git_in_remote(&["for-each-ref", full_ref])
            .is_empty()
    }

    fn has_ref(&self, name: &str) -> bool {
        !self.repo.git(&["for-each-ref", name]).is_empty()
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

fn top(state: &JournalState) -> &EntrySummary {
    state.undo.as_ref().expect("an undo entry")
}

#[tokio::test]
async fn deleting_a_remote_branch_is_undone_to_the_same_commit_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::with_remote_refs(backend).await;
        let tip = rig.rev("topic");

        rig.ok(
            OperationKind::BranchDeleteRemote,
            json!({ "remote": "origin", "name": "topic" }),
        )
        .await;
        assert!(!rig.remote_has("refs/heads/topic"), "{name}");
        assert!(
            rig.has_ref("refs/heads/topic"),
            "{name}: the local branch stays"
        );
        let state = rig.journal().await;
        assert_eq!(
            top(&state).kind,
            OperationKind::BranchDeleteRemote,
            "{name}"
        );
        assert_eq!(top(&state).label, "Delete branch topic on origin", "{name}");

        // A network mutation: it asks first, and says what it does.
        let prompt = rig.undo_prompt().await;
        assert!(
            prompt.contains("re-creates branch topic on origin"),
            "{name}: {prompt}"
        );
        assert!(!rig.remote_has("refs/heads/topic"), "{name}: asked only");

        let out = rig.undo().await;
        assert!(
            out.steps
                .iter()
                .any(|s| s.contains("re-create topic on origin")),
            "{name}: {:?}",
            out.steps
        );
        assert_eq!(rig.remote_rev("refs/heads/topic"), tip.to_hex(), "{name}");
        assert_eq!(
            rig.rev("refs/remotes/origin/topic"),
            tip,
            "{name}: the tracking ref is back"
        );

        rig.redo().await;
        assert!(!rig.remote_has("refs/heads/topic"), "{name}: redone");
        // And once more round, to prove the entry survives a full cycle.
        rig.undo().await;
        assert_eq!(rig.remote_rev("refs/heads/topic"), tip.to_hex(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_remote_branch_recreated_meanwhile_refuses_undo_and_redo() {
    for (name, backend) in backends() {
        let mut rig = Rig::with_remote_refs(backend).await;
        let tip = rig.rev("topic");
        rig.ok(
            OperationKind::BranchDeleteRemote,
            json!({ "remote": "origin", "name": "topic" }),
        )
        .await;

        // Someone pushes a branch of that name again, elsewhere.
        rig.repo
            .git(&["commit", "--allow-empty", "-m", "elsewhere"]);
        let other = rig.rev("HEAD");
        rig.repo.git(&[
            "push",
            "origin",
            &format!("{}:refs/heads/topic", other.to_hex()),
        ]);

        match rig.refused(OperationKind::Undo).await {
            UndoRefusal::RemoteRefMoved(moved) => {
                assert_eq!(moved.remote, "origin", "{name}");
                assert_eq!(moved.name, "refs/heads/topic", "{name}");
                assert_eq!(moved.expected, None, "{name}");
                assert_eq!(moved.actual, Some(other), "{name}");
            }
            r => panic!("{name}: {r:?}"),
        }
        assert_eq!(
            rig.remote_rev("refs/heads/topic"),
            other.to_hex(),
            "{name}: the recreated branch is untouched"
        );
        assert_ne!(other, tip);

        // Delete it again outside, then undo works; redo then refuses when a
        // different value sits there.
        rig.repo.git(&["push", "origin", "--delete", "topic"]);
        rig.undo().await;
        assert_eq!(rig.remote_rev("refs/heads/topic"), tip.to_hex(), "{name}");
        rig.repo.git(&[
            "push",
            "--force",
            "origin",
            &format!("{}:refs/heads/topic", other.to_hex()),
        ]);
        match rig.refused(OperationKind::Redo).await {
            UndoRefusal::RemoteRefMoved(moved) => {
                assert_eq!(moved.expected, Some(tip), "{name}");
                assert_eq!(moved.actual, Some(other), "{name}");
            }
            r => panic!("{name}: {r:?}"),
        }
        assert_eq!(rig.remote_rev("refs/heads/topic"), other.to_hex(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn deleting_a_remote_tag_is_undone_with_its_annotation_and_redone() {
    for (name, backend) in backends() {
        for tag in ["light", "v1"] {
            let mut rig = Rig::with_remote_refs(backend.clone()).await;
            let object = rig.rev(&format!("refs/tags/{tag}"));
            rig.ok(
                OperationKind::TagDeleteRemote,
                json!({ "remote": "origin", "name": tag }),
            )
            .await;
            let refname = format!("refs/tags/{tag}");
            assert!(!rig.remote_has(&refname), "{name} {tag}");
            assert!(rig.has_ref(&refname), "{name} {tag}: the local tag stays");
            let state = rig.journal().await;
            assert_eq!(top(&state).kind, OperationKind::TagDeleteRemote);
            assert_eq!(top(&state).label, format!("Delete tag {tag} on origin"));

            let prompt = rig.undo_prompt().await;
            assert!(
                prompt.contains(&format!("re-creates tag {tag} on origin")),
                "{name}: {prompt}"
            );
            rig.undo().await;
            assert_eq!(
                rig.remote_rev(&refname),
                object.to_hex(),
                "{name} {tag}: the tag object itself is back"
            );
            if tag == "v1" {
                assert_eq!(
                    rig.repo.git_in_remote(&["cat-file", "-t", "refs/tags/v1"]),
                    "tag",
                    "{name}: still annotated"
                );
            }
            rig.redo().await;
            assert!(!rig.remote_has(&refname), "{name} {tag}: redone");
            rig.stop().await;
        }
    }
}

#[tokio::test]
async fn a_remote_tag_recreated_meanwhile_refuses_the_undo() {
    for (name, backend) in backends() {
        let mut rig = Rig::with_remote_refs(backend).await;
        rig.ok(
            OperationKind::TagDeleteRemote,
            json!({ "remote": "origin", "name": "v1" }),
        )
        .await;
        rig.repo
            .git(&["commit", "--allow-empty", "-m", "elsewhere"]);
        rig.repo.git(&["tag", "-f", "v1", "HEAD"]);
        let other = rig.rev("refs/tags/v1");
        rig.repo.git(&["push", "origin", "refs/tags/v1"]);

        assert!(
            matches!(
                rig.refused(OperationKind::Undo).await,
                UndoRefusal::RemoteRefMoved(m) if m.actual == Some(other)
            ),
            "{name}"
        );
        assert_eq!(rig.remote_rev("refs/tags/v1"), other.to_hex(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_collected_object_makes_the_undo_report_it_cannot_be_done() {
    for (name, backend) in backends() {
        let mut rig = Rig::with_remote_refs(backend).await;
        rig.ok(
            OperationKind::BranchDeleteRemote,
            json!({ "remote": "origin", "name": "topic" }),
        )
        .await;
        // The recorded value is a commit this repository does not have (the
        // pin was released and the object collected).
        let path = rig.repo.root().join(".git/gitpoulpi/journal.json");
        let text = std::fs::read_to_string(&path).unwrap();
        let mut journal: Value = serde_json::from_str(&text).unwrap();
        let entry = journal["undo"].as_array_mut().unwrap().last_mut().unwrap();
        entry["remote_refs"][0]["old"] = json!("ab".repeat(20));
        std::fs::write(&path, serde_json::to_string(&journal).unwrap()).unwrap();

        assert!(
            matches!(
                rig.refused(OperationKind::Undo).await,
                UndoRefusal::RemoteObjectMissing { .. }
            ),
            "{name}"
        );
        assert!(!rig.remote_has("refs/heads/topic"), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn deleting_a_branch_here_and_on_the_remote_is_one_entry_undone_together() {
    for (name, backend) in backends() {
        let mut rig = Rig::with_remote_refs(backend).await;
        let tip = rig.rev("topic");
        rig.ok(
            OperationKind::BranchDeleteRemote,
            json!({ "remote": "origin", "name": "topic", "and_local": true }),
        )
        .await;
        assert!(!rig.remote_has("refs/heads/topic"), "{name}");
        assert!(!rig.has_ref("refs/heads/topic"), "{name}");
        let state = rig.journal().await;
        assert_eq!(state.undo_count, 1, "{name}: one entry for both halves");
        assert!(top(&state).label.ends_with("and on origin"), "{name}");

        rig.undo().await;
        assert_eq!(rig.remote_rev("refs/heads/topic"), tip.to_hex(), "{name}");
        assert_eq!(rig.rev("refs/heads/topic"), tip, "{name}");

        rig.redo().await;
        assert!(!rig.remote_has("refs/heads/topic"), "{name}");
        assert!(!rig.has_ref("refs/heads/topic"), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn deleting_a_tag_here_and_on_the_remote_is_one_entry_undone_together() {
    for (name, backend) in backends() {
        let mut rig = Rig::with_remote_refs(backend).await;
        let object = rig.rev("refs/tags/v1");
        rig.ok(
            OperationKind::TagDeleteRemote,
            json!({ "remote": "origin", "name": "v1", "and_local": true }),
        )
        .await;
        assert!(!rig.remote_has("refs/tags/v1"), "{name}");
        assert!(!rig.has_ref("refs/tags/v1"), "{name}");
        assert_eq!(rig.journal().await.undo_count, 1, "{name}");

        rig.undo().await;
        assert_eq!(rig.rev("refs/tags/v1"), object, "{name}");
        assert_eq!(rig.remote_rev("refs/tags/v1"), object.to_hex(), "{name}");
        rig.redo().await;
        assert!(!rig.remote_has("refs/tags/v1"), "{name}");
        assert!(!rig.has_ref("refs/tags/v1"), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn deleting_a_ref_the_remote_does_not_have_records_nothing() {
    for (name, backend) in backends() {
        let mut rig = Rig::with_remote_refs(backend).await;
        // Whatever the plain deletion answers, nothing was deleted: no entry.
        let _ = rig
            .call(
                OperationKind::BranchDeleteRemote,
                json!({ "remote": "origin", "name": "nonexistent" }),
            )
            .await;
        assert!(rig.journal().await.undo.is_none(), "{name}");
        rig.stop().await;
    }
}
