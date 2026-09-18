//! The tab picker opens with the active tab highlighted, so Up/Down move from
//! where the owner is instead of from the first row.

mod common;

use std::time::Duration;

use gitpoulpi_session::GlobalSession;
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const SESSION: &str = "pty-tab-picker-focus";

#[test]
fn the_picker_opens_on_the_active_tab_and_down_enter_lands_on_the_next() {
    let names = ["orbit", "moonbase", "comet", "nebula"];
    let subjects = [
        "sub-one-only",
        "sub-two-only",
        "sub-three-only",
        "sub-four-only",
    ];
    let mut keep = Vec::new();
    let mut roots = Vec::new();
    for (name, subject) in names.iter().zip(subjects) {
        let fx = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("f.txt", "x\n", subject)
            .unwrap()
            .build();
        let root = fx.root().parent().unwrap().join(name);
        std::fs::rename(fx.root(), &root).unwrap();
        keep.push(fx);
        roots.push(root);
    }
    let config = tempfile::tempdir().unwrap();
    let file = config
        .path()
        .join("sessions")
        .join(format!("{SESSION}.toml"));
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    let mut s = GlobalSession::default();
    for r in &roots {
        s.open(r);
    }
    s.activate(2);
    s.save(&file).unwrap();

    let mut term: Terminal = common::launch(&roots[2])
        .session(SESSION)
        .config_dir(config.path())
        .size(200, 50)
        .no_exit_pause()
        .ready("sub-three-only")
        .mouse_capture()
        .spawn();
    term.send_keys("C-n").unwrap();
    term.send_text("t").unwrap();
    term.wait_for("the tab picker", STEP, |s| {
        s.contains("Alt+Up/Down to reorder")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Down from the highlighted third tab is the fourth; from row 0 it
    // would be the second.
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the fourth tab shown", STEP, |s| {
        s.contains("sub-four-only")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.text().contains("sub-two-only"));
    term.stop().unwrap();
}
