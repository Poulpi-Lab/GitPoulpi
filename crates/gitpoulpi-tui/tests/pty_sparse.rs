//! Sparse checkout end to end through a pty by `xtask::pty`: the real
//! binary, daemon and git. No network: the clone is from a fixture
//! repository's path.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, and every wait describes the whole state
//! it is waiting for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 180;
const ROWS: u16 = 45;

/// Forward slashes: typing a literal `\` through ConPTY's synthetic input
/// does not reach the app (see `pty_ui.rs`), and Windows accepts `/`.
fn typed(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn launch(
    repo: &Path,
    session: &str,
    config_toml: &str,
    wait_for: &str,
) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(config.path().join("config.toml"), config_toml).expect("seed config.toml");
    let term = common::launch(repo)
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .spawn();
    let needle = wait_for.to_string();
    term.wait_for("the first window", STARTUP, move |s| s.contains(&needle))
        .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

fn palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

fn layered() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("README.md", "root\n", "sparse-test-root")
        .unwrap()
        .commit_file("src/main.rs", "fn main() {}\n", "sparse-test-src")
        .unwrap()
        .commit_file("docs/guide.md", "guide\n", "sparse-test-docs")
        .unwrap()
        .commit_file("lib/util.rs", "// util\n", "sparse-test-lib")
        .unwrap()
        .build()
}

/// `Ctrl+O`, `c`: the Clone dialog. Ticking "Sparse checkout" moves the
/// keyboard to its directories field; with `src` typed, Enter clones only
/// `src` (and the root files) - the history is whole, `docs` and `lib` are
/// not on disk - and the new tab says "sparse" in its footer.
#[test]
fn clone_with_sparse_checks_out_only_the_chosen_directory() {
    let source = layered();
    let start = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "sparse-clone-start-commit")
        .unwrap()
        .build();
    let root = tempfile::tempdir().unwrap();
    let (mut term, _config) = launch(
        start.root(),
        "pty-sparse-clone",
        &format!("[repos]\nclone_root = '{}'\n", root.path().display()),
        "sparse-clone-start-commit",
    );

    term.send_keys("C-o").expect("go to the management tab");
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Clone")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("c").expect("Clone");
    term.wait_for("the Clone dialog", STEP, |s| {
        s.contains("Clone a repository") && s.contains("Sparse checkout (only some directories)")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_text(&typed(source.root()))
        .expect("type the source path");
    let name = source
        .root()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let expect_name = name.clone();
    term.wait_for("the full path", STEP, move |s| {
        s.contains("Full path:") && s.contains(&expect_name)
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Url -> Name -> Shallow -> Sparse, then Space ticks it.
    for _ in 0..3 {
        term.send_keys("Tab").unwrap();
    }
    term.send_text(" ").expect("tick sparse checkout");
    term.wait_for("the directories field, and Clone refusing", STEP, |s| {
        s.contains("[x] Sparse checkout") && s.contains("enter at least one directory")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("src").expect("type the directory");
    term.wait_for("Clone live again", STEP, |s| {
        !s.contains("enter at least one directory")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("clone it");

    term.wait_for("the clone open as a sparse tab", STARTUP, |s| {
        s.contains("sparse-test-lib") && !s.contains("Clone a repository") && s.contains("sparse")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let dest = root.path().join(&name);
    assert!(dest.join("src/main.rs").is_file(), "src missing");
    assert!(dest.join("README.md").is_file(), "root files stay");
    assert!(!dest.join("docs").exists(), "docs must not be checked out");
    assert!(!dest.join("lib").exists(), "lib must not be checked out");
}

/// The palette's "Repository: Sparse Checkout...": on a repository that is
/// sparse on `src`, the dialog shows `src`; adding `docs` checks it out;
/// Disable brings everything back. Each submit closes the dialog and the
/// status line answers.
#[test]
fn the_sparse_dialog_adds_a_directory_and_disables() {
    let repo = layered();
    repo.git(&["sparse-checkout", "set", "--cone", "src"]);
    assert!(!repo.root().join("docs").exists());
    let (mut term, _config) = launch(repo.root(), "pty-sparse-dialog", "", "sparse-test-lib");
    term.wait_for("the sparse badge", STEP, |s| s.contains("sparse"))
        .unwrap_or_else(|e| panic!("{e}"));

    palette(&mut term, "Sparse Check", "Repository: Sparse Checkout");
    term.wait_for("the dialog on the current directories", STEP, |s| {
        s.contains("Sparse checkout is on") && s.contains("Disable (full checkout)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("End").unwrap();
    term.send_text(" docs").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the status line", STEP, |s| {
        s.contains("sparse checkout applied: src docs") && !s.contains("Disable (full checkout)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(repo.root().join("docs/guide.md").is_file());
    assert!(!repo.root().join("lib").exists());

    // The keyboard twin of the Disable button: Tab, Tab, Enter.
    palette(&mut term, "Sparse Check", "Repository: Sparse Checkout");
    term.wait_for("the dialog again", STEP, |s| {
        s.contains("Disable (full checkout)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Tab").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("everything checked out again", STEP, |s| {
        s.contains("sparse checkout disabled")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(repo.root().join("lib/util.rs").is_file());
    assert!(repo.root().join("src/main.rs").is_file());
}

/// `Ctrl+N` over the dialog badges its field and buttons; the Disable
/// button's badge presses it, like a click.
#[test]
fn a_badge_presses_the_sparse_dialogs_disable_button() {
    let repo = layered();
    repo.git(&["sparse-checkout", "set", "--cone", "src"]);
    let (mut term, _config) = launch(repo.root(), "pty-sparse-badge", "", "sparse-test-lib");
    term.wait_for("the sparse badge", STEP, |s| s.contains("sparse"))
        .unwrap_or_else(|e| panic!("{e}"));
    palette(&mut term, "Sparse Check", "Repository: Sparse Checkout");
    term.wait_for("the dialog", STEP, |s| {
        s.contains("Disable (full checkout)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The badge sits on the button's left edge, over its first letters.
    let badge = |s: &xtask::pty::Screen| -> Option<char> {
        let (col, row) = s.find("sable (full checkout)")?;
        let line: Vec<char> = s.line(row).chars().collect();
        let col = usize::from(col);
        (col.saturating_sub(4)..col).rev().find_map(|i| {
            (i + 2 < line.len()
                && line[i] == ' '
                && line[i + 2] == ' '
                && line[i + 1].is_ascii_alphanumeric())
            .then(|| line[i + 1])
        })
    };
    term.send_keys("C-n").unwrap();
    term.wait_for("the badges", STEP, |s| badge(s).is_some())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let key = badge(&term.screen()).expect("the Disable badge");
    term.send_text(&key.to_string()).unwrap();
    term.wait_for("everything checked out again", STEP, |s| {
        s.contains("sparse checkout disabled")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(repo.root().join("lib/util.rs").is_file());
}
