//! HTML to Markdown (a small, forgiving subset) and Markdown to HTML.
//!
//! Reading: some trackers and Azure DevOps Server return HTML bodies. Only
//! the tags issue text uses are understood; everything else is dropped with
//! its text kept, `<script>` and `<style>` with their content.
//! Writing: Azure DevOps Server takes HTML, so the user's Markdown is
//! rendered with pulldown-cmark. Raw HTML in the Markdown is escaped, never
//! passed through, and `javascript:` style link targets are neutralised.

use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html};

use super::{decode_entities, sanitize};

enum Token<'a> {
    Text(&'a str),
    Open {
        name: String,
        attrs: &'a str,
    },
    Close(String),
}

fn tokenize(src: &str) -> Vec<Token<'_>> {
    let mut out = Vec::new();
    let mut rest = src;
    while let Some(lt) = rest.find('<') {
        let after = &rest[lt + 1..];
        let starts_tag = after
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '/' || c == '!');
        if !starts_tag {
            out.push(Token::Text(&rest[..lt + 1]));
            rest = after;
            continue;
        }
        if lt > 0 {
            out.push(Token::Text(&rest[..lt]));
        }
        if let Some(body) = after.strip_prefix("!--") {
            // A comment: drop it. Unterminated: drop the rest.
            rest = body.find("-->").map_or("", |e| &body[e + 3..]);
            continue;
        }
        let Some(end) = tag_end(after) else {
            // Unterminated tag: literal text.
            out.push(Token::Text(&rest[lt..]));
            rest = "";
            break;
        };
        let inner = &after[..end];
        rest = &after[end + 1..];
        if inner.starts_with('!') || inner.starts_with('?') {
            continue;
        }
        if let Some(name) = inner.strip_prefix('/') {
            out.push(Token::Close(tag_name(name)));
        } else {
            let name = tag_name(inner);
            let attrs = inner.get(name.len()..).unwrap_or("");
            out.push(Token::Open { name, attrs });
        }
    }
    if !rest.is_empty() {
        out.push(Token::Text(rest));
    }
    out
}

/// Index of the `>` closing a tag, skipping quoted attribute values.
fn tag_end(s: &str) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (i, c) in s.char_indices() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '>') => return Some(i),
            _ => {}
        }
    }
    None
}

fn tag_name(s: &str) -> String {
    s.chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn attr(attrs: &str, name: &str) -> Option<String> {
    let lower = attrs.to_ascii_lowercase();
    let mut from = 0;
    while let Some(p) = lower[from..].find(name) {
        let at = from + p;
        from = at + name.len();
        let before_ok = at == 0 || lower.as_bytes()[at - 1].is_ascii_whitespace();
        let rest = attrs[from..].trim_start();
        let Some(rest) = rest.strip_prefix('=').filter(|_| before_ok) else {
            continue;
        };
        let rest = rest.trim_start();
        let value = match rest.chars().next() {
            Some(q @ ('"' | '\'')) => rest[1..].split(q).next().unwrap_or(""),
            _ => rest.split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or(""),
        };
        return Some(decode_entities(value));
    }
    None
}

fn safe_url(url: &str) -> bool {
    let u = url.trim().to_ascii_lowercase();
    !(u.starts_with("javascript:") || u.starts_with("vbscript:") || u.starts_with("data:"))
}

#[derive(Default)]
struct State {
    out: String,
    lists: Vec<Option<u32>>,
    pre: bool,
    code: bool,
    skip: Option<String>,
    marks: Vec<(String, usize, String)>,
    row: Vec<String>,
    rows_in_table: Vec<usize>,
    fence_pending: bool,
}

impl State {
    fn newline(&mut self) {
        if !self.out.is_empty() && !self.out.ends_with('\n') {
            self.out.push('\n');
        }
    }

    fn blank(&mut self) {
        self.trim_trailing_blanks();
        if self.out.is_empty() {
            return;
        }
        while !self.out.ends_with("\n\n") {
            self.out.push('\n');
        }
    }

    fn trim_trailing_blanks(&mut self) {
        while self.out.ends_with(' ') && !self.out.ends_with("  ") {
            self.out.pop();
        }
    }

    fn text(&mut self, raw: &str) {
        let decoded = decode_entities(raw);
        if self.fence_pending {
            self.out.push('\n');
            self.fence_pending = false;
        }
        if self.pre {
            self.out.push_str(&decoded);
            return;
        }
        let mut last_space = self.out.is_empty() || self.out.ends_with([' ', '\n']);
        for c in decoded.chars() {
            if c.is_whitespace() {
                if !last_space {
                    self.out.push(' ');
                    last_space = true;
                }
            } else {
                if c == '<' && !self.code {
                    self.out.push('\\');
                }
                self.out.push(c);
                last_space = false;
            }
        }
    }

    fn mark(&mut self, name: &str, extra: String) {
        self.marks.push((name.to_string(), self.out.len(), extra));
    }

    fn take_mark(&mut self, name: &str) -> Option<(String, String)> {
        let pos = self.marks.iter().rposition(|(n, _, _)| n == name)?;
        let (_, at, extra) = self.marks.remove(pos);
        let at = at.min(self.out.len());
        Some((self.out.split_off(at), extra))
    }
}

fn open(st: &mut State, name: &str, attrs: &str) {
    match name {
        "script" | "style" => st.skip = Some(name.to_string()),
        "p" | "div" | "section" | "article" | "details" | "figure" | "ul_" => st.blank(),
        "br" => {
            st.out.push_str("  \n");
        }
        "hr" => {
            st.blank();
            st.out.push_str("---\n\n");
        }
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            st.blank();
            let level = (name.as_bytes()[1] - b'0') as usize;
            st.out.push_str(&"#".repeat(level));
            st.out.push(' ');
        }
        "b" | "strong" => st.out.push_str("**"),
        "i" | "em" => st.out.push('*'),
        "del" | "s" | "strike" => st.out.push_str("~~"),
        "summary" => {
            st.blank();
            st.out.push_str("**");
        }
        "code" if !st.pre => {
            st.code = true;
            st.out.push('`');
        }
        "code" => {
            if st.fence_pending {
                if let Some(class) = attr(attrs, "class") {
                    if let Some(lang) = class
                        .split_whitespace()
                        .find_map(|c| c.strip_prefix("language-").or_else(|| c.strip_prefix("lang-")))
                    {
                        st.out.push_str(lang);
                    }
                }
            }
        }
        "pre" => {
            st.blank();
            st.out.push_str("```");
            st.pre = true;
            st.fence_pending = true;
        }
        "a" => st.mark("a", attr(attrs, "href").unwrap_or_default()),
        "img" => {
            let src = attr(attrs, "src").unwrap_or_default();
            if safe_url(&src) && !src.is_empty() {
                let alt = attr(attrs, "alt").unwrap_or_default();
                st.out.push_str(&format!("![{alt}]({src})"));
            }
        }
        "ul" | "ol" => {
            if st.lists.is_empty() {
                st.blank();
            } else {
                st.newline();
            }
            st.lists.push(if name == "ol" { Some(0) } else { None });
        }
        "li" => {
            st.newline();
            let depth = st.lists.len().saturating_sub(1);
            let bullet = match st.lists.last_mut() {
                Some(Some(n)) => {
                    *n += 1;
                    format!("{n}.")
                }
                _ => "-".to_string(),
            };
            st.out.push_str(&format!("{}{bullet} ", "  ".repeat(depth)));
        }
        "blockquote" => {
            st.blank();
            st.mark("blockquote", String::new());
        }
        "table" => {
            st.blank();
            st.rows_in_table.push(0);
        }
        "tr" => st.row.clear(),
        "td" | "th" => st.mark("cell", String::new()),
        _ => {}
    }
}

fn close(st: &mut State, name: &str) {
    match name {
        "p" | "div" | "section" | "article" | "details" | "figure" | "summary" => {
            if name == "summary" {
                st.out.push_str("**");
            }
            st.blank();
        }
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => st.blank(),
        "b" | "strong" => st.out.push_str("**"),
        "i" | "em" => st.out.push('*'),
        "del" | "s" | "strike" => st.out.push_str("~~"),
        "code" if st.code => {
            st.code = false;
            st.out.push('`');
        }
        "pre" => {
            if st.fence_pending {
                st.out.push('\n');
                st.fence_pending = false;
            }
            st.pre = false;
            st.newline();
            st.out.push_str("```\n\n");
        }
        "a" => {
            if let Some((text, href)) = st.take_mark("a") {
                if !href.is_empty() && safe_url(&href) && !text.trim().is_empty() {
                    st.out.push_str(&format!("[{}]({href})", text.trim()));
                } else {
                    st.out.push_str(&text);
                }
            }
        }
        "ul" | "ol" => {
            st.lists.pop();
            if st.lists.is_empty() {
                st.blank();
            }
        }
        "blockquote" => {
            if let Some((body, _)) = st.take_mark("blockquote") {
                let quoted: Vec<String> = body
                    .trim()
                    .lines()
                    .map(|l| if l.is_empty() { ">".to_string() } else { format!("> {l}") })
                    .collect();
                st.out.push_str(&quoted.join("\n"));
                st.blank();
            }
        }
        "td" | "th" => {
            if let Some((cell, _)) = st.take_mark("cell") {
                st.row.push(cell.split_whitespace().collect::<Vec<_>>().join(" "));
            }
        }
        "tr" => {
            if !st.row.is_empty() {
                let cells = std::mem::take(&mut st.row);
                st.newline();
                st.out.push_str(&format!("| {} |\n", cells.join(" | ")));
                if let Some(n) = st.rows_in_table.last_mut() {
                    if *n == 0 {
                        st.out.push_str(&format!("| {} |\n", vec!["---"; cells.len()].join(" | ")));
                    }
                    *n += 1;
                }
            }
        }
        "table" => {
            st.rows_in_table.pop();
            st.blank();
        }
        _ => {}
    }
}

/// HTML to Markdown. Unknown tags are dropped, their text kept.
pub fn html_to_markdown(src: &str) -> String {
    let src = sanitize(src);
    let mut st = State::default();
    for token in tokenize(&src) {
        if let Some(skipping) = st.skip.clone() {
            if matches!(&token, Token::Close(n) if *n == skipping) {
                st.skip = None;
            }
            continue;
        }
        match token {
            Token::Text(t) => st.text(t),
            Token::Open { name, attrs } => open(&mut st, &name, attrs),
            Token::Close(name) => close(&mut st, &name),
        }
    }
    let mut s = st.out;
    while s.ends_with(['\n', ' ']) {
        s.pop();
    }
    s.trim_start_matches('\n').to_string()
}

/// Markdown to HTML for hosts that take HTML (Azure DevOps Server).
pub fn markdown_to_html(md: &str) -> String {
    let md = sanitize(md);
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let events = Parser::new_ext(&md, opts).map(|ev| match ev {
        Event::Html(h) | Event::InlineHtml(h) => Event::Text(h),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) if !safe_url(&dest_url) => Event::Start(Tag::Link {
            link_type,
            dest_url: CowStr::Borrowed("#"),
            title,
            id,
        }),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) if !safe_url(&dest_url) => Event::Start(Tag::Image {
            link_type,
            dest_url: CowStr::Borrowed(""),
            title,
            id,
        }),
        other => other,
    });
    let mut out = String::new();
    html::push_html(&mut out, events);
    out.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_to_md_table() {
        let cases: &[(&str, &str)] = &[
            ("<p>Hello <b>world</b></p>", "Hello **world**"),
            ("<p>a</p><p>b</p>", "a\n\nb"),
            ("a<br>b", "a  \nb"),
            ("<h2>Title</h2>", "## Title"),
            ("<i>it</i> <em>em</em> <s>gone</s> <code>x</code>", "*it* *em* ~~gone~~ `x`"),
            ("<a href=\"https://x.io/a?b=1&amp;c=2\">link</a>", "[link](https://x.io/a?b=1&c=2)"),
            ("<a href=\"javascript:alert(1)\">bad</a>", "bad"),
            ("<img src=\"a.png\" alt=\"pic\">", "![pic](a.png)"),
            ("<ul><li>one</li><li>two<ul><li>deep</li></ul></li></ul>", "- one\n- two\n  - deep"),
            ("<ol><li>a</li><li>b</li></ol>", "1. a\n2. b"),
            ("<blockquote><p>q</p></blockquote>", "> q"),
            ("<hr>", "---"),
            ("x &lt; y &amp;&amp; z", "x \\< y && z"),
            ("<!-- hidden -->shown", "shown"),
            ("<script>alert(1)</script>text<style>p{}</style>", "text"),
            ("<span class=\"x\">kept</span> <unknown>too</unknown>", "kept too"),
            ("1 < 2 and 3 > 2", "1 \\< 2 and 3 > 2"),
            ("<details><summary>More</summary>body</details>", "**More**\n\nbody"),
        ];
        for (input, want) in cases {
            assert_eq!(html_to_markdown(input), *want, "{input:?}");
        }
    }

    #[test]
    fn html_pre_and_table() {
        assert_eq!(
            html_to_markdown("<pre><code class=\"language-rust\">let a = 1;\nlet b = 2;</code></pre>"),
            "```rust\nlet a = 1;\nlet b = 2;\n```"
        );
        assert_eq!(html_to_markdown("<pre>x &lt; y</pre>"), "```\nx < y\n```");
        assert_eq!(
            html_to_markdown("<table><tr><th>a</th><th>b</th></tr><tr><td>1</td><td>2</td></tr></table>"),
            "| a | b |\n| --- | --- |\n| 1 | 2 |"
        );
    }

    #[test]
    fn html_strips_escapes() {
        assert_eq!(html_to_markdown("<p>a\u{1b}[31mb</p>"), "a[31mb");
    }

    #[test]
    fn md_to_html_table() {
        let cases: &[(&str, &str)] = &[
            ("**b** and *i*", "<p><strong>b</strong> and <em>i</em></p>"),
            ("# T", "<h1>T</h1>"),
            ("[a](https://x.io)", "<p><a href=\"https://x.io\">a</a></p>"),
            ("- a\n- b", "<ul>\n<li>a</li>\n<li>b</li>\n</ul>"),
            ("<script>alert(1)</script>", "&lt;script&gt;alert(1)&lt;/script&gt;"),
            ("[x](javascript:alert(1))", "<p><a href=\"#\">x</a></p>"),
            ("~~gone~~", "<p><del>gone</del></p>"),
        ];
        for (input, want) in cases {
            assert_eq!(markdown_to_html(input), *want, "{input:?}");
        }
    }

    #[test]
    fn md_to_html_table_and_code() {
        let html = markdown_to_html("| a | b |\n|---|---|\n| 1 | 2 |");
        assert!(html.contains("<table>") && html.contains("<td>1</td>"), "{html}");
        let html = markdown_to_html("```rust\nlet a = 1;\n```");
        assert!(html.contains("<pre><code class=\"language-rust\">"), "{html}");
    }
}
