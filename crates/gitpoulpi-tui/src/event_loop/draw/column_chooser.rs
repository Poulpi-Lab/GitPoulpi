//! The Log's columns chooser (right-click the header, or the palette): a
//! checklist popup over the Log pane, one row per column, toggled by a click
//! or by Space / Enter under the keyboard cursor, with the app's close cross
//! on its title. `Message` is always shown (`LogColumn::hideable`), so it is
//! listed dimmed with no hit target: the popup still reads as every column.

use super::super::*;

/// Paints the chooser and registers its hit targets (the cross, then the
/// rows) at the front of `frame_hits`.
pub(super) fn paint(
    frame: &mut Frame,
    log_area: Rect,
    hidden: &std::collections::HashSet<LogColumn>,
    cursor: Option<usize>,
    hovered_target: Option<&HitTarget>,
    frame_hits: &mut Vec<(Rect, HitTarget)>,
) {
    let title = gitpoulpi_i18n::msg::log_draw::COLUMNS_TITLE.get();
    // Wide enough for the title and the cross beside it.
    let want_w = (display_width(title) as u16 + 8)
        .max(30)
        .min(log_area.width);
    let want_h = (LogColumn::ALL.len() as u16 + 2).min(log_area.height);
    let popup = Rect::new(
        log_area.x + (log_area.width - want_w) / 2,
        log_area.y + (log_area.height - want_h) / 2,
        want_w,
        want_h,
    );
    frame.render_clipped(Clear, popup);
    let hovered_item = match hovered_target {
        Some(HitTarget::ColumnMenuItem(col)) => Some(*col),
        _ => None,
    };
    let items: Vec<ListItem> = LogColumn::ALL
        .iter()
        .map(|&col| {
            let mark = crate::dialog_kit::checkbox_mark(!hidden.contains(&col));
            if col.hideable() {
                let style = if cursor.and_then(|i| LogColumn::ALL.get(i)) == Some(&col) {
                    Style::default().bg(th().ui.selection)
                } else if hovered_item == Some(col) {
                    hover_style(Style::default())
                } else {
                    Style::default()
                };
                ListItem::new(format!(" {mark} {}", col.title())).style(style)
            } else {
                ListItem::new(format!(
                    " {mark} {} {}",
                    col.title(),
                    gitpoulpi_i18n::msg::log_draw::ALWAYS_SHOWN.get()
                ))
                .style(Style::default().fg(th().ui.muted))
            }
        })
        .collect();
    frame.render_clipped(
        List::new(items).block(
            Block::default()
                .title(title)
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th().ui.accent))
                .style(Style::default().bg(th().ui.panel)),
        ),
        popup,
    );
    // The close cross, on the title's border like every closable popup.
    if let Some(cross) = crate::chrome::close_cross::paint(
        frame.buffer_mut(),
        popup,
        matches!(hovered_target, Some(HitTarget::ColumnMenuClose)),
    ) {
        frame_hits.insert(0, (cross, HitTarget::ColumnMenuClose));
    }
    for (i, &col) in LogColumn::ALL.iter().enumerate() {
        if !col.hideable() {
            continue;
        }
        let item_row = popup.y + 1 + i as u16;
        if item_row >= popup.y + popup.height.saturating_sub(1) {
            break;
        }
        frame_hits.insert(
            0,
            (
                Rect::new(popup.x + 1, item_row, popup.width.saturating_sub(2), 1),
                HitTarget::ColumnMenuItem(col),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn drawn(
        hidden: &std::collections::HashSet<LogColumn>,
    ) -> (Vec<String>, Vec<(Rect, HitTarget)>) {
        let mut terminal = Terminal::new(TestBackend::new(100, 20)).unwrap();
        let mut hits = Vec::new();
        terminal
            .draw(|frame| {
                let area = frame.area();
                paint(frame, area, hidden, Some(0), None, &mut hits);
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let lines = (0..buf.area.height)
            .map(|y| {
                (0..buf.area.width)
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect::<String>()
            })
            .collect();
        (lines, hits)
    }

    /// Red when a row draws anything but the shared checkbox marks (the file
    /// glyph the maintainer's font showed), loses a row, or the cross is missing
    /// or is not what a click on it would hit.
    #[test]
    fn rows_use_the_shared_checkbox_marks_and_the_title_has_a_clickable_cross() {
        let hidden: std::collections::HashSet<LogColumn> = [LogColumn::Sha].into();
        let (lines, hits) = drawn(&hidden);
        let text = lines.join("\n");
        assert!(
            !text.contains('\u{2611}') && !text.contains('\u{2610}'),
            "{text}"
        );
        for col in LogColumn::ALL {
            let want = crate::dialog_kit::checkbox_mark(!hidden.contains(&col));
            assert!(
                text.contains(&format!("{want} {}", col.title())),
                "{} row\n{text}",
                col.title()
            );
        }
        assert!(
            text.contains("[ ] SHA") && text.contains("[x] Author"),
            "{text}"
        );
        assert!(text.contains("(always shown)"), "{text}");
        // The cross is on the title row, and its hit rect covers it.
        let (rect, _) = hits
            .iter()
            .find(|(_, t)| matches!(t, HitTarget::ColumnMenuClose))
            .expect("a close target");
        assert!(
            lines[rect.y as usize].contains('\u{2715}'),
            "the cross on the frame row\n{text}"
        );
        // One target per toggleable column, none for Message.
        let items: Vec<LogColumn> = hits
            .iter()
            .filter_map(|(_, t)| match t {
                HitTarget::ColumnMenuItem(c) => Some(*c),
                _ => None,
            })
            .collect();
        assert_eq!(items.len(), LogColumn::ALL.len() - 1);
        assert!(!items.contains(&LogColumn::Message));
    }
}
