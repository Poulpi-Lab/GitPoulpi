//! The write path of the `config.toml` / `tui.toml` split, through the public
//! API only: an interface write copies `config.toml` into a complete
//! `tui.toml` and edits that; a common write touches `config.toml` alone;
//! `config.toml` is never modified by an interface write.

use std::collections::BTreeMap;
use std::path::PathBuf;

use gitpoulpi_config::Config;

const OLD_STYLE: &str = "# my notes\n[general]\nundo_depth = 9\n\n[ui]\ntheme = \"dusk\"\nmouse_enabled = false\nlogo_color = \"red\"\nlanguage = \"fr\"\n\n[keybindings.global]\nquit = [\"ctrl+q\"]\n\n[merge]\nside_by_side_min_width = 77\n";

fn setup(config: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let cfg = dir.path().join("config.toml");
    std::fs::write(&cfg, config).unwrap();
    let tui = dir.path().join("tui.toml");
    (dir, cfg, tui)
}

fn doc(path: &PathBuf) -> toml::Value {
    toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn an_interface_write_leaves_config_toml_byte_identical_and_creates_a_complete_tui_toml() {
    let (_dir, cfg, tui) = setup(OLD_STYLE);
    gitpoulpi_config::set_in_file(&cfg, "ui.minimap", "false").unwrap();
    assert_eq!(std::fs::read_to_string(&cfg).unwrap(), OLD_STYLE);
    let tui_doc = doc(&tui);
    // The full derived copy plus the edit, never just the one key.
    assert_eq!(tui_doc["ui"]["theme"].as_str(), Some("dusk"));
    assert_eq!(tui_doc["ui"]["mouse_enabled"].as_bool(), Some(false));
    assert_eq!(tui_doc["ui"]["language"].as_str(), Some("fr"));
    assert_eq!(tui_doc["ui"]["minimap"].as_bool(), Some(false));
    assert!(tui_doc["ui"].get("logo_color").is_none(), "common key");
    assert_eq!(
        tui_doc["keybindings"]["global"]["quit"][0].as_str(),
        Some("ctrl+q")
    );
    assert_eq!(
        tui_doc["merge"]["side_by_side_min_width"].as_integer(),
        Some(77)
    );
    let loaded = Config::load(&cfg).unwrap();
    assert_eq!(loaded.ui.theme, "dusk");
    assert!(!loaded.ui.minimap);
}

#[test]
fn a_common_write_leaves_tui_toml_untouched() {
    let (_dir, cfg, tui) = setup(OLD_STYLE);
    gitpoulpi_config::set_in_file(&cfg, "ui.minimap", "false").unwrap();
    let tui_before = std::fs::read(&tui).unwrap();
    gitpoulpi_config::set_in_file(&cfg, "general.undo_depth", "4").unwrap();
    gitpoulpi_config::set_in_file(&cfg, "ui.logo_color", "teal").unwrap();
    assert_eq!(std::fs::read(&tui).unwrap(), tui_before);
    let loaded = Config::load(&cfg).unwrap();
    assert_eq!(loaded.general.undo_depth, 4);
    assert_eq!(loaded.ui.logo_color, "teal");
    assert!(!loaded.ui.minimap, "the interface key is still there");
}

#[test]
fn a_common_write_without_a_tui_toml_does_not_create_one() {
    let (_dir, cfg, tui) = setup(OLD_STYLE);
    gitpoulpi_config::set_in_file(&cfg, "general.undo_depth", "4").unwrap();
    assert!(!tui.exists());
}

#[test]
fn an_interface_write_is_idempotent() {
    let (_dir, cfg, tui) = setup(OLD_STYLE);
    gitpoulpi_config::set_in_file(&cfg, "ui.theme", "light").unwrap();
    let first = std::fs::read(&tui).unwrap();
    gitpoulpi_config::set_in_file(&cfg, "ui.theme", "light").unwrap();
    assert_eq!(std::fs::read(&tui).unwrap(), first);
    assert_eq!(std::fs::read_to_string(&cfg).unwrap(), OLD_STYLE);
}

#[test]
fn a_bad_interface_value_writes_nothing() {
    let (_dir, cfg, tui) = setup(OLD_STYLE);
    assert!(gitpoulpi_config::set_in_file(&cfg, "ui.mouse_enabled", "perhaps").is_err());
    assert!(!tui.exists(), "validation runs before tui.toml is created");
    assert_eq!(std::fs::read_to_string(&cfg).unwrap(), OLD_STYLE);
}

#[test]
fn once_tui_toml_exists_a_key_it_lacks_is_the_default_not_config_toml() {
    let (_dir, cfg, tui) = setup(OLD_STYLE);
    gitpoulpi_config::set_in_file(&cfg, "ui.minimap", "false").unwrap();
    // Reset "theme" the way a user editing tui.toml by hand would.
    let mut d = doc(&tui);
    d["ui"].as_table_mut().unwrap().remove("theme");
    std::fs::write(&tui, toml::to_string_pretty(&d).unwrap()).unwrap();
    gitpoulpi_config::set_in_file(&cfg, "ui.minimap", "true").unwrap();
    assert_eq!(
        Config::load(&cfg).unwrap().ui.theme,
        Config::default().ui.theme
    );
}

#[test]
fn two_threads_racing_the_first_write_lose_nothing() {
    for _ in 0..20 {
        let (_dir, cfg, tui) = setup(OLD_STYLE);
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let spawn = |key: &'static str, value: &'static str| {
            let (cfg, barrier) = (cfg.clone(), barrier.clone());
            std::thread::spawn(move || {
                barrier.wait();
                gitpoulpi_config::set_in_file(&cfg, key, value).unwrap();
            })
        };
        let a = spawn("ui.minimap", "false");
        let b = spawn("ui.blame_side", "left");
        a.join().unwrap();
        b.join().unwrap();
        let tui_doc = doc(&tui);
        assert_eq!(tui_doc["ui"]["minimap"].as_bool(), Some(false));
        assert_eq!(tui_doc["ui"]["blame_side"].as_str(), Some("left"));
        assert_eq!(tui_doc["ui"]["theme"].as_str(), Some("dusk"), "the copy");
        assert_eq!(std::fs::read_to_string(&cfg).unwrap(), OLD_STYLE);
    }
}

#[test]
fn a_racing_creator_never_clobbers_an_edited_tui_toml() {
    let (dir, cfg, tui) = setup(OLD_STYLE);
    gitpoulpi_config::set_in_file(&cfg, "ui.minimap", "false").unwrap();
    let paths = gitpoulpi_config::ConfigPaths::for_dir(dir.path());
    let before = std::fs::read(&tui).unwrap();
    assert!(!gitpoulpi_config::split::ensure_tui_file(&paths).unwrap());
    assert_eq!(std::fs::read(&tui).unwrap(), before);
}

#[test]
fn the_column_writers_go_to_tui_toml() {
    let (_dir, cfg, tui) = setup(OLD_STYLE);
    let names = vec!["author".to_string()];
    let widths = BTreeMap::from([("author".to_string(), 12u16)]);
    gitpoulpi_config::set_log_columns_in_file(&cfg, &names, &names, &widths, &widths).unwrap();
    gitpoulpi_config::set_log_column_layout_in_file(&cfg, &names, &names).unwrap();
    gitpoulpi_config::set_commit_column_in_file(&cfg, &names, &names, &widths).unwrap();
    gitpoulpi_config::set_sidebar_sections_hidden_in_file(&cfg, &names).unwrap();
    gitpoulpi_config::set_log_columns_layout_in_file(
        &cfg,
        gitpoulpi_config::LogColumnsLayout::Scroll,
    )
    .unwrap();
    assert_eq!(std::fs::read_to_string(&cfg).unwrap(), OLD_STYLE);
    let ui = doc(&tui)["ui"].clone();
    for key in [
        "log_columns_hidden",
        "log_column_widths",
        "log_column_widths_scroll",
        "commit_column_columns_widths",
        "sidebar_sections_hidden",
        "log_columns_layout",
    ] {
        assert!(ui.get(key).is_some(), "{key} in tui.toml");
    }
    assert_eq!(ui["theme"].as_str(), Some("dusk"));
}

#[test]
fn the_keybinding_writers_go_to_tui_toml_and_reset_cleans_it() {
    let (_dir, cfg, tui) = setup(OLD_STYLE);
    gitpoulpi_config::set_keybinding_in_file(&cfg, "global", "help", &["F1".to_string()]).unwrap();
    assert_eq!(std::fs::read_to_string(&cfg).unwrap(), OLD_STYLE);
    let loaded = Config::load(&cfg).unwrap();
    assert_eq!(loaded.keybindings.get("global", "help").unwrap(), ["F1"]);
    assert_eq!(
        loaded.keybindings.get("global", "quit").unwrap(),
        ["ctrl+q"]
    );
    gitpoulpi_config::reset_keybinding_in_file(&cfg, "global", "help").unwrap();
    assert!(Config::load(&cfg)
        .unwrap()
        .keybindings
        .get("global", "help")
        .is_none());
    gitpoulpi_config::reset_all_keybindings_in_file(&cfg).unwrap();
    assert!(doc(&tui).get("keybindings").is_none());
    assert_eq!(std::fs::read_to_string(&cfg).unwrap(), OLD_STYLE);
    // `config.toml` still carries the old table, the interface file owns it.
    assert!(Config::load(&cfg)
        .unwrap()
        .keybindings
        .get("global", "quit")
        .is_none());
}

#[test]
fn resetting_nothing_creates_nothing() {
    let (_dir, cfg, tui) = setup("[general]\nundo_depth = 2\n");
    gitpoulpi_config::reset_all_keybindings_in_file(&cfg).unwrap();
    gitpoulpi_config::reset_keybinding_in_file(&cfg, "global", "help").unwrap();
    assert!(!tui.exists());
}

#[test]
fn list_writers_follow_the_key_home() {
    let (_dir, cfg, tui) = setup(OLD_STYLE);
    let entries = vec![gitpoulpi_config::OpenInEntry::default()];
    // `ui.open_in` is a machine setting: config.toml only.
    gitpoulpi_config::set_open_in_in_file(&cfg, Some(&entries)).unwrap();
    assert!(!tui.exists());
    assert!(doc(&cfg)["ui"].get("open_in").is_some());
}

#[cfg(unix)]
#[test]
fn a_read_only_directory_reads_fine_and_refuses_writes_with_an_io_error() {
    use std::os::unix::fs::PermissionsExt;
    let (dir, cfg, tui) = setup(OLD_STYLE);
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
    let probe = dir.path().join("probe");
    let writable = std::fs::write(&probe, "x").is_ok();
    if !writable {
        assert_eq!(Config::load(&cfg).unwrap().ui.theme, "dusk");
        let err = gitpoulpi_config::set_in_file(&cfg, "ui.minimap", "false").unwrap_err();
        assert!(matches!(err, gitpoulpi_config::Error::Io(_)), "{err:?}");
        assert!(!tui.exists());
        assert_eq!(std::fs::read_to_string(&cfg).unwrap(), OLD_STYLE);
    }
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
}
