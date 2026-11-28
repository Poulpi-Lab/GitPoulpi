//! What the commands need to know about the window: which pane has the keyboard (the key context a
//! keystroke is resolved in) and the facts the palette's availability rules read.

use gitpoulpi_app::keymap::Ctx;
use gitpoulpi_app::palette::vocab::undo::Avail;
use gitpoulpi_app::palette::PaletteContext;
use gpui_kit::{App, Window};

/// A snapshot of the window, plain data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowState {
    /// The key context of the pane that has the keyboard; `Ctx::Global` when none does.
    pub pane: Ctx,
    /// The file diff is shown in the centre instead of the Log.
    pub diff_open: bool,
    /// The Log's selection is a real commit (not the WIP row or a stash).
    pub selected_commit: bool,
    /// The working tree has uncommitted changes.
    pub has_wip: bool,
    /// What the journal offers for Undo and Redo.
    pub undo: Avail,
    pub redo: Avail,
}

impl Default for WindowState {
    fn default() -> Self {
        WindowState {
            pane: Ctx::Global,
            diff_open: false,
            selected_commit: false,
            has_wip: false,
            undo: Avail::Nothing,
            redo: Avail::Nothing,
        }
    }
}

impl WindowState {
    /// The palette's availability context for this window.
    pub fn palette_context(&self) -> PaletteContext {
        PaletteContext {
            workspace_active: false,
            open_repository_tabs: 1,
            current_tab_index: 0,
            in_refs_pane: self.pane == Ctx::Refs,
            in_diff_view: self.diff_open,
            selected_commit: self.selected_commit,
            has_wip: self.has_wip,
            undo: self.undo,
            redo: self.redo,
            lfs_program_ok: true,
            ..PaletteContext::default()
        }
    }
}

/// The state of the main window (the default state before it exists).
pub fn window_state(window: &Window, cx: &App) -> WindowState {
    match crate::app::workspace_of(cx) {
        Some(ws) => ws.read(cx).command_state(window, cx),
        None => WindowState::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_context_follows_the_window() {
        let state = WindowState {
            pane: Ctx::Refs,
            diff_open: true,
            selected_commit: true,
            has_wip: true,
            undo: Avail::Ready,
            redo: Avail::Nothing,
        };
        let ctx = state.palette_context();
        assert!(ctx.in_refs_pane && ctx.in_diff_view && ctx.selected_commit && ctx.has_wip);
        assert_eq!(ctx.undo, Avail::Ready);
        let ctx = WindowState::default().palette_context();
        assert!(!ctx.in_refs_pane && !ctx.selected_commit);
    }
}
