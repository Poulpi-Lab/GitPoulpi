//! Markdown to Textile, for a Redmine server configured with Textile (the
//! reading direction is [`crate::markup::textile_to_markdown`]).
//!
//! Forgiving rather than complete, like the other converters: plain text is
//! passed through unescaped (Textile marks only act at word boundaries, and
//! a stray `*` in a sentence stays readable), what Textile has no word for
//! (task boxes, footnotes) becomes readable text.

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::markup::sanitize;

#[derive(Default)]
struct Writer {
    bufs: Vec<String>,
    lists: Vec<bool>,
    item_marks: Vec<String>,
    cells: Vec<String>,
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
}

/// Markdown to Textile.
pub(crate) fn markdown_to_textile(md: &str) -> String {
    let md = sanitize(md);
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let mut w = Writer::default();
    for ev in Parser::new_ext(&md, opts) {
        match ev {
            Event::Start(tag) => start(&mut w, tag),
            Event::End(tag) => end(&mut w, tag),
            Event::Text(t) | Event::Html(t) | Event::InlineHtml(t) => w.buf().push_str(&t),
            Event::Code(c) => {
                let b = w.buf();
                b.push('@');
                b.push_str(&c);
                b.push('@');
            }
            Event::SoftBreak | Event::HardBreak => w.buf().push('\n'),
            Event::Rule => w.buf().push_str("---\n\n"),
            Event::TaskListMarker(done) => w.buf().push_str(if done { "[x] " } else { "[ ] " }),
            _ => {}
        }
    }
    let mut s = w.bufs.concat();
    while s.ends_with('\n') {
        s.pop();
    }
    s
}

fn start(w: &mut Writer, tag: Tag<'_>) {
    match tag {
        Tag::Heading { .. } | Tag::BlockQuote(_) => w.push_buf(),
        Tag::CodeBlock(kind) => {
            w.in_code_block = true;
            w.code_lang = match kind {
                CodeBlockKind::Fenced(l) => l
                    .split_whitespace()
                    .next()
                    .map(str::to_lowercase)
                    .filter(|l| !l.is_empty()),
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
        Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. } => {
            w.link_dests.push(dest_url.to_string());
            w.push_buf();
        }
        Tag::Table(_) => w.push_buf(),
        Tag::TableHead | Tag::TableRow => w.cells.clear(),
        Tag::TableCell => w.push_buf(),
        _ => {}
    }
}

fn end(w: &mut Writer, tag: TagEnd) {
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
            // `bq..` runs over blank lines until the next block signature.
            let sig = if text.contains("\n\n") { "bq.." } else { "bq." };
            w.buf().push_str(&format!("{sig} {text}\n\n"));
        }
        TagEnd::CodeBlock => {
            let code = w.pop_buf();
            w.in_code_block = false;
            let code = code.trim_end_matches('\n');
            let open = match w.code_lang.take() {
                Some(lang) => format!("<pre><code class=\"{lang}\">"),
                None => "<pre><code>".to_string(),
            };
            w.buf()
                .push_str(&format!("{open}\n{code}\n</code></pre>\n\n"));
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
                dest
            } else {
                format!("\"{text}\":{dest}")
            };
            w.buf().push_str(&piece);
        }
        TagEnd::Image => {
            let _alt = w.pop_buf();
            let dest = w.link_dests.pop().unwrap_or_default();
            w.buf().push_str(&format!("!{dest}!"));
        }
        TagEnd::TableCell => {
            let cell = w.pop_buf();
            let cell = cell.trim().replace('\n', " ");
            w.cells.push(cell);
        }
        TagEnd::TableHead => {
            let row = format!("|_. {} |\n", w.cells.join(" |_. "));
            w.cells.clear();
            w.buf().push_str(&row);
        }
        TagEnd::TableRow => {
            let row = format!("| {} |\n", w.cells.join(" | "));
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
    fn markdown_to_textile_table() {
        let cases: &[(&str, &str)] = &[
            ("# Title", "h1. Title"),
            ("### Sub", "h3. Sub"),
            (
                "a **bold** and *it* and ~~gone~~ and `mono`",
                "a *bold* and _it_ and -gone- and @mono@",
            ),
            (
                "see [the docs](https://x.io/d) now",
                "see \"the docs\":https://x.io/d now",
            ),
            ("![shot](shot.png)", "!shot.png!"),
            ("- one\n  - two\n- three", "* one\n** two\n* three"),
            ("1. a\n   1. b\n2. c", "# a\n## b\n# c"),
            ("> quoted", "bq. quoted"),
            ("---", "---"),
            ("a\n\nb", "a\n\nb"),
            ("- [x] done\n- [ ] todo", "* [x] done\n* [ ] todo"),
            (
                "```rust\nlet a = 1;\n```",
                "<pre><code class=\"rust\">\nlet a = 1;\n</code></pre>",
            ),
            ("```\nplain\n```", "<pre><code>\nplain\n</code></pre>"),
            (
                "| a | b |\n|---|---|\n| 1 | 2 |",
                "|_. a |_. b |\n| 1 | 2 |",
            ),
        ];
        for (input, want) in cases {
            assert_eq!(markdown_to_textile(input), *want, "{input:?}");
        }
    }

    #[test]
    fn what_goes_out_reads_back_to_the_same_markdown() {
        for md in [
            "# Title\n\nsome **bold** text",
            "- one\n- two",
            "1. a\n1. b",
            "> quoted",
        ] {
            let back = crate::markup::textile_to_markdown(&markdown_to_textile(md));
            assert_eq!(back.trim(), md, "{md:?}");
        }
    }

    #[test]
    fn control_characters_never_go_out() {
        assert_eq!(markdown_to_textile("hi\u{1b}[31m there"), "hi[31m there");
    }
}
