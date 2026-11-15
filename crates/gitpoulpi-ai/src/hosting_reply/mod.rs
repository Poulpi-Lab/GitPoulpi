//! Writing help for conversations on a host: [`AiAction::HostingReply`]
//! drafts a comment, a thread answer, a review comment or a review summary,
//! [`AiAction::IssueBody`] the description of a new issue.
//!
//! **The AI only drafts.** [`draft`] turns a [`ReplyRequest`] into a
//! [`Draft`] (text and a report of what went into the request) and nothing
//! else: this module knows the conversation *shapes* of `gitpoulpi-hosting`
//! but no host client, so no code path leads from here to a post, an edit or
//! a delete. Posting stays the user's `Ctrl+Enter`.
//!
//! The request is built from the conversation ([`prompt::build`]): the item's
//! header and description, the thread being answered, the recent entries
//! within `ai.limits.hosting_reply_context_bytes` ([`plan`]), then the user's
//! per-use hint and, when improving, their draft. **No repository access
//! yet** ([`RepoAccess`] has no way to ask for it): a later slice adds it
//! behind the per-use "Let the AI read the repository" choice, off by default
//! (`ai.hosting_reply.read_repository`).

use gitpoulpi_config::AiAction;
use serde::Serialize;

use crate::assist::Job;
use crate::limits::Cut;
use crate::{AiClient, Error, Result};

#[cfg(feature = "conversation")]
pub mod convert;
pub mod plan;
pub mod prompt;

pub use prompt::{build, Built, BASE_PROMPT};

/// What the text is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplyTarget {
    /// A new comment in the item's conversation.
    Comment,
    /// An answer in one thread of the conversation: `thread` is the id of a
    /// code thread, of a comment, or of a comment's discussion.
    ThreadReply { thread: String },
    /// A new review comment on a line of code (`hunk`: the diff excerpt around it).
    LineComment {
        path: String,
        line: Option<u32>,
        hunk: Option<String>,
    },
    /// The summary of the review being submitted.
    ReviewSummary,
    /// The description of a new issue (the [`AiAction::IssueBody`] action).
    IssueBody { title: String },
}

impl ReplyTarget {
    /// The action that drafts it.
    pub fn action(&self) -> AiAction {
        match self {
            Self::IssueBody { .. } => AiAction::IssueBody,
            _ => AiAction::HostingReply,
        }
    }

    /// The name used in the CLI's answer.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Comment => "comment",
            Self::ThreadReply { .. } => "thread_reply",
            Self::LineComment { .. } => "line_comment",
            Self::ReviewSummary => "review_summary",
            Self::IssueBody { .. } => "issue_body",
        }
    }
}

/// Whether the AI may read the repository for this use. Reading it is not
/// built yet, so the only value is `Off`; the per-use choice and its default
/// (`ai.hosting_reply.read_repository`, off) come with that slice.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum RepoAccess {
    #[default]
    Off,
}

/// What the model is told about the item: its one-line header and its
/// description, both written by other people. Built by the caller (see
/// [`convert`] for the conversation shapes of `gitpoulpi-hosting`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemDigest {
    /// `Issue #12 "Title" (open) by alice, labels: bug`.
    pub summary: String,
    pub description: String,
}

/// One entry of the conversation, rendered by the caller: a comment, a
/// review, a code thread. System events are not entries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Entry {
    /// The ids that say which thread it belongs to: its own id, its
    /// discussion's, the id of the comment it answers. A
    /// [`ReplyTarget::ThreadReply`] picks the entries holding its id.
    pub thread_ids: Vec<String>,
    /// What the model sees in the list of recent entries.
    pub text: String,
    /// What it shows when it is the thread being answered (a code thread
    /// adds its diff hunk); `None`: `text`.
    pub thread_text: Option<String>,
}

/// Everything a draft is made from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplyRequest {
    pub target: ReplyTarget,
    /// The item being talked about; `None` for a new issue.
    pub item: Option<ItemDigest>,
    /// The conversation, oldest first (the newest window of it).
    pub entries: Vec<Entry>,
    /// What the user wants in this answer, typed at the moment of use.
    pub hint: String,
    /// The user's own text to improve; `None` or blank writes a new one.
    pub draft: Option<String>,
    /// The markup the host takes, named for the model ("GitHub Flavored
    /// Markdown").
    pub flavor: String,
    pub repo: RepoAccess,
}

impl ReplyRequest {
    pub fn new(target: ReplyTarget) -> Self {
        Self {
            target,
            item: None,
            entries: Vec::new(),
            hint: String::new(),
            draft: None,
            flavor: "Markdown".into(),
            repo: RepoAccess::Off,
        }
    }
}

/// What went into a request, for the status line and the CLI's answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContextReport {
    /// Conversation entries in the request.
    pub entries_sent: usize,
    /// Entries left out for lack of room.
    pub entries_omitted: usize,
    /// Whether the repository was read (never, for now).
    pub repository: bool,
}

/// The result: a text for the user to read, edit and post themselves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Draft {
    pub text: String,
    /// The text improves the user's own draft rather than being new.
    pub improved: bool,
    pub context: ContextReport,
}

/// The text to post out of the model's answer: trimmed, and not wrapped in
/// a code fence the model put around the whole answer.
pub fn parse_draft(answer: &str) -> Result<String> {
    let text = answer.trim();
    let text = unfence(text).unwrap_or(text).trim();
    if text.is_empty() {
        return Err(Error::Empty);
    }
    Ok(text.to_string())
}

/// The inside of a text that is one single fenced block of plain text or
/// Markdown (what a model does when it "quotes" its answer); `None` when the
/// text is anything else, so a reply that really is a code block stays one.
fn unfence(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("```")?;
    let (info, body) = rest.split_once('\n')?;
    if !matches!(
        info.trim().to_ascii_lowercase().as_str(),
        "" | "markdown" | "md" | "text" | "txt"
    ) {
        return None;
    }
    let body = body.strip_suffix("```")?;
    // Another fence inside means it is a reply with code, not a wrapper.
    (!body.contains("```")).then_some(body)
}

/// Drafts the text for `request` with `job` (a [`AiAction::HostingReply`] or
/// [`AiAction::IssueBody`] job), over `client`. The conversation is held to
/// `ai.limits.hosting_reply_context_bytes`; over it, the job's limit policy
/// decides (a frontend asks, the CLI truncates and reports in
/// [`Job::truncated`], `--send-all` lifts the cap).
pub async fn draft_with(job: &Job, client: &dyn AiClient, request: &ReplyRequest) -> Result<Draft> {
    let built = prepare(job, request).await?;
    let answer = job.run_with(client, &built.user).await?;
    finish(request, &built, &answer)
}

/// [`draft_with`] over the real transport (or the agent subprocess).
#[cfg(feature = "network")]
pub async fn draft(job: &Job, request: &ReplyRequest) -> Result<Draft> {
    let built = prepare(job, request).await?;
    let answer = job.run(&built.user).await?;
    finish(request, &built, &answer)
}

fn finish(request: &ReplyRequest, built: &Built, answer: &str) -> Result<Draft> {
    Ok(Draft {
        text: parse_draft(answer)?,
        improved: is_improving(request),
        context: ContextReport {
            entries_sent: built.entries_sent,
            entries_omitted: built.entries_omitted,
            repository: false,
        },
    })
}

fn is_improving(request: &ReplyRequest) -> bool {
    request
        .draft
        .as_deref()
        .is_some_and(|d| !d.trim().is_empty())
}

/// Builds the user message, asking the job's limit policy when the
/// conversation is over its budget.
pub async fn prepare(job: &Job, request: &ReplyRequest) -> Result<Built> {
    if request.target.action() != job.action {
        return Err(Error::Invalid(format!(
            "this request is for the {} action, not {}",
            request.target.action().key(),
            job.action.key()
        )));
    }
    let capped = build(request, job.limit.limits.reply_context);
    let cuts: Vec<Cut> = capped.cuts.clone();
    if job.limit.decide(cuts).await? {
        return Ok(build(request, usize::MAX));
    }
    Ok(capped)
}

#[cfg(test)]
mod tests;
