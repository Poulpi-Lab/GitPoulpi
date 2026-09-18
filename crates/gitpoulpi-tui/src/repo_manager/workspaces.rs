//! The Home tab's Workspaces page: the workspaces, grouped as in the
//! Repository Management, each expandable to its repositories.
//!
//! Pure like the rest of this view: data in, one render walk out (lines and
//! hit rects together), and key / click decisions that return a
//! [`WsAction`] for `tabs::apply_manager_action` (and `workspace_ui`) to
//! carry out. Nothing here touches a file or the daemon.
//!
//! ```text
//!  Repository Management                         [1 Repositories] [2 Workspaces]
//!
//!  [ New workspace (n) ] [ From open tabs (a) ]
//!
//!  ▾ ● Acme            3 repositories  as Work  active      [ Open ] [ Edit... ] [⋮]
//!       acme-api       ~/src/acme-api                       [ Open ] [Remove] [⋮]
//!       acme-web       ~/src/acme-web   missing   [Locate...] [Clone] [Remove] [⋮]
//! ```

use std::collections::HashSet;
use std::path::PathBuf;

use gitpoulpi_i18n::msg::workspaces as m;

use super::render::{button_style, highlight, Cursor, MORE};
use super::*;
use crate::th;

/// What the management window shows: the Home list, or (while the
/// Workspaces tab is selected) the Workspaces view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HomePage {
    Repositories,
    Workspaces,
}

/// One repository of a workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsRepo {
    pub path: PathBuf,
    pub name: String,
    /// The folder is there and is a repository.
    pub present: bool,
    /// The recorded `origin` URL, what Clone uses.
    pub remote: Option<String>,
}

/// One workspace, as the page lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsEntry {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub description: String,
    /// The default profile's name, when it names an existing profile.
    pub profile: Option<String>,
    pub repos: Vec<WsRepo>,
}

impl WsEntry {
    pub fn missing(&self) -> impl Iterator<Item = &WsRepo> {
        self.repos.iter().filter(|r| !r.present)
    }
}

/// One row of the page, in display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsRow {
    /// Indexes [`WorkspacesPage::entries`].
    Workspace(usize),
    Repo {
        ws: usize,
        repo: usize,
    },
    /// An expanded workspace with no repository.
    Empty(usize),
}

/// Where a click landed on the page. Row indices index [`WorkspacesPage::rows`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsHit {
    New,
    FromTabs,
    Row(usize),
    Open(usize),
    Edit(usize),
    Menu(usize),
    /// A missing repository's Locate...
    Locate(usize),
    Clone(usize),
    Remove(usize),
}

/// What the Home page asks the window to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsAction {
    /// Make the workspace active and show its tabs.
    Open(String),
    New,
    /// New workspace pre-filled with the open tabs.
    FromTabs,
    Edit(String),
    /// Asks first; never touches the repositories.
    Delete(String),
    FetchAll(String),
    PullAll(String),
    /// Clone every missing repository that has a recorded remote.
    CloneMissing(String),
    /// Open (or focus) the repository as a tab.
    OpenRepo(PathBuf),
    Locate {
        id: String,
        path: PathBuf,
    },
    CloneRepo {
        id: String,
        path: PathBuf,
    },
    RemoveRepo {
        id: String,
        path: PathBuf,
    },
    /// The row's menu, hung under the pointer or the selected row.
    Menu {
        id: String,
        repo: Option<PathBuf>,
    },
}

/// The page's state.
#[derive(Debug, Clone, Default)]
pub struct WorkspacesPage {
    pub entries: Vec<WsEntry>,
    /// The window's active workspace id.
    pub active: Option<String>,
    expanded: HashSet<String>,
    pub selected: usize,
    pub scroll: usize,
    follow: bool,
    list_height: usize,
    /// What a walk over a workspace is doing right now ("fetching Acme: 2
    /// of 5..."), shown on the footer line until it ends.
    pub progress: Option<String>,
    /// `workspace_ui::apply::epoch()` the entries were loaded at.
    pub epoch: u64,
}

impl WorkspacesPage {
    pub fn set_entries(&mut self, entries: Vec<WsEntry>, active: Option<String>) {
        let was = self.selected_key();
        self.entries = entries;
        self.active = active;
        self.expanded
            .retain(|id| self.entries.iter().any(|e| e.id == *id));
        if let Some(key) = was {
            if let Some(i) = self.rows().iter().position(|r| self.key_of(*r) == key) {
                self.selected = i;
            }
        }
        self.clamp();
    }

    fn key_of(&self, row: WsRow) -> (String, Option<PathBuf>) {
        match row {
            WsRow::Workspace(i) | WsRow::Empty(i) => (self.entries[i].id.clone(), None),
            WsRow::Repo { ws, repo } => (
                self.entries[ws].id.clone(),
                Some(self.entries[ws].repos[repo].path.clone()),
            ),
        }
    }

    fn selected_key(&self) -> Option<(String, Option<PathBuf>)> {
        self.rows().get(self.selected).map(|r| self.key_of(*r))
    }

    pub fn rows(&self) -> Vec<WsRow> {
        let mut rows = Vec::new();
        for (i, e) in self.entries.iter().enumerate() {
            rows.push(WsRow::Workspace(i));
            if self.expanded.contains(&e.id) {
                if e.repos.is_empty() {
                    rows.push(WsRow::Empty(i));
                }
                rows.extend((0..e.repos.len()).map(|repo| WsRow::Repo { ws: i, repo }));
            }
        }
        rows
    }

    pub fn selected_row(&self) -> Option<WsRow> {
        self.rows().get(self.selected).copied()
    }

    /// The workspace the cursor is in (on its row or one of its repositories).
    pub fn selected_workspace(&self) -> Option<&WsEntry> {
        match self.selected_row()? {
            WsRow::Workspace(i) | WsRow::Empty(i) | WsRow::Repo { ws: i, .. } => {
                self.entries.get(i)
            }
        }
    }

    pub fn selected_repo(&self) -> Option<(&WsEntry, &WsRepo)> {
        match self.selected_row()? {
            WsRow::Repo { ws, repo } => {
                let e = self.entries.get(ws)?;
                Some((e, e.repos.get(repo)?))
            }
            _ => None,
        }
    }

    fn clamp(&mut self) {
        let len = self.rows().len();
        self.selected = self.selected.min(len.saturating_sub(1));
        self.follow = true;
    }

    fn move_selection(&mut self, delta: isize) {
        let len = self.rows().len();
        if len == 0 {
            return;
        }
        self.selected = (self.selected as isize + delta).clamp(0, len as isize - 1) as usize;
        self.follow = true;
    }

    pub fn scroll_by(&mut self, delta: isize) {
        let len = self.rows().len();
        let max = len.saturating_sub(self.list_height.max(1));
        self.scroll = (self.scroll as isize + delta).clamp(0, max as isize) as usize;
        self.follow = false;
    }

    fn toggle(&mut self, id: &str) {
        if !self.expanded.remove(id) {
            self.expanded.insert(id.to_string());
        }
        self.clamp();
    }

    pub(super) fn set_all(&mut self, open: bool) {
        if open {
            self.expanded = self.entries.iter().map(|e| e.id.clone()).collect();
        } else {
            self.expanded.clear();
            // The cursor follows its workspace's row.
            if let Some(ws) = self.selected_workspace().map(|e| e.id.clone()) {
                if let Some(i) = self
                    .rows()
                    .iter()
                    .position(|r| matches!(r, WsRow::Workspace(w) if self.entries[*w].id == ws))
                {
                    self.selected = i;
                }
            }
        }
        self.clamp();
    }

    fn row_action(&self, row: WsRow) -> Option<WsAction> {
        Some(match row {
            WsRow::Workspace(i) | WsRow::Empty(i) => WsAction::Open(self.entries[i].id.clone()),
            WsRow::Repo { ws, repo } => {
                let r = &self.entries[ws].repos[repo];
                if r.present {
                    WsAction::OpenRepo(r.path.clone())
                } else {
                    WsAction::Locate {
                        id: self.entries[ws].id.clone(),
                        path: r.path.clone(),
                    }
                }
            }
        })
    }

    fn menu_for(&self, row: WsRow) -> WsAction {
        match row {
            WsRow::Workspace(i) | WsRow::Empty(i) => WsAction::Menu {
                id: self.entries[i].id.clone(),
                repo: None,
            },
            WsRow::Repo { ws, repo } => WsAction::Menu {
                id: self.entries[ws].id.clone(),
                repo: Some(self.entries[ws].repos[repo].path.clone()),
            },
        }
    }
}

impl RepoManager {
    /// Switches page (`1`, `2`, the switcher).
    pub fn show_page(&mut self, page: HomePage) {
        if self.page != page {
            self.page = page;
            self.search_focused = false;
            self.message = None;
        }
    }

    /// A row is selected on the page that shows (palette availability).
    pub fn row_selected(&self) -> bool {
        match self.page {
            HomePage::Workspaces => self.ws.selected_row().is_some(),
            HomePage::Repositories => self.selected_repo().is_some(),
        }
    }

    /// The hit that stands for the selected row (a keyboard-opened menu
    /// hangs under it).
    pub fn selected_row_hit(&self) -> ManagerHit {
        if self.page == HomePage::Workspaces {
            ManagerHit::Ws(WsHit::Row(self.ws.selected))
        } else {
            ManagerHit::Row(self.selected)
        }
    }

    fn ws_rows(&self) -> Vec<WsRow> {
        self.ws.rows()
    }

    /// The action for the selected row (Enter, double click).
    pub(crate) fn ws_activate(&mut self) -> ManagerAction {
        match self.ws.selected_row().and_then(|r| self.ws.row_action(r)) {
            Some(action) => ManagerAction::Workspace(action),
            None => ManagerAction::Handled,
        }
    }

    pub(crate) fn ws_menu(&mut self) -> ManagerAction {
        match self.ws.selected_row() {
            Some(row) => ManagerAction::Workspace(self.ws.menu_for(row)),
            None => ManagerAction::Handled,
        }
    }

    /// The Workspaces page's `x` / Delete: a workspace asks to be deleted, a
    /// repository leaves its workspace.
    pub(crate) fn ws_close(&mut self) -> ManagerAction {
        match self.ws.selected_row() {
            Some(WsRow::Repo { ws, repo }) => ManagerAction::Workspace(WsAction::RemoveRepo {
                id: self.ws.entries[ws].id.clone(),
                path: self.ws.entries[ws].repos[repo].path.clone(),
            }),
            Some(WsRow::Workspace(i) | WsRow::Empty(i)) => {
                ManagerAction::Workspace(WsAction::Delete(self.ws.entries[i].id.clone()))
            }
            None => ManagerAction::Handled,
        }
    }

    /// Edit, Fetch all... for the workspace under the cursor.
    pub(crate) fn ws_on_selected(
        &mut self,
        make: impl FnOnce(String) -> WsAction,
    ) -> ManagerAction {
        match self.ws.selected_workspace() {
            Some(e) => ManagerAction::Workspace(make(e.id.clone())),
            None => {
                self.message = Some(m::SELECT_FIRST.get().into());
                ManagerAction::Handled
            }
        }
    }

    pub fn ws_locate_selected(&mut self) -> ManagerAction {
        match self.ws.selected_repo() {
            Some((e, r)) => ManagerAction::Workspace(WsAction::Locate {
                id: e.id.clone(),
                path: r.path.clone(),
            }),
            None => {
                self.message = Some(m::SELECT_REPOSITORY_FIRST.get().into());
                ManagerAction::Handled
            }
        }
    }
}

/// Keys of the Workspaces page. `ev` is already known not to be a `Ctrl`
/// chord and not typed into a field.
pub(super) fn handle_key(
    manager: &mut RepoManager,
    ev: &KeyEvent,
    key: super::input::Key,
) -> ManagerAction {
    use super::input::Key;
    use crate::keymap::{self, Action as Bind, Ctx};
    manager.message = None;
    let km = keymap::current();
    if km.is(Bind::GlobalQuit, ev) {
        return ManagerAction::Quit;
    }
    if km.is(Bind::GlobalContextMenu, ev) {
        return manager.ws_menu();
    }
    match km.action_in(Ctx::Home, ev) {
        Some(Bind::HomeWsNew) => return ManagerAction::Workspace(WsAction::New),
        Some(Bind::HomeWsFromTabs) => return ManagerAction::Workspace(WsAction::FromTabs),
        Some(Bind::HomeWsEdit) => return manager.ws_on_selected(WsAction::Edit),
        Some(Bind::HomeWsFetch) => return manager.ws_on_selected(WsAction::FetchAll),
        Some(Bind::HomeWsPull) => return manager.ws_on_selected(WsAction::PullAll),
        Some(Bind::HomeWsCloneMissing) => return manager.ws_on_selected(WsAction::CloneMissing),
        Some(Bind::HomeWsLocate) => return manager.ws_locate_selected(),
        Some(Bind::HomeCloseRepo) => return manager.ws_close(),
        Some(Bind::HomeCollapseAll) => {
            manager.ws.set_all(false);
            return ManagerAction::Handled;
        }
        Some(Bind::HomeExpandAll) => {
            manager.ws.set_all(true);
            return ManagerAction::Handled;
        }
        _ => {}
    }
    let page = manager.ws.list_height.max(1) as isize;
    match key {
        Key::Up | Key::Char('k') => manager.ws.move_selection(-1),
        Key::Down | Key::Char('j') => manager.ws.move_selection(1),
        Key::PageUp => manager.ws.move_selection(-page),
        Key::PageDown => manager.ws.move_selection(page),
        Key::Home => manager.ws.move_selection(isize::MIN / 2),
        Key::End => manager.ws.move_selection(isize::MAX / 2),
        Key::Enter => return manager.ws_activate(),
        Key::Char(' ') => {
            if let Some(id) = manager.ws.selected_workspace().map(|e| e.id.clone()) {
                manager.ws.toggle(&id);
            }
        }
        Key::Right => {
            if let Some(id) = manager.ws.selected_workspace().map(|e| e.id.clone()) {
                if !manager.ws.expanded.contains(&id) {
                    manager.ws.toggle(&id);
                }
            }
        }
        // Left folds the workspace the cursor is in; on a repository it
        // first goes up to the workspace's own row.
        Key::Left => {
            if let Some(id) = manager.ws.selected_workspace().map(|e| e.id.clone()) {
                let on_workspace = matches!(
                    manager.ws.selected_row(),
                    Some(WsRow::Workspace(_) | WsRow::Empty(_))
                );
                if on_workspace && manager.ws.expanded.contains(&id) {
                    manager.ws.toggle(&id);
                } else if let Some(i) = manager.ws.rows().iter().position(
                    |r| matches!(r, WsRow::Workspace(w) if manager.ws.entries[*w].id == id),
                ) {
                    manager.ws.selected = i;
                    manager.ws.follow = true;
                }
            }
        }
        Key::Esc => return ManagerAction::Leave,
        Key::Char('?') => return ManagerAction::NotHandled,
        _ => {}
    }
    ManagerAction::Handled
}

/// A left click on the page.
pub(super) fn click(manager: &mut RepoManager, hit: WsHit, double: bool) -> ManagerAction {
    let rows = manager.ws_rows();
    let select = |manager: &mut RepoManager, index: usize| {
        manager.ws.selected = index;
        manager.ws.follow = true;
    };
    match hit {
        WsHit::New => ManagerAction::Workspace(WsAction::New),
        WsHit::FromTabs => ManagerAction::Workspace(WsAction::FromTabs),
        WsHit::Row(index) => {
            select(manager, index);
            let Some(row) = rows.get(index).copied() else {
                return ManagerAction::Handled;
            };
            if double {
                return manager.ws_activate();
            }
            // A click on a workspace's row folds it; on a repository, selects.
            if let WsRow::Workspace(i) | WsRow::Empty(i) = row {
                let id = manager.ws.entries[i].id.clone();
                manager.ws.toggle(&id);
            }
            ManagerAction::Handled
        }
        WsHit::Open(index) | WsHit::Locate(index) => {
            select(manager, index);
            match rows.get(index).and_then(|r| manager.ws.row_action(*r)) {
                Some(action) => ManagerAction::Workspace(action),
                None => ManagerAction::Handled,
            }
        }
        WsHit::Edit(index) => {
            select(manager, index);
            manager.ws_on_selected(WsAction::Edit)
        }
        WsHit::Menu(index) => {
            select(manager, index);
            manager.ws_menu()
        }
        WsHit::Clone(index) => {
            select(manager, index);
            match manager.ws.selected_repo() {
                Some((e, r)) => ManagerAction::Workspace(WsAction::CloneRepo {
                    id: e.id.clone(),
                    path: r.path.clone(),
                }),
                None => ManagerAction::Handled,
            }
        }
        WsHit::Remove(index) => {
            select(manager, index);
            manager.ws_close()
        }
    }
}

/// The row a right click at `hit` is about, selecting it.
pub(super) fn menu_for_hit(manager: &mut RepoManager, hit: WsHit) -> Option<ManagerAction> {
    let index = match hit {
        WsHit::Row(i)
        | WsHit::Open(i)
        | WsHit::Edit(i)
        | WsHit::Menu(i)
        | WsHit::Locate(i)
        | WsHit::Clone(i)
        | WsHit::Remove(i) => i,
        _ => return None,
    };
    manager.ws.rows().get(index)?;
    manager.ws.selected = index;
    Some(manager.ws_menu())
}

// -- drawing ------------------------------------------------------------

/// Rows above the list: title, blank, buttons, hint, blank.
const CHROME_TOP: u16 = 5;
const CHROME_BOTTOM: u16 = 1;

fn matches_row(hover: Option<ManagerHit>, offset: usize) -> bool {
    matches!(hover, Some(ManagerHit::Ws(WsHit::Row(i))) if i == offset)
}

/// The list part of the page: everything under the title row. `lines[0]` is
/// the line at `area.y + 1` (the title is the caller's).
pub(super) fn render(
    manager: &mut RepoManager,
    area: Rect,
    lines: &mut Vec<Line<'static>>,
    hits: &mut Vec<(Rect, ManagerHit)>,
) {
    lines.push(Line::default());
    let width = area.width as usize;
    let hover = manager.hover;

    // Buttons.
    let y = area.y + 2;
    let mut c = Cursor::new(area, y);
    c.push(" ", Style::default());
    let km = crate::keymap::current();
    for (hit, label, action) in [
        (
            WsHit::New,
            m::BTN_NEW.get(),
            crate::keymap::Action::HomeWsNew,
        ),
        (
            WsHit::FromTabs,
            m::BTN_FROM_TABS.get(),
            crate::keymap::Action::HomeWsFromTabs,
        ),
    ] {
        let hovered = hover == Some(ManagerHit::Ws(hit));
        let key = km.shown(action);
        let text = if key.is_empty() {
            format!(" {label} ")
        } else {
            format!(" {label} ({key}) ")
        };
        let rect = c.push(&text, button_style(hovered));
        hits.push((rect, ManagerHit::Ws(hit)));
        c.push(" ", Style::default());
    }
    let right_width: usize = [ManagerButton::CollapseAll, ManagerButton::ExpandAll]
        .iter()
        .map(|b| display_width(b.label()) + 2 + 1)
        .sum();
    if c.room() > right_width + 1 {
        c.pad_to(area.x + area.width - right_width as u16, Style::default());
        for button in [ManagerButton::CollapseAll, ManagerButton::ExpandAll] {
            let hovered = hover == Some(ManagerHit::Button(button));
            let rect = c.push(&format!(" {} ", button.label()), button_style(hovered));
            hits.push((rect, ManagerHit::Button(button)));
            c.push(" ", Style::default());
        }
    }
    lines.push(c.line());

    // A one-line hint.
    let mut c = Cursor::new(area, area.y + 3);
    c.push(
        &format!(" {}", m::PAGE_HINT.get()),
        Style::default().fg(th().ui.muted),
    );
    lines.push(c.line());
    lines.push(Line::default());

    // The list.
    let list_top = area.y + CHROME_TOP;
    let list_height = area.height.saturating_sub(CHROME_TOP + CHROME_BOTTOM) as usize;
    manager.ws.list_height = list_height.max(1);
    let rows = manager.ws.rows();
    if manager.ws.follow && list_height > 0 {
        if manager.ws.selected < manager.ws.scroll {
            manager.ws.scroll = manager.ws.selected;
        } else if manager.ws.selected >= manager.ws.scroll + list_height {
            manager.ws.scroll = manager.ws.selected + 1 - list_height;
        }
    }
    manager.ws.scroll = manager
        .ws
        .scroll
        .min(rows.len().saturating_sub(list_height.max(1)));

    if manager.ws.entries.is_empty() {
        let mut c = Cursor::new(area, list_top);
        c.push(
            &format!("  {}", m::EMPTY.get()),
            Style::default()
                .fg(th().ui.muted)
                .add_modifier(Modifier::ITALIC),
        );
        lines.push(c.line());
    }
    let name_w = manager
        .ws
        .entries
        .iter()
        .map(|e| display_width(&e.name))
        .max()
        .unwrap_or(8)
        .clamp(8, 26);
    for (offset, row) in rows
        .iter()
        .enumerate()
        .skip(manager.ws.scroll)
        .take(list_height)
    {
        let y = list_top + (offset - manager.ws.scroll) as u16;
        let selected = offset == manager.ws.selected;
        let line = match *row {
            WsRow::Workspace(i) => workspace_line(manager, i, offset, name_w, area, y, hits),
            WsRow::Empty(_) => {
                let mut c = Cursor::new(area, y);
                c.push(
                    &format!("      {}", m::EMPTY_WORKSPACE.get()),
                    Style::default()
                        .fg(th().ui.muted)
                        .add_modifier(Modifier::ITALIC),
                );
                c.line()
            }
            WsRow::Repo { ws, repo } => repo_line(manager, ws, repo, offset, name_w, area, y, hits),
        };
        // Band last-registered: the row band covers what its buttons did not.
        hits.push((
            Rect::new(area.x, y, area.width, 1),
            ManagerHit::Ws(WsHit::Row(offset)),
        ));
        lines.push(if selected {
            highlight(line, width, th().ui.selection)
        } else if matches_row(hover, offset) {
            highlight(line, width, th().ui.hover)
        } else {
            line
        });
    }
    while lines.len() < CHROME_TOP as usize + list_height {
        lines.push(Line::default());
    }
}

/// The footer of the page: progress, a message, or the keys.
pub(super) fn footer(manager: &RepoManager, width: usize) -> Line<'static> {
    use crate::keymap::Action as Bind;
    if let Some(text) = manager
        .mode_hint
        .as_ref()
        .or(manager.message.as_ref())
        .or(manager.ws.progress.as_ref())
    {
        let color = if manager.ws.progress.is_some()
            && manager.message.is_none()
            && manager.mode_hint.is_none()
        {
            th().ui.accent
        } else {
            th().status.warning
        };
        return Line::from(Span::styled(
            fit(&format!(" {text}"), width),
            Style::default().fg(color),
        ));
    }
    let km = crate::keymap::current();
    let mut spans = vec![Span::raw(" ")];
    let mut hints: Vec<(String, &'static str)> = vec![(
        crate::dialog_kit::key_name(crate::keymap::chord::Key::Enter),
        gitpoulpi_i18n::msg::dialog::HINT_OPEN.get(),
    )];
    for (action, what) in [
        (Bind::HomeWsNew, m::HINT_NEW.get()),
        (Bind::HomeWsEdit, m::HINT_EDIT.get()),
        (Bind::HomeCloseRepo, m::HINT_DELETE_REMOVE.get()),
        (Bind::HomeWsFetch, m::HINT_FETCH.get()),
        (Bind::HomeWsPull, m::HINT_PULL.get()),
        (
            Bind::GlobalContextMenu,
            gitpoulpi_i18n::msg::home::HINT_MENU.get(),
        ),
    ] {
        let shown = km.shown(action);
        if !shown.is_empty() {
            hints.push((shown, what));
        }
    }
    hints.push((
        crate::dialog_kit::key_name(crate::keymap::chord::Key::Esc),
        gitpoulpi_i18n::msg::home::HINT_BACK.get(),
    ));
    for (key, what) in hints {
        spans.push(Span::styled(key, Style::default().fg(th().ui.accent)));
        spans.push(Span::styled(
            format!(" {what}   "),
            Style::default().fg(th().ui.muted),
        ));
    }
    Line::from(spans)
}

#[allow(clippy::too_many_arguments)]
fn workspace_line(
    manager: &RepoManager,
    index: usize,
    offset: usize,
    name_w: usize,
    area: Rect,
    y: u16,
    hits: &mut Vec<(Rect, ManagerHit)>,
) -> Line<'static> {
    let entry = &manager.ws.entries[index];
    let hover = manager.hover;
    let open = manager.ws.expanded.contains(&entry.id);
    let active = manager.ws.active.as_deref() == Some(entry.id.as_str());

    let open_text = format!(" {} ", gitpoulpi_i18n::msg::home::ROW_OPEN.get());
    let edit_text = format!(" {} ", m::ROW_EDIT.get());
    let more_text = format!(" {MORE} ");
    let actions_width = display_width(&open_text)
        + 1
        + display_width(&edit_text)
        + 1
        + display_width(&more_text)
        + 2;

    let mut c = Cursor::new(area, y);
    // The band marks the selection; the arrow says folded or not.
    c.push("  ", Style::default());
    c.push(
        &format!(" {} ", crate::fold_mark::glyph_open(open)),
        Style::default().fg(th().ui.accent),
    );
    c.push(
        &format!("{} ", crate::workspace_ui::DOT),
        Style::default().fg(crate::workspace_ui::color_of(entry.color.as_deref())),
    );
    let content_end = (area.x + area.width).saturating_sub(actions_width as u16);
    c.end = content_end;
    c.push(
        &format!("{:<w$}  ", fit(&entry.name, name_w), w = name_w),
        Style::default()
            .fg(th().ui.text)
            .add_modifier(Modifier::BOLD),
    );
    let missing = entry.missing().count();
    c.push(
        &format!("{}  ", m::repo_count(entry.repos.len() as u64)),
        Style::default().fg(th().ui.muted),
    );
    if missing > 0 {
        c.push(
            &format!("{}  ", m::missing_count(missing as u64)),
            Style::default().fg(th().status.warning),
        );
    }
    if let Some(profile) = &entry.profile {
        c.push(
            &format!("{}  ", m::profile_label(profile)),
            Style::default().fg(th().ui.muted),
        );
    }
    if active {
        c.push(
            &format!("{}  ", m::ACTIVE.get()),
            Style::default()
                .fg(th().ui.accent)
                .add_modifier(Modifier::BOLD),
        );
    }
    c.end = area.x + area.width;
    c.pad_to(content_end, Style::default());
    let mut action = |c: &mut Cursor, label: &str, hit: WsHit| {
        let hovered = hover == Some(ManagerHit::Ws(hit));
        let rect = c.push(label, button_style(hovered));
        hits.push((rect, ManagerHit::Ws(hit)));
        c.push(" ", Style::default());
    };
    action(&mut c, &open_text, WsHit::Open(offset));
    action(&mut c, &edit_text, WsHit::Edit(offset));
    action(&mut c, &more_text, WsHit::Menu(offset));
    c.line()
}

#[allow(clippy::too_many_arguments)]
fn repo_line(
    manager: &RepoManager,
    ws: usize,
    repo: usize,
    offset: usize,
    name_w: usize,
    area: Rect,
    y: u16,
    hits: &mut Vec<(Rect, ManagerHit)>,
) -> Line<'static> {
    let entry = &manager.ws.entries[ws];
    let r = &entry.repos[repo];
    let hover = manager.hover;
    let fg = if r.present {
        th().ui.text
    } else {
        th().ui.muted
    };

    let first_text = format!(
        " {} ",
        if r.present {
            gitpoulpi_i18n::msg::home::ROW_OPEN.get()
        } else {
            m::ROW_LOCATE.get()
        }
    );
    let clone_text = format!(" {} ", m::ROW_CLONE.get());
    let remove_text = format!(" {} ", gitpoulpi_i18n::msg::home::ROW_REMOVE.get());
    let more_text = format!(" {MORE} ");
    let can_clone = !r.present && r.remote.is_some();
    let actions_width = display_width(&first_text)
        + 1
        + if can_clone {
            display_width(&clone_text) + 1
        } else {
            0
        }
        + display_width(&remove_text)
        + 1
        + display_width(&more_text)
        + 2;

    let mut c = Cursor::new(area, y);
    c.push("      ", Style::default());
    let content_end = (area.x + area.width).saturating_sub(actions_width as u16);
    c.end = content_end;
    c.push(
        &format!("{:<w$}  ", fit(&r.name, name_w), w = name_w),
        Style::default().fg(fg),
    );
    if !r.present {
        c.push(
            &format!("{}  ", m::MISSING.get()),
            Style::default().fg(th().status.warning),
        );
    }
    let room = c.room();
    c.push(
        &crate::tail_fit(&r.path.display().to_string(), room),
        Style::default().fg(th().ui.muted),
    );
    c.end = area.x + area.width;
    c.pad_to(content_end, Style::default());
    let mut action = |c: &mut Cursor, label: &str, hit: WsHit| {
        let hovered = hover == Some(ManagerHit::Ws(hit));
        let rect = c.push(label, button_style(hovered));
        hits.push((rect, ManagerHit::Ws(hit)));
        c.push(" ", Style::default());
    };
    action(
        &mut c,
        &first_text,
        if r.present {
            WsHit::Open(offset)
        } else {
            WsHit::Locate(offset)
        },
    );
    if can_clone {
        action(&mut c, &clone_text, WsHit::Clone(offset));
    }
    action(&mut c, &remove_text, WsHit::Remove(offset));
    action(&mut c, &more_text, WsHit::Menu(offset));
    c.line()
}
