//! Letting the AI read the repository for one reply: the choice between the
//! two routes and the caches around them.
//!
//! Called only when "Let the AI read the repository" is ticked for the use.
//!
//! - A **local agent** profile reads the repository itself, in its working
//!   folder, with the read-only tools of its harness (and no hosting
//!   credentials in its environment); its session is kept between replies
//!   when `reuse_agent_session` is on and the harness supports it.
//! - Any **HTTP** profile gets a context GitPoulpi builds through a
//!   [`RepoReader`], from the cache when it can.
//!
//! The AI still only drafts: nothing here reaches a host.

use std::path::Path;
use std::sync::Arc;

use gitpoulpi_config::AiProfile;

use super::cache::{ContextCache, SessionKey, SessionStore};
use super::plan::plan_context;
use super::repo_context::{build, BuildInput, ContextKey, ContextLimits, RepoContext, RepoReader};
use super::{AgentSession, ReplyRequest, RepoAccess};
use crate::assist::{Job, NO_READ_ONLY_MODE};
use crate::presets::{harness_of, SessionUse};
use crate::{Error, Result};

/// Why the AI cannot read the repository with this profile, for the
/// tooltip of the disabled choice; `None`: it can.
pub fn blocked_reason(profile: &AiProfile) -> Option<&'static str> {
    use gitpoulpi_config::{AgentRunMode, AiProviderKind};
    if profile.kind != AiProviderKind::LocalAgent {
        return None;
    }
    let trusted = profile.run_mode == AgentRunMode::Headless
        && harness_of(profile).is_some_and(|h| h.read_only_tools().is_some());
    (!trusted).then_some(NO_READ_ONLY_MODE)
}

/// What [`read_repository`] needs besides the job and the request.
pub struct RepoSetup<'a> {
    pub reader: &'a dyn RepoReader,
    pub root: &'a Path,
    pub cache: &'a mut ContextCache,
    pub sessions: &'a mut SessionStore,
    /// `ai.hosting_reply.reuse_agent_session`.
    pub reuse_session: bool,
    /// "Refresh context": forget what is kept and build again.
    pub refresh: bool,
    pub limits: ContextLimits,
    pub privacy: crate::privacy::Privacy,
}

/// What a run leaves to settle afterwards (see [`settle`]).
#[derive(Debug, Clone, Default)]
pub struct Settle {
    session: Option<SessionKey>,
}

/// Prepares `request` to read the repository: returns the job to run (a
/// local agent's, set up read-only) and sets `request.repo`.
pub async fn read_repository(
    job: Job,
    request: &mut ReplyRequest,
    setup: &mut RepoSetup<'_>,
) -> Result<(Job, Settle)> {
    let head = setup.reader.head().await?;
    let head_hex = head.to_hex();
    if setup.refresh {
        setup.cache.refresh();
        setup.sessions.drop_repo(setup.root);
    } else {
        setup.cache.head_changed(head);
        setup.sessions.keep_head(setup.root, &head_hex);
    }

    if job.profile.kind == gitpoulpi_config::AiProviderKind::LocalAgent {
        let harness = harness_of(&job.profile)
            .filter(|h| h.read_only_tools().is_some())
            .ok_or_else(|| Error::Invalid(NO_READ_ONLY_MODE.into()))?;
        let key = SessionKey {
            repo_root: setup.root.to_path_buf(),
            head: head_hex,
            action: job.action,
        };
        let (session, note) = if setup.reuse_session && harness.supports_sessions() {
            let used = setup.sessions.next_use(&key);
            let note = match used {
                SessionUse::Start(_) => AgentSession::Started,
                SessionUse::Resume(_) => AgentSession::Continued,
            };
            (Some(used), Some(note))
        } else {
            (None, None)
        };
        let job = job.with_read_only_agent(setup.root, session.as_ref())?;
        let tools = harness
            .read_only_tools()
            .map(|m| m.tools.to_string())
            .unwrap_or_default();
        request.repo = RepoAccess::AgentReadOnly {
            root: setup.root.to_path_buf(),
            head,
            tools,
            session: note,
        };
        return Ok((
            job,
            Settle {
                session: session.is_some().then_some(key),
            },
        ));
    }

    let tree: Vec<String> = setup.reader.list_tree(head).await?;
    let selection = plan_context(request, &tree, &setup.limits);
    let key = ContextKey {
        repo_root: setup.root.to_path_buf(),
        head,
        selection_hash: selection.hash(),
        limits_hash: setup.limits.hash(),
        privacy_hash: setup.privacy.cache_hash(),
    };
    let context: Arc<RepoContext> = match setup.cache.get(&key) {
        Some(found) => {
            // Everything in it was read before.
            let mut again = (*found).clone();
            for part in &mut again.parts {
                part.reused = true;
            }
            Arc::new(again)
        }
        None => {
            let now = setup.cache.now();
            let built = build(
                &BuildInput {
                    reader: setup.reader,
                    root: setup.root,
                    head,
                    tree: &tree,
                    selection: &selection,
                    limits: setup.limits,
                    privacy: &setup.privacy,
                    now,
                },
                setup.cache.parts(),
            )
            .await?;
            setup.cache.put(built)
        }
    };
    request.repo = RepoAccess::Built(context);
    Ok((job, Settle::default()))
}

/// After the run: an agent session that ran is kept for the next reply, one
/// whose run failed is forgotten.
pub fn settle(sessions: &mut SessionStore, settle: &Settle, ok: bool) {
    let Some(key) = &settle.session else { return };
    if ok {
        sessions.mark_started(key);
    } else {
        sessions.forget_unstarted(key);
    }
}

/// [`super::draft`] for a request that reads the repository: sets it up
/// ([`read_repository`]), drafts, and settles the agent session.
#[cfg(feature = "network")]
pub async fn draft_reading(
    job: Job,
    mut request: ReplyRequest,
    setup: &mut RepoSetup<'_>,
) -> Result<super::Draft> {
    let (job, pending) = read_repository(job, &mut request, setup).await?;
    let result = super::draft(&job, &request).await;
    settle(setup.sessions, &pending, result.is_ok());
    result
}

/// [`draft_reading`] over a caller-supplied client (tests).
pub async fn draft_reading_with(
    job: Job,
    client: &dyn crate::AiClient,
    mut request: ReplyRequest,
    setup: &mut RepoSetup<'_>,
) -> Result<super::Draft> {
    let (job, pending) = read_repository(job, &mut request, setup).await?;
    let result = super::draft_with(&job, client, &request).await;
    settle(setup.sessions, &pending, result.is_ok());
    result
}
