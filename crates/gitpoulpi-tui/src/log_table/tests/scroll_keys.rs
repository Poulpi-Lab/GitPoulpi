//! The pure parts of the sideways scroll: the offset arithmetic, keeping a
//! picked column in view, the scroll resize and the chooser's layout row.

use super::*;
use gitpoulpi_config::LogColumnsLayout;

fn geo(h: u16) -> LogColumnsGeo {
    let scroll: std::collections::HashMap<LogColumn, u16> = [
        (LogColumn::Ref, 20),
        (LogColumn::Graph, 10),
        (LogColumn::Message, 50),
        (LogColumn::Author, 20),
        (LogColumn::Date, 20),
        (LogColumn::Sha, 10),
    ]
    .into();
    LogColumnsGeo::compute(
        Rect::new(0, 0, 62, 10),
        &LogColumn::ALL,
        &[LogColumn::Changes].into(),
        &LogWidthMaps {
            fit: &Default::default(),
            scroll: &scroll,
        },
        (LogColumnsLayout::Scroll, h),
    )
}

#[test]
fn scrolling_stays_inside_the_content() {
    let g = geo(0);
    assert_eq!(scrolled(&g, -4), 0);
    assert_eq!(scrolled(&g, 4), 4);
    assert_eq!(scrolled(&g, 9999), g.max_h_offset());
}

#[test]
fn a_picked_column_is_brought_into_view_with_the_least_movement() {
    let g = geo(0);
    // Already in view: no movement. Past the right edge: its end meets it.
    assert_eq!(offset_to_show(&g, LogColumn::Ref), 0);
    let (x, w) = g.span_of(LogColumn::Date).unwrap();
    assert_eq!(offset_to_show(&g, LogColumn::Date), x + w - g.inner.width);
    // Cut at the left: its start meets the left edge.
    let far = geo(g.max_h_offset());
    let (x, _) = far.span_of(LogColumn::Ref).unwrap();
    assert_eq!(offset_to_show(&far, LogColumn::Ref), x);
    // Wider than the viewport: its start.
    let wide: std::collections::HashMap<LogColumn, u16> = [(LogColumn::Message, 200)].into();
    let w = LogColumnsGeo::compute(
        Rect::new(0, 0, 62, 10),
        &LogColumn::ALL,
        &[LogColumn::Changes].into(),
        &LogWidthMaps {
            fit: &Default::default(),
            scroll: &wide,
        },
        (LogColumnsLayout::Scroll, 0),
    );
    let (x, _) = w.span_of(LogColumn::Message).unwrap();
    assert_eq!(
        offset_to_show(&w, LogColumn::Message),
        x.min(w.max_h_offset())
    );
}

#[test]
fn a_scroll_resize_changes_one_column_within_its_limits() {
    let g = geo(0);
    assert_eq!(scroll_resized(&g, LogColumn::Author, 2), 22);
    assert_eq!(scroll_resized(&g, LogColumn::Author, -2), 18);
    assert_eq!(
        scroll_resized(&g, LogColumn::Author, -9999),
        LogColumn::Author.drag_min_width()
    );
    assert_eq!(
        scroll_resized(&g, LogColumn::Author, 9999),
        MAX_COLUMN_CELLS
    );
}

#[test]
fn the_chooser_always_has_the_layout_row_and_message_fits_only_when_scrolling() {
    let rows = chooser_rows(Some(LogColumn::Message));
    assert!(rows.contains(&ChooserRow::Layout));
    let fit_row = ChooserRow::FitColumn(LogColumn::Message);
    assert!(!fit_row.actionable(LogColumnsLayout::Fit));
    assert!(fit_row.actionable(LogColumnsLayout::Scroll));
    assert!(
        !ChooserRow::Layout.closes(),
        "a checkbox keeps the chooser open"
    );
    // The cursor steps over the separator and the dimmed Message fit row.
    let at = rows
        .iter()
        .position(|r| *r == ChooserRow::Column(LogColumn::Sha))
        .unwrap();
    let next = step_chooser(&rows, LogColumnsLayout::Fit, at, true);
    assert_eq!(rows[next], ChooserRow::Layout);
    let after = step_chooser(&rows, LogColumnsLayout::Fit, next, true);
    assert_eq!(
        rows[after],
        ChooserRow::FitAll,
        "the dimmed fit row skipped"
    );
}
