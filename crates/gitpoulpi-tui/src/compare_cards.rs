//! The comparison panel's cards: every compared commit has
//! its own zone (avatar, author, date, sha, summary); here each selected Log
//! row is a two-line card between the panel's header and its file list.
//!
//! ```text
//! ▎AB Jane Doe               3 days ago  a1b2c3d
//! ▎   Fix the parser                          [To]
//! ▎📦 stash@{0}              2 hours ago  9f8e7d6
//! ▎   WIP on main: tidy
//! ▎✎  Working tree
//! ▎   Uncommitted changes
//! ```
//!
//! This module only builds the lines (pure: the caller hands in the clock,
//! the avatar mode and the hover); the right pane paints them inside the
//! message block, which already scrolls, drags and resizes, and registers
//! one `HitTarget::CompareCard` per painted line. The avatar pictures are
//! laid over the initials slot of the first line by the caller, through the
//! Log's own `avatar::LogAvatarImages`.

use gitpoulpi_config::AvatarFallback;
use gitpoulpi_i18n::msg;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::avatar::{self, LogAvatarMode};
use crate::compare::{relative_age, Card, CardKind, CompareSpec, EndMark};
use crate::theme::{th, STASH_ICON};
use crate::ui_kit::{display_width, short_oid, truncate_with_ellipsis};

/// Cells the avatar slot takes before the text: two for the picture or its
/// initials plus a gap; nothing when the Log shows no avatar either.
const LEAD: usize = 3;

/// One painted line of the cards section.
pub(crate) struct CardRow {
    pub(crate) line: Line<'static>,
    /// Index into `CompareSpec::cards`; `None` for the range note.
    pub(crate) card: Option<usize>,
    /// The card's first line: where its avatar picture goes.
    pub(crate) first: bool,
}

/// What the cards are drawn with.
pub(crate) struct CardLook {
    /// Inner width of the block the lines fill.
    pub(crate) width: usize,
    pub(crate) now: i64,
    pub(crate) avatars: LogAvatarMode,
    pub(crate) fallback: AvatarFallback,
    /// The card under the pointer.
    pub(crate) hovered: Option<usize>,
    /// The display row of the Log's cursor, whose card is lit.
    pub(crate) cursor_row: Option<usize>,
}

/// Where the first line's avatar slot starts, relative to the line.
pub(crate) const SLOT_X: u16 = 1;

/// The picture slot of the card whose first line is painted `line` rows
/// below the block's top, or `None` when it does not fit the block.
pub(crate) fn slot_area(inner: ratatui::layout::Rect, line: u16) -> Option<ratatui::layout::Rect> {
    let area = ratatui::layout::Rect::new(inner.x + SLOT_X, inner.y + line, avatar::SLOT_COLS, 1);
    (area.right() <= inner.right()).then_some(area)
}

/// Lines the cards take: two per card, one more for a range's note.
pub(crate) fn line_count(spec: &CompareSpec) -> usize {
    spec.cards.len() * 2 + usize::from(spec.rows > 2)
}

pub(crate) fn build(spec: &CompareSpec, look: &CardLook) -> Vec<CardRow> {
    let mut rows = Vec::with_capacity(line_count(spec));
    if spec.rows > 2 {
        rows.push(CardRow {
            line: Line::from(Span::styled(
                truncate_with_ellipsis(&msg::compare::range_note(spec.rows as u64), look.width),
                Style::default().fg(th().ui.muted),
            )),
            card: None,
            first: false,
        });
    }
    for (i, card) in spec.cards.iter().enumerate() {
        let lit = look.hovered == Some(i);
        let current = look.cursor_row == Some(card.row);
        let bg = if lit {
            Some(th().ui.hover)
        } else if current {
            Some(th().ui.selection)
        } else {
            None
        };
        let [first, second] = card_lines(card, look, bg);
        rows.push(CardRow {
            line: first,
            card: Some(i),
            first: true,
        });
        rows.push(CardRow {
            line: second,
            card: Some(i),
            first: false,
        });
    }
    rows
}

fn card_lines(
    card: &Card,
    look: &CardLook,
    bg: Option<ratatui::style::Color>,
) -> [Line<'static>; 2] {
    let paint = |style: Style| bg.map_or(style, |b| style.bg(b));
    let lead = if look.avatars == LogAvatarMode::Off {
        0
    } else {
        LEAD
    };
    let bar_color = match card.kind {
        CardKind::Wip => th().graph.wip,
        CardKind::Stash => th().graph.stash,
        CardKind::Commit => avatar::badge(&card.author_name, &card.author_email).color,
    };
    let bar = Span::styled("\u{258E}", paint(Style::default().fg(bar_color)));
    let inner = look.width.saturating_sub(1);

    // -- what leads the first line: the picture's slot --
    let mut first: Vec<Span<'static>> = vec![bar.clone()];
    if lead > 0 {
        let (glyph, color) = match card.kind {
            CardKind::Commit => {
                let cols = if look.avatars == LogAvatarMode::Picture {
                    usize::from(avatar::SLOT_COLS)
                } else {
                    1
                };
                avatar::fallback_text(look.fallback, &card.author_name, &card.author_email, cols)
            }
            CardKind::Stash => (STASH_ICON.to_string(), th().ui.muted),
            CardKind::Wip => ("\u{270E}".to_string(), th().status.warning),
        };
        let used = display_width(&glyph);
        first.push(Span::styled(
            glyph,
            paint(Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ));
        first.push(Span::styled(
            " ".repeat(lead.saturating_sub(used)),
            paint(Style::default()),
        ));
    }

    // -- name on the left, date and sha on the right --
    let (name, name_style) = match card.kind {
        CardKind::Wip => (
            msg::compare::WORKING_TREE.get().to_string(),
            Style::default().fg(th().status.warning),
        ),
        CardKind::Stash => (card.author_name.clone(), Style::default().fg(th().ui.muted)),
        CardKind::Commit => (
            if card.author_name.is_empty() {
                card.author_email.clone()
            } else {
                card.author_name.clone()
            },
            Style::default().fg(th().ui.text),
        ),
    };
    let sha = card.id.map(short_oid).unwrap_or_default();
    let date = if card.kind == CardKind::Wip {
        String::new()
    } else {
        relative_age(look.now, card.timestamp)
    };
    let room = inner.saturating_sub(lead);
    // The right-hand facts go first when the pane is narrow: the date, then
    // the sha; the name keeps at least six cells.
    let mut right = match (date.is_empty(), sha.is_empty()) {
        (false, false) => format!("{date}  {sha}"),
        (true, false) => sha.clone(),
        (false, true) => date.clone(),
        (true, true) => String::new(),
    };
    if display_width(&right) + 7 > room {
        right = sha.clone();
    }
    if display_width(&right) + 7 > room {
        right.clear();
    }
    let name_room = room.saturating_sub(display_width(&right) + usize::from(!right.is_empty()));
    let name = truncate_with_ellipsis(&name, name_room);
    let gap = room.saturating_sub(display_width(&name) + display_width(&right));
    first.push(Span::styled(
        name,
        paint(name_style.add_modifier(Modifier::BOLD)),
    ));
    first.push(Span::styled(" ".repeat(gap), paint(Style::default())));
    first.push(Span::styled(
        right,
        paint(Style::default().fg(th().ui.muted)),
    ));

    // -- the summary, and the From / To tag on the right --
    let tag = card.mark.map(|m| {
        format!(
            " {} ",
            match m {
                EndMark::From => msg::compare::FROM_TAG.get(),
                EndMark::To => msg::compare::TO_TAG.get(),
            }
        )
    });
    let tag_w = tag.as_deref().map_or(0, display_width);
    let (text, text_style) = match card.kind {
        CardKind::Wip => (
            msg::compare::UNCOMMITTED.get().to_string(),
            Style::default().fg(th().ui.muted),
        ),
        CardKind::Stash => (
            card.summary.clone(),
            Style::default()
                .fg(th().ui.muted)
                .add_modifier(Modifier::ITALIC),
        ),
        CardKind::Commit => (card.summary.clone(), Style::default().fg(th().ui.text)),
    };
    let summary_room = room.saturating_sub(tag_w + usize::from(tag_w > 0));
    let text = truncate_with_ellipsis(&text, summary_room);
    let gap2 = room.saturating_sub(display_width(&text) + tag_w);
    let mut second: Vec<Span<'static>> = vec![
        bar,
        Span::styled(" ".repeat(lead), paint(Style::default())),
        Span::styled(text, paint(text_style)),
        Span::styled(" ".repeat(gap2), paint(Style::default())),
    ];
    if let Some(tag) = tag {
        second.push(Span::styled(
            tag,
            Style::default()
                .fg(th().ui.on_accent)
                .bg(th().ui.accent)
                .add_modifier(Modifier::BOLD),
        ));
    }
    [Line::from(first), Line::from(second)]
}

#[cfg(test)]
mod tests;
