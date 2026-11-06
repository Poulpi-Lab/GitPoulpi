//! Proves the daemon's core promise (plan §1.1): a mutation made by one
//! client shows up, live, to a different client connected to the same
//! session -- without polling.

use std::sync::Arc;
use std::time::Duration;

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{
    CheckoutTarget, CommitOptions, ConflictResolution, ConflictSide, MergeOptions, OperationKind,
    RefKind, StageTarget,
};
use gitpoulpi_core::GitBackend;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Event, Request};
use gitpoulpi_testkit::FixtureRepoBuilder;

fn json(value: impl serde::Serialize) -> serde_json::Value {
    serde_json::to_value(value).unwrap()
}

/// A stage changes only the index and the working tree view, so its
/// `RepoChanged` must say so: a client then skips the log, refs and LFS
/// reads (the slow ones).
#[tokio::test]
async fn a_stage_file_broadcast_carries_an_index_only_scope() {
    use gitpoulpi_ipc::ChangeScope;
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let repo_ref = repo.repo_ref();
    std::fs::write(repo.root().join("new.txt"), "hello").unwrap();

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let daemon = Daemon::new("scope-test");
    let handle = daemon.spawn(backend).await.unwrap();
    let mut watcher = Client::connect(&handle.endpoint())
        .await
        .unwrap()
        .into_stream();
    let mut actor = Client::connect(&handle.endpoint()).await.unwrap();
    let resp = actor
        .call(&Request {
            id: 1,
            session: "scope-test".into(),
            repo: Some(repo_ref.id.clone()),
            command: OperationKind::StageFile,
            args: json(serde_json::json!({ "target": StageTarget::File("new.txt".into()) })),
        })
        .await
        .unwrap();
    assert!(resp.result.is_ok(), "stage failed: {:?}", resp.result);

    let scope = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let msg: gitpoulpi_ipc::ServerMessage =
                gitpoulpi_ipc::transport::read_message(&mut watcher)
                    .await
                    .unwrap()
                    .expect("connection closed before event arrived");
            if let gitpoulpi_ipc::ServerMessage::Event(Event::RepoChanged { repo, scope }) = msg {
                if repo == repo_ref.id {
                    return scope;
                }
            }
        }
    })
    .await
    .expect("timed out waiting for RepoChanged");
    assert!(scope.contains(ChangeScope::INDEX));
    assert!(!scope.intersects(
        ChangeScope::LOG | ChangeScope::REFS | ChangeScope::LFS | ChangeScope::STASHES
    ));
    handle.shutdown().await.unwrap();
}

#[tokio::test]
async fn a_mutation_from_one_client_is_pushed_live_to_another() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let repo_ref = repo.repo_ref();
    std::fs::write(repo.root().join("new.txt"), "hello").unwrap();

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let daemon = Daemon::new("live-update-test");
    let handle = daemon.spawn(backend).await.unwrap();

    // Client A subscribes to events by connecting and reading raw
    // ServerMessages off the wire (Client::call only surfaces Responses).
    let mut watcher = Client::connect(&handle.endpoint())
        .await
        .unwrap()
        .into_stream();

    // Client B stages + commits.
    let mut actor = Client::connect(&handle.endpoint()).await.unwrap();
    let stage_req = Request {
        id: 1,
        session: "live-update-test".into(),
        repo: Some(repo_ref.id.clone()),
        command: OperationKind::StageFile,
        args: json(serde_json::json!({ "target": StageTarget::File("new.txt".into()) })),
    };
    let resp = actor.call(&stage_req).await.unwrap();
    assert!(resp.result.is_ok(), "stage failed: {:?}", resp.result);

    let commit_req = Request {
        id: 2,
        session: "live-update-test".into(),
        repo: Some(repo_ref.id.clone()),
        command: OperationKind::Commit,
        args: json(serde_json::json!({
            "message": "add new.txt",
            "opts": CommitOptions::default(),
        })),
    };
    let resp = actor.call(&commit_req).await.unwrap();
    assert!(resp.result.is_ok(), "commit failed: {:?}", resp.result);

    // Client A should see a RepoChanged event pushed without having sent
    // any request of its own.
    let saw_repo_changed = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let msg: gitpoulpi_ipc::ServerMessage =
                gitpoulpi_ipc::transport::read_message(&mut watcher)
                    .await
                    .unwrap()
                    .expect("connection closed before event arrived");
            if let gitpoulpi_ipc::ServerMessage::Event(Event::RepoChanged { repo, .. }) = msg {
                if repo == repo_ref.id {
                    return;
                }
            }
        }
    })
    .await;
    assert!(saw_repo_changed.is_ok(), "timed out waiting for live event");

    handle.shutdown().await.unwrap();
}

/// `Event::ConflictDetected` must be broadcast when a mutation stops on
/// unmerged files, so a frontend can tell that from "the merge landed"
/// without polling `conflicts`. Deleting the probe in `server.rs` makes
/// this time out.
#[tokio::test]
async fn a_merge_that_stops_on_conflicts_pushes_conflict_detected() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let repo_ref = repo.repo_ref();

    // Two branches editing the same line. Built through the backend
    // directly -- this test is about the *event*, and routing the setup
    // through the daemon would only make the failure harder to read.
    let setup = SystemGitBackend::new();
    let base = setup.log(&repo_ref, &Default::default()).unwrap().nodes[0].id;
    setup.branch_create(&repo_ref, "feature", base).unwrap();
    setup
        .checkout(&repo_ref, &CheckoutTarget::Branch("feature".into()))
        .unwrap();
    std::fs::write(repo.root().join("shared.txt"), "theirs\n").unwrap();
    setup
        .stage(&repo_ref, &StageTarget::File("shared.txt".into()))
        .unwrap();
    setup
        .commit(&repo_ref, "theirs", &CommitOptions::default())
        .unwrap();
    setup
        .checkout(&repo_ref, &CheckoutTarget::Branch("main".into()))
        .unwrap();
    std::fs::write(repo.root().join("shared.txt"), "ours\n").unwrap();
    setup
        .stage(&repo_ref, &StageTarget::File("shared.txt".into()))
        .unwrap();
    setup
        .commit(&repo_ref, "ours", &CommitOptions::default())
        .unwrap();
    let feature = setup
        .refs(&repo_ref)
        .unwrap()
        .into_iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "feature"))
        .expect("feature branch");

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let daemon = Daemon::new("conflict-event-test");
    let handle = daemon.spawn(backend).await.unwrap();

    let mut watcher = Client::connect(&handle.endpoint())
        .await
        .unwrap()
        .into_stream();
    let mut actor = Client::connect(&handle.endpoint()).await.unwrap();

    let merge_req = Request {
        id: 1,
        session: "conflict-event-test".into(),
        repo: Some(repo_ref.id.clone()),
        command: OperationKind::Merge,
        args: json(serde_json::json!({
            "from": feature,
            "opts": MergeOptions::default(),
        })),
    };
    let resp = actor.call(&merge_req).await.unwrap();
    assert!(resp.result.is_ok(), "merge failed: {:?}", resp.result);

    let paths = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let msg: gitpoulpi_ipc::ServerMessage =
                gitpoulpi_ipc::transport::read_message(&mut watcher)
                    .await
                    .unwrap()
                    .expect("connection closed before event arrived");
            if let gitpoulpi_ipc::ServerMessage::Event(Event::ConflictDetected { repo, paths }) =
                msg
            {
                if repo == repo_ref.id {
                    return paths;
                }
            }
        }
    })
    .await
    .expect("timed out waiting for ConflictDetected");
    assert_eq!(paths, vec!["shared.txt".to_string()]);

    // Resolving the last unmerged path must NOT announce a conflict that
    // is no longer there -- the probe reports what the repo holds after
    // the operation, not that the operation was conflict-shaped.
    let resolve_req = Request {
        id: 2,
        session: "conflict-event-test".into(),
        repo: Some(repo_ref.id.clone()),
        command: OperationKind::ResolveConflict,
        args: json(serde_json::json!({
            "path": "shared.txt",
            "resolution": ConflictResolution::TakeSide(ConflictSide::Ours),
        })),
    };
    let resp = actor.call(&resolve_req).await.unwrap();
    assert!(resp.result.is_ok(), "resolve failed: {:?}", resp.result);

    let stray = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let msg: gitpoulpi_ipc::ServerMessage =
                gitpoulpi_ipc::transport::read_message(&mut watcher)
                    .await
                    .unwrap()
                    .expect("connection closed");
            if let gitpoulpi_ipc::ServerMessage::Event(Event::ConflictDetected { .. }) = msg {
                return;
            }
        }
    })
    .await;
    assert!(
        stray.is_err(),
        "a repo with nothing unmerged must not announce conflicts"
    );

    handle.shutdown().await.unwrap();
}

/// A Git LFS push run through the daemon streams `Event::LfsProgress` to a
/// subscriber - ending with a `done` report - and then `RepoChanged`.
/// Skipped where `git-lfs` is not installed.
#[tokio::test]
async fn an_lfs_push_streams_progress_then_repo_changed() {
    gitpoulpi_testkit::require_git_lfs!("an_lfs_push_streams_progress_then_repo_changed");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_lfs_origin()
        .unwrap()
        .with_lfs()
        .unwrap()
        .lfs_track("*.bin")
        .unwrap()
        .commit_lfs_file("big.bin", b"payload for the daemon test", "add big")
        .unwrap()
        .build();
    let repo_ref = repo.repo_ref();

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let daemon = Daemon::new("lfs-progress-event-test");
    let handle = daemon.spawn(backend).await.unwrap();
    let mut watcher = Client::connect(&handle.endpoint())
        .await
        .unwrap()
        .into_stream();
    let mut actor = Client::connect(&handle.endpoint()).await.unwrap();

    let push = Request {
        id: 1,
        session: "lfs-progress-event-test".into(),
        repo: Some(repo_ref.id.clone()),
        command: OperationKind::LfsPush,
        args: json(serde_json::json!({ "remote": "origin" })),
    };
    let resp = actor.call(&push).await.unwrap();
    assert!(resp.result.is_ok(), "lfs push failed: {:?}", resp.result);

    let mut progress = Vec::new();
    let saw_repo_changed = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let msg: gitpoulpi_ipc::ServerMessage =
                gitpoulpi_ipc::transport::read_message(&mut watcher)
                    .await
                    .unwrap()
                    .expect("connection closed before the events arrived");
            match msg {
                gitpoulpi_ipc::ServerMessage::Event(Event::LfsProgress {
                    repo,
                    operation,
                    progress: p,
                }) if repo == repo_ref.id => {
                    assert_eq!(operation, OperationKind::LfsPush);
                    progress.push(p);
                }
                gitpoulpi_ipc::ServerMessage::Event(Event::RepoChanged { repo, .. })
                    if repo == repo_ref.id =>
                {
                    return;
                }
                _ => {}
            }
        }
    })
    .await;
    assert!(
        saw_repo_changed.is_ok(),
        "timed out waiting for RepoChanged"
    );
    assert!(
        progress.last().is_some_and(|p| p.done),
        "LfsProgress must arrive, ending done, before RepoChanged: {progress:?}"
    );
    assert_eq!(repo.remote_lfs_objects().len(), 1);

    handle.shutdown().await.unwrap();
}

/// `refs hide` from one client reaches another as `RefVisibilityChanged`,
/// and the saved filter is what the TUI will read.
#[tokio::test]
async fn a_ref_filter_change_from_one_client_is_broadcast_to_another() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .branch("feature/x")
        .unwrap()
        .build();
    let repo_ref = repo.repo_ref();

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let daemon = Daemon::new("ref-visibility-test");
    let handle = daemon.spawn(backend).await.unwrap();
    let mut watcher = Client::connect(&handle.endpoint())
        .await
        .unwrap()
        .into_stream();
    let mut actor = Client::connect(&handle.endpoint()).await.unwrap();
    let resp = actor
        .call(&Request {
            id: 1,
            session: "ref-visibility-test".into(),
            repo: Some(repo_ref.id.clone()),
            command: OperationKind::SetRefVisibility,
            args: json(serde_json::json!({ "mode": "hide", "names": ["feature/x"] })),
        })
        .await
        .unwrap();
    let answer = resp.result.expect("set ref visibility failed");
    assert_eq!(answer["hidden"], serde_json::json!(["feature/x"]));

    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let msg: gitpoulpi_ipc::ServerMessage =
                gitpoulpi_ipc::transport::read_message(&mut watcher)
                    .await
                    .unwrap()
                    .expect("connection closed before event arrived");
            if let gitpoulpi_ipc::ServerMessage::Event(Event::RefVisibilityChanged { repo }) = msg {
                if repo == repo_ref.id {
                    return;
                }
            }
        }
    })
    .await
    .expect("timed out waiting for RefVisibilityChanged");

    let saved = std::fs::read_to_string(repo.state_dir().join("repo-state.toml"))
        .expect("the filter is persisted in repo-state.toml");
    assert!(saved.contains("feature/x"), "{saved}");
    handle.shutdown().await.unwrap();
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
