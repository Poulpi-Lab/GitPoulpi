//! The help popup's tabs: the key bindings table, then one concise guide per
//! area of the user docs. Tabs switch by key (`Tab`, `Shift+Tab`, `Left`,
//! `Right`, `Alt+1`..) and by click; the header under the pointer lights up,
//! the filter narrows the current tab and `Ctrl+D` opens that tab's docs page.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn launch(session: &str, log: Option<&std::path::Path>) -> (Terminal, tempfile::TempDir) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("help tabs probe")
        .unwrap()
        .build();
    let mut builder = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready("help tabs probe")
        .mouse_capture();
    if let Some(log) = log {
        builder = builder.env("GITPOULPI_OPEN_LOG", log);
    }
    // The fixture repo has to outlive the app: leak it with the process.
    std::mem::forget(repo);
    builder.start()
}

/// Each tab's own marker (an entry title only that tab draws).
const MARKERS: [&str; 8] = [
    "Keybindings",
    "Command palette",
    "Reading the graph",
    "Working changes",
    "Branches and",
    "Link a repository",
    "Themes",
    "AI (optional)",
];

fn on_tab(s: &Screen, tab: usize) -> bool {
    MARKERS[tab] == "Keybindings" && s.contains("Keybindings - ")
        || MARKERS[tab] != "Keybindings" && s.contains(MARKERS[tab])
}

fn wait_tab(term: &mut Terminal, tab: usize) {
    term.wait_for(MARKERS[tab], STEP, |s| on_tab(s, tab))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Drawn whole: later reads of the screen are not torn frames.
    term.wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The popup's tab row (the row of the "Hosting" header, which only it draws
/// next to "AI & CLI").
fn tab_row(s: &Screen) -> u16 {
    s.find_all("AI & CLI").first().expect("the tab row").1
}

fn header(s: &Screen, label: &str) -> (u16, u16) {
    let row = tab_row(s);
    let col = s
        .find_all(label)
        .into_iter()
        .find(|(_, r)| *r == row)
        .unwrap_or_else(|| panic!("{label:?} on the tab row\n{}", s.text()))
        .0;
    (col, row)
}

#[test]
fn tabs_switch_by_key() {
    let (mut term, _c) = launch("pty-help-tabs-keys", None);
    term.send_text("?").unwrap();
    wait_tab(&mut term, 0);
    term.send_keys("Tab").unwrap();
    wait_tab(&mut term, 1);
    term.send_keys("Right").unwrap();
    wait_tab(&mut term, 2);
    term.send_keys("Left").unwrap();
    wait_tab(&mut term, 1);
    term.send_keys("BTab").unwrap();
    wait_tab(&mut term, 0);
    // Wraps backwards to the last tab.
    term.send_keys("BTab").unwrap();
    wait_tab(&mut term, 7);
    term.send_keys("M-6").unwrap();
    wait_tab(&mut term, 5);
    term.send_keys("M-1").unwrap();
    wait_tab(&mut term, 0);
    term.stop().unwrap();
}

#[test]
fn tabs_switch_by_click_and_the_header_under_the_pointer_lights_up() {
    let (mut term, _c) = launch("pty-help-tabs-click", None);
    term.send_text("?").unwrap();
    wait_tab(&mut term, 0);
    let s = term
        .wait_for("the whole tab row", STEP, |s| {
            s.find_all("AI & CLI").first().is_some_and(|(_, row)| {
                ["Setup", "Hosting"]
                    .iter()
                    .all(|l| s.find_all(l).iter().any(|(_, r)| r == row))
            })
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = header(&s, "Setup");
    let rest = header(&s, "Hosting");
    let before = s.cell(c + 1, r).map(|x| format!("{:?}", x.bg));
    term.move_to(c + 1, r).unwrap();
    term.wait_for("the hovered header lights up", STEP, |s| {
        s.cell(c + 1, r).map(|x| format!("{:?}", x.bg)) != before
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Only that one: its neighbor keeps the plain look.
    assert_eq!(
        snap(&term)
            .cell(rest.0 + 1, r)
            .map(|x| format!("{:?}", x.bg)),
        s.cell(rest.0 + 1, r).map(|x| format!("{:?}", x.bg))
    );
    term.click(c + 1, r).unwrap();
    wait_tab(&mut term, 6);
    let (c, r) = header(&snap(&term), "Changes");
    term.click(c + 1, r).unwrap();
    wait_tab(&mut term, 3);
    // A click on a header does not close the popup.
    assert!(snap(&term).contains("Working changes"));
    term.stop().unwrap();
}

#[test]
fn the_filter_narrows_the_current_tab_and_ctrl_d_opens_its_page() {
    let log = tempfile::NamedTempFile::new().unwrap();
    let (mut term, _c) = launch("pty-help-tabs-filter", Some(log.path()));
    term.send_text("?").unwrap();
    wait_tab(&mut term, 0);
    term.send_keys("M-2").unwrap();
    wait_tab(&mut term, 1);
    term.send_text("palette").unwrap();
    term.wait_for("the filter narrowed the tab", STEP, |s| {
        s.contains("filter \"palette\"") && !s.contains("Shortcut prefixes")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(snap(&term).contains("Command palette"));
    term.send_keys("C-d").unwrap();
    let url = "https://github.com/Poulpi-Lab/GitPoulpi/blob/main/docs/user/interface-tour.md";
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let logged = std::fs::read_to_string(log.path()).unwrap_or_default();
        if logged.lines().any(|l| l == url) {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "{logged:?}");
        std::thread::sleep(Duration::from_millis(200));
    }
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
