//! The right pane while two or more Log rows are selected: a header with the two ends and a Swap
//! button, one card per selected row (a click moves the Log cursor to that row), then the usual changed
//! files list of what differs between the oldest and the newest.

use gitpoulpi_i18n::msg;
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, Context, FontWeight, InteractiveElement as _, IntoElement as _, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _,
};

use super::widgets::boxed;
use super::{Mode, RightEvent, RightPanel};
use crate::panels::log::compare::{CardKind, CompareSpec, EndMark};
use crate::theme::Skin;
use crate::util::dates;

impl RightPanel {
    /// Shows the comparison of the selected rows.
    pub fn show_compare(&mut self, spec: CompareSpec, cx: &mut Context<Self>) {
        self.selected = None;
        let fut = self.backend.target_diff(spec.target.clone());
        self.load_with(Mode::Compare(spec), fut, cx);
    }

    /// The comparison on show, if the pane is comparing.
    pub fn compare_spec(&self) -> Option<&CompareSpec> {
        match &self.mode {
            Mode::Compare(spec) => Some(spec),
            _ => None,
        }
    }

    pub(super) fn compare_view(
        &self,
        spec: &CompareSpec,
        skin: &Skin,
        cx: &mut Context<Self>,
    ) -> gpui_kit::Div {
        let now = dates::now_secs();
        let swappable = spec.can_swap;
        let swap = div()
            .id("compare-swap")
            .px(px(8.0))
            .h(px(20.0))
            .flex()
            .items_center()
            .rounded(px(2.0))
            .text_xs()
            .border_1()
            .border_color(skin.border)
            .when(swappable, |d| {
                d.cursor_pointer()
                    .text_color(skin.accent)
                    .hover(|d| d.bg(skin.hover))
            })
            .when(!swappable, |d| d.text_color(skin.muted))
            .child(msg::compare::SWAP.get())
            .on_click(cx.listener(|_, _, _, cx| cx.emit(RightEvent::SwapCompare)));
        let cards = spec.cards.iter().map(|c| {
            let row = c.row;
            let tag = c.mark.map(|m| match m {
                EndMark::From => msg::compare::FROM_TAG.get(),
                EndMark::To => msg::compare::TO_TAG.get(),
            });
            let age = match c.kind {
                CardKind::Wip => String::new(),
                _ => dates::relative_date(c.timestamp, now),
            };
            v_flex()
                .id(("compare-card", row))
                .px(px(8.0))
                .py(px(3.0))
                .gap(px(1.0))
                .border_b_1()
                .border_color(skin.border)
                .cursor_pointer()
                .hover(|d| d.bg(skin.hover))
                .child(
                    h_flex()
                        .gap(px(6.0))
                        .items_center()
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .truncate()
                                .child(c.who.clone()),
                        )
                        .child(div().text_xs().text_color(skin.muted).child(age))
                        .child(div().flex_1())
                        .when_some(tag, |d, t| {
                            d.child(
                                div()
                                    .px(px(5.0))
                                    .text_xs()
                                    .rounded(px(2.0))
                                    .bg(skin.accent)
                                    .text_color(skin.on_button)
                                    .child(t),
                            )
                        }),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(skin.muted)
                        .truncate()
                        .child(match c.id {
                            Some(id) if c.kind == CardKind::Commit => {
                                format!("{} {}", crate::daemon::commit_ops::short(id), c.summary)
                            }
                            _ => c.summary.clone(),
                        }),
                )
                .on_click(cx.listener(move |_, _, _, cx| cx.emit(RightEvent::GoToRow(row))))
        });
        let header = boxed(
            skin,
            msg::compare::TITLE.get().into(),
            Some(swap.into_any_element()),
        )
        .child(
            v_flex()
                .px(px(8.0))
                .pb(px(6.0))
                .gap(px(3.0))
                .child(
                    div()
                        .truncate()
                        .child(msg::compare::from_line(&spec.from_label)),
                )
                .child(
                    div()
                        .truncate()
                        .child(msg::compare::to_line(&spec.to_label)),
                )
                .when(spec.rows > 2, |d| {
                    d.child(
                        div()
                            .text_xs()
                            .text_color(skin.muted)
                            .child(msg::compare::range_note(spec.rows as u64)),
                    )
                })
                .child(
                    v_flex()
                        .id("compare-cards")
                        .max_h(px(180.0))
                        .overflow_y_scroll()
                        .border_1()
                        .border_color(skin.border)
                        .rounded(px(3.0))
                        .children(cards),
                ),
        );
        self.changed_files(header, skin, cx)
    }
}
