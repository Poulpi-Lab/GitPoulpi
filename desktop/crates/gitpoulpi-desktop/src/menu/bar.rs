//! The menu bar view: headings in a row, one open popup under the heading, keyboard and mouse.
//!
//! Keyboard: F10 or Alt + a heading's mnemonic opens a menu (see [`super::handle_chord`]); Left / Right
//! change heading, Up / Down move, Enter or Space runs (or enters a submenu), Escape closes. Items are
//! rows with the command's current key on the right; a disabled item stays listed, with its reason as a
//! tooltip and in the popup's footer line. When the window is too narrow for every heading, the last
//! ones fold into one `...` menu that lists their submenus.

use gitpoulpi_app::keymap;
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    deferred, div, px, App, Context, FocusHandle, Focusable, InteractiveElement as _, IntoElement,
    KeyDownEvent, MouseButton, ParentElement as _, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window,
};

use super::model::{self, Action, Group, Item, Menu};
use crate::commands::{self, context::window_state, status};
use crate::strings;
use crate::theme::Skin;

const BAR_HEIGHT: f32 = 28.0;
/// Estimated width of a heading: padding plus its characters.
const HEADING_PAD: f32 = 20.0;
const CHAR_WIDTH: f32 = 8.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Level {
    Groups,
    Items,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Open {
    pub heading: usize,
    pub level: Level,
    pub group: usize,
    pub item: usize,
}

pub struct MenuBar {
    /// Every menu, as the registry lays them out.
    menus: Vec<Menu>,
    /// What the bar shows: `menus` with the ones that do not fit folded into a last `...` menu.
    effective: Vec<Menu>,
    /// How many of `menus` are shown as headings.
    visible: usize,
    width: f32,
    epoch: u64,
    pub(super) open: Option<Open>,
    focus: FocusHandle,
    previous: Option<gpui_kit::WeakFocusHandle>,
    scroll: ScrollHandle,
}

impl MenuBar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut bar = Self {
            menus: Vec::new(),
            effective: Vec::new(),
            visible: 0,
            width: 1600.0,
            epoch: gitpoulpi_i18n::epoch(),
            open: None,
            focus: cx.focus_handle(),
            previous: None,
            scroll: ScrollHandle::new(),
        };
        bar.rebuild(None, cx);
        bar
    }

    /// Rebuilds the menus for the window's current state (availability, keys, recent repositories).
    fn rebuild(&mut self, window: Option<&Window>, cx: &mut Context<Self>) {
        let state = match window {
            Some(w) => window_state(w, cx),
            None => Default::default(),
        };
        let recent = crate::prefs::PrefsStore::get(cx).recent().to_vec();
        self.menus = model::build(state.palette_context(), &keymap::current(), &recent);
        self.epoch = gitpoulpi_i18n::epoch();
        self.relayout();
    }

    /// The settings changed (key bindings, language, recent repositories): the menus are built again,
    /// unless one is open (it rebuilds when it closes).
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        if self.open.is_none() {
            self.rebuild(None, cx);
            cx.notify();
        }
    }

    /// Recomputes which headings fit and folds the rest.
    fn relayout(&mut self) {
        self.visible = self.visible_count(self.width);
        self.effective = self.menus[..self.visible.min(self.menus.len())].to_vec();
        if self.visible < self.menus.len() {
            let folded = self.folded_menu(self.visible);
            self.effective.push(folded);
        }
    }

    /// The window got another width.
    pub fn set_width(&mut self, width: f32) {
        if (width - self.width).abs() > 0.5 {
            self.width = width;
            self.relayout();
            if let Some(o) = self.open.as_mut() {
                o.heading = o.heading.min(self.effective.len().saturating_sub(1));
            }
        }
    }

    /// The menus as the bar shows them.
    pub fn shown_menus(&self) -> &[Menu] {
        &self.effective
    }

    pub fn menus(&self) -> &[Menu] {
        &self.menus
    }

    pub fn open_state(&self) -> Option<Open> {
        self.open
    }

    /// Opens the menu with this mnemonic (a folded one opens the `...` menu on its submenu).
    pub fn open_mnemonic(&mut self, c: char, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(real) = self
            .menus
            .iter()
            .position(|m| m.mnemonic == Some(c.to_ascii_lowercase()))
        else {
            return false;
        };
        if real < self.visible {
            self.open_heading(real, window, cx);
        } else {
            self.open_heading(self.visible, window, cx);
            let before: usize = self.menus[self.visible..real]
                .iter()
                .map(|m| m.groups.len())
                .sum();
            if let Some(o) = self.open.as_mut() {
                o.group = before;
            }
        }
        true
    }

    /// Opens heading `ix` (rebuilding the menus first, so what is disabled is current).
    pub fn open_heading(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.open.is_none() {
            self.previous = window.focused(cx).map(|h| h.downgrade());
        }
        self.rebuild(Some(window), cx);
        let ix = ix.min(self.effective.len().saturating_sub(1));
        let flat = self.effective.get(ix).is_none_or(Menu::is_flat);
        self.open = Some(Open {
            heading: ix,
            level: if flat { Level::Items } else { Level::Groups },
            group: 0,
            item: 0,
        });
        self.focus.focus(window, cx);
        cx.notify();
    }

    /// Opens the first heading (F10).
    pub fn open_first(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_heading(0, window, cx);
    }

    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.open.take().is_none() {
            return;
        }
        match self.previous.take().and_then(|h| h.upgrade()) {
            Some(handle) => handle.focus(window, cx),
            None => window.focus(&self.focus, cx),
        }
        cx.notify();
    }

    fn current_menu(&self) -> Option<&Menu> {
        self.open.and_then(|o| self.effective.get(o.heading))
    }

    fn current_group(&self) -> Option<&Group> {
        let o = self.open?;
        self.current_menu()?.groups.get(o.group)
    }

    /// The item under the selection, when the selection is on an item.
    pub fn selected_item(&self) -> Option<&Item> {
        let o = self.open?;
        if o.level == Level::Groups && !self.current_menu()?.is_flat() {
            return None;
        }
        self.current_group()?.items.get(o.item)
    }

    fn step_heading(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(o) = self.open else { return };
        let n = self.effective.len() as isize;
        if n == 0 {
            return;
        }
        let next = (o.heading as isize + delta).rem_euclid(n) as usize;
        self.open_heading(next, window, cx);
    }

    fn step(&mut self, delta: isize, cx: &mut Context<Self>) {
        let Some(mut o) = self.open else { return };
        let Some(menu) = self.effective.get(o.heading) else {
            return;
        };
        let len = if o.level == Level::Groups && !menu.is_flat() {
            menu.groups.len()
        } else {
            menu.groups.get(o.group).map_or(0, |g| g.items.len())
        };
        if len == 0 {
            return;
        }
        let ix = if o.level == Level::Groups && !menu.is_flat() {
            &mut o.group
        } else {
            &mut o.item
        };
        *ix = (*ix as isize + delta).rem_euclid(len as isize) as usize;
        if o.level == Level::Groups && !menu.is_flat() {
            o.item = 0;
        }
        self.open = Some(o);
        self.scroll.scroll_to_item(o.item);
        cx.notify();
    }

    fn enter_group(&mut self, cx: &mut Context<Self>) {
        if let Some(o) = self.open.as_mut() {
            o.level = Level::Items;
            o.item = 0;
        }
        cx.notify();
    }

    /// Enter / click on the selection: a submenu opens, an item runs (a disabled one says why).
    pub fn activate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(o) = self.open else { return };
        if o.level == Level::Groups && !self.current_menu().is_none_or(Menu::is_flat) {
            self.enter_group(cx);
            return;
        }
        let Some(item) = self.selected_item().cloned() else {
            return;
        };
        self.run_item(item, window, cx);
    }

    fn run_item(&mut self, item: Item, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(reason) = &item.disabled {
            status::set(cx, format!("{} - {}", item.label, reason.to_lowercase()));
            return;
        }
        self.close(window, cx);
        let action = item.action;
        window.defer(cx, move |window, cx| match action {
            Action::Command(command) => {
                commands::run(command, window, cx);
            }
            Action::Recent(path) => {
                if let Some(ws) = crate::app::workspace_of(cx) {
                    ws.update(cx, |ws, cx| ws.open_repo(std::path::Path::new(&path), cx));
                }
            }
        });
    }

    fn on_key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some(o) = self.open else { return };
        let flat = self.current_menu().is_none_or(Menu::is_flat);
        match ev.keystroke.key.as_str() {
            "escape" | "f10" => self.close(window, cx),
            "down" => self.step(1, cx),
            "up" => self.step(-1, cx),
            "home" => self.step(-(o.item as isize + o.group as isize), cx),
            "enter" | "space" => self.activate(window, cx),
            "left" if o.level == Level::Items && !flat => {
                self.open = Some(Open {
                    level: Level::Groups,
                    ..o
                });
                cx.notify();
            }
            "left" => self.step_heading(-1, window, cx),
            "right" if o.level == Level::Groups && !flat => self.enter_group(cx),
            "right" => self.step_heading(1, window, cx),
            _ => return,
        }
        cx.stop_propagation();
    }

    /// How many headings fit in `width` before the rest folds into the `...` menu.
    fn visible_count(&self, width: f32) -> usize {
        let cell = |m: &Menu| HEADING_PAD + CHAR_WIDTH * m.title.chars().count() as f32;
        let all: f32 = self.menus.iter().map(cell).sum();
        if all <= width - 12.0 {
            return self.menus.len();
        }
        // Room for the `...` heading itself.
        let mut used = 12.0 + HEADING_PAD + CHAR_WIDTH * 3.0;
        let mut count = 0;
        for m in &self.menus {
            used += cell(m);
            if used > width {
                break;
            }
            count += 1;
        }
        count.max(1)
    }
}

impl Focusable for MenuBar {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

/// The underlined mnemonic of a title: (before, letter, after).
fn mnemonic_parts(title: &str, mnemonic: Option<char>) -> (String, Option<String>, String) {
    let Some(m) = mnemonic else {
        return (title.to_string(), None, String::new());
    };
    match title
        .char_indices()
        .find(|(_, c)| c.to_ascii_lowercase() == m)
    {
        Some((at, c)) => (
            title[..at].to_string(),
            Some(c.to_string()),
            title[at + c.len_utf8()..].to_string(),
        ),
        None => (title.to_string(), None, String::new()),
    }
}

impl Render for MenuBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.open.is_none() && self.epoch != gitpoulpi_i18n::epoch() {
            self.rebuild(None, cx);
        }
        let skin = cx.global::<Skin>().clone();
        self.set_width(window.viewport_size().width.as_f32());
        let overflow = self.visible < self.menus.len();
        let last = self.effective.len().saturating_sub(1);
        let open = self.open;

        let mut bar = h_flex()
            .id("gp-menu-bar")
            .key_context("GpMenu")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, ev, window, cx| this.on_key(ev, window, cx)))
            .flex_none()
            .h(px(BAR_HEIGHT))
            .px(px(6.0))
            .gap(px(2.0))
            .items_center()
            .border_b_1()
            .border_color(skin.border)
            .text_sm();

        for (ix, menu) in self.effective.iter().enumerate() {
            let is_open = open.is_some_and(|o| o.heading == ix);
            bar = if overflow && ix == last {
                bar.child(self.folded_cell(ix, menu.clone(), is_open, &skin, cx))
            } else {
                bar.child(self.heading_cell(ix, menu.clone(), is_open, &skin, cx))
            };
        }
        bar
    }
}

impl MenuBar {
    /// The menus past the visible ones, as one menu of submenus.
    fn folded_menu(&self, visible: usize) -> Menu {
        let mut groups = Vec::new();
        for menu in &self.menus[visible..] {
            for group in &menu.groups {
                groups.push(Group {
                    title: if menu.is_flat() {
                        menu.title.clone()
                    } else {
                        format!("{} - {}", menu.title, group.title)
                    },
                    items: group.items.clone(),
                });
            }
        }
        Menu {
            heading: self.menus[visible].heading,
            title: "...".to_string(),
            mnemonic: None,
            groups,
        }
    }

    fn heading_cell(
        &self,
        ix: usize,
        menu: Menu,
        is_open: bool,
        skin: &Skin,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let (before, letter, after) = mnemonic_parts(&menu.title, menu.mnemonic);
        let label = h_flex()
            .child(before)
            .when_some(letter, |d, l| d.child(div().underline().child(l)))
            .child(after);
        div()
            .relative()
            .child(
                div()
                    .id(("gp-menu-heading", ix))
                    .h(px(BAR_HEIGHT - 6.0))
                    .px(px(8.0))
                    .flex()
                    .items_center()
                    .rounded(px(3.0))
                    .text_color(skin.text)
                    .when(is_open, |d| d.bg(skin.selection))
                    .hover(|d| d.bg(skin.hover))
                    .child(label)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if this.open.is_some_and(|o| o.heading == ix) {
                            this.close(window, cx);
                        } else {
                            this.open_heading(ix, window, cx);
                        }
                    }))
                    .on_hover(cx.listener(move |this, hovered: &bool, window, cx| {
                        if *hovered && this.open.is_some_and(|o| o.heading != ix) {
                            this.open_heading(ix, window, cx);
                        }
                    })),
            )
            .when(is_open, |d| d.child(self.popup(&menu, skin, cx)))
    }

    fn folded_cell(
        &self,
        ix: usize,
        folded: Menu,
        is_open: bool,
        skin: &Skin,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .relative()
            .child(
                div()
                    .id("gp-menu-overflow")
                    .h(px(BAR_HEIGHT - 6.0))
                    .px(px(8.0))
                    .flex()
                    .items_center()
                    .rounded(px(3.0))
                    .text_color(skin.text)
                    .when(is_open, |d| d.bg(skin.selection))
                    .hover(|d| d.bg(skin.hover))
                    .child("...")
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if this.open.is_some_and(|o| o.heading == ix) {
                            this.close(window, cx);
                        } else {
                            this.open_heading(ix, window, cx);
                        }
                    })),
            )
            .when(is_open, |d| d.child(self.popup(&folded, skin, cx)))
    }

    /// The popup under a heading: the submenu list (unless flat) and the items of the selected one.
    fn popup(&self, menu: &Menu, skin: &Skin, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(o) = self.open else {
            return div().into_any_element();
        };
        let flat = menu.is_flat();
        let group = menu
            .groups
            .get(o.group.min(menu.groups.len().saturating_sub(1)));
        let selected_item = if o.level == Level::Items || flat {
            group.and_then(|g| g.items.get(o.item))
        } else {
            None
        };
        let footer: SharedString = selected_item
            .and_then(|i| i.disabled.clone())
            .unwrap_or_default()
            .into();

        let groups_panel = (!flat).then(|| {
            v_flex()
                .min_w(px(180.0))
                .py(px(4.0))
                .border_r_1()
                .border_color(skin.border)
                .children(menu.groups.iter().enumerate().map(|(gi, g)| {
                    let selected = gi == o.group;
                    h_flex()
                        .id(("gp-menu-group", gi))
                        .px(px(10.0))
                        .h(px(26.0))
                        .gap_3()
                        .items_center()
                        .justify_between()
                        .text_color(skin.text)
                        .when(selected, |d| d.bg(skin.selection))
                        .hover(|d| d.bg(skin.hover))
                        .child(div().truncate().child(g.title.clone()))
                        .child(div().text_color(skin.muted).child(">"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(o) = this.open.as_mut() {
                                o.group = gi;
                                o.item = 0;
                                o.level = Level::Items;
                            }
                            cx.notify();
                        }))
                }))
        });

        let items_panel = group.map(|g| {
            v_flex()
                .id("gp-menu-items")
                .track_scroll(&self.scroll)
                .overflow_y_scroll()
                .max_h(px(440.0))
                .min_w(px(300.0))
                .py(px(4.0))
                .children(g.items.iter().enumerate().map(|(ii, item)| {
                    let selected = (o.level == Level::Items || flat) && ii == o.item;
                    let disabled = item.disabled.is_some();
                    let reason = item.disabled.clone();
                    let item_for_click = item.clone();
                    h_flex()
                        .id(("gp-menu-item", ii))
                        .px(px(10.0))
                        .h(px(26.0))
                        .gap_4()
                        .items_center()
                        .justify_between()
                        .text_color(if disabled { skin.muted } else { skin.text })
                        .when(selected, |d| d.bg(skin.selection))
                        .when(!disabled, |d| d.hover(|d| d.bg(skin.hover)))
                        .child(div().flex_1().truncate().child(item.label.clone()))
                        .child(
                            div()
                                .flex_none()
                                .text_xs()
                                .text_color(skin.muted)
                                .child(item.shortcut.clone()),
                        )
                        .when_some(reason, |d, reason| {
                            d.tooltip(move |window, cx| {
                                gpui_kit::component::tooltip::Tooltip::new(reason.clone())
                                    .build(window, cx)
                            })
                        })
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.run_item(item_for_click.clone(), window, cx)
                        }))
                }))
        });

        deferred(
            v_flex()
                .absolute()
                .top(px(BAR_HEIGHT - 4.0))
                .left_0()
                .occlude()
                .bg(skin.bg)
                .border_1()
                .border_color(skin.border)
                .rounded(px(4.0))
                .shadow_md()
                .on_mouse_down_out(cx.listener(|this, _, window, cx| this.close(window, cx)))
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .child(
                    h_flex()
                        .items_start()
                        .when_some(groups_panel, |d, p| d.child(p))
                        .when_some(items_panel, |d, p| d.child(p)),
                )
                .child(
                    h_flex()
                        .h(px(24.0))
                        .px(px(10.0))
                        .gap_3()
                        .items_center()
                        .justify_between()
                        .border_t_1()
                        .border_color(skin.border)
                        .text_xs()
                        .text_color(skin.muted)
                        .child(div().flex_1().truncate().child(footer))
                        .child(
                            div()
                                .id("gp-menu-close")
                                .px(px(6.0))
                                .rounded(px(3.0))
                                .hover(|d| d.bg(skin.hover))
                                .child("\u{00D7}")
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.close(window, cx)),
                                ),
                        ),
                ),
        )
        .with_priority(10)
        .into_any_element()
    }
}

/// The text shown for a menu item that is not implemented, for tests and tooltips.
pub fn not_available() -> &'static str {
    strings::NOT_AVAILABLE_YET
}
