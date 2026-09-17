//! `OperationKind::SetRefVisibility`: the hidden/solo ref filter of one
//! repository, kept in its `repo-state.toml` (`RepoState::ref_visibility`)
//! -- the same file the TUI reads and writes. Going through the daemon
//! (rather than the CLI editing the file) is what lets the server broadcast
//! `Event::RefVisibilityChanged` so a running TUI can follow.

use std::collections::BTreeMap;

use gitpoulpi_core::domain::RefVisibility;
use gitpoulpi_ipc::Request;
use gitpoulpi_session::RepoState;
use serde_json::{json, Value};

use crate::dispatch::{err, ok, Dispatched};

pub fn set(request: &Request) -> Dispatched {
    let repo = match crate::dispatch::repo_ref(request) {
        Ok(r) => r,
        Err(m) => return err(request.id, m, false),
    };
    let mode = request
        .args
        .get("mode")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let clear = request
        .args
        .get("clear")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let names: Vec<String> = request
        .args
        .get("names")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    if !matches!(mode, "hide" | "show" | "solo" | "replace") {
        return err(
            request.id,
            format!("unknown ref visibility mode `{mode}` (hide, show, solo or replace)"),
            false,
        );
    }
    let listed = |key: &str| -> Vec<String> {
        request
            .args
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    };
    if mode != "replace" && names.is_empty() && !clear {
        return err(request.id, "name at least one ref, or clear".into(), false);
    }

    let state_dir = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().session.per_repo_state_dir.clone())
        .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
    let path = RepoState::path_for(&repo.root, &state_dir);
    let mut state = match RepoState::load(&path) {
        Ok(s) => s,
        Err(e) => return err(request.id, e.to_string(), false),
    };
    if mode == "replace" {
        // The TUI's own edits: it holds both full sets, so it sends them
        // whole (`hidden`, `solo`) instead of a delta. Solo wins a ref
        // listed in both, as it does in the TUI's own save.
        state.ref_visibility.clear();
        for name in listed("hidden") {
            state.ref_visibility.insert(name, RefVisibility::Hidden);
        }
        for name in listed("solo") {
            state.ref_visibility.insert(name, RefVisibility::Solo);
        }
    } else {
        apply(&mut state.ref_visibility, mode, &names, clear);
    }
    if let Err(e) = state.save(&path) {
        return err(request.id, e.to_string(), false);
    }
    let pick = |wanted: RefVisibility| -> Vec<&String> {
        state
            .ref_visibility
            .iter()
            .filter(|(_, v)| **v == wanted)
            .map(|(k, _)| k)
            .collect()
    };
    ok(
        request.id,
        json!({ "hidden": pick(RefVisibility::Hidden), "solo": pick(RefVisibility::Solo) }),
        true,
    )
}

/// `hide` / `solo` mark the named refs (a ref is one or the other, as in the
/// TUI); `show` returns them to the default. `clear` empties what the mode
/// owns: hide -> every hidden ref, solo -> every soloed ref, show -> both.
fn apply(map: &mut BTreeMap<String, RefVisibility>, mode: &str, names: &[String], clear: bool) {
    if clear {
        map.retain(|_, v| match mode {
            "hide" => *v != RefVisibility::Hidden,
            "solo" => *v != RefVisibility::Solo,
            _ => false,
        });
    }
    for name in names {
        match mode {
            "hide" => {
                map.insert(name.clone(), RefVisibility::Hidden);
            }
            "solo" => {
                map.insert(name.clone(), RefVisibility::Solo);
            }
            _ => {
                map.remove(name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hide_solo_show_and_clear() {
        let mut m = BTreeMap::new();
        apply(&mut m, "hide", &["a".into(), "b".into()], false);
        apply(&mut m, "solo", &["c".into()], false);
        assert_eq!(m.len(), 3);
        apply(&mut m, "show", &["a".into()], false);
        assert!(!m.contains_key("a"));
        apply(&mut m, "hide", &[], true);
        assert_eq!(m.get("c"), Some(&RefVisibility::Solo));
        assert_eq!(m.len(), 1);
        apply(&mut m, "show", &[], true);
        assert!(m.is_empty());
    }
}
