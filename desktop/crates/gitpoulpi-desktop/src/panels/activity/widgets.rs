//! The few controls the panels of this area share: a button chip and a tab chip.

use gpui_kit::component::h_flex;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, ElementId, InteractiveElement as _, ParentElement as _, SharedString, Stateful,
    Styled as _,
};

use crate::theme::Skin;

/// A bordered button with an optional key hint (`[u]`); dimmed and inert when `enabled` is off, red when
/// `danger`. The caller adds `.on_click`.
pub fn chip(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    key: &str,
    enabled: bool,
    danger: bool,
    skin: &Skin,
) -> Stateful<gpui_kit::Div> {
    let color = if !enabled {
        skin.muted
    } else if danger {
        skin.danger
    } else {
        skin.text
    };
    let key = key.to_string();
    h_flex()
        .id(id)
        .h(px(26.0))
        .px_2()
        .gap_1()
        .items_center()
        .rounded(px(3.0))
        .border_1()
        .border_color(skin.border)
        .text_sm()
        .text_color(color)
        .when(enabled, |d| d.cursor_pointer().hover(|d| d.bg(skin.hover)))
        .child(div().child(label.into()))
        .when(!key.is_empty(), |d| {
            d.child(
                div()
                    .text_xs()
                    .text_color(skin.muted)
                    .child(format!("[{key}]")),
            )
        })
}

/// A tab of a panel: the active one is highlighted.
pub fn tab_chip(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    active: bool,
    skin: &Skin,
) -> Stateful<gpui_kit::Div> {
    h_flex()
        .id(id)
        .h(px(26.0))
        .px_3()
        .items_center()
        .rounded(px(3.0))
        .text_sm()
        .cursor_pointer()
        .when(active, |d| {
            d.bg(skin.selection)
                .text_color(skin.text)
                .border_b_2()
                .border_color(skin.accent)
        })
        .when(!active, |d| {
            d.text_color(skin.muted).hover(|d| d.bg(skin.hover))
        })
        .child(label.into())
}
