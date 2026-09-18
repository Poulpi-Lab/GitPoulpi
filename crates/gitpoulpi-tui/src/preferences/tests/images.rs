//! The Images page: viewer keys and the `[[ui.image_editors]]` list.
use super::super::list_editor::ListOp;
use super::*;
use gitpoulpi_config::image_editors::Detected;
use gitpoulpi_config::OpenInEntry;
use model::{OpenInButton, OpenInTarget};

fn entry(label: &str, command: &str) -> OpenInEntry {
    OpenInEntry::new(label, command)
}

/// A state on the Images page holding `A`, `B`, `C`, with nothing detected.
fn images_state(dir: &Path) -> PreferencesState {
    let mut st = state_at(dir, "Images");
    st.image_detected.set(Detected::default()).unwrap();
    let list = vec![
        entry("A", "a ${file}"),
        entry("B", "b ${file}"),
        entry("C", "c ${file}"),
    ];
    gitpoulpi_config::set_image_editors_in_file(&dir.join("config.toml"), Some(&list)).unwrap();
    st.config.ui.image_editors = Some(list);
    st
}

fn saved(dir: &Path) -> Config {
    Config::load(&dir.join("config.toml")).unwrap()
}

fn labels(dir: &Path) -> Vec<String> {
    saved(dir)
        .ui
        .image_editors
        .unwrap_or_default()
        .into_iter()
        .map(|e| e.label)
        .collect()
}

fn row(st: &PreferencesState, target: Target) -> usize {
    st.fields()
        .iter()
        .position(|f| f.target == target)
        .unwrap_or_else(|| panic!("no row {target:?}"))
}

fn serial() -> std::sync::MutexGuard<'static, ()> {
    crate::tab_dir::TEST_SERIAL
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

#[test]
#[cfg(feature = "image-view")]
fn the_viewer_rows_are_live_keys_and_comboboxes() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = images_state(dir.path());
    st.set_sub_tab(SubTab::General);
    let fields = st.fields();
    for key in [
        "ui.image_view",
        "ui.image_view_text_fallback",
        "ui.image_view_max_mib",
        "ui.image_view_pixelated_from",
        "ui.image_view_sync",
        "ui.image_editor_default",
    ] {
        assert!(
            fields.iter().any(|f| f.target == Target::Global(key)),
            "{key} missing"
        );
    }
    let fallback = row(&st, Target::Global("ui.image_view_text_fallback"));
    let Control::Choice { options, .. } = &st.fields()[fallback].control else {
        panic!("the fallback must be a combobox");
    };
    assert_eq!(options, &["halfblock", "card"]);
    assert_eq!(st.fields()[fallback].option_label("card"), "Info card only");
    st.field = fallback;
    st.commit_choice(&Target::Global("ui.image_view_text_fallback"), 1);
    st.commit_bool(&Target::Global("ui.image_view"), false);
    let config = saved(dir.path());
    assert_eq!(
        config.ui.image_view_text_fallback,
        gitpoulpi_config::ImageTextFallback::Card
    );
    assert!(!config.ui.image_view);
}

#[test]
fn adding_a_preset_appends_it_and_selects_it() {
    let _s = serial();
    let dir = tempfile::tempdir().unwrap();
    let mut st = images_state(dir.path());
    let options = st.images_preset_options();
    let wanted = options
        .get(1)
        .cloned()
        .and_then(|(_, e)| e)
        .expect("a preset");
    st.images_op(ListOp::Add(Some(0)));
    let list = saved(dir.path()).ui.image_editors.unwrap();
    assert_eq!(list.len(), 4);
    assert_eq!(list[3], wanted);
    assert_eq!(st.image_index, 3);
}

#[test]
fn custom_and_restore_turn_the_key_on_and_off() {
    let _s = serial();
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "Images");
    st.image_detected.set(Detected::default()).unwrap();
    assert!(st.config.ui.image_editors.is_none());
    st.images_op(ListOp::Add(Some(st.images_preset_options().len() - 1)));
    assert!(saved(dir.path()).ui.image_editors.is_some());
    st.images_button(OpenInButton::Restore);
    assert!(saved(dir.path()).ui.image_editors.is_none());
}

#[test]
fn label_and_command_edits_are_validated_and_stored() {
    let _s = serial();
    let dir = tempfile::tempdir().unwrap();
    let mut st = images_state(dir.path());
    st.image_index = 1;
    let label = row(&st, Target::ImageEditor(OpenInTarget::Label));
    let command = row(&st, Target::ImageEditor(OpenInTarget::Command));
    st.field = label;
    assert!(!st.commit_text(label, "C"), "a taken label is refused");
    assert!(!st.commit_text(command, "gimp ${nope}"), "unknown variable");
    assert!(!st.commit_text(command, ""), "empty command");
    assert!(st.commit_text(label, "Bee"));
    assert!(st.commit_text(command, "bee --open ${file}"));
    let list = saved(dir.path()).ui.image_editors.unwrap();
    assert_eq!(list[1], entry("Bee", "bee --open ${file}"));
}

#[test]
fn a_chip_appends_its_variable() {
    let _s = serial();
    let dir = tempfile::tempdir().unwrap();
    let mut st = images_state(dir.path());
    st.images_insert_variable(1);
    assert_eq!(
        saved(dir.path()).ui.image_editors.unwrap()[0].command,
        "a ${file} ${dir}"
    );
}

#[test]
fn reorder_and_remove_keep_the_default_in_step() {
    let _s = serial();
    let dir = tempfile::tempdir().unwrap();
    let mut st = images_state(dir.path());
    st.images_op(ListOp::Move { from: 2, to: 1 });
    assert_eq!(labels(dir.path()), ["A", "C", "B"]);
    assert_eq!(st.image_index, 1);
    st.images_op(ListOp::Move { from: 1, to: 2 });
    assert_eq!(labels(dir.path()), ["A", "B", "C"]);
    st.set_sub_tab(SubTab::General);
    st.field = row(&st, Target::Global("ui.image_editor_default"));
    st.commit_choice(&Target::Global("ui.image_editor_default"), 2);
    st.set_sub_tab(SubTab::List);
    assert_eq!(saved(dir.path()).ui.image_editor_default, "C");
    st.image_index = 2;
    let label = row(&st, Target::ImageEditor(OpenInTarget::Label));
    assert!(st.commit_text(label, "Sea"));
    assert_eq!(saved(dir.path()).ui.image_editor_default, "Sea");
    st.images_op(ListOp::Remove(2));
    assert_eq!(saved(dir.path()).ui.image_editor_default, "");
    assert_eq!(labels(dir.path()), ["A", "B"]);
}

#[test]
fn the_test_button_runs_the_entry_on_a_read_only_sample_png() {
    let _s = serial();
    let dir = tempfile::tempdir().unwrap();
    let mut st = images_state(dir.path());
    // A command that exists where the tests run and ignores its argument
    // (`true` is not a program on Windows).
    let command = if cfg!(windows) {
        "cmd /c rem ${file}"
    } else {
        "true ${file}"
    };
    let list = vec![entry("Fake", command)];
    st.config.ui.image_editors = Some(list);
    st.image_index = 0;
    st.images_button(OpenInButton::Test);
    let message = st.message.clone().unwrap_or_default();
    assert!(message.contains("Fake"), "{message}");
    let sample = std::env::temp_dir()
        .join(format!("gitpoulpi-image-{}", std::process::id()))
        .join("settings-test")
        .join("gitpoulpi-sample.png");
    let bytes = std::fs::read(&sample).expect("the sample was written");
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    assert!(std::fs::metadata(&sample).unwrap().permissions().readonly());
}

#[test]
fn search_finds_the_images_page() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    let images = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::Images)
        .unwrap();
    st.on_key(KeyEvent::from(KeyCode::Char('/')));
    for c in "sample".chars() {
        st.on_key(KeyEvent::from(KeyCode::Char(c)));
    }
    assert!(
        st.search
            .as_ref()
            .unwrap()
            .hits
            .iter()
            .any(|h| h.section == images),
        "{:?}",
        st.search.as_ref().unwrap().hits
    );
}

fn has_key(fields: &[Field], key: &str) -> bool {
    fields
        .iter()
        .any(|f| matches!(f.target, Target::Global(k) if k == key))
}

#[test]
fn without_the_viewer_the_page_keeps_only_the_editors_part() {
    let _g = serial();
    let dir = tempfile::tempdir().unwrap();
    let st = images_state(dir.path());
    let with = st.images_general_fields_for(true);
    for key in [
        "ui.image_view",
        "ui.image_view_text_fallback",
        "ui.image_view_max_mib",
        "ui.image_view_pixelated_from",
        "ui.image_view_sync",
    ] {
        assert!(has_key(&with, key), "{key} is on the full page");
    }
    let without = st.images_general_fields_for(false);
    for key in [
        "ui.image_view",
        "ui.image_view_text_fallback",
        "ui.image_view_max_mib",
        "ui.image_view_pixelated_from",
        "ui.image_view_sync",
    ] {
        assert!(
            !has_key(&without, key),
            "{key} is hidden without the viewer"
        );
    }
    // The editors part is all there: the default entry, the list editing.
    assert!(has_key(&without, "ui.image_editor_default"));
    // The list tab holds the selected editor's rows, with or without it.
    assert!(st
        .images_item_fields()
        .iter()
        .any(|f| matches!(f.target, Target::ImageEditor(OpenInTarget::Command))));
    assert!(without.len() < with.len());
}

/// What the real build shows follows its features.
#[test]
fn the_page_of_this_build_matches_its_features() {
    let _g = serial();
    let dir = tempfile::tempdir().unwrap();
    let mut st = images_state(dir.path());
    st.set_sub_tab(SubTab::General);
    assert_eq!(
        has_key(&st.fields(), "ui.image_view"),
        crate::build_features::IMAGE_VIEW
    );
}
