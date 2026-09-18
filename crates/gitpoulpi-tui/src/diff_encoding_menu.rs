//! The Diff view's encoding dropdown: the bar's `Encoding` button opens a
//! combobox listing Auto-detect first, then the editor's encodings
//! (`editor_charset::ALL`), the current one marked. A clone of
//! `image_view/edit_in.rs`, which explains the shape.
//!
//! The popup is modal: while it is open it owns the keyboard and the mouse
//! (a click away closes it and is consumed). Its rows and frame are put at
//! the front of the frame's hit list from the paint that drew them, so
//! nothing behind it hovers.
//!
//! The review has its own copy of the dropdown inside its page (its popups
//! are the page's, see `review_view/popups.rs`); both list the same
//! options through [`options`] and read a pick through [`pick_of`].

use std::sync::Mutex;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::chrome::HitTarget;
use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::diff_recode::AUTO_VALUE;
use crate::editor_charset::{self as charset, Charset};
use gitpoulpi_i18n::msg::diff_view as msg;

/// What a pick chose: `None` is Auto-detect.
pub(crate) type Pick = Option<Charset>;

/// The dropdown's rows: Auto-detect, then every encoding, in the editor's
/// order.
pub(crate) fn options() -> Vec<ComboOption> {
    std::iter::once(ComboOption::new(msg::ENC_AUTO_DETECT.get(), AUTO_VALUE))
        .chain(charset::ALL.iter().map(|c| ComboOption::plain(c.name())))
        .collect()
}

/// The combo value of the current choice.
pub(crate) fn value_of(current: Option<Charset>) -> String {
    current.map_or_else(|| AUTO_VALUE.to_string(), |c| c.name().to_string())
}

/// The choice a picked combo value stands for; `None` for a value that is
/// neither (nothing changes then).
pub(crate) fn pick_of(value: &str) -> Option<Pick> {
    if value == AUTO_VALUE {
        Some(None)
    } else {
        Charset::from_name(value).map(Some)
    }
}

/// A ready combo, open, on `current`.
pub(crate) fn combo(current: Option<Charset>) -> Combo {
    let mut combo = Combo::new("").with_search().with_click_picks();
    combo.set_options(options());
    combo.select(Some(value_of(current)));
    combo.open();
    combo
}

static POPUP: Mutex<Option<Combo>> = Mutex::new(None);
/// Where the popup hangs from: the button as painted last.
static ANCHOR: Mutex<Option<Rect>> = Mutex::new(None);

/// Records where the bar's button was painted.
pub(crate) fn note_button(rect: Rect) {
    if let Ok(mut a) = ANCHOR.lock() {
        *a = Some(rect);
    }
}

pub(crate) fn is_open() -> bool {
    POPUP
        .lock()
        .is_ok_and(|p| p.as_ref().is_some_and(Combo::is_open))
}

/// Opens the dropdown on `current`.
pub(crate) fn open(current: Option<Charset>) {
    if let Ok(mut p) = POPUP.lock() {
        *p = Some(combo(current));
    }
}

fn settle(slot: &mut Option<Combo>, outcome: ComboOutcome) -> Option<Pick> {
    let pick = match outcome {
        ComboOutcome::Picked(value) | ComboOutcome::Custom(value) => pick_of(&value),
        _ => None,
    };
    if slot.as_ref().is_some_and(|c| !c.is_open()) {
        *slot = None;
    }
    pick
}

/// A key while the popup is open (it swallows them all).
pub(crate) fn on_key(key: KeyEvent) -> Option<Pick> {
    let mut slot = POPUP.lock().ok()?;
    let outcome = slot.as_mut()?.on_key(key);
    settle(&mut slot, outcome)
}

/// A mouse event while the popup is open (it swallows them all).
pub(crate) fn on_mouse(mouse: MouseEvent) -> Option<Pick> {
    let mut slot = POPUP.lock().ok()?;
    let outcome = slot.as_mut()?.on_mouse(mouse);
    settle(&mut slot, outcome)
}

/// Pasted text goes to the search line.
pub(crate) fn paste(text: &str) {
    if let Ok(mut p) = POPUP.lock() {
        if let Some(combo) = p.as_mut() {
            combo.paste(text);
        }
    }
}

/// Paints the open popup over the frame and registers its rows, then the
/// whole popup, at the front of `hits`.
pub(crate) fn render_popup(
    frame: &mut Frame,
    hovered_row: Option<usize>,
    hits: &mut Vec<(Rect, HitTarget)>,
) {
    let bounds = frame.area();
    let Ok(mut slot) = POPUP.lock() else {
        return;
    };
    let Some(combo) = slot.as_mut() else {
        return;
    };
    let fallback = || {
        Rect::new(
            bounds.right().saturating_sub(16),
            bounds.y + 1,
            16.min(bounds.width),
            1,
        )
    };
    let anchor = ANCHOR
        .lock()
        .ok()
        .and_then(|a| *a)
        .filter(|a| a.right() <= bounds.right() && a.bottom() <= bounds.bottom())
        .unwrap_or_else(fallback);
    combo.set_anchor(anchor);
    combo.set_hover(hovered_row);
    combo.render_popup(frame.buffer_mut(), bounds);
    let mut front: Vec<(Rect, HitTarget)> = combo
        .item_rects()
        .into_iter()
        .map(|(rect, row)| (rect, HitTarget::OpenInItem(row)))
        .collect();
    let popup = combo.popup_rect();
    if popup.width > 0 {
        front.push((popup, HitTarget::OpenInPopup));
    }
    hits.splice(0..0, front);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_detect_comes_first_then_the_editors_encodings() {
        let rows = options();
        assert_eq!(rows[0].value, AUTO_VALUE);
        assert_eq!(rows.len(), charset::ALL.len() + 1);
        assert_eq!(rows[1].label, "UTF-8");
        for (row, cs) in rows[1..].iter().zip(charset::ALL) {
            assert_eq!(pick_of(&row.value), Some(Some(*cs)));
        }
        assert_eq!(pick_of(AUTO_VALUE), Some(None));
        assert_eq!(pick_of("nonsense"), None);
    }
}
