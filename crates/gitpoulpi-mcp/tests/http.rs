//! The streamable-HTTP transport on an ephemeral loopback port, driven by
//! a hand-rolled HTTP/1.1 client (no extra client dependencies): the
//! same tools as stdio, and the loopback-only guard.

use std::sync::OnceLock;

use gitpoulpi_config::{DefaultBackend, McpConfig};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;

fn isolated_config_dir() -> &'static std::path::Path {
    static DIR: OnceLock<TempDir> = OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().expect("could not create a temp config dir"))
        .path()
}

/// POSTs one JSON-RPC message to `/mcp` and returns (status line, JSON body).
async fn post(addr: std::net::SocketAddr, message: Value) -> (String, Option<Value>) {
    let body = message.to_string();
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let request = format!(
        "POST /mcp HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\n\
         Accept: application/json, text/event-stream\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.unwrap();
    let text = String::from_utf8_lossy(&raw).into_owned();
    let status = text.lines().next().unwrap_or_default().to_string();
    // The body may be chunked; the JSON object is what lies between the
    // first `{` after the headers and the last `}`.
    let payload = text.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or("");
    let json = match (payload.find('{'), payload.rfind('}')) {
        (Some(a), Some(b)) if a < b => serde_json::from_str(&payload[a..=b]).ok(),
        _ => None,
    };
    (status, json)
}

async fn start(session: &str) -> (std::net::SocketAddr, CancellationToken) {
    std::env::set_var("GITPOULPI_CONFIG_DIR", isolated_config_dir());
    let listener = gitpoulpi_mcp::http::bind("127.0.0.1:0", false)
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let shutdown = CancellationToken::new();
    let token = shutdown.clone();
    let session = session.to_string();
    tokio::spawn(async move {
        let _ = gitpoulpi_mcp::http::serve(
            listener,
            session,
            DefaultBackend::default(),
            McpConfig::default(),
            false,
            token,
        )
        .await;
    });
    (addr, shutdown)
}

#[tokio::test]
async fn tools_list_and_a_read_tool_work_over_http() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .build();
    let (addr, shutdown) = start("mcp-http-roundtrip").await;

    let (status, init) = post(
        addr,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": { "name": "http-test", "version": "0" } } }),
    )
    .await;
    assert!(status.contains("200"), "{status}");
    assert!(init.unwrap()["result"]["serverInfo"]["name"] == "gitpoulpi-mcp");

    let (status, list) = post(
        addr,
        json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {} }),
    )
    .await;
    assert!(status.contains("200"), "{status}");
    let list = list.expect("tools/list returned no JSON");
    let tools = list["result"]["tools"].as_array().unwrap();
    assert!(tools.iter().any(|t| t["name"] == "status"));

    let (_, call) = post(
        addr,
        json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {
            "name": "status",
            "arguments": { "repo": repo.root().to_string_lossy() } } }),
    )
    .await;
    let result = &call.expect("tools/call returned no JSON")["result"];
    assert_ne!(result.get("isError"), Some(&json!(true)), "{result}");
    assert!(result["structuredContent"]["files"].is_array(), "{result}");

    shutdown.cancel();
}

#[tokio::test]
async fn non_loopback_bind_needs_allow_remote() {
    let err = gitpoulpi_mcp::http::bind("0.0.0.0:0", false)
        .await
        .expect_err("a non-loopback bind must be refused");
    assert!(err.contains("--allow-remote"), "{err}");
    assert!(gitpoulpi_mcp::http::bind("0.0.0.0:0", true).await.is_ok());
    assert!(gitpoulpi_mcp::http::bind("not an address", false)
        .await
        .is_err());
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
