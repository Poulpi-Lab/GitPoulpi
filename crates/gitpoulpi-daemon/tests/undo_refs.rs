//! Undo/redo of the ref-only operations through the daemon
//! (the undo-redo design): branch rename, set or
//! clear the upstream, tracking branch creation, tag create / delete /
//! annotate, fast-forward, reword and fetch-ref-as-branch. Every test
//! runs against both backends, and each operation is covered for undo,
//! redo and the refusal after an outside change.

use std::sync::Arc;

use gitpoulpi_core::domain::{
    CommitOptions, Direction, FastForwardOutcome, Oid, OperationKind, RepoId, StageTarget,
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
        Self::on(backend, repo).await
    }

    async fn on(backend: Arc<dyn GitBackend>, repo: FixtureRepo) -> Rig {
        // These tests are about a blocked entry: outside changes are not
        // recorded here (outside_journal.rs covers them recorded).
        std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
        let handle = Daemon::new("undo-refs-test").spawn(backend).await.unwrap();
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
                session: "undo-refs-test".into(),
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

    async fn undo(&mut self) -> UndoOutcome {
        let out: UndoOutcome =
            serde_json::from_value(self.ok(OperationKind::Undo, json!({})).await).unwrap();
        assert_eq!(out.direction, Direction::Undo);
        out
    }

    async fn redo(&mut self) -> UndoOutcome {
        serde_json::from_value(self.ok(OperationKind::Redo, json!({})).await).unwrap()
    }

    /// The typed refusal `command` ends in; nothing may have changed.
    async fn refused(&mut self, command: OperationKind) -> UndoRefusal {
        let e = self.call(command, json!({})).await.unwrap_err();
        match e.kind {
            Some(TypedError::UndoRefused(r)) => r,
            other => panic!("expected an undo refusal, got {other:?} ({})", e.message),
        }
    }

    fn rev(&self, spec: &str) -> Oid {
        Oid::from_hex(&self.repo.git(&["rev-parse", spec])).unwrap()
    }

    fn has_ref(&self, name: &str) -> bool {
        !self.repo.git(&["for-each-ref", name]).is_empty()
    }

    fn has_config(&self, key: &str) -> bool {
        self.repo
            .git(&["config", "--local", "--list"])
            .lines()
            .any(|l| l.starts_with(&format!("{key}=")))
    }

    fn config(&self, key: &str) -> String {
        self.repo.git(&["config", "--local", "--get", key])
    }

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

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

// ------------------------------------------------------------- rename

#[tokio::test]
async fn branch_rename_undo_and_redo_carry_the_upstream_and_head() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let tip = rig.rev("HEAD");
        rig.repo.git(&["branch", "feature"]);
        rig.repo.git(&["config", "branch.feature.remote", "origin"]);
        rig.repo
            .git(&["config", "branch.feature.merge", "refs/heads/feature"]);

        // A branch that is not checked out.
        rig.ok(
            OperationKind::BranchRename,
            json!({ "old_name": "feature", "new_name": "renamed" }),
        )
        .await;
        assert!(!rig.has_ref("refs/heads/feature"), "{name}");
        assert_eq!(rig.rev("renamed"), tip, "{name}");
        assert_eq!(rig.config("branch.renamed.remote"), "origin", "{name}");

        let out = rig.undo().await;
        assert!(
            out.entry.label.starts_with("Rename branch feature"),
            "{name}"
        );
        assert_eq!(rig.rev("feature"), tip, "{name}");
        assert!(!rig.has_ref("refs/heads/renamed"), "{name}");
        assert_eq!(rig.config("branch.feature.remote"), "origin", "{name}");
        assert_eq!(
            rig.config("branch.feature.merge"),
            "refs/heads/feature",
            "{name}"
        );
        assert!(!rig.has_config("branch.renamed.remote"), "{name}");

        rig.redo().await;
        assert!(!rig.has_ref("refs/heads/feature"), "{name}");
        assert_eq!(rig.rev("renamed"), tip, "{name}");
        assert_eq!(rig.config("branch.renamed.merge"), "refs/heads/feature");
        rig.undo().await;

        // The checked-out branch: HEAD follows it both ways.
        rig.repo.git(&["checkout", "-q", "feature"]);
        rig.ok(
            OperationKind::BranchRename,
            json!({ "old_name": "feature", "new_name": "moved" }),
        )
        .await;
        assert_eq!(rig.repo.git(&["symbolic-ref", "HEAD"]), "refs/heads/moved");
        rig.undo().await;
        assert_eq!(
            rig.repo.git(&["symbolic-ref", "HEAD"]),
            "refs/heads/feature",
            "{name}: HEAD goes back with the name"
        );
        assert_eq!(rig.rev("feature"), tip, "{name}");
        assert!(!rig.has_ref("refs/heads/moved"), "{name}");
        rig.redo().await;
        assert_eq!(rig.repo.git(&["symbolic-ref", "HEAD"]), "refs/heads/moved");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_branch_rename_is_refused_after_an_outside_change() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let tip = rig.rev("HEAD");
        rig.repo.git(&["branch", "feature"]);
        rig.ok(
            OperationKind::BranchRename,
            json!({ "old_name": "feature", "new_name": "renamed" }),
        )
        .await;
        // Renamed again outside: the new name is gone.
        rig.repo.git(&["branch", "-m", "renamed", "elsewhere"]);
        let refusal = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(&refusal, UndoRefusal::RefMoved { name, .. } if name == "refs/heads/renamed"),
            "{name}: {refusal:?}"
        );
        assert!(!rig.has_ref("refs/heads/feature"), "{name}: unchanged");
        // The old name taken by someone else also refuses.
        rig.repo.git(&["branch", "-m", "elsewhere", "renamed"]);
        rig.repo.git(&["branch", "feature", "HEAD"]);
        let refusal = rig.refused(OperationKind::Undo).await;
        assert!(matches!(refusal, UndoRefusal::RefMoved { .. }), "{name}");
        rig.repo.git(&["branch", "-D", "feature"]);
        rig.undo().await;
        assert_eq!(rig.rev("feature"), tip, "{name}");
        rig.stop().await;
    }
}

// ----------------------------------------------------------- upstream

fn with_origin_repo() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .branch("work")
        .unwrap()
        .checkout("work")
        .unwrap()
        .commit_file("work.txt", "work", "work commit")
        .unwrap()
        .push("work")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    // The fixture may or may not have set an upstream; start without.
    let _ = std::process::Command::new("git")
        .args(["branch", "--unset-upstream", "main"])
        .current_dir(repo.root())
        .output();
    repo
}

#[tokio::test]
async fn set_and_unset_upstream_round_trip_and_refuse_outside_config_changes() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, with_origin_repo()).await;
        assert!(!rig.has_config("branch.main.remote"), "{name}");

        rig.ok(
            OperationKind::BranchSetUpstream,
            json!({ "branch": "main", "upstream": "origin/main" }),
        )
        .await;
        assert_eq!(rig.config("branch.main.remote"), "origin", "{name}");
        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::BranchSetUpstream);
        assert!(!rig.has_config("branch.main.remote"), "{name}");
        assert!(!rig.has_config("branch.main.merge"), "{name}");
        rig.redo().await;
        assert_eq!(rig.config("branch.main.merge"), "refs/heads/main", "{name}");

        // Changing it to another remote branch remembers the old one.
        rig.ok(
            OperationKind::BranchSetUpstream,
            json!({ "branch": "main", "upstream": "origin/work" }),
        )
        .await;
        assert_eq!(rig.config("branch.main.merge"), "refs/heads/work", "{name}");
        rig.undo().await;
        assert_eq!(rig.config("branch.main.merge"), "refs/heads/main", "{name}");
        rig.redo().await;
        rig.undo().await;

        // Unset, then undo puts it back.
        rig.ok(
            OperationKind::BranchSetUpstream,
            json!({ "branch": "main", "upstream": null }),
        )
        .await;
        assert!(!rig.has_config("branch.main.remote"), "{name}");
        rig.undo().await;
        assert_eq!(rig.config("branch.main.remote"), "origin", "{name}");
        rig.redo().await;

        // Edited outside: the config section is no longer as the entry
        // left it.
        rig.undo().await;
        rig.redo().await;
        rig.repo
            .git(&["config", "branch.main.merge", "refs/heads/zzz"]);
        let refusal = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(&refusal, UndoRefusal::ConfigChanged { section } if section == "branch.main"),
            "{name}: {refusal:?}"
        );
        assert_eq!(rig.config("branch.main.merge"), "refs/heads/zzz", "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn setting_an_upstream_that_changes_nothing_records_no_entry() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, with_origin_repo()).await;
        // Unsetting an upstream that is not set succeeds and is a no-op.
        rig.ok(
            OperationKind::BranchSetUpstream,
            json!({ "branch": "main", "upstream": null }),
        )
        .await;
        assert_eq!(
            rig.refused(OperationKind::Undo).await,
            UndoRefusal::NothingToUndo,
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn setting_core_autocrlf_and_eol_is_undoable_and_redoable() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, with_origin_repo()).await;
        rig.repo
            .git(&["config", "--local", "--unset-all", "core.autocrlf"]);
        if rig.has_config("core.eol") {
            rig.repo
                .git(&["config", "--local", "--unset-all", "core.eol"]);
        }
        assert!(!rig.has_config("core.autocrlf"), "{name}");

        rig.ok(
            OperationKind::SetRepoConfig,
            json!({ "key": "core.autocrlf", "value": "input" }),
        )
        .await;
        assert_eq!(rig.config("core.autocrlf"), "input", "{name}");
        // Replace it: undo goes back to the previous value, not to unset.
        rig.ok(
            OperationKind::SetRepoConfig,
            json!({ "key": "core.autocrlf", "value": "true" }),
        )
        .await;
        assert_eq!(rig.config("core.autocrlf"), "true", "{name}");
        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::SetRepoConfig, "{name}");
        assert_eq!(rig.config("core.autocrlf"), "input", "{name}");
        rig.undo().await;
        assert!(!rig.has_config("core.autocrlf"), "{name}");
        rig.redo().await;
        rig.redo().await;
        assert_eq!(rig.config("core.autocrlf"), "true", "{name}");

        // Unsetting is recorded too, and a no-op unset is not.
        rig.ok(
            OperationKind::SetRepoConfig,
            json!({ "key": "core.autocrlf", "value": null }),
        )
        .await;
        assert!(!rig.has_config("core.autocrlf"), "{name}");
        rig.undo().await;
        assert_eq!(rig.config("core.autocrlf"), "true", "{name}");
        rig.redo().await;
        rig.ok(
            OperationKind::SetRepoConfig,
            json!({ "key": "core.eol", "value": null }),
        )
        .await;
        assert!(!rig.has_config("core.eol"), "{name}");
        rig.undo().await; // undoes the autocrlf unset, not the no-op
        assert_eq!(rig.config("core.autocrlf"), "true", "{name}");

        // Edited outside: refused, as for any config section.
        rig.redo().await;
        rig.undo().await;
        rig.repo.git(&["config", "core.autocrlf", "false"]);
        let refusal = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(&refusal, UndoRefusal::ConfigChanged { section } if section == "core.autocrlf"),
            "{name}: {refusal:?}"
        );
        // A key outside the allow-list is refused by the backend.
        assert!(rig
            .call(
                OperationKind::SetRepoConfig,
                json!({ "key": "core.sshCommand", "value": "x" }),
            )
            .await
            .is_err());
        rig.stop().await;
    }
}

// ------------------------------------------------- tracking creation

#[tokio::test]
async fn creating_a_tracking_branch_is_undone_by_going_back_and_redone_by_recreating() {
    for (name, backend) in backends() {
        let repo = with_origin_repo();
        // The remote branch is known, the local one is not.
        repo.git(&["branch", "-D", "work"]);
        let mut rig = Rig::on(backend, repo).await;
        let main_tip = rig.rev("main");
        let work_tip = rig.rev("refs/remotes/origin/work");

        rig.ok(
            OperationKind::BranchCreateTracking,
            json!({ "local_name": "work", "upstream": "origin/work" }),
        )
        .await;
        assert_eq!(rig.repo.git(&["symbolic-ref", "HEAD"]), "refs/heads/work");
        assert!(rig.repo.root().join("work.txt").exists(), "{name}");

        let out = rig.undo().await;
        assert!(out.entry.label.contains("tracking origin/work"), "{name}");
        assert_eq!(rig.repo.git(&["symbolic-ref", "HEAD"]), "refs/heads/main");
        assert!(!rig.has_ref("refs/heads/work"), "{name}");
        assert!(!rig.has_config("branch.work.remote"), "{name}");
        assert!(!rig.repo.root().join("work.txt").exists(), "{name}");
        assert_eq!(rig.rev("main"), main_tip, "{name}");

        rig.redo().await;
        assert_eq!(rig.repo.git(&["symbolic-ref", "HEAD"]), "refs/heads/work");
        assert_eq!(rig.rev("work"), work_tip, "{name}");
        assert_eq!(rig.config("branch.work.remote"), "origin", "{name}");
        assert!(rig.repo.root().join("work.txt").exists(), "{name}");

        // HEAD moved away outside: undo refuses (the entry requires HEAD).
        rig.repo.git(&["checkout", "-q", "main"]);
        let refusal = rig.refused(OperationKind::Undo).await;
        assert!(matches!(refusal, UndoRefusal::HeadMoved { .. }), "{name}");
        rig.stop().await;
    }
}

// --------------------------------------------------------------- tags

#[tokio::test]
async fn tag_create_undo_deletes_and_redo_restores_the_same_object() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let tip = rig.rev("HEAD");

        rig.ok(
            OperationKind::TagCreate,
            json!({ "name": "light", "at": tip, "annotated_message": null }),
        )
        .await;
        assert_eq!(rig.rev("refs/tags/light"), tip, "{name}");
        rig.undo().await;
        assert!(!rig.has_ref("refs/tags/light"), "{name}");
        rig.redo().await;
        assert_eq!(rig.rev("refs/tags/light"), tip, "{name}");

        rig.ok(
            OperationKind::TagCreate,
            json!({ "name": "v1", "at": tip, "annotated_message": "release one" }),
        )
        .await;
        let object = rig.rev("refs/tags/v1");
        assert_ne!(object, tip, "{name}: annotated, so its own object");
        rig.undo().await;
        assert!(!rig.has_ref("refs/tags/v1"), "{name}");
        // Gone from the repository for good unless something keeps it.
        rig.repo.git(&["gc", "-q", "--prune=now"]);
        rig.redo().await;
        assert_eq!(
            rig.rev("refs/tags/v1"),
            object,
            "{name}: redo restores the same tag object, it does not re-tag"
        );
        assert_eq!(rig.repo.git(&["cat-file", "-t", "refs/tags/v1"]), "tag");

        // Moved outside: refuse.
        rig.repo.git(&["tag", "-f", "v1", "HEAD"]);
        let refusal = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(&refusal, UndoRefusal::RefMoved { name, .. } if name == "refs/tags/v1"),
            "{name}: {refusal:?}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn deleting_an_annotated_tag_is_undone_with_the_very_same_tag_object() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .annotated_tag("v1")
            .unwrap()
            .tag("light")
            .unwrap()
            .build();
        let mut rig = Rig::on(backend, repo).await;
        let object = rig.rev("refs/tags/v1");
        let light = rig.rev("refs/tags/light");

        rig.ok(OperationKind::TagDelete, json!({ "name": "v1" }))
            .await;
        assert!(!rig.has_ref("refs/tags/v1"), "{name}");
        // The pin keeps the tag object through a prune.
        rig.repo.git(&["gc", "-q", "--prune=now"]);
        rig.undo().await;
        assert_eq!(rig.rev("refs/tags/v1"), object, "{name}: same object");
        assert_eq!(rig.repo.git(&["cat-file", "-t", "refs/tags/v1"]), "tag");
        rig.redo().await;
        assert!(!rig.has_ref("refs/tags/v1"), "{name}");
        rig.undo().await;

        rig.ok(OperationKind::TagDelete, json!({ "name": "light" }))
            .await;
        rig.undo().await;
        assert_eq!(rig.rev("refs/tags/light"), light, "{name}");
        assert_eq!(
            rig.repo.git(&["cat-file", "-t", "refs/tags/light"]),
            "commit",
            "{name}: a lightweight tag comes back lightweight"
        );

        // Recreated outside under the same name: refuse the redo.
        rig.redo().await;
        rig.repo.git(&["tag", "light"]);
        let refusal = rig.refused(OperationKind::Undo).await;
        assert!(matches!(refusal, UndoRefusal::RefMoved { .. }), "{name}");
        rig.stop().await;
    }
}

/// Two separate requests, a remote delete then a local one (the combined
/// gesture sends one request with `and_local`: undo_remote_delete.rs). Each
/// half is its own entry: undoing the local one restores the very same local
/// tag object, which is then *local only* again (the remote entry underneath
/// is what re-creates it there), and a tag push puts it back on the remote.
#[tokio::test]
async fn undoing_a_combined_tag_delete_restores_it_locally_and_a_push_puts_it_back() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .with_origin()
            .unwrap()
            .annotated_tag("v1")
            .unwrap()
            .push("main")
            .unwrap()
            .push("refs/tags/v1")
            .unwrap()
            .build();
        let mut rig = Rig::on(backend, repo).await;
        let object = rig.rev("refs/tags/v1");

        // What the TUI and `tag delete --both` do: remote first, then
        // the journaled local delete.
        rig.ok(
            OperationKind::TagDeleteRemote,
            json!({ "remote": "origin", "name": "v1" }),
        )
        .await;
        rig.ok(OperationKind::TagDelete, json!({ "name": "v1" }))
            .await;
        assert!(!rig.has_ref("refs/tags/v1"), "{name}");
        assert_eq!(rig.repo.git_in_remote(&["tag", "--list"]), "", "{name}");

        let undone = rig.undo().await;
        assert_eq!(
            undone.entry.kind,
            OperationKind::TagDelete,
            "{name}: the local delete is the top entry"
        );
        assert_eq!(rig.rev("refs/tags/v1"), object, "{name}: same tag object");
        assert_eq!(
            rig.repo.git_in_remote(&["tag", "--list"]),
            "",
            "{name}: undo never touches the remote"
        );
        let refs: Vec<gitpoulpi_core::domain::Ref> =
            serde_json::from_value(rig.ok(OperationKind::Refs, json!({})).await).unwrap();
        let tag = refs.iter().find(|r| r.tag_name() == Some("v1")).unwrap();
        assert!(tag.is_unpushed_tag(), "{name}: local only again");

        rig.ok(
            OperationKind::TagPush,
            json!({ "remote": "origin", "selection": { "Named": ["v1"] } }),
        )
        .await;
        assert_eq!(rig.repo.git_in_remote(&["tag", "--list"]), "v1", "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn annotating_a_tag_is_undone_to_the_lightweight_tag_and_redone_to_the_same_object() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .tag("v1")
            .unwrap()
            .build();
        let mut rig = Rig::on(backend, repo).await;
        let tip = rig.rev("HEAD");

        rig.ok(
            OperationKind::TagAnnotate,
            json!({ "name": "v1", "message": "now annotated" }),
        )
        .await;
        let object = rig.rev("refs/tags/v1");
        assert_ne!(object, tip, "{name}");

        rig.undo().await;
        assert_eq!(rig.rev("refs/tags/v1"), tip, "{name}");
        assert_eq!(rig.repo.git(&["cat-file", "-t", "refs/tags/v1"]), "commit");
        rig.redo().await;
        assert_eq!(rig.rev("refs/tags/v1"), object, "{name}: the same object");
        rig.stop().await;
    }
}

#[tokio::test]
async fn moving_an_annotated_tag_is_undone_to_the_old_tag_object_and_redone_to_the_new_one() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .annotated_tag("v1")
            .unwrap()
            .commit("second")
            .unwrap()
            .build();
        let mut rig = Rig::on(backend, repo).await;
        let tip = rig.rev("HEAD");
        let old = rig.rev("refs/tags/v1");

        rig.ok(
            OperationKind::TagMove,
            json!({ "name": "v1", "target": tip, "force": false }),
        )
        .await;
        let moved = rig.rev("refs/tags/v1");
        assert_ne!(moved, old, "{name}: a new tag object");
        assert_eq!(rig.rev("v1^{commit}"), tip, "{name}");

        // One entry: a single undo is back where it was, tag object included.
        rig.undo().await;
        assert_eq!(rig.rev("refs/tags/v1"), old, "{name}");
        rig.repo.git(&["gc", "-q", "--prune=now"]);
        rig.redo().await;
        assert_eq!(rig.rev("refs/tags/v1"), moved, "{name}: the same object");
        rig.stop().await;
    }
}

#[tokio::test]
async fn merging_a_branch_into_a_tag_is_one_undo_entry() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .branch("side")
            .unwrap()
            .commit_file("side.txt", "side\n", "side work")
            .unwrap()
            .checkout("main")
            .unwrap()
            .commit_file("main.txt", "main\n", "main work")
            .unwrap()
            .tag("v1")
            .unwrap()
            .build();
        let mut rig = Rig::on(backend, repo).await;
        let old = rig.rev("refs/tags/v1");

        rig.ok(
            OperationKind::TagMergeBranch,
            json!({ "name": "v1", "branch": "side", "message": null }),
        )
        .await;
        let merged = rig.rev("refs/tags/v1");
        assert_ne!(merged, old, "{name}");

        rig.undo().await;
        assert_eq!(
            rig.rev("refs/tags/v1"),
            old,
            "{name}: one undo undoes the whole thing"
        );
        rig.redo().await;
        assert_eq!(rig.rev("refs/tags/v1"), merged, "{name}");
        rig.stop().await;
    }
}

// ------------------------------------------------------- fast-forward

fn ff_repo() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .branch("feature")
        .unwrap()
        .checkout("feature")
        .unwrap()
        .commit_file("feature.txt", "from feature\n", "feature work")
        .unwrap()
        .checkout("main")
        .unwrap()
        .branch("other")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build()
}

#[tokio::test]
async fn fast_forwarding_a_branch_that_is_not_checked_out_moves_only_its_ref() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, ff_repo()).await;
        let base = rig.rev("other");
        let target = rig.rev("feature");

        let answer = rig
            .ok(
                OperationKind::FastForward,
                json!({ "branch": "other", "target": target }),
            )
            .await;
        let outcome: FastForwardOutcome = serde_json::from_value(answer).unwrap();
        assert_eq!(
            outcome,
            FastForwardOutcome::FastForwarded {
                from: base,
                to: target
            }
        );
        assert_eq!(rig.rev("other"), target, "{name}");

        let out = rig.undo().await;
        assert!(out.entry.label.starts_with("Fast-forward other"), "{name}");
        assert_eq!(rig.rev("other"), base, "{name}");
        assert!(!rig.repo.root().join("feature.txt").exists(), "{name}");
        rig.redo().await;
        assert_eq!(rig.rev("other"), target, "{name}");
        rig.undo().await;

        // Moved outside since: refuse.
        rig.redo().await;
        rig.repo.git(&["branch", "-f", "other", "main"]);
        let refusal = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(&refusal, UndoRefusal::RefMoved { name, .. } if name == "refs/heads/other"),
            "{name}: {refusal:?}"
        );
        assert_eq!(rig.rev("other"), base, "{name}: nothing changed");
        rig.stop().await;
    }
}

#[tokio::test]
async fn fast_forwarding_the_checked_out_branch_undoes_with_its_files_and_keeps_local_edits() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, ff_repo()).await;
        let base = rig.rev("main");
        let target = rig.rev("feature");
        std::fs::write(rig.repo.root().join("mine.txt"), "mine").unwrap();

        rig.ok(
            OperationKind::FastForward,
            json!({ "branch": "main", "target": target }),
        )
        .await;
        assert!(rig.repo.root().join("feature.txt").exists(), "{name}");

        rig.undo().await;
        assert_eq!(rig.rev("main"), base, "{name}");
        assert_eq!(rig.rev("HEAD"), base, "{name}");
        assert!(
            !rig.repo.root().join("feature.txt").exists(),
            "{name}: the files followed the branch back"
        );
        assert!(
            rig.repo.root().join("mine.txt").exists(),
            "{name}: an unrelated file stays"
        );
        assert_eq!(rig.repo.git(&["status", "--porcelain"]), "?? mine.txt");

        rig.redo().await;
        assert_eq!(rig.rev("main"), target, "{name}");
        assert!(rig.repo.root().join("feature.txt").exists(), "{name}");

        // A local edit to a file the undo would rewrite: refused, typed,
        // and nothing moved; the entry is still undoable afterwards.
        std::fs::write(rig.repo.root().join("feature.txt"), "my edit\n").unwrap();
        let refusal = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(refusal, UndoRefusal::WouldOverwrite { .. }),
            "{name}: {refusal:?}"
        );
        assert_eq!(rig.rev("main"), target, "{name}: the branch did not move");
        assert_eq!(
            std::fs::read_to_string(rig.repo.root().join("feature.txt")).unwrap(),
            "my edit\n",
            "{name}: the edit survived"
        );
        rig.repo.git(&["checkout", "--", "feature.txt"]);
        rig.undo().await;
        assert_eq!(rig.rev("main"), base, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_fast_forward_that_changes_nothing_records_no_entry() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, ff_repo()).await;
        let base = rig.rev("main");
        let answer = rig
            .ok(
                OperationKind::FastForward,
                json!({ "branch": "main", "target": base }),
            )
            .await;
        assert_eq!(
            serde_json::from_value::<FastForwardOutcome>(answer).unwrap(),
            FastForwardOutcome::UpToDate
        );
        assert_eq!(
            rig.refused(OperationKind::Undo).await,
            UndoRefusal::NothingToUndo,
            "{name}"
        );
        rig.stop().await;
    }
}

// ------------------------------------------------------------- reword

#[tokio::test]
async fn reword_undo_restores_the_old_tip_and_redo_the_same_new_one() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let first = rig.commit_file("one.txt", "first message").await;
        let second = rig.commit_file("two.txt", "second message").await;
        // A staged change that the reword must not disturb either way.
        std::fs::write(rig.repo.root().join("staged.txt"), "s").unwrap();
        rig.ok(
            OperationKind::StageFile,
            json!({ "target": StageTarget::File("staged.txt".into()) }),
        )
        .await;

        // An older commit: its descendant is replayed on top.
        rig.ok(
            OperationKind::Reword,
            json!({ "commit": first, "message": "better first message" }),
        )
        .await;
        let reworded_tip = rig.rev("HEAD");
        assert_ne!(reworded_tip, second, "{name}");
        assert_eq!(
            rig.repo.git(&["log", "-1", "--format=%s", "HEAD~1"]),
            "better first message",
            "{name}"
        );

        let out = rig.undo().await;
        assert!(out.entry.label.starts_with("Reword"), "{name}");
        assert_eq!(rig.rev("HEAD"), second, "{name}: the old tip, same oid");
        assert_eq!(rig.rev("main"), second, "{name}");
        assert_eq!(
            rig.repo.git(&["status", "--porcelain"]),
            "A  staged.txt",
            "{name}: index and files untouched"
        );
        rig.redo().await;
        assert_eq!(
            rig.rev("HEAD"),
            reworded_tip,
            "{name}: redo never re-rewrites"
        );

        // A commit on top since: the entry requires HEAD, so it refuses.
        rig.repo.git(&["commit", "-q", "-m", "outside"]);
        let refusal = rig.refused(OperationKind::Undo).await;
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

// ----------------------------------------------- fetch ref as branch

fn pr_repo() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .branch("work")
        .unwrap()
        .checkout("work")
        .unwrap()
        .commit_file("pr.txt", "one", "pr commit")
        .unwrap()
        .push("work")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    let tip = repo.git(&["rev-parse", "work"]);
    repo.git_in_remote(&["update-ref", "refs/pull/7/head", &tip]);
    repo
}

#[tokio::test]
async fn fetch_ref_as_branch_is_undone_for_the_created_and_the_fast_forwarded_branch() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, pr_repo()).await;
        let first = rig.rev("work");
        let args =
            json!({ "remote": "origin", "remote_ref": "refs/pull/7/head", "branch": "pr/7" });

        // Created.
        rig.ok(OperationKind::FetchRefAsBranch, args.clone()).await;
        assert_eq!(rig.rev("refs/heads/pr/7"), first, "{name}");
        assert_eq!(rig.config("branch.pr/7.remote"), "origin", "{name}");
        let out = rig.undo().await;
        assert!(out.entry.label.contains("as branch pr/7"), "{name}");
        assert!(!rig.has_ref("refs/heads/pr/7"), "{name}");
        assert!(!rig.has_config("branch.pr/7.remote"), "{name}");
        rig.redo().await;
        assert_eq!(rig.rev("refs/heads/pr/7"), first, "{name}");
        assert_eq!(rig.config("branch.pr/7.remote"), "origin", "{name}");

        // The PR moves; a second fetch fast-forwards the local branch.
        rig.repo.git(&["checkout", "-q", "work"]);
        rig.repo
            .git(&["commit", "-q", "--allow-empty", "-m", "more"]);
        let moved = rig.rev("work");
        rig.repo.git(&["push", "-q", "origin", "work"]);
        rig.repo.git(&["checkout", "-q", "main"]);
        rig.repo
            .git_in_remote(&["update-ref", "refs/pull/7/head", &moved.to_hex()]);
        rig.ok(OperationKind::FetchRefAsBranch, args.clone()).await;
        assert_eq!(rig.rev("refs/heads/pr/7"), moved, "{name}");
        rig.undo().await;
        assert_eq!(rig.rev("refs/heads/pr/7"), first, "{name}");
        assert!(rig.has_config("branch.pr/7.remote"), "{name}: config stays");
        rig.redo().await;
        assert_eq!(rig.rev("refs/heads/pr/7"), moved, "{name}");

        // Moved outside since: refuse.
        rig.repo.git(&["branch", "-f", "pr/7", "main"]);
        let refusal = rig.refused(OperationKind::Undo).await;
        assert!(matches!(refusal, UndoRefusal::RefMoved { .. }), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn fetch_ref_as_branch_on_the_checked_out_branch_moves_the_files_back() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, pr_repo()).await;
        let args =
            json!({ "remote": "origin", "remote_ref": "refs/pull/7/head", "branch": "pr/7" });
        rig.ok(OperationKind::FetchRefAsBranch, args.clone()).await;
        let first = rig.rev("pr/7");
        rig.repo.git(&["checkout", "-q", "pr/7"]);
        assert!(rig.repo.root().join("pr.txt").exists());

        // The PR gains a commit with a new file.
        rig.repo.git(&["checkout", "-q", "work"]);
        std::fs::write(rig.repo.root().join("more.txt"), "more").unwrap();
        rig.repo.git(&["add", "more.txt"]);
        rig.repo.git(&["commit", "-q", "-m", "more"]);
        let moved = rig.rev("work");
        rig.repo.git(&["push", "-q", "origin", "work"]);
        rig.repo.git(&["checkout", "-q", "pr/7"]);
        rig.repo
            .git_in_remote(&["update-ref", "refs/pull/7/head", &moved.to_hex()]);

        rig.ok(OperationKind::FetchRefAsBranch, args).await;
        assert_eq!(rig.rev("HEAD"), moved, "{name}");
        assert!(rig.repo.root().join("more.txt").exists(), "{name}");
        rig.undo().await;
        assert_eq!(rig.rev("HEAD"), first, "{name}");
        assert!(
            !rig.repo.root().join("more.txt").exists(),
            "{name}: the files followed the checked-out branch back"
        );
        rig.redo().await;
        assert!(rig.repo.root().join("more.txt").exists(), "{name}");
        rig.stop().await;
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();

// ------------------------------------------------------------- amend commit

#[tokio::test]
async fn amend_commit_undo_restores_the_old_tip_and_redo_the_same_new_one() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let first = rig.commit_file("one.txt", "first message").await;
        let second = rig.commit_file("two.txt", "second message").await;
        std::fs::write(rig.repo.root().join("staged.txt"), "s").unwrap();
        rig.ok(
            OperationKind::StageFile,
            json!({ "target": StageTarget::File("staged.txt".into()) }),
        )
        .await;

        rig.ok(
            OperationKind::AmendCommit,
            json!({ "commit": first, "amend": { "author_name": "Ada Lovelace", "author_email": "ada@example.org" } }),
        )
        .await;
        let amended_tip = rig.rev("HEAD");
        assert_ne!(amended_tip, second, "{name}");
        assert_eq!(
            rig.repo.git(&["log", "-1", "--format=%an <%ae>", "HEAD~1"]),
            "Ada Lovelace <ada@example.org>",
            "{name}"
        );

        let out = rig.undo().await;
        assert!(out.entry.label.starts_with("Amend"), "{name}");
        assert_eq!(rig.rev("HEAD"), second, "{name}: the old tip, same oid");
        assert_eq!(
            rig.repo.git(&["status", "--porcelain"]),
            "A  staged.txt",
            "{name}: index and files untouched"
        );
        rig.redo().await;
        assert_eq!(
            rig.rev("HEAD"),
            amended_tip,
            "{name}: redo never re-rewrites"
        );
        rig.stop().await;
    }
}
