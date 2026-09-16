//! Commit-message trailers (`Signed-off-by: Name <email>`), parsed with git's
//! own grammar (`git interpret-trailers --parse` with no configuration):
//!
//! - the trailer block is the **last paragraph** of the message, never the
//!   first one (the title), and stops before a `---` divider line;
//! - a line is a trailer when it is `Key: value`: the key is letters, digits
//!   and `-` (blanks may sit between the key and the `:`), a line that starts
//!   with a blank continues the line above;
//! - `Signed-off-by: ` and `(cherry picked from commit ` lines are the
//!   recognised kinds: a block with one of them is a trailer block when a
//!   third of its lines are trailers, any other block only when **all** are;
//! - lines starting with `#` are comments and count for nothing.
//!
//! Every edit changes only the trailer lines it names: the rest of the
//! message stays byte for byte (line endings of an inserted line follow the
//! block's). Pure functions over text; the engine keeps the bytes of a
//! message that is not valid UTF-8 untouched.

use std::ops::Range;

use serde::{Deserialize, Serialize};

use super::identity::Ident;
use super::message::CompiledPattern;

/// What `add` does when a trailer with the same key is already there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrailerIfExists {
    /// Add it unless the very same `key: value` is already there (git's
    /// `addIfDifferent`).
    #[default]
    Add,
    /// Remove every trailer with that key and add this one at the end.
    Replace,
    /// Leave the message alone when the key is present.
    Skip,
}

/// A parsed trailer: the key as written, the value with its continuation
/// lines folded onto one line (as `--parse` prints it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trailer {
    pub key: String,
    pub value: String,
}

/// One line group of the block: a trailer with its continuation lines, or
/// something else (a cherry-pick line, a non-trailer line, a comment).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockLine {
    /// The bytes of the line(s), line ending included.
    pub range: Range<usize>,
    pub trailer: Option<Parsed>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed {
    pub key: String,
    /// Folded value.
    pub value: String,
    /// The raw value: after the separator and its blanks, up to the end of
    /// the last continuation line, line ending excluded.
    pub raw: Range<usize>,
}

/// The trailer block of a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// Bytes of the block (its first line to the end of its last line).
    pub range: Range<usize>,
    pub lines: Vec<BlockLine>,
}

impl Block {
    pub fn trailers(&self) -> impl Iterator<Item = &Parsed> {
        self.lines.iter().filter_map(|l| l.trailer.as_ref())
    }
}

/// C `isspace`.
fn sp(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0b' | '\x0c')
}

fn divider(l: &Line<'_>) -> bool {
    l.text
        .strip_prefix("---")
        .is_some_and(|rest| rest.chars().next().is_some_and(sp))
}

struct Line<'a> {
    start: usize,
    text: &'a str,
}

impl Line<'_> {
    fn end(&self) -> usize {
        self.start + self.text.len()
    }
    /// The text without its line ending.
    fn body(&self) -> &str {
        self.text.strip_suffix('\n').unwrap_or(self.text)
    }
    fn blank(&self) -> bool {
        self.text.chars().all(sp)
    }
    /// A continuation line (starts with a blank).
    fn indented(&self) -> bool {
        self.text.starts_with(sp)
    }
}

fn lines(message: &str) -> Vec<Line<'_>> {
    let mut at = 0;
    message
        .split_inclusive('\n')
        .map(|text| {
            let line = Line { start: at, text };
            at += text.len();
            line
        })
        .collect()
}

/// `(key end, separator end)` of a `Key: value` line.
fn split_key(line: &str) -> Option<(usize, usize)> {
    let mut key_end = None;
    for (i, c) in line.char_indices() {
        if c == ':' {
            return key_end.or(Some(i)).filter(|k| *k > 0).map(|k| (k, i + 1));
        }
        if key_end.is_none() && (c.is_ascii_alphanumeric() || c == '-') {
            continue;
        }
        if i > 0 && (c == ' ' || c == '\t') {
            key_end.get_or_insert(i);
            continue;
        }
        return None;
    }
    None
}

const RECOGNISED: [&str; 2] = ["Signed-off-by: ", "(cherry picked from commit "];

/// The trailer block of `message`, if it has one.
pub fn find_block(message: &str) -> Option<Block> {
    let all = lines(message);
    // Everything from a `---` divider on is not the message.
    let log_end = all.iter().position(divider).unwrap_or(all.len());
    let log = &all[..log_end];
    // The title: lines up to the first blank one (comments skipped).
    let title_end = log
        .iter()
        .position(|l| !l.text.starts_with('#') && l.blank())
        .unwrap_or(log.len());
    let (mut trailer_lines, mut non_trailer, mut cont) = (0usize, 0usize, 0usize);
    let mut recognised = false;
    let mut only_spaces = true;
    // The block's last line (exclusive) and first line, as indices.
    let mut last = None;
    let mut first = log.len();
    let mut accepted = false;
    let mut broke = false;
    let mut i = log.len();
    while i > title_end {
        i -= 1;
        let l = &log[i];
        if l.text.starts_with('#') {
            non_trailer += cont;
            cont = 0;
            continue;
        }
        if l.blank() {
            if only_spaces {
                continue;
            }
            non_trailer += cont;
            accepted = (recognised && trailer_lines * 3 >= non_trailer)
                || (trailer_lines > 0 && non_trailer == 0);
            first = i + 1;
            broke = true;
            break;
        }
        only_spaces = false;
        last.get_or_insert(i + 1);
        if RECOGNISED.iter().any(|p| l.text.starts_with(p)) {
            trailer_lines += 1;
            cont = 0;
            recognised = true;
        } else if !l.indented() && split_key(l.body()).is_some() {
            trailer_lines += 1;
            cont = 0;
        } else if l.indented() {
            cont += 1;
        } else {
            non_trailer += 1 + cont;
            cont = 0;
        }
    }
    if !broke {
        // Reached the title without meeting a blank line below it.
        non_trailer += cont;
        accepted = (recognised && trailer_lines * 3 >= non_trailer)
            || (trailer_lines > 0 && non_trailer == 0);
        first = (title_end + 1).min(log.len());
    }
    let last = last?;
    if !accepted || first >= last {
        return None;
    }
    // Group continuation lines with the trailer above them.
    let mut groups: Vec<BlockLine> = Vec::new();
    let mut open: Option<Open> = None;
    for l in &log[first..last] {
        let is_trailer = !l.text.starts_with('#') && !l.indented() && split_key(l.body()).is_some();
        if is_trailer {
            Open::flush(&mut open, &mut groups, message, l.start);
            let body = l.body();
            let (key_end, sep_end) = split_key(body).unwrap_or((0, 0));
            let rest = &body[sep_end..];
            let vstart = l.start + sep_end + (rest.len() - rest.trim_start_matches(sp).len());
            let vend = l.start + body.trim_end_matches(sp).len();
            open = Some(Open {
                start: l.start,
                key: body[..key_end].to_string(),
                vstart,
                vend: vend.max(vstart),
            });
        } else if l.indented() && !l.blank() && open.is_some() {
            if let Some(o) = open.as_mut() {
                o.vend = l.start + l.body().trim_end_matches(sp).len();
            }
        } else {
            Open::flush(&mut open, &mut groups, message, l.start);
            groups.push(BlockLine {
                range: l.start..l.end(),
                trailer: None,
            });
        }
    }
    let end = log[last - 1].end();
    Open::flush(&mut open, &mut groups, message, end);
    Some(Block {
        range: log[first].start..end,
        lines: groups,
    })
}

/// A trailer being read: its first line and the end of its value so far.
struct Open {
    start: usize,
    key: String,
    vstart: usize,
    vend: usize,
}

impl Open {
    fn flush(open: &mut Option<Open>, groups: &mut Vec<BlockLine>, message: &str, to: usize) {
        if let Some(o) = open.take() {
            let raw = o.vstart..o.vend;
            groups.push(BlockLine {
                range: o.start..to,
                trailer: Some(Parsed {
                    key: o.key,
                    value: fold(&message[raw.clone()]),
                    raw,
                }),
            });
        }
    }
}

/// A value with its continuation lines folded onto one line.
fn fold(raw: &str) -> String {
    raw.lines()
        .map(|l| l.trim_matches(sp))
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The trailers of `message`, as `git interpret-trailers --parse` prints
/// them.
pub fn parse(message: &str) -> Vec<Trailer> {
    find_block(message)
        .map(|b| {
            b.trailers()
                .map(|t| Trailer {
                    key: t.key.clone(),
                    value: t.value.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn same_key(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

fn eol_of(message: &str, block: Option<&Block>) -> &'static str {
    let probe = block.map_or(message, |b| &message[b.range.clone()]);
    if probe.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

/// Inserts `line` as the last line of the block, or as a new block.
fn insert(message: &str, block: Option<&Block>, line: &str) -> String {
    let eol = eol_of(message, block);
    let (head, tail, new_block) = match block {
        Some(b) => (&message[..b.range.end], &message[b.range.end..], false),
        None => {
            // After the last non-blank line before any divider.
            let all = lines(message);
            let end = all.iter().position(divider).unwrap_or(all.len());
            let at = all[..end]
                .iter()
                .rposition(|l| !l.blank())
                .map_or(0, |i| all[i].end());
            (&message[..at], &message[at..], true)
        }
    };
    let mut out = String::with_capacity(message.len() + line.len() + 4);
    out.push_str(head);
    let ended = head.is_empty() || head.ends_with('\n');
    if !ended {
        out.push_str(eol);
    }
    if new_block && !head.is_empty() {
        out.push_str(eol);
    }
    out.push_str(line);
    // A message that had no final newline keeps having none.
    if !tail.is_empty() || ended {
        out.push_str(eol);
    }
    out.push_str(tail);
    out
}

/// `message` with the trailer `key: value` added per `if_exists`.
pub fn add(message: &str, key: &str, value: &str, if_exists: TrailerIfExists) -> String {
    let block = find_block(message);
    let line = format!("{key}: {value}");
    let same: Vec<&Parsed> = block
        .iter()
        .flat_map(Block::trailers)
        .filter(|t| same_key(&t.key, key))
        .collect();
    match if_exists {
        TrailerIfExists::Skip if !same.is_empty() => message.to_string(),
        TrailerIfExists::Add if same.iter().any(|t| t.value == fold(value)) => message.to_string(),
        TrailerIfExists::Replace if !same.is_empty() => {
            if same.len() == 1 && same[0].value == fold(value) {
                return message.to_string();
            }
            let removed = remove(message, key, None);
            let block = find_block(&removed);
            insert(&removed, block.as_ref(), &line)
        }
        _ => insert(message, block.as_ref(), &line),
    }
}

/// `message` without the trailers named `key` (and whose value matches
/// `only`, when given). When that empties the block, the blank line that
/// separated it goes too, so removing what `add` put there restores the
/// message.
pub fn remove(message: &str, key: &str, only: Option<&CompiledPattern>) -> String {
    let Some(block) = find_block(message) else {
        return message.to_string();
    };
    let doomed = |l: &BlockLine| {
        l.trailer
            .as_ref()
            .is_some_and(|t| same_key(&t.key, key) && only.map_or(true, |p| p.is_match(&t.value)))
    };
    if !block.lines.iter().any(doomed) {
        return message.to_string();
    }
    let head = &message[..block.range.start];
    let tail = &message[block.range.end..];
    let kept: String = block
        .lines
        .iter()
        .filter(|l| !doomed(l))
        .map(|l| &message[l.range.clone()])
        .collect();
    // A message without a final newline stays without one.
    let no_final_nl = tail.is_empty() && !message.ends_with('\n');
    let strip = |mut s: String| {
        if no_final_nl && s.ends_with('\n') {
            s.pop();
            if s.ends_with('\r') {
                s.pop();
            }
        }
        s
    };
    if block.lines.iter().all(doomed) {
        // Drop the blank line(s) between the paragraph above and the block.
        let body = head.trim_end_matches(['\r', '\n']);
        let mut out = String::from(body);
        if head.len() > body.len() || !tail.is_empty() {
            out.push_str(if head.contains("\r\n") { "\r\n" } else { "\n" });
        }
        out.push_str(tail);
        return strip(out);
    }
    strip(format!("{head}{kept}{tail}"))
}

/// `message` with every trailer's raw value passed through `f(key, folded
/// value, raw value)`; `Some(new)` replaces the raw value (continuation
/// lines included).
pub fn map_values(message: &str, mut f: impl FnMut(&str, &str, &str) -> Option<String>) -> String {
    let Some(block) = find_block(message) else {
        return message.to_string();
    };
    let mut out = String::with_capacity(message.len());
    let mut at = 0;
    for t in block.trailers() {
        if let Some(new) = f(&t.key, &t.value, &message[t.raw.clone()]) {
            out.push_str(&message[at..t.raw.start]);
            out.push_str(&new);
            at = t.raw.end;
        }
    }
    out.push_str(&message[at..]);
    out
}

/// Replaces text inside the values of the `key` trailers.
pub fn map_text(message: &str, key: &str, find: &CompiledPattern, with: &str) -> String {
    map_values(message, |k, _, raw| {
        if !same_key(k, key) {
            return None;
        }
        let new = find.replace_all(raw, with);
        (new != raw).then_some(new)
    })
}

/// `Name <email>` as a trailer value; `None` for anything else.
pub fn person_of(value: &str) -> Option<Ident> {
    let value = value.trim();
    let inner = value.strip_suffix('>')?;
    let open = inner.rfind('<')?;
    let (name, email) = (&inner[..open], &inner[open + 1..]);
    if email.contains(['<', '>']) {
        return None;
    }
    Some(Ident::new(name.trim(), email.trim()))
}

/// `Name <email>` for a trailer value.
pub fn person_text(ident: &Ident) -> String {
    if ident.name.is_empty() {
        format!("<{}>", ident.email)
    } else {
        format!("{} <{}>", ident.name, ident.email)
    }
}

/// The identity trailers (value `Name <email>`, any key) through `f`; `f`
/// returns the new identity or `None` to leave the trailer.
pub fn map_identities(message: &str, mut f: impl FnMut(&Ident) -> Option<Ident>) -> String {
    map_values(message, |_, folded, raw| {
        // Only a value that is exactly `Name <email>` (on one line).
        if raw.contains('\n') {
            return None;
        }
        let old = person_of(folded)?;
        let new = f(&old)?;
        (new != old).then(|| person_text(&new))
    })
}

#[cfg(test)]
mod tests;
