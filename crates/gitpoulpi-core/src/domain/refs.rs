use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefKind {
    LocalBranch { name: String },
    RemoteBranch { remote: String, name: String },
    Tag { name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RefVisibility {
    #[default]
    Shown,
    Hidden,
    Solo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ref {
    pub kind: RefKind,
    pub target: super::Oid,
    pub visibility: RefVisibility,
    /// True for the one local branch HEAD currently points to (backend-
    /// reported: `Repository::head()` / `git symbolic-ref --short HEAD`),
    /// never inferred client-side. Always false for remote branches, tags,
    /// and every ref when HEAD is detached.
    pub is_head: bool,
    /// For a local branch that has one, the *shorthand* name of the
    /// remote-tracking branch it tracks (`origin/feature/login`), exactly
    /// as that ref's own `RefKind::RemoteBranch` would spell it
    /// (`format!("{remote}/{name}")`). `None` for a local branch with no
    /// upstream, and always `None` for remote branches and tags.
    ///
    /// It lives on `Ref` rather than a second call so the two answers
    /// cannot disagree about a branch created in between.
    #[serde(default)]
    pub upstream: Option<String>,
    /// For a tag: which remotes are known to have it (see [`TagSync`]).
    /// `None` for every non-tag ref, and for a tag whose sync state was
    /// not computed - which is a different statement from "no remote
    /// has it" and must render differently.
    #[serde(default)]
    pub tag_sync: Option<TagSync>,
}

/// The ref namespace GitPoulpi keeps its *remote tag* bookkeeping in.
///
/// git has no remote-tracking refs for tags: `git fetch` copies a remote's
/// tags into `refs/tags/*` beside the local ones, and `git ls-remote` is a
/// network round trip that must never sit on a render path. So GitPoulpi
/// keeps a local mirror of what the remote last said it had,
/// `refs/gitpoulpi/remote-tags/<remote>/<tag>`, which
/// [`crate::GitBackend::refs`] answers from for free. `fetch` refreshes it
/// and a tag push or remote tag delete updates it at once.
///
/// Deliberately **not** under `refs/remotes/` (git's; a tag there would
/// read as a branch) and excluded from `LogOptions::all_refs`, which
/// enumerates heads/tags/remotes by name rather than saying `--all`, so it
/// never adds a line to the commit graph.
pub const REMOTE_TAG_MIRROR_PREFIX: &str = "refs/gitpoulpi/remote-tags/";

/// The mirror refname recording that `remote` has tag `tag`.
pub fn remote_tag_mirror_ref(remote: &str, tag: &str) -> String {
    format!("{REMOTE_TAG_MIRROR_PREFIX}{remote}/{tag}")
}

/// The `(remote, tag)` a mirror refname records, or `None` for any other
/// ref.
///
/// The part after the prefix must never be split on the first `/`: a
/// remote name may contain one (`grp/sub`) and so may a tag
/// (`release/1.0`), so a left-anchored split files `grp/sub` + `v1` under
/// remote `grp`. Reuses [`split_remote_ref`]'s resolution (against the
/// repository's *actual* remote list, longest match first, first-`/`
/// fallback for a removed remote).
pub fn parse_remote_tag_mirror_ref<'a>(
    refname: &'a str,
    remote_names: &[String],
) -> Option<(String, &'a str)> {
    let rest = refname.strip_prefix(REMOTE_TAG_MIRROR_PREFIX)?;
    let (remote, tag) = split_remote_ref(rest, remote_names)?;
    (!remote.is_empty() && !tag.is_empty()).then_some((remote, tag))
}

/// Which remotes a tag is known to exist on - the state that decides
/// whether a tag renders as "unpushed" and gets a `Push <tag> to
/// <remote>` entry in its menu.
///
/// Empty `pushed_to` means *no remote we know of has this tag at this
/// commit*, which is the state a client marks. Learned entirely from the
/// local mirror namespace (see [`REMOTE_TAG_MIRROR_PREFIX`]), never from
/// the network; the mirror is refreshed by every fetch and by our own tag
/// push / remote delete.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagSync {
    /// Remote names whose mirror holds this tag, pointing at the same
    /// commit, sorted. A tag the remote has at a *different* commit is
    /// not listed: it would push only with `--force`, so calling it
    /// "pushed" would hide a real divergence (see `differs_on`).
    pub pushed_to: Vec<String>,
    /// Remote names that have a tag of this name at a *different* commit,
    /// sorted.
    #[serde(default)]
    pub differs_on: Vec<String>,
    /// Whether `refs/tags/<name>` exists in this repository. False means
    /// the tag is **remote only**: a remote has it and it was never
    /// fetched here (or was deleted here). The row's `target` is then the
    /// commit the remote's tag points at, read from the mirror.
    #[serde(default = "default_true")]
    pub local: bool,
}

fn default_true() -> bool {
    true
}

impl Default for TagSync {
    fn default() -> Self {
        TagSync {
            pushed_to: Vec::new(),
            differs_on: Vec::new(),
            local: true,
        }
    }
}

/// Where a tag lives, the one answer every surface (sidebar, Log pills,
/// menus, search, CLI) renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TagWhere {
    /// Here only; no known remote has it.
    Local,
    /// On a remote only; not fetched (or deleted) here.
    Remote,
    /// Here and on a remote, at the same commit.
    Both,
    /// Here and on a remote, but at different commits.
    Differs,
}

impl TagWhere {
    /// The stable word the CLI/MCP `where` field carries.
    pub fn word(self) -> &'static str {
        match self {
            TagWhere::Local => "local",
            TagWhere::Remote => "remote",
            TagWhere::Both => "both",
            TagWhere::Differs => "differs",
        }
    }
}

impl TagSync {
    pub fn is_unpushed(&self) -> bool {
        self.pushed_to.is_empty()
    }

    /// Every remote that has a tag of this name, same commit or not.
    pub fn on_remotes(&self) -> Vec<String> {
        let mut all: Vec<String> = self
            .pushed_to
            .iter()
            .chain(self.differs_on.iter())
            .cloned()
            .collect();
        all.sort();
        all.dedup();
        all
    }

    pub fn where_(&self) -> TagWhere {
        let on_remote = !self.pushed_to.is_empty() || !self.differs_on.is_empty();
        match (self.local, on_remote) {
            (false, _) => TagWhere::Remote,
            (true, false) => TagWhere::Local,
            (true, true) if !self.differs_on.is_empty() => TagWhere::Differs,
            (true, true) => TagWhere::Both,
        }
    }
}

impl Ref {
    /// The tag name, for a tag; `None` for anything else.
    pub fn tag_name(&self) -> Option<&str> {
        match &self.kind {
            RefKind::Tag { name } => Some(name.as_str()),
            _ => None,
        }
    }

    /// A tag no remote is known to have. False for every non-tag ref, and
    /// false for a tag whose sync state was never computed (`None`) --
    /// "we did not look" must not render as "it is not pushed".
    pub fn is_unpushed_tag(&self) -> bool {
        self.tag_name().is_some()
            && matches!(&self.tag_sync, Some(sync) if sync.local && sync.is_unpushed())
    }

    /// A tag a remote has and this repository does not (not fetched, or
    /// deleted here): [`TagWhere::Remote`].
    pub fn is_remote_only_tag(&self) -> bool {
        self.tag_name().is_some() && matches!(&self.tag_sync, Some(sync) if !sync.local)
    }

    /// Where this tag lives, for a tag whose sync state was computed.
    pub fn tag_where(&self) -> Option<TagWhere> {
        self.tag_name()?;
        self.tag_sync.as_ref().map(TagSync::where_)
    }
}

/// Split the part of a ref name after `refs/remotes/` into the remote it
/// belongs to and the branch name inside it.
///
/// **Never split on the first `/`.** A remote name may itself contain
/// separators (`git remote add grp/sub <url>` is legal), so
/// `refs/remotes/grp/sub/main` is remote `grp/sub`, branch `main` - and
/// splitting on the first separator gives remote `grp`, branch
/// `sub/main`, which names a remote that does not exist and makes every
/// join against `Remote::name` miss. Resolution is against the
/// repository's *actual* remote list, longest match first, so a remote
/// `origin` and a remote `origin/mirror` can coexist and the more
/// specific one wins.
///
/// One function in the domain crate so the backends cannot split the same
/// ref differently.
///
/// `remote_names` being empty, or matching nothing (a stale
/// remote-tracking ref whose remote has since been removed), falls back
/// to the first separator - such a ref still has to appear somewhere
/// rather than vanish.
pub fn split_remote_ref<'a>(rest: &'a str, remote_names: &[String]) -> Option<(String, &'a str)> {
    let mut best: Option<&str> = None;
    for candidate in remote_names {
        if let Some(branch) = rest.strip_prefix(candidate.as_str()) {
            if branch.starts_with('/')
                && branch.len() > 1
                && best.is_none_or(|b| candidate.len() > b.len())
            {
                best = Some(candidate.as_str());
            }
        }
    }
    match best {
        Some(remote) => Some((remote.to_string(), &rest[remote.len() + 1..])),
        None => rest
            .split_once('/')
            .map(|(remote, branch)| (remote.to_string(), branch)),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_remote_tag_mirror_ref, split_remote_ref};

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_mirror_ref_with_an_ordinary_remote_name_splits_on_its_own_first_slash() {
        assert_eq!(
            parse_remote_tag_mirror_ref(
                "refs/gitpoulpi/remote-tags/origin/v1",
                &names(&["origin"])
            ),
            Some(("origin".to_string(), "v1"))
        );
    }

    #[test]
    fn a_mirror_ref_whose_tag_contains_a_slash_keeps_it_whole() {
        assert_eq!(
            parse_remote_tag_mirror_ref(
                "refs/gitpoulpi/remote-tags/origin/release/1.0",
                &names(&["origin"])
            ),
            Some(("origin".to_string(), "release/1.0"))
        );
    }

    #[test]
    fn a_mirror_ref_whose_remote_contains_a_slash_is_not_split_in_half() {
        // The bug this pins: splitting on the first `/` reads this as
        // remote `grp`, tag `sub/v1` -- a remote that does not exist,
        // and the tag reports as unpushed to it forever.
        assert_eq!(
            parse_remote_tag_mirror_ref(
                "refs/gitpoulpi/remote-tags/grp/sub/v1",
                &names(&["grp/sub"])
            ),
            Some(("grp/sub".to_string(), "v1"))
        );
    }

    #[test]
    fn both_the_remote_and_the_tag_may_contain_a_slash_at_once() {
        assert_eq!(
            parse_remote_tag_mirror_ref(
                "refs/gitpoulpi/remote-tags/grp/sub/release/1.0",
                &names(&["grp/sub"])
            ),
            Some(("grp/sub".to_string(), "release/1.0"))
        );
    }

    #[test]
    fn a_mirror_ref_for_a_remote_that_no_longer_exists_still_parses() {
        assert_eq!(
            parse_remote_tag_mirror_ref("refs/gitpoulpi/remote-tags/gone/v1", &names(&["origin"])),
            Some(("gone".to_string(), "v1"))
        );
    }

    #[test]
    fn a_non_mirror_ref_is_not_a_mirror_ref() {
        assert_eq!(
            parse_remote_tag_mirror_ref("refs/tags/v1", &names(&["origin"])),
            None
        );
    }

    #[test]
    fn the_ordinary_case_is_the_first_segment() {
        assert_eq!(
            split_remote_ref("origin/feature/login", &names(&["origin"])),
            Some(("origin".into(), "feature/login"))
        );
    }

    #[test]
    fn a_remote_whose_name_contains_a_slash_is_not_split_in_half() {
        assert_eq!(
            split_remote_ref("grp/sub/main", &names(&["grp/sub"])),
            Some(("grp/sub".into(), "main"))
        );
    }

    #[test]
    fn the_longest_matching_remote_wins() {
        // Both remotes are a prefix of this ref. `origin` alone would
        // claim it and call the branch `mirror/main`.
        assert_eq!(
            split_remote_ref("origin/mirror/main", &names(&["origin", "origin/mirror"])),
            Some(("origin/mirror".into(), "main"))
        );
    }

    #[test]
    fn a_remote_name_is_only_matched_at_a_separator() {
        // `orig` must not claim `origin/main` by bare string prefix.
        assert_eq!(
            split_remote_ref("origin/main", &names(&["orig"])),
            Some(("origin".into(), "main"))
        );
    }

    #[test]
    fn an_unknown_remote_still_yields_a_ref_rather_than_nothing() {
        assert_eq!(
            split_remote_ref("gone/main", &names(&["origin"])),
            Some(("gone".into(), "main"))
        );
        assert_eq!(split_remote_ref("nameless", &names(&["origin"])), None);
    }
}
