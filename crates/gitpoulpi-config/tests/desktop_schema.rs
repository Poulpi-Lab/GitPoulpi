//! `desktop.toml`: the schema, its drift guard against the terminal UI's
//! `[ui]`, the loader, the sparse writers, the environment layer and the
//! per-installation desktop look. Public API only, temp directories only.

use std::path::Path;

use gitpoulpi_config::desktop_ui_file::{DesktopSettings, DESKTOP_LIVE_KEYS};
use gitpoulpi_config::split::{Home, KEY_HOME};
use gitpoulpi_config::{
    Config, ConfigPaths, DesktopColumnWidths, DesktopConfig, DesktopUiConfig, Source,
    DESKTOP_ONLY_KEYS, LIVE_KEYS,
};

fn json_keys<T: serde::Serialize>(value: &T) -> Vec<String> {
    serde_json::to_value(value)
        .unwrap()
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect()
}

#[test]
fn desktop_ui_holds_every_per_interface_key_and_no_terminal_only_one() {
    let desktop = json_keys(&DesktopUiConfig::default());
    let terminal = serde_json::to_value(Config::default().ui).unwrap();
    let defaults = serde_json::to_value(DesktopUiConfig::default()).unwrap();

    for (key, home) in KEY_HOME {
        let Some(name) = key.strip_prefix("ui.") else {
            continue;
        };
        let present = desktop.iter().any(|k| k == name);
        match home {
            Home::PerInterface => {
                // `diff_view_mode` is never serialized; it is TUI-only anyway.
                assert!(
                    present,
                    "{key} is per-interface but missing in desktop.toml"
                );
                assert_eq!(
                    defaults[name], terminal[name],
                    "{key} default drifted from the terminal UI"
                );
            }
            Home::Tui | Home::Common => {
                assert!(!present, "{key} is {home:?} and must not be a desktop key");
            }
        }
    }
    // The rest are the declared desktop-only keys, nothing else.
    for name in &desktop {
        let classified = KEY_HOME.iter().any(|(k, _)| *k == format!("ui.{name}"));
        assert!(
            classified || DESKTOP_ONLY_KEYS.contains(&name.as_str()),
            "ui.{name} is neither classified nor declared desktop-only"
        );
    }
    for name in DESKTOP_ONLY_KEYS {
        assert!(desktop.iter().any(|k| k == name), "{name} is not a key");
        assert!(
            !KEY_HOME.iter().any(|(k, _)| *k == format!("ui.{name}")),
            "{name} is desktop-only but classified"
        );
    }
}

#[test]
fn the_desktop_live_keys_are_the_per_interface_live_keys() {
    let mut expected: Vec<String> = LIVE_KEYS
        .iter()
        .filter(|k| {
            k.starts_with("ui.") && gitpoulpi_config::split::home_of(k) == Home::PerInterface
        })
        .map(|k| format!("desktop.{k}"))
        .collect();
    let mut actual: Vec<String> = DESKTOP_LIVE_KEYS.iter().map(|k| k.to_string()).collect();
    expected.sort();
    actual.sort();
    assert_eq!(actual, expected);
}

#[test]
fn a_missing_file_is_the_defaults_and_a_read_never_creates_it() {
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    let config = DesktopConfig::load(&paths).unwrap();
    assert_eq!(config, DesktopConfig::default());
    assert_eq!(serde_json::to_value(config.ui.language).unwrap(), "auto");
    assert_eq!(config.ui.theme, "system");
    assert_eq!(
        (config.ui.window_width, config.ui.window_height),
        (1500, 900)
    );
    assert_eq!(config.ui.window_x, None);
    assert!(!paths.desktop_file().exists());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[test]
fn it_is_not_seeded_from_tui_toml() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("tui.toml"),
        "[ui]\ntheme = \"dusk\"\nlanguage = \"fr\"\n",
    )
    .unwrap();
    let config = DesktopConfig::load(&ConfigPaths::for_dir(dir.path())).unwrap();
    assert_eq!(config.ui.theme, "system");
    assert_eq!(serde_json::to_value(config.ui.language).unwrap(), "auto");
}

#[test]
fn pixel_values_are_clamped_on_load_and_floats_are_accepted() {
    let config = DesktopConfig::from_toml_str(
        "[ui]\nleft_pane_width = 5\nright_pane_width = 99999.4\nwindow_width = 1200.6\nwindow_x = -99999\n\
         [ui.column_widths_px]\nref_w = 1\ngraph_w = 650.2\n",
    )
    .unwrap();
    assert_eq!(config.ui.left_pane_width, 180);
    assert_eq!(config.ui.right_pane_width, 760);
    assert_eq!(config.ui.window_width, 1201);
    assert_eq!(config.ui.window_x, Some(-32_000));
    assert_eq!(config.ui.column_widths_px.ref_w, 64);
    assert_eq!(config.ui.column_widths_px.graph_w, 650);
    assert_eq!(config.ui.column_widths_px.sha_w, 72);
}

#[test]
fn unknown_keys_are_ignored_by_the_loader_and_kept_by_the_writers() {
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    std::fs::write(
        paths.desktop_file(),
        "future = 1\n\n[ui]\nfuture_key = \"x\"\ntheme = \"dusk\"\n\n[future_table]\na = 2\n",
    )
    .unwrap();
    assert_eq!(DesktopConfig::load(&paths).unwrap().ui.theme, "dusk");
    DesktopConfig::set_in_file(&paths, "ui.minimap", "true").unwrap();
    let doc: toml::Value =
        toml::from_str(&std::fs::read_to_string(paths.desktop_file()).unwrap()).unwrap();
    assert_eq!(doc["future"].as_integer(), Some(1));
    assert_eq!(doc["ui"]["future_key"].as_str(), Some("x"));
    assert_eq!(doc["ui"]["theme"].as_str(), Some("dusk"));
    assert_eq!(doc["ui"]["minimap"].as_bool(), Some(true));
    assert_eq!(doc["future_table"]["a"].as_integer(), Some(2));
}

const COMMON: &str = "# keep me\n[general]\nundo_depth = 9\n\n[ui]\ntheme = \"dusk\"\n";
const TUI: &str = "[ui]\nminimap = true\n";

fn with_other_files() -> (tempfile::TempDir, ConfigPaths) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("config.toml"), COMMON).unwrap();
    std::fs::write(dir.path().join("tui.toml"), TUI).unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    (dir, paths)
}

#[test]
fn desktop_writes_never_touch_config_toml_or_tui_toml() {
    let (dir, paths) = with_other_files();
    DesktopConfig::set_in_file(&paths, "ui.theme", "dark").unwrap();
    DesktopConfig::set_in_file(&paths, "ui.window_x", "40").unwrap();
    DesktopConfig::set_list_in_file(&paths, "ui.log_columns_hidden", &["sha".into()]).unwrap();
    DesktopConfig::set_window(&paths, (1300, 800), Some((10, 20)), true).unwrap();
    DesktopConfig::set_panes(&paths, true, false, 300, 400).unwrap();
    DesktopConfig::set_column_widths(&paths, None, DesktopColumnWidths::default()).unwrap();
    DesktopConfig::set_column_widths(&paths, Some("/r"), DesktopColumnWidths::default()).unwrap();
    DesktopConfig::set_keybinding(&paths, "global", "quit", &["Ctrl+Q".into()]).unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.path().join("config.toml")).unwrap(),
        COMMON
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("tui.toml")).unwrap(),
        TUI
    );
    // And the other direction: the terminal UI's writers leave desktop.toml alone.
    let before = std::fs::read(paths.desktop_file()).unwrap();
    gitpoulpi_config::set_in_file(&paths.config_file(), "ui.minimap", "false").unwrap();
    gitpoulpi_config::set_in_file(&paths.config_file(), "general.undo_depth", "4").unwrap();
    assert_eq!(std::fs::read(paths.desktop_file()).unwrap(), before);
    // The terminal UI's view is unchanged by desktop values (`tui.toml` owns
    // the theme and says nothing: the default, as the split specifies).
    let tui_view = Config::load(&paths.config_file()).unwrap();
    assert_eq!(tui_view.ui.theme, "system");
    assert_eq!(tui_view.general.undo_depth, 4);
}

#[test]
fn writers_store_sparse_values_and_the_typed_setters_clamp() {
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    DesktopConfig::set_window(&paths, (10, 10), None, false).unwrap();
    DesktopConfig::set_panes(&paths, true, true, 9999, 1).unwrap();
    let c = DesktopConfig::load(&paths).unwrap();
    assert_eq!((c.ui.window_width, c.ui.window_height), (900, 560));
    assert_eq!((c.ui.left_pane_width, c.ui.right_pane_width), (520, 280));
    let doc: toml::Value =
        toml::from_str(&std::fs::read_to_string(paths.desktop_file()).unwrap()).unwrap();
    assert!(
        doc["ui"].get("theme").is_none(),
        "sparse: only what was written"
    );
    assert!(doc["ui"].get("window_x").is_none());

    DesktopConfig::set_window(&paths, (1400, 850), Some((5, -7)), false).unwrap();
    let c = DesktopConfig::load(&paths).unwrap();
    assert_eq!((c.ui.window_x, c.ui.window_y), (Some(5), Some(-7)));
    DesktopConfig::set_window(&paths, (1400, 850), None, false).unwrap();
    assert_eq!(DesktopConfig::load(&paths).unwrap().ui.window_x, None);
}

#[test]
fn column_widths_per_repository_fall_back_to_the_default_and_reset() {
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    let own = DesktopColumnWidths {
        ref_w: 9999,
        ..Default::default()
    };
    DesktopConfig::set_column_widths(&paths, Some("/work/a.b"), own).unwrap();
    let c = DesktopConfig::load(&paths).unwrap();
    assert_eq!(c.column_widths_for("/work/a.b").ref_w, 420);
    assert_eq!(c.column_widths_for("/other"), c.ui.column_widths_px);
    DesktopConfig::reset_column_widths(&paths, "/work/a.b").unwrap();
    let c = DesktopConfig::load(&paths).unwrap();
    assert!(c.repos.is_empty());
    let text = std::fs::read_to_string(paths.desktop_file()).unwrap();
    assert!(!text.contains("repos"), "{text}");
}

#[test]
fn the_generic_writer_refuses_bad_values_and_leaves_the_file_alone() {
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    for (key, value) in [
        ("ui.left_pane_width", "5"),
        ("ui.column_widths_px.sha_w", "9"),
        ("ui.window_width", "wide"),
        ("ui.toolbar_position", "sideways"),
        ("ui.no_such_key", "1"),
        ("ui.tui_only_key_mouse_enabled", "true"),
        ("ui.mouse_enabled", "true"),
        ("nothing.else", "1"),
        ("repos.x", "1"),
    ] {
        let err = DesktopConfig::set_in_file(&paths, key, value);
        assert!(err.is_err(), "{key}={value} should be refused");
    }
    assert!(!paths.desktop_file().exists());
    let stored = DesktopConfig::set_in_file(&paths, "ui.left_pane_width", "300").unwrap();
    assert_eq!(stored, serde_json::json!(300));
}

#[test]
fn unset_and_keybinding_resets_drop_the_key_and_emptied_tables() {
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    DesktopConfig::set_in_file(&paths, "ui.minimap", "true").unwrap();
    DesktopConfig::set_keybinding(&paths, "global", "quit", &["Ctrl+Q".into()]).unwrap();
    assert_eq!(
        DesktopConfig::load(&paths)
            .unwrap()
            .keybindings
            .get("global", "quit")
            .unwrap(),
        ["Ctrl+Q"]
    );
    DesktopConfig::reset_keybinding(&paths, "global", "quit").unwrap();
    DesktopConfig::unset_in_file(&paths, "ui.minimap").unwrap();
    let doc: toml::Value =
        toml::from_str(&std::fs::read_to_string(paths.desktop_file()).unwrap()).unwrap();
    assert!(
        doc.get("keybindings").is_none() && doc.get("ui").is_none(),
        "{doc}"
    );
    // Unsetting what is not there is a no-op.
    DesktopConfig::reset_all_keybindings(&paths).unwrap();
}

#[test]
fn two_threads_writing_do_not_lose_each_others_keys() {
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    std::thread::scope(|s| {
        for key in [
            "ui.minimap",
            "ui.ref_tree",
            "ui.diff_word_wrap",
            "ui.image_view",
        ] {
            let paths = &paths;
            s.spawn(move || {
                let value = if key == "ui.ref_tree" || key == "ui.image_view" {
                    "false"
                } else {
                    "true"
                };
                DesktopConfig::set_in_file(paths, key, value).unwrap();
            });
        }
    });
    let c = DesktopConfig::load(&paths).unwrap();
    assert!(c.ui.minimap && c.ui.diff_word_wrap && !c.ui.ref_tree && !c.ui.image_view);
}

#[cfg(unix)]
#[test]
fn a_read_only_directory_reads_fine_and_refuses_writes_with_an_io_error() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
    if std::fs::write(dir.path().join("probe"), "x").is_err() {
        assert_eq!(
            DesktopConfig::load(&paths).unwrap(),
            DesktopConfig::default()
        );
        let err = DesktopConfig::set_in_file(&paths, "ui.minimap", "true").unwrap_err();
        assert!(matches!(err, gitpoulpi_config::Error::Io(_)), "{err:?}");
        assert!(!paths.desktop_file().exists());
    }
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
}

fn settings(file: &Path, env: &[(&str, &str)]) -> DesktopSettings {
    let env: Vec<(String, String)> = env
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    DesktopSettings::resolve_from(file, move |name| {
        env.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone())
    })
    .unwrap()
}

#[test]
fn the_environment_beats_the_file_by_the_mechanical_rule() {
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    DesktopConfig::set_in_file(&paths, "ui.minimap", "false").unwrap();
    assert_eq!(
        gitpoulpi_config::env_var_for("desktop.ui.minimap"),
        "GITPOULPI_DESKTOP_UI_MINIMAP"
    );
    let file = paths.desktop_file();
    let plain = settings(&file, &[]);
    assert_eq!(plain.get("desktop.ui.minimap").unwrap(), false);
    assert_eq!(plain.source_of("desktop.ui.minimap"), Source::File);
    assert_eq!(plain.source_of("desktop.ui.ref_tree"), Source::Default);
    let env = settings(&file, &[("GITPOULPI_DESKTOP_UI_MINIMAP", "true")]);
    assert_eq!(env.get("desktop.ui.minimap").unwrap(), true);
    assert_eq!(env.source_of("desktop.ui.minimap"), Source::Env);
    // The terminal UI's variable does not apply to the desktop.
    let tui_var = settings(&file, &[("GITPOULPI_UI_MINIMAP", "true")]);
    assert_eq!(tui_var.get("desktop.ui.minimap").unwrap(), false);
    // A wrong value is an error against the variable the user typed.
    let err = DesktopSettings::resolve_from(&file, |n| {
        (n == "GITPOULPI_DESKTOP_UI_MINIMAP").then(|| "maybe".to_string())
    })
    .unwrap_err();
    assert!(
        err.to_string().contains("GITPOULPI_DESKTOP_UI_MINIMAP"),
        "{err}"
    );
}

mod desktop_look {
    use gitpoulpi_config::installation::{
        desktop_look_value, effective_desktop_look, set_desktop_look_in_file, Identity,
    };
    use gitpoulpi_config::{Config, ConfigPaths, DesktopConfig, DesktopUiConfig};

    fn identity(id: &str) -> Identity {
        Identity {
            id: id.to_string(),
            kind: gitpoulpi_config::Kind::Pinned,
            root: String::new(),
            channel: None,
        }
    }

    #[test]
    fn an_older_entry_keeps_desktop_look_through_the_unknown_field_passthrough() {
        let text = "[installation.settings.a]\nkind = \"dev\"\n[installation.settings.a.desktop_look]\ntheme = \"dusk\"\n";
        let config: Config = toml::from_str(text).unwrap();
        let entry = &config.installation.settings["a"];
        assert_eq!(entry.desktop_look["theme"], "dusk");
        let back = toml::to_string(&config).unwrap();
        assert!(back.contains("desktop_look"), "{back}");
    }

    #[test]
    fn the_primary_follows_desktop_toml_and_others_their_entry_or_the_default() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path());
        std::fs::write(paths.config_file(), "[installation]\nprimary = \"p\"\n").unwrap();
        let (a, p) = (identity("a"), identity("p"));

        // Nothing stored: the default look.
        let config = Config::load(&paths.config_file()).unwrap();
        let desktop = DesktopConfig::load(&paths).unwrap();
        let default = desktop_look_value(&DesktopUiConfig::default(), "theme");
        assert_eq!(
            effective_desktop_look(&config, &desktop.ui, "a")["theme"],
            default
        );

        // The primary writes desktop.toml, another installation its entry.
        set_desktop_look_in_file(&paths, &p, "theme", "dusk").unwrap();
        set_desktop_look_in_file(&paths, &a, "theme", "dark").unwrap();
        let config = Config::load(&paths.config_file()).unwrap();
        let desktop = DesktopConfig::load(&paths).unwrap();
        assert_eq!(desktop.ui.theme, "dusk");
        assert_eq!(
            effective_desktop_look(&config, &desktop.ui, "p")["theme"],
            "dusk"
        );
        assert_eq!(
            effective_desktop_look(&config, &desktop.ui, "a")["theme"],
            "dark"
        );
        // Another installation does not follow the primary's desktop.toml.
        assert_eq!(
            effective_desktop_look(&config, &desktop.ui, "z")["theme"],
            default
        );
        // The terminal UI's look is untouched by any of it.
        assert_eq!(config.installation.settings["a"].look.len(), 0);
        assert_eq!(config.ui.theme, "system");
        assert!(set_desktop_look_in_file(&paths, &a, "minimap", "true").is_err());
        assert!(set_desktop_look_in_file(&paths, &a, "theme", "").is_ok());
    }
}
