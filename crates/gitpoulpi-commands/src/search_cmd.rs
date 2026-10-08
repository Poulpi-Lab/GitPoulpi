//! `gitpoulpi search <query> [--workspace id] [--kind k]... [--limit n]`.
//!
//! One daemon read per repository (`OperationKind::Search`: the daemon keeps
//! each repository's index warm), merged here by score. Without a workspace
//! it is the current repository alone. The `repo` kind is answered here, not
//! by a daemon: a repository is not inside a repository.

use gitpoulpi_core::domain::{
    parse_query, OperationKind, RepoRef, SearchHit, SearchHits, SearchKind, SearchRequest,
    SearchTarget,
};
use gitpoulpi_core::fuzzy::{self, Anchor};
use serde_json::{json, Value};

use crate::cli::SearchKindArg;
use crate::client::{discover_repo, Session};
use crate::commands::emitted;
use crate::repo_cmd::Emitted;
#[cfg(feature = "workspaces")]
use crate::workspace_cmd::{load_workspace, selected_workspace};

/// Without workspaces there is no workspace to select, and so none to load.
#[cfg(not(feature = "workspaces"))]
fn selected_workspace(_flag: Option<&str>) -> Option<String> {
    None
}

#[cfg(not(feature = "workspaces"))]
fn load_workspace(id: &str) -> Result<gitpoulpi_session::Workspace, String> {
    Err(format!(
        "no workspace `{id}` (workspaces are not built into this version)"
    ))
}

/// What the user asked, before it meets a repository.
pub struct SearchArgs {
    pub query: String,
    #[cfg(feature = "workspaces")]
    pub workspace: Option<String>,
    pub kinds: Vec<SearchKindArg>,
    pub limit: usize,
}

/// The repository's name as the tabs show it: the last path component.
fn repo_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

/// `r:` hits: the workspace's repositories whose name (else path) match.
fn repo_hits(paths: &[std::path::PathBuf], text: &str) -> Vec<SearchHit> {
    let text = text.to_lowercase();
    paths
        .iter()
        .filter_map(|path| {
            let name = repo_name(path);
            let by_name = fuzzy::matched(&name.to_lowercase(), &text, Anchor::Whole);
            let (score, positions) = match by_name {
                Some(m) => (m.score, m.positions),
                // A path match ranks under every name match.
                None => (
                    fuzzy::score(&path.to_string_lossy(), &text, Anchor::Whole)? - 20_000,
                    Vec::new(),
                ),
            };
            Some(SearchHit {
                kind: SearchKind::Repo,
                label: name,
                detail: path.to_string_lossy().into_owned(),
                target: SearchTarget::Repo {
                    path: path.to_string_lossy().into_owned(),
                },
                score,
                positions,
                repo: Some(path.to_string_lossy().into_owned()),
            })
        })
        .collect()
}

pub async fn run(
    ipc: &mut Session,
    current: Option<RepoRef>,
    args: SearchArgs,
) -> Result<Emitted, String> {
    #[cfg(feature = "workspaces")]
    let workspace_id = selected_workspace(args.workspace.as_deref());
    #[cfg(not(feature = "workspaces"))]
    let workspace_id = selected_workspace(None);
    let parsed = parse_query(&args.query);
    let mut kinds: Vec<SearchKind> = args.kinds.iter().map(|k| (*k).into()).collect();
    if kinds.is_empty() {
        kinds = SearchKind::EVERY.to_vec();
    }
    // A prefix replaces the filter, like typing it in the TUI.
    if let Some(kind) = parsed.kind {
        kinds = vec![kind];
    } else if parsed.hosting_only {
        kinds = vec![SearchKind::PullRequest, SearchKind::Issue];
    }
    let limit = if args.limit == 0 { 50 } else { args.limit };

    // The repositories to ask: the workspace's, or the current one.
    let (workspace_name, paths): (Option<String>, Vec<std::path::PathBuf>) = match &workspace_id {
        Some(id) => {
            let w = load_workspace(id)?;
            (Some(w.id.clone()), w.repos.clone())
        }
        None => {
            let repo = current.ok_or("not inside a git repository (pass --repo or --workspace)")?;
            (None, vec![repo.root])
        }
    };

    let mut hits: Vec<SearchHit> = Vec::new();
    let mut partial = false;
    let mut searched: Vec<Value> = Vec::new();
    let wants_repos = kinds.contains(&SearchKind::Repo) && workspace_id.is_some();
    if wants_repos {
        hits.extend(repo_hits(&paths, &parsed.text));
    }
    let daemon_kinds: Vec<SearchKind> = kinds
        .iter()
        .copied()
        .filter(|k| *k != SearchKind::Repo && !k.is_hosting())
        .collect();
    let hosting_kinds: Vec<SearchKind> = kinds.iter().copied().filter(|k| k.is_hosting()).collect();
    if !hosting_kinds.is_empty() {
        // The caches the TUI keeps: files only, never a host.
        let state_dir = gitpoulpi_config::Settings::resolve_ambient()
            .map(|s| s.config().session.per_repo_state_dir.clone())
            .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
        for path in &paths {
            let cache = gitpoulpi_session::RepoState::hosting_cache_path_for(path, &state_dir);
            let items = gitpoulpi_hosting::cache_search::load(&cache);
            let root = path.to_string_lossy().into_owned();
            hits.extend(
                gitpoulpi_hosting::cache_search::search(
                    &items,
                    &hosting_kinds,
                    &parsed.text,
                    limit,
                )
                .into_iter()
                .map(|mut h| {
                    h.repo = Some(root.clone());
                    h
                }),
            );
        }
    }
    if !daemon_kinds.is_empty() {
        for path in &paths {
            let repo = match discover_repo(Some(&path.to_string_lossy())) {
                Ok(repo) => repo,
                Err(e) => {
                    searched.push(json!({ "path": path, "searched": false, "reason": e }));
                    continue;
                }
            };
            ipc.set_repo(Some(repo.clone()));
            let request = SearchRequest {
                query: args.query.clone(),
                kinds: daemon_kinds.clone(),
                limit,
                wait: true,
            };
            match ipc
                .call::<SearchHits>(OperationKind::Search, json!(request))
                .await
            {
                Ok(found) => {
                    partial |= found.partial;
                    searched.push(
                        json!({ "path": repo.root, "searched": true, "hits": found.hits.len() }),
                    );
                    let root = repo.root.to_string_lossy().into_owned();
                    hits.extend(found.hits.into_iter().map(|mut h| {
                        h.repo = Some(root.clone());
                        h
                    }));
                }
                Err(e) => searched.push(json!({ "path": path, "searched": false, "reason": e })),
            }
        }
    }

    // Best first; equal scores by label, then repository, so the order is
    // the same on every run.
    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.label.cmp(&b.label))
            .then_with(|| a.repo.cmp(&b.repo))
    });
    hits.truncate(limit);

    let human = if hits.is_empty() {
        "no matches".to_string()
    } else {
        hits.iter()
            .map(|h| {
                let repo = h
                    .repo
                    .as_deref()
                    .map(|r| repo_name(std::path::Path::new(r)))
                    .unwrap_or_default();
                format!(
                    "{:<9} {:<16} {}  {}",
                    h.kind.name(),
                    repo,
                    h.label,
                    h.detail
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    Ok(emitted(
        json!({
            "query": args.query,
            "workspace": workspace_name,
            "hits": hits,
            "partial": partial,
            "repos": searched,
        }),
        human,
    ))
}
