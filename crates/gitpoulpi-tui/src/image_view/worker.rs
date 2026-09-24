//! A background thread whose queue holds one request: a newer
//! [`Worker::submit`] replaces the one still waiting (latest wins), while a
//! request already running is left to finish. The caller keeps showing the
//! last picture until the next result lands ([`Worker::try_recv`]).
//!
//! The job runs inside `catch_unwind`: a panicking job drops that request
//! and the thread carries on with the next one, so a hostile file can
//! never take the worker down. Also here: the tiny [`Lru`] the decoded
//! bitmaps are kept in (two per pane).

use std::collections::VecDeque;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

struct Slot<Req> {
    pending: Option<(u64, Req)>,
    closed: bool,
}

/// One finished request.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Done<Res> {
    /// The id [`Worker::submit`] returned for the request.
    pub id: u64,
    pub value: Res,
}

pub(crate) struct Worker<Req, Res> {
    shared: Arc<(Mutex<Slot<Req>>, Condvar)>,
    results: Receiver<Done<Res>>,
    next_id: u64,
    handle: Option<JoinHandle<()>>,
}

impl<Req: Send + 'static, Res: Send + 'static> Worker<Req, Res> {
    /// Starts the thread. `wake` runs after every finished request (the
    /// event loop's redraw nudge); it must be cheap and never block.
    pub(crate) fn spawn(
        name: &str,
        job: impl Fn(Req) -> Res + Send + 'static,
        wake: impl Fn() + Send + 'static,
    ) -> Worker<Req, Res> {
        let shared = Arc::new((
            Mutex::new(Slot {
                pending: None,
                closed: false,
            }),
            Condvar::new(),
        ));
        let (tx, results) = channel();
        let theirs = Arc::clone(&shared);
        let handle = std::thread::Builder::new()
            .name(name.to_string())
            .spawn(move || loop {
                let next = {
                    let (lock, cv) = &*theirs;
                    let mut slot = lock.lock().unwrap_or_else(|e| e.into_inner());
                    loop {
                        if slot.closed {
                            return;
                        }
                        if let Some(r) = slot.pending.take() {
                            break r;
                        }
                        slot = cv.wait(slot).unwrap_or_else(|e| e.into_inner());
                    }
                };
                let (id, req) = next;
                if let Ok(value) = catch_unwind(AssertUnwindSafe(|| job(req))) {
                    if tx.send(Done { id, value }).is_err() {
                        return;
                    }
                    wake();
                }
            })
            .ok();
        Worker {
            shared,
            results,
            next_id: 0,
            handle,
        }
    }

    /// Queues `req`, replacing a request that has not started yet.
    /// Returns its id (ids only grow).
    pub(crate) fn submit(&mut self, req: Req) -> u64 {
        self.next_id += 1;
        let id = self.next_id;
        let (lock, cv) = &*self.shared;
        lock.lock().unwrap_or_else(|e| e.into_inner()).pending = Some((id, req));
        cv.notify_one();
        id
    }

    /// The id of the most recent [`Worker::submit`], `0` before any.
    pub(crate) fn latest_id(&self) -> u64 {
        self.next_id
    }

    /// The newest finished result, if any; older ones queued behind it are
    /// dropped (the screen only ever needs the latest picture).
    pub(crate) fn try_recv(&self) -> Option<Done<Res>> {
        let mut last = None;
        while let Ok(d) = self.results.try_recv() {
            last = Some(d);
        }
        last
    }
}

impl<Req, Res> Drop for Worker<Req, Res> {
    fn drop(&mut self) {
        let (lock, cv) = &*self.shared;
        {
            let mut s = lock.lock().unwrap_or_else(|e| e.into_inner());
            s.closed = true;
            s.pending = None;
        }
        cv.notify_all();
        // Not joined: a running decode of a huge file would stall the UI
        // thread on close. The thread exits as soon as the job returns.
        self.handle.take();
    }
}

/// A least-recently-used map of a few entries (linear scan; capacity is 2).
pub(crate) struct Lru<K, V> {
    cap: usize,
    items: VecDeque<(K, V)>,
}

impl<K: PartialEq, V> Lru<K, V> {
    pub(crate) fn new(cap: usize) -> Self {
        Lru {
            cap: cap.max(1),
            items: VecDeque::new(),
        }
    }

    /// Looks `k` up and marks it most recently used.
    pub(crate) fn get(&mut self, k: &K) -> Option<&V> {
        let i = self.items.iter().position(|(key, _)| key == k)?;
        let item = self.items.remove(i)?;
        self.items.push_front(item);
        self.items.front().map(|(_, v)| v)
    }

    /// Inserts (replacing an equal key) and evicts the least recently used.
    pub(crate) fn put(&mut self, k: K, v: V) {
        self.items.retain(|(key, _)| *key != k);
        self.items.push_front((k, v));
        self.items.truncate(self.cap);
    }

    pub(crate) fn clear(&mut self) {
        self.items.clear();
    }

    pub(crate) fn len(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::time::{Duration, Instant};

    fn wait_for<T>(mut f: impl FnMut() -> Option<T>) -> T {
        let end = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(v) = f() {
                return v;
            }
            assert!(Instant::now() < end, "timed out");
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    /// A job that blocks until the test sends it a token, and reports when
    /// it has started.
    fn gated() -> (Worker<u32, u32>, Sender<()>, Receiver<u32>) {
        let (gate_tx, gate_rx) = channel::<()>();
        let (started_tx, started_rx) = channel::<u32>();
        let gate_rx = Mutex::new(gate_rx);
        let w = Worker::spawn(
            "test-worker",
            move |n| {
                started_tx.send(n).unwrap();
                gate_rx.lock().unwrap().recv().unwrap();
                n * 10
            },
            || {},
        );
        (w, gate_tx, started_rx)
    }

    #[test]
    fn a_newer_request_replaces_the_queued_ones() {
        let (mut w, gate, started) = gated();
        let first = w.submit(1);
        assert_eq!(started.recv_timeout(Duration::from_secs(5)).unwrap(), 1);
        // 1 is running; 2, 3 and 4 arrive while it is busy.
        w.submit(2);
        w.submit(3);
        let last = w.submit(4);
        assert_eq!(w.latest_id(), last);
        gate.send(()).unwrap(); // finish 1
                                // Only 4 runs next: 2 and 3 were superseded.
        assert_eq!(started.recv_timeout(Duration::from_secs(5)).unwrap(), 4);
        gate.send(()).unwrap();
        let done = wait_for(|| {
            let d = w.try_recv()?;
            (d.id == last).then_some(d)
        });
        assert_eq!(done.value, 40);
        assert!(first < last);
        assert!(started.try_recv().is_err(), "2 and 3 never ran");
    }

    #[test]
    fn results_carry_their_request_id_and_wake_the_loop() {
        let woke = Arc::new(Mutex::new(0u32));
        let w2 = Arc::clone(&woke);
        let mut w = Worker::spawn(
            "test-wake",
            |n: u32| n + 1,
            move || *w2.lock().unwrap() += 1,
        );
        let id = w.submit(41);
        let d = wait_for(|| w.try_recv());
        assert_eq!((d.id, d.value), (id, 42));
        wait_for(|| (*woke.lock().unwrap() == 1).then_some(()));
    }

    #[test]
    fn a_panicking_job_does_not_kill_the_worker() {
        let mut w = Worker::spawn(
            "test-panic",
            |n: u32| {
                assert!(n != 0, "boom");
                n
            },
            || {},
        );
        w.submit(0);
        let ok = w.submit(7);
        let d = wait_for(|| w.try_recv());
        assert_eq!((d.id, d.value), (ok, 7));
    }

    #[test]
    fn the_lru_keeps_the_most_recently_used() {
        let mut c = Lru::new(2);
        c.put("a", 1);
        c.put("b", 2);
        assert_eq!(c.get(&"a"), Some(&1)); // a is now the freshest
        c.put("c", 3); // evicts b
        assert_eq!(c.get(&"b"), None);
        assert_eq!(c.get(&"a"), Some(&1));
        assert_eq!(c.get(&"c"), Some(&3));
        c.put("c", 30);
        assert_eq!((c.len(), c.get(&"c")), (2, Some(&30)));
        c.clear();
        assert_eq!(c.len(), 0);
    }
}
