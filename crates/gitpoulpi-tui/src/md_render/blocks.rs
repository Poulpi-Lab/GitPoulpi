//! Markdown blocks to lines: the `pulldown-cmark` event walk.
//!
//! Inline events accumulate styled characters ([`Atom`]s); a block's end
//! wraps them under the prefix its containers ask for (quote bars, list
//! bullets and indentation). Code blocks and tables lay themselves out.

use std::collections::HashSet;

use pulldown_cmark::{
    Alignment, BlockQuoteKind, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};

use super::layout::{atoms, wrap, Atom, LineBuf, Prefix};
use super::{
    code, is_banned, links, Context, Doc, LineKind, LinkId, LinkTarget, Look, MdLine, Span,
    CODE_BLOCK_LINES, CODE_DOC_LINES,
};
use crate::cell_text::cells;

const BULLETS: [&str; 2] = ["\u{2022} ", "\u{25E6} "];
const BAR: &str = "\u{2502}";
const RULE: char = '\u{2500}';

enum Ctx {
    Quote,
    List {
        next: Option<u64>,
        tight: bool,
        items: usize,
    },
    Item {
        marker: Option<Vec<(String, Look)>>,
        width: usize,
        tight: bool,
    },
}

struct CodeBuf {
    lang: String,
    text: String,
}

struct TableBuf {
    aligns: Vec<Alignment>,
    head: Vec<Vec<Atom>>,
    rows: Vec<Vec<Vec<Atom>>>,
    cur: Vec<Vec<Atom>>,
}

struct Builder<'a> {
    doc: Doc,
    width: usize,
    ctx: &'a Context,
    stack: Vec<Ctx>,
    atoms: Vec<Atom>,
    base: Look,
    bold: u32,
    italic: u32,
    strike: u32,
    links: Vec<Option<LinkId>>,
    gap: bool,
    code: Option<CodeBuf>,
    code_lines: usize,
    image: Option<(String, String)>,
    table: Option<TableBuf>,
    heading: Option<HeadingLevel>,
    in_comment: bool,
    in_tag: bool,
    html_links: u32,
}

pub(super) fn build(text: &str, width: usize, ctx: &Context) -> Doc {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_GFM;
    let events: Vec<Event> = Parser::new_ext(text, opts).collect();
    let loose = loose_lists(&events);
    let mut b = Builder {
        doc: Doc::default(),
        width,
        ctx,
        stack: Vec::new(),
        atoms: Vec::new(),
        base: Look::Plain,
        bold: 0,
        italic: 0,
        strike: 0,
        links: Vec::new(),
        gap: false,
        code: None,
        code_lines: 0,
        image: None,
        table: None,
        heading: None,
        in_comment: false,
        in_tag: false,
        html_links: 0,
    };
    for (i, ev) in events.into_iter().enumerate() {
        b.event(i, ev, &loose);
    }
    b.flush();
    b.doc
}

/// The indexes of the `Start(List)` events whose items hold paragraphs
/// (a "loose" list, with a blank line between items).
fn loose_lists(events: &[Event]) -> HashSet<usize> {
    #[derive(PartialEq)]
    enum K {
        List(usize),
        Item,
        Other,
    }
    let mut stack: Vec<K> = Vec::new();
    let mut loose = HashSet::new();
    for (i, ev) in events.iter().enumerate() {
        match ev {
            Event::Start(tag) => {
                if matches!(tag, Tag::Paragraph) && stack.last() == Some(&K::Item) {
                    if let Some(K::List(at)) = stack.iter().rev().find(|k| matches!(k, K::List(_)))
                    {
                        loose.insert(*at);
                    }
                }
                stack.push(match tag {
                    Tag::List(_) => K::List(i),
                    Tag::Item => K::Item,
                    _ => K::Other,
                });
            }
            Event::End(_) => {
                stack.pop();
            }
            _ => {}
        }
    }
    loose
}

fn alert_label(kind: BlockQuoteKind) -> &'static str {
    use gitpoulpi_i18n::msg::md_render as m;
    match kind {
        BlockQuoteKind::Note => m::ALERT_NOTE,
        BlockQuoteKind::Tip => m::ALERT_TIP,
        BlockQuoteKind::Important => m::ALERT_IMPORTANT,
        BlockQuoteKind::Warning => m::ALERT_WARNING,
        BlockQuoteKind::Caution => m::ALERT_CAUTION,
    }
    .get()
}

fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    s.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

fn attr<'s>(tag: &'s str, name: &str) -> Option<&'s str> {
    let lower = tag.to_ascii_lowercase();
    for quote in ['"', '\''] {
        let needle = format!("{name}={quote}");
        if let Some(at) = lower.find(&needle) {
            let from = at + needle.len();
            let len = tag[from..].find(quote)?;
            return Some(&tag[from..from + len]);
        }
    }
    None
}

impl Builder<'_> {
    fn event(&mut self, i: usize, ev: Event, loose: &HashSet<usize>) {
        match ev {
            Event::Start(tag) => self.start(i, tag, loose),
            Event::End(end) => self.end(end),
            Event::Text(s) => self.text(&s),
            Event::Code(s) => {
                let link = self.links.last().copied().flatten();
                let clean: String = s.chars().filter(|c| !is_banned(*c)).collect();
                self.atoms.extend(atoms(&clean, Look::Code, link));
            }
            Event::SoftBreak => self.push_space(),
            Event::HardBreak => self.atoms.push(Atom {
                ch: '\n',
                look: Look::Plain,
                link: None,
            }),
            Event::Html(s) | Event::InlineHtml(s) => self.html(&s),
            Event::FootnoteReference(l) => {
                self.atoms
                    .extend(atoms(&format!("[^{l}]"), Look::Dim, None));
            }
            Event::TaskListMarker(done) => {
                if let Some(Ctx::Item {
                    marker: Some(m),
                    width,
                    ..
                }) = self.stack.last_mut()
                {
                    let mark = if done { "[x] " } else { "[ ] " };
                    m.push((mark.to_string(), Look::Dim));
                    *width += 4;
                }
            }
            Event::Rule => {
                self.flush();
                self.start_block();
                let n = self.avail();
                let rule: String = std::iter::repeat_n(RULE, n).collect();
                self.emit(&atoms(&rule, Look::Dim, None));
                self.gap = true;
            }
            _ => {}
        }
    }

    fn start(&mut self, i: usize, tag: Tag, loose: &HashSet<usize>) {
        match tag {
            Tag::Paragraph | Tag::HtmlBlock | Tag::MetadataBlock(_) => {}
            Tag::Heading { level, .. } => {
                self.flush();
                self.base = Look::Heading;
                self.heading = Some(level);
            }
            Tag::BlockQuote(kind) => {
                self.flush();
                self.start_block();
                self.stack.push(Ctx::Quote);
                self.gap = false;
                if let Some(kind) = kind {
                    let label = atoms(alert_label(kind), Look::Heading, None);
                    self.emit(&label);
                }
            }
            Tag::CodeBlock(kind) => {
                self.flush();
                let lang = match kind {
                    CodeBlockKind::Fenced(info) => code::language(&info).to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                self.code = Some(CodeBuf {
                    lang,
                    text: String::new(),
                });
            }
            Tag::List(first) => {
                self.flush();
                let tight = !loose.contains(&i);
                if matches!(self.stack.last(), Some(Ctx::Item { tight: true, .. })) {
                    self.gap = false;
                }
                self.stack.push(Ctx::List {
                    next: first,
                    tight,
                    items: 0,
                });
            }
            Tag::Item => {
                self.flush();
                let depth = self
                    .stack
                    .iter()
                    .filter(|c| matches!(c, Ctx::List { .. }))
                    .count();
                let (marker, tight, first) = match self.stack.iter_mut().rev().find_map(|c| {
                    if let Ctx::List { next, tight, items } = c {
                        Some((next, *tight, items))
                    } else {
                        None
                    }
                }) {
                    Some((next, tight, items)) => {
                        let first = *items == 0;
                        *items += 1;
                        let marker = match next {
                            Some(n) => {
                                let m = format!("{n}. ");
                                *n += 1;
                                m
                            }
                            None => BULLETS[depth.saturating_sub(1) % 2].to_string(),
                        };
                        (marker, tight, first)
                    }
                    None => (BULLETS[0].to_string(), true, true),
                };
                if !tight && !first {
                    self.blank();
                    self.gap = false;
                }
                let width = cells(&marker);
                self.stack.push(Ctx::Item {
                    marker: Some(vec![(marker, Look::Marker)]),
                    width,
                    tight,
                });
            }
            Tag::Emphasis => self.italic += 1,
            Tag::Strong => self.bold += 1,
            Tag::Strikethrough => self.strike += 1,
            Tag::Link { dest_url, .. } => {
                let id = links::register(&mut self.doc, &dest_url);
                self.links.push(id);
            }
            Tag::Image { dest_url, .. } => {
                self.image = Some((dest_url.to_string(), String::new()));
            }
            Tag::Table(aligns) => {
                self.flush();
                self.table = Some(TableBuf {
                    aligns,
                    head: Vec::new(),
                    rows: Vec::new(),
                    cur: Vec::new(),
                });
            }
            Tag::TableHead | Tag::TableRow => {
                if let Some(t) = self.table.as_mut() {
                    t.cur.clear();
                }
            }
            Tag::TableCell => self.atoms.clear(),
            Tag::FootnoteDefinition(label) => {
                self.flush();
                self.atoms
                    .extend(atoms(&format!("[^{label}]: "), Look::Dim, None));
            }
            _ => {}
        }
    }

    fn end(&mut self, end: TagEnd) {
        match end {
            TagEnd::Paragraph | TagEnd::HtmlBlock | TagEnd::FootnoteDefinition => self.flush(),
            TagEnd::Heading(level) => {
                self.flush();
                if matches!(level, HeadingLevel::H1 | HeadingLevel::H2) && self.heading.is_some() {
                    self.heading_rule();
                }
                self.heading = None;
                self.base = Look::Plain;
            }
            TagEnd::BlockQuote(_) => {
                self.flush();
                self.stack.pop();
                self.gap = true;
            }
            TagEnd::CodeBlock => self.finish_code(),
            TagEnd::List(_) => {
                self.flush();
                self.stack.pop();
                self.gap = true;
            }
            TagEnd::Item => {
                self.flush();
                self.stack.pop();
                self.gap = false;
            }
            TagEnd::Emphasis => self.italic = self.italic.saturating_sub(1),
            TagEnd::Strong => self.bold = self.bold.saturating_sub(1),
            TagEnd::Strikethrough => self.strike = self.strike.saturating_sub(1),
            TagEnd::Link => {
                self.links.pop();
            }
            TagEnd::Image => self.finish_image(),
            TagEnd::TableCell => {
                let mut cell = std::mem::take(&mut self.atoms);
                for a in &mut cell {
                    if a.ch == '\n' {
                        a.ch = ' ';
                    }
                }
                if let Some(t) = self.table.as_mut() {
                    t.cur.push(trim(cell));
                }
            }
            TagEnd::TableHead => {
                if let Some(t) = self.table.as_mut() {
                    t.head = std::mem::take(&mut t.cur);
                }
            }
            TagEnd::TableRow => {
                if let Some(t) = self.table.as_mut() {
                    let row = std::mem::take(&mut t.cur);
                    t.rows.push(row);
                }
            }
            TagEnd::Table => self.finish_table(),
            _ => {}
        }
    }

    // -- inline -------------------------------------------------------------------

    fn look(&self) -> Look {
        if self.links.last().is_some_and(Option::is_some) {
            Look::Link
        } else if self.strike > 0 {
            Look::Strike
        } else if self.base == Look::Heading {
            Look::Heading
        } else {
            match (self.bold > 0, self.italic > 0) {
                (true, true) => Look::BoldItalic,
                (true, false) => Look::Bold,
                (false, true) => Look::Italic,
                (false, false) => self.base,
            }
        }
    }

    fn push_space(&mut self) {
        let look = self.look();
        let link = self.links.last().copied().flatten();
        self.atoms.push(Atom {
            ch: ' ',
            look,
            link,
        });
    }

    fn text(&mut self, s: &str) {
        if let Some(code) = self.code.as_mut() {
            code.text.push_str(s);
            return;
        }
        if let Some((_, alt)) = self.image.as_mut() {
            alt.push_str(s);
            return;
        }
        let clean: String = s.chars().filter(|c| !is_banned(*c) || *c == '\n').collect();
        let look = self.look();
        match self.links.last() {
            Some(link) => {
                let link = *link;
                self.atoms.extend(atoms(&clean, look, link));
            }
            None => {
                let a = links::linked_atoms(&clean, look, &mut self.doc, self.ctx);
                self.atoms.extend(a);
            }
        }
    }

    fn finish_image(&mut self) {
        let Some((dest, alt)) = self.image.take() else {
            return;
        };
        let alt = alt.trim();
        let label = if alt.is_empty() {
            "[image]".to_string()
        } else {
            format!("[image: {alt}]")
        };
        let id = links::register(&mut self.doc, &dest);
        let look = if id.is_some() { Look::Link } else { Look::Dim };
        self.atoms.extend(atoms(&label, look, id));
    }

    // -- inline HTML --------------------------------------------------------------

    fn html(&mut self, s: &str) {
        let mut rest = s;
        loop {
            if self.in_comment {
                match rest.find("-->") {
                    Some(i) => {
                        rest = &rest[i + 3..];
                        self.in_comment = false;
                        continue;
                    }
                    None => return,
                }
            }
            if self.in_tag {
                match rest.find('>') {
                    Some(i) => {
                        rest = &rest[i + 1..];
                        self.in_tag = false;
                        continue;
                    }
                    None => return,
                }
            }
            let Some(i) = rest.find('<') else {
                self.html_text(rest);
                return;
            };
            self.html_text(&rest[..i]);
            let tail = &rest[i..];
            if let Some(after) = tail.strip_prefix("<!--") {
                self.in_comment = true;
                rest = after;
                continue;
            }
            let is_tag = tail[1..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || matches!(c, '/' | '!' | '?'));
            if !is_tag {
                self.html_text("<");
                rest = &tail[1..];
                continue;
            }
            match tail.find('>') {
                Some(j) => {
                    self.html_tag(&tail[1..j]);
                    rest = &tail[j + 1..];
                }
                None => {
                    self.in_tag = true;
                    return;
                }
            }
        }
    }

    fn html_text(&mut self, t: &str) {
        if t.is_empty() {
            return;
        }
        let t = decode_entities(&t.replace('\n', " "));
        self.text(&t);
    }

    fn html_break(&mut self) {
        if self.atoms.last().is_some_and(|a| a.ch != '\n') {
            self.atoms.push(Atom {
                ch: '\n',
                look: Look::Plain,
                link: None,
            });
        }
    }

    fn html_tag(&mut self, inner: &str) {
        let closing = inner.starts_with('/');
        let name: String = inner
            .trim_start_matches('/')
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        match name.as_str() {
            "br" => self.html_break(),
            "summary" => {
                if closing {
                    self.bold = self.bold.saturating_sub(1);
                    self.html_break();
                } else {
                    self.html_break();
                    self.bold += 1;
                }
            }
            "p" | "div" | "li" | "tr" | "ul" | "ol" | "details" | "blockquote" | "pre" | "h1"
            | "h2" | "h3" | "h4" | "h5" | "h6" => self.html_break(),
            "a" => {
                if closing {
                    if self.html_links > 0 {
                        self.html_links -= 1;
                        self.links.pop();
                    }
                } else {
                    let id = attr(inner, "href").and_then(|h| links::register(&mut self.doc, h));
                    self.links.push(id);
                    self.html_links += 1;
                }
            }
            "img" if !closing => {
                let alt = attr(inner, "alt").unwrap_or("").trim().to_string();
                let label = if alt.is_empty() {
                    "[image]".to_string()
                } else {
                    format!("[image: {}]", decode_entities(&alt))
                };
                let id = attr(inner, "src").and_then(|s| links::register(&mut self.doc, s));
                let look = if id.is_some() { Look::Link } else { Look::Dim };
                let clean: String = label.chars().filter(|c| !is_banned(*c)).collect();
                self.atoms.extend(atoms(&clean, look, id));
            }
            _ => {}
        }
    }

    // -- blocks -------------------------------------------------------------------

    fn avail(&self) -> usize {
        self.width.saturating_sub(self.prefix(false).width).max(2)
    }

    /// The cells every line of the current containers starts with. `first`
    /// takes (and so uses up) the pending bullets of the open items.
    fn prefix(&self, first: bool) -> Prefix {
        // `first` consumption needs `&mut`; see `take_prefix`.
        let mut p = Prefix::default();
        for c in &self.stack {
            match c {
                Ctx::Quote => p.push(&format!("{BAR} "), Look::Marker),
                Ctx::List { .. } => {}
                Ctx::Item { marker, width, .. } => match (first, marker) {
                    (true, Some(m)) => {
                        for (t, l) in m {
                            p.push(t, *l);
                        }
                    }
                    _ => p.push(&" ".repeat(*width), Look::Plain),
                },
            }
        }
        p
    }

    /// The prefix of a first line; the bullets it shows are used up.
    fn take_prefix(&mut self) -> Prefix {
        let p = self.prefix(true);
        for c in &mut self.stack {
            if let Ctx::Item { marker, .. } = c {
                *marker = None;
            }
        }
        p
    }

    fn blank(&mut self) {
        let mut p = Prefix::default();
        for c in &self.stack {
            if matches!(c, Ctx::Quote) {
                p.push(BAR, Look::Marker);
            }
        }
        self.doc.lines.push(LineBuf::new(&p).finish());
    }

    fn start_block(&mut self) {
        if self.gap {
            self.blank();
            self.gap = false;
        }
    }

    fn emit(&mut self, a: &[Atom]) {
        let rest = self.prefix(false);
        let first = self.take_prefix();
        let lines = wrap(a, self.width, &first, &rest);
        self.doc.lines.extend(lines);
    }

    fn flush(&mut self) {
        if self.atoms.is_empty() {
            return;
        }
        let a = trim(std::mem::take(&mut self.atoms));
        if a.is_empty() {
            return;
        }
        self.start_block();
        self.emit(&a);
        self.gap = true;
    }

    fn heading_rule(&mut self) {
        let prefix = self.prefix(false).width;
        let widest = self
            .doc
            .lines
            .iter()
            .rev()
            .take_while(|l| !l.spans.is_empty())
            .map(MdLine::width)
            .max()
            .unwrap_or(0);
        let n = widest.saturating_sub(prefix).clamp(1, self.avail());
        let rule: String = std::iter::repeat_n(RULE, n).collect();
        self.emit(&atoms(&rule, Look::Dim, None));
    }

    fn finish_code(&mut self) {
        let Some(cb) = self.code.take() else { return };
        self.start_block();
        self.gap = true;
        let text = cb.text.strip_suffix('\n').unwrap_or(&cb.text);
        if text.is_empty() {
            return;
        }
        let mut lines: Vec<String> = text.split('\n').map(|l| l.replace('\t', "    ")).collect();
        let cap = CODE_BLOCK_LINES.min(CODE_DOC_LINES.saturating_sub(self.code_lines));
        let hidden = lines.len().saturating_sub(cap);
        lines.truncate(cap);
        self.code_lines += lines.len();
        let runs = code::highlight(&cb.lang, &lines);
        let rest = self.prefix(false);
        let first = self.take_prefix();
        let limit = self.width.max(rest.width + 2);
        for (n, line_runs) in runs.into_iter().enumerate() {
            let prefix = if n == 0 { &first } else { &rest };
            self.code_line(line_runs, prefix, limit);
        }
        if hidden > 0 {
            let more = atoms(&format!("\u{2026} +{hidden}"), Look::Dim, None);
            self.emit(&more);
        }
        if cb.lang.eq_ignore_ascii_case("mermaid")
            && self.ctx.diagrams
            && crate::build_features::MERMAID
        {
            self.doc.links.push(LinkTarget {
                url: String::new(),
                xref: None,
                diagram: Some(text.to_string()),
            });
            let id = self.doc.links.len() - 1;
            let label = format!(
                "\u{a0}{}\u{a0}",
                gitpoulpi_i18n::msg::item_view::OPEN_DIAGRAM.get()
            );
            self.emit(&atoms(&label, Look::Button, Some(id)));
        }
    }

    /// One code line, hard-wrapped by cells (continuations keep the panel).
    fn code_line(&mut self, runs: code::Runs, prefix: &Prefix, limit: usize) {
        let new_line = |prefix: &Prefix| {
            let mut spans = Vec::new();
            for (t, l) in &prefix.spans {
                spans.push(Span {
                    text: t.clone(),
                    look: *l,
                    link: None,
                    hl: None,
                });
            }
            MdLine {
                spans,
                kind: LineKind::Code { from: prefix.width },
            }
        };
        let mut line = new_line(prefix);
        let mut used = prefix.width;
        for (text, hl) in runs {
            let mut piece = String::new();
            for ch in text.chars() {
                let w = super::layout::cw(ch);
                if used + w > limit && used > prefix.width {
                    if !piece.is_empty() {
                        line.spans.push(Span {
                            text: std::mem::take(&mut piece),
                            look: Look::Plain,
                            link: None,
                            hl,
                        });
                    }
                    self.doc
                        .lines
                        .push(std::mem::replace(&mut line, new_line(prefix)));
                    used = prefix.width;
                }
                piece.push(ch);
                used += w;
            }
            if !piece.is_empty() {
                line.spans.push(Span {
                    text: piece,
                    look: Look::Plain,
                    link: None,
                    hl,
                });
            }
        }
        self.doc.lines.push(line);
    }

    // -- tables -------------------------------------------------------------------

    fn finish_table(&mut self) {
        let Some(t) = self.table.take() else { return };
        let cols = t
            .rows
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .max(t.head.len());
        if cols == 0 {
            return;
        }
        self.start_block();
        self.gap = true;
        let bold = |cell: &[Atom]| -> Vec<Atom> {
            cell.iter()
                .map(|a| Atom {
                    look: if a.look == Look::Plain {
                        Look::Bold
                    } else {
                        a.look
                    },
                    ..*a
                })
                .collect()
        };
        let width_of = |cell: &[Atom]| cell.iter().map(|a| super::layout::cw(a.ch)).sum();
        let mut widths = vec![0usize; cols];
        for row in std::iter::once(&t.head).chain(t.rows.iter()) {
            for (c, cell) in row.iter().enumerate() {
                widths[c] = widths[c].max(width_of(cell));
            }
        }
        let total = 1 + widths.iter().map(|w| w + 3).sum::<usize>();
        let rest = self.prefix(false);
        let first = self.take_prefix();
        if total <= self.width.saturating_sub(rest.width) {
            let border = |l: char, m: char, r: char| -> String {
                let mut s = String::from(l);
                for (c, w) in widths.iter().enumerate() {
                    s.extend(std::iter::repeat_n(RULE, w + 2));
                    s.push(if c + 1 == cols { r } else { m });
                }
                s
            };
            let push_border = |b: &mut Self, text: String, p: &Prefix| {
                let mut line = LineBuf::new(p);
                line.push_str(&text, Look::Dim, None);
                b.doc.lines.push(line.finish());
            };
            push_border(self, border('\u{250C}', '\u{252C}', '\u{2510}'), &first);
            let mut rows: Vec<(Vec<Vec<Atom>>, bool)> = Vec::new();
            if !t.head.is_empty() {
                rows.push((t.head.iter().map(|c| bold(c)).collect(), true));
            }
            rows.extend(t.rows.iter().map(|r| (r.clone(), false)));
            for (n, (row, is_head)) in rows.iter().enumerate() {
                let mut line = LineBuf::new(&rest);
                #[allow(clippy::needless_range_loop)]
                for c in 0..cols {
                    line.push_str(
                        if c == 0 { "\u{2502} " } else { " \u{2502} " },
                        Look::Dim,
                        None,
                    );
                    let empty = Vec::new();
                    let cell = row.get(c).unwrap_or(&empty);
                    let pad = widths[c] - width_of(cell);
                    let (left, right) = match t.aligns.get(c) {
                        Some(Alignment::Right) => (pad, 0),
                        Some(Alignment::Center) => (pad / 2, pad - pad / 2),
                        _ => (0, pad),
                    };
                    line.push_str(&" ".repeat(left), Look::Plain, None);
                    for a in cell {
                        line.push_atom(*a);
                    }
                    line.push_str(&" ".repeat(right), Look::Plain, None);
                }
                line.push_str(" \u{2502}", Look::Dim, None);
                self.doc.lines.push(line.finish());
                if *is_head && n == 0 {
                    push_border(self, border('\u{251C}', '\u{253C}', '\u{2524}'), &rest);
                }
            }
            push_border(self, border('\u{2514}', '\u{2534}', '\u{2518}'), &rest);
            return;
        }
        // Too wide for a box: `header: value` lines per row.
        let mut first = Some(first);
        let mut emit = |b: &mut Self, a: &[Atom], rest: &Prefix| {
            let p = first.take().unwrap_or_else(|| rest.clone());
            let lines = wrap(a, b.width, &p, rest);
            b.doc.lines.extend(lines);
        };
        if t.rows.is_empty() {
            let mut a: Vec<Atom> = Vec::new();
            for (c, cell) in t.head.iter().enumerate() {
                if c > 0 {
                    a.extend(atoms(" | ", Look::Dim, None));
                }
                a.extend(bold(cell));
            }
            emit(self, &a, &rest);
            return;
        }
        for (n, row) in t.rows.iter().enumerate() {
            if n > 0 {
                self.blank();
            }
            for (c, cell) in row.iter().enumerate() {
                if cell.is_empty() {
                    continue;
                }
                let mut a: Vec<Atom> = Vec::new();
                if let Some(h) = t.head.get(c).filter(|h| !h.is_empty()) {
                    a.extend(bold(h));
                    a.extend(atoms(": ", Look::Plain, None));
                }
                a.extend(cell.iter().copied());
                emit(self, &a, &rest);
            }
        }
    }
}

/// `a` without leading and trailing blanks and breaks.
fn trim(mut a: Vec<Atom>) -> Vec<Atom> {
    let blank = |x: &Atom| x.ch.is_whitespace();
    while a.last().is_some_and(blank) {
        a.pop();
    }
    let lead = a.iter().take_while(|x| blank(x)).count();
    a.drain(..lead);
    a
}
