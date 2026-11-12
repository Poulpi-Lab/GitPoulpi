//! Settings > Repository > LFS and the toolbar's `LFS` button, through a pty
//! (the fake program is a shell script, which Git for Windows runs too).
//!
//! Like `pty_lfs_toolbar.rs`, no real `git-lfs` is needed: a stand-in script on
//! the `PATH` answers `version` and writes what `git lfs install --local`
//! writes (the filter config and a `pre-push` hook). What the test pins is
//! GitPoulpi's side: a repository where LFS is not initialized has no `LFS`
//! toolbar button, the page says so and offers the init, and once it ran the
//! button appears and the page reports the repository as initialized.
//!
//! No hard-coded coordinates: every position is found on the screen.

mod common;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(60);

struct FakeLfs {
    _dir: tempfile::TempDir,
    bin: PathBuf,
    log: PathBuf,
}

impl FakeLfs {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        common::fake_git_lfs(dir.path());
        let log = dir.path().join("calls.log");
        std::fs::write(&log, "").unwrap();
        Self {
            bin: dir.path().to_path_buf(),
            log,
            _dir: dir,
        }
    }

    fn calls(&self) -> Vec<String> {
        common::read_log(&self.log)
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
    let path = common::path_with_first(&fake.bin);
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

#[cfg_attr(
    windows,
    ignore = "the fake git-lfs on PATH never runs: Git for Windows puts its own mingw64\\bin, with the real git-lfs.exe, ahead of PATH"
)]
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
