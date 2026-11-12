//! A failed operation says which operation it was: the answer's `context`
//! carries the label, the main arguments, the repository, the frontend the
//! connection named in its `Hello` and the request's id. Nothing secret is
//! in it, and an older client that ignores the field loses nothing.

use std::sync::Arc;

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request, ResponseError};
use gitpoulpi_testkit::FixtureRepoBuilder;

async fn failing_call(
    session: &str,
    repo: &gitpoulpi_testkit::FixtureRepo,
    id: u64,
    command: OperationKind,
    args: serde_json::Value,
) -> ResponseError {
    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let daemon = Daemon::new(session);
    let handle = daemon.spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let response = client
        .call(&Request {
            id,
            session: session.into(),
            repo: Some(repo.repo_ref().id.clone()),
            command,
            args,
        })
        .await
        .unwrap();
    handle.shutdown().await.unwrap();
    response.result.expect_err("the operation must fail")
}

#[tokio::test]
async fn removing_a_remote_that_does_not_exist_names_the_operation_and_the_remote() {
    gitpoulpi_ipc::auth::set_client_name("CLI");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let error = failing_call(
        "error-context-remote",
        &repo,
        41,
        OperationKind::RemoteRemove,
        serde_json::json!({ "name": "cm6" }),
    )
    .await;
    let context = error.context.expect("the failure carries its context");
    assert_eq!(context.label, "Remove remote");
    assert_eq!(context.operation, "RemoteRemove");
    assert_eq!(context.args, "remote=cm6");
    assert_eq!(context.request, 41);
    assert_eq!(context.frontend.as_deref(), Some("CLI"));
    assert_eq!(
        context.repo.as_deref(),
        Some(repo.repo_ref().id.0.as_str()),
        "the repository the request named"
    );
    assert!(
        error.message.contains("cm6"),
        "the message is untouched: {}",
        error.message
    );
}

#[tokio::test]
async fn a_network_operation_names_its_job_and_hides_credentials_in_urls() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let url = "https://me:hunter2@127.0.0.1:1/r.git";
    let error = failing_call(
        "error-context-fetch",
        &repo,
        7,
        OperationKind::Fetch,
        serde_json::json!({ "remote": url }),
    )
    .await;
    let context = error.context.expect("the failure carries its context");
    assert_eq!(context.label, "Fetch");
    assert!(context.job.is_some(), "a fetch is a job: {context:?}");
    let shown = serde_json::to_string(&context).unwrap();
    assert!(!shown.contains("hunter2"), "{shown}");
}

#[tokio::test]
async fn the_context_is_an_added_field_older_clients_ignore() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let error = failing_call(
        "error-context-wire",
        &repo,
        3,
        OperationKind::RemoteRemove,
        serde_json::json!({ "name": "nope" }),
    )
    .await;
    // The wire form still has every field it had; `context` is one more.
    let value = serde_json::to_value(&error).unwrap();
    for field in ["message", "unsupported"] {
        assert!(value.get(field).is_some(), "{field} in {value}");
    }
    assert_eq!(value["context"]["label"], "Remove remote");
    // And a reply without it (an older daemon) still reads.
    let mut older = value.clone();
    older.as_object_mut().unwrap().remove("context");
    let parsed: ResponseError = serde_json::from_value(older).unwrap();
    assert!(parsed.context.is_none());
}
