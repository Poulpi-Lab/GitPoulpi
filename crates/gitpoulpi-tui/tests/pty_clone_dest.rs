//! The Clone dialog when the destination folder already exists: the confirm
//! popup (empty folder, the same clone, any other folder), by keyboard and
//! by clicking its buttons. The real binary, daemon and git through a pty;
//! no network: the clone is from a fixture repository's path.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, and every wait describes the whole state it
//! is waiting for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 180;
const ROWS: u16 = 45;

struct Setup {
    term: Terminal,
    root: tempfile::TempDir,
    source: FixtureRepo,
    /// The folder name the clone proposes.
    name: String,
    _start: FixtureRepo,
    _config: tempfile::TempDir,
}

/// Forward slashes: typing a literal `\` through ConPTY's synthetic input
/// does not reach the app (see `pty_ui.rs`), and Windows accepts `/`.
fn typed(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

/// The Clone dialog open on a fresh config, `prepare` run on the clone
/// root and the proposed folder name before the dialog opens, the source's
/// path typed as the URL and the full path shown.
fn open_clone(session: &str, prepare: impl FnOnce(&Path, &str, &FixtureRepo)) -> Setup {
    let source = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("s.txt", "s\n", "clone-source-only-commit")
        .unwrap()
        .build();
    let start = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "clone-dest-start-commit")
        .unwrap()
        .build();
    let root = tempfile::tempdir().unwrap();
    let name = source
        .root()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    prepare(root.path(), &name, &source);
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!("[repos]\nclone_root = '{}'\n", root.path().display()),
    )
    .unwrap();
    let mut term = common::launch(start.root())
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .spawn();
    term.wait_for("the first window", STARTUP, |s| {
        s.contains("clone-dest-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-o").expect("the management tab");
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Clone")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("c").expect("Clone");
    term.wait_for("the Clone dialog", STEP, |s| {
        s.contains("Clone a repository") && s.contains("Clone with URL")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(&typed(source.root())).expect("the URL");
    let n = name.clone();
    term.wait_for("the full path", STEP, move |s| {
        s.contains("Full path:") && s.contains(&n)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    Setup {
        term,
        root,
        source,
        name,
        _start: start,
        _config: config,
    }
}

fn click_label(term: &mut Terminal, label: &str) {
    let (col, row) = term
        .screen()
        .find(label)
        .unwrap_or_else(|| panic!("no {label:?}:\n{}", term.text()));
    term.click(col + 1, row).expect("click the button");
}

fn wait_clone_tab(term: &mut Terminal) {
    term.wait_for("the clone open as a tab", STARTUP, |s| {
        s.contains("clone-source-only-commit") && !s.contains("Clone a repository")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git");
    assert!(out.status.success(), "git {args:?}: {out:?}");
}

/// Keyboard: an existing empty folder asks, Enter on the default button
/// clones into it.
#[test]
fn an_empty_destination_asks_then_clones_into_it() {
    let mut s = open_clone("pty-clone-dest-empty", |root, name, _| {
        std::fs::create_dir(root.join(name)).unwrap();
    });
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the confirm popup", STEP, |x| {
            x.contains("The folder exists and is empty") && x.contains("Clone into it")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").unwrap();
    wait_clone_tab(&mut s.term);
    let dest = s.root.path().join(&s.name);
    assert!(dest.join(".git").is_dir(), "no clone at {}", dest.display());
}

/// Mouse: a folder with other files offers the first free `<name>-2`;
/// clicking the button clones there and leaves the folder alone.
#[test]
fn a_busy_destination_offers_a_free_name_and_the_button_clicks() {
    let mut s = open_clone("pty-clone-dest-busy", |root, name, _| {
        std::fs::create_dir(root.join(name)).unwrap();
        std::fs::write(root.join(name).join("mine.txt"), "mine\n").unwrap();
    });
    s.term.send_keys("Enter").unwrap();
    let label = format!("Use \"{}-2\"", s.name);
    let l = label.clone();
    s.term
        .wait_for("the confirm popup", STEP, move |x| {
            x.contains("The folder is not empty") && x.contains(&l) && x.contains("Back")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    click_label(&mut s.term, &label);
    wait_clone_tab(&mut s.term);
    let free = s.root.path().join(format!("{}-2", s.name));
    assert!(free.join(".git").is_dir(), "no clone at {}", free.display());
    assert!(s.root.path().join(&s.name).join("mine.txt").is_file());
}

/// Keyboard twin of the above: Enter on the default button takes the name.
#[test]
fn a_busy_destination_takes_the_free_name_by_keyboard() {
    let mut s = open_clone("pty-clone-dest-busy-keys", |root, name, _| {
        std::fs::create_dir(root.join(name)).unwrap();
        std::fs::write(root.join(name).join("mine.txt"), "mine\n").unwrap();
    });
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the confirm popup", STEP, |x| {
            x.contains("The folder is not empty")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").unwrap();
    wait_clone_tab(&mut s.term);
    let free = s.root.path().join(format!("{}-2", s.name));
    assert!(free.join(".git").is_dir(), "no clone at {}", free.display());
}

/// Mouse: the folder already is a clone of this URL, so the popup offers to
/// open it; clicking opens the tab and clones nothing.
#[test]
fn an_existing_clone_of_the_url_is_opened_not_cloned_again() {
    let mut s = open_clone("pty-clone-dest-same", |root, name, source| {
        git(
            root,
            &["clone", "-q", &typed(source.root()), &format!("./{name}")],
        );
    });
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the confirm popup", STEP, |x| {
            x.contains("Already cloned here") && x.contains("Open it")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    click_label(&mut s.term, "Open it");
    wait_clone_tab(&mut s.term);
    assert!(!s.root.path().join(format!("{}-2", s.name)).exists());
    let _ = &s.source;
}

/// Back, by click and by Esc, closes the popup, keeps the dialog and clones
/// nothing.
#[test]
fn back_closes_the_popup_by_click_and_by_escape() {
    let mut s = open_clone("pty-clone-dest-back", |root, name, _| {
        std::fs::create_dir(root.join(name)).unwrap();
        std::fs::write(root.join(name).join("mine.txt"), "mine\n").unwrap();
    });
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the confirm popup", STEP, |x| {
            x.contains("The folder is not empty")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    click_label(&mut s.term, "Back");
    s.term
        .wait_for("the form again", STEP, |x| {
            !x.contains("The folder is not empty") && x.contains("Clone a repository")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the popup again", STEP, |x| {
            x.contains("The folder is not empty")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Esc").unwrap();
    s.term
        .wait_for("the form again", STEP, |x| {
            !x.contains("The folder is not empty") && x.contains("Clone a repository")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!s.root.path().join(format!("{}-2", s.name)).exists());
}
