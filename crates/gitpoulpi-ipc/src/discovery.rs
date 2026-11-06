//! Finding a running daemon without ever waiting on a dead one.
//!
//! A daemon publishes `sessions/<session>.token` (the secret a client presents
//! first, written before the port file), `sessions/<session>.port` (and `.pid` when it is a
//! standalone process, `.host` when it lives inside a TUI or an MCP server).
//! A crash leaves the files behind, and on Windows connecting to a dead
//! loopback port takes about two seconds to be refused (instant on Linux), so
//! every consumer of those files goes through here:
//!
//! 1. a file whose recorded process is gone is stale: deleted, never
//!    connected to ([`live_port_with`]);
//! 2. a connect to loopback is bounded by [`CONNECT_TIMEOUT`], whatever the OS
//!    does with a refused port ([`connect`], [`connect_with`]).
//!
//! The probes and the pid check are injectable so the tests need neither a
//! socket nor a real dead process.

use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_config::secure_fs;
use tokio::net::TcpStream;

use crate::auth::{Endpoint, Token};
use crate::transport::TransportError;

/// How long a loopback connect gets. A live daemon accepts in well under a
/// millisecond; past this the port is as good as dead.
pub const CONNECT_TIMEOUT: Duration = Duration::from_millis(150);

/// How long the handshake after a connect may take.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// The endpoint `port_file` names: its port and, when a `.token` file sits
/// next to it, the token. A missing token file is a daemon from before tokens
/// (its `Hello` takes none). `None` for a missing or unreadable port file.
pub fn endpoint(port_file: &Path) -> Option<Endpoint> {
    let port: u16 = std::fs::read_to_string(port_file)
        .ok()?
        .trim()
        .parse()
        .ok()?;
    Some(Endpoint::new(port, read_token(port_file)))
}

/// The endpoint for `port`, with the token of `port_file` when that file still
/// names `port` (a newer daemon may have replaced it: then its token is not
/// this port's).
pub fn endpoint_for_port(port_file: &Path, port: u16) -> Endpoint {
    match endpoint(port_file) {
        Some(found) if found.port == port => found,
        _ => Endpoint::new(port, None),
    }
}

fn read_token(port_file: &Path) -> Option<Token> {
    Token::from_hex(&std::fs::read_to_string(sibling(port_file, "token")).ok()?)
}

/// `127.0.0.1:port`, giving up after [`CONNECT_TIMEOUT`], then the token
/// handshake. A refusal is `PermissionDenied` with the daemon's own message.
pub async fn connect(endpoint: &Endpoint) -> io::Result<TcpStream> {
    let mut stream = connect_with(endpoint.port, CONNECT_TIMEOUT, |p| {
        TcpStream::connect(("127.0.0.1", p))
    })
    .await?;
    let _ = stream.set_nodelay(true);
    match tokio::time::timeout(
        HANDSHAKE_TIMEOUT,
        crate::auth::handshake(&mut stream, endpoint.token.as_ref()),
    )
    .await
    {
        Ok(Ok(())) => Ok(stream),
        Ok(Err(TransportError::Refused(message))) => {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, message))
        }
        Ok(Err(TransportError::Io(e))) => Err(e),
        Ok(Err(e)) => Err(io::Error::other(e.to_string())),
        Err(_) => Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "the daemon did not answer the handshake",
        )),
    }
}

/// Publishes a daemon's endpoint: the token file first (user-only), then the
/// port file, whose appearance means "ready".
pub fn publish(port_file: &Path, endpoint: &Endpoint) -> io::Result<()> {
    if let Some(parent) = port_file.parent() {
        secure_fs::create_private_dir_all(parent)?;
    }
    if let Some(token) = &endpoint.token {
        secure_fs::write_private(&sibling(port_file, "token"), token.to_hex().as_bytes())?;
    }
    secure_fs::write_private(port_file, endpoint.port.to_string().as_bytes())
}

/// [`connect`] with the connect itself and the patience injectable.
pub async fn connect_with<S, F, Fut>(port: u16, patience: Duration, connect: F) -> io::Result<S>
where
    F: FnOnce(u16) -> Fut,
    Fut: Future<Output = io::Result<S>>,
{
    match tokio::time::timeout(patience, connect(port)).await {
        Ok(result) => result,
        Err(_) => Err(io::Error::new(
            io::ErrorKind::TimedOut,
            format!(
                "127.0.0.1:{port} did not answer within {} ms",
                patience.as_millis()
            ),
        )),
    }
}

/// Whether something accepts connections on `127.0.0.1:port` (bounded).
pub async fn is_listening(port: u16) -> bool {
    connect_with(port, CONNECT_TIMEOUT, |p| {
        TcpStream::connect(("127.0.0.1", p))
    })
    .await
    .is_ok()
}

/// The sibling file of `port_file` with another extension.
fn sibling(port_file: &Path, extension: &str) -> PathBuf {
    port_file.with_extension(extension)
}

/// The process that wrote `port_file`: a standalone daemon's `.pid`, else an
/// embedded daemon's `.host`. `None` when neither exists (a file from an older
/// build: nothing to judge by, so it is probed like before).
pub fn recorded_pid(port_file: &Path) -> Option<u32> {
    ["pid", "host"].iter().find_map(|ext| {
        std::fs::read_to_string(sibling(port_file, ext))
            .ok()?
            .trim()
            .parse()
            .ok()
    })
}

/// Records the calling process as the owner of `port_file` (the `.host` file
/// an embedded daemon writes; never `.pid`, whose existence means "a
/// standalone daemon that `daemon stop` may kill").
pub fn write_host_file(port_file: &Path) {
    let _ = secure_fs::write_private(
        &sibling(port_file, "host"),
        std::process::id().to_string().as_bytes(),
    );
}

/// Removes `port_file` and its siblings, but only while the port file still
/// holds `port`: a newer daemon may have replaced it since.
pub fn remove_if_ours(port_file: &Path, port: u16) {
    let ours = std::fs::read_to_string(port_file)
        .ok()
        .and_then(|raw| raw.trim().parse::<u16>().ok())
        == Some(port);
    if ours {
        remove_files(port_file);
    }
}

/// Removes `port_file` and every file published beside it (`.pid`, `.host`,
/// `.token`).
pub fn remove_files(port_file: &Path) {
    let _ = std::fs::remove_file(port_file);
    let _ = std::fs::remove_file(sibling(port_file, "pid"));
    let _ = std::fs::remove_file(sibling(port_file, "host"));
    let _ = std::fs::remove_file(sibling(port_file, "token"));
}

/// Whether the process that wrote `port_file` is gone. Deletes the files then.
/// A file with no recorded process is never stale by this test.
pub fn remove_if_stale(port_file: &Path, pid_alive: &(dyn Fn(u32) -> bool + Sync)) -> bool {
    match recorded_pid(port_file) {
        Some(pid) if !pid_alive(pid) => {
            remove_files(port_file);
            true
        }
        _ => false,
    }
}

/// The port `port_file` names if a daemon is really there: a stale file is
/// deleted and not connected to, anything else is probed within `patience`.
/// `None` for a missing, unreadable, stale or unanswering file.
pub async fn live_port_with<S, F, Fut>(
    port_file: &Path,
    patience: Duration,
    pid_alive: &(dyn Fn(u32) -> bool + Sync),
    connect: F,
) -> Option<u16>
where
    F: FnOnce(u16) -> Fut,
    Fut: Future<Output = io::Result<S>>,
{
    let port: u16 = std::fs::read_to_string(port_file)
        .ok()?
        .trim()
        .parse()
        .ok()?;
    if remove_if_stale(port_file, pid_alive) {
        return None;
    }
    connect_with(port, patience, connect)
        .await
        .ok()
        .map(|_| port)
}

/// [`live_port_with`] against the real loopback and the real process table.
pub async fn live_port(port_file: &Path) -> Option<u16> {
    live_port_with(port_file, CONNECT_TIMEOUT, &pid_alive, |p| {
        TcpStream::connect(("127.0.0.1", p))
    })
    .await
}

/// [`live_port`] plus the token of the daemon that answered.
pub async fn live_endpoint(port_file: &Path) -> Option<Endpoint> {
    let port = live_port(port_file).await?;
    Some(Endpoint::new(port, read_token(port_file)))
}

/// Is a process with this id running? A process we may not signal counts as
/// running (the answer errs towards probing the port).
#[cfg(unix)]
pub fn pid_alive(pid: u32) -> bool {
    let Ok(pid) = libc::pid_t::try_from(pid) else {
        return true;
    };
    if pid == 0 {
        return true;
    }
    // SAFETY: signal 0 only checks that the process exists.
    if unsafe { libc::kill(pid, 0) } == 0 {
        return true;
    }
    io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

#[cfg(windows)]
pub fn pid_alive(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, STILL_ACTIVE};
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    // SAFETY: plain handle calls; the handle is closed on every path.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            // Access denied still means it exists; anything else, it is gone.
            return io::Error::last_os_error().raw_os_error() == Some(5);
        }
        let mut code = 0u32;
        let ok = GetExitCodeProcess(handle, &mut code);
        CloseHandle(handle);
        ok == 0 || code == STILL_ACTIVE as u32
    }
}

#[cfg(not(any(unix, windows)))]
pub fn pid_alive(_pid: u32) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn files(pid_ext: Option<(&str, u32)>) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let port_file = dir.path().join("work.port");
        std::fs::write(&port_file, "40404").unwrap();
        if let Some((ext, pid)) = pid_ext {
            std::fs::write(dir.path().join(format!("work.{ext}")), pid.to_string()).unwrap();
        }
        (dir, port_file)
    }

    #[tokio::test]
    async fn a_port_file_whose_process_is_dead_is_deleted_without_connecting() {
        for ext in ["pid", "host"] {
            let (dir, port_file) = files(Some((ext, 4242)));
            let attempts = AtomicUsize::new(0);
            let found = live_port_with(&port_file, CONNECT_TIMEOUT, &|_| false, |_| {
                attempts.fetch_add(1, Ordering::SeqCst);
                async { Ok::<(), io::Error>(()) }
            })
            .await;
            assert_eq!(found, None);
            assert_eq!(
                attempts.load(Ordering::SeqCst),
                0,
                "no connect for a dead pid"
            );
            assert!(!port_file.exists(), "the stale .port goes");
            assert!(!dir.path().join(format!("work.{ext}")).exists());
        }
    }

    #[tokio::test]
    async fn a_port_file_whose_process_lives_is_probed_and_kept() {
        let (_dir, port_file) = files(Some(("host", 4242)));
        let found = live_port_with(&port_file, CONNECT_TIMEOUT, &|_| true, |_| async {
            Ok::<(), io::Error>(())
        })
        .await;
        assert_eq!(found, Some(40404));
        assert!(port_file.exists());
    }

    #[tokio::test]
    async fn a_port_file_with_no_recorded_process_is_still_probed() {
        let (_dir, port_file) = files(None);
        let attempts = AtomicUsize::new(0);
        let found = live_port_with(&port_file, CONNECT_TIMEOUT, &|_| false, |_| {
            attempts.fetch_add(1, Ordering::SeqCst);
            async { Err::<(), _>(io::Error::from(io::ErrorKind::ConnectionRefused)) }
        })
        .await;
        assert_eq!(found, None);
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_connect_that_never_finishes_gives_up_quickly() {
        let started = std::time::Instant::now();
        let result = connect_with(1, CONNECT_TIMEOUT, |_| {
            std::future::pending::<io::Result<()>>()
        })
        .await;
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::TimedOut);
        assert!(
            started.elapsed() < Duration::from_millis(200),
            "took {:?}",
            started.elapsed()
        );
    }

    #[test]
    fn only_our_own_port_file_is_removed() {
        let (dir, port_file) = files(Some(("host", 1)));
        remove_if_ours(&port_file, 40405);
        assert!(port_file.exists(), "a newer daemon's file stays");
        remove_if_ours(&port_file, 40404);
        assert!(!port_file.exists());
        assert!(!dir.path().join("work.host").exists());
    }

    #[test]
    fn this_process_is_alive_and_a_reaped_child_is_not() {
        assert!(pid_alive(std::process::id()));
        #[cfg(unix)]
        {
            let mut child = std::process::Command::new("true").spawn().unwrap();
            let pid = child.id();
            child.wait().unwrap();
            assert!(!pid_alive(pid));
        }
    }
}
