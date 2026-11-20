//! Two daemons (two sessions) on one repository share one journal file, as
//! an embedded CLI daemon and a TUI daemon do: the stamp check keeps each
//! one reading what the other wrote, so neither corrupts the file (the
//! installations design, 1.5: two sessions of the same environment).

use std::collections::HashSet;
use std::sync::Arc;

use gitpoulpi_core::domain::{CommitOptions, OperationKind, StageTarget};
use gitpoulpi_core::journal::Journal;
use gitpoulpi_core::GitBackend;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::json;

#[tokio::test]
async fn two_daemons_on_one_repo_never_corrupt_each_others_journal() {
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    std::env::set_var("GITPOULPI_GENERAL_UNDO_DEPTH", "0");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let backend: Arc<dyn GitBackend> = Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new());
    let mut rigs = Vec::new();
    for session in ["first", "second"] {
        let handle = Daemon::new(session)
            .spawn(Arc::clone(&backend))
            .await
            .unwrap();
        let client = Client::connect(&handle.endpoint()).await.unwrap();
        rigs.push((session, handle, client));
    }
    let id = repo.repo_ref().id;
    let mut next = 1u64;

    // Alternating operations, one daemon after the other.
    for round in 0..4 {
        for (session, _, client) in rigs.iter_mut() {
            let name = format!("{session}-{round}.txt");
            std::fs::write(repo.root().join(&name), &name).unwrap();
            for (command, args) in [
                (
                    OperationKind::StageFile,
                    json!({ "target": StageTarget::File(name.clone().into()) }),
                ),
                (
                    OperationKind::Commit,
                    json!({ "message": name, "opts": CommitOptions::default() }),
                ),
            ] {
                next += 1;
                client
                    .call(&Request {
                        id: next,
                        session: (*session).into(),
                        repo: Some(id.clone()),
                        command,
                        args,
                    })
                    .await
                    .unwrap()
                    .result
                    .unwrap_or_else(|e| panic!("{command:?} failed: {}", e.message));
            }
        }
    }

    for (_, handle, _) in rigs {
        handle.shutdown().await.unwrap();
    }
    let text = std::fs::read_to_string(repo.root().join(".git/gitpoulpi/journal.json")).unwrap();
    let journal: Journal = serde_json::from_str(&text).expect("the journal is intact");
    let ids: Vec<u64> = journal.undo.iter().map(|e| e.id).collect();
    let unique: HashSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), ids.len(), "entry ids never repeat: {ids:?}");
    assert_eq!(ids.len(), 16, "both daemons' 8 operations each: {ids:?}");
}
