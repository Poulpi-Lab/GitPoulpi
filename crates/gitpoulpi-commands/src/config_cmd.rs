//! `gitpoulpi config get|set|path`.
//!
//! Handled here rather than in `commands/` because none of it is a
//! `GitBackend` operation: it reads and writes a file on this machine, so
//! it must work outside a repository and must not start a daemon.
//!
//! `get` reports the value in force, **which layer it came from**
//! (`default`/`file`/`env`/`flag`), and whether anything in GitPoulpi
//! actually reads that key, so an ignored key can be told from a
//! misspelled one.

use gitpoulpi_config::desktop_ui_file::{self as desktop, DesktopSettings};
use gitpoulpi_config::installation::{self, is_look_key};
use gitpoulpi_config::{ConfigPaths, Overrides, Settings};
use serde_json::json;
use std::path::PathBuf;

use crate::cli::ConfigAction;
use crate::commands::emitted;
use crate::repo_cmd::Emitted;

/// Runs one `config` action. Shared by the CLI binary and the MCP server
/// (`config get` / `config path` are tools), so neither can reach a
/// different answer.
pub fn execute(action: ConfigAction, overrides: &Overrides) -> Result<Emitted, String> {
    let paths = ConfigPaths::discover()
        .ok_or_else(|| "could not determine this OS's config directory".to_string())?;
    let file = paths.config_file();

    match action {
        ConfigAction::Path => Ok(emitted(
            json!({
                "path": file.to_string_lossy(),
                "tui_file": paths.tui_file().to_string_lossy(),
                "desktop_file": paths.desktop_file().to_string_lossy(),
                "config_dir": paths.config_dir().to_string_lossy(),
                "exists": file.exists(),
                "tui_exists": paths.tui_file().exists(),
                "desktop_exists": paths.desktop_file().exists(),
            }),
            file.display().to_string(),
        )),

        ConfigAction::Get { key } if desktop::strip_prefix(&key).is_some() => {
            get_desktop(&paths, &key)
        }

        ConfigAction::Set { key, value } if desktop::strip_prefix(&key).is_some() => {
            set_desktop(&paths, &key, &value)
        }

        ConfigAction::Get { key } => {
            let settings = Settings::resolve(Some(&paths), overrides).map_err(|e| e.to_string())?;
            let mut value = settings.config().get(&key).map_err(|e| e.to_string())?;
            let mut source = settings.source_of(&key);
            // The colours are this installation's own: `get` answers what it
            // shows, not what another installation stored in `ui.*`.
            if let Some(short) = key.strip_prefix("ui.").filter(|_| is_look_key(&key)) {
                let cur = installation::current();
                let own = installation::effective(settings.config(), &cur.identity);
                if let Some(v) = own.look.get(short) {
                    value = json!(v);
                    let own_entry = cur.config.installation.settings.get(&cur.identity.id);
                    if own_entry.is_some_and(|e| e.look.contains_key(short)) {
                        source = gitpoulpi_config::Source::File;
                    }
                }
            }
            let implemented = Settings::key_is_implemented(&key);
            // The file that owns the key: `tui.toml` for an interface key,
            // `config.toml` otherwise (a read before `tui.toml` exists is
            // derived from `config.toml` in memory; the first write creates it).
            let key_file = home_file(&paths, &key);
            let mut human = format!("{key} = {value}  (source: {})", source.as_str());
            if !implemented {
                human.push_str(&format!("\nnote: {UNREAD_NOTE}"));
            }
            Ok(emitted(
                json!({
                    "key": key,
                    "value": value,
                    "source": source,
                    "implemented": implemented,
                    "file": key_file.to_string_lossy(),
                }),
                human,
            ))
        }

        ConfigAction::Set { key, value } => {
            match key.as_str() {
                "ui.theme" => crate::theme_cmd::validate_ui_theme(&value)?,
                "ui.graph_color_theme" => crate::theme_cmd::validate_graph_color_theme(&value)?,
                _ => {}
            }
            // Deliberately the *file* layer, not the resolved one: writing
            // back a `Settings` would bake whatever `GITPOULPI_*` variable
            // or `--backend` flag happened to be in force into config.toml,
            // so a one-off override would silently become permanent.
            // The colour keys are one installation's own: its entry, or
            // `ui.*` only when it is the primary.
            // Interface keys are written to `tui.toml`, the rest to
            // `config.toml` (a colour of a non-primary installation is its
            // `[installation]` entry, in `config.toml`).
            let mut written = home_file(&paths, &key);
            let stored = if let Some(short) = key.strip_prefix("ui.").filter(|_| is_look_key(&key))
            {
                let cur = installation::current();
                // The same test `set_look_in_file` makes, on the file itself.
                let primary = gitpoulpi_config::Config::load(&file)
                    .ok()
                    .and_then(|c| c.installation.primary);
                if primary.as_deref() != Some(cur.identity.id.as_str()) {
                    written = file.clone();
                }
                installation::set_look_in_file(&file, &cur.identity, short, &value)
                    .map(serde_json::Value::String)
            } else {
                gitpoulpi_config::set_in_file(&file, &key, &value)
            }
            .map_err(|e| e.to_string())?;
            let mut human = format!("{key} = {stored}  ->  {}", written.display());
            if let Some(note) = not_built_note(&key) {
                human.push_str(&format!("\nnote: {note}"));
            }
            if !Settings::key_is_implemented(&key) {
                human.push_str(&format!("\nnote: {UNREAD_NOTE}"));
            }
            Ok(emitted(
                json!({
                    "key": key,
                    "value": stored,
                    "path": file.to_string_lossy(),
                    "file": written.to_string_lossy(),
                }),
                human,
            ))
        }
    }
}

/// `config get desktop.<key>`: the value in force for the desktop frontend,
/// resolved from `desktop.toml` and `GITPOULPI_DESKTOP_UI_*`.
fn get_desktop(paths: &ConfigPaths, key: &str) -> Result<Emitted, String> {
    let settings = DesktopSettings::resolve(paths).map_err(|e| e.to_string())?;
    let mut value = settings.get(key).map_err(|e| e.to_string())?;
    let mut source = settings.source_of(key);
    let short = key
        .strip_prefix("desktop.ui.")
        .filter(|k| is_look_key(&format!("ui.{k}")));
    if let Some(short) = short {
        // This installation's own colours, as for `ui.*`.
        let cur = installation::current();
        let own = installation::effective_desktop_look(
            &cur.config,
            &settings.config().ui,
            &cur.identity.id,
        );
        if let Some(v) = own.get(short) {
            value = json!(v);
            let own_entry = cur.config.installation.settings.get(&cur.identity.id);
            if own_entry.is_some_and(|e| e.desktop_look.contains_key(short)) {
                source = gitpoulpi_config::Source::File;
            }
        }
    }
    let implemented = desktop::key_is_implemented(key);
    let mut human = format!("{key} = {value}  (source: {})", source.as_str());
    if !implemented {
        human.push_str(&format!("\nnote: {UNREAD_NOTE}"));
    }
    Ok(emitted(
        json!({
            "key": key,
            "value": value,
            "source": source,
            "implemented": implemented,
            "file": paths.desktop_file().to_string_lossy(),
        }),
        human,
    ))
}

/// `config set desktop.<key> <value>`: written to `desktop.toml` (a colour of
/// a non-primary installation is its `[installation]` entry in `config.toml`).
fn set_desktop(paths: &ConfigPaths, key: &str, value: &str) -> Result<Emitted, String> {
    let short = desktop::strip_prefix(key).unwrap_or(key);
    match short {
        "ui.theme" => crate::theme_cmd::validate_ui_theme(value)?,
        "ui.graph_color_theme" => crate::theme_cmd::validate_graph_color_theme(value)?,
        _ => {}
    }
    let mut written = paths.desktop_file();
    let look = short
        .strip_prefix("ui.")
        .filter(|k| is_look_key(&format!("ui.{k}")));
    let stored = if let Some(look) = look {
        let cur = installation::current();
        let primary = gitpoulpi_config::Config::load(&paths.config_file())
            .ok()
            .and_then(|c| c.installation.primary);
        if primary.as_deref() != Some(cur.identity.id.as_str()) {
            written = paths.config_file();
        }
        installation::set_desktop_look_in_file(paths, &cur.identity, look, value)
            .map(serde_json::Value::String)
    } else {
        gitpoulpi_config::DesktopConfig::set_in_file(paths, short, value)
    }
    .map_err(|e| e.to_string())?;
    let mut human = format!("{key} = {stored}  ->  {}", written.display());
    if !desktop::key_is_implemented(key) {
        human.push_str(&format!("\nnote: {UNREAD_NOTE}"));
    }
    Ok(emitted(
        json!({
            "key": key,
            "value": stored,
            "path": paths.config_file().to_string_lossy(),
            "file": written.to_string_lossy(),
        }),
        human,
    ))
}

/// The file that owns `key` in `paths`' directory.
fn home_file(paths: &ConfigPaths, key: &str) -> PathBuf {
    match gitpoulpi_config::split::home_of_write(key) {
        gitpoulpi_config::split::Home::Common => paths.config_file(),
        _ => paths.tui_file(),
    }
}

/// English on purpose: reading it from the i18n catalog (`msg::settings::*`)
/// keeps the whole catalog blob (about 3 MiB) alive in `gitpoulpi-mcp`, which
/// otherwise links none of it - see `docs/developer/binary-size.md`.
const NOT_BUILT_NOTE: &str = "(not built into this binary; kept for other builds)";

/// The `integrations` table (without a hosting provider) and the `ai`
/// table (without AI) still parse and are written back faithfully, but
/// nothing in such a binary reads them.
fn not_built_note(key: &str) -> Option<String> {
    let under = |table: &str| {
        key == table
            || key
                .strip_prefix(table)
                .is_some_and(|rest| rest.starts_with('.') || rest.starts_with('['))
    };
    if !cfg!(feature = "hosting") && under("integrations") {
        return Some(NOT_BUILT_NOTE.to_string());
    }
    if !cfg!(feature = "ai") && under("ai") {
        return Some(NOT_BUILT_NOTE.to_string());
    }
    None
}

const UNREAD_NOTE: &str = "nothing reads this key yet - it is stored and read back faithfully, \
     but no component consults it. See docs/user/config-reference.md.";

#[cfg(all(test, not(feature = "hosting")))]
mod not_built_tests {
    use super::not_built_note;

    #[test]
    fn integrations_keys_get_the_note_and_others_do_not() {
        assert!(not_built_note("integrations").is_some());
        assert!(not_built_note("integrations.0.url").is_some());
        assert!(not_built_note("integrations[0].url").is_some());
        assert!(not_built_note("ui.theme").is_none());
    }
}

#[cfg(all(test, not(feature = "ai")))]
mod ai_not_built_tests {
    use super::not_built_note;

    #[test]
    fn ai_keys_get_the_note_and_others_do_not() {
        assert!(not_built_note("ai.enabled").is_some());
        assert!(not_built_note("ai").is_some());
        assert!(not_built_note("ai.profiles[0].model").is_some());
        assert!(not_built_note("aim.x").is_none());
        assert!(not_built_note("ui.theme").is_none());
    }
}
