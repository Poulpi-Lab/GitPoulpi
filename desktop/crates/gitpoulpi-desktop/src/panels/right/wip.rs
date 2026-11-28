//! The right pane for the WIP row: working changes (Unstaged / Staged) over the commit form.

use gitpoulpi_i18n::msg;
use gpui_kit::component::{
    h_flex,
    input::{Input, Textarea},
    v_flex, Sizable as _,
};
use gpui_kit::{
    div, px, Context, FontWeight, InteractiveElement as _, IntoElement as _, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _,
};

use super::files::FileRow;
use super::widgets::{boxed, ListKind};
use super::RightPanel;
use crate::theme::Skin;

/// Longest summary line before git hosts start to wrap it.
const SUMMARY_BUDGET: usize = 72;
const LIST_ROW_H: f32 = 24.0;

impl RightPanel {
    /// Rows of the status entries that have a state in the working tree (`staged: false`) or in the
    /// index (`staged: true`).
    fn status_rows(&self, staged: bool) -> Vec<FileRow> {
        self.status
            .iter()
            .enumerate()
            .filter_map(|(ix, e)| {
                let status = if staged { e.index? } else { e.worktree? };
                Some(FileRow::file(
                    e.path.to_string_lossy().into_owned(),
                    status,
                    0,
                    0,
                    ix,
                ))
            })
            .collect()
    }

    pub(super) fn wip_view(&self, skin: &Skin, cx: &mut Context<Self>) -> gpui_kit::Div {
        let unstaged = self.status_rows(false);
        let staged = self.status_rows(true);
        let title = msg::status_list::title_changes(self.status.len() as u64, &self.branch)
            .trim()
            .to_string();
        let action = |label: &str| {
            div()
                .text_xs()
                .text_color(skin.accent)
                .font_weight(FontWeight::SEMIBOLD)
                .child(label.to_string())
                .into_any_element()
        };
        let height = |n: usize| px(n.clamp(1, 8) as f32 * LIST_ROW_H + 4.0);
        let unstaged_box = boxed(
            skin,
            format!(
                "{} ({})",
                msg::status_pane::UNSTAGED_TITLE.get(),
                unstaged.len()
            )
            .into(),
            Some(action(msg::status_pane::STAGE_ALL.get())),
        )
        .flex_none()
        .child(
            div()
                .h(height(unstaged.len()))
                .flex()
                .flex_col()
                .child(self.file_list(
                    unstaged.clone(),
                    &self.scroll,
                    "ufiles",
                    ListKind::Working { staged: false },
                    cx,
                    skin,
                )),
        );
        let staged_box = boxed(
            skin,
            format!(
                "{} ({})",
                msg::status_pane::STAGED_TITLE.get(),
                staged.len()
            )
            .into(),
            Some(action(msg::status_pane::UNSTAGE_ALL.get())),
        )
        .flex_none()
        .child(
            div()
                .h(height(staged.len()))
                .flex()
                .flex_col()
                .child(self.file_list(
                    staged.clone(),
                    &self.scroll_staged,
                    "sfiles",
                    ListKind::Working { staged: true },
                    cx,
                    skin,
                )),
        );
        let check = |on: bool, label: &str| {
            h_flex()
                .gap(px(6.0))
                .text_sm()
                .child(if on { "[x]" } else { "[ ]" })
                .child(label.to_string())
        };
        let can_commit = !staged.is_empty() && self.summary_len > 0;
        let button_label = if staged.is_empty() {
            msg::composer::NEED_STAGED.get().to_string()
        } else if self.summary_len == 0 {
            msg::composer::NEED_SUMMARY.get().to_string()
        } else {
            msg::composer::commit_files(staged.len() as u64)
        };
        v_flex()
            .size_full()
            .gap(px(8.0))
            .child(
                h_flex()
                    .flex_none()
                    .h(px(26.0))
                    .gap(px(8.0))
                    .items_center()
                    .child(
                        div()
                            .px(px(6.0))
                            .rounded(px(2.0))
                            .bg(skin.danger)
                            .text_color(skin.on_dark)
                            .text_xs()
                            .child("\u{2715}"),
                    )
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(title)),
            )
            .child(self.controls(skin, cx))
            .child(unstaged_box)
            .child(staged_box)
            .child(div().flex_1())
            .child(
                v_flex()
                    .flex_none()
                    .gap(px(6.0))
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                v_flex()
                                    .gap(px(2.0))
                                    .child(
                                        check(self.amend, msg::composer::AMEND.get())
                                            .id("amend")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.amend = !this.amend;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        check(self.skip_hooks, msg::composer::SKIP_HOOKS.get())
                                            .id("hooks")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.skip_hooks = !this.skip_hooks;
                                                cx.notify();
                                            })),
                                    ),
                            )
                            .child(div().text_xs().text_color(skin.muted).child(format!(
                                "#\u{25BE} {}",
                                SUMMARY_BUDGET.saturating_sub(self.summary_len)
                            ))),
                    )
                    .child(
                        v_flex()
                            .border_1()
                            .border_color(skin.border)
                            .rounded(px(3.0))
                            .p(px(4.0))
                            .gap(px(4.0))
                            .child(Input::new(&self.summary).small())
                            .child(Textarea::new(&self.description)),
                    )
                    .child(
                        div()
                            .h(px(28.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(3.0))
                            .text_sm()
                            .bg(if can_commit {
                                skin.accent
                            } else {
                                skin.input_bg
                            })
                            .text_color(if can_commit {
                                skin.on_button
                            } else {
                                skin.muted
                            })
                            .child(button_label),
                    ),
            )
    }
}
