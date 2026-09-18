//! The Status pane's folder context menu, the empty-area menu and the
//! Create File flow, end to end through a pty: the real binary, the real
//! daemon, real `git`.
//!
//! Same rules as `pty_file_menu.rs`: nothing located by a hard-coded
//! coordinate, every wait describes the whole state it waits for, temp
//! config dirs and fixture repositories only, and `general.file_explorer`
//! pointed at a recorder so no real Explorer is ever started.

mod common;

use common::git;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(20);

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("alpha/a.txt", "one\n", "add a")
        .unwrap()
        .commit_file("beta/b.txt", "two\n", "add b")
        .unwrap()
        .write_worktree("alpha/a.txt", b"one changed\n")
        .unwrap()
        .write_worktree("beta/b.txt", b"two changed\n")
        .unwrap()
        .build()
}

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

/// Selects the WIP row so the Status pane lists the changes (Tree mode is
/// the default) and waits for the folder rows.
fn show_changes(term: &mut Terminal) {
    let (col, row) = term
        .wait_for_text("\u{25CC}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
    term.wait_for("the folder rows", STEP, |s| {
        s.contains("alpha/") && s.contains("beta/")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Right-clicks the Unstaged folder row of `folder`.
fn open_folder_menu_on(term: &mut Terminal, folder: &str) {
    show_changes(term);
    // Located afresh and right-clicked until the menu is up: the rows of the
    // list can still move after their first paint (the daemon's answers land
    // in stages), and a right-click on the row that was there a moment ago
    // opens nothing.
    let needle = format!("\u{25BE} {folder}/");
    term.act_until(
        "the folder menu",
        Duration::from_secs(4),
        STEP,
        |t| {
            let (col, row) = t.wait_for_settled_text(&needle, Duration::from_millis(150), STEP)?;
            t.right_click(col + 3, row)
        },
        |s| s.contains("Stash folder"),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
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

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = Instant::now() + STEP;
    while !ok() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn the_folder_menu_lists_its_rows_in_order() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-foldermenu-order", "");
    open_folder_menu_on(&mut term, "alpha");
    let screen = term.screen();
    let labels = [
        "Stage folder",
        "Discard all changes in folder...",
        "Ignore all files in 'alpha/'",
        "Stash folder",
        "Create patch from changes in folder...",
        "Open folder",
        "Create a file in this folder",
    ];
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
    // The separator before the last group takes a row of its own.
    let (_, patch_row) = screen
        .find("Create patch from changes in folder...")
        .unwrap();
    let (_, open_row) = screen.find("Open folder").unwrap();
    assert!(open_row > patch_row + 1, "no separator before Open folder");
    term.send_keys("Esc").unwrap();
    term.wait_for("the menu closed", STEP, |s| !s.contains("Stash folder"))
        .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn stage_folder_stages_only_that_folder() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-foldermenu-stage", "");
    open_folder_menu_on(&mut term, "alpha");
    click_row(&mut term, "Stage folder");
    wait_until("alpha/a.txt to be staged", || {
        git(repo.root(), &["diff", "--cached", "--name-only"]).trim() == "alpha/a.txt"
    });
    assert!(
        git(repo.root(), &["diff", "--name-only"]).contains("beta/b.txt"),
        "beta/b.txt stays unstaged"
    );
}

#[test]
fn the_folder_menu_also_opens_from_the_keyboard() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-foldermenu-keys", "");
    show_changes(&mut term);
    let (col, row) = term
        .wait_for_text("\u{25BE} alpha/", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    // Click selects the row (and toggles its fold); the menu key opens it.
    term.click(col + 3, row).unwrap();
    term.wait_for("the folder collapsed", STEP, |s| {
        s.contains("\u{25B8} alpha/")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The palette twin of the menu key (`.` needs the Status pane focused).
    term.send_keys("C-p").unwrap();
    term.send_text("Working Changes: Open Selected Folder Menu")
        .unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the folder menu", STEP, |s| s.contains("Stash folder"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_until("alpha/a.txt to be staged by the keyboard", || {
        git(repo.root(), &["diff", "--cached", "--name-only"]).trim() == "alpha/a.txt"
    });
}

#[test]
fn ignore_folder_appends_dir_slash_to_gitignore() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-foldermenu-ignore", "");
    open_folder_menu_on(&mut term, "beta");
    click_row(&mut term, "Ignore all files in 'beta/'");
    // Every ignore asks which file first; Enter takes the default, the root.
    term.wait_for("the rule-file picker", STEP, |s| {
        s.contains("add to which file?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_until(".gitignore to get /beta/", || {
        std::fs::read_to_string(repo.root().join(".gitignore"))
            .unwrap_or_default()
            .lines()
            .any(|l| l == "/beta/")
    });
    assert!(
        std::process::Command::new("git")
            .args(["check-ignore", "--no-index", "-q", "beta/b.txt"])
            .current_dir(repo.root())
            .status()
            .unwrap()
            .success(),
        "git really ignores the folder"
    );
}

#[test]
fn open_folder_goes_through_the_explorer_template() {
    let repo = fixture();
    let scratch = tempfile::tempdir().unwrap();
    let log: PathBuf = scratch.path().join("explorer.txt");
    let toml = format!(
        "[general]\nfile_explorer = {}\n",
        toml_string(&recorder("{path}", &log))
    );
    let (mut term, _config) = launch(repo.root(), "pty-foldermenu-open", &toml);
    open_folder_menu_on(&mut term, "alpha");
    click_row(&mut term, "Open folder");
    wait_until("the recorder to be written", || {
        std::fs::read_to_string(&log).is_ok_and(|t| t.contains("alpha"))
    });
}

#[test]
fn right_click_on_the_empty_area_offers_create_file_and_creates_a_nested_file() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-foldermenu-empty", "");
    show_changes(&mut term);
    // The Unstaged pane hugs its rows, so the spot under the last one is
    // the pane's own bottom border.
    // Located afresh and right-clicked until the menu is up: the rows can
    // still move after their first paint (the daemon's answers land in
    // stages), and a right-click on the spot that was under the last row a
    // moment ago opens nothing.
    term.act_until(
        "the empty-area menu",
        Duration::from_secs(4),
        STEP,
        |t| {
            let (col, row) =
                t.wait_for_settled_text("\u{270E} b.txt", Duration::from_millis(150), STEP)?;
            t.right_click(col + 1, row + 1)
        },
        |s| s.contains("Create file") && !s.contains("Stash folder"),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_row(&mut term, "Create file");
    term.wait_for("the name prompt", STEP, |s| s.contains("New file name"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A refused name keeps the prompt open and says why.
    term.send_text("bad*name.txt").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the refusal", STEP, |s| s.contains("is not allowed"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for _ in 0..12 {
        term.send_keys("Backspace").unwrap();
    }
    term.send_text("notes/new.md").unwrap();
    term.send_keys("Enter").unwrap();
    wait_until("notes/new.md to exist", || {
        repo.root().join("notes").join("new.md").is_file()
    });
    term.wait_for("the editor on the new file", STEP, |s| {
        s.contains("Edit - notes/new.md @ working tree")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_until("the new file to be untracked", || {
        git(repo.root(), &["status", "--porcelain", "-uall"]).contains("?? notes/new.md")
    });
}

#[test]
fn create_a_file_in_this_folder_creates_it_inside_the_folder_and_refuses_an_existing_one() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-foldermenu-create", "");
    open_folder_menu_on(&mut term, "alpha");
    click_row(&mut term, "Create a file in this folder");
    term.wait_for("the prompt naming the folder", STEP, |s| {
        s.contains("New file name in alpha/")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("a.txt").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the existing file refused", STEP, |s| {
        s.contains("already exists")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for _ in 0..5 {
        term.send_keys("Backspace").unwrap();
    }
    term.send_text("c.txt").unwrap();
    term.send_keys("Enter").unwrap();
    wait_until("alpha/c.txt to exist", || {
        repo.root().join("alpha").join("c.txt").is_file()
    });
    assert!(!repo.root().join("c.txt").exists());
    term.wait_for("the editor on the new file", STEP, |s| {
        s.contains("Edit - alpha/c.txt @ working tree")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_palette_creates_a_file_too() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-foldermenu-palette", "");
    show_changes(&mut term);
    term.send_keys("C-p").unwrap();
    term.send_text("Working Changes: Create File").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the name prompt", STEP, |s| s.contains("New file name"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("top.txt").unwrap();
    term.send_keys("Enter").unwrap();
    wait_until("top.txt to exist", || repo.root().join("top.txt").is_file());
}
