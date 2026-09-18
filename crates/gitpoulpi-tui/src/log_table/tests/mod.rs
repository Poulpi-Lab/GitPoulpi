//! Tests for the Log table (`log_table`).

use super::*;

/// The shipped default: every column but `Changes`, which is hidden until
/// the user turns it on.
pub(super) fn changes_off() -> std::collections::HashSet<LogColumn> {
    [LogColumn::Changes].into()
}

/// The default display order minus the hidden `Changes`.
pub(super) fn shown_order() -> Vec<LogColumn> {
    LogColumn::ALL
        .into_iter()
        .filter(|c| !changes_off().contains(c))
        .collect()
}

pub(super) fn sample_commit(n: u8, summary: &str) -> CommitNode {
    let mut bytes = [0u8; 20];
    bytes[0] = n;
    CommitNode {
        id: Oid(bytes),
        parents: Vec::new(),
        summary: summary.to_string(),
        message: summary.to_string(),
        author_name: "Tester".into(),
        author_email: "tester@example.com".into(),
        timestamp: 0,
        refs: Vec::new(),
    }
}

/// The pane geometry the resize tests share: a Log pane 80 wide with
/// its top border on row 3, so the header row is row 4.
pub(super) fn log_geometry() -> (Rect, Vec<(LogColumn, u16, u16)>) {
    let log_area = Rect::new(0, 3, 80, 20);
    let widths = LogColumnWidths::for_area_width(
        log_area.width,
        &changes_off(),
        &std::collections::HashMap::new(),
    );
    let order = shown_order();
    (log_area, column_x_offsets(log_area.x, &order, &widths))
}

/// One line of a buffer, as text.
pub(super) fn line(buf: &ratatui::buffer::Buffer, y: u16) -> String {
    (0..buf.area.width)
        .map(|x| buf[(x, y)].symbol().to_string())
        .collect()
}

mod author_cell;
mod boundaries;
mod column_resize;
mod columns;
mod connector;
mod geometry;
mod ghost;
mod i18n;
mod prefs;
mod render;
mod search;
mod select_band;
mod stash;
mod wip;
