//! The colour theme and the graph colours belong to one installation each:
//! two installations sharing one `config.toml` (pinned with
//! `GITPOULPI_INSTALL_ID`) never recolour each other. Only the primary owns
//! the shared `ui.*` keys. The config folder is a scratch directory.

use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;

fn run(config: &Path, id: &str, args: &[&str]) -> (bool, Value) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .env("GITPOULPI_INSTALL_ID", id)
        .env_remove("GITPOULPI_ENV")
        .arg("--json")
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("not JSON ({e}): {stdout}"));
    (output.status.success(), json)
}

fn get(config: &Path, id: &str, key: &str) -> String {
    let (ok, json) = run(config, id, &["config", "get", key]);
    assert!(ok, "{json}");
    json["value"].as_str().unwrap().to_string()
}

fn file_value(config: &Path, key: &str) -> Option<String> {
    let text = std::fs::read_to_string(config.join("config.toml")).ok()?;
    let doc: toml::Value = toml::from_str(&text).ok()?;
    doc.get("ui")?.get(key)?.as_str().map(str::to_string)
}

#[test]
fn a_theme_set_by_one_installation_never_reaches_the_other() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path();
    std::fs::write(config.join("config.toml"), "[ui]\ntheme = \"nord\"\n").unwrap();
    let (ok, made) = run(config, "alpha", &["installation", "set", "name", "Alpha"]);
    assert!(ok, "{made}");
    let (ok, made) = run(config, "beta", &["installation", "set", "name", "Beta"]);
    assert!(ok, "{made}");
    let (ok, json) = run(config, "alpha", &["config", "set", "ui.theme", "light"]);
    assert!(ok, "{json}");
    let (ok, json) = run(
        config,
        "alpha",
        &["config", "set", "ui.graph_color_theme", "light"],
    );
    assert!(ok, "{json}");
    // Nobody is primary: A wrote its entry, never the shared key.
    assert_eq!(file_value(config, "theme").as_deref(), Some("nord"));
    assert_eq!(get(config, "alpha", "ui.theme"), "light");
    assert_ne!(
        get(config, "beta", "ui.theme"),
        "light",
        "B keeps its own theme"
    );
    let (ok, json) = run(config, "beta", &["config", "set", "ui.theme", "dark"]);
    assert!(ok, "{json}");
    assert_eq!(get(config, "alpha", "ui.theme"), "light");
    assert_eq!(get(config, "beta", "ui.theme"), "dark");
    assert_eq!(get(config, "alpha", "ui.graph_color_theme"), "light");
    assert_eq!(get(config, "beta", "ui.graph_color_theme"), "theme");
    // The primary writes the shared key, older builds keep working.
    let (ok, json) = run(config, "alpha", &["installation", "set", "name", "A2"]);
    assert!(ok, "{json}");
    let text = std::fs::read_to_string(config.join("config.toml")).unwrap();
    let mut doc: toml::Value = toml::from_str(&text).unwrap();
    doc["installation"]
        .as_table_mut()
        .unwrap()
        .insert("primary".into(), toml::Value::String("p-alpha".into()));
    std::fs::write(config.join("config.toml"), toml::to_string(&doc).unwrap()).unwrap();
    let (ok, json) = run(config, "alpha", &["config", "set", "ui.theme", "dark"]);
    assert!(ok, "{json}");
    assert_eq!(file_value(config, "theme").as_deref(), Some("dark"));
    assert_eq!(get(config, "beta", "ui.theme"), "dark");
}
