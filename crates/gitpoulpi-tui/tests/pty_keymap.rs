//! The keymap registry (`src/keymap/`), end to end: the window chords
//! and the Ctrl+G / Ctrl+N prefixes still do what they always did now
//! that they are resolved through the registry. These were green before
//! the refactor and are the proof it changed no behavior.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded
//! coordinate, and every wait describes the whole state it waits for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "Message";

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(200, 40)
        .no_exit_pause()
        .ready_when("the working-changes pane", |s| {
            s.contains("Unstaged") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start()
}

/// One commit and an unstaged edit.
fn dirty_fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "keymap-probe-commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .build()
}

/// In a narrow terminal the Ctrl+G hint wraps onto more rows instead of being
/// cut at the right edge: the last key (`d=delete`) is still on screen.
#[test]
fn ctrl_g_hint_wraps_in_a_narrow_terminal() {
    let repo = dirty_fixture();
    let (mut term, _c) = common::launch(repo.root())
        .session("pty-keymap-git-wrap")
        .size(80, 30)
        .no_exit_pause()
        .ready_when("the working-changes pane", |s| s.contains("Unstaged"))
        .mouse_capture()
        .start();
    term.send_keys("C-g").unwrap();
    term.wait_for("the whole Ctrl+G hint over several rows", STEP, |s| {
        s.contains("s=stage") && s.contains("(Esc to cancel)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// `Ctrl+G` shows its hint; `s` with no Status selection says so; `a`
/// stages everything (the leader table, resolved through the registry).
#[test]
fn ctrl_g_prefix_hint_and_second_keys() {
    let repo = dirty_fixture();
    let (mut term, _c) = launch(repo.root(), "pty-keymap-git");
    term.send_keys("C-g").unwrap();
    term.wait_for("the Ctrl+G hint", STEP, |s| {
        s.contains("Ctrl+G:") && s.contains("s=stage")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // A key the table binds, with nothing to act on: its message.
    term.send_text("o").unwrap();
    term.wait_for("the no-stash message", STEP, |s| {
        s.contains("no stash to pop")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-g").unwrap();
    term.send_text("s").unwrap();
    term.wait_for("the file staged by Ctrl+G s", STEP, |s| {
        s.contains("Staged Files (1)") && s.contains("Unstaged Files (0)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-g").unwrap();
    term.send_text("u").unwrap();
    term.wait_for("the file unstaged by Ctrl+G u", STEP, |s| {
        s.contains("Staged Files (0)") && s.contains("Unstaged Files (1)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-g").unwrap();
    term.send_text("a").unwrap();
    term.wait_for("everything staged by Ctrl+G a", STEP, |s| {
        s.contains("Staged Files (1)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// `Ctrl+N Shift+H` / `Ctrl+N Shift+L` collapse and restore the side panes.
#[test]
fn ctrl_n_prefix_collapses_panes() {
    let repo = dirty_fixture();
    let (mut term, _c) = launch(repo.root(), "pty-keymap-nav");
    term.send_keys("C-n").unwrap();
    term.wait_for("the Ctrl+N hint", STEP, |s| {
        s.contains("Ctrl+N:") && s.contains("collapse Refs")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("h").unwrap();
    term.wait_for("the Refs pane collapsed", STEP, |s| {
        s.contains("Refs pane collapsed")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-n").unwrap();
    term.send_text("H").unwrap();
    term.wait_for("the Refs pane restored", STEP, |s| {
        s.contains("Refs pane restored")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-n").unwrap();
    term.send_text("L").unwrap();
    term.wait_for("the Status pane collapsed", STEP, |s| {
        s.contains("Status pane collapsed")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// The picker chords after `Ctrl+N`: `l` the language picker, `t` the theme
/// picker, `Tab` the tab picker, `Shift+L` collapses the Status pane. Each
/// picker is closed with Esc before the next chord.
#[test]
fn ctrl_n_l_t_tab_open_the_pickers_and_shift_l_collapses_status() {
    let repo = dirty_fixture();
    let (mut term, _c) = launch(repo.root(), "pty-keymap-nav-pickers");
    for (key, title, gone) in [
        ("l", "Change Language (", "Change Language ("),
        (
            "t",
            "System (follows terminal)",
            "System (follows terminal)",
        ),
        ("Tab", "Select Tab", "Select Tab"),
    ] {
        term.send_keys("C-n").unwrap();
        term.wait_for_text("select tab", STEP).unwrap();
        term.send_keys(key).unwrap();
        term.wait_for(title, STEP, |s| s.contains(title))
            .unwrap_or_else(|e| panic!("Ctrl+N {key}: {e}"));
        term.send_keys("Escape").unwrap();
        term.wait_for_text_to_go(gone, STEP)
            .unwrap_or_else(|e| panic!("{e}"));
    }
    term.send_keys("C-n").unwrap();
    term.wait_for_text("select tab", STEP).unwrap();
    term.send_keys("L").unwrap();
    term.wait_for("the Status pane collapsed", STEP, |s| {
        s.contains("Status pane collapsed")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// `Ctrl+O` goes Home; `Ctrl+P` opens the palette.
#[test]
fn ctrl_o_goes_home_and_ctrl_p_opens_the_palette() {
    let repo = dirty_fixture();
    let (mut term, _c) = launch(repo.root(), "pty-keymap-window");
    term.send_keys("C-p").unwrap();
    term.send_text("Fetch All").unwrap();
    term.wait_for("the palette listing Fetch", STEP, |s| {
        s.contains("Git: Fetch All Remotes")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Esc").unwrap();
    term.wait_for("the palette closed", STEP, |s| {
        !s.contains("Git: Fetch All Remotes")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-o").unwrap();
    term.wait_for("Home", STEP, |s| s.contains("Repository Management"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

// `[keybindings]` rebinds the prefixes live.

fn launch_with_config(
    repo: &Path,
    session: &str,
    config_toml: &str,
) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .config(config_toml)
        .size(200, 40)
        .no_exit_pause()
        .ready_when("the working-changes pane", |s| {
            s.contains("Unstaged") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start()
}

/// `git_prefix = "Ctrl+B"`: `C-b s` stages, and the old `C-g` opens
/// nothing (no hint, nothing staged).
#[test]
fn rebound_git_prefix_works_and_the_old_one_does_not() {
    let repo = dirty_fixture();
    let (mut term, _c) = launch_with_config(
        repo.root(),
        "pty-keymap-rebind-git",
        "[keybindings.global]\ngit_prefix = \"Ctrl+B\"\n",
    );
    term.send_keys("C-g").unwrap();
    term.send_text("s").unwrap();
    // `s` fell through as an ordinary key; give it time to (not) act,
    // then prove the old prefix is dead by the new one still working.
    term.send_keys("C-b").unwrap();
    term.wait_for("the Ctrl+B hint", STEP, |s| {
        s.contains("s=stage") && !s.contains("Staged Files (1)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("s").unwrap();
    term.wait_for("the file staged by Ctrl+B s", STEP, |s| {
        s.contains("Staged Files (1)") && s.contains("Unstaged Files (0)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// `nav_prefix = "Alt+n"`: `M-n h` collapses Refs.
#[test]
fn rebound_nav_prefix_works() {
    let repo = dirty_fixture();
    let (mut term, _c) = launch_with_config(
        repo.root(),
        "pty-keymap-rebind-nav",
        "[keybindings.global]\nnav_prefix = \"Alt+n\"\n",
    );
    term.send_keys("C-n").unwrap();
    term.send_text("H").unwrap();
    term.send_keys("M-n").unwrap();
    term.send_text("H").unwrap();
    term.wait_for("the Refs pane collapsed by Alt+N h", STEP, |s| {
        s.contains("Refs pane collapsed")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// A typo is reported on the status line and the default stays.
#[test]
fn a_bad_binding_is_reported_and_the_default_stays() {
    let repo = dirty_fixture();
    let (mut term, _c) = launch_with_config(
        repo.root(),
        "pty-keymap-bad",
        "[keybindings.global]\ngit_prefix = \"Ctrl+Nope\"\n",
    );
    term.wait_for("the ignored-entry message", STEP, |s| {
        s.contains("keybindings: 1 entry ignored")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-g").unwrap();
    term.send_text("s").unwrap();
    term.wait_for("the file staged by the default Ctrl+G s", STEP, |s| {
        s.contains("Staged Files (1)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

// Every display surface shows the effective binding.

const REBOUND: &str = "[keybindings.global]\ngit_prefix = \"Ctrl+B\"\n";

/// The palette's right column follows the rebinding.
#[test]
fn palette_shows_the_rebound_prefix() {
    let repo = dirty_fixture();
    let (mut term, _c) = launch_with_config(repo.root(), "pty-keymap-palette", REBOUND);
    term.send_keys("C-p").unwrap();
    term.send_text("stage selected").unwrap();
    term.wait_for("the stage row with the new binding", STEP, |s| {
        s.contains("Ctrl+B s") && !s.contains("Ctrl+G s")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// The help overlay's section title and key column follow it too.
#[test]
fn help_shows_the_rebound_prefix() {
    let repo = dirty_fixture();
    let (mut term, _c) = launch_with_config(repo.root(), "pty-keymap-help", REBOUND);
    term.send_text("?").unwrap();
    term.wait_for("the help overlay", STEP, |s| s.contains("Keybindings"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("Git actions").unwrap();
    term.wait_for("the Git actions section under Ctrl+B", STEP, |s| {
        s.contains("Git actions - Ctrl+B, then:")
            && s.contains("Ctrl+B s")
            && !s.contains("Ctrl+G s")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// The pending-prefix footer names the new prefix; the bottom hint line
/// does too.
#[test]
fn prefix_footer_shows_the_rebound_prefix() {
    let repo = dirty_fixture();
    let (mut term, _c) = launch_with_config(repo.root(), "pty-keymap-footer", REBOUND);
    term.wait_for("the idle hint line with Ctrl+B", STEP, |s| {
        s.contains("Ctrl+B: git") && !s.contains("Ctrl+G: git")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-b").unwrap();
    term.wait_for("the Ctrl+B footer", STEP, |s| {
        s.contains("Ctrl+B: s=stage") && !s.contains("Ctrl+G:")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}
