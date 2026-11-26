//! `gitpoulpi installation show|set`: which GitPoulpi installation this is and
//! its own settings. Local (no daemon, no repository), like `alias` and
//! `desktop`. The work is `gitpoulpi_config::installation`.

use std::sync::OnceLock;

use gitpoulpi_config::installation::{self, Current};
use gitpoulpi_config::Config;
use serde_json::{json, Value};

use crate::cli::InstallationAction;
use crate::commands::emitted;
use crate::repo_cmd::Emitted;

/// What moves the installation's launchers when its launcher name changes:
/// given the config before and after, it renames the shortcuts and the
/// folder-menu entry in place. The shortcut machinery lives in the TUI crate,
/// which this one does not link, so the program registers it at start.
pub type NameChangeHook = fn(&Config, &Config);

static NAME_CHANGE: OnceLock<NameChangeHook> = OnceLock::new();

/// Registers the launcher rename run after `installation set name` and
/// `installation set name_with_version`. The first registration wins.
pub fn on_launcher_name_change(hook: NameChangeHook) {
    let _ = NAME_CHANGE.set(hook);
}

/// What follows when the installation's colour changes: given the config
/// after, it rewrites the shortcut and folder-menu icons in place.
pub type ColorChangeHook = fn(&Config);

static COLOR_CHANGE: OnceLock<ColorChangeHook> = OnceLock::new();

/// Registers the icon refresh run after `installation set logo_color`. The
/// first registration wins.
pub fn on_logo_color_change(hook: ColorChangeHook) {
    let _ = COLOR_CHANGE.set(hook);
}

/// The settings of one installation as JSON.
fn settings_json(e: &installation::Effective) -> Value {
    json!({
        "name": e.name,
        "name_with_version": e.name_with_version,
        "logo_color": e.logo_color,
        "command_alias": e.command_alias,
        "command_alias_add": e.command_alias_add,
        "folder_menu_submenu": e.folder_menu_submenu,
        "env": e.env,
        "mcp_bind": e.mcp_bind,
    })
}

fn show(cur: &Current) -> Emitted {
    let id = &cur.identity.id;
    let others: Vec<Value> = cur
        .config
        .installation
        .settings
        .iter()
        .filter(|(other, _)| *other != id)
        .map(|(other, entry)| {
            let eff = installation::effective_of(&cur.config, other);
            json!({
                "id": other,
                "kind": entry.kind.as_str(),
                "path": entry.path,
                "exists": !entry.path.is_empty() && std::path::Path::new(&entry.path).exists(),
                "primary": cur.config.installation.primary.as_deref() == Some(other.as_str()),
                "name": eff.as_ref().map(|e| e.name.clone()),
            })
        })
        .collect();
    let human = format!(
        "{} ({}) {}\nid {id}, environment {}{}",
        cur.effective.name,
        cur.identity.kind.as_str(),
        cur.identity.root,
        cur.environment,
        if cur.effective.primary {
            ", primary"
        } else {
            ""
        },
    );
    emitted(
        json!({
            "id": id,
            "short": cur.identity.short(),
            "kind": cur.identity.kind.as_str(),
            "path": cur.identity.root,
            "channel": cur.identity.channel.map(|c| c.as_str()),
            "env": cur.environment,
            "primary": cur.effective.primary,
            "settings": settings_json(&cur.effective),
            "others": others,
        }),
        human,
    )
}

/// A colour is a bare lowercase word (the TUI checks it against the build's colorways).
fn valid_color(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn set(cur: &Current, field: &str, value: &str) -> Result<Emitted, String> {
    let file = cur
        .config_file
        .as_deref()
        .ok_or("could not determine this OS's config directory")?;
    let value = value.trim();
    let cleaned = match field {
        "name" => {
            if !value.is_empty() {
                installation::validate_name(value).map_err(|e| format!("{e:?}: {value:?}"))?;
            }
            value.to_string()
        }
        "name_with_version" => String::new(),
        "logo_color" => {
            if !valid_color(value) {
                return Err(format!("{value:?} is not a colour name"));
            }
            value.to_string()
        }
        "command_alias" | "command_alias_add" => {
            if !value.is_empty() {
                gitpoulpi_config::desktop::alias::validate_name(value)
                    .map_err(|e| format!("{}: {value:?}", e.describe()))?;
            }
            value.to_string()
        }
        "env" => {
            let clean = installation::sanitize(value);
            if clean.is_empty() && !value.is_empty() {
                return Err(format!("{value:?} is not an environment name"));
            }
            clean
        }
        "folder_menu_submenu" | "mcp_bind" => value.to_string(),
        other => {
            return Err(format!(
                "unknown field {other:?}; one of name, name_with_version, logo_color, command_alias, command_alias_add, folder_menu_submenu, env, mcp_bind"
            ))
        }
    };
    let flag = match field {
        "name_with_version" => Some(match value {
            "true" | "on" | "yes" | "1" => true,
            "false" | "off" | "no" | "0" | "" => false,
            _ => return Err(format!("{value:?} is not on or off")),
        }),
        _ => None,
    };
    let id = cur.identity.id.clone();
    let identity = cur.identity.clone();
    let before = Config::load(file).unwrap_or_default();
    installation::set_installation_in_file(file, |config| {
        let entry =
            config
                .settings
                .entry(id)
                .or_insert_with(|| gitpoulpi_config::InstallationEntry {
                    path: identity.root.clone(),
                    kind: identity.kind,
                    ..Default::default()
                });
        match field {
            "name" => entry.name = cleaned.clone(),
            "name_with_version" => entry.name_with_version = flag.unwrap_or(false),
            "logo_color" => entry.logo_color = cleaned.clone(),
            "command_alias" => entry.command_alias = cleaned.clone(),
            "command_alias_add" => entry.command_alias_add = cleaned.clone(),
            "folder_menu_submenu" => entry.folder_menu_submenu = cleaned.clone(),
            "env" => entry.env = cleaned.clone(),
            "mcp_bind" => entry.mcp_bind = cleaned.clone(),
            _ => {}
        }
    })
    .map_err(|e| e.to_string())?;
    // The primary also owns the keys older builds read.
    if cur.effective.primary {
        let key = match field {
            "logo_color" => Some("ui.logo_color"),
            "command_alias" => Some("ui.command_alias"),
            "command_alias_add" => Some("ui.command_alias_add"),
            _ => None,
        };
        if let Some(key) = key {
            gitpoulpi_config::set_in_file(file, key, &cleaned).map_err(|e| e.to_string())?;
        }
    }
    // The launcher name is part of the shortcuts' names: they follow it now,
    // never waiting for the next `desktop shortcut install`.
    if matches!(field, "name" | "name_with_version") {
        if let (Some(hook), Ok(after)) = (NAME_CHANGE.get(), Config::load(file)) {
            hook(&before, &after);
        }
    }
    // Likewise the icon of the shortcuts: it wears the colour.
    if field == "logo_color" {
        if let (Some(hook), Ok(after)) = (COLOR_CHANGE.get(), Config::load(file)) {
            hook(&after);
        }
    }
    Ok(emitted(
        json!({ "field": field, "value": if field == "name_with_version" { json!(flag) } else { json!(cleaned) } }),
        format!("{field} = {value}"),
    ))
}

pub fn execute(action: InstallationAction) -> Result<Emitted, String> {
    let cur = installation::current();
    match action {
        InstallationAction::Show => Ok(show(cur)),
        InstallationAction::Set { field, value } => set(cur, &field, &value),
    }
}
