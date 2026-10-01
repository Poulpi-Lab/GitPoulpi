use std::sync::Arc;

use crate::client::{client_for, HostingClient, NewIssue};
use crate::http::{HttpRequest, Method, RecordedTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::model::{IssueFilter, IssueState, StateFilter};

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

fn trello(transport: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    let mut i = Integration::new("tr", IntegrationKind::Trello, "https://trello.com");
    i.account = "KEY123".into();
    client_for(i, "SECRETTOKEN".into(), transport)
}

const LISTS: &str =
    r#"[{"id":"l1","name":"To Do"},{"id":"l2","name":"Doing"},{"id":"l3","name":"Done"}]"#;

fn card(n: u64, list: &str, closed: bool) -> String {
    format!(
        r#"{{"id":"c{n}","name":"Card {n}","idShort":{n},"shortUrl":"https://trello.com/c/Sh{n}",
        "url":"https://trello.com/c/Sh{n}/{n}-card","closed":{closed},"idList":"{list}",
        "idMembers":["m1"],"members":[{{"id":"m1","fullName":"Ann A","username":"ann"}}],
        "labels":[{{"name":"ui","color":"red"}},{{"name":"","color":"green"}}],
        "dateLastActivity":"2026-10-01T10:00:00.000Z"}}"#
    )
}

#[test]
fn the_oauth_header_carries_key_and_token_and_no_url_does() {
    let body = r#"{"id":"m1","username":"ann","fullName":"Ann A","url":"https://trello.com/ann"}"#;
    let t = Arc::new(RecordedTransport::new().on("/1/members/me", 200, body));
    let check = block_on(trello(t.clone()).check_token()).unwrap();
    let reqs = t.requests();
    assert_eq!(reqs.len(), 1, "check_token is one call");
    assert!(reqs[0]
        .url
        .starts_with("https://api.trello.com/1/members/me?"));
    assert_eq!(
        header(&reqs[0], "Authorization"),
        Some(r#"OAuth oauth_consumer_key="KEY123", oauth_token="SECRETTOKEN""#)
    );
    assert_eq!(check.user.login, "ann");
    assert_eq!(check.user.name, "Ann A");
    assert_eq!(check.user.id, "m1");
    assert!(check.scopes.is_none());
}

#[test]
fn boards_map_to_short_links_and_no_url_ever_carries_a_credential() {
    let boards = r#"[{"id":"b1","name":"Roadmap","shortLink":"AbCd1234","url":"https://trello.com/b/AbCd1234/roadmap","desc":"","prefs":{"permissionLevel":"private"}}]"#;
    let t = Arc::new(
        RecordedTransport::new()
            .on("/boards/AbCd1234/lists", 200, LISTS)
            .on(
                "/boards/AbCd1234/cards/",
                200,
                &format!("[{}]", card(1, "l1", false)),
            )
            .on("/boards/AbCd1234/lists", 200, LISTS)
            .on(
                "/cards",
                200,
                r#"{"idShort":9,"shortUrl":"https://trello.com/c/New9"}"#,
            )
            .on("/members/me/boards", 200, boards),
    );
    let c = trello(t.clone());
    let projects = block_on(c.list_projects()).unwrap();
    assert_eq!(projects[0].full_name, "AbCd1234");
    assert_eq!(projects[0].name, "Roadmap");
    assert_eq!(projects[0].id, "b1");
    assert_eq!(
        projects[0].web_url.as_deref(),
        Some("https://trello.com/b/AbCd1234/roadmap")
    );
    assert!(projects[0].private);
    assert!(projects[0].description.is_none());
    assert!(t.requests()[0].url.contains("filter=open"));

    block_on(c.list_issues("AbCd1234", &IssueFilter::default())).unwrap();
    block_on(c.create_issue(&NewIssue {
        project: "AbCd1234".into(),
        title: "t".into(),
        list_id: Some("l1".into()),
        ..Default::default()
    }))
    .unwrap();
    for r in t.requests() {
        let lower = r.url.to_lowercase();
        assert!(
            !lower.contains("token=") && !lower.contains("key="),
            "credential in URL: {}",
            r.url
        );
        assert!(!r.url.contains("SECRETTOKEN") && !r.url.contains("KEY123"));
        assert!(header(&r, "Authorization").is_some_and(|h| h.starts_with("OAuth ")));
    }
}

#[test]
fn cards_are_fetched_trimmed_and_mapped() {
    let cards = format!("[{},{}]", card(7, "l2", false), card(8, "l3", true));
    let t = Arc::new(
        RecordedTransport::new()
            .on("/boards/B1/lists", 200, LISTS)
            .on("/boards/B1/cards/all", 200, &cards),
    );
    let rows = block_on(trello(t.clone()).list_issues(
        "B1",
        &IssueFilter {
            state: StateFilter::All,
            ..Default::default()
        },
    ))
    .unwrap();
    let urls: Vec<String> = t.requests().into_iter().map(|r| r.url).collect();
    assert_eq!(urls.len(), 2);
    assert!(urls[1].starts_with("https://api.trello.com/1/boards/B1/cards/all?fields="));
    assert!(urls[1].contains("members=true") && urls[1].contains("limit=1000"));
    assert!(urls[1].contains("fields=name,idShort,shortUrl,"));
    let first = &rows[0];
    assert_eq!(first.number, 7);
    assert_eq!(first.id, "c7");
    assert_eq!(first.reference, "https://trello.com/c/Sh7");
    assert_eq!(first.web_url, "https://trello.com/c/Sh7");
    assert_eq!(first.iid_label, "#7");
    assert_eq!(first.kind_label.as_deref(), Some("Doing"));
    assert_eq!(first.title, "Card 7");
    assert_eq!(first.assignees, vec!["Ann A"]);
    assert_eq!(first.labels, vec!["ui", "green"]);
    assert_eq!(first.state, IssueState::Open);
    assert_eq!(first.updated_at, "2026-10-01T10:00:00.000Z");
    assert_eq!(rows[1].state, IssueState::Closed);
}

#[test]
fn the_state_picks_the_cards_filter_segment() {
    for (state, segment) in [
        (StateFilter::Open, "/cards/open?"),
        (StateFilter::Closed, "/cards/closed?"),
    ] {
        let t = Arc::new(
            RecordedTransport::new()
                .on("/lists", 200, LISTS)
                .on("/cards/", 200, "[]"),
        );
        block_on(trello(t.clone()).list_issues(
            "B1",
            &IssueFilter {
                state,
                ..Default::default()
            },
        ))
        .unwrap();
        assert!(
            t.requests()[1].url.contains(segment),
            "{}",
            t.requests()[1].url
        );
    }
}

#[test]
fn the_filter_is_a_list_of_list_names_resolved_through_the_board_lists() {
    let cards = format!(
        "[{},{},{}]",
        card(1, "l1", false),
        card(2, "l2", false),
        card(3, "l3", false)
    );
    let t = Arc::new(
        RecordedTransport::new()
            .on("/boards/B1/lists", 200, LISTS)
            .on("/boards/B1/cards/", 200, &cards),
    );
    let rows = block_on(trello(t.clone()).list_issues(
        "B1",
        &IssueFilter {
            query: Some(" doing , TO DO ".into()),
            ..Default::default()
        },
    ))
    .unwrap();
    let numbers: Vec<u64> = rows.iter().map(|r| r.number).collect();
    assert_eq!(numbers, [1, 2]);
    assert!(t.requests()[0].url.contains("/boards/B1/lists?filter=open"));
}

#[test]
fn assigned_to_me_keeps_only_my_cards() {
    let mut other = card(2, "l1", false);
    other = other.replace(r#""idMembers":["m1"]"#, r#""idMembers":["m9"]"#);
    let cards = format!("[{},{}]", card(1, "l1", false), other);
    let t = Arc::new(
        RecordedTransport::new()
            .on("/lists", 200, LISTS)
            .on("/members/me", 200, r#"{"id":"m1","username":"ann"}"#)
            .on("/cards/", 200, &cards),
    );
    let rows = block_on(trello(t).list_issues(
        "B1",
        &IssueFilter {
            assigned_to_me: true,
            ..Default::default()
        },
    ))
    .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].number, 1);
}

#[test]
fn create_options_are_the_open_lists_first_is_the_default() {
    let t = Arc::new(RecordedTransport::new().on("/boards/B1/lists", 200, LISTS));
    let options = block_on(trello(t).issue_create_options("B1")).unwrap();
    let ids: Vec<&str> = options.lists.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, ["l1", "l2", "l3"]);
    assert_eq!(options.default_list.as_deref(), Some("l1"));
    assert!(options.types.is_empty() && options.default_type.is_none());
}

#[test]
fn create_issue_posts_the_card_to_the_chosen_or_default_list() {
    let created = r#"{"id":"c9","idShort":9,"shortUrl":"https://trello.com/c/New9","url":"https://trello.com/c/New9/9-x"}"#;
    let t = Arc::new(RecordedTransport::new().on("/1/cards", 200, created));
    let made = block_on(trello(t.clone()).create_issue(&NewIssue {
        project: "B1".into(),
        title: "Fix it".into(),
        body: "details".into(),
        list_id: Some("l2".into()),
        ..Default::default()
    }))
    .unwrap();
    assert_eq!(made.number, 9);
    assert_eq!(made.reference, "https://trello.com/c/New9");
    assert_eq!(made.web_url, "https://trello.com/c/New9");
    let req = &t.requests()[0];
    assert_eq!(req.method, Method::Post);
    assert_eq!(req.url, "https://api.trello.com/1/cards");
    let body: serde_json::Value = serde_json::from_str(req.body.as_deref().unwrap()).unwrap();
    assert_eq!(
        body,
        serde_json::json!({"idList":"l2","name":"Fix it","desc":"details"})
    );

    // No list given: the board's first open list.
    let t = Arc::new(
        RecordedTransport::new()
            .on("/boards/B1/lists", 200, LISTS)
            .on("/1/cards", 200, created),
    );
    block_on(trello(t.clone()).create_issue(&NewIssue {
        project: "B1".into(),
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
    assert_eq!(body["idList"], "l1");
    assert!(body.get("desc").is_none());
}

#[test]
fn api_token_limit_exceeded_is_a_rate_limit_error() {
    let t = Arc::new(RecordedTransport::new().on(
        "/members/me",
        429,
        r#"{"error":"API_TOKEN_LIMIT_EXCEEDED","message":"Rate limit exceeded"}"#,
    ));
    let err = block_on(trello(t).check_token()).unwrap_err();
    assert!(matches!(err, crate::Error::RateLimited { .. }), "{err:?}");
}

#[test]
fn a_rejected_token_is_unauthorized() {
    let t = Arc::new(RecordedTransport::new().on("/members/me", 401, "invalid token"));
    let err = block_on(trello(t).check_token()).unwrap_err();
    assert!(matches!(err, crate::Error::Unauthorized { .. }), "{err}");
}

#[test]
fn there_are_no_pull_requests_or_ssh_keys() {
    let t = Arc::new(RecordedTransport::new());
    let c = trello(t.clone());
    assert!(matches!(
        block_on(c.list_pull_requests("B1", &Default::default())),
        Err(crate::Error::Unsupported(_))
    ));
    assert!(matches!(
        block_on(c.list_ssh_keys()),
        Err(crate::Error::Unsupported(_))
    ));
    assert!(t.requests().is_empty());
}

#[test]
fn the_spec_links_a_short_url_or_a_bare_short_link() {
    use crate::spec::spec_for;
    let spec = spec_for(gitpoulpi_config::ApiFamily::Trello);
    assert_eq!(
        (spec.issue_web_url)("https://trello.com", "B", "https://trello.com/c/Ab").as_deref(),
        Some("https://trello.com/c/Ab")
    );
    assert_eq!(
        (spec.issue_web_url)("https://trello.com/", "B", "Ab").as_deref(),
        Some("https://trello.com/c/Ab")
    );
    assert_eq!((spec.issue_web_url)("https://x", "B", " ").as_deref(), None);
    assert!(IntegrationKind::IMPLEMENTED.contains(&IntegrationKind::Trello));
}

#[test]
fn moving_a_card_puts_it_in_the_list_named_like_the_column() {
    use crate::board::{columns, BoardSource, BoardSpec};
    let t = Arc::new(
        RecordedTransport::new()
            .on("/lists?filter=open", 200, LISTS)
            .on("/cards/c1", 200, "{}"),
    );
    let c = trello(t.clone());
    assert_eq!(c.board_source(), BoardSource::Lists);
    let names = block_on(c.issue_statuses("B1")).unwrap();
    assert_eq!(names, ["To Do", "Doing", "Done"]);
    let spec = BoardSpec {
        source: BoardSource::Lists,
        declared: names,
        labels: Vec::new(),
    };
    let cols = columns(&spec, &[]);
    let issue = crate::model::IssueSummary {
        number: 1,
        id: "c1".into(),
        kind_label: Some("To Do".into()),
        ..Default::default()
    };
    block_on(c.move_issue("B1", &issue, &spec, &cols[1])).unwrap();
    let reqs = t.requests();
    let put = reqs.last().unwrap();
    assert_eq!(put.method, Method::Put);
    assert!(put.url.ends_with("/1/cards/c1"), "{}", put.url);
    let body: serde_json::Value = serde_json::from_str(put.body.as_deref().unwrap()).unwrap();
    assert_eq!(body, serde_json::json!({"idList": "l2"}));
}
