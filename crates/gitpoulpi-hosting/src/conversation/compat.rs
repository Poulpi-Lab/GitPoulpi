//! Helpers the code-host providers share when they map into the
//! conversation shapes: the older review types (`ReviewComment`,
//! `ReviewThread`, `SubmittedReview`) become conversation ones, a merged
//! timeline is ordered by time, and the identity source is decided.

use super::model::{
    Actor, ActorKind, CodeThread, Comment, Markup, MarkupFlavor, ReviewEntry, TimelineEntry,
};
use super::time::iso_secs;
use super::TokenSource;
use crate::review::invalid;
use crate::review::model::{ReviewComment, ReviewThread, SubmittedReview};
use crate::Result;

/// Refuses an empty body before any request.
pub(crate) fn check_body(what: &str, body: &str) -> Result<()> {
    if body.trim().is_empty() {
        return Err(invalid(what, "the comment is empty"));
    }
    Ok(())
}

/// An actor known only by its login (the review types carry nothing more).
pub(crate) fn actor_from_login(login: &str) -> Actor {
    if login.trim().is_empty() {
        Actor::unknown()
    } else {
        Actor::new(login, login, ActorKind::User)
    }
}

pub(crate) fn comment_from_review(
    c: ReviewComment,
    discussion_id: Option<&str>,
    flavor: MarkupFlavor,
) -> Comment {
    Comment {
        id: c.id,
        author: actor_from_login(&c.author),
        body: Markup::new(c.body, flavor),
        created_at: c.created_at,
        updated_at: None,
        edited: false,
        reactions: Vec::new(),
        in_reply_to: c.in_reply_to,
        discussion_id: discussion_id.map(str::to_string),
        internal: false,
        can_edit: None,
        can_delete: None,
        web_url: c.web_url,
    }
}

/// A thread of the review model as a code thread; every comment of it
/// carries the thread id as its discussion id.
pub(crate) fn thread_from_review(t: ReviewThread, flavor: MarkupFlavor) -> CodeThread {
    let id = t.id.clone();
    CodeThread {
        id: t.id,
        path: t.path,
        line: t.line,
        original_line: t.original_line,
        side: t.side,
        start_line: t.start_line,
        resolved: t.resolved,
        outdated: t.outdated,
        resolvable: t.resolvable,
        diff_hunk: None,
        comments: t
            .comments
            .into_iter()
            .map(|c| comment_from_review(c, Some(&id), flavor))
            .collect(),
    }
}

pub(crate) fn review_entry(r: SubmittedReview, flavor: MarkupFlavor) -> ReviewEntry {
    ReviewEntry {
        id: r.id,
        author: actor_from_login(&r.author),
        state: r.state,
        body: Markup::new(r.body, flavor),
        submitted_at: r.submitted_at,
    }
}

/// Oldest first; timestamps that do not parse sort first (stable).
pub(crate) fn sort_by_time(entries: &mut [TimelineEntry]) {
    entries.sort_by_key(|e| iso_secs(e.at()).unwrap_or(i64::MIN));
}

/// Where the token comes from: the repository's token command when it acts
/// for this project, else the integration profile's own. The flag says
/// whether the command is in use (its user is then not the profile's).
pub(crate) fn token_source(
    scopes: &crate::scope_handle::ScopeHandle,
    integration_id: &str,
    project: &str,
) -> (TokenSource, bool) {
    if scopes.acts_as_bot(integration_id, project) {
        (TokenSource::RepoTokenCommand, true)
    } else {
        (
            TokenSource::Profile {
                id: integration_id.to_string(),
            },
            false,
        )
    }
}
