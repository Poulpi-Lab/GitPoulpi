use tokio::net::TcpStream;

use crate::auth::Endpoint;
use crate::protocol::{Request, ServerMessage};
use crate::transport::{read_message, write_message, Result, TransportError};

/// The longest a request may go unanswered. Generous on purpose: a clone
/// or a push over a slow link is legitimately slow; this only bounds a
/// request that will never be answered.
pub const CALL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15 * 60);

/// A connected IPC client: one TCP link to its session's daemon (the
/// daemon multiplexes many clients; each needs only one).
pub struct Client {
    stream: TcpStream,
}

impl Client {
    /// Connects to a daemon and authenticates (see [`crate::auth`]). Get the
    /// endpoint from [`crate::discovery::endpoint`], or from a daemon this
    /// process embeds.
    pub async fn connect(endpoint: &Endpoint) -> Result<Self> {
        let port = endpoint.port;
        let mut stream = TcpStream::connect(("127.0.0.1", port))
            .await
            .map_err(|_| TransportError::ConnectFailed(format!("127.0.0.1:{port}")))?;
        // Request/response ping-pong of small frames: Nagle plus delayed
        // ACK would hold each one back ~40 ms.
        let _ = stream.set_nodelay(true);
        crate::auth::handshake(&mut stream, endpoint.token.as_ref()).await?;
        Ok(Self { stream })
    }

    /// Sends `request` and waits for the daemon's matching `Response`,
    /// discarding any `Event`s that arrive first (a caller that also wants
    /// live events should use `split()` instead and read the stream itself).
    ///
    /// Gives up after [`CALL_TIMEOUT`]: a response the daemon dropped must
    /// not leave the caller waiting forever.
    pub async fn call(&mut self, request: &Request) -> Result<crate::protocol::Response> {
        match tokio::time::timeout(CALL_TIMEOUT, self.call_untimed(request)).await {
            Ok(result) => result,
            Err(_) => Err(TransportError::ConnectFailed(format!(
                "the daemon did not answer within {} minutes",
                CALL_TIMEOUT.as_secs() / 60
            ))),
        }
    }

    async fn call_untimed(&mut self, request: &Request) -> Result<crate::protocol::Response> {
        write_message(&mut self.stream, request).await?;
        loop {
            match read_message::<_, ServerMessage>(&mut self.stream).await? {
                Some(ServerMessage::Response(resp)) if resp.request_id == request.id => {
                    return Ok(resp)
                }
                Some(_) => continue,
                None => {
                    return Err(TransportError::ConnectFailed(
                        "connection closed by daemon".into(),
                    ))
                }
            }
        }
    }

    /// Splits into a raw request-writer and message-reader pair for
    /// callers (the TUI) that need to interleave sending requests with
    /// continuously reading both responses and pushed `Event`s.
    pub fn into_stream(self) -> TcpStream {
        self.stream
    }
}
