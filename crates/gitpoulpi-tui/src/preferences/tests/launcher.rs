//! General's Launcher group: rows, the combobox's detected terminals and the
//! Create / Remove buttons, all against temp directories.

use super::*;
use crate::launcher::Env;
use gitpoulpi_config::desktop::{Dirs, Place};
use gitpoulpi_config::open_in::Os;
use launcher_page::LauncherButton;

fn temp_env(root: &Path) -> Env {
    std::fs::create_dir_all(root.join("Desktop")).unwrap();
    Env {
        os: Os::Linux,
        dirs: Dirs {
            applications: root.join("data/applications"),
            icon_root: root.join("data/icons/hicolor"),
            desktop: Some(root.join("Desktop")),
            ..Dirs::default()
        },
        exe: PathBuf::from("/opt/gp/gitpoulpi"),
        launch_exe: None,
        newest: false,
        has: std::sync::Arc::new(|p| matches!(p, "kitty" | "foot")),
        which: std::sync::Arc::new(|_| None),
        refresh_icon_cache: false,
    }
}

/// A state whose shortcuts live under `dir`, with the first machine read done.
fn launcher_state(dir: &Path) -> PreferencesState {
    let mut st = state(dir);
    let root = dir.join("machine");
    st.set_launcher_env(move || temp_env(&root));
    st.launcher_start(None);
    settle(&mut st);
    st
}

/// Polls until the running job has been applied.
fn settle(st: &mut PreferencesState) {
    for _ in 0..500 {
        st.poll_launcher();
        if st.launcher.is_idle() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("the launcher job never finished");
}

/// What the launcher place is called on this OS (macOS: the Applications
/// folder; the temp environment is the Linux layout either way).
fn launcher_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "Applications folder"
    } else {
        "App launcher"
    }
}

fn labels(st: &PreferencesState) -> Vec<String> {
    st.fields_of(PrefSection::General)
        .iter()
        .map(|f| f.label.clone())
        .collect()
}

fn coming(st: &PreferencesState, label: &str) -> bool {
    st.fields_of(PrefSection::General)
        .iter()
        .find(|f| f.label == label)
        .unwrap_or_else(|| panic!("no row {label}"))
        .coming
        .is_some()
}

#[test]
fn the_terminal_combobox_offers_automatic_the_os_default_and_what_is_installed() {
    let tmp = tempfile::tempdir().unwrap();
    let st = launcher_state(tmp.path());
    let fields = st.fields_of(PrefSection::General);
    let combo = fields
        .iter()
        .find(|f| f.target == Target::Global("ui.launch_terminal"))
        .expect("the terminal row");
    let Control::Combo { options, value, .. } = &combo.control else {
        panic!("a combobox, never click-to-cycle");
    };
    assert_eq!(value, "auto");
    let values: Vec<&str> = options.iter().map(|(_, v)| v.as_str()).collect();
    assert_eq!(values, ["auto", "default", "kitty", "foot"]);
    assert!(
        options[0].0.contains("kitty"),
        "auto names its pick: {options:?}"
    );
}

#[test]
fn create_then_remove_through_the_buttons_in_a_temp_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = launcher_state(tmp.path());
    let entry = tmp
        .path()
        .join("machine/data/applications/gitpoulpi.desktop");

    assert!(labels(&st).contains(&format!("Create: {}", launcher_name())));
    assert!(
        !labels(&st).contains(&format!("Remove: {}", launcher_name())),
        "nothing to remove yet"
    );

    st.run_action(PrefAction::Launcher(LauncherButton::Install(
        Place::Launcher,
    )));
    settle(&mut st);
    let text = std::fs::read_to_string(&entry).unwrap();
    assert!(
        text.contains("Exec=kitty --class gitpoulpi /opt/gp/gitpoulpi"),
        "{text}"
    );
    assert!(
        !labels(&st)
            .iter()
            .any(|l| l.starts_with(&format!("Create: {}", launcher_name()))
                || l.starts_with("Update")),
        "a created shortcut has no Create or Update button: {:?}",
        labels(&st)
    );
    assert!(!coming(&st, &format!("Remove: {}", launcher_name())));
    assert_eq!(
        st.take_message().as_deref(),
        Some(format!("created: {}", launcher_name()).as_str())
    );

    st.run_action(PrefAction::Launcher(LauncherButton::Remove(
        Place::Launcher,
    )));
    settle(&mut st);
    assert!(!entry.exists());
    assert!(!labels(&st).contains(&format!("Remove: {}", launcher_name())));
    assert!(labels(&st).contains(&format!("Create: {}", launcher_name())));
}

#[test]
fn a_foreign_file_disables_create_and_hides_remove() {
    let tmp = tempfile::tempdir().unwrap();
    let apps = tmp.path().join("machine/data/applications");
    std::fs::create_dir_all(&apps).unwrap();
    std::fs::write(apps.join("gitpoulpi.desktop"), "[Desktop Entry]\nName=x\n").unwrap();
    let st = launcher_state(tmp.path());
    assert!(coming(&st, &format!("Create: {}", launcher_name())));
    assert!(!labels(&st).contains(&format!("Remove: {}", launcher_name())));
}

#[test]
fn a_press_while_a_job_runs_is_queued_not_lost() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = launcher_state(tmp.path());
    let entry = tmp
        .path()
        .join("machine/data/applications/gitpoulpi.desktop");
    st.launcher_start(None);
    st.run_action(PrefAction::Launcher(LauncherButton::Install(
        Place::Launcher,
    )));
    settle(&mut st);
    assert!(entry.exists());
}

#[test]
fn changing_the_terminal_rewrites_a_created_shortcut_without_any_button() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = launcher_state(tmp.path());
    let entry = tmp
        .path()
        .join("machine/data/applications/gitpoulpi.desktop");
    st.run_action(PrefAction::Launcher(LauncherButton::Install(
        Place::Launcher,
    )));
    settle(&mut st);
    assert!(std::fs::read_to_string(&entry)
        .unwrap()
        .contains("Exec=kitty "));
    st.config.ui.launch_terminal = "foot".to_string();
    settle(&mut st);
    let text = std::fs::read_to_string(&entry).unwrap();
    assert!(text.contains("Exec=foot "), "{text}");
    assert!(!labels(&st).iter().any(|l| l.starts_with("Update")));
}

#[test]
fn there_is_no_taskbar_row_and_no_taskbar_catalog_key() {
    let tmp = tempfile::tempdir().unwrap();
    let st = launcher_state(tmp.path());
    assert!(
        !labels(&st)
            .iter()
            .any(|l| l.to_lowercase().contains("taskbar")),
        "a pinned copy follows the icon by itself; no hint row"
    );
    let en = include_str!("../../../../gitpoulpi-i18n/locales/en.toml");
    assert!(!en.contains("\ntaskbar = ") && !en.contains("taskbar_hint"));
}

#[test]
fn the_launcher_group_has_the_reuse_window_toggle_on_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = launcher_state(dir.path());
    let label = gitpoulpi_i18n::msg::launcher::REUSE.get();
    let row = |st: &PreferencesState| {
        st.fields_of(PrefSection::General)
            .into_iter()
            .find(|f| f.label == label)
            .unwrap_or_else(|| panic!("no row {label}: {:?}", labels(st)))
    };
    let field = row(&st);
    assert!(matches!(field.control, Control::Toggle(true)));
    assert!(
        field.coming.is_none(),
        "it works on every OS that has a launcher"
    );
    // The row follows the config value.
    st.config.ui.reuse_window = false;
    assert!(matches!(row(&st).control, Control::Toggle(false)));
}

// -- Dev mode: the launcher build choice ----------------------------------

fn build_field(st: &PreferencesState) -> Option<Field> {
    st.fields_of(PrefSection::General)
        .into_iter()
        .find(|f| f.target == Target::Global("ui.launcher_build"))
}

#[test]
fn the_launcher_build_row_exists_only_in_dev_mode() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = launcher_state(tmp.path());
    // The test binary is not in a checkout's target/<profile>: not dev mode.
    assert!(!st.dev_mode());
    assert!(build_field(&st).is_none());
    assert!(!st
        .setting_rows()
        .iter()
        .any(|r| r.key == "ui.launcher_build"));
    assert!(!st.row_pool().iter().any(|f| f.label == "Launcher build"));

    st.config.general.dev_mode = Some(true);
    let field = build_field(&st).expect("shown in dev mode");
    let Control::Combo { options, value, .. } = &field.control else {
        panic!("a combobox, never click-to-cycle");
    };
    assert_eq!(value, "current");
    let values: Vec<&str> = options.iter().map(|(_, v)| v.as_str()).collect();
    assert_eq!(values, ["current", "newest"]);
    // The palette and the settings search reach it like any other row.
    assert!(st
        .setting_rows()
        .iter()
        .any(|r| r.key == "ui.launcher_build"));

    st.config.general.dev_mode = Some(false);
    assert!(build_field(&st).is_none(), "an explicit false hides it");
}

#[test]
fn the_launcher_build_row_is_painted_in_dev_mode_and_absent_otherwise() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = launcher_state(tmp.path());
    st.config.general.dev_mode = Some(true);
    let at = st
        .fields_of(PrefSection::General)
        .iter()
        .position(|f| f.target == Target::Global("ui.launcher_build"))
        .unwrap();
    st.field = at;
    assert!(draw_screen(&mut st).contains("Launcher build"));
    st.config.general.dev_mode = None;
    st.field = 0;
    assert!(!draw_screen(&mut st).contains("Launcher build"));
}

#[test]
fn newest_needs_both_dev_mode_and_the_choice() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = launcher_state(tmp.path());
    st.config.general.dev_mode = Some(true);
    assert!(!st.launcher_newest());
    st.config.ui.launcher_build = gitpoulpi_config::LauncherBuild::Newest;
    assert!(st.launcher_newest());
    st.config.general.dev_mode = Some(false);
    assert!(
        !st.launcher_newest(),
        "newest means nothing outside dev mode"
    );
}
