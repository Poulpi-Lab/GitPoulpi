//! Painting the undo-safety panel and taking its keys.

use gitpoulpi_core::domain::{short, JournalState};
use gitpoulpi_i18n::msg::undo_safety as m;
use gitpoulpi_link::undo_label as label;
use gpui_kit::component::{h_flex, v_flex, WindowExt as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, uniform_list, AnyElement, App, AppContext as _, Context, Entity, Focusable,
    InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window,
};

use super::safety::{current, OpenSafety, SafetyView};
use super::safety_model::{
    age, day_heading, history_detail, row_state, tag, Act, Line, RowState, Tab,
};
use super::widgets::{chip, tab_chip};
use crate::theme::Skin;

const ROW_H: f32 = 24.0;

impl Focusable for SafetyView {
    fn focus_handle(&self, _: &App) -> gpui_kit::FocusHandle {
        self.focus.clone()
    }
}

impl SafetyView {
    /// A key typed in the panel: moves, switches view, runs the action whose letter it is.
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
        let key = ks.key.as_str();
        match key {
            "up" => self.move_selection(-1, cx),
            "down" => self.move_selection(1, cx),
            "pageup" => self.move_selection(-8, cx),
            "pagedown" => self.move_selection(8, cx),
            "home" => self.move_selection(-100_000, cx),
            "end" => self.move_selection(100_000, cx),
            "escape" => window.close_dialog(cx),
            "1" => self.set_tab(Tab::History, cx),
            "2" => self.set_tab(Tab::Reflog, cx),
            "3" => self.set_tab(Tab::Lost, cx),
            "tab" => {
                let next = match (self.tab, ks.modifiers.shift) {
                    (Tab::History, false) | (Tab::Lost, true) => Tab::Reflog,
                    (Tab::Reflog, false) | (Tab::History, true) => Tab::Lost,
                    (Tab::Lost, false) | (Tab::Reflog, true) => Tab::History,
                };
                self.set_tab(next, cx);
            }
            letter => {
                let acts = self.actions();
                let all = [
                    Act::UndoTo,
                    Act::RedoTo,
                    Act::UndoSequence,
                    Act::RestoreOnly,
                    Act::Offer,
                    Act::ShowInLog,
                    Act::CreateBranch,
                    Act::RestoreRef,
                    Act::CherryPick,
                    Act::CreateTag,
                    Act::ApplyToWorktree,
                    Act::SaveAsStash,
                    Act::LoadMore,
                ];
                // Several views share a letter; the action that applies to the selected row wins.
                let hit = all
                    .iter()
                    .copied()
                    .find(|a| a.key() == letter && acts.contains(a))
                    .or_else(|| {
                        all.iter()
                            .copied()
                            .find(|a| a.key() == letter && self.owns(*a))
                    });
                if let Some(act) = hit {
                    self.run(act, cx);
                }
            }
        }
    }

    /// Whether `act` belongs to the open view at all (so a letter that does not apply to the selected
    /// row still says why instead of doing nothing).
    fn owns(&self, act: Act) -> bool {
        match self.tab {
            Tab::History => matches!(
                act,
                Act::UndoTo | Act::RedoTo | Act::UndoSequence | Act::RestoreOnly | Act::Offer
            ),
            Tab::Reflog => matches!(
                act,
                Act::ShowInLog
                    | Act::CreateBranch
                    | Act::CreateTag
                    | Act::ApplyToWorktree
                    | Act::SaveAsStash
                    | Act::RestoreRef
                    | Act::LoadMore
            ),
            Tab::Lost => matches!(
                act,
                Act::CreateBranch
                    | Act::CreateTag
                    | Act::CherryPick
                    | Act::ApplyToWorktree
                    | Act::SaveAsStash
            ),
        }
    }

    /// The header line over the list.
    fn header(&self, state: &JournalState) -> String {
        match self.tab {
            Tab::History => {
                if !self.history.loaded {
                    return m::LOADING.get().to_string();
                }
                let n = self.history.undo_count as u64;
                let redo = self
                    .history
                    .rows
                    .len()
                    .saturating_sub(self.history.undo_count) as u64;
                let mut text = m::header_unlimited(n);
                if redo > 0 {
                    text = format!("{text}  -  {}", m::header_redo(redo));
                }
                let _ = state;
                text
            }
            Tab::Reflog => m::reflog_for(&self.reflog.refname),
            Tab::Lost => {
                if self.lost.scanning && self.lost.commits.is_empty() {
                    m::LOST_SCANNING.get().to_string()
                } else {
                    m::LOST_INTRO.get().to_string()
                }
            }
        }
    }

    /// The number of lines in the open list.
    pub fn line_count(&self) -> usize {
        match self.tab {
            Tab::History => self.history.lines.len(),
            Tab::Reflog => self.reflog.rows(),
            Tab::Lost => self.lost.commits.len(),
        }
    }

    /// The text of the open list when it has nothing to list.
    pub fn empty_text(&self) -> Option<String> {
        match self.tab {
            Tab::History => self.history.error.clone().or_else(|| {
                (self.history.loaded && self.history.rows.is_empty())
                    .then(|| m::HISTORY_EMPTY.get().to_string())
            }),
            Tab::Reflog => self.reflog.error.clone().or_else(|| {
                (self.reflog.loaded && self.reflog.entries.is_empty())
                    .then(|| m::reflog_empty(&self.reflog.refname))
            }),
            Tab::Lost => self.lost.error.clone().or_else(|| {
                (self.lost.loaded && !self.lost.scanning && self.lost.commits.is_empty())
                    .then(|| m::LOST_EMPTY.get().to_string())
            }),
        }
    }

    fn render_row(
        &self,
        ix: usize,
        skin: &Skin,
        state: &JournalState,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let base = h_flex()
            .id(("safety-row", ix))
            .h(px(ROW_H))
            .px_3()
            .gap_3()
            .items_center()
            .text_sm();
        match self.tab {
            Tab::History => match self.history.lines[ix] {
                Line::Day(day) => base
                    .text_color(skin.accent)
                    .child(day_heading(day, crate::prefs::PrefsStore::get(cx).dates()))
                    .into_any_element(),
                Line::Redo => {
                    let n = self
                        .history
                        .rows
                        .len()
                        .saturating_sub(self.history.undo_count) as u64;
                    base.text_color(skin.muted)
                        .border_t_1()
                        .border_color(skin.border)
                        .child(m::header_redo(n))
                        .into_any_element()
                }
                Line::Row(row) => {
                    let r = &self.history.rows[row];
                    let st = row_state(r, row, state);
                    let selected = self.history.selected == ix;
                    let color = match &st {
                        RowState::Blocked(_) => skin.danger,
                        RowState::Undone => skin.muted,
                        RowState::Undoable => skin.text,
                    };
                    base.text_color(color)
                        .when(selected, |d| d.bg(skin.selection))
                        .cursor_pointer()
                        .hover(|d| d.bg(skin.hover))
                        .child(
                            div()
                                .flex_none()
                                .w(px(110.0))
                                .text_color(skin.muted)
                                .child(age(r.entry.summary.at)),
                        )
                        .child(
                            div()
                                .flex_1()
                                .truncate()
                                .child(SharedString::from(label::entry_text(&r.entry))),
                        )
                        .child(
                            div()
                                .flex_none()
                                .text_xs()
                                .text_color(skin.muted)
                                .child(tag(r, &st)),
                        )
                        .on_click(cx.listener(move |this, _, _, cx| this.select(ix, cx)))
                        .into_any_element()
                }
            },
            Tab::Reflog => {
                if self.reflog.is_more_row(ix) {
                    return base
                        .text_color(skin.accent)
                        .cursor_pointer()
                        .child(if self.reflog.fetching_more {
                            m::LOADING.get().to_string()
                        } else {
                            m::BTN_LOAD_MORE.get().to_string()
                        })
                        .on_click(cx.listener(move |this, _, _, cx| this.select(ix, cx)))
                        .into_any_element();
                }
                let e = &self.reflog.entries[ix];
                let selected = self.reflog.cursor.selected == ix;
                base.text_color(skin.text)
                    .when(selected, |d| d.bg(skin.selection))
                    .cursor_pointer()
                    .hover(|d| d.bg(skin.hover))
                    .child(
                        div()
                            .flex_none()
                            .w(px(64.0))
                            .text_color(skin.accent)
                            .child(short(e.new)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .truncate()
                            .child(SharedString::from(e.message.clone())),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_xs()
                            .text_color(skin.muted)
                            .child(age(e.time)),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| this.select(ix, cx)))
                    .into_any_element()
            }
            Tab::Lost => {
                let c = &self.lost.commits[ix];
                let selected = self.lost.cursor.selected == ix;
                let source = match c.source {
                    gitpoulpi_core::domain::LostSource::Reflog => m::SOURCE_REFLOG.get(),
                    gitpoulpi_core::domain::LostSource::Pin => m::SOURCE_PIN.get(),
                    _ => m::SOURCE_DANGLING.get(),
                };
                base.text_color(skin.text)
                    .when(selected, |d| d.bg(skin.selection))
                    .cursor_pointer()
                    .hover(|d| d.bg(skin.hover))
                    .child(
                        div()
                            .flex_none()
                            .w(px(64.0))
                            .text_color(skin.accent)
                            .child(short(c.oid)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .truncate()
                            .child(SharedString::from(c.summary.clone())),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_xs()
                            .text_color(skin.muted)
                            .child(format!("{}, {}", c.author, age(c.time))),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_xs()
                            .text_color(skin.muted)
                            .child(source),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| this.select(ix, cx)))
                    .into_any_element()
            }
        }
    }

    /// The lines under the list: what the selected row is and what its actions bring back.
    fn detail_lines(&self, state: &JournalState) -> Vec<String> {
        match self.tab {
            Tab::History => self
                .history
                .selected()
                .map(|r| {
                    let index = self.history.selected_row().unwrap_or(0);
                    history_detail(r, &row_state(r, index, state))
                })
                .unwrap_or_default(),
            Tab::Reflog => self
                .reflog
                .selected()
                .map(|e| {
                    vec![format!(
                        "{} -> {}  {}",
                        short(e.old),
                        short(e.new),
                        e.committer
                    )]
                })
                .unwrap_or_default(),
            Tab::Lost => self
                .lost
                .selected()
                .map(|c| vec![format!("{}  {}", short(c.oid), c.summary)])
                .unwrap_or_default(),
        }
    }
}

impl Render for SafetyView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let state = self.journal_state();
        let count = self.line_count();
        let tabs = h_flex()
            .gap_1()
            .child(
                tab_chip(
                    "safety-tab-history",
                    m::TAB_HISTORY.get(),
                    self.tab == Tab::History,
                    &skin,
                )
                .on_click(cx.listener(|this, _, _, cx| this.set_tab(Tab::History, cx))),
            )
            .child(
                tab_chip(
                    "safety-tab-reflog",
                    m::tab_reflog(&self.reflog.refname),
                    self.tab == Tab::Reflog,
                    &skin,
                )
                .on_click(cx.listener(|this, _, _, cx| this.set_tab(Tab::Reflog, cx))),
            )
            .child(
                tab_chip(
                    "safety-tab-lost",
                    m::TAB_LOST.get(),
                    self.tab == Tab::Lost,
                    &skin,
                )
                .on_click(cx.listener(|this, _, _, cx| this.set_tab(Tab::Lost, cx))),
            );
        let list: AnyElement = match self.empty_text() {
            Some(text)
                if count == 0 || self.tab == Tab::History && self.history.rows.is_empty() =>
            {
                div()
                    .flex_1()
                    .p_3()
                    .text_sm()
                    .text_color(skin.muted)
                    .child(text)
                    .into_any_element()
            }
            _ => uniform_list(
                "safety-list",
                count,
                cx.processor(|this, range: std::ops::Range<usize>, _window, cx| {
                    let skin = cx.global::<Skin>().clone();
                    let state = this.journal_state();
                    range
                        .map(|ix| this.render_row(ix, &skin, &state, cx))
                        .collect::<Vec<_>>()
                }),
            )
            .track_scroll(&self.scroll)
            .flex_1()
            .min_h_0()
            .into_any_element(),
        };
        let refname = self.reflog.refname.clone();
        let buttons: Vec<AnyElement> = self
            .actions()
            .into_iter()
            .map(|act| {
                let danger = matches!(act, Act::UndoSequence);
                chip(
                    ("safety-act", act as usize),
                    act.label(&state, &refname),
                    act.key(),
                    !self.busy,
                    danger,
                    &skin,
                )
                .on_click(cx.listener(move |this, _, _, cx| this.run(act, cx)))
                .into_any_element()
            })
            .collect();
        let detail = self.detail_lines(&state);
        v_flex()
            .key_context("GpSafety")
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
                    .child(self.header(&state)),
            )
            .child(div().px_2().py_1().child(tabs))
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
                    .min_h(px(54.0))
                    .text_xs()
                    .text_color(skin.muted)
                    .children(
                        detail
                            .into_iter()
                            .map(|l| div().child(SharedString::from(l))),
                    )
                    .when_some(self.note.clone(), |d, (text, error)| {
                        d.child(
                            div()
                                .text_color(if error { skin.danger } else { skin.text })
                                .child(SharedString::from(text)),
                        )
                    }),
            )
            .child(
                h_flex()
                    .px_2()
                    .py_2()
                    .gap_2()
                    .flex_wrap()
                    .children(buttons)
                    .child(div().flex_1())
                    .child(
                        chip("safety-close", m::BTN_CLOSE.get(), "", true, false, &skin)
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    ),
            )
    }
}

/// Shows the panel on `tab` (the Reflog of `refname`) in a dialog; the open one is reused.
pub fn open(
    tab: Tab,
    refname: Option<String>,
    window: &mut Window,
    cx: &mut App,
) -> Option<Entity<SafetyView>> {
    if let Some(existing) = current(cx) {
        existing.update(cx, |v, cx| match (tab, refname.as_deref()) {
            (Tab::Reflog, Some(name)) => v.show_reflog_of(name, cx),
            _ => v.set_tab(tab, cx),
        });
        return Some(existing);
    }
    let backend = crate::app::workspace_of(cx)?.read(cx).backend().clone();
    let view = cx.new(|cx| SafetyView::new(backend, tab, refname, window, cx));
    cx.set_global(OpenSafety(view.downgrade()));
    window.open_dialog(cx, {
        let view = view.clone();
        move |dialog, _, _| {
            let view = view.clone();
            dialog
                .close_button(true)
                .overlay_closable(true)
                .w(px(880.0))
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
