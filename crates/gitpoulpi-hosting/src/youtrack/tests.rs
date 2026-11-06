use std::sync::Arc;

use serde_json::{json, Value};

use crate::client::{client_for, HostingClient, NewIssue};
use crate::http::{HttpRequest, Method, RecordedTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::metadata::{IssueEdit, MetaKind};
use crate::model::{IssueFilter, IssueState, StateFilter};

use super::query::{epoch_ms_to_iso, issues_query};

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn client(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    let i = Integration::new(
        "yt",
        IntegrationKind::Youtrack,
        "https://acme.youtrack.cloud/",
    );
    client_for(
        i,
        "perm:token".into(),
        t,
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap()
}

fn body(r: &HttpRequest) -> Value {
    serde_json::from_str(r.body.as_deref().unwrap()).unwrap()
}

fn issue(n: u32, resolved: bool) -> Value {
    json!({
        "id": format!("2-{n}"), "idReadable": format!("ABC-{n}"), "summary": format!("Issue {n}"),
        "resolved": if resolved { json!(1760000100000u64) } else { Value::Null },
        "updated": 1760000000000u64,
        "reporter": { "login": "bob", "fullName": "Bob" },
        "tags": [ { "name": "ui" } ],
        "customFields": [
            { "name": "Type", "value": { "name": "Bug" } },
            { "name": "State", "value": { "name": if resolved { "Done" } else { "In Progress" } } },
            { "name": "Assignee", "value": { "login": "ann", "fullName": "Ann Lee" } },
            { "name": "Priority", "value": { "name": "Major" } }
        ]
    })
}

const PROJECTS: &str = r#"[
  {"id":"0-1","shortName":"ABC","name":"Alphabet","description":"Letters","archived":false},
  {"id":"0-2","shortName":"OLD","name":"Old","archived":true}]"#;

const FIELDS: &str = r#"[
  {"field":{"name":"State"},"bundle":{"values":[
     {"name":"Submitted","isResolved":false},{"name":"In Progress","isResolved":false},
     {"name":"Done","isResolved":true}]}},
  {"field":{"name":"Fix versions"},"bundle":{"values":[
     {"name":"1.0","released":true},{"name":"1.1","releaseDate":1767225600000},
     {"name":"1.2"},{"name":"0.9","archived":true}]}},
  {"field":{"name":"Assignee"}}]"#;

#[test]
fn the_query_is_exact_for_every_filter_combination() {
    let f = |state, mine, query: Option<&str>| IssueFilter {
        state,
        assigned_to_me: mine,
        query: query.map(String::from),
        ..Default::default()
    };
    assert_eq!(
        issues_query("ABC", &f(StateFilter::Open, false, None)),
        "project: {ABC} #Unresolved sort by: updated desc"
    );
    assert_eq!(
        issues_query(
            "ABC",
            &f(StateFilter::Closed, true, Some("Type: Bug or Type: Task"))
        ),
        "project: {ABC} #Resolved for: me and (Type: Bug or Type: Task) sort by: updated desc",
        "the filter's `or` stays inside its parentheses"
    );
    assert_eq!(
        issues_query("ABC", &f(StateFilter::All, false, Some("  "))),
        "project: {ABC} sort by: updated desc"
    );
    assert_eq!(
        issues_query(
            "ABC",
            &f(
                StateFilter::All,
                false,
                Some("tag: ui sort by: created asc")
            )
        ),
        "project: {ABC} and (tag: ui) sort by: created asc",
        "their sort replaces ours"
    );
    assert_eq!(
        issues_query(
            "ABC",
            &f(StateFilter::All, false, Some("summary: {sort by: me}"))
        ),
        "project: {ABC} and (summary: {sort by: me}) sort by: updated desc",
        "a `sort by:` inside braces is part of the value"
    );
}

#[test]
fn dates_come_from_epoch_milliseconds() {
    assert_eq!(epoch_ms_to_iso(1_760_000_000_000), "2025-10-09T08:53:20Z");
    assert_eq!(epoch_ms_to_iso(0), "");
}

#[test]
fn the_token_is_a_bearer_and_users_me_is_the_check() {
    let t = Arc::new(RecordedTransport::new().on(
        "/api/users/me",
        200,
        r#"{"id":"1-1","login":"ann","fullName":"Ann Lee","email":"a@x"}"#,
    ));
    let check = block_on(client(t.clone()).check_token()).unwrap();
    let req = &t.requests()[0];
    assert!(
        req.url
            .starts_with("https://acme.youtrack.cloud/api/users/me?fields="),
        "{}",
        req.url
    );
    let auth = req.headers.iter().find(|(k, _)| k == "Authorization");
    assert_eq!(auth.map(|(_, v)| v.as_str()), Some("Bearer perm:token"));
    assert_eq!(check.user.login, "ann");
    assert_eq!(check.user.name, "Ann Lee");
    assert!(check.scopes.is_none());
}

#[test]
fn a_rejected_token_is_unauthorized() {
    let t = Arc::new(RecordedTransport::new().on(
        "/api/users/me",
        401,
        r#"{"error":"Unauthorized","error_description":"Invalid token"}"#,
    ));
    let err = block_on(client(t).check_token()).unwrap_err();
    assert!(matches!(err, crate::Error::Unauthorized { .. }), "{err:?}");
}

#[test]
fn projects_are_listed_by_short_name_and_archived_ones_are_hidden() {
    let t = Arc::new(RecordedTransport::new().on("/api/admin/projects", 200, PROJECTS));
    let projects = block_on(client(t.clone()).list_projects()).unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].full_name, "ABC");
    assert_eq!(projects[0].name, "Alphabet");
    assert_eq!(
        projects[0].web_url.as_deref(),
        Some("https://acme.youtrack.cloud/projects/ABC")
    );
    assert!(t.requests()[0].url.contains("$top=100&$skip=0"));
}

#[test]
fn issues_are_listed_with_fields_and_the_query_and_paged_by_skip() {
    let page: Vec<Value> = (1..=100).map(|n| issue(n, false)).collect();
    let t = Arc::new(
        RecordedTransport::new()
            .on("$skip=0", 200, &Value::Array(page).to_string())
            .on(
                "$skip=100",
                200,
                &Value::Array(vec![issue(101, true)]).to_string(),
            ),
    );
    let filter = IssueFilter {
        state: StateFilter::All,
        ..Default::default()
    };
    let rows = block_on(client(t.clone()).list_issues("ABC", &filter)).unwrap();
    assert_eq!(rows.len(), 101);
    let first = &rows[0];
    assert_eq!(first.reference, "ABC-1");
    assert_eq!(first.number, 1);
    assert_eq!(first.id, "2-1");
    assert_eq!(first.title, "Issue 1");
    assert_eq!(first.state, IssueState::Open);
    assert_eq!(first.status_label.as_deref(), Some("In Progress"));
    assert_eq!(first.kind_label.as_deref(), Some("Bug"));
    assert_eq!(first.assignees, vec!["ann"]);
    assert_eq!(first.author, "bob");
    assert_eq!(first.labels, vec!["ui"]);
    assert_eq!(first.updated_at, "2025-10-09T08:53:20Z");
    assert_eq!(first.web_url, "https://acme.youtrack.cloud/issue/ABC-1");
    assert_eq!(rows[100].state, IssueState::Closed, "resolved is closed");

    let urls: Vec<String> = t.requests().iter().map(|r| r.url.clone()).collect();
    assert_eq!(urls.len(), 2);
    assert!(urls[0].starts_with("https://acme.youtrack.cloud/api/issues?fields=id,idReadable,"));
    assert!(
        urls[0].contains("&query=project%3A%20%7BABC%7D%20sort%20by%3A%20updated%20desc&"),
        "{}",
        urls[0]
    );
}

fn create_transport() -> Arc<RecordedTransport> {
    Arc::new(
        RecordedTransport::new()
            .on("/admin/projects/0-1/customFields", 200, FIELDS)
            .on("/admin/projects/0-1/team/users", 200, r#"[{"id":"1-1","login":"ann","fullName":"Ann Lee"},{"id":"1-2","login":"bob","fullName":"Bob"}]"#)
            .on("/api/admin/projects", 200, PROJECTS)
            .on("/api/tags", 200, r##"[{"id":"6-1","name":"UI","color":{"background":"#ff0000"}},{"id":"6-2","name":"bug"}]"##)
            .on("/api/issues?fields=id,idReadable", 200, r#"{"id":"2-9","idReadable":"ABC-9"}"#),
    )
}

#[test]
fn creating_an_issue_sends_the_project_id_state_assignee_version_and_tags() {
    let t = create_transport();
    let created = block_on(client(t.clone()).create_issue(&NewIssue {
        project: "ABC".into(),
        title: "Crash".into(),
        body: "It crashes".into(),
        labels: vec!["ui".into()],
        assignees: vec!["ann".into()],
        // The second open, unreleased, unarchived version: "1.2".
        milestone: Some(2),
        list_id: Some("Submitted".into()),
        ..Default::default()
    }))
    .unwrap();
    assert_eq!(created.reference, "ABC-9");
    assert_eq!(created.number, 9);
    assert_eq!(created.web_url, "https://acme.youtrack.cloud/issue/ABC-9");
    let req = t
        .requests()
        .into_iter()
        .find(|r| r.method == Method::Post)
        .unwrap();
    assert_eq!(
        body(&req),
        json!({
            "project": { "id": "0-1" }, "summary": "Crash", "description": "It crashes",
            "customFields": [
                { "name": "State", "$type": "StateIssueCustomField", "value": { "name": "Submitted" } },
                { "name": "Assignee", "$type": "SingleUserIssueCustomField", "value": { "login": "ann" } },
                { "name": "Fix versions", "$type": "MultiVersionIssueCustomField", "value": [ { "name": "1.2" } ] }
            ],
            "tags": [ { "id": "6-1" } ]
        })
    );
}

#[test]
fn the_pickers_see_members_tags_and_unreleased_versions() {
    let t = create_transport();
    let c = client(t);
    let members = block_on(c.project_members("ABC", "ann")).unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].login, "ann");
    assert_eq!(members[0].name, "Ann Lee");
    let labels = block_on(c.project_labels("ABC")).unwrap();
    assert_eq!(labels[0].color, "ff0000");
    let versions = block_on(c.project_milestones("ABC")).unwrap();
    let shown: Vec<(u64, &str)> = versions.iter().map(|v| (v.id, v.title.as_str())).collect();
    assert_eq!(shown, vec![(1, "1.1"), (2, "1.2")]);
    assert_eq!(versions[0].due_on.as_deref(), Some("2026-01-01"));
}

#[test]
fn the_state_bundle_is_the_board_and_the_create_choices() {
    let t = create_transport();
    let c = client(t);
    assert_eq!(
        block_on(c.issue_statuses("ABC")).unwrap(),
        vec!["Submitted", "In Progress", "Done"]
    );
    let options = block_on(c.issue_create_options("ABC")).unwrap();
    let lists: Vec<&str> = options.lists.iter().map(|l| l.label.as_str()).collect();
    assert_eq!(lists, vec!["Submitted", "In Progress"]);
    assert_eq!(options.default_list.as_deref(), Some("Submitted"));
}

#[test]
fn moving_and_closing_post_the_state_field() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/admin/projects/0-1/customFields", 200, FIELDS)
            .on("/api/admin/projects", 200, PROJECTS)
            .on("/api/issues/ABC-5", 200, r#"{"id":"2-5"}"#),
    );
    let c = client(t.clone());
    block_on(c.set_issue_status("ABC", 5, "in progress")).unwrap();
    block_on(c.set_issue_state("ABC", 5, true)).unwrap();
    block_on(c.set_issue_state("ABC", 5, false)).unwrap();
    let posts: Vec<HttpRequest> = t
        .requests()
        .into_iter()
        .filter(|r| r.method == Method::Post)
        .collect();
    assert_eq!(posts.len(), 3);
    assert!(posts[0].url.ends_with("/api/issues/ABC-5?fields=id"));
    let state = |r: &HttpRequest| body(r)["customFields"][0]["value"]["name"].clone();
    assert_eq!(state(&posts[0]), json!("In Progress"));
    assert_eq!(state(&posts[1]), json!("Done"));
    assert_eq!(state(&posts[2]), json!("Submitted"));
    let err = block_on(c.set_issue_status("ABC", 5, "Nowhere")).unwrap_err();
    assert!(err.to_string().contains("Nowhere"), "{err}");
}

#[test]
fn editing_clears_the_assignee_swaps_the_version_and_diffs_the_tags() {
    let meta = json!({
        "id": "2-5", "idReadable": "ABC-5", "tags": [ { "name": "bug" } ],
        "customFields": [ { "name": "Assignee", "value": { "login": "ann" } },
                          { "name": "Fix versions", "value": [ { "name": "1.1" } ] } ]
    });
    let t = Arc::new(
        RecordedTransport::new()
            .on("/admin/projects/0-1/customFields", 200, FIELDS)
            .on("/api/admin/projects", 200, PROJECTS)
            .on(
                "/api/tags",
                200,
                r#"[{"id":"6-1","name":"UI"},{"id":"6-2","name":"bug"}]"#,
            )
            .on("/api/issues/ABC-5/tags", 200, r#"{"id":"6-1"}"#)
            .on("fields=id,idReadable,tags", 200, &meta.to_string())
            .on("/api/issues/ABC-5", 200, r#"{"id":"2-5"}"#),
    );
    let c = client(t.clone());
    block_on(c.update_issue(
        "ABC",
        5,
        &IssueEdit {
            assignees: Some(vec![]),
            labels: Some(vec!["ui".into()]),
            milestone: Some(Some(2)),
        },
    ))
    .unwrap();
    let reqs = t.requests();
    let field_post = reqs
        .iter()
        .find(|r| r.method == Method::Post && r.url.contains("/issues/ABC-5?fields=id"))
        .unwrap();
    assert_eq!(
        body(field_post),
        json!({ "customFields": [
            { "name": "Assignee", "$type": "SingleUserIssueCustomField", "value": null },
            { "name": "Fix versions", "$type": "MultiVersionIssueCustomField", "value": [ { "name": "1.2" } ] }
        ] })
    );
    let add = reqs
        .iter()
        .find(|r| r.method == Method::Post && r.url.contains("/tags"))
        .unwrap();
    assert_eq!(body(add), json!({ "id": "6-1" }));
    let remove = reqs.iter().find(|r| r.method == Method::Delete).unwrap();
    assert!(
        remove.url.ends_with("/api/issues/ABC-5/tags/6-2"),
        "{}",
        remove.url
    );
}

#[test]
fn an_issues_metadata_names_its_version_by_position() {
    let meta = json!({
        "id": "2-5", "idReadable": "ABC-5", "tags": [ { "name": "bug" } ],
        "customFields": [ { "name": "Assignee", "value": { "login": "ann" } },
                          { "name": "Fix versions", "value": [ { "name": "1.2" } ] } ]
    });
    let t = Arc::new(
        RecordedTransport::new()
            .on("/admin/projects/0-1/customFields", 200, FIELDS)
            .on("/api/admin/projects", 200, PROJECTS)
            .on("/api/issues/ABC-5", 200, &meta.to_string()),
    );
    let m = block_on(client(t).item_meta("ABC", 5, MetaKind::Issue)).unwrap();
    assert_eq!(m.assignees, vec!["ann"]);
    assert_eq!(m.labels, vec!["bug"]);
    let ms = m.milestone.unwrap();
    assert_eq!((ms.id, ms.title.as_str()), (2, "1.2"));
}

#[test]
fn youtrack_keys_are_only_references_for_the_linked_projects() {
    use crate::refs::{from_branch, scan_message, IssueLinker, RefContext};
    let ctx = RefContext {
        issues: Some(IssueLinker {
            family: gitpoulpi_config::ApiFamily::YouTrack,
            web_base: "https://acme.youtrack.cloud".into(),
            project: "ABC".into(),
            project_keys: Vec::new(),
            cards: Vec::new(),
        }),
        pulls: None,
    };
    let found: Vec<String> = scan_message("Fix ABC-12, not UTF-8 or XYZ-3", &ctx)
        .into_iter()
        .map(|f| f.url.unwrap())
        .collect();
    assert_eq!(found, vec!["https://acme.youtrack.cloud/issue/ABC-12"]);
    assert_eq!(
        from_branch("feature/ABC-7-fix", &ctx).map(|f| f.reference),
        Some("ABC-7".to_string())
    );
}

/// Needs a YouTrack and a permanent token: `GITPOULPI_LIVE_YOUTRACK_URL=https://...
/// GITPOULPI_LIVE_YOUTRACK_TOKEN=perm:... cargo test -p gitpoulpi-hosting --features
/// youtrack live_youtrack -- --ignored`.
#[test]
#[ignore = "live: set GITPOULPI_LIVE_TESTS=1 and run with --ignored"]
fn live_youtrack_reads() {
    if !gitpoulpi_testkit::live::enabled("live_youtrack_reads") {
        return;
    }
    let url = std::env::var("GITPOULPI_LIVE_YOUTRACK_URL").expect("GITPOULPI_LIVE_YOUTRACK_URL");
    let token =
        std::env::var("GITPOULPI_LIVE_YOUTRACK_TOKEN").expect("GITPOULPI_LIVE_YOUTRACK_TOKEN");
    let transport = Arc::new(crate::http::ReqwestTransport::new().unwrap());
    let c = client_for(
        Integration::new("live", IntegrationKind::Youtrack, &url),
        token,
        transport,
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    eprintln!(
        "user: {:?}",
        block_on(c.check_token()).expect("users/me").user
    );
    let projects = block_on(c.list_projects()).expect("projects");
    eprintln!(
        "projects: {:?}",
        projects.iter().map(|p| &p.full_name).collect::<Vec<_>>()
    );
    let p = projects.first().expect("a project").full_name.clone();
    let issues = block_on(c.list_issues(&p, &IssueFilter::default())).expect("issues");
    eprintln!("{} open issues in {p}", issues.len());
    eprintln!(
        "states: {:?}",
        block_on(c.issue_statuses(&p)).expect("states")
    );
    eprintln!(
        "members: {}",
        block_on(c.project_members(&p, "")).expect("members").len()
    );
    eprintln!(
        "versions: {}",
        block_on(c.project_milestones(&p)).expect("versions").len()
    );
    eprintln!(
        "tags: {}",
        block_on(c.project_labels(&p)).expect("tags").len()
    );
}
