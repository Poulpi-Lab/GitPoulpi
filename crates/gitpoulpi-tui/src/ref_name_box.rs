//! Where a name is typed: inline, or in a small framed box.
//!
//! The names that `InputMode::ComposingRefName` collects for a new branch,
//! a new tag, a branch rename and a new file are never typed in the status
//! line. Each has a spot where the thing being named already is: the Log's
//! Ref cell, the sidebar's LOCAL row, the Status list. When no such spot
//! can be drawn (another view is on screen, the Ref column is hidden, the
//! row is not loaded...) a small framed **name box** appears instead, with
//! the same field, the error inside it, Cancel and confirm buttons and a
//! close cross.
//!
//! The painters take turns through a [`NameSlot`]: the first one that can
//! draw this frame marks the slot, the rest skip, and the box draws only if
//! nobody did. The slot is **sticky**: once the box has drawn, an inline
//! painter does not take the field back until the mode ends, so the field
//! never jumps between places on a scroll or a resize.

use crate::wide_edge::Clear;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui::Frame;

use super::*;
use crate::dialog_kit::{self, ButtonState};
use gitpoulpi_i18n::msg::menu_inline as msg;

/// A place the name field can be drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NameSpot {
    /// Over the branch's LOCAL row in the Refs sidebar.
    Refs,
    /// In the Ref column of a Log row.
    Log,
    /// A row of the Status list.
    Status,
    /// The framed name box.
    Box,
}

/// Which painter owns the field: per frame (`drawn`) and across frames
/// (`sticky`).
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct NameSlot {
    sticky: Option<NameSpot>,
    drawn: Option<NameSpot>,
}

impl NameSlot {
    /// Whether `spot` may draw the field now: nobody has this frame, and the
    /// field is not parked somewhere else. The box is the last resort, so
    /// it may draw whenever nothing else did.
    pub(crate) fn may_draw(&self, spot: NameSpot) -> bool {
        self.drawn.is_none()
            && (spot == NameSpot::Box || self.sticky.is_none_or(|stuck| stuck == spot))
    }

    pub(crate) fn mark(&mut self, spot: NameSpot) {
        self.drawn = Some(spot);
    }

    pub(crate) fn drawn(&self) -> bool {
        self.drawn.is_some()
    }

    /// After a frame: where the field drew is where it stays.
    pub(crate) fn end_frame(&mut self) {
        if let Some(spot) = self.drawn.take() {
            self.sticky = Some(spot);
        }
    }

    /// The mode ended.
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
}

/// A clickable part of the name box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RefNamePart {
    Confirm,
    Cancel,
    Close,
}

/// Whether `op` is named inline (or in the box) rather than in the footer.
pub(crate) fn is_inline_name(op: &RefNameOp) -> bool {
    matches!(
        op,
        RefNameOp::NewBranch
            | RefNameOp::NewTag
            | RefNameOp::NewBranchAt(_)
            | RefNameOp::NewTagAt(_)
            | RefNameOp::NewAnnotatedTagAt(_)
            | RefNameOp::RenameBranch { .. }
            | RefNameOp::CreateFile(_)
    )
}

fn sha(at: &Oid) -> String {
    at.to_hex()[..SHORT_SHA_LEN].to_string()
}

fn folder_text(folder: &std::path::Path) -> String {
    folder.to_string_lossy().replace('\\', "/")
}

/// The box's title.
pub(crate) fn box_title(op: &RefNameOp) -> String {
    match op {
        RefNameOp::NewBranch => msg::BOX_NEW_BRANCH.get().to_string(),
        RefNameOp::NewTag => msg::BOX_NEW_TAG.get().to_string(),
        RefNameOp::NewBranchAt(at) => msg::box_new_branch_at(sha(at)),
        RefNameOp::NewTagAt(at) => msg::box_new_tag_at(sha(at)),
        RefNameOp::NewAnnotatedTagAt(at) => msg::box_new_annotated_tag_at(sha(at)),
        RefNameOp::RenameBranch { name, .. } => msg::box_rename(name),
        RefNameOp::CreateFile(folder) if folder.as_os_str().is_empty() => {
            msg::BOX_NEW_FILE_ROOT.get().to_string()
        }
        RefNameOp::CreateFile(folder) => msg::box_new_file(folder_text(folder)),
        _ => String::new(),
    }
}

/// What an empty field says.
pub(crate) fn box_placeholder(op: &RefNameOp) -> &'static str {
    match op {
        RefNameOp::NewTag | RefNameOp::NewTagAt(_) | RefNameOp::NewAnnotatedTagAt(_) => {
            msg::PLACEHOLDER_TAG.get()
        }
        RefNameOp::CreateFile(_) => msg::PLACEHOLDER_FILE.get(),
        _ => msg::PLACEHOLDER_BRANCH.get(),
    }
}

/// The confirm button's label.
pub(crate) fn box_confirm_label(op: &RefNameOp) -> &'static str {
    match op {
        RefNameOp::RenameBranch { .. } => msg::RENAME_BUTTON.get(),
        _ => msg::CREATE_BUTTON.get(),
    }
}

const WIN: dialog_kit::Win = dialog_kit::Win::movable("ref_name");

/// Height: border, field, error row, a gap, buttons, border.
const BOX_HEIGHT: u16 = 6;

/// Paints the name box over the frame and returns its hit rects.
pub(crate) fn paint_name_box(
    frame: &mut Frame,
    area: Rect,
    op: &RefNameOp,
    value: &str,
    error: Option<&str>,
    editor: &crate::text_edit::FieldEditor<MainField>,
    hovered: Option<RefNamePart>,
) -> Vec<(Rect, RefNamePart)> {
    let cancel_label = gitpoulpi_i18n::msg::dialog::CANCEL.get();
    let confirm = box_confirm_label(op);
    let buttons_w = dialog_kit::button_width(cancel_label) + 2 + dialog_kit::button_width(confirm);
    let title = format!(" {} ", box_title(op));
    let want_w = 56u16
        .max(buttons_w + 4)
        .max(display_width(&title) as u16 + 6);
    if area.width < 24 || area.height < BOX_HEIGHT {
        return Vec::new();
    }
    let popup = WIN.place(
        area,
        dialog_kit::centered(area, want_w.min(area.width), BOX_HEIGHT),
    );
    frame.render_clipped(Clear, popup);
    frame.render_clipped(
        Block::default()
            .title(title)
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th().ui.accent))
            .style(Style::default().bg(th().ui.panel)),
        popup,
    );
    let buf = frame.buffer_mut();
    let mut hits = Vec::new();
    let cross = crate::chrome::close_cross::paint(buf, popup, hovered == Some(RefNamePart::Close));
    if let Some(cross) = cross {
        hits.push((cross, RefNamePart::Close));
    }
    let inner = Rect::new(
        popup.x + 2,
        popup.y + 1,
        popup.width.saturating_sub(4),
        popup.height.saturating_sub(2),
    );
    // The same one-line field as the inline boxes: caret first, the
    // placeholder after it while empty.
    let field = Rect::new(inner.x, inner.y, inner.width, 1);
    crate::pointer_map::field(field);
    editor.drawn(
        MainField::RefName,
        Rect::new(field.x + 1, field.y, field.width.saturating_sub(1), 1),
        false,
    );
    buf.set_line(
        field.x,
        field.y,
        &name_line(
            box_placeholder(op),
            value,
            editor.caret(MainField::RefName),
            usize::from(field.width),
            error.is_some(),
        ),
        field.width,
    );
    if let Some(error) = error {
        dialog_kit::put(
            buf,
            inner.x,
            inner.y + 1,
            &format!("\u{2717} {error}"),
            inner.width,
            Style::default()
                .fg(th().status.danger)
                .bg(th().ui.panel)
                .add_modifier(ratatui::style::Modifier::BOLD),
        );
    }
    let y = inner.y + 3;
    let confirm_w = dialog_kit::button_width(confirm);
    let confirm_x = inner.right().saturating_sub(confirm_w);
    let cancel_x = confirm_x.saturating_sub(2 + dialog_kit::button_width(cancel_label));
    let cancel = dialog_kit::button(
        buf,
        cancel_x,
        y,
        cancel_label,
        ButtonState::Normal {
            hovered: hovered == Some(RefNamePart::Cancel),
            focused: false,
        },
    );
    let ok = dialog_kit::button(
        buf,
        confirm_x,
        y,
        confirm,
        ButtonState::Primary {
            hovered: hovered == Some(RefNamePart::Confirm),
            focused: false,
        },
    );
    hits.push((cancel, RefNamePart::Cancel));
    hits.push((ok, RefNamePart::Confirm));
    WIN.decorate(buf, popup, cross, true);
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_painter_wins_and_the_box_is_the_last_resort() {
        let mut slot = NameSlot::default();
        assert!(slot.may_draw(NameSpot::Refs) && slot.may_draw(NameSpot::Log));
        slot.mark(NameSpot::Refs);
        assert!(!slot.may_draw(NameSpot::Log) && !slot.may_draw(NameSpot::Box));
        assert!(slot.drawn());
    }

    #[test]
    fn the_field_stays_where_it_first_drew() {
        let mut slot = NameSlot::default();
        slot.mark(NameSpot::Box);
        slot.end_frame();
        // The Log row is back in view: the box keeps the field.
        assert!(!slot.may_draw(NameSpot::Log));
        assert!(slot.may_draw(NameSpot::Box));
        slot.reset();
        assert!(slot.may_draw(NameSpot::Log));
    }

    #[test]
    fn a_parked_inline_spot_falls_back_to_the_box_only() {
        let mut slot = NameSlot::default();
        slot.mark(NameSpot::Log);
        slot.end_frame();
        assert!(slot.may_draw(NameSpot::Log) && slot.may_draw(NameSpot::Box));
        assert!(!slot.may_draw(NameSpot::Refs));
        // A frame nobody drew in leaves it where it was.
        slot.end_frame();
        assert!(!slot.may_draw(NameSpot::Refs));
    }
}
