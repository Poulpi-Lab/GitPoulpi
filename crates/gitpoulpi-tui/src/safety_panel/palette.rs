//! The command-palette twins of the panel's row actions: the same `Act`s the
//! buttons, the right-click menu and the letters run, listed in the palette
//! while the panel is open on the matching view, acting on its selected row.
//! A row that cannot do the action says why (status line and the panel's own
//! note) instead of silently doing nothing.

#[allow(unused_imports)]
pub(crate) use gitpoulpi_app::palette::vocab::safety_cmd::SafetyCmd;
use gitpoulpi_core::domain::JournalState;
use gitpoulpi_i18n::msg::undo_safety as m;

use super::{Act, PanelOutcome, SafetyPanel, Tab};

/// The panel row action a palette row runs: the terminal panel's own `Act`.
pub(crate) trait SafetyCmdExt {
    fn act(self) -> Option<Act>;
}

impl SafetyCmdExt for SafetyCmd {
    /// The row action it runs (`None` for a refresh and Close).
    fn act(self) -> Option<Act> {
        Some(match self {
            Self::UndoTo => Act::UndoTo,
            Self::RedoTo => Act::RedoTo,
            Self::UndoSequence => Act::UndoSequence,
            Self::RestoreOnly => Act::RestoreOnly,
            Self::Offer => Act::Offer,
            Self::ReflogShow => Act::ShowInLog,
            Self::ReflogBranch | Self::LostBranch => Act::CreateBranch,
            Self::ReflogTag | Self::LostTag => Act::CreateTag,
            Self::ReflogApply | Self::LostApply => Act::ApplyToWorktree,
            Self::ReflogStash | Self::LostStash => Act::SaveAsStash,
            Self::ReflogRestore => Act::RestoreRef,
            Self::ReflogMore => Act::LoadMore,
            Self::LostPick => Act::CherryPick,
            Self::HistoryRefresh
            | Self::ReflogRefresh
            | Self::LostRefresh
            | Self::Close
            | Self::NextPane
            | Self::PathTree
            | Self::ResetPanes
            | Self::ArrangeDialog
            | Self::ResetDialog => return None,
        })
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
        // The pane and window rows act on the panel itself, not on a row.
        match cmd {
            SafetyCmd::NextPane => {
                self.overlay = None;
                self.cycle_focus_pub(true);
                return (PanelOutcome::Stay, None);
            }
            SafetyCmd::PathTree => {
                self.overlay = None;
                self.panes.toggle_mode();
                return (PanelOutcome::Stay, None);
            }
            SafetyCmd::ResetPanes => {
                self.overlay = None;
                self.panes.reset_dividers();
                return (PanelOutcome::Stay, None);
            }
            SafetyCmd::ArrangeDialog => {
                self.arrange();
                self.say(
                    gitpoulpi_i18n::msg::dialog_frame::ARRANGING
                        .get()
                        .to_string(),
                );
                return (PanelOutcome::Stay, None);
            }
            SafetyCmd::ResetDialog => {
                self.overlay = None;
                crate::dialog_kit::frame::reset_top();
                self.say(
                    gitpoulpi_i18n::msg::dialog_frame::RESET_DONE
                        .get()
                        .to_string(),
                );
                return (PanelOutcome::Stay, None);
            }
            _ => {}
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
                    Act::UndoSequence if row.redo => m::WHY_ALREADY_UNDONE.get().to_string(),
                    Act::UndoSequence => gitpoulpi_i18n::msg::undo_ui::WHY_NO_SEQUENCE
                        .get()
                        .to_string(),
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
