//! Tag operations, and the remote-tag mirror that makes "is this tag
//! pushed?" a *local* question.
//!
//! git keeps no remote-tracking refs for tags -- a fetch copies the
//! remote's tags into `refs/tags/*`, the same namespace the local ones
//! live in -- so the only truthful source is `git ls-remote`, a network
//! round trip. A round trip cannot sit on a render path, and the TUI's
//! standing rule is that the input loop never waits on a git subprocess.
//!
//! So this module maintains one:
//! `refs/gitpoulpi/remote-tags/<remote>/<tag>` (see
//! [`gitpoulpi_core::domain::REMOTE_TAG_MIRROR_PREFIX`]). It is written
//! here and by `network::fetch`, and read by `refs::refs`, which
//! is then free. `Git2Backend` reads the same refs natively and delegates
//! these writes here, so the two backends cannot drift.

use std::collections::BTreeSet;

use gitpoulpi_core::domain::{
    remote_tag_mirror_ref, Oid, PushOutcome, RepoRef, TagSelection, REMOTE_TAG_MIRROR_PREFIX,
};
use gitpoulpi_core::{Error, Result};

use crate::process::{run, run_async, run_bytes, run_with_stdin, run_with_stdin_bytes};
use crate::refs::refs;

/// The refspec that mirrors `remote`'s tags into our namespace. Added to
/// the user's own fetch by `network::fetch` (one invocation, via
/// `-c remote.<name>.fetch=`, which appends to the configured refspecs).
///
/// `+` because a tag that moved on the remote must overwrite ours rather
/// than be rejected as a non-fast-forward - this is a cache of what the
/// remote says, not a history.
pub(crate) fn mirror_refspec(remote: &str) -> String {
    format!("+refs/tags/*:{REMOTE_TAG_MIRROR_PREFIX}{remote}/*")
}

/// Replaces tag `name` with an annotated tag carrying `message`, at the
/// commit it already points at.
///
/// `-f` because the tag exists; `<name>^{commit}` as the target because
/// the tag must not move - naming the tag itself is how the commit is
/// looked up, so there is no second opinion about where it was. The
/// existence check is explicit and first, because `git tag -f -a` on an
/// unknown name would happily *create* the tag, and "Annotate v9"
/// inventing a tag is worse than refusing.
pub fn tag_annotate(repo: &RepoRef, name: &str, message: &str) -> Result<()> {
    let tag_ref = format!("refs/tags/{name}");
    run(&repo.root, &["rev-parse", "--verify", "--quiet", &tag_ref])
        .map_err(|_| Error::Backend(format!("no tag named `{name}`")))?;
    let at = format!("{tag_ref}^{{commit}}");
    run(&repo.root, &["tag", "-f", "-a", name, "-m", message, &at])?;
    Ok(())
}

/// Moves tag `name` to the commit `to` (`GitBackend::tag_move`).
///
/// An annotated tag is re-written with the same bytes but a new `object`
/// line (`retarget_tag_object`), hashed in and swapped with a compare-and-swap
/// `update-ref`, so the backends write the same object.
pub fn tag_move(repo: &RepoRef, name: &str, to: Oid, force: bool) -> Result<()> {
    let full = format!("refs/tags/{name}");
    let no_tag = |_| Error::Backend(format!("no tag named `{name}`"));
    let old = run(&repo.root, &["rev-parse", "--verify", "--quiet", &full])
        .map_err(no_tag)?
        .trim()
        .to_string();
    let old_commit = run(
        &repo.root,
        &["rev-parse", "--verify", &format!("{full}^{{commit}}")],
    )?
    .trim()
    .to_string();
    let to_hex = to.to_hex();
    let target = format!("{to_hex}^{{commit}}");
    run(&repo.root, &["rev-parse", "--verify", "--quiet", &target])
        .map_err(|_| Error::Backend(format!("{to_hex} is not a commit")))?;
    if old_commit == to_hex {
        return Ok(());
    }
    if !force && !crate::branch::is_ancestor(repo, &old_commit, &to_hex)? {
        return Err(Error::NotFastForward);
    }
    let new = if run(&repo.root, &["cat-file", "-t", &full])?.trim() == "commit" {
        to_hex.clone()
    } else {
        let raw = run_bytes(&repo.root, &["cat-file", "tag", &full])?;
        let moved = gitpoulpi_core::domain::retarget_tag_object(&raw, &to_hex)?;
        run_with_stdin_bytes(
            &repo.root,
            &["hash-object", "-t", "tag", "-w", "--stdin"],
            &moved,
        )?
        .trim()
        .to_string()
    };
    let message = format!("tag: move {name} to {}", &to_hex[..7]);
    run(
        &repo.root,
        &["update-ref", "-m", &message, &full, &new, &old],
    )?;
    Ok(())
}

/// Merges `branch` into the commit of tag `name` with `merge-tree`, commits
/// the result detached with `commit-tree` and moves the tag
/// (`GitBackend::tag_merge_branch`).
pub fn tag_merge_branch(
    repo: &RepoRef,
    name: &str,
    branch: &str,
    message: Option<&str>,
) -> Result<Oid> {
    let tag_commit = run(
        &repo.root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("refs/tags/{name}^{{commit}}"),
        ],
    )
    .map_err(|_| Error::Backend(format!("no tag named `{name}`")))?
    .trim()
    .to_string();
    let tip = run(
        &repo.root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{branch}^{{commit}}"),
        ],
    )
    .map_err(|_| Error::Backend(format!("no revision named `{branch}`")))?
    .trim()
    .to_string();
    if crate::branch::is_ancestor(repo, &tip, &tag_commit)? {
        return Err(Error::Backend(format!(
            "{name} already contains {branch}: nothing to merge"
        )));
    }
    if !crate::process::git_at_least(2, 38) {
        return Err(Error::Unsupported {
            operation: "tag_merge_branch (needs git 2.38 or later)",
        });
    }
    let args = [
        "merge-tree",
        "--write-tree",
        tag_commit.as_str(),
        tip.as_str(),
    ];
    let out = crate::process::git(&repo.root, &args)
        .output()
        .map_err(crate::process::spawn_failed)?;
    let tree = match out.status.code() {
        Some(0) => String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string(),
        Some(1) => {
            return Err(Error::Conflict {
                operation: "merge into tag",
            })
        }
        _ => {
            return Err(Error::Backend(format!(
                "could not merge {branch} into {name}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )))
        }
    };
    let message = message
        .map(str::to_string)
        .unwrap_or_else(|| format!("Merge branch '{branch}' into tag '{name}'"));
    let merged = run_with_stdin(
        &repo.root,
        &[
            "commit-tree",
            "--no-gpg-sign",
            &tree,
            "-p",
            &tag_commit,
            "-p",
            &tip,
        ],
        &message,
    )?
    .trim()
    .to_string();
    let merged =
        Oid::from_hex(&merged).ok_or_else(|| Error::Backend(format!("odd commit id {merged}")))?;
    tag_move(repo, name, merged, false)?;
    Ok(merged)
}

/// Sends `selection`'s tags to `remote` in a single `git push`, then
/// records them in the mirror.
///
/// One push rather than one per tag: a credential prompt, a pre-push
/// hook and a connection are each paid once, and the caller gets one
/// result to report instead of N partial ones.
pub async fn tag_push(
    repo: &RepoRef,
    remote: &str,
    selection: &TagSelection,
) -> Result<PushOutcome> {
    let names = resolve(repo, selection)?;
    if names.is_empty() {
        // Nothing to do is a success. "Push all unpushed tags" pressed
        // twice must not report an error the second time.
        return Ok(PushOutcome::default());
    }

    let mut args = vec!["push".to_string(), remote.to_string()];
    args.extend(names.iter().map(|n| format!("refs/tags/{n}")));
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    run_async(&repo.root, &args_ref).await?;

    record_pushed(repo, remote, &names)?;
    Ok(PushOutcome { pushed_refs: names })
}

/// `git push --delete`, then drops the tag's mirror ref so the badge
/// comes back without waiting for a fetch.
pub async fn tag_delete_remote(repo: &RepoRef, remote: &str, name: &str) -> Result<()> {
    let target = format!("refs/tags/{name}");
    run_async(&repo.root, &["push", remote, "--delete", &target]).await?;
    forget_pushed(repo, remote, &[name.to_string()])?;
    Ok(())
}

/// Brings tag `name` from `remote` into `refs/tags/`, the "Fetch this tag"
/// of a remote-only tag.
///
/// `--no-tags` so only the named tag moves, and **no `+`** on the refspec:
/// a local tag of that name at another commit makes git refuse ("would
/// clobber the existing tag") instead of silently losing it, which is what
/// a tag that `differs` needs. The mirror already holds the remote's value
/// (that is how the tag was known), so it needs no write.
pub async fn tag_fetch(repo: &RepoRef, remote: &str, name: &str) -> Result<()> {
    if remote.is_empty() || remote.starts_with('-') || name.starts_with('-') {
        return Err(Error::Backend(format!(
            "`{remote}` `{name}` is not a remote and tag name"
        )));
    }
    let spec = format!("refs/tags/{name}:refs/tags/{name}");
    run_async(
        &repo.root,
        &["fetch", "--quiet", "--no-tags", remote, &spec],
    )
    .await?;
    Ok(())
}

/// Deletes the local tags that a pruning fetch just saw the remote lose.
///
/// `pruned` is `(mirror ref, old oid)` of each pruned mirror ref. A local
/// tag goes only when **all** of these hold, so that "the remote deleted
/// it" is the only reason it disappears: no remote has the name any more
/// (checked on the post-prune mirror), and it still points at the commit
/// the remote's tag pointed at (a tag re-made locally since is yours).
/// A tag that was never on a remote is never a candidate.
pub(crate) fn delete_local_tags_lost_on_remote(
    repo: &RepoRef,
    pruned: &[(String, String)],
) -> Result<Vec<String>> {
    if pruned.is_empty() {
        return Ok(Vec::new());
    }
    let remote_names: Vec<String> = crate::remotes::remotes(repo)?
        .into_iter()
        .map(|r| r.name)
        .collect();
    let current = refs(repo)?;
    let mut gone = Vec::new();
    for (mirror_ref, old) in pruned {
        let Some((_, tag)) =
            gitpoulpi_core::domain::parse_remote_tag_mirror_ref(mirror_ref, &remote_names)
        else {
            continue;
        };
        let Some(local) = current
            .iter()
            .find(|r| r.tag_name() == Some(tag) && !r.is_remote_only_tag())
        else {
            continue;
        };
        let still_elsewhere = local
            .tag_sync
            .as_ref()
            .is_some_and(|sync| !sync.on_remotes().is_empty());
        if still_elsewhere {
            continue;
        }
        let peeled = run(
            &repo.root,
            &[
                "rev-parse",
                "--verify",
                "--quiet",
                &format!("{old}^{{commit}}"),
            ],
        );
        if peeled.ok().and_then(|o| Oid::from_hex(o.trim())) != Some(local.target) {
            continue;
        }
        if !gone.iter().any(|g| g == tag) {
            gone.push(tag.to_string());
        }
    }
    if !gone.is_empty() {
        let script: String = gone
            .iter()
            .map(|n| {
                format!(
                    "delete refs/tags/{n}
"
                )
            })
            .collect();
        run_with_stdin(&repo.root, &["update-ref", "--stdin"], &script)?;
    }
    Ok(gone)
}

/// Turns a [`TagSelection`] into the tag names to push.
///
/// An unknown name is an error rather than a silent skip: a frontend
/// that asked to push `v9` and got back "pushed nothing, successfully"
/// has no way to tell that from "already up to date".
fn resolve(repo: &RepoRef, selection: &TagSelection) -> Result<Vec<String>> {
    let all = refs(repo)?;
    match selection {
        TagSelection::Named(names) => {
            let known: BTreeSet<&str> = all
                .iter()
                .filter(|r| !r.is_remote_only_tag())
                .filter_map(|r| r.tag_name())
                .collect();
            for name in names {
                if !known.contains(name.as_str()) {
                    return Err(Error::Backend(format!("no tag named `{name}`")));
                }
            }
            Ok(names.clone())
        }
        // Resolved here, at push time, rather than in the frontend:
        // a tag created since the last refresh is included instead of
        // silently left behind.
        TagSelection::AllUnpushed => Ok(all
            .iter()
            .filter(|r| r.is_unpushed_tag())
            .filter_map(|r| r.tag_name().map(str::to_string))
            .collect()),
    }
}

/// Writes one mirror ref per pushed tag, in a single `git update-ref
/// --stdin` transaction.
///
/// The value stored is the tag's **peeled commit**, which is what
/// `refs::refs` compares against - an annotated tag's own object id
/// would peel to the same commit when the mirror is later overwritten by
/// a fetch, so both spellings answer the same question, and storing the
/// peeled one means there is exactly one thing to compare.
fn record_pushed(repo: &RepoRef, remote: &str, names: &[String]) -> Result<()> {
    let by_name: Vec<(String, Oid)> = refs(repo)?
        .into_iter()
        .filter(|r| !r.is_remote_only_tag())
        .filter_map(|r| r.tag_name().map(|n| (n.to_string(), r.target)))
        .filter(|(n, _)| names.contains(n))
        .collect();
    let mut script = String::new();
    for (name, target) in by_name {
        script.push_str(&format!(
            "update {} {}\n",
            remote_tag_mirror_ref(remote, &name),
            target.to_hex()
        ));
    }
    if script.is_empty() {
        return Ok(());
    }
    run_with_stdin(&repo.root, &["update-ref", "--stdin"], &script)?;
    Ok(())
}

/// Drops mirror refs, in one transaction.
///
/// A failure is swallowed on purpose: the remote deletion this follows
/// has already happened, the mirror is a cache, and the next fetch
/// prunes whatever is left. Turning "the badge is one refresh stale"
/// into "the operation failed" would be a lie about the remote.
fn forget_pushed(repo: &RepoRef, remote: &str, names: &[String]) -> Result<()> {
    let mut script = String::new();
    for name in names {
        script.push_str(&format!("delete {}\n", remote_tag_mirror_ref(remote, name)));
    }
    if script.is_empty() {
        return Ok(());
    }
    let _ = run_with_stdin(&repo.root, &["update-ref", "--stdin"], &script);
    Ok(())
}
