//! Stand-in for `worktree_dialog.rs` when the `worktree` feature is off
//! (custom builds): the same `pub(crate)` surface, inert. Nothing opens it
//! (the menu rows and palette entries are not offered), so it only exists
//! to keep `repo_dialogs` and the menus compiling unchanged.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use crossterm::event::{KeyEvent, MouseEvent};
use gitpoulpi_core::domain::{WorktreeAddOptions, WorktreeStart};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorktreeSeed {
    pub(crate) start: WorktreeStart,
    pub(crate) label: String,
    pub(crate) new_branch: String,
    pub(crate) branch_busy: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WtFocus {
    Create,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WorktreeOutcome {
    Stay,
    Cancel,
    Browse(PathBuf),
    Submit {
        path: PathBuf,
        opts: WorktreeAddOptions,
        open_tab: bool,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct WorktreeDialog {
    pub(crate) seed: WorktreeSeed,
    pub(crate) focus: WtFocus,
    close: crate::dialog_kit::CloseHit,
}

impl WorktreeDialog {
    pub(crate) fn nav_targets(&self) -> Vec<(ratatui::layout::Rect, (u16, u16))> {
        Vec::new()
    }

    pub(crate) fn new(seed: WorktreeSeed, _repo_root: &Path) -> Self {
        Self {
            seed,
            focus: WtFocus::Create,
            close: Default::default(),
        }
    }

    pub(crate) fn set_parent(&mut self, _dir: &Path) {}

    pub(crate) fn paste(&mut self, _text: &str) {}

    /// Any key closes the (never opened) dialog.
    pub(crate) fn on_key(&mut self, _key: KeyEvent) -> WorktreeOutcome {
        WorktreeOutcome::Cancel
    }

    pub(crate) fn on_mouse(&mut self, _mouse: MouseEvent) -> WorktreeOutcome {
        WorktreeOutcome::Stay
    }

    pub(crate) fn render(&mut self, _rect: Rect, _buf: &mut Buffer) {}
}

impl crate::dialog_kit::Dismiss for WorktreeDialog {
    fn is_dirty(&self) -> bool {
        false
    }

    fn close_hit(&self) -> &crate::dialog_kit::CloseHit {
        &self.close
    }

    fn close_hit_mut(&mut self) -> &mut crate::dialog_kit::CloseHit {
        &mut self.close
    }
}

pub(crate) fn worktree_rect(frame: Rect) -> Rect {
    frame
}
