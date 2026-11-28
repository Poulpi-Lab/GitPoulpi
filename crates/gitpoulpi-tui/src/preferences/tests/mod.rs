use super::*;
use crate::live_settings::{
    active_profile_author, date_time_pattern, format_date, install_date_time_format,
    DATE_PATTERNS_TEST_LOCK,
};
use gitpoulpi_config::{Integration, IntegrationKind, TabDirFollow};
use gitpoulpi_secrets::InMemoryTokens;

mod buttons;
mod choice;
mod close_guard;
mod commit_pages;
mod conflict_pages;
mod fields;
mod git_terms_english;
mod i18n;
mod images;
mod input;
mod integrations_scroll;
mod launcher;
mod launcher_alias;
mod launcher_folder_menu;
mod launcher_install;
mod list_editor;
mod list_page;
mod live;
#[cfg(feature = "mcp")]
mod mcp_rows;
mod open_in;
mod pages;
mod palette_settings;
mod persist;
mod quantity;
mod quantity_rows;
pub(crate) mod render;
mod repo_pages;
mod schema_walk;
mod search;
mod sequences;
mod sequences_ai;
mod sub_tabs;
mod theme;
mod tokens;
mod updates_rows;
mod view_memory;
mod workflow;

/// A configured, unverified profile, the way the page leaves one:
/// no token yet, so the cache says `Some(false)` without a store call.
pub(super) fn add_profile(state: &mut PreferencesState, kind: IntegrationKind) -> String {
    let id = Integration::new_id(kind, state.config.integrations.iter());
    let integration = Integration::new(id.clone(), kind, "https://x.example.com");
    state
        .token_presence
        .insert(integration.token_ref.clone(), Some(false));
    state.config.integrations.push(integration);
    id
}

/// Shows `section`'s page.
pub(super) fn go(state: &mut PreferencesState, section: PrefSection) {
    state.section = PrefSection::ALL.iter().position(|s| *s == section).unwrap();
    // Most tests of a master-detail page drive the list and the selected
    // item's rows; the ones about its General tab say so with `go_tab`.
    state.remember_sub_tab(section, SubTab::List);
}

/// Shows `tab` of `section`'s page (a master-detail page).
pub(super) fn go_tab(state: &mut PreferencesState, section: PrefSection, tab: SubTab) {
    go(state, section);
    state.remember_sub_tab(section, tab);
    state.field = state.first_selectable();
}

pub(crate) fn state(dir: &Path) -> PreferencesState {
    let config_path = dir.join("config.toml");
    PreferencesState::with_parts(
        Config::default(),
        Some(config_path),
        dir,
        RepoState::default(),
        RepoState::path_for(dir, ".gitpoulpi"),
        std::sync::Arc::new(InMemoryTokens::default()),
    )
}

/// A state opened on the section whose `Debug` name is `section` (the
/// layout test of `i18n_layout_tests`).
pub(crate) fn state_at(dir: &Path, section: &str) -> PreferencesState {
    let mut st = state(dir);
    st.section = PrefSection::ALL
        .iter()
        .position(|s| format!("{s:?}") == section)
        .unwrap_or_else(|| panic!("no section {section}"));
    // The list tab of a master-detail page (see `go`).
    st.remember_sub_tab(PrefSection::ALL[st.section], SubTab::List);
    st
}

// -- Choice fields are dropdowns, never click-to-cycle -----------------

/// Draws `state` at 100x30 and returns the screen text.
pub(super) fn draw_screen(state: &mut PreferencesState) -> String {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let mut terminal = Terminal::new(TestBackend::new(100, 50)).unwrap();
    terminal
        .draw(|frame| state.render(frame, frame.area()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    (0..50)
        .map(|y| {
            (0..100)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The selected section row of the sidebar is shown by its highlight alone:
/// no arrow glyph.
#[test]
fn the_selected_sidebar_section_has_no_arrow_marker() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = state(tmp.path());
    let screen = draw_screen(&mut st);
    assert!(!screen.contains('\u{25B8}'), "{screen}");
}
mod cjk_dropdown;
