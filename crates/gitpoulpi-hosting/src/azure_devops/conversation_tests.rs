//! The Azure DevOps conversation against recorded answers: pull request
//! threads (general, code, system), work item comments (Markdown on
//! Services, HTML on Server), the window cursors, the exact requests of
//! post / reply / edit / delete, and identity.

use std::sync::Arc;

use serde_json::{json, Value};

use super::AzureDevopsClient;
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

const PR: &str = include_str!("fixtures/conversation/pr.json");
const THREADS: &str = include_str!("fixtures/conversation/threads.json");
const WORK_ITEM: &str = include_str!("fixtures/conversation/work_item.json");
const WI_MD: &str = include_str!("fixtures/conversation/wi_comments_md.json");
const WI_HTML: &str = include_str!("fixtures/conversation/wi_comments_html.json");
const REPO: &str = r#"{"id":"r-1","name":"Repo","project":{"name":"Proj"}}"#;
const ORG: &str = "https://dev.azure.com/acme";
const PRS: &str = "https://dev.azure.com/acme/Proj/_apis/git/repositories/r-1/pullRequests/9";
const PROJECT: &str = "Proj/Repo";

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn services(t: &Arc<RecordedTransport>) -> AzureDevopsClient {
    AzureDevopsClient::new(
        Integration::new("az", IntegrationKind::AzureDevops, ORG),
        "tok".into(),
        t.clone(),
    )
}

fn server(t: &Arc<RecordedTransport>) -> AzureDevopsClient {
    AzureDevopsClient::new(
        Integration::new(
            "tfs",
            IntegrationKind::AzureDevopsServer,
            "https://tfs.corp/tfs/DefaultCollection",
        ),
        "tok".into(),
        t.clone(),
    )
}

fn body_of(r: &HttpRequest) -> Value {
    serde_json::from_str(r.body.as_deref().unwrap()).unwrap()
}

fn pr_transport(threads: &str) -> Arc<RecordedTransport> {
    Arc::new(
        RecordedTransport::new()
            .on("pullRequests/9/threads", 200, threads)
            .on("pullRequests/9?", 200, PR)
            .on("git/repositories/Repo?", 200, REPO),
    )
}

fn pr() -> ItemKey {
    ItemKey::pull_request(9)
}

fn wi() -> ItemKey {
    ItemKey::issue(42)
}

fn shape(entries: &[TimelineEntry]) -> Vec<&'static str> {
    entries
        .iter()
        .map(|e| match e {
            TimelineEntry::Comment(_) => "comment",
            TimelineEntry::Review(_) => "review",
            TimelineEntry::Thread(_) => "thread",
            TimelineEntry::Event(_) => "event",
        })
        .collect()
}

#[test]
fn a_pull_request_maps_general_code_and_system_threads() {
    let t = pr_transport(THREADS);
    let page = block_on(services(&t).item_timeline(PROJECT, &pr(), None, &TimelineOpts::default()))
        .unwrap();
    let h = &page.header;
    assert_eq!(h.reference, "!9");
    assert_eq!(h.state_label, "merged");
    assert!(!h.open);
    assert_eq!(h.author.login, "ann@acme.test");
    assert_eq!(h.author.display, "Ann A");
    assert_eq!(h.labels, ["backend"]);
    assert_eq!(
        h.web_url,
        "https://dev.azure.com/acme/Proj/_git/Repo/pullrequest/9"
    );
    assert_eq!(page.older, None);
    assert_eq!(page.total_hint, Some(7));
    // source branch update, two comments of thread 1, code thread 2, the
    // approval, a general comment, the completion; the deleted thread and
    // the deleted comment are gone.
    assert_eq!(
        shape(&page.entries),
        ["event", "comment", "comment", "thread", "review", "comment", "event"]
    );
    let TimelineEntry::Comment(reply) = &page.entries[2] else {
        panic!()
    };
    assert_eq!(reply.id, "1.2");
    assert_eq!(reply.in_reply_to.as_deref(), Some("1.1"));
    assert_eq!(reply.discussion_id.as_deref(), Some("1"));
    assert!(reply.edited, "content changed an hour later");
    assert_eq!(reply.author.kind, ActorKind::User);
    let TimelineEntry::Thread(thread) = &page.entries[3] else {
        panic!()
    };
    assert_eq!(thread.id, "2");
    assert_eq!(thread.path, "src/net.rs", "the leading slash is dropped");
    assert_eq!((thread.line, thread.start_line), (Some(12), Some(10)));
    assert_eq!(thread.side, DiffSide::New);
    assert!(thread.resolved && thread.resolvable);
    assert_eq!(thread.comments.len(), 1, "the deleted reply is hidden");
    let TimelineEntry::Review(vote) = &page.entries[4] else {
        panic!()
    };
    assert_eq!(vote.state, ReviewState::Approved);
    assert_eq!(vote.author.display, "Bob B");
    assert!(matches!(
        &page.entries[6],
        TimelineEntry::Event(e) if e.kind == EventKind::Merged
    ));
    assert!(matches!(
        &page.entries[0],
        TimelineEntry::Event(e) if matches!(&e.kind, EventKind::Other { text } if text.contains("updated the source branch"))
    ));
}

#[test]
fn without_events_a_verdict_is_still_shown() {
    let t = pr_transport(THREADS);
    let page =
        block_on(services(&t).item_timeline(PROJECT, &pr(), None, &TimelineOpts { events: false }))
            .unwrap();
    assert_eq!(
        shape(&page.entries),
        ["comment", "comment", "thread", "review", "comment"]
    );
}

#[test]
fn a_long_pull_request_is_cut_to_the_newest_window_by_index() {
    let threads: Vec<Value> = (1..=305)
        .map(|i| {
            json!({ "id": i, "status": "active", "comments": [
                { "id": 1, "parentCommentId": 0, "author": { "uniqueName": "a@x" },
                  "content": format!("c{i}"), "commentType": "text",
                  "publishedDate": format!("2026-10-02T{:02}:{:02}:00Z", i / 60, i % 60) }] })
        })
        .collect();
    let t = pr_transport(&json!({ "value": threads }).to_string());
    let c = services(&t);
    let first = block_on(c.item_timeline(PROJECT, &pr(), None, &TimelineOpts::default())).unwrap();
    assert_eq!(first.entries.len(), 300);
    assert_eq!(first.older, Some(PageCursor("5".into())));
    assert_eq!(first.total_hint, Some(305));
    let second = block_on(c.item_timeline(
        PROJECT,
        &pr(),
        first.older.as_ref(),
        &TimelineOpts::default(),
    ))
    .unwrap();
    assert_eq!(second.entries.len(), 5);
    assert_eq!(second.older, None);
    assert!(matches!(
        block_on(c.item_timeline(
            PROJECT,
            &pr(),
            Some(&PageCursor("x".into())),
            &TimelineOpts::default()
        )),
        Err(Error::Api { status: 422, .. })
    ));
}

#[test]
fn a_pull_request_post_opens_a_thread_and_a_reply_answers_the_first_comment() {
    let thread = r#"{"id":11,"status":"active","comments":[{"id":1,"parentCommentId":0,
        "author":{"uniqueName":"me@acme.test","displayName":"Me"},"content":"hi",
        "publishedDate":"2026-10-02T10:00:00Z","lastUpdatedDate":"2026-10-02T10:00:00Z",
        "commentType":"text"}]}"#;
    let reply = r#"{"id":2,"parentCommentId":1,"author":{"uniqueName":"me@acme.test"},
        "content":"yes","publishedDate":"2026-10-02T10:05:00Z","commentType":"text"}"#;
    let t = Arc::new(
        RecordedTransport::new()
            .on("threads/2/comments", 200, reply)
            .on("pullRequests/9/threads", 200, thread)
            .on("git/repositories/Repo?", 200, REPO),
    );
    let c = services(&t);
    let made = block_on(c.post_item_comment(PROJECT, &pr(), "hi", false)).unwrap();
    assert_eq!(made.id, "11.1");
    assert_eq!(made.discussion_id.as_deref(), Some("11"));
    let answered = block_on(c.reply_in_discussion(PROJECT, &pr(), "2", "yes")).unwrap();
    assert_eq!(answered.id, "2.2");
    assert_eq!(answered.in_reply_to.as_deref(), Some("2.1"));
    let posts: Vec<_> = t
        .requests()
        .into_iter()
        .filter(|r| r.method == Method::Post)
        .collect();
    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0].url, format!("{PRS}/threads?api-version=7.1"));
    assert_eq!(
        body_of(&posts[0]),
        json!({ "comments": [{ "parentCommentId": 0, "content": "hi", "commentType": 1 }], "status": 1 })
    );
    assert_eq!(
        posts[1].url,
        format!("{PRS}/threads/2/comments?api-version=7.1")
    );
    assert_eq!(
        body_of(&posts[1]),
        json!({ "parentCommentId": 1, "content": "yes", "commentType": 1 })
    );
}

#[test]
fn a_pull_request_comment_is_edited_and_deleted_by_thread_and_comment() {
    let edited = r#"{"id":2,"parentCommentId":1,"author":{"uniqueName":"me@acme.test"},
        "content":"yes!","publishedDate":"2026-10-02T10:05:00Z",
        "lastContentUpdatedDate":"2026-10-02T11:05:00Z","commentType":"text"}"#;
    let t = Arc::new(
        RecordedTransport::new()
            .on("threads/2/comments/2", 200, edited)
            .on("git/repositories/Repo?", 200, REPO),
    );
    let c = services(&t);
    let made = block_on(c.edit_item_comment(PROJECT, &pr(), "2.2", "yes!")).unwrap();
    assert!(made.edited);
    block_on(c.delete_item_comment(PROJECT, &pr(), "2.2")).unwrap();
    let sent: Vec<_> = t
        .requests()
        .into_iter()
        .filter(|r| r.url.contains("pullRequests"))
        .collect();
    assert_eq!(sent[0].method, Method::Patch);
    assert_eq!(
        sent[0].url,
        format!("{PRS}/threads/2/comments/2?api-version=7.1")
    );
    assert_eq!(body_of(&sent[0]), json!({ "content": "yes!" }));
    assert_eq!(sent[1].method, Method::Delete);
    assert_eq!(
        sent[1].url,
        format!("{PRS}/threads/2/comments/2?api-version=7.1")
    );
    assert!(matches!(
        block_on(c.edit_item_comment(PROJECT, &pr(), "garbage", "x")),
        Err(Error::Api { status: 422, .. })
    ));
}

fn wi_page(ids: &[u64], token: Option<&str>) -> String {
    let rows: Vec<Value> = ids
        .iter()
        .map(|id| {
            json!({ "id": id, "text": format!("c{id}"), "format": "markdown",
                    "createdBy": { "uniqueName": "a@x" },
                    "createdDate": format!("2026-10-01T10:{:02}:00Z", id % 60),
                    "modifiedDate": format!("2026-10-01T10:{:02}:00Z", id % 60) })
        })
        .collect();
    let mut v = json!({ "totalCount": 9, "comments": rows });
    if let Some(t) = token {
        v["continuationToken"] = json!(t);
    }
    v.to_string()
}

#[test]
fn work_item_comments_are_read_newest_first_by_continuation_token() {
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "continuationToken=t%202",
                200,
                &wi_page(&[7, 6], Some("t 3")),
            )
            .on(
                "continuationToken=t%201",
                200,
                &wi_page(&[9, 8], Some("t 2")),
            )
            .on(
                "workItems/42/comments",
                200,
                &wi_page(&[11, 10], Some("t 1")),
            )
            .on("workItems/42?", 200, WORK_ITEM),
    );
    let c = services(&t);
    let page = block_on(c.item_timeline("Proj", &wi(), None, &TimelineOpts::default())).unwrap();
    let h = &page.header;
    assert_eq!(h.reference, "#42");
    assert_eq!(h.state_label, "active");
    assert!(h.open);
    assert_eq!(h.labels, ["bug", "triage"]);
    assert!(h.body.text.contains("**crashes**"), "{}", h.body.text);
    assert_eq!(
        h.web_url,
        "https://dev.azure.com/acme/Proj/_workitems/edit/42"
    );
    assert_eq!(page.entries.len(), 6, "three pages make one window");
    assert_eq!(page.older, Some(PageCursor("t 3".into())));
    assert_eq!(page.total_hint, Some(9));
    let TimelineEntry::Comment(last) = page.entries.last().unwrap() else {
        panic!()
    };
    assert_eq!(last.id, "11", "oldest first inside the window");
    let urls: Vec<String> = t.requests().into_iter().map(|r| r.url).collect();
    assert_eq!(
        urls[1],
        "https://dev.azure.com/acme/Proj/_apis/wit/workItems/42/comments?$top=100&order=desc&api-version=7.1-preview.4"
    );
    assert!(urls[2].contains("continuationToken=t%201"));
}

#[test]
fn work_item_markdown_is_kept_and_server_html_is_converted() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("workItems/42/comments", 200, WI_MD)
            .on("workItems/42?", 200, WORK_ITEM),
    );
    let md = block_on(services(&t).item_timeline("Proj", &wi(), None, &TimelineOpts::default()))
        .unwrap();
    let TimelineEntry::Comment(second) = &md.entries[1] else {
        panic!()
    };
    assert_eq!(second.body.text, "Second, in **markdown**.");
    assert!(second.edited);
    assert_eq!(second.body.flavor, MarkupFlavor::AzureMd);

    let t = Arc::new(
        RecordedTransport::new()
            .on("workItems/42/comments", 200, WI_HTML)
            .on("workItems/42?", 200, WORK_ITEM),
    );
    let page =
        block_on(server(&t).item_timeline("Proj", &wi(), None, &TimelineOpts::default())).unwrap();
    let TimelineEntry::Comment(c) = &page.entries[0] else {
        panic!()
    };
    assert!(c.body.text.contains("**retry**"), "{}", c.body.text);
    assert!(
        c.body.text.contains("[this](https://example.test/x)"),
        "{}",
        c.body.text
    );
    assert!(!c.body.text.contains('<'));
    let urls: Vec<String> = t.requests().into_iter().map(|r| r.url).collect();
    assert!(
        urls[1].ends_with("workItems/42/comments?$top=100&order=desc&api-version=7.0-preview.3"),
        "{}",
        urls[1]
    );
}

#[test]
fn work_item_comments_are_posted_edited_and_deleted_in_the_hosts_format() {
    let row = r#"{"id":501,"text":"x","format":"markdown","createdBy":{"uniqueName":"me@acme.test"},
        "createdDate":"2026-10-02T10:00:00Z","modifiedDate":"2026-10-02T10:00:00Z"}"#;
    let t = Arc::new(RecordedTransport::new().on("workItems/42/comments", 200, row));
    let c = services(&t);
    let made = block_on(c.post_item_comment("Proj", &wi(), "Hi **there**", false)).unwrap();
    assert_eq!(made.id, "501");
    block_on(c.edit_item_comment("Proj", &wi(), "501", "Hi!")).unwrap();
    block_on(c.delete_item_comment("Proj", &wi(), "501")).unwrap();
    let sent = t.requests();
    let base = "https://dev.azure.com/acme/Proj/_apis/wit/workItems/42/comments";
    assert_eq!(
        (sent[0].method, sent[0].url.as_str()),
        (
            Method::Post,
            &*format!("{base}?format=markdown&api-version=7.1-preview.4")
        )
    );
    assert_eq!(body_of(&sent[0]), json!({ "text": "Hi **there**" }));
    assert_eq!(
        (sent[1].method, sent[1].url.as_str()),
        (
            Method::Patch,
            &*format!("{base}/501?format=markdown&api-version=7.1-preview.4")
        )
    );
    assert_eq!(
        (sent[2].method, sent[2].url.as_str()),
        (
            Method::Delete,
            &*format!("{base}/501?api-version=7.1-preview.4")
        )
    );

    // Server has no Markdown format: HTML is sent, without the format flag.
    let t = Arc::new(RecordedTransport::new().on("workItems/42/comments", 200, row));
    block_on(server(&t).post_item_comment("Proj", &wi(), "Hi **there**", false)).unwrap();
    let sent = t.requests();
    assert!(
        sent[0].url.ends_with("/comments?api-version=7.0-preview.3"),
        "{}",
        sent[0].url
    );
    let text = body_of(&sent[0])["text"].as_str().unwrap().to_string();
    assert!(text.contains("<strong>there</strong>"), "{text}");
    assert!(!text.contains("**"));
}

#[test]
fn what_azure_cannot_do_is_refused_before_any_request() {
    let t = Arc::new(RecordedTransport::new());
    let c = services(&t);
    assert!(matches!(
        block_on(c.reply_in_discussion("Proj", &wi(), "1", "x")),
        Err(Error::Unsupported(_))
    ));
    for result in [
        block_on(c.post_item_comment("Proj", &wi(), " ", false)),
        block_on(c.post_item_comment("Proj", &pr(), "x", true)),
    ] {
        assert!(matches!(result, Err(Error::Api { status: 422, .. })));
    }
    assert!(t.requests().is_empty());
}

#[test]
fn capabilities_differ_by_kind_and_edition() {
    let t = Arc::new(RecordedTransport::new());
    let pr_caps = services(&t).conversation_caps(ItemKind::PullRequest);
    assert!(pr_caps.reply_in_thread && pr_caps.events && pr_caps.edit_own && pr_caps.delete_own);
    assert!(!pr_caps.resolve && !pr_caps.internal_notes && !pr_caps.reactions);
    let item = services(&t).conversation_caps(ItemKind::Issue);
    assert!(item.comment && !item.reply_in_thread && !item.events);
    assert_eq!(item.write_flavor, MarkupFlavor::AzureMd);
    let tfs = server(&t).conversation_caps(ItemKind::Issue);
    assert_eq!(tfs.write_flavor, MarkupFlavor::Html);
    assert_eq!(
        server(&t)
            .conversation_caps(ItemKind::PullRequest)
            .write_flavor,
        MarkupFlavor::AzureMd,
        "pull request comments are Markdown on Server too"
    );
}

#[test]
fn the_identity_is_the_connection_users_account() {
    let data = r#"{"authenticatedUser":{"id":"u-1","providerDisplayName":"Me",
        "properties":{"Account":{"$value":"me@acme.test"}}}}"#;
    let t = Arc::new(RecordedTransport::new().on("/_apis/connectionData", 200, data));
    let who = block_on(services(&t).posting_identity(PROJECT)).unwrap();
    assert_eq!(who.login.as_deref(), Some("me@acme.test"));
    assert_eq!(who.kind, IdentityKind::PersonalToken);
    assert_eq!(who.source, TokenSource::Profile { id: "az".into() });
    let mut c = services(&t);
    c.scopes = FakeScopes {
        bot: true,
        ..Default::default()
    }
    .handle()
    .0;
    assert_eq!(
        block_on(c.posting_identity(PROJECT)).unwrap().source,
        TokenSource::RepoTokenCommand
    );

    // A server that does not say: still an identity, flagged unknown.
    let t = Arc::new(RecordedTransport::new().on("/_apis/connectionData", 404, "{}"));
    let who = block_on(services(&t).posting_identity(PROJECT)).unwrap();
    assert_eq!(who.login, None);
    assert_eq!(who.kind, IdentityKind::Unknown);
    assert!(!who.display.is_empty());
}
