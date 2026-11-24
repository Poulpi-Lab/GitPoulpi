//! Sub-tabs of the master-detail pages (`PrefSection::has_list`) and of the
//! Keyboard & mouse page (its generic rows, then the keybindings).
//!
//! Every such page splits in two so a global setting is never mistaken for a
//! property of the selected item: a **General** tab (what the page does as a
//! whole: the active profile, the default entry, the global instructions...)
//! and a **list** tab named after the items, holding the list and the selected
//! item's rows together.
//!
//! The tab in front is remembered per page for the session (`LAST_TABS`, like
//! the last section), and General is the default. Everything that reads a
//! page's rows goes through [`PreferencesState::fields_of_tab`], so the
//! search, the palette's `Settings:` rows and the "go to this setting" links
//! name the tab a row lives on and land on it.

use super::*;
use crate::dialog_kit;
use crate::keymap::Action;
use crate::th;
use gitpoulpi_i18n::msg::settings_screen as screen;

/// Which half of a master-detail page is in front.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SubTab {
    /// The list and the selected item's rows.
    List,
    /// The page's own settings, none of them an item's (the default: a page
    /// opens on it).
    #[default]
    General,
}

/// Pages on their list tab, one bit per `PrefSection::ALL` index, kept for
/// the process: reopening Settings lands where the user was. Only
/// `PreferencesState::open` reads it, so unit tests stay independent.
pub(super) static LAST_TABS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl PrefSection {
    /// Whether the page is split into a General tab and a list tab.
    pub(crate) fn has_sub_tabs(self) -> bool {
        (self.has_list() && self != PrefSection::Sequences) || self == PrefSection::Keybindings
    }

    /// The list tab's label: the items' name, as the list titles it.
    pub(super) fn list_tab_title(self) -> String {
        use gitpoulpi_i18n::msg as m;
        match self {
            PrefSection::Profiles => m::settings_profiles::LIST_TITLE.get().to_string(),
            PrefSection::Ai => m::ai_page::PROFILES_HEADING.get().to_string(),
            PrefSection::OpenIn => m::settings_open_in::LIST_TITLE.get().to_string(),
            PrefSection::Images => m::settings_images::LIST_TITLE.get().to_string(),
            PrefSection::Workflow => m::settings_workflow::TAB_RULES.get().to_string(),
            PrefSection::Keybindings => screen::TAB_KEYBINDINGS.get().to_string(),
            _ => String::new(),
        }
    }
}

impl PreferencesState {
    /// The tab of `section` (the list tab for a page without tabs).
    pub(super) fn sub_tab_of(&self, section: PrefSection) -> SubTab {
        let at = PrefSection::ALL.iter().position(|s| *s == section);
        match at {
            Some(i) if section.has_sub_tabs() => {
                if self.list_tabs & (1 << i) != 0 {
                    SubTab::List
                } else {
                    SubTab::General
                }
            }
            _ => SubTab::List,
        }
    }

    /// The tab of the page in front.
    pub(super) fn sub_tab(&self) -> SubTab {
        self.sub_tab_of(self.current_section())
    }

    /// Whether the list (and the keyboard's List focus) is on screen.
    pub(super) fn list_shown(&self) -> bool {
        self.current_section().has_list() && self.sub_tab() == SubTab::List
    }

    /// Records `tab` for `section` without touching the cursor.
    pub(super) fn remember_sub_tab(&mut self, section: PrefSection, tab: SubTab) {
        if let Some(i) = PrefSection::ALL.iter().position(|s| *s == section) {
            match tab {
                SubTab::List => self.list_tabs |= 1 << i,
                SubTab::General => self.list_tabs &= !(1 << i),
            }
        }
    }

    /// Brings `tab` of the current page to the front: the cursor goes to its
    /// first row, the list gets the keyboard when it is shown.
    pub(super) fn set_sub_tab(&mut self, tab: SubTab) {
        if !self.current_section().has_sub_tabs() {
            return;
        }
        self.edit = None;
        self.choice = None;
        self.save_view();
        self.remember_sub_tab(self.current_section(), tab);
        self.strip = false;
        self.restore_view();
        if self.focus != PrefFocus::Sections {
            self.focus = if tab == SubTab::List && self.current_section().has_list() {
                PrefFocus::List
            } else {
                PrefFocus::Fields
            };
        }
    }

    /// The other tab, by key (`Alt+Right` / `Alt+Left`: with two tabs, a step
    /// past an end stays put).
    pub(super) fn step_sub_tab(&mut self, delta: isize) {
        let next = match (self.sub_tab(), delta.signum()) {
            (SubTab::General, 1) => SubTab::List,
            (SubTab::List, -1) => SubTab::General,
            (current, _) => current,
        };
        if next != self.sub_tab() {
            self.set_sub_tab(next);
        }
    }

    /// The key twins of the strip, anywhere on a tabbed page.
    pub(super) fn sub_tab_key(&mut self, key: &KeyEvent) -> bool {
        if !self.current_section().has_sub_tabs() {
            return false;
        }
        let km = crate::keymap::current();
        if km.is(Action::PreferencesNextTab, key) {
            self.step_sub_tab(1);
        } else if km.is(Action::PreferencesPrevTab, key) {
            self.step_sub_tab(-1);
        } else {
            return false;
        }
        true
    }

    /// A click on a tab of the strip. Whether it was one.
    pub(super) fn sub_tab_click(&mut self, col: u16, row: u16) -> bool {
        let hit = self
            .tab_rects
            .iter()
            .find(|(r, _)| r.contains((col, row).into()))
            .map(|(_, t)| *t);
        let Some(tab) = hit else {
            return false;
        };
        if tab != self.sub_tab() {
            self.set_sub_tab(tab);
        }
        true
    }

    /// The page's title row, the strip under it, and the room left for the
    /// page. The strip's rects come out of this very walk.
    pub(super) fn render_tab_head(&mut self, frame: &mut Frame, area: Rect) -> Rect {
        let section = self.current_section();
        let head = Rect::new(area.x, area.y, area.width, 2.min(area.height));
        frame.render_widget(Clear, head);
        let title = format!(
            " {} ",
            if section.is_repo_scoped() {
                screen::page_title_repo(section.title())
            } else {
                screen::page_title_global(section.title())
            }
        );
        frame.render_widget(
            Paragraph::new(title),
            Rect::new(area.x, area.y, area.width, 1),
        );

        self.tab_rects.clear();
        let buf = frame.buffer_mut();
        let front = self.sub_tab();
        let y = area.y + 1;
        let mut x = area.x + 1;
        let right = area.right().saturating_sub(1);
        // A page with a single face (Sequences) has nothing to switch.
        let labels: Vec<(SubTab, String)> = if section.has_sub_tabs() {
            vec![
                (SubTab::General, screen::SECTION_GENERAL.get().to_string()),
                (SubTab::List, section.list_tab_title()),
            ]
        } else {
            Vec::new()
        };
        for (tab, label) in labels {
            let text = format!(" {label} ");
            let w = display_width(&text) as u16;
            if x + w > right {
                break;
            }
            let rect = Rect::new(x, y, w, 1);
            let hovered = self
                .mouse_pos
                .is_some_and(|(c, r)| rect.contains((c, r).into()));
            let style = if tab == front {
                Style::default()
                    .fg(th().ui.on_accent)
                    .bg(th().ui.accent)
                    .add_modifier(Modifier::BOLD)
            } else if hovered {
                crate::theme::plain_button_style(true)
            } else {
                Style::default().fg(th().ui.text).bg(th().ui.panel)
            };
            dialog_kit::put(buf, x, y, &text, w, style);
            crate::pointer_map::hand(rect);
            self.tab_rects.push((rect, tab));
            x += w + 1;
        }
        // The keys, muted, after the tabs.
        let km = crate::keymap::current();
        let hint = format!(
            "{} / {}",
            km.label(Action::PreferencesPrevTab),
            km.label(Action::PreferencesNextTab)
        );
        if section.has_sub_tabs() && x + 1 + display_width(&hint) as u16 <= right {
            dialog_kit::put(
                buf,
                x + 1,
                y,
                &hint,
                right.saturating_sub(x + 1),
                Style::default().fg(th().ui.muted),
            );
        }
        Rect::new(
            area.x,
            area.y + 2,
            area.width,
            area.height.saturating_sub(2),
        )
    }
}
