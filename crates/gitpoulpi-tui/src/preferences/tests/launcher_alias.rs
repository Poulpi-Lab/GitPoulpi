//! General's command alias and PATH rows: the combobox, the validation, the
//! Create / Remove buttons and the palette route, against temp directories.

use super::*;
use crate::preferences::launcher_alias::{Machine, ToolsButton};
use gitpoulpi_config::desktop::alias;
use gitpoulpi_config::desktop::path::{self as user_path, UserPath, UserPathValues};
use gitpoulpi_config::open_in::Os;
use std::sync::{Arc, Mutex};

struct Memory(Mutex<UserPathValues>);

impl UserPath for Memory {
    fn read(&self) -> Result<UserPathValues, String> {
        Ok(self.0.lock().unwrap().clone())
    }
    fn write(&self, path: &str, added: Option<&str>) -> Result<(), String> {
        *self.0.lock().unwrap() = UserPathValues {
            path: path.into(),
            added: added.map(String::from),
        };
        Ok(())
    }
}

fn machine(root: &Path, path_var: &str) -> Machine {
    let alias = alias::Env {
        os: Os::Linux,
        bin_dir: root.join("home/.local/bin"),
        exe: PathBuf::from("/opt/gp/gitpoulpi-tui"),
        newest: false,
        path_var: Some(path_var.into()),
        pathext: None,
    };
    let path = user_path::Env {
        os: Os::Linux,
        home: root.join("home"),
        config_home: root.join("home/.config"),
        wanted: vec![PathBuf::from("/opt/gp"), alias.bin_dir.clone()],
        path_var: Some(path_var.into()),
    };
    Machine {
        alias,
        path,
        user: Arc::new(Memory(Mutex::new(UserPathValues::default()))),
        registry: Arc::new(gitpoulpi_config::desktop::registry::MemoryRegistry::new()),
    }
}

fn tools_state(dir: &Path, path_var: &'static str) -> PreferencesState {
    let mut st = state(dir);
    let root = dir.join("machine");
    std::fs::create_dir_all(root.join("home")).unwrap();
    st.set_tools_machine(move || machine(&root, path_var));
    st.tools_start(None);
    settle(&mut st);
    st
}

fn settle(st: &mut PreferencesState) {
    for _ in 0..500 {
        st.poll_tools();
        if st.tools.is_idle() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("the alias job never finished");
}

fn row(st: &PreferencesState, label: &str) -> Option<Field> {
    st.fields_of(PrefSection::Launcher)
        .into_iter()
        .find(|f| f.label == label)
}

fn coming(st: &PreferencesState, label: &str) -> bool {
    row(st, label)
        .unwrap_or_else(|| panic!("no row {label}"))
        .coming
        .is_some()
}

fn info(st: &PreferencesState, label: &str) -> String {
    match row(st, label).unwrap().control {
        Control::Info(text) => text,
        other => panic!("not an info row: {other:?}"),
    }
}

#[test]
fn the_alias_row_is_a_combobox_with_none_and_gp_and_free_text() {
    let tmp = tempfile::tempdir().unwrap();
    let st = tools_state(tmp.path(), "/usr/bin");
    let f = row(&st, "Command alias").expect("the alias row");
    assert_eq!(f.target, Target::Global("ui.command_alias"));
    let Control::Combo {
        options,
        value,
        custom,
    } = &f.control
    else {
        panic!("a combobox, never click-to-cycle");
    };
    assert!(*custom, "any name can be typed");
    assert_eq!(value, "");
    let values: Vec<&str> = options.iter().map(|(_, v)| v.as_str()).collect();
    assert_eq!(values, ["", "gp"]);
    assert!(f.live, "the key is a live setting");
}

#[test]
fn an_empty_or_bad_name_disables_create_and_says_why() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = tools_state(tmp.path(), "/usr/bin");
    assert!(coming(&st, "Create: command alias"));
    assert_eq!(info(&st, "Alias status"), "type a name above");
    for (name, text) in [
        ("g p", "the name has a space"),
        ("a/b", "the name has a path separator"),
        ("-x", "the name cannot start with - or ."),
    ] {
        st.config.ui.command_alias = name.into();
        settle(&mut st);
        assert!(coming(&st, "Create: command alias"), "{name}");
        assert_eq!(info(&st, "Alias status"), text);
    }
    // A press on a disabled name never queues a job.
    st.config.ui.command_alias = "g p".into();
    st.run_action(PrefAction::Tools(ToolsButton::AliasCreate));
    assert!(st.tools.is_idle());
    assert_eq!(st.take_message().as_deref(), Some("the name has a space"));
}

#[cfg(unix)]
#[test]
fn create_then_remove_the_alias_through_the_buttons() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = tools_state(tmp.path(), "/usr/bin");
    st.config.ui.command_alias = "gp".into();
    settle(&mut st);
    let link = tmp.path().join("machine/home/.local/bin/gp");
    assert!(!coming(&st, "Create: command alias"));
    assert!(row(&st, "Remove: command alias").is_none(), "nothing yet");

    st.run_action(PrefAction::Tools(ToolsButton::AliasCreate));
    settle(&mut st);
    assert_eq!(
        std::fs::read_link(&link).unwrap(),
        Path::new("/opt/gp/gitpoulpi-tui")
    );
    assert_eq!(
        st.take_message().as_deref(),
        Some("Created command alias: gp")
    );
    assert_eq!(info(&st, "Alias status"), "created");
    assert!(
        row(&st, "Create: command alias").is_none(),
        "a created alias has no Create button"
    );
    assert!(!coming(&st, "Remove: command alias"));

    st.run_action(PrefAction::Tools(ToolsButton::AliasRemove));
    settle(&mut st);
    assert!(std::fs::symlink_metadata(&link).is_err());
    assert_eq!(
        st.take_message().as_deref(),
        Some("Removed command alias: gp")
    );
    assert!(row(&st, "Remove: command alias").is_none());
}

#[test]
fn the_add_alias_row_is_a_combobox_offering_none_gpa_and_the_alias_name_with_an_a() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = tools_state(tmp.path(), "/usr/bin");
    let f = row(&st, "Add-tab alias").expect("the add alias row");
    assert_eq!(f.target, Target::Global("ui.command_alias_add"));
    let Control::Combo {
        options,
        value,
        custom,
    } = &f.control
    else {
        panic!("a combobox, never click-to-cycle");
    };
    assert!(*custom && value.is_empty());
    let values: Vec<&str> = options.iter().map(|(_, v)| v.as_str()).collect();
    assert_eq!(values, ["", "gpa"]);
    assert!(f.live);
    // With another main name, its own derived suggestion joins the list.
    st.config.ui.command_alias = "gitp".into();
    let f = row(&st, "Add-tab alias").unwrap();
    let Control::Combo { options, .. } = &f.control else {
        panic!()
    };
    let values: Vec<&str> = options.iter().map(|(_, v)| v.as_str()).collect();
    assert_eq!(values, ["", "gpa", "gitpa"]);
    // No status row until a name is typed.
    assert!(row(&st, "Add-tab status").is_none());
}

#[cfg(unix)]
#[test]
fn create_makes_both_aliases_and_each_has_its_own_state() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = tools_state(tmp.path(), "/usr/bin");
    st.config.ui.command_alias = "gp".into();
    st.config.ui.command_alias_add = "gpa".into();
    settle(&mut st);
    assert_eq!(info(&st, "Add-tab status"), "not created");
    assert!(!coming(&st, "Create: command alias"));

    st.run_action(PrefAction::Tools(ToolsButton::AliasCreate));
    settle(&mut st);
    let bin = tmp.path().join("machine/home/.local/bin");
    assert_eq!(
        std::fs::read_link(bin.join("gp")).unwrap(),
        Path::new("/opt/gp/gitpoulpi-tui")
    );
    let script = std::fs::read_to_string(bin.join("gpa")).unwrap();
    assert!(
        script.contains("exec '/opt/gp/gitpoulpi-tui' --add \"$@\""),
        "{script}"
    );
    assert_eq!(
        st.take_message().as_deref(),
        Some("Created command alias: gp, gpa")
    );
    assert_eq!(info(&st, "Alias status"), "created");
    assert_eq!(info(&st, "Add-tab status"), "created");
    assert!(
        row(&st, "Create: command alias").is_none(),
        "both exist: nothing left to create"
    );
    // Remove takes out every alias of ours, both kinds.
    st.run_action(PrefAction::Tools(ToolsButton::AliasRemove));
    settle(&mut st);
    assert!(std::fs::symlink_metadata(bin.join("gp")).is_err());
    assert!(std::fs::symlink_metadata(bin.join("gpa")).is_err());
}

#[cfg(unix)]
#[test]
fn the_add_alias_can_be_created_alone_and_a_bad_add_name_blocks_with_its_reason() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = tools_state(tmp.path(), "/usr/bin");
    st.config.ui.command_alias_add = "gpa".into();
    settle(&mut st);
    assert!(
        !coming(&st, "Create: command alias"),
        "the add name is enough"
    );
    st.run_action(PrefAction::Tools(ToolsButton::AliasCreate));
    settle(&mut st);
    let bin = tmp.path().join("machine/home/.local/bin");
    assert!(std::fs::symlink_metadata(bin.join("gp")).is_err());
    assert!(bin.join("gpa").is_file());

    st.config.ui.command_alias_add = "g p".into();
    settle(&mut st);
    assert_eq!(info(&st, "Add-tab status"), "the name has a space");
    assert!(coming(&st, "Create: command alias"));
}

#[cfg(unix)]
#[test]
fn a_file_that_is_not_ours_is_left_alone_and_a_name_on_path_warns() {
    let tmp = tempfile::tempdir().unwrap();
    let bin = tmp.path().join("machine/home/.local/bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::write(bin.join("gp"), "#!/bin/sh\n").unwrap();
    let other = tmp.path().join("other");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(other.join("zz"), "").unwrap();
    let path_var: &'static str = Box::leak(format!("{}", other.display()).into_boxed_str());
    let mut st = tools_state(tmp.path(), path_var);
    st.config.ui.command_alias = "gp".into();
    settle(&mut st);
    assert!(coming(&st, "Create: command alias"));
    assert_eq!(
        info(&st, "Alias status"),
        "a file that is not GitPoulpi's is there"
    );
    assert_eq!(
        std::fs::read_to_string(bin.join("gp")).unwrap(),
        "#!/bin/sh\n"
    );
    // A name some other program on PATH already answers to gets a warning.
    st.config.ui.command_alias = "zz".into();
    settle(&mut st);
    let warning = row(&st, "Command alias").unwrap().warning.unwrap();
    assert!(warning.contains("already runs"), "{warning}");
    assert!(warning.contains("zz"), "{warning}");
    assert!(
        !coming(&st, "Create: command alias"),
        "a warning, not a block"
    );
}

#[cfg(unix)]
#[test]
fn path_add_and_remove_touch_only_our_block_in_the_rc_file() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("machine/home");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join(".bashrc"), "export A=1\n").unwrap();
    let mut st = tools_state(tmp.path(), "/usr/bin");
    assert_eq!(info(&st, "PATH"), "not added");
    assert!(row(&st, "Remove GitPoulpi from PATH").is_none());

    st.run_action(PrefAction::Tools(ToolsButton::PathAdd));
    settle(&mut st);
    let rc = std::fs::read_to_string(home.join(".bashrc")).unwrap();
    assert!(rc.starts_with("export A=1\n"), "{rc}");
    assert!(
        rc.contains("X-GitPoulpi=1") && rc.contains("/opt/gp"),
        "{rc}"
    );
    assert_eq!(info(&st, "PATH"), "added (new terminals)");
    assert!(row(&st, "Add GitPoulpi to PATH").is_none());

    st.run_action(PrefAction::Tools(ToolsButton::PathRemove));
    settle(&mut st);
    assert_eq!(
        std::fs::read_to_string(home.join(".bashrc")).unwrap(),
        "export A=1\n"
    );
    assert_eq!(info(&st, "PATH"), "not added");
}

#[cfg(unix)]
#[test]
fn directories_already_on_path_disable_add_and_hide_remove() {
    let tmp = tempfile::tempdir().unwrap();
    let bin = tmp.path().join("machine/home/.local/bin");
    let var: &'static str = Box::leak(format!("/opt/gp:{}", bin.display()).into_boxed_str());
    let st = tools_state(tmp.path(), var);
    assert_eq!(info(&st, "PATH"), "already on PATH");
    assert!(coming(&st, "Add GitPoulpi to PATH"));
    assert!(row(&st, "Remove GitPoulpi from PATH").is_none());
}

#[cfg(unix)]
#[test]
fn the_palette_route_selects_the_acted_on_button_so_the_page_scrolls_to_it() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = tools_state(tmp.path(), "/usr/bin");
    st.config.ui.command_alias = "gp".into();
    settle(&mut st);
    st.tools_command(ToolsButton::PathAdd);
    settle(&mut st);
    // The Add button vanished once done and Remove took its row: the
    // selection stayed on the same index.
    assert_eq!(
        st.fields()[st.field].target,
        Target::Action(PrefAction::Tools(ToolsButton::PathRemove))
    );
    assert!(st.kept_in_view.is_none(), "the keep-in-view rule must run");
    assert!(st.flash.is_some());
}

#[test]
fn an_old_windows_shim_is_rewritten_by_the_read() {
    let dir = tempfile::tempdir().unwrap();
    let mut m = machine(dir.path(), "");
    m.alias.os = Os::Windows;
    m.alias.exe = PathBuf::from("C:/GP/gitpoulpi-tui.exe");
    std::fs::create_dir_all(&m.alias.bin_dir).unwrap();
    let old = "@echo off\r\nREM X-GitPoulpi=1\r\n\"C:/GP/gitpoulpi-tui.exe\" %*\r\n";
    let file = m.alias.bin_dir.join("gp.cmd");
    std::fs::write(&file, old).unwrap();
    assert_eq!(alias::status(&m.alias, "gp").state, alias::State::Outdated);
    crate::preferences::launcher_alias::sync_shims(&m);
    assert_eq!(alias::status(&m.alias, "gp").state, alias::State::Installed);
    assert!(std::fs::read_to_string(&file)
        .unwrap()
        .contains("GITPOULPI_CMDCMDLINE"));
}
