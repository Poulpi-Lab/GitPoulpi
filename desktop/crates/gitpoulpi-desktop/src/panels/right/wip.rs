//! The right pane for the WIP row: the working changes (Unstaged / Staged Files) over the commit form.
//!
//! Each list is a section box with its title, count and bulk buttons. A row stands for one file (or a
//! folder in Tree mode): click selects (Ctrl toggles, Shift takes a range), a double click opens the
//! diff, the hover button stages or unstages it, and a row can be dragged onto the other box. The
//! keyboard does all of it too (see `stage`).

use std::sync::Arc;

use gitpoulpi_app::palette::vocab::status_pane::StatusSection;
use gitpoulpi_i18n::msg;
use gpui_kit::assets::IconName as AssetIcon;
use gpui_kit::component::{h_flex, scroll::Scrollbar, v_flex, Icon};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, uniform_list, AppContext as _, Context, FontWeight, Hsla, InteractiveElement as _,
    IntoElement as _, ParentElement as _, Render, SharedString, StatefulInteractiveElement as _,
    Styled as _, UniformListScrollHandle, Window,
};

use super::files::{status_glyph, FileRow};
use super::stage::DraggedFiles;
use super::widgets::boxed;
use super::RightPanel;
use crate::theme::Skin;

const LIST_ROW_H: f32 = 24.0;

/// What follows the pointer while files are dragged.
struct DragGhost(SharedString);

impl Render for DragGhost {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui_kit::IntoElement {
        let skin = cx.global::<Skin>().clone();
        div()
            .px_2()
            .py_1()
            .rounded(px(3.0))
            .bg(skin.selection)
            .border_1()
            .border_color(skin.accent)
            .text_sm()
            .text_color(skin.text)
            .child(self.0.clone())
    }
}

/// One row as the list paints it.
#[derive(Clone)]
struct RowView {
    row: FileRow,
    selected: bool,
    cursor: bool,
    /// What a drag that starts on this row carries: the selection when the row is in it, else the row.
    drag: Arc<DraggedFiles>,
}

impl RightPanel {
    fn section_title(&self, section: StatusSection, n: usize) -> String {
        format!("{} ({n})", section.title())
    }

    fn row_views(
        &self,
        section: StatusSection,
        rows: Vec<FileRow>,
        files: &[String],
    ) -> Vec<RowView> {
        rows.into_iter()
            .map(|row| {
                let selected = self.selection.contains(section, &row);
                let paths = if selected {
                    self.selection
                        .paths(section)
                        .iter()
                        .map(|p| p.to_string_lossy().into_owned())
                        .collect()
                } else if row.folder {
                    let prefix = format!("{}/", row.path);
                    files
                        .iter()
                        .filter(|f| f.starts_with(&prefix))
                        .cloned()
                        .collect()
                } else {
                    vec![row.path.clone()]
                };
                RowView {
                    selected,
                    cursor: self.selection.is_cursor(section, &row),
                    drag: Arc::new(DraggedFiles {
                        from: section,
                        paths,
                    }),
                    row,
                }
            })
            .collect()
    }

    /// The list of one section.
    fn status_list(
        &self,
        section: StatusSection,
        views: Vec<RowView>,
        scroll: &UniformListScrollHandle,
        cx: &mut Context<Self>,
        skin: &Skin,
    ) -> gpui_kit::Div {
        let (success, warning, danger, info, muted, text, selection, hover, accent) = (
            skin.success,
            skin.warning,
            skin.danger,
            skin.info,
            skin.muted,
            skin.text,
            skin.selection,
            skin.hover,
            skin.accent,
        );
        let me = cx.entity().downgrade();
        let focus = self.focus.clone();
        let n = views.len();
        let id: &'static str = match section {
            StatusSection::Unstaged => "ufiles",
            StatusSection::Staged => "sfiles",
        };
        div()
            .relative()
            .flex_1()
            .min_h_0()
            .child(
                uniform_list(id, n, move |range, _w, _cx| {
                    range
                        .map(|i| {
                            let v = views[i].clone();
                            let r = v.row.clone();
                            let (glyph, class) = status_glyph(r.status);
                            let color: Hsla = [success, warning, danger, info][class as usize];
                            let (me_click, me_button, me_menu) =
                                (me.clone(), me.clone(), me.clone());
                            let menu_path = r.path.clone();
                            let (focus, paths) = (focus.clone(), v.drag.paths.clone());
                            let button_label = section.row_label();
                            h_flex()
                                .id((id, i))
                                .group("wip-row")
                                .w_full()
                                .h(px(LIST_ROW_H))
                                .px(px(8.0))
                                .pl(px(8.0 + r.depth as f32 * 14.0))
                                .gap(px(6.0))
                                .items_center()
                                .cursor_pointer()
                                .when(v.selected, |d| d.bg(selection))
                                .when(v.cursor, |d| d.border_l_2().border_color(accent))
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
                                        .text_color(if r.folder { muted } else { text })
                                        .child(r.label()),
                                )
                                .child(
                                    div()
                                        .id((id, i + 1_000_000))
                                        .flex_none()
                                        .invisible()
                                        .group_hover("wip-row", |s| s.visible())
                                        .px_1()
                                        .rounded(px(2.0))
                                        .text_xs()
                                        .text_color(accent)
                                        .cursor_pointer()
                                        .hover(|d| d.bg(hover))
                                        .child(button_label)
                                        .on_click(move |_, _, cx| {
                                            cx.stop_propagation();
                                            let paths = paths.iter().map(Into::into).collect();
                                            let _ = me_button.update(cx, |this, cx| {
                                                this.stage_paths(
                                                    paths,
                                                    section == StatusSection::Unstaged,
                                                    cx,
                                                )
                                            });
                                        }),
                                )
                                .on_drag(v.drag.clone(), {
                                    let label = if v.drag.paths.len() > 1 {
                                        format!("{} files", v.drag.paths.len())
                                    } else {
                                        r.label()
                                    };
                                    move |_, _, _, cx| {
                                        let label = SharedString::from(label.clone());
                                        cx.new(|_| DragGhost(label))
                                    }
                                })
                                .on_mouse_down(gpui_kit::MouseButton::Right, {
                                    let folder = r.folder;
                                    move |ev, _, cx| {
                                        if folder {
                                            return;
                                        }
                                        cx.stop_propagation();
                                        let (path, at) = (menu_path.clone(), ev.position);
                                        let side = match section {
                                            StatusSection::Unstaged => {
                                                crate::ref_target::FileSide::Unstaged
                                            }
                                            StatusSection::Staged => {
                                                crate::ref_target::FileSide::Staged
                                            }
                                        };
                                        let _ = me_menu.update(cx, |this, cx| {
                                            this.file_row_menu(path, side, at, cx)
                                        });
                                    }
                                })
                                .on_click(move |ev, window, cx| {
                                    focus.focus(window, cx);
                                    let mods = ev.modifiers();
                                    let ix = i;
                                    let _ = me_click.update(cx, |this, cx| {
                                        this.click_row(section, ix, mods, cx);
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

    /// One section box: title with count, bulk buttons, the list, and the drop target of the other list.
    fn section_box(
        &self,
        section: StatusSection,
        scroll: &UniformListScrollHandle,
        cx: &mut Context<Self>,
        skin: &Skin,
    ) -> gpui_kit::Div {
        let (rows, files) = self.section_view(section);
        let count = files.len();
        let height = px(rows.len().clamp(1, 8) as f32 * LIST_ROW_H + 4.0);
        let views = self.row_views(section, rows, &files);
        let button = |id: &'static str, label: &str, color: Hsla| {
            div()
                .id(id)
                .px_1()
                .rounded(px(2.0))
                .text_xs()
                .text_color(color)
                .font_weight(FontWeight::SEMIBOLD)
                .cursor_pointer()
                .hover(|d| d.bg(skin.hover))
                .child(label.to_string())
        };
        let actions = match section {
            StatusSection::Unstaged => h_flex()
                .gap_2()
                .child(
                    button("stage-all", section.bulk_label(), skin.accent)
                        .on_click(cx.listener(|this, _, _, cx| this.stage_all(cx))),
                )
                .child(
                    button(
                        "discard-all",
                        msg::status_list::DISCARD_ALL.get(),
                        skin.danger,
                    )
                    .on_click(cx.listener(|this, _, window, cx| this.discard_all(window, cx))),
                ),
            StatusSection::Staged => h_flex().gap_2().child(
                button("unstage-all", section.bulk_label(), skin.accent)
                    .on_click(cx.listener(|this, _, _, cx| this.unstage_all(cx))),
            ),
        };
        let accent = skin.accent;
        boxed(
            skin,
            self.section_title(section, count).into(),
            Some(actions.into_any_element()),
        )
        .flex_none()
        .can_drop(move |d, _, _| {
            d.downcast_ref::<Arc<DraggedFiles>>()
                .is_some_and(|d| d.from != section)
        })
        .drag_over::<Arc<DraggedFiles>>(move |s, _, _, _| s.border_color(accent))
        .on_drop(
            cx.listener(move |this, d: &Arc<DraggedFiles>, _, cx| this.drop_files(section, d, cx)),
        )
        .child(
            div()
                .h(height)
                .flex()
                .flex_col()
                .child(self.status_list(section, views, scroll, cx, skin)),
        )
    }

    pub(super) fn wip_view(
        &self,
        _window: &mut Window,
        skin: &Skin,
        cx: &mut Context<Self>,
    ) -> gpui_kit::Div {
        let conflicts = self.conflict_count();
        let n = self.status.len() as u64;
        let title = if conflicts > 0 {
            msg::status_list::title_changes_conflicted(n, &self.branch, conflicts as u64)
        } else {
            msg::status_list::title_changes(n, &self.branch)
        }
        .trim()
        .to_string();
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
            .child(self.section_box(StatusSection::Unstaged, &self.scroll, cx, skin))
            .child(self.section_box(StatusSection::Staged, &self.scroll_staged, cx, skin))
            .child(div().flex_1())
            .child(self.composer_view(skin, cx))
    }
}
