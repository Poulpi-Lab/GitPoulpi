//! The conversation view in a build without the `conversation` feature: the
//! surface `repo_dialogs` and `hosting_ui` name, with nothing behind it.
//! `ItemTarget` and `ItemView` have no value, so the compiler proves no item
//! view can ever open, and every entry point keeps the browser.
#![allow(dead_code)]

use std::sync::Arc;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::hosting_task::ClientFactory;
use crate::item_cmd::{ItemAvail, ItemCmd};

/// What the view would open on: never built here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ItemTarget {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ItemEffect {
    None,
    Close,
    Message(String),
    OpenUrl(String),
    ShowCommit(String),
}

/// No provider's issues have a conversation to read in this build.
pub(crate) fn supports_issues(_kind: gitpoulpi_config::IntegrationKind) -> bool {
    false
}

/// No provider's requests have a conversation to read in this build.
pub(crate) fn supports_requests(_kind: gitpoulpi_config::IntegrationKind) -> bool {
    false
}

/// Never open in this build.
pub(crate) enum ItemView {}

impl ItemView {
    pub(crate) fn open(target: ItemTarget, _factory: Arc<dyn ClientFactory>) -> Self {
        match target {}
    }

    pub(crate) fn busy(&self) -> bool {
        match *self {}
    }

    pub(crate) fn next_deadline(&self) -> Option<std::time::Instant> {
        match *self {}
    }

    pub(crate) fn takes_palette(&self) -> bool {
        match *self {}
    }

    pub(crate) fn avail(&self) -> ItemAvail {
        match *self {}
    }

    pub(crate) fn run(&mut self, _cmd: ItemCmd) -> ItemEffect {
        match *self {}
    }

    pub(crate) fn on_key(&mut self, _key: KeyEvent) -> ItemEffect {
        match *self {}
    }

    pub(crate) fn on_mouse(&mut self, _mouse: MouseEvent) -> ItemEffect {
        match *self {}
    }

    pub(crate) fn paste(&mut self, _text: &str) {
        match *self {}
    }

    pub(crate) fn on_hosting_changed(&mut self, _change: &gitpoulpi_ipc::HostingChangeArgs) {
        match *self {}
    }

    pub(crate) fn poll(&mut self) -> ItemEffect {
        match *self {}
    }

    pub(crate) fn render(&mut self, _area: Rect, _buf: &mut Buffer) {
        match *self {}
    }

    pub(crate) fn nav_targets(&self) -> Vec<(Rect, (u16, u16))> {
        match *self {}
    }
}
