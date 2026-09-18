//! The other number-and-unit rows (the two minute timers, the log file size,
//! the image size limit, the two history rewrite script timeouts): an older
//! value reads back in the largest even unit, typing a number writes the
//! total in the key's own base unit plus the unit key, a refused number
//! writes nothing.

use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn press(st: &mut PreferencesState, code: KeyCode) {
    st.on_key(KeyEvent::new(code, KeyModifiers::NONE));
}

/// Selects the row of `key` on `section`.
fn on_row(st: &mut PreferencesState, section: PrefSection, key: &'static str) -> usize {
    go(st, section);
    st.focus = PrefFocus::Fields;
    // The Integrations page keeps its generic rows in a strip above its body.
    st.strip = section == PrefSection::Integrations;
    // A master-detail page keeps its own settings on the General tab.
    let on_page =
        |st: &PreferencesState| st.fields().iter().any(|f| f.target == Target::Global(key));
    if !on_page(st) && section.has_sub_tabs() {
        st.remember_sub_tab(section, SubTab::General);
    }
    st.field = st
        .fields()
        .iter()
        .position(|f| f.target == Target::Global(key))
        .unwrap_or_else(|| panic!("no row {key}"));
    st.field
}

fn stored(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("config.toml")).unwrap_or_default()
}

fn shown(st: &PreferencesState) -> String {
    st.fields()[st.field].value_text()
}

/// Types `text` into the selected row's number and presses Enter.
fn type_number(st: &mut PreferencesState, text: &str) {
    press(st, KeyCode::Enter);
    for _ in 0..8 {
        press(st, KeyCode::Backspace);
    }
    for c in text.chars() {
        press(st, KeyCode::Char(c));
    }
    press(st, KeyCode::Enter);
}

/// Picks unit number `pick` of the selected row from its list.
fn pick_unit(st: &mut PreferencesState, pick: usize) {
    let index = st.field;
    st.commit_quantity_unit(index, pick);
}

#[test]
fn the_auto_fetch_interval_is_minutes_or_hours_and_zero_turns_it_off() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    // An old config: 60 minutes reads back as one hour, 45 stays minutes.
    for (minutes, text) in [(60, "1 hour"), (45, "45 minutes"), (5, "5 minutes")] {
        st.config.general.auto_fetch_interval_minutes = minutes;
        on_row(
            &mut st,
            PrefSection::Sync,
            "general.auto_fetch_interval_minutes",
        );
        assert_eq!(shown(&st), text, "{minutes} minutes");
    }
    // 0 is a valid number (background fetching off) and keeps the minutes key.
    type_number(&mut st, "0");
    let file = stored(dir.path());
    assert!(file.contains("auto_fetch_interval_minutes = 0"), "{file}");
    assert!(
        file.contains("auto_fetch_interval_unit = \"minutes\""),
        "{file}"
    );
    type_number(&mut st, "30");
    assert!(stored(dir.path()).contains("auto_fetch_interval_minutes = 30"));
    // Hours: the number stays, cut to the most the unit allows (60 minutes).
    pick_unit(&mut st, 1);
    let file = stored(dir.path());
    assert!(file.contains("auto_fetch_interval_minutes = 60"), "{file}");
    assert!(
        file.contains("auto_fetch_interval_unit = \"hours\""),
        "{file}"
    );
    assert_eq!(shown(&st), "1 hour");
    // Too many: refused, nothing written.
    pick_unit(&mut st, 0);
    type_number(&mut st, "61");
    assert!(st.edit.is_some() && st.edit_invalid.is_some());
    // The unit switch kept the number (1), the refused 61 wrote nothing.
    assert!(stored(dir.path()).contains("auto_fetch_interval_minutes = 1\n"));
}

#[test]
fn the_hosting_refresh_is_minutes_or_hours_up_to_two_hours() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    st.config.general.hosting_refresh_minutes = 120;
    on_row(
        &mut st,
        PrefSection::Integrations,
        "general.hosting_refresh_minutes",
    );
    assert_eq!(shown(&st), "2 hours");
    pick_unit(&mut st, 0);
    type_number(&mut st, "90");
    let file = stored(dir.path());
    assert!(file.contains("hosting_refresh_minutes = 90"), "{file}");
    assert!(
        file.contains("hosting_refresh_unit = \"minutes\""),
        "{file}"
    );
    assert_eq!(shown(&st), "90 minutes");
    // Hours: 90 is cut to the 2 that fit, stored as 120 minutes.
    pick_unit(&mut st, 1);
    let file = stored(dir.path());
    assert!(file.contains("hosting_refresh_minutes = 120"), "{file}");
    assert!(file.contains("hosting_refresh_unit = \"hours\""), "{file}");
    pick_unit(&mut st, 0);
    type_number(&mut st, "121");
    assert!(st.edit.is_some(), "above two hours is refused");
    assert!(stored(dir.path()).contains("hosting_refresh_minutes = 2\n"));
}

#[test]
fn the_log_file_size_is_mib_or_gib() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    on_row(&mut st, PrefSection::General, "log.max_file_mb");
    assert_eq!(shown(&st), "5 MiB");
    st.config.log.max_file_mb = 1024;
    on_row(&mut st, PrefSection::General, "log.max_file_mb");
    assert_eq!(shown(&st), "1 GiB");
    // One GiB is stored as 1024 MiB: the key keeps its base unit.
    pick_unit(&mut st, 0);
    type_number(&mut st, "256");
    let file = stored(dir.path());
    assert!(file.contains("max_file_mb = 256"), "{file}");
    assert!(file.contains("max_file_unit = \"mib\""), "{file}");
    pick_unit(&mut st, 1);
    let file = stored(dir.path());
    assert!(file.contains("max_file_mb = 1024"), "{file}");
    assert!(file.contains("max_file_unit = \"gib\""), "{file}");
    type_number(&mut st, "2");
    assert!(st.edit.is_some(), "2 GiB is above the 1024 MiB cap");
    assert!(stored(dir.path()).contains("max_file_mb = 1024"));
}

#[test]
#[cfg(feature = "image-view")]
fn the_image_size_limit_is_mib_or_gib() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    on_row(&mut st, PrefSection::Images, "ui.image_view_max_mib");
    assert_eq!(shown(&st), "32 MiB");
    type_number(&mut st, "64");
    let file = stored(dir.path());
    assert!(file.contains("image_view_max_mib = 64"), "{file}");
    assert!(file.contains("image_view_max_unit = \"mib\""), "{file}");
    pick_unit(&mut st, 1);
    let file = stored(dir.path());
    assert!(file.contains("image_view_max_mib = 1024"), "{file}");
    assert!(file.contains("image_view_max_unit = \"gib\""), "{file}");
    assert_eq!(shown(&st), "1 GiB");
}

#[test]
fn the_script_timeout_is_milliseconds_or_seconds() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    on_row(&mut st, PrefSection::Sync, "rewrite.script_timeout_ms");
    assert_eq!(shown(&st), "5 seconds");
    st.config.rewrite.script_timeout_ms = 1500;
    on_row(&mut st, PrefSection::Sync, "rewrite.script_timeout_ms");
    assert_eq!(shown(&st), "1500 milliseconds");
    pick_unit(&mut st, 0);
    type_number(&mut st, "2500");
    let file = stored(dir.path());
    assert!(file.contains("script_timeout_ms = 2500"), "{file}");
    assert!(
        file.contains("script_timeout_unit = \"milliseconds\""),
        "{file}"
    );
    pick_unit(&mut st, 1);
    type_number(&mut st, "30");
    let file = stored(dir.path());
    assert!(file.contains("script_timeout_ms = 30000"), "{file}");
    assert!(file.contains("script_timeout_unit = \"seconds\""), "{file}");
    // Ten minutes is the most.
    type_number(&mut st, "601");
    assert!(st.edit.is_some());
    assert!(stored(dir.path()).contains("script_timeout_ms = 30000"));
}

#[test]
fn the_script_total_timeout_is_seconds_minutes_or_hours() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    on_row(&mut st, PrefSection::Sync, "rewrite.script_total_timeout_s");
    assert_eq!(shown(&st), "1 hour");
    st.config.rewrite.script_total_timeout_s = 90;
    on_row(&mut st, PrefSection::Sync, "rewrite.script_total_timeout_s");
    assert_eq!(shown(&st), "90 seconds");
    pick_unit(&mut st, 1);
    type_number(&mut st, "10");
    let file = stored(dir.path());
    assert!(file.contains("script_total_timeout_s = 600"), "{file}");
    assert!(
        file.contains("script_total_timeout_unit = \"minutes\""),
        "{file}"
    );
    pick_unit(&mut st, 2);
    type_number(&mut st, "24");
    assert!(stored(dir.path()).contains("script_total_timeout_s = 86400"));
    type_number(&mut st, "25");
    assert!(st.edit.is_some(), "more than a day is refused");
    assert!(stored(dir.path()).contains("script_total_timeout_s = 86400"));
}
