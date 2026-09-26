//! `gitpoulpi hosting` -- link.

use gitpoulpi_config::IntegrationCategory;
use gitpoulpi_core::domain::{OperationKind, Remote};
use gitpoulpi_hosting::{resolve_for_repo, Integration, ResolveRemote};
use serde_json::{json, Value};

use crate::cli::LinkKind;
use crate::client::Session;
use crate::repo_cmd::Emitted;

use super::*;

/// The project a repository's remotes resolve to, and the profile that
/// answers for it.
pub(super) struct RepoTarget {
    pub(super) integration: Integration,
    pub(super) integrations: Vec<Integration>,
    pub(super) project: String,
    pub(super) remote: String,
    pub(super) ambiguous: bool,
    /// Taken from the repository's saved link (`pulls_source` /
    /// `issues_source`), not resolved from a remote.
    pub(super) linked: bool,
    /// The link's provider-native narrowing of the issue list (a Jira JQL
    /// clause, an Azure WIQL condition, Trello list names).
    pub(super) filter: Option<String>,
}

/// `--id` beats the repository's pinned profile, which beats resolving by
/// remote host (see `gitpoulpi_hosting::resolve_for_repo`).
pub(super) fn resolve_target(
    remotes: &[Remote],
    integrations: Vec<Integration>,
    id: Option<&str>,
    pinned: Option<&str>,
) -> Result<RepoTarget, String> {
    if integrations.is_empty() {
        return Err(no_integrations_message());
    }
    if let Some(id) = id {
        pick(&integrations, Some(id))?;
    }
    let resolve_remotes: Vec<ResolveRemote> = remotes
        .iter()
        .map(|r| ResolveRemote::new(r.name.clone(), r.fetch_url.clone()))
        .collect();
    let resolution =
        resolve_for_repo(&resolve_remotes, &integrations, id.or(pinned)).ok_or_else(|| {
            format!(
                "no configured integration matches this repository's remotes ({}); \
                 add one with `gitpoulpi hosting integrations add` or pin one with \
                 `gitpoulpi hosting use --id <id>`",
                if remotes.is_empty() {
                    "it has none".to_string()
                } else {
                    remotes
                        .iter()
                        .map(|r| r.fetch_url.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            )
        })?;
    let target = RepoTarget {
        integration: resolution.integration.clone(),
        project: resolution.project_path.clone(),
        remote: resolution.remote.clone(),
        ambiguous: resolution.ambiguous,
        linked: false,
        filter: None,
        integrations: Vec::new(),
    };
    Ok(RepoTarget {
        integrations,
        ..target
    })
}

/// Which of the repository's two per-repo lists a command reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Which {
    Pulls,
    Issues,
}

impl Which {
    fn noun(self) -> &'static str {
        match self {
            Which::Pulls => "pull/merge request",
            Which::Issues => "issue",
        }
    }

    pub(super) fn arg(self) -> &'static str {
        match self {
            Which::Pulls => "pulls",
            Which::Issues => "issues",
        }
    }

    /// Whether the profile's kind can serve this list at all, and if not
    /// why (Bitbucket retired its issues; Jira and Trello have no pull
    /// requests).
    pub(super) fn unsupported_by(self, integration: &Integration) -> Option<String> {
        let caps = integration.kind.capabilities();
        let label = integration.kind.label();
        match self {
            Which::Pulls if !caps.pull_requests => Some(format!(
                "{label} (`{}`) has no pull requests; only an issue list can link to it",
                integration.id
            )),
            Which::Issues if !caps.issues => Some(format!(
                "{label} (`{}`) has no issue tracker (Bitbucket retired its issues on \
                 2026-08-20); link a Jira project instead",
                integration.id
            )),
            _ => None,
        }
    }

    fn source(
        self,
        state: &gitpoulpi_session::RepoState,
    ) -> Option<&gitpoulpi_session::IssuesSource> {
        match self {
            Which::Pulls => state.pulls_source.as_ref(),
            Which::Issues => state.issues_source.as_ref(),
        }
    }
}

/// What a repository with nothing linked says: no fetch, and how to link.
pub(super) fn unlinked_message(which: Which) -> String {
    format!(
        "this repository has no {noun} source linked, so nothing is fetched (a repository is \
         never linked on its own; its remotes only suggest one). Link it with \
         `gitpoulpi hosting link {arg} --id <profile> [--project <owner/name>]` (or in the \
         TUI's sidebar), or pass --id to use that profile on the project this repository's \
         remote names.",
        noun = which.noun(),
        arg = which.arg()
    )
}

/// The target a saved link names: the profile by id, the provider's own
/// project. No remote is consulted - the link is the answer.
pub(super) fn resolve_linked(
    integrations: Vec<Integration>,
    source: &gitpoulpi_session::IssuesSource,
    which: Which,
) -> Result<RepoTarget, String> {
    let integration = integrations
        .iter()
        .find(|i| i.id == source.integration)
        .cloned()
        .ok_or_else(|| {
            format!(
                "this repository's {} source is linked to `{}`, which is not configured any \
                 more; relink it with `gitpoulpi hosting link {} --id <profile>` or unlink it \
                 with `--clear`",
                which.noun(),
                source.integration,
                which.arg()
            )
        })?;
    if !integration.enabled {
        return Err(format!(
            "this repository's {} source is linked to `{}`, which is disabled",
            which.noun(),
            integration.id
        ));
    }
    if let Some(why) = which.unsupported_by(&integration) {
        return Err(format!(
            "this repository's {} source is linked to `{}`: {why}",
            which.noun(),
            integration.id
        ));
    }
    if source.project.trim().is_empty() {
        return Err(format!(
            "this repository's {noun} source is linked to `{id}` but no repository is chosen \
             yet: `gitpoulpi hosting link {arg} --id {id} --project <owner/name>`",
            noun = which.noun(),
            id = integration.id,
            arg = which.arg()
        ));
    }
    Ok(RepoTarget {
        project: source.project.clone(),
        remote: String::new(),
        ambiguous: false,
        linked: true,
        filter: source.filter.clone().filter(|f| !f.trim().is_empty()),
        integration,
        integrations,
    })
}

/// The target a list command acts on. `--id` is the explicit one-off
/// override (that profile, on the project the remotes name). Otherwise the
/// repository's saved link answers, and an unlinked repository is an error
/// that fetches nothing - the TUI's rule. A profile pinned with `hosting
/// use` keeps resolving pull/merge requests by remote.
pub(super) fn target_from_state(
    which: Which,
    state: &gitpoulpi_session::RepoState,
    remotes: &[Remote],
    integrations: Vec<Integration>,
    id: Option<&str>,
) -> Result<RepoTarget, String> {
    // A local Markdown profile named with --id needs no remote: its folder
    // is the project.
    if let Some(local) = id
        .and_then(|id| integrations.iter().find(|i| i.id == id))
        .filter(|i| i.kind.capabilities().local_folder)
        .cloned()
    {
        let project = match which.source(state).filter(|s| s.integration == local.id) {
            Some(source) if !source.project.trim().is_empty() => source.project.clone(),
            _ => local.url.clone(),
        };
        return Ok(RepoTarget {
            project,
            remote: String::new(),
            ambiguous: false,
            linked: false,
            filter: None,
            integration: local,
            integrations,
        });
    }
    if id.is_some() {
        return resolve_target(
            remotes,
            integrations,
            id,
            state.hosting_integration.as_deref(),
        );
    }
    if let Some(source) = which.source(state) {
        return resolve_linked(integrations, source, which);
    }
    if which == Which::Pulls && state.hosting_integration.is_some() {
        return resolve_target(
            remotes,
            integrations,
            None,
            state.hosting_integration.as_deref(),
        );
    }
    Err(unlinked_message(which))
}

pub(super) fn load_state(ipc: &Session) -> Result<gitpoulpi_session::RepoState, String> {
    let root = ipc
        .repo_root()
        .ok_or_else(|| "not inside a git repository (pass --repo)".to_string())?;
    gitpoulpi_session::RepoState::load(&crate::commands::repo_state_path(root))
        .map_err(|e| e.to_string())
}

pub(super) async fn repo_target(
    env: &HostingEnv,
    ipc: &mut Session,
    id: Option<&str>,
    which: Which,
) -> Result<RepoTarget, String> {
    let state = load_state(ipc)?;
    // Unlinked: answer before touching the daemon or the host.
    let pinned_pulls = which == Which::Pulls && state.hosting_integration.is_some();
    if id.is_none() && which.source(&state).is_none() && !pinned_pulls {
        return Err(unlinked_message(which));
    }
    let remotes: Vec<Remote> = if id.is_some() || which.source(&state).is_none() {
        ipc.call(OperationKind::Remotes, json!({})).await?
    } else {
        Vec::new()
    };
    let mut target = target_from_state(which, &state, &remotes, env.integrations()?, id)?;
    // A local Markdown folder is relative to the repository.
    if target.integration.kind.capabilities().local_folder {
        if let Some(root) = ipc.repo_root() {
            target.project = gitpoulpi_hosting::resolve_local_folder(root, &target.project)
                .to_string_lossy()
                .into_owned();
        }
    }
    Ok(target)
}

/// `hosting link`: shows, sets or clears one list's source.
pub(super) async fn run_link(
    env: &HostingEnv,
    ipc: &mut Session,
    kind: LinkKind,
    id: Option<String>,
    project: Option<String>,
    filter: Option<String>,
    clear: bool,
) -> Result<Emitted, String> {
    let which = match kind {
        LinkKind::Pulls => Which::Pulls,
        LinkKind::Issues => Which::Issues,
    };
    let root = ipc
        .repo_root()
        .ok_or_else(|| "not inside a git repository (pass --repo)".to_string())?
        .to_path_buf();
    let path = crate::commands::repo_state_path(&root);
    let mut state = gitpoulpi_session::RepoState::load(&path).map_err(|e| e.to_string())?;
    let mut changed = false;
    if clear {
        changed = which.source(&state).is_some();
        match which {
            Which::Pulls => state.pulls_source = None,
            Which::Issues => state.issues_source = None,
        }
    } else if let Some(id) = id {
        let integrations = env.integrations()?;
        let integration = pick(&integrations, Some(&id))?;
        if !integration.enabled {
            return Err(format!("`{id}` is disabled"));
        }
        if let Some(why) = which.unsupported_by(integration) {
            return Err(why);
        }
        let filter = filter
            .map(|f| f.trim().to_string())
            .filter(|f| !f.is_empty());
        if filter.is_some() && which == Which::Pulls {
            return Err("--filter narrows the issue list only (`hosting link issues`)".into());
        }
        let project = match project {
            Some(p) => p,
            // A folder source points at its profile's folder unless told
            // otherwise.
            None if integration.kind.capabilities().local_folder => integration.url.clone(),
            None => {
                let remotes: Vec<Remote> = ipc.call(OperationKind::Remotes, json!({})).await?;
                let resolve_remotes: Vec<ResolveRemote> = remotes
                    .iter()
                    .map(|r| ResolveRemote::new(r.name.clone(), r.fetch_url.clone()))
                    .collect();
                resolve_for_repo(
                    &resolve_remotes,
                    std::slice::from_ref(integration),
                    Some(&id),
                )
                .map(|r| r.project_path)
                .filter(|p| !p.is_empty())
                .ok_or_else(|| {
                    format!(
                        "no remote of this repository names a project on `{id}`; pass --project"
                    )
                })?
            }
        };
        let source = gitpoulpi_session::IssuesSource {
            integration: id,
            project,
            filter,
        };
        match which {
            Which::Pulls => state.pulls_source = Some(source),
            Which::Issues => state.issues_source = Some(source),
        }
        changed = true;
    }
    if changed {
        state.save(&path).map_err(|e| e.to_string())?;
    }
    let source = which.source(&state);
    let human = match source {
        Some(s) => format!(
            "{} source: `{}` / {}{}",
            which.noun(),
            s.integration,
            if s.project.is_empty() {
                "(no repository chosen)"
            } else {
                &s.project
            },
            s.filter
                .as_deref()
                .map(|f| format!(" [{f}]"))
                .unwrap_or_default()
        ),
        None => format!("{} source: not linked, nothing is fetched", which.noun()),
    };
    Ok(Emitted {
        exit_code: 0,
        json: json!({
            "kind": which.arg(),
            "linked": source.is_some(),
            "integration": source.map(|s| s.integration.clone()),
            "project": source.map(|s| s.project.clone()),
            "filter": source.and_then(|s| s.filter.clone()),
            "changed": changed,
        }),
        human,
    })
}

pub(super) fn target_json(target: &RepoTarget) -> Value {
    json!({
        "integration": target.integration.id,
        "project": target.project,
        "remote": target.remote,
        "ambiguous": target.ambiguous,
        "source": if target.linked { "link" } else { "remote" },
    })
}

/// Pins (`id`), unpins (`clear`) or leaves alone the repository's profile.
/// Pinning a profile that does not exist or is not a hosting profile is an
/// error here even though a stale pin is tolerated at resolution time.
pub(super) fn apply_use(
    state: &mut gitpoulpi_session::RepoState,
    integrations: &[Integration],
    id: Option<&str>,
    clear: bool,
) -> Result<bool, String> {
    if clear {
        let had = state.hosting_integration.take().is_some();
        return Ok(had);
    }
    let Some(id) = id else {
        return Ok(false);
    };
    let integration = pick(integrations, Some(id))?;
    if integration.kind.category() != IntegrationCategory::Hosting || !integration.enabled {
        return Err(format!("`{id}` is not an enabled hosting integration"));
    }
    state.hosting_integration = Some(id.to_string());
    Ok(true)
}

pub(super) async fn run_use(
    env: &HostingEnv,
    ipc: &mut Session,
    id: Option<String>,
    clear: bool,
) -> Result<Emitted, String> {
    let root = ipc
        .repo_root()
        .ok_or_else(|| "not inside a git repository (pass --repo)".to_string())?
        .to_path_buf();
    let path = crate::commands::repo_state_path(&root);
    let mut state = gitpoulpi_session::RepoState::load(&path).map_err(|e| e.to_string())?;
    let integrations = env.integrations()?;
    if apply_use(&mut state, &integrations, id.as_deref(), clear)? {
        state.save(&path).map_err(|e| e.to_string())?;
    }
    let remotes: Vec<Remote> = ipc.call(OperationKind::Remotes, json!({})).await?;
    let resolve_remotes: Vec<ResolveRemote> = remotes
        .iter()
        .map(|r| ResolveRemote::new(r.name.clone(), r.fetch_url.clone()))
        .collect();
    let resolved = resolve_for_repo(
        &resolve_remotes,
        &integrations,
        state.hosting_integration.as_deref(),
    );
    let human = match (&state.hosting_integration, &resolved) {
        (Some(pinned), _) => format!("this repository is pinned to `{pinned}`"),
        (None, Some(r)) => format!("resolved by remote host: `{}`", r.integration.id),
        (None, None) => "no integration matches this repository".to_string(),
    };
    Ok(Emitted {
        exit_code: 0,
        json: json!({
            "pinned": state.hosting_integration,
            "resolved": resolved.as_ref().map(|r| r.integration.id.clone()),
            "ambiguous": resolved.as_ref().is_some_and(|r| r.ambiguous),
        }),
        human,
    })
}
