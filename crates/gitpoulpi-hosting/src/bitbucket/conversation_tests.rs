//! The Bitbucket Cloud and Data Center conversations against recorded
//! answers: mapping, pagination, the exact requests of post / reply / edit /
//! delete, issues refused, and identity.

use std::sync::Arc;

use super::dc::BitbucketDcClient;
use super::BitbucketClient;
use crate::client::HostingClient;
use crate::conversation::{
    ActorKind, EventKind, IdentityKind, ItemKey, ItemKind, MarkupFlavor, PageCursor, TimelineEntry,
    TimelineOpts, TokenSource,
};
use crate::http::{HttpRequest, Method, RecordedTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::review::model::{DiffSide, ReviewState};
use crate::scope_fake::FakeScopes;
use crate::Error;

const CLOUD_PR: &str = include_str!("fixtures/conversation/cloud_pr.json");
const CLOUD_COMMENTS: &str = include_str!("fixtures/conversation/cloud_comments.json");
const CLOUD_ACTIVITY: &str = include_str!("fixtures/conversation/cloud_activity.json");
const DC_PR: &str = include_str!("fixtures/conversation/dc_pr.json");
const DC_ACTIVITIES: &str = include_str!("fixtures/conversation/dc_activities.json");

const CLOUD: &str = "https://api.bitbucket.org/2.0/repositories/ws/repo/pullrequests";
const DC: &str = "https://git.example.com/rest/api/1.0/projects/K/repos/r/pull-requests";

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn cloud(t: &Arc<RecordedTransport>) -> BitbucketClient {
    BitbucketClient::new(
        Integration::new("bb", IntegrationKind::Bitbucket, "https://bitbucket.org"),
        "tok".into(),
        t.clone(),
    )
}

fn dc(t: &Arc<RecordedTransport>) -> BitbucketDcClient {
    BitbucketDcClient::new(
        Integration::new(
            "dc",
            IntegrationKind::BitbucketDataCenter,
            "https://git.example.com/",
        ),
        "tok".into(),
        t.clone(),
    )
}

fn body_of(r: &HttpRequest) -> serde_json::Value {
    serde_json::from_str(r.body.as_deref().unwrap()).unwrap()
}

/// The comments of PR 12: the first page has a `next`, and so do two empty
/// ones after it (a window is three pages).
fn cloud_pages() -> RecordedTransport {
    let empty = |n: u32| {
        format!(
            r#"{{"values":[],"next":"https://api.bitbucket.org/2.0/repositories/ws/repo/pullrequests/12/comments?pagelen=100&sort=-created_on&page={n}"}}"#
        )
    };
    RecordedTransport::new()
        .on("sort=-created_on&page=2", 200, &empty(3))
        .on("sort=-created_on&page=3", 200, &empty(4))
}

fn pr(n: u64) -> ItemKey {
    ItemKey::pull_request(n)
}

// ---- Cloud ----------------------------------------------------------------

#[test]
fn a_cloud_pull_request_maps_comments_threads_replies_and_activity() {
    let t = Arc::new(
        cloud_pages()
            .on("pullrequests/12/activity", 200, CLOUD_ACTIVITY)
            .on("pullrequests/12/comments", 200, CLOUD_COMMENTS)
            .on("pullrequests/12", 200, CLOUD_PR),
    );
    let page =
        block_on(cloud(&t).item_timeline("ws/repo", &pr(12), None, &TimelineOpts::default()))
            .unwrap();
    let h = &page.header;
    assert_eq!(h.reference, "#12");
    assert_eq!(h.state_label, "merged");
    assert!(!h.open);
    assert_eq!(h.author.display, "Ann A");
    assert_eq!(h.body.flavor, MarkupFlavor::BitbucketMd);
    assert_eq!(page.total_hint, Some(5));
    assert!(page.older.is_some(), "the host gave a next page");

    let shape: Vec<&str> = page
        .entries
        .iter()
        .map(|e| match e {
            TimelineEntry::Comment(_) => "comment",
            TimelineEntry::Review(_) => "review",
            TimelineEntry::Thread(_) => "thread",
            TimelineEntry::Event(_) => "event",
        })
        .collect();
    // rename event, 10 comment, 12 comment, thread 13 + reply 14, approval, merge.
    assert_eq!(
        shape,
        ["event", "comment", "comment", "thread", "review", "event"]
    );
    let TimelineEntry::Comment(edited) = &page.entries[2] else {
        panic!()
    };
    assert_eq!(edited.id, "12");
    assert!(edited.edited, "updated 40 minutes later");
    assert_eq!(edited.author.display, "Carol C");
    let TimelineEntry::Thread(thread) = &page.entries[3] else {
        panic!()
    };
    assert_eq!(thread.id, "13");
    assert_eq!(thread.path, "src/net.rs");
    assert_eq!((thread.line, thread.side), (Some(17), DiffSide::New));
    assert!(thread.resolved && thread.resolvable);
    assert_eq!(thread.comments.len(), 2, "the deleted comment is hidden");
    assert_eq!(thread.comments[1].in_reply_to.as_deref(), Some("13"));
    assert_eq!(thread.comments[1].discussion_id.as_deref(), Some("13"));
    let TimelineEntry::Review(approval) = &page.entries[4] else {
        panic!()
    };
    assert_eq!(approval.state, ReviewState::Approved);
    assert_eq!(approval.author.login, "bob");
    assert!(matches!(
        &page.entries[0],
        TimelineEntry::Event(e) if e.kind == EventKind::Renamed {
            from: "Retries".into(),
            to: "Add retries".into()
        }
    ));
    assert!(matches!(
        &page.entries[5],
        TimelineEntry::Event(e) if e.kind == EventKind::Merged
    ));
}

#[test]
fn cloud_events_can_be_left_out_and_the_activity_is_then_not_read() {
    let t = Arc::new(
        cloud_pages()
            .on("pullrequests/12/comments", 200, CLOUD_COMMENTS)
            .on("pullrequests/12", 200, CLOUD_PR),
    );
    let page = block_on(cloud(&t).item_timeline(
        "ws/repo",
        &pr(12),
        None,
        &TimelineOpts { events: false },
    ))
    .unwrap();
    assert!(page
        .entries
        .iter()
        .all(|e| matches!(e, TimelineEntry::Comment(_) | TimelineEntry::Thread(_))));
    assert!(!t.requests().iter().any(|r| r.url.contains("/activity")));
}

#[test]
fn a_cloud_activity_that_cannot_be_read_does_not_fail_the_timeline() {
    let t = Arc::new(
        cloud_pages()
            .on("pullrequests/12/activity", 403, "{}")
            .on("pullrequests/12/comments", 200, CLOUD_COMMENTS)
            .on("pullrequests/12", 200, CLOUD_PR),
    );
    let page =
        block_on(cloud(&t).item_timeline("ws/repo", &pr(12), None, &TimelineOpts::default()))
            .unwrap();
    assert_eq!(page.entries.len(), 3);
}

#[test]
fn cloud_reads_newest_first_and_the_cursor_is_the_hosts_next_link() {
    let t = Arc::new(
        cloud_pages()
            .on("pullrequests/12/activity", 200, r#"{"values":[]}"#)
            .on("pullrequests/12/comments", 200, CLOUD_COMMENTS)
            .on("pullrequests/12", 200, CLOUD_PR),
    );
    let c = cloud(&t);
    let first =
        block_on(c.item_timeline("ws/repo", &pr(12), None, &TimelineOpts::default())).unwrap();
    let urls: Vec<String> = t.requests().into_iter().map(|r| r.url).collect();
    assert_eq!(urls[0], format!("{CLOUD}/12"));
    assert_eq!(
        urls[1],
        format!("{CLOUD}/12/comments?pagelen=100&sort=-created_on")
    );
    let cursor = first.older.clone().unwrap();
    assert!(cursor.0.ends_with("&page=4"), "three pages were read");
    // A cursor to another host is never followed (the token would leak).
    let err = block_on(c.item_timeline(
        "ws/repo",
        &pr(12),
        Some(&PageCursor("https://evil.example/x".into())),
        &TimelineOpts::default(),
    ))
    .unwrap_err();
    assert!(matches!(err, Error::Api { status: 422, .. }), "{err}");
}

#[test]
fn cloud_post_reply_edit_delete_send_exactly_these_requests() {
    let made = r#"{"id":20,"created_on":"2026-10-02T10:00:00+00:00","updated_on":"2026-10-02T10:00:00+00:00",
        "content":{"raw":"hi"},"user":{"nickname":"me"}}"#;
    let reply = r#"{"id":21,"created_on":"2026-10-02T10:00:00+00:00","updated_on":"2026-10-02T10:00:00+00:00",
        "content":{"raw":"yes"},"user":{"nickname":"me"},"parent":{"id":13}}"#;
    let t = Arc::new(
        RecordedTransport::new()
            .on("comments/21", 200, reply)
            .on("comments/20", 204, "")
            .on("pullrequests/12/comments", 201, made),
    );
    let c = cloud(&t);
    let a = block_on(c.post_item_comment("ws/repo", &pr(12), "hi", false)).unwrap();
    assert_eq!(a.id, "20");
    let b = block_on(c.reply_in_discussion("ws/repo", &pr(12), "13", "yes")).unwrap();
    assert_eq!(b.discussion_id.as_deref(), Some("13"));
    block_on(c.edit_item_comment("ws/repo", &pr(12), "21", "yes!")).unwrap();
    block_on(c.delete_item_comment("ws/repo", &pr(12), "20")).unwrap();
    let sent = t.requests();
    assert_eq!(sent.len(), 4);
    assert_eq!(
        (sent[0].method, sent[0].url.as_str()),
        (Method::Post, &*format!("{CLOUD}/12/comments"))
    );
    assert_eq!(
        body_of(&sent[0]),
        serde_json::json!({"content":{"raw":"hi"}})
    );
    assert_eq!(
        body_of(&sent[1]),
        serde_json::json!({"content":{"raw":"yes"},"parent":{"id":13}})
    );
    assert_eq!(
        (sent[2].method, sent[2].url.as_str()),
        (Method::Put, &*format!("{CLOUD}/12/comments/21"))
    );
    assert_eq!(
        body_of(&sent[2]),
        serde_json::json!({"content":{"raw":"yes!"}})
    );
    assert_eq!(
        (sent[3].method, sent[3].url.as_str()),
        (Method::Delete, &*format!("{CLOUD}/12/comments/20"))
    );
    assert_eq!(sent[3].body, None);
}

#[test]
fn empty_internal_and_foreign_replies_are_refused_before_any_request() {
    let t = Arc::new(RecordedTransport::new());
    let c = cloud(&t);
    for result in [
        block_on(c.post_item_comment("ws/repo", &pr(12), " ", false)),
        block_on(c.post_item_comment("ws/repo", &pr(12), "x", true)),
        block_on(c.reply_in_discussion("ws/repo", &pr(12), "not-a-number", "x")),
    ] {
        assert!(matches!(result, Err(Error::Api { status: 422, .. })));
    }
    assert!(t.requests().is_empty());
}

#[test]
fn bitbucket_issues_are_unsupported_everywhere() {
    let t = Arc::new(RecordedTransport::new());
    for caps in [
        cloud(&t).conversation_caps(ItemKind::Issue),
        dc(&t).conversation_caps(ItemKind::Issue),
    ] {
        assert!(!caps.read && !caps.comment);
    }
    let issue = ItemKey::issue(3);
    assert!(matches!(
        block_on(cloud(&t).item_timeline("ws/repo", &issue, None, &TimelineOpts::default())),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        block_on(dc(&t).post_item_comment("K/r", &issue, "x", false)),
        Err(Error::Unsupported(_))
    ));
    assert!(t.requests().is_empty());
}

#[test]
fn cloud_capabilities_are_honest() {
    let t = Arc::new(RecordedTransport::new());
    let caps = cloud(&t).conversation_caps(ItemKind::PullRequest);
    assert!(caps.read && caps.comment && caps.reply_in_thread);
    assert!(caps.edit_own && caps.delete_own);
    assert!(!caps.internal_notes && !caps.reactions && !caps.resolve);
}

#[test]
fn a_cloud_personal_token_posts_as_its_nickname() {
    let user = r#"{"uuid":"{u-1}","nickname":"me","display_name":"Me Myself"}"#;
    let t = Arc::new(RecordedTransport::new().on("/user", 200, user));
    let who = block_on(cloud(&t).posting_identity("ws/repo")).unwrap();
    assert_eq!(who.login.as_deref(), Some("me"));
    assert_eq!(who.kind, IdentityKind::PersonalToken);
    assert_eq!(who.source, TokenSource::Profile { id: "bb".into() });
    let mut c = cloud(&t);
    c.scopes = FakeScopes {
        bot: true,
        ..Default::default()
    }
    .handle()
    .0;
    let who = block_on(c.posting_identity("ws/repo")).unwrap();
    assert_eq!(who.source, TokenSource::RepoTokenCommand);
    let t = Arc::new(RecordedTransport::new().on("/user", 401, "{}"));
    assert!(matches!(
        block_on(cloud(&t).posting_identity("ws/repo")),
        Err(Error::Unauthorized { .. })
    ));
}

// ---- Data Center ----------------------------------------------------------

#[test]
fn a_dc_pull_request_maps_threads_replies_events_and_approvals() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("pull-requests/5/activities", 200, DC_ACTIVITIES)
            .on("pull-requests/5", 200, DC_PR),
    );
    let page =
        block_on(dc(&t).item_timeline("K/r", &pr(5), None, &TimelineOpts::default())).unwrap();
    assert_eq!(page.header.reference, "#5");
    assert_eq!(page.header.state_label, "open");
    assert!(page.header.open);
    assert_eq!(page.header.author.login, "ann");
    assert_eq!(page.older, None);
    let shape: Vec<&str> = page
        .entries
        .iter()
        .map(|e| match e {
            TimelineEntry::Comment(_) => "comment",
            TimelineEntry::Review(_) => "review",
            TimelineEntry::Thread(_) => "thread",
            TimelineEntry::Event(_) => "event",
        })
        .collect();
    // opened, pushed, plain comment, thread, approval: oldest first although
    // the host answers newest first.
    assert_eq!(shape, ["event", "event", "comment", "thread", "review"]);
    assert!(matches!(
        &page.entries[1],
        TimelineEntry::Event(e) if e.kind == EventKind::CommitsPushed {
            count: 2,
            head: Some("t2".into())
        }
    ));
    let TimelineEntry::Thread(thread) = &page.entries[3] else {
        panic!()
    };
    assert_eq!(thread.id, "31");
    assert_eq!(
        (thread.path.as_str(), thread.line),
        ("src/net.rs", Some(17))
    );
    assert_eq!(thread.side, DiffSide::New);
    assert!(!thread.resolved && thread.resolvable);
    assert_eq!(thread.comments.len(), 2);
    assert_eq!(thread.comments[0].can_edit, Some(false));
    assert_eq!(thread.comments[1].can_edit, Some(true));
    assert_eq!(thread.comments[1].in_reply_to.as_deref(), Some("31"));
    assert_eq!(thread.comments[1].discussion_id.as_deref(), Some("31"));
    assert!(thread.comments[1].edited);
    assert_eq!(page.entries[3].at(), "2025-10-01T14:00:00Z");
    let TimelineEntry::Review(approval) = &page.entries[4] else {
        panic!()
    };
    assert_eq!(approval.state, ReviewState::Approved);
}

#[test]
fn dc_without_events_keeps_comments_threads_and_approvals() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("pull-requests/5/activities", 200, DC_ACTIVITIES)
            .on("pull-requests/5", 200, DC_PR),
    );
    let page = block_on(dc(&t).item_timeline("K/r", &pr(5), None, &TimelineOpts { events: false }))
        .unwrap();
    assert_eq!(page.entries.len(), 3);
    assert!(page
        .entries
        .iter()
        .all(|e| !matches!(e, TimelineEntry::Event(_))));
}

#[test]
fn dc_pages_by_start_and_hands_back_the_next_start() {
    let page = |next: bool, id: u64| {
        format!(
            r#"{{"isLastPage":{},"nextPageStart":{},"values":[{{"id":{id},"createdDate":1759323600000,"action":"COMMENTED","user":{{"name":"a"}},"comment":{{"id":{id},"text":"c{id}","author":{{"name":"a"}},"createdDate":1759323600000,"updatedDate":1759323600000,"comments":[]}}}}]}}"#,
            !next,
            if next { "100" } else { "null" },
        )
    };
    let t = Arc::new(
        RecordedTransport::new()
            .on("activities?limit=100&start=0", 200, &page(true, 1))
            .on("activities?limit=100&start=100", 200, &page(true, 2))
            .on("activities?limit=100&start=200", 200, &page(true, 3))
            .on("pull-requests/5", 200, DC_PR),
    );
    let c = dc(&t);
    let first = block_on(c.item_timeline("K/r", &pr(5), None, &TimelineOpts::default())).unwrap();
    assert_eq!(first.entries.len(), 3, "three pages make one window");
    assert_eq!(first.older, Some(PageCursor("100".into())));
    let bad = block_on(c.item_timeline(
        "K/r",
        &pr(5),
        Some(&PageCursor("x".into())),
        &TimelineOpts::default(),
    ));
    assert!(matches!(bad, Err(Error::Api { status: 422, .. })));
}

#[test]
fn dc_post_reply_edit_delete_send_exactly_these_requests() {
    let made = r#"{"id":40,"version":0,"text":"hi","author":{"name":"me"},"createdDate":1759323600000,
        "updatedDate":1759323600000,"comments":[]}"#;
    let current = r#"{"id":40,"version":3,"text":"hi","author":{"name":"me"},"createdDate":1759323600000,
        "updatedDate":1759323600000,"comments":[]}"#;
    let t = Arc::new(
        RecordedTransport::new()
            .on("comments/40?version=3", 204, "")
            .on("comments/40", 200, current)
            .on("pull-requests/5/comments", 201, made),
    );
    let c = dc(&t);
    let a = block_on(c.post_item_comment("K/r", &pr(5), "hi", false)).unwrap();
    assert_eq!(
        (a.id.as_str(), a.discussion_id.as_deref()),
        ("40", Some("40"))
    );
    let b = block_on(c.reply_in_discussion("K/r", &pr(5), "31", "yes")).unwrap();
    assert_eq!(b.discussion_id.as_deref(), Some("31"));
    block_on(c.edit_item_comment("K/r", &pr(5), "40", "hi!")).unwrap();
    block_on(c.delete_item_comment("K/r", &pr(5), "40")).unwrap();
    let sent = t.requests();
    let line = |i: usize| (sent[i].method, sent[i].url.clone());
    assert_eq!(line(0), (Method::Post, format!("{DC}/5/comments")));
    assert_eq!(body_of(&sent[0]), serde_json::json!({"text":"hi"}));
    assert_eq!(
        body_of(&sent[1]),
        serde_json::json!({"text":"yes","parent":{"id":31}})
    );
    // An edit reads the comment first for its version, then quotes it.
    assert_eq!(line(2), (Method::Get, format!("{DC}/5/comments/40")));
    assert_eq!(line(3), (Method::Put, format!("{DC}/5/comments/40")));
    assert_eq!(
        body_of(&sent[3]),
        serde_json::json!({"text":"hi!","version":3})
    );
    assert_eq!(line(4), (Method::Get, format!("{DC}/5/comments/40")));
    assert_eq!(
        line(5),
        (Method::Delete, format!("{DC}/5/comments/40?version=3"))
    );
}

#[test]
fn a_dc_user_posts_as_the_name_the_host_reports() {
    let t = Arc::new(RecordedTransport::new().on_with_headers(
        "/application-properties",
        200,
        "{}",
        &[("X-AUSERNAME", "jdoe")],
    ));
    let who = block_on(dc(&t).posting_identity("K/r")).unwrap();
    assert_eq!(who.login.as_deref(), Some("jdoe"));
    assert_eq!(who.kind, IdentityKind::PersonalToken);
    assert_eq!(who.source, TokenSource::Profile { id: "dc".into() });
    let t = Arc::new(RecordedTransport::new().on("/application-properties", 200, "{}"));
    assert!(block_on(dc(&t).posting_identity("K/r")).is_err());
    let author = crate::conversation::Actor::new("jdoe", "Jane", ActorKind::User);
    assert_eq!(who.authored(&author), Some(true));
}
