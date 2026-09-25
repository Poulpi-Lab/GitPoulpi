//! `gitpoulpi hosting refs` -- the issue / pull request references in a
//! commit message or a branch name, through the repository's links.

use gitpoulpi_core::domain::{CommitGraph, LogOptions, OperationKind};
use gitpoulpi_hosting::refs::{
    from_branch, scan_message, FoundRef, IssueLinker, PullLinker, RefContext, RefKind,
};
use serde_json::{json, Value};

use crate::client::Session;
use crate::repo_cmd::Emitted;

use super::*;

/// What the repository's two links make a token mean. A link whose profile
/// is gone or disabled counts as unlinked: nothing is a reference.
fn context(
    state: &gitpoulpi_session::RepoState,
    integrations: &[Integration],
    root: Option<&std::path::Path>,
) -> RefContext {
    let linked = |source: &gitpoulpi_session::IssuesSource| {
        let integration = integrations
            .iter()
            .find(|i| i.id == source.integration && i.enabled)?;
        let base = gitpoulpi_hosting::integration_web_base(integration)?;
        let mut project = source.project.clone();
        // A local folder is read against the repository.
        if let (true, Some(root)) = (integration.kind.capabilities().local_folder, root) {
            project = gitpoulpi_hosting::resolve_local_folder(root, &project)
                .to_string_lossy()
                .into_owned();
        }
        Some((integration.kind.family(), base, project))
    };
    RefContext {
        issues: state
            .issues_source
            .as_ref()
            .and_then(linked)
            .map(|(family, web_base, project)| IssueLinker {
                family,
                web_base,
                project,
                project_keys: Vec::new(),
                cards: Vec::new(),
            }),
        pulls: state
            .pulls_source
            .as_ref()
            .and_then(linked)
            .map(|(family, web_base, project)| PullLinker {
                family,
                web_base,
                project,
            }),
    }
}

fn found_json(found: &FoundRef) -> Value {
    json!({
        "text": found.text,
        "kind": match found.kind {
            RefKind::Issue => "issue",
            RefKind::PullRequest => "pull_request",
        },
        "reference": found.reference,
        "url": found.url,
    })
}

pub(super) async fn run_refs(
    env: &HostingEnv,
    ipc: &mut Session,
    message: Option<String>,
    commit: Option<String>,
    branch: Option<String>,
) -> Result<Emitted, String> {
    let state = load_state(ipc)?;
    let ctx = context(&state, &env.integrations()?, ipc.repo_root());
    let (source, input, found): (&str, String, Vec<FoundRef>) = match (message, commit, branch) {
        (Some(text), None, None) => ("message", text.clone(), scan_message(&text, &ctx)),
        (None, Some(rev), None) => {
            let graph: CommitGraph = ipc
                .call(
                    OperationKind::Log,
                    json!({ "opts": LogOptions {
                        limit: Some(1),
                        start_ref: Some(rev.clone()),
                        ..Default::default()
                    } }),
                )
                .await?;
            let node = graph
                .nodes
                .first()
                .ok_or_else(|| format!("no commit `{rev}`"))?;
            ("commit", rev, scan_message(&node.message, &ctx))
        }
        (None, None, Some(name)) => (
            "branch",
            name.clone(),
            from_branch(&name, &ctx).into_iter().collect(),
        ),
        _ => return Err("pass one of --message, --commit or --branch".into()),
    };
    let human = if found.is_empty() {
        "no references".to_string()
    } else {
        found
            .iter()
            .map(|f| {
                format!(
                    "{:<12} {:<13} {}",
                    f.reference,
                    match f.kind {
                        RefKind::Issue => "issue",
                        RefKind::PullRequest => "pull request",
                    },
                    f.url.as_deref().unwrap_or("(no link)")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    Ok(Emitted {
        exit_code: 0,
        json: json!({
            "source": source,
            "input": input,
            "issues_linked": ctx.issues.is_some(),
            "pulls_linked": ctx.pulls.is_some(),
            "refs": found.iter().map(found_json).collect::<Vec<_>>(),
        }),
        human,
    })
}
