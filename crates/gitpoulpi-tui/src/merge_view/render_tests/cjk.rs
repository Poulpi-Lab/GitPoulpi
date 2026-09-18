//! Wide chars (CJK, emoji) in the A / B panes: the pane edge, and so the
//! divider between them, is a fixed column however far the text is
//! scrolled, and a wide char cut by the scroll edge leaves a blank.

use super::*;
use crate::merge_view::render::text_spans;

fn cjk_view() -> MergeView {
    let regions = vec![
        merged(&[
            "日本語のテキスト",
            "ascii line",
            "こんにちは世界、元気ですか",
        ]),
        conflict(
            &["x"],
            &["日本語のテキスト ours 😀 end", "second"],
            &["こんにちは theirs 日本語", "e\u{301}e\u{301} combining"],
        ),
        merged(&["末尾の行", "tail"]),
    ];
    let doc = MergeDocument {
        entry: ConflictEntry::from_stages("a.txt".into(), stage(1), stage(2), stage(3)).unwrap(),
        base: blob(1, projection(&regions, Base)),
        ours: blob(2, projection(&regions, Ours)),
        theirs: blob(3, projection(&regions, Theirs)),
        regions,
        labels: SideLabels::default(),
        worktree: WorktreeFileState::Markers,
        eol: Eol::Lf,
        bom: false,
        final_newline: true,
        encoding: None,
        marker_size: 7,
    };
    MergeView::new(MergeModel::new(doc))
}

fn plain(spans: &[ratatui::text::Span<'_>]) -> String {
    spans.iter().map(|s| s.content.as_ref()).collect()
}

#[test]
fn a_wide_char_cut_by_the_scroll_edge_is_a_blank_not_a_shift() {
    let (spans, used) = text_spans("日本語", None, 1, 10, None);
    assert_eq!(plain(&spans), " 本語");
    assert_eq!(used, 5);
    let (spans, used) = text_spans("日本語", None, 3, 10, None);
    assert_eq!(plain(&spans), " 語");
    assert_eq!(used, 3);
    let (spans, used) = text_spans("日本語", None, 2, 10, None);
    assert_eq!(plain(&spans), "本語");
    assert_eq!(used, 4);
}

#[test]
fn the_divider_never_moves_with_wide_chars_scrolled_or_not() {
    super::super::layout::tests_override::with(60, || {
        let mut v = cjk_view();
        let (buf, _) = draw(&mut v, 62, 22);
        let bars = |buf: &Buffer, x: u16| {
            (0..buf.area.height)
                .filter(|&y| buf[(x, y)].symbol() == "\u{2502}")
                .count()
        };
        // The divider column: the one holding the rule glyph on the most
        // rows.
        let col = (0..buf.area.width).max_by_key(|&x| bars(&buf, x)).unwrap();
        let rows = bars(&buf, col);
        assert!(rows >= 6, "a divider is drawn:\n{}", text(&buf));
        for h in 0..14usize {
            v.h_scroll = h;
            let (buf, _) = draw(&mut v, 62, 22);
            assert_eq!(bars(&buf, col), rows, "h={h}:\n{}", text(&buf));
        }
    });
}
