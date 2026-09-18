//! "Which `.gitignore`?" and "Treat as binary / Text", end to end through a
//! pty: the real binary, the real daemon, real `git`.
//!
//! Same rules as `pty_file_menu.rs`: nothing located by a hard-coded
//! coordinate, every wait describes the whole state it waits for, temp
//! config dirs and fixture repositories only.

mod common;

use common::git;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(20);

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the fixture's log", |s| s.contains("add a"))
        .start()
}

/// A tracked text file, a tracked file with a NUL in it, and an untracked
/// log deep in a folder: everything the tests below act on.
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    let _ = STARTUP;
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("notes.dat", "one\ntwo\n", "add a")
        .unwrap()
        .commit_bytes("raw.bin", b"a\0b\nc\n", "add raw")
        .unwrap()
        .write_worktree("notes.dat", b"one\nchanged line\n")
        .unwrap()
        .write_worktree("raw.bin", b"a\0b\nc\nplain added\n")
        .unwrap()
        .write_worktree("src/a/junk.log", b"noise\n")
        .unwrap()
        .build()
}

/// Selects the WIP row so the Status pane lists the changes.
fn show_changes(term: &mut Terminal, needle: &str) {
    let (col, row) = term
        .wait_for_text("\u{25CC}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
    term.wait_for_text(needle, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Right-clicks the row showing `name` until the file menu is up.
fn open_menu_on(term: &mut Terminal, name: &str) {
    show_changes(term, name);
    common::right_click_text_until(term, name, "the file menu", |s| s.contains("Stash file"));
}

/// Right-clicks the list row of `name` once the Diff view covers the Log
/// (the list row is the one led by the modified marker).
fn open_menu_on_listed(term: &mut Terminal, name: &str) {
    let needle = format!("\u{270E} {name}");
    common::right_click_text_until(term, &needle, "the file menu", |s| s.contains("Stash file"));
}

/// Opens the file's diff in the central view and waits for `shown` (the
/// state before the rule is written).
fn show_diff_of(term: &mut Terminal, name: &str, shown: &str) {
    show_changes(term, name);
    let (col, row) = term
        .screen()
        .find(name)
        .unwrap_or_else(|| panic!("{name} not listed:\n{}", term.text()));
    term.click(col + 1, row).unwrap();
    term.wait_for(
        &format!("the diff of {name} showing '{shown}'"),
        STEP,
        |s| s.contains(shown),
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

fn read(repo: &Path, rel: &str) -> String {
    std::fs::read_to_string(repo.join(rel)).unwrap_or_default()
}

fn check_ignored(repo: &Path, rel: &str) -> bool {
    std::process::Command::new("git")
        .args(["check-ignore", "-q", rel])
        .current_dir(repo)
        .status()
        .unwrap()
        .success()
}

/// Ignoring `src/a/junk.log` into `src/a/.gitignore`, a file that does not
/// exist yet: the picker lists the root first, marks the missing files
/// "new", writes the pattern relative to the chosen folder, and git really
/// ignores the file.
#[test]
fn ignoring_a_file_into_a_subfolder_gitignore_writes_a_relative_line() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-rulefiles-subfolder");
    open_menu_on(&mut term, "junk.log");
    click_row(&mut term, "Ignore \u{203A}");
    click_row(&mut term, "Ignore 'junk.log'");
    let screen = term
        .wait_for("the picker with every candidate file", STEP, |s| {
            s.contains("add to which file?")
                && s.contains(".gitignore (repository root, new)")
                && s.contains("src/.gitignore (new)")
                && s.contains("src/a/.gitignore (new)")
                && s.contains(".git/info/exclude (this clone only")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The root is first and the line each file would get is shown.
    let (_, root_row) = screen.find(".gitignore (repository root, new)").unwrap();
    let (_, sub_row) = screen.find("src/a/.gitignore (new)").unwrap();
    assert!(root_row < sub_row, "{}", term.text());
    assert!(screen.line(sub_row).contains("/junk.log"));
    assert!(screen.line(root_row).contains("/src/a/junk.log"));

    // A click selects the row, Enter writes.
    click_row(&mut term, "src/a/.gitignore (new)");
    term.send_keys("Enter").unwrap();
    wait_until("src/a/.gitignore to hold the line", || {
        read(repo.root(), "src/a/.gitignore") == "/junk.log\n"
    });
    assert!(!repo.root().join(".gitignore").exists(), "root untouched");
    assert!(check_ignored(repo.root(), "src/a/junk.log"));
    assert!(!git(repo.root(), &["status", "--porcelain", "-uall"]).contains("junk.log"));
    term.stop().unwrap();
}

/// The picker is keyboard-driven: type to narrow, Enter writes. An
/// extension rule goes into `.git/info/exclude` (this clone only).
#[test]
fn ignoring_an_extension_into_info_exclude_by_keyboard() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-rulefiles-exclude");
    open_menu_on(&mut term, "junk.log");
    click_row(&mut term, "Ignore \u{203A}");
    click_row(&mut term, "All files with the extension '.log'");
    term.wait_for("the picker", STEP, |s| s.contains("add to which file?"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("exclude").unwrap();
    term.wait_for("only the per-clone file left", STEP, |s| {
        s.contains(".git/info/exclude") && !s.contains("src/a/.gitignore")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_until("*.log in .git/info/exclude", || {
        read(repo.root(), ".git/info/exclude")
            .lines()
            .any(|l| l == "*.log")
    });
    assert!(!repo.root().join(".gitignore").exists());
    assert!(check_ignored(repo.root(), "src/a/junk.log"));
    term.stop().unwrap();
}

/// Down + Enter picks the second row without the mouse.
#[test]
fn the_picker_moves_with_the_arrow_keys() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-rulefiles-arrows");
    open_menu_on(&mut term, "junk.log");
    click_row(&mut term, "Ignore \u{203A}");
    click_row(&mut term, "Ignore 'junk.log'");
    term.wait_for("the picker", STEP, |s| {
        s.contains("add to which file?") && s.contains("src/.gitignore (new)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    wait_until("src/.gitignore to hold the line", || {
        read(repo.root(), "src/.gitignore") == "/a/junk.log\n"
    });
    assert!(check_ignored(repo.root(), "src/a/junk.log"));
    term.stop().unwrap();
}

/// Esc closes the picker and writes nothing, anywhere.
#[test]
fn escape_cancels_the_picker_and_writes_nothing() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-rulefiles-cancel");
    open_menu_on(&mut term, "junk.log");
    click_row(&mut term, "Ignore \u{203A}");
    click_row(&mut term, "Ignore 'junk.log'");
    term.wait_for("the picker", STEP, |s| s.contains("add to which file?"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Esc").unwrap();
    term.wait_for("the picker closed", STEP, |s| {
        !s.contains("add to which file?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    std::thread::sleep(Duration::from_millis(600));
    for rel in [".gitignore", "src/.gitignore", "src/a/.gitignore"] {
        assert!(!repo.root().join(rel).exists(), "{rel} was written");
    }
    assert!(!read(repo.root(), ".git/info/exclude").contains("junk.log"));
    assert!(git(repo.root(), &["status", "--porcelain", "-uall"]).contains("junk.log"));
    term.stop().unwrap();
}

/// "Treat as binary" on a text file: the picker, the rule in
/// `.gitattributes`, and git (and the Diff view) now call it binary.
/// Ctrl+Z takes the rule out again.
#[test]
fn treat_as_binary_on_a_text_file_writes_the_rule_and_the_diff_follows() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-rulefiles-binary");
    show_diff_of(&mut term, "notes.dat", "changed line");
    open_menu_on_listed(&mut term, "notes.dat");
    // A text file offers Binary.
    click_row(&mut term, "Treat as binary \u{203A}");
    click_row(&mut term, "This file ('notes.dat')");
    term.wait_for("the picker", STEP, |s| {
        s.contains("Treat as binary: write to which file?")
            && s.contains(".gitattributes (repository root, new)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_until(".gitattributes to hold the rule", || {
        read(repo.root(), ".gitattributes") == "/notes.dat binary\n"
    });
    assert!(
        git(repo.root(), &["diff", "--", "notes.dat"]).contains("Binary files"),
        "git itself calls it binary"
    );
    // The Diff view of the open file follows: its lines are gone.
    term.wait_for("the diff no longer showing the text lines", STEP, |s| {
        !s.contains("changed line")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The status line says what was written where.
    term.wait_for("the status line", STEP, |s| {
        s.contains("/notes.dat") && s.contains(".gitattributes")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The file now offers Text, and the journaled write is undone by Ctrl+Z.
    std::thread::sleep(Duration::from_millis(800));
    term.send_keys("C-z").unwrap();
    wait_until("Ctrl+Z to remove .gitattributes", || {
        !repo.root().join(".gitattributes").exists()
    });
    assert!(git(repo.root(), &["diff", "--", "notes.dat"]).contains("+changed line"));
    term.stop().unwrap();
}

/// "Treat as text" on a file git calls binary (a NUL byte): the row says
/// Text, the rule is `text diff`, and the diff shows its lines.
#[test]
fn treat_as_text_on_a_binary_looking_file_makes_the_diff_text() {
    let repo = fixture();
    assert!(git(repo.root(), &["diff", "--", "raw.bin"]).contains("Binary files"));
    let (mut term, _config) = launch(repo.root(), "pty-rulefiles-text");
    show_diff_of(&mut term, "raw.bin", "raw.bin - unstaged");
    assert!(!term.text().contains("plain added"), "binary: no lines yet");
    open_menu_on_listed(&mut term, "raw.bin");
    click_row(&mut term, "Treat as text \u{203A}");
    click_row(&mut term, "This file ('raw.bin')");
    term.wait_for("the picker", STEP, |s| {
        s.contains("Treat as text: write to which file?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_until(".gitattributes to hold the rule", || {
        read(repo.root(), ".gitattributes") == "/raw.bin text diff\n"
    });
    assert!(
        git(repo.root(), &["diff", "--", "raw.bin"]).contains("+plain added"),
        "git itself diffs it as text now"
    );
    term.wait_for("the Diff view showing the added line", STEP, |s| {
        s.contains("plain added")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The rule can go to a folder's own `.gitattributes`, relative to it, and
/// to the per-clone file.
#[test]
fn treat_as_binary_into_a_subfolder_attributes_file_is_relative_to_it() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-rulefiles-attr-sub");
    open_menu_on(&mut term, "junk.log");
    click_row(&mut term, "Treat as binary \u{203A}");
    click_row(&mut term, "This file ('junk.log')");
    term.wait_for("the picker", STEP, |s| {
        s.contains("Treat as binary: write to which file?")
            && s.contains("src/a/.gitattributes (new)")
            && s.contains(".git/info/attributes (this clone only")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_row(&mut term, "src/a/.gitattributes (new)");
    term.send_keys("Enter").unwrap();
    wait_until("src/a/.gitattributes to hold the rule", || {
        read(repo.root(), "src/a/.gitattributes") == "/junk.log binary\n"
    });
    assert!(!repo.root().join(".gitattributes").exists());
    term.stop().unwrap();
}

/// A text-looking file that a committed `*.dat binary` rule makes binary
/// offers "Treat as text": the answer comes from git's attributes, not
/// from sniffing the content. The new, more specific line wins over it.
#[test]
fn a_file_a_committed_rule_makes_binary_offers_text() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(".gitattributes", "*.dat binary\n", "add a rules")
        .unwrap()
        .commit_file("notes.dat", "one\ntwo\n", "add notes")
        .unwrap()
        .write_worktree("notes.dat", b"one\nchanged line\n")
        .unwrap()
        .build();
    assert!(git(repo.root(), &["diff", "--", "notes.dat"]).contains("Binary files"));
    let (mut term, _config) = launch(repo.root(), "pty-rulefiles-committed");
    show_diff_of(&mut term, "notes.dat", "notes.dat - unstaged");
    assert!(!term.text().contains("changed line"));
    open_menu_on_listed(&mut term, "notes.dat");
    click_row(&mut term, "Treat as text \u{203A}");
    click_row(&mut term, "This file ('notes.dat')");
    term.wait_for("the picker on the existing root file", STEP, |s| {
        s.contains("Treat as text: write to which file?")
            && s.contains(".gitattributes (repository root)")
            && !s.contains(".gitattributes (repository root, new)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_until("the more specific line after the committed one", || {
        read(repo.root(), ".gitattributes") == "*.dat binary\n/notes.dat text diff\n"
    });
    assert!(git(repo.root(), &["diff", "--", "notes.dat"]).contains("+changed line"));
    term.stop().unwrap();
}

/// A rule closer to the file wins over the one just written: the status
/// line says so instead of letting the user think it worked.
#[test]
fn a_closer_rule_that_still_overrides_is_reported() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("sub/.gitattributes", "n.dat text diff\n", "add a rules")
        .unwrap()
        .commit_file("sub/n.dat", "one\ntwo\n", "add notes")
        .unwrap()
        .write_worktree("sub/n.dat", b"one\nchanged line\n")
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-rulefiles-override");
    open_menu_on(&mut term, "n.dat");
    click_row(&mut term, "Treat as binary \u{203A}");
    click_row(&mut term, "This file ('n.dat')");
    term.wait_for("the picker", STEP, |s| {
        s.contains("Treat as binary: write to which file?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_until("the root rule", || {
        read(repo.root(), ".gitattributes") == "/sub/n.dat binary\n"
    });
    term.wait_for("the warning", STEP, |s| {
        s.contains("another rule still overrides it")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        git(repo.root(), &["diff", "--", "sub/n.dat"]).contains("+changed line"),
        "the closer rule still wins in git itself"
    );
    term.stop().unwrap();
}

/// A repository opened through a symlinked path (macOS temp dirs sit behind
/// `/var` -> `/private/var`, Windows has junctions and `subst` drives) must
/// behave as when opened by its real path: a `*.dat binary` rule makes the
/// text-looking file binary, so its menu offers Text, not Binary.
#[cfg(unix)]
#[test]
fn a_repository_opened_through_a_symlink_still_sees_its_attributes() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(".gitattributes", "*.dat binary\n", "add rules")
        .unwrap()
        .commit_file("notes.dat", "one\ntwo\n", "add a")
        .unwrap()
        .write_worktree("notes.dat", b"one\nchanged line\n")
        .unwrap()
        .build();
    let links = tempfile::tempdir().unwrap();
    let link = links.path().join("via-link");
    std::os::unix::fs::symlink(repo.root(), &link).unwrap();
    let (mut term, _config) = common::launch(&link)
        .session("pty-rulefiles-symlink")
        .size(200, 50)
        .no_exit_pause()
        .arg("--repo")
        .arg(&link)
        .ready_when("the fixture's log", |s| s.contains("add a"))
        .start();
    show_diff_of(&mut term, "notes.dat", "notes.dat - unstaged");
    open_menu_on_listed(&mut term, "notes.dat");
    term.wait_for("the Text row, not Binary", STEP, |s| {
        s.contains("Treat as text")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.text().contains("Treat as binary"), "{}", term.text());
    term.stop().unwrap();
}

/// Right-clicking a file right away, before git's binary answer can have
/// reached the UI: the open menu must not offer "Treat as binary" for a
/// file a committed `*.dat binary` rule already makes binary. Its row is a
/// placeholder first and turns into "Treat as text" in place, without
/// reopening the menu. Two files, so whichever one the Diff view is not
/// showing has nobody asking about it but the menu itself.
#[test]
fn a_menu_opened_at_once_corrects_its_treat_row_when_gits_answer_lands() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(".gitattributes", "*.dat binary\n", "add a rules")
        .unwrap()
        .commit_file("notes.dat", "one\ntwo\n", "add notes")
        .unwrap()
        .commit_file("other.dat", "one\ntwo\n", "add other")
        .unwrap()
        .write_worktree("notes.dat", b"one\nchanged line\n")
        .unwrap()
        .write_worktree("other.dat", b"one\nchanged line\n")
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-rulefiles-immediate");
    show_changes(&mut term, "notes.dat");
    for name in ["notes.dat", "other.dat"] {
        common::right_click_text_until(&mut term, name, "the file menu", |s| {
            s.contains("Stash file")
        });
        term.wait_for("the Text row in the same open menu", STEP, |s| {
            s.contains("Stash file") && s.contains("Treat as text")
        })
        .unwrap_or_else(|e| panic!("{name}: {e}\n{}", term.text()));
        assert!(!term.text().contains("Treat as binary"), "{}", term.text());
        term.send_keys("Escape").unwrap();
        term.wait_for("the menu closed", STEP, |s| !s.contains("Stash file"))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    }
    term.stop().unwrap();
}
