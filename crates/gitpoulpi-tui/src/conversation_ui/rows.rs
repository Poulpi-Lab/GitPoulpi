//! The timeline as screen rows for one width.
//!
//! [`build`] is pure: it takes a snapshot of the entries and answers the
//! rows (one screen line each) and where each item starts, so it runs on a
//! worker thread and its answer is drawn as it is. Everything that depends on
//! the language (the verbs of an event, the chips) is composed at paint time
//! from the structured rows, so a language switch never needs a new layout.

use std::sync::Arc;

use gitpoulpi_hosting::conversation::{
    Comment, EventKind, ItemHeader, Markup, MarkupFlavor, TimelineEntry,
};

use crate::md_render::{self, Doc, LinkTarget};

/// Indent of a comment's text under its head line.
pub(crate) const BODY_INDENT: u16 = 2;
/// Indent of the comments inside a code thread.
pub(crate) const THREAD_INDENT: u16 = 4;
/// The hunk lines a thread shows at most (its last ones).
const HUNK_LINES: usize = 8;

/// What an item of the timeline is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum What {
    /// The description of the issue or request.
    Body,
    /// `entries[i]`.
    Entry(usize),
}

/// One selectable item: the description, a comment, a review, a code thread
/// or an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Item {
    /// Stable across refreshes (`body`, a comment id, a thread id).
    pub(crate) key: String,
    pub(crate) what: What,
    pub(crate) start: usize,
    pub(crate) len: usize,
    /// Newer than what the user saw at the last visit.
    pub(crate) unread: bool,
    /// The links of its text, in order of appearance.
    pub(crate) links: Vec<LinkTarget>,
}

/// One screen row. `usize` is the item index.
#[derive(Debug, Clone)]
pub(crate) enum Row {
    /// Who and when (and the chips) of a comment, a review or the description.
    Head(usize),
    /// A thread's headline: path, line, state.
    ThreadHead(usize),
    /// One line of a thread's diff excerpt.
    Hunk(usize, String),
    /// A comment of a thread: its author and time. `usize` after the item is
    /// the index into the thread's comments.
    Sub(usize, usize),
    /// One line of Markdown, `indent` cells in.
    Body(usize, Arc<Doc>, usize, u16),
    /// "No description" under a description that is empty.
    Empty(usize),
    /// The reactions of a comment, already worded (`+1 3  heart 1`).
    Reactions(usize, String),
    /// A system event, one line.
    Event(usize),
    Blank,
}

impl Row {
    pub(crate) fn item(&self) -> Option<usize> {
        match self {
            Row::Head(i)
            | Row::ThreadHead(i)
            | Row::Hunk(i, _)
            | Row::Sub(i, _)
            | Row::Body(i, ..)
            | Row::Empty(i)
            | Row::Reactions(i, _)
            | Row::Event(i) => Some(*i),
            Row::Blank => None,
        }
    }
}

/// What a layout is made from; two equal keys give the same rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LayoutKey {
    pub(crate) width: usize,
    pub(crate) theme: u64,
    pub(crate) events: bool,
    /// Bumped by the owner when the entries, the context or the unread
    /// boundary change.
    pub(crate) rev: u64,
}

/// The rows of a timeline.
#[derive(Debug, Clone, Default)]
pub(crate) struct Layout {
    pub(crate) key: Option<LayoutKey>,
    pub(crate) rows: Vec<Row>,
    pub(crate) items: Vec<Item>,
}

/// What [`build`] reads.
#[derive(Debug, Clone)]
pub(crate) struct Input {
    pub(crate) header: Option<ItemHeader>,
    pub(crate) entries: Vec<TimelineEntry>,
    pub(crate) ctx: md_render::Context,
    pub(crate) events: bool,
    /// Entries strictly after this time are unread; `None`: nothing is.
    pub(crate) seen_at: Option<String>,
    /// The login of whoever posts from here: their own entries are never
    /// "new" to them.
    pub(crate) me: Option<String>,
}

/// The text of a comment as Markdown: a plain-text host's text is escaped so
/// its `*` and `#` stay what they were, and lines keep their breaks.
pub(crate) fn markdown_of(markup: &Markup) -> String {
    match markup.flavor {
        MarkupFlavor::PlainText => escape_plain(&markup.text),
        _ => markup.text.clone(),
    }
}

fn escape_plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    let mut first = true;
    for line in text.lines() {
        if !first {
            out.push_str("  \n");
        }
        first = false;
        for c in line.chars() {
            if matches!(c, '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '#' | '~' | '|') {
                out.push('\\');
            }
            out.push(c);
        }
    }
    out
}

fn is_me(me: &Option<String>, author: &gitpoulpi_hosting::conversation::Actor) -> bool {
    me.as_deref()
        .is_some_and(|m| m.eq_ignore_ascii_case(&author.login))
}

/// Lays out the whole timeline for `content_width` cells of text (the
/// description, then every entry; events only with `input.events`).
pub(crate) fn build(input: &Input, content_width: usize, key: LayoutKey) -> Layout {
    let mut out = Layout {
        key: Some(key),
        ..Layout::default()
    };
    if let Some(h) = &input.header {
        let at = out.rows.len();
        let item = out.items.len();
        out.items.push(Item {
            key: "body".to_string(),
            what: What::Body,
            start: at,
            len: 0,
            unread: false,
            links: Vec::new(),
        });
        out.rows.push(Row::Head(item));
        let doc = body_doc(&h.body, content_width, BODY_INDENT, &input.ctx);
        if h.body.text.trim().is_empty() {
            out.rows.push(Row::Empty(item));
        } else {
            push_doc(&mut out, item, doc, BODY_INDENT);
        }
        out.rows.push(Row::Blank);
        close(&mut out, item);
    }
    for (index, entry) in input.entries.iter().enumerate() {
        if matches!(entry, TimelineEntry::Event(_)) && !input.events {
            continue;
        }
        let item = out.items.len();
        let key = entry_key(entry);
        let unread = input.seen_at.as_deref().is_some_and(|seen| entry.at() > seen)
            && !entry_is_mine(entry, &input.me);
        out.items.push(Item {
            key,
            what: What::Entry(index),
            start: out.rows.len(),
            len: 0,
            unread,
            links: Vec::new(),
        });
        match entry {
            TimelineEntry::Comment(c) => {
                out.rows.push(Row::Head(item));
                comment_body(&mut out, item, c, content_width, BODY_INDENT, &input.ctx);
                out.rows.push(Row::Blank);
            }
            TimelineEntry::Review(r) => {
                out.rows.push(Row::Head(item));
                if !r.body.text.trim().is_empty() {
                    let doc = body_doc(&r.body, content_width, BODY_INDENT, &input.ctx);
                    push_doc(&mut out, item, doc, BODY_INDENT);
                }
                out.rows.push(Row::Blank);
            }
            TimelineEntry::Thread(t) => {
                out.rows.push(Row::ThreadHead(item));
                if let Some(hunk) = &t.diff_hunk {
                    let lines: Vec<&str> = hunk.lines().collect();
                    let from = lines.len().saturating_sub(HUNK_LINES);
                    for line in &lines[from..] {
                        out.rows.push(Row::Hunk(item, md_render::sanitize(line)));
                    }
                }
                for (ci, c) in t.comments.iter().enumerate() {
                    out.rows.push(Row::Sub(item, ci));
                    comment_body(&mut out, item, c, content_width, THREAD_INDENT, &input.ctx);
                }
                out.rows.push(Row::Blank);
            }
            TimelineEntry::Event(_) => out.rows.push(Row::Event(item)),
        }
        close(&mut out, item);
    }
    // Links, per item, from the docs its rows hold.
    let mut links: Vec<Vec<LinkTarget>> = vec![Vec::new(); out.items.len()];
    for row in &out.rows {
        if let Row::Body(i, doc, ..) = row {
            for l in &doc.links {
                if !links[*i].contains(l) {
                    links[*i].push(l.clone());
                }
            }
        }
    }
    for (item, l) in out.items.iter_mut().zip(links) {
        item.links = l;
    }
    out
}

fn close(out: &mut Layout, item: usize) {
    let start = out.items[item].start;
    out.items[item].len = out.rows.len() - start;
}

fn comment_body(
    out: &mut Layout,
    item: usize,
    c: &Comment,
    width: usize,
    indent: u16,
    ctx: &md_render::Context,
) {
    if !c.body.text.trim().is_empty() {
        let doc = body_doc(&c.body, width, indent, ctx);
        push_doc(out, item, doc, indent);
    }
    if !c.reactions.is_empty() {
        let text = c
            .reactions
            .iter()
            .map(|r| format!("{} {}", md_render::sanitize(&r.name), r.count))
            .collect::<Vec<_>>()
            .join("  ");
        out.rows.push(Row::Reactions(item, text));
    }
}

fn body_doc(markup: &Markup, width: usize, indent: u16, ctx: &md_render::Context) -> Arc<Doc> {
    let room = width.saturating_sub(usize::from(indent));
    Arc::new(md_render::render(&markdown_of(markup), room, ctx))
}

fn push_doc(out: &mut Layout, item: usize, doc: Arc<Doc>, indent: u16) {
    // Trailing blank lines of a document are the gap's job.
    let mut n = doc.lines.len();
    while n > 0 && doc.lines[n - 1].spans.iter().all(|s| s.text.trim().is_empty()) {
        n -= 1;
    }
    for line in 0..n {
        out.rows.push(Row::Body(item, Arc::clone(&doc), line, indent));
    }
}

pub(super) fn entry_key(entry: &TimelineEntry) -> String {
    match entry {
        TimelineEntry::Comment(c) => format!("c{}", c.id),
        TimelineEntry::Review(r) => format!("r{}", r.id),
        TimelineEntry::Thread(t) => format!("t{}", t.id),
        TimelineEntry::Event(e) => format!("e{}", e.id),
    }
}

fn entry_is_mine(entry: &TimelineEntry, me: &Option<String>) -> bool {
    match entry {
        TimelineEntry::Comment(c) => is_me(me, &c.author),
        TimelineEntry::Review(r) => is_me(me, &r.author),
        TimelineEntry::Thread(t) => t.comments.last().is_some_and(|c| is_me(me, &c.author)),
        TimelineEntry::Event(e) => e.actor.as_ref().is_some_and(|a| is_me(me, a)),
    }
}

/// `2026-11-12T09:14:03Z` as `2026-11-12 09:14`.
pub(crate) fn short_time(iso: &str) -> String {
    let clean = md_render::sanitize(iso);
    let mut out: String = clean.chars().take(16).collect();
    if out.len() > 10 && out.as_bytes().get(10) == Some(&b'T') {
        out.replace_range(10..11, " ");
    }
    out
}

/// What an event says after the actor's name, as `(words)` for the status
/// verbs; composed at paint time.
pub(crate) fn event_label(kind: &EventKind) -> String {
    use gitpoulpi_i18n::msg::conversation_ui as m;
    let clean = |s: &str| md_render::sanitize(s);
    match kind {
        EventKind::Opened => m::EV_OPENED.get().to_string(),
        EventKind::Closed => m::EV_CLOSED.get().to_string(),
        EventKind::Reopened => m::EV_REOPENED.get().to_string(),
        EventKind::Merged => m::EV_MERGED.get().to_string(),
        EventKind::Labeled { added, removed } => match (added.is_empty(), removed.is_empty()) {
            (false, true) => m::ev_label_added(clean(&added.join(", "))),
            (true, false) => m::ev_label_removed(clean(&removed.join(", "))),
            _ => m::ev_label_changed(clean(&added.join(", ")), clean(&removed.join(", "))),
        },
        EventKind::Assigned { added, removed } => match (added.is_empty(), removed.is_empty()) {
            (false, true) => m::ev_assigned(clean(&added.join(", "))),
            (true, false) => m::ev_unassigned(clean(&removed.join(", "))),
            _ => m::ev_assignees_changed(clean(&added.join(", ")), clean(&removed.join(", "))),
        },
        EventKind::Renamed { from, to } => m::ev_renamed(clean(from), clean(to)),
        EventKind::CommitsPushed { count, head } => match head {
            Some(h) => m::ev_pushed_head(u64::from(*count), clean(&h.chars().take(8).collect::<String>())),
            None => m::ev_pushed(u64::from(*count)),
        },
        EventKind::CrossReferenced { from, .. } => m::ev_referenced(clean(from)),
        EventKind::StatusChanged { from, to } => m::ev_status(clean(from), clean(to)),
        EventKind::ReviewRequested { who } => m::ev_review_requested(clean(who)),
        EventKind::Other { text } => clean(text),
    }
}
