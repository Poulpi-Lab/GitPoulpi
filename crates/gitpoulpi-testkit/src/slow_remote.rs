//! A remote that never answers, to cancel a transfer in the middle of it.
//!
//! A loopback TCP server speaking nothing: it accepts a connection and holds
//! it. `git fetch|push|clone git://127.0.0.1:<port>/x` connects and then waits
//! for the ref advertisement forever, so the transfer is "in flight" for as
//! long as the test wants, with no timing guess and no real network.
//!
//! The server also *observes*: it counts accepted connections and the ones
//! the peer closed, which is how a test proves the git process that held the
//! connection is dead (a killed process closes its sockets).
//!
//! Plain `git` hooks are not an option for a clone: nothing exists yet to
//! carry a config, and the contract suite touches no process-wide environment.

use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// See the module docs.
pub struct SlowRemote {
    port: u16,
    accepted: Arc<AtomicUsize>,
    closed: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
}

impl SlowRemote {
    pub fn start() -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind the slow remote");
        let port = listener.local_addr().unwrap().port();
        let accepted = Arc::new(AtomicUsize::new(0));
        let closed = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        {
            let (accepted, closed, stop) = (accepted.clone(), closed.clone(), stop.clone());
            std::thread::spawn(move || {
                for stream in listener.incoming() {
                    if stop.load(Ordering::SeqCst) {
                        return;
                    }
                    let Ok(mut stream) = stream else { continue };
                    accepted.fetch_add(1, Ordering::SeqCst);
                    let (closed, stop) = (closed.clone(), stop.clone());
                    std::thread::spawn(move || {
                        // Hold the connection: read (and ignore) until the peer
                        // goes away.
                        let mut buf = [0u8; 4096];
                        loop {
                            match stream.read(&mut buf) {
                                Ok(0) | Err(_) => break,
                                Ok(_) => {}
                            }
                            if stop.load(Ordering::SeqCst) {
                                break;
                            }
                        }
                        closed.fetch_add(1, Ordering::SeqCst);
                    });
                }
            });
        }
        Self {
            port,
            accepted,
            closed,
            stop,
        }
    }

    /// A URL git accepts for this server (`git://`).
    pub fn url(&self) -> String {
        format!("git://127.0.0.1:{}/slow.git", self.port)
    }

    /// Connections accepted so far: a transfer is under way when it is >= 1.
    pub fn accepted(&self) -> usize {
        self.accepted.load(Ordering::SeqCst)
    }

    /// Connections the peer has closed (its process ended or gave up).
    pub fn closed(&self) -> usize {
        self.closed.load(Ordering::SeqCst)
    }

    /// Waits until `accepted() >= n`; false after `timeout`.
    pub fn wait_accepted(&self, n: usize, timeout: Duration) -> bool {
        wait_until(timeout, || self.accepted() >= n)
    }

    /// Waits until `closed() >= n`; false after `timeout`.
    pub fn wait_closed(&self, n: usize, timeout: Duration) -> bool {
        wait_until(timeout, || self.closed() >= n)
    }
}

impl Drop for SlowRemote {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // Wakes the accept loop so its thread ends.
        let _ = TcpStream::connect(("127.0.0.1", self.port));
    }
}

fn wait_until(timeout: Duration, mut done: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if done() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    done()
}
