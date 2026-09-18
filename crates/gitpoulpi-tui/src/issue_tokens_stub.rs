//! `issue_tokens` for a build with no hosting provider: a commit
//! message's `#123` / `!45` are never links, so no token is ever found
//! and the message is painted as plain text.
#![allow(dead_code)]

use ratatui::style::Style;
use ratatui::text::{Line, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Link {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) reference: String,
    pub(crate) url: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TokenBases;

pub(crate) fn links(_text: &str, _bases: &TokenBases) -> Vec<Link> {
    Vec::new()
}

pub(crate) fn styled_line(
    text: &str,
    base: Style,
    _links: &[Link],
    _hot: &[bool],
) -> Line<'static> {
    Line::from(Span::styled(text.to_string(), base))
}
