//! The timeline over a fake provider: the page comes from memory, the rows
//! are laid out for real, and the paint is read back from a `Buffer`.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_hosting::conversation::{
    Actor, ActorKind, CodeThread, Comment, ConversationCaps, Event, EventKind, IdentityKind,
    ItemHeader, ItemKey, ItemKind, Markup, MarkupFlavor, PageCursor, PostingIdentity, ReviewEntry,
    TimelineEntry, TimelineOpts, TimelinePage, TokenSource,
};
use gitpoulpi_hosting::review::model::{DiffSide, ReviewState};
use gitpoulpi_hosting::{HostedProject, HostedSshKey, HostingClient, Integration, IntegrationKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;

use super::*;
use crate::hosting_task::ClientFactory;

pub(crate) const AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 100,
    height: 30,
};

/// What the fake provider answers and what it was asked.
pub(crate) struct Shared {
    pub(crate) page: Result<TimelinePage, String>,
    pub(crate) older: Option<TimelinePage>,
    pub(crate) caps: ConversationCaps,
    pub(crate) identity: Result<PostingIdentity, String>,
    /// Every `(project, key)` a timeline was read for.
    pub(crate) reads: Vec<(String, ItemKey)>,
    /// Every request the host saw that is not a read.
    pub(crate) writes: usize,
    /// What the host was asked to write, as `kind|id|body`.
    pub(crate) write_log: Vec<String>,
    /// The next write is refused with this message.
    pub(crate) fail_write: Option<String>,
    /// A write waits this long before the host answers.
    pub(crate) write_delay: Duration,
    /// The next timeline read waits this long.
    pub(crate) delay: Duration,
    /// The next read fails like an unreachable host.
    pub(crate) offline: bool,
}

pub(crate) struct Fake {
    pub(crate) integration: Integration,
    pub(crate) shared: Arc<Mutex<Shared>>,
}

#[async_trait::async_trait]
impl HostingClient for Fake {
    fn integration(&self) -> &Integration {
        &self.integration
    }
    async fn list_projects(&self) -> gitpoulpi_hosting::Result<Vec<HostedProject>> {
        Ok(Vec::new())
    }
    async fn list_ssh_keys(&self) -> gitpoulpi_hosting::Result<Vec<HostedSshKey>> {
        Ok(Vec::new())
    }
    async fn upload_ssh_key(&self, _: &str, _: &str) -> gitpoulpi_hosting::Result<HostedSshKey> {
        Err(gitpoulpi_hosting::Error::Unsupported("upload".into()))
    }
    async fn create_pull_request(
        &self,
        _: &gitpoulpi_hosting::NewPullRequest,
    ) -> gitpoulpi_hosting::Result<gitpoulpi_hosting::CreatedPullRequest> {
        Err(gitpoulpi_hosting::Error::Unsupported("create".into()))
    }
    fn conversation_caps(&self, _kind: ItemKind) -> ConversationCaps {
        self.shared.lock().unwrap().caps.clone()
    }
    async fn posting_identity(&self, _: &str) -> gitpoulpi_hosting::Result<PostingIdentity> {
        self.shared
            .lock()
            .unwrap()
            .identity
            .clone()
            .map_err(gitpoulpi_hosting::Error::Local)
    }
    async fn post_item_comment(
        &self,
        _: &str,
        _: &ItemKey,
        body: &str,
        internal: bool,
    ) -> gitpoulpi_hosting::Result<Comment> {
        self.write("post", if internal { "internal" } else { "" }, body)
            .await
    }
    async fn reply_in_discussion(
        &self,
        _: &str,
        _: &ItemKey,
        discussion_id: &str,
        body: &str,
    ) -> gitpoulpi_hosting::Result<Comment> {
        self.write("reply", discussion_id, body).await
    }
    async fn edit_item_comment(
        &self,
        _: &str,
        _: &ItemKey,
        comment_id: &str,
        body: &str,
    ) -> gitpoulpi_hosting::Result<Comment> {
        self.write("edit", comment_id, body).await
    }
    async fn delete_item_comment(
        &self,
        _: &str,
        _: &ItemKey,
        comment_id: &str,
    ) -> gitpoulpi_hosting::Result<()> {
        self.write("delete", comment_id, "").await.map(|_| ())
    }
    async fn item_timeline(
        &self,
        project: &str,
        key: &ItemKey,
        older: Option<&PageCursor>,
        _: &TimelineOpts,
    ) -> gitpoulpi_hosting::Result<TimelinePage> {
        let (delay, offline) = {
            let mut s = self.shared.lock().unwrap();
            s.reads.push((project.to_string(), key.clone()));
            (s.delay, std::mem::take(&mut s.offline))
        };
        tokio::time::sleep(delay).await;
        if offline {
            return Err(gitpoulpi_hosting::Error::Http("connection refused".into()));
        }
        let s = self.shared.lock().unwrap();
        if older.is_some() {
            return s
                .older
                .clone()
                .ok_or_else(|| gitpoulpi_hosting::Error::Local("no older page".into()));
        }
        s.page.clone().map_err(gitpoulpi_hosting::Error::Local)
    }
}

impl Fake {
    /// Records a write and answers like a host that accepted it, authored
    /// by whoever the identity is (comment id 900 + the number of writes).
    async fn write(&self, kind: &str, arg: &str, body: &str) -> gitpoulpi_hosting::Result<Comment> {
        let (delay, fail, login, n) = {
            let mut s = self.shared.lock().unwrap();
            s.writes += 1;
            s.write_log.push(format!("{kind}|{arg}|{body}"));
            let login = match &s.identity {
                Ok(id) => id.login.clone().unwrap_or_else(|| "my-app[bot]".into()),
                Err(_) => "nobody".into(),
            };
            (s.write_delay, s.fail_write.take(), login, s.writes)
        };
        tokio::time::sleep(delay).await;
        if let Some(why) = fail {
            return Err(gitpoulpi_hosting::Error::Local(why));
        }
        let id = if kind == "edit" {
            arg.to_string()
        } else {
            format!("{}", 900 + n)
        };
        let mut c = comment(&id, &login, body, "2026-11-15T10:00:00Z");
        if login.ends_with("[bot]") {
            c.author.kind = ActorKind::Bot;
        }
        Ok(c)
    }
}

pub(crate) struct Factory(pub(crate) Arc<Mutex<Shared>>);

impl ClientFactory for Factory {
    fn client(
        &self,
        integration: &Integration,
        _: &[Integration],
    ) -> gitpoulpi_hosting::Result<Box<dyn HostingClient>> {
        Ok(Box::new(Fake {
            integration: integration.clone(),
            shared: self.0.clone(),
        }))
    }
}

pub(crate) fn integration() -> Integration {
    Integration::new(
        "corp",
        IntegrationKind::Gitlab,
        "https://gitlab.example.com",
    )
}

pub(crate) fn actor(login: &str) -> Actor {
    Actor::new(login, login, ActorKind::User)
}

pub(crate) fn comment(id: &str, author: &str, body: &str, at: &str) -> Comment {
    Comment {
        id: id.into(),
        author: actor(author),
        body: Markup::new(body, MarkupFlavor::GitlabMd),
        created_at: at.into(),
        updated_at: None,
        edited: false,
        reactions: Vec::new(),
        in_reply_to: None,
        discussion_id: None,
        internal: false,
        can_edit: None,
        can_delete: None,
        web_url: Some(format!(
            "https://gitlab.example.com/g/p/-/issues/142#note_{id}"
        )),
    }
}

pub(crate) fn header() -> ItemHeader {
    ItemHeader {
        key: ItemKey::issue(142),
        reference: "#142".into(),
        title: "Crash when opening a repository".into(),
        state_label: "open".into(),
        open: true,
        author: actor("alice"),
        body: Markup::new(
            "When I open a repository with **locked** files it freezes.\n\n- [x] reproduced on Linux",
            MarkupFlavor::GitlabMd,
        ),
        created_at: "2026-11-12T09:14:00Z".into(),
        web_url: "https://gitlab.example.com/g/p/-/issues/142".into(),
        labels: vec!["bug".into(), "lfs".into()],
    }
}

pub(crate) fn page(entries: Vec<TimelineEntry>) -> TimelinePage {
    TimelinePage {
        header: header(),
        entries,
        older: None,
        total_hint: None,
    }
}

pub(crate) fn caps() -> ConversationCaps {
    ConversationCaps {
        read: true,
        comment: true,
        reply_in_thread: true,
        edit_own: true,
        delete_own: true,
        events: true,
        read_flavor: MarkupFlavor::GitlabMd,
        write_flavor: MarkupFlavor::GitlabMd,
        ..ConversationCaps::NONE
    }
}

pub(crate) fn me(login: &str) -> PostingIdentity {
    PostingIdentity::new(
        Some(login.into()),
        IdentityKind::PersonalToken,
        TokenSource::Profile { id: "corp".into() },
    )
}

pub(crate) fn shared(entries: Vec<TimelineEntry>) -> Arc<Mutex<Shared>> {
    Arc::new(Mutex::new(Shared {
        page: Ok(page(entries)),
        older: None,
        caps: caps(),
        identity: Ok(me("bob")),
        reads: Vec::new(),
        writes: 0,
        write_log: Vec::new(),
        fail_write: None,
        write_delay: Duration::ZERO,
        delay: Duration::ZERO,
        offline: false,
    }))
}

pub(crate) fn conversation(
    shared: &Arc<Mutex<Shared>>,
    cache: Option<std::path::PathBuf>,
    seen: Option<&str>,
) -> Conversation {
    Conversation::new(
        Arc::new(Factory(shared.clone())),
        worker::Source {
            integration: integration(),
            all: vec![integration()],
            project: "g/p".into(),
            key: ItemKey::issue(142),
        },
        cache,
        md_render::Context {
            xrefs: gitpoulpi_hosting::xrefs::XrefContext {
                family: Some(gitpoulpi_config::ApiFamily::Gitlab),
                web_base: "https://gitlab.example.com".into(),
                project: "g/p".into(),
                ..Default::default()
            },
            ..Default::default()
        },
        seen.map(str::to_string),
        true,
    )
}

/// Polls until `done` holds.
pub(crate) fn wait_for(conv: &mut Conversation, what: &str, done: impl Fn(&Conversation) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        conv.poll();
        if done(conv) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

pub(crate) fn ready(conv: &mut Conversation) {
    wait_for(conv, "the conversation", |c| {
        c.header.is_some() && c.identity.is_some() && !c.busy()
    });
}

pub(crate) fn paint(conv: &mut Conversation, area: Rect) -> Buffer {
    let mut buf = Buffer::empty(AREA.union(area));
    conv.paint(&mut buf, area, true);
    buf
}

pub(crate) fn text(buf: &Buffer) -> String {
    let mut out = String::new();
    for y in 0..buf.area.height {
        let line: String = (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect::<String>();
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn entries() -> Vec<TimelineEntry> {
    vec![
        TimelineEntry::Event(Event {
            id: "e1".into(),
            actor: Some(actor("carol")),
            at: "2026-11-12T10:01:00Z".into(),
            kind: EventKind::Labeled {
                added: vec!["lfs".into()],
                removed: vec![],
            },
        }),
        TimelineEntry::Comment(comment(
            "c1",
            "carol",
            "Same as #120, see `src/lfs.rs`.\n\n```rust\nfn lock() -> Result<()> { Ok(()) }\n```",
            "2026-11-12T10:30:00Z",
        )),
        TimelineEntry::Comment(comment(
            "c2",
            "bob",
            "Thanks, fixed in 1.2.",
            "2026-11-13T08:00:00Z",
        )),
    ]
}

#[test]
fn the_timeline_shows_the_description_comments_and_events() {
    let shared = shared(entries());
    let mut conv = conversation(&shared, None, None);
    ready(&mut conv);
    let buf = paint(&mut conv, AREA);
    let screen = text(&buf);
    assert!(screen.contains("alice"), "{screen}");
    assert!(screen.contains("opened this"), "{screen}");
    assert!(screen.contains("locked"), "bold text is rendered: {screen}");
    assert!(!screen.contains("**"), "no raw Markdown markers: {screen}");
    assert!(screen.contains("carol added the label lfs"), "{screen}");
    assert!(screen.contains("fn lock()"), "code block: {screen}");
    assert!(screen.contains("Thanks, fixed in 1.2."), "{screen}");
}

#[test]
fn events_can_be_hidden() {
    let shared = shared(entries());
    let mut conv = conversation(&shared, None, None);
    ready(&mut conv);
    conv.set_events(false);
    let screen = text(&paint(&mut conv, AREA));
    assert!(!screen.contains("added the label"), "{screen}");
    assert!(screen.contains("carol"), "{screen}");
}

#[test]
fn rows_partition_into_items_whatever_the_width() {
    let shared = shared(entries());
    let mut conv = conversation(&shared, None, None);
    ready(&mut conv);
    for width in [12usize, 20, 40, 80, 120] {
        let input = conv.input();
        let layout = rows::build(
            &input,
            width,
            LayoutKey {
                width,
                theme: 0,
                events: true,
                rev: 0,
            },
        );
        let mut next = 0;
        for item in &layout.items {
            assert_eq!(item.start, next, "items are contiguous at {width}");
            next = item.start + item.len;
        }
        assert_eq!(
            next,
            layout.rows.len(),
            "every row belongs to an item at {width}"
        );
        assert_eq!(layout.items.len(), 1 + entries().len());
    }
}

#[test]
fn cjk_text_is_never_cut_through_a_glyph() {
    let shared = shared(vec![TimelineEntry::Comment(comment(
        "c1",
        "kai",
        "\u{4F60}\u{597D}\u{4E16}\u{754C}\u{4F60}\u{597D}\u{4E16}\u{754C}\u{4F60}\u{597D}\u{4E16}\u{754C}",
        "2026-11-12T10:30:00Z",
    ))]);
    let mut conv = conversation(&shared, None, None);
    ready(&mut conv);
    let area = Rect::new(0, 0, 21, 24);
    let _ = paint(&mut conv, area);
    for row in &conv.layout().rows {
        if let rows::Row::Body(_, doc, n, indent) = row {
            let width = doc.lines[*n].width() + usize::from(*indent);
            assert!(width <= 19, "a body line of {width} cells overflows 19");
        }
    }
}

#[test]
fn arrows_and_ctrl_j_k_move_the_selection_and_bare_letters_ignore_ctrl() {
    let shared = shared(entries());
    let mut conv = conversation(&shared, None, None);
    ready(&mut conv);
    let _ = paint(&mut conv, AREA);
    assert_eq!(conv.sel, 0);
    conv.on_key(key(KeyCode::Down));
    assert_eq!(conv.sel, 1);
    conv.on_key(ctrl('j'));
    assert_eq!(conv.sel, 2);
    conv.on_key(ctrl('k'));
    assert_eq!(conv.sel, 1);
    conv.on_key(key(KeyCode::End));
    assert_eq!(conv.sel, conv.items() - 1);
    conv.on_key(key(KeyCode::Home));
    assert_eq!(conv.sel, 0);
    // `r` replies; Ctrl+R is not the timeline's.
    assert_eq!(conv.on_key(key(KeyCode::Char('r'))), Some(Act::Reply));
    assert_eq!(conv.on_key(ctrl('r')), None);
    assert_eq!(conv.on_key(ctrl('n')), None);
    assert_eq!(conv.on_key(key(KeyCode::Esc)), Some(Act::Close));
    assert_eq!(conv.on_key(key(KeyCode::F(5))), Some(Act::Refresh));
}

#[test]
fn what_arrived_since_the_last_visit_is_unread_but_not_what_i_wrote() {
    let shared = shared(entries());
    let mut conv = conversation(&shared, None, Some("2026-11-12T10:00:00Z"));
    ready(&mut conv);
    let screen = text(&paint(&mut conv, AREA));
    // The label event and carol's comment are new; bob (me) wrote the last.
    assert_eq!(conv.unread_count(), 2, "{screen}");
    assert!(
        screen.contains("\u{258C}"),
        "the unread marker is drawn: {screen}"
    );
    assert!(screen.contains("new"), "{screen}");
    assert!(conv.next_unread());
    assert!(conv.selected().unwrap().unread);
    // Nothing is unread without a previous visit.
    let mut first = conversation(&shared, None, None);
    ready(&mut first);
    assert_eq!(first.unread_count(), 0);
}

#[test]
fn a_link_hit_rect_is_exactly_the_underlined_cells() {
    let shared = shared(entries());
    let mut conv = conversation(&shared, None, None);
    ready(&mut conv);
    let buf = paint(&mut conv, AREA);
    let links: Vec<Rect> = conv
        .hits
        .iter()
        .filter(|(_, h)| matches!(h, Hit::Link(_)))
        .map(|(r, _)| *r)
        .collect();
    assert!(!links.is_empty(), "the #120 reference is a link");
    for rect in &links {
        for x in rect.x..rect.right() {
            assert!(
                buf[(x, rect.y)].modifier.contains(Modifier::UNDERLINED),
                "cell {x} of a link is underlined"
            );
        }
    }
    // And no underlined cell is outside a link rect.
    let underlined: usize = (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .filter(|&x| buf[(x, y)].modifier.contains(Modifier::UNDERLINED))
                .count()
        })
        .sum();
    let covered: usize = links.iter().map(|r| usize::from(r.width)).sum();
    assert_eq!(underlined, covered, "underlined cells equal link cells");
}

#[test]
fn clicking_a_link_answers_its_target() {
    let shared = shared(entries());
    let mut conv = conversation(&shared, None, None);
    ready(&mut conv);
    let _ = paint(&mut conv, AREA);
    let (rect, _) = *conv
        .hits
        .iter()
        .find(|(_, h)| matches!(h, Hit::Link(_)))
        .unwrap();
    let act = conv.on_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: rect.x,
        row: rect.y,
        modifiers: KeyModifiers::NONE,
    });
    match act {
        Some(Act::Link(t)) => assert!(t.url.contains("120"), "{t:?}"),
        other => panic!("expected a link, got {other:?}"),
    }
}

#[test]
fn a_thread_shows_its_hunk_and_its_comments() {
    let thread = TimelineEntry::Thread(CodeThread {
        id: "t1".into(),
        path: "src/parse.rs".into(),
        line: Some(42),
        original_line: None,
        side: DiffSide::New,
        start_line: None,
        resolved: false,
        outdated: false,
        resolvable: true,
        diff_hunk: Some("@@ -40,3 +40,4 @@\n let a = 1;\n+let n = tokens.len();".into()),
        comments: vec![
            comment("n1", "kai", "Why not iterate?", "2026-11-12T11:00:00Z"),
            comment(
                "n2",
                "bob",
                "Because I need the count.",
                "2026-11-12T11:05:00Z",
            ),
        ],
    });
    let review = TimelineEntry::Review(ReviewEntry {
        id: "r1".into(),
        author: actor("kai"),
        state: ReviewState::ChangesRequested,
        body: Markup::new("Please look at the parser.", MarkupFlavor::GitlabMd),
        submitted_at: "2026-11-12T12:00:00Z".into(),
    });
    let shared = shared(vec![thread, review]);
    let mut conv = conversation(&shared, None, None);
    ready(&mut conv);
    let screen = text(&paint(&mut conv, AREA));
    assert!(screen.contains("Thread  src/parse.rs:42"), "{screen}");
    assert!(screen.contains("unresolved"), "{screen}");
    assert!(screen.contains("+let n = tokens.len();"), "{screen}");
    assert!(screen.contains("Because I need the count."), "{screen}");
    assert!(screen.contains("requested changes"), "{screen}");
}

#[test]
fn an_unreachable_host_leaves_the_saved_copy_with_an_offline_banner() {
    let dir = tempfile::tempdir().unwrap();
    let shared = shared(entries());
    let mut conv = conversation(&shared, Some(dir.path().to_path_buf()), None);
    ready(&mut conv);
    drop(conv);
    shared.lock().unwrap().offline = true;
    let mut again = conversation(&shared, Some(dir.path().to_path_buf()), None);
    wait_for(&mut again, "the offline banner", |c| c.is_offline());
    let screen = text(&paint(&mut again, AREA));
    assert!(
        screen.contains("Offline: showing the copy saved"),
        "{screen}"
    );
    assert!(
        screen.contains("Thanks, fixed in 1.2."),
        "the saved copy shows: {screen}"
    );
    assert_eq!(
        again.reply_reason(),
        Some(gitpoulpi_i18n::msg::item_view::WHY_OFFLINE.get())
    );
}

#[test]
fn without_a_saved_copy_a_failed_read_says_so_and_f5_retries() {
    let shared = shared(entries());
    shared.lock().unwrap().offline = true;
    let mut conv = conversation(&shared, None, None);
    wait_for(&mut conv, "the failure", |c| {
        matches!(c.load, Load::Failed(_))
    });
    let screen = text(&paint(&mut conv, AREA));
    assert!(
        screen.contains("could not load the conversation"),
        "{screen}"
    );
    assert!(screen.contains("Press F5"), "{screen}");
    conv.rx = None;
    conv.refresh();
    wait_for(&mut conv, "the retry", |c| c.header.is_some());
}

#[test]
fn load_older_prepends_and_keeps_the_selection() {
    let shared = shared(entries());
    {
        let mut s = shared.lock().unwrap();
        s.page.as_mut().unwrap().older = Some(PageCursor("2".into()));
        s.page.as_mut().unwrap().total_hint = Some(5);
        s.older = Some(TimelinePage {
            header: header(),
            entries: vec![TimelineEntry::Comment(comment(
                "c0",
                "dan",
                "The very first report.",
                "2026-11-11T08:00:00Z",
            ))],
            older: None,
            total_hint: None,
        });
    }
    let mut conv = conversation(&shared, None, None);
    ready(&mut conv);
    let screen = text(&paint(&mut conv, AREA));
    assert!(screen.contains("Load older (2 more)"), "{screen}");
    conv.select(2);
    let sel = conv.selected().unwrap().key.clone();
    conv.load_older();
    wait_for(&mut conv, "the older window", |c| {
        c.older.is_none() && !c.busy()
    });
    let _ = paint(&mut conv, AREA);
    assert_eq!(
        conv.selected().unwrap().key,
        sel,
        "the same entry stays selected"
    );
    assert!(conv.entries.len() == 4);
    conv.scroll_to(0);
    let screen = text(&paint(&mut conv, AREA));
    assert!(screen.contains("The very first report."), "{screen}");
    assert!(!screen.contains("Load older"), "{screen}");
}

#[test]
fn a_resize_leaves_no_stale_cells() {
    let shared = shared(entries());
    let mut conv = conversation(&shared, None, None);
    ready(&mut conv);
    let mut buf = Buffer::empty(AREA);
    conv.paint(&mut buf, AREA, true);
    let narrow = Rect::new(0, 0, 40, 30);
    conv.paint(&mut buf, narrow, true);
    // Every cell of the narrow area is what a fresh paint there gives.
    let mut fresh = Buffer::empty(AREA);
    let mut other = conversation(&shared, None, None);
    ready(&mut other);
    other.paint(&mut fresh, narrow, true);
    for y in 0..narrow.height {
        for x in 0..narrow.width {
            assert_eq!(
                buf[(x, y)].symbol(),
                fresh[(x, y)].symbol(),
                "stale cell at {x},{y}"
            );
        }
    }
}

#[test]
fn the_selected_entry_has_buttons_and_edit_only_on_my_own_comments() {
    let shared = shared(entries());
    let mut conv = conversation(&shared, None, None);
    ready(&mut conv);
    // carol's comment: Reply and Links, no Edit.
    conv.select(2);
    let screen = text(&paint(&mut conv, AREA));
    assert!(screen.contains("Reply"), "{screen}");
    assert!(!screen.contains("Edit"), "not mine: {screen}");
    // bob's (mine) comment shows Edit and Delete, greyed until writing lands.
    conv.select(3);
    let screen = text(&paint(&mut conv, AREA));
    assert!(
        screen.contains("Edit") && screen.contains("Delete"),
        "{screen}"
    );
    assert!(conv
        .hits
        .iter()
        .any(|(_, h)| matches!(h, Hit::Btn(Btn::Edit, _))));
}

#[test]
fn quoting_an_entry_prefixes_its_text() {
    let shared = shared(entries());
    let mut conv = conversation(&shared, None, None);
    ready(&mut conv);
    let _ = paint(&mut conv, AREA);
    conv.select(3);
    let q = conv.quote().unwrap();
    assert!(
        q.starts_with("> **bob**\n> Thanks, fixed in 1.2.\n\n"),
        "{q}"
    );
}

#[test]
fn the_cache_round_trips_and_stays_bounded() {
    let dir = tempfile::tempdir().unwrap();
    let key = ItemKey::issue(7);
    let file = cache::file_for(dir.path(), "corp", "g/p", &key);
    assert!(
        file.ends_with("conversations/corp/g_p/issue-7.json"),
        "{file:?}"
    );
    cache::write(&file, &page(Vec::new()));
    let (back, at) = cache::read(&file).unwrap();
    assert_eq!(back.header.title, "Crash when opening a repository");
    assert!(at > 0);
    for n in 0..(cache::KEEP + 5) as u64 {
        let f = cache::file_for(dir.path(), "corp", "g/p", &ItemKey::issue(1000 + n));
        cache::write(&f, &page(Vec::new()));
    }
    let count = std::fs::read_dir(dir.path().join("conversations/corp/g_p"))
        .unwrap()
        .count();
    assert!(count <= cache::KEEP, "{count} files kept");
    assert!(cache::read(&dir.path().join("nothing.json")).is_none());
}
