//! The Jira conversation against recorded answers.

use std::sync::Arc;

use serde_json::{json, Value};

use crate::client::{client_for, HostingClient};
use crate::conversation::{
    ActorKind, IdentityKind, ItemKey, ItemKind, MarkupFlavor, PageCursor, TimelineEntry,
    TimelineOpts,
};
use crate::http::{HttpRequest, Method, RecordedTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::Error;

const ISSUE: &str = include_str!("fixtures/conversation/issue.json");
const COMMENTS: &str = include_str!("fixtures/conversation/comments.json");
const CREATED: &str = include_str!("fixtures/conversation/comment_created.json");
const MYSELF_CLOUD: &str = include_str!("fixtures/conversation/myself_cloud.json");
const MYSELF_DC: &str = include_str!("fixtures/conversation/myself_dc.json");

const V2: &str = "https://acme.atlassian.net/rest/api/2";

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn cloud(t: &Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    let mut i = Integration::new(
        "jc",
        IntegrationKind::JiraCloud,
        "https://acme.atlassian.net/",
    );
    i.account = "me@acme.com".into();
    client_for(
        i,
        "tok".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap()
}

fn dc(t: &Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    let i = Integration::new("jd", IntegrationKind::JiraDataCenter, "https://jira.corp");
    client_for(
        i,
        "pat".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap()
}

fn key() -> ItemKey {
    ItemKey {
        kind: ItemKind::Issue,
        number: 7,
        id: "ABC-7".into(),
    }
}

fn body_of(r: &HttpRequest) -> Value {
    serde_json::from_str(r.body.as_deref().unwrap()).unwrap()
}

#[test]
fn an_issue_maps_comments_wiki_bodies_bots_internal_and_the_changelog() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/issue/ABC-7/comment", 200, COMMENTS)
            .on("/issue/ABC-7?", 200, ISSUE),
    );
    let page =
        block_on(cloud(&t).item_timeline("ABC", &key(), None, &TimelineOpts::default())).unwrap();

    let h = &page.header;
    assert_eq!(h.reference, "ABC-7");
    assert_eq!(h.key.number, 7);
    assert_eq!(h.state_label, "Done");
    assert!(!h.open);
    assert_eq!(h.labels, ["bug", "ui"]);
    assert_eq!(h.author.display, "Ann Lee");
    assert_eq!(h.web_url, "https://acme.atlassian.net/browse/ABC-7");
    assert_eq!(h.body.flavor, MarkupFlavor::Gfm);
    assert!(h.body.text.contains("## Steps"), "{}", h.body.text);
    assert!(h.body.text.contains("**bold**"), "{}", h.body.text);
    assert_eq!(page.older, None);
    assert_eq!(page.total_hint, Some(2));

    let shape: Vec<String> = page
        .entries
        .iter()
        .map(|e| match e {
            TimelineEntry::Comment(c) => format!("comment {}", c.id),
            TimelineEntry::Event(e) => format!("event {:?}", e.kind),
            other => format!("{other:?}"),
        })
        .collect();
    assert_eq!(
        shape,
        [
            r#"event StatusChanged { from: "To Do", to: "In Progress" }"#,
            "comment 10",
            r#"event Labeled { added: ["ui"], removed: [] }"#,
            r#"event Assigned { added: ["Bob Ray"], removed: [] }"#,
            r#"event Renamed { from: "Login breaks", to: "Login fails behind a proxy" }"#,
            "comment 11",
            r#"event Other { text: "set resolution to Fixed" }"#,
        ]
    );
    let comment = |id: &str| {
        page.entries
            .iter()
            .find_map(|e| match e {
                TimelineEntry::Comment(c) if c.id == id => Some(c.clone()),
                _ => None,
            })
            .unwrap()
    };
    let plain = comment("10");
    assert_eq!(plain.body.text, "Works on `main`");
    assert!(!plain.edited && !plain.internal);
    assert_eq!(plain.author.login, "a1");
    assert_eq!(
        plain.web_url.as_deref(),
        Some("https://acme.atlassian.net/browse/ABC-7?focusedCommentId=10")
    );
    let bot = comment("11");
    assert_eq!(bot.author.kind, ActorKind::Bot);
    assert!(bot.edited, "updated an hour later");
    assert!(bot.internal, "restricted to a role");
    assert!(bot.body.text.contains("**bold**"), "{}", bot.body.text);

    let urls: Vec<String> = t.requests().into_iter().map(|r| r.url).collect();
    assert_eq!(
        urls,
        [
            format!(
                "{V2}/issue/ABC-7?fields=summary,status,reporter,creator,labels,description,created&expand=changelog"
            ),
            format!("{V2}/issue/ABC-7/comment?startAt=0&maxResults=100&orderBy=-created"),
        ]
    );
}

#[test]
fn events_can_be_left_out_and_the_changelog_is_then_not_asked_for() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/issue/ABC-7/comment", 200, COMMENTS)
            .on("/issue/ABC-7?", 200, ISSUE),
    );
    let page =
        block_on(cloud(&t).item_timeline("ABC", &key(), None, &TimelineOpts { events: false }))
            .unwrap();
    assert_eq!(page.entries.len(), 2);
    assert!(!t.requests()[0].url.contains("changelog"));
}

fn comment_rows(from: usize, count: usize) -> Vec<Value> {
    // Newest first, one minute apart: comment `n` is minute `n`.
    (from..from + count)
        .map(|n| {
            let minute = 449 - n;
            json!({
                "id": format!("{}", 1000 + minute),
                "author": { "accountId": "a1", "displayName": "Ann Lee" },
                "body": format!("c{minute}"),
                "created": format!("2024-05-{:02}T{:02}:{:02}:00.000+0000", 1 + minute / 1440, (minute / 60) % 24, minute % 60),
                "updated": format!("2024-05-{:02}T{:02}:{:02}:00.000+0000", 1 + minute / 1440, (minute / 60) % 24, minute % 60),
            })
        })
        .collect()
}

fn comment_page(start: usize, count: usize, total: usize) -> String {
    json!({ "startAt": start, "maxResults": 100, "total": total,
            "comments": comment_rows(start, count) })
    .to_string()
}

#[test]
fn the_newest_window_is_three_pages_and_older_continues_from_the_cursor() {
    // 450 comments; the issue's changelog holds one event inside the newest
    // window (minute 440) and one in the older one (minute 100).
    let issue = json!({
        "key": "ABC-7",
        "fields": { "summary": "S", "status": { "name": "Open" }, "created": "2024-05-01T00:00:00.000+0000" },
        "changelog": { "histories": [
            { "id": "1", "created": "2024-05-01T07:20:00.000+0000",
              "items": [{ "field": "status", "fromString": "A", "toString": "B" }] },
            { "id": "2", "created": "2024-05-01T01:40:00.000+0000",
              "items": [{ "field": "status", "fromString": "B", "toString": "C" }] }
        ] }
    })
    .to_string();
    let t = Arc::new(
        RecordedTransport::new()
            .on("startAt=0&", 200, &comment_page(0, 100, 450))
            .on("startAt=100&", 200, &comment_page(100, 100, 450))
            .on("startAt=200&", 200, &comment_page(200, 100, 450))
            .on("startAt=300&", 200, &comment_page(300, 100, 450))
            .on("startAt=400&", 200, &comment_page(400, 50, 450))
            .on("/issue/ABC-7?", 200, &issue),
    );
    let c = dc(&t);
    let k = key();
    let first = block_on(c.item_timeline("ABC", &k, None, &TimelineOpts::default())).unwrap();
    let comments = |p: &crate::conversation::TimelinePage| {
        p.entries
            .iter()
            .filter(|e| matches!(e, TimelineEntry::Comment(_)))
            .count()
    };
    assert_eq!(comments(&first), 300);
    // Oldest to newest inside the window: comment 150 first, 449 last.
    let ids: Vec<&str> = first
        .entries
        .iter()
        .filter_map(|e| match e {
            TimelineEntry::Comment(c) => Some(c.body.text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!((ids[0], ids[299]), ("c150", "c449"));
    let events = |p: &crate::conversation::TimelinePage| {
        p.entries
            .iter()
            .filter(|e| matches!(e, TimelineEntry::Event(_)))
            .count()
    };
    assert_eq!(events(&first), 1, "only the event inside the window");
    let cursor = first.older.clone().expect("150 more");
    assert!(cursor.0.starts_with("300|"), "{}", cursor.0);

    let second =
        block_on(c.item_timeline("ABC", &k, Some(&cursor), &TimelineOpts::default())).unwrap();
    assert_eq!(comments(&second), 150);
    assert_eq!(events(&second), 1, "the older event, not the newer one");
    assert_eq!(second.older, None);
    let urls: Vec<String> = t
        .requests()
        .into_iter()
        .map(|r| r.url)
        .filter(|u| u.contains("/comment"))
        .collect();
    assert!(urls[3].contains("startAt=300&"), "{urls:?}");
    assert!(urls[4].contains("startAt=400&"), "{urls:?}");

    assert!(matches!(
        block_on(c.item_timeline(
            "ABC",
            &k,
            Some(&PageCursor("x".into())),
            &TimelineOpts::default()
        )),
        Err(Error::Api { status: 422, .. })
    ));
}

#[test]
fn posting_converts_markdown_to_wiki_and_sends_it_over_v2() {
    let t = Arc::new(RecordedTransport::new().on("/comment", 201, CREATED));
    let c = cloud(&t);
    let posted =
        block_on(c.post_item_comment("ABC", &key(), "**Thanks** for `main`", false)).unwrap();
    assert_eq!(posted.id, "12");
    let sent = t.requests();
    assert_eq!(sent[0].method, Method::Post);
    assert_eq!(sent[0].url, format!("{V2}/issue/ABC-7/comment"));
    assert_eq!(
        body_of(&sent[0]),
        json!({ "body": "*Thanks* for {{main}}" })
    );
}

#[test]
fn the_key_is_built_from_the_project_when_the_item_carries_none() {
    let t = Arc::new(RecordedTransport::new().on("/comment", 201, CREATED));
    block_on(cloud(&t).post_item_comment("ABC", &ItemKey::issue(7), "hi", false)).unwrap();
    assert_eq!(t.requests()[0].url, format!("{V2}/issue/ABC-7/comment"));
}

#[test]
fn editing_puts_and_deleting_deletes_the_comment() {
    let t = Arc::new(RecordedTransport::new().on("/comment/12", 200, CREATED).on(
        "/comment/13",
        204,
        "",
    ));
    let c = dc(&t);
    block_on(c.edit_item_comment("ABC", &key(), "12", "# Title")).unwrap();
    block_on(c.delete_item_comment("ABC", &key(), "13")).unwrap();
    let sent = t.requests();
    assert_eq!(sent[0].method, Method::Put);
    assert_eq!(
        sent[0].url,
        "https://jira.corp/rest/api/2/issue/ABC-7/comment/12"
    );
    assert_eq!(body_of(&sent[0]), json!({ "body": "h1. Title" }));
    assert_eq!(sent[1].method, Method::Delete);
    assert_eq!(
        sent[1].url,
        "https://jira.corp/rest/api/2/issue/ABC-7/comment/13"
    );
    assert_eq!(sent[1].body, None);
}

#[test]
fn what_jira_cannot_do_is_refused_before_anything_is_sent() {
    let t = Arc::new(RecordedTransport::new());
    let c = cloud(&t);
    assert!(matches!(
        block_on(c.post_item_comment("ABC", &key(), "  ", false)),
        Err(Error::Api { status: 422, .. })
    ));
    let long = "x".repeat(32_768);
    assert!(matches!(
        block_on(c.post_item_comment("ABC", &key(), &long, false)),
        Err(Error::Api { status: 422, .. })
    ));
    assert!(matches!(
        block_on(c.post_item_comment("ABC", &key(), "secret", true)),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        block_on(c.reply_in_discussion("ABC", &key(), "1", "hi")),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        block_on(c.item_timeline(
            "ABC",
            &ItemKey::pull_request(1),
            None,
            &TimelineOpts::default()
        )),
        Err(Error::Unsupported(_))
    ));
    assert!(t.requests().is_empty());
}

#[test]
fn capabilities_are_honest() {
    let t = Arc::new(RecordedTransport::new());
    let caps = cloud(&t).conversation_caps(ItemKind::Issue);
    assert!(caps.read && caps.comment && caps.edit_own && caps.delete_own && caps.events);
    assert!(!caps.reply_in_thread && !caps.internal_notes && !caps.resolve && !caps.reactions);
    assert_eq!(caps.write_flavor, MarkupFlavor::JiraWiki);
    assert_eq!(caps.read_flavor, MarkupFlavor::Gfm);
    assert!(!cloud(&t).conversation_caps(ItemKind::PullRequest).read);
}

#[test]
fn identity_is_the_account_on_cloud_and_the_user_name_on_data_center() {
    let t = Arc::new(RecordedTransport::new().on("/myself", 200, MYSELF_CLOUD));
    let me = block_on(cloud(&t).posting_identity("ABC")).unwrap();
    assert_eq!(me.login.as_deref(), Some("a1"));
    assert_eq!(me.display, "Ann Lee");
    assert_eq!(me.kind, IdentityKind::PersonalToken);
    let t = Arc::new(RecordedTransport::new().on("/myself", 200, MYSELF_DC));
    let me = block_on(dc(&t).posting_identity("ABC")).unwrap();
    assert_eq!(me.login.as_deref(), Some("ann"));
    // The login matches the author of a comment, so edit and delete gate.
    let author = crate::conversation::Actor::new("ann", "Ann Lee", ActorKind::User);
    assert_eq!(me.authored(&author), Some(true));
}
