use std::collections::HashSet;

use gitpoulpi_core::domain::RepoRef;
use gitpoulpi_core::Result;

use crate::process::{run, run_z_fields};

/// Command lines are capped at about 32k characters on Windows; stay well
/// under it and split a very long path list into several batches.
const MAX_ARG_BYTES: usize = 20_000;

fn batches(specs: &[String]) -> Vec<&[String]> {
    let mut out = Vec::new();
    let (mut start, mut size) = (0, 0);
    for (i, s) in specs.iter().enumerate() {
        if size + s.len() + 1 > MAX_ARG_BYTES && i > start {
            out.push(&specs[start..i]);
            start = i;
            size = 0;
        }
        size += s.len() + 1;
    }
    if start < specs.len() {
        out.push(&specs[start..]);
    }
    out
}

/// Discards every uncommitted change to each of `paths` - see
/// `GitBackend::discard` for the contract. A constant number of git
/// processes for the whole batch:
///
/// 1. `git reset -q -- <paths>` unstages them all, so nothing is left
///    half-reset.
/// 2. `git ls-files -z -- <paths>` lists what the index now holds, which
///    after step 1 is the tracked set.
/// 3. `git checkout -q -- <tracked>` restores those worktree copies.
/// 4. Anything never tracked at `HEAD` has nothing to restore to, so it is
///    removed.
///
/// A path that exists nowhere is ignored (discarding is idempotent).
/// Pathspecs are literal (`:(literal)`), so glob characters are only
/// themselves.
pub fn discard(repo: &RepoRef, paths: &[String]) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }
    let specs: Vec<String> = paths
        .iter()
        .map(|p| format!(":(literal){}", p.replace('\\', "/")))
        .collect();

    let mut tracked: Vec<String> = Vec::new();
    for batch in batches(&specs) {
        let mut reset = vec!["reset", "-q", "--"];
        reset.extend(batch.iter().map(String::as_str));
        let _ = run(&repo.root, &reset);

        let mut ls = vec!["ls-files", "-z", "--"];
        ls.extend(batch.iter().map(String::as_str));
        if let Ok(found) = run_z_fields(&repo.root, &ls) {
            tracked.extend(found);
        }
    }

    let restore_specs: Vec<String> = tracked.iter().map(|p| format!(":(literal){p}")).collect();
    for batch in batches(&restore_specs) {
        let mut checkout = vec!["checkout", "-q", "--"];
        checkout.extend(batch.iter().map(String::as_str));
        let _ = run(&repo.root, &checkout);
    }

    let tracked: HashSet<&str> = tracked.iter().map(String::as_str).collect();
    let mut removed: Vec<String> = Vec::new();
    for path in paths {
        let norm = path.replace('\\', "/");
        if tracked.contains(norm.as_str()) {
            continue;
        }
        let full = repo.root.join(path);
        if (full.is_file() || full.is_symlink()) && std::fs::remove_file(&full).is_ok() {
            removed.push(norm);
        }
    }
    // Folders the deleted files leave empty go too (never one that still
    // holds anything, tracked or not).
    crate::prune::prune_empty_parents(&repo.root, &removed, &|dir| {
        let spec = format!(":(literal){dir}");
        run_z_fields(&repo.root, &["ls-files", "-z", "--", &spec]).map_or(true, |f| !f.is_empty())
    });
    Ok(())
}
