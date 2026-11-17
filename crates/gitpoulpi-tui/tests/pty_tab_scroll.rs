//! Tabs that do not fit the tab bar scroll: the active tab is brought into
//! view whenever it changes (keyboard, picker, resize), the arrow buttons at
//! the bar's ends move it by one tab, the wheel over the bar scrolls it, and
//! a tab dragged onto an arrow scrolls the bar while it rests there.

mod common;

use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_session::GlobalSession;
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const SESSION: &str = "pty-tab-scroll";
const NAMES: [&str; 8] = [
    "orbit", "moonbase", "comet", "nebula", "quasar", "pulsar", "meteor", "galaxy",
];
const LEFT: &str = "\u{2039}";
const RIGHT: &str = "\u{203A}";

struct Tabs {
    _keep: Vec<gitpoulpi_testkit::FixtureRepo>,
    roots: Vec<PathBuf>,
    config: tempfile::TempDir,
}

fn session_file(config: &Path) -> PathBuf {
    config.join("sessions").join(format!("{SESSION}.toml"))
}

/// Eight repositories open as tabs, `active` the selected one.
fn tabs(active: usize) -> Tabs {
    let mut keep = Vec::new();
    let mut roots = Vec::new();
    for name in NAMES {
        let fx = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("f.txt", "x\n", &format!("{name}-only-commit"))
            .unwrap()
            .build();
        let root = fx.root().parent().unwrap().join(name);
        std::fs::rename(fx.root(), &root).unwrap();
        keep.push(fx);
        roots.push(root);
    }
    let config = tempfile::tempdir().unwrap();
    let file = session_file(config.path());
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    let mut s = GlobalSession::default();
    for r in &roots {
        s.open(r);
    }
    s.activate(active);
    s.save(&file).unwrap();
    Tabs {
        _keep: keep,
        roots,
        config,
    }
}

fn start(t: &Tabs, active: usize, width: u16) -> Terminal {
    common::launch(&t.roots[active])
        .session(SESSION)
        .config_dir(t.config.path())
        .size(width, 30)
        .no_exit_pause()
        .ready(&format!("{}-only-commit", NAMES[active]))
        .mouse_capture()
        .spawn()
}

/// The tab bar (row 1; the title bar above names the repository too).
fn bar(s: &Screen) -> String {
    s.text().lines().nth(1).unwrap_or("").to_string()
}

/// The tab names on the bar, left to right.
fn shown(s: &Screen) -> Vec<&'static str> {
    let line = bar(s);
    let mut found: Vec<(usize, &'static str)> = NAMES
        .into_iter()
        .filter_map(|n| line.find(&format!(" {n} ")).map(|i| (i, n)))
        .collect();
    found.sort();
    found.into_iter().map(|(_, n)| n).collect()
}

fn glyph_on_bar(term: &Terminal, glyph: &str) -> (u16, u16) {
    term.screen()
        .find_all(glyph)
        .into_iter()
        .find(|(_, r)| *r == 1)
        .unwrap_or_else(|| panic!("no {glyph} on the tab bar\n{}", term.text()))
}

fn order(config: &Path) -> Vec<String> {
    GlobalSession::load(&session_file(config))
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

/// Switching to the last tab from the keyboard scrolls it into view; then
/// each click on the left arrow shows exactly one more tab on the left,
/// until the first tab is back.
#[test]
fn the_active_tab_scrolls_into_view_and_the_left_arrow_brings_the_first_back() {
    let t = tabs(0);
    let mut term = start(&t, 0, 72);
    term.wait_for("the first tabs, the last one hidden", STEP, |s| {
        let names = shown(s);
        names.first() == Some(&"orbit") && !names.contains(&"galaxy") && bar(s).contains(RIGHT)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The keyboard: the tab picker (`Ctrl+N Tab`), filtered to the last tab.
    term.send_keys("C-n").unwrap();
    term.send_keys("Tab").unwrap();
    term.wait_for("the tab picker", STEP, |s| s.contains("Select Tab"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("galaxy").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the last tab shown whole on the bar", STEP, |s| {
        s.contains("galaxy-only-commit")
            && bar(s).contains(" galaxy x")
            && !shown(s).contains(&"orbit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // One tab per click on the left arrow.
    let mut first = shown(&term.screen())[0];
    while first != "orbit" {
        let at = NAMES.iter().position(|n| *n == first).unwrap();
        let want = NAMES[at - 1];
        let (x, row) = glyph_on_bar(&term, LEFT);
        term.click(x, row).unwrap();
        term.wait_for("one more tab on the left", STEP, |s| {
            shown(s).first() == Some(&want)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        first = want;
    }
    term.stop().unwrap();
}

/// The wheel over the bar scrolls it, and the right arrow does too.
#[test]
fn the_wheel_and_the_right_arrow_scroll_the_bar() {
    let t = tabs(0);
    let mut term = start(&t, 0, 72);
    term.wait_for("the first tab at the start", STEP, |s| {
        shown(s).first() == Some(&"orbit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (x, row) = glyph_on_bar(&term, RIGHT);
    term.scroll(true, x.saturating_sub(10), row).unwrap();
    term.wait_for("the bar scrolled by the wheel", STEP, |s| {
        shown(s).first() == Some(&"moonbase")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(x, row).unwrap();
    term.wait_for("the bar scrolled by the arrow", STEP, |s| {
        shown(s).first() == Some(&"comet")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The view itself did not change: the bar only scrolled.
    assert!(term.text().contains("orbit-only-commit"));
    term.stop().unwrap();
}

/// Holding the right arrow down keeps scrolling, to the last tab, with no
/// other input (the scrollbar arrows' auto-repeat).
#[test]
fn holding_the_right_arrow_keeps_scrolling() {
    let t = tabs(0);
    let mut term = start(&t, 0, 72);
    term.wait_for("the first tab at the start", STEP, |s| {
        shown(s).first() == Some(&"orbit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (x, row) = glyph_on_bar(&term, RIGHT);
    term.press(x, row).unwrap();
    term.wait_for("the bar scrolled to its end while held", STEP, |s| {
        shown(s).last() == Some(&"galaxy")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(x, row).unwrap();
    term.stop().unwrap();
}

/// A wide terminal shows every tab; narrowed, it keeps the active (last)
/// tab in view.
#[test]
fn a_resize_keeps_the_active_tab_in_view() {
    let t = tabs(7);
    let mut term = start(&t, 7, 200);
    term.wait_for("every tab, no arrows", STEP, |s| {
        shown(s).len() == NAMES.len() && !bar(s).contains(RIGHT)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.resize(72, 30).unwrap();
    term.wait_for("the active tab still on the bar", STEP, |s| {
        let names = shown(s);
        bar(s).contains(LEFT) && names.contains(&"galaxy") && !names.contains(&"orbit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A tab carried onto the right arrow scrolls the bar while it rests there;
/// dropped on the arrow, it lands after the last tab shown.
#[test]
fn dragging_a_tab_onto_the_right_arrow_scrolls_the_bar() {
    let t = tabs(0);
    let mut term = start(&t, 0, 72);
    term.wait_for("the first tab at the start", STEP, |s| {
        shown(s).first() == Some(&"orbit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (ox, row) = term
        .screen()
        .find_all("orbit")
        .into_iter()
        .find(|(_, r)| *r == 1)
        .unwrap();
    let (ax, _) = glyph_on_bar(&term, RIGHT);
    term.press(ox + 1, row).unwrap();
    term.drag_to(ox + 5, row).unwrap();
    term.drag_to(ax, row).unwrap();
    term.wait_for("the bar scrolled to its end", STEP, |s| {
        shown(s).last() == Some(&"galaxy")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(ax, row).unwrap();
    term.wait_for("orbit dropped last", STEP, |_| {
        order(t.config.path()).last().map(String::as_str) == Some("orbit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{:?}", order(t.config.path())));
    term.stop().unwrap();
}

/// `Ctrl+N` badges the live arrow like any other control, and its badge
/// scrolls the bar as a click does; a dimmed arrow wears no badge.
#[test]
fn the_right_arrow_wears_a_ctrl_n_badge_that_scrolls() {
    let t = tabs(0);
    let mut term = start(&t, 0, 72);
    term.wait_for("the first tab at the start", STEP, |s| {
        shown(s).first() == Some(&"orbit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (rx, row) = glyph_on_bar(&term, RIGHT);
    let (lx, _) = glyph_on_bar(&term, LEFT);
    term.send_keys("C-n").unwrap();
    let screen = term
        .wait_for("a badge over the right arrow", STEP, |s| {
            s.cell(rx, row)
                .is_some_and(|c| c.text != RIGHT && c.text.trim().len() == 1)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        screen.cell(lx, row).map(|c| c.text.as_str()),
        Some(LEFT),
        "the dimmed left arrow has no badge"
    );
    let badge = screen.cell(rx, row).unwrap().text.trim().to_string();
    term.send_text(&badge).unwrap();
    term.wait_for("the bar scrolled by the badge", STEP, |s| {
        shown(s).first() == Some(&"moonbase")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
