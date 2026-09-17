use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{
    Blame, BlameContent, BlameLine, Blob, BlobBody, BlobBytes, BlobContent, BlobSource,
    ConflictEntry, ImageInfo, MergeDocument, Oid, RawStage, RepoRef, StageEntry, Worktree,
    MAX_TEXT_BLOB_BYTES, MODE_GITLINK,
};
#[cfg(feature = "submodule")]
use gitpoulpi_core::domain::{Submodule, SubmoduleState};
use gitpoulpi_core::{Error, Result};

use crate::repo::{map_err, open, to_domain_oid, to_git2_oid};

/// `git2::Repository::worktrees()` lists only *linked* worktrees, so the
/// main one is synthesized here from the common dir - see [`Worktree`]
/// for why the contract requires it to be present and first.
pub fn worktrees(repo: &RepoRef) -> Result<Vec<Worktree>> {
    let git_repo = open(repo)?;

    // The shared `.git` directory's parent is the main worktree, even
    // when `repo` points at a linked one.
    let common = common_git_dir(&git_repo);
    let main_path = common
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| common.clone());

    // Linked worktrees first (cheap admin-dir reads), then every branch
    // lookup (one `Repository::open` per worktree, the dominant cost) on
    // scoped threads, so N worktrees cost about one open.
    let mut linked: Vec<(PathBuf, bool)> = Vec::new();
    for name in git_repo.worktrees().map_err(map_err)?.iter().flatten() {
        let wt = match git_repo.find_worktree(name) {
            Ok(wt) => wt,
            // A worktree whose directory was deleted without
            // `git worktree prune` still has an admin entry; skip it
            // rather than failing the whole listing.
            Err(_) => continue,
        };
        let locked = matches!(wt.is_locked(), Ok(git2::WorktreeLockStatus::Locked(_)));
        linked.push((wt.path().to_path_buf(), locked));
    }

    let mut paths: Vec<&Path> = vec![main_path.as_path()];
    paths.extend(linked.iter().map(|(p, _)| p.as_path()));
    let branches: Vec<Option<String>> = std::thread::scope(|scope| {
        let handles: Vec<_> = paths
            .iter()
            .map(|p| scope.spawn(move || branch_at(p)))
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap_or(None))
            .collect()
    });
    let mut branches = branches.into_iter();

    let mut out = vec![Worktree {
        branch: branches.next().flatten(),
        path: main_path.clone(),
        is_main: true,
        is_locked: false,
    }];
    for ((path, is_locked), branch) in linked.into_iter().zip(branches) {
        out.push(Worktree {
            branch,
            path,
            is_main: false,
            is_locked,
        });
    }

    Ok(out)
}

/// `true` when the repository provably has no unmerged paths, so the
/// caller can skip the `git diff --diff-filter=U` process (100-300 ms on
/// Windows, paid by every refresh). `false` means "possibly": ask git.
pub fn provably_no_conflicts(repo: &RepoRef) -> bool {
    open(repo)
        .and_then(|r| r.index().map_err(map_err))
        .map(|index| !index.has_conflicts())
        .unwrap_or(false)
}

/// `true` when no rebase is in progress: neither `rebase-merge` nor
/// `rebase-apply` exists in the git directory (the first test
/// `SystemGitBackend::rebase_state` makes). Saves a `git rev-parse
/// --absolute-git-dir` spawn on every refresh.
pub fn provably_no_rebase(repo: &RepoRef) -> bool {
    open(repo)
        .map(|r| {
            let dir = r.path();
            !dir.join("rebase-merge").exists() && !dir.join("rebase-apply").exists()
        })
        .unwrap_or(false)
}

/// The shared `.git` directory. git2 0.19 exposes no `commondir()`, but
/// a linked worktree's own git dir (`<main>/.git/worktrees/<name>`)
/// contains a `commondir` file pointing back at the shared one; without
/// that file we already are the main repo.
fn common_git_dir(repo: &git2::Repository) -> PathBuf {
    let git_dir = repo.path();
    let common = gitpoulpi_core::paths::common_dir(git_dir);
    if common == git_dir {
        return common;
    }
    std::fs::canonicalize(&common).unwrap_or(common)
}

/// Short branch name checked out at `path`, or `None` when HEAD is
/// detached, unborn, or the path isn't readable as a repo.
fn branch_at(path: &std::path::Path) -> Option<String> {
    let repo = git2::Repository::open(path).ok()?;
    let head = repo.head().ok()?;
    if !head.is_branch() {
        return None;
    }
    head.shorthand().map(str::to_string)
}

/// See `GitBackend::read_blob`.
///
/// The size is learned from the object database's *header*
/// (`odb.read_header`), so a 400 MiB file is rejected without being
/// loaded; the working-tree case does the same with `fs::metadata`.
///
/// The working tree is read straight off disk, never through `git show
/// :path` or a checkout, which apply smudge filters and would return a
/// *filtered* rendering instead of the bytes an editor would open.
pub fn read_blob(repo: &RepoRef, path: &Path, source: &BlobSource) -> Result<Blob> {
    let rel = repo_relative(path)?;
    let (size, content, image) = match source {
        BlobSource::WorkingTree => {
            let full = repo.root.join(&rel);
            let size = std::fs::metadata(&full)
                .map_err(|e| Error::Backend(format!("{rel}: {e}")))?
                .len();
            if size > MAX_TEXT_BLOB_BYTES {
                (size, BlobContent::TooLarge, None)
            } else {
                let raw =
                    std::fs::read(&full).map_err(|e| Error::Backend(format!("{rel}: {e}")))?;
                let content = BlobContent::from_bytes(&raw);
                let image = Blob::image_of(&content, &raw);
                (size, content, image)
            }
        }
        _ => {
            let git_repo = open(repo)?;
            read_odb_blob(&git_repo, object_id(&git_repo, &rel, source)?)?
        }
    };

    Ok(Blob {
        path: PathBuf::from(rel),
        source: source.clone(),
        size,
        content,
        image,
    })
}

/// The object id of `rel` at an index or commit `source`.
fn object_id(git_repo: &git2::Repository, rel: &str, source: &BlobSource) -> Result<git2::Oid> {
    match source {
        BlobSource::WorkingTree => Err(Error::Backend("the working tree has no object".into())),
        BlobSource::Index => {
            let index = git_repo.index().map_err(map_err)?;
            // Stage 0 only: an unmerged path has stages 1/2/3 and no
            // single "the staged content", so it is an error here rather
            // than an arbitrary side silently winning.
            let entry = index
                .get_path(Path::new(rel), 0)
                .ok_or_else(|| Error::Backend(format!("{rel} is not in the index")))?;
            Ok(entry.id)
        }
        BlobSource::Stage(stage) => {
            let index = git_repo.index().map_err(map_err)?;
            let entry = (1..=3)
                .contains(stage)
                .then(|| index.get_path(Path::new(rel), i32::from(*stage)));
            entry
                .flatten()
                .map(|e| e.id)
                .ok_or_else(|| Error::Backend(format!("{rel} has no stage {stage}")))
        }
        BlobSource::Commit(oid) => {
            let commit = git_repo.find_commit(to_git2_oid(*oid)?).map_err(map_err)?;
            let tree = commit.tree().map_err(map_err)?;
            let entry = tree
                .get_path(Path::new(rel))
                .map_err(|_| Error::Backend(format!("{rel} is not in commit {oid}")))?;
            if entry.kind() != Some(git2::ObjectType::Blob) {
                return Err(Error::Backend(format!("{rel} is not a file at {oid}")));
            }
            Ok(entry.id())
        }
    }
}

/// See `GitBackend::read_blob_bytes`. Same size-before-content rule as
/// [`read_blob`]; an LFS pointer is resolved by the system-git backend's
/// shared helper so both backends answer alike.
pub fn read_blob_bytes(
    repo: &RepoRef,
    path: &Path,
    source: &BlobSource,
    max: u64,
) -> Result<BlobBytes> {
    let rel = repo_relative(path)?;
    let (size, raw) = match source {
        BlobSource::WorkingTree => {
            let full = repo.root.join(&rel);
            let size = std::fs::metadata(&full)
                .map_err(|e| Error::Backend(format!("{rel}: {e}")))?
                .len();
            let raw = if size > max {
                None
            } else {
                Some(std::fs::read(&full).map_err(|e| Error::Backend(format!("{rel}: {e}")))?)
            };
            (size, raw)
        }
        _ => {
            let git_repo = open(repo)?;
            let id = object_id(&git_repo, &rel, source)?;
            let (len, _kind) = git_repo
                .odb()
                .map_err(map_err)?
                .read_header(id)
                .map_err(map_err)?;
            let raw = if len as u64 > max {
                None
            } else {
                Some(git_repo.find_blob(id).map_err(map_err)?.content().to_vec())
            };
            (len as u64, raw)
        }
    };
    let body = match raw {
        None => BlobBody::TooLarge,
        Some(raw) => gitpoulpi_cli_backend::body_of(repo, raw, max),
    };
    Ok(BlobBytes {
        path: PathBuf::from(rel),
        source: source.clone(),
        size,
        body,
    })
}

/// Size-then-content for one object-database blob, so the size cap is
/// honoured without the content ever being read.
fn read_odb_blob(
    git_repo: &git2::Repository,
    id: git2::Oid,
) -> Result<(u64, BlobContent, Option<ImageInfo>)> {
    let odb = git_repo.odb().map_err(map_err)?;
    let (len, _kind) = odb.read_header(id).map_err(map_err)?;
    let size = len as u64;
    if size > MAX_TEXT_BLOB_BYTES {
        return Ok((size, BlobContent::TooLarge, None));
    }
    let blob = git_repo.find_blob(id).map_err(map_err)?;
    let content = BlobContent::from_bytes(blob.content());
    let image = Blob::image_of(&content, blob.content());
    Ok((size, content, image))
}

/// See `GitBackend::blame`.
///
/// The blob at `at` is read first and the result is built from *its*
/// lines, not libgit2's idea of the line count: blame is rendered beside
/// file content, so both must be split by the same code
/// (`BlobContent::from_bytes`), and a binary or oversized file must be
/// reported the same way by both backends.
pub fn blame(repo: &RepoRef, path: &Path, at: Oid) -> Result<Blame> {
    let rel = repo_relative(path)?;
    let blob = read_blob(repo, path, &BlobSource::Commit(at))?;
    let lines = match &blob.content {
        BlobContent::Text { lines, .. } => lines.clone(),
        BlobContent::Binary => {
            return Ok(Blame {
                path: blob.path,
                at,
                size: blob.size,
                content: BlameContent::Binary,
            })
        }
        BlobContent::TooLarge => {
            return Ok(Blame {
                path: blob.path,
                at,
                size: blob.size,
                content: BlameContent::TooLarge,
            })
        }
        BlobContent::LfsPointer(pointer) => {
            return Ok(Blame {
                path: blob.path,
                at,
                size: blob.size,
                content: BlameContent::LfsPointer(pointer.clone()),
            })
        }
    };

    let git_repo = open(repo)?;
    let mut opts = git2::BlameOptions::new();
    // Without `newest_commit` libgit2 blames HEAD's version of the file,
    // which would silently ignore `at` entirely and hand a File History
    // view the wrong lines for every commit but the tip.
    opts.newest_commit(to_git2_oid(at)?);
    let blame = git_repo
        .blame_file(Path::new(&rel), Some(&mut opts))
        .map_err(map_err)?;

    // One metadata lookup (and one `previous` computation) per *commit and
    // path*, not per line: a 5000-line file typically has a few dozen
    // distinct blamed commits.
    let mut metas: HashMap<(git2::Oid, String), CommitMeta> = HashMap::new();
    let mut out = Vec::with_capacity(lines.len());
    for (idx, content) in lines.into_iter().enumerate() {
        let final_line = idx as u32 + 1;
        let hunk = blame.get_line(final_line as usize).ok_or_else(|| {
            Error::Backend(format!("blame has no entry for {rel} line {final_line}"))
        })?;
        let commit_id = hunk.final_commit_id();
        let orig_path = hunk
            .path()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|| rel.clone());
        let key = (commit_id, orig_path.clone());
        if !metas.contains_key(&key) {
            let meta = commit_meta(&git_repo, commit_id, &orig_path)?;
            metas.insert(key.clone(), meta);
        }
        let meta = &metas[&key];
        let signature = hunk.final_signature();
        // A hunk covers a run of consecutive lines; this line's position
        // inside that run is the same in the original file.
        let offset = final_line.saturating_sub(hunk.final_start_line() as u32);
        out.push(BlameLine {
            commit: to_domain_oid(commit_id)?,
            author_name: signature.name().unwrap_or_default().to_string(),
            author_email: signature.email().unwrap_or_default().to_string(),
            timestamp: signature.when().seconds(),
            committer_time: meta.committer_time,
            summary: meta.summary.clone(),
            final_line,
            orig_line: hunk.orig_start_line() as u32 + offset,
            orig_path: PathBuf::from(orig_path),
            previous: meta.previous.clone(),
            content,
        });
    }

    Ok(Blame {
        path: blob.path,
        at,
        size: blob.size,
        content: BlameContent::Lines(out),
    })
}

/// What blame needs about one `(commit, path)`, computed once.
struct CommitMeta {
    summary: String,
    committer_time: i64,
    previous: Option<(Oid, PathBuf)>,
}

/// `previous` is the commit's **first parent** and the file's path there,
/// the same pair `git blame --porcelain` prints: the path itself when the
/// parent has it, else the old name when rename detection (50%) pairs the
/// two across the commit, else `None` (root commit, or the file is born
/// here).
fn commit_meta(repo: &git2::Repository, id: git2::Oid, path: &str) -> Result<CommitMeta> {
    let commit = repo.find_commit(id).map_err(map_err)?;
    let summary = commit.summary().unwrap_or_default().to_string();
    let committer_time = commit.committer().when().seconds();
    let Ok(parent) = commit.parent(0) else {
        return Ok(CommitMeta {
            summary,
            committer_time,
            previous: None,
        });
    };
    let parent_tree = parent.tree().map_err(map_err)?;
    let previous_path = if parent_tree.get_path(Path::new(path)).is_ok() {
        Some(path.to_string())
    } else {
        let tree = commit.tree().map_err(map_err)?;
        let mut diff = repo
            .diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)
            .map_err(map_err)?;
        let mut find = git2::DiffFindOptions::new();
        find.renames(true);
        diff.find_similar(Some(&mut find)).map_err(map_err)?;
        diff.deltas()
            .find(|d| {
                d.status() == git2::Delta::Renamed
                    && d.new_file()
                        .path()
                        .is_some_and(|p| p.to_string_lossy().replace('\\', "/") == path)
            })
            .and_then(|d| d.old_file().path())
            .map(|p| p.to_string_lossy().replace('\\', "/"))
    };
    Ok(CommitMeta {
        summary,
        committer_time,
        previous: match previous_path {
            Some(p) => Some((to_domain_oid(parent.id())?, PathBuf::from(p))),
            None => None,
        },
    })
}

/// Domain paths are repo-relative and git spells them with `/` on every
/// platform; a `PathBuf` join on Windows yields backslashes no tree lookup
/// matches. Normalizing here, not at each call site, is why there is no
/// `cfg(windows)` in this crate.
fn repo_relative(path: &Path) -> Result<String> {
    let text = path
        .to_str()
        .ok_or_else(|| Error::Backend(format!("path {} is not valid utf-8", path.display())))?;
    Ok(text.replace('\\', "/").trim_start_matches("./").to_string())
}

/// Every path at `at`, sorted, with `/` separators. Mirrors
/// `gitpoulpi_cli_backend::content::list_tree`; the contract suite holds
/// the two to the same answer.
pub fn list_tree(repo: &RepoRef, at: Oid) -> Result<Vec<PathBuf>> {
    let git = open(repo)?;
    let oid = to_git2_oid(at)?;
    // `find_commit` rather than `find_object` + peel: an oid that is not
    // a commit must be an error, not an empty listing, and the contract
    // suite pins that. The system-git twin gets it from `^{tree}`.
    let commit = git
        .find_commit(oid)
        .map_err(|e| Error::Backend(format!("no commit {}: {e}", at.to_hex())))?;
    let tree = commit
        .tree()
        .map_err(|e| Error::Backend(format!("commit {} has no tree: {e}", at.to_hex())))?;

    let mut paths: Vec<PathBuf> = Vec::new();
    // `walk` yields a directory entry *and* then its contents, so the
    // filter is what keeps directories out -- see the trait doc for why
    // they are not entries.
    tree.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
        if entry.kind() == Some(git2::ObjectType::Tree) {
            return git2::TreeWalkResult::Ok;
        }
        if let Some(name) = entry.name() {
            // `dir` is already `/`-separated and either empty or
            // `/`-terminated, so this concatenation is the whole conversion.
            paths.push(PathBuf::from(format!("{dir}{name}")));
        }
        git2::TreeWalkResult::Ok
    })
    .map_err(|e| Error::Backend(format!("walking the tree of {}: {e}", at.to_hex())))?;

    // Defensive, not load-bearing: git sorts tree entries comparing a
    // directory as if it ended in `/`, so a recursive walk already comes
    // out ordered by full path and removing this breaks no test. It stays
    // because the contract promises sorted output and should be enforced,
    // not coincidental.
    paths.sort();
    Ok(paths)
}

/// Every submodule `.gitmodules` declares, natively (no process): what is
/// declared, what the index records, and what is checked out. Agrees with
/// the system backend's `git submodule status` reading (the contract suite
/// holds them to it): not populated is `Uninitialized`, a checked-out
/// commit other than the recorded one is `Modified`.
#[cfg(feature = "submodule")]
pub fn submodules(repo: &RepoRef) -> Result<Vec<Submodule>> {
    if !repo.root.join(".gitmodules").is_file() {
        return Ok(Vec::new());
    }
    let git_repo = open(repo)?;
    let conflicted: std::collections::HashSet<String> = git_repo
        .index()
        .ok()
        .filter(|index| index.has_conflicts())
        .map(|index| {
            index
                .conflicts()
                .into_iter()
                .flatten()
                .flatten()
                .filter_map(|c| c.our.or(c.their).or(c.ancestor))
                .map(|e| String::from_utf8_lossy(&e.path).into_owned())
                .collect()
        })
        .unwrap_or_default();
    let mut out = Vec::new();
    for sm in git_repo.submodules().map_err(map_err)? {
        let path = sm.path().to_string_lossy().replace('\\', "/");
        let expected = sm.index_id().and_then(|id| to_domain_oid(id).ok());
        let head = sm.workdir_id().and_then(|id| to_domain_oid(id).ok());
        let state = if conflicted.contains(&path) {
            SubmoduleState::Conflicted
        } else if head.is_none() {
            SubmoduleState::Uninitialized
        } else if head != expected {
            SubmoduleState::Modified
        } else {
            SubmoduleState::UpToDate
        };
        out.push(Submodule {
            name: sm.name().unwrap_or(&path).to_string(),
            url: sm.url().unwrap_or_default().to_string(),
            branch: sm.branch().map(str::to_string),
            state,
            expected,
            head,
            path,
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn stage_entry(entry: &Option<git2::IndexEntry>) -> Result<Option<StageEntry>> {
    entry
        .as_ref()
        .map(|e| {
            Ok(StageEntry {
                mode: e.mode,
                oid: to_domain_oid(e.id)?,
            })
        })
        .transpose()
}

fn conflict_entry(c: &git2::IndexConflict) -> Result<Option<ConflictEntry>> {
    let Some(path) = c
        .our
        .as_ref()
        .or(c.their.as_ref())
        .or(c.ancestor.as_ref())
        .map(|e| String::from_utf8_lossy(&e.path).into_owned())
    else {
        return Ok(None);
    };
    Ok(ConflictEntry::from_stages(
        PathBuf::from(path),
        stage_entry(&c.ancestor)?,
        stage_entry(&c.our)?,
        stage_entry(&c.their)?,
    ))
}

/// See `GitBackend::conflict_entries`: libgit2 reads the index's unmerged
/// entries, no process and no blob. Sorted by path, as `git ls-files -u`
/// lists them.
pub fn conflict_entries(repo: &RepoRef) -> Result<Vec<ConflictEntry>> {
    let git_repo = open(repo)?;
    let index = git_repo.index().map_err(map_err)?;
    if !index.has_conflicts() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for conflict in index.conflicts().map_err(map_err)? {
        let conflict = conflict.map_err(map_err)?;
        out.extend(conflict_entry(&conflict)?);
    }
    out.sort_by(|a, b| a.path.as_os_str().cmp(b.path.as_os_str()));
    Ok(out)
}

/// See `GitBackend::merge_document`: the stages are read natively (header
/// first, so an oversized blob is never loaded); the merge itself is
/// `CliFallback` (libgit2 0.19's binding has no `git_merge_file`, and its
/// xdiff could place regions differently than git's), through the same
/// code `SystemGitBackend` runs.
pub fn merge_document(repo: &RepoRef, path: &Path) -> Result<MergeDocument> {
    let rel = repo_relative(path)?;
    let git_repo = open(repo)?;
    let index = git_repo.index().map_err(map_err)?;
    let conflict = git2::IndexConflict {
        ancestor: index.get_path(Path::new(&rel), 1),
        our: index.get_path(Path::new(&rel), 2),
        their: index.get_path(Path::new(&rel), 3),
    };
    let entry = conflict_entry(&conflict)?
        .ok_or_else(|| Error::Backend(format!("{rel} is not conflicted")))?;
    let odb = git_repo.odb().map_err(map_err)?;
    let read = |stage: Option<StageEntry>| -> Result<Option<RawStage>> {
        let Some(stage) = stage else { return Ok(None) };
        if stage.mode == MODE_GITLINK {
            return Ok(Some(RawStage {
                entry: stage,
                size: 0,
                bytes: None,
            }));
        }
        let id = to_git2_oid(stage.oid)?;
        let (len, _kind) = odb.read_header(id).map_err(map_err)?;
        let size = len as u64;
        let bytes = if size > MAX_TEXT_BLOB_BYTES {
            None
        } else {
            Some(git_repo.find_blob(id).map_err(map_err)?.content().to_vec())
        };
        Ok(Some(RawStage {
            entry: stage,
            size,
            bytes,
        }))
    };
    let raws = [read(entry.base)?, read(entry.ours)?, read(entry.theirs)?];
    gitpoulpi_cli_backend::document_from_stages(repo, entry, raws)
}
