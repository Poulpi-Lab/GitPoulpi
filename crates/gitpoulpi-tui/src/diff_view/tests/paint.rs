use super::*;
use crate::th;

// ---- the per-hunk button and the cursor band ----------------------

/// The button is painted at the right end of the header row, and the
/// offset it reports is where it actually landed - the caller
/// registers its hit rect from this, so the two cannot drift.
#[test]
fn the_hunk_button_reports_where_it_painted_itself() {
    let mut r = render_diff(&two_hunk_file(), DiffViewMode::Inline, true, false, 60);
    let header = r.hunk_starts[0];
    let before: String = r.lines[header]
        .spans
        .iter()
        .map(|s| s.content.as_ref())
        .collect();
    let (dx, bw) = hunk_button(&mut r.lines, header, 60, "Stage hunk").expect("it fits at 60");
    assert_eq!(bw as usize, "Stage hunk".len() + 2);
    assert!(dx + bw <= 60, "the button must stay inside the pane");
    let after: String = r.lines[header]
        .spans
        .iter()
        .map(|s| s.content.as_ref())
        .collect();
    assert!(after.starts_with(&before), "the header text is preserved");
    assert!(after.contains("Stage hunk"));
    // The reported offset is the column the label really starts at.
    let painted_at = after.find(" Stage hunk ").expect("painted");
    assert_eq!(painted_at, dx as usize, "got {after:?}");
    // Too narrow: nothing is painted rather than something drawn off
    // the right edge, invisible but still clickable.
    let mut narrow = render_diff(&two_hunk_file(), DiffViewMode::Inline, true, false, 20);
    let row = narrow.hunk_starts[0];
    assert_eq!(hunk_button(&mut narrow.lines, row, 20, "Stage hunk"), None);
}

/// The cursor and the selection are full-width bands: spans stop
/// where the text does, so without the padding the highlight would
/// end mid-row and read as a highlight on the text instead of on the
/// line.
#[test]
fn the_cursor_and_the_selection_paint_full_width_bands() {
    use ratatui::backend::TestBackend;
    use ratatui::widgets::Paragraph;
    use ratatui::Terminal;

    let width = 40u16;
    let mut r = render_diff(&two_hunk_file(), DiffViewMode::Inline, true, false, width);
    let rows = r.lines.len() as u16;
    apply_cursor(&mut r.lines, 0, width, 2, Some((1, 3)));
    let mut terminal = Terminal::new(TestBackend::new(width, rows)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(Paragraph::new(r.lines.clone()), frame.area());
        })
        .unwrap();
    let buffer = terminal.backend().buffer().clone();
    let bg_of = |x: u16, y: u16| buffer[(x, y)].style().bg;
    for x in 0..width {
        assert_eq!(
            bg_of(x, 2),
            Some(th().ui.selection),
            "cursor row col {x} is not part of the band"
        );
        assert_eq!(
            bg_of(x, 1),
            Some(th().diff.range_bg),
            "selected row col {x} is not part of the band"
        );
    }
    // An untouched row keeps the diff's own coloring (and is
    // certainly not wearing the cursor's).
    assert_ne!(bg_of(0, 0), Some(th().ui.selection));
    assert_ne!(bg_of(0, 0), Some(th().diff.range_bg));
}

/// The event loop paints only the visible window of a (cached) render:
/// absolute rows map into the window, rows outside it are skipped
/// without panicking, and the hunk button's rect is where it painted.
#[test]
fn the_visible_window_paints_cursor_and_button_at_window_rows() {
    let r = render_diff(&two_hunk_file(), DiffViewMode::Inline, true, false, 60);
    let mut state = DiffViewState::for_working(DiffSource::Unstaged, PathBuf::from("a"));
    state.scroll = r.hunk_starts[1] - 1; // window starts one row above hunk 1
    state.cursor = r.hunk_starts[1] + 1; // inside hunk 1
    state.selection_anchor = Some(0); // mostly above the window
    let inner = Rect::new(3, 7, 60, 3);
    let (lines, buttons) = visible_window(&r, &state, inner, None, None);
    assert_eq!(lines.len(), 3);
    let (button, rect) = *buttons.last().expect("the cursor's hunk shows its button");
    assert_eq!(button, DiffBodyButton::Hunk(HunkId(1)));
    assert_eq!(rect.y, inner.y + 1, "the header is the window's second row");
    let text: String = lines[1].spans.iter().map(|s| s.content.as_ref()).collect();
    assert_eq!(
        text.find(" Stage hunk ").map(|x| x as u16 + inner.x),
        Some(rect.x)
    );
    // The cursor band on window row 2, the selection on rows 0..=1.
    let bg = |row: usize| lines[row].spans.last().unwrap().style.bg;
    assert_eq!(bg(2), Some(th().ui.selection));
    assert_eq!(bg(0), Some(th().diff.range_bg));
    // A commit's diff has no button at all.
    state.source = DiffSource::Commit;
    assert!(visible_window(&r, &state, inner, None, None).1.is_empty());
}

/// On the Unstaged half the hovered hunk's header
/// carries `Discard hunk` beside `Stage hunk`, and the hovered changed
/// row its own `✖ Discard` and `+ Stage` - each rect exactly where
/// its label was painted. The Staged half gets no discard at all, and
/// a context row no row buttons.
#[test]
fn hovering_shows_discard_and_per_line_buttons_on_the_unstaged_half() {
    let r = render_diff(&two_hunk_file(), DiffViewMode::Inline, true, false, 60);
    let inner = Rect::new(2, 4, 60, r.lines.len() as u16);
    let text = |lines: &[Line<'static>], row: usize| -> String {
        lines[row]
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect()
    };
    let at = |lines: &[Line<'static>], rect: Rect, label: &str| {
        let row = text(lines, (rect.y - inner.y) as usize);
        let col = row.chars().collect::<Vec<_>>();
        let found: String = col[(rect.x - inner.x) as usize..][..rect.width as usize]
            .iter()
            .collect();
        assert_eq!(found, format!(" {label} "), "painted where it is clickable");
    };
    let del_row = r.hunk_starts[0] + 2; // "old a"
    let mut state = DiffViewState::for_working(DiffSource::Unstaged, PathBuf::from("a"));
    state.cursor = r.lines.len() - 1; // hunk 1, so hunk 0 is the mouse's

    let (lines, buttons) = visible_window(&r, &state, inner, Some(del_row), None);
    let kinds: Vec<DiffBodyButton> = buttons.iter().map(|(b, _)| *b).collect();
    assert_eq!(
        kinds,
        [
            DiffBodyButton::DiscardHunk(HunkId(0)),
            DiffBodyButton::Hunk(HunkId(0)),
            DiffBodyButton::DiscardLine(del_row),
            DiffBodyButton::Line(del_row),
        ]
    );
    at(&lines, buttons[0].1, discard_hunk_label());
    at(&lines, buttons[1].1, "Stage hunk");
    at(&lines, buttons[2].1, &line_discard_label());
    at(&lines, buttons[3].1, &line_stage_label());
    assert!(
        text(&lines, del_row).contains("old a"),
        "short text is kept"
    );

    // The Staged half: `Unstage hunk` and `− Unstage`, nothing else.
    state.source = DiffSource::Staged;
    let (lines, buttons) = visible_window(&r, &state, inner, Some(del_row), None);
    let kinds: Vec<DiffBodyButton> = buttons.iter().map(|(b, _)| *b).collect();
    assert_eq!(
        kinds,
        [
            DiffBodyButton::Hunk(HunkId(0)),
            DiffBodyButton::Line(del_row)
        ]
    );
    at(&lines, buttons[1].1, &line_unstage_label());

    // A context row shows no row buttons (the header's stay).
    state.source = DiffSource::Unstaged;
    let ctx_row = r.hunk_starts[0] + 1;
    let (_, buttons) = visible_window(&r, &state, inner, Some(ctx_row), None);
    assert!(buttons
        .iter()
        .all(|(b, _)| matches!(b, DiffBodyButton::Hunk(_) | DiffBodyButton::DiscardHunk(_))));
}

/// Each body button fills under the pointer (and only that one): the
/// maintainer reported Discard/Stage hunk and the line buttons never lit.
#[test]
fn the_hovered_diff_body_button_alone_changes_its_style() {
    let r = render_diff(&two_hunk_file(), DiffViewMode::Inline, true, false, 60);
    let inner = Rect::new(2, 4, 60, r.lines.len() as u16);
    let del_row = r.hunk_starts[0] + 2;
    let mut state = DiffViewState::for_working(DiffSource::Unstaged, PathBuf::from("a"));
    // Cursor away from the buttons' rows, then on the hunk header itself
    // (the cursor band recolors that row's spans).
    for cursor in [r.lines.len() - 1, r.hunk_starts[0]] {
        state.cursor = cursor;
        let styles = |hover: Option<DiffBodyButton>| -> Vec<Vec<ratatui::style::Style>> {
            let (lines, buttons) = visible_window(&r, &state, inner, Some(del_row), hover);
            buttons
                .iter()
                .map(|(_, rect)| {
                    let row = &lines[(rect.y - inner.y) as usize];
                    let mut x = 0u16;
                    let mut out = Vec::new();
                    for span in &row.spans {
                        let w = span.content.chars().count() as u16;
                        if x == rect.x - inner.x {
                            out.push(span.style);
                        }
                        x += w;
                    }
                    out
                })
                .collect()
        };
        let rest = styles(None);
        for (i, button) in [
            DiffBodyButton::DiscardHunk(HunkId(0)),
            DiffBodyButton::Hunk(HunkId(0)),
            DiffBodyButton::DiscardLine(del_row),
            DiffBodyButton::Line(del_row),
        ]
        .into_iter()
        .enumerate()
        {
            let lit = styles(Some(button));
            for (j, (a, b)) in rest.iter().zip(&lit).enumerate() {
                if i == j {
                    assert_ne!(a, b, "{button:?} must change when hovered");
                } else {
                    assert_eq!(a, b, "{button:?} must not light another button");
                }
            }
        }
    }
}

/// A changed row whose text reaches the right edge still gets its
/// buttons: the text is cut short under them (a header row, by
/// contrast, gets none rather than lose its `@@` text).
#[test]
fn row_buttons_overlay_a_long_row_but_never_a_header() {
    let long = "x".repeat(80);
    let mut lines = vec![Line::from(long.clone())];
    let placed = row_buttons(
        &mut lines,
        0,
        40,
        &[("+ Stage", th().ui.accent)],
        true,
        None,
    )
    .expect("overlaid on a long row");
    assert_eq!(placed, [(31, 9)]);
    let text: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
    assert_eq!(text.chars().count(), 40, "{text:?}");
    assert!(text.ends_with(" + Stage "), "{text:?}");
    let mut header = vec![Line::from(long)];
    assert_eq!(
        row_buttons(
            &mut header,
            0,
            40,
            &[("Stage hunk", th().ui.accent)],
            false,
            None
        ),
        None
    );
}

#[test]
fn the_highlight_band_tints_exactly_the_rows_of_its_lines_over_the_cursor() {
    let r = render_diff(&sample(), DiffViewMode::Inline, true, false, 40);
    let mut lines = r.lines.clone();
    let band = DiffHighlight {
        side: 1,
        from: 11,
        to: 12,
    };
    apply_cursor(&mut lines, 0, 40, 3, None);
    apply_highlight(&mut lines, 0, 40, &r.rows, Some(band));
    let bg = |l: &Line<'static>| l.spans[0].style.bg;
    // New lines 11 (the add) and 12 (context) are tinted, the cursor's
    // own band gives way on row 3; the delete (no new number) is not.
    assert_eq!(bg(&lines[1]), r.lines[1].spans[0].style.bg);
    assert_eq!(bg(&lines[2]), r.lines[2].spans[0].style.bg);
    assert_eq!(bg(&lines[3]), Some(th().diff.search_hit_bg));
    assert_eq!(bg(&lines[4]), Some(th().diff.search_hit_bg));
}
