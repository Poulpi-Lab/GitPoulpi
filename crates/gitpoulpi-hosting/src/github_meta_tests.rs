use crate::client::{client_for, HostingClient, NewIssue, NewPullRequest};
use crate::http::{HttpRequest, Method, RecordedTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::metadata::{IssueEdit, MetaKind, PrEdit};
use std::sync::Arc;

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn client(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    client_for(
        Integration::new("gh", IntegrationKind::Github, "https://github.com"),
        "tok".into(),
        t,
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap()
}

fn enterprise(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    client_for(
        Integration::new("ghe", IntegrationKind::GithubEnterprise, "https://ghe.corp"),
        "tok".into(),
        t,
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap()
}

fn body_of(r: &HttpRequest) -> serde_json::Value {
    serde_json::from_str(r.body.as_deref().unwrap()).unwrap()
}

#[test]
fn members_labels_and_milestones_are_listed_and_filtered() {
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "/repos/me/p/assignees",
                200,
                r#"[{"login":"octocat","id":1,"html_url":"https://github.com/octocat"},
                    {"login":"hubot","id":2}]"#,
            )
            .on(
                "/repos/me/p/labels",
                200,
                r#"[{"name":"bug","color":"d73a4a","description":"Broken"},{"name":"ui"}]"#,
            )
            .on(
                "/repos/me/p/milestones",
                200,
                r#"[{"number":3,"title":"v1","due_on":"2026-11-01T00:00:00Z","state":"open"}]"#,
            ),
    );
    let c = client(t.clone());
    let all = block_on(c.project_members("me/p", "")).unwrap();
    assert_eq!(all.len(), 2);
    let some = block_on(c.project_members("me/p", "HUB")).unwrap();
    assert_eq!(some.len(), 1);
    assert_eq!(some[0].login, "hubot");
    assert_eq!(some[0].id, "2");

    let labels = block_on(c.project_labels("me/p")).unwrap();
    assert_eq!(labels[0].color, "d73a4a");
    assert_eq!(labels[1].color, "");

    let milestones = block_on(c.project_milestones("me/p")).unwrap();
    assert_eq!(milestones[0].id, 3);
    assert_eq!(milestones[0].due_on.as_deref(), Some("2026-11-01"));
    assert!(t.requests()[0]
        .url
        .starts_with("https://api.github.com/repos/me/p/assignees?"));
}

#[test]
fn enterprise_uses_its_own_api_root() {
    let t = Arc::new(RecordedTransport::new().on("/labels", 200, "[]"));
    block_on(enterprise(t.clone()).project_labels("me/p")).unwrap();
    assert!(t.requests()[0]
        .url
        .starts_with("https://ghe.corp/api/v3/repos/me/p/labels?"));
}

#[test]
fn item_meta_reads_a_pull_request_with_its_reviewers() {
    let t = Arc::new(RecordedTransport::new().on(
        "/repos/me/p/pulls/7",
        200,
        r#"{"assignees":[{"login":"a","id":1}],"requested_reviewers":[{"login":"r","id":2}],
            "labels":[{"name":"bug"}],
            "milestone":{"number":3,"title":"v1","state":"open","due_on":null}}"#,
    ));
    let meta = block_on(client(t).item_meta("me/p", 7, MetaKind::PullRequest)).unwrap();
    assert_eq!(meta.assignees, vec!["a"]);
    assert_eq!(meta.reviewers, vec!["r"]);
    assert_eq!(meta.labels, vec!["bug"]);
    assert_eq!(meta.milestone.unwrap().id, 3);
}

#[test]
fn editing_an_issue_patches_the_three_fields_and_null_clears_the_milestone() {
    let t = Arc::new(RecordedTransport::new().on("/issues/4", 200, "{}"));
    let edit = IssueEdit {
        assignees: Some(vec!["a".into()]),
        labels: Some(vec![]),
        milestone: Some(None),
    };
    block_on(client(t.clone()).update_issue("me/p", 4, &edit)).unwrap();
    let sent = &t.requests()[0];
    assert_eq!(sent.method, Method::Patch);
    assert!(sent.url.ends_with("/repos/me/p/issues/4"));
    let body = body_of(sent);
    assert_eq!(body["assignees"], serde_json::json!(["a"]));
    assert_eq!(body["labels"], serde_json::json!([]));
    assert!(body.get("milestone").is_some() && body["milestone"].is_null());
}

#[test]
fn an_empty_edit_sends_nothing() {
    let t = Arc::new(RecordedTransport::new());
    block_on(client(t.clone()).update_issue("me/p", 4, &IssueEdit::default())).unwrap();
    assert!(t.requests().is_empty());
}

#[test]
fn reviewers_are_added_and_removed_through_requested_reviewers() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/requested_reviewers", 201, "{}")
            .on(
                "/pulls/7",
                200,
                r#"{"requested_reviewers":[{"login":"a","id":1},{"login":"b","id":2}]}"#,
            ),
    );
    let edit = PrEdit {
        reviewers: Some(vec!["b".into(), "c".into()]),
        ..PrEdit::default()
    };
    block_on(client(t.clone()).update_pull_request("me/p", 7, &edit)).unwrap();
    let seen = t.requests();
    let post = seen
        .iter()
        .find(|r| r.method == Method::Post)
        .expect("an add");
    assert!(post.url.ends_with("/pulls/7/requested_reviewers"));
    assert_eq!(body_of(post)["reviewers"], serde_json::json!(["c"]));
    let delete = seen
        .iter()
        .find(|r| r.method == Method::Delete)
        .expect("a removal");
    assert!(delete.url.ends_with("/pulls/7/requested_reviewers"));
    assert_eq!(body_of(delete)["reviewers"], serde_json::json!(["a"]));
    assert!(
        seen.iter().all(|r| r.method != Method::Patch),
        "no issue PATCH when only reviewers change"
    );
}

#[test]
fn a_new_issue_carries_assignees_and_a_milestone() {
    let t = Arc::new(RecordedTransport::new().on(
        "/issues",
        201,
        r#"{"number":9,"html_url":"https://github.com/me/p/issues/9"}"#,
    ));
    block_on(client(t.clone()).create_issue(&NewIssue {
        project: "me/p".into(),
        title: "t".into(),
        assignees: vec!["a".into()],
        milestone: Some(3),
        ..NewIssue::default()
    }))
    .unwrap();
    let body = body_of(&t.requests()[0]);
    assert_eq!(body["assignees"], serde_json::json!(["a"]));
    assert_eq!(body["milestone"], serde_json::json!(3));
}

fn new_pr() -> NewPullRequest {
    NewPullRequest {
        project: "me/p".into(),
        source_branch: "f".into(),
        target_branch: "main".into(),
        title: "t".into(),
        assignees: vec!["a".into()],
        reviewers: vec!["r".into()],
        labels: vec!["bug".into()],
        milestone: Some(3),
        ..NewPullRequest::default()
    }
}

#[test]
fn a_new_pull_request_gets_its_metadata_in_a_second_round() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/requested_reviewers", 201, "{}")
            .on("/issues/12", 200, "{}")
            .on(
                "/pulls",
                201,
                r#"{"number":12,"html_url":"https://github.com/me/p/pull/12"}"#,
            ),
    );
    let made = block_on(client(t.clone()).create_pull_request(&new_pr())).unwrap();
    assert_eq!(made.number, 12);
    assert!(made.warning.is_none());
    let seen = t.requests();
    let patch = seen.iter().find(|r| r.method == Method::Patch).unwrap();
    let body = body_of(patch);
    assert_eq!(body["labels"], serde_json::json!(["bug"]));
    assert_eq!(body["milestone"], serde_json::json!(3));
    let review = seen
        .iter()
        .find(|r| r.url.ends_with("/pulls/12/requested_reviewers"))
        .unwrap();
    assert_eq!(body_of(review)["reviewers"], serde_json::json!(["r"]));
}

#[test]
fn a_metadata_failure_after_the_pull_request_exists_is_a_warning() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/issues/12", 422, r#"{"message":"Validation Failed"}"#)
            .on(
                "/pulls",
                201,
                r#"{"number":12,"html_url":"https://github.com/me/p/pull/12"}"#,
            ),
    );
    let made = block_on(client(t).create_pull_request(&new_pr())).unwrap();
    assert_eq!(made.number, 12);
    assert!(made.warning.unwrap().contains("metadata"));
}

#[test]
fn moving_a_card_swaps_the_status_label_then_closes_through_the_issue_state() {
    use crate::board::{columns, BoardSource, BoardSpec};
    use crate::model::IssueSummary;
    let t = Arc::new(RecordedTransport::new().on("/issues/7", 200, "{}"));
    let c = client(t.clone());
    assert_eq!(c.board_source(), BoardSource::Labels);
    let spec = BoardSpec {
        source: BoardSource::Labels,
        declared: Vec::new(),
        labels: vec!["status::todo".into(), "status::doing".into()],
    };
    let cols = columns(&spec, &[]);
    let issue = IssueSummary {
        number: 7,
        labels: vec!["bug".into(), "status::todo".into()],
        ..IssueSummary::default()
    };
    block_on(c.move_issue("me/p", &issue, &spec, &cols[2])).unwrap();
    let reqs = t.requests();
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].method, Method::Patch);
    assert_eq!(
        body_of(&reqs[0]),
        serde_json::json!({"labels": ["bug", "status::doing"]})
    );
    // To the Closed column: the state call only.
    block_on(c.move_issue("me/p", &issue, &spec, &cols[3])).unwrap();
    let reqs = t.requests();
    assert_eq!(reqs.len(), 2);
    assert_eq!(body_of(&reqs[1]), serde_json::json!({"state": "closed"}));
}
