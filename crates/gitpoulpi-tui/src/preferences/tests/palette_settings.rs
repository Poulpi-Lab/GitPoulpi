//! The Settings fields as palette rows: generated from the page model, applied
//! by the page's own code on a temp `config.toml`.

use super::*;
use crate::preferences::palette_settings::{Edit, SettingKind};

fn temp_state() -> (tempfile::TempDir, PreferencesState) {
    let dir = tempfile::tempdir().unwrap();
    let st = state(dir.path());
    (dir, st)
}

fn file(dir: &tempfile::TempDir) -> String {
    std::fs::read_to_string(dir.path().join("config.toml")).unwrap_or_default()
}

#[test]
fn the_rows_cover_toggles_choices_numbers_and_text_and_skip_headings() {
    let (_dir, st) = temp_state();
    let rows = st.setting_rows();
    let kind = |key: &str| rows.iter().find(|r| r.key == key).map(|r| r.kind.clone());
    assert_eq!(kind("ui.minimap"), Some(SettingKind::Toggle));
    assert!(matches!(
        kind("general.auto_stash"),
        Some(SettingKind::Choice(options)) if options.len() == 3
    ));
    assert!(matches!(
        kind("general.max_commits_in_graph"),
        Some(SettingKind::Number { min: 0, .. })
    ));
    assert_eq!(kind("general.default_branch_name"), Some(SettingKind::Text));
    // A number with a unit takes two parts: the palette opens its page.
    assert_eq!(
        kind("general.auto_fetch_interval_minutes"),
        Some(SettingKind::Page)
    );
    // Installed terminals are found at run time: the page lists them.
    assert_eq!(kind("ui.launch_terminal"), Some(SettingKind::Page));
    // One row per key, none for a heading or an info line.
    let mut keys: Vec<_> = rows.iter().map(|r| r.key).collect();
    keys.sort_unstable();
    let before = keys.len();
    keys.dedup();
    assert_eq!(before, keys.len());
    assert!(rows.iter().all(|r| !r.label.is_empty()));
}

/// Every plain row of the pages is reachable from the palette.
#[test]
fn every_global_row_of_the_pool_has_a_palette_row() {
    let (_dir, st) = temp_state();
    let rows = st.setting_rows();
    for field in st.row_pool() {
        if let Target::Global(key) = field.target {
            if matches!(
                field.control,
                Control::Heading | Control::Info(_) | Control::Swatches(_)
            ) {
                continue;
            }
            assert!(
                rows.iter().any(|r| r.key == key),
                "{key} has no palette row"
            );
        }
    }
}

#[test]
fn a_toggle_flips_and_persists_like_the_page() {
    let (dir, mut st) = temp_state();
    let before = st.setting_current("general.remember_tabs").unwrap();
    st.apply_setting("general.remember_tabs", Edit::Toggle)
        .unwrap();
    let after = st.setting_current("general.remember_tabs").unwrap();
    assert_ne!(before.shown, after.shown);
    assert!(file(&dir).contains("remember_tabs"), "{}", file(&dir));
    // And back.
    st.apply_setting("general.remember_tabs", Edit::Toggle)
        .unwrap();
    assert_eq!(
        st.setting_current("general.remember_tabs").unwrap().shown,
        before.shown
    );
}

#[test]
fn the_radar_toggle_applies_live_and_persists() {
    let (dir, mut st) = temp_state();
    if !crate::build_features::RADAR {
        return;
    }
    assert!(crate::build_features::radar_enabled());
    st.apply_setting("general.radar_enabled", Edit::Toggle)
        .unwrap();
    assert!(!crate::build_features::radar_enabled(), "no restart");
    assert!(
        file(&dir).contains("radar_enabled = false"),
        "{}",
        file(&dir)
    );
    st.apply_setting("general.radar_enabled", Edit::Toggle)
        .unwrap();
    assert!(crate::build_features::radar_enabled());
}

#[test]
fn a_choice_is_picked_by_index_and_marked_current() {
    let (dir, mut st) = temp_state();
    let SettingKind::Choice(options) = st
        .setting_rows()
        .into_iter()
        .find(|r| r.key == "general.auto_stash")
        .unwrap()
        .kind
    else {
        panic!("a choice");
    };
    let never = options.iter().position(|o| o == "never").unwrap();
    st.apply_setting("general.auto_stash", Edit::Choose(never))
        .unwrap();
    assert_eq!(
        st.setting_current("general.auto_stash").unwrap().choice,
        Some(never)
    );
    assert!(
        file(&dir).contains("auto_stash = \"never\""),
        "{}",
        file(&dir)
    );
}

#[test]
fn a_number_is_validated_like_the_page() {
    let (dir, mut st) = temp_state();
    st.apply_setting("general.max_commits_in_graph", Edit::Text("7".into()))
        .unwrap();
    assert!(file(&dir).contains("max_commits_in_graph = 7"));
    let outside = st
        .apply_setting(
            "general.max_commits_in_graph",
            Edit::Text("99999999".into()),
        )
        .unwrap_err();
    assert!(!outside.is_empty());
    let text = st
        .apply_setting("general.max_commits_in_graph", Edit::Text("soon".into()))
        .unwrap_err();
    assert!(!text.is_empty());
    // The refused values changed nothing.
    assert!(file(&dir).contains("max_commits_in_graph = 7"));
    assert_eq!(
        st.setting_current("general.max_commits_in_graph")
            .unwrap()
            .text,
        "7"
    );
}

#[test]
fn a_text_prefills_with_its_value_and_a_wrong_edit_kind_is_refused() {
    let (_dir, mut st) = temp_state();
    st.apply_setting("general.default_branch_name", Edit::Text("trunk".into()))
        .unwrap();
    assert_eq!(
        st.setting_current("general.default_branch_name")
            .unwrap()
            .text,
        "trunk"
    );
    // A toggle on a text row does nothing.
    assert!(st
        .apply_setting("general.default_branch_name", Edit::Toggle)
        .is_err());
    // An unknown key says so.
    assert!(st.apply_setting("no.such.key", Edit::Toggle).is_err());
}
