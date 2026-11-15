//! Writing to the host: post, reply, edit and delete a comment.
//!
//! Like the reads, a write is a plain thread over a channel drained by the
//! render loop. The widget shows the effect at once (an optimistic entry, an
//! edited body, a removed comment) and takes it back when the host refuses:
//! the surface then gets the error and the text to put back in its comment
//! box, so what the user typed is never lost.
//!
//! Posting to a host is an outward action: nothing here is journaled and
//! nothing can be undone (the Undo story is told by `undo_ui`).

use std::path::PathBuf;
use std::sync::{mpsc, Arc};

use gitpoulpi_hosting::conversation::{
    Actor, ActorKind, CodeThread, Comment, IdentityKind, Markup, MarkupFlavor, PostingIdentity,
    TimelineEntry,
};

use super::worker::Source;
use super::{Conversation, Writing};
use crate::hosting_task::{error_text, request_task_failed, ClientFactory};
use crate::redraw::NotifySend as _;

/// What to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WriteOp {
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
    pub(crate) fn body(&self) -> Option<&str> {
        match self {
            Self::Post { body, .. } | Self::Reply { body, .. } | Self::Edit { body, .. } => {
                Some(body)
            }
            Self::Delete { .. } => None,
        }
    }
}

/// What the worker says.
pub(crate) enum WMsg {
    /// The host's answer: the comment it holds now (a delete has none).
    Done(Box<Result<Option<Comment>, String>>),
    /// Who a post is made as, asked again after a post (a first post can
    /// teach an App bot's name).
    Identity(Result<PostingIdentity, String>),
    Finished,
}

/// The outcome the surface acts on, once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WriteDone {
    pub(crate) op: WriteOp,
    pub(crate) result: Result<(), String>,
}

/// Starts a write. `state_path` is the repository's state file, where the
/// id of a posted comment is kept (`posted_comment_ids`); `conversation` is
/// the key it is kept under.
pub(crate) fn start(
    factory: Arc<dyn ClientFactory>,
    source: Source,
    state_path: Option<PathBuf>,
    conversation: String,
    op: WriteOp,
) -> mpsc::Receiver<WMsg> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let rt = match rt {
            Ok(rt) => rt,
            Err(e) => {
                let _ = tx.send_notify(WMsg::Done(Box::new(Err(request_task_failed(e)))));
                let _ = tx.send_notify(WMsg::Finished);
                return;
            }
        };
        rt.block_on(async {
            let client = match factory.client(&source.integration, &source.all) {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send_notify(WMsg::Done(Box::new(Err(error_text(&e)))));
                    return;
                }
            };
            let (project, key) = (&source.project, &source.key);
            let result: gitpoulpi_hosting::Result<Option<Comment>> = match &op {
                WriteOp::Post { body, internal } => client
                    .post_item_comment(project, key, body, *internal)
                    .await
                    .map(Some),
                WriteOp::Reply {
                    discussion_id,
                    body,
                } => client
                    .reply_in_discussion(project, key, discussion_id, body)
                    .await
                    .map(Some),
                WriteOp::Edit { id, body } => client
                    .edit_item_comment(project, key, id, body)
                    .await
                    .map(Some),
                WriteOp::Delete { id } => client
                    .delete_item_comment(project, key, id)
                    .await
                    .map(|()| None),
            };
            match result {
                Ok(comment) => {
                    remember(state_path.as_deref(), &conversation, &op, comment.as_ref());
                    let _ = tx.send_notify(WMsg::Done(Box::new(Ok(comment))));
                    // A first post can teach an App bot's login.
                    let identity = client
                        .posting_identity(project)
                        .await
                        .map_err(|e| error_text(&e));
                    let _ = tx.send_notify(WMsg::Identity(identity));
                }
                Err(e) => {
                    let _ = tx.send_notify(WMsg::Done(Box::new(Err(error_text(&e)))));
                }
            }
        });
        let _ = tx.send_notify(WMsg::Finished);
    });
    rx
}

/// Keeps (or forgets) the id of a comment of ours in the repository's state.
fn remember(
    state_path: Option<&std::path::Path>,
    conversation: &str,
    op: &WriteOp,
    comment: Option<&Comment>,
) {
    let Some(path) = state_path else { return };
    let mut state = gitpoulpi_session::RepoState::load(path).unwrap_or_default();
    match (op, comment) {
        (WriteOp::Post { .. } | WriteOp::Reply { .. }, Some(c)) => {
            state.record_posted_comment(conversation, &c.id);
        }
        (WriteOp::Delete { id }, _) => state.forget_posted_comment(conversation, id),
        _ => return,
    }
    let _ = state.save(path);
}

// -- the optimistic view ---------------------------------------------------------

/// The id prefix of an entry the host has not answered for yet.
pub(crate) const PENDING: &str = "pending:";

/// Whether `id` is an entry still being sent.
pub(crate) fn is_pending(id: &str) -> bool {
    id.starts_with(PENDING)
}

/// `2026-11-15T09:30:00Z` for unix `secs`.
pub(crate) fn iso_utc(secs: i64) -> String {
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
pub(crate) fn pending_author(identity: Option<&PostingIdentity>) -> Actor {
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
pub(crate) fn pending_comment(
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
pub(crate) fn apply(entries: &mut Vec<TimelineEntry>, op: &WriteOp, pending: Option<Comment>) {
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
pub(crate) fn settle(entries: &mut [TimelineEntry], op: &WriteOp, answer: Option<Comment>) {
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
pub(crate) fn thread_of<'a>(
    entries: &'a [TimelineEntry],
    discussion_id: &str,
) -> Option<&'a CodeThread> {
    entries.iter().find_map(|e| match e {
        TimelineEntry::Thread(t) if t.id == discussion_id => Some(t),
        _ => None,
    })
}

// -- the widget's side -----------------------------------------------------------

/// Where a reply to the selected entry goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReplyTo {
    /// Into the discussion or code thread (`label` names it).
    Discussion { id: String, label: String },
    /// A new comment with the entry quoted.
    Quote,
}

impl Conversation {
    /// Who the comment box posts as, for the optimistic entry.
    fn author(&self) -> Actor {
        pending_author(match &self.identity {
            Some(Ok(id)) => Some(id),
            _ => None,
        })
    }

    /// Where the id of a posted comment is kept (the repository's state
    /// file, and the key of this conversation), and the ones already known.
    pub(crate) fn set_write_context(
        &mut self,
        state_path: Option<PathBuf>,
        key: String,
        posted: Vec<String>,
    ) {
        self.write_ctx = state_path.map(|p| (p, key));
        self.posted = posted.into_iter().collect();
        self.rev += 1;
    }

    /// A write is waiting for the host.
    pub(crate) fn is_writing(&self) -> bool {
        self.writing.is_some()
    }

    /// Where a reply to the selected entry goes.
    pub(crate) fn reply_to(&self) -> ReplyTo {
        let thread_ok = self.caps.as_ref().is_some_and(|c| c.reply_in_thread);
        if !thread_ok {
            return ReplyTo::Quote;
        }
        match self.selected_entry() {
            Some(TimelineEntry::Thread(t)) => ReplyTo::Discussion {
                id: t.id.clone(),
                label: match t.line {
                    Some(line) => format!("{}:{line}", t.path),
                    None => t.path.clone(),
                },
            },
            Some(TimelineEntry::Comment(c)) if !is_pending(&c.id) => {
                match c.discussion_id.as_deref().filter(|d| !d.is_empty()) {
                    Some(id) => ReplyTo::Discussion {
                        id: id.to_string(),
                        label: c.author.display.clone(),
                    },
                    None => ReplyTo::Quote,
                }
            }
            _ => ReplyTo::Quote,
        }
    }

    /// Shows `op` at once and sends it. `false`: nothing was started.
    pub(crate) fn submit(&mut self, op: WriteOp) -> bool {
        if self.writing.is_some() {
            return false;
        }
        self.write_seq += 1;
        let (state_path, conversation) = match &self.write_ctx {
            Some((p, k)) => (Some(p.clone()), k.clone()),
            None => (None, String::new()),
        };
        let pending = match &op {
            WriteOp::Post { body, internal } => Some(pending_comment(
                self.write_seq,
                self.author(),
                body,
                *internal,
                super::now_secs(),
                None,
            )),
            WriteOp::Reply {
                discussion_id,
                body,
            } => Some(pending_comment(
                self.write_seq,
                self.author(),
                body,
                false,
                super::now_secs(),
                Some(discussion_id),
            )),
            _ => None,
        };
        let backup = self.entries.clone();
        if pending.is_some() {
            self.select_when_laid_out = Some(format!("c{PENDING}{}", self.write_seq));
        }
        apply(&mut self.entries, &op, pending);
        self.rev += 1;
        self.write_rx = Some(start(
            std::sync::Arc::clone(&self.factory),
            self.source.clone(),
            state_path,
            conversation,
            op.clone(),
        ));
        self.writing = Some(Writing { op, backup });
        true
    }

    /// Tests: nothing is in flight and nothing waits to be taken.
    #[cfg(test)]
    pub(crate) fn take_idle_for_tests(&self) -> bool {
        self.writing.is_none() && self.write_rx.is_none() && self.write_done.is_none()
    }

    /// The outcome of a finished write, once.
    pub(crate) fn take_write_done(&mut self) -> Option<WriteDone> {
        self.write_done.take()
    }

    pub(super) fn poll_write(&mut self) -> bool {
        let mut changed = false;
        while let Some(rx) = &self.write_rx {
            match rx.try_recv() {
                Ok(WMsg::Done(result)) => {
                    let result = *result;
                    changed = true;
                    let Some(w) = self.writing.take() else {
                        continue;
                    };
                    match result {
                        Ok(answer) => {
                            match (&w.op, &answer) {
                                (WriteOp::Post { .. } | WriteOp::Reply { .. }, Some(c)) => {
                                    self.posted.insert(c.id.clone());
                                }
                                (WriteOp::Delete { id }, _) => {
                                    self.posted.remove(id);
                                }
                                _ => {}
                            }
                            // Keep the new comment selected under its real id.
                            if let (WriteOp::Post { .. } | WriteOp::Reply { .. }, Some(c)) =
                                (&w.op, &answer)
                            {
                                if self.select_when_laid_out.is_some()
                                    || self.sel_key.starts_with(&format!("c{PENDING}"))
                                {
                                    self.select_when_laid_out = Some(format!("c{}", c.id));
                                }
                            }
                            settle(&mut self.entries, &w.op, answer);
                            self.write_done = Some(WriteDone {
                                op: w.op,
                                result: Ok(()),
                            });
                        }
                        Err(why) => {
                            self.entries = w.backup;
                            self.select_when_laid_out = None;
                            self.write_done = Some(WriteDone {
                                op: w.op,
                                result: Err(why),
                            });
                        }
                    }
                    self.rev += 1;
                }
                Ok(WMsg::Identity(identity)) => {
                    changed = true;
                    if identity.is_ok() {
                        self.identity = Some(identity);
                        self.rev += 1;
                    }
                }
                Ok(WMsg::Finished) => {
                    self.write_rx = None;
                    changed = true;
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.write_rx = None;
                    if let Some(w) = self.writing.take() {
                        self.entries = w.backup;
                        self.write_done = Some(WriteDone {
                            op: w.op,
                            result: Err(gitpoulpi_i18n::msg::item_view::LOAD_STOPPED
                                .get()
                                .to_string()),
                        });
                        self.rev += 1;
                    }
                    changed = true;
                    break;
                }
            }
        }
        changed
    }
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
