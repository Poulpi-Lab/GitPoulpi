//! `gitpoulpi undo` / `redo`, end to end through the real binary: the JSON
//! payloads, the exit codes (0 done, 2 refused), `--dry-run` and `--list`.
//! Each CLI command is its own short-lived embedded daemon, so these also
//! prove the journal persists between processes.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::{
    Direction, JournalListing, Oid, OperationKind, UndoOutcome, UndoRefusal,
};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().expect("temp config dir"))
        .path()
}

/// `(exit code, parsed stdout)` of `gitpoulpi --json ...`.
fn run(repo: &FixtureRepo, session: &str, args: &[&str]) -> (i32, Value) {
    run_depth(repo, session, "0", args)
}

/// `run` with `general.undo_depth` set through its environment variable
/// (the tests below want unlimited history unless they say otherwise).
fn run_depth(repo: &FixtureRepo, session: &str, depth: &str, args: &[&str]) -> (i32, Value) {
    run_with(repo, session, depth, "false", args)
}

/// `run_depth`, with `general.journal_outside` set too. The tests of a
/// blocked entry turn it off; `outside.rs` has it on.
fn run_with(
    repo: &FixtureRepo,
    session: &str,
    depth: &str,
    outside: &str,
    args: &[&str],
) -> (i32, Value) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .env("GITPOULPI_GENERAL_UNDO_DEPTH", depth)
        .env("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", outside)
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(args)
        .current_dir(repo.root())
        .output()
        .expect("failed to run gitpoulpi");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout}\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code().unwrap_or(-1), json)
}

fn ok(repo: &FixtureRepo, session: &str, args: &[&str]) -> Value {
    let (code, json) = run(repo, session, args);
    assert_eq!(code, 0, "`gitpoulpi {}` failed: {json}", args.join(" "));
    json
}

fn head(repo: &FixtureRepo) -> Oid {
    Oid::from_hex(&repo.git(&["rev-parse", "HEAD"])).unwrap()
}

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build()
}

#[test]
fn commit_undo_redo_round_trips_with_identical_oids() {
    let repo = fixture();
    let session = "undo-roundtrip";
    let parent = head(&repo);
    std::fs::write(repo.root().join("x.txt"), "x").unwrap();
    ok(&repo, session, &["stage", "--file", "x.txt"]);
    ok(&repo, session, &["commit", "-m", "x"]);
    let commit = head(&repo);
    assert_ne!(commit, parent);

    let undone: UndoOutcome = serde_json::from_value(ok(&repo, session, &["undo"])).unwrap();
    assert_eq!(undone.direction, Direction::Undo);
    assert!(undone.applied);
    assert_eq!(undone.entry.kind, OperationKind::Commit);
    assert_eq!(head(&repo), parent);
    assert_eq!(repo.git(&["status", "--porcelain"]), "A  x.txt");

    let redone: UndoOutcome = serde_json::from_value(ok(&repo, session, &["redo"])).unwrap();
    assert_eq!(redone.direction, Direction::Redo);
    assert_eq!(head(&repo), commit, "redo restores the very same commit");
}

#[test]
fn dry_run_changes_nothing_and_says_what_it_would_do() {
    let repo = fixture();
    let session = "undo-dry-run";
    std::fs::write(repo.root().join("x.txt"), "x").unwrap();
    ok(&repo, session, &["stage", "--file", "x.txt"]);
    ok(&repo, session, &["commit", "-m", "x"]);
    let commit = head(&repo);
    let journal = std::fs::read(repo.root().join(".git/gitpoulpi/journal.json")).unwrap();

    let outcome: UndoOutcome =
        serde_json::from_value(ok(&repo, session, &["undo", "--dry-run"])).unwrap();
    assert!(!outcome.applied);
    assert!(!outcome.steps.is_empty());
    assert_eq!(head(&repo), commit);
    assert_eq!(
        std::fs::read(repo.root().join(".git/gitpoulpi/journal.json")).unwrap(),
        journal
    );
    // It is still undoable afterwards.
    ok(&repo, session, &["undo"]);
}

#[test]
fn a_refused_undo_exits_2_with_the_typed_reason_and_changes_nothing() {
    let repo = fixture();
    let session = "undo-refused";
    let (code, json) = run(&repo, session, &["undo"]);
    assert_eq!(code, 2);
    let reason: UndoRefusal = serde_json::from_value(json["refused"]["reason"].clone()).unwrap();
    assert_eq!(reason, UndoRefusal::NothingToUndo);

    std::fs::write(repo.root().join("x.txt"), "x").unwrap();
    ok(&repo, session, &["stage", "--file", "x.txt"]);
    ok(&repo, session, &["commit", "-m", "x"]);
    repo.git(&["commit", "--allow-empty", "-m", "outside"]);
    let theirs = head(&repo);

    let (code, json) = run(&repo, session, &["undo"]);
    assert_eq!(code, 2);
    let reason: UndoRefusal = serde_json::from_value(json["refused"]["reason"].clone()).unwrap();
    assert!(matches!(reason, UndoRefusal::RefMoved { actual, .. } if actual == Some(theirs)));
    assert!(json["refused"]["message"]
        .as_str()
        .unwrap()
        .contains("outside GitPoulpi"));
    assert_eq!(head(&repo), theirs, "nothing changed");

    let (code, _) = run(&repo, session, &["redo"]);
    assert_eq!(code, 2, "nothing to redo is a refusal too");
}

#[test]
fn list_shows_the_journal_and_a_stale_expect_is_refused() {
    let repo = fixture();
    let session = "undo-list";
    let tip = head(&repo).to_hex();
    ok(&repo, session, &["branch", "create", "feature", &tip]);
    std::fs::write(repo.root().join("x.txt"), "x").unwrap();
    ok(&repo, session, &["stage", "--file", "x.txt"]);
    ok(&repo, session, &["commit", "-m", "x"]);

    let listing: JournalListing =
        serde_json::from_value(ok(&repo, session, &["undo", "--list"])).unwrap();
    // Staging is journaled too.
    assert_eq!(listing.undo.len(), 3);
    assert_eq!(listing.undo[0].summary.kind, OperationKind::BranchCreate);
    assert_eq!(listing.undo[1].summary.kind, OperationKind::StageFile);
    assert_eq!(listing.undo[2].summary.kind, OperationKind::Commit);
    assert!(listing.redo.is_empty());
    let (create_id, commit_id) = (listing.undo[0].summary.id, listing.undo[2].summary.id);

    let (code, json) = run(
        &repo,
        session,
        &["undo", "--expect", &create_id.to_string()],
    );
    assert_eq!(code, 2);
    let reason: UndoRefusal = serde_json::from_value(json["refused"]["reason"].clone()).unwrap();
    assert!(matches!(reason, UndoRefusal::StaleEntry { .. }));
    ok(
        &repo,
        session,
        &["undo", "--expect", &commit_id.to_string()],
    );

    let listing: JournalListing =
        serde_json::from_value(ok(&repo, session, &["undo", "--list"])).unwrap();
    assert_eq!((listing.undo.len(), listing.redo.len()), (2, 1));
}

#[test]
fn branch_delete_undo_restores_the_branch() {
    let repo = fixture();
    let session = "undo-branch";
    let tip = head(&repo);
    ok(
        &repo,
        session,
        &["branch", "create", "feature", &tip.to_hex()],
    );
    ok(&repo, session, &["branch", "delete", "feature"]);
    assert!(repo.git(&["for-each-ref", "refs/heads/feature"]).is_empty());
    ok(&repo, session, &["undo"]);
    assert_eq!(repo.git(&["rev-parse", "feature"]), tip.to_hex());
}

#[test]
fn the_default_depth_of_one_keeps_only_the_latest_operation() {
    let repo = fixture();
    let session = "undo-depth-one";
    let tip = head(&repo).to_hex();
    let go = |args: &[&str]| {
        let (code, json) = run_depth(&repo, session, "1", args);
        assert_eq!(code, 0, "{args:?}: {json}");
        json
    };
    go(&["branch", "create", "feature", &tip]);
    go(&["branch", "create", "other", &tip]);
    let listing: JournalListing = serde_json::from_value(go(&["undo", "--list"])).unwrap();
    assert_eq!(listing.undo.len(), 1, "depth 1 forgets the older entry");
    assert!(listing.undo[0].summary.label.contains("other"));
}

/// A commit already on its upstream is a risky undo: nothing happens
/// (exit 3, the prompt) until `--yes`.
#[test]
fn undoing_a_pushed_commit_needs_yes() {
    let repo = fixture();
    let session = "undo-pushed";
    let remote = tempfile::tempdir().unwrap();
    let bare = remote.path().join("origin.git");
    std::process::Command::new("git")
        .args(["init", "--bare", "-q"])
        .arg(&bare)
        .status()
        .unwrap();
    let branch = repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]);
    repo.git(&["remote", "add", "origin", &bare.to_string_lossy()]);
    repo.git(&["push", "-u", "origin", &branch]);
    std::fs::write(repo.root().join("x.txt"), "x").unwrap();
    ok(&repo, session, &["stage", "--file", "x.txt"]);
    ok(&repo, session, &["commit", "-m", "x"]);
    let commit = head(&repo);
    repo.git(&["push", "origin", &branch]);

    let (code, json) = run(&repo, session, &["undo"]);
    assert_eq!(code, 3, "{json}");
    assert!(json["needs_confirmation"]["prompt"]
        .as_str()
        .unwrap()
        .contains("already on origin/"));
    assert_eq!(head(&repo), commit, "nothing changed");

    let dry: UndoOutcome =
        serde_json::from_value(ok(&repo, session, &["undo", "--dry-run"])).unwrap();
    assert!(dry.needs_confirmation.is_some());

    ok(&repo, session, &["undo", "--yes"]);
    assert_ne!(head(&repo), commit);
}

#[test]
fn an_annotated_tag_delete_is_undone_with_the_same_tag_object_across_processes() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .annotated_tag("v1")
        .unwrap()
        .build();
    let session = "undo-tag";
    let object = repo.git(&["rev-parse", "refs/tags/v1"]);

    ok(&repo, session, &["tag", "delete", "v1"]);
    assert!(repo.git(&["for-each-ref", "refs/tags/v1"]).is_empty());
    // Nothing but the journal's pin keeps the tag object alive now.
    repo.git(&["gc", "-q", "--prune=now"]);

    let undone: UndoOutcome = serde_json::from_value(ok(&repo, session, &["undo"])).unwrap();
    assert_eq!(undone.entry.kind, OperationKind::TagDelete);
    assert_eq!(repo.git(&["rev-parse", "refs/tags/v1"]), object);
    assert_eq!(repo.git(&["cat-file", "-t", "refs/tags/v1"]), "tag");

    ok(&repo, session, &["redo"]);
    assert!(repo.git(&["for-each-ref", "refs/tags/v1"]).is_empty());
}

#[test]
fn a_branch_rename_is_undone_and_a_tag_moved_outside_is_refused_with_exit_2() {
    let repo = fixture();
    let session = "undo-rename";
    repo.git(&["branch", "feature"]);
    ok(&repo, session, &["branch", "rename", "feature", "renamed"]);
    ok(&repo, session, &["undo"]);
    assert!(!repo.git(&["for-each-ref", "refs/heads/feature"]).is_empty());
    assert!(repo.git(&["for-each-ref", "refs/heads/renamed"]).is_empty());
    ok(&repo, session, &["redo"]);
    ok(&repo, session, &["undo"]);

    // A tag created through GitPoulpi, then moved behind its back.
    ok(&repo, session, &["tag", "create", "v1"]);
    repo.git(&["commit", "--allow-empty", "-m", "outside"]);
    repo.git(&["tag", "-f", "v1", "HEAD"]);
    let (code, json) = run(&repo, session, &["undo"]);
    assert_eq!(code, 2);
    let reason: UndoRefusal = serde_json::from_value(json["refused"]["reason"].clone()).unwrap();
    assert!(
        matches!(&reason, UndoRefusal::RefMoved { name, .. } if name == "refs/tags/v1"),
        "{reason:?}"
    );
}

#[test]
fn a_fast_forward_of_the_checked_out_branch_is_undone_and_refused_over_a_local_edit() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .branch("feature")
        .unwrap()
        .commit_file("feature.txt", "from feature\n", "feature work")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    let session = "undo-ff";
    let base = head(&repo);
    ok(
        &repo,
        session,
        &["branch", "fast-forward", "main", "feature"],
    );
    assert!(repo.root().join("feature.txt").exists());

    std::fs::write(repo.root().join("feature.txt"), "my edit\n").unwrap();
    let (code, json) = run(&repo, session, &["undo"]);
    assert_eq!(code, 2);
    let reason: UndoRefusal = serde_json::from_value(json["refused"]["reason"].clone()).unwrap();
    assert!(
        matches!(reason, UndoRefusal::WouldOverwrite { .. }),
        "{reason:?}"
    );
    assert_eq!(
        std::fs::read_to_string(repo.root().join("feature.txt")).unwrap(),
        "my edit\n"
    );

    repo.git(&["checkout", "--", "feature.txt"]);
    ok(&repo, session, &["undo"]);
    assert_eq!(head(&repo), base);
    assert!(!repo.root().join("feature.txt").exists());
}

fn binary() -> Vec<u8> {
    let mut bytes: Vec<u8> = (0..=255u8).collect();
    bytes.extend_from_slice(b"\r\nCRLF\r\n\0tail");
    bytes
}

#[test]
fn a_discard_is_undone_byte_exact_redone_and_refused_after_an_outside_edit() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "first")
        .unwrap()
        .build();
    let session = "undo-discard";
    std::fs::write(repo.root().join("a.txt"), "one\nedited\n").unwrap();
    std::fs::write(repo.root().join("blob.bin"), binary()).unwrap();

    ok(&repo, session, &["discard", "--file", "blob.bin"]);
    ok(&repo, session, &["discard", "--file", "a.txt"]);
    assert!(!repo.root().join("blob.bin").exists());
    assert_eq!(std::fs::read(repo.root().join("a.txt")).unwrap(), b"one\n");

    // Last discard first: a.txt comes back.
    let out: UndoOutcome = serde_json::from_value(ok(&repo, session, &["undo"])).unwrap();
    assert_eq!(out.entry.kind, OperationKind::Discard);
    assert!(out.applied);
    assert_eq!(
        std::fs::read(repo.root().join("a.txt")).unwrap(),
        b"one\nedited\n"
    );
    // Then the untracked binary, byte for byte.
    ok(&repo, session, &["undo"]);
    assert_eq!(
        std::fs::read(repo.root().join("blob.bin")).unwrap(),
        binary()
    );

    // Redo discards it again.
    ok(&repo, session, &["redo"]);
    assert!(!repo.root().join("blob.bin").exists());
    ok(&repo, session, &["redo"]);
    assert_eq!(std::fs::read(repo.root().join("a.txt")).unwrap(), b"one\n");

    // Edited since the discard: refused (exit 2), nothing restored.
    std::fs::write(repo.root().join("a.txt"), "outside\n").unwrap();
    let (code, json) = run(&repo, session, &["undo"]);
    assert_eq!(code, 2);
    let reason: UndoRefusal = serde_json::from_value(json["refused"]["reason"].clone()).unwrap();
    assert!(matches!(reason, UndoRefusal::PathChanged { path } if path == "a.txt"));
    assert_eq!(
        std::fs::read(repo.root().join("a.txt")).unwrap(),
        b"outside\n"
    );
}

#[test]
fn stage_and_unstage_are_undone_and_refused_after_an_outside_index_change() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "first")
        .unwrap()
        .build();
    let session = "undo-stage";
    std::fs::write(repo.root().join("a.txt"), "one\nedited\n").unwrap();
    std::fs::write(repo.root().join("new.txt"), "new\n").unwrap();
    let clean = repo.git(&["ls-files", "-s"]);

    ok(&repo, session, &["stage", "--all"]);
    let staged = repo.git(&["ls-files", "-s"]);
    assert_ne!(staged, clean);
    ok(&repo, session, &["unstage", "--file", "new.txt"]);
    let partly = repo.git(&["ls-files", "-s"]);

    let out: UndoOutcome = serde_json::from_value(ok(&repo, session, &["undo"])).unwrap();
    assert_eq!(out.entry.kind, OperationKind::UnstageFile);
    assert_eq!(repo.git(&["ls-files", "-s"]), staged);
    ok(&repo, session, &["redo"]);
    assert_eq!(repo.git(&["ls-files", "-s"]), partly);
    ok(&repo, session, &["undo"]);
    ok(&repo, session, &["undo"]);
    assert_eq!(repo.git(&["ls-files", "-s"]), clean);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "one\nedited\n",
        "the working tree is never touched"
    );

    // An outside `git add` blocks the redo, with a typed reason.
    repo.git(&["add", "new.txt"]);
    let (code, json) = run(&repo, session, &["redo"]);
    assert_eq!(code, 2);
    let reason: UndoRefusal = serde_json::from_value(json["refused"]["reason"].clone()).unwrap();
    assert_eq!(reason, UndoRefusal::IndexChanged);
}

#[test]
fn rm_file_is_undone_byte_exact() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "first")
        .unwrap()
        .build();
    let session = "undo-rm";
    std::fs::write(repo.root().join("blob.bin"), binary()).unwrap();
    ok(&repo, session, &["rm-file", "blob.bin", "a.txt"]);
    assert!(!repo.root().join("blob.bin").exists() && !repo.root().join("a.txt").exists());
    ok(&repo, session, &["undo"]);
    assert_eq!(
        std::fs::read(repo.root().join("blob.bin")).unwrap(),
        binary()
    );
    assert_eq!(std::fs::read(repo.root().join("a.txt")).unwrap(), b"one\n");
}

#[test]
fn ignore_appends_to_gitignore_and_undo_puts_it_back() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "first")
        .unwrap()
        .build();
    let session = "undo-ignore";
    std::fs::write(repo.root().join("x.log"), "log").unwrap();
    let json = ok(&repo, session, &["ignore", "*.log"]);
    assert_eq!(json["ignored"]["added"], true);
    assert_eq!(
        std::fs::read_to_string(repo.root().join(".gitignore")).unwrap(),
        "*.log\n"
    );
    assert!(!repo.git(&["status", "--porcelain"]).contains("x.log"));
    ok(&repo, session, &["undo"]);
    assert!(!repo.root().join(".gitignore").exists());
    assert!(repo.git(&["status", "--porcelain"]).contains("x.log"));
    ok(&repo, session, &["redo"]);
    assert_eq!(
        std::fs::read_to_string(repo.root().join(".gitignore")).unwrap(),
        "*.log\n"
    );
    let again = ok(&repo, session, &["ignore", "*.log"]);
    assert_eq!(again["ignored"]["added"], false);
}

/// A real kill: a `post-commit` hook SIGKILLs the gitpoulpi process (the
/// embedded daemon) after git moved the branch and before the journal
/// entry was finished. The next process finds the entry `Interrupted`,
/// refuses to undo it, keeps its pins, and carries on working.
#[cfg(target_os = "linux")]
#[test]
fn a_daemon_killed_between_the_commit_and_its_journal_write_leaves_an_interrupted_entry() {
    use std::os::unix::fs::PermissionsExt;
    let repo = fixture();
    let session = "undo-kill";
    let parent = head(&repo);
    std::fs::write(repo.root().join("x.txt"), "x").unwrap();
    ok(&repo, session, &["stage", "--file", "x.txt"]);

    let hook = repo.root().join(".git/hooks/post-commit");
    std::fs::create_dir_all(hook.parent().unwrap()).unwrap();
    // $PPID is `git commit`; its parent is the gitpoulpi process.
    std::fs::write(
        &hook,
        "#!/bin/sh\nkill -9 \"$(awk '{print $4}' /proc/$PPID/stat)\"\nsleep 5\n",
    )
    .unwrap();
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .env("GITPOULPI_GENERAL_UNDO_DEPTH", "0")
        .env("GITPOULPI_GENERAL_DEFAULT_BACKEND", "system")
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["commit", "-m", "x"])
        .current_dir(repo.root())
        .output()
        .expect("failed to run gitpoulpi");
    assert!(
        !output.status.success() && output.status.code().is_none(),
        "the process was killed by a signal: {:?}",
        output.status
    );
    std::fs::remove_file(&hook).unwrap();
    let commit = head(&repo);
    assert_ne!(commit, parent, "git had already moved the branch");

    let listing: JournalListing =
        serde_json::from_value(ok(&repo, session, &["undo", "--list"])).unwrap();
    let kinds: Vec<_> = listing
        .undo
        .iter()
        .map(|e| (e.summary.kind, e.summary.state))
        .collect();
    assert_eq!(
        kinds,
        vec![
            (
                OperationKind::StageFile,
                gitpoulpi_core::domain::EntryState::Done
            ),
            (
                OperationKind::Commit,
                gitpoulpi_core::domain::EntryState::Interrupted
            ),
        ]
    );
    let (code, json) = run(&repo, session, &["undo"]);
    assert_eq!(code, 2, "{json}");
    let reason: UndoRefusal = serde_json::from_value(json["refused"]["reason"].clone()).unwrap();
    assert!(
        matches!(reason, UndoRefusal::Interrupted { .. }),
        "{reason:?}"
    );
    assert_eq!(head(&repo), commit, "nothing was moved");
    // The old tip is still there for a hand repair (pinned or reachable).
    repo.git(&["cat-file", "-e", &parent.to_string()]);
    // And the tool keeps working: a new operation is recorded as usual.
    std::fs::write(repo.root().join("y.txt"), "y").unwrap();
    ok(&repo, session, &["stage", "--file", "y.txt"]);
    let listing: JournalListing =
        serde_json::from_value(ok(&repo, session, &["undo", "--list"])).unwrap();
    assert_eq!(listing.undo.len(), 3);
}

#[test]
fn a_merge_commit_is_undone_and_redone_through_the_cli_across_processes() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .branch("feature")
        .unwrap()
        .checkout("feature")
        .unwrap()
        .commit_file("feature.txt", "f\n", "feature work")
        .unwrap()
        .checkout("main")
        .unwrap()
        .commit_file("main.txt", "m\n", "main work")
        .unwrap()
        .build();
    let session = "undo-merge";
    let before = head(&repo);
    ok(&repo, session, &["merge", "feature"]);
    let merged = head(&repo);
    assert_ne!(merged, before);
    assert!(repo.root().join("feature.txt").exists());

    let undone: UndoOutcome = serde_json::from_value(ok(&repo, session, &["undo"])).unwrap();
    assert_eq!(undone.entry.kind, OperationKind::Merge);
    assert_eq!(head(&repo), before);
    assert!(!repo.root().join("feature.txt").exists());

    ok(&repo, session, &["redo"]);
    assert_eq!(head(&repo), merged, "redo restores the very same merge");
    assert!(repo.root().join("feature.txt").exists());
}

#[test]
fn a_rebase_is_undone_and_redone_through_the_cli_and_refused_after_an_outside_commit() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .branch("feature")
        .unwrap()
        .checkout("feature")
        .unwrap()
        .commit_file("feature.txt", "f\n", "feature work")
        .unwrap()
        .checkout("main")
        .unwrap()
        .commit_file("main.txt", "m\n", "main work")
        .unwrap()
        .checkout("feature")
        .unwrap()
        .build();
    let session = "undo-rebase";
    let before = head(&repo);
    ok(&repo, session, &["rebase", "main"]);
    let rebased = head(&repo);
    assert_ne!(rebased, before);

    let undone: UndoOutcome = serde_json::from_value(ok(&repo, session, &["undo"])).unwrap();
    assert_eq!(undone.entry.kind, OperationKind::Rebase);
    assert_eq!(head(&repo), before);
    ok(&repo, session, &["redo"]);
    assert_eq!(head(&repo), rebased);

    repo.git(&["commit", "--allow-empty", "-m", "outside"]);
    let theirs = head(&repo);
    let (code, json) = run(&repo, session, &["undo"]);
    assert_eq!(code, 2);
    let reason: UndoRefusal = serde_json::from_value(json["refused"]["reason"].clone()).unwrap();
    assert!(matches!(reason, UndoRefusal::RefMoved { .. }), "{reason:?}");
    assert_eq!(head(&repo), theirs);
}

/// `general.undo_enabled = false`: `undo` / `redo` answer with a clear
/// error (exit 2, the refusal), and nothing was recorded to undo.
#[test]
fn undo_and_redo_say_so_when_undo_is_turned_off() {
    let repo = fixture();
    for verb in ["undo", "redo"] {
        let output = Command::cargo_bin("gitpoulpi")
            .unwrap()
            .env("GITPOULPI_CONFIG_DIR", config_dir())
            .env("GITPOULPI_GENERAL_UNDO_ENABLED", "false")
            .arg("--json")
            .args(["--session", "undo-off"])
            .args(["--repo", &repo.root().to_string_lossy()])
            .arg(verb)
            .current_dir(repo.root())
            .output()
            .expect("failed to run gitpoulpi");
        assert_ne!(output.status.code(), Some(0), "{verb}");
        let text = String::from_utf8_lossy(&output.stdout).into_owned()
            + &String::from_utf8_lossy(&output.stderr);
        assert!(
            text.contains("Undo is turned off (Settings > General)"),
            "{verb}: {text}"
        );
    }
}

#[test]
fn undo_forget_drops_a_blocked_top_entry_and_never_touches_the_repository() {
    let repo = fixture();
    let session = "undo-forget";
    // Not blocked yet: nothing to forget (refused, nothing dropped).
    std::fs::write(repo.root().join("x.txt"), "x").unwrap();
    ok(&repo, session, &["stage", "--file", "x.txt"]);
    ok(&repo, session, &["commit", "-m", "x"]);
    let (code, _) = run(&repo, session, &["undo-forget", "--yes"]);
    assert_ne!(code, 0, "a healthy top entry is not forgotten");

    repo.git(&["commit", "--allow-empty", "-m", "outside"]);
    let theirs = head(&repo);
    let listing = |repo: &FixtureRepo| -> JournalListing {
        serde_json::from_value(ok(repo, session, &["undo", "--list"])).unwrap()
    };
    let before = listing(&repo).undo.len();

    // Without --yes: asked, nothing dropped.
    let (code, json) = run(&repo, session, &["undo-forget"]);
    assert_eq!(code, 3);
    assert!(json["needs_confirmation"]["prompt"].is_string());
    assert_eq!(listing(&repo).undo.len(), before);
    // A dry run changes nothing either.
    let dry: UndoOutcome =
        serde_json::from_value(ok(&repo, session, &["undo-forget", "--dry-run"])).unwrap();
    assert!(!dry.applied);
    assert_eq!(listing(&repo).undo.len(), before);

    let done: UndoOutcome =
        serde_json::from_value(ok(&repo, session, &["undo-forget", "--yes"])).unwrap();
    assert!(done.applied);
    assert_eq!(done.entry.kind, OperationKind::Commit);
    assert_eq!(listing(&repo).undo.len(), before - 1);
    assert_eq!(head(&repo), theirs, "the repository is untouched");
}

#[test]
fn treat_writes_gitattributes_makes_the_diff_binary_and_undo_puts_it_back() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("notes.dat", "one\ntwo\n", "first")
        .unwrap()
        .build();
    let session = "undo-treat";
    std::fs::write(repo.root().join("notes.dat"), "one\nchanged\n").unwrap();
    assert!(repo.git(&["diff"]).contains("+changed"));

    let json = ok(&repo, session, &["treat", "binary", "notes.dat"]);
    assert_eq!(json["treated"]["file"], ".gitattributes");
    assert_eq!(
        json["treated"]["patterns"],
        serde_json::json!(["/notes.dat"])
    );
    assert_eq!(json["treated"]["changed"], true);
    assert_eq!(json["treated"]["effective"], true);
    assert_eq!(
        std::fs::read_to_string(repo.root().join(".gitattributes")).unwrap(),
        "/notes.dat binary\n"
    );
    assert!(repo.git(&["diff"]).contains("Binary files"));

    ok(&repo, session, &["undo"]);
    assert!(!repo.root().join(".gitattributes").exists());
    assert!(repo.git(&["diff"]).contains("+changed"));
    ok(&repo, session, &["redo"]);
    assert!(repo.git(&["diff"]).contains("Binary files"));

    // Flipping it rewrites the line instead of adding a second one.
    let back = ok(&repo, session, &["treat", "text", "notes.dat"]);
    assert_eq!(back["treated"]["effective"], true);
    assert_eq!(
        std::fs::read_to_string(repo.root().join(".gitattributes")).unwrap(),
        "/notes.dat text diff\n"
    );
}

#[test]
fn convert_rewrites_the_files_reports_the_skipped_and_one_undo_puts_them_all_back() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "first")
        .unwrap()
        .build();
    let session = "undo-convert";
    std::fs::write(repo.root().join("a.txt"), b"a1\r\na2\r\n").unwrap();
    std::fs::write(repo.root().join("b.txt"), b"caf\xe9\r\n").unwrap();
    std::fs::write(repo.root().join("c.bin"), b"\0\r\n").unwrap();

    let json = ok(
        &repo,
        session,
        &["convert", "--eol", "lf", "a.txt", "b.txt", "c.bin"],
    );
    assert_eq!(
        json["converted"]["converted"],
        serde_json::json!(["a.txt", "b.txt"])
    );
    assert_eq!(json["converted"]["skipped"][0][0], "c.bin");
    assert_eq!(
        std::fs::read(repo.root().join("a.txt")).unwrap(),
        b"a1\na2\n"
    );
    assert_eq!(
        std::fs::read(repo.root().join("b.txt")).unwrap(),
        b"caf\xe9\n"
    );

    let json = ok(&repo, session, &["convert", "--encoding", "UTF-8", "b.txt"]);
    assert_eq!(json["converted"]["converted"], serde_json::json!(["b.txt"]));
    assert_eq!(
        std::fs::read(repo.root().join("b.txt")).unwrap(),
        "caf\u{e9}\n".as_bytes()
    );

    ok(&repo, session, &["undo"]);
    assert_eq!(
        std::fs::read(repo.root().join("b.txt")).unwrap(),
        b"caf\xe9\n"
    );
    ok(&repo, session, &["undo"]);
    assert_eq!(
        std::fs::read(repo.root().join("a.txt")).unwrap(),
        b"a1\r\na2\r\n"
    );
    assert_eq!(
        std::fs::read(repo.root().join("b.txt")).unwrap(),
        b"caf\xe9\r\n"
    );

    // Indentation, and the refusals.
    std::fs::write(repo.root().join("a.txt"), b"\tx\n").unwrap();
    ok(
        &repo,
        session,
        &["convert", "--indent", "spaces:2", "a.txt"],
    );
    assert_eq!(std::fs::read(repo.root().join("a.txt")).unwrap(), b"  x\n");
    for bad in [
        &["convert", "--encoding", "Klingon", "a.txt"][..],
        &["convert", "a.txt"][..],
        &["convert", "--indent", "spaces:9", "a.txt"][..],
    ] {
        let output = Command::cargo_bin("gitpoulpi")
            .unwrap()
            .env("GITPOULPI_CONFIG_DIR", config_dir())
            .arg("--json")
            .args(["--session", session])
            .args(["--repo", &repo.root().to_string_lossy()])
            .args(bad)
            .current_dir(repo.root())
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "`gitpoulpi {}` must fail",
            bad.join(" ")
        );
    }
}

#[test]
fn the_per_clone_files_take_ignore_and_treat_rules_that_undo_does_not_track() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "first")
        .unwrap()
        .build();
    let session = "undo-info-files";
    std::fs::create_dir_all(repo.root().join("sub")).unwrap();
    std::fs::write(repo.root().join("sub/x.log"), "log").unwrap();
    std::fs::write(repo.root().join("a.dat"), "plain\n").unwrap();

    let json = ok(
        &repo,
        session,
        &["ignore", "/sub/x.log", "--file", ".git/info/exclude"],
    );
    assert_eq!(json["ignored"]["added"], true);
    assert!(!repo.root().join(".gitignore").exists());
    assert!(!repo.git(&["status", "--porcelain"]).contains("x.log"));

    let json = ok(
        &repo,
        session,
        &[
            "treat",
            "binary",
            "a.dat",
            "--extension",
            "--file",
            ".git/info/attributes",
        ],
    );
    assert_eq!(json["treated"]["file"], ".git/info/attributes");
    assert_eq!(json["treated"]["patterns"], serde_json::json!(["*.dat"]));
    assert_eq!(json["treated"]["effective"], true);
    assert!(!repo.root().join(".gitattributes").exists());

    // Nothing was journaled: undo has nothing to undo.
    let (code, _) = run(&repo, session, &["undo"]);
    assert_eq!(code, 2);

    // Bad targets are refused, not written.
    for args in [
        &["treat", "binary", "a.dat", "--file", "a.dat"][..],
        &["treat", "binary", "Makefile", "--extension"][..],
        &["ignore", "x", "--file", ".git/info/attributes"][..],
    ] {
        let (code, _) = run(&repo, session, args);
        assert_ne!(code, 0, "{args:?}");
    }
}

#[test]
fn a_stash_push_with_nothing_to_stash_says_so_and_records_nothing() {
    let repo = fixture();
    let json = ok(&repo, "undo-empty-stash", &["stash", "push"]);
    assert_eq!(json["stash"], Value::Null, "{json}");
    let listing: JournalListing =
        serde_json::from_value(ok(&repo, "undo-empty-stash", &["undo", "--list"])).unwrap();
    assert!(listing.undo.is_empty(), "{listing:?}");
    assert!(repo.git(&["stash", "list"]).is_empty());
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
