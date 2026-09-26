use crate::mock::*;
use crate::support::*;

/// `ui.hide_finished_pr_chip` (default on): the toolbar chip of the
/// checked-out branch's request is hidden once it is merged, and the dim
/// "+ MR" chip takes its place; off, the merged request's chip stays.
fn launch_with_merged_head(
    ui_extra: &str,
    session: &str,
) -> (xtask::pty::Terminal, tempfile::TempDir) {
    let mock = start_mock();
    MERGED_HEAD_PORTS.lock().unwrap().push(mock.port);
    let repo = fixture(mock.port);
    let config =
        config_toml_ui(mock.port, "", "auto").replace("[ui]\n", &format!("[ui]\n{ui_extra}"));
    let launched = launch(&repo, session, &config, COLS, "Add the feature");
    // The mock and the repo live as long as the process: leak them for the test.
    std::mem::forget(mock);
    std::mem::forget(repo);
    launched
}

#[test]
fn a_merged_request_hides_the_toolbar_chip_by_default() {
    let (term, _config) = launch_with_merged_head("", "pty-hosting-chip-hidden");
    term.wait_for("the merged request's section", STARTUP, |s| {
        s.contains("MERGE REQUESTS (2)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let screen = term
        .wait_for(
            "the new-request chip instead of the merged one",
            STARTUP,
            |s| s.contains("+ MR"),
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("GL !53"), "{}", screen.text());
}

#[test]
fn turning_the_setting_off_keeps_the_merged_request_chip() {
    let (term, _config) =
        launch_with_merged_head("hide_finished_pr_chip = false\n", "pty-hosting-chip-shown");
    let screen = term
        .wait_for("the merged request's chip", STARTUP, |s| {
            s.contains("GL !53")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("+ MR"), "{}", screen.text());
}
