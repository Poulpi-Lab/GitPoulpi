//! Textile to Markdown (Redmine's default markup; read only).
//!
//! Covers what issue text uses: headings, bold/italic/strike/code, links,
//! images, quotes, lists, tables, `bc.` code blocks and `<pre>`. Attributes
//! (`p(class).`, `%{color:red}`) are dropped, text kept.

use std::sync::OnceLock;

use regex::Regex;

use super::{Block, Lines, sanitize, split_spans, wrap_pairs};

fn re(cell: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).expect("static pattern"))
}

fn heading_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"^h([1-6])[^.\s]*\.\s+(.*)$")
}

fn para_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"^(?:p|div)[^.\s]*\.\s+(.*)$")
}

fn bq_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"^bq[^.\s]*\.\s+(.*)$")
}

fn code_block_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"^(?:bc|pre)\.\.?\s*(.*)$")
}

fn list_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"^([*#]+)\s+(.*)$")
}

fn link_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r#""([^"\n]+?)(?:\([^)\n]*\))?":(\S+)"#)
}

fn image_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"!([^\s!()]+)(?:\(([^)\n]*)\))?!")
}

fn span_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"%(?:\{[^}]*\})?([^%\n]+)%")
}

/// Trailing punctuation that belongs to the sentence, not to a URL.
fn split_url_tail(url: &str) -> (&str, &str) {
    let trimmed = url.trim_end_matches(['.', ',', ';', ':', '!', '?', ')']);
    (trimmed, &url[trimmed.len()..])
}

fn inline(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for (is_code, piece) in split_spans(text, "@", "@") {
        // `@name@` only when it looks like code: no blank inside.
        if is_code && !piece.chars().any(char::is_whitespace) {
            out.push('`');
            out.push_str(piece);
            out.push('`');
        } else if is_code {
            out.push_str(&inline_plain(&format!("@{piece}@")));
        } else {
            out.push_str(&inline_plain(piece));
        }
    }
    out
}

fn inline_plain(text: &str) -> String {
    let s = image_re()
        .replace_all(text, |c: &regex::Captures<'_>| {
            let src = &c[1];
            if src.contains('.') || src.contains('/') {
                format!("![{}]({src})", c.get(2).map_or("", |m| m.as_str()))
            } else {
                c[0].to_string()
            }
        })
        .into_owned();
    let s = link_re()
        .replace_all(&s, |c: &regex::Captures<'_>| {
            let (url, tail) = split_url_tail(&c[2]);
            format!("[{}]({url}){tail}", &c[1])
        })
        .into_owned();
    let s = span_re().replace_all(&s, "$1").into_owned();
    let s = wrap_pairs(&s, '*', "**", "**");
    let s = wrap_pairs(&s, '-', "~~", "~~");
    let s = wrap_pairs(&s, '+', "<ins>", "</ins>");
    let s = wrap_pairs(&s, '^', "<sup>", "</sup>");
    wrap_pairs(&s, '~', "<sub>", "</sub>")
}

fn table_cells(line: &str) -> (Vec<String>, bool) {
    let t = line.trim();
    let t = t.strip_prefix('|').unwrap_or(t);
    let t = t.strip_suffix('|').unwrap_or(t);
    let mut header = false;
    let cells = t
        .split('|')
        .map(|c| {
            let c = c.trim();
            let c = match c.strip_prefix("_.") {
                Some(rest) => {
                    header = true;
                    rest
                }
                None => c,
            };
            inline(c.trim())
        })
        .collect();
    (cells, header)
}

fn is_block_start(line: &str) -> bool {
    let t = line.trim();
    t.is_empty()
        || heading_re().is_match(t)
        || bq_re().is_match(t)
        || para_re().is_match(t)
        || code_block_re().is_match(t)
        || list_re().is_match(t)
        || t.starts_with('|')
        || t.starts_with("<pre")
        || t == "---"
}

/// Textile to Markdown.
pub fn textile_to_markdown(src: &str) -> String {
    let src = sanitize(src);
    let input: Vec<&str> = src.lines().collect();
    let mut out = Lines::default();
    let mut table_open = false;
    let mut i = 0;
    while i < input.len() {
        let line = input[i];
        i += 1;
        let t = line.trim();
        if t.is_empty() {
            out.blank();
            table_open = false;
            continue;
        }
        // <pre> ... </pre>, also <pre><code>
        if t.starts_with("<pre") {
            out.blank();
            out.push(Block::Code, "```".to_string());
            let mut cur = strip_tags(t.trim_start_matches(|c| c != '>').trim_start_matches('>'));
            let mut first = true;
            loop {
                let closed = cur.contains("</pre>") || cur.contains("</code>");
                let body = cur
                    .split("</")
                    .next()
                    .unwrap_or("")
                    .to_string();
                if !body.is_empty() || (!closed && !first) {
                    out.push(Block::Code, super::decode_entities(&body));
                }
                first = false;
                if closed || i >= input.len() {
                    break;
                }
                cur = strip_tags(input[i]);
                i += 1;
            }
            out.push(Block::Code, "```".to_string());
            continue;
        }
        if let Some(c) = code_block_re().captures(t) {
            out.blank();
            out.push(Block::Code, "```".to_string());
            if !c[1].is_empty() {
                out.push(Block::Code, c[1].to_string());
            }
            while i < input.len() && !input[i].trim().is_empty() {
                out.push(Block::Code, input[i].to_string());
                i += 1;
            }
            out.push(Block::Code, "```".to_string());
            continue;
        }
        if let Some(c) = heading_re().captures(t) {
            let level: usize = c[1].parse().unwrap_or(1);
            out.push(Block::Heading, format!("{} {}", "#".repeat(level), inline(&c[2])));
        } else if let Some(c) = bq_re().captures(t) {
            out.push(Block::Quote, format!("> {}", inline(&c[1])));
        } else if t == "---" {
            out.push(Block::Rule, "---".to_string());
        } else if t.starts_with('|') {
            let (cells, header) = table_cells(t);
            let row = format!("| {} |", cells.join(" | "));
            out.push(Block::Table, row);
            if !table_open {
                table_open = true;
                let _ = header;
                out.push(Block::Table, format!("| {} |", vec!["---"; cells.len()].join(" | ")));
            }
        } else if let Some(c) = list_re().captures(t) {
            let marks = &c[1];
            let indent: usize = marks[..marks.len() - 1]
                .chars()
                .map(|m| if m == '#' { 3 } else { 2 })
                .sum();
            let bullet = if marks.ends_with('#') { "1." } else { "-" };
            out.push(
                Block::List,
                format!("{}{bullet} {}", " ".repeat(indent), inline(&c[2])),
            );
        } else {
            let body = para_re().captures(t).map_or(t, |c| c.get(1).map_or(t, |m| m.as_str()));
            let mut text = inline(body);
            if input.get(i).is_some_and(|n| !is_block_start(n)) {
                text.push_str("  ");
            }
            out.push(Block::Para, text);
        }
    }
    out.finish()
}

fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    let mut rest = s;
    // Keep closing markers so the caller can find the end of the block.
    while let Some(c) = rest.chars().next() {
        if c == '<' && !rest.starts_with("</pre") && !rest.starts_with("</code") {
            in_tag = true;
        }
        if !in_tag {
            out.push(c);
        }
        if c == '>' {
            in_tag = false;
        }
        rest = &rest[c.len_utf8()..];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table() {
        let cases: &[(&str, &str)] = &[
            ("h1. Title", "# Title"),
            ("h2(#anchor). Sub", "## Sub"),
            ("p. plain", "plain"),
            ("a *bold* _it_ -gone- @code@ end", "a **bold** _it_ ~~gone~~ `code` end"),
            ("see \"the docs\":https://x.io/d.", "see [the docs](https://x.io/d)."),
            ("!shot.png!", "![](shot.png)"),
            ("!shot.png(alt text)!", "![alt text](shot.png)"),
            ("Wow! really!", "Wow! really!"),
            ("* one\n** two\n* three", "- one\n  - two\n- three"),
            ("# a\n## b", "1. a\n   1. b"),
            ("bq. quoted", "> quoted"),
            ("---", "---"),
            ("%{color:red}hot%", "hot"),
            ("mail me@x.org or you@y.org", "mail me@x.org or you@y.org"),
            ("one\ntwo", "one  \ntwo"),
            ("#12 stays and [[Wiki]] too", "#12 stays and [[Wiki]] too"),
        ];
        for (input, want) in cases {
            assert_eq!(textile_to_markdown(input), *want, "{input:?}");
        }
    }

    #[test]
    fn blocks() {
        assert_eq!(
            textile_to_markdown("bc. let a = 1;\nlet b = 2;\n\nafter"),
            "```\nlet a = 1;\nlet b = 2;\n```\n\nafter"
        );
        assert_eq!(
            textile_to_markdown("<pre>\nx &lt; y\n</pre>"),
            "```\nx < y\n```"
        );
        assert_eq!(
            textile_to_markdown("|_. a |_. b |\n| 1 | 2 |"),
            "| a | b |\n| --- | --- |\n| 1 | 2 |"
        );
    }

    #[test]
    fn strips_escapes() {
        assert_eq!(textile_to_markdown("x\u{1b}]0;t\u{7}y"), "x]0;ty");
    }
}
