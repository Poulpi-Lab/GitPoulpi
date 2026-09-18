//! The Home tab (the `⌂` Repository Management view): the window-level
//! shortcuts that make sense without a repository work there, and the
//! row `Enter` opens is highlighted on arrival.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Color, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// Launches in `repo`, seeds a recent list, goes Home with `Ctrl+O`.
fn launch_home(
    repo: &Path,
    session: &str,
    recent: &[(&Path, &str)],
) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let mut toml = String::new();
    for (path, name) in recent {
        toml.push_str(&format!(
            "[[repos]]\npath = '{}'\nname = '{name}'\nlast_opened = 1\n\n",
            path.display()
        ));
    }
    std::fs::write(config.path().join("recent.toml"), toml).expect("seed recent.toml");
    let mut term = common::launch(repo)
        .session(session)
        .config_dir(config.path())
        .size(180, 45)
        .spawn();
    term.wait_for("the first window", STARTUP, |s| {
        s.contains("home-probe-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-o").expect("go Home");
    term.wait_for("Home", STEP, |s| s.contains("Repository Management"))
        .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

fn fixtures() -> (
    gitpoulpi_testkit::FixtureRepo,
    gitpoulpi_testkit::FixtureRepo,
) {
    let a = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "home-probe-commit")
        .unwrap()
        .build();
    let b = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("b.txt", "b\n", "other")
        .unwrap()
        .build();
    (a, b)
}

const SELECTION_BG: Color = Color::Rgb(52, 58, 90);

fn setup() -> (
    Terminal,
    tempfile::TempDir,
    (
        gitpoulpi_testkit::FixtureRepo,
        gitpoulpi_testkit::FixtureRepo,
    ),
) {
    let fx = fixtures();
    let (term, config) = launch_home(
        fx.0.root(),
        "pty-home-shortcuts",
        &[(fx.1.root(), "bravo-repo")],
    );
    (term, config, fx)
}

fn settings_open(s: &xtask::pty::Screen) -> bool {
    s.contains("Settings - Tab") && s.contains("Search settings")
}

/// `Ctrl+,` (a kitty-protocol terminal sends CSI u; a ConPTY, a win32 key event) opens Settings.
#[test]
fn ctrl_comma_opens_settings_from_home() {
    let (mut term, _c, _fx) = setup();
    term.send_keys("C-,").unwrap();
    term.wait_for("Settings over Home", STEP, settings_open)
        .unwrap_or_else(|e| panic!("{e}"));
}

/// `Ctrl+N ,` - the chord that works in every terminal - opens Settings,
/// and `Ctrl+N` shows its hint in the Home footer while pending.
#[test]
fn ctrl_n_chord_opens_settings_from_home() {
    let (mut term, _c, _fx) = setup();
    term.send_keys("C-n").unwrap();
    term.wait_for("the Ctrl+N hint on Home", STEP, |s| {
        s.contains("Ctrl+N:") && s.contains("=settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(",").unwrap();
    term.wait_for("Settings over Home", STEP, settings_open)
        .unwrap_or_else(|e| panic!("{e}"));
}

/// `Ctrl+N ?` opens the help overlay; `Ctrl+N` + the search badge puts the
/// cursor in the Home search.
#[test]
fn ctrl_n_chord_help_and_search_badge_work_on_home() {
    let (mut term, _c, _fx) = setup();
    term.send_keys("C-n").unwrap();
    term.send_text("?").unwrap();
    term.wait_for("help over Home", STEP, |s| s.contains("Keybindings"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("help closed", STEP, |s| !s.contains("Keybindings"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-n").unwrap();
    // The badge key is looked up in what the last frame drew, so the frame
    // with the prefix hint (`=settings`) and the badge has to be on screen
    // before `1` is sent; `" 1 "` alone also matches the ordinary page.
    term.wait_for("the prefix hint and the badges drawn", STEP, |s| {
        s.contains("<badge>=focus") && s.contains(" 1 ")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("1").unwrap();
    term.wait_for("the search focused", STEP, |s| {
        s.contains("Enter open best match")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// `Ctrl+G` and `Ctrl+W` need a repository: a short message, no crash.
#[test]
fn repository_only_chords_answer_on_home() {
    let (mut term, _c, _fx) = setup();
    term.send_keys("C-g").unwrap();
    term.wait_for("the Ctrl+G message", STEP, |s| {
        s.contains("need an open repository")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-w").unwrap();
    term.wait_for("the Ctrl+W message", STEP, |s| {
        s.contains("cannot be closed")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.wait_for("the palette still opens", STEP, |s| {
        s.contains("Command Palette")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The row `Enter` opens is highlighted on arrival, `Down` moves the
/// highlight, and it is there again after coming back from a tab.
#[test]
fn the_first_row_is_highlighted_on_arrival_and_down_moves_it() {
    let (mut term, _c, _fx) = setup();
    let highlighted_rows = |screen: &xtask::pty::Screen| -> Vec<u16> {
        let rows = 45u16;
        (0..rows)
            .filter(|&r| screen.cell(20, r).is_some_and(|c| c.bg == SELECTION_BG))
            .collect()
    };
    // Exactly one row is highlighted, and its left gutter carries no arrow
    // (the owner mistook it for a section fold arrow).
    let no_arrow_in_gutter = |s: &xtask::pty::Screen, row: u16| {
        (0..3).all(|x| {
            s.cell(x, row)
                .is_some_and(|c| c.text != "\u{25B8}" && c.text != "\u{25BE}")
        })
    };
    let one_highlighted = |s: &xtask::pty::Screen| {
        let rows = highlighted_rows(s);
        rows.len() == 1 && no_arrow_in_gutter(s, rows[0])
    };
    let screen = term
        .wait_for("the highlight on arrival, no arrow", STEP, |s| {
            one_highlighted(s)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let marker_row = highlighted_rows(&screen)[0];

    term.send_keys("Down").unwrap();
    let after = term
        .wait_for("the highlight moved", STEP, |s| {
            let rows = highlighted_rows(s);
            rows.len() == 1 && rows[0] != marker_row
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(highlighted_rows(&after).len(), 1);

    // Back from a tab: Esc leaves Home for the repository tab, the house
    // key returns, the highlight is there again.
    term.send_keys("Escape").unwrap();
    term.wait_for("the repository tab", STEP, |s| {
        s.contains("home-probe-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-o").unwrap();
    let back = term
        .wait_for("Home again, the highlight back", STEP, |s| {
            s.contains("Repository Management") && one_highlighted(s)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(highlighted_rows(&back).len(), 1);
}

/// In the command palette on Home, a command that needs a repository is
/// grayed out with "needs an open repository" (Enter reports it instead of
/// running it); one that does not (Settings) still runs.
#[test]
fn the_palette_on_home_grays_repository_commands() {
    let (mut term, _c, _fx) = setup();
    term.send_keys("C-p").unwrap();
    term.wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("Focus Log Pane").unwrap();
    term.wait_for("the reason under the list", STEP, |s| {
        s.contains("needs an open repository")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Enter on the grayed row does not run it: the reason is reported,
    // and we are still on Home.
    term.send_keys("Enter").unwrap();
    term.wait_for("the reason as a message", STEP, |s| {
        s.contains("Focus Log Pane: needs an open repository")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(term.screen().contains("Repository Management"));
    // Settings is not grayed: running it opens Settings over Home.
    term.send_keys("C-p").unwrap();
    term.wait_for("the palette again", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("Settings").unwrap();
    term.wait_for("the settings command", STEP, |s| {
        s.contains("App: Settings...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("Settings over Home", STEP, settings_open)
        .unwrap_or_else(|e| panic!("{e}"));
}

/// The Home tab shows the window's one Settings gear (on the tab row), not
/// a second one of its own, and that gear opens Settings.
#[test]
fn home_has_a_single_settings_gear() {
    let (a, b) = fixtures();
    let (mut term, _config) = launch_home(a.root(), "pty-home-gear", &[(b.root(), "other")]);
    let screen = term.screen();
    let gears = screen.text().matches('\u{2699}').count();
    assert_eq!(gears, 1, "one gear on Home:\n{}", screen.text());
    let (c, r) = screen.find("\u{2699}").unwrap();
    term.click(c, r).unwrap();
    term.wait_for("Settings open", STEP, |s| s.contains("Git & encoding"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// `Ctrl+N`, then waits for the frame that shows the prefix hint and the
/// badges: the badge key is looked up in what the last frame drew.
fn open_badges(term: &mut Terminal) {
    term.send_keys("C-n").unwrap();
    term.wait_for("the prefix hint and the badges", STEP, |s| {
        s.contains("<badge>=focus")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Badges on the Home tab after the search (`1`) and the four sections
/// (`2`-`5`): the toolbar buttons Browse `6`, Clone `7`, Init `8`, Collapse
/// all `9` and Expand all (the next free letter). Pressing one is a click on
/// that button.
#[test]
fn ctrl_n_badges_press_the_home_toolbar_buttons() {
    let (mut term, _c, _fx) = setup();
    // Clone `7`: the Clone dialog opens.
    open_badges(&mut term);
    term.send_text("7").unwrap();
    term.wait_for("the Clone dialog", STEP, |s| {
        s.contains("Clone a repository")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("the dialog closed", STEP, |s| {
        !s.contains("Clone a repository")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Init `8`: the Init dialog opens.
    open_badges(&mut term);
    term.send_text("8").unwrap();
    term.wait_for("the Init dialog", STEP, |s| {
        s.contains("Initialize a repository")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("the dialog closed", STEP, |s| {
        !s.contains("Initialize a repository")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Collapse all `9` hides the repository rows.
    assert!(term.screen().contains("bravo-repo"));
    open_badges(&mut term);
    term.send_text("9").unwrap();
    term.wait_for("the sections folded", STEP, |s| !s.contains("bravo-repo"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The letters after `9` skip every `Ctrl+N` key (`a` opens About), so
    // the unfolding is checked with the button's own key.
    term.send_text("+").unwrap();
    term.wait_for("the sections unfolded", STEP, |s| s.contains("bravo-repo"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// A section badge (`2`-`5`) lands the selection on that section's header:
/// the selection highlight moves to the All section's header (`5`).
#[test]
fn ctrl_n_section_badge_selects_that_section() {
    let (mut term, _c, _fx) = setup();
    let on_all_header = |s: &xtask::pty::Screen| {
        s.find("ALL REPOSITORIES")
            .is_some_and(|(_, row)| s.cell(20, row).is_some_and(|c| c.bg == SELECTION_BG))
    };
    assert!(!on_all_header(&term.screen()));
    open_badges(&mut term);
    term.send_text("5").unwrap();
    term.wait_for("the highlight on the All header", STEP, |s| {
        on_all_header(s)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// `Esc` cancels the badges without acting and without leaving Home: the
/// key that would have pressed Clone then does nothing.
#[test]
fn ctrl_n_escape_cancels_the_home_badges() {
    let (mut term, _c, _fx) = setup();
    open_badges(&mut term);
    term.send_keys("Escape").unwrap();
    term.wait_for("the hint gone", STEP, |s| !s.contains("<badge>=focus"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("7").unwrap();
    std::thread::sleep(Duration::from_millis(400));
    let screen = term.screen();
    assert!(
        screen.contains("Repository Management") && !screen.contains("Clone a repository"),
        "{}",
        screen.text()
    );
}

/// On the Workspaces tab the badges are its two buttons (`1` New, `2` From
/// open tabs).
#[test]
fn ctrl_n_badges_press_the_workspaces_buttons() {
    let (mut term, _c, _fx) = setup();
    term.send_keys("M-j").unwrap();
    term.wait_for("the Workspaces page", STEP, |s| s.contains("Workspaces"))
        .unwrap_or_else(|e| panic!("{e}"));
    open_badges(&mut term);
    term.send_text("1").unwrap();
    term.wait_for("the new workspace form", STEP, |s| {
        // The form's placeholder; "New workspace" is also on the button.
        s.contains("what it groups")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}
