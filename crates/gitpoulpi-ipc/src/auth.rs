//! The daemon token: what a client must present first on every connection.
//!
//! Loopback TCP reaches every account on the machine, so a daemon draws a
//! random token at start, publishes it in a user-only file next to its port
//! file (`sessions/<session>.token`, written before the port file) and refuses
//! a connection whose first request is not `Hello` carrying that token. A
//! process that cannot read the user's files cannot speak to the daemon.
//!
//! [`authenticate`] is the daemon's check, [`handshake`] the client's side;
//! both are generic over the stream so they are tested over `duplex`.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use tokio::io::{AsyncRead, AsyncWrite};

use gitpoulpi_core::domain::OperationKind;

use crate::protocol::{DaemonInfo, Request, Response, ResponseError, ServerMessage};
use crate::transport::{read_message, write_message, TransportError};

/// How long a new connection gets to send its `Hello`.
pub const AUTH_TIMEOUT: Duration = Duration::from_secs(5);

/// The request id of the handshake `Hello` (never a client's own id).
pub const HANDSHAKE_REQUEST_ID: u64 = u64::MAX;

/// What a refused connection is told. An older client prints it as is.
pub const AUTH_REQUIRED_MESSAGE: &str = "This session daemon requires authentication from a client of the same version or newer. Restart this window or command after updating.";

/// 32 random bytes, written as 64 hex characters.
#[derive(Clone, PartialEq, Eq)]
pub struct Token([u8; 32]);

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Token(..)")
    }
}

impl Token {
    /// A fresh token from the operating system's random source.
    pub fn generate() -> std::io::Result<Self> {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).map_err(|e| std::io::Error::other(e.to_string()))?;
        Ok(Self(bytes))
    }

    /// The token a file or a request holds; `None` unless it is exactly 64 hex
    /// digits.
    pub fn from_hex(text: &str) -> Option<Self> {
        let text = text.trim();
        if text.len() != 64 || !text.is_ascii() {
            return None;
        }
        let mut bytes = [0u8; 32];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&text[2 * i..2 * i + 2], 16).ok()?;
        }
        Some(Self(bytes))
    }

    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// Constant-time comparison.
    pub fn matches(&self, other: &Token) -> bool {
        self.0[..].ct_eq(&other.0[..]).into()
    }
}

/// Where a daemon listens and the token it wants. `token` is `None` for a
/// daemon from before tokens existed (no `.token` file): its `Hello` is sent
/// without one.
#[derive(Debug, Clone)]
pub struct Endpoint {
    pub port: u16,
    pub token: Option<Token>,
}

impl Endpoint {
    pub fn new(port: u16, token: Option<Token>) -> Self {
        Self { port, token }
    }
}

#[derive(Serialize, Deserialize)]
struct HelloArgs {
    #[serde(default)]
    token: Option<String>,
    /// Which frontend this connection is (`TUI`, `CLI`, `MCP`): the daemon
    /// names it in the errors and log lines of this connection's requests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    client: Option<String>,
}

static CLIENT_NAME: std::sync::Mutex<Option<&'static str>> = std::sync::Mutex::new(None);

/// Names this process's frontend (`"TUI"`, `"CLI"`, `"MCP"`) for the
/// connections it opens from now on. The last call wins: `gitpoulpi` with no
/// subcommand starts as the CLI and becomes the TUI.
pub fn set_client_name(name: &'static str) {
    *CLIENT_NAME.lock().unwrap_or_else(|e| e.into_inner()) = Some(name);
}

fn client_name() -> Option<&'static str> {
    *CLIENT_NAME.lock().unwrap_or_else(|e| e.into_inner())
}

/// A client name as the daemon keeps it: a short word, nothing a client could
/// use to put markup or a second line in a log.
fn clean_client_name(name: &str) -> Option<String> {
    let name: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ' '))
        .take(16)
        .collect();
    let name = name.trim().to_string();
    (!name.is_empty()).then_some(name)
}

/// The first request of a connection.
pub fn hello_request(token: Option<&Token>) -> Request {
    let args = HelloArgs {
        token: token.map(Token::to_hex),
        client: client_name().map(str::to_string),
    };
    Request {
        id: HANDSHAKE_REQUEST_ID,
        session: String::new(),
        repo: None,
        command: OperationKind::Hello,
        args: serde_json::to_value(args).unwrap_or(serde_json::Value::Null),
    }
}

fn refusal(request_id: u64) -> ServerMessage {
    ServerMessage::Response(Response {
        request_id,
        result: Err(ResponseError {
            message: AUTH_REQUIRED_MESSAGE.to_string(),
            unsupported: false,
            kind: None,
            detail: None,
            context: None,
            auth_required: true,
        }),
    })
}

/// [`authenticate_within`] with [`AUTH_TIMEOUT`].
pub async fn authenticate<S>(stream: &mut S, token: &Token, info: &DaemonInfo) -> bool
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    authenticate_within(stream, token, info, AUTH_TIMEOUT).await
}

/// The daemon's check of a new connection: the first frame must arrive within
/// `patience`, be a `Hello`, and carry the token. Then it is answered with the
/// daemon's `DaemonInfo` and `true` is returned; otherwise the refusal is sent
/// (when there is a request to answer) and `false` is returned: close the
/// connection.
pub async fn authenticate_within<S>(
    stream: &mut S,
    token: &Token,
    info: &DaemonInfo,
    patience: Duration,
) -> bool
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    authenticate_named_within(stream, token, info, patience)
        .await
        .is_some()
}

/// [`authenticate`] that also returns the frontend name the client gave in its
/// `Hello` (`Some(None)`: let in, but anonymous; `None`: refused).
pub async fn authenticate_named<S>(
    stream: &mut S,
    token: &Token,
    info: &DaemonInfo,
) -> Option<Option<String>>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    authenticate_named_within(stream, token, info, AUTH_TIMEOUT).await
}

async fn authenticate_named_within<S>(
    stream: &mut S,
    token: &Token,
    info: &DaemonInfo,
    patience: Duration,
) -> Option<Option<String>>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let first = tokio::time::timeout(patience, read_message::<_, Request>(stream)).await;
    let request = match first {
        Ok(Ok(Some(request))) => request,
        // Closed, malformed or too slow: nothing worth answering.
        Ok(Ok(None)) | Err(_) => return None,
        Ok(Err(_)) => {
            let _ = write_message(stream, &refusal(0)).await;
            return None;
        }
    };
    let hello = (request.command == OperationKind::Hello)
        .then(|| serde_json::from_value::<HelloArgs>(request.args.clone()).ok())
        .flatten();
    let client = hello
        .as_ref()
        .and_then(|args| args.client.as_deref())
        .and_then(clean_client_name);
    let presented = hello
        .and_then(|args| args.token)
        .and_then(|hex| Token::from_hex(&hex));
    let accepted = presented.is_some_and(|presented| token.matches(&presented));
    if !accepted {
        let _ = write_message(stream, &refusal(request.id)).await;
        return None;
    }
    let reply = ServerMessage::Response(Response {
        request_id: request.id,
        result: Ok(serde_json::to_value(info).unwrap_or(serde_json::Value::Null)),
    });
    write_message(stream, &reply).await.ok()?;
    Some(client)
}

/// The client's side: sends `Hello` (with the token when there is one) and
/// waits for the answer. A daemon that wants a token and got none, or the
/// wrong one, says so (`auth_required`): that is [`TransportError::Refused`].
pub async fn handshake<S>(stream: &mut S, token: Option<&Token>) -> Result<(), TransportError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    write_message(stream, &hello_request(token)).await?;
    loop {
        match read_message::<_, ServerMessage>(stream).await? {
            Some(ServerMessage::Response(resp)) if resp.request_id == HANDSHAKE_REQUEST_ID => {
                return match resp.result {
                    Err(error) if error.auth_required => {
                        Err(TransportError::Refused(error.message))
                    }
                    // Any other answer: the daemon let the connection in
                    // (a `Hello` that is not served says nothing about auth).
                    _ => Ok(()),
                };
            }
            // A live event of a daemon that does not gate on the token.
            Some(_) => continue,
            None => {
                return Err(TransportError::ConnectFailed(
                    "connection closed by daemon".into(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::DaemonKind;

    fn info() -> DaemonInfo {
        DaemonInfo {
            session: "s".into(),
            pid: 1,
            kind: DaemonKind::Embedded,
            started_at: 0,
            backend: "git2".into(),
            version: "0".into(),
            git_sha: None,
        }
    }

    fn other_token() -> Token {
        let mut bytes = [0u8; 32];
        bytes[31] = 1;
        Token(bytes)
    }

    async fn reply(client: &mut tokio::io::DuplexStream) -> Response {
        match read_message::<_, ServerMessage>(client).await.unwrap() {
            Some(ServerMessage::Response(r)) => r,
            other => panic!("unexpected {other:?}"),
        }
    }

    #[tokio::test]
    async fn the_right_token_is_answered_with_the_daemon_info() {
        let token = Token::generate().unwrap();
        let (mut daemon_end, mut client) = tokio::io::duplex(4096);
        let info = info();
        let daemon = authenticate(&mut daemon_end, &token, &info);
        let (ok, shaken) = tokio::join!(daemon, handshake(&mut client, Some(&token)));
        assert!(ok);
        shaken.unwrap();
    }

    #[tokio::test]
    async fn the_frontend_a_hello_names_is_returned_cleaned() {
        let token = Token::generate().unwrap();
        let (mut daemon_end, mut client) = tokio::io::duplex(4096);
        let mut hello = hello_request(Some(&token));
        hello.args["client"] = serde_json::json!("CLI\n[op #9]");
        write_message(&mut client, &hello).await.unwrap();
        let named = authenticate_named(&mut daemon_end, &token, &info()).await;
        assert_eq!(named, Some(Some("CLIop 9".to_string())));
        // An anonymous client is let in all the same.
        let (mut daemon_end, mut client) = tokio::io::duplex(4096);
        write_message(&mut client, &hello_request(Some(&token)))
            .await
            .unwrap();
        let named = authenticate_named(&mut daemon_end, &token, &info()).await;
        assert_eq!(named, Some(None));
    }

    #[tokio::test]
    async fn a_wrong_token_is_refused_with_the_auth_required_flag() {
        let token = Token::generate().unwrap();
        let (mut daemon_end, mut client) = tokio::io::duplex(4096);
        write_message(&mut client, &hello_request(Some(&other_token())))
            .await
            .unwrap();
        assert!(!authenticate(&mut daemon_end, &token, &info()).await);
        let resp = reply(&mut client).await;
        let error = resp.result.unwrap_err();
        assert!(error.auth_required);
        assert_eq!(error.message, AUTH_REQUIRED_MESSAGE);
    }

    #[tokio::test]
    async fn a_hello_without_a_token_is_refused() {
        let token = Token::generate().unwrap();
        let (mut daemon_end, mut client) = tokio::io::duplex(4096);
        write_message(&mut client, &hello_request(None))
            .await
            .unwrap();
        assert!(!authenticate(&mut daemon_end, &token, &info()).await);
        assert!(reply(&mut client).await.result.unwrap_err().auth_required);
    }

    #[tokio::test]
    async fn a_first_request_that_is_not_hello_is_refused_even_with_the_token() {
        let token = Token::generate().unwrap();
        let (mut daemon_end, mut client) = tokio::io::duplex(4096);
        let mut request = hello_request(Some(&token));
        request.command = OperationKind::Shutdown;
        request.id = 7;
        write_message(&mut client, &request).await.unwrap();
        assert!(!authenticate(&mut daemon_end, &token, &info()).await);
        let resp = reply(&mut client).await;
        assert_eq!(resp.request_id, 7);
        assert!(resp.result.unwrap_err().auth_required);
    }

    #[tokio::test]
    async fn an_old_clients_plain_hello_gets_the_message_it_can_print() {
        let token = Token::generate().unwrap();
        let (mut daemon_end, mut client) = tokio::io::duplex(4096);
        let mut request = hello_request(None);
        request.args = serde_json::Value::Null;
        write_message(&mut client, &request).await.unwrap();
        assert!(!authenticate(&mut daemon_end, &token, &info()).await);
        let error = reply(&mut client).await.result.unwrap_err();
        assert_eq!(error.message, AUTH_REQUIRED_MESSAGE);
        assert!(!error.unsupported);
    }

    #[tokio::test]
    async fn a_silent_connection_is_dropped_after_the_timeout() {
        let token = Token::generate().unwrap();
        let (mut daemon_end, _client) = tokio::io::duplex(4096);
        let started = std::time::Instant::now();
        let ok =
            authenticate_within(&mut daemon_end, &token, &info(), Duration::from_millis(50)).await;
        assert!(!ok);
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[tokio::test]
    async fn a_client_reads_a_refusal_as_refused() {
        let token = Token::generate().unwrap();
        let (mut daemon_end, mut client) = tokio::io::duplex(4096);
        let (info, wrong) = (info(), other_token());
        let daemon = authenticate(&mut daemon_end, &token, &info);
        let (ok, shaken) = tokio::join!(daemon, handshake(&mut client, Some(&wrong)));
        assert!(!ok);
        assert!(matches!(shaken, Err(TransportError::Refused(m)) if m == AUTH_REQUIRED_MESSAGE));
    }

    #[test]
    fn a_token_round_trips_through_hex_and_rejects_bad_text() {
        let token = Token::generate().unwrap();
        let hex = token.to_hex();
        assert_eq!(hex.len(), 64);
        assert!(Token::from_hex(&hex).unwrap().matches(&token));
        assert!(Token::from_hex(&hex[..62]).is_none());
        assert!(Token::from_hex(&"zz".repeat(32)).is_none());
        assert!(!token.matches(&other_token()));
    }

    #[test]
    fn debug_never_prints_the_token() {
        let token = Token::generate().unwrap();
        assert!(!format!("{token:?}").contains(&token.to_hex()));
    }
}
