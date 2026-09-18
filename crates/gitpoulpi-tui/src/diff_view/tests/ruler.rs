use super::*;

#[test]
fn the_ruler_marks_changes_the_cursor_and_every_occurrence_of_the_selected_text() {
    use crate::scroll_marks::Mark;
    let f = file(vec![hunk(vec![
        line(LineOrigin::Context, "let value = 1;"),
        line(LineOrigin::Delete, "old"),
        line(LineOrigin::Add, "let value = 2;"),
        line(LineOrigin::Context, "nothing here"),
        line(LineOrigin::Context, "value again"),
    ])]);
    let r = render_diff(&f, DiffViewMode::Inline, true, false, 80);
    let mut state = DiffViewState::for_commit(test_oid(), 0, PathBuf::from("f.rs"));
    state.hunks_only = false;
    state.cursor = 4;
    // Select "value" on the first text line (row 1: row 0 is the header).
    state.text_sel = Some(text_select::TextSelection {
        side: 0,
        anchor: text_select::TextPos { line: 0, offset: 4 },
        head: text_select::TextPos { line: 0, offset: 8 },
    });
    let marks = ruler_marks(&r, &state);
    assert!(marks.contains(&(2, Mark::Del)));
    assert!(marks.contains(&(3, Mark::Add)));
    assert!(marks.contains(&(4, Mark::Cursor)));
    let occ: Vec<usize> = marks
        .iter()
        .filter(|(_, m)| *m == Mark::Occurrence)
        .map(|(row, _)| *row)
        .collect();
    assert_eq!(occ, vec![1, 3, 5], "rows holding `value`: {marks:?}");

    // Hunks-only mode has no ruler.
    state.hunks_only = true;
    assert!(ruler_marks(&r, &state).is_empty());
}
