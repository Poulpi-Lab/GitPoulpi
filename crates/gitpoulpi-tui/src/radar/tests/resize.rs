//! The Radar's header row and `ui.column_resize`: where the list starts, what
//! each mode grabs, what rests in the header and what a hover draws.

use super::*;
use crate::radar::Cell;
use crossterm::event::MouseEvent;
use gitpoulpi_config::ColumnResize;

fn set(mode: ColumnResize) {
    crate::live_settings::install_column_resize(mode);
}

fn mouse(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn divider(ui: &RadarUi, cell: Cell) -> Rect {
    ui.hits
        .iter()
        .find(|(_, h)| *h == Hit::Divider(cell))
        .map(|(r, _)| *r)
        .unwrap_or_else(|| panic!("no {cell:?} divider in {:?}", ui.hits))
}

fn buffer(ui: &mut RadarUi, w: u16, h: u16) -> ratatui::buffer::Buffer {
    let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
    terminal
        .draw(|f| {
            let area = f.area();
            ui.render(f, area);
        })
        .unwrap();
    terminal.backend().buffer().clone()
}

fn row_text(buf: &ratatui::buffer::Buffer, y: u16) -> String {
    (0..buf.area.width)
        .map(|x| buf[(x, y)].symbol().to_string())
        .collect()
}

/// A Radar with enough requests to scroll.
fn long_list(dir: &std::path::Path) -> RadarUi {
    let (mut ui, app, _) = radar(dir);
    ui.apply_answer(answer(
        &app,
        "acme/app",
        (100..140)
            .map(|n| pr(n, "kai", "2026-10-09T10:00:00Z"))
            .collect(),
        vec![],
    ));
    ui.rebuild();
    ui
}

#[test]
fn a_header_row_names_the_columns_and_the_list_starts_under_it() {
    set(ColumnResize::Line);
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, ..) = radar(dir.path());
    let buf = buffer(&mut ui, 120, 30);
    let header = row_text(&buf, 5);
    for name in ["Repository", "Title", "Author", "Age"] {
        assert!(header.contains(name), "{name} in {header:?}");
    }
    // The first list row is directly under it.
    assert!(row_text(&buf, 6).contains("MY PULL REQUESTS"));
    // The window is what is left of the body: six rows above, one below.
    assert_eq!(ui.list_height, 30 - 6 - 1);
    // The first row's band starts under the header.
    let first = ui
        .hits
        .iter()
        .filter_map(|(r, h)| matches!(h, Hit::Heading(_)).then_some(r.y))
        .min()
        .unwrap();
    assert_eq!(first, 6);
}

#[test]
fn selection_scroll_and_clicks_agree_with_the_rows_under_the_header() {
    set(ColumnResize::Line);
    let dir = tempfile::tempdir().unwrap();
    let mut ui = long_list(dir.path());
    let last_item = ui
        .rows
        .iter()
        .rposition(|r| matches!(r, Row::Item(_)))
        .unwrap();
    ui.selected = last_item;
    ui.follow = true;
    buffer(&mut ui, 120, 20);
    let rows: Vec<(Rect, usize)> = ui
        .hits
        .iter()
        .filter_map(|(r, h)| match h {
            Hit::Row(i) => Some((*r, *i)),
            _ => None,
        })
        .collect();
    assert_eq!(ui.list_height, 13);
    let (sel_rect, _) = rows.iter().find(|(_, i)| *i == last_item).unwrap();
    assert!(
        sel_rect.y >= 6 && sel_rect.y < 6 + 13,
        "the selected row scrolled into the window: {sel_rect:?}"
    );
    // The row hit rects are the rows the window paints: one per line from the
    // header's bottom, and a click on one selects exactly that row.
    let (rect, index) = rows.iter().min_by_key(|(r, _)| r.y).copied().unwrap();
    assert!(rect.y >= 6);
    ui.on_mouse(
        mouse(MouseEventKind::Down(MouseButton::Left), rect.x + 30, rect.y),
        false,
    );
    assert_eq!(ui.selected, index);
    // A press on the header row selects nothing.
    ui.selected = last_item;
    ui.on_mouse(
        mouse(MouseEventKind::Down(MouseButton::Left), rect.x + 30, 5),
        false,
    );
    assert_eq!(ui.selected, last_item);
    // Paging moves by the window the list really has.
    ui.selected = 1;
    ui.follow = true;
    ui.on_key(key(KeyCode::PageDown));
    assert!(ui.selected >= 1 + 13 - 2, "{}", ui.selected);
}

#[test]
fn line_mode_grabs_the_gap_down_the_list_and_ticks_the_header() {
    set(ColumnResize::Line);
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, ..) = radar(dir.path());
    let buf = buffer(&mut ui, 120, 30);
    for cell in [Cell::Repo, Cell::Num, Cell::Author] {
        let rect = divider(&ui, cell);
        assert_eq!(rect.y, 5, "{cell:?} starts at the header");
        assert_eq!(
            buf[(rect.x, 5)].style().fg,
            Some(crate::th().ui.muted),
            "{cell:?} rests muted"
        );
        assert_eq!(rect.bottom(), 6 + ui.list_height as u16, "{cell:?}");
        assert_eq!(rect.width, 1);
        assert_eq!(buf[(rect.x, 5)].symbol(), "\u{2502}", "{cell:?} tick");
        // At rest the list shows the gap blank, whatever the row.
        for y in 6..12 {
            assert_ne!(buf[(rect.x, y)].symbol(), "\u{2502}", "{cell:?} row {y}");
        }
    }
    // Hovering the gap anywhere draws the thin line, header to last row.
    let rect = divider(&ui, Cell::Repo);
    ui.on_mouse(mouse(MouseEventKind::Moved, rect.x, 14), false);
    assert!(ui.resize_pointer(), "the resize pointer over the gap");
    let buf = buffer(&mut ui, 120, 30);
    for y in rect.y..rect.bottom() {
        assert_eq!(buf[(rect.x, y)].symbol(), "\u{2502}", "row {y}");
        assert_eq!(buf[(rect.x, y)].style().fg, Some(crate::th().ui.accent));
    }
    // Off it: the default pointer, and the line is gone.
    ui.on_mouse(mouse(MouseEventKind::Moved, rect.x + 8, 14), false);
    assert!(!ui.resize_pointer());
    let buf = buffer(&mut ui, 120, 30);
    assert_ne!(buf[(rect.x, 14)].symbol(), "\u{2502}");
}

#[test]
fn line_mode_drag_starts_from_the_header_or_from_mid_height() {
    set(ColumnResize::Line);
    for start_row in [5u16, 14] {
        let dir = tempfile::tempdir().unwrap();
        let (mut ui, ..) = radar(dir.path());
        ui.state.columns.repo = Some(8);
        buffer(&mut ui, 120, 30);
        let rect = divider(&ui, Cell::Repo);
        ui.on_mouse(
            mouse(MouseEventKind::Down(MouseButton::Left), rect.x, start_row),
            false,
        );
        assert!(ui.resize_pointer(), "dragging keeps the resize pointer");
        ui.on_mouse(
            mouse(MouseEventKind::Drag(MouseButton::Left), rect.x + 5, 20),
            false,
        );
        ui.on_mouse(
            mouse(MouseEventKind::Up(MouseButton::Left), rect.x + 5, 20),
            false,
        );
        assert_eq!(ui.state.columns.repo, Some(13), "from row {start_row}");
    }
}

#[test]
fn header_mode_has_no_tick_and_grabs_the_first_cell_of_the_next_header_only() {
    set(ColumnResize::Header);
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, ..) = radar(dir.path());
    ui.state.columns.repo = Some(8);
    let buf = buffer(&mut ui, 120, 30);
    for x in 0..120 {
        assert_ne!(buf[(x, 5)].symbol(), "\u{2502}", "no tick at {x}");
    }
    let rect = divider(&ui, Cell::Repo);
    assert_eq!((rect.y, rect.height), (5, 1), "the header cell only");
    // The first cell after the separator: the next column's.
    let repo_end = 5 + 8;
    assert_eq!(rect.x, repo_end + 1);
    // Mid-height on the gap grabs nothing; the header cell does.
    ui.on_mouse(mouse(MouseEventKind::Moved, repo_end, 14), false);
    assert!(!ui.resize_pointer());
    ui.on_mouse(
        mouse(MouseEventKind::Down(MouseButton::Left), repo_end, 14),
        false,
    );
    assert!(ui.col_drag.is_none(), "no drag from the body");
    ui.on_mouse(mouse(MouseEventKind::Moved, rect.x, 5), false);
    assert!(ui.resize_pointer());
    let buf = buffer(&mut ui, 120, 30);
    assert_eq!(buf[(rect.x, 5)].style().bg, Some(crate::th().ui.accent));
    assert_ne!(buf[(rect.x, 6)].style().bg, Some(crate::th().ui.accent));
    ui.on_mouse(
        mouse(MouseEventKind::Down(MouseButton::Left), rect.x, 5),
        false,
    );
    ui.on_mouse(
        mouse(MouseEventKind::Drag(MouseButton::Left), rect.x + 4, 5),
        false,
    );
    ui.on_mouse(
        mouse(MouseEventKind::Up(MouseButton::Left), rect.x + 4, 5),
        false,
    );
    assert_eq!(ui.state.columns.repo, Some(12));
}

/// A selected row is one band: the gap between two columns carries the
/// selection color in both modes, so no hole shows at rest.
#[test]
fn the_selection_runs_through_the_gaps() {
    for mode in [ColumnResize::Line, ColumnResize::Header] {
        set(mode);
        let dir = tempfile::tempdir().unwrap();
        let (mut ui, ..) = radar(dir.path());
        select(&mut ui, "Change 1");
        let buf = buffer(&mut ui, 120, 30);
        let selected = ui.selected;
        let y = ui
            .hits
            .iter()
            .find_map(|(r, h)| (*h == Hit::Row(selected)).then_some(r.y))
            .unwrap();
        let sel = crate::th().ui.selection;
        for cell in [Cell::Repo, Cell::Author] {
            let gap = match mode {
                ColumnResize::Line => divider(&ui, cell).x,
                ColumnResize::Header => divider(&ui, cell).x - 1,
            };
            assert_eq!(buf[(gap, y)].bg, sel, "{mode:?} {cell:?}");
        }
    }
}

fn down_drag_up(ui: &mut RadarUi, rect: Rect, to: (u16, u16)) {
    ui.on_mouse(
        mouse(MouseEventKind::Down(MouseButton::Left), rect.x, 14),
        false,
    );
    ui.on_mouse(
        mouse(MouseEventKind::Drag(MouseButton::Left), to.0, to.1),
        false,
    );
    ui.on_mouse(
        mouse(MouseEventKind::Up(MouseButton::Left), to.0, to.1),
        false,
    );
}

fn line_cells(buf: &ratatui::buffer::Buffer, rect: Rect) -> usize {
    (rect.y..rect.bottom())
        .filter(|y| buf[(rect.x, *y)].style().fg == Some(crate::th().ui.accent))
        .count()
}

/// A drag ended away from the boundary leaves no line and no resize pointer,
/// for every sizable separator and both modes; the mouse never leaves a
/// column picked. Red when the press picks the column or the release does not
/// re-read the hover.
#[test]
fn a_drag_released_away_from_its_separator_leaves_nothing_lit() {
    for mode in [ColumnResize::Line, ColumnResize::Header] {
        set(mode);
        for cell in [Cell::Repo, Cell::Num, Cell::Author, Cell::Age] {
            let dir = tempfile::tempdir().unwrap();
            let (mut ui, ..) = radar(dir.path());
            buffer(&mut ui, 120, 30);
            let rect = divider(&ui, cell);
            let start = if mode == ColumnResize::Header { 5 } else { 14 };
            ui.on_mouse(mouse(MouseEventKind::Moved, rect.x, start), false);
            assert!(ui.resize_pointer(), "{cell:?} {mode:?} hover");
            ui.on_mouse(
                mouse(MouseEventKind::Down(MouseButton::Left), rect.x, start),
                false,
            );
            assert!(ui.sizing_column());
            ui.on_mouse(
                mouse(MouseEventKind::Drag(MouseButton::Left), rect.x + 20, 20),
                false,
            );
            ui.on_mouse(
                mouse(MouseEventKind::Up(MouseButton::Left), rect.x + 20, 20),
                false,
            );
            assert!(!ui.sizing_column());
            assert_eq!(ui.col_pick, None, "{cell:?} {mode:?}: a drag picks nothing");
            assert!(!ui.resize_pointer(), "{cell:?} {mode:?}: default pointer");
            let rect = divider(&ui, cell);
            let buf = buffer(&mut ui, 120, 30);
            assert_eq!(line_cells(&buf, rect), 0, "{cell:?} {mode:?}: no line");
        }
    }
}

/// Released exactly on the boundary: it shows as hover, and a move away takes
/// it down.
#[test]
fn a_drag_released_on_the_separator_hovers_until_the_pointer_leaves() {
    set(ColumnResize::Line);
    for cell in [Cell::Repo, Cell::Num, Cell::Author, Cell::Age] {
        let dir = tempfile::tempdir().unwrap();
        let (mut ui, ..) = radar(dir.path());
        buffer(&mut ui, 120, 30);
        let rect = divider(&ui, cell);
        // No movement: the release is on the (unmoved) boundary.
        ui.on_mouse(
            mouse(MouseEventKind::Down(MouseButton::Left), rect.x, 14),
            false,
        );
        ui.on_mouse(
            mouse(MouseEventKind::Up(MouseButton::Left), rect.x, 14),
            false,
        );
        assert!(ui.resize_pointer(), "{cell:?}: still on the boundary");
        let rect = divider(&ui, cell);
        let buf = buffer(&mut ui, 120, 30);
        assert_eq!(line_cells(&buf, rect), usize::from(rect.height), "{cell:?}");
        ui.on_mouse(mouse(MouseEventKind::Moved, rect.x + 9, 14), false);
        assert!(!ui.resize_pointer());
        let buf = buffer(&mut ui, 120, 30);
        assert_eq!(line_cells(&buf, rect), 0, "{cell:?}: gone after the move");
    }
}

/// The keyboard's column mode keeps its column lit until Esc.
#[test]
fn the_keyboard_column_mode_keeps_its_column_lit_until_escape() {
    set(ColumnResize::Line);
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, ..) = radar(dir.path());
    buffer(&mut ui, 120, 30);
    ui.on_key(key(KeyCode::Char('C')));
    assert_eq!(ui.col_pick, Some(Cell::Repo));
    let rect = divider(&ui, Cell::Repo);
    let buf = buffer(&mut ui, 120, 30);
    assert_eq!(line_cells(&buf, rect), usize::from(rect.height));
    ui.on_key(key(KeyCode::Esc));
    let buf = buffer(&mut ui, 120, 30);
    assert_eq!(line_cells(&buf, rect), 0);
}

/// The age column has its own separator: a drag sizes it leftwards, the
/// width is kept, the key mode reaches it, and a cut age has a tooltip.
#[test]
fn the_age_column_is_sizable() {
    set(ColumnResize::Line);
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, ..) = radar(dir.path());
    buffer(&mut ui, 140, 30);
    let auto = ui.col_eff[3];
    assert!(auto >= 3, "fits the widest age and its title: {auto}");
    let rect = divider(&ui, Cell::Age);
    down_drag_up(&mut ui, rect, (rect.x - 3, 14));
    assert_eq!(ui.state.columns.age, Some((auto + 3).max(4) as u16));
    let saved = RadarState::load(&RadarState::file_in(&dir.path().join("cfg")));
    assert_eq!(saved.columns.age, ui.state.columns.age);
    // Double-click fits it again.
    buffer(&mut ui, 140, 30);
    let rect = divider(&ui, Cell::Age);
    ui.on_mouse(
        mouse(MouseEventKind::Down(MouseButton::Left), rect.x, 14),
        true,
    );
    assert_eq!(ui.state.columns.age, None);
    // The keys: Tab walks Repo, Branches, #, Author, Age.
    ui.on_key(key(KeyCode::Char('C')));
    for _ in 0..4 {
        ui.on_key(key(KeyCode::Tab));
    }
    assert_eq!(ui.col_pick, Some(Cell::Age));
    buffer(&mut ui, 140, 30);
    ui.on_key(key(KeyCode::Char(']')));
    assert!(ui.state.columns.age.is_some());
}

/// The list outgrows its window: the shared bar rides its last column, no
/// row reaches it, and a drag on the thumb scrolls the list without the
/// selection pulling it back.
#[test]
fn a_long_list_has_a_scrollbar_the_thumb_drags() {
    set(ColumnResize::Line);
    let dir = tempfile::tempdir().unwrap();
    let mut ui = long_list(dir.path());
    let buf = buffer(&mut ui, 120, 20);
    let track = ui.bar.expect("the list overflows, so it has a bar");
    assert_eq!(track.rect.x, 119, "the last column");
    assert_eq!(buf[(track.rect.x, track.rect.y)].symbol(), "\u{2191}");
    assert_eq!(
        buf[(track.rect.x, track.rect.bottom() - 1)].symbol(),
        "\u{2193}"
    );
    assert!(
        ui.hits.iter().all(|(r, _)| r.right() <= track.rect.x),
        "no hit rect runs under the bar"
    );
    let before = ui.scroll;
    ui.on_mouse(
        mouse(
            MouseEventKind::Down(MouseButton::Left),
            track.rect.x,
            track.rect.y + 1,
        ),
        false,
    );
    ui.on_mouse(
        mouse(
            MouseEventKind::Drag(MouseButton::Left),
            0,
            track.rect.bottom() + 9,
        ),
        false,
    );
    ui.on_mouse(mouse(MouseEventKind::Up(MouseButton::Left), 0, 0), false);
    buffer(&mut ui, 120, 20);
    assert!(ui.scroll > before, "the drag scrolled the list");
    assert!(!ui.follow, "the selection does not pull it back");
}
