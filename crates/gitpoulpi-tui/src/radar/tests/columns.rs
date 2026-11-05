//! The sizable columns without a terminal: widths, the ellipsis, the hit
//! rects of the separator and of the cut cells (from the paint walk), the
//! drag, the keys and what is saved.

use super::*;
use crate::radar::Cell;
use crossterm::event::MouseEvent;

const LONG: &str = "group/subgroup/project";

/// A Radar whose repositories have a long name.
fn long_named(dir: &std::path::Path) -> RadarUi {
    let (mut ui, ..) = radar(dir);
    for item in &mut ui.board.items {
        item.repo_name = LONG.into();
        item.project = LONG.into();
    }
    ui
}

fn rect_of(ui: &RadarUi, want: impl Fn(Hit) -> bool) -> Rect {
    ui.hits
        .iter()
        .find(|(_, h)| want(*h))
        .map(|(r, _)| *r)
        .unwrap_or_else(|| panic!("no such hit in {:?}", ui.hits))
}

fn mouse(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn down(ui: &mut RadarUi, at: Rect, double: bool) {
    ui.on_mouse(
        mouse(MouseEventKind::Down(MouseButton::Left), at.x, at.y),
        double,
    );
}

#[test]
fn the_repository_column_is_wide_enough_for_a_typical_project_path() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = long_named(dir.path());
    let text = screen(&mut ui, 120, 30);
    assert!(text.contains(LONG), "{text}");
    assert!(!ui
        .hits
        .iter()
        .any(|(_, h)| matches!(h, Hit::Cut(_, Cell::Repo))));
}

#[test]
fn a_narrow_column_ends_in_an_ellipsis_and_registers_a_cut_cell_and_a_separator() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = long_named(dir.path());
    ui.state.columns.repo = Some(8);
    let text = screen(&mut ui, 120, 30);
    assert!(!text.contains(LONG), "{text}");
    assert!(text.contains("group/s\u{2026}"), "{text}");
    let cut = rect_of(&ui, |h| matches!(h, Hit::Cut(_, Cell::Repo)));
    let sep = rect_of(&ui, |h| h == Hit::Divider(Cell::Repo));
    assert_eq!(cut.width, 8);
    assert_eq!(
        sep.x,
        cut.right(),
        "the separator is the cell after the name"
    );
    // `line` mode: the separator runs from the header row down the list.
    assert!(sep.y < cut.y && sep.bottom() > cut.y, "{sep:?} {cut:?}");
    // The cut cell's tooltip is the whole name.
    let item = &ui.board.items[0];
    assert_eq!(RadarUi::cut_text(item, Cell::Repo), LONG);
}

#[test]
fn dragging_the_separator_sizes_the_column_and_moves_the_separator() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = long_named(dir.path());
    ui.state.columns.repo = Some(8);
    screen(&mut ui, 120, 30);
    let sep = rect_of(&ui, |h| h == Hit::Divider(Cell::Repo));
    down(&mut ui, sep, false);
    // The pointer may leave the separator's row and cell while dragging.
    ui.on_mouse(
        mouse(
            MouseEventKind::Drag(MouseButton::Left),
            sep.x + 6,
            sep.y + 3,
        ),
        false,
    );
    ui.on_mouse(
        mouse(MouseEventKind::Up(MouseButton::Left), sep.x + 6, sep.y + 3),
        false,
    );
    assert_eq!(ui.state.columns.repo, Some(14));
    screen(&mut ui, 120, 30);
    let moved = rect_of(&ui, |h| h == Hit::Divider(Cell::Repo));
    assert_eq!(moved.x, sep.x + 6, "the hit rect follows the new width");
    // Saved on release.
    let saved = RadarState::load(&RadarState::file_in(&dir.path().join("cfg")));
    assert_eq!(saved.columns.repo, Some(14));
    // A drag never goes under the least width.
    down(&mut ui, moved, false);
    ui.on_mouse(
        mouse(MouseEventKind::Drag(MouseButton::Left), 0, moved.y),
        false,
    );
    ui.on_mouse(
        mouse(MouseEventKind::Up(MouseButton::Left), 0, moved.y),
        false,
    );
    assert_eq!(
        ui.state.columns.repo,
        Some(crate::radar::columns::MIN_WIDTH as u16)
    );
}

#[test]
fn a_double_click_on_the_separator_fits_the_column_again() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = long_named(dir.path());
    ui.state.columns.repo = Some(8);
    screen(&mut ui, 120, 30);
    let sep = rect_of(&ui, |h| h == Hit::Divider(Cell::Repo));
    down(&mut ui, sep, true);
    assert_eq!(ui.state.columns.repo, None);
    let text = screen(&mut ui, 120, 30);
    assert!(text.contains(LONG), "{text}");
}

#[test]
fn the_keys_pick_size_and_reset_a_column_and_the_width_is_saved() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = long_named(dir.path());
    ui.state.columns.repo = Some(8);
    screen(&mut ui, 120, 30);
    press(&mut ui, 'C');
    assert_eq!(ui.col_pick, Some(Cell::Repo));
    assert!(ui.message.as_deref().unwrap_or("").contains("Repository"));
    press(&mut ui, ']');
    assert_eq!(ui.state.columns.repo, Some(10));
    press(&mut ui, '[');
    press(&mut ui, '[');
    assert_eq!(ui.state.columns.repo, Some(6));
    let saved = RadarState::load(&RadarState::file_in(&dir.path().join("cfg")));
    assert_eq!(saved.columns.repo, Some(6));
    // Tab moves to the number column, then to the author column; each width
    // is its own.
    ui.on_key(key(KeyCode::Tab));
    assert_eq!(ui.col_pick, Some(Cell::Num));
    screen(&mut ui, 120, 30);
    press(&mut ui, ']');
    assert!(ui.state.columns.number.is_some());
    ui.on_key(key(KeyCode::Tab));
    assert_eq!(ui.col_pick, Some(Cell::Author));
    screen(&mut ui, 120, 30);
    press(&mut ui, ']');
    assert!(ui.state.columns.author.is_some());
    assert_eq!(ui.state.columns.repo, Some(6));
    press(&mut ui, '0');
    assert!(ui.state.columns.is_auto());
    // Esc leaves the mode without closing the view.
    assert_eq!(ui.on_key(key(KeyCode::Esc)), Outcome::None);
    assert_eq!(ui.col_pick, None);
}

#[test]
fn the_palette_commands_do_what_the_keys_do() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = long_named(dir.path());
    ui.state.columns.repo = Some(8);
    screen(&mut ui, 120, 30);
    ui.command(RadarCommand::Column(ColumnCmd::Wider));
    assert_eq!(ui.state.columns.repo, Some(10));
    ui.command(RadarCommand::Column(ColumnCmd::Narrower));
    assert_eq!(ui.state.columns.repo, Some(8));
    ui.command(RadarCommand::Column(ColumnCmd::Reset));
    assert!(ui.state.columns.is_auto());
}

#[test]
fn a_cut_title_and_author_are_cut_cells_with_the_full_text_for_the_tooltip() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = long_named(dir.path());
    ui.board.items[0].title = "A title that is far too long for the narrow window ".repeat(3);
    ui.board.items[0].author = "someone-with-a-very-long-login".into();
    ui.rows = vec![Row::Item(0)];
    ui.state.columns.author = Some(8);
    let text = screen(&mut ui, 100, 20);
    assert!(text.contains('\u{2026}'), "{text}");
    for cell in [Cell::Title, Cell::Author] {
        let hit = ui
            .hits
            .iter()
            .find(|(_, h)| matches!(h, Hit::Cut(0, c) if *c == cell));
        assert!(hit.is_some(), "{cell:?} is cut: {:?}", ui.hits);
    }
    assert_eq!(
        RadarUi::cut_text(&ui.board.items[0], Cell::Author),
        "someone-with-a-very-long-login"
    );
    assert!(RadarUi::cut_text(&ui.board.items[0], Cell::Title).starts_with("A title that"));
}

#[test]
fn the_status_line_of_the_selected_row_names_the_full_project() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = long_named(dir.path());
    ui.state.columns.repo = Some(6);
    select(&mut ui, "Change 1");
    let text = screen(&mut ui, 160, 30);
    let status = text.lines().nth(4).unwrap();
    assert!(status.contains(LONG), "{status}");
}

/// Every item row's author starts at the same cell, whatever else the row
/// wears: checks and conflict marks, a label, a pin, a draft chip, the
/// dimmed chip of a repository with no clone.
#[test]
fn the_author_column_starts_at_the_same_cell_on_every_row() {
    let dir = tempfile::tempdir().unwrap();
    // Ages of one and several digits ("1d", "283d") vary the age column.
    set_clock(1_791_000_000);
    let (mut ui, ..) = radar(dir.path());
    assert!(ui.board.items.len() >= 4);
    ui.board.items[0].checks = Some(gitpoulpi_hosting::ChecksState::Passing);
    ui.board.items[0].has_conflicts = Some(true);
    ui.board.items[0].labels = vec!["bug".into(), "ui".into()];
    ui.board.items[1].draft = true;
    ui.board.items[1].author = "al".into();
    ui.board.items[2].repo = None;
    ui.board.items[2].author = "mid-length-one".into();
    ui.board.items[3].updated_at = "2026-01-01T10:00:00Z".into();
    let key = ui.board.items[3].key.clone();
    ui.state.pins.push(key);
    let text = screen(&mut ui, 150, 30);
    let lines: Vec<Vec<char>> = text.lines().map(|l| l.chars().collect()).collect();
    // One separator, down the whole list: the cell before the author column.
    let seps: Vec<Rect> = ui
        .hits
        .iter()
        .filter(|(_, h)| matches!(h, Hit::Divider(Cell::Author)))
        .map(|(r, _)| *r)
        .collect();
    assert_eq!(seps.len(), 1, "{seps:?}\n{text}");
    let sep = seps[0];
    let item_rows: Vec<u16> = (0..lines.len() as u16)
        .filter(|y| lines[*y as usize].contains(&'\u{22EE}'))
        .collect();
    assert!(item_rows.len() >= 4, "{text}");
    for y in item_rows {
        assert!(sep.y < y && y < sep.bottom(), "row {y} is under the line");
        let line = &lines[y as usize];
        assert_eq!(line[sep.x as usize], ' ', "row {y} gap:\n{text}");
        assert_ne!(line[sep.x as usize + 1], ' ', "row {y} author:\n{text}");
    }
    assert!(text.contains("al "), "{text}");
}

/// A lit separator in `line` mode is a thin accent line down the list; in
/// `header` mode the Log's resize strip (a solid accent cell) on the header.
#[test]
fn a_lit_separator_wears_the_configured_style() {
    use gitpoulpi_config::ColumnResize;
    for mode in [ColumnResize::Line, ColumnResize::Header] {
        crate::live_settings::install_column_resize(mode);
        let dir = tempfile::tempdir().unwrap();
        let (mut ui, ..) = radar(dir.path());
        ui.col_pick = Some(Cell::Repo);
        let mut terminal = Terminal::new(TestBackend::new(140, 30)).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                ui.render(f, area);
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let seps: Vec<Rect> = ui
            .hits
            .iter()
            .filter(|(_, h)| matches!(h, Hit::Divider(Cell::Repo)))
            .map(|(r, _)| *r)
            .collect();
        assert_eq!(seps.len(), 1, "{mode:?}");
        let r = seps[0];
        match mode {
            ColumnResize::Line => {
                assert!(r.height > 4, "the line runs down the list: {r:?}");
                for y in r.y..r.bottom() {
                    let cell = &buf[(r.x, y)];
                    assert_eq!(cell.symbol(), "\u{2502}", "row {y}");
                    assert_eq!(cell.fg, crate::th().ui.accent);
                    assert_ne!(cell.bg, crate::th().ui.accent, "thin, not a block");
                }
            }
            ColumnResize::Header => {
                assert_eq!(r.height, 1, "the header cell only");
                let cell = &buf[(r.x, r.y)];
                assert_eq!(cell.bg, crate::th().ui.accent);
                assert_eq!(cell.fg, crate::th().ui.on_accent);
            }
        }
    }
    crate::live_settings::install_column_resize(ColumnResize::Line);
}

/// The `#` column: the widest number plus the room for the check marks; a
/// drag sizes it, the width is saved and the author column stays aligned.
#[test]
fn the_number_column_is_sized_by_drag_and_kept_aligned() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, app, _) = radar(dir.path());
    ui.apply_answer(app_with_checks(&app));
    ui.rebuild();
    let text = screen(&mut ui, 140, 30);
    // Default: label and marks fit, nothing is cut (no tooltip hit on `#`).
    assert!(text.contains("\u{2717}!"), "{text}");
    assert!(!ui
        .hits
        .iter()
        .any(|(_, h)| matches!(h, Hit::Cut(_, Cell::Num))));
    let author_x = |ui: &RadarUi, text: &str| -> Vec<usize> {
        let _ = ui;
        text.lines()
            .filter(|l| l.contains('\u{22EE}'))
            .map(|l| {
                let at = l
                    .find("kai")
                    .or_else(|| l.find("ann"))
                    .or_else(|| l.find("bob"));
                // A column, not a byte: the row carries wide glyphs.
                l[..at.expect("an author")].chars().count()
            })
            .collect()
    };
    let before = author_x(&ui, &text);
    let sep = rect_of(&ui, |h| h == Hit::Divider(Cell::Num));
    down(&mut ui, sep, false);
    ui.on_mouse(
        mouse(MouseEventKind::Drag(MouseButton::Left), sep.x + 4, sep.y),
        false,
    );
    ui.on_mouse(
        mouse(MouseEventKind::Up(MouseButton::Left), sep.x + 4, sep.y),
        false,
    );
    let width = ui.state.columns.number.expect("sized");
    assert_eq!(width, 9, "5 cells (label and marks) plus the drag of 4");
    let text = screen(&mut ui, 140, 30);
    let moved = rect_of(&ui, |h| h == Hit::Divider(Cell::Num));
    assert_eq!(moved.x, sep.x + 4, "the separator follows the width");
    // The title starts right after the separator on every row, the author
    // column did not move (the title is the flexible one).
    assert_eq!(author_x(&ui, &text), before, "{text}");
    assert!(text.contains("\u{2717}!"), "{text}");
    // Saved on release.
    let saved = RadarState::load(&RadarState::file_in(&dir.path().join("cfg")));
    assert_eq!(saved.columns.number, Some(width));
    // Narrow: the number is cut with an ellipsis and a tooltip hit exists.
    ui.board.items[0].label = "#1234567".into();
    ui.state.columns.number = Some(5);
    screen(&mut ui, 140, 30);
    assert!(ui
        .hits
        .iter()
        .any(|(_, h)| matches!(h, Hit::Cut(_, Cell::Num))));
    // A double click fits it again.
    let sep = rect_of(&ui, |h| h == Hit::Divider(Cell::Num));
    down(&mut ui, sep, true);
    assert_eq!(ui.state.columns.number, None);
    // The header names it over its cells.
    let text = screen(&mut ui, 140, 30);
    let header = text.lines().nth(5).unwrap();
    let rows: Vec<usize> = text
        .lines()
        .filter(|l| l.contains("Change"))
        .map(|l| l[..l.find('#').unwrap()].chars().count())
        .collect();
    let at = header[..header.find('#').unwrap()].chars().count();
    assert!(rows.iter().all(|x| *x == at), "{text}");
}

/// The buffer cells of row `y`, x in `from..to`.
fn cells(buf: &ratatui::buffer::Buffer, y: u16, from: u16, to: u16) -> Vec<String> {
    (from..to)
        .map(|x| buf[(x, y)].symbol().to_string())
        .collect()
}

/// A column sized down to 1 or 2 cells: a CJK author is cut to the ellipsis
/// (and a blank) inside its own cells, never spilling into the next column,
/// the header is cut the same way, and the separator before it stays one
/// grabbable cell that does not cover either neighbour.
#[test]
fn a_one_or_two_cell_author_column_stays_inside_its_cells() {
    const NAME: &str = "\u{65E5}\u{672C}\u{8A9E}";
    for width in [1u16, 2] {
        let dir = tempfile::tempdir().unwrap();
        let (mut ui, ..) = radar(dir.path());
        for item in &mut ui.board.items {
            item.author = NAME.into();
        }
        ui.state.columns.author = Some(width);
        let (w, h) = (150u16, 30u16);
        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                ui.render(f, area);
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let sep = rect_of(&ui, |h| h == Hit::Divider(Cell::Author));
        assert_eq!((sep.width, ui.col_eff[1]), (1, usize::from(width)));
        let x = sep.x + 1;
        let mut rows = 0;
        for y in sep.y + 1..sep.bottom() {
            let line = cells(&buf, y, 0, w);
            if !line.iter().any(|c| c == "\u{22EE}") {
                continue;
            }
            rows += 1;
            // Inside the column: the ellipsis then blanks, no half glyph.
            let inside = cells(&buf, y, x, x + width);
            assert_eq!(inside[0], "\u{2026}", "row {y}: {inside:?}");
            assert!(inside[1..].iter().all(|c| c == " "), "row {y}: {inside:?}");
            // Nothing of the name past the column: the gap cells are blank.
            assert_eq!(
                cells(&buf, y, x + width, x + width + 2),
                [" ", " "],
                "row {y}"
            );
            assert!(line.iter().all(|c| !NAME.contains(c.as_str()) || c == " "));
            // The separator cell is the gap before the column, left blank.
            assert_eq!(buf[(sep.x, y)].symbol(), " ", "row {y}");
        }
        assert!(rows >= 2, "item rows seen");
        // The header is cut inside the column too, and the cell after it is blank.
        let head = cells(&buf, sep.y, x, x + width + 1);
        assert_eq!(head[usize::from(width) - 1], "\u{2026}", "{head:?}");
        assert_eq!(head[usize::from(width)], " ", "{head:?}");
        // The cut cells are the tooltip's, exactly the column wide.
        let cut = ui
            .hits
            .iter()
            .find(|(_, h)| matches!(h, Hit::Cut(_, Cell::Author)))
            .map(|(r, _)| *r)
            .expect("a cut author");
        assert_eq!((cut.x, cut.width), (x, width));
    }
}

/// The least width of every sized column is one cell: for a stored width, by
/// the keys and by a drag.
#[test]
fn every_sizable_column_goes_down_to_one_cell() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = long_named(dir.path());
    ui.state.columns.repo = Some(1);
    ui.state.columns.author = Some(1);
    ui.state.columns.number = Some(1);
    ui.state.columns.age = Some(1);
    screen(&mut ui, 150, 30);
    assert_eq!(ui.col_eff, [1, 1, 1, 1], "stored widths of 1 are kept");
    for cell in [Cell::Repo, Cell::Num, Cell::Author, Cell::Age] {
        let sep = rect_of(&ui, |h| h == Hit::Divider(cell));
        assert_eq!(sep.width, 1, "{cell:?}");
    }
    // The keys narrow a column to one cell and no further.
    ui.state.columns.repo = Some(3);
    screen(&mut ui, 150, 30);
    press(&mut ui, 'C');
    press(&mut ui, '[');
    assert_eq!(ui.state.columns.repo, Some(1));
    press(&mut ui, '[');
    assert_eq!(ui.state.columns.repo, Some(1));
    // A drag far past the minimum stops at one cell.
    for (cell, to) in [(Cell::Repo, 0u16), (Cell::Author, 149)] {
        ui.state.columns.repo = Some(12);
        ui.state.columns.author = Some(12);
        screen(&mut ui, 150, 30);
        let sep = rect_of(&ui, |h| h == Hit::Divider(cell));
        down(&mut ui, sep, false);
        ui.on_mouse(
            mouse(MouseEventKind::Drag(MouseButton::Left), to, sep.y),
            false,
        );
        ui.on_mouse(
            mouse(MouseEventKind::Up(MouseButton::Left), to, sep.y),
            false,
        );
        let got = match cell {
            Cell::Repo => ui.state.columns.repo,
            _ => ui.state.columns.author,
        };
        assert_eq!(got, Some(1), "{cell:?}");
    }
}
