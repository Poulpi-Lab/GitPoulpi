//! The commit context menu: a popup at the pointer (right-click) or at the selected row (the menu key,
//! Shift+F10, `.`). Submenus drill in, as in the terminal UI: Enter or Right opens one, Left or Esc goes
//! back, Esc at the top closes. Rows are commands run through the one dispatcher; a row that cannot run
//! stays listed with its reason. The close control, a click away and Esc all close it.

use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    actions, anchored, deferred, div, point, px, Anchor, App, Context, EventEmitter, FocusHandle,
    Focusable, InteractiveElement as _, IntoElement, KeyBinding, MouseButton, ParentElement as _,
    Pixels, Point, Render, SharedString, StatefulInteractiveElement as _, Styled as _, Window,
};

use super::menu_items::{is_row, Item, Kind};
use crate::commands::{self, status};
use crate::theme::Skin;

actions!(
    gp_log_menu,
    [MenuUp, MenuDown, MenuFirst, MenuLast, MenuRun, MenuBack, MenuInto]
);

/// Binds the menu's keys. They are bound in the menu's own context, which sits inside the Log's: the
/// deeper context wins, so the Log's arrows and Esc do not act while the menu is open.
pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", MenuUp, Some(CONTEXT)),
        KeyBinding::new("down", MenuDown, Some(CONTEXT)),
        KeyBinding::new("home", MenuFirst, Some(CONTEXT)),
        KeyBinding::new("end", MenuLast, Some(CONTEXT)),
        KeyBinding::new("enter", MenuRun, Some(CONTEXT)),
        KeyBinding::new("space", MenuRun, Some(CONTEXT)),
        KeyBinding::new("right", MenuInto, Some(CONTEXT)),
        KeyBinding::new("left", MenuBack, Some(CONTEXT)),
        KeyBinding::new("backspace", MenuBack, Some(CONTEXT)),
        KeyBinding::new("escape", MenuBack, Some(CONTEXT)),
    ]);
}

/// Key context of the open menu. It contains `GpMenu`, so the global key interceptor stays out of the
/// way while the menu owns the keyboard.
pub const CONTEXT: &str = "GpMenuLog";

/// The menu closed itself (a row ran, Esc, the close control, a click away).
pub struct Closed;

pub struct CommitMenu {
    title: String,
    root: Vec<Item>,
    /// The drill-in path: an index into each level's list, outermost first.
    path: Vec<usize>,
    cursor: usize,
    at: Point<Pixels>,
    focus: FocusHandle,
    /// Where the keyboard goes back to when the menu closes.
    back_to: FocusHandle,
}

impl EventEmitter<Closed> for CommitMenu {}

impl CommitMenu {
    pub fn new(
        title: String,
        items: Vec<Item>,
        at: Point<Pixels>,
        back_to: FocusHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let this = Self {
            title,
            cursor: first_row(&items),
            root: items,
            path: Vec::new(),
            at,
            focus: cx.focus_handle(),
            back_to,
        };
        this.focus.focus(window, cx);
        this
    }

    /// The rows of the level on show.
    pub fn items(&self) -> &[Item] {
        let mut list: &[Item] = &self.root;
        for &ix in &self.path {
            match list.get(ix).map(|i| &i.kind) {
                Some(Kind::Sub(inner)) => list = inner,
                _ => break,
            }
        }
        list
    }

    /// The labels of the rows on show (rules left out), for tests and the title.
    pub fn labels(&self) -> Vec<String> {
        self.items()
            .iter()
            .filter(|i| is_row(i))
            .map(|i| i.label.clone())
            .collect()
    }

    /// The row under the cursor.
    pub fn current(&self) -> Option<&Item> {
        self.items().get(self.cursor)
    }

    /// How deep the drill-in is (0 at the top).
    pub fn depth(&self) -> usize {
        self.path.len()
    }

    fn step(&mut self, delta: isize, cx: &mut Context<Self>) {
        let items = self.items();
        let rows: Vec<usize> = (0..items.len()).filter(|&i| is_row(&items[i])).collect();
        if rows.is_empty() {
            return;
        }
        let at = rows.iter().position(|&i| i == self.cursor).unwrap_or(0) as isize;
        let next = (at + delta).rem_euclid(rows.len() as isize) as usize;
        self.cursor = rows[next];
        cx.notify();
    }

    fn jump(&mut self, last: bool, cx: &mut Context<Self>) {
        let items = self.items();
        let rows: Vec<usize> = (0..items.len()).filter(|&i| is_row(&items[i])).collect();
        if let Some(&ix) = if last { rows.last() } else { rows.first() } {
            self.cursor = ix;
            cx.notify();
        }
    }

    /// Enter / click on a row: a submenu opens, a command runs, a disabled row says why.
    pub fn activate_at(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item) = self.items().get(ix).cloned() else {
            return;
        };
        self.cursor = ix;
        if let Some(reason) = &item.disabled {
            status::set(cx, format!("{} - {}", item.label, reason.to_lowercase()));
            cx.notify();
            return;
        }
        match item.kind {
            Kind::Rule => {}
            Kind::Sub(inner) => {
                self.path.push(ix);
                self.cursor = first_row(&inner);
                cx.notify();
            }
            Kind::Run(command) => {
                self.close(window, cx);
                window.defer(cx, move |window, cx| {
                    commands::run(command, window, cx);
                });
            }
        }
    }

    pub fn activate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.activate_at(self.cursor, window, cx);
    }

    /// Left / Esc: one level back, or close at the top.
    pub fn back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.path.pop() {
            Some(ix) => {
                self.cursor = ix;
                cx.notify();
            }
            None => self.close(window, cx),
        }
    }

    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.back_to.focus(window, cx);
        cx.emit(Closed);
    }

    /// One key, as the menu reads it (`up`, `down`, `home`, `end`, `enter`, `space`, `right`, `left`,
    /// `escape`). `true` when the key was the menu's.
    pub fn press(&mut self, key: &str, window: &mut Window, cx: &mut Context<Self>) -> bool {
        match key {
            "down" => self.step(1, cx),
            "up" => self.step(-1, cx),
            "home" => self.jump(false, cx),
            "end" => self.jump(true, cx),
            "enter" | "space" => self.activate(window, cx),
            "right" => {
                if self
                    .current()
                    .is_some_and(|i| matches!(i.kind, Kind::Sub(_)))
                {
                    self.activate(window, cx);
                }
            }
            "left" | "backspace" | "escape" => self.back(window, cx),
            _ => return false,
        }
        true
    }

    fn header(&self) -> String {
        let mut label = self.title.clone();
        let mut list: &[Item] = &self.root;
        for &ix in &self.path {
            if let Some(item) = list.get(ix) {
                label = item.label.clone();
                if let Kind::Sub(inner) = &item.kind {
                    list = inner;
                }
            }
        }
        label
    }
}

fn first_row(items: &[Item]) -> usize {
    items.iter().position(is_row).unwrap_or(0)
}

impl Focusable for CommitMenu {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for CommitMenu {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let cursor = self.cursor;
        let deep = self.depth() > 0;
        let footer: SharedString = self
            .current()
            .and_then(|i| i.disabled.clone())
            .unwrap_or_default()
            .into();
        let rows = self
            .items()
            .iter()
            .enumerate()
            .map(|(ix, item)| {
                if !is_row(item) {
                    return div()
                        .h(px(1.0))
                        .my(px(3.0))
                        .bg(skin.border)
                        .into_any_element();
                }
                let disabled = item.disabled.is_some();
                let sub = matches!(item.kind, Kind::Sub(_));
                h_flex()
                    .id(("gp-log-menu-row", ix))
                    .px(px(10.0))
                    .h(px(26.0))
                    .gap_4()
                    .items_center()
                    .justify_between()
                    .text_color(if disabled {
                        skin.muted
                    } else if item.danger {
                        skin.danger
                    } else {
                        skin.text
                    })
                    .when(cursor == ix, |d| d.bg(skin.selection))
                    .when(!disabled, |d| {
                        d.cursor_pointer().hover(|d| d.bg(skin.hover))
                    })
                    .child(div().flex_1().truncate().child(item.label.clone()))
                    .child(
                        div()
                            .flex_none()
                            .text_xs()
                            .text_color(skin.muted)
                            .child(if sub {
                                ">".to_string()
                            } else {
                                item.shortcut.clone()
                            }),
                    )
                    .on_click(
                        cx.listener(move |this, _, window, cx| this.activate_at(ix, window, cx)),
                    )
                    .into_any_element()
            })
            .collect::<Vec<_>>();

        deferred(
            anchored()
                .position(point(self.at.x, self.at.y))
                .anchor(Anchor::TopLeft)
                .snap_to_window_with_margin(px(8.0))
                .child(
                    v_flex()
                        .id("gp-log-menu")
                        .key_context(CONTEXT)
                        .track_focus(&self.focus)
                        .on_action(cx.listener(|this, _: &MenuUp, w, cx| {
                            this.press("up", w, cx);
                        }))
                        .on_action(cx.listener(|this, _: &MenuDown, w, cx| {
                            this.press("down", w, cx);
                        }))
                        .on_action(cx.listener(|this, _: &MenuFirst, w, cx| {
                            this.press("home", w, cx);
                        }))
                        .on_action(cx.listener(|this, _: &MenuLast, w, cx| {
                            this.press("end", w, cx);
                        }))
                        .on_action(cx.listener(|this, _: &MenuRun, w, cx| {
                            this.press("enter", w, cx);
                        }))
                        .on_action(cx.listener(|this, _: &MenuInto, w, cx| {
                            this.press("right", w, cx);
                        }))
                        .on_action(cx.listener(|this, _: &MenuBack, w, cx| {
                            this.press("escape", w, cx);
                        }))
                        .occlude()
                        .w(px(320.0))
                        .bg(skin.bg)
                        .text_sm()
                        .border_1()
                        .border_color(skin.border)
                        .rounded(px(4.0))
                        .shadow_md()
                        .on_mouse_down_out(
                            cx.listener(|this, _, window, cx| this.close(window, cx)),
                        )
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .child(
                            h_flex()
                                .h(px(26.0))
                                .px(px(10.0))
                                .gap_2()
                                .items_center()
                                .border_b_1()
                                .border_color(skin.border)
                                .text_color(skin.muted)
                                .child(
                                    div()
                                        .id("gp-log-menu-back")
                                        .when(deep, |d| d.cursor_pointer().child("<"))
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            if this.depth() > 0 {
                                                this.back(window, cx);
                                            }
                                        })),
                                )
                                .child(div().flex_1().truncate().child(self.header()))
                                .child(
                                    div()
                                        .id("gp-log-menu-close")
                                        .px(px(6.0))
                                        .rounded(px(3.0))
                                        .cursor_pointer()
                                        .hover(|d| d.bg(skin.hover))
                                        .child("\u{00D7}")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.close(window, cx)
                                        })),
                                ),
                        )
                        .child(
                            v_flex()
                                .id("gp-log-menu-rows")
                                .max_h(px(480.0))
                                .overflow_y_scroll()
                                .py(px(4.0))
                                .children(rows),
                        )
                        .when(!footer.is_empty(), |d| {
                            d.child(
                                div()
                                    .px(px(10.0))
                                    .h(px(22.0))
                                    .flex()
                                    .items_center()
                                    .border_t_1()
                                    .border_color(skin.border)
                                    .text_xs()
                                    .text_color(skin.muted)
                                    .child(footer),
                            )
                        }),
                ),
        )
        .with_priority(10)
    }
}
