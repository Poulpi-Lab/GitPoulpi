//! Linux file managers. Each has its own mechanism; the text builders are
//! pure and the rest is plain file work under [`Env::data_home`] and
//! [`Env::config_home`].
//!
//! - GNOME Files and Caja: an executable script in their `scripts` folder
//!   (shown under Scripts). The file name is the label, which follows the UI
//!   language, so Remove finds ours by the marker on the second line.
//! - Dolphin: a KDE service menu (must be executable, KF6 ignores it else).
//! - Nemo: two `.nemo_action` files (selection and background).
//! - PCManFM: a file-manager action.
//! - Thunar: a block merged into `uca.xml` ([`super::thunar`]).
//!
//! KDE Frameworks before 5.85 (`kservices5/ServiceMenus`) are not supported.

use std::path::{Path, PathBuf};

use super::{is_ours, thunar, Env, Manager, ICON_NAME};
use crate::desktop::{exec_quote, sh_quote, write_file, State, MARKER};

const ICON_REL: &str = "icons/hicolor/256x256/apps/gitpoulpi-folder.png";

/// The script GNOME Files and Caja run (given the selected folder, or none on
/// the background).
pub fn script_text(helper: &Path) -> String {
    format!(
        "#!/bin/sh\n# {MARKER}\ndir=\"${{1:-.}}\"\n[ -d \"$dir\" ] || dir=.\ncd -- \"$dir\" && exec {} launch --folder \"$PWD\"\n",
        sh_quote(&helper.to_string_lossy())
    )
}

pub fn dolphin_text(helper: &Path, label: &str) -> String {
    format!(
        "[Desktop Entry]\nType=Service\nMimeType=inode/directory;\nActions=openInGitPoulpi;\nX-KDE-Priority=TopLevel\n{MARKER}\n\n[Desktop Action openInGitPoulpi]\nName={label}\nIcon={ICON_NAME}\nExec={} launch --folder %f\n",
        exec_quote(&helper.to_string_lossy())
    )
}

/// `(selection, background)` Nemo actions.
pub fn nemo_texts(helper: &Path, label: &str) -> (String, String) {
    let h = exec_quote(&helper.to_string_lossy());
    let make = |sel: &str, ext: &str, arg: &str| {
        format!(
            "[Nemo Action]\nName={label}\nComment={label}\nExec={h} launch --folder {arg}\nIcon-Name={ICON_NAME}\nSelection={sel}\nExtensions={ext}\n{MARKER}\n"
        )
    };
    (make("s", "dir;", "%F"), make("None", "any;", "%P"))
}

pub fn fm_actions_text(helper: &Path, label: &str) -> String {
    format!(
        "[Desktop Entry]\nType=Action\nName={label}\nIcon={ICON_NAME}\nProfiles=folder;\n{MARKER}\n\n[X-Action-Profile folder]\nMimeTypes=inode/directory;\nExec={} launch --folder %f\n",
        exec_quote(&helper.to_string_lossy())
    )
}

fn script_dir(env: &Env, m: Manager) -> PathBuf {
    match m {
        Manager::Caja => env.config_home.join("caja/scripts"),
        _ => env.data_home.join("nautilus/scripts"),
    }
}

fn is_script_manager(m: Manager) -> bool {
    matches!(m, Manager::Nautilus | Manager::Caja)
}

/// A file name for `label`.
fn file_label(label: &str) -> String {
    label.replace(['/', '\0'], "-")
}

/// `(directory, [(file name, text, must be executable)])` for the managers
/// that keep fixed-name files.
fn fixed(
    env: &Env,
    helper: &Path,
    label: &str,
    m: Manager,
) -> (PathBuf, Vec<(String, String, bool)>) {
    match m {
        Manager::Dolphin => (
            env.data_home.join("kio/servicemenus"),
            vec![(
                "gitpoulpi-open.desktop".into(),
                dolphin_text(helper, label),
                true,
            )],
        ),
        Manager::Nemo => {
            let (sel, here) = nemo_texts(helper, label);
            (
                env.data_home.join("nemo/actions"),
                vec![
                    ("gitpoulpi-open.nemo_action".into(), sel, false),
                    ("gitpoulpi-open-here.nemo_action".into(), here, false),
                ],
            )
        }
        _ => (
            env.data_home.join("file-manager/actions"),
            vec![(
                "gitpoulpi-open.desktop".into(),
                fm_actions_text(helper, label),
                false,
            )],
        ),
    }
}

fn thunar_file(env: &Env) -> PathBuf {
    env.config_home.join("Thunar/uca.xml")
}

#[cfg(unix)]
fn executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn executable(_: &Path) -> bool {
    true
}

/// The scripts in `dir` that are ours.
fn our_scripts(dir: &Path) -> Vec<PathBuf> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = read
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            std::fs::read_to_string(p)
                .ok()
                .and_then(|t| t.lines().nth(1).map(|l| l == format!("# {MARKER}")))
                .unwrap_or(false)
        })
        .collect();
    out.sort();
    out
}

fn first_name(text: &str) -> Option<String> {
    text.lines()
        .find_map(|l| l.strip_prefix("Name=").map(str::to_string))
}

pub(super) fn state(env: &Env, label: &str, m: Manager) -> State {
    let helper = env.helper.as_deref().unwrap_or(Path::new(""));
    let has_helper = env.helper.is_some();
    if is_script_manager(m) {
        let dir = script_dir(env, m);
        let ours = our_scripts(&dir);
        if ours.is_empty() {
            return if dir.join(file_label(label)).exists() {
                State::Foreign
            } else {
                State::NotInstalled
            };
        }
        let current = has_helper
            && ours.iter().all(|p| {
                std::fs::read_to_string(p).ok().as_deref() == Some(script_text(helper).as_str())
                    && executable(p)
            });
        return if current && ours.len() == 1 {
            State::Installed
        } else {
            State::Outdated
        };
    }
    if m == Manager::Thunar {
        let Ok(text) = std::fs::read_to_string(thunar_file(env)) else {
            return State::NotInstalled;
        };
        let Some(found) = thunar::label_of(&text) else {
            return State::NotInstalled;
        };
        let want = thunar::block(helper, &found);
        return if has_helper && thunar::block_of(&text) == Some(want.as_str()) {
            State::Installed
        } else {
            State::Outdated
        };
    }
    let (dir, files) = fixed(env, helper, label, m);
    let read: Vec<Option<String>> = files
        .iter()
        .map(|(name, _, _)| std::fs::read_to_string(dir.join(name)).ok())
        .collect();
    let ours: Vec<&String> = read.iter().flatten().filter(|t| is_ours(t)).collect();
    if ours.is_empty() {
        return if read.iter().any(Option::is_some) {
            State::Foreign
        } else {
            State::NotInstalled
        };
    }
    // Compare with the label the files carry, so a language change alone is
    // not "outdated".
    let found = ours.first().and_then(|t| first_name(t)).unwrap_or_default();
    let (_, wanted) = fixed(env, helper, &found, m);
    let current = has_helper
        && wanted.iter().zip(&read).all(|((name, text, exec), got)| {
            got.as_deref() == Some(text.as_str()) && (!exec || executable(&dir.join(name)))
        });
    if current {
        State::Installed
    } else {
        State::Outdated
    }
}

pub(super) fn install(env: &Env, label: &str, m: Manager) -> Result<(), String> {
    let helper = env.helper_path()?;
    if is_script_manager(m) {
        let dir = script_dir(env, m);
        let path = dir.join(file_label(label));
        let ours = our_scripts(&dir);
        if path.exists() && !ours.contains(&path) {
            return Err(format!(
                "{} is not ours; not overwriting it",
                path.display()
            ));
        }
        for old in ours.iter().filter(|p| **p != path) {
            let _ = std::fs::remove_file(old);
        }
        return write_file(&path, script_text(helper).as_bytes(), true);
    }
    if m == Manager::Thunar {
        let file = thunar_file(env);
        let existing = std::fs::read_to_string(&file).ok();
        let merged = thunar::merge(existing.as_deref(), &thunar::block(helper, label))?;
        return write_atomic(&file, &merged);
    }
    let (dir, files) = fixed(env, helper, label, m);
    for (name, _, _) in &files {
        let path = dir.join(name);
        if let Ok(t) = std::fs::read_to_string(&path) {
            if !is_ours(&t) {
                return Err(format!(
                    "{} is not ours; not overwriting it",
                    path.display()
                ));
            }
        }
    }
    for (name, text, exec) in &files {
        write_file(&dir.join(name), text.as_bytes(), *exec)?;
    }
    Ok(())
}

pub(super) fn remove(env: &Env, m: Manager) -> Result<bool, String> {
    if is_script_manager(m) {
        let mut any = false;
        for p in our_scripts(&script_dir(env, m)) {
            std::fs::remove_file(&p).map_err(|e| format!("{}: {e}", p.display()))?;
            any = true;
        }
        return Ok(any);
    }
    if m == Manager::Thunar {
        let file = thunar_file(env);
        let Ok(text) = std::fs::read_to_string(&file) else {
            return Ok(false);
        };
        let (rest, found) = thunar::unmerge(&text);
        if !found {
            return Ok(false);
        }
        if thunar::is_skeleton(&rest) {
            std::fs::remove_file(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        } else {
            write_atomic(&file, &rest)?;
        }
        return Ok(true);
    }
    let (dir, files) = fixed(env, Path::new(""), "", m);
    let mut any = false;
    for (name, _, _) in files {
        let path = dir.join(name);
        if std::fs::read_to_string(&path).is_ok_and(|t| is_ours(&t)) {
            std::fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            any = true;
        }
    }
    Ok(any)
}

/// Temp file in the same folder, then rename: Thunar never reads half a file.
fn write_atomic(path: &Path, text: &str) -> Result<(), String> {
    let tmp = path.with_extension("xml.gitpoulpi-tmp");
    write_file(&tmp, text.as_bytes(), false)?;
    // Keep the user's file mode.
    if let Ok(meta) = std::fs::metadata(path) {
        let _ = std::fs::set_permissions(&tmp, meta.permissions());
    }
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("{}: {e}", path.display())
    })
}

fn icon_path(env: &Env) -> PathBuf {
    env.data_home.join(ICON_REL)
}

/// Writes the icon the entries name (best effort refresh of the cache).
pub(super) fn write_icon(env: &Env) {
    let Some(png) = &env.icon_png else {
        return;
    };
    if write_file(&icon_path(env), png, false).is_err() {
        return;
    }
    if env.refresh_hooks {
        let _ = std::process::Command::new("gtk-update-icon-cache")
            .arg(env.data_home.join("icons/hicolor"))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}

/// Rewrites the icon when an entry of ours is left and the picture differs
/// from the one wanted (another logo color). `true`: written.
pub(super) fn sync_icon(env: &Env) -> bool {
    let Some(png) = &env.icon_png else {
        return false;
    };
    let ours = Manager::for_os(env.os)
        .iter()
        .any(|&m| matches!(state(env, "", m), State::Installed | State::Outdated));
    if !ours || std::fs::read(icon_path(env)).ok().as_ref() == Some(png) {
        return false;
    }
    write_icon(env);
    true
}

/// Deletes the icon once no entry of ours is left.
pub(super) fn remove_icon_if_unused(env: &Env) {
    let left = Manager::for_os(env.os)
        .iter()
        .any(|&m| matches!(state(env, "", m), State::Installed | State::Outdated));
    if !left {
        let _ = std::fs::remove_file(icon_path(env));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::folder_menu::{self, Unavailable};
    use crate::desktop::registry::MemoryRegistry;
    use crate::open_in::Os;
    use std::sync::Arc;

    fn env(root: &Path, present: &'static [&'static str]) -> Env {
        let helper = root.join("bin dir/gitpoulpi");
        std::fs::create_dir_all(helper.parent().unwrap()).unwrap();
        std::fs::write(&helper, "").unwrap();
        Env {
            os: Os::Linux,
            data_home: root.join("data"),
            config_home: root.join("config"),
            home: root.to_path_buf(),
            exe: helper.clone(),
            helper: Some(helper),
            has: Arc::new(move |p| present.contains(&p)),
            registry: Arc::new(MemoryRegistry::new()),
            icon_png: Some(vec![1, 2, 3]),
            refresh_hooks: false,
            ctx: crate::desktop::InstallCtx::legacy(),
        }
    }

    const LABEL: &str = "Open in GitPoulpi";
    const ALL: &[&str] = &[
        "nautilus",
        "dolphin",
        "nemo",
        "thunar",
        "caja",
        "pcmanfm-qt",
    ];

    #[test]
    fn the_text_builders_match_their_golden_output() {
        let h = Path::new("/opt/gp/gitpoulpi");
        assert_eq!(
            script_text(h),
            "#!/bin/sh\n# X-GitPoulpi=1\ndir=\"${1:-.}\"\n[ -d \"$dir\" ] || dir=.\ncd -- \"$dir\" && exec '/opt/gp/gitpoulpi' launch --folder \"$PWD\"\n"
        );
        assert_eq!(
            dolphin_text(h, LABEL),
            "[Desktop Entry]\nType=Service\nMimeType=inode/directory;\nActions=openInGitPoulpi;\nX-KDE-Priority=TopLevel\nX-GitPoulpi=1\n\n[Desktop Action openInGitPoulpi]\nName=Open in GitPoulpi\nIcon=gitpoulpi-folder\nExec=/opt/gp/gitpoulpi launch --folder %f\n"
        );
        let (sel, here) = nemo_texts(h, LABEL);
        assert_eq!(
            sel,
            "[Nemo Action]\nName=Open in GitPoulpi\nComment=Open in GitPoulpi\nExec=/opt/gp/gitpoulpi launch --folder %F\nIcon-Name=gitpoulpi-folder\nSelection=s\nExtensions=dir;\nX-GitPoulpi=1\n"
        );
        assert!(here.contains("Selection=None\nExtensions=any;\n") && here.contains("--folder %P"));
        assert_eq!(
            fm_actions_text(h, LABEL),
            "[Desktop Entry]\nType=Action\nName=Open in GitPoulpi\nIcon=gitpoulpi-folder\nProfiles=folder;\nX-GitPoulpi=1\n\n[X-Action-Profile folder]\nMimeTypes=inode/directory;\nExec=/opt/gp/gitpoulpi launch --folder %f\n"
        );
    }

    #[test]
    fn awkward_helper_paths_are_quoted_for_each_format() {
        let h = Path::new("/o p/it's $x \"q\" 50%/gitpoulpi");
        assert!(script_text(h).contains("exec '/o p/it'\\''s $x \"q\" 50%/gitpoulpi' launch"));
        let d = dolphin_text(h, LABEL);
        assert!(
            d.contains("Exec=\"/o p/it's \\$x \\\"q\\\" 50%%/gitpoulpi\" launch --folder %f"),
            "{d}"
        );
        assert!(thunar::block(h, LABEL).contains("&apos;"));
    }

    #[test]
    fn install_writes_only_for_detected_managers_and_dolphin_is_executable() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), &["dolphin", "nemo"]);
        let done = folder_menu::install(&e, LABEL).unwrap();
        assert_eq!(done, [Manager::Dolphin, Manager::Nemo]);
        let dolphin = e.data_home.join("kio/servicemenus/gitpoulpi-open.desktop");
        assert!(dolphin.is_file());
        assert!(executable(&dolphin));
        assert!(e
            .data_home
            .join("nemo/actions/gitpoulpi-open.nemo_action")
            .is_file());
        assert!(e
            .data_home
            .join("nemo/actions/gitpoulpi-open-here.nemo_action")
            .is_file());
        assert!(!e.data_home.join("nautilus").exists());
        assert!(!e.config_home.join("Thunar").exists());
        assert!(icon_path(&e).is_file());
        let s = folder_menu::status(&e, LABEL);
        assert!(folder_menu::all_installed(&s) && folder_menu::has_any_ours(&s));
        // A second Install has nothing to do.
        assert!(folder_menu::install(&e, LABEL).unwrap().is_empty());
    }

    #[test]
    fn remove_deletes_ours_only_keeps_user_scripts_and_the_icon_goes_last() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), ALL);
        let scripts = e.data_home.join("nautilus/scripts");
        std::fs::create_dir_all(&scripts).unwrap();
        std::fs::write(scripts.join("mine"), "#!/bin/sh\necho hi\n").unwrap();
        let uca = "<actions>\n<action><unique-id>1</unique-id></action>\n</actions>\n";
        std::fs::create_dir_all(e.config_home.join("Thunar")).unwrap();
        std::fs::write(thunar_file(&e), uca).unwrap();
        let done = folder_menu::install(&e, LABEL).unwrap();
        assert_eq!(done.len(), 6);
        assert!(scripts.join(LABEL).is_file() && executable(&scripts.join(LABEL)));
        assert!(e.config_home.join("caja/scripts").join(LABEL).is_file());
        assert!(icon_path(&e).is_file());
        let removed = folder_menu::remove(&e).unwrap();
        assert_eq!(removed.len(), 6);
        assert!(scripts.join("mine").is_file());
        assert!(!scripts.join(LABEL).exists());
        assert_eq!(std::fs::read_to_string(thunar_file(&e)).unwrap(), uca);
        assert!(!icon_path(&e).exists());
        assert!(!e
            .data_home
            .join("kio/servicemenus/gitpoulpi-open.desktop")
            .exists());
        assert!(folder_menu::remove(&e).unwrap().is_empty());
    }

    #[test]
    fn a_missing_uca_file_is_created_and_deleted_again() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), &["thunar"]);
        folder_menu::install(&e, LABEL).unwrap();
        assert!(thunar_file(&e).is_file());
        folder_menu::remove(&e).unwrap();
        assert!(!thunar_file(&e).exists());
    }

    #[test]
    fn a_malformed_uca_file_is_left_alone() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), &["thunar"]);
        std::fs::create_dir_all(e.config_home.join("Thunar")).unwrap();
        std::fs::write(thunar_file(&e), "garbage").unwrap();
        assert!(folder_menu::install(&e, LABEL).is_err());
        assert_eq!(std::fs::read_to_string(thunar_file(&e)).unwrap(), "garbage");
    }

    #[test]
    fn an_undetected_manager_that_still_holds_ours_is_removed() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), &["dolphin"]);
        folder_menu::install(&e, LABEL).unwrap();
        let gone = Env {
            has: Arc::new(|_| false),
            ..e.clone()
        };
        assert_eq!(folder_menu::remove(&gone).unwrap(), [Manager::Dolphin]);
    }

    #[test]
    fn a_foreign_file_is_never_overwritten_or_removed() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), &["dolphin", "nautilus"]);
        let f = e.data_home.join("kio/servicemenus/gitpoulpi-open.desktop");
        std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        std::fs::write(&f, "[Desktop Entry]\nmine\n").unwrap();
        let s = std::fs::create_dir_all(e.data_home.join("nautilus/scripts"));
        s.unwrap();
        std::fs::write(e.data_home.join("nautilus/scripts").join(LABEL), "x").unwrap();
        let st = folder_menu::status(&e, LABEL);
        assert!(st
            .entries
            .iter()
            .filter(|x| x.detected)
            .all(|x| x.state == State::Foreign));
        assert!(folder_menu::install(&e, LABEL).unwrap().is_empty());
        assert!(folder_menu::remove(&e).unwrap().is_empty());
        assert_eq!(
            std::fs::read_to_string(&f).unwrap(),
            "[Desktop Entry]\nmine\n"
        );
    }

    #[test]
    fn a_moved_helper_is_outdated_but_a_new_label_alone_is_not() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), ALL);
        folder_menu::install(&e, LABEL).unwrap();
        let st = folder_menu::status(&e, "Ouvrir dans GitPoulpi");
        assert!(
            st.entries
                .iter()
                .filter(|x| x.detected)
                .all(|x| x.state == State::Installed),
            "{st:?}"
        );
        let moved = Env {
            helper: Some(tmp.path().join("elsewhere/gitpoulpi")),
            ..e.clone()
        };
        let st = folder_menu::status(&moved, LABEL);
        assert!(st
            .entries
            .iter()
            .filter(|x| x.detected)
            .all(|x| x.state == State::Outdated));
        assert_eq!(folder_menu::sync(&moved, LABEL).unwrap().len(), 6);
        assert!(folder_menu::all_installed(&folder_menu::status(
            &moved, LABEL
        )));
    }

    #[test]
    fn sync_follows_a_new_logo_color_and_never_writes_the_icon_with_no_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), &["dolphin"]);
        assert!(folder_menu::sync(&e, LABEL).unwrap().is_empty());
        assert!(!icon_path(&e).exists(), "nothing of ours: no icon");
        folder_menu::install(&e, LABEL).unwrap();
        assert!(folder_menu::sync(&e, LABEL).unwrap().is_empty());
        let recolored = Env {
            icon_png: Some(vec![9, 9]),
            ..e.clone()
        };
        assert_eq!(
            folder_menu::sync(&recolored, LABEL).unwrap(),
            vec![Manager::Dolphin]
        );
        assert_eq!(std::fs::read(icon_path(&e)).unwrap(), vec![9, 9]);
        assert!(folder_menu::sync(&recolored, LABEL).unwrap().is_empty());
    }

    #[test]
    fn a_renamed_script_replaces_the_old_one() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path(), &["nautilus"]);
        folder_menu::install(&e, LABEL).unwrap();
        let st = folder_menu::status(&e, "Ouvrir");
        assert_eq!(st.entries[0].state, State::Installed);
        // Force the rewrite under the new name.
        super::install(&e, "Ouvrir", Manager::Nautilus).unwrap();
        let dir = e.data_home.join("nautilus/scripts");
        assert!(dir.join("Ouvrir").is_file() && !dir.join(LABEL).exists());
    }

    #[test]
    fn no_helper_means_unavailable_and_install_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let e = Env {
            helper: None,
            ..env(tmp.path(), ALL)
        };
        assert_eq!(
            folder_menu::status(&e, LABEL).unavailable,
            Some(Unavailable::NoHelper)
        );
        assert!(folder_menu::install(&e, LABEL).is_err());
    }
}
