//! `PtySession` against a real child process (unix: `sh`, Windows: PowerShell).

use std::time::{Duration, Instant};

use gitpoulpi_pty::{PtySession, SpawnOptions};

/// Options running `script`: `sh -c` on unix, Windows PowerShell on Windows.
fn shell(unix_script: &str, windows_script: &str) -> SpawnOptions {
    if cfg!(windows) {
        let mut opts = SpawnOptions::new("powershell.exe");
        opts.args = vec![
            "-NoProfile".into(),
            "-NonInteractive".into(),
            "-Command".into(),
            windows_script.into(),
        ];
        opts
    } else {
        let mut opts = SpawnOptions::new("sh");
        opts.args = vec!["-c".into(), unix_script.into()];
        opts
    }
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("timed out waiting for {what}");
}

#[test]
fn history_that_scrolled_off_is_read_back_in_order() {
    let mut opts = shell(
        "seq 1 50; sleep 30",
        "1..50 | ForEach-Object { $_ }; Start-Sleep 30",
    );
    opts.rows = 20;
    opts.cols = 40;
    opts.scrollback = 100;
    let session = PtySession::spawn(opts).unwrap();
    wait_until("the output", || session.screen_text().contains("50"));
    let lines = session.full_lines();
    // Only one screenful of history can be looked back at (vt100 0.15), so
    // the tail is complete and in order.
    assert_eq!(lines.last().map(String::as_str), Some("50"));
    let numbers: Vec<u32> = lines.iter().filter_map(|l| l.parse().ok()).collect();
    assert!(numbers.len() >= 20, "{numbers:?}");
    assert!(numbers.windows(2).all(|w| w[1] == w[0] + 1), "{numbers:?}");
}

#[test]
fn a_long_line_that_the_terminal_wrapped_is_joined_back() {
    let long = "x".repeat(70);
    let mut opts = shell(
        &format!("echo {long}; sleep 30"),
        &format!("Write-Output {long}; Start-Sleep 30"),
    );
    opts.rows = 10;
    opts.cols = 30;
    let session = PtySession::spawn(opts).unwrap();
    wait_until("the line", || {
        session.screen_text().matches('x').count() >= 70
    });
    assert!(session.full_lines().contains(&long));
}

#[test]
fn stop_kills_the_child_and_reports_the_exit() {
    let opts = shell("sleep 60", "Start-Sleep 60");
    let mut session = PtySession::spawn(opts).unwrap();
    assert!(session.exited().is_none());
    session.stop();
    wait_until("the exit", || session.exited().is_some());
}
