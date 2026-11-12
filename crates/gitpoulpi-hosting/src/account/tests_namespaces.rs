//! The Namespaces filter on every account-wide list, over fake transports:
//! the review requests and the assigned issues come from global host queries,
//! so only the filter on the project path keeps `formation/...` out of a
//! `ptm` profile.

use std::sync::Arc;

use super::*;
use crate::client::{client_for, HostingClient};
use crate::http::RecordedTransport;
use crate::{Integration, IntegrationKind};

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn client(
    kind: IntegrationKind,
    url: &str,
    transport: &Arc<RecordedTransport>,
) -> Box<dyn HostingClient> {
    let mut integration = Integration::new("acct", kind, url);
    integration.user = Some(gitpoulpi_config::ConnectedUser {
        login: "me".to_string(),
        ..Default::default()
    });
    client_for(
        integration,
        "tok".into(),
        transport.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap()
}

fn spaces(lines: &[&str], regex: bool) -> AccountQuery {
    AccountQuery {
        review_requests: true,
        open_requests: true,
        assigned_issues: true,
        include_drafts: true,
        namespaces: lines.iter().map(|s| s.to_string()).collect(),
        namespaces_regex: regex,
        ..AccountQuery::default()
    }
}

fn gl_mr(iid: u64, project: &str) -> String {
    format!(
        r#"{{"iid":{iid},"title":"mr{iid}","state":"opened","draft":false,
        "source_branch":"f","target_branch":"main","source_project_id":1,"target_project_id":1,
        "author":{{"username":"ann"}},"web_url":"https://gitlab.example/{project}/-/merge_requests/{iid}",
        "updated_at":"2026-10-0{iid}T00:00:00Z","assignees":[],"reviewers":[],
        "references":{{"full":"{project}!{iid}"}}}}"#
    )
}

fn gl_issue(iid: u64, project: &str) -> String {
    format!(
        r#"{{"iid":{iid},"title":"i{iid}","web_url":"https://gitlab.example/{project}/-/issues/{iid}",
        "updated_at":"2026-10-0{iid}T00:00:00Z","author":{{"username":"ann"}},
        "assignees":[{{"username":"me"}}],"labels":[],"references":{{"full":"{project}#{iid}"}}}}"#
    )
}

fn gitlab_transport() -> Arc<RecordedTransport> {
    // Every merge-request list (global, group) answers the same mixed rows,
    // as a host would that holds both namespaces; so do the issues.
    let mrs = format!(
        "[{}]",
        [
            gl_mr(1, "ptm/produits/x"),
            gl_mr(2, "formation/y"),
            gl_mr(3, "ptmx/z"),
            gl_mr(4, "ptm/w"),
        ]
        .join(",")
    );
    let issues = format!(
        "[{}]",
        [gl_issue(5, "formation/y"), gl_issue(6, "ptm/produits/x")].join(",")
    );
    Arc::new(
        RecordedTransport::new()
            .on("/groups?", 200, r#"[{"id":7}]"#)
            .on("/projects?", 200, "[]")
            .on("/merge_requests", 200, &mrs)
            .on("/issues", 200, &issues),
    )
}

fn projects(items: &AccountItems) -> (Vec<String>, Vec<String>) {
    let mut prs: Vec<String> = items.prs.iter().map(|p| p.project.clone()).collect();
    let mut issues: Vec<String> = items.issues.iter().map(|i| i.project.clone()).collect();
    prs.sort();
    issues.sort();
    (prs, issues)
}

#[test]
fn gitlab_every_section_keeps_only_the_namespace() {
    let transport = gitlab_transport();
    let c = client(
        IntegrationKind::GitlabSelfManaged,
        "https://gitlab.example",
        &transport,
    );
    let items = block_on(c.account_items(&spaces(&["ptm"], false))).unwrap();
    let (prs, issues) = projects(&items);
    // `formation/y` and `ptmx/z` are gone from the review list, the group
    // list and the issues alike; `ptm/w` and `ptm/produits/x` stay.
    assert_eq!(prs, ["ptm/produits/x", "ptm/w"]);
    assert_eq!(issues, ["ptm/produits/x"]);
}

#[test]
fn gitlab_without_namespaces_keeps_every_project() {
    let transport = gitlab_transport();
    let c = client(
        IntegrationKind::GitlabSelfManaged,
        "https://gitlab.example",
        &transport,
    );
    let items = block_on(c.account_items(&spaces(&[], false))).unwrap();
    let (prs, issues) = projects(&items);
    assert_eq!(prs.len(), 4);
    assert_eq!(issues.len(), 2);
}

#[test]
fn gitlab_two_lines_and_a_subgroup_are_alternatives() {
    let transport = gitlab_transport();
    let c = client(
        IntegrationKind::GitlabSelfManaged,
        "https://gitlab.example",
        &transport,
    );
    let items = block_on(c.account_items(&spaces(&["ptm/produits", "formation"], false))).unwrap();
    let (prs, issues) = projects(&items);
    assert_eq!(prs, ["formation/y", "ptm/produits/x"]);
    assert_eq!(issues, ["formation/y", "ptm/produits/x"]);
}

#[test]
fn gitlab_regex_mode_filters_every_section_and_searches_by_membership() {
    let transport = gitlab_transport();
    let c = client(
        IntegrationKind::GitlabSelfManaged,
        "https://gitlab.example",
        &transport,
    );
    let items = block_on(c.account_items(&spaces(&["^ptm/.*/x$", "^formation/"], true))).unwrap();
    let (prs, issues) = projects(&items);
    assert_eq!(prs, ["formation/y", "ptm/produits/x"]);
    assert_eq!(issues, ["formation/y", "ptm/produits/x"]);
    // A pattern is not a group: the open-request walk is the membership one.
    let seen: Vec<String> = transport.requests().into_iter().map(|r| r.url).collect();
    assert!(
        seen.iter().any(|u| u.contains("/groups?min_access_level")),
        "{seen:?}"
    );
    assert!(
        !seen
            .iter()
            .any(|u| u.contains("/groups/%5E") || u.contains("/groups/^")),
        "{seen:?}"
    );
}

#[test]
fn an_invalid_regex_returns_nothing_and_a_note_instead_of_failing() {
    let transport = gitlab_transport();
    let c = client(
        IntegrationKind::GitlabSelfManaged,
        "https://gitlab.example",
        &transport,
    );
    let items = block_on(c.account_items(&spaces(&["(oops"], true))).unwrap();
    assert!(items.prs.is_empty() && items.issues.is_empty());
    assert!(
        items.notes.iter().any(|n| n.contains("(oops")),
        "{:?}",
        items.notes
    );
}

fn gh_row(n: u64, repo: &str) -> String {
    format!(
        r#"{{"number":{n},"title":"t{n}","draft":false,"state":"open",
        "html_url":"https://github.com/{repo}/pull/{n}","updated_at":"2026-10-0{n}T00:00:00Z",
        "repository_url":"https://api.github.com/repos/{repo}","user":{{"login":"ann"}},
        "assignees":[],"labels":[],"milestone":null}}"#
    )
}

#[test]
fn github_scopes_the_global_searches_and_filters_every_section() {
    let body = format!(
        r#"{{"total_count":3,"items":[{}]}}"#,
        [
            gh_row(1, "ptm/api"),
            gh_row(2, "formation/web"),
            gh_row(3, "ptmx/q")
        ]
        .join(",")
    );
    let transport = Arc::new(RecordedTransport::new().on("/search/issues", 200, &body));
    let c = client(IntegrationKind::Github, "https://github.com", &transport);
    let items = block_on(c.account_items(&spaces(&["ptm", "acme/tools"], false))).unwrap();
    let (prs, issues) = projects(&items);
    assert_eq!(prs, ["ptm/api"]);
    assert_eq!(issues, ["ptm/api"]);
    let seen: Vec<String> = transport.requests().into_iter().map(|r| r.url).collect();
    let decoded: Vec<String> = seen
        .iter()
        .map(|u| {
            u.replace("%40", "@")
                .replace("%3A", ":")
                .replace("%2F", "/")
                .replace('+', " ")
                .replace("%20", " ")
        })
        .collect();
    // Review requests and assigned issues are narrowed server side too.
    assert!(
        decoded
            .iter()
            .any(|u| u.contains("review-requested:@me user:ptm repo:acme/tools")),
        "{decoded:?}"
    );
    assert!(
        decoded
            .iter()
            .any(|u| u.contains("assignee:@me user:ptm repo:acme/tools")),
        "{decoded:?}"
    );
}

#[test]
fn gitea_filters_the_global_review_and_issue_searches() {
    let row = |n: u64, repo: &str, pull: bool| {
        let pr = if pull {
            r#","pull_request":{"draft":false,"merged":false}"#
        } else {
            ""
        };
        format!(
            r#"{{"number":{n},"title":"t{n}","html_url":"https://git.example/{repo}/pulls/{n}",
            "updated_at":"2026-10-0{n}T00:00:00Z","user":{{"login":"ann"}},"assignees":null,
            "labels":[],"milestone":null,"repository":{{"full_name":"{repo}"}}{pr}}}"#
        )
    };
    let transport = Arc::new(
        RecordedTransport::new()
            .on(
                "type=pulls",
                200,
                &format!(
                    "[{},{}]",
                    row(1, "ptm/a", true),
                    row(2, "formation/b", true)
                ),
            )
            .on(
                "type=issues",
                200,
                &format!(
                    "[{},{}]",
                    row(3, "ptm/c", false),
                    row(4, "formation/d", false)
                ),
            ),
    );
    let c = client(IntegrationKind::Forgejo, "https://git.example", &transport);
    let query = AccountQuery {
        review_requests: true,
        assigned_issues: true,
        namespaces: vec!["ptm".into()],
        ..AccountQuery::default()
    };
    let items = block_on(c.account_items(&query)).unwrap();
    let (prs, issues) = projects(&items);
    assert_eq!(prs, ["ptm/a"]);
    assert_eq!(issues, ["ptm/c"]);
}

#[test]
fn the_fingerprint_tells_plain_from_regex() {
    let plain = spaces(&["a"], false);
    let regex = spaces(&["a"], true);
    assert_ne!(plain.fingerprint(), regex.fingerprint());
}
