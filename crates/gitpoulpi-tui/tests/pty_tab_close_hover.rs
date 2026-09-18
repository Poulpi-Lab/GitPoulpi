//! The tab close cross is painted on the selected tab and the hovered one
//! only; the other tabs keep their width and do not jump on hover.

mod common;

use std::path::PathBuf;
use std::time::Duration;

use gitpoulpi_session::GlobalSession;
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const SESSION: &str = "pty-tab-close-hover";

fn repo(name: &str, subject: &str) -> (gitpoulpi_testkit::FixtureRepo, PathBuf) {
    let fx = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "x\n", subject)
        .unwrap()
        .build();
    let root = fx.root().parent().unwrap().join(name);
    std::fs::rename(fx.root(), &root).unwrap();
    (fx, root)
}

fn bar(term: &Terminal) -> String {
    term.text().lines().nth(1).unwrap_or("").to_string()
}

#[test]
fn the_close_cross_shows_on_the_active_and_the_hovered_tab_only() {
    let (a, orbit) = repo("orbit", "alpha-only-commit");
    let (b, moonbase) = repo("moonbase", "bravo-only-commit");
    let config = tempfile::tempdir().unwrap();
    let file = config
        .path()
        .join("sessions")
        .join(format!("{SESSION}.toml"));
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    let mut s = GlobalSession::default();
    s.open(&moonbase);
    s.open(&orbit);
    s.activate(1);
    s.save(&file).unwrap();
    let mut term = common::launch(&orbit)
        .session(SESSION)
        .config_dir(config.path())
        .size(200, 50)
        .no_exit_pause()
        .ready("alpha-only-commit")
        .mouse_capture()
        .spawn();
    term.wait_for("both tabs", STEP, |s| {
        let l = s.text().lines().nth(1).unwrap_or("").to_string();
        l.contains("orbit") && l.contains("moonbase")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Active tab: cross. Inactive, not hovered: none.
    let at_rest = bar(&term);
    assert!(at_rest.contains("orbit x"), "{at_rest}");
    assert!(!at_rest.contains("moonbase x"), "{at_rest}");
    let (mx, row) = term
        .screen()
        .find_all("moonbase")
        .into_iter()
        .find(|(_, r)| *r == 1)
        .unwrap();
    let col_of = |l: &str| l[..l.find("moonbase").unwrap()].chars().count();
    let before = col_of(&at_rest);
    term.move_to(mx + 2, row).unwrap();
    term.wait_for("the cross on the hovered tab", STEP, |s| {
        s.text().lines().nth(1).unwrap_or("").contains("moonbase x")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(col_of(&bar(&term)), before, "no jump");
    // Away: gone again.
    term.move_to(mx + 2, 20).unwrap();
    term.wait_for("the cross gone", STEP, |s| {
        !s.text().lines().nth(1).unwrap_or("").contains("moonbase x")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The hovered tab's cross closes it (a confirm modal asks first).
    term.move_to(mx + 2, row).unwrap();
    term.wait_for("the cross", STEP, |s| {
        s.text().lines().nth(1).unwrap_or("").contains("moonbase x")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let line = bar(&term);
    let at = line.find("moonbase x").unwrap();
    let cross = line[..at].chars().count() as u16 + "moonbase ".len() as u16;
    term.click(cross, row).unwrap();
    term.wait_for("the close prompt", STEP, |s| {
        s.contains("Close the tab for 'moonbase'")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
    drop((a, b));
}
