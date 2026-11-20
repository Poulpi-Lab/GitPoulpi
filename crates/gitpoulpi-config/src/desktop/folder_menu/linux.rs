//! Linux file managers. Each has its own mechanism; the text builders are
//! pure and the rest is plain file work under [`Env::data_home`] and
//! [`Env::config_home`].
//!
//! Every installation owns its own files (the primary keeps the legacy names,
//! the others append `-i<8hex>`) and its own icon `gitpoulpi-folder[-i<8hex>].png`
//! in its logo colour. Ownership is read through [`super::is_mine`]: only what
//! is tagged with the running installation's id (or an untagged legacy file
//! while it is the primary) is ever rewritten or removed.
//!
//! - GNOME Files and Caja: an executable script in their `scripts` folder
//!   (shown under Scripts), in `scripts/<submenu>/` when a submenu is asked
//!   for (a subfolder is a submenu; installations that give the same name,
//!   trimmed and lower-cased, share the folder, and the last one out removes
//!   it with `rmdir`, which fails while it is not empty). The file name is the
//!   label, which follows the UI language, so Remove finds ours by the tag.
//! - Dolphin: a KDE service menu (must be executable, KF6 ignores it else),
//!   `X-KDE-Submenu=<name>` for a submenu (Dolphin merges same-name submenus
//!   itself, so nothing is shared).
//! - Nemo: two `.nemo_action` files (selection and background), always at the
//!   root: a submenu would mean editing the user's `actions-tree.json`.
//! - PCManFM: a file-manager action; a submenu is a shared `Type=Menu` file
//!   whose `ItemsList=` names its members. Each installation adds or removes
//!   only its own id, and the file goes when the list is empty.
//! - Thunar: one block per installation merged into `uca.xml`
//!   ([`super::thunar`]), always at the root.
//!
//! KDE Frameworks before 5.85 (`kservices5/ServiceMenus`) are not supported.

use std::path::{Path, PathBuf};

use super::{is_mine, menu_hash, normalize, thunar, Env, Manager, Who};
use crate::desktop::{exec_quote, sh_quote, write_file, State};

/// The `[Desktop Entry]` key a shared submenu file carries (no owner: the
/// members are the owners).
const MENU_KEY: &str = "X-GitPoulpi-Menu";

fn icon_rel(who: &Who) -> String {
    format!("icons/hicolor/256x256/apps/{}.png", who.icon_name())
}

/// The script GNOME Files and Caja run (given the selected folder, or none on
/// the background).
pub(crate) fn script_text(helper: &Path, who: &Who) -> String {
    format!(
        "#!/bin/sh\n{}dir=\"${{1:-.}}\"\n[ -d \"$dir\" ] || dir=.\ncd -- \"$dir\" && exec {} launch --folder \"$PWD\"\n",
        who.owner_lines("# "),
        sh_quote(&helper.to_string_lossy())
    )
}

pub(crate) fn dolphin_text(helper: &Path, label: &str, who: &Who) -> String {
    let submenu = if who.submenu.is_empty() {
        String::new()
    } else {
        format!("X-KDE-Submenu={}\n", who.submenu)
    };
    format!(
        "[Desktop Entry]\nType=Service\nMimeType=inode/directory;\nActions=openInGitPoulpi;\nX-KDE-Priority=TopLevel\n{submenu}{}\n[Desktop Action openInGitPoulpi]\nName={label}\nIcon={}\nExec={} launch --folder %f\n",
        who.owner_lines(""),
        who.icon_name(),
        exec_quote(&helper.to_string_lossy())
    )
}

/// `(selection, background)` Nemo actions.
pub(crate) fn nemo_texts(helper: &Path, label: &str, who: &Who) -> (String, String) {
    let h = exec_quote(&helper.to_string_lossy());
    let make = |sel: &str, ext: &str, arg: &str| {
        format!(
            "[Nemo Action]\nName={label}\nComment={label}\nExec={h} launch --folder {arg}\nIcon-Name={}\nSelection={sel}\nExtensions={ext}\n{}",
            who.icon_name(),
            who.owner_lines("")
        )
    };
    (make("s", "dir;", "%F"), make("None", "any;", "%P"))
}

pub(crate) fn fm_actions_text(helper: &Path, label: &str, who: &Who) -> String {
    format!(
        "[Desktop Entry]\nType=Action\nName={label}\nIcon={}\nProfiles=folder;\n{}\n[X-Action-Profile folder]\nMimeTypes=inode/directory;\nExec={} launch --folder %f\n",
        who.icon_name(),
        who.owner_lines(""),
        exec_quote(&helper.to_string_lossy())
    )
}

fn script_root(env: &Env, m: Manager) -> PathBuf {
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

/// The sub-folders of `root`, sorted.
fn subdirs(root: &Path) -> Vec<PathBuf> {
    let Ok(read) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = read
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    out.sort();
    out
}

/// The folder this installation's script goes into: `root`, or the submenu
/// folder (an existing one whose name matches trimmed and lower-cased, else a
/// new one named as typed).
fn script_dir(root: &Path, who: &Who) -> PathBuf {
    let Some(norm) = who.submenu_norm() else {
        return root.to_path_buf();
    };
    subdirs(root)
        .into_iter()
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| normalize(n) == norm)
        })
        .unwrap_or_else(|| root.join(file_label(&who.submenu)))
}

/// The scripts of the running installation in `root` and one level down.
fn our_scripts(env: &Env, root: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![root.to_path_buf()];
    dirs.extend(subdirs(root));
    let mut out = Vec::new();
    for dir in dirs {
        let Ok(read) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut files: Vec<PathBuf> = read
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .filter(|p| std::fs::read_to_string(p).is_ok_and(|t| is_mine(env, &t)))
            .collect();
        files.sort();
        out.extend(files);
    }
    out
}

/// `rmdir` the submenu folder a script left, unless it is the root. Fails
/// (and is ignored) while another installation's script is inside.
fn last_one_out(root: &Path, removed: &Path) {
    if let Some(dir) = removed.parent().filter(|d| *d != root) {
        let _ = std::fs::remove_dir(dir);
    }
}

/// The file stem every file of this installation's action shares.
fn action_stem(who: &Who) -> String {
    format!("gitpoulpi-open{}", who.suffix())
}

/// `(directory, [(file name, text, must be executable)])` for the managers
/// that keep fixed-name files.
fn fixed(
    env: &Env,
    helper: &Path,
    label: &str,
    m: Manager,
    who: &Who,
) -> (PathBuf, Vec<(String, String, bool)>) {
    let stem = action_stem(who);
    match m {
        Manager::Dolphin => (
            env.data_home.join("kio/servicemenus"),
            vec![(
                format!("{stem}.desktop"),
                dolphin_text(helper, label, who),
                true,
            )],
        ),
        Manager::Nemo => {
            let (sel, here) = nemo_texts(helper, label, who);
            (
                env.data_home.join("nemo/actions"),
                vec![
                    (format!("{stem}.nemo_action"), sel, false),
                    (
                        format!("gitpoulpi-open-here{}.nemo_action", who.suffix()),
                        here,
                        false,
                    ),
                ],
            )
        }
        _ => (
            pcmanfm_dir(env),
            vec![(
                format!("{stem}.desktop"),
                fm_actions_text(helper, label, who),
                false,
            )],
        ),
    }
}

fn pcmanfm_dir(env: &Env) -> PathBuf {
    env.data_home.join("file-manager/actions")
}

/// A shared PCManFM submenu file.
struct Menu {
    name: String,
    norm: String,
    items: Vec<String>,
}

impl Menu {
    fn path(dir: &Path, norm: &str) -> PathBuf {
        dir.join(format!("gitpoulpi-menu-m{}.desktop", menu_hash(norm)))
    }

    fn text(&self) -> String {
        let items: String = self.items.iter().map(|i| format!("{i};")).collect();
        format!(
            "[Desktop Entry]\nType=Menu\nName={}\nItemsList={items}\n{MENU_KEY}={}\n",
            self.name, self.norm
        )
    }

    fn parse(text: &str) -> Option<Menu> {
        let get = |key: &str| {
            text.lines()
                .find_map(|l| l.strip_prefix(key)?.strip_prefix('='))
        };
        let norm = get(MENU_KEY)?.to_string();
        Some(Menu {
            name: get("Name").unwrap_or_default().to_string(),
            norm,
            items: get("ItemsList")
                .unwrap_or_default()
                .split(';')
                .filter(|i| !i.is_empty())
                .map(str::to_string)
                .collect(),
        })
    }
}

/// The shared submenu files in `dir`.
fn menus(dir: &Path) -> Vec<(PathBuf, Menu)> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<(PathBuf, Menu)> = read
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("gitpoulpi-menu-m") && n.ends_with(".desktop"))
        })
        .filter_map(|p| {
            let menu = Menu::parse(&std::fs::read_to_string(&p).ok()?)?;
            Some((p, menu))
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Makes this installation's id a member of the menu `want` (created when
/// missing) and of no other, deleting a menu left with no member.
fn set_membership(env: &Env, who: &Who, want: Option<&str>) -> Result<(), String> {
    let dir = pcmanfm_dir(env);
    let id = action_stem(who);
    let write = |path: &Path, menu: &Menu| write_file(path, menu.text().as_bytes(), false);
    let mut found = false;
    for (path, mut menu) in menus(&dir) {
        let has = menu.items.contains(&id);
        if want == Some(menu.norm.as_str()) {
            found = true;
            if !has {
                menu.items.push(id.clone());
                menu.items.sort();
                write(&path, &menu)?;
            }
        } else if has {
            menu.items.retain(|i| *i != id);
            if menu.items.is_empty() {
                std::fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            } else {
                write(&path, &menu)?;
            }
        }
    }
    if let (Some(norm), false) = (want, found) {
        let path = Menu::path(&dir, norm);
        if path.exists() {
            return Err(format!(
                "{} is not ours; not overwriting it",
                path.display()
            ));
        }
        let menu = Menu {
            name: who.submenu.clone(),
            norm: norm.to_string(),
            items: vec![id],
        };
        write(&path, &menu)?;
    }
    Ok(())
}

/// Is the membership what the running installation asks for?
fn membership_current(env: &Env, who: &Who) -> bool {
    let id = action_stem(who);
    let want = who.submenu_norm();
    let all = menus(&pcmanfm_dir(env));
    let wanted_ok = want.as_deref().is_none_or(|n| {
        all.iter()
            .any(|(_, m)| m.norm == n && m.items.contains(&id))
    });
    wanted_ok
        && all
            .iter()
            .filter(|(_, m)| want.as_deref() != Some(m.norm.as_str()))
            .all(|(_, m)| !m.items.contains(&id))
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

fn first_name(text: &str) -> Option<String> {
    text.lines()
        .find_map(|l| l.strip_prefix("Name=").map(str::to_string))
}

pub(super) fn state(env: &Env, label: &str, m: Manager) -> State {
    let who = Who::of(env);
    let helper = env.helper.as_deref().unwrap_or(Path::new(""));
    let has_helper = env.helper.is_some();
    if is_script_manager(m) {
        let root = script_root(env, m);
        let ours = our_scripts(env, &root);
        if ours.is_empty() {
            return if script_dir(&root, &who).join(file_label(label)).exists() {
                State::Foreign
            } else {
                State::NotInstalled
            };
        }
        let dir = script_dir(&root, &who);
        let current = has_helper
            && ours.len() == 1
            && ours[0].parent() == Some(dir.as_path())
            && std::fs::read_to_string(&ours[0]).ok().as_deref()
                == Some(script_text(helper, &who).as_str())
            && executable(&ours[0]);
        return if current {
            State::Installed
        } else {
            State::Outdated
        };
    }
    if m == Manager::Thunar {
        let uid = thunar::unique_id(&who);
        let Ok(text) = std::fs::read_to_string(thunar_file(env)) else {
            return State::NotInstalled;
        };
        let Some(found) = thunar::label_of(&text, &uid) else {
            return State::NotInstalled;
        };
        let want = thunar::block(helper, &found, &uid, &who.icon_name());
        return if has_helper && thunar::block_of(&text, &uid) == Some(want.as_str()) {
            State::Installed
        } else {
            State::Outdated
        };
    }
    let (dir, files) = fixed(env, helper, label, m, &who);
    let read: Vec<Option<String>> = files
        .iter()
        .map(|(name, _, _)| std::fs::read_to_string(dir.join(name)).ok())
        .collect();
    let ours: Vec<&String> = read.iter().flatten().filter(|t| is_mine(env, t)).collect();
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
    let (_, wanted) = fixed(env, helper, &found, m, &who);
    let current = has_helper
        && wanted.iter().zip(&read).all(|((name, text, exec), got)| {
            got.as_deref() == Some(text.as_str()) && (!exec || executable(&dir.join(name)))
        })
        && (m != Manager::PcManFm || membership_current(env, &who));
    if current {
        State::Installed
    } else {
        State::Outdated
    }
}

pub(super) fn install(env: &Env, label: &str, m: Manager) -> Result<(), String> {
    let helper = env.helper_path()?;
    let who = Who::of(env);
    if is_script_manager(m) {
        let root = script_root(env, m);
        let path = script_dir(&root, &who).join(file_label(label));
        let ours = our_scripts(env, &root);
        if path.exists() && !ours.contains(&path) {
            return Err(format!(
                "{} is not ours; not overwriting it",
                path.display()
            ));
        }
        write_file(&path, script_text(helper, &who).as_bytes(), true)?;
        for old in ours.iter().filter(|p| **p != path) {
            let _ = std::fs::remove_file(old);
            last_one_out(&root, old);
        }
        return Ok(());
    }
    if m == Manager::Thunar {
        let file = thunar_file(env);
        let existing = std::fs::read_to_string(&file).ok();
        let uid = thunar::unique_id(&who);
        let block = thunar::block(helper, label, &uid, &who.icon_name());
        let merged = thunar::merge(existing.as_deref(), &block, &uid)?;
        return write_atomic(&file, &merged);
    }
    let (dir, files) = fixed(env, helper, label, m, &who);
    for (name, _, _) in &files {
        let path = dir.join(name);
        if let Ok(t) = std::fs::read_to_string(&path) {
            if !is_mine(env, &t) {
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
    if m == Manager::PcManFm {
        set_membership(env, &who, who.submenu_norm().as_deref())?;
    }
    Ok(())
}

pub(super) fn remove(env: &Env, m: Manager) -> Result<bool, String> {
    let who = Who::of(env);
    if is_script_manager(m) {
        let root = script_root(env, m);
        let mut any = false;
        for p in our_scripts(env, &root) {
            std::fs::remove_file(&p).map_err(|e| format!("{}: {e}", p.display()))?;
            last_one_out(&root, &p);
            any = true;
        }
        return Ok(any);
    }
    if m == Manager::Thunar {
        let file = thunar_file(env);
        let Ok(text) = std::fs::read_to_string(&file) else {
            return Ok(false);
        };
        let (rest, found) = thunar::unmerge(&text, &thunar::unique_id(&who));
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
    let (dir, files) = fixed(env, Path::new(""), "", m, &who);
    let mut any = false;
    for (name, _, _) in files {
        let path = dir.join(name);
        if std::fs::read_to_string(&path).is_ok_and(|t| is_mine(env, &t)) {
            std::fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            any = true;
        }
    }
    if m == Manager::PcManFm {
        set_membership(env, &who, None)?;
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
    env.data_home.join(icon_rel(&Who::of(env)))
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

fn holds_entry(env: &Env) -> bool {
    Manager::for_os(env.os)
        .iter()
        .any(|&m| matches!(state(env, "", m), State::Installed | State::Outdated))
}

/// Rewrites the icon when an entry of ours is left and the picture differs
/// from the one wanted (another logo color). `true`: written.
pub(super) fn sync_icon(env: &Env) -> bool {
    let Some(png) = &env.icon_png else {
        return false;
    };
    if !holds_entry(env) || std::fs::read(icon_path(env)).ok().as_ref() == Some(png) {
        return false;
    }
    write_icon(env);
    true
}

/// Deletes the icon once no entry of this installation is left (the other
/// installations' icons have their own names).
pub(super) fn remove_icon_if_unused(env: &Env) {
    if !holds_entry(env) {
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
            ico: None,
            submenu: String::new(),
        }
    }

    fn legacy() -> Who {
        Who {
            id: String::new(),
            short: "i".into(),
            primary: true,
            submenu: String::new(),
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
            script_text(h, &legacy()),
            "#!/bin/sh\n# X-GitPoulpi=1\ndir=\"${1:-.}\"\n[ -d \"$dir\" ] || dir=.\ncd -- \"$dir\" && exec '/opt/gp/gitpoulpi' launch --folder \"$PWD\"\n"
        );
        assert_eq!(
            dolphin_text(h, LABEL, &legacy()),
            "[Desktop Entry]\nType=Service\nMimeType=inode/directory;\nActions=openInGitPoulpi;\nX-KDE-Priority=TopLevel\nX-GitPoulpi=1\n\n[Desktop Action openInGitPoulpi]\nName=Open in GitPoulpi\nIcon=gitpoulpi-folder\nExec=/opt/gp/gitpoulpi launch --folder %f\n"
        );
        let (sel, here) = nemo_texts(h, LABEL, &legacy());
        assert_eq!(
            sel,
            "[Nemo Action]\nName=Open in GitPoulpi\nComment=Open in GitPoulpi\nExec=/opt/gp/gitpoulpi launch --folder %F\nIcon-Name=gitpoulpi-folder\nSelection=s\nExtensions=dir;\nX-GitPoulpi=1\n"
        );
        assert!(here.contains("Selection=None\nExtensions=any;\n") && here.contains("--folder %P"));
        assert_eq!(
            fm_actions_text(h, LABEL, &legacy()),
            "[Desktop Entry]\nType=Action\nName=Open in GitPoulpi\nIcon=gitpoulpi-folder\nProfiles=folder;\nX-GitPoulpi=1\n\n[X-Action-Profile folder]\nMimeTypes=inode/directory;\nExec=/opt/gp/gitpoulpi launch --folder %f\n"
        );
    }

    #[test]
    fn awkward_helper_paths_are_quoted_for_each_format() {
        let h = Path::new("/o p/it's $x \"q\" 50%/gitpoulpi");
        assert!(script_text(h, &legacy())
            .contains("exec '/o p/it'\\''s $x \"q\" 50%/gitpoulpi' launch"));
        let d = dolphin_text(h, LABEL, &legacy());
        assert!(
            d.contains("Exec=\"/o p/it's \\$x \\\"q\\\" 50%%/gitpoulpi\" launch --folder %f"),
            "{d}"
        );
        assert!(thunar::block(h, LABEL, thunar::UNIQUE_ID, "gitpoulpi-folder").contains("&apos;"));
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

    use crate::desktop::folder_menu::testing::{ctx, A, B};
    use crate::desktop::OWNER_TAG;
    use crate::installation::short_id;

    fn env_of(
        root: &Path,
        id: &str,
        primary: Option<&str>,
        submenu: &str,
        png: u8,
        present: &'static [&'static str],
    ) -> Env {
        Env {
            ctx: ctx(id, primary),
            submenu: submenu.into(),
            icon_png: Some(vec![png]),
            ..env(root, present)
        }
    }

    fn read(p: &Path) -> String {
        std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
    }

    #[test]
    fn scripts_go_into_a_shared_subfolder_removed_by_the_last_one() {
        let tmp = tempfile::tempdir().unwrap();
        let a = env_of(tmp.path(), A, None, "Dev", 1, &["nautilus"]);
        let b = env_of(tmp.path(), B, None, "  dev ", 2, &["nautilus"]);
        let root = a.data_home.join("nautilus/scripts");
        folder_menu::install(&a, "Open in A").unwrap();
        folder_menu::install(&b, "Open in B").unwrap();
        // One folder, named as the first one typed it, shared by both.
        let dirs: Vec<_> = subdirs(&root);
        assert_eq!(dirs, [root.join("Dev")]);
        let (sa, sb) = (root.join("Dev/Open in A"), root.join("Dev/Open in B"));
        assert!(executable(&sa) && executable(&sb));
        assert!(read(&sa).contains(&format!("# X-GitPoulpi-Install={A}")));
        assert!(
            !read(&sa).contains("# X-GitPoulpi=1"),
            "non-primary: no legacy marker"
        );
        assert_eq!(
            folder_menu::status(&a, "Open in A").entries[0].state,
            State::Installed
        );
        // The first one out leaves the folder to the second.
        folder_menu::remove(&a).unwrap();
        assert!(!sa.exists() && sb.exists() && root.join("Dev").is_dir());
        folder_menu::remove(&b).unwrap();
        assert!(!root.join("Dev").exists() && root.is_dir());
    }

    #[test]
    fn changing_the_submenu_moves_the_script_and_the_last_out_rule_applies() {
        let tmp = tempfile::tempdir().unwrap();
        let a = env_of(tmp.path(), A, None, "Dev", 1, &["nautilus"]);
        let root = a.data_home.join("nautilus/scripts");
        folder_menu::install(&a, "Open in A").unwrap();
        let flat = Env {
            submenu: String::new(),
            ..a.clone()
        };
        assert_eq!(
            folder_menu::status(&flat, "Open in A").entries[0].state,
            State::Outdated
        );
        folder_menu::sync(&flat, "Open in A").unwrap();
        assert!(root.join("Open in A").is_file() && !root.join("Dev").exists());
    }

    #[test]
    fn dolphin_menus_carry_x_kde_submenu() {
        let tmp = tempfile::tempdir().unwrap();
        let a = env_of(tmp.path(), A, None, "Dev", 1, &["dolphin"]);
        let b = env_of(tmp.path(), B, None, "", 2, &["dolphin"]);
        folder_menu::install(&a, "Open in A").unwrap();
        folder_menu::install(&b, "Open in B").unwrap();
        let dir = a.data_home.join("kio/servicemenus");
        let fa = read(&dir.join(format!("gitpoulpi-open-{}.desktop", short_id(A))));
        let fb = read(&dir.join(format!("gitpoulpi-open-{}.desktop", short_id(B))));
        assert!(fa.contains("\nX-KDE-Submenu=Dev\n"), "{fa}");
        assert!(!fb.contains("X-KDE-Submenu"), "{fb}");
        assert!(
            fa.contains(&format!("{OWNER_TAG}={A}")) && fb.contains(&format!("{OWNER_TAG}={B}"))
        );
        assert!(fa.contains(&format!("Icon=gitpoulpi-folder-{}\n", short_id(A))));
        // A never touches B's file, and removing leaves it in place.
        folder_menu::remove(&a).unwrap();
        assert!(dir
            .join(format!("gitpoulpi-open-{}.desktop", short_id(B)))
            .is_file());
        assert_eq!(
            folder_menu::status(&b, "Open in B").entries[1].state,
            State::Installed
        );
    }

    #[test]
    fn pcmanfm_menu_lists_members_and_goes_when_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let a = env_of(tmp.path(), A, None, "Dev", 1, &["pcmanfm"]);
        let b = env_of(tmp.path(), B, None, "dev", 2, &["pcmanfm"]);
        let dir = pcmanfm_dir(&a);
        let menu = Menu::path(&dir, "dev");
        folder_menu::install(&a, "Open in A").unwrap();
        folder_menu::install(&b, "Open in B").unwrap();
        let (ida, idb) = (
            format!("gitpoulpi-open-{}", short_id(A)),
            format!("gitpoulpi-open-{}", short_id(B)),
        );
        let text = read(&menu);
        assert!(text.contains("Type=Menu\nName=Dev\n"), "{text}");
        assert!(
            text.contains(&format!("ItemsList={ida};{idb};\n"))
                || text.contains(&format!("ItemsList={idb};{ida};\n")),
            "{text}"
        );
        assert!(text.contains("X-GitPoulpi-Menu=dev"));
        assert!(!text.contains(OWNER_TAG), "a shared menu has no owner");
        assert_eq!(
            folder_menu::status(&a, "Open in A").entries[5].state,
            State::Installed
        );
        folder_menu::remove(&a).unwrap();
        let text = read(&menu);
        assert!(text.contains(&idb) && !text.contains(&ida), "{text}");
        folder_menu::remove(&b).unwrap();
        assert!(!menu.exists(), "the last one out deletes the menu");
        assert!(dir.is_dir() && menus(&dir).is_empty());
    }

    #[test]
    fn thunar_keeps_one_block_per_installation() {
        let tmp = tempfile::tempdir().unwrap();
        let a = env_of(tmp.path(), A, None, "Dev", 1, &["thunar"]);
        let b = env_of(tmp.path(), B, None, "", 2, &["thunar"]);
        let uca = "<actions>\n<action><unique-id>1</unique-id></action>\n</actions>\n";
        std::fs::create_dir_all(a.config_home.join("Thunar")).unwrap();
        std::fs::write(thunar_file(&a), uca).unwrap();
        folder_menu::install(&a, "Open in A").unwrap();
        folder_menu::install(&b, "Open in B").unwrap();
        let text = read(&thunar_file(&a));
        for id in [A, B] {
            let uid = format!("gitpoulpi-folder-menu-{}", short_id(id));
            assert_eq!(
                text.matches(&format!("<unique-id>{uid}</unique-id>"))
                    .count(),
                1
            );
            assert!(text.contains(&format!("<icon>gitpoulpi-folder-{}</icon>", short_id(id))));
        }
        // No submenu in Thunar: the block stays at the root.
        assert!(text.contains("<submenu></submenu>"));
        folder_menu::remove(&a).unwrap();
        let text = read(&thunar_file(&a));
        assert!(text.contains(&short_id(B)) && !text.contains(&short_id(A)));
        assert_eq!(
            folder_menu::status(&b, "Open in B").entries[3].state,
            State::Installed
        );
        folder_menu::remove(&b).unwrap();
        assert_eq!(read(&thunar_file(&a)), uca);
    }

    #[test]
    fn each_installation_has_its_own_coloured_folder_icon() {
        let tmp = tempfile::tempdir().unwrap();
        let p = env_of(tmp.path(), A, Some(A), "", 1, &["dolphin"]);
        let b = env_of(tmp.path(), B, Some(A), "", 2, &["dolphin"]);
        folder_menu::install(&p, "Open in A").unwrap();
        folder_menu::install(&b, "Open in B").unwrap();
        let icons = p.data_home.join("icons/hicolor/256x256/apps");
        // The primary keeps the legacy name; the other has its own.
        let primary_png = icons.join("gitpoulpi-folder.png");
        let other_png = icons.join(format!("gitpoulpi-folder-{}.png", short_id(B)));
        assert_eq!(std::fs::read(&primary_png).unwrap(), vec![1]);
        assert_eq!(std::fs::read(&other_png).unwrap(), vec![2]);
        // A new colour for B rewrites B's picture only.
        let recoloured = Env {
            icon_png: Some(vec![7]),
            ..b.clone()
        };
        assert_eq!(
            folder_menu::sync(&recoloured, "Open in B").unwrap(),
            vec![Manager::Dolphin]
        );
        assert_eq!(std::fs::read(&other_png).unwrap(), vec![7]);
        assert_eq!(std::fs::read(&primary_png).unwrap(), vec![1]);
        // B leaving removes its picture and not the primary's.
        folder_menu::remove(&b).unwrap();
        assert!(!other_png.exists() && primary_png.is_file());
    }

    #[test]
    fn untagged_legacy_entries_are_untouched_without_a_primary_and_adopted_by_it() {
        let tmp = tempfile::tempdir().unwrap();
        let legacy_ctx = env(tmp.path(), &["dolphin"]);
        folder_menu::install(&legacy_ctx, LABEL).unwrap();
        let file = legacy_ctx
            .data_home
            .join("kio/servicemenus/gitpoulpi-open.desktop");
        let before = read(&file);
        // No primary: A installs its own file and cannot see the legacy one as its own.
        let a = env_of(tmp.path(), A, None, "", 1, &["dolphin"]);
        assert_eq!(
            folder_menu::status(&a, "Open in A").entries[1].state,
            State::NotInstalled
        );
        folder_menu::install(&a, "Open in A").unwrap();
        folder_menu::remove(&a).unwrap();
        assert_eq!(read(&file), before);
        // The primary adopts it and tags it.
        let p = env_of(tmp.path(), A, Some(A), "", 1, &["dolphin"]);
        assert_eq!(
            folder_menu::status(&p, LABEL).entries[1].state,
            State::Outdated
        );
        folder_menu::sync(&p, LABEL).unwrap();
        assert!(read(&file).contains(&format!("{OWNER_TAG}={A}")));
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
