//! Settings > Keyboard & mouse, end to end: rebind a key with the
//! capture popup and see it work, resolve a conflict with Replace, reset
//! one row and then everything. A temp `GITPOULPI_CONFIG_DIR` per test;
//! nothing is located by a hard-coded coordinate.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "Message";

fn launch(repo: &Path, session: &str, config_toml: &str) -> (Terminal, tempfile::TempDir) {
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

fn dirty_fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "keybindings-probe-commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .build()
}

/// Opens Settings on the Keyboard & mouse page through the palette.
fn open_page(term: &mut Terminal) {
    term.send_keys("C-p").unwrap();
    term.send_text("App: Keybindings").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("App: Keybindings...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the Keyboard & mouse page", STEP, |s| {
        s.contains("Keyboard & mouse") && s.contains("Command Palette")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Filters the list with the page's search field and leaves it.
fn search(term: &mut Terminal, text: &str, expect: &str) {
    term.send_text("/").unwrap();
    term.send_text(text).unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the filtered list", STEP, |s| s.contains(expect))
        .unwrap_or_else(|e| panic!("{e}"));
}

fn config_text(dir: &tempfile::TempDir) -> String {
    std::fs::read_to_string(dir.path().join("config.toml")).unwrap_or_default()
}

fn wait_config(dir: &tempfile::TempDir, what: &str, ok: impl Fn(&str) -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while std::time::Instant::now() < deadline {
        if ok(&config_text(dir)) {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("config.toml never showed {what}: {}", config_text(dir));
}

/// Rebind `Stage Selected File` to `e` (a clash with `Edit Commit
/// Message`), Replace, and the new key works from the main window.
#[test]
fn rebinding_with_the_capture_popup_replaces_and_applies_live() {
    let repo = dirty_fixture();
    let (mut term, config) = launch(repo.root(), "pty-kb-rebind", "");
    open_page(&mut term);
    search(&mut term, "stage selected", "Stage Selected File");
    term.send_keys("Enter").unwrap();
    term.wait_for("the capture popup", STEP, |s| {
        s.contains("Press the new key")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("e").unwrap();
    term.wait_for("the conflict line and Replace", STEP, |s| {
        s.contains("already Edit Commit Message") && s.contains("Replace")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the saved message", STEP, |s| s.contains("saved to"))
        .unwrap_or_else(|e| panic!("{e}"));
    wait_config(&config, "the new binding", |t| {
        t.contains("stage_file = \"e\"") && t.contains("edit_message = []")
    });
    // Leave Settings (Esc clears the search, then closes). Esc and the
    // next byte too close together would read as one Alt chord.
    term.send_keys("Esc").unwrap();
    term.wait_for("the search cleared", STEP, |s| {
        s.contains("label, id or key")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Esc").unwrap();
    term.wait_for("the main window", STEP, |s| {
        s.contains("Unstaged Files (1)") && !s.contains("Keyboard & mouse")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-g").unwrap();
    term.send_text("e").unwrap();
    term.wait_for("the file staged by Ctrl+G e", STEP, |s| {
        s.contains("Staged Files (1)") && s.contains("Unstaged Files (0)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// A key already taken in `global` names its owner; Cancel changes
/// nothing. Then `r` resets a modified row and `R` + `y` resets all.
#[test]
fn global_conflict_cancel_row_reset_and_reset_all() {
    let repo = dirty_fixture();
    let (mut term, config) = launch(
        repo.root(),
        "pty-kb-reset",
        "[keybindings.git_prefix]\npull = \"Shift+X\"\nstash_pop = \"Shift+Y\"\n",
    );
    open_page(&mut term);
    // Capture C-p on a global action: the conflict names the palette.
    search(&mut term, "refresh", "Refresh");
    term.send_keys("Enter").unwrap();
    term.wait_for("the capture popup", STEP, |s| {
        s.contains("Press the new key")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.wait_for("the conflict naming the palette", STEP, |s| {
        s.contains("already Command Palette")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Esc").unwrap();
    term.wait_for("the popup closed", STEP, |s| {
        !s.contains("already Command Palette")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!config_text(&config).contains("refresh"));
    // Reset one row.
    term.send_keys("Esc").unwrap();
    term.wait_for("the search cleared", STEP, |s| {
        s.contains("label, id or key")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    search(&mut term, "git_prefix.pull", "Shift+X");
    term.send_text("r").unwrap();
    wait_config(&config, "pull reset", |t| {
        !t.contains("pull =") && t.contains("stash_pop")
    });
    // Reset everything, after the confirmation.
    term.send_text("R").unwrap();
    term.wait_for("the confirmation", STEP, |s| {
        s.contains("Every shortcut goes back to its default")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("y").unwrap();
    wait_config(&config, "no [keybindings]", |t| !t.contains("keybindings"));
    term.stop().unwrap();
}

// Pane shortcuts and the plain global keys follow the config.

/// `log.view_all_files = "Shift+V"`: `V` ticks View all files and the old
/// `a` does nothing; `global.find_in_log = "Shift+F"`: `F` opens the find
/// bar and the old `f` does not.
#[test]
fn log_and_global_plain_keys_follow_the_config() {
    let repo = dirty_fixture();
    let (mut term, _c) = launch(
        repo.root(),
        "pty-kb-log",
        "[keybindings.log]\nview_all_files = \"Shift+V\"\n[keybindings.global]\nfind_in_log = \"Shift+F\"\n",
    );
    // `a` is inert now; if it had ticked the box, `V` would untick it.
    term.send_text("a").unwrap();
    term.send_text("V").unwrap();
    term.wait_for("View all files ticked by Shift+V alone", STEP, |s| {
        s.contains("[x] View all files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("V").unwrap();
    term.wait_for("View all files unticked", STEP, |s| {
        s.contains("[ ] View all files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // `f` is inert: were it to open the bar, the `F` after it would be
    // typed into the query and nothing would match.
    term.send_text("f").unwrap();
    term.send_text("F").unwrap();
    term.send_text("keybindings-probe").unwrap();
    term.wait_for("the find bar found the commit", STEP, |s| {
        s.contains("1 of 1")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// `home.favorite = "s"`: `s` stars the selected repository on the Home
/// tab; the old `f` does not.
#[test]
fn home_favorite_follows_the_config() {
    let repo = dirty_fixture();
    let other = dirty_fixture();
    let (mut term, config) = launch(
        repo.root(),
        "pty-kb-home",
        "[keybindings.home]\nfavorite = \"s\"\n",
    );
    std::fs::write(
        config.path().join("recent.toml"),
        format!(
            "[[repos]]\npath = '{}'\nname = 'bravo-repo'\nlast_opened = 1\n",
            other.root().display()
        ),
    )
    .unwrap();
    term.send_keys("C-o").unwrap();
    term.wait_for("Home with the recent repository", STEP, |s| {
        s.contains("Repository Management") && s.contains("bravo-repo")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("f").unwrap();
    term.send_text("s").unwrap();
    term.wait_for("one star, from `s` alone", STEP, |s| {
        s.contains("FAVORITES  1")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// `home.favorite = "s"`: the palette's "Add or Remove Selected from
/// Favorites" still stars the repository.
#[test]
fn home_palette_favorite_works_after_the_key_is_rebound() {
    let repo = dirty_fixture();
    let other = dirty_fixture();
    let (mut term, config) = launch(
        repo.root(),
        "pty-kb-home-palette",
        "[keybindings.home]\nfavorite = \"s\"\n",
    );
    std::fs::write(
        config.path().join("recent.toml"),
        format!(
            "[[repos]]\npath = '{}'\nname = 'bravo-repo'\nlast_opened = 1\n",
            other.root().display()
        ),
    )
    .unwrap();
    term.send_keys("C-o").unwrap();
    term.wait_for("Home with the recent repository", STEP, |s| {
        s.contains("Repository Management") && s.contains("bravo-repo")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("Add or Remove Selected from Fav").unwrap();
    term.wait_for("the command listed", STEP, |s| {
        s.contains("Repositories: Add or Remove Selected")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("one star, from the palette", STEP, |s| {
        s.contains("FAVORITES  1")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// Obsidian-like conflicts: a clashing key is drawn red, `c` lists the
/// action with every action it clashes with, Esc shows them all again.
#[test]
fn a_clashing_key_is_red_and_c_lists_its_conflicts() {
    let repo = dirty_fixture();
    let (mut term, _config) = launch(
        repo.root(),
        "pty-kb-conflicts",
        // Stage toggle on `q` hides the global Quit (same chord, child context).
        "[keybindings.status]\nstage_toggle = \"q\"\n",
    );
    open_page(&mut term);
    let s = snap(&term);
    let (cc, cr) = s.find("[Ctrl+P]").expect("a plain chip");
    let plain = s.cell(cc + 1, cr).unwrap().fg;
    search(&mut term, "status.stage_toggle", "[q]");
    let s = snap(&term);
    let (pc, pr) = s.find("[q]").expect("the clashing chip");
    assert_ne!(
        s.cell(pc + 1, pr).unwrap().fg,
        plain,
        "a clash is not the plain color"
    );
    term.send_text("c").unwrap();
    let s = term
        .wait_for("the action and the one it clashes with", STEP, |s| {
            s.contains("Conflicts of") && s.find_all("[q]").len() >= 2
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !s.contains("Command Palette"),
        "only the parties are listed"
    );
    term.send_keys("Esc").unwrap();
    term.wait_for("the search filter back", STEP, |s| {
        !s.contains("Conflicts of") && s.contains("[q]")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A palette row's key button (and Alt+K on the selected row) opens Settings
/// > Keyboard & mouse filtered on that row's action.
#[test]
fn the_palette_key_button_and_alt_k_open_the_shortcut_filtered() {
    let repo = dirty_fixture();
    let (mut term, _config) = launch(repo.root(), "pty-kb-palette", "");
    let palette_on = |term: &mut Terminal| {
        term.send_keys("C-p").unwrap();
        term.send_text("Settings").unwrap();
        term.wait_for("the Settings row", STEP, |s| s.contains("App: Settings..."))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    };
    let filtered =
        |s: &xtask::pty::Screen| s.contains("Keyboard & mouse") && s.contains("global.settings");
    // Keyboard.
    palette_on(&mut term);
    term.send_keys("M-k").unwrap();
    term.wait_for("the page filtered on Settings", STEP, filtered)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Esc clears the search, then closes Settings.
    common::retry_until(
        &mut term,
        "Settings closed",
        6,
        Duration::from_secs(2),
        |t| t.send_keys("Escape").unwrap(),
        |s| s.contains("Unstaged") && !s.contains("Keyboard & mouse"),
    );
    // Mouse: the button at the end of the row.
    palette_on(&mut term);
    let (c, r) = snap(&term).find("[k]").expect("the key button on the row");
    term.click(c, r).unwrap();
    term.wait_for("the page filtered on Settings", STEP, filtered)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A command with no key of its own ("Repository: Change Profile...") gets the
/// key button; Alt+K opens its `palette.change_profile` row, a captured key
/// is written to `[keybindings.palette]`, the palette row then shows it, and
/// the key runs the command (with one profile it says why it cannot).
#[test]
fn a_palette_only_command_can_be_bound_and_the_key_runs_it() {
    let repo = dirty_fixture();
    let (mut term, config) = launch(repo.root(), "pty-kb-palette-cmd", "");
    term.send_keys("C-p").unwrap();
    term.send_text("Change Profile").unwrap();
    term.wait_for("the row and its key button", STEP, |s| {
        s.contains("Repository: Change Profile...")
            && s.contains("[k]")
            && s.contains("Alt+K to set its shortcut")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("M-k").unwrap();
    term.wait_for("the page filtered on the command", STEP, |s| {
        s.contains("Keyboard & mouse") && s.contains("palette.change_profile")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the capture popup", STEP, |s| {
        s.contains("Press the new key")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("M-j").unwrap();
    term.wait_for("the key to save", STEP, |s| s.contains("Key: Alt+J"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the saved message", STEP, |s| s.contains("saved to"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_config(&config, "the palette binding", |t| {
        t.contains("[keybindings.palette]") && t.contains("change_profile = \"Alt+J\"")
    });
    term.send_keys("Esc").unwrap();
    term.wait_for("the search cleared", STEP, |s| {
        s.contains("label, id or key")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Esc").unwrap();
    term.wait_for("the main window", STEP, |s| {
        s.contains("Unstaged Files (1)") && !s.contains("Keyboard & mouse")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The key runs it.
    term.send_keys("M-j").unwrap();
    term.wait_for("the command's reason", STEP, |s| {
        s.contains("add a second profile in Settings first")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The palette row shows the binding in place of the empty shortcut.
    term.send_keys("C-p").unwrap();
    term.send_text("Change Profile").unwrap();
    term.wait_for("the row showing Alt+J", STEP, |s| {
        s.contains("Repository: Change Profile...") && s.contains("Alt+J")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
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
