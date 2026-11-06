//! Local transport for the client<->daemon IPC link.
//!
//! The transport is **loopback TCP plus a discovery file**: the daemon
//! binds `127.0.0.1:0` (an OS-assigned ephemeral port) and writes that port
//! to `<session>.port` under the config dir's `sessions/` directory (see
//! `gitpoulpi_config::ConfigPaths::ipc_port_file`); clients read the file
//! to find it. It is localhost-only and needs no platform-specific
//! dependency. Loopback alone is open to every account on the machine, so
//! the connection is token-authenticated: the daemon publishes a random
//! token in a user-only `<session>.token` file (written before the port
//! file) and refuses a connection whose first request is not a `Hello`
//! carrying it (see [`crate::auth`]). A named pipe or Unix socket would give
//! OS-level access control instead.
//!
//! Every message in both directions is framed as a 4-byte big-endian
//! length prefix followed by that many bytes of JSON.

use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("failed to connect to ipc endpoint {0}: is the daemon running?")]
    ConnectFailed(String),
    /// The daemon answered the handshake with a refusal (the message is its
    /// own wording).
    #[error("{0}")]
    Refused(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("malformed message: {0}")]
    Codec(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, TransportError>;

/// Largest frame either side accepts (512 MiB). A full-repo diff or log is
/// tens of MB at most; the cap only stops a corrupt or hostile length prefix
/// from asking for a 4 GiB buffer.
pub const MAX_FRAME_LEN: usize = 512 * 1024 * 1024;

/// Writes one length-prefixed JSON message to `writer`.
pub async fn write_message<W, T>(writer: &mut W, message: &T) -> Result<()>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    // Serialized straight after a reserved prefix: one buffer, no copy.
    let mut frame = vec![0u8; 4];
    serde_json::to_writer(&mut frame, message)?;
    send_frame(writer, frame).await
}

/// Writes one frame whose JSON payload is the concatenation of `parts`
/// (already-serialized fragments, e.g. a cached result between a fixed head
/// and tail), as a single write.
pub async fn write_raw_frame<W>(writer: &mut W, parts: &[&[u8]]) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    let mut frame = Vec::with_capacity(4 + parts.iter().map(|p| p.len()).sum::<usize>());
    frame.extend_from_slice(&[0u8; 4]);
    for part in parts {
        frame.extend_from_slice(part);
    }
    send_frame(writer, frame).await
}

/// Fills in the length prefix of `frame` (4 reserved bytes then payload)
/// and sends it in one write, not two: a 4-byte prefix followed by a
/// separate payload write meets Nagle + delayed ACK on loopback and adds
/// ~40 ms to every request and response.
async fn send_frame<W: AsyncWrite + Unpin>(writer: &mut W, mut frame: Vec<u8>) -> Result<()> {
    let len = frame.len() - 4;
    if len > MAX_FRAME_LEN {
        return Err(frame_too_large(len));
    }
    frame[..4].copy_from_slice(&(len as u32).to_be_bytes());
    writer.write_all(&frame).await?;
    writer.flush().await?;
    Ok(())
}

fn frame_too_large(len: usize) -> TransportError {
    TransportError::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!("ipc frame of {len} bytes exceeds the {MAX_FRAME_LEN} byte limit"),
    ))
}

/// Reads one length-prefixed JSON message from `reader`, or `Ok(None)` if
/// the peer closed the connection cleanly before sending a length prefix.
pub async fn read_message<R, T>(reader: &mut R) -> Result<Option<T>>
where
    R: AsyncRead + Unpin,
    T: DeserializeOwned,
{
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    }
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_FRAME_LEN {
        return Err(frame_too_large(len));
    }
    // Grown as bytes arrive rather than trusting the prefix up front.
    let mut payload = Vec::with_capacity(len.min(1 << 20));
    let got = reader.take(len as u64).read_to_end(&mut payload).await?;
    if got < len {
        return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into());
    }
    Ok(Some(serde_json::from_slice(&payload)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_message_round_trips() {
        let (mut a, mut b) = tokio::io::duplex(1024);
        write_message(&mut a, &serde_json::json!({"k": [1, 2]}))
            .await
            .unwrap();
        let back: serde_json::Value = read_message(&mut b).await.unwrap().unwrap();
        assert_eq!(back, serde_json::json!({"k": [1, 2]}));
    }

    #[tokio::test]
    async fn a_raw_frame_reads_as_its_parts_joined() {
        let (mut a, mut b) = tokio::io::duplex(1024);
        write_raw_frame(&mut a, &[b"{\"a\":", b"[1,2]", b"}"])
            .await
            .unwrap();
        let back: serde_json::Value = read_message(&mut b).await.unwrap().unwrap();
        assert_eq!(back, serde_json::json!({"a": [1, 2]}));
    }

    #[tokio::test]
    async fn an_oversized_length_prefix_is_refused_without_allocating() {
        let (mut a, mut b) = tokio::io::duplex(1024);
        a.write_all(&u32::MAX.to_be_bytes()).await.unwrap();
        let err = read_message::<_, serde_json::Value>(&mut b)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("exceeds"), "{err}");
    }

    #[tokio::test]
    async fn a_truncated_payload_is_an_error_not_a_hang() {
        let (mut a, mut b) = tokio::io::duplex(1024);
        a.write_all(&10u32.to_be_bytes()).await.unwrap();
        a.write_all(b"{}").await.unwrap();
        drop(a);
        assert!(read_message::<_, serde_json::Value>(&mut b).await.is_err());
    }
}
