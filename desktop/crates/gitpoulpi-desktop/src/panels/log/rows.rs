//! One Log row: Ref pill, graph cells, message, author, date, SHA.

use std::collections::HashSet;
use std::ops::Range;
use std::sync::Arc;

use gitpoulpi_core::domain::FileStatus;
use gpui_kit::assets::IconName as AssetIcon;
use gpui_kit::component::{h_flex, Icon};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, AnyElement, App, ClickEvent, FontWeight, InteractiveElement as _, IntoElement,
    MouseButton, MouseDownEvent, ParentElement as _, StatefulInteractiveElement as _, Styled as _,
    WeakEntity,
};

use super::data::LogData;
use super::pills::{pill_element, pills_for};
use super::LogPanel;
use crate::graph::{Entry, Palette, RowGraphic, CH};
use crate::prefs::ColumnLayout;
use crate::strings;
use crate::theme::Skin;
use crate::util::dates;

/// Everything the row closure of the list needs; owned, so the closure can outlive `render`.
pub(super) struct RowsContext {
    pub me: WeakEntity<LogPanel>,
    pub data: LogData,
    pub selected: usize,
    pub layout: ColumnLayout,
    pub palette: Palette,
    /// The commit whose pill cursor is on, and which pill (see `LogPanel::pill_cursor`).
    pub pill_cursor: Option<(gitpoulpi_core::domain::Oid, usize)>,
    pub marked: Arc<HashSet<usize>>,
    pub matches: Arc<HashSet<usize>>,
    pub query_active: bool,
    pub more: bool,
    pub total: usize,
}

/// The cells of one row that differ by kind.
struct Cells {
    ref_cell: gpui_kit::Div,
    graphic: Option<(RowGraphic, bool)>,
    message: gpui_kit::Div,
    author: String,
    date: String,
    sha: String,
}

impl RowsContext {
    pub fn render(&self, range: Range<usize>, cx: &mut App) -> Vec<AnyElement> {
        let below = self.total.saturating_sub(range.end);
        let visible = range.len();
        if self.more && below <= LogPanel::lookahead(visible) {
            let me = self.me.clone();
            cx.defer(move |cx| {
                let _ = me.update(cx, |this, cx| this.load_more(below, visible, cx));
            });
        }
        let skin = cx.global::<Skin>().clone();
        range.map(|ix| self.row(ix, &skin)).collect()
    }

    fn row(&self, ix: usize, skin: &Skin) -> AnyElement {
        let cursor = ix == self.selected;
        let selected = cursor || self.marked.contains(&ix);
        let dim = self.query_active && !self.matches.contains(&ix);
        let entries = self.data.layout.entries();
        let entry = entries[ix];
        let me = self.me.clone();
        let me_menu = self.me.clone();
        let l = self.layout;
        let cells = match entry {
            Entry::Commit(ci) => self.commit_cells(entry, ci, dim, skin),
            Entry::Wip => self.wip_cells(entry, skin),
            Entry::Stash(si) => self.stash_cells(entry, si, skin),
        };
        let graph_cell = {
            let el = div()
                .w(px(l.graph_w))
                .flex_none()
                .h_full()
                .overflow_hidden()
                .relative();
            match cells.graphic {
                Some((graphic, connector)) => {
                    let palette = self.palette.clone();
                    el.child(
                        gpui_kit::canvas(
                            |_, _, _| (),
                            move |bounds, _, window, _| {
                                graphic.paint(window, bounds, ix, connector, &palette)
                            },
                        )
                        .size_full(),
                    )
                }
                None => el,
            }
        };
        let text_cell = |text: String, width: f32| {
            div()
                .w(px(width))
                .flex_none()
                .px_2()
                .truncate()
                .text_color(skin.muted)
                .child(text)
        };
        let (selection, hover) = (skin.selection, skin.hover);
        h_flex()
            .id(("row", ix))
            .w_full()
            .h(px(CH))
            .flex_none()
            .items_center()
            .when(selected, |d| d.bg(selection))
            .hover(move |d| if selected { d } else { d.bg(hover) })
            .on_click({
                let me = me.clone();
                move |ev: &ClickEvent, window, cx| {
                    let mods = ev.modifiers();
                    let _ = me.update(cx, |this, cx| this.click_row(ix, mods, window, cx));
                }
            })
            .on_mouse_down(
                MouseButton::Right,
                move |ev: &MouseDownEvent, window, cx| {
                    let at = ev.position;
                    // Asked before the Log is updated: the window state reads the Log too.
                    let palette =
                        crate::commands::context::window_state(window, cx).palette_context();
                    let _ = me_menu.update(cx, |this, cx| {
                        this.open_menu_on(ix, at, palette, window, cx)
                    });
                },
            )
            .child(cells.ref_cell.w(px(l.ref_w)).flex_none().h_full())
            .child(graph_cell)
            .child(cells.message)
            .child(text_cell(cells.author, l.author_w))
            .child(text_cell(cells.date, l.date_w))
            .child(text_cell(cells.sha, l.sha_w))
            .into_any_element()
    }

    fn commit_cells(&self, entry: Entry, ci: usize, dim: bool, skin: &Skin) -> Cells {
        let node = &self.data.nodes[ci];
        let graphic = self.data.layout.graphic(entry);
        let lane = graphic
            .as_ref()
            .map(|g| g.lane_color(&self.palette))
            .unwrap_or(skin.muted);
        let pills = self
            .data
            .refs_by_commit
            .get(&node.id)
            .map(|i| pills_for(&self.data.refs, i))
            .unwrap_or_default();
        let has_pill = !pills.is_empty();
        // The pill cursor on this commit shows its pill in the Ref column (else the first one).
        let cursor = self
            .pill_cursor
            .filter(|(oid, _)| *oid == node.id)
            .map(|(_, i)| i)
            .filter(|i| *i < pills.len());
        let shown = (!pills.is_empty()).then(|| cursor.unwrap_or(0));
        let ref_cell = h_flex()
            .items_center()
            .pl(px(4.0))
            .overflow_hidden()
            .when_some(shown.map(|i| (i, &pills[i])), |d, (i, p)| {
                let on_cursor = cursor == Some(i);
                d.child(super::ref_menu::interactive(
                    ci,
                    node.id,
                    p,
                    div()
                        .rounded(px(4.0))
                        .border_1()
                        .border_color(if on_cursor {
                            skin.accent
                        } else {
                            gpui_kit::transparent_black()
                        })
                        .child(pill_element(p, pills.len() - 1, lane, skin)),
                ))
                .child(
                    div()
                        .flex_1()
                        .min_w(px(4.0))
                        .h(px(1.0))
                        .bg(lane)
                        .ml(px(2.0)),
                )
            });
        // The terminal UI draws every Message cell bold, muted when the row is dimmed by a search.
        let message = div()
            .flex_1()
            .min_w_0()
            .px_2()
            .truncate()
            .when(!dim, |d| d.font_weight(FontWeight::BOLD))
            .when(dim, |d| d.text_color(skin.muted))
            .child(node.summary.clone());
        Cells {
            ref_cell,
            graphic: graphic.map(|g| (g, has_pill)),
            message,
            author: node.author_name.clone(),
            date: dates::iso_date(node.timestamp),
            sha: node.id.to_hex()[..7].to_string(),
        }
    }

    fn wip_cells(&self, entry: Entry, skin: &Skin) -> Cells {
        let (mut added, mut modified) = (0, 0);
        for e in self.data.status.iter() {
            if e.index == Some(FileStatus::Added)
                || e.worktree == Some(FileStatus::Untracked)
                || e.worktree == Some(FileStatus::Added)
            {
                added += 1;
            } else {
                modified += 1;
            }
        }
        let counts = h_flex()
            .gap_3()
            .text_xs()
            .child(
                h_flex()
                    .gap_1()
                    .text_color(skin.warning)
                    .child(Icon::new(AssetIcon::Pencil).size(px(11.0)))
                    .child(format!("{modified}")),
            )
            .child(
                h_flex()
                    .gap_1()
                    .text_color(skin.success)
                    .child("+")
                    .child(format!("{added}")),
            );
        let message = h_flex()
            .flex_1()
            .min_w_0()
            .px_2()
            .items_center()
            .child(
                div()
                    .italic()
                    .text_color(skin.muted)
                    .child(strings::WIP_PLACEHOLDER),
            )
            .child(div().flex_1())
            .child(counts);
        Cells {
            ref_cell: h_flex(),
            graphic: self.data.layout.graphic(entry).map(|g| (g, false)),
            message,
            author: String::new(),
            date: String::new(),
            sha: String::new(),
        }
    }

    fn stash_cells(&self, entry: Entry, si: usize, skin: &Skin) -> Cells {
        let (text, id, date, sha) = match self.data.stashes.get(si) {
            Some(st) => (
                st.message.clone(),
                strings::stash_name(st.id.0),
                dates::iso_date(st.timestamp),
                st.commit
                    .map(|c| c.to_hex()[..7].to_string())
                    .unwrap_or_default(),
            ),
            None => Default::default(),
        };
        let message = div()
            .flex_1()
            .min_w_0()
            .px_2()
            .truncate()
            .italic()
            .text_color(skin.muted)
            .child(text);
        Cells {
            ref_cell: h_flex(),
            graphic: self.data.layout.graphic(entry).map(|g| (g, false)),
            message,
            author: id,
            date,
            sha,
        }
    }
}
