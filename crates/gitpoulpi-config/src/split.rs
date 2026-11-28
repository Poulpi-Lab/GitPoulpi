//! Which file of the config directory owns which setting.
//!
//! Settings live in up to three files of one directory: `config.toml`
//! (common to every interface), `tui.toml` (the terminal UI's interface
//! settings) and `desktop.toml` (the desktop UI's, a later step). This module
//! is the read path of the split: the classification of every interface key
//! ([`KEY_HOME`]), the pure function that derives a `tui.toml` from a
//! `config.toml` ([`derive_tui_document`]), the merge the loader applies
//! ([`merge_documents`]) and the no-clobber creation of the file
//! ([`ensure_tui_file`], not called by anything yet).
//!
//! Design rules, so the next reader does not have to rediscover them:
//!
//! * **Copy, never move.** `config.toml` is never modified by the split, so an
//!   older build sharing the directory keeps reading its full file.
//! * **Without a `tui.toml` nothing changes**: the loader reads `config.toml`
//!   alone, exactly as before the split existed.
//! * **With a `tui.toml`, it owns every key it can own.** The `[ui]` keys
//!   homed in [`Home::Tui`] / [`Home::PerInterface`] (and any unknown `[ui]`
//!   key), the whole `[keybindings]` table and `merge.side_by_side_min_width`
//!   are read from it only. A key it does not say is the built-in default,
//!   never the value `config.toml` still carries: falling back would bring a
//!   reset key back to life and make `source_of` lie.
//! * **`Config` stays the terminal UI view** (common plus TUI), and `ui.*`
//!   keeps meaning the terminal UI.

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::{ConfigPaths, Error, Result};

/// A frontend with settings of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interface {
    /// The terminal UI: `tui.toml`.
    Tui,
    /// The desktop UI: `desktop.toml`.
    Desktop,
}

impl Interface {
    /// The file name of this interface's settings in the config directory.
    pub fn file_name(self) -> &'static str {
        match self {
            Interface::Tui => TUI_FILE_NAME,
            Interface::Desktop => DESKTOP_FILE_NAME,
        }
    }
}

/// `tui.toml`.
pub const TUI_FILE_NAME: &str = "tui.toml";
/// `desktop.toml`.
pub const DESKTOP_FILE_NAME: &str = "desktop.toml";

/// Where a key lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Home {
    /// `config.toml` only: one answer per machine.
    Common,
    /// `tui.toml` only: terminal capabilities and cell geometry.
    Tui,
    /// One value per interface: in `tui.toml` and in `desktop.toml`.
    PerInterface,
}

use Home::{Common, PerInterface, Tui};

/// Every key of the tables that hold interface settings (`[ui]`,
/// `[keybindings]`, `[merge]`), with its home. A key of `[ui]` / `[merge]`
/// the serialized default config has and this table lacks fails
/// `every_interface_key_is_classified`, so a new field cannot ship
/// unclassified. `keybindings` stands for the whole table (it has no fixed
/// keys).
pub const KEY_HOME: &[(&str, Home)] = &[
    // Language and vocabulary.
    ("ui.language", PerInterface),
    ("ui.translate_git_terms", PerInterface),
    ("ui.git_terms_english", PerInterface),
    // Look (the installation `LOOK_KEYS`).
    ("ui.theme", PerInterface),
    ("ui.graph_color_theme", PerInterface),
    ("ui.graph_colorization", PerInterface),
    ("ui.graph_lane_background", PerInterface),
    ("ui.graph_row_tint_strength", PerInterface),
    // Layout.
    ("ui.toolbar_position", PerInterface),
    ("ui.toolbar_align", PerInterface),
    ("ui.toolbar_pr_pinned", PerInterface),
    ("ui.statusbar_position", PerInterface),
    ("ui.tabs_position", PerInterface),
    ("ui.dialog_geometry", PerInterface),
    // View preferences.
    ("ui.ref_tree", PerInterface),
    ("ui.graph_show_stashes", PerInterface),
    ("ui.ref_connector", PerInterface),
    ("ui.log_wip_counts_in_message", PerInterface),
    ("ui.reveal_on_ref_click", PerInterface),
    ("ui.tree_folders_first", PerInterface),
    ("ui.graph_ghost_ref", PerInterface),
    ("ui.hide_finished_pr_chip", PerInterface),
    ("ui.composer_files", PerInterface),
    ("ui.blame_side", PerInterface),
    ("ui.column_resize", PerInterface),
    ("ui.log_columns_layout", PerInterface),
    ("ui.tab_dir_follow", PerInterface),
    // Diff.
    ("ui.diff_layout", PerInterface),
    ("ui.diff_hunks_only", PerInterface),
    ("ui.diff_show_whitespace", PerInterface),
    ("ui.diff_word_wrap", PerInterface),
    ("ui.diff_intraline", PerInterface),
    ("ui.minimap", PerInterface),
    // Images.
    ("ui.image_view", PerInterface),
    ("ui.image_view_max_mib", PerInterface),
    ("ui.image_view_max_unit", PerInterface),
    ("ui.image_view_pixelated_from", PerInterface),
    ("ui.image_view_sync", PerInterface),
    // Avatars (`avatar_sources` is a privacy choice: common).
    ("ui.avatars", PerInterface),
    ("ui.show_avatars", PerInterface),
    ("ui.avatar_fallback", PerInterface),
    // Dates follow the language.
    ("ui.date_locale", PerInterface),
    ("ui.date_format", PerInterface),
    ("ui.date_time_format", PerInterface),
    ("ui.date_verbose_format", PerInterface),
    // A display word; the CLI output stays English.
    ("ui.pr_terminology", PerInterface),
    // Columns and sidebar.
    ("ui.log_columns_hidden", PerInterface),
    ("ui.log_columns_order", PerInterface),
    ("ui.commit_column_columns_visible", PerInterface),
    ("ui.commit_column_columns_order", PerInterface),
    ("ui.sidebar_sections_hidden", PerInterface),
    ("ui.sidebar_sections_scope", PerInterface),
    // Each window attaches to the viewer registry.
    ("ui.allow_agent_view_control", PerInterface),
    // `gitpoulpi launch` reads it for the interface it launches.
    ("ui.reuse_window", PerInterface),
    // Terminal capabilities.
    ("ui.mouse_enabled", Tui),
    ("ui.graph_renderer", Tui),
    ("ui.image_protocol", Tui),
    ("ui.kitty_keyboard", Tui),
    ("ui.color_depth", Tui),
    ("ui.text_cursor", Tui),
    ("ui.image_view_text_fallback", Tui),
    // Measured in cells or rows.
    ("ui.blame_width", Tui),
    ("ui.mouse_scroll_lines", Tui),
    ("ui.log_column_widths", Tui),
    ("ui.log_column_widths_scroll", Tui),
    ("ui.commit_column_columns_widths", Tui),
    // They start the terminal UI inside a terminal.
    ("ui.launch", Tui),
    ("ui.launch_command", Tui),
    ("ui.launch_terminal", Tui),
    ("ui.relaunch_in_terminal", Tui),
    // A fix for terminal palettes.
    ("ui.input_color", Tui),
    ("ui.input_color_custom", Tui),
    // Legacy: `diff_view_mode` is folded into `diff_layout` / `diff_hunks_only`
    // while the `tui.toml` is derived (it is never serialized, so the
    // exhaustiveness test cannot see it).
    ("ui.date_word_format", Tui),
    ("ui.diff_view_mode", Tui),
    // External tools of this machine.
    ("ui.terminal_launch", Common),
    ("ui.terminal_launch_command", Common),
    ("ui.open_in", Common),
    ("ui.open_in_default", Common),
    ("ui.image_editors", Common),
    ("ui.image_editor_default", Common),
    // Installation identity: `installation set`, `apply_handover` and
    // `dev_mode` read them from `config.toml`.
    ("ui.logo_color", Common),
    ("ui.command_alias", Common),
    ("ui.command_alias_add", Common),
    ("ui.launcher_build", Common),
    // Which network services are contacted: one answer per machine.
    ("ui.avatar_sources", Common),
    // `log --format mermaid` in the CLI uses them as defaults.
    ("ui.mermaid_detail", Common),
    ("ui.mermaid_orientation", Common),
    ("ui.mermaid_fence", Common),
    ("ui.mermaid_subject_width", Common),
    // Measured in cells.
    ("merge.side_by_side_min_width", Tui),
    // The whole table; each interface has its own action set.
    ("keybindings", PerInterface),
];

/// The home `key` is classified under, `None` for a key the table does not
/// know. A dotted key under `keybindings` is classified as the table is.
pub fn classified(key: &str) -> Option<Home> {
    if key == "keybindings" || key.starts_with("keybindings.") {
        return Some(PerInterface);
    }
    KEY_HOME.iter().find(|(k, _)| *k == key).map(|(_, h)| *h)
}

/// The home of a dotted key. Anything outside `[ui]`, `[keybindings]` and
/// `[merge]` is [`Home::Common`]; an *unknown* `ui.*` key is [`Home::Tui`], so
/// a key typed by hand or written by a newer build travels with the
/// interface file instead of being dropped.
pub fn home_of(key: &str) -> Home {
    if let Some(home) = classified(key) {
        return home;
    }
    if key.starts_with("ui.") {
        Tui
    } else {
        Common
    }
}

/// `<dir of config_file>/tui.toml`, only for a file named `config.toml`: the
/// split is a property of the config directory, not of any TOML a caller
/// happens to load.
pub(crate) fn tui_sibling(config_file: &Path) -> Option<PathBuf> {
    (config_file.file_name()? == "config.toml").then(|| config_file.with_file_name(TUI_FILE_NAME))
}

fn empty_table() -> toml::Value {
    toml::Value::Table(toml::map::Map::new())
}

/// The content of `tui.toml` as a pure function of a `config.toml` document:
/// the `[ui]` keys homed in the terminal UI (legacy keys folded in, unknown
/// keys copied too), the `[keybindings]` table and
/// `merge.side_by_side_min_width`. Idempotent:
/// `derive(derive(d)) == derive(d)`.
pub fn derive_tui_document(config_doc: &toml::Value) -> toml::Value {
    let mut out = toml::map::Map::new();
    let Some(src) = config_doc.as_table() else {
        return empty_table();
    };

    if let Some(ui) = src.get("ui").and_then(|v| v.as_table()) {
        let mut kept = toml::map::Map::new();
        for (key, value) in ui {
            if home_of(&format!("ui.{key}")) != Common {
                kept.insert(key.clone(), value.clone());
            }
        }
        // Fold the legacy `diff_view_mode` exactly as `UiConfig::migrate_legacy`
        // does: it fills `diff_layout` / `diff_hunks_only` only where the
        // file does not say them.
        if let Some(raw) = kept.remove("diff_view_mode") {
            if let Ok(legacy) = raw.try_into::<crate::LegacyDiffViewMode>() {
                if !ui.contains_key("diff_layout") {
                    if let Ok(v) = toml::Value::try_from(legacy.layout()) {
                        kept.insert("diff_layout".into(), v);
                    }
                }
                if !ui.contains_key("diff_hunks_only") {
                    kept.insert(
                        "diff_hunks_only".into(),
                        toml::Value::Boolean(legacy.hunks_only()),
                    );
                }
            }
        }
        if !kept.is_empty() {
            out.insert("ui".into(), toml::Value::Table(kept));
        }
    }

    if let Some(keybindings) = src.get("keybindings").filter(|v| v.is_table()) {
        out.insert("keybindings".into(), keybindings.clone());
    }

    if let Some(width) = src
        .get("merge")
        .and_then(|m| m.get("side_by_side_min_width"))
    {
        let mut merge = toml::map::Map::new();
        merge.insert("side_by_side_min_width".into(), width.clone());
        out.insert("merge".into(), toml::Value::Table(merge));
    }

    toml::Value::Table(out)
}

/// The document the loader deserializes when a `tui.toml` exists: the common
/// document with every key the terminal UI file can own replaced by what the
/// `tui.toml` says (nothing, when it says nothing). Keys `tui.toml` carries
/// but does not own (a common `[ui]` key typed there) are ignored:
/// `config.toml` owns them.
pub fn merge_documents(common_doc: &toml::Value, tui_doc: &toml::Value) -> toml::Value {
    let mut out = match common_doc {
        toml::Value::Table(t) => t.clone(),
        _ => toml::map::Map::new(),
    };
    let tui = tui_doc.as_table();

    // `[keybindings]` is one table the file owns whole.
    match tui
        .and_then(|t| t.get("keybindings"))
        .filter(|v| v.is_table())
    {
        Some(theirs) => {
            out.insert("keybindings".into(), theirs.clone());
        }
        None => {
            out.remove("keybindings");
        }
    }

    for table in ["ui", "merge"] {
        let owned = |key: &str| home_of(&format!("{table}.{key}")) != Common;
        let theirs = tui.and_then(|t| t.get(table)).and_then(|v| v.as_table());
        let mut merged = out
            .get(table)
            .and_then(|v| v.as_table())
            .cloned()
            .unwrap_or_default();
        merged.retain(|key, _| !owned(key));
        if let Some(theirs) = theirs {
            for (key, value) in theirs {
                if owned(key) {
                    merged.insert(key.clone(), value.clone());
                }
            }
        }
        if merged.is_empty() {
            out.remove(table);
        } else {
            out.insert(table.into(), toml::Value::Table(merged));
        }
    }
    toml::Value::Table(out)
}

/// What the loader reads: the common document, merged with the sibling
/// `tui.toml` when there is one. `None` when neither file exists. The second
/// element is the `tui.toml` that was read.
pub(crate) fn read_merged(config_file: &Path) -> Result<Option<(toml::Value, Option<PathBuf>)>> {
    let tui_path = tui_sibling(config_file).filter(|p| p.exists());
    let common = if config_file.exists() {
        Some(toml::from_str::<toml::Value>(&std::fs::read_to_string(
            config_file,
        )?)?)
    } else {
        None
    };
    match (common, tui_path) {
        (None, None) => Ok(None),
        (Some(common), None) => Ok(Some((common, None))),
        (common, Some(tui_path)) => {
            let tui: toml::Value = toml::from_str(&std::fs::read_to_string(&tui_path)?)?;
            let common = common.unwrap_or_else(empty_table);
            Ok(Some((merge_documents(&common, &tui), Some(tui_path))))
        }
    }
}

/// Creates `tui.toml` from `config.toml` unless it exists. Returns whether
/// this call created it.
///
/// The file is written to a temporary name and hard-linked into place, which
/// refuses to replace an existing file, so a racing process that lost never
/// overwrites a `tui.toml` that already received edits. Where hard links are
/// not available it falls back to `create_new`. `config.toml` is only read.
///
/// Not called from anywhere yet: the writers and the frontends wire it in
/// later steps.
pub fn ensure_tui_file(paths: &ConfigPaths) -> Result<bool> {
    let target = paths.tui_file();
    if target.exists() {
        return Ok(false);
    }
    let config_file = paths.config_file();
    let common = if config_file.exists() {
        toml::from_str::<toml::Value>(&std::fs::read_to_string(&config_file)?)?
    } else {
        empty_table()
    };
    let body = toml::to_string_pretty(&derive_tui_document(&common))?;
    if let Some(parent) = target.parent() {
        crate::secure_fs::create_private_dir_all(parent)?;
    }
    create_new_file(&target, body.as_bytes()).map_err(Error::Io)
}

fn open_new(path: &Path) -> std::io::Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

fn create_new_file(target: &Path, bytes: &[u8]) -> std::io::Result<bool> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let tmp = target.with_file_name(format!(
        ".{TUI_FILE_NAME}.{}.{}.tmp",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let staged = (|| {
        let mut file = open_new(&tmp)?;
        file.write_all(bytes)?;
        file.flush()
    })();
    if let Err(err) = staged {
        let _ = std::fs::remove_file(&tmp);
        return Err(err);
    }
    let linked = std::fs::hard_link(&tmp, target);
    let _ = std::fs::remove_file(&tmp);
    match linked {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(_) => {
            // No hard links here (some network or FAT file systems).
            match open_new(target) {
                Ok(mut file) => {
                    let written = file.write_all(bytes).and_then(|()| file.flush());
                    if let Err(err) = written {
                        let _ = std::fs::remove_file(target);
                        return Err(err);
                    }
                    Ok(true)
                }
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
                Err(err) => Err(err),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    fn doc(raw: &str) -> toml::Value {
        toml::from_str(raw).unwrap()
    }

    /// A new field of `[ui]` or `[merge]` fails here until it is classified in
    /// [`KEY_HOME`].
    #[test]
    fn every_interface_key_is_classified() {
        let json = serde_json::to_value(Config::default()).unwrap();
        let mut missing = Vec::new();
        for table in ["ui", "merge"] {
            for key in json[table].as_object().unwrap().keys() {
                let full = format!("{table}.{key}");
                if classified(&full).is_none() {
                    missing.push(full);
                }
            }
        }
        assert!(
            classified("keybindings").is_some(),
            "[keybindings] is unclassified"
        );
        assert!(
            missing.is_empty(),
            "classify these keys in split::KEY_HOME: {missing:?}"
        );
    }

    /// The tables that are interface settings are exactly the three this
    /// module knows: a new top-level table of the schema holding interface
    /// settings must be added to the merge.
    #[test]
    fn only_the_known_tables_hold_interface_keys() {
        for (key, _) in KEY_HOME {
            let table = key.split('.').next().unwrap();
            assert!(["ui", "merge", "keybindings"].contains(&table), "{key}");
        }
    }

    #[test]
    fn every_classified_key_exists_in_the_schema() {
        let json = serde_json::to_value(Config::default()).unwrap();
        // Read but never serialized.
        let legacy_only = ["ui.diff_view_mode"];
        for (key, _) in KEY_HOME {
            if *key == "keybindings" || legacy_only.contains(key) {
                continue;
            }
            let (table, field) = key.split_once('.').unwrap();
            assert!(
                json[table].get(field).is_some(),
                "{key} is classified but the schema has no such key"
            );
        }
    }

    #[test]
    fn the_table_has_no_duplicates() {
        let mut seen = std::collections::BTreeSet::new();
        for (key, _) in KEY_HOME {
            assert!(seen.insert(*key), "{key} is classified twice");
        }
    }

    #[test]
    fn home_of_routes_known_unknown_and_foreign_keys() {
        assert_eq!(home_of("ui.theme"), PerInterface);
        assert_eq!(home_of("ui.mouse_enabled"), Tui);
        assert_eq!(home_of("ui.logo_color"), Common);
        assert_eq!(home_of("ui.some_future_key"), Tui);
        assert_eq!(home_of("keybindings.global.quit"), PerInterface);
        assert_eq!(home_of("merge.side_by_side_min_width"), Tui);
        assert_eq!(home_of("general.undo_depth"), Common);
        assert_eq!(home_of("installation.primary"), Common);
    }

    const SOURCE: &str = r#"
[general]
undo_depth = 7

[ui]
theme = "dusk"
mouse_enabled = false
logo_color = "red"
diff_view_mode = "split"
mystery = 3

[keybindings.global]
quit = ["ctrl+q"]

[merge]
side_by_side_min_width = 90
"#;

    #[test]
    fn derive_keeps_interface_keys_folds_legacy_and_copies_unknown() {
        let tui = derive_tui_document(&doc(SOURCE));
        let ui = tui["ui"].as_table().unwrap();
        assert_eq!(ui["theme"].as_str(), Some("dusk"));
        assert_eq!(ui["mouse_enabled"].as_bool(), Some(false));
        assert_eq!(ui["mystery"].as_integer(), Some(3));
        assert_eq!(ui["diff_layout"].as_str(), Some("split"));
        assert_eq!(ui["diff_hunks_only"].as_bool(), Some(true));
        assert!(!ui.contains_key("diff_view_mode"));
        assert!(
            !ui.contains_key("logo_color"),
            "common stays in config.toml"
        );
        assert_eq!(
            tui["keybindings"]["global"]["quit"][0].as_str(),
            Some("ctrl+q")
        );
        assert_eq!(
            tui["merge"]["side_by_side_min_width"].as_integer(),
            Some(90)
        );
        assert!(tui.get("general").is_none());
    }

    #[test]
    fn derive_does_not_override_a_layout_the_file_states() {
        let tui = derive_tui_document(&doc(
            "[ui]\ndiff_view_mode = \"split\"\ndiff_layout = \"inline\"\n",
        ));
        assert_eq!(tui["ui"]["diff_layout"].as_str(), Some("inline"));
    }

    #[test]
    fn derive_is_idempotent() {
        let once = derive_tui_document(&doc(SOURCE));
        assert_eq!(derive_tui_document(&once), once);
        let merged = merge_documents(&doc(SOURCE), &once);
        assert_eq!(derive_tui_document(&merged), once);
    }

    #[test]
    fn derive_of_nothing_is_an_empty_document() {
        assert_eq!(derive_tui_document(&doc("")), empty_table());
        assert_eq!(
            derive_tui_document(&doc("[general]\nundo_depth = 3\n")),
            empty_table()
        );
    }

    #[test]
    fn merge_lets_tui_toml_own_its_keys_and_leaves_common_alone() {
        let common = doc(SOURCE);
        let tui = doc(
            "[ui]\ntheme = \"night\"\nlogo_color = \"blue\"\n\n[keybindings.global]\nquit = [\"q\"]\n",
        );
        let merged = merge_documents(&common, &tui);
        assert_eq!(merged["ui"]["theme"].as_str(), Some("night"));
        // `tui.toml` is silent on mouse_enabled: the default applies, not the
        // old `config.toml` value.
        assert!(merged["ui"].get("mouse_enabled").is_none());
        assert!(merged["ui"].get("mystery").is_none());
        assert_eq!(merged["ui"]["logo_color"].as_str(), Some("red"));
        assert_eq!(
            merged["keybindings"]["global"]["quit"][0].as_str(),
            Some("q")
        );
        assert!(merged.get("merge").is_none());
        assert_eq!(merged["general"]["undo_depth"].as_integer(), Some(7));
    }

    #[test]
    fn merging_the_derived_document_gives_back_the_original() {
        let common = doc(
            "[general]\nundo_depth = 2\n[ui]\ntheme = \"x\"\nmystery = 1\nlogo_color = \"red\"\n[keybindings.a]\nb = [\"c\"]\n[merge]\nside_by_side_min_width = 5\n",
        );
        assert_eq!(
            merge_documents(&common, &derive_tui_document(&common)),
            common
        );
    }

    fn paths_with(config: Option<&str>) -> (tempfile::TempDir, ConfigPaths) {
        let dir = tempfile::tempdir().unwrap();
        if let Some(body) = config {
            std::fs::write(dir.path().join("config.toml"), body).unwrap();
        }
        let paths = ConfigPaths::for_dir(dir.path());
        (dir, paths)
    }

    #[test]
    fn ensure_creates_the_derived_file_and_never_touches_config_toml() {
        let (dir, paths) = paths_with(Some(SOURCE));
        assert!(ensure_tui_file(&paths).unwrap());
        let written: toml::Value =
            toml::from_str(&std::fs::read_to_string(paths.tui_file()).unwrap()).unwrap();
        assert_eq!(written, derive_tui_document(&doc(SOURCE)));
        assert_eq!(
            std::fs::read_to_string(dir.path().join("config.toml")).unwrap(),
            SOURCE
        );
        // No temporary file is left behind.
        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty());
    }

    #[test]
    fn ensure_is_idempotent_and_keeps_later_edits() {
        let (_dir, paths) = paths_with(Some(SOURCE));
        assert!(ensure_tui_file(&paths).unwrap());
        std::fs::write(paths.tui_file(), "[ui]\ntheme = \"edited\"\n").unwrap();
        assert!(!ensure_tui_file(&paths).unwrap());
        assert_eq!(
            std::fs::read_to_string(paths.tui_file()).unwrap(),
            "[ui]\ntheme = \"edited\"\n"
        );
    }

    #[test]
    fn ensure_without_a_config_file_creates_an_empty_one() {
        let (_dir, paths) = paths_with(None);
        assert!(ensure_tui_file(&paths).unwrap());
        assert_eq!(std::fs::read_to_string(paths.tui_file()).unwrap(), "");
    }

    #[test]
    fn ensure_refuses_a_config_file_that_does_not_parse() {
        let (_dir, paths) = paths_with(Some("[ui\n"));
        assert!(ensure_tui_file(&paths).is_err());
        assert!(!paths.tui_file().exists());
    }

    #[test]
    fn two_racing_creators_produce_one_file_and_one_winner() {
        for _ in 0..20 {
            let (_dir, paths) = paths_with(Some(SOURCE));
            let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
            let handles: Vec<_> = (0..2)
                .map(|_| {
                    let (paths, barrier) = (paths.clone(), barrier.clone());
                    std::thread::spawn(move || {
                        barrier.wait();
                        ensure_tui_file(&paths).unwrap()
                    })
                })
                .collect();
            let created: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();
            assert_eq!(created.iter().filter(|c| **c).count(), 1, "{created:?}");
            let written: toml::Value =
                toml::from_str(&std::fs::read_to_string(paths.tui_file()).unwrap()).unwrap();
            assert_eq!(written, derive_tui_document(&doc(SOURCE)));
        }
    }

    #[test]
    fn creation_never_replaces_an_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("tui.toml");
        std::fs::write(&target, "edited").unwrap();
        assert!(!create_new_file(&target, b"fresh").unwrap());
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "edited");
        // The `create_new` fallback refuses too.
        assert!(
            matches!(open_new(&target), Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists)
        );
    }

    #[test]
    fn only_a_file_named_config_toml_has_a_sibling() {
        assert_eq!(
            tui_sibling(Path::new("/x/config.toml")),
            Some(PathBuf::from("/x/tui.toml"))
        );
        assert_eq!(tui_sibling(Path::new("/x/other.toml")), None);
    }
}
