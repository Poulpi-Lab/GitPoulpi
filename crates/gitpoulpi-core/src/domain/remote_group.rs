//! Several remotes for one branch: the extra push targets and opt-in pull
//! sources GitPoulpi keeps beside git's own upstream.
//!
//! They live in the branch's own config section, next to
//! `branch.<b>.remote` / `branch.<b>.merge` (the *primary*, plain git
//! semantics, never touched here):
//!
//! ```text
//! [branch "feature"]
//!     gitpoulpi-push-to = gitlab         # same-named branch
//!     gitpoulpi-push-to = backup:feat-x  # another branch name
//!     gitpoulpi-pull-from = gitlab
//! ```
//!
//! A value is `<remote>` or `<remote>:<branch>`. `:` cannot appear in a
//! ref name, hence not in a remote name, while `/` can (`grp/sub`).
//! Both variables are multi-valued; file order is the order of use.
//!
//! Pure: no I/O. The backends read and write the entries, this module
//! only interprets and rewrites them.

use serde::{Deserialize, Serialize};

/// The variable listing extra push targets (lowercase: `config --list`
/// lowercases variable names anyway).
pub const PUSH_TO_VARIABLE: &str = "gitpoulpi-push-to";
/// The variable listing remotes a plain Pull also integrates.
pub const PULL_FROM_VARIABLE: &str = "gitpoulpi-pull-from";

/// A branch on a remote.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TargetRef {
    pub remote: String,
    pub branch: String,
}

impl TargetRef {
    /// Parses a config value; `default_branch` stands in for the
    /// same-named form (`<remote>` and `<remote>:`). `None` when the
    /// remote part is empty.
    pub fn parse(value: &str, default_branch: &str) -> Option<TargetRef> {
        let value = value.trim();
        let (remote, branch) = match value.split_once(':') {
            Some((r, b)) => (r.trim(), b.trim()),
            None => (value, ""),
        };
        if remote.is_empty() {
            return None;
        }
        Some(TargetRef {
            remote: remote.to_string(),
            branch: if branch.is_empty() {
                default_branch.to_string()
            } else {
                branch.to_string()
            },
        })
    }

    /// The config value: `<remote>` when the branch is `default_branch`,
    /// else `<remote>:<branch>`.
    pub fn to_value(&self, default_branch: &str) -> String {
        if self.branch == default_branch || self.branch.is_empty() {
            self.remote.clone()
        } else {
            format!("{}:{}", self.remote, self.branch)
        }
    }
}

/// Everything GitPoulpi knows about a branch's remotes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchRemotes {
    pub branch: String,
    /// Git's own upstream (`branch.<b>.remote` + `merge`), if any.
    pub primary: Option<TargetRef>,
    /// Extra push targets, after the primary, in order.
    #[serde(default)]
    pub push_to: Vec<TargetRef>,
    /// Extra pull sources, in order.
    #[serde(default)]
    pub pull_from: Vec<TargetRef>,
    /// Remote names used by the keys that no configured remote has (a
    /// rename or removal done with plain git). Filled by
    /// [`BranchRemotes::note_missing`]; never an error.
    #[serde(default)]
    pub missing: Vec<String>,
}

impl BranchRemotes {
    /// Fills `missing` against the repository's remote names.
    pub fn note_missing(&mut self, known: &[String]) {
        let mut missing: Vec<String> = Vec::new();
        for t in self.push_to.iter().chain(self.pull_from.iter()) {
            if !known.contains(&t.remote) && !missing.contains(&t.remote) {
                missing.push(t.remote.clone());
            }
        }
        self.missing = missing;
    }

    /// Whether the branch has any extra push target or pull source.
    pub fn is_multi(&self) -> bool {
        !self.push_to.is_empty() || !self.pull_from.is_empty()
    }
}

/// Whether `key` (`branch.<name>.<variable>`) is one of ours, and the
/// branch it belongs to.
fn variable_of(key: &str) -> Option<(&str, &'static str)> {
    let rest = key.strip_prefix("branch.")?;
    let (branch, variable) = rest.rsplit_once('.')?;
    if branch.is_empty() {
        return None;
    }
    if variable.eq_ignore_ascii_case(PUSH_TO_VARIABLE) {
        Some((branch, PUSH_TO_VARIABLE))
    } else if variable.eq_ignore_ascii_case(PULL_FROM_VARIABLE) {
        Some((branch, PULL_FROM_VARIABLE))
    } else {
        None
    }
}

/// Whether `key` is one of the two variables, on any branch.
pub fn is_remote_group_key(key: &str) -> bool {
    variable_of(key).is_some()
}

/// The remote part of a stored value (`origin:topic` gives `origin`).
pub fn value_remote(value: &str) -> &str {
    value.split_once(':').map_or(value, |(r, _)| r).trim()
}

/// Reads `branch`'s remotes out of config `entries` (`(key, value)`
/// pairs with full keys, in file order; the whole config or just the
/// branch section). Empty values are skipped, repeats collapse, and a
/// target equal to the primary is dropped.
pub fn parse_branch_remotes(entries: &[(String, String)], branch: &str) -> BranchRemotes {
    let own = |key: &str| variable_of(key).is_some_and(|(b, _)| b == branch);
    let value_of = |variable: &str| {
        entries
            .iter()
            .find(|(k, _)| {
                k.strip_prefix("branch.")
                    .and_then(|r| r.strip_prefix(branch))
                    .and_then(|r| r.strip_prefix('.'))
                    .is_some_and(|v| v.eq_ignore_ascii_case(variable))
            })
            .map(|(_, v)| v.trim().to_string())
    };
    let primary = value_of("remote")
        .filter(|r| !r.is_empty() && r != ".")
        .map(|remote| TargetRef {
            remote,
            branch: value_of("merge")
                .map(|m| m.strip_prefix("refs/heads/").unwrap_or(&m).to_string())
                .filter(|b| !b.is_empty())
                .unwrap_or_else(|| branch.to_string()),
        });
    let mut out = BranchRemotes {
        branch: branch.to_string(),
        primary,
        ..BranchRemotes::default()
    };
    for (key, value) in entries.iter().filter(|(k, _)| own(k)) {
        let Some(target) = TargetRef::parse(value, branch) else {
            continue;
        };
        if out.primary.as_ref() == Some(&target) {
            continue;
        }
        let list = match variable_of(key) {
            Some((_, PUSH_TO_VARIABLE)) => &mut out.push_to,
            _ => &mut out.pull_from,
        };
        if !list.contains(&target) {
            list.push(target);
        }
    }
    out
}

/// `section_entries` with this module's variables replaced by those of
/// `remotes`: every other entry (upstream included) stays as it is, in
/// order; the push targets then the pull sources follow, deduplicated,
/// with the primary itself left out. Fit for `set_config_section`.
pub fn entries_with_remotes(
    section_entries: &[(String, String)],
    remotes: &BranchRemotes,
) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = section_entries
        .iter()
        .filter(|(k, _)| !is_remote_group_key(k))
        .cloned()
        .collect();
    let branch = &remotes.branch;
    for (variable, targets) in [
        (PUSH_TO_VARIABLE, &remotes.push_to),
        (PULL_FROM_VARIABLE, &remotes.pull_from),
    ] {
        let mut seen: Vec<&TargetRef> = Vec::new();
        for t in targets {
            if t.remote.is_empty() || remotes.primary.as_ref() == Some(t) || seen.contains(&t) {
                continue;
            }
            seen.push(t);
            out.push((format!("branch.{branch}.{variable}"), t.to_value(branch)));
        }
    }
    out
}

/// `entries` with every value naming remote `old` now naming `new`
/// (a `:<branch>` part is kept).
pub fn rename_remote_in(
    entries: &[(String, String)],
    old: &str,
    new: &str,
) -> Vec<(String, String)> {
    entries
        .iter()
        .map(|(k, v)| {
            if is_remote_group_key(k) && value_remote(v) == old {
                let value = match v.split_once(':') {
                    Some((_, b)) => format!("{new}:{b}"),
                    None => new.to_string(),
                };
                (k.clone(), value)
            } else {
                (k.clone(), v.clone())
            }
        })
        .collect()
}

/// `entries` without the values naming remote `name`.
pub fn drop_remote_in(entries: &[(String, String)], name: &str) -> Vec<(String, String)> {
    entries
        .iter()
        .filter(|(k, v)| !(is_remote_group_key(k) && value_remote(v) == name))
        .cloned()
        .collect()
}

/// How a branch stands against one remote's copy of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteRelation {
    UpToDate,
    /// Local has `n` commits the remote lacks.
    LocalAhead(u32),
    /// The remote has `n` commits local lacks.
    RemoteAhead(u32),
    Diverged {
        ahead: u32,
        behind: u32,
    },
    /// No remote-tracking ref for the branch (never pushed, or not fetched).
    Absent,
    /// Both exist but share no history.
    Unrelated,
}

/// The part a remote plays for a branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetRole {
    Primary,
    PushTo,
    PullFrom,
}

/// One remote of a branch with where it stands. No network: computed from
/// the remote-tracking refs as they are.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTargetState {
    pub remote: String,
    /// The branch name on that remote.
    pub branch: String,
    pub role: TargetRole,
    /// Also a pull source (a push target that is listed in both).
    #[serde(default)]
    pub pull: bool,
    /// The remote-tracking ref read (`gitlab/feature`), when it exists.
    #[serde(default)]
    pub tracking: Option<String>,
    pub relation: RemoteRelation,
    /// The remote is no longer configured.
    #[serde(default)]
    pub missing: bool,
}

/// Answer of the `RemoteGroupStatus` read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteGroupStatus {
    pub remotes: BranchRemotes,
    /// Primary first, then the push targets, then pull-only sources.
    pub targets: Vec<RemoteTargetState>,
    /// The saved grouped run of this worktree, when one waits (stopped on
    /// conflicts, canceled, or with targets to retry).
    #[serde(default)]
    pub pending: Option<super::PendingGroup>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(k: &str, v: &str) -> (String, String) {
        (k.to_string(), v.to_string())
    }

    fn sample() -> Vec<(String, String)> {
        vec![
            e("branch.feat.remote", "origin"),
            e("branch.feat.merge", "refs/heads/feat"),
            e("branch.feat.gitpoulpi-push-to", "gitlab"),
            e("branch.feat.gitpoulpi-push-to", "backup:x"),
            e("branch.feat.gitpoulpi-pull-from", "gitlab"),
            e("branch.other.gitpoulpi-push-to", "nope"),
        ]
    }

    #[test]
    fn parses_primary_targets_and_sources_in_order() {
        let r = parse_branch_remotes(&sample(), "feat");
        assert_eq!(
            r.primary,
            Some(TargetRef {
                remote: "origin".into(),
                branch: "feat".into()
            })
        );
        assert_eq!(r.push_to.len(), 2);
        assert_eq!(r.push_to[0].remote, "gitlab");
        assert_eq!(r.push_to[0].branch, "feat");
        assert_eq!(r.push_to[1].remote, "backup");
        assert_eq!(r.push_to[1].branch, "x");
        assert_eq!(r.pull_from.len(), 1);
    }

    #[test]
    fn remote_names_with_a_slash_and_empty_values() {
        let entries = vec![
            e("branch.a.b.gitpoulpi-push-to", "grp/sub"),
            e("branch.a.b.gitpoulpi-push-to", ""),
            e("branch.a.b.gitpoulpi-push-to", ":x"),
            e("branch.a.b.gitpoulpi-push-to", "grp/sub:topic/1"),
            e("branch.a.b.gitpoulpi-push-to", "grp/sub"),
        ];
        let r = parse_branch_remotes(&entries, "a.b");
        assert_eq!(r.push_to.len(), 2, "{r:?}");
        assert_eq!(r.push_to[0].remote, "grp/sub");
        assert_eq!(r.push_to[0].branch, "a.b");
        assert_eq!(r.push_to[1].branch, "topic/1");
        // `a` is a different branch than `a.b`.
        assert!(parse_branch_remotes(&entries, "a").push_to.is_empty());
        assert_eq!(TargetRef::parse("r:", "m").unwrap().branch, "m");
        assert!(TargetRef::parse("", "m").is_none());
    }

    #[test]
    fn the_primary_is_not_repeated() {
        let mut entries = sample();
        entries.push(e("branch.feat.gitpoulpi-push-to", "origin"));
        entries.push(e("branch.feat.gitpoulpi-pull-from", "origin:feat"));
        let r = parse_branch_remotes(&entries, "feat");
        assert_eq!(r.push_to.len(), 2);
        assert_eq!(r.pull_from.len(), 1);
    }

    #[test]
    fn writing_keeps_other_entries_and_order() {
        let section: Vec<_> = sample()
            .into_iter()
            .filter(|(k, _)| k.starts_with("branch.feat."))
            .collect();
        let mut r = parse_branch_remotes(&section, "feat");
        r.push_to.push(TargetRef {
            remote: "origin".into(),
            branch: "feat".into(),
        });
        r.push_to.push(TargetRef {
            remote: "mirror".into(),
            branch: "feat".into(),
        });
        r.pull_from.clear();
        let out = entries_with_remotes(&section, &r);
        assert_eq!(out[0], e("branch.feat.remote", "origin"));
        assert_eq!(out[1], e("branch.feat.merge", "refs/heads/feat"));
        assert_eq!(out[2], e("branch.feat.gitpoulpi-push-to", "gitlab"));
        assert_eq!(out[3], e("branch.feat.gitpoulpi-push-to", "backup:x"));
        assert_eq!(out[4], e("branch.feat.gitpoulpi-push-to", "mirror"));
        assert_eq!(out.len(), 5);
    }

    #[test]
    fn rename_and_drop() {
        let renamed = rename_remote_in(&sample(), "backup", "bk/up");
        assert!(renamed.contains(&e("branch.feat.gitpoulpi-push-to", "bk/up:x")));
        let renamed = rename_remote_in(&sample(), "gitlab", "gl");
        assert!(renamed.contains(&e("branch.feat.gitpoulpi-push-to", "gl")));
        assert!(renamed.contains(&e("branch.feat.gitpoulpi-pull-from", "gl")));
        assert!(renamed.contains(&e("branch.feat.remote", "origin")));
        let dropped = drop_remote_in(&sample(), "gitlab");
        assert_eq!(dropped.len(), sample().len() - 2);
        // A branch.remote of the same name is git's to handle, not ours.
        let own = vec![e("branch.x.remote", "gitlab")];
        assert_eq!(drop_remote_in(&own, "gitlab"), own);
    }

    #[test]
    fn missing_remotes_are_reported_not_fatal() {
        let mut r = parse_branch_remotes(&sample(), "feat");
        r.note_missing(&["origin".into(), "gitlab".into()]);
        assert_eq!(r.missing, vec!["backup".to_string()]);
    }
}
