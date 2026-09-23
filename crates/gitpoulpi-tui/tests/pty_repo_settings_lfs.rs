//! Settings > Repository > LFS and the toolbar's `LFS` button, through a pty
//! (unix only: the fake program is a shell script).
//!
//! Like `pty_lfs_toolbar.rs`, no real `git-lfs` is needed: a stand-in script on
//! the `PATH` answers `version` and writes what `git lfs install --local`
//! writes (the filter config and a `pre-push` hook). What the test pins is
//! GitPoulpi's side: a repository where LFS is not initialized has no `LFS`
//! toolbar button, the page says so and offers the init, and once it ran the
//! button appears and the page reports the repository as initialized.
//!
//! No hard-coded coordinates: every position is found on the screen.
#![cfg(unix)]

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(60);

const SCRIPT: &str = r#"#!/bin/sh
echo "$@" >> "$FAKE_LFS_LOG"
case "$1" in
  version) echo "git-lfs/3.5.1 (GitHub; linux amd64; go 1.21.0)" ;;
  install)
    git config --local filter.lfs.clean "git-lfs clean -- %f"
    git config --local filter.lfs.smudge "git-lfs smudge -- %f"
    git config --local filter.lfs.process "git-lfs filter-process"
    git config --local filter.lfs.required true
    hooks="$(git rev-parse --git-path hooks)"
    mkdir -p "$hooks"
    printf '#!/bin/sh\n# stand-in for the git lfs pre-push hook\nexit 0\n' > "$hooks/pre-push"
    chmod +x "$hooks/pre-push"
    ;;
esac
exit 0
"#;

struct FakeLfs {
    _dir: tempfile::TempDir,
    bin: PathBuf,
    log: PathBuf,
}

impl FakeLfs {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("git-lfs");
        std::fs::write(&script, SCRIPT).unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let log = dir.path().join("calls.log");
        std::fs::write(&log, "").unwrap();
        Self {
            bin: dir.path().to_path_buf(),
            log,
            _dir: dir,
        }
    }

    fn calls(&self) -> Vec<String> {
        std::fs::read_to_string(&self.log)
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }

    fn wait_for_call(&self, prefix: &str) {
        let start = Instant::now();
        while !self.calls().iter().any(|c| c.starts_with(prefix)) {
            assert!(
                start.elapsed() < STEP,
                "git-lfs was never called with `{prefix}`: {:?}",
                self.calls()
            );
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

fn launch(repo: &Path, fake: &FakeLfs) -> Terminal {
    let path = format!(
        "{}:{}",
        fake.bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let (term, _config) = common::launch(repo)
        .session("pty-repo-settings-lfs")
        .config("")
        .size(200, 60)
        .no_exit_pause()
        .env("PATH", path)
        .env("FAKE_LFS_LOG", &fake.log)
        // Neither the machine's own nor the system git config may already
        // carry LFS filters: the test is about a repository without.
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GITPOULPI_NO_BROWSER", "1")
        .ready_when("the toolbar and the log", |s| {
            s.contains("lfs-page-commit") && s.contains("Pull")
        })
        .mouse_capture()
        .start();
    // The config dir lives as long as the terminal.
    std::mem::forget(_config);
    term
}

const BUTTON: &str = "LFS \u{25BE}";

fn click_text(term: &mut Terminal, text: &str) {
    let screen = term
        .wait_for(&format!("`{text}` on screen"), STEP, |s| s.contains(text))
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find(text).unwrap();
    term.click(col + 1, row).unwrap();
}

#[test]
fn a_plain_repo_has_no_lfs_button_until_the_page_initializes_it() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "lfs-page-commit")
        .unwrap()
        .build();
    let fake = FakeLfs::new();
    let mut term = launch(repo.root(), &fake);

    // Give the LFS read time to land: the toolbar must still have no button.
    std::thread::sleep(Duration::from_secs(2));
    assert!(
        !term.text().contains(BUTTON),
        "a repository without LFS shows no LFS button:\n{}",
        term.text()
    );

    // Settings > LFS.
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "LFS");
    term.wait_for("the not initialized state", STEP, |s| {
        s.contains("Git LFS is not set up in") && s.contains("Set up LFS here")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    click_text(&mut term, "Set up LFS here");
    fake.wait_for_call("install --local");
    term.wait_for("the page reporting LFS as initialized", STEP, |s| {
        s.contains("Git LFS is set up in") && !s.contains("Set up LFS here")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Back to the repository: the toolbar now has the button.
    term.send_keys("Escape").unwrap();
    term.wait_for("the LFS button", STEP, |s| s.contains(BUTTON))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}
