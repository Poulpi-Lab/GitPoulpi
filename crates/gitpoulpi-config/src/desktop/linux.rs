//! Linux: a freedesktop `.desktop` entry (app launcher, optional Desktop
//! copy) and the hicolor icon it names. On Omarchy the entry is exactly what
//! `omarchy-tui-install` writes (`xdg-terminal-exec --app-id=TUI.tile -e`).

use std::path::{Path, PathBuf};

use super::{terminals, Dirs, Place, Request, State, Status, MARKER};
use crate::open_in::Os;

pub(super) const FILE: &str = "gitpoulpi.desktop";
const ICON_STEM: &str = "gitpoulpi";

/// One `Exec=` argument: quoted when it holds a reserved character, with
/// the escapes the Desktop Entry spec asks for (and `\` doubled twice, once
/// for the quoting and once for the string-value escaping).
pub fn exec_quote(arg: &str) -> String {
    let arg = arg.replace('%', "%%");
    let reserved = |c: char| {
        c.is_whitespace()
            || matches!(
                c,
                '"' | '\''
                    | '\\'
                    | '>'
                    | '<'
                    | '~'
                    | '|'
                    | '&'
                    | ';'
                    | '$'
                    | '*'
                    | '?'
                    | '#'
                    | '('
                    | ')'
                    | '`'
            )
    };
    if arg.is_empty() || arg.chars().any(reserved) {
        let mut out = String::from("\"");
        for c in arg.chars() {
            match c {
                '"' | '`' | '$' => {
                    out.push('\\');
                    out.push(c);
                }
                '\\' => out.push_str("\\\\\\\\"),
                _ => out.push(c),
            }
        }
        out.push('"');
        out
    } else {
        arg
    }
}

/// The `.desktop` file text for a command line, an `Icon=` name and the
/// window class the terminal was started with (`StartupWMClass`, so a dock
/// groups the running window under this entry).
pub fn entry_text(argv: &[String], icon: &str, wm_class: Option<&str>) -> String {
    entry_text_with_action(argv, None, icon, wm_class)
}

/// [`entry_text`], plus a "New Window" desktop action running `new_window`
/// (the right-click menu of a dock or launcher entry).
pub fn entry_text_with_action(
    argv: &[String],
    new_window: Option<&[String]>,
    icon: &str,
    wm_class: Option<&str>,
) -> String {
    let join = |argv: &[String]| {
        argv.iter()
            .map(|a| exec_quote(a))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let exec = join(argv);
    let class = wm_class.map_or_else(String::new, |c| format!("StartupWMClass={c}\n"));
    let actions = new_window.map_or("", |_| "Actions=new-window;\n");
    let section = new_window.map_or_else(String::new, |argv| {
        format!(
            "\n[Desktop Action new-window]\nName=New Window\nExec={}\n",
            join(argv)
        )
    });
    format!(
        "[Desktop Entry]\nVersion=1.0\nType=Application\nName=GitPoulpi\n\
         Comment=Git client for the terminal\nExec={exec}\nTerminal=false\n\
         Icon={icon}\nCategories=Development;RevisionControl;\nStartupNotify=true\n{actions}{class}{MARKER}\n{section}"
    )
}

/// The `Exec=` command: the terminal's, run through `gitpoulpi launch` when
/// there is one (it shows a GitPoulpi window that is already open). The
/// second value is the same command with `--new-window`, for the desktop
/// action (only when there is a launch helper to run it).
fn argv(req: &Request) -> Result<(Vec<String>, Option<Vec<String>>), String> {
    let exe = req.exe.to_string_lossy();
    let inner = terminals::argv(Os::Linux, req.terminal, &exe)
        .ok_or_else(|| format!("no terminal to start GitPoulpi in ({})", req.terminal))?;
    Ok(match req.launch_exe {
        Some(launch) => {
            let launch = launch.to_string_lossy();
            (
                terminals::launch_argv(Os::Linux, &launch, &inner),
                Some(terminals::launch_argv_with(
                    Os::Linux,
                    &launch,
                    &inner,
                    true,
                )),
            )
        }
        None => (inner, None),
    })
}

fn entry(req: &Request, icon: &str) -> Result<String, String> {
    let (argv, new_window) = argv(req)?;
    Ok(entry_text_with_action(
        &argv,
        new_window.as_deref(),
        icon,
        terminals::wm_class(Os::Linux, req.terminal),
    ))
}

fn entry_path(place: Place, dirs: &Dirs) -> Result<PathBuf, String> {
    match place {
        Place::Launcher => Ok(dirs.applications.join(FILE)),
        Place::Desktop => dirs
            .desktop
            .as_ref()
            .map(|d| d.join(FILE))
            .ok_or_else(|| "there is no Desktop folder".to_string()),
        Place::StartMenu => Err("the Start menu is Windows only".into()),
    }
}

/// The `Icon=` name of a colorway: it carries a hash of the picture, so a
/// new color is a new name. Launchers cache an icon by its name or path
/// (omarchy-shell's Qt image cache never re-reads a file it has shown), so
/// rewriting the same file in place left the old color on screen.
fn icon_name(req: &Request) -> String {
    match req.icons.largest() {
        Some(png) => {
            // FNV-1a: stable across builds, unlike `DefaultHasher`.
            let hash = png.iter().fold(0x811c_9dc5_u32, |h, b| {
                (h ^ u32::from(*b)).wrapping_mul(0x0100_0193)
            });
            format!("{ICON_STEM}-{hash:08x}")
        }
        None => ICON_STEM.to_string(),
    }
}

fn icon_dir(dirs: &Dirs) -> PathBuf {
    dirs.icon_root.join("256x256/apps")
}

fn icon_path(dirs: &Dirs, name: &str) -> PathBuf {
    icon_dir(dirs).join(format!("{name}.png"))
}

/// Whether `file` is an icon we wrote: `gitpoulpi.png` (before the names
/// carried a hash) or `gitpoulpi-<8 hex>.png`.
fn is_our_icon(file: &str) -> bool {
    let Some(stem) = file.strip_suffix(".png") else {
        return false;
    };
    stem == ICON_STEM
        || stem
            .strip_prefix(ICON_STEM)
            .and_then(|r| r.strip_prefix('-'))
            .is_some_and(|h| h.len() == 8 && h.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// Deletes our icon files except `keep`.
fn remove_icons(dirs: &Dirs, keep: Option<&str>) {
    let Ok(read) = std::fs::read_dir(icon_dir(dirs)) else {
        return;
    };
    let keep = keep.map(|k| format!("{k}.png"));
    for e in read.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if is_our_icon(&name) && keep.as_deref() != Some(name.as_str()) {
            let _ = std::fs::remove_file(e.path());
        }
    }
}

/// The `Icon=` value of an entry.
fn entry_icon(text: &str) -> Option<&str> {
    text.lines().find_map(|l| l.strip_prefix("Icon="))
}

/// `text` with its `Icon=` line set to `icon` (added when it has none).
fn with_icon(text: &str, icon: &str) -> String {
    let mut out: String = text
        .lines()
        .map(|l| {
            if l.starts_with("Icon=") {
                format!("Icon={icon}\n")
            } else {
                format!("{l}\n")
            }
        })
        .collect();
    if entry_icon(text).is_none() {
        out.push_str(&format!("Icon={icon}\n"));
    }
    out
}

pub(super) fn status(place: Place, dirs: &Dirs, req: &Request) -> Status {
    let none = Status {
        state: State::NotInstalled,
        icon_stale: false,
    };
    let Ok(path) = entry_path(place, dirs) else {
        return none;
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return none;
    };
    if !text.lines().any(|l| l == MARKER) {
        return Status {
            state: State::Foreign,
            icon_stale: false,
        };
    }
    let name = icon_name(req);
    // The icon name is the icon's business: a new color is a stale icon,
    // not an outdated entry.
    let state = match entry(req, &name) {
        Ok(wanted) if wanted == with_icon(&text, &name) => State::Installed,
        _ => State::Outdated,
    };
    let icon_stale = req.icons.largest().is_some_and(|png| {
        entry_icon(&text) != Some(name.as_str())
            || std::fs::read(icon_path(dirs, &name)).ok().as_deref() != Some(png)
    });
    Status { state, icon_stale }
}

/// Writes the icon under its color's name, points our entries at it and
/// deletes the icons of the previous colors.
pub(super) fn write_icon(dirs: &Dirs, req: &Request) -> Result<(), String> {
    let Some(png) = req.icons.largest() else {
        return Ok(());
    };
    let name = icon_name(req);
    super::write_file(&icon_path(dirs, &name), png, false)?;
    for place in [Place::Launcher, Place::Desktop] {
        let Ok(path) = entry_path(place, dirs) else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if text.lines().any(|l| l == MARKER) && entry_icon(&text) != Some(name.as_str()) {
            // Rewriting the entry is also what makes a launcher that
            // watches the applications folder reload it.
            super::write_file(&path, with_icon(&text, &name).as_bytes(), true)?;
        }
    }
    remove_icons(dirs, Some(&name));
    if req.refresh_icon_cache {
        // Best effort, like omarchy-tui-install.
        let _ = std::process::Command::new("gtk-update-icon-cache")
            .arg(&dirs.icon_root)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
    Ok(())
}

pub(super) fn install(place: Place, dirs: &Dirs, req: &Request) -> Result<PathBuf, String> {
    let path = entry_path(place, dirs)?;
    if status(place, dirs, req).state == State::Foreign {
        return Err(format!(
            "{} is not ours; not overwriting it",
            path.display()
        ));
    }
    let text = entry(req, &icon_name(req))?;
    super::write_file(&path, text.as_bytes(), true)?;
    write_icon(dirs, req)?;
    Ok(path)
}

pub(super) fn remove(place: Place, dirs: &Dirs) -> Result<bool, String> {
    let path = entry_path(place, dirs)?;
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(false);
    };
    if !text.lines().any(|l| l == MARKER) {
        return Err(format!("{} is not ours; not removing it", path.display()));
    }
    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    // The icon goes with the last entry that names it.
    let other = match place {
        Place::Launcher => dirs.desktop.as_ref().map(|d| d.join(FILE)),
        _ => Some(dirs.applications.join(FILE)),
    };
    if !other.is_some_and(|p| p.exists()) {
        remove_icons(dirs, None);
    }
    Ok(true)
}

/// The XDG Desktop folder, if it exists (`user-dirs.dirs`, else `~/Desktop`).
pub(super) fn desktop_dir(home: &Path, config_home: &Path) -> Option<PathBuf> {
    let from_file = std::fs::read_to_string(config_home.join("user-dirs.dirs"))
        .ok()
        .and_then(|text| {
            text.lines().find_map(|l| {
                let v = l.trim().strip_prefix("XDG_DESKTOP_DIR=")?;
                let v = v.trim().trim_matches('"');
                Some(match v.strip_prefix("$HOME") {
                    Some(rest) => home.join(rest.trim_start_matches('/')),
                    None => PathBuf::from(v),
                })
            })
        });
    let dir = from_file.unwrap_or_else(|| home.join("Desktop"));
    dir.is_dir().then_some(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::IconSet;

    fn req<'a>(exe: &'a Path, term: &'a str, icons: &'a IconSet) -> Request<'a> {
        Request {
            os: Os::Linux,
            exe,
            terminal: term,
            terminal_exe: None,
            launch_exe: None,
            icons,
            refresh_icon_cache: false,
            newest: false,
        }
    }

    fn dirs(root: &Path) -> Dirs {
        Dirs {
            applications: root.join("applications"),
            icon_root: root.join("icons/hicolor"),
            desktop: Some(root.join("Desktop")),
            ..Dirs::default()
        }
    }

    fn lines(text: &str) -> Vec<&str> {
        text.lines().collect()
    }

    #[test]
    fn the_omarchy_entry_matches_what_omarchy_tui_install_writes() {
        let text = entry_text(
            &terminals::argv(Os::Linux, "xdg-terminal-exec", "/usr/bin/gitpoulpi").unwrap(),
            "gitpoulpi",
            terminals::wm_class(Os::Linux, "xdg-terminal-exec"),
        );
        let l = lines(&text);
        assert!(l.contains(
            &"Exec=xdg-terminal-exec --app-id=TUI.tile -e /usr/bin/gitpoulpi --launcher"
        ));
        for want in [
            "Terminal=false",
            "Type=Application",
            "Icon=gitpoulpi",
            "StartupNotify=true",
            "StartupWMClass=TUI.tile",
            MARKER,
        ] {
            assert!(l.contains(&want), "{want}");
        }
    }

    #[test]
    fn exec_lines_per_terminal() {
        let exe = "/opt/gp/gitpoulpi";
        let exec = |id: &str| {
            entry_text(
                &terminals::argv(Os::Linux, id, exe).unwrap(),
                "gitpoulpi",
                None,
            )
            .lines()
            .find_map(|l| l.strip_prefix("Exec=").map(str::to_string))
            .unwrap()
        };
        assert_eq!(
            exec("kitty"),
            "kitty --class gitpoulpi /opt/gp/gitpoulpi --launcher"
        );
        assert_eq!(
            exec("wezterm"),
            "wezterm start --class gitpoulpi -- /opt/gp/gitpoulpi --launcher"
        );
        assert_eq!(
            exec("alacritty"),
            "alacritty --class gitpoulpi -e /opt/gp/gitpoulpi --launcher"
        );
        assert_eq!(
            exec("ghostty"),
            "ghostty --class=io.github.poulpi_lab.gitpoulpi -e /opt/gp/gitpoulpi --launcher"
        );
        assert_eq!(
            exec("foot"),
            "foot --app-id=gitpoulpi /opt/gp/gitpoulpi --launcher"
        );
        assert_eq!(exec("konsole"), "konsole -e /opt/gp/gitpoulpi --launcher");
    }

    #[test]
    fn the_entry_names_the_class_the_terminal_is_started_with() {
        let exe = PathBuf::from("/opt/gp/gitpoulpi");
        let icons = IconSet::default();
        for (term, class) in [
            ("kitty", Some("gitpoulpi")),
            ("wezterm", Some("gitpoulpi")),
            ("foot", Some("gitpoulpi")),
            ("alacritty", Some("gitpoulpi")),
            ("ghostty", Some("io.github.poulpi_lab.gitpoulpi")),
            ("konsole", None),
        ] {
            let text = entry(&req(&exe, term, &icons), "gitpoulpi").unwrap();
            let found = text.lines().find_map(|l| l.strip_prefix("StartupWMClass="));
            assert_eq!(found, class, "{term}");
        }
    }

    #[test]
    fn through_launch_the_exec_is_gitpoulpi_launch_then_the_terminal_command() {
        let exe = PathBuf::from("/opt/gp/gitpoulpi");
        let icons = IconSet::default();
        let launch = PathBuf::from("/opt/gp/gitpoulpi");
        let mut r = req(&exe, "kitty", &icons);
        let plain = entry(&r, "gitpoulpi").unwrap();
        r.launch_exe = Some(&launch);
        let text = entry(&r, "gitpoulpi").unwrap();
        assert!(
            text.contains(
                "Exec=/opt/gp/gitpoulpi launch -- kitty --class gitpoulpi /opt/gp/gitpoulpi --launcher\n"
            ),
            "{text}"
        );
        // Right-click > New Window: the same command, never reusing a window.
        assert!(text.contains("\nActions=new-window;\n"), "{text}");
        assert!(
            text.contains(
                "\n[Desktop Action new-window]\nName=New Window\nExec=/opt/gp/gitpoulpi launch --new-window -- kitty --class gitpoulpi /opt/gp/gitpoulpi --launcher\n"
            ),
            "{text}"
        );
        // The marker stays in the main section; the plain form has no action.
        let main = text.split("\n[Desktop Action").next().unwrap();
        assert!(main.lines().any(|l| l == MARKER));
        assert!(!plain.contains("Desktop Action"), "{plain}");
        // The old form reads as outdated, so the sync rewrites it.
        let tmp = tempfile::tempdir().unwrap();
        let d = dirs(tmp.path());
        std::fs::create_dir_all(&d.applications).unwrap();
        std::fs::write(d.applications.join(FILE), &plain).unwrap();
        assert_eq!(status(Place::Launcher, &d, &r).state, State::Outdated);
        install(Place::Launcher, &d, &r).unwrap();
        assert_eq!(status(Place::Launcher, &d, &r).state, State::Installed);
    }

    #[test]
    fn a_path_with_spaces_or_specials_is_quoted_and_escaped() {
        assert_eq!(exec_quote("/a b/gp"), "\"/a b/gp\"");
        assert_eq!(exec_quote("/plain/gp"), "/plain/gp");
        assert_eq!(exec_quote("/a\"b$c"), "\"/a\\\"b\\$c\"");
        assert_eq!(exec_quote("/100%/x"), "/100%%/x");
        assert_eq!(exec_quote("/a\\b"), "\"/a\\\\\\\\b\"");
        let line = entry_text(
            &["kitty".into(), "/My Apps/gitpoulpi".into()],
            "gitpoulpi",
            None,
        );
        assert!(line.contains("Exec=kitty \"/My Apps/gitpoulpi\"\n"));
    }

    #[test]
    fn install_writes_entry_and_icon_then_remove_deletes_only_ours() {
        let tmp = tempfile::tempdir().unwrap();
        let d = dirs(tmp.path());
        let icons = IconSet {
            pngs: vec![(256, vec![9, 9, 9])],
        };
        let exe = PathBuf::from("/opt/gp/gitpoulpi");
        let r = req(&exe, "kitty", &icons);

        assert_eq!(status(Place::Launcher, &d, &r).state, State::NotInstalled);
        let path = install(Place::Launcher, &d, &r).unwrap();
        assert_eq!(path, d.applications.join(FILE));
        let st = status(Place::Launcher, &d, &r);
        assert_eq!((st.state, st.icon_stale), (State::Installed, false));
        assert_eq!(
            std::fs::read(icon_path(&d, &icon_name(&r))).unwrap(),
            [9, 9, 9]
        );

        // Another terminal: the entry is outdated until Update.
        let r2 = req(&exe, "alacritty", &icons);
        assert_eq!(status(Place::Launcher, &d, &r2).state, State::Outdated);
        // A new icon color is a stale icon, not an outdated entry.
        let icons2 = IconSet {
            pngs: vec![(256, vec![1])],
        };
        let st = status(Place::Launcher, &d, &req(&exe, "kitty", &icons2));
        assert_eq!((st.state, st.icon_stale), (State::Installed, true));

        assert!(remove(Place::Launcher, &d).unwrap());
        assert!(!path.exists() && !icon_path(&d, &icon_name(&r)).exists());
        assert!(!remove(Place::Launcher, &d).unwrap());
    }

    #[test]
    fn a_file_without_our_marker_is_never_overwritten_or_removed() {
        let tmp = tempfile::tempdir().unwrap();
        let d = dirs(tmp.path());
        std::fs::create_dir_all(&d.applications).unwrap();
        let path = d.applications.join(FILE);
        std::fs::write(&path, "[Desktop Entry]\nName=mine\n").unwrap();
        let icons = IconSet::default();
        let exe = PathBuf::from("/x/gp");
        let r = req(&exe, "kitty", &icons);
        assert_eq!(status(Place::Launcher, &d, &r).state, State::Foreign);
        assert!(install(Place::Launcher, &d, &r).is_err());
        assert!(remove(Place::Launcher, &d).is_err());
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "[Desktop Entry]\nName=mine\n"
        );
    }

    #[test]
    fn the_icon_outlives_the_first_of_two_entries_only() {
        let tmp = tempfile::tempdir().unwrap();
        let d = dirs(tmp.path());
        std::fs::create_dir_all(d.desktop.as_ref().unwrap()).unwrap();
        let icons = IconSet {
            pngs: vec![(256, vec![5])],
        };
        let exe = PathBuf::from("/x/gp");
        let r = req(&exe, "kitty", &icons);
        install(Place::Launcher, &d, &r).unwrap();
        install(Place::Desktop, &d, &r).unwrap();
        remove(Place::Launcher, &d).unwrap();
        assert!(icon_path(&d, &icon_name(&r)).exists());
        remove(Place::Desktop, &d).unwrap();
        assert!(!icon_path(&d, &icon_name(&r)).exists());
    }

    #[test]
    fn a_new_color_is_a_new_icon_name_in_every_entry_and_the_old_icon_goes() {
        let tmp = tempfile::tempdir().unwrap();
        let d = dirs(tmp.path());
        std::fs::create_dir_all(d.desktop.as_ref().unwrap()).unwrap();
        let exe = PathBuf::from("/x/gp");
        let (old, new) = (
            IconSet {
                pngs: vec![(256, vec![1, 2])],
            },
            IconSet {
                pngs: vec![(256, vec![3, 4])],
            },
        );
        let r = req(&exe, "kitty", &old);
        install(Place::Launcher, &d, &r).unwrap();
        install(Place::Desktop, &d, &r).unwrap();
        // An icon a user put there by hand under another name stays.
        std::fs::write(icon_dir(&d).join("gitpoulpi-mine.png"), [7]).unwrap();
        let (old_name, r2) = (icon_name(&r), req(&exe, "kitty", &new));
        let new_name = icon_name(&r2);
        assert_ne!(old_name, new_name);
        assert!(status(Place::Launcher, &d, &r2).icon_stale);

        write_icon(&d, &r2).unwrap();
        for place in [Place::Launcher, Place::Desktop] {
            let text = std::fs::read_to_string(entry_path(place, &d).unwrap()).unwrap();
            assert_eq!(entry_icon(&text), Some(new_name.as_str()), "{place:?}");
            let st = status(place, &d, &r2);
            assert_eq!((st.state, st.icon_stale), (State::Installed, false));
        }
        assert_eq!(std::fs::read(icon_path(&d, &new_name)).unwrap(), [3, 4]);
        assert!(!icon_path(&d, &old_name).exists());
        assert!(icon_dir(&d).join("gitpoulpi-mine.png").exists());
    }

    #[test]
    fn desktop_dir_reads_user_dirs_and_needs_the_folder_to_exist() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path().join("home");
        let cfg = tmp.path().join("cfg");
        std::fs::create_dir_all(home.join("Bureau")).unwrap();
        std::fs::create_dir_all(&cfg).unwrap();
        std::fs::write(
            cfg.join("user-dirs.dirs"),
            "XDG_DESKTOP_DIR=\"$HOME/Bureau\"\n",
        )
        .unwrap();
        assert_eq!(desktop_dir(&home, &cfg), Some(home.join("Bureau")));
        assert_eq!(desktop_dir(&home, &tmp.path().join("none")), None);
    }
}
