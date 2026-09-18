//! What can be done on a History row: one enum shared by the keys, the
//! right-click menu and the palette, so the three cannot drift.

use gitpoulpi_i18n::msg::history_view as msg;

use crate::context_menu::Entry;
use crate::keymap::Action as Bind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum HistoryAction {
    /// Select the row's commit in the Log, loading more of the Log if needed.
    ShowInLog,
    CopySha,
    CopySummary,
    /// The right side shows the row's diff of the file.
    ShowDiff,
    /// The right side shows the file as it was at the row's commit.
    OpenFile,
    /// The right side shows the file blamed as of the row's commit.
    Blame,
    /// The row's commit against the working tree, on the right.
    CompareWorkingTree,
    /// Check the file out of the row's commit into the working tree.
    Restore,
    /// Extend the selection by a row: the keyboard twin of Shift+click.
    RangeUp,
    RangeDown,
    /// The keyboard goes to the other half.
    SwitchFocus,
    WidenList,
    NarrowList,
    ResetSplit,
}

impl HistoryAction {
    pub(crate) const ALL: [HistoryAction; 14] = [
        HistoryAction::ShowInLog,
        HistoryAction::CopySha,
        HistoryAction::CopySummary,
        HistoryAction::ShowDiff,
        HistoryAction::OpenFile,
        HistoryAction::Blame,
        HistoryAction::CompareWorkingTree,
        HistoryAction::Restore,
        HistoryAction::RangeUp,
        HistoryAction::RangeDown,
        HistoryAction::SwitchFocus,
        HistoryAction::WidenList,
        HistoryAction::NarrowList,
        HistoryAction::ResetSplit,
    ];

    /// The registry action carrying this one's key.
    pub(crate) fn bind(self) -> Bind {
        match self {
            HistoryAction::ShowInLog => Bind::HistoryShowInLog,
            HistoryAction::CopySha => Bind::HistoryCopySha,
            HistoryAction::CopySummary => Bind::HistoryCopySummary,
            HistoryAction::ShowDiff => Bind::HistoryShowDiff,
            HistoryAction::OpenFile => Bind::HistoryOpenFile,
            HistoryAction::Blame => Bind::HistoryBlame,
            HistoryAction::CompareWorkingTree => Bind::HistoryCompareWorkingTree,
            HistoryAction::Restore => Bind::HistoryRestore,
            HistoryAction::RangeUp => Bind::HistoryRangeUp,
            HistoryAction::RangeDown => Bind::HistoryRangeDown,
            HistoryAction::SwitchFocus => Bind::HistorySwitchFocus,
            HistoryAction::WidenList => Bind::HistoryWidenList,
            HistoryAction::NarrowList => Bind::HistoryNarrowList,
            HistoryAction::ResetSplit => Bind::HistoryResetSplit,
        }
    }

    /// The row's text in the context menu.
    pub(crate) fn label(self) -> &'static str {
        match self {
            HistoryAction::ShowInLog => msg::ACTION_SHOW_IN_LOG.get(),
            HistoryAction::CopySha => msg::ACTION_COPY_SHA.get(),
            HistoryAction::CopySummary => msg::ACTION_COPY_SUMMARY.get(),
            HistoryAction::ShowDiff => msg::ACTION_SHOW_DIFF.get(),
            HistoryAction::OpenFile => msg::ACTION_OPEN_FILE.get(),
            HistoryAction::Blame => msg::ACTION_BLAME.get(),
            HistoryAction::CompareWorkingTree => msg::ACTION_COMPARE_WORKING_TREE.get(),
            HistoryAction::Restore => msg::ACTION_RESTORE.get(),
            HistoryAction::RangeUp => msg::ACTION_RANGE_UP.get(),
            HistoryAction::RangeDown => msg::ACTION_RANGE_DOWN.get(),
            HistoryAction::SwitchFocus => msg::ACTION_SWITCH_FOCUS.get(),
            HistoryAction::WidenList => msg::ACTION_WIDEN_LIST.get(),
            HistoryAction::NarrowList => msg::ACTION_NARROW_LIST.get(),
            HistoryAction::ResetSplit => msg::ACTION_RESET_SPLIT.get(),
        }
    }
}

/// The right-click menu of a row.
pub(crate) fn menu_entries(multi: bool) -> Vec<Entry<HistoryAction>> {
    use HistoryAction as A;
    let row = |a: A| Entry::action(a.label(), a).with_key(a.bind());
    let mut entries = vec![row(A::ShowInLog), Entry::Separator];
    entries.push(row(A::CopySha));
    entries.push(row(A::CopySummary));
    entries.push(Entry::Separator);
    entries.push(row(A::ShowDiff));
    entries.push(row(A::OpenFile));
    entries.push(row(A::Blame));
    entries.push(row(A::CompareWorkingTree));
    if multi {
        entries.push(Entry::Separator);
    } else {
        entries.push(Entry::Separator);
        entries.push(Entry::danger(A::Restore.label(), A::Restore).with_key(A::Restore.bind()));
    }
    entries
}
