//! A loopback HTTP server that asks for credentials and records what it is
//! given, so a test sees exactly which identity a git process presented
//! without a smart-HTTP implementation.
//!
//! It answers `401` plus `WWW-Authenticate: Basic` to any request that
//! carries no `Authorization: Basic` header (git then asks its credential
//! helpers and retries), and `404` to one that does. The transfer therefore
//! always fails at the end, which is fine: the *recorded header* is the
//! assertion. [`AuthHttpRemote::start_accepting`] instead answers `200` with the
//! advertisement of an empty repository, so a transfer succeeds and git
//! reports the credential as accepted (`credential approve`). Every `Authorization` value received is kept, in order, whatever
//! its scheme, so a test can also prove that a header it did not want sent
//! (a repository's `http.extraHeader`) never arrived.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// See the module docs.
pub struct AuthHttpRemote {
    port: u16,
    seen: Arc<Mutex<Vec<String>>>,
    requests: Arc<Mutex<usize>>,
    stop: Arc<AtomicBool>,
}

impl AuthHttpRemote {
    pub fn start() -> Self {
        Self::start_with(false)
    }

    /// Like [`Self::start`], but a request with a `Basic` header gets the
    /// advertisement of an empty repository: `ls-remote` and `fetch` succeed.
    pub fn start_accepting() -> Self {
        Self::start_with(true)
    }

    fn start_with(accepting: bool) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind the auth remote");
        let port = listener.local_addr().unwrap().port();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let requests = Arc::new(Mutex::new(0usize));
        let stop = Arc::new(AtomicBool::new(false));
        {
            let (seen, requests, stop) = (seen.clone(), requests.clone(), stop.clone());
            std::thread::spawn(move || {
                for stream in listener.incoming() {
                    if stop.load(Ordering::SeqCst) {
                        return;
                    }
                    let Ok(stream) = stream else { continue };
                    let (seen, requests) = (seen.clone(), requests.clone());
                    std::thread::spawn(move || serve(stream, &seen, &requests, accepting));
                }
            });
        }
        Self {
            port,
            seen,
            requests,
            stop,
        }
    }

    /// `host:port`, the key of a URL-scoped config entry.
    pub fn host(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

    /// A repository URL on this server.
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}/r.git", self.port)
    }

    /// Requests answered so far.
    pub fn requests(&self) -> usize {
        *self.requests.lock().unwrap()
    }

    /// Every `Authorization` header value received, verbatim, in order.
    pub fn authorizations(&self) -> Vec<String> {
        self.seen.lock().unwrap().clone()
    }

    /// The `user:password` of every `Basic` header received.
    pub fn basic_credentials(&self) -> Vec<String> {
        self.authorizations()
            .iter()
            .filter_map(|v| v.strip_prefix("Basic "))
            .filter_map(gitpoulpi_core::domain::b64::decode)
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .collect()
    }
}

impl Drop for AuthHttpRemote {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // Unblock `accept`.
        let _ = TcpStream::connect(("127.0.0.1", self.port));
    }
}

fn serve(
    mut stream: TcpStream,
    seen: &Mutex<Vec<String>>,
    requests: &Mutex<usize>,
    accepting: bool,
) {
    let mut head = Vec::new();
    let mut buf = [0u8; 2048];
    while !head.windows(4).any(|w| w == b"\r\n\r\n") {
        match stream.read(&mut buf) {
            Ok(0) | Err(_) => return,
            Ok(n) => head.extend_from_slice(&buf[..n]),
        }
        if head.len() > 64 * 1024 {
            return;
        }
    }
    let text = String::from_utf8_lossy(&head);
    let authorization = text.lines().find_map(|l| {
        let (name, value) = l.split_once(':')?;
        name.eq_ignore_ascii_case("authorization")
            .then(|| value.trim().to_string())
    });
    *requests.lock().unwrap() += 1;
    let basic = authorization
        .as_deref()
        .is_some_and(|v| v.starts_with("Basic "));
    if let Some(value) = authorization {
        seen.lock().unwrap().push(value);
    }
    let answer: Vec<u8> = if basic && accepting {
        empty_repository_advertisement()
    } else if basic {
        b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec()
    } else {
        b"HTTP/1.1 401 Unauthorized\r\nWWW-Authenticate: Basic realm=\"gitpoulpi-test\"\r\n\
          Content-Length: 0\r\nConnection: close\r\n\r\n"
            .to_vec()
    };
    let _ = stream.write_all(&answer);
}

/// A `200` carrying the smart-HTTP advertisement of a repository with no refs.
fn empty_repository_advertisement() -> Vec<u8> {
    let pkt = |text: &str| format!("{:04x}{text}", text.len() + 4);
    let body = format!(
        "{}0000{}0000",
        pkt("# service=git-upload-pack\n"),
        pkt(&format!(
            "{} capabilities^{{}}\0side-band-64k\n",
            "0".repeat(40)
        ))
    );
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/x-git-upload-pack-advertisement\r\n\
         Cache-Control: no-cache\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}
