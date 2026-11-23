//! macOS: a Finder Quick Action, a `.workflow` bundle in `~/Library/Services`
//! holding a "Run Shell Script" action that gets the selected folders as
//! arguments. Finder has no entry for a folder's background, only for the
//! selected folders (right-click > Quick Actions). Finder only lists the
//! bundle there once the `pbs` preferences carry its presentation modes (what
//! Automator writes on save), so `install` registers them; without that the
//! bundle shows only under Services. The first use may need enabling in System
//! Settings > Keyboard > Keyboard Shortcuts > Services.
//!
//! One Quick Action `<label>.workflow` per installation, each tagged with its
//! owner ([`OWNER_TAG`]; only the primary also writes the legacy marker).
//! There is no submenu (Finder lists Quick Actions flat, so `Env::submenu` is
//! ignored here) and no coloured icon: a service can only name a system
//! `NSIconName`, never a picture of its own, so every installation shows the
//! same system symbol and is told apart by its label.
//!
//! This is the one piece built from an undocumented file format; it lives in
//! its own file so it can be switched off alone (`coming(...)` on the page).

use std::path::{Path, PathBuf};

use super::thunar::xml_escape;
use super::{may_touch, Env, Who};
use crate::desktop::owner::Facts;
use crate::desktop::{sh_quote, write_file, State, MARKER, OWNER_TAG};

const PLIST_HEAD: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n";

const EXT: &str = "workflow";

fn services(env: &Env) -> PathBuf {
    env.home.join("Library/Services")
}

fn file_label(label: &str) -> String {
    label.replace(['/', ':', '\0'], "-")
}

/// The owner keys of `Info.plist`: the legacy marker (primary only) and the tag.
fn owner_keys(who: &Who) -> String {
    let mut out = String::new();
    if who.primary {
        out.push_str(&format!("\t<key>{MARKER}</key>\n\t<true/>\n"));
    }
    if !who.id.is_empty() {
        out.push_str(&format!(
            "\t<key>{OWNER_TAG}</key>\n\t<string>{}</string>\n",
            xml_escape(&who.id)
        ));
    }
    out
}

/// What the owner rule needs from an `Info.plist`.
fn plist_facts(text: &str) -> Facts<'_> {
    let tag = text
        .split_once(&format!("<key>{OWNER_TAG}</key>"))
        .and_then(|(_, rest)| rest.split_once("<string>"))
        .and_then(|(_, rest)| rest.split_once("</string>"))
        .map(|(t, _)| t);
    Facts {
        tag,
        legacy_marker: text.contains(&format!("<key>{MARKER}</key>")),
        names_my_exe: false,
        claimed_by: None,
    }
}

/// `Contents/Info.plist`: the service Finder lists for folders.
pub(crate) fn info_plist(label: &str, who: &Who) -> String {
    let l = xml_escape(label);
    format!(
        "{PLIST_HEAD}<dict>\n\
         \t<key>NSServices</key>\n\
         \t<array>\n\
         \t\t<dict>\n\
         \t\t\t<key>NSBackgroundColorName</key>\n\
         \t\t\t<string>background</string>\n\
         \t\t\t<key>NSIconName</key>\n\
         \t\t\t<string>NSActionTemplate</string>\n\
         \t\t\t<key>NSMenuItem</key>\n\
         \t\t\t<dict>\n\
         \t\t\t\t<key>default</key>\n\
         \t\t\t\t<string>{l}</string>\n\
         \t\t\t</dict>\n\
         \t\t\t<key>NSMessage</key>\n\
         \t\t\t<string>runWorkflowAsService</string>\n\
         \t\t\t<key>NSRequiredContext</key>\n\
         \t\t\t<dict>\n\
         \t\t\t\t<key>NSApplicationIdentifier</key>\n\
         \t\t\t\t<string>com.apple.finder</string>\n\
         \t\t\t</dict>\n\
         \t\t\t<key>NSSendFileTypes</key>\n\
         \t\t\t<array>\n\
         \t\t\t\t<string>public.folder</string>\n\
         \t\t\t</array>\n\
         \t\t</dict>\n\
         \t</array>\n\
         {}</dict>\n</plist>\n",
        owner_keys(who)
    )
}

/// The script of the action: one launch per selected folder.
pub fn script(helper: &Path) -> String {
    format!(
        "for f in \"$@\"; do {} launch --folder \"$f\"; done",
        sh_quote(&helper.to_string_lossy())
    )
}

/// `Contents/document.wflow` with one "Run Shell Script" action.
pub fn wflow(script: &str) -> String {
    let s = xml_escape(script);
    format!(
        "{PLIST_HEAD}<dict>\n\
         \t<key>AMApplicationBuild</key>\n\t<string>523</string>\n\
         \t<key>AMApplicationVersion</key>\n\t<string>2.10</string>\n\
         \t<key>AMDocumentVersion</key>\n\t<string>2</string>\n\
         \t<key>actions</key>\n\
         \t<array>\n\
         \t\t<dict>\n\
         \t\t\t<key>action</key>\n\
         \t\t\t<dict>\n\
         \t\t\t\t<key>AMAccepts</key>\n\
         \t\t\t\t<dict>\n\
         \t\t\t\t\t<key>Container</key>\n\t\t\t\t\t<string>List</string>\n\
         \t\t\t\t\t<key>Optional</key>\n\t\t\t\t\t<true/>\n\
         \t\t\t\t\t<key>Types</key>\n\t\t\t\t\t<array>\n\t\t\t\t\t\t<string>com.apple.cocoa.path</string>\n\t\t\t\t\t</array>\n\
         \t\t\t\t</dict>\n\
         \t\t\t\t<key>AMActionVersion</key>\n\t\t\t\t<string>2.0.3</string>\n\
         \t\t\t\t<key>AMApplication</key>\n\t\t\t\t<array>\n\t\t\t\t\t<string>Automator</string>\n\t\t\t\t</array>\n\
         \t\t\t\t<key>AMProvides</key>\n\
         \t\t\t\t<dict>\n\
         \t\t\t\t\t<key>Container</key>\n\t\t\t\t\t<string>List</string>\n\
         \t\t\t\t\t<key>Types</key>\n\t\t\t\t\t<array>\n\t\t\t\t\t\t<string>com.apple.cocoa.path</string>\n\t\t\t\t\t</array>\n\
         \t\t\t\t</dict>\n\
         \t\t\t\t<key>ActionBundlePath</key>\n\t\t\t\t<string>/System/Library/Automator/Run Shell Script.action</string>\n\
         \t\t\t\t<key>ActionName</key>\n\t\t\t\t<string>Run Shell Script</string>\n\
         \t\t\t\t<key>ActionParameters</key>\n\
         \t\t\t\t<dict>\n\
         \t\t\t\t\t<key>COMMAND_STRING</key>\n\t\t\t\t\t<string>{s}</string>\n\
         \t\t\t\t\t<key>CheckedForUserDefaultShell</key>\n\t\t\t\t\t<true/>\n\
         \t\t\t\t\t<key>inputMethod</key>\n\t\t\t\t\t<integer>1</integer>\n\
         \t\t\t\t\t<key>shell</key>\n\t\t\t\t\t<string>/bin/sh</string>\n\
         \t\t\t\t\t<key>source</key>\n\t\t\t\t\t<string></string>\n\
         \t\t\t\t</dict>\n\
         \t\t\t\t<key>BundleIdentifier</key>\n\t\t\t\t<string>com.apple.RunShellScript</string>\n\
         \t\t\t\t<key>CFBundleVersion</key>\n\t\t\t\t<string>2.0.3</string>\n\
         \t\t\t\t<key>CanShowSelectedItemsWhenRun</key>\n\t\t\t\t<false/>\n\
         \t\t\t\t<key>CanShowWhenRun</key>\n\t\t\t\t<true/>\n\
         \t\t\t\t<key>Category</key>\n\t\t\t\t<array>\n\t\t\t\t\t<string>AMCategoryUtilities</string>\n\t\t\t\t</array>\n\
         \t\t\t\t<key>Class Name</key>\n\t\t\t\t<string>RunShellScriptAction</string>\n\
         \t\t\t\t<key>InputUUID</key>\n\t\t\t\t<string>5A1E0E55-7C2B-4D6E-9A57-0F2B6C1D4E01</string>\n\
         \t\t\t\t<key>OutputUUID</key>\n\t\t\t\t<string>5A1E0E55-7C2B-4D6E-9A57-0F2B6C1D4E02</string>\n\
         \t\t\t\t<key>UUID</key>\n\t\t\t\t<string>5A1E0E55-7C2B-4D6E-9A57-0F2B6C1D4E03</string>\n\
         \t\t\t\t<key>isViewVisible</key>\n\t\t\t\t<true/>\n\
         \t\t\t</dict>\n\
         \t\t\t<key>isViewVisible</key>\n\t\t\t<true/>\n\
         \t\t</dict>\n\
         \t</array>\n\
         \t<key>connectors</key>\n\t<dict/>\n\
         \t<key>workflowMetaData</key>\n\
         \t<dict>\n\
         \t\t<key>applicationBundleID</key>\n\t\t<string>com.apple.finder</string>\n\
         \t\t<key>applicationBundleIDsByPath</key>\n\t\t<dict>\n\t\t\t<key>/System/Library/CoreServices/Finder.app</key>\n\t\t\t<string>com.apple.finder</string>\n\t\t</dict>\n\
         \t\t<key>applicationPath</key>\n\t\t<string>/System/Library/CoreServices/Finder.app</string>\n\
         \t\t<key>applicationPaths</key>\n\t\t<array>\n\t\t\t<string>/System/Library/CoreServices/Finder.app</string>\n\t\t</array>\n\
         \t\t<key>inputTypeIdentifier</key>\n\t\t<string>com.apple.Automator.fileSystemObject.folder</string>\n\
         \t\t<key>outputTypeIdentifier</key>\n\t\t<string>com.apple.Automator.nothing</string>\n\
         \t\t<key>presentationMode</key>\n\t\t<integer>15</integer>\n\
         \t\t<key>processesInput</key>\n\t\t<integer>0</integer>\n\
         \t\t<key>serviceApplicationBundleID</key>\n\t\t<string>com.apple.finder</string>\n\
         \t\t<key>serviceApplicationPath</key>\n\t\t<string>/System/Library/CoreServices/Finder.app</string>\n\
         \t\t<key>serviceInputTypeIdentifier</key>\n\t\t<string>com.apple.Automator.fileSystemObject.folder</string>\n\
         \t\t<key>serviceOutputTypeIdentifier</key>\n\t\t<string>com.apple.Automator.nothing</string>\n\
         \t\t<key>serviceProcessesInput</key>\n\t\t<integer>0</integer>\n\
         \t\t<key>systemImageName</key>\n\t\t<string>NSActionTemplate</string>\n\
         \t\t<key>useAutomaticInputType</key>\n\t\t<integer>0</integer>\n\
         \t\t<key>workflowTypeIdentifier</key>\n\t\t<string>com.apple.Automator.servicesMenu</string>\n\
         \t</dict>\n\
         </dict>\n</plist>\n"
    )
}

/// The running installation's bundles in `~/Library/Services`: `(bundle path,
/// label)`.
fn ours(env: &Env) -> Vec<(PathBuf, String)> {
    let Ok(read) = std::fs::read_dir(services(env)) else {
        return Vec::new();
    };
    let mut out: Vec<(PathBuf, String)> = read
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == EXT))
        .filter(|p| {
            std::fs::read_to_string(p.join("Contents/Info.plist"))
                .is_ok_and(|t| may_touch(env, &plist_facts(&t)))
        })
        .filter_map(|p| {
            let label = p.file_stem()?.to_string_lossy().into_owned();
            Some((p, label))
        })
        .collect();
    out.sort();
    out
}

pub(super) fn state(env: &Env, label: &str) -> State {
    let mine = ours(env);
    let Some((path, found)) = mine.first() else {
        return if services(env)
            .join(format!("{}.{EXT}", file_label(label)))
            .exists()
        {
            State::Foreign
        } else {
            State::NotInstalled
        };
    };
    let Some(helper) = env.helper.as_deref() else {
        return State::Outdated;
    };
    let who = Who::of(env);
    let same = |rel: &str, want: String| {
        std::fs::read_to_string(path.join(rel)).ok().as_deref() == Some(want.as_str())
    };
    if mine.len() == 1
        && same("Contents/Info.plist", info_plist(found, &who))
        && same("Contents/document.wflow", wflow(&script(helper)))
    {
        State::Installed
    } else {
        State::Outdated
    }
}

pub(super) fn install(env: &Env, label: &str) -> Result<(), String> {
    let helper = env.helper_path()?;
    let who = Who::of(env);
    let root = services(env).join(format!("{}.{EXT}", file_label(label)));
    let mine = ours(env);
    if root.exists() && !mine.iter().any(|(p, _)| *p == root) {
        return Err(format!(
            "{} is not ours; not overwriting it",
            root.display()
        ));
    }
    for (old, _) in mine.iter().filter(|(p, _)| *p != root) {
        let _ = std::fs::remove_dir_all(old);
    }
    write_file(
        &root.join("Contents/Info.plist"),
        info_plist(label, &who).as_bytes(),
        false,
    )?;
    write_file(
        &root.join("Contents/document.wflow"),
        wflow(&script(helper)).as_bytes(),
        false,
    )?;
    refresh(env, Some(label));
    Ok(())
}

pub(super) fn remove(env: &Env) -> Result<bool, String> {
    let mine = ours(env);
    for (p, _) in &mine {
        // `ours` read the marker from Info.plist, so only our bundles are here.
        std::fs::remove_dir_all(p).map_err(|e| format!("{}: {e}", p.display()))?;
    }
    if !mine.is_empty() {
        refresh(env, None);
    }
    Ok(!mine.is_empty())
}

/// The `defaults` key and value that make Finder list the service under
/// right-click > Quick Actions. Automator writes this entry into the `pbs`
/// preferences when it saves a Quick Action; a bundle dropped into
/// `~/Library/Services` without it only reaches the Services submenu. The
/// presentation modes must be booleans, hence the XML fragment.
pub fn presentation_entry(label: &str) -> (String, String) {
    let escaped = label.replace('\\', "\\\\").replace('"', "\\\"");
    let key = format!("\"(null) - {escaped} - runWorkflowAsService\"");
    let value = "<dict><key>presentation_modes</key><dict>\
                 <key>ContextMenu</key><true/>\
                 <key>FinderPreview</key><true/>\
                 <key>ServicesMenu</key><true/>\
                 <key>TouchBar</key><true/>\
                 </dict></dict>"
        .to_string();
    (key, value)
}

/// Tells the services database about the change (best effort). `label` is the
/// entry just installed: its Quick Actions presentation is registered; removal
/// leaves the (harmless, Automator leaves them too) preference entry behind.
fn refresh(env: &Env, label: Option<&str>) {
    if !env.refresh_hooks {
        return;
    }
    let quiet = |mut c: std::process::Command| {
        let _ = c
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    };
    if let Some(label) = label {
        let (key, value) = presentation_entry(label);
        let mut c = std::process::Command::new("/usr/bin/defaults");
        c.args([
            "write",
            "pbs",
            "NSServicesStatus",
            "-dict-add",
            &key,
            &value,
        ]);
        quiet(c);
    }
    let mut c = std::process::Command::new("/System/Library/CoreServices/pbs");
    c.arg("-update");
    quiet(c);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::folder_menu::{self, Manager};
    use crate::desktop::registry::MemoryRegistry;
    use crate::open_in::Os;
    use std::sync::Arc;

    fn env(root: &Path) -> Env {
        Env {
            os: Os::Macos,
            data_home: PathBuf::new(),
            config_home: PathBuf::new(),
            home: root.to_path_buf(),
            exe: root.join("gitpoulpi-tui"),
            helper: Some(root.join("gitpoulpi")),
            has: Arc::new(|_| false),
            registry: Arc::new(MemoryRegistry::new()),
            icon_png: None,
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

    fn env_of(root: &Path, id: &str, primary: Option<&str>) -> Env {
        Env {
            ctx: crate::desktop::folder_menu::testing::ctx(id, primary),
            // A submenu is not possible here and must change nothing.
            submenu: "Dev".into(),
            ..env(root)
        }
    }

    #[test]
    fn macos_writes_one_quick_action_per_installation() {
        use crate::desktop::folder_menu::testing::{A, B};
        let tmp = tempfile::tempdir().unwrap();
        let (a, b) = (env_of(tmp.path(), A, None), env_of(tmp.path(), B, None));
        let services = tmp.path().join("Library/Services");
        // A legacy bundle (marker, no tag) belongs to nobody while there is no primary.
        let legacy_bundle = services.join("Open in GitPoulpi.workflow");
        std::fs::create_dir_all(legacy_bundle.join("Contents")).unwrap();
        let legacy_plist = info_plist("Open in GitPoulpi", &legacy());
        std::fs::write(legacy_bundle.join("Contents/Info.plist"), &legacy_plist).unwrap();

        assert_eq!(folder_menu::install(&a, "Open in A").unwrap().len(), 1);
        assert_eq!(folder_menu::install(&b, "Open in B").unwrap().len(), 1);
        let plist = |n: &str| {
            std::fs::read_to_string(services.join(format!("{n}.workflow/Contents/Info.plist")))
                .unwrap()
        };
        for (n, id) in [("Open in A", A), ("Open in B", B)] {
            let p = plist(n);
            assert!(p.contains(&format!("<key>{OWNER_TAG}</key>\n\t<string>{id}</string>")));
            assert!(
                !p.contains(&format!("<key>{MARKER}</key>")),
                "only the primary writes the legacy marker"
            );
        }
        assert_eq!(
            folder_menu::status(&a, "Open in A").entries[0].state,
            State::Installed
        );
        // A removes its own bundle only.
        assert_eq!(folder_menu::remove(&a).unwrap(), [Manager::Finder]);
        assert!(!services.join("Open in A.workflow").exists());
        assert!(services.join("Open in B.workflow").exists());
        assert!(legacy_bundle.join("Contents/Info.plist").is_file());
        assert!(folder_menu::remove(&b).unwrap().len() == 1);
        assert_eq!(
            std::fs::read_to_string(legacy_bundle.join("Contents/Info.plist")).unwrap(),
            legacy_plist
        );
    }

    #[test]
    fn the_plists_hold_the_service_the_marker_and_the_script() {
        let p = info_plist("Open & Go", &legacy());
        for needle in [
            "NSSendFileTypes",
            "public.folder",
            "runWorkflowAsService",
            "com.apple.finder",
            "<key>X-GitPoulpi=1</key>",
            "NSIconName",
            "NSBackgroundColorName",
            "Open &amp; Go",
        ] {
            assert!(p.contains(needle), "{needle}");
        }
        let w = wflow(&script(Path::new("/opt/it's/gitpoulpi")));
        for needle in [
            "com.apple.RunShellScript",
            "RunShellScriptAction",
            "com.apple.Automator.servicesMenu",
            "com.apple.Automator.fileSystemObject.folder",
            "<key>presentationMode</key>",
            "<key>applicationBundleIDsByPath</key>",
            "<key>serviceApplicationPath</key>",
            "for f in &quot;$@&quot;; do",
            "launch --folder &quot;$f&quot;; done",
        ] {
            assert!(w.contains(needle), "{needle}");
        }
    }

    #[test]
    fn the_quick_actions_registration_is_a_boolean_dict_keyed_by_the_label() {
        let (key, value) = presentation_entry("Open \"x\"");
        assert_eq!(key, r#""(null) - Open \"x\" - runWorkflowAsService""#);
        assert!(value.contains("<key>ContextMenu</key><true/>"));
        assert!(!value.contains("<integer>"));
    }

    #[test]
    fn install_then_remove_round_trip_and_the_label_alone_is_not_outdated() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path());
        assert_eq!(
            folder_menu::install(&e, "Open in GitPoulpi").unwrap(),
            [Manager::Finder]
        );
        let bundle = tmp
            .path()
            .join("Library/Services/Open in GitPoulpi.workflow");
        assert!(bundle.join("Contents/Info.plist").is_file());
        assert!(bundle.join("Contents/document.wflow").is_file());
        assert_eq!(
            folder_menu::status(&e, "Ouvrir").entries[0].state,
            State::Installed
        );
        assert_eq!(folder_menu::remove(&e).unwrap(), [Manager::Finder]);
        assert!(!bundle.exists());
    }

    #[test]
    fn a_bundle_without_the_marker_is_foreign_and_never_removed() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path());
        let bundle = tmp
            .path()
            .join("Library/Services/Open in GitPoulpi.workflow");
        std::fs::create_dir_all(bundle.join("Contents")).unwrap();
        std::fs::write(bundle.join("Contents/Info.plist"), "<plist/>").unwrap();
        assert_eq!(
            folder_menu::status(&e, "Open in GitPoulpi").entries[0].state,
            State::Foreign
        );
        assert!(folder_menu::install(&e, "Open in GitPoulpi")
            .unwrap()
            .is_empty());
        assert!(super::install(&e, "Open in GitPoulpi").is_err());
        assert!(folder_menu::remove(&e).unwrap().is_empty());
        assert!(bundle.join("Contents/Info.plist").is_file());
    }
}
