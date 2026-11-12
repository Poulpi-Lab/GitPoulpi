//! Which operation a failed action ran into.
//!
//! The daemon answers a failed request with the operation's context (label,
//! main arguments, repository, request id: `ResponseError::context`). The
//! error type the link hands up has no room for it, so `call_raw` leaves it in
//! a slot that belongs to the action being run ([`capture`]); the Activity Log
//! reads it when the action's failure is reported.

use std::future::Future;
use std::sync::{Arc, Mutex};

use gitpoulpi_core::OpContext;

tokio::task_local! {
    /// The context of the last failed request of the action in progress.
    static SLOT: Arc<Mutex<Option<OpContext>>>;
}

/// A request of the action in progress failed with `context`.
pub(super) fn note(context: &OpContext) {
    let _ = SLOT.try_with(|slot| {
        *slot.lock().unwrap_or_else(|e| e.into_inner()) = Some(context.clone());
    });
}

/// Runs `work`; returns what it returned and the context of the last request
/// in it that failed, if the daemon gave one.
pub(super) async fn capture<F: Future>(work: F) -> (F::Output, Option<OpContext>) {
    let slot = Arc::new(Mutex::new(None));
    let out = SLOT.scope(Arc::clone(&slot), work).await;
    let context = slot.lock().unwrap_or_else(|e| e.into_inner()).take();
    (out, context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_last_failed_request_of_the_scope_is_returned() {
        let first = OpContext::named("Fetch", "remote=origin");
        let second = OpContext::named("Remove remote", "remote=cm6");
        let (value, context) = capture(async {
            note(&first);
            note(&second);
            7
        })
        .await;
        assert_eq!(value, 7);
        assert_eq!(context, Some(second));
    }

    #[tokio::test]
    async fn outside_a_scope_a_note_is_dropped() {
        note(&OpContext::named("Fetch", ""));
        let (_, context) = capture(async {}).await;
        assert_eq!(context, None);
    }
}
