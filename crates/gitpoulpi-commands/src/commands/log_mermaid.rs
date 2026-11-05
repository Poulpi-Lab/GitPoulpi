//! `log --format mermaid`: the Log's commits as a Mermaid `gitGraph` script.
//! The script comes from `gitpoulpi_core::domain::mermaid_git_graph`, the
//! same pure function the TUI's "Copy as Mermaid gitGraph" popup runs.

use std::collections::HashSet;

use gitpoulpi_core::domain::{
    attach_refs, mermaid_git_graph, CommitGraph, LogOptions, MermaidDetail, MermaidOptions,
    MermaidOrientation, Oid, OperationKind,
};
use serde_json::json;

use super::{emitted, resolve, Emitted};
use crate::client::Session;

/// Commits taken when neither `--range` nor `--limit` says how many.
const DEFAULT_LIMIT: u32 = 30;

pub(super) struct Args {
    pub graph: bool,
    pub limit: Option<u32>,
    pub skip: u32,
    pub path: Vec<String>,
    pub detail: Option<MermaidDetail>,
    pub orientation: Option<MermaidOrientation>,
    pub fence: bool,
    pub range: Option<String>,
    pub subject_width: Option<usize>,
}

async fn walk(session: &mut Session, opts: LogOptions) -> Result<CommitGraph, String> {
    session
        .call(OperationKind::Log, json!({ "opts": opts }))
        .await
}

pub(super) async fn log_mermaid(session: &mut Session, args: Args) -> Result<Emitted, String> {
    let mut commits = match &args.range {
        Some(range) => {
            let (from, to) = match range.split_once("..") {
                Some((a, b)) => (a, if b.is_empty() { "HEAD" } else { b }),
                None => (range.as_str(), "HEAD"),
            };
            let mut walked = walk(
                session,
                LogOptions {
                    start_ref: Some(to.to_string()),
                    paths: args.path.iter().map(Into::into).collect(),
                    ..LogOptions::default()
                },
            )
            .await?;
            let excluded: HashSet<Oid> = walk(
                session,
                LogOptions {
                    start_ref: Some(from.to_string()),
                    ..LogOptions::default()
                },
            )
            .await?
            .nodes
            .into_iter()
            .map(|n| n.id)
            .collect();
            walked.nodes.retain(|n| !excluded.contains(&n.id));
            if let Some(limit) = args.limit {
                walked.nodes.truncate(limit as usize);
            }
            walked
        }
        None => {
            walk(
                session,
                LogOptions {
                    limit: Some(args.limit.unwrap_or(DEFAULT_LIMIT)),
                    skip: args.skip,
                    start_ref: None,
                    all_refs: args.graph,
                    paths: args.path.iter().map(Into::into).collect(),
                    use_mailmap: false,
                    hidden: Default::default(),
                },
            )
            .await?
        }
    };
    attach_refs(&mut commits.nodes, &super::refs::all_refs(session).await?);
    let head = resolve(session, "HEAD").await.ok();
    // What the popup last used (`ui.mermaid_*`) is the default.
    let ui = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().ui.clone())
        .unwrap_or_default();
    let detail = args.detail.unwrap_or(ui.mermaid_detail);
    let text = mermaid_git_graph(
        &commits.nodes,
        head,
        &MermaidOptions {
            detail,
            orientation: args.orientation.unwrap_or(ui.mermaid_orientation),
            fence: args.fence || ui.mermaid_fence,
            subject_width: args
                .subject_width
                .unwrap_or(ui.mermaid_subject_width as usize),
        },
    );
    let ids: Vec<String> = commits.nodes.iter().map(|n| n.id.to_hex()).collect();
    Ok(emitted(
        json!({
            "format": "mermaid",
            "detail": detail.as_str(),
            "commits": ids,
            "text": text,
        }),
        text.trim_end().to_string(),
    ))
}
