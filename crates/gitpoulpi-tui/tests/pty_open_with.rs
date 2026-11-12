//! "Open in" for binary files, through a real pty. The desktop is fake: a
//! temp `XDG_DATA_DIRS` holds `mime/globs2` and `.desktop` files whose `Exec`
//! lines run a shell script that appends its arguments to a log, a temp
//! `XDG_CONFIG_HOME` holds `mimeapps.list`. No real application is started and
//! the real desktop configuration is never read. Temp config dir and fixture
//! repos only.
//!
//! * a modified binary file in the Diff view: the button, the list with the
//!   default first and marked (and `NoDisplay` left out), a pick launches the
//!   application on the working-tree file;
//! * a committed binary file in the File View: the keyboard twin, a read-only
//!   temp copy holding the blob, and a status notice;
//! * a type the OS knows no app for keeps the plain notice;
//! * the palette, and the file context menus of the Status pane and of the
//!   commit's file list;
//! * a picture's "Edit in" list also holds the OS's apps for its type.

// XDG only: macOS offers the default program through `open`, with no list of
// registered apps (its Open with page is covered by the config unit tests).
#![cfg(all(unix, not(target_os = "macos")))]

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

/// Binary for git (it holds a NUL), different at each revision.
fn blob(revision: u8) -> Vec<u8> {
    let mut bytes = b"\0GPDOC\0".to_vec();
    bytes.push(revision);
    bytes
}

fn png() -> Vec<u8> {
    let rgba: Vec<u8> = (0..16).flat_map(|_| [255u8, 0, 0, 255]).collect();
    gitpoulpi_testkit::png_bytes(4, 4, &rgba)
}

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
            "# fake shared-mime-info\n\
             50:application/x-gp-doc:*.gpdoc\n\
             50:application/x-gp-none:*.gpnone\n\
             50:image/png:*.png\n",
        )
        .unwrap();
        // One script per application: two entries running one executable are
        // one program as far as the Edit in list is concerned.
        let entry = |file: &str, name: &str, flag: &str, mime: &str, extra: &str| {
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
                     Exec={} {flag} %f\nMimeType={mime}\n{extra}",
                    script.display()
                ),
            )
            .unwrap();
        };
        entry(
            "alpha.desktop",
            "Alpha Reader",
            "--alpha",
            "application/x-gp-doc;",
            "",
        );
        entry(
            "zeta.desktop",
            "Zeta Viewer",
            "--zeta",
            "application/x-gp-doc;",
            "",
        );
        entry(
            "ghost.desktop",
            "Ghost Hidden",
            "--ghost",
            "application/x-gp-doc;",
            "NoDisplay=true\n",
        );
        entry(
            "painter.desktop",
            "GP Painter",
            "--painter",
            "image/png;",
            "",
        );
        // application/x-gp-none has no handler on purpose.
        let config = root.join("config");
        std::fs::create_dir_all(&config).unwrap();
        std::fs::write(
            config.join("mimeapps.list"),
            "[Default Applications]\napplication/x-gp-doc=zeta.desktop;\n",
        )
        .unwrap();
        Self { dir }
    }

    fn launches(&self) -> String {
        std::fs::read_to_string(self.dir.path().join("launches.log")).unwrap_or_default()
    }

    /// Waits for the log to hold a line and returns it.
    fn launched(&self) -> String {
        let deadline = Instant::now() + STEP;
        while self.launches().trim().is_empty() {
            assert!(Instant::now() < deadline, "no application ever ran");
            std::thread::sleep(Duration::from_millis(50));
        }
        self.launches().lines().next().unwrap().to_string()
    }
}

fn launch(
    repo: &Path,
    session: &str,
    desktop: &Desktop,
    config: &str,
) -> (Terminal, tempfile::TempDir) {
    let root = desktop.dir.path();
    common::launch(repo)
        .session(session)
        .config(format!("[ui]\ngraph_renderer = \"text\"\n{config}"))
        .size(160, 45)
        .no_exit_pause()
        .env("XDG_DATA_DIRS", root.join("share"))
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("XDG_CONFIG_DIRS", root.join("none"))
        .env("XDG_CURRENT_DESKTOP", "GPTest")
        .ready_when("the fixture's log", |s| s.contains("add docs"))
        .mouse_capture()
        .start()
}

/// HEAD changed only `note.txt`, so the binaries are "unchanged" and View all
/// files opens them in the File View. With `dirty`, the working tree holds a
/// different `doc.gpdoc`.
fn repo(dirty: bool) -> gitpoulpi_testkit::FixtureRepo {
    let b = FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("doc.gpdoc", &blob(1))
        .unwrap()
        .write_worktree("data.gpnone", &blob(1))
        .unwrap()
        .write_worktree("pic.png", &png())
        .unwrap()
        .write_worktree("note.txt", b"first\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("add docs")
        .unwrap()
        .commit_file("note.txt", "second\n", "touch the note")
        .unwrap();
    let b = if dirty {
        b.write_worktree("doc.gpdoc", &blob(2)).unwrap()
    } else {
        b
    };
    b.build()
}

fn open_file_view(term: &mut Terminal, name: &str) {
    if term.screen().find(name).is_none() {
        let (c, r) = term.wait_for_text("View all files", STEP).unwrap();
        term.click(c + 6, r).unwrap();
    }
    let (fc, fr) = term
        .wait_for_text(name, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(fc, fr).unwrap();
    term.wait_for("the File View", STEP, |s| {
        s.contains(&format!("File View - {name} @"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn click_text(term: &mut Terminal, text: &str) {
    let (col, row) = term
        .wait_for_text(text, STEP)
        .unwrap_or_else(|e| panic!("{text:?}: {e}\n{}", term.text()));
    term.click(col + 1, row).unwrap();
}

fn palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

/// The binary view's "Open in" button, not the tab bar's one on the top rows.
fn button(s: &xtask::pty::Screen) -> Option<(u16, u16)> {
    s.find_all("Open in").into_iter().find(|&(_, row)| row > 3)
}

/// The popup's rows, top to bottom.
fn wait_list(term: &Terminal) {
    term.wait_for("the Open in list", STEP, |s| {
        s.contains("Zeta Viewer") && s.contains("Alpha Reader")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn same_file(a: &Path, b: &Path) -> bool {
    std::fs::canonicalize(a).unwrap() == std::fs::canonicalize(b).unwrap()
}

/// The argument the launch script got after its flag.
fn path_of(line: &str, flag: &str) -> PathBuf {
    PathBuf::from(
        line.strip_prefix(&format!("{flag} "))
            .unwrap_or_else(|| panic!("{line:?} does not start with {flag}"))
            .trim(),
    )
}

/// A modified binary file in the Diff view: the button, the default first and
/// marked, the hidden app left out; a pick runs the app on the working-tree
/// file itself.
#[test]
fn a_modified_binary_in_the_diff_view_offers_the_registered_apps() {
    let desktop = Desktop::new();
    let repo = repo(true);
    let (mut term, _config) = launch(repo.root(), "pty-open-with-diff", &desktop, "");
    let (col, row) = term.wait_for_text("\u{25CC}", STEP).unwrap();
    term.click(col, row).unwrap();
    term.wait_for_text("\u{270E} doc.gpdoc", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_text(&mut term, "\u{270E} doc.gpdoc");
    let screen = term
        .wait_for("the binary row", STEP, |s| {
            button(s).is_some() && s.contains("no textual changes")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = button(&screen).unwrap();
    term.click(c + 1, r).unwrap();
    wait_list(&term);
    let screen = term.screen();
    assert!(!screen.contains("Ghost Hidden"), "{}", term.text());
    // The default is first and marked; the other follows.
    let (_, zr) = screen.find("\u{2022} Zeta Viewer").unwrap_or_else(|| {
        panic!("the default is marked\n{}", term.text());
    });
    let (_, ar) = screen.find("Alpha Reader").unwrap();
    assert!(zr < ar, "default first");
    assert_eq!(screen.find("Zeta Viewer").map(|p| p.1), Some(zr));
    click_text(&mut term, "Alpha Reader");
    let line = desktop.launched();
    assert!(
        same_file(&path_of(&line, "--alpha"), &repo.root().join("doc.gpdoc")),
        "{line}"
    );
    term.wait_for("the popup closed", STEP, |s| !s.contains("Zeta Viewer"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A committed binary in the File View: the keyboard opens the list, and the
/// app gets a read-only temp copy holding the blob.
#[test]
fn a_committed_binary_is_handed_over_as_a_read_only_copy() {
    let desktop = Desktop::new();
    let repo = repo(false);
    let (mut term, _config) = launch(repo.root(), "pty-open-with-commit", &desktop, "");
    open_file_view(&mut term, "doc.gpdoc");
    term.wait_for("the Open in button and the old notice", STEP, |s| {
        button(s).is_some() && s.contains("binary file (8 bytes)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("S-o").unwrap();
    wait_list(&term);
    click_text(&mut term, "Zeta Viewer");
    let line = desktop.launched();
    let copy = path_of(&line, "--zeta");
    assert!(
        copy.to_string_lossy().contains("gitpoulpi-image-"),
        "a temp copy, not the repository: {copy:?}"
    );
    assert!(!copy.starts_with(repo.root()));
    assert_eq!(std::fs::read(&copy).unwrap(), blob(1), "the blob's bytes");
    assert!(std::fs::metadata(&copy).unwrap().permissions().readonly());
    term.wait_for("the copy notice", STEP, |s| {
        s.contains("opened a copy of doc.gpdoc")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A type the OS knows no app for keeps today's notice and no button.
#[test]
fn an_unknown_type_keeps_the_plain_binary_notice() {
    let desktop = Desktop::new();
    let repo = repo(false);
    let (mut term, _config) = launch(repo.root(), "pty-open-with-none", &desktop, "");
    open_file_view(&mut term, "data.gpnone");
    term.wait_for("the plain notice", STEP, |s| {
        s.contains("binary file (8 bytes) - nothing to show")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Let any lookup finish: still no button.
    term.wait_for_stable(Duration::from_millis(600), STEP)
        .unwrap();
    assert!(button(&term.screen()).is_none(), "{}", term.text());
    // The palette says why.
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Open with").unwrap();
    term.wait_for("the entry is listed but unavailable", STEP, |s| {
        s.contains("File: Open With")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for_stable(Duration::from_millis(400), STEP)
        .unwrap();
    assert!(desktop.launches().is_empty());
    term.stop().unwrap();
}

/// The palette's twin: "Open in Default app" runs the default, "Open with..."
/// opens the list.
#[test]
fn the_palette_runs_the_default_app_and_opens_the_list() {
    let desktop = Desktop::new();
    let repo = repo(false);
    let (mut term, _config) = launch(repo.root(), "pty-open-with-palette", &desktop, "");
    open_file_view(&mut term, "doc.gpdoc");
    term.wait_for("the Open in button", STEP, |s| button(s).is_some())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    palette(
        &mut term,
        "File: Open in Default App",
        "File: Open in Default App",
    );
    let line = desktop.launched();
    assert!(line.starts_with("--zeta "), "the default app: {line}");

    palette(&mut term, "File: Open with", "File: Open With...");
    wait_list(&term);
    term.send_keys("Escape").unwrap();
    term.wait_for("the popup closed", STEP, |s| !s.contains("Alpha Reader"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The file menu of the Status pane (a working-tree file) and of the commit's
/// file list (a commit's version) both offer "Open with...".
#[test]
fn the_file_context_menus_offer_open_with() {
    let desktop = Desktop::new();
    let repo = repo(true);
    let (mut term, _config) = launch(repo.root(), "pty-open-with-menus", &desktop, "");
    let (col, row) = term.wait_for_text("\u{25CC}", STEP).unwrap();
    term.click(col, row).unwrap();
    term.wait_for_text("\u{270E} doc.gpdoc", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    common::right_click_text_until(&mut term, "\u{270E} doc.gpdoc", "the file menu", |s| {
        s.contains("Open with...")
    });
    click_text(&mut term, "Open with...");
    wait_list(&term);
    click_text(&mut term, "Alpha Reader");
    let line = desktop.launched();
    assert!(
        same_file(&path_of(&line, "--alpha"), &repo.root().join("doc.gpdoc")),
        "{line}"
    );
    term.stop().unwrap();

    // The commit's file list: a copy of the commit's version.
    let desktop = Desktop::new();
    let repo = repo_clean();
    let (mut term, _config) = launch(repo.root(), "pty-open-with-menus2", &desktop, "");
    let (c, r) = term.wait_for_text("View all files", STEP).unwrap();
    term.click(c + 6, r).unwrap();
    term.wait_for_text("doc.gpdoc", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    common::right_click_text_until(&mut term, "note.txt", "a text file's menu", |s| {
        s.contains("Copy file path")
    });
    assert!(
        !term.screen().contains("Open with..."),
        "no row on plain text\n{}",
        term.text()
    );
    term.send_keys("Escape").unwrap();
    common::right_click_text_until(&mut term, "doc.gpdoc", "the binary's menu", |s| {
        s.contains("Open with...")
    });
    click_text(&mut term, "Open with...");
    wait_list(&term);
    click_text(&mut term, "Zeta Viewer");
    let line = desktop.launched();
    let copy = path_of(&line, "--zeta");
    assert_eq!(std::fs::read(&copy).unwrap(), blob(1));
    assert!(!copy.starts_with(repo.root()));
    term.stop().unwrap();
}

/// The program list opened from a menu row is a continuation of that menu:
/// drawn to its right on the row's line (never at the screen's corner), from
/// a mouse click and from the keyboard alike.
#[test]
fn the_list_hangs_beside_the_menu_it_was_opened_from() {
    for keyboard in [false, true] {
        let desktop = Desktop::new();
        let repo = repo(true);
        let session = format!("pty-open-with-flyout-{keyboard}");
        let (mut term, _config) = launch(repo.root(), &session, &desktop, "");
        let (col, row) = term.wait_for_text("\u{25CC}", STEP).unwrap();
        term.click(col, row).unwrap();
        term.wait_for_text("\u{270E} doc.gpdoc", STEP)
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        common::right_click_text_until(&mut term, "\u{270E} doc.gpdoc", "the file menu", |s| {
            s.contains("Open with...")
        });
        let screen = term.screen();
        let (menu_col, menu_row) = screen.find("Open with...").unwrap();
        if keyboard {
            // Step down until the entry's row carries the selection band
            // (separators and greyed rows are skipped by the menu).
            let band = |s: &xtask::pty::Screen| s.cell(menu_col + 1, menu_row).unwrap().bg;
            let plain = band(&screen);
            for _ in 0..30 {
                if band(&term.screen()) != plain {
                    break;
                }
                term.send_keys("Down").unwrap();
                term.wait_for_stable(Duration::from_millis(80), STEP)
                    .unwrap();
            }
            assert_ne!(band(&term.screen()), plain, "{}", term.text());
            term.send_keys("Enter").unwrap();
        } else {
            term.click(menu_col + 1, menu_row).unwrap();
        }
        wait_list(&term);
        let screen = term.screen();
        let (list_col, list_row) = screen.find("Zeta Viewer").unwrap();
        // Beside the menu, to its right, on or below the activated row.
        assert!(
            list_col > menu_col + "Open with...".len() as u16 && list_row >= menu_row,
            "keyboard={keyboard}: menu at ({menu_col},{menu_row}), list at ({list_col},{list_row})\n{}",
            term.text()
        );
        term.send_keys("Escape").unwrap();
        term.stop().unwrap();
    }
}

fn repo_clean() -> gitpoulpi_testkit::FixtureRepo {
    repo(false)
}

/// A picture's "Edit in" list also holds the OS's apps for its type, after the
/// configured editors.
#[test]
fn a_pictures_edit_in_list_gets_the_os_apps_after_the_configured_ones() {
    let desktop = Desktop::new();
    let repo = repo(false);
    let editors = "image_editor_default = \"Mine\"\n\
         [[ui.image_editors]]\nlabel = \"Mine\"\ncommand = \"true ${file}\"\n";
    let (mut term, _config) = launch(repo.root(), "pty-open-with-image", &desktop, editors);
    open_file_view(&mut term, "pic.png");
    click_text(&mut term, "Edit in");
    term.wait_for("the Edit in list with the OS app", STEP, |s| {
        s.contains("Mine") && s.contains("GP Painter") && s.contains("Configure")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    let (_, mine) = screen.find("Mine").unwrap();
    let (_, painter) = screen.find("GP Painter").unwrap();
    assert!(mine < painter, "configured first");
    click_text(&mut term, "GP Painter");
    let line = desktop.launched();
    let copy = path_of(&line, "--painter");
    assert_eq!(std::fs::read(&copy).unwrap(), png());
    term.stop().unwrap();
}

/// Linux "Open file in default program" on a type whose default is a
/// terminal program (Omarchy: text is `nvim.desktop`, `Terminal=true`): it
/// runs in `$TERMINAL`, not through `xdg-open`, which would start it in the
/// TUI's own terminal where nothing shows.
#[test]
fn a_terminal_default_program_opens_in_terminal() {
    let desktop = Desktop::new();
    let root = desktop.dir.path();
    std::fs::write(
        root.join("share/mime/globs2"),
        "50:text/x-gp-note:*.gpnote\n",
    )
    .unwrap();
    let log = root.join("launches.log");
    let script = |name: &str, body: &str| {
        let path = root.join(name);
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    };
    let editor = script("termed.sh", "exit 0");
    // Named like Omarchy's launcher, which takes the command without -e.
    std::fs::create_dir_all(root.join("bin")).unwrap();
    let terminal = script(
        "bin/xdg-terminal-exec",
        &format!("echo \"TERM $*\" >> '{}'", log.display()),
    );
    std::fs::write(
        root.join("share/applications/termed.desktop"),
        format!(
            "[Desktop Entry]\nType=Application\nName=Term Editor\nExec={} --term %F\n\
             Terminal=true\nMimeType=text/x-gp-note;\n",
            editor.display()
        ),
    )
    .unwrap();
    std::fs::write(
        root.join("config/mimeapps.list"),
        "[Default Applications]\ntext/x-gp-note=termed.desktop;\n",
    )
    .unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("notes.gpnote", "one\n", "add docs")
        .unwrap()
        .write_worktree("notes.gpnote", b"one\ntwo\n")
        .unwrap()
        .build();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-open-default-terminal")
        .config("[ui]\ngraph_renderer = \"text\"\n")
        .size(160, 45)
        .no_exit_pause()
        .env("XDG_DATA_DIRS", root.join("share"))
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("XDG_CONFIG_DIRS", root.join("none"))
        .env("XDG_CURRENT_DESKTOP", "GPTest")
        .env("TERMINAL", &terminal)
        .ready_when("the changed note", |s| {
            s.contains("Unstaged") && s.contains("notes.gpnote")
        })
        .mouse_capture()
        .start();
    common::right_click_text_until(&mut term, "notes.gpnote", "the file menu", |s| {
        s.contains("Open file in default program")
    });
    click_text(&mut term, "Open file in default program");
    let line = desktop.launched();
    assert!(
        line.starts_with(&format!("TERM {} --term ", editor.display()))
            && line.ends_with("notes.gpnote"),
        "{line}"
    );
    term.stop().unwrap();
}
