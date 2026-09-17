use std::path::PathBuf;

use gitpoulpi_core::domain::{
    CommitStat, DiffFileList, DiffTarget, FileDiff, FileStatus, Oid, RepoRef,
};
use gitpoulpi_core::Result;

use crate::process::{run, run_bytes};

/// `git <args>`' stdout as text for a person to read: UTF-8, or decoded with
/// the fallback encoding in force (`git.default_encoding`, see
/// `gitpoulpi_core::domain::with_fallback_encoding`). Otherwise the same
/// "not utf-8" error `run` gives.
fn run_text(root: &std::path::Path, args: &[&str]) -> Result<String> {
    let bytes = run_bytes(root, args)?;
    gitpoulpi_core::domain::decode_text(&bytes)
        .map(std::borrow::Cow::into_owned)
        .ok_or_else(|| gitpoulpi_core::Error::Backend("git output was not utf-8".to_string()))
}

/// The canonical empty-tree object, used to diff a commit with no parent
/// (e.g. a repo's initial commit) against "nothing" instead of special-
/// casing that in every diff target below.
const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

pub fn diff(repo: &RepoRef, target: DiffTarget) -> Result<Vec<FileDiff>> {
    diff_inner(repo, target, None)
}

/// `diff` restricted to literal `paths`: `-- :(literal)p ...`.
/// Empty `paths` answers nothing without spawning git.
pub fn diff_paths(repo: &RepoRef, target: DiffTarget, paths: &[PathBuf]) -> Result<Vec<FileDiff>> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    diff_inner(repo, target, Some(paths))
}

/// The `-- :(literal)p ...` tail restricting a `git diff` to `paths`.
fn pathspec_tail(paths: Option<&[PathBuf]>) -> Vec<String> {
    let Some(paths) = paths else {
        return Vec::new();
    };
    let mut tail = vec!["--".to_string()];
    tail.extend(
        paths
            .iter()
            .map(|p| format!(":(literal){}", p.to_string_lossy().replace('\\', "/"))),
    );
    tail
}

fn diff_inner(
    repo: &RepoRef,
    target: DiffTarget,
    paths: Option<&[PathBuf]>,
) -> Result<Vec<FileDiff>> {
    let tail = pathspec_tail(paths);
    let args: Vec<String> = match target {
        DiffTarget::WorkingTreeVsIndex => vec![
            "diff".into(),
            "--no-color".into(),
            "--no-ext-diff".into(),
            "--no-textconv".into(),
            "-U3".into(),
        ],
        DiffTarget::IndexVsHead => {
            vec![
                "diff".into(),
                "--no-color".into(),
                "--no-ext-diff".into(),
                "--no-textconv".into(),
                "-U3".into(),
                "--cached".into(),
            ]
        }
        DiffTarget::CommitVsParent(oid) => {
            // One process, not two (a spawn is the expensive part on
            // Windows): `<hex>^` goes straight into the diff. Only a commit
            // without a parent (a root commit, a shallow clone's boundary)
            // fails that, and it is retried against the empty tree.
            let hex = oid.to_hex();
            let parent = format!("{hex}^");
            let first = with_tail(commit_diff_args(&parent, &hex), &tail);
            let first: Vec<&str> = first.iter().map(String::as_str).collect();
            match run_text(&repo.root, &first) {
                Ok(raw) => return Ok(gitpoulpi_core::diff_text::parse_unified(&raw)),
                Err(_) => with_tail(commit_diff_args(EMPTY_TREE, &hex), &tail),
            }
        }
        DiffTarget::CommitRange(from, to) => {
            compare_args(&[from.to_hex().as_str(), to.to_hex().as_str()])
        }
        DiffTarget::CommitVsWorkingTree(oid) => compare_args(&[oid.to_hex().as_str()]),
        DiffTarget::CommitSpan(oldest, newest) => {
            // Same one-process shape as `CommitVsParent`: the parent goes
            // in as `<hex>^`, and only a root commit's failure is retried
            // against the empty tree.
            let (oldest, newest) = (oldest.to_hex(), newest.to_hex());
            let parent = format!("{oldest}^");
            let first = with_tail(compare_args(&[parent.as_str(), newest.as_str()]), &tail);
            let first: Vec<&str> = first.iter().map(String::as_str).collect();
            match run_text(&repo.root, &first) {
                Ok(raw) => return Ok(gitpoulpi_core::diff_text::parse_unified(&raw)),
                Err(_) => compare_args(&[EMPTY_TREE, newest.as_str()]),
            }
        }
    };
    let mut args = args;
    if !matches!(target, DiffTarget::CommitVsParent(_)) {
        args.extend(tail);
    }
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();

    let raw = run_text(&repo.root, &args_ref)?;
    Ok(gitpoulpi_core::diff_text::parse_unified(&raw))
}

/// `git diff` of `new` against `old`, in the exact form every other
/// target here uses.
fn commit_diff_args<'a>(old: &'a str, new: &'a str) -> [&'a str; 7] {
    [
        "diff",
        "--no-color",
        "--no-ext-diff",
        "--no-textconv",
        "-U3",
        old,
        new,
    ]
}

fn with_tail(args: impl IntoIterator<Item = impl ToString>, tail: &[String]) -> Vec<String> {
    let mut all: Vec<String> = args.into_iter().map(|a| a.to_string()).collect();
    all.extend(tail.iter().cloned());
    all
}

/// The system diff between arbitrary revisions (or one revision and the
/// working tree). `--no-renames`: a rename is a delete plus an add, as
/// libgit2's plain tree diff lists it, so both backends answer alike
/// whatever the user's `diff.renames` says.
fn compare_args(revs: &[&str]) -> Vec<String> {
    let mut args: Vec<String> = [
        "diff",
        "--no-color",
        "--no-ext-diff",
        "--no-textconv",
        "--no-renames",
        "-U3",
    ]
    .iter()
    .map(|a| a.to_string())
    .collect();
    args.extend(revs.iter().map(|r| r.to_string()));
    args
}

/// The system-git `commit_stats`: ONE `git log --no-walk --numstat`
/// process for the whole batch (not one per commit). `--root` shows a root
/// commit against nothing, `--diff-merges=first-parent` a merge against
/// its first parent, `--no-renames` keeps a rename as two files like
/// libgit2's plain tree diff. Binary files print `-\t-` and count as a
/// file with no lines.
pub fn commit_stats(repo: &RepoRef, ids: &[Oid]) -> Result<Vec<CommitStat>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let hexes: Vec<String> = ids.iter().map(|o| o.to_hex()).collect();
    let mut args: Vec<&str> = vec![
        "log",
        "--no-walk=unsorted",
        "--no-color",
        "--no-ext-diff",
        "--no-textconv",
        "--no-color",
        "--root",
        "--diff-merges=first-parent",
        "--numstat",
        "--format=%x01%H",
    ];
    args.extend(hexes.iter().map(String::as_str));
    let raw = run(&repo.root, &args)?;
    let mut by_id: std::collections::HashMap<String, (u32, u32, u32)> =
        std::collections::HashMap::new();
    for chunk in raw.split('\u{1}').skip(1) {
        let mut lines = chunk.lines();
        let Some(hash) = lines.next() else { continue };
        let (mut files, mut ins, mut del) = (0, 0, 0);
        for line in lines {
            let mut cols = line.splitn(3, '\t');
            let (Some(a), Some(d), Some(_)) = (cols.next(), cols.next(), cols.next()) else {
                continue;
            };
            files += 1;
            ins += a.parse::<u32>().unwrap_or(0);
            del += d.parse::<u32>().unwrap_or(0);
        }
        by_id.insert(hash.trim().to_string(), (files, ins, del));
    }
    Ok(ids
        .iter()
        .zip(&hexes)
        .map(|(&id, hex)| {
            let (files, insertions, deletions) = by_id.get(hex).copied().unwrap_or_default();
            CommitStat {
                id,
                files,
                insertions,
                deletions,
            }
        })
        .collect())
}

/// The system-git `diff_files`: `git diff --name-status -z`, one process
/// that reads no blob and prints no hunk. Status letters map the way
/// `parse_unified` maps its header lines (`A` added, `D` deleted,
/// `R` renamed - reported under its new path - anything else modified),
/// in git's own order.
pub fn diff_files(repo: &RepoRef, target: DiffTarget) -> Result<DiffFileList> {
    let base = [
        "diff",
        "--no-color",
        "--no-ext-diff",
        "--no-textconv",
        "--name-status",
        "-z",
    ];
    let with = |extra: &[&str]| -> Vec<String> {
        base.iter()
            .chain(extra.iter())
            .map(|a| a.to_string())
            .collect()
    };
    let run_owned = |args: Vec<String>| {
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        run(&repo.root, &refs)
    };
    let raw = match target {
        DiffTarget::WorkingTreeVsIndex => run_owned(with(&[]))?,
        DiffTarget::IndexVsHead => run_owned(with(&["--cached"]))?,
        DiffTarget::CommitVsParent(oid) => {
            let hex = oid.to_hex();
            match run_owned(with(&[&format!("{hex}^"), &hex])) {
                Ok(raw) => raw,
                // No parent: against the empty tree, as `diff` does.
                Err(_) => run_owned(with(&[EMPTY_TREE, &hex]))?,
            }
        }
        DiffTarget::CommitRange(from, to) => {
            run_owned(with(&["--no-renames", &from.to_hex(), &to.to_hex()]))?
        }
        DiffTarget::CommitVsWorkingTree(oid) => run_owned(with(&["--no-renames", &oid.to_hex()]))?,
        DiffTarget::CommitSpan(oldest, newest) => {
            let (oldest, newest) = (oldest.to_hex(), newest.to_hex());
            match run_owned(with(&["--no-renames", &format!("{oldest}^"), &newest])) {
                Ok(raw) => raw,
                Err(_) => run_owned(with(&["--no-renames", EMPTY_TREE, &newest]))?,
            }
        }
    };
    Ok(DiffFileList {
        files: parse_name_status(&raw),
        hunks_included: false,
    })
}

/// Parses `--name-status -z`: `<status>\0<path>\0`, with `R`/`C` followed
/// by two paths (old, new).
fn parse_name_status(raw: &str) -> Vec<FileDiff> {
    let mut files = Vec::new();
    let mut fields = raw.split('\0');
    while let Some(code) = fields.next() {
        if code.is_empty() {
            continue;
        }
        let renamed = code.starts_with('R') || code.starts_with('C');
        let status = match code.chars().next() {
            Some('A') => FileStatus::Added,
            Some('D') => FileStatus::Deleted,
            Some('R') | Some('C') => FileStatus::Renamed,
            _ => FileStatus::Modified,
        };
        let mut path = fields.next().unwrap_or_default();
        if renamed {
            path = fields.next().unwrap_or(path);
        }
        files.push(FileDiff {
            path: PathBuf::from(path),
            status,
            hunks: Vec::new(),
        });
    }
    files
}
