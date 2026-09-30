//! Building the JQL a list query sends. Pure.

use crate::model::{IssueFilter, StateFilter};

/// `"text"` with the two characters JQL escapes inside a quoted string.
pub(super) fn quote(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The user's filter without a trailing `ORDER BY ...` (ours comes last),
/// ignoring any `order by` inside a quoted string.
fn strip_order_by(filter: &str) -> &str {
    let bytes = filter.as_bytes();
    let lower = filter.to_ascii_lowercase();
    let mut quote: Option<u8> = None;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == b'\\' {
                    i += 1;
                } else if b == q {
                    quote = None;
                }
            }
            None => {
                if b == b'"' || b == b'\'' {
                    quote = Some(b);
                } else if lower[i..].starts_with("order")
                    && (i == 0 || bytes[i - 1].is_ascii_whitespace() || bytes[i - 1] == b')')
                {
                    let rest = lower[i + 5..].trim_start();
                    if rest.starts_with("by") && lower[i + 5..].starts_with(char::is_whitespace) {
                        return filter[..i].trim_end();
                    }
                }
            }
        }
        i += 1;
    }
    filter
}

/// `project = "KEY" [AND statusCategory ...] [AND assignee = currentUser()]
/// [AND (filter)] ORDER BY updated DESC`.
///
/// The key is quoted because two-letter keys such as `IN` or `TO` collide
/// with JQL reserved words. `project = ...` also makes the query *bounded*,
/// which `search/jql` requires.
pub(super) fn issues_jql(project: &str, filter: &IssueFilter) -> String {
    let mut jql = format!("project = {}", quote(project.trim()));
    match filter.state {
        StateFilter::Open => jql.push_str(" AND statusCategory != Done"),
        StateFilter::Closed => jql.push_str(" AND statusCategory = Done"),
        StateFilter::All => {}
    }
    if filter.assigned_to_me {
        jql.push_str(" AND assignee = currentUser()");
    }
    if let Some(extra) = filter.query.as_deref() {
        let extra = strip_order_by(extra.trim()).trim();
        if !extra.is_empty() {
            jql.push_str(&format!(" AND ({extra})"));
        }
    }
    jql.push_str(" ORDER BY updated DESC");
    jql
}
