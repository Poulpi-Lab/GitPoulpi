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
use crate::conversation_ui::tests::{
    comment, integration, shared, text, Factory, Shared,
};
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
            crate::conversation_ui::Hit::Link(i) => view
                .conv
                .link(*i)
                .is_some_and(|l| format!("{:?}", l.xref).contains(needle) || l.url.contains(needle)),
            _ => false,
        })
        .unwrap_or_else(|| panic!("no link to {needle}: {:?}", view.conv.links));
    (hit.0.x, hit.0.y)
}

#[test]
fn the_header_names_the_item_and_who_posts() {
    let mut r = rig();
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(screen.contains("#142  Crash when opening a repository"), "{screen}");
    assert!(screen.contains("\u{25CF} open"), "{screen}");
    assert!(screen.contains("opened by alice on 2026-11-12 09:14"), "{screen}");
    assert!(screen.contains("bug  lfs"), "{screen}");
    assert!(screen.contains("2 comments"), "{screen}");
    assert!(screen.contains("Posting as bob (personal token)"), "{screen}");
    assert!(screen.contains("Open in browser") && screen.contains("Refresh"), "{screen}");
}

#[test]
fn an_app_bot_without_a_known_login_is_named_and_a_command_token_says_so() {
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
        screen.contains("Posting as the GitHub App bot (repository token command)"),
        "{screen}"
    );
    r.view.conv.set_identity(Ok(PostingIdentity::new(
        Some("poulpi-lab[bot]".into()),
        IdentityKind::AppBot,
        TokenSource::RepoTokenCommand,
    )));
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(
        screen.contains("Posting as poulpi-lab[bot] (repository token command)"),
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
    assert!(screen.contains("Read only: this host does not take comments from here."), "{screen}");
    assert!(!screen.contains("New comment"), "{screen}");
    assert_eq!(
        ItemCmd::NewComment.unavailable(Some(r.view.avail())),
        Some(gitpoulpi_i18n::msg::item_view::WHY_NO_COMPOSER.get())
    );
}

#[test]
fn send_is_greyed_with_its_reason_and_nothing_is_posted() {
    let mut r = rig();
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(screen.contains("New comment"), "{screen}");
    assert!(screen.contains("Send"), "{screen}");
    // Type, then try to send by key and by mouse: only a reason comes back.
    r.view.on_key(key(KeyCode::Char('c')));
    for c in "hello".chars() {
        r.view.on_key(key(KeyCode::Char(c)));
    }
    let _ = draw(&mut r.view, WIDE);
    let effect = r
        .view
        .on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL));
    assert_eq!(effect, ItemEffect::None);
    let why = gitpoulpi_i18n::msg::item_view::WHY_WRITING_LATER.get();
    assert_eq!(r.view.message.as_deref(), Some(why));
    r.view.message = None;
    let (rect, _) = *r.view.hits.iter().find(|(_, h)| *h == Hit::Send).unwrap();
    assert_eq!(r.view.on_mouse(click(rect.x, rect.y)), ItemEffect::None);
    assert_eq!(r.view.message.as_deref(), Some(why), "the button answers with the reason");
    let screen = text(&draw(&mut r.view, WIDE));
    assert!(screen.contains(why), "the reason is drawn in the view: {screen}");
    assert_eq!(r.shared.lock().unwrap().writes, 0, "nothing was posted");
}

#[test]
fn the_close_cross_and_esc_close_a_clean_view_and_a_typed_one_asks() {
    let mut r = rig();
    let _ = draw(&mut r.view, WIDE);
    let cross = r.view.guard.close.rect().expect("the frame paints a close cross");
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
    assert!(r.view.guard.is_open(), "a typed reply is not thrown away silently");
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
    assert_eq!(r.view.on_key(KeyEvent::new(KeyCode::Left, KeyModifiers::ALT)), ItemEffect::None);
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
    assert!(crate::link_confirm::is_open(), "the real address is asked about first");
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
    r.view.on_key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::ALT));
    let screen = text(&draw(&mut r.view, narrow));
    assert!(screen.contains("bold") && !screen.contains("**bold**"), "{screen}");
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
    assert_eq!(ItemCmd::Back.unavailable(Some(a)), Some(m::WHY_NO_BACK.get()));
    assert_eq!(ItemCmd::LoadOlder.unavailable(Some(a)), Some(m::WHY_NO_OLDER.get()));
    assert_eq!(ItemCmd::NextUnread.unavailable(Some(a)), Some(m::WHY_NO_UNREAD.get()));
    assert_eq!(ItemCmd::Edit.unavailable(Some(a)), Some(m::WHY_NOT_YOURS.get()));
    assert_eq!(ItemCmd::Reply.unavailable(Some(a)), None);
    assert_eq!(ItemCmd::Refresh.unavailable(Some(a)), None);
    assert_eq!(ItemCmd::Close.unavailable(Some(a)), None);
    assert_eq!(ItemCmd::Close.unavailable(None), Some(m::WHY_NO_VIEW.get()));
    // Bob's own comment: Edit is allowed by the host and the login, and the
    // reason left is that writing is not in this version.
    r.view.conv.select(2);
    let a = r.view.avail();
    assert_eq!(ItemCmd::Edit.unavailable(Some(a)), Some(m::WHY_WRITING_LATER.get()));
    assert_eq!(ItemCmd::Send.unavailable(Some(a)), Some(m::WHY_WRITING_LATER.get()));
}

#[test]
fn reply_quotes_the_entry_into_the_composer() {
    let mut r = rig();
    let _ = draw(&mut r.view, WIDE);
    r.view.on_key(key(KeyCode::Down));
    assert_eq!(r.view.on_key(key(KeyCode::Char('r'))), ItemEffect::None);
    assert_eq!(r.view.focus, Focus::Composer);
    let text_in = r.view.md.as_ref().unwrap().text().to_string();
    assert!(text_in.starts_with("> **carol**\n> Same as #120"), "{text_in}");
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
    assert!(r.shared.lock().unwrap().reads.len() > before, "Enter on Refresh reads again");
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
    let seen = state.conversation_seen.get(&t.seen_key()).expect("a visit is recorded");
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
