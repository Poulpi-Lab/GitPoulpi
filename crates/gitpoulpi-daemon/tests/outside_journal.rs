//! Changes made outside GitPoulpi (a shell, an AI agent) are recorded in the
//! undo journal (the outside-operations design): found by the
//! ref snapshot diff, one entry per coherent change, never double-recorded
//! for the daemon's own operations, and undone, redone and restored like any
//! other entry. Moves are made with plain `git`; every test runs against both
//! backends.

use std::sync::Arc;

use gitpoulpi_core::domain::{
    CommitOptions, EntrySource, JournalListing, JournalState, ListedEntry, Oid, OperationKind,
    RepoId, RestoreOutcome, StageTarget, UndoOutcome, UndoRefusal,
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
    async fn new(backend: Arc<dyn GitBackend>) -> Rig {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", "one\n", "initial")
            .unwrap()
            .build();
        Rig::on(backend, repo).await
    }

    async fn on(backend: Arc<dyn GitBackend>, repo: FixtureRepo) -> Rig {
        static DEPTH: std::sync::Once = std::sync::Once::new();
        DEPTH.call_once(|| std::env::set_var("GITPOULPI_GENERAL_UNDO_DEPTH", "0"));
        let handle = Daemon::new("outside-test")
            .spawn(backend.clone())
            .await
            .unwrap();
        let client = Client::connect_port(handle.port).await.unwrap();
        let id = repo.repo_ref().id;
        let mut rig = Rig {
            handle: Some(handle),
            client,
            backend,
            repo,
            id,
            next: 1,
        };
        // The first look at a repository is its baseline.
        rig.state().await;
        rig
    }

    /// A new daemon on the same repository (the first one stopped).
    async fn restart(&mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
        let handle = Daemon::new("outside-test")
            .spawn(self.backend.clone())
            .await
            .unwrap();
        self.client = Client::connect_port(handle.port).await.unwrap();
        self.handle = Some(handle);
    }

    async fn call(&mut self, command: OperationKind, args: Value) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "outside-test".into(),
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

    async fn state(&mut self) -> JournalState {
        serde_json::from_value(self.ok(OperationKind::JournalState, json!({})).await).unwrap()
    }

    async fn listing(&mut self) -> JournalListing {
        serde_json::from_value(self.ok(OperationKind::JournalList, json!({})).await).unwrap()
    }

    /// Entries newest first.
    async fn entries(&mut self) -> Vec<ListedEntry> {
        let mut v = self.listing().await.undo;
        v.reverse();
        v
    }

    async fn undo(&mut self, confirmed: bool) -> UndoOutcome {
        serde_json::from_value(
            self.ok(OperationKind::Undo, json!({ "confirmed": confirmed }))
                .await,
        )
        .unwrap()
    }

    async fn redo(&mut self) -> UndoOutcome {
        serde_json::from_value(
            self.ok(OperationKind::Redo, json!({ "confirmed": true }))
                .await,
        )
        .unwrap()
    }

    async fn refusal(&mut self, command: OperationKind, args: Value) -> UndoRefusal {
        let e = self.call(command, args).await.unwrap_err();
        match e.kind {
            Some(TypedError::UndoRefused(r)) => r,
            other => panic!("expected an undo refusal, got {other:?} ({})", e.message),
        }
    }

    fn rev(&self, spec: &str) -> Oid {
        Oid::from_hex(&self.repo.git(&["rev-parse", spec])).unwrap()
    }

    fn has_ref(&self, name: &str) -> bool {
        !self.repo.git(&["for-each-ref", name]).trim().is_empty()
    }

    fn file(&self, name: &str) -> String {
        std::fs::read_to_string(self.repo.root().join(name)).unwrap()
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

fn kinds(entries: &[ListedEntry]) -> Vec<OperationKind> {
    entries.iter().map(|e| e.summary.kind).collect()
}

#[tokio::test]
async fn an_outside_commit_is_an_entry_and_ctrl_z_takes_it_back() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let before = rig.rev("HEAD");
        assert!(rig.entries().await.is_empty(), "{name}: nothing yet");

        std::fs::write(rig.repo.root().join("a.txt"), "two\n").unwrap();
        rig.repo.git(&["commit", "-a", "-m", "agent edit"]);
        let after = rig.rev("HEAD");

        let state = rig.state().await;
        let top = state.undo.expect("an entry");
        assert_eq!(top.kind, OperationKind::Outside, "{name}");
        assert!(top.label.contains("agent edit"), "{name}: {}", top.label);
        let entries = rig.entries().await;
        assert_eq!(entries.len(), 1, "{name}");
        assert_eq!(entries[0].source, EntrySource::Outside, "{name}");
        let info = entries[0].outside.as_ref().expect("who and why");
        assert!(info.message.starts_with("commit: agent edit"), "{name}");
        assert!(info.committer.as_deref().is_some_and(|c| c.contains('<')));
        assert!(
            state.blocked.is_none(),
            "{name}: not blocked: it is the top"
        );

        // Ctrl+Z: the branch and its files go back (`reset --keep`).
        rig.undo(true).await;
        assert_eq!(rig.rev("HEAD"), before, "{name}");
        assert_eq!(rig.file("a.txt"), "one\n", "{name}");
        // The undo is ours: it records nothing new.
        let entries = rig.entries().await;
        assert!(entries.is_empty(), "{name}: {:?}", kinds(&entries));

        rig.redo().await;
        assert_eq!(rig.rev("HEAD"), after, "{name}");
        assert_eq!(rig.file("a.txt"), "two\n", "{name}");
        assert_eq!(
            rig.entries().await.len(),
            1,
            "{name}: redo is not a new entry"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_burst_is_one_entry_and_our_own_operations_are_never_recorded_twice() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        // Our own work first: a branch and a commit through the daemon.
        rig.ok(
            OperationKind::BranchCreate,
            json!({ "name": "mine", "at": rig.rev("HEAD").to_hex() }),
        )
        .await;
        std::fs::write(rig.repo.root().join("b.txt"), "b").unwrap();
        rig.ok(
            OperationKind::StageFile,
            json!({ "target": StageTarget::File("b.txt".into()) }),
        )
        .await;
        rig.ok(
            OperationKind::Commit,
            json!({ "message": "own", "opts": CommitOptions::default() }),
        )
        .await;
        let kinds_now = kinds(&rig.entries().await);
        assert!(
            kinds_now.iter().all(|k| *k != OperationKind::Outside),
            "{name}: {kinds_now:?}"
        );
        let own = kinds_now.len();

        // Then an agent makes five commits between two looks.
        for n in 1..=5 {
            rig.repo
                .git(&["commit", "--allow-empty", "-m", &format!("agent {n}")]);
        }
        let entries = rig.entries().await;
        assert_eq!(entries.len(), own + 1, "{name}: {:?}", kinds(&entries));
        assert_eq!(entries[0].summary.kind, OperationKind::Outside, "{name}");
        assert!(entries[0].summary.label.contains("agent 5"), "{name}");

        // And our next operation lands after it, still not duplicated.
        rig.ok(
            OperationKind::BranchCreate,
            json!({ "name": "later", "at": rig.rev("HEAD").to_hex() }),
        )
        .await;
        let entries = rig.entries().await;
        assert_eq!(entries.len(), own + 2, "{name}: {:?}", kinds(&entries));
        assert_eq!(entries[0].summary.kind, OperationKind::BranchCreate);
        assert_eq!(entries[1].summary.kind, OperationKind::Outside);
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_branch_deleted_outside_comes_back_even_after_the_reflog_and_gc_are_gone() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.repo.git(&["checkout", "-q", "-b", "agent-work"]);
        rig.repo
            .git(&["commit", "--allow-empty", "-m", "unique work"]);
        let tip = rig.rev("HEAD");
        rig.repo.git(&["checkout", "-q", "main"]);
        // Drop what the checkout and creation entries hold: start clean.
        rig.state().await;
        let base = rig.entries().await.len();

        rig.repo.git(&["branch", "-D", "agent-work"]);
        // The reflog of the branch went with it; make sure nothing else
        // keeps the commit, then let git collect garbage.
        let top = rig.state().await.undo.expect("recorded");
        assert!(
            top.label.contains("deleted branch agent-work"),
            "{name}: {}",
            top.label
        );
        rig.repo.git(&["reflog", "expire", "--expire=now", "--all"]);
        rig.repo.git(&["gc", "-q", "--prune=now"]);
        assert!(!rig.has_ref("refs/heads/agent-work"), "{name}");

        rig.undo(false).await;
        assert!(rig.has_ref("refs/heads/agent-work"), "{name}");
        assert_eq!(rig.rev("agent-work"), tip, "{name}");
        assert_eq!(rig.entries().await.len(), base, "{name}");
        rig.redo().await;
        assert!(!rig.has_ref("refs/heads/agent-work"), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn restore_only_this_brings_back_a_deleted_branch_and_keeps_later_work() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.repo.git(&["branch", "keep-me"]);
        rig.state().await;
        let kept = rig.rev("keep-me");
        rig.repo.git(&["branch", "-D", "keep-me"]);
        let id = rig.state().await.undo.expect("recorded").id;

        // Later actions: ours, and another outside one.
        rig.ok(
            OperationKind::BranchCreate,
            json!({ "name": "after", "at": rig.rev("HEAD").to_hex() }),
        )
        .await;
        rig.repo.git(&["tag", "v-later"]);
        rig.state().await;

        let out: RestoreOutcome = serde_json::from_value(
            rig.ok(OperationKind::JournalRestore, json!({ "id": id }))
                .await,
        )
        .unwrap();
        assert!(out.applied, "{name}");
        assert_eq!(rig.rev("keep-me"), kept, "{name}: the branch is back");
        assert!(rig.has_ref("refs/heads/after"), "{name}: later work kept");
        assert!(rig.has_ref("refs/tags/v-later"), "{name}: later work kept");
        rig.stop().await;
    }
}

#[tokio::test]
async fn undoing_an_outside_move_over_a_conflicting_local_edit_refuses_and_changes_nothing() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        let before = rig.rev("HEAD");
        std::fs::write(rig.repo.root().join("a.txt"), "two\n").unwrap();
        rig.repo.git(&["commit", "-a", "-m", "agent edit"]);
        rig.state().await;
        // The same file edited locally since: moving back would overwrite it.
        std::fs::write(rig.repo.root().join("a.txt"), "mine\n").unwrap();

        let refusal = rig
            .refusal(OperationKind::Undo, json!({ "confirmed": true }))
            .await;
        assert!(
            matches!(refusal, UndoRefusal::WouldOverwrite { .. }),
            "{name}: {refusal:?}"
        );
        assert_eq!(rig.file("a.txt"), "mine\n", "{name}: no work discarded");
        assert_ne!(rig.rev("HEAD"), before, "{name}: nothing moved");
        // The entry is still there to undo once the edit is dealt with.
        assert_eq!(rig.entries().await.len(), 1, "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_stash_dropped_outside_is_stored_again_by_undo() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        std::fs::write(rig.repo.root().join("a.txt"), "wip\n").unwrap();
        rig.repo.git(&["stash", "push", "-m", "precious"]);
        rig.state().await;
        let stash = rig.rev("stash@{0}");
        rig.repo.git(&["stash", "drop"]);
        let top = rig.state().await.undo.expect("recorded");
        assert!(
            top.label.starts_with("stash dropped"),
            "{name}: {}",
            top.label
        );
        assert!(rig.repo.git(&["stash", "list"]).trim().is_empty(), "{name}");

        rig.undo(false).await;
        assert_eq!(rig.rev("stash@{0}"), stash, "{name}: the stash is back");
        rig.redo().await;
        assert!(rig.repo.git(&["stash", "list"]).trim().is_empty(), "{name}");
        rig.stop().await;
    }
}

#[tokio::test]
async fn changes_made_while_no_daemon_ran_are_found_at_the_next_start() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.repo.git(&["branch", "gone"]);
        rig.state().await;
        let base = rig.entries().await.len();
        rig.restart().await;
        // Nothing is watching now.
        rig.repo.git(&["branch", "-D", "gone"]);
        rig.repo
            .git(&["commit", "--allow-empty", "-m", "while closed"]);
        let entries = rig.entries().await;
        assert_eq!(entries.len(), base + 2, "{name}: {:?}", kinds(&entries));
        let labels: Vec<&str> = entries.iter().map(|e| e.summary.label.as_str()).collect();
        assert!(
            labels.iter().any(|l| l.contains("deleted branch gone")),
            "{name}: {labels:?}"
        );
        assert!(
            labels.iter().any(|l| l.contains("while closed")),
            "{name}: {labels:?}"
        );
        rig.stop().await;
    }
}

/// A history rewrite (`git filter-repo`) moves every ref, the pins included,
/// and prunes the old objects, while the refs snapshot still names them. The
/// next look records the change but must not pin an object that is gone: a
/// pin to it is an invalid ref (`git fsck`, `git branch -d` complain).
#[tokio::test]
async fn a_change_after_a_history_rewrite_never_pins_a_pruned_object() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.repo.git(&["branch", "rewritten"]);
        rig.repo.git(&["checkout", "-q", "rewritten"]);
        rig.repo
            .git(&["commit", "--allow-empty", "-m", "before the rewrite"]);
        rig.repo.git(&["checkout", "-q", "main"]);
        rig.state().await;
        let old = rig.rev("rewritten");
        rig.restart().await;

        // What a rewrite leaves: the branch and every pin on the new
        // commit, the old one pruned.
        rig.repo.git(&["checkout", "-q", "rewritten"]);
        rig.repo.git(&[
            "commit",
            "--amend",
            "--allow-empty",
            "-m",
            "after the rewrite",
        ]);
        rig.repo.git(&["checkout", "-q", "main"]);
        let new = rig.rev("rewritten").to_string();
        let pins = rig
            .repo
            .git(&["for-each-ref", "--format=%(refname)", "refs/gitpoulpi/"]);
        for pin in pins.lines().filter(|l| !l.is_empty()) {
            rig.repo.git(&["update-ref", pin, &new]);
        }
        rig.repo.git(&["reflog", "expire", "--expire=now", "--all"]);
        rig.repo.git(&["gc", "-q", "--prune=now"]);
        assert!(
            rig.repo
                .try_git(&["cat-file", "-e", &old.to_string()])
                .is_none(),
            "{name}: the old commit must be pruned"
        );

        rig.repo.git(&["branch", "-D", "rewritten"]);
        let labels: Vec<String> = rig
            .entries()
            .await
            .into_iter()
            .map(|e| e.summary.label)
            .collect();
        assert!(
            labels
                .iter()
                .any(|l| l.contains("deleted branch rewritten")),
            "{name}: {labels:?}"
        );
        let pinned = rig
            .repo
            .git(&["for-each-ref", "--format=%(objectname)", "refs/gitpoulpi/"]);
        assert!(
            !pinned.contains(&old.to_string()),
            "{name}: a pin names the pruned object"
        );
        assert!(
            rig.repo
                .try_git(&["fsck", "--connectivity-only", "--no-dangling"])
                .is_some(),
            "{name}: the repository has invalid refs"
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn a_rebase_that_stops_is_recorded_once_it_ends_and_as_one_entry() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.repo.git(&["checkout", "-q", "-b", "topic"]);
        std::fs::write(rig.repo.root().join("a.txt"), "topic\n").unwrap();
        rig.repo.git(&["commit", "-a", "-m", "topic edit"]);
        rig.repo.git(&["checkout", "-q", "main"]);
        std::fs::write(rig.repo.root().join("a.txt"), "main\n").unwrap();
        rig.repo.git(&["commit", "-a", "-m", "main edit"]);
        rig.repo.git(&["checkout", "-q", "topic"]);
        rig.state().await;
        let base = rig.entries().await.len();

        // The rebase stops on a conflict: nothing is recorded mid-way.
        assert!(rig.repo.try_git(&["rebase", "main"]).is_none(), "{name}");
        let mid = rig.entries().await;
        assert_eq!(mid.len(), base, "{name}: {:?}", kinds(&mid));
        // The agent resolves it and goes on.
        std::fs::write(rig.repo.root().join("a.txt"), "both\n").unwrap();
        rig.repo.git(&["add", "a.txt"]);
        rig.repo
            .git(&["-c", "core.editor=true", "rebase", "--continue"]);
        let done = rig.entries().await;
        assert_eq!(done.len(), base + 1, "{name}: {:?}", kinds(&done));
        assert_eq!(done[0].summary.kind, OperationKind::Outside, "{name}");
        assert!(
            // git before 2.35 names the action "rebase (continue) (finish)".
            done[0].summary.label.starts_with("rebase (finish)")
                || done[0]
                    .summary
                    .label
                    .starts_with("rebase (continue) (finish)"),
            "{name}: {}",
            done[0].summary.label
        );
        rig.stop().await;
    }
}

#[tokio::test]
async fn an_outside_checkout_is_undone_by_checking_the_old_branch_out_again() {
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend).await;
        rig.repo.git(&["branch", "other"]);
        rig.state().await;
        rig.repo.git(&["checkout", "-q", "other"]);
        let top = rig.state().await.undo.expect("recorded");
        assert!(top.label.starts_with("checkout:"), "{name}: {}", top.label);
        rig.undo(true).await;
        assert_eq!(
            rig.repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
            "main",
            "{name}"
        );
        rig.stop().await;
    }
}

/// A reftable repository has no loose refs and no `logs/`: the fingerprint
/// that decides whether to compare ref snapshots must still see a change made
/// in a shell. (Only the system backend: libgit2 cannot open such a repo.)
#[tokio::test]
async fn an_outside_change_is_found_in_a_reftable_repository() {
    let Ok(builder) = FixtureRepoBuilder::new_reftable() else {
        eprintln!("SKIPPED: this git cannot create a reftable repository");
        return;
    };
    let repo = builder
        .commit_file("a.txt", "one\n", "initial")
        .unwrap()
        .build();
    let backend: Arc<dyn GitBackend> = Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new());
    let mut rig = Rig::on(backend, repo).await;
    let before = rig.rev("HEAD");
    // Let the baseline age past the fingerprint's "too recent to tell" window.
    tokio::time::sleep(std::time::Duration::from_millis(3200)).await;
    assert!(rig.entries().await.is_empty());

    std::fs::write(rig.repo.root().join("a.txt"), "two\n").unwrap();
    rig.repo.git(&["commit", "-a", "-m", "agent edit"]);
    let entries = rig.entries().await;
    assert_eq!(kinds(&entries), vec![OperationKind::Outside]);
    assert!(entries[0].summary.label.contains("agent edit"));

    // A change that moves no branch, only adds a ref, is seen as well.
    tokio::time::sleep(std::time::Duration::from_millis(3200)).await;
    rig.repo.git(&["branch", "side"]);
    let entries = rig.entries().await;
    assert_eq!(entries.len(), 2, "{:?}", kinds(&entries));
    rig.undo(true).await;
    rig.undo(true).await;
    assert_eq!(rig.rev("HEAD"), before);
    rig.stop().await;
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
