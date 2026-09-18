//! The proactive conflict badge: "Conflicts with origin/main in 2 files",
//! a strip on the Log's border row (the banners' row, below them in
//! priority), and the state behind it (plan `repo-settings.md`, slice 3).
//!
//! # Where the answer comes from
//!
//! The daemon's `PredictConflicts` (`gitpoulpi_daemon::conflict_prevention`):
//! it picks the base branch, merges in memory, caches by `(head, base)` and
//! debounces. This side only decides *when to ask* and *what to show*, and
//! never waits: the request goes down the shared raw queue and the answer
//! lands in a [`Reply`] polled each frame (the `graph::total` shape).
//!
//! # When it asks
//!
//! Once per key: a hash of every ref's target (a commit, a fetch that moved
//! the base, a checkout) plus the settings generation (Settings was closed:
//! a switch or the Base branches may have changed). A new key while one
//! request waits sends the next one at once: the daemon supersedes the older
//! (that is its debounce), and its answer is dropped here.
//!
//! # No stale badge
//!
//! The strip is drawn only when the answer's HEAD commit is HEAD now
//! ([`ConflictBadge::shown`]), so a commit, a checkout or a rebase hides it
//! in the same frame, before any new answer exists. A detached HEAD asks
//! nothing and shows nothing.

use std::time::{Duration, Instant};

use gitpoulpi_core::domain::conflict_prevention::{ConflictPrediction, PredictionStatus};
use gitpoulpi_core::domain::{Oid, Ref, RefKind};
use gitpoulpi_i18n::msg::conflict_prevention as t;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::daemon_link::Reply;
use crate::operation_banner::{paint_strip, StripButton};

/// What the daemon waits for before it merges: the repository must be quiet.
pub(crate) const DEBOUNCE_MS: u64 = 2000;

/// A superseded request (another window asked) is asked again after this.
const RETRY_AFTER: Duration = Duration::from_secs(3);

#[derive(Default)]
pub(crate) struct ConflictBadge {
    /// The last answer and the key it was asked for.
    known: Option<(u64, ConflictPrediction)>,
    /// The request in flight and its key.
    pending: Option<(u64, Reply<ConflictPrediction>)>,
    /// An answer that failed is not asked again for the same key.
    failed_key: Option<u64>,
    retry_at: Option<Instant>,
    /// Settings was open last tick (its closing forgets the answer).
    prefs_open: bool,
}

/// Hash of what a prediction depends on: every ref's target and the
/// settings generation.
pub(crate) fn key_of(refs: &[Ref], settings_generation: u64) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for r in refs {
        match &r.kind {
            RefKind::LocalBranch { name } => (0u8, name, r.is_head).hash(&mut hasher),
            RefKind::RemoteBranch { remote, name } => (1u8, remote, name).hash(&mut hasher),
            // A tag is nobody's base.
            RefKind::Tag { .. } => continue,
        }
        r.target.hash(&mut hasher);
    }
    settings_generation.hash(&mut hasher);
    hasher.finish()
}

impl ConflictBadge {
    /// Polls the answer and asks for a new one when `key` changed. `key` is
    /// `None` when there is nothing to ask about (no refs yet, a detached
    /// HEAD): the old answer is dropped. `start` sends the request; it runs
    /// at most once per call.
    pub(crate) fn tick(
        &mut self,
        key: Option<u64>,
        now: Instant,
        start: impl FnOnce() -> Reply<ConflictPrediction>,
    ) {
        if let Some((asked, reply)) = &self.pending {
            if let Some(answer) = reply.take() {
                let asked = *asked;
                self.pending = None;
                match answer {
                    Ok(p) if p.status == PredictionStatus::Superseded => {
                        self.retry_at = Some(now + RETRY_AFTER);
                    }
                    // Only an answer to the question still being asked.
                    Ok(p) if Some(asked) == key => self.known = Some((asked, p)),
                    Ok(_) => {}
                    Err(_) => self.failed_key = Some(asked),
                }
            }
        }
        let Some(key) = key else {
            self.known = None;
            self.pending = None;
            return;
        };
        let answered = matches!(&self.known, Some((k, _)) if *k == key);
        let in_flight = matches!(&self.pending, Some((k, _)) if *k == key);
        let waiting = self.retry_at.is_some_and(|at| now < at);
        if !answered && !in_flight && !waiting && self.failed_key != Some(key) {
            self.retry_at = None;
            self.pending = Some((key, start()));
        }
    }

    /// Tells the badge whether Settings is open; `true` once, when it was just
    /// closed (the answer is forgotten: a switch or the Base branches may have
    /// changed).
    pub(crate) fn settings_closed(&mut self, open: bool) -> bool {
        let closed = self.prefs_open && !open;
        self.prefs_open = open;
        if closed {
            self.forget();
        }
        closed
    }

    /// What was answered may no longer apply.
    fn forget(&mut self) {
        self.known = None;
        self.pending = None;
        self.failed_key = None;
        self.retry_at = None;
    }

    /// The prediction to show: conflicts, for the HEAD that is checked out
    /// now.
    pub(crate) fn shown(&self, head: Option<Oid>) -> Option<&ConflictPrediction> {
        let (_, p) = self.known.as_ref()?;
        let head = head?;
        (p.status == PredictionStatus::Conflicts && p.head.as_deref() == Some(&head.to_hex()))
            .then_some(p)
    }

    /// Whether the last answer says detection is off (for the palette's note).
    pub(crate) fn is_off(&self) -> bool {
        matches!(&self.known, Some((_, p)) if p.status == PredictionStatus::Disabled)
    }
}

/// "Conflicts with origin/main in 2 files".
pub(crate) fn text(prediction: &ConflictPrediction) -> String {
    t::badge(
        prediction.files.len() as u64,
        prediction.base_ref.as_deref().unwrap_or("?"),
    )
}

/// Draws the strip on `area`'s first row and returns the button's rect.
pub(crate) fn render(
    frame: &mut Frame,
    area: Rect,
    prediction: &ConflictPrediction,
    hovered: bool,
) -> Vec<Rect> {
    let strip = [StripButton {
        label: t::BADGE_BUTTON.get(),
        danger: false,
        hovered,
    }];
    paint_strip(frame, area, &[text(prediction)], &strip)
        .into_iter()
        .map(|(rect, _)| rect)
        .collect()
}

/// A branch as the daemon names it (`main`, `origin/main`) among `refs`.
fn find_ref<'a>(refs: &'a [Ref], name: &str) -> Option<&'a Ref> {
    refs.iter().find(|r| match &r.kind {
        RefKind::LocalBranch { name: n } => n == name,
        RefKind::RemoteBranch { remote, name: n } => format!("{remote}/{n}") == name,
        RefKind::Tag { .. } => false,
    })
}

/// The popup's two actions: the branch menu's own Merge and Rebase, run on
/// the base branch (both journaled, so Undo takes them back; a conflicting
/// merge stops in the merge editor like any other).
pub(crate) fn apply_popup_action(
    action: crate::conflict_popup::PopupAction,
    refs: &gitpoulpi_core::Result<Vec<Ref>>,
    link: &crate::daemon_link::DaemonLink,
    last_message: &mut Option<String>,
) {
    use crate::conflict_popup::PopupAction as A;
    use gitpoulpi_i18n::msg::menu_ref as m;
    let (A::Merge(name) | A::Rebase(name)) = &action;
    let Some(base) = refs
        .as_ref()
        .ok()
        .and_then(|refs| find_ref(refs, name))
        .cloned()
    else {
        *last_message = Some(crate::git_actions::NO_SUCH_LOCAL_BRANCH.get().to_string());
        return;
    };
    match action {
        A::Merge(name) => {
            link.submit_action(crate::daemon_link::Action::Merge(base));
            *last_message = Some(m::merging(&name));
        }
        A::Rebase(name) => {
            link.submit_action(crate::daemon_link::Action::Rebase(base));
            *last_message = Some(m::rebasing_onto(&name));
        }
    }
}

#[cfg(test)]
mod tests;
