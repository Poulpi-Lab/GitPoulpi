use super::*;

/// The keys this view understands, decoded by the caller - the same
/// shape `repo_switcher::Key` has, so this module needs no crossterm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Backspace,
    Enter,
    Esc,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    /// The row's context menu, opened by `.`, `Shift+F10` or the Menu/Apps
    /// key - see `context_menu::is_menu_key`, decoded from the raw
    /// crossterm event by `from_crossterm` since this enum takes no
    /// modifiers of its own.
    Menu,
    Other,
}

impl Key {
    /// `modifiers` is only consulted to tell `Shift+F10` (a menu key)
    /// apart from a plain, unmodified `F10` (currently unbound, falls to
    /// `Key::Other`) - see `context_menu::is_menu_key`.
    pub fn from_crossterm(code: KeyCode, modifiers: KeyModifiers) -> Self {
        if code != KeyCode::Char('.') && crate::context_menu::is_menu_key(code, modifiers) {
            return Key::Menu;
        }
        match code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Enter => Key::Enter,
            KeyCode::Esc => Key::Esc,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::Delete => Key::Delete,
            _ => Key::Other,
        }
    }
}

/// The window-level keys this view lets through to the ordinary
/// dispatch: the palette (`Ctrl+P`), the workspace search (`Ctrl+Alt+F`), Preferences (`Ctrl+,`), help
/// (`?`, unless it is being typed into the search), the navigation
/// prefix (`Ctrl+N`) and the tab picker (`t`) - neither while the search
/// is being typed into either, same reasoning as `?`. Everything else is
/// the view's - so no key reaches the empty panes behind it.
pub fn passes_through(manager: &RepoManager, key: &KeyEvent) -> bool {
    use crate::keymap::{self, Action};
    let km = keymap::current();
    // A review hosted over the Radar owns the keyboard like the review
    // dialog of a repository's window: only the palette reaches past it, and
    // only while no comment box or picker is open in it.
    if manager.radar.hosts_review() {
        return km.is(Action::GlobalPalette, key) && manager.radar.review_takes_palette();
    }
    if manager.radar.hosts_item() {
        return km.is(Action::GlobalPalette, key) && manager.radar.item_takes_palette();
    }
    if km.is(Action::GlobalPalette, key)
        || km.is(Action::GlobalSettings, key)
        || km.is(Action::GlobalWorkspaceSearch, key)
    {
        return true;
    }
    (km.is(Action::GlobalHelp, key)
        || km.is(Action::GlobalNavPrefix, key)
        || km.is(Action::GlobalSelectTab, key))
        && !manager.search_focused
        && !manager.radar.typing()
}

/// [`handle_event`] for a decoded key: the form the tests use. A `Key` carries no modifiers of its own, so one is rebuilt.
#[cfg(test)]
pub fn handle_key(manager: &mut RepoManager, key: Key, ctrl: bool) -> ManagerAction {
    let code = match key {
        Key::Char(c) => KeyCode::Char(c),
        Key::Backspace => KeyCode::Backspace,
        Key::Enter => KeyCode::Enter,
        Key::Esc => KeyCode::Esc,
        Key::Up => KeyCode::Up,
        Key::Down => KeyCode::Down,
        Key::Left => KeyCode::Left,
        Key::Right => KeyCode::Right,
        Key::Home => KeyCode::Home,
        Key::End => KeyCode::End,
        Key::PageUp => KeyCode::PageUp,
        Key::PageDown => KeyCode::PageDown,
        Key::Delete => KeyCode::Delete,
        Key::Menu => KeyCode::Menu,
        Key::Other => KeyCode::Null,
    };
    let modifiers = if ctrl {
        KeyModifiers::CONTROL
    } else {
        KeyModifiers::NONE
    };
    handle_event(manager, &KeyEvent::new(code, modifiers))
}

/// Every key, as a decision. The Home tab's own shortcuts and the global
/// quit / context-menu keys are looked up in the keymap (`home.*`);
/// motion, Enter, Esc and the search field's typing stay fixed.
pub fn handle_event(manager: &mut RepoManager, ev: &KeyEvent) -> ManagerAction {
    use crate::keymap::{self, Action as Bind, Ctx};
    let key = Key::from_crossterm(ev.code, ev.modifiers);
    let ctrl = ev.modifiers.contains(KeyModifiers::CONTROL);
    // Control chords belong to the window (`Ctrl+P`, `Ctrl+,`), except
    // `Ctrl+F`, which is *find* here as everywhere. The rest are
    // swallowed: `Ctrl+G p` must not try to pull a repository that is
    // not there.
    if ctrl {
        return match key {
            Key::Char('f') | Key::Char('F') => {
                manager.search_focused = manager.page == HomePage::Repositories;
                ManagerAction::Handled
            }
            Key::Char('p') | Key::Char('P') | Key::Char(',') => ManagerAction::NotHandled,
            // The chords below act on a repository, which this view has
            // none of: say so rather than swallowing them silently.
            Key::Char('g') | Key::Char('G') => {
                manager.message = Some(msg::home::actions_need_tab(
                    crate::keymap::current().shown(crate::keymap::Action::GlobalGitPrefix),
                ));
                ManagerAction::Handled
            }
            Key::Char('w') | Key::Char('W') | Key::Char('t') | Key::Char('T') => {
                manager.message = Some(msg::home::HOME_TAB_FIXED.get().into());
                ManagerAction::Handled
            }
            _ => ManagerAction::Handled,
        };
    }

    if manager.page == HomePage::Workspaces {
        return super::workspaces::handle_key(manager, ev, key);
    }

    if manager.search_focused {
        match key {
            Key::Char(c) => {
                manager.query.push(c);
                manager.select_first_repo();
            }
            Key::Backspace => {
                manager.query.pop();
                manager.select_first_repo();
            }
            // Esc clears first, then leaves the field: the same two
            // steps the Refs filter takes.
            Key::Esc if !manager.query.is_empty() => {
                manager.query.clear();
                manager.select_first_repo();
            }
            // A New tab has nothing behind its search field to fall back
            // to: Esc on the empty field closes the tab.
            Key::Esc if manager.new_tab => return ManagerAction::Leave,
            Key::Esc | Key::Down | Key::Up => manager.search_focused = false,
            // Enter opens the best match -- type three letters, Enter.
            Key::Enter => {
                manager.search_focused = false;
                return manager.activate_selected();
            }
            _ => {}
        }
        return ManagerAction::Handled;
    }

    manager.message = None;
    let km = keymap::current();
    if km.is(Bind::GlobalQuit, ev) {
        return ManagerAction::Quit;
    }
    if km.is(Bind::GlobalContextMenu, ev) {
        return manager.menu_selected();
    }
    match km.action_in(Ctx::Home, ev) {
        Some(Bind::HomeSearch) => return manager.focus_search(),
        Some(Bind::HomeFavorite) => return manager.favorite_selected(),
        Some(Bind::HomeCloseRepo) => return manager.close_selected(),
        Some(Bind::HomeBrowse) => return ManagerAction::Browse,
        Some(Bind::HomeClone) => return ManagerAction::Clone,
        Some(Bind::HomeInit) => return ManagerAction::Init,
        Some(Bind::HomeCollapseAll) => {
            manager.set_collapsed_all(true);
            return ManagerAction::Handled;
        }
        Some(Bind::HomeExpandAll) => {
            manager.set_collapsed_all(false);
            return ManagerAction::Handled;
        }
        _ => {}
    }
    let page = manager.list_height.max(1) as isize;
    match key {
        Key::Up | Key::Char('k') => manager.move_selection(-1),
        Key::Down | Key::Char('j') => manager.move_selection(1),
        Key::PageUp => manager.move_selection(-page),
        Key::PageDown => manager.move_selection(page),
        Key::Home => manager.move_selection(isize::MIN / 2),
        Key::End => manager.move_selection(isize::MAX / 2),
        Key::Enter => return manager.activate_selected(),
        // Left folds the section the cursor is in (from any of its
        // rows, landing on the header); Right unfolds it. Space toggles.
        Key::Left => {
            let section = manager.section_of(manager.selected);
            if !manager.collapsed.contains(&section) {
                manager.toggle_section(section);
            } else {
                manager.selected = manager
                    .rows()
                    .iter()
                    .position(|r| *r == ManagerRow::Header(section))
                    .unwrap_or(0);
            }
        }
        Key::Right => {
            if let Some(ManagerRow::Header(section)) = manager.selected_row() {
                if manager.collapsed.contains(&section) {
                    manager.toggle_section(section);
                }
            }
        }
        Key::Char(' ') => {
            let section = manager.section_of(manager.selected);
            manager.toggle_section(section);
        }
        Key::Esc => {
            if !manager.query.is_empty() {
                manager.query.clear();
                manager.select_first_repo();
            } else {
                return ManagerAction::Leave;
            }
        }
        // `?` is the window's help; everything else is swallowed, so a
        // stray key never reaches the empty panes behind the view.
        Key::Char('?') => return ManagerAction::NotHandled,
        _ => {}
    }
    ManagerAction::Handled
}

/// A left click on `hit`. `double` is a double click on the same cell.
///
/// The search box is the one text-entry field in this view, and every
/// click that isn't on it releases its typing focus. The query is kept,
/// so the list stays filtered; only the keyboard goes back to the view's
/// own shortcuts (`n`, `c`, `i`, ...). Done for every hit variant, so a
/// new one cannot forget it.
pub fn click(manager: &mut RepoManager, hit: ManagerHit, double: bool) -> ManagerAction {
    manager.message = None;
    if hit != ManagerHit::Search {
        manager.search_focused = false;
    }
    let rows = manager.rows();
    let repo_at = |index: usize| match rows.get(index) {
        Some(ManagerRow::Repo { section, repo }) => Some((*section, *repo)),
        _ => None,
    };
    match hit {
        ManagerHit::Button(ManagerButton::Browse) => ManagerAction::Browse,
        ManagerHit::Button(ManagerButton::Clone) => ManagerAction::Clone,
        ManagerHit::Button(ManagerButton::Init) => ManagerAction::Init,
        ManagerHit::Button(ManagerButton::CollapseAll) => {
            manager.set_collapsed_all(true);
            ManagerAction::Handled
        }
        ManagerHit::Button(ManagerButton::ExpandAll) => {
            manager.set_collapsed_all(false);
            ManagerAction::Handled
        }
        ManagerHit::Ws(hit) => super::workspaces::click(manager, hit, double),
        ManagerHit::Search => {
            manager.search_focused = true;
            ManagerAction::Handled
        }
        ManagerHit::Header(section) => {
            manager.toggle_section(section);
            ManagerAction::Handled
        }
        ManagerHit::SectionAction(section) => manager.section_action(section),
        ManagerHit::Row(index) => {
            manager.selected = index;
            if double {
                manager.activate_selected()
            } else {
                ManagerAction::Handled
            }
        }
        ManagerHit::Star(index) => match repo_at(index) {
            Some((_, repo)) => ManagerAction::ToggleFavorite(manager.repos[repo].path.clone()),
            None => ManagerAction::Handled,
        },
        ManagerHit::RowOpen(index) => match repo_at(index) {
            Some((_, repo)) => {
                manager.selected = index;
                manager.open_repo(repo)
            }
            None => ManagerAction::Handled,
        },
        ManagerHit::RowClose(index) => match repo_at(index) {
            Some((section, repo)) => {
                manager.selected = index;
                manager.close_repo(section, repo)
            }
            None => ManagerAction::Handled,
        },
        ManagerHit::RowMenu(index) => match repo_at(index) {
            Some((_, repo)) => {
                manager.selected = index;
                ManagerAction::Menu { repo }
            }
            None => ManagerAction::Handled,
        },
    }
}

/// A right click on the Workspaces page: selects the row and asks for its menu.
pub fn ws_menu_for_hit(manager: &mut RepoManager, hit: WsHit) -> Option<ManagerAction> {
    super::workspaces::menu_for_hit(manager, hit)
}

/// The repository a right click at `hit` is about, for its menu.
pub fn menu_target(manager: &RepoManager, hit: ManagerHit) -> Option<(usize, usize)> {
    let index = match hit {
        ManagerHit::Row(i)
        | ManagerHit::Star(i)
        | ManagerHit::RowOpen(i)
        | ManagerHit::RowClose(i)
        | ManagerHit::RowMenu(i) => i,
        _ => return None,
    };
    match manager.rows().get(index) {
        Some(ManagerRow::Repo { repo, .. }) => Some((index, *repo)),
        _ => None,
    }
}
