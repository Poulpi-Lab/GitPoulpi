use super::*;

/// The File View fills the central column it is given --
/// bar on the first row, the file's lines in a bordered body with
/// the `✕ Esc` close on its top border - rather than a popup
/// centered on the whole window.
#[test]
fn the_file_view_draws_in_the_central_column_with_the_bar_and_a_close() {
    use gitpoulpi_core::domain::{Blob, BlobContent, BlobSource};
    use ratatui::backend::TestBackend;
    let request = FileInsightRequest {
        path: PathBuf::from("README.md"),
        at: oid(7),
        kind: FileInsightKind::Content,
        worktree: false,
        staged: false,
    };
    let mut insight = FileInsight::loading(request.clone());
    insight.accept(
        &request,
        crate::daemon_link::FileInsightPayload::Content(Blob {
            path: PathBuf::from("README.md"),
            source: BlobSource::Commit(oid(7)),
            size: 24,
            image: None,
            content: BlobContent::Text {
                lines: vec!["first line".into(), "second line".into()],
                crlf: false,
                final_newline: true,
            },
        }),
    );
    // A 70x12 window whose central column is x 10..60, y 2..12.
    let central = Rect::new(10, 2, 50, 10);
    let mut terminal = Terminal::new(TestBackend::new(70, 12)).unwrap();
    let mut drawn = None;
    terminal
        .draw(|frame| {
            drawn = Some(render_file_panel(
                frame,
                central,
                &mut insight,
                false,
                false,
                None,
                false,
                false,
            ));
        })
        .unwrap();
    let drawn = drawn.unwrap();
    let lines = buffer_text(terminal.backend().buffer());
    assert_eq!(
        drawn.bar_area,
        Rect::new(10, 2, 50, 1),
        "bar on the first row"
    );
    assert_eq!(drawn.body, Rect::new(10, 3, 50, 9), "body is the rest");
    assert!(
        lines[2].contains("File View") && lines[2].contains("Blame"),
        "{lines:#?}"
    );
    assert!(
        lines[3].contains("README.md"),
        "title on the body border: {lines:#?}"
    );
    assert!(lines[4].contains("first line"), "{lines:#?}");
    assert!(lines[5].contains("second line"), "{lines:#?}");
    // Nothing outside the central column was painted.
    for line in &lines {
        assert!(line[..10].trim().is_empty(), "left of the column: {line:?}");
    }
    let close = drawn.close.expect("a close target on the border");
    assert_eq!(close.y, 3);
    assert!(lines[3].contains("\u{2715}"), "{:?}", lines[3]);
    assert!(!lines[3].contains("Esc"), "{:?}", lines[3]);
}

/// The File View paints a text selection over the
/// selected chars only - not the line-number gutter, not the rest
/// of the row - and the selected row's own band does not hide it.
#[test]
fn the_file_view_highlights_the_selected_text_only() {
    use crate::text_select::{TextPos, TextSelection};
    use gitpoulpi_core::domain::{Blob, BlobContent, BlobSource};
    use ratatui::backend::TestBackend;
    let request = FileInsightRequest {
        path: PathBuf::from("README.md"),
        at: oid(7),
        kind: FileInsightKind::Content,
        worktree: false,
        staged: false,
    };
    let mut insight = FileInsight::loading(request.clone());
    insight.accept(
        &request,
        crate::daemon_link::FileInsightPayload::Content(Blob {
            path: PathBuf::from("README.md"),
            source: BlobSource::Commit(oid(7)),
            size: 24,
            image: None,
            content: BlobContent::Text {
                lines: vec!["first line".into(), "second line".into()],
                crlf: false,
                final_newline: true,
            },
        }),
    );
    // "line" of the first line through "sec" of the second.
    insight.text_sel = Some(TextSelection {
        side: 0,
        anchor: TextPos { line: 0, offset: 6 },
        head: TextPos { line: 1, offset: 2 },
    });
    let central = Rect::new(0, 0, 40, 8);
    let mut terminal = Terminal::new(TestBackend::new(40, 8)).unwrap();
    terminal
        .draw(|frame| {
            render_file_panel(
                frame,
                central,
                &mut insight,
                false,
                false,
                None,
                false,
                false,
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer();
    let lit = |y: u16| -> String {
        (0..40)
            .filter(|&x| buf[(x, y)].bg == th().ui.text_selection_bg)
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect()
    };
    // Bar row 0, top border row 1, text from row 2; the gutter is
    // "1 " (one digit plus a space) after the left border.
    assert_eq!(lit(2), "line ", "the chars plus the line break cell");
    assert_eq!(lit(3), "sec");
    assert_eq!(buf[(1, 2)].symbol(), "1");
    assert_ne!(
        buf[(1, 2)].bg,
        th().ui.text_selection_bg,
        "the gutter stays unlit"
    );
}

#[test]
fn the_file_bar_marks_the_open_kind_and_grays_diff_view_without_a_diff() {
    let bar = file_bar(FileInsightKind::Blame, false, false, None, false, 80);
    let labels: Vec<DiffBarButton> = bar.buttons.iter().map(|(_, _, b)| *b).collect();
    assert_eq!(
        labels,
        vec![
            DiffBarButton::FileView,
            DiffBarButton::DiffView,
            DiffBarButton::Blame,
            // The column's selector: one item, two hits.
            DiffBarButton::ColumnFile,
            DiffBarButton::ColumnAll,
        ]
    );
    let style_of = |text: &str| {
        bar.line
            .spans
            .iter()
            .find(|s| s.content.contains(text))
            .map(|s| s.style)
            .unwrap()
    };
    assert_eq!(
        style_of("Blame").bg,
        Some(th().ui.accent),
        "the open kind is lit"
    );
    assert!(
        style_of("Diff View").add_modifier.contains(Modifier::DIM),
        "no diff to go back to"
    );
    // Narrow: File View (the open kind) survives longest -- there is
    // there is no Close button on this bar, so the
    // width needed to keep one real button *and* the `⋯` is bigger
    // than it used to be (" File View " alone is already 11 wide).
    let narrow = file_bar(FileInsightKind::Content, true, false, None, false, 16);
    assert!(narrow
        .buttons
        .iter()
        .any(|(_, _, b)| *b == DiffBarButton::FileView));
}

/// Maintainer report: the central headers' `\u{2715} Esc` close button
/// (File View, and identically Diff view / Editor) hovered with weak
/// contrast next to the Settings gear and the app's own top-right
/// close cross, which both stand out well. Root cause: the close
/// button went through `hover_style` (a subtle background *lift* for
/// elements that already carry their own base style), while the gear
/// and app-close button go through `plain_button_style` (a plain
/// glyph filled solid on hover). They must render identically --
/// pinned here by comparing the actual rendered cell styles, not by
/// re-deriving the expectation from the same helper both call.
/// Falsified by reverting `render_file_panel`'s `close_style` back to
/// `hover_style(Style::default())`.
#[test]
fn the_close_esc_button_matches_the_gear_s_hover_style() {
    use gitpoulpi_core::domain::{Blob, BlobContent, BlobSource};
    use ratatui::backend::TestBackend;

    let request = FileInsightRequest {
        path: PathBuf::from("README.md"),
        at: oid(7),
        kind: FileInsightKind::Content,
        worktree: false,
        staged: false,
    };
    let mut insight = FileInsight::loading(request.clone());
    insight.accept(
        &request,
        crate::daemon_link::FileInsightPayload::Content(Blob {
            path: PathBuf::from("README.md"),
            source: BlobSource::Commit(oid(7)),
            size: 24,
            image: None,
            content: BlobContent::Text {
                lines: vec!["first line".into()],
                crlf: false,
                final_newline: true,
            },
        }),
    );
    // Same 70x12 window / central-column split as
    // `the_file_view_draws_in_the_central_column_with_the_bar_and_a_close`.
    let central = Rect::new(10, 2, 50, 10);
    let mut terminal = Terminal::new(TestBackend::new(70, 12)).unwrap();
    let mut close_rect = None;
    terminal
        .draw(|frame| {
            close_rect = render_file_panel(
                frame,
                central,
                &mut insight,
                false,
                false,
                None,
                false,
                true,
            )
            .close;
        })
        .unwrap();
    let close_rect = close_rect.expect("the hovered close button must register a rect");
    let close_buf = terminal.backend().buffer().clone();
    let close_x = (close_rect.x..close_rect.right())
        .find(|&x| close_buf[(x, close_rect.y)].symbol() == "\u{2715}")
        .expect("the close glyph must be drawn inside its own registered rect");
    let close_style = close_buf[(close_x, close_rect.y)].style();

    let mut gear_terminal = Terminal::new(TestBackend::new(60, 1)).unwrap();
    let mut hits = Vec::new();
    gear_terminal
        .draw(|frame| {
            render_gear(frame, Rect::new(50, 0, 3, 1), 20, true, &mut hits);
        })
        .unwrap();
    let gear_buf = gear_terminal.backend().buffer().clone();
    let gear_x = (0..60)
        .find(|&x| gear_buf[(x, 0)].symbol() == "\u{2699}")
        .expect("the gear glyph must be drawn");
    let gear_style = gear_buf[(gear_x, 0)].style();

    assert_eq!(
        close_style.bg, gear_style.bg,
        "hovered close button and hovered gear must share the same background"
    );
    assert_eq!(
        close_style.fg, gear_style.fg,
        "hovered close button and hovered gear must share the same foreground"
    );
    assert_eq!(close_style.bg, Some(th().ui.button_hover));
}
