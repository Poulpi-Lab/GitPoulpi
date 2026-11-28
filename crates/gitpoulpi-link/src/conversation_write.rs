//! What a frontend writes to a conversation and how it shows it before the
//! host has answered: the write operations (post, reply, edit, delete), the
//! optimistic entry an unanswered post appears as, and the settle step that
//! replaces it by the host's answer or takes it back. The worker thread that
//! sends the write and the comment box stay in the frontend.
//!
//! Posting to a host is an outward action: nothing here is journaled and
//! nothing can be undone.

use gitpoulpi_hosting::conversation::{
    Actor, ActorKind, CodeThread, Comment, IdentityKind, Markup, MarkupFlavor, PostingIdentity,
    TimelineEntry,
};

/// What to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteOp {
    /// A comment on the item. `internal`: only members see it (GitLab).
    Post {
        body: String,
        internal: bool,
    },
    /// An answer in a discussion or a code thread.
    Reply {
        discussion_id: String,
        body: String,
    },
    /// A new text for one of our comments.
    Edit {
        id: String,
        body: String,
    },
    Delete {
        id: String,
    },
}

impl WriteOp {
    pub fn body(&self) -> Option<&str> {
        match self {
            Self::Post { body, .. } | Self::Reply { body, .. } | Self::Edit { body, .. } => {
                Some(body)
            }
            Self::Delete { .. } => None,
        }
    }
}

/// The outcome the surface acts on, once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteDone {
    pub op: WriteOp,
    pub result: Result<(), String>,
}

// -- the optimistic view ---------------------------------------------------------

/// The id prefix of an entry the host has not answered for yet.
pub const PENDING: &str = "pending:";

/// Whether `id` is an entry still being sent.
pub fn is_pending(id: &str) -> bool {
    id.starts_with(PENDING)
}

/// `2026-11-15T09:30:00Z` for unix `secs`.
pub fn iso_utc(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rest = secs.rem_euclid(86_400);
    // Civil from days (Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

/// The author an unanswered post is shown under.
pub fn pending_author(identity: Option<&PostingIdentity>) -> Actor {
    match identity {
        Some(id) => {
            let login = id.login.clone().unwrap_or_else(|| id.display.clone());
            let kind = match id.kind {
                IdentityKind::AppBot | IdentityKind::ProjectOrGroupBot => ActorKind::Bot,
                _ => ActorKind::User,
            };
            Actor::new(login, id.display.clone(), kind)
        }
        None => Actor::unknown(),
    }
}

/// A comment as it will look once the host has it.
pub fn pending_comment(
    seq: u64,
    author: Actor,
    body: &str,
    internal: bool,
    now: i64,
    discussion: Option<&str>,
) -> Comment {
    Comment {
        id: format!("{PENDING}{seq}"),
        author,
        body: Markup::new(body, MarkupFlavor::CommonMark),
        created_at: iso_utc(now),
        updated_at: None,
        edited: false,
        reactions: Vec::new(),
        in_reply_to: None,
        discussion_id: discussion.map(str::to_string),
        internal,
        can_edit: Some(false),
        can_delete: Some(false),
        web_url: None,
    }
}

fn comments_mut(entry: &mut TimelineEntry) -> Vec<&mut Comment> {
    match entry {
        TimelineEntry::Comment(c) => vec![c],
        TimelineEntry::Thread(t) => t.comments.iter_mut().collect(),
        _ => Vec::new(),
    }
}

/// Shows `op` as if the host had accepted it.
pub fn apply(entries: &mut Vec<TimelineEntry>, op: &WriteOp, pending: Option<Comment>) {
    match op {
        WriteOp::Post { .. } => {
            if let Some(c) = pending {
                entries.push(TimelineEntry::Comment(c));
            }
        }
        WriteOp::Reply { discussion_id, .. } => {
            let Some(c) = pending else { return };
            let thread = entries.iter_mut().find_map(|e| match e {
                TimelineEntry::Thread(t) if t.id == *discussion_id => Some(t),
                _ => None,
            });
            match thread {
                Some(t) => t.comments.push(c),
                None => entries.push(TimelineEntry::Comment(c)),
            }
        }
        WriteOp::Edit { id, body } => {
            for e in entries.iter_mut() {
                for c in comments_mut(e) {
                    if c.id == *id {
                        c.body = Markup::new(body.as_str(), MarkupFlavor::CommonMark);
                        c.edited = true;
                    }
                }
            }
        }
        WriteOp::Delete { id } => {
            entries.retain_mut(|e| match e {
                TimelineEntry::Comment(c) => c.id != *id,
                TimelineEntry::Thread(t) => {
                    t.comments.retain(|c| c.id != *id);
                    !t.comments.is_empty()
                }
                _ => true,
            });
        }
    }
}

/// Puts the host's own answer where the optimistic copy was.
pub fn settle(entries: &mut [TimelineEntry], op: &WriteOp, answer: Option<Comment>) {
    let Some(answer) = answer else { return };
    let target = match op {
        WriteOp::Post { .. } | WriteOp::Reply { .. } => None,
        WriteOp::Edit { id, .. } => Some(id.as_str()),
        WriteOp::Delete { .. } => return,
    };
    for e in entries.iter_mut() {
        for c in comments_mut(e) {
            let hit = match target {
                Some(id) => c.id == id,
                None => is_pending(&c.id),
            };
            if hit {
                let keep_discussion = c.discussion_id.clone();
                let keep_reply = c.in_reply_to.clone();
                *c = answer.clone();
                if c.discussion_id.is_none() {
                    c.discussion_id = keep_discussion;
                }
                if c.in_reply_to.is_none() {
                    c.in_reply_to = keep_reply;
                }
                return;
            }
        }
    }
}

/// The thread a discussion id names, when the entries hold one.
pub fn thread_of<'a>(entries: &'a [TimelineEntry], discussion_id: &str) -> Option<&'a CodeThread> {
    entries.iter().find_map(|e| match e {
        TimelineEntry::Thread(t) if t.id == discussion_id => Some(t),
        _ => None,
    })
}

/// Where a reply to the selected entry goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplyTo {
    /// Into the discussion or code thread (`label` names it).
    Discussion { id: String, label: String },
    /// A new comment with the entry quoted.
    Quote,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_utc_formats_known_instants() {
        assert_eq!(iso_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_utc(1_714_557_600), "2024-05-01T10:00:00Z");
        assert_eq!(iso_utc(1_709_164_799), "2024-02-28T23:59:59Z");
        assert_eq!(iso_utc(1_709_164_800), "2024-02-29T00:00:00Z");
        assert_eq!(iso_utc(1_709_251_200), "2024-03-01T00:00:00Z");
    }
}
