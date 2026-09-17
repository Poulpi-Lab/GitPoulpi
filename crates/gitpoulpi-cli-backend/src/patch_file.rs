//! `git format-patch` / `git diff` for [`GitBackend::patch_create`], and
//! `git apply` for [`GitBackend::patch_apply`].
//!
//! Named `patch_file.rs` because `patch.rs` is the hunk/line patch-text
//! module behind staging and discarding.
//!
//! Everything that touches patch bytes goes through
//! [`crate::process::run_bytes`], never `String`-based `run`: a patch can
//! carry raw CRLF line endings that must survive byte-for-byte, and
//! `str::lines()` would drop the `\r`.
//!
//! [`GitBackend::patch_create`]: gitpoulpi_core::GitBackend::patch_create
//! [`GitBackend::patch_apply`]: gitpoulpi_core::GitBackend::patch_apply

use std::path::Path;

use gitpoulpi_core::domain::{PatchApplied, PatchCreated, PatchSource, RepoRef};
use gitpoulpi_core::{Error, Result};

use crate::process::{run, run_bytes};

/// Refuses (writing nothing) if `dest` exists or its parent is missing.
/// Checked up front so every `patch_create` branch below can build its
/// bytes first and write once, at the end, having already confirmed the
/// destination is writable.
fn check_dest(dest: &Path) -> Result<()> {
    if dest.exists() {
        return Err(Error::Backend(format!("{} already exists", dest.display())));
    }
    match dest.parent() {
        Some(parent) if !parent.as_os_str().is_empty() && !parent.exists() => Err(Error::Backend(
            format!("{} has no parent directory", parent.display()),
        )),
        _ => Ok(()),
    }
}

/// The parent-count of `oid`, and implicitly a check that `oid` names a
/// real commit - `git rev-list` fails on an unknown one.
fn short_hex(hex: &str) -> &str {
    &hex[..hex.len().min(8)]
}

fn parent_count(repo: &RepoRef, oid: &gitpoulpi_core::domain::Oid) -> Result<usize> {
    let raw = run(
        &repo.root,
        &["rev-list", "--parents", "-n", "1", &oid.to_hex()],
    )
    .map_err(|_| Error::Backend(format!("{} is not a known commit", oid.to_hex())))?;
    Ok(raw.split_whitespace().count().saturating_sub(1))
}

pub fn patch_create(repo: &RepoRef, source: &PatchSource, dest: &Path) -> Result<PatchCreated> {
    check_dest(dest)?;

    let (bytes, commits): (Vec<u8>, usize) = match source {
        PatchSource::Commits(oids) => {
            if oids.is_empty() {
                return Err(Error::Backend("no commits given".into()));
            }
            let mut buf = Vec::new();
            for oid in oids {
                if parent_count(repo, oid)? > 1 {
                    return Err(Error::Backend(format!(
                        "{} is a merge commit and cannot become a patch",
                        oid.to_hex()
                    )));
                }
                let one = run_bytes(
                    &repo.root,
                    &["format-patch", "-1", "--stdout", "--binary", &oid.to_hex()],
                )?;
                buf.extend_from_slice(&one);
            }
            (buf, oids.len())
        }
        PatchSource::Range { base, tip } => {
            let range = format!("{}..{}", base.to_hex(), tip.to_hex());
            let count_raw = run(&repo.root, &["rev-list", "--no-merges", "--count", &range])?;
            let commits: usize = count_raw.trim().parse().unwrap_or(0);
            if commits == 0 {
                return Err(Error::Backend(format!(
                    "{range} has no non-merge commits to turn into a patch"
                )));
            }
            let bytes = run_bytes(
                &repo.root,
                &[
                    "format-patch",
                    &range,
                    "--stdout",
                    "--binary",
                    "--no-merges",
                ],
            )?;
            (bytes, commits)
        }
        PatchSource::CommitPaths { commit, paths } => {
            if paths.is_empty() {
                return Err(Error::Backend("no paths given".into()));
            }
            if parent_count(repo, commit)? > 1 {
                return Err(Error::Backend(format!(
                    "{} is a merge commit and cannot become a patch",
                    commit.to_hex()
                )));
            }
            let hex = commit.to_hex();
            // A pathspec the commit did not touch still yields a mail
            // header with no diff: ask the commit's own change list first.
            let mut touched: Vec<&str> = vec![
                "diff-tree",
                "--no-commit-id",
                "--name-only",
                "-r",
                &hex,
                "--",
            ];
            touched.extend(paths.iter().map(String::as_str));
            if run(&repo.root, &touched)?.trim().is_empty() {
                return Err(Error::Backend(format!(
                    "{} did not change the given paths",
                    short_hex(&hex)
                )));
            }
            let mut args: Vec<&str> =
                vec!["format-patch", "-1", "--stdout", "--binary", &hex, "--"];
            args.extend(paths.iter().map(String::as_str));
            (run_bytes(&repo.root, &args)?, 1)
        }
        PatchSource::WorkingTree { paths } => {
            run(&repo.root, &["rev-parse", "--verify", "HEAD"])
                .map_err(|_| Error::Backend("HEAD has no commits yet".into()))?;
            let mut args: Vec<&str> = vec!["diff", "HEAD", "--binary"];
            if !paths.is_empty() {
                args.push("--");
                args.extend(paths.iter().map(String::as_str));
            }
            let bytes = run_bytes(&repo.root, &args)?;
            (bytes, 0)
        }
    };

    if bytes.is_empty() {
        return Err(Error::Backend("the patch would be empty".into()));
    }

    std::fs::write(dest, &bytes)
        .map_err(|e| Error::Backend(format!("failed to write {}: {e}", dest.display())))?;

    Ok(PatchCreated {
        path: dest.to_path_buf(),
        commits,
        bytes: bytes.len() as u64,
    })
}

/// Parses `git apply --numstat -z`'s NUL-delimited records
/// (`<added>\t<deleted>\t<path>`) into just the paths, in order.
fn parse_numstat_z(raw: &str) -> Vec<String> {
    raw.split('\0')
        .filter(|record| !record.is_empty())
        .filter_map(|record| record.rsplit('\t').next())
        .map(str::to_string)
        .collect()
}

pub fn patch_apply(repo: &RepoRef, patch: &Path) -> Result<PatchApplied> {
    if !patch.is_file() {
        return Err(Error::Backend(format!("{} is not a file", patch.display())));
    }
    let patch_arg = patch.to_string_lossy().to_string();

    // Dry run first: `--check` verifies the patch applies without touching
    // the tree. `--numstat -z` on the same invocation yields the file list,
    // rather than parsing the real apply's human output.
    let numstat = run(
        &repo.root,
        &["apply", "--check", "--numstat", "-z", &patch_arg],
    )?;
    let files = parse_numstat_z(&numstat);
    if files.is_empty() {
        return Err(Error::Backend("the patch applies no changes".into()));
    }

    run(&repo.root, &["apply", "--whitespace=nowarn", &patch_arg])?;

    Ok(PatchApplied { files })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_numstat_z_records_into_paths() {
        let raw = "3\t1\tsrc/a.rs\0-\t-\tassets/logo.png\0";
        assert_eq!(
            parse_numstat_z(raw),
            vec!["src/a.rs".to_string(), "assets/logo.png".to_string()]
        );
    }

    #[test]
    fn empty_numstat_is_no_files() {
        assert!(parse_numstat_z("").is_empty());
    }
}
