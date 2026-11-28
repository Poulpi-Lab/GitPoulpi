//! What the window knows about the repository's workflow: the daemon's
//! `Workflow` report, kept process-wide so the sidebar's row builder
//! (`refs_tree::ref_display_rows`, called from a dozen places that must agree
//! on the rows) can group branches without every caller being handed it.
//!
//! The model (`FlowView`, the grouped rows, the tooltips) lives in
//! `gitpoulpi_app::flow::view`; this module keeps the store and the hovered
//! branch, and calls the model with the stored view.
//!
//! One store per process: a window installs its report when it arrives and
//! clears it when it opens (a repository switch restarts the window). In unit
//! tests the store is **thread-local**, like the process-wide settings in
//! `live_settings`, so tests running in parallel never see each other's.

use std::sync::Arc;

#[cfg(test)]
pub(crate) use gitpoulpi_app::flow::view::GROUP_PREFIX;
pub(crate) use gitpoulpi_app::flow::view::{is_group_path, FlowView};

use crate::refs_tree::{CollapsedFolders, RefRowKind, RefTreeRow};

// ---------------------------------------------------------------------
// The store
// ---------------------------------------------------------------------

#[cfg(not(test))]
mod store {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::RwLock;

    static VIEW: RwLock<Option<Arc<FlowView>>> = RwLock::new(None);
    static PLAIN: AtomicBool = AtomicBool::new(false);
    static VERSION: AtomicU64 = AtomicU64::new(0);

    pub(super) fn current() -> Option<Arc<FlowView>> {
        VIEW.read().ok().and_then(|v| v.clone())
    }
    pub(super) fn install(view: Option<Arc<FlowView>>) {
        if let Ok(mut slot) = VIEW.write() {
            *slot = view;
        }
        VERSION.fetch_add(1, Ordering::SeqCst);
    }
    pub(super) fn version() -> u64 {
        VERSION.load(Ordering::SeqCst)
    }
    pub(super) fn plain() -> bool {
        PLAIN.load(Ordering::SeqCst)
    }
    pub(super) fn set_plain(on: bool) {
        PLAIN.store(on, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod store {
    use super::*;
    use std::cell::{Cell, RefCell};

    thread_local! {
        static VIEW: RefCell<Option<Arc<FlowView>>> = const { RefCell::new(None) };
        static PLAIN: Cell<bool> = const { Cell::new(false) };
        static VERSION: Cell<u64> = const { Cell::new(0) };
    }

    pub(super) fn current() -> Option<Arc<FlowView>> {
        VIEW.with(|v| v.borrow().clone())
    }
    pub(super) fn install(view: Option<Arc<FlowView>>) {
        VIEW.with(|v| *v.borrow_mut() = view);
        VERSION.with(|v| v.set(v.get() + 1));
    }
    pub(super) fn version() -> u64 {
        VERSION.with(Cell::get)
    }
    pub(super) fn plain() -> bool {
        PLAIN.with(Cell::get)
    }
    pub(super) fn set_plain(on: bool) {
        PLAIN.with(|p| p.set(on));
    }
}

/// The window's current report, if one arrived.
pub(crate) fn current() -> Option<Arc<FlowView>> {
    store::current()
}

/// The report when a workflow is configured (the toolbar button, the
/// palette commands and the grouping all need exactly that).
pub(crate) fn configured() -> Option<Arc<FlowView>> {
    current().filter(|v| v.configured())
}

pub(crate) fn install(view: Option<Arc<FlowView>>) {
    store::install(view);
}

/// Changes whenever a report is installed or cleared (the Settings page
/// refreshes its copy when it moves).
pub(crate) fn version() -> u64 {
    store::version()
}

/// The sidebar shows the plain LOCAL / REMOTE trees instead of the groups.
pub(crate) fn plain() -> bool {
    store::plain()
}

pub(crate) fn set_plain(on: bool) {
    store::set_plain(on);
}

// ---------------------------------------------------------------------
// The grouped sidebar rows (the model is in the app crate)
// ---------------------------------------------------------------------

/// The rows of LOCAL / REMOTE grouped by the workflow, or `None` when the
/// plain tree applies (no workflow, the toggle is off, tags).
#[allow(dead_code)]
pub(crate) fn grouped_rows(
    names: &[String],
    kind: RefRowKind,
    collapsed: &CollapsedFolders,
) -> Option<Vec<RefTreeRow>> {
    if plain() {
        return None;
    }
    gitpoulpi_app::flow::view::grouped_rows(&*configured()?, names, kind, collapsed)
}

/// The branches of `names` that sit under the group folder `path` (what
/// "hide / solo this folder" acts on).
pub(crate) fn group_members(names: &[String], path: &str) -> Vec<String> {
    match configured() {
        Some(view) => gitpoulpi_app::flow::view::group_members(&view, names, path),
        None => Vec::new(),
    }
}

/// The tooltip lines of a sidebar branch, `None` when no workflow is
/// configured or the branch is unknown to the report.
pub(crate) fn branch_tooltip(full_name: &str) -> Option<Vec<String>> {
    gitpoulpi_app::flow::view::branch_tooltip(&*configured()?, full_name)
}

// ---------------------------------------------------------------------
// The hovered sidebar branch (its tooltip)
// ---------------------------------------------------------------------

#[cfg(not(test))]
static HOVER: std::sync::Mutex<Option<(String, u16, u16)>> = std::sync::Mutex::new(None);
#[cfg(test)]
thread_local! {
    static HOVER: std::cell::RefCell<Option<(String, u16, u16)>> =
        const { std::cell::RefCell::new(None) };
}

/// The pointer is over sidebar branch `name` at `(col, row)`, or over none.
pub(crate) fn set_hover(hover: Option<(String, u16, u16)>) {
    #[cfg(not(test))]
    if let Ok(mut slot) = HOVER.lock() {
        *slot = hover;
    }
    #[cfg(test)]
    HOVER.with(|h| *h.borrow_mut() = hover);
}

fn hover() -> Option<(String, u16, u16)> {
    #[cfg(not(test))]
    {
        HOVER.lock().ok().and_then(|h| h.clone())
    }
    #[cfg(test)]
    {
        HOVER.with(|h| h.borrow().clone())
    }
}

/// The tooltip key of the hovered branch (a change of key is a change of
/// hover target), `None` when it has no tooltip.
pub(crate) fn hover_key() -> Option<u64> {
    use std::hash::{Hash, Hasher};
    let (name, _, _) = hover()?;
    branch_tooltip(&name)?;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut h);
    Some(h.finish())
}

/// `(key, lines, col, row)` of the hovered branch's tooltip.
pub(crate) fn hover_tooltip() -> Option<(u64, Vec<String>, u16, u16)> {
    let key = hover_key()?;
    let (name, col, row) = hover()?;
    Some((key, branch_tooltip(&name)?, col, row))
}
