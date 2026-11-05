//! The author cell and its avatar slot.

use super::*;
use super::{changes_off, sample_commit};
use crate::graph::LANE_PALETTE;

/// The Author cell alone, for `mode`, as text.
fn render_author_cell(mode: avatar::LogAvatarMode, name: &str) -> String {
    render_author_buf(
        mode,
        name,
        "x@example.com",
        gitpoulpi_config::AvatarFallback::Initials,
    )
    .content
    .iter()
    .map(|c| c.symbol())
    .collect()
}

fn render_author_buf(
    mode: avatar::LogAvatarMode,
    name: &str,
    email: &str,
    fallback: gitpoulpi_config::AvatarFallback,
) -> ratatui::buffer::Buffer {
    use ratatui::buffer::Buffer;
    use ratatui::widgets::Widget;

    let mut commit = sample_commit(3, "x");
    commit.author_name = name.to_string();
    commit.author_email = email.to_string();
    let order = [LogColumn::Author];
    let width = 16u16;
    let mut overrides = std::collections::HashMap::new();
    overrides.insert(LogColumn::Author, width);
    let widths = LogColumnWidths::for_area_width(82, &changes_off(), &overrides);
    let row = commit_row(
        &commit,
        &[],
        Line::default(),
        LANE_PALETTE[0],
        &widths,
        &order,
        false,
        false,
        None,
        false,
        mode,
        fallback,
        None,
    );
    let area = Rect::new(0, 0, widths.width_of(LogColumn::Author), 1);
    let mut buf = Buffer::empty(area);
    Widget::render(
        Table::new(vec![row], [Constraint::Length(area.width)]),
        area,
        &mut buf,
    );
    buf
}

/// With no picture protocol the Author column must still tell authors
/// apart, whatever `avatar_fallback` says: an identicon squeezed into one
/// cell is blank for some authors, so every row shows its initial in its
/// badge color.
///
/// Falsified: dropped the one-cell override in `avatar::fallback_text`,
/// watched blocks (and blanks) replace the letters, restored.
#[test]
fn a_text_terminal_shows_every_author_by_initials_even_with_the_identicon_fallback() {
    for i in 0..60 {
        let name = format!("Zed{i} Quux");
        let email = format!("author{i}@example.com");
        let buf = render_author_buf(
            avatar::LogAvatarMode::Glyph,
            &name,
            &email,
            gitpoulpi_config::AvatarFallback::Identicon,
        );
        let cell = &buf[(0, 0)];
        assert_eq!(cell.symbol(), "Z", "{email}");
        assert_eq!(cell.fg, avatar::badge(&name, &email).color, "{email}");
    }
}

/// On an image terminal the Author cell opens with the picture's
/// two-cell slot - the two-letter initials until the picture covers
/// them - then a gutter, then the name, so `avatar::slot_area` (the
/// column's two leading cells) lands exactly on the letters.
///
/// Falsified: rendered `badge_glyph` (one cell) in `Picture` mode,
/// watched the name start one column early, restored.
#[test]
fn the_picture_slot_is_the_author_cells_two_leading_cells() {
    let text = render_author_cell(avatar::LogAvatarMode::Picture, "Ada Lovelace");
    assert!(text.starts_with("AL Ada"), "{text:?}");
    let text = render_author_cell(avatar::LogAvatarMode::Picture, "Cher");
    assert!(text.starts_with("CH Cher"), "{text:?}");
    // One letter is padded to the slot's width.
    let text = render_author_cell(avatar::LogAvatarMode::Picture, "");
    assert!(text.starts_with("?  "), "{text:?}");
    // A text terminal keeps its one-cell glyph, and off is the name.
    let text = render_author_cell(avatar::LogAvatarMode::Glyph, "Ada Lovelace");
    assert!(text.starts_with("A Ada"), "{text:?}");
    let text = render_author_cell(avatar::LogAvatarMode::Off, "Ada Lovelace");
    assert!(text.starts_with("Ada"), "{text:?}");
}
