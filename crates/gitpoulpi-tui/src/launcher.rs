//! The TUI side of desktop shortcuts (`gitpoulpi_config::desktop`): the
//! machine (directories, installed terminals, our exe), the icon in the
//! user's logo color, and the blocking jobs Settings runs on a thread.
//!
//! Nothing here may run on the input thread: [`Env::real`] lists `PATH` and
//! looks at the disk, [`run`] writes files (and spawns PowerShell on Windows).
//! Every directory comes through [`Env`], so tests use temp dirs.

use std::path::PathBuf;
use std::sync::Arc;

use gitpoulpi_config::desktop::{
    self, terminals, Dirs, IconSet, InstallCtx, Place, Request, Status,
};
use gitpoulpi_config::open_in::Os;

type Which = Arc<dyn Fn(&str) -> Option<PathBuf> + Send + Sync>;

/// What shortcuts are made from.
#[derive(Clone)]
pub(crate) struct Env {
    pub os: Os,
    pub dirs: Dirs,
    pub exe: PathBuf,
    /// The program shortcuts run `launch` through (`terminals::launch_helper`);
    /// `None`: they run the terminal command themselves.
    pub launch_exe: Option<PathBuf>,
    /// Dev mode with `ui.launcher_build = "newest"`
    /// (`gitpoulpi_config::dev_mode::launcher_newest`): shortcuts follow the
    /// most recently built `dist` / `release` exe and never read `Outdated`
    /// because the other profile started.
    pub newest: bool,
    /// Is this program (or `app:Name` bundle) installed?
    pub has: Arc<dyn Fn(&str) -> bool + Send + Sync>,
    /// The absolute path of a program on `PATH` (Windows shortcuts need it).
    pub which: Which,
    /// Run `gtk-update-icon-cache` after writing the icon (never in tests).
    pub refresh_icon_cache: bool,
    /// Who writes. Today every writer still acts as the only installation
    /// (the primary); later slices switch on the per-installation names.
    pub ctx: InstallCtx,
}

impl Env {
    /// This machine. Blocking: lists every `PATH` directory once.
    pub(crate) fn real() -> Env {
        let os = Os::current();
        let home = gitpoulpi_config::home_dir().unwrap_or_default();
        let dirs = Dirs::discover(os, &|k| std::env::var(k).ok(), &home);
        let on_path = crate::repo_menu::path_lookup();
        let has = move |probe: &str| match probe.strip_prefix("app:") {
            Some(app) => mac_app_exists(app, &home),
            None => on_path(probe) || windows_alias_exists(probe),
        };
        let which = |program: &str| {
            if let Some(alias) = windows_alias(program).filter(|p| p.is_file()) {
                return Some(alias);
            }
            crate::repo_menu::resolve_program(
                program,
                std::env::var_os("PATH").as_deref(),
                std::env::var("PATHEXT").ok().as_deref(),
                cfg!(windows),
                std::path::Path::new("."),
            )
        };
        // The one `gitpoulpi` executable, also when a `gitpoulpi-tui` stub (or
        // the cargo test bin) is what runs: shortcuts never name the stub.
        let exe =
            gitpoulpi_config::desktop::alias::tui_exe(&std::env::current_exe().unwrap_or_default());
        let newest = gitpoulpi_config::dev_mode::launcher_newest_on_disk(&exe);
        let real_fs = gitpoulpi_config::installation::RealFs;
        let identity = gitpoulpi_config::installation::identify(
            &exe,
            &gitpoulpi_config::installation::IdentityEnv::real(&real_fs),
        );
        Env {
            ctx: InstallCtx::sole(&identity),
            os,
            dirs,
            launch_exe: terminals::launch_helper(os, &exe, &|p| p.is_file()),
            exe,
            newest,
            has: Arc::new(has),
            which: Arc::new(which),
            // Tests and scripted runs set this to keep helper programs out of it.
            refresh_icon_cache: std::env::var_os("GITPOULPI_DESKTOP_NO_HOOKS").is_none(), // i18n-english
        }
    }

    /// The terminals installed here, as `(id, label)`.
    pub(crate) fn terminals(&self) -> Vec<(&'static str, &'static str)> {
        terminals::installed(self.os, &*self.has)
            .iter()
            .map(|t| (t.id, t.label))
            .collect()
    }

    /// The id a `ui.launch_terminal` value means here.
    pub(crate) fn resolve(&self, setting: &str) -> Option<&'static str> {
        terminals::resolve(setting, self.os, &*self.has)
    }
}

fn mac_app_exists(app: &str, home: &std::path::Path) -> bool {
    [
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications/Utilities"),
        home.join("Applications"), // i18n-english
    ]
    .iter()
    .any(|d| d.join(format!("{app}.app")).is_dir())
}

/// Windows Terminal is an "app execution alias" in `WindowsApps`, which the
/// `PATH` listing may not see as a file.
fn windows_alias(program: &str) -> Option<PathBuf> {
    if !cfg!(windows) || program != "wt" {
        return None;
    }
    let local = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(local).join("Microsoft/WindowsApps/wt.exe"))
}

fn windows_alias_exists(program: &str) -> bool {
    windows_alias(program).is_some_and(|p| p.exists())
}

/// A shortcut row as Settings shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Row {
    pub place: Place,
    pub status: Status,
}

/// What a background job learned about the machine.
#[derive(Clone)]
pub(crate) struct Snapshot {
    pub terminals: Vec<(&'static str, &'static str)>,
    /// The terminal `auto` / the setting resolves to now.
    pub resolved: Option<&'static str>,
    pub rows: Vec<Row>,
}

/// One action on one shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Op {
    Install(Place),
    Remove(Place),
}

/// What an [`Op`] came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Done {
    Installed(Place),
    Removed(Place),
    NothingToRemove(Place),
    NoTerminal,
    Failed(String),
}

/// The pixel octopus in colorway `slug` as the icon set shortcuts use.
pub(crate) fn icon_set(slug: &str) -> IconSet {
    let way = &crate::logo_colors::ways()[crate::logo_colors::index_of(slug)];
    let sprite = crate::logo_colors::bitmap(way, 1.0, false, 0.0, (0, 0));
    let mut pngs = Vec::new();
    for side in [16u32, 32, 48, 256] {
        let mut scaled = crate::terminal_image::Bitmap::transparent(side, side);
        for y in 0..side {
            for x in 0..side {
                let (sx, sy) = (x * sprite.width / side, y * sprite.height / side);
                if let Some((r, g, b, a)) = sprite.rgba(sx, sy) {
                    if a > 0 {
                        scaled.set_pixel(x, y, (r, g, b));
                    }
                }
            }
        }
        if let Some(png) = crate::terminal_image::png_bytes(&scaled) {
            pngs.push((side, png));
        }
    }
    IconSet { pngs }
}

/// Reads the state of every shortcut here; icons that are stale (the logo
/// color changed) are rewritten on the way, which is cheap and spawns nothing.
pub(crate) fn snapshot(env: &Env, setting: &str, logo: &str) -> Snapshot {
    let terminals = env.terminals();
    let resolved = env.resolve(setting);
    let icons = icon_set(logo);
    let mut rows = Vec::new();
    for place in env.dirs.places(env.os) {
        let status = match resolved {
            Some(terminal) => {
                let terminal_exe = terminal_exe(env, terminal);
                let req = request(env, terminal, terminal_exe.as_deref(), &icons);
                let mut status = desktop::status(place, &env.dirs, &req);
                if status.icon_stale
                    && status.state != desktop::State::Foreign
                    && desktop::refresh_icon(&env.dirs, &req).is_ok()
                {
                    status.icon_stale = false;
                }
                status
            }
            None => Status {
                state: desktop::State::NotInstalled,
                icon_stale: false,
            },
        };
        rows.push(Row { place, status });
    }
    Snapshot {
        terminals,
        resolved,
        rows,
    }
}

/// Rewrites the icon of every shortcut we made whose icon is not the one
/// colorway `logo` makes; true when at least one was rewritten. Nothing is
/// written when no shortcut is installed or the file there is not ours
/// (`Status` reads `NotInstalled` / `Foreign` with `icon_stale` false).
/// The shortcut itself is untouched: only the icon file changes, so the
/// `.desktop` `Icon=` name and the `.lnk` target stay valid. Blocking.
pub(crate) fn refresh_icons(env: &Env, setting: &str, logo: &str) -> bool {
    // The terminal only matters to `state`; the icon check does not read it.
    let terminal = env.resolve(setting).unwrap_or("");
    let terminal_exe = (!terminal.is_empty())
        .then(|| terminal_exe(env, terminal))
        .flatten();
    let icons = icon_set(logo);
    let req = request(env, terminal, terminal_exe.as_deref(), &icons);
    let stale = env.dirs.places(env.os).into_iter().any(|place| {
        let status = desktop::status(place, &env.dirs, &req);
        status.icon_stale && status.state != desktop::State::Foreign
    });
    stale && desktop::refresh_icon(&env.dirs, &req).is_ok()
}

/// Brings every shortcut we made up to date with the settings: one whose
/// state is `Outdated` (another terminal chosen, the exe moved) is rewritten
/// in place, our copies pinned to the taskbar get the current command line
/// (`--launcher` included), then stale icons are refreshed. True when anything was written.
/// A `NotInstalled` or `Foreign` shortcut is never touched: this never
/// creates. Blocking.
pub(crate) fn sync_shortcuts(env: &Env, setting: &str, logo: &str) -> bool {
    let mut changed = false;
    if let Some(terminal) = env.resolve(setting) {
        let terminal_exe = terminal_exe(env, terminal);
        let icons = icon_set(logo);
        let req = request(env, terminal, terminal_exe.as_deref(), &icons);
        for place in env.dirs.places(env.os) {
            if desktop::status(place, &env.dirs, &req).state == desktop::State::Outdated
                && desktop::install(place, &env.dirs, &req).is_ok()
            {
                changed = true;
            }
        }
        // Windows: a copy pinned to the taskbar keeps the command line it
        // was pinned with, whatever state the Start menu entry is in.
        changed |= desktop::sync_pinned(&env.dirs, &req);
    }
    // The file-manager entries that are ours follow the exe too (and the
    // icon); a missing one is never created.
    let label = gitpoulpi_i18n::msg::launcher::FOLDER_MENU_LABEL.get();
    changed |= desktop::folder_menu::sync(&folder_menu_env(env, Some(logo)), label)
        .is_ok_and(|done| !done.is_empty());
    refresh_icons(env, setting, logo) || changed
}

pub(crate) fn terminal_exe(env: &Env, terminal: &str) -> Option<PathBuf> {
    let program = terminals::find(env.os, terminal)?.probe;
    (!program.is_empty() && !program.starts_with("app:"))
        .then(|| (env.which)(program))
        .flatten()
}

fn request<'a>(
    env: &'a Env,
    terminal: &'a str,
    terminal_exe: Option<&'a std::path::Path>,
    icons: &'a IconSet,
) -> Request<'a> {
    Request {
        os: env.os,
        exe: &env.exe,
        terminal,
        terminal_exe,
        launch_exe: env.launch_exe.as_deref(),
        newest: env.newest,
        icons,
        ctx: &env.ctx,
        refresh_icon_cache: env.refresh_icon_cache,
    }
}

/// The folder-menu machine (`gitpoulpi_config::desktop::folder_menu`) for
/// `env`: the same exe, `PATH` probe and hook switch, the real registry (or the
/// test file), and the icon in colorway `logo` when one is given (Install
/// needs it, a status read does not). Blocking: it reads the disk.
pub(crate) fn folder_menu_env(env: &Env, logo: Option<&str>) -> desktop::folder_menu::Env {
    let home = gitpoulpi_config::home_dir().unwrap_or_default();
    let mut fe = desktop::folder_menu::Env::discover(
        env.os,
        &|k| std::env::var(k).ok(),
        &home,
        &env.exe,
        env.has.clone(),
        desktop::registry::from_env(),
    );
    fe.icon_png = logo.and_then(|l| icon_set(l).largest().map(<[u8]>::to_vec));
    fe.refresh_hooks = env.refresh_icon_cache;
    fe.ctx = env.ctx.clone();
    fe
}

/// Runs `op`. Blocking.
pub(crate) fn run(env: &Env, setting: &str, logo: &str, op: Op) -> Done {
    match op {
        Op::Remove(place) => match desktop::remove(place, &env.dirs, env.os, &env.ctx) {
            Ok(true) => Done::Removed(place),
            Ok(false) => Done::NothingToRemove(place),
            Err(e) => Done::Failed(e),
        },
        Op::Install(place) => {
            let Some(terminal) = env.resolve(setting) else {
                return Done::NoTerminal;
            };
            let icons = icon_set(logo);
            let terminal_exe = terminal_exe(env, terminal);
            let req = request(env, terminal, terminal_exe.as_deref(), &icons);
            match desktop::install(place, &env.dirs, &req) {
                Ok(_) => Done::Installed(place),
                Err(e) => Done::Failed(e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_config::desktop::State;

    fn env(root: &std::path::Path, has: &'static [&'static str]) -> Env {
        std::fs::create_dir_all(root.join("Desktop")).unwrap();
        Env {
            os: Os::Linux,
            dirs: Dirs {
                applications: root.join("data/applications"),
                icon_root: root.join("data/icons/hicolor"),
                desktop: Some(root.join("Desktop")),
                ..Dirs::default()
            },
            exe: PathBuf::from("/opt/gp/gitpoulpi"),
            launch_exe: None,
            newest: false,
            has: Arc::new(move |p| has.contains(&p)),
            which: Arc::new(|_| None),
            refresh_icon_cache: false,
            ctx: InstallCtx::legacy(),
        }
    }

    fn pixel(png: &[u8], x: usize, y: usize) -> [u8; 4] {
        let mut reader = png::Decoder::new(std::io::Cursor::new(png))
            .read_info()
            .unwrap();
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buf).unwrap();
        let i = (y * info.width as usize + x) * 4;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    #[test]
    fn install_remove_round_trip_in_a_temp_dir_names_the_chosen_terminal() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), &["kitty", "foot"]);
        assert_eq!(
            run(&e, "auto", "", Op::Install(Place::Launcher)),
            Done::Installed(Place::Launcher)
        );
        let text = std::fs::read_to_string(e.dirs.applications.join("gitpoulpi.desktop")).unwrap();
        assert!(
            text.contains("Exec=kitty --class gitpoulpi /opt/gp/gitpoulpi --launcher\n"),
            "{text}"
        );
        let snap = snapshot(&e, "auto", "");
        assert_eq!(snap.resolved, Some("kitty"));
        assert_eq!(snap.rows[0].status.state, State::Installed);
        assert_eq!(snap.rows[1].status.state, State::NotInstalled);
        // Another terminal in the setting: outdated until the sync rewrites it.
        assert_eq!(
            snapshot(&e, "foot", "").rows[0].status.state,
            State::Outdated
        );
        assert_eq!(
            run(&e, "auto", "", Op::Remove(Place::Launcher)),
            Done::Removed(Place::Launcher)
        );
        assert_eq!(
            run(&e, "auto", "", Op::Remove(Place::Launcher)),
            Done::NothingToRemove(Place::Launcher)
        );
    }

    #[test]
    fn no_terminal_installed_is_reported_not_written() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), &[]);
        assert_eq!(
            run(&e, "auto", "", Op::Install(Place::Launcher)),
            Done::NoTerminal
        );
        assert!(!e.dirs.applications.exists());
    }

    #[test]
    fn the_icon_is_the_octopus_in_the_chosen_colorway() {
        let ways = crate::logo_colors::ways();
        if ways.len() < 2 {
            return;
        }
        let (a, b) = (&ways[0].slug, &ways[1].slug);
        let (ia, ib) = (icon_set(a), icon_set(b));
        let big = |s: &IconSet| s.pngs.iter().find(|(n, _)| *n == 256).unwrap().1.clone();
        assert_ne!(big(&ia), big(&ib));
        // Every opaque pixel of the 256 px icon is a pixel of that colorway's sprite.
        let sprite = crate::logo_colors::bitmap(&ways[0], 1.0, false, 0.0, (0, 0));
        for (x, y) in [(128usize, 100usize), (128, 140), (90, 120), (60, 200)] {
            let want = sprite.rgba((x / 2) as u32, (y / 2) as u32).unwrap();
            let got = pixel(&big(&ia), x, y);
            assert_eq!(got, [want.0, want.1, want.2, want.3], "({x},{y})");
        }
    }

    /// The icon file our launcher entry names (its name changes with the
    /// color so launchers that cache icons see the new one).
    fn named_icon(e: &Env) -> std::path::PathBuf {
        let text = std::fs::read_to_string(e.dirs.applications.join("gitpoulpi.desktop")).unwrap();
        let name = text.lines().find_map(|l| l.strip_prefix("Icon=")).unwrap();
        e.dirs.icon_root.join(format!("256x256/apps/{name}.png"))
    }

    #[test]
    fn refresh_icons_rewrites_only_icons_of_shortcuts_we_made() {
        let ways = crate::logo_colors::ways();
        if ways.len() < 2 {
            return;
        }
        let (a, b) = (&ways[0].slug, &ways[1].slug);
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), &["kitty"]);
        let apps = e.dirs.icon_root.join("256x256/apps");
        // No shortcut: nothing is written, not even the icon.
        assert!(!refresh_icons(&e, "auto", b));
        assert!(!apps.exists());
        // One of ours: the pixels become the new colorway's.
        run(&e, "auto", a, Op::Install(Place::Launcher));
        let old = named_icon(&e);
        assert!(!refresh_icons(&e, "auto", a), "same color: nothing to do");
        assert!(refresh_icons(&e, "auto", b));
        let icon = named_icon(&e);
        assert_ne!(icon, old, "a new color is a new icon name");
        assert!(!old.exists());
        let want = icon_set(b);
        assert_eq!(std::fs::read(&icon).unwrap(), want.largest().unwrap());
        assert_eq!(
            pixel(&std::fs::read(&icon).unwrap(), 128, 140),
            pixel(want.largest().unwrap(), 128, 140)
        );
        // A .desktop that is not ours: left alone.
        std::fs::write(
            e.dirs.applications.join("gitpoulpi.desktop"),
            "[Desktop Entry]\n",
        )
        .unwrap();
        assert!(!refresh_icons(&e, "auto", a));
        assert_eq!(std::fs::read(&icon).unwrap(), want.largest().unwrap());
    }

    #[test]
    fn a_changed_logo_color_refreshes_the_icon_of_an_existing_shortcut() {
        let ways = crate::logo_colors::ways();
        if ways.len() < 2 {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), &["kitty"]);
        run(&e, "auto", &ways[0].slug, Op::Install(Place::Launcher));
        let before = std::fs::read(named_icon(&e)).unwrap();
        snapshot(&e, "auto", &ways[1].slug);
        let icon = named_icon(&e);
        assert_ne!(std::fs::read(&icon).unwrap(), before);
        assert_eq!(
            std::fs::read(&icon).unwrap(),
            icon_set(&ways[1].slug).largest().unwrap()
        );
    }
}

#[cfg(test)]
mod sync_tests {
    use super::*;
    use gitpoulpi_config::desktop::State;

    fn env(root: &std::path::Path, has: &'static [&'static str]) -> Env {
        std::fs::create_dir_all(root.join("Desktop")).unwrap();
        Env {
            os: Os::Linux,
            dirs: Dirs {
                applications: root.join("data/applications"),
                icon_root: root.join("data/icons/hicolor"),
                desktop: Some(root.join("Desktop")),
                ..Dirs::default()
            },
            exe: PathBuf::from("/opt/gp/gitpoulpi"),
            launch_exe: None,
            newest: false,
            has: Arc::new(move |p| has.contains(&p)),
            which: Arc::new(|_| None),
            refresh_icon_cache: false,
            ctx: InstallCtx::legacy(),
        }
    }

    fn entry(e: &Env) -> std::path::PathBuf {
        e.dirs.applications.join("gitpoulpi.desktop")
    }

    #[test]
    fn a_changed_terminal_rewrites_our_shortcut_in_place() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), &["kitty", "foot"]);
        run(&e, "kitty", "", Op::Install(Place::Launcher));
        assert!(!sync_shortcuts(&e, "kitty", ""), "nothing to do");
        assert!(sync_shortcuts(&e, "foot", ""));
        let text = std::fs::read_to_string(entry(&e)).unwrap();
        assert!(
            text.contains("Exec=foot --app-id=gitpoulpi /opt/gp/gitpoulpi --launcher\n"),
            "{text}"
        );
        assert!(text.contains("X-GitPoulpi=1"), "{text}");
        assert_eq!(
            snapshot(&e, "foot", "").rows[0].status.state,
            State::Installed
        );
        // The Desktop copy was never made, so it is still not created.
        assert!(!tmp.path().join("Desktop/gitpoulpi.desktop").exists());
    }

    #[test]
    fn a_file_that_is_not_ours_is_left_alone() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), &["kitty", "foot"]);
        std::fs::create_dir_all(&e.dirs.applications).unwrap();
        std::fs::write(entry(&e), "[Desktop Entry]\nExec=mine\n").unwrap();
        assert!(!sync_shortcuts(&e, "foot", ""));
        assert_eq!(
            std::fs::read_to_string(entry(&e)).unwrap(),
            "[Desktop Entry]\nExec=mine\n"
        );
    }

    #[test]
    fn nothing_is_created_when_there_is_no_shortcut() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), &["kitty"]);
        assert!(!sync_shortcuts(&e, "kitty", ""));
        assert!(!e.dirs.applications.exists());
        assert!(!tmp.path().join("Desktop/gitpoulpi.desktop").exists());
    }

    /// Two TUIs of one checkout (dist, release) must not rewrite the shortcut
    /// in turn: in newest mode neither does, otherwise each one does.
    #[test]
    fn newest_mode_stops_the_dist_and_release_tuis_rewriting_each_others_shortcuts() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("co");
        std::fs::create_dir_all(root.join("crates/gitpoulpi-tui")).unwrap();
        std::fs::write(root.join("Cargo.toml"), "").unwrap();
        let mk = |profile: &str| {
            let dir = root.join("target").join(profile);
            std::fs::create_dir_all(&dir).unwrap();
            let p = dir.join("gitpoulpi-tui");
            std::fs::write(&p, "").unwrap();
            p
        };
        let (dist, release) = (mk("dist"), mk("release"));
        let machine = tmp.path().join("m");
        let from = |exe: &std::path::Path, newest: bool| Env {
            exe: exe.to_path_buf(),
            newest,
            ..env(&machine, &["kitty"])
        };
        let entry = machine.join("data/applications/gitpoulpi.desktop");
        // Without newest mode: the shortcut follows whichever TUI started last.
        assert_eq!(
            run(
                &from(&dist, false),
                "kitty",
                "",
                Op::Install(Place::Launcher)
            ),
            Done::Installed(Place::Launcher)
        );
        assert!(sync_shortcuts(&from(&release, false), "kitty", ""));
        assert!(sync_shortcuts(&from(&dist, false), "kitty", ""));
        // Newest mode: the entry (naming dist) is up to date for both.
        let before = std::fs::read_to_string(&entry).unwrap();
        assert!(!sync_shortcuts(&from(&release, true), "kitty", ""));
        assert!(!sync_shortcuts(&from(&dist, true), "kitty", ""));
        assert_eq!(std::fs::read_to_string(&entry).unwrap(), before);
        assert_eq!(
            snapshot(&from(&release, true), "kitty", "").rows[0]
                .status
                .state,
            State::Installed
        );
    }
}
