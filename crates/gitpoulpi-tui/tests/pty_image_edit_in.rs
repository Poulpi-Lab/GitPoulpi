//! The image view's "Edit in" combobox, through a real pty (image view slice
//! S7). The editors are fake scripts appending their arguments to a log: no
//! real editor or GUI is ever started. Temp config dir and fixture repos only.
//!
//! * a working-tree picture, from the Status pane's file menu: the editor gets
//!   the worktree path itself;
//! * a committed picture, from the File View header: the editor gets a temp
//!   copy whose bytes equal the blob, read-only, with a status notice;
//! * the popup is modal: a click away closes it and reaches nothing under it;
//! * a failing editor reports its error on the status line;
//! * the palette: "Image: Edit in default" runs the default, "Image: Edit
//!   in..." opens the list, "Configure..." opens Settings.

#![cfg(unix)]

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];

fn png(color: [u8; 4]) -> Vec<u8> {
    let rgba: Vec<u8> = (0..16).flat_map(|_| color).collect();
    gitpoulpi_testkit::png_bytes(4, 4, &rgba)
}

struct Work {
    dir: tempfile::TempDir,
}

impl Work {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let script = |name: &str, body: &str| {
            let path = dir.path().join(name);
            std::fs::write(&path, body).unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        };
        script(
            "ok.sh",
            &format!(
                "#!/bin/sh\necho \"$@\" >> '{}'\n",
                dir.path().join("launches.log").display()
            ),
        );
        script(
            "fail.sh",
            "#!/bin/sh\necho boom-from-the-editor >&2\nexit 2\n",
        );
        Self { dir }
    }

    fn config(&self) -> String {
        let ok = self.dir.path().join("ok.sh");
        let fail = self.dir.path().join("fail.sh");
        format!(
            "[ui]\ngraph_renderer = \"text\"\nimage_editor_default = \"Fake editor\"\n\
             [[ui.image_editors]]\nlabel = \"Fake editor\"\ncommand = \"{} --edit ${{file}}\"\n\
             [[ui.image_editors]]\nlabel = \"Failing editor\"\ncommand = \"{} ${{file}}\"\n",
            ok.display(),
            fail.display()
        )
    }

    fn launches(&self) -> String {
        std::fs::read_to_string(self.dir.path().join("launches.log")).unwrap_or_default()
    }

    /// Waits for the log to hold a line, and returns the file the editor got.
    fn launched(&self) -> PathBuf {
        let deadline = Instant::now() + STEP;
        while self.launches().trim().is_empty() {
            assert!(Instant::now() < deadline, "the editor never ran");
            std::thread::sleep(Duration::from_millis(50));
        }
        let log = self.launches();
        let line = log.lines().next().unwrap();
        PathBuf::from(line.strip_prefix("--edit ").unwrap_or(line).trim())
    }
}

fn launch(repo: &Path, session: &str, work: &Work) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .config(work.config())
        .size(160, 45)
        .no_exit_pause()
        .ready_when("the fixture's log", |s| s.contains("add pictures"))
        .mouse_capture()
        .start()
}

/// HEAD changed only `note.txt`, so `pic.png` is "unchanged" and View all
/// files opens it in the File View. The working tree then holds a different
/// `wip.png` edit of it (blue).
fn repo(dirty: bool) -> gitpoulpi_testkit::FixtureRepo {
    let b = FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("pic.png", &png(RED))
        .unwrap()
        .write_worktree("note.txt", b"first\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("add pictures")
        .unwrap()
        .commit_file("note.txt", "second\n", "touch the note")
        .unwrap();
    let b = if dirty {
        b.write_worktree("pic.png", &png(BLUE)).unwrap()
    } else {
        b
    };
    b.build()
}

fn open_file_view(term: &mut Terminal, name: &str) {
    if snap(&term).find(name).is_none() {
        let (c, r) = term.wait_for_text("View all files", STEP).unwrap();
        term.click(c + 6, r).unwrap();
    }
    let (fc, fr) = term
        .wait_for_text(name, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(fc, fr).unwrap();
    term.wait_for("the File View", STEP, |s| {
        s.contains(&format!("File View - {name} @")) && s.contains("Edit in")
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

fn wait_popup(term: &Terminal) {
    term.wait_for("the Edit in list", STEP, |s| {
        s.contains("Fake editor") && s.contains("Failing editor") && s.contains("Configure")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn same_file(a: &Path, b: &Path) -> bool {
    std::fs::canonicalize(a).unwrap() == std::fs::canonicalize(b).unwrap()
}

/// A modified picture in the Status pane: its file menu has "Edit in...", the
/// popup lists the editors, and picking one hands the editor the file in the
/// working tree (it saves in place).
#[test]
fn a_working_tree_picture_is_edited_in_place_from_the_file_menu() {
    let work = Work::new();
    let repo = repo(true);
    let (mut term, _config) = launch(repo.root(), "pty-image-edit-wip", &work);
    let (col, row) = term.wait_for_text("\u{25CC}", STEP).unwrap();
    term.click(col, row).unwrap();
    term.wait_for_text("\u{270E} pic.png", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    common::right_click_text_until(&mut term, "\u{270E} pic.png", "the file menu", |s| {
        s.contains("Edit in...")
    });
    click_text(&mut term, "Edit in...");
    wait_popup(&term);
    // The default is marked.
    assert!(
        snap(&term).contains("\u{2022} Fake editor"),
        "{}",
        term.text()
    );
    click_text(&mut term, "Fake editor");
    let got = work.launched();
    assert!(same_file(&got, &repo.root().join("pic.png")), "{got:?}");
    assert!(!snap(&term).contains("Failing editor"), "{}", term.text());
    term.stop().unwrap();
}

/// A committed picture in the File View: the header button lists the editors;
/// the editor gets a read-only temp copy holding the blob, and the status line
/// says so. Clicking away closes the list and reaches nothing under it.
#[test]
fn a_committed_picture_is_handed_over_as_a_read_only_copy() {
    let work = Work::new();
    let repo = repo(false);
    let (mut term, _config) = launch(repo.root(), "pty-image-edit-commit", &work);
    open_file_view(&mut term, "pic.png");
    click_text(&mut term, "Edit in");
    wait_popup(&term);

    // Modal: a click on another file of the list (which would open it) only
    // closes the list; the File View stays on the picture and no editor ran.
    click_text(&mut term, "note.txt");
    term.wait_for("the list closed", STEP, |s| !s.contains("Failing editor"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        snap(&term).contains("File View - pic.png @"),
        "{}",
        term.text()
    );
    assert!(work.launches().is_empty());

    // The keyboard twin opens it again.
    term.send_keys("S-o").unwrap();
    wait_popup(&term);
    click_text(&mut term, "Fake editor");
    let got = work.launched();
    assert!(
        got.to_string_lossy().contains("gitpoulpi-image-"),
        "a temp copy, not the repository: {got:?}"
    );
    assert!(!got.starts_with(repo.root()));
    assert_eq!(std::fs::read(&got).unwrap(), png(RED), "the blob's bytes");
    assert!(
        std::fs::metadata(&got).unwrap().permissions().readonly(),
        "{got:?} is read-only"
    );
    term.wait_for("the copy notice", STEP, |s| {
        s.contains("opened a copy of pic.png")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// An editor that fails right away reports its error on the status line.
#[test]
fn a_failing_editor_reports_its_error() {
    let work = Work::new();
    let repo = repo(false);
    let (mut term, _config) = launch(repo.root(), "pty-image-edit-fail", &work);
    open_file_view(&mut term, "pic.png");
    click_text(&mut term, "Edit in");
    wait_popup(&term);
    click_text(&mut term, "Failing editor");
    // The status line's error (no card); its entry has what the editor printed.
    common::open_status_error(&mut term, "External tool failed");
    term.wait_for("the editor's error", STEP, |s| {
        s.contains("boom-from-the-editor")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The palette: "Edit in default" runs the default, "Edit in..." opens the
/// list focused, and "Configure..." opens Settings on the Images page.
#[test]
fn the_palette_runs_the_default_opens_the_list_and_configures() {
    let work = Work::new();
    let repo = repo(false);
    let (mut term, _config) = launch(repo.root(), "pty-image-edit-palette", &work);
    open_file_view(&mut term, "pic.png");

    palette(
        &mut term,
        "Edit in default",
        "Image: Edit in Default Editor",
    );
    let got = work.launched();
    assert_eq!(std::fs::read(&got).unwrap(), png(RED));

    palette(&mut term, "Image: Edit In...", "Image: Edit In...");
    wait_popup(&term);
    click_text(&mut term, "Configure");
    term.wait_for("Settings on the Images page", STEP, |s| {
        s.contains("Repo-specific settings") && s.contains("Fake editor")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The commit's changed-files menu offers "Edit in..." on a picture only: it
/// opens the picture's File View at that commit and the editors list as soon
/// as it is loaded; a text file's menu has no such row.
#[test]
fn the_commit_files_menu_offers_edit_in_on_pictures_only() {
    let work = Work::new();
    let repo = repo(false);
    let (mut term, _config) = launch(repo.root(), "pty-image-edit-commitmenu", &work);
    let (c, r) = term.wait_for_text("View all files", STEP).unwrap();
    term.click(c + 6, r).unwrap();
    term.wait_for_text("pic.png", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    common::right_click_text_until(&mut term, "note.txt", "the text file's menu", |s| {
        s.contains("Copy file path")
    });
    assert!(!snap(&term).contains("Edit in..."), "{}", term.text());
    term.send_keys("Escape").unwrap();
    common::right_click_text_until(&mut term, "pic.png", "the picture's menu", |s| {
        s.contains("Edit in...")
    });
    click_text(&mut term, "Edit in...");
    wait_popup(&term);
    assert!(
        snap(&term).contains("File View - pic.png @"),
        "{}",
        term.text()
    );
    click_text(&mut term, "Fake editor");
    let got = work.launched();
    assert_eq!(std::fs::read(&got).unwrap(), png(RED));
    term.stop().unwrap();
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
