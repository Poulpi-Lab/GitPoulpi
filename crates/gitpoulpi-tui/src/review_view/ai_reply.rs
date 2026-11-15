//! What the review's comment box and Finish summary give their "Write with
//! AI" help: the request being reviewed (header, description, the
//! conversation and the code threads) and what the text is for (a comment, an
//! answer in one thread, a comment on a line, the summary). The AI only
//! drafts; the text goes out through the box's own Post / Save and the
//! Finish dialog.

use super::popups::ComposeFor;
use super::ReviewView;
use crate::md_editor::ai::{AiContextSource, AiScope};

impl ReviewView {
    /// The helper's scope and source for a text of this review, `None` while
    /// AI is off or the request is not loaded yet.
    pub(super) fn composer_ai(
        &self,
        purpose: Option<&ComposeFor>,
    ) -> Option<(AiScope, Box<dyn AiContextSource>)> {
        #[cfg(feature = "ai")]
        {
            self.composer_ai_request(purpose)
        }
        #[cfg(not(feature = "ai"))]
        {
            let _ = purpose;
            None
        }
    }

    #[cfg(feature = "ai")]
    fn composer_ai_request(
        &self,
        purpose: Option<&ComposeFor>,
    ) -> Option<(AiScope, Box<dyn AiContextSource>)> {
        use gitpoulpi_ai::hosting_reply::{Entry, ItemDigest, ReplyRequest, ReplyTarget};

        if !self.ai_on() {
            return None;
        }
        let host = self.host.as_ref()?;
        let target = match purpose {
            None | Some(ComposeFor::DraftText) => ReplyTarget::ReviewSummary,
            Some(ComposeFor::Conversation) => ReplyTarget::Comment,
            Some(ComposeFor::Reply { thread_id }) => ReplyTarget::ThreadReply {
                thread: thread_id.clone(),
            },
            Some(ComposeFor::Line { path, line, .. }) => ReplyTarget::LineComment {
                path: path.clone(),
                line: Some(*line),
                hunk: None,
            },
            Some(ComposeFor::EditPending(i)) => match self.pending.get(*i) {
                Some(c) => ReplyTarget::LineComment {
                    path: c.path.clone(),
                    line: Some(c.line),
                    hunk: None,
                },
                None => ReplyTarget::Comment,
            },
            Some(ComposeFor::Suggestion(_)) => ReplyTarget::Comment,
        };
        let summary = &host.detail.summary;
        let mut request = ReplyRequest::new(target);
        request.item = Some(ItemDigest {
            summary: format!(
                "Pull request {} \"{}\" ({:?}) by {}",
                summary.iid_label,
                summary.title.trim(),
                summary.state,
                summary.author
            ),
            description: host.detail.body.clone(),
        });
        for c in &host.conversation {
            request.entries.push(Entry {
                thread_ids: std::iter::once(c.id.clone())
                    .chain(c.in_reply_to.clone())
                    .collect(),
                text: format!(
                    "Comment by {} on {}:\n{}",
                    c.author,
                    c.created_at,
                    c.body.trim()
                ),
                thread_text: None,
            });
        }
        for t in &host.threads {
            let place = match t.line.or(t.original_line) {
                Some(line) => format!("{}:{line}", t.path),
                None => t.path.clone(),
            };
            let comments: Vec<String> = t
                .comments
                .iter()
                .map(|c| {
                    format!(
                        "Comment by {} on {}:\n{}",
                        c.author,
                        c.created_at,
                        c.body.trim()
                    )
                })
                .collect();
            let text = format!("Code thread on {place}\n{}", comments.join("\n\n"));
            request.entries.push(Entry {
                thread_ids: std::iter::once(t.id.clone())
                    .chain(t.comments.iter().map(|c| c.id.clone()))
                    .collect(),
                text,
                thread_text: None,
            });
        }
        let scope = AiScope {
            session: self.sug.session().to_string(),
            root: self.target.repo_root.clone(),
        };
        Some((
            scope,
            Box::new(crate::md_editor::ai::ReplyTemplate(request)),
        ))
    }
}
