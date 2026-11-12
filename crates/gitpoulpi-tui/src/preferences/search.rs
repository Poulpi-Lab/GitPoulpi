//! Settings search: a popup that finds a setting by label or help text and
//! jumps to it (switches page, scrolls to the row, selects and flashes it).
//!
//! Reachable from the keyboard (`/` or `Ctrl+F` on the Settings screen),
//! the mouse (a click on the search box at the top of the sidebar) and the
//! command palette (`App: Search Settings...`). The popup is modal: while
//! it is open it owns every key and every pointer event, and its hit rects
//! are recorded by the paint walk (`render_search`), never recomputed.

use crate::th;
use crate::wide_edge::Clear;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use super::{Control, PrefFocus, PrefSection, PreferencesState, SubTab};
use crate::display_width;
use gitpoulpi_i18n::msg::settings_screen as screen;

/// Result rows shown at once.
const VISIBLE_ROWS: usize = 10;

/// Text for an AI agent (the `ui show settings` errors): always English,
/// whatever language the window is in.
fn agent(text: impl FnOnce() -> String) -> String {
    gitpoulpi_i18n::with_lang(gitpoulpi_i18n::Lang::En, text)
}

/// The Radar block's controls, as the search lists them.
fn radar_labels() -> [&'static str; 7] {
    use gitpoulpi_i18n::msg::radar_settings as r;
    [
        r::HEADING.get(),
        r::REVIEW_REQUESTS.get(),
        r::OPEN_REQUESTS.get(),
        r::INCLUDE_DRAFTS.get(),
        r::ASSIGNED_ISSUES.get(),
        r::NAMESPACES.get(),
        r::REVIEW_MODE.get(),
    ]
}

/// One searchable setting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Hit {
    pub(super) section: usize,
    /// The sub-tab of a master-detail page the row lives on.
    pub(super) tab: SubTab,
    pub(super) field: usize,
    pub(super) label: String,
    pub(super) page: String,
    /// `true` when only the help text (not the label or page) matched.
    pub(super) in_help: bool,
    /// A control of the Integrations page's Radar block: the jump lands
    /// on it, not on a row of the field list.
    pub(super) radar: bool,
}

/// The open popup.
#[derive(Debug, Default)]
pub(super) struct SettingsSearch {
    pub(super) query: String,
    /// What `Ctrl+Z` / `Ctrl+Y` walk in the search box.
    history: crate::text_history::QueryHistory,
    pub(super) hits: Vec<Hit>,
    pub(super) selected: usize,
    /// Painted this frame: the popup, and each visible result row with its
    /// index into `hits`.
    pub(super) popup: Rect,
    /// The popup's close cross as last painted.
    pub(super) close: Option<Rect>,
    pub(super) rows: Vec<(Rect, usize)>,
}

/// One row offered to the search: what is shown (`label`, `page`) and what
/// is matched. `help` and the `*_en` fields are matched too, so the English
/// words still find a row while the interface is in another language.
#[derive(Debug, Clone, Default)]
pub(super) struct Entry {
    pub(super) section: usize,
    pub(super) tab: SubTab,
    pub(super) field: usize,
    pub(super) label: String,
    pub(super) page: String,
    pub(super) help: String,
    pub(super) label_en: String,
    pub(super) page_en: String,
    pub(super) help_en: String,
    pub(super) radar: bool,
}

/// The matches of `query` among `entries`, best first. Every
/// whitespace-separated word of the query has to occur (case insensitively)
/// in the label, the page title or the help, each in the interface
/// language or in English.
pub(super) fn matches(query: &str, entries: &[Entry]) -> Vec<Hit> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    if words.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(u8, usize, Hit)> = Vec::new();
    for (order, entry) in entries.iter().enumerate() {
        let (section, field, label, page) =
            (&entry.section, &entry.field, &entry.label, &entry.page);
        let tab = entry.tab;
        let label_l = format!("{} {}", label, entry.label_en).to_lowercase();
        let page_l = format!("{} {}", page, entry.page_en).to_lowercase();
        let help_l = format!("{} {}", entry.help, entry.help_en).to_lowercase();
        let tier = if words.iter().all(|w| label_l.contains(w)) {
            0
        } else if words
            .iter()
            .all(|w| label_l.contains(w) || page_l.contains(w))
        {
            1
        } else if words
            .iter()
            .all(|w| label_l.contains(w) || page_l.contains(w) || help_l.contains(w))
        {
            2
        } else {
            continue;
        };
        scored.push((
            tier,
            order,
            Hit {
                section: *section,
                tab,
                field: *field,
                label: label.clone(),
                page: page.clone(),
                in_help: tier == 2,
                radar: entry.radar,
            },
        ));
    }
    scored.sort_by_key(|(tier, order, _)| (*tier, *order));
    scored.into_iter().map(|(_, _, hit)| hit).collect()
}

impl PreferencesState {
    /// Every searchable row of every page, with the English texts beside
    /// the translated ones when the interface is not English.
    pub(super) fn search_entries(&self) -> Vec<Entry> {
        let mut out = self.search_entries_now();
        if gitpoulpi_i18n::current() != gitpoulpi_i18n::Lang::En {
            let english = {
                let _en = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::En);
                self.search_entries_now()
            };
            for (entry, en) in out.iter_mut().zip(english) {
                entry.label_en = en.label;
                entry.page_en = en.page;
                entry.help_en = en.help;
            }
        }
        out
    }

    fn search_entries_now(&self) -> Vec<Entry> {
        let mut out = Vec::new();
        for (section_index, section) in PrefSection::ALL.iter().enumerate() {
            if !section.is_built() {
                continue;
            }
            let page = if section.is_repo_scoped() {
                screen::page_this_repo(section.title())
            } else {
                section.title().to_string()
            };
            // A tabbed page lists the rows of both its tabs, General first.
            let tabs: &[SubTab] = if section.has_sub_tabs() {
                &[SubTab::General, SubTab::List]
            } else {
                &[SubTab::List]
            };
            for tab in tabs {
                for (index, field) in self.fields_of_tab(*section, *tab).iter().enumerate() {
                    if matches!(
                        field.control,
                        Control::Heading | Control::Info(_) | Control::Swatches(_)
                    ) {
                        continue;
                    }
                    out.push(Entry {
                        section: section_index,
                        tab: *tab,
                        field: index,
                        label: field.label.trim().to_string(),
                        page: page.clone(),
                        help: field.help.clone().unwrap_or_default(),
                        ..Entry::default()
                    });
                }
            }
            // The Radar block of a connected code host is a card of the
            // Integrations page, not a field list: its controls are found
            // by name and the jump opens the card.
            if *section == PrefSection::Integrations {
                for label in radar_labels() {
                    out.push(Entry {
                        section: section_index,
                        tab: SubTab::List,
                        label: label.to_string(),
                        page: page.clone(),
                        radar: true,
                        ..Entry::default()
                    });
                }
            }
            // The keybindings tab holds an editor, not rows: it is found by
            // its own name.
            if *section == PrefSection::Keybindings {
                out.push(Entry {
                    section: section_index,
                    tab: SubTab::List,
                    field: 0,
                    label: section.list_tab_title(),
                    page: page.clone(),
                    ..Entry::default()
                });
            }
        }
        out
    }

    /// Lands on `page`, or on the page that holds `setting` (a config key
    /// such as `ui.minimap`, or `keys.<action id>` on the Keyboard
    /// Shortcuts page), selecting and flashing the row - what an agent's
    /// `ui show settings` does. It never edits a value. `Err` names what
    /// was not found, and leaves the screen as it was.
    pub(crate) fn show_page(
        &mut self,
        page: Option<gitpoulpi_ipc::ui::SettingsPage>,
        setting: Option<&str>,
    ) -> Result<(), String> {
        use gitpoulpi_ipc::ui::SettingsPage as P;
        let index_of = |s: PrefSection| PrefSection::ALL.iter().position(|x| *x == s).unwrap_or(0);
        self.shown_setting = setting.map(str::to_string);
        let wanted = page.map(PrefSection::from_page);
        // A legacy page id, or `general`: rows moved, so a named setting is
        // looked up on every page.
        let search_all = page.is_some_and(P::is_legacy);
        if let Some(id) = setting.and_then(|k| k.strip_prefix("keys.")) {
            if wanted.is_some_and(|s| s != PrefSection::Keybindings) {
                return Err(agent(|| screen::agent_keys_page(id)));
            }
            let Some(action) = crate::keymap::ACTIONS.iter().position(|d| d.id == id) else {
                return Err(agent(|| screen::agent_no_action(id)));
            };
            self.show_keybindings(false);
            self.keys.selected = action;
            return Ok(());
        }
        let Some(key) = setting else {
            if let Some(section) = wanted {
                self.search = None;
                self.edit = None;
                self.choice = None;
                self.section = index_of(section);
                self.strip = false;
                self.focus = PrefFocus::Fields;
                self.scroll = 0;
                self.kept_in_view = None;
            }
            return Ok(());
        };
        for (i, section) in PrefSection::ALL.iter().enumerate() {
            if !search_all && wanted.is_some_and(|w| w != *section) {
                continue;
            }
            // Both tabs of a tabbed page: a setting lives on exactly one.
            let tabs: &[SubTab] = if section.has_sub_tabs() {
                &[SubTab::General, SubTab::List]
            } else {
                &[SubTab::List]
            };
            for tab in tabs {
                let field = self.fields_of_tab(*section, *tab).iter().position(|f| {
                    matches!(f.target, super::Target::Global(k) if k == key) && f.selectable()
                });
                if let Some(field) = field {
                    self.jump_to(&Hit {
                        section: i,
                        tab: *tab,
                        field,
                        label: String::new(),
                        page: String::new(),
                        in_help: false,
                        radar: false,
                    });
                    return Ok(());
                }
            }
        }
        Err(match page {
            Some(p) => agent(|| screen::agent_no_setting_on_page(key, format!("{p:?}"))),
            None => agent(|| screen::agent_no_setting(key)),
        })
    }

    /// Whether a value, the search box or a prompt is being typed into.
    pub(crate) fn typing(&self) -> bool {
        self.edit.is_some() || self.search.is_some() || self.choice.is_some()
    }

    /// The page showing and the row an agent landed on, for `ui state`.
    pub(crate) fn view_state(&self) -> (&'static str, Option<String>) {
        (self.current_section().id(), self.shown_setting.clone())
    }

    /// Opens the search popup (an empty query lists nothing yet).
    pub(crate) fn open_search(&mut self) {
        self.edit = None;
        self.choice = None;
        self.search = Some(SettingsSearch::default());
    }

    fn refresh_search(&mut self) {
        let query = self
            .search
            .as_ref()
            .map(|s| s.query.clone())
            .unwrap_or_default();
        let hits = matches(&query, &self.search_entries());
        if let Some(search) = self.search.as_mut() {
            search.hits = hits;
            search.selected = 0;
        }
    }

    /// Switches to the hit's page and lands on its row.
    pub(super) fn jump_to(&mut self, hit: &Hit) {
        if hit.radar {
            self.search = None;
            self.edit = None;
            self.choice = None;
            self.show_radar_options();
            return;
        }
        self.search = None;
        self.edit = None;
        self.choice = None;
        self.section = hit.section;
        self.remember_sub_tab(PrefSection::ALL[hit.section], hit.tab);
        self.strip = PrefSection::ALL[hit.section].has_strip();
        self.field = hit.field;
        self.focus = PrefFocus::Fields;
        self.scroll = 0;
        // Forces the keep-in-view rule to run even when the page and the
        // row did not change.
        self.kept_in_view = None;
        self.flash = Some((hit.section, hit.field));
    }

    fn jump_to_selected(&mut self) {
        let hit = self
            .search
            .as_ref()
            .and_then(|s| s.hits.get(s.selected).cloned());
        if let Some(hit) = hit {
            self.jump_to(&hit);
        }
    }

    pub(super) fn search_key(&mut self, key: KeyEvent) -> bool {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let Some(search) = self.search.as_mut() else {
            return true;
        };
        let last = search.hits.len().saturating_sub(1);
        // The shared popup-list keys: arrows, Ctrl+N/P/J/K, Home/End, PgUp/PgDn.
        let nav = crate::popup_list::nav(
            &key,
            search.selected,
            search.hits.len(),
            crate::popup_list::View::page().min(VISIBLE_ROWS),
        );
        match key.code {
            _ if nav.is_some() => search.selected = nav.unwrap_or(0),
            KeyCode::Esc => self.search = None,
            KeyCode::Enter => self.jump_to_selected(),
            KeyCode::Tab => search.selected = (search.selected + 1).min(last),
            KeyCode::BackTab => search.selected = search.selected.saturating_sub(1),
            // Ctrl+Z / Ctrl+Y walk the box's edits.
            _ if search.history.key(&mut search.query, &key) => self.refresh_search(),
            KeyCode::Backspace => {
                search.history.pop(&mut search.query);
                self.refresh_search();
            }
            KeyCode::Char('u') if ctrl => {
                search.history.wipe(&mut search.query);
                self.refresh_search();
            }
            KeyCode::Char(c) if !ctrl && !key.modifiers.contains(KeyModifiers::ALT) => {
                search.history.push(&mut search.query, c);
                self.refresh_search();
            }
            _ => {}
        }
        true
    }

    pub(super) fn search_paste(&mut self, text: &str) {
        if let Some(search) = self.search.as_mut() {
            let clean = text.replace(['\r', '\n'], " ");
            search.history.paste(&mut search.query, clean.trim_end());
            self.refresh_search();
        }
    }

    /// The popup is modal: every pointer event is its own.
    pub(super) fn search_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) {
        let Some(search) = self.search.as_mut() else {
            return;
        };
        use crate::popup_list::{View, WHEEL_STEP};
        let selected = search.selected;
        match kind {
            // The scrollbar first: its column is the popup's right border.
            MouseEventKind::Down(MouseButton::Left) if View::bar_press(col, row, selected) => {}
            MouseEventKind::Drag(MouseButton::Left) => View::bar_drag(row, selected),
            MouseEventKind::Up(MouseButton::Left) => View::bar_release(),
            MouseEventKind::Down(MouseButton::Left)
                if search.close.is_some_and(|r| r.contains((col, row).into())) =>
            {
                self.search = None;
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(index) = search
                    .rows
                    .iter()
                    .find(|(r, _)| r.contains((col, row).into()))
                    .map(|(_, i)| *i)
                {
                    search.selected = index;
                    // A click selects; the second click (double click) jumps.
                    if crate::popup_list::View::click(index) {
                        self.jump_to_selected();
                    }
                } else if !search.popup.contains((col, row).into())
                    && !self
                        .search_box
                        .is_some_and(|r| r.contains((col, row).into()))
                {
                    self.search = None;
                }
            }
            // The wheel scrolls the window; the selection stays put.
            MouseEventKind::ScrollDown => View::wheel(WHEEL_STEP as isize, selected),
            MouseEventKind::ScrollUp => View::wheel(-(WHEEL_STEP as isize), selected),
            _ => {}
        }
    }

    /// The search text box at the top of the sidebar; records its rect.
    /// The caret sits in it while the popup is open.
    pub(super) fn render_search_box(&mut self, frame: &mut Frame, sidebar: Rect) {
        self.search_box = None;
        if sidebar.width < 6 || sidebar.height < 3 {
            return;
        }
        let rect = Rect::new(sidebar.x, sidebar.y, sidebar.width - 1, 1);
        let hovered = self
            .mouse_pos
            .is_some_and(|(c, r)| rect.contains((c, r).into()));
        let focused = self.search.is_some();
        let bg = crate::theme::input_bg(focused, hovered);
        let buf = frame.buffer_mut();
        buf.set_style(rect, Style::default().bg(bg));
        let inner_w = rect.width.saturating_sub(2) as usize;
        let (text, style) = match self.search.as_ref() {
            Some(search) => {
                let mut q = search.query.clone();
                while display_width(&q) + 1 > inner_w && !q.is_empty() {
                    q.remove(0);
                }
                (
                    format!("{q}\u{2588}"),
                    Style::default().fg(th().ui.text).bg(bg),
                )
            }
            None => (
                screen::SEARCH_PLACEHOLDER.get().to_string(),
                Style::default().fg(th().ui.muted).bg(bg),
            ),
        };
        buf.set_stringn(rect.x + 1, rect.y, text, inner_w, style);
        crate::pointer_map::field(rect);
        self.search_box = Some(rect);
    }

    /// The popup, anchored directly under the search box, left-aligned
    /// with it and clamped to the screen; records its hit rects.
    pub(super) fn render_search(&mut self, frame: &mut Frame) {
        let (Some(search), Some(anchor)) = (self.search.as_mut(), self.search_box) else {
            return;
        };
        let screen = frame.area();
        let mouse = self.mouse_pos;
        let x = anchor.x.min(screen.right().saturating_sub(1));
        let y = anchor.bottom();
        let width = screen.right().saturating_sub(x).min(76);
        let shown = search.hits.len().clamp(1, VISIBLE_ROWS);
        let height = (shown as u16 + 2).min(screen.bottom().saturating_sub(y));
        if width < 4 || height < 3 {
            return;
        }
        let area = Rect::new(x, y, width, height);
        search.popup = area;
        crate::pointer_map::plain(area);
        search.rows.clear();
        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(format!(" {} ", screen::SEARCH_TITLE.get()))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(th().ui.accent));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let hovered = mouse.is_some_and(|(c, r)| {
            Rect::new(area.right().saturating_sub(5), area.y, 3, 1).contains((c, r).into())
        });
        search.close = crate::chrome::close_cross::paint(frame.buffer_mut(), area, hovered);

        let mut lines: Vec<Line<'static>> = Vec::new();
        if search.hits.is_empty() {
            lines.push(Line::from(Span::styled(
                if search.query.trim().is_empty() {
                    screen::SEARCH_HINT.get()
                } else {
                    screen::SEARCH_NONE.get()
                },
                Style::default().fg(th().ui.muted),
            )));
        }
        let first = crate::popup_list::View::window_start(
            search.selected,
            inner.height as usize,
            search.hits.len(),
        );
        for (offset, (index, hit)) in search
            .hits
            .iter()
            .enumerate()
            .skip(first)
            .take(VISIBLE_ROWS)
            .enumerate()
        {
            let y = inner.y + offset as u16;
            if y >= inner.bottom() {
                break;
            }
            let selected = index == search.selected;
            let style = if selected {
                Style::default().fg(th().ui.text).bg(th().ui.selection)
            } else {
                Style::default().fg(th().ui.text)
            };
            let page_style = if selected {
                Style::default().fg(th().ui.muted).bg(th().ui.selection)
            } else {
                Style::default().fg(th().ui.muted)
            };
            let tail = if hit.in_help {
                format!("  \u{00B7}  {} {}", hit.page, screen::DESCRIPTION_TAG.get())
            } else {
                format!("  \u{00B7}  {}", hit.page)
            };
            let used = display_width(&hit.label) + display_width(&tail);
            let pad = (inner.width as usize).saturating_sub(used);
            lines.push(Line::from(vec![
                Span::styled(hit.label.clone(), style.add_modifier(Modifier::BOLD)),
                Span::styled(tail, page_style),
                Span::styled(" ".repeat(pad), style),
            ]));
            search
                .rows
                .push((Rect::new(inner.x, y, inner.width, 1), index));
        }
        frame.render_widget(Paragraph::new(lines), inner);
        crate::popup_list::View::paint_bar(
            frame.buffer_mut(),
            Rect::new(area.x, inner.y, area.width, inner.height),
            search.hits.len(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(section: usize, field: usize, label: &str, help: &str) -> Entry {
        Entry {
            section,
            field,
            label: label.into(),
            page: "Page".into(),
            help: help.into(),
            ..Entry::default()
        }
    }

    #[test]
    fn label_matches_rank_before_description_matches_and_all_words_must_hit() {
        let entries = vec![
            entry(0, 0, "Auto-prune", "Removes stale refs on fetch."),
            entry(0, 1, "Wrap lines", "Long lines wrap."),
            entry(1, 0, "Other", "Also about prune and wrap."),
        ];
        let hits = matches("prune", &entries);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].label, "Auto-prune");
        assert!(!hits[0].in_help);
        assert!(hits[1].in_help);
        assert!(matches("prune wrap", &entries)
            .iter()
            .all(|h| h.label == "Other"));
        assert!(matches("   ", &entries).is_empty());
        assert!(matches("PRUNE", &entries).len() == 2, "case-insensitive");
    }
}
