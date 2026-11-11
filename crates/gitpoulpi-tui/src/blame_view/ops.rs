//! What the Blame view can do: one enum shared by the keys, the detail
//! card and menu, the palette and the mouse, so the four can't drift apart.

use gitpoulpi_i18n::msg::blame_view as msg;
use gitpoulpi_i18n::M;

use crate::keymap::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum BlameOp {
    /// Select the hunk's commit in the Log (and leave the view).
    ShowInLog,
    CopySha,
    /// The commit's diff of this file.
    ShowDiff,
    /// Blame the file again as of the commit's parent.
    BlameParent,
    OpenHistory,
    ToggleCard,
    NextHunk,
    PrevHunk,
    NextSame,
    PrevSame,
    Goto,
    Wider,
    Narrower,
}

impl BlameOp {
    pub(crate) const ALL: [BlameOp; 13] = [
        BlameOp::ShowInLog,
        BlameOp::CopySha,
        BlameOp::ShowDiff,
        BlameOp::BlameParent,
        BlameOp::OpenHistory,
        BlameOp::ToggleCard,
        BlameOp::NextHunk,
        BlameOp::PrevHunk,
        BlameOp::NextSame,
        BlameOp::PrevSame,
        BlameOp::Goto,
        BlameOp::Wider,
        BlameOp::Narrower,
    ];

    /// The ops the detail card and the right-click menu list, in order.
    pub(crate) const POPUP: [BlameOp; 5] = [
        BlameOp::ShowInLog,
        BlameOp::CopySha,
        BlameOp::ShowDiff,
        BlameOp::BlameParent,
        BlameOp::OpenHistory,
    ];

    /// The keymap action (key, palette shortcut, help) of the op.
    pub(crate) fn action(self) -> Action {
        match self {
            BlameOp::ShowInLog => Action::FileInsightOpenCommit,
            BlameOp::CopySha => Action::FileInsightBlameCopySha,
            BlameOp::ShowDiff => Action::FileInsightBlameDiff,
            BlameOp::BlameParent => Action::FileInsightBlameParent,
            BlameOp::OpenHistory => Action::FileInsightBlameHistory,
            BlameOp::ToggleCard => Action::FileInsightBlameCard,
            BlameOp::NextHunk => Action::FileInsightBlameNext,
            BlameOp::PrevHunk => Action::FileInsightBlamePrev,
            BlameOp::NextSame => Action::FileInsightBlameNextSame,
            BlameOp::PrevSame => Action::FileInsightBlamePrevSame,
            BlameOp::Goto => Action::FileInsightBlameGoto,
            BlameOp::Wider => Action::FileInsightBlameWider,
            BlameOp::Narrower => Action::FileInsightBlameNarrower,
        }
    }

    /// The palette entry's label ("Blame: ...").
    pub(crate) const fn palette_label(self) -> M {
        match self {
            BlameOp::ShowInLog => msg::OP_SHOW_IN_LOG,
            BlameOp::CopySha => msg::OP_COPY_SHA,
            BlameOp::ShowDiff => msg::OP_SHOW_DIFF,
            BlameOp::BlameParent => msg::OP_PARENT,
            BlameOp::OpenHistory => msg::OP_HISTORY,
            BlameOp::ToggleCard => msg::OP_CARD,
            BlameOp::NextHunk => msg::OP_NEXT,
            BlameOp::PrevHunk => msg::OP_PREV,
            BlameOp::NextSame => msg::OP_NEXT_SAME,
            BlameOp::PrevSame => msg::OP_PREV_SAME,
            BlameOp::Goto => msg::OP_GOTO,
            BlameOp::Wider => msg::OP_WIDER,
            BlameOp::Narrower => msg::OP_NARROWER,
        }
    }

    /// The short label of a card / menu row (only the [`BlameOp::POPUP`] ones).
    pub(crate) fn popup_label(self) -> M {
        match self {
            BlameOp::ShowInLog => msg::ROW_SHOW_IN_LOG,
            BlameOp::CopySha => msg::ROW_COPY_SHA,
            BlameOp::ShowDiff => msg::ROW_SHOW_DIFF,
            BlameOp::BlameParent => msg::ROW_PARENT,
            BlameOp::OpenHistory => msg::ROW_HISTORY,
            other => other.palette_label(),
        }
    }
}

/// What a hit rect of the Blame view means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlameHit {
    /// The annotation cell of file line `n` (0-based).
    Annot(usize),
    /// The annotation column's border: drag to resize.
    Handle,
    /// The card or menu's body (swallows the pointer).
    Card,
    /// The card's close cross: like Esc, the card is dismissed.
    CardClose,
    /// A row of the card or menu.
    Op(BlameOp),
}
