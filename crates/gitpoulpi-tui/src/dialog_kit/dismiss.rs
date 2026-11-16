//! Closing a dialog: the cross, the confirmation box, and the one rule
//! that decides what a close request does (the ux-settings-and-close design
//! section 3).
//!
//! * [`panel_closable`] is [`super::panel`] plus the shared close cross.
//! * [`CloseHit`] remembers where the cross was painted (the hit rect comes
//!   from the paint walk), whether the pointer is on it, and whether it is
//!   flashing because an outside click on a dirty form was swallowed.
//! * [`Dismiss`] + [`dismiss`] decide Close / Swallow / Confirm for a
//!   [`DismissKind`], the same way for every dialog.
//! * [`Confirm`] is the modal "are you sure" box: title, body, buttons.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use super::{button, button_width, centered, hit_at, panel, put, ButtonState};
use crate::chrome::close_cross;
use crate::th;

/// [`panel`] plus the close cross on its top border; returns the cross's
/// hit rect (`None` when the panel is too narrow to carry it).
pub(crate) fn panel_closable(
    buf: &mut Buffer,
    rect: Rect,
    title: &str,
    hovered: bool,
) -> Option<Rect> {
    panel(buf, rect, title);
    close_cross::paint(buf, rect.intersection(buf.area), hovered)
}

/// The close cross of one dialog: where it is, and its transient looks.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CloseHit {
    rect: Option<Rect>,
    hover: bool,
    flash: bool,
}

impl CloseHit {
    /// Paints `rect` as a closable panel and records the cross.
    pub(crate) fn panel(&mut self, buf: &mut Buffer, rect: Rect, title: &str) {
        let flash = std::mem::take(&mut self.flash);
        self.rect = panel_closable(buf, rect, title, self.hover);
        if flash {
            close_cross::flash(buf, rect.intersection(buf.area));
        }
    }

    /// [`CloseHit::panel`] for a dialog that moves (and may resize): the
    /// frame's grips, banner and gesture registration are added.
    pub(crate) fn framed(
        &mut self,
        buf: &mut Buffer,
        rect: Rect,
        title: &str,
        win: &super::Win,
        enabled: bool,
    ) {
        self.panel(buf, rect, title);
        win.decorate(buf, rect, self.rect, enabled);
    }

    /// The cross as last painted.
    #[allow(dead_code)] // tests and the confirmations of C3 / X1
    pub(crate) fn rect(&self) -> Option<Rect> {
        self.rect
    }

    /// Whether `(col, row)` is on the cross.
    pub(crate) fn hit(&self, col: u16, row: u16) -> bool {
        self.rect.is_some_and(|r| r.contains((col, row).into()))
    }

    /// Follows the pointer.
    pub(crate) fn set_hover(&mut self, col: u16, row: u16) {
        self.hover = self.hit(col, row);
    }

    /// Makes the cross blink on the next frame (a swallowed outside click).
    pub(crate) fn request_flash(&mut self) {
        self.flash = true;
    }
}

/// How a dialog treats a close request while it holds typed work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Guard {
    /// Closing never asks and an outside click always closes.
    None,
    /// An outside click on a dirty form is swallowed (the cross flashes);
    /// the cross and Esc still close without asking. Single-line forms.
    IgnoreOutsideWhenDirty,
    #[allow(dead_code)] // first user: the issue / PR forms (C3)
    /// A dirty form asks "discard?" before any close. Multi-line text.
    ConfirmWhenDirty,
}

/// What asked to close the dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Esc and Cancel: callers of the confirmation slices (C3 / X1)
pub(crate) enum DismissKind {
    Cross,
    Esc,
    Outside,
    /// The dialog's own Cancel button.
    Cancel,
}

/// What to do about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DismissAction {
    Close,
    /// Do nothing and make the cross flash.
    Swallow,
    /// Ask before discarding.
    Confirm,
}

/// A dialog that can be closed from several places.
pub(crate) trait Dismiss {
    /// The user typed or changed something worth keeping.
    fn is_dirty(&self) -> bool;

    fn guard(&self) -> Guard {
        Guard::IgnoreOutsideWhenDirty
    }

    fn close_hit(&self) -> &CloseHit;

    fn close_hit_mut(&mut self) -> &mut CloseHit;
}

/// The one decision: close, swallow or ask, for every dialog.
pub(crate) fn dismiss(dialog: &dyn Dismiss, kind: DismissKind) -> DismissAction {
    let dirty = dialog.is_dirty();
    match (dialog.guard(), kind) {
        (Guard::None, _) => DismissAction::Close,
        (Guard::ConfirmWhenDirty, _) if dirty => DismissAction::Confirm,
        (Guard::IgnoreOutsideWhenDirty, DismissKind::Outside) if dirty => DismissAction::Swallow,
        _ => DismissAction::Close,
    }
}

/// What a [`Confirm`] box answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfirmEvent<C> {
    /// Still open.
    Stay,
    /// A button was pressed (or Esc / the cross / an outside click, which
    /// answer the Cancel choice).
    Chose(C),
}

/// Every confirmation moves (never resizes: its size follows its text) and
/// they share one place.
const CONFIRM_WIN: super::Win = super::Win::movable("confirm");

/// A modal confirmation: `[(label, choice)]` buttons, Left / Right / Tab
/// move, Enter presses, Esc / the cross / a click outside is Cancel.
#[derive(Debug, Clone)]
pub(crate) struct Confirm<C: Copy + PartialEq> {
    pub title: String,
    pub body: String,
    buttons: Vec<(String, C)>,
    cancel: C,
    /// Index of the focused button.
    pub selected: usize,
    /// Index of the button drawn as the primary action, if any.
    primary: Option<usize>,
    hover: Option<usize>,
    panel: Rect,
    close: CloseHit,
    hits: Vec<(Rect, usize)>,
    /// A line break in the body ends the line (a list, paragraphs) instead
    /// of flowing into the text.
    keep_breaks: bool,
    /// The narrowest the box is drawn (a body that carries an address).
    min_width: u16,
    /// The buttons are a list, one per row (long labels, a pick list): the
    /// arrows and `Ctrl+J` / `Ctrl+K` move through them like Left / Right.
    stacked: bool,
}

impl<C: Copy + PartialEq> Confirm<C> {
    /// `cancel` is the choice Esc and outside clicks answer; it should be
    /// one of `buttons`. The first button is focused.
    pub(crate) fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        buttons: Vec<(String, C)>,
        cancel: C,
    ) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            buttons,
            cancel,
            selected: 0,
            primary: None,
            hover: None,
            panel: Rect::default(),
            close: CloseHit::default(),
            hits: Vec::new(),
            keep_breaks: false,
            min_width: 44,
            stacked: false,
        }
    }

    /// Lays the buttons out one per row instead of side by side.
    pub(crate) fn stacked(mut self) -> Self {
        self.stacked = true;
        self
    }

    /// Draws the box at least `width` cells wide (when the screen allows).
    pub(crate) fn min_width(mut self, width: u16) -> Self {
        self.min_width = width;
        self
    }

    /// Keeps the body's line breaks (each line is still wrapped to the box).
    pub(crate) fn keep_breaks(mut self) -> Self {
        self.keep_breaks = true;
        self
    }

    /// Focuses (and highlights as primary) the button for `choice`.
    #[allow(dead_code)] // used by the confirmations of C3 / X1
    pub(crate) fn with_primary(mut self, choice: C) -> Self {
        if let Some(i) = self.buttons.iter().position(|(_, c)| *c == choice) {
            self.primary = Some(i);
            self.selected = i;
        }
        self
    }

    /// Highlights the button for `choice` as the primary action without
    /// moving the focus.
    pub(crate) fn primary(mut self, choice: C) -> Self {
        self.primary = self.buttons.iter().position(|(_, c)| *c == choice);
        self
    }

    fn press(&self, index: usize) -> ConfirmEvent<C> {
        ConfirmEvent::Chose(self.buttons[index].1)
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> ConfirmEvent<C> {
        let n = self.buttons.len().max(1);
        match key.code {
            KeyCode::Esc => return ConfirmEvent::Chose(self.cancel),
            KeyCode::Left | KeyCode::BackTab => self.selected = (self.selected + n - 1) % n,
            KeyCode::Right | KeyCode::Tab => self.selected = (self.selected + 1) % n,
            KeyCode::Up if self.stacked => self.selected = (self.selected + n - 1) % n,
            KeyCode::Down if self.stacked => self.selected = (self.selected + 1) % n,
            KeyCode::Char('k') if self.stacked && key.modifiers == KeyModifiers::CONTROL => {
                self.selected = (self.selected + n - 1) % n
            }
            KeyCode::Char('j') if self.stacked && key.modifiers == KeyModifiers::CONTROL => {
                self.selected = (self.selected + 1) % n
            }
            KeyCode::Enter if !self.buttons.is_empty() => return self.press(self.selected),
            _ => {}
        }
        ConfirmEvent::Stay
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> ConfirmEvent<C> {
        let hit = hit_at(&self.hits, mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Moved => {
                self.hover = hit;
                self.close.set_hover(mouse.column, mouse.row);
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(i) = hit {
                    return self.press(i);
                }
                let outside = !self.panel.contains((mouse.column, mouse.row).into());
                if outside || self.close.hit(mouse.column, mouse.row) {
                    return ConfirmEvent::Chose(self.cancel);
                }
            }
            _ => {}
        }
        ConfirmEvent::Stay
    }

    /// Paints the box centered in `bounds`; returns the button hit rects
    /// (the choice each one answers), also used by `on_mouse`.
    pub(crate) fn render(&mut self, buf: &mut Buffer, bounds: Rect) -> Vec<(Rect, C)> {
        self.hits.clear();
        let widths = self.buttons.iter().map(|(l, _)| button_width(l));
        let buttons_w: u16 = if self.stacked {
            widths.max().unwrap_or(0)
        } else {
            widths.sum::<u16>() + 2 * self.buttons.len().saturating_sub(1) as u16
        };
        // Stacked, the buttons take a row each below the text.
        let button_rows = if self.stacked {
            self.buttons.len().max(1) as u16
        } else {
            1
        };
        let want_w = (crate::display_width(&self.title) as u16 + 8)
            .max(buttons_w + 4)
            .max(self.min_width)
            .min(bounds.width);
        let inner_w = want_w.saturating_sub(4).max(1) as usize;
        let lines: Vec<String> = if self.keep_breaks {
            self.body
                .split('\n')
                .flat_map(|line| {
                    let wrapped = crate::ui_kit::wrap_words(line, inner_w);
                    if wrapped.is_empty() {
                        vec![String::new()]
                    } else {
                        wrapped
                    }
                })
                .collect()
        } else {
            crate::ui_kit::wrap_words(&self.body, inner_w)
        };
        // A word wider than the box (a long web address) is broken over
        // lines rather than cut at the frame: the whole text stays readable.
        let lines: Vec<String> = lines
            .into_iter()
            .flat_map(|line| hard_break(&line, inner_w))
            .collect();
        let want_h = lines.len() as u16 + 3 + button_rows;
        let panel_rect = CONFIRM_WIN.place(bounds, centered(bounds, want_w, want_h));
        self.panel = panel_rect;
        if panel_rect.width < 8 || panel_rect.height < 5 {
            return Vec::new();
        }
        self.close
            .framed(buf, panel_rect, &self.title, &CONFIRM_WIN, true);
        let text = Style::default().fg(th().ui.text).bg(th().ui.panel);
        let room = panel_rect.height.saturating_sub(3 + button_rows) as usize;
        for (i, line) in lines.iter().take(room).enumerate() {
            put(
                buf,
                panel_rect.x + 2,
                panel_rect.y + 1 + i as u16,
                line,
                panel_rect.width - 4,
                text,
            );
        }
        let mut y = panel_rect.bottom() - 1 - button_rows;
        let mut x = panel_rect.x + panel_rect.width.saturating_sub(buttons_w) / 2;
        let mut out = Vec::new();
        for (i, (label, choice)) in self.buttons.iter().enumerate() {
            let hovered = self.hover == Some(i);
            let focused = self.selected == i;
            let state = if self.primary == Some(i) {
                ButtonState::Primary { hovered, focused }
            } else {
                ButtonState::Normal { hovered, focused }
            };
            let r = button(buf, x, y, label, state);
            self.hits.push((r, i));
            out.push((r, *choice));
            if self.stacked {
                y += 1;
            } else {
                x += r.width + 2;
            }
        }
        out
    }
}

/// `line` cut into pieces of at most `width` cells (`line` itself when it
/// fits), never shortened.
fn hard_break(line: &str, width: usize) -> Vec<String> {
    if crate::display_width(line) <= width {
        return vec![line.to_string()];
    }
    let mut out = Vec::new();
    let mut piece = String::new();
    let mut piece_w = 0;
    for ch in line.chars() {
        let w = crate::display_width(ch.encode_utf8(&mut [0; 4]));
        if piece_w + w > width && !piece.is_empty() {
            out.push(std::mem::take(&mut piece));
            piece_w = 0;
        }
        piece.push(ch);
        piece_w += w;
    }
    if !piece.is_empty() {
        out.push(piece);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    struct Form {
        dirty: bool,
        guard: Guard,
        close: CloseHit,
    }

    impl Dismiss for Form {
        fn is_dirty(&self) -> bool {
            self.dirty
        }
        fn guard(&self) -> Guard {
            self.guard
        }
        fn close_hit(&self) -> &CloseHit {
            &self.close
        }
        fn close_hit_mut(&mut self) -> &mut CloseHit {
            &mut self.close
        }
    }

    fn form(dirty: bool, guard: Guard) -> Form {
        Form {
            dirty,
            guard,
            close: CloseHit::default(),
        }
    }

    #[test]
    fn a_confirmation_moves_and_arranges_but_does_not_resize() {
        let mut confirm = Confirm::new(
            "Sure?",
            "This cannot be undone.",
            vec![("Yes".to_string(), 1u8), ("No".to_string(), 0u8)],
            0u8,
        );
        super::super::frame::testing::check_gestures(false, || {
            let area = Rect::new(0, 0, 160, 50);
            let mut buf = Buffer::empty(area);
            confirm.render(&mut buf, area);
        });
    }

    #[test]
    fn dismiss_decides_close_swallow_or_confirm() {
        use DismissAction::*;
        use DismissKind::*;
        let single = |d| form(d, Guard::IgnoreOutsideWhenDirty);
        for kind in [Cross, Esc, Cancel] {
            assert_eq!(dismiss(&single(true), kind), Close, "{kind:?}");
        }
        assert_eq!(dismiss(&single(false), Outside), Close);
        assert_eq!(dismiss(&single(true), Outside), Swallow);
        let multi = |d| form(d, Guard::ConfirmWhenDirty);
        assert_eq!(dismiss(&multi(false), Esc), Close);
        for kind in [Cross, Esc, Outside, Cancel] {
            assert_eq!(dismiss(&multi(true), kind), Confirm, "{kind:?}");
        }
        assert_eq!(dismiss(&form(true, Guard::None), Outside), Close);
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn click(col: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    fn confirm() -> Confirm<u8> {
        Confirm::new(
            "Discard?",
            "You typed a description.",
            vec![("Cancel".into(), 0), ("Discard".into(), 1)],
            0,
        )
    }

    #[test]
    fn confirm_keys_move_press_and_cancel() {
        let mut c = confirm();
        assert_eq!(c.on_key(key(KeyCode::Right)), ConfirmEvent::Stay);
        assert_eq!(c.selected, 1);
        assert_eq!(c.on_key(key(KeyCode::Tab)), ConfirmEvent::Stay);
        assert_eq!(c.selected, 0);
        assert_eq!(c.on_key(key(KeyCode::Left)), ConfirmEvent::Stay);
        assert_eq!(c.on_key(key(KeyCode::Enter)), ConfirmEvent::Chose(1));
        assert_eq!(c.on_key(key(KeyCode::Esc)), ConfirmEvent::Chose(0));
    }

    #[test]
    fn confirm_paints_a_cross_and_hit_rects_equal_the_buttons() {
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        let mut c = confirm();
        let hits = c.render(&mut buf, area);
        let text = super::super::buffer_text(&buf);
        assert!(
            text.contains('\u{2715}') && text.contains("Discard?"),
            "{text}"
        );
        assert!(!text.contains("Esc"), "{text}");
        assert_eq!(hits.len(), 2);
        for (rect, choice) in &hits {
            let label: String = (rect.x..rect.right())
                .map(|x| buf[(x, rect.y)].symbol())
                .collect();
            assert_eq!(
                label.trim(),
                if *choice == 0 { "Cancel" } else { "Discard" }
            );
        }
        let discard = hits[1].0;
        assert_eq!(
            c.on_mouse(click(discard.x + 1, discard.y)),
            ConfirmEvent::Chose(1)
        );
        assert_eq!(
            c.on_mouse(click(0, 0)),
            ConfirmEvent::Chose(0),
            "outside = cancel"
        );
        let inside = c.panel;
        assert_eq!(
            c.on_mouse(click(inside.x + 2, inside.y + 1)),
            ConfirmEvent::Stay,
            "inside the text does nothing"
        );
        let cross = c.close.rect().expect("a cross");
        assert_eq!(
            c.on_mouse(click(cross.x + 1, cross.y)),
            ConfirmEvent::Chose(0)
        );
    }
}
