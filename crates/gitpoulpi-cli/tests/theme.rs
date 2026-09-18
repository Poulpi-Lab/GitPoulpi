//! `gitpoulpi theme list|export` and the `ui.theme` / `ui.graph_color_theme`
//! validation of `config set`. JSON assertions, throwaway config dir.

use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;

fn run(config_dir: &Path, args: &[&str]) -> (bool, Value) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir)
        .current_dir(config_dir)
        .arg("--json")
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout} / {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.success(), json)
}

fn write_user_theme(dir: &Path, slug: &str, text: &str) {
    let themes = dir.join("themes");
    std::fs::create_dir_all(&themes).unwrap();
    std::fs::write(themes.join(format!("{slug}.toml")), text).unwrap();
}

fn row<'a>(list: &'a Value, slug: &str) -> &'a Value {
    list["themes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["slug"] == slug)
        .unwrap_or_else(|| panic!("no theme {slug} in {list}"))
}

#[test]
fn theme_list_has_dark_light_and_user_themes() {
    let config = tempfile::tempdir().unwrap();
    write_user_theme(
        config.path(),
        "mine",
        "name = \"My Theme\"\nbase = \"light\"\n[ui]\naccent = \"#ff0000\"\n",
    );
    let (ok, list) = run(config.path(), &["theme", "list"]);
    assert!(ok, "{list}");

    let dark = row(&list, "dark");
    assert_eq!(dark["source"], "preset");
    assert_eq!(dark["light"], false);
    assert!(dark["lane_palette"].is_string(), "{dark}");
    let light = row(&list, "light");
    assert_eq!(light["light"], true);

    let mine = row(&list, "mine");
    assert_eq!(mine["source"], "user");
    assert_eq!(mine["label"], "My Theme");
    assert_eq!(mine["light"], true);
}

#[test]
fn config_set_ui_theme_rejects_unknown_names_and_lists_the_valid_ones() {
    let config = tempfile::tempdir().unwrap();
    write_user_theme(config.path(), "mine", "base = \"dark\"\n");
    let (ok, json) = run(config.path(), &["config", "set", "ui.theme", "nope"]);
    assert!(!ok);
    let err = json["error"].as_str().unwrap();
    for name in ["nope", "system", "dark", "light", "mine"] {
        assert!(err.contains(name), "{err}");
    }
    assert!(!config.path().join("config.toml").exists());

    for good in ["dark", "light", "system", "default"] {
        let (ok, json) = run(config.path(), &["config", "set", "ui.theme", good]);
        assert!(ok, "{good}: {json}");
    }
}

#[test]
fn config_set_graph_color_theme_rejects_unknown_palettes() {
    let config = tempfile::tempdir().unwrap();
    let (ok, json) = run(
        config.path(),
        &["config", "set", "ui.graph_color_theme", "nope"],
    );
    assert!(!ok);
    let err = json["error"].as_str().unwrap();
    for name in ["nope", "theme", "gitpoulpi", "light"] {
        assert!(err.contains(name), "{err}");
    }
}

#[test]
fn theme_export_round_trips_through_the_user_file_parser() {
    let config = tempfile::tempdir().unwrap();
    let (ok, dark) = run(config.path(), &["theme", "export", "dark"]);
    assert!(ok, "{dark}");
    let toml = dark["toml"].as_str().unwrap();
    assert!(toml.contains("[ui]"), "{toml}");

    write_user_theme(config.path(), "copy", toml);
    let (ok, again) = run(config.path(), &["theme", "export", "copy"]);
    assert!(ok, "{again}");
    assert_eq!(again["toml"], dark["toml"]);

    let (ok, list) = run(config.path(), &["theme", "list"]);
    assert!(ok);
    assert_eq!(list["warnings"].as_array().unwrap().len(), 0, "{list}");

    let (ok, err) = run(config.path(), &["theme", "export", "nope"]);
    assert!(!ok);
    assert!(err["error"].as_str().unwrap().contains("dark"));
}

#[test]
fn config_set_accepts_every_preset_user_theme_and_lane_palette_end_to_end() {
    let config = tempfile::tempdir().unwrap();
    write_user_theme(
        config.path(),
        "mine",
        "base = \"nord\"\ngraph_palette = [\"#ff0000\"]\n",
    );
    let (ok, list) = run(config.path(), &["theme", "list"]);
    assert!(ok, "{list}");
    let slugs: Vec<String> = list["themes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["slug"].as_str().unwrap().to_string())
        .collect();
    assert!(slugs.len() >= 11, "{slugs:?}");
    for slug in &slugs {
        let (ok, json) = run(config.path(), &["config", "set", "ui.theme", slug]);
        assert!(ok, "{slug}: {json}");
        let (ok, got) = run(config.path(), &["config", "get", "ui.theme"]);
        assert!(ok, "{got}");
        assert_eq!(got["value"], slug.as_str(), "{got}");
    }
    for palette in [
        "theme",
        "gitpoulpi",
        "serie",
        "nord",
        "catppuccin-latte",
        "mine",
    ] {
        let (ok, json) = run(
            config.path(),
            &["config", "set", "ui.graph_color_theme", palette],
        );
        assert!(ok, "{palette}: {json}");
    }
    let (ok, json) = run(config.path(), &["config", "set", "ui.input_color", "theme"]);
    assert!(ok, "{json}");
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
