//! Home and the tab bar (lot A): Home has the repository tabs' status bar with
//! the Activity Log button at its bottom right (and no top-right "Global log"),
//! the Radar is a pinned tab right after Home, the `+` opens a small menu
//! (New tab, Radar, Workspaces) and the title bar carries a Search button.
//!
//! Real binary and daemon through a pty; temp config dirs and fixture
//! repositories only, no network.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

struct Setup {
    term: Terminal,
    name: String,
    _start: FixtureRepo,
    _config: tempfile::TempDir,
}

fn launch(session: &str) -> Setup {
    let start = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "tab-bar-start-commit")
        .unwrap()
        .build();
    let name = start
        .root()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let config = tempfile::tempdir().unwrap();
    let term = common::launch(start.root())
        .session(session)
        .config_dir(config.path())
        .size(180, 45)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    term.wait_for("the first window", STARTUP, |s| {
        s.contains("tab-bar-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    Setup {
        term,
        name,
        _start: start,
        _config: config,
    }
}

/// The tab bar's row (under the title border).
fn bar(screen: &Screen) -> String {
    screen.line(1)
}

/// The column of `needle` on `line`, counting cells (not bytes).
fn col_in(line: &str, needle: &str) -> u16 {
    let at = line
        .find(needle)
        .unwrap_or_else(|| panic!("no {needle:?} in {line:?}"));
    line[..at].chars().count() as u16
}

/// The status bar's row: the last row inside the window border.
fn status_row(screen: &Screen) -> u16 {
    screen.rows() - 2
}

fn on_home(s: &Screen) -> bool {
    s.contains("Repository Management") && s.contains("Search repositories")
}

fn global_tab_shows(s: &Screen) -> bool {
    s.contains("Scope:") && s.contains("All repositories")
}

fn radar_shows(s: &Screen) -> bool {
    s.contains("Scope") && s.contains("Open tabs") && s.contains("Refresh")
}

/// Point 1: Home gets the status bar of a repository tab, with the Activity
/// button at the same place and doing the same (here: the Global tab); the
/// old top-right "Global log" button is gone.
#[test]
fn home_has_the_status_bar_with_the_activity_button_at_its_bottom_right() {
    let mut s = launch("pty-tabbar-status");
    // Where a repository tab puts the button.
    // The status bar is drawn after the log rows (the first window only
    // proves the commit is there), so wait for the button itself.
    let screen = s
        .term
        .wait_for("the status bar's Activity button", STEP, |x| {
            x.line(status_row(x)).contains("Activity")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let repo_row = status_row(&screen);
    let repo_col = col_in(&screen.line(repo_row), "Activity");

    s.term.send_keys("C-o").unwrap();
    let screen = s
        .term
        .wait_for("Home with its status bar", STEP, |x| {
            on_home(x) && x.line(status_row(x)).contains("Activity")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(
        !screen.contains("Global log"),
        "the top-right entry is gone:\n{}",
        screen.text()
    );
    let row = status_row(&screen);
    assert_eq!(row, repo_row);
    assert_eq!(
        col_in(&screen.line(row), "Activity"),
        repo_col,
        "same place as on a repository tab:\n{}",
        screen.text()
    );

    // Click: the Global tab (Home has no repository of its own).
    s.term.click(repo_col + 1, row).unwrap();
    s.term
        .wait_for(
            "the global log by the status bar button",
            STEP,
            global_tab_shows,
        )
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Escape").unwrap();
    s.term
        .wait_for("closed", STEP, |x| !global_tab_shows(x))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    // The key a repository tab uses for its Activity Log (`Ctrl+N Shift+A`).
    s.term.send_keys("C-n").unwrap();
    s.term.send_keys("A").unwrap();
    s.term
        .wait_for("the Activity Log by its key", STEP, global_tab_shows)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Escape").unwrap();
    s.term.stop().unwrap();
}

/// Point 2: the Radar is a tab right after Home, reached by click; the
/// house takes it back to Home, and the tab has no close button.
#[test]
fn the_radar_is_a_pinned_tab_right_after_home() {
    let mut s = launch("pty-tabbar-radar-click");
    let screen = s.term.screen();
    let line = bar(&screen);
    let house = col_in(&line, "\u{2302}");
    let pad = col_in(&line, "Radar");
    let tab = col_in(&line, &s.name);
    assert!(
        house < pad && pad < tab,
        "Home, Radar, then the repository tab: {line}"
    );
    assert!(
        !line.contains("Radar x"),
        "no close cross while neither open nor hovered: {line}"
    );

    s.term.click(pad + 2, 1).unwrap();
    let screen = s
        .term
        .wait_for("the Radar tab", STEP, radar_shows)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(
        bar(&screen).contains("Radar x"),
        "the open tab has a close button: {}",
        bar(&screen)
    );
    assert!(
        bar(&screen).contains("Radar"),
        "the tab stays in the bar: {}",
        bar(&screen)
    );

    // The house leaves it for Home.
    s.term.click(house + 1, 1).unwrap();
    s.term
        .wait_for("Home", STEP, |x| on_home(x) && !radar_shows(x))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    // From Home the tab opens it in place.
    let pad = col_in(&bar(&s.term.screen()), "Radar");
    s.term.click(pad + 2, 1).unwrap();
    s.term
        .wait_for("the Radar again", STEP, radar_shows)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.stop().unwrap();
}

/// Point 2, keyboard and palette: `Alt+R` and `Radar: Open` reach the
/// same tab.
#[test]
fn the_radar_tab_is_reached_by_key_and_palette() {
    let mut s = launch("pty-tabbar-radar-keys");
    s.term.send_keys("M-r").unwrap();
    s.term
        .wait_for("the Radar by Alt+R", STEP, radar_shows)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(bar(&s.term.screen()).contains("Radar"));
    // Home, then the palette from there.
    let house = col_in(&bar(&s.term.screen()), "\u{2302}");
    s.term.click(house + 1, 1).unwrap();
    s.term
        .wait_for("Home", STEP, |x| on_home(x) && !radar_shows(x))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("C-p").unwrap();
    s.term.send_text("Radar: Open").unwrap();
    s.term
        .wait_for("the palette entry", STEP, |x| x.contains("Radar: Open"))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the Radar by the palette", STEP, radar_shows)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.stop().unwrap();
}

/// Point 3: the `+` opens a menu with New tab, Radar and Workspaces and no
/// "New workspace" row (creating one lives on the Workspaces tab, the palette
/// and its key); a click on the Workspaces row opens that tab.
#[test]
fn the_plus_opens_a_menu_with_workspaces_and_no_new_workspace_row() {
    let mut s = launch("pty-tabbar-plus");
    let screen = s.term.screen();
    let plus = col_in(&bar(&screen), " + ");

    // A click opens the menu and nothing else happens yet.
    s.term.click(plus + 1, 1).unwrap();
    let screen = s
        .term
        .wait_for("the + menu", STEP, |x| {
            x.find_all("Workspaces").iter().any(|(_, r)| *r > 1)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(
        !bar(&screen).contains("New tab"),
        "no New tab yet: {}",
        bar(&screen)
    );
    assert!(
        !screen.contains("New workspace"),
        "the + menu has no New workspace row:\n{}",
        screen.text()
    );
    let (col, row) = screen
        .find_all("Workspaces")
        .into_iter()
        .find(|(_, r)| *r > 1)
        .unwrap();
    s.term.click(col + 1, row).unwrap();
    s.term
        .wait_for("the Workspaces tab", STEP, |x| {
            x.contains("New workspace...") || x.contains("New workspace (n)")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.stop().unwrap();
}

/// Each `+` menu row is offered exactly when its own tab is not open: with
/// the Radar showing (and the Workspaces tab not open), Workspaces is still
/// offered.
#[test]
fn the_plus_menu_offers_workspaces_while_the_radar_shows() {
    let mut s = launch("pty-tabbar-plus-radar");
    s.term.send_keys("M-r").unwrap();
    s.term
        .wait_for("the Radar by Alt+R", STEP, radar_shows)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let plus = col_in(&bar(&s.term.screen()), " + ");
    s.term.click(plus + 1, 1).unwrap();
    s.term
        .wait_for("Workspaces in the + menu", STEP, |x| {
            x.find_all("Workspaces").iter().any(|(_, r)| *r > 1)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.stop().unwrap();
}

/// With the Radar tab closed, the `+` menu's Radar row shows its binding
/// (Alt+R) like the other rows do.
#[test]
fn the_plus_menu_radar_row_shows_its_key() {
    let mut s = launch("pty-tabbar-plus-radar-key");
    s.term.send_keys("C-p").unwrap();
    s.term.send_text("Radar: Close").unwrap();
    s.term
        .wait_for("the palette entry", STEP, |x| x.contains("Radar: Close"))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the Radar tab gone", STEP, |x| !bar(x).contains("Radar"))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    // Retried: a click aimed at a torn frame opens nothing.
    common::retry_until(
        &mut s.term,
        "the Radar row with its key",
        4,
        Duration::from_secs(10),
        |t| {
            let screen = t
                .wait_for_stable(Duration::from_millis(300), STEP)
                .unwrap_or_else(|e| panic!("{e}"));
            let plus = col_in(&bar(&screen), " + ");
            t.click(plus + 1, 1).unwrap();
        },
        |x| (2..8).any(|r| x.line(r).contains("Radar") && x.line(r).contains("Alt+R")),
    );
    s.term.stop().unwrap();
}

/// The Workspaces tab is open in the session but a narrow bar has no room to
/// draw it next to the Radar: the `+` menu offers Workspaces again, so the
/// tab is never unreachable (it used to vanish from both).
#[test]
fn the_plus_menu_offers_workspaces_when_the_narrow_bar_cannot_draw_its_tab() {
    let mut s = launch("pty-tabbar-plus-narrow");
    s.term.send_keys("M-j").unwrap();
    s.term
        .wait_for("the Workspaces tab", STEP, |x| {
            x.contains("New workspace...") || x.contains("New workspace (n)")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.resize(42, 45).unwrap();
    let screen = s
        .term
        .wait_for("the narrow bar", STEP, |x| {
            x.line(1).contains("Radar") && !x.line(1).contains("Workspaces")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    // Off the Workspaces page, so its title is not on screen by itself.
    let house = col_in(&bar(&screen), "\u{2302}");
    s.term.click(house + 1, 1).unwrap();
    let screen = s
        .term
        .wait_for("Home", STEP, |x| {
            on_home(x) && !x.contains("New workspace") && !x.contains("Workspaces")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let plus = col_in(&bar(&screen), " + ");
    s.term.click(plus + 1, 1).unwrap();
    s.term
        .wait_for("Workspaces in the + menu", STEP, |x| {
            x.find_all("Workspaces").iter().any(|(_, r)| *r > 1)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.stop().unwrap();
}

/// Point 3, mouse and palette: a click on the "New tab" row opens a New tab,
/// and `Tab: New...` opens the menu.
#[test]
fn the_plus_menu_rows_click_and_the_palette_twin() {
    let mut s = launch("pty-tabbar-plus-click");
    s.term.send_keys("C-p").unwrap();
    s.term.send_text("Tab: New...").unwrap();
    s.term
        .wait_for("the palette entry", STEP, |x| x.contains("Tab: New..."))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Enter").unwrap();
    let screen = s
        .term
        .wait_for("the + menu from the palette", STEP, |x| {
            x.find_all("Workspaces").iter().any(|(_, r)| *r > 1)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    // The row "New tab" of the menu is below the bar.
    let (col, row) = screen
        .find_all("New tab")
        .into_iter()
        .find(|(_, r)| *r > 1)
        .unwrap_or_else(|| panic!("no New tab row\n{}", screen.text()));
    s.term.click(col + 1, row).unwrap();
    s.term
        .wait_for("the New tab", STEP, |x| {
            x.contains("OPEN REPOSITORIES") && bar(x).contains("New tab")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.stop().unwrap();
}

/// The global Search button (magnifier and label) is in the title bar, flush
/// left of the gear: it
/// opens the global search from a repository tab and from Home, and the tab
/// row no longer carries it.
#[test]
fn the_title_bar_has_a_search_button_that_opens_the_global_search() {
    let mut s = launch("pty-tabbar-search");
    let screen = s.term.screen();
    let (mag, row) = screen
        .find("\u{2315}")
        .filter(|(_, r)| *r == 0)
        .unwrap_or_else(|| panic!("no magnifier on the title row\n{}", screen.text()));
    let (gear, gear_row) = screen.find("\u{2699}").expect("the gear");
    assert_eq!((row, gear_row), (0, 0));
    assert_eq!(
        gear,
        mag + 4 + "Global search".len() as u16,
        "the magnifier and its label are flush left of the gear"
    );
    assert!(
        screen.line(0).contains("\u{2315} Global search "),
        "{}",
        screen.text()
    );
    assert!(
        !bar(&screen).contains('\u{2315}') && !bar(&screen).contains("Global search"),
        "the tab row has no search: {}",
        bar(&screen)
    );
    s.term.click(mag, 0).unwrap();
    s.term
        .wait_for("the search popup", STEP, |x| {
            x.contains("Search open repositories")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Escape").unwrap();
    s.term
        .wait_for("closed", STEP, |x| !x.contains("Search open repositories"))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    // The same button on Home.
    s.term.send_keys("C-o").unwrap();
    s.term
        .wait_for("Home", STEP, on_home)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let (mag, _) = s
        .term
        .screen()
        .find("\u{2315}")
        .filter(|(_, r)| *r == 0)
        .expect("the magnifier on Home");
    s.term.click(mag, 0).unwrap();
    s.term
        .wait_for("the search popup on Home", STEP, |x| {
            x.contains("Search open repositories")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.stop().unwrap();
}
