//! The conversation of an issue or a pull / merge request as a scrollable,
//! selectable timeline: the description, comments (Markdown through
//! `md_render`), reviews, code threads with their diff excerpt and system
//! events, with unread markers, "Load older", an offline banner and the
//! buttons of the selected entry.
//!
//! The widget owns the data and the view (selection, scrolling, hit rects of
//! its last paint); the surface around it (`item_view`) owns the frame, the
//! header, the composer and what a click on a link or a button means. The
//! reads run on a worker (`worker`), and so does the layout of the rows
//! (`rows::build`, pure): after a width change the old rows are drawn clipped
//! until the new ones arrive, so nothing here blocks the render thread.

// Built only with a hosting provider and the `conversation` feature.
#![allow(dead_code)]

mod cache;
mod input;
mod paint;
pub(crate) mod rows;
#[cfg(test)]
pub(crate) mod tests;
mod unread;
pub(crate) mod worker;

use std::sync::{mpsc, Arc};
use std::time::Instant;

use gitpoulpi_hosting::conversation::{
    Comment, ConversationCaps, ItemHeader, PageCursor, PostingIdentity, TimelineEntry, TimelinePage,
};
use ratatui::layout::Rect;

pub(crate) use input::Act;
use rows::{Input, Layout, LayoutKey, Row, What};
use worker::{Failure, Msg};

use crate::md_render::{self, LinkTarget};

/// Unix seconds now (what a visit is stamped with).
pub(crate) fn now_secs() -> i64 {
    cache::now()
}

/// Writing from the view is the next slice's job: until it lands, every
/// control that would post, edit or delete is drawn greyed with a reason.
pub(crate) const WRITING: bool = false;

/// A spinner shows once a read takes this long (maintainer rule 9).
pub(crate) const SPIN_AFTER: std::time::Duration = std::time::Duration::from_millis(150);

/// The buttons of the selected entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Btn {
    Reply,
    Edit,
    Delete,
    Links,
}

/// A click target of the timeline's last paint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Hit {
    /// A row of item `n`.
    Item(usize),
    /// A button of item `n`.
    Btn(Btn, usize),
    /// A link of the last paint's list (`Conversation::link`).
    Link(usize),
    LoadOlder,
}

/// Where the first read stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Load {
    Loading,
    Ready,
    /// Nothing to show: the read failed and there is no saved copy.
    Failed(String),
}

/// The saved copy is what is shown, because the host could not be reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Offline {
    /// Unix seconds when the copy was saved; 0 when unknown.
    pub(crate) saved_at: i64,
    pub(crate) why: String,
}

pub(crate) struct Conversation {
    pub(crate) source: worker::Source,
    cache_dir: Option<std::path::PathBuf>,
    factory: Arc<dyn crate::hosting_task::ClientFactory>,
    started: Instant,
    /// When the running read began, and whether it is a quiet one (the
    /// timer's: no spinner).
    busy_since: Instant,
    quiet: bool,
    pub(crate) load: Load,
    pub(crate) header: Option<ItemHeader>,
    pub(crate) entries: Vec<TimelineEntry>,
    pub(crate) older: Option<PageCursor>,
    pub(crate) total_hint: Option<u32>,
    pub(crate) caps: Option<ConversationCaps>,
    pub(crate) identity: Option<Result<PostingIdentity, String>>,
    pub(crate) offline: Option<Offline>,
    /// A line above the timeline: the host asked to slow down, or a refresh
    /// failed.
    pub(crate) notice: Option<String>,
    pub(crate) events: bool,
    pub(crate) ctx: md_render::Context,
    seen_at: Option<String>,
    rev: u64,
    // rows
    layout: Arc<Layout>,
    layout_rx: Option<(LayoutKey, mpsc::Receiver<Layout>)>,
    sync_layout: bool,
    // reads
    rx: Option<mpsc::Receiver<Msg>>,
    loading_older: bool,
    last_fresh: Instant,
    older_loaded: bool,
    // view
    pub(crate) top: usize,
    pub(crate) sel: usize,
    sel_key: String,
    /// The selection the last paint scrolled into view.
    followed: Option<String>,
    follow: bool,
    pub(crate) view_rows: usize,
    pub(crate) bar: Option<crate::scrollbar::Track>,
    pub(crate) drag: crate::scrollbar::BarDrag,
    pub(crate) hits: Vec<(Rect, Hit)>,
    pub(crate) links: Vec<LinkTarget>,
    pub(crate) hover: Option<Hit>,
    /// The area the last paint used.
    pub(crate) area: Rect,
}

impl Conversation {
    pub(crate) fn new(
        factory: Arc<dyn crate::hosting_task::ClientFactory>,
        source: worker::Source,
        cache_dir: Option<std::path::PathBuf>,
        ctx: md_render::Context,
        seen_at: Option<String>,
        events: bool,
    ) -> Self {
        let mut me = Self {
            source,
            cache_dir,
            factory,
            started: Instant::now(),
            busy_since: Instant::now(),
            quiet: false,
            load: Load::Loading,
            header: None,
            entries: Vec::new(),
            older: None,
            total_hint: None,
            caps: None,
            identity: None,
            offline: None,
            notice: None,
            events,
            ctx,
            seen_at,
            rev: 0,
            layout: Arc::new(Layout::default()),
            layout_rx: None,
            sync_layout: cfg!(test),
            rx: None,
            loading_older: false,
            last_fresh: Instant::now(),
            older_loaded: false,
            top: 0,
            sel: 0,
            sel_key: "body".to_string(),
            followed: None,
            follow: true,
            view_rows: 0,
            bar: None,
            drag: Default::default(),
            hits: Vec::new(),
            links: Vec::new(),
            hover: None,
            area: Rect::default(),
        };
        me.start(worker::Mode::Open);
        me
    }

    // -- reads ---------------------------------------------------------------

    fn start(&mut self, mode: worker::Mode) {
        self.busy_since = Instant::now();
        self.quiet = false;
        self.rx = Some(worker::start(
            Arc::clone(&self.factory),
            self.source.clone(),
            self.cache_dir.clone(),
            mode,
        ));
    }

    /// Reads the newest window again (F5, the timer, after a post).
    pub(crate) fn refresh(&mut self) {
        if self.rx.is_some() {
            return;
        }
        self.notice = None;
        self.start(worker::Mode::Refresh);
    }

    /// [`Conversation::refresh`] for the timer: nothing shows while it runs.
    pub(crate) fn refresh_quietly(&mut self) {
        if self.rx.is_some() {
            return;
        }
        self.start(worker::Mode::Refresh);
        self.quiet = true;
    }

    /// Reads the window before the one held.
    pub(crate) fn load_older(&mut self) {
        if self.rx.is_some() {
            return;
        }
        if let Some(cursor) = self.older.clone() {
            self.loading_older = true;
            self.start(worker::Mode::Older(cursor));
        }
    }

    /// The running read is the timer's.
    pub(crate) fn is_quiet(&self) -> bool {
        self.quiet && self.layout_rx.is_none()
    }

    /// A read is running.
    pub(crate) fn busy(&self) -> bool {
        self.rx.is_some() || self.layout_rx.is_some()
    }

    /// The spinner moves: a read has run past [`SPIN_AFTER`].
    pub(crate) fn spinning(&self) -> bool {
        self.rx.is_some() && !self.quiet && self.busy_since.elapsed() >= SPIN_AFTER
    }

    pub(crate) fn spinner(&self) -> char {
        crate::activity::spinner_frame(self.started.elapsed())
    }

    /// Whether the first read has not answered.
    pub(crate) fn loading(&self) -> bool {
        self.load == Load::Loading
    }

    /// Drains the worker: `true` when something changed on screen.
    pub(crate) fn poll(&mut self) -> bool {
        let mut changed = false;
        while let Some(rx) = &self.rx {
            match rx.try_recv() {
                Ok(msg) => {
                    changed = true;
                    self.apply(msg);
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.rx = None;
                    self.loading_older = false;
                    if self.load == Load::Loading {
                        self.load = Load::Failed(
                            gitpoulpi_i18n::msg::item_view::LOAD_STOPPED
                                .get()
                                .to_string(),
                        );
                    }
                    changed = true;
                    break;
                }
            }
        }
        if let Some((_, rx)) = &self.layout_rx {
            match rx.try_recv() {
                Ok(layout) => {
                    self.layout_rx = None;
                    self.take_layout(layout);
                    changed = true;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.layout_rx = None;
                    changed = true;
                }
            }
        }
        changed
    }

    fn apply(&mut self, msg: Msg) {
        match msg {
            Msg::Cached(page, saved_at) => {
                if self.header.is_none() {
                    self.adopt(*page, true);
                    self.load = Load::Ready;
                    self.offline = Some(Offline {
                        saved_at,
                        why: String::new(),
                    });
                }
            }
            Msg::Caps(caps) => {
                self.caps = Some(caps);
                self.rev += 1;
            }
            Msg::Fresh(page) => {
                self.adopt(*page, false);
                self.load = Load::Ready;
                self.offline = None;
                self.notice = None;
                self.last_fresh = Instant::now();
            }
            Msg::Older(page) => {
                self.loading_older = false;
                self.prepend(*page);
            }
            Msg::Identity(identity) => {
                self.identity = Some(identity);
                self.rev += 1;
            }
            Msg::Failed(failure) => self.failed(failure),
            Msg::Done => {
                self.rx = None;
                self.loading_older = false;
            }
        }
    }

    fn failed(&mut self, failure: Failure) {
        self.loading_older = false;
        let have = self.header.is_some();
        match failure {
            Failure::Offline(why) if have => {
                let saved_at = self.offline.as_ref().map_or(0, |o| o.saved_at);
                self.offline = Some(Offline { saved_at, why });
            }
            Failure::RateLimited(why) if have => self.notice = Some(why),
            Failure::Other(why) | Failure::Offline(why) | Failure::RateLimited(why) if have => {
                self.notice = Some(why);
            }
            Failure::Unsupported(why)
            | Failure::Other(why)
            | Failure::Offline(why)
            | Failure::RateLimited(why) => self.load = Load::Failed(why),
        }
    }

    /// Takes a page the host (or the saved copy) sent as the newest window.
    fn adopt(&mut self, page: TimelinePage, cached: bool) {
        let TimelinePage {
            header,
            entries,
            older,
            total_hint,
        } = page;
        let keep: Vec<TimelineEntry> = if self.older_loaded {
            let first = entries.first().map(|e| e.at().to_string());
            self.entries
                .iter()
                .filter(|e| first.as_deref().is_some_and(|f| e.at() < f))
                .cloned()
                .collect()
        } else {
            Vec::new()
        };
        if !self.older_loaded {
            self.older = older;
        }
        self.header = Some(header);
        self.entries = keep.into_iter().chain(entries).collect();
        self.total_hint = total_hint;
        let _ = cached;
        self.rev += 1;
    }

    fn prepend(&mut self, page: TimelinePage) {
        self.older = page.older;
        self.older_loaded = true;
        let mut older = page.entries;
        older.append(&mut self.entries);
        self.entries = older;
        self.rev += 1;
        // The window grew above the view: keep what was on screen where it was.
        self.follow = false;
        self.followed = Some(self.sel_key.clone());
        self.top = 0;
    }

    pub(crate) fn set_context(&mut self, ctx: md_render::Context) {
        if self.ctx != ctx {
            self.ctx = ctx;
            self.rev += 1;
        }
    }

    /// Tests: who posts, as the worker would have said it.
    #[cfg(test)]
    pub(crate) fn set_identity(&mut self, identity: Result<PostingIdentity, String>) {
        self.identity = Some(identity);
        self.rev += 1;
    }

    pub(crate) fn set_events(&mut self, on: bool) {
        if self.events != on {
            self.events = on;
            self.rev += 1;
        }
    }

    // -- rows ----------------------------------------------------------------

    fn input(&self) -> Input {
        Input {
            header: self.header.clone(),
            entries: self.entries.clone(),
            ctx: self.ctx.clone(),
            events: self.events,
            seen_at: self.seen_at.clone(),
            me: self.me(),
        }
    }

    /// The login posts are made as, when known.
    pub(crate) fn me(&self) -> Option<String> {
        match &self.identity {
            Some(Ok(id)) => id.login.clone(),
            _ => None,
        }
    }

    /// Makes sure the rows for `content_width` are being laid out.
    pub(crate) fn ensure_layout(&mut self, content_width: usize) {
        if self.header.is_none() {
            return;
        }
        let want = LayoutKey {
            width: content_width.max(crate::md_render::MIN_WIDTH),
            theme: crate::theme::theme_generation(),
            events: self.events,
            rev: self.rev,
        };
        if self.layout.key.as_ref() == Some(&want) {
            return;
        }
        if self.sync_layout {
            let layout = rows::build(&self.input(), want.width, want);
            self.take_layout(layout);
            return;
        }
        if self.layout_rx.is_some() {
            return;
        }
        let input = self.input();
        let (tx, rx) = mpsc::channel();
        let key = want.clone();
        let width = want.width;
        std::thread::spawn(move || {
            use crate::redraw::NotifySend as _;
            let layout = rows::build(&input, width, want);
            let _ = tx.send_notify(layout);
        });
        self.layout_rx = Some((key, rx));
    }

    fn take_layout(&mut self, layout: Layout) {
        let was = self.layout.items.get(self.sel).map(|i| i.key.clone());
        self.layout = Arc::new(layout);
        let key = was.unwrap_or_else(|| self.sel_key.clone());
        self.sel = self
            .layout
            .items
            .iter()
            .position(|i| i.key == key)
            .unwrap_or_else(|| self.sel.min(self.layout.items.len().saturating_sub(1)));
        self.sel_key = self
            .layout
            .items
            .get(self.sel)
            .map_or(key, |i| i.key.clone());
        self.top = self.top.min(self.layout.rows.len().saturating_sub(1));
    }

    /// The rows last laid out (drawn as they are, even for another width).
    pub(crate) fn layout(&self) -> &Layout {
        &self.layout
    }

    pub(crate) fn items(&self) -> usize {
        self.layout.items.len()
    }

    // -- selection -----------------------------------------------------------

    pub(crate) fn select(&mut self, item: usize) {
        let last = self.layout.items.len().saturating_sub(1);
        self.sel = item.min(last);
        if let Some(i) = self.layout.items.get(self.sel) {
            self.sel_key = i.key.clone();
        }
        self.follow = true;
    }

    pub(crate) fn selected(&self) -> Option<&rows::Item> {
        self.layout.items.get(self.sel)
    }

    /// The selected item's entry (`None` on the description).
    pub(crate) fn selected_entry(&self) -> Option<&TimelineEntry> {
        match self.selected()?.what {
            What::Body => None,
            What::Entry(i) => self.entries.get(i),
        }
    }

    /// The selected comment (a plain one, or the first of a thread).
    pub(crate) fn selected_comment(&self) -> Option<&Comment> {
        match self.selected_entry()? {
            TimelineEntry::Comment(c) => Some(c),
            TimelineEntry::Thread(t) => t.comments.first(),
            _ => None,
        }
    }

    pub(crate) fn step(&mut self, delta: isize) {
        let n = self.layout.items.len();
        if n == 0 {
            return;
        }
        let to = (self.sel as isize + delta).clamp(0, n as isize - 1) as usize;
        self.select(to);
    }

    /// Selects the next unread item after the selection (wrapping once).
    pub(crate) fn next_unread(&mut self) -> bool {
        match unread::next(&self.layout.items, self.sel) {
            Some(i) => {
                self.select(i);
                true
            }
            None => false,
        }
    }

    pub(crate) fn has_unread(&self) -> bool {
        self.layout.items.iter().any(|i| i.unread)
    }

    pub(crate) fn unread_count(&self) -> usize {
        self.layout.items.iter().filter(|i| i.unread).count()
    }

    /// The newest entry shown now: what a visit leaves as "seen".
    pub(crate) fn newest(&self) -> Option<(String, String)> {
        unread::newest(&self.entries)
    }

    // -- scrolling -----------------------------------------------------------

    pub(crate) fn scroll(&mut self, delta: isize) {
        let max = self.layout.rows.len().saturating_sub(self.view_rows.max(1));
        self.top = (self.top as isize + delta).clamp(0, max as isize) as usize;
        self.follow = false;
        self.followed = Some(self.sel_key.clone());
    }

    pub(crate) fn scroll_to(&mut self, top: usize) {
        let max = self.layout.rows.len().saturating_sub(self.view_rows.max(1));
        self.top = top.min(max);
        self.follow = false;
        self.followed = Some(self.sel_key.clone());
    }

    /// A page: scrolls and selects the item at the top of the view.
    pub(crate) fn page(&mut self, down: bool) {
        let page = self.view_rows.saturating_sub(1).max(1) as isize;
        self.scroll(if down { page } else { -page });
        let top = self.top;
        if let Some(i) = self.layout.items.iter().position(|i| i.start + i.len > top) {
            self.sel = i;
            self.sel_key = self.layout.items[i].key.clone();
            self.followed = Some(self.sel_key.clone());
        }
    }

    /// Scrolls the selected item into view when the selection moved.
    fn follow_selection(&mut self) {
        let Some(item) = self.layout.items.get(self.sel) else {
            return;
        };
        let rows = self.view_rows.max(1);
        if self.follow || self.followed.as_deref() != Some(item.key.as_str()) {
            let (start, len) = (item.start, item.len);
            if len <= rows {
                if start + len > self.top + rows {
                    self.top = start + len - rows;
                }
                if start < self.top {
                    self.top = start;
                }
            } else {
                self.top = start;
            }
            self.followed = Some(item.key.clone());
            self.follow = false;
        }
        let max = self.layout.rows.len().saturating_sub(rows);
        self.top = self.top.min(max);
    }

    // -- what the buttons may do ---------------------------------------------

    /// Whether the connection is down: posting waits for it.
    pub(crate) fn is_offline(&self) -> bool {
        self.offline.as_ref().is_some_and(|o| !o.why.is_empty())
    }

    pub(crate) fn can_comment(&self) -> bool {
        self.caps.as_ref().is_some_and(|c| c.comment)
    }

    /// Whether the selected entry is the user's own comment, the host
    /// allowing edits. `None` means the host did not say and nothing is
    /// known about the login: the control stays hidden.
    pub(crate) fn mine(&self, comment: &Comment) -> bool {
        if let Some(can) = comment.can_edit {
            return can;
        }
        match &self.identity {
            Some(Ok(id)) => id.authored(&comment.author).unwrap_or(false),
            _ => false,
        }
    }

    pub(crate) fn can_edit_selected(&self) -> bool {
        self.caps.as_ref().is_some_and(|c| c.edit_own)
            && self.selected_comment().is_some_and(|c| self.mine(c))
    }

    pub(crate) fn can_delete_selected(&self) -> bool {
        self.caps.as_ref().is_some_and(|c| c.delete_own)
            && self
                .selected_comment()
                .is_some_and(|c| c.can_delete.unwrap_or_else(|| self.mine(c)))
    }

    /// Whether the selected item can be answered, and the reason when not.
    pub(crate) fn reply_reason(&self) -> Option<&'static str> {
        use gitpoulpi_i18n::msg::item_view as m;
        let Some(caps) = &self.caps else {
            return Some(m::WHY_NOT_LOADED.get());
        };
        let thread = matches!(self.selected_entry(), Some(TimelineEntry::Thread(_)));
        let allowed = if thread {
            caps.reply_in_thread
        } else {
            caps.comment
        };
        if !allowed {
            return Some(m::WHY_NO_REPLY.get());
        }
        if self.is_offline() {
            return Some(m::WHY_OFFLINE.get());
        }
        None
    }

    /// The quoted text of the selected item for a quote reply.
    pub(crate) fn quote(&self) -> Option<String> {
        let (author, body) = match self.selected()?.what {
            What::Body => {
                let h = self.header.as_ref()?;
                (h.author.display.clone(), rows::markdown_of(&h.body))
            }
            What::Entry(i) => match self.entries.get(i)? {
                TimelineEntry::Comment(c) => (c.author.display.clone(), rows::markdown_of(&c.body)),
                TimelineEntry::Review(r) => (r.author.display.clone(), rows::markdown_of(&r.body)),
                TimelineEntry::Thread(t) => {
                    let c = t.comments.last()?;
                    (c.author.display.clone(), rows::markdown_of(&c.body))
                }
                TimelineEntry::Event(_) => return None,
            },
        };
        let mut out = format!("> **{}**\n", md_render::sanitize(&author));
        for line in md_render::sanitize(&body).lines() {
            out.push_str("> ");
            out.push_str(line);
            out.push('\n');
        }
        out.push('\n');
        Some(out)
    }

    /// The address of the selected item on the host.
    pub(crate) fn selected_url(&self) -> Option<String> {
        let own = match self.selected_entry() {
            Some(TimelineEntry::Comment(c)) => c.web_url.clone(),
            Some(TimelineEntry::Thread(t)) => t.comments.first().and_then(|c| c.web_url.clone()),
            _ => None,
        };
        own.filter(|u| !u.is_empty())
            .or_else(|| self.header.as_ref().map(|h| h.web_url.clone()))
            .filter(|u| !u.is_empty())
    }

    /// How many entries a "Load older" would bring, when known.
    pub(crate) fn older_count(&self) -> Option<u32> {
        self.total_hint
            .map(|t| t.saturating_sub(self.entries.len() as u32))
            .filter(|n| *n > 0)
    }

    /// The links of the selected item.
    pub(crate) fn selected_links(&self) -> Vec<LinkTarget> {
        self.selected().map(|i| i.links.clone()).unwrap_or_default()
    }

    /// The link a click landed on.
    pub(crate) fn link(&self, id: usize) -> Option<&LinkTarget> {
        self.links.get(id)
    }

    pub(crate) fn hit_at(&self, col: u16, row: u16) -> Option<Hit> {
        self.hits
            .iter()
            .find(|(r, _)| r.contains((col, row).into()))
            .map(|(_, h)| *h)
    }

    /// True while the layout shown is not the one wanted (a width change).
    pub(crate) fn relaying(&self) -> bool {
        self.layout_rx.is_some()
    }

    /// The rows of the item at `row`, for tests and hit routing.
    pub(crate) fn item_at_row(&self, row: usize) -> Option<usize> {
        match self.layout.rows.get(row)? {
            Row::Blank => None,
            r => r.item(),
        }
    }

    /// When the last fresh read landed (the 60 s refresh counts from it).
    pub(crate) fn since_fresh(&self) -> std::time::Duration {
        self.last_fresh.elapsed()
    }

    pub(crate) fn age_text(&self) -> Option<String> {
        let o = self.offline.as_ref()?;
        (o.saved_at > 0).then(|| age(cache::now() - o.saved_at))
    }
}

/// `3 min`, `2 h`, `4 days`: how old a saved copy is.
pub(crate) fn age(secs: i64) -> String {
    use gitpoulpi_i18n::msg::item_view as m;
    let secs = secs.max(0);
    if secs < 90 {
        m::AGE_NOW.get().to_string()
    } else if secs < 90 * 60 {
        m::age_minutes((secs / 60) as u64)
    } else if secs < 36 * 3600 {
        m::age_hours((secs / 3600) as u64)
    } else {
        m::age_days((secs / 86_400) as u64)
    }
}
