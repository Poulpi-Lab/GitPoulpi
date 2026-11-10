//! The right pane's file lists end to end through a pty: the real binary, the
//! real daemon, real `git`. Multi-select with the Log's gestures
//! (Ctrl+click, Shift+click, Insert), the menus of the commit's files, View
//! all files and the empty areas, the bulk Ignore / Convert / Restore and
//! their Ctrl+Z.
//!
//! Same rules as `pty_folder_menu.rs`: nothing located by a hard-coded
//! coordinate, every wait describes the whole state it waits for, temp config
//! dirs and fixture repositories only, and the file explorer / editor never
//! started (`GITPOULPI_DESKTOP_NO_HOOKS` through `common::launch`).

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(20);

fn three_files() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("one.txt", "1\n", "add files")
        .unwrap()
        .commit_file("two.txt", "2\n", "add two")
        .unwrap()
        .commit_file("three.txt", "3\n", "add three")
        .unwrap()
        .write_worktree("one.txt", b"1 changed\n")
        .unwrap()
        .write_worktree("two.txt", b"2 changed\n")
        .unwrap()
        .write_worktree("three.txt", b"3 changed\n")
        .unwrap()
        .build()
}

fn launch(
    repo: &Path,
    session: &str,
    newest_subject: &'static str,
) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the fixture's log", move |s| s.contains(newest_subject))
        .start()
}

/// Selects the WIP row so the Status pane lists the changes.
fn show_changes(term: &mut Terminal, first_file: &str) {
    let (col, row) = term
        .wait_for_text("\u{25CC}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
    term.wait_for("the changed files", STEP, |s| s.contains(first_file))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Where the list row naming `file` is drawn (marker, then the name).
fn row_of(term: &Terminal, file: &str) -> (u16, u16) {
    term.wait_for_text(file, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

/// The menu row whose whole text is `label` (so `LF` is not found inside
/// `CRLF`, `UTF-8` not inside `UTF-8 BOM`, `Stage` not inside `Stage all`):
/// a popup row is the label between the menu's own borders, padded with
/// spaces.
fn exact_row(screen: &Screen, label: &str) -> Option<(u16, u16)> {
    screen.find_all(label).into_iter().find(|&(c, r)| {
        let line: Vec<char> = screen.line(r).chars().collect();
        let (c, n) = (c as usize, label.chars().count());
        let opens = c >= 1 && line[c - 1] == '\u{2502}';
        let rest: String = line.iter().skip(c + n).collect();
        let closes = rest.trim_start_matches(' ').starts_with('\u{2502}');
        opens && closes
    })
}

fn click_exact(term: &mut Terminal, label: &str) {
    let screen = term
        .wait_for(&format!("a menu row '{label}'"), STEP, |s| {
            exact_row(s, label).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = exact_row(&screen, label).unwrap();
    term.click(col + 1, row).unwrap();
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = Instant::now() + xtask::pty::scaled(STEP);
    while !ok() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Runs `attempt` (which does something and reports whether it worked)
/// until it does, or the step deadline passes.
fn wait_until_with(what: &str, mut attempt: impl FnMut() -> bool) {
    let deadline = Instant::now() + xtask::pty::scaled(STEP);
    while !attempt() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
    }
}

fn wait_for_file(path: &Path, want: &[u8], within: Duration) -> bool {
    let deadline = Instant::now() + xtask::pty::scaled(within);
    loop {
        if std::fs::read(path).ok().as_deref() == Some(want) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Right-clicks the row naming `file` (shifted by `(dx, dy)`) until `menu`
/// shows. The row is located afresh for every attempt, once it has held
/// still: a commit's details arrive in stages and can push the list down a
/// row after its first paint, and the click then lands on the wrong row.
fn open_menu(
    term: &mut Terminal,
    file: &str,
    (dx, dy): (u16, u16),
    what: &str,
    menu: impl FnMut(&Screen) -> bool,
) -> Screen {
    term.act_until(
        what,
        Duration::from_secs(4),
        STEP,
        |t| {
            let (col, row) = t.wait_for_settled_text(file, Duration::from_millis(150), STEP)?;
            t.right_click(col + dx, row + dy)
        },
        menu,
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

fn staged(repo: &Path) -> Vec<String> {
    let out = std::process::Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(repo)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

/// Ctrl+clicks `one.txt` and `three.txt` and opens the menu on `three.txt`.
fn select_one_and_three(term: &mut Terminal) {
    show_changes(term, "one.txt");
    let (c1, r1) = row_of(term, "one.txt");
    term.ctrl_click(c1, r1).unwrap();
    let (c3, r3) = row_of(term, "three.txt");
    term.ctrl_click(c3, r3).unwrap();
    term.wait_for("2 files selected", STEP, |s| s.contains("2 files selected"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    open_menu(term, "three.txt", (0, 0), "the selection's menu", |s| {
        s.contains("Treat as binary") && s.contains("Copy file paths")
    });
}

#[test]
fn ctrl_click_two_files_then_stage_stages_exactly_those() {
    let repo = three_files();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-stage", "add three");
    select_one_and_three(&mut term);
    assert!(
        term.text().contains("2 files"),
        "the menu's caption names the selection:\n{}",
        term.text()
    );
    click_exact(&mut term, "Stage");
    wait_until("exactly the two selected files staged", || {
        staged(repo.root()) == ["one.txt", "three.txt"]
    });
}

#[test]
fn shift_click_takes_the_range_and_a_plain_click_starts_over() {
    let repo = three_files();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-range", "add three");
    show_changes(&mut term, "one.txt");
    let (c1, r1) = row_of(&term, "one.txt");
    term.click(c1, r1).unwrap();
    // The list is sorted: one, three, two.
    let (c3, r3) = row_of(&term, "two.txt");
    term.shift_click(c3, r3).unwrap();
    term.wait_for("3 files selected", STEP, |s| s.contains("3 files selected"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // A plain click on one row starts over: its menu is the single file's.
    let (c2, r2) = row_of(&term, "two.txt");
    term.click(c2, r2).unwrap();
    // Not `open_menu`: the click opens the file's diff, whose header names
    // `two.txt` above the list, so locating it again would find the header.
    term.right_click(c2, r2).unwrap();
    term.wait_for("the single file's menu", STEP, |s| {
        s.contains("Stash file") && !s.contains("Copy file paths")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_keyboard_route_selects_and_opens_the_menu_on_the_selection() {
    let repo = three_files();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-keys", "add three");
    show_changes(&mut term, "one.txt");
    // Ctrl+click selects without opening a diff (a diff would own the arrows).
    let (c1, r1) = row_of(&term, "one.txt");
    term.ctrl_click(c1, r1).unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Insert").unwrap();
    term.wait_for("2 files selected", STEP, |s| s.contains("2 files selected"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys(".").unwrap();
    term.wait_for("the selection's menu", STEP, |s| {
        s.contains("Copy file paths") && s.contains("Treat as binary")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The first selectable row is Stage.
    term.send_keys("Enter").unwrap();
    wait_until("the two selected files staged", || {
        staged(repo.root()).len() == 2
    });
}

/// Space selects in Unstaged / Staged as everywhere else; Ctrl+Space, the
/// key with consequences, stages: the selection when there is one, else the
/// row under the cursor.
#[test]
fn space_selects_and_ctrl_space_stages_in_the_working_changes() {
    let repo = three_files();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-space", "add three");
    show_changes(&mut term, "one.txt");
    // Ctrl+click selects without opening a diff (a diff would own the arrows).
    let (c1, r1) = row_of(&term, "one.txt");
    term.ctrl_click(c1, r1).unwrap();
    // The list is sorted: one, three, two. Down then Space adds three.txt.
    term.send_keys("Down").unwrap();
    term.send_keys("Space").unwrap();
    term.wait_for("2 files selected", STEP, |s| s.contains("2 files selected"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(staged(repo.root()).is_empty(), "Space never stages");
    term.send_keys("C-Space").unwrap();
    wait_until("the two selected files staged", || {
        let s = staged(repo.root());
        s.len() == 2 && s.iter().any(|p| p == "one.txt") && s.iter().any(|p| p == "three.txt")
    });
}

#[test]
fn ignoring_a_selection_through_the_picker_writes_every_pattern() {
    let repo = three_files();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-ignore", "add three");
    select_one_and_three(&mut term);
    click_exact(&mut term, "Ignore \u{203A}");
    click_exact(&mut term, "These files");
    term.wait_for("the which-file picker", STEP, |s| s.contains(".gitignore"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_until("both patterns in .gitignore", || {
        std::fs::read_to_string(repo.root().join(".gitignore"))
            .map(|t| t.lines().any(|l| l == "/one.txt") && t.lines().any(|l| l == "/three.txt"))
            .unwrap_or(false)
    });
    let text = std::fs::read_to_string(repo.root().join(".gitignore")).unwrap();
    assert!(!text.contains("two.txt"), "{text}");
}

fn docs_crlf() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("docs/a.txt", "a1\na2\n", "add a")
        .unwrap()
        .commit_file("docs/b.txt", "b1\nb2\n", "add b")
        .unwrap()
        .write_worktree("docs/a.txt", b"a1\r\na2\r\n")
        .unwrap()
        .write_worktree("docs/b.txt", b"b1\r\nb2\r\n")
        .unwrap()
        .build()
}

#[test]
fn converting_a_folder_to_lf_rewrites_the_bytes_and_ctrl_z_restores_them() {
    let repo = docs_crlf();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-eol", "add b");
    show_changes(&mut term, "docs/");
    open_menu(&mut term, "docs/", (2, 0), "the folder menu", |s| {
        s.contains("Stash folder")
    });
    click_exact(&mut term, "Convert \u{203A}");
    click_exact(&mut term, "Line endings \u{203A}");
    click_exact(&mut term, "LF");
    let a = repo.root().join("docs/a.txt");
    let b = repo.root().join("docs/b.txt");
    wait_until("both files rewritten with LF", || {
        std::fs::read(&a).ok().as_deref() == Some(b"a1\na2\n")
            && std::fs::read(&b).ok().as_deref() == Some(b"b1\nb2\n")
    });
    term.wait_for("the summary", STEP, |s| s.contains("converted 2 files"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-z").unwrap();
    wait_until("one Ctrl+Z puts both CRLF files back", || {
        std::fs::read(&a).ok().as_deref() == Some(b"a1\r\na2\r\n")
            && std::fs::read(&b).ok().as_deref() == Some(b"b1\r\nb2\r\n")
    });
}

#[test]
fn converting_a_selection_from_windows_1252_to_utf8_and_back_with_ctrl_z() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("README.md", "hi\n", "add readme")
        .unwrap()
        .write_worktree("latin1.txt", b"caf\xe9\n")
        .unwrap()
        .write_worktree("latin2.txt", b"cr\xe8me\n")
        .unwrap()
        .write_worktree("bin.dat", b"\x00\x01\x02")
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-enc", "add readme");
    show_changes(&mut term, "latin1.txt");
    let (c1, r1) = row_of(&term, "latin1.txt");
    term.ctrl_click(c1, r1).unwrap();
    let (c2, r2) = row_of(&term, "latin2.txt");
    term.ctrl_click(c2, r2).unwrap();
    // The cursor started on bin.dat (first row): it joined the selection with
    // the first Ctrl+click, as the Log's cursor commit does.
    term.wait_for("3 files selected", STEP, |s| s.contains("3 files selected"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    open_menu(
        &mut term,
        "latin2.txt",
        (0, 0),
        "the selection's menu",
        |s| s.contains("Copy file paths"),
    );
    click_exact(&mut term, "Convert \u{203A}");
    click_exact(&mut term, "Encoding \u{203A}");
    click_exact(&mut term, "UTF-8");
    let l1 = repo.root().join("latin1.txt");
    let l2 = repo.root().join("latin2.txt");
    wait_until("both files re-encoded as UTF-8", || {
        std::fs::read(&l1).ok().as_deref() == Some("caf\u{e9}\n".as_bytes())
            && std::fs::read(&l2).ok().as_deref() == Some("cr\u{e8}me\n".as_bytes())
    });
    // The binary file is skipped and said so.
    term.wait_for("the summary names the skipped binary", STEP, |s| {
        s.contains("converted 2 files, skipped 1 (1 binary)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        std::fs::read(repo.root().join("bin.dat")).unwrap(),
        b"\x00\x01\x02"
    );
    term.send_keys("C-z").unwrap();
    wait_until("Ctrl+Z restores the Windows-1252 bytes", || {
        std::fs::read(&l1).ok().as_deref() == Some(b"caf\xe9\n")
            && std::fs::read(&l2).ok().as_deref() == Some(b"cr\xe8me\n")
    });
}

#[test]
fn a_file_open_in_the_editor_blocks_the_conversion_and_nothing_is_written() {
    let repo = three_files();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-editor", "add three");
    show_changes(&mut term, "one.txt");
    open_menu(&mut term, "one.txt", (0, 0), "the file menu", |s| {
        exact_row(s, "Edit file").is_some()
    });
    click_exact(&mut term, "Edit file");
    term.wait_for("the editor", STEP, |s| s.contains("1 changed"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c1, r1) = row_of(&term, "one.txt");
    term.ctrl_click(c1, r1).unwrap();
    let (c2, r2) = row_of(&term, "two.txt");
    term.ctrl_click(c2, r2).unwrap();
    term.wait_for("2 files selected", STEP, |s| s.contains("2 files selected"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    open_menu(&mut term, "two.txt", (0, 0), "the selection's menu", |s| {
        s.contains("Copy file paths")
    });
    click_exact(&mut term, "Convert \u{203A}");
    click_exact(&mut term, "Line endings \u{203A}");
    click_exact(&mut term, "CRLF");
    term.wait_for("the refusal", STEP, |s| {
        s.contains("one.txt is open in the editor")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        std::fs::read(repo.root().join("two.txt")).unwrap(),
        b"2 changed\n",
        "nothing of the batch is written"
    );
}

fn two_commits() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("notes/a.txt", "v1\n", "first version")
        .unwrap()
        .commit_file("notes/a.txt", "v2\n", "second version")
        .unwrap()
        .build()
}

/// Clicks the commit row of `subject` and waits for its changed files.
fn select_commit(term: &mut Terminal, subject: &str) {
    let (col, row) = term
        .wait_for_text(subject, STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
    term.wait_for("the commit's changed files", STEP, |s| {
        s.contains("Changed files") && s.contains("a.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_commit_file_menu_has_the_full_set_and_restore_is_undone_by_ctrl_z() {
    let repo = two_commits();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-restore", "second version");
    select_commit(&mut term, "first version");
    let screen = open_menu(&mut term, "a.txt", (0, 0), "the commit file menu", |s| {
        s.contains("Restore file from this commit")
    });
    let wants = [
        "File History",
        "File Blame",
        "Open file",
        "Open in ",
        "Open file in default program",
        "Show in folder",
        "Edit file",
        "Copy file path",
        "Copy full file path",
        "Delete file...",
    ];
    // The frame is read in pieces: wait for the whole menu before judging.
    let screen = term
        .wait_for("every menu row drawn", STEP, |s| {
            wants.iter().all(|w| s.contains(w))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = screen;
    click_exact(&mut term, "Restore file from this commit");
    term.wait_for("the confirmation", STEP, |s| s.contains("Overwrite"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("y").unwrap();
    let file = repo.root().join("notes/a.txt");
    wait_until("the file holds the old version", || {
        std::fs::read(&file).ok().as_deref() == Some(b"v1\n")
    });
    // The file is rewritten *before* the daemon journals the operation, and
    // a restore reports nothing on screen when it is done, so there is no
    // signal to wait for between "the file changed" and "Ctrl+Z has
    // something to undo". A Ctrl+Z that comes too early is a no-op; press it
    // again until the file is back (nothing else is journaled here, so a
    // repeat after a slow success undoes nothing more).
    wait_until_with("Ctrl+Z brings the current version back", || {
        term.send_keys("C-z").unwrap();
        wait_for_file(&file, b"v2\n", Duration::from_secs(3))
    });
}

#[test]
fn deleting_from_the_commit_file_menu_confirms_and_removes_the_working_copy() {
    let repo = two_commits();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-delete", "second version");
    select_commit(&mut term, "second version");
    open_menu(&mut term, "a.txt", (0, 0), "the commit file menu", |s| {
        s.contains("Delete file...")
    });
    click_exact(&mut term, "Delete file...");
    term.wait_for("the confirmation", STEP, |s| s.contains("from disk?"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("y").unwrap();
    wait_until("the working copy gone", || {
        !repo.root().join("notes/a.txt").exists()
    });
}

#[test]
fn the_empty_area_of_the_commit_files_offers_create_file_and_the_repo_folder() {
    let repo = two_commits();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-empty", "second version");
    select_commit(&mut term, "first version");
    // Two rows under the last file is inside the list but on no row.
    open_menu(&mut term, "a.txt", (0, 3), "the empty-area menu", |s| {
        s.contains("Create file") && s.contains("Open repository folder")
    });
}

#[test]
fn the_unstaged_empty_area_offers_the_bulk_buttons() {
    let repo = three_files();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-unstaged-empty", "add three");
    show_changes(&mut term, "one.txt");
    open_menu(&mut term, "three.txt", (0, 2), "the empty-area menu", |s| {
        exact_row(s, "Stage all changes").is_some()
            && exact_row(s, "Discard all changes...").is_some()
            && exact_row(s, "Open repository folder").is_some()
            && exact_row(s, "Create file").is_some()
    });
}

#[test]
fn a_commit_folder_row_opens_the_folder_menu() {
    let repo = two_commits();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-folder", "second version");
    select_commit(&mut term, "first version");
    // The commit's files default to the flat Path list: the Tree switch
    // gives the folder row.
    // The Tree switch is a toggle: clicked once, at a position that has held
    // still, and its effect (the folder row) awaited before going on.
    let (c, r) = term
        .wait_for_settled_text("Tree", Duration::from_millis(150), STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c + 1, r).unwrap();
    // In the flat list the folder is part of the file's row; in the tree it
    // has a row of its own.
    term.wait_for("the folder row", STEP, |s| {
        s.find_all("notes/")
            .into_iter()
            .any(|(_, r)| !s.line(r).contains("a.txt"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    open_menu(&mut term, "notes/", (2, 0), "the folder menu", |s| {
        s.contains("Open folder")
            && s.contains("Create a file in this folder")
            && s.contains("Treat files in folder as")
            && s.contains("Convert")
    });
}

#[test]
fn view_all_files_menu_on_an_unchanged_file_has_open_file_and_copy_paths() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("README.md", "hi\n", "add readme")
        .unwrap()
        .commit_file("src/main.rs", "fn main() {}\n", "add main")
        .unwrap()
        .write_worktree("README.md", b"hi changed\n")
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-viewall", "add main");
    show_changes(&mut term, "README.md");
    let (c, r) = term
        .wait_for_text("[ ] View all files", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c + 1, r).unwrap();
    term.wait_for("the whole tree", STEP, |s| s.contains("All files"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    open_menu(&mut term, "main.rs", (0, 0), "the file menu", |s| {
        s.contains("Open file") && s.contains("Copy full file path") && s.contains("Delete file...")
    });
}

/// One commit that adds two files, so a per-file patch can be told from the
/// whole commit's.
fn two_files_in_one_commit() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("base.txt", "base\n", "base")
        .unwrap()
        .write_worktree("a.txt", b"alpha\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .commit("both files")
        .unwrap()
        .build()
}

/// Create patch on one file of a commit writes a patch that carries that
/// file and not the commit's other one.
#[test]
fn create_patch_from_a_commit_file_carries_only_that_file() {
    let repo = two_files_in_one_commit();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-patch", "both files");
    let (col, row) = term
        .wait_for_text("both files", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
    term.wait_for("the commit's two changed files", STEP, |s| {
        s.contains("Changed files") && s.contains("a.txt") && s.contains("both files.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The list row, not the Log's subject: the one led by the file name.
    let (col, row) = snap(&term)
        .find_all("a.txt")
        .into_iter()
        .find(|&(_, r)| !snap(&term).line(r).contains("both files.txt"))
        .unwrap_or_else(|| panic!("a.txt not listed:\n{}", term.text()));
    term.right_click(col, row).unwrap();
    term.wait_for("the commit file menu", STEP, |s| {
        s.contains("Create patch from file changes...")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_exact(&mut term, "Create patch from file changes...");
    term.wait_for("the Create patch dialog", STEP, |s| {
        s.contains("Create patch") && s.contains("Save here")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = snap(&term).find("Save here").unwrap();
    term.click(c + 1, r).unwrap();
    let patch = || {
        std::fs::read_dir(repo.root())
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|e| e == "patch"))
    };
    wait_until("the patch written", || patch().is_some());
    let text = std::fs::read_to_string(patch().unwrap()).unwrap();
    assert!(text.contains("a.txt"), "the asked file:\n{text}");
    assert!(
        !text.contains("both files.txt"),
        "only the asked file:\n{text}"
    );
}

/// "Diff against working tree" in a commit file's menu opens the diff of the
/// commit's version against what is on disk.
#[test]
fn diff_against_working_tree_from_a_commit_file_shows_the_disk_version() {
    let repo = two_commits();
    std::fs::write(repo.root().join("notes/a.txt"), "v3 on disk\n").unwrap();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-diffwt", "second version");
    select_commit(&mut term, "first version");
    open_menu(&mut term, "a.txt", (0, 0), "the commit file menu", |s| {
        s.contains("Diff against working tree")
    });
    click_exact(&mut term, "Diff against working tree");
    term.wait_for("the commit's v1 against the disk's v3", STEP, |s| {
        s.contains("v3 on disk") && s.contains("v1") && s.contains("notes/a.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Ctrl+A selects every row of the focused file list.
#[test]
fn ctrl_a_selects_every_file_of_the_focused_list() {
    let repo = three_files();
    let (mut term, _config) = launch(repo.root(), "pty-filepane-ctrla", "add three");
    show_changes(&mut term, "one.txt");
    let (c1, r1) = row_of(&term, "one.txt");
    term.ctrl_click(c1, r1).unwrap();
    term.send_keys("C-a").unwrap();
    term.wait_for("3 files selected", STEP, |s| s.contains("3 files selected"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
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
