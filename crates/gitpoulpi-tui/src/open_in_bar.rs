//! The tab bar's "Open in" combobox: the button painted with
//! [`Combo::render_field`] and the popup listing `tab_dir::entries()`, the
//! default marked `•`, a last "Configure…" row.
//!
//! The popup is modal: while it is open it owns the keyboard and the mouse
//! (a click away closes it and is consumed), so nothing behind it hovers or
//! reacts. Its rows are registered at the front of the frame's hit list from
//! the very paint that drew them (`Combo::item_rects`). The keyboard twins are
//! the palette's "Tab: Open in…" ([`open`]) and "Tab: Open in default"
//! ([`run_default`]), both bindable.
//!
//! Like the other tab-bar state this is process-wide: the window restarts on
//! every tab switch, the last-used entry (the popup's starting highlight) is
//! for the session and is not persisted.

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::chrome::HitTarget;
use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::display_width;
use gitpoulpi_i18n::msg::open_in_bar as msg;

/// The value of the last row.
const CONFIGURE: &str = "configure";
/// No entry was run yet.
const NONE: usize = usize::MAX;

static POPUP: Mutex<Option<Combo>> = Mutex::new(None);
/// Where the popup hangs from: the button as painted, else the bar's right end.
static ANCHOR: Mutex<Option<Rect>> = Mutex::new(None);
static LAST_USED: AtomicUsize = AtomicUsize::new(NONE);
/// The tooltip text last built, so the hover path hands out `&'static str`
/// without leaking on every frame.
static TIP: Mutex<Option<(String, &'static str)>> = Mutex::new(None);

/// What a pick in the popup chose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Pick {
    /// Run entry `n` of `tab_dir::entries()`.
    Entry(usize),
    /// "Configure…": the Settings screen.
    Configure,
    /// A row of the agent chooser ("Tab: Open AI Agent Here" with several
    /// local agents): the AI profile id.
    Agent(String),
}

/// The value prefix of the agent chooser's rows.
const AGENT_PREFIX: &str = "agent:";

/// Cells the button takes: ` Open in ▾ `, the label plus the field's padding
/// and arrow, the width every other combobox field gives them.
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
    set_anchor(Some(rect));
}

fn set_anchor(rect: Option<Rect>) {
    if let Ok(mut a) = ANCHOR.lock() {
        *a = rect;
    }
}

/// Called each frame before the tab bar paints: the popup falls back to the
/// bar's right end when the button is not drawn (hidden on the launch folder,
/// a narrow window) and the list was opened from the palette.
pub(crate) fn set_bar(bar: Rect) {
    let w = button_width().min(bar.width);
    set_anchor(Some(Rect::new(bar.right().saturating_sub(w), bar.y, w, 1)));
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
}

/// The popup's rows: entries (the default marked), then "Configure...".
fn options(labels: &[String], default: usize) -> Vec<ComboOption> {
    let mut out: Vec<ComboOption> = labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let mark = if i == default { "\u{2022} " } else { "  " };
            ComboOption::new(format!("{mark}{label}"), i.to_string())
        })
        .collect();
    out.push(ComboOption::new(
        format!("  {}", msg::CONFIGURE.get()),
        CONFIGURE,
    ));
    out
}

/// Opens the popup, highlighting the last-used entry (else the default).
pub(crate) fn open() {
    let labels = crate::tab_dir::entries();
    let default = crate::tab_dir::default_index();
    let mut combo = Combo::new("").with_click_picks();
    combo.set_options(options(&labels, default));
    let last = LAST_USED.load(Ordering::Relaxed);
    let start = if last < labels.len() { last } else { default };
    if start < labels.len() {
        combo.select(Some(start.to_string()));
    }
    combo.open();
    if let Ok(mut p) = POPUP.lock() {
        *p = Some(combo);
    }
}

/// Opens the agent chooser: the same popup over the tab bar, one row per
/// local AI agent profile (`id`, name), no "Configure..." row.
pub(crate) fn open_agents(agents: &[(String, String)]) {
    let rows = agents
        .iter()
        .map(|(id, name)| ComboOption::new(format!("  {name}"), format!("{AGENT_PREFIX}{id}")))
        .collect();
    let mut combo = Combo::new("").with_click_picks();
    combo.set_options(rows);
    if let Some((id, _)) = agents.first() {
        combo.select(Some(format!("{AGENT_PREFIX}{id}")));
    }
    combo.open();
    if let Ok(mut p) = POPUP.lock() {
        *p = Some(combo);
    }
}

fn pick_of(value: &str) -> Option<Pick> {
    if let Some(id) = value.strip_prefix(AGENT_PREFIX) {
        return Some(Pick::Agent(id.to_string()));
    }
    if value == CONFIGURE {
        Some(Pick::Configure)
    } else {
        value.parse().ok().map(Pick::Entry)
    }
}

/// Settles an outcome: a pick ends the popup, a closed popup is dropped.
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
/// popup, at the front of `hits`: the pointer over the popup never reaches
/// what is drawn under it.
pub(crate) fn render_popup(
    frame: &mut Frame,
    hovered_row: Option<usize>,
    hits: &mut Vec<(Rect, HitTarget)>,
) {
    let Some(anchor) = ANCHOR.lock().ok().and_then(|a| *a) else {
        return;
    };
    let bounds = frame.area();
    let Ok(mut slot) = POPUP.lock() else {
        return;
    };
    let Some(combo) = slot.as_mut() else {
        return;
    };
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

/// Runs entry `index` on `path`: the status line's answer. Remembers it as the
/// popup's next starting highlight.
pub(crate) fn run_entry(index: usize, path: &Path) -> String {
    LAST_USED.store(index, Ordering::Relaxed);
    crate::tab_dir::run_open_in(index, path).unwrap_or_else(|e| {
        crate::activity_log::report_problem_clear(crate::activity_log::ProblemSource::Tool, &e)
    })
}

/// "Tab: Open in Default": the default entry on `path`.
pub(crate) fn run_default(path: &Path) -> String {
    if crate::tab_dir::entries().is_empty() {
        return msg::NO_ENTRIES.get().to_string();
    }
    run_entry(crate::tab_dir::default_index(), path)
}

/// The tooltip: "Open in <default entry>…", the plain label with no entry.
pub(crate) fn tooltip() -> &'static str {
    let labels = crate::tab_dir::entries();
    let Some(label) = labels.get(crate::tab_dir::default_index()) else {
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
    // One small string per distinct default label over the session.
    let text: &'static str = Box::leak(msg::tooltip(label).into_boxed_str());
    *cached = Some((label.clone(), text));
    text
}

#[cfg(test)]
mod tests;
