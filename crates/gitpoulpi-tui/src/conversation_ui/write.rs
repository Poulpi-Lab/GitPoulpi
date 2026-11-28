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

use gitpoulpi_hosting::conversation::{Actor, Comment, PostingIdentity, TimelineEntry};

pub(crate) use gitpoulpi_link::conversation_write::*;

use super::worker::Source;
use super::{Conversation, Writing};
use crate::hosting_task::{error_text, request_task_failed, ClientFactory};
use crate::redraw::NotifySend as _;

/// What the worker says.
pub(crate) enum WMsg {
    /// The host's answer: the comment it holds now (a delete has none).
    Done(Box<Result<Option<Comment>, String>>),
    /// Who a post is made as, asked again after a post (a first post can
    /// teach an App bot's name).
    Identity(Result<PostingIdentity, String>),
    Finished,
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
                    // Other windows showing this item read it again.
                    crate::hosting_notify::notify_item(
                        &source.integration.id,
                        project,
                        key,
                        match &op {
                            WriteOp::Post { .. } | WriteOp::Reply { .. } => {
                                gitpoulpi_ipc::HostingChange::Posted
                            }
                            WriteOp::Edit { .. } => gitpoulpi_ipc::HostingChange::Edited,
                            WriteOp::Delete { .. } => gitpoulpi_ipc::HostingChange::Deleted,
                        },
                    );
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

// -- the widget's side -----------------------------------------------------------

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
