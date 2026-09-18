//! The image view in a build without the `image-view` feature: the same
//! entry points as `image_view/`, all of them doing nothing, so the File
//! View, the keys, the mouse and the palette call them unconditionally (the
//! pattern of `hosting_ui_stub.rs`). Nothing here decodes or draws a picture,
//! so a binary without the feature links none of the engine.

#[allow(dead_code)]
pub(crate) mod pane {
    use crossterm::event::{KeyEvent, MouseEventKind};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::Frame;

    use crate::chrome::HitTarget;
    use crate::daemon_link::{DaemonLink, FileInsightOutcome};
    use crate::file_view::FileInsight;
    use crate::terminal_image::Capability;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) enum ImageHit {
        ZoomIn,
        ZoomOut,
        Fit,
        ActualSize,
        Flip(super::layout::Side),
        Divider,
        EditIn,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(crate) enum ZoomOp {
        In,
        Out,
        Fit,
        Actual,
    }

    pub(crate) struct Painted {
        pub(crate) hits: Vec<(Rect, HitTarget)>,
    }

    pub(crate) fn init(_capability: Capability) {}
    pub(crate) fn finish_frame(_buf: &Buffer) {}
    pub(crate) fn write_pending<W: std::io::Write>(_out: &mut W) -> std::io::Result<()> {
        Ok(())
    }
    pub(crate) fn active() -> bool {
        false
    }
    pub(crate) fn animating() -> bool {
        false
    }
    pub(crate) fn shows(_insight: &FileInsight) -> bool {
        false
    }
    pub(crate) fn sync(_insight: Option<&FileInsight>, _link: &DaemonLink) {}
    pub(crate) fn offer(outcome: FileInsightOutcome) -> Option<FileInsightOutcome> {
        Some(outcome)
    }
    pub(crate) fn render(_frame: &mut Frame, _body: Rect, _inner: Rect) -> Painted {
        Painted { hits: Vec::new() }
    }
    pub(crate) fn set_hovered(_target: Option<&HitTarget>) {}
    pub(crate) fn run_zoom(_op: ZoomOp) -> bool {
        false
    }
    pub(crate) fn on_key(_km: &crate::keymap::Keymap, _key: &KeyEvent) -> bool {
        false
    }
    pub(crate) fn dragging() -> bool {
        false
    }
    pub(crate) fn on_mouse(
        _kind: MouseEventKind,
        _col: u16,
        _row: u16,
        _mods: crossterm::event::KeyModifiers,
    ) -> bool {
        false
    }
    impl ImageHit {
        pub(crate) fn op(self) -> Option<ZoomOp> {
            match self {
                ImageHit::ZoomIn => Some(ZoomOp::In),
                ImageHit::ZoomOut => Some(ZoomOp::Out),
                ImageHit::Fit => Some(ZoomOp::Fit),
                ImageHit::ActualSize => Some(ZoomOp::Actual),
                ImageHit::Flip(_) | ImageHit::Divider | ImageHit::EditIn => None,
            }
        }
    }
    pub(crate) fn click(_hit: ImageHit) {}
}

/// "Edit in" has no picture to edit without the feature: no menu offers it.
#[allow(dead_code)]
pub(crate) mod edit_in {
    use std::path::{Path, PathBuf};

    use crossterm::event::{KeyEvent, MouseEvent};
    use ratatui::layout::Rect;
    use ratatui::Frame;

    use crate::chrome::HitTarget;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) enum Target {
        Pane,
        File(PathBuf),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum Pick {
        Entry(usize),
        Configure,
    }

    pub(crate) fn is_open() -> bool {
        false
    }
    pub(crate) fn open(_target: Target) {}
    pub(crate) fn request_when_ready() {}
    pub(crate) fn on_key(_key: KeyEvent) -> Option<Pick> {
        None
    }
    pub(crate) fn on_mouse(_mouse: MouseEvent) -> Option<Pick> {
        None
    }
    pub(crate) fn paste(_text: &str) {}
    pub(crate) fn render_popup(
        _frame: &mut Frame,
        _hovered_row: Option<usize>,
        _hits: &mut Vec<(Rect, HitTarget)>,
    ) {
    }
    pub(crate) fn run_entry(_index: usize, _repo_root: &Path) -> String {
        String::new()
    }
    pub(crate) fn run_default(_target: &Target, _repo_root: &Path) -> String {
        String::new()
    }
    pub(crate) fn is_image_path(_path: &Path) -> bool {
        false
    }
}

#[allow(dead_code)]
pub(crate) mod layout {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) enum Side {
        Before,
        After,
    }
}

#[allow(dead_code)]
pub(crate) mod split {
    use crossterm::event::{KeyEvent, KeyModifiers, MouseEventKind};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::Frame;

    use super::layout::Side;
    use super::pane::ZoomOp;
    use crate::chrome::HitTarget;
    use crate::daemon_link::{FileInsightOutcome, FileInsightRequest};
    use crate::diff_view::DiffViewState;
    use crate::terminal_image::Capability;
    use gitpoulpi_core::domain::FileDiff;

    /// No previews in the merge editor's cards.
    pub(crate) mod thumbs {
        pub(crate) fn wanted(_path: &std::path::Path) -> bool {
            false
        }
        pub(crate) fn follow(
            _path: &std::path::Path,
            _stages: &[(u8, gitpoulpi_core::domain::Oid)],
        ) {
        }
        pub(crate) fn paint(
            _buf: &mut ratatui::buffer::Buffer,
            _area: ratatui::layout::Rect,
            _stage: u8,
        ) {
        }
    }

    pub(crate) fn init(_capability: Capability) {}
    pub(crate) fn refresh_worktree(_repo_root: &std::path::Path) {}
    pub(crate) fn begin_frame() {}
    pub(crate) fn finish_frame(_buf: &Buffer) {}
    pub(crate) fn write_pending<W: std::io::Write>(_out: &mut W) -> std::io::Result<()> {
        Ok(())
    }
    pub(crate) fn take_requests() -> Vec<FileInsightRequest> {
        Vec::new()
    }
    pub(crate) fn active() -> bool {
        false
    }
    pub(crate) fn split_shown() -> bool {
        false
    }
    pub(crate) fn animating() -> bool {
        false
    }
    pub(crate) fn dragging() -> bool {
        false
    }
    pub(crate) fn set_hovered(_target: Option<&HitTarget>) {}
    pub(crate) fn wants(_state: &DiffViewState, _file: &FileDiff) -> bool {
        false
    }
    pub(crate) fn offer(outcome: FileInsightOutcome) -> Option<FileInsightOutcome> {
        Some(outcome)
    }
    pub(crate) fn render(
        _frame: &mut Frame,
        _inner: Rect,
        _st: &DiffViewState,
        _file: &FileDiff,
    ) -> Option<Vec<(Rect, HitTarget)>> {
        None
    }
    pub(crate) fn run_zoom(_op: ZoomOp) -> bool {
        false
    }
    pub(crate) fn flip() -> bool {
        false
    }
    pub(crate) fn show_side(_side: Side) {}
    pub(crate) fn edit_file(
        _repo_root: &std::path::Path,
    ) -> Result<(std::path::PathBuf, Option<String>), String> {
        Err(String::new())
    }
    pub(crate) fn widen(_direction: i32) -> bool {
        false
    }
    pub(crate) fn reset_split() -> bool {
        false
    }
    pub(crate) fn on_key(_km: &crate::keymap::Keymap, _key: &KeyEvent) -> bool {
        false
    }
    pub(crate) fn on_mouse(
        _kind: MouseEventKind,
        _col: u16,
        _row: u16,
        _mods: KeyModifiers,
    ) -> bool {
        false
    }
}
