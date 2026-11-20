//! "Open in GitPoulpi" in the file manager's right-click menu
//! (Settings > Launcher, `gitpoulpi desktop folder-menu`).
//!
//! | OS | Where | What |
//! |---|---|---|
//! | Windows | `HKCU\Software\Classes\{Directory,Directory\Background,Drive}\shell\GitPoulpi` | one verb, plus App Paths `gitpoulpi.exe` |
//! | Linux | each file manager's own mechanism ([`linux`], [`thunar`]) | a script, service menu or action |
//! | macOS | `~/Library/Services/<label>.workflow` | a Finder Quick Action ([`macos`]) |
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

use super::registry::SharedRegistry;
use super::State;
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
    /// Who writes (owner tag, entry names). Not read yet.
    pub ctx: super::InstallCtx,
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

/// The managers' brand names, joined for a message.
pub fn names(managers: &[Manager]) -> String {
    managers
        .iter()
        .map(|m| m.name())
        .collect::<Vec<_>>()
        .join(", ")
}

/// A file of ours: `MARKER` is a line of its own (`# MARKER` in a script).
pub(crate) fn is_ours(text: &str) -> bool {
    text.lines()
        .any(|l| l == MARKER || l.strip_prefix("# ") == Some(MARKER))
}
