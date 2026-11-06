//! A rejection a frontend reacts to with more than a status line (the
//! remote diverged) must reach it *typed*, through the daemon, not as an
//! English sentence to pattern-match.

use std::sync::Arc;

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{
    Oid, OperationKind, PushOptions, PushRejectReason, RewordOutcome, RewordRefusal,
};
use gitpoulpi_core::TypedError;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request};
use gitpoulpi_testkit::FixtureRepoBuilder;

#[tokio::test]
async fn a_diverged_push_arrives_as_a_typed_rejection() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .commit("mine")
        .unwrap()
        .build();
    repo.push_from_elsewhere("main", "theirs");

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let daemon = Daemon::new("typed-errors-test");
    let handle = daemon.spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();

    let resp = client
        .call(&Request {
            id: 1,
            session: "typed-errors-test".into(),
            repo: Some(repo.repo_ref().id.clone()),
            command: OperationKind::Push,
            args: serde_json::json!({ "remote": "origin", "opts": PushOptions::default() }),
        })
        .await
        .unwrap();

    let error = resp.result.expect_err("a diverged push must fail");
    match error.kind {
        Some(TypedError::PushRejected(rejection)) => {
            assert_eq!(rejection.remote, "origin");
            assert_eq!(rejection.branch, "main");
            assert_eq!(rejection.reason, PushRejectReason::NonFastForward);
        }
        other => panic!(
            "expected a typed PushRejected, got {other:?} ({})",
            error.message
        ),
    }

    handle.shutdown().await.unwrap();
}

/// A reword goes through the daemon both ways: a success is a
/// `RewordOutcome`, and a refusal arrives typed.
#[tokio::test]
async fn a_reword_runs_through_the_daemon_and_a_refusal_arrives_typed() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let head = repo.git(&["rev-parse", "HEAD"]);
    let first = repo.git(&["rev-parse", "HEAD~1"]);

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let daemon = Daemon::new("typed-errors-reword-test");
    let handle = daemon.spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let request = |id: u64, commit: &str, message: &str| Request {
        id,
        session: "typed-errors-reword-test".into(),
        repo: Some(repo.repo_ref().id.clone()),
        command: OperationKind::Reword,
        args: serde_json::json!({
            "commit": Oid::from_hex(commit).unwrap(),
            "message": message,
        }),
    };

    let refused = client.call(&request(1, &head, "   ")).await.unwrap();
    let error = refused.result.expect_err("an empty message is refused");
    assert_eq!(
        error.kind,
        Some(TypedError::RewordRefused(RewordRefusal::EmptyMessage)),
        "{}",
        error.message
    );

    let done = client
        .call(&request(2, &first, "first, reworded"))
        .await
        .unwrap();
    let outcome: RewordOutcome = serde_json::from_value(done.result.unwrap()).unwrap();
    assert_eq!(outcome.previous_head.to_hex(), head);
    assert_eq!(outcome.replayed, 1);
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s", "HEAD~1"]),
        "first, reworded"
    );

    handle.shutdown().await.unwrap();
}

/// A split refusal arrives typed through the daemon.
#[tokio::test]
async fn a_split_refusal_arrives_typed() {
    use gitpoulpi_core::domain::SplitRefusal;

    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let head = repo.git(&["rev-parse", "HEAD"]);

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let daemon = Daemon::new("typed-errors-split-test");
    let handle = daemon.spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let resp = client
        .call(&Request {
            id: 1,
            session: "typed-errors-split-test".into(),
            repo: Some(repo.repo_ref().id.clone()),
            command: OperationKind::SplitCommit,
            args: serde_json::json!({
                "req": { "commit": Oid::from_hex(&head).unwrap(), "parts": [] },
            }),
        })
        .await
        .unwrap();
    let error = resp.result.expect_err("no parts is refused");
    assert_eq!(
        error.kind,
        Some(TypedError::SplitRefused(SplitRefusal::TooFewParts)),
        "{}",
        error.message
    );

    handle.shutdown().await.unwrap();
}

/// A history rewrite refusal arrives typed through the daemon, and a stale
/// apply (a run the daemon does not hold) too.
#[tokio::test]
async fn a_rewrite_refusal_arrives_typed() {
    use gitpoulpi_core::domain::rewrite::{Recipe, RewriteRefusal, Scope};

    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .build();
    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let daemon = Daemon::new("typed-errors-rewrite-test");
    let handle = daemon.spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let repo_id = repo.repo_ref().id.clone();
    async fn call(
        client: &mut Client,
        repo: &gitpoulpi_core::domain::RepoId,
        id: u64,
        command: OperationKind,
        args: serde_json::Value,
    ) -> gitpoulpi_ipc::ResponseError {
        let request = Request {
            id,
            session: "typed-errors-rewrite-test".into(),
            repo: Some(repo.clone()),
            command,
            args,
        };
        client.call(&request).await.unwrap().result.unwrap_err()
    }
    let no_rule = Recipe::new("nothing", Scope::AllLocal, vec![]);
    let error = call(
        &mut client,
        &repo_id,
        1,
        OperationKind::RewritePreview,
        serde_json::json!({ "recipe": no_rule }),
    )
    .await;
    assert!(
        matches!(
            error.kind,
            Some(TypedError::RewriteRefused(RewriteRefusal::NoRule, _))
        ),
        "{error:?}"
    );
    let error = call(
        &mut client,
        &repo_id,
        2,
        OperationKind::RewriteApply,
        serde_json::json!({ "run": "run-none", "digest": "d" }),
    )
    .await;
    assert!(
        matches!(
            error.kind,
            Some(TypedError::RewriteRefused(RewriteRefusal::Stale, _))
        ),
        "{error:?}"
    );

    handle.shutdown().await.unwrap();
}

/// Fast-forward and worktree-add go through the daemon: a fast-forward
/// answers a `FastForwardOutcome`, a diverged one arrives as the typed
/// `NotFastForward`, and a new worktree comes back as its `Worktree`.
#[tokio::test]
async fn fast_forward_and_worktree_add_run_through_the_daemon() {
    use gitpoulpi_core::domain::{FastForwardOutcome, Worktree, WorktreeAddOptions, WorktreeStart};

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
    let main = Oid::from_hex(&repo.git(&["rev-parse", "main"])).unwrap();
    let behind = Oid::from_hex(&repo.git(&["rev-parse", "behind"])).unwrap();

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let daemon = Daemon::new("typed-errors-ff-test");
    let handle = daemon.spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let request = |id: u64, command: OperationKind, args: serde_json::Value| Request {
        id,
        session: "typed-errors-ff-test".into(),
        repo: Some(repo.repo_ref().id.clone()),
        command,
        args,
    };

    let done = client
        .call(&request(
            1,
            OperationKind::FastForward,
            serde_json::json!({ "branch": "behind", "target": main }),
        ))
        .await
        .unwrap();
    let outcome: FastForwardOutcome = serde_json::from_value(done.result.unwrap()).unwrap();
    assert_eq!(
        outcome,
        FastForwardOutcome::FastForwarded {
            from: behind,
            to: main
        }
    );

    let refused = client
        .call(&request(
            2,
            OperationKind::FastForward,
            serde_json::json!({ "branch": "diverged", "target": main }),
        ))
        .await
        .unwrap();
    let error = refused
        .result
        .expect_err("a diverged branch cannot fast-forward");
    assert_eq!(
        error.kind,
        Some(TypedError::NotFastForward),
        "{}",
        error.message
    );

    let path = repo.root().parent().unwrap().join("wt-daemon");
    let added = client
        .call(&request(
            3,
            OperationKind::WorktreeAdd,
            serde_json::json!({
                "path": path,
                "opts": WorktreeAddOptions {
                    start: WorktreeStart::Commit(main),
                    new_branch: Some("via-daemon".into()),
                },
            }),
        ))
        .await
        .unwrap();
    let worktree: Worktree = serde_json::from_value(added.result.unwrap()).unwrap();
    assert_eq!(worktree.branch.as_deref(), Some("via-daemon"));
    assert!(path.join("initial.txt").exists());

    handle.shutdown().await.unwrap();
}

/// An interactive-rebase refusal (a plan that is not a permutation of the
/// range) arrives through the daemon typed, and `RebaseContinue` on a
/// repo with no rebase in progress arrives typed too.
#[tokio::test]
async fn rebase_refusal_round_trips_typed() {
    use gitpoulpi_core::domain::{InteractiveRebase, RebaseRefusal};

    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base")
        .unwrap()
        .commit("one")
        .unwrap()
        .build();
    let base = Oid::from_hex(&repo.git(&["rev-parse", "HEAD~1"])).unwrap();

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let daemon = Daemon::new("typed-errors-rebase-test");
    let handle = daemon.spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();

    let refused = client
        .call(&Request {
            id: 1,
            session: "typed-errors-rebase-test".into(),
            repo: Some(repo.repo_ref().id.clone()),
            command: OperationKind::RebaseInteractive,
            args: serde_json::json!({
                "request": InteractiveRebase {
                    upstream: Some(base),
                    onto: None,
                    steps: vec![],
                    autosquash: false,
                    cherry_pick: false,
                },
            }),
        })
        .await
        .unwrap();
    let error = refused.result.expect_err("an empty plan must be refused");
    assert_eq!(
        error.kind,
        Some(TypedError::RebaseRefused(RebaseRefusal::PlanMismatch)),
        "{}",
        error.message
    );

    let no_rebase = client
        .call(&Request {
            id: 2,
            session: "typed-errors-rebase-test".into(),
            repo: Some(repo.repo_ref().id.clone()),
            command: OperationKind::RebaseContinue,
            args: serde_json::json!({}),
        })
        .await
        .unwrap();
    let error = no_rebase.result.expect_err("no rebase is in progress");
    assert_eq!(
        error.kind,
        Some(TypedError::RebaseRefused(RebaseRefusal::NoRebaseInProgress)),
        "{}",
        error.message
    );

    handle.shutdown().await.unwrap();
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
