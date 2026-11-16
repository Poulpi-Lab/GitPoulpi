//! AI for GitPoulpi: completion clients and per-action prompts.
//!
//! No TUI, CLI or daemon dependency: this crate takes text in and gives
//! text out. Three things live here.
//!
//! - **Clients** ([`AiClient`]): OpenAI chat-completions (also OpenRouter
//!   and any OpenAI-compatible server through a base URL), the Anthropic
//!   Messages API, and a *local agent* (`claude -p`, ...) driven as a
//!   subprocess. HTTP goes through `gitpoulpi-hosting`'s `HttpTransport`,
//!   so every test drives a recorded transport and never opens a socket.
//! - **Prompt builders** ([`prompts`]): pure functions turning a diff (or
//!   a set of commits) into the user message of an action, truncated to a
//!   budget.
//! - **Answer parsers** ([`parse`]): turning the model's text into a
//!   commit message, a stash name, a PR title/description, or a validated
//!   commit-composer plan.
//!
//! The API key is never read from `config.toml`; the caller loads it from
//! `gitpoulpi-secrets` and hands it to [`client_for`].

mod agent;
#[cfg(feature = "terminal-agent")]
pub mod agent_terminal;
mod api;
pub mod assist;
pub mod hosting_reply;
pub mod limits;
pub mod parse;
pub mod presets;
pub mod privacy;
pub mod prompts;
pub mod review;
pub mod review_context;
mod rules_answer;
pub mod vars;
pub mod worktree_name;

use std::sync::Arc;

pub use agent::{expand_env_refs, resolve_command, LocalAgentClient};
pub use api::{AnthropicClient, OpenAiClient};
use gitpoulpi_config::{AiProfile, AiProviderKind};
use gitpoulpi_hosting::{HttpTransport, TrustGate};
pub use hosting_reply::repo_context::{ContextBlock, ContextSummary};
pub use limits::{Cut, LimitAnswer, LimitGate, LimitPolicy, Limits, Part};
pub use parse::{CommitMessage, PlannedCommit, PlannedSplit, PullRequestText};
pub use privacy::{Prepared, Privacy, PrivacyClient, PrivacySummary, SendGate};
pub use rules_answer::{RuleProblem, RulesAnswer, RulesSettings};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("the AI profile has no API key (add one in Settings > AI > Profiles)")]
    MissingKey,
    #[error("the AI profile is incomplete: {0}")]
    BadProfile(String),
    #[error("the AI service rejected the API key ({0})")]
    Unauthorized(String),
    #[error("the AI service answered HTTP {status}: {detail}")]
    Api { status: u16, detail: String },
    #[error("could not reach the AI service: {0}")]
    Http(String),
    #[error("the AI answer could not be understood: {0}")]
    Decode(String),
    /// The security check of the profile's address refused: nothing was sent.
    #[error("{0}")]
    Trust(gitpoulpi_hosting::TrustProblem),
    /// An input over a limit the user can change (`ai.limits.*`): callers
    /// warn and offer "send anyway" instead of reporting a failure.
    #[error(
        "this {what} is too large to send to the AI ({bytes} bytes; the limit is {limit}). \
         Raise ai.limits.conflict_bytes in config.toml or Settings > AI > Limits"
    )]
    TooLarge {
        what: &'static str,
        bytes: usize,
        limit: usize,
    },
    #[error("the AI answered nothing")]
    Empty,
    #[error("the local agent failed: {0}")]
    Agent(String),
    /// A terminal-mode agent stopped at a first-run dialog (folder trust,
    /// login, theme...) that GitPoulpi never answers for the user.
    #[error(
        "Use \"Open the agent\" in Settings > AI, answer {agent}'s first-run question once \
         (\"{detail}\" in {folder}), then try again"
    )]
    AgentSetup {
        agent: String,
        folder: String,
        detail: String,
    },
    #[error("the AI did not answer within {0} seconds")]
    Timeout(u64),
    #[error("{0}")]
    Invalid(String),
    /// `--print-sent`: the request was prepared and shown, not sent.
    #[error("nothing was sent (print-only)")]
    PrintOnly,
    /// The user looked at what would be sent and canceled.
    #[error("canceled: nothing was sent")]
    Declined,
}

/// How long one completion may take before it is given up on.
pub const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// Anything that turns a system prompt and a user message into text.
#[async_trait::async_trait]
pub trait AiClient: Send + Sync {
    async fn complete(&self, system: &str, user: &str) -> Result<String>;

    /// [`AiClient::complete`] with a block of repository context that stays
    /// the same across replies. The default puts it after the system prompt
    /// (the stable-prefix order OpenAI-compatible servers cache by themselves);
    /// the Anthropic client sends it as its own cacheable system block and
    /// reads the cache counters back.
    ///
    /// With a `history` (the earlier turns of the same exchange) this default,
    /// for a client with no message list of its own (a local agent reads one
    /// document), writes them as a transcript in front of the new message
    /// ([`transcript`]), so the agent knows what a resumed session would.
    async fn complete_request(&self, request: &Request<'_>) -> Result<Completion> {
        Ok(Completion {
            text: self
                .complete(
                    &request.system_text(),
                    &transcript(request.history, request.user),
                )
                .await?,
            usage: Usage::default(),
        })
    }
}

/// One earlier turn of an exchange: what was sent and the answer that came
/// back (the message list of a chat API is these, in order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Turn {
    pub user: String,
    pub assistant: String,
}

/// One request to a model: the instructions, the repository context (when
/// the user let the AI read the repository), the earlier turns of the
/// exchange and the user message.
///
/// The order is the cache order: instructions, repository context, then
/// `history` (its first turn holds the conversation, the same in every
/// request of an exchange), then `user`. Anything that changes from one
/// request to the next comes last.
#[derive(Debug, Clone, Copy)]
pub struct Request<'a> {
    pub system: &'a str,
    pub context: Option<&'a ContextBlock>,
    /// Earlier turns, oldest first; empty for a first request.
    pub history: &'a [Turn],
    pub user: &'a str,
}

impl Request<'_> {
    /// The instructions with the repository context after them, as one text.
    pub fn system_text(&self) -> String {
        match self.context {
            Some(context) if !context.text.is_empty() => {
                format!("{}\n\n{}", self.system, context.text)
            }
            _ => self.system.to_string(),
        }
    }
}

/// `history` and `user` as one document, for a client that takes a single
/// message: the first turn as it was, then each answer and the next
/// instruction under a plain label. A stable prefix: a later request only
/// appends.
pub fn transcript(history: &[Turn], user: &str) -> String {
    let Some((first, rest)) = history.split_first() else {
        return user.to_string();
    };
    let mut out = first.user.clone();
    out.push_str(&format!(
        "\n\nYour draft so far:\n{}",
        first.assistant.trim()
    ));
    for turn in rest {
        out.push_str(&format!(
            "\n\nThe user then asked:\n{}\n\nYour draft:\n{}",
            turn.user.trim(),
            turn.assistant.trim()
        ));
    }
    out.push_str(&format!("\n\nThe user now asks:\n{}", user.trim()));
    out
}

/// An answer and what the provider said about its cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    pub text: String,
    pub usage: Usage,
}

/// The provider's prompt-cache counters (Anthropic's `cache_read_input_tokens`
/// and `cache_creation_input_tokens`); zero when it reports none.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Usage {
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
}

/// Builds the client a profile describes over `transport`.
///
/// `api_key` is required for every kind but `local_agent`. For the kinds that
/// send the key over HTTP, `gate` binds it to the origin it was stored for:
/// another `base_url` is refused before anything is sent.
pub fn client_for(
    profile: &AiProfile,
    api_key: Option<String>,
    transport: Arc<dyn HttpTransport>,
    gate: &TrustGate,
) -> Result<Box<dyn AiClient>> {
    client_for_in(profile, api_key, transport, None, gate, &Privacy::default())
}

/// The transport an API-key profile may use: `transport`, held to the
/// origin of the profile's base URL once the gate admits it.
fn guarded(
    profile: &AiProfile,
    has_key: bool,
    transport: Arc<dyn HttpTransport>,
    gate: &TrustGate,
) -> Result<Arc<dyn HttpTransport>> {
    let base = profile.effective_base_url();
    let origins = gate
        .admit_ai(&profile.id, base, has_key)
        .map_err(|e| match e {
            gitpoulpi_hosting::Error::Trust(p) => Error::Trust(p),
            other => Error::BadProfile(other.to_string()),
        })?;
    let record = gate.ai_record_of(&profile.id);
    Ok(Arc::new(gitpoulpi_hosting::trust::GuardedTransport::new(
        transport,
        gitpoulpi_hosting::TrustSubject::Ai(profile.id.clone()),
        record
            .as_ref()
            .map(|r| r.origin.clone())
            .unwrap_or_default(),
        origins,
        record.and_then(|r| r.tls),
    )))
}

/// [`client_for`] for an action on the repository at `repo_root` (the
/// working folder of a local agent set to `repository`). Every client it
/// returns is wrapped in [`PrivacyClient`]: each request is cleaned by
/// `privacy` ([`privacy::prepare`]) before it is sent.
pub fn client_for_in(
    profile: &AiProfile,
    api_key: Option<String>,
    transport: Arc<dyn HttpTransport>,
    repo_root: Option<&std::path::Path>,
    gate: &TrustGate,
    privacy: &Privacy,
) -> Result<Box<dyn AiClient>> {
    client_for_in_with(profile, api_key, transport, repo_root, gate, privacy, false)
}

/// [`client_for_in`], with `scrub_env`: a local agent starts without the
/// hosting credentials of this process's environment ([`agent::credential_env`]),
/// so an agent that reads the repository cannot publish or push with them.
pub fn client_for_in_with(
    profile: &AiProfile,
    api_key: Option<String>,
    transport: Arc<dyn HttpTransport>,
    repo_root: Option<&std::path::Path>,
    gate: &TrustGate,
    privacy: &Privacy,
    scrub_env: bool,
) -> Result<Box<dyn AiClient>> {
    // Nothing is sent: no key, no trust decision and no agent are needed.
    if privacy.print_only {
        return Ok(Box::new(PrivacyClient::print_only(privacy.clone())));
    }
    let inner = raw_client_for(profile, api_key, transport, repo_root, gate, scrub_env)?;
    Ok(Box::new(PrivacyClient::new(inner, privacy.clone())))
}

fn raw_client_for(
    profile: &AiProfile,
    api_key: Option<String>,
    transport: Arc<dyn HttpTransport>,
    repo_root: Option<&std::path::Path>,
    gate: &TrustGate,
    scrub_env: bool,
) -> Result<Box<dyn AiClient>> {
    let key = || {
        api_key
            .clone()
            .filter(|k| !k.trim().is_empty())
            .ok_or(Error::MissingKey)
    };
    let has_key = api_key.as_deref().is_some_and(|k| !k.trim().is_empty());
    let transport = match profile.kind {
        AiProviderKind::LocalAgent => transport,
        _ => guarded(profile, has_key, transport, gate)?,
    };
    Ok(match profile.kind {
        AiProviderKind::OpenAi | AiProviderKind::OpenRouter | AiProviderKind::OpenAiCompatible => {
            Box::new(OpenAiClient::new(
                transport,
                profile.effective_base_url(),
                profile.effective_model(),
                // A local OpenAI-compatible server (Ollama) needs no key.
                if profile.kind == AiProviderKind::OpenAiCompatible {
                    api_key.filter(|k| !k.trim().is_empty())
                } else {
                    Some(key()?)
                },
            ))
        }
        AiProviderKind::Anthropic => Box::new(AnthropicClient::new(
            transport,
            profile.effective_base_url(),
            profile.effective_model(),
            key()?,
        )),
        AiProviderKind::LocalAgent => local_agent_client(profile, repo_root, scrub_env)?,
    })
}

/// The client of a local agent profile: headless, or the virtual terminal.
fn local_agent_client(
    profile: &AiProfile,
    repo_root: Option<&std::path::Path>,
    scrub_env: bool,
) -> Result<Box<dyn AiClient>> {
    use gitpoulpi_config::AgentRunMode;
    match profile.run_mode {
        AgentRunMode::Headless => Ok(Box::new(
            LocalAgentClient::from_profile(profile)?
                .in_workdir(agent_workdir(profile, repo_root))
                .scrubbing_env(scrub_env),
        )),
        #[cfg(feature = "terminal-agent")]
        AgentRunMode::Terminal => Ok(Box::new(agent_terminal::TerminalAgentClient::from_profile(
            profile, repo_root,
        )?)),
        #[cfg(not(feature = "terminal-agent"))]
        AgentRunMode::Terminal => Err(Error::BadProfile(
            "this build has no virtual terminal support".into(),
        )),
    }
}

#[cfg(feature = "terminal-agent")]
fn agent_workdir(
    profile: &AiProfile,
    repo_root: Option<&std::path::Path>,
) -> Option<std::path::PathBuf> {
    agent_terminal::workdir_for(profile, repo_root)
}

#[cfg(not(feature = "terminal-agent"))]
fn agent_workdir(
    profile: &AiProfile,
    repo_root: Option<&std::path::Path>,
) -> Option<std::path::PathBuf> {
    // A headless agent runs where the process is, unless it is set to work in
    // the repository (the choice "Let the AI read the repository" does that
    // and must not depend on the build).
    (profile.workdir == gitpoulpi_config::AgentWorkdir::Repository)
        .then(|| repo_root.filter(|p| !p.as_os_str().is_empty()))
        .flatten()
        .map(std::path::Path::to_path_buf)
}

/// [`client_for`] over the real network transport, with the AI timeout.
#[cfg(feature = "network")]
pub fn client_for_network(
    profile: &AiProfile,
    api_key: Option<String>,
) -> Result<Box<dyn AiClient>> {
    client_for_network_in(profile, api_key, None, &Privacy::default())
}

/// [`client_for_network`] for an action on the repository at `repo_root`.
#[cfg(feature = "network")]
pub fn client_for_network_in(
    profile: &AiProfile,
    api_key: Option<String>,
    repo_root: Option<&std::path::Path>,
    privacy: &Privacy,
) -> Result<Box<dyn AiClient>> {
    client_for_network_with(profile, api_key, repo_root, privacy, false)
}

/// [`client_for_network_in`], with the hosting credentials taken out of a
/// local agent's environment when `scrub_env` (see [`client_for_in_with`]).
#[cfg(feature = "network")]
pub fn client_for_network_with(
    profile: &AiProfile,
    api_key: Option<String>,
    repo_root: Option<&std::path::Path>,
    privacy: &Privacy,
    scrub_env: bool,
) -> Result<Box<dyn AiClient>> {
    let transport = gitpoulpi_hosting::ReqwestTransport::with_timeout(
        std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS),
    )
    .map_err(|e| Error::Http(e.to_string()))?;
    client_for_in_with(
        profile,
        api_key,
        Arc::new(transport),
        repo_root,
        &TrustGate::ambient(),
        privacy,
        scrub_env,
    )
}

/// Checks that a profile can answer: a one-word round trip.
pub async fn test_connection(client: &dyn AiClient) -> Result<()> {
    let answer = client
        .complete("You answer with the single word OK.", "Reply now.")
        .await?;
    if answer.trim().is_empty() {
        Err(Error::Empty)
    } else {
        Ok(())
    }
}
