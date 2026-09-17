use gitpoulpi_core::domain::{
    decode_commit_text, parse_remote_tag_mirror_ref, Oid, Ref, RefKind, RefVisibility, Remote,
    RepoRef, Stash, StashId,
};
use gitpoulpi_core::Result;

use crate::repo::{map_err, open, to_domain_oid, to_git2_oid};

pub fn refs(repo: &RepoRef) -> Result<Vec<Ref>> {
    let git_repo = open(repo)?;
    // HEAD's shorthand name (e.g. "main") when it's a symbolic ref to a
    // local branch; None when detached or unborn, in which case no ref is
    // ever marked is_head -- the caller must not fall back to guessing.
    let head_branch: Option<String> = git_repo
        .head()
        .ok()
        .filter(|h| h.is_branch())
        .and_then(|h| h.shorthand().map(str::to_string));

    // Read before the refs so `refs/remotes/<rest>` is split against the
    // remotes that actually exist rather than on the first `/` -- see
    // `gitpoulpi_core::domain::split_remote_ref`.
    let remote_names: Vec<String> = git_repo
        .remotes()
        .map_err(map_err)?
        .iter()
        .flatten()
        .map(str::to_string)
        .collect();

    let mut out = Vec::new();
    let mut ref_names: std::collections::HashSet<String> = Default::default();
    // tag name -> the commits the remotes are known to have it at, read
    // from GitPoulpi's remote-tag mirror in this same walk. git keeps no
    // remote-tracking refs for tags, and `refs()` is on every render path,
    // so a local answer is the only acceptable kind.
    let mut mirror: std::collections::BTreeMap<String, Vec<(String, Oid)>> = Default::default();
    for reference in git_repo.references().map_err(map_err)? {
        let reference = reference.map_err(map_err)?;
        let Some(name) = reference.name() else {
            continue;
        };
        ref_names.insert(name.to_string());
        let Some(target) = reference.target() else {
            continue;
        };
        let oid = to_domain_oid(target)?;

        if let Some((remote, tag)) = parse_remote_tag_mirror_ref(name, &remote_names) {
            // Peeled, like the tag rows below: a mirror ref written by a
            // fetch points at the annotated tag *object*, one written by our
            // own push at the commit, and both must answer the same question.
            let commit_oid = reference.peel_to_commit().map(|c| c.id()).unwrap_or(target);
            mirror
                .entry(tag.to_string())
                .or_default()
                .push((remote, to_domain_oid(commit_oid)?));
            continue;
        }

        if let Some(branch) = name.strip_prefix("refs/heads/") {
            // Resolved after the walk, from one config snapshot (see
            // `resolve_upstream`): `Branch::upstream` re-reads the config
            // for every branch.
            let upstream = None;
            out.push(Ref {
                kind: RefKind::LocalBranch {
                    name: branch.to_string(),
                },
                target: oid,
                visibility: RefVisibility::Shown,
                is_head: head_branch.as_deref() == Some(branch),
                upstream,
                tag_sync: None,
            });
        } else if let Some(rest) = name.strip_prefix("refs/remotes/") {
            if let Some((remote, branch)) =
                gitpoulpi_core::domain::split_remote_ref(rest, &remote_names)
            {
                out.push(Ref {
                    kind: RefKind::RemoteBranch {
                        remote,
                        name: branch.to_string(),
                    },
                    target: oid,
                    visibility: RefVisibility::Shown,
                    is_head: false,
                    upstream: None,
                    tag_sync: None,
                });
            }
        } else if let Some(tag) = name.strip_prefix("refs/tags/") {
            // `reference.target()` on an annotated tag is the *tag object's*
            // oid; `peel_to_commit` dereferences it to the real commit (a
            // no-op for a lightweight tag).
            let commit_oid = reference.peel_to_commit().map(|c| c.id()).unwrap_or(target);
            out.push(Ref {
                kind: RefKind::Tag {
                    name: tag.to_string(),
                },
                target: to_domain_oid(commit_oid)?,
                visibility: RefVisibility::Shown,
                is_head: false,
                upstream: None,
                // Filled in below, once every mirror ref has been seen.
                tag_sync: None,
            });
        }
    }

    // Second pass: the mirror entry for a tag can appear after the tag
    // itself in libgit2's iteration order.
    let mut upstreams = UpstreamResolver::new(&git_repo, &ref_names);
    gitpoulpi_cli_backend::apply_tag_sync(&mut out, &mirror);
    for r in out.iter_mut() {
        if let RefKind::LocalBranch { name } = &r.kind {
            r.upstream = upstreams.resolve(name);
        }
    }
    Ok(out)
}

/// `Branch::upstream` for many branches off one config snapshot.
///
/// libgit2's `git_branch_upstream_name` reads `branch.<n>.remote` and
/// `branch.<n>.merge`, maps the merge ref through the remote's fetch
/// refspec (`.` means a local branch), and `Branch::upstream` then fails
/// if that ref does not exist. Here the config is read once, each remote
/// looked up once, and existence checked against the refs `refs()` already
/// walked. Every failure still means "no upstream to show".
struct UpstreamResolver<'a> {
    repo: &'a git2::Repository,
    config: Option<git2::Config>,
    ref_names: &'a std::collections::HashSet<String>,
    remotes: std::collections::HashMap<String, Option<git2::Remote<'a>>>,
}

impl<'a> UpstreamResolver<'a> {
    fn new(repo: &'a git2::Repository, ref_names: &'a std::collections::HashSet<String>) -> Self {
        let config = repo.config().and_then(|mut c| c.snapshot()).ok();
        Self {
            repo,
            config,
            ref_names,
            remotes: Default::default(),
        }
    }

    fn resolve(&mut self, branch: &str) -> Option<String> {
        let config = self.config.as_ref()?;
        let remote_name = config.get_string(&format!("branch.{branch}.remote")).ok()?;
        let merge = config.get_string(&format!("branch.{branch}.merge")).ok()?;
        let full = if remote_name == "." {
            merge
        } else {
            let repo = self.repo;
            let remote = self
                .remotes
                .entry(remote_name.clone())
                .or_insert_with(|| repo.find_remote(&remote_name).ok())
                .as_ref()?;
            let mut mapped = None;
            for spec in remote.refspecs() {
                if spec.direction() == git2::Direction::Fetch && spec.src_matches(&merge) {
                    mapped = spec
                        .transform(&merge)
                        .ok()
                        .and_then(|b| b.as_str().map(str::to_string));
                    break;
                }
            }
            mapped?
        };
        if !self.ref_names.contains(&full) {
            return None;
        }
        full.strip_prefix("refs/heads/")
            .or_else(|| full.strip_prefix("refs/remotes/"))
            .map(str::to_string)
    }
}

pub fn remotes(repo: &RepoRef) -> Result<Vec<Remote>> {
    let git_repo = open(repo)?;
    let names = git_repo.remotes().map_err(map_err)?;
    let mut out = Vec::new();
    for name in names.iter().flatten() {
        let remote = git_repo.find_remote(name).map_err(map_err)?;
        let fetch_url = remote.url().unwrap_or_default().to_string();
        let push_url = Some(remote.pushurl().unwrap_or(&fetch_url).to_string());
        out.push(Remote {
            name: name.to_string(),
            fetch_url,
            push_url,
        });
    }
    Ok(out)
}

/// `git stash` messages default to "WIP on <branch>: ..." or "On
/// <branch>: ..." - mirrors `gitpoulpi_cli_backend::stashes`'s
/// best-effort parsing so both backends agree.
fn branch_from_message(message: &str) -> String {
    for prefix in ["WIP on ", "On "] {
        if let Some(rest) = message.strip_prefix(prefix) {
            if let Some((branch, _)) = rest.split_once(':') {
                return branch.to_string();
            }
        }
    }
    String::new()
}

pub fn stashes(repo: &RepoRef) -> Result<Vec<Stash>> {
    let git_repo = open(repo)?;
    let reflog = match git_repo.reflog("refs/stash") {
        Ok(log) => log,
        Err(_) => return Ok(Vec::new()),
    };

    let mut out = Vec::new();
    for (i, entry) in reflog.iter().enumerate() {
        let message =
            decode_commit_text(entry.message_bytes().unwrap_or_default(), None).into_owned();
        // The stash commit is the reflog entry's new value; its first
        // parent is the commit the stash was made on.
        let commit_id = entry.id_new();
        let base = git_repo
            .find_commit(commit_id)
            .ok()
            .and_then(|c| c.parent_id(0).ok())
            .and_then(|id| to_domain_oid(id).ok());
        out.push(Stash {
            id: StashId(i as u32),
            branch_at_creation: branch_from_message(&message),
            message,
            timestamp: entry.committer().when().seconds(),
            commit: to_domain_oid(commit_id).ok(),
            base,
        });
    }
    Ok(out)
}

/// See `GitBackend::resolve`. `revparse_single` accepts the same
/// revision grammar as `git rev-parse`, and `peel_to_commit` does what
/// the CLI backend's `^{commit}` suffix does: an annotated tag resolves
/// to the commit it tags, not to the tag object.
pub fn resolve(repo: &RepoRef, rev: &str) -> Result<Oid> {
    let git_repo = open(repo)?;
    let object = git_repo.revparse_single(rev).map_err(map_err)?;
    let commit = object.peel_to_commit().map_err(map_err)?;
    to_domain_oid(commit.id())
}

/// See `GitBackend::merge_base`. libgit2 reports "no common ancestor" as
/// `NotFound`, which is an answer here, not a failure.
pub fn merge_base(repo: &RepoRef, a: Oid, b: Oid) -> Result<Option<Oid>> {
    let git_repo = open(repo)?;
    let (a, b) = (to_git2_oid(a)?, to_git2_oid(b)?);
    match git_repo.merge_base(a, b) {
        Ok(found) => to_domain_oid(found).map(Some),
        Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
        Err(e) => Err(map_err(e)),
    }
}

/// See `GitBackend::ahead_behind`.
pub fn ahead_behind(repo: &RepoRef, tip: Oid, base: Oid) -> Result<(u32, u32)> {
    let git_repo = open(repo)?;
    let (a, b) = (to_git2_oid(tip)?, to_git2_oid(base)?);
    let (ahead, behind) = git_repo.graph_ahead_behind(a, b).map_err(map_err)?;
    Ok((ahead as u32, behind as u32))
}
