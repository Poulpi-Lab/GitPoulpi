//! The Repository Management tab: the leftmost tab of
//! the bar (the `⌂`), a full view of its own rather than a popup.
//!
//! **Why a tab and not a popup.** This view is the window that has *no*
//! repository behind it (`tabs::no_repository_ref`), so there is nothing
//! underneath to steal the wheel or the clicks, by construction rather
//! than by a guard in every mouse handler.
//!
//! Like `repo_switcher`, everything here is **pure**: state, a key
//! decision function, a click decision function, and one render walk
//! that produces the lines *and* the hit rects together (the
//! `log_table` rule -- a second walk computing the rects drifts from
//! the one that drew the glyphs). `event_loop.rs` routes input here and
//! acts on the [`ManagerAction`] that comes back.

mod input;
mod render;
mod workspaces;

use gitpoulpi_i18n::msg;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::repo_switcher::RepoFacts;
use crate::{button_bg, display_width, fit, text_field};

/// One repository the tab knows about, from any of its sources: an open
/// tab, a favorite, a recent entry. The four sections are views over one
/// list of these, joined by path, so starring a repository in one section
/// stars it in all four.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownRepo {
    pub path: PathBuf,
    pub name: String,
    /// The branch the recent list remembered. Shown, dimmed, until the
    /// background probe says what is checked out now.
    pub branch_hint: Option<String>,
    pub exists: bool,
    pub is_repo: bool,
    /// Its index in the tab bar, when it is open.
    pub tab: Option<usize>,
    /// Its position in the favorites list, when it is starred.
    pub favorite: Option<usize>,
    /// Its position in the recent list (newest first), when it is in it.
    pub recent: Option<usize>,
    /// `None` until the probe answers - *still looking*.
    pub facts: Option<RepoFacts>,
}

impl KnownRepo {
    pub fn available(&self) -> bool {
        self.exists && self.is_repo
    }

    /// The owner/namespace column: the default remote's owner once the
    /// probe has read it, else the parent folder's name - which is what
    /// tells two clones called `app` apart when neither has a remote.
    pub fn owner_label(&self) -> String {
        if let Some(owner) = self.facts.as_ref().and_then(|f| f.owner.clone()) {
            return owner;
        }
        self.path
            .parent()
            .and_then(Path::file_name)
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// What the owner column shows. When another known repository has the
    /// same name, the parent folder is appended so the two can be told
    /// apart (`poulpi-lab  work/tools`).
    fn column_text(&self, ambiguous: bool) -> String {
        let owner = self.owner_label();
        if !ambiguous {
            return owner;
        }
        let parent = self
            .path
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let parent = parent.trim_end_matches(['/', '\\']);
        let tail: Vec<&str> = parent.rsplit(['/', '\\']).take(2).collect();
        let tail = tail.into_iter().rev().collect::<Vec<_>>().join("/");
        if tail.is_empty() || tail == owner {
            owner
        } else {
            format!("{owner} \u{b7} {tail}")
        }
    }

    /// The one filter of the Home list: the name, the path or the branch
    /// (the probe's answer, else the one the recent list remembered).
    fn matches(&self, filter: &str) -> bool {
        filter.is_empty()
            || self.name.to_lowercase().contains(filter)
            || self.path.to_string_lossy().to_lowercase().contains(filter)
            || self
                .facts
                .as_ref()
                .and_then(|f| f.branch.as_deref())
                .or(self.branch_hint.as_deref())
                .is_some_and(|b| b.to_lowercase().contains(filter))
    }
}

/// Lower-cased names carried by more than one known repository.
fn duplicate_names(repos: &[KnownRepo]) -> std::collections::HashSet<String> {
    let mut seen = std::collections::HashSet::new();
    let mut dup = std::collections::HashSet::new();
    for r in repos {
        let name = r.name.to_lowercase();
        if !seen.insert(name.clone()) {
            dup.insert(name);
        }
    }
    dup
}

/// The four collapsible sections, in the order they are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Section {
    Open,
    Favorites,
    Recent,
    All,
}

impl Section {
    pub const ORDER: [Section; 4] = [
        Section::Open,
        Section::Favorites,
        Section::Recent,
        Section::All,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Section::Open => msg::home::SECTION_OPEN.get(),
            Section::Favorites => msg::home::SECTION_FAVORITES.get(),
            Section::Recent => msg::home::SECTION_RECENT.get(),
            Section::All => msg::home::SECTION_ALL.get(),
        }
    }

    /// The section's own button, right-aligned on its header row.
    pub fn action_label(self) -> Option<&'static str> {
        match self {
            Section::Open => Some(msg::home::CLOSE_ALL_TABS.get()),
            Section::Recent => Some(msg::home::REMOVE_ALL.get()),
            Section::Favorites | Section::All => None,
        }
    }

    fn empty_text(self) -> &'static str {
        match self {
            Section::Open => msg::home::EMPTY_OPEN.get(),
            Section::Favorites => msg::home::EMPTY_FAVORITES.get(),
            Section::Recent => msg::home::EMPTY_RECENT.get(),
            Section::All => msg::home::EMPTY_ALL.get(),
        }
    }

    fn contains(self, repo: &KnownRepo) -> bool {
        match self {
            Section::Open => repo.tab.is_some(),
            Section::Favorites => repo.favorite.is_some(),
            Section::Recent => repo.recent.is_some(),
            Section::All => true,
        }
    }
}

/// One row of the list, in display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagerRow {
    Header(Section),
    /// `repo` indexes [`RepoManager::repos`].
    Repo {
        section: Section,
        repo: usize,
    },
    /// A section with nothing (matching) in it says so, rather than
    /// showing a header with nothing under it.
    Empty(Section),
}

/// The five buttons above the list. The `Integrations` button
/// beside Init is deliberately not here yet - room is left for it
/// at the end of the left group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagerButton {
    Browse,
    Clone,
    Init,
    CollapseAll,
    ExpandAll,
}

impl ManagerButton {
    fn label(self) -> &'static str {
        match self {
            ManagerButton::Browse => msg::home::BTN_BROWSE.get(),
            ManagerButton::Clone => msg::home::BTN_CLONE.get(),
            ManagerButton::Init => msg::home::BTN_INIT.get(),
            ManagerButton::CollapseAll => msg::home::COLLAPSE_ALL.get(),
            ManagerButton::ExpandAll => msg::home::EXPAND_ALL.get(),
        }
    }

    /// The key that does the same thing, shown in the button itself so
    /// the keyboard path is discoverable from the mouse one.
    fn key(self) -> &'static str {
        match self {
            ManagerButton::Browse => "b",
            ManagerButton::Clone => "c",
            ManagerButton::Init => "i",
            ManagerButton::CollapseAll => "-",
            ManagerButton::ExpandAll => "+",
        }
    }
}

/// Where a click landed. Row indices index [`RepoManager::rows`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagerHit {
    Button(ManagerButton),
    Search,
    Header(Section),
    SectionAction(Section),
    /// A control of the Workspaces page.
    Ws(WsHit),
    Row(usize),
    Star(usize),
    RowOpen(usize),
    RowClose(usize),
    RowMenu(usize),
}

/// What `event_loop.rs` should do about the last key or click.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagerAction {
    /// Handled inside the view (a keystroke into the search, a moved
    /// selection, a collapsed section).
    Handled,
    /// Not ours - the palette's `Ctrl+P`, Preferences' `Ctrl+,`, help.
    NotHandled,
    /// Open the repository at this path as a tab (it is not one yet).
    Open(PathBuf),
    /// Switch to the tab at this index.
    FocusTab(usize),
    /// Close the tab at this index (asks first, like every close).
    CloseTab(usize),
    CloseAllTabs,
    ToggleFavorite(PathBuf),
    RemoveRecent(PathBuf),
    /// Forget the path in every list that stores it (recent and
    /// favorites). Never touches the folder or an open tab.
    RemoveFromList(PathBuf),
    RemoveAllRecent,
    Browse,
    Clone,
    Init,
    /// The Workspaces page asks for something (`workspace_ui` does it).
    Workspace(WsAction),
    /// Open the `⋮` menu for this repository (index into `repos`). The
    /// caller anchors it: at the click, or under the selected row.
    Menu {
        repo: usize,
    },
    /// Back to the active repository tab (`Esc` with an empty search).
    Leave,
    Quit,
}

/// The whole view's state.
#[derive(Debug, Clone)]
pub struct RepoManager {
    pub repos: Vec<KnownRepo>,
    pub query: String,
    pub search_focused: bool,
    /// This view is a transient "New tab" (the `+` on the tab bar) rather
    /// than the Home tab: the `⌂` is not the selected tab, a "New tab" tab
    /// is, and `Esc` / `Ctrl+W` / its `x` return to the previous tab.
    pub new_tab: bool,
    /// Which page of the Home tab is showing.
    pub page: HomePage,
    /// The Workspaces page's state.
    pub ws: WorkspacesPage,
    pub collapsed: HashSet<Section>,
    /// Index into [`RepoManager::rows`].
    pub selected: usize,
    /// First list row on screen.
    pub scroll: usize,
    /// Whether the next draw should scroll to keep the selection in
    /// view. The keyboard sets it; the wheel clears it, so scrolling to
    /// look at something does not snap back to the selection.
    follow: bool,
    /// A line at the bottom: what just happened, or why it did not.
    pub message: Option<String>,
    pub hover: Option<ManagerHit>,
    /// The number of list rows the last draw had room for, for paging.
    list_height: usize,
    /// A status-bar line for a pending key prefix (`Ctrl+N`), set by the
    /// event loop each frame: the Home tab covers the window's own
    /// status bar, so the prefix's hint is shown in this view's footer.
    pub mode_hint: Option<String>,
    /// The search box's caret and selection - the shared editing core
    /// every text field types through.
    search_edit: crate::text_edit::FieldEditor<()>,
    /// The Radar, when it is showing over the repository list.
    pub(crate) radar: crate::radar::RadarSlot,
    /// The Release notes tab, when it is showing over the repository list.
    pub(crate) notes: crate::updates::notes::ReleaseNotesSlot,
}

impl RepoManager {
    pub fn new(repos: Vec<KnownRepo>) -> Self {
        let mut manager = Self {
            repos,
            query: String::new(),
            search_focused: false,
            new_tab: false,
            page: HomePage::Repositories,
            ws: WorkspacesPage::default(),
            collapsed: HashSet::new(),
            selected: 0,
            scroll: 0,
            follow: true,
            message: None,
            hover: None,
            list_height: 10,
            mode_hint: None,
            search_edit: Default::default(),
            radar: Default::default(),
            notes: Default::default(),
        };
        manager.select_first_repo();
        manager
    }

    /// Keeps the search box's caret with its focus: leaving the box
    /// forgets the caret, so coming back starts at the end again.
    pub(crate) fn sync_search(&mut self) {
        self.search_edit.sync(self.search_focused.then_some(()));
    }

    /// An editing key for the focused search box:
    /// `None` when the box isn't focused or the key isn't an editing key,
    /// which then goes on to [`handle_key`].
    pub(crate) fn edit_search(&mut self, key: &KeyEvent) -> Option<crate::text_edit::KeyEdit> {
        self.sync_search();
        if !self.search_focused || !self.search_edit.claims((), &self.query, key, false) {
            return None;
        }
        let edit = self.search_edit.on_key((), &mut self.query, key, false);
        if edit == crate::text_edit::KeyEdit::Edited {
            self.select_first_repo();
        }
        Some(edit)
    }

    /// The mouse half of the search box: a press (after [`click`] gave
    /// it the keyboard), a drag, a release.
    pub(crate) fn search_mouse(
        &mut self,
        kind: crossterm::event::MouseEventKind,
        col: u16,
        row: u16,
    ) {
        use crossterm::event::{MouseButton, MouseEventKind};
        self.sync_search();
        match kind {
            MouseEventKind::Down(MouseButton::Left) if self.search_focused => {
                self.search_edit
                    .mouse_down((), &self.query, col, row, std::time::Instant::now());
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                self.search_edit.mouse_drag(&self.query, col, row);
            }
            MouseEventKind::Up(MouseButton::Left) => self.search_edit.mouse_up(),
            _ => {}
        }
    }

    /// Whether a press in the search box is being dragged.
    pub(crate) fn search_dragging(&self) -> bool {
        self.search_edit.is_dragging()
    }

    /// Replaces the known repositories (after a favorite toggled, a tab
    /// closed, a recent entry removed), keeping the probe's answers and
    /// the selection on the same repository when it is still listed.
    pub fn set_repos(&mut self, mut repos: Vec<KnownRepo>) {
        let selected_path = self.selected_repo().map(|r| r.path.clone());
        for repo in &mut repos {
            if repo.facts.is_none() {
                repo.facts = self
                    .repos
                    .iter()
                    .find(|old| old.path == repo.path)
                    .and_then(|old| old.facts.clone());
            }
        }
        self.repos = repos;
        if let Some(path) = selected_path {
            if let Some(index) = self.rows().iter().position(
                |r| matches!(r, ManagerRow::Repo { repo, .. } if self.repos[*repo].path == path),
            ) {
                self.selected = index;
            }
        }
        self.clamp();
    }

    /// Records one probe answer.
    pub fn learn(&mut self, path: &Path, facts: RepoFacts) {
        for repo in self.repos.iter_mut().filter(|r| r.path == path) {
            repo.facts = Some(facts.clone());
        }
    }

    fn filter(&self) -> String {
        self.query.trim().to_lowercase()
    }

    /// The repositories one section shows, filtered, in that section's
    /// order: tab order, starring order, most recent first, and by name
    /// for *All*.
    pub fn section_repos(&self, section: Section) -> Vec<usize> {
        let filter = self.filter();
        let mut out: Vec<usize> = (0..self.repos.len())
            .filter(|&i| section.contains(&self.repos[i]) && self.repos[i].matches(&filter))
            .collect();
        match section {
            Section::Open => out.sort_by_key(|&i| self.repos[i].tab),
            Section::Favorites => out.sort_by_key(|&i| self.repos[i].favorite),
            Section::Recent => out.sort_by_key(|&i| self.repos[i].recent),
            Section::All => out.sort_by(|&a, &b| {
                self.repos[a]
                    .name
                    .to_lowercase()
                    .cmp(&self.repos[b].name.to_lowercase())
                    .then_with(|| self.repos[a].path.cmp(&self.repos[b].path))
            }),
        }
        out
    }

    /// Every row, in display order. The one function the render walk,
    /// the keyboard and the mouse all read, so none of them can count a
    /// collapsed section differently.
    pub fn rows(&self) -> Vec<ManagerRow> {
        let mut rows = Vec::new();
        for section in Section::ORDER {
            rows.push(ManagerRow::Header(section));
            if self.collapsed.contains(&section) {
                continue;
            }
            let repos = self.section_repos(section);
            if repos.is_empty() {
                rows.push(ManagerRow::Empty(section));
            }
            rows.extend(
                repos
                    .into_iter()
                    .map(|repo| ManagerRow::Repo { section, repo }),
            );
        }
        rows
    }

    pub fn selected_row(&self) -> Option<ManagerRow> {
        self.rows().get(self.selected).copied()
    }

    pub fn selected_repo(&self) -> Option<&KnownRepo> {
        match self.selected_row()? {
            ManagerRow::Repo { repo, .. } => self.repos.get(repo),
            _ => None,
        }
    }

    fn clamp(&mut self) {
        let len = self.rows().len();
        self.selected = self.selected.min(len.saturating_sub(1));
        self.follow = true;
    }

    /// Lands on the first repository row, falling back to the first row.
    /// What the view opens on, and where a new search puts the cursor,
    /// so `Enter` right after typing opens the best match.
    fn select_first_repo(&mut self) {
        self.selected = self
            .rows()
            .iter()
            .position(|r| matches!(r, ManagerRow::Repo { .. }))
            .unwrap_or(0);
        self.follow = true;
    }

    fn move_selection(&mut self, delta: isize) {
        let len = self.rows().len();
        if len == 0 {
            return;
        }
        let next = (self.selected as isize + delta).clamp(0, len as isize - 1);
        self.selected = next as usize;
        self.follow = true;
    }

    pub fn set_collapsed_all(&mut self, collapsed: bool) {
        if self.page == HomePage::Workspaces {
            self.ws.set_all(!collapsed);
            return;
        }
        if collapsed {
            self.collapsed = Section::ORDER.into_iter().collect();
        } else {
            self.collapsed.clear();
        }
        // The selection follows its section's header, so collapsing
        // everything does not leave it on a row that is no longer there.
        if collapsed {
            let header = self.section_of(self.selected);
            self.selected = self
                .rows()
                .iter()
                .position(|r| *r == ManagerRow::Header(header))
                .unwrap_or(0);
        }
        self.clamp();
    }

    fn toggle_section(&mut self, section: Section) {
        if !self.collapsed.remove(&section) {
            self.collapsed.insert(section);
        }
        self.selected = self
            .rows()
            .iter()
            .position(|r| *r == ManagerRow::Header(section))
            .unwrap_or(0);
        self.clamp();
    }

    /// The section the row at `index` belongs to.
    fn section_of(&self, index: usize) -> Section {
        match self.rows().get(index) {
            Some(ManagerRow::Header(s)) | Some(ManagerRow::Empty(s)) => *s,
            Some(ManagerRow::Repo { section, .. }) => *section,
            None => Section::Open,
        }
    }

    /// What "open" means for a repository: focus its tab when it has
    /// one, open it otherwise, explain when it cannot be opened.
    fn open_repo(&mut self, repo: usize) -> ManagerAction {
        let Some(known) = self.repos.get(repo) else {
            return ManagerAction::Handled;
        };
        if let Some(tab) = known.tab {
            return ManagerAction::FocusTab(tab);
        }
        if !known.available() {
            let path = known.path.display();
            let key = crate::keymap::current().shown(crate::keymap::Action::HomeCloseRepo);
            self.message = Some(if known.exists {
                msg::home::not_a_repo_any_more(path, key)
            } else {
                msg::home::gone_from_disk_msg(path, key)
            });
            return ManagerAction::Handled;
        }
        ManagerAction::Open(known.path.clone())
    }

    /// The row's close/remove button, and `x`/`Delete` on it: close the
    /// tab in *Open*, drop the entry in *Recent*, un-star it in
    /// *Favorites*. *All* has nothing to remove a repository from.
    fn close_repo(&mut self, section: Section, repo: usize) -> ManagerAction {
        let Some(known) = self.repos.get(repo) else {
            return ManagerAction::Handled;
        };
        match section {
            Section::Open => known
                .tab
                .map(ManagerAction::CloseTab)
                .unwrap_or(ManagerAction::Handled),
            Section::Recent => ManagerAction::RemoveRecent(known.path.clone()),
            Section::Favorites => ManagerAction::ToggleFavorite(known.path.clone()),
            Section::All => ManagerAction::RemoveFromList(known.path.clone()),
        }
    }

    /// `HomeRemoveFromList` / the menu's *Remove from list*: forget the
    /// selected repository in every stored list, whatever section the
    /// cursor is in.
    pub(crate) fn remove_selected_from_list(&mut self) -> ManagerAction {
        match self.selected_repo() {
            Some(repo) => ManagerAction::RemoveFromList(repo.path.clone()),
            None => ManagerAction::Handled,
        }
    }

    /// `HomeFavorite`: star or un-star the selected repository.
    pub(crate) fn favorite_selected(&mut self) -> ManagerAction {
        match self.selected_repo() {
            Some(repo) => ManagerAction::ToggleFavorite(repo.path.clone()),
            None => ManagerAction::Handled,
        }
    }

    /// `HomeCloseRepo`: close, remove or un-star the selected repository.
    pub(crate) fn close_selected(&mut self) -> ManagerAction {
        if self.page == HomePage::Workspaces {
            return self.ws_close();
        }
        match self.selected_row() {
            Some(ManagerRow::Repo { section, repo }) => self.close_repo(section, repo),
            _ => ManagerAction::Handled,
        }
    }

    /// `GlobalContextMenu`: the selected repository's menu.
    pub(crate) fn menu_selected(&mut self) -> ManagerAction {
        if self.page == HomePage::Workspaces {
            return self.ws_menu();
        }
        match self.selected_row() {
            Some(ManagerRow::Repo { repo, .. }) => ManagerAction::Menu { repo },
            _ => ManagerAction::Handled,
        }
    }

    /// `HomeSearch`: focus the search field.
    pub(crate) fn focus_search(&mut self) -> ManagerAction {
        // The search filters repositories: it lives on their page.
        self.show_page(HomePage::Repositories);
        self.search_focused = true;
        ManagerAction::Handled
    }

    fn section_action(&mut self, section: Section) -> ManagerAction {
        match section {
            Section::Open => ManagerAction::CloseAllTabs,
            Section::Recent => ManagerAction::RemoveAllRecent,
            _ => ManagerAction::Handled,
        }
    }

    pub(crate) fn activate_selected(&mut self) -> ManagerAction {
        if self.page == HomePage::Workspaces {
            return self.ws_activate();
        }
        match self.selected_row() {
            Some(ManagerRow::Header(section)) => {
                self.toggle_section(section);
                ManagerAction::Handled
            }
            Some(ManagerRow::Repo { repo, .. }) => self.open_repo(repo),
            _ => ManagerAction::Handled,
        }
    }

    pub fn scroll_by(&mut self, delta: isize) {
        if self.page == HomePage::Workspaces {
            self.ws.scroll_by(delta);
            return;
        }
        let len = self.rows().len();
        let max = len.saturating_sub(self.list_height.max(1));
        self.scroll = (self.scroll as isize + delta).clamp(0, max as isize) as usize;
        self.follow = false;
    }
}

pub use input::{click, handle_event, menu_target, passes_through, ws_menu_for_hit};
pub use render::{render, ManagerRender};
pub use workspaces::{HomePage, WorkspacesPage, WsAction, WsEntry, WsHit, WsRepo};

// Only the tests name these.
#[cfg(test)]
pub use input::{handle_key, Key};
#[cfg(test)]
pub use render::{MORE, STAR_OFF, STAR_ON};

#[cfg(test)]
mod tests;
