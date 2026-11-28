//! Fetching the Log: the first page with refs, stashes and working changes, then further pages as the
//! list is scrolled. When to ask, how much and what to do with the answer is the shared
//! `LazyLog`; the graph layout is computed on a background thread each time.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use gitpoulpi_app::graph::{lookahead_rows, refs_key, CommitTotal};
use gitpoulpi_core::domain::{CommitNode, Ref, Stash};
use gpui_kit::{AppContext as _, Context, Task};

use super::data::{compute_layout, shown_nodes, LogData};
use super::{load_settings, LogEvent, LogPanel, Summary};
use crate::prefs::PrefsStore;

/// A count in flight: the answer lands in `reply`, dropping this cancels it.
pub struct PendingCount {
    reply: Arc<Mutex<Option<Option<u64>>>>,
    _task: Task<()>,
}

/// The commit total of the status line (`CommitTotal` decides when to count and what to show).
pub type CommitCount = CommitTotal<PendingCount>;

impl LogPanel {
    /// Loads (or reloads) the current repository from the top.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let started = Instant::now();
        self.columns.widths = PrefsStore::get(cx).columns_for(&self.repo_key());
        let (settings, method) = load_settings(cx);
        self.lazy.apply_settings(settings);
        self.count_method = method;
        self.generation += 1;
        let generation = self.generation;
        self.reloading = true;
        self._page = None;
        let backend = self.backend.clone();
        let refs = backend.refs();
        let stashes = backend.stashes();
        let status = backend.status();
        let head = backend.head();
        let filter = self.filter.clone();
        let solo = self.solo.clone();
        let depth = self.lazy.refresh_depth();
        let palette_len = Self::palette_len(cx);
        self._load = Some(cx.spawn(async move |this, cx| {
            let refs: Vec<Ref> = refs.await.unwrap_or_default();
            let stashes: Vec<Stash> = stashes.await.unwrap_or_default();
            let status = status.await.map(|w| w.files).unwrap_or_default();
            let head = head.await.ok();
            let refresh = backend.history_refresh(filter, depth).await;
            let (held, asked, walk) = match refresh {
                Ok(r) => (r.nodes, r.asked, Some(r.walk)),
                Err(_) => (Vec::new(), depth, None),
            };
            let dirty = !status.is_empty();
            let (held, refs, stashes) = (Arc::new(held), Arc::new(refs), Arc::new(stashes));
            let shown = shown_nodes(&held, &refs, &solo);
            let (shown_bg, refs_bg, stashes_bg) = (shown.clone(), refs.clone(), stashes.clone());
            let layout = cx
                .background_spawn(async move {
                    compute_layout(&shown_bg, &refs_bg, &stashes_bg, dirty, head, palette_len)
                })
                .await;
            let ms = started.elapsed().as_millis();
            let _ = this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                let loaded = held.len();
                this.reloading = false;
                let kept = this.data.key_at(this.selected);
                let old_row = this.selected;
                this.data = LogData::new(
                    held,
                    shown,
                    refs,
                    stashes,
                    status.clone(),
                    walk,
                    head,
                    layout,
                );
                // Everything the old list had in flight (a page, the "end of history" latch) described it.
                this.lazy.restart();
                this.lazy.on_refresh_asked(loaded, asked);
                this.last_ms = ms;
                // The selection follows its commit; when that is gone the nearest row stays selected.
                this.selected = kept
                    .and_then(|k| this.data.position_of_key(&k))
                    .unwrap_or_else(|| old_row.min(this.data.entry_count().saturating_sub(1)));
                this.recompute_matches(cx);
                this.tick_total(cx);
                this.emit_summary(cx);
                this.recompute_marks();
                this.sync_right(cx);
            });
        }));
    }

    /// The Refs sidebar changed which refs are hidden or soloed. What the daemon walks changes with the
    /// hidden set (reload from the top); solo only filters what is held.
    pub fn set_visibility(
        &mut self,
        hidden: HashSet<String>,
        solo: HashSet<String>,
        cx: &mut Context<Self>,
    ) {
        let filter = gitpoulpi_app::graph::walk_filter(&hidden, &solo);
        let walk_changed = filter != self.filter;
        let solo_changed = solo != self.solo;
        self.hidden = hidden;
        self.solo = solo;
        if walk_changed {
            self.filter = filter;
            self.lazy.restart();
            self.total.forget();
            self.reload(cx);
        } else if solo_changed {
            self.relayout(cx);
        }
    }

    /// Fetches the next page when the list nears its end. `rows_below_view` and `visible_rows` are in
    /// Log rows as shown (the soloed list is shorter than the held one).
    pub(super) fn load_more(
        &mut self,
        rows_below_view: usize,
        visible_rows: usize,
        cx: &mut Context<Self>,
    ) {
        if self.reloading {
            return;
        }
        let Some(request) = self.lazy.next_page(rows_below_view, visible_rows) else {
            return;
        };
        let Some(walk) = self.data.walk.clone() else {
            return;
        };
        self.lazy.mark_requested(request);
        let page = self.backend.history_page(&walk, request);
        let generation = self.generation;
        self._page = Some(cx.spawn(async move |this, cx| {
            // A failed page ends the walk like a short one: nothing is asked again until the next reload.
            let page: Vec<CommitNode> = page.await.unwrap_or_default();
            let _ = this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                let mut held = std::mem::take(&mut this.data.held);
                let appended = this.lazy.on_page(request, page, Arc::make_mut(&mut held));
                this.data.held = held;
                if appended {
                    this.relayout(cx);
                } else {
                    this.emit_summary(cx);
                }
            });
        }));
    }

    /// Lays the held commits out again (a page arrived, or the solo set changed).
    pub(super) fn relayout(&mut self, cx: &mut Context<Self>) {
        let generation = self.generation;
        let held = self.data.held.clone();
        let (refs, stashes) = (self.data.refs.clone(), self.data.stashes.clone());
        let shown = shown_nodes(&held, &refs, &self.solo);
        let dirty = self.data.dirty();
        let head = self.data.head_target();
        let palette_len = Self::palette_len(cx);
        let shown_bg = shown.clone();
        self._layout = Some(cx.spawn(async move |this, cx| {
            let layout = cx
                .background_spawn(async move {
                    compute_layout(&shown_bg, &refs, &stashes, dirty, head, palette_len)
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                this.data.nodes = shown;
                this.data.layout = Arc::new(layout);
                let rows = this.data.entry_count();
                this.selected = this.selected.min(rows.saturating_sub(1));
                this.recompute_matches(cx);
                this.tick_total(cx);
                this.emit_summary(cx);
                cx.notify();
            });
        }));
    }

    /// Asks for the total when the refs moved, and takes the answer once it arrived.
    pub(super) fn tick_total(&mut self, cx: &mut Context<Self>) {
        let Some(walk) = self.data.walk.clone() else {
            return;
        };
        let backend = self.backend.clone();
        let key = refs_key(&self.data.refs);
        self.total.tick(
            self.count_method,
            key,
            self.lazy.is_exhausted(),
            || {
                let reply: Arc<Mutex<Option<Option<u64>>>> = Arc::default();
                let answer = reply.clone();
                let count = backend.history_total(&walk);
                let task = cx.spawn(async move |this, cx| {
                    let n = count.await.ok();
                    *answer.lock().unwrap() = Some(n);
                    let _ = this.update(cx, |this, cx| {
                        this.tick_total(cx);
                        this.emit_summary(cx);
                    });
                });
                PendingCount { reply, _task: task }
            },
            |pending| pending.reply.lock().unwrap().take(),
        );
    }

    /// Tells the window what the status line shows.
    pub(super) fn emit_summary(&self, cx: &mut Context<Self>) {
        let loaded = self.data.held.len();
        cx.emit(LogEvent::Loaded(Summary {
            loaded,
            total: self.total.shown(loaded, self.lazy.is_exhausted()),
            branch: self.data.head_branch.clone(),
            ms: self.last_ms,
            status: self.data.status.to_vec(),
        }));
    }

    /// Rows below which the next page is due, for the row painter.
    pub(super) fn lookahead(visible_rows: usize) -> usize {
        lookahead_rows(visible_rows)
    }
}
