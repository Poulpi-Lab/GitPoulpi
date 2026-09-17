//! Undo/redo hardening through the daemon (the undo-redo design):
//! detached HEAD for the history operations, the "already
//! pushed" confirmation for them, linked worktrees, a journal written by
//! a newer GitPoulpi, and two clients racing for the same entry. Every
//! test runs against both backends.

use std::sync::Arc;

use gitpoulpi_core::domain::{
    CommitOptions, Direction, Oid, OperationKind, RepoId, StageTarget, UndoOutcome, UndoRefusal,
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
    async fn new(backend: Arc<dyn GitBackend>) -> Rig {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .build();
        Self::on(backend, repo).await
    }

    async fn on(backend: Arc<dyn GitBackend>, repo: FixtureRepo) -> Rig {
        // These tests are about a blocked entry: outside changes are not
        // recorded here (outside_journal.rs covers them recorded).
        std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
        static ENV: std::sync::Once = std::sync::Once::new();
        ENV.call_once(|| std::env::set_var("GITPOULPI_GENERAL_UNDO_DEPTH", "0"));
        let handle = Daemon::new("undo-hardening-test")
            .spawn(backend)
            .await
            .unwrap();
        let client = Client::connect_port(handle.port).await.unwrap();
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
                session: "undo-hardening-test".into(),
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

    async fn redo(&mut self) -> UndoOutcome {
        serde_json::from_value(self.ok(OperationKind::Redo, json!({})).await).unwrap()
    }

    fn rev(&self, spec: &str) -> Oid {
        Oid::from_hex(&self.repo.git(&["rev-parse", spec])).unwrap()
    }

    async fn stop(mut self) -> FixtureRepo {
        self.handle.take().unwrap().shutdown().await.unwrap();
        self.repo
    }
}

impl Rig {
    /// The `Ref` JSON of a local branch, as the daemon's `Refs` read gives it.
    async fn branch_ref(&mut self, name: &str) -> Value {
        let refs = self.ok(OperationKind::Refs, json!({})).await;
        refs.as_array()
            .unwrap()
            .iter()
            .find(|r| r["kind"]["LocalBranch"]["name"] == name)
            .unwrap_or_else(|| panic!("no branch {name}"))
            .clone()
    }

    fn commit_in_shell(&self, file: &str, contents: &str, message: &str) -> Oid {
        std::fs::write(self.repo.root().join(file), contents).unwrap();
        self.repo.git(&["add", file]);
        self.repo.git(&["commit", "-q", "-m", message]);
        self.rev("HEAD")
    }

    async fn confirmation_needed(&mut self, command: OperationKind) -> String {
        let e = self.call(command, json!({})).await.unwrap_err();
        match e.kind {
            Some(TypedError::NeedsConfirmation(p)) => p,
            other => panic!("expected a confirmation, got {other:?} ({})", e.message),
        }
    }
}

impl Rig {
    async fn call_on(
        &mut self,
        repo: &RepoId,
        command: OperationKind,
        args: Value,
    ) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "undo-hardening-test".into(),
                repo: Some(repo.clone()),
                command,
                args,
            })
            .await
            .unwrap()
            .result
    }

    fn head(&self) -> Oid {
        self.rev("HEAD")
    }

    /// Writes `name` (staged) and commits it through the daemon.
    async fn commit_file(&mut self, name: &str, message: &str) -> Oid {
        std::fs::write(self.repo.root().join(name), name).unwrap();
        self.ok(
            OperationKind::StageFile,
            json!({ "target": StageTarget::File(name.into()) }),
        )
        .await;
        let oid = self
            .ok(
                OperationKind::Commit,
                json!({ "message": message, "opts": CommitOptions::default() }),
            )
            .await;
        serde_json::from_value(oid).unwrap()
    }
}

impl Rig {
    fn detached(&self) -> bool {
        self.repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]) == "HEAD"
    }

    /// An `origin` the current branch tracks, with everything pushed.
    fn with_origin(&self) -> tempfile::TempDir {
        let remote = tempfile::tempdir().unwrap();
        let bare = remote.path().join("origin.git");
        std::process::Command::new("git")
            .args(["init", "--bare", "-q"])
            .arg(&bare)
            .status()
            .unwrap();
        self.repo
            .git(&["remote", "add", "origin", &bare.to_string_lossy()]);
        self.repo.git(&["push", "-u", "origin", "main"]);
        remote
    }
}

// ------------------------------------------------------- detached HEAD

/// Runs `op` on a detached HEAD, then checks undo and redo move HEAD
/// alone, leaving it detached.
async fn detached_round_trip(
    name: &str,
    setup: impl Fn(&Rig),
    command: OperationKind,
    args: impl Fn(&mut Rig) -> std::pin::Pin<Box<dyn std::future::Future<Output = Value> + '_>>,
) {
    for (backend_name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        setup(&rig);
        assert!(rig.detached(), "{name}: setup detaches HEAD");
        let before = rig.rev("HEAD");
        let args = args(&mut rig).await;
        rig.ok(command, args).await;
        let after = rig.rev("HEAD");
        assert_ne!(
            after, before,
            "{name}/{backend_name}: the operation moved HEAD"
        );
        assert!(rig.detached(), "{name}/{backend_name}");

        let out = rig.undo().await;
        assert!(out.applied, "{name}/{backend_name}");
        assert_eq!(rig.rev("HEAD"), before, "{name}/{backend_name}");
        assert!(rig.detached(), "{name}/{backend_name}: still detached");
        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), after, "{name}/{backend_name}");
        assert!(rig.detached(), "{name}/{backend_name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn history_operations_on_a_detached_head_are_recorded_and_undone_with_head_alone() {
    // Merge: main diverged from feature, detached on main.
    detached_round_trip(
        "merge",
        |rig| {
            rig.repo.git(&["branch", "feature"]);
            rig.repo.git(&["checkout", "-q", "feature"]);
            rig.commit_in_shell("f.txt", "f", "feature work");
            rig.repo.git(&["checkout", "-q", "main"]);
            rig.commit_in_shell("m.txt", "m", "main work");
            rig.repo.git(&["checkout", "-q", "--detach"]);
        },
        OperationKind::Merge,
        |rig| Box::pin(async move { json!({ "from": rig.branch_ref("feature").await }) }),
    )
    .await;
    // Cherry-pick.
    detached_round_trip(
        "cherry-pick",
        |rig| {
            rig.repo.git(&["checkout", "-q", "-b", "other"]);
            rig.commit_in_shell("a.txt", "a", "pick a");
            rig.repo.git(&["checkout", "-q", "--detach", "main"]);
        },
        OperationKind::CherryPick,
        |rig| Box::pin(async move { json!({ "commits": [rig.rev("other")] }) }),
    )
    .await;
    // Revert.
    detached_round_trip(
        "revert",
        |rig| {
            rig.commit_in_shell("r.txt", "r", "add r");
            rig.repo.git(&["checkout", "-q", "--detach"]);
        },
        OperationKind::Revert,
        |rig| Box::pin(async move { json!({ "commit": rig.rev("HEAD") }) }),
    )
    .await;
    // Reset.
    detached_round_trip(
        "reset",
        |rig| {
            rig.commit_in_shell("one.txt", "1", "one");
            rig.commit_in_shell("two.txt", "2", "two");
            rig.repo.git(&["checkout", "-q", "--detach"]);
        },
        OperationKind::Reset,
        |rig| Box::pin(async move { json!({ "target": rig.rev("HEAD~1"), "mode": "Mixed" }) }),
    )
    .await;
    // Rebase.
    detached_round_trip(
        "rebase",
        |rig| {
            rig.repo.git(&["checkout", "-q", "-b", "feature"]);
            rig.commit_in_shell("f.txt", "f", "feature work");
            rig.repo.git(&["checkout", "-q", "main"]);
            rig.commit_in_shell("m.txt", "m", "main work");
            rig.repo.git(&["checkout", "-q", "--detach", "feature"]);
        },
        OperationKind::Rebase,
        |rig| Box::pin(async move { json!({ "onto": rig.branch_ref("main").await }) }),
    )
    .await;
}

// --------------------------------------------------- already pushed

#[tokio::test]
async fn undoing_a_pushed_history_operation_asks_first_and_proceeds_once_confirmed() {
    for (name, backend) in backends() {
        // Merge commit.
        let mut rig = Rig::new(Arc::clone(&backend)).await;
        let _remote = rig.with_origin();
        rig.repo.git(&["branch", "feature"]);
        rig.repo.git(&["checkout", "-q", "feature"]);
        rig.commit_in_shell("f.txt", "f", "feature work");
        rig.repo.git(&["checkout", "-q", "main"]);
        rig.commit_in_shell("m.txt", "m", "main work");
        rig.repo.git(&["push", "-q", "origin", "main"]);
        let before = rig.rev("HEAD");
        let feature = rig.branch_ref("feature").await;
        rig.ok(OperationKind::Merge, json!({ "from": feature }))
            .await;
        let merged = rig.rev("HEAD");
        // Not pushed yet: no question.
        let dry: UndoOutcome = serde_json::from_value(
            rig.ok(OperationKind::Undo, json!({ "dry_run": true }))
                .await,
        )
        .unwrap();
        assert!(dry.needs_confirmation.is_none(), "{name}: unpushed merge");
        rig.repo.git(&["push", "-q", "origin", "main"]);
        let prompt = rig.confirmation_needed(OperationKind::Undo).await;
        assert!(
            prompt.contains("already on origin/main"),
            "{name}: {prompt}"
        );
        assert_eq!(rig.rev("HEAD"), merged, "{name}: nothing moved");
        rig.ok(OperationKind::Undo, json!({ "confirmed": true }))
            .await;
        assert_eq!(rig.rev("HEAD"), before, "{name}");
        // Redo is local-only and never asks.
        let redone = rig.redo().await;
        assert!(redone.applied, "{name}");
        assert_eq!(rig.rev("HEAD"), merged, "{name}");
        rig.stop().await;

        // Cherry-pick and revert.
        for (what, command) in [
            ("cherry-pick", OperationKind::CherryPick),
            ("revert", OperationKind::Revert),
        ] {
            let mut rig = Rig::new(Arc::clone(&backend)).await;
            let _remote = rig.with_origin();
            let args = if command == OperationKind::CherryPick {
                rig.repo.git(&["checkout", "-q", "-b", "other"]);
                let pick = rig.commit_in_shell("a.txt", "a", "pick a");
                rig.repo.git(&["checkout", "-q", "main"]);
                json!({ "commits": [pick] })
            } else {
                let target = rig.commit_in_shell("r.txt", "r", "add r");
                rig.repo.git(&["push", "-q", "origin", "main"]);
                json!({ "commit": target })
            };
            let before = rig.rev("HEAD");
            rig.ok(command, args).await;
            rig.repo.git(&["push", "-q", "origin", "main"]);
            let prompt = rig.confirmation_needed(OperationKind::Undo).await;
            assert!(
                prompt.contains("already on origin/main"),
                "{name}/{what}: {prompt}"
            );
            rig.ok(OperationKind::Undo, json!({ "confirmed": true }))
                .await;
            assert_eq!(rig.rev("HEAD"), before, "{name}/{what}");
            rig.stop().await;
        }

        // Rebase of a branch whose rebased tip was then pushed.
        let mut rig = Rig::new(Arc::clone(&backend)).await;
        let _remote = rig.with_origin();
        rig.repo.git(&["checkout", "-q", "-b", "feature"]);
        rig.commit_in_shell("f.txt", "f", "feature work");
        rig.repo.git(&["checkout", "-q", "main"]);
        rig.commit_in_shell("m.txt", "m", "main work");
        rig.repo.git(&["push", "-q", "origin", "main"]);
        rig.repo.git(&["checkout", "-q", "feature"]);
        let before = rig.rev("HEAD");
        let main = rig.branch_ref("main").await;
        rig.ok(OperationKind::Rebase, json!({ "onto": main })).await;
        rig.repo.git(&["push", "-q", "-u", "origin", "feature"]);
        let prompt = rig.confirmation_needed(OperationKind::Undo).await;
        assert!(
            prompt.contains("already on origin/feature"),
            "{name}/rebase: {prompt}"
        );
        rig.ok(OperationKind::Undo, json!({ "confirmed": true }))
            .await;
        assert_eq!(rig.rev("HEAD"), before, "{name}/rebase");
        rig.stop().await;
    }
}

// ------------------------------------------------------ linked worktrees

#[tokio::test]
async fn a_linked_worktree_has_its_own_journal_and_its_own_pins() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let linked_parent = tempfile::tempdir().unwrap();
        let linked = linked_parent.path().join("wt");
        rig.repo.git(&[
            "worktree",
            "add",
            "-q",
            "-b",
            "wt-branch",
            &linked.to_string_lossy(),
        ]);
        let linked_id = gitpoulpi_core::domain::RepoRef::from_root(&linked).id;

        let main_before = rig.rev("main");
        let main_commit = rig.commit_file("main.txt", "main side").await;
        let wt_before = Oid::from_hex(
            std::str::from_utf8(
                &std::process::Command::new("git")
                    .args(["rev-parse", "HEAD"])
                    .current_dir(&linked)
                    .output()
                    .unwrap()
                    .stdout,
            )
            .unwrap()
            .trim(),
        )
        .unwrap();
        std::fs::write(linked.join("wt.txt"), "worktree side").unwrap();
        rig.call_on(
            &linked_id,
            OperationKind::StageFile,
            json!({ "target": StageTarget::File("wt.txt".into()) }),
        )
        .await
        .unwrap();
        let wt_commit: Oid = serde_json::from_value(
            rig.call_on(
                &linked_id,
                OperationKind::Commit,
                json!({ "message": "wt commit", "opts": CommitOptions::default() }),
            )
            .await
            .unwrap(),
        )
        .unwrap();

        // Two journals, one per worktree, each with its own commit.
        let wt_state: gitpoulpi_core::domain::JournalState = serde_json::from_value(
            rig.call_on(&linked_id, OperationKind::JournalState, json!({}))
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            wt_state.undo.map(|e| e.kind),
            Some(OperationKind::Commit),
            "{name}: the linked worktree recorded its commit"
        );
        // Undo in the worktree moves only the worktree.
        rig.call_on(&linked_id, OperationKind::Undo, json!({}))
            .await
            .unwrap();
        assert_eq!(rig.rev("wt-branch"), wt_before, "{name}");
        assert_eq!(rig.rev("main"), main_commit, "{name}: main untouched");
        // And undo in main moves only main.
        rig.undo().await;
        assert_eq!(rig.rev("main"), main_before, "{name}");
        assert_eq!(rig.rev("wt-branch"), wt_before, "{name}");
        // Redo in the worktree still finds its commit.
        rig.call_on(&linked_id, OperationKind::Redo, json!({}))
            .await
            .unwrap();
        assert_eq!(rig.rev("wt-branch"), wt_commit, "{name}");
        let repo = rig.stop().await;

        // Each journal's pins are the refs it names, and nobody else's:
        // ids restart at 1 in every worktree, the namespaces keep them apart.
        let git_dirs = [
            repo.root().join(".git"),
            repo.root().join(".git/worktrees/wt"),
        ];
        let mut checked = 0;
        for dir in git_dirs {
            let text = std::fs::read_to_string(dir.join("gitpoulpi/journal.json"))
                .unwrap_or_else(|e| panic!("{name}: {}: {e}", dir.display()));
            let journal: gitpoulpi_core::journal::Journal = serde_json::from_str(&text).unwrap();
            for entry in journal
                .undo
                .iter()
                .chain(&journal.redo)
                .chain(&journal.retired)
            {
                for pin in &entry.pins {
                    assert_eq!(
                        repo.git(&["rev-parse", &pin.name]),
                        pin.oid.to_string(),
                        "{name}: pin {} of entry {} was overwritten",
                        pin.name,
                        entry.id
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked >= 4, "{name}: {checked} pins checked");
        let names = repo.git(&[
            "for-each-ref",
            "--format=%(refname)",
            "refs/gitpoulpi/undo/",
        ]);
        assert!(
            names.contains("undo/1/0") && names.contains("undo/wt-wt-"),
            "{name}: {names}"
        );
    }
}

// ------------------------------------------------ a newer journal

#[tokio::test]
async fn a_journal_written_by_a_newer_version_is_left_untouched_and_never_blocks_work() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .build();
        let dir = repo.root().join(".git/gitpoulpi");
        std::fs::create_dir_all(&dir).unwrap();
        let newer =
            br#"{"version":99,"next_id":7,"undo":[{"future":"shape"}],"redo":[],"retired":[]}"#;
        std::fs::write(dir.join("journal.json"), newer).unwrap();

        let mut rig = Rig::on(backend, repo).await;
        // Git work goes on, unrecorded.
        let commit = rig.commit_file("a.txt", "newer").await;
        assert_eq!(rig.head(), commit, "{name}");
        // Undo explains instead of guessing.
        let e = rig.call(OperationKind::Undo, json!({})).await.unwrap_err();
        assert!(
            e.message.contains("version 99"),
            "{name}: the message names the version: {}",
            e.message
        );
        let state = rig.call(OperationKind::JournalState, json!({})).await;
        assert!(state.is_err(), "{name}");
        assert_eq!(rig.head(), commit, "{name}: nothing moved");
        let repo = rig.stop().await;
        assert_eq!(
            std::fs::read(repo.root().join(".git/gitpoulpi/journal.json")).unwrap(),
            newer,
            "{name}: the file is byte for byte what the newer version wrote"
        );
    }
}

// ------------------------------------------------ concurrent clients

#[tokio::test]
async fn two_clients_undoing_the_same_entry_never_both_win() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let first = rig.head();
        rig.commit_file("a.txt", "one").await;
        let second_tip = rig.commit_file("b.txt", "two").await;
        let top = {
            let state: gitpoulpi_core::domain::JournalState =
                serde_json::from_value(rig.ok(OperationKind::JournalState, json!({})).await)
                    .unwrap();
            state.undo.unwrap().id
        };
        let port = rig.handle.as_ref().unwrap().port;
        let id = rig.id.clone();
        let racers: Vec<_> = (0..4u64)
            .map(|n| {
                let id = id.clone();
                tokio::spawn(async move {
                    let mut client = Client::connect_port(port).await.unwrap();
                    client
                        .call(&Request {
                            id: 100 + n,
                            session: "undo-hardening-test".into(),
                            repo: Some(id),
                            command: OperationKind::Undo,
                            // Everyone saw the same top entry and wants
                            // exactly that one undone.
                            args: json!({ "expect": top }),
                        })
                        .await
                        .unwrap()
                        .result
                })
            })
            .collect();
        let mut won = 0;
        for racer in racers {
            match racer.await.unwrap() {
                Ok(_) => won += 1,
                Err(e) => assert!(
                    matches!(
                        e.kind,
                        Some(TypedError::UndoRefused(
                            UndoRefusal::StaleEntry { .. } | UndoRefusal::NothingToUndo
                        ))
                    ) || matches!(e.kind, Some(TypedError::UndoRefused(_))),
                    "{name}: {e:?}"
                ),
            }
        }
        assert_eq!(won, 1, "{name}: exactly one undo of that entry");
        assert_ne!(rig.head(), second_tip, "{name}");
        assert_ne!(
            rig.head(),
            first,
            "{name}: the older commit was not undone too"
        );
        rig.stop().await;
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
