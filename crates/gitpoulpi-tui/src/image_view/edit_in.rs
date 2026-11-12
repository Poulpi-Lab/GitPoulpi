//! The image view's "Edit in" combobox: the header button painted with
//! [`Combo::render_field`] and a popup listing `image_editor::entries()`, the
//! default marked `•`, a last "Configure…" row (plan D9, slice S7). A clone of
//! `open_in_bar.rs`, which explains the shape.
//!
//! The popup is modal: while it is open it owns the keyboard and the mouse
//! (a click away closes it and is consumed). Its rows and its frame are put at
//! the front of the frame's hit list from the very paint that drew them
//! (`Combo::item_rects`), as `HitTarget::OpenInItem` / `OpenInPopup`: only one
//! of the two popups can be open at a time, and the popup handles its own
//! clicks, so the targets only exist to shield and to light the hovered row.
//!
//! Which file is edited ([`Target`]): the image pane's own picture (the
//! working-tree file for a WIP image, else a read-only temp copy), or a file
//! path given by a context menu. Neither showing nor editing an image changes
//! git state: no journal entry, the Undo button is unaffected.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::chrome::HitTarget;
use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::display_width;
use gitpoulpi_i18n::msg::image_edit_in as msg;

/// The value of the last row.
const CONFIGURE: &str = "configure";
/// The value of the "looking up apps..." row: not a choice.
const WAIT: &str = "wait";
/// No entry was run yet.
const NONE: usize = usize::MAX;
/// How long a menu's request to open the list waits for the picture to load.
const PENDING_FOR: Duration = Duration::from_secs(8);

/// What the popup edits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Target {
    /// The picture the image pane shows.
    Pane,
    /// This file, absolute (a working-tree image picked from a menu).
    File(PathBuf),
}

/// What a pick in the popup chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pick {
    /// Run entry `n` of `image_editor::entries()`.
    Entry(usize),
    /// "Configure…": Settings, Images.
    Configure,
}

static POPUP: Mutex<Option<Combo>> = Mutex::new(None);
static TARGET: Mutex<Option<Target>> = Mutex::new(None);
/// Where the popup hangs from: the button as painted last.
static ANCHOR: Mutex<Option<Rect>> = Mutex::new(None);
/// The context-menu row that opened the list: it hangs beside it like a
/// submenu. `None` from the button, a key or the palette.
static PARENT: Mutex<Option<(Rect, u16)>> = Mutex::new(None);
/// The menu row of a request waiting for its picture ([`request_when_ready`]).
static PENDING_PARENT: Mutex<Option<(Rect, u16)>> = Mutex::new(None);
static LAST_USED: AtomicUsize = AtomicUsize::new(NONE);
/// A menu asked for the list as soon as its picture is loaded.
static PENDING: Mutex<Option<Instant>> = Mutex::new(None);
static TIP: Mutex<Option<(String, &'static str)>> = Mutex::new(None);
/// The file name the popup was opened for (its type picks the OS apps), and
/// whether the OS lookup for it was still running when the list was built.
static OPENED_FOR: Mutex<Option<(String, bool)>> = Mutex::new(None);

/// Cells the button takes: ` Edit in ▾ `.
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
        .is_ok_and(|p| p.as_ref().is_some_and(Combo::is_open))
}

#[cfg(test)]
pub(crate) fn close() {
    if let Ok(mut p) = POPUP.lock() {
        *p = None;
    }
    if let Ok(mut p) = PENDING.lock() {
        *p = None;
    }
}

/// The name of the picture the pane shows (Diff view first, else File View).
fn pane_name() -> Option<String> {
    if super::split::active() {
        super::split::current_name()
    } else {
        super::pane::current_name()
    }
}

fn name_of(target: &Target) -> String {
    match target {
        Target::File(path) => path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        Target::Pane => pane_name().unwrap_or_default(),
    }
}

fn options(labels: &[String], default: usize, waiting: bool) -> Vec<ComboOption> {
    let mut out: Vec<ComboOption> = labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let mark = if i == default { "\u{2022} " } else { "  " };
            ComboOption::new(format!("{mark}{label}"), i.to_string())
        })
        .collect();
    if waiting {
        out.push(ComboOption::new(
            format!("  {}", gitpoulpi_i18n::msg::open_with::LOOKING_UP.get()),
            WAIT,
        ));
    }
    out.push(ComboOption::new(
        format!("  {}", msg::CONFIGURE.get()),
        CONFIGURE,
    ));
    out
}

/// Opens the popup on `target`, highlighting the last-used entry (else the
/// default).
pub(crate) fn open(target: Target) {
    let name = name_of(&target);
    open_with_parent(target, name, None);
}

/// [`open`] as the continuation of the context-menu row just activated (mouse
/// or keyboard): beside that menu, clamped to the screen.
pub(crate) fn open_from_menu(target: Target) {
    let name = name_of(&target);
    open_with_parent(target, name, crate::context_menu::activated_row());
}

/// [`open`] for a picture whose file name the caller knows (the pane opens it
/// from inside its own paint, with its state borrowed).
pub(super) fn open_for(target: Target, name: String) {
    let parent = PENDING_PARENT.lock().ok().and_then(|mut p| p.take());
    open_with_parent(target, name, parent);
}

fn open_with_parent(target: Target, name: String, parent: Option<(Rect, u16)>) {
    if let Ok(mut p) = PARENT.lock() {
        *p = parent;
    }
    let waiting = !name.is_empty() && crate::open_with::ready(&name).is_none();
    let labels = crate::image_editor::entries_for(&name);
    let default = crate::image_editor::default_index_for(&name);
    if let Ok(mut o) = OPENED_FOR.lock() {
        *o = Some((name, waiting));
    }
    let mut combo = Combo::new("").with_click_picks();
    combo.set_options(options(&labels, default, waiting));
    let last = LAST_USED.load(Ordering::Relaxed);
    let start = if last < labels.len() { last } else { default };
    if start < labels.len() {
        combo.select(Some(start.to_string()));
    }
    combo.open();
    if let Ok(mut t) = TARGET.lock() {
        *t = Some(target);
    }
    if let Ok(mut p) = POPUP.lock() {
        *p = Some(combo);
    }
}

/// A context menu asked for the list on the picture it is about to open: the
/// pane opens it once that picture is ready.
pub(crate) fn request_when_ready() {
    if let Ok(mut p) = PENDING_PARENT.lock() {
        *p = crate::context_menu::activated_row();
    }
    if let Ok(mut p) = PENDING.lock() {
        *p = Some(Instant::now());
    }
}

/// Whether a request is waiting (and not stale). Taking it clears it.
pub(crate) fn take_pending() -> bool {
    PENDING
        .lock()
        .ok()
        .and_then(|mut p| p.take())
        .is_some_and(|since| since.elapsed() < PENDING_FOR)
}

/// Whether a request is waiting, without taking it.
pub(crate) fn pending() -> bool {
    PENDING
        .lock()
        .is_ok_and(|p| p.is_some_and(|since| since.elapsed() < PENDING_FOR))
}

fn pick_of(value: &str) -> Option<Pick> {
    if value == CONFIGURE {
        Some(Pick::Configure)
    } else {
        value.parse().ok().map(Pick::Entry)
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
    let Some(combo) = slot.as_mut() else {
        return;
    };
    // The button as painted; else (opened from a menu or the palette while the
    // button is hidden) the top right corner.
    let anchor = ANCHOR.lock().ok().and_then(|a| *a).unwrap_or_else(|| {
        let w = button_width().min(bounds.width);
        Rect::new(bounds.right().saturating_sub(w), bounds.y + 1, w, 1)
    });
    let anchor = if anchor.right() > bounds.right() || anchor.bottom() > bounds.bottom() {
        let w = button_width().min(bounds.width);
        Rect::new(bounds.right().saturating_sub(w), bounds.y + 1, w, 1)
    } else {
        anchor
    };
    // The OS's apps landed while the list was open: add them.
    if let Ok(mut opened) = OPENED_FOR.lock() {
        if let Some((name, waiting @ true)) = opened.as_mut() {
            if crate::open_with::ready(name).is_some() {
                *waiting = false;
                let labels = crate::image_editor::entries_for(name);
                let default = crate::image_editor::default_index_for(name);
                let keep = combo.highlighted_value().map(str::to_string);
                combo.set_options(options(&labels, default, false));
                combo.select(keep);
            }
        }
    }
    combo.set_anchor(anchor);
    combo.set_flyout(PARENT.lock().ok().and_then(|p| *p));
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

/// The file the current target stands for, and a notice when it is a copy.
fn resolve(target: &Target, repo_root: &Path) -> Result<(PathBuf, Option<String>), String> {
    match target {
        Target::File(path) => Ok((path.clone(), None)),
        // The Diff view's pictures when it shows them, else the File View's.
        Target::Pane if super::split::active() => super::split::edit_file(repo_root),
        Target::Pane => super::pane::edit_file(repo_root),
    }
}

/// Runs entry `index` on the popup's target: the status line's answer.
/// Remembers it as the popup's next starting highlight.
pub(crate) fn run_entry(index: usize, repo_root: &Path) -> String {
    LAST_USED.store(index, Ordering::Relaxed);
    let target = TARGET
        .lock()
        .ok()
        .and_then(|t| t.clone())
        .unwrap_or(Target::Pane);
    run_on(index, &target, repo_root)
}

/// Runs entry `index` on `target`.
pub(crate) fn run_on(index: usize, target: &Target, repo_root: &Path) -> String {
    let (file, notice) = match resolve(target, repo_root) {
        Ok(found) => found,
        Err(why) => return why,
    };
    match crate::image_editor::run(index, &file) {
        Ok(opening) => notice.unwrap_or(opening),
        Err(why) => crate::activity_log::report_problem_clear(
            crate::activity_log::ProblemSource::Tool,
            &why,
        ),
    }
}

/// "Edit in default": the default entry on `target`.
pub(crate) fn run_default(target: &Target, repo_root: &Path) -> String {
    let name = name_of(target);
    if crate::image_editor::entries_for(&name).is_empty() {
        return msg::NO_ENTRIES.get().to_string();
    }
    let index = crate::image_editor::default_index_for(&name);
    LAST_USED.store(index, Ordering::Relaxed);
    run_on(index, target, repo_root)
}

/// The tooltip: "Edit in <default entry>…", the plain label with no entry.
pub(crate) fn tooltip() -> &'static str {
    // No file name: the tooltip is read while the pane is being painted, with
    // its state borrowed. The default is a configured editor either way.
    let labels = crate::image_editor::entries_for("");
    let Some(label) = labels.get(crate::image_editor::default_index_for("")) else {
        return msg::BUTTON.get();
    };
    let Ok(mut cached) = TIP.lock() else {
        return msg::BUTTON.get();
    };
    if let Some((for_label, text)) = cached.as_ref() {
        if for_label == label {
            return text;
        }
    }
    let text: &'static str = Box::leak(msg::tooltip(label).into_boxed_str());
    *cached = Some((label.clone(), text));
    text
}

/// Whether `path` names a picture a menu should offer "Edit in..." for.
pub(crate) fn is_image_path(path: &Path) -> bool {
    super::pane::has_image_extension(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn the_popup_rows_lead_the_hit_list_and_the_slot_rect_is_the_painted_button() {
        let _serial = crate::tab_dir::TEST_SERIAL
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        close();
        let row = Rect::new(0, 3, 80, 1);
        let slot = super::super::header::edit_slot(row);
        let rect = slot.rect.expect("a wide row has the slot");
        assert_eq!(rect.right(), row.right(), "at the right end");
        assert_eq!(slot.hits.len(), 1);
        assert_eq!(slot.hits[0].0, rect, "the hit is the painted rect");
        let narrow = super::super::header::edit_slot(Rect::new(0, 0, 20, 1));
        assert!(narrow.rect.is_none() && narrow.width == 0);

        open(Target::Pane);
        let mut hits = vec![(Rect::new(0, 0, 5, 1), HitTarget::OpenRepositoryButton)];
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| {
                render_button(frame.buffer_mut(), rect, false);
                render_popup(frame, None, &mut hits);
            })
            .unwrap();
        assert!(matches!(
            hits.last(),
            Some((_, HitTarget::OpenRepositoryButton))
        ));
        assert!(hits
            .iter()
            .take(hits.len() - 1)
            .all(|(_, t)| matches!(t, HitTarget::OpenInItem(_) | HitTarget::OpenInPopup)));
        // At least the default app row.
        let items = hits
            .iter()
            .filter(|(_, t)| matches!(t, HitTarget::OpenInItem(_)))
            .count();
        assert!(items >= 1, "{hits:?}");
        close();
    }
}
