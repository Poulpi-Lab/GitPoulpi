mod files;
mod messages;
mod refs;

#[cfg(feature = "submodule")]
pub use files::submodules;
pub use files::{
    blame, conflict_entries, list_tree, merge_document, provably_no_conflicts, provably_no_rebase,
    read_blob, read_blob_bytes, worktrees,
};
pub use messages::commit_messages;
pub use refs::{ahead_behind, merge_base, refs, remotes, resolve, stashes};

use std::cell::RefCell;
use std::path::PathBuf;

use gitpoulpi_core::domain::{
    commit_summary, decode_commit_text, CommitGraph, CommitNode, CommitStat, DiffFileList,
    DiffLine, DiffTarget, FileDiff, FileStatus, Hunk, HunkId, LineOrigin, LogOptions, Oid, RepoRef,
    StatusEntry, WorkingTree,
};
use gitpoulpi_core::Result;

use crate::repo::{map_err, open, to_domain_oid, to_git2_oid};

pub fn status(repo: &RepoRef) -> Result<WorkingTree> {
    let git_repo = open(repo)?;
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(true);
    let statuses = git_repo.statuses(Some(&mut opts)).map_err(map_err)?;

    let files = statuses
        .iter()
        .filter_map(|entry| {
            let path = entry.path().ok().map(PathBuf::from)?;
            let s = entry.status();

            let index = if s.contains(git2::Status::CONFLICTED) {
                Some(FileStatus::Conflicted)
            } else if s.contains(git2::Status::INDEX_NEW) {
                Some(FileStatus::Added)
            } else if s.contains(git2::Status::INDEX_MODIFIED) {
                Some(FileStatus::Modified)
            } else if s.contains(git2::Status::INDEX_DELETED) {
                Some(FileStatus::Deleted)
            } else if s.contains(git2::Status::INDEX_RENAMED) {
                Some(FileStatus::Renamed)
            } else {
                None
            };

            let worktree = if s.contains(git2::Status::CONFLICTED) {
                Some(FileStatus::Conflicted)
            } else if s.contains(git2::Status::WT_NEW) {
                Some(FileStatus::Untracked)
            } else if s.contains(git2::Status::WT_MODIFIED) {
                Some(FileStatus::Modified)
            } else if s.contains(git2::Status::WT_DELETED) {
                Some(FileStatus::Deleted)
            } else if s.contains(git2::Status::WT_RENAMED) {
                Some(FileStatus::Renamed)
            } else {
                None
            };

            if index.is_none() && worktree.is_none() {
                return None;
            }
            Some(StatusEntry {
                path,
                index,
                worktree,
            })
        })
        .collect();

    Ok(WorkingTree {
        files,
        ..WorkingTree::default()
    })
}

/// The commit total counted in-process (`log.commit_count = "libgit2"`):
/// the same refs as `log` (branches, tags, remotes; `HEAD` or `start_ref`
/// otherwise), no sorting, nothing hidden and no commit object loaded by
/// this code, so no process is spawned and the walk is as cheap as libgit2
/// allows. A path filter needs history simplification libgit2 has no
/// equivalent of, so that one case still asks system git.
pub fn count_commits(repo: &RepoRef, opts: &LogOptions) -> Result<u64> {
    if !opts.paths.is_empty() {
        return gitpoulpi_core::GitBackend::count_commits(
            &gitpoulpi_cli_backend::SystemGitBackend::new(),
            repo,
            opts,
        );
    }
    let git_repo = open(repo)?;
    let Ok(mut revwalk) = git_repo.revwalk() else {
        return Ok(0);
    };
    let pushed = if opts.all_refs {
        // Same refs as `log` (never the remote-tag mirror).
        push_all_refs(&git_repo, &mut revwalk, &opts.hidden)
    } else {
        match &opts.start_ref {
            Some(start) => revwalk.push_ref(start).or_else(|_| {
                git_repo
                    .revparse_single(start)
                    .and_then(|object| revwalk.push(object.id()))
            }),
            None => revwalk.push_head(),
        }
    };
    if pushed.is_err() {
        // Unborn branch: a valid state, zero commits.
        return Ok(0);
    }
    // Order is irrelevant to a count; skipping the sort lets libgit2 stop
    // buffering and sorting the whole history.
    revwalk.set_sorting(git2::Sort::NONE).map_err(map_err)?;
    Ok(revwalk.count() as u64)
}

/// The repository's `.mailmap` (the worktree's file, else `HEAD:.mailmap`
/// in a bare repository) and then `mailmap.file`, as git layers them.
///
/// Parsed by `gitpoulpi_core`'s `Mailmap`, not libgit2's: libgit2 compares
/// the email case-sensitively where git does not (`ALEX@old.example.com` is
/// not mapped by a line for `alex@old.example.com`), and the contract suite
/// pins that both backends show what `git log --use-mailmap` shows.
fn load_mailmap(git_repo: &git2::Repository) -> gitpoulpi_core::domain::rewrite::Mailmap {
    use gitpoulpi_core::domain::rewrite::Mailmap;
    let mut map = match git_repo.workdir() {
        Some(dir) => std::fs::read_to_string(dir.join(".mailmap")).unwrap_or_default(),
        None => git_repo
            .revparse_single("HEAD:.mailmap")
            .ok()
            .and_then(|o| o.into_blob().ok())
            .map(|b| String::from_utf8_lossy(b.content()).into_owned())
            .unwrap_or_default(),
    };
    map = map.replace('\r', "");
    let mut out = Mailmap::parse(&map);
    if let Some(file) = git_repo
        .config()
        .ok()
        .and_then(|c| c.get_path("mailmap.file").ok())
    {
        if let Ok(text) = std::fs::read_to_string(file) {
            out.extend(Mailmap::parse(&text));
        }
    }
    out
}

/// Each identity as the `.mailmap` shows it (see `GitBackend::mailmap_resolve`).
pub fn mailmap_resolve(
    repo: &RepoRef,
    idents: &[gitpoulpi_core::domain::rewrite::Ident],
) -> Result<Vec<gitpoulpi_core::domain::rewrite::Ident>> {
    let map = load_mailmap(&open(repo)?);
    Ok(idents.iter().map(|i| map.resolve(i)).collect())
}

/// Seed `revwalk` for an `all_refs` walk. Without a filter, the three ref
/// families by glob; with one, a push per visible tip
/// (`gitpoulpi_core::domain::visible_tips`, the decision both backends
/// share). `Err` means the walk has no starting point (unborn, or nothing
/// visible).
fn push_all_refs(
    git_repo: &git2::Repository,
    revwalk: &mut git2::Revwalk<'_>,
    filter: &gitpoulpi_core::domain::RefFilter,
) -> std::result::Result<(), git2::Error> {
    if filter.is_empty() {
        // A repository can lack any of the three (no remotes, no tags) and
        // `push_glob` errors on a glob that matches nothing, so only a
        // failure of *all three* means the walk has no starting point.
        let heads = revwalk.push_glob("refs/heads/*");
        let tags = revwalk.push_glob("refs/tags/*");
        let remotes = revwalk.push_glob("refs/remotes/*");
        return if heads.is_ok() || tags.is_ok() || remotes.is_ok() {
            Ok(())
        } else {
            heads
        };
    }
    let mut refs: Vec<(String, Oid)> = Vec::new();
    for pattern in ["refs/heads/*", "refs/tags/*", "refs/remotes/*"] {
        for reference in git_repo.references_glob(pattern)?.flatten() {
            // `refs/remotes/<r>/HEAD` and friends mirror another ref: not a
            // tip of their own (hiding the branch they name must hide it).
            if reference.kind() == Some(git2::ReferenceType::Symbolic) {
                continue;
            }
            let (Ok(name), Ok(commit)) = (reference.name(), reference.peel_to_commit()) else {
                // A tag of a tree or a blob: no commit, no tip.
                continue;
            };
            if let Ok(id) = to_domain_oid(commit.id()) {
                refs.push((name.to_string(), id));
            }
        }
    }
    let head = git_repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_commit().ok())
        .and_then(|c| to_domain_oid(c.id()).ok());
    let tips = gitpoulpi_core::domain::visible_tips(filter, &refs, head);
    if tips.is_empty() {
        return Err(git2::Error::from_str("no visible tip"));
    }
    for tip in tips {
        revwalk.push(git2::Oid::from_bytes(&tip.0)?)?;
    }
    Ok(())
}

pub fn log(repo: &RepoRef, opts: &LogOptions) -> Result<CommitGraph> {
    let git_repo = open(repo)?;
    let mut revwalk = match git_repo.revwalk() {
        Ok(w) => w,
        Err(_) => return Ok(CommitGraph::default()),
    };

    let pushed = if opts.all_refs {
        // The three ref families the UI draws, named explicitly: not
        // `push_glob("*")`, which would include GitPoulpi's remote-tag mirror
        // (`refs/gitpoulpi/remote-tags/...`), bookkeeping that must never put
        // a line in the commit graph. The system-git backend says
        // `--branches --tags --remotes` for the same reason.
        push_all_refs(&git_repo, &mut revwalk, &opts.hidden)
    } else {
        match &opts.start_ref {
            // A ref name, else any revision git parses (a commit id): the
            // system-git backend takes both, and so must this one.
            Some(start) => revwalk.push_ref(start).or_else(|_| {
                git_repo
                    .revparse_single(start)
                    .and_then(|object| revwalk.push(object.id()))
            }),
            None => revwalk.push_head(),
        }
    };
    if pushed.is_err() {
        // unborn branch (no commits yet) -- a valid, empty state.
        return Ok(CommitGraph::default());
    }
    // TIME alone is reverse-chronological and does NOT guarantee a
    // commit's parents are walked after it; with diverged branches a parent
    // could surface before a child and corrupt the TUI's one-row-per-commit
    // lane bookkeeping in `build_graph_lines` (it assumes parents appear
    // later in `nodes`). TOPOLOGICAL enforces "children before parents";
    // with TIME it is the git2 equivalent of `git log --graph --date-order`.
    revwalk
        .set_sorting(git2::Sort::TIME | git2::Sort::TOPOLOGICAL)
        .map_err(map_err)?;

    // `log.use_mailmap`: the same files `git log --format=%aN` reads.
    let mailmap = opts.use_mailmap.then(|| load_mailmap(&git_repo));
    let mut nodes = Vec::new();
    // `skip` is applied to the *walk*, before `limit`, so page `n` is the
    // same slice of the same ordering the unpaged walk would produce (see
    // `check_log_skip_pages_history`). libgit2 has no `--skip`, so oids are
    // skipped without being turned into a `Commit`: one revwalk step per
    // skipped commit and nothing more.
    let mut to_skip = opts.skip;
    for oid in revwalk {
        if to_skip > 0 {
            to_skip -= 1;
            continue;
        }
        if let Some(limit) = opts.limit {
            if nodes.len() as u32 >= limit {
                break;
            }
        }
        let oid = oid.map_err(map_err)?;
        let commit = git_repo.find_commit(oid).map_err(map_err)?;
        let author = commit.author();
        // Display text follows git: the commit's `encoding` header, else
        // valid UTF-8, else the fallback encoding (`git.default_encoding`).
        let header = commit.message_encoding().ok().flatten();
        let message_bytes = commit.message_bytes();
        let decoded = decode_commit_text(message_bytes, header);
        let summary = match (&decoded, header) {
            (std::borrow::Cow::Borrowed(_), None) => commit
                .summary()
                .ok()
                .flatten()
                .unwrap_or_default()
                .to_string(),
            _ => commit_summary(&decoded),
        };
        let message = decoded.trim_end().to_string();
        let mut author_name = decode_commit_text(author.name_bytes(), header).into_owned();
        let mut author_email = author.email().unwrap_or_default().to_string();
        if let Some(map) = mailmap.as_ref().filter(|m| !m.is_empty()) {
            let shown = map.resolve(&gitpoulpi_core::domain::rewrite::Ident::new(
                &author_name,
                &author_email,
            ));
            author_name = shown.name;
            author_email = shown.email;
        }
        nodes.push(CommitNode {
            id: to_domain_oid(oid)?,
            parents: commit
                .parent_ids()
                .map(to_domain_oid)
                .collect::<Result<Vec<_>>>()?,
            summary,
            // git2's message() keeps the trailing newline git normalizes
            // onto stored messages; trimmed to match SystemGitBackend's `%B`
            // parsing (see cli-backend's log.rs).
            message,
            author_name,
            author_email,
            // The author date, as SystemGitBackend's `%at` (a rebased or
            // amended commit keeps it; the committer date is "now").
            timestamp: commit.author().when().seconds(),
            refs: Vec::new(),
        });
    }

    Ok(CommitGraph { nodes })
}

#[derive(Default)]
struct DiffBuilder {
    files: Vec<FileDiff>,
}

impl DiffBuilder {
    fn on_file(&mut self, delta: git2::DiffDelta) {
        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .map(PathBuf::from)
            .unwrap_or_default();
        let status = match delta.status() {
            git2::Delta::Added => FileStatus::Added,
            git2::Delta::Deleted => FileStatus::Deleted,
            git2::Delta::Renamed | git2::Delta::Copied => FileStatus::Renamed,
            _ => FileStatus::Modified,
        };
        self.files.push(FileDiff {
            path,
            status,
            hunks: Vec::new(),
        });
    }

    fn on_hunk(&mut self, hunk: git2::DiffHunk) {
        if let Some(file) = self.files.last_mut() {
            file.hunks.push(Hunk {
                id: HunkId(file.hunks.len() as u32),
                old_start: hunk.old_start(),
                old_lines: hunk.old_lines(),
                new_start: hunk.new_start(),
                new_lines: hunk.new_lines(),
                lines: Vec::new(),
            });
        }
    }

    fn on_line(&mut self, line: git2::DiffLine) {
        let origin = match line.origin() {
            '+' => LineOrigin::Add,
            '-' => LineOrigin::Delete,
            ' ' => LineOrigin::Context,
            _ => return,
        };
        let content = gitpoulpi_core::domain::decode_text_lossy(line.content())
            .trim_end_matches(['\r', '\n'])
            .to_string();
        if let Some(file) = self.files.last_mut() {
            if let Some(hunk) = file.hunks.last_mut() {
                hunk.lines.push(DiffLine {
                    origin,
                    content,
                    staged: false,
                });
            }
        }
    }
}

fn build_file_diffs(diff: &git2::Diff) -> Result<Vec<FileDiff>> {
    let builder = RefCell::new(DiffBuilder::default());
    diff.foreach(
        &mut |delta, _progress| {
            builder.borrow_mut().on_file(delta);
            true
        },
        None,
        Some(&mut |_delta, hunk| {
            builder.borrow_mut().on_hunk(hunk);
            true
        }),
        Some(&mut |_delta, _hunk, line| {
            builder.borrow_mut().on_line(line);
            true
        }),
    )
    .map_err(map_err)?;
    Ok(builder.into_inner().files)
}

pub fn diff(repo: &RepoRef, target: DiffTarget) -> Result<Vec<FileDiff>> {
    let git_repo = open(repo)?;
    let diff = git_diff(&git_repo, target, None)?;
    build_file_diffs(&diff)
}

/// `diff` restricted to literal `paths`: libgit2 pathspecs
/// with matching disabled, so `*` and `[` in a name mean themselves.
pub fn diff_paths(repo: &RepoRef, target: DiffTarget, paths: &[PathBuf]) -> Result<Vec<FileDiff>> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    let git_repo = open(repo)?;
    let diff = git_diff(&git_repo, target, Some(paths))?;
    build_file_diffs(&diff)
}

/// `diff`'s file list without the hunks.
///
/// The same `git2::Diff`, built by the same `git_diff`, walked with the
/// same `DiffBuilder::on_file`, with only the patch generation skipped:
/// listing deltas compares two trees, while `foreach` with hunk and line
/// callbacks reads and diffs every changed blob (99% of a commit's diff
/// time).
pub fn diff_files(repo: &RepoRef, target: DiffTarget) -> Result<DiffFileList> {
    let git_repo = open(repo)?;
    let diff = git_diff(&git_repo, target, None)?;
    let mut builder = DiffBuilder::default();
    for delta in diff.deltas() {
        builder.on_file(delta);
    }
    Ok(DiffFileList {
        files: builder.files,
        hunks_included: false,
    })
}

/// `GitBackend::commit_stats`: each commit's tree against its first
/// parent's, through libgit2's own diff statistics (no rename detection,
/// binary files count no lines).
pub fn commit_stats(repo: &RepoRef, ids: &[Oid]) -> Result<Vec<CommitStat>> {
    let git_repo = open(repo)?;
    let mut opts = git2::DiffOptions::new();
    ids.iter()
        .map(|&id| {
            let commit = git_repo.find_commit(to_git2_oid(id)?).map_err(map_err)?;
            let new_tree = commit.tree().map_err(map_err)?;
            let old_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
            let stats = git_repo
                .diff_tree_to_tree(old_tree.as_ref(), Some(&new_tree), Some(&mut opts))
                .map_err(map_err)?
                .stats()
                .map_err(map_err)?;
            Ok(CommitStat {
                id,
                files: stats.files_changed() as u32,
                insertions: stats.insertions() as u32,
                deletions: stats.deletions() as u32,
            })
        })
        .collect()
}

fn git_diff<'r>(
    git_repo: &'r git2::Repository,
    target: DiffTarget,
    paths: Option<&[PathBuf]>,
) -> Result<git2::Diff<'r>> {
    let mut opts = git2::DiffOptions::new();
    opts.context_lines(3);
    if let Some(paths) = paths {
        opts.disable_pathspec_match(true);
        for p in paths {
            opts.pathspec(p.to_string_lossy().replace('\\', "/"));
        }
    }

    let diff = match target {
        DiffTarget::WorkingTreeVsIndex => git_repo
            .diff_index_to_workdir(None, Some(&mut opts))
            .map_err(map_err)?,
        DiffTarget::IndexVsHead => {
            let head_tree = git_repo.head().ok().and_then(|h| h.peel_to_tree().ok());
            git_repo
                .diff_tree_to_index(head_tree.as_ref(), None, Some(&mut opts))
                .map_err(map_err)?
        }
        DiffTarget::CommitVsParent(oid) => {
            let git2_oid = crate::repo::to_git2_oid(oid)?;
            let commit = git_repo.find_commit(git2_oid).map_err(map_err)?;
            let new_tree = commit.tree().map_err(map_err)?;
            let old_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
            git_repo
                .diff_tree_to_tree(old_tree.as_ref(), Some(&new_tree), Some(&mut opts))
                .map_err(map_err)?
        }
        DiffTarget::CommitRange(from, to) => {
            let from_tree = git_repo
                .find_commit(crate::repo::to_git2_oid(from)?)
                .map_err(map_err)?
                .tree()
                .map_err(map_err)?;
            let to_tree = git_repo
                .find_commit(crate::repo::to_git2_oid(to)?)
                .map_err(map_err)?
                .tree()
                .map_err(map_err)?;
            git_repo
                .diff_tree_to_tree(Some(&from_tree), Some(&to_tree), Some(&mut opts))
                .map_err(map_err)?
        }
        DiffTarget::CommitVsWorkingTree(oid) => {
            let tree = git_repo
                .find_commit(crate::repo::to_git2_oid(oid)?)
                .map_err(map_err)?
                .tree()
                .map_err(map_err)?;
            // Staged and unstaged edits of tracked files, like
            // `git diff <commit>`.
            git_repo
                .diff_tree_to_workdir_with_index(Some(&tree), Some(&mut opts))
                .map_err(map_err)?
        }
        DiffTarget::CommitSpan(oldest, newest) => {
            let oldest = git_repo
                .find_commit(crate::repo::to_git2_oid(oldest)?)
                .map_err(map_err)?;
            let from_tree = oldest.parent(0).ok().and_then(|p| p.tree().ok());
            let to_tree = git_repo
                .find_commit(crate::repo::to_git2_oid(newest)?)
                .map_err(map_err)?
                .tree()
                .map_err(map_err)?;
            git_repo
                .diff_tree_to_tree(from_tree.as_ref(), Some(&to_tree), Some(&mut opts))
                .map_err(map_err)?
        }
    };
    Ok(diff)
}
