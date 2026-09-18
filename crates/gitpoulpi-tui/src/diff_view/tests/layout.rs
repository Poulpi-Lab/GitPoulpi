use super::*;

#[test]
fn a_diff_with_no_hunks_renders_a_message_not_a_panic_or_a_blank_pane() {
    // Binary files and pure mode/rename changes arrive with zero hunks.
    let rendered = render_diff(&file(vec![]), DiffViewMode::Inline, true, false, 80);
    assert_eq!(rendered.lines.len(), 1);
    assert!(plain(&rendered.lines)[0].contains("binary"));
    assert!(rendered.hunk_starts.is_empty());
}

#[test]
fn a_zero_width_pane_still_produces_well_formed_rows() {
    // ratatui hands out degenerate rects mid-resize; every width
    // subtraction in here is saturating for this reason.
    for mode in [DiffViewMode::Inline, DiffViewMode::Split] {
        let rendered = render_diff(&sample(), mode, true, false, 0);
        assert!(!rendered.lines.is_empty());
        let rendered = render_diff(&sample(), mode, true, true, 0);
        assert!(!rendered.lines.is_empty());
    }
}

#[test]
fn inline_mode_shows_every_line_with_old_and_new_numbers() {
    let rendered = render_diff(&sample(), DiffViewMode::Inline, true, false, 80);
    let text = plain(&rendered.lines);
    assert!(text[0].starts_with("@@ -10,3 +10,3 @@"));
    // context, delete, add, context -- all four, nothing trimmed.
    assert_eq!(text.len(), 5);
    assert!(text[1].contains("10") && text[1].contains("fn main"));
    // The delete carries an old number but no new one; the add the
    // reverse -- that asymmetry is the whole point of the gutter.
    assert!(text[2].contains('-') && text[2].contains("let x = 1;"));
    assert!(text[3].contains('+') && text[3].contains("let x = 2;"));
}

/// Hunks only shows every line git gave the hunk - its three lines
/// of context included, nothing trimmed (the pre-item-30 Inline and
/// Split views, which is what every old setting migrates to).
#[test]
fn hunks_only_keeps_all_of_gits_context() {
    let f = file(vec![hunk(vec![
        line(LineOrigin::Context, "a"),
        line(LineOrigin::Context, "b"),
        line(LineOrigin::Context, "c"),
        line(LineOrigin::Delete, "d"),
        line(LineOrigin::Add, "D"),
        line(LineOrigin::Context, "e"),
        line(LineOrigin::Context, "f"),
        line(LineOrigin::Context, "g"),
    ])]);
    let rendered = render_diff(&f, DiffViewMode::Inline, true, false, 80);
    assert_eq!(rendered.lines.len(), 1 + 8);
    let text = plain(&rendered.lines).join("\n");
    assert!(text.contains(" a") && text.contains(" g"), "{text}");
}

#[test]
fn split_mode_puts_the_delete_left_and_the_add_right_on_one_row() {
    let rendered = render_diff(&sample(), DiffViewMode::Split, true, false, 80);
    let text = plain(&rendered.lines);
    // Row 0 is the hunk header; row 1 is the shared context line; row 2
    // pairs the delete with the add.
    let paired = &text[2];
    assert!(paired.contains("let x = 1;"), "old side present");
    assert!(paired.contains("let x = 2;"), "new side present");
}

#[test]
fn split_mode_leaves_the_left_side_blank_for_a_pure_addition() {
    // An added file has adds and nothing else -- the "before" column
    // must render empty rather than the row collapsing or panicking.
    let f = file(vec![hunk(vec![
        line(LineOrigin::Add, "brand new"),
        line(LineOrigin::Add, "second line"),
    ])]);
    let rendered = render_diff(&f, DiffViewMode::Split, true, false, 80);
    let text = plain(&rendered.lines);
    assert!(text[1].contains("brand new"));
    // Nothing from a nonexistent old side leaked in.
    assert!(!text[1].contains("second line"));
    assert!(text[2].contains("second line"));
}

#[test]
fn split_mode_handles_a_deleted_file_with_an_empty_new_side() {
    let f = file(vec![hunk(vec![
        line(LineOrigin::Delete, "gone one"),
        line(LineOrigin::Delete, "gone two"),
    ])]);
    let rendered = render_diff(&f, DiffViewMode::Split, true, false, 80);
    let text = plain(&rendered.lines);
    assert!(text[1].contains("gone one"));
    assert!(text[2].contains("gone two"));
}

#[test]
fn hiding_whitespace_collapses_a_trailing_space_only_change_to_context() {
    let f = file(vec![hunk(vec![
        line(LineOrigin::Context, "before"),
        line(LineOrigin::Delete, "    let x = 1;   "),
        line(LineOrigin::Add, "    let x = 1;"),
    ])]);
    let shown = render_diff(&f, DiffViewMode::Inline, true, false, 80);
    let hidden = render_diff(&f, DiffViewMode::Inline, false, false, 80);
    assert!(plain(&shown.lines).join("\n").contains('-'));
    // The only change was whitespace, so with the toggle off the whole
    // hunk disappears and the pane says so instead of going blank.
    let text = plain(&hidden.lines).join("\n");
    assert!(text.contains("only whitespace changes"), "got: {text}");
    assert!(hidden.hunk_starts.is_empty());
}

#[test]
fn hiding_whitespace_keeps_a_hunk_that_also_has_a_real_change() {
    let f = file(vec![hunk(vec![
        line(LineOrigin::Delete, "keep me   "),
        line(LineOrigin::Delete, "old value"),
        line(LineOrigin::Add, "keep me"),
        line(LineOrigin::Add, "new value"),
    ])]);
    let hidden = render_diff(&f, DiffViewMode::Inline, false, false, 80);
    let text = plain(&hidden.lines).join("\n");
    assert!(text.contains("old value") && text.contains("new value"));
    assert_eq!(hidden.hunk_starts.len(), 1);
}

#[test]
fn word_wrap_off_truncates_and_wrap_on_produces_extra_rows() {
    let long = "let message = \"a very long line of source code that will not fit\";";
    let f = file(vec![hunk(vec![line(LineOrigin::Add, long)])]);
    let narrow = 40;
    let off = render_diff(&f, DiffViewMode::Inline, true, false, narrow);
    let on = render_diff(&f, DiffViewMode::Inline, true, true, narrow);
    assert_eq!(off.lines.len(), 2, "header + exactly one truncated row");
    assert!(
        plain(&off.lines)[1].contains('\u{2026}'),
        "ellipsis expected"
    );
    assert!(
        on.lines.len() > off.lines.len(),
        "wrapping must add rows, got {} vs {}",
        on.lines.len(),
        off.lines.len()
    );
}

#[test]
fn word_wrap_hard_breaks_a_single_token_too_long_to_ever_fit() {
    let token = "x".repeat(200);
    let f = file(vec![hunk(vec![line(LineOrigin::Add, &token)])]);
    let rendered = render_diff(&f, DiffViewMode::Inline, true, true, 40);
    assert!(rendered.lines.len() > 3, "a 200-char token must break up");
    // Nothing overflows the content width.
    for l in plain(&rendered.lines).iter().skip(1) {
        assert!(l.chars().count() <= 40, "row overflows the pane: {l:?}");
    }
}

#[test]
fn no_row_overflows_the_pane_width_with_wrap_off_in_any_mode() {
    let long = "y".repeat(300);
    let f = file(vec![hunk(vec![
        line(LineOrigin::Delete, &long),
        line(LineOrigin::Add, &long),
    ])]);
    for mode in [DiffViewMode::Inline, DiffViewMode::Split] {
        let rendered = render_diff(&f, mode, true, false, 60);
        for l in plain(&rendered.lines).iter().skip(1) {
            assert!(
                l.chars().count() <= 60,
                "{mode:?} row overflows: {} chars",
                l.chars().count()
            );
        }
    }
}

#[test]
fn hunk_starts_point_at_each_hunks_header_row() {
    let f = file(vec![
        hunk(vec![line(LineOrigin::Add, "one")]),
        hunk(vec![line(LineOrigin::Add, "two")]),
    ]);
    let rendered = render_diff(&f, DiffViewMode::Inline, true, false, 80);
    assert_eq!(rendered.hunk_starts.len(), 2);
    let text = plain(&rendered.lines);
    for &start in &rendered.hunk_starts {
        assert!(
            text[start].starts_with("@@"),
            "hunk_starts must land on a header, found {:?}",
            text[start]
        );
    }
}

#[test]
fn tabs_expand_so_indented_code_stays_aligned() {
    let f = file(vec![hunk(vec![line(LineOrigin::Add, "\tindented")])]);
    let rendered = render_diff(&f, DiffViewMode::Inline, true, false, 80);
    assert!(!plain(&rendered.lines)[1].contains('\t'));
}
