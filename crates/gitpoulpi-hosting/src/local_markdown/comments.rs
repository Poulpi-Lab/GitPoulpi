//! The comments of a local note: a final `## Comments` section in the note
//! itself.
//!
//! ```markdown
//! (the note's own text)
//!
//! ## Comments
//!
//! ### kai - 2026-11-15T10:00:00Z
//! First comment, **any Markdown**.
//!
//! ### ann - 2026-11-15T11:30:00Z (edited 2026-11-15T12:00:00Z)
//! Second one.
//! ```
//!
//! The format is built to round-trip and to survive a hand edit:
//!
//! - the section is the last `## Comments` heading outside a code fence and
//!   runs to the end of the file (nothing may follow it);
//! - an entry starts at a line of exactly `### <author> - <UTC time>` with
//!   an optional ` (edited <UTC time>)`; its body is the lines up to the
//!   next entry, without the blank lines around it;
//! - a body line that would read as an entry marker or as the heading is
//!   written with a leading backslash (`\### ...`, which Markdown shows as
//!   the same text) and read back without it;
//! - everything above the heading is never touched, line endings included
//!   (the file's own style is used for what is written).
//!
//! An entry's id is `<time>|<author>`; posting moves the time on by a second
//! until the id is free, so ids stay unique and stable across edits and
//! deletes of other entries.

use std::sync::OnceLock;

use regex::Regex;

const HEADING: &str = "## Comments";

fn marker_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"^### (.+) - (\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z)(?: \(edited (\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z)\))?$",
        )
        .expect("static pattern")
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Entry {
    pub author: String,
    pub at: String,
    pub edited: Option<String>,
    pub body: String,
}

impl Entry {
    /// `<time>|<author>`: what a caller passes back to edit or delete it.
    pub fn id(&self) -> String {
        format!("{}|{}", self.at, self.author)
    }
}

/// The body split into what is above the section, and the entries.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct Parsed {
    /// Everything before the heading (all of it when there is none).
    pub head: String,
    /// Whether a heading was found.
    pub has_section: bool,
    pub entries: Vec<Entry>,
}

fn is_fence(line: &str) -> Option<(char, usize)> {
    let t = line.trim_start();
    let c = t.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let n = t.chars().take_while(|x| *x == c).count();
    (n >= 3).then_some((c, n))
}

/// Byte offset of the last `## Comments` heading line outside a fence.
fn heading_offset(body: &str) -> Option<usize> {
    let mut fence: Option<(char, usize)> = None;
    let mut found = None;
    let mut offset = 0;
    for raw in body.split_inclusive('\n') {
        let line = raw.trim_end_matches(['\n', '\r']);
        match (fence, is_fence(line)) {
            (None, Some(f)) => fence = Some(f),
            (Some((c, n)), Some((c2, n2))) if c == c2 && n2 >= n && line.trim().len() == n2 => {
                fence = None
            }
            (None, None) if line.trim_end() == HEADING => found = Some(offset),
            _ => {}
        }
        offset += raw.len();
    }
    found
}

/// Whether `line`, minus any leading backslashes, reads as the heading or
/// an entry marker (so it needs, or carries, one escaping backslash).
fn markerish(line: &str) -> bool {
    let base = line.trim_start_matches('\\').trim_end();
    base == HEADING || marker_re().is_match(base)
}

fn unescape(line: &str) -> &str {
    if line.starts_with('\\') && markerish(line) {
        &line[1..]
    } else {
        line
    }
}

fn escape(line: &str) -> String {
    if markerish(line) {
        format!("\\{line}")
    } else {
        line.to_string()
    }
}

pub(super) fn parse(body: &str) -> Parsed {
    let Some(at) = heading_offset(body) else {
        return Parsed {
            head: body.to_string(),
            ..Parsed::default()
        };
    };
    let (head, section) = body.split_at(at);
    let mut entries: Vec<Entry> = Vec::new();
    let mut current: Option<(Entry, Vec<String>)> = None;
    let finish = |cur: Option<(Entry, Vec<String>)>, out: &mut Vec<Entry>| {
        if let Some((mut e, lines)) = cur {
            e.body = lines.join("\n").trim_matches('\n').to_string();
            out.push(e);
        }
    };
    for raw in section.lines().skip(1) {
        let line = raw.trim_end_matches('\r');
        if let Some(c) = marker_re().captures(line.trim_end()) {
            finish(current.take(), &mut entries);
            current = Some((
                Entry {
                    author: c[1].to_string(),
                    at: c[2].to_string(),
                    edited: c.get(3).map(|m| m.as_str().to_string()),
                    body: String::new(),
                },
                Vec::new(),
            ));
        } else if let Some((_, lines)) = current.as_mut() {
            lines.push(unescape(line).to_string());
        }
    }
    finish(current.take(), &mut entries);
    Parsed {
        head: head.to_string(),
        has_section: true,
        entries,
    }
}

fn render_entry(e: &Entry, eol: &str) -> String {
    let edited = e
        .edited
        .as_ref()
        .map(|t| format!(" (edited {t})"))
        .unwrap_or_default();
    let mut out = format!("### {} - {}{edited}{eol}", e.author, e.at);
    let body = e.body.trim_matches(['\n', '\r']);
    if !body.is_empty() {
        for line in body.lines() {
            out.push_str(&escape(line.trim_end_matches('\r')));
            out.push_str(eol);
        }
    }
    out.push_str(eol);
    out
}

/// The text to put after `head` for these entries: the heading, a blank
/// line, and each entry followed by a blank line.
fn render_section(entries: &[Entry], eol: &str) -> String {
    let mut out = format!("{HEADING}{eol}{eol}");
    for e in entries {
        out.push_str(&render_entry(e, eol));
    }
    out
}

/// `head` ends with one blank line (or is empty), so a heading can follow.
fn ready_for_heading(head: &str, eol: &str) -> String {
    if head.trim().is_empty() {
        return head.to_string();
    }
    let trimmed = head.trim_end_matches(['\n', '\r']);
    format!("{trimmed}{eol}{eol}")
}

/// `body` with `entry` added at the end. Everything already there stays
/// byte for byte.
pub(super) fn append(body: &str, entry: &Entry, eol: &str) -> String {
    let parsed = parse(body);
    if !parsed.has_section {
        return format!(
            "{}{}",
            ready_for_heading(body, eol),
            render_section(std::slice::from_ref(entry), eol)
        );
    }
    let mut out = body.trim_end_matches(['\n', '\r']).to_string();
    out.push_str(eol);
    out.push_str(eol);
    out.push_str(&render_entry(entry, eol));
    out
}

/// `body` with the section rewritten from `entries` (edit and delete). The
/// head is kept byte for byte.
pub(super) fn rewrite(parsed: &Parsed, entries: &[Entry], eol: &str) -> String {
    format!("{}{}", parsed.head, render_section(entries, eol))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(author: &str, at: &str, body: &str) -> Entry {
        Entry {
            author: author.into(),
            at: at.into(),
            edited: None,
            body: body.into(),
        }
    }

    #[test]
    fn a_note_without_a_section_gets_one_and_the_head_is_kept() {
        let body = "# Title\n\nSome text.\n";
        let e = entry("kai", "2026-11-15T10:00:00Z", "Hello\nworld");
        let out = append(body, &e, "\n");
        assert_eq!(
            out,
            "# Title\n\nSome text.\n\n## Comments\n\n### kai - 2026-11-15T10:00:00Z\nHello\nworld\n\n"
        );
        let p = parse(&out);
        assert_eq!(p.head, "# Title\n\nSome text.\n\n");
        assert_eq!(p.entries, [e]);
    }

    #[test]
    fn appending_keeps_every_earlier_byte() {
        let first = append("text", &entry("kai", "2026-11-15T10:00:00Z", "one"), "\n");
        let second = append(&first, &entry("ann", "2026-11-15T11:00:00Z", "two"), "\n");
        assert!(second.starts_with(&first.trim_end_matches('\n').to_string()));
        let p = parse(&second);
        assert_eq!(p.entries.len(), 2);
        assert_eq!(p.entries[1].author, "ann");
    }

    #[test]
    fn crlf_files_stay_crlf() {
        let body = "text\r\n";
        let out = append(body, &entry("kai", "2026-11-15T10:00:00Z", "a\nb"), "\r\n");
        assert_eq!(
            out,
            "text\r\n\r\n## Comments\r\n\r\n### kai - 2026-11-15T10:00:00Z\r\na\r\nb\r\n\r\n"
        );
        let p = parse(&out);
        assert_eq!(p.entries[0].body, "a\nb");
    }

    #[test]
    fn lines_that_look_like_markers_are_escaped_and_come_back() {
        let tricky = "### ann - 2026-11-15T09:00:00Z\n## Comments\n\\### ann - 2026-11-15T09:00:00Z\nplain # line";
        let e = entry("kai", "2026-11-15T10:00:00Z", tricky);
        let out = append("x", &e, "\n");
        assert!(out.contains("\n\\### ann - 2026-11-15T09:00:00Z\n"));
        assert!(out.contains("\n\\## Comments\n"));
        let p = parse(&out);
        assert_eq!(p.entries.len(), 1, "no phantom entry");
        assert_eq!(p.entries[0].body, tricky);
    }

    #[test]
    fn a_heading_inside_a_code_fence_of_the_note_is_not_the_section() {
        let body = "```\n## Comments\n```\n\ntext\n";
        assert!(!parse(body).has_section);
        let out = append(body, &entry("kai", "2026-11-15T10:00:00Z", "hi"), "\n");
        let p = parse(&out);
        assert!(p.has_section);
        assert_eq!(p.head, format!("{body}\n"));
        assert_eq!(p.entries.len(), 1);
    }

    #[test]
    fn edited_entries_and_authors_with_dashes_round_trip() {
        let mut e = entry("a - b", "2026-11-15T10:00:00Z", "text");
        e.edited = Some("2026-11-16T10:00:00Z".into());
        let out = rewrite(&parse("head\n\n"), std::slice::from_ref(&e), "\n");
        assert_eq!(parse(&out).entries, [e.clone()]);
        assert_eq!(e.id(), "2026-11-15T10:00:00Z|a - b");
    }

    #[test]
    fn a_hand_written_section_parses_leniently() {
        let body = "note\n\n## Comments\n\nstray text\n### kai - 2026-11-15T10:00:00Z\n\n\nbody\n\n\n### ann - 2026-11-15T10:05:00Z\n";
        let p = parse(body);
        assert_eq!(p.entries.len(), 2);
        assert_eq!(p.entries[0].body, "body");
        assert_eq!(p.entries[1].body, "");
    }

    #[test]
    fn an_empty_body_note_gets_the_section_alone() {
        let out = append("", &entry("kai", "2026-11-15T10:00:00Z", "x"), "\n");
        assert!(out.starts_with("## Comments\n\n### kai"));
    }
}
