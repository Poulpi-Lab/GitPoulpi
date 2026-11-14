//! The plan of a grouped push or pull: how a branch stands against each of
//! its remotes (after a preflight), and what to do about each by default.
//!
//! Pure, no I/O. The daemon gathers the facts (`ls-remote`, selective
//! fetch, cherry counts, reflog, conflict prediction) and feeds them to
//! [`classify`] and [`suggest`]; the UI and CLI show the result and let the
//! user pick another of the [`allowed`](PlannedTarget::allowed) actions.

use serde::{Deserialize, Serialize};

use super::remote_group::{RemoteRelation, TargetRef, TargetRole};
use super::Oid;

/// Which way a group runs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupMode {
    #[default]
    Push,
    Pull,
}

/// What one target's row does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupAction {
    Push,
    FastForwardFrom,
    MergeFrom,
    RebaseOnto,
    ForceWithLease,
    Skip,
}

/// What the preflight found for one target, before any suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteClass {
    pub relation: RemoteRelation,
    /// Diverged only: the remote's side is just older copies of local
    /// commits (no unmatched remote commit) or its tip is in the branch's
    /// reflog ("you rebased or amended what you pushed").
    pub rewritten: bool,
}

impl RemoteClass {
    /// Whether the remote already holds everything local has (equal, or
    /// ahead of local, or only older copies of rebased local commits).
    pub fn holds_local(&self, local_unmatched: Option<u32>) -> bool {
        match self.relation {
            RemoteRelation::UpToDate | RemoteRelation::RemoteAhead(_) => true,
            RemoteRelation::Diverged { .. } => local_unmatched == Some(0),
            _ => false,
        }
    }
}

/// The facts of one target as the daemon read them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationFacts {
    /// `(ahead, behind)` of local against the remote's tip; `None` when the
    /// remote has no such branch.
    pub ahead_behind: Option<(u32, u32)>,
    /// Local and the remote tip share a merge base.
    pub related: bool,
    /// `cherry_counts` remote side: remote commits with no patch-equal
    /// local commit. `None` when not computed.
    pub remote_unmatched: Option<u32>,
    /// The remote's tip appears in the local branch's reflog.
    pub remote_tip_in_reflog: bool,
}

/// Classifies one target.
pub fn classify(f: &RelationFacts) -> RemoteClass {
    let (relation, rewritten) = match f.ahead_behind {
        None => (RemoteRelation::Absent, false),
        Some((0, 0)) => (RemoteRelation::UpToDate, false),
        Some((a, 0)) => (RemoteRelation::LocalAhead(a), false),
        Some((0, b)) => (RemoteRelation::RemoteAhead(b), false),
        Some((ahead, behind)) => {
            if !f.related {
                (RemoteRelation::Unrelated, false)
            } else {
                (
                    RemoteRelation::Diverged { ahead, behind },
                    f.remote_unmatched == Some(0) || f.remote_tip_in_reflog,
                )
            }
        }
    };
    RemoteClass {
        relation,
        rewritten,
    }
}

/// What [`suggest`] needs besides the class.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SuggestContext {
    pub mode: GroupMode,
    /// `git.pull_mode` is rebase.
    pub prefer_rebase: bool,
    /// Another target already holds the local commits, so a rebase would
    /// need a force push there.
    pub others_already_have_local: bool,
    /// The host or a hook refused this target (protected branch): force
    /// would be refused too.
    pub refused: bool,
}

/// A default action and the others the user may pick (default first).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suggestion {
    pub action: GroupAction,
    pub allowed: Vec<GroupAction>,
}

fn pick(action: GroupAction, rest: &[GroupAction]) -> Suggestion {
    let mut allowed = vec![action];
    allowed.extend(rest.iter().copied().filter(|a| *a != action));
    Suggestion { action, allowed }
}

/// The default action per relation (the table in the multi-remote plan,
/// section 4) and the allowed set.
pub fn suggest(class: &RemoteClass, ctx: &SuggestContext) -> Suggestion {
    use GroupAction as A;
    if ctx.refused {
        return pick(A::Skip, &[]);
    }
    match ctx.mode {
        GroupMode::Push => match class.relation {
            // Equal now, but another row may move local: a no-op push is
            // harmless, skipping would leave this remote behind.
            RemoteRelation::UpToDate | RemoteRelation::LocalAhead(_) | RemoteRelation::Absent => {
                pick(A::Push, &[A::Skip])
            }
            RemoteRelation::RemoteAhead(_) => pick(A::FastForwardFrom, &[A::Skip]),
            RemoteRelation::Diverged { .. } if class.rewritten => {
                pick(A::ForceWithLease, &[A::MergeFrom, A::RebaseOnto, A::Skip])
            }
            RemoteRelation::Diverged { .. } => {
                let first = if ctx.prefer_rebase && !ctx.others_already_have_local {
                    A::RebaseOnto
                } else {
                    A::MergeFrom
                };
                pick(
                    first,
                    &[A::MergeFrom, A::RebaseOnto, A::ForceWithLease, A::Skip],
                )
            }
            RemoteRelation::Unrelated => pick(A::Skip, &[A::ForceWithLease]),
        },
        // Pull: sources follow the pull mode; nothing is forced.
        GroupMode::Pull => match class.relation {
            RemoteRelation::UpToDate
            | RemoteRelation::LocalAhead(_)
            | RemoteRelation::Absent
            | RemoteRelation::Unrelated => pick(A::Skip, &[]),
            RemoteRelation::RemoteAhead(_) => pick(A::FastForwardFrom, &[A::Skip]),
            RemoteRelation::Diverged { .. } => {
                let first = if ctx.prefer_rebase {
                    A::RebaseOnto
                } else {
                    A::MergeFrom
                };
                pick(first, &[A::MergeFrom, A::RebaseOnto, A::Skip])
            }
        },
    }
}

/// A comparable form of a remote URL: scheme and credentials dropped, host
/// lower-cased, `git@host:path` turned into `host/path`, a trailing `.git`
/// and slashes removed, so two URLs of one repository compare equal.
/// Local paths only lose the trailing `/` and `.git`.
pub fn normalize_url(url: &str) -> String {
    let url = url.trim().replace('\\', "/");
    let (scheme, rest) = match url.split_once("://") {
        Some((s, r)) => (Some(s.to_ascii_lowercase()), r.to_string()),
        None => (None, url.clone()),
    };
    let local = scheme.as_deref() == Some("file") || (scheme.is_none() && !looks_scp(&rest));
    let mut out = if local {
        // `file:///srv/x` leaves `/srv/x`.
        if scheme.is_some() && !rest.starts_with('/') {
            format!("/{rest}")
        } else {
            rest
        }
    } else {
        // host[:port]/path, or scp-like user@host:path.
        let (authority, path) = if scheme.is_none() {
            let (a, p) = rest.split_once(':').unwrap_or((&rest, ""));
            (a.to_string(), p.trim_start_matches('/').to_string())
        } else {
            let (a, p) = rest.split_once('/').unwrap_or((&rest, ""));
            (a.to_string(), p.to_string())
        };
        let host = authority.rsplit('@').next().unwrap_or(&authority);
        let host = if scheme.is_some() {
            host.split(':').next().unwrap_or(host)
        } else {
            host
        };
        format!("{}/{path}", host.to_ascii_lowercase())
    };
    while out.ends_with('/') {
        out.pop();
    }
    if let Some(s) = out.strip_suffix(".git") {
        out = s.to_string();
    }
    out
}

/// `host:path` or `user@host:path` (not a Windows drive).
fn looks_scp(s: &str) -> bool {
    if s.starts_with('/') {
        return false;
    }
    match s.split_once(':') {
        Some((host, _)) => host.len() > 1 && !host.contains('/'),
        None => false,
    }
}

/// One row of a plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedTarget {
    pub remote: String,
    /// The branch name on that remote.
    pub branch: String,
    pub role: TargetRole,
    pub relation: RemoteRelation,
    /// Diverged because local was rebased or amended (see [`RemoteClass`]).
    #[serde(default)]
    pub rewritten: bool,
    pub ahead: u32,
    pub behind: u32,
    /// Patch-unmatched counts (`cherry_counts`), when computed.
    #[serde(default)]
    pub local_unmatched: Option<u32>,
    #[serde(default)]
    pub remote_unmatched: Option<u32>,
    pub suggested: GroupAction,
    pub allowed: Vec<GroupAction>,
    /// What the remote held when the plan looked: the oid a
    /// `ForceWithLease` push must expect. `None` for an absent branch
    /// (push with `PushLease::Absent`).
    #[serde(default)]
    pub lease: Option<Oid>,
    /// Paths a merge of the remote's tip into local would conflict in
    /// (Diverged rows only).
    #[serde(default)]
    pub conflicts: Vec<String>,
    /// An earlier target of this plan is the same repository and branch
    /// under another remote name.
    #[serde(default)]
    pub same_url_as: Option<String>,
    /// The remote was asked (`ls-remote`).
    #[serde(default)]
    pub checked: bool,
    /// Its objects were fetched because the plan lacked its tip.
    #[serde(default)]
    pub fetched: bool,
    /// The remote could not be asked or fetched; the row falls back to the
    /// local tracking ref and defaults to Skip.
    #[serde(default)]
    pub error: Option<String>,
    /// The remote is not configured.
    #[serde(default)]
    pub missing: bool,
}

/// Answer of `RemoteGroupPlan`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteGroupPlan {
    pub branch: String,
    pub mode: GroupMode,
    pub local: Oid,
    /// Whether the remotes were asked (no network when false).
    pub preflight: bool,
    /// In run order: the primary first, then the others.
    pub targets: Vec<PlannedTarget>,
}

fn default_true() -> bool {
    true
}

/// Arguments of `RemoteGroupPlan`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteGroupPlanRequest {
    pub branch: String,
    #[serde(default)]
    pub mode: GroupMode,
    /// An explicit list (a one-off "push to selected remotes"); `None` is
    /// the configured primary plus the push targets (push) or the primary
    /// plus the pull sources (pull).
    #[serde(default)]
    pub targets: Option<Vec<TargetRef>>,
    /// Ask the remotes (`ls-remote`, fetch when needed). Default on.
    #[serde(default = "default_true")]
    pub preflight: bool,
    /// `git.pull_mode` is rebase.
    #[serde(default)]
    pub prefer_rebase: bool,
    /// Targets a push was already refused on (protected): they default to
    /// Skip and offer nothing else.
    #[serde(default)]
    pub refused: Vec<TargetRef>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use GroupAction as A;

    fn facts(ab: Option<(u32, u32)>) -> RelationFacts {
        RelationFacts {
            ahead_behind: ab,
            related: true,
            remote_unmatched: None,
            remote_tip_in_reflog: false,
        }
    }

    fn class(f: RelationFacts) -> RemoteClass {
        classify(&f)
    }

    fn ctx() -> SuggestContext {
        SuggestContext::default()
    }

    #[test]
    fn classification_covers_every_relation() {
        assert_eq!(class(facts(None)).relation, RemoteRelation::Absent);
        assert_eq!(
            class(facts(Some((0, 0)))).relation,
            RemoteRelation::UpToDate
        );
        assert_eq!(
            class(facts(Some((2, 0)))).relation,
            RemoteRelation::LocalAhead(2)
        );
        assert_eq!(
            class(facts(Some((0, 3)))).relation,
            RemoteRelation::RemoteAhead(3)
        );
        let d = class(facts(Some((2, 1))));
        assert_eq!(
            d.relation,
            RemoteRelation::Diverged {
                ahead: 2,
                behind: 1
            }
        );
        assert!(!d.rewritten);
        let mut u = facts(Some((2, 1)));
        u.related = false;
        assert_eq!(class(u).relation, RemoteRelation::Unrelated);
    }

    #[test]
    fn rewritten_by_cherry_counts_or_reflog() {
        let mut f = facts(Some((2, 2)));
        f.remote_unmatched = Some(0);
        assert!(class(f).rewritten);
        let mut f = facts(Some((2, 2)));
        f.remote_unmatched = Some(1);
        assert!(!class(f).rewritten);
        f.remote_tip_in_reflog = true;
        assert!(class(f).rewritten);
        // Never "rewritten" when not diverged.
        let mut f = facts(Some((0, 2)));
        f.remote_tip_in_reflog = true;
        assert!(!class(f).rewritten);
    }

    #[test]
    fn table_up_to_date_and_ahead_push() {
        for ab in [Some((0, 0)), Some((3, 0)), None] {
            let s = suggest(&class(facts(ab)), &ctx());
            assert_eq!(s.action, A::Push, "{ab:?}");
            assert_eq!(s.allowed, vec![A::Push, A::Skip]);
        }
    }

    #[test]
    fn table_remote_ahead_fast_forwards() {
        let s = suggest(&class(facts(Some((0, 2)))), &ctx());
        assert_eq!(s.action, A::FastForwardFrom);
        assert_eq!(s.allowed, vec![A::FastForwardFrom, A::Skip]);
    }

    #[test]
    fn table_rewritten_diverged_forces_with_lease() {
        let mut f = facts(Some((2, 2)));
        f.remote_unmatched = Some(0);
        let s = suggest(&class(f), &ctx());
        assert_eq!(s.action, A::ForceWithLease);
        assert_eq!(
            s.allowed,
            vec![A::ForceWithLease, A::MergeFrom, A::RebaseOnto, A::Skip]
        );
    }

    #[test]
    fn table_real_divergence_merges_or_rebases() {
        let mut f = facts(Some((2, 1)));
        f.remote_unmatched = Some(1);
        let c = class(f);
        let s = suggest(&c, &ctx());
        assert_eq!(s.action, A::MergeFrom);
        assert_eq!(
            s.allowed,
            vec![A::MergeFrom, A::RebaseOnto, A::ForceWithLease, A::Skip]
        );
        let rebase = SuggestContext {
            prefer_rebase: true,
            ..ctx()
        };
        let s = suggest(&c, &rebase);
        assert_eq!(s.action, A::RebaseOnto);
        assert_eq!(s.allowed[1], A::MergeFrom);
        assert_eq!(s.allowed.len(), 4);
        // Another target already holds the local commits: merge, and
        // rebase stays offered.
        let held = SuggestContext {
            prefer_rebase: true,
            others_already_have_local: true,
            ..ctx()
        };
        let s = suggest(&c, &held);
        assert_eq!(s.action, A::MergeFrom);
        assert!(s.allowed.contains(&A::RebaseOnto));
    }

    #[test]
    fn table_unrelated_skips_but_offers_force() {
        let mut f = facts(Some((2, 2)));
        f.related = false;
        let s = suggest(&class(f), &ctx());
        assert_eq!(s.action, A::Skip);
        assert_eq!(s.allowed, vec![A::Skip, A::ForceWithLease]);
    }

    #[test]
    fn table_refused_rows_skip_and_never_offer_force() {
        let refused = SuggestContext {
            refused: true,
            ..ctx()
        };
        for ab in [Some((0, 0)), Some((2, 2)), Some((0, 1)), None] {
            let mut f = facts(ab);
            f.remote_unmatched = Some(0);
            let s = suggest(&class(f), &refused);
            assert_eq!(s.action, A::Skip);
            assert_eq!(s.allowed, vec![A::Skip]);
        }
    }

    #[test]
    fn pull_mode_follows_the_pull_mode_and_never_forces() {
        let pull = SuggestContext {
            mode: GroupMode::Pull,
            ..ctx()
        };
        assert_eq!(
            suggest(&class(facts(Some((0, 2)))), &pull).action,
            A::FastForwardFrom
        );
        assert_eq!(suggest(&class(facts(Some((2, 0)))), &pull).action, A::Skip);
        assert_eq!(suggest(&class(facts(Some((0, 0)))), &pull).action, A::Skip);
        assert_eq!(suggest(&class(facts(None)), &pull).action, A::Skip);
        let d = class(facts(Some((1, 1))));
        assert_eq!(suggest(&d, &pull).action, A::MergeFrom);
        let rebase = SuggestContext {
            prefer_rebase: true,
            ..pull
        };
        let s = suggest(&d, &rebase);
        assert_eq!(s.action, A::RebaseOnto);
        assert!(!s.allowed.contains(&A::ForceWithLease));
    }

    #[test]
    fn holds_local_covers_equal_ahead_and_rebased_copies() {
        let c = |a, b| class(facts(Some((a, b))));
        assert!(c(0, 0).holds_local(None));
        assert!(c(0, 2).holds_local(None));
        assert!(!c(2, 0).holds_local(None));
        assert!(c(2, 2).holds_local(Some(0)));
        assert!(!c(2, 2).holds_local(Some(1)));
        assert!(!class(facts(None)).holds_local(None));
    }

    #[test]
    fn url_normalisation_detects_twins() {
        let same = [
            "https://github.com/o/r.git",
            "https://user:tok@GitHub.com/o/r",
            "git@github.com:o/r.git",
            "ssh://git@github.com:22/o/r.git/",
        ];
        let first = normalize_url(same[0]);
        for u in same {
            assert_eq!(normalize_url(u), first, "{u}");
        }
        assert_ne!(normalize_url("https://github.com/o/R"), first);
        assert_ne!(normalize_url("https://gitlab.com/o/r.git"), first);
        assert_eq!(normalize_url("/srv/git/a.git"), "/srv/git/a");
        assert_eq!(normalize_url("file:///srv/git/a.git/"), "/srv/git/a");
        assert_eq!(normalize_url("C:/x/a.git"), "C:/x/a");
    }
}
