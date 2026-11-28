use gitpoulpi_core::domain::{LogOptions, RepoRef};
use gitpoulpi_core::Result;

use crate::log::{filtered_tips, run_with_head_fallback, tips_stdin};
use crate::process::{run_bytes, run_bytes_with_stdin};

/// `git rev-list --count` over the refs `log::log` walks. No `--topo-order`
/// (order is irrelevant to a count and it would defeat the commit-graph
/// shortcut), no format, no skip/limit. git uses the commit-graph file when
/// one exists, which makes this much cheaper than the log itself.
pub fn count_commits(repo: &RepoRef, opts: &LogOptions) -> Result<u64> {
    let mut args = vec!["rev-list".to_string(), "--count".to_string()];
    let mut stdin: Option<Vec<u8>> = None;
    let mut head_seed = false;
    if opts.all_refs && !opts.hidden.is_empty() {
        // The visible tips only (see `log::log`).
        let tips = filtered_tips(repo, &opts.hidden)?;
        if tips.is_empty() {
            return Ok(0);
        }
        args.push("--stdin".to_string());
        stdin = Some(tips_stdin(&tips));
    } else if opts.all_refs {
        // Same three families as `log`, never `--all` (the remote-tag
        // mirror under refs/gitpoulpi must not count).
        args.push("--branches".to_string());
        args.push("--tags".to_string());
        args.push("--remotes".to_string());
        head_seed = true;
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
    // An unborn HEAD makes `rev-list` fail: a valid state, zero commits.
    let go = |args: &[String]| {
        let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
        match &stdin {
            Some(input) => run_bytes_with_stdin(&repo.root, &args_ref, input),
            None => run_bytes(&repo.root, &args_ref),
        }
    };
    let result = if head_seed {
        run_with_head_fallback(&args, go)
    } else {
        go(&args)
    };
    let Ok(raw) = result else {
        return Ok(0);
    };
    Ok(String::from_utf8_lossy(&raw).trim().parse().unwrap_or(0))
}
