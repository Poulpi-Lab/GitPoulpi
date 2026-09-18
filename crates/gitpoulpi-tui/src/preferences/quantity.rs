//! The number-and-unit row (`Control::Quantity`): `[ 2 ] [weeks v]` on one
//! line. The number is a text field (click it or Enter to type, Up/Down
//! while typing to step), the unit a dropdown that opens on a click and
//! never cycles. Left/Right/Tab move between the two parts; a mouse click
//! on a part focuses and acts on it (its cells are the row's `btn_cells`, the
//! same hit-rect walk as the inline buttons, with part 0 = number, part 1 =
//! unit, and `ai.chip` as the focused part).
//!
//! What is stored: the row's own key holds `number x unit weight` (days for
//! the log retention, minutes for the auto-fetch interval...), the control's
//! `unit_key` the unit the number is read in. A refused number (not whole,
//! under 1 - or under 0 where 0 turns the feature off - or above the maximum
//! in the chosen unit) keeps the field open, red, with the reason on the
//! line under it.

use super::*;
use crate::th;
use gitpoulpi_i18n::msg::settings_screen as screen;

/// Cells of the number's band (one blank each side, four digits and a spare).
pub(super) const NUMBER_CELLS: usize = 8;
/// The number and the unit band are this far apart.
const GAP: usize = 1;

/// The parts of the row.
pub(super) const PART_NUMBER: usize = 0;
pub(super) const PART_UNIT: usize = 1;

impl PreferencesState {
    /// The selected row, when it is a quantity.
    fn selected_quantity(&self) -> bool {
        self.fields()
            .get(self.field)
            .is_some_and(|f| matches!(f.control, Control::Quantity { .. }))
    }

    /// Whether the number being typed belongs to a quantity row.
    pub(super) fn editing_quantity(&self) -> bool {
        let Some((index, _, _)) = self.edit.as_ref() else {
            return false;
        };
        self.fields()
            .get(*index)
            .is_some_and(|f| matches!(f.control, Control::Quantity { .. }))
    }

    /// The width of the unit band: the widest unit name, a pad and the arrow.
    fn unit_band_width(units: &[QuantityUnit], room: usize) -> usize {
        let widest = units
            .iter()
            .map(|u| display_width(&u.label))
            .max()
            .unwrap_or(0);
        (widest + 4)
            .min(room.saturating_sub(NUMBER_CELLS + GAP))
            .max(4)
    }

    /// The control's spans after the number band: the gap and the unit band.
    /// Records the unit's hit cell and, when its dropdown is open, where it
    /// hangs from.
    pub(super) fn quantity_unit_spans(
        &self,
        field: &Field,
        index: usize,
        selected: bool,
        value_col: usize,
        control_dy: u16,
        room: usize,
    ) -> Vec<Span<'static>> {
        let Control::Quantity { unit, units, .. } = &field.control else {
            return Vec::new();
        };
        let live = field.coming.is_none();
        let band_w = Self::unit_band_width(units, room);
        let at = value_col + NUMBER_CELLS + GAP;
        let hovered = self.mouse_pos.is_some_and(|(c, r)| {
            self.btn_rects
                .iter()
                .any(|(rect, f, b)| *f == index && *b == PART_UNIT && rect.contains((c, r).into()))
        });
        let focused = selected && self.ai.chip_focus(2) == PART_UNIT;
        let bg = crate::theme::input_bg(focused, hovered);
        let style = Style::default().fg(th().ui.accent).bg(bg);
        let style = if focused {
            style.fg(th().ui.text)
        } else {
            style
        };
        let inner = band_w.saturating_sub(4);
        let name = units.get(*unit).map(|u| u.label.as_str()).unwrap_or("");
        let name = crate::ui_kit::fit(name, inner);
        let pad = inner.saturating_sub(display_width(&name));
        let band = format!(" {name}{} \u{25BE} ", " ".repeat(pad));
        if live {
            self.btn_cells.borrow_mut().push((
                index,
                PART_UNIT,
                usize::from(control_dy),
                at as u16,
                band_w as u16,
            ));
        }
        if self.choice.as_ref().is_some_and(|(i, _)| *i == index) {
            self.choice_value_at
                .set(Some((at as u16, control_dy, band_w as u16)));
        }
        vec![Span::raw(" ".repeat(GAP)), Span::styled(band, style)]
    }

    /// The number's band when it is not being typed in. Records its hit cell.
    pub(super) fn quantity_number_spans(
        &self,
        field: &Field,
        index: usize,
        selected: bool,
        value_col: usize,
        control_dy: u16,
        band_w: usize,
    ) -> Vec<Span<'static>> {
        let Control::Quantity { value, .. } = &field.control else {
            return Vec::new();
        };
        if field.coming.is_none() {
            self.btn_cells.borrow_mut().push((
                index,
                PART_NUMBER,
                usize::from(control_dy),
                value_col as u16,
                band_w as u16,
            ));
        }
        let hovered = self.mouse_pos.is_some_and(|(c, r)| {
            self.btn_rects.iter().any(|(rect, f, b)| {
                *f == index && *b == PART_NUMBER && rect.contains((c, r).into())
            })
        });
        let focused = selected && self.ai.chip_focus(2) == PART_NUMBER;
        let bg = crate::theme::input_bg(focused, hovered);
        let ink = if focused {
            th().ui.text
        } else {
            th().ui.accent
        };
        let inner = band_w.saturating_sub(2);
        let text = crate::ui_kit::fit(&value.to_string(), inner);
        let pad = inner.saturating_sub(display_width(&text));
        vec![Span::styled(
            format!(" {text}{} ", " ".repeat(pad)),
            Style::default().fg(ink).bg(bg),
        )]
    }

    /// The red reason under a refused number, in the value column.
    pub(super) fn quantity_error_line(
        &self,
        index: usize,
        value_col: usize,
        width: usize,
    ) -> Option<Line<'static>> {
        if self.edit_invalid != Some(index) {
            return None;
        }
        let (at, text) = self.inline_error.as_ref()?;
        if *at != index {
            return None;
        }
        let room = width.saturating_sub(value_col).max(1);
        Some(Line::from(Span::styled(
            format!(
                "{}{}",
                " ".repeat(value_col),
                crate::ui_kit::fit(text, room)
            ),
            Style::default().fg(th().status.danger),
        )))
    }

    /// Left/Right/Tab move between the number and the unit. `true` when the
    /// key was used; Enter and Space fall through to `activate`.
    pub(super) fn quantity_key(&mut self, key: &KeyEvent) -> bool {
        if !self.selected_quantity() {
            return false;
        }
        let at = self.ai.chip_focus(2);
        match key.code {
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab if at == PART_NUMBER => {
                self.ai.chip = PART_UNIT;
                true
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::BackTab if at == PART_UNIT => {
                self.ai.chip = PART_NUMBER;
                true
            }
            _ => false,
        }
    }

    /// Enter, Space or a click on the selected quantity: type the number, or
    /// open the unit dropdown, by the part in focus.
    pub(super) fn activate_quantity(&mut self, field: &Field) {
        let Control::Quantity {
            value, unit, units, ..
        } = &field.control
        else {
            return;
        };
        if self.ai.chip_focus(2) == PART_NUMBER {
            self.caret_edit.reset();
            self.edit = Some((self.field, value.to_string(), false));
            return;
        }
        let mut combo = crate::combo::Combo::new(screen::NONE.get());
        combo.set_options(
            units
                .iter()
                .enumerate()
                .map(|(i, u)| crate::combo::ComboOption::new(u.label.clone(), i.to_string()))
                .collect(),
        );
        combo.select(Some(unit.to_string()));
        combo.open();
        self.choice = Some((self.field, combo));
    }

    /// Up/Down while the number is being typed: one more or less, within
    /// the range the chosen unit allows. `true` when handled.
    pub(super) fn quantity_step(&mut self, delta: i64) -> bool {
        let Some((index, draft, _)) = self.edit.clone() else {
            return false;
        };
        let fields = self.fields();
        let Some(Control::Quantity {
            value,
            unit,
            units,
            max_total,
            allow_zero,
            ..
        }) = fields.get(index).map(|f| &f.control)
        else {
            return false;
        };
        let weight = units.get(*unit).map(|u| u.weight).unwrap_or(1).max(1);
        let top = (max_total / weight).max(1);
        let now = draft.trim().parse::<i64>().unwrap_or(*value);
        let next = (now + delta).clamp(i64::from(!*allow_zero), top);
        self.caret_edit.reset();
        self.edit = Some((index, next.to_string(), false));
        self.edit_invalid = None;
        true
    }

    /// A typed number. `false`: refused (the reason is set, inline and in
    /// the message), the caller keeps the field open.
    pub(super) fn commit_quantity_number(&mut self, index: usize, draft: &str) -> bool {
        let fields = self.fields();
        let Some(field) = fields.get(index) else {
            return true;
        };
        let (
            Control::Quantity {
                unit,
                units,
                max_total,
                allow_zero,
                unit_key,
                ..
            },
            Target::Global(key),
        ) = (&field.control, field.target.clone())
        else {
            return true;
        };
        let lo = i64::from(!*allow_zero);
        let weight = units.get(*unit).map(|u| u.weight).unwrap_or(1).max(1);
        let top = (max_total / weight).max(1);
        let refuse = |this: &mut Self, why: String| {
            this.inline_error = Some((index, why.clone()));
            this.message = Some(why);
            false
        };
        match draft.trim().parse::<i64>() {
            Ok(n) if (lo..=top).contains(&n) => {
                let unit_name = units[*unit].key;
                self.write_quantity(key, unit_key, n * weight, unit_name)
            }
            Ok(n) => refuse(self, screen::value_outside(key, n, lo, top)),
            Err(_) => refuse(self, screen::not_whole_number(key, format!("{draft:?}"))),
        }
    }

    /// A unit picked in the dropdown: the number stays, cut to the most the
    /// new unit allows.
    pub(super) fn commit_quantity_unit(&mut self, index: usize, pick: usize) {
        let fields = self.fields();
        let Some(field) = fields.get(index) else {
            return;
        };
        let (
            Control::Quantity {
                value,
                units,
                max_total,
                allow_zero,
                unit_key,
                ..
            },
            Target::Global(key),
        ) = (&field.control, field.target.clone())
        else {
            return;
        };
        let Some(picked) = units.get(pick) else {
            return;
        };
        let top = (max_total / picked.weight.max(1)).max(1);
        let n = (*value).clamp(i64::from(!*allow_zero), top);
        self.write_quantity(key, unit_key, n * picked.weight, picked.key);
    }

    /// Writes the total and the unit. The total first: it is what the
    /// application log reads, and an older GitPoulpi that does not know the
    /// unit key still gets the right number of days.
    fn write_quantity(&mut self, key: &str, unit_key: &str, total: i64, unit: &str) -> bool {
        self.write_global(key, &total.to_string()) && self.write_global(unit_key, unit)
    }

    /// Selecting a quantity row starts on its number.
    pub(super) fn quantity_focus_reset(&mut self) {
        if self.selected_quantity() {
            self.ai.chip = PART_NUMBER;
        }
    }
}
