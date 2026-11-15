//! The GitHub conversation against recorded answers: mapping, pagination,
//! the exact requests of post / reply / edit / delete, and who a post is
//! made as.

use std::sync::Arc;

use super::GitHubClient;
use crate::client::HostingClient;
use crate::conversation::{
    ActorKind, ConversationCaps, EventKind, IdentityKind, ItemKey, ItemKind, PageCursor,
    TimelineEntry, TimelineOpts, TokenSource,
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
const THREADS: &str = include_str!("fixtures/conversation/threads.json");
const COMMENT_CREATED: &str = include_str!("fixtures/conversation/comment_created.json");
const REVIEW_COMMENT_EDITED: &str =
    include_str!("fixtures/conversation/review_comment_edited.json");
const REPLY: &str = include_str!("fixtures/conversation/reply_mutation.json");
const VIEWER: &str = include_str!("fixtures/conversation/viewer.json");
const VIEWER_FORBIDDEN: &str = include_str!("fixtures/conversation/viewer_forbidden.json");
const USER_FORBIDDEN: &str = include_str!("fixtures/conversation/user_forbidden.json");
const USER: &str =
    r#"{"login":"octo","id":9,"name":"Octo Cat","html_url":"https://github.com/octo"}"#;

const API: &str = "https://api.github.com/repos/octo/thing";
const PROJECT: &str = "octo/thing";

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn client_with(token: &str, transport: &Arc<RecordedTransport>) -> GitHubClient {
    GitHubClient::new(
        Integration::new("gh", IntegrationKind::Github, "https://github.com"),
        token.into(),
        transport.clone(),
    )
}

fn client(transport: &Arc<RecordedTransport>) -> GitHubClient {
    client_with("tok", transport)
}

fn no_events() -> TimelineOpts {
    TimelineOpts { events: false }
}

#[test]
fn an_issue_maps_header_comments_bots_edits_reactions_and_events() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("issues/7/timeline", 200, TIMELINE_ISSUE)
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
    assert_eq!(h.labels, ["bug", "good first issue"]);
    assert_eq!(h.author.login, "alice");
    assert!(h.body.text.contains("**empty**"));
    assert_eq!(page.older, None);

    assert_eq!(page.entries.len(), 8, "the subscribed row is dropped");
    let TimelineEntry::Comment(carol) = &page.entries[1] else {
        panic!("{:?}", page.entries[1]);
    };
    assert_eq!(carol.id, "501");
    assert!(carol.edited, "updated 30 minutes after it was created");
    assert_eq!(carol.author.kind, ActorKind::User);
    assert_eq!(
        carol
            .reactions
            .iter()
            .map(|r| (r.name.as_str(), r.count))
            .collect::<Vec<_>>(),
        [("+1", 2), ("heart", 1)]
    );
    let TimelineEntry::Comment(bot) = &page.entries[2] else {
        panic!("{:?}", page.entries[2]);
    };
    assert_eq!(bot.author.kind, ActorKind::Bot);
    assert_eq!(bot.author.login, "poulpi-lab[bot]");
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
                from: "Crash on empty config".into(),
                to: "Crash when the config is empty".into()
            },
            &EventKind::CrossReferenced {
                from: "octo/thing#12".into(),
                url: Some("https://github.com/octo/thing/pull/12".into())
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
    let page = block_on(client(&t).item_timeline(PROJECT, &ItemKey::issue(7), None, &no_events()))
        .unwrap();
    assert_eq!(page.entries.len(), 3);
    assert!(page
        .entries
        .iter()
        .all(|e| matches!(e, TimelineEntry::Comment(_))));
}

#[test]
fn a_pull_request_adds_reviews_commit_runs_and_code_threads() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/graphql", 200, THREADS)
            .on("issues/12/timeline", 200, TIMELINE_PULL)
            .on("issues/12", 200, PULL),
    );
    let page = block_on(client(&t).item_timeline(
        PROJECT,
        &ItemKey::pull_request(12),
        None,
        &TimelineOpts::default(),
    ))
    .unwrap();
    assert_eq!(page.header.state_label, "merged");
    assert_eq!(page.header.reference, "#12");

    // Oldest first, threads placed by when they were opened.
    let shape: Vec<String> = page
        .entries
        .iter()
        .map(|e| match e {
            TimelineEntry::Thread(t) => format!("thread {}", t.id),
            TimelineEntry::Review(r) => format!("review {:?}", r.state),
            TimelineEntry::Comment(c) => format!("comment {}", c.id),
            TimelineEntry::Event(e) => match &e.kind {
                EventKind::CommitsPushed { count, head } => {
                    format!("pushed {count} {}", head.clone().unwrap_or_default())
                }
                EventKind::ReviewRequested { who } => format!("requested {who}"),
                EventKind::Merged => "merged".to_string(),
                other => format!("{other:?}"),
            },
        })
        .collect();
    assert_eq!(
        shape,
        [
            "thread PRRT_kwDOB",
            "pushed 2 bbb2222",
            "requested alice",
            "thread PRRT_kwDOA",
            "review Approved",
            "pushed 0 ccc3333",
            "comment 601",
            "merged",
        ],
        "consecutive commits fold into one run; an empty COMMENTED review is dropped"
    );

    let TimelineEntry::Thread(open) = &page.entries[3] else {
        panic!()
    };
    assert_eq!(open.path, "src/config.rs");
    assert_eq!(open.line, Some(42));
    assert_eq!(open.side, DiffSide::New);
    assert!(open.diff_hunk.as_deref().unwrap().starts_with("@@ -40,6"));
    assert_eq!(open.comments.len(), 2);
    assert_eq!(open.comments[0].id, "review-4001");
    assert_eq!(
        open.comments[0].discussion_id.as_deref(),
        Some("PRRT_kwDOA")
    );
    assert_eq!(open.comments[0].reactions.len(), 1);
    assert_eq!(open.comments[1].author.kind, ActorKind::Bot);
    assert!(open.comments[1].edited, "lastEditedAt is set");
    assert_eq!(open.comments[1].in_reply_to.as_deref(), Some("review-4001"));
    assert_eq!(open.comments[1].can_edit, Some(true));

    let TimelineEntry::Thread(stale) = &page.entries[0] else {
        panic!()
    };
    assert!(stale.outdated && stale.resolved);
    assert_eq!(stale.line, None);
    assert_eq!(stale.original_line, Some(9));
    assert_eq!(stale.side, DiffSide::Old);
    assert_eq!(stale.comments[0].author.login, "ghost");

    let TimelineEntry::Review(r) = &page.entries[4] else {
        panic!()
    };
    assert_eq!(r.state, ReviewState::Approved);
    assert_eq!(r.body.text, "Looks good now.");
}

fn one_comment(id: u64, at: &str) -> String {
    format!(
        r#"[{{"id":{id},"event":"commented","user":{{"login":"u","type":"User"}},"created_at":"{at}","updated_at":"{at}","body":"c{id}"}}]"#
    )
}

#[test]
fn the_newest_window_is_read_first_and_older_pages_by_cursor() {
    let link = format!("<{API}/issues/7/timeline?per_page=100&page=4>; rel=\"last\"");
    let t = Arc::new(
        RecordedTransport::new()
            .on_with_headers(
                "timeline?per_page=100&page=1",
                200,
                &one_comment(1, "2024-01-01T00:00:00Z"),
                &[("Link", &link)],
            )
            .on(
                "timeline?per_page=100&page=2",
                200,
                &one_comment(2, "2024-02-01T00:00:00Z"),
            )
            .on(
                "timeline?per_page=100&page=3",
                200,
                &one_comment(3, "2024-03-01T00:00:00Z"),
            )
            .on(
                "timeline?per_page=100&page=4",
                200,
                &one_comment(4, "2024-04-01T00:00:00Z"),
            )
            .on("issues/7", 200, ISSUE),
    );
    let c = client(&t);
    let first = block_on(c.item_timeline(PROJECT, &ItemKey::issue(7), None, &no_events())).unwrap();
    let ids: Vec<String> = first
        .entries
        .iter()
        .map(|e| match e {
            TimelineEntry::Comment(c) => c.id.clone(),
            _ => String::new(),
        })
        .collect();
    assert_eq!(ids, ["2", "3", "4"], "three pages, oldest to newest");
    let cursor = first.older.clone().expect("page 1 is still to load");
    assert_eq!(cursor.0, "1@2024-02-01T00:00:00Z");
    assert_eq!(first.total_hint, Some(301));
    let urls: Vec<String> = t.requests().into_iter().map(|r| r.url).collect();
    assert_eq!(
        urls,
        [
            format!("{API}/issues/7"),
            format!("{API}/issues/7/timeline?per_page=100&page=1"),
            format!("{API}/issues/7/timeline?per_page=100&page=2"),
            format!("{API}/issues/7/timeline?per_page=100&page=3"),
            format!("{API}/issues/7/timeline?per_page=100&page=4"),
        ]
    );

    let second =
        block_on(c.item_timeline(PROJECT, &ItemKey::issue(7), Some(&cursor), &no_events()))
            .unwrap();
    assert_eq!(second.entries.len(), 1);
    assert_eq!(second.older, None, "page 1 starts the conversation");

    let bad = block_on(c.item_timeline(
        PROJECT,
        &ItemKey::issue(7),
        Some(&PageCursor("nonsense".into())),
        &no_events(),
    ));
    assert!(matches!(bad, Err(Error::Api { status: 422, .. })));
}

#[test]
fn posting_sends_exactly_one_post_and_maps_the_bot_author() {
    let t = Arc::new(RecordedTransport::new().on("issues/7/comments", 201, COMMENT_CREATED));
    let c = block_on(client(&t).post_item_comment(
        PROJECT,
        &ItemKey::issue(7),
        "Fixed in main.",
        false,
    ))
    .unwrap();
    let sent = t.requests();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].method, Method::Post);
    assert_eq!(sent[0].url, format!("{API}/issues/7/comments"));
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(sent[0].body.as_deref().unwrap()).unwrap(),
        serde_json::json!({ "body": "Fixed in main." })
    );
    assert!(sent[0]
        .headers
        .iter()
        .any(|(k, v)| k == "Authorization" && v == "Bearer tok"));
    assert_eq!(c.id, "777");
    assert_eq!(c.author.kind, ActorKind::Bot);
    assert_eq!(c.author.login, "poulpi-lab[bot]");
}

#[test]
fn a_pull_request_comment_uses_the_same_endpoint_and_add_comment_still_works() {
    let t = Arc::new(RecordedTransport::new().on("issues/12/comments", 201, COMMENT_CREATED));
    let old = block_on(client(&t).add_comment(PROJECT, 12, "hello")).unwrap();
    assert_eq!(old.id, "777");
    assert_eq!(old.author, "poulpi-lab[bot]");
    assert_eq!(t.requests()[0].url, format!("{API}/issues/12/comments"));
}

#[test]
fn an_empty_comment_and_an_internal_one_are_refused_before_any_request() {
    let t = Arc::new(RecordedTransport::new());
    let c = client(&t);
    assert!(matches!(
        block_on(c.post_item_comment(PROJECT, &ItemKey::issue(7), "  \n", false)),
        Err(Error::Api { status: 422, .. })
    ));
    assert!(matches!(
        block_on(c.post_item_comment(PROJECT, &ItemKey::issue(7), "secret", true)),
        Err(Error::Api { status: 422, .. })
    ));
    assert!(t.requests().is_empty());
}

#[test]
fn the_first_post_of_an_app_bot_teaches_its_login() {
    let t = Arc::new(RecordedTransport::new().on("issues/7/comments", 201, COMMENT_CREATED));
    let (handle, scopes) = FakeScopes {
        bot: true,
        ..Default::default()
    }
    .handle();
    let mut c = client_with("ghs_installation", &t);
    c.scopes = handle;
    block_on(c.post_item_comment(PROJECT, &ItemKey::issue(7), "hi", false)).unwrap();
    assert_eq!(scopes.learned().as_deref(), Some("poulpi-lab[bot]"));
}

#[test]
fn a_configured_name_or_a_personal_token_is_never_overwritten_by_learning() {
    // "Post as" configured: nothing to learn.
    let t = Arc::new(RecordedTransport::new().on("issues/7/comments", 201, COMMENT_CREATED));
    let (handle, scopes) = FakeScopes {
        bot: true,
        post_as: Some("mine[bot]".into()),
        ..Default::default()
    }
    .handle();
    let mut c = client_with("ghs_installation", &t);
    c.scopes = handle;
    block_on(c.post_item_comment(PROJECT, &ItemKey::issue(7), "hi", false)).unwrap();
    assert_eq!(scopes.learned(), None);

    // A personal token with no token command: a bot author is not "me".
    let (handle, scopes) = FakeScopes::default().handle();
    let mut c = client(&t);
    c.scopes = handle;
    block_on(c.post_item_comment(PROJECT, &ItemKey::issue(7), "hi", false)).unwrap();
    assert_eq!(scopes.learned(), None);
}

#[test]
fn editing_and_deleting_address_the_right_comment_route() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("pulls/comments/4002", 200, REVIEW_COMMENT_EDITED)
            .on("issues/comments/777", 200, COMMENT_CREATED),
    );
    let c = client(&t);
    let key = ItemKey::pull_request(12);

    let plain = block_on(c.edit_item_comment(PROJECT, &key, "777", "Fixed.")).unwrap();
    assert_eq!(plain.id, "777");
    let code =
        block_on(c.edit_item_comment(PROJECT, &key, "review-4002", "Done, pushed.")).unwrap();
    assert_eq!(code.id, "review-4002");
    assert!(code.edited, "09:30 to 09:50");
    assert_eq!(code.in_reply_to.as_deref(), Some("review-4001"));

    block_on(c.delete_item_comment(PROJECT, &key, "777")).unwrap();
    block_on(c.delete_item_comment(PROJECT, &key, "review-4002")).unwrap();

    let sent = t.requests();
    let shape: Vec<(Method, String)> = sent.iter().map(|r| (r.method, r.url.clone())).collect();
    assert_eq!(
        shape,
        [
            (Method::Patch, format!("{API}/issues/comments/777")),
            (Method::Patch, format!("{API}/pulls/comments/4002")),
            (Method::Delete, format!("{API}/issues/comments/777")),
            (Method::Delete, format!("{API}/pulls/comments/4002")),
        ]
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(sent[1].body.as_deref().unwrap()).unwrap(),
        serde_json::json!({ "body": "Done, pushed." })
    );
    assert_eq!(sent[2].body, None);
}

#[test]
fn replying_in_a_thread_goes_through_graphql_and_an_issue_has_no_threads() {
    let t = Arc::new(RecordedTransport::new().on("/graphql", 200, REPLY));
    let c = client(&t);
    let reply = block_on(c.reply_in_discussion(
        PROJECT,
        &ItemKey::pull_request(12),
        "PRRT_kwDOA",
        "Agreed.",
    ))
    .unwrap();
    assert_eq!(reply.id, "review-4010");
    assert_eq!(reply.discussion_id.as_deref(), Some("PRRT_kwDOA"));
    assert_eq!(reply.author.kind, ActorKind::Bot);
    let sent = t.requests();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].url, "https://api.github.com/graphql");
    let body: serde_json::Value = serde_json::from_str(sent[0].body.as_deref().unwrap()).unwrap();
    assert!(body["query"]
        .as_str()
        .unwrap()
        .contains("addPullRequestReviewThreadReply"));
    assert_eq!(
        body["variables"],
        serde_json::json!({ "thread": "PRRT_kwDOA", "body": "Agreed." })
    );

    let on_issue = block_on(c.reply_in_discussion(PROJECT, &ItemKey::issue(7), "x", "hi"));
    assert!(matches!(on_issue, Err(Error::Unsupported(_))));
    assert_eq!(t.requests().len(), 1, "nothing was sent for the issue");
}

#[test]
fn capabilities_follow_the_item_kind() {
    let t = Arc::new(RecordedTransport::new());
    let c = client(&t);
    let issue = c.conversation_caps(ItemKind::Issue);
    let pull = c.conversation_caps(ItemKind::PullRequest);
    assert!(issue.comment && issue.edit_own && !issue.reply_in_thread && !issue.resolve);
    assert!(pull.reply_in_thread && pull.resolve && pull.reactions);
    assert!(!pull.internal_notes);
}

#[test]
fn a_provider_without_conversations_says_unsupported() {
    let t = Arc::new(RecordedTransport::new());
    let c = crate::client::client_for(
        Integration::new(
            "rm",
            IntegrationKind::Redmine,
            "https://redmine.example.com",
        ),
        "tok".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    assert_eq!(c.conversation_caps(ItemKind::Issue), ConversationCaps::NONE);
    assert!(matches!(
        block_on(c.item_timeline("p", &ItemKey::issue(1), None, &no_events())),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        block_on(c.post_item_comment("p", &ItemKey::issue(1), "x", false)),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        block_on(c.posting_identity("p")),
        Err(Error::Unsupported(_))
    ));
    assert!(t.requests().is_empty());
}

// ---- who a post is made as ----

#[test]
fn a_personal_token_posts_as_its_user() {
    let t = Arc::new(RecordedTransport::new().on("/user", 200, USER));
    let who = block_on(client(&t).posting_identity(PROJECT)).unwrap();
    assert_eq!(who.login.as_deref(), Some("octo"));
    assert_eq!(who.kind, IdentityKind::PersonalToken);
    assert_eq!(who.source, TokenSource::Profile { id: "gh".into() });
    assert_eq!(who.display, "octo");
}

fn forbidden() -> RecordedTransport {
    RecordedTransport::new().on("/user", 403, USER_FORBIDDEN)
}

#[test]
fn an_app_bot_uses_the_configured_post_as_name() {
    let t = Arc::new(forbidden());
    let (handle, _) = FakeScopes {
        bot: true,
        post_as: Some("poulpi-lab[bot]".into()),
        ..Default::default()
    }
    .handle();
    let mut c = client(&t);
    c.scopes = handle;
    let who = block_on(c.posting_identity(PROJECT)).unwrap();
    assert_eq!(who.login.as_deref(), Some("poulpi-lab[bot]"));
    assert_eq!(who.kind, IdentityKind::AppBot);
    assert_eq!(who.source, TokenSource::RepoTokenCommand);
    assert!(
        t.requests().iter().all(|r| !r.url.contains("/graphql")),
        "a configured name needs no lookup"
    );
}

#[test]
fn an_app_bot_without_a_name_asks_graphql_viewer() {
    let t = Arc::new(forbidden().on("/graphql", 200, VIEWER));
    let who = block_on(client(&t).posting_identity(PROJECT)).unwrap();
    assert_eq!(who.login.as_deref(), Some("poulpi-lab[bot]"));
    assert_eq!(who.kind, IdentityKind::AppBot);
    let body = t.requests().last().unwrap().body.clone().unwrap();
    assert!(body.contains("viewer"));
}

#[test]
fn an_app_bot_the_host_will_not_name_is_unknown_but_still_shown() {
    let t = Arc::new(forbidden().on("/graphql", 200, VIEWER_FORBIDDEN));
    let who = block_on(client(&t).posting_identity(PROJECT)).unwrap();
    assert_eq!(who.login, None);
    assert_eq!(who.kind, IdentityKind::AppBot);
    assert_eq!(who.display, "the GitHub App bot");
}

#[test]
fn a_login_learned_from_an_earlier_post_is_used_before_asking() {
    let t = Arc::new(forbidden());
    let (handle, _) = FakeScopes {
        bot: true,
        learned: std::sync::Mutex::new(Some("learned[bot]".into())),
        ..Default::default()
    }
    .handle();
    let mut c = client(&t);
    c.scopes = handle;
    let who = block_on(c.posting_identity(PROJECT)).unwrap();
    assert_eq!(who.login.as_deref(), Some("learned[bot]"));
    assert!(t.requests().iter().all(|r| !r.url.contains("/graphql")));
}

#[test]
fn an_installation_token_never_calls_user() {
    let t = Arc::new(RecordedTransport::new().on("/graphql", 200, VIEWER));
    let who = block_on(client_with("ghs_abc", &t).posting_identity(PROJECT)).unwrap();
    assert_eq!(who.kind, IdentityKind::AppBot);
    assert!(t.requests().iter().all(|r| !r.url.ends_with("/user")));
}

#[test]
fn an_unreadable_user_answer_is_an_error_not_a_guess() {
    let t = Arc::new(RecordedTransport::new().on("/user", 500, "{}"));
    assert!(block_on(client(&t).posting_identity(PROJECT)).is_err());
}
