//! The Log row arithmetic: window, visible rows, paging, index <-> screen row.

use super::*;
use super::{changes_off, shown_order};

/// `log_visible_rows` and `log_row_to_index` must agree on which rows
/// the Log can show: the lazy loader uses the first to decide "how
/// much is still below the window", and a click uses the second to
/// decide which commit it landed on. Two hand-counted constants for
/// the same band is exactly the drift `where-we-are.md` names as this
/// file's repeat offender.
///
/// Falsified by moving `log_visible_rows`' bound by one row
/// (`area.height - 4`), which is what any hand-counted rewrite of it
/// would get wrong: the loader then fetches a page a row early or a
/// row late at every terminal size.
///
/// Note that `area.height - 3` is *not* a falsification - it is the
/// same function, since the `y` in `(y + height - 1) - (y + 2)`
/// cancels. The sweep over `y` is here because a reader will assume
/// otherwise, not because it separates those two.
#[test]
fn log_window_is_exactly_the_rows_ratatui_would_draw_and_its_offset() {
    // Row for row: render a real `Table` over every row (the old
    // path) and over just the window (the new one) and compare the
    // buffers and the settled offset.
    for total in [0usize, 1, 5, 12] {
        // 0 is skipped: a zero-height area never reaches the offset
        // logic (`Table` returns early); `draw_log` handles it.
        for visible in 1..8usize {
            for offset in 0..14usize {
                for selected in std::iter::once(None).chain((0..14).map(Some)) {
                    let area = Rect::new(0, 0, 8, visible as u16);
                    let table = |rows: std::ops::Range<usize>| {
                        Table::new(
                            rows.map(|i| Row::new(vec![Cell::from(format!("r{i}"))]))
                                .collect::<Vec<_>>(),
                            [Constraint::Length(8)],
                        )
                        .row_highlight_style(Style::default().add_modifier(Modifier::BOLD))
                    };
                    let mut full_state = TableState::default();
                    *full_state.selected_mut() = selected;
                    *full_state.offset_mut() = offset;
                    let mut full = ratatui::buffer::Buffer::empty(area);
                    StatefulWidget::render(table(0..total), area, &mut full, &mut full_state);

                    let w = log_window(offset, selected, visible, total);
                    let mut state = TableState::default();
                    *state.selected_mut() = w.selected;
                    let mut windowed = ratatui::buffer::Buffer::empty(area);
                    // The window's rows carry their absolute labels.
                    StatefulWidget::render(table(w.start..w.end), area, &mut windowed, &mut state);
                    let ctx =
                        format!("total={total} visible={visible} offset={offset} sel={selected:?}");
                    if total > 0 {
                        assert_eq!(w.start, full_state.offset(), "{ctx}");
                    }
                    assert_eq!(windowed, full, "{ctx}");
                }
            }
        }
    }
}

#[test]
fn log_visible_rows_matches_the_band_log_row_to_index_accepts() {
    // `y` is swept as well as `height`. At `y == 0` the correct
    // formula and a naive `height - 3` agree exactly, so a sweep of
    // one origin cannot tell them apart -- and the Log pane never
    // sits at the top of the window anyway: it is under the tab bar
    // and the toolbar.
    for y in 0..5u16 {
        for height in 3..24u16 {
            let area = Rect::new(0, y, 40, height);
            let accepted = (0..y + height + 2)
                .filter(|&row| log_row_to_index(area, 0, row, 1000).is_some())
                .count();
            assert_eq!(
                log_visible_rows(area),
                accepted,
                "height {height}: the row count the loader works from and the rows a \
             click resolves must be the same band"
            );
        }
    }
}

/// `PageUp`/`PageDown` move the Log's selection by exactly one visible
/// screen - "page down/up c'est monter/descendre d'un écran
/// visible" - and clamp at the ends rather than wrapping, the same
/// as every other jump command in this app (`Home`/`End`/`PageUp`/
/// `PageDown` in the repository overlay clamp too, per its own doc
/// comment).
#[test]
fn page_delta_moves_the_log_selection_by_one_visible_screen_and_clamps() {
    let area = Rect::new(0, 3, 40, 22);
    let page = log_page_delta(area);
    assert_eq!(page, log_visible_rows(area) as i32);
    assert!(
        page > 1,
        "the fixture must be tall enough for this to mean something"
    );

    let len = (page as usize) * 5;
    let mut state = TableState::default();
    state.select(Some(0));

    move_table_selection(&mut state, len, page);
    assert_eq!(state.selected(), Some(page as usize));

    move_table_selection(&mut state, len, -page);
    assert_eq!(state.selected(), Some(0));

    // Clamped, not wrapped: paging up from the top stays at the top,
    // and paging down from near the bottom stops at the last row
    // rather than overshooting past `len`.
    move_table_selection(&mut state, len, -page);
    assert_eq!(state.selected(), Some(0));

    state.select(Some(len - 1));
    move_table_selection(&mut state, len, page);
    assert_eq!(state.selected(), Some(len - 1));
}

/// A pane too short to show even one full row still pages by one row,
/// rather than becoming a dead key (`log_page_delta` floors at 1).
/// Falsified by removing the `.max(1)`: at this height
/// `log_visible_rows` is 0 and the selection would never move.
#[test]
fn page_delta_is_never_zero_even_in_a_pane_too_short_to_show_a_row() {
    let area = Rect::new(0, 3, 40, 2);
    assert_eq!(log_visible_rows(area), 0);
    assert_eq!(log_page_delta(area), 1);
}

/// Lazy loading appends rows *under* the ones on screen, so every row
/// a click could already resolve has to keep resolving to the same
/// commit. This is the Log's row arithmetic - the pinned WIP row, the
/// header, `row_to_index`'s `top_offset` - which `where-we-are.md`
/// records as having produced real bugs more than once, and an append
/// is a new way to disturb it.
///
/// Falsified by making `log_row_to_index` derive its shift from `len`
/// (`rel + offset + len % 3`): every index then moves when the list
/// grows, and this names the first row that did. The two lengths
/// below are deliberately incongruent mod 2 and mod 3, so a
/// length-dependent shift cannot cancel itself out between them.
#[test]
fn appending_a_page_does_not_move_the_row_a_click_resolves_to() {
    let area = Rect::new(0, 0, 40, 20);
    for offset in [0usize, 1, 5, 37] {
        for row in 0..24u16 {
            let before = log_row_to_index(area, offset, row, 41);
            let after = log_row_to_index(area, offset, row, 93);
            if let Some(index) = before {
                assert_eq!(
                    after,
                    Some(index),
                    "offset={offset} row={row}: appending a page moved \
                     the commit this row resolves to"
                );
            }
        }
    }
}

/// **The WIP row is one line, like every commit.** It used to be two,
/// and both of its lines resolved to it; now the line right under it
/// is the first commit.
#[test]
fn the_wip_row_is_one_line_and_the_first_commit_is_right_under_it() {
    // Log pane at y = 0: border on row 0, header on row 1, content
    // from row 2.
    let area = Rect::new(0, 0, 40, 20);
    let len = 4; // WIP + 3 commits
    assert_eq!(log_row_to_index(area, 0, 2, len), Some(0), "the WIP row");
    assert_eq!(
        log_row_to_index(area, 0, 3, len),
        Some(1),
        "the first commit"
    );
    assert_eq!(log_row_to_index(area, 0, 4, len), Some(2));
    // Scrolled: plain offset arithmetic.
    assert_eq!(log_row_to_index(area, 1, 2, len), Some(1));
    // Border and header rows are not selectable, nor is past the end.
    assert_eq!(log_row_to_index(area, 0, 0, len), None);
    assert_eq!(log_row_to_index(area, 0, 1, len), None);
    assert_eq!(log_row_to_index(area, 0, 6, len), None);
}

#[test]
fn log_index_to_row_round_trips_with_log_row_to_index() {
    let area = Rect::new(0, 0, 40, 20);
    let top = log_top_y(area);
    let len = 6;
    for offset in 0..3usize {
        for index in offset..len {
            let row =
                log_index_to_row(top, offset, index).expect("a visible index must map to a row");
            assert_eq!(
                log_row_to_index(area, offset, row, len),
                Some(index),
                "offset={offset} index={index} row={row}"
            );
            // One line per row, WIP row included.
            assert_eq!(row, top + (index - offset) as u16);
        }
    }
    // Rows scrolled above the viewport have no screen position.
    assert_eq!(log_index_to_row(top, 2, 1), None);
}

/// Through the **real Log `Table` widget**: a
/// wheel-frozen offset (`table_render_state(.., follow: false, ..)`)
/// must survive `Table::render` even though `selected` is scrolled
/// off screen - ratatui's own `get_row_bounds` snaps the offset back
/// to the selection whenever `selected` is `Some` and outside the
/// current window, which is exactly the drag-back the wheel
/// scrolling must avoid. `ui_kit`'s unit tests already prove this at the
/// `TableState` level; this one proves the actual widget this crate
/// renders agrees.
#[test]
fn a_wheel_frozen_offset_survives_rendering_through_the_real_log_table() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let log_area = Rect::new(0, 0, 40, 8); // header+borders -> 5 content rows
    let visible = log_visible_rows(log_area);
    assert!(visible >= 5, "the test area must fit at least 5 rows");
    let total = visible + 10; // far more rows than fit
    let order = shown_order();
    let widths = LogColumnWidths::for_area_width(
        log_area.width,
        &changes_off(),
        &std::collections::HashMap::new(),
    );
    let rows: Vec<Row> = (0..total)
        .map(|i| {
            Row::new(order.iter().map(|&c| {
                if c == LogColumn::Message {
                    Cell::from(format!("commit {i}"))
                } else {
                    Cell::from("")
                }
            }))
        })
        .collect();

    // Selection sits on row 0 (scrolled out of view); the wheel has
    // already moved the persisted offset down to `visible`, and
    // marked the list as no longer following.
    let mut state = TableState::default();
    state.select(Some(0));
    *state.offset_mut() = visible;
    let follow = false;

    let mut terminal = Terminal::new(TestBackend::new(log_area.width, log_area.height)).unwrap();
    terminal
        .draw(|frame| {
            let header: Vec<Cell> = order.iter().map(|&c| Cell::from(c.title())).collect();
            let mut render_state = table_render_state(&state, follow, visible);
            frame.render_stateful_clipped(
                log_table(rows, header, &order, &widths, Style::default()),
                log_area,
                &mut render_state,
            );
            *state.offset_mut() = render_state.offset();
        })
        .unwrap();

    assert_eq!(
        state.offset(),
        visible,
        "rendering must not drag the frozen offset back to the \
         off-screen selection"
    );
    assert_eq!(
        state.selected(),
        Some(0),
        "the real selection itself must never be touched by rendering"
    );
    // And the row actually drawn at the top of the body must be the
    // one the frozen offset names (`visible`), not commit 0.
    let top = log_top_y(log_area);
    let buf = terminal.backend().buffer();
    let text: String = (0..log_area.width)
        .map(|x| buf[(x, top)].symbol().to_string())
        .collect();
    assert!(
        text.contains(&format!("commit {visible}")),
        "expected the frozen offset's first row, got: {text:?}"
    );
}
