//! `GitBackend::rebase_plan`/`rebase_interactive`/`rebase_state`/
//! `rebase_continue`/`rebase_skip`/`rebase_abort` over a real `git
//! rebase -i`, driven with an explicit, pre-decided todo list rather
//! than an interactively edited one.
//!
//! The todo list is written to `<gitdir>/gitpoulpi-rebase/todo` (LF
//! bytes) and copied into place by `GIT_SEQUENCE_EDITOR`, so nothing
//! ever opens a real editor: `GIT_SEQUENCE_EDITOR='cp "$GITPOULPI_TODO"'`
//! (git appends the path it generated as the next argument) and
//! `GIT_EDITOR=:` (a no-op, so `squash`'s concatenated message and any
//! other editor prompt is accepted as-is). `Reword` steps are `pick`
//! plus an `exec` line running `git commit --amend -F
//! <gitpoulpi-rebase/msg-<oid>>` -- not the native `reword` command,
//! which would need the editor to actually change anything.
//!
//! Both `upstream` and the range are resolved and validated (ancestry,
//! no merge commits, `validate_plan`) *before* anything touches the
//! repository or the git dir, so every refusal in
//! [`gitpoulpi_core::domain::RebaseRefusal`] leaves HEAD, the branch and
//! the reflog untouched.

use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{
    normalize_reword_message, validate_plan, InteractiveRebase, Oid, PlanCommit, RebaseAction,
    RebaseOutcome, RebasePlanDraft, RebaseRefusal, RebaseState, RepoRef,
};
use gitpoulpi_core::{Error, Result};

use crate::history_ops::{conflicted_paths, head_oid};
use crate::process::{run, run_status_with_env};
use crate::rewrite::IN_PROGRESS_MARKERS;

/// Directory (relative to the git dir) the todo and reword message files
/// live in, and that `rebase_abort`/a completed rebase clean up.
const WORKDIR: &str = "gitpoulpi-rebase";

fn absolute_git_dir(repo: &RepoRef) -> Result<PathBuf> {
    let out = run(&repo.root, &["rev-parse", "--absolute-git-dir"])?;
    Ok(PathBuf::from(out.trim()))
}

fn in_progress(git_dir: &Path) -> bool {
    IN_PROGRESS_MARKERS
        .iter()
        .any(|marker| git_dir.join(marker).exists())
}

fn is_dirty(repo: &RepoRef) -> Result<bool> {
    let out = run(&repo.root, &["status", "--porcelain", "-uno"])?;
    Ok(!out.trim().is_empty())
}

/// The linear range git would replay, oldest first, as `(oid, parent
/// count)` -- the parent count is what flags a merge commit.
fn range_with_parent_counts(repo: &RepoRef, upstream: Option<Oid>) -> Result<Vec<(Oid, usize)>> {
    let head = head_oid(repo)?;
    if let Some(up) = upstream {
        let ok = run_status_with_env(
            &repo.root,
            &["merge-base", "--is-ancestor", &up.to_hex(), &head.to_hex()],
            &[],
        )?;
        if !ok.success {
            return Err(Error::RebaseRefused(RebaseRefusal::NotOnCurrentBranch));
        }
    }
    let range_spec = match upstream {
        Some(up) => format!("{}..{}", up.to_hex(), head.to_hex()),
        None => head.to_hex(),
    };
    let raw = run(
        &repo.root,
        &[
            "rev-list",
            "--reverse",
            "--topo-order",
            "--parents",
            &range_spec,
        ],
    )?;
    let mut out = Vec::new();
    for line in raw.lines() {
        let mut fields = line.split_whitespace();
        let commit = fields
            .next()
            .and_then(Oid::from_hex)
            .ok_or_else(|| Error::Backend("rev-list gave an unreadable line".into()))?;
        let parents = fields.count();
        out.push((commit, parents));
    }
    Ok(out)
}

fn commit_summary(repo: &RepoRef, commit: Oid) -> Result<String> {
    run(&repo.root, &["log", "-1", "--format=%s", &commit.to_hex()]).map(|s| s.trim().to_string())
}

fn commit_message(repo: &RepoRef, commit: Oid) -> Result<String> {
    run(&repo.root, &["log", "-1", "--format=%B", &commit.to_hex()])
}

pub fn rebase_plan(repo: &RepoRef, upstream: Option<Oid>) -> Result<RebasePlanDraft> {
    let range = range_with_parent_counts(repo, upstream)?;
    if range.iter().any(|(_, parents)| *parents > 1) {
        return Err(Error::RebaseRefused(RebaseRefusal::MergeInRange));
    }
    let mut commits = Vec::with_capacity(range.len());
    for (id, _) in range {
        commits.push(PlanCommit {
            id,
            summary: commit_summary(repo, id)?,
            message: commit_message(repo, id)?,
        });
    }
    Ok(RebasePlanDraft { upstream, commits })
}

pub fn rebase_interactive(repo: &RepoRef, req: &InteractiveRebase) -> Result<RebaseOutcome> {
    let git_dir = absolute_git_dir(repo)?;
    if in_progress(&git_dir) {
        return Err(Error::RebaseRefused(RebaseRefusal::OperationInProgress));
    }
    if is_dirty(repo)? {
        return Err(Error::RebaseRefused(RebaseRefusal::UncommittedChanges));
    }

    // Cherry-pick mode replays arbitrary commits onto HEAD: the "range"
    // is the plan's own commits (which must be distinct), and the rebase
    // runs `-i HEAD` so the todo -- not the empty `HEAD..HEAD` range --
    // decides what is applied.
    let head_hex = head_oid(repo)?.to_hex();
    let range_oids: Vec<Oid> = if req.cherry_pick {
        if req.steps.is_empty() {
            return Err(Error::RebaseRefused(RebaseRefusal::PlanMismatch));
        }
        let mut seen = std::collections::HashSet::new();
        for step in &req.steps {
            if !seen.insert(step.commit) {
                return Err(Error::RebaseRefused(RebaseRefusal::PlanMismatch));
            }
            let parents = run(
                &repo.root,
                &["rev-list", "--parents", "-n", "1", &step.commit.to_hex()],
            )
            .map_err(|_| Error::RebaseRefused(RebaseRefusal::PlanMismatch))?;
            if parents.split_whitespace().count() > 2 {
                return Err(Error::RebaseRefused(RebaseRefusal::MergeInRange));
            }
        }
        req.steps.iter().map(|s| s.commit).collect()
    } else {
        let range = range_with_parent_counts(repo, req.upstream)?;
        if range.iter().any(|(_, parents)| *parents > 1) {
            return Err(Error::RebaseRefused(RebaseRefusal::MergeInRange));
        }
        range.iter().map(|(id, _)| *id).collect()
    };
    validate_plan(&range_oids, &req.steps).map_err(Error::RebaseRefused)?;

    let mut steps = req.steps.clone();
    if req.autosquash {
        let mut rows = Vec::with_capacity(steps.len());
        for step in &steps {
            rows.push((step.commit, commit_summary(repo, step.commit)?, step.action));
        }
        gitpoulpi_core::domain::autosquash_order(&mut rows);
        let mut by_oid: std::collections::HashMap<Oid, gitpoulpi_core::domain::RebaseStep> =
            steps.into_iter().map(|s| (s.commit, s)).collect();
        steps = rows
            .into_iter()
            .map(|(oid, _, action)| {
                let mut s = by_oid.remove(&oid).expect("row oid came from steps");
                s.action = action;
                s
            })
            .collect();
    }

    let all_dropped = !steps.is_empty() && steps.iter().all(|s| s.action == RebaseAction::Drop);

    let work_dir = git_dir.join(WORKDIR);
    std::fs::create_dir_all(&work_dir)?;
    let todo_path = work_dir.join("todo");

    let mut todo = String::new();
    if all_dropped {
        todo.push_str("noop\n");
    } else {
        let mut i = 0;
        while i < steps.len() {
            let step = &steps[i];
            i += 1;
            let hex = step.commit.to_hex();
            match step.action {
                RebaseAction::Pick => todo.push_str(&format!(
                    "pick {hex}
"
                )),
                RebaseAction::Edit => todo.push_str(&format!(
                    "edit {hex}
"
                )),
                RebaseAction::Drop => todo.push_str(&format!(
                    "drop {hex}
"
                )),
                RebaseAction::Squash => todo.push_str(&format!(
                    "squash {hex}
"
                )),
                RebaseAction::Fixup => todo.push_str(&format!(
                    "fixup {hex}
"
                )),
                RebaseAction::Reword => {
                    let message = normalize_reword_message(step.message.as_deref().unwrap_or(""))
                        .expect("validate_plan already refused an empty reword message");
                    let msg_path = work_dir.join(format!("msg-{hex}"));
                    std::fs::write(&msg_path, message.as_bytes())?;
                    todo.push_str(&format!(
                        "pick {hex}
"
                    ));
                    // Commits folded into a reworded commit: its message
                    // is the final one (the caller already combined the
                    // squashed messages into it), so every fold is a
                    // fixup and the message is written once the whole
                    // group is applied.
                    while let Some(next) = steps.get(i) {
                        if !matches!(next.action, RebaseAction::Squash | RebaseAction::Fixup) {
                            break;
                        }
                        todo.push_str(&format!(
                            "fixup {}
",
                            next.commit.to_hex()
                        ));
                        i += 1;
                    }
                    todo.push_str(&format!(
                        "exec git commit --amend --no-verify --allow-empty --cleanup=verbatim -F \"$(git rev-parse --git-path {WORKDIR}/msg-{hex})\"
"
                    ));
                }
            }
        }
    }
    std::fs::write(&todo_path, todo.as_bytes())?;

    let todo_path_str = todo_path.to_string_lossy().replace('\\', "/");
    let mut args = vec![
        "rebase",
        "-i",
        "--no-autosquash",
        "--empty=drop",
        "--no-rebase-merges",
    ];
    let onto_hex = req.onto.map(|o| o.to_hex());
    let upstream_hex = req.upstream.map(|o| o.to_hex());
    if req.cherry_pick {
        args.push(&head_hex);
    } else {
        if let Some(onto) = &onto_hex {
            args.push("--onto");
            args.push(onto);
        }
        match &upstream_hex {
            Some(up) => args.push(up),
            None => args.push("--root"),
        }
    }

    let env = [
        ("GIT_SEQUENCE_EDITOR", "cp \"$GITPOULPI_TODO\""),
        ("GITPOULPI_TODO", todo_path_str.as_str()),
        ("GIT_EDITOR", ":"),
    ];
    let output = run_status_with_env(&repo.root, &args, &env)?;
    classify(repo, &git_dir, output.success, &output.stderr)
}

/// The remaining (not yet applied) commit oids from `<state dir>/git-
/// rebase-todo`, oldest-step-first as git wrote them.
fn remaining_from_todo(state_dir: &Path) -> Vec<Oid> {
    let todo_path = state_dir.join("git-rebase-todo");
    let Ok(contents) = std::fs::read_to_string(todo_path) else {
        return Vec::new();
    };
    contents
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let mut fields = line.split_whitespace();
            let cmd = fields.next()?;
            if !matches!(
                cmd,
                "pick" | "edit" | "squash" | "fixup" | "drop" | "reword"
            ) {
                return None;
            }
            fields.next().and_then(Oid::from_hex)
        })
        .collect()
}

fn cleanup(git_dir: &Path) {
    let _ = std::fs::remove_dir_all(git_dir.join(WORKDIR));
}

/// Turns the raw outcome of a `git rebase` invocation into a typed
/// [`RebaseOutcome`], by inspecting the git dir's state files rather than
/// the exit code: a stop for an `edit` step and a stop for a conflict are
/// both non-Completed, and only conflicted paths tell them apart.
fn classify(repo: &RepoRef, git_dir: &Path, success: bool, stderr: &str) -> Result<RebaseOutcome> {
    let rebase_merge = git_dir.join("rebase-merge");
    let rebase_apply = git_dir.join("rebase-apply");
    let state_dir = if rebase_merge.exists() {
        Some(rebase_merge)
    } else if rebase_apply.exists() {
        Some(rebase_apply)
    } else {
        None
    };

    match state_dir {
        None => {
            if success {
                cleanup(git_dir);
                Ok(RebaseOutcome::Completed)
            } else {
                Err(crate::process::failed(&["rebase"], stderr))
            }
        }
        Some(dir) => {
            let conflicts = conflicted_paths(repo)?;
            let remaining = remaining_from_todo(&dir);
            if !conflicts.is_empty() {
                let paused_at = run(&repo.root, &["rev-parse", "REBASE_HEAD"])
                    .ok()
                    .and_then(|s| Oid::from_hex(s.trim()))
                    .or_else(|| {
                        std::fs::read_to_string(dir.join("stopped-sha"))
                            .ok()
                            .and_then(|s| Oid::from_hex(s.trim()))
                    })
                    .ok_or_else(|| Error::Backend("rebase stopped with no REBASE_HEAD".into()))?;
                Ok(RebaseOutcome::Conflicted {
                    paused_at,
                    remaining,
                })
            } else {
                let at = head_oid(repo)?;
                Ok(RebaseOutcome::Stopped { at, remaining })
            }
        }
    }
}

pub fn rebase_state(repo: &RepoRef) -> Result<Option<RebaseState>> {
    let git_dir = absolute_git_dir(repo)?;
    // `rebase-apply/applying` is a `git am`, not a rebase.
    if git_dir.join("rebase-apply").join("applying").exists() {
        return Ok(None);
    }
    let (dir, interactive) = if git_dir.join("rebase-merge").exists() {
        (git_dir.join("rebase-merge"), true)
    } else if git_dir.join("rebase-apply").exists() {
        (git_dir.join("rebase-apply"), false)
    } else {
        return Ok(None);
    };

    let read = |name: &str| std::fs::read_to_string(dir.join(name)).ok();
    let head_name = read("head-name").and_then(|s| {
        let s = s.trim().to_string();
        if s.is_empty() || s == "detached HEAD" {
            None
        } else {
            Some(
                s.strip_prefix("refs/heads/")
                    .map(str::to_string)
                    .unwrap_or(s),
            )
        }
    });
    let onto = read("onto")
        .and_then(|s| Oid::from_hex(s.trim()))
        .ok_or_else(|| Error::Backend("rebase state has no onto".into()))?;
    let (step, total) = if interactive {
        (
            read("msgnum")
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0),
            read("end").and_then(|s| s.trim().parse().ok()).unwrap_or(0),
        )
    } else {
        (
            read("next")
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0),
            read("last")
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0),
        )
    };
    let conflicted = !conflicted_paths(repo)?.is_empty();
    let stopped_at = read("stopped-sha")
        .and_then(|s| Oid::from_hex(s.trim()))
        .or_else(|| {
            run(&repo.root, &["rev-parse", "REBASE_HEAD"])
                .ok()
                .and_then(|s| Oid::from_hex(s.trim()))
        });

    Ok(Some(RebaseState {
        head_name,
        onto,
        stopped_at,
        step,
        total,
        conflicted,
        interactive,
    }))
}

fn require_in_progress(git_dir: &Path) -> Result<()> {
    if !in_progress(git_dir) || git_dir.join("rebase-apply").join("applying").exists() {
        return Err(Error::RebaseRefused(RebaseRefusal::NoRebaseInProgress));
    }
    Ok(())
}

pub fn rebase_continue(repo: &RepoRef) -> Result<RebaseOutcome> {
    let git_dir = absolute_git_dir(repo)?;
    require_in_progress(&git_dir)?;
    let output = run_status_with_env(
        &repo.root,
        &["rebase", "--continue"],
        &[("GIT_EDITOR", ":")],
    )?;
    classify(repo, &git_dir, output.success, &output.stderr)
}

pub fn rebase_skip(repo: &RepoRef) -> Result<RebaseOutcome> {
    let git_dir = absolute_git_dir(repo)?;
    require_in_progress(&git_dir)?;
    let output = run_status_with_env(&repo.root, &["rebase", "--skip"], &[("GIT_EDITOR", ":")])?;
    classify(repo, &git_dir, output.success, &output.stderr)
}

pub fn rebase_abort(repo: &RepoRef) -> Result<()> {
    let git_dir = absolute_git_dir(repo)?;
    require_in_progress(&git_dir)?;
    let output = run_status_with_env(&repo.root, &["rebase", "--abort"], &[])?;
    cleanup(&git_dir);
    if output.success {
        Ok(())
    } else {
        Err(Error::Backend(format!(
            "git rebase --abort failed: {}",
            output.stderr
        )))
    }
}
