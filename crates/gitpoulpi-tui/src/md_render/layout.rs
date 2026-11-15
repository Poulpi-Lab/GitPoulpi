//! Wrapping by cells: styled characters in, [`MdLine`]s out. A wide glyph
//! never straddles the edge and a word longer than the line is hard-broken.

use super::{LineKind, LinkId, Look, MdLine, Span};
use crate::cell_text::char_cells;

/// One styled character. `'\n'` is a forced line break.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Atom {
    pub(super) ch: char,
    pub(super) look: Look,
    pub(super) link: Option<LinkId>,
}

pub(super) fn atoms(text: &str, look: Look, link: Option<LinkId>) -> Vec<Atom> {
    text.chars().map(|ch| Atom { ch, look, link }).collect()
}

/// The leading cells of a line (a bullet, quote bars, indentation).
#[derive(Debug, Clone, Default)]
pub(super) struct Prefix {
    pub(super) spans: Vec<(String, Look)>,
    pub(super) width: usize,
}

impl Prefix {
    pub(super) fn push(&mut self, text: &str, look: Look) {
        self.width += crate::cell_text::cells(text);
        self.spans.push((text.to_string(), look));
    }
}

/// A line under construction.
pub(super) struct LineBuf {
    spans: Vec<Span>,
    pub(super) used: usize,
}

impl LineBuf {
    pub(super) fn new(prefix: &Prefix) -> Self {
        let mut line = LineBuf {
            spans: Vec::new(),
            used: 0,
        };
        for (text, look) in &prefix.spans {
            line.push_str(text, *look, None);
        }
        line.used = prefix.width;
        line
    }

    pub(super) fn push_str(&mut self, text: &str, look: Look, link: Option<LinkId>) {
        if text.is_empty() {
            return;
        }
        match self.spans.last_mut() {
            Some(last) if last.look == look && last.link == link && last.hl.is_none() => {
                last.text.push_str(text);
            }
            _ => self.spans.push(Span {
                text: text.to_string(),
                look,
                link,
                hl: None,
            }),
        }
    }

    pub(super) fn push_atom(&mut self, a: Atom) {
        let mut buf = [0u8; 4];
        self.push_str(a.ch.encode_utf8(&mut buf), a.look, a.link);
        self.used += char_cells(a.ch);
    }

    pub(super) fn finish(self) -> MdLine {
        MdLine {
            spans: self.spans,
            kind: LineKind::Text,
        }
    }
}

fn is_gap(c: char) -> bool {
    c == ' ' || c == '\t'
}

/// The look of the space between two words: the words' own when both share
/// a look that shows on a blank, plain otherwise.
fn gap_atom(before: Atom, after: Atom) -> Atom {
    let shared = before.look == after.look
        && before.link == after.link
        && matches!(
            before.look,
            Look::Heading | Look::Quote | Look::Link | Look::Strike | Look::Dim
        );
    if shared {
        Atom { ch: ' ', ..before }
    } else {
        Atom {
            ch: ' ',
            look: Look::Plain,
            link: None,
        }
    }
}

/// Greedy wrap of `atoms` to `width` cells. `first` and `rest` prefix the
/// first and the later lines. Always at least one line.
pub(super) fn wrap(atoms: &[Atom], width: usize, first: &Prefix, rest: &Prefix) -> Vec<MdLine> {
    let mut out: Vec<MdLine> = Vec::new();
    let mut line = LineBuf::new(first);
    let mut fresh = true; // nothing but the prefix on the line
    let mut last: Option<Atom> = None;
    // Split into words; a forced break is a `None` word.
    let mut cur: Vec<Atom> = Vec::new();
    let mut seq: Vec<Option<Vec<Atom>>> = Vec::new();
    for &a in atoms {
        if a.ch == '\n' {
            if !cur.is_empty() {
                seq.push(Some(std::mem::take(&mut cur)));
            }
            seq.push(None);
        } else if is_gap(a.ch) {
            if !cur.is_empty() {
                seq.push(Some(std::mem::take(&mut cur)));
            }
        } else {
            cur.push(a);
        }
    }
    if !cur.is_empty() {
        seq.push(Some(cur));
    }
    let mut limit = width.max(first.width + 2);
    let newline = |out: &mut Vec<MdLine>, line: &mut LineBuf, fresh: &mut bool| {
        let done = std::mem::replace(line, LineBuf::new(rest));
        out.push(done.finish());
        *fresh = true;
    };
    for item in seq {
        let Some(word) = item else {
            newline(&mut out, &mut line, &mut fresh);
            limit = width.max(rest.width + 2);
            last = None;
            continue;
        };
        let word_w: usize = word.iter().map(|a| char_cells(a.ch)).sum();
        let gap = usize::from(!fresh);
        if !fresh && line.used + gap + word_w > limit {
            newline(&mut out, &mut line, &mut fresh);
            limit = width.max(rest.width + 2);
        } else if !fresh {
            if let Some(prev) = last {
                line.push_atom(gap_atom(prev, word[0]));
            }
        }
        for &a in &word {
            let w = char_cells(a.ch);
            if line.used + w > limit && !fresh {
                newline(&mut out, &mut line, &mut fresh);
                limit = width.max(rest.width + 2);
            }
            line.push_atom(a);
            fresh = false;
            last = Some(a);
        }
    }
    out.push(line.finish());
    out
}

#[cfg(test)]
mod unit {
    use super::*;

    fn lines(text: &str, width: usize) -> Vec<String> {
        let p = Prefix::default();
        wrap(&atoms(text, Look::Plain, None), width, &p, &p)
            .iter()
            .map(MdLine::text)
            .collect()
    }

    #[test]
    fn words_wrap_and_long_words_break() {
        assert_eq!(lines("aa bb cc", 5), ["aa bb", "cc"]);
        assert_eq!(lines("abcdefgh", 3), ["abc", "def", "gh"]);
    }

    #[test]
    fn a_wide_glyph_moves_to_the_next_line() {
        assert_eq!(lines("abc\u{4e2d}", 4), ["abc", "\u{4e2d}"]);
    }
}
