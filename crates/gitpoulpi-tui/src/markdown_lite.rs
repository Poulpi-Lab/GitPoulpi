//! Just enough Markdown for a request's description and comments: the
//! pieces people actually write, shown as styled text instead of raw
//! markers (`**moons**` is "moons" in bold).
//!
//! Supported: headings (`#`), bullet and numbered lists, block quotes,
//! fenced code blocks, and inline `**bold**`, `__bold__`, `*italic*`,
//! `_italic_`, `~~strikethrough~~`, `` `code` `` and `[text](url)` / `![alt](url)` (the text is
//! shown, the address is dropped). Anything else is plain text. It is a
//! display helper: lenient, never failing, and it keeps every word of the
//! source.
//!
//! [`render`] returns lines that are already wrapped to a width, each a list
//! of spans carrying a [`Look`]; the caller maps looks onto its theme.

// Only the in-app review shows Markdown.
#![cfg_attr(not(feature = "review"), allow(dead_code))]

use crate::ui_kit::display_width;

/// How a run of text is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Look {
    Plain,
    Bold,
    Italic,
    BoldItalic,
    /// `~~struck out~~`.
    Strike,
    Code,
    /// A link's text.
    Link,
    /// A heading line.
    Heading,
    /// Text inside a block quote.
    Quote,
    /// A list bullet or a quote bar.
    Marker,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Span {
    pub(crate) text: String,
    pub(crate) look: Look,
}

/// One wrapped screen line.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Line {
    pub(crate) spans: Vec<Span>,
}

impl Line {
    /// The line without styling (tests, width checks).
    #[cfg(test)]
    pub(crate) fn text(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }

    fn push(&mut self, text: &str, look: Look) {
        if text.is_empty() {
            return;
        }
        match self.spans.last_mut() {
            Some(last) if last.look == look => last.text.push_str(text),
            _ => self.spans.push(Span {
                text: text.to_string(),
                look,
            }),
        }
    }
}

/// Inline markup of one source line, as styled characters.
fn inline(src: &str, base: Look) -> Vec<(char, Look)> {
    let chars: Vec<char> = src.chars().collect();
    let mut out: Vec<(char, Look)> = Vec::new();
    let (mut bold, mut italic, mut strike) = (false, false, false);
    let look = |bold: bool, italic: bool| match (bold, italic, base) {
        (true, true, _) => Look::BoldItalic,
        (true, false, _) => Look::Bold,
        (false, true, _) => Look::Italic,
        (false, false, b) => b,
    };
    let styled = |strike: bool, bold: bool, italic: bool| {
        if strike {
            Look::Strike
        } else {
            look(bold, italic)
        }
    };
    let word = |c: Option<&char>| c.is_some_and(|c| c.is_alphanumeric());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1);
        // Backslash escapes a markup character.
        if c == '\\' && next.is_some_and(|n| "\\`*_[]()#>-!".contains(*n)) {
            out.push((chars[i + 1], styled(strike, bold, italic)));
            i += 2;
            continue;
        }
        // `code`
        if c == '`' {
            if let Some(end) = chars[i + 1..].iter().position(|&x| x == '`') {
                if end > 0 {
                    for &ch in &chars[i + 1..i + 1 + end] {
                        out.push((ch, Look::Code));
                    }
                    i += end + 2;
                    continue;
                }
            }
        }
        // [text](url) and ![alt](url)
        if c == '[' || (c == '!' && next == Some(&'[')) {
            let open = if c == '!' { i + 1 } else { i };
            if let Some(close) = chars[open..].iter().position(|&x| x == ']') {
                let close = open + close;
                if chars.get(close + 1) == Some(&'(') {
                    if let Some(end) = chars[close + 2..].iter().position(|&x| x == ')') {
                        let label: String = chars[open + 1..close].iter().collect();
                        let label = if label.trim().is_empty() {
                            chars[close + 2..close + 2 + end].iter().collect()
                        } else {
                            label
                        };
                        for ch in label.chars() {
                            out.push((ch, Look::Link));
                        }
                        i = close + 2 + end + 1;
                        continue;
                    }
                }
            }
        }
        // ~~strikethrough~~
        if c == '~' && next == Some(&'~') {
            let has_close = chars[i + 2..]
                .windows(2)
                .any(|w| w[0] == '~' && w[1] == '~');
            if strike || (has_close && chars.get(i + 2).is_some_and(|n| !n.is_whitespace())) {
                strike = !strike;
                i += 2;
                continue;
            }
        }
        // **bold** / __bold__
        if (c == '*' || c == '_') && next == Some(&c) {
            let closing = bold;
            let can_open = !word(i.checked_sub(1).and_then(|p| chars.get(p))) || c == '*';
            let has_close = chars[i + 2..].windows(2).any(|w| w[0] == c && w[1] == c);
            if closing
                || (can_open && has_close && chars.get(i + 2).is_some_and(|n| !n.is_whitespace()))
            {
                bold = !bold;
                i += 2;
                continue;
            }
            // A doubled marker that opens nothing is plain text.
            out.push((c, styled(strike, bold, italic)));
            out.push((c, styled(strike, bold, italic)));
            i += 2;
            continue;
        }
        // *italic* / _italic_
        if c == '*' || c == '_' {
            let prev = i.checked_sub(1).and_then(|p| chars.get(p));
            let intra = c == '_' && word(prev) && word(next);
            let opening_ok = next.is_some_and(|n| !n.is_whitespace()) && !intra;
            let has_close = chars[i + 1..].contains(&c);
            if italic || (opening_ok && has_close) {
                if italic && prev.is_some_and(|p| p.is_whitespace()) && !has_close {
                    // A lone marker after a space: plain text.
                } else {
                    italic = !italic;
                    i += 1;
                    continue;
                }
            }
        }
        out.push((c, styled(strike, bold, italic)));
        i += 1;
    }
    out
}

/// Greedy word wrap of styled characters; `first` and `rest` are the
/// prefixes (a bullet, a quote bar) of the first and the later lines.
fn wrap(
    chars: &[(char, Look)],
    width: usize,
    first: (&str, Look),
    rest: (&str, Look),
) -> Vec<Line> {
    let mut lines: Vec<Line> = Vec::new();
    let mut line = Line::default();
    line.push(first.0, first.1);
    let mut used = display_width(first.0);
    let prefix_w = display_width(rest.0);
    let mut fresh = true; // nothing but the prefix on the line yet
    let start_line =
        |lines: &mut Vec<Line>, line: &mut Line, used: &mut usize, fresh: &mut bool| {
            lines.push(std::mem::take(line));
            line.push(rest.0, rest.1);
            *used = prefix_w;
            *fresh = true;
        };
    for word in chars.split(|(c, _)| c.is_whitespace()) {
        if word.is_empty() {
            continue;
        }
        let word_w: usize = word
            .iter()
            .map(|(c, _)| display_width(&c.to_string()))
            .sum();
        let gap = usize::from(!fresh);
        if !fresh && used + gap + word_w > width {
            start_line(&mut lines, &mut line, &mut used, &mut fresh);
        }
        if !fresh {
            // The space is plain unless the words around it share a look
            // that shows on a blank (a heading, a quote, code).
            let before = line.spans.last().map_or(Look::Plain, |s| s.look);
            let look = if before == word[0].1
                && matches!(
                    before,
                    Look::Heading | Look::Quote | Look::Code | Look::Link
                ) {
                before
            } else {
                Look::Plain
            };
            line.push(" ", look);
            used += 1;
        }
        for &(c, look) in word {
            let w = display_width(&c.to_string());
            if used + w > width && !fresh {
                start_line(&mut lines, &mut line, &mut used, &mut fresh);
            }
            line.push(&c.to_string(), look);
            used += w;
            fresh = false;
        }
    }
    lines.push(line);
    lines
}

/// `text` as wrapped, styled lines (at least `width` 8 columns).
pub(crate) fn render(text: &str, width: usize) -> Vec<Line> {
    let width = width.max(8);
    let mut out: Vec<Line> = Vec::new();
    let mut in_code = false;
    for source in text.replace("\r\n", "\n").lines() {
        let trimmed = source.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            // Verbatim; a line longer than the width is cut, not wrapped.
            let mut line = Line::default();
            let mut used = 0;
            for c in source.replace('\t', "    ").chars() {
                let w = display_width(&c.to_string());
                if used + w > width {
                    break;
                }
                line.push(&c.to_string(), Look::Code);
                used += w;
            }
            out.push(line);
            continue;
        }
        if trimmed.is_empty() {
            out.push(Line::default());
            continue;
        }
        // Headings.
        let hashes = trimmed.chars().take_while(|c| *c == '#').count();
        if (1..=6).contains(&hashes) && trimmed[hashes..].starts_with(' ') {
            let body = trimmed[hashes..].trim().trim_end_matches('#').trim();
            let chars: Vec<(char, Look)> = inline(body, Look::Heading)
                .into_iter()
                .map(|(c, l)| (c, if l == Look::Plain { Look::Heading } else { l }))
                .collect();
            out.extend(wrap(&chars, width, ("", Look::Plain), ("", Look::Plain)));
            continue;
        }
        // Block quotes.
        if let Some(body) = trimmed.strip_prefix('>') {
            let chars = inline(body.trim_start(), Look::Quote);
            out.extend(wrap(
                &chars,
                width,
                ("\u{2502} ", Look::Marker),
                ("\u{2502} ", Look::Marker),
            ));
            continue;
        }
        // Lists.
        let indent = source.len() - trimmed.len();
        let bullet = ["- ", "* ", "+ "]
            .iter()
            .find_map(|m| trimmed.strip_prefix(m))
            .map(|rest| ("\u{2022} ".to_string(), rest));
        let numbered = trimmed.split_once(". ").and_then(|(n, rest)| {
            (!n.is_empty() && n.len() <= 3 && n.chars().all(|c| c.is_ascii_digit()))
                .then(|| (format!("{n}. "), rest))
        });
        if let Some((marker, body)) = bullet.or(numbered) {
            let pad = " ".repeat(indent.min(8) + display_width(&marker));
            let lead = format!("{}{marker}", " ".repeat(indent.min(8)));
            let chars = inline(body.trim(), Look::Plain);
            // The indent stays plain, the marker is drawn as one.
            let mut lines = wrap(&chars, width, (&lead, Look::Marker), (&pad, Look::Plain));
            if let Some(first) = lines.first_mut() {
                first.spans.retain(|s| !s.text.is_empty());
            }
            out.extend(lines);
            continue;
        }
        let chars = inline(source.trim(), Look::Plain);
        out.extend(wrap(&chars, width, ("", Look::Plain), ("", Look::Plain)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(src: &str, width: usize) -> Vec<String> {
        render(src, width).iter().map(Line::text).collect()
    }

    fn looks(src: &str) -> Vec<(String, Look)> {
        render(src, 80)[0]
            .spans
            .iter()
            .map(|s| (s.text.clone(), s.look))
            .collect()
    }

    #[test]
    fn bold_italic_and_code_lose_their_markers() {
        assert_eq!(
            looks("Adds the **moons** column and `Vec` and *soon* more"),
            vec![
                ("Adds the ".into(), Look::Plain),
                ("moons".into(), Look::Bold),
                (" column and ".into(), Look::Plain),
                ("Vec".into(), Look::Code),
                (" and ".into(), Look::Plain),
                ("soon".into(), Look::Italic),
                (" more".into(), Look::Plain),
            ]
        );
        assert_eq!(
            looks("__bold__ and _it_"),
            vec![
                ("bold".into(), Look::Bold),
                (" and ".into(), Look::Plain),
                ("it".into(), Look::Italic),
            ]
        );
    }

    #[test]
    fn strikethrough_loses_its_markers_and_a_lone_pair_stays() {
        assert_eq!(
            looks("keep ~~old~~ new"),
            vec![
                ("keep ".into(), Look::Plain),
                ("old".into(), Look::Strike),
                (" new".into(), Look::Plain),
            ]
        );
        assert_eq!(
            texts("a ~~ b and ~~unclosed", 80),
            ["a ~~ b and ~~unclosed"]
        );
    }

    #[test]
    fn snake_case_and_lone_markers_are_left_alone() {
        assert_eq!(
            texts("use my_var_name and 2 * 3 = 6", 80),
            ["use my_var_name and 2 * 3 = 6"]
        );
        assert_eq!(texts("**unclosed bold", 80), ["**unclosed bold"]);
        assert_eq!(texts("a ** b", 80), ["a ** b"]);
    }

    #[test]
    fn links_show_their_text_and_images_their_alt() {
        assert_eq!(
            looks("see [the docs](https://x.test/d) now"),
            vec![
                ("see ".into(), Look::Plain),
                ("the docs".into(), Look::Link),
                (" now".into(), Look::Plain),
            ]
        );
        assert_eq!(
            texts("![diagram](a.png) and [](https://u.test)", 80),
            ["diagram and https://u.test"]
        );
    }

    #[test]
    fn headings_lists_and_quotes_are_laid_out() {
        let lines = texts("# Title\n\n- one\n* two\n1. first\n> quoted words", 80);
        assert_eq!(
            lines,
            [
                "Title",
                "",
                "\u{2022} one",
                "\u{2022} two",
                "1. first",
                "\u{2502} quoted words"
            ]
        );
        assert_eq!(render("## Sub", 80)[0].spans[0].look, Look::Heading);
        assert_eq!(render("- x", 80)[0].spans[0].look, Look::Marker);
    }

    #[test]
    fn a_long_bullet_wraps_under_its_text() {
        let lines = texts("- alpha beta gamma delta epsilon", 14);
        assert_eq!(lines, ["\u{2022} alpha beta", "  gamma delta", "  epsilon"]);
    }

    #[test]
    fn code_blocks_are_verbatim_and_never_wrapped() {
        let lines = texts("```rust\nlet x = **1**;\n```\nafter", 80);
        assert_eq!(lines, ["let x = **1**;", "after"]);
        assert_eq!(render("```\ncode\n```", 80)[0].spans[0].look, Look::Code);
    }

    #[test]
    fn wrapping_keeps_every_word_and_respects_the_width() {
        let src = "The quick brown fox jumps over the lazy dog again and again";
        for width in [10usize, 17, 30] {
            let lines = texts(src, width);
            assert!(lines.iter().all(|l| display_width(l) <= width), "{lines:?}");
            assert_eq!(lines.join(" "), src);
        }
        // A word longer than the width is broken, nothing is lost.
        let lines = texts("abcdefghijklmnopqrstuvwxyz", 10);
        assert_eq!(lines.concat(), "abcdefghijklmnopqrstuvwxyz");
        assert!(lines.iter().all(|l| display_width(l) <= 10));
    }

    #[test]
    fn escapes_and_empty_input_are_fine() {
        assert_eq!(texts("a \\*star\\* b", 80), ["a *star* b"]);
        assert!(render("", 80).is_empty());
        assert_eq!(texts("one\n\ntwo", 80), ["one", "", "two"]);
    }
}
