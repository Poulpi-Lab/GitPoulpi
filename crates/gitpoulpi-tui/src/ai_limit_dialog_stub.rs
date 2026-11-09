//! `ai_limit_dialog` for a build without the `ai` feature: no request is ever
//! prepared, so the dialog is never open.

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::Frame;

pub(crate) fn is_open() -> bool {
    false
}

pub(crate) fn on_key(_key: KeyEvent) {}

pub(crate) fn on_mouse(_mouse: MouseEvent) {}

pub(crate) fn take_open_setting() -> Option<&'static str> {
    None
}

pub(crate) fn render(_frame: &mut Frame) {}
