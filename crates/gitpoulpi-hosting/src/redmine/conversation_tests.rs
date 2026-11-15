//! The Redmine conversation against recorded answers.

use std::sync::Arc;

use gitpoulpi_config::RedmineMarkup;
use serde_json::{json, Value};

use crate::client::{client_for, HostingClient};
use crate::conversation::{
    ActorKind, IdentityKind, ItemKey, ItemKind, MarkupFlavor, TimelineEntry, TimelineOpts,
};
use crate::http::{HttpRequest, Method, RecordedTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::Error;

const SITE: &str = "http://127.0.0.1:3002";
const STATUSES: &str = r#"{"issue_statuses":[
  {"id":1,"name":"New","is_closed":false},{"id":2,"name":"In Progress","is_closed":false},
  {"id":5,"name":"Closed","is_closed":true}]}"#;

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn client(t: &Arc<RecordedTransport>, markup: RedmineMarkup) -> Box<dyn HostingClient> {
    let mut i = Integration::new("rm", IntegrationKind::Redmine, "http://127.0.0.1:3002/");
    i.markup = markup;
    client_for(
        i,
        "key-123".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap()
}

fn key() -> ItemKey {
    ItemKey::issue(5)
}

fn body_of(r: &HttpRequest) -> Value {
    serde_json::from_str(r.body.as_deref().unwrap()).unwrap()
}

fn issue(journals: Value) -> String {
    json!({ "issue": {
        "id": 5, "subject": "Crash on start", "description": "h2. Steps\n\n* one",
        "status": { "id": 5, "name": "Closed", "is_closed": true },
        "author": { "id": 3, "name": "Bob Ray" },
        "created_on": "2024-05-01T08:00:00Z",
        "journals": journals
    } })
    .to_string()
}

fn journals() -> Value {
    json!([
        { "id": 11, "user": { "id": 3, "name": "Bob Ray" }, "notes": "*Thanks*, see \"the docs\":https://x.io",
          "created_on": "2024-05-01T10:00:00Z", "private_notes": false,
          "details": [ { "property": "attr", "name": "status_id", "old_value": "1", "new_value": "2" } ] },
        { "id": 12, "user": { "id": 4, "name": "Ann Lee" }, "notes": "",
          "created_on": "2024-05-01T11:00:00Z",
          "details": [
            { "property": "attr", "name": "status_id", "old_value": "2", "new_value": "5" },
            { "property": "attr", "name": "subject", "old_value": "Crash", "new_value": "Crash on start" },
            { "property": "attr", "name": "assigned_to_id", "old_value": null, "new_value": "4" },
            { "property": "attr", "name": "priority_id", "old_value": "2", "new_value": "3" } ] },
        { "id": 13, "user": { "id": 4, "name": "Ann Lee" }, "notes": "Only for the team",
          "created_on": "2024-05-01T12:00:00Z", "updated_on": "2024-05-01T12:30:00Z",
          "private_notes": true, "details": [] }
    ])
}

#[test]
fn journals_become_comments_and_events_with_textile_read_as_markdown() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/issue_statuses.json", 200, STATUSES)
            .on("/issues/5.json?include=journals", 200, &issue(journals())),
    );
    let page = block_on(client(&t, RedmineMarkup::Textile).item_timeline(
        "bench",
        &key(),
        None,
        &TimelineOpts::default(),
    ))
    .unwrap();
    let h = &page.header;
    assert_eq!(h.reference, "#5");
    assert!(!h.open);
    assert_eq!(h.state_label, "Closed");
    assert_eq!(h.body.text, "## Steps\n\n- one");
    assert_eq!(h.web_url, format!("{SITE}/issues/5"));
    assert_eq!(h.author.login, "3");

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
            "comment 11",
            r#"event StatusChanged { from: "New", to: "In Progress" }"#,
            "event Closed",
            r#"event Renamed { from: "Crash", to: "Crash on start" }"#,
            r#"event Other { text: "changed the assignee" }"#,
            r#"event Other { text: "changed priority from 2 to 3" }"#,
            "comment 13",
        ]
    );
    let TimelineEntry::Comment(c) = &page.entries[0] else {
        panic!()
    };
    assert_eq!(c.body.text, "**Thanks**, see [the docs](https://x.io)");
    assert_eq!(c.body.flavor, MarkupFlavor::Gfm);
    assert_eq!(c.author.kind, ActorKind::User);
    assert!(!c.internal && !c.edited);
    assert_eq!(
        c.web_url.as_deref(),
        Some("http://127.0.0.1:3002/issues/5#note-1")
    );
    let TimelineEntry::Comment(private) = &page.entries[6] else {
        panic!()
    };
    assert!(private.internal && private.edited);
}

#[test]
fn a_markdown_server_is_read_as_it_is_and_events_can_be_left_out() {
    let t = Arc::new(RecordedTransport::new().on(
        "/issues/5.json?include=journals",
        200,
        &issue(journals()),
    ));
    let page = block_on(client(&t, RedmineMarkup::Markdown).item_timeline(
        "bench",
        &key(),
        None,
        &TimelineOpts { events: false },
    ))
    .unwrap();
    assert_eq!(page.entries.len(), 2, "the journal of changes only is gone");
    let TimelineEntry::Comment(c) = &page.entries[0] else {
        panic!()
    };
    assert_eq!(c.body.text, "*Thanks*, see \"the docs\":https://x.io");
    assert_eq!(page.header.body.text, "h2. Steps\n\n* one");
    assert_eq!(t.requests().len(), 1, "no status table without events");
}

#[test]
fn the_newest_window_is_cut_locally() {
    let many: Vec<Value> = (1..=350u64)
        .map(|n| {
            json!({ "id": n, "user": { "id": 1, "name": "A" }, "notes": format!("n{n}"),
                    "created_on": "2024-05-01T10:00:00Z", "details": [] })
        })
        .collect();
    let t = Arc::new(RecordedTransport::new().on("/issues/5.json", 200, &issue(json!(many))));
    let c = client(&t, RedmineMarkup::Markdown);
    let first = block_on(c.item_timeline("b", &key(), None, &TimelineOpts::default())).unwrap();
    assert_eq!(first.entries.len(), 300);
    assert_eq!(first.total_hint, Some(350));
    let second =
        block_on(c.item_timeline("b", &key(), first.older.as_ref(), &TimelineOpts::default()))
            .unwrap();
    assert_eq!(second.entries.len(), 50);
    assert_eq!(second.older, None);
}

fn after_post() -> String {
    issue(json!([
        { "id": 20, "user": { "id": 3, "name": "Bob Ray" }, "notes": "h1. Title\n\nsome *bold* text",
          "created_on": "2024-05-02T10:00:00Z", "private_notes": true, "details": [] }
    ]))
}

#[test]
fn posting_sends_textile_with_a_put_then_reads_the_comment_back() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/issues/5.json?include=journals", 200, &after_post())
            .on("/issues/5.json", 204, ""),
    );
    let c = client(&t, RedmineMarkup::Textile);
    let posted =
        block_on(c.post_item_comment("bench", &key(), "# Title\n\nsome **bold** text", true))
            .unwrap();
    assert_eq!(posted.id, "20");
    assert!(posted.internal);
    let sent = t.requests();
    assert_eq!(sent[0].method, Method::Put);
    assert_eq!(sent[0].url, format!("{SITE}/issues/5.json"));
    assert_eq!(
        body_of(&sent[0]),
        json!({ "issue": { "notes": "h1. Title\n\nsome *bold* text", "private_notes": true } })
    );
    assert_eq!(sent[1].method, Method::Get);
    assert_eq!(
        sent[1].url,
        format!("{SITE}/issues/5.json?include=journals")
    );
}

#[test]
fn a_markdown_server_gets_the_markdown_untouched() {
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "/issues/5.json?include=journals",
                200,
                &issue(
                    json!([{ "id": 21, "user": { "id": 3, "name": "B" }, "notes": "**hi**",
                    "created_on": "2024-05-02T10:00:00Z", "details": [] }]),
                ),
            )
            .on("/issues/5.json", 204, ""),
    );
    let c = client(&t, RedmineMarkup::Markdown);
    block_on(c.post_item_comment("bench", &key(), "**hi**", false)).unwrap();
    assert_eq!(
        body_of(&t.requests()[0]),
        json!({ "issue": { "notes": "**hi**" } }),
        "no private_notes unless asked"
    );
}

#[test]
fn editing_puts_the_journal_and_reads_it_back() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/issues/5.json?include=journals", 200, &issue(journals()))
            .on("/journals/13.json", 204, ""),
    );
    let c = client(&t, RedmineMarkup::Markdown);
    let edited = block_on(c.edit_item_comment("bench", &key(), "13", "New text")).unwrap();
    assert_eq!(edited.id, "13");
    let sent = t.requests();
    assert_eq!(sent[0].method, Method::Put);
    assert_eq!(sent[0].url, format!("{SITE}/journals/13.json"));
    assert_eq!(
        body_of(&sent[0]),
        json!({ "journal": { "notes": "New text" } })
    );
    assert!(matches!(
        block_on(c.edit_item_comment("bench", &key(), "abc", "x")),
        Err(Error::Api { status: 422, .. })
    ));
}

#[test]
fn what_redmine_cannot_do_is_unsupported_and_capabilities_say_so() {
    let t = Arc::new(RecordedTransport::new());
    let c = client(&t, RedmineMarkup::Textile);
    assert!(matches!(
        block_on(c.delete_item_comment("b", &key(), "13")),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        block_on(c.reply_in_discussion("b", &key(), "13", "x")),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        block_on(c.post_item_comment("b", &key(), "  ", false)),
        Err(Error::Api { status: 422, .. })
    ));
    assert!(matches!(
        block_on(c.item_timeline(
            "b",
            &ItemKey::pull_request(1),
            None,
            &TimelineOpts::default()
        )),
        Err(Error::Unsupported(_))
    ));
    assert!(t.requests().is_empty());
    let caps = c.conversation_caps(ItemKind::Issue);
    assert!(caps.read && caps.comment && caps.edit_own && caps.events && caps.internal_notes);
    assert!(!caps.delete_own && !caps.reply_in_thread);
    assert_eq!(caps.write_flavor, MarkupFlavor::Textile);
    let md = client(&t, RedmineMarkup::Markdown).conversation_caps(ItemKind::Issue);
    assert_eq!(md.write_flavor, MarkupFlavor::Gfm);
}

#[test]
fn identity_is_the_numeric_user_id_shown_by_name() {
    let t = Arc::new(RecordedTransport::new().on(
        "/users/current.json",
        200,
        r#"{"user":{"id":4,"login":"ann","firstname":"Ann","lastname":"Lee"}}"#,
    ));
    let me = block_on(client(&t, RedmineMarkup::Textile).posting_identity("b")).unwrap();
    assert_eq!(me.login.as_deref(), Some("4"));
    assert_eq!(me.display, "Ann Lee");
    assert_eq!(me.kind, IdentityKind::PersonalToken);
    // A journal's user carries the same id, so Edit gates on it.
    let author = crate::conversation::Actor::new("4", "Ann Lee", ActorKind::User);
    assert_eq!(me.authored(&author), Some(true));
}
