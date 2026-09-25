//! The Kanban board in a build without the `board` feature (or without a
//! hosting provider): the surface `repo_dialogs` and `hosting_ui` name, with
//! nothing behind it. `BoardTarget` and `BoardView` have no value, so the
//! compiler proves no board can ever open; the palette commands, the ISSUES
//! header button and the key are tagged or not offered.
#![allow(dead_code)]

use std::sync::Arc;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::board_cmd::{BoardAvail, BoardCmd};
use crate::hosting_task::ClientFactory;
use crate::issue_form::IssueSeed;

/// What the board would open on: never built here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BoardTarget {}

/// The column a New issue form should start in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NewIn {
    pub(crate) column: Option<String>,
    pub(crate) label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BoardEffect {
    None,
    Close,
    Message(String),
    OpenUrl(String),
    NewIssue(Box<IssueSeed>, NewIn),
}

/// Never open in this build.
pub(crate) enum BoardView {}

impl BoardView {
    pub(crate) fn open(target: BoardTarget, _factory: Arc<dyn ClientFactory>) -> Self {
        match target {}
    }

    pub(crate) fn refresh(&mut self) {
        match *self {}
    }

    pub(crate) fn busy(&self) -> bool {
        match *self {}
    }

    pub(crate) fn takes_palette(&self) -> bool {
        match *self {}
    }

    pub(crate) fn avail(&self) -> BoardAvail {
        match *self {}
    }

    pub(crate) fn run(&mut self, _cmd: BoardCmd) -> BoardEffect {
        match *self {}
    }

    pub(crate) fn paste(&mut self, _text: &str) {
        match *self {}
    }

    pub(crate) fn on_key(&mut self, _key: KeyEvent) -> BoardEffect {
        match *self {}
    }

    pub(crate) fn on_mouse(&mut self, _mouse: MouseEvent) -> BoardEffect {
        match *self {}
    }

    pub(crate) fn poll(&mut self) -> BoardEffect {
        match *self {}
    }

    pub(crate) fn render(&mut self, _area: Rect, _buf: &mut Buffer) {
        match *self {}
    }
}
