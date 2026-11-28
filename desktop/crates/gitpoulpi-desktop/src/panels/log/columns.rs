//! The Log's column header: widths, drag resize, double-click auto-fit.
//!
//! The user's widths are preferences ([`ColumnWidths`]); what is drawn is `resolve(preferences, pane
//! width)`, so a narrow window squeezes Ref and Graph first and never rewrites the preference. A drag
//! starts from the width currently drawn and writes the preference of that one column.

use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;

use gitpoulpi_i18n::msg;
use gpui_kit::component::h_flex;
use gpui_kit::{
    div, px, App, Context, FontWeight, Hsla, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, SharedString, Styled as _, TextRun, Window,
};

use super::pills::{icons_width, pills_for};
use super::LogPanel;
use crate::graph::CW;
use crate::prefs::{resolve, Col, ColumnLayout, ColumnWidths, PrefsStore};
use crate::theme::Skin;

/// Pane width assumed until the first frame measures it.
const ASSUMED_WIDTH: f32 = 900.0;

#[derive(Clone, Copy)]
pub(super) struct Drag {
    col: Col,
    start_x: f32,
    start_w: f32,
}

pub(super) struct ColumnState {
    /// The user's preferred widths.
    pub widths: ColumnWidths,
    /// Width of the header row as last measured; read by the next render.
    avail: Rc<Cell<f32>>,
    pub drag: Option<Drag>,
}

impl ColumnState {
    pub fn for_repo(repo: &str, cx: &App) -> Self {
        Self {
            widths: PrefsStore::get(cx).columns_for(repo),
            avail: Rc::new(Cell::new(ASSUMED_WIDTH)),
            drag: None,
        }
    }

    /// The widths to draw for the measured pane.
    pub fn layout(&self) -> ColumnLayout {
        resolve(&self.widths, self.avail.get())
    }

    pub fn drag_to(&mut self, drag: Drag, x: f32) {
        let dx = x - drag.start_x;
        // Right-edge handles (Ref, Graph) grow with the pointer, left-edge handles against it.
        let signed = if matches!(drag.col, Col::Ref | Col::Graph) {
            dx
        } else {
            -dx
        };
        self.widths.set(drag.col, drag.start_w + signed);
    }
}

fn column_label(col: Col) -> SharedString {
    match col {
        Col::Ref => msg::log_table::COL_REF.get(),
        Col::Graph => msg::log_table::COL_GRAPH.get(),
        Col::Author => msg::log_table::COL_AUTHOR.get(),
        Col::Date => msg::log_table::COL_DATE.get(),
        Col::Sha => msg::log_table::COL_SHA.get(),
    }
    .into()
}

fn text_width(window: &Window, text: &str, size: f32, bold: bool) -> f32 {
    let mut font = window.text_style().font();
    if bold {
        font.weight = FontWeight::BOLD;
    }
    let run = TextRun {
        len: text.len(),
        font,
        color: Hsla::default(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window
        .text_system()
        .shape_line(text.to_string().into(), px(size), &[run], None)
        .width
        .as_f32()
}

impl LogPanel {
    /// Width of the column header as measured by the last frame (what the columns were resolved for).
    pub fn measured_width(&self) -> f32 {
        self.columns.avail.get()
    }

    pub(super) fn persist_columns(&self, cx: &mut Context<Self>) {
        let repo = self.repo_key();
        let widths = self.columns.widths;
        PrefsStore::set_columns(cx, Some(&repo), widths);
    }

    /// Forgets this repository's own widths: back to the shared default.
    pub fn reset_widths(&mut self, cx: &mut Context<Self>) {
        let repo = self.repo_key();
        PrefsStore::reset_columns_for(cx, &repo);
        self.columns.widths = PrefsStore::get(cx).columns_for(&repo);
        cx.notify();
    }

    fn fit(&mut self, col: Col, window: &Window) {
        const SIZE: f32 = 13.0;
        const PAD: f32 = 18.0;
        let width = match col {
            Col::Author => {
                let mut seen = HashSet::new();
                let mut w: f32 = 0.0;
                for n in self.data.nodes.iter().take(4000) {
                    if seen.insert(n.author_name.as_str()) {
                        w = w.max(text_width(window, &n.author_name, SIZE, false));
                    }
                }
                w + PAD
            }
            Col::Date => text_width(window, "0000-00-00 00:00", SIZE, false) + PAD,
            Col::Sha => text_width(window, "abcdef0", SIZE, false) + PAD,
            Col::Graph => self.data.layout.max_cells() as f32 * CW + 8.0,
            Col::Ref => {
                let mut w: f32 = 0.0;
                for idxs in self.data.refs_by_commit.values().take(3000) {
                    let pills = pills_for(&self.data.refs, idxs);
                    if let Some(p) = pills.first() {
                        let extra = if pills.len() > 1 { 34.0 } else { 0.0 };
                        w = w.max(
                            text_width(window, &p.name, 12.0, p.checked_out)
                                + icons_width(p)
                                + extra
                                + 26.0,
                        );
                    }
                }
                w
            }
        };
        self.columns.widths.set(col, width);
    }

    pub fn fit_all(&mut self, window: &Window, cx: &mut Context<Self>) {
        for col in Col::ALL {
            self.fit(col, window);
        }
        self.persist_columns(cx);
        cx.notify();
    }

    fn header_cell(&self, label: SharedString, width: Option<f32>, skin: &Skin) -> gpui_kit::Div {
        let cell = h_flex()
            .h_full()
            .items_center()
            .px_2()
            .relative()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(skin.header_fg)
            .child(div().truncate().child(label));
        match width {
            Some(w) => cell.w(px(w)).flex_none(),
            None => cell.flex_1().min_w_0(),
        }
    }

    /// A drag handle on a column edge: right edge for columns left of Message, left edge for those
    /// right of it (so the Message column always absorbs the change).
    fn handle(
        &self,
        col: Col,
        left_edge: bool,
        skin: &Skin,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let border = skin.border;
        let hover = skin.border_focused;
        let id: SharedString = format!("col-handle-{}", col as usize).into();
        let handle = div()
            .id(id)
            .absolute()
            .top_0()
            .h_full()
            .w(px(7.0))
            .cursor_col_resize()
            .flex()
            .justify_center()
            .child(div().w(px(1.0)).h_full().bg(border))
            .hover(move |s| s.bg(hover.opacity(0.35)))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, ev: &gpui_kit::MouseDownEvent, window, cx| {
                    if ev.click_count >= 2 {
                        this.fit(col, window);
                        this.persist_columns(cx);
                        cx.notify();
                    } else {
                        let start_w = this.columns.layout().get(col);
                        this.columns.drag = Some(Drag {
                            col,
                            start_x: ev.position.x.as_f32(),
                            start_w,
                        });
                    }
                    cx.stop_propagation();
                }),
            );
        if left_edge {
            handle.left(px(-3.0))
        } else {
            handle.right(px(-3.0))
        }
    }

    /// Records the header's width after layout and asks for one more frame when it changed, so the
    /// columns are resolved against the real pane width.
    fn width_probe(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let avail = self.columns.avail.clone();
        let me = cx.entity().downgrade();
        gpui_kit::canvas(
            move |bounds, _, cx| {
                let width = bounds.size.width.as_f32();
                if (avail.get() - width).abs() > 0.5 {
                    avail.set(width);
                    let me = me.clone();
                    cx.defer(move |cx| {
                        let _ = me.update(cx, |_, cx| cx.notify());
                    });
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .size_full()
    }

    pub(super) fn header(&self, skin: &Skin, cx: &mut Context<Self>) -> impl IntoElement {
        let l = self.columns.layout();
        let fixed = |this: &Self, col: Col, left_edge: bool, cx: &mut Context<Self>| {
            this.header_cell(column_label(col), Some(l.get(col)), skin)
                .child(this.handle(col, left_edge, skin, cx))
        };
        h_flex()
            .relative()
            .flex_none()
            .h(px(26.0))
            .w_full()
            .items_center()
            .border_b_1()
            .border_color(skin.border)
            .child(self.width_probe(cx))
            .child(fixed(self, Col::Ref, false, cx))
            .child(fixed(self, Col::Graph, false, cx))
            .child(self.header_cell(msg::log_table::COL_MESSAGE.get().into(), None, skin))
            .child(fixed(self, Col::Author, true, cx))
            .child(fixed(self, Col::Date, true, cx))
            .child(fixed(self, Col::Sha, true, cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> ColumnState {
        ColumnState {
            widths: ColumnWidths::default(),
            avail: Rc::new(Cell::new(1200.0)),
            drag: None,
        }
    }

    #[test]
    fn dragging_a_right_edge_handle_follows_the_pointer() {
        let mut s = state();
        let drag = Drag {
            col: Col::Ref,
            start_x: 100.0,
            start_w: 190.0,
        };
        s.drag_to(drag, 140.0);
        assert_eq!(s.widths.ref_w, 230.0);
    }

    #[test]
    fn dragging_a_left_edge_handle_goes_against_the_pointer() {
        let mut s = state();
        let drag = Drag {
            col: Col::Author,
            start_x: 500.0,
            start_w: 130.0,
        };
        s.drag_to(drag, 470.0);
        assert_eq!(s.widths.author_w, 160.0);
    }

    #[test]
    fn drag_is_clamped_to_the_column_limits() {
        let mut s = state();
        s.drag_to(
            Drag {
                col: Col::Ref,
                start_x: 0.0,
                start_w: 190.0,
            },
            -5000.0,
        );
        assert_eq!(s.widths.ref_w, Col::Ref.min());
    }

    #[test]
    fn a_narrow_pane_leaves_the_stored_widths_alone() {
        let s = state();
        s.avail.set(600.0);
        let drawn = s.layout();
        assert!(drawn.ref_w < s.widths.ref_w);
        assert_eq!(s.widths, ColumnWidths::default());
    }
}
