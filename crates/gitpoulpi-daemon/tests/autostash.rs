//! `general.auto_stash` through the daemon: a checkout, merge, rebase or
//! pull blocked by local changes is refused (`never`), answered with the
//! typed `NeedsAutoStash` (`ask`), or stashed, run and popped (`always`).
//! The env var is process-global, so everything runs in one test.

use std::sync::Arc;

use gitpoulpi_core::domain::{OperationKind, RepoId};
use gitpoulpi_core::{GitBackend, TypedError};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request, ResponseError};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};

const ENV: &str = "GITPOULPI_GENERAL_AUTO_STASH";
/// Serializes the tests that set `ENV` (it is process-global).
static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
const BASE: &str = "1\n2\n3\n4\n5\n";

struct Rig {
    handle: Option<gitpoulpi_daemon::DaemonHandle>,
    client: Client,
    repo: FixtureRepo,
    id: RepoId,
    next: u64,
}

impl Rig {
    async fn new(backend: Arc<dyn GitBackend>, other_edit: &str) -> Rig {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", BASE, "base")
            .unwrap()
            .branch("other")
            .unwrap()
            .checkout("other")
            .unwrap()
            .commit_file("a.txt", other_edit, "other edit")
            .unwrap()
            .checkout("main")
            .unwrap()
            .build();
        Rig::with_repo(backend, repo).await
    }

    async fn with_repo(backend: Arc<dyn GitBackend>, repo: FixtureRepo) -> Rig {
        let handle = Daemon::new("autostash").spawn(backend).await.unwrap();
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
                session: "autostash".into(),
                repo: Some(self.id.clone()),
                command,
                args,
            })
            .await
            .unwrap()
            .result
    }

    fn dirty_last_line(&self) {
        std::fs::write(self.repo.root().join("a.txt"), "1\n2\n3\n4\n5 local\n").unwrap();
    }

    fn branch(&self) -> String {
        self.repo
            .git(&["rev-parse", "--abbrev-ref", "HEAD"])
            .trim()
            .to_string()
    }

    fn file(&self) -> String {
        std::fs::read_to_string(self.repo.root().join("a.txt")).unwrap()
    }

    fn b(&self) -> String {
        std::fs::read_to_string(self.repo.root().join("b.txt")).unwrap()
    }

    fn stashes(&self) -> String {
        self.repo.git(&["stash", "list"])
    }

    async fn stop(mut self) {
        self.handle.take().unwrap().shutdown().await.unwrap();
    }
}

fn backends() -> Vec<(&'static str, Arc<dyn GitBackend>)> {
    vec![
        (
            "system",
            Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new()),
        ),
        ("git2", Arc::new(gitpoulpi_git2_backend::Git2Backend::new())),
    ]
}

fn to_other() -> Value {
    json!({ "target": { "Branch": "other" } })
}

#[tokio::test]
async fn auto_stash_modes_for_a_blocked_checkout() {
    let _env = ENV_LOCK.lock().await;
    std::env::remove_var(ENV);
    for (name, backend) in backends() {
        // other edits line 1; the local edit is on line 5: blocked, and
        // the pop merges cleanly.
        let mut rig = Rig::new(backend.clone(), "1 other\n2\n3\n4\n5\n").await;
        rig.dirty_last_line();

        // never: refused as today, nothing stashed, edit kept.
        std::env::set_var(ENV, "never");
        let e = rig
            .call(OperationKind::Checkout, to_other())
            .await
            .unwrap_err();
        assert!(e.kind.is_none(), "{name}: {e:?}");
        assert_eq!(rig.branch(), "main", "{name}");
        assert!(rig.stashes().is_empty(), "{name}");

        // ask: the typed answer, still nothing changed.
        std::env::set_var(ENV, "ask");
        let e = rig
            .call(OperationKind::Checkout, to_other())
            .await
            .unwrap_err();
        assert!(
            matches!(&e.kind, Some(TypedError::NeedsAutoStash { operation }) if operation == "checkout"),
            "{name}: {e:?}"
        );
        assert_eq!(rig.branch(), "main", "{name}");
        assert!(rig.stashes().is_empty(), "{name}");
        assert_eq!(rig.file(), "1\n2\n3\n4\n5 local\n", "{name}");

        // ask + the request's own answer: stash, checkout, pop.
        let mut args = to_other();
        args["autostash"] = json!("always");
        rig.call(OperationKind::Checkout, args)
            .await
            .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        assert_eq!(rig.branch(), "other", "{name}");
        assert_eq!(rig.file(), "1 other\n2\n3\n4\n5 local\n", "{name}");
        assert!(rig.stashes().is_empty(), "{name}: popped");
        let state = rig.repo.state_dir().join("repo-state.toml");
        let text = std::fs::read_to_string(&state).unwrap_or_default();
        assert!(!text.contains("pending_auto_stash"), "{name}: {text}");
        rig.stop().await;

        // always from the config, and the undo journal covers the steps.
        std::env::set_var(ENV, "always");
        let mut rig = Rig::new(backend.clone(), "1 other\n2\n3\n4\n5\n").await;
        rig.dirty_last_line();
        rig.call(OperationKind::Checkout, to_other())
            .await
            .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        assert_eq!(rig.branch(), "other", "{name}");
        assert_eq!(rig.file(), "1 other\n2\n3\n4\n5 local\n", "{name}");
        let listing = rig
            .call(OperationKind::JournalList, json!({}))
            .await
            .unwrap();
        let kinds: Vec<String> = listing["undo"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|e| {
                        e["summary"]["kind"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string()
                    })
                    .collect()
            })
            .unwrap_or_default();
        for expected in ["StashPush", "Checkout", "StashPop"] {
            assert!(
                kinds.iter().any(|k| k == expected),
                "{name}: journal {kinds:?} lacks {expected}"
            );
        }
        rig.stop().await;

        // A clean tree is never touched: a bad branch name is an ordinary error.
        let mut rig = Rig::new(backend.clone(), "1 other\n2\n3\n4\n5\n").await;
        let e = rig
            .call(
                OperationKind::Checkout,
                json!({ "target": { "Branch": "nope" } }),
            )
            .await
            .unwrap_err();
        assert!(e.kind.is_none(), "{name}");
        assert!(rig.stashes().is_empty(), "{name}");
        rig.stop().await;

        // A pop that conflicts: the checkout stands, the stash stays, and
        // the pending record says so.
        let mut rig = Rig::new(backend.clone(), "1\n2\n3\n4\n5 other\n").await;
        rig.dirty_last_line();
        let e = rig
            .call(OperationKind::Checkout, to_other())
            .await
            .unwrap_err();
        assert!(e.message.contains("stash"), "{name}: {}", e.message);
        assert_eq!(rig.branch(), "other", "{name}");
        assert!(!rig.stashes().is_empty(), "{name}: the stash is kept");
        let text = std::fs::read_to_string(rig.repo.state_dir().join("repo-state.toml"))
            .unwrap_or_default();
        assert!(text.contains("Conflicted"), "{name}: {text}");
        rig.stop().await;
    }
    std::env::remove_var(ENV);
}

#[tokio::test]
async fn auto_stash_covers_merge_and_rebase_too() {
    // Separate file-level env state is shared with the test above, so
    // this one passes the mode in the request instead.
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend.clone(), "1 other\n2\n3\n4\n5\n").await;
        rig.dirty_last_line();
        // Merge is driven by `Ref`; take its exact JSON from the backend.
        let refs = rig.call(OperationKind::Refs, json!({})).await.unwrap();
        let other = refs
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["kind"]["LocalBranch"]["name"] == "other")
            .unwrap()
            .clone();
        let r = rig
            .call(
                OperationKind::Merge,
                json!({ "from": other, "opts": {}, "autostash": "always" }),
            )
            .await;
        assert!(r.is_ok(), "{name}: merge {r:?}");
        assert_eq!(rig.file(), "1 other\n2\n3\n4\n5 local\n", "{name}");
        assert!(rig.stashes().is_empty(), "{name}");
        rig.stop().await;

        let mut rig = Rig::new(backend.clone(), "1 other\n2\n3\n4\n5\n").await;
        rig.repo.git(&["checkout", "-q", "-b", "feature"]);
        std::fs::write(rig.repo.root().join("f.txt"), "f\n").unwrap();
        rig.repo.git(&["add", "f.txt"]);
        rig.repo.git(&["commit", "-q", "-m", "feature work"]);
        rig.dirty_last_line();
        let refs = rig.call(OperationKind::Refs, json!({})).await.unwrap();
        let other = refs
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["kind"]["LocalBranch"]["name"] == "other")
            .unwrap()
            .clone();
        let r = rig
            .call(
                OperationKind::Rebase,
                json!({ "onto": other, "opts": {}, "autostash": "always" }),
            )
            .await;
        assert!(r.is_ok(), "{name}: rebase {r:?}");
        assert_eq!(rig.file(), "1 other\n2\n3\n4\n5 local\n", "{name}");
        assert!(rig.stashes().is_empty(), "{name}");
        rig.stop().await;
    }
}

/// A repo whose `origin/main` has moved on with `remote_edit` of `a.txt`
/// (pushed from a scratch clone), plus a rig for it. The local `main`
/// still has the base commit and a dirty last line, so a pull is blocked.
async fn pull_rig(backend: Arc<dyn GitBackend>, remote_edit: &str) -> Rig {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", BASE, "base")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .build();
    let scratch = repo.root().parent().unwrap().join("scratch-clone");
    let remote = repo.remote_path().unwrap().to_string_lossy().to_string();
    let scratch_str = scratch.to_string_lossy().to_string();
    repo.git_at(repo.root(), &["clone", "-q", &remote, &scratch_str]);
    std::fs::write(scratch.join("a.txt"), remote_edit).unwrap();
    repo.git_at(&scratch, &["add", "a.txt"]);
    repo.git_at(
        &scratch,
        &[
            "-c",
            "user.name=Else",
            "-c",
            "user.email=else@x.test",
            "commit",
            "-q",
            "-m",
            "remote edit",
        ],
    );
    repo.git_at(&scratch, &["push", "-q", "origin", "main"]);
    let handle = Daemon::new("autostash").spawn(backend).await.unwrap();
    let client = Client::connect(&handle.endpoint()).await.unwrap();
    let id = repo.repo_ref().id;
    let rig = Rig {
        handle: Some(handle),
        client,
        repo,
        id,
        next: 1,
    };
    rig.dirty_last_line();
    rig
}

/// Waits for the next `PendingAutoStashChanged` for `repo` on `watcher`.
async fn next_pending_event(watcher: &mut tokio::net::TcpStream, repo: &RepoId) {
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            let msg: gitpoulpi_ipc::ServerMessage = gitpoulpi_ipc::transport::read_message(watcher)
                .await
                .unwrap()
                .expect("connection closed before the event arrived");
            if let gitpoulpi_ipc::ServerMessage::Event(
                gitpoulpi_ipc::Event::PendingAutoStashChanged { repo: changed },
            ) = msg
            {
                if &changed == repo {
                    return;
                }
            }
        }
    })
    .await
    .expect("timed out waiting for PendingAutoStashChanged");
}

#[tokio::test]
async fn auto_stash_for_a_blocked_pull_and_its_live_event() {
    let _env = ENV_LOCK.lock().await;
    std::env::remove_var(ENV);
    for (name, backend) in backends() {
        // The remote edits line 1, the local edit is on line 5: the pull
        // is blocked, and the pop merges cleanly.
        let remote_edit = "1 remote\n2\n3\n4\n5\n";

        // ask (the default): the typed answer, nothing changed.
        std::env::set_var(ENV, "ask");
        let mut rig = pull_rig(backend.clone(), remote_edit).await;
        let e = rig.call(OperationKind::Pull, json!({})).await.unwrap_err();
        assert!(
            matches!(&e.kind, Some(TypedError::NeedsAutoStash { operation }) if operation == "pull"),
            "{name}: {e:?}"
        );
        assert!(rig.stashes().is_empty(), "{name}");
        assert_eq!(rig.file(), "1\n2\n3\n4\n5 local\n", "{name}");

        // ask + the request's own answer (what `--autostash` sends).
        let mut watcher = Client::connect(&rig.handle.as_ref().unwrap().endpoint())
            .await
            .unwrap()
            .into_stream();
        rig.call(OperationKind::Pull, json!({ "autostash": "always" }))
            .await
            .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        assert_eq!(rig.file(), "1 remote\n2\n3\n4\n5 local\n", "{name}");
        assert!(rig.stashes().is_empty(), "{name}: popped");
        next_pending_event(&mut watcher, &rig.id).await;
        let text = std::fs::read_to_string(rig.repo.state_dir().join("repo-state.toml"))
            .unwrap_or_default();
        assert!(!text.contains("pending_auto_stash"), "{name}: {text}");
        rig.stop().await;

        // always from the config; three journal entries.
        std::env::set_var(ENV, "always");
        let mut rig = pull_rig(backend.clone(), remote_edit).await;
        rig.call(OperationKind::Pull, json!({}))
            .await
            .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        assert_eq!(rig.file(), "1 remote\n2\n3\n4\n5 local\n", "{name}");
        assert!(rig.stashes().is_empty(), "{name}");
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
        for expected in ["StashPush", "Pull", "StashPop"] {
            assert!(
                kinds.contains(&expected),
                "{name}: {kinds:?} lacks {expected}"
            );
        }
        rig.stop().await;

        // A pop that conflicts: the pull stands, the stash stays, the
        // pending record says Conflicted, and the event fires.
        let mut rig = pull_rig(backend.clone(), "1\n2\n3\n4\n5 remote\n").await;
        let mut watcher = Client::connect(&rig.handle.as_ref().unwrap().endpoint())
            .await
            .unwrap()
            .into_stream();
        let e = rig.call(OperationKind::Pull, json!({})).await.unwrap_err();
        assert!(e.message.contains("stash"), "{name}: {}", e.message);
        assert!(!rig.stashes().is_empty(), "{name}: the stash is kept");
        next_pending_event(&mut watcher, &rig.id).await;
        let text = std::fs::read_to_string(rig.repo.state_dir().join("repo-state.toml"))
            .unwrap_or_default();
        assert!(text.contains("Conflicted"), "{name}: {text}");
        rig.stop().await;
    }
    std::env::remove_var(ENV);
}

/// A user's own pop or drop of the pending auto-stash (the TUI banner's
/// "Pop now" / "Drop", or `gitpoulpi stash pop|drop`) clears the record
/// and tells every client, so the banner goes away live.
#[tokio::test]
async fn a_manual_pop_or_drop_of_the_pending_stash_clears_the_record_and_fires_the_event() {
    let _env = ENV_LOCK.lock().await;
    std::env::set_var(ENV, "always");
    for (name, backend) in backends() {
        for command in [OperationKind::StashPop, OperationKind::StashDrop] {
            // other edits line 1, the local edit is on line 1 too: the
            // checkout goes through, the pop conflicts, the record stays.
            let mut rig = Rig::new(backend.clone(), "1 other\n2\n3\n4\n5\n").await;
            std::fs::write(rig.repo.root().join("a.txt"), "1 local\n2\n3\n4\n5\n").unwrap();
            let e = rig
                .call(OperationKind::Checkout, to_other())
                .await
                .unwrap_err();
            assert!(e.message.contains("stash"), "{name}: {}", e.message);
            let state_file = rig.repo.state_dir().join("repo-state.toml");
            let record = move || std::fs::read_to_string(&state_file).unwrap_or_default();
            assert!(record().contains("Conflicted"), "{name}: {}", record());

            // Back to a state where the stash applies cleanly.
            rig.repo.git(&["reset", "-q", "--hard"]);
            rig.repo.git(&["checkout", "-q", "main"]);
            let mut watcher = Client::connect(&rig.handle.as_ref().unwrap().endpoint())
                .await
                .unwrap()
                .into_stream();
            rig.call(command, json!({ "id": 0 }))
                .await
                .unwrap_or_else(|e| panic!("{name} {command:?}: {}", e.message));
            next_pending_event(&mut watcher, &rig.id).await;
            assert!(
                !record().contains("pending_auto_stash"),
                "{name} {command:?}: {}",
                record()
            );
            if command == OperationKind::StashPop {
                assert_eq!(rig.file(), "1 local\n2\n3\n4\n5\n", "{name}");
            }
            rig.stop().await;
        }
    }
    std::env::remove_var(ENV);
}

/// While a pending auto-stash record exists, the operations that would
/// bury or tangle it are refused with the typed `PendingAutoStash`; reads
/// and the resolving operations stay allowed, and popping the stash lifts
/// the block.
#[tokio::test]
async fn a_pending_auto_stash_blocks_history_moving_operations_until_popped() {
    let _env = ENV_LOCK.lock().await;
    std::env::set_var(ENV, "always");
    let blocked = [
        (OperationKind::Checkout, to_other(), "checkout"),
        (OperationKind::Merge, json!({}), "merge"),
        (OperationKind::Rebase, json!({}), "rebase"),
        (OperationKind::RebaseInteractive, json!({}), "rebase"),
        (OperationKind::Pull, json!({}), "pull"),
        (OperationKind::FastForward, json!({}), "fast-forward"),
        (OperationKind::CherryPick, json!({}), "cherry-pick"),
        (OperationKind::Revert, json!({}), "revert"),
        (OperationKind::Reword, json!({}), "reword"),
        (OperationKind::AmendCommit, json!({}), "commit amend"),
        (OperationKind::SplitCommit, json!({}), "commit split"),
        (OperationKind::RewriteApply, json!({}), "history rewrite"),
        (
            OperationKind::BranchCreateTracking,
            json!({}),
            "branch checkout",
        ),
        (OperationKind::Undo, json!({}), "undo"),
        (OperationKind::Redo, json!({}), "redo"),
    ];
    for (name, backend) in backends() {
        let mut rig = Rig::new(backend.clone(), "1 other\n2\n3\n4\n5\n").await;
        std::fs::write(rig.repo.root().join("a.txt"), "1 local\n2\n3\n4\n5\n").unwrap();
        let e = rig
            .call(OperationKind::Checkout, to_other())
            .await
            .unwrap_err();
        assert!(e.message.contains("stash"), "{name}: {}", e.message);
        assert_eq!(rig.branch(), "other", "{name}");
        // The conflicted pop leaves a record: put the tree back so the
        // stash applies again, as a user resolving it would.
        rig.repo.git(&["reset", "-q", "--hard"]);
        rig.repo.git(&["checkout", "-q", "main"]);

        for (command, args, word) in &blocked {
            let e = rig.call(*command, args.clone()).await.unwrap_err();
            assert!(
                matches!(&e.kind, Some(TypedError::PendingAutoStash { operation, stash, hint })
                    if operation == word && stash.0 == 0 && !hint.is_empty()),
                "{name} {command:?}: {e:?}"
            );
        }
        // Even a would-be auto-stash does not start another one.
        let mut args = to_other();
        args["autostash"] = json!("always");
        assert!(rig
            .call(OperationKind::Checkout, args)
            .await
            .unwrap_err()
            .kind
            .is_some());
        assert_eq!(rig.branch(), "main", "{name}: nothing moved");

        // Reads, staging and checking out a single file stay possible.
        for command in [
            OperationKind::Status,
            OperationKind::Stashes,
            OperationKind::Refs,
            OperationKind::Conflicts,
        ] {
            rig.call(command, json!({}))
                .await
                .unwrap_or_else(|e| panic!("{name} {command:?}: {}", e.message));
        }
        let head = rig.repo.git(&["rev-parse", "HEAD"]).trim().to_string();
        rig.call(
            OperationKind::Checkout,
            json!({ "target": { "File": { "path": "a.txt", "source": head } } }),
        )
        .await
        .unwrap_or_else(|e| panic!("{name}: file restore: {}", e.message));

        // Popping resolves it, and the operations work again.
        rig.call(OperationKind::StashPop, json!({ "id": 0 }))
            .await
            .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        rig.repo.git(&["checkout", "-q", "--", "a.txt"]);
        rig.call(OperationKind::Checkout, to_other())
            .await
            .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        assert_eq!(rig.branch(), "other", "{name}");
        rig.stop().await;
    }
    std::env::remove_var(ENV);
}

/// A rig with a pending record: a checkout whose auto-stash pop conflicts,
/// then the tree put back to `main` so the stash applies cleanly again.
async fn pending_rig(backend: Arc<dyn GitBackend>) -> Rig {
    let mut rig = Rig::new(backend, "1 other\n2\n3\n4\n5\n").await;
    std::fs::write(rig.repo.root().join("a.txt"), "1 local\n2\n3\n4\n5\n").unwrap();
    let e = rig
        .call(OperationKind::Checkout, to_other())
        .await
        .unwrap_err();
    assert!(e.message.contains("stash"), "{}", e.message);
    rig.repo.git(&["reset", "-q", "--hard"]);
    rig.repo.git(&["checkout", "-q", "main"]);
    rig
}

fn record(rig: &Rig) -> String {
    std::fs::read_to_string(rig.repo.state_dir().join("repo-state.toml")).unwrap_or_default()
}

/// Waits for the next `PendingAutoStashGone` for `repo` on `watcher`.
async fn next_gone_event(watcher: &mut tokio::net::TcpStream, repo: &RepoId) {
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            let msg: gitpoulpi_ipc::ServerMessage = gitpoulpi_ipc::transport::read_message(watcher)
                .await
                .unwrap()
                .expect("connection closed before the event arrived");
            if let gitpoulpi_ipc::ServerMessage::Event(
                gitpoulpi_ipc::Event::PendingAutoStashGone { repo: changed, .. },
            ) = msg
            {
                if &changed == repo {
                    return;
                }
            }
        }
    })
    .await
    .expect("timed out waiting for PendingAutoStashGone");
}

/// Stuck state #2/#4: the stash dropped with plain git. The record must go
/// (with an event saying so) instead of refusing every history operation
/// forever; Pop on the vanished stash fails without touching another.
#[tokio::test]
async fn a_stash_dropped_outside_clears_the_record_and_unblocks() {
    let _env = ENV_LOCK.lock().await;
    std::env::set_var(ENV, "always");
    for (name, backend) in backends() {
        let mut rig = pending_rig(backend).await;
        assert!(record(&rig).contains("pending_auto_stash"), "{name}");
        assert!(
            record(&rig).contains("stash_commit"),
            "{name}: {}",
            record(&rig)
        );
        let mut watcher = Client::connect(&rig.handle.as_ref().unwrap().endpoint())
            .await
            .unwrap()
            .into_stream();
        rig.repo.git(&["stash", "drop", "-q"]);

        // A history operation is no longer refused, and says nothing of
        // a stash.
        rig.call(OperationKind::Checkout, to_other())
            .await
            .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        assert_eq!(rig.branch(), "other", "{name}");
        assert!(!record(&rig).contains("pending_auto_stash"), "{name}");
        next_gone_event(&mut watcher, &rig.id).await;
        rig.stop().await;
    }
    std::env::remove_var(ENV);
}

/// A refresh read alone clears the record (the TUI's banner goes away
/// without the user trying anything).
#[tokio::test]
async fn a_refresh_read_clears_a_record_whose_stash_is_gone() {
    let _env = ENV_LOCK.lock().await;
    std::env::set_var(ENV, "always");
    for (name, backend) in backends() {
        let mut rig = pending_rig(backend).await;
        let mut watcher = Client::connect(&rig.handle.as_ref().unwrap().endpoint())
            .await
            .unwrap()
            .into_stream();
        rig.repo.git(&["stash", "drop", "-q"]);
        rig.call(OperationKind::Stashes, json!({})).await.unwrap();
        assert!(!record(&rig).contains("pending_auto_stash"), "{name}");
        next_pending_event(&mut watcher, &rig.id).await;
        rig.stop().await;
    }
    std::env::remove_var(ENV);
}

/// `Refs` reconciles like `Status` and `Stashes`, but it is answered through
/// the read cache: when it is the read that finds the stash gone it clears
/// the record, and the events must still go out (the other reads then find
/// nothing left to report, so a lost notice leaves the banner up for good).
#[tokio::test]
async fn a_refs_read_that_clears_the_record_still_tells_the_clients() {
    let _env = ENV_LOCK.lock().await;
    std::env::set_var(ENV, "always");
    for (name, backend) in backends() {
        let mut rig = pending_rig(backend).await;
        let mut watcher = Client::connect(&rig.handle.as_ref().unwrap().endpoint())
            .await
            .unwrap()
            .into_stream();
        rig.repo.git(&["stash", "drop", "-q"]);
        rig.call(OperationKind::Refs, json!({})).await.unwrap();
        assert!(!record(&rig).contains("pending_auto_stash"), "{name}");
        next_gone_event(&mut watcher, &rig.id).await;
        rig.stop().await;
    }
    std::env::remove_var(ENV);
}

/// Stuck state #2: a stash pushed elsewhere shifts the index. The record
/// follows its commit, and a pop by commit takes the right stash.
#[tokio::test]
async fn a_shifted_stash_index_is_followed_by_commit() {
    let _env = ENV_LOCK.lock().await;
    std::env::set_var(ENV, "always");
    for (name, backend) in backends() {
        let mut rig = pending_rig(backend).await;
        let auto = rig.repo.git(&["rev-parse", "stash@{0}"]).trim().to_string();
        // Another stash lands on top: the auto-stash is now stash@{1}.
        std::fs::write(rig.repo.root().join("a.txt"), "1\n2\n3\n4\n5 foreign\n").unwrap();
        rig.repo.git(&["stash", "push", "-q", "-m", "foreign"]);
        rig.call(OperationKind::Status, json!({})).await.unwrap();
        assert!(
            record(&rig).contains("stash_id = 1"),
            "{name}: {}",
            record(&rig)
        );

        // The banner pops "its" stash: id 0 is stale, the commit decides.
        rig.call(OperationKind::StashPop, json!({ "id": 0, "commit": auto }))
            .await
            .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        assert_eq!(rig.file(), "1 local\n2\n3\n4\n5\n", "{name}");
        assert!(
            rig.stashes().contains("foreign"),
            "{name}: {}",
            rig.stashes()
        );
        assert!(!record(&rig).contains("pending_auto_stash"), "{name}");

        // Popping by a commit that is gone fails and touches nothing.
        let e = rig
            .call(OperationKind::StashDrop, json!({ "id": 0, "commit": auto }))
            .await
            .unwrap_err();
        assert!(e.message.contains("no longer"), "{name}: {}", e.message);
        assert!(rig.stashes().contains("foreign"), "{name}");
        rig.stop().await;
    }
    std::env::remove_var(ENV);
}

/// A record from before commit ids were stored (index only) is resolved
/// once and gets its commit; an old record whose stash is gone is cleared.
#[tokio::test]
async fn an_index_only_record_is_migrated() {
    let _env = ENV_LOCK.lock().await;
    std::env::set_var(ENV, "always");
    for (name, backend) in backends() {
        let mut rig = pending_rig(backend).await;
        let path = rig.repo.state_dir().join("repo-state.toml");
        let old: String = record(&rig)
            .lines()
            .filter(|l| !l.starts_with("stash_commit"))
            .map(|l| format!("{l}\n"))
            .collect();
        assert!(!old.contains("stash_commit"), "{name}");
        std::fs::write(&path, &old).unwrap();
        rig.call(OperationKind::Status, json!({})).await.unwrap();
        assert!(
            record(&rig).contains("stash_commit"),
            "{name}: {}",
            record(&rig)
        );

        // Old record, stash dropped outside: cleared.
        std::fs::write(&path, &old).unwrap();
        rig.repo.git(&["stash", "drop", "-q"]);
        rig.call(OperationKind::Status, json!({})).await.unwrap();
        assert!(!record(&rig).contains("pending_auto_stash"), "{name}");
        rig.stop().await;
    }
    std::env::remove_var(ENV);
}

const INCLUDE_UNTRACKED: &str = "GITPOULPI_GENERAL_INCLUDE_UNTRACKED_IN_AUTOSTASH";

/// `other` adds `u.txt`; the working tree holds an untracked `u.txt`: the
/// checkout is blocked by it, the auto-stash (untracked included) takes it,
/// and putting it back fails with "already exists" - nothing is applied.
async fn pop_failed_rig(backend: Arc<dyn GitBackend>) -> Rig {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", BASE, "base")
        .unwrap()
        .branch("other")
        .unwrap()
        .checkout("other")
        .unwrap()
        .commit_file("u.txt", "tracked on other\n", "add u")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("u.txt"), "mine\n").unwrap();
    let mut rig = Rig::with_repo(backend, repo).await;
    let e = rig
        .call(OperationKind::Checkout, to_other())
        .await
        .unwrap_err();
    assert!(e.message.contains("stash"), "{}", e.message);
    rig
}

/// Stuck state #1: a pop that fails with nothing applied is `PopFailed`
/// (not `Conflicted`), keeps the stash, and *Retry pop* works once the
/// obstacle is gone.
#[tokio::test]
async fn a_failed_pop_with_nothing_applied_is_pop_failed_and_can_be_retried() {
    let _env = ENV_LOCK.lock().await;
    std::env::set_var(ENV, "always");
    std::env::set_var(INCLUDE_UNTRACKED, "true");
    for (name, backend) in backends() {
        let mut rig = pop_failed_rig(backend).await;
        assert_eq!(rig.branch(), "other", "{name}");
        let rec = record(&rig);
        assert!(rec.contains("PopFailed"), "{name}: {rec}");
        assert!(!rec.contains("Conflicted"), "{name}: {rec}");
        assert!(rec.contains("error"), "{name}: {rec}");
        assert!(!rig.stashes().is_empty(), "{name}");

        // Retry while the obstacle is still there: fails, stays PopFailed.
        rig.call(OperationKind::StashPop, json!({ "id": 0 }))
            .await
            .unwrap_err();
        assert!(record(&rig).contains("PopFailed"), "{name}");
        assert!(!rig.stashes().is_empty(), "{name}");

        // Remove the obstacle (back to main: u.txt is not tracked there).
        rig.repo.git(&["checkout", "-q", "main"]);
        rig.call(OperationKind::StashPop, json!({ "id": 0 }))
            .await
            .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        assert!(!record(&rig).contains("pending_auto_stash"), "{name}");
        assert!(rig.stashes().is_empty(), "{name}");
        assert_eq!(
            std::fs::read_to_string(rig.repo.root().join("u.txt")).unwrap(),
            "mine\n",
            "{name}"
        );
        rig.stop().await;
    }
    std::env::remove_var(INCLUDE_UNTRACKED);
    std::env::remove_var(ENV);
}

/// Stuck state #8: a manual pop of the pending stash that conflicts leaves
/// the conflicts in the tree: the record becomes `Conflicted`.
#[tokio::test]
async fn a_manual_pop_that_conflicts_makes_the_record_conflicted() {
    let _env = ENV_LOCK.lock().await;
    std::env::set_var(ENV, "always");
    for (name, backend) in backends() {
        // pending_rig leaves a clean `main`, where the stash applies; the
        // record is Conflicted from the checkout. Make it `Applied` by
        // hand, then move `main` so the pop conflicts.
        let mut rig = pending_rig(backend).await;
        let state_file = rig.repo.state_dir().join("repo-state.toml");
        let text = std::fs::read_to_string(&state_file).unwrap();
        std::fs::write(&state_file, text.replace("Conflicted", "Applied")).unwrap();
        std::fs::write(rig.repo.root().join("a.txt"), "1 other\n2\n3\n4\n5\n").unwrap();
        rig.repo.git(&["commit", "-qam", "same line, other text"]);
        std::fs::write(rig.repo.root().join("a.txt"), "1 mine\n2\n3\n4\n5\n").unwrap();
        rig.repo.git(&["commit", "-qam", "diverge"]);
        let e = rig
            .call(OperationKind::StashPop, json!({ "id": 0 }))
            .await
            .unwrap_err();
        assert!(
            !rig.repo.git(&["ls-files", "-u"]).is_empty(),
            "{name}: {e:?}"
        );
        let rec = record(&rig);
        assert!(rec.contains("Conflicted"), "{name}: {rec}");
        assert!(!rig.stashes().is_empty(), "{name}");
        rig.stop().await;
    }
    std::env::remove_var(ENV);
}

/// A pull whose auto-stash pop conflicts: `Conflicted`, the pull stands.
/// Returns the rig and the commit HEAD was on before the pull.
async fn conflicted_pull_rig(backend: Arc<dyn GitBackend>) -> (Rig, String) {
    let mut rig = pull_rig(backend, "1\n2\n3\n4\n5 remote\n").await;
    let before = rig.repo.git(&["rev-parse", "HEAD"]).trim().to_string();
    let e = rig.call(OperationKind::Pull, json!({})).await.unwrap_err();
    assert!(e.message.contains("stash"), "{}", e.message);
    assert!(record(&rig).contains("Conflicted"), "{}", record(&rig));
    assert_ne!(rig.repo.git(&["rev-parse", "HEAD"]).trim(), before);
    (rig, before)
}

fn assert_started_over(name: &str, rig: &Rig, before: &str) {
    assert_eq!(
        rig.repo.git(&["rev-parse", "HEAD"]).trim(),
        before,
        "{name}: HEAD is back at the original commit"
    );
    assert_eq!(rig.file(), "1\n2\n3\n4\n5 local\n", "{name}");
    assert!(rig.stashes().is_empty(), "{name}: stash popped");
    assert!(!record(rig).contains("pending_auto_stash"), "{name}");
    assert!(rig.repo.git(&["ls-files", "-u"]).is_empty(), "{name}");
    assert!(
        !rig.repo
            .git(&["for-each-ref", "refs/gitpoulpi-backup/"])
            .is_empty(),
        "{name}: a backup of the discarded files is kept"
    );
}

/// Stuck states #3 (second half) and #5: *Start over* takes a conflicted
/// auto-stashed pull back to the original commit with the local changes
/// restored exactly; Undo routes to it; both ask first.
#[tokio::test]
async fn start_over_takes_a_conflicted_auto_stashed_pull_back() {
    let _env = ENV_LOCK.lock().await;
    std::env::set_var(ENV, "always");
    for (name, backend) in backends() {
        // Start over, asked for, then confirmed.
        let (mut rig, before) = conflicted_pull_rig(backend.clone()).await;
        let conflicted = rig.file();
        let e = rig
            .call(OperationKind::AutoStashStartOver, json!({}))
            .await
            .unwrap_err();
        assert!(
            matches!(&e.kind, Some(TypedError::NeedsConfirmation(p)) if p.contains("undo") && p.contains("refs/gitpoulpi-backup/")),
            "{name}: {e:?}"
        );
        assert_eq!(rig.file(), conflicted, "{name}: nothing changed yet");
        let dry = rig
            .call(
                OperationKind::AutoStashStartOver,
                json!({ "dry_run": true }),
            )
            .await
            .unwrap();
        assert_eq!(dry["applied"], false, "{name}: {dry}");
        assert_eq!(rig.file(), conflicted, "{name}");
        rig.call(
            OperationKind::AutoStashStartOver,
            json!({ "confirmed": true }),
        )
        .await
        .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        assert_started_over(name, &rig, &before);
        rig.stop().await;

        // Undo means the same while the record is pending.
        let (mut rig, before) = conflicted_pull_rig(backend.clone()).await;
        let e = rig.call(OperationKind::Undo, json!({})).await.unwrap_err();
        assert!(
            matches!(&e.kind, Some(TypedError::NeedsConfirmation(p)) if p.contains("Start over")),
            "{name}: {e:?}"
        );
        rig.call(OperationKind::Undo, json!({ "confirmed": true }))
            .await
            .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        assert_started_over(name, &rig, &before);
        rig.stop().await;
    }
    std::env::remove_var(ENV);
}

/// Start over never loses work: local changes on other paths are refused
/// up front, and a step failing midway leaves the stash and the files.
#[tokio::test]
async fn start_over_stops_with_the_stash_intact_when_a_step_fails() {
    let _env = ENV_LOCK.lock().await;
    std::env::set_var(ENV, "always");
    for (name, backend) in backends() {
        // Other paths with local changes: refused, nothing touched.
        let (mut rig, _) = conflicted_pull_rig(backend.clone()).await;
        let conflicted = rig.file();
        std::fs::write(rig.repo.root().join("b.txt"), "x\n").unwrap();
        rig.repo.git(&["add", "b.txt"]);
        let e = rig
            .call(
                OperationKind::AutoStashStartOver,
                json!({ "confirmed": true }),
            )
            .await
            .unwrap_err();
        assert!(e.message.contains("b.txt"), "{name}: {}", e.message);
        assert_eq!(rig.file(), conflicted, "{name}");
        assert!(!rig.stashes().is_empty(), "{name}");
        assert!(record(&rig).contains("Conflicted"), "{name}");
        // Undo keeps its old refusal when Start over is not possible.
        let e = rig.call(OperationKind::Undo, json!({})).await.unwrap_err();
        assert!(e.message.contains("auto-stash"), "{name}: {}", e.message);
        rig.stop().await;

        // HEAD moved on outside: refused up front, nothing touched.
        let (mut rig, _) = conflicted_pull_rig(backend.clone()).await;
        let conflicted = rig.file();
        let stashes = rig.stashes();
        rig.repo.git(&["update-ref", "refs/heads/main", "HEAD~1"]);
        let e = rig
            .call(
                OperationKind::AutoStashStartOver,
                json!({ "confirmed": true }),
            )
            .await
            .unwrap_err();
        // The move is now an entry of its own (recorded as made outside
        // GitPoulpi), so the newest operation is not the pull's.
        assert!(
            e.message.contains("moved branch main") && e.message.contains("newest operation"),
            "{name}: {}",
            e.message
        );
        assert_eq!(rig.file(), conflicted, "{name}");
        assert_eq!(rig.stashes(), stashes, "{name}");
        rig.stop().await;
    }
    std::env::remove_var(ENV);
}

/// Start over also covers a checkout whose pop failed with nothing applied
/// (`PopFailed`): back on the original branch, the untracked file restored.
#[tokio::test]
async fn start_over_takes_a_checkout_with_a_failed_pop_back() {
    let _env = ENV_LOCK.lock().await;
    std::env::set_var(ENV, "always");
    std::env::set_var(INCLUDE_UNTRACKED, "true");
    for (name, backend) in backends() {
        let mut rig = pop_failed_rig(backend).await;
        assert!(record(&rig).contains("PopFailed"), "{name}");
        assert_eq!(rig.branch(), "other", "{name}");
        rig.call(
            OperationKind::AutoStashStartOver,
            json!({ "confirmed": true }),
        )
        .await
        .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        assert_eq!(rig.branch(), "main", "{name}");
        assert_eq!(
            std::fs::read_to_string(rig.repo.root().join("u.txt")).unwrap(),
            "mine\n",
            "{name}"
        );
        assert!(rig.stashes().is_empty(), "{name}");
        assert!(!record(&rig).contains("pending_auto_stash"), "{name}");
        rig.stop().await;
    }
    std::env::remove_var(INCLUDE_UNTRACKED);
    std::env::remove_var(ENV);
}

/// A staging step stacked on the operation does not stop Start over.
#[tokio::test]
async fn start_over_ignores_staging_done_since() {
    let _env = ENV_LOCK.lock().await;
    std::env::set_var(ENV, "always");
    for (name, backend) in backends() {
        let (mut rig, before) = conflicted_pull_rig(backend).await;
        rig.call(
            OperationKind::StageFile,
            json!({ "target": { "File": "a.txt" } }),
        )
        .await
        .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        rig.call(
            OperationKind::AutoStashStartOver,
            json!({ "confirmed": true }),
        )
        .await
        .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        assert_started_over(name, &rig, &before);
        rig.stop().await;
    }
    std::env::remove_var(ENV);
}

/// The pending record's stash commit, read from `repo-state.toml`.
fn record_commit(rig: &Rig) -> String {
    record(rig)
        .lines()
        .find_map(|l| l.strip_prefix("stash_commit = "))
        .map(|v| v.trim().trim_matches('"').to_string())
        .unwrap_or_else(|| panic!("no stash_commit in {}", record(rig)))
}

/// Stuck state #6: an `Applied` record with a dirty tree (a crash between
/// steps, edits after a stopped operation). Pop would overwrite the edits;
/// stashing them is allowed now, the record follows its stash, and both
/// sets of changes come back.
#[tokio::test]
async fn an_applied_record_with_a_dirty_tree_can_stash_the_edits_then_pop() {
    let _env = ENV_LOCK.lock().await;
    std::env::set_var(ENV, "always");
    for (name, backend) in backends() {
        let mut rig = pending_rig(backend).await;
        let state_file = rig.repo.state_dir().join("repo-state.toml");
        let text = std::fs::read_to_string(&state_file).unwrap();
        std::fs::write(&state_file, text.replace("Conflicted", "Applied")).unwrap();
        let commit = record_commit(&rig);
        std::fs::write(rig.repo.root().join("a.txt"), "1\n2\n3\n4\n5 dirty\n").unwrap();
        // The trap: the pop would overwrite the edits.
        rig.call(OperationKind::StashPop, json!({ "commit": commit }))
            .await
            .unwrap_err();
        rig.repo.git(&["checkout", "-q", "--", "a.txt"]);
        std::fs::write(rig.repo.root().join("a.txt"), "1\n2\n3\n4\n5 dirty\n").unwrap();
        // Put the record back to Applied (the failed pop recorded PopFailed).
        let text = std::fs::read_to_string(&state_file).unwrap();
        std::fs::write(&state_file, text.replace("PopFailed", "Applied")).unwrap();

        rig.call(
            OperationKind::StashPush,
            json!({ "message": "my edits", "include_untracked": false }),
        )
        .await
        .unwrap_or_else(|e| panic!("{name}: stash push refused: {}", e.message));
        assert!(
            record(&rig).contains("stash_id = 1"),
            "{name}: the record follows its stash: {}",
            record(&rig)
        );
        rig.call(OperationKind::StashPop, json!({ "commit": commit }))
            .await
            .unwrap_or_else(|e| panic!("{name}: pop: {}", e.message));
        assert!(!record(&rig).contains("pending_auto_stash"), "{name}");
        assert_eq!(rig.file(), "1 local\n2\n3\n4\n5\n", "{name}");
        // The user's own edits are in the other stash, still there.
        rig.repo.git(&["commit", "-qam", "the auto-stashed change"]);
        rig.call(OperationKind::StashPop, json!({ "id": 0 }))
            .await
            .unwrap_or_else(|e| panic!("{name}: second pop: {}", e.message));
        assert_eq!(rig.file(), "1 local\n2\n3\n4\n5 dirty\n", "{name}");
        rig.stop().await;
    }
    std::env::remove_var(ENV);
}

/// A feature branch whose rebase onto `other` conflicts, with a dirty
/// tree: the auto-stash is made, the rebase stops, the record waits for it
/// (`after_operation`).
async fn conflicted_rebase_rig(backend: Arc<dyn GitBackend>) -> Rig {
    let mut rig = Rig::new(backend, "1 other\n2\n3\n4\n5\n").await;
    rig.repo.git(&["checkout", "-q", "-b", "feature"]);
    std::fs::write(rig.repo.root().join("a.txt"), "1 feature\n2\n3\n4\n5\n").unwrap();
    std::fs::write(rig.repo.root().join("b.txt"), "b\n").unwrap();
    rig.repo.git(&["add", "b.txt"]);
    rig.repo.git(&["commit", "-qam", "feature edit"]);
    std::fs::write(rig.repo.root().join("b.txt"), "b local\n").unwrap();
    let refs = rig.call(OperationKind::Refs, json!({})).await.unwrap();
    let other = refs
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["kind"]["LocalBranch"]["name"] == "other")
        .unwrap()
        .clone();
    let _ = rig
        .call(
            OperationKind::Rebase,
            json!({ "onto": other, "opts": {}, "autostash": "always" }),
        )
        .await;
    assert!(!rig.repo.git(&["ls-files", "-u"]).is_empty());
    assert!(record(&rig).contains("Applied"), "{}", record(&rig));
    assert!(!rig.stashes().is_empty());
    rig
}

fn resolve_rebase_conflict(rig: &Rig) {
    std::fs::write(rig.repo.root().join("a.txt"), "1 resolved\n2\n3\n4\n5\n").unwrap();
    rig.repo.git(&["add", "a.txt"]);
}

/// Stuck state #7: the auto-stashed rebase conflicts; continuing it in the
/// app pops the stash.
#[tokio::test]
async fn continuing_an_auto_stashed_conflicted_rebase_pops_the_stash() {
    let _env = ENV_LOCK.lock().await;
    for (name, backend) in backends() {
        let mut rig = conflicted_rebase_rig(backend).await;
        resolve_rebase_conflict(&rig);
        rig.call(OperationKind::RebaseContinue, json!({}))
            .await
            .unwrap_or_else(|e| panic!("{name}: continue: {}", e.message));
        assert!(rig.stashes().is_empty(), "{name}: stash popped");
        assert!(!record(&rig).contains("pending_auto_stash"), "{name}");
        assert_eq!(rig.b(), "b local\n", "{name}");
        rig.stop().await;
    }
}

/// Aborting it pops the stash too: the changes are back where they were.
#[tokio::test]
async fn aborting_an_auto_stashed_conflicted_rebase_pops_the_stash() {
    let _env = ENV_LOCK.lock().await;
    for (name, backend) in backends() {
        let mut rig = conflicted_rebase_rig(backend).await;
        rig.call(OperationKind::RebaseAbort, json!({}))
            .await
            .unwrap_or_else(|e| panic!("{name}: abort: {}", e.message));
        assert!(rig.stashes().is_empty(), "{name}: stash popped");
        assert!(!record(&rig).contains("pending_auto_stash"), "{name}");
        assert_eq!(rig.b(), "b local\n", "{name}");
        assert_eq!(rig.file(), "1 feature\n2\n3\n4\n5\n", "{name}");
        rig.stop().await;
    }
}

/// The operation ended with plain git: the next refresh read pops.
#[tokio::test]
async fn a_rebase_finished_with_plain_git_pops_on_the_next_read() {
    let _env = ENV_LOCK.lock().await;
    for (name, backend) in backends() {
        let mut rig = conflicted_rebase_rig(backend).await;
        // Still in progress: a read leaves the stash alone.
        rig.call(OperationKind::Status, json!({})).await.unwrap();
        assert!(!rig.stashes().is_empty(), "{name}");
        resolve_rebase_conflict(&rig);
        let out = std::process::Command::new("git")
            .current_dir(rig.repo.root())
            .env("GIT_EDITOR", "true")
            .args(["rebase", "--continue"])
            .output()
            .unwrap();
        assert!(out.status.success(), "{name}: {out:?}");
        assert!(!rig.stashes().is_empty(), "{name}: plain git does not pop");
        rig.call(OperationKind::Status, json!({})).await.unwrap();
        assert!(rig.stashes().is_empty(), "{name}: stash popped by the read");
        assert!(!record(&rig).contains("pending_auto_stash"), "{name}");
        assert_eq!(rig.b(), "b local\n", "{name}");
        rig.stop().await;
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
