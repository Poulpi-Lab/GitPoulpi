//! The Trello conversation against recorded answers.

use std::sync::Arc;

use serde_json::{json, Value};

use crate::client::{client_for, HostingClient};
use crate::conversation::{
    ActorKind, IdentityKind, ItemKey, ItemKind, MarkupFlavor, TimelineEntry, TimelineOpts,
};
use crate::http::{HttpRequest, Method, RecordedTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::Error;

const API: &str = "https://api.trello.com/1";

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn trello(t: &Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    let mut i = Integration::new("tr", IntegrationKind::Trello, "https://trello.com");
    i.account = "KEY123".into();
    client_for(
        i,
        "SECRETTOKEN".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap()
}

fn key() -> ItemKey {
    ItemKey {
        kind: ItemKind::Issue,
        number: 9,
        id: "c9".into(),
    }
}

fn body_of(r: &HttpRequest) -> Value {
    serde_json::from_str(r.body.as_deref().unwrap()).unwrap()
}

const CARD: &str = r#"{"id":"c9","idShort":9,"name":"Fix the login","desc":"It **fails**",
  "url":"https://trello.com/c/Sh9/9-fix","closed":false,"idList":"l2","list":{"name":"Doing"},
  "labels":[{"name":"ui","color":"red"},{"name":"","color":"green"}]}"#;

fn member(user: &str, name: &str) -> Value {
    json!({ "id": "m", "username": user, "fullName": name })
}

/// Newest first, as Trello answers.
fn actions() -> String {
    json!([
        { "id": "a4", "type": "commentCard", "date": "2024-05-02T12:00:00.000Z",
          "memberCreator": member("ann", "Ann Lee"),
          "data": { "text": "Edited text", "dateLastEdited": "2024-05-02T12:30:00.000Z" } },
        { "id": "a3", "type": "updateCard", "date": "2024-05-02T11:00:00.000Z",
          "memberCreator": member("bob", "Bob Ray"),
          "data": { "listBefore": { "name": "To Do" }, "listAfter": { "name": "Doing" } } },
        { "id": "a2", "type": "commentCard", "date": "2024-05-02T10:00:00.000Z",
          "memberCreator": member("bob", "Bob Ray"), "data": { "text": "First" } },
        { "id": "a1b", "type": "updateCard", "date": "2024-05-01T11:00:00.000Z",
          "memberCreator": member("bob", "Bob Ray"),
          "data": { "old": { "closed": true }, "card": { "closed": false } } },
        { "id": "a1a", "type": "updateCard", "date": "2024-05-01T10:30:00.000Z",
          "memberCreator": member("bob", "Bob Ray"),
          "data": { "old": { "closed": false }, "card": { "closed": true } } },
        { "id": "a1", "type": "createCard", "date": "2024-05-01T10:00:00.000Z",
          "memberCreator": member("bob", "Bob Ray"), "data": {} }
    ])
    .to_string()
}

#[test]
fn a_card_maps_comments_moves_archiving_and_the_header() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/cards/c9/actions?", 200, &actions())
            .on("/cards/c9?", 200, CARD),
    );
    let page =
        block_on(trello(&t).item_timeline("B1", &key(), None, &TimelineOpts::default())).unwrap();
    let h = &page.header;
    assert_eq!(h.reference, "#9");
    assert_eq!(h.key.id, "c9");
    assert_eq!(h.state_label, "Doing");
    assert!(h.open);
    assert_eq!(h.author.login, "bob");
    assert_eq!(h.created_at, "2024-05-01T10:00:00.000Z");
    assert_eq!(h.labels, ["ui", "green"]);
    assert_eq!(h.body.flavor, MarkupFlavor::TrelloMd);
    assert_eq!(h.web_url, "https://trello.com/c/Sh9/9-fix");

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
            "event Opened",
            "event Closed",
            "event Reopened",
            "comment a2",
            r#"event StatusChanged { from: "To Do", to: "Doing" }"#,
            "comment a4",
        ],
        "oldest first"
    );
    let TimelineEntry::Comment(edited) = &page.entries[5] else {
        panic!()
    };
    assert!(edited.edited);
    assert_eq!(
        edited.updated_at.as_deref(),
        Some("2024-05-02T12:30:00.000Z")
    );
    assert_eq!(edited.author.kind, ActorKind::User);
    assert_eq!(page.older, None);
    let urls: Vec<String> = t.requests().into_iter().map(|r| r.url).collect();
    assert!(urls[1].starts_with(&format!(
        "{API}/cards/c9/actions?filter=commentCard,updateCard:idList,updateCard:closed,createCard&limit=1000"
    )));
}

#[test]
fn events_can_be_left_out_and_a_card_without_its_id_is_refused() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/cards/c9/actions?", 200, &actions())
            .on("/cards/c9?", 200, CARD),
    );
    let c = trello(&t);
    let page =
        block_on(c.item_timeline("B1", &key(), None, &TimelineOpts { events: false })).unwrap();
    assert_eq!(page.entries.len(), 2);
    let before = t.requests().len();
    assert!(matches!(
        block_on(c.item_timeline("B1", &ItemKey::issue(9), None, &TimelineOpts::default())),
        Err(Error::Api { status: 422, .. })
    ));
    assert_eq!(t.requests().len(), before);
}

#[test]
fn post_edit_and_delete_send_the_exact_requests() {
    let created = json!({ "id": "a9", "type": "commentCard", "date": "2024-05-03T10:00:00.000Z",
        "memberCreator": member("ann", "Ann Lee"), "data": { "text": "Fixed in **1.2**" } })
    .to_string();
    let t = Arc::new(
        RecordedTransport::new()
            .on("/actions/a9/text", 200, &created)
            .on("/actions/a9", 200, "{}")
            .on("/actions/comments", 200, &created),
    );
    let c = trello(&t);
    let posted = block_on(c.post_item_comment("B1", &key(), "Fixed in **1.2**", false)).unwrap();
    assert_eq!(posted.id, "a9");
    assert_eq!(posted.body.text, "Fixed in **1.2**");
    block_on(c.edit_item_comment("B1", &key(), "a9", "Updated")).unwrap();
    block_on(c.delete_item_comment("B1", &key(), "a9")).unwrap();
    let sent = t.requests();
    assert_eq!(sent[0].method, Method::Post);
    assert_eq!(sent[0].url, format!("{API}/cards/c9/actions/comments"));
    assert_eq!(body_of(&sent[0]), json!({ "text": "Fixed in **1.2**" }));
    assert_eq!(sent[1].method, Method::Put);
    assert_eq!(sent[1].url, format!("{API}/actions/a9/text"));
    assert_eq!(body_of(&sent[1]), json!({ "value": "Updated" }));
    assert_eq!(sent[2].method, Method::Delete);
    assert_eq!(sent[2].url, format!("{API}/actions/a9"));
    assert!(
        sent.iter().all(|r| !r.url.contains("SECRETTOKEN")),
        "the token never travels in a URL"
    );
}

#[test]
fn what_trello_cannot_do_is_refused_before_anything_is_sent() {
    let t = Arc::new(RecordedTransport::new());
    let c = trello(&t);
    assert!(matches!(
        block_on(c.post_item_comment("B1", &key(), " ", false)),
        Err(Error::Api { status: 422, .. })
    ));
    assert!(matches!(
        block_on(c.post_item_comment("B1", &key(), &"x".repeat(16_385), false)),
        Err(Error::Api { status: 422, .. })
    ));
    assert!(matches!(
        block_on(c.post_item_comment("B1", &key(), "x", true)),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        block_on(c.reply_in_discussion("B1", &key(), "a", "x")),
        Err(Error::Unsupported(_))
    ));
    assert!(t.requests().is_empty());
    let caps = c.conversation_caps(ItemKind::Issue);
    assert!(caps.read && caps.comment && caps.edit_own && caps.delete_own && caps.events);
    assert!(!caps.reply_in_thread && !caps.internal_notes);
    assert_eq!(caps.max_body, Some(16_384));
    assert!(!c.conversation_caps(ItemKind::PullRequest).read);
}

#[test]
fn identity_is_the_member_username() {
    let t = Arc::new(RecordedTransport::new().on(
        "/members/me",
        200,
        r#"{"id":"m1","username":"ann","fullName":"Ann Lee"}"#,
    ));
    let me = block_on(trello(&t).posting_identity("B1")).unwrap();
    assert_eq!(me.login.as_deref(), Some("ann"));
    assert_eq!(me.display, "Ann Lee");
    assert_eq!(me.kind, IdentityKind::PersonalToken);
}
