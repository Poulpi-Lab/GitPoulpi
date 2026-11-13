//! Windows Explorer: three shell verbs under `HKCU\Software\Classes` (a
//! folder, a folder's background, a drive), plus the App Paths key that lets
//! `gitpoulpi` typed in the address bar start the console-less launcher. All
//! of it is data over the [`Registry`](crate::desktop::registry::Registry)
//! trait, so it runs and is tested on every OS.
//!
//! A key without the `X-GitPoulpi` value is foreign and never written or
//! deleted. Remove deletes our keys, then the parents (`Directory\shell`, ...)
//! only when nothing is left in them, which restores what Install found.
//! Windows 11 shows these verbs only in the classic menu (Show more options).

use std::path::Path;

use super::Env;
use crate::desktop::registry::{RegOp, Values};
use crate::desktop::State;

const CLASSES: &str = "Software\\Classes";
const APP_PATHS: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\App Paths";
const MARK_NAME: &str = "X-GitPoulpi";
const MARK_VALUE: &str = "1";

/// The verb keys, relative to `Software\Classes`.
const ROOTS: [&str; 3] = [
    "Directory\\shell\\GitPoulpi",
    "Directory\\Background\\shell\\GitPoulpi",
    "Drive\\shell\\GitPoulpi",
];

/// Parents Install may have created, deepest first within each branch.
const PARENTS: [&str; 6] = [
    "Directory\\Background\\shell",
    "Directory\\Background",
    "Directory\\shell",
    "Directory",
    "Drive\\shell",
    "Drive",
];

fn full(rel: &str) -> String {
    format!("{CLASSES}\\{rel}")
}

fn s(x: &str) -> String {
    x.to_string()
}

/// The App Paths key for a program name (`gitpoulpi.exe`).
pub fn app_path_key(name: &str) -> String {
    format!("{APP_PATHS}\\{name}")
}

fn app_path_values(helper: &Path) -> Values {
    let dir = helper
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    vec![
        (s(""), helper.display().to_string()),
        (s("Path"), dir),
        (s(MARK_NAME), s(MARK_VALUE)),
    ]
}

/// Writes App Paths `<name>` (like `gitpoulpi.exe`) pointing at `helper`.
pub fn app_path_ops(name: &str, helper: &Path) -> Vec<RegOp> {
    let key = app_path_key(name);
    app_path_values(helper)
        .into_iter()
        .map(|(n, v)| RegOp::Set {
            key: key.clone(),
            name: n,
            value: v,
        })
        .collect()
}

/// Deletes App Paths `<name>`. Callers check the marker first.
pub fn app_path_remove(name: &str) -> RegOp {
    RegOp::DeleteTree(app_path_key(name))
}

fn marked(values: &Values) -> bool {
    values
        .iter()
        .any(|(n, v)| n.eq_ignore_ascii_case(MARK_NAME) && v == MARK_VALUE)
}

/// Is the App Paths key `<name>` there and ours? (`None`: absent.)
pub fn app_path_is_ours(
    registry: &dyn crate::desktop::registry::Registry,
    name: &str,
) -> Result<Option<bool>, String> {
    let got = registry.read(&[app_path_key(name)])?;
    Ok(got.into_iter().next().flatten().map(|(v, _)| marked(&v)))
}

/// The keys (relative to `HKCU`) and values Install writes for the verbs.
pub fn wanted_keys(helper: &Path, label: &str) -> Vec<(String, Values)> {
    let icon = format!("\"{}\",0", helper.with_file_name("gitpoulpi.exe").display());
    let command = format!("\"{}\" --folder \"%V\"", helper.display());
    let mut out = Vec::new();
    for root in ROOTS {
        out.push((
            full(root),
            vec![
                (s(""), s(label)),
                (s("MUIVerb"), s(label)),
                (s("Icon"), icon.clone()),
                (s(MARK_NAME), s(MARK_VALUE)),
            ],
        ));
        out.push((
            format!("{}\\command", full(root)),
            vec![(s(""), command.clone())],
        ));
    }
    out
}

fn norm(values: &Values) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = values
        .iter()
        .map(|(n, x)| (n.to_ascii_lowercase(), x.clone()))
        .collect();
    v.sort();
    v
}

fn all_keys() -> Vec<String> {
    let mut keys: Vec<String> = wanted_keys(Path::new(""), "")
        .into_iter()
        .map(|(k, _)| k)
        .collect();
    keys.push(app_path_key("gitpoulpi.exe"));
    keys
}

pub(super) fn state(env: &Env, label: &str) -> Result<State, String> {
    let keys = all_keys();
    let got = env.registry.read(&keys)?;
    let n = ROOTS.len() * 2;
    let roots: Vec<&Option<(Values, bool)>> = (0..ROOTS.len()).map(|i| &got[i * 2]).collect();
    let is_ours = |g: &Option<(Values, bool)>| g.as_ref().is_some_and(|(v, _)| marked(v));
    if roots.iter().any(|g| g.is_some() && !is_ours(g)) {
        return Ok(State::Foreign);
    }
    if !roots.iter().any(|g| is_ours(g)) {
        return Ok(State::NotInstalled);
    }
    let Some(helper) = env.helper.as_deref() else {
        return Ok(State::Outdated);
    };
    // The label the keys carry, so a language change alone is not "outdated".
    let found = roots
        .iter()
        .filter_map(|g| g.as_ref())
        .flat_map(|(v, _)| v.iter())
        .find(|(n, _)| n == "MUIVerb")
        .map_or_else(|| label.to_string(), |(_, v)| v.clone());
    let mut current = true;
    for (i, (_, want)) in wanted_keys(helper, &found).iter().enumerate() {
        match &got[i] {
            Some((vals, _)) if norm(vals) == norm(want) => {}
            _ => current = false,
        }
    }
    // App Paths: ours must match; a foreign one is left alone and ignored.
    match &got[n] {
        None => current = false,
        Some((vals, _)) if marked(vals) => {
            current &= norm(vals) == norm(&app_path_values(helper));
        }
        Some(_) => {}
    }
    Ok(if current {
        State::Installed
    } else {
        State::Outdated
    })
}

pub(super) fn install(env: &Env, label: &str) -> Result<(), String> {
    let helper = env.helper_path()?;
    let keys = all_keys();
    let got = env.registry.read(&keys)?;
    let mut ops = Vec::new();
    for (i, root) in ROOTS.iter().enumerate() {
        match &got[i * 2] {
            Some((v, _)) if !marked(v) => {
                return Err(format!("{CLASSES}\\{root} is not ours; not overwriting it"));
            }
            // Drop stale values and keys before writing again.
            Some(_) => ops.push(RegOp::DeleteTree(full(root))),
            None => {}
        }
    }
    for (key, values) in wanted_keys(helper, label) {
        for (name, value) in values {
            ops.push(RegOp::Set {
                key: key.clone(),
                name,
                value,
            });
        }
    }
    // App Paths: skip a key somebody else owns.
    let app = &got[ROOTS.len() * 2];
    if app.as_ref().is_none_or(|(v, _)| marked(v)) {
        ops.extend(app_path_ops("gitpoulpi.exe", helper));
    }
    env.registry.apply(&ops)
}

pub(super) fn remove(env: &Env) -> Result<bool, String> {
    let keys = all_keys();
    let got = env.registry.read(&keys)?;
    let mut ops = Vec::new();
    for (i, root) in ROOTS.iter().enumerate() {
        if got[i * 2].as_ref().is_some_and(|(v, _)| marked(v)) {
            ops.push(RegOp::DeleteTree(full(root)));
        }
    }
    let app = &got[ROOTS.len() * 2];
    if app.as_ref().is_some_and(|(v, _)| marked(v)) {
        ops.push(app_path_remove("gitpoulpi.exe"));
    }
    if ops.is_empty() {
        return Ok(false);
    }
    for p in PARENTS {
        ops.push(RegOp::DeleteIfEmpty(full(p)));
    }
    env.registry.apply(&ops)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::folder_menu::{self, Manager};
    use crate::desktop::registry::{MemoryRegistry, Registry};
    use crate::open_in::Os;
    use std::path::PathBuf;
    use std::sync::Arc;

    const LABEL: &str = "Open in GitPoulpi";

    fn env(reg: Arc<MemoryRegistry>) -> Env {
        Env {
            os: Os::Windows,
            data_home: PathBuf::new(),
            config_home: PathBuf::new(),
            home: PathBuf::new(),
            exe: PathBuf::from("C:/gp/gitpoulpi-tui.exe"),
            helper: Some(PathBuf::from("C:/gp/gitpoulpi-launch.exe")),
            has: Arc::new(|_| false),
            registry: reg,
            icon_png: None,
            refresh_hooks: false,
        }
    }

    fn set(key: &str, name: &str, value: &str) -> RegOp {
        RegOp::Set {
            key: key.into(),
            name: name.into(),
            value: value.into(),
        }
    }

    #[test]
    fn install_writes_exactly_the_planned_keys_and_values() {
        let reg = Arc::new(MemoryRegistry::new());
        let e = env(reg.clone());
        let done = folder_menu::install(&e, LABEL).unwrap();
        assert_eq!(done, [Manager::Explorer]);
        let mut want: Vec<(String, String, String)> = Vec::new();
        let cmd = "\"C:/gp/gitpoulpi-launch.exe\" --folder \"%V\"";
        let icon = "\"C:/gp/gitpoulpi.exe\",0";
        for root in ROOTS {
            let k = full(root).to_ascii_lowercase();
            for (n, v) in [
                ("", LABEL),
                ("MUIVerb", LABEL),
                ("Icon", icon),
                ("X-GitPoulpi", "1"),
            ] {
                want.push((k.clone(), n.into(), v.into()));
            }
            want.push((format!("{k}\\command"), "".into(), cmd.into()));
        }
        let app = app_path_key("gitpoulpi.exe").to_ascii_lowercase();
        want.push((app.clone(), "".into(), "C:/gp/gitpoulpi-launch.exe".into()));
        want.push((app.clone(), "Path".into(), "C:/gp".into()));
        want.push((app, "X-GitPoulpi".into(), "1".into()));
        // Parents exist as empty keys.
        let mut got: Vec<_> = reg
            .dump()
            .into_iter()
            .filter(|(_, _, v)| v != "<key>")
            .collect();
        got.sort();
        want.sort();
        assert_eq!(got, want);
        assert_eq!(
            folder_menu::status(&e, LABEL).entries[0].state,
            State::Installed
        );
    }

    #[test]
    fn remove_restores_the_registry_with_and_without_a_user_verb() {
        // A user verb already under Directory\shell: the parent stays.
        let reg = Arc::new(MemoryRegistry::new());
        reg.apply(&[
            set(&full("Directory\\shell\\Mine"), "", "x"),
            set(APP_PATHS, "keep", "1"),
        ])
        .unwrap();
        let before = reg.dump();
        let e = env(reg.clone());
        folder_menu::install(&e, LABEL).unwrap();
        assert_ne!(reg.dump(), before);
        assert_eq!(folder_menu::remove(&e).unwrap(), [Manager::Explorer]);
        assert_eq!(reg.dump(), before);
        // No verbs before: nothing after (the parents we created are gone).
        let reg = Arc::new(MemoryRegistry::new());
        reg.apply(&[set(CLASSES, "keep", "1"), set(APP_PATHS, "keep", "1")])
            .unwrap();
        let before = reg.dump();
        let e = env(reg.clone());
        folder_menu::install(&e, LABEL).unwrap();
        folder_menu::remove(&e).unwrap();
        assert_eq!(reg.dump(), before);
    }

    #[test]
    fn a_foreign_verb_is_untouched_by_install_and_remove() {
        let reg = Arc::new(MemoryRegistry::new());
        let k = full(ROOTS[0]);
        reg.apply(&[set(&k, "", "theirs")]).unwrap();
        let before = reg.dump();
        let e = env(reg.clone());
        assert_eq!(
            folder_menu::status(&e, LABEL).entries[0].state,
            State::Foreign
        );
        assert!(folder_menu::install(&e, LABEL).unwrap().is_empty());
        assert!(folder_menu::remove(&e).unwrap().is_empty());
        assert_eq!(reg.dump(), before);
    }

    #[test]
    fn a_moved_exe_is_outdated_and_a_new_label_alone_is_not() {
        let reg = Arc::new(MemoryRegistry::new());
        let e = env(reg.clone());
        folder_menu::install(&e, LABEL).unwrap();
        let st = folder_menu::status(&e, "Ouvrir dans GitPoulpi");
        assert_eq!(st.entries[0].state, State::Installed);
        let moved = Env {
            helper: Some(PathBuf::from("D:/new/gitpoulpi-launch.exe")),
            ..e.clone()
        };
        assert_eq!(
            folder_menu::status(&moved, LABEL).entries[0].state,
            State::Outdated
        );
        assert_eq!(
            folder_menu::sync(&moved, LABEL).unwrap(),
            [Manager::Explorer]
        );
        assert_eq!(
            folder_menu::status(&moved, LABEL).entries[0].state,
            State::Installed
        );
        let cmd = reg.read(&[format!("{}\\command", full(ROOTS[0]))]).unwrap();
        assert!(cmd[0].as_ref().unwrap().0[0].1.contains("D:/new/"));
    }

    #[test]
    fn app_paths_come_and_go_with_the_menu_and_a_foreign_one_stays() {
        let reg = Arc::new(MemoryRegistry::new());
        let e = env(reg.clone());
        let key = app_path_key("gitpoulpi.exe");
        folder_menu::install(&e, LABEL).unwrap();
        assert!(reg.read(std::slice::from_ref(&key)).unwrap()[0].is_some());
        folder_menu::remove(&e).unwrap();
        assert!(reg.read(std::slice::from_ref(&key)).unwrap()[0].is_none());
        // Somebody else's key of that name survives both.
        reg.apply(&[set(&key, "", "theirs")]).unwrap();
        folder_menu::install(&e, LABEL).unwrap();
        assert_eq!(
            reg.read(std::slice::from_ref(&key)).unwrap()[0]
                .as_ref()
                .unwrap()
                .0[0]
                .1,
            "theirs"
        );
        folder_menu::remove(&e).unwrap();
        assert!(reg.read(&[key]).unwrap()[0].is_some());
    }

    #[test]
    fn no_helper_means_unavailable() {
        let reg = Arc::new(MemoryRegistry::new());
        let e = Env {
            helper: None,
            ..env(reg)
        };
        assert!(folder_menu::status(&e, LABEL).unavailable.is_some());
        assert!(folder_menu::install(&e, LABEL).is_err());
    }
}
