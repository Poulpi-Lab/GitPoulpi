//! The GitLab conversation against recorded answers.

use std::sync::Arc;

use super::GitLabClient;
use crate::client::HostingClient;
use crate::conversation::{
    ActorKind, EventKind, IdentityKind, ItemKey, ItemKind, PageCursor, TimelineEntry,
    TimelineOpts, TokenSource,
};
use crate::http::{Method, RecordedTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::review::model::DiffSide;
use crate::scope_fake::FakeScopes;
use crate::Error;

const ISSUE: &str = include_str!("fixtures/conversation/issue.json");
const MR: &str = include_str!("fixtures/conversation/mr.json");
const DISCUSSIONS_ISSUE: &str = include_str!("fixtures/conversation/discussions_issue.json");
const DISCUSSIONS_MR: &str = include_str!("fixtures/conversation/discussions_mr.json");
const NOTE_CREATED: &str = include_str!("fixtures/conversation/note_created.json");
const USER: &str = include_str!("fixtures/conversation/user.json");
const USER_BOT: &str = include_str!("fixtures/conversation/user_bot.json");

const API: &str = "https://gitlab.com/api/v4/projects/group%2Fproj";
const PROJECT: &str = "group/proj";

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn client(transport: &Arc<RecordedTransport>) -> GitLabClient {
    GitLabClient::new(
        Integration::new("gl", IntegrationKind::Gitlab, "https://gitlab.com"),
        "tok".into(),
        transport.clone(),
    )
}

fn no_events() -> TimelineOpts {
    TimelineOpts { events: false }
}

#[test]
fn an_issue_maps_discussions_system_notes_bots_edits_and_internal_notes() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("issues/3/discussions", 200, DISCUSSIONS_ISSUE)
            .on("issues/3", 200, ISSUE),
    );
    let page = block_on(client(&t).item_timeline(
        PROJECT,
        &ItemKey::issue(3),
        None,
        &TimelineOpts::default(),
    ))
    .unwrap();
    let h = &page.header;
    assert_eq!(h.reference, "#3");
    assert_eq!(h.state_label, "closed");
    assert!(!h.open);
    assert_eq!(h.labels, ["bug", "auth"]);
    assert_eq!(h.author.display, "Dana Scully");
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
            r#"event Labeled { added: ["bug"], removed: [] }"#,
            "comment 11",
            "comment 12",
            "comment 13",
            "comment 14",
            r#"event Assigned { added: ["erin"], removed: [] }"#,
            r#"event Renamed { from: "Login breaks", to: "Login fails behind a proxy" }"#,
            "event Closed",
        ]
    );

    let comment = |i: usize| match &page.entries[i] {
        TimelineEntry::Comment(c) => c.clone(),
        other => panic!("{other:?}"),
    };
    let edited = comment(1);
    assert!(edited.edited, "updated 20 minutes later");
    assert_eq!(edited.author.kind, ActorKind::User);
    assert_eq!(
        edited.web_url.as_deref(),
        Some("https://gitlab.com/group/proj/-/issues/3#note_11")
    );
    let first = comment(2);
    let reply = comment(3);
    assert_eq!(first.discussion_id.as_deref(), Some("d3"));
    assert_eq!(first.in_reply_to, None);
    assert_eq!(reply.in_reply_to.as_deref(), Some("12"));
    assert_eq!(reply.author.kind, ActorKind::Bot);
    assert!(!reply.edited, "one second apart is not an edit");
    assert!(!first.internal && comment(4).internal);
}

#[test]
fn events_can_be_left_out() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("issues/3/discussions", 200, DISCUSSIONS_ISSUE)
            .on("issues/3", 200, ISSUE),
    );
    let page = block_on(client(&t).item_timeline(PROJECT, &ItemKey::issue(3), None, &no_events()))
        .unwrap();
    assert_eq!(page.entries.len(), 4);
    assert!(page
        .entries
        .iter()
        .all(|e| matches!(e, TimelineEntry::Comment(_))));
}

#[test]
fn a_merge_request_makes_threads_of_positional_discussions() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("merge_requests/5/discussions", 200, DISCUSSIONS_MR)
            .on("merge_requests/5", 200, MR),
    );
    let page = block_on(client(&t).item_timeline(
        PROJECT,
        &ItemKey::pull_request(5),
        None,
        &TimelineOpts::default(),
    ))
    .unwrap();
    assert_eq!(page.header.reference, "!5");
    assert_eq!(page.header.state_label, "merged");
    assert_eq!(page.entries.len(), 4);

    let TimelineEntry::Event(pushed) = &page.entries[0] else {
        panic!()
    };
    assert_eq!(
        pushed.kind,
        EventKind::CommitsPushed {
            count: 3,
            head: None
        }
    );
    let TimelineEntry::Thread(thread) = &page.entries[1] else {
        panic!("{:?}", page.entries[1]);
    };
    assert_eq!(thread.id, "m2");
    assert_eq!(thread.path, "src/net.rs");
    assert_eq!(thread.line, Some(17));
    assert_eq!(thread.side, DiffSide::New);
    assert!(thread.resolvable && !thread.resolved && !thread.outdated);
    assert_eq!(thread.comments.len(), 2);
    assert_eq!(thread.comments[1].in_reply_to.as_deref(), Some("21"));
    assert_eq!(thread.comments[0].discussion_id.as_deref(), Some("m2"));
    let TimelineEntry::Event(mention) = &page.entries[2] else {
        panic!()
    };
    assert_eq!(
        mention.kind,
        EventKind::CrossReferenced {
            from: "merge request !9".into(),
            url: None
        }
    );
    assert!(matches!(
        &page.entries[3],
        TimelineEntry::Event(e) if e.kind == EventKind::Merged
    ));
}

#[test]
fn the_newest_window_is_read_first_and_older_pages_by_cursor() {
    let page_of = |id: u64| {
        format!(
            r#"[{{"id":"d{id}","notes":[{{"id":{id},"body":"c{id}","author":{{"username":"u"}},"created_at":"2024-0{id}-01T00:00:00Z","updated_at":"2024-0{id}-01T00:00:00Z","system":false}}]}}]"#
        )
    };
    let t = Arc::new(
        RecordedTransport::new()
            .on_with_headers(
                "discussions?per_page=100&page=1",
                200,
                &page_of(1),
                &[("X-Total-Pages", "4")],
            )
            .on("discussions?per_page=100&page=2", 200, &page_of(2))
            .on("discussions?per_page=100&page=3", 200, &page_of(3))
            .on("discussions?per_page=100&page=4", 200, &page_of(4))
            .on("issues/3", 200, ISSUE),
    );
    let c = client(&t);
    let first = block_on(c.item_timeline(PROJECT, &ItemKey::issue(3), None, &no_events())).unwrap();
    let ids: Vec<String> = first
        .entries
        .iter()
        .map(|e| match e {
            TimelineEntry::Comment(c) => c.id.clone(),
            _ => String::new(),
        })
        .collect();
    assert_eq!(ids, ["2", "3", "4"]);
    assert_eq!(first.older, Some(PageCursor("1".into())));
    assert_eq!(first.total_hint, Some(301));
    let urls: Vec<String> = t.requests().into_iter().map(|r| r.url).collect();
    assert_eq!(
        urls,
        [
            format!("{API}/issues/3"),
            format!("{API}/issues/3/discussions?per_page=100&page=1"),
            format!("{API}/issues/3/discussions?per_page=100&page=2"),
            format!("{API}/issues/3/discussions?per_page=100&page=3"),
            format!("{API}/issues/3/discussions?per_page=100&page=4"),
        ]
    );
    let second = block_on(c.item_timeline(
        PROJECT,
        &ItemKey::issue(3),
        first.older.as_ref(),
        &no_events(),
    ))
    .unwrap();
    assert_eq!(second.entries.len(), 1);
    assert_eq!(second.older, None);
}

fn body_of(r: &crate::http::HttpRequest) -> serde_json::Value {
    serde_json::from_str(r.body.as_deref().unwrap()).unwrap()
}

#[test]
fn posting_and_replying_send_exactly_one_request_each() {
    let t = Arc::new(RecordedTransport::new().on("/notes", 201, NOTE_CREATED));
    let c = client(&t);
    let made = block_on(c.post_item_comment(PROJECT, &ItemKey::issue(3), "Thanks, will look.", true))
        .unwrap();
    assert_eq!(made.id, "88");
    assert!(made.internal);
    block_on(c.post_item_comment(PROJECT, &ItemKey::issue(3), "hello", false)).unwrap();
    let replied = block_on(c.reply_in_discussion(
        PROJECT,
        &ItemKey::pull_request(5),
        "d3",
        "Squid 5 it is.",
    ))
    .unwrap();
    assert_eq!(replied.discussion_id.as_deref(), Some("d3"));

    let sent = t.requests();
    assert_eq!(sent.len(), 3);
    assert!(sent.iter().all(|r| r.method == Method::Post));
    assert_eq!(sent[0].url, format!("{API}/issues/3/notes"));
    assert_eq!(
        body_of(&sent[0]),
        serde_json::json!({ "body": "Thanks, will look.", "internal": true })
    );
    assert_eq!(body_of(&sent[1]), serde_json::json!({ "body": "hello" }));
    assert_eq!(
        sent[2].url,
        format!("{API}/merge_requests/5/discussions/d3/notes")
    );
    assert_eq!(
        body_of(&sent[2]),
        serde_json::json!({ "body": "Squid 5 it is." })
    );
    assert!(sent[0]
        .headers
        .iter()
        .any(|(k, v)| k == "PRIVATE-TOKEN" && v == "tok"));
}

#[test]
fn empty_comments_are_refused_before_any_request() {
    let t = Arc::new(RecordedTransport::new());
    let c = client(&t);
    assert!(matches!(
        block_on(c.post_item_comment(PROJECT, &ItemKey::issue(3), " ", false)),
        Err(Error::Api { status: 422, .. })
    ));
    assert!(matches!(
        block_on(c.reply_in_discussion(PROJECT, &ItemKey::issue(3), "d", "")),
        Err(Error::Api { status: 422, .. })
    ));
    assert!(t.requests().is_empty());
}

#[test]
fn editing_puts_and_deleting_deletes_the_note() {
    let t = Arc::new(RecordedTransport::new().on("/notes/88", 200, NOTE_CREATED));
    let c = client(&t);
    let edited =
        block_on(c.edit_item_comment(PROJECT, &ItemKey::issue(3), "88", "Updated.")).unwrap();
    assert_eq!(edited.id, "88");
    block_on(c.delete_item_comment(PROJECT, &ItemKey::pull_request(5), "88")).unwrap();
    let sent = t.requests();
    assert_eq!(sent[0].method, Method::Put);
    assert_eq!(sent[0].url, format!("{API}/issues/3/notes/88"));
    assert_eq!(body_of(&sent[0]), serde_json::json!({ "body": "Updated." }));
    assert_eq!(sent[1].method, Method::Delete);
    assert_eq!(sent[1].url, format!("{API}/merge_requests/5/notes/88"));
    assert_eq!(sent[1].body, None);
}

#[test]
fn capabilities_say_internal_notes_and_resolution_only_where_they_exist() {
    let t = Arc::new(RecordedTransport::new());
    let c = client(&t);
    let issue = c.conversation_caps(ItemKind::Issue);
    let mr = c.conversation_caps(ItemKind::PullRequest);
    assert!(issue.internal_notes && issue.reply_in_thread && !issue.resolve);
    assert!(mr.resolve && !mr.reactions);
}

#[test]
fn a_personal_token_posts_as_its_user_and_a_bot_token_as_a_bot() {
    let t = Arc::new(RecordedTransport::new().on("/user", 200, USER));
    let who = block_on(client(&t).posting_identity(PROJECT)).unwrap();
    assert_eq!(who.login.as_deref(), Some("dana"));
    assert_eq!(who.kind, IdentityKind::PersonalToken);
    assert_eq!(who.source, TokenSource::Profile { id: "gl".into() });

    let t = Arc::new(RecordedTransport::new().on("/user", 200, USER_BOT));
    let (handle, _) = FakeScopes {
        bot: true,
        ..Default::default()
    }
    .handle();
    let mut c = client(&t);
    c.scopes = handle;
    let who = block_on(c.posting_identity(PROJECT)).unwrap();
    assert_eq!(who.login.as_deref(), Some("project_42_bot_a1b2c3"));
    assert_eq!(who.kind, IdentityKind::ProjectOrGroupBot);
    assert_eq!(who.source, TokenSource::RepoTokenCommand);
    // And its own notes are recognised as its own.
    let me = Some(true);
    let bot_actor = crate::conversation::Actor::new(
        "project_42_bot_a1b2c3",
        "triage-bot",
        ActorKind::Bot,
    );
    assert_eq!(who.authored(&bot_actor), me);
}

#[test]
fn a_rejected_token_is_an_error() {
    let t = Arc::new(RecordedTransport::new().on("/user", 401, r#"{"message":"401 Unauthorized"}"#));
    assert!(matches!(
        block_on(client(&t).posting_identity(PROJECT)),
        Err(Error::Unauthorized { .. })
    ));
}
