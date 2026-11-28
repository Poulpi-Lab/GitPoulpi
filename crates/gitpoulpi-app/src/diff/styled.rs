//! Neutral styled lines: what the terminal UI paints with its theme, said
//! without a toolkit. A [`StyledLine`] is one *logical* row (no wrapping, no
//! scrolling, no cell padding: those are layout and stay with the frontend);
//! each span names a [`DiffRole`] that the frontend resolves against its own
//! theme.

use super::intraline::CharRange;
use gitpoulpi_core::domain::{human_size, Hunk, LfsChange, LfsPointer, LineOrigin};
use gitpoulpi_i18n::msg::diff_view as msg;

/// What a span is, in the terms the terminal UI really distinguishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiffRole {
    /// An added line, its `+` marker and its text.
    Add,
    /// A removed line, its `-` marker and its text.
    Delete,
    /// An unchanged line and its blank marker.
    Context,
    /// Line numbers and the Split column separator.
    Gutter,
    /// The `@@ -a,b +c,d @@` row (bold).
    HunkHeader,
    /// A "nothing to show" row (muted italic).
    Notice,
    /// A card heading (bold, normal text colour).
    Title,
    /// Plain text in the normal text colour.
    Text,
    /// Secondary text (muted).
    Muted,
}

impl DiffRole {
    /// The role of a diff line's text and marker.
    pub fn of_origin(origin: &LineOrigin) -> Self {
        match origin {
            LineOrigin::Add => DiffRole::Add,
            LineOrigin::Delete => DiffRole::Delete,
            LineOrigin::Context => DiffRole::Context,
        }
    }
}

/// An sRGB colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// A syntax-highlight style laid over a span's role. Defined for the
/// highlighters that come later; the diff rows carry `None` today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxStyle {
    pub fg: Rgb,
    pub bold: bool,
    pub italic: bool,
}

/// A run of text with one look.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledSpan {
    pub text: String,
    pub role: DiffRole,
    /// A changed word: drawn stronger than the rest of its line.
    pub emph: bool,
    pub syntax: Option<SyntaxStyle>,
}

impl StyledSpan {
    pub fn new(text: impl Into<String>, role: DiffRole) -> Self {
        Self {
            text: text.into(),
            role,
            emph: false,
            syntax: None,
        }
    }
}

/// One logical row as a list of spans.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StyledLine {
    pub spans: Vec<StyledSpan>,
}

impl StyledLine {
    /// A line of one span.
    pub fn single(text: impl Into<String>, role: DiffRole) -> Self {
        Self {
            spans: vec![StyledSpan::new(text, role)],
        }
    }

    /// The text of every span, joined.
    pub fn plain(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }
}

/// Width of the Inline gutter (`{old:>5} {new:>5} `).
pub const GUTTER_WIDTH: usize = 12;

/// The Inline gutter text: both line numbers, or blanks of the same width
/// when line numbers are off (so selection geometry does not move).
pub fn gutter_text(old: Option<u32>, new: Option<u32>, line_numbers: bool) -> String {
    if !line_numbers {
        return " ".repeat(GUTTER_WIDTH);
    }
    let o = old.map(|n| n.to_string()).unwrap_or_default();
    let n = new.map(|n| n.to_string()).unwrap_or_default();
    format!("{o:>5} {n:>5} ")
}

/// The one-character marker of a line and its role.
pub fn marker(origin: &LineOrigin) -> (&'static str, DiffRole) {
    match origin {
        LineOrigin::Add => ("+", DiffRole::Add),
        LineOrigin::Delete => ("-", DiffRole::Delete),
        LineOrigin::Context => (" ", DiffRole::Context),
    }
}

/// `text` cut into spans along `marked` (char ranges): the marked stretches
/// carry `emph`, everything else does not. Context lines pass no ranges.
pub fn content_spans(text: &str, marked: &[CharRange], role: DiffRole) -> Vec<StyledSpan> {
    if marked.is_empty() {
        return vec![StyledSpan::new(text, role)];
    }
    let mut spans: Vec<StyledSpan> = Vec::new();
    let mut run = String::new();
    let mut run_marked = false;
    let mut at = 0usize;
    for (i, c) in text.chars().enumerate() {
        while at < marked.len() && marked[at].1 <= i {
            at += 1;
        }
        let is_marked = at < marked.len() && marked[at].0 <= i;
        if is_marked != run_marked && !run.is_empty() {
            spans.push(StyledSpan {
                emph: run_marked,
                ..StyledSpan::new(std::mem::take(&mut run), role)
            });
        }
        run_marked = is_marked;
        run.push(c);
    }
    if !run.is_empty() {
        spans.push(StyledSpan {
            emph: run_marked,
            ..StyledSpan::new(run, role)
        });
    }
    spans
}

/// One line in the Inline layout: gutter, marker, text (changed words
/// emphasised on Add and Delete lines).
pub fn unified_line(
    origin: &LineOrigin,
    text: &str,
    (old, new): (Option<u32>, Option<u32>),
    marked: &[CharRange],
    line_numbers: bool,
) -> StyledLine {
    let (mark, role) = marker(origin);
    let marked = if role == DiffRole::Context {
        &[][..]
    } else {
        marked
    };
    let mut spans = vec![
        StyledSpan::new(gutter_text(old, new, line_numbers), DiffRole::Gutter),
        StyledSpan::new(mark, role),
    ];
    spans.extend(content_spans(text, marked, role));
    StyledLine { spans }
}

/// One half of a Split row: line number, marker, text.
pub fn split_side_line(
    origin: &LineOrigin,
    text: &str,
    number: u32,
    marked: &[CharRange],
    line_numbers: bool,
) -> StyledLine {
    let (mark, role) = marker(origin);
    let marked = if role == DiffRole::Context {
        &[][..]
    } else {
        marked
    };
    let num = if line_numbers {
        format!("{number:>5} ")
    } else {
        " ".repeat(6)
    };
    let mut spans = vec![
        StyledSpan::new(num, DiffRole::Gutter),
        StyledSpan::new(mark, role),
    ];
    spans.extend(content_spans(text, marked, role));
    StyledLine { spans }
}

/// The `@@ -a,b +c,d @@` header row of a hunk.
pub fn hunk_header(hunk: &Hunk) -> StyledLine {
    StyledLine::single(
        format!(
            "@@ -{},{} +{},{} @@",
            hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines
        ),
        DiffRole::HunkHeader,
    )
}

/// A "nothing to show" row.
pub fn notice(text: impl Into<String>) -> StyledLine {
    StyledLine::single(text, DiffRole::Notice)
}

/// A Git LFS file's diff as a card: the object's old and new size and short
/// oid, and whether the new object is in the local store.
pub fn lfs_card(change: &LfsChange, local: Option<bool>) -> Vec<StyledLine> {
    let short = |p: &LfsPointer| format!("{}...", p.short_oid());
    let mut lines = vec![StyledLine::single(msg::LFS_TITLE.get(), DiffRole::Title)];
    let mut push = |text: String, role: DiffRole| lines.push(StyledLine::single(text, role));
    match (&change.old, &change.new) {
        (Some(old), Some(new)) => {
            push(
                format!("{} -> {}", human_size(old.size), human_size(new.size)),
                DiffRole::Text,
            );
            push(
                format!("sha256 {} -> {}", short(old), short(new)),
                DiffRole::Muted,
            );
        }
        (None, Some(new)) => {
            push(msg::lfs_new(human_size(new.size)), DiffRole::Text);
            push(format!("sha256 {}", short(new)), DiffRole::Muted);
        }
        (Some(old), None) => {
            push(msg::lfs_removed(human_size(old.size)), DiffRole::Text);
            push(format!("sha256 {}", short(old)), DiffRole::Muted);
        }
        (None, None) => {}
    }
    if change.new.is_some() {
        push(
            match local {
                Some(true) => msg::LFS_LOCAL.get().to_string(),
                Some(false) => msg::LFS_NOT_DOWNLOADED.get().to_string(),
                None => msg::LFS_UNKNOWN.get().to_string(),
            },
            DiffRole::Muted,
        );
    }
    push(msg::LFS_RAW_HINT.get().to_string(), DiffRole::Muted);
    lines
}
