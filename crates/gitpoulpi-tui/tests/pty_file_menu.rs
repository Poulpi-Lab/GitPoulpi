//! The Status pane's file context menu, end to end through a pty: the real
//! binary, the real daemon, real `git`.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! every wait describes the whole state it waits for, temp config dirs and
//! fixture repositories only. The external tools (`general.file_explorer`,
//! `general.external_editor`) are pointed at a harmless recorder -- a shell
//! one-liner appending its argument to a file -- so no real Explorer or
//! VS Code is ever started.

mod common;

use common::git;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(20);

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "add a")
        .unwrap()
        .commit_file("b.txt", "two\n", "add b")
        .unwrap()
        .write_worktree("a.txt", b"one changed\n")
        .unwrap()
        .write_worktree("b.txt", b"two changed\n")
        .unwrap()
        .build()
}

/// A template that appends `{placeholder}` to `record` instead of opening
/// anything.
fn recorder(placeholder: &str, record: &Path) -> String {
    if cfg!(windows) {
        format!("cmd /c echo {placeholder}>>\"{}\"", record.display())
    } else {
        format!("sh -c \"echo {placeholder} >> '{}'\"", record.display())
    }
}

/// `value` as a TOML string literal, quotes and backslashes escaped.
fn toml_string(value: &str) -> String {
    toml::Value::String(value.to_string()).to_string()
}

fn launch(repo: &Path, session: &str, config_toml: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .config(config_toml)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the fixture's log", |s| s.contains("add b"))
        .start()
}

/// Selects the WIP row so the Status pane lists the changes, and
/// right-clicks `name`'s row in the Unstaged half.
fn open_menu_on(term: &mut Terminal, name: &str) {
    let (col, row) = term
        .wait_for_text("\u{25CC}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
    let needle = format!("\u{270E} {name}");
    term.wait_for_text(&needle, STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    common::right_click_text_until(term, &needle, "the file menu", |s| s.contains("Stash file"));
}

fn click_row(term: &mut Terminal, label: &str) {
    let screen = term
        .wait_for(&format!("a menu row '{label}'"), STEP, |s| {
            s.contains(label)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = screen.find(label).unwrap();
    term.click(col + 1, row).unwrap();
}

fn wait_for_record(path: &Path) -> String {
    let deadline = Instant::now() + STEP;
    loop {
        if let Ok(text) = std::fs::read_to_string(path) {
            if !text.trim().is_empty() {
                return text;
            }
        }
        assert!(
            Instant::now() < deadline,
            "the recorder never wrote {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Right-clicking a modified file lists the rows in order.
#[test]
fn right_click_a_modified_file_lists_the_menu_in_order() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-filemenu-order", "");
    open_menu_on(&mut term, "a.txt");

    let labels = [
        "Discard changes...",
        "Ignore \u{203A}",
        "Stash file",
        "File History",
        "File Blame",
        "Open in external diff tool",
        "Open file in default program",
        "Show in folder",
        "Copy file path",
        "Create patch from file changes...",
        "Edit file",
        "Delete file...",
    ];
    // The frame is read in pieces: the first row can be on screen while the
    // last is still being drawn, so wait for the whole menu before judging.
    let screen = term
        .wait_for("every menu row drawn", STEP, |s| {
            labels.iter().all(|l| s.contains(l))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let rows: Vec<u16> = labels
        .iter()
        .map(|l| {
            screen
                .find(l)
                .unwrap_or_else(|| panic!("'{l}' missing from the menu:\n{}", term.text()))
                .1
        })
        .collect();
    assert!(
        rows.windows(2).all(|w| w[0] < w[1]),
        "rows out of order: {rows:?} for {labels:?}"
    );
    // The editor row is named after the editor; whichever it is, it sits
    // between the diff-tool row and the default-program row.
    let (_, diff_row) = screen.find("Open in external diff tool").unwrap();
    let (_, default_row) = screen.find("Open file in default program").unwrap();
    assert!(
        (diff_row + 1..default_row).any(|r| screen.line(r).contains("Open in ")),
        "no 'Open in <editor>' row between the diff tool and the default program"
    );
    term.send_keys("Esc").unwrap();
    term.wait_for("the menu closed", STEP, |s| !s.contains("Stash file"))
        .unwrap_or_else(|e| panic!("{e}"));
}

/// `Ignore >` drills in, and the extension row appends `*.txt`.
#[test]
fn ignore_by_extension_appends_to_gitignore() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-filemenu-ignore", "");
    open_menu_on(&mut term, "a.txt");
    click_row(&mut term, "Ignore \u{203A}");
    click_row(&mut term, "All files with the extension '.txt'");
    // Every ignore asks which file first; Enter takes the default, the root.
    term.wait_for("the rule-file picker", STEP, |s| {
        s.contains("add to which file?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    let ignore = repo.root().join(".gitignore");
    let deadline = Instant::now() + STEP;
    loop {
        let text = std::fs::read_to_string(&ignore).unwrap_or_default();
        if text.lines().any(|l| l == "*.txt") {
            break;
        }
        assert!(
            Instant::now() < deadline,
            ".gitignore never got *.txt: {text:?}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Ignoring is a journaled daemon operation: `Ctrl+Z` takes the pattern
/// back out (the `.gitignore` it created is removed) and the file shows
/// as untracked again.
#[test]
fn ignoring_a_file_from_the_menu_is_undone_by_ctrl_z() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "add a")
        .unwrap()
        .commit_file("b.txt", "two\n", "add b")
        .unwrap()
        .write_worktree("junk.log", b"noise\n")
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-filemenu-ignore-undo", "");
    let (col, row) = term
        .wait_for_text("junk.log", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.right_click(col + 1, row).unwrap();
    term.wait_for("the file menu", STEP, |s| s.contains("Stash file"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_row(&mut term, "Ignore \u{203A}");
    click_row(&mut term, "Ignore 'junk.log'");
    // Every ignore asks which file first; Enter takes the default, the root.
    term.wait_for("the rule-file picker", STEP, |s| {
        s.contains("add to which file?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    let ignore = repo.root().join(".gitignore");
    let deadline = Instant::now() + STEP;
    while !std::fs::read_to_string(&ignore)
        .unwrap_or_default()
        .lines()
        .any(|l| l == "/junk.log")
    {
        assert!(Instant::now() < deadline, ".gitignore never got /junk.log");
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(
        !git(repo.root(), &["status", "--porcelain"]).contains("junk.log"),
        "ignored files leave the status"
    );
    // Let the journal reach the window before the key.
    std::thread::sleep(Duration::from_millis(800));
    term.send_keys("C-z").unwrap();
    let deadline = Instant::now() + STEP;
    while ignore.exists() {
        assert!(Instant::now() < deadline, "Ctrl+Z never removed .gitignore");
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(
        git(repo.root(), &["status", "--porcelain"]).contains("?? junk.log"),
        "the file is untracked again"
    );
    term.stop().unwrap();
}

/// `Delete file` goes through the daemon, so `Ctrl+Z` brings it back.
#[test]
fn deleting_a_file_from_the_menu_is_undone_by_ctrl_z() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-filemenu-delete-undo", "");
    open_menu_on(&mut term, "b.txt");
    click_row(&mut term, "Delete file...");
    term.wait_for("the confirm modal", STEP, |s| {
        s.contains("Delete b.txt from disk?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("y").unwrap();
    let deadline = Instant::now() + STEP;
    while repo.root().join("b.txt").exists() {
        assert!(Instant::now() < deadline, "b.txt was never deleted");
        std::thread::sleep(Duration::from_millis(100));
    }
    std::thread::sleep(Duration::from_millis(800));
    term.send_keys("C-z").unwrap();
    let deadline = Instant::now() + STEP;
    while !repo.root().join("b.txt").exists() {
        assert!(Instant::now() < deadline, "Ctrl+Z never restored b.txt");
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(
        std::fs::read_to_string(repo.root().join("b.txt")).unwrap(),
        "two changed\n"
    );
    term.stop().unwrap();
}

/// `Stash file` stashes that path only; the other modified file stays.
#[test]
fn stash_file_stashes_only_that_file() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-filemenu-stash", "");
    open_menu_on(&mut term, "a.txt");
    click_row(&mut term, "Stash file");

    let deadline = Instant::now() + STEP;
    loop {
        let stashed = git(repo.root(), &["stash", "show", "--name-only", "stash@{0}"]);
        if !stashed.trim().is_empty() {
            assert_eq!(stashed.trim(), "a.txt", "only a.txt may be stashed");
            break;
        }
        assert!(Instant::now() < deadline, "nothing was stashed");
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(git(repo.root(), &["stash", "list"]).lines().count(), 1);
    // git writes the stash before it resets the file: wait for the reset.
    let deadline = Instant::now() + STEP;
    while std::fs::read_to_string(repo.root().join("a.txt"))
        .unwrap_or_default()
        .replace("\r\n", "\n")
        != "one\n"
    {
        assert!(
            Instant::now() < deadline,
            "the stashed file never went back to HEAD"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(
        std::fs::read_to_string(repo.root().join("b.txt"))
            .unwrap()
            .replace("\r\n", "\n"),
        "two changed\n",
        "the other file keeps its change"
    );
}

/// `Show in folder` and `Open in <editor>` launch the configured
/// templates (a recorder here), with the file's absolute path.
#[test]
fn show_in_folder_and_the_editor_run_the_configured_templates() {
    let repo = fixture();
    let scratch = tempfile::tempdir().unwrap();
    let explorer_log: PathBuf = scratch.path().join("explorer.txt");
    let editor_log: PathBuf = scratch.path().join("editor.txt");
    let toml = format!(
        "[general]\nfile_explorer = {}\nexternal_editor = {}\n",
        toml_string(&recorder("{path}", &explorer_log)),
        toml_string(&recorder("{file}", &editor_log)),
    );
    let (mut term, _config) = launch(repo.root(), "pty-filemenu-tools", &toml);

    open_menu_on(&mut term, "a.txt");
    click_row(&mut term, "Show in folder");
    let seen = wait_for_record(&explorer_log);
    assert!(seen.contains("a.txt"), "explorer got {seen:?}");
    assert!(
        seen.to_lowercase().contains(
            &repo
                .root()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_lowercase()
        ),
        "the path is absolute, inside the repository: {seen:?}"
    );

    open_menu_on(&mut term, "b.txt");
    let screen = term
        .wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen
        .find_all("Open in ")
        .into_iter()
        // The tab bar's own "Open in" button sits above the menu.
        .rfind(|(_, r)| !screen.line(*r).contains("external diff"))
        .unwrap_or_else(|| panic!("no editor row:\n{}", term.text()));
    term.click(col + 1, row).unwrap();
    let seen = wait_for_record(&editor_log);
    assert!(seen.contains("b.txt"), "editor got {seen:?}");
}

/// `Copy file path` reports it on the status line, and `Delete file`
/// asks first, then removes the file from disk.
#[test]
fn copy_path_reports_and_delete_file_confirms_first() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-filemenu-delete", "");
    open_menu_on(&mut term, "a.txt");
    click_row(&mut term, "Copy file path");
    term.wait_for("the copy confirmation", STEP, |s| {
        s.contains("copied a.txt to the clipboard")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    open_menu_on(&mut term, "b.txt");
    click_row(&mut term, "Delete file...");
    term.wait_for("the confirm modal", STEP, |s| {
        s.contains("Delete b.txt from disk?") && s.contains("Cancel")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("n").unwrap();
    term.wait_for("the modal gone", STEP, |s| {
        !s.contains("Delete b.txt from disk?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(repo.root().join("b.txt").exists(), "n must keep the file");

    open_menu_on(&mut term, "b.txt");
    click_row(&mut term, "Delete file...");
    term.wait_for("the confirm modal again", STEP, |s| {
        s.contains("Delete b.txt from disk?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("y").unwrap();
    let deadline = Instant::now() + STEP;
    while repo.root().join("b.txt").exists() {
        assert!(Instant::now() < deadline, "b.txt was never deleted");
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Clicking `Discard changes...` in the menu acts on the menu, not on the
/// pane underneath: the discard confirmation opens and the file is still
/// modified until it is confirmed.
#[test]
fn clicking_discard_changes_in_the_menu_opens_its_confirmation() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-filemenu-discard", "");
    open_menu_on(&mut term, "a.txt");
    click_row(&mut term, "Discard changes...");
    term.wait_for("the discard confirmation", STEP, |s| {
        !s.contains("Stash file") && s.contains("Discard")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "one changed\n",
        "nothing is discarded before the confirmation"
    );
    term.stop().unwrap();
}

/// `HOVER_BG` in `gitpoulpi_tui::theme`.
const HOVER_BG: xtask::pty::Color = xtask::pty::Color::Rgb(34, 38, 56);

/// Hovering a menu row hovers that row only: the status rows lying under
/// the open menu must not light up too.
#[test]
fn hovering_a_menu_row_does_not_hover_the_rows_behind_it() {
    let mut builder = FixtureRepoBuilder::new().unwrap();
    for name in ["a", "b", "c", "d", "e", "f", "g", "h"] {
        builder = builder
            .commit_file(&format!("{name}.txt"), "one\n", &format!("add {name}"))
            .unwrap();
    }
    for name in ["a", "b", "c", "d", "e", "f", "g", "h"] {
        builder = builder
            .write_worktree(&format!("{name}.txt"), b"changed\n")
            .unwrap();
    }
    let repo = builder.build();
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-filemenu-hover")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture's log", STARTUP, |s| s.contains("add h"))
        .unwrap_or_else(|e| panic!("{e}"));
    open_menu_on(&mut term, "a.txt");

    for label in ["File History", "Show in folder", "Copy file path"] {
        let screen = term
            .wait_for_stable(Duration::from_millis(300), STEP)
            .unwrap_or_else(|e| panic!("{e}"));
        let (col, row) = screen
            .find(label)
            .unwrap_or_else(|| panic!("'{label}' missing:\n{}", term.text()));
        term.move_to(col + 1, row).unwrap();
        // The menu's own borders on that line bound the part that may be lit.
        let screen = term
            .wait_for("the menu row hovered", STEP, |s| {
                s.cell(col + 1, row)
                    .is_some_and(|c| c.bg != screen.cell(col + 1, row).unwrap().bg)
                    || s.cell(col + 1, row).is_some_and(|c| c.bg == HOVER_BG)
            })
            .unwrap_or_else(|e| panic!("{e}"));
        let left = (0..col).rev().find(|c| {
            screen
                .cell(*c, row)
                .is_some_and(|cell| cell.text == "\u{2502}")
        });
        let right = (col..200).find(|c| {
            screen
                .cell(*c, row)
                .is_some_and(|cell| cell.text == "\u{2502}")
        });
        let (Some(left), Some(right)) = (left, right) else {
            panic!("no menu borders around '{label}':\n{}", term.text());
        };
        for r in 0..50u16 {
            for c in (0..200u16).filter(|c| *c < left || *c > right) {
                if r != row {
                    continue;
                }
                let cell = screen.cell(c, r).unwrap();
                assert_ne!(
                    cell.bg,
                    HOVER_BG,
                    "cell ({c},{r}) behind the menu is hovered while '{label}' is:\n{}",
                    term.text()
                );
            }
        }
    }
    term.stop().unwrap();
}
