//! Leaving when the updater stops the session (the install-channels design).
//!
//! The MCP server hosts (or joins) its session's daemon. The updater stops
//! that daemon with `Shutdown {reason: "update"}`, which broadcasts
//! `Event::ShuttingDown`; a server that kept running would hold the install
//! folder open and, on its next tool call, start a fresh daemon from the old
//! binaries. So it watches the stream and exits for good.

use gitpoulpi_ipc::{transport, Client, Endpoint, Event, ServerMessage, ShutdownArgs};

/// Waits on the daemon's event stream until it announces an update
/// shutdown (`true`) or the connection ends without one (`false`).
pub async fn wait_for_update_shutdown(endpoint: &Endpoint) -> bool {
    let Ok(client) = Client::connect(endpoint).await else {
        return false;
    };
    let mut stream = client.into_stream();
    loop {
        match transport::read_message::<_, ServerMessage>(&mut stream).await {
            Ok(Some(ServerMessage::Event(Event::ShuttingDown { reason, .. })))
                if reason == ShutdownArgs::UPDATE =>
            {
                return true
            }
            Ok(Some(_)) => continue,
            Ok(None) | Err(_) => return false,
        }
    }
}

/// Starts the watcher for this session's published daemon; the process ends
/// when the updater asks for it. No published daemon, no watcher: an
/// unpublished embedded daemon is stopped by the updater killing the process.
pub fn spawn(session: &str) {
    let Some(port_file) =
        gitpoulpi_config::ConfigPaths::discover().map(|p| p.ipc_port_file(session))
    else {
        return;
    };
    tokio::spawn(async move {
        let Some(endpoint) = gitpoulpi_ipc::discovery::live_endpoint(&port_file).await else {
            return;
        };
        if wait_for_update_shutdown(&endpoint).await {
            eprintln!("gitpoulpi-mcp: closing for an update");
            std::process::exit(0);
        }
    });
}
