//! Question dialogs: a title, some lines, and numbered answers. The discard confirmation and the
//! sensitive-files question are both this view.
//!
//! Keyboard: `1`..`9` answer, Up / Down move the highlight, Enter answers the highlighted one (the
//! dialog's own "confirm" binding is taken over), `y` / `n`
//! answer the first / last when the dialog asks for it (a yes-or-no question), Escape, the close
//! control and a click outside dismiss it. With `highlight_first` off no answer is highlighted until an
//! arrow key or a digit is pressed, so a stray Enter answers nothing (the sensitive-files rule of the
//! terminal UI).

use gpui_kit::base::actions::Confirm;
use gpui_kit::component::{h_flex, v_flex, WindowExt as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, App, AppContext as _, Context, Entity, FocusHandle, Focusable, Global,
    InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, WeakEntity, Window,
};

use crate::theme::Skin;

/// One answer button.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    pub label: String,
    /// Drawn in the danger colour (a discard).
    pub destructive: bool,
}

impl Choice {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            destructive: false,
        }
    }

    pub fn destructive(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            destructive: true,
        }
    }
}

/// What the dialog says and offers.
#[derive(Clone, Debug)]
pub struct Spec {
    pub title: String,
    /// The body: one line each (a file with its reason, a sentence).
    pub lines: Vec<String>,
    pub choices: Vec<Choice>,
    /// A footer line with the keys.
    pub hint: String,
    /// Highlight the first answer at once (a plain confirmation).
    pub highlight_first: bool,
    /// `y` answers the first choice and `n` the last.
    pub yes_no_keys: bool,
}

type Answered = Box<dyn FnOnce(usize, &mut App)>;
type Dismissed = Box<dyn FnOnce()>;

/// The dialog that is open, for the tests and for [`answer_open`].
struct Open(WeakEntity<ChoiceView>);
impl Global for Open {}

pub struct ChoiceView {
    spec: Spec,
    highlighted: Option<usize>,
    on_answer: Option<Answered>,
    on_dismiss: Option<Dismissed>,
    focus: FocusHandle,
}

impl Drop for ChoiceView {
    /// Closed by Escape, the close control or a click outside: the question was not answered.
    fn drop(&mut self) {
        if self.on_answer.take().is_some() {
            if let Some(dismissed) = self.on_dismiss.take() {
                dismissed();
            }
        }
    }
}

impl ChoiceView {
    fn answer(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        if ix >= self.spec.choices.len() {
            return;
        }
        let Some(answered) = self.on_answer.take() else {
            return;
        };
        self.on_dismiss = None;
        window.close_dialog(cx);
        // After the dialog is gone, like the pickers: the answer may open another one.
        window.defer(cx, move |_, cx| answered(ix, cx));
    }

    fn step(&mut self, delta: isize, cx: &mut Context<Self>) {
        let n = self.spec.choices.len();
        if n == 0 {
            return;
        }
        self.highlighted = Some(match self.highlighted {
            Some(h) => h.saturating_add_signed(delta).min(n - 1),
            None if delta < 0 => n - 1,
            None => 0,
        });
        cx.notify();
    }

    fn key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let ks = &ev.keystroke;
        if ks.modifiers.control || ks.modifiers.alt || ks.modifiers.platform {
            return;
        }
        let last = self.spec.choices.len().saturating_sub(1);
        match ks.key.as_str() {
            "up" => self.step(-1, cx),
            "down" => self.step(1, cx),
            "y" if self.spec.yes_no_keys => self.answer(0, window, cx),
            "n" if self.spec.yes_no_keys => self.answer(last, window, cx),
            k => {
                if let Some(d) = k.parse::<usize>().ok().filter(|d| *d >= 1) {
                    if d <= self.spec.choices.len() {
                        self.highlighted = Some(d - 1);
                        self.answer(d - 1, window, cx);
                    }
                }
            }
        }
    }

    pub fn spec(&self) -> &Spec {
        &self.spec
    }
}

impl Focusable for ChoiceView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for ChoiceView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let me = cx.entity().downgrade();
        let highlighted = self.highlighted;
        let buttons = self.spec.choices.iter().enumerate().map(|(ix, c)| {
            let me = me.clone();
            let color = if c.destructive {
                skin.danger
            } else {
                skin.text
            };
            div()
                .id(("choice", ix))
                .px_3()
                .h(px(28.0))
                .flex()
                .items_center()
                .rounded(px(3.0))
                .border_1()
                .border_color(if highlighted == Some(ix) {
                    skin.accent
                } else {
                    skin.border
                })
                .text_sm()
                .text_color(color)
                .cursor_pointer()
                .hover(|d| d.bg(skin.hover))
                .when(highlighted == Some(ix), |d| d.bg(skin.selection))
                .child(format!("{}. {}", ix + 1, c.label))
                .on_click(move |_, window, cx| {
                    let _ = me.update(cx, |this, cx| this.answer(ix, window, cx));
                })
        });
        v_flex()
            .key_context("GpChoice")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, ev, window, cx| this.key(ev, window, cx)))
            // The dialog binds Enter to its own "confirm"; here it answers the highlighted choice (and
            // nothing while none is highlighted).
            .on_action(cx.listener(|this, _: &Confirm, window, cx| {
                if let Some(h) = this.highlighted {
                    this.answer(h, window, cx);
                }
            }))
            .w_full()
            .p_3()
            .gap_3()
            .child(
                div()
                    .pr(px(32.0))
                    .text_base()
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(self.spec.title.clone()),
            )
            .child(
                v_flex()
                    .id("choice-lines")
                    .max_h(px(240.0))
                    .overflow_y_scroll()
                    .gap_1()
                    .text_sm()
                    .children(
                        self.spec
                            .lines
                            .iter()
                            .map(|l| div().child(SharedString::from(l.clone()))),
                    ),
            )
            .child(h_flex().gap_2().flex_wrap().children(buttons))
            .child(
                div()
                    .text_xs()
                    .text_color(skin.muted)
                    .child(self.spec.hint.clone()),
            )
    }
}

/// Opens the question. `on_answer` runs with the chosen index once the dialog is gone; `on_dismiss`
/// runs when it was closed without an answer (Escape, the close control, a click outside).
pub fn open(
    window: &mut Window,
    cx: &mut App,
    spec: Spec,
    on_answer: impl FnOnce(usize, &mut App) + 'static,
    on_dismiss: impl FnOnce() + 'static,
) {
    let highlighted = (spec.highlight_first && !spec.choices.is_empty()).then_some(0);
    let view = cx.new(|cx| ChoiceView {
        spec,
        highlighted,
        on_answer: Some(Box::new(on_answer)),
        on_dismiss: Some(Box::new(on_dismiss)),
        focus: cx.focus_handle(),
    });
    cx.set_global(Open(view.downgrade()));
    let held = view.clone();
    window.open_dialog(cx, move |dialog, _, _| {
        let view = held.clone();
        dialog
            .close_button(true)
            .overlay_closable(true)
            .w(px(560.0))
            .p_0()
            .content(move |content, _, _| content.child(view.clone()))
    });
    // The field focus follows once the dialog is drawn.
    let focus = view.read(cx).focus.clone();
    window.defer(cx, move |window, cx| focus.focus(window, cx));
}

/// The open question, if any.
pub fn current(cx: &App) -> Option<Entity<ChoiceView>> {
    cx.try_global::<Open>().and_then(|o| o.0.upgrade())
}

/// Answers the open question with choice `ix` (what a click on it does); `false` when none is open.
pub fn answer_open(ix: usize, window: &mut Window, cx: &mut App) -> bool {
    match current(cx) {
        Some(view) => {
            view.update(cx, |v, cx| v.answer(ix, window, cx));
            true
        }
        None => false,
    }
}
