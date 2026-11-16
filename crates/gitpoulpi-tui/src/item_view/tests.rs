//! The conversation view over a fake provider: the surface around the
//! timeline (frame, header, identity line, composer dock, links, Back) is the
//! real code; the paint is read back from a `Buffer`.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_config::ApiFamily;
use gitpoulpi_hosting::conversation::{
    IdentityKind, ItemKey, PostingIdentity, TimelineEntry, TokenSource,
};
use gitpoulpi_hosting::refs::{IssueLinker, PullLinker, RefContext};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::*;
use crate::conversation_ui::tests::{comment, integration, shared, text, Factory, Shared};
use crate::item_cmd::ItemCmd;

const WIDE: Rect = Rect {
    x: 0,
    y: 0,
    width: 130,
    height: 36,
};

fn bases() -> RefContext {
    RefContext {
        issues: Some(IssueLinker {
            family: ApiFamily::Gitlab,
            web_base: "https://gitlab.example.com".into(),
            project: "g/p".into(),
            project_keys: Vec::new(),
            cards: Vec::new(),
        }),
        pulls: Some(PullLinker {
            family: ApiFamily::Gitlab,
            web_base: "https://gitlab.example.com".into(),
            project: "g/p".into(),
        }),
    }
}

fn target(repo_root: Option<std::path::PathBuf>) -> ItemTarget {
    ItemTarget {
        integration: integration(),
        all: vec![integration()],
        project: "g/p".into(),
        key: ItemKey::issue(142),
        reference: "#142".into(),
        title: "Crash when opening a repository".into(),
        web_url: "https://gitlab.example.com/g/p/-/issues/142".into(),
        repo_root,
        bases: bases(),
    }
}

fn entries() -> Vec<TimelineEntry> {
    vec![
        TimelineEntry::Comment(comment(
            "c1",
            "carol",
            "Same as #120, see https://example.org/page and 1a2b3c4d5e6f.",
            "2026-11-12T10:30:00Z",
        )),
        TimelineEntry::Comment(comment("c2", "bob", "Thanks.", "2026-11-13T08:00:00Z")),
    ]
}

struct Rig {
    view: ItemView,
    shared: Arc<Mutex<Shared>>,
}

fn rig_at(entries: Vec<TimelineEntry>, repo_root: Option<std::path::PathBuf>) -> Rig {
    let shared = shared(entries);
    let factory: Arc<dyn ClientFactory> = Arc::new(Factory(shared.clone()));
    let mut view = ItemView::open(target(repo_root), factory);
    wait(&mut view, "the conversation", |v| {
        v.conv.header.is_some() && v.conv.identity.is_some() && !v.conv.busy()
    });
    Rig { view, shared }
}

fn rig() -> Rig {
    rig_at(entries(), None)
}

/// A rig on a project of its own: the notification log is shared by every
/// test, so a test that reads it names its project.
fn rig_in(project: &str) -> Rig {
    let shared = shared(entries());
    let factory: Arc<dyn ClientFactory> = Arc::new(Factory(shared.clone()));
    let mut t = target(None);
    t.project = project.into();
    let mut view = ItemView::open(t, factory);
    wait(&mut view, "the conversation", |v| {
        v.conv.header.is_some() && v.conv.identity.is_some() && !v.conv.busy()
    });
    Rig { view, shared }
}

fn wait(view: &mut ItemView, what: &str, done: impl Fn(&ItemView) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        view.poll();
        if done(view) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn draw(view: &mut ItemView, area: Rect) -> Buffer {
    let mut buf = Buffer::empty(area);
    view.render(area, &mut buf);
    buf
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn click(col: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn link_cell(view: &ItemView, needle: &str) -> (u16, u16) {
    let hit = view
        .conv
        .hits
        .iter()
        .find(|(_, h)| match h {
            crate::conversation_ui::Hit::Link(i) => view.conv.link(*i).is_some_and(|l| {
                format!("{:?}", l.xref).contains(needle) || l.url.contains(needle)
            }),
            _ => false,
        })
        .unwrap_or_else(|| panic!("no link to {needle}: {:?}", view.conv.links));
    (hit.0.x, hit.0.y)
}

#[test]
fn the_header_names_the_item_and_who_posts() {
    let mut r = rig();
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(
        screen.contains("#142  Crash when opening a repository"),
        "{screen}"
    );
    assert!(screen.contains("\u{25CF} open"), "{screen}");
    assert!(
        screen.contains("opened by alice on 2026-11-12 09:14"),
        "{screen}"
    );
    assert!(screen.contains("bug  lfs"), "{screen}");
    assert!(screen.contains("2 comments"), "{screen}");
    assert!(
        screen.contains("Posting as bob (personal token)"),
        "{screen}"
    );
    assert!(
        screen.contains("Open in browser") && screen.contains("Refresh"),
        "{screen}"
    );
}

#[test]
fn an_app_bot_without_a_known_login_is_named_and_a_known_one_is_shown() {
    let mut r = rig();
    r.shared.lock().unwrap().identity = Ok(PostingIdentity::new(
        None,
        IdentityKind::AppBot,
        TokenSource::RepoTokenCommand,
    ));
    let identity = r.shared.lock().unwrap().identity.clone();
    r.view.conv.set_identity(identity);
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(
        screen.contains("Posting as the GitHub App bot (its name is not known yet"),
        "{screen}"
    );
    r.view.conv.set_identity(Ok(PostingIdentity::new(
        Some("my-app[bot]".into()),
        IdentityKind::AppBot,
        TokenSource::RepoTokenCommand,
    )));
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(
        screen.contains("Posting as my-app[bot] (GitHub App bot)"),
        "{screen}"
    );
}

#[test]
fn a_host_that_takes_no_comments_says_read_only_and_has_no_composer() {
    let mut r = rig();
    r.shared.lock().unwrap().caps.comment = false;
    let caps = r.shared.lock().unwrap().caps.clone();
    r.view.conv.caps = Some(caps);
    r.view.md = None;
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(
        screen.contains("Read only: this host does not take comments from here."),
        "{screen}"
    );
    assert!(!screen.contains("New comment"), "{screen}");
    assert_eq!(
        ItemCmd::NewComment.unavailable(Some(r.view.avail())),
        Some(gitpoulpi_i18n::msg::item_view::WHY_NO_COMPOSER.get())
    );
}

#[test]
fn the_close_cross_and_esc_close_a_clean_view_and_a_typed_one_asks() {
    let mut r = rig();
    let _ = draw(&mut r.view, WIDE);
    let cross = r
        .view
        .guard
        .close
        .rect()
        .expect("the frame paints a close cross");
    assert_eq!(r.view.on_mouse(click(cross.x, cross.y)), ItemEffect::Close);
    assert_eq!(r.view.on_key(key(KeyCode::Esc)), ItemEffect::Close);
    assert_eq!(r.view.on_key(key(KeyCode::Char('q'))), ItemEffect::Close);

    // Typed text: Esc in the field goes back to the list, a second Esc asks.
    r.view.on_key(key(KeyCode::Char('c')));
    r.view.on_key(key(KeyCode::Char('x')));
    let _ = draw(&mut r.view, WIDE);
    assert_eq!(r.view.on_key(key(KeyCode::Esc)), ItemEffect::None);
    assert_eq!(r.view.focus, Focus::Timeline);
    assert_eq!(r.view.on_key(key(KeyCode::Esc)), ItemEffect::None);
    assert!(
        r.view.guard.is_open(),
        "a typed reply is not thrown away silently"
    );
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(screen.contains("Discard this text?"), "{screen}");
    // Keep editing stays; the discard closes.
    assert_eq!(r.view.on_key(key(KeyCode::Esc)), ItemEffect::None);
    assert!(!r.view.guard.is_open());
    r.view.on_key(key(KeyCode::Esc));
    r.view.on_key(key(KeyCode::Right));
    assert_eq!(r.view.on_key(key(KeyCode::Enter)), ItemEffect::Close);
}

#[test]
fn a_reference_opens_in_the_view_and_back_returns() {
    let mut r = rig();
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Down));
    let _ = draw(&mut r.view, WIDE);
    let (x, y) = link_cell(&r.view, "120");
    assert_eq!(r.view.on_mouse(click(x, y)), ItemEffect::None);
    assert_eq!(r.view.depth(), 1);
    wait(&mut r.view, "the second item", |v| v.conv.header.is_some());
    assert!(
        r.shared
            .lock()
            .unwrap()
            .reads
            .iter()
            .any(|(p, k)| p == "g/p" && k.number == 120),
        "{:?}",
        r.shared.lock().unwrap().reads
    );
    assert_eq!(r.view.target.reference, "#120");
    // Back: the button, the key, Backspace.
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(screen.contains("Back"), "{screen}");
    assert_eq!(
        r.view
            .on_key(KeyEvent::new(KeyCode::Left, KeyModifiers::ALT)),
        ItemEffect::None
    );
    assert_eq!(r.view.depth(), 0);
    assert_eq!(r.view.target.reference, "#142");
    assert_eq!(r.view.on_key(key(KeyCode::Backspace)), ItemEffect::None);
    assert!(r.view.message.is_some(), "nothing to go back to says so");
}

#[test]
fn a_commit_reveals_in_the_log_and_an_external_address_asks_first() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = rig_at(entries(), Some(dir.path().to_path_buf()));
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Down));
    let _ = draw(&mut r.view, WIDE);
    let (x, y) = link_cell(&r.view, "Commit");
    assert_eq!(
        r.view.on_mouse(click(x, y)),
        ItemEffect::ShowCommit("1a2b3c4d5e6f".into())
    );
    // An address typed in the text goes through the confirm dialog.
    assert!(!crate::link_confirm::is_open());
    let (x, y) = link_cell(&r.view, "example.org");
    assert_eq!(r.view.on_mouse(click(x, y)), ItemEffect::None);
    assert!(
        crate::link_confirm::is_open(),
        "the real address is asked about first"
    );
    crate::link_confirm::on_key(key(KeyCode::Esc));
    assert!(!crate::link_confirm::is_open());
}

#[test]
fn without_a_clone_a_commit_is_a_web_address() {
    let mut r = rig();
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Down));
    let _ = draw(&mut r.view, WIDE);
    let (x, y) = link_cell(&r.view, "Commit");
    assert_eq!(r.view.on_mouse(click(x, y)), ItemEffect::None);
    assert!(crate::link_confirm::is_open());
    crate::link_confirm::on_key(key(KeyCode::Esc));
}

#[test]
fn enter_follows_the_only_link_and_l_lists_several() {
    let mut r = rig();
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Down));
    // Three links in carol's comment: Enter and `l` open the menu.
    assert_eq!(r.view.on_key(key(KeyCode::Enter)), ItemEffect::None);
    assert!(r.view.menu.is_some());
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(screen.contains("issue #120"), "{screen}");
    // Down, Enter on "issue #120" is the first row: pick it.
    r.view.on_key(key(KeyCode::Enter));
    assert!(r.view.menu.is_none());
    assert_eq!(r.view.depth(), 1);
}

#[test]
fn a_70_column_window_disables_split_and_preview_works() {
    let mut r = rig();
    let narrow = Rect::new(0, 0, 70, 30);
    let screen = text(&draw(&mut r.view, narrow));
    assert!(screen.contains("Split"), "{screen}");
    assert_eq!(
        ItemCmd::ToggleSplit.unavailable(Some(r.view.avail())),
        Some(gitpoulpi_i18n::msg::md_editor::SPLIT_NEEDS_WIDTH.get())
    );
    r.view.on_key(key(KeyCode::Char('c')));
    for c in "**bold**".chars() {
        r.view.on_key(key(KeyCode::Char(c)));
    }
    r.view
        .on_key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::ALT));
    let screen = text(&draw(&mut r.view, narrow));
    assert!(
        screen.contains("bold") && !screen.contains("**bold**"),
        "{screen}"
    );
    // On a wide window the same command is available.
    let _ = draw(&mut r.view, WIDE);
    assert_eq!(ItemCmd::ToggleSplit.unavailable(Some(r.view.avail())), None);
}

#[test]
fn palette_commands_carry_their_reasons() {
    let mut r = rig();
    let _ = draw(&mut r.view, WIDE);
    let a = r.view.avail();
    use gitpoulpi_i18n::msg::item_view as m;
    assert_eq!(
        ItemCmd::Back.unavailable(Some(a)),
        Some(m::WHY_NO_BACK.get())
    );
    assert_eq!(
        ItemCmd::LoadOlder.unavailable(Some(a)),
        Some(m::WHY_NO_OLDER.get())
    );
    assert_eq!(
        ItemCmd::NextUnread.unavailable(Some(a)),
        Some(m::WHY_NO_UNREAD.get())
    );
    assert_eq!(
        ItemCmd::Edit.unavailable(Some(a)),
        Some(m::WHY_NOT_YOURS.get())
    );
    assert_eq!(ItemCmd::Reply.unavailable(Some(a)), None);
    assert_eq!(ItemCmd::Refresh.unavailable(Some(a)), None);
    assert_eq!(ItemCmd::Close.unavailable(Some(a)), None);
    assert_eq!(ItemCmd::Close.unavailable(None), Some(m::WHY_NO_VIEW.get()));
    // Bob's own comment: Edit and Delete are allowed by the login; Send has
    // nothing to send yet.
    r.view.conv.select(2);
    let a = r.view.avail();
    assert_eq!(ItemCmd::Edit.unavailable(Some(a)), None);
    assert_eq!(ItemCmd::Delete.unavailable(Some(a)), None);
    assert_eq!(ItemCmd::Send.unavailable(Some(a)), Some(m::WHY_EMPTY.get()));
}

#[test]
fn reply_quotes_the_entry_into_the_composer() {
    let mut r = rig();
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Down));
    assert_eq!(r.view.on_key(key(KeyCode::Char('r'))), ItemEffect::None);
    assert_eq!(r.view.focus, Focus::Composer);
    let text_in = r.view.md.as_ref().unwrap().text().to_string();
    assert!(
        text_in.starts_with("> **carol**\n> Same as #120"),
        "{text_in}"
    );
}

#[test]
fn buttons_are_reachable_with_tab_and_enter() {
    let mut r = rig();
    let _ = draw(&mut r.view, WIDE);
    // Timeline, composer, then Back, Refresh, Browser, Events.
    r.view.on_key(key(KeyCode::Tab));
    assert_eq!(r.view.focus, Focus::Composer);
    r.view.on_key(key(KeyCode::Tab));
    assert_eq!(r.view.focus, Focus::Bar(Bar::Back));
    r.view.on_key(key(KeyCode::Right));
    assert_eq!(r.view.focus, Focus::Bar(Bar::Refresh));
    let before = r.shared.lock().unwrap().reads.len();
    r.view.on_key(key(KeyCode::Enter));
    wait(&mut r.view, "the refresh", |v| !v.conv.busy());
    assert!(
        r.shared.lock().unwrap().reads.len() > before,
        "Enter on Refresh reads again"
    );
    // The events checkbox toggles with Space.
    r.view.on_key(key(KeyCode::Right));
    r.view.on_key(key(KeyCode::Right));
    assert_eq!(r.view.focus, Focus::Bar(Bar::Events));
    assert!(r.view.conv.events);
    r.view.on_key(key(KeyCode::Char(' ')));
    assert!(!r.view.conv.events);
}

#[test]
fn the_palette_key_is_the_views_while_nothing_has_the_keyboard() {
    let mut r = rig();
    assert!(r.view.takes_palette());
    r.view.on_key(key(KeyCode::Char('c')));
    assert!(!r.view.takes_palette(), "a focused composer owns the keys");
    r.view.on_key(key(KeyCode::Esc));
    assert!(r.view.takes_palette());
}

#[test]
fn the_seen_mark_is_left_when_the_view_goes_away() {
    let dir = tempfile::tempdir().unwrap();
    let t = {
        let r = rig_at(entries(), Some(dir.path().to_path_buf()));
        let t = r.view.target.clone();
        drop(r);
        t
    };
    let state = gitpoulpi_session::RepoState::load(&t.state_path().unwrap()).unwrap();
    let seen = state
        .conversation_seen
        .get(&t.seen_key())
        .expect("a visit is recorded");
    assert_eq!(seen.at, "2026-11-13T08:00:00Z");
    // A second visit sees nothing new; a newer comment is flagged.
    let s = shared(entries());
    if let Ok(p) = &mut s.lock().unwrap().page {
        p.entries.push(TimelineEntry::Comment(comment(
            "c3",
            "carol",
            "One more.",
            "2026-11-14T08:00:00Z",
        )));
    }
    let factory: Arc<dyn ClientFactory> = Arc::new(Factory(s.clone()));
    let mut view = ItemView::open(t, factory);
    wait(&mut view, "the second visit", |v| {
        v.conv.header.is_some() && v.conv.identity.is_some() && !v.conv.busy()
    });
    let _ = draw(&mut view, WIDE);
    assert_eq!(view.conv.unread_count(), 1);
}

// -- writing ------------------------------------------------------------------------

fn ctrl_enter() -> KeyEvent {
    KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL)
}

fn type_text(view: &mut ItemView, text: &str) {
    for c in text.chars() {
        if c == '\n' {
            view.on_key(key(KeyCode::Enter));
        } else {
            view.on_key(key(KeyCode::Char(c)));
        }
    }
}

fn wait_written(view: &mut ItemView) {
    wait(view, "the write", |v| v.conv.take_idle_for_tests());
}

fn log(r: &Rig) -> Vec<String> {
    r.shared.lock().unwrap().write_log.clone()
}

fn entry_ids(view: &ItemView) -> Vec<String> {
    view.conv
        .entries
        .iter()
        .filter_map(|e| match e {
            TimelineEntry::Comment(c) => Some(c.id.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_comment_shows_at_once_goes_to_the_host_as_typed_and_clears_the_box() {
    let mut r = rig();
    r.shared.lock().unwrap().write_delay = Duration::from_millis(300);
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Char('c')));
    type_text(&mut r.view, "Thanks\nagain");
    let _ = draw(&mut r.view, WIDE);
    assert_eq!(r.view.on_key(ctrl_enter()), ItemEffect::None);
    // Before the host answers: the entry is there, pending, and the view says so.
    assert!(r.view.conv.is_writing());
    assert_eq!(entry_ids(&r.view), ["c1", "c2", "pending:1"]);
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(screen.contains("Sending..."), "{screen}");
    wait_written(&mut r.view);
    assert_eq!(log(&r), ["post||Thanks\nagain"]);
    assert_eq!(
        entry_ids(&r.view),
        ["c1", "c2", "901"],
        "the host's own copy replaced the pending one"
    );
    assert_eq!(r.view.md.as_ref().unwrap().text(), "", "the box is empty");
    assert_eq!(r.view.compose, Compose::New);
    assert_eq!(r.view.focus, Focus::Timeline);
    // Posting to a host is outward: Undo says it cannot take it back.
    assert!(
        crate::undo_ui::nothing_to_undo().contains("comment posted to a host"),
        "{}",
        crate::undo_ui::nothing_to_undo()
    );
}

#[test]
fn a_refused_comment_goes_back_with_its_text_and_the_error_under_it() {
    let mut r = rig();
    r.shared.lock().unwrap().fail_write = Some("403 Forbidden".into());
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Char('c')));
    type_text(&mut r.view, "Keep me");
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(ctrl_enter());
    wait_written(&mut r.view);
    assert_eq!(
        entry_ids(&r.view),
        ["c1", "c2"],
        "the optimistic entry is gone"
    );
    assert_eq!(r.view.md.as_ref().unwrap().text(), "Keep me");
    assert_eq!(r.view.focus, Focus::Composer);
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(screen.contains("not sent: 403 Forbidden"), "{screen}");
    assert!(screen.contains("Keep me"), "{screen}");
    // Sending again works and clears the error.
    r.view.on_key(ctrl_enter());
    wait_written(&mut r.view);
    assert_eq!(log(&r).len(), 2);
    assert_eq!(r.view.compose_error, None);
    assert_eq!(r.view.md.as_ref().unwrap().text(), "");
}

#[test]
fn send_is_greyed_with_its_reason_until_there_is_text() {
    let mut r = rig();
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Char('c')));
    let _ = draw(&mut r.view, WIDE);
    let why = gitpoulpi_i18n::msg::item_view::WHY_EMPTY.get();
    assert_eq!(r.view.send_reason(), Some(why));
    let (rect, _) = *r.view.hits.iter().find(|(_, h)| *h == Hit::Send).unwrap();
    assert_eq!(r.view.on_mouse(click(rect.x, rect.y)), ItemEffect::None);
    assert_eq!(r.view.message.as_deref(), Some(why));
    assert_eq!(r.shared.lock().unwrap().writes, 0, "nothing was posted");
    type_text(&mut r.view, "x");
    let _ = draw(&mut r.view, WIDE);
    assert_eq!(r.view.send_reason(), None);
    // The Send button posts too.
    let (rect, _) = *r.view.hits.iter().find(|(_, h)| *h == Hit::Send).unwrap();
    r.view.on_mouse(click(rect.x, rect.y));
    wait_written(&mut r.view);
    assert_eq!(log(&r), ["post||x"]);
}

fn thread_entries() -> Vec<TimelineEntry> {
    let mut c1 = comment("c1", "carol", "Why not iterate?", "2026-11-12T10:30:00Z");
    c1.discussion_id = Some("d1".into());
    vec![
        TimelineEntry::Comment(c1),
        TimelineEntry::Comment(comment("c2", "bob", "Thanks.", "2026-11-13T08:00:00Z")),
    ]
}

#[test]
fn a_reply_goes_into_the_discussion_and_a_quote_into_a_new_comment() {
    let mut r = rig_at(thread_entries(), None);
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Down));
    // `r`: a reply into carol's discussion.
    assert_eq!(r.view.on_key(key(KeyCode::Char('r'))), ItemEffect::None);
    assert!(
        matches!(r.view.compose, Compose::Reply { .. }),
        "{:?}",
        r.view.compose
    );
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(screen.contains("Reply to carol"), "{screen}");
    type_text(&mut r.view, "Because");
    r.view.on_key(ctrl_enter());
    wait_written(&mut r.view);
    assert_eq!(log(&r), ["reply|d1|Because"]);
    assert_eq!(r.view.compose, Compose::New);
    // `Q`: the entry quoted into a new comment (also a button on a discussion).
    r.view.on_key(key(KeyCode::Esc));
    r.view.conv.select(1);
    assert_eq!(r.view.on_key(key(KeyCode::Char('Q'))), ItemEffect::None);
    assert_eq!(r.view.compose, Compose::New);
    let quoted = r.view.md.as_ref().unwrap().text().to_string();
    assert!(
        quoted.starts_with("> **carol**\n> Why not iterate?"),
        "{quoted}"
    );
    r.view.focus = Focus::Timeline;
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(screen.contains("Quote"), "the button is there: {screen}");
}

#[test]
fn an_own_comment_is_edited_with_its_text_and_unchanged_text_is_refused() {
    let mut r = rig();
    let _ = draw(&mut r.view, WIDE);
    r.view.conv.select(2);
    assert_eq!(r.view.on_key(key(KeyCode::Char('e'))), ItemEffect::None);
    assert!(matches!(r.view.compose, Compose::Edit { .. }));
    assert_eq!(r.view.md.as_ref().unwrap().text(), "Thanks.");
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(
        screen.contains("Edit comment") && screen.contains("Save"),
        "{screen}"
    );
    // Nothing changed: Save says so and the host is not asked.
    r.view.on_key(ctrl_enter());
    assert_eq!(
        r.view.message.as_deref(),
        Some(gitpoulpi_i18n::msg::item_view::WHY_UNCHANGED.get())
    );
    assert_eq!(r.shared.lock().unwrap().writes, 0);
    assert!(!r.view.composer_dirty());
    type_text(&mut r.view, " a lot");
    r.view.on_key(ctrl_enter());
    wait_written(&mut r.view);
    assert_eq!(log(&r), ["edit|c2|Thanks. a lot"]);
    match &r.view.conv.entries[1] {
        TimelineEntry::Comment(c) => assert_eq!(c.body.text, "Thanks. a lot"),
        other => panic!("{other:?}"),
    }
    assert_eq!(r.view.compose, Compose::New);
}

#[test]
fn delete_asks_first_and_keep_does_nothing() {
    let mut r = rig();
    let _ = draw(&mut r.view, WIDE);
    r.view.conv.select(2);
    r.view.on_key(key(KeyCode::Char('d')));
    assert!(r.view.delete_ask.is_some());
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(screen.contains("Delete this comment?"), "{screen}");
    assert!(!r.view.takes_palette());
    // Esc keeps it.
    r.view.on_key(key(KeyCode::Esc));
    assert!(r.view.delete_ask.is_none());
    assert_eq!(r.shared.lock().unwrap().writes, 0);
    // Delete: Tab to the button, Enter.
    r.view.on_key(key(KeyCode::Char('d')));
    r.view.on_key(key(KeyCode::Tab));
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Enter));
    assert!(r.view.conv.is_writing());
    assert_eq!(entry_ids(&r.view), ["c1"], "gone at once");
    wait_written(&mut r.view);
    assert_eq!(log(&r), ["delete|c2|"]);
}

#[test]
fn a_refused_delete_brings_the_comment_back_and_says_why() {
    let mut r = rig();
    r.shared.lock().unwrap().fail_write = Some("500 oops".into());
    let _ = draw(&mut r.view, WIDE);
    r.view.conv.select(2);
    r.view.on_key(key(KeyCode::Char('d')));
    r.view.on_key(key(KeyCode::Tab));
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Enter));
    wait_written(&mut r.view);
    assert_eq!(entry_ids(&r.view), ["c1", "c2"]);
    assert_eq!(r.view.message.as_deref(), Some("not deleted: 500 oops"));
}

#[test]
fn edit_and_delete_follow_the_provider_and_the_author() {
    use gitpoulpi_i18n::msg::item_view as m;
    let mut r = rig();
    let _ = draw(&mut r.view, WIDE);
    // carol's comment is not mine.
    r.view.conv.select(1);
    assert_eq!(r.view.avail().edit, Some(m::WHY_NOT_YOURS.get()));
    assert_eq!(r.view.avail().delete, Some(m::WHY_NOT_YOURS.get()));
    r.view.conv.select(2);
    assert_eq!(r.view.avail().edit, None);
    // A host that lets nobody edit or delete says so, mine or not.
    let mut caps = r.view.conv.caps.clone().unwrap();
    caps.edit_own = false;
    caps.delete_own = false;
    r.view.conv.caps = Some(caps);
    assert_eq!(r.view.avail().edit, Some(m::WHY_NO_EDIT.get()));
    assert_eq!(r.view.avail().delete, Some(m::WHY_NO_DELETE.get()));
    assert_eq!(
        r.view.on_key(key(KeyCode::Char('e'))),
        ItemEffect::None,
        "the reason goes to the view's last row"
    );
    assert_eq!(r.view.message.as_deref(), Some(m::WHY_NO_EDIT.get()));
}

fn bot_comment(id: &str) -> TimelineEntry {
    let mut c = comment(id, "my-app[bot]", "A bot says hi", "2026-11-13T09:00:00Z");
    c.author.kind = gitpoulpi_hosting::conversation::ActorKind::Bot;
    TimelineEntry::Comment(c)
}

fn unknown_bot() -> Result<PostingIdentity, String> {
    Ok(PostingIdentity::new(
        None,
        IdentityKind::AppBot,
        TokenSource::RepoTokenCommand,
    ))
}

#[test]
fn an_app_bot_with_no_known_login_edits_only_what_this_install_posted() {
    use gitpoulpi_i18n::msg::item_view as m;
    let dir = tempfile::tempdir().unwrap();
    // The state file says this install posted c8, not c9.
    let t = target(Some(dir.path().to_path_buf()));
    let path = t.state_path().unwrap();
    let mut state = gitpoulpi_session::RepoState::default();
    state.record_posted_comment(&t.seen_key(), "c8");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    state.save(&path).unwrap();
    let s = shared(vec![bot_comment("c8"), bot_comment("c9")]);
    s.lock().unwrap().identity = unknown_bot();
    let factory: Arc<dyn ClientFactory> = Arc::new(Factory(s));
    let mut view = ItemView::open(t, factory);
    wait(&mut view, "the conversation", |v| {
        v.conv.header.is_some() && v.conv.identity.is_some() && !v.conv.busy()
    });
    let _ = draw(&mut view, WIDE);
    view.conv.select(1);
    assert_eq!(view.avail().edit, None, "c8 was posted here");
    assert_eq!(view.avail().delete, None);
    view.conv.select(2);
    assert_eq!(
        view.avail().edit,
        Some(m::WHY_NOT_YOURS.get()),
        "c9 may be anyone's: never guessed"
    );
    // Once the login is known the author decides, bot or person.
    view.conv.set_identity(Ok(PostingIdentity::new(
        Some("my-app[bot]".into()),
        IdentityKind::AppBot,
        TokenSource::RepoTokenCommand,
    )));
    assert_eq!(view.avail().edit, None);
}

#[test]
fn what_this_install_posted_is_kept_by_the_worker_and_found_again() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = rig_at(entries(), Some(dir.path().to_path_buf()));
    r.shared.lock().unwrap().identity = unknown_bot();
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Char('c')));
    type_text(&mut r.view, "from the bot");
    r.view.on_key(ctrl_enter());
    wait_written(&mut r.view);
    let state = gitpoulpi_session::RepoState::load(&r.view.target.state_path().unwrap()).unwrap();
    assert_eq!(
        state.posted_comments_of(&r.view.target.seen_key()),
        vec!["901".to_string()]
    );
    // The new comment may be edited at once, though the login is unknown.
    let _ = draw(&mut r.view, WIDE);
    r.view.conv.select(r.view.conv.items() - 1);
    assert_eq!(r.view.avail().edit, None);
}

#[test]
fn another_client_posting_makes_the_open_view_read_again() {
    use gitpoulpi_ipc::{HostingChange, HostingChangeArgs, HostingItemKind};
    let mut r = rig();
    let reads = r.shared.lock().unwrap().reads.len();
    let change = |number: u64| HostingChangeArgs {
        integration_id: "corp".into(),
        project: "g/p".into(),
        kind: HostingItemKind::Issue,
        number,
        id: String::new(),
        change: HostingChange::Posted,
        origin: String::new(),
    };
    r.view.on_hosting_changed(&change(7));
    r.view.poll();
    assert_eq!(r.shared.lock().unwrap().reads.len(), reads, "another item");
    r.view.on_hosting_changed(&change(142));
    wait(&mut r.view, "the refresh", |v| {
        v.conv.since_fresh() < Duration::from_millis(500) && !v.conv.busy()
    });
    assert!(r.shared.lock().unwrap().reads.len() > reads);
}

#[test]
fn an_internal_note_is_a_checkbox_with_a_key_where_the_host_has_them() {
    let mut r = rig();
    let _ = draw(&mut r.view, WIDE);
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(!screen.contains("Internal note"), "{screen}");
    assert_eq!(
        ItemCmd::ToggleInternal.unavailable(Some(r.view.avail())),
        Some(gitpoulpi_i18n::msg::item_view::WHY_NO_INTERNAL.get())
    );
    let mut caps = r.view.conv.caps.clone().unwrap();
    caps.internal_notes = true;
    r.view.conv.caps = Some(caps);
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(screen.contains("[ ] Internal note"), "{screen}");
    r.view.on_key(key(KeyCode::Char('c')));
    type_text(&mut r.view, "for members");
    // The key twin, then the mouse twin, toggle it.
    r.view
        .on_key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::ALT));
    assert!(r.view.internal);
    let _ = draw(&mut r.view, WIDE);
    let (rect, _) = *r
        .view
        .hits
        .iter()
        .find(|(_, h)| *h == Hit::Internal)
        .unwrap();
    r.view.on_mouse(click(rect.x, rect.y));
    assert!(!r.view.internal);
    r.view.run(ItemCmd::ToggleInternal);
    assert!(r.view.internal);
    r.view.on_key(ctrl_enter());
    wait_written(&mut r.view);
    assert_eq!(log(&r), ["post|internal|for members"]);
}

#[test]
fn cancel_puts_only_the_box_away_and_never_closes_the_view() {
    let mut r = rig();
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Char('c')));
    type_text(&mut r.view, "typed");
    let _ = draw(&mut r.view, WIDE);
    let (rect, _) = *r.view.hits.iter().find(|(_, h)| *h == Hit::Cancel).unwrap();
    assert_eq!(r.view.on_mouse(click(rect.x, rect.y)), ItemEffect::None);
    assert!(r.view.guard.is_open(), "typed text asks first");
    // Discard is the second button.
    r.view.on_key(key(KeyCode::Right));
    assert_eq!(r.view.on_key(key(KeyCode::Enter)), ItemEffect::None);
    assert!(!r.view.guard.is_open());
    r.view.poll();
    assert_eq!(r.view.md.as_ref().unwrap().text(), "");
    assert_eq!(r.view.focus, Focus::Timeline);
}

#[test]
fn while_a_comment_is_sent_the_box_and_the_actions_wait() {
    use gitpoulpi_i18n::msg::item_view as m;
    let mut r = rig();
    r.shared.lock().unwrap().write_delay = Duration::from_millis(400);
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Char('c')));
    type_text(&mut r.view, "one");
    r.view.on_key(ctrl_enter());
    assert_eq!(r.view.send_reason(), Some(m::WHY_SENDING.get()));
    type_text(&mut r.view, "more");
    assert_eq!(r.view.md.as_ref().unwrap().text(), "one", "input is held");
    assert_eq!(r.view.conv.reply_reason(), Some(m::WHY_SENDING.get()));
    wait_written(&mut r.view);
    assert_eq!(log(&r).len(), 1);
}

#[test]
fn who_posts_is_named_for_every_kind_of_token() {
    let id = |login: Option<&str>, kind, source| {
        super::draw::identity_line(&PostingIdentity::new(
            login.map(str::to_string),
            kind,
            source,
        ))
    };
    let profile = TokenSource::Profile { id: "corp".into() };
    assert_eq!(
        id(Some("bob"), IdentityKind::PersonalToken, profile.clone()),
        "Posting as bob (personal token)"
    );
    assert_eq!(
        id(
            Some("my-app[bot]"),
            IdentityKind::AppBot,
            TokenSource::RepoTokenCommand
        ),
        "Posting as my-app[bot] (GitHub App bot)"
    );
    assert!(
        id(None, IdentityKind::AppBot, TokenSource::RepoTokenCommand)
            .starts_with("Posting as the GitHub App bot (its name is not known yet")
    );
    assert_eq!(
        id(Some("tok"), IdentityKind::ProjectOrGroupBot, profile),
        "Posting as tok (project bot)"
    );
}

#[test]
fn a_host_that_does_not_read_markdown_says_what_the_text_is_sent_as() {
    use gitpoulpi_hosting::conversation::MarkupFlavor;
    let mut r = rig();
    let mut caps = r.view.conv.caps.clone().unwrap();
    caps.write_flavor = MarkupFlavor::JiraWiki;
    r.view.conv.caps = Some(caps);
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Char('c')));
    type_text(&mut r.view, "**hi**");
    r.view
        .on_key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::ALT));
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(screen.contains("Sent as Jira wiki markup"), "{screen}");
    // Markdown hosts say nothing of the kind.
    let mut caps = r.view.conv.caps.clone().unwrap();
    caps.write_flavor = MarkupFlavor::GitlabMd;
    r.view.conv.caps = Some(caps);
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(!screen.contains("Sent as"), "{screen}");
}

#[cfg(feature = "mermaid")]
#[test]
fn g_opens_the_diagram_and_m_stays_load_older() {
    let body = "see\n\n```mermaid\ngraph TD\n  A-->B\n```\n";
    let mut r = rig_at(
        vec![TimelineEntry::Comment(comment(
            "c1",
            "carol",
            body,
            "2026-11-12T10:30:00Z",
        ))],
        None,
    );
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Down));
    // `m` is Load older (nothing older here), never the diagram.
    let effect = r.view.on_key(key(KeyCode::Char('m')));
    assert!(!matches!(effect, ItemEffect::OpenDiagram(_)), "{effect:?}");
    assert_eq!(
        r.view.message.as_deref(),
        Some(gitpoulpi_i18n::msg::item_view::WHY_NO_OLDER.get())
    );
    // `g` is the key the palette shows for Open Diagram.
    let effect = r.view.on_key(key(KeyCode::Char('g')));
    assert!(
        matches!(&effect, ItemEffect::OpenDiagram(s) if s.contains("A-->B")),
        "{effect:?}"
    );
    let hint = crate::palette::PALETTE_COMMANDS
        .iter()
        .find(|e| {
            matches!(
                e.command,
                crate::palette::PaletteCommand::Item(ItemCmd::OpenDiagram)
            )
        })
        .map(|e| e.shortcut);
    assert_eq!(hint, Some("g"));
}

#[test]
fn azure_devops_server_comments_say_they_are_sent_as_html() {
    use gitpoulpi_hosting::conversation::MarkupFlavor;
    let mut r = rig();
    let mut caps = r.view.conv.caps.clone().unwrap();
    caps.write_flavor = MarkupFlavor::Html;
    r.view.conv.caps = Some(caps);
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Char('c')));
    type_text(&mut r.view, "**hi**");
    r.view
        .on_key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::ALT));
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(screen.contains("Sent as HTML"), "{screen}");
}

#[test]
fn a_text_over_the_hosts_limit_shows_the_count_and_is_refused_before_the_host() {
    let mut r = rig();
    let mut caps = r.view.conv.caps.clone().unwrap();
    caps.max_body = Some(10);
    r.view.conv.caps = Some(caps);
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Char('c')));
    type_text(&mut r.view, "0123456789");
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(
        screen.contains("10 / 10"),
        "the count and the limit: {screen}"
    );
    assert_eq!(r.view.send_reason(), None, "at the limit is fine");
    type_text(&mut r.view, "x");
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(screen.contains("11 / 10"), "{screen}");
    let why = gitpoulpi_i18n::msg::item_view::WHY_TOO_LONG.get();
    assert_eq!(r.view.send_reason(), Some(why));
    assert_eq!(r.view.on_key(ctrl_enter()), ItemEffect::None);
    assert_eq!(r.view.message.as_deref(), Some(why));
    assert_eq!(
        r.shared.lock().unwrap().writes,
        0,
        "nothing reached the host"
    );
}

#[test]
fn every_write_of_this_window_tells_the_daemon_what_changed() {
    use gitpoulpi_ipc::HostingChange as C;
    let project = "notify/writes";
    let mut r = rig_in(project);
    let changes = || {
        crate::hosting_notify::testing::sent(project)
            .iter()
            .map(|a| (a.change, a.number))
            .collect::<Vec<_>>()
    };
    assert_eq!(changes(), []);
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Char('c')));
    type_text(&mut r.view, "Hello");
    r.view.on_key(ctrl_enter());
    wait_written(&mut r.view);
    assert_eq!(changes(), [(C::Posted, 142)]);
    // Edit our own comment.
    let _ = draw(&mut r.view, WIDE);
    r.view.conv.select(2);
    r.view.on_key(key(KeyCode::Char('e')));
    type_text(&mut r.view, " again");
    r.view.on_key(ctrl_enter());
    wait_written(&mut r.view);
    assert_eq!(changes(), [(C::Posted, 142), (C::Edited, 142)]);
    // Delete it.
    let _ = draw(&mut r.view, WIDE);
    r.view.conv.select(2);
    r.view.on_key(key(KeyCode::Char('d')));
    r.view.on_key(key(KeyCode::Tab));
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Enter));
    wait_written(&mut r.view);
    assert_eq!(
        changes(),
        [(C::Posted, 142), (C::Edited, 142), (C::Deleted, 142)]
    );
    // Every one carries this process's origin, so the echo is dropped.
    assert!(crate::hosting_notify::testing::sent(project)
        .iter()
        .all(crate::hosting_notify::is_own));
}

#[test]
fn a_refused_write_tells_nobody() {
    let project = "notify/refused";
    let mut r = rig_in(project);
    r.shared.lock().unwrap().fail_write = Some("403 Forbidden".into());
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Char('c')));
    type_text(&mut r.view, "x");
    r.view.on_key(ctrl_enter());
    wait_written(&mut r.view);
    assert_eq!(crate::hosting_notify::testing::sent(project), []);
}

/// Narrow window: the header buttons clip their labels, none is dropped.
#[test]
fn the_header_buttons_keep_all_three_at_the_narrowest_width() {
    let mut r = rig();
    for width in [120u16, 60, 44, 42] {
        let area = Rect::new(0, 0, width, 30);
        let _ = draw(&mut r.view, area);
        for bar in [Bar::Back, Bar::Refresh, Bar::Browser] {
            assert!(
                r.view
                    .hits
                    .iter()
                    .any(|(rect, h)| *h == Hit::Bar(bar) && rect.width >= 5),
                "{bar:?} vanished at {width} columns: {:?}",
                r.view.hits
            );
        }
    }
}
