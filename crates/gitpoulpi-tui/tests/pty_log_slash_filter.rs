//! `/` with the Log focused puts the caret in the Refs filter field (as it
//! does when the Refs pane is focused); inside a text field it is just typed.
//! Whole-state waits.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(30);

/// Falsified: without the `/` arm of `keys_normal_nav` (the Log focused) the
/// key does nothing and the list is never filtered (red at "the Refs list is
/// filtered").
#[test]
fn slash_in_the_log_filters_the_refs_and_is_typed_in_a_text_field() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("slash probe commit")
        .unwrap()
        .branch("zebraalpha")
        .unwrap()
        .branch("zebrabeta")
        .unwrap()
        .build();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-log-slash")
        .size(200, 50)
        .ready("slash probe commit")
        .mouse_capture()
        .start();
    term.wait_for("both branches listed", STEP, |s| {
        s.contains("zebraalpha") && s.contains("zebrabeta")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The Log has the focus (a click on its row); `/` then a ref name.
    let (c, r) = term.wait_for_text("slash probe commit", STEP).unwrap();
    term.click(c + 1, r).unwrap();
    term.send_text("/").unwrap();
    term.send_text("zebraal").unwrap();
    // In the text field `/` is typed, not a second focus request.
    term.send_text("/x").unwrap();
    term.wait_for("the slash typed in the filter", STEP, |s| {
        s.contains("zebraal/x")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("BSpace BSpace Enter").unwrap();
    term.wait_for("the Refs list is filtered", STEP, |s| {
        // Only the matching branch is left in the LOCAL section (the checked
        // out one still has its pill on the Log's row).
        s.contains("LOCAL (1)") && s.contains("zebraalpha")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}
