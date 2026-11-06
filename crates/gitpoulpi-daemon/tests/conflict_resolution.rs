//! The merge conflict editor's write side through the daemon (plan
//! `merge-conflict-editor.md` 3.6, 3.7, 7, maintainer answers 1 and 4): a
//! "Save & mark resolved" is journaled and Ctrl+Z puts the conflict back
//! while the operation runs, `ConflictUnresolve` works from the daemon's
//! record, a whitespace-only conflict resolves itself, and the new reads
//! answer. Every test runs against both backends.

use std::sync::Arc;

use gitpoulpi_core::domain::{
    CommitOptions, ConflictEntry, ConflictKind, ConflictRefusal, ConflictResolution, Direction,
    JournalState, MergeDocument, OperationKind, OperationState, RepoId, ResolveOptions,
    StageTarget, UndoOutcome, UndoRefusal,
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
    /// `file` is `base` on main and feature, then `ours` on main and
    /// `theirs` on feature; main checked out.
    async fn diverged(
        backend: Arc<dyn GitBackend>,
        file: &str,
        base: &str,
        ours: &str,
        theirs: &str,
    ) -> Rig {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .build();
        let handle = Daemon::new("conflict-resolution-test")
            .spawn(backend)
            .await
            .unwrap();
        let client = Client::connect(&handle.endpoint()).await.unwrap();
        let id = repo.repo_ref().id;
        let rig = Rig {
            handle: Some(handle),
            client,
            repo,
            id,
            next: 1,
        };
        rig.commit(file, base, "base");
        rig.repo.git(&["branch", "feature"]);
        rig.repo.git(&["checkout", "-q", "feature"]);
        rig.commit(file, theirs, "theirs");
        rig.repo.git(&["checkout", "-q", "main"]);
        rig.commit(file, ours, "ours");
        rig
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
                session: "conflict-resolution-test".into(),
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

    async fn merge_feature(&mut self) {
        let refs = self.ok(OperationKind::Refs, json!({})).await;
        let feature = refs
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["kind"]["LocalBranch"]["name"] == "feature")
            .unwrap()
            .clone();
        self.ok(OperationKind::Merge, json!({ "from": feature }))
            .await;
    }

    async fn entries(&mut self) -> Vec<ConflictEntry> {
        serde_json::from_value(self.ok(OperationKind::ConflictEntries, json!({})).await).unwrap()
    }

    async fn resolve(&mut self, path: &str, text: &str) {
        self.ok(
            OperationKind::ResolveConflict,
            json!({ "path": path, "resolution": ConflictResolution::UseContent(text.into()) }),
        )
        .await;
    }

    async fn journal(&mut self) -> JournalState {
        serde_json::from_value(self.ok(OperationKind::JournalState, json!({})).await).unwrap()
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

    async fn undo_refusal(&mut self) -> UndoRefusal {
        let e = self.call(OperationKind::Undo, json!({})).await.unwrap_err();
        match e.kind {
            Some(TypedError::UndoRefused(r)) => r,
            other => panic!("expected an undo refusal, got {other:?} ({})", e.message),
        }
    }

    fn file(&self, name: &str) -> String {
        std::fs::read_to_string(self.repo.root().join(name)).unwrap()
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

async fn real_conflict(backend: Arc<dyn GitBackend>) -> Rig {
    Rig::diverged(backend, "c.txt", "base\n", "ours\n", "theirs\n").await
}

#[tokio::test]
async fn save_and_mark_resolved_is_undone_and_redone_while_the_merge_runs() {
    for (name, backend) in backends() {
        let mut rig = real_conflict(backend).await;
        rig.merge_feature().await;
        let markers = rig.file("c.txt");
        assert!(markers.contains("<<<<<<<"), "{name}: a real conflict stays");
        let before = rig.entries().await;
        assert_eq!(before.len(), 1, "{name}");

        rig.resolve("c.txt", "both\n").await;
        assert!(rig.entries().await.is_empty(), "{name}");
        let state = rig.journal().await;
        assert_eq!(
            state.undo.as_ref().unwrap().label,
            "Resolve c.txt",
            "{name}"
        );
        assert_eq!(state.blocked, None, "{name}: undoable during the merge");

        let out = rig.undo().await;
        assert!(out.entry.label.starts_with("Resolve c.txt"), "{name}");
        assert_eq!(rig.entries().await, before, "{name}: the conflict is back");
        assert_eq!(rig.file("c.txt"), markers, "{name}: markers as git wrote");

        rig.redo().await;
        assert!(rig.entries().await.is_empty(), "{name}");
        assert_eq!(rig.file("c.txt"), "both\n", "{name}");

        // And again, from the redone state.
        rig.undo().await;
        assert_eq!(rig.entries().await, before, "{name}");
        rig.redo().await;

        // Concluding the merge ends the resolution's undo: the next undo
        // is the merge itself.
        rig.ok(
            OperationKind::Commit,
            json!({ "message": "merge", "opts": CommitOptions::default() }),
        )
        .await;
        let out = rig.undo().await;
        assert!(
            out.entry.label.starts_with("Merge feature"),
            "{name}: {}",
            out.entry.label
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn an_undo_of_a_resolution_is_refused_when_the_file_was_edited_since() {
    for (name, backend) in backends() {
        let mut rig = real_conflict(backend).await;
        rig.merge_feature().await;
        rig.resolve("c.txt", "both\n").await;
        std::fs::write(rig.repo.root().join("c.txt"), "edited later\n").unwrap();
        // The journal state names why (the buttons gray out).
        let state = rig.journal().await;
        assert!(
            matches!(state.blocked, Some(UndoRefusal::PathChanged { .. })),
            "{name}: {:?}",
            state.blocked
        );
        assert!(matches!(
            rig.undo_refusal().await,
            UndoRefusal::PathChanged { .. }
        ));
        assert_eq!(rig.file("c.txt"), "edited later\n", "{name}: untouched");
        rig.stop().await;
    }
}

#[tokio::test]
async fn aborting_the_operation_drops_the_resolutions_and_the_pending_merge_entry() {
    for (name, backend) in backends() {
        let mut rig = real_conflict(backend).await;
        let head = rig.repo.git(&["rev-parse", "HEAD"]);
        rig.merge_feature().await;
        rig.resolve("c.txt", "both\n").await;
        rig.ok(OperationKind::OperationAbort, json!({})).await;
        assert_eq!(rig.repo.git(&["rev-parse", "HEAD"]), head, "{name}");
        assert_eq!(rig.file("c.txt"), "ours\n", "{name}");
        // The resolution and the merge's pending entry are gone; what is
        // left to undo is the abort itself.
        let state = rig.journal().await;
        assert_eq!(state.undo.as_ref().unwrap().label, "Abort merge", "{name}");
        assert_eq!(state.blocked, None, "{name}");
        let state: Option<OperationState> =
            serde_json::from_value(rig.ok(OperationKind::OperationState, json!({})).await).unwrap();
        assert_eq!(state, None, "{name}");
        rig.undo().await;
        let state: Option<OperationState> =
            serde_json::from_value(rig.ok(OperationKind::OperationState, json!({})).await).unwrap();
        assert!(state.is_some(), "{name}: the merge is back");
        rig.stop().await;
    }
}

#[tokio::test]
async fn skipping_an_empty_cherry_pick_ends_it_and_settles_its_pending_entry() {
    for (name, backend) in backends() {
        let mut rig = real_conflict(backend).await;
        let head = rig.repo.git(&["rev-parse", "HEAD"]);
        let feature = rig.repo.git(&["rev-parse", "feature"]);
        // A conflicted pick, resolved by taking ours: now empty.
        let _ = rig
            .call(
                OperationKind::CherryPick,
                json!({ "commits": [feature], "opts": {} }),
            )
            .await;
        let state: Option<OperationState> =
            serde_json::from_value(rig.ok(OperationKind::OperationState, json!({})).await).unwrap();
        assert!(
            matches!(state, Some(OperationState::CherryPick { .. })),
            "{name}: {state:?}"
        );
        rig.repo.git(&["checkout", "--ours", "c.txt"]);
        rig.repo.git(&["add", "c.txt"]);
        rig.ok(OperationKind::OperationSkip, json!({})).await;
        let state: Option<OperationState> =
            serde_json::from_value(rig.ok(OperationKind::OperationState, json!({})).await).unwrap();
        assert_eq!(state, None, "{name}");
        assert_eq!(rig.repo.git(&["rev-parse", "HEAD"]), head, "{name}");
        // No pending journal entry is left stuck by the skipped pick.
        assert_eq!(rig.journal().await.blocked, None, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_merge_whose_result_equals_head_is_committed_with_nothing_staged() {
    for (name, backend) in backends() {
        let mut rig = real_conflict(backend).await;
        rig.merge_feature().await;
        rig.repo.git(&["checkout", "--ours", "c.txt"]);
        rig.repo.git(&["add", "c.txt"]);
        rig.ok(
            OperationKind::Commit,
            json!({ "message": "Merge feature (ours)", "opts": CommitOptions::default() }),
        )
        .await;
        let parents = rig.repo.git(&["rev-list", "--parents", "-n1", "HEAD"]);
        assert_eq!(parents.split_whitespace().count(), 3, "{name}: {parents}");
        let state: Option<OperationState> =
            serde_json::from_value(rig.ok(OperationKind::OperationState, json!({})).await).unwrap();
        assert_eq!(state, None, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn staging_during_the_merge_is_undone_before_the_resolution() {
    for (name, backend) in backends() {
        let mut rig = real_conflict(backend).await;
        rig.merge_feature().await;
        rig.resolve("c.txt", "both\n").await;
        std::fs::write(rig.repo.root().join("other.txt"), "x\n").unwrap();
        rig.ok(
            OperationKind::StageFile,
            json!({ "target": StageTarget::File("other.txt".into()) }),
        )
        .await;
        // The staging is on top, then the resolution under it.
        let out = rig.undo().await;
        assert_eq!(out.entry.label, "Stage other.txt", "{name}");
        let out = rig.undo().await;
        assert!(out.entry.label.starts_with("Resolve c.txt"), "{name}");
        assert_eq!(rig.entries().await.len(), 1, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn unresolve_uses_the_record_of_what_was_resolved() {
    for (name, backend) in backends() {
        let mut rig = real_conflict(backend).await;
        rig.merge_feature().await;
        let before = rig.entries().await;
        let markers = rig.file("c.txt");

        // Nothing resolved yet: no record, a plain error.
        assert!(
            rig.call(OperationKind::ConflictUnresolve, json!({ "path": "c.txt" }))
                .await
                .is_err(),
            "{name}"
        );

        rig.resolve("c.txt", "both\n").await;
        rig.ok(OperationKind::ConflictUnresolve, json!({ "path": "c.txt" }))
            .await;
        assert_eq!(rig.entries().await, before, "{name}");
        assert_eq!(rig.file("c.txt"), markers, "{name}");
        // The record was spent.
        assert!(
            rig.call(OperationKind::ConflictUnresolve, json!({ "path": "c.txt" }))
                .await
                .is_err(),
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_stale_resolution_is_a_typed_refusal_and_save_without_staging_is_not_journaled() {
    for (name, backend) in backends() {
        let mut rig = real_conflict(backend).await;
        rig.merge_feature().await;
        let before = rig.entries().await;
        let [base, ours, theirs] = ResolveOptions::oids_of(&before[0]);

        let stale = ResolveOptions {
            stage: true,
            expected: Some([base, theirs, ours]),
        };
        let e = rig
            .call(
                OperationKind::ResolveConflict,
                json!({
                    "path": "c.txt",
                    "resolution": ConflictResolution::UseContent("no\n".into()),
                    "opts": stale,
                }),
            )
            .await
            .unwrap_err();
        assert_eq!(
            e.kind,
            Some(TypedError::ConflictRefused(ConflictRefusal::Stale)),
            "{name}"
        );
        assert_eq!(rig.entries().await, before, "{name}");

        // "Save": the file changes, the stages stay, and there is nothing
        // to undo (still conflicted).
        let save = ResolveOptions {
            stage: false,
            expected: Some([base, ours, theirs]),
        };
        rig.ok(
            OperationKind::ResolveConflict,
            json!({
                "path": "c.txt",
                "resolution": ConflictResolution::UseContent("saved\n".into()),
                "opts": save,
            }),
        )
        .await;
        assert_eq!(rig.file("c.txt"), "saved\n", "{name}");
        assert_eq!(rig.entries().await, before, "{name}");
        assert!(
            rig.undo_refusal().await == UndoRefusal::InProgress { op: "merge".into() },
            "{name}: only the pending merge entry"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn the_reads_answer_with_the_stages_the_document_and_the_operation() {
    for (name, backend) in backends() {
        let mut rig = real_conflict(backend).await;
        rig.merge_feature().await;
        let entries = rig.entries().await;
        assert_eq!(entries.len(), 1, "{name}");
        assert_eq!(entries[0].kind, ConflictKind::BothModified, "{name}");

        let doc: MergeDocument = serde_json::from_value(
            rig.ok(OperationKind::ConflictShow, json!({ "path": "c.txt" }))
                .await,
        )
        .unwrap();
        assert_eq!(doc.conflict_count(), 1, "{name}");
        assert_eq!(doc.entry, entries[0], "{name}");

        let state: Option<OperationState> =
            serde_json::from_value(rig.ok(OperationKind::OperationState, json!({})).await).unwrap();
        assert!(
            matches!(&state, Some(OperationState::Merge { their_label, .. }) if their_label == "feature"),
            "{name}: {state:?}"
        );
        rig.stop().await;
    }
}

// -- whitespace-only conflicts resolve themselves ------------------------------------------

#[tokio::test]
async fn a_whitespace_only_conflict_is_resolved_by_the_merge_and_undoable() {
    for (name, backend) in backends() {
        // Both sides touched the same line, one with a trailing space, the
        // other with a trailing tab: git conflicts, nobody cares which.
        let mut rig = Rig::diverged(
            backend,
            "w.txt",
            "keep\nvalue = 1\nend\n",
            "keep\nvalue = 1 \nend\n",
            "keep\nvalue = 1\t\nend\n",
        )
        .await;
        rig.merge_feature().await;

        assert!(rig.entries().await.is_empty(), "{name}: nothing left to do");
        assert_eq!(rig.file("w.txt"), "keep\nvalue = 1 \nend\n", "{name}: ours");
        let state = rig.journal().await;
        let label = state.undo.unwrap().label;
        assert!(
            label.starts_with("Auto-resolve")
                && label.contains("whitespace only")
                && label.contains("w.txt"),
            "{name}: {label}"
        );
        // The merge is still open for the person to conclude.
        let op: Option<OperationState> =
            serde_json::from_value(rig.ok(OperationKind::OperationState, json!({})).await).unwrap();
        assert!(matches!(op, Some(OperationState::Merge { .. })), "{name}");

        // Undoable like any resolution: the conflict is back.
        rig.undo().await;
        assert_eq!(rig.entries().await.len(), 1, "{name}");
        assert!(rig.file("w.txt").contains("<<<<<<<"), "{name}");
        rig.redo().await;
        assert!(rig.entries().await.is_empty(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn ambiguous_whitespace_and_real_conflicts_are_left_alone() {
    for (name, backend) in backends() {
        // Different indentation on each side: whitespace only, but which?
        let mut rig = Rig::diverged(backend, "w.txt", "a\nx\n", "a\n  x\n", "a\n\tx\n").await;
        rig.merge_feature().await;
        assert_eq!(rig.entries().await.len(), 1, "{name}");
        assert!(rig.file("w.txt").contains("<<<<<<<"), "{name}");
        rig.stop().await;
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
