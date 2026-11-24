//! What a row says, as plain data: its state, its age, its words. Pure, so
//! the tests pin them without drawing anything.

use gitpoulpi_core::domain::JournalState;
use gitpoulpi_i18n::msg::undo_safety as m;

use super::HistoryRow;
use crate::daemon_link::error_text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RowState {
    /// On the undo stack and the repository agrees.
    Undoable,
    /// The newest entry, and the repository moved under it: why.
    Blocked(String),
    /// On the redo stack.
    Undone,
}

/// The state of history row `index`, given the daemon's view of the two
/// top entries.
pub(crate) fn row_state(row: &HistoryRow, index: usize, state: &JournalState) -> RowState {
    if row.redo {
        return RowState::Undone;
    }
    match (&state.blocked, index) {
        (Some(refusal), 0) => RowState::Blocked(error_text::undo_refusal(refusal)),
        _ => RowState::Undoable,
    }
}

/// The entry was made outside GitPoulpi (a shell, an AI agent).
fn is_outside(row: &HistoryRow) -> bool {
    row.entry.source == gitpoulpi_core::domain::EntrySource::Outside
}

/// The short tag at the right end of a history row.
pub(crate) fn tag(row: &HistoryRow, state: &RowState) -> String {
    match state {
        RowState::Blocked(_) => m::TAG_BLOCKED.get().to_string(),
        RowState::Undone => m::TAG_UNDONE.get().to_string(),
        RowState::Undoable if is_outside(row) => m::TAG_OUTSIDE.get().to_string(),
        RowState::Undoable if row.entry.summary.origin.is_some() => sequence_badge(row),
        RowState::Undoable if row.entry.restore.restorable() => m::TAG_RESTORABLE.get().to_string(),
        RowState::Undoable => String::new(),
    }
}

/// `⟳ Backup`: the entry was made by a step of a sequence run.
pub(crate) fn sequence_badge(row: &HistoryRow) -> String {
    row.entry
        .summary
        .origin
        .as_ref()
        .map(|o| format!("\u{27F3} {}", o.name))
        .unwrap_or_default()
}

/// The line under the history list for the selected row.
pub(crate) fn history_detail(row: &HistoryRow, state: &RowState) -> Vec<String> {
    let mut lines = Vec::new();
    match state {
        RowState::Blocked(why) => lines.push(m::detail_blocked(why)),
        RowState::Undone => lines.push(m::DETAIL_UNDONE.get().to_string()),
        RowState::Undoable => lines.push(m::DETAIL_UNDOABLE.get().to_string()),
    }
    if let Some(o) = &row.entry.summary.origin {
        lines.push(m::detail_sequence(&o.name, o.step, o.of));
    }
    if is_outside(row) {
        let info = row.entry.outside.as_ref();
        lines.push(m::detail_outside(
            info.and_then(|i| i.committer.as_deref())
                .map(person)
                .unwrap_or("?")
                .to_string(),
        ));
    }
    if !row.redo {
        if row.entry.restore.restorable() {
            let items: Vec<String> = row
                .entry
                .restore
                .items
                .iter()
                .map(super::label::restore_item)
                .collect();
            lines.push(m::detail_restorable(items.join(", ")));
        } else if let Some(why) = &row.entry.restore.not_restorable {
            lines.push(m::detail_not_restorable(super::label::not_restorable(why)));
        }
    }
    lines
}

/// `Name <email>` to `Name`.
pub(crate) fn person(committer: &str) -> &str {
    committer.split(" <").next().unwrap_or(committer).trim()
}

/// How long ago, in the language's relative-time words.
pub(crate) fn age(then: i64) -> String {
    use gitpoulpi_i18n::msg::reltime as r;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let secs = (now - then).max(0) as u64;
    match secs {
        0..=59 => r::JUST_NOW.get().to_string(),
        60..=3599 => r::minutes_ago(secs / 60),
        3600..=86_399 => r::hours_ago(secs / 3600),
        86_400..=604_799 => r::days_ago(secs / 86_400),
        604_800..=2_591_999 => r::weeks_ago(secs / 604_800),
        2_592_000..=31_535_999 => r::months_ago(secs / 2_592_000),
        _ => r::years_ago(secs / 31_536_000),
    }
}

/// The heading of a day group: Today, Yesterday, else the date in the
/// user's short pattern.
pub(crate) fn day_heading(day: i64) -> String {
    use gitpoulpi_i18n::msg::undo_label as l;
    let today = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
        .div_euclid(86_400);
    match today - day {
        0 => l::DAY_TODAY.get().to_string(),
        1 => l::DAY_YESTERDAY.get().to_string(),
        _ => crate::live_settings::format_date(
            &crate::live_settings::date_short_pattern(),
            day * 86_400,
        ),
    }
}
