//! Writers and the resolved view of `desktop.toml` (see `crate::split` for the
//! three-file design and `schema::desktop_ui` for the schema).
//!
//! Every writer is a **sparse** read-modify-write of the document, like the
//! ones for `config.toml`: it validates the edited document as a
//! [`DesktopConfig`] before anything is written, writes atomically (private
//! temporary file, then rename), keeps keys it does not know, and never
//! touches `config.toml` or `tui.toml`. A first write creates the file without
//! clobbering one a racing process just created. Writes of this process are
//! serialized by the same lock as the other config writers.
//!
//! Comments are not preserved (the document is re-serialized from
//! `toml::Value`), exactly as for `config.toml`.
//!
//! The desktop calls these through [`DesktopConfig`]'s associated functions:
//! [`DesktopConfig::set_in_file`], [`DesktopConfig::set_window`],
//! [`DesktopConfig::set_panes`], [`DesktopConfig::set_column_widths`], ...

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::schema::{coerce_desktop, DesktopColumnWidths, DesktopConfig, DesktopUiConfig};
use crate::{insert_dotted, json_to_toml, remove_dotted};
use crate::{ConfigPaths, Error, Result};

/// The `desktop.ui.*` keys some desktop component reads and an environment
/// variable (`GITPOULPI_DESKTOP_UI_<KEY>`, the same mechanical rule as
/// `GITPOULPI_UI_*`) may therefore override: the per-interface keys of
/// [`crate::LIVE_KEYS`]. A test keeps the two lists in step.
pub const DESKTOP_LIVE_KEYS: &[&str] = &[
    "desktop.ui.language",
    "desktop.ui.theme",
    "desktop.ui.translate_git_terms",
    "desktop.ui.git_terms_english",
    "desktop.ui.date_format",
    "desktop.ui.date_locale",
    "desktop.ui.date_time_format",
    "desktop.ui.date_verbose_format",
    "desktop.ui.graph_lane_background",
    "desktop.ui.graph_row_tint_strength",
    "desktop.ui.toolbar_position",
    "desktop.ui.toolbar_align",
    "desktop.ui.dialog_geometry",
    "desktop.ui.toolbar_pr_pinned",
    "desktop.ui.statusbar_position",
    "desktop.ui.tabs_position",
    "desktop.ui.graph_color_theme",
    "desktop.ui.ref_tree",
    "desktop.ui.graph_show_stashes",
    "desktop.ui.ref_connector",
    "desktop.ui.log_wip_counts_in_message",
    "desktop.ui.reveal_on_ref_click",
    "desktop.ui.tree_folders_first",
    "desktop.ui.graph_ghost_ref",
    "desktop.ui.allow_agent_view_control",
    "desktop.ui.tab_dir_follow",
    "desktop.ui.reuse_window",
    "desktop.ui.image_view",
    "desktop.ui.image_view_max_mib",
    "desktop.ui.image_view_max_unit",
    "desktop.ui.image_view_pixelated_from",
    "desktop.ui.image_view_sync",
    "desktop.ui.show_avatars",
    "desktop.ui.avatars",
    "desktop.ui.avatar_fallback",
    "desktop.ui.composer_files",
    "desktop.ui.sidebar_sections_scope",
    "desktop.ui.blame_side",
    "desktop.ui.column_resize",
    "desktop.ui.log_columns_layout",
    "desktop.ui.diff_layout",
    "desktop.ui.pr_terminology",
    "desktop.ui.hide_finished_pr_chip",
    "desktop.ui.diff_hunks_only",
    "desktop.ui.diff_show_whitespace",
    "desktop.ui.diff_word_wrap",
    "desktop.ui.diff_intraline",
    "desktop.ui.minimap",
];

/// The prefix every `desktop.toml` key carries in `config get|set`.
pub const KEY_PREFIX: &str = "desktop.";

/// `key` without [`KEY_PREFIX`] when it names a `desktop.toml` key.
pub fn strip_prefix(key: &str) -> Option<&str> {
    key.strip_prefix(KEY_PREFIX)
}

/// Whether some desktop component reads `key` (`desktop.ui.theme`): the
/// desktop reads its own window, pane and column keys as well.
pub fn key_is_implemented(key: &str) -> bool {
    DESKTOP_LIVE_KEYS.contains(&key)
        || key.starts_with("desktop.keybindings.")
        || strip_prefix(key).is_some_and(|k| {
            k.strip_prefix("ui.")
                .is_some_and(|k| crate::DESKTOP_ONLY_KEYS.iter().any(|o| k.starts_with(o)))
        })
}

fn bad(key: &str) -> impl FnOnce(toml::de::Error) -> Error {
    let key = key.to_string();
    move |err| Error::BadValue {
        key,
        value: "<value>".to_string(),
        expected: err.to_string(),
    }
}

/// The one sparse read-modify-write of `desktop.toml`. `edit` may run twice
/// when a racing process created the file first. `clamp`: out-of-range pixel
/// values are pulled into range instead of refused.
fn edit_document(
    path: &Path,
    label: &str,
    clamp: bool,
    edit: impl Fn(&mut toml::Value) -> Result<()>,
) -> Result<DesktopConfig> {
    let _guard = crate::split::lock_writes();
    for _ in 0..2 {
        let existed = path.exists();
        let mut doc = crate::split::read_doc(path)?;
        edit(&mut doc)?;
        let mut config: DesktopConfig = doc.clone().try_into().map_err(bad(label))?;
        if !clamp {
            config.validate()?;
        }
        config.sanitize();
        let body = toml::to_string_pretty(&doc)?;
        if let Some(parent) = path.parent() {
            crate::secure_fs::create_private_dir_all(parent)?;
        }
        if existed {
            crate::secure_fs::write_private(path, body.as_bytes())?;
            return Ok(config);
        }
        // No clobber: a process that created the file meanwhile wins, and the
        // edit is replayed on top of what it wrote.
        if crate::split::create_new_file(path, body.as_bytes())? {
            return Ok(config);
        }
    }
    Err(Error::Io(std::io::Error::other(
        "desktop.toml kept changing while it was written",
    )))
}

fn int(v: u32) -> toml::Value {
    toml::Value::Integer(i64::from(v))
}

fn table(entries: Vec<(&str, toml::Value)>) -> toml::Value {
    toml::Value::Table(
        entries
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

fn widths_value(w: DesktopColumnWidths) -> toml::Value {
    table(vec![
        ("ref_w", int(w.ref_w)),
        ("graph_w", int(w.graph_w)),
        ("author_w", int(w.author_w)),
        ("date_w", int(w.date_w)),
        ("sha_w", int(w.sha_w)),
    ])
}

/// Whether the document at `path` says `key` (dotted).
fn file_says(path: &Path, key: &str) -> Result<bool> {
    let doc = crate::split::read_doc(path)?;
    let mut cursor = &doc;
    for segment in key.split('.') {
        match cursor.get(segment) {
            Some(next) => cursor = next,
            None => return Ok(false),
        }
    }
    Ok(true)
}

impl DesktopConfig {
    /// Sets one dotted key relative to `desktop.toml` (`ui.theme`,
    /// `ui.column_widths_px.ref_w`, `keybindings.<ctx>.<action>`) from text,
    /// as `gitpoulpi config set desktop.ui.theme ...` does. A value outside a
    /// pixel range is refused. Returns the value as stored.
    pub fn set_in_file(paths: &ConfigPaths, key: &str, value: &str) -> Result<serde_json::Value> {
        if let Some(rest) = key.strip_prefix("keybindings.") {
            let (ctx, action) = rest
                .split_once('.')
                .ok_or_else(|| Error::UnknownKey(key.to_string()))?;
            let chords = crate::keybindings::chords_from_cli(value);
            Self::set_keybinding(paths, ctx, action, &chords)?;
            return Ok(serde_json::json!(chords));
        }
        if key.split('.').next() == Some("repos") {
            return Err(Error::UnknownKey(key.to_string()));
        }
        let schema_value = DesktopConfig::default().get(key)?;
        let coerced = coerce_desktop(key, &schema_value, value)?;
        let config = edit_document(&paths.desktop_file(), key, false, |doc| {
            insert_dotted(doc, key, json_to_toml(coerced.clone()))
        })
        .map_err(|err| match err {
            Error::BadValue {
                key: k,
                value: v,
                expected,
            } => Error::BadValue {
                key: k,
                value: if v == "<value>" { value.to_string() } else { v },
                expected,
            },
            other => other,
        })?;
        config.get(key)
    }

    /// Writes a list of strings (`ui.log_columns_hidden`, ...); an empty list
    /// is written as `[]` (the user chose "none").
    pub fn set_list_in_file(paths: &ConfigPaths, key: &str, items: &[String]) -> Result<()> {
        if !matches!(
            DesktopConfig::default().get(key)?,
            serde_json::Value::Array(_)
        ) {
            return Err(Error::UnsettableKey {
                key: key.to_string(),
                kind: "value",
            });
        }
        let value = toml::Value::Array(items.iter().cloned().map(toml::Value::String).collect());
        edit_document(&paths.desktop_file(), key, false, |doc| {
            insert_dotted(doc, key, value.clone())
        })
        .map(|_| ())
    }

    /// Removes `key` (back to its default); a no-op when the file does not
    /// say it. An emptied table is dropped too.
    pub fn unset_in_file(paths: &ConfigPaths, key: &str) -> Result<()> {
        DesktopConfig::default().get(key)?;
        Self::remove_if_present(paths, key)
    }

    /// Remembers the window: size (clamped), position (`None` leaves the
    /// window manager to place it) and whether it is maximized.
    pub fn set_window(
        paths: &ConfigPaths,
        size: (u32, u32),
        position: Option<(i32, i32)>,
        maximized: bool,
    ) -> Result<()> {
        let mut ui = DesktopUiConfig {
            window_width: size.0,
            window_height: size.1,
            window_x: position.map(|p| p.0),
            window_y: position.map(|p| p.1),
            ..Default::default()
        };
        ui.sanitize();
        edit_document(&paths.desktop_file(), "ui.window_width", true, |doc| {
            insert_dotted(doc, "ui.window_width", int(ui.window_width))?;
            insert_dotted(doc, "ui.window_height", int(ui.window_height))?;
            insert_dotted(doc, "ui.window_maximized", toml::Value::Boolean(maximized))?;
            match (ui.window_x, ui.window_y) {
                (Some(x), Some(y)) => {
                    insert_dotted(doc, "ui.window_x", toml::Value::Integer(i64::from(x)))?;
                    insert_dotted(doc, "ui.window_y", toml::Value::Integer(i64::from(y)))
                }
                _ => {
                    remove_dotted(doc, "ui.window_x");
                    remove_dotted(doc, "ui.window_y");
                    Ok(())
                }
            }
        })
        .map(|_| ())
    }

    /// Remembers the side panes: visibility and widths (clamped).
    pub fn set_panes(
        paths: &ConfigPaths,
        left_visible: bool,
        right_visible: bool,
        left_width: u32,
        right_width: u32,
    ) -> Result<()> {
        let mut ui = DesktopUiConfig {
            left_pane_width: left_width,
            right_pane_width: right_width,
            ..Default::default()
        };
        ui.sanitize();
        edit_document(&paths.desktop_file(), "ui.left_pane_width", true, |doc| {
            insert_dotted(
                doc,
                "ui.left_pane_visible",
                toml::Value::Boolean(left_visible),
            )?;
            insert_dotted(
                doc,
                "ui.right_pane_visible",
                toml::Value::Boolean(right_visible),
            )?;
            insert_dotted(doc, "ui.left_pane_width", int(ui.left_pane_width))?;
            insert_dotted(doc, "ui.right_pane_width", int(ui.right_pane_width))
        })
        .map(|_| ())
    }

    /// Remembers Log column widths (clamped): the default for every
    /// repository with `repo == None`, else that repository's own.
    pub fn set_column_widths(
        paths: &ConfigPaths,
        repo: Option<&str>,
        widths: DesktopColumnWidths,
    ) -> Result<()> {
        let value = widths_value(widths.sanitized());
        let not_a_table = || Error::UnsettableKey {
            key: "repos".into(),
            kind: "value",
        };
        edit_document(&paths.desktop_file(), "ui.column_widths_px", true, |doc| {
            let Some(repo) = repo else {
                return insert_dotted(doc, "ui.column_widths_px", value.clone());
            };
            let root = doc.as_table_mut().ok_or_else(not_a_table)?;
            let repos = root
                .entry("repos")
                .or_insert_with(|| toml::Value::Table(Default::default()))
                .as_table_mut()
                .ok_or_else(not_a_table)?;
            let entry = repos
                .entry(repo)
                .or_insert_with(|| toml::Value::Table(Default::default()))
                .as_table_mut()
                .ok_or_else(not_a_table)?;
            entry.insert("column_widths_px".into(), value.clone());
            Ok(())
        })
        .map(|_| ())
    }

    /// Forgets `repo`'s own column widths (it follows the default again). A
    /// repository entry left empty is dropped.
    pub fn reset_column_widths(paths: &ConfigPaths, repo: &str) -> Result<()> {
        let path = paths.desktop_file();
        let doc = crate::split::read_doc(&path)?;
        if doc
            .get("repos")
            .and_then(|r| r.get(repo))
            .and_then(|r| r.get("column_widths_px"))
            .is_none()
        {
            return Ok(());
        }
        edit_document(&path, "repos", true, |doc| {
            if let Some(repos) = doc.get_mut("repos").and_then(toml::Value::as_table_mut) {
                if let Some(entry) = repos.get_mut(repo).and_then(toml::Value::as_table_mut) {
                    entry.remove("column_widths_px");
                    if entry.is_empty() {
                        repos.remove(repo);
                    }
                }
                if repos.is_empty() {
                    if let Some(root) = doc.as_table_mut() {
                        root.remove("repos");
                    }
                }
            }
            Ok(())
        })
        .map(|_| ())
    }

    /// Sets `[keybindings.<ctx>] <action>`: one chord as a string, several (or
    /// none: unbound) as an array.
    pub fn set_keybinding(
        paths: &ConfigPaths,
        ctx: &str,
        action: &str,
        chords: &[String],
    ) -> Result<()> {
        let key = format!("keybindings.{ctx}.{action}");
        crate::keybindings::check_name(&key, ctx)?;
        crate::keybindings::check_name(&key, action)?;
        let value = match chords {
            [one] => toml::Value::String(one.clone()),
            many => toml::Value::Array(many.iter().cloned().map(toml::Value::String).collect()),
        };
        edit_document(&paths.desktop_file(), "keybindings", true, |doc| {
            insert_dotted(doc, &key, value.clone())
        })
        .map(|_| ())
    }

    /// Removes one binding (back to its default); emptied tables go too.
    pub fn reset_keybinding(paths: &ConfigPaths, ctx: &str, action: &str) -> Result<()> {
        Self::remove_if_present(paths, &format!("keybindings.{ctx}.{action}"))
    }

    /// Removes the whole `[keybindings]` table.
    pub fn reset_all_keybindings(paths: &ConfigPaths) -> Result<()> {
        Self::remove_if_present(paths, "keybindings")
    }

    fn remove_if_present(paths: &ConfigPaths, key: &str) -> Result<()> {
        let path = paths.desktop_file();
        if !file_says(&path, key)? {
            return Ok(());
        }
        edit_document(&path, key, true, |doc| {
            remove_dotted(doc, key);
            Ok(())
        })
        .map(|_| ())
    }
}

/// `desktop.toml` resolved like [`crate::Settings`]: built-in defaults, then
/// the file, then `GITPOULPI_DESKTOP_UI_*` environment variables (only for
/// [`DESKTOP_LIVE_KEYS`]). Keys are the full dotted spelling
/// (`desktop.ui.theme`).
#[derive(Debug)]
pub struct DesktopSettings {
    config: DesktopConfig,
    file: PathBuf,
    raw_file: Option<toml::Value>,
    overridden: BTreeMap<String, ()>,
}

impl DesktopSettings {
    /// Against the real process environment.
    pub fn resolve(paths: &ConfigPaths) -> Result<Self> {
        Self::resolve_from(&paths.desktop_file(), |name| std::env::var(name).ok())
    }

    /// The testable core (`env` is a lookup, see `Settings::resolve_from`).
    pub fn resolve_from(file: &Path, env: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let raw_file = if file.exists() {
            let raw = std::fs::read_to_string(file)?;
            Some(toml::from_str::<toml::Value>(&raw)?)
        } else {
            None
        };
        let mut config = DesktopConfig::load_file(file)?;
        let mut overridden = BTreeMap::new();
        for key in DESKTOP_LIVE_KEYS {
            if let Some(value) = env(&crate::env_var_for(key)) {
                let relative = strip_prefix(key).unwrap_or(key);
                config.set(relative, &value).map_err(|err| match err {
                    Error::BadValue {
                        value, expected, ..
                    } => Error::BadValue {
                        key: crate::env_var_for(key),
                        value,
                        expected,
                    },
                    other => other,
                })?;
                overridden.insert((*key).to_string(), ());
            }
        }
        Ok(Self {
            config,
            file: file.to_path_buf(),
            raw_file,
            overridden,
        })
    }

    pub fn config(&self) -> &DesktopConfig {
        &self.config
    }

    /// The `desktop.toml` this resolved against, whether or not it exists.
    pub fn file(&self) -> &Path {
        &self.file
    }

    /// The value in force for a full key (`desktop.ui.theme`).
    pub fn get(&self, key: &str) -> Result<serde_json::Value> {
        self.config
            .get(strip_prefix(key).ok_or_else(|| Error::UnknownKey(key.to_string()))?)
    }

    /// Which layer supplied the value in force for `key`.
    pub fn source_of(&self, key: &str) -> crate::Source {
        if self.overridden.contains_key(key) {
            return crate::Source::Env;
        }
        let mut cursor = match &self.raw_file {
            Some(doc) => doc,
            None => return crate::Source::Default,
        };
        for segment in strip_prefix(key).unwrap_or(key).split('.') {
            match cursor.get(segment) {
                Some(next) => cursor = next,
                None => return crate::Source::Default,
            }
        }
        crate::Source::File
    }
}
