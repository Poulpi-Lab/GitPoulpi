use std::sync::{Arc, Mutex};

use super::*;
use crate::github::GitHubClient;
use crate::http::RecordedTransport;
use crate::scoped_transport::tests::Scopes;
use crate::IntegrationKind;

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(f)
}

fn guarded(transport: Arc<RecordedTransport>, token: &str) -> AssignmentGuard {
    let integration = Integration::new("gh", IntegrationKind::Github, "https://github.com");
    let scopes = Arc::new(Scopes {
        tokens: Mutex::new(vec!["x".into()]),
        repo: "acme/app",
        asks: Mutex::default(),
        blocked: true,
    });
    AssignmentGuard::with_scopes(
        Box::new(GitHubClient::new(integration, token.into(), transport)),
        scopes,
    )
}

fn issue(project: &str, assignees: &[&str]) -> NewIssue {
    NewIssue {
        project: project.into(),
        title: "t".into(),
        assignees: assignees.iter().map(|a| a.to_string()).collect(),
        ..NewIssue::default()
    }
}

const CREATED: &str = r##"{"number":7,"html_url":"https://github.com/acme/app/issues/7"}"##;

#[test]
fn creating_an_issue_with_assignees_in_a_blocked_repository_is_refused_before_any_request() {
    let transport = Arc::new(RecordedTransport::new().on("/issues", 201, CREATED));
    let client = guarded(transport.clone(), "t");
    let err = block_on(client.create_issue(&issue("acme/app", &["someone"]))).unwrap_err();
    assert!(matches!(err, Error::AssignmentBlocked), "{err:?}");
    assert!(transport.requests().is_empty());
}

#[test]
fn an_issue_without_assignees_or_in_another_repository_goes_through() {
    let transport = Arc::new(RecordedTransport::new().on("/issues", 201, CREATED));
    let client = guarded(transport.clone(), "t");
    block_on(client.create_issue(&issue("acme/app", &[]))).unwrap();
    block_on(client.create_issue(&issue("other/lib", &["someone"]))).unwrap();
    assert_eq!(transport.requests().len(), 2);
}

#[test]
fn editing_assignees_is_refused_but_labels_and_clearing_are_not() {
    let transport = Arc::new(RecordedTransport::new().on("/issues/3", 200, "{}"));
    let client = guarded(transport.clone(), "t");
    let assign = IssueEdit {
        assignees: Some(vec!["someone".into()]),
        ..IssueEdit::default()
    };
    let err = block_on(client.update_issue("acme/app", 3, &assign)).unwrap_err();
    assert!(matches!(err, Error::AssignmentBlocked), "{err:?}");
    let pr = PrEdit {
        assignees: Some(vec!["someone".into()]),
        ..PrEdit::default()
    };
    let err = block_on(client.update_pull_request("acme/app", 3, &pr)).unwrap_err();
    assert!(matches!(err, Error::AssignmentBlocked), "{err:?}");
    assert!(transport.requests().is_empty());
}

#[test]
fn an_app_installation_token_has_no_user_and_asks_nothing() {
    let transport = Arc::new(RecordedTransport::new().on("/user", 200, "{}"));
    let client = guarded(transport.clone(), "ghs_installation");
    let err = block_on(client.current_user()).unwrap_err();
    assert!(matches!(err, Error::BotToken), "{err:?}");
    assert!(transport.requests().is_empty());
}

#[test]
fn a_403_on_user_means_a_bot_not_an_error() {
    let transport = Arc::new(RecordedTransport::new().on("/user", 403, "{}"));
    let client = guarded(transport, "opaque-token");
    let err = block_on(client.current_user()).unwrap_err();
    assert!(matches!(err, Error::BotToken), "{err:?}");
}
