//! A window that finds its session's daemon on another build takes the
//! session over instead of showing a dead-end error (`session-and-state.md`,
//! "Hello and the build check").
//!
//! The rule, in `takeover_stale_daemon`: both builds known and different,
//! the old daemon has no unfinished job -> ask it to shut down, wait until
//! its port is closed, then start this process's own embedded daemon and
//! publish it. Anything short of that (jobs running, the shutdown unanswered,
//! the old daemon still alive after the wait) leaves the daemon alone and the
//! caller falls back to the mismatch error. A process is never killed.
//!
//! Ping-pong guard: `link_loop` calls this once per link start and then goes
//! through `link_loop_with_port`, whose own `Hello` check still returns the
//! plain mismatch error without a second takeover. So a window never shuts
//! down the daemon it has just started (same build, nothing to do), and two
//! windows of different builds cannot fight in a loop: the older one only
//! takes over again if its user presses `r`.

use super::*;

/// How long the old daemon gets to close its port after `Shutdown`.
const GONE_WITHIN: Duration = Duration::from_secs(3);
/// Bound on each request to the old daemon (an old build may hang up).
const CALL_TIMEOUT: Duration = Duration::from_secs(3);

pub(super) enum Takeover {
    /// Same build, unknown build, or nothing we could safely do: connect to
    /// the port as it is (the normal `Hello` check then reports a mismatch).
    Unchanged,
    /// Jobs are running on the other build's daemon; the message says so.
    Busy(String),
    /// The old daemon is gone and this process serves the session on `port`.
    Restarted { port: u16, notice: String },
}

pub(super) async fn takeover_stale_daemon(
    port: u16,
    session: &str,
    port_file: Option<&std::path::Path>,
) -> Takeover {
    let Some(my_sha) = this_build_git_sha() else {
        return Takeover::Unchanged;
    };
    let Ok(info) = ask::<gitpoulpi_ipc::DaemonInfo>(port, session, OperationKind::Hello).await
    else {
        return Takeover::Unchanged;
    };
    let Some(daemon_sha) = info.git_sha else {
        return Takeover::Unchanged;
    };
    if daemon_sha == my_sha {
        return Takeover::Unchanged;
    }
    let Ok(jobs) = ask::<Vec<gitpoulpi_ipc::JobInfo>>(port, session, OperationKind::JobList).await
    else {
        return Takeover::Unchanged;
    };
    if jobs.iter().any(|job| !job.state.is_finished()) {
        return Takeover::Busy(gitpoulpi_i18n::msg::link_error::build_mismatch_jobs(
            &daemon_sha,
            &my_sha,
        ));
    }
    if ask::<serde_json::Value>(port, session, OperationKind::Shutdown)
        .await
        .is_err()
    {
        return Takeover::Unchanged;
    }
    let deadline = tokio::time::Instant::now() + GONE_WITHIN;
    while gitpoulpi_ipc::discovery::is_listening(port).await {
        if tokio::time::Instant::now() >= deadline {
            return Takeover::Unchanged;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    // An embedded daemon cannot remove the files its host process wrote; a
    // standalone one already did. Only ours-by-port, a newer daemon's stay.
    if let Some(port_file) = port_file {
        gitpoulpi_ipc::discovery::remove_if_ours(port_file, port);
    }
    match ensure_daemon_at(port_file, session).await {
        Ok(new_port) => Takeover::Restarted {
            port: new_port,
            notice: gitpoulpi_i18n::msg::link_error::daemon_restarted(&daemon_sha, &my_sha),
        },
        Err(_) => Takeover::Unchanged,
    }
}

/// One bounded request on a connection of its own (not the link's).
async fn ask<T: serde::de::DeserializeOwned>(
    port: u16,
    session: &str,
    command: OperationKind,
) -> Result<T, ()> {
    let call = async {
        let mut client = gitpoulpi_ipc::Client::connect_port(port).await.ok()?;
        let response = client
            .call(&Request {
                id: 1,
                session: session.to_string(),
                repo: None,
                command,
                args: serde_json::json!({}),
            })
            .await
            .ok()?;
        serde_json::from_value(response.result.ok()?).ok()
    };
    tokio::time::timeout(CALL_TIMEOUT, call)
        .await
        .ok()
        .flatten()
        .ok_or(())
}
