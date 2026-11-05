//! A hook for tests only: make one operation panic, to prove a panicking
//! operation is answered with an error and does not take the daemon (or
//! the waiting client) down. Disarmed by default; costs one relaxed load.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use gitpoulpi_core::domain::OperationKind;

static ARMED: AtomicBool = AtomicBool::new(false);
static OPERATIONS: Mutex<Vec<OperationKind>> = Mutex::new(Vec::new());

/// Arms the hook for exactly `operations` (empty disarms it).
#[doc(hidden)]
pub fn panic_on_for_tests(operations: &[OperationKind]) {
    let mut list = OPERATIONS.lock().unwrap_or_else(|e| e.into_inner());
    *list = operations.to_vec();
    ARMED.store(!list.is_empty(), Ordering::Relaxed);
}

type Gate = std::sync::Arc<(Mutex<HoldState>, std::sync::Condvar)>;

/// A test-only gate: the armed operation stops inside its blocking dispatch
/// until `release`, so a test can shut the daemon down while it is in flight.
#[doc(hidden)]
pub struct Hold {
    state: Gate,
}

#[derive(Default)]
struct HoldState {
    reached: bool,
    released: bool,
}

static HOLD: Mutex<Option<(OperationKind, Gate)>> = Mutex::new(None);

impl Hold {
    /// Arms the gate for `operation` (process-global: one test per binary).
    pub fn arm(operation: OperationKind) -> Self {
        let state: Gate = std::sync::Arc::default();
        *HOLD.lock().unwrap_or_else(|e| e.into_inner()) = Some((operation, state.clone()));
        Self { state }
    }

    /// Blocks until the operation is inside the gate.
    pub fn wait_reached(&self) {
        let (lock, cv) = &*self.state;
        let mut s = lock.lock().unwrap_or_else(|e| e.into_inner());
        while !s.reached {
            s = cv.wait(s).unwrap_or_else(|e| e.into_inner());
        }
    }

    /// Lets the held operation go on, and disarms the gate.
    pub fn release(&self) {
        *HOLD.lock().unwrap_or_else(|e| e.into_inner()) = None;
        let (lock, cv) = &*self.state;
        lock.lock().unwrap_or_else(|e| e.into_inner()).released = true;
        cv.notify_all();
    }
}

fn maybe_hold(command: OperationKind) {
    let gate = HOLD
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .filter(|(op, _)| *op == command)
        .map(|(_, state)| state.clone());
    let Some(gate) = gate else { return };
    let (lock, cv) = &*gate;
    let mut s = lock.lock().unwrap_or_else(|e| e.into_inner());
    s.reached = true;
    cv.notify_all();
    while !s.released {
        s = cv.wait(s).unwrap_or_else(|e| e.into_inner());
    }
}

pub(crate) fn maybe_panic(command: OperationKind) {
    maybe_hold(command);
    if !ARMED.load(Ordering::Relaxed) {
        return;
    }
    let armed = OPERATIONS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .contains(&command);
    if armed {
        panic!("test hook: {command:?} panics on purpose");
    }
}

/// `GITPOULPI_TEST_TRANSFER_DELAY_MS`: every network job sleeps this long
/// *after* it has its lock and before it calls the backend, so a test can find
/// a job `Running` (or others queued behind it) without a real slow network.
/// The sleep is part of the job's work: canceling the job drops it. Unset in
/// production.
pub(crate) async fn transfer_delay() {
    let ms = std::env::var("GITPOULPI_TEST_TRANSFER_DELAY_MS")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok());
    if let Some(ms) = ms {
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    }
}

async fn delay_from(var: &str) {
    let ms = std::env::var(var)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok());
    if let Some(ms) = ms {
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    }
}

/// `GITPOULPI_TEST_PULL_FETCH_DELAY_MS`: a pull sleeps this long before each
/// of its downloads, inside the cancellable part of the job.
pub(crate) async fn pull_fetch_delay() {
    delay_from("GITPOULPI_TEST_PULL_FETCH_DELAY_MS").await;
}

/// `GITPOULPI_TEST_INTEGRATE_DELAY_MS`: a pull sleeps this long once it has
/// passed its point of no return, so a test can find it integrating.
pub(crate) async fn integrate_delay() {
    delay_from("GITPOULPI_TEST_INTEGRATE_DELAY_MS").await;
}

/// `GITPOULPI_TEST_STEP_DELAY_MS`: a workflow sleeps this long before each
/// step (where it looks for a cancel request).
pub(crate) async fn step_delay() {
    delay_from("GITPOULPI_TEST_STEP_DELAY_MS").await;
}

impl Drop for Hold {
    /// A test that fails while holding must not leave the dispatch thread
    /// parked (the runtime would wait for it forever).
    fn drop(&mut self) {
        self.release();
    }
}
