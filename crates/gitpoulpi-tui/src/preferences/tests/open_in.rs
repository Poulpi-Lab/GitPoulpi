use super::super::list_editor::ListOp;
use super::*;
use gitpoulpi_config::open_in::Detected;
use gitpoulpi_config::OpenInEntry;
use model::{OpenInButton, OpenInTarget};

fn entry(label: &str, command: &str) -> OpenInEntry {
    OpenInEntry::new(label, command)
}

/// A state on the Open in page holding `A`, `B`, `C`, with nothing detected.
fn open_in_state(dir: &Path) -> PreferencesState {
    let mut st = state_at(dir, "OpenIn");
    st.open_in_detected.set(Detected::default()).unwrap();
    let list = vec![
        entry("A", "a ${cwd}"),
        entry("B", "b ${cwd}"),
        entry("C", "c ${cwd}"),
    ];
    gitpoulpi_config::set_open_in_in_file(&dir.join("config.toml"), Some(&list)).unwrap();
    st.config.ui.open_in = Some(list);
    st
}

fn saved(dir: &Path) -> Config {
    Config::load(&dir.join("config.toml")).unwrap()
}

fn labels(config: &Config) -> Vec<String> {
    config
        .ui
        .open_in
        .clone()
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
fn adding_a_preset_from_the_menu_appends_it_with_its_command() {
    let _s = serial();
    let dir = tempfile::tempdir().unwrap();
    let mut st = open_in_state(dir.path());
    let options = st.open_in_preset_options();
    let wanted = options
        .get(1)
        .cloned()
        .and_then(|(_, e)| e)
        .expect("a preset after the placeholder");
    // The menu is the presets then Custom: pick 0 is the first preset.
    st.open_in_op(ListOp::Add(Some(0)));
    let list = saved(dir.path()).ui.open_in.unwrap();
    assert_eq!(list.len(), 4);
    assert_eq!(list[3], wanted);
    assert_eq!(st.open_in_index, 3, "the new entry is selected");
    let spec = st.open_in_list_spec();
    let super::super::list_editor::AddKind::Menu(menu) = &spec.add else {
        panic!("a menu");
    };
    assert_eq!(menu.last().unwrap(), "Add custom", "Custom closes the menu");
    crate::tab_dir::reset_for_tests();
}

#[test]
fn the_first_edit_turns_an_absent_list_into_a_saved_one() {
    let _s = serial();
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "OpenIn");
    st.open_in_detected.set(Detected::default()).unwrap();
    assert!(st.config.ui.open_in.is_none());
    let before = st.open_in_list();
    assert!(!before.is_empty(), "the computed list shows (the explorer)");
    st.open_in_op(ListOp::Add(Some(st.open_in_preset_options().len() - 1)));
    let list = saved(dir.path()).ui.open_in.expect("now a saved list");
    assert_eq!(list.len(), before.len() + 1);
    assert_eq!(&list[..before.len()], &before[..]);
    // Restore goes back to "not configured".
    st.open_in_button(OpenInButton::Restore);
    assert!(saved(dir.path()).ui.open_in.is_none());
    crate::tab_dir::reset_for_tests();
}

#[test]
fn editing_the_label_and_the_command_updates_config_toml() {
    let _s = serial();
    let dir = tempfile::tempdir().unwrap();
    let mut st = open_in_state(dir.path());
    st.open_in_index = 1;
    let label = row(&st, Target::OpenIn(OpenInTarget::Label));
    let command = row(&st, Target::OpenIn(OpenInTarget::Command));
    assert!(st.commit_text(label, "  Bee  "));
    assert!(st.commit_text(command, "bee --dir ${cwd} ${branch}"));
    let list = saved(dir.path()).ui.open_in.unwrap();
    assert_eq!(list[1], entry("Bee", "bee --dir ${cwd} ${branch}"));
    // A taken or empty label and an unknown variable are refused, nothing written.
    assert!(!st.commit_text(label, "A"));
    assert!(!st.commit_text(label, " "));
    assert!(!st.commit_text(command, "x ${nope}"));
    assert_eq!(saved(dir.path()).ui.open_in.unwrap(), list);
    crate::tab_dir::reset_for_tests();
}

#[test]
fn a_chip_appends_its_variable_to_the_command() {
    let _s = serial();
    let dir = tempfile::tempdir().unwrap();
    let mut st = open_in_state(dir.path());
    st.focus = PrefFocus::Fields;
    st.field = row(&st, Target::OpenIn(OpenInTarget::Chips));
    st.ai.chip = 3; // ${name}
    st.activate();
    let list = saved(dir.path()).ui.open_in.unwrap();
    assert_eq!(list[0].command, "a ${cwd} ${name}");
    crate::tab_dir::reset_for_tests();
}

#[test]
fn move_reorders_and_remove_drops_the_entry() {
    let _s = serial();
    let dir = tempfile::tempdir().unwrap();
    let mut st = open_in_state(dir.path());
    st.open_in_op(ListOp::Move { from: 2, to: 1 });
    assert_eq!(labels(&saved(dir.path())), ["A", "C", "B"]);
    assert_eq!(st.open_in_index, 1, "the selection follows the entry");
    st.open_in_op(ListOp::Move { from: 1, to: 0 });
    assert_eq!(labels(&saved(dir.path())), ["C", "A", "B"]);
    st.open_in_op(ListOp::Move { from: 0, to: 1 });
    assert_eq!(labels(&saved(dir.path())), ["A", "C", "B"]);
    st.open_in_op(ListOp::Remove(1));
    assert_eq!(labels(&saved(dir.path())), ["A", "B"]);
    st.open_in_op(ListOp::Remove(0));
    st.open_in_op(ListOp::Remove(0));
    assert_eq!(
        saved(dir.path()).ui.open_in,
        Some(Vec::new()),
        "an emptied list is a saved empty list"
    );
    // Nothing left: the page has no entry rows.
    assert!(!st
        .fields()
        .iter()
        .any(|f| matches!(f.target, Target::OpenIn(OpenInTarget::Label))));
    crate::tab_dir::reset_for_tests();
}

#[test]
fn the_default_entry_is_starred_and_follows_a_move() {
    let _s = serial();
    let dir = tempfile::tempdir().unwrap();
    let mut st = open_in_state(dir.path());
    let starred = |st: &PreferencesState| {
        st.open_in_list_spec()
            .items
            .iter()
            .position(|i| i.mark == super::super::list_editor::Mark::Default)
    };
    assert_eq!(starred(&st), Some(0), "the first entry is the default");
    st.set_sub_tab(SubTab::General);
    st.field = row(&st, Target::Global("ui.open_in_default"));
    st.commit_choice(&Target::Global("ui.open_in_default"), 2);
    assert_eq!(starred(&st), Some(2));
    st.open_in_op(ListOp::Move { from: 2, to: 0 });
    assert_eq!(starred(&st), Some(0), "the star follows the label");
    crate::tab_dir::reset_for_tests();
}

#[test]
fn the_default_combobox_lists_only_existing_labels_and_persists() {
    let _s = serial();
    let dir = tempfile::tempdir().unwrap();
    let mut st = open_in_state(dir.path());
    st.set_sub_tab(SubTab::General);
    let fields = st.fields();
    let at = row(&st, Target::Global("ui.open_in_default"));
    let Control::Choice { options, index } = &fields[at].control else {
        panic!("the default is a combobox");
    };
    assert_eq!(options, &["A", "B", "C"]);
    assert_eq!(*index, 0);
    st.field = at;
    st.commit_choice(&Target::Global("ui.open_in_default"), 2);
    assert_eq!(saved(dir.path()).ui.open_in_default, "C");
    // Renaming the default entry keeps it the default; removing it clears it.
    st.open_in_index = 2;
    st.set_sub_tab(SubTab::List);
    let label = row(&st, Target::OpenIn(OpenInTarget::Label));
    assert!(st.commit_text(label, "Sea"));
    assert_eq!(saved(dir.path()).ui.open_in_default, "Sea");
    st.open_in_op(ListOp::Remove(2));
    assert_eq!(saved(dir.path()).ui.open_in_default, "");
    crate::tab_dir::reset_for_tests();
}

#[test]
fn selecting_a_list_row_changes_the_entry_the_rows_edit() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = open_in_state(dir.path());
    st.apply_list_op(ListOp::Select(2));
    let fields = st.fields();
    let label = row(&st, Target::OpenIn(OpenInTarget::Label));
    assert!(matches!(&fields[label].control, Control::Text(t) if t == "C"));
}

#[test]
fn add_preset_groups_this_os_and_flags_unverified_ones() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "OpenIn");
    st.config.ui.open_in = Some(Vec::new());
    st.open_in_detected
        .set(Detected {
            editors: vec!["code"],
            ..Detected::default()
        })
        .unwrap();
    let options: Vec<String> = st
        .open_in_preset_options()
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    assert_eq!(options[0], "Choose a preset...");
    let at = |needle: &str| {
        options
            .iter()
            .position(|o| o.contains(needle))
            .unwrap_or_else(|| panic!("{needle} in {options:?}"))
    };
    // Detected first, then Explorer / Multiplexer / Terminal / Editor.
    assert!(at("Detected: VS Code") < at("Multiplexer: "));
    assert!(at("Multiplexer: ") < at("Terminal: "));
    assert!(at("Terminal: ") < at("Editor: "));
    // No preset carries a verification flag.
    assert!(options.iter().all(|o| !o.contains("unverified")));
    // VS Code is not offered twice.
    assert_eq!(options.iter().filter(|o| o.contains("VS Code")).count(), 1);
    // Only this OS.
    if cfg!(not(windows)) {
        assert!(!options.iter().any(|o| o.contains("Windows Terminal")));
    }
}

#[test]
fn search_finds_the_open_in_page() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    st.on_key(KeyEvent::from(KeyCode::Char('/')));
    for c in "open in".chars() {
        st.on_key(KeyEvent::from(KeyCode::Char(c)));
    }
    let open_in = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::OpenIn)
        .unwrap();
    assert!(
        st.search
            .as_ref()
            .unwrap()
            .hits
            .iter()
            .any(|h| h.section == open_in),
        "{:?}",
        st.search.as_ref().unwrap().hits
    );
    // An agent can land on the page's default row by its key.
    st.search = None;
    st.show_page(None, Some("ui.open_in_default")).unwrap();
    assert_eq!(st.current_section(), PrefSection::OpenIn);
    // The default entry is a General tab row: the jump lands on that tab.
    assert_eq!(st.sub_tab(), SubTab::General);
    assert_eq!(
        st.fields()[st.field].target,
        Target::Global("ui.open_in_default")
    );
}
