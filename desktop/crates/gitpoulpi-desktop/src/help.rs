//! The keybindings view: every bound key of the shared keymap, grouped by context the way the
//! terminal UI's Settings page groups them (global first, then each pane, view and form).
//!
//! The data is [`lines`], pure and tested; [`open`] shows it in a dialog with a filter field and a
//! close button.

use std::rc::Rc;

use gitpoulpi_app::keymap::{self, Ctx, Keymap, ACTIONS};
use gitpoulpi_i18n::msg;
use gpui_kit::component::{
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex, WindowExt as _,
};
use gpui_kit::{
    div, px, uniform_list, App, AppContext as _, Context, Entity, Focusable as _, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, Subscription, UniformListScrollHandle,
    Window,
};

use crate::theme::Skin;

/// One line of the view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    /// A context's title.
    Heading(String),
    /// A bound action: its label, its keys (all of them, joined) and its stable id.
    Binding {
        label: String,
        keys: String,
        id: String,
    },
}

/// The lines for `filter` (matched against the label, the id, the keys and the context's title, in the
/// UI language or in English).
pub fn lines(km: &Keymap, filter: &str) -> Vec<Line> {
    let needle = filter.trim().to_lowercase();
    let mut out = Vec::new();
    for ctx in Ctx::ALL {
        let title = ctx.title().get().to_string();
        let title_en = ctx.title().english().to_lowercase();
        let mut rows = Vec::new();
        for def in ACTIONS.iter().filter(|d| d.ctx == *ctx && d.built()) {
            let keys = km.shown_labels(def.action);
            if keys.is_empty() {
                continue;
            }
            let (label, id) = (def.display_label().into_owned(), def.config_id());
            let keys = keys.join(" / ");
            let hit = needle.is_empty()
                || [
                    label.to_lowercase(),
                    def.english_label().to_lowercase(),
                    id.to_lowercase(),
                    keys.to_lowercase(),
                    title.to_lowercase(),
                    title_en.clone(),
                ]
                .iter()
                .any(|h| h.contains(&needle));
            if hit {
                rows.push(Line::Binding {
                    label,
                    keys,
                    id: id.to_string(),
                });
            }
        }
        if !rows.is_empty() {
            out.push(Line::Heading(title));
            out.extend(rows);
        }
    }
    out
}

/// The title of the view: the terminal UI's name for it.
pub fn title() -> &'static str {
    msg::settings_screen::TAB_KEYBINDINGS.get()
}

pub struct HelpView {
    input: Entity<InputState>,
    lines: Rc<Vec<Line>>,
    scroll: UniformListScrollHandle,
    _subs: Vec<Subscription>,
}

impl HelpView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx).placeholder(msg::keys_page::SEARCH_PLACEHOLDER.get())
        });
        let subs = vec![cx.subscribe_in(
            &input,
            window,
            |this: &mut HelpView, _, ev: &InputEvent, _, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.refresh(cx);
                }
            },
        )];
        let mut this = Self {
            input,
            lines: Rc::new(Vec::new()),
            scroll: UniformListScrollHandle::new(),
            _subs: subs,
        };
        this.refresh(cx);
        this
    }

    fn refresh(&mut self, cx: &mut Context<Self>) {
        let filter = self.input.read(cx).value().to_string();
        self.lines = Rc::new(lines(&keymap::current(), &filter));
        cx.notify();
    }

    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn set_filter(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.input
            .update(cx, |i, cx| i.set_value(text.to_string(), window, cx));
        self.refresh(cx);
    }
}

impl Render for HelpView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let lines = self.lines.clone();
        let list = uniform_list("help-lines", lines.len(), {
            let skin = skin.clone();
            let lines = lines.clone();
            move |range, _window, _cx| {
                range
                    .map(|ix| match &lines[ix] {
                        Line::Heading(title) => div()
                            .h(px(28.0))
                            .px_3()
                            .pt(px(6.0))
                            .text_sm()
                            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                            .text_color(skin.accent)
                            .child(SharedString::from(title.clone()))
                            .into_any_element(),
                        Line::Binding { label, keys, id } => h_flex()
                            .h(px(24.0))
                            .px_3()
                            .gap_3()
                            .items_center()
                            .text_sm()
                            .text_color(skin.text)
                            .child(
                                div()
                                    .flex_1()
                                    .truncate()
                                    .child(SharedString::from(label.clone())),
                            )
                            .child(
                                div()
                                    .flex_none()
                                    .text_color(skin.muted)
                                    .text_xs()
                                    .child(SharedString::from(id.clone())),
                            )
                            .child(
                                div()
                                    .flex_none()
                                    .w(px(190.0))
                                    .truncate()
                                    .text_right()
                                    .child(SharedString::from(keys.clone())),
                            )
                            .into_any_element(),
                    })
                    .collect()
            }
        })
        .flex_1()
        .min_h_0()
        .track_scroll(&self.scroll);
        let body = if lines.is_empty() {
            div()
                .flex_1()
                .p_3()
                .text_sm()
                .text_color(skin.muted)
                .child(msg::keys_page::NO_MATCH.get())
                .into_any_element()
        } else {
            list.into_any_element()
        };
        v_flex()
            .w_full()
            .child(
                div()
                    .px_3()
                    .pt_2()
                    .pr(px(40.0))
                    .text_sm()
                    .text_color(skin.muted)
                    .child(title()),
            )
            .child(
                div()
                    .p_2()
                    .border_b_1()
                    .border_color(skin.border)
                    .child(Input::new(&self.input).cleanable(true)),
            )
            .child(div().h(px(440.0)).flex().child(body))
    }
}

/// Shows the keybindings in a dialog.
pub fn open(window: &mut Window, cx: &mut App) -> Entity<HelpView> {
    let view = cx.new(|cx| HelpView::new(window, cx));
    window.open_dialog(cx, {
        let view = view.clone();
        move |dialog, _, _| {
            let view = view.clone();
            dialog
                .close_button(true)
                .overlay_closable(true)
                .w(px(760.0))
                .p_0()
                .content(move |content, _, _| content.child(view.clone()))
        }
    });
    let focus = view.clone();
    window.defer(cx, move |window, cx| {
        let handle = focus.read(cx).input.read(cx).focus_handle(cx);
        handle.focus(window, cx);
    });
    view
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bindings_are_grouped_by_context_global_first() {
        let lines = lines(&Keymap::defaults(), "");
        assert_eq!(
            lines.first(),
            Some(&Line::Heading(Ctx::Global.title().get().to_string()))
        );
        let headings = lines
            .iter()
            .filter(|l| matches!(l, Line::Heading(_)))
            .count();
        assert!(headings > 10, "{headings} contexts");
        assert!(lines.contains(&Line::Binding {
            label: gitpoulpi_i18n::msg::keymap_action::GLOBAL_QUIT
                .get()
                .to_string(),
            keys: "q".to_string(),
            id: "global.quit".to_string(),
        }));
    }

    #[test]
    fn the_filter_matches_label_id_and_keys() {
        let km = Keymap::defaults();
        for needle in ["quit", "global.quit", "ctrl+p"] {
            assert!(
                lines(&km, needle).iter().any(|l| matches!(
                    l,
                    Line::Binding { id, .. } if id == "global.quit" || id == "global.palette"
                )),
                "{needle}"
            );
        }
        assert!(lines(&km, "no such shortcut anywhere").is_empty());
    }

    #[test]
    fn a_remapped_key_shows_in_the_view() {
        let km = keymap::test_keymap(&[("global", "quit", &["Ctrl+Q"])]);
        assert!(lines(&km, "global.quit").contains(&Line::Binding {
            label: gitpoulpi_i18n::msg::keymap_action::GLOBAL_QUIT
                .get()
                .to_string(),
            keys: "Ctrl+Q".to_string(),
            id: "global.quit".to_string(),
        }));
    }
}
