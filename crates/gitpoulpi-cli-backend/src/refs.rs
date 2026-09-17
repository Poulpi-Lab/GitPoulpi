use std::collections::BTreeMap;

use gitpoulpi_core::domain::{
    parse_remote_tag_mirror_ref, split_remote_ref, Oid, Ref, RefKind, RefVisibility, RepoRef,
    TagSync, REMOTE_TAG_MIRROR_PREFIX,
};
use gitpoulpi_core::{Error, Result};

use crate::process::run;

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

    let raw = run(
        &repo.root,
        &[
            "for-each-ref",
            // `%(objectname)` on an annotated tag is the tag object's own
            // oid; `%(*objectname)` is the peeled commit oid, empty for
            // lightweight tags, so the parser falls back to `objectname`.
            //
            // `%(upstream:short)` is already spelled the way
            // `RefKind::RemoteBranch` spells a remote branch
            // (`origin/feature/login`), and empty for untracked refs.
            "--format=%(refname)\u{1}%(objectname)\u{1}%(*objectname)\u{1}%(upstream:short)",
            "refs/heads",
            "refs/remotes",
            "refs/tags",
            // GitPoulpi's own remote-tag mirror, walked in the same
            // `for-each-ref` rather than a second spawn. These rows never
            // become `Ref`s; they only fill in each tag's `tag_sync`.
            REMOTE_TAG_MIRROR_PREFIX,
        ],
    )?;

    // tag name -> the commits the remotes are known to have it at.
    let mut mirror: BTreeMap<String, Vec<(String, Oid)>> = BTreeMap::new();
    let mut refs: Vec<Ref> = Vec::new();

    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.splitn(4, '\u{1}');
        let (Some(refname), Some(objectname)) = (parts.next(), parts.next()) else {
            continue;
        };
        let peeled = parts.next().unwrap_or("").trim();
        let upstream = parts.next().unwrap_or("").trim();
        let target = if peeled.is_empty() {
            Oid::from_hex(objectname.trim())
        } else {
            Oid::from_hex(peeled)
        };
        let Some(target) = target else { continue };

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
