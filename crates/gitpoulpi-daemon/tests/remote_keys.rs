//! A branch's extra push targets / pull sources (`gitpoulpi-push-to`,
//! `gitpoulpi-pull-from`): `BranchSetRemotes` is journaled and undoable,
//! `RemoteGroupStatus` and `Ref.multi_remote` answer from local refs.
//! Both backends.

use std::sync::Arc;

use gitpoulpi_core::domain::{
    Direction, OperationKind, Ref, RefKind, RemoteGroupStatus, RemoteRelation, RepoId, TargetRole,
    UndoOutcome,
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
    repo: FixtureRepo,
    id: RepoId,
    next: u64,
}

impl Rig {
    async fn new(backend: Arc<dyn GitBackend>, repo: FixtureRepo) -> Rig {
        std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
        let handle = Daemon::new("remote-keys-test")
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
                session: "remote-keys-test".into(),
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

    async fn redo(&mut self) {
        self.ok(OperationKind::Redo, json!({})).await;
    }

    fn all(&self, key: &str) -> Vec<String> {
        self.repo
            .try_git(&["config", "--local", "--get-all", key])
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }

    async fn status(&mut self, branch: &str) -> RemoteGroupStatus {
        let v = self
            .ok(
                OperationKind::RemoteGroupStatus,
                json!({ "branch": branch }),
            )
            .await;
        serde_json::from_value(v).unwrap()
    }

    async fn refs(&mut self) -> Vec<Ref> {
        let v = self.ok(OperationKind::Refs, json!({})).await;
        serde_json::from_value(v).unwrap()
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

/// `main` pushed to origin (its upstream) and gitlab, then one more local
/// commit; `backup` is a third remote main was never pushed to.
fn three_remotes() -> FixtureRepo {
    FixtureRepoBuilder::new()
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
        .track("main", "origin")
        .unwrap()
        .commit("second")
        .unwrap()
        .build()
}

#[tokio::test]
async fn set_remotes_is_journaled_and_undo_restores_the_section() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend, three_remotes()).await;
        assert!(
            rig.all("branch.main.gitpoulpi-push-to").is_empty(),
            "{name}"
        );

        rig.ok(
            OperationKind::BranchSetRemotes,
            json!({ "branch": "main", "push_to": ["gitlab", "backup:x"], "pull_from": ["gitlab"] }),
        )
        .await;
        assert_eq!(
            rig.all("branch.main.gitpoulpi-push-to"),
            vec!["gitlab", "backup:x"],
            "{name}"
        );
        assert_eq!(rig.all("branch.main.gitpoulpi-pull-from"), vec!["gitlab"]);
        assert_eq!(
            rig.all("branch.main.remote"),
            vec!["origin"],
            "upstream kept"
        );

        // Replacing keeps the upstream and drops what is no longer listed.
        rig.ok(
            OperationKind::BranchSetRemotes,
            json!({ "branch": "main", "push_to": ["backup"], "pull_from": [] }),
        )
        .await;
        assert_eq!(rig.all("branch.main.gitpoulpi-push-to"), vec!["backup"]);
        assert!(
            rig.all("branch.main.gitpoulpi-pull-from").is_empty(),
            "{name}"
        );

        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::BranchSetRemotes, "{name}");
        assert_eq!(out.entry.label, "Set remotes of main", "{name}");
        assert_eq!(
            rig.all("branch.main.gitpoulpi-push-to"),
            vec!["gitlab", "backup:x"],
            "{name}"
        );
        assert_eq!(rig.all("branch.main.gitpoulpi-pull-from"), vec!["gitlab"]);
        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::BranchSetRemotes);
        assert!(
            rig.all("branch.main.gitpoulpi-push-to").is_empty(),
            "{name}"
        );
        assert_eq!(rig.all("branch.main.remote"), vec!["origin"], "{name}");
        rig.redo().await;
        rig.redo().await;
        assert_eq!(rig.all("branch.main.gitpoulpi-push-to"), vec!["backup"]);

        // An unknown remote is refused and changes nothing.
        let e = rig
            .call(
                OperationKind::BranchSetRemotes,
                json!({ "branch": "main", "push_to": ["nope"] }),
            )
            .await
            .unwrap_err();
        assert!(e.message.contains("nope"), "{name}: {}", e.message);
        assert_eq!(rig.all("branch.main.gitpoulpi-push-to"), vec!["backup"]);
        rig.stop().await;
    }
}

#[tokio::test]
async fn remote_group_status_reads_each_remote_from_local_refs() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend, three_remotes()).await;

        // No keys yet: no decoration, and the status has just the primary.
        assert!(
            rig.refs().await.iter().all(|r| r.multi_remote.is_none()),
            "{name}"
        );
        let s = rig.status("main").await;
        assert_eq!(s.targets.len(), 1, "{name}");
        assert_eq!(s.targets[0].role, TargetRole::Primary);
        assert_eq!(s.targets[0].remote, "origin");
        // `second` is unpushed: ahead of origin by one.
        assert_eq!(
            s.targets[0].relation,
            RemoteRelation::LocalAhead(1),
            "{name}"
        );

        rig.ok(
            OperationKind::BranchSetRemotes,
            json!({ "branch": "main", "push_to": ["gitlab", "backup:other"], "pull_from": ["gitlab", "backup:other"] }),
        )
        .await;
        let s = rig.status("main").await;
        let roles: Vec<(String, TargetRole)> = s
            .targets
            .iter()
            .map(|t| (format!("{}:{}", t.remote, t.branch), t.role))
            .collect();
        assert_eq!(
            roles,
            vec![
                ("origin:main".to_string(), TargetRole::Primary),
                ("gitlab:main".to_string(), TargetRole::PushTo),
                ("backup:other".to_string(), TargetRole::PushTo),
            ],
            "{name}"
        );
        assert_eq!(
            s.targets[1].relation,
            RemoteRelation::LocalAhead(1),
            "{name}"
        );
        assert_eq!(s.targets[1].tracking.as_deref(), Some("gitlab/main"));
        assert!(s.targets[1].pull && s.targets[2].pull, "{name}");
        assert_eq!(s.targets[2].relation, RemoteRelation::Absent, "{name}");
        assert_eq!(s.targets[2].tracking, None);
        assert!(s.remotes.missing.is_empty(), "{name}");

        // The Refs read carries it, on that branch only.
        let refs = rig.refs().await;
        let decorated: Vec<&Ref> = refs.iter().filter(|r| r.multi_remote.is_some()).collect();
        assert_eq!(decorated.len(), 1, "{name}");
        assert!(matches!(&decorated[0].kind, RefKind::LocalBranch { name } if name == "main"));
        let multi = decorated[0].multi_remote.as_ref().unwrap();
        assert_eq!(multi.len(), 3, "{name}");
        assert_eq!(multi[1].relation, RemoteRelation::LocalAhead(1), "{name}");

        // A new commit is seen through the memo (the tip changed).
        rig.repo
            .git(&["commit", "--allow-empty", "-q", "-m", "third"]);
        let refs = rig.refs().await;
        let main = refs.iter().find(|r| r.multi_remote.is_some()).unwrap();
        assert_eq!(
            main.multi_remote.as_ref().unwrap()[1].relation,
            RemoteRelation::LocalAhead(2),
            "{name}"
        );

        // Removing a remote with plain git leaves the keys: reported, not fatal.
        rig.repo.git(&["remote", "remove", "backup"]);
        let s = rig.status("main").await;
        assert_eq!(s.remotes.missing, vec!["backup".to_string()], "{name}");
        assert!(s.targets[2].missing, "{name}");
        rig.stop().await;
    }
}
