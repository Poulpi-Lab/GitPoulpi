use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

use crate::client::{client_for, HostingClient, NewIssue};
use crate::http::{HttpRequest, HttpResponse, HttpTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::metadata::IssueEdit;
use crate::model::{IssueFilter, IssueState, StateFilter};
use crate::Error;

use super::client::issues_filter;

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

type Handler = Box<dyn Fn(&str, &Value) -> (u16, String) + Send + Sync>;

/// A GraphQL fake: the handler sees the document and its variables and
/// answers; every request is kept for the asserts.
struct Fake {
    handler: Handler,
    seen: Mutex<Vec<HttpRequest>>,
}

impl Fake {
    fn new(handler: impl Fn(&str, &Value) -> (u16, String) + Send + Sync + 'static) -> Arc<Self> {
        Arc::new(Self {
            handler: Box::new(handler),
            seen: Mutex::new(Vec::new()),
        })
    }

    /// `(document, variables)` of every request, in order.
    fn calls(&self) -> Vec<(String, Value)> {
        self.seen
            .lock()
            .unwrap()
            .iter()
            .map(|r| {
                let body: Value = serde_json::from_str(r.body.as_deref().unwrap()).unwrap();
                (
                    body["query"].as_str().unwrap().to_string(),
                    body["variables"].clone(),
                )
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl HttpTransport for Fake {
    async fn send(&self, request: HttpRequest) -> crate::Result<HttpResponse> {
        self.seen.lock().unwrap().push(request.clone());
        let body: Value = serde_json::from_str(request.body.as_deref().unwrap()).unwrap();
        let (status, text) = (self.handler)(body["query"].as_str().unwrap(), &body["variables"]);
        Ok(HttpResponse {
            status,
            headers: Default::default(),
            body: text,
        })
    }
}

fn client(t: Arc<Fake>) -> Box<dyn HostingClient> {
    let i = Integration::new("ln", IntegrationKind::Linear, "https://linear.app");
    client_for(i, "lin_api_key".into(), t)
}

fn data(v: Value) -> (u16, String) {
    (200, json!({ "data": v }).to_string())
}

fn issue_node(n: u32, kind: &str) -> Value {
    json!({
        "id": format!("uuid-{n}"), "identifier": format!("ENG-{n}"), "number": n,
        "title": format!("Issue {n}"), "url": format!("https://linear.app/acme/issue/ENG-{n}/x"),
        "branchName": format!("eng-{n}-issue-{n}"), "updatedAt": "2026-10-01T10:00:00.000Z",
        "priorityLabel": "High", "state": { "name": "In Progress", "type": kind },
        "creator": { "name": "Bob Builder", "displayName": "bob" },
        "assignee": { "name": "Ann Lee", "displayName": "ann" },
        "labels": { "nodes": [ { "name": "bug" } ] }
    })
}

const TEAM: &str = r#"{"teams":{"nodes":[{"id":"team-1","key":"ENG","name":"Engineering"}]}}"#;

fn team() -> (u16, String) {
    data(serde_json::from_str(TEAM).unwrap())
}

#[test]
fn the_key_goes_bare_in_authorization_and_the_viewer_is_the_token_check() {
    let t = Fake::new(|_, _| {
        data(
            json!({ "viewer": { "id": "u1", "name": "Ann Lee", "displayName": "ann",
            "email": "ann@acme.com", "url": "https://linear.app/acme/profiles/ann" } }),
        )
    });
    let check = block_on(client(t.clone()).check_token()).unwrap();
    let req = &t.seen.lock().unwrap()[0].clone();
    assert_eq!(req.url, "https://api.linear.app/graphql");
    let auth = req
        .headers
        .iter()
        .find(|(k, _)| k == "Authorization")
        .map(|(_, v)| v.as_str());
    assert_eq!(auth, Some("lin_api_key"), "no Bearer prefix");
    assert_eq!(check.user.login, "ann");
    assert_eq!(check.user.name, "Ann Lee");
    assert_eq!(check.user.id, "u1");
    assert!(check.scopes.is_none() && check.missing_scopes.is_empty());
}

#[test]
fn the_issue_filter_is_exact_for_every_combination() {
    let f = |state, mine| IssueFilter {
        state,
        assigned_to_me: mine,
        ..Default::default()
    };
    assert_eq!(
        issues_filter("ENG", &f(StateFilter::Open, false)),
        json!({ "team": { "key": { "eq": "ENG" } },
                "state": { "type": { "nin": ["completed", "canceled"] } } })
    );
    assert_eq!(
        issues_filter("ENG", &f(StateFilter::Closed, true)),
        json!({ "team": { "key": { "eq": "ENG" } },
                "state": { "type": { "in": ["completed", "canceled"] } },
                "assignee": { "isMe": { "eq": true } } })
    );
    assert_eq!(
        issues_filter(" ENG ", &f(StateFilter::All, false)),
        json!({ "team": { "key": { "eq": "ENG" } } })
    );
}

#[test]
fn issues_are_listed_by_cursor_and_mapped() {
    let t = Fake::new(|_, vars| match vars["after"].as_str() {
        None => data(json!({ "issues": {
            "nodes": [issue_node(1, "started"), issue_node(2, "completed")],
            "pageInfo": { "hasNextPage": true, "endCursor": "cur-1" } } })),
        Some("cur-1") => data(json!({ "issues": {
            "nodes": [issue_node(3, "canceled")],
            "pageInfo": { "hasNextPage": false, "endCursor": null } } })),
        Some(other) => panic!("unexpected cursor {other}"),
    });
    let filter = IssueFilter {
        state: StateFilter::All,
        assigned_to_me: true,
        ..Default::default()
    };
    let rows = block_on(client(t.clone()).list_issues("ENG", &filter)).unwrap();
    assert_eq!(rows.len(), 3);
    let first = &rows[0];
    assert_eq!(first.reference, "ENG-1");
    assert_eq!(first.number, 1);
    assert_eq!(first.id, "uuid-1");
    assert_eq!(first.title, "Issue 1");
    assert_eq!(first.state, IssueState::Open);
    assert_eq!(first.status_label.as_deref(), Some("In Progress"));
    assert_eq!(first.kind_label.as_deref(), Some("High"));
    assert_eq!(first.assignees, vec!["ann"]);
    assert_eq!(first.author, "bob");
    assert_eq!(first.labels, vec!["bug"]);
    assert_eq!(first.branch_name.as_deref(), Some("eng-1-issue-1"));
    assert_eq!(first.web_url, "https://linear.app/acme/issue/ENG-1/x");
    assert_eq!(rows[1].state, IssueState::Closed, "completed is closed");
    assert_eq!(rows[2].state, IssueState::Closed, "canceled is closed");

    let calls = t.calls();
    assert_eq!(calls.len(), 2);
    assert!(
        calls[0].0.contains("issues(filter: $filter"),
        "{}",
        calls[0].0
    );
    assert_eq!(
        calls[0].1["filter"],
        json!({ "team": { "key": { "eq": "ENG" } }, "assignee": { "isMe": { "eq": true } } })
    );
    assert_eq!(calls[0].1["after"], Value::Null);
    assert_eq!(calls[1].1["after"], json!("cur-1"));
}

#[test]
fn teams_are_the_projects_and_the_workspace_slug_builds_their_page() {
    let t = Fake::new(|_, _| {
        data(json!({ "organization": { "urlKey": "acme" },
            "teams": { "nodes": [
                { "id": "t1", "key": "ENG", "name": "Engineering", "description": "The code" },
                { "id": "t2", "key": "OPS", "name": "Operations" } ],
              "pageInfo": { "hasNextPage": false, "endCursor": null } } }))
    });
    let projects = block_on(client(t).list_projects()).unwrap();
    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].full_name, "ENG");
    assert_eq!(projects[0].name, "Engineering");
    assert_eq!(projects[0].description.as_deref(), Some("The code"));
    assert_eq!(
        projects[0].web_url.as_deref(),
        Some("https://linear.app/acme/team/ENG/all")
    );
    assert_eq!(projects[1].full_name, "OPS");
}

#[test]
fn creating_an_issue_resolves_names_to_ids_and_sends_one_mutation() {
    let t = Fake::new(|q, _| {
        if q.contains("query Team(") {
            team()
        } else if q.contains("query Labels") {
            data(json!({ "issueLabels": { "nodes": [
                { "id": "l-bug", "name": "Bug", "color": "#eb5757" },
                { "id": "l-ui", "name": "UI", "color": "#000000" } ],
                "pageInfo": { "hasNextPage": false } } }))
        } else if q.contains("query Members") {
            data(json!({ "team": { "members": { "nodes": [
                { "id": "u-ann", "name": "Ann Lee", "displayName": "ann", "email": "a@x", "active": true },
                { "id": "u-old", "name": "Old", "displayName": "old", "email": "o@x", "active": false } ],
                "pageInfo": { "hasNextPage": false } } } }))
        } else if q.contains("query Cycle(") {
            data(json!({ "cycles": { "nodes": [ { "id": "cyc-7", "number": 7 } ] } }))
        } else if q.contains("mutation IssueCreate") {
            data(json!({ "issueCreate": { "success": true, "issue": {
                "id": "new", "identifier": "ENG-42", "number": 42,
                "url": "https://linear.app/acme/issue/ENG-42/t" } } }))
        } else {
            panic!("unexpected document {q}")
        }
    });
    let created = block_on(client(t.clone()).create_issue(&NewIssue {
        project: "ENG".into(),
        title: "Crash".into(),
        body: "It crashes".into(),
        labels: vec!["bug".into()],
        assignees: vec!["ANN".into(), "ignored".into()],
        milestone: Some(7),
        list_id: Some("state-todo".into()),
        ..Default::default()
    }))
    .unwrap();
    assert_eq!(created.reference, "ENG-42");
    assert_eq!(created.number, 42);
    assert_eq!(created.web_url, "https://linear.app/acme/issue/ENG-42/t");
    let calls = t.calls();
    let (doc, vars) = calls.last().unwrap();
    assert!(doc.contains("issueCreate(input: $input)"));
    assert_eq!(
        vars["input"],
        json!({
            "teamId": "team-1", "title": "Crash", "description": "It crashes",
            "labelIds": ["l-bug"], "assigneeId": "u-ann", "cycleId": "cyc-7",
            "stateId": "state-todo"
        })
    );
}

#[test]
fn an_unknown_label_is_refused_before_anything_is_created() {
    let t = Fake::new(|q, _| {
        if q.contains("query Team(") {
            team()
        } else {
            data(json!({ "issueLabels": { "nodes": [],
                "pageInfo": { "hasNextPage": false } } }))
        }
    });
    let err = block_on(client(t.clone()).create_issue(&NewIssue {
        project: "ENG".into(),
        title: "x".into(),
        labels: vec!["nope".into()],
        ..Default::default()
    }))
    .unwrap_err();
    assert!(err.to_string().contains("nope"), "{err}");
    assert!(!t.calls().iter().any(|(q, _)| q.contains("mutation")));
}

#[test]
fn editing_clears_or_sets_the_assignee_labels_and_cycle() {
    let t = Fake::new(|q, _| {
        if q.contains("query Team(") {
            team()
        } else if q.contains("query IssueId") {
            data(json!({ "issues": { "nodes": [ { "id": "uuid-9" } ] } }))
        } else if q.contains("query Labels") {
            data(
                json!({ "issueLabels": { "nodes": [ { "id": "l-bug", "name": "bug" } ],
                "pageInfo": { "hasNextPage": false } } }),
            )
        } else if q.contains("mutation IssueUpdate") {
            data(json!({ "issueUpdate": { "success": true } }))
        } else {
            panic!("unexpected document {q}")
        }
    });
    block_on(client(t.clone()).update_issue(
        "ENG",
        9,
        &IssueEdit {
            assignees: Some(vec![]),
            labels: Some(vec!["Bug".into()]),
            milestone: Some(None),
        },
    ))
    .unwrap();
    let calls = t.calls();
    let (doc, vars) = calls.last().unwrap();
    assert!(doc.contains("issueUpdate(id: $id"));
    assert_eq!(vars["id"], json!("uuid-9"));
    assert_eq!(
        vars["input"],
        json!({ "assigneeId": null, "labelIds": ["l-bug"], "cycleId": null })
    );
    let find = calls
        .iter()
        .find(|(q, _)| q.contains("query IssueId"))
        .unwrap();
    assert_eq!(
        find.1["filter"],
        json!({ "team": { "key": { "eq": "ENG" } }, "number": { "eq": 9 } })
    );
}

#[test]
fn cycles_are_the_milestones_and_labels_lose_their_hash() {
    let t = Fake::new(|q, _| {
        if q.contains("query Team(") {
            team()
        } else if q.contains("query Cycles") {
            data(json!({ "team": { "cycles": { "nodes": [
                { "id": "c8", "number": 8, "name": null, "endsAt": "2026-11-15T00:00:00.000Z", "isActive": false },
                { "id": "c7", "number": 7, "name": "Autumn", "endsAt": "2026-11-01T00:00:00.000Z", "isActive": true } ],
                "pageInfo": { "hasNextPage": false } } } }))
        } else {
            data(json!({ "issueLabels": { "nodes": [
                { "id": "l1", "name": "bug", "color": "#eb5757", "description": "Broken" } ],
                "pageInfo": { "hasNextPage": false } } }))
        }
    });
    let c = client(t);
    let cycles = block_on(c.project_milestones("ENG")).unwrap();
    assert_eq!(cycles.iter().map(|m| m.id).collect::<Vec<_>>(), vec![7, 8]);
    assert_eq!(cycles[0].title, "Autumn");
    assert_eq!(cycles[0].state, "active");
    assert_eq!(cycles[0].due_on.as_deref(), Some("2026-11-01"));
    assert_eq!(cycles[1].title, "Cycle 8");
    let labels = block_on(c.project_labels("ENG")).unwrap();
    assert_eq!(labels[0].name, "bug");
    assert_eq!(labels[0].color, "eb5757");
    assert_eq!(labels[0].description, "Broken");
}

fn states_answer() -> (u16, String) {
    data(
        json!({ "team": { "defaultIssueState": { "id": "s-todo" }, "states": { "nodes": [
        { "id": "s-done", "name": "Done", "type": "completed", "position": 0 },
        { "id": "s-prog", "name": "In Progress", "type": "started", "position": 1 },
        { "id": "s-todo", "name": "Todo", "type": "unstarted", "position": 0 },
        { "id": "s-back", "name": "Backlog", "type": "backlog", "position": 0 },
        { "id": "s-cancel", "name": "Canceled", "type": "canceled", "position": 1 } ],
        "pageInfo": { "hasNextPage": false } } } }),
    )
}

#[test]
fn states_are_the_board_columns_and_close_picks_the_completed_one() {
    let t = Fake::new(|q, _| {
        if q.contains("query Team(") {
            team()
        } else if q.contains("query States") {
            states_answer()
        } else if q.contains("query IssueId") {
            data(json!({ "issues": { "nodes": [ { "id": "uuid-5" } ] } }))
        } else {
            data(json!({ "issueUpdate": { "success": true } }))
        }
    });
    let c = client(t.clone());
    assert_eq!(
        block_on(c.issue_statuses("ENG")).unwrap(),
        vec!["Backlog", "Todo", "In Progress", "Done", "Canceled"]
    );
    block_on(c.set_issue_state("ENG", 5, true)).unwrap();
    assert_eq!(
        t.calls().last().unwrap().1,
        json!({ "id": "uuid-5", "input": { "stateId": "s-done" } })
    );
    block_on(c.set_issue_state("ENG", 5, false)).unwrap();
    assert_eq!(
        t.calls().last().unwrap().1["input"],
        json!({ "stateId": "s-todo" })
    );
    block_on(c.set_issue_status("ENG", 5, "in progress")).unwrap();
    assert_eq!(
        t.calls().last().unwrap().1["input"],
        json!({ "stateId": "s-prog" })
    );
    let err = block_on(c.set_issue_status("ENG", 5, "Nowhere")).unwrap_err();
    assert!(err.to_string().contains("Nowhere"), "{err}");
}

#[test]
fn the_new_issue_form_offers_the_open_states_with_the_teams_default() {
    let t = Fake::new(|q, _| {
        if q.contains("query Team(") {
            team()
        } else {
            states_answer()
        }
    });
    let options = block_on(client(t).issue_create_options("ENG")).unwrap();
    let labels: Vec<&str> = options.lists.iter().map(|l| l.label.as_str()).collect();
    assert_eq!(labels, vec!["Backlog", "Todo", "In Progress"]);
    assert_eq!(options.default_list.as_deref(), Some("s-todo"));
}

#[test]
fn an_unknown_team_is_a_clear_error() {
    let t = Fake::new(|_, _| data(json!({ "teams": { "nodes": [] } })));
    let err = block_on(client(t).project_labels("ZZZ")).unwrap_err();
    assert!(err.to_string().contains("ZZZ"), "{err}");
}

#[test]
fn graphql_errors_become_typed_errors() {
    let status = |code: u16, body: Value| {
        let t = Fake::new(move |_, _| (code, body.to_string()));
        block_on(client(t).current_user()).unwrap_err()
    };
    let auth = status(
        401,
        json!({ "errors": [ { "message": "Authentication required", "extensions": { "type": "authentication error" } } ] }),
    );
    assert!(matches!(auth, Error::Unauthorized { .. }), "{auth:?}");
    let limited = status(
        400,
        json!({ "errors": [ { "message": "Rate limit exceeded", "extensions": { "code": "RATELIMITED" } } ] }),
    );
    assert!(matches!(limited, Error::RateLimited { .. }), "{limited:?}");
    let invalid = status(
        200,
        json!({ "errors": [ { "message": "Argument Validation Error" } ], "data": null }),
    );
    match invalid {
        Error::Api { detail, .. } => assert_eq!(detail, "Argument Validation Error"),
        other => panic!("{other:?}"),
    }
    let html = {
        let t = Fake::new(|_, _| (502, "<html>bad gateway</html>".into()));
        block_on(client(t).current_user()).unwrap_err()
    };
    assert!(matches!(html, Error::Api { status: 502, .. }), "{html:?}");
}

#[test]
fn linear_keys_are_only_references_for_the_linked_teams() {
    use crate::refs::{scan_message, IssueLinker, RefContext};
    let ctx = RefContext {
        issues: Some(IssueLinker {
            family: gitpoulpi_config::ApiFamily::Linear,
            web_base: "https://linear.app/acme".into(),
            project: "ENG".into(),
            project_keys: vec!["OPS".into()],
            cards: Vec::new(),
        }),
        pulls: None,
    };
    let found: Vec<(String, Option<String>)> =
        scan_message("Fixes ENG-12 and OPS-3, not UTF-8 or SHA-256", &ctx)
            .into_iter()
            .map(|f| (f.reference, f.url))
            .collect();
    assert_eq!(
        found,
        vec![
            (
                "ENG-12".to_string(),
                Some("https://linear.app/acme/issue/ENG-12".to_string())
            ),
            (
                "OPS-3".to_string(),
                Some("https://linear.app/acme/issue/OPS-3".to_string())
            ),
        ]
    );
}

#[test]
fn linear_rate_limit_headers_are_read() {
    use crate::conditional::rate_observation;
    let mut headers = std::collections::BTreeMap::new();
    headers.insert(
        "x-ratelimit-requests-remaining".to_string(),
        "12".to_string(),
    );
    headers.insert("x-ratelimit-requests-reset".to_string(), "5000".to_string());
    let response = HttpResponse {
        status: 200,
        headers,
        body: String::new(),
    };
    let o = rate_observation(gitpoulpi_config::ApiFamily::Linear, &response, 4000).unwrap();
    assert_eq!((o.remaining, o.reset_at), (12, 5000));
    let throttled = HttpResponse {
        status: 429,
        headers: [("retry-after".to_string(), "30".to_string())].into(),
        body: String::new(),
    };
    let o = rate_observation(gitpoulpi_config::ApiFamily::Linear, &throttled, 4000).unwrap();
    assert_eq!((o.remaining, o.reset_at), (0, 4030));
}

/// Needs a Linear workspace and a personal API key:
/// `GITPOULPI_LIVE_LINEAR_TOKEN=lin_api_... cargo test -p gitpoulpi-hosting
/// --features linear live_linear -- --ignored`.
#[test]
#[ignore = "live: set GITPOULPI_LIVE_TESTS=1 and run with --ignored"]
fn live_linear_reads() {
    if !gitpoulpi_testkit::live::enabled("live_linear_reads") {
        return;
    }
    let token = std::env::var("GITPOULPI_LIVE_LINEAR_TOKEN").expect("GITPOULPI_LIVE_LINEAR_TOKEN");
    let transport = Arc::new(crate::http::ReqwestTransport::new().unwrap());
    let i = Integration::new("live", IntegrationKind::Linear, "https://linear.app");
    let c = client_for(i, token, transport);
    let check = block_on(c.check_token()).expect("viewer");
    eprintln!("viewer: {:?}", check.user);
    let teams = block_on(c.list_projects()).expect("teams");
    eprintln!(
        "teams: {:?}",
        teams.iter().map(|t| &t.full_name).collect::<Vec<_>>()
    );
    let team = teams.first().expect("a team").full_name.clone();
    let issues = block_on(c.list_issues(&team, &IssueFilter::default())).expect("issues");
    eprintln!("{} open issues in {team}", issues.len());
    eprintln!(
        "statuses: {:?}",
        block_on(c.issue_statuses(&team)).expect("states")
    );
    eprintln!(
        "members: {}",
        block_on(c.project_members(&team, ""))
            .expect("members")
            .len()
    );
    eprintln!(
        "labels: {}",
        block_on(c.project_labels(&team)).expect("labels").len()
    );
    eprintln!(
        "cycles: {}",
        block_on(c.project_milestones(&team)).expect("cycles").len()
    );
}
