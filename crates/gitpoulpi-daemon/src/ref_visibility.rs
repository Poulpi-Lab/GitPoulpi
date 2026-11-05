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
    if !matches!(
        mode,
        "hide" | "show" | "solo" | "replace" | "hide-remote" | "show-remote"
    ) {
        return err(
            request.id,
            format!("unknown ref visibility mode `{mode}` (hide, show, solo, hide-remote, show-remote or replace)"),
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
        // Absent leaves the hidden remotes alone: a client that predates
        // them must not wipe what another one set.
        if request.args.get("hidden_remotes").is_some() {
            state.hidden_remotes = normalized(listed("hidden_remotes"));
        }
    } else if matches!(mode, "hide-remote" | "show-remote") {
        apply_remotes(&mut state, mode == "hide-remote", &names, clear);
    } else {
        apply(&mut state.ref_visibility, mode, &names, clear);
        // `hide --clear` and `show --clear` empty what hiding owns, and a
        // hidden remote is hiding.
        if clear && matches!(mode, "hide" | "show") {
            state.hidden_remotes.clear();
        }
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
        json!({
            "hidden": pick(RefVisibility::Hidden),
            "solo": pick(RefVisibility::Solo),
            "hidden_remotes": state.hidden_remotes,
        }),
        true,
    )
}

fn normalized(mut names: Vec<String>) -> Vec<String> {
    names.sort();
    names.dedup();
    names
}

/// Hide (`hide == true`) or show whole remotes. Hiding sets the flag and
/// drops the now-covered individual `<remote>/...` hides; showing clears the
/// flag AND every individual hide under the remote (a remote whose branches
/// were hidden one by one comes back whole). Soloed refs are left alone.
/// `clear` with no names empties the flag set (with names it is ignored).
fn apply_remotes(state: &mut RepoState, hide: bool, remotes: &[String], clear: bool) {
    if clear && remotes.is_empty() {
        state.hidden_remotes.clear();
        return;
    }
    for remote in remotes {
        let under = format!("{remote}/");
        state
            .ref_visibility
            .retain(|name, v| !(*v == RefVisibility::Hidden && name.starts_with(&under)));
        if hide {
            state.hidden_remotes.push(remote.clone());
        } else {
            state.hidden_remotes.retain(|r| r != remote);
        }
    }
    state.hidden_remotes = normalized(std::mem::take(&mut state.hidden_remotes));
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
    fn hiding_a_remote_sets_the_flag_and_absorbs_its_individual_hides() {
        let mut state = RepoState::default();
        apply(
            &mut state.ref_visibility,
            "hide",
            &["archive/old-a".into(), "origin/main".into(), "side".into()],
            false,
        );
        apply(
            &mut state.ref_visibility,
            "solo",
            &["archive/old-b".into()],
            false,
        );
        apply_remotes(&mut state, true, &["archive".into()], false);
        assert_eq!(state.hidden_remotes, vec!["archive"]);
        assert!(
            !state.ref_visibility.contains_key("archive/old-a"),
            "covered now"
        );
        assert_eq!(
            state.ref_visibility.get("archive/old-b"),
            Some(&RefVisibility::Solo),
            "solo is untouched"
        );
        assert_eq!(
            state.ref_visibility.get("origin/main"),
            Some(&RefVisibility::Hidden)
        );
        // Twice is once.
        apply_remotes(&mut state, true, &["archive".into()], false);
        assert_eq!(state.hidden_remotes, vec!["archive"]);
    }

    #[test]
    fn showing_a_remote_clears_the_flag_and_every_hide_under_it() {
        let mut state = RepoState::default();
        apply(
            &mut state.ref_visibility,
            "hide",
            &[
                "archive/old-a".into(),
                "archive/old-b".into(),
                "side".into(),
            ],
            false,
        );
        state.hidden_remotes = vec!["archive".into(), "other".into()];
        apply_remotes(&mut state, false, &["archive".into()], false);
        assert_eq!(state.hidden_remotes, vec!["other"]);
        assert_eq!(state.ref_visibility.len(), 1, "only `side` is left");
        // `clear` with no names empties the flags.
        apply_remotes(&mut state, false, &[], true);
        assert!(state.hidden_remotes.is_empty());
    }

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
