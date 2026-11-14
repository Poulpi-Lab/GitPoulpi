//! The history rewrite engine (the history-rewrite design,
//! "Engine"): own plumbing, a fixed number of `git` processes whatever the
//! size of the history.
//!
//! Preview: `rev-parse`, `rev-list`, one `cat-file --batch`; every new
//! commit is hashed here and written into a temporary pack under the git
//! dir - nothing is in the object store yet. Apply: one `rev-parse` to check
//! the refs still are where the preview saw them, `index-pack` (which
//! recomputes every id), then one `update-ref --stdin` transaction with
//! compare-and-swap on every old value.

mod analyze;
mod blobs;
mod encoding;
mod engine;
mod extract;
mod impact;
mod lfs;
mod mailmap;
mod moves;
mod notes;
mod objects;
mod pack;
mod plumbing;
mod purge;
pub(crate) mod push;
mod refs;
mod rules;
mod scan;
mod script;
mod serialize;
mod sign;
mod store;
mod trees;
mod walk;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use gitpoulpi_core::domain::rewrite::{
    Recipe, RewriteMaps, RewriteOutcome, RewritePlan, RewriteProgressSink, RewriteRecord,
    RewriteRefusal, RunId,
};
use gitpoulpi_core::domain::{Oid, RepoRef};
use gitpoulpi_core::{Error, Result};

use crate::process::{failed_with, git, run_with_stdin, spawn_failed};
use walk::refuse;

pub(crate) use mailmap::{
    add as mailmap_add, identities as mailmap_identities, resolve as mailmap_resolve,
};

pub(crate) fn preview(
    repo: &RepoRef,
    recipe: &Recipe,
    probe: &crate::tools::ToolProbe,
    sink: RewriteProgressSink<'_>,
) -> Result<RewritePlan> {
    engine::preview(&repo.root, recipe, probe, sink)
}

/// The "Check signing" smoke test (`rewrite_check_signing`).
pub(crate) fn check_signing(
    repo: &RepoRef,
    probe: &crate::tools::ToolProbe,
) -> Result<gitpoulpi_core::domain::rewrite::SigningCheck> {
    Ok(sign::check(&repo.root, probe))
}

/// The git dir without a spawn when `root` has a plain `.git` directory.
pub(super) fn git_dir(root: &Path) -> Result<PathBuf> {
    let plain = root.join(".git");
    if plain.is_dir() {
        return Ok(plain);
    }
    let out = crate::process::run(root, &["rev-parse", "--absolute-git-dir"])?;
    Ok(PathBuf::from(out.trim()))
}

fn stale(detail: &str) -> Error {
    refuse(RewriteRefusal::Stale, detail)
}

pub(crate) fn discard(repo: &RepoRef, run: &RunId) -> Result<()> {
    if !store::valid_run(run) {
        return Ok(());
    }
    store::remove_run(&store::run_dir(&git_dir(&repo.root)?, run))
}

/// A preview's plan, without its rows, rebuilt from its run directory: how a
/// preview outlives the process that made it. `None` when the run is gone,
/// older than the plan file's recipe, or its file no longer hashes to its own
/// digest (tampered): the caller answers `stale`.
pub(crate) fn load(repo: &RepoRef, run: &RunId) -> Result<Option<RewritePlan>> {
    if !store::valid_run(run) {
        return Ok(None);
    }
    let dir = store::run_dir(&git_dir(&repo.root)?, run);
    let Some(stored) = store::read_plan(&dir) else {
        return Ok(None);
    };
    let Some(recipe) = &stored.recipe else {
        return Ok(None);
    };
    if engine::digest(recipe, &stored.refs) != stored.digest || !dir.join(store::PACK).is_file() {
        return Ok(None);
    }
    Ok(Some(RewritePlan {
        run: run.clone(),
        digest: stored.digest,
        counts: stored.counts,
        refs: stored.refs,
        warnings: stored.warnings,
        per_rule: stored.per_rule,
        rows: Vec::new(),
    }))
}

/// `rewrite_extract`: see `extract.rs`.
pub(crate) fn extract_to(
    repo: &RepoRef,
    plan: &RewritePlan,
    target: &Path,
    probe: &crate::tools::ToolProbe,
) -> Result<RewriteOutcome> {
    extract::extract(repo, plan, target, probe, &mut |_| {
        std::ops::ControlFlow::Continue(())
    })
}

pub(crate) fn apply(repo: &RepoRef, plan: &RewritePlan) -> Result<RewriteOutcome> {
    let root = &repo.root;
    if !store::valid_run(&plan.run) {
        return Err(stale("the plan names no run"));
    }
    if plan.refs.is_empty() {
        // The preview changed nothing: no pack, no ref.
        discard(repo, &plan.run)?;
        return Ok(RewriteOutcome {
            run: plan.run.clone(),
            moved: Vec::new(),
            commits_changed: 0,
        });
    }

    // Where the refs are now, and the git dir, from one `rev-parse`.
    let names: Vec<&str> = plan.refs.iter().map(|r| r.name.as_str()).collect();
    let mut args = vec!["rev-parse", "--absolute-git-dir"];
    args.extend(names.iter().copied());
    let now = git(root, &args).output().map_err(spawn_failed)?;
    if !now.status.success() {
        return Err(stale("a ref of the plan is gone"));
    }
    let text = String::from_utf8_lossy(&now.stdout);
    let mut lines = text.lines();
    let git_dir = PathBuf::from(lines.next().unwrap_or_default());
    let current: Vec<Option<Oid>> = lines.map(|l| Oid::from_hex(l.trim())).collect();
    let dir = store::run_dir(&git_dir, &plan.run);
    let stored = store::read_plan(&dir).ok_or_else(|| stale("the preview is gone"))?;
    if stored.digest != plan.digest || stored.refs != plan.refs {
        return Err(stale("the plan is not the one that was previewed"));
    }
    // A plan that extracts into a new repository describes that repository's
    // refs: applied here it would rewrite this one. It goes through
    // `rewrite_extract` with a folder.
    if stored
        .recipe
        .as_ref()
        .and_then(|r| r.extraction().map(|(_, mode)| mode))
        .is_some_and(|mode| {
            matches!(
                mode,
                Some(
                    gitpoulpi_core::domain::rewrite::ExtractMode::Clone
                        | gitpoulpi_core::domain::rewrite::ExtractMode::Submodule
                )
            )
        })
    {
        return Err(refuse(
            RewriteRefusal::InvalidRule,
            "this plan extracts into a new repository: apply it with the folder of that repository",
        ));
    }
    for (r, now) in plan.refs.iter().zip(&current) {
        if *now != Some(r.old) {
            return Err(stale(&format!("{} moved since the preview", r.name)));
        }
    }

    // Objects first: a failure from here on leaves unreachable objects
    // (gc collects them), never a half-moved history.
    let pack =
        File::open(dir.join(store::PACK)).map_err(|_| stale("the preview's pack is gone"))?;
    // A run that created no object (every change dropped a commit) has an
    // empty pack, which index-pack has nothing to do with.
    if pack
        .metadata()
        .map_or(true, |m| m.len() > pack::EMPTY_PACK_LEN)
    {
        let index_args = ["index-pack", "--stdin"];
        let indexed = git(root, &index_args)
            .stdin(Stdio::from(pack))
            .output()
            .map_err(spawn_failed)?;
        if !indexed.status.success() {
            return Err(failed_with(
                &index_args,
                indexed.status.code(),
                &String::from_utf8_lossy(&indexed.stderr),
            ));
        }
    }

    // S21: the LFS objects of an import go into the local store before any ref
    // moves (a failure leaves unreferenced objects, never pointers without
    // content), and the filters are there to smudge them on checkout.
    let imported = stored.counts.lfs_objects > 0;
    if imported {
        let storage = lfs::storage_dir(root, &gitpoulpi_core::paths::common_dir(&git_dir));
        lfs::install(&dir, &storage)?;
        if !crate::lfs::filters_configured(repo)? {
            crate::process::run(root, &["lfs", "install", "--local"])?;
        }
    }

    // One transaction, compare-and-swap on every old value. The checked-out
    // branch moves through `HEAD`, so HEAD's reflog says what happened.
    let head_ref = std::fs::read_to_string(git_dir.join("HEAD"))
        .ok()
        .and_then(|t| t.trim().strip_prefix("ref:").map(|n| n.trim().to_string()));
    let mut input = String::new();
    let mut undo = String::new();
    let mut head_move = None;
    for r in &plan.refs {
        let Some(new) = r.new else {
            // Nothing is left of it in the new history (an extraction).
            input.push_str(&format!("delete {} {}\n", r.name, r.old.to_hex()));
            undo.push_str(&format!("create {} {}\n", r.name, r.old.to_hex()));
            continue;
        };
        let through_head = r.name == "HEAD" || head_ref.as_deref() == Some(r.name.as_str());
        if through_head {
            head_move = Some((r.old, new));
        }
        let target = if through_head {
            "HEAD"
        } else {
            r.name.as_str()
        };
        input.push_str(&format!(
            "update {target} {} {}\n",
            new.to_hex(),
            r.old.to_hex()
        ));
        undo.push_str(&format!(
            "update {target} {} {}\n",
            r.old.to_hex(),
            new.to_hex()
        ));
    }
    if let Some((old, _)) = head_move {
        input.push_str(&format!("update ORIG_HEAD {}\n", old.to_hex()));
    }
    let log = "rewrite history";
    run_with_stdin(root, &["update-ref", "-m", log, "--stdin"], &input)
        .map_err(|e| stale(&format!("a ref moved during the apply: {e}")))?;

    // The checked-out commit has another tree: bring the index and the
    // working tree along (a two-tree merge keeps local edits to files that
    // did not change). If git refuses, the refs go back.
    if let (true, Some((old, new))) = (stored.counts.head_tree_changed, head_move) {
        let (old, new) = (old.to_hex(), new.to_hex());
        let args = ["read-tree", "-m", "-u", old.as_str(), new.as_str()];
        let updated = git(root, &args).output().map_err(spawn_failed)?;
        if !updated.status.success() {
            let why = String::from_utf8_lossy(&updated.stderr).trim().to_string();
            return match run_with_stdin(root, &["update-ref", "-m", log, "--stdin"], &undo) {
                Ok(_) => Err(refuse(
                    RewriteRefusal::DirtyPathsTouched,
                    &format!("git could not update the working tree, the refs were put back: {why}"),
                )),
                Err(e) => Err(Error::Backend(format!(
                    "git could not update the working tree ({why}) and the refs could not be put back: {e}"
                ))),
            };
        }
    }

    if imported && stored.counts.head_tree_changed {
        // The files on disk are the real content, not the pointers (a smudge
        // that did not run, or ran before the new `.gitattributes` was there).
        // Best effort: the refs have moved either way.
        let _ = git(root, &["lfs", "checkout"]).output();
        // `lfs checkout` rewrites the files after git recorded their stat:
        // refresh the entries once so `status` starts clean (Windows keeps
        // them flagged as modified otherwise).
        let _ = git(root, &["update-index", "-q", "--refresh"]).output();
    }

    store::write_ref_map(&dir, &plan.refs)?;
    store::write_moved(&dir, &plan.refs)?;
    store::drop_temporaries(&dir);
    Ok(RewriteOutcome {
        run: plan.run.clone(),
        moved: plan.refs.clone(),
        commits_changed: plan.counts.commits_changed,
    })
}

pub(crate) fn scan_secrets(
    repo: &RepoRef,
    scope: &gitpoulpi_core::domain::rewrite::Scope,
    sink: RewriteProgressSink<'_>,
) -> Result<gitpoulpi_core::domain::rewrite::SecretScan> {
    scan::scan(repo, scope, sink)
}

pub(crate) fn analyze(
    repo: &RepoRef,
    scope: &gitpoulpi_core::domain::rewrite::Scope,
    top: u32,
    sink: RewriteProgressSink<'_>,
) -> Result<gitpoulpi_core::domain::rewrite::SizeAnalysis> {
    analyze::analyze(repo, scope, top, sink)
}

/// What an applied run moved, from its run directory.
pub(crate) fn record(repo: &RepoRef, run: &RunId) -> Result<RewriteRecord> {
    if !store::valid_run(run) {
        return Err(stale("the request names no run"));
    }
    let git_dir = git_dir(&repo.root)?;
    let run = &store::resolve_run(&git_dir, run)?;
    let dir = store::run_dir(&git_dir, run);
    let refs = store::read_moved(&dir)
        .ok_or_else(|| stale("this run was not applied here, or its files are gone"))?;
    let on_remote = push::on_remote_names(repo, &refs)?;
    Ok(RewriteRecord {
        run: run.clone(),
        dir,
        refs,
        on_remote,
    })
}

/// The run's maps, copied to `out` when given.
pub(crate) fn maps(repo: &RepoRef, run: &RunId, out: Option<&Path>) -> Result<RewriteMaps> {
    if !store::valid_run(run) {
        return Err(stale("the request names no run"));
    }
    let git_dir = git_dir(&repo.root)?;
    let run = &store::resolve_run(&git_dir, run)?;
    let dir = store::run_dir(&git_dir, run);
    let (commit_map, ref_map) = (dir.join(store::COMMIT_MAP), dir.join(store::REF_MAP));
    if !commit_map.is_file() || !ref_map.is_file() {
        return Err(stale(
            "this run has no maps: it was not applied here, or they were swept",
        ));
    }
    let Some(out) = out else {
        return Ok(RewriteMaps {
            run: run.clone(),
            dir,
            commit_map,
            ref_map,
            copied: false,
        });
    };
    std::fs::create_dir_all(out)
        .map_err(|e| Error::Backend(format!("cannot create {}: {e}", out.display())))?;
    let copy = |from: &Path, name: &str| -> Result<PathBuf> {
        let to = out.join(name);
        std::fs::copy(from, &to)
            .map_err(|e| Error::Backend(format!("cannot write {}: {e}", to.display())))?;
        Ok(to)
    };
    Ok(RewriteMaps {
        run: run.clone(),
        dir: out.to_path_buf(),
        commit_map: copy(&commit_map, store::COMMIT_MAP)?,
        ref_map: copy(&ref_map, store::REF_MAP)?,
        copied: true,
    })
}

pub(crate) use purge::purge;
pub(crate) use push::push_rewritten;

pub(crate) fn script_check(
    repo: &RepoRef,
    script: &gitpoulpi_core::domain::rewrite::ScriptRef,
    policy: &gitpoulpi_core::domain::rewrite::ScriptPolicy,
    probe: &crate::tools::ToolProbe,
) -> Result<gitpoulpi_core::domain::rewrite::script::ScriptCheck> {
    script::check(&repo.root, script, script::ScriptEnv { policy, probe })
}

pub(crate) fn script_test(
    repo: &RepoRef,
    script: &gitpoulpi_core::domain::rewrite::ScriptRef,
    policy: &gitpoulpi_core::domain::rewrite::ScriptPolicy,
    probe: &crate::tools::ToolProbe,
    commits: &[Oid],
) -> Result<Vec<gitpoulpi_core::domain::rewrite::script::ScriptTestRow>> {
    script::test(
        &repo.root,
        script,
        script::ScriptEnv { policy, probe },
        commits,
    )
}

pub(crate) fn blob_diff(
    repo: &RepoRef,
    run: &RunId,
    commit: Oid,
    path: &str,
) -> Result<gitpoulpi_core::domain::rewrite::BlobChange> {
    engine::blob_diff(&repo.root, run, commit, path)
}
