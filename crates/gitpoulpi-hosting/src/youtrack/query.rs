//! Building the YouTrack query a list sends. Pure.

use crate::model::{IssueFilter, StateFilter};

/// Where a trailing `sort by:` clause starts, ignoring one inside quotes or
/// braces.
fn sort_start(query: &str) -> Option<usize> {
    let lower = query.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut quote: Option<u8> = None;
    let mut brace = false;
    for (i, &b) in bytes.iter().enumerate() {
        match (quote, b) {
            (Some(q), _) if b == q => quote = None,
            (Some(_), _) => {}
            (None, b'"' | b'\'') => quote = Some(b),
            (None, b'{') => brace = true,
            (None, b'}') => brace = false,
            (None, b's') if !brace && lower[i..].starts_with("sort by:") => return Some(i),
            _ => {}
        }
    }
    None
}

/// `project: {ABC} [#Unresolved|#Resolved] [for: me] [(filter)] sort by:
/// updated desc`.
///
/// The short name goes in braces so a name with a dash or a space cannot
/// end the clause. The user's filter is parenthesised (so its `or` cannot
/// escape into the project clause); a `sort by:` of theirs replaces ours.
pub(super) fn issues_query(project: &str, filter: &IssueFilter) -> String {
    let mut q = format!("project: {{{}}}", project.trim());
    match filter.state {
        StateFilter::Open => q.push_str(" #Unresolved"),
        StateFilter::Closed => q.push_str(" #Resolved"),
        StateFilter::All => {}
    }
    if filter.assigned_to_me {
        q.push_str(" for: me");
    }
    let mut sort = "sort by: updated desc".to_string();
    if let Some(extra) = filter.query.as_deref() {
        let extra = extra.trim();
        let (cond, own_sort) = match sort_start(extra) {
            Some(i) => (extra[..i].trim(), Some(extra[i..].trim())),
            None => (extra, None),
        };
        if !cond.is_empty() {
            q.push_str(&format!(" and ({cond})"));
        }
        if let Some(s) = own_sort.filter(|s| !s.is_empty()) {
            sort = s.to_string();
        }
    }
    format!("{q} {sort}")
}

/// `1760000000000` to `2025-10-09T08:53:20Z`; empty for 0.
pub(super) fn epoch_ms_to_iso(ms: u64) -> String {
    if ms == 0 {
        return String::new();
    }
    let secs = ms / 1000;
    let (days, rem) = (secs / 86_400, secs % 86_400);
    // Howard Hinnant's civil_from_days.
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}
