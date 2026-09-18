//! The popup both second-level palette pickers (`App: Change Theme...`,
//! `App: Change Language...`) paint: a filter field, a ticked list with a
//! highlight, a hint band. Geometry and hit rects are the tab picker's.

use ratatui::style::Color;

/// One painted row.
pub(crate) struct PickerRow<'a> {
    pub(crate) label: &'a str,
    /// Ticked: the value in force.
    pub(crate) ticked: bool,
    /// Optional color chips, right-aligned.
    pub(crate) chips: &'a [Color],
}

/// What the popup shows besides its rows.
pub(crate) struct PickerView<'a> {
    pub(crate) field: crate::modes::MainField,
    pub(crate) title: String,
    pub(crate) draft: &'a str,
    pub(crate) selected: usize,
    pub(crate) empty: &'a str,
    pub(crate) hint: &'a str,
    /// A segmented prefilter under the field: its labels and the active one.
    pub(crate) segments: Option<(Vec<&'a str>, usize)>,
}

/// Paints the picker and returns the popup's rects, captured from this walk.
pub(crate) fn render(
    frame: &mut ratatui::Frame,
    view: &PickerView<'_>,
    rows_all: &[PickerRow<'_>],
    field_editor: &crate::text_edit::FieldEditor<crate::modes::MainField>,
) -> crate::chrome::PalettePopup {
    use crate::ui_kit::{centered_rect, truncate_with_ellipsis, ClippedRender};
    use crate::wide_edge::Clear;
    use crate::{text_field, th};
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::style::{Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph};
    let popup = centered_rect(46, 60, frame.area());
    frame.render_clipped(Clear, popup);
    let seg_h = u16::from(view.segments.is_some());
    let [input_area, seg_area, list_area, hint_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(seg_h),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .areas(popup);
    let mut segments = [Rect::ZERO; 4];
    if let Some((labels, active)) = &view.segments {
        let border = Style::default().fg(th().ui.accent);
        let mut spans = vec![Span::styled("\u{2502} ", border)];
        let mut x = seg_area.x + 2;
        for (i, label) in labels.iter().enumerate().take(4) {
            let text = format!(" {label} ");
            let w = crate::ui_kit::display_width(&text) as u16;
            let style = if i == *active {
                Style::default()
                    .fg(th().ui.on_accent)
                    .bg(th().ui.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(th().ui.muted)
            };
            segments[i] = Rect::new(x, seg_area.y, w, 1);
            spans.push(Span::styled(text, style));
            spans.push(Span::raw(" "));
            x += w + 1;
        }
        let used = (x - seg_area.x) as usize;
        let pad = (seg_area.width as usize).saturating_sub(used + 1);
        spans.push(Span::raw(" ".repeat(pad)));
        spans.push(Span::styled("\u{2502}", border));
        frame.render_clipped(Paragraph::new(Line::from(spans)), seg_area);
    }
    let draft_w = input_area.width.saturating_sub(2);
    field_editor.drawn(
        view.field,
        Rect::new(input_area.x + 1, input_area.y + 1, draft_w, 1),
        false,
    );
    let draft_line = text_field::edit_line(
        view.draft,
        field_editor.caret(view.field),
        draft_w as usize,
        true,
        text_field::EditLook::new(Style::default().bg(crate::input_focus_bg()), "_"),
    );
    frame.render_clipped(
        Paragraph::new(draft_line).block(
            Block::default()
                .title(format!(" {} ", view.title))
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th().ui.accent)),
        ),
        input_area,
    );

    let rows = list_area.height as usize;
    let window_start = crate::popup_list::View::window_start(view.selected, rows, rows_all.len());
    let inner_w = list_area.width.saturating_sub(2) as usize;
    let items: Vec<ListItem> = if rows_all.is_empty() {
        vec![ListItem::new(format!("  {}", view.empty)).style(Style::default().fg(th().ui.muted))]
    } else {
        rows_all
            .iter()
            .enumerate()
            .skip(window_start)
            .take(rows)
            .map(|(i, choice)| {
                let mut base = Style::default().fg(th().ui.text);
                if i == view.selected {
                    base = base.bg(th().ui.selection).add_modifier(Modifier::BOLD);
                }
                // A tick marks the theme in force, as the Settings combo does.
                let tick = if choice.ticked { "\u{2713} " } else { "  " };
                let chips_w = if choice.chips.is_empty() { 0 } else { 5 };
                let label =
                    truncate_with_ellipsis(choice.label, inner_w.saturating_sub(2 + chips_w));
                let mut spans = vec![Span::styled(format!("{tick}{label}"), base)];
                if !choice.chips.is_empty() {
                    let used = 2 + crate::ui_kit::display_width(&label);
                    let pad = inner_w.saturating_sub(used + chips_w - 1);
                    spans.push(Span::styled(" ".repeat(pad), base));
                    // The theme's own bg, text, accent and selection; the
                    // bg chip of a transparent theme is the terminal's.
                    for chip in choice.chips {
                        spans.push(Span::styled("\u{2588}", Style::default().fg(*chip)));
                    }
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
    crate::popup_list::View::paint_bar(frame.buffer_mut(), list_area, rows_all.len());
    let popup_rects = crate::chrome::PalettePopup {
        popup,
        list: list_area,
        window_start,
        count: rows_all.len(),
        segments,
    };
    crate::chrome::paint_popup_row_hover(frame.buffer_mut(), &popup_rects, view.selected);
    frame.render_clipped(
        Paragraph::new(view.hint)
            .style(Style::default().fg(th().ui.muted))
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                    .border_style(Style::default().fg(th().ui.accent)),
            ),
        hint_area,
    );
    popup_rects
}
