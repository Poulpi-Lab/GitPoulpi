//! Tells the daemon about the TUI's own writes to an issue or a request
//! (post, reply, edit, delete, state, review, create, edit), so another
//! window or session showing the same item reads it again. The CLI and the
//! MCP server do the same from their own processes.
//!
//! The daemon echoes the event to every client, the writer included; each
//! notification carries this process's `origin`, so the window that wrote
//! drops the echo (it already shows what it wrote) instead of reading twice.
//!
//! The write workers run on threads of their own with no handle on the
//! link, so the link's requester is kept here, once, like the jobs' one.

#![cfg_attr(not(feature = "hosting"), allow(dead_code))]

use std::sync::{Mutex, OnceLock};

use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_ipc::{HostingChange, HostingChangeArgs, HostingItemKind};

use crate::daemon_link::Requester;

static REQUESTER: Mutex<Option<Requester>> = Mutex::new(None);

/// The link's requester, set at connect (and by tests).
pub(crate) fn set_requester(requester: Requester) {
    *REQUESTER.lock().unwrap_or_else(|e| e.into_inner()) = Some(requester);
}

/// What this process calls itself in a notification.
pub(crate) fn origin() -> &'static str {
    static ORIGIN: OnceLock<String> = OnceLock::new();
    ORIGIN.get_or_init(|| {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        format!("tui-{}-{nanos:x}", std::process::id())
    })
}

/// Whether `change` is this process's own notification coming back.
pub(crate) fn is_own(change: &HostingChangeArgs) -> bool {
    change.origin == origin()
}

/// Tells the daemon that an item of `project` changed. Best effort and
/// never blocking: the write already succeeded, and with no link (a test, a
/// window that has none) nobody is told.
pub(crate) fn notify(
    integration_id: &str,
    project: &str,
    kind: HostingItemKind,
    number: u64,
    id: &str,
    change: HostingChange,
) {
    let args = HostingChangeArgs {
        integration_id: integration_id.to_string(),
        project: project.to_string(),
        kind,
        number,
        id: id.to_string(),
        change,
        origin: origin().to_string(),
    };
    #[cfg(test)]
    testing::SENT
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push(args.clone());
    let requester = REQUESTER.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let Some(requester) = requester else { return };
    if let Ok(args) = serde_json::to_value(args) {
        // The answer (null) is not needed.
        let _ = requester.call::<serde_json::Value>(OperationKind::NotifyHostingChange, args);
    }
}

/// [`notify`] for an item named by its conversation key.
#[cfg(feature = "hosting")]
pub(crate) fn notify_item(
    integration_id: &str,
    project: &str,
    key: &gitpoulpi_hosting::conversation::ItemKey,
    change: HostingChange,
) {
    use gitpoulpi_hosting::conversation::ItemKind;
    let kind = match key.kind {
        ItemKind::Issue => HostingItemKind::Issue,
        ItemKind::PullRequest => HostingItemKind::PullRequest,
    };
    notify(integration_id, project, kind, key.number, &key.id, change);
}

#[cfg(test)]
pub(crate) mod testing {
    use super::*;

    /// Every notification made in this test process (the workers run on
    /// threads of their own, so nothing thread-local would see them).
    pub(super) static SENT: Mutex<Vec<HostingChangeArgs>> = Mutex::new(Vec::new());

    /// The notifications made for `project`: tests filter on a project of
    /// their own, since the log is shared.
    pub(crate) fn sent(project: &str) -> Vec<HostingChangeArgs> {
        SENT.lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .filter(|a| a.project == project)
            .cloned()
            .collect()
    }
}
