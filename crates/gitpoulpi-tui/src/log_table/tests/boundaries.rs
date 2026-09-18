//! Column boundaries: grab cells, header hit targets and drags.

use super::*;
use super::{changes_off, log_geometry, sample_commit, shown_order};

// --- Log column resize: where you can grab, and what moves ---

/// Defect: the grab zone ran the pane's full height, so clicking a
/// commit anywhere near a column edge started a resize drag instead of
/// selecting that row.
#[test]
fn a_column_boundary_is_grabbable_on_the_header_row_only() {
    // The `header` mode (the first letter of the next name is the grab).
    crate::live_settings::install_column_resize(gitpoulpi_config::ColumnResize::Header);
    let (log_area, offsets) = log_geometry();
    let hits = log_header_hit_targets(log_area, &offsets);
    let header_row = log_area.y + 1;
    let boundary_x = log_boundary_x(&offsets, LogColumn::Graph).unwrap();

    assert_eq!(
        find_hit_target(&hits, boundary_x, header_row),
        Some(HitTarget::LogColumnBoundary(LogColumn::Graph))
    );
    // One row below the header is already table body: a commit row, and
    // nothing else. Same for the bottom of the pane.
    assert_eq!(find_hit_target(&hits, boundary_x, header_row + 1), None);
    assert_eq!(
        find_hit_target(&hits, boundary_x, log_area.y + log_area.height - 2),
        None
    );
    // The pane's top border carries the block title (and the search
    // bar overlay), not a resize zone.
    assert_eq!(find_hit_target(&hits, boundary_x, log_area.y), None);
}

/// The boundary strips are registered before the header cells because
/// `find_hit_target` takes the first match and a strip sits on the
/// seam between two header cells. Registered the other way round, the
/// grab cell answered "drag-to-reorder" instead.
///
/// One cell either side, the header cell keeps its click: there is no
/// grab margin any more, because a margin is a region that lights up
/// somewhere other than where the pointer is.
#[test]
fn the_grab_cell_wins_over_the_header_cell_it_sits_on() {
    // The `header` mode (the first letter of the next name is the grab).
    crate::live_settings::install_column_resize(gitpoulpi_config::ColumnResize::Header);
    let (log_area, offsets) = log_geometry();
    let hits = log_header_hit_targets(log_area, &offsets);
    let header_row = log_area.y + 1;
    let boundary_x = log_boundary_x(&offsets, LogColumn::Graph).unwrap();

    assert_eq!(
        find_hit_target(&hits, boundary_x, header_row),
        Some(HitTarget::LogColumnBoundary(LogColumn::Graph))
    );
    assert_eq!(
        find_hit_target(&hits, boundary_x + 1, header_row),
        Some(HitTarget::LogColumnHeader(LogColumn::Message))
    );
    assert_eq!(
        find_hit_target(&hits, boundary_x - 1, header_row),
        Some(HitTarget::LogColumnHeader(LogColumn::Graph))
    );
}

/// The rendered table and the hit targets have to agree on where a
/// column edge is, or the grab zones sit in the middle of columns.
/// ratatui's `Table` defaults to `column_spacing: 1`, which over-
/// subscribes a set of `Length`s that already fill the pane and makes
/// the solver shrink them: see `log_table`.
#[test]
fn the_rendered_columns_start_where_the_hit_targets_say_they_do() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let (log_area, offsets) = log_geometry();
    let widths = LogColumnWidths::for_area_width(
        log_area.width,
        &changes_off(),
        &std::collections::HashMap::new(),
    );
    let order = shown_order();
    let mut terminal = Terminal::new(TestBackend::new(
        log_area.width,
        log_area.y + log_area.height,
    ))
    .unwrap();
    let mut state = TableState::default();
    state.select(Some(0));
    terminal
        .draw(|frame| {
            let header: Vec<Cell> = order.iter().map(|&c| Cell::from(c.title())).collect();
            frame.render_stateful_clipped(
                log_table(
                    vec![Row::new(vec![Cell::from("x")])],
                    header,
                    &order,
                    &widths,
                    Style::default(),
                ),
                log_area,
                &mut state,
            );
        })
        .unwrap();

    // Per *cell*, not per byte: the pane's border glyph is multi-byte,
    // so a byte offset into the rendered line is not a column.
    let buf = terminal.backend().buffer().clone();
    let header_row = log_area.y + 1;
    let cells: Vec<String> = (0..log_area.width)
        .map(|x| buf[(x, header_row)].symbol().to_string())
        .collect();
    let rendered = cells.concat();
    for &(col, x, _) in &offsets {
        let title = col.title();
        let at: String = cells[x as usize..]
            .iter()
            .take(title.chars().count())
            .cloned()
            .collect::<Vec<_>>()
            .concat();
        assert_eq!(
            at, title,
            "{title} is hit-tested at column {x}, which renders [{at}]: [{rendered}]"
        );
    }
}

/// A rendered commit row must not let one column's text touch the
/// next column's: the columns tile the pane exactly, so the gutter has
/// to come out of the text's own width.
#[test]
fn a_rendered_row_keeps_a_gutter_between_its_columns() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let (log_area, offsets) = log_geometry();
    let widths = LogColumnWidths::for_area_width(
        log_area.width,
        &changes_off(),
        &std::collections::HashMap::new(),
    );
    let order = shown_order();
    let mut commit = sample_commit(1, &"a summary far too long for the column".repeat(3));
    commit.author_name = "A Very Long Author Name".into();
    let mut terminal = Terminal::new(TestBackend::new(
        log_area.width,
        log_area.y + log_area.height,
    ))
    .unwrap();
    terminal
        .draw(|frame| {
            let header: Vec<Cell> = order.iter().map(|&c| Cell::from(c.title())).collect();
            let row = commit_row(
                &commit,
                &[],
                Line::from(""),
                Color::Reset,
                &widths,
                &order,
                false,
                false,
                None,
                false,
                avatar::LogAvatarMode::Off,
                gitpoulpi_config::AvatarFallback::Initials,
                None,
            );
            frame.render_stateful_clipped(
                log_table(vec![row], header, &order, &widths, Style::default()),
                log_area,
                &mut TableState::default(),
            );
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();
    let data_row = log_area.y + 2;
    for &(col, x, w) in &offsets {
        // The last visible column's right edge is the pane's border,
        // which is a gutter of its own; `Ref` and `Graph` draw their
        // own spacing.
        if matches!(col, LogColumn::Sha | LogColumn::Ref | LogColumn::Graph) {
            continue;
        }
        let last = buf[(x + w - 1, data_row)].symbol().to_string();
        assert_eq!(
            last,
            " ",
            "{col:?} runs into the next column: [{}]",
            (x..x + w)
                .map(|c| buf[(c, data_row)].symbol().to_string())
                .collect::<Vec<_>>()
                .concat()
        );
    }
    // ... and the date is still printed in full, gutter and all.
    let (_, date_x, date_w) = *offsets
        .iter()
        .find(|(c, _, _)| *c == LogColumn::Date)
        .unwrap();
    let date: String = (date_x..date_x + date_w)
        .map(|c| buf[(c, data_row)].symbol().to_string())
        .collect::<Vec<_>>()
        .concat();
    assert_eq!(date.trim_end(), format_timestamp(commit.timestamp));
}

/// Dragging a boundary must move *that* boundary. Overriding only the
/// column to its left cannot: `Message` absorbs the leftover, so the
/// right edge of every column after it is pinned by the columns that
/// follow, and widening `Author` merely stole width from `Message` and
/// slid `Author` leftwards - the edge under the pointer never budged.
#[test]
fn dragging_a_boundary_moves_that_boundary_even_past_the_elastic_column() {
    let (log_area, _) = log_geometry();
    let order = shown_order();
    let hidden = changes_off();

    for (left, delta) in [
        (LogColumn::Ref, 2i16),
        (LogColumn::Graph, -2),
        (LogColumn::Message, 2),
        (LogColumn::Author, 4),
        (LogColumn::Date, -3),
    ] {
        let mut overrides: std::collections::HashMap<LogColumn, u16> =
            std::collections::HashMap::new();
        let widths = LogColumnWidths::for_area_width(log_area.width, &hidden, &overrides);
        let offsets = column_x_offsets(log_area.x, &order, &widths);
        let from = log_boundary_x(&offsets, left).unwrap();
        let pointer = (from as i16 + delta) as u16;

        for (c, w) in boundary_drag_widths(&offsets, left, pointer).unwrap() {
            overrides.insert(c, w);
        }

        let widths = LogColumnWidths::for_area_width(log_area.width, &hidden, &overrides);
        let offsets = column_x_offsets(log_area.x, &order, &widths);
        assert_eq!(
            log_boundary_x(&offsets, left),
            Some(pointer),
            "{left:?}'s boundary stayed at {from} instead of following the pointer to {pointer}"
        );
        // Nothing outside the dragged pair moved, and the widths still
        // tile the pane exactly.
        assert_eq!(
            offsets.iter().map(|(_, _, w)| *w).sum::<u16>(),
            log_area.width - 2
        );
    }
}

/// Neither neighbor can be crushed: the boundary stops at the floor
/// instead of letting a column reach zero width.
#[test]
fn a_boundary_drag_stops_at_both_neighbors_minimum_widths() {
    let (_, offsets) = log_geometry();
    let pair = boundary_drag_widths(&offsets, LogColumn::Author, 0).unwrap();
    assert_eq!(pair[0].1, LogColumn::Author.min_width());
    let pair = boundary_drag_widths(&offsets, LogColumn::Author, 10_000).unwrap();
    assert_eq!(pair[1].1, LogColumn::Date.min_width());
    // The last visible column's right edge is the pane border, not a
    // draggable boundary.
    assert!(boundary_drag_widths(&offsets, LogColumn::Sha, 40).is_none());
    assert_eq!(log_boundary_x(&offsets, LogColumn::Sha), None);
}

/// The highlight is drawn at the boundary, so it follows a drag; drawn
/// at the pointer's own column it lagged, and mid-drag it froze at the
/// column the drag started from (a held button reports as `Drag`, never
/// `Moved`).
#[test]
fn the_highlight_follows_the_boundary_through_a_drag() {
    let (log_area, offsets) = log_geometry();
    let order = shown_order();
    let mut overrides: std::collections::HashMap<LogColumn, u16> = std::collections::HashMap::new();

    let start = log_boundary_x(&offsets, LogColumn::Author).unwrap();
    // The drag handler's own arithmetic: the same rect builder the
    // hover predicate uses, at the edge's column rather than the
    // pointer's.
    let strip = |bx: Option<u16>| bx.map(|bx| log_boundary_grab_rect(log_area, bx));
    assert_eq!(
        strip(log_boundary_x(&offsets, LogColumn::Author)),
        Some(log_boundary_grab_rect(log_area, start))
    );

    // Drag six cells right, one event at a time, as the mouse handler
    // does: the strip tracks the edge on every step.
    for pointer in start + 1..=start + 6 {
        let widths = LogColumnWidths::for_area_width(log_area.width, &changes_off(), &overrides);
        let offsets = column_x_offsets(log_area.x, &order, &widths);
        for (c, w) in boundary_drag_widths(&offsets, LogColumn::Author, pointer).unwrap() {
            overrides.insert(c, w);
        }
        let widths = LogColumnWidths::for_area_width(log_area.width, &changes_off(), &overrides);
        let offsets = column_x_offsets(log_area.x, &order, &widths);
        assert_eq!(
            strip(log_boundary_x(&offsets, LogColumn::Author)),
            Some(log_boundary_grab_rect(log_area, pointer)),
            "the highlight stayed behind at {start} while the boundary moved to {pointer}"
        );
    }

    // No boundary at all (the last visible column's edge is the pane's
    // own border): nothing lit, nothing left painted at the old edge.
    assert_eq!(strip(None), None);
}
