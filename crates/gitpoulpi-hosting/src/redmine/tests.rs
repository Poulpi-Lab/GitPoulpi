use std::sync::Arc;

use serde_json::{json, Value};

use crate::board::BoardSource;
use crate::client::{client_for, HostingClient, NewIssue};
use crate::http::{HttpRequest, Method, RecordedTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::metadata::{IssueEdit, MetaKind};
use crate::model::{IssueFilter, IssueState, StateFilter};

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn client(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    let i = Integration::new("rm", IntegrationKind::Redmine, "http://127.0.0.1:3002/");
    client_for(
        i,
        "key-123".into(),
        t,
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap()
}

fn body(r: &HttpRequest) -> Value {
    serde_json::from_str(r.body.as_deref().unwrap()).unwrap()
}

fn header<'a>(r: &'a HttpRequest, name: &str) -> Option<&'a str> {
    r.headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

fn issue(id: u64, status: &str, closed: bool) -> Value {
    json!({
        "id": id,
        "project": { "id": 1, "name": "Bench" },
        "tracker": { "id": 1, "name": "Bug" },
        "status": { "id": 1, "name": status, "is_closed": closed },
        "author": { "id": 1, "name": "Redmine Admin" },
        "assigned_to": { "id": 1, "name": "Redmine Admin" },
        "fixed_version": { "id": 7, "name": "v0.1" },
        "subject": format!("Issue {id}"),
        "updated_on": "2026-10-15T06:13:39Z",
        "closed_on": if closed { json!("2026-10-15T07:00:00Z") } else { Value::Null }
    })
}

const STATUSES: &str = r#"{"issue_statuses":[
  {"id":1,"name":"New","is_closed":false},{"id":2,"name":"In Progress","is_closed":false},
  {"id":5,"name":"Closed","is_closed":true},{"id":6,"name":"Rejected","is_closed":true}]}"#;

const MEMBERS: &str = r#"{"memberships":[
  {"id":1,"project":{"id":1,"name":"Bench"},"user":{"id":1,"name":"Redmine Admin"}},
  {"id":2,"project":{"id":1,"name":"Bench"},"user":{"id":4,"name":"Ann Lee"}},
  {"id":3,"project":{"id":1,"name":"Bench"},"group":{"id":9,"name":"Devs"}}],"total_count":3}"#;

#[test]
fn the_key_travels_in_its_own_header_and_current_user_is_the_check() {
    let t = Arc::new(RecordedTransport::new().on(
        "/users/current.json",
        200,
        r#"{"user":{"id":1,"login":"admin","firstname":"Redmine","lastname":"Admin"}}"#,
    ));
    let check = block_on(client(t.clone()).check_token()).unwrap();
    let req = &t.requests()[0];
    assert_eq!(req.url, "http://127.0.0.1:3002/users/current.json");
    assert_eq!(header(req, "X-Redmine-API-Key"), Some("key-123"));
    assert_eq!(header(req, "Authorization"), None, "no second scheme");
    assert_eq!(check.user.login, "admin");
    assert_eq!(check.user.name, "Redmine Admin");
    assert!(check.scopes.is_none());
}

#[test]
fn a_rejected_key_is_unauthorized() {
    let t = Arc::new(RecordedTransport::new().on("/users/current.json", 401, ""));
    let err = block_on(client(t).check_token()).unwrap_err();
    assert!(matches!(err, crate::Error::Unauthorized { .. }), "{err:?}");
}

#[test]
fn projects_are_keyed_by_identifier() {
    let t = Arc::new(RecordedTransport::new().on(
        "/projects.json",
        200,
        r#"{"projects":[{"id":1,"name":"Bench","identifier":"bench","description":"","is_public":false}],"total_count":1}"#,
    ));
    let projects = block_on(client(t.clone()).list_projects()).unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].full_name, "bench");
    assert_eq!(projects[0].name, "Bench");
    assert!(projects[0].private);
    assert_eq!(
        projects[0].web_url.as_deref(),
        Some("http://127.0.0.1:3002/projects/bench")
    );
    assert!(t.requests()[0].url.contains("limit=100&offset=0"));
}

#[test]
fn issues_map_and_the_filter_becomes_query_parameters() {
    let page = json!({
        "issues": [issue(3, "New", false), issue(2, "Closed", true)],
        "total_count": 2, "offset": 0, "limit": 100
    });
    let t = Arc::new(RecordedTransport::new().on("/issues.json", 200, &page.to_string()));
    let rows = block_on(client(t.clone()).list_issues(
        "bench",
        &IssueFilter {
            state: StateFilter::Open,
            assigned_to_me: true,
            ..Default::default()
        },
    ))
    .unwrap();
    let url = &t.requests()[0].url;
    for part in [
        "project_id=bench",
        "status_id=open",
        "assigned_to_id=me",
        "sort=updated_on:desc",
    ] {
        assert!(url.contains(part), "{part} in {url}");
    }
    assert_eq!(rows.len(), 2);
    let first = &rows[0];
    assert_eq!(first.number, 3);
    assert_eq!(first.reference, "#3");
    assert_eq!(first.kind_label.as_deref(), Some("Bug"));
    assert_eq!(first.status_label.as_deref(), Some("New"));
    assert_eq!(first.assignees, ["Redmine Admin"]);
    assert_eq!(first.milestone.as_deref(), Some("v0.1"));
    assert_eq!(first.state, IssueState::Open);
    assert!(first.labels.is_empty(), "core Redmine has no labels");
    assert_eq!(first.web_url, "http://127.0.0.1:3002/issues/3");
    assert_eq!(rows[1].state, IssueState::Closed);
}

#[test]
fn a_reopened_issue_is_open_although_closed_on_is_still_set() {
    let mut reopened = issue(5, "New", false);
    reopened["closed_on"] = json!("2026-10-15T07:00:00Z");
    let t = Arc::new(RecordedTransport::new().on(
        "/issues.json",
        200,
        &json!({ "issues": [reopened], "total_count": 1 }).to_string(),
    ));
    let rows = block_on(client(t).list_issues("bench", &IssueFilter::default())).unwrap();
    assert_eq!(rows[0].state, IssueState::Open, "seen on a real Redmine");
}

#[test]
fn the_state_filter_spells_open_closed_and_all() {
    for (state, want) in [
        (StateFilter::Open, "status_id=open"),
        (StateFilter::Closed, "status_id=closed"),
        (StateFilter::All, "status_id=%2A"),
    ] {
        let t = Arc::new(RecordedTransport::new().on("/issues.json", 200, r#"{"issues":[]}"#));
        block_on(client(t.clone()).list_issues(
            "bench",
            &IssueFilter {
                state,
                ..Default::default()
            },
        ))
        .unwrap();
        let url = &t.requests()[0].url;
        assert!(url.contains(want), "{want} in {url}");
        assert!(!url.contains("assigned_to_id"), "{url}");
    }
}

#[test]
fn a_full_page_asks_for_the_next_one() {
    let rows: Vec<Value> = (1..=100).map(|n| issue(n, "New", false)).collect();
    let first = json!({ "issues": rows, "total_count": 101 }).to_string();
    let t = Arc::new(RecordedTransport::new().on("offset=0", 200, &first).on(
        "offset=100",
        200,
        &json!({ "issues": [issue(101, "New", false)], "total_count": 101 }).to_string(),
    ));
    let all = block_on(client(t.clone()).list_issues("bench", &IssueFilter::default())).unwrap();
    assert_eq!(all.len(), 101);
    assert_eq!(t.requests().len(), 2);
}

#[test]
fn creating_an_issue_sends_tracker_assignee_and_version() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/memberships.json", 200, MEMBERS)
            .on("/issues.json", 201, r#"{"issue":{"id":12}}"#),
    );
    let created = block_on(client(t.clone()).create_issue(&NewIssue {
        project: "bench".into(),
        title: "Crash".into(),
        body: "It crashes".into(),
        issue_type: Some("2".into()),
        assignees: vec!["ann lee".into()],
        milestone: Some(7),
        ..Default::default()
    }))
    .unwrap();
    assert_eq!(created.number, 12);
    assert_eq!(created.reference, "#12");
    assert_eq!(created.web_url, "http://127.0.0.1:3002/issues/12");
    let post = t
        .requests()
        .into_iter()
        .find(|r| r.method == Method::Post)
        .unwrap();
    assert_eq!(post.url, "http://127.0.0.1:3002/issues.json");
    assert_eq!(
        body(&post),
        json!({ "issue": {
            "project_id": "bench", "subject": "Crash", "description": "It crashes",
            "tracker_id": 2, "assigned_to_id": 4, "fixed_version_id": 7
        }})
    );
}

#[test]
fn a_label_is_refused_before_anything_is_sent() {
    let t = Arc::new(RecordedTransport::new());
    let err = block_on(client(t.clone()).create_issue(&NewIssue {
        project: "bench".into(),
        title: "x".into(),
        labels: vec!["bug".into()],
        ..Default::default()
    }))
    .unwrap_err();
    assert!(err.to_string().contains("no labels"), "{err}");
    assert!(t.requests().is_empty());
    let err = block_on(client(t.clone()).update_issue(
        "bench",
        3,
        &IssueEdit {
            labels: Some(vec!["bug".into()]),
            ..Default::default()
        },
    ))
    .unwrap_err();
    assert!(err.to_string().contains("no labels"), "{err}");
    assert!(t.requests().is_empty());
}

#[test]
fn trackers_are_the_forms_dropdown() {
    let t = Arc::new(RecordedTransport::new().on(
        "/projects/bench.json?include=trackers",
        200,
        r#"{"project":{"id":1,"trackers":[{"id":1,"name":"Bug"},{"id":2,"name":"Feature"}]}}"#,
    ));
    let options = block_on(client(t).issue_create_options("bench")).unwrap();
    let labels: Vec<_> = options.types.iter().map(|c| c.label.as_str()).collect();
    assert_eq!(labels, ["Bug", "Feature"]);
    assert_eq!(options.default_type.as_deref(), Some("1"));
    assert!(options.lists.is_empty());
}

#[test]
fn members_are_matched_by_name_and_filtered() {
    let t = Arc::new(RecordedTransport::new().on("/memberships.json", 200, MEMBERS));
    let users = block_on(client(t).project_members("bench", "lee")).unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].login, "Ann Lee");
    assert_eq!(users[0].id, "4");
}

#[test]
fn milestones_are_the_open_versions() {
    let t = Arc::new(RecordedTransport::new().on(
        "/projects/bench/versions.json",
        200,
        r#"{"versions":[
          {"id":7,"name":"v0.1","status":"open","due_date":"2026-11-14"},
          {"id":8,"name":"v0.0","status":"closed","due_date":null}]}"#,
    ));
    let list = block_on(client(t).project_milestones("bench")).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, 7);
    assert_eq!(list[0].due_on.as_deref(), Some("2026-11-14"));
}

#[test]
fn item_meta_reads_the_assignee_and_the_version() {
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "/issues/3.json",
                200,
                &json!({ "issue": issue(3, "New", false) }).to_string(),
            )
            .on(
                "/versions.json",
                200,
                r#"{"versions":[{"id":7,"name":"v0.1","status":"open","due_date":"2026-11-14"}]}"#,
            ),
    );
    let meta = block_on(client(t).item_meta("bench", 3, MetaKind::Issue)).unwrap();
    assert_eq!(meta.assignees, ["Redmine Admin"]);
    assert!(meta.labels.is_empty());
    let milestone = meta.milestone.unwrap();
    assert_eq!((milestone.id, milestone.title.as_str()), (7, "v0.1"));
}

#[test]
fn an_edit_puts_ids_and_clears_with_null() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/memberships.json", 200, MEMBERS)
            .on("/issues/3.json", 204, ""),
    );
    block_on(client(t.clone()).update_issue(
        "bench",
        3,
        &IssueEdit {
            assignees: Some(vec!["Ann Lee".into()]),
            milestone: Some(Some(7)),
            ..Default::default()
        },
    ))
    .unwrap();
    block_on(client(t.clone()).update_issue(
        "bench",
        3,
        &IssueEdit {
            assignees: Some(vec![]),
            milestone: Some(None),
            ..Default::default()
        },
    ))
    .unwrap();
    let puts: Vec<_> = t
        .requests()
        .into_iter()
        .filter(|r| r.method == Method::Put)
        .collect();
    assert_eq!(puts.len(), 2);
    assert_eq!(
        body(&puts[0]),
        json!({ "issue": { "assigned_to_id": 4, "fixed_version_id": 7 } })
    );
    assert_eq!(
        body(&puts[1]),
        json!({ "issue": { "assigned_to_id": null, "fixed_version_id": null } })
    );
}

#[test]
fn the_board_is_the_statuses_and_a_move_is_a_status_change() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/issue_statuses.json", 200, STATUSES)
            .on("/issues/3.json", 204, ""),
    );
    let c = client(t.clone());
    assert_eq!(c.board_source(), BoardSource::LocalStatus);
    let names = block_on(c.issue_statuses("bench")).unwrap();
    assert_eq!(names, ["New", "In Progress", "Closed", "Rejected"]);
    block_on(c.set_issue_status("bench", 3, "in progress")).unwrap();
    block_on(c.set_issue_state("bench", 3, true)).unwrap();
    block_on(c.set_issue_state("bench", 3, false)).unwrap();
    let puts: Vec<Value> = t
        .requests()
        .iter()
        .filter(|r| r.method == Method::Put)
        .map(body)
        .collect();
    assert_eq!(
        puts,
        [
            json!({ "issue": { "status_id": 2 } }),
            json!({ "issue": { "status_id": 5 } }),
            json!({ "issue": { "status_id": 1 } }),
        ]
    );
    let err = block_on(c.set_issue_status("bench", 3, "Nowhere")).unwrap_err();
    assert!(err.to_string().contains("no status"), "{err}");
}

#[test]
fn a_validation_error_reaches_the_user() {
    let t = Arc::new(RecordedTransport::new().on(
        "/issues.json",
        422,
        r#"{"errors":["Subject cannot be blank"]}"#,
    ));
    let err = block_on(client(t).create_issue(&NewIssue {
        project: "bench".into(),
        title: String::new(),
        ..Default::default()
    }))
    .unwrap_err();
    assert!(err.to_string().contains("Subject cannot be blank"), "{err}");
}

#[test]
fn pull_requests_and_ssh_keys_are_unsupported() {
    let t = Arc::new(RecordedTransport::new());
    assert!(block_on(client(t.clone()).list_ssh_keys()).is_err());
    assert!(t.requests().is_empty());
}

#[test]
fn the_issue_page_and_chip() {
    use crate::spec::spec_for;
    use gitpoulpi_config::ApiFamily;
    let spec = spec_for(ApiFamily::Redmine);
    assert_eq!(spec.tag, "RM");
    assert_eq!(
        (spec.issue_web_url)("http://h:3002/", "bench", "#12").as_deref(),
        Some("http://h:3002/issues/12")
    );
    assert_eq!((spec.issue_web_url)("http://h", "bench", "ABC-1"), None);
}

/// Reads against a real Redmine (the Docker bench: `scripts/test-bench.sh up
/// redmine`, then `seed redmine`). Run with `cargo test -p gitpoulpi-hosting
/// live_redmine -- --ignored --nocapture`, with `GITPOULPI_LIVE_REDMINE` (the
/// URL), `GITPOULPI_LIVE_REDMINE_KEY` and optionally
/// `GITPOULPI_LIVE_REDMINE_PROJECT` (default `bench`). Reads only. Never part
/// of the gate.
#[test]
#[ignore = "live: set GITPOULPI_LIVE_TESTS=1 and run with --ignored"]
fn live_redmine_reads() {
    if !gitpoulpi_testkit::live::enabled("live_redmine_reads") {
        return;
    }
    let (Ok(url), Ok(key)) = (
        std::env::var("GITPOULPI_LIVE_REDMINE"),
        std::env::var("GITPOULPI_LIVE_REDMINE_KEY"),
    ) else {
        panic!("set GITPOULPI_LIVE_REDMINE and GITPOULPI_LIVE_REDMINE_KEY");
    };
    let project =
        std::env::var("GITPOULPI_LIVE_REDMINE_PROJECT").unwrap_or_else(|_| "bench".to_string());
    let transport = Arc::new(crate::ReqwestTransport::new().expect("transport"));
    let client = client_for(
        Integration::new("live", IntegrationKind::Redmine, url),
        key,
        transport,
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    block_on(async {
        let check = client.check_token().await.expect("key");
        println!("user: {}", check.user.login);
        let projects = client.list_projects().await.expect("projects");
        assert!(projects.iter().any(|p| p.full_name == project));
        let issues = client
            .list_issues(&project, &IssueFilter::default())
            .await
            .expect("issues");
        println!("open issues: {}", issues.len());
        assert!(issues.iter().all(|i| i.state == IssueState::Open));
        let statuses = client.issue_statuses(&project).await.expect("statuses");
        assert!(!statuses.is_empty());
        let options = client
            .issue_create_options(&project)
            .await
            .expect("trackers");
        assert!(!options.types.is_empty());
        client.project_members(&project, "").await.expect("members");
        client.project_milestones(&project).await.expect("versions");
    });
}
