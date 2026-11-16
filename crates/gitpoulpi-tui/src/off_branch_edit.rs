//! The edit that waits for a checkout: "Check out <branch> and edit" on a
//! commit that is not on the checked-out branch (`guide_dialog`'s off-branch
//! dialog) sends the ordinary checkout, then the window opens the Edit commit
//! message / Amend popup once the refs show that branch checked out and the
//! Log holds it. A checkout that never lands (a dirty tree the daemon
//! refused, say) ends the wait after [`PATIENCE`] with a line in the status
//! bar; nothing else is left behind.

use std::cell::RefCell;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{CommitGraph, Oid, Ref, RefKind};

use crate::reword_ui::{AmendScope, RewordForm};

/// How long a checkout may take before the edit is given up.
const PATIENCE: Duration = Duration::from_secs(20);

struct Pending {
    commit: Oid,
    /// `None`: Edit commit message; `Some`: Amend with that scope.
    scope: Option<AmendScope>,
    branch: String,
    since: Instant,
}

thread_local! {
    static PENDING: RefCell<Option<Pending>> = const { RefCell::new(None) };
}

/// What the window does with the waiting edit this tick.
pub(crate) enum Step {
    Wait,
    /// The branch is checked out: open the popup.
    Open(Box<RewordForm>),
    /// The checkout did not land in time: say so.
    GiveUp(String),
}

/// An edit of `commit` waits for `branch` to be checked out.
pub(crate) fn wait_for(commit: Oid, scope: Option<AmendScope>, branch: String) {
    PENDING.with(|p| {
        *p.borrow_mut() = Some(Pending {
            commit,
            scope,
            branch,
            since: Instant::now(),
        });
    });
}

/// An edit is waiting for its checkout.
pub(crate) fn waiting() -> bool {
    PENDING.with(|p| p.borrow().is_some())
}

/// One step of the wait; `None` when nothing waits.
pub(crate) fn step(
    log: &gitpoulpi_core::Result<CommitGraph>,
    refs: &gitpoulpi_core::Result<Vec<Ref>>,
) -> Option<Step> {
    PENDING.with(|p| {
        let mut slot = p.borrow_mut();
        let pending = slot.as_ref()?;
        let on_branch = refs.as_deref().unwrap_or(&[]).iter().any(|r| {
            r.is_head && matches!(&r.kind, RefKind::LocalBranch { name } if *name == pending.branch)
        });
        if on_branch {
            let form = match pending.scope {
                None => crate::log_menu::reword_form_for(log, refs, pending.commit),
                Some(scope) => crate::log_menu::amend_form_for(log, refs, pending.commit, scope),
            };
            // The Log can still show the old branch for a frame: wait for it.
            if let Ok(form) = form {
                *slot = None;
                return Some(Step::Open(Box::new(form)));
            }
        }
        if pending.since.elapsed() > PATIENCE {
            let short = &pending.commit.to_hex()[..crate::log_menu::SHORT_SHA_LEN];
            let message = gitpoulpi_i18n::msg::edit_off_branch::gave_up(short);
            *slot = None;
            return Some(Step::GiveUp(message));
        }
        Some(Step::Wait)
    })
}
