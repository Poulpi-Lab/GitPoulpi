//! Read-only views of the one hover source of truth: the `HitTarget`
//! under the pointer (`hovered_target`, set from the single
//! `find_hit_target` call a `Moved` event makes). Each helper answers one
//! "is this control lit?" question, so no per-control `hovered_*` flag
//! can drift out of step with the hit test.

use super::*;

/// A borrowed view of `hovered_target`.
#[derive(Clone, Copy)]
pub(super) struct Hover<'a>(&'a Option<HitTarget>);

/// Wraps the current `hovered_target`.
pub(super) fn hover(target: &Option<HitTarget>) -> Hover<'_> {
    Hover(target)
}

impl Hover<'_> {
    pub(super) fn gear(self) -> bool {
        matches!(self.0, Some(HitTarget::Gear))
    }

    pub(super) fn about(self) -> bool {
        matches!(self.0, Some(HitTarget::About))
    }

    pub(super) fn workspace_chip(self) -> bool {
        matches!(self.0, Some(HitTarget::WorkspaceChip))
    }

    pub(super) fn profile_field(self) -> bool {
        matches!(self.0, Some(HitTarget::ProfileField))
    }

    pub(super) fn close_app(self) -> bool {
        matches!(self.0, Some(HitTarget::CloseApp))
    }

    pub(super) fn add_remote(self) -> bool {
        matches!(self.0, Some(HitTarget::AddRemoteButton))
    }

    pub(super) fn add_worktree(self) -> bool {
        matches!(self.0, Some(HitTarget::AddWorktreeButton))
    }

    /// The Refs outer block's right-aligned title button.
    pub(super) fn refs_title_button(self) -> bool {
        matches!(
            self.0,
            Some(HitTarget::StopSoloing) | Some(HitTarget::ShowAllHiddenRefs)
        )
    }

    pub(super) fn close_diff(self) -> bool {
        matches!(self.0, Some(HitTarget::CloseDiff))
    }

    /// The Confirm popup's button row: `true` is the confirm side.
    pub(super) fn confirm(self) -> Option<bool> {
        match self.0 {
            Some(HitTarget::ConfirmButton(confirm)) => Some(*confirm),
            _ => None,
        }
    }

    /// The row of whichever popup menu the pointer is over.
    pub(super) fn menu_row(self) -> Option<usize> {
        match self.0 {
            Some(HitTarget::RefContextMenuItem(i))
            | Some(HitTarget::LogContextMenuItem(i))
            | Some(HitTarget::WipContextMenuItem(i))
            | Some(HitTarget::RemoteMenuItem(i))
            | Some(HitTarget::FileContextMenuItem(i))
            | Some(HitTarget::FolderContextMenuItem(i))
            | Some(HitTarget::BarOverflowMenuItem(i))
            | Some(HitTarget::PaneContextMenuItem(i))
            | Some(HitTarget::DiffHunkContextMenuItem(i))
            | Some(HitTarget::HistoryContextMenuItem(i))
            | Some(HitTarget::CommitColumn(crate::commit_column::ColumnHit::ColumnsMenuItem(i)))
            | Some(HitTarget::Hosting(HostingHit::MenuItem(i)))
            | Some(HitTarget::Hosting(HostingHit::PickerItem(i)))
            | Some(HitTarget::OpenInItem(i))
            | Some(HitTarget::WorkspaceItem(i))
            | Some(HitTarget::ProfileItem(i)) => Some(*i),
            _ => None,
        }
    }

    pub(super) fn column_menu_item(self) -> Option<ChooserRow> {
        match self.0 {
            Some(HitTarget::ColumnMenuItem(col)) => Some(ChooserRow::Column(*col)),
            Some(HitTarget::ColumnMenuAction(row)) => Some(*row),
            _ => None,
        }
    }

    /// The tab-folder button under the pointer.
    pub(super) fn tab_dir(self) -> Option<crate::tab_dir::TabDirButton> {
        match self.0 {
            Some(HitTarget::TabDir(button)) => Some(*button),
            _ => None,
        }
    }

    pub(super) fn folder(self) -> bool {
        matches!(self.0, Some(HitTarget::OpenRepositoryButton))
    }

    pub(super) fn radar_tab(self) -> bool {
        matches!(self.0, Some(HitTarget::RadarTab))
    }

    pub(super) fn radar_close(self) -> bool {
        matches!(self.0, Some(HitTarget::RadarClose))
    }

    pub(super) fn workspaces_tab(self) -> bool {
        matches!(self.0, Some(HitTarget::WorkspacesTab))
    }

    pub(super) fn workspaces_close(self) -> bool {
        matches!(self.0, Some(HitTarget::WorkspacesClose))
    }

    pub(super) fn notes_close(self) -> bool {
        matches!(self.0, Some(HitTarget::NotesClose))
    }

    pub(super) fn global_search(self) -> bool {
        matches!(self.0, Some(HitTarget::GlobalSearchButton))
    }

    pub(super) fn new_tab(self) -> bool {
        matches!(self.0, Some(HitTarget::NewTabButton))
    }

    pub(super) fn tab_overflow(self) -> bool {
        matches!(self.0, Some(HitTarget::TabOverflow))
    }

    pub(super) fn tab_scroll_left(self) -> bool {
        matches!(self.0, Some(HitTarget::TabScrollLeft))
    }

    pub(super) fn tab_scroll_right(self) -> bool {
        matches!(self.0, Some(HitTarget::TabScrollRight))
    }

    pub(super) fn pane_collapse(self) -> Option<PaneSide> {
        match self.0 {
            Some(HitTarget::CollapsePane(side)) => Some(*side),
            _ => None,
        }
    }

    pub(super) fn pane_restore(self) -> Option<PaneSide> {
        match self.0 {
            Some(HitTarget::RestorePane(side)) => Some(*side),
            _ => None,
        }
    }

    pub(super) fn diff_button(self) -> Option<DiffBarButton> {
        match self.0 {
            Some(HitTarget::DiffBarButton(button)) => Some(*button),
            _ => None,
        }
    }

    pub(super) fn amend_checkbox(self) -> bool {
        matches!(self.0, Some(HitTarget::AmendCheckbox))
    }

    pub(super) fn skip_hooks_checkbox(self) -> bool {
        matches!(self.0, Some(HitTarget::SkipHooksCheckbox))
    }

    pub(super) fn commit_button(self) -> bool {
        matches!(self.0, Some(HitTarget::CommitButton))
    }

    pub(super) fn ai_generate(self) -> bool {
        matches!(self.0, Some(HitTarget::AiGenerate))
    }

    pub(super) fn ai_compose(self) -> bool {
        matches!(self.0, Some(HitTarget::AiCompose))
    }

    pub(super) fn issue_ref(self) -> bool {
        matches!(self.0, Some(HitTarget::ComposerIssueRef))
    }
}
