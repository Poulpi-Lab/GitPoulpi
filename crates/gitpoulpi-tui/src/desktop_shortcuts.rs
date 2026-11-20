//! The scriptable twin of Settings > Launcher, for
//! `gitpoulpi desktop shortcut status|install|remove`. Same code as the
//! screen (`launcher`), run on the calling thread: a CLI has no frame to keep.

use gitpoulpi_config::desktop::{folder_menu, terminals, Place, State};
use gitpoulpi_config::open_in::Os;

use crate::launcher::{self, Done, Env, Op};

/// One shortcut's state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutState {
    /// `launcher`, `desktop` or `startmenu`.
    pub place: &'static str,
    /// `not_installed`, `installed`, `outdated` or `foreign`.
    pub state: &'static str,
    /// The icon file is not the one this installation's color makes.
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

/// The terminal setting and the colour this installation wears (its entry,
/// else `ui.logo_color` when it is the primary; never another installation's).
pub(crate) fn settings() -> (String, String) {
    let config = gitpoulpi_config::ConfigPaths::discover()
        .and_then(|p| gitpoulpi_config::Config::load(&p.config_file()).ok())
        .unwrap_or_default();
    let logo = crate::launcher_sync::effective_color(&config);
    (config.ui.launch_terminal, logo)
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

/// The terminal command that opens `folder` (or only GitPoulpi, with `None`),
/// built when a file-manager entry is clicked from `ui.launch_terminal`, so
/// changing the terminal never leaves an entry outdated. `gitpoulpi launch
/// --folder` and `gitpoulpi-launch` run it when no open window takes the
/// folder as a tab. Blocking (lists `PATH`).
pub fn start_command(folder: Option<&std::path::Path>) -> Result<Vec<String>, String> {
    let (setting, _) = settings();
    start_command_with(&Env::real(), &setting, folder)
}

/// The program the terminal runs: `gitpoulpi` itself, which with no
/// subcommand starts the TUI (on Windows `gitpoulpi.exe` beside the
/// console-less `gitpoulpi-launch.exe` that is running).
fn terminal_program(env: &Env) -> std::path::PathBuf {
    if env.os == Os::Windows {
        env.exe.with_file_name("gitpoulpi.exe")
    } else {
        env.exe.clone()
    }
}

fn start_command_with(
    env: &Env,
    setting: &str,
    folder: Option<&std::path::Path>,
) -> Result<Vec<String>, String> {
    const NO_TERMINAL: &str = "no terminal to start GitPoulpi in"; // i18n-english
    let terminal = env.resolve(setting).ok_or(NO_TERMINAL)?;
    let exe = terminal_program(env).to_string_lossy().into_owned();
    let mut argv = match folder {
        Some(f) => terminals::folder_argv(env.os, terminal, &exe, &f.to_string_lossy()),
        None if env.os == Os::Windows && terminal == "console" => {
            Some(vec![exe.clone(), terminals::LAUNCHER_FLAG.to_string()])
        }
        None => terminals::argv(env.os, terminal, &exe),
    }
    .ok_or(NO_TERMINAL)?;
    if env.os == Os::Windows && terminal != "console" {
        if let Some(found) = launcher::terminal_exe(env, terminal) {
            argv[0] = found.to_string_lossy().into_owned();
        }
    }
    Ok(argv)
}

/// One file manager's state, for `gitpoulpi desktop folder-menu status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderMenuEntry {
    pub manager: &'static str,
    pub name: &'static str,
    pub detected: bool,
    /// `not_installed`, `installed`, `outdated` or `foreign`.
    pub state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderMenuReport {
    pub entries: Vec<FolderMenuEntry>,
    /// The launcher program is missing next to GitPoulpi.
    pub unavailable: bool,
    pub error: Option<String>,
}

/// The label the CLI writes: English, like all CLI text, naming this
/// installation so that two installations never share an entry.
fn cli_label() -> String {
    format!("Open in {}", launcher::this_ctx().name) // i18n-english
}

fn cli_env(with_icon: bool) -> folder_menu::Env {
    let (_, logo) = settings();
    let mut env = launcher::folder_menu_env(&Env::real(), with_icon.then_some(logo.as_str()));
    // The submenu this installation asks for (its own setting, never another's).
    env.submenu = gitpoulpi_config::installation::current()
        .effective
        .folder_menu_submenu
        .clone();
    env
}

pub fn folder_menu_status() -> FolderMenuReport {
    let s = folder_menu::status(&cli_env(false), &cli_label());
    FolderMenuReport {
        entries: s
            .entries
            .iter()
            .map(|e| FolderMenuEntry {
                manager: e.manager.as_str(),
                name: e.manager.name(),
                detected: e.detected,
                state: state_name(e.state),
            })
            .collect(),
        unavailable: s.unavailable.is_some(),
        error: s.error,
    }
}

fn managers_lines(verb: &str, done: &[folder_menu::Manager]) -> Vec<String> {
    if done.is_empty() {
        vec![format!("nothing was {verb}")] // i18n-english
    } else {
        done.iter()
            .map(|m| format!("{verb}: {}", m.as_str())) // i18n-english
            .collect()
    }
}

pub fn folder_menu_install() -> Result<Vec<String>, String> {
    let done = folder_menu::install(&cli_env(true), &cli_label())?;
    Ok(managers_lines("installed", &done))
}

pub fn folder_menu_remove() -> Result<Vec<String>, String> {
    let done = folder_menu::remove(&cli_env(false))?;
    Ok(managers_lines("removed", &done))
}

/// The OS this build runs on, for `status` output.
pub fn os_name() -> &'static str {
    match Os::current() {
        Os::Windows => "windows",
        Os::Macos => "macos",
        Os::Linux => "linux",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_done_reads_as_a_sentence_for_every_verb() {
        for verb in ["installed", "removed"] {
            assert_eq!(
                managers_lines(verb, &[]),
                vec![format!("nothing was {verb}")]
            );
        }
    }
}
