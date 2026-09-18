use super::*;

/// The panel's outer rect in these tests: the inner rect starts at
/// (1, 1).
fn body() -> ratatui::layout::Rect {
    ratatui::layout::Rect::new(0, 0, 80, 10)
}

fn select(insight: &mut FileInsight, from: (u16, u16), to: (u16, u16)) {
    let anchor = text_pos_at(insight, body(), from.0, from.1).unwrap();
    let head = text_pos_at(insight, body(), to.0, to.1).unwrap();
    insight.text_sel = Some(TextSelection {
        side: 0,
        anchor,
        head,
    });
}

/// Blame copies the code: the sha, author and date columns are never
/// part of a selection, even one dragged from inside them.
#[test]
fn a_blame_selection_copies_the_code_not_the_attribution_gutter() {
    let mut insight = blame_insight(3);
    select(&mut insight, (1, 1), (79, 2));
    assert_eq!(
        insight.yank().unwrap(),
        ("the selection", "line 0\nline 1".into())
    );
    // A partial drag inside the code column.
    let x0 = 1 + BLAME_GUTTER;
    select(&mut insight, (x0 + 2, 3), (x0 + 3, 3));
    assert_eq!(insight.yank().unwrap().1, "ne");
}

/// Scrolled sideways, and scrolled down: positions are the text's own.
#[test]
fn a_selection_under_both_scrolls_lands_on_the_real_chars() {
    let mut insight = blame_insight(20);
    if let FileInsightData::Blame(blame) = &mut insight.data {
        if let BlameContent::Lines(lines) = &mut blame.content {
            lines[12].content = "0123456789abcdef".to_string();
        }
    }
    insight.offset = 10;
    insight.h_offset = 4;
    let x0 = 1 + BLAME_GUTTER;
    // Screen row 3 is line 12; column 1 of the text is char 4.
    select(&mut insight, (x0 + 1, 3), (x0 + 3, 3));
    assert_eq!(insight.yank().unwrap().1, "456");
    // The selection is anchored in the text: scrolling keeps it.
    insight.offset = 0;
    insight.h_offset = 0;
    assert_eq!(insight.yank().unwrap().1, "456");
}

/// No selection: `y` copies the selected row's text.
#[test]
fn without_a_selection_the_selected_row_is_copied() {
    let mut insight = blame_insight(3);
    insight.selected = 2;
    assert_eq!(insight.yank().unwrap(), ("the line", "line 2".into()));
}

/// Nothing to select on a notice row.
#[test]
fn a_loading_view_has_no_text_to_select() {
    let insight = FileInsight::loading(request(FileInsightKind::Content));
    assert!(insight_segs(&insight, 80, 10).is_empty());
    assert_eq!(text_pos_at(&insight, body(), 5, 1), None);
    assert!(insight.yank().is_err());
}
