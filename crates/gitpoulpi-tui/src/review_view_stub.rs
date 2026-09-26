//! The in-app code review in a build without the `review` feature (or
//! without a hosting provider): the surface `repo_dialogs` and `hosting_ui`
//! name, with nothing behind it. `ReviewTarget` and `ReviewView` have no
//! value, so the compiler proves no review can ever open. `supports` says no
//! for every provider, which is what already hides the row menu entry and the
//! Enter-on-a-row review for a provider without the view.
#![allow(dead_code)]

use std::sync::Arc;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::daemon_link::Requester;
use crate::hosting_task::ClientFactory;
use crate::review_cmd::{ReviewAvail, ReviewCmd};

/// Whether the review view exists for this provider: not in this build.
pub(crate) fn supports(_kind: gitpoulpi_config::IntegrationKind) -> bool {
    false
}

/// What the review would open on: never built here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReviewTarget {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReviewEffect {
    None,
    Close,
    Message(String),
    Checkout(u64),
}

/// Never open in this build.
pub(crate) enum ReviewView {}

impl ReviewView {
    pub(crate) fn open(
        target: ReviewTarget,
        _factory: Arc<dyn ClientFactory>,
        _requester: Requester,
        _ai_enabled: bool,
        _session: String,
    ) -> Self {
        match target {}
    }

    pub(crate) fn open_api(
        target: ReviewTarget,
        _factory: Arc<dyn ClientFactory>,
        _ai_enabled: bool,
        _session: String,
        _config_dir: Option<std::path::PathBuf>,
    ) -> Self {
        match target {}
    }

    pub(crate) fn busy(&self) -> bool {
        match *self {}
    }

    pub(crate) fn takes_palette(&self) -> bool {
        match *self {}
    }

    pub(crate) fn avail(&self) -> ReviewAvail {
        match *self {}
    }

    pub(crate) fn run(&mut self, _cmd: ReviewCmd) -> ReviewEffect {
        match *self {}
    }

    pub(crate) fn paste(&mut self, _text: &str) {
        match *self {}
    }

    pub(crate) fn on_key(&mut self, _key: KeyEvent) -> ReviewEffect {
        match *self {}
    }

    pub(crate) fn on_mouse(&mut self, _mouse: MouseEvent) -> ReviewEffect {
        match *self {}
    }

    pub(crate) fn poll(&mut self) -> ReviewEffect {
        match *self {}
    }

    pub(crate) fn render_all(&mut self, _area: Rect, _buf: &mut Buffer) {
        match *self {}
    }
}
