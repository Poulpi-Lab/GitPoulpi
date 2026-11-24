//! The Log's author filter: the funnel in the Author column's header, the
//! popup it opens and the pure model behind both.
//!
//! The filter is not a second mechanism. It feeds the same match set the graph
//! search paints (`log_search`, `graph::cache::LogLayoutCache`): the commits of
//! the chosen authors are the hits, every other row dims, and a text search that
//! is open at the same time narrows the hits further (both apply). Nothing is
//! removed from the graph and nothing is fetched: the author list is the
//! authors of the commits the Log has loaded (mailmap-mapped when
//! `log.use_mailmap` is on, the daemon maps them before they arrive), so the
//! list and the dimming always agree about which commits exist.
//!
//! State: the chosen authors' keys live in the window (`log_author_filter`) and
//! in `RepoState::log_author_filter` (written on every change, read when the
//! window opens), so the filter is per repository and survives a tab switch,
//! which rebuilds the window.

mod popup;
mod render;
#[cfg(test)]
mod tests;

pub(crate) use popup::{AuthorFilterPopup, AuthorRow};
pub(crate) use render::render;

use gitpoulpi_core::domain::CommitNode;
use gitpoulpi_session::RepoState;
use ratatui::layout::Rect;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The funnel glyph of the Author header (outline: no filter; filled: active).
/// Neither has emoji presentation, so one cell each.
pub(crate) const FUNNEL: &str = "\u{25BD}";
pub(crate) const FUNNEL_ACTIVE: &str = "\u{25BC}";

/// One author of the loaded history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuthorEntry {
    pub(crate) key: String,
    pub(crate) name: String,
    pub(crate) email: String,
    /// Loaded commits by this author (0: chosen earlier, none loaded now).
    pub(crate) commits: usize,
}

/// What identifies an author: the lower-cased email, else the lower-cased
/// name (a commit with an empty email still belongs to someone).
pub(crate) fn author_key(name: &str, email: &str) -> String {
    let email = email.trim().to_lowercase();
    if email.is_empty() {
        name.trim().to_lowercase()
    } else {
        email
    }
}

/// The authors of `nodes`, most commits first, then by name, then by key (so
/// the order never depends on the walk). `chosen` keys with no loaded commit
/// are appended with 0 commits, so a filter set earlier can always be undone
/// from the list.
pub(crate) fn build_authors(nodes: &[CommitNode], chosen: &BTreeSet<String>) -> Vec<AuthorEntry> {
    let mut by_key: BTreeMap<String, AuthorEntry> = BTreeMap::new();
    for c in nodes {
        let key = author_key(&c.author_name, &c.author_email);
        by_key
            .entry(key.clone())
            .and_modify(|e| e.commits += 1)
            .or_insert_with(|| AuthorEntry {
                key,
                name: c.author_name.clone(),
                email: c.author_email.clone(),
                commits: 1,
            });
    }
    for key in chosen {
        by_key.entry(key.clone()).or_insert_with(|| AuthorEntry {
            key: key.clone(),
            name: key.clone(),
            email: String::new(),
            commits: 0,
        });
    }
    let mut out: Vec<AuthorEntry> = by_key.into_values().collect();
    out.sort_by(|a, b| {
        b.commits
            .cmp(&a.commits)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.key.cmp(&b.key))
    });
    out
}

/// Whether `commit` is by one of the chosen authors.
pub(crate) fn commit_by(commit: &CommitNode, chosen: &BTreeSet<String>) -> bool {
    chosen.contains(&author_key(&commit.author_name, &commit.author_email))
}

/// The display names of the chosen authors, for the tooltip and the Log's
/// note: the name a loaded commit carries, else the stored key.
pub(crate) fn chosen_names(nodes: &[CommitNode], chosen: &BTreeSet<String>) -> Vec<String> {
    let mut names: BTreeMap<String, String> =
        chosen.iter().map(|k| (k.clone(), k.clone())).collect();
    let mut left = chosen.len();
    for c in nodes {
        if left == 0 {
            break;
        }
        let key = author_key(&c.author_name, &c.author_email);
        if let Some(slot) = names.get_mut(&key) {
            if *slot == key {
                *slot = c.author_name.clone();
                left -= 1;
            }
        }
    }
    let mut out: Vec<String> = names.into_values().collect();
    out.sort_by_key(|n| n.to_lowercase());
    out
}

/// The chosen authors as the Log's top-border note: the first name and how
/// many more.
pub(crate) fn note_text(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => gitpoulpi_i18n::msg::author_filter::note_one(one),
        [first, rest @ ..] => gitpoulpi_i18n::msg::author_filter::note_many(first, rest.len()),
    }
}

/// Where the funnel sits in a header cell `width` cells wide whose title is
/// `title_w` cells: `(title cells, funnel x offset)`. The funnel follows the
/// title (a space between, one cell of gutter after); the title is never cut
/// for it, so a column too narrow for both draws no funnel (the keyboard and
/// the palette still reach the filter, and the Log's top border says it is on).
pub(crate) fn funnel_layout(width: u16, title_w: u16) -> Option<(u16, u16)> {
    (width >= title_w + 3).then_some((title_w, title_w + 1))
}

/// The two cells (a space and the glyph) the funnel's hit target covers.
pub(crate) fn funnel_rect(column_x: u16, width: u16, title_w: u16, y: u16) -> Option<Rect> {
    funnel_layout(width, title_w).map(|(_, dx)| Rect::new(column_x + dx - 1, y, 2, 1))
}

// The Author header's place and the chosen authors' names, set by the paint
// walk and read by the popup (anchor) and the tooltip.
thread_local! {
    static ANCHOR: std::cell::RefCell<Option<Rect>> = const { std::cell::RefCell::new(None) };
    static TIP: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// The Author column's header cell as painted (`None`: the column is not on
/// screen). The popup opens under it.
pub(crate) fn set_anchor(rect: Option<Rect>) {
    ANCHOR.with(|a| *a.borrow_mut() = rect);
}

pub(crate) fn anchor() -> Option<Rect> {
    ANCHOR.with(|a| *a.borrow())
}

/// The chosen authors' names the header tooltip lists (empty: no filter).
pub(crate) fn set_tip(names: Vec<String>) {
    TIP.with(|t| *t.borrow_mut() = names);
}

pub(crate) fn tip_lines() -> Vec<String> {
    use gitpoulpi_i18n::msg::author_filter as m;
    let names = TIP.with(|t| t.borrow().clone());
    if names.is_empty() {
        return vec![m::TIP_OFF.get().to_string()];
    }
    let mut out = vec![m::tip_on(names.len())];
    out.extend(names.into_iter().map(|n| format!("  {n}")));
    out
}

/// The filter saved for this repository.
pub(crate) fn load(repo_root: &Path) -> BTreeSet<String> {
    RepoState::load(&crate::fold_state::state_path(repo_root))
        .unwrap_or_default()
        .log_author_filter
        .into_iter()
        .collect()
}

/// Load-modify-save: the file also holds the folds and the ref filters.
pub(crate) fn save(repo_root: &Path, chosen: &BTreeSet<String>) {
    let path = crate::fold_state::state_path(repo_root);
    let mut state = RepoState::load(&path).unwrap_or_default();
    state.log_author_filter = chosen.iter().cloned().collect();
    let _ = state.save(&path);
}

/// Toggles `key` in `chosen` and saves; the status line for it.
pub(crate) fn toggle(repo_root: &Path, chosen: &mut BTreeSet<String>, key: &str) -> String {
    if !chosen.remove(key) {
        chosen.insert(key.to_string());
    }
    save(repo_root, chosen);
    status_text(chosen.len())
}

/// Drops the whole filter and saves; the status line for it.
pub(crate) fn clear(repo_root: &Path, chosen: &mut BTreeSet<String>) -> String {
    chosen.clear();
    save(repo_root, chosen);
    gitpoulpi_i18n::msg::author_filter::STATUS_CLEARED
        .get()
        .to_string()
}

/// What Enter (or a click) on the highlighted row does: the Clear row drops
/// the filter and closes the popup, an author row toggles that author and
/// leaves it open (several can be chosen). Returns the status line and whether
/// the popup closes.
pub(crate) fn activate(
    popup: &mut AuthorFilterPopup,
    repo_root: &Path,
    chosen: &mut BTreeSet<String>,
) -> (Option<String>, bool) {
    match popup.highlighted() {
        Some(AuthorRow::Clear) => {
            let message = clear(repo_root, chosen);
            popup.chosen.clear();
            (Some(message), true)
        }
        Some(AuthorRow::Author(i)) => {
            let Some(key) = popup.entry(i).map(|e| e.key.clone()) else {
                return (None, false);
            };
            let message = toggle(repo_root, chosen, &key);
            popup.chosen = chosen.clone();
            (Some(message), false)
        }
        None => (None, false),
    }
}

/// Opens the popup over the Log's shown commits.
pub(crate) fn open_popup(
    nodes: &[CommitNode],
    chosen: &BTreeSet<String>,
    placement: gitpoulpi_config::AvatarPlacement,
) -> AuthorFilterPopup {
    AuthorFilterPopup::open(nodes, chosen, crate::avatar::enabled(placement))
}

fn status_text(count: usize) -> String {
    if count == 0 {
        gitpoulpi_i18n::msg::author_filter::STATUS_CLEARED
            .get()
            .to_string()
    } else {
        gitpoulpi_i18n::msg::author_filter::status_on(count)
    }
}
