//! The one-line message a command leaves in the status line ("not available yet", a pending prefix key).
//!
//! Kept deliberately small: a global holding the text, a setter and a getter. The status line reads
//! [`current`] when it draws; whoever owns the status line's model can fold this into it.

use gpui_kit::{App, Global, SharedString};

#[derive(Default)]
struct Message(Option<SharedString>);
impl Global for Message {}

/// Shows `text` in the status line until the next message or [`clear`].
pub fn set(cx: &mut App, text: impl Into<SharedString>) {
    cx.set_global(Message(Some(text.into())));
    cx.refresh_windows();
}

/// Removes the message.
pub fn clear(cx: &mut App) {
    if current(cx).is_some() {
        cx.set_global(Message(None));
        cx.refresh_windows();
    }
}

/// The message to show, if any.
pub fn current(cx: &App) -> Option<SharedString> {
    cx.try_global::<Message>().and_then(|m| m.0.clone())
}
