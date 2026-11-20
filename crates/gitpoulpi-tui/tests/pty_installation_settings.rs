//! Settings > Launcher > "This installation", end to end through a pty: the
//! name, colour and version of the running installation, becoming the primary,
//! and the environment. The harness pins `GITPOULPI_INSTALL_ID=pty` (id
//! `p-pty`) and, with no primary declared, its launchers carry its own name.
//!
//! HOME, XDG_DATA_HOME and XDG_CONFIG_HOME point at a scratch directory and the
//! terminal is a fake program on PATH, so nothing here can reach the real
//! launchers or the real config. The files are read back, not trusted from the
//! screen alone.

#![cfg(all(unix, not(target_os = "macos")))]

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const MESSAGE: &str = "installation probe commit";

struct Setup {
    term: Terminal,
    config: tempfile::TempDir,
    home: tempfile::TempDir,
    _repo: gitpoulpi_testkit::FixtureRepo,
}

impl Setup {
    fn data(&self) -> PathBuf {
        self.home.path().join("data")
    }

    fn config_toml(&self) -> toml::Value {
        let text =
            std::fs::read_to_string(self.config.path().join("config.toml")).unwrap_or_default();
        toml::from_str(&text).unwrap_or_else(|e| panic!("config.toml: {e}\n{text}"))
    }

    /// The `[installation.settings.p-pty]` table, empty until written.
    fn entry(&self) -> toml::Value {
        self.config_toml()
            .get("installation")
            .and_then(|i| i.get("settings"))
            .and_then(|s| s.get("p-pty"))
            .cloned()
            .unwrap_or_else(|| toml::Value::Table(Default::default()))
    }

    /// Every `.desktop` launcher under the scratch applications folder.
    fn launchers(&self) -> Vec<PathBuf> {
        let mut found: Vec<PathBuf> = std::fs::read_dir(self.data().join("applications"))
            .map(|d| d.flatten().map(|e| e.path()).collect())
            .unwrap_or_default();
        found.sort();
        found
    }
}

/// Starts the TUI with `config_extra` after the `[ui]` table the tests share,
/// a fake `kitty` on PATH and the scratch home. `prepare` runs first, with the
/// scratch home, to lay files out.
fn start(session: &str, config_toml: &str, prepare: impl FnOnce(&Path)) -> Setup {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = home.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let kitty = bin.join("kitty");
    std::fs::write(&kitty, "#!/bin/sh\n").unwrap();
    std::fs::set_permissions(&kitty, std::fs::Permissions::from_mode(0o755)).unwrap();
    prepare(home.path());
    std::fs::write(config.path().join("config.toml"), config_toml).unwrap();
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .env("HOME", home.path())
        .env("PATH", &path)
        .env("XDG_DATA_HOME", home.path().join("data"))
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .size(200, 90)
        .no_exit_pause()
        .ready(MESSAGE)
        .spawn();
    Setup {
        term,
        config,
        home,
        _repo: repo,
    }
}

fn palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the palette entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

fn wait_until(what: &str, term: &Terminal, ok: impl Fn() -> bool) {
    let deadline = Instant::now() + STARTUP;
    while !ok() {
        assert!(Instant::now() < deadline, "{what}\n{}", term.text());
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// The `Name=` line of `path`.
fn desktop_name(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()?
        .lines()
        .find_map(|l| l.strip_prefix("Name=").map(str::to_string))
}

const UI: &str = "[ui]\ngraph_renderer = \"text\"\nlaunch_terminal = \"kitty\"\n";

#[test]
fn name_colour_and_version_from_the_keyboard_write_this_installation_only() {
    let mut s = start("installation-name", UI, |_| {});
    // A launcher to rename: created from the palette, under this installation's own name.
    palette(&mut s.term, "launcher create", "Launcher: Create Shortcut");
    wait_until("the launcher was written", &s.term, || {
        !s.launchers().is_empty()
    });
    let launcher = s.launchers().remove(0);
    assert!(
        launcher
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("gitpoulpi-i"),
        "a non-primary launcher is not the legacy file: {}",
        launcher.display()
    );

    // Launcher name: the palette opens the row for typing.
    palette(
        &mut s.term,
        "settings: launcher name",
        "Settings: Launcher name...",
    );
    s.term
        .wait_for("the name row open for typing", STEP, |t| {
            t.contains("Launcher name") && t.contains("Settings - Tab")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    for _ in 0..48 {
        s.term.send_keys("BSpace").unwrap();
    }
    s.term.send_text("Poulpi Test").unwrap();
    s.term.send_keys("Enter").unwrap();
    wait_until("the name reached config.toml", &s.term, || {
        s.entry().get("name").and_then(|n| n.as_str()) == Some("Poulpi Test")
    });
    wait_until("the launcher was renamed", &s.term, || {
        desktop_name(&launcher).as_deref() == Some("Poulpi Test")
    });

    // Include version in the launcher name: a checkbox, flipped from the palette.
    palette(
        &mut s.term,
        "settings: toggle include version",
        "Settings: Toggle Include version in the launcher name",
    );
    let version = {
        let v = env!("CARGO_PKG_VERSION");
        let mut parts = v.split('.');
        format!("{}.{}", parts.next().unwrap(), parts.next().unwrap())
    };
    wait_until("the version reached config.toml", &s.term, || {
        s.entry().get("name_with_version").and_then(|n| n.as_bool()) == Some(true)
    });
    wait_until("the launcher carries the version", &s.term, || {
        desktop_name(&launcher).as_deref() == Some(&format!("Poulpi Test {version}"))
    });

    // Octopus colour: the palette opens the page on the row, a pick writes it.
    palette(
        &mut s.term,
        "settings: octopus colour",
        "Settings: Octopus colour...",
    );
    s.term
        .wait_for("Settings on the colour row", STEP, |t| {
            t.contains("Octopus colour") && t.contains("Settings - Tab")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let before = s
        .entry()
        .get("logo_color")
        .and_then(|c| c.as_str())
        .map(str::to_string);
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the colour list", STEP, |t| t.contains("\u{2713}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Down").unwrap();
    s.term.send_keys("Enter").unwrap();
    wait_until("a colour reached the entry", &s.term, || {
        let now = s
            .entry()
            .get("logo_color")
            .and_then(|c| c.as_str())
            .map(str::to_string);
        now.is_some() && now != before
    });

    // Only this installation's entry changed: no `ui.*` key, no other entry.
    let file = s.config_toml();
    assert!(
        file.get("ui").and_then(|u| u.get("logo_color")).is_none(),
        "ui.logo_color is the primary's: {file}"
    );
    assert!(
        file.get("ui")
            .and_then(|u| u.get("command_alias"))
            .is_none(),
        "{file}"
    );
    let entries = file["installation"]["settings"].as_table().unwrap();
    assert_eq!(entries.keys().collect::<Vec<_>>(), ["p-pty"], "{file}");
    assert!(file["installation"].get("primary").is_none(), "{file}");
    let _ = s.term.stop();
}

#[test]
fn becoming_primary_takes_the_legacy_names() {
    // A launcher an older GitPoulpi made under the plain name, nobody's.
    let legacy_text = "[Desktop Entry]\nType=Application\nName=GitPoulpi\nExec=/opt/old/gitpoulpi\nX-GitPoulpi=1\n";
    let mut s = start("installation-primary", UI, |home| {
        let apps = home.join("data/applications");
        std::fs::create_dir_all(&apps).unwrap();
        std::fs::write(apps.join("gitpoulpi.desktop"), legacy_text).unwrap();
    });
    let legacy = s.data().join("applications/gitpoulpi.desktop");
    // The one-time note, non-modal, points at Settings > Launcher.
    s.term
        .wait_for("the legacy launchers note", STARTUP, |t| {
            t.contains("Legacy launchers found")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert_eq!(
        std::fs::read_to_string(&legacy).unwrap(),
        legacy_text,
        "an untagged legacy launcher is never rewritten on its own"
    );

    palette(
        &mut s.term,
        "launcher become the primary",
        "Launcher: Become the Primary Installation",
    );
    wait_until("this installation became the primary", &s.term, || {
        s.config_toml()
            .get("installation")
            .and_then(|i| i.get("primary"))
            .and_then(|p| p.as_str())
            == Some("p-pty")
    });
    wait_until(
        "the legacy launcher is this installation's",
        &s.term,
        || {
            std::fs::read_to_string(&legacy).is_ok_and(|t| {
                t.contains("X-GitPoulpi-Install=p-pty") && t.contains("X-GitPoulpi=1")
            })
        },
    );
    assert_eq!(
        desktop_name(&legacy).as_deref(),
        Some("GitPoulpi"),
        "the primary owns the plain name"
    );
    assert_eq!(
        s.launchers(),
        [legacy],
        "no launcher under a name of its own is left next to it"
    );
    let _ = s.term.stop();
}

/// The screen shows the Become the primary row with `mark` as its checkbox.
fn primary_row(screen: &str, mark: &str) -> bool {
    screen
        .lines()
        .any(|l| l.contains("Become the primary") && l.contains(mark))
}

#[test]
fn the_primary_checkbox_follows_the_state_and_enter_toggles_like_space() {
    let ui = format!("{UI}logo_color = \"violet\"\n");
    let mut s = start("installation-primary-row", &ui, |_| {});
    palette(
        &mut s.term,
        "settings: become the primary",
        "Settings: Become the primary...",
    );
    s.term
        .wait_for("Settings on the primary row", STEP, |t| {
            primary_row(&t.text(), "[ ]")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Space").unwrap();
    s.term
        .wait_for("the checkbox is checked", STEP, |t| {
            primary_row(&t.text(), "[x]")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert_eq!(
        s.config_toml()["installation"]["primary"].as_str(),
        Some("p-pty")
    );
    s.term
        .wait_for("the name row says the primary is GitPoulpi", STEP, |t| {
            t.contains("the primary is always called GitPoulpi")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    // Enter toggles like Space (the Settings checkbox convention).
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the checkbox is cleared", STEP, |t| {
            primary_row(&t.text(), "[ ]")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let file = s.config_toml();
    assert!(file["installation"].get("primary").is_none(), "{file}");
    assert_eq!(
        file["ui"]["logo_color"].as_str(),
        Some("violet"),
        "ui.* is what it was before the primary: {file}"
    );
    assert!(file["installation"].get("ui_before").is_none(), "{file}");
    let _ = s.term.stop();
}

#[test]
fn the_environment_row_writes_this_installations_env() {
    let other = tempfile::tempdir().unwrap();
    let config = format!(
        "{UI}\n[installation.settings.bbbb000000000002]\npath = {:?}\nkind = \"dev\"\nname = \"GitPoulpi (dev)\"\nenv = \"keepme\"\n",
        other.path().display().to_string()
    );
    let mut s = start("installation-env", &config, |_| {});
    palette(
        &mut s.term,
        "settings: environment",
        "Settings: Environment...",
    );
    s.term
        .wait_for("Settings on the environment row", STEP, |t| {
            t.contains("Environment") && t.contains("Settings - Tab")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the list offers the other's environment", STEP, |t| {
            t.contains("keepme") && t.contains("own (default)")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_text("shared").unwrap();
    s.term.send_keys("Enter").unwrap();
    wait_until("the environment reached the entry", &s.term, || {
        s.entry().get("env").and_then(|e| e.as_str()) == Some("shared")
    });
    let file = s.config_toml();
    assert_eq!(
        file["installation"]["settings"]["bbbb000000000002"]["env"].as_str(),
        Some("keepme"),
        "another installation's environment is not ours to change: {file}"
    );
    assert!(
        file.get("ui").and_then(|u| u.get("env")).is_none(),
        "{file}"
    );
    let _ = s.term.stop();
}

/// Waits until the terminal has written nothing for `quiet`.
fn settle(term: &Terminal, quiet: Duration) {
    let deadline = Instant::now() + STARTUP;
    let mut last = term.bytes_received();
    let mut since = Instant::now();
    while since.elapsed() < quiet {
        assert!(Instant::now() < deadline, "never quiet\n{}", term.text());
        std::thread::sleep(Duration::from_millis(50));
        let now = term.bytes_received();
        if now != last {
            last = now;
            since = Instant::now();
        }
    }
}

/// A job that outlives the key press that started it repaints the rows when
/// it ends, with no key pressed meanwhile (the worker's answer wakes the
/// loop). Another installation is the primary and its legacy-named launcher
/// is a FIFO: only the handover job reads it, and the read blocks until the
/// test opens the gate, after the screen went quiet. Resigning follows.
#[test]
fn a_slow_primary_job_repaints_the_rows_without_a_key_press() {
    use std::sync::atomic::{AtomicBool, Ordering};
    const OLD: &str = "bbbb000000000002";
    let other = tempfile::tempdir().unwrap();
    let config = format!(
        "{UI}logo_color = \"violet\"\n[installation]\nprimary = \"{OLD}\"\n\n[installation.settings.{OLD}]\npath = {:?}\nkind = \"dev\"\nname = \"GitPoulpi\"\n",
        other.path().display().to_string()
    );
    let fifo = std::cell::RefCell::new(PathBuf::new());
    let mut s = start("installation-primary-idle", &config, |home| {
        let apps = home.join("data/applications");
        std::fs::create_dir_all(&apps).unwrap();
        let path = apps.join("gitpoulpi.desktop");
        assert!(std::process::Command::new("mkfifo")
            .arg(&path)
            .status()
            .unwrap()
            .success());
        *fifo.borrow_mut() = path;
    });
    let path = fifo.into_inner();
    let gate = std::sync::Arc::new(AtomicBool::new(false));
    let open = gate.clone();
    let served = format!(
        "[Desktop Entry]\nType=Application\nName=GitPoulpi\nExec=/x\nX-GitPoulpi=1\nX-GitPoulpi-Install={OLD}\n"
    );
    let stop = std::sync::Arc::new(AtomicBool::new(false));
    let stopped = stop.clone();
    std::thread::spawn(move || {
        use std::io::Write;
        while !open.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(20));
        }
        // Opening for write waits for a reader (the Read at the page's open,
        // then the handover job), which then sees EOF. Never creates the file.
        while !stopped.load(Ordering::SeqCst) {
            match std::fs::OpenOptions::new().write(true).open(&path) {
                Ok(mut f) => {
                    let _ = f.write_all(served.as_bytes());
                }
                Err(_) => std::thread::sleep(Duration::from_millis(5)),
            }
        }
    });
    palette(
        &mut s.term,
        "settings: become the primary",
        "Settings: Become the primary...",
    );
    s.term
        .wait_for("Settings on the primary row", STARTUP, |t| {
            primary_row(&t.text(), "[ ]")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Space").unwrap();
    // The question: click the button to the right of Cancel.
    let screen = s
        .term
        .wait_for("the confirmation", STEP, |t| {
            t.contains("Take over the name") && t.contains("Cancel")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let (cancel_col, row) = screen.find("Cancel").expect("the Cancel button");
    let mut col = cancel_col + "Cancel".len() as u16;
    while screen
        .cell(col, row)
        .is_some_and(|c| c.text.trim().is_empty())
    {
        col += 1;
    }
    s.term.click(col + 1, row).unwrap();
    // The job is stuck on the FIFO. Once the screen is quiet, release it; no
    // key follows, only the worker's answer can change the screen.
    settle(&s.term, Duration::from_secs(2));
    assert!(primary_row(&s.term.text(), "[ ]"), "{}", s.term.text());
    gate.store(true, Ordering::SeqCst);
    s.term
        .wait_for(
            "the checkbox is checked by itself",
            Duration::from_millis(1500),
            |t| {
                primary_row(&t.text(), "[x]")
                    && t.contains("the primary is always called GitPoulpi")
            },
        )
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    stop.store(true, Ordering::SeqCst);
    // Resigning repaints the same way.
    settle(&s.term, Duration::from_secs(2));
    s.term.send_keys("Space").unwrap();
    s.term
        .wait_for(
            "the checkbox is cleared by itself",
            Duration::from_millis(1500),
            |t| primary_row(&t.text(), "[ ]"),
        )
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let _ = s.term.stop();
}
