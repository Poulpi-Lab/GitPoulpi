//! The run directory: `<git-dir>/gitpoulpi/rewrite/<run>/` holds a preview's
//! temporary pack, the plan Apply checks, and (kept after Apply) the
//! `commit-map` and `ref-map` in git-filter-repo's format. Under the git dir
//! so the pack and the object store share a volume.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::rewrite::{
    Counts, PlanDigest, Recipe, RefChange, RewritePlan, RewriteWarning, RuleCount, RunId,
};
use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::{Error, Result};
use serde::{Deserialize, Serialize};

pub(crate) const PACK: &str = "objects.pack";
pub(crate) const PLAN: &str = "plan.json";
pub(crate) const COMMIT_MAP: &str = "commit-map";
pub(crate) const REF_MAP: &str = "ref-map";

/// What Apply compares its argument with: the digest and the ref moves the
/// preview computed (the rows stay in memory, they can be 100 000).
#[derive(Serialize, Deserialize)]
pub(crate) struct StoredPlan {
    pub(crate) digest: PlanDigest,
    pub(crate) refs: Vec<RefChange>,
    /// What a later process needs to rebuild the plan without its rows (a
    /// preview survives its daemon): the recipe the digest is recomputed from,
    /// and the counts, warnings and per-rule numbers the apply confirms with.
    #[serde(default)]
    pub(crate) recipe: Option<Recipe>,
    #[serde(default)]
    pub(crate) counts: Counts,
    #[serde(default)]
    pub(crate) warnings: Vec<RewriteWarning>,
    #[serde(default)]
    pub(crate) per_rule: Vec<RuleCount>,
}

fn io(what: &str, path: &Path, e: std::io::Error) -> Error {
    Error::Backend(format!("{what} {}: {e}", path.display()))
}

/// A run id is a name we made up: letters, digits and `-` only, so it can
/// never walk out of the run directory.
pub(crate) fn valid_run(run: &RunId) -> bool {
    !run.0.is_empty()
        && run
            .0
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub(crate) fn run_dir(git_dir: &Path, run: &RunId) -> PathBuf {
    git_dir.join("gitpoulpi").join("rewrite").join(&run.0)
}

pub(crate) fn write_plan(dir: &Path, plan: &RewritePlan, recipe: &Recipe) -> Result<()> {
    let stored = StoredPlan {
        digest: plan.digest.clone(),
        refs: plan.refs.clone(),
        // Never the secret values: they are not written to disk.
        recipe: Some(recipe.saved()),
        counts: plan.counts,
        warnings: plan.warnings.clone(),
        per_rule: plan.per_rule.clone(),
    };
    let path = dir.join(PLAN);
    let text = serde_json::to_vec(&stored).map_err(|e| Error::Backend(e.to_string()))?;
    std::fs::write(&path, text).map_err(|e| io("cannot write", &path, e))
}

pub(crate) fn read_plan(dir: &Path) -> Option<StoredPlan> {
    serde_json::from_slice(&std::fs::read(dir.join(PLAN)).ok()?).ok()
}

/// `old new` per rewritten commit, a header line first.
pub(crate) fn write_commit_map(
    dir: &Path,
    order: &[Oid],
    new_id: &HashMap<Oid, Oid>,
    dropped: &HashSet<Oid>,
) -> Result<()> {
    let mut text = format!("{:<40} new\n", "old");
    for old in order {
        if dropped.contains(old) {
            // filter-repo's way to say a commit was pruned.
            text.push_str(&format!("{} {}\n", old.to_hex(), "0".repeat(40)));
        } else if let Some(new) = new_id.get(old) {
            text.push_str(&format!("{} {}\n", old.to_hex(), new.to_hex()));
        }
    }
    let path = dir.join(COMMIT_MAP);
    std::fs::write(&path, text).map_err(|e| io("cannot write", &path, e))
}

/// `old new refname` per moved ref, a header line first.
pub(crate) fn write_ref_map(dir: &Path, refs: &[RefChange]) -> Result<()> {
    let mut text = format!("{:<40} {:<40} ref\n", "old", "new");
    for r in refs {
        let new = r.new.map(|n| n.to_hex()).unwrap_or_else(|| "0".repeat(40));
        text.push_str(&format!("{} {} {}\n", r.old.to_hex(), new, r.name));
    }
    let path = dir.join(REF_MAP);
    std::fs::write(&path, text).map_err(|e| io("cannot write", &path, e))
}

/// What an apply moved, for the push and the note that come after it (the
/// plan file is gone by then).
pub(crate) const MOVED: &str = "moved.json";

pub(crate) fn write_moved(dir: &Path, refs: &[RefChange]) -> Result<()> {
    let path = dir.join(MOVED);
    let text = serde_json::to_vec(refs).map_err(|e| Error::Backend(e.to_string()))?;
    std::fs::write(&path, text).map_err(|e| io("cannot write", &path, e))
}

pub(crate) fn read_moved(dir: &Path) -> Option<Vec<RefChange>> {
    serde_json::from_slice(&std::fs::read(dir.join(MOVED)).ok()?).ok()
}

/// `latest` as a run id: the most recently applied run of this repository
/// (the newest `moved.json`), so a script can push or write the note of the
/// rewrite it just made without remembering the id. Any other id is itself.
pub(crate) fn resolve_run(git_dir: &Path, run: &RunId) -> Result<RunId> {
    if run.0 != "latest" {
        return Ok(run.clone());
    }
    let dir = git_dir.join("gitpoulpi").join("rewrite");
    let newest = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let modified = std::fs::metadata(e.path().join(MOVED))
                .ok()?
                .modified()
                .ok()?;
            Some((modified, e.file_name().to_string_lossy().into_owned()))
        })
        .max();
    match newest {
        Some((_, name)) => Ok(RunId(name)),
        None => Err(Error::RewriteRefused(
            gitpoulpi_core::domain::rewrite::RewriteRefusal::Stale,
            "no rewrite was applied here (or its files were swept)".into(),
        )),
    }
}

/// Removes everything of a run but its maps (and the record of what moved).
pub(crate) fn drop_temporaries(dir: &Path) {
    let _ = std::fs::remove_file(dir.join(PACK));
    let _ = std::fs::remove_file(dir.join(PLAN));
    let _ = std::fs::remove_dir_all(dir.join(super::lfs::DIR));
}

/// Removes a run, maps included. Idempotent.
pub(crate) fn remove_run(dir: &Path) -> Result<()> {
    match std::fs::remove_dir_all(dir) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(io("cannot remove", dir, e)),
    }
}
