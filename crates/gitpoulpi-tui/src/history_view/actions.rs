//! What can be done on a History row: one enum shared by the keys, the
//! right-click menu and the palette, so the three cannot drift.

use crate::context_menu::Entry;
#[allow(unused_imports)]
pub(crate) use gitpoulpi_app::palette::vocab::history::HistoryAction;

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
