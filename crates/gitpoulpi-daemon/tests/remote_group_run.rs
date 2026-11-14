//! `RemoteGroupRun`: integrate first, then push to every target in order with
//! a status each; conflicts save the run and nothing continues by itself;
//! cancel between targets and during a push; restart; retry; journal; pull
//! mode. Both backends, 2-3 bare remotes.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{
    GroupAction, GroupRunStatus, Integration, JournalState, OperationKind, PendingGroup,
    PushRejectReason, RemoteGroupOutcome, RemoteGroupPlan, RemoteGroupStatus, RepoId, RepoRef,
    RunTarget, TargetResult, TargetStatus,
};
use gitpoulpi_core::{GitBackend, TypedError};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::transport::{read_message, write_message};
use gitpoulpi_ipc::{Client, Event, JobState, Request, Response, ResponseError, ServerMessage};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder, SlowRemote};
use serde_json::{json, Value};

gitpoulpi_testkit::isolate_binary!();

const SESSION: &str = "remote-group-run-test";

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
    });
}

struct Rig {
    handle: Option<gitpoulpi_daemon::DaemonHandle>,
    client: Client,
    repo: FixtureRepo,
    id: RepoId,
    backend: Arc<dyn GitBackend>,
    next: u64,
}

impl Rig {
    async fn new(backend: Arc<dyn GitBackend>, repo: FixtureRepo) -> Rig {
        setup_env();
        let handle = Daemon::new(SESSION).spawn(backend.clone()).await.unwrap();
        let client = Client::connect(&handle.endpoint()).await.unwrap();
        let id = repo.repo_ref().id;
        Rig {
            handle: Some(handle),
            client,
            repo,
            id,
            backend,
            next: 1,
        }
    }

    /// Stops the daemon and starts a fresh one over the same repository.
    async fn restart(&mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
        let handle = Daemon::new(SESSION)
            .spawn(self.backend.clone())
            .await
            .unwrap();
        self.client = Client::connect(&handle.endpoint()).await.unwrap();
        self.handle = Some(handle);
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

    async fn plan(&mut self, args: Value) -> RemoteGroupPlan {
        let v = self
            .call(OperationKind::RemoteGroupPlan, args)
            .await
            .unwrap_or_else(|e| panic!("plan failed: {}", e.message));
        serde_json::from_value(v).unwrap()
    }

    async fn run_raw(&mut self, args: Value) -> Result<RemoteGroupOutcome, ResponseError> {
        self.call(OperationKind::RemoteGroupRun, args)
            .await
            .map(|v| serde_json::from_value(v).unwrap())
    }

    async fn run(&mut self, args: Value) -> RemoteGroupOutcome {
        self.run_raw(args)
            .await
            .unwrap_or_else(|e| panic!("run failed: {}", e.message))
    }

    async fn status(&mut self, branch: &str) -> RemoteGroupStatus {
        let v = self
            .call(
                OperationKind::RemoteGroupStatus,
                json!({ "branch": branch }),
            )
            .await
            .unwrap();
        serde_json::from_value(v).unwrap()
    }

    async fn pending(&mut self, branch: &str) -> Option<PendingGroup> {
        self.status(branch).await.pending
    }

    async fn journal(&mut self) -> JournalState {
        serde_json::from_value(
            self.call(OperationKind::JournalState, json!({}))
                .await
                .unwrap(),
        )
        .unwrap()
    }

    /// A plan's suggested decisions, with `over` replacing some actions.
    fn decisions(plan: &RemoteGroupPlan, over: &[(&str, GroupAction)]) -> Vec<RunTarget> {
        plan.targets
            .iter()
            .map(|t| {
                let action = over
                    .iter()
                    .find(|(r, _)| *r == t.remote)
                    .map_or(t.suggested, |(_, a)| *a);
                RunTarget::from_planned(t, action)
            })
            .collect()
    }

    fn tip(&self, rev: &str) -> String {
        self.repo.git(&["rev-parse", rev]).trim().to_string()
    }

    fn remote_tip(&self, remote: &str, branch: &str) -> String {
        let args = ["rev-parse", &format!("refs/heads/{branch}")];
        if remote == "origin" {
            self.repo.git_in_remote(&args)
        } else {
            self.repo.git_in_named_remote(remote, &args)
        }
        .trim()
        .to_string()
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

/// `main` on origin (upstream), gitlab and backup, all equal to local, with
/// gitlab and backup as extra push targets; then `n` local commits.
fn all_in_sync(local_commits: usize) -> FixtureRepoBuilder {
    base_with_file(local_commits, None)
}

fn base_with_file(local_commits: usize, file: Option<(&str, &str)>) -> FixtureRepoBuilder {
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
        .push_targets("main", &["gitlab", "backup"])
        .unwrap();
    for i in 0..local_commits {
        b = b.commit(&format!("local {i}")).unwrap();
    }
    b
}

fn result<'a>(out: &'a RemoteGroupOutcome, remote: &str) -> &'a TargetResult {
    out.targets
        .iter()
        .find(|t| t.remote == remote)
        .unwrap_or_else(|| panic!("no result for {remote}: {out:?}"))
}

/// Plays another user editing `file` on `remote`'s `main` (a conflicting
/// change when local edited it too).
fn edit_elsewhere(repo: &FixtureRepo, remote: &str, file: &str, contents: &str) {
    let bare = if remote == "origin" {
        repo.git_in_remote(&["rev-parse", "--absolute-git-dir"])
    } else {
        repo.git_in_named_remote(remote, &["rev-parse", "--absolute-git-dir"])
    };
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
    run(&dir, &["commit", "-am", "theirs"]);
    run(&dir, &["push", "origin", "main"]);
}

fn assert_all_at(rig: &Rig, name: &str, want: &str) {
    for remote in ["origin", "gitlab", "backup"] {
        assert_eq!(rig.remote_tip(remote, "main"), want, "{name}: {remote}");
    }
}

#[tokio::test]
async fn all_fast_forward_pushes_everywhere_and_is_not_journaled() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend, all_in_sync(2).build()).await;
        let before = rig.journal().await.undo_count;
        let plan = rig.plan(json!({ "branch": "main" })).await;
        let out = rig
            .run(json!({ "branch": "main", "targets": Rig::decisions(&plan, &[]) }))
            .await;
        assert_eq!(out.status, GroupRunStatus::Done, "{name}: {out:?}");
        let order: Vec<_> = out.targets.iter().map(|t| t.remote.as_str()).collect();
        assert_eq!(order, ["origin", "gitlab", "backup"]);
        for t in &out.targets {
            assert_eq!(t.status, TargetStatus::Pushed { created: false }, "{name}");
        }
        assert!(!out.pending);
        assert_all_at(&rig, name, &rig.tip("HEAD"));
        assert_eq!(
            rig.journal().await.undo_count,
            before,
            "{name}: a pure push is not journaled"
        );
        assert!(rig.pending("main").await.is_none());
        rig.stop().await;
    }
}

#[tokio::test]
async fn diverged_remote_is_merged_then_everything_is_pushed_and_undo_takes_the_merge_back() {
    for (name, backend) in backends() {
        let repo = all_in_sync(1).build();
        repo.push_from_elsewhere_to("gitlab", "main", "theirs on gitlab");
        let mut rig = Rig::new(backend, repo).await;
        let before_tip = rig.tip("HEAD");
        let before = rig.journal().await.undo_count;
        let plan = rig.plan(json!({ "branch": "main" })).await;
        let decisions = Rig::decisions(&plan, &[]);
        assert_eq!(
            decisions
                .iter()
                .find(|d| d.remote == "gitlab")
                .unwrap()
                .action,
            GroupAction::MergeFrom
        );
        let out = rig
            .run(json!({ "branch": "main", "targets": decisions }))
            .await;
        assert_eq!(out.status, GroupRunStatus::Done, "{name}: {out:?}");
        assert!(
            result(&out, "gitlab").integrated.is_some(),
            "{name}: {out:?}"
        );
        let merged = rig.tip("HEAD");
        assert_ne!(merged, before_tip);
        assert_all_at(&rig, name, &merged);
        // One entry for the local part; the pushes are not undone.
        let state = rig.journal().await;
        assert_eq!(state.undo_count, before + 1, "{name}");
        assert!(
            state.undo.as_ref().unwrap().label.contains("Grouped push"),
            "{name}: {:?}",
            state.undo
        );
        let undone = rig
            .call(OperationKind::Undo, json!({ "confirmed": true }))
            .await
            .unwrap();
        assert_eq!(undone["applied"], true, "{name}: {undone}");
        assert_eq!(rig.tip("HEAD"), before_tip, "{name}");
        assert_eq!(
            rig.remote_tip("origin", "main"),
            merged,
            "{name}: undo never touches the remotes"
        );
        rig.stop().await;
    }
}

/// local edits shared.txt, gitlab edits it differently.
async fn conflicting_rig(backend: Arc<dyn GitBackend>) -> (Rig, RemoteGroupOutcome) {
    let repo = base_with_file(0, Some(("shared.txt", "base\n")))
        .write_worktree("shared.txt", b"mine\n")
        .unwrap()
        .stage("shared.txt")
        .unwrap()
        .commit("mine")
        .unwrap()
        .build();
    edit_elsewhere(&repo, "gitlab", "shared.txt", "theirs\n");
    let mut rig = Rig::new(backend, repo).await;
    let plan = rig.plan(json!({ "branch": "main" })).await;
    let decisions = Rig::decisions(&plan, &[]);
    let gl = decisions.iter().find(|d| d.remote == "gitlab").unwrap();
    assert_eq!(gl.action, GroupAction::MergeFrom);
    let out = rig
        .run(json!({ "branch": "main", "targets": decisions }))
        .await;
    (rig, out)
}

#[tokio::test]
async fn a_conflict_saves_the_run_and_only_an_explicit_continue_pushes() {
    for (name, backend) in backends() {
        let (mut rig, out) = conflicting_rig(backend).await;
        assert_eq!(out.status, GroupRunStatus::Conflicted, "{name}: {out:?}");
        assert!(out.pending);
        assert_eq!(out.conflicts, ["shared.txt"], "{name}");
        let stopped = out.stopped.clone().expect("stopped");
        assert_eq!(stopped.remote, "gitlab");
        // Nothing was pushed yet: integrations come first.
        let old_origin = rig.remote_tip("origin", "main");
        assert_ne!(old_origin, rig.tip("HEAD"));
        // A second run while one waits is refused, typed.
        let again = rig
            .run_raw(json!({
                "branch": "main",
                "targets": [{"remote": "origin", "branch": "main", "action": "push"}]
            }))
            .await
            .unwrap_err();
        assert!(
            matches!(again.kind, Some(TypedError::RemoteGroupPending { .. })),
            "{name}: {again:?}"
        );
        // Not ready while the merge is open.
        let p = rig.pending("main").await.unwrap();
        assert!(!p.ready_to_continue, "{name}");
        // Continue is refused while unresolved.
        let early = rig
            .run_raw(json!({ "resume": "continue", "expect_run_id": out.run_id }))
            .await;
        assert!(early.is_err(), "{name}");
        // Resolve and conclude the merge with plain git.
        std::fs::write(rig.repo.root().join("shared.txt"), "both\n").unwrap();
        rig.repo.git(&["add", "shared.txt"]);
        rig.repo.git(&["commit", "--no-edit"]);
        // The flag turns on; nothing is pushed by itself.
        let p = rig.pending("main").await.unwrap();
        assert!(p.ready_to_continue, "{name}: {p:?}");
        assert_eq!(rig.remote_tip("origin", "main"), old_origin, "{name}");
        // Wrong run id is refused.
        assert!(rig
            .run_raw(json!({ "resume": "continue", "expect_run_id": "nope" }))
            .await
            .is_err());
        let done = rig
            .run(json!({ "resume": "continue", "expect_run_id": out.run_id }))
            .await;
        assert_eq!(done.status, GroupRunStatus::Done, "{name}: {done:?}");
        assert_all_at(&rig, name, &rig.tip("HEAD"));
        assert!(rig.pending("main").await.is_none(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn after_a_rebase_a_target_holding_the_old_commits_needs_a_decision_not_a_silent_force() {
    for (name, backend) in backends() {
        let repo = base_with_file(0, Some(("shared.txt", "base\n")))
            .write_worktree("shared.txt", b"mine\n")
            .unwrap()
            .stage("shared.txt")
            .unwrap()
            .commit("mine")
            .unwrap()
            .build();
        // backup already holds the commit that the rebase will replace.
        repo.git(&["push", "backup", "main"]);
        edit_elsewhere(&repo, "gitlab", "shared.txt", "theirs\n");
        let mut rig = Rig::new(backend, repo).await;
        let plan = rig.plan(json!({ "branch": "main" })).await;
        let decisions = Rig::decisions(&plan, &[("gitlab", GroupAction::RebaseOnto)]);
        let backup_before = rig.remote_tip("backup", "main");
        let out = rig
            .run(json!({ "branch": "main", "targets": decisions }))
            .await;
        assert_eq!(out.status, GroupRunStatus::Conflicted, "{name}: {out:?}");
        std::fs::write(rig.repo.root().join("shared.txt"), "both\n").unwrap();
        rig.repo.git(&["add", "shared.txt"]);
        let status = std::process::Command::new("git")
            .args(["-c", "core.editor=true", "rebase", "--continue"])
            .current_dir(rig.repo.root())
            .env("GIT_EDITOR", "true")
            .status()
            .unwrap();
        assert!(status.success());
        let stop = rig
            .run(json!({ "resume": "continue", "expect_run_id": out.run_id }))
            .await;
        assert_eq!(
            stop.status,
            GroupRunStatus::NeedsResolution,
            "{name}: {stop:?}"
        );
        let plan = stop.plan.clone().expect("a fresh plan");
        assert!(plan.targets.iter().any(|t| t.remote == "backup"));
        assert_eq!(rig.remote_tip("backup", "main"), backup_before, "{name}");
        assert!(stop.pending);
        // The caller decides: force backup (with the lease the plan saw).
        let backup = plan.targets.iter().find(|t| t.remote == "backup").unwrap();
        let answer = RunTarget::from_planned(backup, GroupAction::ForceWithLease);
        let done = rig
            .run(json!({
                "resume": "continue",
                "expect_run_id": out.run_id,
                "targets": [answer]
            }))
            .await;
        assert_eq!(done.status, GroupRunStatus::Done, "{name}: {done:?}");
        assert_eq!(
            result(&done, "backup").replaced.map(|o| o.to_hex()),
            Some(backup_before)
        );
        assert_all_at(&rig, name, &rig.tip("HEAD"));
        rig.stop().await;
    }
}

#[tokio::test]
async fn abort_aborts_the_merge_and_drops_the_run() {
    for (name, backend) in backends() {
        let (mut rig, out) = conflicting_rig(backend).await;
        assert_eq!(out.status, GroupRunStatus::Conflicted);
        let started = out.stopped.as_ref().unwrap().pre_tip.to_hex();
        let aborted = rig
            .run(json!({ "resume": "abort", "expect_run_id": out.run_id }))
            .await;
        assert_eq!(aborted.status, GroupRunStatus::Aborted, "{name}");
        assert!(!aborted.pending);
        assert_eq!(rig.tip("HEAD"), started, "{name}: back where it began");
        assert!(rig
            .repo
            .try_git(&["rev-parse", "--verify", "-q", "MERGE_HEAD"])
            .is_none());
        assert!(rig.pending("main").await.is_none(), "{name}");
        // Nothing waits any more: a new run is accepted.
        let again = rig
            .run_raw(json!({
                "branch": "main",
                "targets": [{"remote": "backup", "branch": "main", "action": "push"}]
            }))
            .await
            .unwrap();
        assert_eq!(again.status, GroupRunStatus::Done, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_saved_run_survives_a_daemon_restart() {
    for (name, backend) in backends() {
        let (mut rig, out) = conflicting_rig(backend).await;
        assert_eq!(out.status, GroupRunStatus::Conflicted);
        rig.restart().await;
        let p = rig.pending("main").await.expect("state survived");
        assert_eq!(p.run_id, out.run_id, "{name}");
        std::fs::write(rig.repo.root().join("shared.txt"), "both\n").unwrap();
        rig.repo.git(&["add", "shared.txt"]);
        rig.repo.git(&["commit", "--no-edit"]);
        let done = rig.run(json!({ "resume": "continue" })).await;
        assert_eq!(done.status, GroupRunStatus::Done, "{name}: {done:?}");
        assert_all_at(&rig, name, &rig.tip("HEAD"));
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_stale_lease_is_rejected_not_overwritten() {
    for (name, backend) in backends() {
        let repo = all_in_sync(0).commit("work").unwrap().build();
        repo.git(&["push", "gitlab", "main"]);
        repo.git(&["commit", "--amend", "-m", "work (reworded)"]);
        let mut rig = Rig::new(backend, repo).await;
        let plan = rig.plan(json!({ "branch": "main" })).await;
        let decisions = Rig::decisions(&plan, &[]);
        let gl = decisions.iter().find(|d| d.remote == "gitlab").unwrap();
        assert_eq!(gl.action, GroupAction::ForceWithLease, "{name}");
        // Someone pushes to gitlab after the plan looked.
        rig.repo
            .push_from_elsewhere_to("gitlab", "main", "after the plan");
        let theirs = rig.remote_tip("gitlab", "main");
        let out = rig
            .run(json!({ "branch": "main", "targets": decisions }))
            .await;
        assert_eq!(out.status, GroupRunStatus::Problems, "{name}: {out:?}");
        assert_eq!(
            result(&out, "gitlab").status,
            TargetStatus::Rejected {
                reason: PushRejectReason::StaleLease
            },
            "{name}: {out:?}"
        );
        assert_eq!(rig.remote_tip("gitlab", "main"), theirs, "{name}");
        // The others went through.
        assert!(matches!(
            result(&out, "backup").status,
            TargetStatus::Pushed { .. }
        ));
        // A rejection is not retryable: no saved run.
        assert!(!out.pending, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_force_records_the_replaced_commit_and_the_recovery_command() {
    for (name, backend) in backends() {
        let repo = all_in_sync(0).commit("work").unwrap().build();
        repo.git(&["push", "gitlab", "main"]);
        repo.git(&["commit", "--amend", "-m", "work (reworded)"]);
        let mut rig = Rig::new(backend, repo).await;
        let old_gitlab = rig.remote_tip("gitlab", "main");
        let plan = rig.plan(json!({ "branch": "main" })).await;
        let out = rig
            .run(json!({ "branch": "main", "targets": Rig::decisions(&plan, &[]) }))
            .await;
        assert_eq!(out.status, GroupRunStatus::Done, "{name}: {out:?}");
        let gl = result(&out, "gitlab");
        assert_eq!(gl.replaced.map(|o| o.to_hex()), Some(old_gitlab.clone()));
        let recovery = gl.recovery.clone().unwrap();
        assert!(
            recovery.contains(&old_gitlab)
                && recovery.contains("--force-with-lease=refs/heads/main:"),
            "{name}: {recovery}"
        );
        assert_eq!(rig.remote_tip("gitlab", "main"), rig.tip("HEAD"));
        assert!(result(&out, "origin").replaced.is_none());
        rig.stop().await;
    }
}

#[tokio::test]
async fn an_unconfigured_remote_and_a_failing_one_do_not_stop_the_rest_and_retry_runs_it_again() {
    for (name, backend) in backends() {
        let repo = all_in_sync(1).build();
        repo.git(&["remote", "add", "broken", "/nonexistent/place.git"]);
        let mut rig = Rig::new(backend, repo).await;
        let t = |remote: &str| json!({"remote": remote, "branch": "main", "action": "push"});
        let out = rig
            .run(json!({
                "branch": "main",
                "targets": [t("origin"), t("broken"), t("ghost"), t("backup")]
            }))
            .await;
        assert_eq!(out.status, GroupRunStatus::Problems, "{name}: {out:?}");
        assert!(matches!(
            result(&out, "origin").status,
            TargetStatus::Pushed { .. }
        ));
        assert!(matches!(
            result(&out, "broken").status,
            TargetStatus::Failed { .. }
        ));
        assert_eq!(result(&out, "ghost").status, TargetStatus::RemoteMissing);
        assert!(matches!(
            result(&out, "backup").status,
            TargetStatus::Pushed { .. }
        ));
        assert!(out.pending, "{name}: the failed one waits for a retry");
        // Fix the remote and retry: only the failed one runs.
        let url = rig
            .repo
            .git_in_named_remote("gitlab", &["rev-parse", "--absolute-git-dir"]);
        rig.repo.git(&["remote", "set-url", "broken", url.trim()]);
        let retried = rig
            .run(json!({ "retry": true, "expect_run_id": out.run_id }))
            .await;
        assert!(
            matches!(
                result(&retried, "broken").status,
                TargetStatus::Pushed { .. }
            ),
            "{name}: {retried:?}"
        );
        assert_eq!(
            result(&retried, "ghost").status,
            TargetStatus::RemoteMissing
        );
        assert_eq!(rig.remote_tip("gitlab", "main"), rig.tip("HEAD"));
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_twin_remote_with_the_same_url_is_pushed_once() {
    for (name, backend) in backends() {
        let repo = all_in_sync(1).build();
        let url = repo.git_in_named_remote("backup", &["rev-parse", "--absolute-git-dir"]);
        repo.git(&["remote", "add", "twin", url.trim()]);
        let mut rig = Rig::new(backend, repo).await;
        let t = |remote: &str| json!({"remote": remote, "branch": "main", "action": "push"});
        let out = rig
            .run(json!({ "branch": "main", "targets": [t("backup"), t("twin")] }))
            .await;
        assert_eq!(out.status, GroupRunStatus::Done, "{name}: {out:?}");
        assert!(matches!(
            result(&out, "backup").status,
            TargetStatus::Pushed { .. }
        ));
        let twin = result(&out, "twin");
        assert_eq!(twin.status, TargetStatus::UpToDate, "{name}");
        assert!(twin.detail.as_deref().unwrap().contains("backup"));
        rig.stop().await;
    }
}

// -- pull mode --------------------------------------------------------------

#[tokio::test]
async fn a_pull_group_integrates_two_sources_as_one_journaled_entry() {
    for (name, backend) in backends() {
        let repo = all_in_sync(0).build();
        repo.push_from_elsewhere_to("origin", "main", "from origin");
        repo.push_from_elsewhere_to("gitlab", "main", "from gitlab");
        let mut rig = Rig::new(backend, repo).await;
        let before = rig.journal().await.undo_count;
        let before_tip = rig.tip("HEAD");
        let plan = rig
            .plan(json!({ "branch": "main", "mode": "pull", "targets": [
                {"remote": "origin", "branch": "main"},
                {"remote": "gitlab", "branch": "main"}
            ]}))
            .await;
        let decisions = Rig::decisions(&plan, &[]);
        // Each source was ahead of local alone; after the first one the
        // second has diverged, and a fast-forward that an earlier
        // integration of this run made impossible becomes a merge.
        let out = rig
            .run(json!({ "branch": "main", "mode": "pull", "targets": decisions }))
            .await;
        assert_eq!(out.status, GroupRunStatus::Done, "{name}: {out:?}");
        for t in &out.targets {
            assert_eq!(t.status, TargetStatus::Integrated, "{name}: {t:?}");
        }
        assert_eq!(result(&out, "gitlab").integrated, Some(Integration::Merged));
        assert_eq!(result(&out, "gitlab").action, GroupAction::MergeFrom);
        // Both sources' commits are in.
        let log = rig.repo.git(&["log", "--format=%s"]);
        assert!(
            log.contains("from origin") && log.contains("from gitlab"),
            "{name}: {log}"
        );
        // Nothing was pushed.
        assert_ne!(rig.remote_tip("backup", "main"), rig.tip("HEAD"));
        let state = rig.journal().await;
        assert_eq!(state.undo_count, before + 1, "{name}");
        assert!(state.undo.unwrap().label.contains("Grouped pull"));
        let undone = rig
            .call(OperationKind::Undo, json!({ "confirmed": true }))
            .await
            .unwrap();
        assert_eq!(undone["applied"], true, "{name}: {undone}");
        assert_eq!(rig.tip("HEAD"), before_tip, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_pull_group_follows_a_rebase_pull_mode_for_a_fast_forward_made_impossible() {
    for (name, backend) in backends() {
        let repo = all_in_sync(0).build();
        repo.push_from_elsewhere_to("origin", "main", "from origin");
        repo.push_from_elsewhere_to("gitlab", "main", "from gitlab");
        let mut rig = Rig::new(backend, repo).await;
        let plan = rig
            .plan(json!({ "branch": "main", "mode": "pull", "targets": [
                {"remote": "origin", "branch": "main"},
                {"remote": "gitlab", "branch": "main"}
            ]}))
            .await;
        let out = rig
            .run(json!({
                "branch": "main", "mode": "pull", "prefer_rebase": true,
                "targets": Rig::decisions(&plan, &[])
            }))
            .await;
        assert_eq!(out.status, GroupRunStatus::Done, "{name}: {out:?}");
        assert_eq!(
            result(&out, "gitlab").integrated,
            Some(Integration::Rebased)
        );
        assert_eq!(result(&out, "gitlab").action, GroupAction::RebaseOnto);
        assert_eq!(
            rig.repo.git(&["rev-list", "--merges", "HEAD"]).trim(),
            "",
            "{name}: rebased, no merge commit"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_group_is_refused_while_an_auto_stash_is_pending() {
    for (name, backend) in backends() {
        let repo = all_in_sync(1).build();
        let mut rig = Rig::new(backend, repo).await;
        write_pending_record(&rig);
        let refused = rig
            .run_raw(json!({ "branch": "main", "targets": [push_target("origin")] }))
            .await
            .unwrap_err();
        assert!(
            matches!(refused.kind, Some(TypedError::PendingAutoStash { .. })),
            "{name}: {refused:?}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn an_integration_blocked_by_local_changes_asks_for_an_auto_stash() {
    for (name, backend) in backends() {
        let repo = all_in_sync(0)
            .commit_file("shared.txt", "base\n", "shared")
            .unwrap()
            .build();
        repo.git(&["push", "origin", "main"]);
        repo.git(&["push", "gitlab", "main"]);
        repo.git(&["push", "backup", "main"]);
        edit_elsewhere(&repo, "gitlab", "shared.txt", "theirs\n");
        let mut rig = Rig::new(backend, repo).await;
        let plan = rig.plan(json!({ "branch": "main" })).await;
        let decisions = Rig::decisions(&plan, &[("gitlab", GroupAction::FastForwardFrom)]);
        // A tracked file the incoming commit rewrites, edited here.
        std::fs::write(rig.repo.root().join("shared.txt"), "mine, uncommitted\n").unwrap();
        let blocked = rig
            .run_raw(json!({
                "branch": "main", "targets": decisions, "autostash": "ask"
            }))
            .await;
        let err = blocked.unwrap_err();
        assert!(
            matches!(err.kind, Some(TypedError::NeedsAutoStash { .. })),
            "{name}: {err:?}"
        );
        // Nothing was pushed or saved.
        assert!(rig.pending("main").await.is_none(), "{name}");
        // With the auto-stash granted, the integration runs on the stashed
        // tree (the pop then conflicts, as the same file changed: reported,
        // the stash kept).
        let again = rig
            .run_raw(json!({
                "branch": "main",
                "targets": Rig::decisions(&plan, &[("gitlab", GroupAction::FastForwardFrom)]),
                "autostash": "always"
            }))
            .await
            .unwrap_err();
        assert!(again.message.contains("stash"), "{name}: {again:?}");
        assert!(
            !rig.repo.git(&["stash", "list"]).trim().is_empty(),
            "{name}"
        );
        assert_eq!(rig.tip("HEAD"), rig.remote_tip("gitlab", "main"), "{name}");
        rig.stop().await;
    }
}

// -- cancel ----------------------------------------------------------------

/// A raw connection: requests that become jobs, their events, the job list.
struct Conn {
    read: tokio::net::tcp::OwnedReadHalf,
    write: tokio::net::tcp::OwnedWriteHalf,
    next: u64,
    responses: HashMap<u64, Response>,
    events: Vec<Event>,
}

impl Conn {
    async fn connect(endpoint: &gitpoulpi_ipc::Endpoint) -> Self {
        let stream = Client::connect(endpoint).await.unwrap().into_stream();
        let (read, write) = stream.into_split();
        Self {
            read,
            write,
            next: 1,
            responses: HashMap::new(),
            events: Vec::new(),
        }
    }

    async fn send(&mut self, repo: Option<&RepoRef>, command: OperationKind, args: Value) -> u64 {
        let id = self.next;
        self.next += 1;
        write_message(
            &mut self.write,
            &Request {
                id,
                session: SESSION.into(),
                repo: repo.map(|r| r.id.clone()),
                command,
                args,
            },
        )
        .await
        .unwrap();
        id
    }

    async fn pump(&mut self) {
        let message: ServerMessage =
            tokio::time::timeout(Duration::from_secs(60), read_message(&mut self.read))
                .await
                .expect("the daemon answers")
                .unwrap()
                .expect("connection open");
        match message {
            ServerMessage::Response(r) => {
                self.responses.insert(r.request_id, r);
            }
            ServerMessage::Event(e) => self.events.push(e),
        }
    }

    async fn wait(&mut self, id: u64) -> Response {
        while !self.responses.contains_key(&id) {
            self.pump().await;
        }
        self.responses.remove(&id).unwrap()
    }

    async fn call(&mut self, command: OperationKind, args: Value) -> Response {
        let id = self.send(None, command, args).await;
        self.wait(id).await
    }

    async fn running_job(&mut self, kind: OperationKind) -> u64 {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let r = self.call(OperationKind::JobList, json!({})).await;
            let jobs: Vec<gitpoulpi_ipc::JobInfo> =
                serde_json::from_value(r.result.unwrap()).unwrap();
            if let Some(j) = jobs
                .iter()
                .find(|j| j.kind == kind && j.state == JobState::Running)
            {
                return j.id;
            }
            assert!(Instant::now() < deadline, "no running job");
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
    }
}

fn wait_for(what: &str, mut done: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !done() {
        assert!(Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(30));
    }
}

/// The record an auto-stash leaves in `repo-state.toml` while its stash waits.
fn write_pending_record(rig: &Rig) {
    use gitpoulpi_session::{PendingAutoStash, PendingAutoStashState, RepoState};
    let path = RepoState::path_for(
        rig.repo.root(),
        gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION,
    );
    let mut state = RepoState::load(&path).unwrap_or_default();
    state.pending_auto_stash = Some(PendingAutoStash {
        stash_id: gitpoulpi_core::domain::StashId(0),
        stash_commit: None,
        reason: OperationKind::Pull,
        created_at: 0,
        state: PendingAutoStashState::Applied,
        error: None,
        after_operation: false,
    });
    state.save(&path).unwrap();
}

fn push_target(remote: &str) -> Value {
    json!({"remote": remote, "branch": "main", "action": "push"})
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cancel_during_a_push_leaves_it_unknown_the_rest_not_attempted_and_the_run_kept() {
    for (name, backend) in backends() {
        setup_env();
        let slow = SlowRemote::start();
        let repo = all_in_sync(1).build();
        repo.git(&["remote", "add", "slow", &slow.url()]);
        let repo_ref = repo.repo_ref();
        let origin_ok = {
            let repo = &repo;
            move || {
                repo.git_in_remote(&["rev-parse", "refs/heads/main"])
                    .trim()
                    .to_string()
            }
        };
        let want = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
        let handle = Daemon::new(SESSION).spawn(backend.clone()).await.unwrap();
        let mut conn = Conn::connect(&handle.endpoint()).await;
        let id = conn
            .send(
                Some(&repo_ref),
                OperationKind::RemoteGroupRun,
                json!({
                    "branch": "main",
                    "targets": [push_target("origin"), push_target("slow"), push_target("backup")]
                }),
            )
            .await;
        assert!(
            slow.wait_accepted(1, Duration::from_secs(30)),
            "{name}: the slow push is in flight"
        );
        let job = conn.running_job(OperationKind::RemoteGroupRun).await;
        conn.call(OperationKind::JobCancel, json!({ "id": job }))
            .await;
        let response = conn.wait(id).await;
        let out: RemoteGroupOutcome = serde_json::from_value(response.result.unwrap()).unwrap();
        assert_eq!(out.status, GroupRunStatus::Canceled, "{name}: {out:?}");
        assert!(matches!(
            result(&out, "origin").status,
            TargetStatus::Pushed { .. }
        ));
        assert_eq!(result(&out, "slow").status, TargetStatus::Unknown, "{name}");
        assert_eq!(
            result(&out, "backup").status,
            TargetStatus::NotAttempted,
            "{name}"
        );
        assert!(out.pending);
        assert_eq!(origin_ok(), want);
        // The git process is gone with the cancel.
        assert!(slow.wait_closed(1, Duration::from_secs(30)), "{name}");
        // Progress was broadcast: a row for the in-flight remote.
        while conn
            .events
            .iter()
            .all(|e| !matches!(e, Event::RemoteGroupProgress { remote, .. } if remote == "slow"))
        {
            conn.pump().await;
        }
        // Continue with the slow one skipped runs the rest.
        let id = conn
            .send(
                Some(&repo_ref),
                OperationKind::RemoteGroupRun,
                json!({
                    "resume": "continue",
                    "expect_run_id": out.run_id,
                    "targets": [{"remote": "slow", "branch": "main", "action": "skip"}]
                }),
            )
            .await;
        let response = conn.wait(id).await;
        let finished: RemoteGroupOutcome =
            serde_json::from_value(response.result.unwrap()).unwrap();
        assert_eq!(
            finished.status,
            GroupRunStatus::Done,
            "{name}: {finished:?}"
        );
        assert_eq!(result(&finished, "slow").status, TargetStatus::Skipped);
        assert!(matches!(
            result(&finished, "backup").status,
            TargetStatus::Pushed { .. }
        ));
        assert_eq!(
            repo.git_in_named_remote("backup", &["rev-parse", "refs/heads/main"])
                .trim(),
            want
        );
        handle.shutdown().await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cancel_between_targets_keeps_what_was_pushed_and_the_rest_not_attempted() {
    for (name, backend) in backends() {
        setup_env();
        // Each target waits this long before it starts (a test hook), so a
        // cancel can land between two of them.
        std::env::set_var("GITPOULPI_TEST_STEP_DELAY_MS", "700");
        let repo = all_in_sync(1).build();
        let repo_ref = repo.repo_ref();
        let want = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
        let handle = Daemon::new(SESSION).spawn(backend.clone()).await.unwrap();
        let mut conn = Conn::connect(&handle.endpoint()).await;
        let id = conn
            .send(
                Some(&repo_ref),
                OperationKind::RemoteGroupRun,
                json!({
                    "branch": "main",
                    "targets": [push_target("origin"), push_target("gitlab"), push_target("backup")]
                }),
            )
            .await;
        // Origin got the commit: the run is waiting out gitlab's delay.
        wait_for("origin pushed", || {
            repo.git_in_remote(&["rev-parse", "refs/heads/main"]).trim() == want
        });
        // The ref moves a moment before the push process ends: let origin's
        // push finish, then cancel inside gitlab's delay.
        tokio::time::sleep(Duration::from_millis(250)).await;
        let job = conn.running_job(OperationKind::RemoteGroupRun).await;
        conn.call(OperationKind::JobCancel, json!({ "id": job }))
            .await;
        let response = conn.wait(id).await;
        std::env::remove_var("GITPOULPI_TEST_STEP_DELAY_MS");
        let out: RemoteGroupOutcome = serde_json::from_value(response.result.unwrap()).unwrap();
        assert_eq!(out.status, GroupRunStatus::Canceled, "{name}: {out:?}");
        assert!(matches!(
            result(&out, "origin").status,
            TargetStatus::Pushed { .. }
        ));
        assert_eq!(result(&out, "gitlab").status, TargetStatus::NotAttempted);
        assert_eq!(result(&out, "backup").status, TargetStatus::NotAttempted);
        assert!(out.pending, "{name}");
        let id = conn
            .send(
                Some(&repo_ref),
                OperationKind::RemoteGroupRun,
                json!({ "resume": "continue" }),
            )
            .await;
        let response = conn.wait(id).await;
        let finished: RemoteGroupOutcome =
            serde_json::from_value(response.result.unwrap()).unwrap();
        assert_eq!(
            finished.status,
            GroupRunStatus::Done,
            "{name}: {finished:?}"
        );
        for remote in ["gitlab", "backup"] {
            assert_eq!(
                repo.git_in_named_remote(remote, &["rev-parse", "refs/heads/main"])
                    .trim(),
                want,
                "{name}: {remote}"
            );
        }
        handle.shutdown().await.unwrap();
    }
}
