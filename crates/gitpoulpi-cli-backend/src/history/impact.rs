//! What a rewrite means for what is already out there: which commits are on
//! a remote, which remote and tip a ref has, which local branches keep the
//! old history, and which stashes sit on rewritten commits.

use std::collections::HashSet;
use std::path::Path;

use gitpoulpi_core::domain::rewrite::PushState;
use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::Result;

use super::objects::Commit;
use super::refs::{RefLine, Refs};
use super::walk::{self, read_git, Head, Reader};
use crate::process::{failed_with, spawn_failed};

const REMOTE_TAGS: &str = "refs/gitpoulpi/remote-tags/";

/// The commits no remote has, from one `rev-list`; without any remote-tracking
/// ref nothing is pushed and nothing is asked of git.
pub(crate) struct Pushed {
    has_remotes: bool,
    local_only: HashSet<Oid>,
}

impl Pushed {
    /// Nothing is on a remote: the answer for a run that does not touch this
    /// repository's own refs.
    pub(crate) fn none() -> Self {
        Self {
            has_remotes: false,
            local_only: HashSet::new(),
        }
    }

    pub(crate) fn is_pushed(&self, commit: &Oid) -> bool {
        self.has_remotes && !self.local_only.contains(commit)
    }
}

/// `rev-list <every local tip> --not --remotes <tag mirror>`: the commits
/// that exist only here.
pub(crate) fn classify(root: &Path, refs: &Refs, head: &Head) -> Result<Pushed> {
    let has_remotes = !refs.remotes.is_empty() || !refs.mirrors.is_empty();
    if !has_remotes {
        return Ok(Pushed {
            has_remotes,
            local_only: HashSet::new(),
        });
    }
    let detached = match head {
        Head::Detached(oid) => Some(oid.to_hex()),
        Head::Branch(_) => None,
    };
    let mut args = vec!["rev-list", "--branches", "--tags"];
    args.extend(detached.as_deref());
    let glob = format!("--glob={REMOTE_TAGS}*");
    args.extend(["--not", "--remotes", glob.as_str()]);
    let out = read_git(root, &args).output().map_err(spawn_failed)?;
    if !out.status.success() {
        return Err(failed_with(
            &args,
            out.status.code(),
            &String::from_utf8_lossy(&out.stderr),
        ));
    }
    Ok(Pushed {
        has_remotes,
        local_only: walk::parse_oids(&out.stdout)?.into_iter().collect(),
    })
}

/// `refs/remotes/<remote>/<rest>` or `refs/gitpoulpi/remote-tags/<remote>/<rest>`.
fn remote_of(name: &str, prefix: &str) -> Option<String> {
    name.strip_prefix(prefix)?
        .split('/')
        .next()
        .map(str::to_string)
}

/// The remote a branch lives on and where that remote's copy was last seen:
/// its upstream, else a tracking ref of the same name, else any remote.
pub(crate) fn remote_of_branch(refs: &Refs, branch: &str) -> Option<(String, Oid)> {
    let short = branch.strip_prefix("refs/heads/").unwrap_or(branch);
    let listed = refs
        .heads
        .iter()
        .find(|r| r.name == branch)
        .map(|r| r.upstream.as_str())
        .unwrap_or_default();
    let pick = |r: &RefLine| remote_of(&r.name, "refs/remotes/").map(|remote| (remote, r.oid));
    let tracking = |r: &&RefLine| !r.name.ends_with("/HEAD");
    refs.remotes
        .iter()
        .find(|r| !listed.is_empty() && r.name == listed)
        .or_else(|| {
            refs.remotes
                .iter()
                .filter(tracking)
                .find(|r| r.name.ends_with(&format!("/{short}")))
        })
        .or_else(|| refs.remotes.iter().find(tracking))
        .and_then(pick)
}

/// How a branch stands: `rewritten_on_remote` is the number of rewritten
/// commits of its history that a remote has.
pub(crate) fn branch_state(refs: &Refs, branch: &str, rewritten_on_remote: u32) -> PushState {
    if rewritten_on_remote == 0 {
        return PushState::LocalOnly;
    }
    match remote_of_branch(refs, branch) {
        Some((remote, remote_oid)) => PushState::OnRemote {
            remote,
            remote_oid,
            rewritten_on_remote,
        },
        None => PushState::LocalOnly,
    }
}

/// A tag is pushed when GitPoulpi's mirror of the remote's tags has it.
pub(crate) fn tag_state(refs: &Refs, tag: &str) -> PushState {
    let short = tag.strip_prefix("refs/tags/").unwrap_or(tag);
    match refs
        .mirrors
        .iter()
        .find(|m| m.name.ends_with(&format!("/{short}")))
    {
        Some(m) => PushState::OnRemote {
            remote: remote_of(&m.name, REMOTE_TAGS).unwrap_or_default(),
            remote_oid: m.oid,
            rewritten_on_remote: 1,
        },
        None => PushState::LocalOnly,
    }
}

/// The local branches in `candidates` that contain any of `roots` (rewritten
/// commits none of whose parents was rewritten: a branch that contains any
/// rewritten commit contains one of them). Chunked so the command line stays
/// short.
pub(crate) fn branches_containing(
    root: &Path,
    roots: &[Oid],
    candidates: &HashSet<String>,
) -> Result<Vec<String>> {
    let mut found = HashSet::new();
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    for chunk in roots.chunks(200) {
        let contains: Vec<String> = chunk
            .iter()
            .map(|r| format!("--contains={}", r.to_hex()))
            .collect();
        let mut args = vec!["for-each-ref", "--format=%(refname)"];
        args.extend(contains.iter().map(String::as_str));
        args.push("refs/heads");
        let out = read_git(root, &args).output().map_err(spawn_failed)?;
        if !out.status.success() {
            return Err(failed_with(
                &args,
                out.status.code(),
                &String::from_utf8_lossy(&out.stderr),
            ));
        }
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if candidates.contains(line) {
                found.insert(line.to_string());
            }
        }
    }
    let mut found: Vec<String> = found.into_iter().collect();
    found.sort();
    Ok(found)
}

/// The stashes whose base commit was rewritten: they still apply, but keep
/// the old history alive.
pub(crate) fn stashes_on(common_dir: &Path, reader: &mut Reader, rewritten: &HashSet<Oid>) -> u32 {
    let Ok(log) = std::fs::read_to_string(common_dir.join("logs").join("refs").join("stash"))
    else {
        return 0;
    };
    let mut count = 0;
    for line in log.lines() {
        let Some(stash) = line.split(' ').nth(1).and_then(Oid::from_hex) else {
            continue;
        };
        let Ok((kind, body)) = reader.get(stash) else {
            continue;
        };
        if kind != "commit" {
            continue;
        }
        let base = Commit::parse(body)
            .ok()
            .and_then(|c| c.parents().ok())
            .and_then(|p| p.first().copied());
        count += u32::from(base.is_some_and(|b| rewritten.contains(&b)));
    }
    count
}
