//! Perf regression guard for the outside-change snapshot
//! (docs/developer/perf-budgets.md): on the system backend a ref-moving
//! mutation adds one snapshot read after it (its own moves are adopted, not
//! recorded), and a poll of a quiet repository adds nothing (the
//! fingerprint of the ref storage is unchanged). On the libgit2 backend none
//! of it spawns a process. Its own binary: the setting is read from the
//! process environment and the spawn counter is process wide.

use std::sync::Arc;

use gitpoulpi_cli_backend::{total_spawn_count, SystemGitBackend};
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request};
use gitpoulpi_testkit::FixtureRepoBuilder;

async fn call(
    client: &mut Client,
    id: u64,
    repo: &gitpoulpi_core::domain::RepoId,
    command: OperationKind,
    args: serde_json::Value,
) -> u64 {
    let before = total_spawn_count();
    let resp = client
        .call(&Request {
            id,
            session: "spawn-budget-outside".into(),
            repo: Some(repo.clone()),
            command,
            args,
        })
        .await
        .unwrap();
    assert!(resp.result.is_ok(), "{command:?} failed: {:?}", resp.result);
    total_spawn_count() - before
}

#[tokio::test]
async fn outside_detection_costs_a_known_number_of_spawns() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let repo_ref = repo.repo_ref();
    let tip = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new("spawn-budget-outside")
        .spawn(backend)
        .await
        .unwrap();
    let mut client = Client::connect_port(handle.port).await.unwrap();
    let id = &repo_ref.id;
    let none = serde_json::json!({});

    // Outside recording off: what a branch creation and a state read cost.
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    call(&mut client, 1, id, OperationKind::Status, none.clone()).await;
    let create_off = call(
        &mut client,
        2,
        id,
        OperationKind::BranchCreate,
        serde_json::json!({ "name": "off", "at": tip }),
    )
    .await;
    let state_off = call(
        &mut client,
        3,
        id,
        OperationKind::JournalState,
        none.clone(),
    )
    .await;

    // On: the first look is the baseline, then the same two requests.
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "true");
    call(
        &mut client,
        4,
        id,
        OperationKind::JournalState,
        none.clone(),
    )
    .await;
    let create_on = call(
        &mut client,
        5,
        id,
        OperationKind::BranchCreate,
        serde_json::json!({ "name": "on", "at": tip }),
    )
    .await;
    // The file system's clock is coarse: the fingerprint is trusted only
    // once the ref storage has been still for a few seconds.
    tokio::time::sleep(std::time::Duration::from_millis(3300)).await;
    call(
        &mut client,
        6,
        id,
        OperationKind::JournalState,
        none.clone(),
    )
    .await;
    let state_on = call(&mut client, 7, id, OperationKind::JournalState, none).await;
    println!(
        "SPAWNS outside: create off={create_off} on={create_on}, quiet state off={state_off} on={state_on}"
    );
    assert_eq!(
        create_on - create_off,
        2,
        "a ref-moving mutation: {create_on} spawns, {create_off} without outside detection, budget +2"
    );
    assert!(
        state_on <= state_off,
        "a quiet poll: {state_on} spawns, {state_off} without outside detection, budget +0"
    );
    handle.shutdown().await.unwrap();
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
