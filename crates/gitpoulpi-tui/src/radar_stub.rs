//! `radar` for a build with no hosting provider (custom builds, the
//! same idea as `hosting_ui_stub.rs`): the surface the event loop calls, all
//! of it inert. There is no Radar view; the palette commands say why.
#![allow(dead_code)]

#[path = "radar/pending.rs"]
pub(crate) mod pending;
#[path = "radar/vocab.rs"]
mod vocab;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;

pub(crate) use vocab::{
    Act, ColumnCmd, ConversationRequest, Outcome, RadarCommand, ReviewRequest, Spot,
};

/// Never open in this build.
#[derive(Debug, Clone, Default)]
pub(crate) struct RadarSlot;

impl RadarSlot {
    pub(crate) fn is_open(&self) -> bool {
        false
    }

    pub(crate) fn open(&mut self, _session: &str) {}

    pub(crate) fn close(&mut self) {}

    pub(crate) fn typing(&self) -> bool {
        false
    }

    pub(crate) fn animating(&self) -> bool {
        false
    }

    pub(crate) fn next_deadline(&self) -> Option<std::time::Instant> {
        None
    }

    pub(crate) fn poll(&mut self) -> bool {
        false
    }

    pub(crate) fn on_key(&mut self, _key: KeyEvent) -> Outcome {
        Outcome::None
    }

    pub(crate) fn pointer_left(&mut self) {}

    pub(crate) fn on_mouse(&mut self, _mouse: MouseEvent, _double: bool) -> Outcome {
        Outcome::None
    }

    pub(crate) fn command(&mut self, _command: RadarCommand) -> Outcome {
        Outcome::None
    }

    pub(crate) fn render(&mut self, _frame: &mut ratatui::Frame, _body: Rect) {}

    pub(crate) fn resize_pointer(&self) -> bool {
        false
    }

    pub(crate) fn pointer_text_at(&self) -> bool {
        false
    }

    pub(crate) fn pointer_hand_at(&self, _col: u16, _row: u16) -> bool {
        false
    }

    pub(crate) fn sizing_column(&self) -> bool {
        false
    }

    pub(crate) fn popup_open(&self) -> bool {
        false
    }

    pub(crate) fn row_selected(&self) -> bool {
        false
    }
}

impl RadarSlot {
    pub(crate) fn take_said(&mut self) -> Option<String> {
        None
    }

    pub(crate) fn hosts_review(&self) -> bool {
        false
    }

    #[cfg(all(feature = "review", feature = "hosting"))]
    pub(crate) fn host_review(&mut self, _view: Box<crate::review_view::ReviewView>) {}

    pub(crate) fn review_takes_palette(&self) -> bool {
        false
    }

    pub(crate) fn review_avail(&self) -> Option<crate::review_cmd::ReviewAvail> {
        None
    }

    pub(crate) fn run_review(&mut self, _cmd: crate::review_cmd::ReviewCmd) -> Outcome {
        Outcome::None
    }

    pub(crate) fn hosts_item(&self) -> bool {
        false
    }

    pub(crate) fn host_item(&mut self, _view: Box<crate::item_view::ItemView>) {}

    pub(crate) fn item_takes_palette(&self) -> bool {
        false
    }

    pub(crate) fn item_avail(&self) -> Option<crate::item_cmd::ItemAvail> {
        None
    }

    pub(crate) fn run_item(&mut self, _cmd: crate::item_cmd::ItemCmd) -> Outcome {
        Outcome::None
    }

    pub(crate) fn paste(&mut self, _text: &str) {}

    pub(crate) fn nav_targets(&self) -> Vec<(Rect, Spot)> {
        Vec::new()
    }

    pub(crate) fn nav_focus(&mut self, _spot: Spot) -> Outcome {
        Outcome::None
    }

    pub(crate) fn set_mode_hint(&mut self, _hint: Option<String>) {}
}
