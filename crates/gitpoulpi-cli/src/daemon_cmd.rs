//! `gitpoulpi daemon start|stop|restart|status` (and its `server` alias).
//!
//! `daemon run` (the foreground entrypoint) stays in `main.rs`; this
//! module is the background half: spawning a detached standalone daemon,
//! asking a running one to shut down (falling back to killing its PID only
//! when it does not answer and recorded one -- an *embedded* daemon never
//! has one, see `ConfigPaths::ipc_pid_file`), and reporting what is running.

use std::net::SocketAddr;
use std::process::Stdio;
use std::time::Duration;

use gitpoulpi_config::{ConfigPaths, DefaultBackend};
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_ipc::{Client, DaemonInfo, DaemonKind, Request};
use serde_json::{json, Value};
use tokio::net::TcpStream;

use gitpoulpi_commands::cli::BackendArg;

/// A connect that hasn't succeeded in this long is not a daemon we could
/// have used anyway - matches `gitpoulpi-cli/src/session.rs`'s probe.
const PROBE_TIMEOUT: Duration = Duration::from_millis(250);
/// How long `daemon stop` waits for the graceful `Shutdown` request to
/// answer before falling back to a PID kill.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
/// How long `daemon start` waits for the detached process to publish its
/// port file and start accepting connections.
const START_TIMEOUT: Duration = Duration::from_secs(5);

fn backend_flag(backend: DefaultBackend) -> &'static str {
    match backend {
        DefaultBackend::Git2 => "git2",
        DefaultBackend::System => "system",
    }
}

fn run_async<F: std::future::Future>(fut: F) -> F::Output {
    tokio::runtime::Runtime::new()
        .expect("failed to start tokio runtime")
        .block_on(fut)
}

fn print_result(json_output: bool, payload: Value, human: impl FnOnce(&Value) -> String) {
    if json_output {
        println!("{payload}");
    } else {
        println!("{}", human(&payload));
    }
}

fn fail(json_output: bool, message: String) -> ! {
    if json_output {
        println!("{}", json!({ "error": message }));
    } else {
        eprintln!("gitpoulpi: {message}");
    }
    std::process::exit(1);
}

fn config_paths(json_output: bool) -> ConfigPaths {
    match ConfigPaths::discover() {
        Some(paths) => paths,
        None => fail(
            json_output,
            "could not determine this OS's config directory".into(),
        ),
    }
}

/// A TCP connect, not a file check - see `session.rs`'s twin of this.
async fn is_listening(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tokio::time::timeout(PROBE_TIMEOUT, TcpStream::connect(addr))
        .await
        .map(|r| r.is_ok())
        .unwrap_or(false)
}

/// The port this session's daemon is actually listening on, if its
/// discovery file names one that is still live.
async fn live_port(paths: &ConfigPaths, session: &str) -> Option<u16> {
    let raw = std::fs::read_to_string(paths.ipc_port_file(session)).ok()?;
    let port: u16 = raw.trim().parse().ok()?;
    is_listening(port).await.then_some(port)
}

/// The endpoint of `session`'s daemon on `port`, with its token.
fn endpoint_of(paths: &ConfigPaths, session: &str, port: u16) -> gitpoulpi_ipc::Endpoint {
    gitpoulpi_ipc::discovery::endpoint_for_port(&paths.ipc_port_file(session), port)
}

async fn hello(endpoint: &gitpoulpi_ipc::Endpoint, session: &str) -> Result<DaemonInfo, String> {
    let mut client = Client::connect(endpoint)
        .await
        .map_err(|e| format!("could not reach the daemon: {e}"))?;
    let response = client
        .call(&Request {
            id: 1,
            session: session.to_string(),
            repo: None,
            command: OperationKind::Hello,
            args: Value::Null,
        })
        .await
        .map_err(|e| format!("daemon connection lost: {e}"))?;
    match response.result {
        Ok(value) => {
            serde_json::from_value(value).map_err(|e| format!("malformed Hello response: {e}"))
        }
        Err(e) => Err(e.message),
    }
}

// ---------------------------------------------------------------------
// status
// ---------------------------------------------------------------------

pub fn status(session: &str, json_output: bool) {
    let paths = config_paths(json_output);
    run_async(status_one(&paths, session, json_output));
}

async fn status_one(paths: &ConfigPaths, session: &str, json_output: bool) {
    let Some(port) = live_port(paths, session).await else {
        print_result(
            json_output,
            json!({ "session": session, "listening": false }),
            |_| format!("{session}: not running"),
        );
        return;
    };

    match hello(&endpoint_of(paths, session, port), session).await {
        Ok(info) => {
            let uptime = (gitpoulpi_core::time::unix_now() - info.started_at).max(0);
            let mut payload = json!({
                "session": session,
                "listening": true,
                "port": port,
                "pid": info.pid,
                "kind": match info.kind { DaemonKind::Embedded => "embedded", DaemonKind::Standalone => "standalone" },
                "uptime_seconds": uptime,
                "backend": info.backend,
                "version": info.version,
                "git_sha": info.git_sha,
            });
            // What this binary was built with (the daemon runs the same
            // build unless a mixed install says otherwise).
            if let (Some(map), serde_json::Value::Object(extra)) = (
                payload.as_object_mut(),
                gitpoulpi_commands::build_features::status_fields(),
            ) {
                map.extend(extra);
            }
            print_result(json_output, payload, |_| {
                format!(
                    "{session}: listening on 127.0.0.1:{port} (pid {}, {}, {} backend, v{}{}, up {})",
                    info.pid,
                    match info.kind {
                        DaemonKind::Embedded => "embedded",
                        DaemonKind::Standalone => "standalone",
                    },
                    info.backend,
                    info.version,
                    info.git_sha
                        .as_deref()
                        .map(|s| format!(" @{s}"))
                        .unwrap_or_default(),
                    format_uptime(uptime),
                )
            });
        }
        Err(message) => {
            // Listening but Hello failed (most likely a daemon binary that
            // predates the request): report "listening", not a hard error.
            print_result(
                json_output,
                json!({ "session": session, "listening": true, "port": port, "error": message }),
                |_| format!("{session}: listening on 127.0.0.1:{port} ({message})"),
            );
        }
    }
}

fn format_uptime(seconds: i64) -> String {
    let seconds = seconds.max(0) as u64;
    let (h, m, s) = (seconds / 3600, (seconds % 3600) / 60, seconds % 60);
    if h > 0 {
        format!("{h}h{m:02}m")
    } else if m > 0 {
        format!("{m}m{s:02}s")
    } else {
        format!("{s}s")
    }
}

// ---------------------------------------------------------------------
// stop
// ---------------------------------------------------------------------

pub fn stop(current_session: &str, all: bool, json_output: bool) {
    let paths = config_paths(json_output);
    let sessions: Vec<String> = if all {
        gitpoulpi_config::list_session_port_files(&paths.sessions_dir())
            .into_iter()
            .map(|(name, _)| name)
            .collect()
    } else {
        vec![current_session.to_string()]
    };

    if all && sessions.is_empty() {
        print_result(json_output, json!({ "stopped": [] }), |_| {
            "no sessions have a daemon publishing a port".to_string()
        });
        return;
    }

    let reports = run_async(async {
        let mut reports = Vec::new();
        for name in &sessions {
            reports.push(stop_one(&paths, name).await);
        }
        reports
    });

    let any_failed = reports.iter().any(|r| r["ok"] == json!(false));
    let payload = json!({ "stopped": reports });
    print_result(json_output, payload, |v| {
        v["stopped"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                format!(
                    "{}: {}",
                    r["session"].as_str().unwrap_or("?"),
                    r["detail"].as_str().unwrap_or("?")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    });
    if any_failed {
        std::process::exit(1);
    }
}

async fn stop_one(paths: &ConfigPaths, session: &str) -> Value {
    let port_file = paths.ipc_port_file(session);
    let Ok(raw) = std::fs::read_to_string(&port_file) else {
        return json!({ "session": session, "ok": true, "method": "none", "detail": "not running" });
    };
    let Ok(port) = raw.trim().parse::<u16>() else {
        gitpoulpi_ipc::discovery::remove_files(&port_file);
        return json!({ "session": session, "ok": true, "method": "none", "detail": "not running (stale port file removed)" });
    };
    if !is_listening(port).await {
        gitpoulpi_ipc::discovery::remove_files(&port_file);
        return json!({ "session": session, "ok": true, "method": "none", "detail": "not running (stale files removed)" });
    }

    let graceful = tokio::time::timeout(
        SHUTDOWN_TIMEOUT,
        send_shutdown(&endpoint_of(paths, session, port), session),
    )
    .await;
    if matches!(graceful, Ok(Ok(()))) {
        // The daemon removes both discovery files itself; wait briefly for
        // the port to stop answering so `daemon stop && daemon start` does
        // not race the old process's teardown.
        wait_until_gone(port, Duration::from_secs(3)).await;
        return json!({ "session": session, "ok": true, "method": "graceful", "detail": "stopped" });
    }

    let pid_file = paths.ipc_pid_file(session);
    match std::fs::read_to_string(&pid_file)
        .ok()
        .and_then(|raw| raw.trim().parse::<u32>().ok())
    {
        Some(pid) => {
            kill_pid(pid);
            gitpoulpi_ipc::discovery::remove_files(&port_file);
            json!({
                "session": session, "ok": true, "method": "killed", "pid": pid,
                "detail": format!("did not answer in time, killed pid {pid}"),
            })
        }
        None => json!({
            "session": session, "ok": false, "method": "none",
            "detail": "did not answer in time and no PID was recorded for it (an embedded \
                       daemon - e.g. inside a running TUI - only stops via the graceful \
                       request; it will not be killed)",
        }),
    }
}

async fn send_shutdown(endpoint: &gitpoulpi_ipc::Endpoint, session: &str) -> Result<(), String> {
    let mut client = Client::connect(endpoint)
        .await
        .map_err(|e| format!("could not reach the daemon: {e}"))?;
    let response = client
        .call(&Request {
            id: 1,
            session: session.to_string(),
            repo: None,
            command: OperationKind::Shutdown,
            args: Value::Null,
        })
        .await
        .map_err(|e| format!("daemon connection lost: {e}"))?;
    response.result.map(|_| ()).map_err(|e| e.message)
}

async fn wait_until_gone(port: u16, timeout: Duration) {
    let deadline = tokio::time::Instant::now() + timeout;
    while tokio::time::Instant::now() < deadline {
        if !is_listening(port).await {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[cfg(windows)]
fn kill_pid(pid: u32) {
    let _ = std::process::Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(unix)]
fn kill_pid(pid: u32) {
    let _ = std::process::Command::new("kill")
        .args(["-9", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

// ---------------------------------------------------------------------
// start / restart
// ---------------------------------------------------------------------

pub fn start(
    session: &str,
    backend: DefaultBackend,
    backend_flag_given: Option<BackendArg>,
    json_output: bool,
) {
    let paths = config_paths(json_output);
    let report = run_async(start_one(&paths, session, backend, backend_flag_given));
    let ok = report["ok"] != json!(false);
    print_result(json_output, report, |v| {
        v["detail"].as_str().unwrap_or("?").to_string()
    });
    if !ok {
        std::process::exit(1);
    }
}

pub fn restart(
    session: &str,
    backend: DefaultBackend,
    backend_flag_given: Option<BackendArg>,
    json_output: bool,
) {
    let paths = config_paths(json_output);
    let (stop_report, start_report) = run_async(async {
        let stop_report = stop_one(&paths, session).await;
        let start_report = start_one(&paths, session, backend, backend_flag_given).await;
        (stop_report, start_report)
    });
    let ok = start_report["ok"] != json!(false);
    let payload = json!({ "stopped": stop_report, "started": start_report });
    print_result(json_output, payload, |v| {
        format!(
            "{}\n{}",
            v["stopped"]["detail"].as_str().unwrap_or("?"),
            v["started"]["detail"].as_str().unwrap_or("?"),
        )
    });
    if !ok {
        std::process::exit(1);
    }
}

async fn start_one(
    paths: &ConfigPaths,
    session: &str,
    backend: DefaultBackend,
    backend_flag_given: Option<BackendArg>,
) -> Value {
    if let Some(port) = live_port(paths, session).await {
        return json!({
            "session": session, "ok": true, "already_running": true, "port": port,
            "detail": format!("{session}: already running on 127.0.0.1:{port}"),
        });
    }

    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(e) => {
            return json!({
                "session": session, "ok": false,
                "detail": format!("could not find this binary's own path to relaunch it: {e}"),
            })
        }
    };

    let mut cmd = std::process::Command::new(exe);
    cmd.args(["daemon", "run", "--session", session]);
    // Pin `--backend` only when the caller passed one: a daemon started
    // without it resolves `config.toml` itself, like `daemon run` typed
    // directly, rather than freezing this process's resolved value.
    if backend_flag_given.is_some() {
        cmd.args(["--backend", backend_flag(backend)]);
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    detach(&mut cmd);

    let child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            return json!({
                "session": session, "ok": false,
                "detail": format!("failed to spawn the detached daemon process: {e}"),
            })
        }
    };
    let spawned_pid = child.id();
    // Deliberately not waited on: a detached daemon outlives this
    // command, so this process must not hold its `Child` (which would
    // only matter for reaping a zombie on Unix if we *did* wait, which we
    // never will).
    std::mem::drop(child);

    let port_file = paths.ipc_port_file(session);
    let deadline = tokio::time::Instant::now() + START_TIMEOUT;
    while tokio::time::Instant::now() < deadline {
        if let Some(port) = live_port(paths, session).await {
            return json!({
                "session": session, "ok": true, "already_running": false,
                "port": port, "pid": spawned_pid,
                "detail": format!("{session}: started, listening on 127.0.0.1:{port} (pid {spawned_pid})"),
            });
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let _ = port_file; // read only through `live_port` above; named for the message below
    json!({
        "session": session, "ok": false,
        "detail": format!(
            "spawned pid {spawned_pid} but it did not publish a listening port within {}s",
            START_TIMEOUT.as_secs()
        ),
    })
}

#[cfg(windows)]
fn detach(cmd: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    // DETACHED_PROCESS: no console inherited, so a closed parent terminal
    // cannot send this process a Ctrl-C/close signal through it.
    // CREATE_NEW_PROCESS_GROUP: not part of the parent's process group
    // either, for the same reason.
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
}

#[cfg(unix)]
fn detach(cmd: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    // Starts a new session (`setsid(2)`) so the child has no controlling
    // terminal and isn't in this process's process group -- a Ctrl-C sent
    // to this CLI invocation's terminal, or the terminal itself closing,
    // must not take the daemon down with it.
    unsafe {
        cmd.pre_exec(|| {
            // Safety: async-signal-safe, called after fork and before
            // exec, as `pre_exec` requires.
            extern "C" {
                fn setsid() -> i32;
            }
            setsid();
            Ok(())
        });
    }
}
