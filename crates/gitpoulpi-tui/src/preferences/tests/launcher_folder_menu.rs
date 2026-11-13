//! The File manager menu rows of the Launcher page: the state rows, the
//! Install / Remove buttons and the palette route, against temp directories
//! and a memory registry (never the real home or registry).

use super::*;
use crate::preferences::launcher_folder_menu::FolderMenuButton;
use gitpoulpi_config::desktop::folder_menu::Env as MenuEnv;
use gitpoulpi_config::desktop::registry::MemoryRegistry;
use gitpoulpi_config::open_in::Os;
use std::sync::Arc;

fn menu_env(root: &Path, helper: bool, present: &'static [&'static str]) -> MenuEnv {
    let bin = root.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    MenuEnv {
        os: Os::Linux,
        data_home: root.join("data"),
        config_home: root.join("config"),
        home: root.join("home"),
        exe: bin.join("gitpoulpi-tui"),
        helper: helper.then(|| bin.join("gitpoulpi")),
        has: Arc::new(move |p| present.contains(&p)),
        registry: Arc::new(MemoryRegistry::new()),
        icon_png: Some(vec![1, 2, 3]),
        refresh_hooks: false,
    }
}

fn menu_state(dir: &Path, helper: bool, present: &'static [&'static str]) -> PreferencesState {
    let mut st = state(dir);
    let root = dir.join("machine");
    st.set_folder_menu_env(move |_| menu_env(&root, helper, present));
    st.folder_menu_start(None);
    settle(&mut st);
    st
}

fn settle(st: &mut PreferencesState) {
    for _ in 0..500 {
        st.poll_folder_menu();
        if st.folder_menu.is_idle() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("the folder menu job never finished");
}

fn rows(st: &PreferencesState) -> Vec<Field> {
    st.fields_of(PrefSection::Launcher)
}

fn info(st: &PreferencesState, label: &str) -> String {
    match rows(st)
        .into_iter()
        .find(|f| f.label == label)
        .unwrap()
        .control
    {
        Control::Info(text) => text,
        other => panic!("not an info row: {other:?}"),
    }
}

fn buttons(st: &PreferencesState) -> Vec<Btn> {
    rows(st)
        .into_iter()
        .find_map(|f| match f.control {
            Control::Buttons(b)
                if b.iter()
                    .any(|b| b.action == PrefAction::FolderMenu(FolderMenuButton::Install)) =>
            {
                Some(b)
            }
            _ => None,
        })
        .expect("the buttons row")
}

#[test]
fn rows_list_the_detected_file_managers_and_both_buttons_start_right() {
    let tmp = tempfile::tempdir().unwrap();
    let st = menu_state(tmp.path(), true, &["dolphin", "thunar"]);
    assert_eq!(info(&st, "Dolphin"), "not created");
    assert_eq!(info(&st, "Thunar"), "not created");
    assert_eq!(info(&st, "Detected"), "Dolphin, Thunar");
    let b = buttons(&st);
    assert_eq!((b[0].label.as_str(), b[0].enabled), ("Install", true));
    assert_eq!((b[1].label.as_str(), b[1].enabled), ("Remove", false));
}

#[test]
fn install_then_remove_through_the_buttons_and_remove_asks_first() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = menu_state(tmp.path(), true, &["dolphin", "thunar"]);
    let machine = tmp.path().join("machine");
    st.run_action(PrefAction::FolderMenu(FolderMenuButton::Install));
    settle(&mut st);
    assert!(machine
        .join("data/kio/servicemenus/gitpoulpi-open.desktop")
        .is_file());
    assert!(machine.join("config/Thunar/uca.xml").is_file());
    assert_eq!(
        st.take_message().as_deref(),
        Some("Installed Open in GitPoulpi: Dolphin, Thunar")
    );
    assert_eq!(info(&st, "Dolphin"), "created");
    assert_eq!(info(&st, "Thunar"), "created");
    let b = buttons(&st);
    assert!(!b[0].enabled, "everything is installed");
    assert!(b[1].enabled);

    // Remove is destructive: pressing it asks, and nothing is touched yet.
    st.press_button(&b[1]);
    assert!(st.confirm_action.is_some());
    assert!(st.folder_menu.is_idle());
    assert!(machine.join("config/Thunar/uca.xml").is_file());

    st.run_action(PrefAction::FolderMenu(FolderMenuButton::Remove));
    settle(&mut st);
    assert!(!machine.join("config/Thunar/uca.xml").exists());
    assert!(!machine
        .join("data/kio/servicemenus/gitpoulpi-open.desktop")
        .exists());
    assert_eq!(
        st.take_message().as_deref(),
        Some("Removed Open in GitPoulpi: Dolphin, Thunar")
    );
    let b = buttons(&st);
    assert!(b[0].enabled && !b[1].enabled);
}

#[test]
fn the_palette_route_selects_the_buttons_row() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = menu_state(tmp.path(), true, &["nemo"]);
    st.folder_menu_command(false);
    settle(&mut st);
    assert!(tmp
        .path()
        .join("machine/data/nemo/actions/gitpoulpi-open.nemo_action")
        .is_file());
    assert_eq!(
        st.fields()[st.field].target,
        Target::Action(PrefAction::FolderMenu(FolderMenuButton::Install))
    );
    assert!(st.flash.is_some());
    st.folder_menu_command(true);
    settle(&mut st);
    assert!(!tmp
        .path()
        .join("machine/data/nemo/actions/gitpoulpi-open.nemo_action")
        .exists());
}

#[test]
fn without_the_launcher_beside_the_exe_both_buttons_are_disabled_with_the_reason() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = menu_state(tmp.path(), false, &["dolphin"]);
    assert_eq!(
        info(&st, "Open in GitPoulpi"),
        "needs the gitpoulpi launcher next to GitPoulpi"
    );
    assert!(buttons(&st).iter().all(|b| !b.enabled));
    st.run_action(PrefAction::FolderMenu(FolderMenuButton::Install));
    assert!(
        st.folder_menu.is_idle(),
        "no job for a menu that cannot be made"
    );
    assert_eq!(
        st.take_message().as_deref(),
        Some("needs the gitpoulpi launcher next to GitPoulpi")
    );
}

#[test]
fn with_no_file_manager_found_install_is_disabled_and_the_row_says_so() {
    let tmp = tempfile::tempdir().unwrap();
    let st = menu_state(tmp.path(), true, &[]);
    assert_eq!(info(&st, "Detected"), "no supported file manager found");
    assert!(!buttons(&st)[0].enabled);
}

#[test]
fn until_the_machine_is_read_the_rows_say_checking_and_the_buttons_wait() {
    let tmp = tempfile::tempdir().unwrap();
    let st = state(tmp.path());
    assert_eq!(info(&st, "Open in GitPoulpi"), "checking...");
    assert!(buttons(&st).iter().all(|b| !b.enabled));
}
