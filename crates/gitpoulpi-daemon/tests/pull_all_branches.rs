//! `PullAllBranches` through the daemon: every row of the planner's table, the
//! checked-out branch in each pull mode, one journal entry and its undo,
//! nothing moved and no entry, a pending auto-stash, local changes with the
//! auto-stash, the other worktrees and a conflict on the checked-out branch.
//! Both backends.

use std::collections::HashMap;
use std::sync::Arc;

use gitpoulpi_core::domain::{
    BranchSyncItem, BranchSyncStatus, JournalState, OperationKind, PullAllBranchesOutcome, RepoId,
};
use gitpoulpi_core::{GitBackend, TypedError};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request, ResponseError};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};

gitpoulpi_testkit::isolate_binary!();

const SESSION: &str = "pull-all-branches-test";

fn backends() -> Vec<(&'static str, Arc<dyn GitBackend>)> {
    vec![
        (
            "system",
            Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new()),
        ),
        ("git2", Arc::new(gitpoulpi_git2_backend::Git2Backend::new())),
    ]
}

fn setup_env() {
    use std::sync::OnceLock;
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
        std::env::set_var("GITPOULPI_GENERAL_AUTO_STASH", "never");
    });
}

struct Rig {
    handle: Option<gitpoulpi_daemon::DaemonHandle>,
    client: Client,
    repo: FixtureRepo,
    id: RepoId,
    next: u64,
}

impl Rig {
    async fn new(backend: Arc<dyn GitBackend>, repo: FixtureRepo) -> Rig {
        setup_env();
        let handle = Daemon::new(SESSION).spawn(backend).await.unwrap();
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
                session: SESSION.into(),
                repo: Some(self.id.clone()),
                command,
                args,
            })
            .await
            .unwrap()
            .result
    }

    async fn pull_all_raw(&mut self, args: Value) -> Result<PullAllBranchesOutcome, ResponseError> {
        self.call(OperationKind::PullAllBranches, args)
            .await
            .map(|v| serde_json::from_value(v).unwrap())
    }

    async fn pull_all(&mut self, args: Value) -> PullAllBranchesOutcome {
        self.pull_all_raw(args)
            .await
            .unwrap_or_else(|e| panic!("pull all branches failed: {}", e.message))
    }

    async fn journal(&mut self) -> JournalState {
        serde_json::from_value(
            self.call(OperationKind::JournalState, json!({}))
                .await
                .unwrap(),
        )
        .unwrap()
    }

    fn tip(&self, rev: &str) -> String {
        self.repo.git(&["rev-parse", rev]).trim().to_string()
    }

    fn origin_tip(&self, branch: &str) -> String {
        self.repo
            .git_in_remote(&["rev-parse", &format!("refs/heads/{branch}")])
            .trim()
            .to_string()
    }

    fn file(&self, path: &str) -> Option<String> {
        std::fs::read_to_string(self.repo.root().join(path)).ok()
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

fn by_branch(out: &PullAllBranchesOutcome) -> HashMap<&str, &BranchSyncItem> {
    out.branches
        .iter()
        .map(|b| (b.branch.as_str(), b))
        .collect()
}

fn status_of(out: &PullAllBranchesOutcome, branch: &str) -> BranchSyncStatus {
    by_branch(out)
        .get(branch)
        .unwrap_or_else(|| panic!("no item for {branch}: {out:?}"))
        .status
}

/// `main` (checked out) plus one branch per case, every one pushed to origin
/// and tracking it, all in sync: `behind`, `uptodate`, `ahead` (one local
/// commit), `diverged` (one local commit; the test adds the remote one),
/// `gone` (the test deletes it on origin).
fn world() -> FixtureRepoBuilder {
    let mut b = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("base.txt", "base\n", "initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap();
    for name in ["behind", "uptodate", "ahead", "diverged", "gone"] {
        b = b
            .branch(name)
            .unwrap()
            .push(name)
            .unwrap()
            .track(name, "origin")
            .unwrap();
        if name == "ahead" || name == "diverged" {
            b = b.commit(&format!("local on {name}")).unwrap();
        }
        b = b.checkout("main").unwrap();
    }
    b
}

/// Plays another user editing `file` on `main` of origin.
fn edit_elsewhere(repo: &FixtureRepo, file: &str, contents: &str) {
    let bare = repo.git_in_remote(&["rev-parse", "--absolute-git-dir"]);
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("w");
    let run = |cwd: &std::path::Path, args: &[&str]| {
        let out = std::process::Command::new("git")
            .args([
                "-c",
                "user.name=Else",
                "-c",
                "user.email=else@gitpoulpi.local",
            ])
            .args(args)
            .current_dir(cwd)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    run(
        scratch.path(),
        &[
            "clone",
            "--branch",
            "main",
            bare.trim(),
            dir.to_str().unwrap(),
        ],
    );
    std::fs::write(dir.join(file), contents).unwrap();
    run(&dir, &["add", "-A"]);
    run(&dir, &["commit", "-m", "theirs"]);
    run(&dir, &["push", "origin", "main"]);
}

#[tokio::test]
async fn every_row_of_the_table_gets_its_status_and_only_behind_branches_move() {
    for (name, backend) in backends() {
        let repo = world().build();
        repo.push_from_elsewhere("main", "main moves");
        repo.push_from_elsewhere("behind", "behind moves");
        repo.push_from_elsewhere("diverged", "diverged moves");
        repo.git(&["push", "origin", ":gone"]);
        repo.git(&["checkout", "-q", "-b", "mirror", "main"]);
        repo.git(&["branch", "--set-upstream-to=main", "mirror"]);
        repo.git(&["checkout", "-q", "main"]);
        let mut rig = Rig::new(backend, repo).await;
        let (ahead_tip, diverged_tip, uptodate_tip) =
            (rig.tip("ahead"), rig.tip("diverged"), rig.tip("uptodate"));
        let before = rig.journal().await.undo_count;

        let out = rig
            .pull_all(json!({ "fetch_opts": { "prune": true } }))
            .await;

        assert_eq!(out.fetched, ["origin"], "{name}");
        assert!(out.fetch_failed.is_empty(), "{name}");
        // The checked-out branch comes first.
        assert_eq!(out.branches[0].branch, "main", "{name}");
        assert!(out.branches[0].is_head, "{name}");
        assert_eq!(status_of(&out, "main"), BranchSyncStatus::Updated, "{name}");
        assert_eq!(
            status_of(&out, "behind"),
            BranchSyncStatus::Updated,
            "{name}"
        );
        assert_eq!(
            status_of(&out, "diverged"),
            BranchSyncStatus::Diverged,
            "{name}"
        );
        assert_eq!(status_of(&out, "ahead"), BranchSyncStatus::Ahead, "{name}");
        assert_eq!(
            status_of(&out, "gone"),
            BranchSyncStatus::UpstreamGone,
            "{name}"
        );
        assert_eq!(
            status_of(&out, "uptodate"),
            BranchSyncStatus::UpToDate,
            "{name}"
        );
        // Another local branch as upstream: reported, never touched.
        assert_eq!(
            status_of(&out, "mirror"),
            BranchSyncStatus::LocalUpstream,
            "{name}"
        );
        assert_eq!(out.updated(), 2, "{name}");
        assert!(!out.has_failures(), "{name}");

        // Refs: the two behind branches follow origin; nothing else moved.
        assert_eq!(rig.tip("main"), rig.origin_tip("main"), "{name}");
        assert_eq!(rig.tip("behind"), rig.origin_tip("behind"), "{name}");
        assert_eq!(rig.tip("diverged"), diverged_tip, "{name}");
        assert_eq!(rig.tip("ahead"), ahead_tip, "{name}");
        assert_eq!(rig.tip("uptodate"), uptodate_tip, "{name}");
        // The checked-out branch's files followed.
        assert!(rig.file("main moves.txt").is_some(), "{name}");
        // One entry for the whole run.
        let state = rig.journal().await;
        assert_eq!(state.undo_count, before + 1, "{name}");
        assert_eq!(
            state.undo.as_ref().unwrap().label,
            "Pull all branches (2 updated)",
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn one_undo_gives_every_branch_and_the_files_back() {
    for (name, backend) in backends() {
        let repo = world().build();
        repo.push_from_elsewhere("main", "main moves");
        repo.push_from_elsewhere("behind", "behind moves");
        let mut rig = Rig::new(backend, repo).await;
        let (main_before, behind_before) = (rig.tip("main"), rig.tip("behind"));

        rig.pull_all(json!({})).await;
        assert_ne!(rig.tip("main"), main_before, "{name}");
        assert!(rig.file("main moves.txt").is_some(), "{name}");

        rig.call(OperationKind::Undo, json!({ "confirmed": true }))
            .await
            .unwrap_or_else(|e| panic!("{name}: undo failed: {}", e.message));
        assert_eq!(rig.tip("main"), main_before, "{name}");
        assert_eq!(rig.tip("behind"), behind_before, "{name}");
        assert!(rig.file("main moves.txt").is_none(), "{name}: files back");
        assert_eq!(
            rig.repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
            "main",
            "{name}"
        );

        rig.call(OperationKind::Redo, json!({ "confirmed": true }))
            .await
            .unwrap_or_else(|e| panic!("{name}: redo failed: {}", e.message));
        assert_eq!(rig.tip("main"), rig.origin_tip("main"), "{name}");
        assert_eq!(rig.tip("behind"), rig.origin_tip("behind"), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_run_that_moves_nothing_leaves_no_entry() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend, world().build()).await;
        let before = rig.journal().await.undo_count;
        let out = rig.pull_all(json!({})).await;
        assert_eq!(out.updated(), 0, "{name}");
        assert_eq!(
            status_of(&out, "main"),
            BranchSyncStatus::UpToDate,
            "{name}"
        );
        assert_eq!(
            out.head_mode, None,
            "{name}: no step for the checked-out branch"
        );
        assert_eq!(rig.journal().await.undo_count, before, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn the_checked_out_branch_follows_the_requested_mode_and_the_others_still_fast_forward() {
    for (name, backend) in backends() {
        // Merge: a merge commit with both histories.
        let repo = world().build();
        repo.git(&["commit", "-q", "--allow-empty", "-m", "local on main"]);
        repo.push_from_elsewhere("main", "main moves");
        repo.push_from_elsewhere("behind", "behind moves");
        let mut rig = Rig::new(backend.clone(), repo).await;
        let remote_tip = {
            rig.repo.git(&["fetch", "-q", "origin"]);
            rig.tip("origin/main")
        };
        let out = rig.pull_all(json!({ "head_mode": "merge" })).await;
        assert_eq!(status_of(&out, "main"), BranchSyncStatus::Updated, "{name}");
        assert_eq!(out.head_mode, Some(gitpoulpi_core::domain::PullMode::Merge));
        let parents = rig.repo.git(&["rev-list", "--parents", "-n", "1", "HEAD"]);
        assert_eq!(
            parents.split_whitespace().count(),
            3,
            "{name}: merge commit"
        );
        assert!(parents.contains(&remote_tip), "{name}");
        assert_eq!(rig.tip("behind"), rig.origin_tip("behind"), "{name}");
        rig.stop().await;

        // Rebase: linear, the local commit replayed on the remote tip.
        let repo = world().build();
        repo.git(&["commit", "-q", "--allow-empty", "-m", "local on main"]);
        repo.push_from_elsewhere("main", "main moves");
        let mut rig = Rig::new(backend.clone(), repo).await;
        let out = rig.pull_all(json!({ "head_mode": "rebase" })).await;
        assert_eq!(status_of(&out, "main"), BranchSyncStatus::Updated, "{name}");
        let parents = rig.repo.git(&["rev-list", "--parents", "-n", "1", "HEAD"]);
        assert_eq!(parents.split_whitespace().count(), 2, "{name}: linear");
        assert_eq!(rig.tip("HEAD~1"), rig.origin_tip("main"), "{name}");
        rig.stop().await;

        // Fast-forward only on a diverged branch: reported, nothing broken, the
        // other branches still move.
        let repo = world().build();
        repo.git(&["commit", "-q", "--allow-empty", "-m", "local on main"]);
        repo.push_from_elsewhere("main", "main moves");
        repo.push_from_elsewhere("behind", "behind moves");
        let mut rig = Rig::new(backend, repo).await;
        let main_before = rig.tip("main");
        let out = rig
            .pull_all(json!({ "head_mode": "fast_forward_only" }))
            .await;
        assert_eq!(
            status_of(&out, "main"),
            BranchSyncStatus::Diverged,
            "{name}: {out:?}"
        );
        assert_eq!(rig.tip("main"), main_before, "{name}");
        assert_eq!(
            status_of(&out, "behind"),
            BranchSyncStatus::Updated,
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn without_a_fetch_the_remote_tracking_refs_are_taken_as_they_are() {
    for (name, backend) in backends() {
        let repo = world().build();
        repo.push_from_elsewhere("main", "main moves");
        repo.push_from_elsewhere("behind", "behind moves");
        let mut rig = Rig::new(backend, repo).await;
        let (main_before, behind_before) = (rig.tip("main"), rig.tip("behind"));

        // Nothing was downloaded yet: the stale tracking refs say "in sync".
        let out = rig.pull_all(json!({ "fetch": false })).await;
        assert!(out.fetched.is_empty(), "{name}");
        assert_eq!(out.updated(), 0, "{name}");
        assert_eq!(rig.tip("main"), main_before, "{name}");
        assert_eq!(rig.tip("behind"), behind_before, "{name}");

        // After a download of the user's own, the run integrates what is there,
        // the checked-out branch included, without touching the network.
        rig.repo.git(&["fetch", "-q", "origin"]);
        for mode in ["fast_forward_only", "merge"] {
            let out = rig
                .pull_all(json!({ "fetch": false, "head_mode": mode }))
                .await;
            assert_eq!(
                status_of(&out, "main"),
                BranchSyncStatus::Updated,
                "{name} {mode}"
            );
            assert_eq!(rig.tip("main"), rig.origin_tip("main"), "{name} {mode}");
            assert_eq!(rig.tip("behind"), rig.origin_tip("behind"), "{name} {mode}");
            // Back to the start for the next mode.
            rig.repo.git(&["reset", "-q", "--hard", &main_before]);
            rig.repo.git(&["branch", "-f", "behind", &behind_before]);
        }
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_remote_that_cannot_be_fetched_is_reported_and_its_branches_are_left_alone() {
    for (name, backend) in backends() {
        let repo = world().build();
        repo.push_from_elsewhere("behind", "behind moves");
        let dead = repo.root().join("no-such-remote.git");
        repo.git(&["remote", "add", "dead", dead.to_str().unwrap()]);
        repo.git(&["branch", "--track", "stale", "origin/behind"]);
        repo.git(&["config", "branch.stale.remote", "dead"]);
        let mut rig = Rig::new(backend, repo).await;

        let out = rig.pull_all(json!({})).await;
        assert_eq!(out.fetched, ["origin"], "{name}");
        assert_eq!(out.fetch_failed.len(), 1, "{name}");
        assert_eq!(out.fetch_failed[0].remote, "dead", "{name}");
        assert_eq!(
            status_of(&out, "stale"),
            BranchSyncStatus::FetchFailed,
            "{name}"
        );
        assert!(out.has_failures(), "{name}");
        // The healthy remote's branches were still brought up to date.
        assert_eq!(
            status_of(&out, "behind"),
            BranchSyncStatus::Updated,
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_pending_auto_stash_refuses_the_run() {
    for (name, backend) in backends() {
        let repo = world()
            .commit_file("a.txt", "1\n2\n3\n", "add a")
            .unwrap()
            .push("main")
            .unwrap()
            .build();
        edit_elsewhere(&repo, "a.txt", "1\n2\n3 remote\n");
        std::fs::write(repo.root().join("a.txt"), "1\n2\n3 local\n").unwrap();
        let mut rig = Rig::new(backend, repo).await;

        // The first run stashes, integrates, and the pop conflicts: the stash
        // stays as a pending record.
        let e = rig
            .pull_all_raw(json!({ "autostash": "always" }))
            .await
            .unwrap_err();
        assert!(e.message.contains("stash"), "{name}: {}", e.message);
        rig.repo.git(&["reset", "-q", "--hard"]);

        let e = rig.pull_all_raw(json!({})).await.unwrap_err();
        assert!(
            matches!(&e.kind, Some(TypedError::PendingAutoStash { .. })),
            "{name}: {e:?}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn local_changes_in_the_way_stop_the_run_before_anything_moves_unless_the_auto_stash_is_on() {
    for (name, backend) in backends() {
        let repo = world()
            .commit_file("a.txt", "1\n2\n3\n", "add a")
            .unwrap()
            .push("main")
            .unwrap()
            .build();
        edit_elsewhere(&repo, "a.txt", "1 remote\n2\n3\n");
        repo.push_from_elsewhere("behind", "behind moves");
        let mut rig = Rig::new(backend, repo).await;
        let behind_before = rig.tip("behind");
        let before = rig.journal().await.undo_count;

        // Refused: the local edit would be overwritten ... and nothing else
        // moved either, the other branches included.
        std::fs::write(rig.repo.root().join("a.txt"), "1 local\n2\n3\n").unwrap();
        let e = rig
            .pull_all_raw(json!({ "autostash": "never" }))
            .await
            .unwrap_err();
        assert!(
            gitpoulpi_core::failure::is_local_changes_message(&e.message),
            "{name}: {}",
            e.message
        );
        assert_eq!(rig.tip("behind"), behind_before, "{name}: nothing moved");
        assert_eq!(rig.journal().await.undo_count, before, "{name}");

        // `ask`: the typed answer the frontends turn into a question.
        let e = rig
            .pull_all_raw(json!({ "autostash": "ask" }))
            .await
            .unwrap_err();
        assert!(
            matches!(&e.kind, Some(TypedError::NeedsAutoStash { .. })),
            "{name}: {e:?}"
        );

        // `always`: stashed, integrated, popped. The edit moves to a line the
        // remote did not touch, so the pop applies cleanly.
        std::fs::write(rig.repo.root().join("a.txt"), "1\n2\n3 local\n").unwrap();
        let out = rig
            .pull_all_raw(json!({ "autostash": "always" }))
            .await
            .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        assert_eq!(status_of(&out, "main"), BranchSyncStatus::Updated, "{name}");
        assert_eq!(
            status_of(&out, "behind"),
            BranchSyncStatus::Updated,
            "{name}"
        );
        assert_eq!(
            rig.file("a.txt").unwrap(),
            "1 remote\n2\n3 local\n",
            "{name}"
        );
        assert!(rig.repo.git(&["stash", "list"]).is_empty(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_behind_branch_in_another_worktree_is_skipped_or_updated_as_asked() {
    for (name, backend) in backends() {
        let (b, wt) = world().worktree("wt-feature", "feature/x").unwrap();
        let repo = b
            .push("feature/x")
            .unwrap()
            .track("feature/x", "origin")
            .unwrap()
            .build();
        repo.push_from_elsewhere("feature/x", "feature moves");
        let mut rig = Rig::new(backend, repo).await;
        let before = rig.tip("feature/x");

        // Skip (the default): reported, nothing touched.
        let out = rig.pull_all(json!({})).await;
        assert_eq!(
            status_of(&out, "feature/x"),
            BranchSyncStatus::CheckedOutElsewhere,
            "{name}: {out:?}"
        );
        assert_eq!(
            by_branch(&out)["feature/x"].worktree.as_deref(),
            Some(wt.as_path()),
            "{name}"
        );
        assert_eq!(rig.tip("feature/x"), before, "{name}");
        assert!(!wt.join("feature moves.txt").exists(), "{name}");

        // Update: fast-forwarded inside that worktree, files included.
        let out = rig.pull_all(json!({ "worktrees": "update" })).await;
        assert_eq!(
            status_of(&out, "feature/x"),
            BranchSyncStatus::WorktreeUpdated,
            "{name}: {out:?}"
        );
        assert_eq!(rig.tip("feature/x"), rig.origin_tip("feature/x"), "{name}");
        assert!(wt.join("feature moves.txt").exists(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn updating_a_worktree_with_local_changes_it_would_overwrite_is_that_branchs_failure() {
    for (name, backend) in backends() {
        let (b, wt) = world().worktree("wt-feature", "feature/x").unwrap();
        let repo = b
            .push("feature/x")
            .unwrap()
            .track("feature/x", "origin")
            .unwrap()
            .build();
        // Remote and worktree both touch `base.txt`.
        let scratch = tempfile::tempdir().unwrap();
        let dir = scratch.path().join("w");
        let bare = repo.git_in_remote(&["rev-parse", "--absolute-git-dir"]);
        let run = |cwd: &std::path::Path, args: &[&str]| {
            let out = std::process::Command::new("git")
                .args([
                    "-c",
                    "user.name=Else",
                    "-c",
                    "user.email=else@gitpoulpi.local",
                ])
                .args(args)
                .current_dir(cwd)
                .output()
                .unwrap();
            assert!(out.status.success(), "git {args:?}");
        };
        run(
            scratch.path(),
            &[
                "clone",
                "--branch",
                "feature/x",
                bare.trim(),
                dir.to_str().unwrap(),
            ],
        );
        std::fs::write(dir.join("base.txt"), "theirs\n").unwrap();
        run(&dir, &["commit", "-qam", "theirs"]);
        run(&dir, &["push", "-q", "origin", "feature/x"]);
        std::fs::write(wt.join("base.txt"), "my edit\n").unwrap();
        let mut rig = Rig::new(backend, repo).await;
        let before = rig.tip("feature/x");

        let out = rig.pull_all(json!({ "worktrees": "update" })).await;
        let item = by_branch(&out)["feature/x"];
        assert_eq!(item.status, BranchSyncStatus::Failed, "{name}: {out:?}");
        assert!(item.detail.is_some(), "{name}");
        assert!(out.has_failures(), "{name}");
        assert_eq!(rig.tip("feature/x"), before, "{name}");
        assert_eq!(
            std::fs::read_to_string(wt.join("base.txt")).unwrap(),
            "my edit\n",
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_conflict_on_the_checked_out_branch_leaves_the_pull_state_and_the_others_updated() {
    for (name, backend) in backends() {
        let repo = world().build();
        // Both sides edit base.txt.
        std::fs::write(repo.root().join("base.txt"), "mine\n").unwrap();
        repo.git(&["commit", "-qam", "mine"]);
        edit_elsewhere(&repo, "base.txt", "theirs\n");
        repo.push_from_elsewhere("behind", "behind moves");
        let mut rig = Rig::new(backend, repo).await;
        let main_before = rig.tip("main");
        let before = rig.journal().await.undo_count;

        let out = rig.pull_all(json!({ "head_mode": "merge" })).await;
        assert_eq!(
            status_of(&out, "main"),
            BranchSyncStatus::Conflicted,
            "{name}: {out:?}"
        );
        assert!(out.conflicted(), "{name}");
        // The same state a conflicting pull leaves.
        let merge_head = rig.repo.root().join(".git").join("MERGE_HEAD");
        assert!(merge_head.exists(), "{name}: a merge is in progress");
        assert_eq!(rig.tip("main"), main_before, "{name}");
        // The other branches were fast-forwarded before the stop.
        assert_eq!(
            status_of(&out, "behind"),
            BranchSyncStatus::Updated,
            "{name}"
        );
        assert_eq!(rig.tip("behind"), rig.origin_tip("behind"), "{name}");
        // Their fast-forwards are one entry of their own; the conflicted pull
        // has the pending entry a plain pull leaves, on top of it.
        let state = rig.journal().await;
        assert_eq!(state.undo_count, before + 2, "{name}");
        assert_eq!(state.undo.as_ref().unwrap().label, "Pull", "{name}");

        // Abort like after any pull: the pull's entry goes, the other branches'
        // entry stays and is undoable.
        rig.call(OperationKind::OperationAbort, json!({}))
            .await
            .unwrap();
        assert!(!merge_head.exists(), "{name}");
        // The abort has its own entry, as after any pull; the pull's pending
        // entry is gone, the other branches' entry (the first) stays.
        let listing = rig
            .call(OperationKind::JournalList, json!({}))
            .await
            .unwrap();
        let kinds: Vec<&str> = listing["undo"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|e| e["summary"]["kind"].as_str())
            .collect();
        assert_eq!(kinds, ["PullAllBranches", "OperationAbort"], "{name}");
        assert_eq!(rig.tip("main"), main_before, "{name}");
        rig.stop().await;
    }
}
