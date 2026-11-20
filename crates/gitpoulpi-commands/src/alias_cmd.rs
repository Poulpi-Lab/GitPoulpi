//! `gitpoulpi alias status|create|remove` and `gitpoulpi path status|add|remove`:
//! the scriptable twins of Settings > Launcher's command alias and
//! "Add GitPoulpi to PATH" rows, and what the installers call. The work is
//! `gitpoulpi_config::desktop::{alias, path}`; nothing here needs a daemon or
//! a repository. Not journaled: there is no git state to undo.
//!
//! Several installations share the alias folder and the user PATH: every
//! action runs as the current installation (`installation::current`), touches
//! only its own aliases and keeps what the others need.

use gitpoulpi_config::desktop::alias::{self, Kind, NameError};
use gitpoulpi_config::desktop::path::{self as user_path, UserPath};
use gitpoulpi_config::installation;
use gitpoulpi_config::open_in::Os;
use serde_json::{json, Value};

use crate::cli::{AliasAction, PathAction};
use crate::commands::emitted;
use crate::repo_cmd::Emitted;

struct Machine {
    alias: alias::Env,
    path: user_path::Env,
    /// Windows: App Paths `<alias>.exe` (a file in tests).
    registry: gitpoulpi_config::desktop::registry::SharedRegistry,
}

fn machine() -> Result<Machine, String> {
    let os = Os::current();
    let home = gitpoulpi_config::home_dir().ok_or("no home directory")?;
    let env = |k: &str| std::env::var(k).ok();
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let cur = installation::current();
    let mut alias = alias::Env::discover(os, &env, &home, &exe).with_ctx(cur.ctx.clone());
    alias.newest = gitpoulpi_config::dev_mode::launcher_newest_on_disk(&exe);
    // The shared record on PATH keeps what other installations rely on: their
    // folders, and the alias folder while one of them still has an alias there.
    let keep = installation::path_keep(cur, &alias);
    let path = user_path::Env::discover(os, &env, &home, &alias).keeping(keep);
    Ok(Machine {
        alias,
        path,
        registry: gitpoulpi_config::desktop::registry::from_env(),
    })
}

/// This installation's command alias and add-tab alias settings (its entry,
/// else `ui.*` when it is the primary).
fn configured_names() -> (String, String) {
    let e = &installation::current().effective;
    (e.command_alias.clone(), e.command_alias_add.clone())
}

/// `{ "id": .., "name": .. }` for the installation an alias belongs to.
fn installation_json(id: Option<&str>) -> Value {
    let cur = installation::current();
    let Some(id) = id else { return Value::Null };
    let name = if id == cur.identity.id {
        Some(cur.effective.name.clone())
    } else {
        installation::effective_of(&cur.config, id).map(|e| e.name)
    };
    json!({ "id": id, "name": name, "mine": id == cur.identity.id })
}

/// An alias made through `alias create` is recorded in this installation's
/// settings when it has none, so an alias an installer made shows in Settings.
fn record_name(name: &str, kind: Kind) {
    let cur = installation::current();
    let Some(file) = cur.config_file.as_deref() else {
        return;
    };
    let id = cur.identity.id.clone();
    let identity = cur.identity.clone();
    let name = name.to_string();
    let primary = cur.effective.primary;
    let (field, current) = match kind {
        Kind::Open => ("ui.command_alias", &cur.effective.command_alias),
        Kind::Add => ("ui.command_alias_add", &cur.effective.command_alias_add),
    };
    if !current.is_empty() {
        return;
    }
    let _ = installation::set_installation_in_file(file, |c| {
        // A pinned id is never seeded on disk, so its entry may be new here.
        let entry =
            c.settings
                .entry(id.clone())
                .or_insert_with(|| gitpoulpi_config::InstallationEntry {
                    path: identity.root.clone(),
                    kind: identity.kind,
                    ..Default::default()
                });
        match kind {
            Kind::Open => entry.command_alias = name.clone(),
            Kind::Add => entry.command_alias_add = name.clone(),
        }
    });
    if primary {
        let _ = gitpoulpi_config::set_in_file(file, field, &name);
    }
}

fn paths(list: &[std::path::PathBuf]) -> Vec<String> {
    list.iter().map(|p| p.display().to_string()).collect()
}

fn alias_json(m: &Machine, name: &str, kind: Kind) -> Value {
    let st = alias::status_kind(&m.alias, name, kind);
    json!({
        "name": name,
        "kind": kind.as_str(),
        "state": st.state.as_str(),
        "shadowed_by": st.shadowed_by.map(|p| p.display().to_string()),
        "powershell_builtin": st.powershell_builtin,
        "installation": installation_json(st.owner.as_deref()),
    })
}

fn path_state(m: &Machine, user: &dyn UserPath) -> Result<Value, String> {
    let st = user_path::status(&m.path, user)?;
    Ok(json!({
        "state": st.state.as_str(),
        "wanted": paths(&st.wanted),
        "added": paths(&st.added),
        "files": paths(&st.files),
    }))
}

pub fn alias(action: AliasAction) -> Result<Emitted, String> {
    let m = machine()?;
    match action {
        AliasAction::Status => {
            let (configured, configured_add) = configured_names();
            let existing: Vec<Value> = alias::list_all(&m.alias)
                .into_iter()
                .map(|e| {
                    json!({
                        "name": e.name,
                        "kind": e.kind.as_str(),
                        "state": e.state.as_str(),
                        "installation": installation_json(e.owner.as_deref()),
                    })
                })
                .collect();
            let setting = (alias::validate_name(&configured).is_ok())
                .then(|| alias_json(&m, &configured, Kind::Open));
            let setting_add = (alias::validate_name(&configured_add).is_ok())
                .then(|| alias_json(&m, &configured_add, Kind::Add));
            let human = if existing.is_empty() {
                "no command alias".to_string()
            } else {
                existing
                    .iter()
                    .map(|a| {
                        format!(
                            "{} ({}): {}",
                            a["name"].as_str().unwrap_or(""),
                            a["kind"].as_str().unwrap_or(""),
                            a["state"]
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            Ok(emitted(
                json!({
                    "os": format!("{:?}", m.alias.os).to_lowercase(),
                    "bin_dir": m.alias.bin_dir.display().to_string(),
                    "exe": m.alias.exe.display().to_string(),
                    "configured": setting,
                    "configured_add": setting_add,
                    "aliases": existing,
                }),
                &human,
            ))
        }
        AliasAction::Create { name, add } => {
            alias::validate_name(&name)
                .map_err(|e: NameError| format!("{}: {name:?}", e.describe()))?;
            let kind = if add { Kind::Add } else { Kind::Open };
            let before = alias_json(&m, &name, kind);
            let path = alias::create_with_app_path(&m.alias, &*m.registry, &name, kind)?;
            record_name(&name, kind);
            Ok(emitted(
                json!({
                    "created": name,
                    "kind": kind.as_str(),
                    "path": path.display().to_string(),
                    "target": m.alias.exe.display().to_string(),
                    "warnings": warnings(&before),
                    "bin_dir_on_path": bin_dir_on_path(&m),
                }),
                format!(
                    "created {} -> {}{}",
                    path.display(),
                    m.alias.exe.display(),
                    if add { " --add" } else { "" }
                ),
            ))
        }
        AliasAction::Remove { name } => {
            let removed: Vec<String> = match name {
                Some(name) => alias::remove_with_app_path(&m.alias, &*m.registry, &name)?
                    .then_some(name)
                    .into_iter()
                    .collect(),
                None => alias::remove_all_with_app_paths(&m.alias, &*m.registry)?,
            };
            let human = if removed.is_empty() {
                "no command alias to remove".to_string()
            } else {
                format!("removed {}", removed.join(", "))
            };
            Ok(emitted(json!({ "removed": removed }), &human))
        }
    }
}

fn bin_dir_on_path(m: &Machine) -> bool {
    m.alias
        .path_var
        .as_ref()
        .is_some_and(|p| std::env::split_paths(p).any(|d| d == m.alias.bin_dir))
}

/// What the name would clash with, as plain sentences.
fn warnings(status: &Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(p) = status["shadowed_by"].as_str() {
        out.push(format!("the name already runs {p}"));
    }
    if status["powershell_builtin"] == true {
        out.push("PowerShell's own alias of that name wins over this one".to_string());
    }
    out
}

pub fn path(action: PathAction) -> Result<Emitted, String> {
    let m = machine()?;
    let user = user_path::Registry;
    match action {
        PathAction::Status => {
            let state = path_state(&m, &user)?;
            let human = state["state"].as_str().unwrap_or("").to_string();
            Ok(emitted(state, &human))
        }
        PathAction::Add => {
            let files = user_path::add(&m.path, &user)?;
            let human = if files.is_empty() {
                "nothing to add: already on PATH".to_string()
            } else {
                "added to PATH for new terminals".to_string()
            };
            Ok(emitted(
                json!({ "changed": paths(&files), "wanted": paths(&m.path.wanted) }),
                &human,
            ))
        }
        PathAction::Remove => {
            let files = user_path::remove(&m.path, &user)?;
            let human = if files.is_empty() {
                "nothing of GitPoulpi's on PATH".to_string()
            } else {
                "removed from PATH".to_string()
            };
            Ok(emitted(json!({ "changed": paths(&files) }), &human))
        }
    }
}
