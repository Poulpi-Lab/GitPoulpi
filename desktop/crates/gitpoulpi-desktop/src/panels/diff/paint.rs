//! Painting the diff's rows: one monospace row (a half, or a whole Inline line) with its cursor and
//! selection look, its click, and the buttons on a hunk header of a working-tree diff.

use std::ops::Range;
use std::sync::Arc;

use gitpoulpi_i18n::msg::diff_actions as msg_actions;
use gpui_kit::component::h_flex;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, AnyElement, FontStyle, FontWeight, HighlightStyle, Hsla, InteractiveElement as _,
    IntoElement, ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _,
    StyledText, TextRun, Window,
};

use super::rows::{body_look, Line, Look, Rows};
use super::{DiffPane, DiffSource, LINE_H, TEXT_PX};
use crate::daemon::staging::PartOp;
use crate::fonts;
use crate::theme::{DiffColors, Skin};

/// The width of one monospace character at the diff's size.
pub(super) fn char_width(window: &Window) -> f32 {
    let mut font = window.text_style().font();
    font.family = fonts::MONO_FAMILY.into();
    let sample = "0000000000";
    let run = TextRun {
        len: sample.len(),
        font,
        color: Hsla::default(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window
        .text_system()
        .shape_line(sample.into(), px(TEXT_PX), &[run], None)
        .width
        .as_f32()
        / sample.len() as f32
}

/// A run of spans as one text with the looks laid over it.
pub(super) fn text_of(spans: &[gitpoulpi_app::diff::StyledSpan], c: &DiffColors) -> StyledText {
    let mut text = String::new();
    let mut runs: Vec<(Range<usize>, HighlightStyle)> = Vec::new();
    for s in spans {
        if s.text.is_empty() {
            continue;
        }
        let start = text.len();
        text.push_str(&s.text);
        let look = Look::of(s).style(c);
        runs.push((
            start..text.len(),
            HighlightStyle {
                color: Some(look.fg),
                background_color: look.bg,
                font_weight: look.bold.then_some(FontWeight::BOLD),
                font_style: look.italic.then_some(FontStyle::Italic),
                ..HighlightStyle::default()
            },
        ));
    }
    StyledText::new(SharedString::from(text)).with_highlights(runs)
}

/// What the row closure of the list needs; owned so the closure outlives `render`.
pub(super) struct Paint {
    pub(super) rows: Arc<Rows>,
    pub(super) colors: DiffColors,
    pub(super) wrap: bool,
    /// Width of one text column in pixels when not wrapping.
    pub(super) text_w: f32,
    pub(super) cw: f32,
    pub(super) cursor: usize,
    pub(super) selection: Option<(usize, usize)>,
    /// What the diff can do with a hunk: its header carries the buttons.
    pub(super) source: DiffSource,
    pub(super) skin: Skin,
    pub(super) me: gpui_kit::WeakEntity<DiffPane>,
}

impl Paint {
    /// One half (or a whole Inline line): its gutter in a column of its own, then the text, which
    /// wraps under itself rather than under the line number.
    fn half(&self, line: &gitpoulpi_app::diff::StyledLine) -> gpui_kit::Div {
        let (gutter, rest) = match line.spans.split_first() {
            Some((g, rest)) if g.role == gitpoulpi_app::diff::DiffRole::Gutter => (Some(g), rest),
            _ => (None, line.spans.as_slice()),
        };
        let bg = body_look(line).and_then(|l| l.row_bg(&self.colors));
        let gutter_w = gutter.map(|g| g.text.chars().count() as f32 * self.cw);
        h_flex()
            .items_start()
            .min_h(px(LINE_H))
            .when_some(bg, |d, bg| d.bg(bg))
            .when_some(gutter.zip(gutter_w), |d, (g, w)| {
                d.child(
                    div()
                        .flex_none()
                        .w(px(w))
                        .child(text_of(std::slice::from_ref(g), &self.colors)),
                )
            })
            .child(if self.wrap {
                div().flex_1().min_w_0().child(text_of(rest, &self.colors))
            } else {
                div()
                    .flex_none()
                    .w(px(self.text_w))
                    .whitespace_nowrap()
                    .child(text_of(rest, &self.colors))
            })
    }

    /// One row with its cursor and selection look, its click, and (on a hunk header of a working-tree
    /// diff) the hunk's buttons.
    pub(super) fn row(&self, ix: usize) -> AnyElement {
        let inner = self.row_content(ix);
        let header = self.rows.meta.get(ix).is_some_and(|m| m.header);
        let selected = self.selection.is_some_and(|(a, b)| ix >= a && ix <= b);
        let skin = &self.skin;
        let me = self.me.clone();
        let mut row = h_flex()
            .id(("drow", ix))
            .w_full()
            .items_start()
            .border_l_2()
            .border_color(if ix == self.cursor {
                skin.accent
            } else {
                gpui_kit::transparent_black()
            })
            .when(selected, |d| d.bg(skin.selection))
            .child(div().flex_1().min_w_0().child(inner))
            .on_click(move |ev, _, cx| {
                let shift = ev.modifiers().shift;
                let _ = me.update(cx, |d, cx| d.set_cursor(ix, shift, cx));
            });
        if header && self.source.is_working() {
            let staged = self.source == DiffSource::Staged;
            let button = |id: &'static str, label: &'static str, color: Hsla, op: PartOp| {
                let me = self.me.clone();
                div()
                    .id((id, ix))
                    .flex_none()
                    .px_2()
                    .text_xs()
                    .text_color(color)
                    .cursor_pointer()
                    .hover(|d| d.bg(skin.hover))
                    .child(label)
                    .on_click(move |_, _, cx| {
                        cx.stop_propagation();
                        let _ = me.update(cx, |d, cx| {
                            d.set_cursor(ix, false, cx);
                            d.request_hunk(op, cx);
                        });
                    })
            };
            row = row.child(
                h_flex()
                    .flex_none()
                    .gap_1()
                    .child(if staged {
                        button(
                            "hunk-unstage",
                            msg_actions::HUNK_UNSTAGE.get(),
                            skin.accent,
                            PartOp::Unstage,
                        )
                    } else {
                        button(
                            "hunk-stage",
                            msg_actions::HUNK_STAGE.get(),
                            skin.accent,
                            PartOp::Stage,
                        )
                    })
                    .when(!staged, |d| {
                        d.child(button(
                            "hunk-discard",
                            msg_actions::HUNK_DISCARD.get(),
                            skin.danger,
                            PartOp::Discard,
                        ))
                    }),
            );
        }
        row.into_any_element()
    }

    fn row_content(&self, ix: usize) -> AnyElement {
        let base = div()
            .w_full()
            .font_family(fonts::MONO_FAMILY)
            .text_size(px(TEXT_PX))
            .line_height(px(LINE_H));
        match &self.rows.lines[ix] {
            Line::Full(line) => base.child(self.half(line)).into_any_element(),
            Line::Split(left, right) => {
                let blank = || div().flex_1().min_w_0().min_h(px(LINE_H));
                let side = |l: &Option<gitpoulpi_app::diff::StyledLine>| match l {
                    Some(l) => self.half(l).flex_1().min_w_0(),
                    None => blank(),
                };
                base.child(
                    h_flex()
                        .w_full()
                        .items_stretch()
                        .child(side(left))
                        .child(
                            div()
                                .w(px(1.0))
                                .flex_none()
                                .bg(self.colors.gutter_fg)
                                .opacity(0.4),
                        )
                        .child(side(right)),
                )
                .into_any_element()
            }
        }
    }
}
