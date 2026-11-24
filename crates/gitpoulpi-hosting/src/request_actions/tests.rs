//! Merge and close, over a recorded transport: nothing opens a socket.

use std::sync::Arc;

use super::{request_actions, MergeMethod};
use crate::client::{client_for, HostingClient};
use crate::http::{Method, RecordedTransport};
use crate::integration::{Integration, IntegrationKind};

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn client(kind: IntegrationKind, base: &str, t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    client_for(
        Integration::new("x", kind, base),
        "tok".into(),
        t,
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap()
}

fn body_of(r: &crate::http::HttpRequest) -> serde_json::Value {
    serde_json::from_str(r.body.as_deref().unwrap()).unwrap()
}

#[test]
fn only_the_three_families_take_merge_and_close_and_gitlab_has_no_rebase() {
    for kind in [
        IntegrationKind::Github,
        IntegrationKind::Gitea,
        IntegrationKind::Forgejo,
    ] {
        let a = request_actions(kind);
        assert!(a.merge && a.rebase && a.close, "{kind:?}");
    }
    let lab = request_actions(IntegrationKind::GitlabSelfManaged);
    assert!(lab.merge && lab.close && !lab.rebase);
    assert!(lab.takes(MergeMethod::Squash) && !lab.takes(MergeMethod::Rebase));
    let none = request_actions(IntegrationKind::JiraCloud);
    assert!(!none.merge && !none.close);
}

#[cfg(feature = "github")]
#[test]
fn github_merges_with_the_chosen_method_and_closes_with_a_state_patch() {
    let t = Arc::new(RecordedTransport::new().on("/pulls/7", 200, "{}"));
    let c = client(IntegrationKind::Github, "https://github.com", t.clone());
    block_on(c.merge_pull_request("acme/app", 7, MergeMethod::Squash)).unwrap();
    block_on(c.close_pull_request("acme/app", 7)).unwrap();
    let sent = t.requests();
    assert_eq!(sent[0].method, Method::Put);
    assert!(sent[0].url.ends_with("/repos/acme/app/pulls/7/merge"));
    assert_eq!(body_of(&sent[0])["merge_method"], "squash");
    assert_eq!(sent[1].method, Method::Patch);
    assert!(sent[1].url.ends_with("/repos/acme/app/pulls/7"));
    assert_eq!(body_of(&sent[1])["state"], "closed");
}

#[cfg(feature = "github")]
#[test]
fn a_host_refusal_comes_back_as_an_error() {
    let t = Arc::new(RecordedTransport::new().on(
        "/pulls/7/merge",
        405,
        r#"{"message":"Pull Request is not mergeable"}"#,
    ));
    let c = client(IntegrationKind::Github, "https://github.com", t);
    let err = block_on(c.merge_pull_request("acme/app", 7, MergeMethod::Merge)).unwrap_err();
    assert!(err.to_string().contains("405") || err.to_string().contains("mergeable"));
}

#[cfg(feature = "gitlab")]
#[test]
fn gitlab_merges_with_squash_flag_and_closes_with_a_state_event() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/merge_requests/58/merge", 200, "{}")
            .on("/merge_requests/58", 200, "{}"),
    );
    let c = client(
        IntegrationKind::GitlabSelfManaged,
        "https://git.example.com",
        t.clone(),
    );
    block_on(c.merge_pull_request("acme/app", 58, MergeMethod::Squash)).unwrap();
    block_on(c.merge_pull_request("acme/app", 58, MergeMethod::Merge)).unwrap();
    block_on(c.close_pull_request("acme/app", 58)).unwrap();
    let sent = t.requests();
    assert_eq!(sent[0].method, Method::Put);
    assert!(sent[0]
        .url
        .ends_with("/projects/acme%2Fapp/merge_requests/58/merge"));
    assert_eq!(body_of(&sent[0])["squash"], true);
    assert_eq!(body_of(&sent[1])["squash"], false);
    assert!(sent[2]
        .url
        .ends_with("/projects/acme%2Fapp/merge_requests/58"));
    assert_eq!(body_of(&sent[2])["state_event"], "close");
    // GitLab's rebase is another call: refused here, nothing sent.
    let err = block_on(c.merge_pull_request("acme/app", 58, MergeMethod::Rebase));
    assert!(err.is_err());
    assert_eq!(t.requests().len(), 3);
}

#[cfg(feature = "gitea")]
#[test]
fn gitea_and_forgejo_merge_with_do_and_close_with_a_state_patch() {
    for kind in [IntegrationKind::Gitea, IntegrationKind::Forgejo] {
        let t = Arc::new(RecordedTransport::new().on("/pulls/12", 200, "{}"));
        let c = client(kind, "https://git.example.com", t.clone());
        block_on(c.merge_pull_request("acme/app", 12, MergeMethod::Rebase)).unwrap();
        block_on(c.close_pull_request("acme/app", 12)).unwrap();
        let sent = t.requests();
        assert_eq!(sent[0].method, Method::Post);
        assert!(sent[0].url.ends_with("/repos/acme/app/pulls/12/merge"));
        assert_eq!(body_of(&sent[0])["Do"], "rebase");
        assert_eq!(sent[1].method, Method::Patch);
        assert_eq!(body_of(&sent[1])["state"], "closed");
    }
}

#[test]
fn a_provider_without_the_calls_says_unsupported() {
    let t = Arc::new(RecordedTransport::new());
    let c = client(
        IntegrationKind::JiraCloud,
        "https://jira.example.com",
        t.clone(),
    );
    assert!(block_on(c.merge_pull_request("P", 1, MergeMethod::Merge)).is_err());
    assert!(block_on(c.close_pull_request("P", 1)).is_err());
    assert!(t.requests().is_empty());
}

#[test]
fn a_draft_marker_is_stripped_from_the_title() {
    assert_eq!(
        super::strip_draft_prefix("Draft: Add login").as_deref(),
        Some("Add login")
    );
    assert_eq!(
        super::strip_draft_prefix("[WIP] Add login").as_deref(),
        Some("Add login")
    );
    assert_eq!(super::strip_draft_prefix("wip:x").as_deref(), Some("x"));
    assert_eq!(super::strip_draft_prefix("Add login"), None);
}

#[cfg(feature = "github")]
#[test]
fn github_marks_ready_through_graphql_with_the_node_id() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/pulls/7", 200, r#"{"node_id":"PR_abc"}"#)
            .on(
                "/graphql",
                200,
                r#"{"data":{"markPullRequestReadyForReview":{}}}"#,
            ),
    );
    let c = client(IntegrationKind::Github, "https://github.com", t.clone());
    block_on(c.mark_pull_request_ready("acme/app", 7)).unwrap();
    let sent = t.requests();
    assert_eq!(sent[0].method, Method::Get);
    assert!(sent[1].url.ends_with("/graphql"));
    let body = body_of(&sent[1]);
    assert!(body["query"]
        .as_str()
        .unwrap()
        .contains("markPullRequestReadyForReview"));
    assert_eq!(body["variables"]["id"], "PR_abc");
}

#[cfg(feature = "gitlab")]
#[test]
fn gitlab_marks_ready_by_putting_the_title_without_its_marker() {
    let t = Arc::new(RecordedTransport::new().on(
        "/merge_requests/58",
        200,
        r#"{"title":"Draft: Round invoice totals"}"#,
    ));
    let c = client(
        IntegrationKind::GitlabSelfManaged,
        "https://git.example.com",
        t.clone(),
    );
    block_on(c.mark_pull_request_ready("acme/app", 58)).unwrap();
    let sent = t.requests();
    assert_eq!(sent[1].method, Method::Put);
    assert_eq!(body_of(&sent[1])["title"], "Round invoice totals");
    // Already ready: nothing is written.
    let t =
        Arc::new(RecordedTransport::new().on("/merge_requests/58", 200, r#"{"title":"Plain"}"#));
    let c = client(
        IntegrationKind::GitlabSelfManaged,
        "https://git.example.com",
        t.clone(),
    );
    block_on(c.mark_pull_request_ready("acme/app", 58)).unwrap();
    assert_eq!(t.requests().len(), 1);
}

#[cfg(feature = "gitea")]
#[test]
fn gitea_marks_ready_by_patching_the_title_without_wip() {
    let t =
        Arc::new(RecordedTransport::new().on("/pulls/12", 200, r#"{"title":"WIP: Fix totals"}"#));
    let c = client(IntegrationKind::Gitea, "https://git.example.com", t.clone());
    block_on(c.mark_pull_request_ready("acme/app", 12)).unwrap();
    let sent = t.requests();
    assert_eq!(sent[1].method, Method::Patch);
    assert_eq!(body_of(&sent[1])["title"], "Fix totals");
}
