//! The streamable-HTTP transport: the same tools as stdio, served by
//! rmcp's `StreamableHttpService` over a hyper HTTP/1 listener.
//!
//! Security model: there is no authentication, so the listener is bound
//! to loopback only. A non-loopback address is refused unless the caller
//! passes `allow_remote` (the binary's `--allow-remote`), which also
//! lifts rmcp's `Host`-header allow-list. Anyone who can reach the port
//! can then run every tool `config.toml`'s `[mcp]` table permits.
//! Local web pages are kept out by that same allow-list (DNS rebinding).
//!
//! The service is stateless with plain JSON responses (no SSE, no
//! `Mcp-Session-Id`): every tool is request/response and all state lives
//! in the daemon, so any MCP client's `initialize` + `tools/*` works.

use std::net::SocketAddr;
use std::sync::Arc;

use gitpoulpi_config::{DefaultBackend, McpConfig};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use hyper_util::service::TowerToHyperService;
use rmcp::transport::streamable_http_server::session::never::NeverSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use crate::server::{GitPoulpiServer, SharedServer};

/// The default listen address of `gitpoulpi-mcp --http`.
pub const DEFAULT_BIND: &str = "127.0.0.1:8808";

/// The address `--http` listens on: an explicit address (`--bind` or
/// `--http ADDR`) wins, then `[mcp] bind`, then [`DEFAULT_BIND`]. An empty
/// `explicit` means "none given"; an empty or legacy `"stdio"` config value
/// counts as unset.
pub fn resolve_bind(explicit: &str, cfg: &McpConfig) -> String {
    let explicit = explicit.trim();
    if !explicit.is_empty() {
        return explicit.to_string();
    }
    match cfg.bind.trim() {
        "" | "stdio" => DEFAULT_BIND.to_string(),
        configured => configured.to_string(),
    }
}

/// Parses `bind` and listens on it, refusing a non-loopback address
/// unless `allow_remote`.
pub async fn bind(bind: &str, allow_remote: bool) -> Result<TcpListener, String> {
    let addr: SocketAddr = bind
        .parse()
        .map_err(|e| format!("invalid bind address `{bind}`: {e}"))?;
    if !addr.ip().is_loopback() && !allow_remote {
        return Err(format!(
            "refusing to listen on non-loopback address {addr}: the MCP HTTP endpoint has no \
             authentication, so anyone who can reach it can run every tool the [mcp] config \
             allows. Pass --allow-remote if you really want that."
        ));
    }
    TcpListener::bind(addr)
        .await
        .map_err(|e| format!("failed to listen on {addr}: {e}"))
}

/// Serves MCP over HTTP (any path; the conventional one is `/mcp`) on
/// `listener` until `shutdown` is canceled.
pub async fn serve(
    listener: TcpListener,
    session: String,
    backend: DefaultBackend,
    cfg: McpConfig,
    allow_remote: bool,
    shutdown: CancellationToken,
) -> Result<(), String> {
    let handler = SharedServer(Arc::new(
        GitPoulpiServer::connect(session, backend, cfg).await?,
    ));
    let mut config = StreamableHttpServerConfig::default()
        .with_stateful_mode(false)
        .with_json_response(true)
        .with_cancellation_token(shutdown.child_token());
    if allow_remote {
        config = config.disable_allowed_hosts();
    }
    let service = StreamableHttpService::new(
        move || Ok(handler.clone()),
        Arc::new(NeverSessionManager::default()),
        config,
    );

    loop {
        let (stream, _) = tokio::select! {
            () = shutdown.cancelled() => return Ok(()),
            accepted = listener.accept() => accepted.map_err(|e| format!("accept failed: {e}"))?,
        };
        let hyper_service = TowerToHyperService::new(service.clone());
        tokio::spawn(async move {
            // Per-connection errors (client hung up mid-request) are not
            // the server's problem.
            let _ = Builder::new(TokioExecutor::new())
                .serve_connection(TokioIo::new(stream), hyper_service)
                .await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(bind: &str) -> McpConfig {
        McpConfig {
            bind: bind.into(),
            ..McpConfig::default()
        }
    }

    #[test]
    fn an_explicit_address_beats_the_config_which_beats_the_default() {
        assert_eq!(
            resolve_bind("127.0.0.1:1", &cfg("127.0.0.1:2")),
            "127.0.0.1:1"
        );
        assert_eq!(resolve_bind("", &cfg("127.0.0.1:2")), "127.0.0.1:2");
        assert_eq!(resolve_bind("", &McpConfig::default()), DEFAULT_BIND);
    }

    /// The old default was the word "stdio"; it and an empty value mean unset.
    #[test]
    fn a_legacy_or_empty_config_bind_means_the_default() {
        assert_eq!(resolve_bind("", &cfg("stdio")), DEFAULT_BIND);
        assert_eq!(resolve_bind("", &cfg("  ")), DEFAULT_BIND);
    }

    /// The config bind goes through the same loopback check as `--http ADDR`.
    #[tokio::test]
    async fn a_configured_non_loopback_bind_still_needs_allow_remote() {
        let addr = resolve_bind("", &cfg("0.0.0.0:0"));
        assert!(bind(&addr, false)
            .await
            .unwrap_err()
            .contains("--allow-remote"));
    }

    #[test]
    fn disabled_config_refuses_to_serve() {
        let off = McpConfig {
            enabled: false,
            ..McpConfig::default()
        };
        let err = crate::ensure_enabled(&off).unwrap_err();
        assert!(err.contains("mcp.enabled"), "{err}");
        assert!(crate::ensure_enabled(&McpConfig::default()).is_ok());
    }
}
