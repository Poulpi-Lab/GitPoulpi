//! Account-wide queries over `RecordedTransport` fakes: the URLs and search
//! qualifiers each host receives, the row mapping, the failure policy and the
//! cache. No socket is opened.

use std::sync::Arc;

use super::*;
use crate::client::{client_for, encode_component, HostingClient};
use crate::http::RecordedTransport;
use crate::model::PullRequestState;
use crate::{Error, Integration, IntegrationKind};

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
    login: &str,
    transport: Arc<RecordedTransport>,
) -> Box<dyn HostingClient> {
    let mut integration = Integration::new("acct", kind, url);
    integration.user = Some(gitpoulpi_config::ConnectedUser {
        login: login.to_string(),
        ..Default::default()
    });
    client_for(integration, "tok".into(), transport)
}

/// `%XX` decoded, to read the query a request carried.
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

fn urls(transport: &RecordedTransport) -> Vec<String> {
    transport
        .requests()
        .into_iter()
        .map(|r| decoded(&r.url))
        .collect()
}

fn review_query() -> AccountQuery {
    AccountQuery {
        review_requests: true,
        ..AccountQuery::default()
    }
}

// ---------------------------------------------------------------- GitHub

fn gh_row(n: u64, repo: &str, draft: bool, updated: &str) -> String {
    format!(
        r#"{{"number":{n},"title":"t{n}","draft":{draft},"state":"open",
        "html_url":"https://github.com/{repo}/pull/{n}","updated_at":"{updated}",
        "repository_url":"https://api.github.com/repos/{repo}","user":{{"login":"ann"}},
        "assignees":[],"labels":[{{"name":"bug"}}],"milestone":null}}"#
    )
}

fn gh_search(rows: &[String]) -> String {
    format!(
        r#"{{"total_count":{},"items":[{}]}}"#,
        rows.len(),
        rows.join(",")
    )
}

fn github(transport: &Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    client(
        IntegrationKind::Github,
        "https://github.com",
        "octo",
        transport.clone(),
    )
}

#[test]
fn github_review_requests_search_the_whole_account_and_map_the_project() {
    let transport = Arc::new(RecordedTransport::new().on(
        "/search/issues",
        200,
        &gh_search(&[
            gh_row(4, "acme/api", false, "2026-10-02T00:00:00Z"),
            gh_row(9, "acme/web", false, "2026-10-03T00:00:00Z"),
        ]),
    ));
    let items = block_on(github(&transport).account_items(&review_query())).unwrap();
    let seen = urls(&transport);
    assert_eq!(seen.len(), 1, "{seen:?}");
    assert!(
        seen[0].contains("q=is:pr is:open archived:false draft:false review-requested:@me"),
        "{}",
        seen[0]
    );
    assert!(seen[0].contains("per_page=100"), "{}", seen[0]);
    assert_eq!(items.prs.len(), 2);
    // Newest first.
    assert_eq!(items.prs[0].project, "acme/web");
    assert_eq!(items.prs[0].pr.number, 9);
    assert_eq!(items.prs[0].clone_url, "https://github.com/acme/web.git");
    assert!(items.prs[0].pr.review_requested);
    assert_eq!(items.prs[0].pr.author, "ann");
    assert_eq!(items.prs[0].pr.iid_label, "#9");
    assert!(items.issues.is_empty() && !items.truncated && items.notes.is_empty());
}

#[test]
fn github_open_requests_search_my_user_and_orgs_in_chunks_of_four() {
    let orgs = r#"[{"login":"a"},{"login":"b"},{"login":"c"},{"login":"d"},{"login":"e"}]"#;
    let transport = Arc::new(RecordedTransport::new().on("/user/orgs", 200, orgs).on(
        "/search/issues",
        200,
        &gh_search(&[gh_row(1, "a/x", false, "2026-10-01T00:00:00Z")]),
    ));
    let query = AccountQuery {
        open_requests: true,
        ..AccountQuery::default()
    };
    let items = block_on(github(&transport).account_items(&query)).unwrap();
    let searches: Vec<String> = urls(&transport)
        .into_iter()
        .filter(|u| u.contains("/search/issues"))
        .collect();
    // user + 5 orgs = 6 namespaces = 2 queries.
    assert_eq!(searches.len(), 2, "{searches:?}");
    assert!(
        searches[0]
            .contains("q=is:pr is:open archived:false draft:false user:octo org:a org:b org:c"),
        "{}",
        searches[0]
    );
    assert!(searches[1].contains("org:d org:e"), "{}", searches[1]);
    // Same row from both queries is one row, not flagged for review.
    assert_eq!(items.prs.len(), 1);
    assert!(!items.prs[0].pr.review_requested);
}

#[test]
fn github_namespaces_override_the_membership_and_skip_the_orgs_call() {
    let transport = Arc::new(RecordedTransport::new().on("/search/issues", 200, &gh_search(&[])));
    let query = AccountQuery {
        open_requests: true,
        include_drafts: true,
        namespaces: vec!["acme".into(), "octo".into()],
        ..AccountQuery::default()
    };
    block_on(github(&transport).account_items(&query)).unwrap();
    let seen = urls(&transport);
    assert_eq!(seen.len(), 1, "{seen:?}");
    assert!(
        seen[0].contains("q=is:pr is:open archived:false org:acme user:octo"),
        "{}",
        seen[0]
    );
    assert!(
        !seen[0].contains("draft:false"),
        "drafts asked for: {}",
        seen[0]
    );
}

#[test]
fn github_assigned_issues_use_the_issue_qualifiers() {
    let transport = Arc::new(RecordedTransport::new().on(
        "/search/issues",
        200,
        &gh_search(&[gh_row(12, "acme/api", false, "2026-10-02T00:00:00Z")]),
    ));
    let query = AccountQuery {
        assigned_issues: true,
        ..AccountQuery::default()
    };
    let items = block_on(github(&transport).account_items(&query)).unwrap();
    assert!(
        urls(&transport)[0].contains("q=is:issue is:open archived:false assignee:@me"),
        "{:?}",
        urls(&transport)
    );
    assert!(items.prs.is_empty());
    assert_eq!(items.issues.len(), 1);
    assert_eq!(items.issues[0].project, "acme/api");
    assert_eq!(items.issues[0].issue.number, 12);
    assert_eq!(items.issues[0].issue.labels, vec!["bug"]);
}

#[test]
fn drafts_are_dropped_unless_asked_for_and_map_to_the_draft_state() {
    let rows = gh_search(&[
        gh_row(1, "a/x", true, "2026-10-01T00:00:00Z"),
        gh_row(2, "a/x", false, "2026-10-02T00:00:00Z"),
    ]);
    let transport = Arc::new(RecordedTransport::new().on("/search/issues", 200, &rows));
    // A host that ignores `draft:false` is filtered locally too.
    let hidden = block_on(github(&transport).account_items(&review_query())).unwrap();
    assert_eq!(hidden.prs.len(), 1);
    assert_eq!(hidden.prs[0].pr.number, 2);

    let query = AccountQuery {
        include_drafts: true,
        ..review_query()
    };
    let shown = block_on(github(&transport).account_items(&query)).unwrap();
    assert_eq!(shown.prs.len(), 2);
    let draft = shown.prs.iter().find(|p| p.pr.number == 1).unwrap();
    assert_eq!(draft.pr.state, PullRequestState::Draft);
}

#[test]
fn a_request_in_two_lists_is_one_row_flagged_for_review() {
    let one = gh_search(&[gh_row(7, "acme/api", false, "2026-10-02T00:00:00Z")]);
    let transport = Arc::new(RecordedTransport::new().on("/user/orgs", 200, "[]").on(
        "/search/issues",
        200,
        &one,
    ));
    let query = AccountQuery {
        review_requests: true,
        open_requests: true,
        ..AccountQuery::default()
    };
    let items = block_on(github(&transport).account_items(&query)).unwrap();
    assert_eq!(items.prs.len(), 1, "deduplicated by project and number");
    assert!(items.prs[0].pr.review_requested);
}

#[test]
fn rows_past_the_limit_are_cut_with_a_note() {
    let rows: Vec<String> = (1..=5)
        .map(|n| gh_row(n, "a/x", false, &format!("2026-10-0{n}T00:00:00Z")))
        .collect();
    let transport = Arc::new(RecordedTransport::new().on("/search/issues", 200, &gh_search(&rows)));
    let query = AccountQuery {
        limit: 3,
        ..review_query()
    };
    let items = block_on(github(&transport).account_items(&query)).unwrap();
    assert_eq!(items.prs.len(), 3);
    assert_eq!(items.prs[0].pr.number, 5, "newest kept");
    assert!(items.truncated);
    assert!(
        items
            .notes
            .iter()
            .any(|n| n.contains("2 more") && n.contains("limit 3")),
        "{:?}",
        items.notes
    );
}

#[test]
fn a_rejected_token_or_a_rate_limit_fails_the_call_but_a_missing_org_list_is_a_note() {
    let rejected = Arc::new(RecordedTransport::new().on(
        "/search/issues",
        401,
        r#"{"message":"Bad credentials"}"#,
    ));
    let err = block_on(github(&rejected).account_items(&review_query())).unwrap_err();
    assert!(matches!(err, Error::Unauthorized { .. }), "{err:?}");

    let limited =
        Arc::new(RecordedTransport::new().on("/search/issues", 429, r#"{"message":"slow down"}"#));
    let err = block_on(github(&limited).account_items(&review_query())).unwrap_err();
    assert!(matches!(err, Error::RateLimited { .. }), "{err:?}");

    // The token cannot read organizations: the user's own repositories are
    // still searched and the gap is reported.
    let partial = Arc::new(
        RecordedTransport::new()
            .on("/user/orgs", 404, r#"{"message":"Not Found"}"#)
            .on(
                "/search/issues",
                200,
                &gh_search(&[gh_row(3, "octo/own", false, "2026-10-02T00:00:00Z")]),
            ),
    );
    let query = AccountQuery {
        open_requests: true,
        ..AccountQuery::default()
    };
    let items = block_on(github(&partial).account_items(&query)).unwrap();
    assert_eq!(items.prs.len(), 1);
    assert!(
        items.notes.iter().any(|n| n.starts_with("organizations:")),
        "{:?}",
        items.notes
    );
}

// ---------------------------------------------------------------- GitLab

fn gl_mr(iid: u64, project: &str, draft: bool, reviewers: &str, updated: &str) -> String {
    format!(
        r#"{{"iid":{iid},"title":"mr{iid}","state":"opened","draft":{draft},
        "source_branch":"feat","target_branch":"main","source_project_id":1,"target_project_id":1,
        "author":{{"username":"ann"}},"web_url":"https://gitlab.example/{project}/-/merge_requests/{iid}",
        "updated_at":"{updated}","assignees":[],"reviewers":[{reviewers}],
        "references":{{"full":"{project}!{iid}"}}}}"#
    )
}

fn gitlab(transport: &Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    client(
        IntegrationKind::GitlabSelfManaged,
        "https://gitlab.example",
        "me",
        transport.clone(),
    )
}

#[test]
fn gitlab_review_requests_use_the_global_list_and_the_reference_for_the_project() {
    let body = format!(
        "[{}]",
        gl_mr(
            5,
            "grp/sub/proj",
            false,
            r#"{"username":"me"}"#,
            "2026-10-02T00:00:00Z"
        )
    );
    let transport = Arc::new(RecordedTransport::new().on("/merge_requests", 200, &body));
    let items = block_on(gitlab(&transport).account_items(&review_query())).unwrap();
    let seen = urls(&transport);
    assert_eq!(seen.len(), 1);
    assert!(
        seen[0].contains("/api/v4/merge_requests?scope=all&state=opened"),
        "{}",
        seen[0]
    );
    assert!(seen[0].contains("reviewer_username=me"), "{}", seen[0]);
    assert!(seen[0].contains("draft=no"), "{}", seen[0]);
    assert_eq!(items.prs[0].project, "grp/sub/proj");
    assert_eq!(items.prs[0].pr.iid_label, "!5");
    assert_eq!(
        items.prs[0].clone_url,
        "https://gitlab.example/grp/sub/proj.git"
    );
    assert!(items.prs[0].pr.review_requested);
}

#[test]
fn gitlab_open_requests_walk_groups_then_owned_projects_and_merge_duplicates() {
    let groups = r#"[{"id":11,"full_path":"grp"},{"id":12,"full_path":"other"}]"#;
    let projects = r#"[{"id":21}]"#;
    let in_group = format!(
        "[{}]",
        gl_mr(1, "grp/proj", false, "", "2026-10-01T00:00:00Z")
    );
    let in_other = format!(
        "[{},{}]",
        gl_mr(1, "grp/proj", false, "", "2026-10-01T00:00:00Z"),
        gl_mr(
            2,
            "other/lib",
            false,
            r#"{"username":"me"}"#,
            "2026-10-03T00:00:00Z"
        )
    );
    let transport = Arc::new(
        RecordedTransport::new()
            .on("/groups?", 200, groups)
            .on("/groups/11/merge_requests", 200, &in_group)
            .on("/groups/12/merge_requests", 200, &in_other)
            .on("/projects?", 200, projects)
            .on("/projects/21/merge_requests", 200, "[]"),
    );
    let query = AccountQuery {
        open_requests: true,
        ..AccountQuery::default()
    };
    let items = block_on(gitlab(&transport).account_items(&query)).unwrap();
    let seen = urls(&transport);
    assert!(
        seen.iter()
            .any(|u| u.contains("/groups?min_access_level=10&top_level_only=true&per_page=11")),
        "{seen:?}"
    );
    assert!(
        seen.iter().any(|u| u
            .contains("/groups/11/merge_requests?include_subgroups=true&state=opened")
            && u.contains("non_archived=true")
            && u.contains("draft=no")),
        "{seen:?}"
    );
    assert!(
        seen.iter().any(|u| u.contains("/projects?owned=true")),
        "{seen:?}"
    );
    assert!(
        seen.iter()
            .any(|u| u.contains("/projects/21/merge_requests")),
        "{seen:?}"
    );
    assert_eq!(items.prs.len(), 2, "grp/proj!1 seen twice is one row");
    // A reviewer list naming me marks the row even outside the review list.
    let lib = items.prs.iter().find(|p| p.project == "other/lib").unwrap();
    assert!(lib.pr.review_requested);
}

#[test]
fn gitlab_namespaces_replace_the_walk_and_a_missing_group_is_a_note() {
    let ok = format!(
        "[{}]",
        gl_mr(3, "known/p", false, "", "2026-10-01T00:00:00Z")
    );
    let transport = Arc::new(
        RecordedTransport::new()
            .on("/groups/known/merge_requests", 200, &ok)
            .on(
                "/groups/gone/merge_requests",
                404,
                r#"{"message":"404 Group Not Found"}"#,
            ),
    );
    let query = AccountQuery {
        open_requests: true,
        include_drafts: true,
        namespaces: vec!["gone".into(), "known".into()],
        ..AccountQuery::default()
    };
    let items = block_on(gitlab(&transport).account_items(&query)).unwrap();
    assert_eq!(items.prs.len(), 1);
    assert!(
        items
            .notes
            .iter()
            .any(|n| n.contains("open merge requests") && n.contains("404")),
        "{:?}",
        items.notes
    );
    assert!(
        urls(&transport).iter().all(|u| !u.contains("draft=no")),
        "drafts asked for"
    );
}

#[test]
fn gitlab_assigned_issues_read_the_project_from_the_reference() {
    let body = r#"[{"iid":8,"title":"bug","web_url":"https://gitlab.example/grp/proj/-/issues/8",
        "updated_at":"2026-10-02T00:00:00Z","author":{"username":"ann"},
        "assignees":[{"username":"me"}],"labels":["p1"],"milestone":{"title":"v2"},
        "references":{"full":"grp/proj#8"}}]"#;
    let transport = Arc::new(RecordedTransport::new().on("/issues", 200, body));
    let query = AccountQuery {
        assigned_issues: true,
        ..AccountQuery::default()
    };
    let items = block_on(gitlab(&transport).account_items(&query)).unwrap();
    assert!(
        urls(&transport)[0].contains("/api/v4/issues?scope=assigned_to_me&state=opened"),
        "{:?}",
        urls(&transport)
    );
    assert_eq!(items.issues[0].project, "grp/proj");
    assert_eq!(items.issues[0].issue.number, 8);
    assert_eq!(items.issues[0].issue.milestone.as_deref(), Some("v2"));
}

// ----------------------------------------------------------------- Gitea

fn gt_row(n: u64, repo: &str, title: &str, draft: bool, pull: bool) -> String {
    let pr = if pull {
        format!(r#","pull_request":{{"draft":{draft},"merged":false}}"#)
    } else {
        String::new()
    };
    format!(
        r#"{{"number":{n},"title":"{title}","html_url":"https://git.example/{repo}/pulls/{n}",
        "updated_at":"2026-10-0{n}T00:00:00Z","user":{{"login":"ann"}},"assignees":null,
        "labels":[{{"name":"bug"}}],"milestone":null,"repository":{{"full_name":"{repo}"}}{pr}}}"#
    )
}

fn gitea(transport: &Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    client(
        IntegrationKind::Forgejo,
        "https://git.example",
        "me",
        transport.clone(),
    )
}

#[test]
fn gitea_review_requests_and_issues_use_the_global_search() {
    let transport = Arc::new(
        RecordedTransport::new()
            .on(
                "type=pulls",
                200,
                &format!("[{}]", gt_row(3, "o/p", "WIP: later", false, true)),
            )
            .on(
                "type=issues",
                200,
                &format!("[{}]", gt_row(4, "o/q", "crash", false, false)),
            ),
    );
    let query = AccountQuery {
        review_requests: true,
        assigned_issues: true,
        include_drafts: true,
        ..AccountQuery::default()
    };
    let items = block_on(gitea(&transport).account_items(&query)).unwrap();
    let seen = urls(&transport);
    assert!(
        seen.iter().any(|u| u.contains("/api/v1/repos/issues/search?state=open&type=pulls&review_requested=true&limit=50&page=1")),
        "{seen:?}"
    );
    assert!(
        seen.iter().any(|u| u.contains("type=issues&assigned=true")),
        "{seen:?}"
    );
    // A `WIP:` title is a draft even without the flag.
    assert_eq!(items.prs[0].pr.state, PullRequestState::Draft);
    assert!(items.prs[0].pr.review_requested);
    assert_eq!(items.prs[0].clone_url, "https://git.example/o/p.git");
    assert_eq!(items.issues[0].project, "o/q");
}

#[test]
fn gitea_open_requests_search_each_namespace_by_owner() {
    let transport = Arc::new(
        RecordedTransport::new()
            .on("/user/orgs", 200, r#"[{"username":"team"}]"#)
            .on(
                "owner=team",
                200,
                &format!("[{}]", gt_row(2, "team/x", "feature", false, true)),
            )
            .on("owner=me", 200, "[]"),
    );
    let query = AccountQuery {
        open_requests: true,
        ..AccountQuery::default()
    };
    let items = block_on(gitea(&transport).account_items(&query)).unwrap();
    let seen = urls(&transport);
    assert!(
        seen.iter().any(|u| u.contains("type=pulls&owner=me")),
        "{seen:?}"
    );
    assert!(
        seen.iter().any(|u| u.contains("type=pulls&owner=team")),
        "{seen:?}"
    );
    assert_eq!(items.prs.len(), 1);
    assert_eq!(items.prs[0].project, "team/x");
    assert!(!items.prs[0].pr.review_requested);
}

// ------------------------------------------------------- other hosts, cache

#[test]
fn a_host_without_account_lists_says_unsupported() {
    let transport = Arc::new(RecordedTransport::new());
    let c = client(
        IntegrationKind::JiraCloud,
        "https://acme.atlassian.net",
        "me",
        transport,
    );
    let err = block_on(c.account_items(&review_query())).unwrap_err();
    assert!(matches!(err, Error::Unsupported(_)), "{err:?}");
}

#[test]
fn the_fingerprint_follows_the_question_not_the_namespace_order() {
    let a = AccountQuery {
        open_requests: true,
        namespaces: vec!["B".into(), "a".into()],
        ..AccountQuery::default()
    };
    let b = AccountQuery {
        namespaces: vec!["a".into(), "b".into()],
        ..a.clone()
    };
    assert_eq!(a.fingerprint(), b.fingerprint());
    let c = AccountQuery {
        include_drafts: true,
        ..a.clone()
    };
    assert_ne!(a.fingerprint(), c.fingerprint());
    assert_ne!(a.fingerprint(), review_query().fingerprint());
    assert_eq!(AccountQuery::default().row_limit(), DEFAULT_LIMIT);
}

fn entry(at: u64, hash: &str) -> CachedAccount {
    CachedAccount {
        fetched_at: at,
        query_hash: hash.into(),
        items: AccountItems {
            notes: vec!["n".into()],
            ..AccountItems::default()
        },
    }
}

#[test]
fn the_cache_reuses_a_young_entry_for_the_same_question_only() {
    let dir = tempfile::tempdir().unwrap();
    let path = cache_path(dir.path());
    assert!(path.ends_with("radar-account.json"));
    AccountCache::store(&path, "gh", entry(1_000, "h1")).unwrap();
    AccountCache::store(&path, "gl", entry(1_100, "h2")).unwrap();
    let cache = AccountCache::load(&path);
    assert_eq!(
        cache.entries.len(),
        2,
        "a second store keeps the first entry"
    );
    assert!(cache
        .fresh("gh", "h1", 1_000 + ACCOUNT_TTL_SECS - 1)
        .is_some());
    assert!(
        cache.fresh("gh", "h1", 1_000 + ACCOUNT_TTL_SECS).is_none(),
        "five minutes old is stale"
    );
    assert!(
        cache.get("gh", "h1").is_some(),
        "stale rows still show while refreshing"
    );
    assert!(
        cache.get("gh", "other").is_none(),
        "another question is a miss"
    );
    assert!(cache.get("nope", "h1").is_none());
}

#[test]
fn a_corrupt_cache_is_empty_and_one_bad_entry_drops_only_itself() {
    let dir = tempfile::tempdir().unwrap();
    let path = cache_path(dir.path());
    assert!(AccountCache::load(&path).entries.is_empty(), "missing file");
    std::fs::write(&path, "{ not json").unwrap();
    assert!(AccountCache::load(&path).entries.is_empty(), "corrupt file");
    let good = serde_json::to_value(entry(5, "h")).unwrap();
    let raw = serde_json::json!({ "ok": good, "bad": { "fetched_at": "yesterday" } });
    std::fs::write(&path, raw.to_string()).unwrap();
    let cache = AccountCache::load(&path);
    assert_eq!(cache.entries.keys().collect::<Vec<_>>(), vec!["ok"]);
    // Storing over a corrupt file still works.
    std::fs::write(&path, "garbage").unwrap();
    AccountCache::store(&path, "gh", entry(7, "h")).unwrap();
    assert_eq!(AccountCache::load(&path).entries.len(), 1);
}

#[test]
fn a_duplicate_only_adds_the_review_flag_and_reviewers() {
    let row = |review: bool, reviewers: &[&str]| AccountPr {
        project: "a/b".into(),
        clone_url: String::new(),
        pr: crate::model::PullRequestSummary {
            number: 3,
            review_requested: review,
            reviewers: reviewers.iter().map(|r| r.to_string()).collect(),
            ..Default::default()
        },
    };
    let mut c = Collector::default();
    c.add_pr(row(false, &["x"]));
    c.add_pr(row(true, &["x", "me"]));
    let items = c.finish(&AccountQuery::default());
    assert_eq!(items.prs.len(), 1);
    assert!(items.prs[0].pr.review_requested);
    assert_eq!(items.prs[0].pr.reviewers, vec!["x", "me"]);

    let issue = AccountIssue {
        project: "a/b".into(),
        clone_url: String::new(),
        issue: crate::model::IssueSummary {
            number: 8,
            ..Default::default()
        },
    };
    let mut c = Collector::default();
    c.add_issue(issue.clone());
    c.add_issue(issue);
    assert_eq!(c.finish(&AccountQuery::default()).issues.len(), 1);
}

#[test]
fn gitlab_walks_at_most_ten_groups_and_says_so() {
    let groups: Vec<serde_json::Value> = (1..=11)
        .map(|id| serde_json::json!({"id": id, "full_path": format!("g{id}")}))
        .collect();
    let transport = Arc::new(
        RecordedTransport::new()
            .on("/groups?", 200, &serde_json::to_string(&groups).unwrap())
            .on("/groups/", 200, "[]")
            .on("/projects?", 200, "[]"),
    );
    let query = AccountQuery {
        open_requests: true,
        ..AccountQuery::default()
    };
    let items = block_on(gitlab(&transport).account_items(&query)).unwrap();
    let walked = urls(&transport)
        .into_iter()
        .filter(|u| u.contains("/groups/") && u.contains("/merge_requests"))
        .count();
    assert_eq!(walked, 10);
    assert!(
        items
            .notes
            .iter()
            .any(|n| n.contains("groups") && n.contains("first 10")),
        "{:?}",
        items.notes
    );
}

#[test]
fn encode_helper_matches_the_qualifier_spelling_used_above() {
    assert_eq!(
        encode_component("review-requested:@me"),
        "review-requested%3A%40me"
    );
}
