//! The menu as a view: rows, keyboard and mouse.

use super::*;

// -- the view ---------------------------------------------------------------------------------------

/// What the menu tells its owner.
pub enum RefMenuEvent {
    /// Run `op` on the menu's target, and close.
    Run(RefOp),
    /// A drop row was chosen.
    Drop(rules::DropOp),
    /// A row that cannot run was chosen: say why (the menu stays open).
    Refused(String),
    Close,
}

impl EventEmitter<RefMenuEvent> for RefMenu {}

pub struct RefMenu {
    title: Option<String>,
    /// The rows of each open level; the last is shown.
    levels: Vec<Vec<Item>>,
    selected: Vec<usize>,
    /// For a drop menu: the op of each row, in order of the actionable rows.
    drops: Vec<rules::DropOp>,
    pub position: Point<Pixels>,
    focus: FocusHandle,
}

impl RefMenu {
    pub fn new(items: Vec<Item>, position: Point<Pixels>, cx: &mut Context<Self>) -> Self {
        let first = items.iter().position(Item::selectable).unwrap_or(0);
        Self {
            title: None,
            levels: vec![items],
            selected: vec![first],
            drops: Vec::new(),
            position,
            focus: cx.focus_handle(),
        }
    }

    /// A drop menu: `title` names the pair, `rows` are the offers.
    pub fn for_drop(
        title: String,
        rows: Vec<(Item, rules::DropOp)>,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) -> Self {
        let drops = rows.iter().map(|(_, op)| *op).collect();
        let mut menu = Self::new(rows.into_iter().map(|(i, _)| i).collect(), position, cx);
        menu.title = Some(title);
        menu.drops = drops;
        menu
    }

    /// The rows shown now.
    pub fn items(&self) -> &[Item] {
        self.levels.last().map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn selected(&self) -> usize {
        self.selected.last().copied().unwrap_or(0)
    }

    pub fn labels(&self) -> Vec<String> {
        self.items().iter().map(|i| i.label().to_string()).collect()
    }

    /// Whether the `ix`th row is dimmed, and why.
    pub fn refusal(&self, ix: usize) -> Option<String> {
        match self.items().get(ix) {
            Some(Item::Action { disabled, .. }) => disabled.clone(),
            _ => None,
        }
    }

    pub fn move_by(&mut self, delta: isize, cx: &mut Context<Self>) {
        let items = self.items();
        let n = items.len() as isize;
        if n == 0 {
            return;
        }
        let mut at = self.selected() as isize;
        loop {
            at += delta;
            if at < 0 || at >= n {
                return;
            }
            if items[at as usize].selectable() {
                break;
            }
        }
        if let Some(s) = self.selected.last_mut() {
            *s = at as usize;
        }
        cx.notify();
    }

    pub fn select(&mut self, ix: usize, cx: &mut Context<Self>) {
        if let Some(s) = self.selected.last_mut() {
            *s = ix;
        }
        cx.notify();
    }

    /// Enter, Right or a click on the `ix`th row.
    pub fn activate(&mut self, ix: usize, cx: &mut Context<Self>) {
        let Some(item) = self.items().get(ix).cloned() else {
            return;
        };
        self.select(ix, cx);
        match item {
            Item::Separator => {}
            Item::Submenu { items, .. } => {
                let first = items.iter().position(Item::selectable).unwrap_or(0);
                self.levels.push(items);
                self.selected.push(first);
                cx.notify();
            }
            Item::Action {
                disabled: Some(why),
                ..
            } => cx.emit(RefMenuEvent::Refused(why)),
            Item::Action { op: Some(op), .. } => {
                if self.drops.is_empty() {
                    cx.emit(RefMenuEvent::Run(op));
                } else if let Some(d) = self.drops.get(ix) {
                    cx.emit(RefMenuEvent::Drop(*d));
                }
            }
            Item::Action { op: None, .. } => {}
        }
    }

    pub fn activate_selected(&mut self, cx: &mut Context<Self>) {
        self.activate(self.selected(), cx);
    }

    /// Left: back to the parent level; Escape on the first level closes.
    pub fn back(&mut self, cx: &mut Context<Self>) -> bool {
        if self.levels.len() > 1 {
            self.levels.pop();
            self.selected.pop();
            cx.notify();
            true
        } else {
            false
        }
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        cx.emit(RefMenuEvent::Close);
    }

    fn key(&mut self, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        let ks = &ev.keystroke;
        if ks.modifiers.control || ks.modifiers.alt || ks.modifiers.platform {
            return;
        }
        match ks.key.as_str() {
            "up" => self.move_by(-1, cx),
            "down" => self.move_by(1, cx),
            "enter" | "right" => self.activate_selected(cx),
            "left" => {
                self.back(cx);
            }
            "escape" if !self.back(cx) => self.close(cx),
            "escape" => {}
            _ => {}
        }
        cx.stop_propagation();
    }
}

impl Focusable for RefMenu {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for RefMenu {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let me = cx.entity().downgrade();
        let selected = self.selected();
        let why = self.refusal(selected);
        let rows = self.items().iter().enumerate().map(|(ix, item)| {
            let me = me.clone();
            match item {
                Item::Separator => div()
                    .h(px(1.0))
                    .my(px(3.0))
                    .bg(skin.border)
                    .into_any_element(),
                Item::Action {
                    label,
                    disabled,
                    danger,
                    key,
                    ..
                } => {
                    let color = if disabled.is_some() {
                        skin.muted
                    } else if *danger {
                        skin.danger
                    } else {
                        skin.text
                    };
                    row(
                        ix,
                        label,
                        key.as_deref(),
                        false,
                        color,
                        ix == selected,
                        &skin,
                        me,
                    )
                }
                Item::Submenu { label, .. } => {
                    row(ix, label, None, true, skin.text, ix == selected, &skin, me)
                }
            }
        });
        v_flex()
            .id("ref-menu")
            .key_context("GpMenu")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, ev, _, cx| this.key(ev, cx)))
            .on_mouse_down_out(cx.listener(|this, _, _, cx| this.close(cx)))
            .min_w(px(260.0))
            .max_w(px(460.0))
            .py(px(4.0))
            .bg(skin.bg)
            .border_1()
            .border_color(skin.border)
            .rounded(px(4.0))
            .shadow_md()
            .text_sm()
            .when_some(self.title.clone(), |d, t| {
                d.child(
                    div()
                        .px_3()
                        .pb(px(3.0))
                        .text_xs()
                        .text_color(skin.muted)
                        .child(SharedString::from(t)),
                )
            })
            .children(rows)
            .when_some(why, |d, why| {
                d.child(
                    div()
                        .px_3()
                        .pt(px(3.0))
                        .text_xs()
                        .text_color(skin.muted)
                        .child(SharedString::from(why)),
                )
            })
    }
}

#[allow(clippy::too_many_arguments)]
fn row(
    ix: usize,
    label: &str,
    key: Option<&str>,
    submenu: bool,
    color: gpui_kit::Hsla,
    selected: bool,
    skin: &Skin,
    me: gpui_kit::WeakEntity<RefMenu>,
) -> gpui_kit::AnyElement {
    let (hover, sel, muted) = (skin.hover, skin.selection, skin.muted);
    gpui_kit::component::h_flex()
        .id(("ref-menu-row", ix))
        .h(px(26.0))
        .px_3()
        .gap_3()
        .items_center()
        .text_color(color)
        .cursor_pointer()
        .when(selected, |d| d.bg(sel))
        .hover(move |d| d.bg(hover))
        .child(
            div()
                .flex_1()
                .truncate()
                .child(SharedString::from(label.to_string())),
        )
        .when_some(key.map(str::to_string), |d, k| {
            d.child(div().flex_none().text_xs().text_color(muted).child(k))
        })
        .when(submenu, |d| {
            d.child(div().flex_none().text_color(muted).child("\u{25B8}"))
        })
        .on_click(move |_, _, cx| {
            let _ = me.update(cx, |this, cx| this.activate(ix, cx));
        })
        .into_any_element()
}
