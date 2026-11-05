//! Refs for the rewrite: one `for-each-ref` lists everything the run needs,
//! the scope is resolved into the refs that move and the walk's tips, and
//! the file system tells which branches other worktrees have checked out and
//! which bases the workflow marks as protected.
//!
//! Never rewritten, never a tip: `refs/remotes/*` (they describe the remote),
//! `refs/gitpoulpi/*` (pins and the tag mirror), `refs/stash`,
//! `refs/replace/*`, `refs/original/*` and everything else outside
//! `refs/heads`, `refs/tags` and `refs/notes`.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::rewrite::{RewriteRefusal, Scope, ScopeOptions};
use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::Result;

use super::walk::{read_git, refuse, Head};
use crate::process::{failed_with, read_with_retry, spawn_failed};
use crate::refs::peel_info;

const HEADS: &str = "refs/heads/";
const TAGS: &str = "refs/tags/";

/// One line of `for-each-ref`.
#[derive(Clone)]
pub(crate) struct RefLine {
    pub(crate) name: String,
    pub(crate) oid: Oid,
    /// `commit`, `tag`, `tree` or `blob`.
    pub(crate) kind: String,
    /// `commit` for an annotated tag that peels (through any number of tags)
    /// to a commit, else empty. Peeled by `cat-file`, never by a `*` field of
    /// `for-each-ref`: git 2.5x aborts on a ref to a missing object.
    pub(crate) peeled_kind: String,
    /// `refs/remotes/<remote>/<branch>` of a branch's upstream, or empty.
    pub(crate) upstream: String,
}

impl RefLine {
    /// Whether a tag reaches a commit (a tag of a tag is followed, a tag of
    /// a tree or blob, or of a missing object, is not rewritten).
    pub(crate) fn reaches_commit(&self) -> bool {
        self.kind == "commit" || (self.kind == "tag" && self.peeled_kind == "commit")
    }
}

#[derive(Default)]
pub(crate) struct Refs {
    pub(crate) heads: Vec<RefLine>,
    pub(crate) tags: Vec<RefLine>,
    pub(crate) notes: Vec<RefLine>,
    /// `refs/remotes/*`.
    pub(crate) remotes: Vec<RefLine>,
    /// `refs/gitpoulpi/remote-tags/*`: the tags known to be on a remote.
    pub(crate) mirrors: Vec<RefLine>,
}

/// One `for-each-ref`: branches, tags, notes, remote-tracking refs and the
/// tag mirror.
pub(crate) fn list(root: &Path) -> Result<Refs> {
    let args = [
        "for-each-ref",
        "--format=%(refname)%00%(objectname)%00%(upstream)",
        "refs/heads",
        "refs/tags",
        "refs/notes",
        "refs/remotes",
        "refs/gitpoulpi/remote-tags",
    ];
    let out = read_with_retry(|| {
        let out = read_git(root, &args).output().map_err(spawn_failed)?;
        if out.status.success() {
            Ok(out)
        } else {
            Err(failed_with(
                &args,
                out.status.code(),
                &String::from_utf8_lossy(&out.stderr),
            ))
        }
    })?;
    let mut lines: Vec<RefLine> = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let f: Vec<&str> = line.split('\0').collect();
        if f.len() < 3 {
            continue;
        }
        let Some(oid) = Oid::from_hex(f[1]) else {
            return Err(refuse(
                RewriteRefusal::UnsupportedObjectFormat,
                "only SHA-1 repositories can be rewritten",
            ));
        };
        let r = RefLine {
            name: f[0].to_string(),
            oid,
            kind: String::new(),
            peeled_kind: String::new(),
            upstream: f[2].to_string(),
        };
        lines.push(r);
    }
    // Type and peeled commit from `cat-file`, never from `%(objecttype)` /
    // `%(*objecttype)`: git 2.5x aborts (or exits 128) on a ref to a missing
    // object. Such a ref is left out: there is nothing to rewrite.
    let mut ids: Vec<Oid> = lines.iter().map(|r| r.oid).collect();
    ids.sort_by_key(|o| o.0);
    ids.dedup();
    let info = peel_info(root, &ids)?;
    let mut refs = Refs::default();
    for mut r in lines {
        let Some(peel) = info.get(&r.oid).filter(|p| p.kind != "missing") else {
            continue;
        };
        r.kind = peel.kind.clone();
        if peel.commit.is_some() {
            r.peeled_kind = "commit".to_string();
        }
        let bucket = if r.name.starts_with(HEADS) {
            &mut refs.heads
        } else if r.name.starts_with(TAGS) {
            &mut refs.tags
        } else if r.name.starts_with("refs/notes/") {
            &mut refs.notes
        } else if r.name.starts_with("refs/remotes/") {
            &mut refs.remotes
        } else {
            &mut refs.mirrors
        };
        bucket.push(r);
    }
    Ok(refs)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RefKind {
    Branch,
    Tag,
    /// A detached HEAD.
    Head,
}

/// A ref the scope names.
#[derive(Clone)]
pub(crate) struct ScopeRef {
    pub(crate) name: String,
    pub(crate) old: Oid,
    pub(crate) kind: RefKind,
}

/// Which commits the rules apply to.
pub(crate) enum Active {
    All,
    /// Exactly these.
    Set(HashSet<Oid>),
    /// The commits after the walk's base up to this tip.
    UpTo(Oid),
}

/// What a scope asks of the walk.
pub(crate) struct Resolved {
    /// The refs that move because the scope says so, in plan order.
    pub(crate) in_scope: Vec<ScopeRef>,
    /// `rev-list` positives.
    pub(crate) tips: Vec<String>,
    /// `rev-list` negatives.
    pub(crate) not: Vec<String>,
    pub(crate) active: Active,
    /// Whether tags move that no scope entry names (they follow commits).
    pub(crate) tags_follow: bool,
}

fn branch_ref(refs: &Refs, name: &str) -> Option<ScopeRef> {
    refs.heads
        .iter()
        .find(|r| r.name == name)
        .map(|r| ScopeRef {
            name: r.name.clone(),
            old: r.oid,
            kind: RefKind::Branch,
        })
}

fn named(refs: &Refs, name: &str) -> Result<Option<ScopeRef>> {
    let tag = |full: &str| {
        refs.tags
            .iter()
            .find(|r| r.name == full && r.reaches_commit())
            .map(|r| ScopeRef {
                name: r.name.clone(),
                old: r.oid,
                kind: RefKind::Tag,
            })
    };
    if name.starts_with(HEADS) {
        return Ok(branch_ref(refs, name));
    }
    if name.starts_with(TAGS) {
        return Ok(tag(name));
    }
    if name.starts_with("refs/") {
        return Err(refuse(
            RewriteRefusal::EmptyScope,
            &format!("{name} is never rewritten"),
        ));
    }
    Ok(branch_ref(refs, &format!("{HEADS}{name}")).or_else(|| tag(&format!("{TAGS}{name}"))))
}

fn spec(r: &ScopeRef) -> String {
    match r.kind {
        RefKind::Head => r.old.to_hex(),
        _ => r.name.clone(),
    }
}

/// The refs a recipe's scope moves, and what the walk reads for it.
pub(crate) fn resolve(
    scope: &Scope,
    options: &ScopeOptions,
    refs: &Refs,
    head: &Head,
) -> Result<Resolved> {
    let current = || -> Result<ScopeRef> {
        match head {
            Head::Branch(name) => branch_ref(refs, name)
                .ok_or_else(|| refuse(RewriteRefusal::EmptyScope, "the branch has no commit")),
            Head::Detached(oid) => Ok(ScopeRef {
                name: "HEAD".into(),
                old: *oid,
                kind: RefKind::Head,
            }),
        }
    };
    let mut out = Resolved {
        in_scope: Vec::new(),
        tips: Vec::new(),
        not: Vec::new(),
        active: Active::All,
        tags_follow: options.include_tags,
    };
    match scope {
        Scope::CurrentBranch { since } => {
            out.in_scope.push(current()?);
            out.not.extend(since.map(|s| s.to_hex()));
        }
        Scope::Selection { commits } => {
            out.in_scope.push(current()?);
            out.active = Active::Set(commits.iter().copied().collect());
        }
        Scope::Range { base, tip } => {
            out.in_scope.push(current()?);
            out.not.push(base.to_hex());
            out.active = Active::UpTo(*tip);
        }
        Scope::Refs { refs: names } => {
            for name in names.iter().filter(|n| !n.trim().is_empty()) {
                match named(refs, name.trim())? {
                    Some(r) => out.in_scope.push(r),
                    None => {
                        return Err(refuse(
                            RewriteRefusal::EmptyScope,
                            &format!("{name} is no local branch or tag"),
                        ))
                    }
                }
            }
        }
        Scope::AllLocal => {
            for r in &refs.heads {
                out.in_scope.push(ScopeRef {
                    name: r.name.clone(),
                    old: r.oid,
                    kind: RefKind::Branch,
                });
            }
            if options.include_tags {
                for r in refs.tags.iter().filter(|r| r.reaches_commit()) {
                    out.in_scope.push(ScopeRef {
                        name: r.name.clone(),
                        old: r.oid,
                        kind: RefKind::Tag,
                    });
                }
            }
        }
    }
    for name in &options.extra_refs {
        match named(refs, name.trim())? {
            Some(r) => out.in_scope.push(r),
            None => {
                return Err(refuse(
                    RewriteRefusal::EmptyScope,
                    &format!("{name} is no local branch or tag"),
                ))
            }
        }
    }
    let mut seen = HashSet::new();
    out.in_scope.retain(|r| seen.insert(r.name.clone()));
    if out.in_scope.is_empty() {
        return Err(refuse(RewriteRefusal::EmptyScope, "the scope names no ref"));
    }
    out.tips = if matches!(scope, Scope::AllLocal) {
        // Globs: a thousand branches must not make a command line too long.
        let mut tips = vec!["--branches".to_string()];
        if options.include_tags {
            tips.push("--tags".to_string());
        }
        tips
    } else {
        out.in_scope.iter().map(spec).collect()
    };
    Ok(out)
}

/// Branches (and names, short or full) the workflow marks as protected, as
/// full ref names: `<git common dir>/gitpoulpi/workflow.toml` and the
/// committed `<root>/.gitpoulpi/workflow.toml`, plus what the caller
/// resolved from a state directory this crate cannot guess.
pub(crate) fn protected_branches(root: &Path, common_dir: &Path, extra: &[String]) -> Protected {
    let mut names: HashSet<String> = extra.iter().cloned().collect();
    for file in [
        common_dir.join("gitpoulpi").join("workflow.toml"),
        root.join(".gitpoulpi").join("workflow.toml"),
    ] {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let Ok(workflow) = toml::from_str::<gitpoulpi_core::workflow::Workflow>(&text) else {
            continue;
        };
        for base in workflow.bases.iter().filter(|b| b.protected) {
            names.extend(base.all_names().map(str::to_string));
        }
    }
    let full = |n: String| {
        if n.starts_with(HEADS) {
            n
        } else {
            format!("{HEADS}{n}")
        }
    };
    let (globs, exact): (Vec<String>, Vec<String>) =
        names.into_iter().partition(|n| n.contains('*'));
    Protected {
        exact: exact.into_iter().map(full).collect(),
        globs: globs.into_iter().map(full).collect(),
    }
}

/// The protected branches: full names, and patterns where `*` stands for
/// any text (`refs/heads/release/*`).
pub(crate) struct Protected {
    exact: HashSet<String>,
    globs: Vec<String>,
}

impl Protected {
    pub(crate) fn contains(&self, name: &String) -> bool {
        self.exact.contains(name) || self.globs.iter().any(|g| glob_match(g, name))
    }
}

/// `*` matches any run of characters (slashes included); everything else is
/// literal.
pub(crate) fn glob_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    let [first, middle @ .., last] = parts.as_slice() else {
        return pattern == text;
    };
    let Some(mut rest) = text.strip_prefix(first) else {
        return false;
    };
    for part in middle {
        match rest.find(part) {
            Some(at) => rest = &rest[at + part.len()..],
            None => return false,
        }
    }
    rest.ends_with(last)
}

fn canonical(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

fn checked_out(head_file: &Path) -> Option<String> {
    let text = std::fs::read_to_string(head_file).ok()?;
    let name = text.trim().strip_prefix("ref:")?.trim();
    name.starts_with(HEADS).then(|| name.to_string())
}

/// The branches checked out in worktrees other than the one at `git_dir`,
/// with the worktree's directory. Read from the git dir's files: no spawn.
pub(crate) fn other_worktrees(common_dir: &Path, git_dir: &Path) -> HashMap<String, PathBuf> {
    let mut out = HashMap::new();
    let here = canonical(git_dir);
    if canonical(common_dir) != here {
        // The main worktree: the common dir is its `.git`.
        if let (Some(branch), Some(dir)) =
            (checked_out(&common_dir.join("HEAD")), common_dir.parent())
        {
            if common_dir.file_name().is_some_and(|n| n == ".git") {
                out.insert(branch, dir.to_path_buf());
            }
        }
    }
    let Ok(entries) = std::fs::read_dir(common_dir.join("worktrees")) else {
        return out;
    };
    for entry in entries.flatten() {
        let dir = entry.path();
        if canonical(&dir) == here {
            continue;
        }
        let Some(branch) = checked_out(&dir.join("HEAD")) else {
            continue;
        };
        let Ok(gitdir) = std::fs::read_to_string(dir.join("gitdir")) else {
            continue;
        };
        if let Some(worktree) = Path::new(gitdir.trim()).parent() {
            out.insert(branch, worktree.to_path_buf());
        }
    }
    out
}
