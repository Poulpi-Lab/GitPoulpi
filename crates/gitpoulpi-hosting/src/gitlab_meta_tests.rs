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
        Integration::new(
            "gl",
            IntegrationKind::GitlabSelfManaged,
            "https://gitlab.example.com",
        ),
        "tok".into(),
        t,
    )
}

fn body_of(r: &HttpRequest) -> serde_json::Value {
    serde_json::from_str(r.body.as_deref().unwrap()).unwrap()
}

const MEMBERS: &str = r#"[{"id":11,"username":"ann","name":"Ann A","web_url":"https://gitlab.example.com/ann"},
    {"id":12,"username":"bob","name":"Bob B"}]"#;

#[test]
fn members_labels_and_milestones_use_the_encoded_project_path() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/members/all", 200, MEMBERS)
            .on(
                "/labels",
                200,
                r##"[{"name":"bug","color":"#d9534f","description":"Broken"}]"##,
            )
            .on(
                "/milestones",
                200,
                r#"[{"id":77,"iid":2,"title":"v2","due_date":"2026-12-01","state":"active"}]"#,
            ),
    );
    let c = client(t.clone());
    let people = block_on(c.project_members("g/sub/p", "an")).unwrap();
    assert_eq!(people[0].login, "ann");
    assert_eq!(people[0].name, "Ann A");
    assert_eq!(people[0].id, "11");
    let labels = block_on(c.project_labels("g/sub/p")).unwrap();
    assert_eq!(labels[0].color, "d9534f", "the # is dropped");
    let milestones = block_on(c.project_milestones("g/sub/p")).unwrap();
    assert_eq!(milestones[0].id, 77, "the global id, not the iid");
    assert_eq!(milestones[0].due_on.as_deref(), Some("2026-12-01"));
    assert_eq!(
        milestones[0].state, "open",
        "GitLab's `active` reads as `open`"
    );
    let urls: Vec<String> = t.requests().iter().map(|r| r.url.clone()).collect();
    assert!(urls[0].contains("/projects/g%2Fsub%2Fp/members/all?query=an&"));
    assert!(urls[1].contains("/projects/g%2Fsub%2Fp/labels?"));
    assert!(urls[2].contains("/projects/g%2Fsub%2Fp/milestones?state=active"));
}

#[test]
fn item_meta_reads_a_merge_request() {
    let t = Arc::new(RecordedTransport::new().on(
        "/merge_requests/5",
        200,
        r#"{"assignees":[{"username":"ann"}],"reviewers":[{"username":"bob"}],
            "labels":["bug","ui"],
            "milestone":{"id":77,"title":"v2","state":"active","due_date":null}}"#,
    ));
    let meta = block_on(client(t).item_meta("g/p", 5, MetaKind::PullRequest)).unwrap();
    assert_eq!(meta.assignees, vec!["ann"]);
    assert_eq!(meta.reviewers, vec!["bob"]);
    assert_eq!(meta.labels, vec!["bug", "ui"]);
    assert_eq!(meta.milestone.unwrap().id, 77);
}

#[test]
fn editing_a_merge_request_sends_ids_labels_and_milestone() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/members/all", 200, MEMBERS)
            .on("/merge_requests/5", 200, "{}"),
    );
    let edit = PrEdit {
        assignees: Some(vec!["ann".into()]),
        reviewers: Some(vec!["bob".into(), "ann".into()]),
        labels: Some(vec!["bug".into(), "ui".into()]),
        milestone: Some(Some(77)),
    };
    block_on(client(t.clone()).update_pull_request("g/p", 5, &edit)).unwrap();
    let put = t
        .requests()
        .into_iter()
        .find(|r| r.method == Method::Put)
        .expect("a PUT");
    assert!(put.url.ends_with("/projects/g%2Fp/merge_requests/5"));
    let body = body_of(&put);
    assert_eq!(body["assignee_ids"], serde_json::json!([11]));
    assert_eq!(body["reviewer_ids"], serde_json::json!([12, 11]));
    assert_eq!(body["labels"], "bug,ui");
    assert_eq!(body["milestone_id"], 77);
}

#[test]
fn clearing_sends_empty_values_and_a_zero_milestone() {
    let t = Arc::new(RecordedTransport::new().on("/issues/3", 200, "{}"));
    let edit = IssueEdit {
        assignees: Some(vec![]),
        labels: Some(vec![]),
        milestone: Some(None),
    };
    block_on(client(t.clone()).update_issue("g/p", 3, &edit)).unwrap();
    let requests = t.requests();
    assert_eq!(requests.len(), 1, "no member lookup to clear");
    let body = body_of(&requests[0]);
    assert_eq!(body["assignee_ids"], serde_json::json!([]));
    assert_eq!(body["labels"], "");
    assert_eq!(body["milestone_id"], 0);
}

#[test]
fn an_unknown_login_is_refused_before_anything_is_sent() {
    let t = Arc::new(RecordedTransport::new().on("/members/all", 200, MEMBERS));
    let edit = IssueEdit {
        assignees: Some(vec!["nobody".into()]),
        ..IssueEdit::default()
    };
    let err = block_on(client(t.clone()).update_issue("g/p", 3, &edit)).unwrap_err();
    assert!(err.to_string().contains("nobody"));
    assert!(t.requests().iter().all(|r| r.method == Method::Get));
}

#[test]
fn a_new_merge_request_and_a_new_issue_carry_their_metadata() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/members/all", 200, MEMBERS)
            .on(
                "/merge_requests",
                201,
                r#"{"iid":8,"web_url":"https://gitlab.example.com/g/p/-/merge_requests/8"}"#,
            )
            .on(
                "/issues",
                201,
                r#"{"iid":9,"web_url":"https://gitlab.example.com/g/p/-/issues/9"}"#,
            ),
    );
    let c = client(t.clone());
    block_on(c.create_pull_request(&NewPullRequest {
        project: "g/p".into(),
        source_branch: "f".into(),
        target_branch: "main".into(),
        title: "t".into(),
        assignees: vec!["ann".into()],
        reviewers: vec!["bob".into()],
        labels: vec!["bug".into()],
        milestone: Some(77),
        ..NewPullRequest::default()
    }))
    .unwrap();
    block_on(c.create_issue(&NewIssue {
        project: "g/p".into(),
        title: "t".into(),
        assignees: vec!["bob".into()],
        milestone: Some(77),
        ..NewIssue::default()
    }))
    .unwrap();
    let posts: Vec<HttpRequest> = t
        .requests()
        .into_iter()
        .filter(|r| r.method == Method::Post)
        .collect();
    let mr = body_of(&posts[0]);
    assert_eq!(mr["assignee_ids"], serde_json::json!([11]));
    assert_eq!(mr["reviewer_ids"], serde_json::json!([12]));
    assert_eq!(mr["labels"], "bug");
    assert_eq!(mr["milestone_id"], 77);
    let issue = body_of(&posts[1]);
    assert_eq!(issue["assignee_ids"], serde_json::json!([12]));
    assert_eq!(issue["milestone_id"], 77);
}
