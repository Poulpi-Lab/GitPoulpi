//! The command-palette twins of the panel's row actions: the same `Act`s the
//! buttons, the right-click menu and the letters run, listed in the palette
//! while the panel is open on the matching view, acting on its selected row.
//! A row that cannot do the action says why (status line and the panel's own
//! note) instead of silently doing nothing.

use gitpoulpi_core::domain::JournalState;
use gitpoulpi_i18n::msg::undo_safety as m;

use super::{Act, PanelOutcome, SafetyPanel, Tab};

/// One palette row of the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SafetyCmd {
    UndoTo,
    RedoTo,
    RestoreOnly,
    Offer,
    HistoryRefresh,
    ReflogShow,
    ReflogBranch,
    ReflogRestore,
    ReflogMore,
    ReflogRefresh,
    LostShow,
    LostBranch,
    LostPick,
    LostRefresh,
    /// Closes the panel, whatever view is open.
    Close,
}

impl SafetyCmd {
    /// The view the row belongs to (`None`: any view).
    pub(crate) fn tab(self) -> Option<Tab> {
        match self {
            Self::UndoTo
            | Self::RedoTo
            | Self::RestoreOnly
            | Self::Offer
            | Self::HistoryRefresh => Some(Tab::History),
            Self::ReflogShow
            | Self::ReflogBranch
            | Self::ReflogRestore
            | Self::ReflogMore
            | Self::ReflogRefresh => Some(Tab::Reflog),
            Self::LostShow | Self::LostBranch | Self::LostPick | Self::LostRefresh => {
                Some(Tab::Lost)
            }
            Self::Close => None,
        }
    }

    /// The row action it runs (`None` for a refresh and Close).
    pub(crate) fn act(self) -> Option<Act> {
        Some(match self {
            Self::UndoTo => Act::UndoTo,
            Self::RedoTo => Act::RedoTo,
            Self::RestoreOnly => Act::RestoreOnly,
            Self::Offer => Act::Offer,
            Self::ReflogShow | Self::LostShow => Act::ShowInLog,
            Self::ReflogBranch | Self::LostBranch => Act::CreateBranch,
            Self::ReflogRestore => Act::RestoreRef,
            Self::ReflogMore => Act::LoadMore,
            Self::LostPick => Act::CherryPick,
            Self::HistoryRefresh | Self::ReflogRefresh | Self::LostRefresh | Self::Close => {
                return None
            }
        })
    }

    /// Why the palette greys the row when `open` (the view on screen, if
    /// any) is not the one it belongs to.
    pub(crate) fn unavailable(self, open: Option<Tab>) -> Option<&'static str> {
        let need = |tab: Tab| match tab {
            Tab::History => m::WHY_OPEN_HISTORY.get(),
            Tab::Reflog => m::WHY_OPEN_REFLOG.get(),
            Tab::Lost => m::WHY_OPEN_LOST.get(),
        };
        match (self.tab(), open) {
            // Close needs the panel, not a view.
            (None, Some(_)) => None,
            (None, None) => Some(need(Tab::History)),
            (Some(tab), Some(open)) if tab == open => None,
            (Some(tab), _) => Some(need(tab)),
        }
    }
}

impl SafetyPanel {
    /// Whether Ctrl+P may open the palette over the panel: not while a
    /// name is being typed into it.
    pub(crate) fn takes_palette(&self) -> bool {
        !matches!(self.overlay, Some(super::Overlay::Name(_)))
    }

    /// Runs a palette row on the selected row of the open view. The second
    /// half is the reason when nothing could be done.
    pub(crate) fn run_command(
        &mut self,
        cmd: SafetyCmd,
        state: &JournalState,
    ) -> (PanelOutcome, Option<String>) {
        if cmd == SafetyCmd::Close {
            return (PanelOutcome::Close, None);
        }
        if let Some(why) = cmd.unavailable(Some(self.tab)) {
            return (PanelOutcome::Stay, Some(why.to_string()));
        }
        if self.job.is_some() {
            return self.refuse(m::WHY_BUSY.get().to_string());
        }
        // A palette row replaces whatever small dialog was up.
        self.overlay = None;
        let Some(act) = cmd.act() else {
            self.say(m::REFRESHING.get().to_string());
            self.reload_current();
            return (PanelOutcome::Stay, None);
        };
        if !self.actions(state).contains(&act) {
            let why = self.why_not(act);
            return self.refuse(why);
        }
        (self.run(act, state), None)
    }

    fn refuse(&mut self, why: String) -> (PanelOutcome, Option<String>) {
        self.complain(why.clone());
        (PanelOutcome::Stay, Some(why))
    }

    /// Why `act` is not offered on the selected row of the open view.
    fn why_not(&self, act: Act) -> String {
        match self.tab {
            Tab::History => {
                let Some(row) = self.selected_history() else {
                    return m::WHY_NO_ROW.get().to_string();
                };
                match act {
                    Act::UndoTo => m::WHY_ALREADY_UNDONE.get().to_string(),
                    Act::RedoTo => m::WHY_NOT_UNDONE.get().to_string(),
                    Act::RestoreOnly if row.redo => m::WHY_RESTORE_REDO.get().to_string(),
                    Act::RestoreOnly => match row.entry.restore.not_restorable.as_deref() {
                        Some(why) => m::detail_not_restorable(why),
                        None => m::WHY_NO_RESTORE.get().to_string(),
                    },
                    Act::Offer if self.history.selected_row() == Some(0) && !row.redo => {
                        m::WHY_NO_OFFER.get().to_string()
                    }
                    Act::Offer => m::WHY_OFFER_NEWEST.get().to_string(),
                    _ => m::WHY_NO_ROW.get().to_string(),
                }
            }
            Tab::Reflog | Tab::Lost => {
                let has_row = match self.tab {
                    Tab::Reflog => self.selected_reflog().is_some(),
                    _ => self.selected_lost().is_some(),
                };
                if act == Act::LoadMore {
                    m::WHY_NO_MORE.get().to_string()
                } else if has_row && act == Act::RestoreRef {
                    m::WHY_NO_REF.get().to_string()
                } else {
                    m::WHY_NO_ROW.get().to_string()
                }
            }
        }
    }
}
