use gitpoulpi_hosting::conversation::{
    Event, EventKind, ItemKey, Markup, ReviewEntry, TimelineEntry,
};
use gitpoulpi_hosting::review::model::ReviewState;

use super::*;
use crate::hosting_reply::{build, ReplyRequest, ReplyTarget};

fn comment(id: &str, who: &str, text: &str) -> Comment {
    Comment {
        id: id.into(),
        author: Actor::new(who, who, ActorKind::User),
        body: Markup::new(text, MarkupFlavor::Gfm),
        created_at: "2026-11-02".into(),
        updated_at: None,
        edited: false,
        reactions: Vec::new(),
        in_reply_to: None,
        discussion_id: None,
        internal: false,
        can_edit: None,
        can_delete: None,
        web_url: None,
    }
}

fn header() -> ItemHeader {
    ItemHeader {
        key: ItemKey::pull_request(7),
        reference: "#7".into(),
        title: "Fix the crash".into(),
        state_label: "open".into(),
        open: true,
        author: Actor::new("alice", "Alice", ActorKind::Bot),
        body: Markup::new("Fixes #12.", MarkupFlavor::Gfm),
        created_at: "2026-11-01".into(),
        web_url: "https://example.test/7".into(),
        labels: vec!["bug".into(), "ui".into()],
    }
}

fn code_thread() -> CodeThread {
    CodeThread {
        id: "ct1".into(),
        path: "src/a.rs".into(),
        line: Some(12),
        original_line: None,
        side: Default::default(),
        start_line: None,
        resolved: false,
        outdated: true,
        resolvable: true,
        diff_hunk: Some("@@ -1 +1 @@\n-old\n+new".into()),
        comments: vec![comment("k1", "bob", "Why this change?")],
    }
}

#[test]
fn the_item_becomes_a_one_line_summary_and_a_description() {
    let d = item_digest(&header());
    assert_eq!(
        d.summary,
        "Pull request #7 \"Fix the crash\" (open) by alice (bot), labels: bug, ui"
    );
    assert_eq!(d.description, "Fixes #12.");
}

#[test]
fn system_events_are_left_out_and_the_rest_keeps_its_order() {
    let list = entries(&[
        TimelineEntry::Comment(comment("a", "bob", "first")),
        TimelineEntry::Event(Event {
            id: "e".into(),
            actor: None,
            at: "2026-11-02".into(),
            kind: EventKind::Closed,
        }),
        TimelineEntry::Review(ReviewEntry {
            id: "r".into(),
            author: Actor::new("carol", "carol", ActorKind::User),
            state: ReviewState::ChangesRequested,
            body: Markup::new("needs work", MarkupFlavor::Gfm),
            submitted_at: "2026-11-03".into(),
        }),
    ]);
    assert_eq!(list.len(), 2, "the event added nothing");
    assert!(list[0].text.contains("first"));
    assert!(list[1]
        .text
        .starts_with("Review by carol on 2026-11-03 (ChangesRequested):"));
    assert!(list[1].text.contains("needs work"));
}

/// A comment is in the thread of its own id, of its discussion and of the
/// comment it answers.
#[test]
fn a_comment_names_every_thread_it_belongs_to() {
    let mut c = comment("t2", "carol", "x");
    c.discussion_id = Some("d9".into());
    c.in_reply_to = Some("t1".into());
    let list = entries(&[TimelineEntry::Comment(c)]);
    assert_eq!(list[0].thread_ids, ["t2", "d9", "t1"]);
}

#[test]
fn a_code_thread_is_found_by_its_id_and_brings_its_hunk_only_when_answered() {
    let page = [TimelineEntry::Thread(code_thread())];
    let list = entries(&page);
    assert!(list[0]
        .text
        .contains("Code thread on src/a.rs:12 (outdated)"));
    assert!(!list[0].text.contains("```diff"), "no hunk in the list");
    let mut r = ReplyRequest::new(ReplyTarget::ThreadReply {
        thread: "ct1".into(),
    });
    r.entries = list;
    let u = build(&r, 49_152).user;
    assert!(u.contains("```diff\n@@ -1 +1 @@\n-old\n+new\n```"), "{u}");
    assert!(u.contains("Why this change?"));
    // A comment of the thread names it too.
    let by_comment = ReplyTarget::ThreadReply {
        thread: "k1".into(),
    };
    r.target = by_comment;
    assert!(build(&r, 49_152).user.contains("Thread being answered"));
}

#[test]
fn every_flavor_has_a_name() {
    for flavor in [
        MarkupFlavor::Gfm,
        MarkupFlavor::GitlabMd,
        MarkupFlavor::GiteaMd,
        MarkupFlavor::CommonMark,
        MarkupFlavor::AzureMd,
        MarkupFlavor::BitbucketMd,
        MarkupFlavor::Html,
        MarkupFlavor::JiraWiki,
        MarkupFlavor::Textile,
        MarkupFlavor::YoutrackMd,
        MarkupFlavor::TrelloMd,
        MarkupFlavor::PlainText,
        MarkupFlavor::ObsidianMd,
    ] {
        assert!(!flavor_name(flavor).is_empty());
    }
}
