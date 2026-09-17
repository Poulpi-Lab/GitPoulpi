//! `GitBackend::{split_source, split_commit}` with plumbing (see
//! the split-commit design): one commit becomes N commits
//! built with `commit-tree` in a throwaway index, the last carrying the
//! original tree verbatim, so the descendants are re-created with their own
//! trees and one `update-ref` moves HEAD. No rebase runs, so nothing can
//! conflict, and the real index and the working tree are never touched.
//!
//! The numbering of hunks comes from one `diff-tree --no-renames` with
//! pinned prefixes, never from `diff(CommitVsParent)`, which honours
//! `diff.renames`.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use gitpoulpi_core::domain::{
    normalize_reword_message, validate_split, DiffLine, FileStatus, Hunk, HunkId, LineOrigin, Oid,
    RepoRef, SplitCommit, SplitFile, SplitOutcome, SplitPiece, SplitRefusal, SplitSource,
    WholeReason,
};
use gitpoulpi_core::{Error, Result};

use crate::patch::{parse_hunks, render_hunks, select_lines, Direction, PatchHunk};
use crate::process::{run, run_bytes, run_with_env_and_stdin, run_with_stdin};
use crate::rewrite::{
    commit_tree, descendants, move_head, operation_in_progress, probe, read_commits, replay,
    CommitObject,
};

/// One file of the commit, with what building a tree from it needs.
struct LoadedFile {
    path: String,
    status: FileStatus,
    whole: Option<WholeReason>,
    /// The mode and blob after the commit, as `diff-tree --raw` printed
    /// them (all zeros for a deletion) - exactly an `--index-info` line.
    new_mode: String,
    new_sha: String,
    hunks: Vec<PatchHunk>,
}

struct Loaded {
    source: SplitSource,
    commit: CommitObject,
    files: Vec<LoadedFile>,
}

pub fn split_source(repo: &RepoRef, commit: Oid) -> Result<SplitSource> {
    Ok(load(&repo.root, commit)?.source)
}

fn refused(r: SplitRefusal) -> Error {
    Error::SplitRefused(r)
}

/// The commit's changes, numbered with `--no-renames`.
fn load(root: &Path, commit: Oid) -> Result<Loaded> {
    let hex = commit.to_hex();
    let object = read_commits(root, std::slice::from_ref(&hex))?
        .pop()
        .ok_or_else(|| Error::Backend(format!("{hex} is not a commit")))?;
    if object.parents.len() > 1 {
        return Err(refused(SplitRefusal::MergeCommit));
    }

    let raw = run(
        root,
        &[
            "diff-tree",
            "-r",
            "--root",
            "--no-commit-id",
            "--no-renames",
            "--ignore-submodules=none",
            "-z",
            "--raw",
            &hex,
        ],
    )?;
    let entries = parse_raw(&raw)?;
    if entries.is_empty() {
        return Err(refused(SplitRefusal::EmptyCommit));
    }
    let patch = run_bytes(
        root,
        &[
            "diff-tree",
            "-p",
            "--root",
            "--no-commit-id",
            "--no-renames",
            "--no-color",
            "--no-ext-diff",
            "--no-textconv",
            "--ignore-submodules=none",
            "-U3",
            "--src-prefix=a/",
            "--dst-prefix=b/",
            &hex,
        ],
    )?;
    // A patch that is not UTF-8 text becomes lossy here; building a part
    // from it then fails in `git apply` before any ref moves.
    let patch = String::from_utf8_lossy(&patch);
    let blocks = split_blocks(&patch);

    let mut files = Vec::with_capacity(entries.len());
    let mut next_block = 0;
    for entry in entries {
        // A type change is printed as a delete plus an add.
        let width = if entry.status == 'T' { 2 } else { 1 };
        let block = blocks.get(next_block).copied().unwrap_or_default();
        next_block += width;
        files.push(classify(entry, block));
    }
    if next_block != blocks.len() {
        return Err(Error::Backend(format!(
            "diff-tree listed {next_block} files but printed {} patches",
            blocks.len()
        )));
    }
    mark_lfs(root, &mut files)?;

    let parent = object.parents.first().and_then(|p| Oid::from_hex(p));
    let source = SplitSource {
        commit,
        parent,
        message: object.message.clone(),
        files: files.iter().map(to_split_file).collect(),
        sign_default: effective_gpg_sign(root),
    };
    Ok(Loaded {
        source,
        commit: object,
        files,
    })
}

/// `commit.gpgSign` as git resolves it (local, global, system; `true`, `yes`,
/// `on` and `1` all count), `false` when unset or unreadable.
fn effective_gpg_sign(root: &Path) -> bool {
    run(root, &["config", "--type=bool", "--get", "commit.gpgSign"])
        .map(|v| v.trim() == "true")
        .unwrap_or(false)
}

struct RawEntry {
    old_mode: String,
    new_mode: String,
    new_sha: String,
    status: char,
    path: String,
}

/// `diff-tree -z --raw`: `:<om> <nm> <os> <ns> <status>\0<path>\0` per file.
fn parse_raw(raw: &str) -> Result<Vec<RawEntry>> {
    let mut fields = raw.split('\0').filter(|f| !f.is_empty());
    let mut out = Vec::new();
    while let Some(meta) = fields.next() {
        let bad = || Error::Backend(format!("unreadable diff-tree line: {meta}"));
        let mut parts = meta.strip_prefix(':').ok_or_else(bad)?.split(' ');
        let old_mode = parts.next().ok_or_else(bad)?;
        let new_mode = parts.next().ok_or_else(bad)?;
        let _old_sha = parts.next().ok_or_else(bad)?;
        let new_sha = parts.next().ok_or_else(bad)?;
        let status = parts
            .next()
            .and_then(|s| s.chars().next())
            .ok_or_else(bad)?;
        let path = fields.next().ok_or_else(bad)?;
        out.push(RawEntry {
            old_mode: old_mode.to_string(),
            new_mode: new_mode.to_string(),
            new_sha: new_sha.to_string(),
            status,
            path: path.to_string(),
        });
    }
    Ok(out)
}

/// One `&str` per `diff --git` block of a multi-file patch. A content line
/// always starts with ` `, `+`, `-` or `\`, so a header cannot be forged.
fn split_blocks(patch: &str) -> Vec<&str> {
    let mut starts: Vec<usize> = Vec::new();
    let mut offset = 0;
    for line in patch.split_inclusive('\n') {
        if line.starts_with("diff --git ") {
            starts.push(offset);
        }
        offset += line.len();
    }
    starts.push(patch.len());
    starts.windows(2).map(|w| &patch[w[0]..w[1]]).collect()
}

fn classify(entry: RawEntry, block: &str) -> LoadedFile {
    let is_binary = block
        .lines()
        .any(|l| l.starts_with("Binary files ") || l == "GIT binary patch");
    let touches = |mode: &str| entry.old_mode == mode || entry.new_mode == mode;
    let hunks = if entry.status == 'M' && !is_binary {
        parse_hunks(block)
    } else {
        Vec::new()
    };
    let whole = if touches("160000") {
        Some(WholeReason::Submodule)
    } else if entry.status == 'A' {
        Some(WholeReason::Added)
    } else if entry.status == 'D' {
        Some(WholeReason::Deleted)
    } else if entry.status != 'M' || is_binary || touches("120000") {
        // A type change or a symlink has no text worth cutting up.
        Some(WholeReason::Binary)
    } else if entry.old_mode != entry.new_mode || hunks.is_empty() {
        Some(WholeReason::ModeOnly)
    } else {
        None
    };
    let status = match entry.status {
        'A' => FileStatus::Added,
        'D' => FileStatus::Deleted,
        _ => FileStatus::Modified,
    };
    LoadedFile {
        path: entry.path,
        status,
        whole,
        new_mode: entry.new_mode,
        new_sha: entry.new_sha,
        hunks: if whole.is_some() { Vec::new() } else { hunks },
    }
}

/// Text files that `filter=lfs` owns are whole-only: their "text" is a
/// pointer. One `check-attr` for all candidates.
fn mark_lfs(root: &Path, files: &mut [LoadedFile]) -> Result<()> {
    let candidates: Vec<&str> = files
        .iter()
        .filter(|f| f.whole.is_none())
        .map(|f| f.path.as_str())
        .collect();
    if candidates.is_empty() {
        return Ok(());
    }
    let input: String = candidates.iter().map(|p| format!("{p}\0")).collect();
    let out = run_with_stdin(root, &["check-attr", "-z", "--stdin", "filter"], &input)?;
    let mut fields = out.split('\0');
    let mut lfs: BTreeSet<String> = BTreeSet::new();
    while let (Some(path), Some(_attr), Some(value)) = (fields.next(), fields.next(), fields.next())
    {
        if value == "lfs" {
            lfs.insert(path.to_string());
        }
    }
    for file in files.iter_mut().filter(|f| lfs.contains(&f.path)) {
        file.whole = Some(WholeReason::Lfs);
        file.hunks.clear();
    }
    Ok(())
}

fn to_split_file(file: &LoadedFile) -> SplitFile {
    SplitFile {
        path: PathBuf::from(&file.path),
        status: file.status,
        whole_only: file.whole,
        hunks: file
            .hunks
            .iter()
            .enumerate()
            .map(|(i, h)| Hunk {
                id: HunkId(i as u32),
                old_start: h.old_start,
                old_lines: h.old_lines,
                new_start: h.new_start,
                new_lines: h.new_lines,
                lines: h
                    .lines
                    .iter()
                    .map(|l| DiffLine {
                        origin: l.origin.clone(),
                        content: l.text.clone(),
                        staged: false,
                    })
                    .collect(),
            })
            .collect(),
    }
}

/// What parts 1..=k hold so far: whole files, and the changed lines picked
/// in each hunk. Cumulative, so every tree is the parent plus a selection
/// on the parent's own preimage.
#[derive(Default, Clone)]
struct Selection {
    files: BTreeSet<String>,
    lines: BTreeMap<(String, u32), BTreeSet<u32>>,
}

impl Selection {
    fn add(&mut self, loaded: &Loaded, piece: &SplitPiece) {
        let key = |p: &Path| p.to_string_lossy().into_owned();
        match piece {
            SplitPiece::File { path } => {
                let path = key(path);
                match loaded.files.iter().find(|f| f.path == path) {
                    // A text file: every changed line of every hunk.
                    Some(file) if !file.hunks.is_empty() => {
                        for (i, h) in file.hunks.iter().enumerate() {
                            let changed = h
                                .lines
                                .iter()
                                .enumerate()
                                .filter(|(_, l)| l.origin != LineOrigin::Context)
                                .map(|(n, _)| n as u32);
                            self.lines
                                .entry((path.clone(), i as u32))
                                .or_default()
                                .extend(changed);
                        }
                    }
                    _ => {
                        self.files.insert(path);
                    }
                }
            }
            SplitPiece::Hunk { path, hunk } => {
                let path = key(path);
                let Some(file) = loaded.files.iter().find(|f| f.path == path) else {
                    return;
                };
                if let Some(h) = file.hunks.get(hunk.0 as usize) {
                    let changed = h
                        .lines
                        .iter()
                        .enumerate()
                        .filter(|(_, l)| l.origin != LineOrigin::Context)
                        .map(|(i, _)| i as u32);
                    self.lines
                        .entry((path, hunk.0))
                        .or_default()
                        .extend(changed);
                }
            }
            SplitPiece::Lines { path, hunk, lines } => {
                self.lines
                    .entry((key(path), hunk.0))
                    .or_default()
                    .extend(lines.iter().copied());
            }
        }
    }
}

/// The temporary index, deleted however the split ends. Lives in the git
/// dir of the worktree itself, never the real index.
struct TempIndex(PathBuf);

impl TempIndex {
    fn new(git_dir: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(Path::new(git_dir).join(format!("gitpoulpi-split-{}-{n}.index", std::process::id())))
    }

    fn path(&self) -> String {
        self.0.to_string_lossy().into_owned()
    }
}

impl Drop for TempIndex {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
        let mut lock = self.0.clone().into_os_string();
        lock.push(".lock");
        let _ = std::fs::remove_file(lock);
    }
}

/// The tree of "parent plus `selection`", built in the temporary index.
fn tree_of(
    root: &Path,
    index: &TempIndex,
    loaded: &Loaded,
    selection: &Selection,
) -> Result<String> {
    let index_path = index.path();
    let env = [("GIT_INDEX_FILE", index_path.as_str())];
    let base = loaded.commit.parents.first();
    let read_tree: Vec<&str> = match base {
        Some(parent) => vec!["read-tree", parent.as_str()],
        None => vec!["read-tree", "--empty"],
    };
    run_with_env_and_stdin(root, &read_tree, &env, "")?;

    let mut patch = String::new();
    for file in &loaded.files {
        let mut cut: Vec<PatchHunk> = Vec::new();
        for (i, hunk) in file.hunks.iter().enumerate() {
            let Some(picked) = selection.lines.get(&(file.path.clone(), i as u32)) else {
                continue;
            };
            let indices: Vec<u32> = picked.iter().copied().collect();
            let part = select_lines(hunk, &indices, Direction::Forward);
            if part.has_changes() {
                cut.push(part);
            }
        }
        if !cut.is_empty() {
            patch.push_str(&render_hunks(&file.path, &cut));
        }
    }
    if !patch.is_empty() {
        run_with_env_and_stdin(
            root,
            &["apply", "--cached", "--whitespace=nowarn", "-"],
            &env,
            &patch,
        )?;
    }

    let info: String = loaded
        .files
        .iter()
        .filter(|f| selection.files.contains(&f.path))
        .map(|f| format!("{} {} 0\t{}\0", f.new_mode, f.new_sha, f.path))
        .collect();
    if !info.is_empty() {
        run_with_env_and_stdin(root, &["update-index", "-z", "--index-info"], &env, &info)?;
    }
    let tree = run_with_env_and_stdin(root, &["write-tree"], &env, "")?;
    Ok(tree.trim().to_string())
}

pub fn split_commit(repo: &RepoRef, req: &SplitCommit) -> Result<SplitOutcome> {
    let root = repo.root.as_path();
    let (git_dir, head) = probe(root)?;
    if operation_in_progress(&git_dir) {
        return Err(refused(SplitRefusal::OperationInProgress));
    }
    let to_replay =
        descendants(root, req.commit, head)?.ok_or(refused(SplitRefusal::NotOnCurrentBranch))?;
    let loaded = load(root, req.commit)?;
    validate_split(&loaded.source, &req.parts).map_err(refused)?;
    let messages: Vec<String> = req
        .parts
        .iter()
        .map(|p| normalize_reword_message(&p.message).ok_or(refused(SplitRefusal::EmptyMessage)))
        .collect::<Result<_>>()?;

    let target = req.commit.to_hex();
    let sign_flag = if req.sign { "-S" } else { "--no-gpg-sign" };
    let as_signing_failure = |e: Error| {
        if req.sign {
            refused(SplitRefusal::SigningFailed)
        } else {
            e
        }
    };

    let index = TempIndex::new(&git_dir);
    let mut created: Vec<String> = Vec::with_capacity(req.parts.len());
    let mut previous: Option<String> = loaded.commit.parents.first().cloned();
    let mut selection = Selection::default();
    let last = req.parts.len() - 1;
    for (k, (part, message)) in req.parts.iter().zip(&messages).enumerate() {
        // Part N is the original tree, verbatim.
        let tree = if k == last {
            loaded.commit.tree.clone()
        } else {
            for piece in &part.pieces {
                selection.add(&loaded, piece);
            }
            tree_of(root, &index, &loaded, &selection)?
        };
        let mut args = vec!["commit-tree", sign_flag, tree.as_str()];
        if let Some(parent) = &previous {
            args.push("-p");
            args.push(parent);
        }
        let new = commit_tree(root, &args, &loaded.commit, message).map_err(as_signing_failure)?;
        previous = Some(new.clone());
        created.push(new);
    }
    drop(index);

    let mut rewritten: HashMap<String, String> = HashMap::new();
    rewritten.insert(target.clone(), created[last].clone());
    if !to_replay.is_empty() {
        let objects = read_commits(root, &to_replay)?;
        replay(root, &to_replay, &objects, &mut rewritten, req.sign).map_err(as_signing_failure)?;
    }
    let head_hex = head.to_hex();
    let new_head = rewritten[&head_hex].clone();

    let log_message = format!("split: {} into {} commits", &target[..7], req.parts.len());
    move_head(root, &new_head, &head_hex, &log_message)?;

    let to_oid = |hex: &String| {
        Oid::from_hex(hex).ok_or_else(|| Error::Backend(format!("not an oid: {hex}")))
    };
    Ok(SplitOutcome {
        created: created.iter().map(to_oid).collect::<Result<_>>()?,
        previous_head: head,
        new_head: to_oid(&new_head)?,
        replayed: to_replay.len(),
    })
}
