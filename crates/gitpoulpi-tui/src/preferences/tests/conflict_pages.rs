//! The Conflict prevention page of the repository group and the global switch
//! on General: their rows, what they save.

use super::*;
use crate::preferences::conflict_pages::SourceChoice;
use crate::preferences::repo_pages::{RpButton, RpTarget};
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

fn select_exact(st: &mut PreferencesState, label: &str) {
    st.focus = PrefFocus::Fields;
    st.field = st.fields().iter().position(|f| f.label == label).unwrap();
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
    // Followed from the default: read-only, no editor.
    assert!(matches!(
        st.fields()
            .iter()
            .find(|f| f.label == "Base branches")
            .unwrap()
            .control,
        Control::Info(_)
    ));
    st.commit_choice(&Target::RepoPage(RpTarget::BaseSource), 1);
    assert_eq!(area(&st), "**");

    select_exact(&mut st, "Base branches");
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

    // Emptied: stored as "not set", the Own list then reads what is in force.
    select_exact(&mut st, "Base branches");
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
    let patterns: Vec<&str> = infos.iter().skip(1).map(|(p, _)| *p).collect();
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

// -- named lists, the default list and the repository's source ---------------

fn type_text(st: &mut PreferencesState, row: &str, text: &str) {
    select(st, row);
    st.activate();
    for c in text.chars() {
        key(st, KeyCode::Char(c), KeyModifiers::NONE);
    }
    key(st, KeyCode::Enter, KeyModifiers::NONE);
}

fn config_on_disk(dir: &Path) -> gitpoulpi_config::Config {
    gitpoulpi_config::Config::load(&dir.join("config.toml")).unwrap()
}

fn base_row(st: &PreferencesState) -> Control {
    st.fields()
        .into_iter()
        .find(|f| f.label == "Base branches")
        .unwrap()
        .control
}

fn press(st: &mut PreferencesState, label: &str) {
    let button = st
        .fields()
        .into_iter()
        .find_map(|f| match f.control {
            Control::Buttons(b) => b.into_iter().find(|b| b.label == label),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no button {label}"));
    st.press_button(&button);
}

fn confirm_yes(st: &mut PreferencesState) {
    assert!(
        st.confirm_action.is_some(),
        "a destructive button asks first"
    );
    key(st, KeyCode::Tab, KeyModifiers::NONE);
    key(st, KeyCode::Enter, KeyModifiers::NONE);
    assert!(st.confirm_action.is_none());
}

#[test]
fn a_list_is_created_named_default_and_followed_by_a_repository_without_its_own() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "General");
    type_text(&mut st, "New list", "trunk");
    let cfg = config_on_disk(dir.path());
    assert_eq!(cfg.conflict_prevention.lists["trunk"].patterns, ["**"]);
    assert_eq!(cfg.conflict_prevention.default_list, "");

    // Edit its patterns in the multi-line editor.
    st.field = st
        .fields()
        .iter()
        .position(|f| f.label == "Patterns")
        .unwrap();
    st.focus = PrefFocus::Fields;
    st.activate();
    key(&mut st, KeyCode::Backspace, KeyModifiers::NONE);
    key(&mut st, KeyCode::Backspace, KeyModifiers::NONE);
    for c in "origin/main".chars() {
        key(&mut st, KeyCode::Char(c), KeyModifiers::NONE);
    }
    key(&mut st, KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert_eq!(
        config_on_disk(dir.path()).conflict_prevention.lists["trunk"].patterns,
        ["origin/main"]
    );

    // Make it the default.
    press(&mut st, "Make default");
    assert_eq!(
        config_on_disk(dir.path()).conflict_prevention.default_list,
        "trunk"
    );

    // A repository that chose nothing follows it, shown on its page.
    let mut page = state_at(dir.path(), "ConflictPrevention");
    page.config = config_on_disk(dir.path());
    match base_row(&page) {
        Control::Info(text) => assert_eq!(text, "origin/main"),
        other => panic!("{other:?}"),
    }
    let fields = page.fields();
    let Control::Choice { options, index } = &fields[1].control else {
        panic!("the source is a choice")
    };
    assert_eq!(options[0], "Default (list trunk)");
    assert_eq!(*index, 0);
}

#[test]
fn a_repository_follows_a_list_then_resets_to_the_default() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "General");
    type_text(&mut st, "New list", "release");
    go(&mut st, PrefSection::ConflictPrevention);
    // The choice is: Default, release, Own list.
    st.commit_choice(&Target::RepoPage(RpTarget::BaseSource), 1);
    let prefs = saved(dir.path());
    assert_eq!(prefs.conflict_base_source.as_deref(), Some("list"));
    assert_eq!(prefs.conflict_base_list.as_deref(), Some("release"));
    assert_eq!(st.source_choice(), SourceChoice::List("release".into()));

    // Editing the list changes what the repository uses: it is shared.
    st.set_conflict_list_patterns(0, "origin/release/**");
    match base_row(&st) {
        Control::Info(text) => assert_eq!(text, "origin/release/**"),
        other => panic!("{other:?}"),
    }

    press(&mut st, "Reset to default");
    confirm_yes(&mut st);
    let prefs = saved(dir.path());
    assert_eq!(prefs.conflict_base_source, None);
    assert_eq!(prefs.conflict_base_list, None);
    assert_eq!(prefs.conflict_base_branches, None);
    assert_eq!(st.source_choice(), SourceChoice::Default);
}

#[test]
fn save_as_list_turns_the_patterns_into_a_followed_list() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "ConflictPrevention");
    st.commit_choice(&Target::RepoPage(RpTarget::BaseSource), 1);
    st.set_base_branches("origin/**\n!origin/abc");
    type_text(&mut st, "Save as list", "mine");
    let cfg = config_on_disk(dir.path());
    assert_eq!(
        cfg.conflict_prevention.lists["mine"].patterns,
        ["origin/**", "!origin/abc"]
    );
    let prefs = saved(dir.path());
    assert_eq!(prefs.conflict_base_source.as_deref(), Some("list"));
    assert_eq!(prefs.conflict_base_list.as_deref(), Some("mine"));
    // A taken name is refused and the field stays open.
    assert!(!st.conflict_commit_text(RpTarget::SaveAsList, "mine"));
}

#[test]
fn an_older_own_list_is_still_the_own_list() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "ConflictPrevention");
    st.repo_state.preferences.conflict_base_branches = Some("origin/**".into());
    assert_eq!(st.source_choice(), SourceChoice::Own);
    assert!(matches!(base_row(&st), Control::Area(t) if t == "origin/**"));
}

#[test]
fn deleting_a_list_asks_names_it_and_clears_the_default() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "General");
    type_text(&mut st, "New list", "old");
    press(&mut st, "Make default");
    let asked = st
        .conflict_confirm_text(PrefAction::RepoPage(RpButton::ConflictListDelete(0)))
        .unwrap();
    assert!(asked.contains("old"), "{asked}");
    press(&mut st, "Delete list");
    confirm_yes(&mut st);
    let cfg = config_on_disk(dir.path());
    assert!(cfg.conflict_prevention.lists.is_empty());
    assert_eq!(cfg.conflict_prevention.default_list, "");
}

#[test]
fn renaming_a_list_keeps_it_the_default_and_the_name_must_be_free() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "General");
    type_text(&mut st, "New list", "a");
    type_text(&mut st, "New list", "b");
    st.commit_choice(&Target::RepoPage(RpTarget::ConflictDefaultList), 1);
    assert!(!st.conflict_commit_text(RpTarget::ConflictListName(0), "b"));
    assert!(st.conflict_commit_text(RpTarget::ConflictListName(0), "z"));
    let cfg = config_on_disk(dir.path());
    let names: Vec<_> = cfg.conflict_prevention.lists.keys().cloned().collect();
    assert_eq!(names, ["b", "z"]);
    assert_eq!(cfg.conflict_prevention.default_list, "z");
    assert!(!st.conflict_commit_text(RpTarget::ConflictListNew, "  "));
}

#[test]
fn reset_to_the_built_in_default_clears_the_default_list() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "General");
    type_text(&mut st, "New list", "x");
    press(&mut st, "Make default");
    assert_eq!(
        config_on_disk(dir.path()).conflict_prevention.default_list,
        "x"
    );
    press(&mut st, "Reset to built-in default");
    assert_eq!(
        config_on_disk(dir.path()).conflict_prevention.default_list,
        ""
    );
}

#[test]
fn the_repository_page_draws_the_source_choice_and_the_list_buttons() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "ConflictPrevention");
    let screen = crate::preferences::tests::draw_screen(&mut st);
    for want in [
        "Base branches from",
        "Default (every branch)",
        "Save as list...",
        "Reset to default",
    ] {
        assert!(screen.contains(want), "{want} missing:\n{screen}");
    }
}

#[test]
fn the_general_page_groups_the_lists_under_the_global_switch() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "General");
    type_text(&mut st, "New list", "trunk");
    let labels: Vec<String> = st.fields().iter().map(|f| f.label.clone()).collect();
    let at = |l: &str| labels.iter().position(|x| x == l).unwrap();
    assert!(at("Conflict prevention") < at("Enable proactive conflict detection"));
    assert!(at("Enable proactive conflict detection") < at("Default base branches"));
    assert!(at("Default base branches") < at("Base branch lists"));
    assert!(at("Base branch lists") < at("trunk"));
    assert!(at("trunk") < at("New list"));
    // The undo group still follows the whole block.
    assert!(at("New list") < at(gitpoulpi_i18n::msg::settings_general::HEADING_UNDO.get()));
}
