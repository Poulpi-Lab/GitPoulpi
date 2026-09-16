//! `ui.reuse_window`: on by default, a live key, readable from the file.

use gitpoulpi_config::{Config, LIVE_KEYS};

#[test]
fn it_is_on_by_default_and_live() {
    assert!(Config::default().ui.reuse_window);
    assert!(LIVE_KEYS.contains(&"ui.reuse_window"));
}

#[test]
fn the_file_can_turn_it_off() {
    let (config, _) = Config::from_toml_str("[ui]\nreuse_window = false\n").unwrap();
    assert!(!config.ui.reuse_window);
    let (config, _) = Config::from_toml_str("[ui]\n").unwrap();
    assert!(config.ui.reuse_window);
}
