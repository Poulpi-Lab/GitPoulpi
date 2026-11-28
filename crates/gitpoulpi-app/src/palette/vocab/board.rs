//! The Kanban board's command vocabulary, kept apart from the view so the
//! command palette (and a build without a hosting provider) can name the
//! commands without the view's types.
#![cfg_attr(not(all(feature = "hosting", feature = "board")), allow(dead_code))]

use gitpoulpi_i18n::msg::board as msg;

/// What the palette needs to know to gray a board command out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardAvail {
    /// The cards are in.
    pub loaded: bool,
    /// A card is selected.
    pub has_card: bool,
    /// The source lets a card change column.
    pub can_move: bool,
    /// The selected card has a column to its left / right.
    pub has_left: bool,
    pub has_right: bool,
    /// The provider takes assignee / label / milestone edits.
    pub can_edit_meta: bool,
    /// The repository blocks assignment.
    pub assignment_blocked: bool,
    /// The source forms its columns from labels (the label set can be set).
    pub label_columns: bool,
    /// Some filter is on.
    pub filtered: bool,
}

/// What the board can be asked to do. The palette lists one entry per value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardCmd {
    /// Pick the column of the selected card in a dropdown.
    MoveTo,
    MoveLeft,
    MoveRight,
    OpenCard,
    EditAssignees,
    EditLabels,
    SetMilestone,
    /// The New issue form, in the selected column.
    NewCard,
    FilterAssignee,
    FilterLabel,
    FilterMilestone,
    /// Type in the text filter.
    FilterText,
    ClearFilters,
    /// The labels that are the columns of a host board.
    SetColumnLabels,
    Refresh,
    Close,
}

impl BoardCmd {
    /// Why the command does nothing right now (`None`: it can run).
    pub fn unavailable(self, avail: Option<BoardAvail>) -> Option<&'static str> {
        let Some(a) = avail else {
            return Some(msg::WHY_NO_VIEW.get());
        };
        match self {
            Self::Refresh | Self::Close | Self::FilterText => None,
            Self::FilterAssignee | Self::FilterLabel | Self::FilterMilestone | Self::NewCard => {
                (!a.loaded).then_some(msg::NOT_LOADED.get())
            }
            Self::ClearFilters => (!a.filtered).then_some(msg::WHY_NO_FILTER.get()),
            Self::OpenCard => (!a.has_card).then_some(msg::WHY_NO_CARD.get()),
            Self::MoveTo | Self::MoveLeft | Self::MoveRight => {
                if !a.has_card {
                    Some(msg::WHY_NO_CARD.get())
                } else if !a.can_move {
                    Some(msg::READ_ONLY.get())
                } else if (self == Self::MoveLeft && !a.has_left)
                    || (self == Self::MoveRight && !a.has_right)
                {
                    Some(msg::WHY_NO_COLUMN.get())
                } else {
                    None
                }
            }
            Self::EditAssignees if a.has_card && a.assignment_blocked => {
                Some(gitpoulpi_i18n::msg::hosting_ui::WHY_ASSIGNMENT_BLOCKED.get())
            }
            Self::EditAssignees | Self::EditLabels | Self::SetMilestone => {
                if !a.has_card {
                    Some(msg::WHY_NO_CARD.get())
                } else if !a.can_edit_meta {
                    Some(msg::WHY_NO_META.get())
                } else {
                    None
                }
            }
            Self::SetColumnLabels => (!a.label_columns).then_some(msg::WHY_NO_LABEL_COLUMNS.get()),
        }
    }
}
