//! What the issue trackers' conversations share: JSON field helpers, the
//! item reference, an empty-body check, and windowing for hosts that hand
//! over the whole history in one answer.

// Each tracker uses a subset (a build with one tracker leaves the rest unused).
#![allow(dead_code)]

use serde_json::Value;

use super::time::iso_secs;
use super::{PageCursor, WINDOW};
use crate::{Error, Result};

/// A refusal before anything is sent, shaped like the host's own 422.
pub(crate) fn invalid(what: &str, detail: impl Into<String>) -> Error {
    Error::Api {
        what: what.to_string(),
        status: 422,
        detail: detail.into(),
    }
}

pub(crate) fn text(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

pub(crate) fn opt_text(v: &Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// A comment must say something, and fit the host's limit: nothing is sent
/// otherwise.
pub(crate) fn check_body(what: &str, body: &str, max: Option<usize>) -> Result<()> {
    if body.trim().is_empty() {
        return Err(invalid(what, "the comment is empty"));
    }
    if let Some(max) = max.filter(|m| body.chars().count() > *m) {
        return Err(invalid(
            what,
            format!("the comment is longer than the {max} characters the host takes"),
        ));
    }
    Ok(())
}

/// Trackers have issues only.
pub(crate) fn require_issue(tracker: &str, key: &super::ItemKey) -> Result<()> {
    if key.kind == super::ItemKind::Issue {
        Ok(())
    } else {
        Err(crate::Error::Unsupported(format!(
            "{tracker} has no pull requests"
        )))
    }
}

/// The tracker's own reference of an item: the key a caller carries in
/// `ItemKey::id` (`ABC-12`, a uuid, a card id), else `<project>-<number>`.
pub(crate) fn item_ref(project: &str, key: &super::ItemKey) -> String {
    if key.id.trim().is_empty() {
        format!("{}-{}", project.trim(), key.number)
    } else {
        key.id.trim().to_string()
    }
}

/// Splits `all` (oldest first) into the newest [`WINDOW`] entries, or the
/// [`WINDOW`] entries before the index `cursor` names. The cursor is the
/// index the returned window starts at, so it only holds while the history
/// does not shrink between two reads (a deleted comment moves it by one).
pub(crate) fn window_of<T>(
    mut all: Vec<T>,
    cursor: Option<&PageCursor>,
) -> Result<(Vec<T>, Option<PageCursor>)> {
    let end = match cursor {
        None => all.len(),
        Some(c) => {
            c.0.parse::<usize>()
                .map_err(|_| invalid("reading a conversation", "bad page cursor"))?
                .min(all.len())
        }
    };
    let start = end.saturating_sub(WINDOW);
    all.truncate(end);
    let window = all.split_off(start);
    let older = (start > 0).then(|| PageCursor(start.to_string()));
    Ok((window, older))
}

/// The time span of the comments a window holds, so that the events (which
/// a tracker lists apart) are cut to the same span.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Span {
    /// Inclusive; `None` = from the start of the history.
    pub lower: Option<i64>,
    /// Exclusive; `None` = up to now.
    pub upper: Option<i64>,
}

impl Span {
    pub fn holds(&self, at: &str) -> bool {
        let Some(t) = iso_secs(at) else {
            return true;
        };
        self.lower.is_none_or(|l| t >= l) && self.upper.is_none_or(|u| t < u)
    }
}

/// `position` and the span's upper bound in one opaque cursor.
pub(crate) fn encode_cursor(position: &str, upper: Option<i64>) -> PageCursor {
    PageCursor(match upper {
        Some(u) => format!("{position}|{u}"),
        None => position.to_string(),
    })
}

/// The inverse of [`encode_cursor`]; `None` for a cursor that is not ours.
pub(crate) fn decode_cursor(cursor: &PageCursor) -> Option<(String, Option<i64>)> {
    match cursor.0.rsplit_once('|') {
        Some((p, u)) => Some((p.to_string(), Some(u.parse().ok()?))),
        None if cursor.0.is_empty() => None,
        None => Some((cursor.0.clone(), None)),
    }
}

/// The span of the window whose comments were created at `created`: from
/// its oldest comment when an older window exists, up to `upper` (where the
/// newer window starts).
pub(crate) fn span_of<'a>(
    created: impl Iterator<Item = &'a str>,
    has_older: bool,
    upper: Option<i64>,
) -> Span {
    let oldest = created.filter_map(iso_secs).min();
    Span {
        lower: if has_older { oldest } else { None },
        upper,
    }
}

/// Seconds of the oldest comment of a window: the next older window's
/// `upper` bound.
pub(crate) fn oldest_secs<'a>(created: impl Iterator<Item = &'a str>) -> Option<i64> {
    created.filter_map(iso_secs).min()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_history_is_one_window() {
        let (w, older) = window_of(vec![1, 2, 3], None).unwrap();
        assert_eq!(w, [1, 2, 3]);
        assert_eq!(older, None);
    }

    #[test]
    fn the_newest_window_comes_first_then_the_one_before() {
        let all: Vec<usize> = (0..650).collect();
        let (w, older) = window_of(all.clone(), None).unwrap();
        assert_eq!((w.len(), w[0], w[299]), (WINDOW, 350, 649));
        assert_eq!(older, Some(PageCursor("350".into())));
        let (w, older) = window_of(all.clone(), older.as_ref()).unwrap();
        assert_eq!((w[0], w[299]), (50, 349));
        let (w, older) = window_of(all, older.as_ref()).unwrap();
        assert_eq!((w.len(), w[0], w[49]), (50, 0, 49));
        assert_eq!(older, None);
    }

    #[test]
    fn a_bad_cursor_is_refused() {
        assert!(window_of(vec![1], Some(&PageCursor("x".into()))).is_err());
    }

    #[test]
    fn cursors_round_trip_and_spans_cut_events() {
        let c = encode_cursor("abc", Some(100));
        assert_eq!(decode_cursor(&c), Some(("abc".into(), Some(100))));
        assert_eq!(
            decode_cursor(&PageCursor("7".into())),
            Some(("7".into(), None))
        );
        assert_eq!(decode_cursor(&PageCursor(String::new())), None);
        let span = Span {
            lower: Some(1_714_557_600),
            upper: Some(1_714_561_200),
        };
        assert!(span.holds("2024-05-01T10:00:00Z"));
        assert!(!span.holds("2024-05-01T09:59:59Z"));
        assert!(!span.holds("2024-05-01T11:00:00Z"));
        assert!(span.holds("not a time"));
    }
}
