//! The read path of the `config.toml` / `tui.toml` split, through the public
//! API only: without a `tui.toml` nothing changes, with one it owns its keys.

use gitpoulpi_config::split::{derive_tui_document, merge_documents};
use gitpoulpi_config::Config;

fn as_json(config: &Config) -> serde_json::Value {
    serde_json::to_value(config).unwrap()
}

fn doc(raw: &str) -> toml::Value {
    toml::from_str(raw).unwrap()
}

fn fixtures() -> Vec<(&'static str, String)> {
    vec![
        ("empty", String::new()),
        (
            "shipped default",
            include_str!("fixtures/default_config.toml").to_string(),
        ),
        (
            "ui and keybindings",
            "[general]\nundo_depth = 9\n\n[ui]\ntheme = \"dusk\"\nmouse_enabled = false\nlogo_color = \"red\"\nlanguage = \"fr\"\nblame_width = 33\n\n[keybindings.global]\nquit = [\"ctrl+q\"]\n\n[merge]\nside_by_side_min_width = 77\n".into(),
        ),
        (
            "legacy keys",
            "[ui]\ndiff_view_mode = \"split\"\nshow_avatars = true\n".into(),
        ),
        (
            "legacy overridden by the new keys",
            "[ui]\ndiff_view_mode = \"split\"\ndiff_layout = \"inline\"\ndiff_hunks_only = false\n".into(),
        ),
        (
            "unknown ui key and nested lists",
            "[ui]\nfuture_knob = \"x\"\nlog_columns_hidden = [\"author\"]\n\n[ui.log_column_widths]\nauthor = 20\n\n[[ui.open_in]]\nid = \"editor\"\nlabel = \"Editor\"\ncommand = \"edit\"\n".into(),
        ),
        (
            "only common tables",
            "[general]\nundo_depth = 3\n\n[log]\nlevel = \"debug\"\n".into(),
        ),
    ]
}

fn dir_with(config: &str, tui: Option<&str>) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("config.toml"), config).unwrap();
    if let Some(tui) = tui {
        std::fs::write(dir.path().join("tui.toml"), tui).unwrap();
    }
    dir
}

#[test]
fn no_tui_toml_gives_exactly_the_config_toml_alone() {
    for (name, raw) in fixtures() {
        let dir = dir_with(&raw, None);
        let loaded = Config::load(&dir.path().join("config.toml")).unwrap();
        let (direct, _) = Config::from_toml_str(&raw).unwrap();
        assert_eq!(as_json(&loaded), as_json(&direct), "{name}");
    }
}

#[test]
fn a_derived_tui_toml_changes_nothing() {
    for (name, raw) in fixtures() {
        let derived = toml::to_string_pretty(&derive_tui_document(&doc(&raw))).unwrap();
        let dir = dir_with(&raw, Some(&derived));
        let loaded = Config::load(&dir.path().join("config.toml")).unwrap();
        let (direct, _) = Config::from_toml_str(&raw).unwrap();
        assert_eq!(as_json(&loaded), as_json(&direct), "{name}");
    }
}

#[test]
fn merging_the_derived_document_restores_the_original_unless_a_legacy_key_was_folded() {
    for (name, raw) in fixtures() {
        let common = doc(&raw);
        let merged = merge_documents(&common, &derive_tui_document(&common));
        let legacy = raw.contains("diff_view_mode");
        if !legacy {
            assert_eq!(merged, common, "{name}");
        }
    }
}

#[test]
fn tui_toml_values_win_and_silence_means_default() {
    let dir = dir_with(
        "[general]\nundo_depth = 9\n\n[ui]\ntheme = \"old\"\nmouse_enabled = false\nlogo_color = \"red\"\n\n[merge]\nside_by_side_min_width = 10\n\n[keybindings.global]\nquit = [\"x\"]\n",
        Some("[ui]\ntheme = \"new\"\n\n[merge]\nside_by_side_min_width = 120\n"),
    );
    let config = Config::load(&dir.path().join("config.toml")).unwrap();
    assert_eq!(config.ui.theme, "new");
    assert!(config.ui.mouse_enabled, "tui.toml is silent: the default");
    assert_eq!(
        config.ui.logo_color, "red",
        "common key stays in config.toml"
    );
    assert_eq!(config.general.undo_depth, 9);
    assert_eq!(config.merge.side_by_side_min_width, 120);
    assert!(config.keybindings.is_empty(), "the table is tui.toml's");
}

#[test]
fn a_common_key_typed_in_tui_toml_is_ignored() {
    let dir = dir_with(
        "[ui]\nlogo_color = \"red\"\n",
        Some("[ui]\nlogo_color = \"blue\"\n"),
    );
    let config = Config::load(&dir.path().join("config.toml")).unwrap();
    assert_eq!(config.ui.logo_color, "red");
}

#[test]
fn a_tui_toml_with_no_config_toml_still_loads() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("tui.toml"), "[ui]\ntheme = \"solo\"\n").unwrap();
    let config = Config::load(&dir.path().join("config.toml")).unwrap();
    assert_eq!(config.ui.theme, "solo");
}

#[test]
fn a_tui_toml_that_does_not_parse_is_an_error_not_a_silent_default() {
    let dir = dir_with("", Some("[ui\n"));
    assert!(Config::load(&dir.path().join("config.toml")).is_err());
}

#[test]
fn a_toml_not_named_config_toml_has_no_sibling() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("other.toml"), "[ui]\ntheme = \"a\"\n").unwrap();
    std::fs::write(dir.path().join("tui.toml"), "[ui]\ntheme = \"b\"\n").unwrap();
    let config = Config::load(&dir.path().join("other.toml")).unwrap();
    assert_eq!(config.ui.theme, "a");
}

#[test]
fn unknown_ui_keys_survive_the_derivation() {
    let common = doc("[ui]\nfuture_knob = \"x\"\n");
    let tui = derive_tui_document(&common);
    assert_eq!(tui["ui"]["future_knob"].as_str(), Some("x"));
    assert_eq!(merge_documents(&common, &tui), common);
}
