//! The local notes' conversation, against real files in a temp folder.

use std::path::Path;

use gitpoulpi_config::{IntegrationKind, LocalMarkdownSettings};

use super::LocalMarkdownClient;
use crate::client::HostingClient;
use crate::conversation::{
    ActorKind, IdentityKind, ItemKey, ItemKind, MarkupFlavor, TimelineEntry, TimelineOpts,
};
use crate::integration::Integration;
use crate::Error;

fn block_on<T>(f: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(f)
}

fn client(dir: &Path) -> LocalMarkdownClient {
    let mut i = Integration::new(
        "notes",
        IntegrationKind::LocalMarkdown,
        dir.to_string_lossy(),
    );
    i.local = Some(LocalMarkdownSettings {
        me: "kai".into(),
        ..LocalMarkdownSettings::default()
    });
    LocalMarkdownClient::new(i)
}

const NOTE: &str = "---\nid: 1\ntitle: Crash on start\nstatus: doing\nlabels: [bug]\ncreated: 2026-11-01T09:00:00\nupdated: 2026-11-01T09:00:00\ncolor: teal\n---\n# Crash on start\n\nSee [[Design]] and #2.\n";

fn setup(note: &str) -> (tempfile::TempDir, LocalMarkdownClient, String) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("1-crash.md"), note).unwrap();
    let c = client(dir.path());
    let project = dir.path().to_string_lossy().into_owned();
    (dir, c, project)
}

fn file(dir: &tempfile::TempDir) -> String {
    std::fs::read_to_string(dir.path().join("1-crash.md")).unwrap()
}

fn timeline(c: &LocalMarkdownClient, p: &str) -> crate::conversation::TimelinePage {
    block_on(c.item_timeline(p, &ItemKey::issue(1), None, &TimelineOpts::default())).unwrap()
}

fn comments(page: &crate::conversation::TimelinePage) -> Vec<crate::conversation::Comment> {
    page.entries
        .iter()
        .map(|e| match e {
            TimelineEntry::Comment(c) => c.clone(),
            other => panic!("{other:?}"),
        })
        .collect()
}

#[test]
fn a_note_without_comments_has_the_header_and_an_empty_timeline() {
    let (_d, c, p) = setup(NOTE);
    let page = timeline(&c, &p);
    let h = &page.header;
    assert_eq!(h.reference, "#1");
    assert_eq!(h.title, "Crash on start");
    assert_eq!(h.state_label, "doing");
    assert!(h.open);
    assert_eq!(h.labels, ["bug"]);
    assert_eq!(h.body.flavor, MarkupFlavor::ObsidianMd);
    assert_eq!(h.body.text, "# Crash on start\n\nSee [[Design]] and #2.\n");
    assert!(h.web_url.starts_with("file://"));
    assert!(page.entries.is_empty() && page.older.is_none());
}

#[test]
fn posting_appends_to_the_note_and_leaves_the_rest_byte_for_byte() {
    let (d, c, p) = setup(NOTE);
    let posted = block_on(c.post_item_comment(
        &p,
        &ItemKey::issue(1),
        "Fixed in **1.2**\nSee [[Design]]",
        false,
    ))
    .unwrap();
    assert_eq!(posted.author.login, "kai");
    assert_eq!(posted.can_edit, Some(true));
    let text = file(&d);
    let body_start = text.find("# Crash on start").unwrap();
    assert!(text[body_start..]
        .starts_with("# Crash on start\n\nSee [[Design]] and #2.\n\n## Comments\n\n### kai - "));
    assert!(
        text.ends_with("Fixed in **1.2**\nSee [[Design]]\n\n"),
        "{text:?}"
    );
    // The frontmatter keeps every key and its order; only `updated` moved.
    assert!(text.starts_with("---\nid: 1\ntitle: Crash on start\nstatus: doing\nlabels: [bug]\ncreated: 2026-11-01T09:00:00\nupdated: "));
    assert!(!text.contains("updated: 2026-11-01T09:00:00"));
    assert!(text.contains("\ncolor: teal\n---\n"));

    let page = timeline(&c, &p);
    let got = comments(&page);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].body.text, "Fixed in **1.2**\nSee [[Design]]");
    assert_eq!(got[0].id, posted.id);
    assert_eq!(
        page.header.body.text,
        "# Crash on start\n\nSee [[Design]] and #2.\n\n"
    );
    assert_eq!(page.total_hint, Some(1));
}

#[test]
fn two_posts_in_one_second_get_distinct_ids() {
    let (_d, c, p) = setup(NOTE);
    let k = ItemKey::issue(1);
    let a = block_on(c.post_item_comment(&p, &k, "one", false)).unwrap();
    let b = block_on(c.post_item_comment(&p, &k, "two", false)).unwrap();
    let e = block_on(c.post_item_comment(&p, &k, "three", false)).unwrap();
    assert_ne!(a.id, b.id);
    assert_ne!(b.id, e.id);
    let got = comments(&timeline(&c, &p));
    assert_eq!(
        got.iter().map(|c| c.body.text.as_str()).collect::<Vec<_>>(),
        ["one", "two", "three"],
        "in the order they were written"
    );
}

#[test]
fn edit_and_delete_work_on_the_local_authors_comments_only() {
    let note = format!(
        "{NOTE}\n## Comments\n\n### ann - 2026-11-02T10:00:00Z\nFrom Ann\n\n### kai - 2026-11-02T11:00:00Z\nMine\n\n### kai - 2026-11-02T12:00:00Z\nAlso mine\n"
    );
    let (d, c, p) = setup(&note);
    let k = ItemKey::issue(1);
    let got = comments(&timeline(&c, &p));
    assert_eq!(
        got.iter().map(|c| c.can_edit).collect::<Vec<_>>(),
        [Some(false), Some(true), Some(true)]
    );
    assert_eq!(got[0].author.kind, ActorKind::User);

    let edited = block_on(c.edit_item_comment(&p, &k, &got[1].id, "Mine, **changed**")).unwrap();
    assert!(edited.edited && edited.updated_at.is_some());
    let text = file(&d);
    assert!(
        text.contains("### kai - 2026-11-02T11:00:00Z (edited "),
        "{text}"
    );
    assert!(text.contains("\nMine, **changed**\n"));
    assert!(
        text.contains("### ann - 2026-11-02T10:00:00Z\nFrom Ann\n"),
        "others untouched"
    );

    block_on(c.delete_item_comment(&p, &k, &got[2].id)).unwrap();
    assert!(!file(&d).contains("Also mine"));
    assert_eq!(comments(&timeline(&c, &p)).len(), 2);

    let denied = block_on(c.edit_item_comment(&p, &k, &got[0].id, "mine now"));
    assert!(matches!(denied, Err(Error::Forbidden { .. })), "{denied:?}");
    assert!(matches!(
        block_on(c.delete_item_comment(&p, &k, &got[0].id)),
        Err(Error::Forbidden { .. })
    ));
    assert!(matches!(
        block_on(c.delete_item_comment(&p, &k, "2000-01-01T00:00:00Z|kai")),
        Err(Error::Local(_))
    ));
    assert!(file(&d).contains("From Ann"));
}

#[test]
fn a_crlf_note_stays_crlf_after_a_post() {
    let (d, c, p) = setup(&NOTE.replace('\n', "\r\n"));
    block_on(c.post_item_comment(&p, &ItemKey::issue(1), "a\nb", false)).unwrap();
    let text = file(&d);
    assert!(text.contains("\r\n## Comments\r\n\r\n### kai - "));
    assert!(text.ends_with("\r\na\r\nb\r\n\r\n"), "{text:?}");
    assert!(
        !text.replace("\r\n", "").contains('\n'),
        "no bare line feed was introduced"
    );
}

#[test]
fn a_note_without_frontmatter_is_not_given_one() {
    let (d, c, p) = setup("# 1 only a title\n\ntext\n");
    block_on(c.post_item_comment(&p, &ItemKey::issue(1), "hi", false)).unwrap();
    let text = file(&d);
    assert!(
        text.starts_with("# 1 only a title\n\ntext\n\n## Comments\n"),
        "{text:?}"
    );
    assert!(!text.starts_with("---"));
}

#[test]
fn a_long_history_is_windowed_with_a_cursor() {
    let mut note = format!("{NOTE}\n## Comments\n\n");
    for n in 0..350u32 {
        note.push_str(&format!(
            "### ann - 2026-11-02T{:02}:{:02}:{:02}Z\nc{n}\n\n",
            n / 3600,
            n % 3600 / 60,
            n % 60
        ));
    }
    let (_d, c, p) = setup(&note);
    let first = timeline(&c, &p);
    assert_eq!(first.entries.len(), 300);
    let second = block_on(c.item_timeline(
        &p,
        &ItemKey::issue(1),
        first.older.as_ref(),
        &TimelineOpts::default(),
    ))
    .unwrap();
    assert_eq!(second.entries.len(), 50);
    assert_eq!(second.older, None);
}

#[test]
fn what_a_note_cannot_do_is_refused_and_capabilities_say_so() {
    let (d, c, p) = setup(NOTE);
    let k = ItemKey::issue(1);
    assert!(matches!(
        block_on(c.post_item_comment(&p, &k, "  ", false)),
        Err(Error::Api { status: 422, .. })
    ));
    assert!(matches!(
        block_on(c.post_item_comment(&p, &k, "x", true)),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        block_on(c.reply_in_discussion(&p, &k, "d", "x")),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        block_on(c.item_timeline(
            &p,
            &ItemKey::pull_request(1),
            None,
            &TimelineOpts::default()
        )),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        block_on(c.item_timeline(&p, &ItemKey::issue(99), None, &TimelineOpts::default())),
        Err(Error::Local(_))
    ));
    assert_eq!(file(&d), NOTE, "nothing was written");
    let caps = c.conversation_caps(ItemKind::Issue);
    assert!(caps.read && caps.comment && caps.edit_own && caps.delete_own);
    assert!(!caps.events && !caps.reply_in_thread && !caps.internal_notes && !caps.reactions);
    assert_eq!(caps.write_flavor, MarkupFlavor::ObsidianMd);
    assert!(!c.conversation_caps(ItemKind::PullRequest).read);
}

#[test]
fn the_identity_is_the_local_author_and_needs_no_network() {
    let (_d, c, _p) = setup(NOTE);
    let me = block_on(c.posting_identity("")).unwrap();
    assert_eq!(me.login.as_deref(), Some("kai"));
    assert_eq!(me.kind, IdentityKind::LocalAuthor);
}
