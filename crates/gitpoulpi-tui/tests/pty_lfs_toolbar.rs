//! The toolbar's `LFS ▾` button and the pull/push LFS pre-flight, end to
//! end through a pty (the fake program is a shell script, which Git for Windows runs too).
//!
//! These tests need no real `git-lfs`: a stand-in `git-lfs` script on the
//! `PATH` answers `version`, writes what `git lfs install --local` writes
//! (the filter config, a `pre-push` hook) and appends every call to a log
//! file the test reads back. What they pin is *what GitPoulpi asks of
//! git-lfs*, which is the part that is GitPoulpi's.
//!
//! No hard-coded coordinates: every position is found on the screen.

mod common;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(60);

/// The stand-in program: its directory (put first on `PATH`) and its log.
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

    /// Waits for a call that starts with `prefix`.
    fn wait_for_call(&self, prefix: &str) -> Vec<String> {
        let start = Instant::now();
        loop {
            let calls = self.calls();
            if calls.iter().any(|c| c.starts_with(prefix)) {
                return calls;
            }
            assert!(
                start.elapsed() < STEP,
                "git-lfs was never called with `{prefix}`; calls: {calls:?}"
            );
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

impl FakeLfs {
    /// Waits until a call equal to `call` has been logged `times` times.
    fn wait_for_calls_equal(&self, call: &str, times: usize) {
        let start = Instant::now();
        loop {
            let calls = self.calls();
            if calls.iter().filter(|c| c.as_str() == call).count() >= times {
                return;
            }
            assert!(
                start.elapsed() < STEP,
                "git-lfs was never called {times} time(s) with exactly `{call}`; calls: {calls:?}"
            );
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

/// A repository that uses LFS (its `.gitattributes` routes `*.bin`), with
/// an origin that already has `main`.
fn lfs_repo() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .with_origin()
        .unwrap()
        .commit_file(
            ".gitattributes",
            "*.bin filter=lfs diff=lfs merge=lfs -text\n",
            "attributes",
        )
        .unwrap()
        .commit_file("a.txt", "one\n", "add something")
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .build()
}

fn launch(
    repo: &Path,
    session: &str,
    fake: &FakeLfs,
    config: &str,
) -> (Terminal, tempfile::TempDir) {
    let path = common::path_with_first(&fake.bin);
    common::launch(repo)
        .session(session)
        .config(config)
        .size(180, 50)
        .no_exit_pause()
        .env("PATH", path)
        .env("FAKE_LFS_LOG", &fake.log)
        // Neither the machine's own nor the system git config may already
        // carry LFS filters: the tests are about a repository without.
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GITPOULPI_NO_BROWSER", "1")
        .ready_when("the toolbar and the log", |s| {
            s.contains("add something") && s.contains("Pull")
        })
        .mouse_capture()
        .start()
}

const ARROW: &str = "LFS \u{25BE}";

/// Clicks the `LFS ▾` button and waits for its five rows.
fn open_dropdown(term: &mut Terminal) {
    // The button is located afresh on every attempt, and a click the TUI
    // had no hit target for yet is repeated until the list answers (it only
    // repeats while the list is not up, so it cannot close it again).
    term.act_until(
        "the LFS dropdown",
        Duration::from_secs(6),
        STEP,
        |t| {
            if let Some((col, row)) = t.screen().find(ARROW) {
                t.click(col + 1, row)?;
            }
            Ok(())
        },
        |s| {
            s.contains("Fetch all LFS files")
                && s.contains("Check out all LFS files")
                && s.contains("Pull all LFS files")
                && s.contains("Push all LFS files")
                && s.contains("Prune local LFS store")
        },
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Clicks the dropdown row `label` until the dropdown has closed on it (a
/// click the TUI dropped leaves the row on screen), locating the row afresh
/// each time.
fn click_row(term: &mut Terminal, label: &str) {
    term.act_until(
        "the dropdown closed on the picked row",
        Duration::from_secs(6),
        STEP,
        |t| {
            if let Some((col, row)) = t.screen().find(label) {
                t.click(col + 1, row)?;
            }
            Ok(())
        },
        |s| !s.contains(label),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[cfg_attr(
    windows,
    ignore = "the fake git-lfs on PATH never runs: Git for Windows puts its own mingw64\\bin, with the real git-lfs.exe, ahead of PATH"
)]
#[test]
fn the_dropdown_runs_each_of_its_five_rows_by_mouse() {
    let repo = lfs_repo();
    let fake = FakeLfs::new();
    let (mut term, _config) = launch(repo.root(), "pty-lfs-tb-mouse", &fake, "");

    open_dropdown(&mut term);
    click_row(&mut term, "Fetch all LFS files");
    let calls = fake.wait_for_call("fetch origin --all");
    assert!(
        !term.text().contains("Fetch all LFS files"),
        "the dropdown closes after a pick"
    );

    open_dropdown(&mut term);
    click_row(&mut term, "Check out all LFS files");
    let calls_after = fake.wait_for_call("checkout");
    assert!(calls_after.len() > calls.len());
    term.wait_for("the checkout reported", STEP, |s| {
        s.contains("checked out the LFS files")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    open_dropdown(&mut term);
    click_row(&mut term, "Pull all LFS files");
    // The daemon runs a pull as its two halves (`fetch` then `checkout`) so
    // the download can be canceled and the checkout cannot: the stand-in
    // sees `fetch origin` (no `--all`) and a second `checkout`, never
    // `pull`.
    fake.wait_for_calls_equal("fetch origin", 1);
    fake.wait_for_calls_equal("checkout", 2);

    open_dropdown(&mut term);
    click_row(&mut term, "Push all LFS files");
    fake.wait_for_call("push origin --all");

    open_dropdown(&mut term);
    click_row(&mut term, "Prune local LFS store");
    fake.wait_for_call("prune");
    let _ = term.stop();
}

/// Keyboard: `Ctrl+G l` opens the dropdown, the arrows move and Enter
/// picks; Esc closes it without running anything.
#[cfg_attr(
    windows,
    ignore = "the fake git-lfs on PATH never runs: Git for Windows puts its own mingw64\\bin, with the real git-lfs.exe, ahead of PATH"
)]
#[test]
fn the_dropdown_opens_and_picks_from_the_keyboard() {
    let repo = lfs_repo();
    let fake = FakeLfs::new();
    let (mut term, _config) = launch(repo.root(), "pty-lfs-tb-keys", &fake, "");

    term.send_keys("C-g").unwrap();
    term.send_text("l").unwrap();
    term.wait_for("the dropdown", STEP, |s| {
        s.contains("Check out all LFS files")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Esc").unwrap();
    term.wait_for("the dropdown closed", STEP, |s| {
        !s.contains("Check out all LFS files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !fake.calls().iter().any(|c| c.starts_with("checkout")),
        "Esc runs nothing: {:?}",
        fake.calls()
    );

    term.send_keys("C-g").unwrap();
    term.send_text("l").unwrap();
    term.wait_for("the dropdown again", STEP, |s| {
        s.contains("Check out all LFS files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    fake.wait_for_call("checkout");
    let _ = term.stop();
}

/// Palette: `LFS: Open Menu` opens the dropdown, and the two commands that
/// have no older twin run on their own.
#[cfg_attr(
    windows,
    ignore = "the fake git-lfs on PATH never runs: Git for Windows puts its own mingw64\\bin, with the real git-lfs.exe, ahead of PATH"
)]
#[test]
fn the_palette_reaches_the_menu_and_the_new_commands() {
    let repo = lfs_repo();
    let fake = FakeLfs::new();
    let (mut term, _config) = launch(repo.root(), "pty-lfs-tb-palette", &fake, "");

    term.send_keys("C-p").unwrap();
    term.send_text("LFS: Open Menu").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Command Palette") && s.contains("LFS: Open Menu")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the dropdown", STEP, |s| {
        s.contains("Fetch all LFS files") && !s.contains("Command Palette")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Esc").unwrap();
    term.wait_for("the dropdown closed", STEP, |s| {
        !s.contains("Fetch all LFS files")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-p").unwrap();
    term.send_text("LFS: Check Out All Files").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("LFS: Check Out All Files") && s.contains("Command Palette")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    fake.wait_for_call("checkout");

    term.send_keys("C-p").unwrap();
    term.send_text("LFS: Push All Objects").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("LFS: Push All Objects") && s.contains("Command Palette")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    fake.wait_for_call("push origin --all");
    let _ = term.stop();
}

/// Hover: the button lights up under the pointer and shows its tooltip.
#[cfg_attr(
    windows,
    ignore = "the fake git-lfs on PATH never runs: Git for Windows puts its own mingw64\\bin, with the real git-lfs.exe, ahead of PATH"
)]
#[test]
fn the_button_highlights_and_explains_itself_on_hover() {
    let repo = lfs_repo();
    let fake = FakeLfs::new();
    let (mut term, _config) = launch(repo.root(), "pty-lfs-tb-hover", &fake, "");
    let (col, row) = term.wait_for_text(ARROW, STEP).unwrap();
    term.move_to(0, 0).unwrap();
    term.move_to(0, 0).unwrap();
    let rest = term.screen().cell(col + 1, row).map(|c| c.bg);
    term.move_to(col + 1, row).unwrap();
    // The fill under the pointer is part of the wait: the tooltip and the
    // highlight are not drawn in one frame.
    let screen = term
        .wait_for("the tooltip and the hover fill", STEP, |s| {
            s.contains("check out, pull or push the large files")
                && s.cell(col + 1, row).map(|c| c.bg) != rest
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let hovered = screen.cell(col + 1, row).map(|c| c.bg);
    assert_ne!(
        rest, hovered,
        "the button fills differently under the pointer"
    );
    let _ = term.stop();
}

/// With Git LFS turned off the button is not on the toolbar at all.
#[cfg_attr(
    windows,
    ignore = "the fake git-lfs on PATH never runs: Git for Windows puts its own mingw64\\bin, with the real git-lfs.exe, ahead of PATH"
)]
#[test]
fn the_button_is_absent_when_lfs_is_turned_off() {
    let repo = lfs_repo();
    let fake = FakeLfs::new();
    let (term, _config) = launch(
        repo.root(),
        "pty-lfs-tb-off",
        &fake,
        "[git]\nlfs_enabled = false\n",
    );
    std::thread::sleep(Duration::from_millis(500));
    assert!(
        !term.text().contains(ARROW),
        "no LFS button:\n{}",
        term.text()
    );
    assert!(
        term.text().contains("Search"),
        "the rest of the toolbar is there"
    );
    let _ = term.stop();
}

/// Push pre-flight: a repository that uses LFS but has no `pre-push` hook
/// gets one (`git lfs install --local`) before the push, and the status
/// line says so. Without the hook the push would upload no LFS objects.
#[cfg_attr(
    windows,
    ignore = "the fake git-lfs on PATH never runs: Git for Windows puts its own mingw64\\bin, with the real git-lfs.exe, ahead of PATH"
)]
#[test]
fn a_push_installs_the_missing_lfs_hook_first_and_says_so() {
    let repo = lfs_repo();
    repo.git(&["commit", "--allow-empty", "-m", "ahead of origin"]);
    let fake = FakeLfs::new();
    let (mut term, _config) = launch(repo.root(), "pty-lfs-tb-push", &fake, "");
    assert!(!repo.root().join(".git/hooks/pre-push").exists());

    term.send_keys("C-g").unwrap();
    term.send_text("P").unwrap();
    term.wait_for("the pre-flight reported on the status line", STEP, |s| {
        s.contains("installed the LFS filters")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let calls = fake.wait_for_call("install --local");
    assert!(calls.iter().any(|c| c == "install --local"), "{calls:?}");
    let hook = std::fs::read_to_string(repo.root().join(".git/hooks/pre-push")).unwrap();
    assert!(hook.contains("git lfs"), "{hook}");
    // The push itself still went through, to the one remote.
    let start = Instant::now();
    while repo
        .git_in_remote(&["log", "--format=%s", "main"])
        .lines()
        .next()
        != Some("ahead of origin")
    {
        assert!(start.elapsed() < STEP, "the push never arrived");
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = term.stop();
}

/// Pull pre-flight: missing smudge filter, same repair before the pull.
#[cfg_attr(
    windows,
    ignore = "the fake git-lfs on PATH never runs: Git for Windows puts its own mingw64\\bin, with the real git-lfs.exe, ahead of PATH"
)]
#[test]
fn a_pull_installs_the_missing_lfs_filters_first() {
    let repo = lfs_repo();
    let fake = FakeLfs::new();
    let (mut term, _config) = launch(repo.root(), "pty-lfs-tb-pull", &fake, "");
    assert!(repo
        .try_git(&["config", "--local", "filter.lfs.smudge"])
        .is_none());

    term.send_keys("C-g").unwrap();
    term.send_text("p").unwrap();
    fake.wait_for_call("install --local");
    term.wait_for("the pre-flight reported", STEP, |s| {
        s.contains("installed the LFS filters")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(repo
        .try_git(&["config", "--local", "filter.lfs.smudge"])
        .is_some());
    let _ = term.stop();
}

/// `git.lfs_autofix_filters = false` turns the pre-flight off: the push
/// goes through exactly as before and nothing is installed.
#[cfg_attr(
    windows,
    ignore = "the fake git-lfs on PATH never runs: Git for Windows puts its own mingw64\\bin, with the real git-lfs.exe, ahead of PATH"
)]
#[test]
fn the_preflight_can_be_turned_off() {
    let repo = lfs_repo();
    repo.git(&["commit", "--allow-empty", "-m", "ahead of origin"]);
    let fake = FakeLfs::new();
    let (mut term, _config) = launch(
        repo.root(),
        "pty-lfs-tb-nofix",
        &fake,
        "[git]\nlfs_autofix_filters = false\n",
    );
    term.send_keys("C-g").unwrap();
    term.send_text("P").unwrap();
    let start = Instant::now();
    while repo
        .git_in_remote(&["log", "--format=%s", "main"])
        .lines()
        .next()
        != Some("ahead of origin")
    {
        assert!(start.elapsed() < STEP, "the push never arrived");
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(
        !fake.calls().iter().any(|c| c.starts_with("install")),
        "nothing installed: {:?}",
        fake.calls()
    );
    assert!(!repo.root().join(".git/hooks/pre-push").exists());
    assert!(!term.text().contains("installed the LFS filters"));
    let _ = term.stop();
}
