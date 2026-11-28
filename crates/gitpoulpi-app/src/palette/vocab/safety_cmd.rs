use super::safety_panel::Tab;
use gitpoulpi_i18n::msg::undo_safety as m;

/// One palette row of the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyCmd {
    UndoTo,
    RedoTo,
    UndoSequence,
    RestoreOnly,
    Offer,
    HistoryRefresh,
    ReflogShow,
    ReflogBranch,
    ReflogTag,
    ReflogApply,
    ReflogStash,
    ReflogRestore,
    ReflogMore,
    ReflogRefresh,
    LostBranch,
    LostTag,
    LostPick,
    LostApply,
    LostStash,
    LostRefresh,
    /// Closes the panel, whatever view is open.
    Close,
    /// Reflog and Lost work: the keyboard goes to the next pane (`Tab`).
    NextPane,
    /// Reflog and Lost work: Path / Tree for the file list (`v`).
    PathTree,
    /// Reflog and Lost work: the dividers back to their default places
    /// (`Alt+0`).
    ResetPanes,
    /// `Dialog: Move or Resize` (`Ctrl+N m`): the arrange mode.
    ArrangeDialog,
    /// `Dialog: Reset Position and Size`.
    ResetDialog,
}

impl SafetyCmd {
    /// The view the row belongs to (`None`: any view).
    pub fn tab(self) -> Option<Tab> {
        match self {
            Self::UndoTo
            | Self::RedoTo
            | Self::UndoSequence
            | Self::RestoreOnly
            | Self::Offer
            | Self::HistoryRefresh => Some(Tab::History),
            Self::ReflogShow
            | Self::ReflogBranch
            | Self::ReflogTag
            | Self::ReflogApply
            | Self::ReflogStash
            | Self::ReflogRestore
            | Self::ReflogMore
            | Self::ReflogRefresh => Some(Tab::Reflog),
            Self::LostBranch
            | Self::LostTag
            | Self::LostPick
            | Self::LostApply
            | Self::LostStash
            | Self::LostRefresh => Some(Tab::Lost),
            Self::Close
            | Self::NextPane
            | Self::PathTree
            | Self::ResetPanes
            | Self::ArrangeDialog
            | Self::ResetDialog => None,
        }
    }

    /// The rows that need the panes (Reflog or Lost work, not History).
    pub fn needs_panes(self) -> bool {
        matches!(self, Self::NextPane | Self::PathTree | Self::ResetPanes)
    }

    /// Why the palette greys the row when `open` (the view on screen, if
    /// any) is not the one it belongs to.
    pub fn unavailable(self, open: Option<Tab>) -> Option<&'static str> {
        // The dialog frame's rows belong to every framed dialog (and to the
        // palette itself), not to this panel.
        if matches!(self, Self::ArrangeDialog | Self::ResetDialog) {
            return None;
        }
        let need = |tab: Tab| match tab {
            Tab::History => m::WHY_OPEN_HISTORY.get(),
            Tab::Reflog => m::WHY_OPEN_REFLOG.get(),
            Tab::Lost => m::WHY_OPEN_LOST.get(),
        };
        if self.needs_panes() {
            return match open {
                Some(Tab::Reflog | Tab::Lost) => None,
                Some(Tab::History) => Some(m::WHY_OPEN_PANES.get()),
                None => Some(need(Tab::History)),
            };
        }
        match (self.tab(), open) {
            // Close needs the panel, not a view.
            (None, Some(_)) => None,
            (None, None) => Some(need(Tab::History)),
            (Some(tab), Some(open)) if tab == open => None,
            (Some(tab), _) => Some(need(tab)),
        }
    }
}
