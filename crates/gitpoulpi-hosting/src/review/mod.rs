//! In-app code review of pull / merge requests (GitHub incl. Enterprise,
//! GitLab incl. self-managed): reading a request, its changed files, its
//! threads and conversation; commenting, replying, resolving; submitting a
//! review. The diff itself comes from git, not from here (see
//! [`model`]'s module docs for how positions map onto it).
//!
//! The operations are [`crate::HostingClient`] methods with `Unsupported`
//! defaults; this module holds the shapes ([`model`]), the per-host
//! implementations ([`github`], [`gitlab`]) and the checks both share.

pub mod api_code;
pub mod model;

#[cfg(feature = "github")]
pub(crate) mod github;
#[cfg(feature = "gitlab")]
pub(crate) mod gitlab;

#[cfg(all(test, feature = "github", feature = "gitlab"))]
mod api_code_tests;
#[cfg(all(test, feature = "github", feature = "gitlab"))]
mod tests;

pub use api_code::{ApiCommit, ApiFileDiff, NOT_SHOWN_BY_THE_HOST};
pub use model::{
    DiffSide, FileStatus, MergeState, PatchSet, PendingComment, PullRequestDetail,
    ReviewCapabilities, ReviewComment, ReviewEvent, ReviewFile, ReviewOutcome, ReviewState,
    ReviewSubmission, ReviewThread, ReviewVote, SubmittedReview, VoteLabel, VoteValue,
};

#[cfg(any(
    feature = "github",
    feature = "gitlab",
    feature = "gitea",
    feature = "gerrit"
))]
use crate::Error;

/// `refs/changes/NN/<change>/<patch set>`: where Gerrit keeps a patch set (NN is
/// the last two digits of the change number, zero-padded).
pub fn change_ref(change: u64, patch_set: u32) -> String {
    format!("refs/changes/{:02}/{change}/{patch_set}", change % 100)
}

/// The patch set number of a `refs/changes/NN/<change>/<ps>` ref.
pub fn patch_set_of_ref(text: &str) -> Option<u32> {
    let rest = text.strip_prefix("refs/changes/")?;
    let mut parts = rest.split('/');
    parts.next()?;
    parts.next()?;
    parts.next()?.parse().ok()
}

/// The local ref a request's head is fetched into: `refs/gitpoulpi/pr/<n>`
/// (never a branch, never checked out). The same name on both hosts.
pub fn review_head_ref(number: u64) -> String {
    format!("refs/gitpoulpi/pr/{number}")
}

/// The local ref the request's *target branch* is fetched into, so the
/// merge base the host diffed against can be computed locally:
/// `refs/gitpoulpi/pr/<n>-base`.
pub fn review_base_ref(number: u64) -> String {
    format!("refs/gitpoulpi/pr/{number}-base")
}

/// The refspec that fetches the request's head, for
/// `GitBackend::fetch_refspec`: `+refs/pull/<n>/head:refs/gitpoulpi/pr/<n>`
/// (GitHub), `+refs/merge-requests/<n>/head:...` (GitLab). `None` for kinds
/// that do not keep the head in a special ref.
pub fn review_head_refspec(kind: crate::IntegrationKind, number: u64) -> Option<String> {
    let (remote_ref, _) = crate::pull_request_head_ref(kind, number)?;
    Some(format!("+{remote_ref}:{}", review_head_ref(number)))
}

/// The refspec that fetches patch set `patch_set` of Gerrit change `number`
/// (`+refs/changes/NN/<change>/<ps>:refs/gitpoulpi/pr/<change>`). `None` for
/// every other kind.
pub fn review_head_refspec_at(
    kind: crate::IntegrationKind,
    number: u64,
    patch_set: u32,
) -> Option<String> {
    (kind.family() == gitpoulpi_config::ApiFamily::Gerrit).then(|| {
        format!(
            "+{}:{}",
            change_ref(number, patch_set),
            review_head_ref(number)
        )
    })
}

/// The local ref patch set `patch_set` of Gerrit change `number` is fetched
/// into when it is the old side of a patch-set comparison:
/// `refs/gitpoulpi/pr/<change>-ps<k>`.
pub fn review_compare_ref(number: u64, patch_set: u32) -> String {
    format!("refs/gitpoulpi/pr/{number}-ps{patch_set}")
}

/// The refspec that fetches patch set `patch_set` into
/// [`review_compare_ref`]. `None` for every kind but Gerrit.
pub fn review_compare_refspec(
    kind: crate::IntegrationKind,
    number: u64,
    patch_set: u32,
) -> Option<String> {
    (kind.family() == gitpoulpi_config::ApiFamily::Gerrit).then(|| {
        format!(
            "+{}:{}",
            change_ref(number, patch_set),
            review_compare_ref(number, patch_set)
        )
    })
}

/// The refspec that fetches the target branch `branch`.
pub fn review_base_refspec(branch: &str, number: u64) -> String {
    format!("+refs/heads/{branch}:{}", review_base_ref(number))
}

#[cfg(any(
    feature = "github",
    feature = "gitlab",
    feature = "gitea",
    feature = "gerrit"
))]
/// A caller mistake the host would reject anyway, caught before any
/// request is sent. Shaped like the host's own 422 so callers handle one
/// kind of error.
pub(crate) fn invalid(what: &str, detail: impl Into<String>) -> Error {
    Error::Api {
        what: what.to_string(),
        status: 422,
        detail: detail.into(),
    }
}

#[cfg(any(
    feature = "github",
    feature = "gitlab",
    feature = "gitea",
    feature = "gerrit"
))]
/// The checks both hosts share before a submission is sent: the event is
/// offered, a verdict that needs words has them, and an empty "comment"
/// review is not posted.
pub(crate) fn validate_submission(
    caps: ReviewCapabilities,
    submission: &ReviewSubmission,
) -> crate::Result<()> {
    const WHAT: &str = "submitting a review";
    if !caps.allows(submission.event) {
        return Err(Error::Unsupported(format!(
            "review event `{}`",
            submission.event.as_str()
        )));
    }
    let has_body = !submission.body.trim().is_empty();
    match submission.event {
        ReviewEvent::RequestChanges if !has_body && submission.comments.is_empty() => {
            return Err(invalid(
                WHAT,
                "a review that requests changes needs a body or at least one comment",
            ));
        }
        ReviewEvent::Comment if !has_body && submission.comments.is_empty() => {
            return Err(invalid(
                WHAT,
                "a comment review needs a body or at least one comment",
            ));
        }
        _ => {}
    }
    for c in &submission.comments {
        if c.body.trim().is_empty() {
            return Err(invalid(
                WHAT,
                format!("the comment on {}:{} has no text", c.path, c.line),
            ));
        }
        if c.path.trim().is_empty() || c.line == 0 {
            return Err(invalid(
                WHAT,
                "a line comment needs a path and a line (1-based)",
            ));
        }
        if let Some(start) = c.start_line {
            if start == 0 || start > c.line {
                return Err(invalid(
                    WHAT,
                    format!(
                        "the range of the comment on {} is backwards ({start}..{})",
                        c.path, c.line
                    ),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(feature = "gitlab")]
/// Counts the added and removed lines of a unified-diff body (hunks only:
/// `@@` headers and `+++`/`---` file headers are not lines of the file).
/// GitLab reports a diff per file but no totals.
pub(crate) fn count_changes(diff: &str) -> (u32, u32) {
    let (mut added, mut removed) = (0u32, 0u32);
    let mut in_hunk = false;
    for line in diff.lines() {
        if line.starts_with("@@") {
            in_hunk = true;
        } else if in_hunk {
            match line.as_bytes().first() {
                Some(b'+') => added += 1,
                Some(b'-') => removed += 1,
                _ => {}
            }
        }
    }
    (added, removed)
}
