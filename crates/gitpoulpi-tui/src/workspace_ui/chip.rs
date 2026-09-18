//! The tab bar's workspace chip and its dropdown.
//!
//! The chip sits right after the `⌂`: a colored dot and the active
//! workspace's name, or "No workspace" dimmed. Clicking it (or `Alt+W`, or the
//! palette's `Workspace: Switch...`) opens a combo listing the workspaces,
//! "No workspace" and "Manage workspaces...".
//!
//! Like `profile_bar` this is process-wide state (the window restarts on
//! every tab switch, and a switch of workspace is one) and the popup is modal
//! while open: it owns the keyboard and the mouse and its rows lead the
//! frame's hit list.
//!
//! **The chip appears once a workspace exists** (or one is active). With none
//! it would be a permanent "No workspace" that can only lead to the Home
//! page; the Home tab and the palette are the way in then.

use std::sync::Mutex;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::Frame;

use gitpoulpi_i18n::msg::workspaces as m;

use crate::chrome::HitTarget;
use crate::combo::{Combo, ComboOption, ComboOutcome};

/// What a pick in the combo asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Pick {
    /// Move the window to this workspace (`None` = no workspace).
    Switch(Option<String>),
    /// "Manage workspaces...": the Home tab's Workspaces page.
    Manage,
}

const NONE_VALUE: &str = "";
const MANAGE_VALUE: &str = "\u{1}manage";

#[derive(Clone)]
struct Entry {
    id: String,
    name: String,
    color: Option<String>,
}

struct State {
    entries: Vec<Entry>,
    active: Option<String>,
    popup: Option<Combo>,
    anchor: Option<Rect>,
}

static STATE: Mutex<State> = Mutex::new(State {
    entries: Vec::new(),
    active: None,
    popup: None,
    anchor: None,
});

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut guard)
}

/// Re-reads the workspaces and the session's active one (window start, and
/// after anything that changed either).
pub(crate) fn refresh(session: &str) {
    let all = super::store::load();
    let active = super::store::active(session).map(|w| w.id);
    install(
        all.workspaces
            .iter()
            .map(|w| (w.id.clone(), w.name.clone(), w.color.clone())),
        active,
    );
}

fn install(
    entries: impl Iterator<Item = (String, String, Option<String>)>,
    active: Option<String>,
) {
    with(|s| {
        s.entries = entries
            .map(|(id, name, color)| Entry { id, name, color })
            .collect();
        s.active = active;
    });
}

/// The active workspace's id.
pub(crate) fn active_id() -> Option<String> {
    with(|s| s.active.clone())
}

/// Worth drawing: a workspace exists or one is active.
pub(crate) fn visible() -> bool {
    with(|s| !s.entries.is_empty() || s.active.is_some())
}

/// What the tab bar draws.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Look {
    pub(crate) dot: Color,
    /// The workspace's name, or "No workspace".
    pub(crate) label: String,
    /// No workspace is active: the label is dimmed.
    pub(crate) none: bool,
}

pub(crate) fn look() -> Option<Look> {
    if !visible() {
        return None;
    }
    Some(with(|s| {
        match s.entries.iter().find(|e| Some(&e.id) == s.active.as_ref()) {
            Some(e) => Look {
                dot: super::color_of(e.color.as_deref()),
                label: e.name.clone(),
                none: false,
            },
            None => Look {
                dot: crate::th().ui.muted,
                label: m::CHIP_NONE.get().to_string(),
                none: true,
            },
        }
    }))
}

/// The chip's rect, remembered as the popup's anchor.
pub(crate) fn set_anchor(rect: Option<Rect>) {
    with(|s| s.anchor = rect);
}

pub(crate) fn is_open() -> bool {
    with(|s| s.popup.as_ref().is_some_and(Combo::is_open))
}

/// Opens the combo on the active workspace. Works with no chip drawn (a
/// narrow bar, the palette): it then hangs from the tab bar's left end.
pub(crate) fn open() {
    with(|s| {
        let mut options = vec![ComboOption::new(m::COMBO_NONE.get(), NONE_VALUE)];
        for e in &s.entries {
            options.push(
                ComboOption::new(e.name.clone(), e.id.clone())
                    .with_swatch(vec![super::color_of(e.color.as_deref())]),
            );
        }
        options.push(ComboOption::new(m::COMBO_MANAGE.get(), MANAGE_VALUE));
        let mut combo = Combo::new("");
        combo.set_options(options);
        combo.select(Some(s.active.clone().unwrap_or_default()));
        combo.open();
        s.popup = Some(combo);
    });
}

fn pick(outcome: ComboOutcome, slot: &mut Option<Combo>) -> Option<Pick> {
    let value = match outcome {
        ComboOutcome::Picked(v) | ComboOutcome::Custom(v) => Some(v),
        _ => None,
    };
    if slot.as_ref().is_some_and(|c| !c.is_open()) {
        *slot = None;
    }
    value.map(|v| match v.as_str() {
        MANAGE_VALUE => Pick::Manage,
        NONE_VALUE => Pick::Switch(None),
        id => Pick::Switch(Some(id.to_string())),
    })
}

pub(crate) fn on_key(key: KeyEvent) -> Option<Pick> {
    with(|s| {
        let outcome = s.popup.as_mut()?.on_key(key);
        pick(outcome, &mut s.popup)
    })
}

pub(crate) fn on_mouse(mouse: MouseEvent) -> Option<Pick> {
    with(|s| {
        let outcome = s.popup.as_mut()?.on_mouse(mouse);
        pick(outcome, &mut s.popup)
    })
}

pub(crate) fn paste(text: &str) {
    with(|s| {
        if let Some(combo) = s.popup.as_mut() {
            combo.paste(text);
        }
    });
}

/// Paints the open popup and puts its rows, then the whole popup, at the
/// front of `hits`.
pub(crate) fn render_popup(
    frame: &mut Frame,
    hovered_row: Option<usize>,
    hits: &mut Vec<(Rect, HitTarget)>,
) {
    let bounds = frame.area();
    with(|s| {
        let anchor = s
            .anchor
            .unwrap_or_else(|| Rect::new(bounds.x, bounds.y + 1, 1, 1));
        let Some(combo) = s.popup.as_mut() else {
            return;
        };
        combo.set_anchor(anchor);
        combo.set_hover(hovered_row);
        combo.render_popup(frame.buffer_mut(), bounds);
        let mut front: Vec<(Rect, HitTarget)> = combo
            .item_rects()
            .into_iter()
            .map(|(rect, row)| (rect, HitTarget::WorkspaceItem(row)))
            .collect();
        let popup = combo.popup_rect();
        if popup.width > 0 {
            front.push((popup, HitTarget::WorkspacePopup));
        }
        hits.splice(0..0, front);
    });
}

#[cfg(test)]
pub(crate) fn install_for_test(entries: &[(&str, &str, Option<&str>)], active: Option<&str>) {
    install(
        entries
            .iter()
            .map(|(i, n, c)| (i.to_string(), n.to_string(), c.map(str::to_string))),
        active.map(str::to_string),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    /// The state is process-wide: tests that set it take turns.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn the_chip_is_hidden_until_a_workspace_exists_then_names_the_active_one() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        install_for_test(&[], None);
        assert!(look().is_none());
        install_for_test(&[("acme", "Acme", Some("green"))], None);
        let none = look().unwrap();
        assert!(none.none && none.label == "No workspace");
        install_for_test(&[("acme", "Acme", Some("green"))], Some("acme"));
        let on = look().unwrap();
        assert!(!on.none);
        assert_eq!(on.label, "Acme");
        install_for_test(&[], None);
    }

    #[test]
    fn the_combo_lists_none_the_workspaces_and_manage_and_returns_what_is_picked() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        install_for_test(
            &[("acme", "Acme", None), ("home", "Home lab", Some("blue"))],
            Some("acme"),
        );
        open();
        assert!(is_open());
        // Opens on the active workspace; Down moves to "Home lab".
        assert_eq!(on_key(key(KeyCode::Down)), None);
        assert_eq!(
            on_key(key(KeyCode::Enter)),
            Some(Pick::Switch(Some("home".into())))
        );
        assert!(!is_open());

        open();
        // Up twice from "Acme": "No workspace".
        on_key(key(KeyCode::Up));
        assert_eq!(on_key(key(KeyCode::Enter)), Some(Pick::Switch(None)));

        open();
        for _ in 0..3 {
            on_key(key(KeyCode::Down));
        }
        assert_eq!(on_key(key(KeyCode::Enter)), Some(Pick::Manage));
        install_for_test(&[], None);
    }
}
