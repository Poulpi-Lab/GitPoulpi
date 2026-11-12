//! `gitpoulpi daemon start|stop|restart|status` (and its `server` alias).
//!
//! Each test points the binary at its own throwaway config dir and a
//! unique session name, so none can see or stop the developer's real live
//! sessions. `start` spawns a real detached child running this same
//! binary (`gitpoulpi daemon run`), so these exercise the actual
//! background lifecycle.

use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::Value;

/// Generous on purpose: a fresh debug binary can be slow under real-time
/// antivirus scanning, and a slow pass is to be tolerated. Every call is
/// wrapped so a real deadlock fails in ~20s with a clear "timed out"
/// instead of blocking the suite.
const CLI_TIMEOUT: Duration = Duration::from_secs(20);

/// Runs one `gitpoulpi` invocation and returns `(exit ok, parsed JSON
/// stdout)`.
///
/// Deliberately **not** `Command::output()`: `daemon start` spawns a
/// *detached* grandchild, and on Windows a detached process spawned from
/// a process whose stdout is an anonymous pipe can inherit a duplicate
/// handle to that pipe, so the OS never signals EOF and `.output()` blocks
/// forever after the immediate child exits. Redirecting to real files
/// never waits on "every writer closed its handle".
fn run(config_dir: &Path, cwd: &Path, args: &[&str]) -> (bool, Value) {
    let stdout_file = tempfile::NamedTempFile::new().unwrap();
    let stderr_file = tempfile::NamedTempFile::new().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
        .env("GITPOULPI_CONFIG_DIR", config_dir)
        .current_dir(cwd)
        .args(args)
        .stdout(stdout_file.reopen().unwrap())
        .stderr(stderr_file.reopen().unwrap())
        .spawn()
        .unwrap();

    let deadline = Instant::now() + CLI_TIMEOUT;
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "gitpoulpi {args:?} did not exit within {CLI_TIMEOUT:?} \
                 (config_dir={config_dir:?})"
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    };

    let stdout = std::fs::read_to_string(stdout_file.path()).unwrap_or_default();
    let stderr = std::fs::read_to_string(stderr_file.path()).unwrap_or_default();
    let value = serde_json::from_str(&stdout)
        .unwrap_or_else(|err| panic!("not JSON ({err}): stdout={stdout} stderr={stderr}"));
    (status.success(), value)
}

/// Best-effort `daemon stop` on drop, so a test that panics partway
/// through still doesn't leave a detached process running past the test
/// binary's lifetime - config dir and session are both unique per test,
/// so this can never reach a real session.
struct StopOnDrop {
    config: std::path::PathBuf,
    cwd: std::path::PathBuf,
    session: String,
}

impl Drop for StopOnDrop {
    fn drop(&mut self) {
        let _ = Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
            .env("GITPOULPI_CONFIG_DIR", &self.config)
            .current_dir(&self.cwd)
            .args(["-s", &self.session, "daemon", "stop", "--json"])
            .output();
    }
}

fn wait_until<F: Fn() -> bool>(timeout: Duration, cond: F) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if cond() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

#[test]
fn status_reports_not_listening_when_nothing_has_ever_run() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let (ok, json) = run(
        config.path(),
        cwd.path(),
        &["-s", "daemon-status-cold", "daemon", "status", "--json"],
    );
    assert!(ok, "{json}");
    assert_eq!(json["listening"], Value::Bool(false), "{json}");
}

#[test]
fn start_then_status_reports_listening_and_stop_reports_not_listening() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let session = "daemon-start-stop-test";
    let _cleanup = StopOnDrop {
        config: config.path().to_path_buf(),
        cwd: cwd.path().to_path_buf(),
        session: session.to_string(),
    };

    let (ok, started) = run(
        config.path(),
        cwd.path(),
        &["-s", session, "daemon", "start", "--json"],
    );
    assert!(ok, "{started}");
    assert_eq!(started["ok"], Value::Bool(true), "{started}");

    // `start` itself waits for the port to be live before returning, but
    // poll status too rather than assuming `start`'s own wait is exactly
    // what `status` will see a moment later.
    let listening = wait_until(Duration::from_secs(15), || {
        let (ok, json) = run(
            config.path(),
            cwd.path(),
            &["-s", session, "daemon", "status", "--json"],
        );
        ok && json["listening"] == Value::Bool(true)
    });
    assert!(
        listening,
        "daemon never reported itself listening after start"
    );

    let (_ok, status) = run(
        config.path(),
        cwd.path(),
        &["-s", session, "daemon", "status", "--json"],
    );
    assert_eq!(status["kind"], Value::from("standalone"), "{status}");
    assert!(status["pid"].as_u64().is_some(), "{status}");
    // The build this binary was made with (a default build has all of it).
    assert!(status["features"].is_array(), "{status}");
    assert!(
        status["languages"]
            .as_array()
            .is_some_and(|l| l.first() == Some(&Value::from("en"))),
        "{status}"
    );
    assert!(status["port"].as_u64().is_some(), "{status}");

    // A second `start` against the same session must be a no-op that
    // reports the already-running daemon rather than trying (and failing)
    // to bind the port file again.
    let (ok, again) = run(
        config.path(),
        cwd.path(),
        &["-s", session, "daemon", "start", "--json"],
    );
    assert!(ok, "{again}");
    assert_eq!(again["already_running"], Value::Bool(true), "{again}");

    let (ok, stopped) = run(
        config.path(),
        cwd.path(),
        &["-s", session, "daemon", "stop", "--json"],
    );
    assert!(ok, "{stopped}");

    let gone = wait_until(Duration::from_secs(15), || {
        let (ok, json) = run(
            config.path(),
            cwd.path(),
            &["-s", session, "daemon", "status", "--json"],
        );
        ok && json["listening"] == Value::Bool(false)
    });
    assert!(gone, "daemon still reported listening after stop");
}

/// A caller that captures `daemon start`'s output through a pipe (a script's
/// `$(...)`, an agent's subprocess) must see the output end when the command
/// exits: the detached daemon must not keep the pipe's write end open (on
/// Windows it inherited it).
#[test]
fn start_output_captured_through_a_pipe_ends_when_the_command_exits() {
    use std::io::Read;
    use std::process::Stdio;

    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let session = "daemon-start-pipe-test";
    let _cleanup = StopOnDrop {
        config: config.path().to_path_buf(),
        cwd: cwd.path().to_path_buf(),
        session: session.to_string(),
    };
    let mut child = Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .current_dir(cwd.path())
        .args(["-s", session, "daemon", "start", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stdout.read_to_string(&mut text);
        let _ = tx.send(text);
    });
    let text = rx.recv_timeout(CLI_TIMEOUT).unwrap_or_else(|_| {
        let _ = child.kill();
        panic!("the output of `daemon start` never ended: the daemon holds the pipe open")
    });
    let _ = child.wait();
    let started: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}"));
    assert_eq!(started["ok"], Value::Bool(true), "{started}");
}

#[test]
fn stop_all_stops_every_session_with_a_daemon_running() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let sessions = ["daemon-stop-all-a", "daemon-stop-all-b"];
    let _cleanups: Vec<_> = sessions
        .iter()
        .map(|s| StopOnDrop {
            config: config.path().to_path_buf(),
            cwd: cwd.path().to_path_buf(),
            session: s.to_string(),
        })
        .collect();

    for session in sessions {
        let (ok, started) = run(
            config.path(),
            cwd.path(),
            &["-s", session, "daemon", "start", "--json"],
        );
        assert!(ok, "{started}");
        let listening = wait_until(Duration::from_secs(15), || {
            let (ok, json) = run(
                config.path(),
                cwd.path(),
                &["-s", session, "daemon", "status", "--json"],
            );
            ok && json["listening"] == Value::Bool(true)
        });
        assert!(listening, "{session} never came up");
    }

    // `--all` is answered from `session list`'s own discovery, so the
    // `-s` on this particular invocation is irrelevant to which sessions
    // it stops -- pass a third, unrelated name to prove that.
    let (ok, stopped) = run(
        config.path(),
        cwd.path(),
        &[
            "-s",
            "not-one-of-the-two",
            "daemon",
            "stop",
            "--all",
            "--json",
        ],
    );
    assert!(ok, "{stopped}");
    let stopped_names: Vec<&str> = stopped["stopped"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["session"].as_str().unwrap())
        .collect();
    for session in sessions {
        assert!(stopped_names.contains(&session), "{stopped}");
    }

    for session in sessions {
        let gone = wait_until(Duration::from_secs(15), || {
            let (ok, json) = run(
                config.path(),
                cwd.path(),
                &["-s", session, "daemon", "status", "--json"],
            );
            ok && json["listening"] == Value::Bool(false)
        });
        assert!(gone, "{session} still listening after --all stop");
    }
}

#[test]
fn the_server_alias_reaches_the_same_status_command() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let (ok, via_daemon) = run(
        config.path(),
        cwd.path(),
        &["-s", "server-alias-test", "daemon", "status", "--json"],
    );
    assert!(ok, "{via_daemon}");
    let (ok, via_server) = run(
        config.path(),
        cwd.path(),
        &["-s", "server-alias-test", "server", "status", "--json"],
    );
    assert!(ok, "{via_server}");
    assert_eq!(via_daemon, via_server);
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
