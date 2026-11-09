//! A double-width glyph cut by an overlay's edge.
//!
//! ratatui stores a wide glyph in cell N and a blank continuation in N+1, and
//! its `diff` never emits the cell after a wide glyph (`to_skip`). When an
//! overlay is painted over CJK text and its left edge lands on the glyph's
//! second half, the overlay's border cell sits right after a still-wide cell:
//! the diff skips it and the border is simply never drawn (a missing corner).
//! The right edge is the mirror case and is already covered by ratatui
//! (`invalidated` re-emits the cell after a glyph that was wide before).
//!
//! The fix lives at the source: every overlay starts with a [`Clear`], and
//! this `Clear` blanks the half-glyph the area's left edge would cut. There is
//! deliberately NO whole-frame post-pass: from the finished buffer alone a
//! glyph cut by an overlay cannot be told from a wide glyph a widget painted
//! legitimately over a non-blank cell (ratatui's `Span` leaves the
//! continuation as it was, e.g. the title bar's octopus over the border's
//! horizontal line), and guessing blanked the octopus.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

/// `ratatui::widgets::Clear` that also blanks a wide glyph whose second half
/// lies in the cleared area (the glyph starts left of it).
pub(crate) struct Clear;

impl Widget for Clear {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }
        ratatui::widgets::Clear.render(area, buf);
        blank_cut_glyphs(buf, area);
    }
}

/// Replaces by a space every wide glyph left of `area` that reaches into it.
pub(crate) fn blank_cut_glyphs(buf: &mut Buffer, area: Rect) {
    for y in area.top()..area.bottom() {
        for k in 1..=2u16 {
            let Some(x) = area.x.checked_sub(k) else {
                break;
            };
            if x < buf.area.x {
                break;
            }
            let cell = &buf[(x, y)];
            if cell.diff_option != ratatui::buffer::CellDiffOption::Skip
                && crate::display_width(cell.symbol()) as u16 > k
            {
                buf[(x, y)].set_symbol(" ");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Clear;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;
    use ratatui::Terminal;

    const W: u16 = 60;
    const H: u16 = 12;

    /// The screen: CJK text on every row, shifted by one column on odd rows
    /// so every overlay x meets both glyph parities.
    fn fill_cjk(buf: &mut Buffer) {
        for y in 0..H {
            let pad = if y % 2 == 0 { "" } else { "a" };
            let text = format!(
                "{pad}{}",
                "\u{4e2d}\u{6587}\u{65e5}\u{672c}\u{8a9e}".repeat(12)
            );
            buf.set_stringn(0, y, &text, W as usize, ratatui::style::Style::default());
        }
    }

    /// Draws the CJK screen, then `overlay`, through a real `Terminal` so ratatui's diff decides what is written.
    fn draw(overlay: impl FnOnce(&mut ratatui::Frame)) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(W, H)).unwrap();
        terminal
            .draw(|frame| {
                fill_cjk(frame.buffer_mut());
                overlay(frame);
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    /// No wide glyph is followed by anything but its blank continuation.
    fn assert_no_straddle(what: &str, buf: &Buffer) {
        for y in 0..H {
            for x in 0..W - 1 {
                if crate::display_width(buf[(x, y)].symbol()) == 2 {
                    assert!(
                        buf[(x + 1, y)].symbol() == " ",
                        "{what}: wide glyph at ({x},{y}) is cut by {:?}",
                        buf[(x + 1, y)].symbol()
                    );
                }
            }
        }
    }

    fn assert_box(what: &str, buf: &Buffer, r: Rect) {
        for (x, y, want) in [
            (r.x, r.y, "\u{256d}"),
            (r.right() - 1, r.y, "\u{256e}"),
            (r.x, r.bottom() - 1, "\u{2570}"),
            (r.right() - 1, r.bottom() - 1, "\u{256f}"),
        ] {
            assert_eq!(buf[(x, y)].symbol(), want, "{what}: corner at ({x},{y})");
        }
        for y in r.y + 1..r.bottom() - 1 {
            for x in [r.x, r.right() - 1] {
                assert_eq!(buf[(x, y)].symbol(), "\u{2502}", "{what}: side ({x},{y})");
            }
        }
    }

    #[test]
    fn a_dialog_panel_keeps_its_border_at_every_column_parity() {
        for x in 6..=9 {
            for w in [11, 12] {
                let rect = Rect::new(x, 2, w, 7);
                let what = format!("panel x={x} w={w}");
                let buf = draw(|f| crate::dialog_kit::panel(f.buffer_mut(), rect, "T"));
                assert_box(&what, &buf, rect);
                assert_no_straddle(&what, &buf);
            }
        }
    }

    #[test]
    fn a_popup_list_keeps_its_border_at_every_column_parity() {
        use crate::context_menu::Menu;
        use crate::refs_sidebar::SidebarSection;
        for col in 6..=9 {
            let ctx = crate::log_menu::MenuContext::default();
            let entries = crate::ref_menu::ref_menu_entries(
                SidebarSection::Local,
                "feature",
                &ctx,
                false,
                false,
                false,
            );
            let menu = Menu::new("feature", entries, col, 2);
            let mut rects = Vec::new();
            let what = format!("menu col={col}");
            let buf = draw(|f| rects = menu.render(f, None));
            assert_no_straddle(&what, &buf);
            // The rects are the rows inside the border.
            let (left, top) = (rects[0].x - 1, rects[0].y - 1);
            assert_eq!(buf[(left, top)].symbol(), "\u{256d}", "{what}");
            let mut y = top + 1;
            while y < H && buf[(left, y)].symbol() != "\u{2570}" {
                assert_ne!(buf[(left, y)].symbol(), " ", "{what}: side row {y}");
                y += 1;
            }
            assert!(y < H, "{what}: no bottom-left corner");
        }
    }

    #[test]
    fn clear_blanks_only_a_glyph_its_left_edge_cuts() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 8, 1));
        buf.set_string(0, 0, "\u{4e2d}\u{6587}", ratatui::style::Style::default());
        // Starts on a glyph boundary: nothing is cut.
        let before = buf.clone();
        Clear.render(Rect::new(2, 0, 2, 1), &mut buf);
        assert_eq!(buf[(0, 0)].symbol(), before[(0, 0)].symbol());
        // Starts on a continuation: the glyph is blanked.
        Clear.render(Rect::new(1, 0, 3, 1), &mut buf);
        assert_eq!(buf[(0, 0)].symbol(), " ");
    }

    /// The title bar's octopus sits over the border's horizontal line, which
    /// ratatui's `Span` leaves in the continuation cell. A whole-frame
    /// "continuation is not blank" pass erased it; a wide glyph a widget
    /// paints itself must survive the frame.
    #[test]
    fn a_wide_glyph_the_app_paints_over_a_border_survives_the_frame() {
        use crate::chrome::{render_about_button, ABOUT_LABEL};
        use ratatui::widgets::{Block, Borders, Paragraph};
        let mut terminal = Terminal::new(TestBackend::new(30, 4)).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                frame.render_widget(Block::default().borders(Borders::ALL).title("t"), area);
                let mut hits = Vec::new();
                render_about_button(frame, 1, 0, false, &mut hits);
                // An emoji row over a bordered row (sidebar / About style).
                frame.render_widget(
                    Paragraph::new("\u{1F419} octo"),
                    Rect::new(0, 2, area.width, 1),
                );
            })
            .unwrap();
        let buf = terminal.backend().buffer();
        let octopus = ABOUT_LABEL.chars().next().unwrap().to_string();
        assert_eq!(buf[(1, 0)].symbol(), octopus, "title bar octopus");
        assert_eq!(buf[(0, 2)].symbol(), octopus, "emoji row");
    }
}
