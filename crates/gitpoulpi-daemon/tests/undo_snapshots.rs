//! Undo/redo of the snapshot-backed operations through the daemon
//! (the undo-redo design):
//! discard (files, hunk, line, file checkout), patch apply, file delete,
//! and the ten stage/unstage operations. Every test runs against both
//! backends and covers undo, redo and the refusal after an outside
//! change; nothing a discard threw away may be lost.

use std::sync::Arc;

use gitpoulpi_core::domain::{
    CheckoutTarget, DiffTarget, JournalListing, LineOrigin, LineSelector, Oid, OperationKind,
    RepoId, StageTarget, UndoOutcome, UndoRefusal,
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
    backend: Arc<dyn GitBackend>,
    repo: FixtureRepo,
    id: RepoId,
    next: u64,
}

impl Rig {
    async fn on(backend: Arc<dyn GitBackend>, repo: FixtureRepo) -> Rig {
        // These tests are about a blocked entry: outside changes are not
        // recorded here (outside_journal.rs covers them recorded).
        std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
        let handle = Daemon::new("undo-snapshots-test")
            .spawn(backend.clone())
            .await
            .unwrap();
        let client = Client::connect(&handle.endpoint()).await.unwrap();
        let id = repo.repo_ref().id;
        Rig {
            handle: Some(handle),
            client,
            backend,
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
                session: "undo-snapshots-test".into(),
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

    async fn refused(&mut self, command: OperationKind) -> UndoRefusal {
        let e = self.call(command, json!({})).await.unwrap_err();
        match e.kind {
            Some(TypedError::UndoRefused(r)) => r,
            other => panic!("expected an undo refusal, got {other:?} ({})", e.message),
        }
    }

    async fn listing(&mut self) -> JournalListing {
        serde_json::from_value(self.ok(OperationKind::JournalList, json!({})).await).unwrap()
    }

    async fn discard(&mut self, paths: &[&str]) {
        self.ok(OperationKind::Discard, json!({ "paths": paths }))
            .await;
    }

    fn path(&self, name: &str) -> std::path::PathBuf {
        self.repo.root().join(name)
    }

    fn read(&self, name: &str) -> Vec<u8> {
        std::fs::read(self.path(name)).unwrap()
    }

    /// Everything the index and the working tree say, to compare states.
    fn state(&self) -> (String, String) {
        (
            self.repo.git(&["ls-files", "-s"]),
            self.repo.git(&["status", "--porcelain=v2", "-uall"]),
        )
    }

    fn index(&self) -> String {
        self.repo.git(&["ls-files", "-s"])
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

fn binary() -> Vec<u8> {
    let mut bytes: Vec<u8> = (0..=255u8).collect();
    bytes.extend_from_slice(b"\r\nCRLF\r\n\0tail");
    bytes
}

/// A repo with a tracked file that has a staged edit and a further
/// working-tree edit, plus an untracked binary file in a new directory.
fn dirty_repo() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\ntwo\n", "first")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("a.txt"), "one\nstaged\n").unwrap();
    repo.git(&["add", "a.txt"]);
    std::fs::write(repo.root().join("a.txt"), "one\nstaged\nunstaged\n").unwrap();
    std::fs::create_dir_all(repo.root().join("junk/deep")).unwrap();
    std::fs::write(repo.root().join("junk/deep/blob.bin"), binary()).unwrap();
    repo
}

// -------------------------------------------------------------- discard

#[tokio::test]
async fn discarding_a_modified_and_an_untracked_file_is_undone_byte_exact_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, dirty_repo()).await;
        let before = rig.state();
        let a_before = rig.read("a.txt");

        rig.discard(&["a.txt", "junk/deep/blob.bin"]).await;
        assert_eq!(rig.read("a.txt"), b"one\ntwo\n", "{name}");
        assert!(!rig.path("junk/deep/blob.bin").exists(), "{name}");
        let after = rig.state();
        assert_ne!(after, before);

        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::Discard, "{name}");
        assert!(out.applied, "{name}");
        assert_eq!(rig.read("a.txt"), a_before, "{name}: the working copy");
        assert_eq!(rig.read("junk/deep/blob.bin"), binary(), "{name}: binary");
        assert_eq!(rig.state(), before, "{name}: index and status as before");
        assert_eq!(rig.repo.git(&["show", ":a.txt"]), "one\nstaged", "{name}");

        // Redo discards again, without re-running anything.
        let out = rig.redo().await;
        assert!(out.applied, "{name}");
        assert_eq!(rig.state(), after, "{name}: redo");
        assert_eq!(rig.read("a.txt"), b"one\ntwo\n", "{name}");
        assert!(!rig.path("junk/deep/blob.bin").exists(), "{name}");

        // And the redo is undoable again, still byte-exact.
        rig.undo().await;
        assert_eq!(rig.read("junk/deep/blob.bin"), binary(), "{name}");
        assert_eq!(rig.state(), before, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_discard_undo_refuses_after_an_outside_edit_and_changes_nothing() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, dirty_repo()).await;
        let before = rig.state();
        rig.discard(&["a.txt", "junk/deep/blob.bin"]).await;

        // The shell writes the discarded untracked path again.
        std::fs::create_dir_all(rig.path("junk/deep")).unwrap();
        std::fs::write(rig.path("junk/deep/blob.bin"), b"someone else").unwrap();
        let reason = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(&reason, UndoRefusal::PathChanged { path } if path == "junk/deep/blob.bin"),
            "{name}: {reason:?}"
        );
        assert_eq!(rig.read("junk/deep/blob.bin"), b"someone else", "{name}");
        assert_eq!(rig.read("a.txt"), b"one\ntwo\n", "{name}: nothing restored");

        // Putting it back as the discard left it makes the undo valid.
        std::fs::remove_file(rig.path("junk/deep/blob.bin")).unwrap();
        rig.undo().await;
        assert_eq!(rig.state(), before, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_tracked_edit_after_the_discard_blocks_the_undo() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, dirty_repo()).await;
        rig.discard(&["a.txt"]).await;
        std::fs::write(rig.path("a.txt"), "edited after the discard\n").unwrap();
        assert!(
            matches!(
                rig.refused(OperationKind::Undo).await,
                UndoRefusal::PathChanged { .. }
            ),
            "{name}"
        );
        assert_eq!(rig.read("a.txt"), b"edited after the discard\n", "{name}");
        rig.stop().await;
    }
}

fn two_hunk_repo() -> (FixtureRepo, String) {
    let original: String = (1..=40).map(|i| format!("line {i}\n")).collect();
    let edited = original
        .replace("line 3\n", "line three\n")
        .replace("line 38\n", "line thirty-eight\n");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("multi.txt", &original, "multi")
        .unwrap()
        .write_worktree("multi.txt", edited.as_bytes())
        .unwrap()
        .build();
    (repo, edited)
}

fn hunks(rig: &Rig) -> Vec<gitpoulpi_core::domain::Hunk> {
    rig.backend
        .diff(&rig.repo.repo_ref(), DiffTarget::WorkingTreeVsIndex)
        .unwrap()
        .into_iter()
        .find(|f| f.path.to_string_lossy() == "multi.txt")
        .map(|f| f.hunks)
        .unwrap()
}

#[tokio::test]
async fn a_discarded_hunk_is_undone_to_the_whole_file_and_redone() {
    for (name, backend) in backends() {
        let (repo, edited) = two_hunk_repo();
        let mut rig = Rig::on(backend, repo).await;
        let first = hunks(&rig)[0].id;
        rig.ok(
            OperationKind::DiscardHunk,
            json!({ "target": gitpoulpi_core::domain::DiscardTarget::Hunk("multi.txt".into(), first) }),
        )
        .await;
        let discarded = rig.read("multi.txt");
        assert_ne!(discarded, edited.as_bytes(), "{name}");

        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::DiscardHunk, "{name}");
        assert_eq!(rig.read("multi.txt"), edited.as_bytes(), "{name}");
        rig.redo().await;
        assert_eq!(rig.read("multi.txt"), discarded, "{name}");
        rig.undo().await;
        assert_eq!(rig.read("multi.txt"), edited.as_bytes(), "{name}");

        // An outside edit blocks it.
        rig.redo().await;
        std::fs::write(rig.path("multi.txt"), "outside\n").unwrap();
        assert!(
            matches!(
                rig.refused(OperationKind::Undo).await,
                UndoRefusal::PathChanged { .. }
            ),
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn discarded_lines_are_undone_and_redone() {
    for (name, backend) in backends() {
        let (repo, edited) = two_hunk_repo();
        let mut rig = Rig::on(backend, repo).await;
        let hunk = hunks(&rig).remove(0);
        let added = hunk
            .lines
            .iter()
            .position(|l| l.origin == LineOrigin::Add)
            .unwrap() as u32;
        rig.ok(
            OperationKind::DiscardLine,
            json!({ "target": gitpoulpi_core::domain::DiscardTarget::Line(
                "multi.txt".into(),
                LineSelector { hunk: hunk.id, line_indices: vec![added] },
            ) }),
        )
        .await;
        let discarded = rig.read("multi.txt");
        assert_ne!(discarded, edited.as_bytes(), "{name}");
        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::DiscardLine, "{name}");
        assert_eq!(rig.read("multi.txt"), edited.as_bytes(), "{name}");
        rig.redo().await;
        assert_eq!(rig.read("multi.txt"), discarded, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_file_checkout_is_undone_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, dirty_repo()).await;
        let head = Oid::from_hex(&rig.repo.git(&["rev-parse", "HEAD"])).unwrap();
        let before = rig.state();
        rig.ok(
            OperationKind::Checkout,
            json!({ "target": CheckoutTarget::File { path: "a.txt".into(), source: head } }),
        )
        .await;
        assert_eq!(rig.read("a.txt"), b"one\ntwo\n", "{name}");
        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::Checkout, "{name}");
        assert_eq!(rig.state(), before, "{name}");
        assert_eq!(rig.read("a.txt"), b"one\nstaged\nunstaged\n", "{name}");
        rig.redo().await;
        assert_eq!(rig.read("a.txt"), b"one\ntwo\n", "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn restoring_several_files_from_a_commit_is_one_entry_undone_and_redone() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", "a1\n", "first")
            .unwrap()
            .commit_file("b.txt", "b1\n", "second")
            .unwrap()
            .build();
        let old = Oid::from_hex(&repo.git(&["rev-parse", "HEAD"])).unwrap();
        std::fs::write(repo.root().join("a.txt"), "a2\n").unwrap();
        std::fs::write(repo.root().join("b.txt"), "b2\n").unwrap();
        repo.git(&["commit", "-qam", "third"]);
        std::fs::write(repo.root().join("a.txt"), "a3 dirty\n").unwrap();
        let mut rig = Rig::on(backend, repo).await;
        let before = rig.state();
        rig.ok(
            OperationKind::Checkout,
            json!({ "target": CheckoutTarget::Files {
                paths: vec!["a.txt".into(), "b.txt".into()],
                source: old,
            } }),
        )
        .await;
        assert_eq!(rig.read("a.txt"), b"a1\n", "{name}");
        assert_eq!(rig.read("b.txt"), b"b1\n", "{name}");
        let out = rig.undo().await;
        assert_eq!(out.entry.label, "Restore 2 files", "{name}");
        assert_eq!(
            rig.read("a.txt"),
            b"a3 dirty\n",
            "{name}: the dirty copy is back"
        );
        assert_eq!(rig.read("b.txt"), b"b2\n", "{name}");
        assert_eq!(rig.state(), before, "{name}");
        rig.redo().await;
        assert_eq!(rig.read("b.txt"), b"b1\n", "{name}");
        rig.stop().await;
    }
}

// ----------------------------------------------------- patch, delete file

#[tokio::test]
async fn a_patch_apply_is_undone_and_redone_including_a_new_file() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("keep.txt", "keep\n", "first")
            .unwrap()
            .build();
        std::fs::write(repo.root().join("keep.txt"), "keep\nplus\n").unwrap();
        std::fs::write(repo.root().join("added.txt"), "new\n").unwrap();
        repo.git(&["add", "-A"]);
        let patch = repo.git(&["diff", "--cached", "--binary"]);
        repo.git(&["reset", "-q", "--hard"]);
        let patch_path = repo.root().join(".git").join("s4.patch");
        std::fs::write(&patch_path, format!("{patch}\n")).unwrap();
        let mut rig = Rig::on(backend, repo).await;
        let clean = rig.state();

        rig.ok(OperationKind::PatchApply, json!({ "patch": patch_path }))
            .await;
        assert_eq!(rig.read("keep.txt"), b"keep\nplus\n", "{name}");
        assert!(rig.path("added.txt").exists(), "{name}");
        let applied = rig.state();

        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::PatchApply, "{name}");
        assert_eq!(rig.read("keep.txt"), b"keep\n", "{name}");
        assert!(!rig.path("added.txt").exists(), "{name}");
        assert_eq!(rig.state(), clean, "{name}");
        rig.redo().await;
        assert_eq!(rig.state(), applied, "{name}");

        std::fs::write(rig.path("added.txt"), "edited\n").unwrap();
        assert!(
            matches!(
                rig.refused(OperationKind::Undo).await,
                UndoRefusal::PathChanged { .. }
            ),
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn deleting_files_is_undone_byte_exact_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, dirty_repo()).await;
        let before = rig.state();
        rig.ok(
            OperationKind::DeleteFile,
            json!({ "paths": ["a.txt", "junk"] }),
        )
        .await;
        assert!(!rig.path("a.txt").exists() && !rig.path("junk/deep/blob.bin").exists());
        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::DeleteFile, "{name}");
        assert_eq!(rig.read("junk/deep/blob.bin"), binary(), "{name}");
        assert_eq!(rig.read("a.txt"), b"one\nstaged\nunstaged\n", "{name}");
        assert_eq!(rig.state(), before, "{name}");
        rig.redo().await;
        assert!(!rig.path("a.txt").exists(), "{name}");
        assert!(!rig.path("junk/deep/blob.bin").exists(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn converting_files_is_one_entry_undone_byte_exact_and_redone() {
    use gitpoulpi_core::textconv::{Charset, ConvertOp, ConvertReport, Eol};
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, dirty_repo()).await;
        std::fs::write(rig.path("a.txt"), b"caf\xe9\r\nline\r\n").unwrap();
        std::fs::create_dir_all(rig.path("src")).unwrap();
        std::fs::write(rig.path("src/b.txt"), b"x\r\ny\r\n").unwrap();
        let before = rig.state();
        let entries_before = rig.listing().await.undo.len();
        let value = rig
            .ok(
                OperationKind::ConvertFiles,
                json!({
                    "paths": ["a.txt", "src/b.txt", "junk/deep/blob.bin", "nope.txt"],
                    "op": ConvertOp::Eol(Eol::Lf),
                }),
            )
            .await;
        let report: ConvertReport = serde_json::from_value(value).unwrap();
        assert_eq!(report.converted, vec!["a.txt", "src/b.txt"], "{name}");
        assert_eq!(report.skipped.len(), 2, "binary and missing: {name}");
        assert_eq!(rig.read("a.txt"), b"caf\xe9\nline\n", "{name}");
        assert_eq!(rig.read("src/b.txt"), b"x\ny\n", "{name}");
        // A second step on the same files, to be sure undo takes the last one.
        rig.ok(
            OperationKind::ConvertFiles,
            json!({ "paths": ["a.txt"], "op": ConvertOp::Encoding(Charset::Utf8) }),
        )
        .await;
        assert_eq!(rig.read("a.txt"), "caf\u{e9}\nline\n".as_bytes(), "{name}");
        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::ConvertFiles, "{name}");
        assert_eq!(rig.read("a.txt"), b"caf\xe9\nline\n", "{name}");
        let out = rig.undo().await;
        assert_eq!(out.entry.label, "Convert 4 files", "{name}");
        assert_eq!(rig.read("a.txt"), b"caf\xe9\r\nline\r\n", "{name}");
        assert_eq!(rig.read("src/b.txt"), b"x\r\ny\r\n", "{name}");
        assert_eq!(rig.state().0, before.0, "{name}");
        assert_eq!(rig.listing().await.undo.len(), entries_before, "{name}");
        rig.redo().await;
        assert_eq!(rig.read("src/b.txt"), b"x\ny\n", "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn ignoring_is_undone_redone_and_refused_once_the_file_changed() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, dirty_repo()).await;
        // No .gitignore yet: undo removes the file it created.
        assert!(!rig.path(".gitignore").exists());
        rig.ok(
            OperationKind::AddToGitignore,
            json!({ "pattern": "*.bin", "file": null }),
        )
        .await;
        assert_eq!(rig.read(".gitignore"), b"*.bin\n", "{name}");
        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::AddToGitignore, "{name}");
        assert!(!rig.path(".gitignore").exists(), "{name}");
        rig.redo().await;
        assert_eq!(rig.read(".gitignore"), b"*.bin\n", "{name}");

        // An existing file comes back byte for byte.
        rig.undo().await;
        std::fs::write(rig.path(".gitignore"), "target").unwrap();
        rig.ok(
            OperationKind::AddToGitignore,
            json!({ "pattern": "/x", "file": null }),
        )
        .await;
        assert_eq!(rig.read(".gitignore"), b"target\n/x\n", "{name}");
        // A repeated pattern changes nothing and records nothing.
        rig.ok(
            OperationKind::AddToGitignore,
            json!({ "pattern": "/x", "file": null }),
        )
        .await;
        rig.undo().await;
        assert_eq!(rig.read(".gitignore"), b"target", "{name}");
        rig.redo().await;

        // Edited since: undo refuses and leaves the edit alone.
        std::fs::write(rig.path(".gitignore"), "mine\n").unwrap();
        assert!(
            matches!(
                rig.refused(OperationKind::Undo).await,
                UndoRefusal::PathChanged { .. }
            ),
            "{name}"
        );
        assert_eq!(rig.read(".gitignore"), b"mine\n", "{name}");
        rig.stop().await;
    }
}

#[cfg(unix)]
#[tokio::test]
async fn a_discarded_executable_and_symlink_come_back_exactly() {
    use std::os::unix::fs::PermissionsExt;
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .build();
        std::fs::write(repo.root().join("run.sh"), "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(
            repo.root().join("run.sh"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        std::os::unix::fs::symlink("run.sh", repo.root().join("link")).unwrap();
        let mut rig = Rig::on(backend, repo).await;
        rig.discard(&["run.sh", "link"]).await;
        assert!(!rig.path("run.sh").exists() && rig.path("link").symlink_metadata().is_err());
        rig.undo().await;
        let mode = std::fs::metadata(rig.path("run.sh"))
            .unwrap()
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0, "{name}: executable bit");
        assert_eq!(
            std::fs::read_link(rig.path("link")).unwrap(),
            std::path::PathBuf::from("run.sh"),
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn nothing_discarded_is_lost_it_is_pinned_and_the_namespace_stays_hidden() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, dirty_repo()).await;
        rig.discard(&["a.txt", "junk/deep/blob.bin"]).await;
        let pins = rig.repo.git(&["for-each-ref", "refs/gitpoulpi/undo/"]);
        assert!(
            pins.lines().count() >= 2,
            "{name}: before and after: {pins}"
        );
        // Every pinned object exists.
        for line in pins.lines() {
            let oid = line.split_whitespace().next().unwrap();
            rig.repo.git(&["cat-file", "-e", oid]);
        }
        // Tree pins must not trip the listings.
        rig.ok(OperationKind::Refs, json!({})).await;
        let opts = gitpoulpi_core::domain::LogOptions {
            all_refs: true,
            ..Default::default()
        };
        rig.ok(OperationKind::Log, json!({ "opts": opts })).await;
        rig.stop().await;
    }
}

// ------------------------------------------------------------- staging

fn staging_repo() -> FixtureRepo {
    let original: String = (1..=40).map(|i| format!("line {i}\n")).collect();
    let edited = original
        .replace("line 3\n", "line three\n")
        .replace("line 38\n", "line thirty-eight\n");
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("multi.txt", &original, "multi")
        .unwrap()
        .commit_file("sub/b.txt", "b\n", "b")
        .unwrap()
        .write_worktree("multi.txt", edited.as_bytes())
        .unwrap()
        .write_worktree("sub/b.txt", b"b changed\n")
        .unwrap()
        .write_worktree("sub/new.txt", b"brand new\n")
        .unwrap()
        .build()
}

async fn stage_roundtrip(rig: &mut Rig, name: &str, command: OperationKind, target: StageTarget) {
    let before = rig.index();
    rig.ok(command, json!({ "target": target })).await;
    let after = rig.index();
    assert_ne!(before, after, "{name}: {command:?} changed the index");
    let work = rig.state().1;

    let out = rig.undo().await;
    assert_eq!(out.entry.kind, command, "{name}");
    assert_eq!(rig.index(), before, "{name}: {command:?} undone");
    let out = rig.redo().await;
    assert!(out.applied, "{name}");
    assert_eq!(rig.index(), after, "{name}: {command:?} redone");
    assert_eq!(rig.state().1, work, "{name}");
    // And undone again, for the next case to start from the same index.
    rig.undo().await;
    assert_eq!(rig.index(), before, "{name}");
}

#[tokio::test]
async fn every_staging_operation_is_undone_and_redone_exactly() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, staging_repo()).await;
        let first = hunks(&rig)[0].id;
        let hunk = hunks(&rig).remove(0);
        let added = hunk
            .lines
            .iter()
            .position(|l| l.origin == LineOrigin::Add)
            .unwrap() as u32;
        let line = StageTarget::Line(
            "multi.txt".into(),
            LineSelector {
                hunk: first,
                line_indices: vec![added],
            },
        );
        use OperationKind::*;
        stage_roundtrip(
            &mut rig,
            name,
            StageFile,
            StageTarget::File("sub/new.txt".into()),
        )
        .await;
        stage_roundtrip(
            &mut rig,
            name,
            StageFolder,
            StageTarget::Folder("sub".into()),
        )
        .await;
        stage_roundtrip(&mut rig, name, StageAll, StageTarget::All).await;
        stage_roundtrip(
            &mut rig,
            name,
            StageHunk,
            StageTarget::Hunk("multi.txt".into(), first),
        )
        .await;
        stage_roundtrip(&mut rig, name, StageLine, line.clone()).await;

        // The unstage side starts from staged changes.
        rig.repo.git(&["add", "-A"]);
        let hunk_target = StageTarget::Hunk("multi.txt".into(), first);
        // A partial unstage needs a hunk id from the staged diff.
        let staged = rig
            .backend
            .diff(&rig.repo.repo_ref(), DiffTarget::IndexVsHead)
            .unwrap();
        let multi = staged
            .iter()
            .find(|f| f.path.to_string_lossy() == "multi.txt")
            .unwrap();
        let staged_hunk = &multi.hunks[0];
        let staged_added = staged_hunk
            .lines
            .iter()
            .position(|l| l.origin == LineOrigin::Add)
            .unwrap() as u32;
        let _ = hunk_target;
        stage_roundtrip(
            &mut rig,
            name,
            UnstageFile,
            StageTarget::File("sub/new.txt".into()),
        )
        .await;
        stage_roundtrip(
            &mut rig,
            name,
            UnstageFolder,
            StageTarget::Folder("sub".into()),
        )
        .await;
        stage_roundtrip(&mut rig, name, UnstageAll, StageTarget::All).await;
        stage_roundtrip(
            &mut rig,
            name,
            UnstageHunk,
            StageTarget::Hunk("multi.txt".into(), staged_hunk.id),
        )
        .await;
        stage_roundtrip(
            &mut rig,
            name,
            UnstageLine,
            StageTarget::Line(
                "multi.txt".into(),
                LineSelector {
                    hunk: staged_hunk.id,
                    line_indices: vec![staged_added],
                },
            ),
        )
        .await;
        rig.stop().await;
    }
}

#[tokio::test]
async fn consecutive_staging_operations_are_separate_entries_undone_in_order() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, staging_repo()).await;
        let clean = rig.index();
        rig.ok(
            OperationKind::StageFile,
            json!({ "target": StageTarget::File("sub/new.txt".into()) }),
        )
        .await;
        let one = rig.index();
        rig.ok(
            OperationKind::StageFile,
            json!({ "target": StageTarget::File("sub/b.txt".into()) }),
        )
        .await;
        assert_eq!(rig.listing().await.undo.len(), 2, "{name}");

        rig.undo().await;
        assert_eq!(rig.index(), one, "{name}");
        rig.undo().await;
        assert_eq!(rig.index(), clean, "{name}");
        rig.redo().await;
        rig.redo().await;
        assert_ne!(rig.index(), one, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_staging_undo_refuses_after_an_outside_index_change_and_recovers() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, staging_repo()).await;
        let clean = rig.index();
        rig.ok(
            OperationKind::StageFile,
            json!({ "target": StageTarget::File("sub/new.txt".into()) }),
        )
        .await;
        let staged = rig.index();

        rig.repo.git(&["add", "multi.txt"]);
        let outside = rig.index();
        assert_eq!(
            rig.refused(OperationKind::Undo).await,
            UndoRefusal::IndexChanged
        );
        assert_eq!(rig.index(), outside, "{name}: nothing changed");

        // Putting the index back makes the entry valid again.
        rig.repo.git(&["reset", "-q", "multi.txt"]);
        assert_eq!(rig.index(), staged, "{name}");
        rig.undo().await;
        assert_eq!(rig.index(), clean, "{name}");

        // A redo is refused the same way.
        rig.repo.git(&["add", "multi.txt"]);
        assert_eq!(
            rig.refused(OperationKind::Redo).await,
            UndoRefusal::IndexChanged
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_staging_undo_survives_a_daemon_restart_and_the_working_tree_is_untouched() {
    for (name, backend) in backends() {
        let repo = staging_repo();
        let root = repo.root().to_path_buf();
        let mut rig = Rig::on(backend.clone(), repo).await;
        let clean = rig.index();
        rig.ok(
            OperationKind::StageAll,
            json!({ "target": StageTarget::All }),
        )
        .await;
        let work = std::fs::read(root.join("multi.txt")).unwrap();
        // A second daemon on the same repository reads the same journal.
        let repo2 = rig.repo.repo_ref();
        let handle = Daemon::new("undo-snapshots-test-2")
            .spawn(backend.clone())
            .await
            .unwrap();
        let mut client = Client::connect(&handle.endpoint()).await.unwrap();
        let resp = client
            .call(&Request {
                id: 1,
                session: "undo-snapshots-test-2".into(),
                repo: Some(repo2.id.clone()),
                command: OperationKind::Undo,
                args: json!({}),
            })
            .await
            .unwrap();
        resp.result
            .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        assert_eq!(rig.index(), clean, "{name}");
        assert_eq!(
            std::fs::read(root.join("multi.txt")).unwrap(),
            work,
            "{name}"
        );
        handle.shutdown().await.unwrap();
        rig.stop().await;
    }
}

#[tokio::test]
async fn staging_that_changes_nothing_records_nothing() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, staging_repo()).await;
        rig.ok(
            OperationKind::UnstageFile,
            json!({ "target": StageTarget::File("multi.txt".into()) }),
        )
        .await;
        assert!(rig.listing().await.undo.is_empty(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn treating_a_file_as_binary_is_undone_and_redone_and_the_per_clone_files_are_not_journaled()
{
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, dirty_repo()).await;
        assert!(!rig.path(".gitattributes").exists());
        let done = rig
            .ok(
                OperationKind::GitattributesTreat,
                json!({ "paths": ["a.txt"], "scope": "File", "treat": "Binary", "file": null }),
            )
            .await;
        assert_eq!(done["effective"], json!(true), "{name}");
        assert_eq!(rig.read(".gitattributes"), b"/a.txt binary\n", "{name}");
        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::GitattributesTreat, "{name}");
        assert!(!rig.path(".gitattributes").exists(), "{name}");
        rig.redo().await;
        assert_eq!(rig.read(".gitattributes"), b"/a.txt binary\n", "{name}");

        // Flipping rewrites the line; undo brings the first one back.
        rig.ok(
            OperationKind::GitattributesTreat,
            json!({ "paths": ["a.txt"], "scope": "File", "treat": "Text", "file": null }),
        )
        .await;
        assert_eq!(rig.read(".gitattributes"), b"/a.txt text diff\n", "{name}");
        rig.undo().await;
        assert_eq!(rig.read(".gitattributes"), b"/a.txt binary\n", "{name}");
        rig.stop().await;

        // The per-clone files are outside the working tree: written, but
        // there is nothing to undo.
        let mut rig = Rig::on(
            backends().into_iter().find(|(n, _)| *n == name).unwrap().1,
            dirty_repo(),
        )
        .await;
        rig.ok(
            OperationKind::AddToGitignore,
            json!({ "pattern": "*.bin", "file": ".git/info/exclude" }),
        )
        .await;
        rig.ok(
            OperationKind::GitattributesTreat,
            json!({ "paths": ["a.txt"], "scope": "File", "treat": "Binary", "file": ".git/info/attributes" }),
        )
        .await;
        assert!(
            matches!(
                rig.refused(OperationKind::Undo).await,
                UndoRefusal::NothingToUndo
            ),
            "{name}"
        );
        assert!(!rig.path(".gitignore").exists(), "{name}");
        rig.stop().await;
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
