//! Undo/redo of the history-moving operations through the daemon
//! (the undo-redo design): merge (fast-forward and
//! merge commit), rebase (plain and interactive, across a conflict),
//! cherry-pick, revert, reset, pull (local part) and the AI composer.
//! Every test runs against both backends.

use std::sync::Arc;

use gitpoulpi_core::domain::{
    CommitOptions, ConflictResolution, Direction, Oid, OperationKind, RepoId, StageTarget,
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
        let handle = Daemon::new("undo-history-test")
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
                session: "undo-history-test".into(),
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

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

impl Rig {
    /// The `Ref` JSON of a local branch, as the daemon's `Refs` read gives it.
    async fn branch_ref(&mut self, name: &str) -> Value {
        let refs = self.ok(OperationKind::Refs, json!({})).await;
        refs.as_array()
            .unwrap()
            .iter()
            .find(|r| r["kind"]["LocalBranch"]["name"] == name)
            .unwrap_or_else(|| panic!("no branch {name}"))
            .clone()
    }

    fn commit_in_shell(&self, file: &str, contents: &str, message: &str) -> Oid {
        std::fs::write(self.repo.root().join(file), contents).unwrap();
        self.repo.git(&["add", file]);
        self.repo.git(&["commit", "-q", "-m", message]);
        self.rev("HEAD")
    }

    fn file(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(self.repo.root().join(name)).ok()
    }

    async fn confirmation_needed(&mut self, command: OperationKind) -> String {
        let e = self.call(command, json!({})).await.unwrap_err();
        match e.kind {
            Some(TypedError::NeedsConfirmation(p)) => p,
            other => panic!("expected a confirmation, got {other:?} ({})", e.message),
        }
    }

    /// main with `a.txt`, and `feature` one commit (`f.txt`) ahead of it;
    /// main then gets `m.txt` too, so merging feature is a merge commit.
    fn diverged(&self) {
        self.repo.git(&["branch", "feature"]);
        self.repo.git(&["checkout", "-q", "feature"]);
        self.commit_in_shell("f.txt", "f", "feature work");
        self.repo.git(&["checkout", "-q", "main"]);
        self.commit_in_shell("m.txt", "m", "main work");
    }
}

fn labeled(out: &UndoOutcome, start: &str, name: &str) {
    assert!(
        out.entry.label.starts_with(start),
        "{name}: {}",
        out.entry.label
    );
}

// -------------------------------------------------------------- merge

#[tokio::test]
async fn a_merge_commit_is_undone_and_redone_to_the_same_oid() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.diverged();
        let before = rig.rev("HEAD");
        let feature = rig.branch_ref("feature").await;
        rig.ok(OperationKind::Merge, json!({ "from": feature }))
            .await;
        let merged = rig.rev("HEAD");
        assert_ne!(merged, before, "{name}");
        assert!(rig.file("f.txt").is_some(), "{name}");

        let out = rig.undo().await;
        labeled(&out, "Merge feature", name);
        assert_eq!(rig.rev("HEAD"), before, "{name}");
        assert!(rig.file("f.txt").is_none(), "{name}: the files followed");
        assert_eq!(rig.repo.git(&["status", "--porcelain"]), "", "{name}");

        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), merged, "{name}: the same merge commit");
        assert!(rig.file("f.txt").is_some(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_fast_forward_merge_is_undone_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let before = rig.rev("HEAD");
        rig.repo.git(&["checkout", "-q", "-b", "feature"]);
        let tip = rig.commit_in_shell("f.txt", "f", "feature work");
        rig.repo.git(&["checkout", "-q", "main"]);
        let feature = rig.branch_ref("feature").await;
        // The default policy would make a merge commit: ask for the fast-forward.
        rig.ok(
            OperationKind::Merge,
            json!({ "from": feature, "opts": { "ff": "allowed" } }),
        )
        .await;
        assert_eq!(rig.rev("HEAD"), tip, "{name}");
        rig.undo().await;
        assert_eq!(rig.rev("HEAD"), before, "{name}");
        assert!(rig.file("f.txt").is_none(), "{name}");
        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), tip, "{name}");
        assert!(rig.file("f.txt").is_some(), "{name}");
        rig.stop().await;
    }
}

/// The default policy (`never`) merges a branch that could fast-forward with a
/// real merge commit (first parent the old tip); undo and redo move it like
/// any other merge.
#[tokio::test]
async fn a_no_ff_merge_of_a_fast_forwardable_branch_is_undone_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let before = rig.rev("HEAD");
        rig.repo.git(&["checkout", "-q", "-b", "feature"]);
        let tip = rig.commit_in_shell("f.txt", "f", "feature work");
        rig.repo.git(&["checkout", "-q", "main"]);
        let feature = rig.branch_ref("feature").await;
        rig.ok(OperationKind::Merge, json!({ "from": feature }))
            .await;
        let merged = rig.rev("HEAD");
        assert_ne!(merged, tip, "{name}: a merge commit, not a fast-forward");
        let parents = rig.repo.git(&["rev-list", "--parents", "-n1", "HEAD"]);
        let parents: Vec<&str> = parents.split_whitespace().collect();
        assert_eq!(parents.len(), 3, "{name}: two parents");
        assert_eq!(
            parents[1],
            before.to_hex(),
            "{name}: first parent is the old main"
        );
        assert_eq!(parents[2], tip.to_hex(), "{name}");

        let out = rig.undo().await;
        labeled(&out, "Merge feature", name);
        assert_eq!(rig.rev("HEAD"), before, "{name}");
        assert!(rig.file("f.txt").is_none(), "{name}");
        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), merged, "{name}: the same merge commit");
        assert!(rig.file("f.txt").is_some(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_merge_undo_is_refused_after_an_outside_commit() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.diverged();
        let feature = rig.branch_ref("feature").await;
        rig.ok(OperationKind::Merge, json!({ "from": feature }))
            .await;
        let merged = rig.rev("HEAD");
        rig.commit_in_shell("late.txt", "x", "committed in the shell");
        let refusal = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(&refusal, UndoRefusal::RefMoved { name, .. } if name == "refs/heads/main"),
            "{name}: {refusal:?}"
        );
        assert!(rig.file("f.txt").is_some(), "{name}: nothing changed");
        rig.repo.git(&["reset", "-q", "--hard", &merged.to_hex()]);
        rig.undo().await;
        rig.stop().await;
    }
}

#[tokio::test]
async fn undoing_a_merge_with_local_changes_asks_first_and_keeps_them() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.commit_in_shell("keep.txt", "one", "keep file");
        rig.diverged();
        let before = rig.rev("HEAD");
        let feature = rig.branch_ref("feature").await;
        rig.ok(OperationKind::Merge, json!({ "from": feature }))
            .await;
        // A local edit to a file the merge did not touch.
        std::fs::write(rig.repo.root().join("keep.txt"), "edited").unwrap();

        let prompt = rig.confirmation_needed(OperationKind::Undo).await;
        assert!(prompt.contains("uncommitted"), "{name}: {prompt}");
        assert!(rig.file("f.txt").is_some(), "{name}: nothing happened yet");

        rig.ok(OperationKind::Undo, json!({ "confirmed": true }))
            .await;
        assert_eq!(rig.rev("HEAD"), before, "{name}");
        assert_eq!(rig.file("keep.txt").as_deref(), Some("edited"), "{name}");
        assert!(rig.file("f.txt").is_none(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn undoing_a_merge_over_a_local_edit_of_a_merged_file_is_refused_untouched() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.diverged();
        let feature = rig.branch_ref("feature").await;
        rig.ok(OperationKind::Merge, json!({ "from": feature }))
            .await;
        let merged = rig.rev("HEAD");
        std::fs::write(rig.repo.root().join("f.txt"), "my edit").unwrap();
        let e = rig
            .call(OperationKind::Undo, json!({ "confirmed": true }))
            .await
            .unwrap_err();
        assert!(
            matches!(
                e.kind,
                Some(TypedError::UndoRefused(UndoRefusal::WouldOverwrite { .. }))
            ),
            "{name}: {e:?}"
        );
        assert_eq!(rig.rev("HEAD"), merged, "{name}");
        assert_eq!(rig.file("f.txt").as_deref(), Some("my edit"), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_conflicted_merge_leaves_a_pending_entry_and_the_concluding_commit_finishes_it() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.commit_in_shell("c.txt", "base", "base");
        rig.repo.git(&["branch", "feature"]);
        rig.repo.git(&["checkout", "-q", "feature"]);
        rig.commit_in_shell("c.txt", "theirs", "theirs");
        rig.repo.git(&["checkout", "-q", "main"]);
        let before = rig.commit_in_shell("c.txt", "ours", "ours");
        let feature = rig.branch_ref("feature").await;
        rig.ok(OperationKind::Merge, json!({ "from": feature }))
            .await;

        let refusal = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(refusal, UndoRefusal::InProgress { .. }),
            "{name}: {refusal:?}"
        );

        rig.ok(
            OperationKind::ResolveConflict,
            json!({ "path": "c.txt", "resolution": ConflictResolution::UseContent("both".into()) }),
        )
        .await;
        rig.ok(
            OperationKind::StageFile,
            json!({ "target": StageTarget::File("c.txt".into()) }),
        )
        .await;
        rig.ok(
            OperationKind::Commit,
            json!({ "message": "merge", "opts": CommitOptions::default() }),
        )
        .await;
        let merged = rig.rev("HEAD");

        let out = rig.undo().await;
        labeled(&out, "Merge feature", name);
        assert_eq!(rig.rev("HEAD"), before, "{name}");
        assert_eq!(rig.file("c.txt").as_deref(), Some("ours"), "{name}");
        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), merged, "{name}");
        rig.stop().await;
    }
}

// ------------------------------------------------------------- rebase

/// feature has one commit on top of the initial one; main moved on.
fn rebase_setup(rig: &Rig) -> Oid {
    rig.repo.git(&["checkout", "-q", "-b", "feature"]);
    rig.commit_in_shell("f.txt", "f", "feature work");
    rig.repo.git(&["checkout", "-q", "main"]);
    rig.commit_in_shell("m.txt", "m", "main work");
    rig.repo.git(&["checkout", "-q", "feature"]);
    rig.rev("HEAD")
}

#[tokio::test]
async fn a_rebase_is_undone_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let before = rebase_setup(&rig);
        let main = rig.branch_ref("main").await;
        rig.ok(OperationKind::Rebase, json!({ "onto": main })).await;
        let rebased = rig.rev("HEAD");
        assert_ne!(rebased, before, "{name}");
        assert!(rig.file("m.txt").is_some(), "{name}");

        let out = rig.undo().await;
        labeled(&out, "Rebase onto main", name);
        assert_eq!(rig.rev("HEAD"), before, "{name}");
        assert!(rig.file("m.txt").is_none(), "{name}");
        assert_eq!(
            rig.repo.git(&["symbolic-ref", "HEAD"]),
            "refs/heads/feature"
        );
        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), rebased, "{name}");
        assert!(rig.file("m.txt").is_some(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn an_interactive_rebase_is_undone_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let base = rig.rev("HEAD");
        rig.commit_in_shell("one.txt", "1", "one");
        rig.commit_in_shell("two.txt", "2", "two");
        let before = rig.rev("HEAD");
        let draft: gitpoulpi_core::domain::RebasePlanDraft = serde_json::from_value(
            rig.ok(OperationKind::RebasePlan, json!({ "upstream": base }))
                .await,
        )
        .unwrap();
        let steps: Vec<_> = draft
            .commits
            .iter()
            .rev()
            .map(|c| json!({ "commit": c.id, "action": "Pick", "message": null }))
            .collect();
        rig.ok(
            OperationKind::RebaseInteractive,
            json!({ "request": {
                "upstream": base, "onto": null, "steps": steps,
                "autosquash": false, "cherry_pick": false
            }}),
        )
        .await;
        let reordered = rig.rev("HEAD");
        assert_ne!(reordered, before, "{name}");
        let out = rig.undo().await;
        labeled(&out, "Interactive rebase", name);
        assert_eq!(rig.rev("HEAD"), before, "{name}");
        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), reordered, "{name}");
        rig.stop().await;
    }
}

fn conflicted_rebase_setup(rig: &Rig) -> Oid {
    rig.commit_in_shell("c.txt", "base", "base");
    rig.repo.git(&["checkout", "-q", "-b", "feature"]);
    rig.commit_in_shell("c.txt", "feature", "feature edit");
    rig.repo.git(&["checkout", "-q", "main"]);
    rig.commit_in_shell("c.txt", "main", "main edit");
    rig.repo.git(&["checkout", "-q", "feature"]);
    rig.rev("HEAD")
}

#[tokio::test]
async fn a_conflicted_rebase_is_pending_until_continued_then_one_undoable_entry() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let before = conflicted_rebase_setup(&rig);
        let main = rig.branch_ref("main").await;
        rig.ok(OperationKind::Rebase, json!({ "onto": main })).await;

        let refusal = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(refusal, UndoRefusal::InProgress { .. }),
            "{name}: {refusal:?}"
        );

        rig.ok(
            OperationKind::ResolveConflict,
            json!({ "path": "c.txt", "resolution": ConflictResolution::UseContent("fixed".into()) }),
        )
        .await;
        rig.ok(
            OperationKind::StageFile,
            json!({ "target": StageTarget::File("c.txt".into()) }),
        )
        .await;
        rig.ok(OperationKind::RebaseContinue, json!({})).await;
        let rebased = rig.rev("HEAD");
        assert_ne!(rebased, before, "{name}");

        let out = rig.undo().await;
        labeled(&out, "Rebase onto main", name);
        assert_eq!(rig.rev("HEAD"), before, "{name}");
        assert_eq!(rig.file("c.txt").as_deref(), Some("feature"), "{name}");
        // One entry only.
        assert_eq!(
            rig.refused(OperationKind::Undo).await,
            UndoRefusal::NothingToUndo,
            "{name}"
        );
        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), rebased, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn an_aborted_rebase_leaves_no_entry() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let before = conflicted_rebase_setup(&rig);
        let main = rig.branch_ref("main").await;
        rig.ok(OperationKind::Rebase, json!({ "onto": main })).await;
        rig.ok(OperationKind::RebaseAbort, json!({})).await;
        assert_eq!(rig.rev("HEAD"), before, "{name}");
        assert_eq!(
            rig.refused(OperationKind::Undo).await,
            UndoRefusal::NothingToUndo,
            "{name}"
        );
        let pins = rig.repo.git(&["for-each-ref", "refs/gitpoulpi/"]);
        assert_eq!(pins, "", "{name}: the aborted entry's pins are released");
        rig.stop().await;
    }
}

// -------------------------------------------- cherry-pick and revert

#[tokio::test]
async fn a_cherry_pick_series_is_undone_as_one_entry_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.repo.git(&["checkout", "-q", "-b", "other"]);
        let a = rig.commit_in_shell("a.txt", "a", "pick a");
        let b = rig.commit_in_shell("b.txt", "b", "pick b");
        rig.repo.git(&["checkout", "-q", "main"]);
        let before = rig.rev("HEAD");
        rig.ok(OperationKind::CherryPick, json!({ "commits": [a, b] }))
            .await;
        let picked = rig.rev("HEAD");
        assert!(
            rig.file("a.txt").is_some() && rig.file("b.txt").is_some(),
            "{name}"
        );

        let out = rig.undo().await;
        labeled(&out, "Cherry-pick 2 commits", name);
        assert_eq!(rig.rev("HEAD"), before, "{name}");
        assert!(
            rig.file("a.txt").is_none() && rig.file("b.txt").is_none(),
            "{name}"
        );
        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), picked, "{name}");
        assert!(rig.file("b.txt").is_some(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_revert_is_undone_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let target = rig.commit_in_shell("r.txt", "r", "add r");
        let before = rig.rev("HEAD");
        rig.ok(OperationKind::Revert, json!({ "commit": target }))
            .await;
        let reverted = rig.rev("HEAD");
        assert!(rig.file("r.txt").is_none(), "{name}");
        let out = rig.undo().await;
        labeled(&out, "Revert", name);
        assert_eq!(rig.rev("HEAD"), before, "{name}");
        assert!(rig.file("r.txt").is_some(), "{name}");
        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), reverted, "{name}");
        assert!(rig.file("r.txt").is_none(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn the_revert_of_a_merge_commit_is_undone_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.repo.git(&["checkout", "-q", "-b", "feature"]);
        rig.commit_in_shell("feature.txt", "f", "feature work");
        rig.repo.git(&["checkout", "-q", "-"]);
        rig.commit_in_shell("main.txt", "m", "main work");
        rig.repo
            .git(&["merge", "-q", "--no-ff", "-m", "merge feature", "feature"]);
        let merge = rig.rev("HEAD");
        rig.ok(
            OperationKind::Revert,
            json!({ "commit": merge, "mainline": 1 }),
        )
        .await;
        let reverted = rig.rev("HEAD");
        assert!(rig.file("feature.txt").is_none(), "{name}");
        assert!(rig.file("main.txt").is_some(), "{name}");
        let out = rig.undo().await;
        labeled(&out, "Revert", name);
        assert_eq!(rig.rev("HEAD"), merge, "{name}");
        assert!(rig.file("feature.txt").is_some(), "{name}");
        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), reverted, "{name}");
        assert!(rig.file("feature.txt").is_none(), "{name}");
        rig.stop().await;
    }
}

// -------------------------------------------------------------- reset

#[tokio::test]
async fn a_soft_reset_is_undone_and_keeps_the_index() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let first = rig.commit_in_shell("one.txt", "1", "one");
        let second = rig.commit_in_shell("two.txt", "2", "two");
        rig.ok(
            OperationKind::Reset,
            json!({ "target": first, "mode": "Soft" }),
        )
        .await;
        assert_eq!(rig.rev("HEAD"), first, "{name}");
        assert_eq!(
            rig.repo.git(&["diff", "--cached", "--name-only"]),
            "two.txt",
            "{name}"
        );
        rig.undo().await;
        assert_eq!(rig.rev("HEAD"), second, "{name}");
        assert_eq!(rig.repo.git(&["status", "--porcelain"]), "", "{name}");
        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), first, "{name}");
        assert_eq!(
            rig.repo.git(&["diff", "--cached", "--name-only"]),
            "two.txt",
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_mixed_reset_is_undone_and_redone_with_the_index_exactly_as_it_was() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let first = rig.commit_in_shell("one.txt", "1", "one");
        let second = rig.commit_in_shell("two.txt", "2", "two");
        // A staged change on top: the reset throws it out of the index.
        std::fs::write(rig.repo.root().join("new.txt"), "n").unwrap();
        rig.repo.git(&["add", "new.txt"]);
        rig.ok(
            OperationKind::Reset,
            json!({ "target": first, "mode": "Mixed" }),
        )
        .await;
        assert_eq!(
            rig.repo.git(&["status", "--porcelain"]),
            "?? new.txt\n?? two.txt",
            "{name}"
        );
        rig.undo().await;
        assert_eq!(rig.rev("HEAD"), second, "{name}");
        assert_eq!(
            rig.repo.git(&["status", "--porcelain"]),
            "A  new.txt",
            "{name}: the staged file is staged again"
        );
        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), first, "{name}");
        assert_eq!(
            rig.repo.git(&["status", "--porcelain"]),
            "?? new.txt\n?? two.txt",
            "{name}"
        );
        // Staging since refuses the undo (the index is not as it was left).
        rig.repo.git(&["add", "two.txt"]);
        let refusal = rig.refused(OperationKind::Undo).await;
        assert_eq!(refusal, UndoRefusal::IndexChanged, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_hard_reset_is_undone_with_the_lost_local_edits_back() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let first = rig.commit_in_shell("one.txt", "1", "one");
        let second = rig.commit_in_shell("two.txt", "2", "two");
        // A staged edit, an unstaged edit and an untracked file.
        std::fs::write(rig.repo.root().join("one.txt"), "staged edit").unwrap();
        rig.repo.git(&["add", "one.txt"]);
        std::fs::write(rig.repo.root().join("two.txt"), "unstaged edit").unwrap();
        std::fs::write(rig.repo.root().join("untracked.txt"), "u").unwrap();

        rig.ok(
            OperationKind::Reset,
            json!({ "target": first, "mode": "Hard" }),
        )
        .await;
        assert_eq!(rig.rev("HEAD"), first, "{name}");
        assert!(rig.file("two.txt").is_none(), "{name}");
        assert_eq!(rig.file("one.txt").as_deref(), Some("1"), "{name}");

        let out = rig.undo().await;
        labeled(&out, "Reset (hard)", name);
        assert_eq!(rig.rev("HEAD"), second, "{name}");
        assert_eq!(
            rig.file("one.txt").as_deref(),
            Some("staged edit"),
            "{name}"
        );
        assert_eq!(
            rig.file("two.txt").as_deref(),
            Some("unstaged edit"),
            "{name}"
        );
        assert_eq!(rig.file("untracked.txt").as_deref(), Some("u"), "{name}");
        assert_eq!(
            rig.repo.git(&["status", "--porcelain"]),
            "M  one.txt\n M two.txt\n?? untracked.txt",
            "{name}: index and worktree as they were"
        );

        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), first, "{name}");
        assert!(rig.file("two.txt").is_none(), "{name}");
        assert_eq!(rig.file("untracked.txt").as_deref(), Some("u"), "{name}");
        rig.undo().await;
        assert_eq!(
            rig.file("two.txt").as_deref(),
            Some("unstaged edit"),
            "{name}"
        );

        // Editing a restored file since refuses the redo and loses nothing.
        std::fs::write(rig.repo.root().join("two.txt"), "newer").unwrap();
        let refusal = rig.refused(OperationKind::Redo).await;
        assert!(
            matches!(refusal, UndoRefusal::PathChanged { .. }),
            "{name}: {refusal:?}"
        );
        assert_eq!(rig.file("two.txt").as_deref(), Some("newer"), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_hard_reset_of_a_clean_tree_is_undone_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let first = rig.commit_in_shell("one.txt", "1", "one");
        let second = rig.commit_in_shell("two.txt", "2", "two");
        rig.ok(
            OperationKind::Reset,
            json!({ "target": first, "mode": "Hard" }),
        )
        .await;
        assert!(rig.file("two.txt").is_none(), "{name}");
        rig.undo().await;
        assert_eq!(rig.rev("HEAD"), second, "{name}");
        assert_eq!(rig.file("two.txt").as_deref(), Some("2"), "{name}");
        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), first, "{name}");
        assert!(rig.file("two.txt").is_none(), "{name}");
        rig.stop().await;
    }
}

// --------------------------------------------------------------- pull

#[tokio::test]
async fn a_pull_is_undone_locally_and_the_fetched_refs_stay() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .with_origin()
            .unwrap()
            .push("main")
            .unwrap()
            .track("main", "origin")
            .unwrap()
            .build();
        let mut rig = Rig::on(backend, repo).await;
        let before = rig.rev("HEAD");
        rig.repo.push_from_elsewhere("main", "someone else");
        rig.ok(OperationKind::Pull, json!({})).await;
        let pulled = rig.rev("HEAD");
        assert_ne!(pulled, before, "{name}");

        let out = rig.undo().await;
        labeled(&out, "Pull", name);
        assert_eq!(rig.rev("HEAD"), before, "{name}");
        assert_eq!(
            rig.rev("origin/main"),
            pulled,
            "{name}: the fetch is not undone"
        );
        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), pulled, "{name}");
        rig.stop().await;
    }
}

// ------------------------------------------------------------ compose

#[tokio::test]
async fn an_ai_compose_of_three_commits_is_one_undoable_entry() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let before = rig.rev("HEAD");
        for f in ["a.txt", "b.txt", "c.txt"] {
            std::fs::write(rig.repo.root().join(f), f).unwrap();
        }
        // The composer unstages everything first; undo gives this back.
        rig.repo.git(&["add", "b.txt"]);
        let made: Vec<Oid> = serde_json::from_value(
            rig.ok(
                OperationKind::ComposeCommits,
                json!({ "commits": [
                    { "message": "add a", "paths": ["a.txt"] },
                    { "message": "add b", "paths": ["b.txt"] },
                    { "message": "add c", "paths": ["c.txt"] },
                ]}),
            )
            .await,
        )
        .unwrap();
        assert_eq!(made.len(), 3, "{name}");
        assert_eq!(rig.rev("HEAD"), made[2], "{name}");

        let out = rig.undo().await;
        labeled(&out, "Compose 3 commits", name);
        assert_eq!(rig.rev("HEAD"), before, "{name}: all three at once");
        assert_eq!(
            rig.repo.git(&["status", "--porcelain"]),
            "A  b.txt\n?? a.txt\n?? c.txt",
            "{name}: the files are back, with the index as it was"
        );
        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), made[2], "{name}");
        assert_eq!(rig.repo.git(&["status", "--porcelain"]), "", "{name}");
        rig.stop().await;
    }
}

// ------------------------------------------- squash left staged (no commit)

/// Decision (repo-settings slice 1): a squash that stages without
/// committing moves no ref, so the journal has nothing to restore and it is
/// **not undoable** as a history entry. The staged changes are the user's to
/// discard (Unstage all, Discard), and an earlier entry must not be undone by
/// mistake: Undo right after it refuses instead of rewinding the merge target.
#[tokio::test]
async fn a_squash_left_staged_is_not_undoable_and_leaves_earlier_entries_alone() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.repo.git(&["branch", "feature"]);
        rig.repo.git(&["checkout", "-q", "feature"]);
        rig.commit_in_shell("f.txt", "f", "feature work");
        rig.repo.git(&["checkout", "-q", "main"]);
        let feature = rig.branch_ref("feature").await;
        let before = rig.rev("HEAD");
        rig.ok(
            OperationKind::Merge,
            json!({ "from": feature, "opts": { "squash": true, "no_commit": true } }),
        )
        .await;
        assert_eq!(rig.rev("HEAD"), before, "{name}: no commit was made");
        assert_eq!(
            rig.repo.git(&["diff", "--cached", "--name-only"]).trim(),
            "f.txt",
            "{name}: the changes are staged"
        );
        let refusal = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(refusal, UndoRefusal::NothingToUndo),
            "{name}: {refusal:?}"
        );
        assert!(
            rig.file("f.txt").is_some(),
            "{name}: still staged and on disk"
        );
        rig.stop().await;
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
