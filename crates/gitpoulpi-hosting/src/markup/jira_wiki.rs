//! Jira wiki markup, both directions.
//!
//! Reading: Jira (Cloud over API v2, Data Center) returns wiki markup for
//! descriptions and comments; it is turned into Markdown for display.
//! Writing: the user writes Markdown, which goes to Jira as wiki markup
//! (owner decision 2 of the plan).

use std::sync::OnceLock;

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use regex::Regex;

use super::{Block, Lines, sanitize, split_spans, wrap_pairs};

fn re(cell: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).expect("static pattern"))
}

fn heading_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"^h([1-6])\.\s*(.*)$")
}

fn list_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"^\s*([*#-]+)\s+(.*)$")
}

fn bq_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"^bq\.\s*(.*)$")
}

fn rule_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"^\s*-{4,}\s*$")
}

fn link_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"\[([^\[\]|\n]*)\|([^\[\]\s|]+)(?:\|[^\[\]\n]*)?\]")
}

fn bare_link_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"\[((?:https?|ftp|mailto):[^\[\]\s|]+)\]")
}

fn mention_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"\[~(?:accountid:)?([^\]\s]+)\]")
}

fn image_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"!([^\s!|]+?)(?:\|[^!\n]*)?!")
}

fn macro_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"\{(?:color|anchor|panel|noformat|code|quote|loading-align)(?::[^}]*)?\}")
}

/// A code fence for the language named by a `{code:...}` macro.
fn fence_lang(macro_args: &str) -> String {
    let args = macro_args.trim_start_matches(':');
    if args.is_empty() {
        return String::new();
    }
    for part in args.split('|') {
        if let Some(lang) = part.strip_prefix("language=") {
            return lang.to_string();
        }
    }
    let first = args.split('|').next().unwrap_or("");
    if first.contains('=') {
        String::new()
    } else {
        first.to_string()
    }
}

fn inline(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for (is_code, piece) in split_spans(text, "{{", "}}") {
        if is_code {
            out.push('`');
            out.push_str(piece);
            out.push('`');
        } else {
            out.push_str(&inline_plain(piece));
        }
    }
    out
}

fn inline_plain(text: &str) -> String {
    let s = text.replace("\\\\", "  \n");
    let s = mention_re().replace_all(&s, "@$1").into_owned();
    let s = image_re()
        .replace_all(&s, |c: &regex::Captures<'_>| {
            let src = &c[1];
            if src.contains('.') || src.contains('/') {
                format!("![]({src})")
            } else {
                c[0].to_string()
            }
        })
        .into_owned();
    let s = link_re()
        .replace_all(&s, |c: &regex::Captures<'_>| {
            let (text, dest) = (&c[1], &c[2]);
            let text = if text.is_empty() { dest } else { text };
            format!("[{text}]({dest})")
        })
        .into_owned();
    let s = bare_link_re().replace_all(&s, "<$1>").into_owned();
    let s = macro_re().replace_all(&s, "").into_owned();
    let s = wrap_pairs(&s, '*', "**", "**");
    let s = wrap_pairs(&s, '-', "~~", "~~");
    let s = wrap_pairs(&s, '+', "<ins>", "</ins>");
    let s = wrap_pairs(&s, '^', "<sup>", "</sup>");
    wrap_pairs(&s, '~', "<sub>", "</sub>")
}

fn cells(line: &str) -> Vec<String> {
    let t = line.trim();
    let header = t.starts_with("||");
    let sep = if header { "||" } else { "|" };
    let t = t.trim_start_matches('|');
    let t = t.strip_suffix(sep).unwrap_or(t);
    let t = t.trim_end_matches('|');
    t.split(sep).map(|c| inline(c.trim())).collect()
}

fn table_line(cells: &[String]) -> String {
    format!("| {} |", cells.join(" | "))
}

/// Does a plain paragraph line stand here (no block syntax)?
fn is_plain(line: &str) -> bool {
    let t = line.trim_end();
    !t.is_empty()
        && !heading_re().is_match(t)
        && !list_re().is_match(t)
        && !bq_re().is_match(t)
        && !rule_re().is_match(t)
        && !t.trim_start().starts_with('|')
        && !t.trim_start().starts_with("{code")
        && !t.trim_start().starts_with("{noformat")
        && !t.trim_start().starts_with("{quote")
}

/// Jira wiki markup to Markdown.
pub fn jira_to_markdown(src: &str) -> String {
    let src = sanitize(src);
    let input: Vec<&str> = src.lines().collect();
    let mut out = Lines::default();
    let mut fence: Option<&'static str> = None;
    let mut in_quote = false;
    let mut table_open = false;
    let mut i = 0;
    while i < input.len() {
        let line = input[i];
        i += 1;
        if let Some(close) = fence {
            if let Some(pos) = line.find(close) {
                if pos > 0 {
                    out.push(Block::Code, line[..pos].to_string());
                }
                out.push(Block::Code, "```".to_string());
                fence = None;
            } else {
                out.push(Block::Code, line.to_string());
            }
            continue;
        }
        let trimmed = line.trim();
        for (open, close) in [("{code", "{code}"), ("{noformat", "{noformat}")] {
            if let Some(rest) = trimmed.strip_prefix(open) {
                if rest.starts_with('}') || rest.starts_with(':') {
                    let end = rest.find('}').unwrap_or(rest.len());
                    let lang = if open == "{code" {
                        fence_lang(&rest[..end])
                    } else {
                        String::new()
                    };
                    let after = rest.get(end + 1..).unwrap_or("");
                    table_open = false;
                    out.blank();
                    out.push(Block::Code, format!("```{lang}"));
                    if let Some(pos) = after.find(close) {
                        out.push(Block::Code, after[..pos].to_string());
                        out.push(Block::Code, "```".to_string());
                    } else {
                        if !after.is_empty() {
                            out.push(Block::Code, after.to_string());
                        }
                        fence = Some(close);
                    }
                    break;
                }
            }
        }
        if fence.is_some() {
            continue;
        }
        if trimmed.starts_with("{code") || trimmed.starts_with("{noformat") {
            // Fully handled above (single-line block).
            if trimmed.contains("{code}") || trimmed.contains("{noformat}") {
                continue;
            }
        }
        if trimmed == "{quote}" {
            in_quote = !in_quote;
            out.blank();
            continue;
        }
        if trimmed.is_empty() {
            out.blank();
            table_open = false;
            continue;
        }
        let kind;
        let mut text;
        if let Some(c) = heading_re().captures(trimmed) {
            kind = Block::Heading;
            text = format!("{} {}", "#".repeat(c[1].parse().unwrap_or(1)), inline(&c[2]));
        } else if let Some(c) = bq_re().captures(trimmed) {
            kind = Block::Quote;
            text = format!("> {}", inline(&c[1]));
        } else if rule_re().is_match(trimmed) {
            kind = Block::Rule;
            text = "---".to_string();
        } else if trimmed.starts_with('|') {
            kind = Block::Table;
            let row = cells(trimmed);
            text = table_line(&row);
            if !table_open {
                table_open = true;
                // Markdown tables need a header row: the first row is it.
                out.push(Block::Table, text);
                text = table_line(&vec!["---".to_string(); row.len()]);
            }
        } else if let Some(c) = list_re().captures(line) {
            kind = Block::List;
            let marks = &c[1];
            let indent: usize = marks[..marks.len() - 1]
                .chars()
                .map(|m| if m == '#' { 3 } else { 2 })
                .sum();
            let bullet = if marks.ends_with('#') { "1." } else { "-" };
            text = format!("{}{bullet} {}", " ".repeat(indent), inline(&c[2]));
        } else {
            kind = Block::Para;
            text = inline(trimmed);
            if input.get(i).is_some_and(|n| is_plain(n)) {
                // Jira keeps the line break inside a paragraph.
                text.push_str("  ");
            }
        }
        if in_quote {
            text = format!("> {text}");
            out.push(Block::Quote, text);
        } else {
            out.push(kind, text);
        }
    }
    if fence.is_some() {
        out.push(Block::Code, "```".to_string());
    }
    out.finish()
}

// ---------------------------------------------------------------------------
// Markdown to Jira wiki

#[derive(Default)]
struct Writer {
    bufs: Vec<String>,
    lists: Vec<bool>,
    item_marks: Vec<String>,
    cells: Vec<String>,
    in_cell: bool,
    link_dests: Vec<String>,
    code_lang: Option<String>,
    in_code_block: bool,
}

impl Writer {
    fn buf(&mut self) -> &mut String {
        if self.bufs.is_empty() {
            self.bufs.push(String::new());
        }
        self.bufs.last_mut().expect("non-empty")
    }

    fn push_buf(&mut self) {
        self.bufs.push(String::new());
    }

    fn pop_buf(&mut self) -> String {
        self.bufs.pop().unwrap_or_default()
    }

    fn text(&mut self, t: &str) {
        let cell = self.in_cell;
        let at_line_start = self.buf().is_empty() || self.buf().ends_with('\n');
        let mut escaped = String::with_capacity(t.len());
        let chars: Vec<char> = t.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            let prev = if i == 0 { None } else { Some(chars[i - 1]) };
            let next = chars.get(i + 1).copied();
            let intraword = prev.is_some_and(super::is_word) && next.is_some_and(super::is_word);
            let needs = match c {
                '{' | '}' | '[' | ']' | '*' | '+' | '^' | '~' => true,
                '_' => !intraword,
                '|' => cell,
                '#' => at_line_start && i == 0,
                _ => false,
            };
            if needs {
                escaped.push('\\');
            }
            escaped.push(c);
        }
        self.buf().push_str(&escaped);
    }
}

/// Markdown to Jira wiki markup.
pub fn markdown_to_jira(md: &str) -> String {
    let md = sanitize(md);
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let mut w = Writer::default();
    let mut head_row = false;
    for ev in Parser::new_ext(&md, opts) {
        match ev {
            Event::Start(tag) => start(&mut w, tag, &mut head_row),
            Event::End(tag) => end(&mut w, tag, &mut head_row),
            Event::Text(t) => {
                if w.in_code_block {
                    w.buf().push_str(&t);
                } else {
                    w.text(&t);
                }
            }
            Event::Code(c) => {
                let b = w.buf();
                b.push_str("{{");
                b.push_str(&c);
                b.push_str("}}");
            }
            Event::Html(h) | Event::InlineHtml(h) => w.text(&h),
            Event::SoftBreak => w.buf().push('\n'),
            Event::HardBreak => w.buf().push_str("\\\\\n"),
            Event::Rule => w.buf().push_str("----\n\n"),
            Event::TaskListMarker(done) => {
                w.buf().push_str(if done { "(/) " } else { "( ) " });
            }
            _ => {}
        }
    }
    let mut s = w.bufs.concat();
    while s.ends_with('\n') {
        s.pop();
    }
    s
}

fn start(w: &mut Writer, tag: Tag<'_>, _head_row: &mut bool) {
    match tag {
        Tag::Heading { .. } => w.push_buf(),
        Tag::BlockQuote(_) => w.push_buf(),
        Tag::CodeBlock(kind) => {
            w.in_code_block = true;
            w.code_lang = match kind {
                CodeBlockKind::Fenced(l) => l.split_whitespace().next().map(str::to_lowercase),
                CodeBlockKind::Indented => None,
            };
            w.push_buf();
        }
        Tag::List(first) => {
            if !w.buf().is_empty() && !w.buf().ends_with('\n') {
                w.buf().push('\n');
            }
            w.lists.push(first.is_some());
        }
        Tag::Item => {
            let marks: String = w
                .lists
                .iter()
                .map(|ordered| if *ordered { '#' } else { '*' })
                .collect();
            w.item_marks.push(marks);
            w.push_buf();
        }
        Tag::Emphasis => w.buf().push('_'),
        Tag::Strong => w.buf().push('*'),
        Tag::Strikethrough => w.buf().push('-'),
        Tag::Link { dest_url, .. } => {
            w.link_dests.push(dest_url.to_string());
            w.push_buf();
        }
        Tag::Image { dest_url, .. } => {
            w.link_dests.push(dest_url.to_string());
            w.push_buf();
        }
        Tag::Table(_) => w.push_buf(),
        Tag::TableHead | Tag::TableRow => w.cells.clear(),
        Tag::TableCell => {
            w.in_cell = true;
            w.push_buf();
        }
        _ => {}
    }
}

fn end(w: &mut Writer, tag: TagEnd, head_row: &mut bool) {
    match tag {
        TagEnd::Paragraph => {
            let sep = if w.lists.is_empty() { "\n\n" } else { "\n" };
            w.buf().push_str(sep);
        }
        TagEnd::Heading(level) => {
            let text = w.pop_buf();
            let n = match level {
                HeadingLevel::H1 => 1,
                HeadingLevel::H2 => 2,
                HeadingLevel::H3 => 3,
                HeadingLevel::H4 => 4,
                HeadingLevel::H5 => 5,
                HeadingLevel::H6 => 6,
            };
            w.buf().push_str(&format!("h{n}. {text}\n\n"));
        }
        TagEnd::BlockQuote(_) => {
            let text = w.pop_buf();
            let text = text.trim_end();
            w.buf().push_str(&format!("{{quote}}\n{text}\n{{quote}}\n\n"));
        }
        TagEnd::CodeBlock => {
            let code = w.pop_buf();
            w.in_code_block = false;
            let lang = w.code_lang.take().unwrap_or_default();
            let head = if lang.is_empty() {
                "{code}".to_string()
            } else {
                format!("{{code:{lang}}}")
            };
            let code = code.trim_end_matches('\n');
            w.buf().push_str(&format!("{head}\n{code}\n{{code}}\n\n"));
        }
        TagEnd::List(_) => {
            w.lists.pop();
            if w.lists.is_empty() {
                w.buf().push('\n');
            }
        }
        TagEnd::Item => {
            let body = w.pop_buf();
            let marks = w.item_marks.pop().unwrap_or_default();
            let body = body.trim_end_matches('\n');
            let (first, rest) = body.split_once('\n').unwrap_or((body, ""));
            let b = w.buf();
            b.push_str(&format!("{marks} {first}\n"));
            if !rest.is_empty() {
                b.push_str(rest);
                b.push('\n');
            }
        }
        TagEnd::Emphasis => w.buf().push('_'),
        TagEnd::Strong => w.buf().push('*'),
        TagEnd::Strikethrough => w.buf().push('-'),
        TagEnd::Link => {
            let text = w.pop_buf();
            let dest = w.link_dests.pop().unwrap_or_default();
            let piece = if text == dest || text.is_empty() {
                format!("[{dest}]")
            } else {
                format!("[{text}|{dest}]")
            };
            w.buf().push_str(&piece);
        }
        TagEnd::Image => {
            let _alt = w.pop_buf();
            let dest = w.link_dests.pop().unwrap_or_default();
            w.buf().push_str(&format!("!{dest}!"));
        }
        TagEnd::TableCell => {
            w.in_cell = false;
            let cell = w.pop_buf();
            let cell = cell.trim().replace('\n', " ");
            w.cells.push(if cell.is_empty() { " ".into() } else { cell });
        }
        TagEnd::TableHead => {
            *head_row = true;
            let row = format!("||{}||\n", w.cells.join("||"));
            w.cells.clear();
            w.buf().push_str(&row);
        }
        TagEnd::TableRow => {
            let row = format!("|{}|\n", w.cells.join("|"));
            w.cells.clear();
            w.buf().push_str(&row);
        }
        TagEnd::Table => {
            let t = w.pop_buf();
            w.buf().push_str(&t);
            w.buf().push('\n');
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jira_to_md_table() {
        let cases: &[(&str, &str)] = &[
            ("h1. Title", "# Title"),
            ("h3. Sub", "### Sub"),
            ("a *bold* and _it_ and -gone- and {{mono}}", "a **bold** and _it_ and ~~gone~~ and `mono`"),
            ("see [the docs|https://x.io/d] now", "see [the docs](https://x.io/d) now"),
            ("see [https://x.io] now", "see <https://x.io> now"),
            ("ping [~jdoe] and [~accountid:5b10]", "ping @jdoe and @5b10"),
            ("!shot.png!", "![](shot.png)"),
            ("Wow! really!", "Wow! really!"),
            ("* one\n** two\n* three", "- one\n  - two\n- three"),
            ("# a\n## b\n# c", "1. a\n   1. b\n1. c"),
            ("bq. quoted", "> quoted"),
            ("{quote}\nq1\nq2\n{quote}", "> q1  \n> q2"),
            ("----", "---"),
            ("{color:red}hot{color}", "hot"),
            ("2 * 3 * 4", "2 * 3 * 4"),
            ("a\n\nb", "a\n\nb"),
            ("para\n* item", "para\n\n- item"),
            ("one\ntwo", "one  \ntwo"),
        ];
        for (input, want) in cases {
            assert_eq!(jira_to_markdown(input), *want, "{input:?}");
        }
    }

    #[test]
    fn jira_code_blocks() {
        assert_eq!(
            jira_to_markdown("{code:java}\nint a = 1;\n*not bold*\n{code}\nafter"),
            "```java\nint a = 1;\n*not bold*\n```\n\nafter"
        );
        assert_eq!(
            jira_to_markdown("{code:title=T|language=rust}\nfn x(){}\n{code}"),
            "```rust\nfn x(){}\n```"
        );
        assert_eq!(jira_to_markdown("{noformat}\nraw\n{noformat}"), "```\nraw\n```");
        assert_eq!(jira_to_markdown("{code}\nunterminated"), "```\nunterminated\n```");
        assert_eq!(jira_to_markdown("{code}x{code}"), "```\nx\n```");
    }

    #[test]
    fn jira_tables() {
        assert_eq!(
            jira_to_markdown("||a||b||\n|1|2|\n|3|4|"),
            "| a | b |\n| --- | --- |\n| 1 | 2 |\n| 3 | 4 |"
        );
        // No header row: the first row becomes the header.
        assert_eq!(jira_to_markdown("|1|2|"), "| 1 | 2 |\n| --- | --- |");
    }

    #[test]
    fn jira_strips_escapes() {
        assert_eq!(jira_to_markdown("a\u{1b}[31m b"), "a[31m b");
    }

    #[test]
    fn md_to_jira_table() {
        let cases: &[(&str, &str)] = &[
            ("# Title", "h1. Title"),
            ("## Sub", "h2. Sub"),
            ("a **bold** and *it* and ~~gone~~ and `mono`", "a *bold* and _it_ and -gone- and {{mono}}"),
            ("[the docs](https://x.io/d)", "[the docs|https://x.io/d]"),
            ("<https://x.io>", "[https://x.io]"),
            ("![alt](shot.png)", "!shot.png!"),
            ("- one\n  - two\n- three", "* one\n** two\n* three"),
            ("1. a\n2. b", "# a\n# b"),
            ("> quoted", "{quote}\nquoted\n{quote}"),
            ("---", "----"),
            ("a\n\nb", "a\n\nb"),
            ("100 * 3 {braces} [brackets]", "100 \\* 3 \\{braces\\} \\[brackets\\]"),
            ("snake_case and _it_", "snake_case and _it_"),
        ];
        for (input, want) in cases {
            assert_eq!(markdown_to_jira(input), *want, "{input:?}");
        }
    }

    #[test]
    fn md_to_jira_code_and_table() {
        assert_eq!(
            markdown_to_jira("```rust\nlet a = 1;\n```"),
            "{code:rust}\nlet a = 1;\n{code}"
        );
        assert_eq!(markdown_to_jira("```\nx\n```"), "{code}\nx\n{code}");
        assert_eq!(
            markdown_to_jira("| a | b |\n|---|---|\n| 1 | 2 |"),
            "||a||b||\n|1|2|"
        );
        assert_eq!(markdown_to_jira("- [x] done\n- [ ] todo"), "* (/) done\n* ( ) todo");
    }

    #[test]
    fn round_trip_keeps_meaning() {
        let md = "# T\n\nsome **bold** text\n\n- a\n- b";
        assert_eq!(jira_to_markdown(&markdown_to_jira(md)), md);
    }
}
