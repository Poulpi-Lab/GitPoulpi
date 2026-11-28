//! The searchable list dialog: a filter field over a list of rows, one selected. The command palette
//! and the theme, language and graph colour pickers are all this view over a different row source.
//!
//! Keyboard: type to filter, Up / Down / PageUp / PageDown move, Enter runs, Escape (or the close
//! button) closes. A row can be dimmed: it still runs, but its reason is shown under the list.

use std::rc::Rc;

use gpui_kit::component::{
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex, WindowExt as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, uniform_list, App, AppContext as _, Context, Entity, EventEmitter, Focusable as _,
    Global, InteractiveElement as _, IntoElement, ParentElement as _, Render, ScrollStrategy,
    SharedString, StatefulInteractiveElement as _, Styled as _, Subscription,
    UniformListScrollHandle, Window,
};

use crate::theme::Skin;

pub(super) const CONTEXT: &str = "GpPicker";

/// The picker that has the keyboard: the text field would take Up and Down for itself, so the
/// keystroke interceptor ([`navigate`]) moves the selection instead.
struct ActivePicker(gpui_kit::WeakEntity<Picker>);
impl Global for ActivePicker {}

/// Moves the selection for a navigation key typed while a picker is open: Up / Down, PageUp /
/// PageDown, Ctrl+K / Ctrl+J (the terminal UI's spelling). `true` when the key was taken.
pub fn navigate(key: &str, ctrl: bool, cx: &mut App) -> bool {
    let delta: isize = match (key, ctrl) {
        ("up", false) | ("k", true) => -1,
        ("down", false) | ("j", true) => 1,
        ("pageup", false) => -(PAGE as isize),
        ("pagedown", false) => PAGE as isize,
        _ => return false,
    };
    let Some(picker) = cx.try_global::<ActivePicker>().and_then(|a| a.0.upgrade()) else {
        return false;
    };
    picker.update(cx, |p, cx| p.step(delta, cx));
    true
}

/// What a row does when chosen (after the picker closed).
pub type RunFn = Rc<dyn Fn(&mut Window, &mut App)>;

/// One row of a picker.
#[derive(Clone)]
pub struct PickRow {
    pub label: SharedString,
    /// Right-aligned: the key that runs the same thing, or a value.
    pub shortcut: SharedString,
    /// Why the row cannot do its job right now (shown under the list while it is selected).
    pub reason: Option<SharedString>,
    pub dim: bool,
    pub run: RunFn,
}

/// Rows for a query (already trimmed of nothing: the raw text of the field).
pub type Provider = Rc<dyn Fn(&str, &App) -> Vec<PickRow>>;

/// How many rows a page key jumps.
const PAGE: usize = 10;

/// The texts around the list.
#[derive(Clone)]
pub struct Texts {
    /// The header: `(shown, total)` rows.
    pub title: Rc<dyn Fn(usize, usize) -> String>,
    pub placeholder: SharedString,
    /// Shown when no row matches.
    pub empty: SharedString,
}

pub struct Picker {
    texts: Texts,
    total: usize,
    input: Entity<InputState>,
    provider: Provider,
    rows: Rc<Vec<PickRow>>,
    selected: usize,
    scroll: UniformListScrollHandle,
    _subs: Vec<Subscription>,
}

/// Emitted when a row was chosen, so the host can close its dialog.
pub struct Chosen;
impl EventEmitter<Chosen> for Picker {}

impl Picker {
    pub fn new(
        texts: Texts,
        provider: Provider,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let placeholder = texts.placeholder.clone();
        let total = provider("", cx).len();
        let input = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder));
        let subs = vec![cx.subscribe_in(
            &input,
            window,
            |this: &mut Picker, _, ev: &InputEvent, window, cx| match ev {
                InputEvent::Change => this.refresh(cx),
                InputEvent::PressEnter { .. } => this.confirm(window, cx),
                _ => {}
            },
        )];
        let mut this = Self {
            texts,
            total,
            input,
            provider,
            rows: Rc::new(Vec::new()),
            selected: 0,
            scroll: UniformListScrollHandle::new(),
            _subs: subs,
        };
        this.refresh(cx);
        this
    }

    /// Re-runs the row source for the current text.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        let query = self.input.read(cx).value().to_string();
        self.rows = Rc::new((self.provider)(&query, cx));
        self.selected = 0;
        self.scroll.scroll_to_item(0, ScrollStrategy::Top);
        cx.notify();
    }

    /// Types `text` into the field (tests and "open the palette with a query").
    pub fn set_query(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.input
            .update(cx, |i, cx| i.set_value(text.to_string(), window, cx));
        self.refresh(cx);
    }

    pub fn rows(&self) -> &[PickRow] {
        &self.rows
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    /// The field's focus handle.
    pub fn field_focus(&self, cx: &App) -> gpui_kit::FocusHandle {
        self.input.read(cx).focus_handle(cx)
    }

    /// Moves the selection by `delta` rows, clamped to the list.
    pub fn step(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.rows.is_empty() {
            return;
        }
        let last = self.rows.len() - 1;
        self.selected = self.selected.saturating_add_signed(delta).min(last);
        self.scroll
            .scroll_to_item(self.selected, ScrollStrategy::Nearest);
        cx.notify();
    }

    /// Runs the selected row and tells the host to close.
    pub fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.choose(self.selected, window, cx);
    }

    fn choose(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(row) = self.rows.get(ix) else {
            return;
        };
        let run = row.run.clone();
        cx.emit(Chosen);
        // After the dialog is gone: the row may open another one.
        window.defer(cx, move |window, cx| run(window, cx));
    }
}

impl Render for Picker {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let rows = self.rows.clone();
        let selected = self.selected;
        let me = cx.entity().downgrade();
        let list = uniform_list("picker-rows", rows.len(), {
            let skin = skin.clone();
            let rows = rows.clone();
            move |range, _window, _cx| {
                range
                    .map(|ix| {
                        let row = &rows[ix];
                        let me = me.clone();
                        let (text, muted) = if row.dim {
                            (skin.muted, skin.muted)
                        } else {
                            (skin.text, skin.muted)
                        };
                        h_flex()
                            .id(("pick", ix))
                            .h(px(26.0))
                            .px_3()
                            .gap_3()
                            .items_center()
                            .text_sm()
                            .text_color(text)
                            .when(ix == selected, |d| d.bg(skin.selection))
                            .hover(|d| d.bg(skin.hover))
                            .child(div().flex_1().truncate().child(row.label.clone()))
                            .child(
                                div()
                                    .flex_none()
                                    .text_xs()
                                    .text_color(muted)
                                    .child(row.shortcut.clone()),
                            )
                            .on_click(move |_, window, cx| {
                                let _ = me.update(cx, |this, cx| this.choose(ix, window, cx));
                            })
                    })
                    .collect()
            }
        })
        .flex_1()
        .min_h_0()
        .track_scroll(&self.scroll);

        let reason = rows.get(selected).and_then(|r| r.reason.clone());
        let title = (self.texts.title)(rows.len(), self.total);
        let body = if rows.is_empty() {
            div()
                .flex_1()
                .p_3()
                .text_sm()
                .text_color(skin.muted)
                .child(self.texts.empty.clone())
                .into_any_element()
        } else {
            list.into_any_element()
        };
        v_flex()
            .key_context(CONTEXT)
            .w_full()
            .child(
                div()
                    .px_3()
                    .pt_2()
                    .pr(px(40.0))
                    .text_sm()
                    .text_color(skin.muted)
                    .child(title),
            )
            .child(
                div()
                    .p_2()
                    .border_b_1()
                    .border_color(skin.border)
                    .child(Input::new(&self.input).cleanable(true)),
            )
            .child(div().h(px(360.0)).flex().child(body))
            .child(
                div()
                    .h(px(24.0))
                    .px_3()
                    .flex()
                    .items_center()
                    .border_t_1()
                    .border_color(skin.border)
                    .text_xs()
                    .text_color(skin.muted)
                    .child(reason.unwrap_or_default()),
            )
    }
}

/// Opens a picker in a dialog with a close button; the field has the keyboard.
pub fn open(window: &mut Window, cx: &mut App, texts: Texts, provider: Provider) -> Entity<Picker> {
    let picker = cx.new(|cx| Picker::new(texts, provider, window, cx));
    cx.set_global(ActivePicker(picker.downgrade()));
    let host = picker.clone();
    window
        .subscribe(&picker, cx, |_, _: &Chosen, window, cx| {
            window.close_dialog(cx)
        })
        .detach();
    window.open_dialog(cx, {
        let picker = picker.clone();
        move |dialog, _, _| {
            let picker = picker.clone();
            dialog
                .close_button(true)
                .overlay_closable(true)
                .w(px(680.0))
                .p_0()
                .content(move |content, _, _| content.child(picker.clone()))
        }
    });
    let focus = host.clone();
    window.defer(cx, move |window, cx| {
        let handle = focus.read(cx).field_focus(cx);
        handle.focus(window, cx);
    });
    picker
}
