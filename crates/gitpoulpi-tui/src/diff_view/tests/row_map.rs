use super::*;

// ---- the row map, and everything staging reads off it -------------

/// Every consumer indexes `rows` with a row index taken from `lines`,
/// so the two must be the same length in every mode, at every width,
/// wrapped or not - including the two "nothing to show" notices.
#[test]
fn the_row_map_has_exactly_one_entry_per_rendered_row() {
    // Long enough to cover both hunks (new lines 10..=14) and more.
    let full: Vec<String> = (1..=20).map(|n| format!("some line {n} of text")).collect();
    for mode in [DiffViewMode::Inline, DiffViewMode::Split] {
        for hunks_only in [true, false] {
            for wrap in [false, true] {
                for width in [0u16, 20, 80] {
                    let r = render_diff_with(
                        &two_hunk_file(),
                        &DiffRenderOpts {
                            mode,
                            hunks_only,
                            show_whitespace: true,
                            word_wrap: wrap,
                            width,
                            h_offset: 0,
                            full_file: Some(&full),
                            lfs_raw: false,
                            lfs_local: None,
                            intraline: false,
                        },
                    );
                    assert_eq!(
                        r.lines.len(),
                        r.rows.len(),
                        "{mode:?} hunks_only={hunks_only} wrap={wrap} width={width}"
                    );
                }
            }
        }
    }
    let empty = render_diff(&file(vec![]), DiffViewMode::Inline, true, false, 80);
    assert_eq!(empty.lines.len(), empty.rows.len());
    assert_eq!(empty.rows[0].hunk, None, "a notice belongs to no hunk");
}

/// The header row is flagged and every body row names its hunk, which
/// is what "stage the hunk under the cursor" resolves through.
#[test]
fn each_row_names_its_hunk_and_only_the_header_is_flagged() {
    let r = render_diff(&two_hunk_file(), DiffViewMode::Inline, true, false, 80);
    assert_eq!(r.hunk_starts.len(), 2);
    for (n, &start) in r.hunk_starts.iter().enumerate() {
        assert!(r.rows[start].header, "row {start} should be a header");
        assert_eq!(r.rows[start].hunk, Some(HunkId(n as u32)));
    }
    // Rows between two headers belong to the earlier hunk.
    let second = r.hunk_starts[1];
    for row in (r.hunk_starts[0] + 1)..second {
        assert_eq!(r.rows[row].hunk, Some(HunkId(0)));
        assert!(!r.rows[row].header);
    }
    assert_eq!(hunk_at(&r.rows, second + 1), Some(HunkId(1)));
}

/// Only the changed lines are addressable, and they carry the index
/// the *backend's* hunk gives them - `LineSelector::line_indices` is
/// resolved against a diff the backend fetches itself, so any other
/// numbering stages the wrong lines.
#[test]
fn only_changed_lines_are_addressable_and_they_carry_backend_indices() {
    let r = render_diff(&two_hunk_file(), DiffViewMode::Inline, true, false, 80);
    // hunk 0: header, ctx, delete(1), add(2)
    assert_eq!(r.rows[1].lines, [None, None], "context is not selectable");
    assert_eq!(r.rows[2].lines, [Some(1), None], "the delete is line 1");
    assert_eq!(r.rows[3].lines, [Some(2), None], "the add is line 2");
}

/// The bug this guards: with the whitespace toggle off, `effective_
/// hunks` drops, rewrites and reorders lines, so a row's position in
/// the rewritten hunk is NOT its position in the backend's. Staging a
/// selection built from the rewritten positions would stage lines the
/// user never pointed at.
#[test]
fn hiding_whitespace_still_reports_the_backends_own_line_numbers() {
    // 0: context, 1: delete (whitespace-only), 2: delete (real),
    // 3: add (whitespace-only pair of 1), 4: add (real pair of 2).
    let f = file(vec![hunk(vec![
        line(LineOrigin::Context, "head"),
        line(LineOrigin::Delete, "same   "),
        line(LineOrigin::Delete, "old value"),
        line(LineOrigin::Add, "same"),
        line(LineOrigin::Add, "new value"),
    ])]);
    let r = render_diff(&f, DiffViewMode::Inline, false, false, 80);
    let addressed: Vec<u32> = r
        .rows
        .iter()
        .flat_map(|row| row.lines.into_iter().flatten())
        .collect();
    // The whitespace-only pair collapsed to context (not selectable);
    // what is left is the real delete (2) and the real add (4).
    assert_eq!(
        addressed,
        vec![2, 4],
        "expected the ORIGINAL indices 2 and 4, got {addressed:?}"
    );
}

/// A split row draws a delete and the add paired with it, so it has
/// to carry both: selecting it selects the change, not half of one.
#[test]
fn a_split_row_carries_both_sides_of_the_change() {
    let r = render_diff(&sample(), DiffViewMode::Split, true, false, 80);
    // Row 0 header, row 1 the shared context, row 2 the pair.
    assert_eq!(r.rows[1].lines, [None, None]);
    assert_eq!(r.rows[2].lines, [Some(1), Some(2)]);
}

/// A line selection names exactly one hunk. A range that reaches into
/// the next one keeps the first hunk's lines and reports the spill --
/// applying both would be wrong, not merely partial, because the
/// first patch moves the diff the second was numbered against.
#[test]
fn a_selection_is_confined_to_one_hunk_and_the_spill_is_reported() {
    let r = render_diff(&two_hunk_file(), DiffViewMode::Inline, true, false, 80);
    let (hunk, lines, spilled) = selected_lines(&r.rows, 0, r.rows.len() - 1).unwrap();
    assert_eq!(hunk, HunkId(0));
    // The second hunk's changed lines are at 3 and 4, so a selection
    // that leaked across the boundary would show up here.
    assert_eq!(lines, vec![1, 2], "only the first hunk's changed lines");
    assert!(spilled, "the range covered hunk 1 too");

    // Inside one hunk, nothing spills.
    let (hunk, lines, spilled) = selected_lines(&r.rows, 2, 3).unwrap();
    assert_eq!((hunk, spilled), (HunkId(0), false));
    assert_eq!(lines, vec![1, 2]);
}

/// A range over context only has nothing to stage. Reporting that
/// beats sending an empty selector, which `git apply` rejects with a
/// patch-format error no user could act on.
#[test]
fn a_context_only_selection_yields_no_lines() {
    let r = render_diff(&two_hunk_file(), DiffViewMode::Inline, true, false, 80);
    let (_, lines, _) = selected_lines(&r.rows, 1, 1).unwrap();
    assert!(lines.is_empty(), "got {lines:?}");
}

// -- overview ruler ----------------------------------------------------

#[test]
fn rows_know_whether_they_add_delete_or_modify() {
    let f = sample(); // ctx, del, add, ctx
    let inline = render_diff(&f, DiffViewMode::Inline, true, false, 80);
    let kinds: Vec<RowChange> = inline.rows.iter().map(|r| r.change).collect();
    assert_eq!(
        kinds,
        vec![
            RowChange::None, // the @@ header
            RowChange::None,
            RowChange::Del,
            RowChange::Add,
            RowChange::None
        ]
    );
    let split = render_diff(&f, DiffViewMode::Split, true, false, 80);
    assert!(
        split.rows.iter().any(|r| r.change == RowChange::Both),
        "a delete paired with an add is one modified row"
    );
    assert!(!split.rows.iter().any(|r| r.change == RowChange::Add));
}

#[test]
fn rows_carry_the_file_line_numbers_of_both_sides_in_both_layouts() {
    // Old 10..12 / new 10..12: context 10, delete old 11, add new 11, context 12.
    let diff = sample();
    let inline = render_diff(&diff, DiffViewMode::Inline, true, false, 80);
    // Row 0 is the `@@` header.
    let numbers: Vec<_> = inline.rows.iter().map(|r| r.numbers).collect();
    assert_eq!(
        numbers,
        vec![
            [None, None],
            [Some(10), Some(10)],
            [Some(11), None],
            [None, Some(11)],
            [Some(12), Some(12)],
        ]
    );
    assert_eq!(row_of_line(&inline.rows, 1, 11), Some(3));
    assert_eq!(row_of_line(&inline.rows, 0, 11), Some(2));
    assert_eq!(row_of_line(&inline.rows, 1, 99), None);

    // Split pairs the delete and the add on one row.
    let split = render_diff(&diff, DiffViewMode::Split, true, false, 100);
    assert_eq!(
        row_of_line(&split.rows, 0, 11),
        row_of_line(&split.rows, 1, 11)
    );
    assert_eq!(line_span(&split.rows, 0, split.rows.len()), Some((10, 12)));
}
