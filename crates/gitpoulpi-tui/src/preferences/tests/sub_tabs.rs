//! The sub-tabs of the master-detail pages: General (the page's own
//! settings) and the list tab (the list and the selected item's rows).

use super::*;
use crossterm::event::{KeyEvent, KeyModifiers, MouseButton, MouseEventKind};
use gitpoulpi_config::OpenInEntry;

const PAGES: [PrefSection; 5] = [
    PrefSection::Profiles,
    PrefSection::Ai,
    PrefSection::OpenIn,
    PrefSection::Images,
    PrefSection::Workflow,
];

fn draw(st: &mut PreferencesState) -> String {
    use ratatui::backend::TestBackend;
    let (w, h) = (130, 45);
    let mut terminal = ratatui::Terminal::new(TestBackend::new(w, h)).unwrap();
    terminal
        .draw(|frame| st.render(frame, frame.area()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    (0..h)
        .map(|y| (0..w).map(|x| buffer[(x, y)].symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

fn alt(st: &mut PreferencesState, code: KeyCode) {
    st.on_key(KeyEvent::new(code, KeyModifiers::ALT));
}

fn click(st: &mut PreferencesState, col: u16, row: u16) {
    st.on_mouse(MouseEventKind::Down(MouseButton::Left), col, row);
}

/// A state on `section` with one profile / entry of each kind to select.
fn page(dir: &Path, section: PrefSection) -> PreferencesState {
    let mut st = state(dir);
    st.config.ai.enabled = true;
    st.config.profiles.push(Profile::new("a", "Ann"));
    st.config.ui.open_in = Some(vec![OpenInEntry::new("A", "a ${cwd}")]);
    st.config.ui.image_editors = Some(vec![OpenInEntry::new("A", "a ${file}")]);
    st.open_in_detected
        .set(gitpoulpi_config::open_in::Detected::default())
        .unwrap();
    st.image_detected
        .set(gitpoulpi_config::image_editors::Detected::default())
        .unwrap();
    st.section = PrefSection::ALL.iter().position(|s| *s == section).unwrap();
    st.focus = PrefFocus::Fields;
    st
}

/// A row that only the page as a whole owns, and one only an item owns.
#[cfg(feature = "ai")]
fn probes(section: PrefSection) -> (&'static str, &'static str) {
    match section {
        PrefSection::Profiles => ("Active profile", "Profile ID"),
        PrefSection::Ai => ("Enable AI", "Provider"),
        PrefSection::OpenIn => ("Default entry", "Command"),
        PrefSection::Images => ("Default entry", "Command"),
        _ => ("Template", "No rules yet"),
    }
}

#[cfg(feature = "ai")]
fn has_row(st: &PreferencesState, label: &str) -> bool {
    st.fields().iter().any(|f| f.label == label)
}

#[test]
#[cfg(feature = "ai")]
fn a_page_opens_on_general_and_each_tab_holds_only_its_own_rows() {
    for section in PAGES {
        let dir = tempfile::tempdir().unwrap();
        let mut st = page(dir.path(), section);
        let (global, item) = probes(section);
        assert_eq!(
            st.sub_tab(),
            SubTab::General,
            "{section:?} opens on General"
        );
        assert!(has_row(&st, global), "{section:?}: {global} on General");
        assert!(!has_row(&st, item), "{section:?}: {item} not on General");
        st.set_sub_tab(SubTab::List);
        assert!(has_row(&st, item), "{section:?}: {item} on the list tab");
        assert!(
            !has_row(&st, global),
            "{section:?}: {global} not on the list"
        );
    }
}

#[test]
fn the_strip_names_the_tabs_and_the_list_is_only_on_the_list_tab() {
    for section in PAGES {
        let dir = tempfile::tempdir().unwrap();
        let mut st = page(dir.path(), section);
        let screen = draw(&mut st);
        assert!(screen.contains(" General "), "{section:?}\n{screen}");
        assert_eq!(st.tab_rects.len(), 2, "{section:?}");
        assert_eq!(st.list_area, Rect::default(), "no list on General");
        st.set_sub_tab(SubTab::List);
        draw(&mut st);
        assert!(st.list_area.width > 0, "{section:?}: the list is drawn");
    }
}

#[test]
fn a_click_on_a_tab_switches_and_the_keys_are_its_twins() {
    for section in PAGES {
        let dir = tempfile::tempdir().unwrap();
        let mut st = page(dir.path(), section);
        draw(&mut st);
        let (_, list_tab) = st.tab_rects[1];
        assert_eq!(list_tab, SubTab::List);
        let rect = st.tab_rects[1].0;
        click(&mut st, rect.x + 1, rect.y);
        assert_eq!(st.sub_tab(), SubTab::List, "{section:?}: click");
        assert_eq!(st.focus, PrefFocus::List, "the list takes the keyboard");
        draw(&mut st);
        let rect = st.tab_rects[0].0;
        click(&mut st, rect.x + 1, rect.y);
        assert_eq!(st.sub_tab(), SubTab::General, "{section:?}: click back");
        assert_eq!(st.focus, PrefFocus::Fields);
        alt(&mut st, KeyCode::Right);
        assert_eq!(st.sub_tab(), SubTab::List, "{section:?}: Alt+Right");
        alt(&mut st, KeyCode::Right);
        assert_eq!(st.sub_tab(), SubTab::List, "an end stays put");
        alt(&mut st, KeyCode::Left);
        assert_eq!(st.sub_tab(), SubTab::General, "{section:?}: Alt+Left");
    }
}

#[test]
#[cfg(feature = "ai")]
fn the_search_and_the_go_to_setting_links_land_on_the_right_tab() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path(), PrefSection::OpenIn);
    // A row of the list tab, then one of General, from the search box.
    st.set_sub_tab(SubTab::List);
    let hits =
        |st: &PreferencesState, q: &str| super::super::search::matches(q, &st.search_entries());
    let hit = hits(&st, "Default entry").remove(0);
    assert_eq!(hit.tab, SubTab::General);
    st.set_sub_tab(SubTab::List);
    st.jump_to(&hit);
    assert_eq!(st.sub_tab(), SubTab::General, "the search lands on General");
    assert_eq!(st.fields()[st.field].label, "Default entry");

    st.set_sub_tab(SubTab::General);
    let hit = hits(&st, "Insert variable").remove(0);
    assert_eq!(hit.tab, SubTab::List);
    st.jump_to(&hit);
    assert_eq!(
        st.sub_tab(),
        SubTab::List,
        "the search lands on the list tab"
    );
    assert_eq!(st.fields()[st.field].label, "Insert variable");

    // A key link (Radar, the palette's Settings rows, agents).
    st.show_page(None, Some("ui.open_in_default")).unwrap();
    assert_eq!(st.sub_tab(), SubTab::General);
    for (key, page) in [
        ("general.active_profile", PrefSection::Profiles),
        ("ui.image_editor_default", PrefSection::Images),
        ("ai.enabled", PrefSection::Ai),
    ] {
        st.set_sub_tab(SubTab::List);
        st.show_page(None, Some(key)).unwrap();
        assert_eq!(st.current_section(), page, "{key}");
        assert_eq!(st.sub_tab(), SubTab::General, "{key}");
    }
}

#[test]
fn the_tab_in_front_is_remembered_per_page() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path(), PrefSection::Profiles);
    st.set_sub_tab(SubTab::List);
    st.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::Images)
        .unwrap();
    assert_eq!(st.sub_tab(), SubTab::General, "another page is untouched");
    st.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::Profiles)
        .unwrap();
    assert_eq!(st.sub_tab(), SubTab::List, "back on the page, same tab");
}

#[test]
fn tab_does_not_visit_the_list_while_general_is_in_front() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path(), PrefSection::Profiles);
    st.focus = PrefFocus::Sections;
    st.on_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(st.focus, PrefFocus::Fields, "no list to stop at");
    st.on_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(st.focus, PrefFocus::Sections);
}
