//! macOS: a minimal `~/Applications/<launcher name>.app` per installation
//! (`GitPoulpi.app` for the primary). Its launcher script asks the chosen
//! terminal to run the exe (Terminal.app and iTerm2 open a `.command` file;
//! WezTerm, kitty and Ghostty take the program through `open -na ... --args`),
//! following the documented bundle layout.
//!
//! Every bundle carries the owner tag in its `Info.plist`; the primary also
//! carries the legacy marker. An installation only rewrites, renames or
//! removes a bundle it owns ([`owner::decide`]), and finds it again by that
//! tag when a rename (the version in the name) moved it.

use std::path::PathBuf;

use super::owner::{self, Facts, Owner};
use super::{Dirs, InstallCtx, Place, Request, State, Status, MARKER, OWNER_TAG};

const PRIMARY_ID: &str = "io.github.poulpi_lab.gitpoulpi.launcher";

/// A string as one POSIX shell word.
pub fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// The bundle identifier of `ctx`: the primary keeps the original, every
/// other installation gets `.i<16hex>` appended.
fn bundle_id(ctx: &InstallCtx) -> String {
    if ctx.is_primary() {
        PRIMARY_ID.to_string()
    } else {
        format!("{PRIMARY_ID}.i{}", super::terminals::hex16(&ctx.id))
    }
}

fn info_plist(ctx: &InstallCtx) -> String {
    let name = xml_escape(&ctx.name);
    let id = bundle_id(ctx);
    let marker = if ctx.is_primary() {
        format!("\t<key>{MARKER}</key><true/>\n")
    } else {
        String::new()
    };
    let tag = if ctx.id.is_empty() {
        String::new()
    } else {
        format!("\t<key>{OWNER_TAG}</key><string>{}</string>\n", ctx.id)
    };
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n<dict>\n\
         \t<key>CFBundleName</key><string>{name}</string>\n\
         \t<key>CFBundleDisplayName</key><string>{name}</string>\n\
         \t<key>CFBundleIdentifier</key><string>{id}</string>\n\
         \t<key>CFBundleExecutable</key><string>GitPoulpi</string>\n\
         \t<key>CFBundleIconFile</key><string>GitPoulpi</string>\n\
         \t<key>CFBundlePackageType</key><string>APPL</string>\n\
         \t<key>CFBundleVersion</key><string>1</string>\n\
         \t<key>LSUIElement</key><true/>\n\
         {marker}{tag}</dict>\n</plist>\n"
    )
}

/// Who owns a bundle whose `Info.plist` reads `plist` ([`owner::decide`]).
fn owner_of(plist: &str, ctx: &InstallCtx) -> Owner {
    let key = format!("<key>{OWNER_TAG}</key><string>");
    let tag = plist
        .split_once(key.as_str())
        .and_then(|(_, rest)| rest.split_once("</string>"))
        .map(|(v, _)| v);
    owner::decide(
        &Facts {
            tag,
            legacy_marker: plist.contains(MARKER),
            names_my_exe: false,
        },
        ctx,
    )
}

fn is_mine(plist: &str, ctx: &InstallCtx) -> bool {
    matches!(owner_of(plist, ctx), Owner::Mine | Owner::Adopt)
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

/// Where `ctx`'s bundle belongs: `<launcher name>.app` (a name is a file
/// name here, so `/` and `:` become `-`).
fn bundle(dirs: &Dirs, ctx: &InstallCtx) -> PathBuf {
    let name: String = ctx
        .name
        .chars()
        .map(|c| if matches!(c, '/' | ':') { '-' } else { c })
        .collect();
    dirs.mac_apps.join(format!("{name}.app"))
}

fn plist_of(root: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(root.join("Contents/Info.plist")).ok()
}

/// Where `ctx`'s bundle is.
enum Found {
    /// Ours, at this path (not the wanted one when a rename is pending).
    Mine(PathBuf),
    /// Something that is not ours sits at the wanted path.
    Blocked(PathBuf),
    None,
}

fn locate(dirs: &Dirs, ctx: &InstallCtx) -> Found {
    let wanted = bundle(dirs, ctx);
    if let Some(plist) = plist_of(&wanted) {
        return if is_mine(&plist, ctx) {
            Found::Mine(wanted)
        } else {
            Found::Blocked(wanted)
        };
    }
    // A rename (the version in the name) left it under its old name.
    if let Ok(read) = std::fs::read_dir(&dirs.mac_apps) {
        let mut paths: Vec<PathBuf> = read.flatten().map(|e| e.path()).collect();
        paths.sort();
        for p in paths {
            let is_app = p.extension().is_some_and(|e| e == "app");
            if is_app && plist_of(&p).is_some_and(|t| is_mine(&t, ctx)) {
                return Found::Mine(p);
            }
        }
    }
    Found::None
}

pub(super) fn status(place: Place, dirs: &Dirs, req: &Request) -> Status {
    let none = Status {
        state: State::NotInstalled,
        icon_stale: false,
    };
    if place != Place::Launcher {
        return none;
    }
    let root = match locate(dirs, req.ctx) {
        Found::Mine(root) => root,
        Found::Blocked(_) => {
            return Status {
                state: State::Foreign,
                icon_stale: false,
            }
        }
        Found::None => return none,
    };
    let exe = req.exe.to_string_lossy();
    let current = root == bundle(dirs, req.ctx)
        && plist_of(&root).as_deref() == Some(info_plist(req.ctx).as_str())
        && launcher_script(req.terminal, &exe).is_ok_and(|s| {
            std::fs::read_to_string(root.join("Contents/MacOS/GitPoulpi")).ok() == Some(s)
        })
        && std::fs::read_to_string(root.join("Contents/Resources/launch.command")).ok()
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
    let root = match locate(dirs, req.ctx) {
        Found::Mine(root) => root,
        Found::Blocked(_) => return Ok(()),
        Found::None => bundle(dirs, req.ctx),
    };
    super::write_file(
        &root.join("Contents/Resources/GitPoulpi.icns"),
        &req.icons.icns(),
        false,
    )
}

/// Bumps the bundle's modification time so Finder and the Dock re-read the
/// icon. Best effort; only a bundle of `ctx`.
pub(super) fn touch_bundle(dirs: &Dirs, ctx: &InstallCtx) {
    if let Found::Mine(root) = locate(dirs, ctx) {
        if let Ok(dir) = std::fs::File::open(root) {
            let _ = dir.set_modified(std::time::SystemTime::now());
        }
    }
}

pub(super) fn install(place: Place, dirs: &Dirs, req: &Request) -> Result<PathBuf, String> {
    if place != Place::Launcher {
        return Err("macOS has one place: the Applications folder".into());
    }
    let wanted = bundle(dirs, req.ctx);
    let mut renamed = false;
    match locate(dirs, req.ctx) {
        Found::Blocked(root) => {
            return Err(format!(
                "{} is not ours; not overwriting it",
                root.display()
            ));
        }
        // Renamed (the version in the name changed): move it, then rewrite.
        Found::Mine(old) if old != wanted => {
            std::fs::rename(&old, &wanted).map_err(|e| e.to_string())?;
            renamed = true;
        }
        _ => {}
    }
    let root = wanted;
    let exe = req.exe.to_string_lossy();
    let script = launcher_script(req.terminal, &exe)?;
    super::write_file(
        &root.join("Contents/Info.plist"),
        info_plist(req.ctx).as_bytes(),
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
    if renamed {
        touch_bundle(dirs, req.ctx);
    }
    Ok(root)
}

pub(super) fn remove(place: Place, dirs: &Dirs, ctx: &InstallCtx) -> Result<bool, String> {
    if place != Place::Launcher {
        return Ok(false);
    }
    match locate(dirs, ctx) {
        Found::None => Ok(false),
        Found::Blocked(root) => Err(format!("{} is not ours; not removing it", root.display())),
        Found::Mine(root) => {
            std::fs::remove_dir_all(&root).map_err(|e| e.to_string())?;
            Ok(true)
        }
    }
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
        assert!(remove(Place::Launcher, &dirs, InstallCtx::legacy_ref()).unwrap());
        assert!(!root.exists());

        std::fs::create_dir_all(root.join("Contents")).unwrap();
        std::fs::write(root.join("Contents/Info.plist"), "<plist/>").unwrap();
        assert!(remove(Place::Launcher, &dirs, InstallCtx::legacy_ref()).is_err());
        assert!(root.exists());
    }

    use crate::desktop::KnownInstall;

    const A: &str = "aaaaaaaaaaaaaaaa";
    const B: &str = "bbbbbbbbbbbbbbbb";

    fn ctx_of(id: &str, primary: Option<&str>, name: &str) -> InstallCtx {
        InstallCtx {
            id: id.into(),
            short: format!("i{}", &id[..8]),
            name: name.into(),
            primary: primary.map(str::to_string),
            known: [A, B]
                .iter()
                .map(|i| KnownInstall {
                    id: (*i).into(),
                    exists: true,
                })
                .collect(),
        }
    }

    fn req_ctx<'a>(
        exe: &'a Path,
        term: &'a str,
        icons: &'a IconSet,
        ctx: &'a InstallCtx,
    ) -> Request<'a> {
        Request {
            ctx,
            ..req(exe, term, icons)
        }
    }

    #[test]
    fn each_installation_gets_its_own_bundle_and_id() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = Dirs {
            mac_apps: tmp.path().join("Applications"),
            ..Dirs::default()
        };
        let icons = IconSet {
            pngs: vec![(256, vec![7])],
        };
        let exe = PathBuf::from("/opt/gp");
        let a = ctx_of(A, Some(A), "GitPoulpi");
        let b = ctx_of(B, Some(A), "GitPoulpi (dev)");
        let ra = install(Place::Launcher, &dirs, &req_ctx(&exe, "kitty", &icons, &a)).unwrap();
        let rb = install(Place::Launcher, &dirs, &req_ctx(&exe, "kitty", &icons, &b)).unwrap();
        assert_eq!(ra, dirs.mac_apps.join("GitPoulpi.app"));
        assert_eq!(rb, dirs.mac_apps.join("GitPoulpi (dev).app"));

        let pa = std::fs::read_to_string(ra.join("Contents/Info.plist")).unwrap();
        let pb = std::fs::read_to_string(rb.join("Contents/Info.plist")).unwrap();
        assert!(pa.contains("<string>io.github.poulpi_lab.gitpoulpi.launcher</string>"));
        assert!(pb.contains(&format!(
            "<string>io.github.poulpi_lab.gitpoulpi.launcher.i{B}</string>"
        )));
        assert!(pb.contains("<string>GitPoulpi (dev)</string>"));
        // The tag in both; the legacy marker only for the primary.
        for (plist, id) in [(&pa, A), (&pb, B)] {
            assert!(plist.contains(&format!("<key>{OWNER_TAG}</key><string>{id}</string>")));
        }
        assert!(pa.contains(MARKER) && !pb.contains(MARKER));
        for c in [&a, &b] {
            assert_eq!(
                status(Place::Launcher, &dirs, &req_ctx(&exe, "kitty", &icons, c)).state,
                State::Installed
            );
        }

        // The version in the name moves the bundle and touches nothing else.
        let b2 = ctx_of(B, Some(A), "GitPoulpi 1.2 (dev)");
        let r2 = req_ctx(&exe, "kitty", &icons, &b2);
        assert_eq!(status(Place::Launcher, &dirs, &r2).state, State::Outdated);
        let moved = install(Place::Launcher, &dirs, &r2).unwrap();
        assert_eq!(moved, dirs.mac_apps.join("GitPoulpi 1.2 (dev).app"));
        assert!(!rb.exists() && ra.exists());
        assert_eq!(status(Place::Launcher, &dirs, &r2).state, State::Installed);
    }

    #[test]
    fn remove_deletes_only_my_bundle() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = Dirs {
            mac_apps: tmp.path().join("Applications"),
            ..Dirs::default()
        };
        let icons = IconSet::default();
        let exe = PathBuf::from("/opt/gp");
        let a = ctx_of(A, Some(A), "GitPoulpi");
        let b = ctx_of(B, Some(A), "GitPoulpi (dev)");
        let ra = install(Place::Launcher, &dirs, &req_ctx(&exe, "kitty", &icons, &a)).unwrap();
        let rb = install(Place::Launcher, &dirs, &req_ctx(&exe, "kitty", &icons, &b)).unwrap();
        assert!(remove(Place::Launcher, &dirs, &b).unwrap());
        assert!(!rb.exists() && ra.exists());
        assert!(!remove(Place::Launcher, &dirs, &b).unwrap());

        // B may not touch a bundle A owns, even at B's own name.
        let a_at_b = dirs.mac_apps.join("GitPoulpi (dev).app");
        std::fs::rename(&ra, &a_at_b).unwrap();
        assert!(remove(Place::Launcher, &dirs, &b).is_err());
        let rb_req = req_ctx(&exe, "kitty", &icons, &b);
        assert_eq!(
            status(Place::Launcher, &dirs, &rb_req).state,
            State::Foreign
        );
        assert!(install(Place::Launcher, &dirs, &rb_req).is_err());
        assert!(a_at_b.exists());
        // A finds its bundle again by the tag.
        assert!(remove(Place::Launcher, &dirs, &a).unwrap());
        assert!(!a_at_b.exists());
    }
}
