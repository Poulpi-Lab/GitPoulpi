//! Which sequence run a request belongs to.
//!
//! The runner registers an origin for each step's ticket
//! ([`super::tickets::set_origin`]); `run_action` opens a scope with the
//! origin of the ticket it runs under, and `call_raw` adds it to the
//! arguments of every mutating request sent inside the scope (`origin`, read
//! by the daemon's journal). Untracked actions and reads carry nothing, so
//! their requests are the same bytes as before.

use std::future::Future;

use gitpoulpi_core::domain::{OperationKind, SequenceOrigin};

tokio::task_local! {
    static ORIGIN: Option<SequenceOrigin>;
}

/// Runs `work` with `origin` attached to its mutating requests.
pub(super) async fn scope<F: Future>(origin: Option<SequenceOrigin>, work: F) -> F::Output {
    ORIGIN.scope(origin, work).await
}

/// `args` with the scope's origin added, when `command` changes something and
/// `args` is an object.
pub(super) fn attach(command: OperationKind, mut args: serde_json::Value) -> serde_json::Value {
    if command.traits().read {
        return args;
    }
    let origin = ORIGIN.try_with(Clone::clone).ok().flatten();
    if let (Some(origin), Some(map)) = (origin, args.as_object_mut()) {
        if let Ok(value) = serde_json::to_value(origin) {
            map.insert("origin".to_string(), value);
        }
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn origin() -> SequenceOrigin {
        SequenceOrigin {
            name: "Backup".into(),
            run_id: 7,
            step: 2,
            of: 4,
        }
    }

    #[tokio::test]
    async fn a_mutation_in_a_scope_carries_the_origin() {
        let args = scope(Some(origin()), async {
            attach(OperationKind::Commit, json!({ "message": "m" }))
        })
        .await;
        assert_eq!(args["origin"]["name"], "Backup");
        assert_eq!(args["origin"]["run_id"], 7);
        assert_eq!(args["message"], "m");
    }

    #[tokio::test]
    async fn reads_and_unscoped_requests_are_untouched() {
        let read = scope(Some(origin()), async {
            attach(OperationKind::Status, json!({}))
        })
        .await;
        assert_eq!(read, json!({}));
        let none = attach(OperationKind::Commit, json!({ "message": "m" }));
        assert_eq!(none, json!({ "message": "m" }));
        let untracked = scope(None, async { attach(OperationKind::Commit, json!({})) }).await;
        assert_eq!(untracked, json!({}));
    }
}
