//! Small forms in a dialog: a title, single-line text fields and check boxes, a reason line when the
//! entry is refused, Cancel and a confirming button. The branch, tag, rename, stash and upstream forms
//! are all this view over a different spec.
//!
//! Keyboard: Tab moves between the fields and buttons, Enter in a field submits, Escape and the close
//! control cancel. A refused entry keeps the dialog open with the reason under the fields; nothing is
//! submitted until the validation passes.

use std::rc::Rc;

use gpui_kit::component::{
    checkbox::Checkbox,
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex, WindowExt as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, App, AppContext as _, Context, Entity, FocusHandle, Focusable, Global,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Subscription, WeakEntity, Window,
};

use crate::theme::Skin;

/// One field of a form.
#[derive(Clone, Debug)]
pub enum Field {
    Text {
        label: String,
        placeholder: String,
        value: String,
    },
    Check {
        label: String,
        checked: bool,
    },
}

impl Field {
    pub fn text(label: impl Into<String>, placeholder: impl Into<String>, value: &str) -> Self {
        Field::Text {
            label: label.into(),
            placeholder: placeholder.into(),
            value: value.to_string(),
        }
    }

    pub fn check(label: impl Into<String>, checked: bool) -> Self {
        Field::Check {
            label: label.into(),
            checked,
        }
    }
}

/// What a form shows.
#[derive(Clone, Debug)]
pub struct FormSpec {
    pub title: String,
    /// Lines under the title (what the form acts on).
    pub intro: Vec<String>,
    pub fields: Vec<Field>,
    /// The confirming button.
    pub submit: String,
    pub cancel: String,
}

/// What the fields hold when the form is submitted, in the spec's order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Values {
    pub texts: Vec<String>,
    pub checks: Vec<bool>,
}

impl Values {
    /// The `i`th text field's value, trimmed.
    pub fn text(&self, i: usize) -> &str {
        self.texts.get(i).map(|t| t.trim()).unwrap_or("")
    }

    pub fn check(&self, i: usize) -> bool {
        self.checks.get(i).copied().unwrap_or(false)
    }
}

pub type Validate = Rc<dyn Fn(&Values) -> Result<(), String>>;
type Submitted = Box<dyn FnOnce(Values, &mut Window, &mut App)>;

struct Open(WeakEntity<FormView>);
impl Global for Open {}

pub struct FormView {
    spec: FormSpec,
    inputs: Vec<Option<Entity<InputState>>>,
    checks: Vec<bool>,
    error: Option<String>,
    touched: bool,
    validate: Validate,
    on_submit: Option<Submitted>,
    focus: FocusHandle,
    _subs: Vec<Subscription>,
}

impl FormView {
    fn new(
        spec: FormSpec,
        validate: Validate,
        on_submit: Submitted,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut inputs = Vec::new();
        let mut checks = Vec::new();
        let mut subs = Vec::new();
        for field in &spec.fields {
            match field {
                Field::Text {
                    placeholder, value, ..
                } => {
                    let (placeholder, value) = (placeholder.clone(), value.clone());
                    let input = cx.new(|cx| {
                        let mut s = InputState::new(window, cx).placeholder(placeholder);
                        s.set_value(value, window, cx);
                        s
                    });
                    subs.push(cx.subscribe_in(
                        &input,
                        window,
                        |this: &mut FormView, _, ev: &InputEvent, window, cx| match ev {
                            InputEvent::Change => {
                                this.touched = true;
                                this.revalidate(cx);
                            }
                            InputEvent::PressEnter { .. } => this.submit(window, cx),
                            _ => {}
                        },
                    ));
                    inputs.push(Some(input));
                    checks.push(false);
                }
                Field::Check { checked, .. } => {
                    inputs.push(None);
                    checks.push(*checked);
                }
            }
        }
        Self {
            spec,
            inputs,
            checks,
            error: None,
            touched: false,
            validate,
            on_submit: Some(on_submit),
            focus: cx.focus_handle(),
            _subs: subs,
        }
    }

    fn revalidate(&mut self, cx: &mut Context<Self>) {
        self.error = (self.validate)(&self.values(cx)).err();
        cx.notify();
    }

    /// What the fields hold now.
    pub fn values(&self, cx: &App) -> Values {
        Values {
            texts: self
                .inputs
                .iter()
                .map(|i| {
                    i.as_ref()
                        .map(|i| i.read(cx).value().to_string())
                        .unwrap_or_default()
                })
                .collect(),
            checks: self.checks.clone(),
        }
    }

    /// Types `text` into the `ix`th field (tests, and prefilling).
    pub fn set_text(&mut self, ix: usize, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(Some(input)) = self.inputs.get(ix) {
            input.update(cx, |i, cx| i.set_value(text.to_string(), window, cx));
        }
    }

    pub fn set_check(&mut self, ix: usize, on: bool, cx: &mut Context<Self>) {
        if let Some(c) = self.checks.get_mut(ix) {
            *c = on;
            cx.notify();
        }
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn spec(&self) -> &FormSpec {
        &self.spec
    }

    /// Enter or the confirming button: validates, then hands the values over and closes.
    pub fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let values = self.values(cx);
        if let Err(why) = (self.validate)(&values) {
            self.error = Some(why);
            self.touched = true;
            cx.notify();
            return;
        }
        let Some(done) = self.on_submit.take() else {
            return;
        };
        window.close_dialog(cx);
        // After the dialog is gone: the answer may open another one.
        window.defer(cx, move |window, cx| done(values, window, cx));
    }

    fn cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.on_submit = None;
        window.close_dialog(cx);
    }
}

impl Focusable for FormView {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.inputs
            .iter()
            .flatten()
            .next()
            .map(|i| i.read(cx).focus_handle(cx))
            .unwrap_or_else(|| self.focus.clone())
    }
}

impl Render for FormView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let me = cx.entity().downgrade();
        let mut body = v_flex().gap_2();
        for (ix, field) in self.spec.fields.iter().enumerate() {
            match field {
                Field::Text { label, .. } => {
                    if let Some(input) = &self.inputs[ix] {
                        body = body.child(
                            v_flex()
                                .gap_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(skin.muted)
                                        .child(SharedString::from(label.clone())),
                                )
                                .child(Input::new(input).cleanable(true)),
                        );
                    }
                }
                Field::Check { label, .. } => {
                    let me = me.clone();
                    body = body.child(
                        Checkbox::new(("form-check", ix))
                            .label(label.clone())
                            .checked(self.checks[ix])
                            .on_click(move |on, _, cx| {
                                let on = *on;
                                let _ = me.update(cx, |this, cx| this.set_check(ix, on, cx));
                            }),
                    );
                }
            }
        }
        let button = |id: &'static str, label: String, primary: bool| {
            div()
                .id(id)
                .px_3()
                .h(px(28.0))
                .flex()
                .items_center()
                .rounded(px(3.0))
                .border_1()
                .border_color(if primary { skin.accent } else { skin.border })
                .text_sm()
                .text_color(skin.text)
                .cursor_pointer()
                .hover(|d| d.bg(skin.hover))
                .child(label)
        };
        let (me_ok, me_cancel) = (me.clone(), me.clone());
        v_flex()
            .key_context("GpForm")
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
            .children(
                self.spec
                    .intro
                    .iter()
                    .map(|l| div().text_sm().child(SharedString::from(l.clone()))),
            )
            .child(body)
            .when_some(self.error.clone(), |d, why| {
                d.child(
                    div()
                        .id("form-error")
                        .text_sm()
                        .text_color(skin.danger)
                        .child(why),
                )
            })
            .child(
                h_flex()
                    .gap_2()
                    .justify_end()
                    .child(
                        button("form-cancel", self.spec.cancel.clone(), false).on_click(
                            move |_, window, cx| {
                                let _ = me_cancel.update(cx, |this, cx| this.cancel(window, cx));
                            },
                        ),
                    )
                    .child(
                        button("form-submit", self.spec.submit.clone(), true).on_click(
                            move |_, window, cx| {
                                let _ = me_ok.update(cx, |this, cx| this.submit(window, cx));
                            },
                        ),
                    ),
            )
    }
}

/// Opens the form. `on_submit` runs with the values once the dialog is gone and `validate` passed.
pub fn open(
    window: &mut Window,
    cx: &mut App,
    spec: FormSpec,
    validate: impl Fn(&Values) -> Result<(), String> + 'static,
    on_submit: impl FnOnce(Values, &mut Window, &mut App) + 'static,
) -> Entity<FormView> {
    let view = cx.new(|cx| FormView::new(spec, Rc::new(validate), Box::new(on_submit), window, cx));
    cx.set_global(Open(view.downgrade()));
    let held = view.clone();
    window.open_dialog(cx, move |dialog, _, _| {
        let view = held.clone();
        dialog
            .close_button(true)
            .overlay_closable(true)
            .w(px(460.0))
            .p_0()
            .content(move |content, _, _| content.child(view.clone()))
    });
    let focus = view.clone();
    window.defer(cx, move |window, cx| {
        let handle = focus.read(cx).focus_handle(cx);
        handle.focus(window, cx);
    });
    view
}

/// The open form, if any.
pub fn current(cx: &App) -> Option<Entity<FormView>> {
    cx.try_global::<Open>().and_then(|o| o.0.upgrade())
}
