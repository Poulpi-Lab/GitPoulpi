use std::path::PathBuf;

use gitpoulpi_core::domain::{
    is_unstaged_rename_source, pair_unstaged_renames, read_workdir_candidate, FileStatus,
    RenameSide, RepoRef, StatusEntry, WorkingTree, RENAME_LIMIT, RENAME_MAX_FILE_BYTES,
};
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
                // A rename/copy record is followed by its origPath token.
                let renamed_from = if kind == "2" {
                    i += 1;
                    tokens.get(i).map(PathBuf::from)
                } else {
                    None
                };
                files.push(StatusEntry {
                    path,
                    index,
                    worktree,
                    renamed_from,
                });
            }
            "u" => {
                let rest = parts.next().unwrap_or_default();
                let mut fields = rest.split(' ');
                for _ in 0..9 {
                    fields.next();
                }
                let path: PathBuf = fields.collect::<Vec<_>>().join(" ").into();
                files.push(StatusEntry {
                    renamed_from: None,
                    path,
                    index: Some(FileStatus::Conflicted),
                    worktree: Some(FileStatus::Conflicted),
                });
            }
            "?" => {
                let rest = parts.next().unwrap_or_default();
                files.push(StatusEntry {
                    renamed_from: None,
                    path: PathBuf::from(rest),
                    index: None,
                    worktree: Some(FileStatus::Untracked),
                });
            }
            _ => {}
        }

        i += 1;
    }

    pair_worktree_renames(repo, &mut files);

    Ok(WorkingTree {
        files,
        ..WorkingTree::default()
    })
}

/// Pairs the working tree's deleted tracked files with its new ones
/// (`git status` leaves that as a deletion plus an untracked file): the
/// shared pairing of `gitpoulpi_core::domain::pair_unstaged_renames`, with
/// the old side read from the index in one `cat-file --batch`. Skipped when
/// either side is empty or over the rename limit, so a large untracked set
/// costs nothing.
fn pair_worktree_renames(repo: &RepoRef, files: &mut [StatusEntry]) {
    let paths: Vec<PathBuf> = files
        .iter()
        .filter(|e| is_unstaged_rename_source(e))
        .map(|e| e.path.clone())
        .collect();
    let any_new = files.iter().any(|e| {
        e.index.is_none()
            && matches!(
                e.worktree,
                Some(FileStatus::Untracked) | Some(FileStatus::Added)
            )
    });
    if paths.is_empty() || paths.len() > RENAME_LIMIT || !any_new {
        return;
    }
    let specs: Vec<String> = paths
        .iter()
        .map(|p| format!(":{}", p.to_string_lossy().replace('\\', "/")))
        .collect();
    let blobs = crate::process::cat_file_batch_lenient(&repo.root, &specs, RENAME_MAX_FILE_BYTES);
    let old_of: std::collections::HashMap<PathBuf, Vec<u8>> = paths
        .into_iter()
        .zip(blobs)
        .filter_map(|(p, b)| b.map(|b| (p, b)))
        .collect();
    pair_unstaged_renames(files, |path, side| match side {
        RenameSide::Old => old_of.get(path).cloned(),
        RenameSide::New => read_workdir_candidate(&repo.root, path),
    });
}
