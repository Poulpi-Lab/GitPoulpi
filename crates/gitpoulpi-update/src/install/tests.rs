use super::*;
use crate::testutil::FakeFs;
use crate::{archive, managed_names, testutil};

const LINUX: &str = "x86_64-unknown-linux-gnu";
const WIN: &str = "x86_64-pc-windows-msvc";

fn marker(version: &str, target: &str) -> String {
    Marker {
        version: version.into(),
        target: target.into(),
    }
    .render()
}

/// An install of `old` in `/app` plus the staged files of `new`.
fn install(target: &str, windows: bool, old: &str, new: &str) -> (FakeFs, PathBuf, Vec<String>) {
    let fs = FakeFs::new(windows);
    let dir = PathBuf::from("/app");
    let names = managed_names(target);
    for name in &names {
        let text = if name == MARKER_FILE {
            marker(old, target)
        } else {
            format!("{name} {old}")
        };
        fs.put(dir.join(name), &text);
    }
    let bytes = testutil::archive_for(new, target);
    let files = archive::extract(&bytes, new, target).unwrap();
    stage(&fs, &dir, &files).unwrap();
    (fs, dir, names)
}

fn text(fs: &FakeFs, dir: &Path, name: &str) -> String {
    fs.text(&dir.join(name)).unwrap_or_default()
}

#[test]
fn staging_puts_new_files_beside_the_old_ones_and_touches_nothing_else() {
    let (fs, dir, names) = install(LINUX, false, "0.1.0", "0.2.0");
    assert_eq!(staged_names(&fs, &dir, &names).len(), 5);
    assert_eq!(staged_version(&fs, &dir).as_deref(), Some("0.2.0"));
    assert_eq!(text(&fs, &dir, "gitpoulpi-tui"), "gitpoulpi-tui 0.1.0");
    assert!(!fs.exists(&dir.join("README.md.new")), "only managed files");
}

#[test]
fn a_binary_the_install_does_not_have_is_not_added() {
    let fs = FakeFs::new(false);
    let dir = PathBuf::from("/app");
    fs.put(dir.join("gitpoulpi-tui"), "t");
    fs.put(dir.join(MARKER_FILE), &marker("0.1.0", LINUX));
    let bytes = testutil::archive_for("0.2.0", LINUX);
    let files = archive::extract(&bytes, "0.2.0", LINUX).unwrap();
    let staged = stage(&fs, &dir, &files).unwrap();
    assert_eq!(staged.len(), 2, "{staged:?}");
    assert!(!fs.exists(&dir.join("gitpoulpi-mcp.new")));
}

#[test]
fn unix_apply_replaces_every_file_and_keeps_the_previous_as_old() {
    let (fs, dir, names) = install(LINUX, false, "0.1.0", "0.2.0");
    apply(&fs, &dir, &names, Replace::Atomic, "gitpoulpi-tui").unwrap();
    assert_eq!(
        text(&fs, &dir, "gitpoulpi-tui"),
        "binary gitpoulpi-tui 0.2.0"
    );
    assert_eq!(text(&fs, &dir, "gitpoulpi-tui.old"), "gitpoulpi-tui 0.1.0");
    assert!(staged_names(&fs, &dir, &names).is_empty(), "no .new left");
    assert_eq!(previous_version(&fs, &dir).as_deref(), Some("0.1.0"));
    let marker_now = Marker::parse(&text(&fs, &dir, MARKER_FILE)).unwrap();
    assert_eq!(marker_now.version, "0.2.0");
}

#[test]
fn windows_apply_moves_the_running_exe_aside_and_the_new_one_in() {
    let (fs, dir, names) = install(WIN, true, "0.1.0", "0.2.0");
    // This process runs gitpoulpi-tui.exe: it can be renamed, never replaced.
    fs.set_running(&dir.join("gitpoulpi-tui.exe"), true);
    apply(&fs, &dir, &names, Replace::MoveAside, "gitpoulpi-tui.exe").unwrap();
    assert_eq!(
        text(&fs, &dir, "gitpoulpi-tui.exe"),
        "binary gitpoulpi-tui.exe 0.2.0"
    );
    assert_eq!(
        text(&fs, &dir, "gitpoulpi-tui.exe.old"),
        "gitpoulpi-tui.exe 0.1.0"
    );
    // The old exe is still running: its .old cannot be deleted yet.
    let first = cleanup(&fs, &dir, &names);
    assert_eq!(first.kept, ["gitpoulpi-tui.exe.old"]);
    assert!(first.removed.contains(&"gitpoulpi.exe.old".to_string()));
    // After the restart that process is gone: the next start removes it.
    fs.set_running(&dir.join("gitpoulpi-tui.exe.old"), false);
    let second = cleanup(&fs, &dir, &names);
    assert_eq!(second.removed, ["gitpoulpi-tui.exe.old"]);
    assert!(second.kept.is_empty());
    assert_eq!(previous_version(&fs, &dir), None);
}

#[test]
fn windows_refuses_when_another_binary_is_running_and_changes_nothing() {
    let (fs, dir, names) = install(WIN, true, "0.1.0", "0.2.0");
    fs.set_running(&dir.join("gitpoulpi-tui.exe"), true);
    // An MCP client keeps gitpoulpi-mcp.exe open.
    fs.set_running(&dir.join("gitpoulpi-mcp.exe"), true);
    let err = apply(&fs, &dir, &names, Replace::MoveAside, "gitpoulpi-tui.exe").unwrap_err();
    assert_eq!(err, UpdateError::Locked("gitpoulpi-mcp.exe".into()));
    assert!(err.to_string().contains("gitpoulpi-mcp.exe"));
    assert_eq!(
        text(&fs, &dir, "gitpoulpi-tui.exe"),
        "gitpoulpi-tui.exe 0.1.0"
    );
    assert!(!fs.exists(&dir.join("gitpoulpi-tui.exe.old")));
    assert_eq!(
        staged_names(&fs, &dir, &names).len(),
        5,
        "the download stays staged for a retry"
    );
}

#[test]
fn a_stale_old_file_that_cannot_be_deleted_names_itself() {
    let (fs, dir, names) = install(WIN, true, "0.1.0", "0.2.0");
    fs.put(dir.join("gitpoulpi.exe.old"), "older");
    fs.set_running(&dir.join("gitpoulpi.exe.old"), true);
    let err = apply(&fs, &dir, &names, Replace::MoveAside, "gitpoulpi-tui.exe").unwrap_err();
    assert_eq!(err, UpdateError::Locked("gitpoulpi.exe.old".into()));
    assert_eq!(
        text(&fs, &dir, "gitpoulpi-tui.exe"),
        "gitpoulpi-tui.exe 0.1.0"
    );
    assert!(
        !fs.exists(&dir.join("gitpoulpi-tui.exe.old")),
        "rolled back"
    );
}

#[test]
fn a_failure_halfway_rolls_everything_back_on_both_strategies() {
    for (target, windows, replace, own) in [
        (LINUX, false, Replace::Atomic, "gitpoulpi-tui"),
        (WIN, true, Replace::MoveAside, "gitpoulpi-tui.exe"),
    ] {
        let (fs, dir, names) = install(target, windows, "0.1.0", "0.2.0");
        // The third file cannot be moved in.
        fs.fail_rename_to(dir.join(&names[2]));
        let err = apply(&fs, &dir, &names, replace, own).unwrap_err();
        assert!(matches!(err, UpdateError::Io { .. }), "{err:?}");
        for name in &names {
            let want = if name == MARKER_FILE {
                marker("0.1.0", target)
            } else {
                format!("{name} 0.1.0")
            };
            assert_eq!(text(&fs, &dir, name), want, "{name} restored");
            assert!(!fs.exists(&dir.join(format!("{name}.old"))), "{name}.old");
            assert!(fs.exists(&dir.join(format!("{name}.new"))), "{name}.new");
        }
    }
}

#[test]
fn applying_with_nothing_staged_says_so() {
    let fs = FakeFs::new(false);
    let names = managed_names(LINUX);
    assert_eq!(
        apply(&fs, Path::new("/app"), &names, Replace::Atomic, "x").unwrap_err(),
        UpdateError::NothingStaged
    );
}

#[test]
fn revert_swaps_back_and_a_second_revert_goes_forward_again() {
    for (target, windows, replace, own) in [
        (LINUX, false, Replace::Atomic, "gitpoulpi-tui"),
        (WIN, true, Replace::MoveAside, "gitpoulpi-tui.exe"),
    ] {
        let (fs, dir, names) = install(target, windows, "0.1.0", "0.2.0");
        apply(&fs, &dir, &names, replace, own).unwrap();
        assert_eq!(previous_version(&fs, &dir).as_deref(), Some("0.1.0"));
        revert(&fs, &dir, &names, replace, own).unwrap();
        let live = Marker::parse(&text(&fs, &dir, MARKER_FILE)).unwrap();
        assert_eq!(live.version, "0.1.0");
        assert_eq!(text(&fs, &dir, own), format!("{own} 0.1.0"));
        assert_eq!(previous_version(&fs, &dir).as_deref(), Some("0.2.0"));
        revert(&fs, &dir, &names, replace, own).unwrap();
        assert_eq!(
            Marker::parse(&text(&fs, &dir, MARKER_FILE))
                .unwrap()
                .version,
            "0.2.0"
        );
    }
}

#[test]
fn revert_with_no_old_files_says_so_and_a_locked_binary_refuses() {
    let (fs, dir, names) = install(WIN, true, "0.1.0", "0.2.0");
    assert_eq!(
        revert(&fs, &dir, &names, Replace::MoveAside, "gitpoulpi-tui.exe").unwrap_err(),
        UpdateError::NothingToRevert
    );
    fs.set_running(&dir.join("gitpoulpi-tui.exe"), true);
    apply(&fs, &dir, &names, Replace::MoveAside, "gitpoulpi-tui.exe").unwrap();
    fs.set_running(&dir.join("gitpoulpi.exe"), true);
    let err = revert(&fs, &dir, &names, Replace::MoveAside, "gitpoulpi-tui.exe").unwrap_err();
    assert_eq!(err, UpdateError::Locked("gitpoulpi.exe".into()));
}

#[test]
fn cleanup_after_a_successful_start_removes_the_old_files() {
    let (fs, dir, names) = install(LINUX, false, "0.1.0", "0.2.0");
    apply(&fs, &dir, &names, Replace::Atomic, "gitpoulpi-tui").unwrap();
    let done = cleanup(&fs, &dir, &names);
    assert_eq!(done.removed.len(), 5);
    assert!(done.kept.is_empty());
    assert_eq!(previous_version(&fs, &dir), None);
    // Nothing left to clean on the next start.
    assert_eq!(cleanup(&fs, &dir, &names), Cleanup::default());
}

#[cfg(unix)]
#[test]
fn the_real_file_system_swaps_atomically_and_keeps_the_exec_bit() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    let names = managed_names(LINUX);
    let fs = crate::fsops::RealFs;
    for name in &names {
        let text = if name == MARKER_FILE {
            marker("0.1.0", LINUX)
        } else {
            format!("{name} 0.1.0")
        };
        std::fs::write(dir.join(name), text).unwrap();
    }
    let files = archive::extract(&testutil::archive_for("0.2.0", LINUX), "0.2.0", LINUX).unwrap();
    stage(&fs, dir, &files).unwrap();
    apply(&fs, dir, &names, Replace::Atomic, "gitpoulpi-tui").unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("gitpoulpi-tui")).unwrap(),
        "binary gitpoulpi-tui 0.2.0"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("gitpoulpi-tui.old")).unwrap(),
        "gitpoulpi-tui 0.1.0"
    );
    let mode = std::fs::metadata(dir.join("gitpoulpi-tui"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o111, 0o111, "executable");
    revert(&fs, dir, &names, Replace::Atomic, "gitpoulpi-tui").unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("gitpoulpi-tui")).unwrap(),
        "gitpoulpi-tui 0.1.0"
    );
    cleanup(&fs, dir, &names);
    assert!(!dir.join("gitpoulpi-tui.old").exists());
}

#[test]
fn only_a_marked_build_next_to_its_marker_is_updatable() {
    let fs = FakeFs::new(false);
    let exe = Path::new("/app/gitpoulpi-tui");
    assert_eq!(
        crate::detect(&fs, exe, LINUX),
        None,
        "no marker: a cargo build"
    );
    fs.put("/app/.gitpoulpi-dist", &marker("0.1.0", LINUX));
    let found = crate::detect(&fs, exe, LINUX).unwrap();
    assert_eq!(found.exe_name, "gitpoulpi-tui");
    assert_eq!(found.marker.version, "0.1.0");
    assert_eq!(
        crate::detect(&fs, exe, WIN),
        None,
        "another platform's marker"
    );
    fs.put("/app/.gitpoulpi-dist", "garbage");
    assert_eq!(crate::detect(&fs, exe, LINUX), None);
    assert_eq!(
        crate::detect(&fs, Path::new("/elsewhere/gitpoulpi-tui"), LINUX),
        None
    );
}
