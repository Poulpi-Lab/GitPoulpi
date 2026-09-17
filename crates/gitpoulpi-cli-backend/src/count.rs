use gitpoulpi_core::domain::{LogOptions, RepoRef};
use gitpoulpi_core::Result;

use crate::process::run_bytes;

/// `git rev-list --count` over the refs `log::log` walks. No `--topo-order`
/// (order is irrelevant to a count and it would defeat the commit-graph
/// shortcut), no format, no skip/limit. git uses the commit-graph file when
/// one exists, which makes this much cheaper than the log itself.
pub fn count_commits(repo: &RepoRef, opts: &LogOptions) -> Result<u64> {
    let mut args = vec!["rev-list".to_string(), "--count".to_string()];
    if opts.all_refs {
        // Same three families as `log`, never `--all` (the remote-tag
        // mirror under refs/gitpoulpi must not count).
        args.push("--branches".to_string());
        args.push("--tags".to_string());
        args.push("--remotes".to_string());
    } else if let Some(start) = &opts.start_ref {
        args.push(start.clone());
    } else {
        args.push("HEAD".to_string());
    }
    if !opts.paths.is_empty() {
        args.push("--".to_string());
        args.extend(
            opts.paths
                .iter()
                .map(|p| p.to_string_lossy().replace('\\', "/")),
        );
    }
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    // An unborn HEAD makes `rev-list` fail: a valid state, zero commits.
    let Ok(raw) = run_bytes(&repo.root, &args_ref) else {
        return Ok(0);
    };
    Ok(String::from_utf8_lossy(&raw).trim().parse().unwrap_or(0))
}
