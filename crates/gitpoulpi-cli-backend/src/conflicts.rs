use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{
    classify_sides, encode_text, BlobContent, ConflictEntry, ConflictOperation, ConflictRefusal,
    ConflictResolution, ConflictSet, ConflictSide, EncodeError, MarkerStyle, MergeDocument, Oid,
    RawStage, RepoRef, ResolveOptions, SideLabels, StageEntry, MAX_TEXT_BLOB_BYTES, MODE_GITLINK,
    MODE_SYMLINK,
};
use gitpoulpi_core::{Error, Result};

use crate::content::repo_relative;
use crate::process::{
    cat_file_batch_capped, git_at_least, run, run_bytes, run_merge_file, run_with_stdin,
};

fn stage_content(repo: &RepoRef, stage: u8, path: &str) -> Option<String> {
    run(&repo.root, &["show", &format!(":{stage}:{path}")]).ok()
}

pub fn conflicts(repo: &RepoRef) -> Result<Vec<ConflictSet>> {
    let raw = run(&repo.root, &["diff", "--name-only", "--diff-filter=U"])?;
    let sets = raw
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|path| ConflictSet {
            path: path.into(),
            base: stage_content(repo, 1, path),
            ours: stage_content(repo, 2, path),
            theirs: stage_content(repo, 3, path),
        })
        .collect();
    Ok(sets)
}

/// See `GitBackend::resolve_conflict`.
///
/// The stale guard runs first and nothing is written before every check
/// passed. Regular files are written in place (so the exec bit survives)
/// and staged with `git add`; a side's whole file goes in through
/// `checkout --ours/--theirs` and `update-index` with the stage's own blob
/// and mode, so the staged result is that stage, exactly.
pub fn resolve_conflict(
    repo: &RepoRef,
    path: &Path,
    resolution: ConflictResolution,
    opts: &ResolveOptions,
) -> Result<()> {
    let rel = repo_relative(path)?;
    let unmerged = entry_for_opt(repo, &rel)?;
    if let Some(expected) = opts.expected {
        let now = unmerged.as_ref().map_or([None; 3], ResolveOptions::oids_of);
        if now != expected {
            return Err(Error::ConflictRefused(ConflictRefusal::Stale));
        }
    }
    match resolution {
        ConflictResolution::TakeSide(side) => take_side(repo, &rel, unmerged, side, opts.stage),
        ConflictResolution::UseContent(text) => {
            write_worktree(repo, &rel, text.as_bytes())?;
            stage_added(repo, &rel, opts)
        }
        ConflictResolution::UseEncoded { text, encoding } => {
            let bytes = encode_text(&text, &encoding).map_err(|e| match e {
                EncodeError::Unencodable { line } => {
                    Error::ConflictRefused(ConflictRefusal::Unencodable { line })
                }
                EncodeError::UnknownEncoding(label) => {
                    Error::Backend(format!("unknown encoding {label}"))
                }
            })?;
            write_worktree(repo, &rel, &bytes)?;
            stage_added(repo, &rel, opts)
        }
        ConflictResolution::Delete => remove_path(repo, &rel, opts.stage),
    }
}

fn stage_added(repo: &RepoRef, rel: &str, opts: &ResolveOptions) -> Result<()> {
    if opts.stage {
        run(&repo.root, &["--literal-pathspecs", "add", "--", rel])?;
    }
    Ok(())
}

/// Writes `bytes` as the file at `rel`. A regular file is overwritten in
/// place (its mode stays); a symlink or directory in the way is removed
/// first (writing through a symlink would change its target).
fn write_worktree(repo: &RepoRef, rel: &str, bytes: &[u8]) -> Result<()> {
    let full = repo.root.join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if let Ok(meta) = std::fs::symlink_metadata(&full) {
        if !meta.is_file() {
            crate::snapshot_ops::remove_any(&full)?;
        }
    }
    std::fs::write(&full, bytes)?;
    Ok(())
}

/// Writes a symlink to `target` (a plain file holding the target text
/// where symlinks are not available, as git does with
/// `core.symlinks=false`).
fn write_symlink(repo: &RepoRef, rel: &str, target: &[u8]) -> Result<()> {
    let full = repo.root.join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent)?;
    }
    crate::snapshot_ops::remove_any(&full)?;
    #[cfg(unix)]
    {
        let target = String::from_utf8_lossy(target).into_owned();
        std::os::unix::fs::symlink(target, &full)?;
    }
    #[cfg(not(unix))]
    std::fs::write(&full, target)?;
    Ok(())
}

/// Index entries through `update-index --index-info`: `<mode> <oid>
/// <stage>\t<path>` lines, a mode of 0 removing every stage of the path.
fn index_info(repo: &RepoRef, lines: &[String]) -> Result<()> {
    let mut input = String::new();
    for line in lines {
        input.push_str(line);
        input.push('\0');
    }
    run_with_stdin(&repo.root, &["update-index", "-z", "--index-info"], &input)?;
    Ok(())
}

const ZERO_OID: &str = "0000000000000000000000000000000000000000";

/// The path goes: the file (when staging or not) and, when staging, every
/// index entry.
fn remove_path(repo: &RepoRef, rel: &str, stage: bool) -> Result<()> {
    crate::snapshot_ops::remove_any(&repo.root.join(rel))?;
    if stage {
        index_info(repo, &[format!("0 {ZERO_OID}\t{rel}")])?;
    }
    Ok(())
}

fn take_side(
    repo: &RepoRef,
    rel: &str,
    unmerged: Option<ConflictEntry>,
    side: ConflictSide,
    stage: bool,
) -> Result<()> {
    let stage_no: u8 = match side {
        ConflictSide::Base => 1,
        ConflictSide::Ours => 2,
        ConflictSide::Theirs => 3,
    };
    let Some(entry) = unmerged else {
        return Err(Error::ConflictRefused(ConflictRefusal::NotConflicted));
    };
    let Some(taken) = entry.stage(side) else {
        // That side has no file here: it deleted the path. Base missing
        // (add/add) has nothing to take.
        if side == ConflictSide::Base {
            return Err(Error::Backend(
                "there is no common ancestor version of this file".into(),
            ));
        }
        return remove_path(repo, rel, stage);
    };
    if taken.mode == MODE_GITLINK {
        // A submodule pointer: the index is all there is, no checkout.
        if stage {
            index_info(
                repo,
                &[
                    format!("0 {ZERO_OID}\t{rel}"),
                    format!("{:o} {} 0\t{rel}", taken.mode, taken.oid),
                ],
            )?;
        }
        return Ok(());
    }
    match side {
        ConflictSide::Ours | ConflictSide::Theirs => {
            let flag = if side == ConflictSide::Ours {
                "--ours"
            } else {
                "--theirs"
            };
            run(
                &repo.root,
                &["--literal-pathspecs", "checkout", flag, "--", rel],
            )?;
        }
        ConflictSide::Base => {
            let bytes = run_bytes(
                &repo.root,
                &["cat-file", "blob", &format!(":{stage_no}:{rel}")],
            )?;
            if taken.mode == MODE_SYMLINK {
                write_symlink(repo, rel, &bytes)?;
            } else {
                write_worktree(repo, rel, &bytes)?;
            }
        }
    }
    if stage {
        index_info(
            repo,
            &[
                format!("0 {ZERO_OID}\t{rel}"),
                format!("{:o} {} 0\t{rel}", taken.mode, taken.oid),
            ],
        )?;
    }
    Ok(())
}

/// See `GitBackend::unresolve_conflict`: the stages go back into the
/// index, then the file git would have left in the working tree.
pub fn unresolve_conflict(repo: &RepoRef, path: &Path, entry: &ConflictEntry) -> Result<()> {
    let rel = repo_relative(path)?;
    if entry_for_opt(repo, &rel)?.is_some() {
        return Err(Error::ConflictRefused(ConflictRefusal::AlreadyConflicted));
    }
    let mut lines = vec![format!("0 {ZERO_OID}\t{rel}")];
    for (n, stage) in [(1, entry.base), (2, entry.ours), (3, entry.theirs)] {
        if let Some(stage) = stage {
            lines.push(format!("{:o} {} {n}\t{rel}", stage.mode, stage.oid));
        }
    }
    index_info(repo, &lines)?;

    // The working tree file.
    let doc = merge_document(repo, path)?;
    if doc.conflict_count() > 0 {
        let crlf = [&doc.ours, &doc.theirs]
            .into_iter()
            .flatten()
            .next()
            .is_some_and(|b| matches!(b.content, BlobContent::Text { crlf: true, .. }));
        let eol = if crlf { "\r\n" } else { "\n" };
        let mut text = doc.render_markers(&MarkerStyle {
            marker_size: 7,
            diff3: false,
            ours_label: &doc.labels.git_ours_label,
            base_label: "",
            theirs_label: &doc.labels.git_theirs_label,
            eol,
            final_newline: doc.final_newline,
        });
        if doc.bom {
            text.insert(0, '\u{feff}');
        }
        // Text decoded with a fallback encoding goes back in it.
        let bytes = match &doc.encoding {
            Some(label) => encode_text(&text, label).unwrap_or_else(|_| text.into_bytes()),
            None => text.into_bytes(),
        };
        return write_worktree(repo, &rel, &bytes);
    }
    // Not a text merge: git leaves our version (the surviving side's for
    // a delete/modify) in the working tree.
    let Some((n, shown)) = [(2, entry.ours), (3, entry.theirs), (1, entry.base)]
        .into_iter()
        .find_map(|(n, s)| s.map(|s| (n, s)))
    else {
        return Ok(());
    };
    if shown.mode == MODE_GITLINK {
        return Ok(());
    }
    let bytes = run_bytes(&repo.root, &["cat-file", "blob", &format!(":{n}:{rel}")])?;
    if shown.mode == MODE_SYMLINK {
        write_symlink(repo, &rel, &bytes)
    } else {
        write_worktree(repo, &rel, &bytes)
    }
}

// -- the cheap listing -------------------------------------------------------------

/// `<mode> <oid> <stage>\t<path>` records of `git ls-files -u -z`, folded
/// into one [`ConflictEntry`] per path (git lists a path's stages
/// together, in stage order, paths sorted).
pub fn parse_unmerged(raw: &str) -> Result<Vec<ConflictEntry>> {
    let mut entries: Vec<ConflictEntry> = Vec::new();
    let mut pending: Option<(String, [Option<StageEntry>; 3])> = None;
    let flush = |pending: &mut Option<(String, [Option<StageEntry>; 3])>,
                 entries: &mut Vec<ConflictEntry>| {
        if let Some((path, [base, ours, theirs])) = pending.take() {
            entries.extend(ConflictEntry::from_stages(path.into(), base, ours, theirs));
        }
    };
    for record in raw.split('\0').filter(|r| !r.is_empty()) {
        let bad = || Error::Backend(format!("git ls-files -u gave `{record}`"));
        let (meta, path) = record.split_once('\t').ok_or_else(bad)?;
        let mut words = meta.split(' ');
        let mode = u32::from_str_radix(words.next().ok_or_else(bad)?, 8).map_err(|_| bad())?;
        let oid = Oid::from_hex(words.next().ok_or_else(bad)?).ok_or_else(bad)?;
        let stage: usize = words.next().ok_or_else(bad)?.parse().map_err(|_| bad())?;
        if !(1..=3).contains(&stage) {
            return Err(bad());
        }
        if pending.as_ref().is_some_and(|(p, _)| p != path) {
            flush(&mut pending, &mut entries);
        }
        let slot = pending.get_or_insert_with(|| (path.to_string(), [None; 3]));
        slot.1[stage - 1] = Some(StageEntry { mode, oid });
    }
    flush(&mut pending, &mut entries);
    Ok(entries)
}

/// See `GitBackend::conflict_entries`: one `git ls-files -u -z`, no blob
/// read.
pub fn conflict_entries(repo: &RepoRef) -> Result<Vec<ConflictEntry>> {
    parse_unmerged(&run(&repo.root, &["ls-files", "-u", "-z"])?)
}

fn entry_for_opt(repo: &RepoRef, rel: &str) -> Result<Option<ConflictEntry>> {
    let raw = run(
        &repo.root,
        &["--literal-pathspecs", "ls-files", "-u", "-z", "--", rel],
    )?;
    Ok(parse_unmerged(&raw)?
        .into_iter()
        .find(|e| e.path.to_str() == Some(rel)))
}

fn entry_for(repo: &RepoRef, rel: &str) -> Result<ConflictEntry> {
    entry_for_opt(repo, rel)?.ok_or_else(|| Error::Backend(format!("{rel} is not conflicted")))
}

// -- the document ----------------------------------------------------------------------

/// See `GitBackend::merge_document`.
///
/// One `ls-files -u`, one `cat-file --batch` for the stages, one
/// `merge-file` (and, the first time in a process, `git --version`).
pub fn merge_document(repo: &RepoRef, path: &Path) -> Result<MergeDocument> {
    let rel = repo_relative(path)?;
    let entry = entry_for(repo, &rel)?;
    let stages = [entry.base, entry.ours, entry.theirs];
    // A gitlink's oid is a commit in another repository: never read.
    let readable: Vec<String> = stages
        .iter()
        .flatten()
        .filter(|s| s.mode != MODE_GITLINK)
        .map(|s| s.oid.to_hex())
        .collect();
    let mut objects =
        cat_file_batch_capped(&repo.root, &readable, MAX_TEXT_BLOB_BYTES)?.into_iter();
    let raws = stages.map(|stage| {
        stage.map(|entry| {
            if entry.mode == MODE_GITLINK {
                return RawStage {
                    entry,
                    size: 0,
                    bytes: None,
                };
            }
            let object = objects.next().expect("one object per readable stage");
            RawStage {
                entry,
                size: object.size,
                bytes: object.bytes,
            }
        })
    });
    document_from_stages(repo, entry, raws)
}

/// Everything after the stages are read, shared with `Git2Backend` (which
/// reads them natively): runs git's 3-way merge on them, reads the labels
/// and the file on disk, and assembles the document.
pub fn document_from_stages(
    repo: &RepoRef,
    entry: ConflictEntry,
    raws: [Option<RawStage>; 3],
) -> Result<MergeDocument> {
    let oids = [
        entry.base.map(|s| s.oid),
        entry.ours.map(|s| s.oid),
        entry.theirs.map(|s| s.oid),
    ];
    let (blobs, job) = classify_sides(&entry, raws);
    let merged = match job {
        Some(job) => {
            let output = merge_stage_bytes(repo, &job, oids)?;
            Some((job.marker, output))
        }
        None => None,
    };
    let labels = side_labels(repo);
    let worktree = read_worktree_file(repo, &entry.path);
    MergeDocument::assemble(entry, blobs, merged, labels, worktree.as_deref())
}

/// `git merge-file -p --zdiff3` over the three sides, with a marker size
/// no line can be mistaken for. `--zdiff3` (git 2.35+; `--diff3` before)
/// is git's own merge with the base kept in each conflict and the lines
/// both sides share trimmed off its edges, as `git merge` writes them.
/// Blobs are passed by id on git 2.43+ (`--object-id`) and through
/// temporary files under the git dir before.
fn merge_stage_bytes(
    repo: &RepoRef,
    job: &gitpoulpi_core::domain::MergeJob,
    oids: [Option<Oid>; 3],
) -> Result<Vec<u8>> {
    merge_stage_bytes_with(repo, job, oids, git_at_least(2, 43))
}

/// [`merge_stage_bytes`] with the blob-passing method chosen by the
/// caller (so the pre-2.43 path is testable on a newer git).
fn merge_stage_bytes_with(
    repo: &RepoRef,
    job: &gitpoulpi_core::domain::MergeJob,
    oids: [Option<Oid>; 3],
    object_id: bool,
) -> Result<Vec<u8>> {
    const EMPTY_BLOB: &str = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
    let style = if git_at_least(2, 35) {
        "--zdiff3"
    } else {
        "--diff3"
    };
    let marker = format!("--marker-size={}", job.marker);
    let head = ["merge-file", "-p", style, &marker];
    let labels = ["-L", "ours", "-L", "base", "-L", "theirs"];

    if object_id {
        let base = oids[0].map_or_else(|| EMPTY_BLOB.to_string(), |o| o.to_hex());
        let ours = oids[1].map(|o| o.to_hex()).unwrap_or_default();
        let theirs = oids[2].map(|o| o.to_hex()).unwrap_or_default();
        let mut args: Vec<&str> = head.to_vec();
        args.push("--object-id");
        args.extend(labels);
        args.extend([ours.as_str(), base.as_str(), theirs.as_str()]);
        return run_merge_file(&repo.root, &args);
    }

    let dir = merge_tmp_dir(repo)?;
    let result = (|| {
        let files = [
            ("ours", &job.ours),
            ("base", &job.base),
            ("theirs", &job.theirs),
        ];
        let mut paths = Vec::new();
        for (name, bytes) in files {
            let file = dir.join(name);
            std::fs::write(&file, bytes)?;
            paths.push(file.to_string_lossy().into_owned());
        }
        let mut args: Vec<&str> = head.to_vec();
        args.extend(labels);
        args.extend(paths.iter().map(String::as_str));
        run_merge_file(&repo.root, &args)
    })();
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// A fresh directory under `<git-dir>/gitpoulpi/merge-tmp/`.
fn merge_tmp_dir(repo: &RepoRef) -> Result<PathBuf> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = git_dir(&repo.root)
        .join("gitpoulpi")
        .join("merge-tmp")
        .join(format!("{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// The file on disk at `path`: its bytes, a symlink's target, or `None`
/// when there is nothing there. A file over the size cap is not read; it
/// is reported as empty (so it counts as present, and as edited).
fn read_worktree_file(repo: &RepoRef, path: &Path) -> Option<Vec<u8>> {
    let full = repo.root.join(path);
    let meta = std::fs::symlink_metadata(&full).ok()?;
    if meta.file_type().is_symlink() {
        return std::fs::read_link(&full)
            .ok()
            .map(|t| t.to_string_lossy().into_owned().into_bytes());
    }
    if !meta.is_file() {
        return None;
    }
    if meta.len() > MAX_TEXT_BLOB_BYTES {
        return Some(Vec::new());
    }
    std::fs::read(&full).ok()
}

// -- labels --------------------------------------------------------------------------------

/// The git directory of the worktree at `root`: `.git` itself, or what a
/// linked worktree's (or submodule's) `.git` file points at.
pub fn git_dir(root: &Path) -> PathBuf {
    let dot_git = root.join(".git");
    if dot_git.is_file() {
        if let Some(target) = std::fs::read_to_string(&dot_git).ok().and_then(|t| {
            t.trim()
                .strip_prefix("gitdir:")
                .map(|p| p.trim().to_string())
        }) {
            let target = PathBuf::from(target);
            return if target.is_absolute() {
                target
            } else {
                root.join(target)
            };
        }
    }
    dot_git
}

pub(crate) fn read_trimmed(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

pub(crate) fn short(sha: &str) -> &str {
    sha.get(..7).unwrap_or(sha)
}

fn branch_of(refname: &str) -> &str {
    refname.strip_prefix("refs/heads/").unwrap_or(refname)
}

/// The pane labels for whatever operation is stopped, read from the git
/// directory without a process (plan 3.5). Roles and refs, never a bare
/// ours / theirs: a rebase swaps what they mean.
pub fn side_labels(repo: &RepoRef) -> SideLabels {
    let dir = git_dir(&repo.root);
    let head = read_trimmed(&dir.join("HEAD")).unwrap_or_default();
    let head_branch = head
        .strip_prefix("ref: ")
        .map(|r| branch_of(r).to_string())
        .unwrap_or_else(|| short(&head).to_string());
    let message_first_line = || {
        read_trimmed(&dir.join("MERGE_MSG"))
            .and_then(|m| m.lines().next().map(str::to_string))
            .unwrap_or_default()
    };
    let mut labels = SideLabels {
        base: "Base".to_string(),
        git_ours_label: "HEAD".to_string(),
        ..SideLabels::default()
    };

    if let Some(sha) = read_trimmed(&dir.join("CHERRY_PICK_HEAD")) {
        let summary = message_first_line();
        labels.op = ConflictOperation::CherryPick;
        labels.current = format!("Current · {head_branch}");
        labels.incoming = format!("Incoming · {} {summary}", short(&sha))
            .trim_end()
            .to_string();
        labels.description = format!("cherry-picking {}", short(&sha));
        labels.git_theirs_label = format!("{} ({summary})", short(&sha));
    } else if let Some(sha) = read_trimmed(&dir.join("REVERT_HEAD")) {
        let line = message_first_line();
        let summary = line
            .strip_prefix("Revert \"")
            .and_then(|l| l.strip_suffix('"'))
            .unwrap_or(&line)
            .to_string();
        labels.op = ConflictOperation::Revert;
        labels.current = format!("Current · {head_branch}");
        labels.incoming = format!("Incoming · revert of {} {summary}", short(&sha))
            .trim_end()
            .to_string();
        labels.description = format!("reverting {}", short(&sha));
        labels.git_theirs_label = format!("parent of {} ({summary})", short(&sha));
    } else if dir.join("rebase-merge").is_dir() {
        let rebase = dir.join("rebase-merge");
        let onto = read_trimmed(&rebase.join("onto")).unwrap_or_default();
        let branch = read_trimmed(&rebase.join("head-name")).unwrap_or_default();
        let stopped = read_trimmed(&dir.join("REBASE_HEAD"))
            .or_else(|| read_trimmed(&rebase.join("stopped-sha")))
            .unwrap_or_default();
        let summary = read_trimmed(&rebase.join("message"))
            .and_then(|m| m.lines().next().map(str::to_string))
            .unwrap_or_default();
        let step = read_trimmed(&rebase.join("msgnum"))
            .zip(read_trimmed(&rebase.join("end")))
            .map(|(n, end)| format!(" · step {n}/{end}"))
            .unwrap_or_default();
        labels.op = ConflictOperation::Rebase;
        labels.current = format!("Current · onto {}", short(&onto));
        labels.incoming = format!("Incoming · your commit {} {summary}", short(&stopped))
            .trim_end()
            .to_string();
        labels.description = format!(
            "rebasing {} onto {}{step}",
            branch_of(&branch),
            short(&onto)
        );
        labels.git_theirs_label = format!("{} ({summary})", short(&stopped));
    } else if let Some(sha) = read_trimmed(&dir.join("MERGE_HEAD")) {
        // `MERGE_HEAD` has one line per merged commit; the message names
        // the branch ("Merge branch 'x'", "Merge remote-tracking branch
        // 'origin/x'", "Merge tag 'v1'").
        let sha = sha.lines().next().unwrap_or_default().to_string();
        let message = message_first_line();
        let name = message
            .split('\'')
            .nth(1)
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| short(&sha))
            .to_string();
        labels.op = ConflictOperation::Merge;
        labels.current = format!("Current · {head_branch}");
        labels.incoming = format!("Incoming · {name}");
        labels.description = format!("merging {name} into {head_branch}");
        labels.git_theirs_label = name;
    } else {
        labels.current = "Current · working tree".to_string();
        labels.incoming = "Incoming · stash".to_string();
        labels.git_theirs_label = "Stashed changes".to_string();
    }
    labels
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_testkit::ConflictFixture;

    #[test]
    fn unmerged_records_fold_into_one_entry_per_path() {
        let a = "a".repeat(40);
        let b = "b".repeat(40);
        let raw = format!(
            "100644 {a} 1\tx y.txt\0100644 {a} 2\tx y.txt\0100755 {b} 3\tx y.txt\0\
             100644 {b} 2\tz\0"
        );
        let entries = parse_unmerged(&raw).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path.to_str(), Some("x y.txt"));
        assert_eq!(
            entries[0].kind,
            gitpoulpi_core::domain::ConflictKind::BothModified
        );
        assert_eq!(entries[0].theirs.unwrap().mode, 0o100755);
        assert_eq!(
            entries[1].kind,
            gitpoulpi_core::domain::ConflictKind::AddedByUs
        );
        assert!(parse_unmerged("").unwrap().is_empty());
        assert!(parse_unmerged("junk\0").is_err());
    }

    /// Before git 2.43 `merge-file` cannot take object ids, so the blobs go
    /// through temporary files: the same merge, and nothing left behind.
    #[test]
    fn the_temp_file_path_merges_like_the_object_id_path() {
        let fx = ConflictFixture::text();
        let repo = fx.repo_ref();
        let entry = entry_for(&repo, "notes.txt").unwrap();
        let stage = |s: Option<StageEntry>| {
            let s = s.unwrap();
            let bytes = fx.stage_bytes(
                if s == entry.ours.unwrap() {
                    2
                } else if s == entry.theirs.unwrap() {
                    3
                } else {
                    1
                },
                "notes.txt",
            );
            RawStage {
                entry: s,
                size: bytes.as_ref().map_or(0, |b| b.len() as u64),
                bytes,
            }
        };
        let (_, job) = classify_sides(
            &entry,
            [
                Some(stage(entry.base)),
                Some(stage(entry.ours)),
                Some(stage(entry.theirs)),
            ],
        );
        let job = job.unwrap();
        let oids = [
            entry.base.map(|s| s.oid),
            entry.ours.map(|s| s.oid),
            entry.theirs.map(|s| s.oid),
        ];
        let by_file = merge_stage_bytes_with(&repo, &job, oids, false).unwrap();
        // `--object-id` is git 2.43+: older git only has the temp-file path.
        if crate::process::git_at_least(2, 43) {
            let by_id = merge_stage_bytes_with(&repo, &job, oids, true).unwrap();
            assert_eq!(by_id, by_file);
        }
        let tmp = git_dir(&repo.root).join("gitpoulpi").join("merge-tmp");
        assert_eq!(std::fs::read_dir(tmp).unwrap().count(), 0, "cleaned up");
    }

    #[test]
    fn a_linked_worktrees_git_file_is_followed() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".git"),
            "gitdir: ../main/.git/worktrees/w\n",
        )
        .unwrap();
        assert_eq!(
            git_dir(dir.path()),
            dir.path().join("../main/.git/worktrees/w")
        );
    }

    #[test]
    fn git_versions_parse() {
        assert_eq!(
            crate::process::parse_git_version("git version 2.45.2.windows.1\n"),
            Some((2, 45))
        );
        assert_eq!(crate::process::parse_git_version("nonsense"), None);
    }
}
