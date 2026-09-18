//! One settings row, laid out: the label in a column, the control as a
//! control (a band, a checkbox, plain muted text) at the page's value
//! column, the help and warnings under the value.
//!
//! The value column is one per page (`set_label_col`), so every value starts
//! at the same cell and the eye needs no dot leader to pair a label with its
//! value. Below `label_col + 24` columns the row stacks: label above, control
//! below, indented by two.

use super::input::flash_unselected;
use super::*;
use crate::th;
use gitpoulpi_i18n::msg::settings_screen as screen;

/// Help and warning lines wrap at this width (readable); controls go to the
/// right edge of the body.
const HELP_CAP: usize = 100;
/// Prose (notes) keeps the older, easier-to-read line length.
const PROSE_CAP: usize = 100;
/// The narrowest room a control needs beside the label column.
const MIN_CONTROL: usize = 24;
const NUMBER_BAND: usize = 10;
/// The narrowest a pick-only choice band gets, so a short "off / ask / auto"
/// list still reads as a control next to the other rows.
const MIN_CHOICE_BAND: usize = 20;
/// A choice band's chrome: one blank each side, the arrow and its blank.
const CHOICE_CHROME: usize = 4;
/// A label wraps onto at most this many lines.
const LABEL_LINES: usize = 2;

impl PreferencesState {
    /// The label as drawn (the AI test button carries the sparkle).
    fn row_label(field: &Field) -> String {
        let label = crate::terminology::tr(&field.label).into_owned();
        if field.target == Target::Action(PrefAction::Ai(ai_page::AiButton::Test)) {
            format!("{} {label}", crate::AI_SPARK)
        } else {
            label
        }
    }

    /// Whether the row has the label/control shape (the others draw their
    /// own lines).
    fn is_labelled(field: &Field) -> bool {
        !matches!(
            field.control,
            Control::Heading | Control::Chips(_) | Control::Area(_)
        ) && !Self::is_note(field)
            && !field.is_self_labelled()
    }

    /// An `Info` row with no value is a sentence, not a label: it takes the
    /// whole width and wraps.
    fn is_note(field: &Field) -> bool {
        matches!(&field.control, Control::Info(text) if text.is_empty()) && field.coming.is_none()
    }

    /// The value column and whether the page stacks. The column is the
    /// widest label plus 3, held to 18..=34 and to 45% of the body; the page
    /// stacks when the column its labels want leaves the control under 24
    /// cells (a narrow body would otherwise wrap every label).
    pub(super) fn layout_for(max_label: usize, width: usize) -> (usize, bool) {
        let wanted = (max_label + 3).clamp(18, 34);
        let cap = 34.min(width * 45 / 100).max(18);
        (wanted.min(cap), wanted + MIN_CONTROL > width)
    }

    /// The band width of a pick-only choice (a `Choice`, or a `Combo` the user
    /// cannot type into): its widest option label or current value, plus the
    /// chrome, at least `MIN_CHOICE_BAND`. `None` for every other control
    /// (editable combos and text fields fill the room).
    fn pick_only_width(field: &Field) -> Option<usize> {
        let labels: Vec<String> = match &field.control {
            Control::Choice { options, .. } => {
                options.iter().map(|o| field.option_label(o)).collect()
            }
            Control::Combo {
                options,
                custom: false,
                ..
            } => options.iter().map(|(l, _)| l.clone()).collect(),
            _ => return None,
        };
        let widest = labels
            .iter()
            .map(|l| display_width(l))
            .chain([display_width(&field.value_text())])
            .max()
            .unwrap_or(0);
        Some((widest + CHOICE_CHROME).max(MIN_CHOICE_BAND))
    }

    /// Computes the page's value column from its widest label. Called once
    /// per frame before any `field_lines`.
    pub(super) fn set_label_col(&self, fields: &[Field], width: usize) {
        let widest = fields
            .iter()
            .filter(|f| Self::is_labelled(f))
            .map(|f| display_width(&Self::row_label(f)))
            .max()
            .unwrap_or(0);
        self.label_col.set(Self::layout_for(widest, width));
    }

    /// One field's lines: the label and control, then help and warnings.
    pub(super) fn field_lines(
        &self,
        field: &Field,
        index: usize,
        selected: bool,
        width: usize,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        if matches!(field.control, Control::Heading) {
            lines.push(Line::from(""));
            let label = crate::terminology::tr(&field.label).into_owned();
            let rule = width.saturating_sub(display_width(&label) + 1);
            lines.push(Line::from(vec![
                Span::styled(
                    label,
                    Style::default()
                        .fg(th().ui.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}", "\u{2500}".repeat(rule)),
                    Style::default().fg(th().ui.muted),
                ),
            ]));
            return lines;
        }
        if let Control::Chips(labels) = &field.control {
            return self.chip_lines(index, labels, selected, width);
        }
        if let Control::Area(text) = &field.control {
            let lines = self.area_lines(field, text, selected, width);
            // The framed text rows are the input box.
            for (n, line) in lines.iter().enumerate() {
                if line
                    .spans
                    .first()
                    .is_some_and(|s| s.content.starts_with("  \u{2502} "))
                {
                    self.ctrl_cells.borrow_mut().push((
                        index,
                        true,
                        n,
                        4,
                        width.saturating_sub(4) as u16,
                    ));
                }
            }
            return lines;
        }
        if let Control::Buttons(buttons) = &field.control {
            return self.button_lines(field, buttons, index, selected, width);
        }

        let full_label = Self::row_label(field);
        if Self::is_note(field) {
            let style = Style::default().fg(th().ui.muted);
            for line in wrap_words(&full_label, width.min(PROSE_CAP)) {
                lines.push(Line::from(Span::styled(line, style)));
            }
            lines.push(Line::from(""));
            return lines;
        }
        let (label_col, stacked) = match self.label_col.get() {
            (0, _) => Self::layout_for(display_width(&full_label), width),
            set => set,
        };
        let value_col = if stacked { 2 } else { label_col };
        let room = width.saturating_sub(value_col).max(1);

        let editing = self.edit.as_ref().is_some_and(|(i, _, _)| *i == index);
        let is_number = matches!(field.control, Control::Number { .. });
        let is_choice = matches!(
            field.control,
            Control::Choice { .. } | Control::Combo { .. }
        ) && field.coming.is_none();
        let is_text = matches!(field.control, Control::Text(_) | Control::Number { .. })
            && field.coming.is_none();
        let is_quantity = matches!(field.control, Control::Quantity { .. });
        let band_w = if is_quantity {
            super::quantity::NUMBER_CELLS
        } else if is_number {
            NUMBER_BAND
        } else {
            // Text fields and editable combos (a value that can be long) take
            // the whole room right of the value column; a pick-only choice
            // fits its widest option.
            Self::pick_only_width(field).unwrap_or(room)
        }
        .min(room);

        let flashing = self.flash == Some((self.section, index));
        let label_style = if flash_unselected(flashing, selected) {
            Style::default().fg(th().ui.text).bg(th().ui.hover)
        } else if field.coming.is_some() {
            Style::default().fg(th().ui.muted)
        } else if selected {
            Style::default().fg(th().ui.text).bg(th().ui.selection)
        } else if matches!(field.control, Control::Info(_) | Control::Swatches(_)) {
            Style::default().fg(th().ui.muted)
        } else {
            Style::default().fg(th().ui.text)
        };

        // The label, wrapped inside its column (the whole row when stacked).
        let label_w = if stacked {
            width
        } else {
            label_col.saturating_sub(2)
        }
        .max(1);
        let mut label_lines = wrap_words(&full_label, label_w);
        if label_lines.len() > LABEL_LINES {
            let rest = label_lines.split_off(LABEL_LINES - 1).join(" ");
            label_lines.push(crate::ui_kit::fit(&rest, label_w));
        }
        let label_span = |text: String| -> Vec<Span<'static>> {
            match text.strip_prefix(crate::AI_SPARK) {
                Some(rest) => vec![
                    Span::styled(crate::AI_SPARK, label_style.fg(th().status.ai)),
                    Span::styled(rest.to_string(), label_style),
                ],
                None => vec![Span::styled(text, label_style)],
            }
        };
        // The line, within the field, the control sits on.
        let control_dy = if stacked { label_lines.len() as u16 } else { 0 };

        // A read-only value wraps in its column; the rest of it follows the row.
        let mut info_rest: Vec<String> = Vec::new();
        // The control, as spans.
        let mut control: Vec<Span<'static>> = if editing {
            let (_, draft, masked) = self.edit.as_ref().expect("editing");
            let caret = self.caret_edit.caret(index).cloned().unwrap_or_default();
            let (shown, caret) = if *masked {
                caret.masked(draft, '\u{2022}')
            } else {
                (draft.clone(), caret)
            };
            let ink = if self.edit_invalid == Some(index) {
                th().status.danger
            } else {
                th().status.warning
            };
            let look = crate::text_field::EditLook::new(
                Style::default().fg(ink).bg(crate::input_focus_bg()),
                "\u{2588}",
            );
            // The band's one-cell padding is kept while editing, so the text
            // does not jump left when the field opens.
            let text_w = band_w.saturating_sub(2).max(1);
            let line = crate::text_field::edit_line(&shown, Some(&caret), text_w, true, look);
            if let Some(real) = self.caret_edit.caret(index) {
                real.scroll.set(caret.scroll.get());
            }
            self.edit_value_at
                .set(Some((value_col as u16 + 1, control_dy, text_w as u16)));
            let pad = Span::styled(" ", Style::default().bg(crate::input_focus_bg()));
            let mut spans = vec![pad.clone()];
            spans.extend(line.spans);
            spans.push(pad);
            spans
        } else if is_quantity {
            self.quantity_number_spans(field, index, selected, value_col, control_dy, band_w)
        } else if let Control::Swatches(chips) = &field.control {
            let mut spans = Vec::new();
            for (n, color) in chips.iter().enumerate() {
                if n > 0 {
                    spans.push(Span::raw(" "));
                }
                spans.push(Span::styled("  ", Style::default().bg(*color)));
            }
            spans
        } else {
            let text = field.value_text();
            let ink = |style: Style| {
                if selected {
                    style.fg(th().ui.text)
                } else {
                    style
                }
            };
            if is_choice || is_text {
                let bg = crate::theme::input_bg(selected, false);
                let style = ink(Style::default().fg(th().ui.accent)).bg(bg);
                let inner = band_w.saturating_sub(if is_choice { 4 } else { 2 });
                let text = crate::ui_kit::fit(&text, inner);
                let pad = inner.saturating_sub(display_width(&text));
                let mut band = format!(" {text}{} ", " ".repeat(pad));
                if is_choice {
                    band.push_str("\u{25BE} ");
                }
                if self.choice.as_ref().is_some_and(|(i, _)| *i == index) {
                    self.choice_value_at
                        .set(Some((value_col as u16, control_dy, band_w as u16)));
                }
                vec![Span::styled(band, style)]
            } else if field.coming.is_some() {
                vec![Span::styled(
                    crate::ui_kit::fit(&text, room),
                    Style::default()
                        .fg(th().ui.muted)
                        .add_modifier(Modifier::ITALIC),
                )]
            } else if matches!(field.control, Control::Info(_)) {
                let mut wrapped = wrap_words(&text, room);
                let first = if wrapped.is_empty() {
                    String::new()
                } else {
                    crate::ui_kit::fit(&wrapped.remove(0), room)
                };
                info_rest = wrapped
                    .into_iter()
                    .map(|w| crate::ui_kit::fit(&w, room))
                    .collect();
                vec![Span::styled(first, Style::default().fg(th().ui.muted))]
            } else {
                vec![Span::styled(
                    crate::ui_kit::fit(&text, room),
                    ink(Style::default().fg(th().ui.accent)),
                )]
            }
        };

        // The control's own cell: the pointer is the hand on a checkbox or a
        // combobox field, the I-beam on an input box.
        let ctrl = if editing {
            Some((true, band_w))
        } else if is_quantity || field.coming.is_some() {
            None
        } else if is_choice {
            Some((false, band_w))
        } else if is_text {
            Some((true, band_w))
        } else if matches!(field.control, Control::Toggle(_)) {
            Some((false, display_width(&field.value_text())))
        } else {
            None
        };
        if let Some((input, w)) = ctrl {
            self.ctrl_cells.borrow_mut().push((
                index,
                input,
                usize::from(control_dy),
                value_col as u16,
                w as u16,
            ));
        }
        if is_quantity {
            control.extend(
                self.quantity_unit_spans(field, index, selected, value_col, control_dy, room),
            );
        }
        if stacked {
            for text in &label_lines {
                lines.push(Line::from(label_span(text.clone())));
            }
            let mut row = vec![Span::raw("  ")];
            row.extend(control);
            lines.push(Line::from(row));
        } else {
            for (n, text) in label_lines.iter().enumerate() {
                let pad = label_col.saturating_sub(display_width(text));
                let mut row = label_span(text.clone());
                if n == 0 {
                    row.push(Span::raw(" ".repeat(pad)));
                    row.extend(control.clone());
                }
                lines.push(Line::from(row));
            }
        }

        let indent = " ".repeat(value_col);
        for line in info_rest {
            lines.push(Line::from(Span::styled(
                format!("{indent}{line}"),
                Style::default().fg(th().ui.muted),
            )));
        }
        if let Some(line) = self.quantity_error_line(index, value_col, width) {
            lines.push(line);
        }
        self.tail_lines(field, value_col, width.min(HELP_CAP), &mut lines);
        lines.push(Line::from(""));
        lines
    }

    /// The help, warning and "not wired" note under a row's control,
    /// indented to the value column.
    pub(super) fn tail_lines(
        &self,
        field: &Field,
        value_col: usize,
        width: usize,
        lines: &mut Vec<Line<'static>>,
    ) {
        let indent = " ".repeat(value_col);
        let text_w = width.saturating_sub(value_col).max(1);
        if let Some(help) = &field.help {
            for line in wrap_words(&crate::terminology::tr(help), text_w) {
                lines.push(Line::from(Span::styled(
                    format!("{indent}{}", crate::ui_kit::fit(&line, text_w)),
                    Style::default().fg(th().ui.muted),
                )));
            }
        }
        if let Some(warning) = &field.warning {
            for (n, line) in wrap_words(warning, text_w.saturating_sub(2))
                .into_iter()
                .enumerate()
            {
                let prefix = if n == 0 { "\u{26A0} " } else { "  " };
                lines.push(Line::from(Span::styled(
                    format!("{indent}{prefix}{line}"),
                    Style::default().fg(th().status.warning),
                )));
            }
        }
        if field.coming.is_none() && !field.live {
            for line in wrap_words(screen::NOT_WIRED.get(), text_w) {
                lines.push(Line::from(Span::styled(
                    format!("{indent}{line}"),
                    Style::default()
                        .fg(th().ui.muted)
                        .add_modifier(Modifier::ITALIC),
                )));
            }
        }
    }
}
