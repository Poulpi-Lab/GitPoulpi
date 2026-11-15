//! Markdown as styled, wrapped, clickable terminal lines.
//!
//! `render` turns a Markdown text into a [`Doc`] laid out for one width: lines
//! of styled spans (built on `pulldown-cmark`), plus the targets of its links.
//! `paint` writes one line into a buffer and, in the same walk, reports (and
//! optionally registers as a hand-pointer control) the rect of every link span
//! it wrote, so the hit rects are exactly the underlined cells.
//!
//! Pipeline: [`sanitize`] (no control characters, no emoji-presentation or
//! joiner characters reach a cell) -> blocks (`blocks.rs`) -> links
//! (`links.rs`) -> layout by cells (`layout.rs`) -> paint. Code blocks are
//! highlighted by `code.rs`; `cache.rs` keeps laid-out documents per width and
//! theme.

// Not every build shows Markdown (the review view needs a hosting provider).
#![allow(dead_code)]

mod blocks;
mod cache;
mod code;
mod layout;
mod links;
#[cfg(test)]
mod tests;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

pub(crate) use cache::DocCache;

use crate::th;

/// Index into [`Doc::links`].
pub(crate) type LinkId = usize;

/// Where a link goes. Only web addresses are ever offered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LinkTarget {
    pub(crate) url: String,
}

/// How a run of text is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Look {
    Plain,
    Bold,
    Italic,
    BoldItalic,
    Strike,
    /// Inline code.
    Code,
    /// A link's text.
    Link,
    Heading,
    Quote,
    /// A list bullet or a quote bar.
    Marker,
    /// Task marks, rules, table borders, footnote labels.
    Dim,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Span {
    pub(crate) text: String,
    pub(crate) look: Look,
    pub(crate) link: Option<LinkId>,
    /// A syntax-highlight style (code blocks): its foreground and modifiers
    /// replace the look's.
    pub(crate) hl: Option<Style>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum LineKind {
    #[default]
    Text,
    /// A code line: from cell `from` to the right edge it sits on the panel
    /// background.
    Code { from: usize },
}

/// One screen line.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct MdLine {
    pub(crate) spans: Vec<Span>,
    pub(crate) kind: LineKind,
}

impl MdLine {
    /// The line without styling (tests, width checks).
    pub(crate) fn text(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }

    pub(crate) fn width(&self) -> usize {
        self.spans
            .iter()
            .map(|s| crate::cell_text::cells(&s.text))
            .sum()
    }
}

/// A laid-out document.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Doc {
    pub(crate) lines: Vec<MdLine>,
    pub(crate) links: Vec<LinkTarget>,
}

/// What references resolve against (the repository's linked sources).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Context {
    pub(crate) bases: crate::issue_tokens::TokenBases,
}

/// Longest code block shown, and longest total code per document.
pub(crate) const CODE_BLOCK_LINES: usize = 400;
pub(crate) const CODE_DOC_LINES: usize = 2000;
/// Narrowest layout width (the callers' panes never go below it).
pub(crate) const MIN_WIDTH: usize = 8;

/// `text` laid out for `width` cells.
pub(crate) fn render(text: &str, width: usize, ctx: &Context) -> Doc {
    blocks::build(&sanitize(text), width.max(MIN_WIDTH), ctx)
}

/// True for the characters that never reach a cell: controls (C0/C1) other
/// than newline and tab, the emoji and joiner machinery (VS15/VS16, ZWJ,
/// skin-tone modifiers) and the bidirectional overrides.
pub(crate) fn is_banned(c: char) -> bool {
    if c == '\n' || c == '\t' {
        return false;
    }
    c.is_control()
        || matches!(
            c,
            '\u{FE0E}'
                | '\u{FE0F}'
                | '\u{200B}'..='\u{200F}'
                | '\u{202A}'..='\u{202E}'
                | '\u{2066}'..='\u{2069}'
                | '\u{1F3FB}'..='\u{1F3FF}'
        )
}

// TODO: switch to `gitpoulpi_hosting::markup::sanitize` once that slice merges.
/// Strips what must not reach the terminal (see [`is_banned`]) plus combining
/// marks with no base to sit on. `\r` is dropped (`\r\n` becomes `\n`).
pub(crate) fn sanitize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut prev: Option<char> = None;
    for c in text.chars() {
        if c == '\r' || is_banned(c) {
            continue;
        }
        let lone_mark = crate::cell_text::char_cells(c) == 0
            && !c.is_control()
            && prev.is_none_or(char::is_whitespace);
        if lone_mark {
            continue;
        }
        out.push(c);
        prev = Some(c);
    }
    out
}

/// The style of a [`Look`] over `base` (the surrounding text's style).
pub(crate) fn look_style(look: Look, base: Style) -> Style {
    let t = th();
    match look {
        Look::Plain => base,
        Look::Bold => base.add_modifier(Modifier::BOLD),
        Look::Italic => base.add_modifier(Modifier::ITALIC),
        Look::BoldItalic => base.add_modifier(Modifier::BOLD | Modifier::ITALIC),
        Look::Strike => base.add_modifier(Modifier::CROSSED_OUT),
        Look::Code => base.fg(t.ui.accent).bg(t.ui.side_bg),
        Look::Link => base.fg(t.ui.accent).add_modifier(Modifier::UNDERLINED),
        Look::Heading => base.fg(t.ui.accent).add_modifier(Modifier::BOLD),
        Look::Quote => base.fg(t.ui.muted).add_modifier(Modifier::ITALIC),
        Look::Marker => base.fg(t.ui.accent),
        Look::Dim => base.fg(t.ui.muted),
    }
}

/// Writes `line` at (`x`, `y`), at most `max` cells wide, over `base`. The
/// whole row of `max` cells is cleared first, so cells left by a previous
/// frame (a wider layout) never show. Returns the rect and link of every
/// link span written; with `register` each also becomes a hand-pointer
/// control (callers that register their own hit list pass `false`).
pub(crate) fn paint(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    max: u16,
    line: &MdLine,
    base: Style,
    register: bool,
) -> Vec<(Rect, LinkId)> {
    let mut hits = Vec::new();
    let area = buf.area;
    if max == 0 || y < area.y || y >= area.bottom() || x >= area.right() {
        return hits;
    }
    let end = x + max.min(area.right() - x);
    for cx in x..end {
        buf[(cx, y)].reset();
        buf[(cx, y)].set_style(base);
    }
    let code = matches!(line.kind, LineKind::Code { .. });
    if let LineKind::Code { from } = line.kind {
        let start = x.saturating_add(from as u16).min(end);
        for px in start..end {
            buf[(px, y)].set_style(base.bg(th().ui.side_bg));
        }
    }
    let mut cx = x;
    for span in &line.spans {
        if cx >= end {
            break;
        }
        let mut style = look_style(span.look, base);
        if code && span.look != Look::Dim && span.look != Look::Marker {
            style = base.bg(th().ui.side_bg);
        }
        if let Some(hl) = span.hl {
            style = style.patch(hl);
        }
        let (nx, _) = buf.set_stringn(cx, y, &span.text, usize::from(end - cx), style);
        if let Some(id) = span.link {
            if nx > cx {
                let rect = Rect::new(cx, y, nx - cx, 1);
                if register {
                    crate::pointer_map::hand(rect);
                }
                hits.push((rect, id));
            }
        }
        cx = nx;
    }
    hits
}
