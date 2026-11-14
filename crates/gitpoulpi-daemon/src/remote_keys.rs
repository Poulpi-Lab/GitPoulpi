//! A branch's extra push targets and pull sources (`gitpoulpi-push-to` /
//! `gitpoulpi-pull-from` in its config section, see
//! `gitpoulpi_core::domain::remote_group`): the `BranchSetRemotes` write,
//! the `RemoteGroupStatus` read and the `Ref.multi_remote` decoration.
//!
//! Everything here reads local refs and config only: never the network.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use gitpoulpi_core::domain::RepoRef;
use gitpoulpi_core::domain::{
    entries_with_remotes, parse_branch_remotes, BranchRemotes, Ref, RefKind, RemoteGroupStatus,
    RemoteRelation, RemoteTargetState, TargetRef, TargetRole,
};
use gitpoulpi_core::{Error, GitBackend, Result};

fn section_of(branch: &str) -> String {
    format!("branch.{branch}")
}

/// Replaces `branch`'s push targets and pull sources with the given config
/// values (`<remote>` or `<remote>:<branch>`), leaving every other variable
/// of its section (upstream included) alone. The section is rewritten
/// whole, so the journal's capture of it restores it.
pub fn set_remotes(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    branch: &str,
    push_to: &[String],
    pull_from: &[String],
) -> Result<()> {
    let section = section_of(branch);
    let entries = backend.config_section(repo, &section)?;
    let parse = |values: &[String]| -> Result<Vec<TargetRef>> {
        values
            .iter()
            .map(|v| {
                TargetRef::parse(v, branch)
                    .ok_or_else(|| Error::Backend(format!("empty remote in `{v}`")))
            })
            .collect()
    };
    let mut remotes = parse_branch_remotes(&entries, branch);
    remotes.push_to = parse(push_to)?;
    remotes.pull_from = parse(pull_from)?;
    let known: Vec<String> = backend.remotes(repo)?.into_iter().map(|r| r.name).collect();
    for t in remotes.push_to.iter().chain(&remotes.pull_from) {
        if !known.contains(&t.remote) {
            return Err(Error::Backend(format!("no such remote: {}", t.remote)));
        }
    }
    let new = entries_with_remotes(&entries, &remotes);
    if new == entries {
        return Ok(());
    }
    backend.set_config_section(repo, &section, &new)
}

/// Where one target stands against its remote-tracking ref in `refs`.
fn state_of(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    refs: &[Ref],
    local: gitpoulpi_core::domain::Oid,
    target: &TargetRef,
    (role, pull): (TargetRole, bool),
    known: &[String],
) -> RemoteTargetState {
    let tracking = refs.iter().find(|r| match &r.kind {
        RefKind::RemoteBranch { remote, name } => {
            *remote == target.remote && *name == target.branch
        }
        _ => false,
    });
    let relation = match tracking {
        None => RemoteRelation::Absent,
        Some(t) => match backend.ahead_behind(repo, local, t.target) {
            Ok((0, 0)) => RemoteRelation::UpToDate,
            Ok((a, 0)) => RemoteRelation::LocalAhead(a),
            Ok((0, b)) => RemoteRelation::RemoteAhead(b),
            Ok((ahead, behind)) => match backend.merge_base(repo, local, t.target) {
                Ok(None) => RemoteRelation::Unrelated,
                _ => RemoteRelation::Diverged { ahead, behind },
            },
            Err(_) => RemoteRelation::Absent,
        },
    };
    RemoteTargetState {
        remote: target.remote.clone(),
        branch: target.branch.clone(),
        role,
        pull,
        tracking: tracking.map(|_| format!("{}/{}", target.remote, target.branch)),
        relation,
        missing: !known.contains(&target.remote),
    }
}

/// The states of every remote of `remotes` (primary, push targets, then
/// pull-only sources) for a branch whose tip is `local`.
fn states(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    refs: &[Ref],
    local: gitpoulpi_core::domain::Oid,
    remotes: &BranchRemotes,
    known: &[String],
) -> Vec<RemoteTargetState> {
    let mut out = Vec::new();
    if let Some(p) = &remotes.primary {
        out.push(state_of(
            backend,
            repo,
            refs,
            local,
            p,
            (TargetRole::Primary, false),
            known,
        ));
    }
    for t in &remotes.push_to {
        let pull = remotes.pull_from.contains(t);
        out.push(state_of(
            backend,
            repo,
            refs,
            local,
            t,
            (TargetRole::PushTo, pull),
            known,
        ));
    }
    for t in &remotes.pull_from {
        if !remotes.push_to.contains(t) {
            out.push(state_of(
                backend,
                repo,
                refs,
                local,
                t,
                (TargetRole::PullFrom, true),
                known,
            ));
        }
    }
    out
}

fn remote_names(backend: &dyn GitBackend, repo: &RepoRef) -> Result<Vec<String>> {
    Ok(backend.remotes(repo)?.into_iter().map(|r| r.name).collect())
}

/// The `RemoteGroupStatus` read.
pub fn status(backend: &dyn GitBackend, repo: &RepoRef, branch: &str) -> Result<RemoteGroupStatus> {
    let refs = backend.refs(repo)?;
    let local = refs
        .iter()
        .find_map(|r| match &r.kind {
            RefKind::LocalBranch { name } if name == branch => Some(r.target),
            _ => None,
        })
        .ok_or_else(|| Error::Backend(format!("no such branch: {branch}")))?;
    let entries = backend.config_section(repo, &section_of(branch))?;
    let known = remote_names(backend, repo)?;
    let mut remotes = parse_branch_remotes(&entries, branch);
    remotes.note_missing(&known);
    let targets = states(backend, repo, &refs, local, &remotes, &known);
    Ok(RemoteGroupStatus { remotes, targets })
}

type Memo = HashMap<PathBuf, (u64, Vec<(String, Vec<RemoteTargetState>)>)>;

fn memo() -> &'static Mutex<Memo> {
    static MEMO: OnceLock<Mutex<Memo>> = OnceLock::new();
    MEMO.get_or_init(|| Mutex::new(HashMap::new()))
}

/// What the decoration depends on: the repository config file (its size
/// and mtime) and every branch's tip.
fn fingerprint(repo: &RepoRef, refs: &[Ref]) -> u64 {
    let mut h = DefaultHasher::new();
    if let Some(dir) = gitpoulpi_core::paths::git_dir(&repo.root) {
        let config = gitpoulpi_core::paths::common_dir(&dir).join("config");
        if let Ok(meta) = std::fs::metadata(&config) {
            meta.len().hash(&mut h);
            meta.modified().ok().hash(&mut h);
        }
    }
    for r in refs {
        match &r.kind {
            RefKind::LocalBranch { name } => ("l", name, r.target).hash(&mut h),
            RefKind::RemoteBranch { remote, name } => ("r", remote, name, r.target).hash(&mut h),
            RefKind::Tag { .. } => {}
        }
    }
    h.finish()
}

/// Fills `Ref.multi_remote` on the local branches that have GitPoulpi keys.
/// Memoised on the config file and the branch tips, so an unchanged
/// repository costs no process.
pub fn decorate_refs(backend: &dyn GitBackend, repo: &RepoRef, refs: &mut [Ref]) {
    let key = fingerprint(repo, refs);
    let cached = memo()
        .lock()
        .ok()
        .and_then(|m| m.get(&repo.root).filter(|(k, _)| *k == key).cloned());
    let found = match cached {
        Some((_, found)) => found,
        None => {
            let found = compute(backend, repo, refs).unwrap_or_default();
            if let Ok(mut m) = memo().lock() {
                m.insert(repo.root.clone(), (key, found.clone()));
            }
            found
        }
    };
    for r in refs.iter_mut() {
        if let RefKind::LocalBranch { name } = &r.kind {
            if let Some((_, states)) = found.iter().find(|(b, _)| b == name) {
                r.multi_remote = Some(states.clone());
            }
        }
    }
}

fn compute(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    refs: &[Ref],
) -> Result<Vec<(String, Vec<RemoteTargetState>)>> {
    let branches: Vec<(&String, gitpoulpi_core::domain::Oid)> = refs
        .iter()
        .filter_map(|r| match &r.kind {
            RefKind::LocalBranch { name } => Some((name, r.target)),
            _ => None,
        })
        .collect();
    if branches.is_empty() {
        return Ok(Vec::new());
    }
    let sections: Vec<String> = branches.iter().map(|(n, _)| section_of(n)).collect();
    let read = backend.config_sections(repo, &sections)?;
    let mut known: Option<Vec<String>> = None;
    let mut out = Vec::new();
    for ((name, tip), (_, entries)) in branches.iter().zip(read) {
        let mut remotes = parse_branch_remotes(&entries, name);
        if !remotes.is_multi() {
            continue;
        }
        if known.is_none() {
            known = Some(remote_names(backend, repo)?);
        }
        let known = known.as_deref().unwrap_or_default();
        remotes.note_missing(known);
        out.push((
            (*name).clone(),
            states(backend, repo, refs, *tip, &remotes, known),
        ));
    }
    Ok(out)
}
