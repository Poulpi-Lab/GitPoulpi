//! The scriptable twin of Settings > Launcher, for
//! `gitpoulpi desktop shortcut status|install|remove`. Same code as the
//! screen (`launcher`), run on the calling thread: a CLI has no frame to keep.

use gitpoulpi_config::desktop::{Place, State};
use gitpoulpi_config::open_in::Os;

use crate::launcher::{self, Done, Env, Op};

/// One shortcut's state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutState {
    /// `launcher`, `desktop` or `startmenu`.
    pub place: &'static str,
    /// `not_installed`, `installed`, `outdated` or `foreign`.
    pub state: &'static str,
    /// The icon file is not the one the current logo color makes.
    pub icon_stale: bool,
}

/// What `status` reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// The terminal `ui.launch_terminal` resolves to (`None`: none found).
    pub terminal: Option<&'static str>,
    /// The terminals found on this machine.
    pub terminals: Vec<&'static str>,
    pub shortcuts: Vec<ShortcutState>,
}

pub(crate) fn settings() -> (String, String) {
    let ui = gitpoulpi_config::ConfigPaths::discover()
        .and_then(|p| gitpoulpi_config::Config::load(&p.config_file()).ok())
        .unwrap_or_default()
        .ui;
    (ui.launch_terminal, ui.logo_color)
}

fn state_name(state: State) -> &'static str {
    match state {
        State::NotInstalled => "not_installed",
        State::Installed => "installed",
        State::Outdated => "outdated",
        State::Foreign => "foreign",
    }
}

/// The state of every shortcut this OS offers.
pub fn status() -> Report {
    let (terminal, logo) = settings();
    report(&Env::real(), &terminal, &logo)
}

fn report(env: &Env, terminal: &str, logo: &str) -> Report {
    let snap = launcher::snapshot(env, terminal, logo);
    Report {
        terminal: snap.resolved,
        terminals: snap.terminals.iter().map(|(id, _)| *id).collect(),
        shortcuts: snap
            .rows
            .iter()
            .map(|r| ShortcutState {
                place: r.place.as_str(),
                state: state_name(r.status.state),
                icon_stale: r.status.icon_stale,
            })
            .collect(),
    }
}

fn places(env: &Env, place: Option<&str>, default_all: bool) -> Result<Vec<Place>, String> {
    let offered = env.dirs.places(env.os);
    match place {
        None if default_all => Ok(offered),
        None => Ok(vec![Place::for_os(env.os)[0]]),
        Some(name) => {
            let mut place = Place::parse(name)
                .ok_or_else(|| format!("unknown place {name:?}: launcher, desktop or startmenu"))?; // i18n-english
                                                                                                    // Windows has no app launcher: its launcher is the Start menu.
            if place == Place::Launcher && env.os == Os::Windows {
                place = Place::StartMenu;
            }
            if offered.contains(&place) || place == Place::Desktop {
                Ok(vec![place])
            } else {
                Err(format!("{name} is not available on this system")) // i18n-english
            }
        }
    }
}

fn describe(done: &Done) -> Result<String, String> {
    match done {
        Done::Installed(p) => Ok(format!("installed {}", p.as_str())), // i18n-english
        Done::Removed(p) => Ok(format!("removed {}", p.as_str())),     // i18n-english
        Done::NothingToRemove(p) => Ok(format!("nothing to remove: {}", p.as_str())), // i18n-english
        Done::NoTerminal => Err("no terminal was found to start GitPoulpi in".into()), // i18n-english
        Done::Failed(e) => Err(e.clone()),
    }
}

/// Creates or updates the shortcut at `place` (default: the OS launcher).
pub fn install(place: Option<&str>) -> Result<Vec<String>, String> {
    let (terminal, logo) = settings();
    apply(&Env::real(), &terminal, &logo, place, false)
}

/// Removes the shortcuts GitPoulpi made at `place` (default: all of them).
pub fn remove(place: Option<&str>) -> Result<Vec<String>, String> {
    let (terminal, logo) = settings();
    apply(&Env::real(), &terminal, &logo, place, true)
}

fn apply(
    env: &Env,
    terminal: &str,
    logo: &str,
    place: Option<&str>,
    remove: bool,
) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for place in places(env, place, remove)? {
        let op = if remove {
            Op::Remove(place)
        } else {
            Op::Install(place)
        };
        out.push(describe(&launcher::run(env, terminal, logo, op))?);
    }
    Ok(out)
}

/// The OS this build runs on, for `status` output.
pub fn os_name() -> &'static str {
    match Os::current() {
        Os::Windows => "windows",
        Os::Macos => "macos",
        Os::Linux => "linux",
    }
}
