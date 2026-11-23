//! "Open in GitPoulpi" in the file manager's right-click menu
//! (Settings > Launcher, `gitpoulpi desktop folder-menu`).
//!
//! | OS | Where | What |
//! |---|---|---|
//! | Windows | `HKCU\Software\Classes\{Directory,Directory\Background,Drive}\shell\GitPoulpi[.i<8hex>]` | one verb per installation (optionally in a shared submenu), plus the primary's App Paths `gitpoulpi.exe` |
//! | Linux | each file manager's own mechanism ([`linux`], [`thunar`]) | per installation: a script, service menu or action, with its own coloured icon (submenus: GNOME Files, Caja, Dolphin, PCManFM) |
//! | macOS | `~/Library/Services/<label>.workflow` | one Finder Quick Action per installation ([`macos`]; no submenu, no coloured icon) |
//!
//! Every entry runs `<helper> [launch] --folder <dir>`; the terminal is chosen
//! when it is clicked (`gitpoulpi launch --folder`), so changing the terminal
//! never makes an entry outdated. Everything we write carries [`MARKER`];
//! Install never overwrites what is not ours and Remove deletes only what is.
//! All roots, the `has(program)` probe and the registry come through [`Env`],
//! so tests use temp dirs and a [`MemoryRegistry`](super::registry::MemoryRegistry).
//! Nothing here may run on the UI thread.

pub mod linux;
pub mod macos;
pub mod thunar;
pub mod windows;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::owner::{decide, Facts, Owner};
use super::registry::SharedRegistry;
use super::{State, OWNER_TAG};
use crate::open_in::Os;

pub use super::MARKER;

/// The stable icon name the Linux entries use. `desktop::linux` never matches
/// it, so removing a launcher shortcut never deletes this icon.
pub const ICON_NAME: &str = "gitpoulpi-folder";

/// A place that has a folder menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Manager {
    Explorer,
    Nautilus,
    Dolphin,
    Nemo,
    Thunar,
    Caja,
    PcManFm,
    Finder,
}

impl Manager {
    pub fn as_str(self) -> &'static str {
        match self {
            Manager::Explorer => "explorer",
            Manager::Nautilus => "nautilus",
            Manager::Dolphin => "dolphin",
            Manager::Nemo => "nemo",
            Manager::Thunar => "thunar",
            Manager::Caja => "caja",
            Manager::PcManFm => "pcmanfm",
            Manager::Finder => "finder",
        }
    }

    /// The brand name (never translated).
    pub fn name(self) -> &'static str {
        match self {
            Manager::Explorer => "File Explorer",
            Manager::Nautilus => "GNOME Files",
            Manager::Dolphin => "Dolphin",
            Manager::Nemo => "Nemo",
            Manager::Thunar => "Thunar",
            Manager::Caja => "Caja",
            Manager::PcManFm => "PCManFM",
            Manager::Finder => "Finder",
        }
    }

    /// Programs on `PATH` that show the file manager is installed (any one).
    pub fn probes(self) -> &'static [&'static str] {
        match self {
            Manager::Nautilus => &["nautilus"],
            Manager::Dolphin => &["dolphin"],
            Manager::Nemo => &["nemo"],
            Manager::Thunar => &["thunar"],
            Manager::Caja => &["caja"],
            Manager::PcManFm => &["pcmanfm", "pcmanfm-qt"],
            Manager::Explorer | Manager::Finder => &[],
        }
    }

    /// The managers `os` can have, in the order the page lists them.
    pub fn for_os(os: Os) -> &'static [Manager] {
        match os {
            Os::Windows => &[Manager::Explorer],
            Os::Macos => &[Manager::Finder],
            Os::Linux => &[
                Manager::Nautilus,
                Manager::Dolphin,
                Manager::Nemo,
                Manager::Thunar,
                Manager::Caja,
                Manager::PcManFm,
            ],
        }
    }
}

/// Everything the module reads from the machine.
#[derive(Clone)]
pub struct Env {
    pub os: Os,
    /// Linux: `$XDG_DATA_HOME`.
    pub data_home: PathBuf,
    /// Linux: `$XDG_CONFIG_HOME`.
    pub config_home: PathBuf,
    pub home: PathBuf,
    /// GitPoulpi's own executable.
    pub exe: PathBuf,
    /// What an entry runs ([`super::terminals::folder_helper`]). `None`: the
    /// menu cannot be installed ([`Unavailable::NoHelper`]).
    pub helper: Option<PathBuf>,
    /// Is this program on `PATH`?
    pub has: Arc<dyn Fn(&str) -> bool + Send + Sync>,
    pub registry: SharedRegistry,
    /// The 256 px icon the Linux entries name.
    pub icon_png: Option<Vec<u8>>,
    /// Run the cache refreshers (`gtk-update-icon-cache`, `pbs`). Off in tests.
    pub refresh_hooks: bool,
    /// Who writes (owner tag, entry names).
    pub ctx: super::InstallCtx,
    /// Windows: the installation's colour `.ico` (`desktop::ensure_ico`) the
    /// verbs show. `None`: the exe's own icon.
    pub ico: Option<PathBuf>,
    /// `folder_menu_submenu`: the submenu the entry goes into, `""` = the root.
    pub submenu: String,
}

impl Env {
    /// The real roots for `os`, from the environment.
    pub fn discover(
        os: Os,
        env: &dyn Fn(&str) -> Option<String>,
        home: &Path,
        exe: &Path,
        has: Arc<dyn Fn(&str) -> bool + Send + Sync>,
        registry: SharedRegistry,
    ) -> Env {
        let var = |k: &str| env(k).filter(|v| !v.is_empty()).map(PathBuf::from);
        Env {
            os,
            data_home: var("XDG_DATA_HOME").unwrap_or_else(|| home.join(".local/share")),
            config_home: var("XDG_CONFIG_HOME").unwrap_or_else(|| home.join(".config")),
            home: home.to_path_buf(),
            exe: exe.to_path_buf(),
            helper: super::terminals::folder_helper(os, exe, &|p| p.is_file()),
            has,
            registry,
            icon_png: None,
            refresh_hooks: false,
            ctx: crate::desktop::InstallCtx::legacy(),
            ico: None,
            submenu: String::new(),
        }
    }

    pub(crate) fn helper_path(&self) -> Result<&Path, String> {
        self.helper
            .as_deref()
            .ok_or_else(|| "the gitpoulpi launcher is not next to GitPoulpi".to_string())
    }
}

/// One file manager and what it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub manager: Manager,
    pub detected: bool,
    pub state: State,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unavailable {
    /// No launch helper beside the exe.
    NoHelper,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub entries: Vec<Entry>,
    pub unavailable: Option<Unavailable>,
    /// Reading a store failed (the registry); the states are then unknown.
    pub error: Option<String>,
}

fn detected(env: &Env, m: Manager) -> bool {
    match m {
        Manager::Explorer | Manager::Finder => true,
        _ => m.probes().iter().any(|p| (env.has)(p)),
    }
}

fn state_of(env: &Env, label: &str, m: Manager) -> Result<State, String> {
    match m {
        Manager::Explorer => windows::state(env, label),
        Manager::Finder => Ok(macos::state(env, label)),
        _ => Ok(linux::state(env, label, m)),
    }
}

/// What every manager of this OS holds. Reads disk (and the registry), so not
/// on the UI thread.
pub fn status(env: &Env, label: &str) -> Status {
    let mut error = None;
    let entries = Manager::for_os(env.os)
        .iter()
        .map(|&manager| Entry {
            manager,
            detected: detected(env, manager),
            state: state_of(env, label, manager).unwrap_or_else(|e| {
                error.get_or_insert(e);
                State::NotInstalled
            }),
        })
        .collect();
    Status {
        entries,
        unavailable: env.helper.is_none().then_some(Unavailable::NoHelper),
        error,
    }
}

/// Entries that are ours (current or outdated).
pub fn has_any_ours(s: &Status) -> bool {
    s.entries
        .iter()
        .any(|e| matches!(e.state, State::Installed | State::Outdated))
}

/// Every detected manager that is not foreign holds a current entry.
pub fn all_installed(s: &Status) -> bool {
    let mut any = false;
    for e in s.entries.iter().filter(|e| e.detected) {
        match e.state {
            State::Installed => any = true,
            State::Foreign => {}
            _ => return false,
        }
    }
    any
}

fn write_one(env: &Env, label: &str, m: Manager) -> Result<(), String> {
    match m {
        Manager::Explorer => windows::install(env, label),
        Manager::Finder => macos::install(env, label),
        _ => linux::install(env, label, m),
    }
}

fn remove_one(env: &Env, m: Manager) -> Result<bool, String> {
    match m {
        Manager::Explorer => windows::remove(env),
        Manager::Finder => macos::remove(env),
        _ => linux::remove(env, m),
    }
}

/// Writes the entry of every detected manager that has none or an outdated
/// one (never a foreign one). Returns the managers written.
pub fn install(env: &Env, label: &str) -> Result<Vec<Manager>, String> {
    env.helper_path()?;
    let mut done = Vec::new();
    for &m in Manager::for_os(env.os) {
        if !detected(env, m) {
            continue;
        }
        if matches!(state_of(env, label, m)?, State::Installed | State::Foreign) {
            continue;
        }
        write_one(env, label, m)?;
        done.push(m);
    }
    if env.os == Os::Linux && !done.is_empty() {
        linux::write_icon(env);
    }
    Ok(done)
}

/// Rewrites the entries that are ours and outdated (the exe moved, the icon
/// changed) and installs nothing new. Returns the managers rewritten.
pub fn sync(env: &Env, label: &str) -> Result<Vec<Manager>, String> {
    if env.helper.is_none() {
        return Ok(Vec::new());
    }
    let mut done = Vec::new();
    for &m in Manager::for_os(env.os) {
        if state_of(env, label, m)? == State::Outdated {
            write_one(env, label, m)?;
            done.push(m);
        }
    }
    if env.os == Os::Linux && linux::sync_icon(env) && done.is_empty() {
        // Only the picture changed (another logo color): report it as a
        // change of every entry that names it.
        done.extend(
            Manager::for_os(env.os)
                .iter()
                .copied()
                .filter(|&m| state_of(env, label, m).is_ok_and(|s| s == State::Installed)),
        );
    }
    Ok(done)
}

/// Removes everything of ours from every manager, detected or not. Returns
/// the managers that held something.
pub fn remove(env: &Env) -> Result<Vec<Manager>, String> {
    let mut done = Vec::new();
    for &m in Manager::for_os(env.os) {
        if remove_one(env, m)? {
            done.push(m);
        }
    }
    if env.os == Os::Linux {
        linux::remove_icon_if_unused(env);
    }
    Ok(done)
}

/// `old` is an installation that stopped being the primary and cannot rewrite
/// its own entries now. On Windows its plain verbs move to its own names
/// without a moment without an entry (its next sync rewrites them); elsewhere
/// the entries under the primary's names are [`remove`]d.
pub fn demote(old: &Env) -> Result<Vec<Manager>, String> {
    if old.os == Os::Windows {
        return Ok(if windows::demote(old)? {
            vec![Manager::Explorer]
        } else {
            Vec::new()
        });
    }
    remove(old)
}

/// After [`install`] wrote `new`'s entries: removes what `old` (the same
/// installation in its previous role, so the old names) left under the old
/// names. Installing already replaced every entry of ours where entries are
/// found by their owner tag (Explorer, Finder, GNOME Files, Caja), and
/// [`remove`] there would delete the new entry as well: those are skipped.
/// The managers that keep fixed names per role (the primary has the plain
/// ones) are cleaned by `old`'s names only. Returns the managers cleaned.
pub fn remove_replaced(old: &Env, new: &Env) -> Result<Vec<Manager>, String> {
    if old.ctx.is_primary() == new.ctx.is_primary() {
        return Ok(Vec::new());
    }
    let mut done = Vec::new();
    for &m in Manager::for_os(old.os) {
        if matches!(
            m,
            Manager::Explorer | Manager::Finder | Manager::Nautilus | Manager::Caja
        ) {
            continue;
        }
        if remove_one(old, m)? {
            done.push(m);
        }
    }
    if old.os == Os::Linux {
        linux::remove_icon_if_unused(old);
    }
    Ok(done)
}

/// The managers' brand names, joined for a message.
pub fn names(managers: &[Manager]) -> String {
    managers
        .iter()
        .map(|m| m.name())
        .collect::<Vec<_>>()
        .join(", ")
}

/// The running installation as the Linux and macOS writers see it: which
/// names it owns and which submenu it asks for.
#[derive(Debug, Clone)]
pub(crate) struct Who {
    pub id: String,
    /// `i<8hex>`.
    pub short: String,
    /// The primary keeps the legacy file names and the legacy marker.
    pub primary: bool,
    /// The submenu label, trimmed; empty = the root.
    pub submenu: String,
}

impl Who {
    pub(crate) fn of(env: &Env) -> Who {
        Who {
            id: env.ctx.id.clone(),
            short: env.ctx.short.clone(),
            primary: env.ctx.is_primary(),
            submenu: env.submenu.trim().replace(|c: char| c.is_control(), " "),
        }
    }

    /// `""` for the primary, `-i<8hex>` otherwise: appended to every name.
    pub(crate) fn suffix(&self) -> String {
        if self.primary {
            String::new()
        } else {
            format!("-{}", self.short)
        }
    }

    /// `gitpoulpi-folder[-i<8hex>]`: this installation's icon.
    pub(crate) fn icon_name(&self) -> String {
        format!("{ICON_NAME}{}", self.suffix())
    }

    /// The owner lines of a file, each ending in a newline, behind `prefix`
    /// (`""`, or `"# "` in a script). Only the primary writes the legacy
    /// marker, so old builds never see the others.
    pub(crate) fn owner_lines(&self, prefix: &str) -> String {
        let mut out = String::new();
        if self.primary {
            out.push_str(&format!("{prefix}{MARKER}\n"));
        }
        if !self.id.is_empty() {
            out.push_str(&format!("{prefix}{OWNER_TAG}={}\n", self.id));
        }
        out
    }

    /// The normalized submenu name, `None` at the root.
    pub(crate) fn submenu_norm(&self) -> Option<String> {
        Some(normalize(&self.submenu)).filter(|n| !n.is_empty())
    }
}

/// Submenu names are equal when trimmed and lower-cased alike.
pub(crate) fn normalize(name: &str) -> String {
    name.trim().to_lowercase()
}

/// 8 hex digits identifying a normalized submenu name.
pub(crate) fn menu_hash(norm: &str) -> String {
    format!(
        "{:08x}",
        crate::installation::fnv1a64(norm.as_bytes()) & 0xffff_ffff
    )
}

/// What the owner rule needs from a text file: the tag and the legacy marker,
/// each a line of its own (behind `# ` in a script).
pub(crate) fn facts_of(text: &str) -> Facts<'_> {
    let mut facts = Facts::default();
    let tag_prefix = format!("{OWNER_TAG}=");
    for line in text.lines() {
        let l = line.strip_prefix("# ").unwrap_or(line);
        if l == MARKER {
            facts.legacy_marker = true;
        } else if let Some(t) = l.strip_prefix(&tag_prefix) {
            facts.tag = Some(t);
        }
    }
    facts
}

/// May the running installation rewrite and remove something with these facts?
pub(crate) fn may_touch(env: &Env, facts: &Facts) -> bool {
    matches!(decide(facts, &env.ctx), Owner::Mine | Owner::Adopt)
}

/// [`may_touch`] for a text file.
pub(crate) fn is_mine(env: &Env, text: &str) -> bool {
    may_touch(env, &facts_of(text))
}

/// Fixtures shared by the Linux and macOS tests: installations with real ids.
#[cfg(test)]
pub(crate) mod testing {
    use super::super::{InstallCtx, KnownInstall};

    pub const A: &str = "aaaaaaaa11111111";
    pub const B: &str = "bbbbbbbb22222222";

    /// The context of installation `id`, with `primary` as the config's primary.
    pub fn ctx(id: &str, primary: Option<&str>) -> InstallCtx {
        InstallCtx {
            id: id.to_string(),
            short: crate::installation::short_id(id),
            name: format!("GitPoulpi {id}"),
            primary: primary.map(str::to_string),
            alias_claims: Vec::new(),
            known: [A, B]
                .iter()
                .map(|k| KnownInstall {
                    id: k.to_string(),
                    exists: true,
                })
                .collect(),
        }
    }
}
