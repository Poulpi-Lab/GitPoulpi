//! The daemon's token gate over a real loopback connection, and the
//! user-only files a standalone daemon publishes. The check itself is unit
//! tested over `duplex` in `gitpoulpi-ipc`; this proves the daemon runs it
//! before reading anything else, and that the files it writes are private.
//!
//! `ConfigPaths::for_dir` over a tempdir throughout, never the real config.

use std::sync::Arc;
use std::time::Duration;

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_config::ConfigPaths;
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::auth::{handshake, hello_request, AUTH_REQUIRED_MESSAGE};
use gitpoulpi_ipc::transport::{read_message, write_message};
use gitpoulpi_ipc::{Client, Endpoint, Request, ServerMessage, Token};
use tokio::net::TcpStream;

fn backend() -> Arc<dyn gitpoulpi_core::GitBackend> {
    Arc::new(SystemGitBackend::new())
}

fn ping(id: u64) -> Request {
    Request {
        id,
        session: "auth-test".into(),
        repo: None,
        command: OperationKind::JobList,
        args: serde_json::Value::Null,
    }
}

async fn raw(port: u16) -> TcpStream {
    TcpStream::connect(("127.0.0.1", port)).await.unwrap()
}

#[tokio::test]
async fn a_client_with_the_token_is_served() {
    let handle = Daemon::new("auth-test").spawn(backend()).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let answer = client.call(&ping(1)).await.unwrap();
    assert!(answer.result.is_ok());
    handle.shutdown().await.unwrap();
}

#[tokio::test]
async fn a_request_before_the_hello_is_refused_and_the_connection_closed() {
    let handle = Daemon::new("auth-test").spawn(backend()).await.unwrap();
    let mut stream = raw(handle.port).await;
    write_message(&mut stream, &ping(1)).await.unwrap();
    let Some(ServerMessage::Response(response)) = read_message(&mut stream).await.unwrap() else {
        panic!("a refusal is a response");
    };
    let error = response.result.unwrap_err();
    assert!(error.auth_required);
    assert_eq!(error.message, AUTH_REQUIRED_MESSAGE);
    let closed: Option<ServerMessage> = read_message(&mut stream).await.unwrap_or(None);
    assert!(closed.is_none(), "the daemon hangs up after refusing");
    handle.shutdown().await.unwrap();
}

#[tokio::test]
async fn a_wrong_token_and_a_missing_token_are_refused() {
    let handle = Daemon::new("auth-test").spawn(backend()).await.unwrap();
    for token in [Some(Token::generate().unwrap()), None] {
        let mut stream = raw(handle.port).await;
        let result = handshake(&mut stream, token.as_ref()).await;
        assert!(
            matches!(&result, Err(gitpoulpi_ipc::TransportError::Refused(m)) if m == AUTH_REQUIRED_MESSAGE),
            "{result:?}"
        );
    }
    handle.shutdown().await.unwrap();
}

#[tokio::test]
async fn a_later_hello_on_an_authenticated_connection_is_still_answered() {
    let handle = Daemon::new("auth-test").spawn(backend()).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let mut hello = hello_request(None);
    hello.id = 5;
    let answer = client.call(&hello).await.unwrap();
    assert!(
        answer.result.is_ok(),
        "the build check's Hello keeps working"
    );
    handle.shutdown().await.unwrap();
}

async fn standalone(paths: &ConfigPaths, session: &str) -> tokio::task::JoinHandle<()> {
    let daemon = Daemon::new(session);
    let paths = paths.clone();
    let task = tokio::spawn(async move {
        daemon.run_foreground_in(backend(), &paths).await.unwrap();
    });
    task
}

async fn wait_for(path: &std::path::Path) {
    for _ in 0..200 {
        if path.exists() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("{path:?} never appeared");
}

#[tokio::test]
async fn a_standalone_daemon_publishes_its_token_before_the_port_and_removes_it_after() {
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    let session = "auth-files";
    let (token_file, port_file) = (paths.ipc_token_file(session), paths.ipc_port_file(session));
    let task = standalone(&paths, session).await;

    // Whenever the port file is visible, the token file already is.
    wait_for(&port_file).await;
    assert!(token_file.exists(), "the token precedes the port file");
    let text = std::fs::read_to_string(&token_file).unwrap();
    assert_eq!(text.trim().len(), 64);
    assert!(Token::from_hex(&text).is_some());

    let endpoint = gitpoulpi_ipc::discovery::endpoint(&port_file).unwrap();
    assert!(endpoint.token.is_some());
    let mut client = Client::connect(&endpoint).await.unwrap();
    let mut stop = ping(9);
    stop.command = OperationKind::Shutdown;
    client.call(&stop).await.unwrap();
    tokio::time::timeout(Duration::from_secs(5), task)
        .await
        .unwrap()
        .unwrap();
    assert!(!token_file.exists(), "the token goes with the port file");
    assert!(!port_file.exists());
}

#[tokio::test]
async fn a_client_without_the_token_file_is_refused_by_a_standalone_daemon() {
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    let session = "auth-nofile";
    let port_file = paths.ipc_port_file(session);
    let task = standalone(&paths, session).await;
    wait_for(&port_file).await;
    let port = gitpoulpi_ipc::discovery::endpoint(&port_file).unwrap().port;
    let blind = Endpoint::new(port, None);
    assert!(Client::connect(&blind).await.is_err());
    let mut client = Client::connect(&gitpoulpi_ipc::discovery::endpoint(&port_file).unwrap())
        .await
        .unwrap();
    let mut stop = ping(9);
    stop.command = OperationKind::Shutdown;
    client.call(&stop).await.unwrap();
    let _ = tokio::time::timeout(Duration::from_secs(5), task).await;
}

#[cfg(unix)]
#[tokio::test]
async fn the_published_files_are_user_only() {
    use std::os::unix::fs::PermissionsExt;
    let mode = |p: &std::path::Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    let session = "auth-modes";
    let port_file = paths.ipc_port_file(session);
    let task = standalone(&paths, session).await;
    wait_for(&port_file).await;
    wait_for(&paths.ipc_pid_file(session)).await;
    assert_eq!(mode(&paths.ipc_token_file(session)), 0o600);
    assert_eq!(mode(&port_file), 0o600);
    assert_eq!(mode(&paths.ipc_pid_file(session)), 0o600);
    assert_eq!(mode(&paths.sessions_dir()), 0o700);

    let mut client = Client::connect(&gitpoulpi_ipc::discovery::endpoint(&port_file).unwrap())
        .await
        .unwrap();
    let mut stop = ping(9);
    stop.command = OperationKind::Shutdown;
    client.call(&stop).await.unwrap();
    let _ = tokio::time::timeout(Duration::from_secs(5), task).await;
}
