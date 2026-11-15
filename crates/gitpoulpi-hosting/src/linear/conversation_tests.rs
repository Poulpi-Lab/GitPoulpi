//! The Linear conversation against a GraphQL fake.

use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

use crate::client::{client_for, HostingClient};
use crate::conversation::{
    ActorKind, EventKind, IdentityKind, ItemKey, ItemKind, MarkupFlavor, PageCursor, TimelineEntry,
    TimelineOpts,
};
use crate::http::{HttpRequest, HttpResponse, HttpTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::Error;

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

type Handler = Box<dyn Fn(&str, &Value) -> Value + Send + Sync>;

struct Fake {
    handler: Handler,
    seen: Mutex<Vec<(String, Value)>>,
}

impl Fake {
    fn new(handler: impl Fn(&str, &Value) -> Value + Send + Sync + 'static) -> Arc<Self> {
        Arc::new(Self {
            handler: Box::new(handler),
            seen: Mutex::new(Vec::new()),
        })
    }

    fn calls(&self) -> Vec<(String, Value)> {
        self.seen.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl HttpTransport for Fake {
    async fn send(&self, request: HttpRequest) -> crate::Result<HttpResponse> {
        let body: Value = serde_json::from_str(request.body.as_deref().unwrap()).unwrap();
        let query = body["query"].as_str().unwrap().to_string();
        self.seen
            .lock()
            .unwrap()
            .push((query.clone(), body["variables"].clone()));
        let data = (self.handler)(&query, &body["variables"]);
        Ok(HttpResponse {
            status: 200,
            headers: Default::default(),
            body: json!({ "data": data }).to_string(),
        })
    }
}

fn client(t: Arc<Fake>) -> Box<dyn HostingClient> {
    let i = Integration::new("ln", IntegrationKind::Linear, "https://linear.app");
    client_for(
        i,
        "lin_api_key".into(),
        t,
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap()
}

fn key() -> ItemKey {
    ItemKey {
        kind: ItemKind::Issue,
        number: 12,
        id: "ENG-12".into(),
    }
}

fn comment(id: &str, minute: u32, extra: Value) -> Value {
    let mut c = json!({
        "id": id, "body": format!("body {id}"),
        "createdAt": format!("2024-05-01T10:{minute:02}:00.000Z"),
        "updatedAt": format!("2024-05-01T10:{minute:02}:00.000Z"),
        "editedAt": null,
        "url": format!("https://linear.app/acme/issue/ENG-12/x#comment-{id}"),
        "user": { "id": "u1", "name": "Ann Lee", "displayName": "ann", "url": "https://linear.app/acme/profiles/ann" },
        "botActor": null, "parent": null
    });
    for (k, v) in extra.as_object().unwrap() {
        c[k] = v.clone();
    }
    c
}

fn issue(comments: Vec<Value>, has_prev: bool, start: &str) -> Value {
    json!({ "issue": {
        "id": "uuid-12", "identifier": "ENG-12", "number": 12,
        "title": "Crash on start", "url": "https://linear.app/acme/issue/ENG-12/crash",
        "description": "It **crashes**.", "createdAt": "2024-05-01T09:00:00.000Z",
        "state": { "name": "Done", "type": "completed" },
        "creator": { "id": "u2", "name": "Bob Ray", "displayName": "bob" },
        "labels": { "nodes": [{ "name": "bug" }] },
        "comments": { "nodes": comments,
            "pageInfo": { "hasPreviousPage": has_prev, "startCursor": start } },
        "history": { "nodes": [
            { "id": "h1", "createdAt": "2024-05-01T10:01:30.000Z",
              "actor": { "id": "u2", "name": "Bob Ray", "displayName": "bob" },
              "fromState": { "name": "Todo" }, "toState": { "name": "In Progress" },
              "addedLabels": [{ "name": "bug" }], "removedLabels": [],
              "fromAssignee": null, "toAssignee": { "name": "Ann Lee", "displayName": "ann" },
              "fromTitle": "Crash", "toTitle": "Crash on start" },
            { "id": "h2", "createdAt": "2024-05-01T11:00:00.000Z", "actor": null,
              "fromState": null, "toState": null, "addedLabels": [], "removedLabels": [],
              "fromAssignee": null, "toAssignee": null, "fromTitle": null, "toTitle": null }
        ] }
    } })
}

#[test]
fn an_issue_maps_comments_replies_bots_edits_and_history() {
    let t = Fake::new(|_, _| {
        issue(
            vec![
                comment("c1", 0, json!({})),
                comment(
                    "c2",
                    5,
                    json!({ "parent": { "id": "c1" }, "editedAt": "2024-05-01T10:09:00.000Z" }),
                ),
                comment(
                    "c3",
                    8,
                    json!({ "user": null, "botActor": { "id": "b1", "name": "Sentry" } }),
                ),
            ],
            false,
            "cur",
        )
    });
    let page =
        block_on(client(t.clone()).item_timeline("ENG", &key(), None, &TimelineOpts::default()))
            .unwrap();
    let h = &page.header;
    assert_eq!(
        (h.reference.as_str(), h.state_label.as_str()),
        ("ENG-12", "Done")
    );
    assert!(!h.open);
    assert_eq!(h.author.login, "bob");
    assert_eq!(h.labels, ["bug"]);
    assert_eq!(h.body.flavor, MarkupFlavor::Gfm);
    assert_eq!(page.older, None);

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
            "comment c1",
            r#"event StatusChanged { from: "Todo", to: "In Progress" }"#,
            r#"event Labeled { added: ["bug"], removed: [] }"#,
            r#"event Assigned { added: ["ann"], removed: [] }"#,
            r#"event Renamed { from: "Crash", to: "Crash on start" }"#,
            "comment c2",
            "comment c3",
        ],
        "the empty history node makes no event"
    );
    let comments: Vec<_> = page
        .entries
        .iter()
        .filter_map(|e| match e {
            TimelineEntry::Comment(c) => Some(c),
            _ => None,
        })
        .collect();
    assert_eq!(comments[0].discussion_id.as_deref(), Some("c1"));
    assert_eq!(comments[0].in_reply_to, None);
    assert_eq!(comments[1].discussion_id.as_deref(), Some("c1"));
    assert_eq!(comments[1].in_reply_to.as_deref(), Some("c1"));
    assert!(comments[1].edited && !comments[0].edited);
    assert_eq!(comments[2].author.kind, ActorKind::Bot);
    assert_eq!(comments[2].author.login, "Sentry");
    assert_eq!(comments[0].author.kind, ActorKind::User);
    assert_eq!(
        t.calls()[0].1,
        json!({ "id": "ENG-12", "before": null }),
        "the identifier goes straight in"
    );
}

#[test]
fn events_can_be_left_out_but_the_window_pages_back_by_cursor() {
    let seen = Arc::new(Mutex::new(Vec::<Value>::new()));
    let log = seen.clone();
    let t = Fake::new(move |q, vars| {
        log.lock().unwrap().push(vars.clone());
        let before = vars["before"].as_str().map(str::to_string);
        let (rows, prev, start) = match (q.contains("IssueConversation"), before.as_deref()) {
            (true, None) => (vec![comment("c5", 50, json!({}))], true, "k1"),
            (false, Some("k1")) => (vec![comment("c4", 40, json!({}))], true, "k2"),
            (false, Some("k2")) => (vec![comment("c3", 30, json!({}))], true, "k3"),
            _ => (vec![comment("c2", 20, json!({}))], false, "k4"),
        };
        if q.contains("IssueConversation") {
            issue(rows, prev, start)
        } else {
            json!({ "issue": { "comments": { "nodes": rows,
                "pageInfo": { "hasPreviousPage": prev, "startCursor": start } } } })
        }
    });
    let c = client(t.clone());
    let first =
        block_on(c.item_timeline("ENG", &key(), None, &TimelineOpts { events: false })).unwrap();
    let ids: Vec<String> = first
        .entries
        .iter()
        .map(|e| match e {
            TimelineEntry::Comment(c) => c.id.clone(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(ids, ["c3", "c4", "c5"], "three rounds, oldest first");
    let cursor = first.older.clone().expect("there is more");
    assert!(cursor.0.starts_with("k3|"), "{}", cursor.0);
    assert_eq!(
        seen.lock()
            .unwrap()
            .iter()
            .map(|v| v["before"].clone())
            .collect::<Vec<_>>(),
        [json!(null), json!("k1"), json!("k2")]
    );

    let second =
        block_on(c.item_timeline("ENG", &key(), Some(&cursor), &TimelineOpts::default())).unwrap();
    let ids: Vec<String> = second
        .entries
        .iter()
        .filter_map(|e| match e {
            TimelineEntry::Comment(c) => Some(c.id.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(ids, ["c2"]);
    assert_eq!(second.older, None);
    assert_eq!(t.calls().last().unwrap().1["before"], json!("k3"));
    assert!(matches!(
        block_on(c.item_timeline(
            "ENG",
            &key(),
            Some(&PageCursor(String::new())),
            &TimelineOpts::default()
        )),
        Err(Error::Api { status: 422, .. })
    ));
}

fn created(parent: Option<&str>) -> Value {
    let mut c = comment("new1", 1, json!({}));
    if let Some(p) = parent {
        c["parent"] = json!({ "id": p });
    }
    json!({ "success": true, "comment": c })
}

#[test]
fn post_reply_edit_and_delete_send_the_exact_mutations() {
    let t = Fake::new(|q, _| {
        if q.contains("commentCreate") {
            json!({ "commentCreate": created(None) })
        } else if q.contains("commentUpdate") {
            json!({ "commentUpdate": created(None) })
        } else {
            json!({ "commentDelete": { "success": true } })
        }
    });
    let c = client(t.clone());
    let k = key();
    block_on(c.post_item_comment("ENG", &k, "Fixed in **1.2**", false)).unwrap();
    block_on(c.reply_in_discussion("ENG", &k, "c1", "Thanks")).unwrap();
    let edited = block_on(c.edit_item_comment("ENG", &k, "new1", "Updated")).unwrap();
    assert_eq!(edited.id, "new1");
    block_on(c.delete_item_comment("ENG", &k, "new1")).unwrap();
    let calls = t.calls();
    assert_eq!(
        calls[0].1,
        json!({ "input": { "issueId": "ENG-12", "body": "Fixed in **1.2**" } })
    );
    assert_eq!(
        calls[1].1,
        json!({ "input": { "issueId": "ENG-12", "body": "Thanks", "parentId": "c1" } })
    );
    assert_eq!(
        calls[2].1,
        json!({ "id": "new1", "input": { "body": "Updated" } })
    );
    assert_eq!(calls[3].1, json!({ "id": "new1" }));
}

#[test]
fn a_refusal_or_an_unsupported_ask_sends_what_it_should() {
    let t = Fake::new(|_, _| json!({ "commentCreate": { "success": false, "comment": null } }));
    let c = client(t.clone());
    assert!(matches!(
        block_on(c.post_item_comment("ENG", &key(), "hello", false)),
        Err(Error::Api { status: 422, .. })
    ));
    let before = t.calls().len();
    assert!(matches!(
        block_on(c.post_item_comment("ENG", &key(), "  ", false)),
        Err(Error::Api { status: 422, .. })
    ));
    assert!(matches!(
        block_on(c.post_item_comment("ENG", &key(), "x", true)),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        block_on(c.item_timeline(
            "ENG",
            &ItemKey::pull_request(1),
            None,
            &TimelineOpts::default()
        )),
        Err(Error::Unsupported(_))
    ));
    assert_eq!(t.calls().len(), before, "nothing was sent");
}

#[test]
fn capabilities_and_identity() {
    let t = Fake::new(
        |_, _| json!({ "viewer": { "id": "u1", "name": "Ann Lee", "displayName": "ann" } }),
    );
    let c = client(t);
    let caps = c.conversation_caps(ItemKind::Issue);
    assert!(caps.read && caps.comment && caps.reply_in_thread && caps.edit_own && caps.delete_own);
    assert!(caps.events && !caps.internal_notes && !caps.resolve);
    assert!(!c.conversation_caps(ItemKind::PullRequest).read);
    let me = block_on(c.posting_identity("ENG")).unwrap();
    assert_eq!(me.login.as_deref(), Some("ann"));
    assert_eq!(me.kind, IdentityKind::PersonalToken);
    // What a comment's author carries compares equal, so Edit gates on it.
    let author = crate::conversation::Actor::new("ann", "Ann Lee", ActorKind::User);
    assert_eq!(me.authored(&author), Some(true));
}

#[test]
fn a_missing_issue_is_a_404() {
    let t = Fake::new(|_, _| json!({ "issue": null }));
    assert!(matches!(
        block_on(client(t).item_timeline("ENG", &key(), None, &TimelineOpts::default())),
        Err(Error::Api { status: 404, .. })
    ));
}

#[test]
fn the_history_event_kinds_are_status_labels_assignee_and_title() {
    let node = json!({ "id": "h", "createdAt": "2024-05-01T10:00:00Z",
        "fromState": { "name": "A" }, "toState": { "name": "B" },
        "addedLabels": [{ "name": "x" }], "removedLabels": [{ "name": "y" }],
        "fromAssignee": { "name": "Old", "displayName": "old" }, "toAssignee": null,
        "fromTitle": null, "toTitle": null });
    let kinds: Vec<EventKind> = super::conversation::history_events(&node)
        .into_iter()
        .map(|e| e.kind)
        .collect();
    assert_eq!(
        kinds,
        [
            EventKind::StatusChanged {
                from: "A".into(),
                to: "B".into()
            },
            EventKind::Labeled {
                added: vec!["x".into()],
                removed: vec!["y".into()]
            },
            EventKind::Assigned {
                added: vec![],
                removed: vec!["old".into()]
            },
        ]
    );
}
