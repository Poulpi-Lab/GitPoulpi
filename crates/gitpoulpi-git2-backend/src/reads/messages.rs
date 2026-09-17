//! `commit_messages`: the workspace search's read of every commit's text.

use gitpoulpi_core::domain::{
    decode_commit_text, split_message, CommitMessage, CommitMessages, CommitMessagesQuery, Oid,
    RepoRef,
};
use gitpoulpi_core::Result;

use crate::repo::{map_err, open};

/// Oids by their bytes: this runs once per commit of the repository.
fn domain(oid: git2::Oid) -> Oid {
    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(oid.as_bytes());
    Oid(bytes)
}

/// The commit every branch, remote-tracking branch, tag (peeled) and HEAD
/// points to. Not `refs/stash`, not GitPoulpi's own refs: the same set the
/// system backend names with `for-each-ref`.
fn tips(repo: &git2::Repository) -> Result<Vec<git2::Oid>> {
    let mut out: Vec<git2::Oid> = Vec::new();
    let refs = repo.references().map_err(map_err)?;
    for reference in refs.flatten() {
        let Some(name) = reference.name() else {
            continue;
        };
        if !(name.starts_with("refs/heads/")
            || name.starts_with("refs/remotes/")
            || name.starts_with("refs/tags/"))
        {
            continue;
        }
        if let Ok(commit) = reference.peel_to_commit() {
            out.push(commit.id());
        }
    }
    if let Ok(head) = repo.head() {
        if let Ok(commit) = head.peel_to_commit() {
            out.push(commit.id());
        }
    }
    out.sort_unstable_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    out.dedup();
    Ok(out)
}

pub fn commit_messages(repo: &RepoRef, query: &CommitMessagesQuery) -> Result<CommitMessages> {
    let git_repo = open(repo)?;
    let tips = tips(&git_repo)?;
    let known: Vec<git2::Oid> = query
        .known_tips
        .iter()
        .filter_map(|o| git2::Oid::from_bytes(&o.0).ok())
        .collect();

    // A known tip no current tip reaches (or one the object database no
    // longer has) means history was rewritten: start over.
    let replace = !known.is_empty() && {
        let unreachable = (|| -> std::result::Result<bool, git2::Error> {
            let mut walk = git_repo.revwalk()?;
            for k in &known {
                if git_repo.find_commit(*k).is_err() {
                    return Ok(true);
                }
                walk.push(*k)?;
            }
            for t in &tips {
                walk.hide(*t)?;
            }
            Ok(walk.next().is_some())
        })();
        unreachable.unwrap_or(true)
    };

    let mut out = CommitMessages {
        tips: tips.iter().map(|o| domain(*o)).collect(),
        replace,
        ..CommitMessages::default()
    };
    if tips.is_empty() {
        return Ok(out);
    }
    let mut walk = git_repo.revwalk().map_err(map_err)?;
    for t in &tips {
        walk.push(*t).map_err(map_err)?;
    }
    if !replace {
        for k in &known {
            walk.hide(*k).map_err(map_err)?;
        }
    }
    walk.set_sorting(git2::Sort::TIME).map_err(map_err)?;
    for oid in walk {
        if query.limit > 0 && out.commits.len() >= query.limit {
            out.truncated = true;
            break;
        }
        let commit = git_repo
            .find_commit(oid.map_err(map_err)?)
            .map_err(map_err)?;
        let header = commit.message_encoding();
        let message = decode_commit_text(commit.message_bytes(), header);
        let (subject, body) = split_message(&message);
        out.commits.push(CommitMessage {
            id: domain(commit.id()),
            author: decode_commit_text(commit.author().name_bytes(), header).into_owned(),
            time: commit.time().seconds(),
            subject,
            body,
        });
    }
    Ok(out)
}
