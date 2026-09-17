use std::path::PathBuf;

use gitpoulpi_core::domain::{FileStatus, RepoRef, StatusEntry, WorkingTree};
use gitpoulpi_core::Result;

use crate::process::run_z_fields;

fn status_char(c: char) -> Option<FileStatus> {
    match c {
        'M' => Some(FileStatus::Modified),
        'A' => Some(FileStatus::Added),
        'D' => Some(FileStatus::Deleted),
        'R' => Some(FileStatus::Renamed),
        'C' => Some(FileStatus::Renamed),
        'U' => Some(FileStatus::Conflicted),
        _ => None,
    }
}

/// Parses `git status --porcelain=v2 -z` output. Record boundaries aren't
/// simply "one line per NUL": a rename/copy record (type `2`) is followed
/// by an extra NUL-terminated origPath token, so this walks the token
/// stream with an explicit cursor instead of mapping line-by-line.
pub fn status(repo: &RepoRef) -> Result<WorkingTree> {
    let tokens = run_z_fields(
        &repo.root,
        &["status", "--porcelain=v2", "-z", "--untracked-files=all"],
    )?;

    let mut files = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        let mut parts = token.splitn(2, ' ');
        let kind = parts.next().unwrap_or_default();

        match kind {
            "1" | "2" => {
                let rest = parts.next().unwrap_or_default();
                let mut fields = rest.split(' ');
                let xy = fields.next().unwrap_or_default();
                // sub, mH, mI, mW, hH, hI[, Xscore for type 2] then path is the remainder.
                let skip_count = if kind == "2" { 7 } else { 6 };
                let mut remaining_fields = fields.clone();
                for _ in 0..skip_count {
                    remaining_fields.next();
                }
                let path: PathBuf = remaining_fields.collect::<Vec<_>>().join(" ").into();

                let mut chars = xy.chars();
                let index = chars.next().and_then(status_char);
                let worktree = chars.next().and_then(status_char);
                files.push(StatusEntry {
                    path,
                    index,
                    worktree,
                });

                if kind == "2" {
                    // consume the origPath token that follows this record
                    i += 1;
                }
            }
            "u" => {
                let rest = parts.next().unwrap_or_default();
                let mut fields = rest.split(' ');
                for _ in 0..9 {
                    fields.next();
                }
                let path: PathBuf = fields.collect::<Vec<_>>().join(" ").into();
                files.push(StatusEntry {
                    path,
                    index: Some(FileStatus::Conflicted),
                    worktree: Some(FileStatus::Conflicted),
                });
            }
            "?" => {
                let rest = parts.next().unwrap_or_default();
                files.push(StatusEntry {
                    path: PathBuf::from(rest),
                    index: None,
                    worktree: Some(FileStatus::Untracked),
                });
            }
            _ => {}
        }

        i += 1;
    }

    Ok(WorkingTree { files })
}
