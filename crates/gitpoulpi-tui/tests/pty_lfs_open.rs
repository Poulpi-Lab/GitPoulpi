//! "Open in default program" and "Open with..." on the Git LFS object card,
//! through a real pty. The desktop is fake (a temp `XDG_DATA_DIRS` and
//! `XDG_CONFIG_HOME`, `.desktop` files whose `Exec` runs a script that appends
//! its arguments to a log): no real application starts and the real desktop
//! configuration is never read.
//!
//! * the working-tree diff of an LFS file: both buttons under the card, the
//!   default program gets the file in the working tree (the real content);
//! * a commit's diff of an LFS file: the keyboard twin (`o`) hands over a
//!   read-only temp copy of the stored object, never the pointer text;
//! * the list button opens the program list.
//!
//! Opening is read-only: no journal entry, nothing to undo.

// XDG only: the fake desktop is a Linux one (the config unit tests cover the
// other systems' program lists).
#![cfg(all(unix, not(target_os = "macos")))]

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{require_git_lfs, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(60);

struct Desktop {
    dir: tempfile::TempDir,
}

impl Desktop {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let apps = root.join("share/applications");
        std::fs::create_dir_all(&apps).unwrap();
        std::fs::create_dir_all(root.join("share/mime")).unwrap();
        std::fs::write(
            root.join("share/mime/globs2"),
            "50:application/x-gp-doc:*.gpdoc\n",
        )
        .unwrap();
        for (file, name, flag) in [
            ("alpha.desktop", "Alpha Reader", "--alpha"),
            ("zeta.desktop", "Zeta Viewer", "--zeta"),
        ] {
            let script = root.join(format!("{}.sh", file.trim_end_matches(".desktop")));
            std::fs::write(
                &script,
                format!(
                    "#!/bin/sh\necho \"$@\" >> '{}'\n",
                    root.join("launches.log").display()
                ),
            )
            .unwrap();
            std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
            std::fs::write(
                apps.join(file),
                format!(
                    "[Desktop Entry]\nType=Application\nName={name}\n\
                     Exec={} {flag} %f\nMimeType=application/x-gp-doc;\n",
                    script.display()
                ),
            )
            .unwrap();
        }
        let config = root.join("config");
        std::fs::create_dir_all(&config).unwrap();
        std::fs::write(
            config.join("mimeapps.list"),
            "[Default Applications]\napplication/x-gp-doc=zeta.desktop;\n",
        )
        .unwrap();
        Self { dir }
    }

    /// Waits for the log to hold a line and returns it.
    fn launched(&self) -> String {
        let log = self.dir.path().join("launches.log");
        let deadline = Instant::now() + STEP;
        loop {
            let text = std::fs::read_to_string(&log).unwrap_or_default();
            if let Some(line) = text.lines().next() {
                return line.to_string();
            }
            assert!(Instant::now() < deadline, "no application ever ran");
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    fn launches(&self) -> String {
        std::fs::read_to_string(self.dir.path().join("launches.log")).unwrap_or_default()
    }
}

fn launch(repo: &Path, session: &str, desktop: &Desktop) -> Terminal {
    let root = desktop.dir.path();
    common::launch(repo)
        .session(session)
        .config("[ui]\ngraph_renderer = \"text\"\n")
        .size(160, 50)
        .no_exit_pause()
        .env("XDG_DATA_DIRS", root.join("share"))
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("XDG_CONFIG_DIRS", root.join("none"))
        .env("XDG_CURRENT_DESKTOP", "GPTest")
        .ready_when("the fixture's log", |s| s.contains("add big"))
        .mouse_capture()
        .start()
        .0
}

fn path_of(line: &str, flag: &str) -> PathBuf {
    PathBuf::from(
        line.strip_prefix(&format!("{flag} "))
            .unwrap_or_else(|| panic!("{line:?} does not start with {flag}"))
            .trim(),
    )
}

fn click_text(term: &mut Terminal, text: &str) {
    let (col, row) = term
        .wait_for_text(text, STEP)
        .unwrap_or_else(|e| panic!("{text:?}: {e}\n{}", term.text()));
    term.click(col + 1, row).unwrap();
}

/// The working-tree diff: both buttons, the default program gets the file in
/// place, and the list button opens the list.
#[test]
fn the_working_tree_card_opens_the_real_file_in_the_default_program() {
    require_git_lfs!("the_working_tree_card_opens_the_real_file_in_the_default_program");
    let desktop = Desktop::new();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_lfs()
        .unwrap()
        .lfs_track("*.gpdoc")
        .unwrap()
        .commit_lfs_file("big.gpdoc", &vec![1u8; 1500], "add big")
        .unwrap()
        .write_worktree("big.gpdoc", &vec![2u8; 4000])
        .unwrap()
        .build();
    let mut term = launch(repo.root(), "pty-lfs-open-wt", &desktop);
    let (col, row) = term.wait_for_text("\u{25CC}", STEP).unwrap();
    term.click(col, row).unwrap();
    click_text(&mut term, "big.gpdoc LFS");
    term.wait_for("the card with both buttons", STEP, |s| {
        s.contains("Git LFS object")
            && s.contains("1.5 KB -> 4.0 KB")
            && s.contains("Open in default program")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_text(&mut term, "Open in default program");
    let line = desktop.launched();
    let file = path_of(&line, "--zeta");
    assert_eq!(
        std::fs::canonicalize(&file).unwrap(),
        std::fs::canonicalize(repo.root().join("big.gpdoc")).unwrap(),
        "the working-tree file itself: {line}"
    );
    assert_eq!(std::fs::read(&file).unwrap(), vec![2u8; 4000]);

    // The list button, right of the default one on the card's row.
    let (c, r) = term
        .screen()
        .find("Open in \u{25BE}")
        .unwrap_or_else(|| panic!("the list button\n{}", term.text()));
    term.click(c + 1, r).unwrap();
    term.wait_for("the program list", STEP, |s| {
        s.contains("Zeta Viewer") && s.contains("Alpha Reader")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    let _ = term.stop();
}

/// A commit's diff: `o` (the default program's key) hands over a read-only
/// copy holding the stored object's bytes, not the pointer text.
#[test]
fn a_commit_card_opens_a_copy_of_the_stored_object_not_the_pointer() {
    require_git_lfs!("a_commit_card_opens_a_copy_of_the_stored_object_not_the_pointer");
    let desktop = Desktop::new();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_lfs()
        .unwrap()
        .lfs_track("*.gpdoc")
        .unwrap()
        .commit_lfs_file("big.gpdoc", &vec![7u8; 1500], "add big")
        .unwrap()
        .build();
    let mut term = launch(repo.root(), "pty-lfs-open-commit", &desktop);
    click_text(&mut term, "add big");
    click_text(&mut term, "big.gpdoc");
    term.wait_for("the card with its buttons", STEP, |s| {
        s.contains("Git LFS object") && s.contains("Open in default program")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(desktop.launches().is_empty());
    term.send_keys("o").unwrap();
    let line = desktop.launched();
    let copy = path_of(&line, "--zeta");
    assert!(
        copy.to_string_lossy().contains("gitpoulpi-image-"),
        "a temp copy, not the repository: {copy:?}"
    );
    assert!(!copy.starts_with(repo.root()));
    assert_eq!(
        std::fs::read(&copy).unwrap(),
        vec![7u8; 1500],
        "the real content, never the pointer"
    );
    assert!(std::fs::metadata(&copy).unwrap().permissions().readonly());
    let _ = term.stop();
}
