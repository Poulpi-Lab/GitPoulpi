//! "Restore only this", restoring a ref to a reflog position and the reflog
//! hint of a blocked undo, through the daemon
//! (the undo-safety-net design). Every test runs against both
//! backends.

use std::sync::Arc;

use gitpoulpi_core::domain::{
    CheckoutTarget, CommitOptions, Direction, EntryState, JournalListing, JournalState, Oid,
    OperationKind, ReflogPage, RepoId, RestoreItemKind, RestoreOutcome, StageTarget, UndoOutcome,
    UndoRefusal,
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
        // These tests are about a blocked entry: outside changes are not
        // recorded here (outside_journal.rs covers them recorded).
        std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
        static ENV: std::sync::Once = std::sync::Once::new();
        ENV.call_once(|| std::env::set_var("GITPOULPI_GENERAL_UNDO_DEPTH", "0"));
        let handle = Daemon::new("undo-restore-test")
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

    async fn new(backend: Arc<dyn GitBackend>) -> Rig {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", "one\ntwo\n", "initial")
            .unwrap()
            .build();
        Self::on(backend, repo).await
    }

    async fn call(&mut self, command: OperationKind, args: Value) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "undo-restore-test".into(),
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
        serde_json::from_value(self.ok(OperationKind::Undo, json!({})).await).unwrap()
    }

    async fn redo(&mut self) -> UndoOutcome {
        serde_json::from_value(self.ok(OperationKind::Redo, json!({})).await).unwrap()
    }

    async fn listing(&mut self) -> JournalListing {
        serde_json::from_value(self.ok(OperationKind::JournalList, json!({})).await).unwrap()
    }

    /// The id of the newest entry of `kind`.
    async fn entry_id(&mut self, kind: OperationKind) -> u64 {
        self.listing()
            .await
            .undo
            .iter()
            .rev()
            .find(|e| e.summary.kind == kind)
            .unwrap_or_else(|| panic!("no {kind:?} entry"))
            .summary
            .id
    }

    async fn restore(&mut self, id: u64, name: Option<&str>) -> RestoreOutcome {
        serde_json::from_value(
            self.ok(
                OperationKind::JournalRestore,
                json!({ "id": id, "name": name }),
            )
            .await,
        )
        .unwrap()
    }

    async fn restore_refused(&mut self, id: u64, name: Option<&str>) -> UndoRefusal {
        let e = self
            .call(
                OperationKind::JournalRestore,
                json!({ "id": id, "name": name }),
            )
            .await
            .unwrap_err();
        match e.kind {
            Some(TypedError::UndoRefused(r)) => r,
            other => panic!("expected an undo refusal, got {other:?} ({})", e.message),
        }
    }

    async fn state(&mut self) -> JournalState {
        serde_json::from_value(self.ok(OperationKind::JournalState, json!({})).await).unwrap()
    }

    async fn commit(&mut self, message: &str) -> Oid {
        serde_json::from_value(
            self.ok(
                OperationKind::Commit,
                json!({ "message": message, "opts": CommitOptions::default() }),
            )
            .await,
        )
        .unwrap()
    }

    async fn stage(&mut self, path: &str) {
        self.ok(
            OperationKind::StageFile,
            json!({ "target": StageTarget::File(path.into()) }),
        )
        .await;
    }

    fn rev(&self, spec: &str) -> Oid {
        Oid::from_hex(&self.repo.git(&["rev-parse", spec])).unwrap()
    }

    fn path(&self, name: &str) -> std::path::PathBuf {
        self.repo.root().join(name)
    }

    fn read(&self, name: &str) -> String {
        std::fs::read_to_string(self.path(name)).unwrap()
    }

    fn write(&self, name: &str, text: &str) {
        std::fs::write(self.path(name), text).unwrap();
    }

    fn has_ref(&self, name: &str) -> bool {
        !self.repo.git(&["for-each-ref", name]).is_empty()
    }

    /// An unrelated operation that stays in the history after the one under test.
    async fn later_commit(&mut self) {
        self.write("later.txt", "later");
        self.stage("later.txt").await;
        self.commit("later work").await;
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

// ------------------------------------------------------------- branches

#[tokio::test]
async fn a_deleted_branch_comes_back_while_later_actions_stay() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.repo.git(&["branch", "feature"]);
        rig.repo.git(&["config", "branch.feature.remote", "origin"]);
        rig.repo
            .git(&["config", "branch.feature.merge", "refs/heads/feature"]);
        let tip = rig.rev("feature");
        rig.ok(OperationKind::BranchDelete, json!({ "name": "feature" }))
            .await;
        let id = rig.entry_id(OperationKind::BranchDelete).await;
        rig.later_commit().await;
        let main_after = rig.rev("main");

        let listed = rig.listing().await;
        let offer = &listed
            .undo
            .iter()
            .find(|e| e.summary.id == id)
            .unwrap()
            .restore;
        assert_eq!(offer.items[0].kind, RestoreItemKind::Branch, "{name}");

        let dry: RestoreOutcome = serde_json::from_value(
            rig.ok(
                OperationKind::JournalRestore,
                json!({ "id": id, "dry_run": true }),
            )
            .await,
        )
        .unwrap();
        assert!(!dry.applied && dry.new_entry.is_none(), "{name}");
        assert!(
            !rig.has_ref("refs/heads/feature"),
            "{name}: a dry run changes nothing"
        );

        let out = rig.restore(id, None).await;
        assert!(out.applied, "{name}");
        assert_eq!(out.refs, vec!["refs/heads/feature".to_string()], "{name}");
        assert_eq!(rig.rev("feature"), tip, "{name}");
        assert_eq!(
            rig.rev("main"),
            main_after,
            "{name}: later commit untouched"
        );
        assert_eq!(
            rig.repo.git(&["config", "branch.feature.remote"]),
            "origin",
            "{name}: the upstream came back"
        );

        // The restore is one undoable entry: Ctrl+Z deletes the branch again.
        let undone = rig.undo().await;
        assert_eq!(undone.entry.kind, OperationKind::JournalRestore, "{name}");
        assert!(!rig.has_ref("refs/heads/feature"), "{name}");
        assert_eq!(rig.rev("main"), main_after, "{name}");
        rig.redo().await;
        assert_eq!(rig.rev("feature"), tip, "{name}");
        // Everything is back: nothing more to restore.
        assert_eq!(
            rig.restore_refused(id, None).await,
            UndoRefusal::AlreadyRestored,
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_taken_branch_name_is_refused_with_a_suggestion_and_another_name_works() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.repo.git(&["branch", "feature"]);
        let tip = rig.rev("feature");
        rig.ok(OperationKind::BranchDelete, json!({ "name": "feature" }))
            .await;
        let id = rig.entry_id(OperationKind::BranchDelete).await;
        // The name is taken now, by something else.
        rig.write("n.txt", "n");
        rig.stage("n.txt").await;
        let other = rig.commit("n").await;
        rig.repo.git(&["branch", "feature", &other.to_hex()]);

        match rig.restore_refused(id, None).await {
            UndoRefusal::NameTaken { name: n, suggested } => {
                assert_eq!(n, "feature", "{name}");
                assert_eq!(suggested, "feature-restored", "{name}");
            }
            other => panic!("{name}: {other:?}"),
        }
        assert_eq!(rig.rev("feature"), other, "{name}: nothing was overwritten");

        rig.restore(id, Some("feature-restored")).await;
        assert_eq!(rig.rev("feature-restored"), tip, "{name}");
        assert_eq!(rig.rev("feature"), other, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_deleted_tag_comes_back_annotated_objects_included() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.repo.git(&["tag", "-a", "v1", "-m", "release one"]);
        let tag_object = rig.rev("refs/tags/v1");
        rig.ok(OperationKind::TagDelete, json!({ "name": "v1" }))
            .await;
        let id = rig.entry_id(OperationKind::TagDelete).await;
        rig.later_commit().await;
        rig.restore(id, None).await;
        assert_eq!(rig.rev("refs/tags/v1"), tag_object, "{name}");
        assert_eq!(rig.repo.git(&["cat-file", "-t", "v1"]), "tag", "{name}");
        rig.stop().await;
    }
}

// --------------------------------------------------------------- stashes

#[tokio::test]
async fn a_dropped_stash_goes_back_on_top() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.write("a.txt", "one\nchanged\n");
        rig.ok(
            OperationKind::StashPush,
            json!({ "message": "wip", "include_untracked": true, "paths": [] }),
        )
        .await;
        let commit = rig.repo.git(&["rev-parse", "stash@{0}"]);
        rig.ok(OperationKind::StashDrop, json!({ "id": 0 })).await;
        assert!(rig.repo.git(&["stash", "list"]).is_empty(), "{name}");
        let id = rig.entry_id(OperationKind::StashDrop).await;
        rig.later_commit().await;
        rig.restore(id, None).await;
        assert_eq!(rig.repo.git(&["rev-parse", "stash@{0}"]), commit, "{name}");
        assert_eq!(
            rig.restore_refused(id, None).await,
            UndoRefusal::AlreadyRestored,
            "{name}"
        );
        // Ctrl+Z drops it again.
        rig.undo().await;
        assert!(rig.repo.git(&["stash", "list"]).is_empty(), "{name}");
        rig.stop().await;
    }
}

// ----------------------------------------------------------------- files

#[tokio::test]
async fn a_discarded_file_is_written_back_in_place_when_nothing_touched_it() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.write("a.txt", "one\nedited\n");
        rig.ok(OperationKind::Discard, json!({ "paths": ["a.txt"] }))
            .await;
        assert_eq!(rig.read("a.txt"), "one\ntwo\n", "{name}");
        let id = rig.entry_id(OperationKind::Discard).await;
        rig.later_commit().await;

        let out = rig.restore(id, None).await;
        assert!(out.side_files.is_empty(), "{name}");
        assert_eq!(rig.read("a.txt"), "one\nedited\n", "{name}");
        assert!(!rig.path("a.txt.restored").exists(), "{name}");
        // Undoing the restore discards the edit again.
        rig.undo().await;
        assert_eq!(rig.read("a.txt"), "one\ntwo\n", "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_file_changed_since_is_never_overwritten_the_kept_bytes_go_next_to_it() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.write("a.txt", "one\nedited\n");
        rig.ok(OperationKind::Discard, json!({ "paths": ["a.txt"] }))
            .await;
        let id = rig.entry_id(OperationKind::Discard).await;
        // Written again since the discard (and a stale side file exists).
        rig.write("a.txt", "completely new\n");
        rig.write("a.txt.restored", "someone else's file\n");

        let out = rig.restore(id, None).await;
        assert_eq!(
            out.side_files,
            vec!["a.txt.restored.2".to_string()],
            "{name}"
        );
        assert_eq!(rig.read("a.txt"), "completely new\n", "{name}: untouched");
        assert_eq!(
            rig.read("a.txt.restored"),
            "someone else's file\n",
            "{name}"
        );
        assert_eq!(rig.read("a.txt.restored.2"), "one\nedited\n", "{name}");

        // Ctrl+Z removes the side file and nothing else.
        rig.undo().await;
        assert!(!rig.path("a.txt.restored.2").exists(), "{name}");
        assert_eq!(rig.read("a.txt"), "completely new\n", "{name}");
        assert_eq!(
            rig.read("a.txt.restored"),
            "someone else's file\n",
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_deleted_file_comes_back_and_a_recreated_one_gets_a_side_file() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.write("scratch.txt", "precious\n");
        rig.ok(
            OperationKind::DeleteFile,
            json!({ "paths": ["scratch.txt"] }),
        )
        .await;
        let id = rig.entry_id(OperationKind::DeleteFile).await;
        rig.later_commit().await;
        rig.restore(id, None).await;
        assert_eq!(rig.read("scratch.txt"), "precious\n", "{name}");
        // Deleted again by hand, then re-created different: restore again.
        rig.undo().await;
        assert!(!rig.path("scratch.txt").exists(), "{name}");
        rig.write("scratch.txt", "new life\n");
        let out = rig.restore(id, None).await;
        assert_eq!(
            out.side_files,
            vec!["scratch.txt.restored".to_string()],
            "{name}"
        );
        assert_eq!(rig.read("scratch.txt"), "new life\n", "{name}");
        assert_eq!(rig.read("scratch.txt.restored"), "precious\n", "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn an_ignore_line_is_re_added_without_replacing_the_file() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.ok(
            OperationKind::AddToGitignore,
            json!({ "pattern": "*.bin", "file": null }),
        )
        .await;
        let id = rig.entry_id(OperationKind::AddToGitignore).await;
        // The file was rewritten since: the line is gone, others are new.
        rig.write(".gitignore", "target/\n");
        rig.restore(id, None).await;
        assert_eq!(rig.read(".gitignore"), "target/\n*.bin\n", "{name}");
        assert_eq!(
            rig.restore_refused(id, None).await,
            UndoRefusal::AlreadyRestored,
            "{name}"
        );
        rig.stop().await;
    }
}

// --------------------------------------------------------------- remotes

#[tokio::test]
async fn a_removed_remote_comes_back_with_tracking_refs_and_upstreams() {
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
        let tracking = rig.rev("refs/remotes/origin/main");
        rig.ok(OperationKind::RemoteRemove, json!({ "name": "origin" }))
            .await;
        assert!(!rig.has_ref("refs/remotes/origin"), "{name}");
        let id = rig.entry_id(OperationKind::RemoteRemove).await;
        rig.later_commit().await;
        rig.restore(id, None).await;
        assert_eq!(rig.rev("refs/remotes/origin/main"), tracking, "{name}");
        assert!(rig.repo.git(&["remote"]).contains("origin"), "{name}");
        assert_eq!(
            rig.repo.git(&["config", "branch.main.remote"]),
            "origin",
            "{name}"
        );
        rig.stop().await;
    }
}

// ------------------------------------------------- history-moving operations

#[tokio::test]
async fn a_hard_reset_is_restored_as_a_branch_at_the_old_tip_never_moving_main() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let first = rig.rev("HEAD");
        rig.write("b.txt", "b");
        rig.stage("b.txt").await;
        let second = rig.commit("second").await;
        rig.ok(
            OperationKind::Reset,
            json!({ "target": first, "mode": "Hard" }),
        )
        .await;
        assert_eq!(rig.rev("main"), first, "{name}");
        let id = rig.entry_id(OperationKind::Reset).await;

        let out = rig.restore(id, None).await;
        let branch = &out.refs[0];
        assert!(
            branch.starts_with("refs/heads/restore/main-"),
            "{name}: {branch}"
        );
        assert_eq!(rig.rev(branch), second, "{name}");
        assert_eq!(
            rig.rev("main"),
            first,
            "{name}: the current branch did not move"
        );
        assert!(
            !rig.path("b.txt").exists(),
            "{name}: the working tree did not change"
        );

        // A name of the caller's choice.
        rig.undo().await;
        let out = rig.restore(id, Some("rescue")).await;
        assert_eq!(out.refs, vec!["refs/heads/rescue".to_string()], "{name}");
        assert_eq!(rig.rev("rescue"), second, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn an_amend_keeps_its_old_commit_reachable_through_a_restore_branch() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let original = rig.rev("HEAD");
        rig.ok(
            OperationKind::Commit,
            json!({ "message": "better", "opts": CommitOptions { amend: true, ..Default::default() } }),
        )
        .await;
        let id = rig.entry_id(OperationKind::Commit).await;
        rig.later_commit().await;
        let out = rig.restore(id, Some("before-amend")).await;
        assert_eq!(rig.rev(&out.refs[0]), original, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn what_cannot_be_restored_says_why() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.repo.git(&["branch", "other"]);
        rig.ok(
            OperationKind::Checkout,
            json!({ "target": CheckoutTarget::Branch("other".into()) }),
        )
        .await;
        let checkout = rig.entry_id(OperationKind::Checkout).await;
        rig.write("n.txt", "n");
        rig.stage("n.txt").await;
        let stage = rig.entry_id(OperationKind::StageFile).await;
        for (id, word) in [(checkout, "HEAD"), (stage, "index")] {
            match rig.restore_refused(id, None).await {
                UndoRefusal::NotRestorable { reason } => {
                    assert!(reason.contains(word), "{name}: {reason}")
                }
                other => panic!("{name}: {other:?}"),
            }
        }
        let listed = rig.listing().await;
        for e in &listed.undo {
            assert!(!e.restore.restorable(), "{name}: {:?}", e.summary.kind);
            assert!(e.restore.not_restorable.is_some(), "{name}");
        }
        // An entry that is not there.
        assert_eq!(
            rig.restore_refused(9999, None).await,
            UndoRefusal::NoSuchEntry { id: 9999 },
            "{name}"
        );
        // An undone entry is redone, not restored.
        rig.undo().await;
        match rig.restore_refused(stage, None).await {
            UndoRefusal::NotRestorable { reason } => assert!(reason.contains("redo"), "{name}"),
            other => panic!("{name}: {other:?}"),
        }
        rig.stop().await;
    }
}

// ------------------------------------------------------------------ reflog

#[tokio::test]
async fn the_reflog_read_shows_what_a_shell_did() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.repo
            .git(&["commit", "--allow-empty", "-m", "from a shell"]);
        let page: ReflogPage = serde_json::from_value(
            rig.ok(
                OperationKind::Reflog,
                json!({ "ref": "main", "skip": 0, "limit": 10 }),
            )
            .await,
        )
        .unwrap();
        assert_eq!(page.refname, "refs/heads/main", "{name}");
        assert_eq!(page.entries[0].message, "commit: from a shell", "{name}");
        assert_eq!(page.entries[0].new, rig.rev("HEAD"), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_branch_that_is_not_checked_out_is_restored_to_a_reflog_position_and_undone() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.repo.git(&["checkout", "-q", "-b", "work"]);
        rig.repo.git(&["commit", "--allow-empty", "-m", "w1"]);
        let w1 = rig.rev("work");
        rig.repo.git(&["commit", "--allow-empty", "-m", "w2"]);
        let w2 = rig.rev("work");
        rig.repo.git(&["checkout", "-q", "main"]);
        // A shell reset moved `work` back and the second commit seems gone.
        rig.repo
            .git(&["update-ref", "refs/heads/work", &w1.to_hex()]);

        rig.ok(
            OperationKind::ReflogRestore,
            json!({ "ref": "work", "oid": w2 }),
        )
        .await;
        assert_eq!(rig.rev("work"), w2, "{name}");
        let undone = rig.undo().await;
        assert_eq!(undone.entry.kind, OperationKind::ReflogRestore, "{name}");
        assert_eq!(rig.rev("work"), w1, "{name}");
        rig.redo().await;
        assert_eq!(rig.rev("work"), w2, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn the_checked_out_branch_keeps_a_dirty_working_tree_safe() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let first = rig.rev("HEAD");
        rig.write("b.txt", "b");
        rig.stage("b.txt").await;
        let second = rig.commit("second").await;
        // Clean: moves back, files follow, undo brings them back.
        rig.ok(
            OperationKind::ReflogRestore,
            json!({ "ref": "main", "oid": first }),
        )
        .await;
        assert_eq!(rig.rev("main"), first, "{name}");
        assert!(!rig.path("b.txt").exists(), "{name}");
        rig.undo().await;
        assert_eq!(rig.rev("main"), second, "{name}");
        assert!(rig.path("b.txt").exists(), "{name}");

        // Dirty: refuses and names the modes; nothing changed.
        rig.write("a.txt", "one\ndirty\n");
        let e = rig
            .call(
                OperationKind::ReflogRestore,
                json!({ "ref": "main", "oid": first }),
            )
            .await
            .unwrap_err();
        assert!(
            matches!(
                e.kind,
                Some(TypedError::UndoRefused(UndoRefusal::WouldOverwrite { .. }))
            ),
            "{name}: {e:?}"
        );
        assert_eq!(rig.rev("main"), second, "{name}");
        assert_eq!(rig.read("a.txt"), "one\ndirty\n", "{name}");

        // keep: the move happens and the edit stays.
        rig.ok(
            OperationKind::ReflogRestore,
            json!({ "ref": "main", "oid": first, "mode": "keep" }),
        )
        .await;
        assert_eq!(rig.rev("main"), first, "{name}");
        assert_eq!(rig.read("a.txt"), "one\ndirty\n", "{name}");
        // Undo asks first when local changes surround a move of files.
        rig.ok(OperationKind::Undo, json!({ "confirmed": true }))
            .await;
        assert_eq!(rig.rev("main"), second, "{name}");
        assert_eq!(rig.read("a.txt"), "one\ndirty\n", "{name}");

        // soft: a reset, journaled as one.
        rig.ok(
            OperationKind::ReflogRestore,
            json!({ "ref": "HEAD", "oid": first, "mode": "soft" }),
        )
        .await;
        assert_eq!(rig.rev("main"), first, "{name}");
        let undone = rig.undo().await;
        assert_eq!(undone.entry.kind, OperationKind::Reset, "{name}");
        assert_eq!(rig.rev("main"), second, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_detached_head_is_restored_to_a_reflog_position() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let first = rig.rev("HEAD");
        rig.write("b.txt", "b");
        rig.stage("b.txt").await;
        let second = rig.commit("second").await;
        rig.repo
            .git(&["checkout", "-q", "--detach", &second.to_hex()]);
        rig.ok(
            OperationKind::ReflogRestore,
            json!({ "ref": "HEAD", "oid": first }),
        )
        .await;
        assert_eq!(rig.rev("HEAD"), first, "{name}");
        assert!(
            rig.repo.git(&["branch", "--show-current"]).is_empty(),
            "{name}"
        );
        rig.undo().await;
        assert_eq!(rig.rev("HEAD"), second, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_blocked_undo_carries_the_reflog_position_that_restores_what_the_entry_expects() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.write("c.txt", "c");
        rig.stage("c.txt").await;
        let ours = rig.commit("ours").await;
        // A shell commit moves main past what Commit left.
        rig.repo
            .git(&["commit", "--allow-empty", "-m", "from a shell"]);
        let moved = rig.rev("main");
        assert_ne!(moved, ours, "{name}");

        let state = rig.state().await;
        let Some(UndoRefusal::RefMoved {
            name: refname,
            expected,
            restore,
            ..
        }) = state.blocked
        else {
            panic!("{name}: not blocked by a moved ref");
        };
        assert_eq!(refname, "refs/heads/main", "{name}");
        assert_eq!(expected, Some(ours), "{name}");
        let hint = restore.unwrap_or_else(|| panic!("{name}: no reflog hint"));
        assert_eq!(hint.oid, ours, "{name}");
        assert_eq!(hint.index, 1, "{name}: one move newer");
        assert!(
            hint.message.starts_with("commit: ours"),
            "{name}: {}",
            hint.message
        );

        // The refusal of the undo itself carries it too.
        let e = rig.call(OperationKind::Undo, json!({})).await.unwrap_err();
        match e.kind {
            Some(TypedError::UndoRefused(UndoRefusal::RefMoved { restore, .. })) => {
                assert_eq!(restore.map(|r| r.oid), Some(ours), "{name}")
            }
            other => panic!("{name}: {other:?}"),
        }

        // Following the offer: main is where Commit left it.
        rig.ok(
            OperationKind::ReflogRestore,
            json!({ "ref": "main", "oid": hint.oid }),
        )
        .await;
        assert_eq!(rig.rev("main"), ours, "{name}");
        let state = rig.state().await;
        assert_eq!(
            state.undo.unwrap().kind,
            OperationKind::ReflogRestore,
            "{name}"
        );
        let _ = (EntryState::Done, Direction::Undo);
        rig.stop().await;
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
