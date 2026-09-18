//! The Profiles page on the master-detail layout (`list_page.rs`).

use super::*;
use crossterm::event::{KeyEvent, KeyModifiers};

fn press(st: &mut PreferencesState, code: KeyCode, modifiers: KeyModifiers) {
    st.on_key(KeyEvent::new(code, modifiers));
}

fn key(st: &mut PreferencesState, code: KeyCode) {
    press(st, code, KeyModifiers::NONE);
}

fn profiles_page(dir: &Path) -> PreferencesState {
    let mut st = state(dir);
    go(&mut st, PrefSection::Profiles);
    st.focus = PrefFocus::List;
    st
}

fn draw(st: &mut PreferencesState, w: u16, h: u16) -> String {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
    terminal
        .draw(|frame| st.render(frame, frame.area()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    (0..h)
        .map(|y| (0..w).map(|x| buffer[(x, y)].symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

fn ids(st: &PreferencesState) -> Vec<String> {
    st.config.profiles.iter().map(|p| p.id.clone()).collect()
}

fn saved_ids(dir: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(dir.join("config.toml")).unwrap_or_default();
    let doc: toml::Value = toml::from_str(&text).unwrap();
    doc.get("profiles")
        .and_then(|p| p.as_array())
        .map(|a| {
            a.iter()
                .map(|p| p["id"].as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn add_duplicate_reorder_and_remove_write_the_config_each_time() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = profiles_page(dir.path());
    key(&mut st, KeyCode::Insert);
    key(&mut st, KeyCode::Insert);
    assert_eq!(ids(&st), ["profile", "profile-2"]);
    assert_eq!(saved_ids(dir.path()), ["profile", "profile-2"]);
    assert_eq!(st.profile_index, 1);

    // Alt+Up moves the selected one first, and the file follows.
    press(&mut st, KeyCode::Up, KeyModifiers::ALT);
    assert_eq!(ids(&st), ["profile-2", "profile"]);
    assert_eq!(saved_ids(dir.path()), ["profile-2", "profile"]);
    assert_eq!(st.profile_index, 0);

    // Ctrl+D copies it right below with a fresh id.
    press(&mut st, KeyCode::Char('d'), KeyModifiers::CONTROL);
    assert_eq!(ids(&st), ["profile-2", "profile-2-2", "profile"]);
    assert_eq!(st.profile_index, 1);
    assert!(st.config.profiles[1].name.contains("(copy)"));

    // A profile somebody edited asks before it goes; Esc keeps it.
    st.config.profiles[1].author_email = "a@example.com".into();
    key(&mut st, KeyCode::Delete);
    key(&mut st, KeyCode::Esc);
    assert_eq!(ids(&st).len(), 3);
    key(&mut st, KeyCode::Delete);
    key(&mut st, KeyCode::Right);
    key(&mut st, KeyCode::Enter);
    assert_eq!(ids(&st), ["profile-2", "profile"]);
    assert_eq!(saved_ids(dir.path()), ["profile-2", "profile"]);
}

#[test]
fn removing_the_active_profile_clears_the_active_choice() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = profiles_page(dir.path());
    key(&mut st, KeyCode::Insert);
    st.config.profiles[0].author_email = "a@example.com".into();
    st.write_global("general.active_profile", "profile");
    key(&mut st, KeyCode::Delete);
    key(&mut st, KeyCode::Right);
    key(&mut st, KeyCode::Enter);
    assert!(st.config.profiles.is_empty());
    assert_eq!(st.config.general.active_profile, "");
}

#[test]
fn tab_cycles_sections_list_fields_and_left_goes_back() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    go(&mut st, PrefSection::Profiles);
    st.focus = PrefFocus::Sections;
    key(&mut st, KeyCode::Tab);
    assert_eq!(st.focus, PrefFocus::List);
    key(&mut st, KeyCode::Tab);
    assert_eq!(st.focus, PrefFocus::Fields);
    key(&mut st, KeyCode::Left);
    assert_eq!(st.focus, PrefFocus::List);
    key(&mut st, KeyCode::Left);
    assert_eq!(st.focus, PrefFocus::Sections);
    // Esc still closes Settings from the list.
    st.focus = PrefFocus::List;
    assert!(!st.on_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
}

#[test]
fn enter_on_the_list_lands_on_the_first_profile_field() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = profiles_page(dir.path());
    key(&mut st, KeyCode::Insert);
    key(&mut st, KeyCode::Enter);
    assert_eq!(st.focus, PrefFocus::Fields);
    let fields = st.fields();
    assert!(matches!(
        fields[st.field].target,
        Target::ProfileField(0, TextSlot::Id)
    ));
}

#[test]
fn the_page_draws_the_list_beside_the_detail_at_120x40() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = profiles_page(dir.path());
    key(&mut st, KeyCode::Insert);
    st.config.profiles[0].author_email = "jo@example.com".into();
    st.config.profiles[0].name = "Work".into();
    key(&mut st, KeyCode::Insert);
    let screen = draw(&mut st, 120, 40);
    assert!(screen.contains("Work"), "{screen}");
    assert!(screen.contains("jo@example"), "{screen}");
    // The profile with no email says so.
    assert!(screen.contains('\u{26A0}'), "{screen}");
    assert!(screen.contains("No email"), "{screen}");
    assert!(screen.contains("+ Add"), "{screen}");
    // The page's own setting is on the other tab.
    assert!(!screen.contains("Active profile"), "{screen}");
    assert!(screen.contains(" General "), "{screen}");
    // Side by side: the list is narrow and the detail starts to its right.
    assert!(st.list_area.width >= 26 && st.list_area.width <= 36);
    let id_line = screen.lines().find(|l| l.contains("Profile ID")).unwrap();
    let x = id_line.find("Profile ID").unwrap();
    assert!(x > st.list_area.right() as usize, "{screen}");
}

#[test]
fn the_page_stacks_the_list_above_the_detail_at_80x24() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = profiles_page(dir.path());
    key(&mut st, KeyCode::Insert);
    let screen = draw(&mut st, 80, 24);
    assert!(screen.contains("+ Add"), "{screen}");
    assert!(screen.contains("Profile ID"), "{screen}");
    // Stacked: the list spans the body, and the detail starts below it.
    assert!(st.list_area.width >= 40, "{:?}", st.list_area);
    let list_row = screen.lines().position(|l| l.contains("+ Add")).unwrap();
    let id_row = screen
        .lines()
        .position(|l| l.contains("Profile ID"))
        .unwrap();
    assert!(list_row < id_row, "{screen}");
}

#[test]
fn a_drag_on_the_list_reorders_the_profiles() {
    use crossterm::event::MouseButton;
    let dir = tempfile::tempdir().unwrap();
    let mut st = profiles_page(dir.path());
    for _ in 0..3 {
        key(&mut st, KeyCode::Insert);
    }
    draw(&mut st, 120, 40);
    let rows: Vec<Rect> = (0..3)
        .map(|i| {
            st.list
                .hits()
                .iter()
                .find(|(_, h)| *h == crate::preferences::list_editor::ListHit::Row(i))
                .unwrap()
                .0
        })
        .collect();
    let (x, y0, y2) = (rows[0].x + 2, rows[0].y, rows[2].y);
    st.on_mouse(MouseEventKind::Down(MouseButton::Left), x, y0);
    st.on_mouse(MouseEventKind::Drag(MouseButton::Left), x, y2);
    st.on_mouse(MouseEventKind::Up(MouseButton::Left), x, y2);
    assert_eq!(ids(&st), ["profile-2", "profile-3", "profile"]);
    assert_eq!(saved_ids(dir.path()), ["profile-2", "profile-3", "profile"]);
}

/// The wheel stops at the last full page, never on a blank screen.
#[test]
fn the_wheel_cannot_scroll_past_the_last_page() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    go(&mut st, PrefSection::General);
    st.focus = PrefFocus::Fields;
    draw(&mut st, 100, 20);
    for _ in 0..200 {
        st.on_mouse(MouseEventKind::ScrollDown, 60, 10);
    }
    draw(&mut st, 100, 20);
    assert!(st.body_lines > st.body_height, "the page must overflow");
    assert_eq!(
        st.scroll,
        st.body_lines - st.body_height,
        "stops with the last line on the last row"
    );
}
