//! `gitpoulpi push --all-targets|--to`, `pull --all-sources` and
//! `remote-group plan|continue|abort|retry`, through the real binary, the
//! daemon and the backend, on fixtures with three bare remotes. Assertions
//! are on deserialized JSON (the domain types), never on text.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::{
    GroupAction, GroupMode, GroupRunStatus, RemoteGroupOutcome, RemoteGroupPlan, RemoteGroupStatus,
    TargetResult, TargetStatus,
};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

fn run(repo: &FixtureRepo, session: &str, args: &[&str]) -> (i32, Value) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .args(["--json", "--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(args)
        .current_dir(repo.root())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "{args:?}: not JSON ({e}): {stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code().unwrap_or(-1), value)
}

fn outcome(value: &Value) -> RemoteGroupOutcome {
    serde_json::from_value(value.clone())
        .unwrap_or_else(|e| panic!("not an outcome ({e}): {value}"))
}

fn result<'a>(out: &'a RemoteGroupOutcome, remote: &str) -> &'a TargetResult {
    out.targets
        .iter()
        .find(|t| t.remote == remote)
        .unwrap_or_else(|| panic!("no result for {remote}: {out:?}"))
}

/// `main` on origin (upstream), gitlab and backup, all equal to local, with
/// `targets` as extra push targets; then `local_commits` local commits.
fn base(local_commits: usize, file: Option<(&str, &str)>, targets: &[&str]) -> FixtureRepoBuilder {
    let mut b = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap();
    if let Some((path, contents)) = file {
        b = b.commit_file(path, contents, "add shared").unwrap();
    }
    b = b
        .with_origin()
        .unwrap()
        .with_remote("gitlab")
        .unwrap()
        .with_remote("backup")
        .unwrap()
        .push("main")
        .unwrap()
        .push_to("gitlab", "main")
        .unwrap()
        .push_to("backup", "main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .push_targets("main", targets)
        .unwrap();
    for i in 0..local_commits {
        b = b.commit(&format!("local {i}")).unwrap();
    }
    b
}

fn tip(repo: &FixtureRepo, remote: &str, branch: &str) -> String {
    let refname = format!("refs/heads/{branch}");
    let out = if remote == "origin" {
        repo.git_in_remote(&["rev-parse", &refname])
    } else {
        repo.git_in_named_remote(remote, &["rev-parse", &refname])
    };
    out.trim().to_string()
}

fn head(repo: &FixtureRepo) -> String {
    repo.git(&["rev-parse", "HEAD"]).trim().to_string()
}

fn assert_all_at_head(repo: &FixtureRepo) {
    let want = head(repo);
    for remote in ["origin", "gitlab", "backup"] {
        assert_eq!(tip(repo, remote, "main"), want, "{remote}");
    }
}

/// Plays another user editing `file` on `remote`'s `main`.
fn edit_elsewhere(repo: &FixtureRepo, remote: &str, file: &str, contents: &str) {
    let bare = if remote == "origin" {
        repo.git_in_remote(&["rev-parse", "--absolute-git-dir"])
    } else {
        repo.git_in_named_remote(remote, &["rev-parse", "--absolute-git-dir"])
    };
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("w");
    let git = |cwd: &Path, args: &[&str]| {
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
    git(
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
    git(&dir, &["commit", "-am", "theirs"]);
    git(&dir, &["push", "origin", "main"]);
}

/// Local edits `shared.txt`, gitlab edits it differently.
fn conflicting() -> FixtureRepo {
    let repo = base(0, Some(("shared.txt", "base\n")), &["gitlab", "backup"])
        .write_worktree("shared.txt", b"mine\n")
        .unwrap()
        .stage("shared.txt")
        .unwrap()
        .commit("mine")
        .unwrap()
        .build();
    edit_elsewhere(&repo, "gitlab", "shared.txt", "theirs\n");
    repo
}

#[test]
fn every_remote_a_fast_forward_runs_and_all_bare_remotes_hold_the_tip() {
    let repo = base(2, None, &["gitlab", "backup"]).build();
    let s = "rg-ff";
    let (code, v) = run(&repo, s, &["push", "--all-targets"]);
    assert_eq!(code, 0, "{v}");
    let out = outcome(&v);
    assert_eq!(out.status, GroupRunStatus::Done);
    assert_eq!(out.mode, GroupMode::Push);
    assert_eq!(out.targets.len(), 3);
    for t in &out.targets {
        assert!(
            matches!(t.status, TargetStatus::Pushed { .. }),
            "{t:?}: pushed"
        );
    }
    assert_all_at_head(&repo);
    // Everything is up to date now.
    let (code, v) = run(&repo, s, &["push", "--all-targets"]);
    assert_eq!(code, 0, "{v}");
    assert!(outcome(&v)
        .targets
        .iter()
        .all(|t| t.status.is_ok() && t.replaced.is_none()));
}

#[test]
fn a_diverged_remote_asks_for_a_resolution_then_resolve_runs_it() {
    let repo = base(1, Some(("shared.txt", "base\n")), &["gitlab", "backup"]).build();
    edit_elsewhere(&repo, "gitlab", "shared.txt", "theirs\n");
    let s = "rg-diverged";
    let before_origin = tip(&repo, "origin", "main");

    let (code, v) = run(&repo, s, &["push", "--all-targets"]);
    assert_eq!(code, 3, "{v}");
    let plan: RemoteGroupPlan =
        serde_json::from_value(v["needs_resolution"]["plan"].clone()).unwrap();
    let gitlab = plan.targets.iter().find(|r| r.remote == "gitlab").unwrap();
    assert_eq!(gitlab.suggested, GroupAction::MergeFrom);
    assert!(gitlab.allowed.contains(&GroupAction::ForceWithLease));
    assert_eq!(
        tip(&repo, "origin", "main"),
        before_origin,
        "nothing pushed"
    );

    // A bad or unavailable decision is a refusal, not a run.
    let (code, v) = run(
        &repo,
        s,
        &["push", "--all-targets", "--resolve", "gitlab=bogus"],
    );
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["refused"]["reason"], "bad_resolve");
    let (code, v) = run(
        &repo,
        s,
        &["push", "--all-targets", "--resolve", "origin=merge"],
    );
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["refused"]["reason"], "action_not_allowed");
    assert_eq!(tip(&repo, "origin", "main"), before_origin);

    let (code, v) = run(
        &repo,
        s,
        &["push", "--all-targets", "--resolve", "gitlab=merge"],
    );
    assert_eq!(code, 0, "{v}");
    let out = outcome(&v);
    assert_eq!(out.status, GroupRunStatus::Done);
    assert_all_at_head(&repo);
    let log = repo.git(&["log", "--format=%s"]);
    assert!(
        log.contains("theirs"),
        "gitlab's commit is merged in: {log}"
    );
}

#[test]
fn a_force_needs_yes_and_records_what_it_replaced() {
    let repo = base(0, None, &["gitlab", "backup"]).build();
    let s = "rg-force";
    let old = head(&repo);
    repo.git(&["commit", "--amend", "-m", "amended"]);
    let new = head(&repo);
    assert_ne!(old, new);
    let mut args = vec![
        "push",
        "--all-targets",
        "--resolve",
        "origin=force",
        "--resolve",
        "gitlab=force",
        "--resolve",
        "backup=force",
    ];

    let (code, v) = run(&repo, s, &args);
    assert_eq!(code, 3, "{v}");
    assert!(v["needs_confirmation"]["prompt"].is_string(), "{v}");
    assert_eq!(tip(&repo, "origin", "main"), old, "nothing was pushed");

    args.push("--yes");
    let (code, v) = run(&repo, s, &args);
    assert_eq!(code, 0, "{v}");
    let out = outcome(&v);
    for remote in ["origin", "gitlab", "backup"] {
        let t = result(&out, remote);
        assert!(matches!(t.status, TargetStatus::Pushed { .. }), "{t:?}");
        assert_eq!(
            t.replaced.map(|o| o.to_string()),
            Some(old.clone()),
            "{remote}"
        );
        assert!(t.recovery.is_some(), "{remote}");
    }
    assert_all_at_head(&repo);
}

#[test]
fn a_conflict_stops_with_exit_4_and_only_continue_resumes_it() {
    let repo = conflicting();
    let s = "rg-conflict";
    let old_origin = tip(&repo, "origin", "main");
    let (code, v) = run(
        &repo,
        s,
        &["push", "--all-targets", "--resolve", "gitlab=merge"],
    );
    assert_eq!(code, 4, "{v}");
    let out = outcome(&v);
    assert_eq!(out.status, GroupRunStatus::Conflicted);
    assert!(out.pending);
    assert_eq!(out.stopped.as_ref().unwrap().remote, "gitlab");
    assert_eq!(out.conflicts, ["shared.txt"]);
    assert_eq!(
        tip(&repo, "origin", "main"),
        old_origin,
        "nothing pushed yet"
    );

    // A second run while one waits is refused.
    let (code, v) = run(&repo, s, &["push", "--all-targets"]);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["refused"]["reason"], "remote_group_pending");
    assert_eq!(v["refused"]["run_id"], out.run_id);

    // The saved run shows in the status; not ready while the merge is open.
    let (code, v) = run(&repo, s, &["remote-group", "status"]);
    assert_eq!(code, 0);
    let st: RemoteGroupStatus = serde_json::from_value(v).unwrap();
    assert!(!st.pending.unwrap().ready_to_continue);
    // Continue is refused while unresolved.
    let (code, v) = run(&repo, s, &["remote-group", "continue"]);
    assert_eq!(code, 2, "{v}");

    // Resolve and conclude with plain git; nothing is pushed by itself.
    std::fs::write(repo.root().join("shared.txt"), "both\n").unwrap();
    repo.git(&["add", "shared.txt"]);
    repo.git(&["commit", "--no-edit"]);
    assert_eq!(tip(&repo, "origin", "main"), old_origin);

    let (code, v) = run(
        &repo,
        s,
        &["remote-group", "continue", "--run", &out.run_id],
    );
    assert_eq!(code, 0, "{v}");
    assert_eq!(outcome(&v).status, GroupRunStatus::Done);
    assert_all_at_head(&repo);
    let (_, v) = run(&repo, s, &["remote-group", "status"]);
    let st: RemoteGroupStatus = serde_json::from_value(v).unwrap();
    assert!(st.pending.is_none());
}

#[test]
fn abort_drops_the_run_and_the_merge_it_started() {
    let repo = conflicting();
    let s = "rg-abort";
    let (code, v) = run(
        &repo,
        s,
        &["push", "--all-targets", "--resolve", "gitlab=merge"],
    );
    assert_eq!(code, 4, "{v}");
    let out = outcome(&v);
    let (code, v) = run(&repo, s, &["remote-group", "abort", "--run", &out.run_id]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(outcome(&v).status, GroupRunStatus::Aborted);
    assert!(
        !repo.root().join(".git/MERGE_HEAD").exists(),
        "the merge was aborted"
    );
    let (_, v) = run(&repo, s, &["remote-group", "status"]);
    let st: RemoteGroupStatus = serde_json::from_value(v).unwrap();
    assert!(st.pending.is_none());
    // Nothing waits any more: there is nothing to continue.
    let (code, v) = run(&repo, s, &["remote-group", "continue"]);
    assert_eq!(code, 2, "{v}");
}

#[test]
fn to_pushes_to_the_named_remotes_for_this_push_only() {
    // Only the primary is configured: no push targets.
    let repo = base(1, None, &[]).build();
    let s = "rg-to";
    let config_before = repo.git(&["config", "--local", "--list"]);
    let (code, v) = run(
        &repo,
        s,
        &["push", "--to", "gitlab", "--to", "backup:feat-x"],
    );
    assert_eq!(code, 0, "{v}");
    let out = outcome(&v);
    assert_eq!(out.targets.len(), 2);
    assert_eq!(tip(&repo, "gitlab", "main"), head(&repo));
    assert_eq!(tip(&repo, "backup", "feat-x"), head(&repo));
    assert_ne!(
        tip(&repo, "origin", "main"),
        head(&repo),
        "origin not asked"
    );
    assert!(matches!(
        result(&out, "backup").status,
        TargetStatus::Pushed { created: true }
    ));
    // Git's config did not change.
    assert_eq!(repo.git(&["config", "--local", "--list"]), config_before);
    assert!(repo
        .try_git(&["config", "--get-all", "branch.main.gitpoulpi-push-to"])
        .is_none_or(|s| s.trim().is_empty()));
    // The plan answers for the same selection without running anything.
    let (code, v) = run(&repo, s, &["remote-group", "plan", "--to", "origin"]);
    assert_eq!(code, 0, "{v}");
    let plan: RemoteGroupPlan = serde_json::from_value(v).unwrap();
    assert_eq!(plan.targets.len(), 1);
    assert_eq!(plan.targets[0].remote, "origin");
    assert_eq!(plan.targets[0].suggested, GroupAction::Push);
}

#[test]
fn pull_all_sources_integrates_the_primary_and_the_pull_source() {
    let repo = base(0, None, &["gitlab"]).build();
    repo.git(&[
        "config",
        "--add",
        "branch.main.gitpoulpi-pull-from",
        "gitlab",
    ]);
    repo.push_from_elsewhere_to("origin", "main", "from origin");
    repo.push_from_elsewhere_to("gitlab", "main", "from gitlab");
    let s = "rg-pull";
    let (code, v) = run(&repo, s, &["pull", "--all-sources"]);
    assert_eq!(code, 0, "{v}");
    let out = outcome(&v);
    assert_eq!(out.mode, GroupMode::Pull);
    assert_eq!(out.status, GroupRunStatus::Done);
    assert_eq!(out.targets.len(), 2);
    for t in &out.targets {
        assert_eq!(t.status, TargetStatus::Integrated, "{t:?}");
    }
    let log = repo.git(&["log", "--format=%s"]);
    assert!(
        log.contains("from origin") && log.contains("from gitlab"),
        "{log}"
    );
    // Nothing was pushed.
    assert_ne!(tip(&repo, "backup", "main"), head(&repo));
}

#[test]
fn retry_runs_the_remote_that_was_unreachable_again() {
    let repo = base(1, None, &["gitlab"]).build();
    repo.git(&["remote", "add", "broken", "/nonexistent/place.git"]);
    let s = "rg-retry";
    let (code, v) = run(
        &repo,
        s,
        &[
            "push",
            "--to",
            "origin",
            "--to",
            "broken",
            "--to",
            "backup",
            "--no-preflight",
        ],
    );
    assert_eq!(code, 4, "{v}");
    let out = outcome(&v);
    assert_eq!(out.status, GroupRunStatus::Problems);
    assert!(matches!(
        result(&out, "origin").status,
        TargetStatus::Pushed { .. }
    ));
    assert!(matches!(
        result(&out, "broken").status,
        TargetStatus::Failed { .. }
    ));
    assert!(matches!(
        result(&out, "backup").status,
        TargetStatus::Pushed { .. }
    ));
    assert!(out.pending);

    // Fix the remote, then retry: only the failed one runs.
    let place = tempfile::tempdir().unwrap();
    let bare = place.path().join("broken.git");
    repo.git(&["init", "--bare", "-q", bare.to_str().unwrap()]);
    repo.git(&["remote", "set-url", "broken", bare.to_str().unwrap()]);
    let (code, v) = run(&repo, s, &["remote-group", "retry", "--run", &out.run_id]);
    assert_eq!(code, 0, "{v}");
    let retried = outcome(&v);
    assert_eq!(retried.status, GroupRunStatus::Done);
    assert!(matches!(
        result(&retried, "broken").status,
        TargetStatus::Pushed { .. }
    ));
    assert_eq!(
        repo.git(&[
            "--git-dir",
            bare.to_str().unwrap(),
            "rev-parse",
            "refs/heads/main"
        ])
        .trim(),
        head(&repo)
    );
}
