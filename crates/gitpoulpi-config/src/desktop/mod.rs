//! Desktop integration: shortcuts that start GitPoulpi from the OS launcher
//! (Settings > General, `gitpoulpi desktop shortcut`).
//!
//! | OS | Launcher | Desktop |
//! |---|---|---|
//! | Linux | `~/.local/share/applications/gitpoulpi.desktop` + hicolor icon | `~/Desktop/gitpoulpi.desktop` |
//! | Windows | Start menu `GitPoulpi.lnk` | Desktop `GitPoulpi.lnk` |
//! | macOS | `~/Applications/GitPoulpi.app` | - |
//!
//! Every file we write carries the marker [`MARKER`]; Remove and Update only
//! touch files that do. Every directory is injected through [`Dirs`], so the
//! tests (and a hand run) never touch the real home. Nothing here may run on
//! the UI thread: Windows spawns PowerShell, every OS does file I/O.

pub mod alias;
pub mod folder_menu;
pub mod icons;
mod linux;
mod macos;
pub mod owner;
pub mod path;
pub mod registry;
pub mod terminals;
mod windows;

use std::path::{Path, PathBuf};

pub use icons::IconSet;
pub use linux::{entry_text, entry_text_with_action, exec_quote};
pub use macos::sh_quote;
pub use windows::quote_arg;

use crate::open_in::Os;

/// Written into everything we create; its presence is what allows removal.
pub const MARKER: &str = "X-GitPoulpi=1";

/// The owner tag: `X-GitPoulpi-Install=<id>` names the installation an
/// artifact belongs to ([`owner::decide`]). Only the primary also writes the
/// legacy [`MARKER`], the one old builds look for.
pub const OWNER_TAG: &str = "X-GitPoulpi-Install";

/// Another installation known from the config, and whether its exe is still there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownInstall {
    pub id: String,
    pub exists: bool,
}

/// Who is writing: the running installation, as the artifact writers and
/// [`owner::decide`] need it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallCtx {
    /// The installation id ([`crate::installation::Identity::id`]).
    pub id: String,
    /// `i<8hex>`, for file names, classes and GTK ids.
    pub short: String,
    /// The launcher name.
    pub name: String,
    /// The id of the primary installation, `None` while nobody is.
    pub primary: Option<String>,
    /// Every installation of the config.
    pub known: Vec<KnownInstall>,
}

impl InstallCtx {
    /// Today's behaviour: the running installation is the only one and owns
    /// the legacy names. What the writers still assume until later slices
    /// switch them over.
    pub fn sole(identity: &crate::installation::Identity) -> Self {
        Self {
            id: identity.id.clone(),
            short: identity.short(),
            name: crate::installation::PRIMARY_NAME.to_string(),
            primary: Some(identity.id.clone()),
            known: Vec::new(),
        }
    }

    /// [`InstallCtx::sole`] without an identity (tests, fakes).
    pub fn legacy() -> Self {
        Self::sole(&crate::installation::Identity {
            id: String::new(),
            kind: crate::Kind::default(),
            root: String::new(),
            channel: None,
        })
    }

    /// One shared [`InstallCtx::legacy`], for a `Request` that must borrow it.
    pub fn legacy_ref() -> &'static InstallCtx {
        static LEGACY: std::sync::OnceLock<InstallCtx> = std::sync::OnceLock::new();
        LEGACY.get_or_init(InstallCtx::legacy)
    }

    /// From the config: the effective name, the primary and every entry;
    /// `exists(path)` tells whether an installation's folder is still there.
    pub fn from_config(
        config: &crate::Config,
        identity: &crate::installation::Identity,
        exists: &dyn Fn(&str) -> bool,
    ) -> Self {
        let eff = crate::installation::effective(config, identity);
        Self {
            id: identity.id.clone(),
            short: identity.short(),
            name: eff.name,
            primary: config.installation.primary.clone(),
            known: config
                .installation
                .settings
                .iter()
                .map(|(id, e)| KnownInstall {
                    id: id.clone(),
                    exists: exists(&e.path),
                })
                .collect(),
        }
    }

    /// Is the running installation the primary?
    pub fn is_primary(&self) -> bool {
        self.primary.as_deref() == Some(self.id.as_str())
    }
}

/// Where a shortcut goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Place {
    /// The app launcher: the Linux `.desktop` entry, the macOS bundle.
    Launcher,
    /// The Desktop folder.
    Desktop,
    /// The Windows Start menu.
    StartMenu,
}

impl Place {
    pub fn as_str(self) -> &'static str {
        match self {
            Place::Launcher => "launcher",
            Place::Desktop => "desktop",
            Place::StartMenu => "startmenu",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "launcher" => Some(Place::Launcher),
            "desktop" => Some(Place::Desktop),
            "startmenu" => Some(Place::StartMenu),
            _ => None,
        }
    }

    /// The places `os` offers, in the order Settings lists them.
    pub fn for_os(os: Os) -> &'static [Place] {
        match os {
            Os::Linux => &[Place::Launcher, Place::Desktop],
            Os::Windows => &[Place::StartMenu, Place::Desktop],
            Os::Macos => &[Place::Launcher],
        }
    }
}

/// What is there now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    NotInstalled,
    /// Ours and matching the current exe and terminal.
    Installed,
    /// Ours, but the exe moved or another terminal is chosen: Update fixes it.
    Outdated,
    /// A file of that name exists and is not ours; it is left alone.
    Foreign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status {
    pub state: State,
    /// The icon file differs from the current one (the logo color changed).
    pub icon_stale: bool,
}

/// Every directory the module writes to.
#[derive(Debug, Clone, Default)]
pub struct Dirs {
    /// Linux: `<data>/applications`.
    pub applications: PathBuf,
    /// Linux: `<data>/icons/hicolor`.
    pub icon_root: PathBuf,
    /// The Desktop folder, when there is one.
    pub desktop: Option<PathBuf>,
    /// Windows: the user's Start menu `Programs` folder.
    pub start_menu: Option<PathBuf>,
    /// macOS: `~/Applications`.
    pub mac_apps: PathBuf,
    /// Windows: where the `.ico` the shortcuts point at is kept.
    pub assets: PathBuf,
    /// Windows: `User Pinned\TaskBar`, where Explorer keeps a copy of every
    /// shortcut pinned to the taskbar.
    pub taskbar_pinned: Option<PathBuf>,
    /// Windows: `User Pinned\ImplicitAppShortcuts`, the copies Explorer keeps
    /// (one folder deep) for pins it matched by AppUserModelID.
    pub implicit_pinned: Option<PathBuf>,
}

impl Dirs {
    /// The real directories for `os`, from the environment. Looks at the
    /// disk (is the Desktop folder there), so not on the UI thread.
    pub fn discover(os: Os, env: &dyn Fn(&str) -> Option<String>, home: &Path) -> Dirs {
        let var = |k: &str| env(k).filter(|v| !v.is_empty()).map(PathBuf::from);
        match os {
            Os::Linux => {
                let data = var("XDG_DATA_HOME").unwrap_or_else(|| home.join(".local/share"));
                let config = var("XDG_CONFIG_HOME").unwrap_or_else(|| home.join(".config"));
                Dirs {
                    applications: data.join("applications"),
                    icon_root: data.join("icons/hicolor"),
                    desktop: linux::desktop_dir(home, &config),
                    ..Dirs::default()
                }
            }
            Os::Windows => {
                let profile = var("USERPROFILE").unwrap_or_else(|| home.to_path_buf());
                let one_drive = profile.join("OneDrive").join("Desktop");
                let desktop = if one_drive.is_dir() {
                    one_drive
                } else {
                    profile.join("Desktop")
                };
                let appdata = var("APPDATA").unwrap_or_else(|| profile.join("AppData/Roaming"));
                Dirs {
                    desktop: Some(desktop),
                    start_menu: Some(appdata.join("Microsoft/Windows/Start Menu/Programs")),
                    taskbar_pinned: Some(
                        appdata
                            .join("Microsoft/Internet Explorer/Quick Launch/User Pinned/TaskBar"),
                    ),
                    implicit_pinned: Some(appdata.join(
                        "Microsoft/Internet Explorer/Quick Launch/User Pinned/ImplicitAppShortcuts",
                    )),
                    assets: var("LOCALAPPDATA")
                        .unwrap_or_else(|| profile.join("AppData/Local"))
                        .join("GitPoulpi"),
                    ..Dirs::default()
                }
            }
            Os::Macos => Dirs {
                mac_apps: home.join("Applications"),
                ..Dirs::default()
            },
        }
    }

    /// The places that make sense here (a Linux box without a Desktop
    /// folder has no Desktop row).
    pub fn places(&self, os: Os) -> Vec<Place> {
        Place::for_os(os)
            .iter()
            .copied()
            .filter(|p| *p != Place::Desktop || self.desktop.is_some())
            .collect()
    }
}

/// What a shortcut should start.
#[derive(Debug, Clone, Copy)]
pub struct Request<'a> {
    pub os: Os,
    /// The absolute path of GitPoulpi's executable.
    pub exe: &'a Path,
    /// A terminal id of [`terminals`], already resolved (not `auto`).
    pub terminal: &'a str,
    /// Windows: the terminal's absolute path, when found on `PATH`.
    pub terminal_exe: Option<&'a Path>,
    /// The program that runs `launch` ([`terminals::launch_helper`]): the
    /// shortcut starts it, and it shows the window of a GitPoulpi that is
    /// already open or else runs the terminal command. `None`: the shortcut
    /// runs the terminal command itself.
    pub launch_exe: Option<&'a Path>,
    /// Dev mode with `ui.launcher_build = "newest"`
    /// ([`crate::dev_mode::launcher_newest`]): a shortcut naming the `dist`
    /// or the `release` build of this checkout's exe is up to date (starting
    /// either build never rewrites it), and what [`install`] writes names the
    /// newest build. `launch` swaps in the newest build again at click time.
    pub newest: bool,
    pub icons: &'a IconSet,
    /// Who writes (names, tags, classes). Not read by the writers yet.
    pub ctx: &'a InstallCtx,
    /// Linux: run `gtk-update-icon-cache` after writing the icon. Off in tests.
    pub refresh_icon_cache: bool,
}

pub fn status(place: Place, dirs: &Dirs, req: &Request) -> Status {
    if req.newest && req.os != Os::Windows {
        // Linux / macOS compare whole files: accept the one the other build
        // would have written. (Windows searches the file for the paths.)
        let mut first = None;
        for (exe, launch) in variants(req) {
            let r = Request {
                exe: &exe,
                launch_exe: launch.as_deref(),
                ..*req
            };
            let st = status_of(place, dirs, &r);
            if st.state == State::Installed {
                return st;
            }
            first.get_or_insert(st);
        }
        return first.unwrap_or_else(|| status_of(place, dirs, req));
    }
    status_of(place, dirs, req)
}

fn status_of(place: Place, dirs: &Dirs, req: &Request) -> Status {
    match req.os {
        Os::Linux => linux::status(place, dirs, req),
        Os::Windows => windows::status(place, dirs, req),
        Os::Macos => macos::status(place, dirs, req),
    }
}

/// `(exe, launch_exe)` as each build of the checkout would write them.
fn variants(req: &Request) -> Vec<(PathBuf, Option<PathBuf>)> {
    let exes = crate::dev_mode::build_variants(req.exe);
    let launches = req.launch_exe.map(crate::dev_mode::build_variants);
    exes.into_iter()
        .enumerate()
        .map(|(i, exe)| {
            let launch = match (&launches, req.launch_exe) {
                (Some(l), _) if l.len() == 2 => Some(l[i.min(1)].clone()),
                (_, l) => l.map(Path::to_path_buf),
            };
            (exe, launch)
        })
        .collect()
}

/// The request with the newest builds of the exe and the launch helper.
fn newest_paths(req: &Request) -> (PathBuf, Option<PathBuf>) {
    (
        crate::dev_mode::newest_build(req.exe),
        req.launch_exe.map(crate::dev_mode::newest_build),
    )
}

/// Creates or rewrites the shortcut; returns what was written.
pub fn install(place: Place, dirs: &Dirs, req: &Request) -> Result<PathBuf, String> {
    if req.newest {
        let (exe, launch) = newest_paths(req);
        let r = Request {
            exe: &exe,
            launch_exe: launch.as_deref(),
            ..*req
        };
        return install_of(place, dirs, &r);
    }
    install_of(place, dirs, req)
}

fn install_of(place: Place, dirs: &Dirs, req: &Request) -> Result<PathBuf, String> {
    match req.os {
        Os::Linux => linux::install(place, dirs, req),
        Os::Windows => windows::install(place, dirs, req),
        Os::Macos => macos::install(place, dirs, req),
    }
}

/// Windows: rewrites the command line of our copies pinned to the taskbar
/// (they keep what the shortcut said when the user pinned it, possibly before
/// `--launcher` existed). `false` elsewhere or when nothing needed it.
pub fn sync_pinned(dirs: &Dirs, req: &Request) -> bool {
    if req.os != Os::Windows {
        return false;
    }
    if req.newest {
        let (exe, launch) = newest_paths(req);
        let r = Request {
            exe: &exe,
            launch_exe: launch.as_deref(),
            ..*req
        };
        return windows::sync_pinned_args(dirs, &r);
    }
    windows::sync_pinned_args(dirs, req)
}

/// Deletes the shortcut if it is ours. `Ok(false)`: there was none.
pub fn remove(place: Place, dirs: &Dirs, os: Os, ctx: &InstallCtx) -> Result<bool, String> {
    match os {
        Os::Linux => linux::remove(place, dirs, ctx),
        Os::Windows => windows::remove(place, dirs),
        Os::Macos => macos::remove(place, dirs, ctx),
    }
}

/// Rewrites only the icon file (the logo color changed): cheap, no spawn.
///
/// The OS shows the new color without touching the shortcut: Linux
/// (`gtk-update-icon-cache`, in `write_icon`), macOS (the bundle's mtime is
/// bumped so Finder re-reads the `.icns`) and Windows. There the taskbar
/// caches an icon by its path, so the `.ico` file name carries a hash of the
/// icon: the new file is written, every shortcut of ours (Start menu,
/// Desktop, the copies in `User Pinned\TaskBar`) is re-pointed at it and the
/// shell told, then older files nothing points at are deleted
/// (`windows::refresh_icon`).
pub fn refresh_icon(dirs: &Dirs, req: &Request) -> Result<(), String> {
    match req.os {
        Os::Linux => linux::write_icon(dirs, req),
        Os::Windows => windows::refresh_icon(dirs, req),
        Os::Macos => {
            macos::write_icon(dirs, req)?;
            macos::touch_bundle(dirs, req.ctx);
            Ok(())
        }
    }
}

/// Windows: the `.ico` for `icons` (its name carries a hash of the picture),
/// which the shortcuts point at and the running GitPoulpi also puts on its
/// terminal window.
pub fn ico_path(dirs: &Dirs, icons: &IconSet) -> PathBuf {
    windows::ico_path(dirs, icons)
}

/// Windows: makes sure the `.ico` for `icons` exists and returns its path
/// (`None` for no icons or when it cannot be written).
pub fn ensure_ico(dirs: &Dirs, icons: &IconSet) -> Option<PathBuf> {
    windows::ensure_ico(dirs, icons)
}

/// Writes `bytes`, creating parents; `executable` sets mode 755 on Unix.
fn write_file(path: &Path, bytes: &[u8], executable: bool) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    std::fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    #[cfg(unix)]
    if executable {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755));
    }
    #[cfg(not(unix))]
    let _ = executable;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_dirs_honour_xdg_data_home_and_need_a_real_desktop_folder() {
        let tmp = tempfile::tempdir().unwrap();
        let env =
            |k: &str| (k == "XDG_DATA_HOME").then(|| tmp.path().join("data").display().to_string());
        let d = Dirs::discover(Os::Linux, &env, tmp.path());
        assert_eq!(d.applications, tmp.path().join("data/applications"));
        assert_eq!(d.icon_root, tmp.path().join("data/icons/hicolor"));
        assert!(d.desktop.is_none());
        assert_eq!(d.places(Os::Linux), [Place::Launcher]);
        std::fs::create_dir(tmp.path().join("Desktop")).unwrap();
        let d = Dirs::discover(Os::Linux, &env, tmp.path());
        assert_eq!(d.places(Os::Linux), [Place::Launcher, Place::Desktop]);
    }

    #[test]
    fn windows_dirs_follow_appdata_and_the_profile() {
        let env = |k: &str| match k {
            "USERPROFILE" => Some("/u/me".to_string()),
            "APPDATA" => Some("/u/me/AppData/Roaming".to_string()),
            _ => None,
        };
        let d = Dirs::discover(Os::Windows, &env, Path::new("/h"));
        assert_eq!(
            d.start_menu.unwrap(),
            Path::new("/u/me/AppData/Roaming/Microsoft/Windows/Start Menu/Programs")
        );
        assert_eq!(d.desktop.unwrap(), Path::new("/u/me/Desktop"));
    }

    #[test]
    fn places_parse_and_print() {
        for p in [Place::Launcher, Place::Desktop, Place::StartMenu] {
            assert_eq!(Place::parse(p.as_str()), Some(p));
        }
        assert_eq!(Place::parse("x"), None);
    }

    #[test]
    fn newest_mode_keeps_a_linux_entry_installed_whichever_build_starts() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("co");
        std::fs::create_dir_all(root.join("crates/gitpoulpi-tui")).unwrap();
        std::fs::write(root.join("Cargo.toml"), "").unwrap();
        let mk = |profile: &str, name: &str, age: u64| {
            let dir = root.join("target").join(profile);
            std::fs::create_dir_all(&dir).unwrap();
            let p = dir.join(name);
            let f = std::fs::File::create(&p).unwrap();
            f.set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(age))
                .unwrap();
            p
        };
        let (dist, dist_cli) = (
            mk("dist", "gitpoulpi-tui", 100),
            mk("dist", "gitpoulpi", 100),
        );
        let (release, release_cli) = (
            mk("release", "gitpoulpi-tui", 1),
            mk("release", "gitpoulpi", 1),
        );
        let dirs = Dirs {
            applications: tmp.path().join("apps"),
            icon_root: tmp.path().join("icons"),
            ..Dirs::default()
        };
        let icons: &'static IconSet = Box::leak(Box::default());
        let req = |exe: &'static Path, launch: &'static Path, newest| Request {
            os: Os::Linux,
            exe,
            terminal: "kitty",
            terminal_exe: None,
            launch_exe: Some(launch),
            newest,
            icons,
            ctx: InstallCtx::legacy_ref(),
            refresh_icon_cache: false,
        };
        let leak = |p: &PathBuf| -> &'static Path { Box::leak(p.clone().into_boxed_path()) };
        let (dist, dist_cli) = (leak(&dist), leak(&dist_cli));
        let (release, release_cli) = (leak(&release), leak(&release_cli));
        // Written from the dist TUI in newest mode: names the newest (release).
        install(Place::Launcher, &dirs, &req(dist, dist_cli, true)).unwrap();
        let text = std::fs::read_to_string(dirs.applications.join("gitpoulpi.desktop")).unwrap();
        assert!(text.contains("release") && !text.contains("dist"), "{text}");
        for (exe, cli) in [(dist, dist_cli), (release, release_cli)] {
            assert_eq!(
                status(Place::Launcher, &dirs, &req(exe, cli, true)).state,
                State::Installed
            );
        }
        // Not in newest mode: the dist TUI sees an outdated entry, as before.
        assert_eq!(
            status(Place::Launcher, &dirs, &req(dist, dist_cli, false)).state,
            State::Outdated
        );
    }
}
