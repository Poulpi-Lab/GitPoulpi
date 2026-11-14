use std::collections::BTreeMap;

use gitpoulpi_core::domain::{
    parse_remote_tag_mirror_ref, split_remote_ref, Oid, Ref, RefKind, RefVisibility, RepoRef,
    TagSync, REMOTE_TAG_MIRROR_PREFIX,
};
use gitpoulpi_core::{Error, Result};

use crate::process::{read_with_retry, run, run_bytes_with_stdin};

pub fn refs(repo: &RepoRef) -> Result<Vec<Ref>> {
    // `symbolic-ref` fails (non-zero exit -> Err here) on a detached HEAD
    // or an unborn branch -- both cases correctly leave `head_branch` as
    // None, so no ref gets marked is_head, matching Git2Backend's
    // `head().ok().filter(is_branch)` behavior for the same states.
    let head_branch: Option<String> = run(&repo.root, &["symbolic-ref", "--short", "HEAD"])
        .ok()
        .map(|s| s.trim().to_string());

    // The remote list is read *before* the refs so `refs/remotes/<rest>`
    // can be split against the remotes that exist rather than on the first
    // `/` -- see `split_remote_ref`.
    let remote_names: Vec<String> = run(&repo.root, &["remote"])
        .map(|raw| {
            raw.lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    // No `%(*objectname)`: git 2.5x aborts (SIGABRT, a double free) on a ref
    // whose object is missing, which a repository being deleted or
    // half-written shows for a moment. Tags are peeled below instead.
    let raw = read_with_retry(|| {
        run(
            &repo.root,
            &[
                "for-each-ref",
                // `%(upstream:short)` is already spelled the way
                // `RefKind::RemoteBranch` spells a remote branch
                // (`origin/feature/login`), and empty for untracked refs.
                "--format=%(refname)\u{1}%(objectname)\u{1}%(upstream:short)",
                "refs/heads",
                "refs/remotes",
                "refs/tags",
                // GitPoulpi's own remote-tag mirror, walked in the same
                // `for-each-ref` rather than a second spawn. These rows never
                // become `Ref`s; they only fill in each tag's `tag_sync`.
                REMOTE_TAG_MIRROR_PREFIX,
            ],
        )
    })?;

    let rows: Vec<(&str, Oid, &str)> = raw
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, '\u{1}');
            let (refname, objectname) = (parts.next()?, parts.next()?);
            let upstream = parts.next().unwrap_or("").trim();
            Some((refname, Oid::from_hex(objectname.trim())?, upstream))
        })
        .collect();
    let peeled = peel_tags(repo, &rows)?;

    // tag name -> the commits the remotes are known to have it at.
    let mut mirror: BTreeMap<String, Vec<(String, Oid)>> = BTreeMap::new();
    let mut refs: Vec<Ref> = Vec::new();

    for (refname, listed, upstream) in rows {
        let target = peeled.get(&listed).copied().unwrap_or(listed);

        if let Some((remote, tag)) = parse_remote_tag_mirror_ref(refname, &remote_names) {
            mirror
                .entry(tag.to_string())
                .or_default()
                .push((remote, target));
            continue;
        }
        let Some(kind) = parse_ref_kind(refname, &remote_names) else {
            continue;
        };
        let is_head = matches!(&kind, RefKind::LocalBranch { name } if Some(name.as_str()) == head_branch.as_deref());
        // Only a local branch can track anything; git leaves the field
        // empty for the rest, but pinning it here means a future format
        // change cannot invent one.
        let upstream = match (&kind, upstream.is_empty()) {
            (RefKind::LocalBranch { .. }, false) => Some(upstream.to_string()),
            _ => None,
        };
        refs.push(Ref {
            kind,
            target,
            visibility: RefVisibility::Shown,
            is_head,
            upstream,
            tag_sync: None,
        });
    }

    apply_tag_sync(&mut refs, &mirror);

    Ok(refs)
}

/// The commit each annotated tag row (a tag or a mirror of one) points at,
/// keyed by the row's own object id. One `cat-file --batch-check` for all of
/// them, and only when there is a tag row. An object that is missing, or
/// that peels to something other than a commit, is left out: the row keeps
/// its own id, as `Git2Backend` does.
fn peel_tags(
    repo: &RepoRef,
    rows: &[(&str, Oid, &str)],
) -> Result<std::collections::HashMap<Oid, Oid>> {
    let mut ids: Vec<Oid> = rows
        .iter()
        .filter(|(name, _, _)| {
            name.starts_with("refs/tags/") || name.starts_with(REMOTE_TAG_MIRROR_PREFIX)
        })
        .map(|(_, id, _)| *id)
        .collect();
    ids.sort_by_key(|id| id.0);
    ids.dedup();
    peel_commits(&repo.root, &ids)
}

/// What `cat-file` says about one object: its own type (`missing` for an
/// object that is gone) and the commit it peels to (through any number of
/// tags), when it does.
pub(crate) struct Peel {
    pub(crate) kind: String,
    pub(crate) commit: Option<Oid>,
}

/// `ids` described by one `cat-file --batch-check` (`<id>` and `<id>^{}` per
/// id; no spawn for an empty list), keyed by the id asked for. Used instead
/// of `%(*objectname)` / `%(objecttype)` in a `for-each-ref` format: git 2.5x
/// aborts (or exits 128) on a ref to a missing object.
pub(crate) fn peel_info(
    root: &std::path::Path,
    ids: &[Oid],
) -> Result<std::collections::HashMap<Oid, Peel>> {
    let mut info = std::collections::HashMap::new();
    if ids.is_empty() {
        return Ok(info);
    }
    let mut stdin = String::new();
    for id in ids {
        stdin.push_str(&format!("{id}\n{id}^{{}}\n"));
    }
    let answer = read_with_retry(|| {
        run_bytes_with_stdin(
            root,
            &["cat-file", "--batch-check=%(objectname) %(objecttype)"],
            stdin.as_bytes(),
        )
    })?;
    // Two answer lines per id, in order. A missing object answers
    // `<input> missing`, so the type is whatever follows the last space.
    let text = String::from_utf8_lossy(&answer);
    let mut lines = text.lines();
    for id in ids {
        let (Some(own), Some(peeled)) = (lines.next(), lines.next()) else {
            break;
        };
        let kind = own.rsplit_once(' ').map_or("missing", |(_, kind)| kind);
        let commit = peeled.strip_suffix(" commit").and_then(Oid::from_hex);
        info.insert(
            *id,
            Peel {
                kind: kind.to_string(),
                commit,
            },
        );
    }
    Ok(info)
}

/// The commit each of `ids` peels to, keyed by the id asked for. An id whose
/// object is missing, or that peels to something other than a commit, is
/// left out.
pub(crate) fn peel_commits(
    root: &std::path::Path,
    ids: &[Oid],
) -> Result<std::collections::HashMap<Oid, Oid>> {
    Ok(peel_info(root, ids)?
        .into_iter()
        .filter_map(|(id, p)| Some((id, p.commit?)))
        .collect())
}

/// Which remotes hold `name`, split by whether they hold it **at this
/// commit**; `local` says whether the tag exists here at all.
///
/// A remote holding the tag at a *different* commit would only accept a
/// `--force` push, so reporting it as "pushed" would hide a divergence: it
/// goes to `differs_on` instead.
///
/// Shared verbatim with `Git2Backend`: the one place the rule lives.
pub fn tag_sync_for(
    name: &str,
    target: Oid,
    mirror: &BTreeMap<String, Vec<(String, Oid)>>,
) -> TagSync {
    let mut pushed_to = Vec::new();
    let mut differs_on = Vec::new();
    for (remote, oid) in mirror.get(name).map(Vec::as_slice).unwrap_or_default() {
        if *oid == target {
            pushed_to.push(remote.clone());
        } else {
            differs_on.push(remote.clone());
        }
    }
    pushed_to.sort();
    pushed_to.dedup();
    differs_on.sort();
    differs_on.dedup();
    TagSync {
        pushed_to,
        differs_on,
        local: true,
    }
}

/// Fills in every tag's `tag_sync` and appends one **remote-only** row per
/// mirrored tag name that has no `refs/tags/<name>` here.
///
/// A remote-only row is a `RefKind::Tag` whose `tag_sync.local` is false
/// and whose target is the commit of the first remote (in name order)
/// that has it; remotes that disagree with that one land in `differs_on`.
/// One function for both backends, so they cannot disagree about it.
pub fn apply_tag_sync(refs: &mut Vec<Ref>, mirror: &BTreeMap<String, Vec<(String, Oid)>>) {
    let mut local: std::collections::BTreeSet<String> = Default::default();
    for r in refs.iter_mut() {
        if let RefKind::Tag { name } = &r.kind {
            local.insert(name.clone());
            r.tag_sync = Some(tag_sync_for(name, r.target, mirror));
        }
    }
    for (name, entries) in mirror {
        if local.contains(name) {
            continue;
        }
        let Some(first) = entries.iter().min_by(|a, b| a.0.cmp(&b.0)) else {
            continue;
        };
        let mut sync = tag_sync_for(name, first.1, mirror);
        sync.local = false;
        refs.push(Ref {
            kind: RefKind::Tag { name: name.clone() },
            target: first.1,
            visibility: RefVisibility::Shown,
            is_head: false,
            upstream: None,
            tag_sync: Some(sync),
        });
    }
}

/// See `GitBackend::resolve`.
///
/// Two details are load-bearing. `^{commit}` makes git peel whatever the
/// revision names down to a commit, so an annotated tag yields the commit
/// it tags rather than the tag object's own oid. `--end-of-options` stops
/// a revision that happens to start with a dash (a branch literally named
/// `-x`, or a typo) from being parsed as a flag by this very command.
pub fn resolve(repo: &RepoRef, rev: &str) -> Result<Oid> {
    let spec = format!("{rev}^{{commit}}");
    let raw = run(
        &repo.root,
        &["rev-parse", "--verify", "--end-of-options", &spec],
    )?;
    Oid::from_hex(raw.trim())
        .ok_or_else(|| Error::Backend(format!("`{rev}` did not resolve to a commit")))
}

fn parse_ref_kind(refname: &str, remote_names: &[String]) -> Option<RefKind> {
    if let Some(name) = refname.strip_prefix("refs/heads/") {
        return Some(RefKind::LocalBranch {
            name: name.to_string(),
        });
    }
    if let Some(rest) = refname.strip_prefix("refs/remotes/") {
        let (remote, name) = split_remote_ref(rest, remote_names)?;
        return Some(RefKind::RemoteBranch {
            remote,
            name: name.to_string(),
        });
    }
    if let Some(name) = refname.strip_prefix("refs/tags/") {
        return Some(RefKind::Tag {
            name: name.to_string(),
        });
    }
    None
}

/// See `GitBackend::merge_base`. `git merge-base` exits 1 with no output
/// when the commits share no history.
pub fn merge_base(repo: &RepoRef, a: Oid, b: Oid) -> Result<Option<Oid>> {
    match run(&repo.root, &["merge-base", &a.to_hex(), &b.to_hex()]) {
        Ok(raw) => Ok(Oid::from_hex(raw.trim())),
        Err(_) => {
            // Distinguish "unrelated" from a real failure by asking again
            // whether both commits exist.
            run(
                &repo.root,
                &["cat-file", "-e", &format!("{}^{{commit}}", a.to_hex())],
            )?;
            run(
                &repo.root,
                &["cat-file", "-e", &format!("{}^{{commit}}", b.to_hex())],
            )?;
            Ok(None)
        }
    }
}

/// See `GitBackend::ahead_behind`.
pub fn cherry_counts(repo: &RepoRef, local: Oid, remote: Oid) -> Result<(u32, u32)> {
    let range = format!("{}...{}", local.to_hex(), remote.to_hex());
    let raw = run(
        &repo.root,
        &[
            "rev-list",
            "--cherry-pick",
            "--left-right",
            "--count",
            &range,
        ],
    )?;
    let mut counts = raw.split_whitespace().map(|n| n.parse::<u32>());
    match (counts.next(), counts.next()) {
        (Some(Ok(l)), Some(Ok(r))) => Ok((l, r)),
        _ => Err(Error::Backend(format!(
            "unexpected `rev-list --count` output: {raw:?}"
        ))),
    }
}

pub fn ahead_behind(repo: &RepoRef, tip: Oid, base: Oid) -> Result<(u32, u32)> {
    let range = format!("{}...{}", tip.to_hex(), base.to_hex());
    let raw = run(&repo.root, &["rev-list", "--left-right", "--count", &range])?;
    let mut counts = raw.split_whitespace().map(|n| n.parse::<u32>());
    match (counts.next(), counts.next()) {
        (Some(Ok(ahead)), Some(Ok(behind))) => Ok((ahead, behind)),
        _ => Err(Error::Backend(format!(
            "unexpected `rev-list --count` output: {raw:?}"
        ))),
    }
}
