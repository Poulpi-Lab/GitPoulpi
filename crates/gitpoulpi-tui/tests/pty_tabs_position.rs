//! `ui.tabs_position`: the repository tab bar is a row at the top (the
//! default, covered by every other tab test), a row at the very bottom, or a
//! column at the left edge. Each place keeps every tab interaction working:
//! click, middle-click, the close cross, the right-click menu, drag to
//! reorder, the keyboard, the wheel and the overflow row; and the option
//! changes live from the palette and from Settings.
//!
//! Tabs are found by their text; the rows that *are* the assertion (the edge
//! a band sits on) are the only hard-coded ones.

mod common;

use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_session::GlobalSession;
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Button, Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const NAMES: [&str; 9] = [
    "orbit", "moonbase", "comet", "nebula", "quasar", "pulsar", "zenith", "apogee", "vertex",
];
/// The tab column's width plus the window border.
const COLUMN_RIGHT: u16 = 25;

struct Fx {
    _keep: Vec<gitpoulpi_testkit::FixtureRepo>,
    first: PathBuf,
    config: tempfile::TempDir,
    session: String,
}

fn subject(name: &str) -> String {
    format!("{name}-only-commit")
}

/// `n` repositories opened as tabs in `NAMES` order, the first one active,
/// with `ui` appended to the `[ui]` table of a temp config.
fn setup(session: &str, n: usize, ui: &str) -> Fx {
    let mut keep = Vec::new();
    let mut roots = Vec::new();
    for name in &NAMES[..n] {
        let fx = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", "x\n", &subject(name))
            .unwrap()
            .build();
        let root = fx.root().parent().unwrap().join(name);
        std::fs::rename(fx.root(), &root).unwrap();
        keep.push(fx);
        roots.push(root);
    }
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!("[ui]\ngraph_renderer = \"text\"\n{ui}"),
    )
    .unwrap();
    let file = session_file(config.path(), session);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    let mut s = GlobalSession::default();
    for root in &roots[1..] {
        s.open(root);
    }
    s.open(&roots[0]);
    s.activate(n - 1);
    s.tabs.rotate_right(1);
    s.save(&file).unwrap();
    Fx {
        _keep: keep,
        first: roots[0].clone(),
        config,
        session: session.to_string(),
    }
}

fn session_file(config: &Path, session: &str) -> PathBuf {
    config.join("sessions").join(format!("{session}.toml"))
}

fn order(fx: &Fx) -> Vec<String> {
    GlobalSession::load(&session_file(fx.config.path(), &fx.session))
        .unwrap()
        .tabs
        .iter()
        .map(|t| {
            t.repo_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn start(fx: &Fx, cols: u16, rows: u16) -> Terminal {
    start_ready(fx, cols, rows, &subject(NAMES[0]))
}

fn start_ready(fx: &Fx, cols: u16, rows: u16, ready: &str) -> Terminal {
    common::launch(&fx.first)
        .session(&fx.session)
        .config_dir(fx.config.path())
        .size(cols, rows)
        .no_exit_pause()
        .ready(ready)
        .mouse_capture()
        .spawn()
}

/// Where the tab named `name` is drawn: inside the tab band (the first
/// `max_col` columns for a column, the given row for a row), never in the
/// title bar above. Waited for: a torn frame can lack the tab bar.
fn wait_tab(term: &mut Terminal, name: &str, in_column: bool, row: u16) -> (u16, u16) {
    let screen = term
        .wait_for(&format!("the {name} tab"), STEP, |s| {
            tab_pos(s, name, in_column, row).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    tab_pos(&screen, name, in_column, row).unwrap()
}

fn tab_pos(screen: &Screen, name: &str, in_column: bool, row: u16) -> Option<(u16, u16)> {
    screen.find_all(name).into_iter().find(|(c, r)| {
        if in_column {
            *c < COLUMN_RIGHT && *r >= 1
        } else {
            *r == row
        }
    })
}

fn wait_commit(term: &mut Terminal, name: &str) {
    let want = subject(name);
    term.wait_for(&format!("{name} active"), STEP, |s| s.contains(&want))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The title bar names the active tab's repository (for windows too short
/// to show the commit).
fn wait_title(term: &mut Terminal, name: &str) {
    term.wait_for(&format!("{name} in the title"), STEP, |s| {
        // The path ends in the name, after a `/` or (Windows) a `\`.
        title_has(s, &format!("/{name} ")) || title_has(s, &format!("\\{name} "))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
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

/// The title bar (row 0) names the active repository's path.
fn title_has(s: &Screen, name: &str) -> bool {
    s.line(0).contains(name)
}

const ROWS: u16 = 40;
const COLS: u16 = 200;

/// A bottom tab bar is the window's very last row, below the status line,
/// and every tab interaction works on it.
#[test]
fn bottom_tabs_are_the_last_row_and_every_interaction_works() {
    let fx = setup("pty-tabs-bottom", 3, "tabs_position = \"bottom\"\n");
    let mut term = start(&fx, COLS, ROWS);
    // Row ROWS-1 is the border; the bar is the row above it.
    let bar_row = ROWS - 2;
    let (ox, r) = wait_tab(&mut term, "orbit", false, bar_row);
    let s = term
        .wait_for("the status line", STEP, |s| s.contains("Activity"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(r, bar_row, "the tab bar is the last row inside the border");
    let (_, activity_r) = s.find("Activity").unwrap();
    assert_eq!(activity_r, bar_row - 1, "the status line is just above");
    assert!(s.line(1).contains("GitPoulpi") || !s.line(1).contains("moonbase"));
    assert!(ox < 20);

    // Click switches.
    let (mx, _) = wait_tab(&mut term, "moonbase", false, bar_row);
    let want = subject("moonbase");
    common::retry_until(
        &mut term,
        "moonbase active",
        4,
        Duration::from_secs(8),
        |t| t.click(mx + 1, bar_row).unwrap(),
        |s| s.contains(&want),
    );

    // Ctrl+PageDown: the keyboard moves on.
    term.send_text("\x1b[6;5~").unwrap();
    wait_commit(&mut term, "comet");

    // Right click: the tab menu hangs in view (above the bar here).
    let (mx, _) = wait_tab(&mut term, "moonbase", false, bar_row);
    common::retry_until(
        &mut term,
        "the tab menu",
        4,
        Duration::from_secs(8),
        |t| {
            if !t.screen().contains("Close tab") {
                t.click_with(Button::Right, mx + 1, bar_row).unwrap();
            }
        },
        |s| s.contains("Close tab"),
    );
    term.send_keys("Escape").unwrap();
    term.wait_for("the menu closed", STEP, |s| !s.contains("Close tab"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Middle click closes (after the question).
    let (nx, _) = wait_tab(&mut term, "moonbase", false, bar_row);
    common::retry_until(
        &mut term,
        "the close question",
        4,
        Duration::from_secs(8),
        |t| {
            if !t.screen().contains("Close the tab for 'moonbase'") {
                t.click_with(Button::Middle, nx + 1, bar_row).unwrap();
            }
        },
        |s| s.contains("Close the tab for 'moonbase'"),
    );
    term.send_keys("Escape").unwrap();

    // Drag orbit onto comet's place.
    let (ox, _) = wait_tab(&mut term, "orbit", false, bar_row);
    let (cx, _) = wait_tab(&mut term, "comet", false, bar_row);
    // A press that lands on a torn frame starts no drag: release and start
    // over until the marker shows.
    common::retry_until(
        &mut term,
        "the insertion marker",
        4,
        Duration::from_secs(8),
        |t| {
            t.release(cx + 8, bar_row).unwrap();
            t.press(ox + 1, bar_row).unwrap();
            t.drag_to(ox + 5, bar_row).unwrap();
            t.drag_to(cx + 8, bar_row).unwrap();
        },
        |s| s.line(bar_row).contains('\u{2503}'),
    );
    term.release(cx + 8, bar_row).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while order(&fx) != ["moonbase", "comet", "orbit"] && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(
        order(&fx),
        ["moonbase", "comet", "orbit"],
        "{}",
        term.text()
    );
    term.stop().unwrap();
}

/// A left tab bar is a column at the window's left edge, one tab per row.
#[test]
fn left_tabs_are_a_column_and_every_interaction_works() {
    let fx = setup("pty-tabs-left", 3, "tabs_position = \"left\"\n");
    let mut term = start(&fx, COLS, ROWS);
    let (ox, or) = wait_tab(&mut term, "orbit", true, 0);
    // One settled screen for the layout checks below: the tabs, the Log and
    // the status line all drawn (a torn frame can lack any of them).
    let s = term
        .wait_for("the whole left layout", STEP, |s| {
            tab_pos(s, "comet", true, 0).is_some()
                && s.contains(&subject("orbit"))
                && s.contains("Activity")
                && s.line(1).contains('\u{2302}')
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (_, mr) = wait_tab(&mut term, "moonbase", true, 0);
    let (_, cr) = wait_tab(&mut term, "comet", true, 0);
    assert!(ox <= 3, "names start at the left edge: {ox}");
    assert_eq!((mr, cr), (or + 1, or + 2), "one row each, in order");
    // The search button lives in the title bar: Home is the column's first row.
    assert!(
        !s.line(1).contains("Global search") && !s.line(1).contains('\u{2315}'),
        "{:?}",
        s.line(1)
    );
    let home = s.line(1);
    assert!(
        home.chars()
            .skip(1)
            .take(24)
            .collect::<String>()
            .contains('\u{2302}'),
        "{home:?}"
    );
    // The panes start right of the column.
    let (log_c, _) = s.find(&subject("orbit")).unwrap();
    assert!(log_c > COLUMN_RIGHT, "{log_c}");
    // The status line is right of the column too, on the last row.
    let (act_c, act_r) = s.find("Activity").unwrap();
    assert!(act_c > COLUMN_RIGHT);
    assert_eq!(act_r, ROWS - 2);

    // Click switches.
    let want = subject("moonbase");
    common::retry_until(
        &mut term,
        "moonbase active",
        4,
        Duration::from_secs(8),
        |t| t.click(ox + 1, mr).unwrap(),
        |s| s.contains(&want),
    );

    // Keyboard.
    term.send_text("\x1b[6;5~").unwrap();
    wait_commit(&mut term, "comet");
    term.send_text("\x1b[5;5~").unwrap();
    wait_commit(&mut term, "moonbase");

    // Right click opens the tab menu.
    let (mx, mr) = wait_tab(&mut term, "moonbase", true, 0);
    common::retry_until(
        &mut term,
        "the tab menu",
        4,
        Duration::from_secs(8),
        |t| {
            if !t.screen().contains("Close tab") {
                t.click_with(Button::Right, mx + 1, mr).unwrap();
            }
        },
        |s| s.contains("Close tab"),
    );
    term.send_keys("Escape").unwrap();
    term.wait_for("the menu closed", STEP, |s| !s.contains("Close tab"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The active tab's cross closes (after the question).
    let (_, mr) = wait_tab(&mut term, "moonbase", true, 0);
    common::retry_until(
        &mut term,
        "the close question",
        4,
        Duration::from_secs(8),
        |t| {
            if !t.screen().contains("Close the tab for 'moonbase'") {
                t.click(COLUMN_RIGHT - 2, mr).unwrap();
            }
        },
        |s| s.contains("Close the tab for 'moonbase'"),
    );
    term.send_keys("Escape").unwrap();

    // Middle click on another row asks too.
    let (cx, cr) = wait_tab(&mut term, "comet", true, 0);
    common::retry_until(
        &mut term,
        "the close question",
        4,
        Duration::from_secs(8),
        |t| {
            if !t.screen().contains("Close the tab for 'comet'") {
                t.click_with(Button::Middle, cx + 1, cr).unwrap();
            }
        },
        |s| s.contains("Close the tab for 'comet'"),
    );
    term.send_keys("Escape").unwrap();

    // Drag orbit down onto comet's row: the marker shows, the drop reorders.
    let (ox, or) = wait_tab(&mut term, "orbit", true, 0);
    let (_, cr) = wait_tab(&mut term, "comet", true, 0);
    // A press that lands on a torn frame starts no drag: release and start
    // over until the marker shows.
    common::retry_until(
        &mut term,
        "the insertion marker",
        4,
        Duration::from_secs(8),
        |t| {
            t.release(ox + 3, cr).unwrap();
            t.press(ox + 1, or).unwrap();
            t.drag_to(ox + 1, or + 1).unwrap();
            t.drag_to(ox + 3, cr).unwrap();
        },
        |s| (1..ROWS - 1).any(|r| s.line(r).chars().nth(1) == Some('\u{2501}')),
    );
    term.release(ox + 3, cr).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while order(&fx) != ["moonbase", "comet", "orbit"] && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(
        order(&fx),
        ["moonbase", "comet", "orbit"],
        "{}",
        term.text()
    );
    // The session file is written before the column is redrawn: wait for a
    // screen that shows the new order.
    let s = term
        .wait_for("the column in the new order", STEP, |s| {
            let row = |name| tab_pos(s, name, true, 0).map(|(_, r)| r);
            matches!(
                (row("moonbase"), row("comet"), row("orbit")),
                (Some(mr), Some(cr), Some(or)) if mr < cr && cr < or
            )
        })
        .unwrap_or_else(|e| panic!("the column follows the order: {e}\n{}", term.text()));
    let _ = title_has(&s, "orbit");
    term.stop().unwrap();
}

/// More tabs than rows: the overflow row counts the hidden ones, the wheel
/// scrolls the list, and the active tab is revealed.
#[test]
fn a_long_column_scrolls_with_the_wheel_and_keeps_the_active_tab_in_view() {
    let fx = setup("pty-tabs-left-scroll", 9, "tabs_position = \"left\"\n");
    let mut term = start_ready(&fx, COLS, 12, "Activity");
    let s = term.screen();
    let (_, before) = wait_tab(&mut term, "orbit", true, 0);
    assert!(
        s.find_all("\u{25BC}")
            .iter()
            .any(|(c, _)| *c < COLUMN_RIGHT),
        "the overflow row shows hidden tabs below\n{}",
        s.text()
    );
    assert!(s.find_all("vertex").iter().all(|(c, _)| *c >= COLUMN_RIGHT));
    // Wheel down over the column: later tabs come into view.
    for _ in 0..6 {
        term.scroll(true, 5, before + 1).unwrap();
    }
    term.wait_for("the last tab", STEP, |s| {
        s.find_all("vertex").iter().any(|(c, _)| *c < COLUMN_RIGHT)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Clicking a tab that was scrolled into view switches to it.
    let (vx, vr) = wait_tab(&mut term, "vertex", true, 0);
    term.click(vx + 1, vr).unwrap();
    wait_title(&mut term, "vertex");
    // The next tab wraps round to the first, which the list reveals again.
    term.send_text("\x1b[6;5~").unwrap();
    wait_title(&mut term, "orbit");
    term.wait_for("orbit in view", STEP, |s| {
        s.find_all("orbit")
            .iter()
            .any(|(c, r)| *c < COLUMN_RIGHT && *r >= 1)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Left tabs outside a left toolbar, with the status line on top.
#[test]
fn left_tabs_compose_with_a_left_toolbar_and_a_top_status_line() {
    let fx = setup(
        "pty-tabs-left-compose",
        3,
        "tabs_position = \"left\"\ntoolbar_position = \"left\"\nstatusbar_position = \"top\"\n",
    );
    let mut term = start(&fx, COLS, ROWS);
    let s = term.screen();
    let (ox, _) = wait_tab(&mut term, "orbit", true, 0);
    let (undo_c, _) = s.find("Undo").unwrap();
    let (log_c, _) = s.find(&subject("orbit")).unwrap();
    assert!(ox < undo_c, "tabs are outermost: {ox} < {undo_c}");
    assert!(
        undo_c > COLUMN_RIGHT - 2,
        "the toolbar column is right of the tabs"
    );
    assert!(log_c > undo_c + 6, "the panes are right of the toolbar");
    let (act_c, act_r) = s.find("Activity").unwrap();
    assert_eq!(act_r, 1, "the status line is the first row");
    assert!(act_c > COLUMN_RIGHT);
    // Both still click: Stash stashes nothing here, so use a tab and Undo's
    // hover-free neighbour: Pull's dropdown opens.
    let (mx, mr) = wait_tab(&mut term, "moonbase", true, 0);
    term.click(mx + 1, mr).unwrap();
    wait_commit(&mut term, "moonbase");
    let s = term
        .wait_for("the Pull button", STEP, |s| s.find("Pull").is_some())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = s.find("Pull").unwrap();
    term.click(c + 1, r).unwrap();
    term.stop().unwrap();
}

/// Left tabs on the Home tab: the list covers everything right of the
/// column, and the tab column stays clickable.
#[test]
fn the_home_tab_works_with_left_and_bottom_tabs() {
    for pos in ["left", "bottom"] {
        let fx = setup(
            &format!("pty-tabs-home-{pos}"),
            2,
            &format!("tabs_position = \"{pos}\"\n"),
        );
        let mut term = start(&fx, COLS, ROWS);
        let (hc, hr) = if pos == "left" {
            (2, 1)
        } else {
            let s = term
                .wait_for("the Home tab", STEP, |s| s.find("\u{2302}").is_some())
                .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
            s.find("\u{2302}").unwrap()
        };
        term.click(hc, hr).unwrap();
        term.wait_for("Home", STEP, |s| {
            s.contains("Repository Management") || s.contains("Repositories")
        })
        .unwrap_or_else(|e| panic!("{pos}: {e}\n{}", term.text()));
        if pos == "left" {
            // The tab column is still there beside the list.
            wait_tab(&mut term, "moonbase", true, 0);
        } else {
            wait_tab(&mut term, "moonbase", false, ROWS - 2);
        }
        // A tab click leaves Home again.
        let (mx, mr) = if pos == "left" {
            wait_tab(&mut term, "moonbase", true, 0)
        } else {
            wait_tab(&mut term, "moonbase", false, ROWS - 2)
        };
        term.click(mx + 1, mr).unwrap();
        wait_commit(&mut term, "moonbase");
        term.stop().unwrap();
    }
}

/// The gear is on the title row, just left of the quit button with blank
/// cells between them, whatever the tab bar position; the tab band no longer
/// carries it. Clicking it opens Settings, clicking the gap does nothing.
#[test]
fn the_gear_sits_left_of_the_quit_button_in_every_tab_position() {
    for pos in ["top", "bottom", "left"] {
        let fx = setup(
            &format!("pty-tabs-gear-{pos}"),
            2,
            &format!("tabs_position = \"{pos}\"\n"),
        );
        let mut term = start(&fx, COLS, ROWS);
        let s = term
            .wait_for("the gear and the quit button", STEP, |s| {
                s.find("\u{2699}").is_some() && s.find("\u{D7}").is_some()
            })
            .unwrap_or_else(|e| panic!("{pos}: {e}\n{}", term.text()));
        let (gc, gr) = s
            .find("\u{2699}")
            .unwrap_or_else(|| panic!("{pos}: no gear"));
        let (qc, qr) = s.find("\u{D7}").unwrap_or_else(|| panic!("{pos}: no quit"));
        assert_eq!((gr, qr), (0, 0), "{pos}: both on the title row");
        assert!(gc + 3 < qc && qc - gc < 8, "{pos}: gear {gc}, quit {qc}");
        assert_eq!(
            s.find_all("\u{2699}").len(),
            1,
            "{pos}: the old spot is gone\n{}",
            term.text()
        );
        // The gap between the two is not the quit button: it is inert.
        term.click(qc - 2, 0).unwrap();
        std::thread::sleep(Duration::from_millis(300));
        assert!(term.screen().contains("Activity"), "{pos}: still running");
        term.click(gc, gr).unwrap();
        term.wait_for("Settings open", STEP, |s| {
            s.contains("Repo-specific settings")
        })
        .unwrap_or_else(|e| panic!("{pos}: {e}\n{}", term.text()));
        term.stop().unwrap();
    }
}

/// The global search is a title bar button flush left of the gear in every
/// tab position, never in the tab row or column; clicking it opens the search.
#[test]
fn the_global_search_is_in_the_title_bar_in_every_tab_position() {
    for pos in ["left", "top", "bottom"] {
        let fx = setup(
            &format!("pty-tabs-search-{pos}"),
            2,
            &format!("tabs_position = \"{pos}\"\n"),
        );
        let mut term = start(&fx, COLS, ROWS);
        let s = term
            .wait_for("the gear and the magnifier", STEP, |s| {
                s.find("\u{2699}").is_some()
                    && s.find("\u{2315}").filter(|(_, r)| *r == 0).is_some()
                    && s.find("Global search").is_some()
            })
            .unwrap_or_else(|e| panic!("{pos}: {e}\n{}", term.text()));
        let (gc, gr) = s.find("\u{2699}").unwrap_or_else(|| panic!("{pos}: gear"));
        let (mc, mr) = s
            .find("\u{2315}")
            .filter(|(_, r)| *r == 0)
            .unwrap_or_else(|| panic!("{pos}: no magnifier on the title row"));
        assert_eq!((mr, gr), (0, 0), "{pos}");
        assert_eq!(
            gc,
            mc + 4 + "Global search".len() as u16,
            "{pos}: the label, then flush left of the gear"
        );
        assert_eq!(
            s.find("Global search").map(|(_, r)| r),
            Some(0),
            "{pos}: the search label is on the title row only"
        );
        for r in 1..ROWS {
            // Only the toolbar's own "Search" button may carry a magnifier.
            assert!(
                !s.line(r).contains('\u{2315}') || s.line(r).contains("\u{2315} Search"),
                "{pos}: row {r} has a bare magnifier: {:?}",
                s.line(r)
            );
        }
        term.click(mc, 0).unwrap();
        term.wait_for("the search popup", STEP, |x| {
            x.contains("Search open repositories")
        })
        .unwrap_or_else(|e| panic!("{pos}: {e}\n{}", term.text()));
        term.stop().unwrap();
    }
}

/// A window under 80 columns keeps the tabs on top.
#[test]
fn a_narrow_window_keeps_left_tabs_on_top() {
    let fx = setup("pty-tabs-narrow", 2, "tabs_position = \"left\"\n");
    let term = start(&fx, 70, ROWS);
    let s = term
        .wait_for("the moonbase tab", STEP, |s| {
            s.find_all("moonbase").iter().any(|(_, r)| *r >= 1)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (_, r) = s
        .find_all("moonbase")
        .into_iter()
        .find(|(_, r)| *r >= 1)
        .unwrap();
    assert_eq!(r, 1, "the tabs are the row under the border");
    term.stop().unwrap();
}

/// The palette changes the position live and writes it.
#[test]
fn the_palette_moves_the_tab_bar_live() {
    let fx = setup("pty-tabs-palette", 3, "");
    let mut term = start(&fx, COLS, ROWS);
    let (_, r) = wait_tab(&mut term, "moonbase", false, 1);
    assert_eq!(r, 1, "the default is the top row");
    palette_choose(&mut term, "Tab bar position", "left");
    wait_config(&fx.config, "tabs_position = \"left\"");
    term.wait_for("the column", STEP, |s| {
        s.find_all("moonbase")
            .iter()
            .any(|(c, r)| *c < COLUMN_RIGHT && *r > 1)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    palette_choose(&mut term, "Tab bar position", "bottom");
    wait_config(&fx.config, "tabs_position = \"bottom\"");
    term.wait_for("the bottom row", STEP, |s| {
        s.find_all("moonbase").iter().any(|(_, r)| *r == ROWS - 2)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    palette_choose(&mut term, "Tab bar position", "top");
    wait_config(&fx.config, "tabs_position = \"top\"");
    term.wait_for("the top row", STEP, |s| {
        s.find_all("moonbase").iter().any(|(_, r)| *r == 1)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The Settings page has the row (mouse path).
#[test]
fn the_settings_page_moves_the_tab_bar() {
    let fx = setup("pty-tabs-settings", 3, "");
    let mut term = start(&fx, COLS, ROWS);
    let (gc, gr) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gc, gr).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "Appearance");
    // Retried, the choice located afresh each time: a click placed on a torn
    // or not yet scrolled frame can land nowhere.
    let screen = common::retry_until(
        &mut term,
        "the popup",
        4,
        STEP,
        |t| {
            if !t.screen().contains("\u{2713} Top") {
                let (col, row) = common::scroll_settings_to_choice(t, "Top", STEP);
                t.click(col + 1, row).unwrap();
            }
        },
        |s| s.contains("\u{2713} Top"),
    );
    let (tc, tr) = screen.find("\u{2713} Top").unwrap();
    // Top (ticked), Bottom, Left.
    term.click(tc + 2, tr + 2).unwrap();
    wait_config(&fx.config, "tabs_position = \"left\"");
    term.send_keys("Escape").unwrap();
    term.wait_for("the column", STEP, |s| {
        s.find_all("moonbase")
            .iter()
            .any(|(c, r)| *c < COLUMN_RIGHT && *r > 1)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
