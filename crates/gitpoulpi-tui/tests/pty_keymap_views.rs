//! Custom keybindings, end to end through a pty: a Diff shortcut, an editor
//! shortcut, an editor-prompt key and two popup shortcuts (the rebase view's
//! reset, the Init dialog's browse) rebound through `[keybindings]`, each
//! seen working and shown where the app prints it. A temp
//! `GITPOULPI_CONFIG_DIR` per test; nothing is located by a hard-coded
//! coordinate.

mod common;

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "Message";

fn launch(
    repo: &Path,
    session: &str,
    config_toml: &str,
    ready: &str,
) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(config.path().join("config.toml"), config_toml).unwrap();
    let term = common::launch(repo)
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    let ready = ready.to_string();
    term.wait_for("the first window", STARTUP, move |s| {
        s.contains(&ready) && s.contains(LOG_HEADER)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_mouse_capture(STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

/// `many.txt`: 30 lines, three changed far enough apart to be three hunks.
fn three_hunks() -> FixtureRepo {
    let original: Vec<String> = (1..=30).map(|n| format!("l{n}")).collect();
    let mut edited = original.clone();
    edited[1] = "CHANGED-TOP".to_string();
    edited[14] = "CHANGED-MIDDLE".to_string();
    edited[27] = "CHANGED-BOTTOM".to_string();
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("many.txt", &(original.join("\n") + "\n"), "add many")
        .unwrap()
        .write_worktree("many.txt", (edited.join("\n") + "\n").as_bytes())
        .unwrap()
        .build()
}

/// Selects the WIP row, clicks `file` in the Unstaged half and waits for
/// its diff.
fn open_unstaged_diff(term: &mut Terminal, file: &str, marker: &str) {
    let (col, row) = term
        .wait_for_text("\u{25CC}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
    let (col, row) = term
        .wait_for_text(&format!("\u{270E} {file}"), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).unwrap();
    let marker = marker.to_string();
    let title = format!("{file} - unstaged");
    term.wait_for("the unstaged diff", STEP, move |s| {
        s.contains(&title) && s.contains(&marker)
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn staged_diff(repo: &Path) -> String {
    let out = Command::new("git")
        .args(["diff", "--cached"])
        .current_dir(repo)
        .output()
        .expect("run git");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// `diff.stage_hunk = "a"`: `a` stages the hunk under the cursor (the
/// bottom one, reached with End then p) and only that one, and the help
/// overlay names `a` for it.
#[test]
fn a_rebound_diff_shortcut_stages_the_hunk_and_shows_in_help() {
    let repo = three_hunks();
    let (mut term, _c) = launch(
        repo.root(),
        "pty-kbv-diff",
        "[keybindings.diff]\nstage_hunk = \"a\"\n",
        "add many",
    );
    open_unstaged_diff(&mut term, "many.txt", "CHANGED-BOTTOM");
    term.send_keys("End p").unwrap();
    term.send_text("a").unwrap();
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let staged = staged_diff(repo.root());
        if staged.contains("CHANGED-BOTTOM") {
            assert!(
                !staged.contains("CHANGED-TOP") && !staged.contains("CHANGED-MIDDLE"),
                "only the hunk under the cursor is staged:\n{staged}"
            );
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "`a` never staged the hunk: {}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    term.send_text("?").unwrap();
    term.wait_for("the help overlay", STEP, |s| s.contains("Keybindings"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("stage/unstage the hunk").unwrap();
    let line = format!("a{}stage/unstage the hunk", " ".repeat(15));
    term.wait_for("the rebound key in the help", STEP, move |s| {
        s.contains(&line)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// `editor.save = "Alt+s"` and `editor_prompt.discard = "x"`: `e` opens
/// the editor from the diff, Ctrl+S is inert, Alt+S writes the file, and
/// the unsaved-changes prompt names (and takes) the rebound Discard key.
#[test]
fn rebound_editor_shortcuts_save_and_show_in_the_prompt() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("notes.txt", "alpha\nbeta\n", "base")
        .unwrap()
        .write_worktree("notes.txt", b"alpha\nbeta\ngamma\n")
        .unwrap()
        .build();
    let path = repo.root().join("notes.txt");
    let (mut term, _c) = launch(
        repo.root(),
        "pty-kbv-editor",
        "[keybindings.editor]\nsave = \"Alt+s\"\n[keybindings.editor_prompt]\ndiscard = \"x\"\n",
        "base",
    );
    open_unstaged_diff(&mut term, "notes.txt", "gamma");
    term.send_text("e").unwrap();
    term.wait_for("the editor", STEP, |s| {
        s.contains("Edit - notes.txt @ working tree") && s.contains("alpha")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("ONE ").unwrap();
    term.wait_for("the dirty marker", STEP, |s| {
        s.contains("ONE alpha") && s.contains("\u{25CF} modified")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    std::thread::sleep(Duration::from_millis(800));
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "alpha\nbeta\ngamma\n",
        "Ctrl+S must be inert once the key is rebound"
    );
    term.send_keys("M-s").unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while std::fs::read_to_string(&path).unwrap() != "ONE alpha\nbeta\ngamma\n" {
        assert!(
            std::time::Instant::now() < deadline,
            "Alt+S never saved: {}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    // More typing, then Esc: the prompt shows the rebound Discard key.
    term.send_text("TWO ").unwrap();
    term.wait_for("the new edit", STEP, |s| s.contains("ONE TWO alpha"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Esc").unwrap();
    term.wait_for("the prompt with the rebound key", STEP, |s| {
        s.contains("Save changes to notes.txt?") && s.contains("Discard (X)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("x").unwrap();
    term.wait_for("the editor closed", STEP, |s| {
        !s.contains("Save changes to") && !s.contains("ONE TWO")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "ONE alpha\nbeta\ngamma\n",
        "Discard must not write"
    );
    term.stop().unwrap();
}

/// `rebase.reset = "Alt+r"` in the interactive rebase view: the footer
/// names it, Alt+R restores the plan and Ctrl+R no longer does.
#[test]
fn a_rebound_rebase_reset_works_and_shows_in_the_footer() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "c1 base")
        .unwrap()
        .commit_file("b.txt", "b\n", "c2 second")
        .unwrap()
        .commit_file("c.txt", "c\n", "c3 third")
        .unwrap()
        .build();
    let (mut term, _c) = launch(
        repo.root(),
        "pty-kbv-rebase",
        "[keybindings.rebase]\nreset = \"Alt+r\"\n",
        "c3 third",
    );
    let (col, row) = term.wait_for_text("c1 base", STEP).unwrap();
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the commit menu", STEP, |s| {
            s.contains("Interactively rebase from here...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Interactively rebase from here...").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the rebase view and its rebound footer", STEP, |s| {
        s.contains("Start rebase") && s.contains("Alt+R reset")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("d").unwrap();
    term.wait_for("a dropped row", STEP, |s| s.contains("Drop   \u{25BE}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-r").unwrap();
    std::thread::sleep(Duration::from_millis(800));
    assert!(
        term.screen().contains("Drop   \u{25BE}"),
        "Ctrl+R must be inert once the key is rebound"
    );
    term.send_keys("M-r").unwrap();
    term.wait_for("the plan restored", STEP, |s| {
        !s.contains("Drop   \u{25BE}") && s.contains("Start rebase")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// `dialog.browse = "Alt+b"`: from the Init dialog, Alt+B opens the
/// folder picker, and the dialog's footer names the key.
#[test]
fn a_rebound_browse_chord_opens_the_picker_from_init() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "kbv-init-base")
        .unwrap()
        .build();
    let (mut term, _c) = launch(
        repo.root(),
        "pty-kbv-init",
        "[keybindings.dialog]\nbrowse = \"Alt+b\"\n",
        "kbv-init-base",
    );
    term.send_keys("C-o").unwrap();
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Init")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("i").unwrap();
    term.wait_for("the Init dialog with the rebound hint", STEP, |s| {
        s.contains("Initialize a repository") && s.contains("Alt+B browse")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-b").unwrap();
    std::thread::sleep(Duration::from_millis(800));
    assert!(!term.screen().contains("QUICK ACCESS"), "Ctrl+B is inert");
    term.send_keys("M-b").unwrap();
    term.wait_for("the folder picker", STEP, |s| {
        s.contains("QUICK ACCESS") || s.contains("FOLDERS")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
