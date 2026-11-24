//! The changed-word emphasis as the terminal buffer shows it: the cells of
//! the words that differ carry the theme's stronger background, the rest of
//! the line keeps the flat tint, in both layouts, under word wrap and when
//! scrolled sideways.

use super::*;
use crate::diff_view::model::{render_diff_with, DiffRenderOpts};
use crate::th;
use ratatui::backend::TestBackend;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;

const OLD: &str = "let total = compute(first, second);";
const NEW: &str = "let total = compute(first, third);";

fn opts(
    mode: DiffViewMode,
    word_wrap: bool,
    width: u16,
    h_offset: usize,
    on: bool,
) -> DiffRenderOpts<'static> {
    DiffRenderOpts {
        mode,
        hunks_only: true,
        show_whitespace: true,
        word_wrap,
        width,
        h_offset,
        full_file: None,
        lfs_raw: false,
        lfs_local: None,
        intraline: on,
    }
}

fn changed_file() -> FileDiff {
    file(vec![hunk(vec![
        line(LineOrigin::Context, "fn main() {"),
        line(LineOrigin::Delete, OLD),
        line(LineOrigin::Add, NEW),
        line(LineOrigin::Context, "}"),
    ])])
}

type Row = (String, Vec<Color>);

/// The drawn buffer of `lines`, one row per line.
fn draw(lines: &[Line<'static>], width: u16) -> ratatui::buffer::Buffer {
    let mut terminal = Terminal::new(TestBackend::new(width, lines.len() as u16)).unwrap();
    terminal
        .draw(|frame| frame.render_widget(Paragraph::new(lines.to_vec()), frame.area()))
        .unwrap();
    terminal.backend().buffer().clone()
}

/// The text and the per-cell background of row `y`.
fn row(buf: &ratatui::buffer::Buffer, y: u16, width: u16) -> Row {
    let mut text = String::new();
    let mut bgs = Vec::new();
    for x in 0..width {
        text.push_str(buf[(x, y)].symbol());
        bgs.push(buf[(x, y)].style().bg.unwrap_or(Color::Reset));
    }
    (text, bgs)
}

/// The background of the first cell of `word` in the row text (every glyph
/// of these rows is one cell wide, so a byte offset is a column).
fn bg_at(row: &Row, word: &str) -> Color {
    let at = row
        .0
        .find(word)
        .unwrap_or_else(|| panic!("{word:?} in {:?}", row.0));
    row.1[at]
}

fn rows_of(render: &DiffRender, width: u16) -> Vec<Row> {
    let buf = draw(&render.lines, width);
    (0..render.lines.len() as u16)
        .map(|y| row(&buf, y, width))
        .collect()
}

fn find_row<'a>(rows: &'a [Row], needle: &str) -> &'a Row {
    rows.iter()
        .find(|r| r.0.contains(needle))
        .unwrap_or_else(|| panic!("no row with {needle:?}"))
}

#[test]
fn inline_marks_only_the_changed_word_of_each_line() {
    let d = th().diff;
    let r = render_diff_with(
        &changed_file(),
        &opts(DiffViewMode::Inline, false, 80, 0, true),
    );
    let rows = rows_of(&r, 80);
    let del = find_row(&rows, "second");
    let add = find_row(&rows, "third");
    assert_eq!(bg_at(del, "second"), d.del_emph_bg);
    assert_eq!(bg_at(add, "third"), d.add_emph_bg);
    // Everything else on those lines keeps the flat tint, the marker too.
    assert_eq!(bg_at(del, "let total"), d.del_bg);
    assert_eq!(bg_at(del, "first"), d.del_bg);
    assert_eq!(bg_at(add, "compute"), d.add_bg);
    assert_eq!(bg_at(del, "-let"), d.del_bg);
    // The unchanged context rows are untouched.
    assert_ne!(bg_at(find_row(&rows, "fn main"), "fn main"), d.add_emph_bg);
}

#[test]
fn off_leaves_one_flat_tint_per_line() {
    let d = th().diff;
    let r = render_diff_with(
        &changed_file(),
        &opts(DiffViewMode::Inline, false, 80, 0, false),
    );
    let rows = rows_of(&r, 80);
    assert_eq!(bg_at(find_row(&rows, "second"), "second"), d.del_bg);
    assert_eq!(bg_at(find_row(&rows, "third"), "third"), d.add_bg);
}

#[test]
fn split_marks_both_halves_of_the_row() {
    let d = th().diff;
    let r = render_diff_with(
        &changed_file(),
        &opts(DiffViewMode::Split, false, 120, 0, true),
    );
    let rows = rows_of(&r, 120);
    let both = find_row(&rows, "second");
    assert!(
        both.0.contains("third"),
        "one row for the pair: {:?}",
        both.0
    );
    assert_eq!(bg_at(both, "second"), d.del_emph_bg);
    assert_eq!(bg_at(both, "third"), d.add_emph_bg);
    assert_eq!(bg_at(both, "first"), d.del_bg);
    let right = both.0.rfind("compute").unwrap();
    assert_eq!(both.1[right], d.add_bg);
    // The padding after the text is flat, not emphasised.
    let last = both.1.len() - 1;
    assert_ne!(both.1[last], d.add_emph_bg);
    assert_ne!(both.1[last], d.del_emph_bg);
}

#[test]
fn word_wrap_keeps_the_mark_on_the_wrapped_row() {
    let d = th().diff;
    // 13 columns of gutter and marker leave 14 for the text: the changed
    // word lands on a second row.
    let r = render_diff_with(
        &changed_file(),
        &opts(DiffViewMode::Inline, true, 27, 0, true),
    );
    let rows = rows_of(&r, 27);
    let del = find_row(&rows, "second");
    assert_eq!(bg_at(del, "second"), d.del_emph_bg);
    let add = find_row(&rows, "third");
    assert_eq!(bg_at(add, "third"), d.add_emph_bg);
    // The row that holds only unchanged words has no emphasis at all.
    let head = find_row(&rows, "-let total");
    assert!(head.1.iter().all(|c| *c != d.del_emph_bg), "{:?}", head.0);
}

#[test]
fn a_sideways_scroll_keeps_the_mark_on_the_right_cells() {
    let d = th().diff;
    // Scrolled past "let total = ": the marker takes one column.
    let r = render_diff_with(
        &changed_file(),
        &opts(DiffViewMode::Inline, false, 60, 12, true),
    );
    let rows = rows_of(&r, 60);
    let del = find_row(&rows, "second");
    assert!(del.0.contains('\u{ab}'), "{:?}", del.0);
    // `\u{ab}` is two bytes: look the word up by chars, not bytes.
    let col_of = |word: &str| del.0[..del.0.find(word).unwrap()].chars().count();
    assert_eq!(del.1[col_of("second")], d.del_emph_bg);
    assert_eq!(del.1[col_of("first")], d.del_bg);
}

#[test]
fn the_row_map_and_selection_text_are_unchanged_by_the_emphasis() {
    let off = render_diff_with(
        &changed_file(),
        &opts(DiffViewMode::Inline, false, 80, 0, false),
    );
    let on = render_diff_with(
        &changed_file(),
        &opts(DiffViewMode::Inline, false, 80, 0, true),
    );
    assert_eq!(plain(&on.lines), plain(&off.lines));
    assert_eq!(on.rows, off.rows);
    assert_eq!(on.text.sides, off.text.sides);
}

/// The cost stays bounded: a big hunk of changed lines, emphasis on.
#[test]
fn a_large_hunk_is_emphasised_within_budget() {
    let n = 3000;
    let mut lines = Vec::new();
    for i in 0..n {
        lines.push(line(
            LineOrigin::Delete,
            &format!("let value_{i} = old_call({i}, alpha, beta);"),
        ));
    }
    for i in 0..n {
        lines.push(line(
            LineOrigin::Add,
            &format!("let value_{i} = new_call({i}, alpha, gamma);"),
        ));
    }
    let big = file(vec![hunk(lines)]);
    let start = std::time::Instant::now();
    let r = render_diff_with(&big, &opts(DiffViewMode::Inline, false, 120, 0, true));
    let took = start.elapsed();
    eprintln!("emphasis of {n} pairs: {took:?}");
    assert_eq!(r.lines.len(), 2 * n + 1);
    assert!(took < std::time::Duration::from_secs(2), "{took:?}");
}
