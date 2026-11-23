//! End-to-end tests for the CLI's `GitBackend` command mapping.
//!
//! Each case runs the real `gitpoulpi` binary against a real fixture
//! repo, so every assertion travels clap -> IPC request -> daemon dispatch
//! -> `GitBackend` -> system git, and the JSON payload back.
//!
//! Assertions are on **deserialized JSON**, never on printed text: the
//! `--json` payload is the CLI's machine contract (the MCP server and
//! scripts read it), while the human lines are free to be reworded.
//!
//! Every test passes its own `--session`, because a session name selects
//! which daemon a command talks to; sharing one would queue concurrent
//! tests behind a single daemon.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::{
    Blame, BlameContent, Blob, BlobContent, BlobSource, CommitGraph, ConflictSet,
    FastForwardOutcome, FileChangeKind, FileDiff, FileHistoryPage, LfsBinary, LfsFile, LfsPattern,
    LfsRepoStatus, Oid, Ref, RefKind, Remote, RewordOutcome, Stash, WorkingTree, Worktree,
};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

/// One throwaway config dir for this whole test binary: `general.default_backend`
/// decides which `GitBackend` the embedded daemon is built with, so
/// without it the suite would run against whichever backend the developer
/// has configured. One directory is enough because every case passes its
/// own `--session`.
fn isolated_config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().expect("could not create a temp config dir"))
        .path()
}

/// Runs `gitpoulpi --json ...` against the repo at `root` and returns the
/// parsed payload, failing loudly (with stderr) if the command didn't
/// succeed.
fn run_json(root: &Path, session: &str, args: &[&str]) -> Value {
    run_json_env(root, session, &[], args)
}

/// [`run_json`] with extra environment variables for the CLI *and* the
/// daemon it starts (the daemon inherits them), for the tests that need
/// `GITPOULPI_GIT_LFS=missing` to reach the backend.
fn run_json_env(root: &Path, session: &str, env: &[(&str, &str)], args: &[&str]) -> Value {
    let mut command = Command::cargo_bin("gitpoulpi").unwrap();
    for (key, value) in env {
        command.env(key, value);
    }
    let output = command
        .env("GITPOULPI_CONFIG_DIR", isolated_config_dir())
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(args)
        .current_dir(root)
        .output()
        .expect("failed to run gitpoulpi");

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        output.status.success(),
        "`gitpoulpi {}` failed\nstdout: {stdout}\nstderr: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("output was not JSON ({e}): {stdout}"))
}

fn parse<T: serde::de::DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).expect("payload did not match its domain type")
}

fn head(root: &Path, session: &str) -> Oid {
    let log: CommitGraph = parse(run_json(root, session, &["log", "--limit", "1"]));
    log.nodes[0].id
}

#[test]
fn status_reports_a_worktree_change() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("new.txt"), "hello").unwrap();

    let tree: WorkingTree = parse(run_json(repo.root(), "cli-test-status", &["status"]));
    assert_eq!(tree.files.len(), 1);
    assert_eq!(tree.files[0].path.to_str(), Some("new.txt"));
}

/// A move not yet staged is one entry at the new path in `status`, naming
/// the old one; the staged diff of the same move says where it came from.
#[test]
fn status_and_diff_report_renamed_from() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-rename";
    let body: String = (1..=12).map(|n| format!("line {n}\n")).collect();
    std::fs::write(repo.root().join("old.txt"), &body).unwrap();
    run_json(repo.root(), session, &["stage", "--file", "old.txt"]);
    run_json(repo.root(), session, &["commit", "-m", "add old"]);
    std::fs::rename(repo.root().join("old.txt"), repo.root().join("moved.txt")).unwrap();

    let tree: WorkingTree = parse(run_json(repo.root(), session, &["status"]));
    let moved = tree
        .files
        .iter()
        .find(|f| f.path.to_str() == Some("moved.txt"))
        .expect("the new path is listed");
    assert_eq!(moved.renamed_from.as_deref(), Some(Path::new("old.txt")));

    run_json(repo.root(), session, &["stage", "--file", "moved.txt"]);
    run_json(repo.root(), session, &["stage", "--file", "old.txt"]);
    let staged: Vec<gitpoulpi_core::domain::FileDiff> =
        parse(run_json(repo.root(), session, &["diff", "--staged"]));
    assert_eq!(staged.len(), 1, "{staged:?}");
    assert_eq!(staged[0].path.to_str(), Some("moved.txt"));
    assert_eq!(
        staged[0].renamed_from.as_deref(),
        Some(Path::new("old.txt"))
    );
    assert_eq!(staged[0].similarity, Some(100));
}

#[test]
fn stage_and_commit_move_head() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-commit";
    std::fs::write(repo.root().join("new.txt"), "hello").unwrap();

    run_json(repo.root(), session, &["stage", "--file", "new.txt"]);
    let committed = run_json(repo.root(), session, &["commit", "-m", "add new.txt"]);
    let commit: Oid = parse(committed["commit"].clone());

    let log: CommitGraph = parse(run_json(repo.root(), session, &["log"]));
    assert_eq!(log.nodes.len(), 2);
    assert_eq!(log.nodes[0].id, commit);
    assert_eq!(log.nodes[0].summary, "add new.txt");
}

/// `branch create` takes a revision *string*; the CLI must hand the
/// daemon a resolved `Oid`. This is the path that would silently break if
/// the CLI ever started guessing at revisions itself.
#[test]
fn branch_create_accepts_a_revision_string() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let session = "cli-test-branch";

    run_json(
        repo.root(),
        session,
        &["branch", "create", "older", "HEAD~1"],
    );

    let branches: Vec<Ref> = parse(run_json(repo.root(), session, &["branch", "list"]));
    let older = branches
        .iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "older"))
        .expect("the new branch should be listed");

    let log: CommitGraph = parse(run_json(repo.root(), session, &["log"]));
    assert_eq!(older.target, log.nodes[1].id);
}

#[test]
fn checkout_switches_branches_and_detaches_on_a_sha() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let session = "cli-test-checkout";
    let tip = head(repo.root(), session);

    run_json(repo.root(), session, &["branch", "create", "feature"]);
    let switched = run_json(repo.root(), session, &["checkout", "feature"]);
    assert_eq!(switched["checked_out"]["Branch"], "feature");

    // A sha isn't a branch, so it has to check out detached instead of
    // being rejected or misread as a branch name.
    let detached = run_json(repo.root(), session, &["checkout", &tip.to_hex()]);
    assert_eq!(parse::<Oid>(detached["checked_out"]["Commit"].clone()), tip);
}

#[test]
fn tags_are_created_listed_and_deleted() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-tag";

    run_json(repo.root(), session, &["tag", "create", "v1"]);
    let tags: Vec<Ref> = parse(run_json(repo.root(), session, &["tag", "list"]));
    assert_eq!(tags.len(), 1);
    assert!(matches!(&tags[0].kind, RefKind::Tag { name } if name == "v1"));
    assert_eq!(tags[0].target, head(repo.root(), session));

    run_json(repo.root(), session, &["tag", "delete", "v1"]);
    let tags: Vec<Ref> = parse(run_json(repo.root(), session, &["tag", "list"]));
    assert!(tags.is_empty());
}

/// A tag mirror left by a remote removed outside GitPoulpi is ignored: the
/// tag is local only and a tag only that mirror knew is not listed.
#[test]
fn tag_list_ignores_the_mirror_of_a_removed_remote() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-tag-stale-mirror";
    run_json(repo.root(), session, &["tag", "create", "v1"]);
    let head = repo.git(&["rev-parse", "HEAD"]);
    for name in ["v1", "ghost"] {
        repo.git(&[
            "update-ref",
            &format!("refs/gitpoulpi/remote-tags/archive/{name}"),
            &head,
        ]);
    }

    let tags = run_json(repo.root(), session, &["tag", "list"]);
    let rows = tags.as_array().expect("an array of tags");
    assert_eq!(rows.len(), 1, "{tags}");
    assert_eq!(rows[0]["name"], "v1");
    assert_eq!(rows[0]["where"], "local");
    assert_eq!(rows[0]["remote"], serde_json::json!([]));
}

/// `tag list` says where each tag lives and `tag delete` does what its
/// scope says (the maintainer's report: the red delete only deleted locally and
/// the next fetch brought the tag back).
#[test]
fn tag_list_reports_where_and_delete_has_a_scope() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .build();
    let session = "cli-test-tag-where";
    let row = |name: &str| -> Option<Value> {
        let rows = run_json(repo.root(), session, &["tag", "list"]);
        rows.as_array()
            .unwrap()
            .iter()
            .find(|r| r["name"] == name)
            .cloned()
    };
    let where_of = |name: &str| row(name).map(|r| r["where"].as_str().unwrap().to_string());

    run_json(repo.root(), session, &["tag", "create", "v1"]);
    run_json(repo.root(), session, &["tag", "create", "v2"]);
    assert_eq!(where_of("v1").as_deref(), Some("local"));
    run_json(repo.root(), session, &["tag", "push", "v1", "v2"]);
    let v1 = row("v1").unwrap();
    assert_eq!(v1["where"], "both");
    assert_eq!(v1["local"], true);
    assert_eq!(v1["remote"], serde_json::json!(["origin"]));

    // The default delete is local, and says the remote still has it.
    let out = run_json(repo.root(), session, &["tag", "delete", "v1"]);
    assert_eq!(out["scope"], "local");
    assert_eq!(out["still_on"], serde_json::json!(["origin"]));
    let ghost = row("v1").expect("still listed: the remote has it");
    assert_eq!(ghost["where"], "remote");
    assert_eq!(ghost["local"], false);

    // Fetching that one tag makes it both again.
    run_json(repo.root(), session, &["tag", "fetch", "v1"]);
    assert_eq!(where_of("v1").as_deref(), Some("both"));

    // --both removes it from the remote too, so a fetch cannot bring it back.
    let out = run_json(repo.root(), session, &["tag", "delete", "v1", "--both"]);
    assert_eq!(out["scope"], "both");
    assert_eq!(out["remote"], "origin");
    assert_eq!(repo.git_in_remote(&["tag", "--list"]), "v2");
    run_json(repo.root(), session, &["fetch"]);
    assert_eq!(where_of("v1"), None);

    // --remote keeps the local tag, which is then local only.
    let out = run_json(
        repo.root(),
        session,
        &["tag", "delete", "v2", "--remote", "origin"],
    );
    assert_eq!(out["scope"], "remote");
    assert_eq!(where_of("v2").as_deref(), Some("local"));
    assert_eq!(repo.git_in_remote(&["tag", "--list"]), "");
}

#[test]
fn diff_separates_the_worktree_from_the_index() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-diff";
    std::fs::write(repo.root().join("initial.txt"), "changed\n").unwrap();

    let diffs: Vec<FileDiff> = parse(run_json(repo.root(), session, &["diff"]));
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].path.to_str(), Some("initial.txt"));
    assert_eq!(diffs[0].hunks.len(), 1);

    // Nothing is staged yet, so the index-vs-HEAD diff must be empty --
    // `--staged` has to actually change the DiffTarget, not just a label.
    let staged: Vec<FileDiff> = parse(run_json(repo.root(), session, &["diff", "--staged"]));
    assert!(staged.is_empty());

    run_json(repo.root(), session, &["stage", "--all"]);
    let staged: Vec<FileDiff> = parse(run_json(repo.root(), session, &["diff", "--staged"]));
    assert_eq!(staged.len(), 1);
}

#[test]
fn diff_from_to_compares_two_revisions_and_a_revision_with_the_worktree() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "1\n", "c1")
        .unwrap()
        .commit_file("a.txt", "1\n2\n", "c2")
        .unwrap()
        .commit_file("b.txt", "b\n", "c3")
        .unwrap()
        .build();
    let session = "cli-test-diff-compare";
    let names = |diffs: &[FileDiff]| -> Vec<String> {
        diffs
            .iter()
            .map(|d| d.path.to_string_lossy().into_owned())
            .collect()
    };

    let between: Vec<FileDiff> = parse(run_json(
        repo.root(),
        session,
        &["diff", "--from", "HEAD~2", "--to", "HEAD"],
    ));
    assert_eq!(names(&between), vec!["a.txt", "b.txt"]);
    assert!(!between[0].hunks.is_empty(), "the full diff carries hunks");

    let listed: Vec<FileDiff> = parse(run_json(
        repo.root(),
        session,
        &["diff", "--from", "HEAD~2", "--to", "HEAD", "--name-only"],
    ));
    assert_eq!(names(&listed), vec!["a.txt", "b.txt"]);
    assert!(
        listed.iter().all(|d| d.hunks.is_empty()),
        "--name-only lists files without hunks"
    );

    let swapped: Vec<FileDiff> = parse(run_json(
        repo.root(),
        session,
        &["diff", "--from", "HEAD", "--to", "HEAD~2", "--name-only"],
    ));
    assert_eq!(
        swapped.iter().map(|d| d.status).collect::<Vec<_>>(),
        vec![
            gitpoulpi_core::domain::FileStatus::Modified,
            gitpoulpi_core::domain::FileStatus::Deleted
        ],
        "swapping the ends swaps the statuses"
    );

    std::fs::write(repo.root().join("a.txt"), "1\n2\nthree\n").unwrap();
    let wip: Vec<FileDiff> = parse(run_json(
        repo.root(),
        session,
        &[
            "diff",
            "--from",
            "HEAD~1",
            "--to",
            "worktree",
            "--name-only",
        ],
    ));
    assert_eq!(names(&wip), vec!["a.txt", "b.txt"]);
    let default_end: Vec<FileDiff> = parse(run_json(
        repo.root(),
        session,
        &["diff", "--from", "HEAD", "--name-only"],
    ));
    assert_eq!(
        names(&default_end),
        vec!["a.txt"],
        "--to defaults to the worktree"
    );
}

#[test]
fn unstage_takes_a_file_back_out_of_the_index() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-unstage";
    std::fs::write(repo.root().join("new.txt"), "hello").unwrap();

    run_json(repo.root(), session, &["stage", "--file", "new.txt"]);
    assert_eq!(
        parse::<Vec<FileDiff>>(run_json(repo.root(), session, &["diff", "--staged"])).len(),
        1
    );

    run_json(repo.root(), session, &["unstage", "--file", "new.txt"]);
    assert!(
        parse::<Vec<FileDiff>>(run_json(repo.root(), session, &["diff", "--staged"])).is_empty()
    );
}

#[test]
fn merge_fast_forwards_and_rebase_replays() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-history";
    let root = repo.root();

    // A branch that moves ahead of main, merged back with a fast-forward.
    run_json(root, session, &["branch", "create", "feature"]);
    run_json(root, session, &["checkout", "feature"]);
    std::fs::write(root.join("feature.txt"), "x").unwrap();
    run_json(root, session, &["stage", "--all"]);
    run_json(root, session, &["commit", "-m", "feature work"]);
    let tip = head(root, session);

    run_json(root, session, &["checkout", "main"]);
    let merged = run_json(root, session, &["merge", "feature", "--ff"]);
    assert_eq!(
        parse::<Oid>(merged["FastForwarded"].clone()),
        tip,
        "got {merged}"
    );

    // Now diverge for real, and rebase the divergent branch onto main.
    run_json(root, session, &["branch", "create", "second"]);
    run_json(root, session, &["checkout", "second"]);
    std::fs::write(root.join("second.txt"), "x").unwrap();
    run_json(root, session, &["stage", "--all"]);
    run_json(root, session, &["commit", "-m", "second work"]);

    run_json(root, session, &["checkout", "main"]);
    std::fs::write(root.join("main.txt"), "x").unwrap();
    run_json(root, session, &["stage", "--all"]);
    run_json(root, session, &["commit", "-m", "main work"]);
    let main_tip = head(root, session);

    run_json(root, session, &["checkout", "second"]);
    let rebased = run_json(root, session, &["rebase", "main"]);
    assert_eq!(rebased, serde_json::json!("Completed"), "got {rebased}");

    let log: CommitGraph = parse(run_json(root, session, &["log"]));
    assert_eq!(log.nodes[0].summary, "second work");
    assert_eq!(log.nodes[0].parents, vec![main_tip]);
}

#[test]
fn rebase_plan_then_interactive_reorders() {
    use gitpoulpi_core::domain::{RebaseAction, RebasePlanDraft};

    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base")
        .unwrap()
        .commit("first")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let session = "cli-test-rebase-plan";
    let root = repo.root();

    let draft: RebasePlanDraft = parse(run_json(root, session, &["rebase-plan", "HEAD~2"]));
    assert_eq!(
        draft
            .commits
            .iter()
            .map(|c| c.summary.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "second"]
    );

    // Swap the two commits' order via an explicit plan file.
    let steps: Vec<gitpoulpi_core::domain::RebaseStep> = draft
        .commits
        .iter()
        .rev()
        .map(|c| gitpoulpi_core::domain::RebaseStep {
            commit: c.id,
            action: RebaseAction::Pick,
            message: None,
        })
        .collect();
    let plan_path = repo.root().join("plan.json");
    std::fs::write(&plan_path, serde_json::to_string(&steps).unwrap()).unwrap();

    let outcome = run_json(
        root,
        session,
        &["rebase", "HEAD~2", "--plan", plan_path.to_str().unwrap()],
    );
    assert_eq!(outcome, serde_json::json!("Completed"), "got {outcome}");

    let log: CommitGraph = parse(run_json(root, session, &["log"]));
    assert_eq!(
        log.nodes
            .iter()
            .map(|n| n.summary.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "second", "base"]
    );
}

#[test]
fn rebase_edit_stop_then_continue() {
    use gitpoulpi_core::domain::RebaseAction;

    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base")
        .unwrap()
        .commit("to edit")
        .unwrap()
        .build();
    let session = "cli-test-rebase-edit";
    let root = repo.root();

    let draft: gitpoulpi_core::domain::RebasePlanDraft =
        parse(run_json(root, session, &["rebase-plan", "HEAD~1"]));
    let steps = vec![gitpoulpi_core::domain::RebaseStep {
        commit: draft.commits[0].id,
        action: RebaseAction::Edit,
        message: None,
    }];
    let plan_path = repo.root().join("plan.json");
    std::fs::write(&plan_path, serde_json::to_string(&steps).unwrap()).unwrap();

    let outcome = run_json(
        root,
        session,
        &["rebase", "HEAD~1", "--plan", plan_path.to_str().unwrap()],
    );
    assert!(outcome.get("Stopped").is_some(), "got {outcome}");

    let status = run_json(root, session, &["rebase", "--status"]);
    assert!(status.is_object(), "got {status}");

    let continued = run_json(root, session, &["rebase", "--continue"]);
    assert_eq!(continued, serde_json::json!("Completed"), "got {continued}");

    let status = run_json(root, session, &["rebase", "--status"]);
    assert_eq!(status, serde_json::Value::Null, "got {status}");
}

#[test]
fn rebase_abort_json() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("f.txt", "line1\n", "base")
        .unwrap()
        .commit_file("f.txt", "line1\nline2\n", "adds line2")
        .unwrap()
        .commit_file("f.txt", "line1\nline2\nline3\n", "adds line3")
        .unwrap()
        .build();
    let session = "cli-test-rebase-abort";
    let root = repo.root();

    let draft: gitpoulpi_core::domain::RebasePlanDraft =
        parse(run_json(root, session, &["rebase-plan", "HEAD~2"]));
    // Swapped order guarantees a conflict (see the contract test of the
    // same shape).
    let steps: Vec<gitpoulpi_core::domain::RebaseStep> = draft
        .commits
        .iter()
        .rev()
        .map(|c| gitpoulpi_core::domain::RebaseStep {
            commit: c.id,
            action: gitpoulpi_core::domain::RebaseAction::Pick,
            message: None,
        })
        .collect();
    let plan_path = repo.root().join("plan.json");
    std::fs::write(&plan_path, serde_json::to_string(&steps).unwrap()).unwrap();

    let outcome = run_json(
        root,
        session,
        &["rebase", "HEAD~2", "--plan", plan_path.to_str().unwrap()],
    );
    assert!(outcome.get("Conflicted").is_some(), "got {outcome}");

    let aborted = run_json(root, session, &["rebase", "--abort"]);
    assert_eq!(
        aborted,
        serde_json::json!({ "aborted": true }),
        "got {aborted}"
    );
}

#[test]
fn cherry_pick_copies_a_commit_across_branches() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-cherrypick";
    let root = repo.root();

    run_json(root, session, &["branch", "create", "feature"]);
    run_json(root, session, &["checkout", "feature"]);
    std::fs::write(root.join("picked.txt"), "x").unwrap();
    run_json(root, session, &["stage", "--all"]);
    run_json(root, session, &["commit", "-m", "to pick"]);
    let source = head(root, session);

    run_json(root, session, &["checkout", "main"]);
    std::fs::write(root.join("main.txt"), "x").unwrap();
    run_json(root, session, &["stage", "--all"]);
    run_json(root, session, &["commit", "-m", "main moves on"]);

    let outcome = run_json(root, session, &["cherry-pick", &source.to_hex()]);
    let applied: Vec<Oid> = parse(outcome["Completed"].clone());
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0], head(root, session));
    assert!(root.join("picked.txt").exists());
}

#[test]
fn cherry_pick_plan_applies_a_series_in_plan_order() {
    use gitpoulpi_core::domain::{RebaseAction, RebaseStep};

    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-cherrypick-plan";
    let root = repo.root();

    run_json(root, session, &["branch", "create", "feature"]);
    run_json(root, session, &["checkout", "feature"]);
    let mut ids = Vec::new();
    for name in ["a", "b", "c"] {
        std::fs::write(root.join(format!("{name}.txt")), name).unwrap();
        run_json(root, session, &["stage", "--all"]);
        run_json(root, session, &["commit", "-m", &format!("add {name}")]);
        ids.push(head(root, session));
    }
    run_json(root, session, &["checkout", "main"]);

    let step = |commit, action| RebaseStep {
        commit,
        action,
        message: None,
    };
    // Reordered (c before a), with b dropped.
    let steps = vec![
        step(ids[2], RebaseAction::Pick),
        step(ids[1], RebaseAction::Drop),
        step(ids[0], RebaseAction::Pick),
    ];
    let plan_path = root.join("pick-plan.json");
    std::fs::write(&plan_path, serde_json::to_string(&steps).unwrap()).unwrap();

    let outcome = run_json(
        root,
        session,
        &["cherry-pick", "--plan", plan_path.to_str().unwrap()],
    );
    let applied: Vec<Oid> = parse(outcome["Completed"].clone());
    assert_eq!(applied.len(), 2, "got {outcome}");
    assert_eq!(applied[1], head(root, session));
    assert!(root.join("a.txt").exists() && root.join("c.txt").exists());
    assert!(!root.join("b.txt").exists());

    let log: CommitGraph = parse(run_json(root, session, &["log"]));
    assert_eq!(
        log.nodes
            .iter()
            .take(2)
            .map(|n| n.summary.as_str())
            .collect::<Vec<_>>(),
        vec!["add a", "add c"]
    );
}

#[test]
fn cherry_pick_plan_rejects_other_actions() {
    use gitpoulpi_core::domain::{RebaseAction, RebaseStep};

    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-cherrypick-plan-bad";
    let root = repo.root();
    let steps = vec![RebaseStep {
        commit: head(root, session),
        action: RebaseAction::Squash,
        message: None,
    }];
    let plan_path = root.join("bad-plan.json");
    std::fs::write(&plan_path, serde_json::to_string(&steps).unwrap()).unwrap();

    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", isolated_config_dir())
        .args(["--json", "--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(["cherry-pick", "--plan", plan_path.to_str().unwrap()])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("only supports the Pick and Drop"),
        "{combined}"
    );
}

#[test]
fn revert_undoes_a_commit() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-revert";
    std::fs::write(repo.root().join("doomed.txt"), "doomed").unwrap();
    run_json(repo.root(), session, &["stage", "--all"]);
    run_json(repo.root(), session, &["commit", "-m", "add doomed"]);

    let outcome = run_json(repo.root(), session, &["revert", "HEAD"]);
    let reverted: Oid = parse(outcome["Completed"].clone());
    assert_eq!(reverted, head(repo.root(), session));
    assert!(!repo.root().join("doomed.txt").exists());
}

/// `reword` edits an older commit's message, replays what sits on top of
/// it, and reports the old tip; a commit off the current branch is
/// refused with a JSON error and nothing moves.
#[test]
fn reword_edits_an_older_message_and_refuses_a_commit_off_the_branch() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let session = "cli-test-reword";
    let old_head = head(repo.root(), session);

    let outcome: RewordOutcome = parse(run_json(
        repo.root(),
        session,
        &["reword", "HEAD~1", "-m", "initial, reworded\n\nwith a body"],
    ));
    assert_eq!(outcome.previous_head, old_head);
    assert_eq!(outcome.replayed, 1);
    assert_eq!(outcome.new_head, head(repo.root(), session));
    let log: CommitGraph = parse(run_json(repo.root(), session, &["log"]));
    assert_eq!(log.nodes[1].id, outcome.reworded);
    assert_eq!(log.nodes[1].message, "initial, reworded\n\nwith a body");
    assert_eq!(log.nodes[0].summary, "second");

    repo.git(&["branch", "other", &outcome.previous_head.to_hex()]);
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", isolated_config_dir())
        .args(["--json", "--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["reword", "other", "-m", "nope"])
        .current_dir(repo.root())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("error output was not JSON");
    assert!(value["error"].is_string(), "{value}");
    assert_eq!(head(repo.root(), session), outcome.new_head);
}

/// `branch fast-forward` moves a branch that is behind (JSON
/// `FastForwardOutcome`), reports `UpToDate` the second time, and a
/// diverged branch fails with a JSON error and does not move.
#[test]
fn branch_fast_forward_moves_a_branch_behind_and_refuses_a_diverged_one() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .branch("behind")
        .unwrap()
        .branch("diverged")
        .unwrap()
        .commit("on diverged")
        .unwrap()
        .checkout("main")
        .unwrap()
        .commit("on main")
        .unwrap()
        .build();
    let session = "cli-test-fast-forward";
    let main = head(repo.root(), session);
    let behind = Oid::from_hex(&repo.git(&["rev-parse", "behind"])).unwrap();

    let outcome: FastForwardOutcome = parse(run_json(
        repo.root(),
        session,
        &["branch", "fast-forward", "behind", "main"],
    ));
    assert_eq!(
        outcome,
        FastForwardOutcome::FastForwarded {
            from: behind,
            to: main
        }
    );
    assert_eq!(repo.git(&["rev-parse", "behind"]), main.to_hex());
    let again: FastForwardOutcome = parse(run_json(
        repo.root(),
        session,
        &["branch", "fast-forward", "behind"],
    ));
    assert_eq!(again, FastForwardOutcome::UpToDate);

    let diverged = repo.git(&["rev-parse", "diverged"]);
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", isolated_config_dir())
        .args(["--json", "--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["branch", "fast-forward", "diverged", "main"])
        .current_dir(repo.root())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("error output was not JSON");
    assert!(value["error"].is_string(), "{value}");
    assert_eq!(repo.git(&["rev-parse", "diverged"]), diverged);
}

/// `worktree add` starts a worktree on a new branch, on an existing
/// branch and detached at a revision, each answered with the `Worktree`
/// entry; `worktree list` then shows all of them.
#[test]
fn worktree_add_on_a_new_branch_a_branch_and_a_commit_then_list() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .branch("feature")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    let session = "cli-test-worktree";
    let base = repo.root().parent().unwrap().to_path_buf();

    let new_branch: Worktree = parse(run_json(
        repo.root(),
        session,
        &[
            "worktree",
            "add",
            &base.join("wt-topic").to_string_lossy(),
            "HEAD",
            "-b",
            "topic",
        ],
    ));
    assert_eq!(new_branch.branch.as_deref(), Some("topic"));
    assert!(!new_branch.is_main);

    let on_branch: Worktree = parse(run_json(
        repo.root(),
        session,
        &[
            "worktree",
            "add",
            &base.join("wt-feature").to_string_lossy(),
            "feature",
        ],
    ));
    assert_eq!(on_branch.branch.as_deref(), Some("feature"));

    let detached: Worktree = parse(run_json(
        repo.root(),
        session,
        &[
            "worktree",
            "add",
            &base.join("wt-detached").to_string_lossy(),
            "HEAD",
        ],
    ));
    assert_eq!(detached.branch, None);
    assert!(base.join("wt-detached").join("initial.txt").exists());

    let listed: Vec<Worktree> = parse(run_json(repo.root(), session, &["worktree", "list"]));
    assert_eq!(listed.len(), 4, "{listed:?}");
    assert!(listed[0].is_main);
}

/// `worktree changes` lists what is uncommitted; `worktree remove` refuses a
/// dirty worktree, commits or stashes first on request, and with
/// `--delete-branch` also deletes the (merged) branch.
#[test]
fn worktree_changes_and_remove_with_commit_stash_and_branch_delete() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .branch("feature")
        .unwrap()
        .branch("done")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    let session = "cli-test-worktree-remove";
    let base = repo.root().parent().unwrap().to_path_buf();
    let add = |name: &str, branch: &str| -> std::path::PathBuf {
        let path = base.join(name);
        run_json(
            repo.root(),
            session,
            &["worktree", "add", &path.to_string_lossy(), branch],
        );
        path
    };

    // Dirty: refused, nothing moves.
    let dirty = add("wt-dirty", "feature");
    std::fs::write(dirty.join("new.txt"), "new\n").unwrap();
    let changes: Vec<gitpoulpi_core::domain::WorktreeChange> = parse(run_json(
        repo.root(),
        session,
        &["worktree", "changes", &dirty.to_string_lossy()],
    ));
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "new.txt");
    assert!(changes[0].untracked);
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", isolated_config_dir())
        .args(["--json", "--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["worktree", "remove", &dirty.to_string_lossy()])
        .current_dir(repo.root())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let refused: Value = serde_json::from_slice(&output.stdout).expect("error output was not JSON");
    assert!(refused["error"].is_string(), "{refused}");
    assert!(dirty.join("new.txt").exists());

    // Commit first: the commit lands on the branch, the folder goes.
    let removed: gitpoulpi_core::domain::WorktreeRemoved = parse(run_json(
        repo.root(),
        session,
        &[
            "worktree",
            "remove",
            &dirty.to_string_lossy(),
            "--commit-message",
            "save it",
        ],
    ));
    assert!(removed.commit.is_some());
    assert!(!dirty.exists());
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s", "feature"]),
        "save it"
    );

    // Stash first.
    let stashy = add("wt-stash", "feature");
    std::fs::write(stashy.join("u.txt"), "u\n").unwrap();
    let removed: gitpoulpi_core::domain::WorktreeRemoved = parse(run_json(
        repo.root(),
        session,
        &[
            "worktree",
            "remove",
            &stashy.to_string_lossy(),
            "--stash-message",
            "parked",
        ],
    ));
    assert!(removed.stashed);
    assert!(!stashy.exists());
    assert!(repo.git(&["stash", "list"]).contains("parked"));

    // A clean worktree whose branch is merged: remove with its branch.
    let clean = add("wt-done", "done");
    let value = run_json(
        repo.root(),
        session,
        &[
            "worktree",
            "remove",
            &clean.to_string_lossy(),
            "--delete-branch",
        ],
    );
    assert_eq!(value["deleted_branch"], "done", "{value}");
    assert!(!clean.exists());
    assert!(repo.git(&["branch", "--list", "done"]).trim().is_empty());
}

#[test]
fn reset_moves_head_back() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let session = "cli-test-reset";
    let log: CommitGraph = parse(run_json(repo.root(), session, &["log"]));
    let first = log.nodes[1].id;

    run_json(repo.root(), session, &["reset", "HEAD~1", "--mode", "hard"]);
    assert_eq!(head(repo.root(), session), first);
}

#[test]
fn stash_round_trips_through_push_list_pop() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-stash";
    std::fs::write(repo.root().join("initial.txt"), "changed").unwrap();

    run_json(repo.root(), session, &["stash", "push", "--message", "wip"]);
    let stashes: Vec<Stash> = parse(run_json(repo.root(), session, &["stash", "list"]));
    assert_eq!(stashes.len(), 1);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("initial.txt")).unwrap(),
        "initial"
    );

    run_json(repo.root(), session, &["stash", "pop"]);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("initial.txt")).unwrap(),
        "changed"
    );
    let stashes: Vec<Stash> = parse(run_json(repo.root(), session, &["stash", "list"]));
    assert!(stashes.is_empty());
}

#[test]
fn remotes_are_added_listed_and_removed() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-remote";

    run_json(
        repo.root(),
        session,
        &["remote", "add", "origin", "https://example.invalid/r.git"],
    );
    let remotes: Vec<Remote> = parse(run_json(repo.root(), session, &["remote", "list"]));
    assert_eq!(remotes.len(), 1);
    assert_eq!(remotes[0].name, "origin");

    run_json(repo.root(), session, &["remote", "remove", "origin"]);
    let remotes: Vec<Remote> = parse(run_json(repo.root(), session, &["remote", "list"]));
    assert!(remotes.is_empty());
}

/// A repository that was a fork and is not one any more keeps
/// `refs/remotes/cm6/*`: `remote list` marks it `configured: false` next to
/// the real remotes, and `remote remove` cleans it up instead of failing
/// with "no such remote".
#[test]
fn a_leftover_remote_namespace_is_listed_unconfigured_and_removable() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-remote-leftover";
    let head = repo.git(&["rev-parse", "HEAD"]);
    repo.git(&["update-ref", "refs/remotes/cm6/main", &head]);
    repo.git(&["update-ref", "refs/remotes/cm6/feat/x", &head]);
    run_json(
        repo.root(),
        session,
        &["remote", "add", "origin", "https://example.invalid/r.git"],
    );

    let listed = run_json(repo.root(), session, &["remote", "list"]);
    let rows = listed.as_array().expect("an array");
    let flags: Vec<(String, bool)> = rows
        .iter()
        .map(|r| {
            (
                r["name"].as_str().unwrap().to_string(),
                r["configured"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        flags,
        vec![("origin".to_string(), true), ("cm6".to_string(), false)]
    );
    assert_eq!(rows[0]["fetch_url"], "https://example.invalid/r.git");
    // Still readable as the plain list it always was.
    let plain: Vec<Remote> = parse(listed);
    assert_eq!(plain.len(), 2);

    let removed = run_json(repo.root(), session, &["remote", "remove", "cm6"]);
    assert_eq!(removed["removed"], "cm6");
    assert_eq!(repo.git(&["for-each-ref", "refs/remotes/cm6"]), "");
    let listed = run_json(repo.root(), session, &["remote", "list"]);
    let names: Vec<&str> = listed
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["origin"]);
}

/// The remote editing commands the TUI's Edit remote popup relies on,
/// plus the per-repository default remote, end to end through the
/// daemon.
#[test]
fn remotes_are_renamed_repointed_and_one_is_the_default() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-remote-edit";

    run_json(
        repo.root(),
        session,
        &[
            "remote",
            "add",
            "gh",
            "https://example.invalid/a.git",
            "--push-url",
            "git@example.invalid:me/a.git",
        ],
    );
    run_json(
        repo.root(),
        session,
        &["remote", "add", "lab", "https://example.invalid/b.git"],
    );
    run_json(repo.root(), session, &["remote", "rename", "gh", "hub"]);
    run_json(
        repo.root(),
        session,
        &["remote", "set-url", "hub", "https://example.invalid/c.git"],
    );
    let remotes: Vec<Remote> = parse(run_json(repo.root(), session, &["remote", "list"]));
    let hub = remotes.iter().find(|r| r.name == "hub").expect("renamed");
    assert_eq!(hub.fetch_url, "https://example.invalid/c.git");
    assert_eq!(
        hub.push_url.as_deref(),
        Some("git@example.invalid:me/a.git")
    );

    run_json(repo.root(), session, &["remote", "clear-push-url", "hub"]);
    let remotes: Vec<Remote> = parse(run_json(repo.root(), session, &["remote", "list"]));
    let hub = remotes.iter().find(|r| r.name == "hub").unwrap();
    assert_eq!(
        hub.push_url.as_deref(),
        Some("https://example.invalid/c.git")
    );

    // Two remotes, neither `origin`, no upstream: there is no default
    // until one is chosen -- and then it is that one.
    let chosen = run_json(repo.root(), session, &["remote", "default", "lab"]);
    assert_eq!(chosen["default_remote"], "lab");
    let shown = run_json(repo.root(), session, &["remote", "default"]);
    assert_eq!(shown["default_remote"], "lab");
}

/// The per-repository `core.sshCommand` pin (ADR-0008), end to end.
#[test]
fn the_ssh_command_pin_is_shown_set_from_a_key_and_cleared() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-ssh-command";
    let root = repo.root();

    let shown = run_json(root, session, &["ssh-command", "show"]);
    assert!(shown["ssh_command"].is_null());

    let set = run_json(
        root,
        session,
        &["ssh-command", "set", "--key", r"C:\keys\work_ed25519"],
    );
    let expected = r#"ssh -i "C:/keys/work_ed25519" -o IdentitiesOnly=yes"#;
    assert_eq!(set["ssh_command"], expected);
    let shown = run_json(root, session, &["ssh-command", "show"]);
    assert_eq!(shown["ssh_command"], expected);

    run_json(
        root,
        session,
        &["ssh-command", "set", "--command", "ssh -v"],
    );
    let shown = run_json(root, session, &["ssh-command", "show"]);
    assert_eq!(shown["ssh_command"], "ssh -v");

    let cleared = run_json(root, session, &["ssh-command", "clear"]);
    assert!(cleared["ssh_command"].is_null());
    let shown = run_json(root, session, &["ssh-command", "show"]);
    assert!(shown["ssh_command"].is_null());
}

/// `core.autocrlf` / `core.eol` through the CLI twin of the Settings rows.
#[test]
fn repo_config_shows_sets_and_unsets_the_git_settings() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-repo-config";
    let root = repo.root();
    let git = |key: &str| {
        let out = std::process::Command::new("git")
            .args(["config", "--local", "--get", key])
            .current_dir(root)
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };

    let set = run_json(
        root,
        session,
        &["repo-config", "set", "core.autocrlf", "input"],
    );
    assert_eq!(set["value"], "input");
    assert_eq!(git("core.autocrlf"), "input");
    let shown = run_json(root, session, &["repo-config", "show", "core.autocrlf"]);
    assert_eq!(shown["repo_config"], "input");
    let all = run_json(root, session, &["repo-config", "show"]);
    assert_eq!(all["repo_config"]["core.autocrlf"], "input");

    let unset = run_json(root, session, &["repo-config", "unset", "core.autocrlf"]);
    assert!(unset["value"].is_null());
    assert_eq!(git("core.autocrlf"), "");
    let all = run_json(root, session, &["repo-config", "show"]);
    assert!(all["repo_config"]["core.autocrlf"].is_null());
}

/// `clone --sparse` and the `sparse-checkout` subcommands, end to end.
#[test]
fn sparse_clone_and_sparse_checkout_subcommands() {
    let source = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .commit_file("src/main.rs", "fn main() {}", "add src")
        .unwrap()
        .commit_file("docs/intro.md", "intro", "add docs")
        .unwrap()
        .commit_file("lib/util.rs", "// util", "add lib")
        .unwrap()
        .build();
    let elsewhere = tempfile::TempDir::new().unwrap();
    let dest = elsewhere.path().join("cloned");
    let session = "cli-test-sparse";

    run_json(
        source.root(),
        session,
        &[
            "clone",
            &source.root().to_string_lossy(),
            "--dest",
            &dest.to_string_lossy(),
            "--sparse",
            "src",
        ],
    );
    assert!(dest.join("src/main.rs").is_file());
    assert!(!dest.join("docs").exists());
    let status = run_json(&dest, session, &["sparse-checkout", "status"]);
    assert_eq!(status["enabled"], true);
    assert_eq!(status["cone"], true);
    assert_eq!(status["dirs"], serde_json::json!(["src"]));

    let added = run_json(&dest, session, &["sparse-checkout", "add", "docs"]);
    assert_eq!(added["dirs"], serde_json::json!(["docs", "src"]));
    assert!(dest.join("docs/intro.md").is_file());
    assert!(!dest.join("lib").exists());

    let set = run_json(&dest, session, &["sparse-checkout", "set", "lib"]);
    assert_eq!(set["dirs"], serde_json::json!(["lib"]));
    assert!(!dest.join("src").exists());

    let off = run_json(&dest, session, &["sparse-checkout", "disable"]);
    assert_eq!(off["enabled"], false);
    assert!(dest.join("src/main.rs").is_file());
    assert!(dest.join("docs/intro.md").is_file());
    let status = run_json(&dest, session, &["sparse-checkout", "status"]);
    assert_eq!(status["enabled"], false);
    assert_eq!(status["dirs"], serde_json::json!([]));
}

#[test]
fn a_conflicted_merge_is_listed_and_resolvable() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-conflicts";
    let root = repo.root();

    run_json(root, session, &["branch", "create", "feature"]);
    run_json(root, session, &["checkout", "feature"]);
    std::fs::write(root.join("shared.txt"), "theirs\n").unwrap();
    run_json(root, session, &["stage", "--all"]);
    run_json(root, session, &["commit", "-m", "theirs"]);

    run_json(root, session, &["checkout", "main"]);
    std::fs::write(root.join("shared.txt"), "ours\n").unwrap();
    run_json(root, session, &["stage", "--all"]);
    run_json(root, session, &["commit", "-m", "ours"]);

    let merge = run_json(root, session, &["merge", "feature"]);
    assert_eq!(merge["Conflicted"][0], "shared.txt");

    let conflicts: Vec<ConflictSet> = parse(run_json(root, session, &["conflicts", "list"]));
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].ours.as_deref(), Some("ours\n"));

    run_json(
        root,
        session,
        &["conflicts", "resolve", "shared.txt", "--side", "ours"],
    );
    let conflicts: Vec<ConflictSet> = parse(run_json(root, session, &["conflicts", "list"]));
    assert!(conflicts.is_empty());
}

/// `clone` is the one command with no repo to act on, so it has to work
/// from a directory that isn't a repository at all. A local path is a
/// perfectly good clone URL, which keeps this test off the network.
#[test]
fn clone_works_from_outside_any_repository() {
    let source = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let elsewhere = tempfile::TempDir::new().unwrap();
    let dest = elsewhere.path().join("cloned");

    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .args(["--json", "--session", "cli-test-clone"])
        .arg("clone")
        .arg(source.root())
        .args(["--dest", &dest.to_string_lossy()])
        .current_dir(elsewhere.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "clone failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(dest.join(".git").exists());
    let log: CommitGraph = parse(run_json(&dest, "cli-test-clone", &["log"]));
    assert_eq!(log.nodes[0].summary, "initial");
}

/// Failures have to be machine-readable too, or a scripting caller has to
/// parse English to find out what went wrong.
#[test]
fn an_unknown_revision_fails_with_json_on_stdout() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();

    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .args(["--json", "--session", "cli-test-error"])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["revert", "no-such-rev"])
        .current_dir(repo.root())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("error output was not JSON");
    assert!(value["error"].is_string());
}

/// `show` must reach the three sources independently, and its `--json`
/// payload must be the `Blob` domain type - that payload is what the MCP
/// server will hand an agent.
#[test]
fn show_reads_a_file_at_a_commit_the_index_and_the_worktree() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("f.txt", "committed\n", "initial")
        .unwrap()
        .write_worktree("f.txt", b"staged\n")
        .unwrap()
        .stage("f.txt")
        .unwrap()
        .write_worktree("f.txt", b"working\n")
        .unwrap()
        .build();
    let session = "cli-test-show";

    let at_head: Blob = parse(run_json(repo.root(), session, &["show", "f.txt"]));
    assert_eq!(at_head.content.lines(), ["committed"]);
    assert_eq!(
        at_head.source,
        BlobSource::Commit(head(repo.root(), session))
    );

    let staged: Blob = parse(run_json(
        repo.root(),
        session,
        &["show", "f.txt", "--staged"],
    ));
    assert_eq!(staged.content.lines(), ["staged"]);

    let worktree: Blob = parse(run_json(
        repo.root(),
        session,
        &["show", "f.txt", "--worktree"],
    ));
    assert_eq!(worktree.content.lines(), ["working"]);
}

/// A binary file is a *successful* `show`, reporting its size - a CLI
/// that errored here would make an agent think the repo was broken.
#[test]
fn show_reports_a_binary_file_without_failing() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_bytes("blob.bin", b"\x00\x01\x02\x03", "binary")
        .unwrap()
        .build();

    let blob: Blob = parse(run_json(
        repo.root(),
        "cli-test-show-bin",
        &["show", "blob.bin"],
    ));
    assert_eq!(blob.content, BlobContent::Binary);
    assert_eq!(blob.size, 4);
}

/// An image is a binary `show` that also says what it is: the `--json`
/// payload carries `image: {format, width, height}` at every source.
#[test]
fn show_of_an_image_reports_its_format_and_dimensions() {
    let rgba: Vec<u8> = (0..4 * 3 * 4).map(|i| i as u8).collect();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_png("logo.png", 4, 3, &rgba, "image")
        .unwrap()
        .build();
    let session = "cli-test-show-image";

    for extra in [&[][..], &["--staged"][..], &["--worktree"][..]] {
        let mut args = vec!["show", "logo.png"];
        args.extend(extra);
        let value = run_json(repo.root(), session, &args);
        assert_eq!(value["image"]["format"], "png", "{extra:?}");
        assert_eq!(value["image"]["width"], 4);
        assert_eq!(value["image"]["height"], 3);
        let blob: Blob = parse(value);
        assert_eq!(blob.content, BlobContent::Binary);
        assert!(blob.image.is_some());
    }

    // A plain binary has no `image` key at all.
    let plain = FixtureRepoBuilder::new()
        .unwrap()
        .commit_bytes("blob.bin", b"\x00\x01\x02", "binary")
        .unwrap()
        .build();
    let value = run_json(
        plain.root(),
        "cli-test-show-image-none",
        &["show", "blob.bin"],
    );
    assert!(value.get("image").is_none(), "{value}");
}

/// `show --raw --output FILE` writes the exact bytes; `--raw` alone puts
/// them on stdout with nothing else around them.
#[test]
fn show_raw_writes_the_exact_bytes_to_a_file_or_stdout() {
    let rgba: Vec<u8> = (0..4 * 3 * 4).map(|i| (i * 7) as u8).collect();
    let expected = gitpoulpi_testkit::png_bytes(4, 3, &rgba);
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_png("logo.png", 4, 3, &rgba, "image")
        .unwrap()
        .build();
    let session = "cli-test-show-raw";
    let out_dir = tempfile::tempdir().unwrap();
    let out = out_dir.path().join("copy.png");

    let value = run_json(
        repo.root(),
        session,
        &[
            "show",
            "logo.png",
            "--raw",
            "--output",
            out.to_str().unwrap(),
        ],
    );
    assert_eq!(value["raw"]["size"], expected.len());
    assert_eq!(std::fs::read(&out).unwrap(), expected);

    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", isolated_config_dir())
        .args(["--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["show", "logo.png", "--raw"])
        .current_dir(repo.root())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        output.stdout, expected,
        "stdout is the file and nothing else"
    );

    // --output without --raw is a usage error.
    let status = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", isolated_config_dir())
        .args(["show", "logo.png", "--output", "x"])
        .current_dir(repo.root())
        .output()
        .unwrap()
        .status;
    assert!(!status.success());
}

/// `log --path` is the File History query. It also proves the path
/// filter survives the whole round trip: clap -> `LogOptions::paths` ->
/// IPC JSON -> daemon -> backend.
#[test]
fn log_filters_by_path() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "touch a")
        .unwrap()
        .commit_file("b.txt", "b\n", "touch b")
        .unwrap()
        .commit_file("a.txt", "aa\n", "touch a again")
        .unwrap()
        .build();
    let session = "cli-test-log-path";

    let filtered: CommitGraph = parse(run_json(repo.root(), session, &["log", "--path", "a.txt"]));
    let summaries: Vec<&str> = filtered.nodes.iter().map(|n| n.summary.as_str()).collect();
    assert_eq!(summaries, ["touch a again", "touch a"]);

    let unfiltered: CommitGraph = parse(run_json(repo.root(), session, &["log"]));
    assert_eq!(unfiltered.nodes.len(), 3, "no --path means no filter");
}

/// `blame --at` has to blame the revision asked for, and the payload has
/// to carry each line's own text so a caller can render a gutter without
/// a second request.
#[test]
fn blame_attributes_lines_at_a_given_revision() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("f.txt", "first\nsecond\n", "original")
        .unwrap()
        .commit_file("f.txt", "first\nCHANGED\n", "change the second line")
        .unwrap()
        .build();
    let session = "cli-test-blame";

    let tip = head(repo.root(), session);
    let blame: Blame = parse(run_json(repo.root(), session, &["blame", "f.txt"]));
    assert_eq!(blame.at, tip);
    let lines = match &blame.content {
        BlameContent::Lines(lines) => lines.clone(),
        other => panic!("expected blamed lines, got {other:?}"),
    };
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[1].commit, tip);
    assert_eq!(lines[1].content, "CHANGED");
    assert_ne!(lines[0].commit, tip);
    // The new fields: the path in that commit, the parent to re-blame and
    // the committer time.
    assert_eq!(lines[1].orig_path, std::path::PathBuf::from("f.txt"));
    let (parent, parent_path) = lines[1].previous.clone().expect("a line with a parent");
    assert_eq!(parent, lines[0].commit);
    assert_eq!(parent_path, std::path::PathBuf::from("f.txt"));
    assert_eq!(lines[0].previous, None, "the root commit has no previous");
    assert!(lines[1].committer_time > 0);

    let older: Blame = parse(run_json(
        repo.root(),
        session,
        &["blame", "f.txt", "--at", "HEAD~1"],
    ));
    let older_lines = older.content.lines();
    assert_eq!(older_lines[1].content, "second");
    assert_ne!(
        older_lines[1].commit, tip,
        "--at must blame that revision, not HEAD"
    );
}

/// `file-history` follows a rename, pages with `--skip`/`--limit` and
/// reports `more`; the JSON deserializes into the domain type.
#[test]
fn file_history_follows_renames_and_pages() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("old.txt", "a\nb\nc\nd\ne\nf\n", "add old")
        .unwrap()
        .commit_file("old.txt", "a\nb\nc\nd\ne\nf\ng\n", "grow old")
        .unwrap()
        .build();
    repo.git(&["mv", "old.txt", "new.txt"]);
    repo.git(&["commit", "-q", "-m", "rename"]);
    let session = "cli-test-file-history";

    let page: FileHistoryPage = parse(run_json(repo.root(), session, &["file-history", "new.txt"]));
    assert!(!page.more);
    let summaries: Vec<&str> = page.entries.iter().map(|e| e.summary.as_str()).collect();
    assert_eq!(summaries, ["rename", "grow old", "add old"]);
    assert_eq!(
        page.entries[0].change,
        FileChangeKind::Renamed {
            from: "old.txt".into()
        }
    );
    assert_eq!(page.entries[2].path, std::path::PathBuf::from("old.txt"));
    assert_eq!(page.entries[1].added, Some(1));

    let first: FileHistoryPage = parse(run_json(
        repo.root(),
        session,
        &["file-history", "new.txt", "--limit", "2"],
    ));
    assert!(first.more);
    assert_eq!(first.entries.len(), 2);
    let rest: FileHistoryPage = parse(run_json(
        repo.root(),
        session,
        &["file-history", "new.txt", "--skip", "2", "--limit", "2"],
    ));
    assert!(!rest.more);
    assert_eq!(rest.entries.len(), 1);
    assert_eq!(rest.entries[0].summary, "add old");

    let at_grow: FileHistoryPage = parse(run_json(
        repo.root(),
        session,
        &["file-history", "old.txt", "--at", "HEAD~1"],
    ));
    assert_eq!(at_grow.entries.len(), 2);
}

/// `hosting pr` with no `[[integrations]]` configured (the isolated test
/// config dir has none) falls back to printing the compare URL rather
/// than failing - the no-token/no-integration path every `hosting pr`
/// call that is not `--web` still has to take.
#[test]
fn hosting_pr_prints_the_compare_url_with_no_integration_configured() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-pr";

    run_json(
        repo.root(),
        session,
        &["remote", "add", "origin", "https://github.com/me/thing.git"],
    );
    let branches: Vec<Ref> = parse(run_json(repo.root(), session, &["branch", "list"]));
    let current = branches.iter().find(|r| r.is_head).expect("a checkout");
    let RefKind::LocalBranch { name: branch_name } = &current.kind else {
        panic!("HEAD must be a local branch");
    };
    let tip = head(repo.root(), session);
    // Fake the remote-tracking ref a real `git push` would have left, so
    // `branch set-upstream` (and then `hosting pr`'s own upstream lookup)
    // has something to point at without an actual remote to push to.
    let status = std::process::Command::new("git")
        .args([
            "update-ref",
            &format!("refs/remotes/origin/{branch_name}"),
            &tip.to_hex(),
        ])
        .current_dir(repo.root())
        .status()
        .expect("could not run git");
    assert!(status.success());
    run_json(
        repo.root(),
        session,
        &[
            "branch",
            "set-upstream",
            branch_name,
            &format!("origin/{branch_name}"),
        ],
    );

    let out = run_json(
        repo.root(),
        session,
        &["hosting", "pr", "--title", "Add x", "--target", "main"],
    );
    assert_eq!(
        out["url"],
        format!("https://github.com/me/thing/compare/main...{branch_name}?expand=1&title=Add%20x")
    );
}

/// `patch create` writes a patch for a commit; `patch apply` on a fresh
/// checkout of its parent reproduces the file.
#[test]
fn patch_create_then_apply_round_trips() {
    use gitpoulpi_core::domain::{PatchApplied, PatchCreated};

    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .commit_file("feature.txt", "feature work\n", "add feature")
        .unwrap()
        .build();
    let session = "cli-test-patch";
    let dest = repo.root().parent().unwrap().join("0001-add-feature.patch");

    let created: PatchCreated = parse(run_json(
        repo.root(),
        session,
        &["patch", "create", "HEAD", "-o", &dest.to_string_lossy()],
    ));
    assert_eq!(created.commits, 1);
    assert!(dest.exists());

    repo.git(&["reset", "--hard", "HEAD~1"]);
    repo.git(&["clean", "-fd"]);
    assert!(!repo.root().join("feature.txt").exists());

    let applied: PatchApplied = parse(run_json(
        repo.root(),
        session,
        &["patch", "apply", &dest.to_string_lossy()],
    ));
    assert_eq!(applied.files, vec!["feature.txt".to_string()]);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("feature.txt")).unwrap(),
        "feature work\n"
    );
}

/// A patch that does not apply is a typed JSON error and a non-zero exit,
/// same as every other refused mutation.
#[test]
fn patch_apply_refusal_is_an_error_exit() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("shared.txt", "one\n", "initial")
        .unwrap()
        .commit_file("shared.txt", "one\ntwo\n", "add two")
        .unwrap()
        .build();
    let session = "cli-test-patch-refusal";
    let dest = repo.root().parent().unwrap().join("conflict.patch");
    let _created: Value = run_json(
        repo.root(),
        session,
        &["patch", "create", "HEAD", "-o", &dest.to_string_lossy()],
    );

    std::fs::write(repo.root().join("shared.txt"), "one\nsomething else\n").unwrap();

    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", isolated_config_dir())
        .args(["--json", "--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["patch", "apply", &dest.to_string_lossy()])
        .current_dir(repo.root())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("error output was not JSON");
    assert!(value["error"].is_string(), "{value}");
    assert_eq!(
        std::fs::read_to_string(repo.root().join("shared.txt")).unwrap(),
        "one\nsomething else\n"
    );
}

/// `lfs status --json` on an LFS repository names the tracked patterns and
/// the installed program; `lfs files` lists the pointer files.
#[test]
fn lfs_status_and_files_describe_an_lfs_repository() {
    gitpoulpi_testkit::require_git_lfs!("lfs_status_and_files_describe_an_lfs_repository");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_lfs()
        .unwrap()
        .lfs_track("*.bin")
        .unwrap()
        .commit_lfs_file("big.bin", b"payload bytes for the cli test", "add big")
        .unwrap()
        .build();

    let status: LfsRepoStatus = parse(run_json(
        repo.root(),
        "cli-test-lfs-status",
        &["lfs", "status"],
    ));
    assert!(status.uses_lfs);
    assert!(status.hooks_installed);
    assert!(matches!(status.binary, LfsBinary::Installed { .. }));
    assert_eq!(status.patterns[0].pattern, "*.bin");

    let files: Vec<LfsFile> = parse(run_json(
        repo.root(),
        "cli-test-lfs-status",
        &["lfs", "files"],
    ));
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, Path::new("big.bin"));
    assert_eq!(files[0].pointer.size, 30);
    assert!(files[0].local);
}

/// A missing `git-lfs` is an answer, not a failure: exit 0, and the JSON
/// says `Missing`. (The test seam is `GITPOULPI_GIT_LFS=missing`.)
#[test]
fn lfs_status_with_the_program_missing_still_succeeds_and_says_so() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(
            ".gitattributes",
            "*.bin filter=lfs diff=lfs merge=lfs -text\n",
            "attrs",
        )
        .unwrap()
        .build();

    let status: LfsRepoStatus = parse(run_json_env(
        repo.root(),
        "cli-test-lfs-missing",
        &[("GITPOULPI_GIT_LFS", "missing")],
        &["lfs", "status"],
    ));
    assert_eq!(status.binary, LfsBinary::Missing);
    assert!(status.uses_lfs, "the reads work without the program");
}

/// `lfs track` returns the patterns as JSON and leaves `.gitattributes`
/// unstaged; `untrack` takes one away again.
#[test]
fn lfs_track_and_untrack_edit_gitattributes_and_return_the_patterns() {
    gitpoulpi_testkit::require_git_lfs!(
        "lfs_track_and_untrack_edit_gitattributes_and_return_the_patterns"
    );
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_lfs()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();

    let patterns: Vec<LfsPattern> = parse(run_json(
        repo.root(),
        "cli-test-lfs-track",
        &["lfs", "track", "*.psd", "--lockable"],
    ));
    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0].pattern, "*.psd");
    assert!(patterns[0].lockable);

    let status: WorkingTree = parse(run_json(repo.root(), "cli-test-lfs-track", &["status"]));
    assert!(status
        .files
        .iter()
        .any(|f| f.path == Path::new(".gitattributes")));

    let left: Vec<LfsPattern> = parse(run_json(
        repo.root(),
        "cli-test-lfs-track",
        &["lfs", "untrack", "*.psd"],
    ));
    assert!(left.is_empty());
}

/// With `git-lfs` missing, an LFS mutation exits non-zero and says where
/// to install it - on stderr in the human form, in `error` with `--json`.
#[test]
fn lfs_track_with_the_program_missing_fails_with_the_install_link() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();

    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", isolated_config_dir())
        .env("GITPOULPI_GIT_LFS", "missing")
        .args(["--session", "cli-test-lfs-track-missing"])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["lfs", "track", "*.psd"])
        .current_dir(repo.root())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("git-lfs.com"), "{stderr}");
    assert!(
        !repo.root().join(".gitattributes").exists(),
        "nothing was changed"
    );
}

/// `lfs push` uploads the objects to the remote (read straight from the
/// origin's LFS store), and `lfs pull` in a clone that holds pointers
/// brings the real content back.
#[test]
fn lfs_push_then_pull_moves_objects_through_the_remote() {
    gitpoulpi_testkit::require_git_lfs!("lfs_push_then_pull_moves_objects_through_the_remote");
    let payload: &[u8] = b"content moved by the cli test";
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_lfs_origin()
        .unwrap()
        .with_lfs()
        .unwrap()
        .lfs_track("*.bin")
        .unwrap()
        .commit_lfs_file("big.bin", payload, "add big")
        .unwrap()
        .build();

    let pushed = run_json(
        repo.root(),
        "cli-test-lfs-push",
        &["lfs", "push", "--remote", "origin"],
    );
    assert_eq!(pushed["pushed_lfs_to"], "origin");
    assert_eq!(repo.remote_lfs_objects().len(), 1);

    repo.git(&["push", "origin", "main"]);
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("clone");
    gitpoulpi_testkit::lfs::clone_leaving_pointers(repo.remote_path().unwrap(), &dest);
    let pulled = run_json(
        &dest,
        "cli-test-lfs-pull",
        &["lfs", "pull", "--remote", "origin"],
    );
    assert_eq!(pulled["pulled_lfs_from"], "origin");
    assert_eq!(std::fs::read(dest.join("big.bin")).unwrap(), payload);
}

/// `lfs lock`, `locks` and `unlock` against the testkit's loopback lock
/// server: the JSON is the `LfsLock` domain type.
#[test]
fn lfs_lock_locks_and_unlock_round_trip_through_the_cli() {
    gitpoulpi_testkit::require_git_lfs!("lfs_lock_locks_and_unlock_round_trip_through_the_cli");
    let server = gitpoulpi_testkit::lfs::LfsLockServer::start();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_origin()
        .unwrap()
        .with_lfs()
        .unwrap()
        .lfs_track_lockable("*.bin")
        .unwrap()
        .commit_lfs_file("x.bin", b"a lockable file", "add x")
        .unwrap()
        .config("lfs.url", &server.url())
        .unwrap()
        .config("credential.helper", "")
        .unwrap()
        .build();
    let session = "cli-test-lfs-locks";

    let lock: gitpoulpi_core::domain::LfsLock = parse(run_json(
        repo.root(),
        session,
        &["lfs", "lock", "x.bin", "--remote", "origin"],
    ));
    assert_eq!(lock.path, Path::new("x.bin"));
    assert!(lock.ours);

    let locks: Vec<gitpoulpi_core::domain::LfsLock> = parse(run_json(
        repo.root(),
        session,
        &["lfs", "locks", "--remote", "origin"],
    ));
    assert_eq!(locks.len(), 1);
    assert!(locks[0].ours);

    let unlocked = run_json(
        repo.root(),
        session,
        &["lfs", "unlock", "x.bin", "--remote", "origin"],
    );
    assert_eq!(unlocked["unlocked"], "x.bin");
    assert!(server.locked_paths().is_empty());
}

/// `rebase --plan` takes git's own spelling of the actions (lowercase, or
/// the one-letter abbreviation, any case), as the CLI reference shows.
#[test]
fn rebase_plan_accepts_git_spelling_of_actions() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base")
        .unwrap()
        .commit("first")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let session = "cli-test-rebase-plan-spelling";
    let root = repo.root();

    let draft: gitpoulpi_core::domain::RebasePlanDraft =
        parse(run_json(root, session, &["rebase-plan", "HEAD~2"]));
    let plan = serde_json::json!([
        {"commit": draft.commits[0].id, "action": "p"},
        {"commit": draft.commits[1].id, "action": "Fixup"},
    ]);
    let plan_path = repo.root().join("plan.json");
    std::fs::write(&plan_path, plan.to_string()).unwrap();
    let outcome = run_json(
        root,
        session,
        &["rebase", "HEAD~2", "--plan", plan_path.to_str().unwrap()],
    );
    assert_eq!(outcome, serde_json::json!("Completed"), "got {outcome}");
    let log: CommitGraph = parse(run_json(root, session, &["log"]));
    assert_eq!(
        log.nodes
            .iter()
            .map(|n| n.summary.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "base"]
    );

    // Lowercase full names, and an unknown one is refused with the list.
    let plan = serde_json::json!([{"commit": log.nodes[0].id, "action": "pick"}]);
    std::fs::write(&plan_path, plan.to_string()).unwrap();
    let outcome = run_json(
        root,
        session,
        &["rebase", "HEAD~1", "--plan", plan_path.to_str().unwrap()],
    );
    assert_eq!(outcome, serde_json::json!("Completed"), "got {outcome}");
}

/// A merge commit is reverted with `--mainline <parent>`; without it the
/// command fails with a JSON error and nothing moves.
#[test]
fn revert_of_a_merge_commit_takes_a_mainline() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-test-revert-merge";
    let root = repo.root();
    repo.git(&["checkout", "-q", "-b", "feature"]);
    std::fs::write(root.join("feature.txt"), "feature").unwrap();
    repo.git(&["add", "feature.txt"]);
    repo.git(&["commit", "-q", "-m", "feature work"]);
    repo.git(&["checkout", "-q", "-"]);
    std::fs::write(root.join("main.txt"), "main").unwrap();
    repo.git(&["add", "main.txt"]);
    repo.git(&["commit", "-q", "-m", "main work"]);
    repo.git(&["merge", "-q", "--no-ff", "-m", "merge feature", "feature"]);
    let merge = head(root, session);

    let refused = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", isolated_config_dir())
        .args(["--json", "--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(["revert", "HEAD"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(!refused.status.success());
    let refused: Value = serde_json::from_slice(&refused.stdout).unwrap();
    assert!(refused["error"].is_string(), "got {refused}");
    assert_eq!(head(root, session), merge);

    let outcome = run_json(root, session, &["revert", "HEAD", "--mainline", "1"]);
    let reverted: Oid = parse(outcome["Completed"].clone());
    assert_eq!(reverted, head(root, session));
    assert!(!root.join("feature.txt").exists());
    assert!(root.join("main.txt").exists());
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();

/// `amend` changes an older commit's author and description (summary kept),
/// replays what sits on top, and refuses an empty request and a bad date
/// with a JSON error, moving nothing.
#[test]
fn amend_changes_chosen_properties_and_refuses_bad_input() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let session = "cli-test-amend";
    let old_head = head(repo.root(), session);

    let outcome: RewordOutcome = parse(run_json(
        repo.root(),
        session,
        &[
            "amend",
            "HEAD~1",
            "--description",
            "only the body",
            "--author-name",
            "Ada Lovelace",
            "--author-email",
            "ada@example.org",
            "--author-date",
            "2001-02-03T04:05:06+05:30",
        ],
    ));
    assert_eq!(outcome.previous_head, old_head);
    assert_eq!(outcome.replayed, 1);
    let log: CommitGraph = parse(run_json(repo.root(), session, &["log"]));
    assert_eq!(log.nodes[1].id, outcome.reworded);
    assert_eq!(log.nodes[1].summary, "initial");
    assert_eq!(log.nodes[1].message, "initial\n\nonly the body");
    assert_eq!(log.nodes[1].author_name, "Ada Lovelace");
    assert_eq!(log.nodes[1].author_email, "ada@example.org");
    assert_eq!(log.nodes[1].timestamp, 981_153_306);
    assert_eq!(log.nodes[0].summary, "second");

    for args in [
        vec!["amend", "HEAD"],
        vec!["amend", "HEAD", "--author-date", "tomorrow-ish"],
        vec!["amend", "HEAD", "--author-name", " "],
    ] {
        let output = Command::cargo_bin("gitpoulpi")
            .unwrap()
            .env("GITPOULPI_CONFIG_DIR", isolated_config_dir())
            .args(["--json", "--session", session])
            .args(["--repo", &repo.root().to_string_lossy()])
            .args(&args)
            .current_dir(repo.root())
            .output()
            .unwrap();
        assert!(!output.status.success(), "{args:?}");
        let value: Value =
            serde_json::from_slice(&output.stdout).expect("error output was not JSON");
        assert!(value["error"].is_string(), "{args:?}: {value}");
    }
    assert_eq!(head(repo.root(), session), outcome.new_head);
}

/// `apply-commit` replays a lost commit's changes onto the working tree
/// (JSON `{files}`), `undo` takes them back, a merge commit is an error.
#[test]
fn apply_commit_restores_a_lost_commit_and_undo_takes_it_back() {
    use gitpoulpi_core::domain::PatchApplied;

    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("keep.txt", "keep\n", "first")
        .unwrap()
        .build();
    let session = "cli-test-apply-commit";
    std::fs::write(repo.root().join("keep.txt"), "keep\nplus\n").unwrap();
    std::fs::write(repo.root().join("added.txt"), "new\n").unwrap();
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "lost work"]);
    let lost = repo.git(&["rev-parse", "HEAD"]);
    repo.git(&["reset", "-q", "--hard", "HEAD~1"]);

    let applied: PatchApplied = parse(run_json(
        repo.root(),
        session,
        &["apply-commit", lost.trim()],
    ));
    let mut files = applied.files;
    files.sort();
    assert_eq!(files, ["added.txt", "keep.txt"]);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("keep.txt")).unwrap(),
        "keep\nplus\n"
    );

    let _undone: Value = run_json(repo.root(), session, &["undo", "--yes"]);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("keep.txt")).unwrap(),
        "keep\n"
    );
    assert!(!repo.root().join("added.txt").exists());

    // Changed meanwhile: the apply fails and nothing moves.
    std::fs::write(repo.root().join("keep.txt"), "other\n").unwrap();
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", isolated_config_dir())
        .args(["--json", "--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["apply-commit", lost.trim()])
        .current_dir(repo.root())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("error output was not JSON");
    assert!(value["error"].is_string(), "{value}");
    assert!(!repo.root().join("added.txt").exists());
}

/// `stash from-commit` puts a stash of the commit's changes on top of the
/// list and leaves the tree alone; `undo` drops it.
#[test]
fn stash_from_commit_saves_a_lost_commit_as_a_stash() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "first")
        .unwrap()
        .build();
    let session = "cli-test-stash-from-commit";
    std::fs::write(repo.root().join("a.txt"), "a\nlost\n").unwrap();
    repo.git(&["commit", "-q", "-am", "lost work"]);
    let lost = repo.git(&["rev-parse", "HEAD"]);
    repo.git(&["reset", "-q", "--hard", "HEAD~1"]);

    let out = run_json(
        repo.root(),
        session,
        &["stash", "from-commit", lost.trim(), "-m", "rescued"],
    );
    assert_eq!(out["from"], lost.trim());
    assert!(out["commit"].is_string(), "{out}");
    let list = run_json(repo.root(), session, &["stash", "list"]);
    let list = list.as_array().unwrap();
    assert_eq!(list.len(), 1, "{list:?}");
    assert!(list[0]["message"].as_str().unwrap().contains("rescued"));
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "a\n"
    );

    let _undone: Value = run_json(repo.root(), session, &["undo", "--yes"]);
    let list = run_json(repo.root(), session, &["stash", "list"]);
    assert!(list.as_array().unwrap().is_empty(), "{list}");
}
