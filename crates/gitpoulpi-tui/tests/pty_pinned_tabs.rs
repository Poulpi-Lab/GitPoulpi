//! The Radar and the Workspaces are tabs of their own, in all three tab-bar
//! positions: opened from the `+` menu (and the palette, and a global key),
//! closed with their `x`, remembered across restarts. Home is one filterable
//! list with no page switcher, Open first.
//!
//! Real binary and daemon through a pty; temp config dirs and fixture
//! repositories only, no network.

mod common;

use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

struct Setup {
    term: Terminal,
    config: tempfile::TempDir,
    _start: FixtureRepo,
}

fn config_for(position: &str) -> String {
    format!("[ui]\ngraph_renderer = \"text\"\ntabs_position = \"{position}\"\n")
}

fn launch_in(config: tempfile::TempDir, start: FixtureRepo, session: &str) -> Setup {
    let term = spawn(start.root(), config.path(), session);
    Setup {
        term,
        config,
        _start: start,
    }
}

fn spawn(start: &Path, config: &Path, session: &str) -> Terminal {
    let term = common::launch(start)
        .session(session)
        .config_dir(config)
        .size(180, 45)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    term.wait_for("the first window", STARTUP, |s| {
        s.contains("pinned-tabs-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term
}

fn launch(position: &str, session: &str) -> Setup {
    let start = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "pinned-tabs-start-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(config.path().join("config.toml"), config_for(position)).unwrap();
    launch_in(config, start, session)
}

fn on_home(s: &Screen) -> bool {
    s.contains("Repository Management") && s.contains("Search repositories")
}

fn radar_shows(s: &Screen) -> bool {
    s.contains("Scope") && s.contains("Open tabs") && s.contains("Refresh")
}

fn workspaces_shows(s: &Screen) -> bool {
    s.contains("New workspace") && s.contains("From open tabs")
}

/// The cell of the `x` that closes the tab labelled `label`: on the row
/// where only blanks sit between the label and an `x` (a horizontal bar
/// draws `Radar x`, the left column pads to its right edge).
fn close_cell(screen: &Screen, label: &str) -> (u16, u16) {
    for (col, row) in screen.find_all(label) {
        let tail = col + label.chars().count() as u16;
        let next = (tail..screen.cols()).find(|c| {
            screen
                .cell(*c, row)
                .is_some_and(|cell| !cell.text.trim().is_empty())
        });
        if let Some(c) = next.filter(|c| screen.cell(*c, row).is_some_and(|x| x.text == "x")) {
            return (c, row);
        }
    }
    panic!("no {label:?} tab with a close cross\n{}", screen.text());
}

/// Whether `needle` is on a row of the list (the tab bar's own rows hold
/// the open repository's name, which is not the list).
fn list_has(screen: &Screen, needle: &str) -> bool {
    (4..screen.rows().saturating_sub(2)).any(|r| screen.line(r).contains(needle))
}

/// Opens the `+` menu through the palette's twin (it hangs under the `+`
/// wherever the bar is).
fn open_plus_menu(term: &mut Terminal, row_label: &str) -> Screen {
    term.send_keys("C-p").unwrap();
    term.send_text("Tab: New...").unwrap();
    term.wait_for("the palette entry", STEP, |x| x.contains("Tab: New..."))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the + menu", STEP, |x| {
        x.contains("New tab") && x.contains(row_label)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

/// Clicks the first occurrence of `text` on a row other than the tab row,
/// the menu row (the tab itself, when present, is not what is wanted).
fn click_menu_row(term: &mut Terminal, screen: &Screen, text: &str) {
    let (col, row) = screen
        .find_all(text)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no {text:?} row\n{}", screen.text()));
    term.click(col + 1, row).unwrap();
}

fn radar_round_trip(position: &str) {
    let mut s = launch(position, &format!("pty-pinned-radar-{position}"));
    // Open it (its tab is there by default), then close it with its `x`.
    s.term.send_keys("M-r").unwrap();
    let screen = s
        .term
        .wait_for("the Radar", STEP, radar_shows)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let (x, y) = close_cell(&screen, "Radar");
    s.term.click(x, y).unwrap();
    let screen = s
        .term
        .wait_for("the Radar tab closed", STEP, |x| {
            on_home(x) && !x.contains("Radar")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(!radar_shows(&screen), "{}", screen.text());

    // Reopen it from the `+` menu.
    let menu = open_plus_menu(&mut s.term, "Radar");
    click_menu_row(&mut s.term, &menu, "Radar");
    let screen = s
        .term
        .wait_for("the Radar reopened by the + menu", STEP, radar_shows)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(screen.contains("Radar"), "its tab is back");
    s.term.stop().unwrap();
}

#[test]
fn the_radar_tab_closes_with_its_x_and_reopens_from_the_plus_menu_with_tabs_on_top() {
    radar_round_trip("top");
}

#[test]
fn the_radar_tab_closes_with_its_x_and_reopens_from_the_plus_menu_with_tabs_at_the_bottom() {
    radar_round_trip("bottom");
}

#[test]
fn the_radar_tab_closes_with_its_x_and_reopens_from_the_plus_menu_with_tabs_on_the_left() {
    radar_round_trip("left");
}

fn workspaces_round_trip(position: &str) {
    let session = format!("pty-pinned-ws-{position}");
    let mut s = launch(position, &session);
    assert!(
        !s.term.screen().contains("Workspaces"),
        "no Workspaces tab until it is opened:\n{}",
        s.term.text()
    );

    // Open it from the `+` menu.
    let menu = open_plus_menu(&mut s.term, "Workspaces");
    click_menu_row(&mut s.term, &menu, "Workspaces");
    let screen = s
        .term
        .wait_for("the Workspaces tab", STEP, workspaces_shows)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let (x, y) = close_cell(&screen, "Workspaces");
    assert!(
        !screen.contains("1 Repositories") && !screen.contains("2 Workspaces"),
        "no page switcher:\n{}",
        screen.text()
    );

    // Close it with its `x`: back on Home, the tab gone.
    s.term.click(x, y).unwrap();
    s.term
        .wait_for("the tab closed", STEP, |x| {
            on_home(x) && !x.contains("Workspaces")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    // Reopen it with the global key, close it with Ctrl+W, reopen it with
    // the palette command and leave it open for the restart.
    s.term.send_keys("M-j").unwrap();
    s.term
        .wait_for("the tab by Alt+J", STEP, workspaces_shows)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("C-w").unwrap();
    s.term
        .wait_for("the tab closed by Ctrl+W", STEP, |x| {
            on_home(x) && !x.contains("Workspaces")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("C-p").unwrap();
    s.term.send_text("Workspace: Manage").unwrap();
    s.term
        .wait_for("the palette entry", STEP, |x| {
            x.contains("Workspace: Manage...")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the tab by the palette", STEP, workspaces_shows)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.stop().unwrap();

    // Remembered: a new run of the same session has the tab (not selected).
    let session_file = s
        .config
        .path()
        .join("sessions")
        .join(format!("{session}.toml"));
    let saved = std::fs::read_to_string(&session_file).unwrap_or_default();
    assert!(saved.contains("workspaces_open = true"), "{saved}");
    let term = spawn(s._start.root(), s.config.path(), &session);
    let screen = term.screen();
    assert!(
        screen.contains("Workspaces"),
        "the tab is back after a restart:\n{}",
        screen.text()
    );
    assert!(!workspaces_shows(&screen), "but it is not the shown view");
    term.stop().unwrap();
}

#[test]
fn the_workspaces_tab_opens_from_the_plus_menu_closes_and_is_remembered_with_tabs_on_top() {
    workspaces_round_trip("top");
}

#[test]
fn the_workspaces_tab_opens_from_the_plus_menu_closes_and_is_remembered_with_tabs_at_the_bottom() {
    workspaces_round_trip("bottom");
}

#[test]
fn the_workspaces_tab_opens_from_the_plus_menu_closes_and_is_remembered_with_tabs_on_the_left() {
    workspaces_round_trip("left");
}

#[test]
fn home_has_no_page_switcher_and_no_workspaces_toolbar_button() {
    let mut s = launch("top", "pty-pinned-no-switcher");
    s.term.send_keys("C-o").unwrap();
    let screen = s
        .term
        .wait_for("Home", STEP, on_home)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let top = screen.line(2);
    assert!(
        !screen.contains("1 Repositories")
            && !screen.contains("2 Workspaces")
            && !screen.contains("Workspaces"),
        "Home is one list; Workspaces is a tab:\n{}",
        screen.text()
    );
    assert!(top.contains("Repository Management"), "{top:?}");
    // The old page key does nothing on Home any more.
    s.term.send_text("2").unwrap();
    std::thread::sleep(Duration::from_millis(400));
    let screen = s.term.screen();
    assert!(
        on_home(&screen) && !workspaces_shows(&screen),
        "{}",
        screen.text()
    );
    s.term.stop().unwrap();
}

/// Open first, then Favorites / Recent / All, and ONE filter (name, branch,
/// path) narrowing all of them.
#[test]
fn one_filter_narrows_favorites_and_every_section_and_open_comes_first() {
    fn named(name: &str, branch: Option<&str>) -> (FixtureRepo, PathBuf) {
        let fx = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", "x\n", "pinned-tabs-start-commit")
            .unwrap()
            .build();
        let root = fx.root().parent().unwrap().join(name);
        std::fs::rename(fx.root(), &root).unwrap();
        if let Some(branch) = branch {
            common::git(&root, &["checkout", "-q", "-b", branch]);
        }
        (fx, root)
    }
    let (_a, open) = named("openrepo", None);
    let (_b, fav) = named("favrepo", None);
    let (_c, recent) = named("recentrepo", Some("quokka-branch"));
    let config = tempfile::tempdir().unwrap();
    let toml = |path: &Path, name: &str| {
        format!(
            "[[repos]]\npath = '{}'\nname = '{name}'\nlast_opened = 1\n\n",
            path.display()
        )
    };
    std::fs::write(
        config.path().join("recent.toml"),
        format!("{}{}", toml(&fav, "favrepo"), toml(&recent, "recentrepo")),
    )
    .unwrap();
    std::fs::write(
        config.path().join("favorites.toml"),
        format!(
            "[[repos]]\npath = '{}'\nname = 'favrepo'\n\n",
            fav.display()
        ),
    )
    .unwrap();
    let mut term = common::launch(&open)
        .session("pty-pinned-filter")
        .config_dir(config.path())
        .size(180, 45)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    term.wait_for("the first window", STARTUP, |s| {
        s.contains("pinned-tabs-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-o").unwrap();
    let screen = term
        .wait_for("every section filled", STEP, |s| {
            on_home(s) && s.contains("FAVORITES") && s.contains("recentrepo")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let row = |s: &Screen, needle: &str| s.find(needle).map(|(_, r)| r);
    assert!(
        row(&screen, "OPEN REPOSITORIES") < row(&screen, "FAVORITES"),
        "Open comes first:\n{}",
        screen.text()
    );
    assert!(
        row(&screen, "FAVORITES") < row(&screen, "RECENT REPOSITORIES"),
        "{}",
        screen.text()
    );
    assert!(
        row(&screen, "RECENT REPOSITORIES") < row(&screen, "ALL REPOSITORIES"),
        "{}",
        screen.text()
    );

    // The filter narrows every section together: the
    // favorite stays in Favorites, Recent and All, and the others go.
    term.send_keys("C-f").unwrap();
    term.send_text("favrepo").unwrap();
    let screen = term
        .wait_for("the list narrowed", STEP, |s| {
            !list_has(s, "openrepo") && !list_has(s, "recentrepo")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let shown = (4..screen.rows())
        .filter(|r| screen.line(*r).contains("favrepo"))
        .count();
    assert!(
        shown >= 3,
        "in Favorites, Recent and All ({shown} rows):\n{}",
        screen.text()
    );

    // A branch matches too (the probe fills it in; the recent entry's own
    // branch is the hint until then).
    for _ in 0..7 {
        term.send_keys("BSpace").unwrap();
    }
    term.send_text("quokka").unwrap();
    term.wait_for("only the repository on that branch", STEP, |s| {
        list_has(s, "recentrepo") && !list_has(s, "favrepo") && !list_has(s, "openrepo")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The Settings gear keeps its icon, and still opens Settings, in every tab
/// bar position and toolbar position, on a repository tab, on Home and on
/// the Radar and Workspaces tabs (it lives on the tab bar's row, so no
/// view and no layout may take the glyph away).
#[test]
fn the_settings_gear_shows_its_icon_in_every_layout_and_on_every_tab() {
    let gear = "\u{2699}";
    for tabs in ["top", "bottom", "left"] {
        for toolbar in ["top", "bottom", "left", "right"] {
            let start = FixtureRepoBuilder::new()
                .unwrap()
                .commit_file("a.txt", "a\n", "pinned-tabs-start-commit")
                .unwrap()
                .build();
            let config = tempfile::tempdir().unwrap();
            std::fs::write(
                config.path().join("config.toml"),
                format!(
                    "[ui]\ngraph_renderer = \"text\"\ntabs_position = \"{tabs}\"\ntoolbar_position = \"{toolbar}\"\n"
                ),
            )
            .unwrap();
            let mut term = spawn(start.root(), config.path(), "pty-pinned-gear");
            type View<'a> = (&'a str, Option<&'a str>, fn(&Screen) -> bool);
            let views: [View; 4] = [
                ("a repository tab", None, |s| {
                    s.contains("pinned-tabs-start-commit")
                }),
                ("Home", Some("C-o"), on_home),
                ("the Radar tab", Some("M-r"), radar_shows),
                ("the Workspaces tab", Some("M-j"), workspaces_shows),
            ];
            for (name, key, shown) in views {
                if let Some(key) = key {
                    term.send_keys(key).unwrap();
                }
                let screen = term
                    .wait_for(name, STEP, shown)
                    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
                assert!(
                    screen.contains(gear),
                    "no gear on {name} (tabs {tabs}, toolbar {toolbar}):\n{}",
                    screen.text()
                );
            }
            // And it is a working button: a click opens Settings.
            let (col, row) = term.screen().find(gear).unwrap();
            term.click(col, row).unwrap();
            term.wait_for("Settings", STEP, |s| s.contains("Repo-specific settings"))
                .unwrap_or_else(|e| {
                    panic!(
                        "gear click (tabs {tabs}, toolbar {toolbar}): {e}\n{}",
                        term.text()
                    )
                });
            term.stop().unwrap();
        }
    }
}
