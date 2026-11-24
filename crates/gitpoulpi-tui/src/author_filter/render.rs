//! Paints the author popup under the Author column's header.

use super::{AuthorFilterPopup, AuthorRow};
use crate::modes::MainField;
use crate::ui_kit::{display_width, truncate_with_ellipsis, ClippedRender};
use crate::{text_field, th};
use gitpoulpi_i18n::msg::author_filter as m;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph};

/// The popup's width in cells (clamped to the screen).
const WIDTH: u16 = 54;
/// The most author rows shown before the list scrolls.
const MAX_ROWS: usize = 10;

/// Paints the popup and returns its rects, captured from this walk. It hangs
/// under the Author header (flipping above when the window has no room below);
/// with no Author column on screen it is centered.
pub(crate) fn render(
    frame: &mut ratatui::Frame,
    popup: &AuthorFilterPopup,
    field_editor: &crate::text_edit::FieldEditor<MainField>,
) -> crate::chrome::PalettePopup {
    use crate::wide_edge::Clear;
    let rows_all = popup.rows();
    let area = frame.area();
    let want_rows = rows_all.len().clamp(2, MAX_ROWS + 1) as u16;
    // field box (3) + list + hint band (2)
    let height = (3 + want_rows + 2).min(area.height);
    let width = WIDTH.min(area.width);
    let rect = match super::anchor() {
        Some(a) => crate::ui_kit::anchored_popup(area, a.x, a.y, width, height),
        None => crate::ui_kit::centered_rect(width.max(1) * 100 / area.width.max(1), 60, area),
    };
    frame.render_clipped(Clear, rect);
    let [input_area, list_area, hint_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .areas(rect);

    let draft_w = input_area.width.saturating_sub(2);
    field_editor.drawn(
        MainField::AuthorFilter,
        Rect::new(input_area.x + 1, input_area.y + 1, draft_w, 1),
        false,
    );
    let draft_line = if popup.draft.is_empty() {
        Line::from(Span::styled(
            m::PLACEHOLDER.get().to_string(),
            Style::default().fg(th().ui.muted),
        ))
    } else {
        text_field::edit_line(
            &popup.draft,
            field_editor.caret(MainField::AuthorFilter),
            draft_w as usize,
            true,
            text_field::EditLook::new(Style::default().bg(crate::input_focus_bg()), "_"),
        )
    };
    frame.render_clipped(
        Paragraph::new(draft_line).block(
            Block::default()
                .title(format!(" {} ", m::TITLE.get()))
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th().ui.accent)),
        ),
        input_area,
    );

    let rows = list_area.height as usize;
    let window_start = crate::popup_list::View::window_start(popup.selected, rows, rows_all.len());
    let inner_w = list_area.width.saturating_sub(2) as usize;
    let items: Vec<ListItem> = if rows_all.len() <= 1 && !popup.draft.trim().is_empty() {
        // Only the Clear row left: the field matches nobody.
        let mut v = vec![clear_item(popup.selected == 0, inner_w)];
        v.push(
            ListItem::new(format!("  {}", m::EMPTY.get()))
                .style(Style::default().fg(th().ui.muted)),
        );
        v.into_iter().take(rows).collect()
    } else {
        rows_all
            .iter()
            .enumerate()
            .skip(window_start)
            .take(rows)
            .map(|(i, row)| match row {
                AuthorRow::Clear => clear_item(i == popup.selected, inner_w),
                AuthorRow::Author(e) => {
                    let Some(entry) = popup.entry(*e) else {
                        return ListItem::new("");
                    };
                    author_item(
                        entry,
                        popup.chosen.contains(&entry.key),
                        i == popup.selected,
                        popup.avatars,
                        inner_w,
                    )
                }
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
    crate::popup_list::View::paint_bar(frame.buffer_mut(), list_area, rows_all.len());
    let rects = crate::chrome::PalettePopup {
        popup: rect,
        list: list_area,
        window_start,
        count: rows_all.len(),
        segments: [Rect::ZERO; 5],
    };
    crate::chrome::paint_popup_row_hover(frame.buffer_mut(), &rects, popup.selected);
    frame.render_clipped(
        Paragraph::new(m::HINT.get().to_string())
            .style(Style::default().fg(th().ui.muted))
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                    .border_style(Style::default().fg(th().ui.accent)),
            ),
        hint_area,
    );
    crate::chrome::paint_popup_cross(frame.buffer_mut(), rect);
    rects
}

fn selected_style(selected: bool) -> Style {
    let base = Style::default().fg(th().ui.text);
    if selected {
        base.bg(th().ui.selection).add_modifier(Modifier::BOLD)
    } else {
        base
    }
}

/// The Clear filter row: a cross, then the label.
fn clear_item(selected: bool, inner_w: usize) -> ListItem<'static> {
    let base = selected_style(selected);
    let label = truncate_with_ellipsis(m::CLEAR.get(), inner_w.saturating_sub(2));
    ListItem::new(Line::from(vec![
        Span::styled("\u{2715} ", base.fg(th().status.danger)),
        Span::styled(label, base),
    ]))
}

/// `✓ [AE] Alice Example  alice@example.com`: the tick of a chosen author, one
/// avatar, the name, the email muted after it.
fn author_item(
    entry: &super::AuthorEntry,
    chosen: bool,
    selected: bool,
    avatars: bool,
    inner_w: usize,
) -> ListItem<'static> {
    let base = selected_style(selected);
    let tick = if chosen { "\u{2713} " } else { "  " };
    let mut spans = vec![Span::styled(
        tick,
        if chosen {
            base.fg(th().ui.accent)
        } else {
            base
        },
    )];
    let mut used = 2;
    if avatars {
        let (text, color) = crate::avatar::fallback_text(
            gitpoulpi_config::AvatarFallback::Initials,
            &entry.name,
            &entry.email,
            usize::from(crate::avatar::SLOT_COLS),
        );
        used += display_width(&text) + 1;
        spans.push(Span::styled(text, base.fg(color)));
        spans.push(Span::styled(" ", base));
    }
    let room = inner_w.saturating_sub(used);
    let name = truncate_with_ellipsis(&entry.name, room);
    used += display_width(&name);
    spans.push(Span::styled(name, base));
    let left = inner_w.saturating_sub(used + 2);
    if !entry.email.is_empty() && left >= 4 {
        let email = truncate_with_ellipsis(&entry.email, left);
        spans.push(Span::styled("  ", base));
        spans.push(Span::styled(email, base.fg(th().ui.muted)));
    }
    ListItem::new(Line::from(spans))
}
