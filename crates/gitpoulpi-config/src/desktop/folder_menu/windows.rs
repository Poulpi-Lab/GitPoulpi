//! Windows Explorer: shell verbs under `HKCU\Software\Classes` (a folder, a
//! folder's background, a drive), plus the App Paths key that lets
//! `gitpoulpi` typed in the address bar start the console-less launcher. All
//! of it is data over the [`Registry`](crate::desktop::registry::Registry)
//! trait, so it runs and is tested on every OS.
//!
//! Every installation has its own verb with its own colour icon:
//! `...\shell\GitPoulpi` for the primary (the name old builds know) and
//! `...\shell\GitPoulpi.i<8hex>` for the others. With `folder_menu_submenu`
//! the verb sits in a shared cascade instead: the container
//! `...\shell\GitPoulpi.Menu.m<8hex of the normalized name>` (label, no icon,
//! `SubCommands=""`, tag [`MENU_TAG`], no owner) holds the members at
//! `...\GitPoulpi.Menu.m...\shell\i<8hex>`. Membership is the tagged members;
//! the last one out deletes the container, and only if it has no subkeys left
//! (`DeleteIfNoSubkeys`; `DeleteTree` is never used on a container).
//!
//! Ownership: a verb is ours when it carries our id in [`OWNER_TAG`], or,
//! for the primary, the legacy marker `X-GitPoulpi=1` (only the primary's
//! root verb and App Paths carry it, so old builds never see another
//! installation's keys). Anything else is foreign and never written or
//! deleted. Remove deletes our keys, then the parents (`Directory\shell`,
//! ...) only when nothing is left in them, which restores what Install found.
//! Windows 11 shows these verbs only in the classic menu (Show more options).

use std::path::Path;

use super::Env;
use crate::desktop::owner::{decide, Facts, Owner};
use crate::desktop::registry::{RegOp, Values};
use crate::desktop::{State, OWNER_TAG};

const CLASSES_KEY: &str = "Software\\Classes";
const APP_PATHS: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\App Paths";
const MARK_NAME: &str = "X-GitPoulpi";
const MARK_VALUE: &str = "1";
/// The submenu container's tag (value: the normalized name), never an owner.
pub const MENU_TAG: &str = "X-GitPoulpi-Menu";
const MENU_PREFIX: &str = "GitPoulpi.Menu.m";
const LEGACY_VERB: &str = "GitPoulpi";

/// The classes that get the verb.
const CLASSES: [&str; 3] = ["Directory", "Directory\\Background", "Drive"];

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
    format!("{CLASSES_KEY}\\{rel}")
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

fn value<'a>(values: &'a Values, name: &str) -> Option<&'a str> {
    values
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

fn marked(values: &Values) -> bool {
    value(values, MARK_NAME) == Some(MARK_VALUE)
}

/// Is the App Paths key `<name>` there and ours? (`None`: absent.)
pub fn app_path_is_ours(
    registry: &dyn crate::desktop::registry::Registry,
    name: &str,
) -> Result<Option<bool>, String> {
    let got = registry.read(&[app_path_key(name)])?;
    Ok(got.into_iter().next().flatten().map(|(v, _)| marked(&v)))
}

/// The submenu a setting names: trimmed and lower-cased, so two installations
/// that spell it differently share one container.
fn normalize(name: &str) -> String {
    name.trim().to_lowercase()
}

/// Where this installation's verbs go.
struct Layout {
    id: String,
    short: String,
    primary: bool,
    sub: Option<Sub>,
}

struct Sub {
    /// The label (as typed, trimmed).
    label: String,
    /// The normalized name the container is tagged with.
    norm: String,
    /// The container's key name under `shell`.
    key_name: String,
}

impl Layout {
    fn of(env: &Env) -> Layout {
        let sub = Some(env.submenu.trim()).filter(|n| !n.is_empty()).map(|n| {
            let norm = normalize(n);
            let hash = crate::installation::fnv1a64(norm.as_bytes());
            Sub {
                label: n.to_string(),
                key_name: format!("{MENU_PREFIX}{:08x}", hash & 0xffff_ffff),
                norm,
            }
        });
        Layout {
            id: env.ctx.id.clone(),
            short: env.ctx.short.clone(),
            primary: env.ctx.is_primary(),
            sub,
        }
    }

    /// The root verb's key name.
    fn verb(&self) -> String {
        if self.primary {
            LEGACY_VERB.to_string()
        } else {
            format!("{LEGACY_VERB}.{}", self.short)
        }
    }

    fn root_rel(&self, class: &str) -> String {
        format!("{class}\\shell\\{}", self.verb())
    }

    fn member_rel(&self, class: &str, container: &str) -> String {
        format!("{class}\\shell\\{container}\\shell\\{}", self.short)
    }

    /// Where the verb for `class` should be.
    fn wanted_rel(&self, class: &str) -> String {
        match &self.sub {
            Some(sub) => self.member_rel(class, &sub.key_name),
            None => self.root_rel(class),
        }
    }

    /// Does the verb carry the legacy marker (the primary's root verb)?
    fn legacy_marker(&self) -> bool {
        self.primary && self.sub.is_none()
    }
}

fn icon_value(env: &Env, helper: &Path) -> String {
    match &env.ico {
        Some(ico) => format!("\"{}\"", ico.display()),
        None => format!("\"{}\",0", helper.with_file_name("gitpoulpi.exe").display()),
    }
}

/// The keys (relative to `HKCU`) and values Install writes for the verbs, the
/// container first. `icon` is the `Icon` value.
fn wanted_keys(lay: &Layout, helper: &Path, label: &str, icon: &str) -> Vec<(String, Values)> {
    let command = format!("\"{}\" --folder \"%V\"", helper.display());
    let mut out = Vec::new();
    for class in CLASSES {
        if let Some(sub) = &lay.sub {
            out.push((
                full(&format!("{class}\\shell\\{}", sub.key_name)),
                vec![
                    (s("MUIVerb"), sub.label.clone()),
                    (s("SubCommands"), s("")),
                    (s(MENU_TAG), sub.norm.clone()),
                ],
            ));
        }
        let key = full(&lay.wanted_rel(class));
        let mut values = vec![
            (s(""), s(label)),
            (s("MUIVerb"), s(label)),
            (s("Icon"), s(icon)),
        ];
        if !lay.id.is_empty() {
            values.push((s(OWNER_TAG), lay.id.clone()));
        }
        if lay.legacy_marker() {
            values.push((s(MARK_NAME), s(MARK_VALUE)));
        }
        out.push((key.clone(), values));
        out.push((format!("{key}\\command"), vec![(s(""), command.clone())]));
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

/// Does this verb belong to the running installation (or may it adopt it)?
fn mine(env: &Env, values: &Values) -> bool {
    let facts = Facts {
        tag: value(values, OWNER_TAG),
        legacy_marker: marked(values),
        claimed_by: None,
        names_my_exe: false,
    };
    matches!(decide(&facts, &env.ctx), Owner::Mine | Owner::Adopt)
}

type Read = Option<(Values, bool)>;

/// Reads `keys` into a lookup by lower-cased key.
fn read_map(env: &Env, keys: &[String]) -> Result<std::collections::HashMap<String, Read>, String> {
    let got = env.registry.read(keys)?;
    Ok(keys
        .iter()
        .map(|k| k.to_ascii_lowercase())
        .zip(got)
        .collect())
}

fn get<'a>(map: &'a std::collections::HashMap<String, Read>, key: &str) -> &'a Read {
    map.get(&key.to_ascii_lowercase()).unwrap_or(&None)
}

/// Our verbs that are not where the settings say: the root verb after a move
/// into a submenu, a member of another submenu. Relative keys.
fn strays(env: &Env, lay: &Layout) -> Result<Vec<String>, String> {
    let mut candidates = Vec::new();
    for class in CLASSES {
        let wanted = lay.wanted_rel(class).to_ascii_lowercase();
        let names = env.registry.subkeys(&full(&format!("{class}\\shell")))?;
        for name in names {
            let rel = if name.eq_ignore_ascii_case(&lay.verb()) {
                lay.root_rel(class)
            } else if name
                .get(..MENU_PREFIX.len())
                .is_some_and(|p| p.eq_ignore_ascii_case(MENU_PREFIX))
            {
                lay.member_rel(class, &name)
            } else {
                continue;
            };
            if rel.to_ascii_lowercase() != wanted {
                candidates.push(rel);
            }
        }
    }
    let keys: Vec<String> = candidates.iter().map(|r| full(r)).collect();
    let got = read_map(env, &keys)?;
    Ok(candidates
        .into_iter()
        .filter(|rel| {
            get(&got, &full(rel))
                .as_ref()
                .is_some_and(|(v, _)| mine(env, v))
        })
        .collect())
}

/// The ops that delete `entries` (our verbs, relative keys) and then every
/// container they leave empty. A container is only ever deleted by
/// `DeleteIfEmpty` / `DeleteIfNoSubkeys`, which fail while a member is left.
fn delete_entries(entries: &[String]) -> Vec<RegOp> {
    let mut ops = Vec::new();
    for rel in entries {
        ops.push(RegOp::DeleteTree(full(rel)));
    }
    for rel in entries {
        // `<class>\shell\<container>\shell\<short>`: the container is two up.
        let Some((members, _)) = rel.rsplit_once('\\') else {
            continue;
        };
        let Some(container) = members.strip_suffix("\\shell") else {
            continue;
        };
        let last = container.rsplit('\\').next().unwrap_or_default();
        if !last
            .get(..MENU_PREFIX.len())
            .is_some_and(|p| p.eq_ignore_ascii_case(MENU_PREFIX))
        {
            continue;
        }
        ops.push(RegOp::DeleteIfEmpty(full(members)));
        ops.push(RegOp::DeleteIfNoSubkeys(full(container)));
    }
    ops
}

pub(super) fn state(env: &Env, label: &str) -> Result<State, String> {
    let lay = Layout::of(env);
    let app_key = app_path_key("gitpoulpi.exe");
    // The label and icon come from our own verbs, so a language change alone
    // (or a status read without the icon) is not "outdated".
    let probe = wanted_keys(&lay, Path::new(""), "", "");
    let mut keys: Vec<String> = probe.iter().map(|(k, _)| k.clone()).collect();
    keys.push(app_key.clone());
    let got = read_map(env, &keys)?;
    let entry_keys: Vec<String> = CLASSES.iter().map(|c| full(&lay.wanted_rel(c))).collect();
    let at: Vec<&Read> = entry_keys.iter().map(|k| get(&got, k)).collect();
    if at
        .iter()
        .any(|g| g.as_ref().is_some_and(|(v, _)| !mine(env, v)))
    {
        return Ok(State::Foreign);
    }
    let stray = strays(env, &lay)?;
    let have = at.iter().any(|g| g.is_some());
    if !have {
        return Ok(if stray.is_empty() {
            State::NotInstalled
        } else {
            State::Outdated
        });
    }
    let Some(helper) = env.helper.as_deref() else {
        return Ok(State::Outdated);
    };
    let found = at
        .iter()
        .filter_map(|g| g.as_ref())
        .find_map(|(v, _)| value(v, "MUIVerb"))
        .map_or_else(|| label.to_string(), str::to_string);
    let found_icon = at
        .iter()
        .filter_map(|g| g.as_ref())
        .find_map(|(v, _)| value(v, "Icon"))
        .map(str::to_string);
    let icon = match (&env.ico, found_icon) {
        (None, Some(found)) => found,
        _ => icon_value(env, helper),
    };
    let mut current = stray.is_empty();
    for (key, want) in wanted_keys(&lay, helper, &found, &icon) {
        match get(&got, &key) {
            // The container's label is shared and may be spelled by another
            // installation: only its tag and cascade have to be there.
            Some((vals, _)) if lay.sub.is_some() && value(&want, MENU_TAG).is_some() => {
                current &= value(vals, MENU_TAG) == value(&want, MENU_TAG)
                    && value(vals, "SubCommands").is_some();
            }
            Some((vals, _)) if norm(vals) == norm(&want) => {}
            _ => current = false,
        }
    }
    // App Paths belong to the primary: ours must match, a foreign one is left
    // alone and ignored.
    if lay.primary {
        match get(&got, &app_key) {
            None => current = false,
            Some((vals, _)) if marked(vals) => {
                current &= norm(vals) == norm(&app_path_values(helper));
            }
            Some(_) => {}
        }
    }
    Ok(if current {
        State::Installed
    } else {
        State::Outdated
    })
}

pub(super) fn install(env: &Env, label: &str) -> Result<(), String> {
    let helper = env.helper_path()?;
    let lay = Layout::of(env);
    let app_key = app_path_key("gitpoulpi.exe");
    let wanted = wanted_keys(&lay, helper, label, &icon_value(env, helper));
    let mut keys: Vec<String> = CLASSES.iter().map(|c| full(&lay.wanted_rel(c))).collect();
    let containers: Vec<String> = wanted
        .iter()
        .filter(|(_, v)| value(v, MENU_TAG).is_some())
        .map(|(k, _)| k.clone())
        .collect();
    keys.extend(containers.iter().cloned());
    keys.push(app_key.clone());
    let got = read_map(env, &keys)?;
    let mut ops = delete_entries(&strays(env, &lay)?);
    for class in CLASSES {
        let key = full(&lay.wanted_rel(class));
        match get(&got, &key) {
            Some((v, _)) if !mine(env, v) => {
                return Err(format!("{key} is not ours; not overwriting it"));
            }
            // Drop stale values and keys before writing again.
            Some(_) => ops.push(RegOp::DeleteTree(key)),
            None => {}
        }
    }
    for key in &containers {
        if let Some((v, _)) = get(&got, key) {
            if value(v, MENU_TAG).is_none() {
                return Err(format!(
                    "{key} is not a GitPoulpi submenu; not overwriting it"
                ));
            }
        }
    }
    for (key, values) in wanted {
        for (name, value) in values {
            ops.push(RegOp::Set {
                key: key.clone(),
                name,
                value,
            });
        }
    }
    // App Paths: only the primary writes it; skip a key somebody else owns.
    if lay.primary && get(&got, &app_key).as_ref().is_none_or(|(v, _)| marked(v)) {
        ops.extend(app_path_ops("gitpoulpi.exe", helper));
    }
    env.registry.apply(&ops)
}

pub(super) fn remove(env: &Env) -> Result<bool, String> {
    let lay = Layout::of(env);
    let app_key = app_path_key("gitpoulpi.exe");
    let mut entries = strays(env, &lay)?;
    let wanted: Vec<String> = CLASSES.iter().map(|c| lay.wanted_rel(c)).collect();
    let mut keys: Vec<String> = wanted.iter().map(|r| full(r)).collect();
    keys.push(app_key.clone());
    let got = read_map(env, &keys)?;
    for rel in wanted {
        if get(&got, &full(&rel))
            .as_ref()
            .is_some_and(|(v, _)| mine(env, v))
        {
            entries.push(rel);
        }
    }
    let mut ops = delete_entries(&entries);
    let mut any = !entries.is_empty();
    if lay.primary && get(&got, &app_key).as_ref().is_some_and(|(v, _)| marked(v)) {
        ops.push(app_path_remove("gitpoulpi.exe"));
        any = true;
    }
    if !any {
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

    /// The legacy verbs (the primary's), relative to `Software\Classes`.
    const ROOTS: [&str; 3] = [
        "Directory\\shell\\GitPoulpi",
        "Directory\\Background\\shell\\GitPoulpi",
        "Drive\\shell\\GitPoulpi",
    ];

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
            ctx: crate::desktop::InstallCtx::legacy(),
            ico: None,
            submenu: String::new(),
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
        reg.apply(&[set(CLASSES_KEY, "keep", "1"), set(APP_PATHS, "keep", "1")])
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

    // ---- per-installation verbs and shared submenus ----

    const A: &str = "aaaaaaaa11111111";
    const B: &str = "bbbbbbbb22222222";

    fn env_for(
        reg: &Arc<MemoryRegistry>,
        id: &str,
        primary: Option<&str>,
        submenu: &str,
        ico: Option<&str>,
    ) -> Env {
        Env {
            helper: Some(PathBuf::from(format!("C:/gp/{id}/gitpoulpi-launch.exe"))),
            ctx: crate::desktop::InstallCtx {
                id: id.to_string(),
                short: crate::installation::short_id(id),
                name: format!("GitPoulpi {id}"),
                primary: primary.map(str::to_string),
                known: Vec::new(),
                alias_claims: Vec::new(),
            },
            ico: ico.map(PathBuf::from),
            submenu: submenu.to_string(),
            ..env(reg.clone())
        }
    }

    fn short(id: &str) -> String {
        crate::installation::short_id(id)
    }

    fn values(reg: &MemoryRegistry, rel: &str) -> Option<Values> {
        reg.read(&[full(rel)]).unwrap().remove(0).map(|(v, _)| v)
    }

    fn val(reg: &MemoryRegistry, rel: &str, name: &str) -> Option<String> {
        values(reg, rel).and_then(|v| value(&v, name).map(str::to_string))
    }

    fn state_of(e: &Env) -> State {
        folder_menu::status(e, LABEL).entries[0].state
    }

    /// The container names under `Directory\shell`.
    fn containers(reg: &MemoryRegistry) -> Vec<String> {
        reg.subkeys(&full("Directory\\shell"))
            .unwrap()
            .into_iter()
            .filter(|n| n.starts_with("gitpoulpi.menu."))
            .collect()
    }

    #[test]
    fn each_installation_writes_its_own_verb_with_its_colour_icon() {
        let reg = Arc::new(MemoryRegistry::new());
        let a = env_for(&reg, A, None, "", Some("C:/a/gitpoulpi-aaa.ico"));
        let b = env_for(&reg, B, None, "", Some("C:/b/gitpoulpi-bbb.ico"));
        folder_menu::install(&a, LABEL).unwrap();
        folder_menu::install(&b, LABEL).unwrap();
        for class in CLASSES {
            for (id, ico) in [(A, "C:/a/gitpoulpi-aaa.ico"), (B, "C:/b/gitpoulpi-bbb.ico")] {
                let rel = format!("{class}\\shell\\GitPoulpi.{}", short(id));
                assert_eq!(val(&reg, &rel, "Icon").unwrap(), format!("\"{ico}\""));
                assert_eq!(val(&reg, &rel, "MUIVerb").unwrap(), LABEL);
                assert_eq!(val(&reg, &rel, OWNER_TAG).unwrap(), id);
                assert_eq!(val(&reg, &rel, MARK_NAME), None);
                assert!(val(&reg, &format!("{rel}\\command"), "")
                    .unwrap()
                    .contains(&format!("C:/gp/{id}/")));
            }
        }
        assert_eq!(state_of(&a), State::Installed);
        assert_eq!(state_of(&b), State::Installed);
        // A new colour is a new .ico: only that installation is outdated, and
        // sync rewrites only its Icon.
        let a2 = Env {
            ico: Some(PathBuf::from("C:/a/gitpoulpi-new.ico")),
            ..a.clone()
        };
        assert_eq!(state_of(&a2), State::Outdated);
        assert_eq!(state_of(&b), State::Installed);
        assert_eq!(folder_menu::sync(&a2, LABEL).unwrap(), [Manager::Explorer]);
        let rel = format!("Directory\\shell\\GitPoulpi.{}", short(A));
        assert_eq!(
            val(&reg, &rel, "Icon").unwrap(),
            "\"C:/a/gitpoulpi-new.ico\""
        );
        assert_eq!(state_of(&a2), State::Installed);
        // A status read without the icon does not call a coloured verb outdated.
        let plain = Env { ico: None, ..a2 };
        assert_eq!(state_of(&plain), State::Installed);
    }

    #[test]
    fn the_primary_keeps_the_legacy_key_and_app_paths() {
        let reg = Arc::new(MemoryRegistry::new());
        reg.apply(&[set(CLASSES_KEY, "keep", "1"), set(APP_PATHS, "keep", "1")])
            .unwrap();
        let before = reg.dump();
        let p = env_for(&reg, A, Some(A), "", Some("C:/a/x.ico"));
        folder_menu::install(&p, LABEL).unwrap();
        for root in ROOTS {
            assert_eq!(val(&reg, root, MARK_NAME).as_deref(), Some("1"));
            assert_eq!(val(&reg, root, OWNER_TAG).as_deref(), Some(A));
            assert_eq!(val(&reg, root, "Icon").as_deref(), Some("\"C:/a/x.ico\""));
        }
        let app = app_path_key("gitpoulpi.exe");
        let got = reg.read(&[app]).unwrap().remove(0).unwrap().0;
        assert!(marked(&got));
        assert_eq!(state_of(&p), State::Installed);
        // Removing the primary takes both away.
        assert_eq!(folder_menu::remove(&p).unwrap(), [Manager::Explorer]);
        assert_eq!(reg.dump(), before);
    }

    #[test]
    fn no_primary_means_no_app_paths_gitpoulpi() {
        let reg = Arc::new(MemoryRegistry::new());
        let a = env_for(&reg, A, None, "", None);
        folder_menu::install(&a, LABEL).unwrap();
        let app = app_path_key("gitpoulpi.exe");
        assert_eq!(reg.read(std::slice::from_ref(&app)).unwrap()[0], None);
        assert!(reg.dump().iter().all(|(k, _, _)| !k.contains("app paths")));
        // Another installation being the primary does not make us write it.
        let b = env_for(&reg, B, Some(A), "", None);
        folder_menu::install(&b, LABEL).unwrap();
        assert_eq!(reg.read(std::slice::from_ref(&app)).unwrap()[0], None);
        assert_eq!(state_of(&b), State::Installed);
        // ...nor remove the primary's.
        let p = env_for(&reg, A, Some(A), "", None);
        folder_menu::install(&p, LABEL).unwrap();
        folder_menu::remove(&b).unwrap();
        assert!(reg.read(&[app]).unwrap()[0].is_some());
    }

    #[test]
    fn two_installations_with_one_submenu_share_one_container() {
        let reg = Arc::new(MemoryRegistry::new());
        let a = env_for(&reg, A, None, "Tools", Some("C:/a/x.ico"));
        // Same name, spelled differently: compared trimmed and lower-cased.
        let b = env_for(&reg, B, Some(B), "  tOOls ", Some("C:/b/x.ico"));
        folder_menu::install(&a, LABEL).unwrap();
        folder_menu::install(&b, LABEL).unwrap();
        let names = containers(&reg);
        assert_eq!(names.len(), 1, "{names:?}");
        for class in CLASSES {
            let c = format!("{class}\\shell\\{}", names[0]);
            assert_eq!(val(&reg, &c, "SubCommands").as_deref(), Some(""));
            assert_eq!(val(&reg, &c, MENU_TAG).as_deref(), Some("tools"));
            // No icon (it would dangle), no owner, no legacy marker.
            for n in ["Icon", OWNER_TAG, MARK_NAME] {
                assert_eq!(val(&reg, &c, n), None, "{n}");
            }
            assert!(val(&reg, &c, "MUIVerb").is_some());
            for (id, ico) in [(A, "C:/a/x.ico"), (B, "C:/b/x.ico")] {
                let m = format!("{c}\\shell\\{}", short(id));
                assert_eq!(val(&reg, &m, OWNER_TAG).as_deref(), Some(id));
                assert_eq!(val(&reg, &m, "Icon").unwrap(), format!("\"{ico}\""));
                assert!(values(&reg, &format!("{m}\\command")).is_some());
            }
            // No root verb of either installation.
            assert!(values(&reg, &format!("{class}\\shell\\GitPoulpi")).is_none());
            assert!(values(&reg, &format!("{class}\\shell\\GitPoulpi.{}", short(A))).is_none());
        }
        assert_eq!(state_of(&a), State::Installed);
        assert_eq!(state_of(&b), State::Installed);
    }

    #[test]
    fn the_last_one_out_removes_the_container_and_not_before() {
        let reg = Arc::new(MemoryRegistry::new());
        reg.apply(&[set(&full("Directory\\shell\\Mine"), "", "x")])
            .unwrap();
        let before = reg.dump();
        let a = env_for(&reg, A, None, "Tools", None);
        let b = env_for(&reg, B, None, "Tools", None);
        folder_menu::install(&a, LABEL).unwrap();
        folder_menu::install(&b, LABEL).unwrap();
        assert_eq!(folder_menu::remove(&a).unwrap(), [Manager::Explorer]);
        let names = containers(&reg);
        assert_eq!(names.len(), 1, "the container stays while B is in it");
        let m = format!("Directory\\shell\\{}\\shell\\{}", names[0], short(B));
        assert!(values(&reg, &m).is_some());
        assert!(values(&reg, &m.replace(&short(B), &short(A))).is_none());
        assert_eq!(state_of(&b), State::Installed);
        // The last one out takes the container and restores the registry.
        assert_eq!(folder_menu::remove(&b).unwrap(), [Manager::Explorer]);
        assert!(containers(&reg).is_empty());
        assert_eq!(reg.dump(), before);
        // A second removal has nothing to do.
        assert!(folder_menu::remove(&b).unwrap().is_empty());
    }

    #[test]
    fn changing_the_submenu_moves_only_my_child() {
        let reg = Arc::new(MemoryRegistry::new());
        let a = env_for(&reg, A, None, "Tools", None);
        let b = env_for(&reg, B, None, "Tools", None);
        folder_menu::install(&a, LABEL).unwrap();
        folder_menu::install(&b, LABEL).unwrap();
        let tools = containers(&reg).remove(0);
        // A moves to another submenu: out of Tools (B stays), into the new one.
        let a2 = Env {
            submenu: "Other".into(),
            ..a.clone()
        };
        assert_eq!(state_of(&a2), State::Outdated);
        assert_eq!(folder_menu::sync(&a2, LABEL).unwrap(), [Manager::Explorer]);
        assert_eq!(state_of(&a2), State::Installed);
        assert_eq!(state_of(&b), State::Installed);
        let names = containers(&reg);
        assert_eq!(names.len(), 2, "{names:?}");
        let in_tools = |id: &str| {
            values(
                &reg,
                &format!("Directory\\shell\\{tools}\\shell\\{}", short(id)),
            )
            .is_some()
        };
        assert!(in_tools(B) && !in_tools(A));
        // A moves to the root: the Other container is gone, Tools stays for B.
        let a3 = Env {
            submenu: String::new(),
            ..a
        };
        assert_eq!(state_of(&a3), State::Outdated);
        folder_menu::sync(&a3, LABEL).unwrap();
        assert_eq!(state_of(&a3), State::Installed);
        assert_eq!(containers(&reg), std::slice::from_ref(&tools));
        assert!(in_tools(B));
        assert!(values(&reg, &format!("Directory\\shell\\GitPoulpi.{}", short(A))).is_some());
        // And B is untouched by the root install of A.
        assert_eq!(state_of(&b), State::Installed);
    }

    #[test]
    fn old_builds_cannot_see_non_primary_keys() {
        let reg = Arc::new(MemoryRegistry::new());
        // Somebody else is the primary; this one has a submenu, the third none.
        let a = env_for(&reg, A, Some(B), "Tools", None);
        let c = env_for(&reg, "cccccccc33333333", Some(B), "", None);
        folder_menu::install(&a, LABEL).unwrap();
        folder_menu::install(&c, LABEL).unwrap();
        // An old build looks at the fixed legacy keys and App Paths, and
        // matches only the exact marker value.
        let old: Vec<String> = ROOTS
            .iter()
            .map(|r| full(r))
            .chain([app_path_key("gitpoulpi.exe")])
            .collect();
        assert!(reg.read(&old).unwrap().iter().all(Option::is_none));
        assert!(
            reg.dump()
                .iter()
                .all(|(_, n, v)| !(n == MARK_NAME && v == MARK_VALUE)),
            "{:?}",
            reg.dump()
        );
        // Their removal never touches the legacy keys either.
        reg.apply(&[set(&full(ROOTS[0]), MARK_NAME, MARK_VALUE)])
            .unwrap();
        folder_menu::remove(&a).unwrap();
        folder_menu::remove(&c).unwrap();
        assert!(values(&reg, ROOTS[0]).is_some());
    }

    #[test]
    fn an_old_primary_verb_without_a_tag_is_adopted_and_tagged() {
        let reg = Arc::new(MemoryRegistry::new());
        let legacy = env(reg.clone());
        folder_menu::install(&legacy, LABEL).unwrap();
        let p = env_for(&reg, A, Some(A), "", None);
        // What an old build wrote carries the marker only.
        assert_eq!(val(&reg, ROOTS[0], OWNER_TAG), None);
        assert_eq!(state_of(&p), State::Outdated);
        folder_menu::sync(&p, LABEL).unwrap();
        assert_eq!(val(&reg, ROOTS[0], OWNER_TAG).as_deref(), Some(A));
        assert_eq!(state_of(&p), State::Installed);
    }

    #[test]
    fn another_installations_verb_is_never_ours() {
        let reg = Arc::new(MemoryRegistry::new());
        let a = env_for(&reg, A, None, "", None);
        folder_menu::install(&a, LABEL).unwrap();
        // The same key names but another owner: foreign for us.
        for class in CLASSES {
            let rel = format!("{class}\\shell\\GitPoulpi.{}", short(A));
            reg.apply(&[set(&full(&rel), OWNER_TAG, B)]).unwrap();
        }
        let before = reg.dump();
        assert_eq!(state_of(&a), State::Foreign);
        assert!(folder_menu::install(&a, LABEL).unwrap().is_empty());
        assert!(folder_menu::remove(&a).unwrap().is_empty());
        assert_eq!(reg.dump(), before);
    }
}
