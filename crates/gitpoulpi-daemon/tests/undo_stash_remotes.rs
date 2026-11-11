//! Undo/redo of stashes, remotes and LFS tracking through the daemon
//! (the undo-redo design). Every test runs against
//! both backends and covers undo, redo and the refusal after an outside
//! change. A stash comes back as `stash@{0}` with the very same commit.

use std::sync::Arc;

use gitpoulpi_core::domain::{JournalListing, OperationKind, RepoId, UndoOutcome, UndoRefusal};
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
        let handle = Daemon::new("undo-stash-remotes-test")
            .spawn(backend.clone())
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
                session: "undo-stash-remotes-test".into(),
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

    fn path(&self, name: &str) -> std::path::PathBuf {
        self.repo.root().join(name)
    }

    fn stash_list(&self) -> String {
        self.repo.git(&["stash", "list", "--format=%H %gs"])
    }

    fn status(&self) -> String {
        self.repo.git(&["status", "--porcelain=v2", "-uall"])
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

/// A repo with one tracked file edited and one untracked file.
fn dirty_repo() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "first")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("a.txt"), "one\nedited\n").unwrap();
    std::fs::write(repo.root().join("fresh.txt"), "untracked\n").unwrap();
    repo
}

fn push_args() -> Value {
    json!({ "message": "wip", "include_untracked": true, "paths": [] })
}

// --------------------------------------------------------------- stashes

#[tokio::test]
async fn a_stash_push_is_undone_and_redone_with_the_same_commit() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, dirty_repo()).await;
        let dirty = rig.status();
        rig.ok(OperationKind::StashPush, push_args()).await;
        let stashed = rig.stash_list();
        assert!(stashed.contains("wip"), "{name}");
        assert!(rig.status().is_empty(), "{name}: the tree is clean");

        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::StashPush, "{name}");
        assert!(out.entry.label.starts_with("Stash \"wip\""), "{name}");
        assert_eq!(rig.status(), dirty, "{name}: the changes are back");
        assert_eq!(
            std::fs::read_to_string(rig.path("fresh.txt")).unwrap(),
            "untracked\n",
            "{name}"
        );
        assert!(rig.stash_list().is_empty(), "{name}: the stash is gone");

        let out = rig.redo().await;
        assert!(out.applied, "{name}");
        assert_eq!(rig.stash_list(), stashed, "{name}: the very same commit");
        assert!(rig.status().is_empty(), "{name}");
        assert!(!rig.path("fresh.txt").exists(), "{name}");

        // And once more, to be sure the redo left a sound entry.
        rig.undo().await;
        assert_eq!(rig.status(), dirty, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_stash_push_undo_refuses_after_an_outside_change() {
    for (name, backend) in backends() {
        // A file edited again after the push.
        let mut rig = Rig::on(backend.clone(), dirty_repo()).await;
        rig.ok(OperationKind::StashPush, push_args()).await;
        std::fs::write(rig.path("a.txt"), "someone else\n").unwrap();
        let reason = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(&reason, UndoRefusal::PathChanged { path } if path == "a.txt"),
            "{name}: {reason:?}"
        );
        assert!(rig.stash_list().contains("wip"), "{name}: nothing changed");
        rig.stop().await;

        // The stash popped in a shell.
        let mut rig = Rig::on(backend, dirty_repo()).await;
        rig.ok(OperationKind::StashPush, push_args()).await;
        rig.repo.git(&["stash", "pop"]);
        let reason = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(
                reason,
                UndoRefusal::StashChanged { .. } | UndoRefusal::PathChanged { .. }
            ),
            "{name}: {reason:?}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_stash_drop_is_undone_as_stash_zero_with_the_same_commit_and_redone() {
    for (name, backend) in backends() {
        let repo = dirty_repo();
        let mut rig = Rig::on(backend, repo).await;
        rig.ok(OperationKind::StashPush, push_args()).await;
        std::fs::write(rig.path("b.txt"), "second\n").unwrap();
        rig.ok(
            OperationKind::StashPush,
            json!({ "message": "second", "include_untracked": true, "paths": [] }),
        )
        .await;
        let both = rig.stash_list();
        assert_eq!(both.lines().count(), 2, "{name}");

        // Drop the older one (stash@{1}); journal entries are by commit.
        rig.ok(OperationKind::StashDrop, json!({ "id": 1 })).await;
        assert_eq!(rig.stash_list().lines().count(), 1, "{name}");
        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::StashDrop, "{name}");
        // Back as stash@{0}: the same commits, the restored one on top.
        let lines: Vec<&str> = both.lines().collect();
        let back = rig.stash_list();
        let back: Vec<&str> = back.lines().collect();
        assert_eq!(back.len(), 2, "{name}");
        assert_eq!(back[0], lines[1], "{name}: the dropped stash is on top");
        assert_eq!(back[1], lines[0], "{name}");

        let out = rig.redo().await;
        assert!(out.applied, "{name}");
        assert_eq!(rig.stash_list().lines().count(), 1, "{name}");
        assert_eq!(rig.stash_list(), lines[0], "{name}: the other one stays");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_stash_drop_undo_refuses_when_the_stash_was_stored_again_outside() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, dirty_repo()).await;
        rig.ok(OperationKind::StashPush, push_args()).await;
        let commit = rig.repo.git(&["rev-parse", "stash@{0}"]);
        rig.ok(OperationKind::StashDrop, json!({ "id": 0 })).await;
        rig.repo.git(&["stash", "store", "-m", "by hand", &commit]);
        let reason = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(reason, UndoRefusal::StashChanged { .. }),
            "{name}: {reason:?}"
        );
        assert_eq!(rig.stash_list().lines().count(), 1, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_stash_pop_is_undone_back_to_the_stash_and_the_dirty_tree_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, dirty_repo()).await;
        rig.ok(OperationKind::StashPush, push_args()).await;
        let stashed = rig.stash_list();
        rig.ok(OperationKind::StashPop, json!({ "id": 0 })).await;
        let popped = rig.status();
        assert!(!popped.is_empty(), "{name}");
        assert!(rig.stash_list().is_empty(), "{name}");

        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::StashPop, "{name}");
        assert!(rig.status().is_empty(), "{name}: the tree is clean again");
        assert_eq!(
            rig.stash_list(),
            stashed,
            "{name}: stash@{{0}}, same commit"
        );

        let out = rig.redo().await;
        assert!(out.applied, "{name}");
        assert_eq!(rig.status(), popped, "{name}");
        assert!(rig.stash_list().is_empty(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_stash_apply_is_undone_and_keeps_the_stash_throughout() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, dirty_repo()).await;
        rig.ok(OperationKind::StashPush, push_args()).await;
        let stashed = rig.stash_list();
        rig.ok(OperationKind::StashApply, json!({ "id": 0 })).await;
        let applied = rig.status();
        assert!(!applied.is_empty(), "{name}");

        rig.undo().await;
        assert!(rig.status().is_empty(), "{name}");
        assert_eq!(rig.stash_list(), stashed, "{name}");
        rig.redo().await;
        assert_eq!(rig.status(), applied, "{name}");
        assert_eq!(rig.stash_list(), stashed, "{name}");

        // Edit a file the apply wrote: the undo must not overwrite it.
        std::fs::write(rig.path("a.txt"), "mine\n").unwrap();
        let reason = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(&reason, UndoRefusal::PathChanged { path } if path == "a.txt"),
            "{name}: {reason:?}"
        );
        assert_eq!(
            std::fs::read_to_string(rig.path("a.txt")).unwrap(),
            "mine\n",
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn nothing_is_lost_the_stash_commits_are_pinned_while_the_entry_lives() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, dirty_repo()).await;
        rig.ok(OperationKind::StashPush, push_args()).await;
        let commit = rig.repo.git(&["rev-parse", "stash@{0}"]);
        rig.ok(OperationKind::StashDrop, json!({ "id": 0 })).await;
        // Even after a gc-style prune the dropped commit is reachable.
        let pins = rig.repo.git(&[
            "for-each-ref",
            "refs/gitpoulpi/undo/",
            "--format=%(objectname)",
        ]);
        assert!(pins.contains(&commit), "{name}: {pins}");
        let listing = rig.listing().await;
        assert_eq!(listing.undo.len(), 2, "{name}");
        rig.stop().await;
    }
}

// --------------------------------------------------------------- remotes

fn remote_repo() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .branch("feature")
        .unwrap()
        .checkout("main")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .push("feature")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .track("feature", "origin")
        .unwrap()
        .build()
}

fn remote_state(rig: &Rig) -> (String, String, String) {
    (
        rig.repo
            .git(&["config", "--local", "--list"])
            .lines()
            .filter(|l| l.starts_with("remote.") || l.starts_with("branch."))
            .collect::<Vec<_>>()
            .join("\n"),
        rig.repo.git(&[
            "for-each-ref",
            "refs/remotes/",
            "refs/gitpoulpi/remote-tags/",
            "--format=%(refname) %(objectname)",
        ]),
        rig.repo.git(&["remote", "-v"]),
    )
}

#[tokio::test]
async fn removing_a_remote_is_undone_with_its_tracking_branches_and_upstreams() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, remote_repo()).await;
        rig.repo.git(&["remote", "set-head", "origin", "main"]);
        let before = remote_state(&rig);
        assert!(before.1.contains("refs/remotes/origin/main"), "{name}");
        assert!(before.1.contains("refs/remotes/origin/HEAD"), "{name}");

        rig.ok(OperationKind::RemoteRemove, json!({ "name": "origin" }))
            .await;
        let removed = remote_state(&rig);
        assert!(!removed.1.contains("refs/remotes/origin"), "{name}");
        assert!(!removed.0.contains("branch.main.remote"), "{name}");

        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::RemoteRemove, "{name}");
        assert_eq!(out.entry.label, "Remove remote origin", "{name}");
        let back = remote_state(&rig);
        assert_eq!(back.1, before.1, "{name}: tracking refs");
        assert_eq!(
            rig.repo.git(&["symbolic-ref", "refs/remotes/origin/HEAD"]),
            "refs/remotes/origin/main",
            "{name}: origin/HEAD is an alias again, not a copy"
        );
        let sorted = |s: &str| {
            let mut l: Vec<&str> = s.lines().collect();
            l.sort();
            l.join("\n")
        };
        assert_eq!(sorted(&back.0), sorted(&before.0), "{name}: config");
        assert_eq!(back.2, before.2, "{name}: remote -v");

        let out = rig.redo().await;
        assert!(out.applied, "{name}");
        assert_eq!(remote_state(&rig), removed, "{name}: redo");
        assert!(
            !remote_state(&rig).1.contains("origin/HEAD"),
            "{name}: redo removes the alias too"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_remote_remove_undo_refuses_after_an_outside_change() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, remote_repo()).await;
        rig.ok(OperationKind::RemoteRemove, json!({ "name": "origin" }))
            .await;
        // A remote of the same name added by hand.
        rig.repo.git(&["remote", "add", "origin", "/elsewhere"]);
        let reason = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(&reason, UndoRefusal::ConfigChanged { section } if section == "remote.origin"),
            "{name}: {reason:?}"
        );
        assert_eq!(rig.repo.git(&["remote"]), "origin", "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn adding_a_remote_is_undone_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, remote_repo()).await;
        rig.ok(
            OperationKind::RemoteAdd,
            json!({ "name": "fork", "url": "/some/where.git" }),
        )
        .await;
        assert!(rig.repo.git(&["remote"]).contains("fork"), "{name}");
        rig.undo().await;
        assert_eq!(rig.repo.git(&["remote"]), "origin", "{name}");
        rig.redo().await;
        assert!(rig.repo.git(&["remote"]).contains("fork"), "{name}");
        assert_eq!(
            rig.repo.git(&["config", "remote.fork.url"]),
            "/some/where.git",
            "{name}"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn renaming_a_remote_is_undone_with_refs_and_upstreams_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, remote_repo()).await;
        rig.repo.git(&["remote", "set-head", "origin", "main"]);
        let before = remote_state(&rig);
        rig.ok(
            OperationKind::RemoteRename,
            json!({ "old_name": "origin", "new_name": "upstream" }),
        )
        .await;
        let renamed = remote_state(&rig);
        assert!(renamed.1.contains("refs/remotes/upstream/main"), "{name}");
        assert_eq!(rig.repo.git(&["config", "branch.main.remote"]), "upstream");
        assert_eq!(
            rig.repo
                .git(&["symbolic-ref", "refs/remotes/upstream/HEAD"]),
            "refs/remotes/upstream/main",
            "{name}"
        );

        rig.undo().await;
        assert_eq!(rig.repo.git(&["remote"]), "origin", "{name}");
        assert_eq!(rig.repo.git(&["config", "branch.main.remote"]), "origin");
        assert_eq!(rig.repo.git(&["config", "branch.feature.remote"]), "origin");
        let back = remote_state(&rig);
        assert!(back.1.contains("refs/remotes/origin/main"), "{name}");
        assert!(!back.1.contains("refs/remotes/upstream/main"), "{name}");
        assert_eq!(back.2, before.2, "{name}");
        assert_eq!(back.1, before.1, "{name}: including origin/HEAD");

        rig.redo().await;
        assert_eq!(rig.repo.git(&["remote"]), "upstream", "{name}");
        assert_eq!(
            rig.repo
                .git(&["symbolic-ref", "refs/remotes/upstream/HEAD"]),
            "refs/remotes/upstream/main",
            "{name}"
        );
        assert_eq!(rig.repo.git(&["config", "branch.main.remote"]), "upstream");
        assert!(remote_state(&rig).1.contains("refs/remotes/upstream/main"));
        rig.stop().await;
    }
}

#[tokio::test]
async fn setting_the_url_and_the_push_url_of_a_remote_is_undone_and_redone() {
    for (name, backend) in backends() {
        let mut rig = Rig::on(backend, remote_repo()).await;
        let original = rig.repo.git(&["config", "remote.origin.url"]);
        rig.ok(
            OperationKind::RemoteSetUrl,
            json!({ "name": "origin", "url": "https://example.test/a.git" }),
        )
        .await;
        rig.ok(
            OperationKind::RemoteSetPushUrl,
            json!({ "name": "origin", "url": "https://example.test/push.git" }),
        )
        .await;
        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::RemoteSetPushUrl, "{name}");
        assert!(
            !rig.repo.git(&["remote", "-v"]).contains("push.git"),
            "{name}"
        );
        rig.undo().await;
        assert_eq!(rig.repo.git(&["config", "remote.origin.url"]), original);
        rig.redo().await;
        rig.redo().await;
        assert_eq!(
            rig.repo.git(&["config", "remote.origin.pushurl"]),
            "https://example.test/push.git",
            "{name}"
        );
        // Changed in a shell since: refused, nothing touched.
        rig.repo
            .git(&["remote", "set-url", "origin", "https://shell.test/x.git"]);
        let reason = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(reason, UndoRefusal::ConfigChanged { .. }),
            "{name}: {reason:?}"
        );
        rig.stop().await;
    }
}

// ------------------------------------------------------------------ LFS

#[tokio::test]
async fn lfs_track_and_untrack_are_undone_and_redone() {
    gitpoulpi_testkit::require_git_lfs!("lfs_track_and_untrack_are_undone_and_redone");
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .with_lfs()
            .unwrap()
            .commit_file("big.psd", "pretend binary\n", "add a psd")
            .unwrap()
            .build();
        let mut rig = Rig::on(backend, repo).await;
        let attrs_path = rig.path(".gitattributes");
        let attrs = move || std::fs::read_to_string(&attrs_path).unwrap_or_default();
        let none = attrs();

        rig.ok(
            OperationKind::LfsTrack,
            json!({ "patterns": ["*.psd"], "opts": { "renormalize": true } }),
        )
        .await;
        let tracked = attrs();
        assert!(tracked.contains("*.psd"), "{name}");
        let staged = rig.repo.git(&["diff", "--cached", "--name-only"]);

        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::LfsTrack, "{name}");
        assert_eq!(attrs(), none, "{name}: .gitattributes is back");
        assert!(rig
            .repo
            .git(&["diff", "--cached", "--name-only"])
            .is_empty());
        rig.redo().await;
        assert_eq!(attrs(), tracked, "{name}");
        assert_eq!(rig.repo.git(&["diff", "--cached", "--name-only"]), staged);

        // Untrack, undo.
        rig.ok(OperationKind::LfsUntrack, json!({ "patterns": ["*.psd"] }))
            .await;
        assert!(!attrs().contains("*.psd"), "{name}");
        rig.undo().await;
        assert!(attrs().contains("*.psd"), "{name}");

        // An outside edit of .gitattributes blocks the undo.
        std::fs::write(rig.path(".gitattributes"), "# mine\n").unwrap();
        let reason = rig.refused(OperationKind::Undo).await;
        assert!(
            matches!(&reason, UndoRefusal::PathChanged { path } if path == ".gitattributes"),
            "{name}: {reason:?}"
        );
        rig.stop().await;
    }
}

// A push git answers "No local changes to save" to leaves nothing to undo.
#[tokio::test]
async fn a_stash_push_with_nothing_to_stash_records_no_entry() {
    for (name, backend) in backends() {
        let clean = || {
            FixtureRepoBuilder::new()
                .unwrap()
                .commit_file("a.txt", "one\n", "first")
                .unwrap()
                .build()
        };
        let cases: Vec<(&str, FixtureRepo, Value)> = vec![
            ("clean tree", clean(), push_args()),
            (
                "named clean path",
                clean(),
                json!({ "message": "wip", "paths": ["a.txt"] }),
            ),
            (
                "untracked only, not included",
                {
                    let r = clean();
                    std::fs::write(r.root().join("new.txt"), "x\n").unwrap();
                    r
                },
                json!({ "message": "wip", "include_untracked": false, "paths": [] }),
            ),
            (
                "stale hint",
                clean(),
                json!({ "message": "wip", "paths": [], "paths_hint": ["a.txt"] }),
            ),
        ];
        for (case, repo, args) in cases {
            let mut rig = Rig::on(backend.clone(), repo).await;
            let _ = rig.call(OperationKind::StashPush, args).await;
            let listing = rig.listing().await;
            assert!(
                listing.undo.is_empty() && listing.redo.is_empty() && listing.retired.is_empty(),
                "{name}/{case}: journal {listing:?}"
            );
            assert!(rig.stash_list().is_empty(), "{name}/{case}");
            rig.stop().await;
        }
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();

#[tokio::test]
async fn a_stash_from_a_commit_is_undone_by_a_drop_and_redone_by_a_store() {
    for (name, backend) in backends() {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", "a\n", "first")
            .unwrap()
            .build();
        std::fs::write(repo.root().join("a.txt"), "a\nlost\n").unwrap();
        repo.git(&["commit", "-q", "-am", "lost work"]);
        let lost = repo.git(&["rev-parse", "HEAD"]);
        repo.git(&["reset", "-q", "--hard", "HEAD~1"]);
        let mut rig = Rig::on(backend, repo).await;

        rig.ok(
            OperationKind::StashFromCommit,
            json!({ "commit": lost, "message": "rescued" }),
        )
        .await;
        let stashed = rig.stash_list();
        assert!(stashed.contains("rescued"), "{name}");
        assert!(rig.status().is_empty(), "{name}: the tree is untouched");

        let out = rig.undo().await;
        assert_eq!(out.entry.kind, OperationKind::StashFromCommit, "{name}");
        assert!(rig.stash_list().is_empty(), "{name}: the stash is gone");
        let out = rig.redo().await;
        assert!(out.applied, "{name}");
        assert_eq!(rig.stash_list(), stashed, "{name}: the very same commit");
        rig.stop().await;
    }
}
