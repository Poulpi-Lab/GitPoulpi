//! `ConflictShow` decodes the stages with `git.default_encoding` like the
//! other text reads do (and stays undecodable without it). The env var is
//! process-global, so everything runs in one test.

use std::sync::Arc;

use gitpoulpi_core::domain::{BlobContent, MergeDocument, OperationKind};
use gitpoulpi_core::GitBackend;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request};
use gitpoulpi_testkit::ConflictFixture;
use serde_json::json;

const ENV: &str = "GITPOULPI_GIT_DEFAULT_ENCODING";

async fn show(client: &mut Client, id: &gitpoulpi_core::domain::RepoId, n: u64) -> MergeDocument {
    let request = Request {
        id: n,
        session: "conflict-encoding-test".into(),
        repo: Some(id.clone()),
        command: OperationKind::ConflictShow,
        args: json!({ "path": "latin1.txt" }),
    };
    let value = client.call(&request).await.unwrap().result.unwrap();
    serde_json::from_value(value).unwrap()
}

#[tokio::test]
async fn the_configured_encoding_decodes_the_conflict_document() {
    std::env::remove_var(ENV);
    let backends: Vec<(&str, Arc<dyn GitBackend>)> = vec![
        (
            "system",
            Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new()),
        ),
        ("git2", Arc::new(gitpoulpi_git2_backend::Git2Backend::new())),
    ];
    for (name, backend) in backends {
        let fx = ConflictFixture::latin1();
        let handle = Daemon::new("conflict-encoding-test")
            .spawn(backend)
            .await
            .unwrap();
        let mut client = Client::connect(&handle.endpoint()).await.unwrap();
        let id = fx.repo_ref().id;
        let plain = show(&mut client, &id, 2).await;
        assert_eq!(plain.conflict_count(), 0, "{name}: no text to merge");
        assert!(
            matches!(plain.ours.unwrap().content, BlobContent::Binary),
            "{name}"
        );

        std::env::set_var(ENV, "windows-1252");
        let doc = show(&mut client, &id, 3).await;
        assert_eq!(doc.conflict_count(), 1, "{name}");
        assert_eq!(doc.encoding.as_deref(), Some("windows-1252"), "{name}");
        assert_eq!(doc.ours.unwrap().lines()[0], "caf\u{e9}", "{name}");
        std::env::remove_var(ENV);
        handle.shutdown().await.unwrap();
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
