//! The "Open in" combobox of a binary file: the button painted with
//! [`Combo::render_field`] and a popup listing the applications the OS
//! registers for the file's type, the default first and marked with a bullet.
//! A clone of `image_view/edit_in.rs`, which explains the shape: the popup is
//! modal, its rows and frame go to the front of the frame's hit list from the
//! very paint that drew them, and a click away closes it and is consumed.
//!
//! The list is built when the popup opens; if the lookup is still running it
//! holds one "looking up apps..." row and fills in as soon as the answer
//! lands (the paint notices). Neither showing nor launching changes git
//! state: no journal entry, the Undo button is unaffected.

use std::path::Path;
use std::sync::{Arc, Mutex};

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::Frame;

use gitpoulpi_config::os_apps::Found;
use gitpoulpi_i18n::msg::open_with as msg;

use super::{Lookup, Subject};
use crate::chrome::HitTarget;
use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::daemon_link::DaemonLink;
use crate::display_width;

/// The value of the row that is not an application.
const NOTHING: &str = "nothing";

/// What a pick chose: run application `n` of the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Pick(pub(crate) usize);

struct Open {
    combo: Combo,
    subject: Subject,
    /// The lookup was still running when the list was built.
    waiting: bool,
    /// The menu and the row of it that opened the list: it hangs beside them
    /// like a submenu. `None` from the button, a key or the palette.
    parent: Option<(Rect, u16)>,
}

static POPUP: Mutex<Option<Open>> = Mutex::new(None);
/// Where the popup hangs from: the button as painted last.
static ANCHOR: Mutex<Option<Rect>> = Mutex::new(None);

/// Cells the button takes: ` Open in v `.
pub(crate) fn button_width() -> u16 {
    display_width(msg::BUTTON.get()) as u16 + 4
}

/// Paints the button as a combobox field into `rect` and remembers it as the
/// popup's anchor.
pub(crate) fn render_button(buf: &mut Buffer, rect: Rect, hovered: bool) {
    let mut field = Combo::new(msg::BUTTON.get());
    field.set_options(vec![ComboOption::plain(msg::BUTTON.get())]);
    field.select(Some(msg::BUTTON.get().to_string()));
    field.render_field(buf, rect, is_open(), hovered);
    if let Ok(mut a) = ANCHOR.lock() {
        *a = Some(rect);
    }
}

pub(crate) fn is_open() -> bool {
    POPUP
        .lock()
        .is_ok_and(|p| p.as_ref().is_some_and(|o| o.combo.is_open()))
}

#[cfg(test)]
pub(crate) fn close() {
    if let Ok(mut p) = POPUP.lock() {
        *p = None;
    }
}

fn options(found: Option<&Found>) -> Vec<ComboOption> {
    match found {
        None => vec![ComboOption::new(
            format!("  {}", msg::LOOKING_UP.get()),
            NOTHING,
        )],
        Some(f) if f.apps.is_empty() => vec![ComboOption::new(
            format!("  {}", msg::NONE_FOUND.get()),
            NOTHING,
        )],
        Some(f) => f
            .apps
            .iter()
            .enumerate()
            .map(|(i, app)| {
                let mark = if app.default { "\u{2022} " } else { "  " };
                ComboOption::new(format!("{mark}{}", app.name), i.to_string())
            })
            .collect(),
    }
}

/// The rows of the list for `found`, as shown.
#[cfg(test)]
pub(super) fn option_labels(found: Option<&Found>) -> Vec<String> {
    options(found).into_iter().map(|o| o.label).collect()
}

fn answer(name: &str) -> Option<Arc<Found>> {
    match super::lookup(name) {
        Lookup::Ready(found) => Some(found),
        Lookup::Pending { .. } => None,
    }
}

fn build(found: Option<&Found>) -> Combo {
    let mut combo = Combo::new("").with_click_picks();
    combo.set_options(options(found));
    if found.is_some_and(|f| !f.apps.is_empty()) {
        combo.select(Some("0".to_string()));
    }
    combo.open();
    combo
}

/// Forgets the button's place; the next paint that draws one sets it again.
/// Called at the start of every frame, so a list opened by a key or the
/// palette while no button is shown never hangs from a stale rect.
pub(crate) fn forget_anchor() {
    if let Ok(mut a) = ANCHOR.lock() {
        *a = None;
    }
}

/// Opens the popup on `subject`, hanging from the button.
pub(crate) fn open(subject: Subject) {
    open_at(subject, None);
}

/// Opens the popup as the continuation of the context-menu row that was just
/// activated (mouse or keyboard): beside that menu, clamped to the screen.
pub(crate) fn open_from_menu(subject: Subject) {
    open_at(subject, crate::context_menu::activated_row());
}

#[cfg(test)]
pub(super) fn open_for_test(subject: Subject, parent: Option<(Rect, u16)>) {
    open_at(subject, parent);
}

fn open_at(subject: Subject, parent: Option<(Rect, u16)>) {
    let found = answer(&subject.name());
    let combo = build(found.as_deref());
    if let Ok(mut p) = POPUP.lock() {
        *p = Some(Open {
            combo,
            waiting: found.is_none(),
            parent,
            subject,
        });
    }
}

fn pick_of(value: &str) -> Option<Pick> {
    value.parse().ok().map(Pick)
}

fn settle(slot: &mut Option<Open>, outcome: ComboOutcome) -> Option<(Pick, Subject)> {
    let pick = match outcome {
        ComboOutcome::Picked(value) | ComboOutcome::Custom(value) => pick_of(&value),
        _ => None,
    };
    let subject = slot.as_ref().map(|o| o.subject.clone());
    if slot.as_ref().is_some_and(|o| !o.combo.is_open()) {
        *slot = None;
    }
    pick.zip(subject)
}

/// A key while the popup is open (it swallows them all).
pub(crate) fn on_key(key: KeyEvent) -> Option<(Pick, Subject)> {
    let mut slot = POPUP.lock().ok()?;
    let outcome = slot.as_mut()?.combo.on_key(key);
    settle(&mut slot, outcome)
}

/// A mouse event while the popup is open (it swallows them all).
pub(crate) fn on_mouse(mouse: MouseEvent) -> Option<(Pick, Subject)> {
    let mut slot = POPUP.lock().ok()?;
    let outcome = slot.as_mut()?.combo.on_mouse(mouse);
    settle(&mut slot, outcome)
}

/// Pasted text goes to the search line.
pub(crate) fn paste(text: &str) {
    if let Ok(mut p) = POPUP.lock() {
        if let Some(open) = p.as_mut() {
            open.combo.paste(text);
        }
    }
}

/// Paints the open popup over the frame and registers its rows, then the whole
/// popup, at the front of `hits`.
pub(crate) fn render_popup(
    frame: &mut Frame,
    hovered_row: Option<usize>,
    hits: &mut Vec<(Rect, HitTarget)>,
) {
    let bounds = frame.area();
    let Ok(mut slot) = POPUP.lock() else {
        return;
    };
    let Some(open) = slot.as_mut() else {
        return;
    };
    // The answer landed while the popup was open: fill the list in.
    if open.waiting {
        if let Some(found) = answer(&open.subject.name()) {
            open.combo = build(Some(&found));
            open.waiting = false;
        }
    }
    let combo = &mut open.combo;
    // The button as painted; else (opened from a menu or the palette while the
    // button is hidden) the top right corner.
    let corner = || {
        let w = button_width().min(bounds.width);
        Rect::new(bounds.right().saturating_sub(w), bounds.y + 1, w, 1)
    };
    let anchor = ANCHOR
        .lock()
        .ok()
        .and_then(|a| *a)
        .filter(|a| a.right() <= bounds.right() && a.bottom() <= bounds.bottom())
        .unwrap_or_else(corner);
    combo.set_anchor(anchor);
    combo.set_flyout(open.parent);
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

/// Runs the application a pick chose: the status line's answer.
pub(crate) fn apply(pick: Pick, subject: &Subject, repo_root: &Path, link: &DaemonLink) -> String {
    match answer(&subject.name()) {
        Some(found) => super::run(&found.apps, pick.0, subject, repo_root, link),
        None => msg::still_looking(subject.name()),
    }
}

/// "Open in Default app": the first application of the list.
pub(crate) fn run_default(subject: &Subject, repo_root: &Path, link: &DaemonLink) -> String {
    match answer(&subject.name()) {
        None => msg::still_looking(subject.name()),
        Some(found) if found.apps.is_empty() => msg::no_app(subject.name()),
        Some(found) => super::run(&found.apps, 0, subject, repo_root, link),
    }
}
