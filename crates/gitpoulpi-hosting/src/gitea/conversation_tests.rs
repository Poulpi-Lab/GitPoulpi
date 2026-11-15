//! The Gitea / Forgejo conversation against recorded answers: mapping,
//! pagination, the exact requests of post / reply / edit / delete, the
//! fallback for a server without the timeline route, and identity.
//! `RecordedTransport` answers the first registered fragment a URL contains,
//! so specific routes are registered before `issues/7`.

use std::sync::Arc;

use super::GiteaClient;
use crate::client::HostingClient;
use crate::conversation::{
    ActorKind, EventKind, IdentityKind, ItemKey, ItemKind, PageCursor, TimelineEntry, TimelineOpts,
    TokenSource,
};
use crate::http::{Method, RecordedTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::review::model::{DiffSide, ReviewState};
use crate::scope_fake::FakeScopes;
use crate::Error;

const ISSUE: &str = include_str!("fixtures/conversation/issue.json");
const PULL: &str = include_str!("fixtures/conversation/pull.json");
const TIMELINE_ISSUE: &str = include_str!("fixtures/conversation/timeline_issue.json");
const TIMELINE_PULL: &str = include_str!("fixtures/conversation/timeline_pull.json");
const USER: &str = r#"{"login":"poulpi","id":3,"full_name":"Poulpi"}"#;
const API: &str = "https://git.example.com/api/v1/repos/o/p";
const PROJECT: &str = "o/p";

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn client_of(kind: IntegrationKind, t: &Arc<RecordedTransport>) -> GiteaClient {
    GiteaClient::new(
        Integration::new("g", kind, "https://git.example.com/"),
        "tok".into(),
        t.clone(),
    )
}

fn client(t: &Arc<RecordedTransport>) -> GiteaClient {
    client_of(IntegrationKind::Gitea, t)
}

fn body_of(r: &crate::http::HttpRequest) -> serde_json::Value {
    serde_json::from_str(r.body.as_deref().unwrap()).unwrap()
}

fn code_comment(id: u64, body: &str) -> String {
    format!(
        r#"{{"id":{id},"body":"{body}","user":{{"login":"poulpi"}},"resolver":null,
        "created_at":"2026-10-02T11:00:01Z","path":"main.rs","commit_id":"h1","position":2,
        "original_position":0,"html_url":"https://git.example.com/o/p/pulls/3#issuecomment-{id}"}}"#
    )
}

fn pull_row() -> String {
    r#"{"number":3,"title":"Greet","state":"closed","draft":false,"merged":true,
    "merged_at":"2026-10-02T13:00:00Z","html_url":"https://git.example.com/o/p/pulls/3",
    "updated_at":"2026-10-02T13:00:00Z","user":{"login":"bob"},
    "head":{"ref":"f","sha":"h1","repo":{"full_name":"o/p"}},
    "base":{"ref":"master","sha":"b1","repo":{"full_name":"o/p"}},
    "assignees":null,"requested_reviewers":[],"mergeable":true}"#
        .to_string()
}

#[test]
fn an_issue_maps_header_comments_bots_edits_and_events() {
    let t = Arc::new(
        RecordedTransport::new()
            .on_with_headers(
                "issues/7/timeline",
                200,
                TIMELINE_ISSUE,
                &[("X-Total-Count", "9")],
            )
            .on("issues/7", 200, ISSUE),
    );
    let page = block_on(client(&t).item_timeline(
        PROJECT,
        &ItemKey::issue(7),
        None,
        &TimelineOpts::default(),
    ))
    .unwrap();
    let h = &page.header;
    assert_eq!(h.reference, "#7");
    assert_eq!(h.state_label, "closed");
    assert!(!h.open);
    assert_eq!(h.labels, ["bug", "help wanted"]);
    assert_eq!(h.author.display, "Alice A");
    assert!(h.body.text.contains("**crashes**"));
    assert_eq!(page.older, None);
    assert_eq!(page.total_hint, Some(9));

    let TimelineEntry::Comment(carol) = &page.entries[1] else {
        panic!("{:?}", page.entries[1]);
    };
    assert_eq!(carol.id, "2");
    assert!(carol.edited, "updated 30 minutes after it was created");
    assert_eq!(carol.author.display, "Carol C");
    let TimelineEntry::Comment(bot) = &page.entries[4] else {
        panic!("{:?}", page.entries[4]);
    };
    assert_eq!(bot.author.kind, ActorKind::Bot);
    assert!(!bot.edited, "one second apart is not an edit");

    let kinds: Vec<&EventKind> = page
        .entries
        .iter()
        .filter_map(|e| match e {
            TimelineEntry::Event(e) => Some(&e.kind),
            _ => None,
        })
        .collect();
    assert_eq!(
        kinds,
        [
            &EventKind::Labeled {
                added: vec!["bug".into()],
                removed: vec![]
            },
            &EventKind::Assigned {
                added: vec!["bob".into()],
                removed: vec![]
            },
            &EventKind::Renamed {
                from: "Crash on empty".into(),
                to: "Crash on empty config".into()
            },
            &EventKind::CrossReferenced {
                from: "o/other#12".into(),
                url: Some("https://git.example.com/o/other/issues/12".into())
            },
            &EventKind::Labeled {
                added: vec![],
                removed: vec!["help wanted".into()]
            },
            &EventKind::Other {
                text: "lock".into()
            },
            &EventKind::Closed,
        ]
    );
}

#[test]
fn events_can_be_left_out() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("issues/7/timeline", 200, TIMELINE_ISSUE)
            .on("issues/7", 200, ISSUE),
    );
    let page = block_on(client(&t).item_timeline(
        PROJECT,
        &ItemKey::issue(7),
        None,
        &TimelineOpts { events: false },
    ))
    .unwrap();
    assert_eq!(page.entries.len(), 2);
    assert!(page
        .entries
        .iter()
        .all(|e| matches!(e, TimelineEntry::Comment(_))));
}

fn pull_fixture() -> Arc<RecordedTransport> {
    Arc::new(
        RecordedTransport::new()
            .on("issues/3/timeline", 200, TIMELINE_PULL)
            .on("issues/3", 200, PULL)
            .on(
                "/pulls/3/reviews/4/comments",
                200,
                &format!("[{}]", code_comment(31, "on the new line")),
            )
            .on(
                "/pulls/3/reviews?",
                200,
                r#"[{"id":4,"state":"COMMENT","body":"looks ok","comments_count":1,
                    "user":{"login":"poulpi"},"submitted_at":"2026-10-02T11:00:00Z"}]"#,
            )
            .on("/repos/o/p/pulls/3", 200, &pull_row()),
    )
}

#[test]
fn a_pull_request_adds_threads_and_reviews_and_skips_the_duplicates() {
    let t = pull_fixture();
    let page = block_on(client(&t).item_timeline(
        PROJECT,
        &ItemKey::pull_request(3),
        None,
        &TimelineOpts::default(),
    ))
    .unwrap();
    assert_eq!(page.header.state_label, "merged");
    assert!(!page.header.open);
    // push, comment, review, thread, review request, merge: the timeline's
    // own `review` and `code` rows are not repeated.
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
    assert_eq!(
        shape,
        ["event", "comment", "review", "thread", "event", "event"]
    );
    let TimelineEntry::Review(review) = &page.entries[2] else {
        panic!()
    };
    assert_eq!(review.state, ReviewState::Commented);
    assert_eq!(review.body.text, "looks ok");
    let TimelineEntry::Thread(thread) = &page.entries[3] else {
        panic!()
    };
    assert_eq!(thread.id, "31|n|2|main.rs");
    assert_eq!(thread.side, DiffSide::New);
    assert_eq!(
        thread.comments[0].discussion_id.as_deref(),
        Some(thread.id.as_str())
    );
    assert!(thread.resolvable);
    let TimelineEntry::Event(pushed) = &page.entries[0] else {
        panic!()
    };
    assert_eq!(
        pushed.kind,
        EventKind::CommitsPushed {
            count: 2,
            head: Some("bbb222".into())
        }
    );
    assert!(matches!(
        &page.entries[4],
        TimelineEntry::Event(e) if e.kind == EventKind::ReviewRequested { who: "alice".into() }
    ));
    assert!(matches!(
        &page.entries[5],
        TimelineEntry::Event(e) if e.kind == EventKind::Merged
    ));
}

#[test]
fn a_server_without_the_timeline_route_falls_back_to_the_comment_list() {
    let comments = r#"[{"id":2,"body":"hello","created_at":"2026-10-01T10:00:00Z",
        "updated_at":"2026-10-01T10:00:00Z","user":{"login":"carol"}}]"#;
    let t = Arc::new(
        RecordedTransport::new()
            .on("issues/7/timeline", 404, "{}")
            .on("issues/7/comments", 200, comments)
            .on("issues/7", 200, ISSUE),
    );
    let page = block_on(client(&t).item_timeline(
        PROJECT,
        &ItemKey::issue(7),
        None,
        &TimelineOpts::default(),
    ))
    .unwrap();
    assert_eq!(page.entries.len(), 1);
    assert!(matches!(&page.entries[0], TimelineEntry::Comment(c) if c.body.text == "hello"));
}

#[test]
fn the_newest_window_is_read_first_and_older_pages_by_cursor() {
    // 350 rows: 7 pages of 50. The window is the last 6 pages, 300 rows.
    let page_of = |n: u32| {
        let rows: Vec<String> = (0..50)
            .map(|i| {
                let id = (n - 1) * 50 + i + 1;
                format!(
                    r#"{{"id":{id},"type":"comment","body":"c{id}","created_at":"2026-10-01T10:00:00Z","user":{{"login":"u"}}}}"#
                )
            })
            .collect();
        format!("[{}]", rows.join(","))
    };
    let mut transport = RecordedTransport::new();
    for n in 1..=7u32 {
        transport = transport.on_with_headers(
            &format!("timeline?limit=50&page={n}"),
            200,
            &page_of(n),
            &[("X-Total-Count", "350")],
        );
    }
    let t = Arc::new(transport.on("issues/3", 200, ISSUE));
    let c = client(&t);
    let first =
        block_on(c.item_timeline(PROJECT, &ItemKey::issue(3), None, &TimelineOpts::default()))
            .unwrap();
    assert_eq!(first.entries.len(), 300);
    let TimelineEntry::Comment(oldest) = &first.entries[0] else {
        panic!()
    };
    assert_eq!(oldest.id, "51", "page 1 is the older window");
    assert_eq!(first.older, Some(PageCursor("t:1".into())));
    assert_eq!(first.total_hint, Some(350));
    let pages: Vec<String> = t
        .requests()
        .into_iter()
        .filter(|r| r.url.contains("timeline"))
        .map(|r| r.url)
        .collect();
    assert_eq!(pages.len(), 7);
    assert_eq!(pages[0], format!("{API}/issues/3/timeline?limit=50&page=1"));

    let second = block_on(c.item_timeline(
        PROJECT,
        &ItemKey::issue(3),
        first.older.as_ref(),
        &TimelineOpts::default(),
    ))
    .unwrap();
    assert_eq!(second.entries.len(), 50);
    assert_eq!(second.older, None);
}

#[test]
fn a_bad_cursor_is_refused() {
    let t = Arc::new(RecordedTransport::new().on("issues/3", 200, ISSUE));
    let err = block_on(client(&t).item_timeline(
        PROJECT,
        &ItemKey::issue(3),
        Some(&PageCursor("nope".into())),
        &TimelineOpts::default(),
    ))
    .unwrap_err();
    assert!(matches!(err, Error::Api { status: 422, .. }), "{err}");
}

#[test]
fn posting_sends_one_request_and_internal_notes_are_refused() {
    let created = r#"{"id":88,"body":"Thanks","user":{"login":"poulpi"},
        "created_at":"2026-10-03T10:00:00Z","updated_at":"2026-10-03T10:00:00Z"}"#;
    let t = Arc::new(RecordedTransport::new().on("issues/7/comments", 201, created));
    let c = client(&t);
    let made = block_on(c.post_item_comment(PROJECT, &ItemKey::issue(7), "Thanks", false)).unwrap();
    assert_eq!(made.id, "88");
    let sent = t.requests();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].method, Method::Post);
    assert_eq!(sent[0].url, format!("{API}/issues/7/comments"));
    assert_eq!(body_of(&sent[0]), serde_json::json!({ "body": "Thanks" }));
    assert!(matches!(
        block_on(c.post_item_comment(PROJECT, &ItemKey::issue(7), "x", true)),
        Err(Error::Api { status: 422, .. })
    ));
    assert!(matches!(
        block_on(c.post_item_comment(PROJECT, &ItemKey::issue(7), "  ", false)),
        Err(Error::Api { status: 422, .. })
    ));
    assert_eq!(t.requests().len(), 1, "refused before any request");
}

#[test]
fn replying_goes_through_the_thread_root_and_an_issue_cannot_reply() {
    let t = Arc::new(RecordedTransport::new().on(
        "/pulls/3/comments/31/replies",
        201,
        &code_comment(40, "answer"),
    ));
    let c = client(&t);
    let made = block_on(c.reply_in_discussion(
        PROJECT,
        &ItemKey::pull_request(3),
        "31|n|2|main.rs",
        "answer",
    ))
    .unwrap();
    assert_eq!(made.id, "40");
    assert_eq!(made.discussion_id.as_deref(), Some("31|n|2|main.rs"));
    let sent = t.requests();
    assert_eq!(sent[0].method, Method::Post);
    assert_eq!(sent[0].url, format!("{API}/pulls/3/comments/31/replies"));
    assert_eq!(body_of(&sent[0]), serde_json::json!({ "body": "answer" }));
    assert!(matches!(
        block_on(c.reply_in_discussion(PROJECT, &ItemKey::issue(7), "x", "y")),
        Err(Error::Unsupported(_))
    ));
}

#[test]
fn editing_patches_and_deleting_deletes_the_comment() {
    let edited = r#"{"id":88,"body":"Updated","user":{"login":"poulpi"},
        "created_at":"2026-10-03T10:00:00Z","updated_at":"2026-10-03T11:00:00Z"}"#;
    let t = Arc::new(RecordedTransport::new().on("issues/comments/88", 200, edited));
    let c = client(&t);
    let made = block_on(c.edit_item_comment(PROJECT, &ItemKey::issue(7), "88", "Updated")).unwrap();
    assert!(made.edited);
    block_on(c.delete_item_comment(PROJECT, &ItemKey::pull_request(3), "88")).unwrap();
    let sent = t.requests();
    assert_eq!(sent[0].method, Method::Patch);
    assert_eq!(sent[0].url, format!("{API}/issues/comments/88"));
    assert_eq!(body_of(&sent[0]), serde_json::json!({ "body": "Updated" }));
    assert_eq!(sent[1].method, Method::Delete);
    assert_eq!(sent[1].url, format!("{API}/issues/comments/88"));
    assert_eq!(sent[1].body, None);
}

#[test]
fn capabilities_are_honest_about_resolution_events_and_internal_notes() {
    let t = Arc::new(RecordedTransport::new());
    let gitea = client(&t);
    let forgejo = client_of(IntegrationKind::Forgejo, &t);
    let issue = gitea.conversation_caps(ItemKind::Issue);
    assert!(issue.edit_own && issue.delete_own && issue.events);
    assert!(!issue.reply_in_thread && !issue.resolve);
    assert!(!issue.internal_notes && !issue.reactions);
    assert!(gitea.conversation_caps(ItemKind::PullRequest).resolve);
    let pr = forgejo.conversation_caps(ItemKind::PullRequest);
    assert!(pr.reply_in_thread && !pr.resolve, "Forgejo cannot resolve");
}

#[test]
fn a_personal_token_posts_as_its_user_and_a_token_command_is_asked_afresh() {
    let t = Arc::new(RecordedTransport::new().on("/user", 200, USER));
    let who = block_on(client(&t).posting_identity(PROJECT)).unwrap();
    assert_eq!(who.login.as_deref(), Some("poulpi"));
    assert_eq!(who.kind, IdentityKind::PersonalToken);
    assert_eq!(who.source, TokenSource::Profile { id: "g".into() });

    // The profile's cached login belongs to its own token, not the command's.
    let t = Arc::new(RecordedTransport::new().on("/user", 200, USER));
    let mut cached = Integration::new("g", IntegrationKind::Gitea, "https://git.example.com/");
    cached.user = Some(gitpoulpi_config::ConnectedUser {
        login: "someone-else".into(),
        name: String::new(),
        id: String::new(),
    });
    let mut c = GiteaClient::new(cached, "tok".into(), t.clone());
    let (handle, _) = FakeScopes {
        bot: true,
        ..Default::default()
    }
    .handle();
    c.scopes = handle;
    let who = block_on(c.posting_identity(PROJECT)).unwrap();
    assert_eq!(who.login.as_deref(), Some("poulpi"));
    assert_eq!(who.source, TokenSource::RepoTokenCommand);
    assert_eq!(t.requests().len(), 1);
}

#[test]
fn a_rejected_token_is_an_error() {
    let t = Arc::new(RecordedTransport::new().on("/user", 401, r#"{"message":"bad"}"#));
    assert!(matches!(
        block_on(client(&t).posting_identity(PROJECT)),
        Err(Error::Unauthorized { .. })
    ));
}
