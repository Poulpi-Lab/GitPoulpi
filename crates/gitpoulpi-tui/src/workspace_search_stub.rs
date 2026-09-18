//! The global search (`Ctrl+Alt+F`, the title bar's Global search button) in
//! a build without the `global-search` feature: the surface `modes` and the
//! event loop name, with nothing behind it. `WorkspaceSearch` has no value
//! (it holds an `Infallible`), so the compiler proves the dialog can never
//! open; its palette command, key and button are tagged or not drawn.
#![allow(dead_code)]

use std::convert::Infallible;
use std::time::Instant;

use crossterm::event::KeyEvent;
use gitpoulpi_ipc::ui::UiIntent;

/// Never open in this build.
pub(crate) struct WorkspaceSearch {
    pub(crate) draft: String,
    pub(crate) selected: usize,
    never: Infallible,
}

/// A result row; none exists here.
pub(crate) enum Row {}

impl WorkspaceSearch {
    pub(crate) fn rows(&self) -> &[Row] {
        match self.never {}
    }

    pub(crate) fn is_heading(&self, _row: usize) -> bool {
        match self.never {}
    }

    pub(crate) fn edited(&mut self) {
        match self.never {}
    }

    pub(crate) fn move_to(&mut self, _to: usize) {
        match self.never {}
    }

    pub(crate) fn toggle_chip(&mut self, _slot: usize) {
        match self.never {}
    }

    pub(crate) fn on_chip_key(&mut self, _key: &KeyEvent) -> bool {
        match self.never {}
    }

    pub(crate) fn tick(&mut self, _now: Instant) {
        match self.never {}
    }

    pub(crate) fn animating(&self) -> bool {
        match self.never {}
    }

    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        match self.never {}
    }

    pub(crate) fn go_intent(&self) -> Option<UiIntent> {
        match self.never {}
    }

    pub(crate) fn review_request(&self) -> Option<(std::path::PathBuf, u64)> {
        match self.never {}
    }
}

pub(crate) fn close_at(_col: u16, _row: u16) -> bool {
    false
}

pub(crate) fn chip_at(_col: u16, _row: u16) -> Option<usize> {
    None
}

pub(crate) fn render(
    _frame: &mut ratatui::Frame,
    search: &WorkspaceSearch,
    _field_editor: &crate::text_edit::FieldEditor<crate::modes::MainField>,
) -> crate::chrome::PalettePopup {
    match search.never {}
}
