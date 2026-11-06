use std::sync::Arc;

use crate::client::{client_for, NewIssue, NewPullRequest};
use crate::http::{HttpRequest, RecordedTransport};
use crate::model::{IssueFilter, PrFilter, PullRequestState, StateFilter};
use crate::spec::{spec_for, CheckoutSource};
use crate::{Error, Integration, IntegrationKind};
use gitpoulpi_config::ApiFamily;

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn profile(account: &str) -> Integration {
    let mut i = Integration::new("bb", IntegrationKind::Bitbucket, "https://bitbucket.org");
    i.account = account.into();
    i
}

fn header<'a>(r: &'a HttpRequest, name: &str) -> Option<&'a str> {
    r.headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

const ME: &str = r#"{"uuid":"{u-1}","nickname":"me","display_name":"Me Myself",
    "links":{"html":{"href":"https://bitbucket.org/%7Bu-1%7D/"}}}"#;

fn pr_row(id: u64, state: &str, draft: bool, fork: &str) -> String {
    format!(
        r#"{{"id":{id},"title":"T{id}","state":"{state}","draft":{draft},
        "source":{{"branch":{{"name":"feat"}},"repository":{{"full_name":"{fork}"}}}},
        "destination":{{"branch":{{"name":"main"}},"repository":{{"full_name":"ws/repo"}}}},
        "author":{{"uuid":"{{a}}","display_name":"Ann"}},
        "updated_on":"2026-10-0{id}T10:00:00Z",
        "links":{{"html":{{"href":"https://bitbucket.org/ws/repo/pull-requests/{id}"}}}},
        "participants":[{{"user":{{"uuid":"{{u-1}}","display_name":"Me Myself"}},"role":"REVIEWER","approved":false}}]}}"#
    )
}

#[test]
fn basic_header_with_an_account_and_bearer_without() {
    let t = Arc::new(RecordedTransport::new().on("/user", 200, ME));
    let c = client_for(
        profile("me@x"),
        "tok".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    let user = block_on(c.current_user()).unwrap();
    assert_eq!(
        (user.login.as_str(), user.name.as_str()),
        ("me", "Me Myself")
    );
    assert_eq!(user.id, "{u-1}");
    let seen = t.requests();
    assert_eq!(seen[0].url, "https://api.bitbucket.org/2.0/user");
    assert_eq!(
        header(&seen[0], "Authorization"),
        Some("Basic bWVAeDp0b2s=")
    );

    let t = Arc::new(RecordedTransport::new().on("/user", 200, ME));
    let c = client_for(
        profile(""),
        "tok".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    block_on(c.check_token()).unwrap();
    assert_eq!(
        header(&t.requests()[0], "Authorization"),
        Some("Bearer tok")
    );
}

#[test]
fn check_token_reports_unknown_scopes() {
    let t = Arc::new(RecordedTransport::new().on("/user", 200, ME));
    let c = client_for(
        profile("me@x"),
        "tok".into(),
        t,
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    let check = block_on(c.check_token()).unwrap();
    assert!(check.scopes.is_none() && check.missing_scopes.is_empty());
}

#[test]
fn error_statuses_map_to_typed_errors() {
    for (status, ok) in [(401u16, 0), (403, 1), (429, 2), (500, 3)] {
        let t = Arc::new(RecordedTransport::new().on(
            "/user",
            status,
            r#"{"type":"error","error":{"message":"nope"}}"#,
        ));
        let c = client_for(
            profile("a@b"),
            "tok".into(),
            t,
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap();
        let err = block_on(c.current_user()).unwrap_err();
        match (ok, &err) {
            (0, Error::Unauthorized { .. })
            | (1, Error::Forbidden { .. })
            | (2, Error::RateLimited { .. })
            | (3, Error::Api { status: 500, .. }) => {}
            _ => panic!("{status}: {err:?}"),
        }
    }
}

#[test]
fn projects_come_from_discovered_workspaces_and_follow_next() {
    let ws = r#"{"values":[{"workspace":{"slug":"ws1"}},{"workspace":{"slug":"ws2"}}]}"#;
    let repo = |name: &str| {
        format!(
            r#"{{"uuid":"{{r}}","full_name":"ws1/{name}","name":"{name}","is_private":true,
            "mainbranch":{{"name":"main"}},
            "links":{{"html":{{"href":"https://bitbucket.org/ws1/{name}"}},
              "clone":[{{"name":"https","href":"https://me@bitbucket.org/ws1/{name}.git"}},
                       {{"name":"ssh","href":"git@bitbucket.org:ws1/{name}.git"}}]}}}}"#
        )
    };
    let page1 = format!(
        r#"{{"values":[{}],"next":"https://api.bitbucket.org/2.0/repositories/ws1?page=2"}}"#,
        repo("a")
    );
    let page2 = format!(r#"{{"values":[{}]}}"#, repo("b"));
    let t = Arc::new(
        RecordedTransport::new()
            .on("/user/workspaces", 200, ws)
            .on("page=2", 200, &page2)
            .on("/repositories/ws1?", 200, &page1)
            .on("/repositories/ws2", 403, r#"{"error":{"message":"scope"}}"#),
    );
    let c = client_for(
        profile("a@b"),
        "tok".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    let projects = block_on(c.list_projects()).unwrap();
    let names: Vec<_> = projects.iter().map(|p| p.full_name.as_str()).collect();
    assert_eq!(names, ["ws1/a", "ws1/b"], "ws2 forbidden is skipped");
    assert_eq!(
        projects[0].https_url.as_deref(),
        Some("https://bitbucket.org/ws1/a.git"),
        "the embedded user is stripped"
    );
    assert_eq!(
        projects[0].ssh_url.as_deref(),
        Some("git@bitbucket.org:ws1/a.git")
    );
    assert_eq!(projects[0].default_branch.as_deref(), Some("main"));
    assert!(t
        .requests()
        .iter()
        .any(|r| r.url.contains("repositories/ws1?role=member")));
}

#[test]
fn a_next_link_to_another_host_is_not_followed() {
    let page = r#"{"values":[],"next":"https://evil.example/2.0/steal"}"#;
    let t = Arc::new(RecordedTransport::new().on("/user/workspaces", 200, page));
    let c = client_for(
        profile("a@b"),
        "tok".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    assert!(matches!(block_on(c.list_projects()), Err(Error::Decode(_))));
    assert_eq!(t.requests().len(), 1);
}

#[test]
fn paging_is_capped() {
    // A host that always answers with a next link must stop at MAX_PAGES.
    let page = r#"{"values":[],"next":"https://api.bitbucket.org/2.0/user/workspaces?again"}"#;
    let t = Arc::new(RecordedTransport::new().on("/user/workspaces", 200, page));
    let c = client_for(
        profile("a@b"),
        "tok".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    block_on(c.list_projects()).unwrap();
    assert_eq!(t.requests().len(), crate::client::MAX_PAGES as usize);
}

#[test]
fn pull_requests_map_states_forks_and_review_requests() {
    let body = format!(
        r#"{{"values":[{},{},{},{},{}]}}"#,
        pr_row(1, "OPEN", false, "ws/repo"),
        pr_row(2, "OPEN", true, "fork/repo"),
        pr_row(3, "MERGED", false, "ws/repo"),
        pr_row(4, "DECLINED", false, "ws/repo"),
        pr_row(5, "SUPERSEDED", false, "ws/repo")
    );
    let t = Arc::new(
        RecordedTransport::new()
            .on("/pullrequests", 200, &body)
            .on("/user", 200, ME),
    );
    let c = client_for(
        profile("a@b"),
        "tok".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    let filter = PrFilter {
        state: StateFilter::All,
        mine: true,
        review_requested: false,
        ..Default::default()
    };
    let rows = block_on(c.list_pull_requests("ws/repo", &filter)).unwrap();
    let states: Vec<_> = rows.iter().map(|r| r.state).collect();
    assert_eq!(
        states,
        [
            PullRequestState::Open,
            PullRequestState::Draft,
            PullRequestState::Merged,
            PullRequestState::Closed,
            PullRequestState::Closed
        ]
    );
    assert_eq!(rows[1].source_project.as_deref(), Some("fork/repo"));
    assert_eq!(rows[0].source_project, None);
    assert_eq!(rows[0].author, "Ann");
    assert_eq!(rows[0].iid_label, "#1");
    assert!(rows[0].review_requested && !rows[2].review_requested);
    assert_eq!(rows[0].reviewers, ["Me Myself"]);
    let url = t.requests().into_iter().last().unwrap().url;
    for state in ["OPEN", "MERGED", "DECLINED", "SUPERSEDED"] {
        assert!(url.contains(&format!("state={state}")), "{url}");
    }
    assert!(url.contains("pagelen=50"));
    assert!(url.contains("q=author.uuid%3D%22%7Bu-1%7D%22"), "{url}");

    let only = PrFilter {
        review_requested: true,
        ..PrFilter::default()
    };
    let rows = block_on(c.list_pull_requests("ws/repo", &only)).unwrap();
    assert_eq!(rows.len(), 2, "open + draft the user must review");
}

#[test]
fn the_branch_lookup_queries_the_source_branch_and_picks_the_best() {
    let body = format!(
        r#"{{"values":[{},{}]}}"#,
        pr_row(3, "MERGED", false, "ws/repo"),
        pr_row(2, "OPEN", false, "ws/repo")
    );
    let t = Arc::new(RecordedTransport::new().on("/pullrequests", 200, &body));
    let c = client_for(
        profile("a@b"),
        "tok".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    let best = block_on(c.pull_request_for_branch("ws/repo", "feat\"x"))
        .unwrap()
        .unwrap();
    assert_eq!(best.number, 2);
    let url = &t.requests()[0].url;
    assert!(
        url.contains("q=source.branch.name%3D%22feat%5C%22x%22"),
        "{url}"
    );
}

#[test]
fn create_pull_request_sends_draft_and_the_fork_repository() {
    let created = pr_row(9, "OPEN", true, "fork/repo");
    let t = Arc::new(RecordedTransport::new().on("/pullrequests", 201, &created));
    let c = client_for(
        profile("a@b"),
        "tok".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    let got = block_on(c.create_pull_request(&NewPullRequest {
        project: "ws/repo".into(),
        source_branch: "feat".into(),
        target_branch: "main".into(),
        title: "Add".into(),
        body: "Why".into(),
        draft: true,
        source_project: Some("fork/repo".into()),
        ..Default::default()
    }))
    .unwrap();
    assert_eq!(got.number, 9);
    assert_eq!(got.web_url, "https://bitbucket.org/ws/repo/pull-requests/9");
    let req = &t.requests()[0];
    assert_eq!(
        req.url,
        "https://api.bitbucket.org/2.0/repositories/ws/repo/pullrequests"
    );
    let body: serde_json::Value = serde_json::from_str(req.body.as_deref().unwrap()).unwrap();
    assert_eq!(body["draft"], true);
    assert_eq!(body["source"]["branch"]["name"], "feat");
    assert_eq!(body["source"]["repository"]["full_name"], "fork/repo");
    assert_eq!(body["destination"]["branch"]["name"], "main");
}

#[test]
fn ssh_keys_list_upload_and_delete_use_the_user_uuid() {
    let key = r#"{"uuid":"{k-1}","key":"ssh-ed25519 AAAA","label":"laptop"}"#;
    let list = format!(r#"{{"values":[{key}]}}"#);
    let t = Arc::new(
        RecordedTransport::new()
            .on("/ssh-keys/", 204, "")
            .on("/ssh-keys", 200, &list)
            // Last: `/2.0/user` is a prefix of `/2.0/users/...`.
            .on("/2.0/user", 200, ME),
    );
    let c = client_for(
        profile("a@b"),
        "tok".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    let keys = block_on(c.list_ssh_keys()).unwrap();
    assert_eq!(
        (keys[0].id.as_str(), keys[0].title.as_str()),
        ("{k-1}", "laptop")
    );
    block_on(c.delete_ssh_key("{k-1}")).unwrap();
    let urls: Vec<_> = t.requests().into_iter().map(|r| r.url).collect();
    assert!(urls
        .iter()
        .any(|u| u == "https://api.bitbucket.org/2.0/users/%7Bu-1%7D/ssh-keys?pagelen=100"));
    assert!(urls
        .iter()
        .any(|u| u == "https://api.bitbucket.org/2.0/users/%7Bu-1%7D/ssh-keys/%7Bk-1%7D"));
}

#[test]
fn upload_ssh_key_posts_label_and_key() {
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "/ssh-keys",
                201,
                r#"{"uuid":"{k-2}","key":"ssh-ed25519 BBBB","label":"new"}"#,
            )
            .on("/2.0/user", 200, ME),
    );
    let c = client_for(
        profile("a@b"),
        "tok".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    let key = block_on(c.upload_ssh_key("new", " ssh-ed25519 BBBB\n")).unwrap();
    assert_eq!(key.id, "{k-2}");
    let post = t
        .requests()
        .into_iter()
        .find(|r| r.method.as_str() == "POST")
        .unwrap();
    let body: serde_json::Value = serde_json::from_str(post.body.as_deref().unwrap()).unwrap();
    assert_eq!(body["label"], "new");
    assert_eq!(body["key"], "ssh-ed25519 BBBB");
}

#[test]
fn issues_are_retired() {
    let t = Arc::new(RecordedTransport::new());
    let c = client_for(
        profile("a@b"),
        "tok".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    let want = "Bitbucket issues were retired on 2026-08-20; link a Jira project instead";
    for err in [
        block_on(c.list_issues("ws/repo", &IssueFilter::default())).unwrap_err(),
        block_on(c.create_issue(&NewIssue::default())).unwrap_err(),
    ] {
        assert!(
            matches!(&err, Error::Unsupported(m) if m == want),
            "{err:?}"
        );
    }
    assert!(
        t.requests().is_empty(),
        "no network call for retired issues"
    );
    assert!(!IntegrationKind::Bitbucket.capabilities().issues);
}

#[test]
fn the_spec_has_urls_checkout_and_the_well_known_host() {
    let s = spec_for(ApiFamily::Bitbucket);
    assert_eq!(
        (s.pr_web_url)("https://bitbucket.org", "ws/repo", 4).as_deref(),
        Some("https://bitbucket.org/ws/repo/pull-requests/4")
    );
    assert_eq!(
        (s.new_pr_url)("https://bitbucket.org", "ws/repo", "feat/x y", Some("main")).as_deref(),
        Some("https://bitbucket.org/ws/repo/pull-requests/new?source=feat/x%20y&dest=main")
    );
    assert_eq!(
        (s.branch_url)("https://bitbucket.org", "ws/repo", "b").as_deref(),
        Some("https://bitbucket.org/ws/repo/branch/b")
    );
    assert_eq!(
        (s.well_known)("bitbucket.org"),
        Some(IntegrationKind::Bitbucket)
    );
    assert_eq!((s.well_known)("bitbucket.example.com"), None);

    let mut pr = block_on(async {
        let body = format!("{{\"values\":[{}]}}", pr_row(7, "OPEN", false, "fork/repo"));
        let t = Arc::new(RecordedTransport::new().on("/pullrequests", 200, &body));
        let c = client_for(
            profile("a@b"),
            "tok".into(),
            t,
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap();
        c.list_pull_requests("ws/repo", &PrFilter::default())
            .await
            .unwrap()
            .remove(0)
    });
    assert_eq!(
        (s.checkout)(&pr),
        CheckoutSource::Branch {
            fork_url: Some("https://bitbucket.org/fork/repo.git".into()),
            branch: "feat".into(),
            local: "pr/7".into()
        }
    );
    pr.source_project = None;
    assert!(matches!(
        (s.checkout)(&pr),
        CheckoutSource::Branch { fork_url: None, .. }
    ));
}
