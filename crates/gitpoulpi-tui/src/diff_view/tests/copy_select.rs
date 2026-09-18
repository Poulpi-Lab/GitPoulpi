use super::*;
use crate::th;

// ---- Text selection ---------------------------

/// The inner rect the selection tests pretend the diff is drawn in.
fn inner(width: u16, height: u16) -> Rect {
    Rect::new(3, 2, width, height)
}

/// A selection from screen cell `from` to `to` on the side under
/// `from`, the way a mouse drag builds one.
fn drag(
    render: &DiffRender,
    area: Rect,
    scroll: usize,
    from: (u16, u16),
    to: (u16, u16),
) -> TextSelection {
    let side = text_side_at(render, area, from.0);
    let anchor = text_pos_at(render, area, scroll, side, from.0, from.1).unwrap();
    let head = text_pos_at(render, area, scroll, side, to.0, to.1).unwrap();
    TextSelection { side, anchor, head }
}

fn state_with(sel: TextSelection) -> DiffViewState {
    let mut s = DiffViewState::for_commit(test_oid(), 0, PathBuf::from("src/lib.rs"));
    s.text_sel = Some(sel);
    s
}

fn change_hunk() -> FileDiff {
    file(vec![hunk(vec![
        line(LineOrigin::Context, "keep"),
        line(LineOrigin::Delete, "old line"),
        line(LineOrigin::Add, "new line"),
    ])])
}

/// The whole point of the feature: a drag over every row copies the
/// file's text - no line numbers, no `+`/`-`, no `@@` header.
#[test]
fn copying_a_selection_strips_gutter_markers_and_the_hunk_header() {
    let r = render_diff(&change_hunk(), DiffViewMode::Inline, true, false, 60);
    let area = inner(60, 10);
    // From the header row (column 0) to past the end of the last row.
    let sel = drag(&r, area, 0, (area.x, area.y), (area.x + 59, area.y + 3));
    let (_, text) = copy_text(&r, &state_with(sel), DiffCopy::Selection).unwrap();
    assert_eq!(text, "keep\nold line\nnew line");
}

/// Columns are measured from the content, not the gutter: the first
/// content cell is the line's first char.
#[test]
fn a_partial_drag_copies_exactly_the_chars_under_it() {
    let r = render_diff(&change_hunk(), DiffViewMode::Inline, true, false, 60);
    let area = inner(60, 10);
    let x0 = area.x + GUTTER_WIDTH as u16 + 1;
    // "old line" is row 2; "line" starts at char 4.
    let sel = drag(&r, area, 0, (x0 + 4, area.y + 2), (x0 + 7, area.y + 2));
    let (_, text) = copy_text(&r, &state_with(sel), DiffCopy::Selection).unwrap();
    assert_eq!(text, "line");
}

#[test]
fn a_selection_across_word_wrapped_rows_copies_the_line_whole() {
    let long = "alpha beta gamma delta epsilon zeta eta theta iota kappa";
    let f = file(vec![hunk(vec![line(LineOrigin::Add, long)])]);
    let r = render_diff(&f, DiffViewMode::Inline, true, true, 40);
    assert!(r.lines.len() > 3, "the line must wrap");
    let area = inner(40, 20);
    let x0 = area.x + GUTTER_WIDTH as u16 + 1;
    let last = area.y + r.lines.len() as u16 - 1;
    // From "pha" on the first row to the end of the last wrapped row.
    let sel = drag(&r, area, 0, (x0 + 2, area.y + 1), (area.x + 39, last));
    let (_, text) = copy_text(&r, &state_with(sel), DiffCopy::Selection).unwrap();
    assert_eq!(text, &long[2..]);
}

/// Scrolled sideways, a column still lands on its real char: the `«`
/// cell stands for the first hidden one.
#[test]
fn a_selection_under_a_horizontal_scroll_copies_the_scrolled_to_chars() {
    let f = file(vec![hunk(vec![line(LineOrigin::Add, "0123456789abcdef")])]);
    let r = render_diff_scrolled(&f, DiffViewMode::Inline, true, false, 60, 5);
    let area = inner(60, 10);
    let x0 = area.x + GUTTER_WIDTH as u16 + 1;
    let sel = drag(&r, area, 0, (x0 + 1, area.y + 1), (x0 + 3, area.y + 1));
    let (_, text) = copy_text(&r, &state_with(sel), DiffCopy::Selection).unwrap();
    assert_eq!(text, "567");
}

/// Split: the side is where the drag started, and a drag that wanders
/// onto the other half keeps extending the first one.
#[test]
fn a_split_selection_stays_on_the_side_it_started_on() {
    let r = render_diff(&change_hunk(), DiffViewMode::Split, true, false, 80);
    let area = inner(80, 10);
    let split = area.x + r.split_x.unwrap();
    // Row 2 pairs "old line" (before) with "new line" (after).
    let right = drag(
        &r,
        area,
        0,
        (split + 7, area.y + 2),
        (area.x + 79, area.y + 2),
    );
    assert_eq!(right.side, 1);
    let (_, text) = copy_text(&r, &state_with(right), DiffCopy::Selection).unwrap();
    assert_eq!(text, "new line");
    let left = drag(
        &r,
        area,
        0,
        (area.x + 7, area.y + 1),
        (split + 20, area.y + 2),
    );
    assert_eq!(left.side, 0);
    let (_, text) = copy_text(&r, &state_with(left), DiffCopy::Selection).unwrap();
    assert_eq!(text, "keep\nold line");
}

/// A wide char is two cells and one char: both of its cells map to it.
#[test]
fn wide_chars_map_both_their_cells_to_one_char() {
    let f = file(vec![hunk(vec![line(
        LineOrigin::Add,
        "\u{65e5}\u{672c} ok",
    )])]);
    let r = render_diff(&f, DiffViewMode::Inline, true, false, 60);
    let area = inner(60, 10);
    let x0 = area.x + GUTTER_WIDTH as u16 + 1;
    let sel = drag(&r, area, 0, (x0 + 1, area.y + 1), (x0 + 3, area.y + 1));
    let (_, text) = copy_text(&r, &state_with(sel), DiffCopy::Selection).unwrap();
    assert_eq!(text, "\u{65e5}\u{672c}");
}

/// Copy line / Copy hunk: whole lines, no decorations; Split's hunk
/// is the after column.
#[test]
fn copy_line_and_copy_hunk_take_whole_lines() {
    let inline = render_diff(&change_hunk(), DiffViewMode::Inline, true, false, 60);
    let mut s = DiffViewState::for_commit(test_oid(), 0, PathBuf::from("src/lib.rs"));
    s.cursor = 2;
    assert_eq!(
        copy_text(&inline, &s, DiffCopy::Line).unwrap(),
        ("the line", "old line".to_string())
    );
    assert_eq!(
        copy_text(&inline, &s, DiffCopy::Hunk).unwrap().1,
        "keep\nold line\nnew line"
    );
    // The `v` range, when one is open.
    s.selection_anchor = Some(1);
    assert_eq!(
        copy_text(&inline, &s, DiffCopy::Line).unwrap().1,
        "keep\nold line"
    );
    let split = render_diff(&change_hunk(), DiffViewMode::Split, true, false, 80);
    assert_eq!(
        copy_text(&split, &s, DiffCopy::Hunk).unwrap().1,
        "keep\nnew line"
    );
    // Nothing selected: `yank` falls back to the line(s).
    assert_eq!(yank(&inline, &s).unwrap().1, "keep\nold line");
    assert!(copy_text(&inline, &s, DiffCopy::Selection).is_err());
}

/// The highlight, through a real buffer: the selected chars' cells
/// only - never the gutter, never the rest of the row.
#[test]
fn the_selection_highlight_covers_the_selected_cells_only() {
    use ratatui::backend::TestBackend;
    use ratatui::widgets::Paragraph;
    use ratatui::Terminal;
    let r = render_diff(&change_hunk(), DiffViewMode::Inline, true, false, 40);
    let mut terminal = Terminal::new(TestBackend::new(40, 5)).unwrap();
    let area = Rect::new(0, 0, 40, 5);
    let x0 = GUTTER_WIDTH as u16 + 1;
    let sel = drag(&r, area, 0, (x0 + 4, 2), (x0 + 7, 2));
    let state = state_with(sel);
    terminal
        .draw(|frame| {
            frame.render_widget(Paragraph::new(r.lines.clone()), area);
            paint_text_selection(frame.buffer_mut(), area, &r, &state);
        })
        .unwrap();
    let buf = terminal.backend().buffer();
    let lit: Vec<(u16, u16)> = (0..5)
        .flat_map(|y| (0..40).map(move |x| (x, y)))
        .filter(|&(x, y)| buf[(x, y)].bg == th().ui.text_selection_bg)
        .collect();
    let expected: Vec<(u16, u16)> = (x0 + 4..=x0 + 7).map(|x| (x, 2)).collect();
    assert_eq!(lit, expected);
    let word: String = expected
        .iter()
        .map(|&(x, y)| buf[(x, y)].symbol())
        .collect();
    assert_eq!(word, "line");
}
