//! The Gitea / Forgejo client over `RecordedTransport`: fixtures are trimmed
//! copies of what the 2026-10-14 containers answered (see `mod.rs`).
//! `RecordedTransport` answers the first registered fragment a URL
//! contains, so specific endpoints come before `pulls/3`.

use std::sync::Arc;

use serde_json::Value;

use crate::client::{client_for, HostingClient, NewIssue, NewPullRequest};
use crate::http::{HttpRequest, Method, RecordedTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::metadata::{IssueEdit, MetaKind, PrEdit};
use crate::model::{ChecksState, IssueFilter, IssueState, PrFilter, PullRequestState, StateFilter};
use crate::review::{DiffSide, FileStatus, MergeState, PendingComment, ReviewEvent};
use crate::review::{ReviewSubmission, ReviewThread};
use crate::Error;

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn gitea(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    client_for(
        Integration::new("g", IntegrationKind::Gitea, "https://git.example.com/"),
        "tok".into(),
        t,
    )
}

fn forgejo(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    client_for(
        Integration::new("c", IntegrationKind::Forgejo, "https://codeberg.org"),
        "tok".into(),
        t,
    )
}

fn body_of(r: &HttpRequest) -> Value {
    serde_json::from_str(r.body.as_deref().unwrap()).unwrap()
}

fn sent(t: &RecordedTransport, method: Method, needle: &str) -> Vec<HttpRequest> {
    t.requests()
        .into_iter()
        .filter(|r| r.method == method && r.url.contains(needle))
        .collect()
}

fn header<'a>(request: &'a HttpRequest, name: &str) -> Option<&'a str> {
    request
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

const USER: &str =
    r#"{"id":1,"login":"poulpi","full_name":"Poulpi","html_url":"https://git.example.com/poulpi"}"#;

fn pull(number: u64, title: &str, extra: &str) -> String {
    format!(
        r#"{{"number":{number},"title":"{title}","state":"open","draft":false,"merged":false,
        "merged_at":null,"html_url":"https://git.example.com/o/p/pulls/{number}",
        "updated_at":"2026-10-14T20:00:00Z","user":{{"login":"poulpi"}},
        "head":{{"ref":"feature/greeting","sha":"h1","repo":{{"full_name":"o/p"}}}},
        "base":{{"ref":"master","sha":"b1","repo":{{"full_name":"o/p"}}}},
        "assignees":null,"requested_reviewers":[{{"login":"alice"}}],"mergeable":true{extra}}}"#
    )
}

#[test]
fn requests_carry_the_token_scheme_and_live_under_api_v1() {
    let t = Arc::new(RecordedTransport::new().on("/api/v1/user", 200, USER));
    let user = block_on(gitea(t.clone()).current_user()).unwrap();
    assert_eq!(
        (user.login.as_str(), user.name.as_str()),
        ("poulpi", "Poulpi")
    );
    let request = &t.requests()[0];
    assert_eq!(request.url, "https://git.example.com/api/v1/user");
    assert_eq!(header(request, "Authorization"), Some("token tok"));
}

/// A Docker container served over plain http: the profile's scheme and
/// port reach the wire untouched (no silent upgrade to https).
#[test]
fn a_plain_http_server_is_called_over_http() {
    let t = Arc::new(RecordedTransport::new().on("/api/v1/user", 200, USER));
    let client = client_for(
        Integration::new("g", IntegrationKind::Gitea, "http://localhost:3000/"),
        "tok".into(),
        t.clone(),
    );
    block_on(client.check_token()).unwrap();
    assert_eq!(t.requests()[0].url, "http://localhost:3000/api/v1/user");
}

#[test]
fn a_token_check_reports_unknown_scopes_never_missing_ones() {
    let t = Arc::new(RecordedTransport::new().on("/user", 200, USER));
    let check = block_on(forgejo(t).check_token()).unwrap();
    assert_eq!(check.user.id, "1");
    assert_eq!(check.scopes, None);
    assert!(check.missing_scopes.is_empty());
}

#[test]
fn a_rejected_token_is_unauthorized() {
    let t =
        Arc::new(RecordedTransport::new().on("/user", 401, r#"{"message":"token is required"}"#));
    let err = block_on(gitea(t).check_token()).unwrap_err();
    assert!(matches!(err, Error::Unauthorized { .. }), "{err}");
}

#[test]
fn projects_normalise_and_page_on_x_hasmore() {
    let page1: String = format!(
        "[{}]",
        (0..50)
            .map(|i| format!(
                r#"{{"full_name":"o/r{i}","name":"r{i}","private":false,"clone_url":"https://git.example.com/o/r{i}.git","ssh_url":"git@git.example.com:o/r{i}.git","description":""}}"#
            ))
            .collect::<Vec<_>>()
            .join(",")
    );
    let t = Arc::new(
        RecordedTransport::new()
            .on_with_headers(
                "/user/repos?limit=50&page=1",
                200,
                &page1,
                &[("X-HasMore", "true")],
            )
            .on_with_headers(
                "/user/repos?limit=50&page=2",
                200,
                r#"[{"full_name":"o/last","name":"last","private":true,"default_branch":"master","html_url":"https://git.example.com/o/last"}]"#,
                &[("X-HasMore", "false")],
            ),
    );
    let projects = block_on(gitea(t.clone()).list_projects()).unwrap();
    assert_eq!(projects.len(), 51);
    assert_eq!(projects[0].id, "o/r0");
    assert_eq!(projects[0].description, None);
    assert_eq!(
        projects[0].clone_url(true),
        Some("git@git.example.com:o/r0.git")
    );
    assert!(projects[50].private);
    assert_eq!(t.requests().len(), 2);
}

#[test]
fn ssh_keys_are_listed_uploaded_and_deleted() {
    let t = Arc::new(RecordedTransport::new().on("/user/keys/9", 204, "").on(
        "/user/keys",
        200,
        r#"[{"id":9,"title":"laptop","key":"ssh-ed25519 AAAA"}]"#,
    ));
    let c = gitea(t.clone());
    let keys = block_on(c.list_ssh_keys()).unwrap();
    assert_eq!(
        (keys[0].id.as_str(), keys[0].title.as_str()),
        ("9", "laptop")
    );
    block_on(c.delete_ssh_key("9")).unwrap();
    assert_eq!(sent(&t, Method::Delete, "/user/keys/9").len(), 1);

    let t = Arc::new(RecordedTransport::new().on(
        "/user/keys",
        201,
        r#"{"id":10,"title":"new","key":"ssh-ed25519 BBBB"}"#,
    ));
    let key = block_on(forgejo(t.clone()).upload_ssh_key("new", " ssh-ed25519 BBBB \n")).unwrap();
    assert_eq!(key.id, "10");
    let body = body_of(&sent(&t, Method::Post, "/user/keys")[0]);
    assert_eq!(body["title"], "new");
    assert_eq!(body["key"], "ssh-ed25519 BBBB");
}

#[test]
fn pull_requests_list_with_states_reviewers_and_forks() {
    let merged = pull(2, "Done", r#","merged":true"#).replace(r#""merged":false,"#, "");
    let draft = pull(4, "WIP: later", "");
    let fork = pull(5, "From a fork", "").replace(
        r#""head":{"ref":"feature/greeting","sha":"h1","repo":{"full_name":"o/p"}}"#,
        r#""head":{"ref":"fix","sha":"h1","repo":{"full_name":"fork/p"}}"#,
    );
    let list = format!("[{},{merged},{draft},{fork}]", pull(3, "Open", ""));
    let t = Arc::new(RecordedTransport::new().on("/repos/o/p/pulls?", 200, &list));
    let c = gitea(t.clone());
    let all = block_on(c.list_pull_requests(
        "o/p",
        &PrFilter {
            state: StateFilter::All,
            ..PrFilter::default()
        },
    ))
    .unwrap();
    let states: Vec<_> = all.iter().map(|p| p.state).collect();
    assert_eq!(
        states,
        vec![
            PullRequestState::Open,
            PullRequestState::Merged,
            PullRequestState::Draft,
            PullRequestState::Open
        ]
    );
    assert_eq!(all[0].reviewers, vec!["alice"]);
    assert!(all[0].assignees.is_empty(), "a null list is empty");
    assert_eq!(all[3].source_project.as_deref(), Some("fork/p"));
    assert!(t.requests()[0].url.contains("state=all"));
}

#[test]
fn mine_and_review_requested_filter_on_the_cached_login() {
    let other = pull(7, "Not mine", "")
        .replace(r#""user":{"login":"poulpi"}"#, r#""user":{"login":"bob"}"#);
    let list = format!("[{},{other}]", pull(3, "Mine", ""));
    let t = Arc::new(
        RecordedTransport::new()
            .on("/user", 200, USER)
            .on("/pulls?", 200, &list),
    );
    let c = gitea(t.clone());
    let mine = block_on(c.list_pull_requests(
        "o/p",
        &PrFilter {
            mine: true,
            ..PrFilter::default()
        },
    ))
    .unwrap();
    assert_eq!(mine.iter().map(|p| p.number).collect::<Vec<_>>(), vec![3]);

    // alice is the requested reviewer of both; the login comes from /user.
    let t = Arc::new(
        RecordedTransport::new()
            .on("/user", 200, r#"{"id":2,"login":"alice"}"#)
            .on("/pulls?", 200, &list),
    );
    let asked = block_on(gitea(t).list_pull_requests(
        "o/p",
        &PrFilter {
            review_requested: true,
            ..PrFilter::default()
        },
    ))
    .unwrap();
    assert_eq!(asked.len(), 2);
    assert!(asked.iter().all(|p| p.review_requested));
}

#[test]
fn checks_and_conflicts_are_filled_on_request() {
    let conflicted = pull(8, "Conflict", "").replace(r#""mergeable":true"#, r#""mergeable":false"#);
    let list = format!("[{},{conflicted}]", pull(3, "Ok", ""));
    let t = Arc::new(
        RecordedTransport::new()
            .on("/user", 200, USER)
            .on(
                "/commits/h1/status",
                200,
                r#"{"state":"failure","total_count":2}"#,
            )
            .on("/pulls?", 200, &list),
    );
    let rows = block_on(gitea(t).list_pull_requests(
        "o/p",
        &PrFilter {
            checks: true,
            ..PrFilter::default()
        },
    ))
    .unwrap();
    assert_eq!(rows[0].checks, Some(ChecksState::Failing));
    assert_eq!(rows[0].has_conflicts, Some(false));
    assert_eq!(rows[1].has_conflicts, Some(true));
}

#[test]
fn a_commit_with_no_statuses_has_no_checks() {
    let t = Arc::new(RecordedTransport::new().on(
        "/commits/h1/status",
        200,
        r#"{"state":"pending","total_count":0,"statuses":null}"#,
    ));
    let c = GiteaClientProbe::new(t);
    assert_eq!(block_on(c.0.commit_checks("o/p", "h1")).unwrap(), None);
}

/// The concrete client, for the `pub(crate)` helpers.
struct GiteaClientProbe(super::GiteaClient);

impl GiteaClientProbe {
    fn new(t: Arc<RecordedTransport>) -> Self {
        Self(super::GiteaClient::new(
            Integration::new("g", IntegrationKind::Gitea, "https://git.example.com"),
            "tok".into(),
            t,
        ))
    }
}

#[test]
fn the_pull_request_of_a_branch_prefers_open_over_merged() {
    let merged = pull(2, "Old", r#","merged":true"#).replace(r#""merged":false,"#, "");
    let other = pull(9, "Other branch", "").replace("feature/greeting", "elsewhere");
    let list = format!("[{merged},{},{other}]", pull(3, "Open", ""));
    let t = Arc::new(RecordedTransport::new().on("/pulls?", 200, &list));
    let best = block_on(gitea(t).pull_request_for_branch("o/p", "feature/greeting"))
        .unwrap()
        .unwrap();
    assert_eq!(best.number, 3);
}

#[test]
fn creating_a_pull_request_resolves_label_names_and_asks_reviewers_after() {
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "/repos/o/p/labels",
                200,
                r#"[{"id":11,"name":"bug"},{"id":12,"name":"review-me"}]"#,
            )
            .on("/requested_reviewers", 201, "[]")
            .on(
                "/repos/o/p/pulls",
                201,
                r#"{"number":4,"html_url":"https://git.example.com/o/p/pulls/4"}"#,
            ),
    );
    let created = block_on(gitea(t.clone()).create_pull_request(&NewPullRequest {
        project: "o/p".into(),
        source_branch: "feat".into(),
        target_branch: "master".into(),
        title: "Add".into(),
        body: "text".into(),
        draft: true,
        assignees: vec!["alice".into()],
        reviewers: vec!["bob".into()],
        labels: vec!["BUG".into()],
        milestone: Some(1),
        ..NewPullRequest::default()
    }))
    .unwrap();
    assert_eq!(created.number, 4);
    assert_eq!(created.warning, None);
    let body = body_of(&sent(&t, Method::Post, "/repos/o/p/pulls")[0]);
    assert_eq!(body["title"], "WIP: Add", "the draft is the WIP prefix");
    assert_eq!(body["head"], "feat");
    assert_eq!(body["base"], "master");
    assert_eq!(body["labels"], serde_json::json!([11]));
    assert_eq!(body["assignees"], serde_json::json!(["alice"]));
    assert_eq!(body["milestone"], 1);
    let reviewers = body_of(&sent(&t, Method::Post, "/pulls/4/requested_reviewers")[0]);
    assert_eq!(reviewers["reviewers"], serde_json::json!(["bob"]));
}

#[test]
fn a_failed_reviewer_request_is_a_warning_not_an_error() {
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "/requested_reviewers",
                422,
                r#"{"message":"cannot review own pull"}"#,
            )
            .on(
                "/repos/o/p/pulls",
                201,
                r#"{"number":4,"html_url":"https://git.example.com/o/p/pulls/4"}"#,
            ),
    );
    let created = block_on(forgejo(t.clone()).create_pull_request(&NewPullRequest {
        project: "o/p".into(),
        source_branch: "feat".into(),
        target_branch: "master".into(),
        title: "Add".into(),
        reviewers: vec!["me".into()],
        source_project: Some("fork/p".into()),
        ..NewPullRequest::default()
    }))
    .unwrap();
    assert!(created.warning.unwrap().contains("reviewers"));
    assert_eq!(
        body_of(&sent(&t, Method::Post, "/repos/o/p/pulls")[0])["head"],
        "fork:feat"
    );
}

#[test]
fn an_unknown_label_is_refused_before_anything_is_created() {
    let t = Arc::new(RecordedTransport::new().on("/labels", 200, r#"[{"id":1,"name":"bug"}]"#));
    let err = block_on(gitea(t.clone()).create_issue(&NewIssue {
        project: "o/p".into(),
        title: "t".into(),
        labels: vec!["nope".into()],
        ..NewIssue::default()
    }))
    .unwrap_err();
    assert!(err.to_string().contains("nope"), "{err}");
    assert!(sent(&t, Method::Post, "/issues").is_empty());
}

#[test]
fn issues_list_without_pull_requests_and_create() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/user", 200, USER)
            .on(
                "/repos/o/p/issues?",
                200,
                r#"[{"number":2,"title":"Crash","state":"open","html_url":"https://git.example.com/o/p/issues/2",
                    "updated_at":"2026-10-14T20:52:54Z","user":{"login":"poulpi"},
                    "assignees":[{"login":"poulpi"}],"labels":[{"name":"bug"}],"pull_request":null},
                   {"number":3,"title":"A PR","state":"closed","html_url":"x","pull_request":{"merged":false}},
                   {"number":1,"title":"Done","state":"closed","html_url":"y","assignees":null,"labels":null}]"#,
            ),
    );
    let c = gitea(t.clone());
    let issues = block_on(c.list_issues(
        "o/p",
        &IssueFilter {
            state: StateFilter::All,
            assigned_to_me: true,
            ..IssueFilter::default()
        },
    ))
    .unwrap();
    assert_eq!(
        issues.iter().map(|i| i.number).collect::<Vec<_>>(),
        vec![2, 1]
    );
    assert_eq!(issues[0].reference, "#2");
    assert_eq!(issues[0].labels, vec!["bug"]);
    assert_eq!(issues[1].state, IssueState::Closed);
    let url = t.requests().last().unwrap().url.clone();
    assert!(
        url.contains("type=issues")
            && url.contains("state=all")
            && url.contains("assigned_by=poulpi"),
        "{url}"
    );

    let t = Arc::new(RecordedTransport::new().on(
        "/repos/o/p/issues",
        201,
        r#"{"number":5,"html_url":"https://git.example.com/o/p/issues/5"}"#,
    ));
    let created = block_on(c_for(t.clone()).create_issue(&NewIssue {
        project: "o/p".into(),
        title: "New".into(),
        body: "b".into(),
        assignees: vec!["alice".into()],
        milestone: Some(2),
        ..NewIssue::default()
    }))
    .unwrap();
    assert_eq!((created.number, created.reference.as_str()), (5, "#5"));
    let body = body_of(&sent(&t, Method::Post, "/repos/o/p/issues")[0]);
    assert_eq!(body["assignees"], serde_json::json!(["alice"]));
    assert_eq!(body["milestone"], 2);
    assert!(body.get("labels").is_none());
}

fn c_for(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    gitea(t)
}

#[test]
fn a_429_is_a_rate_limit() {
    let t = Arc::new(RecordedTransport::new().on_with_headers(
        "/user",
        429,
        "",
        &[("Retry-After", "30")],
    ));
    let err = block_on(gitea(t).check_token()).unwrap_err();
    assert!(matches!(err, Error::RateLimited { .. }), "{err}");
}

// ---- slice M ----

#[test]
fn members_labels_and_milestones() {
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "/repos/o/p/assignees",
                200,
                r#"[{"id":2,"login":"alice","full_name":"Alice A","html_url":"https://git.example.com/alice"},
                    {"id":3,"login":"bob"}]"#,
            )
            .on(
                "/repos/o/p/labels",
                200,
                r#"[{"id":1,"name":"bug","color":"d73a4a","description":"Broken"}]"#,
            )
            .on(
                "/repos/o/p/milestones?state=open",
                200,
                r#"[{"id":2,"title":"v0.2","state":"open","due_on":null},
                    {"id":1,"title":"v0.1","state":"open","due_on":"2026-12-01T12:00:00Z"}]"#,
            ),
    );
    let c = gitea(t.clone());
    let people = block_on(c.project_members("o/p", "ALI")).unwrap();
    assert_eq!(people.len(), 1);
    assert_eq!(people[0].name, "Alice A");
    let labels = block_on(c.project_labels("o/p")).unwrap();
    assert_eq!(
        (labels[0].name.as_str(), labels[0].color.as_str()),
        ("bug", "d73a4a")
    );
    let milestones = block_on(c.project_milestones("o/p")).unwrap();
    assert_eq!(
        milestones
            .iter()
            .map(|m| m.title.as_str())
            .collect::<Vec<_>>(),
        vec!["v0.1", "v0.2"],
        "dated first"
    );
    assert_eq!(milestones[0].id, 1);
    assert_eq!(milestones[0].due_on.as_deref(), Some("2026-12-01"));
}

#[test]
fn item_meta_reads_a_pull_request_and_an_issue() {
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "/pulls/3",
                200,
                r#"{"assignees":null,"requested_reviewers":[{"login":"alice"}],"labels":[{"id":1,"name":"bug"}],
                    "milestone":{"id":1,"title":"v0.1","state":"open"}}"#,
            )
            .on(
                "/issues/1",
                200,
                r#"{"assignees":[{"login":"bob"}],"labels":[],"milestone":null}"#,
            ),
    );
    let c = gitea(t.clone());
    let pr = block_on(c.item_meta("o/p", 3, MetaKind::PullRequest)).unwrap();
    assert_eq!(pr.reviewers, vec!["alice"]);
    assert_eq!(pr.labels, vec!["bug"]);
    assert_eq!(pr.milestone.unwrap().id, 1);
    let issue = block_on(c.item_meta("o/p", 1, MetaKind::Issue)).unwrap();
    assert_eq!(issue.assignees, vec!["bob"]);
    assert_eq!(issue.milestone, None);
}

#[test]
fn an_issue_edit_patches_assignees_and_milestone_and_puts_label_names() {
    let t = Arc::new(RecordedTransport::new().on("/repos/o/p/issues/1", 200, "{}"));
    let c = gitea(t.clone());
    block_on(c.update_issue(
        "o/p",
        1,
        &IssueEdit {
            assignees: Some(vec!["alice".into()]),
            labels: Some(vec!["bug".into()]),
            milestone: Some(Some(2)),
        },
    ))
    .unwrap();
    let patch = body_of(&sent(&t, Method::Patch, "/issues/1")[0]);
    assert_eq!(
        patch,
        serde_json::json!({"assignees":["alice"],"milestone":2})
    );
    let put = &sent(&t, Method::Put, "/issues/1/labels")[0];
    assert_eq!(body_of(put), serde_json::json!({"labels":["bug"]}));

    // Clearing: an empty assignee list and milestone 0; no label call when
    // the labels are left alone.
    let t = Arc::new(RecordedTransport::new().on("/issues/1", 200, "{}"));
    block_on(gitea(t.clone()).update_issue(
        "o/p",
        1,
        &IssueEdit {
            assignees: Some(vec![]),
            labels: None,
            milestone: Some(None),
        },
    ))
    .unwrap();
    assert_eq!(
        body_of(&sent(&t, Method::Patch, "/issues/1")[0]),
        serde_json::json!({"assignees":[],"milestone":0})
    );
    assert!(sent(&t, Method::Put, "/labels").is_empty());
}

#[test]
fn a_pull_request_edit_diffs_the_reviewers() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/requested_reviewers", 204, "")
            .on(
                "/repos/o/p/pulls/3",
                200,
                r#"{"requested_reviewers":[{"login":"alice"},{"login":"bob"}]}"#,
            )
            .on("/repos/o/p/issues/3", 201, "{}"),
    );
    block_on(gitea(t.clone()).update_pull_request(
        "o/p",
        3,
        &PrEdit {
            reviewers: Some(vec!["bob".into(), "carol".into()]),
            labels: Some(vec![]),
            ..PrEdit::default()
        },
    ))
    .unwrap();
    let add = body_of(&sent(&t, Method::Post, "/requested_reviewers")[0]);
    assert_eq!(add["reviewers"], serde_json::json!(["carol"]));
    let remove = body_of(&sent(&t, Method::Delete, "/requested_reviewers")[0]);
    assert_eq!(remove["reviewers"], serde_json::json!(["alice"]));
    assert_eq!(
        body_of(&sent(&t, Method::Put, "/issues/3/labels")[0]),
        serde_json::json!({"labels":[]})
    );
}

// ---- slice R1 ----

#[test]
fn capabilities_differ_by_kind_on_resolution_only() {
    let t = Arc::new(RecordedTransport::new());
    let g = gitea(t.clone()).review_capabilities();
    let f = forgejo(t).review_capabilities();
    assert!(g.review && g.approve && g.request_changes && !g.line_ranges);
    assert!(g.resolve);
    assert!(f.review && f.approve && f.request_changes && !f.line_ranges);
    assert!(!f.resolve, "Forgejo 11 has no resolve endpoint");
}

#[test]
fn the_detail_reads_head_base_merge_state_and_checks() {
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "/commits/h1/status",
                200,
                r#"{"state":"success","total_count":1}"#,
            )
            .on(
                "/repos/o/p/pulls/3",
                200,
                &pull(
                    3,
                    "Greet",
                    r#","body":"Adds a name.","created_at":"2026-10-14T20:52:59Z","additions":7,"deletions":1,"changed_files":2"#,
                ),
            ),
    );
    let d = block_on(gitea(t).pull_request_detail("o/p", 3)).unwrap();
    assert_eq!((d.head_sha.as_str(), d.base_sha.as_str()), ("h1", "b1"));
    assert_eq!(d.body, "Adds a name.");
    assert_eq!(d.merge_state, MergeState::Clean);
    assert_eq!(d.checks, Some(ChecksState::Passing));
    assert_eq!(
        (d.additions, d.deletions, d.changed_files),
        (Some(7), Some(1), Some(2))
    );
    assert_eq!(d.summary.number, 3);
    assert!(d.capabilities.resolve);
}

#[test]
fn a_conflicting_pull_request_says_so() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/commits/", 404, r#"{"message":"no"}"#)
            .on(
                "/repos/o/p/pulls/3",
                200,
                &pull(3, "x", "").replace(r#""mergeable":true"#, r#""mergeable":false"#),
            ),
    );
    let d = block_on(forgejo(t).pull_request_detail("o/p", 3)).unwrap();
    assert_eq!(d.merge_state, MergeState::Conflicting);
    assert_eq!(
        d.checks, None,
        "an unreadable status is no badge, not an error"
    );
}

#[test]
fn a_draft_is_blocked_never_conflicting() {
    let t = Arc::new(RecordedTransport::new().on("/commits/", 404, "{}").on(
        "/repos/o/p/pulls/3",
        200,
        &pull(3, "WIP: later", "").replace(r#""mergeable":true"#, r#""mergeable":false"#),
    ));
    let d = block_on(gitea(t).pull_request_detail("o/p", 3)).unwrap();
    assert_eq!(d.merge_state, MergeState::Blocked);
}

#[test]
fn files_map_statuses_and_renames() {
    let t = Arc::new(RecordedTransport::new().on(
        "/pulls/3/files",
        200,
        r#"[{"filename":"NOTES.md","status":"added","additions":3,"deletions":0},
            {"filename":"main.rs","status":"changed","additions":4,"deletions":1},
            {"filename":"b.rs","status":"renamed","previous_filename":"a.rs","additions":0,"deletions":0},
            {"filename":"gone.rs","status":"deleted","additions":0,"deletions":9}]"#,
    ));
    let files = block_on(gitea(t).pull_request_files("o/p", 3)).unwrap();
    let kinds: Vec<_> = files.iter().map(|f| f.status).collect();
    assert_eq!(
        kinds,
        vec![
            FileStatus::Added,
            FileStatus::Modified,
            FileStatus::Renamed,
            FileStatus::Removed
        ]
    );
    assert_eq!(files[2].previous_path.as_deref(), Some("a.rs"));
    assert_eq!((files[1].additions, files[1].deletions), (4, 1));
}

fn comment_json(
    id: u64,
    review: u64,
    position: u32,
    original: u32,
    resolver: bool,
    commit: &str,
    body: &str,
) -> String {
    let resolver = if resolver {
        r#"{"login":"poulpi"}"#
    } else {
        "null"
    };
    format!(
        r#"{{"id":{id},"body":"{body}","user":{{"login":"poulpi"}},"resolver":{resolver},
        "pull_request_review_id":{review},"created_at":"2026-10-14T20:55:59Z","path":"main.rs",
        "commit_id":"{commit}","position":{position},"original_position":{original},
        "html_url":"https://git.example.com/o/p/pulls/3#issuecomment-{id}"}}"#
    )
}

fn thread_fixture() -> Arc<RecordedTransport> {
    // review 1 is alice's pending request (no comments), review 2 holds
    // the new-line comment 4, the old-line comment 5 and the reply 7.
    Arc::new(
        RecordedTransport::new()
            .on(
                "/pulls/3/reviews/2/comments",
                200,
                &format!(
                    "[{},{},{}]",
                    comment_json(5, 2, 0, 2, false, "h1", "old side"),
                    comment_json(4, 2, 2, 0, true, "h1", "on the new line"),
                    comment_json(7, 2, 2, 0, false, "h1", "a reply"),
                ),
            )
            .on(
                "/pulls/3/reviews/3/comments",
                200,
                &format!("[{}]", comment_json(9, 3, 5, 0, false, "older", "stale")),
            )
            .on(
                "/pulls/3/reviews?",
                200,
                r#"[{"id":1,"state":"REQUEST_REVIEW","comments_count":0},
                    {"id":2,"state":"COMMENT","comments_count":3},
                    {"id":3,"state":"COMMENT","comments_count":1}]"#,
            )
            .on("/repos/o/p/pulls/3", 200, &pull(3, "Greet", "")),
    )
}

#[test]
fn threads_group_comments_by_line_and_side() {
    let threads = block_on(gitea(thread_fixture()).pull_request_threads("o/p", 3)).unwrap();
    assert_eq!(threads.len(), 3);
    assert!(threads.iter().all(|t| t.resolvable));
    let on_forgejo = block_on(forgejo(thread_fixture()).pull_request_threads("o/p", 3)).unwrap();
    assert!(
        on_forgejo.iter().all(|t| !t.resolvable),
        "no resolve endpoint"
    );
    let new_line: &ReviewThread = &threads[0];
    assert_eq!(new_line.id, "4|n|2|main.rs");
    assert_eq!(new_line.anchor(), Some((DiffSide::New, 2)));
    assert!(new_line.resolved, "the root carries the resolver");
    assert!(!new_line.outdated);
    assert_eq!(
        new_line
            .comments
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        vec!["4", "7"],
        "root first, then the reply"
    );
    let old_line = &threads[1];
    assert_eq!(old_line.id, "5|o|2|main.rs");
    assert_eq!(old_line.anchor(), Some((DiffSide::Old, 2)));
    assert!(!old_line.resolved);
    let stale = &threads[2];
    assert!(stale.outdated, "written on another commit than the head");
    assert_eq!(stale.anchor(), None);
    assert_eq!(stale.original_line, Some(5));
}

#[test]
fn no_commented_review_means_no_threads_and_no_head_lookup() {
    let t = Arc::new(RecordedTransport::new().on(
        "/pulls/3/reviews?",
        200,
        r#"[{"id":1,"state":"REQUEST_REVIEW","comments_count":0}]"#,
    ));
    let threads = block_on(gitea(t.clone()).pull_request_threads("o/p", 3)).unwrap();
    assert!(threads.is_empty());
    assert_eq!(t.requests().len(), 1);
}

#[test]
fn submitted_reviews_carry_verdicts_and_skip_requests_and_bare_comment_reviews() {
    let t = Arc::new(RecordedTransport::new().on(
        "/pulls/3/reviews?",
        200,
        r#"[
        {"id":2,"state":"COMMENT","body":"","comments_count":4,"user":{"login":"poulpi"}},
        {"id":4,"state":"COMMENT","body":"looks ok","user":{"login":"poulpi"},"submitted_at":"2026-10-14T20:55:59Z"},
        {"id":5,"state":"REQUEST_CHANGES","body":"no","dismissed":true,"user":{"login":"alice"}},
        {"id":6,"state":"APPROVED","body":"","user":{"login":"alice"}},
        {"id":7,"state":"REQUEST_REVIEW","body":"","user":{"login":"bob"}},
        {"id":8,"state":"REQUEST_CHANGES","body":"fix","user":{"login":"bob"}}]"#,
    ));
    let rows = block_on(gitea(t).pull_request_reviews("o/p", 3)).unwrap();
    let got: Vec<_> = rows
        .iter()
        .map(|r| (r.id.as_str(), r.author.as_str(), r.state))
        .collect();
    assert_eq!(
        got,
        vec![
            ("4", "poulpi", crate::review::ReviewState::Commented),
            ("5", "alice", crate::review::ReviewState::Dismissed),
            ("6", "alice", crate::review::ReviewState::Approved),
            ("8", "bob", crate::review::ReviewState::ChangesRequested),
        ]
    );
}

#[test]
fn the_conversation_is_the_issue_comments() {
    let t = Arc::new(RecordedTransport::new().on(
        "/issues/3/comments",
        200,
        r#"[{"id":1,"body":"hello","user":{"login":"alice"},"created_at":"2026-10-14T21:00:00Z","html_url":"https://git.example.com/o/p/pulls/3#issuecomment-1"}]"#,
    ));
    let c = gitea(t.clone());
    let rows = block_on(c.pull_request_conversation("o/p", 3)).unwrap();
    assert_eq!(
        (rows[0].author.as_str(), rows[0].body.as_str()),
        ("alice", "hello")
    );
    let t = Arc::new(RecordedTransport::new().on(
        "/issues/3/comments",
        201,
        r#"{"id":2,"body":"hi","user":{"login":"poulpi"},"created_at":"x"}"#,
    ));
    let posted = block_on(gitea(t.clone()).add_comment("o/p", 3, "hi")).unwrap();
    assert_eq!(posted.id, "2");
    assert_eq!(
        body_of(&sent(&t, Method::Post, "/issues/3/comments")[0]),
        serde_json::json!({"body":"hi"})
    );
    let err = block_on(gitea(Arc::new(RecordedTransport::new())).add_comment("o/p", 3, "  "))
        .unwrap_err();
    assert!(matches!(err, Error::Api { status: 422, .. }));
}

#[test]
fn a_reply_uses_the_reply_endpoint_of_the_root_comment() {
    let t = Arc::new(RecordedTransport::new().on(
        "/pulls/3/comments/4/replies",
        201,
        &comment_json(8, 2, 2, 0, false, "h1", "thanks"),
    ));
    let reply =
        block_on(gitea(t.clone()).reply_to_thread("o/p", 3, "4|n|2|main.rs", "thanks")).unwrap();
    assert_eq!(reply.id, "8");
    assert_eq!(sent(&t, Method::Post, "/comments/4/replies").len(), 1);
}

#[test]
fn a_reply_falls_back_to_a_one_comment_review_without_the_endpoint() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/comments/4/replies", 405, "")
            .on(
                "/pulls/3/reviews/11/comments",
                200,
                &format!("[{}]", comment_json(12, 11, 2, 0, false, "h1", "thanks")),
            )
            .on("/pulls/3/reviews", 200, r#"{"id":11,"state":"COMMENT"}"#),
    );
    let reply =
        block_on(forgejo(t.clone()).reply_to_thread("o/p", 3, "4|n|2|main.rs", "thanks")).unwrap();
    assert_eq!(reply.id, "12");
    let review = body_of(&sent(&t, Method::Post, "/pulls/3/reviews")[0]);
    assert_eq!(review["event"], "COMMENT");
    assert_eq!(
        review["comments"],
        serde_json::json!([{"path":"main.rs","body":"thanks","new_position":2}])
    );

    // An old-side thread replies on the old side.
    let t = Arc::new(
        RecordedTransport::new()
            .on("/comments/5/replies", 404, "")
            .on(
                "/pulls/3/reviews/11/comments",
                200,
                &format!("[{}]", comment_json(13, 11, 0, 2, false, "h1", "x")),
            )
            .on("/pulls/3/reviews", 200, r#"{"id":11}"#),
    );
    block_on(forgejo(t.clone()).reply_to_thread("o/p", 3, "5|o|2|main.rs", "x")).unwrap();
    let review = body_of(&sent(&t, Method::Post, "/pulls/3/reviews")[0]);
    assert_eq!(review["comments"][0]["old_position"], 2);
    assert!(review["comments"][0].get("new_position").is_none());
}

#[test]
fn resolving_posts_to_the_root_and_a_missing_endpoint_is_unsupported() {
    let t = Arc::new(RecordedTransport::new().on("/pulls/comments/4/", 204, ""));
    let c = gitea(t.clone());
    block_on(c.resolve_thread("o/p", 3, "4|n|2|main.rs", true)).unwrap();
    block_on(c.resolve_thread("o/p", 3, "4|n|2|main.rs", false)).unwrap();
    assert_eq!(sent(&t, Method::Post, "/pulls/comments/4/resolve").len(), 1);
    assert_eq!(
        sent(&t, Method::Post, "/pulls/comments/4/unresolve").len(),
        1
    );

    let t = Arc::new(RecordedTransport::new().on("/resolve", 405, ""));
    let err = block_on(forgejo(t).resolve_thread("o/p", 3, "4|n|2|main.rs", true)).unwrap_err();
    assert!(matches!(err, Error::Unsupported(_)), "{err}");
    let err = block_on(
        gitea(Arc::new(RecordedTransport::new())).resolve_thread("o/p", 3, "garbage", true),
    )
    .unwrap_err();
    assert!(matches!(err, Error::Api { status: 422, .. }));
}

fn line_comment(path: &str, line: u32, side: DiffSide, body: &str) -> PendingComment {
    PendingComment {
        path: path.into(),
        previous_path: None,
        line,
        side,
        start_line: None,
        other_line: None,
        body: body.into(),
    }
}

#[test]
fn a_review_is_one_post_with_events_positions_and_the_head() {
    for (event, wire) in [
        (ReviewEvent::Approve, "APPROVED"),
        (ReviewEvent::RequestChanges, "REQUEST_CHANGES"),
        (ReviewEvent::Comment, "COMMENT"),
    ] {
        let t = Arc::new(RecordedTransport::new().on("/pulls/3/reviews", 200, r#"{"id":21}"#));
        let outcome = block_on(gitea(t.clone()).submit_review(
            "o/p",
            3,
            &ReviewSubmission {
                votes: Vec::new(),
                event,
                body: "see inline".into(),
                comments: vec![
                    line_comment("main.rs", 2, DiffSide::New, "new"),
                    line_comment("main.rs", 5, DiffSide::Old, "old"),
                ],
                head_sha: Some("h1".into()),
            },
        ))
        .unwrap();
        assert_eq!(outcome.comments, 2);
        assert_eq!(outcome.review_id.as_deref(), Some("21"));
        let posts = sent(&t, Method::Post, "/pulls/3/reviews");
        assert_eq!(posts.len(), 1);
        let body = body_of(&posts[0]);
        assert_eq!(body["event"], wire);
        assert_eq!(body["commit_id"], "h1");
        assert_eq!(body["body"], "see inline");
        assert_eq!(
            body["comments"],
            serde_json::json!([
                {"path":"main.rs","body":"new","new_position":2},
                {"path":"main.rs","body":"old","old_position":5}
            ])
        );
    }
}

#[test]
fn an_empty_verdict_that_needs_words_is_refused_before_any_request() {
    let t = Arc::new(RecordedTransport::new());
    let err = block_on(gitea(t.clone()).submit_review(
        "o/p",
        3,
        &ReviewSubmission {
            votes: Vec::new(),
            event: ReviewEvent::RequestChanges,
            body: " ".into(),
            comments: vec![],
            head_sha: None,
        },
    ))
    .unwrap_err();
    assert!(matches!(err, Error::Api { status: 422, .. }), "{err}");
    assert!(t.requests().is_empty());

    // An approval needs no words.
    let t = Arc::new(RecordedTransport::new().on("/reviews", 200, r#"{"id":1}"#));
    block_on(forgejo(t).submit_review(
        "o/p",
        3,
        &ReviewSubmission {
            votes: Vec::new(),
            event: ReviewEvent::Approve,
            body: String::new(),
            comments: vec![],
            head_sha: None,
        },
    ))
    .unwrap();
}

#[test]
fn the_server_refusing_an_own_approval_surfaces_its_message() {
    let t = Arc::new(RecordedTransport::new().on(
        "/reviews",
        422,
        r#"{"message":"approve your own pull is not allowed"}"#,
    ));
    let err = block_on(gitea(t).submit_review(
        "o/p",
        3,
        &ReviewSubmission {
            votes: Vec::new(),
            event: ReviewEvent::Approve,
            body: String::new(),
            comments: vec![],
            head_sha: None,
        },
    ))
    .unwrap_err();
    assert!(err.to_string().contains("own pull"), "{err}");
}

// ---- spec and wiring ----

#[test]
fn urls_remote_and_head_ref_come_from_the_spec() {
    use gitpoulpi_config::ApiFamily;
    let spec = crate::spec_for(ApiFamily::Gitea);
    assert_eq!((spec.tag, spec.pr_sigil), ("GT", '#'));
    assert_eq!(
        (spec.pr_web_url)("https://codeberg.org", "o/p", 3).as_deref(),
        Some("https://codeberg.org/o/p/pulls/3")
    );
    assert_eq!(
        (spec.issue_web_url)("https://codeberg.org", "o/p", "7").as_deref(),
        Some("https://codeberg.org/o/p/issues/7")
    );
    assert_eq!(
        (spec.new_pr_url)("https://codeberg.org", "o/p", "feat", Some("main")).as_deref(),
        Some("https://codeberg.org/o/p/compare/main...feat")
    );
    assert_eq!(
        (spec.new_issue_url)("https://codeberg.org", "o/p", "A b", "text", &[]).as_deref(),
        Some("https://codeberg.org/o/p/issues/new?title=A%20b&body=text")
    );
    assert_eq!(
        (spec.branch_url)("https://codeberg.org", "o/p", "main").as_deref(),
        Some("https://codeberg.org/o/p/src/branch/main")
    );
    assert_eq!(
        crate::pull_request_head_ref(IntegrationKind::Forgejo, 3),
        Some(("refs/pull/3/head".to_string(), "pr/3".to_string()))
    );
    assert_eq!(
        crate::review_head_refspec(IntegrationKind::Gitea, 3).as_deref(),
        Some("+refs/pull/3/head:refs/gitpoulpi/pr/3")
    );
    assert_eq!(
        crate::well_known_kind("https://codeberg.org"),
        Some(IntegrationKind::Forgejo)
    );
    assert_eq!(crate::well_known_kind("https://git.example.com"), None);
}

#[test]
fn a_remote_of_any_spelling_finds_the_profile() {
    let profiles = vec![Integration::new(
        "cb",
        IntegrationKind::Forgejo,
        "https://codeberg.org",
    )];
    for url in [
        "https://codeberg.org/o/p.git",
        "git@codeberg.org:o/p.git",
        "ssh://git@codeberg.org:2222/o/p.git",
    ] {
        let remotes = [crate::ResolveRemote::new("origin", url)];
        let found = crate::resolve_for_repo(&remotes, &profiles, None).unwrap();
        assert_eq!(
            (found.integration.id.as_str(), found.project_path.as_str()),
            ("cb", "o/p"),
            "{url}"
        );
    }
}

/// Read-only calls against a real server: `cargo test -p gitpoulpi-hosting
/// -- --ignored live_gitea`, with `GITPOULPI_LIVE_GITEA` (the instance's
/// URL), `GITPOULPI_LIVE_GITEA_TOKEN` and optionally
/// `GITPOULPI_LIVE_GITEA_PROJECT` (default `poulpi/gitpoulpi-sandbox`) and
/// `GITPOULPI_LIVE_GITEA_PR` (default 3). Never part of the gate.
#[test]
#[ignore = "live: set GITPOULPI_LIVE_TESTS=1 and run with --ignored"]
fn live_gitea_reads() {
    if !gitpoulpi_testkit::live::enabled("live_gitea_reads") {
        return;
    }
    let (Ok(url), Ok(token)) = (
        std::env::var("GITPOULPI_LIVE_GITEA"),
        std::env::var("GITPOULPI_LIVE_GITEA_TOKEN"),
    ) else {
        panic!("set GITPOULPI_LIVE_GITEA and GITPOULPI_LIVE_GITEA_TOKEN");
    };
    let project = std::env::var("GITPOULPI_LIVE_GITEA_PROJECT")
        .unwrap_or_else(|_| "poulpi/gitpoulpi-sandbox".to_string());
    let number: u64 = std::env::var("GITPOULPI_LIVE_GITEA_PR")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(3);
    let transport = Arc::new(crate::ReqwestTransport::new().expect("transport"));
    let client = client_for(
        Integration::new("live", IntegrationKind::Gitea, url),
        token,
        transport,
    );
    block_on(async {
        let check = client.check_token().await.expect("token");
        assert!(!check.user.login.is_empty());
        assert!(!client.list_projects().await.expect("projects").is_empty());
        let prs = client
            .list_pull_requests(
                &project,
                &PrFilter {
                    state: StateFilter::All,
                    ..PrFilter::default()
                },
            )
            .await
            .expect("pull requests");
        assert!(prs.iter().any(|p| p.number == number), "{prs:?}");
        let detail = client
            .pull_request_detail(&project, number)
            .await
            .expect("detail");
        assert!(!detail.head_sha.is_empty());
        client
            .pull_request_files(&project, number)
            .await
            .expect("files");
        client
            .pull_request_threads(&project, number)
            .await
            .expect("threads");
        client.project_labels(&project).await.expect("labels");
        client
            .project_milestones(&project)
            .await
            .expect("milestones");
        client.project_members(&project, "").await.expect("members");
    });
}
