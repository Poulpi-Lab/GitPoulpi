//! Building blocks of the right pane: boxed sections, the file list and its controls.

use gitpoulpi_i18n::msg;
use gpui_kit::assets::IconName as AssetIcon;
use gpui_kit::component::{h_flex, scroll::Scrollbar, v_flex, Icon};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, uniform_list, Context, FontWeight, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _,
    UniformListScrollHandle,
};

use super::files::{status_glyph, FileRow};
use super::{Mode, RightPanel};
use crate::strings;
use crate::theme::Skin;

/// Which list a row belongs to.
#[derive(Clone, Copy)]
pub(super) enum ListKind {
    /// Files of the selected commit.
    Commit,
    /// Working tree changes, unstaged or staged.
    Working { staged: bool },
}

pub(super) fn boxed(
    skin: &Skin,
    title: SharedString,
    right: Option<gpui_kit::AnyElement>,
) -> gpui_kit::Div {
    let head = h_flex()
        .flex_none()
        .h(px(26.0))
        .px(px(8.0))
        .items_center()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(skin.text)
                .child(title),
        )
        .child(div().flex_1())
        .when_some(right, |d, r| d.child(r));
    v_flex()
        .w_full()
        .border_1()
        .border_color(skin.border)
        .rounded(px(3.0))
        .child(head)
}

fn segment(label: String, on: bool, skin: &Skin) -> gpui_kit::Div {
    div()
        .px(px(8.0))
        .h(px(20.0))
        .flex()
        .items_center()
        .text_xs()
        .rounded(px(2.0))
        .when(on, |d| d.bg(skin.accent).text_color(skin.on_button))
        .when(!on, |d| d.text_color(skin.text))
        .child(label)
}

impl RightPanel {
    pub(super) fn file_list(
        &self,
        rows: Vec<FileRow>,
        scroll: &UniformListScrollHandle,
        id: &'static str,
        kind: ListKind,
        cx: &mut Context<Self>,
        skin: &Skin,
    ) -> gpui_kit::Div {
        let (success, warning, danger, info, muted, text, selection, hover) = (
            skin.success,
            skin.warning,
            skin.danger,
            skin.info,
            skin.muted,
            skin.text,
            skin.selection,
            skin.hover,
        );
        let selected = self.selected;
        let me = cx.entity().downgrade();
        let n = rows.len();
        let commit_list = matches!(kind, ListKind::Commit);
        div()
            .relative()
            .flex_1()
            .min_h_0()
            .child(
                uniform_list(id, n, move |range, _w, _cx| {
                    range
                        .map(|i| {
                            let r = rows[i].clone();
                            let (glyph, class) = status_glyph(r.status);
                            let color: Hsla = [success, warning, danger, info][class as usize];
                            let me = me.clone();
                            let path = r.path.clone();
                            let is_selected = selected == Some(r.ix) && !r.folder && commit_list;
                            h_flex()
                                .id((id, i))
                                .w_full()
                                .h(px(24.0))
                                .px(px(8.0))
                                .pl(px(8.0 + r.depth as f32 * 14.0))
                                .gap(px(6.0))
                                .items_center()
                                .when(is_selected, |d| d.bg(selection))
                                .hover(|d| d.bg(hover))
                                .child(if r.folder {
                                    Icon::new(AssetIcon::FolderOpen)
                                        .size(px(14.0))
                                        .text_color(warning)
                                        .into_any_element()
                                } else {
                                    div()
                                        .w(px(14.0))
                                        .flex_none()
                                        .text_color(color)
                                        .font_weight(FontWeight::BOLD)
                                        .child(glyph)
                                        .into_any_element()
                                })
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .truncate()
                                        .text_sm()
                                        .text_color(text)
                                        .child(r.label()),
                                )
                                .when(!r.folder && commit_list, |d| {
                                    d.child(
                                        div()
                                            .flex_none()
                                            .text_xs()
                                            .text_color(success)
                                            .child(format!("+{}", r.adds)),
                                    )
                                    .child(
                                        div()
                                            .flex_none()
                                            .text_xs()
                                            .text_color(danger)
                                            .child(format!("-{}", r.dels)),
                                    )
                                })
                                .when(r.folder, |d| d.text_color(muted))
                                .on_click(move |_, _, cx| {
                                    if r.folder {
                                        return;
                                    }
                                    let _ = me.update(cx, |this, cx| match kind {
                                        ListKind::Commit => this.open_commit_file(r.ix, cx),
                                        ListKind::Working { staged } => {
                                            this.open_working(path.clone(), staged, cx)
                                        }
                                    });
                                })
                                .into_any_element()
                        })
                        .collect()
                })
                .size_full()
                .track_scroll(scroll),
            )
            .child(Scrollbar::vertical(scroll))
    }

    /// Sort toggle, Path / Tree switch and "View all files".
    pub(super) fn controls(&self, skin: &Skin, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .flex_none()
            .h(px(28.0))
            .px(px(8.0))
            .gap(px(10.0))
            .items_center()
            .text_xs()
            .child(
                div()
                    .id("sort")
                    .text_color(skin.muted)
                    .child(strings::sort_label(self.sort_az))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.sort_az = !this.sort_az;
                        this.rebuild_rows(cx);
                    })),
            )
            .child(
                h_flex()
                    .gap(px(2.0))
                    .child(
                        segment(
                            msg::commit_detail::MODE_PATH.get().to_string(),
                            !self.tree,
                            skin,
                        )
                        .id("m-path")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.tree = false;
                            this.rebuild_rows(cx);
                        })),
                    )
                    .child(
                        segment(
                            format!("{} [m]", msg::commit_detail::MODE_TREE.get()),
                            self.tree,
                            skin,
                        )
                        .id("m-tree")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.tree = true;
                            this.rebuild_rows(cx);
                        })),
                    ),
            )
            .when(matches!(self.mode, Mode::Wip), |d| {
                d.child(
                    div()
                        .text_color(skin.accent)
                        .child(msg::button_bar::COLLAPSE_ALL.get()),
                )
            })
            .child(div().flex_1())
            .child(
                h_flex()
                    .id("view-all")
                    .gap(px(4.0))
                    .text_color(skin.text)
                    .child(if self.view_all { "[x]" } else { "[ ]" })
                    .child(format!(
                        "{} [a]",
                        msg::commit_detail::VIEW_ALL_FILES
                            .get()
                            .trim_end_matches(char::is_whitespace)
                    ))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.view_all = !this.view_all;
                        cx.notify();
                    })),
            )
    }
}
