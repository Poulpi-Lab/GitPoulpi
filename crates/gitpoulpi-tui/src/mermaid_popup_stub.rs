//! The Copy as Mermaid gitGraph popup in a build without the `mermaid`
//! feature: the surface the event loop calls, all of it inert. Nothing opens
//! it (the menu entry, the palette command and the key are all tagged).
#![allow(dead_code)]

use std::path::PathBuf;

use crossterm::event::{KeyEvent, MouseEvent};
use gitpoulpi_core::domain::{CommitGraph, Oid, Ref};
use ratatui::Frame;

/// What the popup asks of the event loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Outcome {
    None,
    Copy(String),
    CopyDiagram(String),
}

pub(crate) fn open(
    _graph: &CommitGraph,
    _refs: &[Ref],
    _selected: &[Oid],
    _cursor: Oid,
) -> Result<(), String> {
    Err("Mermaid is not built into this binary".to_string()) // i18n-english
}

pub(crate) fn open_script(_text: &str) {}

pub(crate) fn is_open() -> bool {
    false
}

pub(crate) fn on_key(_key: KeyEvent, _config_path: &Option<PathBuf>) -> Outcome {
    Outcome::None
}

pub(crate) fn on_mouse(_mouse: MouseEvent, _config_path: &Option<PathBuf>) -> Outcome {
    Outcome::None
}

pub(crate) fn render(_frame: &mut Frame) {}
