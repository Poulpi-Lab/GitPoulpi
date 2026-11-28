//! Fetching the Log: the first page with refs, stashes and working changes, then further pages as the
//! list is scrolled. The graph layout is computed on a background thread each time.

use std::sync::Arc;
use std::time::Instant;

use gitpoulpi_core::domain::{CommitNode, Ref, Stash};
use gpui_kit::{AppContext as _, Context};

use super::data::{compute_layout, LogData};
use super::{LogEvent, LogPanel, Summary};
use crate::daemon::{HistoryPlan, LogSource, NEXT_PAGE};
use crate::prefs::PrefsStore;

impl LogPanel {
    /// Loads (or reloads) the current repository from the top.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let started = Instant::now();
        self.columns.widths = PrefsStore::get(cx).columns_for(&self.repo_key());
        let backend = self.backend.clone();
        let refs = backend.refs();
        let stashes = backend.stashes();
        let status = backend.status();
        let head = backend.head();
        let palette_len = Self::palette_len(cx);
        self._load = Some(cx.spawn(async move |this, cx| {
            let refs: Vec<Ref> = refs.await.unwrap_or_default();
            let stashes: Vec<Stash> = stashes.await.unwrap_or_default();
            let status = status.await.map(|w| w.files).unwrap_or_default();
            let head = head.await.ok();
            let start = backend.history_start(&refs).await;
            let (nodes, plan, more, total) = match start {
                Ok(s) => (s.nodes, s.plan, s.more, s.total),
                Err(_) => (
                    Vec::new(),
                    HistoryPlan {
                        source: LogSource::AllRefs,
                        head_only: 0,
                    },
                    false,
                    None,
                ),
            };
            let dirty = !status.is_empty();
            let (n2, r2, s2) = (Arc::new(nodes), Arc::new(refs), Arc::new(stashes));
            let (nodes_bg, refs_bg, stashes_bg) = (n2.clone(), r2.clone(), s2.clone());
            let layout = cx
                .background_spawn(async move {
                    compute_layout(&nodes_bg, &refs_bg, &stashes_bg, dirty, head, palette_len)
                })
                .await;
            let ms = started.elapsed().as_millis();
            let _ = this.update(cx, |this, cx| {
                this.data = LogData::new(n2, r2, s2, status.clone(), plan, head, layout);
                this.more = more;
                this.loading_more = false;
                this.selected = 0;
                this.recompute_matches(cx);
                let branch = this.data.head_branch.clone();
                cx.emit(LogEvent::Loaded(Summary {
                    loaded: this.data.nodes.len(),
                    total,
                    branch,
                    ms,
                    status,
                }));
                if let Some(sel) = this.data.selection_at(0) {
                    cx.emit(LogEvent::Selected(sel));
                }
                cx.notify();
            });
        }));
    }

    /// Fetches the next page once the list nears its end.
    pub(super) fn load_more(&mut self, cx: &mut Context<Self>) {
        let Some(plan) = self.data.plan else { return };
        if self.loading_more || !self.more {
            return;
        }
        self.loading_more = true;
        let loaded = self.data.nodes.len();
        let page = self.backend.history_next(plan, loaded);
        let (refs, stashes) = (self.data.refs.clone(), self.data.stashes.clone());
        let dirty = self.data.dirty();
        let head = self.data.head_target();
        let palette_len = Self::palette_len(cx);
        self._load = Some(cx.spawn(async move |this, cx| {
            let page: Vec<CommitNode> = page.await.unwrap_or_default();
            let fetched = page.len();
            let all = this
                .update(cx, |this, _| {
                    Arc::make_mut(&mut this.data.nodes).extend(page);
                    this.data.nodes.clone()
                })
                .ok();
            let Some(all) = all else { return };
            let layout = cx
                .background_spawn(async move {
                    compute_layout(&all, &refs, &stashes, dirty, head, palette_len)
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.data.layout = Arc::new(layout);
                this.more = fetched as u32 >= NEXT_PAGE;
                this.loading_more = false;
                this.recompute_matches(cx);
                cx.notify();
            });
        }));
    }
}
