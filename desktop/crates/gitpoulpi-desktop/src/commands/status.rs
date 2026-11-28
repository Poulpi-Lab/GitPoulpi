//! The status line's one message: what a command, a pending key chord or an action said last.
//!
//! Kept deliberately small: a global holding the [`Notice`], setters and getters. Commands, the
//! workspace (`say`, the link's results in `app::live`) and the key handling all write here, and the
//! status line reads it when it draws, so there is one place a line can come from.

use gpui_kit::{App, Global, SharedString};

/// One line of the status line; `error` marks a failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    pub text: String,
    pub error: bool,
}

impl Notice {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            error: false,
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            error: true,
        }
    }
}

#[derive(Default)]
struct Message(Option<Notice>);
impl Global for Message {}

/// Shows `text` in the status line until the next message or [`clear`].
pub fn set(cx: &mut App, text: impl Into<String>) {
    set_notice(cx, Notice::info(text));
}

/// Shows `text` as a failure.
pub fn set_error(cx: &mut App, text: impl Into<String>) {
    set_notice(cx, Notice::error(text));
}

/// Shows `notice`.
pub fn set_notice(cx: &mut App, notice: Notice) {
    cx.set_global(Message(Some(notice)));
    cx.refresh_windows();
}

/// Removes the message.
pub fn clear(cx: &mut App) {
    if notice(cx).is_some() {
        cx.set_global(Message(None));
        cx.refresh_windows();
    }
}

/// The message to show, if any.
pub fn notice(cx: &App) -> Option<Notice> {
    cx.try_global::<Message>().and_then(|m| m.0.clone())
}

/// The text of the message to show, if any.
pub fn current(cx: &App) -> Option<SharedString> {
    notice(cx).map(|n| SharedString::from(n.text))
}
