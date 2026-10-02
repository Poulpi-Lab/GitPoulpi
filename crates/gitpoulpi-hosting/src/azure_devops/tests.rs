//! Canned-response tests: exact headers, URLs and bodies per call. No
//! network, no real token (`tok-azure-123` must appear in no URL and no
//! error text).

use std::sync::Arc;

use gitpoulpi_config::{ApiFamily, Integration, IntegrationKind};

use super::AzureDevopsClient;
use crate::client::{HostingClient, NewIssue, NewPullRequest};
use crate::http::{HttpRequest, Method, RecordedTransport};
use crate::model::{IssueFilter, IssueState, PrFilter, PullRequestState, StateFilter};
use crate::spec::{spec_for, CheckoutSource};
use crate::Error;

const TOKEN: &str = "tok-azure-123";
const BASIC: &str = "Basic OnRvay1henVyZS0xMjM=";
const ORG: &str = "https://dev.azure.com/acme";

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn services() -> Integration {
    Integration::new("az", IntegrationKind::AzureDevops, ORG)
}

fn server() -> Integration {
    Integration::new(
        "tfs",
        IntegrationKind::AzureDevopsServer,
        "https://tfs.corp/tfs/DefaultCollection",
    )
}

fn client(integration: Integration, transport: &Arc<RecordedTransport>) -> AzureDevopsClient {
    AzureDevopsClient::new(integration, TOKEN.to_string(), transport.clone())
}

fn header<'a>(request: &'a HttpRequest, name: &str) -> Option<&'a str> {
    request
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

fn assert_no_token(transport: &RecordedTransport) {
    for r in transport.requests() {
        assert!(!r.url.contains(TOKEN), "token in url {}", r.url);
    }
}

const REPO: &str = r#"{"id":"r-1","name":"Repo One","project":{"name":"My Project"}}"#;

// ---- remotes ----------------------------------------------------------

fn parse(remote: &str) -> Option<(String, String)> {
    (spec_for(ApiFamily::AzureDevops).parse_remote)(remote).map(|r| (r.web_base, r.project))
}

#[test]
fn every_remote_spelling_parses_to_org_root_and_project_repo() {
    let acme = |p: &str| Some((ORG.to_string(), p.to_string()));
    for (remote, expected) in [
        ("https://acme@dev.azure.com/acme/P/_git/R", acme("P/R")),
        ("https://dev.azure.com/acme/P/_git/R", acme("P/R")),
        ("https://dev.azure.com/acme/_git/R", acme("R/R")),
        (
            "https://acme@dev.azure.com/acme/My%20Project/_git/Repo%20One",
            acme("My Project/Repo One"),
        ),
        ("https://acme.visualstudio.com/P/_git/R", acme("P/R")),
        (
            "https://acme.visualstudio.com/DefaultCollection/P/_git/R",
            acme("P/R"),
        ),
        ("git@ssh.dev.azure.com:v3/acme/P/R", acme("P/R")),
        ("acme@vs-ssh.visualstudio.com:v3/acme/P/R", acme("P/R")),
        (
            "ssh://git@ssh.dev.azure.com/v3/acme/My%20Project/R",
            acme("My Project/R"),
        ),
        (
            "https://tfs.corp/tfs/DefaultCollection/P/_git/R",
            Some((
                "https://tfs.corp/tfs/DefaultCollection".to_string(),
                "P/R".to_string(),
            )),
        ),
        (
            "ssh://tfs.corp:22/tfs/DefaultCollection/P/_git/R",
            Some((
                "https://tfs.corp/tfs/DefaultCollection".to_string(),
                "P/R".to_string(),
            )),
        ),
        (
            "http://tfs.corp:8080/tfs/Coll/P/_git/R",
            Some((
                "http://tfs.corp:8080/tfs/Coll".to_string(),
                "P/R".to_string(),
            )),
        ),
        (
            "https://tfs.corp/tfs/Coll/_git/R",
            Some(("https://tfs.corp/tfs/Coll".to_string(), "R/R".to_string())),
        ),
    ] {
        assert_eq!(parse(remote), expected, "{remote}");
    }
    for bad in [
        "",
        "not a url",
        "https://dev.azure.com/acme",
        "git@github.com:a/b.git",
    ] {
        assert_eq!(parse(bad), None, "{bad}");
    }
}

#[test]
fn two_orgs_on_one_host_are_two_profiles() {
    let owns = |integration: &Integration, remote: &str| {
        let spec = spec_for(ApiFamily::AzureDevops);
        let remote = (spec.parse_remote)(remote).unwrap();
        (spec.owns_remote)(integration, &remote)
    };
    let acme = services();
    let other = Integration::new(
        "o",
        IntegrationKind::AzureDevops,
        "https://dev.azure.com/other",
    );
    let legacy = Integration::new(
        "l",
        IntegrationKind::AzureDevops,
        "https://acme.visualstudio.com",
    );
    for remote in [
        "https://acme@dev.azure.com/acme/P/_git/R",
        "git@ssh.dev.azure.com:v3/acme/P/R",
        "https://acme.visualstudio.com/P/_git/R",
    ] {
        assert!(owns(&acme, remote), "{remote}");
        assert!(owns(&legacy, remote), "{remote}");
        assert!(!owns(&other, remote), "{remote}");
    }
    let tfs = server();
    assert!(owns(
        &tfs,
        "ssh://tfs.corp:22/tfs/DefaultCollection/P/_git/R"
    ));
    assert!(owns(
        &tfs,
        "https://TFS.corp/tfs/defaultcollection/P/_git/R"
    ));
    assert!(!owns(&tfs, "https://tfs.corp/tfs/Other/P/_git/R"));
    // No organization typed yet: owns nothing.
    let bare = Integration::new("b", IntegrationKind::AzureDevops, "");
    assert!(!owns(&bare, "https://dev.azure.com/acme/P/_git/R"));
}

#[test]
fn the_resolver_tells_two_orgs_apart() {
    use crate::{resolve_for_repo, ResolveRemote};
    let integrations = vec![
        Integration::new(
            "other",
            IntegrationKind::AzureDevops,
            "https://dev.azure.com/other",
        ),
        services(),
    ];
    let remotes = [ResolveRemote::new(
        "origin",
        "https://acme@dev.azure.com/acme/P/_git/R",
    )];
    let found = resolve_for_repo(&remotes, &integrations, None).unwrap();
    assert_eq!(found.integration.id, "az");
    assert_eq!(found.project_path, "P/R");
    assert!(!found.ambiguous);
}

// ---- spec -------------------------------------------------------------

#[test]
fn urls_follow_the_azure_shapes_and_encode_names() {
    let s = spec_for(ApiFamily::AzureDevops);
    assert_eq!(s.tag, "AZ");
    assert_eq!(s.pr_sigil, '!');
    assert_eq!(
        (s.pr_web_url)(ORG, "My Project/Repo", 12).as_deref(),
        Some("https://dev.azure.com/acme/My%20Project/_git/Repo/pullrequest/12")
    );
    assert_eq!(
        (s.new_pr_url)(ORG, "P/R", "feature/x", Some("main")).as_deref(),
        Some("https://dev.azure.com/acme/P/_git/R/pullrequestcreate?sourceRef=feature%2Fx&targetRef=main")
    );
    for reference in ["7", "#7", "AB#7"] {
        assert_eq!(
            (s.issue_web_url)(ORG, "P", reference).as_deref(),
            Some("https://dev.azure.com/acme/P/_workitems/edit/7"),
            "{reference}"
        );
    }
    assert_eq!((s.issue_web_url)(ORG, "P", "ABC-1"), None);
    assert_eq!(
        (s.branch_url)(ORG, "P/R", "a/b").as_deref(),
        Some("https://dev.azure.com/acme/P/_git/R?version=GBa%2Fb")
    );
    assert_eq!(
        (s.commit_url)(ORG, "P/R", "abc").as_deref(),
        Some("https://dev.azure.com/acme/P/_git/R/commit/abc")
    );
    assert_eq!(
        (s.token_page)(&services()).as_deref(),
        Some("https://dev.azure.com/acme/_usersSettings/tokens")
    );
    let pr = crate::model::PullRequestSummary {
        number: 3,
        iid_label: "!3".into(),
        title: String::new(),
        state: PullRequestState::Open,
        source_branch: "feat".into(),
        target_branch: "main".into(),
        source_project: None,
        author: String::new(),
        web_url: String::new(),
        updated_at: String::new(),
        review_requested: false,
        assignees: vec![],
        reviewers: vec![],
        checks: None,
        has_conflicts: None,
    };
    assert_eq!(
        (s.checkout)(&pr),
        CheckoutSource::Branch {
            fork_url: None,
            branch: "feat".into(),
            local: "pr/3".into()
        }
    );
}

#[test]
fn azure_hosts_are_well_known() {
    for host in [
        "https://dev.azure.com",
        "https://dev.azure.com/acme",
        "https://ssh.dev.azure.com",
        "https://acme.visualstudio.com",
        "https://vs-ssh.visualstudio.com",
    ] {
        assert_eq!(
            crate::well_known_kind(host),
            Some(IntegrationKind::AzureDevops),
            "{host}"
        );
    }
    assert_eq!(crate::well_known_kind("https://tfs.corp"), None);
    assert_eq!(
        crate::well_known_kind("https://github.com"),
        Some(IntegrationKind::Github)
    );
}

// ---- token and user ---------------------------------------------------

#[test]
fn check_token_reads_connection_data_with_a_basic_header() {
    let t = Arc::new(RecordedTransport::new().on(
        "/acme/_apis/connectionData",
        200,
        r#"{"authenticatedUser":{"id":"u-1","providerDisplayName":"Jane Doe",
            "properties":{"Account":{"$type":"System.String","$value":"jane@acme.test"}}}}"#,
    ));
    let check = block_on(client(services(), &t).check_token()).unwrap();
    assert_eq!(check.user.id, "u-1");
    assert_eq!(check.user.name, "Jane Doe");
    assert_eq!(check.user.login, "jane@acme.test");
    assert!(check.scopes.is_none() && check.missing_scopes.is_empty());
    let sent = &t.requests()[0];
    assert_eq!(sent.method, Method::Get);
    assert_eq!(sent.url, "https://dev.azure.com/acme/_apis/connectionData");
    assert_eq!(header(sent, "Authorization"), Some(BASIC));
    assert_no_token(&t);
}

#[test]
fn check_token_falls_back_to_a_plain_access_check() {
    let t = Arc::new(RecordedTransport::new().on("connectionData", 404, "{}").on(
        "/_apis/projects",
        200,
        r#"{"count":0,"value":[]}"#,
    ));
    let c = client(services(), &t);
    let check = block_on(c.check_token()).unwrap();
    assert_eq!(check.user.id, "");
    assert_eq!(
        t.requests()[1].url,
        "https://dev.azure.com/acme/_apis/projects?$top=1&api-version=7.1"
    );
    // The user is unknown: "mine" cannot be answered.
    assert!(matches!(
        block_on(c.list_pull_requests(
            "P/R",
            &PrFilter {
                mine: true,
                ..PrFilter::default()
            }
        )),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        block_on(c.current_user()),
        Err(Error::Unsupported(_))
    ));
}

#[test]
fn http_failures_map_to_typed_errors_without_the_token() {
    let fail = |status: u16, headers: &[(&str, &str)]| {
        let t = Arc::new(RecordedTransport::new().on_with_headers(
            "/_apis/git/repositories",
            status,
            r#"{"message":"TF400813: nope"}"#,
            headers,
        ));
        let err = block_on(client(services(), &t).list_projects()).unwrap_err();
        assert!(!err.to_string().contains(TOKEN));
        err
    };
    assert!(matches!(fail(401, &[]), Error::Unauthorized { .. }));
    assert!(matches!(
        fail(403, &[("x-ratelimit-remaining", "0")]),
        Error::Forbidden { .. }
    ));
    assert!(matches!(
        fail(
            429,
            &[("Retry-After", "30"), ("X-RateLimit-Resource", "git")]
        ),
        Error::RateLimited { .. }
    ));
    assert!(matches!(fail(500, &[]), Error::Api { status: 500, .. }));
}

#[test]
fn an_invalid_pat_is_a_203_sign_in_page_and_reads_as_unauthorized() {
    let t = Arc::new(RecordedTransport::new().on(
        "/_apis/git/repositories",
        203,
        "<html><title>Sign In</title></html>",
    ));
    let err = block_on(client(services(), &t).list_projects()).unwrap_err();
    assert!(matches!(err, Error::Unauthorized { .. }), "{err}");
    // Same page behind a 200.
    let t = Arc::new(RecordedTransport::new().on("/_apis/git/repositories", 200, "<html></html>"));
    assert!(matches!(
        block_on(client(services(), &t).list_projects()),
        Err(Error::Unauthorized { .. })
    ));
}

// ---- repositories -----------------------------------------------------

#[test]
fn list_projects_maps_org_repositories_to_clonable_rows() {
    let t = Arc::new(RecordedTransport::new().on(
        "/acme/_apis/git/repositories?api-version=7.1",
        200,
        r#"{"count":3,"value":[
          {"id":"r-1","name":"Repo One","defaultBranch":"refs/heads/main",
           "project":{"name":"My Project","visibility":"private"},
           "remoteUrl":"https://acme@dev.azure.com/acme/My%20Project/_git/Repo%20One",
           "sshUrl":"git@ssh.dev.azure.com:v3/acme/My%20Project/Repo%20One",
           "webUrl":"https://dev.azure.com/acme/My%20Project/_git/Repo%20One"},
          {"id":"r-2","name":"Gone","isDisabled":true,"project":{"name":"P"}},
          {"id":"r-3","name":"Open","project":{"name":"P","visibility":"public"},
           "remoteUrl":"https://dev.azure.com/acme/P/_git/Open"}]}"#,
    ));
    let rows = block_on(client(services(), &t).list_projects()).unwrap();
    assert_eq!(rows.len(), 2, "disabled repositories are skipped");
    let one = &rows[0];
    assert_eq!(one.full_name, "My Project/Repo One");
    assert_eq!(one.id, "r-1");
    assert_eq!(one.default_branch.as_deref(), Some("main"));
    assert_eq!(
        one.https_url.as_deref(),
        Some("https://dev.azure.com/acme/My%20Project/_git/Repo%20One"),
        "userinfo stripped"
    );
    assert_eq!(
        one.clone_url(true),
        Some("git@ssh.dev.azure.com:v3/acme/My%20Project/Repo%20One")
    );
    assert!(one.private && !rows[1].private);
    assert_eq!(
        rows[1].web_url.as_deref(),
        Some("https://dev.azure.com/acme/P/_git/Open")
    );
    assert_eq!(rows[0].integration_id, "az");
    assert_no_token(&t);
}

// ---- pull requests ----------------------------------------------------

fn pull(id: u64, status: &str, draft: bool, source: &str, creation: &str, closed: &str) -> String {
    let closed = if closed.is_empty() {
        String::new()
    } else {
        format!(r#""closedDate":"{closed}","#)
    };
    format!(
        r#"{{"pullRequestId":{id},"title":"PR {id}","status":"{status}","isDraft":{draft},
        "sourceRefName":"refs/heads/{source}","targetRefName":"refs/heads/main",
        "createdBy":{{"displayName":"Jane","uniqueName":"jane@x","id":"u-1"}},
        "creationDate":"{creation}",{closed}
        "reviewers":[{{"id":"u-2","displayName":"Bob","vote":0}},
                     {{"id":"g-1","displayName":"Team","vote":0,"isContainer":true}}],
        "repository":{{"id":"r-1","name":"Repo One","project":{{"name":"My Project"}}}}}}"#
    )
}

fn pulls_page(items: &[String]) -> String {
    format!(
        r#"{{"value":[{}],"count":{}}}"#,
        items.join(","),
        items.len()
    )
}

#[test]
fn list_pull_requests_maps_states_and_resolves_the_repo_id_first() {
    let body = pulls_page(&[
        pull(1, "active", false, "a", "2026-01-01T00:00:00Z", ""),
        pull(2, "active", true, "b", "2026-01-02T00:00:00Z", ""),
        pull(
            3,
            "completed",
            false,
            "c",
            "2026-01-03T00:00:00Z",
            "2026-01-05T00:00:00Z",
        ),
        pull(
            4,
            "abandoned",
            false,
            "d",
            "2026-01-04T00:00:00Z",
            "2026-01-06T00:00:00Z",
        ),
    ]);
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "/My%20Project/_apis/git/repositories/Repo%20One?api-version=7.1",
                200,
                REPO,
            )
            .on("/pullrequests", 200, &body),
    );
    let c = client(services(), &t);
    let all = block_on(c.list_pull_requests(
        "My Project/Repo One",
        &PrFilter {
            state: StateFilter::All,
            ..PrFilter::default()
        },
    ))
    .unwrap();
    let states: Vec<_> = all.iter().map(|p| p.state).collect();
    assert_eq!(
        states,
        [
            PullRequestState::Open,
            PullRequestState::Draft,
            PullRequestState::Merged,
            PullRequestState::Closed
        ]
    );
    assert_eq!(all[0].iid_label, "!1");
    assert_eq!(all[0].source_branch, "a");
    assert_eq!(all[0].target_branch, "main");
    assert_eq!(all[0].author, "Jane");
    assert_eq!(
        all[0].updated_at, "2026-01-01T00:00:00Z",
        "created when never closed"
    );
    assert_eq!(all[2].updated_at, "2026-01-05T00:00:00Z", "closed wins");
    assert_eq!(all[0].reviewers, ["Bob"], "groups are not reviewers");
    assert_eq!(
        all[0].web_url,
        "https://dev.azure.com/acme/My%20Project/_git/Repo%20One/pullrequest/1"
    );
    let urls: Vec<String> = t.requests().into_iter().map(|r| r.url).collect();
    assert_eq!(
        urls[1],
        "https://dev.azure.com/acme/My%20Project/_apis/git/repositories/r-1/pullrequests?searchCriteria.status=all&$top=100&$skip=0&api-version=7.1"
    );
    // The id is cached: a second call does not look the repository up again.
    block_on(c.list_pull_requests("My Project/Repo One", &PrFilter::default())).unwrap();
    assert_eq!(t.requests().len(), 3);
    assert!(t.requests()[2].url.contains("searchCriteria.status=active"));
    assert_no_token(&t);
}

#[test]
fn closed_filter_drops_live_requests() {
    let body = pulls_page(&[
        pull(1, "active", false, "a", "2026-01-01T00:00:00Z", ""),
        pull(
            3,
            "completed",
            false,
            "c",
            "2026-01-03T00:00:00Z",
            "2026-01-05T00:00:00Z",
        ),
    ]);
    let t = Arc::new(
        RecordedTransport::new()
            .on("/repositories/R?", 200, REPO)
            .on("/pullrequests", 200, &body),
    );
    let rows = block_on(client(services(), &t).list_pull_requests(
        "P/R",
        &PrFilter {
            state: StateFilter::Closed,
            ..PrFilter::default()
        },
    ))
    .unwrap();
    assert_eq!(rows.iter().map(|p| p.number).collect::<Vec<_>>(), [3]);
}

#[test]
fn mine_and_review_requested_use_the_current_users_id() {
    let body = pulls_page(&[pull(1, "active", false, "a", "2026-01-01T00:00:00Z", "")]);
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "connectionData",
                200,
                r#"{"authenticatedUser":{"id":"u-2","providerDisplayName":"Bob"}}"#,
            )
            .on("/repositories/R?", 200, REPO)
            .on("/pullrequests", 200, &body),
    );
    let rows = block_on(client(services(), &t).list_pull_requests(
        "P/R",
        &PrFilter {
            mine: true,
            review_requested: true,
            ..PrFilter::default()
        },
    ))
    .unwrap();
    let url = t.requests().pop().unwrap().url;
    assert!(url.contains("searchCriteria.creatorId=u-2"), "{url}");
    assert!(url.contains("searchCriteria.reviewerId=u-2"), "{url}");
    assert!(rows[0].review_requested, "Bob has a pending (0) vote");
}

#[test]
fn pagination_walks_full_pages_and_stops_on_a_short_one() {
    let full: Vec<String> = (1..=100)
        .map(|i| pull(i, "active", false, "x", "2026-01-01T00:00:00Z", ""))
        .collect();
    let t = Arc::new(
        RecordedTransport::new()
            .on("/repositories/R?", 200, REPO)
            .on("$skip=0&", 200, &pulls_page(&full))
            .on(
                "$skip=100&",
                200,
                &pulls_page(&[pull(101, "active", false, "x", "2026-01-01T00:00:00Z", "")]),
            ),
    );
    let rows =
        block_on(client(services(), &t).list_pull_requests("P/R", &PrFilter::default())).unwrap();
    assert_eq!(rows.len(), 101);
    assert_eq!(t.requests().len(), 3, "lookup + two pages, no third");
}

#[test]
fn pull_request_for_branch_filters_on_the_source_ref_and_picks_the_best() {
    let body = pulls_page(&[
        pull(
            5,
            "completed",
            false,
            "feat",
            "2026-01-01T00:00:00Z",
            "2026-01-02T00:00:00Z",
        ),
        pull(6, "active", false, "feat", "2026-01-03T00:00:00Z", ""),
    ]);
    let t = Arc::new(
        RecordedTransport::new()
            .on("/repositories/R?", 200, REPO)
            .on("/pullrequests", 200, &body),
    );
    let best = block_on(client(services(), &t).pull_request_for_branch("P/R", "feat"))
        .unwrap()
        .unwrap();
    assert_eq!(best.number, 6);
    let url = &t.requests()[1].url;
    assert!(
        url.contains("searchCriteria.sourceRefName=refs%2Fheads%2Ffeat")
            && url.contains("searchCriteria.status=all"),
        "{url}"
    );
}

#[test]
fn a_pull_request_from_a_fork_records_the_fork_project() {
    let body = pulls_page(&[r#"{"pullRequestId":9,"title":"t","status":"active",
        "sourceRefName":"refs/heads/f","targetRefName":"refs/heads/main",
        "createdBy":{"displayName":"J"},"creationDate":"2026-01-01T00:00:00Z",
        "forkSource":{"repository":{"id":"r-9","name":"Fork","project":{"name":"Other"}}},
        "repository":{"id":"r-1","name":"R","project":{"name":"P"}}}"#
        .to_string()]);
    let t = Arc::new(
        RecordedTransport::new()
            .on("/repositories/R?", 200, REPO)
            .on("/pullrequests", 200, &body),
    );
    let rows =
        block_on(client(services(), &t).list_pull_requests("P/R", &PrFilter::default())).unwrap();
    assert_eq!(rows[0].source_project.as_deref(), Some("Other/Fork"));
}

#[test]
fn create_pull_request_posts_full_refs_and_builds_the_browser_url() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/repositories/R?", 200, REPO)
            .on(
                "/pullrequests",
                201,
                r#"{"pullRequestId":22,"status":"active"}"#,
            ),
    );
    let created = block_on(client(services(), &t).create_pull_request(&NewPullRequest {
        project: "P/R".into(),
        source_branch: "feat/x".into(),
        target_branch: "main".into(),
        title: "A feature".into(),
        body: "Why".into(),
        draft: true,
        source_project: None,
        ..Default::default()
    }))
    .unwrap();
    assert_eq!(created.number, 22);
    assert_eq!(
        created.web_url,
        "https://dev.azure.com/acme/P/_git/R/pullrequest/22"
    );
    let post = t.requests().pop().unwrap();
    assert_eq!(post.method, Method::Post);
    assert_eq!(
        post.url,
        "https://dev.azure.com/acme/P/_apis/git/repositories/r-1/pullrequests?api-version=7.1"
    );
    assert_eq!(header(&post, "Content-Type"), Some("application/json"));
    let body: serde_json::Value = serde_json::from_str(post.body.as_deref().unwrap()).unwrap();
    assert_eq!(
        body,
        serde_json::json!({
            "sourceRefName": "refs/heads/feat/x",
            "targetRefName": "refs/heads/main",
            "title": "A feature",
            "description": "Why",
            "isDraft": true
        })
    );
}

#[test]
fn a_fork_pull_request_is_unsupported_with_a_clear_message() {
    let t = Arc::new(RecordedTransport::new());
    let err = block_on(client(services(), &t).create_pull_request(&NewPullRequest {
        project: "P/R".into(),
        source_branch: "f".into(),
        target_branch: "main".into(),
        title: "t".into(),
        body: String::new(),
        draft: false,
        source_project: Some("Other/Fork".into()),
        ..Default::default()
    }))
    .unwrap_err();
    assert!(err.to_string().contains("fork"), "{err}");
    assert!(t.requests().is_empty());
}

// ---- work items -------------------------------------------------------

fn work_item(id: u64, state: &str, assignee: &str) -> String {
    format!(
        r#"{{"id":{id},"fields":{{"System.Title":"Item {id}","System.State":"{state}",
        "System.WorkItemType":"Bug","System.TeamProject":"My Project",
        "System.ChangedDate":"2026-02-0{id}T00:00:00Z","System.Tags":"ui; urgent",
        "System.CreatedBy":{{"displayName":"Jane"}},{assignee}}}}}"#
    )
}

#[test]
fn list_issues_runs_wiql_then_batch_reads_in_wiql_order() {
    let wiql = r#"{"queryType":"flat","workItems":[{"id":3,"url":"u"},{"id":1,"url":"u"}]}"#;
    let batch = format!(
        r#"{{"count":2,"value":[{},{}]}}"#,
        work_item(1, "Active", r#""System.AssignedTo":"Bob Builder <bob@x>""#),
        work_item(
            3,
            "Closed",
            r#""System.AssignedTo":{"displayName":"Ann","uniqueName":"ann@x"}"#
        ),
    );
    let t = Arc::new(RecordedTransport::new().on("/wit/wiql", 200, wiql).on(
        "/wit/workitems",
        200,
        &batch,
    ));
    let rows = block_on(client(services(), &t).list_issues(
        "My Project",
        &IssueFilter {
            state: StateFilter::Open,
            assigned_to_me: true,
            query: Some("[System.Tags] CONTAINS 'ui'".into()),
            ..Default::default()
        },
    ))
    .unwrap();
    assert_eq!(
        rows.iter().map(|r| r.number).collect::<Vec<_>>(),
        [3, 1],
        "WIQL order"
    );
    let first = &rows[0];
    assert_eq!(first.reference, "#3");
    assert_eq!(first.iid_label, "#3");
    assert_eq!(first.id, "3");
    assert_eq!(first.state, IssueState::Closed);
    assert_eq!(first.status_label.as_deref(), Some("Closed"));
    assert_eq!(first.kind_label.as_deref(), Some("Bug"));
    assert_eq!(first.assignees, ["Ann"]);
    assert_eq!(first.labels, ["ui", "urgent"]);
    assert_eq!(first.author, "Jane");
    assert_eq!(
        first.web_url,
        "https://dev.azure.com/acme/My%20Project/_workitems/edit/3"
    );
    assert_eq!(rows[1].assignees, ["Bob Builder"]);
    assert_eq!(rows[1].state, IssueState::Open);

    let sent = t.requests();
    assert_eq!(sent[0].method, Method::Post);
    assert_eq!(
        sent[0].url,
        "https://dev.azure.com/acme/My%20Project/_apis/wit/wiql?$top=200&api-version=7.1"
    );
    let query: serde_json::Value = serde_json::from_str(sent[0].body.as_deref().unwrap()).unwrap();
    assert_eq!(
        query["query"],
        "SELECT [System.Id] FROM WorkItems WHERE [System.TeamProject] = @project \
         AND [System.State] NOT IN ('Done', 'Closed', 'Removed', 'Resolved') \
         AND [System.AssignedTo] = @Me AND ([System.Tags] CONTAINS 'ui') \
         ORDER BY [System.ChangedDate] DESC"
    );
    assert!(
        sent[1].url.starts_with(
            "https://dev.azure.com/acme/My%20Project/_apis/wit/workitems?ids=3,1&fields=System.Title,"
        ) && sent[1].url.ends_with("&api-version=7.1"),
        "{}",
        sent[1].url
    );
    assert_no_token(&t);
}

#[test]
fn list_issues_with_no_hits_skips_the_batch_read_and_all_states_add_no_clause() {
    let t = Arc::new(RecordedTransport::new().on("/wit/wiql", 200, r#"{"workItems":[]}"#));
    let rows = block_on(client(services(), &t).list_issues(
        "P",
        &IssueFilter {
            state: StateFilter::All,
            ..IssueFilter::default()
        },
    ))
    .unwrap();
    assert!(rows.is_empty());
    assert_eq!(t.requests().len(), 1);
    let body: serde_json::Value =
        serde_json::from_str(t.requests()[0].body.as_deref().unwrap()).unwrap();
    assert!(!body["query"].as_str().unwrap().contains("System.State"));
}

#[test]
fn create_issue_posts_a_json_patch_document() {
    let t = Arc::new(RecordedTransport::new().on(
        "/wit/workitems/$User%20Story",
        200,
        r#"{"id":42,"rev":1,"fields":{}}"#,
    ));
    let created = block_on(client(services(), &t).create_issue(&NewIssue {
        project: "My Project".into(),
        title: "Do <it>".into(),
        body: "line 1\nline 2 & more".into(),
        labels: vec!["a".into(), "b".into()],
        issue_type: Some("User Story".into()),
        list_id: None,
        ..Default::default()
    }))
    .unwrap();
    assert_eq!(created.number, 42);
    assert_eq!(created.reference, "#42");
    assert_eq!(
        created.web_url,
        "https://dev.azure.com/acme/My%20Project/_workitems/edit/42"
    );
    let post = &t.requests()[0];
    assert_eq!(
        post.url,
        "https://dev.azure.com/acme/My%20Project/_apis/wit/workitems/$User%20Story?api-version=7.1"
    );
    assert_eq!(
        header(post, "Content-Type"),
        Some("application/json-patch+json")
    );
    assert_eq!(
        post.headers
            .iter()
            .filter(|(k, _)| k == "Content-Type")
            .count(),
        1
    );
    let ops: serde_json::Value = serde_json::from_str(post.body.as_deref().unwrap()).unwrap();
    assert_eq!(
        ops,
        serde_json::json!([
            {"op":"add","path":"/fields/System.Title","value":"Do <it>"},
            {"op":"add","path":"/fields/System.Description","value":"line 1<br>line 2 &amp; more"},
            {"op":"add","path":"/fields/System.Tags","value":"a; b"}
        ])
    );
}

#[test]
fn create_issue_defaults_its_type_from_the_projects_options() {
    let types = r#"{"count":3,"value":[
        {"name":"Epic","isDisabled":false},{"name":"Task","isDisabled":false},
        {"name":"Bug","isDisabled":false}]}"#;
    let t = Arc::new(
        RecordedTransport::new()
            .on("/wit/workitemtypes", 200, types)
            .on("/wit/workitems/$Task", 200, r#"{"id":7}"#),
    );
    let c = client(services(), &t);
    let created = block_on(c.create_issue(&NewIssue {
        project: "P".into(),
        title: "t".into(),
        ..NewIssue::default()
    }))
    .unwrap();
    assert_eq!(created.number, 7);
    assert!(t
        .requests()
        .last()
        .unwrap()
        .url
        .contains("/workitems/$Task?"));
}

#[test]
fn issue_create_options_lists_enabled_types_with_a_default() {
    let t = Arc::new(RecordedTransport::new().on(
        "/My%20Project/_apis/wit/workitemtypes?api-version=7.1",
        200,
        r#"{"count":3,"value":[
          {"name":"Epic","referenceName":"E","isDisabled":false},
          {"name":"Issue","referenceName":"I","isDisabled":false},
          {"name":"Old","referenceName":"O","isDisabled":true}]}"#,
    ));
    let options = block_on(client(services(), &t).issue_create_options("My Project")).unwrap();
    assert_eq!(
        options
            .types
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        ["Epic", "Issue"]
    );
    assert_eq!(options.default_type.as_deref(), Some("Issue"));
    assert!(options.lists.is_empty());
}

// ---- Server -----------------------------------------------------------

#[test]
fn server_kinds_use_the_collection_root_and_api_version_7_0() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/_apis/git/repositories?", 200, r#"{"value":[]}"#)
            .on("connectionData", 200, r#"{"authenticatedUser":{"id":"u"}}"#),
    );
    let c = client(server(), &t);
    block_on(c.list_projects()).unwrap();
    block_on(c.check_token()).unwrap();
    let urls: Vec<String> = t.requests().into_iter().map(|r| r.url).collect();
    assert_eq!(
        urls[0],
        "https://tfs.corp/tfs/DefaultCollection/_apis/git/repositories?api-version=7.0"
    );
    assert_eq!(
        urls[1],
        "https://tfs.corp/tfs/DefaultCollection/_apis/connectionData"
    );
    assert_eq!(header(&t.requests()[0], "Authorization"), Some(BASIC));
}

#[test]
fn services_without_an_organization_is_a_bad_url_not_a_request() {
    let t = Arc::new(RecordedTransport::new());
    let bare = Integration::new("b", IntegrationKind::AzureDevops, "");
    let err = block_on(client(bare, &t).list_projects()).unwrap_err();
    assert!(matches!(err, Error::BadUrl(_)), "{err}");
    assert!(t.requests().is_empty());
}

#[test]
fn legacy_visualstudio_urls_are_normalized_to_the_org_root() {
    let t =
        Arc::new(RecordedTransport::new().on("/_apis/git/repositories", 200, r#"{"value":[]}"#));
    let legacy = Integration::new(
        "l",
        IntegrationKind::AzureDevops,
        "https://acme.visualstudio.com",
    );
    block_on(client(legacy, &t).list_projects()).unwrap();
    assert!(t.requests()[0]
        .url
        .starts_with("https://dev.azure.com/acme/_apis/git/repositories"));
}

#[test]
fn client_for_builds_this_client_and_ssh_keys_stay_unsupported() {
    let t = Arc::new(RecordedTransport::new());
    let boxed = crate::client_for(services(), TOKEN.to_string(), t);
    assert!(matches!(
        block_on(boxed.list_ssh_keys()),
        Err(Error::Unsupported(_))
    ));
}

// ---- the Kanban board: states and moves -----------------------------------

const TYPES: &str = r#"{"count":2,"value":[{"name":"Bug"},{"name":"Task"}]}"#;
const BUG_STATES: &str = r#"{"count":4,"value":[
  {"name":"Resolved","category":"Resolved"},
  {"name":"New","category":"Proposed"},
  {"name":"Active","category":"InProgress"},
  {"name":"Closed","category":"Completed"}]}"#;
const TASK_STATES: &str = r#"{"count":3,"value":[
  {"name":"To Do","category":"Proposed"},
  {"name":"In Progress","category":"InProgress"},
  {"name":"Done","category":"Completed"}]}"#;

fn board_item(kind: &str, state: &str) -> crate::model::IssueSummary {
    crate::model::IssueSummary {
        number: 42,
        id: "42".into(),
        reference: "#42".into(),
        kind_label: Some(kind.into()),
        status_label: Some(state.into()),
        ..Default::default()
    }
}

fn state_column(name: &str) -> crate::board::BoardColumn {
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

fn board_spec() -> crate::board::BoardSpec {
    crate::board::BoardSpec {
        source: crate::board::BoardSource::LocalStatus,
        ..Default::default()
    }
}

fn state_routes() -> RecordedTransport {
    RecordedTransport::new()
        .on("/wit/workitemtypes/Bug/states", 200, BUG_STATES)
        .on("/wit/workitemtypes/Task/states", 200, TASK_STATES)
        .on("/wit/workitemtypes?", 200, TYPES)
}

#[test]
fn the_columns_are_the_states_of_every_type_by_category_on_both_flavours() {
    for (integration, root, version) in [
        (services(), ORG, "7.1"),
        (server(), "https://tfs.corp/tfs/DefaultCollection", "7.0"),
    ] {
        let t = Arc::new(state_routes());
        let c = client(integration, &t);
        assert_eq!(c.board_source(), crate::board::BoardSource::LocalStatus);
        let names = block_on(c.issue_statuses("My Project/Repo")).unwrap();
        assert_eq!(
            names,
            [
                "New",
                "To Do",
                "Active",
                "In Progress",
                "Resolved",
                "Closed",
                "Done"
            ]
        );
        let urls: Vec<String> = t.requests().into_iter().map(|r| r.url).collect();
        assert_eq!(
            urls[0],
            format!("{root}/My%20Project/_apis/wit/workitemtypes?api-version={version}")
        );
        assert_eq!(
            urls[1],
            format!("{root}/My%20Project/_apis/wit/workitemtypes/Bug/states?api-version={version}")
        );
        assert_no_token(&t);
    }
}

#[test]
fn a_move_patches_system_state_with_a_json_patch_on_both_flavours() {
    for (integration, root, version) in [
        (services(), ORG, "7.1"),
        (server(), "https://tfs.corp/tfs/DefaultCollection", "7.0"),
    ] {
        let t = Arc::new(state_routes().on("/wit/workitems/42", 200, r#"{"id":42}"#));
        let c = client(integration, &t);
        block_on(c.move_issue(
            "Proj/Repo",
            &board_item("Task", "To Do"),
            &board_spec(),
            &state_column("in progress"),
        ))
        .unwrap();
        let patch = t.requests().pop().unwrap();
        assert_eq!(patch.method, Method::Patch);
        assert_eq!(
            patch.url,
            format!("{root}/Proj/_apis/wit/workitems/42?api-version={version}")
        );
        assert_eq!(
            header(&patch, "Content-Type"),
            Some("application/json-patch+json")
        );
        let ops: serde_json::Value = serde_json::from_str(patch.body.as_deref().unwrap()).unwrap();
        assert_eq!(
            ops,
            serde_json::json!([{"op":"add","path":"/fields/System.State","value":"In Progress"}])
        );
    }
}

#[test]
fn a_type_without_the_target_state_is_refused_before_anything_is_patched() {
    let t = Arc::new(state_routes());
    let c = client(services(), &t);
    let err = block_on(c.move_issue(
        "Proj/Repo",
        &board_item("Bug", "New"),
        &board_spec(),
        &state_column("Done"),
    ))
    .unwrap_err();
    assert_eq!(err.to_string(), "a Bug has no state Done (work item #42)");
    assert!(t.requests().iter().all(|r| r.method == Method::Get));
}

#[test]
fn moving_to_the_state_it_is_in_does_nothing() {
    let t = Arc::new(state_routes());
    let c = client(services(), &t);
    block_on(c.move_issue(
        "Proj/Repo",
        &board_item("Task", "Done"),
        &board_spec(),
        &state_column("done"),
    ))
    .unwrap();
    assert!(t.requests().is_empty());
}
