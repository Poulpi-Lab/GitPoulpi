//! Markup conversion for issue and pull-request text (plan `conversations`).
//!
//! Hosts speak different markup: Jira wiki, Textile (Redmine), HTML (Azure
//! DevOps Server, some trackers) and flavours of Markdown. Everything the
//! app displays is normalised to Markdown first, and text the user writes is
//! converted back for hosts that do not take Markdown. All of it is pure and
//! lives here so the TUI, the CLI and MCP agree.
//!
//! Every importer starts with [`sanitize`]: no escape sequence or control
//! character reaches a renderer.
//!
//! The converters are deliberately forgiving rather than complete: what they
//! do not understand stays as readable text.

mod html_lite;
mod jira_wiki;
mod sanitize;
mod textile;

pub use html_lite::{html_to_markdown, markdown_to_html};
pub use jira_wiki::{jira_to_markdown, markdown_to_jira};
pub use sanitize::sanitize;
pub use textile::textile_to_markdown;

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Rewrites `marker text marker` pairs (`*bold*`) to `open text close`.
///
/// An opener sits at the start or after a non-word character and is followed
/// by a non-blank; a closer follows a non-blank and is followed by a non-word
/// character. Pairs never span a newline, so `2 * 3 * 4` and `a-b-c` stay.
pub(crate) fn wrap_pairs(text: &str, marker: char, open: &str, close: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() + 8);
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == marker && opens_at(&chars, i, marker) {
            if let Some(j) = closer_for(&chars, i, marker) {
                out.push_str(open);
                out.extend(&chars[i + 1..j]);
                out.push_str(close);
                i = j + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn opens_at(chars: &[char], i: usize, marker: char) -> bool {
    let prev_ok = i == 0 || !is_word(chars[i - 1]) && chars[i - 1] != marker;
    let next_ok = chars
        .get(i + 1)
        .is_some_and(|n| !n.is_whitespace() && *n != marker);
    prev_ok && next_ok
}

fn closer_for(chars: &[char], i: usize, marker: char) -> Option<usize> {
    let mut j = i + 1;
    while j < chars.len() {
        let c = chars[j];
        if c == '\n' {
            return None;
        }
        if c == marker
            && j > i + 1
            && !chars[j - 1].is_whitespace()
            && chars[j - 1] != marker
            && chars.get(j + 1).is_none_or(|n| !is_word(*n) && *n != marker)
        {
            return Some(j);
        }
        j += 1;
    }
    None
}

/// Splits `text` at spans delimited by `start` and `end` (code spans), as
/// `(is_code, piece)`; the delimiters are not part of the pieces. An
/// unterminated opener is plain text.
pub(crate) fn split_spans<'a>(text: &'a str, start: &str, end: &str) -> Vec<(bool, &'a str)> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(a) = rest.find(start) {
        let after = &rest[a + start.len()..];
        match after.find(end) {
            Some(b) if b > 0 && !after[..b].contains('\n') => {
                if a > 0 {
                    out.push((false, &rest[..a]));
                }
                out.push((true, &after[..b]));
                rest = &after[b + end.len()..];
            }
            _ => {
                out.push((false, &rest[..a + start.len()]));
                rest = after;
            }
        }
    }
    if !rest.is_empty() {
        out.push((false, rest));
    }
    out
}

/// What a produced Markdown line is, to decide where blank lines go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Block {
    Para,
    List,
    Heading,
    Quote,
    Table,
    Code,
    Rule,
}

/// Collects lines and puts one blank line between blocks of different kinds.
#[derive(Default)]
pub(crate) struct Lines {
    lines: Vec<String>,
    last: Option<Block>,
}

impl Lines {
    pub(crate) fn push(&mut self, kind: Block, line: String) {
        if let Some(last) = self.last {
            let same_run = last == kind && matches!(
                kind,
                Block::List | Block::Table | Block::Quote | Block::Para | Block::Code
            );
            if !same_run && !self.lines.last().is_some_and(String::is_empty) {
                self.lines.push(String::new());
            }
        }
        self.lines.push(line);
        self.last = Some(kind);
    }

    pub(crate) fn blank(&mut self) {
        if !self.lines.last().is_some_and(String::is_empty) && !self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.last = None;
    }

    pub(crate) fn finish(self) -> String {
        let mut s = self.lines.join("\n");
        while s.ends_with('\n') {
            s.pop();
        }
        s
    }
}

/// Decodes the handful of HTML entities that show up in issue text.
pub(crate) fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(a) = rest.find('&') {
        out.push_str(&rest[..a]);
        let tail = &rest[a..];
        let Some(semi) = tail.find(';').filter(|s| *s <= 10) else {
            out.push('&');
            rest = &tail[1..];
            continue;
        };
        let name = &tail[1..semi];
        let decoded = match name {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" | "#39" => Some('\''),
            "nbsp" => Some(' '),
            _ => name
                .strip_prefix('#')
                .and_then(|n| {
                    n.strip_prefix(['x', 'X'])
                        .map(|h| u32::from_str_radix(h, 16))
                        .unwrap_or_else(|| n.parse())
                        .ok()
                })
                .and_then(char::from_u32),
        };
        match decoded {
            Some(c) => {
                out.push(c);
                rest = &tail[semi + 1..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_pairs_table() {
        let cases: &[(&str, char, &str, &str)] = &[
            ("a *b* c", '*', "a **b** c", ""),
            ("*b*", '*', "**b**", ""),
            ("2 * 3 * 4", '*', "2 * 3 * 4", ""),
            ("a-b-c", '-', "a-b-c", ""),
            ("x -gone- y", '-', "x ~~gone~~ y", ""),
            ("*open\nclose*", '*', "*open\nclose*", ""),
            ("snake*case*x", '*', "snake*case*x", ""),
        ];
        for (input, marker, want, _) in cases {
            let (open, close) = if *marker == '*' {
                ("**", "**")
            } else {
                ("~~", "~~")
            };
            assert_eq!(wrap_pairs(input, *marker, open, close), *want, "{input:?}");
        }
    }

    #[test]
    fn entities() {
        assert_eq!(decode_entities("a &amp; b &lt;c&gt; &#65;&#x42; &bogus; &"), "a & b <c> AB &bogus; &");
    }
}
