//! `GitBackend::predict_merge_conflicts` over `merge-tree --write-tree`.

use gitpoulpi_core::domain::{Oid, RepoRef};
use gitpoulpi_core::{Error, Result};

use crate::process::{git, git_at_least, spawn_failed};

/// The first git that has `merge-tree --write-tree` (a real merge, in
/// memory, that names the conflicted files).
const MIN_GIT: (u32, u32) = (2, 38);

pub fn predict_merge_conflicts(repo: &RepoRef, ours: Oid, theirs: Oid) -> Result<Vec<String>> {
    if !git_at_least(MIN_GIT.0, MIN_GIT.1) {
        return Err(Error::Unsupported {
            operation: "predict_merge_conflicts (needs git 2.38 or later)",
        });
    }
    let (ours, theirs) = (ours.to_hex(), theirs.to_hex());
    let args = [
        "merge-tree",
        "--write-tree",
        "--name-only",
        "-z",
        ours.as_str(),
        theirs.as_str(),
    ];
    let output = git(&repo.root, &args).output().map_err(spawn_failed)?;
    // 0: clean, 1: conflicts (the files are listed); anything else failed.
    match output.status.code() {
        Some(0) => Ok(Vec::new()),
        Some(1) => Ok(parse(&output.stdout)),
        _ => Err(crate::process::failed(
            &args,
            &String::from_utf8_lossy(&output.stderr),
        )),
    }
}

/// `<tree oid>` then the conflicted names, each NUL-ended, then an empty
/// field and the informational messages (`-z`, no quoting).
fn parse(stdout: &[u8]) -> Vec<String> {
    let mut fields = stdout.split(|b| *b == 0);
    // The first field is the tree's id (with `-z` it ends in NUL too).
    let first = fields.next().unwrap_or_default();
    let mut names = Vec::new();
    // Older output ends the id with a newline instead: names may follow it.
    if let Some(rest) = first.split(|b| *b == b'\n').nth(1) {
        if !rest.is_empty() {
            names.push(String::from_utf8_lossy(rest).into_owned());
        }
    }
    for field in fields {
        if field.is_empty() {
            break;
        }
        names.push(String::from_utf8_lossy(field).into_owned());
    }
    names.sort();
    names.dedup();
    names
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn it_reads_the_names_up_to_the_empty_field() {
        let out = b"0123456789012345678901234567890123456789\0a.txt\0dir/b c.txt\0\0Auto-merging a.txt\nCONFLICT (content): x\0";
        assert_eq!(parse(out), vec!["a.txt", "dir/b c.txt"]);
        assert!(parse(b"0123456789012345678901234567890123456789\0").is_empty());
    }
}
