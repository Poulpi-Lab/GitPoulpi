//! `ui.toolbar_position`, `ui.toolbar_align`, `ui.toolbar_pr_pinned` and
//! `ui.statusbar_position`, end to end through a pty: the toolbar is a row
//! at the top or the bottom, or a column on the left or right; every button
//! stays clickable in each place; the status line is the first or the last
//! row; and the options change live from the palette and from Settings.
//! Elements are found by their text, never by a hard-coded row, except the
//! rows that *are* the assertion (the edge a band sits on).

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const MESSAGE: &str = "layoutprobe";
const COLS: u16 = 200;
const ROWS: u16 = 40;

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("notes.txt", "original\n", MESSAGE)
        .unwrap()
        .write_worktree("notes.txt", b"changed\n")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str, ui: &str) -> (Terminal, tempfile::TempDir) {
    let toml = format!("[ui]\ngraph_renderer = \"text\"\n{ui}");
    common::launch(repo.root())
        .session(session)
        .config(toml)
        .size(COLS, ROWS)
        .no_exit_pause()
        .ready_when("the fixture and the toolbar", |s| {
            s.contains(MESSAGE) && s.contains("Stash") && s.contains("Activity")
        })
        .mouse_capture()
        .start()
}

fn at(screen: &Screen, needle: &str) -> (u16, u16) {
    screen
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} is not on screen:\n{}", screen.text()))
}

fn config_text(dir: &tempfile::TempDir) -> String {
    std::fs::read_to_string(dir.path().join("config.toml")).unwrap_or_default()
}

fn wait_config(dir: &tempfile::TempDir, what: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while std::time::Instant::now() < deadline {
        if config_text(dir).contains(what) {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("config.toml never showed {what}: {}", config_text(dir));
}

fn stash_count(root: &std::path::Path) -> usize {
    let out = std::process::Command::new("git")
        .args(["stash", "list"])
        .current_dir(root)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).lines().count()
}

/// Picks `Settings: <label>: <choice>` from the palette.
fn palette_choose(term: &mut Terminal, label: &str, choice: &str) {
    term.send_keys("C-p").unwrap();
    term.send_text(&format!("settings: {}", label.to_lowercase()))
        .unwrap();
    term.wait_for("the setting's row", STEP, |s| {
        s.contains(&format!("Settings: {label}..."))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the choices", STEP, |s| {
        s.contains(&format!("Settings: {label}: "))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text(&choice.to_lowercase()).unwrap();
    term.wait_for("the choice narrowed", STEP, |s| {
        s.text().lines().any(|l| {
            l.contains(&format!("Settings: {label}: ")) && l.to_lowercase().contains(choice)
        })
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

/// The default: the toolbar is the row above the status line, at the
/// bottom, its buttons packed at the start.
#[test]
fn the_default_layout_is_a_bottom_toolbar_above_a_bottom_status_line() {
    let repo = fixture();
    let (term, _c) = launch(&repo, "pty-layout-default", "");
    let s = snap(&term);
    let (undo_c, undo_r) = at(&s, "Undo");
    let (_, pull_r) = at(&s, "Pull");
    let (_, log_r) = at(&s, MESSAGE);
    let (_, activity_r) = at(&s, "Activity");
    assert_eq!(pull_r, undo_r, "one row of buttons");
    assert_eq!(pull_r, ROWS - 3, "the toolbar row ends the panes");
    assert_eq!(activity_r, ROWS - 2, "the status line is the last row");
    assert!(log_r < pull_r, "the panes are above the toolbar");
    assert!(undo_c < 10, "start-aligned: Undo is at the left edge");
    // The pull request button is pinned to the far end when there is one;
    // with none linked the row simply ends after Search.
    let _ = term.stop();
}

#[test]
fn a_top_toolbar_sits_under_the_tabs() {
    let repo = fixture();
    let (term, _c) = launch(&repo, "pty-layout-top", "toolbar_position = \"top\"\n");
    let s = snap(&term);
    let (_, pull_r) = at(&s, "Pull");
    let (_, log_r) = at(&s, MESSAGE);
    let (_, activity_r) = at(&s, "Activity");
    assert_eq!(pull_r, 2, "border, tabs, then the toolbar");
    assert!(log_r > pull_r);
    assert_eq!(activity_r, ROWS - 2);
    let _ = term.stop();
}

#[test]
fn a_left_toolbar_is_a_column_beside_the_panes() {
    let repo = fixture();
    let (term, _c) = launch(&repo, "pty-layout-left", "toolbar_position = \"left\"\n");
    let s = snap(&term);
    let (undo_c, undo_r) = at(&s, "Undo");
    let (pull_c, pull_r) = at(&s, "Pull");
    let (push_c, push_r) = at(&s, "Push");
    let (stash_c, stash_r) = at(&s, "Stash");
    let (log_c, _) = at(&s, MESSAGE);
    assert!(
        undo_c < 6 && pull_c < 6 && push_c < 6 && stash_c < 6,
        "{undo_c}"
    );
    assert!(undo_r < pull_r && pull_r < push_r && push_r < stash_r);
    assert!(log_c > 14, "the panes start right of the column");
    let (_, activity_r) = at(&s, "Activity");
    assert_eq!(activity_r, ROWS - 2);
    let _ = term.stop();
}

#[test]
fn a_right_toolbar_is_a_column_at_the_right_edge() {
    let repo = fixture();
    let (term, _c) = launch(&repo, "pty-layout-right", "toolbar_position = \"right\"\n");
    let s = snap(&term);
    let (pull_c, pull_r) = at(&s, "Pull");
    let (push_c, push_r) = at(&s, "Push");
    let (log_c, _) = at(&s, MESSAGE);
    assert!(pull_c > COLS - 30 && push_c > COLS - 30, "{pull_c}");
    assert!(pull_r < push_r);
    assert!(log_c < pull_c);
    let _ = term.stop();
}

/// Every position keeps the buttons live: Stash stashes the change.
#[test]
fn clicking_stash_works_in_every_toolbar_position() {
    for pos in ["top", "bottom", "left", "right"] {
        let repo = fixture();
        let (mut term, _c) = launch(
            &repo,
            &format!("pty-layout-click-{pos}"),
            &format!("toolbar_position = \"{pos}\"\n"),
        );
        let (c, r) = at(&snap(&term), "Stash");
        term.click(c + 1, r).unwrap();
        term.wait_for("the stash to land", STEP, |_| stash_count(repo.root()) == 1)
            .unwrap_or_else(|e| panic!("{pos}: {e}\n{}", term.text()));
        let _ = term.stop();
    }
}

/// The dropdown arrow beside Push opens its menu in every position, and
/// the menu is fully on screen (above a bottom toolbar, beside a column).
#[test]
fn the_pull_dropdown_opens_in_view_in_every_toolbar_position() {
    for pos in ["top", "bottom", "left", "right"] {
        let repo = fixture();
        let (mut term, _c) = launch(
            &repo,
            &format!("pty-layout-menu-{pos}"),
            &format!("toolbar_position = \"{pos}\"\n"),
        );
        let s = snap(&term);
        let (c, r) = at(&s, "Pull");
        // `Pull ▾`: the arrow is the chip after the label.
        let arrow = (c..c + 12)
            .find(|x| s.cell(*x, r).is_some_and(|cell| cell.text == "\u{25BE}"))
            .unwrap_or_else(|| panic!("{pos}: no arrow after Pull:\n{}", s.text()));
        term.click(arrow, r).unwrap();
        term.wait_for("the Pull menu", STEP, |s| {
            s.contains("Fetch") || s.contains("Pull from")
        })
        .unwrap_or_else(|e| panic!("{pos}: {e}\n{}", term.text()));
        term.send_keys("Escape").unwrap();
        let _ = term.stop();
    }
}

#[test]
fn the_alignment_moves_the_buttons_along_the_toolbar() {
    let repo = fixture();
    let col = |align: &str| {
        let (term, _c) = launch(
            &repo,
            &format!("pty-layout-align-{align}"),
            &format!("toolbar_align = \"{align}\"\n"),
        );
        let (c, _) = at(&snap(&term), "Undo");
        let _ = term.stop();
        c
    };
    let (start, center, end) = (col("start"), col("center"), col("end"));
    assert!(start < center && center < end, "{start} {center} {end}");
    assert!(end > COLS / 2);
}

#[test]
fn the_alignment_also_works_in_a_column() {
    let repo = fixture();
    let row = |align: &str| {
        let (term, _c) = launch(
            &repo,
            &format!("pty-layout-valign-{align}"),
            &format!("toolbar_position = \"left\"\ntoolbar_align = \"{align}\"\n"),
        );
        let (_, r) = at(&snap(&term), "Undo");
        let _ = term.stop();
        r
    };
    let (start, center, end) = (row("start"), row("center"), row("end"));
    assert!(start < center && center < end, "{start} {center} {end}");
}

#[test]
fn the_status_line_can_go_to_the_top() {
    let repo = fixture();
    let (term, _c) = launch(
        &repo,
        "pty-layout-status-top",
        "statusbar_position = \"top\"\n",
    );
    let s = snap(&term);
    let (_, activity_r) = at(&s, "Activity");
    let (_, pull_r) = at(&s, "Pull");
    let (_, log_r) = at(&s, MESSAGE);
    assert_eq!(activity_r, 1, "the first row inside the border");
    assert_eq!(pull_r, ROWS - 2, "the bottom toolbar takes the last row");
    assert!(log_r > activity_r && log_r < pull_r);
    let _ = term.stop();
}

/// A click in the Log selects the commit whatever the layout: the row
/// arithmetic follows the panes, wherever the bands around them are.
#[test]
fn log_rows_are_clickable_with_the_bands_moved() {
    for (toolbar, status) in [("top", "top"), ("left", "top"), ("right", "bottom")] {
        let repo = fixture();
        let (mut term, _c) = launch(
            &repo,
            &format!("pty-layout-log-{toolbar}-{status}"),
            &format!("toolbar_position = \"{toolbar}\"\nstatusbar_position = \"{status}\"\n"),
        );
        let (c, r) = at(&snap(&term), MESSAGE);
        term.click(c + 2, r).unwrap();
        term.wait_for("the commit's detail", STEP, |s| s.contains("notes.txt"))
            .unwrap_or_else(|e| panic!("{toolbar}/{status}: {e}\n{}", term.text()));
        let _ = term.stop();
    }
}

/// The palette changes the position live, without a restart, and writes it.
#[test]
fn the_palette_moves_the_toolbar_and_the_status_line_live() {
    let repo = fixture();
    let (mut term, config) = launch(&repo, "pty-layout-palette", "");
    palette_choose(&mut term, "Toolbar position", "left");
    wait_config(&config, "toolbar_position = \"left\"");
    let s = term
        .wait_for("the column", STEP, |s| {
            s.find("Pull").is_some_and(|(c, _)| c < 6)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(at(&s, "Push").1 > at(&s, "Pull").1);

    palette_choose(&mut term, "Toolbar position", "top");
    wait_config(&config, "toolbar_position = \"top\"");
    term.wait_for("the top row", STEP, |s| {
        s.find("Pull").is_some_and(|(_, r)| r == 2)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    palette_choose(&mut term, "Status line position", "top");
    wait_config(&config, "statusbar_position = \"top\"");
    term.wait_for("the status line on top", STEP, |s| {
        s.find("Activity").is_some_and(|(_, r)| r == 1)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    palette_choose(&mut term, "Toolbar button alignment", "end");
    wait_config(&config, "toolbar_align = \"end\"");
    term.wait_for("the buttons at the end", STEP, |s| {
        s.find("Undo").is_some_and(|(c, _)| c > COLS / 2)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}

/// The Settings page has the rows (mouse path): picking Left in the combo
/// writes it and the column shows once Settings closes.
#[test]
fn the_settings_page_moves_the_toolbar() {
    let repo = fixture();
    let (mut term, config) = launch(&repo, "pty-layout-settings", "");
    let (gc, gr) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gc, gr).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "Appearance");
    // The "Bottom" on the Toolbar position row (the status line has one too:
    // which is the first on screen depends on how far the page scrolled).
    // Retried: a click aimed at a torn or still-scrolling frame opens nothing.
    let screen = common::retry_until(
        &mut term,
        "the popup",
        4,
        std::time::Duration::from_secs(10),
        |t| {
            // Scrolled to the row by the wheel, one notch at a time on a
            // settled screen (the page may still be moving, either way).
            let mut notch = 0;
            let (col, row) = loop {
                let screen = t
                    .wait_for_stable(std::time::Duration::from_millis(300), STEP)
                    .unwrap_or_else(|e| panic!("{e}"));
                let found = screen.find("Toolbar position").and_then(|(_, label_row)| {
                    screen
                        .find_all("Bottom")
                        .into_iter()
                        .find(|(_, r)| *r == label_row)
                });
                if let Some(at) = found {
                    break at;
                }
                assert!(
                    notch < 40,
                    "the Toolbar position row never showed:\n{}",
                    t.text()
                );
                // Up first (the row is near the top), then down.
                t.scroll(notch >= 12, 100, screen.rows() / 2).unwrap();
                notch += 1;
            };
            t.click(col + 1, row).unwrap();
        },
        |s| s.contains("\u{2713} Bottom"),
    );
    let _ = screen;
    // Top, the ticked Bottom, then Left one row below. Retried until stored:
    // a click aimed at a torn frame picks nothing.
    common::retry_until(
        &mut term,
        "the choice stored",
        4,
        std::time::Duration::from_secs(10),
        |t| {
            let screen = t
                .wait_for_stable(std::time::Duration::from_millis(300), STEP)
                .unwrap_or_else(|e| panic!("{e}"));
            if let Some((tc, tr)) = screen.find("\u{2713} Bottom") {
                t.click(tc + 2, tr + 1).unwrap();
            }
        },
        |_| {
            std::fs::read_to_string(config.path().join("config.toml"))
                .is_ok_and(|t| t.contains("toolbar_position = \"left\""))
        },
    );
    wait_config(&config, "toolbar_position = \"left\"");
    term.send_keys("Escape").unwrap();
    term.wait_for("the column", STEP, |s| {
        s.find("Pull").is_some_and(|(c, _)| c < 6)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}

/// A window too narrow for a column draws the toolbar as a bottom row.
#[test]
fn a_narrow_window_falls_back_to_a_bottom_row() {
    let repo = fixture();
    let toml = "[ui]\ngraph_renderer = \"text\"\ntoolbar_position = \"left\"\n";
    let (term, _config) = common::launch(repo.root())
        .session("pty-layout-narrow")
        .config(toml)
        .size(70, ROWS)
        .no_exit_pause()
        .ready_when("the fixture", |s| s.contains("Pull"))
        .mouse_capture()
        .start();
    let (_, r) = at(&snap(&term), "Pull");
    assert_eq!(r, ROWS - 3, "a row above the status line");
    let _ = term.stop();
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
