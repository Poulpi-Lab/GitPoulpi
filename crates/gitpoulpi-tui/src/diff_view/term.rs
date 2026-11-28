//! The terminal side of the diff's neutral styled lines: each `DiffRole`
//! resolved against the theme, each `StyledLine` turned into a ratatui `Line`.

use super::intraline::emph_for;
use crate::th;
use gitpoulpi_app::diff::{DiffRole, StyledLine, StyledSpan};
use gitpoulpi_core::domain::LineOrigin;
use ratatui::prelude::*;

/// The theme's look for a role.
pub(crate) fn role_style(role: DiffRole) -> Style {
    let t = th();
    match role {
        DiffRole::Add => Style::default().fg(t.diff.add_fg).bg(t.diff.add_bg),
        DiffRole::Delete => Style::default().fg(t.diff.del_fg).bg(t.diff.del_bg),
        DiffRole::Context | DiffRole::Text => Style::default().fg(t.ui.text),
        DiffRole::Gutter => Style::default().fg(t.diff.gutter_fg),
        DiffRole::HunkHeader => Style::default()
            .fg(t.diff.hunk_fg)
            .add_modifier(Modifier::BOLD),
        DiffRole::Notice => Style::default()
            .fg(t.ui.muted)
            .add_modifier(Modifier::ITALIC),
        DiffRole::Title => Style::default().fg(t.ui.text).add_modifier(Modifier::BOLD),
        DiffRole::Muted => Style::default().fg(t.ui.muted),
    }
}

/// The look of a span: its role, stronger on a changed word of an added or
/// removed line.
pub(crate) fn span_style(span: &StyledSpan) -> Style {
    let base = role_style(span.role);
    if !span.emph {
        return base;
    }
    let origin = match span.role {
        DiffRole::Add => LineOrigin::Add,
        DiffRole::Delete => LineOrigin::Delete,
        _ => return base,
    };
    emph_for(&origin, base).unwrap_or(base)
}

/// A neutral line as a ratatui line.
pub(crate) fn line_of(line: &StyledLine) -> Line<'static> {
    Line::from(
        line.spans
            .iter()
            .map(|s| Span::styled(s.text.clone(), span_style(s)))
            .collect::<Vec<_>>(),
    )
}
