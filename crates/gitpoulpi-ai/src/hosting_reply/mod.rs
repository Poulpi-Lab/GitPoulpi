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
//! per-use hint and, when improving, their draft.
//!
//! **Repository access is off unless the user ticked "Let the AI read the
//! repository" for this use** (`ai.hosting_reply.read_repository` is only the
//! default of that box, off). Then [`repo_access::read_repository`] sets
//! [`RepoAccess`]: a local agent reads the repository itself with read-only
//! tools, any other profile gets a [`repo_context::RepoContext`] built through
//! a [`repo_context::RepoReader`] (privacy first, budgeted, cached in
//! [`cache`]). Either way the AI only drafts.

use std::path::PathBuf;
use std::sync::Arc;

use gitpoulpi_config::AiAction;
use gitpoulpi_core::domain::Oid;
use serde::Serialize;

use crate::assist::Job;
use crate::limits::Cut;
use crate::{AiClient, ContextBlock, ContextSummary, Error, Result, Turn};
use repo_context::{AgentReads, RepoContext};

pub mod cache;
#[cfg(feature = "conversation")]
pub mod convert;
pub mod exchange;
pub mod plan;
pub mod prompt;
pub mod repo_access;
pub mod repo_context;

pub use exchange::Exchange;
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

/// Whether the AI may read the repository for this use. Off unless the user
/// ticked "Let the AI read the repository" for it (its default is
/// `ai.hosting_reply.read_repository`, off); [`repo_access::read_repository`]
/// sets the other two.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum RepoAccess {
    #[default]
    Off,
    /// A local agent reads the repository itself, in its working folder,
    /// with read-only tools (the job was set up by
    /// [`crate::assist::Job::with_read_only_agent`]).
    AgentReadOnly {
        root: PathBuf,
        head: Oid,
        /// The read-only tools it keeps, for "What is sent".
        tools: String,
        /// Whether its session is new or continued; `None`: one-shot.
        session: Option<AgentSession>,
    },
    /// GitPoulpi read the repository and sends the result as context.
    Built(Arc<RepoContext>),
}

/// How a local agent's session stands for one reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSession {
    Started,
    Continued,
}

impl AgentSession {
    pub fn name(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Continued => "continued",
        }
    }
}

/// The commits a pull request carries, as revisions a [`RepoReader`] can
/// diff.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiffRange {
    pub base: String,
    pub head: String,
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
    /// The range of a pull request, so its diff can go in the repository
    /// context (only read when the repository is).
    pub diff: Option<DiffRange>,
    /// The earlier requests of this exchange (a first draft, then "shorter"
    /// ...), oldest first, with their answers. When the model would not
    /// remember them (anything but a resumed agent session) they go along as
    /// earlier turns, see [`exchange`]. The `hint` and `draft` above are the
    /// newest request, not part of it.
    pub exchange: Vec<Exchange>,
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
            diff: None,
            exchange: Vec::new(),
        }
    }

    /// Whether a resumed agent session already holds the exchange, so
    /// GitPoulpi must not give it back.
    fn session_remembers(&self) -> bool {
        matches!(
            self.repo,
            RepoAccess::AgentReadOnly {
                session: Some(AgentSession::Continued),
                ..
            }
        )
    }

    /// The repository part of the request as the model and the dialog see
    /// it: the context block (empty text for an agent that reads for itself).
    pub fn context_block(&self) -> Option<ContextBlock> {
        match &self.repo {
            RepoAccess::Off => None,
            RepoAccess::Built(context) => Some(context.block()),
            RepoAccess::AgentReadOnly {
                root,
                head,
                tools,
                session,
            } => Some(ContextBlock {
                text: String::new(),
                summary: ContextSummary {
                    head: head.to_hex().chars().take(7).collect(),
                    agent: Some(AgentReads {
                        tools: tools.clone(),
                        folder: root.display().to_string(),
                        session: session.map_or("none", AgentSession::name).to_string(),
                    }),
                    ..ContextSummary::default()
                },
            }),
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
    /// Earlier turns of the exchange sent again as messages.
    #[serde(skip_serializing_if = "is_zero")]
    pub history_turns: usize,
    /// Earlier turns left out for lack of room.
    #[serde(skip_serializing_if = "is_zero")]
    pub history_omitted: usize,
    /// Whether the AI read the repository (by a built context or as an
    /// agent with read-only tools).
    pub repository: bool,
    /// What of the repository, when it was read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<RepoReport>,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

/// What the repository part of a request was.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RepoReport {
    /// `context` (GitPoulpi read it) or `agent` (the agent reads it).
    pub route: &'static str,
    /// The commit, short.
    pub head: String,
    pub parts: Vec<repo_context::PartSummary>,
    pub bytes: usize,
    pub budget: usize,
    pub reused_parts: usize,
    /// The whole context came from the cache.
    pub context_reused: bool,
    pub cuts: Vec<repo_context::ContextCut>,
    pub left_out: Vec<String>,
    /// The provider's prompt cache: tokens read and written.
    pub provider_cache_read_tokens: u64,
    pub provider_cache_creation_tokens: u64,
    /// The tools of an agent that reads the repository.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_tools: Option<String>,
    /// `started`, `continued`, or `none` for an agent run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_session: Option<String>,
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
    let block = request.context_block();
    let done = job
        .run_context_with(client, block.as_ref(), &built.history, &built.user)
        .await?;
    finish(request, &built, done)
}

/// [`draft_with`] over the real transport (or the agent subprocess).
#[cfg(feature = "network")]
pub async fn draft(job: &Job, request: &ReplyRequest) -> Result<Draft> {
    let built = prepare(job, request).await?;
    let block = request.context_block();
    let done = job
        .run_context(block.as_ref(), &built.history, &built.user)
        .await?;
    finish(request, &built, done)
}

fn finish(request: &ReplyRequest, built: &Built, done: crate::Completion) -> Result<Draft> {
    let repo = match &request.repo {
        RepoAccess::Off => None,
        RepoAccess::Built(context) => Some(RepoReport {
            route: "context",
            head: context.key.head.to_hex().chars().take(7).collect(),
            parts: context.summary().parts,
            bytes: context.total,
            budget: context.budget,
            reused_parts: context.reused_parts(),
            context_reused: !context.parts.is_empty()
                && context.reused_parts() == context.parts.len(),
            cuts: context.cuts.clone(),
            left_out: context.left_out.clone(),
            provider_cache_read_tokens: done.usage.cache_read_tokens,
            provider_cache_creation_tokens: done.usage.cache_creation_tokens,
            agent_tools: None,
            agent_session: None,
        }),
        RepoAccess::AgentReadOnly {
            head,
            tools,
            session,
            ..
        } => Some(RepoReport {
            route: "agent",
            head: head.to_hex().chars().take(7).collect(),
            parts: Vec::new(),
            bytes: 0,
            budget: 0,
            reused_parts: 0,
            context_reused: false,
            cuts: Vec::new(),
            left_out: Vec::new(),
            provider_cache_read_tokens: 0,
            provider_cache_creation_tokens: 0,
            agent_tools: Some(tools.clone()),
            agent_session: Some(session.map_or("none", AgentSession::name).to_string()),
        }),
    };
    Ok(Draft {
        text: parse_draft(&done.text)?,
        improved: is_improving(request),
        context: ContextReport {
            entries_sent: built.entries_sent,
            entries_omitted: built.entries_omitted,
            history_turns: built.history.len(),
            history_omitted: built.history_omitted,
            repository: repo.is_some(),
            repo,
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
    let budget = job.limit.limits.reply_context;
    // An exchange the model does not remember is given back: the first
    // message is the conversation as the first request asked it, then each
    // answer and the next instruction; the request itself is only the new
    // instruction. A resumed agent session needs none of it.
    let first = if request.session_remembers() {
        None
    } else {
        request.exchange.first()
    };
    let message = |cap: usize| match first {
        Some(first) => prompt::build_with(request, &first.hint, first.draft.as_deref(), cap),
        None => build(request, cap),
    };
    let capped = message(budget);
    let cuts: Vec<Cut> = capped.cuts.clone();
    let mut built = if job.limit.decide(cuts).await? {
        message(usize::MAX)
    } else {
        capped
    };
    if first.is_some() {
        let mut turns = vec![Turn {
            user: std::mem::take(&mut built.user),
            assistant: request.exchange[0].answer.clone(),
        }];
        for earlier in &request.exchange[1..] {
            turns.push(Turn {
                user: prompt::follow_up(&earlier.hint, earlier.draft.as_deref()),
                assistant: earlier.answer.clone(),
            });
        }
        (built.history, built.history_omitted) = exchange::trim(turns, budget);
        built.user = prompt::follow_up(&request.hint, request.draft.as_deref());
    }
    Ok(built)
}

#[cfg(test)]
mod repo_tests;
#[cfg(test)]
mod tests;
