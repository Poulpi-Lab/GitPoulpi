use std::path::Path;

use gitpoulpi_core::domain::{DiscardTarget, HunkId, LineSelector, RepoRef, StageTarget};
use gitpoulpi_core::{Error, Result};

use crate::patch::{self, Direction, PatchHunk};
use crate::process::{run, run_with_stdin};

pub fn stage(repo: &RepoRef, target: &StageTarget) -> Result<()> {
    match target {
        StageTarget::File(path) => {
            run(&repo.root, &["add", "--", &path_str(path)])?;
        }
        StageTarget::Folder(path) => {
            run(&repo.root, &["add", "--", &path_str(path)])?;
        }
        StageTarget::Files(paths) => {
            if !paths.is_empty() {
                let mut args = vec!["add".to_string(), "--".to_string()];
                args.extend(paths.iter().map(|p| path_str(p)));
                let args: Vec<&str> = args.iter().map(String::as_str).collect();
                run(&repo.root, &args)?;
            }
        }
        StageTarget::All => {
            run(&repo.root, &["add", "-A"])?;
        }
        StageTarget::Hunk(path, hunk_id) => {
            if let Some(whole) = untracked_hunk(repo, path)? {
                // The one hunk of a new file is the whole file.
                if hunk_id.0 != 0 {
                    return Err(hunk_not_found(path, *hunk_id));
                }
                return apply_new_file(repo, path, &whole);
            }
            let hunk = find_hunk(repo, Side::Worktree, path, *hunk_id)?;
            apply(repo, path, &hunk, &["apply", "--cached", "-"])?;
        }
        StageTarget::Line(path, selector) => {
            if let Some(whole) = untracked_hunk(repo, path)? {
                if selector.hunk.0 != 0 {
                    return Err(hunk_not_found(path, selector.hunk));
                }
                let picked =
                    patch::select_lines(&whole, &selector.line_indices, Direction::Forward);
                if !picked.has_changes() {
                    return Err(Error::Backend(format!(
                        "the selection covers no line of {}",
                        path.display()
                    )));
                }
                return apply_new_file(repo, path, &picked);
            }
            let hunk = select(repo, Side::Worktree, path, selector, Direction::Forward)?;
            apply(repo, path, &hunk, &["apply", "--cached", "-"])?;
        }
    }
    Ok(())
}

fn hunk_not_found(path: &Path, id: HunkId) -> Error {
    Error::Backend(format!("hunk {} not found in {}", id.0, path.display()))
}

/// For a file git has never seen (not in the index, present on disk): the
/// one all-added hunk the frontends show for it, numbered `HunkId(0)`
/// (`git diff` has no hunks for it). `None` for any other file.
fn untracked_hunk(repo: &RepoRef, path: &Path) -> Result<Option<PatchHunk>> {
    let spec = format!(":(literal){}", path_str(path));
    if !run(&repo.root, &["ls-files", "-z", "--", &spec])?.is_empty() {
        return Ok(None);
    }
    let full = repo.root.join(path);
    if !full.is_file() {
        return Ok(None);
    }
    let bytes = std::fs::read(&full)
        .map_err(|e| Error::Backend(format!("could not read {}: {e}", path.display())))?;
    let text = String::from_utf8(bytes).map_err(|_| {
        Error::Backend(format!(
            "{} is not text: stage or discard it as a whole file",
            path.display()
        ))
    })?;
    Ok(patch::whole_file_hunk(&text))
}

/// Stages `hunk` (all additions) as the creation of `path`.
fn apply_new_file(repo: &RepoRef, path: &Path, hunk: &PatchHunk) -> Result<()> {
    let text = patch::render_new_file(&path_str(path), file_mode(&repo.root.join(path)), hunk);
    run_with_stdin(&repo.root, &["apply", "--cached", "-"], &text)?;
    Ok(())
}

#[cfg(unix)]
fn file_mode(path: &Path) -> &'static str {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(path) {
        Ok(m) if m.permissions().mode() & 0o111 != 0 => "100755",
        _ => "100644",
    }
}

#[cfg(not(unix))]
fn file_mode(_path: &Path) -> &'static str {
    "100644"
}

/// Discarding part of a file git has never seen: the hunk deletes the file,
/// lines are cut out of it.
fn discard_untracked(
    repo: &RepoRef,
    path: &Path,
    whole: &PatchHunk,
    target: &DiscardTarget,
) -> Result<()> {
    let full = repo.root.join(path);
    let (id, indices) = match target {
        DiscardTarget::Hunk(_, id) => (*id, None),
        DiscardTarget::Line(_, selector) => (selector.hunk, Some(&selector.line_indices)),
    };
    if id.0 != 0 {
        return Err(hunk_not_found(path, id));
    }
    let Some(indices) = indices else {
        return std::fs::remove_file(&full)
            .map_err(|e| Error::Backend(format!("could not delete {}: {e}", path.display())));
    };
    if !indices.iter().any(|i| (*i as usize) < whole.lines.len()) {
        return Err(Error::Backend(format!(
            "the selection covers no line of {}",
            path.display()
        )));
    }
    let mut out = String::new();
    for (i, line) in whole.lines.iter().enumerate() {
        if indices.contains(&(i as u32)) {
            continue;
        }
        out.push_str(&line.text);
        if !line.no_eol {
            out.push('\n');
        }
    }
    std::fs::write(&full, out)
        .map_err(|e| Error::Backend(format!("could not write {}: {e}", path.display())))
}

pub fn unstage(repo: &RepoRef, target: &StageTarget) -> Result<()> {
    match target {
        StageTarget::File(path) => {
            unstage_paths(repo, &path_str(path))?;
        }
        StageTarget::Files(paths) => {
            if !paths.is_empty() {
                let specs: Vec<String> = paths.iter().map(|p| path_str(p)).collect();
                let specs: Vec<&str> = specs.iter().map(String::as_str).collect();
                unstage_specs(repo, &specs)?;
            }
        }
        // A folder or everything never reaches an unmerged path: it has no
        // staged change to take back, and `git restore --staged` would
        // reset it to `HEAD`, dropping its conflict stages (the whole
        // merge's, from "unstage all").
        StageTarget::Folder(path) => {
            unstage_all_but_unmerged(repo, &path_str(path))?;
        }
        StageTarget::All => {
            unstage_all_but_unmerged(repo, ".")?;
        }
        StageTarget::Hunk(path, hunk_id) => {
            let hunk = find_hunk(repo, Side::Index, path, *hunk_id)?;
            apply(repo, path, &hunk, &["apply", "--cached", "--reverse", "-"])?;
        }
        StageTarget::Line(path, selector) => {
            let hunk = select(repo, Side::Index, path, selector, Direction::Reverse)?;
            apply(repo, path, &hunk, &["apply", "--cached", "--reverse", "-"])?;
        }
    }
    Ok(())
}

/// `git restore --staged -- <spec>`; on an unborn branch (no `HEAD` to
/// restore from) it empties the index entries for `spec` instead.
fn unstage_paths(repo: &RepoRef, spec: &str) -> Result<()> {
    unstage_specs(repo, &[spec])
}

/// [`unstage_paths`] for several pathspecs in one git call (atomic).
fn unstage_specs(repo: &RepoRef, specs: &[&str]) -> Result<()> {
    let mut restore = vec!["restore", "--staged", "--"];
    restore.extend_from_slice(specs);
    match run(&repo.root, &restore) {
        // Git 2.55 quotes it: "could not resolve 'HEAD'".
        Err(Error::Git { message: msg, .. })
            if msg.contains("could not resolve HEAD")
                || msg.contains("could not resolve 'HEAD'") =>
        {
            let mut rm = vec!["rm", "-r", "-q", "--cached", "--ignore-unmatch", "--"];
            rm.extend_from_slice(specs);
            run(&repo.root, &rm)?;
            Ok(())
        }
        other => other.map(|_| ()),
    }
}

/// [`unstage_paths`] for a folder or everything: when `spec` reaches
/// unmerged paths, names the staged, merged ones one by one instead
/// (`git restore` refuses a pathspec that excludes them).
fn unstage_all_but_unmerged(repo: &RepoRef, spec: &str) -> Result<()> {
    // No conflict stage in the index (read without spawning git): the
    // plain restore, one spawn.
    let unmerged = match crate::index_scan::has_unmerged(&crate::conflicts::git_dir(&repo.root)) {
        Some(false) => Vec::new(),
        _ => unmerged_under(repo, spec),
    };
    if unmerged.is_empty() {
        return unstage_paths(repo, spec);
    }
    let staged = run(
        &repo.root,
        &[
            "diff-index",
            "--cached",
            "--name-only",
            "--no-renames",
            "-z",
            "HEAD",
            "--",
            spec,
        ],
    )?;
    let mut args: Vec<String> = ["restore", "--staged", "--"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    args.extend(
        staged
            .split('\0')
            .filter(|p| !p.is_empty() && !unmerged.iter().any(|u| u == p))
            .map(|p| format!(":(literal){p}")),
    );
    if args.len() == 3 {
        return Ok(());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run(&repo.root, &refs).map(|_| ())
}

/// `GitBackend::discard_part`: the working-tree-vs-index
/// hunk (or the selected lines of it) applied *in reverse to the working
/// tree* - no `--cached`, so the index is never touched and what comes
/// back is the index's content for those lines.
///
/// The line selection is built exactly like unstaging's (`Direction::
/// Reverse`), and for the same reason: both reverse-apply a patch onto
/// the side that holds the diff's *new* content (the index when
/// unstaging, the working tree here).
pub fn discard_part(repo: &RepoRef, target: &DiscardTarget) -> Result<()> {
    let (DiscardTarget::Hunk(path, _) | DiscardTarget::Line(path, _)) = target;
    if let Some(whole) = untracked_hunk(repo, path)? {
        return discard_untracked(repo, path, &whole, target);
    }
    let (path, hunk) = match target {
        DiscardTarget::Hunk(path, hunk_id) => {
            (path, find_hunk(repo, Side::Worktree, path, *hunk_id)?)
        }
        DiscardTarget::Line(path, selector) => (
            path,
            select(repo, Side::Worktree, path, selector, Direction::Reverse)?,
        ),
    };
    apply(repo, path, &hunk, &["apply", "--reverse", "-"])
}

/// Which diff a `HunkId` is numbered in.
#[derive(Clone, Copy)]
enum Side {
    /// Working tree vs index - the Unstaged half.
    Worktree,
    /// Index vs `HEAD` - the Staged half.
    Index,
}

/// The unmerged paths `spec` covers (`git ls-files -u -z`, deduplicated).
/// Empty when none, or when git cannot tell.
fn unmerged_under(repo: &RepoRef, spec: &str) -> Vec<String> {
    let Ok(out) = run(&repo.root, &["ls-files", "-u", "-z", "--", spec]) else {
        return Vec::new();
    };
    let mut paths: Vec<String> = out
        .split('\0')
        .filter_map(|record| record.split_once('\t').map(|(_, p)| p.to_string()))
        .collect();
    paths.dedup();
    paths
}

fn path_str(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

/// The hunks of one file's diff, *byte-faithful*: read straight from
/// `git diff` for that path rather than from the parsed `FileDiff`, whose
/// lines have lost their `\r` and their "\ No newline at end of file"
/// markers - either of which makes a patch rebuilt from them fail to
/// apply (see `patch.rs`).
///
/// The numbering is the same one `crate::diff` assigns (the hunks of the
/// file, in order), because it is the same command with the same `-U3`,
/// just restricted to one literal pathspec.
fn file_hunks(repo: &RepoRef, side: Side, path: &Path) -> Result<Vec<PatchHunk>> {
    let spec = format!(":(literal){}", path_str(path));
    let mut args = vec![
        "diff",
        "--no-color",
        "--no-ext-diff",
        "--no-textconv",
        "-U3",
    ];
    if matches!(side, Side::Index) {
        args.push("--cached");
    }
    args.extend(["--", &spec]);
    let raw = run(&repo.root, &args)?;
    let hunks = patch::parse_hunks(&raw);
    if hunks.is_empty() {
        return Err(Error::Backend(format!(
            "no diff found for {}",
            path.display()
        )));
    }
    Ok(hunks)
}

fn find_hunk(repo: &RepoRef, side: Side, path: &Path, id: HunkId) -> Result<PatchHunk> {
    file_hunks(repo, side, path)?
        .into_iter()
        .nth(id.0 as usize)
        .ok_or_else(|| Error::Backend(format!("hunk {} not found in {}", id.0, path.display())))
}

fn select(
    repo: &RepoRef,
    side: Side,
    path: &Path,
    selector: &LineSelector,
    direction: Direction,
) -> Result<PatchHunk> {
    let hunk = find_hunk(repo, side, path, selector.hunk)?;
    let selected = patch::select_lines(&hunk, &selector.line_indices, direction);
    if !selected.has_changes() {
        return Err(Error::Backend(format!(
            "the selection covers no added or removed line of hunk {} in {}",
            selector.hunk.0,
            path.display()
        )));
    }
    Ok(selected)
}

/// Feeds `hunk` as a one-hunk patch of `path` to `git <args>`. `git
/// apply` checks the whole patch before writing anything, so a patch that
/// no longer applies (the file moved underneath the caller) is an error
/// that leaves the file as it was.
fn apply(repo: &RepoRef, path: &Path, hunk: &PatchHunk, args: &[&str]) -> Result<()> {
    let text = patch::render(&path_str(path), hunk);
    run_with_stdin(&repo.root, args, &text)?;
    Ok(())
}
