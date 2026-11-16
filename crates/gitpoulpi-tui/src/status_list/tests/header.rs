use super::*;

fn draw_status_toolbar(
    width: u16,
    hovered: Option<&HitTarget>,
) -> (
    String,
    ratatui::buffer::Buffer,
    Vec<(Rect, HitTarget)>,
    Option<OverflowSlot>,
) {
    draw_status_toolbar_with(width, hovered, false)
}

fn draw_status_toolbar_with(
    width: u16,
    hovered: Option<&HitTarget>,
    has_unstaged_changes: bool,
) -> (
    String,
    ratatui::buffer::Buffer,
    Vec<(Rect, HitTarget)>,
    Option<OverflowSlot>,
) {
    use ratatui::backend::TestBackend;
    let area = Rect::new(0, 0, width, 1);
    let mut terminal = Terminal::new(TestBackend::new(width, 1)).unwrap();
    let mut out = None;
    terminal
        .draw(|frame| {
            let (line, hits, overflow) = status_toolbar_row(
                area,
                FileListMode::Tree,
                Some(true),
                None,
                hovered,
                has_unstaged_changes,
            );
            frame.render_clipped(Paragraph::new(line), area);
            out = Some((hits, overflow));
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let text = (0..width)
        .map(|x| buf[(x, 0)].symbol().to_string())
        .collect();
    let (hits, overflow) = out.unwrap();
    (text, buf, hits, overflow)
}

fn paint_header(width: u16, ai: bool) -> (ratatui::buffer::Buffer, Vec<(Rect, HitTarget)>) {
    use ratatui::backend::TestBackend;
    let mut terminal = ratatui::Terminal::new(TestBackend::new(width, 3)).unwrap();
    let mut hits = Vec::new();
    terminal
        .draw(|frame| {
            render_status_header_buttons(frame, Rect::new(0, 0, width, 3), ai, None, &mut hits);
        })
        .unwrap();
    (terminal.backend().buffer().clone(), hits)
}

/// The pane says the repository is conflicted in its own title - red
/// `!` markers on individual rows are easy to read past, and reading
/// past them is how a file full of `<<<<<<<` gets committed.
#[test]
fn the_third_panes_title_announces_conflicts_and_is_otherwise_unchanged() {
    assert_eq!(
        working_changes_title(3, "main", 0),
        " 3 file changes on main "
    );
    let conflicted = working_changes_title(3, "main", 2);
    assert!(conflicted.contains("2 CONFLICTED"), "got: {conflicted}");
    assert!(conflicted.contains("3 file changes on main"));
}

/// The status sub-pane's block title and its bulk stage/unstage button
/// share one row: the title starts just past the left border, the
/// button is anchored to the right one. The button used to be gated on
/// its own label alone, so between roughly 22 and 40 columns the two
/// overlapped and the header read `Unstaged File Stage all changes`.
#[test]
fn the_status_section_title_never_collides_with_its_bulk_button() {
    for section in StatusSection::ORDER {
        let title = format!("\u{25BE} {} (3)", section.title());
        let bulk = section.bulk_label();
        let mut ever_drawn = false;
        for width in 0..=120u16 {
            let area = Rect::new(0, 0, width, 6);
            let Some(button) = status_bulk_button_rect(area, &title, bulk) else {
                continue;
            };
            ever_drawn = true;
            // The title is rendered by the `Block`, which starts it one
            // cell past the left border.
            let title_end = area.x + 1 + display_width(&title) as u16;
            assert!(
                button.x >= title_end,
                "at width {width} the button starts at {} but the title \
                 [{title}] runs to {title_end}",
                button.x
            );
            assert!(
                button.x + button.width < area.x + area.width,
                "at width {width} the button runs into the right border"
            );
        }
        assert!(
            ever_drawn,
            "the button must still appear at some width, or this \
             assertion passes for the wrong reason"
        );
    }
}

/// The rendered half: the header row of a narrow sub-pane must still
/// spell its own title.
#[test]
fn a_narrow_status_section_header_still_spells_its_title() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    for section in StatusSection::ORDER {
        let title = format!("\u{25BE} {} (3)", section.title());
        let bulk = section.bulk_label();
        for width in 16..=80u16 {
            let area = Rect::new(0, 0, width, 5);
            let mut terminal = Terminal::new(TestBackend::new(width, area.height)).unwrap();
            terminal
                .draw(|frame| {
                    frame.render_clipped(
                        Block::default()
                            .title(title.clone())
                            .border_type(BorderType::Rounded)
                            .borders(Borders::ALL),
                        area,
                    );
                    if let Some(button) = status_bulk_button_rect(area, &title, bulk) {
                        frame.render_clipped(
                            Paragraph::new(Line::from(Span::raw(format!(" {bulk} ")))),
                            button,
                        );
                    }
                })
                .unwrap();
            let buf = terminal.backend().buffer().clone();
            let row: String = (0..width)
                .map(|x| buf[(x, 0)].symbol().to_string())
                .collect::<Vec<_>>()
                .concat();
            // Whatever of the title the block had room for must survive
            // intact -- the button must never be painted over it.
            let visible: String = title
                .chars()
                .scan(0usize, |w, c| {
                    *w += display_width(&c.to_string());
                    (*w <= area.width.saturating_sub(2) as usize).then_some(c)
                })
                .collect();
            assert!(
                row.contains(&visible),
                "at width {width} the header reads [{row}], which has lost \
                 [{visible}]"
            );
        }
    }
}

#[test]
fn the_working_changes_row_keeps_collapse_all_when_it_fits() {
    let (text, _, hits, overflow) = draw_status_toolbar(40, None);
    assert_eq!(text.trim_end(), " Path  Tree [m]    Collapse all");
    assert!(overflow.is_none());
    assert!(hits
        .iter()
        .any(|(_, t)| *t == HitTarget::CollapseAllButton { collapse: true }));
}

#[test]
fn a_narrow_working_changes_row_puts_collapse_all_behind_the_overflow() {
    let (text, idle, hits, overflow) = draw_status_toolbar(20, None);
    assert_eq!(text.trim_end(), " Path  Tree   \u{22EF}");
    let (rect, hidden) = overflow.expect("an overflow button");
    assert_eq!(hidden, vec![OverflowItem::CollapseAll { collapse: true }]);
    let more = HitTarget::BarOverflow(OverflowBar::StatusToolbar);
    assert_eq!(find_hit_target(&hits, rect.x + 1, 0), Some(more.clone()));
    // Hover: the same lift Collapse all itself gets.
    let (_, hovered, _, _) = draw_status_toolbar(20, Some(&more));
    assert_ne!(idle[(rect.x + 1, 0)].style().bg, Some(th().ui.hover));
    assert_eq!(hovered[(rect.x + 1, 0)].style().bg, Some(th().ui.hover));
}

#[test]
fn a_section_header_too_narrow_for_its_bulk_label_gets_an_overflow_instead() {
    let title = "\u{25BE} Unstaged Files (3)";
    let bulk = StatusSection::Unstaged.bulk_label();
    // Room for the full label: the label, as before.
    let wide = Rect::new(0, 0, 60, 5);
    let (rect, is_more) = status_bulk_button_layout(wide, title, bulk).unwrap();
    assert!(!is_more);
    assert_eq!(rect.width, display_width(bulk) as u16 + 2);
    // Room for the ⋯ only: the ⋯, right-aligned, never over the title.
    let narrow = Rect::new(0, 0, 26, 5);
    let (rect, is_more) = status_bulk_button_layout(narrow, title, bulk).unwrap();
    assert!(is_more);
    assert_eq!(rect, Rect::new(26 - 3 - 1, 0, 3, 1));
    assert!(rect.x > display_width(title) as u16);
    // Not even that: a compact glyph overlays the title's end.
    let (rect, is_more) = status_bulk_button_layout(Rect::new(0, 0, 22, 5), title, bulk).unwrap();
    assert!(!is_more);
    assert_eq!(rect, Rect::new(22 - 3 - 1, 0, 3, 1));
    // Only a pane too small to hold the glyph loses it.
    assert!(status_bulk_button_layout(Rect::new(0, 0, 4, 5), title, bulk).is_none());
}

/// Trash at the far left in red, AI Explain at the far right (purple
/// with AI on, gray with it off); both hit rects come from the paint.
#[test]
fn the_header_has_a_red_trash_far_left_and_an_explain_sparkle_far_right() {
    let (buf, hits) = paint_header(60, true);
    let d = hits
        .iter()
        .find(|(_, t)| *t == HitTarget::StatusDiscardAllButton)
        .unwrap()
        .0;
    let e = hits
        .iter()
        .find(|(_, t)| *t == HitTarget::StatusExplainButton)
        .unwrap()
        .0;
    // The pane's collapse chevron sits at x = 1: two empty cells between
    // it and the button, so one cannot be clicked for the other.
    assert_eq!(d.x, 4);
    assert!(
        buf[(d.x + 1, 0)].symbol().starts_with('\u{2717}'),
        "the heavy cross, which every font draws"
    );
    assert_eq!(d.width, 3);
    assert_eq!(
        1 + HEADER_LEFT_RESERVE,
        d.right(),
        "the title starts after it"
    );
    assert_eq!(buf[(d.x + 1, 0)].fg, th().status.danger);
    assert_eq!(e.right(), 59);
    assert_eq!(buf[(e.x + 1, 0)].fg, th().status.ai);
    // AI off: no Explain button, no hit rect for it.
    let (_, off_hits) = paint_header(60, false);
    assert!(off_hits
        .iter()
        .all(|(_, t)| *t != HitTarget::StatusExplainButton));
    assert_eq!(off_hits.len(), 1);
    // Narrow: the Explain icon keeps its place (the title is clipped before
    // it), and only a pane with no room beside the trash loses it.
    let (_, hits) = paint_header(30, true);
    assert_eq!(hits.len(), 2);
    let (_, hits) = paint_header(16, true);
    assert_eq!(hits.len(), 1);
}

/// Maintainer U1: Discard All and Explain must visibly light on hover.
#[test]
fn the_header_buttons_change_style_when_hovered() {
    use ratatui::backend::TestBackend;
    for target in [
        HitTarget::StatusDiscardAllButton,
        HitTarget::StatusExplainButton,
    ] {
        let paint = |hovered: Option<&HitTarget>, ai: bool| {
            let mut terminal = ratatui::Terminal::new(TestBackend::new(60, 3)).unwrap();
            let mut hits = Vec::new();
            terminal
                .draw(|frame| {
                    render_status_header_buttons(
                        frame,
                        Rect::new(0, 0, 60, 3),
                        ai,
                        hovered,
                        &mut hits,
                    );
                })
                .unwrap();
            let rect = hits.iter().find(|(_, t)| *t == target).unwrap().0;
            terminal.backend().buffer()[(rect.x + 1, 0)].style()
        };
        let modes: &[bool] = if target == HitTarget::StatusExplainButton {
            &[true]
        } else {
            &[true, false]
        };
        for &ai in modes {
            assert_ne!(
                paint(None, ai),
                paint(Some(&target), ai),
                "{target:?} ai={ai}"
            );
        }
    }
}
