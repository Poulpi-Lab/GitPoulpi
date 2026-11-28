//! What the commands need to know about the window: which pane has the keyboard (the key context a
//! keystroke is resolved in) and the facts the palette's availability rules read.

use gitpoulpi_app::keymap::Ctx;
use gitpoulpi_app::palette::vocab::undo::Avail;
use gitpoulpi_app::palette::{PaletteContext, PaletteSummary};
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
    /// A file of the Unstaged / Staged list is selected (or has the cursor).
    pub unstaged_selected: bool,
    pub staged_selected: bool,
    /// Some file is conflicted: git refuses to commit.
    pub has_conflicts: bool,
    /// The open diff is one of the two working-tree halves (its hunks can be staged), and which one.
    pub stageable_diff: bool,
    pub diff_stages: bool,
    /// What is selected in the Refs sidebar.
    pub refs: crate::ref_target::SelectionFacts,
    /// What the journal offers for Undo and Redo.
    pub undo: Avail,
    pub redo: Avail,
    /// The view the undo-safety panel is open on (`None`: it is closed).
    pub safety_tab: Option<gitpoulpi_app::palette::vocab::safety_panel::Tab>,
    /// A blocked undo entry offers a branch to restore; the newest entries are one sequence run's.
    pub undo_offer: bool,
    pub undo_sequence: bool,
    /// The saved or last grouped push.
    pub group: gitpoulpi_app::palette::vocab::push_group::GroupState,
}

impl Default for WindowState {
    fn default() -> Self {
        WindowState {
            pane: Ctx::Global,
            diff_open: false,
            selected_commit: false,
            has_wip: false,
            unstaged_selected: false,
            staged_selected: false,
            has_conflicts: false,
            stageable_diff: false,
            diff_stages: false,
            refs: Default::default(),
            undo: Avail::Nothing,
            redo: Avail::Nothing,
            safety_tab: None,
            undo_offer: false,
            undo_sequence: false,
            group: Default::default(),
        }
    }
}

impl WindowState {
    /// The palette's availability context for this window.
    pub fn palette_context(&self) -> PaletteContext {
        // What the shared summary covers comes from it; the rest is what only the desktop's panes know.
        PaletteContext {
            selected_local_branch: self.refs.local_branch,
            selected_remote_branch: self.refs.remote_branch,
            selected_tag: self.refs.tag,
            selected_tag_is_unpushed: self.refs.tag_unpushed,
            selected_tag_remote_only: self.refs.tag_remote_only,
            selected_tag_differs: self.refs.tag_differs,
            selected_stash: self.refs.stash,
            selected_folder: self.refs.folder,
            has_stash: self.refs.has_stash,
            safety_tab: self.safety_tab,
            undo_offer: self.undo_offer,
            undo_sequence: self.undo_sequence,
            group: self.group,
            ..PaletteContext::from_summary(PaletteSummary {
                settings_open: false,
                in_refs_pane: self.pane == Ctx::Refs,
                in_diff_view: self.diff_open,
                in_file_view: false,
                selected_commit: self.selected_commit,
                has_wip: self.has_wip,
                unstaged_file_selected: self.unstaged_selected,
                staged_file_selected: self.staged_selected,
                commit_file_selected: false,
                has_conflicts: self.has_conflicts,
                in_stageable_diff: self.stageable_diff,
                diff_stages: self.diff_stages,
                undo: self.undo,
                redo: self.redo,
                open_repository_tabs: 1,
                current_tab_index: 0,
            })
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
            ..WindowState::default()
        };
        let ctx = state.palette_context();
        assert!(ctx.in_refs_pane && ctx.in_diff_view && ctx.selected_commit && ctx.has_wip);
        assert_eq!(ctx.undo, Avail::Ready);
        let ctx = WindowState::default().palette_context();
        assert!(!ctx.in_refs_pane && !ctx.selected_commit);
    }
}
