//! The Refs sidebar: Filter box, then the LOCAL, REMOTE, WORKTREES, STASHES and TAGS sections, each a
//! boxed list with a chevron, an icon, its title and a count, folders split on `/`, a check mark on the
//! checked-out branch, an arrow on tags that are also on the remote. Colours follow the theme
//! (`refs.local` / `refs.remote` / `refs.tag`).

mod model;

use std::collections::HashMap;
use std::rc::Rc;

use gitpoulpi_core::domain::Oid;
use gitpoulpi_i18n::msg;
use gpui_kit::assets::IconName as AssetIcon;
use gpui_kit::component::{
    h_flex,
    input::{Input, InputEvent, InputState},
    list::ListItem,
    tree::{tree, TreeItem, TreeState},
    v_flex, Icon, Sizable as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable,
    FontWeight, Hsla, InteractiveElement as _, IntoElement, MouseButton, ParentElement as _,
    Render, SharedString, StatefulInteractiveElement as _, Styled as _, Subscription, Task, Window,
};
use model::{Kind, Meta};

use crate::daemon::Shared;
use crate::theme::Skin;

pub enum RefsEvent {
    Selected(Oid),
}

struct Section {
    kind: Kind,
    open: bool,
    count: usize,
    rows: usize,
    state: Entity<TreeState>,
    /// Unfiltered items.
    items: Vec<TreeItem>,
}

pub struct RefsPanel {
    backend: Shared,
    sections: Vec<Section>,
    meta: Rc<HashMap<SharedString, Meta>>,
    filter: Entity<InputState>,
    focus: FocusHandle,
    _task: Option<Task<()>>,
    _subs: Vec<Subscription>,
}

impl EventEmitter<RefsEvent> for RefsPanel {}

fn section_icon(kind: Kind) -> AssetIcon {
    match kind {
        Kind::Local => AssetIcon::Laptop,
        Kind::Remote => AssetIcon::Globe,
        Kind::Worktrees => AssetIcon::Leaf,
        Kind::Stashes => AssetIcon::Package,
        Kind::Tags => AssetIcon::Tag,
    }
}

const ROW_H: f32 = 24.0;

impl RefsPanel {
    pub fn new(backend: Shared, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let filter =
            cx.new(|cx| InputState::new(window, cx).placeholder(msg::sidebar::FILTER_HINT.get()));
        let subs = vec![cx.subscribe(&filter, |this, input, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::Change) {
                this.apply_filter(input.read(cx).value().to_string(), cx);
            }
        })];
        let sections = Kind::ALL
            .into_iter()
            .map(|kind| Section {
                kind,
                open: true,
                count: 0,
                rows: 0,
                state: cx.new(|cx| TreeState::new(cx)),
                items: Vec::new(),
            })
            .collect();
        let mut this = Self {
            backend,
            sections,
            meta: Rc::default(),
            filter,
            focus: cx.focus_handle(),
            _task: None,
            _subs: subs,
        };
        this.reload(cx);
        this
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let refs = self.backend.refs();
        let stashes = self.backend.stashes();
        let worktrees = self.backend.worktrees();
        self._task = Some(cx.spawn(async move |this, cx| {
            let refs = refs.await.unwrap_or_default();
            let stashes = stashes.await.unwrap_or_default();
            let worktrees = worktrees.await.unwrap_or_default();
            let built = model::build(&refs, &stashes, &worktrees);
            let _ = this.update(cx, |this, cx| this.install(built, cx));
        }));
    }

    fn install(&mut self, built: model::SectionItems, cx: &mut Context<Self>) {
        for ((sec, items), count) in self.sections.iter_mut().zip(built.items).zip(built.counts) {
            sec.items = items;
            sec.count = count;
        }
        self.meta = Rc::new(built.meta);
        let q = self.filter.read(cx).value().to_string();
        self.apply_filter(q, cx);
    }

    fn apply_filter(&mut self, q: String, cx: &mut Context<Self>) {
        let q = q.to_lowercase();
        for sec in &mut self.sections {
            let items = if q.is_empty() {
                sec.items.clone()
            } else {
                model::filter_items(&sec.items, &q)
            };
            sec.rows = model::count_rows(&items);
            sec.state.update(cx, |s, cx| s.set_items(items, cx));
        }
        cx.notify();
    }

    pub fn focus_filter(&self, window: &mut Window, cx: &mut App) {
        self.filter.read(cx).focus_handle(cx).focus(window, cx);
    }

    fn section_box(
        &self,
        si: usize,
        colors: &[Hsla; 5],
        skin: &Skin,
        cx: &mut Context<Self>,
    ) -> gpui_kit::Div {
        let sec = &self.sections[si];
        let (border, text, muted) = (skin.border, skin.text, skin.muted);
        let kind = sec.kind;
        let color = colors[si];
        let folder_color = skin.warning;
        let tall = sec.rows > 9;
        let open = sec.open;
        let meta = self.meta.clone();
        let me = cx.entity().downgrade();
        let body_h = (sec.rows.max(1) as f32 * ROW_H + 8.0).min(240.0);
        let tree_el = tree(&sec.state, move |ix, entry, sel, _window, _cx| {
            let item = entry.item();
            let m = meta.get(&item.id).cloned();
            let depth = entry.depth();
            let folder = m.as_ref().map(|m| m.folder).unwrap_or(false) || entry.is_folder();
            let is_head = m.as_ref().map(|m| m.is_head).unwrap_or(false);
            let tag_remote = m.as_ref().map(|m| m.tag_remote).unwrap_or(false);
            let target = m.as_ref().and_then(|m| m.target);
            let me = me.clone();
            ListItem::new(ix)
                .selected(sel)
                .w_full()
                .h(px(ROW_H))
                .child(
                    h_flex()
                        .w_full()
                        .h_full()
                        .items_center()
                        .gap(px(5.0))
                        .pl(px(8.0 + depth as f32 * 14.0))
                        .child(
                            div()
                                .w(px(12.0))
                                .flex_none()
                                .text_xs()
                                .text_color(text)
                                .child(if is_head { "\u{2713}" } else { "" }),
                        )
                        .when(folder, |d| {
                            d.child(
                                Icon::new(if entry.is_expanded() {
                                    AssetIcon::FolderOpen
                                } else {
                                    AssetIcon::Folder
                                })
                                .size(px(14.0))
                                .text_color(folder_color),
                            )
                        })
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .text_sm()
                                .text_color(if folder { text } else { color })
                                .when(is_head, |d| d.font_weight(FontWeight::BOLD))
                                .child(item.label.clone()),
                        )
                        .when(tag_remote, |d| {
                            d.child(
                                Icon::new(AssetIcon::ArrowUp)
                                    .size(px(11.0))
                                    .text_color(muted),
                            )
                            .child(div().w(px(4.0)))
                        }),
                )
                .on_click(move |_, _, cx| {
                    if let Some(oid) = target {
                        let _ = me.update(cx, |_, cx| cx.emit(RefsEvent::Selected(oid)));
                    }
                })
        })
        .size_full();
        let header = h_flex()
            .id(("sec-head", si))
            .flex_none()
            .h(px(28.0))
            .px(px(8.0))
            .gap(px(6.0))
            .items_center()
            .on_click(cx.listener(move |this, _, _, cx| {
                if let Some(s) = this.sections.get_mut(si) {
                    s.open = !s.open;
                }
                cx.notify();
            }))
            .child(
                Icon::new(if open {
                    AssetIcon::ChevronDown
                } else {
                    AssetIcon::ChevronRight
                })
                .size(px(12.0))
                .text_color(muted),
            )
            .child(
                Icon::new(section_icon(kind))
                    .size(px(15.0))
                    .text_color(color),
            )
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(text)
                    .child(format!("{} ({})", kind.title(), sec.count)),
            )
            .child(div().flex_1())
            .when(kind == Kind::Remote, |d| {
                d.child(Icon::new(AssetIcon::Plus).size(px(13.0)).text_color(muted))
            });
        let mut boxed = v_flex()
            .w_full()
            .border_1()
            .border_color(border)
            .rounded(px(3.0))
            .overflow_hidden()
            .child(header);
        if open && sec.rows > 0 {
            boxed = if tall {
                boxed
                    .flex_1()
                    .min_h(px(120.0))
                    .child(div().flex_1().min_h_0().child(tree_el))
            } else {
                boxed.flex_none().child(div().h(px(body_h)).child(tree_el))
            };
        }
        boxed
    }
}

impl Focusable for RefsPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for RefsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let colors = [skin.local, skin.remote, skin.local, skin.warning, skin.tag];
        let mut sections = v_flex().flex_1().min_h_0().gap(px(6.0)).overflow_hidden();
        for si in 0..self.sections.len() {
            sections = sections.child(self.section_box(si, &colors, &skin, cx));
        }
        v_flex()
            .size_full()
            .bg(skin.side_bg)
            .p(px(6.0))
            .gap(px(6.0))
            .child(
                h_flex()
                    .flex_none()
                    .h(px(20.0))
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(skin.text)
                            .child(msg::sidebar::TITLE_REFS.get()),
                    )
                    .child(
                        Icon::new(AssetIcon::ChevronLeft)
                            .size(px(14.0))
                            .text_color(skin.muted),
                    ),
            )
            .child(
                v_flex()
                    .flex_none()
                    .border_1()
                    .border_color(skin.border)
                    .rounded(px(3.0))
                    .p(px(4.0))
                    .gap(px(2.0))
                    .child(
                        div()
                            .px(px(4.0))
                            .text_xs()
                            .text_color(skin.muted)
                            .child(msg::sidebar::FILTER_TITLE.get()),
                    )
                    .child(Input::new(&self.filter).small().cleanable(true)),
            )
            .child(sections)
            .on_mouse_down(MouseButton::Left, |_, _, _| {})
    }
}
