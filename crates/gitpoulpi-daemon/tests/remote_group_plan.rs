//! `RemoteGroupPlan`: preflight with `ls-remote`, a selective fetch,
//! classification against what the remotes hold, suggested actions,
//! conflict prediction, same-URL twins and an explicit target list.
//! Both backends, 2-3 bare remotes.

use std::sync::Arc;

use gitpoulpi_core::domain::{
    GroupAction, GroupMode, OperationKind, RemoteGroupPlan, RemoteRelation, RepoId, TargetRole,
};
use gitpoulpi_core::GitBackend;
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
    _repo: FixtureRepo,
    id: RepoId,
    next: u64,
}

impl Rig {
    async fn new(backend: Arc<dyn GitBackend>, repo: FixtureRepo) -> Rig {
        std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
        let handle = Daemon::new("remote-group-plan-test")
            .spawn(backend)
            .await
            .unwrap();
        let client = Client::connect(&handle.endpoint()).await.unwrap();
        let id = repo.repo_ref().id;
        Rig {
            handle: Some(handle),
            client,
            _repo: repo,
            id,
            next: 1,
        }
    }

    async fn call(&mut self, command: OperationKind, args: Value) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "remote-group-plan-test".into(),
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

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

/// `main` on origin (upstream), gitlab and backup, all equal to local,
/// with gitlab and backup as extra push targets; then `n` local commits.
fn all_in_sync(local_commits: usize) -> FixtureRepoBuilder {
    let mut b = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
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

fn row<'a>(p: &'a RemoteGroupPlan, remote: &str) -> &'a gitpoulpi_core::domain::PlannedTarget {
    p.targets
        .iter()
        .find(|t| t.remote == remote)
        .unwrap_or_else(|| panic!("no row for {remote}: {p:?}"))
}

#[tokio::test]
async fn all_fast_forward_pushes_everywhere_primary_first() {
    for (name, backend) in backends() {
        let repo = all_in_sync(2).build();
        let mut rig = Rig::new(backend, repo).await;
        let p = rig.plan(json!({ "branch": "main" })).await;
        let names: Vec<_> = p.targets.iter().map(|t| t.remote.as_str()).collect();
        assert_eq!(names, ["origin", "gitlab", "backup"], "{name}");
        assert_eq!(p.targets[0].role, TargetRole::Primary);
        assert_eq!(p.targets[1].role, TargetRole::PushTo);
        for t in &p.targets {
            assert_eq!(t.relation, RemoteRelation::LocalAhead(2), "{name}");
            assert_eq!(t.suggested, GroupAction::Push, "{name}");
            assert!(t.checked && !t.fetched && t.error.is_none(), "{name} {t:?}");
            assert!(t.lease.is_some());
        }
        rig.stop().await;
    }
}

#[tokio::test]
async fn diverged_remote_suggests_merge_and_fetches_only_that_remote() {
    for (name, backend) in backends() {
        let repo = all_in_sync(1).build();
        repo.push_from_elsewhere_to("gitlab", "main", "theirs on gitlab");
        let mut rig = Rig::new(backend, repo).await;
        let p = rig.plan(json!({ "branch": "main" })).await;
        let gl = row(&p, "gitlab");
        assert_eq!(
            gl.relation,
            RemoteRelation::Diverged {
                ahead: 1,
                behind: 1
            },
            "{name}"
        );
        assert_eq!(gl.suggested, GroupAction::MergeFrom, "{name}");
        assert!(!gl.rewritten);
        assert!(gl.fetched, "gitlab's new tip had to be fetched");
        assert!(gl.allowed.contains(&GroupAction::ForceWithLease));
        // The others did not need a fetch and still push.
        for r in ["origin", "backup"] {
            assert!(!row(&p, r).fetched, "{name} {r}");
            assert_eq!(row(&p, r).suggested, GroupAction::Push);
        }
        rig.stop().await;
    }
}

#[tokio::test]
async fn remote_ahead_suggests_fast_forward() {
    for (name, backend) in backends() {
        let repo = all_in_sync(0).build();
        repo.push_from_elsewhere_to("backup", "main", "ahead");
        let mut rig = Rig::new(backend, repo).await;
        let p = rig.plan(json!({ "branch": "main" })).await;
        let b = row(&p, "backup");
        assert_eq!(b.relation, RemoteRelation::RemoteAhead(1), "{name}");
        assert_eq!(b.suggested, GroupAction::FastForwardFrom);
        rig.stop().await;
    }
}

#[tokio::test]
async fn rebased_copies_on_a_remote_suggest_force_with_lease() {
    for (name, backend) in backends() {
        let repo = all_in_sync(0).commit("work").unwrap().build();
        // Publish "work" to gitlab, then amend it locally: gitlab holds an
        // older copy of the commit, patch-equal but another hash.
        repo.git(&["push", "gitlab", "main"]);
        repo.git(&["commit", "--amend", "-m", "work (reworded)"]);
        let mut rig = Rig::new(backend, repo).await;
        let p = rig.plan(json!({ "branch": "main" })).await;
        let gl = row(&p, "gitlab");
        assert!(
            matches!(gl.relation, RemoteRelation::Diverged { .. }),
            "{name} {gl:?}"
        );
        assert!(gl.rewritten, "{name}");
        assert_eq!(gl.suggested, GroupAction::ForceWithLease, "{name}");
        assert!(gl.lease.is_some());
        rig.stop().await;
    }
}

#[tokio::test]
async fn absent_branch_pushes_with_no_lease() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .with_origin()
            .unwrap()
            .with_remote("gitlab")
            .unwrap()
            .push("main")
            .unwrap()
            .track("main", "origin")
            .unwrap()
            .push_targets("main", &["gitlab"])
            .unwrap()
            .build();
        let mut rig = Rig::new(backend, repo).await;
        let p = rig.plan(json!({ "branch": "main" })).await;
        let gl = row(&p, "gitlab");
        assert_eq!(gl.relation, RemoteRelation::Absent, "{name}");
        assert_eq!(gl.suggested, GroupAction::Push);
        assert!(gl.lease.is_none());
        assert!(gl.checked);
        rig.stop().await;
    }
}

#[tokio::test]
async fn unrelated_history_skips_and_offers_force() {
    for (name, backend) in backends() {
        let repo = all_in_sync(0).build();
        // Replace backup's main with an unrelated root commit.
        let alien = repo
            .git_in_named_remote(
                "backup",
                &[
                    "-c",
                    "user.name=x",
                    "-c",
                    "user.email=x@x",
                    "commit-tree",
                    "-m",
                    "alien",
                    "4b825dc642cb6eb9a060e54bf8d69288fbee4904",
                ],
            )
            .trim()
            .to_string();
        repo.git_in_named_remote("backup", &["update-ref", "refs/heads/main", &alien]);
        repo.git(&["commit", "--allow-empty", "-m", "local only"]);
        let mut rig = Rig::new(backend, repo).await;
        let p = rig.plan(json!({ "branch": "main" })).await;
        let b = row(&p, "backup");
        assert_eq!(b.relation, RemoteRelation::Unrelated, "{name} {b:?}");
        assert_eq!(b.suggested, GroupAction::Skip);
        assert!(b.allowed.contains(&GroupAction::ForceWithLease));
        rig.stop().await;
    }
}

#[tokio::test]
async fn conflicting_divergence_lists_the_paths() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", "base\n", "base")
            .unwrap()
            .with_origin()
            .unwrap()
            .push("main")
            .unwrap()
            .track("main", "origin")
            .unwrap()
            .commit_file("a.txt", "mine\n", "mine")
            .unwrap()
            .build();
        // Another user changes the same line on origin.
        repo.git_in_remote(&["config", "user.name", "o"]);
        let clone = repo.root().join("..").join("other-clone");
        let origin = repo.remote_path().unwrap().to_string_lossy().to_string();
        run(&["clone", &origin, &clone.to_string_lossy()]);
        std::fs::write(clone.join("a.txt"), "theirs\n").unwrap();
        run_in(&clone, &["add", "a.txt"]);
        run_in(
            &clone,
            &[
                "-c",
                "user.name=o",
                "-c",
                "user.email=o@o",
                "commit",
                "-m",
                "theirs",
            ],
        );
        run_in(&clone, &["push", "origin", "main"]);
        let mut rig = Rig::new(backend, repo).await;
        let p = rig.plan(json!({ "branch": "main" })).await;
        let o = row(&p, "origin");
        assert_eq!(o.conflicts, vec!["a.txt".to_string()], "{name}");
        assert_eq!(o.suggested, GroupAction::MergeFrom);
        rig.stop().await;
        let _ = std::fs::remove_dir_all(&clone);
    }
}

fn run(args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "{args:?}: {out:?}");
}

fn run_in(dir: &std::path::Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "{args:?}: {out:?}");
}

#[tokio::test]
async fn same_url_twin_is_flagged_and_skipped() {
    for (name, backend) in backends() {
        let repo = all_in_sync(1).build();
        let url = repo
            .git(&["remote", "get-url", "gitlab"])
            .trim()
            .to_string();
        repo.git(&["remote", "add", "gitlab-twin", &format!("{url}/")]);
        repo.git(&[
            "config",
            "--add",
            "branch.main.gitpoulpi-push-to",
            "gitlab-twin",
        ]);
        let mut rig = Rig::new(backend, repo).await;
        let p = rig.plan(json!({ "branch": "main" })).await;
        let twin = row(&p, "gitlab-twin");
        assert_eq!(twin.same_url_as.as_deref(), Some("gitlab"), "{name}");
        assert_eq!(twin.suggested, GroupAction::Skip);
        assert!(twin.allowed.contains(&GroupAction::Push));
        assert!(row(&p, "gitlab").same_url_as.is_none());
        rig.stop().await;
    }
}

#[tokio::test]
async fn explicit_target_list_replaces_the_configured_one() {
    for (name, backend) in backends() {
        let repo = all_in_sync(1).build();
        let mut rig = Rig::new(backend, repo).await;
        let p = rig
            .plan(json!({
                "branch": "main",
                "targets": [
                    { "remote": "backup", "branch": "main" },
                    { "remote": "origin", "branch": "other-name" },
                ],
            }))
            .await;
        let names: Vec<_> = p
            .targets
            .iter()
            .map(|t| (t.remote.as_str(), t.branch.as_str()))
            .collect();
        assert_eq!(
            names,
            [("backup", "main"), ("origin", "other-name")],
            "{name}"
        );
        assert_eq!(p.targets[0].role, TargetRole::PushTo);
        // `origin:other-name` is a different branch than the primary's.
        assert_eq!(p.targets[1].relation, RemoteRelation::Absent);
        rig.stop().await;
    }
}

#[tokio::test]
async fn refused_target_defaults_to_skip_without_force() {
    for (name, backend) in backends() {
        let repo = all_in_sync(1).build();
        let mut rig = Rig::new(backend, repo).await;
        let p = rig
            .plan(json!({
                "branch": "main",
                "refused": [{ "remote": "gitlab", "branch": "main" }],
            }))
            .await;
        let gl = row(&p, "gitlab");
        assert_eq!(gl.suggested, GroupAction::Skip, "{name}");
        assert_eq!(gl.allowed, vec![GroupAction::Skip]);
        assert_eq!(row(&p, "origin").suggested, GroupAction::Push);
        rig.stop().await;
    }
}

#[tokio::test]
async fn an_unreachable_remote_falls_back_to_its_tracking_ref_and_skips() {
    for (name, backend) in backends() {
        let repo = all_in_sync(1).build();
        repo.git(&["remote", "set-url", "backup", "/nonexistent/backup.git"]);
        let mut rig = Rig::new(backend, repo).await;
        let p = rig.plan(json!({ "branch": "main" })).await;
        let b = row(&p, "backup");
        assert!(b.error.is_some(), "{name}");
        assert_eq!(b.suggested, GroupAction::Skip);
        assert_eq!(b.relation, RemoteRelation::LocalAhead(1));
        assert_eq!(row(&p, "origin").suggested, GroupAction::Push);
        rig.stop().await;
    }
}

#[tokio::test]
async fn without_preflight_the_plan_uses_tracking_refs_only() {
    for (name, backend) in backends() {
        let repo = all_in_sync(1).build();
        // Someone pushes behind our back: only a preflight would see it.
        repo.push_from_elsewhere_to("gitlab", "main", "unseen");
        let mut rig = Rig::new(backend, repo).await;
        let p = rig
            .plan(json!({ "branch": "main", "preflight": false }))
            .await;
        assert!(!p.preflight);
        let gl = row(&p, "gitlab");
        assert_eq!(gl.relation, RemoteRelation::LocalAhead(1), "{name}");
        assert!(!gl.checked);
        rig.stop().await;
    }
}

#[tokio::test]
async fn pull_mode_lists_the_primary_and_pull_sources() {
    for (name, backend) in backends() {
        let repo = all_in_sync(0).build();
        repo.git(&[
            "config",
            "--add",
            "branch.main.gitpoulpi-pull-from",
            "gitlab",
        ]);
        repo.push_from_elsewhere_to("gitlab", "main", "from gitlab");
        let mut rig = Rig::new(backend, repo).await;
        let p = rig.plan(json!({ "branch": "main", "mode": "pull" })).await;
        assert_eq!(p.mode, GroupMode::Pull);
        let names: Vec<_> = p.targets.iter().map(|t| t.remote.as_str()).collect();
        assert_eq!(names, ["origin", "gitlab"], "{name}");
        assert_eq!(row(&p, "origin").suggested, GroupAction::Skip);
        assert_eq!(row(&p, "gitlab").suggested, GroupAction::FastForwardFrom);
        assert_eq!(row(&p, "gitlab").role, TargetRole::PullFrom);
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_hook_protected_remote_is_planned_normally_and_refusal_is_a_later_signal() {
    // A pre-receive hook cannot be seen by a plan (only a push finds out),
    // so the row is an ordinary push; the plan request's `refused` list is
    // how a rejection feeds back (see the test above).
    for (name, backend) in backends() {
        let repo = all_in_sync(1)
            .with_pre_receive_hook("gitlab", "echo protected >&2; exit 1")
            .unwrap()
            .build();
        let mut rig = Rig::new(backend, repo).await;
        let p = rig.plan(json!({ "branch": "main" })).await;
        assert_eq!(row(&p, "gitlab").suggested, GroupAction::Push, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn unknown_branch_is_an_error() {
    for (_, backend) in backends() {
        let repo = all_in_sync(0).build();
        let mut rig = Rig::new(backend, repo).await;
        let r = rig
            .call(OperationKind::RemoteGroupPlan, json!({ "branch": "nope" }))
            .await;
        assert!(r.is_err());
        rig.stop().await;
    }
}
