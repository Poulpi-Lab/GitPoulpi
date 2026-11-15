//! The YouTrack conversation against recorded answers.

use std::sync::Arc;

use serde_json::{json, Value};

use crate::client::{client_for, HostingClient};
use crate::conversation::{
    ActorKind, ItemKey, ItemKind, MarkupFlavor, PageCursor, TimelineEntry, TimelineOpts,
};
use crate::http::{HttpRequest, Method, RecordedTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::Error;

const API: &str = "https://acme.youtrack.cloud/api";
const T0: u64 = 1_714_557_600_000; // 2024-05-01T10:00:00Z

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn client(t: &Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    let i = Integration::new(
        "yt",
        IntegrationKind::Youtrack,
        "https://acme.youtrack.cloud/",
    );
    client_for(
        i,
        "perm:token".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap()
}

fn key() -> ItemKey {
    ItemKey {
        kind: ItemKind::Issue,
        number: 12,
        id: "ABC-12".into(),
    }
}

fn body_of(r: &HttpRequest) -> Value {
    serde_json::from_str(r.body.as_deref().unwrap()).unwrap()
}

fn issue() -> String {
    json!({
        "id": "2-12", "idReadable": "ABC-12", "summary": "Crash on start",
        "description": "It **crashes**", "created": T0 - 3_600_000, "resolved": null,
        "reporter": { "id": "1-1", "login": "bob", "fullName": "Bob Ray" },
        "tags": [{ "name": "ui" }],
        "customFields": [{ "name": "State", "value": { "name": "In Progress" } }]
    })
    .to_string()
}

fn comment(n: u64, markdown: bool, deleted: bool) -> Value {
    json!({
        "id": format!("4-{n}"), "text": format!("c{n}"), "usesMarkdown": markdown,
        "created": T0 + n * 60_000, "updated": T0 + n * 60_000, "deleted": deleted,
        "author": { "id": "1-2", "login": "ann", "fullName": "Ann Lee" }
    })
}

#[test]
fn an_issue_maps_comments_flavors_edits_and_skips_deleted_ones() {
    let mut edited = comment(2, true, false);
    edited["updated"] = json!(T0 + 2 * 60_000 + 600_000);
    let rows = json!([
        comment(1, true, false),
        edited,
        comment(3, false, false),
        comment(4, true, true)
    ])
    .to_string();
    let t = Arc::new(RecordedTransport::new().on("/comments?", 200, &rows).on(
        "/issues/ABC-12?",
        200,
        &issue(),
    ));
    let page =
        block_on(client(&t).item_timeline("ABC", &key(), None, &TimelineOpts::default())).unwrap();
    let h = &page.header;
    assert_eq!(h.reference, "ABC-12");
    assert_eq!(h.key.number, 12);
    assert_eq!(h.state_label, "In Progress");
    assert!(h.open);
    assert_eq!(h.author.login, "bob");
    assert_eq!(h.labels, ["ui"]);
    assert_eq!(h.web_url, "https://acme.youtrack.cloud/issue/ABC-12");
    assert_eq!(h.created_at, "2024-05-01T09:00:00Z");
    assert_eq!(page.entries.len(), 3, "the deleted one is gone");
    let c: Vec<_> = page
        .entries
        .iter()
        .map(|e| match e {
            TimelineEntry::Comment(c) => c.clone(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(c[0].body.flavor, MarkupFlavor::YoutrackMd);
    assert_eq!(c[0].created_at, "2024-05-01T10:01:00Z");
    assert!(!c[0].edited && c[1].edited);
    assert_eq!(c[2].body.flavor, MarkupFlavor::PlainText, "legacy wiki");
    assert_eq!(c[0].author.display, "Ann Lee");
    assert_eq!(c[0].author.kind, ActorKind::User);
    assert_eq!(page.older, None);
    let urls: Vec<String> = t.requests().into_iter().map(|r| r.url).collect();
    assert_eq!(urls.len(), 2);
    assert!(urls[1].starts_with(&format!("{API}/issues/ABC-12/comments?fields=")));
    assert!(urls[1].ends_with("&$top=100&$skip=0"), "{}", urls[1]);
}

#[test]
fn a_long_history_is_read_whole_and_windowed() {
    let page_of = |from: u64, count: u64| {
        json!((from..from + count)
            .map(|n| comment(n, true, false))
            .collect::<Vec<_>>())
        .to_string()
    };
    let t = Arc::new(
        RecordedTransport::new()
            .on("$skip=0", 200, &page_of(0, 100))
            .on("$skip=100", 200, &page_of(100, 100))
            .on("$skip=200", 200, &page_of(200, 100))
            .on("$skip=300", 200, &page_of(300, 100))
            .on("$skip=400", 200, &page_of(400, 100))
            .on("$skip=500", 200, &page_of(500, 100))
            .on("$skip=600", 200, &page_of(600, 50))
            .on("/issues/ABC-12?", 200, &issue()),
    );
    let c = client(&t);
    let first = block_on(c.item_timeline("ABC", &key(), None, &TimelineOpts::default())).unwrap();
    assert_eq!(first.entries.len(), 300);
    let ids = |p: &crate::conversation::TimelinePage| -> Vec<String> {
        p.entries
            .iter()
            .map(|e| match e {
                TimelineEntry::Comment(c) => c.body.text.clone(),
                other => panic!("{other:?}"),
            })
            .collect()
    };
    let got = ids(&first);
    assert_eq!((got[0].as_str(), got[299].as_str()), ("c350", "c649"));
    assert_eq!(first.total_hint, Some(650));
    let second = block_on(c.item_timeline(
        "ABC",
        &key(),
        first.older.as_ref(),
        &TimelineOpts::default(),
    ))
    .unwrap();
    assert_eq!(ids(&second)[0], "c50");
    assert_eq!(second.older, Some(PageCursor("50".into())));
}

#[test]
fn post_edit_and_delete_send_the_exact_requests() {
    let created = comment(9, true, false).to_string();
    let t = Arc::new(
        RecordedTransport::new()
            .on("/comments/4-9", 200, &created)
            .on("/comments?", 200, &created),
    );
    let c = client(&t);
    let posted = block_on(c.post_item_comment("ABC", &key(), "Fixed in **1.2**", false)).unwrap();
    assert_eq!(posted.id, "4-9");
    block_on(c.edit_item_comment("ABC", &key(), "4-9", "Updated")).unwrap();
    block_on(c.delete_item_comment("ABC", &key(), "4-9")).unwrap();
    let sent = t.requests();
    assert_eq!(sent[0].method, Method::Post);
    assert!(sent[0]
        .url
        .starts_with(&format!("{API}/issues/ABC-12/comments?fields=")));
    assert_eq!(
        body_of(&sent[0]),
        json!({ "text": "Fixed in **1.2**", "usesMarkdown": true })
    );
    assert_eq!(sent[1].method, Method::Post, "YouTrack edits with POST");
    assert!(sent[1]
        .url
        .starts_with(&format!("{API}/issues/ABC-12/comments/4-9?fields=")));
    assert_eq!(
        body_of(&sent[1]),
        json!({ "text": "Updated", "usesMarkdown": true })
    );
    assert_eq!(sent[2].method, Method::Delete);
    assert_eq!(sent[2].url, format!("{API}/issues/ABC-12/comments/4-9"));
    assert_eq!(sent[2].body, None);
}

#[test]
fn what_youtrack_cannot_do_is_refused_before_anything_is_sent() {
    let t = Arc::new(RecordedTransport::new());
    let c = client(&t);
    assert!(matches!(
        block_on(c.post_item_comment("ABC", &key(), " ", false)),
        Err(Error::Api { status: 422, .. })
    ));
    assert!(matches!(
        block_on(c.post_item_comment("ABC", &key(), "x", true)),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        block_on(c.reply_in_discussion("ABC", &key(), "1", "x")),
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
    let caps = c.conversation_caps(ItemKind::Issue);
    assert!(caps.read && caps.comment && caps.edit_own && caps.delete_own);
    assert!(!caps.events && !caps.reply_in_thread && !caps.internal_notes);
    assert_eq!(caps.write_flavor, MarkupFlavor::YoutrackMd);
    assert!(!c.conversation_caps(ItemKind::PullRequest).read);
}

#[test]
fn identity_is_the_login_of_users_me() {
    let t = Arc::new(RecordedTransport::new().on(
        "/users/me",
        200,
        r#"{"id":"1-2","login":"ann","fullName":"Ann Lee"}"#,
    ));
    let me = block_on(client(&t).posting_identity("ABC")).unwrap();
    assert_eq!(me.login.as_deref(), Some("ann"));
    assert_eq!(me.display, "Ann Lee");
    let author = crate::conversation::Actor::new("ann", "Ann Lee", ActorKind::User);
    assert_eq!(me.authored(&author), Some(true));
}
