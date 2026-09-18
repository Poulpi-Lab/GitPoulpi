//! Redraw on demand: a TUI with nothing to do writes nothing (the loop must
//! not redraw on a timer).
//!
//! The rules are `pty_ui.rs`'s, except that this test must watch a window of
//! *time*: there is nothing to wait for when the claim is that nothing happens.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;

const SETTLE_TIMEOUT: Duration = Duration::from_secs(40);
/// Settled = quiet for longer than a status message lives (3 s minimum, then
/// the next refresh clears it: one legitimate repaint), which also outlasts
/// the daemon link's 2 s poll.
const QUIET: Duration = Duration::from_secs(6);
const WATCH: Duration = Duration::from_secs(5);

#[test]
fn an_idle_tui_writes_zero_bytes() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "first commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let term = common::launch(repo.root())
        .session("idle-zero-bytes")
        .config_dir(config.path())
        .size(160, 40)
        .no_exit_pause()
        .ready("first commit")
        .spawn();

    // Settle: the first loads, the second outcome and the avatar worker all
    // draw; wait for a stretch with no output at all.
    let deadline = Instant::now() + SETTLE_TIMEOUT;
    let mut last = term.bytes_received();
    let mut quiet_since = Instant::now();
    while quiet_since.elapsed() < QUIET {
        assert!(
            Instant::now() < deadline,
            "the TUI never went quiet: it is still drawing while idle ({} bytes so far, screen:\n{})",
            term.bytes_received(),
            term.text()
        );
        std::thread::sleep(Duration::from_millis(50));
        let now = term.bytes_received();
        if now != last {
            last = now;
            quiet_since = Instant::now();
        }
    }

    // Then a watched window: not a byte.
    let before = term.bytes_received();
    std::thread::sleep(WATCH);
    let after = term.bytes_received();
    assert_eq!(
        after - before,
        0,
        "an idle TUI wrote {} bytes in {WATCH:?}: {:?}",
        after - before,
        String::from_utf8_lossy(&term.raw_output()[before as usize..])
    );
}
