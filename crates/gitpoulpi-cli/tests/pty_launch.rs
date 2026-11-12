//! A launcher click while a TUI is open: `gitpoulpi launch` finds the window
//! through the session daemon, the TUI is asked to raise its terminal (here
//! the test hook `GITPOULPI_RAISE_HOOK` stands in for the compositor), and no
//! second terminal is started. When the window cannot raise itself, or the
//! setting is off, the terminal command runs as before.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);
const SESSION: &str = "cli-pty-launch";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build()
}

/// A fake program: a `/bin/sh` script on unix, a `.cmd` batch file on Windows.
fn script(dir: &Path, name: &str, unix_body: &str, windows_body: &str) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join(format!("{name}.sh"));
        std::fs::write(&path, format!("#!/bin/sh\n{unix_body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        let _ = windows_body;
        path
    }
    #[cfg(windows)]
    {
        let path = dir.join(format!("{name}.cmd"));
        std::fs::write(&path, format!("@echo off\r\n{windows_body}\r\n")).unwrap();
        let _ = unix_body;
        path
    }
}

/// A fake that creates the file it is given as its first argument.
fn toucher(dir: &Path) -> PathBuf {
    script(dir, "terminal", "touch \"$1\"", "type nul > \"%~1\"")
}

/// A fake that creates `file`.
fn toucher_of(dir: &Path, name: &str, file: &Path) -> PathBuf {
    script(
        dir,
        name,
        &format!("touch '{}'", file.display()),
        &format!("type nul > \"{}\"", file.display()),
    )
}

fn exists_soon(file: &Path) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if file.exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

/// The TUI in a pty, with its own embedded daemon and the given raise hook.
fn open_window(config: &Path, repo: &Path, hook: &Path) -> Terminal {
    let term = Terminal::builder(env!("CARGO_BIN_EXE_gitpoulpi"))
        .args(["-s", SESSION])
        .cwd(repo)
        .env("GITPOULPI_CONFIG_DIR", config)
        .env("GITPOULPI_RAISE_HOOK", hook)
        .env("GITPOULPI_TERMINAL_IMAGE", "off")
        .env("GITPOULPI_GENERAL_FETCH_ON_TAB_OPEN", "false")
        .env("GITPOULPI_GENERAL_AUTO_FETCH_INTERVAL_MINUTES", "0")
        .env("GITPOULPI_RELAUNCHED", "1")
        .size(200, 50)
        .spawn()
        .expect("spawn the TUI");
    term.wait_for("the Log", STEP, |s| s.text().contains("initial commit"))
        .unwrap_or_else(|e| panic!("{e}"));
    // The window is attached once `ui viewers` lists it.
    let deadline = Instant::now() + STEP;
    loop {
        let out = Command::cargo_bin("gitpoulpi")
            .unwrap()
            .env("GITPOULPI_CONFIG_DIR", config)
            .args(["--json", "--session", SESSION])
            .args(["--repo", &repo.to_string_lossy()])
            .args(["ui", "viewers"])
            .output()
            .unwrap();
        if let Ok(v) = serde_json::from_slice::<Value>(&out.stdout) {
            if v.as_array().is_some_and(|a| !a.is_empty()) {
                break;
            }
        }
        assert!(Instant::now() < deadline, "no window attached");
        std::thread::sleep(Duration::from_millis(150));
    }
    term
}

fn launch(config: &Path, terminal: &Path, marker: &Path) -> Value {
    let out = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .arg("--json")
        .args(["launch", "--"])
        .arg(terminal)
        .arg(marker)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&out.stdout)))
}

#[test]
fn a_second_launch_shows_the_open_window_and_starts_nothing() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let raised = config.path().join("raised");
    let hook = toucher_of(config.path(), "hook", &raised);
    let terminal = toucher(config.path());
    let _tui = open_window(config.path(), repo.root(), &hook);

    let started = config.path().join("started");
    let answer = launch(config.path(), &terminal, &started);
    assert_eq!(answer["action"], "raised", "{answer}");
    assert_eq!(answer["session"], SESSION);
    assert!(exists_soon(&raised), "the window was asked to raise itself");
    std::thread::sleep(Duration::from_millis(500));
    assert!(!started.exists(), "no second terminal was started");
}

#[test]
fn a_window_that_cannot_raise_itself_leaves_the_click_to_start_a_new_one() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let hook = script(config.path(), "hook", "exit 1", "exit /b 1");
    let terminal = toucher(config.path());
    let _tui = open_window(config.path(), repo.root(), &hook);

    let started = config.path().join("started");
    let answer = launch(config.path(), &terminal, &started);
    assert_eq!(answer["action"], "started", "{answer}");
    assert_eq!(answer["reason"], "could_not_raise");
    assert!(exists_soon(&started));
}

#[test]
fn with_reuse_off_the_click_starts_a_new_window_even_though_one_is_open() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\nreuse_window = false\n",
    )
    .unwrap();
    let raised = config.path().join("raised");
    let hook = toucher_of(config.path(), "hook", &raised);
    let terminal = toucher(config.path());
    let _tui = open_window(config.path(), repo.root(), &hook);

    let started = config.path().join("started");
    let answer = launch(config.path(), &terminal, &started);
    assert_eq!(answer["reason"], "disabled", "{answer}");
    assert!(exists_soon(&started));
    assert!(!raised.exists());
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
