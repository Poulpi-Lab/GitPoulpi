//! Extracting a subdirectory as a new repository (`ExtractMode::Clone` and
//! `ExtractMode::Submodule`; the in-place mode is the ordinary apply).
//!
//! The new repository is a `git clone --no-local` of this one, rewritten
//! there with the recipe over every local branch and tag, then cleaned: the
//! remote and its tracking refs go, the reflogs expire and `gc --prune=now`
//! drops the old objects, so it holds only the extracted history. This
//! repository is not touched in clone mode (the preview's run directory is
//! removed too). In submodule mode its history is kept as it is and one
//! commit on the current branch replaces the folder by a submodule of the
//! new repository: the history is not rewritten here because the commits
//! before that one still hold the folder's files, and other clones of this
//! repository (and every tag) keep pointing at them; a rewrite would force
//! everyone to recover for a change that does not need it.

use std::path::Path;

use gitpoulpi_core::domain::rewrite::{
    ExtractMode, PathMove, PushState, Recipe, RefChange, RewriteOutcome, RewritePlan,
    RewriteProgressSink, RewriteRefusal, RuleKind,
};
use gitpoulpi_core::domain::{Oid, RepoRef};
use gitpoulpi_core::{Error, Result};

use super::walk::{refuse, Head};
use super::{engine, git_dir, stale, store};
use crate::process::{git, run, spawn_failed};

/// The folder and the mode of the recipe's extraction, when it is one that
/// leaves this repository alone.
fn elsewhere(recipe: &Recipe) -> Option<(String, ExtractMode)> {
    match recipe.extraction() {
        Some((dir, Some(mode @ (ExtractMode::Clone | ExtractMode::Submodule)))) => {
            Some((dir.trim_matches('/').to_string(), mode))
        }
        _ => None,
    }
}

/// The recipe the new repository runs: the same rules, the extraction in
/// place (there the clone's own history is the one being rewritten).
fn clone_recipe(recipe: &Recipe) -> Recipe {
    let mut r = recipe.clone();
    for rule in &mut r.rules {
        if let RuleKind::MovePaths(PathMove::ExtractSubdir { mode, .. }) = &mut rule.kind {
            *mode = Some(ExtractMode::InPlace);
        }
    }
    r.scope = gitpoulpi_core::domain::rewrite::Scope::AllLocal;
    r.scope_options.include_tags = true;
    r.scope_options.include_protected.clear();
    r
}

fn io(what: &str, path: &Path, e: std::io::Error) -> Error {
    Error::Backend(format!("{what} {}: {e}", path.display()))
}

/// Whether `target` can receive a new repository: absent, or an empty folder.
fn check_target(root: &Path, target: &Path) -> Result<()> {
    if target.as_os_str().is_empty() {
        return Err(refuse(
            RewriteRefusal::InvalidRule,
            "choose the folder of the new repository",
        ));
    }
    let absolute = std::path::absolute(target).map_err(|e| io("cannot resolve", target, e))?;
    let inside = std::fs::canonicalize(root).ok().is_some_and(|r| {
        let mut probe = absolute.clone();
        while !probe.exists() {
            if !probe.pop() {
                break;
            }
        }
        std::fs::canonicalize(&probe).is_ok_and(|p| p.starts_with(&r))
    });
    if inside {
        return Err(refuse(
            RewriteRefusal::InvalidRule,
            "the new repository cannot be inside this one",
        ));
    }
    match std::fs::read_dir(target) {
        Ok(mut entries) => {
            if entries.next().is_some() {
                return Err(refuse(
                    RewriteRefusal::InvalidRule,
                    &format!("{} is not empty", target.display()),
                ));
            }
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(refuse(
            RewriteRefusal::InvalidRule,
            &format!("{} is not a folder", target.display()),
        )),
    }
}

/// Every local branch: `(full name, tip)`.
fn local_branches(root: &Path) -> Result<Vec<(String, String)>> {
    let out = run(
        root,
        &[
            "for-each-ref",
            "--format=%(refname) %(objectname)",
            "refs/heads/",
        ],
    )?;
    Ok(out
        .lines()
        .filter_map(|l| l.split_once(' '))
        .map(|(n, o)| (n.to_string(), o.to_string()))
        .collect())
}

/// A path as a git argument: never with Windows' verbatim prefix
/// (`\\?\C:\x`), which git reads as a host name (`hostname contains invalid
/// characters`).
fn git_arg_path(path: &Path) -> String {
    gitpoulpi_core::paths::strip_verbatim_prefix(path)
        .to_string_lossy()
        .into_owned()
}

/// The clone: `--no-local` (a real transfer, so the new repository shares no
/// object file with this one), a local branch for each local branch here,
/// no remote.
fn clone_here(root: &Path, target: &Path) -> Result<()> {
    let target_text = git_arg_path(target);
    let root_text = git_arg_path(root);
    run(
        root,
        &[
            "clone",
            "--quiet",
            "--no-local",
            "--",
            &root_text,
            &target_text,
        ],
    )?;
    let mut input = String::new();
    for (name, tip) in local_branches(root)? {
        input.push_str(&format!("update {name} {tip}\n"));
    }
    crate::process::run_with_stdin(target, &["update-ref", "--stdin"], &input)?;
    run(target, &["remote", "remove", "origin"])?;
    Ok(())
}

/// Drops the old history from the clone: reflogs, ORIG_HEAD and the
/// preview's bookkeeping go, then everything unreachable.
fn clean_clone(target: &Path) -> Result<()> {
    let _ = run(target, &["update-ref", "-d", "ORIG_HEAD"]);
    if let Ok(dir) = git_dir(target) {
        let _ = std::fs::remove_dir_all(dir.join("gitpoulpi"));
    }
    run(target, &["reflog", "expire", "--expire=now", "--all"])?;
    run(target, &["gc", "--prune=now", "--quiet"])?;
    Ok(())
}

/// Rewrites the clone with the recipe and applies it there.
fn rewrite_clone(
    target: &Path,
    recipe: &Recipe,
    probe: &crate::tools::ToolProbe,
    sink: RewriteProgressSink<'_>,
) -> Result<RewritePlan> {
    let plan = engine::preview(target, &clone_recipe(recipe), probe, sink)?;
    let repo = RepoRef::from_root(target);
    super::apply(&repo, &plan)?;
    clean_clone(target)?;
    Ok(plan)
}

/// What stands between a submodule commit and a clean result: edits or
/// untracked files in the folder, edits to `.gitmodules`.
fn refuse_dirty(root: &Path, dir: &str) -> Result<()> {
    let out = git(
        root,
        &["status", "--porcelain=v1", "-z", "--", dir, ".gitmodules"],
    )
    .env("GIT_OPTIONAL_LOCKS", "0")
    .output()
    .map_err(spawn_failed)?;
    if !out.status.success() {
        return Err(Error::Backend(format!(
            "git status failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    let dirty: Vec<String> = out
        .stdout
        .split(|b| *b == 0)
        .filter(|r| r.len() > 3)
        .map(|r| String::from_utf8_lossy(&r[3..]).into_owned())
        .collect();
    if dirty.is_empty() {
        return Ok(());
    }
    const LISTED: usize = 50;
    let more = dirty.len().saturating_sub(LISTED);
    let mut detail = dirty[..dirty.len() - more].join("\n");
    if more > 0 {
        detail.push_str("\n...");
    }
    Err(refuse(RewriteRefusal::DirtyPathsTouched, &detail))
}

/// Puts the folder, the index entry and `.gitmodules` back as HEAD has them
/// after a submodule that could not be finished.
fn roll_back(root: &Path, dir: &str) {
    let _ = run(
        root,
        &["reset", "--quiet", "HEAD", "--", dir, ".gitmodules"],
    );
    let _ = std::fs::remove_dir_all(root.join(dir));
    let _ = run(root, &["checkout", "--quiet", "--", dir]);
    if run(root, &["cat-file", "-e", "HEAD:.gitmodules"]).is_ok() {
        let _ = run(root, &["checkout", "--quiet", "--", ".gitmodules"]);
    } else {
        let _ = std::fs::remove_file(root.join(".gitmodules"));
    }
    let _ = run(
        root,
        &["config", "--remove-section", &format!("submodule.{dir}")],
    );
    if let Ok(git) = git_dir(root) {
        let _ = std::fs::remove_dir_all(git.join("modules").join(dir));
    }
}

/// The folder becomes a submodule of `target`: one commit on the current
/// branch.
fn replace_with_submodule(root: &Path, dir: &str, target: &Path, head: &str) -> Result<RefChange> {
    let old = Oid::from_hex(run(root, &["rev-parse", "HEAD"])?.trim())
        .ok_or_else(|| Error::Backend("HEAD is not a commit".into()))?;
    let url = git_arg_path(&std::fs::canonicalize(target).unwrap_or_else(|_| target.to_path_buf()));
    let made = (|| -> Result<Oid> {
        run(root, &["rm", "-r", "-q", "--", dir])?;
        // A local path is the file transport, which git refuses for
        // submodules unless it is allowed (here, for this one command).
        run(
            root,
            &[
                "-c",
                "protocol.file.allow=always",
                "submodule",
                "add",
                "--quiet",
                "--",
                &url,
                dir,
            ],
        )?;
        let message = format!(
            "Move {dir} into its own repository as a submodule\n\n\
             The history of {dir} lives in {url}; the commits before this one keep the \
             folder's files as they were."
        );
        run(
            root,
            &[
                "commit",
                "--quiet",
                "--only",
                "-m",
                &message,
                "--",
                ".gitmodules",
                dir,
            ],
        )?;
        Oid::from_hex(run(root, &["rev-parse", "HEAD"])?.trim())
            .ok_or_else(|| Error::Backend("HEAD is not a commit".into()))
    })();
    let new = match made {
        Ok(new) => new,
        Err(e) => {
            roll_back(root, dir);
            return Err(e);
        }
    };
    Ok(RefChange {
        name: head.to_string(),
        old,
        new: Some(new),
        pushed: PushState::LocalOnly,
        protected: false,
        included: true,
        checked_out_in: None,
    })
}

pub(crate) fn extract(
    repo: &RepoRef,
    plan: &RewritePlan,
    target: &Path,
    probe: &crate::tools::ToolProbe,
    sink: RewriteProgressSink<'_>,
) -> Result<RewriteOutcome> {
    let root = &repo.root;
    if !store::valid_run(&plan.run) {
        return Err(stale("the plan names no run"));
    }
    let gitdir = git_dir(root)?;
    let dir = store::run_dir(&gitdir, &plan.run);
    let stored = store::read_plan(&dir).ok_or_else(|| stale("the preview is gone"))?;
    if stored.digest != plan.digest || stored.refs != plan.refs {
        return Err(stale("the plan is not the one that was previewed"));
    }
    let recipe = stored
        .recipe
        .ok_or_else(|| stale("the preview has no recipe"))?;
    let Some((folder, mode)) = elsewhere(&recipe) else {
        return Err(refuse(
            RewriteRefusal::InvalidRule,
            "this recipe does not extract into a clone or a submodule",
        ));
    };
    check_target(root, target)?;
    // The refs of the plan are the new repository's: here nothing may have
    // moved since the preview either.
    let probe_head = super::walk::head_of(root)?;
    let head_ref = match (&probe_head, mode) {
        (Head::Branch(name), _) => Some(name.clone()),
        (Head::Detached(_), ExtractMode::Submodule) => {
            return Err(refuse(
                RewriteRefusal::InvalidRule,
                "check a branch out first: the submodule commit goes on it",
            ))
        }
        (Head::Detached(_), _) => None,
    };
    if mode == ExtractMode::Submodule {
        refuse_dirty(root, &folder)?;
        if run(root, &["cat-file", "-t", &format!("HEAD:{folder}")])
            .map_or(true, |t| t.trim() != "tree")
        {
            return Err(refuse(
                RewriteRefusal::EmptyScope,
                &format!("{folder} is not in the checked-out commit"),
            ));
        }
    }

    // The new repository; a failure leaves nothing behind.
    let existed = target.exists();
    let built = (|| {
        clone_here(root, target)?;
        rewrite_clone(target, &recipe, probe, sink)
    })();
    let clone_plan = match built {
        Ok(p) => p,
        Err(e) => {
            let _ = std::fs::remove_dir_all(target);
            if existed {
                let _ = std::fs::create_dir_all(target);
            }
            return Err(e);
        }
    };

    let mut outcome = RewriteOutcome {
        run: plan.run.clone(),
        moved: Vec::new(),
        commits_changed: clone_plan.counts.commits_changed,
    };
    if mode == ExtractMode::Submodule {
        let head = head_ref.expect("checked above");
        match replace_with_submodule(root, &folder, target, &head) {
            Ok(change) => {
                outcome.moved.push(change);
                outcome.commits_changed = 1;
            }
            Err(e) => {
                let _ = std::fs::remove_dir_all(target);
                return Err(e);
            }
        }
    }
    // Nothing of the preview is left to apply here.
    store::remove_run(&dir)?;
    Ok(outcome)
}

#[cfg(test)]
mod arg_path_tests {
    use super::*;

    #[test]
    fn a_verbatim_path_never_reaches_git() {
        assert_eq!(
            git_arg_path(Path::new(r"\\?\C:\Users\X\repo")),
            r"C:\Users\X\repo"
        );
        assert_eq!(
            git_arg_path(Path::new(r"\\?\UNC\srv\share\repo")),
            r"\\srv\share\repo"
        );
        assert_eq!(git_arg_path(Path::new("/tmp/repo")), "/tmp/repo");
    }
}
