//! Pins the serialized form of the config schema: a default `Config` is
//! byte-identical to `fixtures/default_config.toml`, and every closed-set
//! enum round-trips through TOML under the name `as_str` reports.

use gitpoulpi_config::sequences::{PauseMode, Scope};
use gitpoulpi_config::*;
use serde::{Deserialize, Serialize};

/// Drops the default values that depend on the OS or the home directory.
fn machine_independent(toml: String) -> String {
    toml.lines()
        .filter(|l| !l.starts_with("clone_root") && !l.starts_with("file_explorer"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

#[test]
fn a_default_config_serialises_exactly_as_pinned() {
    let text = toml::to_string(&Config::default()).unwrap();
    let text = machine_independent(text);
    if std::env::var_os("BLESS_DEFAULT_CONFIG").is_some() {
        std::fs::write(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/default_config.toml"
            ),
            &text,
        )
        .unwrap();
    }
    let pinned = include_str!("fixtures/default_config.toml");
    assert_eq!(text, pinned, "the serialized default config changed");
}

#[test]
fn removed_keymap_keys_in_an_old_config_are_ignored_and_not_written_back() {
    let old = "[ui]\nkeymap_preset = \"vim\"\n[ui.keymap_overrides]\n\"x\" = \"y\"\n";
    let (config, _) = Config::from_toml_str(old).expect("an old config still loads");
    let text = toml::to_string(&config).unwrap();
    assert!(!text.contains("keymap"), "{text}");
}

#[test]
fn a_default_config_reparses_to_the_same_text() {
    let text = toml::to_string(&Config::default()).unwrap();
    let (again, _) = Config::from_toml_str(&text).unwrap();
    assert_eq!(toml::to_string(&again).unwrap(), text);
}

#[derive(Serialize, Deserialize)]
struct Wrap<T> {
    value: T,
}

macro_rules! round_trips {
    ($($ty:ident),+ $(,)?) => {$(
        {
            let mut names = std::collections::HashSet::new();
            assert!(!$ty::ALL.is_empty(), stringify!($ty));
            for &v in $ty::ALL {
                let text = toml::to_string(&Wrap { value: v }).unwrap();
                assert_eq!(
                    text.trim(),
                    format!("value = \"{}\"", v.as_str()),
                    "{}::{:?} serialises under a name other than as_str",
                    stringify!($ty),
                    v
                );
                let back: Wrap<$ty> = toml::from_str(&text).unwrap();
                assert_eq!(back.value, v, stringify!($ty));
                assert!(names.insert(v.as_str()), "duplicate name in {}", stringify!($ty));
            }
        }
    )+};
}

#[test]
fn every_enum_value_round_trips_through_toml_under_its_as_str_name() {
    round_trips!(
        GraphColorization,
        GraphRenderer,
        KittyKeyboard,
        ColorDepth,
        TextCursor,
        TabDirFollow,
        LauncherBuild,
        ImageProtocol,
        GraphLaneBackground,
        GraphRowTintStrength,
        ToolbarPosition,
        ToolbarAlign,
        DialogGeometry,
        StatusbarPosition,
        TabsPosition,
        DiffLayout,
        PrTerminology,
        EditorEol,
        AvatarPlacement,
        AvatarSources,
        AvatarFallback,
        ComposerFiles,
        BlameSide,
        ColumnResize,
        SidebarSectionsScope,
        ImageTextFallback,
        ImagePixelatedFrom,
        AutoCrlf,
        EolMode,
        PullButtonMode,
        PushBehavior,
        RetentionUnit,
        Scope,
        PauseMode,
    );
}

#[test]
fn the_all_lists_have_the_pinned_lengths() {
    assert_eq!(GraphColorization::ALL.len(), 4);
    assert_eq!(GraphRenderer::ALL.len(), 3);
    assert_eq!(KittyKeyboard::ALL.len(), 2);
    assert_eq!(ColorDepth::ALL.len(), 4);
    assert_eq!(TextCursor::ALL.len(), 2);
    assert_eq!(TabDirFollow::ALL.len(), 2);
    assert_eq!(ImageProtocol::ALL.len(), 5);
    assert_eq!(GraphLaneBackground::ALL.len(), 2);
    assert_eq!(GraphRowTintStrength::ALL.len(), 3);
    assert_eq!(ToolbarPosition::ALL.len(), 4);
    assert_eq!(ToolbarAlign::ALL.len(), 3);
    assert_eq!(DialogGeometry::ALL.len(), 3);
    assert_eq!(StatusbarPosition::ALL.len(), 2);
    assert_eq!(TabsPosition::ALL.len(), 3);
    assert_eq!(DiffLayout::ALL.len(), 2);
    assert_eq!(PrTerminology::ALL.len(), 3);
    assert_eq!(EditorEol::ALL.len(), 3);
    assert_eq!(AvatarPlacement::ALL.len(), 3);
    assert_eq!(AvatarSources::ALL.len(), 4);
    assert_eq!(AvatarFallback::ALL.len(), 2);
    assert_eq!(ComposerFiles::ALL.len(), 2);
    assert_eq!(BlameSide::ALL.len(), 2);
    assert_eq!(ColumnResize::ALL.len(), 2);
    assert_eq!(SidebarSectionsScope::ALL.len(), 2);
    assert_eq!(ImageTextFallback::ALL.len(), 2);
    assert_eq!(ImagePixelatedFrom::ALL.len(), 3);
    assert_eq!(AutoCrlf::ALL.len(), 4);
    assert_eq!(EolMode::ALL.len(), 3);
    assert_eq!(PullButtonMode::ALL.len(), 3);
    assert_eq!(PushBehavior::ALL.len(), 2);
    assert_eq!(RetentionUnit::ALL.len(), 5);
    assert_eq!(Scope::ALL.len(), 3);
    assert_eq!(PauseMode::ALL.len(), 3);
}

#[test]
fn the_old_theme_spelling_still_parses() {
    let ui: UiConfig = toml::from_str("theme = \"default\"").unwrap();
    assert_eq!(ui.theme_slug(), "system");
    let ui: UiConfig = toml::from_str("theme = \"light\"").unwrap();
    assert_eq!(ui.theme_slug(), "light");
    assert_eq!(UiConfig::default().theme_slug(), "system");
}

#[test]
fn the_old_graph_color_theme_values_still_parse() {
    assert_eq!(UiConfig::default().graph_color_slug(), "theme");
    assert_eq!(UiConfig::default().input_color, "theme");
    for old in ["gitpoulpi", "light", "material", "serie"] {
        let ui: UiConfig = toml::from_str(&format!("graph_color_theme = \"{old}\"")).unwrap();
        assert_eq!(ui.graph_color_slug(), old);
        assert!(
            gitpoulpi_config::theme::lane_palette(old).is_some(),
            "{old}"
        );
    }
}

#[test]
fn the_launch_settings_default_to_off_and_round_trip() {
    let ui = Config::default().ui;
    assert_eq!(ui.launch, "off");
    assert!(ui.launch_command.is_empty());
    let (cfg, _) = Config::from_toml_str(
        "[ui]\nlaunch = \"custom\"\nlaunch_command = \"my-term --cd {repo} {gitpoulpi} -s {session}\"\n",
    )
    .unwrap();
    assert_eq!(cfg.ui.launch, "custom");
    let (again, _) = Config::from_toml_str(&toml::to_string(&cfg).unwrap()).unwrap();
    assert_eq!(again.ui.launch_command, cfg.ui.launch_command);
    let mut cfg = Config::default();
    cfg.set("ui.launch", "kitty").unwrap();
    assert_eq!(cfg.ui.launch, "kitty");
}

#[test]
fn the_tab_folder_settings_default_to_buttons_and_auto_and_round_trip() {
    let ui = Config::default().ui;
    assert_eq!(ui.tab_dir_follow, TabDirFollow::Buttons);
    assert_eq!(ui.terminal_launch, "auto");
    assert!(ui.terminal_launch_command.is_empty());
    let (cfg, _) = Config::from_toml_str(
        "[ui]\ntab_dir_follow = \"auto\"\nterminal_launch = \"custom\"\nterminal_launch_command = \"my-term {repo}\"\n",
    )
    .unwrap();
    assert_eq!(cfg.ui.tab_dir_follow, TabDirFollow::Auto);
    let (again, _) = Config::from_toml_str(&toml::to_string(&cfg).unwrap()).unwrap();
    assert_eq!(again.ui.tab_dir_follow, TabDirFollow::Auto);
    assert_eq!(again.ui.terminal_launch, "custom");
    assert_eq!(again.ui.terminal_launch_command, "my-term {repo}");
    let mut cfg = Config::default();
    cfg.set("ui.tab_dir_follow", "auto").unwrap();
    assert_eq!(cfg.ui.tab_dir_follow, TabDirFollow::Auto);
    assert!(cfg.set("ui.tab_dir_follow", "sometimes").is_err());
}

#[test]
fn an_old_fetch_all_pull_mode_reads_as_merge() {
    let config: gitpoulpi_config::Config = toml::from_str("[git]\npull_mode = \"fetch_all\"\n")
        .expect("an older config with fetch_all still loads");
    assert_eq!(
        config.git.pull_mode,
        gitpoulpi_config::PullButtonMode::Merge
    );
}

#[test]
fn open_in_is_lossless_none_versus_some_and_round_trips() {
    // Absent key: None, and still absent after a save.
    let (cfg, _) = Config::from_toml_str("[ui]\nterminal_launch = \"kitty\"\n").unwrap();
    assert_eq!(cfg.ui.open_in, None);
    assert!(cfg.ui.open_in_default.is_empty());
    let text = toml::to_string(&cfg).unwrap();
    assert!(!text.contains("open_in ") && !text.contains("[[ui.open_in]]"));
    assert_eq!(
        Config::from_toml_str(&text).unwrap().0.ui.open_in,
        None,
        "None must stay None"
    );
    // A list round-trips in order.
    let (cfg, _) = Config::from_toml_str(
        "[ui]\nopen_in_default = \"B\"\n\n[[ui.open_in]]\nlabel = \"A\"\ncommand = \"a ${cwd}\"\n\n[[ui.open_in]]\nlabel = \"B\"\ncommand = \"b --dir '${cwd}'\"\n",
    )
    .unwrap();
    let list = cfg.ui.open_in.clone().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[1], OpenInEntry::new("B", "b --dir '${cwd}'"));
    let (again, _) = Config::from_toml_str(&toml::to_string(&cfg).unwrap()).unwrap();
    assert_eq!(again.ui.open_in, cfg.ui.open_in);
    assert_eq!(again.ui.open_in_default, "B");
    // The default label is settable from a shell; the list is not.
    let mut cfg = Config::default();
    cfg.set("ui.open_in_default", "A").unwrap();
    assert_eq!(cfg.ui.open_in_default, "A");
    assert!(cfg.get("ui.open_in").unwrap().is_null());
    assert!(LIVE_KEYS.contains(&"ui.open_in") && LIVE_KEYS.contains(&"ui.open_in_default"));
}

#[test]
fn set_open_in_in_file_writes_none_some_and_an_empty_list() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "[ui]\nterminal_launch = \"kitty\"\n").unwrap();
    let entries = vec![OpenInEntry::new("A", "a ${cwd}")];
    set_open_in_in_file(&path, Some(&entries)).unwrap();
    let cfg = Config::load(&path).unwrap();
    assert_eq!(cfg.ui.open_in, Some(entries));
    assert_eq!(cfg.ui.terminal_launch, "kitty", "other keys survive");
    set_open_in_in_file(&path, Some(&[])).unwrap();
    assert_eq!(Config::load(&path).unwrap().ui.open_in, Some(vec![]));
    set_open_in_in_file(&path, None).unwrap();
    assert_eq!(Config::load(&path).unwrap().ui.open_in, None);
}

#[test]
fn the_image_view_settings_default_and_round_trip() {
    let ui = Config::default().ui;
    assert!(ui.image_view);
    assert_eq!(ui.image_view_max_mib, 32);
    assert_eq!(ui.image_view_text_fallback, ImageTextFallback::Halfblock);
    assert_eq!(ui.image_view_pixelated_from, ImagePixelatedFrom::X2);
    assert!(ui.image_view_sync);
    assert_eq!(ui.image_editors, None);
    assert!(ui.image_editor_default.is_empty());
    let mut cfg = Config::default();
    cfg.set("ui.image_view", "false").unwrap();
    cfg.set("ui.image_view_max_mib", "8").unwrap();
    cfg.set("ui.image_view_text_fallback", "card").unwrap();
    cfg.set("ui.image_view_pixelated_from", "never").unwrap();
    cfg.set("ui.image_view_sync", "false").unwrap();
    cfg.set("ui.image_editor_default", "GIMP").unwrap();
    assert!(cfg.set("ui.image_view_pixelated_from", "3x").is_err());
    let (again, _) = Config::from_toml_str(&toml::to_string(&cfg).unwrap()).unwrap();
    assert!(!again.ui.image_view);
    assert_eq!(again.ui.image_view_max_mib, 8);
    assert_eq!(again.ui.image_view_text_fallback, ImageTextFallback::Card);
    assert_eq!(
        again.ui.image_view_pixelated_from,
        ImagePixelatedFrom::Never
    );
    assert!(!again.ui.image_view_sync);
    assert_eq!(again.ui.image_editor_default, "GIMP");
}

#[test]
fn image_editors_is_lossless_none_versus_some_and_round_trips() {
    let (cfg, _) = Config::from_toml_str("[ui]\nimage_view = true\n").unwrap();
    assert_eq!(cfg.ui.image_editors, None);
    let text = toml::to_string(&cfg).unwrap();
    assert!(!text.contains("image_editors"), "None stays absent");
    assert_eq!(
        Config::from_toml_str(&text).unwrap().0.ui.image_editors,
        None
    );
    let (cfg, _) = Config::from_toml_str(
        "[ui]\nimage_editor_default = \"B\"\n\n[[ui.image_editors]]\nlabel = \"A\"\ncommand = \"a ${file}\"\n\n[[ui.image_editors]]\nlabel = \"B\"\ncommand = \"b --in '${file}'\"\n",
    )
    .unwrap();
    let list = cfg.ui.image_editors.clone().unwrap();
    assert_eq!(list[1], OpenInEntry::new("B", "b --in '${file}'"));
    let (again, _) = Config::from_toml_str(&toml::to_string(&cfg).unwrap()).unwrap();
    assert_eq!(again.ui.image_editors, cfg.ui.image_editors);
    let (empty, _) = Config::from_toml_str("[ui]\nimage_editors = []\n").unwrap();
    assert_eq!(empty.ui.image_editors, Some(vec![]));
}

#[test]
fn set_image_editors_in_file_writes_none_some_and_an_empty_list() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let entries = vec![OpenInEntry::new("GIMP", "gimp ${file}")];
    set_image_editors_in_file(&path, Some(&entries)).unwrap();
    let (cfg, _) = Config::from_toml_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(cfg.ui.image_editors, Some(entries));
    set_image_editors_in_file(&path, Some(&[])).unwrap();
    let (cfg, _) = Config::from_toml_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(cfg.ui.image_editors, Some(vec![]));
    set_image_editors_in_file(&path, None).unwrap();
    let (cfg, _) = Config::from_toml_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(cfg.ui.image_editors, None);
}

#[test]
fn the_layout_options_default_to_a_bottom_toolbar_and_status_line_and_round_trip() {
    let ui = UiConfig::default();
    assert_eq!(ui.toolbar_position, ToolbarPosition::Bottom);
    assert_eq!(ui.toolbar_align, ToolbarAlign::Start);
    assert!(ui.toolbar_pr_pinned);
    assert_eq!(ui.statusbar_position, StatusbarPosition::Bottom);
    // A config that never heard of them gets the defaults.
    let old: UiConfig = toml::from_str("theme = \"dark\"").unwrap();
    assert_eq!(old.toolbar_position, ToolbarPosition::Bottom);
    assert_eq!(old.statusbar_position, StatusbarPosition::Bottom);
    for pos in ToolbarPosition::ALL {
        for align in ToolbarAlign::ALL {
            for status in StatusbarPosition::ALL {
                let text = format!(
                    "toolbar_position = \"{}\"\ntoolbar_align = \"{}\"\ntoolbar_pr_pinned = false\nstatusbar_position = \"{}\"\n",
                    pos.as_str(),
                    align.as_str(),
                    status.as_str()
                );
                let ui: UiConfig = toml::from_str(&text).unwrap();
                assert_eq!(
                    (ui.toolbar_position, ui.toolbar_align, ui.statusbar_position),
                    (*pos, *align, *status)
                );
                assert!(!ui.toolbar_pr_pinned);
                let again: UiConfig = toml::from_str(&toml::to_string(&ui).unwrap()).unwrap();
                assert_eq!(again.toolbar_position, *pos);
                assert_eq!(again.toolbar_align, *align);
                assert_eq!(again.statusbar_position, *status);
            }
        }
    }
    assert!(ToolbarPosition::Left.is_vertical() && ToolbarPosition::Right.is_vertical());
    assert!(!ToolbarPosition::Top.is_vertical() && !ToolbarPosition::Bottom.is_vertical());
    assert!(toml::from_str::<UiConfig>("toolbar_position = \"middle\"").is_err());
}

#[test]
fn the_tab_bar_defaults_to_the_top_and_every_position_round_trips() {
    assert_eq!(UiConfig::default().tabs_position, TabsPosition::Top);
    let old: UiConfig = toml::from_str("theme = \"dark\"").unwrap();
    assert_eq!(old.tabs_position, TabsPosition::Top);
    for pos in TabsPosition::ALL {
        let ui: UiConfig =
            toml::from_str(&format!("tabs_position = \"{}\"\n", pos.as_str())).unwrap();
        assert_eq!(ui.tabs_position, *pos);
        let again: UiConfig = toml::from_str(&toml::to_string(&ui).unwrap()).unwrap();
        assert_eq!(again.tabs_position, *pos);
    }
    assert!(toml::from_str::<UiConfig>("tabs_position = \"right\"").is_err());
}

#[test]
fn sequences_are_left_out_while_empty_and_round_trip_when_present() {
    use gitpoulpi_config::sequences::{SequenceDef, StepSpec};
    let text = toml::to_string(&Config::default()).unwrap();
    assert!(!text.contains("sequences"), "{text}");
    let config = Config {
        sequences: vec![SequenceDef {
            id: "backup".into(),
            name: "Backup".into(),
            steps: vec![StepSpec::new("stage_all"), StepSpec::new("push")],
            ..SequenceDef::default()
        }],
        ..Config::default()
    };
    let text = toml::to_string(&config).unwrap();
    let (again, _) = Config::from_toml_str(&text).unwrap();
    assert_eq!(again.sequences, config.sequences);
    assert_eq!(toml::to_string(&again).unwrap(), text);
    assert!(LIVE_KEYS.contains(&"sequences"));
}
