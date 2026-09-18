//! Opening a repository must never show `log error` on screen before the
//! real log appears. Red without the fix: the Log/right pane/Refs sidebar
//! were seeded with a client-side placeholder error, held until the first
//! `RefreshOutcome`, and rendered like a real error (see
//! `docs/developer/where-we-are.md`, "Loading state, not an error").
//!
//! The rules are `pty_ui.rs`'s.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STARTUP: Duration = Duration::from_secs(60);
const FIRST_OUTPUT: Duration = Duration::from_secs(20);
const COLS: u16 = 200;
const ROWS: u16 = 50;

/// Small on purpose - `CLAUDE.md`'s "keep test repos small" - one
/// commit is enough to give the Log something real to land on.
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\ntwo\nthree\n", "initial commit")
        .unwrap()
        .build()
}

/// Launches the TUI in a pty against `repo` and, while waiting for the real
/// log, asserts that no intermediate screen shows `log error`. Falsifiable:
/// without the loading-state fix this fails on the very first poll.
#[test]
fn opening_a_repository_never_shows_log_error_before_the_log_appears() {
    let repo = fixture();
    let config = tempfile::tempdir().expect("temp config dir");
    let term = common::launch(repo.root())
        .session("loading-probe")
        .config_dir(config.path())
        .size(COLS, ROWS)
        .spawn();

    term.wait_for_first_output(FIRST_OUTPUT)
        .unwrap_or_else(|e| panic!("the TUI drew nothing: {e}"));

    term.wait_for(
        "the fixture's log, with no `log error` shown on any earlier frame",
        STARTUP,
        |s| {
            let text = s.text();
            assert!(
                !text.contains("log error"),
                "the Log showed an error before the real log loaded - \
                 the \"connecting to the daemon\" placeholder is being \
                 rendered as a failure again:\n{text}"
            );
            text.contains("initial commit")
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

/// A stand-in daemon that reads one request and hangs up without
/// answering - what a stopping daemon, or one from a different build
/// that cannot parse a request, looks like from the TUI's side. Serves
/// until the test process ends.
fn hang_up_daemon() -> u16 {
    use std::io::Read;
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            std::thread::spawn(move || {
                let mut len = [0u8; 4];
                if stream.read_exact(&mut len).is_ok() {
                    let mut payload = vec![0u8; u32::from_be_bytes(len) as usize];
                    let _ = stream.read_exact(&mut payload);
                }
            });
        }
    });
    port
}

/// A window whose daemon connection dies before the first refresh lands
/// must say so (the reason and a Retry), and `r` must recover it. Red
/// without the fix: "Loading repository..." then "Still loading..." forever.
///
/// The session's discovery file is pointed at a stand-in that hangs up, so
/// the link loses its connection mid-request. Then the file is pointed at a
/// port nobody listens on, so the Retry starts its own embedded daemon and
/// the fixture's log must appear. Falsified by removing the reader task's
/// `pending...close(reason)` in `daemon_link.rs`.
#[test]
fn a_lost_daemon_connection_is_an_error_with_retry_not_endless_loading() {
    let repo = fixture();
    let config = tempfile::tempdir().expect("temp config dir");
    let sessions = config.path().join("sessions");
    std::fs::create_dir_all(&sessions).unwrap();
    let port_file = sessions.join("lost-link.port");
    std::fs::write(&port_file, hang_up_daemon().to_string()).unwrap();

    let mut term = common::launch(repo.root())
        .session("lost-link")
        .config_dir(config.path())
        .size(COLS, ROWS)
        .spawn();

    term.wait_for_first_output(FIRST_OUTPUT)
        .unwrap_or_else(|e| panic!("the TUI drew nothing: {e}"));
    term.wait_for(
        "the lost connection shown as an error with a Retry",
        STARTUP,
        |s| {
            let text = s.text();
            text.contains("Lost the connection to the session daemon")
                && text.contains("Press r to retry")
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));

    // Nothing listens here any more: the retry's probe fails and it
    // starts a daemon of its own, exactly as a fresh launch would.
    let dead_port = {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.local_addr().unwrap().port()
    };
    std::fs::write(&port_file, dead_port.to_string()).unwrap();
    term.send_keys("r").expect("retry");

    term.wait_for("the fixture's log, after the retry", STARTUP, |s| {
        let text = s.text();
        !text.contains("Lost the connection") && text.contains("initial commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
