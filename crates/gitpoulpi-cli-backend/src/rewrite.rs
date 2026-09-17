//! What reword and split share: reading commits, re-creating them with
//! `commit-tree` (author kept, committer now), replaying the descendants of
//! a rewritten commit and moving HEAD in one `update-ref` transaction.
//!
//! No rebase: every tree is reused as is, so nothing can conflict and the
//! index and working tree are never touched.

use std::collections::HashMap;
use std::path::Path;

use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::{Error, Result};

use crate::process::{run, run_with_env_and_stdin, run_with_stdin};

/// The files and directories git leaves in the git dir while an
/// operation is under way - what libgit2's `Repository::state` looks
/// for, so both backends refuse in the same situations.
pub(crate) const IN_PROGRESS_MARKERS: &[&str] = &[
    "MERGE_HEAD",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
    "BISECT_LOG",
    "rebase-merge",
    "rebase-apply",
];

/// The absolute git dir and HEAD's oid, from one `rev-parse`.
pub(crate) fn probe(root: &Path) -> Result<(String, Oid)> {
    let probe = run(root, &["rev-parse", "--absolute-git-dir", "HEAD"])?;
    let mut lines = probe.lines();
    let git_dir = lines
        .next()
        .ok_or_else(|| Error::Backend("rev-parse gave no git dir".into()))?
        .to_string();
    let head = lines
        .next()
        .and_then(|l| Oid::from_hex(l.trim()))
        .ok_or_else(|| Error::Backend("HEAD was not a valid oid".into()))?;
    Ok((git_dir, head))
}

/// Whether a merge, rebase, cherry-pick, revert or bisect is under way.
pub(crate) fn operation_in_progress(git_dir: &str) -> bool {
    IN_PROGRESS_MARKERS
        .iter()
        .any(|marker| Path::new(git_dir).join(marker).exists())
}

/// The commits between `target` (excluded) and `head`, parents first.
/// `Ok(None)` when `target` is not an ancestor of `head`; empty when it
/// is `head`.
pub(crate) fn descendants(root: &Path, target: Oid, head: Oid) -> Result<Option<Vec<String>>> {
    if target == head {
        return Ok(Some(Vec::new()));
    }
    // Descendants of the target that are ancestors of HEAD, parents
    // first. Empty exactly when the target is not an ancestor of HEAD.
    let range = format!("{}..{}", target.to_hex(), head.to_hex());
    let listed = run(
        root,
        &[
            "rev-list",
            "--ancestry-path",
            "--topo-order",
            "--reverse",
            &range,
        ],
    )?;
    let listed: Vec<String> = listed.lines().map(str::to_string).collect();
    Ok(if listed.is_empty() {
        None
    } else {
        Some(listed)
    })
}

/// What a commit is made of, as far as re-creating it goes.
pub(crate) struct CommitObject {
    pub(crate) tree: String,
    pub(crate) parents: Vec<String>,
    pub(crate) author_name: String,
    pub(crate) author_email: String,
    /// `<unix-seconds> <+hhmm>`, as stored.
    pub(crate) author_date: String,
    /// Verbatim, trailing newline included.
    pub(crate) message: String,
}

impl CommitObject {
    /// This commit with the given author parts replaced.
    pub(crate) fn clone_with_author(
        &self,
        name: Option<&str>,
        email: Option<&str>,
        date: Option<&str>,
    ) -> CommitObject {
        CommitObject {
            tree: self.tree.clone(),
            parents: self.parents.clone(),
            author_name: name.unwrap_or(&self.author_name).to_string(),
            author_email: email.unwrap_or(&self.author_email).to_string(),
            author_date: date.unwrap_or(&self.author_date).to_string(),
            message: self.message.clone(),
        }
    }
}

/// The commits `oids`, in that order, from one `git cat-file --batch`.
pub(crate) fn read_commits(root: &Path, oids: &[String]) -> Result<Vec<CommitObject>> {
    let input: String = oids.iter().map(|o| format!("{o}\n")).collect();
    let raw = run_with_stdin(root, &["cat-file", "--batch"], &input)?;
    let mut rest = raw.as_str();
    let mut out = Vec::with_capacity(oids.len());
    for oid in oids {
        let (header, after) = rest
            .split_once('\n')
            .ok_or_else(|| Error::Backend(format!("cat-file ended before {oid}")))?;
        let mut fields = header.split(' ');
        let (kind, size) = (fields.nth(1), fields.next());
        if kind != Some("commit") {
            return Err(Error::Backend(format!("{oid} is not a commit: {header}")));
        }
        let size: usize = size
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| Error::Backend(format!("bad cat-file header: {header}")))?;
        let body = after
            .get(..size)
            .ok_or_else(|| Error::Backend(format!("cat-file cut {oid} short")))?;
        out.push(parse_commit(body)?);
        // Each object is followed by a newline of cat-file's own.
        rest = after.get(size + 1..).unwrap_or_default();
    }
    Ok(out)
}

pub(crate) fn parse_commit(body: &str) -> Result<CommitObject> {
    let (headers, message) = body.split_once("\n\n").unwrap_or((body, ""));
    let mut tree = None;
    let mut parents = Vec::new();
    let mut author = None;
    for line in headers.lines() {
        if let Some(t) = line.strip_prefix("tree ") {
            tree = Some(t.to_string());
        } else if let Some(p) = line.strip_prefix("parent ") {
            parents.push(p.to_string());
        } else if let Some(a) = line.strip_prefix("author ") {
            author = Some(a);
        }
    }
    let tree = tree.ok_or_else(|| Error::Backend("commit without a tree".into()))?;
    let author = author.ok_or_else(|| Error::Backend("commit without an author".into()))?;
    let (ident, date) = author
        .rsplit_once("> ")
        .ok_or_else(|| Error::Backend(format!("unreadable author: {author}")))?;
    let (name, email) = ident
        .split_once(" <")
        .or_else(|| ident.strip_prefix('<').map(|e| ("", e)))
        .ok_or_else(|| Error::Backend(format!("unreadable author: {author}")))?;
    Ok(CommitObject {
        tree,
        parents,
        author_name: name.to_string(),
        author_email: email.to_string(),
        author_date: date.to_string(),
        message: message.to_string(),
    })
}

/// `commit` again with `message`, its parents mapped through `rewritten`,
/// its author kept, and the committer git would pick now. `sign` asks for
/// `-S`; otherwise `--no-gpg-sign`, whatever `commit.gpgSign` says.
pub(crate) fn recreate(
    root: &Path,
    commit: &CommitObject,
    message: &str,
    rewritten: &HashMap<String, String>,
    sign: bool,
) -> Result<String> {
    recreate_as(
        root,
        commit,
        message,
        rewritten,
        sign,
        &CommitterOverride::default(),
    )
}

/// [`recreate`] with a committer that may be forced.
pub(crate) fn recreate_as(
    root: &Path,
    commit: &CommitObject,
    message: &str,
    rewritten: &HashMap<String, String>,
    sign: bool,
    committer: &CommitterOverride,
) -> Result<String> {
    let mut args = vec![
        "commit-tree",
        if sign { "-S" } else { "--no-gpg-sign" },
        commit.tree.as_str(),
    ];
    for parent in &commit.parents {
        args.push("-p");
        args.push(rewritten.get(parent).unwrap_or(parent));
    }
    commit_tree_as(root, &args, commit, message, committer)
}

/// Runs a prepared `commit-tree` command with `author`'s name, email and
/// date.
pub(crate) fn commit_tree(
    root: &Path,
    args: &[&str],
    author: &CommitObject,
    message: &str,
) -> Result<String> {
    commit_tree_as(root, args, author, message, &CommitterOverride::default())
}

/// Committer fields to force on a re-created commit; whatever is `None`
/// comes from git's own identity and clock, as ever.
#[derive(Default)]
pub(crate) struct CommitterOverride {
    pub(crate) name: Option<String>,
    pub(crate) email: Option<String>,
    /// `<seconds> <+hhmm>`.
    pub(crate) date: Option<String>,
}

/// [`commit_tree`] with a committer that may be forced.
pub(crate) fn commit_tree_as(
    root: &Path,
    args: &[&str],
    author: &CommitObject,
    message: &str,
    committer: &CommitterOverride,
) -> Result<String> {
    // `@` makes git read the date exactly as stored, not guess at it.
    let date = format!("@{}", author.author_date);
    let committer_date = committer.date.as_ref().map(|d| format!("@{d}"));
    let mut env = vec![
        ("GIT_AUTHOR_NAME", author.author_name.as_str()),
        ("GIT_AUTHOR_EMAIL", author.author_email.as_str()),
        ("GIT_AUTHOR_DATE", date.as_str()),
    ];
    if let Some(name) = &committer.name {
        env.push(("GIT_COMMITTER_NAME", name.as_str()));
    }
    if let Some(email) = &committer.email {
        env.push(("GIT_COMMITTER_EMAIL", email.as_str()));
    }
    if let Some(date) = &committer_date {
        env.push(("GIT_COMMITTER_DATE", date.as_str()));
    }
    let out = run_with_env_and_stdin(root, args, &env, message)?;
    Ok(out.trim().to_string())
}

/// Re-creates `to_replay` (parents first, `objects` read from them) on top
/// of what `rewritten` already maps, adding each new commit to it.
pub(crate) fn replay(
    root: &Path,
    to_replay: &[String],
    objects: &[CommitObject],
    rewritten: &mut HashMap<String, String>,
    sign: bool,
) -> Result<()> {
    for (old, object) in to_replay.iter().zip(objects) {
        let new = recreate(root, object, &object.message, rewritten, sign)?;
        rewritten.insert(old.clone(), new);
    }
    Ok(())
}

/// `update HEAD` follows HEAD to the checked-out branch (or moves a
/// detached HEAD), checking it still is where we read it; the reflog
/// message lands on both. One transaction: ORIG_HEAD moves with it or
/// nothing does.
pub(crate) fn move_head(root: &Path, new_head: &str, old_head: &str, log: &str) -> Result<()> {
    run_with_stdin(
        root,
        &["update-ref", "-m", log, "--stdin"],
        &format!("update HEAD {new_head} {old_head}\nupdate ORIG_HEAD {old_head}\n"),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_commit_object_is_read_field_by_field() {
        let body = "tree abc\nparent p1\nparent p2\nauthor Ann Example <ann@x.org> 981173706 +0530\ncommitter C <c@x> 1 +0000\ngpgsig -----BEGIN\n more\n\nsubject\n\nbody\n";
        let c = parse_commit(body).unwrap();
        assert_eq!(c.tree, "abc");
        assert_eq!(c.parents, vec!["p1", "p2"]);
        assert_eq!(c.author_name, "Ann Example");
        assert_eq!(c.author_email, "ann@x.org");
        assert_eq!(c.author_date, "981173706 +0530");
        assert_eq!(c.message, "subject\n\nbody\n");
    }
}
