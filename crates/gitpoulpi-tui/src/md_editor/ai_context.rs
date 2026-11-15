//! The repository part of "Write with AI": whether the choice "Let the AI read
//! the repository" is offered, the worker's reader over its own daemon
//! connection, what is kept between two drafts, and the line that says what
//! the last draft read.
//!
//! **Off by default** (`ai.hosting_reply.read_repository`, and the box is
//! unticked every time unless the user turned that setting on). The AI only
//! drafts: nothing here reaches a host, and a local agent runs with read-only
//! tools (see `gitpoulpi_ai::hosting_reply::repo_access`).
//!
//! What is kept (per repository, for the life of the window): the built
//! contexts (LRU of 8, time to live `context_ttl_minutes`), the parts of
//! them, and a local agent's session. A new HEAD drops them (every draft
//! compares it); "Refresh context" ([`forget`]) drops them at once.

use gitpoulpi_i18n::msg::md_editor as msg;

use super::ai::AiScope;

/// Why the choice is not offered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Why {
    /// The repository is not cloned here: there is nothing to read.
    NoClone,
    /// The chosen local agent has no read-only mode GitPoulpi trusts.
    NoReadOnly,
    /// This text is written from something else (a pull request description
    /// is written from its diff).
    NotHere,
}

impl Why {
    /// The few words in parentheses after the disabled box.
    pub(crate) fn short(self) -> &'static str {
        match self {
            Why::NoClone => msg::AI_REPO_NO_CLONE.get(),
            Why::NoReadOnly => msg::AI_REPO_NO_READONLY.get(),
            Why::NotHere => msg::AI_REPO_NOT_HERE.get(),
        }
    }

    /// The sentence shown when the disabled box is activated.
    pub(crate) fn full(self) -> &'static str {
        match self {
            Why::NoClone => msg::AI_REPO_NO_CLONE_WHY.get(),
            Why::NoReadOnly => msg::AI_REPO_NO_READONLY_WHY.get(),
            Why::NotHere => msg::AI_REPO_NOT_HERE_WHY.get(),
        }
    }
}

/// Whether the box can be ticked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Choice {
    Offered,
    Disabled(Why),
}

/// "12k" for 12000 tokens.
fn tokens_text(n: u64) -> String {
    if n >= 1000 {
        format!("{}k", n / 1000)
    } else {
        n.to_string()
    }
}

/// "31.2 KB".
fn size_text(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    }
}

#[cfg(feature = "ai")]
pub(crate) use real::{default_on, draft_reading, forget, line, offered};

#[cfg(not(feature = "ai"))]
pub(crate) use stub::{default_on, forget, offered};

#[cfg(feature = "ai")]
mod real {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    use gitpoulpi_ai::assist::Job;
    use gitpoulpi_ai::hosting_reply::cache::{ContextCache, SessionStore};
    use gitpoulpi_ai::hosting_reply::repo_access::{blocked_reason, RepoSetup};
    use gitpoulpi_ai::hosting_reply::repo_context::{ContextLimits, RepoReader};
    use gitpoulpi_ai::hosting_reply::{ContextReport, Draft, ReplyRequest};
    use gitpoulpi_config::{AiAction, AiConfig};
    use gitpoulpi_core::domain::{
        BlobBody, BlobBytes, BlobSource, DiffTarget, FileDiff, GrepHit, Oid, OperationKind,
        RepoRef, MAX_GREP_FILE_BYTES,
    };
    use gitpoulpi_ipc::{Client, Request};
    use serde_json::json;

    use super::{msg, size_text, tokens_text, AiScope, Choice, Why};

    /// The caches of one repository.
    struct Kept {
        cache: ContextCache,
        sessions: SessionStore,
    }

    static KEPT: Mutex<Option<HashMap<PathBuf, Kept>>> = Mutex::new(None);

    fn take(root: &Path, ttl_minutes: u32) -> Kept {
        let mut slot = KEPT.lock().unwrap_or_else(|e| e.into_inner());
        slot.get_or_insert_with(HashMap::new)
            .remove(root)
            .unwrap_or_else(|| Kept {
                cache: ContextCache::new(ttl_minutes),
                sessions: SessionStore::new(),
            })
    }

    fn put_back(root: &Path, kept: Kept) {
        let mut slot = KEPT.lock().unwrap_or_else(|e| e.into_inner());
        slot.get_or_insert_with(HashMap::new)
            .insert(root.to_path_buf(), kept);
    }

    /// "Refresh context": forget what was built and the agent's session for
    /// this repository; the next draft reads it again.
    pub(crate) fn forget(root: &Path) {
        let mut slot = KEPT.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(map) = slot.as_mut() {
            map.remove(root);
        }
    }

    fn repo_settings(config: &AiConfig, root: &Path) -> gitpoulpi_config::AiHostingReply {
        let state_dir = gitpoulpi_config::Settings::resolve_ambient()
            .map(|s| s.config().session.per_repo_state_dir.clone())
            .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
        let path = gitpoulpi_session::RepoState::path_for(root, &state_dir);
        let prefs = gitpoulpi_session::RepoState::load(&path)
            .map(|s| s.preferences.ai_hosting_reply)
            .unwrap_or_default();
        config.hosting_reply.with_repo(&prefs)
    }

    /// The default of the box: `ai.hosting_reply.read_repository`, repository
    /// override applied. Off unless the user turned it on.
    pub(crate) fn default_on(scope: &AiScope) -> bool {
        #[cfg(test)]
        {
            let _ = scope;
            false
        }
        #[cfg(not(test))]
        {
            let Some(root) = scope.root.as_deref() else {
                return false;
            };
            let Ok(settings) = gitpoulpi_config::Settings::resolve_ambient() else {
                return false;
            };
            repo_settings(&settings.config().ai, root).read_repository
        }
    }

    /// Whether the box is offered for a text of this scope. `allows` is false
    /// for a text written from something else (a pull request description).
    pub(crate) fn offered(scope: &AiScope, allows: bool) -> Choice {
        if !allows {
            return Choice::Disabled(Why::NotHere);
        }
        if scope.root.is_none() {
            return Choice::Disabled(Why::NoClone);
        }
        #[cfg(not(test))]
        if let Ok(settings) = gitpoulpi_config::Settings::resolve_ambient() {
            let config = &settings.config().ai;
            if let Ok(resolved) = config.resolve(AiAction::HostingReply, None) {
                if blocked_reason(&resolved.profile).is_some() {
                    return Choice::Disabled(Why::NoReadOnly);
                }
            }
        }
        #[cfg(test)]
        let _ = (AiAction::HostingReply, blocked_reason);
        Choice::Offered
    }

    /// The reader the worker uses: its own connection to the daemon, read
    /// operations only.
    struct DaemonReader {
        client: tokio::sync::Mutex<(Client, u64)>,
        session: String,
        repo: RepoRef,
    }

    impl DaemonReader {
        async fn connect(session: &str, root: &Path) -> Result<Self, String> {
            let endpoint = crate::daemon_link::ensure_daemon(session).await?;
            let client = Client::connect(&endpoint)
                .await
                .map_err(gitpoulpi_i18n::msg::ai_ui::daemon_unreachable)?;
            Ok(Self {
                client: tokio::sync::Mutex::new((client, 0)),
                session: session.to_string(),
                repo: RepoRef::from_root(root),
            })
        }

        async fn call<T: serde::de::DeserializeOwned>(
            &self,
            command: OperationKind,
            args: serde_json::Value,
        ) -> Result<T, String> {
            let mut guard = self.client.lock().await;
            guard.1 += 1;
            let request = Request {
                id: guard.1,
                session: self.session.clone(),
                repo: Some(self.repo.id.clone()),
                command,
                args,
            };
            let response = guard
                .0
                .call(&request)
                .await
                .map_err(gitpoulpi_i18n::msg::ai_ui::daemon_lost)?;
            match response.result {
                Ok(value) => serde_json::from_value(value).map_err(|e| {
                    gitpoulpi_i18n::msg::ai_ui::daemon_malformed(format!("{command:?}"), e)
                }),
                Err(e) => Err(e.message),
            }
        }
    }

    fn failed(what: &str, why: String) -> gitpoulpi_ai::Error {
        let message = format!("could not read the repository ({what}): {why}"); // i18n-english
        gitpoulpi_ai::Error::Invalid(message)
    }

    #[async_trait::async_trait]
    impl RepoReader for DaemonReader {
        async fn head(&self) -> gitpoulpi_ai::Result<Oid> {
            self.call(OperationKind::Resolve, json!({ "rev": "HEAD" }))
                .await
                .map_err(|e| failed("HEAD", e))
        }

        async fn list_tree(&self, at: Oid) -> gitpoulpi_ai::Result<Vec<String>> {
            let paths: Vec<PathBuf> = self
                .call(OperationKind::ListTree, json!({ "at": at }))
                .await
                .map_err(|e| failed("the file list", e))?; // i18n-english
            Ok(paths
                .into_iter()
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .collect())
        }

        async fn read(&self, path: &str, at: Oid) -> gitpoulpi_ai::Result<Option<Vec<u8>>> {
            let blob: Result<BlobBytes, String> = self
                .call(
                    OperationKind::ReadBlobBytes,
                    json!({
                        "path": path,
                        "source": BlobSource::Commit(at),
                        "max": MAX_GREP_FILE_BYTES,
                    }),
                )
                .await;
            // A path missing at this commit is not worth failing the draft for.
            let Ok(blob) = blob else { return Ok(None) };
            Ok(match blob.body {
                BlobBody::Bytes(bytes) if !bytes[..bytes.len().min(8000)].contains(&0) => {
                    Some(bytes)
                }
                _ => None,
            })
        }

        async fn grep(
            &self,
            terms: &[String],
            at: Oid,
            max_hits: usize,
        ) -> gitpoulpi_ai::Result<Vec<GrepHit>> {
            self.call(
                OperationKind::GrepTree,
                json!({ "at": at, "terms": terms, "max_hits": max_hits }),
            )
            .await
            .map_err(|e| failed("the search", e)) // i18n-english
        }

        async fn diff(&self, base: &str, head: &str) -> gitpoulpi_ai::Result<Option<String>> {
            let resolve = |rev: String| async move {
                self.call::<Oid>(OperationKind::Resolve, json!({ "rev": rev }))
                    .await
                    .ok()
            };
            let (Some(base), Some(head)) = (resolve(base.into()).await, resolve(head.into()).await)
            else {
                return Ok(None);
            };
            let from: Oid = self
                .call(OperationKind::MergeBase, json!({ "a": base, "b": head }))
                .await
                .unwrap_or(base);
            let files: Vec<FileDiff> = self
                .call(
                    OperationKind::Diff,
                    json!({ "target": DiffTarget::CommitRange(from, head) }),
                )
                .await
                .map_err(|e| failed("the pull request's diff", e))?; // i18n-english
            Ok(Some(gitpoulpi_ai::assist::render_diff(&files)))
        }
    }

    /// Drafts `request` with the repository read, on the worker's own
    /// connection and with this window's caches.
    pub(crate) async fn draft_reading(
        job: Job,
        request: ReplyRequest,
        scope: &AiScope,
        config: &AiConfig,
    ) -> Result<Draft, String> {
        let root = scope
            .root
            .clone()
            .ok_or_else(|| msg::AI_REPO_NO_CLONE.get().to_string())?;
        let reply = repo_settings(config, &root);
        let reader = DaemonReader::connect(&scope.session, &root).await?;
        let mut kept = take(&root, reply.context_ttl_minutes_clamped());
        let mut setup = RepoSetup {
            reader: &reader,
            root: &root,
            cache: &mut kept.cache,
            sessions: &mut kept.sessions,
            reuse_session: reply.reuse_agent_session,
            refresh: false,
            limits: ContextLimits::from_config(&config.limits),
            privacy: job.privacy.clone(),
        };
        let result =
            gitpoulpi_ai::hosting_reply::repo_access::draft_reading(job, request, &mut setup)
                .await
                .map_err(|e| e.to_string());
        put_back(&root, kept);
        result
    }

    /// The line that says what a draft read: "Context reused (4 of 5 parts,
    /// HEAD a1b2c3d)", "Provider cache: 12k tokens reused", "Agent session
    /// continued". `None` when the repository was not read.
    pub(crate) fn line(report: &ContextReport) -> Option<String> {
        let repo = report.repo.as_ref()?;
        let mut out: Vec<String> = Vec::new();
        if repo.route == "agent" {
            out.push(msg::ai_ctx_agent(&repo.head));
            match repo.agent_session.as_deref() {
                Some("continued") => out.push(msg::AI_CTX_SESSION_CONTINUED.get().to_string()),
                Some("started") => out.push(msg::AI_CTX_SESSION_STARTED.get().to_string()),
                _ => {}
            }
        } else {
            out.push(if repo.context_reused {
                msg::ai_ctx_reused(repo.reused_parts, repo.parts.len(), &repo.head)
            } else {
                msg::ai_ctx_built(repo.parts.len(), size_text(repo.bytes), &repo.head)
            });
            if repo.provider_cache_read_tokens > 0 {
                out.push(msg::ai_ctx_cache(tokens_text(
                    repo.provider_cache_read_tokens,
                )));
            }
        }
        Some(out.join(". "))
    }
}

#[cfg(not(feature = "ai"))]
mod stub {
    use std::path::Path;

    use super::{AiScope, Choice, Why};

    pub(crate) fn default_on(_: &AiScope) -> bool {
        false
    }

    pub(crate) fn offered(_: &AiScope, _: bool) -> Choice {
        Choice::Disabled(Why::NotHere)
    }

    pub(crate) fn forget(_: &Path) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_and_tokens_read_like_the_dialog_says_them() {
        assert_eq!(size_text(512), "512 B");
        assert_eq!(size_text(31 * 1024 + 205), "31.2 KB");
        assert_eq!(tokens_text(999), "999");
        assert_eq!(tokens_text(12_345), "12k");
    }

    #[test]
    fn the_box_needs_a_clone_and_a_text_that_allows_it() {
        let none = AiScope::default();
        assert_eq!(offered(&none, true), Choice::Disabled(Why::NoClone));
        assert_eq!(offered(&none, false), Choice::Disabled(Why::NotHere));
        let cloned = AiScope {
            session: "s".into(),
            root: Some("/r".into()),
        };
        #[cfg(feature = "ai")]
        assert_eq!(offered(&cloned, true), Choice::Offered);
        assert_eq!(offered(&cloned, false), Choice::Disabled(Why::NotHere));
    }
}
