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

pub(crate) fn maybe_panic(command: OperationKind) {
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
