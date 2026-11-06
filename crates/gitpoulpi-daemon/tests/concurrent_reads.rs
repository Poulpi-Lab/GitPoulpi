//! Reads run concurrently, mutations stay in order.
//!
//! Reads share the per-repo lock and run on their own task, so the TUI's
//! commit-files request is not queued behind the periodic refresh. These
//! tests pin the half that could silently break things: a client that
//! *pipelines* a mutation between two reads must still see the reads on
//! either side of it answered as the repository was at that point.

use std::collections::HashMap;
use std::sync::Arc;

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{FileStatus, OperationKind, StageTarget, WorkingTree};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::transport::{read_message, write_message};
use gitpoulpi_ipc::{Request, Response, ServerMessage};
use gitpoulpi_testkit::FixtureRepoBuilder;

const SESSION: &str = "concurrent-reads-test";

fn request(id: u64, repo: &gitpoulpi_core::domain::RepoRef, command: OperationKind) -> Request {
    let args = match command {
        OperationKind::StageFile => {
            serde_json::json!({ "target": StageTarget::File("new.txt".into()) })
        }
        _ => serde_json::json!({}),
    };
    Request {
        id,
        session: SESSION.into(),
        repo: Some(repo.id.clone()),
        command,
        args,
    }
}

/// Sends every request before reading any answer, then collects the
/// answers by id - in whatever order they come back.
async fn pipelined(
    endpoint: &gitpoulpi_ipc::Endpoint,
    requests: &[Request],
) -> HashMap<u64, Response> {
    let stream = gitpoulpi_ipc::Client::connect(endpoint)
        .await
        .unwrap()
        .into_stream();
    let (mut read_half, mut write_half) = stream.into_split();
    for request in requests {
        write_message(&mut write_half, request).await.unwrap();
    }
    let mut answers = HashMap::new();
    while answers.len() < requests.len() {
        let message: ServerMessage = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            read_message(&mut read_half),
        )
        .await
        .expect("an answer within a minute")
        .unwrap()
        .expect("the daemon closed the connection");
        if let ServerMessage::Response(response) = message {
            answers.insert(response.request_id, response);
        }
    }
    answers
}

fn staged(response: &Response) -> Option<FileStatus> {
    let value = response.result.clone().expect("status succeeded");
    let tree: WorkingTree = serde_json::from_value(value).unwrap();
    tree.files
        .iter()
        .find(|f| f.path.to_string_lossy() == "new.txt")
        .and_then(|f| f.index)
}

#[tokio::test]
async fn a_mutation_pipelined_between_two_reads_is_seen_by_the_second_only() {
    let fixture = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .write_worktree("new.txt", b"hello\n")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new(SESSION).spawn(backend).await.unwrap();

    let answers = pipelined(
        &handle.endpoint(),
        &[
            request(1, &repo, OperationKind::Status),
            request(2, &repo, OperationKind::Log),
            request(3, &repo, OperationKind::StageFile),
            request(4, &repo, OperationKind::Status),
        ],
    )
    .await;

    assert!(answers[&2].result.is_ok(), "the read beside it succeeded");
    assert!(
        answers[&3].result.is_ok(),
        "stage: {:?}",
        answers[&3].result
    );
    assert_eq!(
        staged(&answers[&1]),
        None,
        "the read sent before the stage must not see it"
    );
    assert_eq!(
        staged(&answers[&4]),
        Some(FileStatus::Added),
        "the read sent after the stage must see it"
    );

    handle.shutdown().await.unwrap();
}

#[tokio::test]
async fn many_pipelined_reads_are_each_answered_under_their_own_id() {
    let fixture = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new(SESSION).spawn(backend).await.unwrap();

    let kinds = [
        OperationKind::Status,
        OperationKind::Log,
        OperationKind::Refs,
        OperationKind::Stashes,
        OperationKind::Worktrees,
        OperationKind::Conflicts,
        OperationKind::Remotes,
    ];
    let requests: Vec<Request> = kinds
        .iter()
        .enumerate()
        .map(|(i, kind)| request(i as u64 + 1, &repo, *kind))
        .collect();
    let answers = pipelined(&handle.endpoint(), &requests).await;
    for (i, kind) in kinds.iter().enumerate() {
        let answer = &answers[&(i as u64 + 1)];
        assert!(answer.result.is_ok(), "{kind:?}: {:?}", answer.result);
    }
    // Each answer is the right *shape* for its own request, so no two
    // were crossed on the way back.
    let log: gitpoulpi_core::domain::CommitGraph =
        serde_json::from_value(answers[&2].result.clone().unwrap()).unwrap();
    assert_eq!(log.nodes.len(), 2);
    let _: WorkingTree = serde_json::from_value(answers[&1].result.clone().unwrap()).unwrap();

    handle.shutdown().await.unwrap();
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
