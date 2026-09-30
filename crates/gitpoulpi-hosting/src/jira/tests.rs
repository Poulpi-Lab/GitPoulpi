use std::sync::Arc;

use crate::auth::base64;
use crate::client::{client_for, HostingClient, NewIssue};
use crate::http::{HttpRequest, Method, RecordedTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::model::{IssueFilter, IssueState, StateFilter};

use super::jql::issues_jql;

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn header<'a>(request: &'a HttpRequest, name: &str) -> Option<&'a str> {
    request
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

fn cloud(transport: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    let mut i = Integration::new(
        "jc",
        IntegrationKind::JiraCloud,
        "https://acme.atlassian.net/",
    );
    i.account = "me@acme.com".into();
    client_for(i, "tok".into(), transport)
}

fn dc(transport: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    let i = Integration::new("jd", IntegrationKind::JiraDataCenter, "https://jira.corp");
    client_for(i, "pat".into(), transport)
}

fn issue(n: u32, done: bool) -> String {
    format!(
        r#"{{"id":"{}","key":"ABC-{n}","fields":{{"summary":"Issue {n}",
        "status":{{"name":"{}","statusCategory":{{"key":"{}"}}}},
        "issuetype":{{"name":"Bug"}},"assignee":{{"displayName":"Ann"}},
        "reporter":{{"displayName":"Bob"}},"labels":["ui"],
        "updated":"2026-10-01T10:00:00.000+0000"}}}}"#,
        10000 + n,
        if done { "Done" } else { "In Progress" },
        if done { "done" } else { "indeterminate" }
    )
}

#[test]
fn the_jql_is_exact_for_every_filter_combination() {
    let f = |state, mine, query: Option<&str>| IssueFilter {
        state,
        assigned_to_me: mine,
        query: query.map(String::from),
        ..Default::default()
    };
    assert_eq!(
        issues_jql("ABC", &f(StateFilter::Open, false, None)),
        r#"project = "ABC" AND statusCategory != Done ORDER BY updated DESC"#
    );
    assert_eq!(
        issues_jql("ABC", &f(StateFilter::Closed, true, Some("component = UI"))),
        r#"project = "ABC" AND statusCategory = Done AND assignee = currentUser() AND (component = UI) ORDER BY updated DESC"#
    );
    assert_eq!(
        issues_jql("IN", &f(StateFilter::All, false, Some("  "))),
        r#"project = "IN" ORDER BY updated DESC"#,
        "a reserved-word key is quoted; a blank filter adds nothing"
    );
}

#[test]
fn the_filter_is_parenthesised_and_a_trailing_order_by_is_dropped() {
    let f = |q: &str| IssueFilter {
        state: StateFilter::All,
        query: Some(q.into()),
        ..Default::default()
    };
    assert_eq!(
        issues_jql("A", &f("labels = x OR labels = y")),
        r#"project = "A" AND (labels = x OR labels = y) ORDER BY updated DESC"#,
        "the OR cannot escape into the project clause"
    );
    assert_eq!(
        issues_jql("A", &f("component = UI order by created ASC")),
        r#"project = "A" AND (component = UI) ORDER BY updated DESC"#
    );
    // An `order by` inside a quoted value is part of the value.
    assert_eq!(
        issues_jql("A", &f(r#"summary ~ "order by me""#)),
        r#"project = "A" AND (summary ~ "order by me") ORDER BY updated DESC"#
    );
    // Quotes in the key are escaped.
    assert_eq!(
        issues_jql(
            r#"A"B"#,
            &IssueFilter {
                state: StateFilter::All,
                ..Default::default()
            }
        ),
        r#"project = "A\"B" ORDER BY updated DESC"#
    );
}

#[test]
fn cloud_authenticates_with_basic_email_token_and_data_center_with_bearer() {
    let body = r#"{"accountId":"5b10","displayName":"Me","emailAddress":"me@acme.com"}"#;
    let t = Arc::new(RecordedTransport::new().on("/rest/api/3/myself", 200, body));
    let check = block_on(cloud(t.clone()).check_token()).unwrap();
    let req = &t.requests()[0];
    assert_eq!(req.url, "https://acme.atlassian.net/rest/api/3/myself");
    assert_eq!(
        header(req, "Authorization"),
        Some(format!("Basic {}", base64(b"me@acme.com:tok")).as_str())
    );
    assert_eq!(check.user.login, "me@acme.com");
    assert_eq!(check.user.id, "5b10");
    assert_eq!(check.user.name, "Me");
    assert!(check.scopes.is_none() && check.missing_scopes.is_empty());

    let body = r#"{"name":"jdoe","key":"JIRAUSER1","displayName":"John Doe"}"#;
    let t = Arc::new(RecordedTransport::new().on("/rest/api/2/myself", 200, body));
    let check = block_on(dc(t.clone()).check_token()).unwrap();
    let req = &t.requests()[0];
    assert_eq!(req.url, "https://jira.corp/rest/api/2/myself");
    assert_eq!(header(req, "Authorization"), Some("Bearer pat"));
    assert_eq!(check.user.login, "jdoe");
    assert_eq!(check.user.id, "JIRAUSER1");
}

#[test]
fn a_rejected_token_is_unauthorized() {
    let t = Arc::new(RecordedTransport::new().on("/myself", 401, "Basic authentication failed"));
    let err = block_on(cloud(t).check_token()).unwrap_err();
    assert!(matches!(err, crate::Error::Unauthorized { .. }), "{err}");
}

#[test]
fn cloud_projects_are_paged_and_map_to_keys() {
    let page1 = format!(
        r#"{{"isLast":false,"values":[{}]}}"#,
        (0..100)
            .map(|n| format!(r#"{{"key":"P{n}","name":"Project {n}"}}"#))
            .collect::<Vec<_>>()
            .join(",")
    );
    let page2 = r#"{"isLast":true,"values":[{"key":"ABC","name":"Alpha"}]}"#;
    let t = Arc::new(RecordedTransport::new().on("startAt=0", 200, &page1).on(
        "startAt=100",
        200,
        page2,
    ));
    let projects = block_on(cloud(t.clone()).list_projects()).unwrap();
    assert_eq!(projects.len(), 101);
    let last = projects.last().unwrap();
    assert_eq!(last.full_name, "ABC");
    assert_eq!(last.name, "Alpha");
    assert_eq!(
        last.web_url.as_deref(),
        Some("https://acme.atlassian.net/browse/ABC")
    );
    assert_eq!(last.integration_id, "jc");
    assert_eq!(t.requests().len(), 2);
    assert!(t.requests()[0].url.contains("/rest/api/3/project/search?"));
}

#[test]
fn data_center_projects_are_a_plain_array() {
    let t = Arc::new(RecordedTransport::new().on(
        "/rest/api/2/project",
        200,
        r#"[{"key":"OPS","name":"Operations"}]"#,
    ));
    let projects = block_on(dc(t.clone()).list_projects()).unwrap();
    assert_eq!(projects[0].full_name, "OPS");
    assert_eq!(
        projects[0].web_url.as_deref(),
        Some("https://jira.corp/browse/OPS")
    );
    assert_eq!(t.requests().len(), 1);
}

#[test]
fn cloud_issues_follow_next_page_token_and_never_call_the_removed_search() {
    let p1 = format!(
        r#"{{"issues":[{},{}],"nextPageToken":"tok/2=","isLast":false}}"#,
        issue(12, false),
        issue(11, true)
    );
    let p2 = format!(r#"{{"issues":[{}],"isLast":true}}"#, issue(3, false));
    let t = Arc::new(
        RecordedTransport::new()
            .on("nextPageToken=tok%2F2%3D", 200, &p2)
            .on("/rest/api/3/search/jql?", 200, &p1),
    );
    let rows = block_on(cloud(t.clone()).list_issues(
        "ABC",
        &IssueFilter {
            state: StateFilter::All,
            ..Default::default()
        },
    ))
    .unwrap();
    assert_eq!(rows.len(), 3);
    let first = &rows[0];
    assert_eq!(first.reference, "ABC-12");
    assert_eq!(first.iid_label, "ABC-12");
    assert_eq!(first.number, 12);
    assert_eq!(first.id, "10012");
    assert_eq!(first.title, "Issue 12");
    assert_eq!(first.state, IssueState::Open);
    assert_eq!(first.status_label.as_deref(), Some("In Progress"));
    assert_eq!(first.kind_label.as_deref(), Some("Bug"));
    assert_eq!(first.author, "Bob");
    assert_eq!(first.assignees, vec!["Ann"]);
    assert_eq!(first.labels, vec!["ui"]);
    assert_eq!(first.web_url, "https://acme.atlassian.net/browse/ABC-12");
    assert_eq!(rows[1].state, IssueState::Closed, "statusCategory done");

    let urls: Vec<String> = t.requests().into_iter().map(|r| r.url).collect();
    assert_eq!(urls.len(), 2);
    assert!(urls.iter().all(|u| u.contains("/rest/api/3/search/jql?")));
    assert!(
        urls.iter().all(|u| !u.contains("/rest/api/3/search?")),
        "the removed endpoint (410) is never called"
    );
    assert!(urls[0].contains("jql=project%20%3D%20%22ABC%22%20ORDER%20BY%20updated%20DESC"));
    assert!(urls[0].contains("fields=summary,status,issuetype,assignee,reporter,labels,updated"));
    assert!(!urls[0].contains("nextPageToken"));
}

#[test]
fn cloud_issue_list_is_capped_at_500_rows() {
    let page = format!(
        r#"{{"issues":[{}],"nextPageToken":"more"}}"#,
        (0..100)
            .map(|n| issue(n + 1, false))
            .collect::<Vec<_>>()
            .join(",")
    );
    let t = Arc::new(RecordedTransport::new().on("/search/jql", 200, &page));
    let rows = block_on(cloud(t.clone()).list_issues("ABC", &IssueFilter::default())).unwrap();
    assert_eq!(rows.len(), 500);
    assert_eq!(t.requests().len(), 5);
}

#[test]
fn data_center_issues_page_with_start_at_until_total() {
    let p1 = format!(
        r#"{{"startAt":0,"total":3,"issues":[{},{}]}}"#,
        issue(3, false),
        issue(2, false)
    );
    let p2 = format!(r#"{{"startAt":2,"total":3,"issues":[{}]}}"#, issue(1, true));
    let t = Arc::new(
        RecordedTransport::new()
            .on("startAt=2", 200, &p2)
            .on("startAt=0", 200, &p1),
    );
    let rows = block_on(dc(t.clone()).list_issues("ABC", &IssueFilter::default())).unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[2].state, IssueState::Closed);
    let reqs = t.requests();
    assert_eq!(reqs.len(), 2);
    assert!(reqs[0]
        .url
        .starts_with("https://jira.corp/rest/api/2/search?jql="));
    assert!(reqs[0].url.contains("statusCategory%20%21%3D%20Done"));
    assert_eq!(header(&reqs[0], "Authorization"), Some("Bearer pat"));
}

#[test]
fn a_429_with_retry_after_is_a_rate_limit_error() {
    let t = Arc::new(RecordedTransport::new().on_with_headers(
        "/search/jql",
        429,
        "",
        &[
            ("Retry-After", "42"),
            ("RateLimit-Reason", "jira-burst-based"),
        ],
    ));
    let err = block_on(cloud(t).list_issues("ABC", &IssueFilter::default())).unwrap_err();
    assert!(
        matches!(
            err,
            crate::Error::RateLimited {
                retry_in_secs: 42,
                ..
            }
        ),
        "{err:?}"
    );
}

#[test]
fn create_options_skip_subtasks_and_default_to_task() {
    let body = r#"{"startAt":0,"maxResults":50,"total":3,"issueTypes":[
        {"id":"1","name":"Bug","subtask":false},
        {"id":"2","name":"Sub-task","subtask":true},
        {"id":"3","name":"Task","subtask":false}]}"#;
    let t = Arc::new(RecordedTransport::new().on("/issue/createmeta/ABC/issuetypes", 200, body));
    let options = block_on(cloud(t.clone()).issue_create_options("ABC")).unwrap();
    let ids: Vec<&str> = options.types.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, ["1", "3"]);
    assert_eq!(options.default_type.as_deref(), Some("3"));
    assert!(options.lists.is_empty());
    assert!(t.requests()[0]
        .url
        .starts_with("https://acme.atlassian.net/rest/api/3/issue/createmeta/"));

    // Data Center answers `values`; no Task means the first type.
    let body = r#"{"startAt":0,"total":1,"values":[{"id":"7","name":"Story"}]}"#;
    let t = Arc::new(RecordedTransport::new().on("/issuetypes", 200, body));
    let options = block_on(dc(t.clone()).issue_create_options("OPS")).unwrap();
    assert_eq!(options.default_type.as_deref(), Some("7"));
    assert!(t.requests()[0]
        .url
        .starts_with("https://jira.corp/rest/api/2/issue/createmeta/OPS/"));
}

#[test]
fn create_issue_posts_v2_plain_text_with_the_type_id_on_both_flavours() {
    let created = r#"{"id":"10050","key":"ABC-50","self":"x"}"#;
    for (client, site, v2) in [
        (
            cloud as fn(_) -> _,
            "https://acme.atlassian.net",
            "/rest/api/2/issue",
        ),
        (dc, "https://jira.corp", "/rest/api/2/issue"),
    ] {
        let t = Arc::new(RecordedTransport::new().on("/issue", 201, created));
        let made = block_on(client(t.clone()).create_issue(&NewIssue {
            project: "ABC".into(),
            title: "Broken".into(),
            body: "plain text".into(),
            labels: vec!["needs review".into(), "ui".into()],
            issue_type: Some("10001".into()),
            list_id: None,
            ..Default::default()
        }))
        .unwrap();
        assert_eq!(made.reference, "ABC-50");
        assert_eq!(made.number, 50);
        assert_eq!(made.web_url, format!("{site}/browse/ABC-50"));
        let req = &t.requests()[0];
        assert_eq!(req.method, Method::Post);
        assert_eq!(req.url, format!("{site}{v2}"), "v2 even on Cloud");
        let body: serde_json::Value = serde_json::from_str(req.body.as_deref().unwrap()).unwrap();
        assert_eq!(
            body,
            serde_json::json!({"fields": {
                "project": {"key": "ABC"},
                "summary": "Broken",
                "issuetype": {"id": "10001"},
                "description": "plain text",
                "labels": ["needs-review", "ui"],
            }})
        );
    }
}

#[test]
fn create_issue_without_a_type_uses_the_projects_default() {
    let meta = r#"{"total":1,"issueTypes":[{"id":"3","name":"Task"}]}"#;
    let t = Arc::new(RecordedTransport::new().on("createmeta", 200, meta).on(
        "/rest/api/2/issue",
        201,
        r#"{"id":"1","key":"ABC-1"}"#,
    ));
    block_on(cloud(t.clone()).create_issue(&NewIssue {
        project: "ABC".into(),
        title: "t".into(),
        ..Default::default()
    }))
    .unwrap();
    let post = t
        .requests()
        .into_iter()
        .find(|r| r.method == Method::Post)
        .unwrap();
    let body: serde_json::Value = serde_json::from_str(post.body.as_deref().unwrap()).unwrap();
    assert_eq!(body["fields"]["issuetype"]["id"], "3");
    assert!(body["fields"].get("description").is_none());
}

#[test]
fn there_are_no_pull_requests_or_ssh_keys() {
    let t = Arc::new(RecordedTransport::new());
    let c = cloud(t.clone());
    assert!(matches!(
        block_on(c.list_pull_requests("ABC", &Default::default())),
        Err(crate::Error::Unsupported(_))
    ));
    assert!(matches!(
        block_on(c.list_ssh_keys()),
        Err(crate::Error::Unsupported(_))
    ));
    assert!(t.requests().is_empty());
}

#[test]
fn the_spec_links_to_the_browse_page() {
    use crate::spec::spec_for;
    let spec = spec_for(gitpoulpi_config::ApiFamily::Jira);
    assert_eq!(
        (spec.issue_web_url)("https://acme.atlassian.net/", "ABC", "ABC-12").as_deref(),
        Some("https://acme.atlassian.net/browse/ABC-12")
    );
    assert_eq!(
        (spec.issue_web_url)("https://x", "ABC", " ").as_deref(),
        None
    );
    assert!(IntegrationKind::IMPLEMENTED.contains(&IntegrationKind::JiraCloud));
    assert!(IntegrationKind::IMPLEMENTED.contains(&IntegrationKind::JiraDataCenter));
}

// ---- the Kanban board (L2 follow-up): statuses and transitions ---------

const STATUSES: &str = r#"[
  {"name":"Bug","statuses":[
    {"id":"1","name":"To Do","statusCategory":{"key":"new"}},
    {"id":"3","name":"Done","statusCategory":{"key":"done"}},
    {"id":"2","name":"In Progress","statusCategory":{"key":"indeterminate"}}]},
  {"name":"Task","statuses":[
    {"id":"1","name":"To Do","statusCategory":{"key":"new"}},
    {"id":"4","name":"In Review","statusCategory":{"key":"indeterminate"}},
    {"id":"3","name":"Done","statusCategory":{"key":"done"}}]}]"#;

const TRANSITIONS: &str = r#"{"transitions":[
  {"id":"11","name":"Start","to":{"id":"2","name":"In Progress"}},
  {"id":"31","name":"Finish","to":{"id":"3","name":"Done"}}]}"#;

fn board_issue(status: &str) -> crate::model::IssueSummary {
    crate::model::IssueSummary {
        number: 7,
        id: "10007".into(),
        reference: "ABC-7".into(),
        status_label: Some(status.into()),
        ..Default::default()
    }
}

fn column(name: &str) -> crate::board::BoardColumn {
    crate::board::columns(
        &crate::board::BoardSpec {
            source: crate::board::BoardSource::LocalStatus,
            declared: vec![name.into()],
            labels: Vec::new(),
        },
        &[],
    )
    .remove(0)
}

fn spec() -> crate::board::BoardSpec {
    crate::board::BoardSpec {
        source: crate::board::BoardSource::LocalStatus,
        ..Default::default()
    }
}

type Make = fn(Arc<RecordedTransport>) -> Box<dyn HostingClient>;

#[test]
fn both_flavours_read_the_columns_from_the_project_statuses() {
    for (make, api) in [
        (cloud as Make, "https://acme.atlassian.net/rest/api/3"),
        (dc as Make, "https://jira.corp/rest/api/2"),
    ] {
        let t = Arc::new(RecordedTransport::new().on("/project/ABC/statuses", 200, STATUSES));
        let c = make(t.clone());
        assert_eq!(c.board_source(), crate::board::BoardSource::LocalStatus);
        let names = block_on(c.issue_statuses("ABC")).unwrap();
        assert_eq!(names, ["To Do", "In Progress", "In Review", "Done"]);
        assert_eq!(t.requests()[0].url, format!("{api}/project/ABC/statuses"));
    }
}

#[test]
fn a_move_reads_the_transitions_then_posts_the_one_leading_to_the_column() {
    for (make, api) in [
        (cloud as Make, "https://acme.atlassian.net/rest/api/3"),
        (dc as Make, "https://jira.corp/rest/api/2"),
    ] {
        let t = Arc::new(RecordedTransport::new().on("/issue/ABC-7/transitions", 200, TRANSITIONS));
        let c = make(t.clone());
        block_on(c.move_issue(
            "ABC",
            &board_issue("To Do"),
            &spec(),
            &column("in progress"),
        ))
        .unwrap();
        let reqs = t.requests();
        assert_eq!(reqs.len(), 2);
        assert_eq!(reqs[0].method, Method::Get);
        assert_eq!(reqs[0].url, format!("{api}/issue/ABC-7/transitions"));
        assert_eq!(reqs[1].method, Method::Post);
        assert_eq!(reqs[1].url, format!("{api}/issue/ABC-7/transitions"));
        let body: serde_json::Value =
            serde_json::from_str(reqs[1].body.as_deref().unwrap()).unwrap();
        assert_eq!(body, serde_json::json!({"transition": {"id": "11"}}));
    }
}

#[test]
fn a_move_with_no_transition_to_the_column_sends_nothing_and_says_why() {
    let t = Arc::new(RecordedTransport::new().on("/transitions", 200, TRANSITIONS));
    let c = dc(t.clone());
    let err = block_on(c.move_issue("ABC", &board_issue("To Do"), &spec(), &column("In Review")))
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "no transition from To Do to In Review for ABC-7"
    );
    assert_eq!(t.requests().len(), 1, "only the GET");
}

#[test]
fn a_transition_is_also_found_by_status_id_and_a_same_column_move_is_a_no_op() {
    let t = Arc::new(RecordedTransport::new().on("/transitions", 200, TRANSITIONS));
    let c = cloud(t.clone());
    block_on(c.move_issue("ABC", &board_issue("To Do"), &spec(), &column("3"))).unwrap();
    let body: serde_json::Value =
        serde_json::from_str(t.requests()[1].body.as_deref().unwrap()).unwrap();
    assert_eq!(body["transition"]["id"], "31");
    let before = t.requests().len();
    block_on(c.move_issue("ABC", &board_issue("Done"), &spec(), &column("done"))).unwrap();
    assert_eq!(t.requests().len(), before);
}
