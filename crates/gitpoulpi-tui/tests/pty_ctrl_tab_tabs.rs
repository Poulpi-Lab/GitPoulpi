//! Ctrl+Tab / Ctrl+Shift+Tab switch repository tabs (next / previous), the
//! way a kitty-protocol terminal sends them; plain Tab does not.

mod common;

use std::time::Duration;

use gitpoulpi_session::GlobalSession;
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const SESSION: &str = "pty-ctrl-tab-tabs";

#[test]
fn ctrl_tab_and_ctrl_shift_tab_switch_tabs() {
    let mut keep = Vec::new();
    let mut roots = Vec::new();
    for (name, subject) in [("orbit", "sub-one-only"), ("comet", "sub-two-only")] {
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
    s.activate(0);
    s.save(&file).unwrap();

    let mut term: Terminal = common::launch(&roots[0])
        .session(SESSION)
        .config_dir(config.path())
        .size(200, 50)
        .no_exit_pause()
        .ready("sub-one-only")
        .mouse_capture()
        // Answers the `CSI ? u` query like kitty does, so the app pushes
        // the disambiguate flag: the path a real kitty-protocol terminal takes.
        .kitty_answers()
        .spawn();
    // Ctrl+Tab (CSI u, modifiers 5): the next tab.
    term.send_text("\x1b[9;5u").unwrap();
    term.wait_for("the second tab", STEP, |s| {
        s.contains("sub-two-only") && !s.contains("sub-one-only")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Ctrl+Shift+Tab (modifiers 6): back to the first.
    term.send_text("\x1b[9;6u").unwrap();
    term.wait_for("the first tab again", STEP, |s| {
        s.contains("sub-one-only") && !s.contains("sub-two-only")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
