//! `general.journal_outside = false`: what happens outside GitPoulpi is not
//! recorded, and the old behaviour stays (the newest entry is blocked). Its
//! own test binary: the setting is read from the process environment.

use std::sync::Arc;

use gitpoulpi_core::domain::{JournalListing, JournalState, OperationKind, UndoRefusal};
use gitpoulpi_core::{GitBackend, TypedError};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::{json, Value};

#[tokio::test]
async fn with_the_setting_off_an_outside_commit_is_not_recorded_and_still_blocks() {
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    let backend: Arc<dyn GitBackend> = Arc::new(gitpoulpi_git2_backend::Git2Backend::new());
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let handle = Daemon::new("outside-off").spawn(backend).await.unwrap();
    let mut client = Client::connect_port(handle.port).await.unwrap();
    let id = repo.repo_ref().id;
    let mut n = 0;
    let mut call = |command: OperationKind, args: Value| {
        n += 1;
        Request {
            id: n,
            session: "outside-off".into(),
            repo: Some(id.clone()),
            command,
            args,
        }
    };

    let head = repo.git(&["rev-parse", "HEAD"]);
    let r = client
        .call(&call(
            OperationKind::BranchCreate,
            json!({ "name": "mine", "at": head.trim() }),
        ))
        .await
        .unwrap();
    assert!(r.result.is_ok());
    repo.git(&["commit", "--allow-empty", "-m", "outside"]);
    repo.git(&["branch", "-m", "mine", "mine2"]);

    let state: JournalState = serde_json::from_value(
        client
            .call(&call(OperationKind::JournalState, json!({})))
            .await
            .unwrap()
            .result
            .unwrap(),
    )
    .unwrap();
    assert_eq!(state.undo_count, 1, "only our own entry");
    assert!(matches!(state.blocked, Some(UndoRefusal::RefMoved { .. })));
    let list: JournalListing = serde_json::from_value(
        client
            .call(&call(OperationKind::JournalList, json!({})))
            .await
            .unwrap()
            .result
            .unwrap(),
    )
    .unwrap();
    assert!(list
        .undo
        .iter()
        .all(|e| e.summary.kind != OperationKind::Outside));
    let e = client
        .call(&call(OperationKind::Undo, json!({})))
        .await
        .unwrap()
        .result
        .unwrap_err();
    assert!(matches!(
        e.kind,
        Some(TypedError::UndoRefused(UndoRefusal::RefMoved { .. }))
    ));
    handle.shutdown().await.unwrap();
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
