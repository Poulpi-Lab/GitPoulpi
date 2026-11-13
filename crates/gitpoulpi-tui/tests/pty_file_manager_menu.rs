//! Settings > Launcher > File manager menu: the palette's `Launcher: Install
//! Open in GitPoulpi Menu` writes the right-click entries, the rows then read
//! "created", and the row's Remove button (keyboard only) takes out exactly
//! what Install wrote.
//!
//! Linux: HOME, XDG_DATA_HOME and XDG_CONFIG_HOME point at a scratch
//! directory, the file managers are fake programs on PATH, and a user's own
//! Thunar action must survive byte for byte. Windows: the registry is the
//! file named by `GITPOULPI_TEST_REGISTRY_FILE`, so the real one is never
//! touched. Nothing is ever started from an entry.
//!
//! The TUI runs as a copy in a scratch install folder with a dummy launch
//! helper beside it (`gitpoulpi`, or `gitpoulpi-launch.exe`), so the helper
//! the entries point at is found whatever the build folder holds.

mod common;

use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const MESSAGE: &str = "folder menu probe commit";

/// The row labelled `label` reads exactly `value` in its value column.
fn row_is(screen: &xtask::pty::Screen, label: &str, value: &str) -> bool {
    screen.text().lines().any(|l| {
        l.find(label).is_some_and(|i| {
            l[i + label.len()..].trim_matches(|c: char| c == ' ' || c == '\u{2502}') == value
        })
    })
}

/// A copy of the TUI in `install`, with `helper` (a dummy) beside it.
fn install_copy(install: &Path, helper: &str) -> PathBuf {
    let built = Path::new(env!("CARGO_BIN_EXE_gitpoulpi-tui"));
    let exe = install.join(built.file_name().unwrap());
    if std::fs::hard_link(built, &exe).is_err() {
        std::fs::copy(built, &exe).unwrap();
    }
    let dummy = install.join(helper);
    std::fs::write(&dummy, "#!/bin/sh\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dummy, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    exe
}

/// Runs the palette's Install entry: the query ranks it first (above its
/// Remove twin, when that is listed too), and Enter runs the first row.
fn install_from_the_palette(term: &mut Terminal) {
    term.send_keys("C-p").unwrap();
    term.send_text("install open in gitpoulpi").unwrap();
    term.wait_for("the Install entry first", STEP, |s| {
        let install = s.find("Launcher: Install Open in GitPoulpi Menu");
        let remove = s.find("Launcher: Remove Open in GitPoulpi Menu");
        match (install, remove) {
            (Some((_, a)), Some((_, b))) => a < b,
            (Some(_), None) => true,
            _ => false,
        }
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

/// The palette left the buttons row selected: Right reaches Remove, Enter
/// presses it, and in the question Right then Enter confirms (Cancel is
/// focused first). Retried while the screen does not move: a key sent into a
/// torn frame is still handled, but a lost one would hang the wait.
fn remove_with_the_keyboard(term: &mut Terminal, done: impl Fn(&xtask::pty::Screen) -> bool) {
    term.send_keys("Right").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the removal question", STEP, |s| {
        s.contains("This cannot") && s.contains("Cancel")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Right").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the rows read not created", STARTUP, |s| {
        !s.contains("This cannot") && done(s)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn install_from_the_palette_then_remove_with_the_keyboard_restores_thunar_byte_for_byte() {
    use std::os::unix::fs::PermissionsExt;
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let data = home.path().join("data");
    let xdg_config = home.path().join("config");
    // Three file managers the detection finds on PATH; none is ever started.
    let bin = home.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    for name in ["dolphin", "thunar", "nautilus"] {
        let path = bin.join(name);
        std::fs::write(&path, "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    // The user's own Thunar action, which must come back unchanged.
    let uca = xdg_config.join("Thunar/uca.xml");
    std::fs::create_dir_all(uca.parent().unwrap()).unwrap();
    let seed = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<actions>\n<action>\n\t<icon>utilities-terminal</icon>\n\t<name>Open Terminal Here</name>\n\t<unique-id>1-1</unique-id>\n\t<command>exo-open --working-directory %f --launch TerminalEmulator</command>\n\t<description>Example for a custom action</description>\n\t<patterns>*</patterns>\n\t<startup-notify/>\n\t<directories/>\n</action>\n</actions>\n";
    std::fs::write(&uca, seed).unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\n",
    )
    .unwrap();
    let install = tempfile::tempdir().unwrap();
    let exe = install_copy(install.path(), "gitpoulpi");
    let mut term = common::launch(repo.root())
        .session("folder-menu-linux")
        .config_dir(config.path())
        .binary(&exe)
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .env("HOME", home.path())
        .env("PATH", &path)
        .env("XDG_DATA_HOME", &data)
        .env("XDG_CONFIG_HOME", &xdg_config)
        .size(200, 120)
        .no_exit_pause()
        .ready(MESSAGE)
        .spawn();
    let dolphin = data.join("kio/servicemenus/gitpoulpi-open.desktop");
    let scripts = data.join("nautilus/scripts");
    let script = scripts.join("Open in GitPoulpi");
    let icon = data.join("icons/hicolor/256x256/apps/gitpoulpi-folder.png");

    install_from_the_palette(&mut term);
    let all = |s: &xtask::pty::Screen, value: &str| {
        row_is(s, "Dolphin", value) && row_is(s, "Thunar", value) && row_is(s, "GNOME Files", value)
    };
    term.wait_for("the rows read created", STARTUP, |s| all(s, "created"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let text = std::fs::read_to_string(&dolphin).expect("the Dolphin service menu");
    let helper = install.path().join("gitpoulpi");
    assert!(
        text.contains(&format!("Exec={} launch --folder %f", helper.display())),
        "{text}"
    );
    assert_eq!(
        std::fs::metadata(&dolphin).unwrap().permissions().mode() & 0o777,
        0o755,
        "KF6 ignores a service menu that is not executable"
    );
    let sh = std::fs::read_to_string(&script).expect("the GNOME Files script");
    assert!(sh.contains("launch --folder"), "{sh}");
    let merged = std::fs::read_to_string(&uca).unwrap();
    assert!(merged.contains("<unique-id>1-1</unique-id>"), "{merged}");
    assert!(
        merged.contains("<unique-id>gitpoulpi-folder-menu</unique-id>"),
        "{merged}"
    );
    assert!(icon.is_file(), "the folder-menu icon was written");

    remove_with_the_keyboard(&mut term, |s| all(s, "not created"));
    assert!(!dolphin.exists(), "the Dolphin service menu is gone");
    assert!(!script.exists(), "the GNOME Files script is gone");
    assert_eq!(
        std::fs::read_to_string(&uca).unwrap(),
        seed,
        "uca.xml is the user's file again, byte for byte"
    );
    assert!(!icon.exists(), "the icon went with the last entry");
    let _ = term.stop();
}

/// Windows: the Explorer verbs and App Paths `gitpoulpi.exe`, in a file that
/// stands in for `HKCU`.
#[cfg(windows)]
#[test]
fn windows_install_from_the_palette_then_remove_with_the_keyboard() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let registry = home.path().join("reg.txt");
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\n",
    )
    .unwrap();
    let install = tempfile::tempdir().unwrap();
    let exe = install_copy(install.path(), "gitpoulpi-launch.exe");
    let mut term = common::launch(repo.root())
        .session("folder-menu-windows")
        .config_dir(config.path())
        .binary(&exe)
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .env("GITPOULPI_TEST_REGISTRY_FILE", &registry)
        .scratch_home(home.path())
        .size(200, 120)
        .no_exit_pause()
        .ready(MESSAGE)
        .spawn();

    install_from_the_palette(&mut term);
    term.wait_for("the row reads created", STARTUP, |s| {
        row_is(s, "Open in GitPoulpi", "created")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let text = std::fs::read_to_string(&registry).expect("the registry file");
    assert!(
        text.lines().any(|l| l
            .starts_with("Software\\Classes\\Directory\\shell\\GitPoulpi\\command\t")
            && l.ends_with("--folder \"%V\"")),
        "{text}"
    );
    assert!(
        text.contains("App Paths\\gitpoulpi.exe\t"),
        "App Paths gitpoulpi.exe comes with Install:\n{text}"
    );

    remove_with_the_keyboard(&mut term, |s| row_is(s, "Open in GitPoulpi", "not created"));
    let left = std::fs::read_to_string(&registry).unwrap_or_default();
    assert!(
        !left.contains("GitPoulpi") && !left.contains("gitpoulpi.exe"),
        "Remove took everything out:\n{left}"
    );
    let _ = term.stop();
}
