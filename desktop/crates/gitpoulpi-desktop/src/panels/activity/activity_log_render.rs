//! Painting the Activity Log and taking its keys.

use gitpoulpi_i18n::msg::activity_log as m;
use gitpoulpi_log::Level;
use gpui_kit::component::{h_flex, v_flex, WindowExt as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, uniform_list, AnyElement, App, AppContext as _, Context, Entity, Focusable,
    InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window,
};

use super::activity_log::{current, ActivityView, OpenLog};
use super::log_model::{level_name, Levels, LogTab, Scope};
use super::widgets::{chip, tab_chip};
use crate::theme::Skin;

/// How far one Left / Right press scrolls the lines sideways.
const HSTEP: f32 = 64.0;

impl Focusable for ActivityView {
    fn focus_handle(&self, _: &App) -> gpui_kit::FocusHandle {
        self.focus.clone()
    }
}

impl ActivityView {
    /// A key typed in the panel.
    pub fn key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let ks = &ev.keystroke;
        if ks.modifiers.alt || ks.modifiers.platform {
            return;
        }
        if ks.modifiers.control {
            if ks.key == "p" {
                window.defer(cx, crate::commands::palette::open);
            }
            return;
        }
        match ks.key.as_str() {
            "up" => self.move_selection(-1, cx),
            "down" => self.move_selection(1, cx),
            "pageup" => self.move_selection(-8, cx),
            "pagedown" => self.move_selection(8, cx),
            "home" => self.select(0, cx),
            "end" => self.select(usize::MAX, cx),
            "left" => self.scroll_by(-HSTEP, cx),
            "right" => self.scroll_by(HSTEP, cx),
            "escape" => window.close_dialog(cx),
            "tab" => {
                let next = match self.tab {
                    LogTab::Git => LogTab::App,
                    LogTab::App => LogTab::Git,
                };
                self.set_tab(next, cx);
            }
            "enter" => self.show_operation(cx),
            "1" => self.toggle_level(Level::Error, cx),
            "2" => self.toggle_level(Level::Warn, cx),
            "3" => self.toggle_level(Level::Info, cx),
            "4" => self.toggle_level(Level::Debug, cx),
            "5" => self.toggle_level(Level::Trace, cx),
            "a" => self.set_levels(Levels::all(), cx),
            "n" => self.set_levels(Levels::none(), cx),
            "c" => self.copy_selected(cx),
            "o" => self.open_folder(cx),
            "s" => self.cycle_scope(cx),
            "g" => self.set_tab(LogTab::Git, cx),
            "r" => self.read_app(cx),
            "x" if self.tab == LogTab::App => self.ask_clear(cx),
            "]" => self.step_problem(true, cx),
            "[" => self.step_problem(false, cx),
            _ => {}
        }
    }

    fn render_line(&self, ix: usize, skin: &Skin, cx: &mut Context<Self>) -> AnyElement {
        let level = self.level_of(ix);
        let color = match level {
            Level::Error => skin.danger,
            Level::Warn => skin.warning,
            Level::Info => skin.text,
            Level::Debug | Level::Trace => skin.muted,
        };
        let text = self.line(ix);
        h_flex()
            .id(("activity-line", ix))
            .h(px(22.0))
            .px_3()
            .items_center()
            .overflow_hidden()
            .text_sm()
            .text_color(color)
            .when(self.selected == ix, |d| d.bg(skin.selection))
            .cursor_pointer()
            .hover(|d| d.bg(skin.hover))
            .child(
                div()
                    .flex_none()
                    .whitespace_nowrap()
                    .ml(px(-self.hscroll))
                    .child(SharedString::from(text)),
            )
            .on_click(cx.listener(move |this, _, _, cx| this.select(ix, cx)))
            .into_any_element()
    }
}

impl Render for ActivityView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let tabs = h_flex()
            .gap_1()
            .child(
                tab_chip(
                    "log-tab-git",
                    m::TAB_GIT.get(),
                    self.tab == LogTab::Git,
                    &skin,
                )
                .on_click(cx.listener(|this, _, _, cx| this.set_tab(LogTab::Git, cx))),
            )
            .child(
                tab_chip(
                    "log-tab-app",
                    m::TAB_APPLICATION.get(),
                    self.tab == LogTab::App,
                    &skin,
                )
                .on_click(cx.listener(|this, _, _, cx| this.set_tab(LogTab::App, cx))),
            );
        let name = self
            .backend
            .repo_root()
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let scope_text = match self.scope {
            Scope::ThisRepo => m::scope_this(name),
            Scope::All => m::SCOPE_ALL.get().to_string(),
        };
        let levels = h_flex()
            .gap_1()
            .items_center()
            .child(
                div()
                    .text_sm()
                    .text_color(skin.muted)
                    .child(m::SHOW_LABEL.get()),
            )
            .children(Level::ALL.into_iter().enumerate().map(|(i, level)| {
                let on = self.levels.shows(level);
                chip(
                    ("log-level", i),
                    format!(
                        "{} {}",
                        if on { "\u{2611}" } else { "\u{2610}" },
                        level_name(level)
                    ),
                    &(i + 1).to_string(),
                    true,
                    false,
                    &skin,
                )
                .on_click(cx.listener(move |this, _, _, cx| this.toggle_level(level, cx)))
            }))
            .child(
                chip("log-all", m::SHOW_ALL.get(), "a", true, false, &skin)
                    .on_click(cx.listener(|this, _, _, cx| this.set_levels(Levels::all(), cx))),
            )
            .child(
                chip("log-none", m::SHOW_NONE.get(), "n", true, false, &skin)
                    .on_click(cx.listener(|this, _, _, cx| this.set_levels(Levels::none(), cx))),
            );
        let count = self.len();
        let list: AnyElement = if count == 0 {
            div()
                .flex_1()
                .p_3()
                .text_sm()
                .text_color(skin.muted)
                .child(match self.tab {
                    LogTab::Git => m::GIT_EMPTY.get(),
                    LogTab::App => m::APP_EMPTY.get(),
                })
                .into_any_element()
        } else {
            uniform_list(
                "activity-list",
                count,
                cx.processor(|this, range: std::ops::Range<usize>, _window, cx| {
                    let skin = cx.global::<Skin>().clone();
                    range
                        .map(|ix| this.render_line(ix, &skin, cx))
                        .collect::<Vec<_>>()
                }),
            )
            .track_scroll(&self.scroll)
            .flex_1()
            .min_h_0()
            .into_any_element()
        };
        let detail = self.detail_lines();
        let hint = match self.tab {
            LogTab::Git => m::HINT_GIT.get(),
            LogTab::App => m::HINT_APPLICATION.get(),
        };
        v_flex()
            .key_context("GpActivity")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, ev, window, cx| this.key(ev, window, cx)))
            .w_full()
            .h(px(560.0))
            .child(
                div()
                    .px_3()
                    .pt_2()
                    .pr(px(40.0))
                    .text_sm()
                    .text_color(skin.muted)
                    .child(m::TITLE.get()),
            )
            .child(
                h_flex()
                    .px_2()
                    .py_1()
                    .gap_3()
                    .items_center()
                    .flex_wrap()
                    .child(tabs)
                    .child(
                        chip(
                            "log-scope",
                            format!("{} {scope_text}", m::SCOPE_LABEL.get()),
                            "s",
                            true,
                            false,
                            &skin,
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.cycle_scope(cx))),
                    )
                    .child(levels),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .border_t_1()
                    .border_b_1()
                    .border_color(skin.border)
                    .child(list),
            )
            .child(
                v_flex()
                    .px_3()
                    .py_1()
                    .h(px(86.0))
                    .overflow_hidden()
                    .text_xs()
                    .text_color(skin.muted)
                    .children(
                        detail
                            .into_iter()
                            .take(6)
                            .map(|l| div().child(SharedString::from(l))),
                    )
                    .when_some(self.note.clone(), |d, text| {
                        d.child(div().text_color(skin.text).child(SharedString::from(text)))
                    }),
            )
            .child(
                h_flex()
                    .px_2()
                    .py_2()
                    .gap_2()
                    .flex_wrap()
                    .child(
                        chip("log-copy", m::COPY.get(), "c", count > 0, false, &skin)
                            .on_click(cx.listener(|this, _, _, cx| this.copy_selected(cx))),
                    )
                    .child(
                        chip(
                            "log-copy-all",
                            m::COPY_ALL.get(),
                            "",
                            count > 0,
                            false,
                            &skin,
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.copy_all(cx))),
                    )
                    .child(
                        chip("log-folder", m::OPEN_FOLDER.get(), "o", true, false, &skin)
                            .on_click(cx.listener(|this, _, _, cx| this.open_folder(cx))),
                    )
                    .when(self.tab == LogTab::App, |d| {
                        d.child(
                            chip("log-clear", m::CLEAR_LOG.get(), "x", true, true, &skin)
                                .on_click(cx.listener(|this, _, _, cx| this.ask_clear(cx))),
                        )
                    })
                    .child(div().flex_1())
                    .child(
                        chip("log-close", msg_close(), "", true, false, &skin)
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    ),
            )
            .child(
                div()
                    .px_3()
                    .pb_2()
                    .text_xs()
                    .text_color(skin.muted)
                    .child(hint),
            )
    }
}

fn msg_close() -> &'static str {
    gitpoulpi_i18n::msg::undo_safety::BTN_CLOSE.get()
}

/// Shows the Activity Log on `tab` in a dialog; the open one is reused.
pub fn open(
    tab: LogTab,
    scope: Scope,
    window: &mut Window,
    cx: &mut App,
) -> Option<Entity<ActivityView>> {
    if let Some(existing) = current(cx) {
        existing.update(cx, |v, cx| {
            v.set_tab(tab, cx);
            if v.scope != scope {
                v.cycle_scope(cx);
            }
        });
        return Some(existing);
    }
    let backend = crate::app::workspace_of(cx)?.read(cx).backend().clone();
    let view = cx.new(|cx| ActivityView::new(backend, tab, scope, window, cx));
    cx.set_global(OpenLog(view.downgrade()));
    window.open_dialog(cx, {
        let view = view.clone();
        move |dialog, _, _| {
            let view = view.clone();
            dialog
                .close_button(true)
                .overlay_closable(true)
                .w(px(920.0))
                .p_0()
                .content(move |content, _, _| content.child(view.clone()))
        }
    });
    let focus = view.clone();
    window.defer(cx, move |window, cx| {
        let handle = focus.read(cx).focus.clone();
        handle.focus(window, cx);
    });
    Some(view)
}
