//! macOS: a minimal `~/Applications/GitPoulpi.app`. Its launcher script asks
//! the chosen terminal to run the exe (Terminal.app and iTerm2 open a
//! `.command` file; WezTerm, kitty and Ghostty take the program through
//! `open -na ... --args`), following the documented bundle layout.

use std::path::PathBuf;

use super::{Dirs, Place, Request, State, Status, MARKER};

const BUNDLE: &str = "GitPoulpi.app";

/// A string as one POSIX shell word.
pub fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn info_plist() -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n<dict>\n\
         \t<key>CFBundleName</key><string>GitPoulpi</string>\n\
         \t<key>CFBundleDisplayName</key><string>GitPoulpi</string>\n\
         \t<key>CFBundleIdentifier</key><string>io.github.poulpi_lab.gitpoulpi.launcher</string>\n\
         \t<key>CFBundleExecutable</key><string>GitPoulpi</string>\n\
         \t<key>CFBundleIconFile</key><string>GitPoulpi</string>\n\
         \t<key>CFBundlePackageType</key><string>APPL</string>\n\
         \t<key>CFBundleVersion</key><string>1</string>\n\
         \t<key>LSUIElement</key><true/>\n\
         \t<key>{MARKER}</key><true/>\n\
         </dict>\n</plist>\n"
    )
}

/// The bundle's `Contents/MacOS/GitPoulpi`.
pub(super) fn launcher_script(terminal: &str, exe: &str) -> Result<String, String> {
    let exe = format!("{} {}", sh_quote(exe), super::terminals::LAUNCHER_FLAG);
    let body = match terminal {
        "apple-terminal" => {
            "exec /usr/bin/open -a Terminal \"$HERE/../Resources/launch.command\"".to_string()
        }
        "iterm" => "exec /usr/bin/open -a iTerm \"$HERE/../Resources/launch.command\"".to_string(),
        "wezterm" => format!("exec /usr/bin/open -na WezTerm --args start -- {exe}"),
        "kitty" => format!("exec /usr/bin/open -na kitty --args {exe}"),
        "ghostty" => format!("exec /usr/bin/open -na Ghostty --args -e {exe}"),
        other => return Err(format!("unknown terminal {other}")),
    };
    Ok(format!("#!/bin/sh\nHERE=\"$(dirname \"$0\")\"\n{body}\n"))
}

fn command_file(exe: &str) -> String {
    format!(
        "#!/bin/sh\nexec {} {}\n",
        sh_quote(exe),
        super::terminals::LAUNCHER_FLAG
    )
}

fn bundle(dirs: &Dirs) -> PathBuf {
    dirs.mac_apps.join(BUNDLE)
}

pub(super) fn status(place: Place, dirs: &Dirs, req: &Request) -> Status {
    let none = Status {
        state: State::NotInstalled,
        icon_stale: false,
    };
    if place != Place::Launcher {
        return none;
    }
    let root = bundle(dirs);
    let Ok(plist) = std::fs::read_to_string(root.join("Contents/Info.plist")) else {
        return none;
    };
    if !plist.contains(MARKER) {
        return Status {
            state: State::Foreign,
            icon_stale: false,
        };
    }
    let exe = req.exe.to_string_lossy();
    let current = launcher_script(req.terminal, &exe).is_ok_and(|s| {
        std::fs::read_to_string(root.join("Contents/MacOS/GitPoulpi")).ok() == Some(s)
    }) && std::fs::read_to_string(root.join("Contents/Resources/launch.command"))
        .ok()
        == Some(command_file(&exe));
    let icon_stale = !req.icons.pngs.is_empty()
        && std::fs::read(root.join("Contents/Resources/GitPoulpi.icns"))
            .ok()
            .as_deref()
            != Some(req.icons.icns().as_slice());
    Status {
        state: if current {
            State::Installed
        } else {
            State::Outdated
        },
        icon_stale,
    }
}

pub(super) fn write_icon(dirs: &Dirs, req: &Request) -> Result<(), String> {
    if req.icons.pngs.is_empty() {
        return Ok(());
    }
    let path = bundle(dirs).join("Contents/Resources/GitPoulpi.icns");
    super::write_file(&path, &req.icons.icns(), false)
}

/// Bumps the bundle's modification time so Finder and the Dock re-read the
/// icon. Best effort.
pub(super) fn touch_bundle(dirs: &Dirs) {
    if let Ok(dir) = std::fs::File::open(bundle(dirs)) {
        let _ = dir.set_modified(std::time::SystemTime::now());
    }
}

pub(super) fn install(place: Place, dirs: &Dirs, req: &Request) -> Result<PathBuf, String> {
    if place != Place::Launcher {
        return Err("macOS has one place: the Applications folder".into());
    }
    let root = bundle(dirs);
    if status(place, dirs, req).state == State::Foreign {
        return Err(format!(
            "{} is not ours; not overwriting it",
            root.display()
        ));
    }
    let exe = req.exe.to_string_lossy();
    let script = launcher_script(req.terminal, &exe)?;
    super::write_file(
        &root.join("Contents/Info.plist"),
        info_plist().as_bytes(),
        false,
    )?;
    super::write_file(
        &root.join("Contents/MacOS/GitPoulpi"),
        script.as_bytes(),
        true,
    )?;
    super::write_file(
        &root.join("Contents/Resources/launch.command"),
        command_file(&exe).as_bytes(),
        true,
    )?;
    write_icon(dirs, req)?;
    Ok(root)
}

pub(super) fn remove(place: Place, dirs: &Dirs) -> Result<bool, String> {
    if place != Place::Launcher {
        return Ok(false);
    }
    let root = bundle(dirs);
    let Ok(plist) = std::fs::read_to_string(root.join("Contents/Info.plist")) else {
        return Ok(false);
    };
    if !plist.contains(MARKER) {
        return Err(format!("{} is not ours; not removing it", root.display()));
    }
    std::fs::remove_dir_all(&root).map_err(|e| e.to_string())?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::IconSet;
    use crate::open_in::Os;
    use std::path::Path;

    fn req<'a>(exe: &'a Path, term: &'a str, icons: &'a IconSet) -> Request<'a> {
        Request {
            os: Os::Macos,
            exe,
            terminal: term,
            terminal_exe: None,
            launch_exe: None,
            icons,
            ctx: crate::desktop::InstallCtx::legacy_ref(),
            refresh_icon_cache: false,
            newest: false,
        }
    }

    #[test]
    fn launcher_scripts_per_terminal_quote_the_exe() {
        let exe = "/Users/me/It's here/gitpoulpi";
        assert!(launcher_script("kitty", exe)
            .unwrap()
            .contains("open -na kitty --args '/Users/me/It'\\''s here/gitpoulpi'"));
        assert!(launcher_script("wezterm", exe)
            .unwrap()
            .contains("--args start -- '"));
        assert!(launcher_script("apple-terminal", exe)
            .unwrap()
            .contains("-a Terminal"));
        assert!(launcher_script("nope", exe).is_err());
    }

    #[test]
    fn install_builds_the_bundle_and_remove_deletes_it_only_when_ours() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = Dirs {
            mac_apps: tmp.path().join("Applications"),
            ..Dirs::default()
        };
        let icons = IconSet {
            pngs: vec![(256, vec![7])],
        };
        let exe = PathBuf::from("/opt/gp");
        let r = req(&exe, "apple-terminal", &icons);
        let root = install(Place::Launcher, &dirs, &r).unwrap();
        for f in [
            "Contents/Info.plist",
            "Contents/MacOS/GitPoulpi",
            "Contents/Resources/GitPoulpi.icns",
            "Contents/Resources/launch.command",
        ] {
            assert!(root.join(f).is_file(), "{f}");
        }
        assert_eq!(status(Place::Launcher, &dirs, &r).state, State::Installed);
        assert_eq!(
            status(Place::Launcher, &dirs, &req(&exe, "iterm", &icons)).state,
            State::Outdated
        );
        assert!(remove(Place::Launcher, &dirs).unwrap());
        assert!(!root.exists());

        std::fs::create_dir_all(root.join("Contents")).unwrap();
        std::fs::write(root.join("Contents/Info.plist"), "<plist/>").unwrap();
        assert!(remove(Place::Launcher, &dirs).is_err());
        assert!(root.exists());
    }
}
