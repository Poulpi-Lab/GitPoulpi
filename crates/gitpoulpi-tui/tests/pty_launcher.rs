//! Settings > Launcher (unix: the XDG `.desktop` entry / macOS bundle; Windows: the
//! Start menu `.lnk`, last test): the palette's `Launcher: Create Shortcut`
//! writes the `.desktop` entry and its icon into a temp `XDG_DATA_HOME`, the
//! row then reads "created", and its Remove button deletes both again.
//!
//! HOME, XDG_DATA_HOME and XDG_CONFIG_HOME point at a scratch directory, so
//! nothing here can reach the real launcher entries. No terminal is started:
//! the entry is only written.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const MESSAGE: &str = "launcher probe commit";
/// The launcher row's label: macOS has one place, the Applications folder.
const PLACE: &str = if cfg!(target_os = "macos") {
    "Applications folder"
} else if cfg!(windows) {
    "Start menu"
} else {
    "App launcher"
};

/// The launcher row's value (the alias row above it also says
/// "not created", so the whole screen cannot be searched).
fn launcher_row_is(screen: &xtask::pty::Screen, value: &str) -> bool {
    // The label column and the value column are separate now: the row reads
    // `App launcher    created`, and `Remove: App launcher` is another row.
    screen.text().lines().any(|l| {
        !l.contains("Remove:")
            && l.find(PLACE).is_some_and(|i| {
                l[i + PLACE.len()..].trim_matches(|c: char| c == ' ' || c == '\u{2502}') == value
            })
    })
}

fn click_text(term: &mut Terminal, text: &str) {
    let (col, row) = snap(term)
        .find_all(text)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{text:?} not on screen:\n{}", term.text()));
    term.click(col + 1, row).unwrap();
}

/// The `.desktop` entry is the XDG launcher; macOS writes an app bundle
/// instead (covered by the next test), and its terminals are not found on
/// PATH.
#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn changing_the_terminal_in_the_settings_rewrites_a_created_shortcut_by_itself() {
    use std::os::unix::fs::PermissionsExt;
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let data = home.path().join("data");
    // Two fake terminals the detection finds on PATH; neither is ever started.
    let bin = home.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    for name in ["kitty", "foot"] {
        let path = bin.join(name);
        std::fs::write(&path, "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\nlaunch_terminal = \"kitty\"\n",
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .session("launcher-autoupdate")
        .config_dir(config.path())
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .env("HOME", home.path())
        .env("PATH", &path)
        .env("XDG_DATA_HOME", &data)
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .size(200, 90)
        .no_exit_pause()
        .ready(MESSAGE)
        .spawn();
    let entry = data.join("applications/gitpoulpi.desktop");

    term.send_keys("C-p").unwrap();
    term.send_text("launcher create").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Launcher: Create Shortcut")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the row reads created", STARTUP, |s| {
        s.contains(&format!("Remove: {PLACE}")) && launcher_row_is(s, "created")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let text = std::fs::read_to_string(&entry).expect("the .desktop entry was written");
    assert!(
        text.contains(" launch -- kitty --class gitpoulpi "),
        "{text}"
    );
    assert!(
        !term.text().contains("Update"),
        "no Update button:\n{}",
        term.text()
    );

    // Pick foot in the terminal combobox: no other press.
    let (vc, vr) = common::choice_at(&snap(&term), "kitty")
        .unwrap_or_else(|| panic!("no terminal value:\n{}", term.text()));
    term.click(vc + 1, vr).unwrap();
    let popup = term
        .wait_for("the terminal list", STEP, |s| s.contains("\u{2713} kitty"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (tc, tr) = popup.find("\u{2713} kitty").unwrap();
    term.click(tc + 2, tr + 1).unwrap(); // foot, the row under kitty

    let deadline = std::time::Instant::now() + STARTUP;
    loop {
        let text = std::fs::read_to_string(&entry).unwrap_or_default();
        if text.contains(" launch -- foot --app-id=gitpoulpi ") {
            assert!(text.contains("\nX-GitPoulpi=1\n"), "{text}");
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Exec= never became foot:\n{text}\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    let _ = term.stop();
}

#[cfg(unix)]
#[test]
fn create_from_the_palette_then_remove_from_the_settings_row() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let data = home.path().join("data");
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\nlaunch_terminal = \"kitty\"\n",
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .session("launcher-shortcut")
        .config_dir(config.path())
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .env("HOME", home.path())
        .env("XDG_DATA_HOME", &data)
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .size(200, 90)
        .no_exit_pause()
        .ready(MESSAGE)
        .spawn();
    // XDG: a `.desktop` entry and its icon; macOS: an app bundle in
    // `~/Applications` (HOME is the scratch directory).
    let mac = cfg!(target_os = "macos");
    let bundle = home.path().join("Applications/GitPoulpi.app");
    let entry = if mac {
        bundle.join("Contents/MacOS/GitPoulpi")
    } else {
        data.join("applications/gitpoulpi.desktop")
    };
    let icons = data.join("icons/hicolor/256x256/apps");
    let our_icons = || {
        if mac {
            return usize::from(bundle.join("Contents/Resources/GitPoulpi.icns").is_file());
        }
        std::fs::read_dir(&icons)
            .map(|d| {
                d.flatten()
                    .filter(|e| e.file_name().to_string_lossy().starts_with("gitpoulpi"))
                    .count()
            })
            .unwrap_or(0)
    };
    assert!(!entry.exists());

    // The palette twin: opens Settings on the Launcher group and creates.
    term.send_keys("C-p").unwrap();
    term.send_text("launcher create").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Launcher: Create Shortcut")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the row reads created", STARTUP, |s| {
        s.contains(&format!("Remove: {PLACE}")) && launcher_row_is(s, "created")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let text = std::fs::read_to_string(&entry).expect("the .desktop entry was written");
    if mac {
        assert!(text.contains("open -na kitty --args"), "{text}");
        assert!(
            std::fs::read_to_string(bundle.join("Contents/Info.plist"))
                .unwrap()
                .contains("X-GitPoulpi"),
            "the bundle carries our marker"
        );
        assert_eq!(our_icons(), 1, "the icon was written");
    } else {
        assert!(
            text.contains(" launch -- kitty --class gitpoulpi "),
            "{text}"
        );
        assert!(text.contains("\nX-GitPoulpi=1\n"), "{text}");
        let name = text.lines().find_map(|l| l.strip_prefix("Icon=")).unwrap();
        assert!(
            icons.join(format!("{name}.png")).is_file(),
            "the icon was written"
        );
    }

    // The row's Remove button deletes only what we made. The palette left it
    // selected (and scrolled into view), so one click runs it.
    click_text(&mut term, &format!("Remove: {PLACE}"));
    // The confirm click is retried while the question is up: a click aimed at
    // a torn frame lands on nothing.
    common::retry_until(
        &mut term,
        "the row reads not created",
        4,
        STEP,
        |t| {
            if t.screen().contains("This cannot") {
                common::confirm_removal(t, STEP);
            }
        },
        |s| launcher_row_is(s, "not created"),
    );
    assert!(!entry.exists(), "the entry is gone");
    assert!(!mac || !bundle.exists(), "the whole bundle is gone");
    assert_eq!(our_icons(), 0, "the icon is gone");
    let _ = term.stop();
}

/// The Windows launcher: a `GitPoulpi.lnk` in the Start menu (a scratch
/// `%APPDATA%`), made from the palette and removed from the settings row.
#[cfg(windows)]
#[test]
fn windows_create_from_the_palette_then_remove_from_the_settings_row() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\n",
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .session("launcher-shortcut-windows")
        .config_dir(config.path())
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .scratch_home(home.path())
        .size(200, 90)
        .no_exit_pause()
        .ready(MESSAGE)
        .spawn();
    let entry = home
        .path()
        .join("appdata/Microsoft/Windows/Start Menu/Programs/GitPoulpi.lnk");
    assert!(!entry.exists());

    term.send_keys("C-p").unwrap();
    term.send_text("launcher create").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Launcher: Create Shortcut")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the row reads created", STARTUP, |s| {
        s.contains(&format!("Remove: {PLACE}")) && launcher_row_is(s, "created")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(entry.is_file(), "the Start menu shortcut was written");

    click_text(&mut term, &format!("Remove: {PLACE}"));
    common::retry_until(
        &mut term,
        "the row reads not created",
        4,
        STEP,
        |t| {
            if t.screen().contains("This cannot") {
                common::confirm_removal(t, STEP);
            }
        },
        |s| launcher_row_is(s, "not created"),
    );
    assert!(!entry.exists(), "the shortcut is gone");
    let _ = term.stop();
}

/// The screen once it has settled: a torn frame is not a place to locate a
/// cell or compare colors in.
fn snap(term: &xtask::pty::Terminal) -> xtask::pty::Screen {
    term.wait_for_stable(
        std::time::Duration::from_millis(300),
        std::time::Duration::from_secs(30),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}
