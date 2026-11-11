//! After Ctrl+N the status line lists the keys of the mode, the dialog-opening
//! ones included (theme, language, Settings, ...), from the keymap, on an
//! 80-column terminal, and ends with a hint to the help key if some are left out.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(60);

#[test]
fn the_nav_mode_status_line_names_the_theme_key_and_the_help_key_at_80_columns() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("nav hint probe")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-nav-hint")
        .config_dir(config.path())
        .size(80, 30)
        .ready("nav hint probe")
        .spawn();
    term.send_keys("C-n").unwrap();
    let screen = term
        .wait_for("the Ctrl+N hint", STEP, |s| s.contains("(Esc to cancel)"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = &screen;
    let text = term.text();
    for want in ["y=theme", "g=language", "?="] {
        assert!(text.contains(want), "{want} missing:\n{text}");
    }
    let _ = term.stop();
}
