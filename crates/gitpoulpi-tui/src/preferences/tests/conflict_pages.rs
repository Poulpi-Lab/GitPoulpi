//! The Conflict prevention page of the repository group and the global switch
//! on General: their rows, what they save.

use super::*;
use crossterm::event::{KeyEvent, KeyModifiers};

fn saved(dir: &Path) -> gitpoulpi_session::RepoPreferences {
    RepoState::load(&RepoState::path_for(dir, ".gitpoulpi"))
        .unwrap()
        .preferences
}

fn select(st: &mut PreferencesState, label_start: &str) {
    st.focus = PrefFocus::Fields;
    st.field = st
        .fields()
        .iter()
        .position(|f| f.label.starts_with(label_start))
        .unwrap_or_else(|| {
            panic!(
                "no `{label_start}` in {:?}",
                st.fields().iter().map(|f| &f.label).collect::<Vec<_>>()
            )
        });
}

fn key(st: &mut PreferencesState, code: KeyCode, mods: KeyModifiers) {
    st.on_key(KeyEvent::new(code, mods));
}

#[test]
fn the_page_is_in_the_repository_group_with_a_stable_id() {
    let section = PrefSection::ConflictPrevention;
    assert!(section.is_repo_scoped());
    assert_eq!(section.id(), "conflict-prevention");
    let page = gitpoulpi_ipc::ui::SettingsPage::parse("conflict-prevention").unwrap();
    assert_eq!(PrefSection::from_page(page), section);
    assert!(section.is_built());
    assert_eq!(section.title(), "Conflict prevention");
}

#[test]
fn the_switch_starts_on_and_is_saved_in_the_repo_data() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "ConflictPrevention");
    let fields = st.fields();
    let switch = &fields[0];
    assert_eq!(switch.label, "Check for conflicts with the base branch");
    assert!(
        matches!(switch.control, Control::Toggle(true)),
        "on by default"
    );
    let help = switch.help.clone().unwrap();
    assert!(
        help.contains("clear the box to skip the check here")
            && help.contains("Settings > General"),
        "{help}"
    );
    assert_eq!(saved(dir.path()).conflict_detection, None);

    select(&mut st, "Check for conflicts");
    st.activate();
    assert_eq!(saved(dir.path()).conflict_detection, Some(false));
    st.activate();
    assert_eq!(saved(dir.path()).conflict_detection, Some(true));
}

#[test]
fn base_branches_default_to_every_branch_and_the_edit_is_saved() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "ConflictPrevention");
    let area = |st: &PreferencesState| match &st
        .fields()
        .iter()
        .find(|f| f.label == "Base branches")
        .unwrap()
        .control
    {
        Control::Area(text) => text.clone(),
        other => panic!("{other:?}"),
    };
    assert_eq!(area(&st), "**");

    select(&mut st, "Base branches");
    st.activate();
    // The editor starts on the current text, the cursor at its end.
    key(&mut st, KeyCode::Backspace, KeyModifiers::NONE);
    key(&mut st, KeyCode::Backspace, KeyModifiers::NONE);
    for c in "origin/**".chars() {
        key(&mut st, KeyCode::Char(c), KeyModifiers::NONE);
    }
    key(&mut st, KeyCode::Enter, KeyModifiers::NONE);
    for c in "!origin/abc".chars() {
        key(&mut st, KeyCode::Char(c), KeyModifiers::NONE);
    }
    key(&mut st, KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert_eq!(
        saved(dir.path()).conflict_base_branches.as_deref(),
        Some("origin/**\n!origin/abc")
    );
    assert_eq!(area(&st), "origin/**\n!origin/abc");

    // Back to the default: stored as "not set", so it follows the default.
    select(&mut st, "Base branches");
    st.activate();
    for _ in 0..30 {
        key(&mut st, KeyCode::Backspace, KeyModifiers::NONE);
    }
    key(&mut st, KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert_eq!(saved(dir.path()).conflict_base_branches, None);
    assert_eq!(area(&st), "**");
}

#[test]
fn the_page_carries_the_four_row_pattern_table() {
    let dir = tempfile::tempdir().unwrap();
    let st = state_at(dir.path(), "ConflictPrevention");
    let fields = st.fields();
    let infos: Vec<(&str, String)> = fields
        .iter()
        .filter_map(|f| match &f.control {
            Control::Info(text) if !text.is_empty() => Some((f.label.as_str(), text.clone())),
            _ => None,
        })
        .collect();
    let patterns: Vec<&str> = infos.iter().map(|(p, _)| *p).collect();
    assert_eq!(
        patterns,
        [
            "origin/main",
            "origin/**",
            "origin/feature/**",
            "!origin/abc"
        ]
    );
    assert!(fields
        .iter()
        .any(|f| f.label.contains("warns you before you")));
}

#[test]
fn the_global_switch_is_on_the_general_page_and_on_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let st = state_at(dir.path(), "General");
    let fields = st.fields();
    let row = fields
        .iter()
        .find(|f| f.label == "Enable proactive conflict detection")
        .expect("the General page has the global switch");
    assert!(matches!(row.control, Control::Toggle(true)));
    assert!(
        gitpoulpi_config::Config::default()
            .general
            .conflict_detection
    );
}
