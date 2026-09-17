//! `general.undo_depth` honoured by the journal: 1 keeps the
//! latest operation only (the default is 100), 0 is unlimited. Its own test binary because it
//! drives the setting through the process environment.

use std::sync::Arc;

use gitpoulpi_core::domain::{CommitOptions, JournalListing, OperationKind, StageTarget};
use gitpoulpi_core::GitBackend;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::{json, Value};

async fn call(
    client: &mut Client,
    id: u64,
    repo: &gitpoulpi_core::domain::RepoId,
    command: OperationKind,
    args: Value,
) -> Value {
    client
        .call(&Request {
            id,
            session: "undo-depth".into(),
            repo: Some(repo.clone()),
            command,
            args,
        })
        .await
        .unwrap()
        .result
        .unwrap_or_else(|e| panic!("{command:?}: {}", e.message))
}

async fn commits_then_list(depth: &str, count: usize) -> JournalListing {
    // Through the config file of this binary's isolated config dir, not
    // `set_var`: the environment is never mutated once tests are running.
    let dir = std::path::PathBuf::from(std::env::var("GITPOULPI_CONFIG_DIR").unwrap());
    std::fs::write(
        dir.join("config.toml"),
        format!("[general]\nundo_depth = {depth}\n"),
    )
    .unwrap();
    let backend: Arc<dyn GitBackend> = Arc::new(gitpoulpi_git2_backend::Git2Backend::new());
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let handle = Daemon::new("undo-depth").spawn(backend).await.unwrap();
    let mut client = Client::connect_port(handle.port).await.unwrap();
    let id = repo.repo_ref().id;
    let mut n = 1;
    for i in 0..count {
        let name = format!("f{i}.txt");
        std::fs::write(repo.root().join(&name), &name).unwrap();
        n += 1;
        call(
            &mut client,
            n,
            &id,
            OperationKind::StageFile,
            json!({ "target": StageTarget::File(name.into()) }),
        )
        .await;
        n += 1;
        call(
            &mut client,
            n,
            &id,
            OperationKind::Commit,
            json!({ "message": format!("c{i}"), "opts": CommitOptions::default() }),
        )
        .await;
    }
    let listing = call(
        &mut client,
        n + 1,
        &id,
        OperationKind::JournalList,
        json!({}),
    )
    .await;
    handle.shutdown().await.unwrap();
    serde_json::from_value(listing).unwrap()
}

#[tokio::test]
async fn depth_one_keeps_only_the_latest_and_zero_keeps_everything() {
    let one = commits_then_list("1", 3).await;
    assert_eq!(one.undo.len(), 1);
    assert!(one.undo[0].summary.label.starts_with("Commit \"c2\""));

    // Each round stages then commits, and staging is journaled too.
    let unlimited = commits_then_list("0", 3).await;
    assert_eq!(unlimited.undo.len(), 6);
    let commits = unlimited
        .undo
        .iter()
        .filter(|e| e.summary.label.starts_with("Commit"))
        .count();
    assert_eq!(commits, 3);
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
