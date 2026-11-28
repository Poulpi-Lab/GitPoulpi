//! The right pane for a selected commit or stash: details above the changed files.

use gitpoulpi_core::domain::{CommitNode, FileStatus};
use gitpoulpi_i18n::msg;
use gpui_kit::component::{h_flex, input::Input, v_flex, ActiveTheme as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{div, px, Context, FontWeight, ParentElement as _, Styled as _};

use super::widgets::{boxed, ListKind};
use super::RightPanel;
use crate::theme::Skin;
use crate::util::dates;

/// Files added, modified and deleted among a commit's diffs.
fn change_counts(files: &[gitpoulpi_core::domain::FileDiff]) -> (u64, u64, u64) {
    let (mut added, mut modified, mut deleted) = (0u64, 0u64, 0u64);
    for f in files {
        match f.status {
            FileStatus::Added | FileStatus::Untracked => added += 1,
            FileStatus::Deleted => deleted += 1,
            _ => modified += 1,
        }
    }
    (added, modified, deleted)
}

/// The message split into its summary line and the body below it.
fn split_message(message: &str) -> (String, String) {
    match message.split_once('\n') {
        Some((s, b)) => (
            s.to_string(),
            b.trim_start_matches('\n').trim_end().to_string(),
        ),
        None => (message.to_string(), String::new()),
    }
}

impl RightPanel {
    pub(super) fn commit_view(
        &self,
        node: &CommitNode,
        stash: &Option<String>,
        skin: &Skin,
        cx: &mut Context<Self>,
    ) -> gpui_kit::Div {
        let (added, modified, deleted) = change_counts(&self.files);
        let (y, m, d, hh, mm) = dates::civil(node.timestamp);
        let date = format!(
            "{} {:02}:{:02}",
            msg::date::day_month_year(
                d.to_string(),
                gitpoulpi_i18n::month_name_of(m).get(),
                format_args!("{y:04}")
            ),
            hh,
            mm
        );
        let parent = node.parents.first().map(|p| p.to_hex()[..7].to_string());
        let mono = cx.theme().mono_font_family.clone();
        let (summary, body) = split_message(&node.message);
        let details = boxed(skin, msg::commit_detail::TITLE.get().into(), None).child(
            v_flex()
                .px(px(8.0))
                .pb(px(8.0))
                .gap(px(3.0))
                .child(
                    h_flex()
                        .gap(px(6.0))
                        .child(
                            div()
                                .text_color(skin.muted)
                                .child(msg::commit_detail::header("").trim_end().to_string()),
                        )
                        .child(
                            div().text_color(skin.accent).font_family(mono).child(
                                stash
                                    .clone()
                                    .unwrap_or_else(|| node.id.to_hex()[..7].to_string()),
                            ),
                        ),
                )
                .child(
                    h_flex()
                        .child(div().child(node.author_name.clone()))
                        .child(div().flex_1())
                        .when_some(parent, |d, p| {
                            d.child(
                                div()
                                    .text_color(skin.muted)
                                    .child(msg::commit_detail::parent(p)),
                            )
                        }),
                )
                .child(
                    h_flex()
                        .gap(px(6.0))
                        .child(
                            div()
                                .text_color(skin.muted)
                                .child(msg::commit_detail::AUTHORED.get()),
                        )
                        .child(div().font_weight(FontWeight::SEMIBOLD).child(date)),
                )
                .child(
                    h_flex()
                        .gap(px(12.0))
                        .text_xs()
                        .when(added > 0, |d| {
                            d.child(
                                div()
                                    .text_color(skin.success)
                                    .child(format!("+ {}", msg::commit_detail::added(added))),
                            )
                        })
                        .when(modified > 0, |d| {
                            d.child(div().text_color(skin.warning).child(format!(
                                "\u{270E} {}",
                                msg::commit_detail::modified(modified)
                            )))
                        })
                        .when(deleted > 0, |d| {
                            d.child(div().text_color(skin.danger).child(format!(
                                "\u{2212} {}",
                                msg::commit_detail::deleted(deleted)
                            )))
                        }),
                )
                .child(
                    v_flex()
                        .mt(px(6.0))
                        .p(px(8.0))
                        .gap(px(4.0))
                        .border_1()
                        .border_color(skin.border)
                        .rounded(px(3.0))
                        .child(div().font_weight(FontWeight::BOLD).child(summary))
                        .when(!body.is_empty(), |d| d.child(div().text_sm().child(body))),
                ),
        );
        let list = self.file_list(
            self.rows.clone(),
            &self.scroll,
            "cfiles",
            ListKind::Commit,
            cx,
            skin,
        );
        let files_box = boxed(
            skin,
            msg::commit_detail::files_title(
                msg::commit_detail::CHANGED_FILES.get(),
                self.rows.len() as u64,
            )
            .into(),
            None,
        )
        .flex_1()
        .min_h_0()
        .child(self.controls(skin, cx))
        .child(
            h_flex()
                .flex_none()
                .px(px(8.0))
                .pb(px(4.0))
                .gap(px(6.0))
                .items_center()
                .child(
                    div()
                        .text_xs()
                        .text_color(skin.muted)
                        .child(msg::commit_detail::FILTER_FILES.get()),
                )
                .child(
                    div()
                        .flex_1()
                        .child(Input::new(&self.filter).small().cleanable(true)),
                ),
        )
        .child(list);
        v_flex()
            .size_full()
            .gap(px(8.0))
            .child(details.flex_none())
            .child(files_box)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_splits_into_summary_and_body() {
        assert_eq!(split_message("one"), ("one".to_string(), String::new()));
        assert_eq!(
            split_message("one\n\ntwo\nthree\n"),
            ("one".to_string(), "two\nthree".to_string())
        );
    }
}
