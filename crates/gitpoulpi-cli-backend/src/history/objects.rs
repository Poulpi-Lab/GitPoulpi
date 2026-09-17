//! A commit as the engine edits it: its raw bytes plus the spans of its
//! headers, and the author / committer lines decoded only when a rule needs
//! them. Bytes throughout - an `encoding ISO-8859-1` commit is legal and a
//! message may not be UTF-8 - and unknown headers, continuation lines and
//! their order are never interpreted, so they survive a rewrite verbatim.

use std::ops::Range;

use gitpoulpi_core::domain::rewrite::GitDate;
use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::{Error, Result};

/// One logical header: `name value`, continuation lines included.
#[derive(Clone)]
struct Header {
    name: Range<usize>,
    /// The whole header with its trailing newline.
    span: Range<usize>,
}

#[derive(Clone)]
pub(crate) struct Commit {
    raw: Vec<u8>,
    headers: Vec<Header>,
    /// Where the message starts (after the blank line); `None` when the
    /// object has no blank line at all.
    message_start: Option<usize>,
}

impl Commit {
    pub(crate) fn parse(raw: Vec<u8>) -> Result<Self> {
        let mut headers = Vec::new();
        let mut pos = 0;
        let mut message_start = None;
        while pos < raw.len() {
            if raw[pos] == b'\n' {
                message_start = Some(pos + 1);
                break;
            }
            let mut end = pos;
            // A header runs to the first newline not followed by a space.
            loop {
                match raw[end..].iter().position(|b| *b == b'\n') {
                    Some(n) => end += n + 1,
                    None => end = raw.len(),
                }
                if end >= raw.len() || raw[end] != b' ' {
                    break;
                }
            }
            let name_end = raw[pos..end]
                .iter()
                .position(|b| *b == b' ' || *b == b'\n')
                .map_or(end, |n| pos + n);
            headers.push(Header {
                name: pos..name_end,
                span: pos..end,
            });
            pos = end;
        }
        let commit = Self {
            raw,
            headers,
            message_start,
        };
        if commit.header_index(b"tree").is_none() {
            return Err(Error::Backend("a commit without a tree".into()));
        }
        Ok(commit)
    }

    fn header_index(&self, name: &[u8]) -> Option<usize> {
        self.headers
            .iter()
            .position(|h| &self.raw[h.name.clone()] == name)
    }

    /// The value of header `i` without the name, the separating space and
    /// the final newline (continuation lines included).
    fn value(&self, i: usize) -> &[u8] {
        let h = &self.headers[i];
        let start = (h.name.end + 1).min(h.span.end);
        let end = h.span.end.saturating_sub(1).max(start);
        &self.raw[start..end]
    }

    pub(crate) fn parents(&self) -> Result<Vec<Oid>> {
        self.headers
            .iter()
            .enumerate()
            .filter(|(_, h)| &self.raw[h.name.clone()] == b"parent")
            .map(|(i, _)| {
                std::str::from_utf8(self.value(i))
                    .ok()
                    .and_then(Oid::from_hex)
                    .ok_or_else(|| Error::Backend("a commit has an unreadable parent".into()))
            })
            .collect()
    }

    pub(crate) fn author(&self) -> Option<Person> {
        Person::parse(self.value(self.header_index(b"author")?))
    }

    pub(crate) fn committer(&self) -> Option<Person> {
        Person::parse(self.value(self.header_index(b"committer")?))
    }

    /// The first `encoding` header's value, when the commit declares one.
    pub(crate) fn encoding(&self) -> Option<&[u8]> {
        Some(self.value(self.header_index(b"encoding")?))
    }

    /// The author line as stored (not necessarily UTF-8).
    pub(crate) fn author_bytes(&self) -> Option<&[u8]> {
        Some(self.value(self.header_index(b"author")?))
    }

    /// The committer line as stored (not necessarily UTF-8).
    pub(crate) fn committer_bytes(&self) -> Option<&[u8]> {
        Some(self.value(self.header_index(b"committer")?))
    }

    pub(crate) fn tree(&self) -> Option<Oid> {
        let i = self.header_index(b"tree")?;
        std::str::from_utf8(self.value(i))
            .ok()
            .and_then(Oid::from_hex)
    }

    pub(crate) fn message(&self) -> &[u8] {
        match self.message_start {
            Some(at) => &self.raw[at..],
            None => &[],
        }
    }

    pub(crate) fn is_signed(&self) -> bool {
        self.headers.iter().any(|h| {
            let name = &self.raw[h.name.clone()];
            name == b"gpgsig" || name == b"gpgsig-sha256"
        })
    }

    /// The object this commit becomes: the same headers in the same order
    /// with the parents mapped, a changed author / committer / message
    /// replaced and the signatures dropped (a signature over the old bytes
    /// would not verify).
    pub(crate) fn serialise(&self, edit: &Edit<'_>) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.raw.len() + 64);
        let mut next_parent = 0;
        for h in &self.headers {
            let name = &self.raw[h.name.clone()];
            match name {
                b"gpgsig" | b"gpgsig-sha256" => {}
                b"encoding" if edit.drop_encoding => {}
                b"tree" if edit.tree.is_some() => {
                    out.extend_from_slice(b"tree ");
                    out.extend_from_slice(edit.tree.unwrap().to_hex().as_bytes());
                    out.push(b'\n');
                }
                // All the parents at the first parent header: there can be
                // fewer than before (a parent that left the history, two
                // that became one), never more.
                b"parent" => {
                    if next_parent == 0 {
                        for p in edit.parents {
                            out.extend_from_slice(b"parent ");
                            out.extend_from_slice(p.to_hex().as_bytes());
                            out.push(b'\n');
                        }
                    }
                    next_parent += 1;
                }
                b"author" if edit.author.is_some() => {
                    out.extend_from_slice(b"author ");
                    out.extend_from_slice(&edit.author.unwrap().line());
                    out.push(b'\n');
                }
                b"committer" if edit.committer.is_some() => {
                    out.extend_from_slice(b"committer ");
                    out.extend_from_slice(&edit.committer.unwrap().line());
                    out.push(b'\n');
                }
                _ => out.extend_from_slice(&self.raw[h.span.clone()]),
            }
        }
        let message = edit
            .message
            .or_else(|| self.message_start.map(|_| self.message()));
        if let Some(message) = message {
            out.push(b'\n');
            out.extend_from_slice(message);
        }
        out
    }
}

/// What differs in a re-created commit; `None` keeps the original bytes.
pub(crate) struct Edit<'a> {
    pub(crate) tree: Option<Oid>,
    pub(crate) parents: &'a [Oid],
    pub(crate) author: Option<&'a Person>,
    pub(crate) committer: Option<&'a Person>,
    pub(crate) message: Option<&'a [u8]>,
    /// The message and names are UTF-8 now: the `encoding` header goes.
    pub(crate) drop_encoding: bool,
}

/// An author or committer line: `Name <email> <seconds> <+hhmm>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Person {
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) seconds: i64,
    pub(crate) offset: i16,
    /// The offset as stored (`-0000` and `+0000` are not the same bytes).
    tz: String,
    /// The line had no name and no space before `<`.
    bare: bool,
}

impl Person {
    /// `None` for a line the engine does not model (not UTF-8, an odd
    /// date): such a line is left as it is.
    pub(crate) fn parse(value: &[u8]) -> Option<Self> {
        let text = std::str::from_utf8(value).ok()?;
        let lt = text.find('<')?;
        let gt = text.rfind('>')?;
        if gt < lt {
            return None;
        }
        let before = &text[..lt];
        let (name, bare) = match before.strip_suffix(' ') {
            Some(name) => (name, false),
            None if before.is_empty() => ("", true),
            None => return None,
        };
        let rest = text[gt + 1..].strip_prefix(' ')?;
        let (seconds, tz) = rest.split_once(' ')?;
        let seconds: i64 = seconds.parse().ok()?;
        let offset = parse_offset(tz)?;
        Some(Self {
            name: name.to_string(),
            email: text[lt + 1..gt].to_string(),
            seconds,
            offset,
            tz: tz.to_string(),
            bare,
        })
    }

    /// `Name <email> seconds +hhmm`.
    pub(crate) fn line(&self) -> Vec<u8> {
        let ident = if self.name.is_empty() && self.bare {
            format!("<{}>", self.email)
        } else {
            format!("{} <{}>", self.name, self.email)
        };
        format!("{ident} {} {}", self.seconds, self.tz).into_bytes()
    }

    pub(crate) fn date(&self) -> GitDate {
        GitDate::new(self.seconds, self.offset)
    }

    /// Sets the date; the stored offset text is kept while the offset is.
    pub(crate) fn set_date(&mut self, date: GitDate) {
        self.seconds = date.seconds;
        if date.offset_minutes != self.offset {
            self.offset = date.offset_minutes;
            self.tz = date.offset_text();
        }
    }

    /// The date as `seconds +hhmm`, for the plan's rows.
    pub(crate) fn date_text(&self) -> String {
        format!("{} {}", self.seconds, self.tz)
    }
}

fn parse_offset(tz: &str) -> Option<i16> {
    let bytes = tz.as_bytes();
    if bytes.len() != 5 || !bytes[1..].iter().all(u8::is_ascii_digit) {
        return None;
    }
    let sign = match bytes[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let hours: i16 = tz[1..3].parse().ok()?;
    let minutes: i16 = tz[3..5].parse().ok()?;
    Some(sign * (hours * 60 + minutes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(extra: &str, message: &[u8]) -> Vec<u8> {
        let mut raw = format!(
            "tree {}\nparent {}\nauthor A <a@x.org> 100 +0200\ncommitter C <c@x.org> 200 -0000\n{extra}\n",
            "1".repeat(40),
            "2".repeat(40)
        )
        .into_bytes();
        raw.extend_from_slice(message);
        raw
    }

    #[test]
    fn headers_keep_their_continuation_lines() {
        let raw = commit(
            "gpgsig -----BEGIN\n \n line\n -----END\nx-other v\n",
            b"hi\n",
        );
        let c = Commit::parse(raw.clone()).unwrap();
        assert!(c.is_signed());
        assert_eq!(c.message(), b"hi\n");
        assert_eq!(c.parents().unwrap().len(), 1);
        // No edit but the signature: every other byte is as it was.
        let parents = c.parents().unwrap();
        let out = c.serialise(&Edit {
            tree: None,
            parents: &parents,
            author: None,
            committer: None,
            message: None,
            drop_encoding: false,
        });
        let text = String::from_utf8(out).unwrap();
        assert!(!text.contains("gpgsig"));
        assert!(text.contains("x-other v\n"));
        assert!(text.ends_with("\n\nhi\n"));
    }

    #[test]
    fn the_encoding_header_is_read_and_can_go() {
        let raw = commit("encoding ISO-8859-1\nx-other v\n", b"caf\xe9\n");
        let c = Commit::parse(raw).unwrap();
        assert_eq!(c.encoding(), Some(&b"ISO-8859-1"[..]));
        let parents = c.parents().unwrap();
        let out = c.serialise(&Edit {
            tree: None,
            parents: &parents,
            author: None,
            committer: None,
            message: Some(b"caf\xc3\xa9\n"),
            drop_encoding: true,
        });
        let text = String::from_utf8(out).unwrap();
        assert!(!text.contains("encoding"));
        assert!(text.contains("x-other v\n"));
        assert!(text.ends_with("\n\ncaf\u{e9}\n"));
    }

    #[test]
    fn a_person_round_trips_with_its_offset_text() {
        let p = Person::parse(b"A B <a@x.org> 100 -0000").unwrap();
        assert_eq!(p.line(), b"A B <a@x.org> 100 -0000");
        let mut q = p.clone();
        q.set_date(GitDate::new(150, 0));
        assert_eq!(q.line(), b"A B <a@x.org> 150 -0000");
        q.set_date(GitDate::new(150, 330));
        assert_eq!(q.line(), b"A B <a@x.org> 150 +0530");
        assert!(Person::parse(b"A <a@x.org> nope +0000").is_none());
        assert!(Person::parse(b"\xff <a@x.org> 1 +0000").is_none());
    }

    #[test]
    fn a_commit_without_a_blank_line_has_no_message() {
        let raw = format!(
            "tree {}\nauthor A <a@x.org> 1 +0000\ncommitter A <a@x.org> 1 +0000\n",
            "1".repeat(40)
        )
        .into_bytes();
        let c = Commit::parse(raw.clone()).unwrap();
        assert_eq!(c.message(), b"");
        let out = c.serialise(&Edit {
            tree: None,
            parents: &[],
            author: None,
            committer: None,
            message: None,
            drop_encoding: false,
        });
        assert_eq!(out, raw);
    }
}
