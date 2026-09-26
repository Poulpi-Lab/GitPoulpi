//! Review without a clone: a target named by `--project`, `hosting review
//! diff` and `hosting file`. The host's API is the only source here; the
//! commands that need the repository stay in [`super::review`].

use gitpoulpi_core::domain::{FileDiff, FileStatus, LineOrigin};
use gitpoulpi_hosting::review::ApiFileDiff;
use gitpoulpi_hosting::HostingClient;
use serde_json::{json, Value};

use crate::cli::FileArgs;
use crate::repo_cmd::Emitted;

use super::link::Which;
use super::*;

/// The target of `--project P [--id I]`: no repository, no remote. `--id`
/// is optional when exactly one integration is configured.
pub(super) fn project_target(
    env: &HostingEnv,
    id: Option<&str>,
    project: &str,
) -> Result<RepoTarget, String> {
    let project = project.trim().trim_matches('/');
    if project.is_empty() {
        return Err("--project is empty (expected owner/name or group/project)".to_string());
    }
    let integrations = env.integrations()?;
    let integration = pick(&integrations, id)?.clone();
    if !integration.enabled {
        return Err(format!("`{}` is disabled", integration.id));
    }
    if let Some(why) = Which::Pulls.unsupported_by(&integration) {
        return Err(why);
    }
    Ok(RepoTarget {
        integration,
        integrations,
        project: project.to_string(),
        remote: String::new(),
        ambiguous: false,
        linked: false,
        filter: None,
    })
}

/// Whether `target` came from `--project` rather than from a repository.
pub(super) fn is_project_target(target: &RepoTarget) -> bool {
    !target.linked && target.remote.is_empty()
}

/// A local `FileDiff` in the API's shape, counts included.
pub(super) fn api_shape(diff: FileDiff) -> ApiFileDiff {
    let count = |origin: LineOrigin| {
        diff.hunks
            .iter()
            .flat_map(|h| h.lines.iter())
            .filter(|l| l.origin == origin)
            .count() as u32
    };
    ApiFileDiff {
        additions: count(LineOrigin::Add),
        deletions: count(LineOrigin::Delete),
        diff,
        previous_path: None,
        omitted: None,
    }
}

/// The request's diff from the host: the files with hunks, and the paths
/// the host left out.
#[cfg_attr(not(feature = "ai"), allow(dead_code))] // only `review suggest` uses it
pub(super) async fn api_files(
    client: &dyn HostingClient,
    target: &RepoTarget,
    number: u64,
) -> Result<(Vec<FileDiff>, Vec<String>), String> {
    let all = client
        .pull_request_diff(&target.project, number, None, None)
        .await
        .map_err(|e| e.to_string())?;
    let omitted = all
        .iter()
        .filter(|f| f.omitted.is_some())
        .map(|f| f.diff.path.to_string_lossy().into_owned())
        .collect();
    let files = all
        .into_iter()
        .filter(|f| f.omitted.is_none())
        .map(|f| f.diff)
        .collect();
    Ok((files, omitted))
}

/// `review diff`: the files with their hunks, from the clone (`local`,
/// already computed by the caller) or from the API.
pub(super) fn diff_output(
    target: &RepoTarget,
    number: u64,
    source: &str,
    mut files: Vec<ApiFileDiff>,
    only: Option<&str>,
) -> Result<Emitted, String> {
    if let Some(only) = only {
        let only = only.trim_start_matches('/');
        files.retain(|f| f.diff.path.to_string_lossy() == only);
        if files.is_empty() {
            return Err(format!(
                "{only} is not among the files of {}",
                super::review::target_label(target, number)
            ));
        }
    }
    let mut human = Vec::new();
    for f in &files {
        human.push(format!(
            "{} {}  +{} -{}{}",
            status_word(f.diff.status),
            f.diff.path.display(),
            f.additions,
            f.deletions,
            f.omitted
                .as_deref()
                .map(|why| format!("  ({why})"))
                .unwrap_or_default()
        ));
        for h in &f.diff.hunks {
            human.push(format!(
                "@@ -{},{} +{},{} @@",
                h.old_start, h.old_lines, h.new_start, h.new_lines
            ));
            for l in &h.lines {
                let mark = match l.origin {
                    LineOrigin::Add => '+',
                    LineOrigin::Delete => '-',
                    LineOrigin::Context => ' ',
                };
                human.push(format!("{mark}{}", l.content));
            }
        }
    }
    let mut out = super::review::target_json(target);
    out["number"] = json!(number);
    out["source"] = json!(source);
    out["files"] = serde_json::to_value(&files).map_err(|e| e.to_string())?;
    Ok(Emitted {
        exit_code: 0,
        json: out,
        human: human.join("\n"),
    })
}

fn status_word(status: FileStatus) -> &'static str {
    match status {
        FileStatus::Added => "added",
        FileStatus::Deleted => "deleted",
        FileStatus::Renamed => "renamed",
        _ => "modified",
    }
}

/// `hosting file`: one file at a revision, from the host.
pub(super) async fn file(env: &HostingEnv, args: FileArgs) -> Result<Emitted, String> {
    let target = project_target(env, args.id.as_deref(), &args.project)?;
    let client = env.client(&target.integration, &target.integrations)?;
    let path = args.path.trim_start_matches('/').to_string();
    let bytes = client
        .file_content(&target.project, &path, args.rev.as_deref())
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| {
            format!(
                "{path} does not exist in {} at {}",
                target.project,
                args.rev.as_deref().unwrap_or("the default branch")
            )
        })?;
    let size = bytes.len();
    let (encoding, content) = match String::from_utf8(bytes) {
        Ok(text) => ("utf-8", text),
        Err(e) => ("base64", gitpoulpi_hosting::auth::base64(e.as_bytes())),
    };
    let human = if encoding == "utf-8" {
        content.clone()
    } else {
        format!("{path}: binary file, {size} bytes (base64 in the JSON output)")
    };
    let mut out: Value = json!({
        "integration": target.integration.id,
        "project": target.project,
        "path": path,
        "ref": args.rev,
        "size": size,
        "encoding": encoding,
        "content": content,
    });
    out["found"] = json!(true);
    Ok(Emitted {
        exit_code: 0,
        json: out,
        human,
    })
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    fn local(args: &[&str]) -> bool {
        let cli = crate::cli::Cli::try_parse_from(
            std::iter::once("gitpoulpi").chain(args.iter().copied()),
        )
        .unwrap();
        crate::commands::is_local(&cli.command.unwrap())
    }

    #[test]
    fn a_named_project_needs_no_daemon_a_repository_review_does() {
        assert!(local(&[
            "hosting",
            "review",
            "show",
            "9",
            "--project",
            "o/r"
        ]));
        assert!(local(&[
            "hosting",
            "review",
            "diff",
            "9",
            "--project",
            "o/r"
        ]));
        assert!(local(&["hosting", "file", "a.md", "--project", "o/r"]));
        assert!(!local(&["hosting", "review", "show", "9"]));
        assert!(!local(&["hosting", "review", "diff", "9", "--api"]));
        assert!(!local(&["hosting", "review", "fetch", "9"]));
    }
}
