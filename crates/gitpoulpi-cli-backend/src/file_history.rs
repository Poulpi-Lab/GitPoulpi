//! `file_history` over the system `git`: one `git log --follow` process.
//!
//! This is also what `Git2Backend` runs: libgit2 has no `--follow`, and
//! re-implementing git's rename-chasing walk would make the two backends
//! disagree on exactly the cases the feature exists for.

use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{FileChangeKind, FileHistoryEntry, FileHistoryPage, Oid, RepoRef};
use gitpoulpi_core::Result;

use crate::content::repo_relative;
use crate::process::run_bytes;

/// See `GitBackend::file_history`.
///
/// Flags, and why:
/// - `--follow -M`: rename tracking (explicit `-M` so a user's
///   `diff.renames=false` cannot turn it off).
/// - `--diff-merges=first-parent`: a kept merge reports its change against
///   its first parent instead of an empty diff.
/// - `--max-count` of `skip + limit + 1` (one past the page, to learn
///   `more`): git walks lazily, so the first page of a huge history is
///   cheap. `--skip` is deliberately not used: git skips commits *before*
///   their diff is computed, so a rename inside the skipped range is never
///   seen and the path stops being followed.
/// - `-z` with `--raw --numstat`: status and line counts in one process,
///   no path quoting.
/// - `--literal-pathspecs`: a file called `*.rs` is that file.
pub fn file_history(
    repo: &RepoRef,
    path: &Path,
    start: Oid,
    skip: usize,
    limit: usize,
) -> Result<FileHistoryPage> {
    let rel = repo_relative(path)?;
    let start = start.to_hex();
    let max_arg = format!(
        "--max-count={}",
        skip.saturating_add(limit).saturating_add(1)
    );
    let raw = run_bytes(
        &repo.root,
        &[
            "--literal-pathspecs",
            "log",
            "--follow",
            "-M",
            "--diff-merges=first-parent",
            "--raw",
            "--numstat",
            "-z",
            "--no-color",
            "--no-show-signature",
            &max_arg,
            "--format=%x01%H%x00%P%x00%an%x00%ae%x00%at%x00%ct%x00%s",
            &start,
            "--",
            &rel,
        ],
    )?;
    let text = String::from_utf8_lossy(&raw);
    let mut entries = parse(&text, &rel);
    entries.drain(..skip.min(entries.len()));
    let more = entries.len() > limit;
    entries.truncate(limit);
    Ok(FileHistoryPage { entries, more })
}

fn parse(text: &str, requested: &str) -> Vec<FileHistoryEntry> {
    let mut out = Vec::new();
    // The path being followed, as of the commit about to be parsed.
    let mut tracking = requested.to_string();
    for chunk in text.split('\u{1}').skip(1) {
        let mut fields = chunk.splitn(8, '\0');
        let mut next = || fields.next().unwrap_or_default();
        let Some(commit) = Oid::from_hex(next()) else {
            continue;
        };
        let parents: Vec<Oid> = next().split(' ').filter_map(Oid::from_hex).collect();
        let author_name = next().to_string();
        let author_email = next().to_string();
        let timestamp = next().parse().unwrap_or_default();
        let committer_timestamp = next().parse().unwrap_or_default();
        let summary = next().to_string();
        let rest = next();

        let mut tokens = rest.trim_start_matches('\n').split('\0');
        let (change, path) = match tokens.next() {
            Some(meta) if meta.starts_with(':') => {
                let status = meta.rsplit(' ').next().unwrap_or("M");
                let first = tokens.next().unwrap_or_default().to_string();
                match status.chars().next() {
                    Some('R') => {
                        let to = tokens.next().unwrap_or_default().to_string();
                        (
                            FileChangeKind::Renamed {
                                from: PathBuf::from(first),
                            },
                            to,
                        )
                    }
                    // Copies are not detected (no `-C`), but a configured
                    // `diff.renames=copies` must not desynchronise parsing.
                    Some('C') => (
                        FileChangeKind::Added,
                        tokens.next().unwrap_or_default().to_string(),
                    ),
                    Some('A') => (FileChangeKind::Added, first),
                    Some('D') => (FileChangeKind::Deleted, first),
                    _ => (FileChangeKind::Modified, first),
                }
            }
            _ => (FileChangeKind::Modified, tracking.clone()),
        };
        // `<added>\t<removed>\t[<path>]`; `-` for both on a binary file.
        let (added, removed) = tokens
            .next()
            .map(|stat| {
                let mut parts = stat.splitn(3, '\t');
                let a = parts.next().and_then(|n| n.parse().ok());
                let r = parts.next().and_then(|n| n.parse().ok());
                (a, r)
            })
            .unwrap_or((Some(0), Some(0)));

        if let FileChangeKind::Renamed { from } = &change {
            tracking = from.to_string_lossy().into_owned();
        }
        out.push(FileHistoryEntry {
            commit,
            parents,
            author_name,
            author_email,
            timestamp,
            committer_timestamp,
            summary,
            path: PathBuf::from(path),
            change,
            added,
            removed,
        });
    }
    out
}
