//! What the last paint put under the pointer, for the surfaces that own the
//! mouse (Settings, dialogs, popups): the buttons, checkboxes and combo rows
//! they painted are hands, the text fields are I-beams. The paint registers a rect as it draws it, so a
//! control a surface does not list for its keyboard badges (a confirmation
//! inside Settings, a combo list) is still answered.
//!
//! Cleared at the start of every frame. The most recently registered rect
//! under a point wins: it is the one painted last, so the one on top.

use ratatui::layout::{Position, Rect};

/// What a registered rect is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// A click does something: the pointing hand.
    Hand,
    /// A text field: the I-beam.
    Field,
    /// A row or a body that lights on hover and is not a control on its
    /// own: the arrow, and nothing painted under it answers.
    Plain,
    /// A column divider: the sideways resize pointer (`ew-resize`).
    ColResize,
    /// A row divider or a bottom border: the vertical resize pointer.
    RowResize,
    /// A dialog's title bar: the move pointer.
    Move,
    /// A dialog's bottom right corner (`nwse-resize`).
    NwseResize,
    /// A dialog's bottom left corner (`nesw-resize`).
    NeswResize,
}

impl Kind {
    /// The pointer a frame / divider kind shows (`None`: a control kind).
    pub(crate) fn resize_shape(self) -> Option<crate::cursor::CursorShape> {
        use crate::cursor::CursorShape as S;
        match self {
            Kind::ColResize => Some(S::ColResize),
            Kind::RowResize => Some(S::RowResize),
            Kind::Move => Some(S::Move),
            Kind::NwseResize => Some(S::NwseResize),
            Kind::NeswResize => Some(S::NeswResize),
            Kind::Hand | Kind::Field | Kind::Plain => None,
        }
    }
}

type Map = Vec<(Rect, Kind)>;

#[cfg(not(test))]
static MAP: std::sync::Mutex<Map> = std::sync::Mutex::new(Vec::new());

// A unit test gets a map of its own (tests run one per thread).
#[cfg(test)]
thread_local! {
    static MAP: std::cell::RefCell<Map> = const { std::cell::RefCell::new(Vec::new()) };
}

fn with<R>(f: impl FnOnce(&mut Map) -> R) -> R {
    #[cfg(not(test))]
    {
        f(&mut MAP.lock().unwrap_or_else(|e| e.into_inner()))
    }
    #[cfg(test)]
    {
        MAP.with(|m| f(&mut m.borrow_mut()))
    }
}

/// Forgets last frame's rects; the paint that follows registers this one's.
pub(crate) fn begin_frame() {
    with(Vec::clear);
}

/// Records that `rect` was painted as a control a click acts on.
pub(crate) fn hand(rect: Rect) {
    add(rect, Kind::Hand);
}

/// Records that `rect` was painted as a text field.
pub(crate) fn field(rect: Rect) {
    add(rect, Kind::Field);
}

/// Records that `rect` was painted as a row (or a body) that keeps the
/// arrow: it also hides what an earlier paint registered under it.
pub(crate) fn plain(rect: Rect) {
    add(rect, Kind::Plain);
}

/// Records `rect` as `kind`, for a widget that decides its kind per instance
/// (a sidebar row that is a combobox, a search box or an item).
#[cfg_attr(not(feature = "hosting"), allow(dead_code))] // only the hosting sidebar rows register per instance
pub(crate) fn register(rect: Rect, kind: Kind) {
    add(rect, kind);
}

fn add(rect: Rect, kind: Kind) {
    if rect.width > 0 && rect.height > 0 {
        with(|m| m.push((rect, kind)));
    }
}

/// Registers a surface's hit list: `kind` says what each hit is (`None`: not
/// a control, a body or a resize strip). The list is in click order (the
/// first rect holding the point answers), the map in paint order (the last
/// does), so the list goes in reversed.
pub(crate) fn from_hits<H>(hits: &[(Rect, H)], kind: impl Fn(&H) -> Option<Kind>) {
    for (rect, hit) in hits.iter().rev() {
        if let Some(kind) = kind(hit) {
            add(*rect, kind);
        }
    }
}

/// The shape the paint registered under the point, applied over `shape`
/// when the hit-target rules left it at the arrow: a combobox, a button or a
/// checkbox a widget registered itself is the hand, an input box the I-beam.
/// What every surface that is not a modal owner uses, so a widget answers
/// the same wherever it is drawn.
pub(crate) fn over(
    shape: crate::cursor::CursorShape,
    col: u16,
    row: u16,
) -> crate::cursor::CursorShape {
    if shape == crate::cursor::CursorShape::Default {
        shape_at(col, row)
    } else {
        shape
    }
}

/// What the topmost registered rect at the point is, if any.
pub(crate) fn at(col: u16, row: u16) -> Option<Kind> {
    let p = Position::new(col, row);
    with(|m| m.iter().rev().find(|(r, _)| r.contains(p)).map(|(_, k)| *k))
}

/// The shape the last paint registered under the point: the hand, the
/// I-beam, else the arrow. What a surface that covers everything behind it
/// (a modal list) answers with.
pub(crate) fn shape_at(col: u16, row: u16) -> crate::cursor::CursorShape {
    use crate::cursor::CursorShape;
    match at(col, row) {
        Some(Kind::Field) => CursorShape::Text,
        Some(Kind::Hand) => CursorShape::Pointer,
        Some(k) if k.resize_shape().is_some() => k.resize_shape().unwrap_or(CursorShape::Default),
        Some(_) | None => CursorShape::Default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rect_painted_last_answers() {
        begin_frame();
        field(Rect::new(0, 0, 20, 1));
        hand(Rect::new(5, 0, 4, 1));
        assert_eq!(at(1, 0), Some(Kind::Field));
        assert_eq!(at(6, 0), Some(Kind::Hand));
        assert_eq!(at(30, 0), None);
        begin_frame();
        assert_eq!(at(6, 0), None);
    }

    #[test]
    fn an_empty_rect_registers_nothing() {
        begin_frame();
        hand(Rect::new(3, 3, 0, 1));
        assert_eq!(at(3, 3), None);
    }

    /// Every shared widget registers its own pointer kind when it paints, so
    /// no screen can forget it: a button, a checkbox and a close cross are the
    /// hand, an input box the I-beam, a list row the arrow, a combobox field
    /// the hand.
    #[test]
    fn each_widget_kind_registers_its_pointer_when_it_paints() {
        use crate::combo::Combo;
        use crate::dialog_kit::{self, ButtonState, CheckState};
        use ratatui::buffer::Buffer;
        let area = Rect::new(0, 0, 60, 6);
        let mut buf = Buffer::empty(area);
        begin_frame();
        let button = dialog_kit::button(
            &mut buf,
            1,
            0,
            "OK",
            ButtonState::Normal {
                hovered: false,
                focused: false,
            },
        );
        assert_eq!(at(button.x + 1, button.y), Some(Kind::Hand), "button");
        let boxed = dialog_kit::checkbox(
            &mut buf,
            1,
            1,
            20,
            "Option",
            true,
            CheckState::Enabled {
                hovered: false,
                focused: false,
            },
        );
        assert_eq!(at(boxed.x + 1, boxed.y), Some(Kind::Hand), "checkbox");
        let mut combo = Combo::new("Pick");
        let field = Rect::new(1, 2, 20, 1);
        combo.render_field(&mut buf, field, false, false);
        assert_eq!(at(field.x + 3, field.y), Some(Kind::Hand), "combobox");
        let edit = crate::text_edit::FieldEditor::<u8>::default();
        let input = Rect::new(1, 3, 20, 1);
        dialog_kit::text_input(&mut buf, input, "", "type", false, false, (&edit, 0));
        assert_eq!(at(input.x + 3, input.y), Some(Kind::Field), "input box");
        let row = Rect::new(1, 4, 20, 1);
        dialog_kit::row_band(&mut buf, row, dialog_kit::row_bg(false, true));
        assert_eq!(at(row.x + 3, row.y), Some(Kind::Plain), "list row");
        assert_eq!(buf[(row.x + 3, row.y)].bg, crate::th().ui.hover);
        let block = Rect::new(0, 0, 40, 6);
        let cross = crate::chrome::close_cross::paint(&mut buf, block, false).unwrap();
        assert_eq!(at(cross.x + 1, cross.y), Some(Kind::Hand), "close cross");
        // A control painted inside a row keeps the hand (painted after it).
        let mark = Rect::new(row.x, row.y, 3, 1);
        hand(mark);
        assert_eq!(at(mark.x + 1, mark.y), Some(Kind::Hand));
        assert_eq!(
            shape_at(row.x + 10, row.y),
            crate::cursor::CursorShape::Default
        );
        assert_eq!(
            shape_at(mark.x + 1, mark.y),
            crate::cursor::CursorShape::Pointer
        );
        assert_eq!(
            shape_at(input.x + 3, input.y),
            crate::cursor::CursorShape::Text
        );
    }

    /// The ordinary path answers the arrow only when nothing registered a
    /// shape; a registered hand or I-beam wins over a hit-target arrow, a
    /// resize or a carry stays.
    #[test]
    fn the_registered_shape_applies_over_the_arrow_only() {
        use crate::cursor::CursorShape;
        begin_frame();
        hand(Rect::new(0, 0, 4, 1));
        field(Rect::new(0, 1, 4, 1));
        plain(Rect::new(0, 2, 4, 1));
        assert_eq!(over(CursorShape::Default, 1, 0), CursorShape::Pointer);
        assert_eq!(over(CursorShape::Default, 1, 1), CursorShape::Text);
        assert_eq!(over(CursorShape::Default, 1, 2), CursorShape::Default);
        assert_eq!(over(CursorShape::ColResize, 1, 0), CursorShape::ColResize);
    }
}
