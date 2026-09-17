//! `git.default_encoding` and the repository's own `encoding` preference,
//! through the daemon: file content read for display is decoded with the
//! fallback encoding (the repo's choice wins over the config default), and
//! without either it stays binary. The env var is process-global, so one
//! test runs everything in sequence.

use std::sync::Arc;

use gitpoulpi_core::domain::{BlobContent, BlobSource, OperationKind};
use gitpoulpi_core::GitBackend;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::{json, Value};

const ENV: &str = "GITPOULPI_GIT_DEFAULT_ENCODING";

async fn read_latin(
    client: &mut Client,
    id: &gitpoulpi_core::domain::RepoId,
    n: u64,
) -> BlobContent {
    let request = Request {
        id: n,
        session: "encoding-test".into(),
        repo: Some(id.clone()),
        command: OperationKind::ReadBlob,
        args: json!({ "path": "latin.txt", "source": BlobSource::Index }),
    };
    let value: Value = client.call(&request).await.unwrap().result.unwrap();
    serde_json::from_value::<gitpoulpi_core::domain::Blob>(value)
        .unwrap()
        .content
}

#[tokio::test]
async fn the_configured_encoding_decodes_blobs_and_the_repo_preference_wins() {
    std::env::remove_var(ENV);
    let backends: Vec<(&str, Arc<dyn GitBackend>)> = vec![
        (
            "system",
            Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new()),
        ),
        ("git2", Arc::new(gitpoulpi_git2_backend::Git2Backend::new())),
    ];
    for (name, backend) in backends {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit_bytes("latin.txt", b"caf\xe9\n", "latin1")
            .unwrap()
            .build();
        let handle = Daemon::new("encoding-test").spawn(backend).await.unwrap();
        let mut client = Client::connect_port(handle.port).await.unwrap();
        let id = repo.repo_ref().id;
        assert_eq!(
            read_latin(&mut client, &id, 2).await,
            BlobContent::Binary,
            "{name}"
        );

        std::env::set_var(ENV, "windows-1252");
        let content = read_latin(&mut client, &id, 3).await;
        assert_eq!(content.lines(), ["caf\u{e9}"], "{name}: from the config");

        // The repository's own preference wins over the config default.
        let state = repo.state_dir().join("repo-state.toml");
        std::fs::create_dir_all(state.parent().unwrap()).unwrap();
        std::fs::write(&state, "[preferences]\nencoding = \"koi8-r\"\n").unwrap();
        let content = read_latin(&mut client, &id, 4).await;
        assert_eq!(content.lines(), ["caf\u{418}"], "{name}: koi8-r 0xE9 is и");
        std::env::remove_var(ENV);

        // Commit messages too, through the cached Log read: the setting is
        // part of the cache key, so flipping it is seen at once.
        std::fs::remove_file(&state).unwrap();
        repo.commit_raw_message(b"caf\xe9 au lait\n", None);
        let summary = |value: Value| -> String {
            serde_json::from_value::<gitpoulpi_core::domain::CommitGraph>(value)
                .unwrap()
                .nodes[0]
                .summary
                .clone()
        };
        let log = |n: u64| Request {
            id: n,
            session: "encoding-test".into(),
            repo: Some(id.clone()),
            command: OperationKind::Log,
            args: json!({}),
        };
        let plain = summary(client.call(&log(5)).await.unwrap().result.unwrap());
        assert!(plain.contains('\u{fffd}'), "{name}: {plain:?}");
        std::env::set_var(ENV, "windows-1252");
        let decoded = summary(client.call(&log(6)).await.unwrap().result.unwrap());
        assert_eq!(decoded, "caf\u{e9} au lait", "{name}");
        std::env::remove_var(ENV);
        handle.shutdown().await.unwrap();
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
