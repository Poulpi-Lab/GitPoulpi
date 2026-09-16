//! Dev mode and the "newest build" launcher.
//!
//! Dev mode is for someone who builds GitPoulpi from a checkout with more
//! than one cargo profile (`release` and `dist`). It is `general.dev_mode`
//! (never shown in Settings): absent means automatic, `true` / `false` wins.
//! Automatic = the running exe sits in `<root>/target/<profile>/` and `<root>`
//! is a GitPoulpi checkout.
//!
//! In dev mode `ui.launcher_build = "newest"` makes the shortcuts and the
//! command alias run the most recently built of `target/dist` and
//! `target/release` ([`newest_build`]). Everything here is pure file-system
//! reading, so tests use temp directories.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::{Config, LauncherBuild};

/// The profile directories "newest" chooses between. Never `debug`, never
/// `custom`.
pub const NEWEST_PROFILES: [&str; 2] = ["dist", "release"];

/// `<root>` when `exe` is `<root>/target/<profile>/<file>` and `<root>` is a
/// GitPoulpi checkout (`Cargo.toml` and `crates/gitpoulpi-tui` exist).
pub fn checkout_root(exe: &Path) -> Option<PathBuf> {
    let target = exe.parent()?.parent()?;
    if target.file_name()? != "target" {
        return None;
    }
    let root = target.parent()?;
    (root.join("Cargo.toml").is_file() && root.join("crates/gitpoulpi-tui").is_dir())
        .then(|| root.to_path_buf())
}

/// Whether dev mode is on: `explicit` (`general.dev_mode`) wins, else
/// automatic detection from `exe`.
pub fn is_dev_mode(explicit: Option<bool>, exe: &Path) -> bool {
    explicit.unwrap_or_else(|| checkout_root(exe).is_some())
}

/// Dev mode with the config's own switch.
pub fn dev_mode(config: &Config, exe: &Path) -> bool {
    is_dev_mode(config.general.dev_mode, exe)
}

/// Whether shortcuts and aliases must start the newest build: dev mode on
/// and `ui.launcher_build = "newest"`. Off, everything behaves as before.
pub fn launcher_newest(config: &Config, exe: &Path) -> bool {
    config.ui.launcher_build == LauncherBuild::Newest && dev_mode(config, exe)
}

/// [`launcher_newest`] for the `config.toml` on disk (a missing or unreadable
/// file reads as the defaults: off). For the programs that have no config of
/// their own in hand when they build a shortcut or an alias.
pub fn launcher_newest_on_disk(exe: &Path) -> bool {
    crate::ConfigPaths::discover()
        .and_then(|p| Config::load(&p.config_file()).ok())
        .is_some_and(|c| launcher_newest(&c, exe))
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path)
        .ok()
        .filter(|m| m.is_file())
        .and_then(|m| m.modified().ok())
}

/// The same file name in `<root>/target/dist/` or `<root>/target/release/`,
/// whichever exists with the newest mtime (`dist` on a tie). `exe` itself when
/// it is not under a GitPoulpi checkout's `target/<profile>/` or neither
/// build exists.
pub fn newest_build(exe: &Path) -> PathBuf {
    let (Some(root), Some(name)) = (checkout_root(exe), exe.file_name()) else {
        return exe.to_path_buf();
    };
    let mut best: Option<(SystemTime, PathBuf)> = None;
    for profile in NEWEST_PROFILES {
        let path = root.join("target").join(profile).join(name);
        if let Some(time) = modified(&path) {
            if best.as_ref().map_or(true, |(t, _)| time > *t) {
                best = Some((time, path));
            }
        }
    }
    best.map_or_else(|| exe.to_path_buf(), |(_, p)| p)
}

/// Every spelling of `exe` that "newest" mode treats as the same launcher:
/// the `dist` and `release` copies of one checkout's file, else just `exe`.
pub fn build_variants(exe: &Path) -> Vec<PathBuf> {
    let (Some(root), Some(name)) = (checkout_root(exe), exe.file_name()) else {
        return vec![exe.to_path_buf()];
    };
    NEWEST_PROFILES
        .iter()
        .map(|p| root.join("target").join(p).join(name))
        .collect()
}

fn same_path(a: &Path, b: &Path) -> bool {
    if cfg!(windows) {
        a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
    } else {
        a == b
    }
}

/// `a` and `b` name the same program of one checkout, built by `dist` or
/// `release` (or are the same path).
pub fn same_build_family(a: &Path, b: &Path) -> bool {
    same_path(a, b)
        || build_variants(a).len() == 2 && build_variants(a).iter().any(|v| same_path(v, b))
}

/// For `gitpoulpi launch`: every argument that is a GitPoulpi program in
/// `<root>/target/dist|release/` is replaced by the newest build of that
/// file, so a click starts what was built last. Other arguments pass through.
pub fn resolve_command(command: &[String]) -> Vec<String> {
    command
        .iter()
        .map(|arg| {
            let path = Path::new(arg);
            let in_profile = path
                .parent()
                .and_then(Path::file_name)
                .is_some_and(|p| NEWEST_PROFILES.iter().any(|n| p == *n));
            let ours = path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("gitpoulpi"));
            if !(in_profile && ours) {
                return arg.clone();
            }
            newest_build(path).to_string_lossy().into_owned()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn checkout(root: &Path) {
        std::fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
        std::fs::create_dir_all(root.join("crates/gitpoulpi-tui")).unwrap();
    }

    fn build(root: &Path, profile: &str, name: &str, age_secs: u64) -> PathBuf {
        let dir = root.join("target").join(profile);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        let file = std::fs::File::create(&path).unwrap();
        let when = SystemTime::now() - Duration::from_secs(age_secs);
        file.set_modified(when).unwrap();
        path
    }

    #[test]
    fn explicit_dev_mode_wins_either_way() {
        let tmp = tempfile::tempdir().unwrap();
        let exe = tmp.path().join("bin/gitpoulpi-tui");
        assert!(is_dev_mode(Some(true), &exe));
        checkout(tmp.path());
        let exe = build(tmp.path(), "release", "gitpoulpi-tui", 0);
        assert!(!is_dev_mode(Some(false), &exe));
    }

    #[test]
    fn auto_dev_mode_needs_a_checkout_around_target_profile() {
        let tmp = tempfile::tempdir().unwrap();
        let loose = build(tmp.path(), "release", "gitpoulpi-tui", 0);
        assert!(!is_dev_mode(None, &loose), "no Cargo.toml / crates");
        checkout(tmp.path());
        assert!(is_dev_mode(None, &loose));
        let elsewhere = tmp.path().join("opt/gitpoulpi-tui");
        assert!(!is_dev_mode(None, &elsewhere));
        let other = tempfile::tempdir().unwrap();
        let notarget = other.path().join("bin/gitpoulpi-tui");
        std::fs::create_dir_all(other.path().join("bin")).unwrap();
        checkout(other.path());
        assert!(!is_dev_mode(None, &notarget), "not under target/");
    }

    #[test]
    fn newest_picks_the_later_mtime_of_dist_and_release() {
        let tmp = tempfile::tempdir().unwrap();
        checkout(tmp.path());
        let dist = build(tmp.path(), "dist", "gitpoulpi-tui", 100);
        let release = build(tmp.path(), "release", "gitpoulpi-tui", 10);
        assert_eq!(newest_build(&dist), release);
        assert_eq!(newest_build(&release), release);
        build(tmp.path(), "dist", "gitpoulpi-tui", 1);
        assert_eq!(newest_build(&release), dist);
    }

    #[test]
    fn newest_with_one_build_missing_or_none_or_debug_ignored() {
        let tmp = tempfile::tempdir().unwrap();
        checkout(tmp.path());
        let debug = build(tmp.path(), "debug", "gitpoulpi-tui", 0);
        assert_eq!(newest_build(&debug), debug, "nothing to choose from");
        let release = build(tmp.path(), "release", "gitpoulpi-tui", 500);
        assert_eq!(newest_build(&debug), release, "debug never wins");
        build(tmp.path(), "custom", "gitpoulpi-tui", 0);
        assert_eq!(newest_build(&release), release, "custom never wins");
        let outside = tmp.path().join("elsewhere/gitpoulpi-tui");
        assert_eq!(newest_build(&outside), outside);
    }

    #[test]
    fn the_command_is_resolved_per_gitpoulpi_program_only() {
        let tmp = tempfile::tempdir().unwrap();
        checkout(tmp.path());
        let old = build(tmp.path(), "dist", "gitpoulpi-tui", 100);
        let new = build(tmp.path(), "release", "gitpoulpi-tui", 1);
        let cmd: Vec<String> = ["wt.exe", old.to_str().unwrap(), "--launcher"]
            .map(String::from)
            .to_vec();
        let out = resolve_command(&cmd);
        assert_eq!(out[0], "wt.exe");
        assert_eq!(out[1], new.to_string_lossy());
        assert_eq!(out[2], "--launcher");
    }

    #[test]
    fn dist_and_release_of_one_file_are_one_family() {
        let tmp = tempfile::tempdir().unwrap();
        checkout(tmp.path());
        let a = build(tmp.path(), "dist", "gitpoulpi-tui", 0);
        let b = build(tmp.path(), "release", "gitpoulpi-tui", 0);
        let c = build(tmp.path(), "release", "gitpoulpi", 0);
        assert!(same_build_family(&a, &b));
        assert!(!same_build_family(&a, &c));
        assert!(!same_build_family(
            &a,
            Path::new("/elsewhere/gitpoulpi-tui")
        ));
    }

    #[test]
    fn launcher_newest_needs_both_switches() {
        let tmp = tempfile::tempdir().unwrap();
        checkout(tmp.path());
        let exe = build(tmp.path(), "release", "gitpoulpi-tui", 0);
        let mut config = Config::default();
        assert!(!launcher_newest(&config, &exe));
        config.ui.launcher_build = LauncherBuild::Newest;
        assert!(launcher_newest(&config, &exe));
        config.general.dev_mode = Some(false);
        assert!(!launcher_newest(&config, &exe));
        config.general.dev_mode = Some(true);
        assert!(launcher_newest(&config, Path::new("/opt/gp/gitpoulpi-tui")));
    }
}
