use gitpoulpi_core::domain::{OperationKind, RepoId};
use serde::{Deserialize, Serialize};

/// One request sent from a client (TUI/CLI/MCP) to the session daemon.
/// `command` names the GitBackend-mirroring operation (see
/// gitpoulpi_core::domain::OperationKind); `args`/`result` carry the
/// operation-specific payload as untyped JSON, decoded per operation by
/// the daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    pub session: String,
    pub repo: Option<RepoId>,
    pub command: OperationKind,
    pub args: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub request_id: u64,
    pub result: std::result::Result<serde_json::Value, ResponseError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub message: String,
    pub unsupported: bool,
    /// The part of the backend's error a client must be able to match on
    /// (a diverged push, a refused fast-forward), or `None` for an error
    /// that is only ever shown. Rebuild the typed error with
    /// `gitpoulpi_core::Error::from_typed`. Optional on the wire, so a
    /// daemon or client from before it existed still parses.
    #[serde(default)]
    pub kind: Option<gitpoulpi_core::TypedError>,
    /// What went wrong, for an error that came out of a backend operation:
    /// the command that ran, its exit code and stderr (redacted), or
    /// libgit2's class and code. Rebuild the error with
    /// `gitpoulpi_core::Error::from_wire`. Optional on the wire like `kind`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<Box<gitpoulpi_core::ErrorDetail>>,
}

/// Which kind of process a daemon is running in, reported by `Hello` and
/// used by `daemon stop`'s PID fallback: a `Standalone` daemon
/// (`gitpoulpi daemon run`, including the detached process `daemon start`
/// spawns) exists only to serve this session, so killing its PID on a
/// timeout is safe. An `Embedded` daemon runs inside a TUI or CLI
/// command's own process, so the PID fallback must never apply to it (the
/// graceful `Shutdown` request is its only path).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DaemonKind {
    Embedded,
    Standalone,
}

/// Answer to `OperationKind::Hello`: enough for `gitpoulpi daemon status`
/// to report without assuming anything about who is asking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonInfo {
    pub session: String,
    pub pid: u32,
    pub kind: DaemonKind,
    /// Unix seconds.
    pub started_at: i64,
    /// `"git2"` or `"system"` - see `gitpoulpi_daemon`'s `backend_for`.
    pub backend: String,
    pub version: String,
    /// Best-effort short commit sha of the running binary's checkout, when
    /// one could be found (`None` for an installed binary with no `.git`
    /// reachable from its working directory).
    pub git_sha: Option<String>,
}

/// Every message the daemon sends to a connected client: either a direct
/// reply to that client's own `Request`, or a broadcast `Event` pushed to
/// every client subscribed to the session because some client's mutation
/// (possibly a *different* client's) changed repo state (§1.1's live-update
/// promise).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Response(Response),
    Event(crate::events::Event),
}
