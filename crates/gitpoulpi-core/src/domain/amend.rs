//! What [`crate::GitBackend::amend_commit`] changes about a commit, and the
//! rules both backends share so the new commit's oid is the same whichever
//! wrote it.

use serde::{Deserialize, Serialize};

use super::{normalize_reword_message, RewordRefusal};

/// The properties of a commit to change; every `None` is kept as it is.
///
/// `message` is the whole message (summary, blank line, body). Dates are
/// `<unix seconds> <+hhmm>` (git's raw format) or ISO 8601
/// (`2001-02-03T04:05:06+05:30`, `2001-02-03 04:05:06 +0530`, `...Z`);
/// see [`parse_git_date`]. The committer fields, when none is given, stay
/// "the current identity, now", as for `reword` and `git commit --amend`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitAmend {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committer_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committer_email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committer_date: Option<String>,
}

/// A point in time as git stores it: UTC seconds and the zone offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GitDate {
    pub seconds: i64,
    /// Minutes east of UTC.
    pub offset_minutes: i32,
}

impl GitDate {
    /// `<seconds> <+hhmm>`, git's raw format (what `GIT_AUTHOR_DATE=@...`
    /// reads back exactly).
    pub fn raw(self) -> String {
        let sign = if self.offset_minutes < 0 { '-' } else { '+' };
        let abs = self.offset_minutes.abs();
        format!("{} {sign}{:02}{:02}", self.seconds, abs / 60, abs % 60)
    }
}

/// A [`CommitAmend`] checked and normalised: trimmed identities, the
/// message as [`normalize_reword_message`] leaves it, dates parsed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedAmend {
    pub message: Option<String>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub author_date: Option<GitDate>,
    pub committer_name: Option<String>,
    pub committer_email: Option<String>,
    pub committer_date: Option<GitDate>,
}

impl CommitAmend {
    /// An amend that only sets the message.
    pub fn message_only(message: &str) -> Self {
        Self {
            message: Some(message.to_string()),
            ..Self::default()
        }
    }

    /// Validates and normalises; the refusals are the ones both backends
    /// return before touching anything.
    pub fn resolve(&self) -> Result<ResolvedAmend, RewordRefusal> {
        let message = match &self.message {
            Some(m) => Some(normalize_reword_message(m).ok_or(RewordRefusal::EmptyMessage)?),
            None => None,
        };
        let date = |d: &Option<String>| match d {
            Some(text) => parse_git_date(text)
                .map(Some)
                .ok_or(RewordRefusal::InvalidDate),
            None => Ok(None),
        };
        Ok(ResolvedAmend {
            message,
            author_name: identity_part(&self.author_name, true)?,
            author_email: identity_part(&self.author_email, false)?,
            author_date: date(&self.author_date)?,
            committer_name: identity_part(&self.committer_name, true)?,
            committer_email: identity_part(&self.committer_email, false)?,
            committer_date: date(&self.committer_date)?,
        })
    }
}

/// A name or an email, trimmed. Git cannot store `<`, `>` or a line break
/// in either, nor an empty one, nor a space in an email.
fn identity_part(value: &Option<String>, name: bool) -> Result<Option<String>, RewordRefusal> {
    let Some(raw) = value else { return Ok(None) };
    let text = raw.trim();
    let bad = text.is_empty()
        || text.contains(['<', '>', '\n', '\r', '\0'])
        || (!name && text.contains(char::is_whitespace));
    if bad {
        Err(RewordRefusal::InvalidIdentity)
    } else {
        Ok(Some(text.to_string()))
    }
}

/// Parses a git raw date (`1700000000 +0100`) or an ISO 8601 date-time
/// (`2001-02-03T04:05:06+05:30`, a space instead of `T`, `Z`, `+hhmm`, or
/// a space before the zone). A date without a zone is read as UTC.
pub fn parse_git_date(text: &str) -> Option<GitDate> {
    let text = text.trim();
    if let Some(date) = parse_raw(text) {
        return Some(date);
    }
    let (day, rest) = text.split_once(['T', ' '])?;
    let mut d = day.split('-');
    let year: i64 = d.next()?.parse().ok()?;
    let month: i64 = d.next()?.parse().ok()?;
    let dom: i64 = d.next()?.parse().ok()?;
    if d.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&dom) {
        return None;
    }
    let rest = rest.trim();
    let (clock, zone) = match rest.find(['+', '-', 'Z']) {
        Some(i) => (rest[..i].trim(), rest[i..].trim()),
        None => (rest, ""),
    };
    let mut c = clock.split(':');
    let hour: i64 = c.next()?.parse().ok()?;
    let minute: i64 = c.next()?.parse().ok()?;
    let second: i64 = match c.next() {
        Some(s) => s.parse().ok()?,
        None => 0,
    };
    if c.next().is_some() || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let offset_minutes = parse_zone(zone)?;
    let local = days_from_civil(year, month, dom) * 86_400 + hour * 3600 + minute * 60 + second;
    Some(GitDate {
        seconds: local - i64::from(offset_minutes) * 60,
        offset_minutes,
    })
}

fn parse_raw(text: &str) -> Option<GitDate> {
    let (secs, zone) = text.split_once(' ')?;
    let seconds: i64 = secs.strip_prefix('@').unwrap_or(secs).parse().ok()?;
    let offset_minutes = parse_zone(zone.trim())?;
    Some(GitDate {
        seconds,
        offset_minutes,
    })
}

/// `""`/`Z` = UTC, `+hhmm`, `+hh:mm` or `+hh`.
fn parse_zone(zone: &str) -> Option<i32> {
    if zone.is_empty() || zone == "Z" {
        return Some(0);
    }
    let sign = match zone.as_bytes()[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let digits: String = zone[1..].chars().filter(|c| *c != ':').collect();
    if !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let (h, m) = match digits.len() {
        2 => (digits.parse::<i32>().ok()?, 0),
        4 => (digits[..2].parse().ok()?, digits[2..].parse().ok()?),
        _ => return None,
    };
    if h > 23 || m > 59 {
        return None;
    }
    Some(sign * (h * 60 + m))
}

/// Days since 1970-01-01 of a proleptic Gregorian date.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_and_raw_dates_agree() {
        let iso = parse_git_date("2001-02-03T04:05:06+05:30").unwrap();
        assert_eq!(iso.offset_minutes, 330);
        assert_eq!(iso.seconds, 981_153_306);
        assert_eq!(iso.raw(), "981153306 +0530");
        assert_eq!(parse_git_date("981153306 +0530"), Some(iso));
        assert_eq!(parse_git_date("2001-02-03 04:05:06 +0530"), Some(iso));
        assert_eq!(
            parse_git_date("2001-02-03 04:05 +0530").map(|d| d.seconds),
            Some(iso.seconds - 6)
        );
        assert_eq!(parse_git_date("1970-01-01T00:00:00Z").unwrap().seconds, 0);
        assert_eq!(
            parse_git_date("1969-12-31T23:00:00-01:00").unwrap().raw(),
            "0 -0100"
        );
    }

    #[test]
    fn nonsense_dates_are_refused() {
        for bad in [
            "",
            "yesterday",
            "2001-13-01T00:00:00",
            "2001-02-03T25:00:00",
            "12 +99",
        ] {
            assert_eq!(parse_git_date(bad), None, "{bad}");
        }
    }

    #[test]
    fn resolve_normalises_and_refuses() {
        let ok = CommitAmend {
            message: Some("\n\nsubject\r\n\r\nbody  \n".into()),
            author_name: Some("  Ann ".into()),
            ..CommitAmend::default()
        }
        .resolve()
        .unwrap();
        assert_eq!(ok.message.as_deref(), Some("subject\n\nbody\n"));
        assert_eq!(ok.author_name.as_deref(), Some("Ann"));
        let refused = |a: CommitAmend| a.resolve().unwrap_err();
        assert_eq!(
            refused(CommitAmend::message_only(" \n")),
            RewordRefusal::EmptyMessage
        );
        for name in ["", "  ", "A <b>", "line\nbreak"] {
            assert_eq!(
                refused(CommitAmend {
                    author_name: Some(name.into()),
                    ..CommitAmend::default()
                }),
                RewordRefusal::InvalidIdentity,
                "{name:?}"
            );
        }
        assert_eq!(
            refused(CommitAmend {
                author_email: Some("a b@x".into()),
                ..CommitAmend::default()
            }),
            RewordRefusal::InvalidIdentity
        );
        assert_eq!(
            refused(CommitAmend {
                committer_date: Some("soon".into()),
                ..CommitAmend::default()
            }),
            RewordRefusal::InvalidDate
        );
    }
}
