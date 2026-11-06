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
pub mod parse;
pub mod presets;
pub mod privacy;
pub mod prompts;
pub mod review;
pub mod review_context;
mod rules_answer;
pub mod vars;

use std::sync::Arc;

pub use agent::{expand_env_refs, resolve_command, LocalAgentClient};
pub use api::{AnthropicClient, OpenAiClient};
use gitpoulpi_config::{AiProfile, AiProviderKind};
use gitpoulpi_hosting::{HttpTransport, TrustGate};
pub use privacy::{Prepared, Privacy, PrivacyClient, PrivacySummary, SendGate};
pub use parse::{CommitMessage, PlannedCommit, PlannedSplit, PullRequestText};
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
    // Nothing is sent: no key, no trust decision and no agent are needed.
    if privacy.print_only {
        return Ok(Box::new(PrivacyClient::print_only(privacy.clone())));
    }
    let inner = raw_client_for(profile, api_key, transport, repo_root, gate)?;
    Ok(Box::new(PrivacyClient::new(inner, privacy.clone())))
}

fn raw_client_for(
    profile: &AiProfile,
    api_key: Option<String>,
    transport: Arc<dyn HttpTransport>,
    repo_root: Option<&std::path::Path>,
    gate: &TrustGate,
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
        AiProviderKind::LocalAgent => local_agent_client(profile, repo_root)?,
    })
}

/// The client of a local agent profile: headless, or the virtual terminal.
fn local_agent_client(
    profile: &AiProfile,
    repo_root: Option<&std::path::Path>,
) -> Result<Box<dyn AiClient>> {
    use gitpoulpi_config::AgentRunMode;
    match profile.run_mode {
        AgentRunMode::Headless => Ok(Box::new(
            LocalAgentClient::from_profile(profile)?.in_workdir(agent_workdir(profile, repo_root)),
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
    _profile: &AiProfile,
    _repo_root: Option<&std::path::Path>,
) -> Option<std::path::PathBuf> {
    None
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
    let transport = gitpoulpi_hosting::ReqwestTransport::with_timeout(
        std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS),
    )
    .map_err(|e| Error::Http(e.to_string()))?;
    client_for_in(
        profile,
        api_key,
        Arc::new(transport),
        repo_root,
        &TrustGate::ambient(),
        privacy,
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
