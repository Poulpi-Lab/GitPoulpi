//! The command palette popup: the filter box, the scrolling list and the hint band.

use super::super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_command_palette(
    frame: &mut Frame,
    draft: &str,
    selected: usize,
    palette_ctx: PaletteContext,
    field_editor: &crate::text_edit::FieldEditor<MainField>,
    journal_state: &gitpoulpi_core::domain::JournalState,
    undo_busy: bool,
    palette_popup: &mut Option<PalettePopup>,
) {
    // Command palette: a centered popup (same `centered_rect` pattern as the
    // help overlay above) with a filter input on top and a live-updating,
    // navigable list of matches below.
    let matches = palette_matches(draft, palette_ctx);
    // Tall: the catalogue is the whole app's command surface, so the popup
    // earns the room.
    let popup = centered_rect(62, 72, frame.area());
    frame.render_clipped(Clear, popup);
    // The hint band is 2 rows: one for the text, one for the
    // popup's bottom border, which only this last block draws.
    let [input_area, list_area, hint_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .areas(popup);
    // The query scrolls inside the box rather than growing through its right
    // border, which would clip the caret.
    let draft_w = input_area.width.saturating_sub(2);
    field_editor.drawn(
        MainField::Palette,
        Rect::new(input_area.x + 1, input_area.y + 1, draft_w, 1),
        false,
    );
    let draft_line = text_field::edit_line(
        draft,
        field_editor.caret(MainField::Palette),
        draft_w as usize,
        true,
        text_field::EditLook::new(Style::default().bg(crate::input_focus_bg()), "_"),
    );
    frame.render_clipped(
        Paragraph::new(draft_line).block(
            Block::default()
                .title(format!(
                    " {} ",
                    gitpoulpi_i18n::msg::palette::title(
                        matches.len(),
                        palette::PALETTE_COMMANDS.len() + palette::settings_listed_count()
                    )
                ))
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th().ui.accent)),
        ),
        input_area,
    );

    // The list is far longer than the popup, so it scrolls: keep `selected`
    // inside a window rather than rendering from row 0 (everything past the
    // first screenful would be unreachable by keyboard).
    // LEFT|RIGHT borders only, so every row of the area is a
    // content row.
    let rows = list_area.height as usize;
    let window_start = crate::popup_list::View::window_start(selected, rows, matches.len());
    // Room for " Ctrl+G P " plus a gap, carved off the right.
    let inner_w = list_area.width.saturating_sub(2) as usize;
    let palette_km = keymap::current();
    let shortcut_w = matches
        .iter()
        .skip(window_start)
        .take(rows)
        .map(|m| {
            palette::entry_shortcut(&palette_km, m.entry)
                .chars()
                .count()
        })
        .max()
        .unwrap_or(0);
    // The last cells: the "configure the shortcut" button.
    let label_w = inner_w.saturating_sub(shortcut_w + 2 + palette::KEY_BUTTON_W as usize);

    let items: Vec<ListItem> = if matches.is_empty() {
        vec![ListItem::new(format!(
            "  {}",
            gitpoulpi_i18n::msg::palette::NO_MATCH.get()
        ))
        .style(Style::default().fg(th().ui.muted))]
    } else {
        matches
            .iter()
            .enumerate()
            .skip(window_start)
            .take(rows)
            .map(|(i, m)| {
                let is_selected = i == selected;
                // An inapplicable command stays listed but
                // reads as inert -- same convention as the
                // diff bar's disabled buttons.
                let base = if m.unavailable.is_some() {
                    Style::default()
                        .fg(th().ui.muted)
                        .add_modifier(Modifier::DIM)
                } else {
                    Style::default().fg(th().ui.text)
                };
                let base = if is_selected {
                    base.bg(th().ui.selection).add_modifier(Modifier::BOLD)
                } else {
                    base
                };
                let mut label_text = m.entry.display_label().into_owned();
                // "Edit: Undo" + the operation it would undo.
                let suffix = match m.entry.command {
                    PaletteCommand::Undo => crate::undo_ui::palette_suffix(
                        journal_state,
                        gitpoulpi_core::domain::Direction::Undo,
                    ),
                    PaletteCommand::Redo => crate::undo_ui::palette_suffix(
                        journal_state,
                        gitpoulpi_core::domain::Direction::Redo,
                    ),
                    _ => None,
                };
                if let Some(op) = suffix {
                    label_text = format!("{label_text} {op}");
                }
                let label = truncate_with_ellipsis(&label_text, label_w);
                let pad = label_w.saturating_sub(crate::ui_kit::display_width(&label));
                // An AI command's sparkle keeps the AI purple
                // (dimmed with the rest when unavailable).
                let (spark, rest) = match label.strip_prefix(theme::AI_SPARK) {
                    Some(rest) => (Some(base.fg(th().status.ai)), rest.to_string()),
                    None => (None, label.clone()),
                };
                let mut spans = vec![Span::styled(" ", base)];
                if let Some(spark_style) = spark {
                    spans.push(Span::styled(theme::AI_SPARK, spark_style));
                }
                spans.push(Span::styled(format!("{rest}{}", " ".repeat(pad)), base));
                spans.push(Span::styled(
                    format!(
                        "{:>shortcut_w$} ",
                        palette::entry_shortcut(&palette_km, m.entry)
                    ),
                    if is_selected {
                        base
                    } else {
                        base.fg(th().ui.muted)
                    },
                ));
                if palette::bind_action(m.entry).is_some() {
                    // A plain button's colors, so it reads as clickable (the
                    // glyph alone is faint in many fonts).
                    spans.push(Span::styled(
                        palette::KEY_BUTTON,
                        Style::default().bg(th().ui.button).fg(th().ui.on_button),
                    ));
                } else {
                    spans.push(Span::styled(
                        " ".repeat(palette::KEY_BUTTON_W as usize),
                        base,
                    ));
                }
                ListItem::new(Line::from(spans))
            })
            .collect()
    };
    frame.render_clipped(
        List::new(items).block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(th().ui.accent)),
        ),
        list_area,
    );
    crate::popup_list::View::paint_bar(frame.buffer_mut(), list_area, matches.len());
    // Captured from the walk that just drew the rows, not recomputed from the
    // percentages afterwards: the `log_table` rule, one popup over.
    *palette_popup = Some(PalettePopup {
        popup,
        list: list_area,
        window_start,
        count: matches.len(),
        segments: [Rect::ZERO; 4],
    });
    if let Some(popup) = palette_popup.as_ref() {
        crate::chrome::paint_popup_row_hover(frame.buffer_mut(), popup, selected);
        // The key button under the pointer lights like any plain button, over
        // the row hover.
        if let Some((col, row)) = crate::chrome::popup_pointer() {
            let x = list_area.right().saturating_sub(1 + palette::KEY_BUTTON_W);
            let on_button = col >= x && col < x + palette::KEY_BUTTON_W;
            let entry = crate::chrome::palette_row_at(popup, col, row)
                .and_then(|i| matches.get(i))
                .filter(|m| palette::bind_action(m.entry).is_some());
            if on_button && entry.is_some() {
                frame.buffer_mut().set_style(
                    Rect::new(x, row, palette::KEY_BUTTON_W, 1),
                    Style::default()
                        .bg(th().ui.button_hover)
                        .fg(th().ui.on_button),
                );
            }
        }
    }

    // Why the highlighted row can't run, when it can't -- the
    // reason belongs next to the cursor, not hidden until you
    // press Enter and get a status-bar line.
    let hint = matches
        .get(selected)
        .and_then(|m| {
            // The journal's own words (what moved) for a
            // blocked Undo / Redo; the catalogue's otherwise.
            match (m.entry.command, m.unavailable) {
                (PaletteCommand::Undo, Some(r)) => Some(
                    crate::undo_ui::reason(
                        journal_state,
                        gitpoulpi_core::domain::Direction::Undo,
                        undo_busy,
                    )
                    .unwrap_or_else(|| r.to_string()),
                ),
                (PaletteCommand::Redo, Some(r)) => Some(
                    crate::undo_ui::reason(
                        journal_state,
                        gitpoulpi_core::domain::Direction::Redo,
                        undo_busy,
                    )
                    .unwrap_or_else(|| r.to_string()),
                ),
                (_, r) => r.map(str::to_string),
            }
        })
        .map(|r| format!(" \u{26A0} {r}"))
        .unwrap_or_else(|| {
            // The key button's keyboard twin, named where the eye is.
            let key = palette_km.shown(keymap::Action::PaletteConfigureShortcut);
            match matches.get(selected) {
                Some(m) if !key.is_empty() && palette::bind_action(m.entry).is_some() => {
                    format!(" {}", gitpoulpi_i18n::msg::palette::hint_with_shortcut(key))
                }
                _ => format!(" {}", gitpoulpi_i18n::msg::palette::HINT.get()),
            }
        });
    frame.render_clipped(
        Paragraph::new(truncate_with_ellipsis(
            &hint,
            hint_area.width.saturating_sub(1) as usize,
        ))
        .style(Style::default().fg(th().ui.muted))
        .block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(Style::default().fg(th().ui.accent)),
        ),
        hint_area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    /// The palette painted over a frame full of `X`: every cell of its rect
    /// (scrollbar column included) must be overwritten, and the cells after
    /// the shortcut column must be plain ASCII, which every terminal and font
    /// draws one cell wide (no VS16, no ambiguous width): WezTerm drew the
    /// old keyboard emoji button off its two cells, shifting the row and
    /// leaving what was behind the popup showing through next to the bar.
    #[test]
    fn the_palette_clears_its_whole_rect_and_ends_rows_in_single_width_glyphs() {
        for (w, h) in [(120u16, 40u16), (80, 24), (200, 60)] {
            let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
            let mut popup = None;
            let editor = crate::text_edit::FieldEditor::default();
            let journal = gitpoulpi_core::domain::JournalState::default();
            let frame = terminal
                .draw(|frame| {
                    let area = frame.area();
                    for y in 0..area.height {
                        for x in 0..area.width {
                            frame.buffer_mut()[(x, y)].set_symbol("X");
                        }
                    }
                    render_command_palette(
                        frame,
                        "",
                        0,
                        PaletteContext::default(),
                        &editor,
                        &journal,
                        false,
                        &mut popup,
                    );
                })
                .unwrap();
            let buf = frame.buffer;
            let popup = popup.expect("the palette recorded its popup");
            for y in popup.popup.y..popup.popup.bottom() {
                for x in popup.popup.x..popup.popup.right() {
                    assert_ne!(
                        buf[(x, y)].symbol(),
                        "X",
                        "{w}x{h}: ({x},{y}) of the popup was not painted"
                    );
                }
            }
            // The row tail: the key button, up to (not including) the border
            // column the scrollbar owns.
            let end = popup.list.right().saturating_sub(1);
            let tail_w = crate::palette::KEY_BUTTON_W + 1;
            for y in popup.list.y..popup.list.bottom() {
                for x in end.saturating_sub(tail_w)..end {
                    let s = buf[(x, y)].symbol();
                    assert!(
                        s.is_ascii(),
                        "{w}x{h}: ({x},{y}) {s:?} in the row tail is not plain ASCII"
                    );
                }
            }
        }
    }
}
