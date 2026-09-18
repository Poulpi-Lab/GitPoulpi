//! `ui.column_resize` in the Log table: where each mode grabs, what rests in
//! the header, what a hover draws and that the gap never looks like a hole.

use super::*;
use super::{changes_off, log_geometry, sample_commit, shown_order};
use gitpoulpi_config::ColumnResize;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;

fn set(mode: ColumnResize) {
    crate::live_settings::install_column_resize(mode);
}

/// The internal boundaries as `(left column, gap x, next x)`.
fn boundaries(offsets: &[(LogColumn, u16, u16)]) -> Vec<(LogColumn, u16, u16)> {
    offsets
        .iter()
        .filter_map(|&(col, ..)| log_boundary_x(offsets, col).map(|bx| (col, bx - 1, bx)))
        .collect()
}

/// Three commit rows, the middle one selected, over the shipped columns, with
/// the resting ticks painted over the header.
fn rendered(select_middle: bool) -> (Buffer, Rect, Vec<(LogColumn, u16, u16)>) {
    let (log_area, offsets) = log_geometry();
    let widths = LogColumnWidths::for_area_width(
        log_area.width,
        &changes_off(),
        &std::collections::HashMap::new(),
    );
    let order = shown_order();
    let rows: Vec<Row> = (0..3u8)
        .map(|i| {
            commit_row(
                &sample_commit(i + 1, "a commit"),
                &[],
                Line::from(""),
                th().ui.accent,
                &widths,
                &order,
                select_middle && i == 1,
                false,
                None,
                false,
                avatar::LogAvatarMode::Off,
                gitpoulpi_config::AvatarFallback::Initials,
                None,
            )
        })
        .collect();
    let mut terminal = Terminal::new(TestBackend::new(
        log_area.width,
        log_area.y + log_area.height,
    ))
    .unwrap();
    let mut state = TableState::default();
    terminal
        .draw(|frame| {
            let header: Vec<Cell> = order.iter().map(|&c| Cell::from(c.title())).collect();
            frame.render_stateful_clipped(
                log_table(rows, header, &order, &widths, Style::default()),
                log_area,
                &mut state,
            );
            paint_log_ticks(frame.buffer_mut(), log_area, &offsets);
        })
        .unwrap();
    (terminal.backend().buffer().clone(), log_area, offsets)
}

#[test]
fn line_mode_grabs_the_gap_from_the_header_to_the_last_row() {
    set(ColumnResize::Line);
    let (log_area, offsets) = log_geometry();
    let hits = log_header_hit_targets(log_area, &offsets);
    let header_row = log_area.y + 1;
    let last_row = log_area.y + log_area.height - 2;
    for (col, gap, next) in boundaries(&offsets) {
        // The header, the middle of the table and its last row: all the
        // boundary.
        for y in [header_row, header_row + 5, last_row] {
            assert_eq!(
                find_hit_target(&hits, gap, y),
                Some(HitTarget::LogColumnBoundary(col)),
                "{col:?} at row {y}"
            );
        }
        // Not the top border, not the bottom border.
        assert_eq!(find_hit_target(&hits, gap, log_area.y), None);
        assert_eq!(find_hit_target(&hits, gap, last_row + 1), None);
        // The first cell of the next column is no longer a grab: in the body
        // it is a commit row, on the header it reorders.
        assert_eq!(find_hit_target(&hits, next, header_row + 5), None);
        assert!(matches!(
            find_hit_target(&hits, next, header_row),
            Some(HitTarget::LogColumnHeader(_))
        ));
        assert_eq!(
            log_boundary_grab_rect(log_area, next),
            Rect::new(gap, header_row, 1, last_row + 1 - header_row)
        );
    }
}

#[test]
fn header_mode_grabs_only_the_first_cell_of_the_next_header() {
    set(ColumnResize::Header);
    let (log_area, offsets) = log_geometry();
    let hits = log_header_hit_targets(log_area, &offsets);
    let header_row = log_area.y + 1;
    for (col, gap, next) in boundaries(&offsets) {
        assert_eq!(
            find_hit_target(&hits, next, header_row),
            Some(HitTarget::LogColumnBoundary(col))
        );
        assert_eq!(find_hit_target(&hits, next, header_row + 3), None);
        assert!(matches!(
            find_hit_target(&hits, gap, header_row),
            Some(HitTarget::LogColumnHeader(_))
        ));
        assert_eq!(find_hit_target(&hits, gap, header_row + 3), None);
    }
}

#[test]
fn the_header_rests_with_a_tick_in_each_gap_in_line_mode_and_none_in_header_mode() {
    set(ColumnResize::Line);
    let (buf, log_area, offsets) = rendered(false);
    let header_row = log_area.y + 1;
    for (_, gap, _) in boundaries(&offsets) {
        assert_eq!(buf[(gap, header_row)].symbol(), "\u{2502}", "gap {gap}");
        // A complete separator at rest: the muted full-height glyph.
        assert_eq!(buf[(gap, header_row)].fg, crate::th().ui.muted, "gap {gap}");
        // The body shows no tick.
        assert_ne!(buf[(gap, header_row + 1)].symbol(), "\u{2502}");
    }
    set(ColumnResize::Header);
    let (buf, ..) = rendered(false);
    // (The panel's own border cells are not ticks.)
    for x in 1..log_area.width - 1 {
        assert_ne!(buf[(x, header_row)].symbol(), "\u{2502}", "x {x}");
    }
}

#[test]
fn hovering_the_gap_anywhere_draws_a_thin_line_over_the_whole_table_height() {
    set(ColumnResize::Line);
    let (log_area, offsets) = log_geometry();
    let hits = log_header_hit_targets(log_area, &offsets);
    let panes = Rect::new(0, 0, log_area.width, log_area.y + log_area.height);
    let header_row = log_area.y + 1;
    let last_row = log_area.y + log_area.height - 2;
    let (col, gap, _) = boundaries(&offsets)[1];
    for pointer_row in [header_row, header_row + 7, last_row] {
        let (grab, strip) =
            resize_grab_at(&hits, gap, pointer_row, panes, 300, 301, false, false).unwrap();
        assert_eq!(grab, ResizeGrab::LogColumn(col));
        let (mut buf, ..) = rendered(false);
        crate::column_resize::paint_grab(&mut buf, ColumnResize::Line, strip);
        for y in header_row..=last_row {
            assert_eq!(buf[(gap, y)].symbol(), "\u{2502}", "row {y}");
            assert_eq!(buf[(gap, y)].style().fg, Some(th().ui.accent));
            assert_ne!(buf[(gap, y)].style().bg, Some(th().ui.accent), "thin");
        }
        // Nothing beside it, nothing on the borders.
        for y in header_row..=last_row {
            assert_ne!(buf[(gap + 1, y)].symbol(), "\u{2502}");
            assert_ne!(buf[(gap - 1, y)].symbol(), "\u{2502}");
        }
        assert_ne!(buf[(gap, log_area.y)].symbol(), "\u{2502}");
    }
}

#[test]
fn header_mode_hover_lights_one_header_cell_as_a_block() {
    set(ColumnResize::Header);
    let (log_area, offsets) = log_geometry();
    let hits = log_header_hit_targets(log_area, &offsets);
    let panes = Rect::new(0, 0, log_area.width, log_area.y + log_area.height);
    let (_, _, next) = boundaries(&offsets)[1];
    let header_row = log_area.y + 1;
    let (_, strip) =
        resize_grab_at(&hits, next, header_row, panes, 300, 301, false, false).unwrap();
    assert_eq!(strip, Rect::new(next, header_row, 1, 1));
    let (mut buf, ..) = rendered(false);
    crate::column_resize::paint_grab(&mut buf, ColumnResize::Header, strip);
    assert_eq!(buf[(next, header_row)].style().bg, Some(th().ui.accent));
    assert_ne!(buf[(next, header_row + 1)].style().bg, Some(th().ui.accent));
}

/// The gap is a cell of the column on its left, not a hole: in a selected row
/// it carries the selection color like the cells around it, so the row reads
/// as one band whichever mode is set.
#[test]
fn the_selection_runs_through_every_text_gap() {
    for mode in [ColumnResize::Line, ColumnResize::Header] {
        set(mode);
        let (buf, log_area, offsets) = rendered(true);
        let y = log_top_y(log_area) + 1;
        let sel = th().ui.selection;
        for (col, gap, next) in boundaries(&offsets) {
            if matches!(col, LogColumn::Ref | LogColumn::Graph) {
                // Their cells keep their own colors (a pill, the lanes).
                continue;
            }
            assert_eq!(buf[(gap, y)].bg, sel, "{mode:?}: {col:?} gap");
            assert_eq!(buf[(next, y)].bg, sel, "{mode:?}: {col:?} next");
        }
    }
}

/// Dragging by the gap moves the edge the way a header-cell drag does: the
/// pointer on the gap is one cell before the edge it holds.
#[test]
fn a_drag_by_the_gap_moves_the_edge_to_the_pointer_plus_one() {
    set(ColumnResize::Line);
    let (_, offsets) = log_geometry();
    let (col, gap, next) = boundaries(&offsets)[3];
    let edge = gap + crate::column_resize::pointer_to_edge(ColumnResize::Line, true);
    assert_eq!(edge, next);
    let held = boundary_drag_widths(&offsets, col, edge).unwrap();
    let still = boundary_drag_widths(&offsets, col, next).unwrap();
    assert_eq!(held, still);
    // Three cells right of the gap is three cells wider.
    let wider = boundary_drag_widths(&offsets, col, edge + 3).unwrap();
    let left = offsets.iter().find(|(c, ..)| *c == col).unwrap();
    let new_left = wider.iter().find(|(c, _)| *c == col).map(|(_, w)| *w);
    assert_eq!(new_left, Some(left.2 + 3));
}

/// At rest the body is the same in both modes (the gap is the left column's
/// own last cell, so the pill end, the lane tint and the selection run
/// through it exactly as they do without a gap); only the header differs.
#[test]
fn at_rest_the_body_is_the_same_in_both_modes_and_only_the_header_differs() {
    set(ColumnResize::Line);
    let (line_buf, log_area, _) = rendered(true);
    set(ColumnResize::Header);
    let (header_buf, ..) = rendered(true);
    let header_row = log_area.y + 1;
    let mut header_diff = 0;
    for y in log_area.y..log_area.y + log_area.height {
        for x in 0..log_area.width {
            if line_buf[(x, y)] != header_buf[(x, y)] {
                assert_eq!(y, header_row, "cell ({x}, {y}) differs outside the header");
                header_diff += 1;
            }
        }
    }
    assert!(header_diff > 0, "line mode rests with ticks in the header");
}
