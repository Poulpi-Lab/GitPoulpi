//! GitPoulpi's MCP server: one tool per `gitpoulpi` CLI leaf, generated
//! from the same clap tree (`registry.rs`) and run through the same
//! `gitpoulpi_commands::execute` the CLI runs through, over the same
//! daemon a TUI on the repository would already be using.
//!
//! # SDK
//!
//! Built on [`rmcp`](https://docs.rs/rmcp) (the official Rust MCP SDK),
//! `server`/`transport-io` features only, with `ServerHandler` implemented
//! by hand in `server.rs` (see there for why no `#[tool]` macros).

pub mod argv;
pub mod http;
pub mod policy;
pub mod registry;
pub mod schema;
pub mod server;

use gitpoulpi_config::DefaultBackend;
pub use gitpoulpi_config::McpConfig;
use rmcp::ServiceExt;

use server::GitPoulpiServer;

/// Refuses when `[mcp] enabled = false`.
pub fn ensure_enabled(cfg: &McpConfig) -> Result<(), String> {
    if cfg.enabled {
        Ok(())
    } else {
        Err(
            "the MCP server is disabled (mcp.enabled = false in config.toml); set \
             mcp.enabled = true (Settings > General) to start it"
                .to_string(),
        )
    }
}

/// Starts the MCP server on stdio for `session`'s daemon (started if
/// needed, and published - see `Session::connect` - so a TUI opened on
/// the same session afterwards joins it). Runs until the peer closes the
/// connection.
pub async fn serve_stdio(
    session: String,
    backend: DefaultBackend,
    cfg: McpConfig,
) -> Result<(), String> {
    let (reader, writer) = rmcp::transport::io::stdio();
    serve(reader, writer, session, backend, cfg).await
}

/// The transport-agnostic core `serve_stdio` wraps around real stdio, so
/// tests can drive the server over an in-process duplex pipe instead.
pub async fn serve<R, W>(
    reader: R,
    writer: W,
    session: String,
    backend: DefaultBackend,
    cfg: McpConfig,
) -> Result<(), String>
where
    R: tokio::io::AsyncRead + Send + Unpin + 'static,
    W: tokio::io::AsyncWrite + Send + Unpin + 'static,
{
    let handler = GitPoulpiServer::connect(session, backend, cfg).await?;
    let running = handler
        .serve((reader, writer))
        .await
        .map_err(|e| format!("mcp server failed to initialize: {e}"))?;
    running
        .waiting()
        .await
        .map_err(|e| format!("mcp server task panicked: {e}"))?;
    Ok(())
}

/// Binds `bind` (empty: `[mcp] bind`, else the default; loopback only unless `allow_remote`), announces the
/// endpoint on stderr (stdout stays untouched), and serves MCP over
/// streamable HTTP until Ctrl+C.
pub async fn serve_http(
    bind: &str,
    allow_remote: bool,
    session: String,
    backend: DefaultBackend,
    cfg: McpConfig,
) -> Result<(), String> {
    ensure_enabled(&cfg)?;
    let bind = http::resolve_bind(bind, &cfg);
    let listener = http::bind(&bind, allow_remote).await?;
    let addr = listener.local_addr().map_err(|e| e.to_string())?;
    eprintln!("gitpoulpi-mcp: serving MCP over HTTP on http://{addr}/mcp (Ctrl+C to stop)");
    let shutdown = tokio_util::sync::CancellationToken::new();
    let on_signal = shutdown.clone();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        on_signal.cancel();
    });
    http::serve(listener, session, backend, cfg, allow_remote, shutdown).await
}
