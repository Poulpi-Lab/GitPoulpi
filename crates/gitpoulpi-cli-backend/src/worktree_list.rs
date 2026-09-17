//! `git worktree list`, always compiled: the "branch is checked out in
//! another worktree" guards need it even when the `worktree` feature
//! (management UI/commands) is off.

use std::path::PathBuf;

use gitpoulpi_core::domain::{RepoRef, Worktree};
use gitpoulpi_core::Result;

use crate::process::run;

/// Parses `git worktree list --porcelain`, whose records are separated by
/// blank lines and look like:
///
/// ```text
/// worktree /path/to/main
/// HEAD 1234abcd...
/// branch refs/heads/main
///
/// worktree /path/to/linked
/// HEAD 5678ef01...
/// detached
/// ```
///
/// A record carries `branch` or `detached` (never both), and optionally
/// `locked` (with or without a trailing reason). The first record is
/// always the main worktree.
pub fn worktrees(repo: &RepoRef) -> Result<Vec<Worktree>> {
    let raw = run(&repo.root, &["worktree", "list", "--porcelain"])?;
    Ok(parse(&raw))
}

fn parse(raw: &str) -> Vec<Worktree> {
    let mut out: Vec<Worktree> = Vec::new();
    let mut current: Option<Worktree> = None;

    for line in raw.lines() {
        let line = line.trim_end();
        if let Some(path) = line.strip_prefix("worktree ") {
            if let Some(done) = current.take() {
                out.push(done);
            }
            current = Some(Worktree {
                path: PathBuf::from(path),
                branch: None,
                // The first record `git worktree list` emits is the main
                // worktree; every later one is linked.
                is_main: out.is_empty(),
                is_locked: false,
            });
        } else if let Some(branch) = line.strip_prefix("branch ") {
            if let Some(wt) = current.as_mut() {
                wt.branch = Some(branch.trim_start_matches("refs/heads/").to_string());
            }
        } else if line == "locked" || line.starts_with("locked ") {
            if let Some(wt) = current.as_mut() {
                wt.is_locked = true;
            }
        }
        // `HEAD <oid>`, `detached` and `bare` need no field of their own:
        // `branch: None` already means detached.
    }

    if let Some(done) = current.take() {
        out.push(done);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_main_and_linked_records() {
        let raw = "worktree /repo\nHEAD abc\nbranch refs/heads/main\n\n\
                   worktree /wt-feature\nHEAD def\nbranch refs/heads/feature\n";
        let got = parse(raw);
        assert_eq!(got.len(), 2);
        assert!(got[0].is_main);
        assert_eq!(got[0].branch.as_deref(), Some("main"));
        assert!(!got[1].is_main);
        assert_eq!(got[1].branch.as_deref(), Some("feature"));
    }

    #[test]
    fn detached_worktree_has_no_branch() {
        let raw = "worktree /repo\nHEAD abc\nbranch refs/heads/main\n\n\
                   worktree /wt\nHEAD def\ndetached\n";
        let got = parse(raw);
        assert_eq!(got[1].branch, None);
    }

    /// `locked` may carry a reason on the same line - both spellings
    /// must set the flag.
    #[test]
    fn locked_is_detected_with_and_without_a_reason() {
        let bare = parse("worktree /repo\nHEAD abc\nlocked\n");
        assert!(bare[0].is_locked);
        let with_reason = parse("worktree /repo\nHEAD abc\nlocked on a USB drive\n");
        assert!(with_reason[0].is_locked);
    }
}
