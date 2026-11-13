//! macOS: a Finder Quick Action, a `.workflow` bundle in `~/Library/Services`
//! holding a "Run Shell Script" action that gets the selected folders as
//! arguments. Finder has no entry for a folder's background, only for the
//! selected folders (right-click > Quick Actions). The first use may need
//! enabling in System Settings > Keyboard > Keyboard Shortcuts > Services.
//!
//! This is the one piece built from an undocumented file format; it lives in
//! its own file so it can be switched off alone (`coming(...)` on the page).

use std::path::{Path, PathBuf};

use super::thunar::xml_escape;
use super::Env;
use crate::desktop::{sh_quote, write_file, State, MARKER};

const PLIST_HEAD: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n";

const EXT: &str = "workflow";

fn services(env: &Env) -> PathBuf {
    env.home.join("Library/Services")
}

fn file_label(label: &str) -> String {
    label.replace(['/', ':', '\0'], "-")
}

/// `Contents/Info.plist`: the service Finder lists for folders.
pub fn info_plist(label: &str) -> String {
    let l = xml_escape(label);
    format!(
        "{PLIST_HEAD}<dict>\n\
         \t<key>NSServices</key>\n\
         \t<array>\n\
         \t\t<dict>\n\
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
         \t<key>{MARKER}</key>\n\
         \t<true/>\n\
         </dict>\n</plist>\n"
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
         \t\t<key>serviceApplicationBundleID</key>\n\t\t<string>com.apple.finder</string>\n\
         \t\t<key>serviceInputTypeIdentifier</key>\n\t\t<string>com.apple.Automator.fileSystemObject.folder</string>\n\
         \t\t<key>serviceOutputTypeIdentifier</key>\n\t\t<string>com.apple.Automator.nothing</string>\n\
         \t\t<key>serviceProcessesInput</key>\n\t\t<integer>0</integer>\n\
         \t\t<key>workflowTypeIdentifier</key>\n\t\t<string>com.apple.Automator.servicesMenu</string>\n\
         \t</dict>\n\
         </dict>\n</plist>\n"
    )
}

/// Our bundles in `~/Library/Services`: `(bundle path, label)`.
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
                .is_ok_and(|t| t.contains(&format!("<key>{MARKER}</key>")))
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
    let same = |rel: &str, want: String| {
        std::fs::read_to_string(path.join(rel)).ok().as_deref() == Some(want.as_str())
    };
    if mine.len() == 1
        && same("Contents/Info.plist", info_plist(found))
        && same("Contents/document.wflow", wflow(&script(helper)))
    {
        State::Installed
    } else {
        State::Outdated
    }
}

pub(super) fn install(env: &Env, label: &str) -> Result<(), String> {
    let helper = env.helper_path()?;
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
        info_plist(label).as_bytes(),
        false,
    )?;
    write_file(
        &root.join("Contents/document.wflow"),
        wflow(&script(helper)).as_bytes(),
        false,
    )?;
    refresh(env);
    Ok(())
}

pub(super) fn remove(env: &Env) -> Result<bool, String> {
    let mine = ours(env);
    for (p, _) in &mine {
        // `ours` read the marker from Info.plist, so only our bundles are here.
        std::fs::remove_dir_all(p).map_err(|e| format!("{}: {e}", p.display()))?;
    }
    if !mine.is_empty() {
        refresh(env);
    }
    Ok(!mine.is_empty())
}

/// Tells the services database about the change (best effort).
fn refresh(env: &Env) {
    if env.refresh_hooks {
        let _ = std::process::Command::new("/System/Library/CoreServices/pbs")
            .arg("-update")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
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
        }
    }

    #[test]
    fn the_plists_hold_the_service_the_marker_and_the_script() {
        let p = info_plist("Open & Go");
        for needle in [
            "NSSendFileTypes",
            "public.folder",
            "runWorkflowAsService",
            "com.apple.finder",
            "<key>X-GitPoulpi=1</key>",
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
            "for f in &quot;$@&quot;; do",
            "launch --folder &quot;$f&quot;; done",
        ] {
            assert!(w.contains(needle), "{needle}");
        }
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
