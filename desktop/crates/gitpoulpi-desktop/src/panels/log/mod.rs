//! The Log: GitPoulpi's centre pane. Columns Ref | Graph | Message | Author | Date | SHA, one row per
//! commit with the WIP row on top and stash rows among the commits, ref pills in the Ref column joined
//! to their commit's node by a one-pixel connector, lanes drawn by the `graph` module.
//!
//! Built on `uniform_list` rather than the kit's `DataTable`: the pill connector has to cross from the
//! Ref cell into the Graph cell, the Message column must flex, and every column needs header-edge drag
//! resize with double-click auto-fit and persistence.
//!
//! Split by concern: `data` (what is loaded), `load` (fetching and paging), `search` (find bar),
//! `columns` (widths, header, resize), `rows` (one row's cells), `pills` (ref pills).

mod columns;
mod data;
mod load;
mod pills;
mod ref_menu;
mod rows;
mod search;

use std::collections::HashSet;

use gitpoulpi_app::graph::{LazyLog, LogLoadSettings, TotalShown};
use gitpoulpi_config::CommitCountMethod;
use gitpoulpi_core::domain::{Oid, RefFilter, StatusEntry};
use gitpoulpi_i18n::msg;
use gpui_kit::component::{scroll::Scrollbar, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    actions, div, px, uniform_list, App, Context, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, KeyBinding, MouseMoveEvent, MouseUpEvent,
    ParentElement as _, Render, ScrollStrategy, Styled as _, Subscription, Task,
    UniformListScrollHandle, Window,
};

use crate::daemon::Shared;
use crate::graph::Palette;
use crate::theme::Skin;

use columns::ColumnState;
use data::LogData;
pub use data::Selection;
use search::SearchState;

actions!(
    gp_log,
    [
        RowUp,
        RowDown,
        PageUp,
        PageDown,
        RowHome,
        RowEnd,
        CloseSearch
    ]
);

/// Key context of the Log; shortcuts below only apply while it has focus.
const CONTEXT: &str = "GpLog";

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", RowUp, Some(CONTEXT)),
        KeyBinding::new("down", RowDown, Some(CONTEXT)),
        KeyBinding::new("k", RowUp, Some(CONTEXT)),
        KeyBinding::new("j", RowDown, Some(CONTEXT)),
        KeyBinding::new("pageup", PageUp, Some(CONTEXT)),
        KeyBinding::new("pagedown", PageDown, Some(CONTEXT)),
        KeyBinding::new("home", RowHome, Some(CONTEXT)),
        KeyBinding::new("end", RowEnd, Some(CONTEXT)),
        KeyBinding::new("escape", CloseSearch, Some(CONTEXT)),
    ]);
}

/// What the status line shows about the loaded history.
pub struct Summary {
    pub loaded: usize,
    pub total: TotalShown,
    pub branch: Option<String>,
    pub ms: u128,
    pub status: Vec<StatusEntry>,
}

pub enum LogEvent {
    Selected(Selection),
    Loaded(Summary),
}

pub struct LogPanel {
    backend: Shared,
    data: LogData,
    selected: usize,
    scroll: UniformListScrollHandle,
    columns: ColumnState,
    search: SearchState,
    focus: FocusHandle,
    /// Paging: page size, ceiling, lazy on/off and the detached-HEAD front (shared with the terminal UI).
    lazy: LazyLog,
    /// The hidden and soloed refs, as the Refs sidebar edits them.
    hidden: HashSet<String>,
    solo: HashSet<String>,
    /// What the hidden set means to the daemon's walk.
    filter: RefFilter,
    total: load::CommitCount,
    count_method: CommitCountMethod,
    last_ms: u128,
    /// Bumped by every reload so a layout computed for an older list is dropped.
    generation: u64,
    _subs: Vec<Subscription>,
    _load: Option<Task<()>>,
    /// The page fetch in flight: its own slot, so a page never cancels a reload.
    _page: Option<Task<()>>,
    /// A reload is in flight: no page is asked for the list it is replacing.
    reloading: bool,
    _layout: Option<Task<()>>,
}

/// The status line's commit count: `500 commits`, `500 / 48213` or `500 / ...` while it is counted.
pub fn commit_count_text(loaded: usize, total: TotalShown) -> String {
    match total {
        TotalShown::Whole => msg::footer::commit_count(loaded as u64),
        TotalShown::Counting => msg::footer::commit_total_pending(loaded),
        TotalShown::Of(t) => msg::footer::commit_total(t, loaded),
    }
}

/// The log settings of the user's config (defaults when it cannot be read).
fn load_settings() -> (LogLoadSettings, CommitCountMethod) {
    match gitpoulpi_config::Settings::resolve_ambient() {
        Ok(s) => (
            LogLoadSettings::from_config(&s.config().general),
            s.config().log.commit_count,
        ),
        Err(_) => (LogLoadSettings::default(), CommitCountMethod::default()),
    }
}

impl EventEmitter<LogEvent> for LogPanel {}

impl LogPanel {
    pub fn new(backend: Shared, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (search, subs) = SearchState::new(window, cx);
        let columns = ColumnState::for_repo(&backend.repo_root().to_string_lossy(), cx);
        let (hidden, solo) = gitpoulpi_app::refs_tree::load_ref_visibility(&backend.repo_root());
        let (settings, count_method) = load_settings();
        let mut this = Self {
            backend,
            data: LogData::default(),
            selected: 0,
            scroll: UniformListScrollHandle::new(),
            columns,
            search,
            focus: cx.focus_handle(),
            lazy: LazyLog::new(settings),
            filter: gitpoulpi_app::graph::walk_filter(&hidden, &solo),
            hidden,
            solo,
            total: load::CommitCount::default(),
            count_method,
            last_ms: 0,
            generation: 0,
            _subs: subs,
            _load: None,
            _page: None,
            reloading: false,
            _layout: None,
        };
        this.reload(cx);
        this
    }

    fn repo_key(&self) -> String {
        self.backend.repo_root().to_string_lossy().into_owned()
    }

    fn palette_len(cx: &App) -> usize {
        cx.global::<Skin>().lanes.len()
    }

    /// Number of commits held (stash and WIP rows excluded).
    pub fn loaded_commits(&self) -> usize {
        self.data.nodes.len()
    }

    /// What the status line says about the total (`Whole`, `Counting` or `Of(total)`).
    pub fn commit_total(&self) -> TotalShown {
        self.total
            .shown(self.data.held.len(), self.lazy.is_exhausted())
    }

    /// Whether the whole history is held (the walk was read to its root).
    pub fn is_exhausted(&self) -> bool {
        self.lazy.is_exhausted()
    }

    /// The selected row.
    pub fn selected_row(&self) -> usize {
        self.selected
    }

    /// The commit of the selected row (`None` on the WIP, a stash or an empty Log).
    pub fn selected_commit(&self) -> Option<Oid> {
        match self.data.key_at(self.selected)? {
            data::RowKey::Commit(oid) => Some(oid),
            _ => None,
        }
    }

    /// Number of rows shown, WIP and stash rows included.
    pub fn row_count(&self) -> usize {
        self.data.entry_count()
    }

    /// The column widths drawn for the pane as last measured.
    pub fn column_layout(&self) -> crate::prefs::ColumnLayout {
        self.columns.layout()
    }

    pub fn select(&mut self, ix: usize, cx: &mut Context<Self>) {
        let total = self.data.entry_count();
        if total == 0 {
            return;
        }
        let ix = ix.min(total - 1);
        self.selected = ix;
        self.scroll.scroll_to_item(ix, ScrollStrategy::Nearest);
        if let Some(sel) = self.data.selection_at(ix) {
            cx.emit(LogEvent::Selected(sel));
        }
        cx.notify();
    }

    fn move_by(&mut self, delta: isize, cx: &mut Context<Self>) {
        let ix = self.selected.saturating_add_signed(delta);
        self.select(ix, cx);
    }

    pub fn select_oid(&mut self, oid: Oid, cx: &mut Context<Self>) {
        if let Some(ix) = self.data.position_of(oid) {
            self.select(ix, cx);
        }
    }
}

impl Focusable for LogPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for LogPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let palette = Palette {
            lanes: skin.lanes.clone(),
            wip: skin.wip,
            stash: skin.stash,
        };
        let total = self.data.entry_count();
        let layout = self.columns.layout();

        if let Some(drag) = self.columns.drag {
            // Track the pointer window-wide while a column edge is held.
            let entity = cx.entity();
            window.on_mouse_event({
                let entity = entity.clone();
                move |ev: &MouseMoveEvent, phase, _window, cx| {
                    if phase.bubble() {
                        entity.update(cx, |this, cx| {
                            this.columns.drag_to(drag, ev.position.x.as_f32());
                            cx.notify();
                        });
                    }
                }
            });
            window.on_mouse_event(move |_: &MouseUpEvent, phase, _window, cx| {
                if phase.bubble() {
                    entity.update(cx, |this, cx| {
                        this.columns.drag = None;
                        this.persist_columns(cx);
                        cx.notify();
                    });
                }
            });
        }

        let rows = rows::RowsContext {
            me: cx.entity().downgrade(),
            data: self.data.clone(),
            selected: self.selected,
            layout,
            palette,
            matches: self.search.match_set(),
            query_active: self.search.active(),
            more: !self.lazy.is_exhausted() && !self.lazy.is_at_ceiling(),
            total,
        };
        let list = uniform_list("log-rows", total, move |range, _window, cx| {
            rows.render(range, cx)
        })
        .size_full()
        .track_scroll(&self.scroll);

        let focus_ring = self.focus.is_focused(window);
        v_flex()
            .size_full()
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &RowUp, _, cx| this.move_by(-1, cx)))
            .on_action(cx.listener(|this, _: &RowDown, _, cx| this.move_by(1, cx)))
            .on_action(cx.listener(|this, _: &PageUp, _, cx| this.move_by(-20, cx)))
            .on_action(cx.listener(|this, _: &PageDown, _, cx| this.move_by(20, cx)))
            .on_action(cx.listener(|this, _: &RowHome, _, cx| this.select(0, cx)))
            .on_action(cx.listener(|this, _: &RowEnd, _, cx| {
                let n = this.data.entry_count();
                this.select(n.saturating_sub(1), cx)
            }))
            .on_action(cx.listener(|this, _: &CloseSearch, window, cx| {
                if this.search.open {
                    this.toggle_search(window, cx);
                }
            }))
            .border_1()
            .border_color(if focus_ring {
                skin.border_focused
            } else {
                skin.border
            })
            .rounded(px(3.0))
            .when(self.search.open, |d| d.child(self.search_bar(&skin, cx)))
            .child(self.header(&skin, cx))
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .child(list)
                    .child(Scrollbar::vertical(&self.scroll)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_commit_count_reads_like_the_terminal_uis() {
        assert_eq!(
            commit_count_text(500, TotalShown::Of(48213)),
            msg::footer::commit_total(48213, 500)
        );
        assert_eq!(
            commit_count_text(500, TotalShown::Counting),
            msg::footer::commit_total_pending(500)
        );
        assert_eq!(
            commit_count_text(2500, TotalShown::Whole),
            msg::footer::commit_count(2500)
        );
        assert_ne!(
            commit_count_text(500, TotalShown::Of(48213)),
            commit_count_text(500, TotalShown::Counting)
        );
    }
}
