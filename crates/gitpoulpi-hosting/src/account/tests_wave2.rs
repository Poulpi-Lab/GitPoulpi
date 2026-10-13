//! Account-wide queries of Azure DevOps, Bitbucket Data Center and Gerrit
//! (and Bitbucket Cloud's refusal) over `RecordedTransport` fakes.

use std::sync::Arc;

use super::*;
use crate::client::{client_for, HostingClient};
use crate::http::{Method, RecordedTransport};
use crate::{Error, Integration, IntegrationKind};

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn client(kind: IntegrationKind, url: &str, t: &Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    let mut integration = Integration::new("acct", kind, url);
    integration.account = "mara".into();
    client_for(integration, "tok".into(), t.clone())
}

fn decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&text[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn urls(t: &RecordedTransport) -> Vec<String> {
    t.requests().iter().map(|r| decoded(&r.url)).collect()
}

fn review() -> AccountQuery {
    AccountQuery {
        review_requests: true,
        ..AccountQuery::default()
    }
}

// ----------------------------------------------------------------- Azure

const AZ: &str = "https://dev.azure.com/acme";
const ME: &str = r#"{"authenticatedUser":{"id":"u-1","providerDisplayName":"Mara"}}"#;

fn az_pull(id: u64, project: &str, draft: bool, vote: i32, date: &str) -> String {
    format!(
        r#"{{"pullRequestId":{id},"title":"t{id}","status":"active","isDraft":{draft},
        "sourceRefName":"refs/heads/a","targetRefName":"refs/heads/main",
        "createdBy":{{"displayName":"ann"}},"creationDate":"{date}",
        "reviewers":[{{"id":"u-1","displayName":"Mara","vote":{vote}}}],
        "repository":{{"id":"r{id}","name":"Repo","project":{{"name":"{project}"}},
        "remoteUrl":"https://acme@dev.azure.com/acme/{project}/_git/Repo"}}}}"#
    )
}

fn az_list(rows: &[String]) -> String {
    format!(r#"{{"count":{},"value":[{}]}}"#, rows.len(), rows.join(","))
}

#[test]
fn azure_review_requests_filter_by_reviewer_at_the_org_level() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/_apis/connectionData", 200, ME)
            .on(
                "/_apis/git/pullrequests",
                200,
                &az_list(&[
                    az_pull(5, "Proj", false, 0, "2026-10-01T10:00:00Z"),
                    // Already voted: not waiting for me any more.
                    az_pull(6, "Proj", false, 10, "2026-10-02T10:00:00Z"),
                    az_pull(7, "Proj", true, 0, "2026-10-03T10:00:00Z"),
                ]),
            ),
    );
    let c = client(IntegrationKind::AzureDevops, AZ, &t);
    let items = block_on(c.account_items(&review())).unwrap();
    let seen = urls(&t);
    assert!(
        seen.iter().any(|u| u.starts_with(
            "https://dev.azure.com/acme/_apis/git/pullrequests?searchCriteria.status=active&searchCriteria.reviewerId=u-1&$top=100&$skip=0"
        )),
        "{seen:?}"
    );
    assert_eq!(items.prs.len(), 1, "voted and draft rows are left out");
    let row = &items.prs[0];
    assert_eq!(row.project, "Proj/Repo");
    assert_eq!(row.clone_url, "https://dev.azure.com/acme/Proj/_git/Repo");
    assert_eq!(row.pr.number, 5);
    assert!(row.pr.review_requested);

    let with_drafts = AccountQuery {
        include_drafts: true,
        ..review()
    };
    let items = block_on(c.account_items(&with_drafts)).unwrap();
    assert_eq!(items.prs.len(), 2);
    assert_eq!(items.prs[0].pr.state, crate::PullRequestState::Draft);
}

#[test]
fn azure_falls_back_to_a_per_project_fan_out_when_the_org_route_is_missing() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/_apis/connectionData", 200, ME)
            .on(
                "/Alpha/_apis/git/pullrequests",
                200,
                &az_list(&[az_pull(1, "Alpha", false, 0, "2026-10-01T10:00:00Z")]),
            )
            .on("/Beta/_apis/git/pullrequests", 200, &az_list(&[]))
            .on("/_apis/git/pullrequests", 404, r#"{"message":"not found"}"#)
            .on(
                "/_apis/projects",
                200,
                r#"{"value":[{"name":"Alpha"},{"name":"Beta"}]}"#,
            ),
    );
    let c = client(IntegrationKind::AzureDevops, AZ, &t);
    let items = block_on(c.account_items(&review())).unwrap();
    let seen = urls(&t);
    assert!(seen
        .iter()
        .any(|u| u.contains("/Beta/_apis/git/pullrequests")));
    assert_eq!(items.prs.len(), 1);
    assert_eq!(items.prs[0].project, "Alpha/Repo");
}

#[test]
fn azure_open_requests_keep_the_named_projects_only() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/_apis/connectionData", 200, ME)
            .on(
                "/_apis/git/pullrequests",
                200,
                &az_list(&[
                    az_pull(1, "Alpha", false, 0, "2026-10-01T10:00:00Z"),
                    az_pull(2, "Beta", false, 0, "2026-10-02T10:00:00Z"),
                ]),
            ),
    );
    let c = client(IntegrationKind::AzureDevops, AZ, &t);
    let query = AccountQuery {
        open_requests: true,
        namespaces: vec!["beta".into()],
        ..AccountQuery::default()
    };
    let items = block_on(c.account_items(&query)).unwrap();
    assert_eq!(items.prs.len(), 1);
    assert_eq!(items.prs[0].project, "Beta/Repo");
}

#[test]
fn azure_assigned_work_items_come_from_an_org_level_wiql() {
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "/_apis/wit/wiql",
                200,
                r#"{"workItems":[{"id":9},{"id":3}]}"#,
            )
            .on(
                "/_apis/wit/workitems?ids=9,3",
                200,
                r#"{"value":[
                {"id":3,"fields":{"System.Title":"Old","System.State":"Active","System.TeamProject":"Proj","System.ChangedDate":"2026-09-01T00:00:00Z"}},
                {"id":9,"fields":{"System.Title":"Fix","System.State":"Active","System.TeamProject":"Proj","System.WorkItemType":"Bug","System.ChangedDate":"2026-10-02T00:00:00Z"}}]}"#,
            ),
    );
    let c = client(IntegrationKind::AzureDevops, AZ, &t);
    let query = AccountQuery {
        assigned_issues: true,
        ..AccountQuery::default()
    };
    let items = block_on(c.account_items(&query)).unwrap();
    let wiql = t
        .requests()
        .into_iter()
        .find(|r| r.url.contains("/_apis/wit/wiql"))
        .unwrap();
    assert_eq!(wiql.method, Method::Post);
    assert!(wiql
        .url
        .starts_with("https://dev.azure.com/acme/_apis/wit/wiql"));
    let body = wiql.body.unwrap();
    assert!(body.contains("[System.AssignedTo] = @Me"), "{body}");
    assert!(body.contains("NOT IN ("), "{body}");
    assert_eq!(items.issues.len(), 2);
    assert_eq!(items.issues[0].issue.number, 9, "newest change first");
    assert_eq!(items.issues[0].project, "Proj");
}

#[test]
fn azure_sign_in_page_fails_the_whole_call() {
    let t =
        Arc::new(RecordedTransport::new().on("/_apis/connectionData", 203, "<html>sign in</html>"));
    let c = client(IntegrationKind::AzureDevops, AZ, &t);
    let err = block_on(c.account_items(&review())).unwrap_err();
    assert!(matches!(err, Error::Unauthorized { .. }), "{err:?}");
}

// ----------------------------------------------------------- Bitbucket DC

const DC: &str = "https://git.example.com";

fn dc_pull(id: u64, key: &str, draft: bool, ms: u64) -> String {
    format!(
        r#"{{"id":{id},"title":"T{id}","state":"OPEN","draft":{draft},"updatedDate":{ms},
        "fromRef":{{"displayId":"feat","repository":{{"slug":"repo","project":{{"key":"{key}"}}}}}},
        "toRef":{{"displayId":"main","repository":{{"slug":"repo","project":{{"key":"{key}"}},
          "links":{{"clone":[{{"name":"ssh","href":"ssh://git@git.example.com/{key}/repo.git"}},
          {{"name":"http","href":"https://mara@git.example.com/scm/{key}/repo.git"}}]}}}}}},
        "author":{{"user":{{"name":"ann","displayName":"Ann"}}}},"reviewers":[],
        "links":{{"self":[{{"href":"https://git.example.com/projects/{key}/repos/repo/pull-requests/{id}"}}]}}}}"#
    )
}

fn dc_page(rows: &[String]) -> String {
    format!(
        r#"{{"size":{},"limit":100,"isLastPage":true,"values":[{}]}}"#,
        rows.len(),
        rows.join(",")
    )
}

#[test]
fn bitbucket_dc_review_requests_use_the_dashboard_reviewer_role() {
    let t = Arc::new(RecordedTransport::new().on(
        "/dashboard/pull-requests",
        200,
        &dc_page(&[dc_pull(4, "PRJ", false, 1_760_000_000_000)]),
    ));
    let c = client(IntegrationKind::BitbucketDataCenter, DC, &t);
    let items = block_on(c.account_items(&review())).unwrap();
    let seen = urls(&t);
    assert_eq!(
        seen,
        vec![
            "https://git.example.com/rest/api/1.0/dashboard/pull-requests?state=OPEN&role=REVIEWER&order=NEWEST&limit=100&participantStatus=UNAPPROVED"
        ]
    );
    assert_eq!(items.prs.len(), 1);
    let row = &items.prs[0];
    assert_eq!(row.project, "PRJ/repo");
    assert_eq!(row.clone_url, "https://git.example.com/scm/PRJ/repo.git");
    assert!(row.pr.review_requested);
    assert!(items.issues.is_empty());
}

#[test]
fn bitbucket_dc_open_requests_merge_author_and_participant_rows() {
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "role=AUTHOR",
                200,
                &dc_page(&[
                    dc_pull(1, "PRJ", false, 1_760_000_000_000),
                    dc_pull(2, "OTH", true, 1_760_000_001_000),
                ]),
            )
            .on(
                "role=PARTICIPANT",
                200,
                &dc_page(&[
                    dc_pull(1, "PRJ", false, 1_760_000_000_000),
                    dc_pull(3, "PRJ", false, 1_760_000_002_000),
                ]),
            ),
    );
    let c = client(IntegrationKind::BitbucketDataCenter, DC, &t);
    let query = AccountQuery {
        open_requests: true,
        assigned_issues: true,
        include_drafts: true,
        ..AccountQuery::default()
    };
    let items = block_on(c.account_items(&query)).unwrap();
    assert_eq!(items.prs.len(), 3, "request 1 appears once");
    assert!(items.issues.is_empty());
    assert!(
        items.notes.iter().any(|n| n.contains("no issue tracker")),
        "{:?}",
        items.notes
    );
    let only_prj = AccountQuery {
        namespaces: vec!["prj".into()],
        ..query
    };
    let items = block_on(c.account_items(&only_prj)).unwrap();
    assert_eq!(items.prs.len(), 2);
}

#[test]
fn bitbucket_cloud_has_no_account_wide_rows_and_says_why() {
    let t = Arc::new(RecordedTransport::new());
    let c = client(IntegrationKind::Bitbucket, "", &t);
    let err = block_on(c.account_items(&review())).unwrap_err();
    match err {
        Error::Unsupported(why) => assert!(why.contains("Bitbucket Cloud"), "{why}"),
        other => panic!("{other:?}"),
    }
    assert!(t.requests().is_empty(), "no request is made");
    assert!(!supports(IntegrationKind::Bitbucket));
    assert!(supports(IntegrationKind::BitbucketDataCenter));
    assert!(supports(IntegrationKind::AzureDevops));
    assert!(supports(IntegrationKind::AzureDevopsServer));
    assert!(supports(IntegrationKind::Gerrit));
}

// ----------------------------------------------------------------- Gerrit

const GR: &str = "https://review.example.com";

fn gr_change(n: u64, project: &str, wip: bool, more: bool) -> String {
    format!(
        r#"{{"_number":{n},"project":"{project}","branch":"main","subject":"s{n}","status":"NEW",
        "work_in_progress":{wip},"updated":"2026-10-0{n} 10:00:00.000000000",
        "owner":{{"_account_id":2,"username":"ann"}},"_more_changes":{more}}}"#
    )
}

fn gr_body(rows: &[String]) -> String {
    format!(")]}}'\n[{}]", rows.join(","))
}

#[test]
fn gerrit_review_requests_exclude_my_own_ignored_and_wip_changes() {
    let t = Arc::new(RecordedTransport::new().on(
        "/a/changes/",
        200,
        &gr_body(&[gr_change(1, "tools/cli", false, false)]),
    ));
    let c = client(IntegrationKind::Gerrit, GR, &t);
    let items = block_on(c.account_items(&review())).unwrap();
    let seen = urls(&t);
    assert!(
        seen[0].contains("q=is:open reviewer:self -owner:self -is:ignored -is:wip&"),
        "{seen:?}"
    );
    assert_eq!(items.prs.len(), 1);
    assert_eq!(items.prs[0].project, "tools/cli");
    assert_eq!(
        items.prs[0].clone_url,
        "https://review.example.com/tools/cli"
    );
    assert!(items.prs[0].pr.review_requested);

    let drafts = AccountQuery {
        include_drafts: true,
        ..review()
    };
    block_on(c.account_items(&drafts)).unwrap();
    assert!(!urls(&t).last().unwrap().contains("-is:wip"));
}

#[test]
fn gerrit_namespaces_are_project_prefixes_and_more_changes_truncates() {
    let t = Arc::new(RecordedTransport::new().on(
        "/a/changes/",
        200,
        &gr_body(&[
            gr_change(1, "tools/cli", false, false),
            gr_change(2, "tools/web", true, true),
        ]),
    ));
    let c = client(IntegrationKind::Gerrit, GR, &t);
    let query = AccountQuery {
        open_requests: true,
        include_drafts: true,
        namespaces: vec!["tools".into()],
        limit: 2,
        ..AccountQuery::default()
    };
    let items = block_on(c.account_items(&query)).unwrap();
    let seen = urls(&t);
    assert!(seen[0].contains("q=is:open projects:tools&"), "{seen:?}");
    assert!(items.truncated, "the server held more changes");
    assert_eq!(items.prs.len(), 2);
    assert!(items
        .prs
        .iter()
        .any(|p| p.pr.state == crate::PullRequestState::Draft));
    assert!(items.notes.iter().any(|n| n.contains("more")));
}
