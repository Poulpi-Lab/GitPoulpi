//! `trust_dialog` for a build with no hosting provider: there is no
//! integration to check, so the dialog never opens.
#![allow(dead_code)]

use std::path::Path;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::Frame;

use crate::preferences::PreferencesState;

/// An answer that never exists in this build.
pub(crate) enum Outcome {}

pub(crate) fn is_open() -> bool {
    false
}

pub(crate) fn feed() {}

pub(crate) fn on_key(_key: KeyEvent) -> Option<Outcome> {
    None
}

pub(crate) fn on_mouse(_mouse: MouseEvent) -> Option<Outcome> {
    None
}

pub(crate) fn render(_frame: &mut Frame) {}

pub(crate) fn open_review() -> bool {
    false
}

pub(crate) fn take_changed() -> bool {
    false
}

pub(crate) fn apply(
    outcome: Outcome,
    _repo_root: &Path,
    _preferences: &mut Option<PreferencesState>,
    _message: &mut Option<String>,
) {
    match outcome {}
}
