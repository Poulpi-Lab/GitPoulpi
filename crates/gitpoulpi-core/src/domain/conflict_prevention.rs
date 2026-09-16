//! Proactive conflict detection: which branch the current one started from,
//! and what a merge into it would conflict on (plan `repo-settings.md`,
//! slice 3).
//!
//! The pure parts live here (the "Base branches" globs and the choice of the
//! base among the allowed branches); the merge itself is
//! `GitBackend::predict_merge_conflicts` and the daemon schedules it.

use serde::{Deserialize, Serialize};

/// The default "Base branches" list: every branch.
pub const DEFAULT_BASE_GLOBS: &str = "**";

/// More candidates than this are not measured (sorted by name, the rest are
/// dropped): each one costs a graph walk, and a repository with hundreds of
/// remote branches would otherwise make every check slow.
pub const MAX_BASE_CANDIDATES: usize = 300;

/// The "Base branches" field's lines as a list: blank lines and `#` comments
/// dropped, an empty result meaning the default (`**`).
pub fn parse_base_globs(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// Whether the branch `name` (`main` for a local branch, `origin/main` for a
/// remote-tracking one) is allowed by the patterns.
///
/// - a pattern is `/`-separated; `**` as a whole segment matches any number
///   of segments (zero included), `*` and `?` match inside one segment;
/// - a leading `!` excludes what the rest matches, and wins over every
///   positive pattern;
/// - with no positive pattern (an empty list, or only exclusions) every
///   branch is allowed except the excluded ones.
pub fn base_allowed(patterns: &[String], name: &str) -> bool {
    let mut positive = false;
    let mut allowed = false;
    for pattern in patterns {
        let pattern = pattern.trim();
        if let Some(excluded) = pattern.strip_prefix('!') {
            if glob_matches(excluded.trim(), name) {
                return false;
            }
        } else if !pattern.is_empty() {
            positive = true;
            allowed |= glob_matches(pattern, name);
        }
    }
    allowed || !positive
}

/// `pattern` against `name`, segment by segment (see [`base_allowed`]).
pub fn glob_matches(pattern: &str, name: &str) -> bool {
    let pat: Vec<&str> = pattern.split('/').collect();
    let nam: Vec<&str> = name.split('/').collect();
    segments_match(&pat, &nam)
}

fn segments_match(pat: &[&str], nam: &[&str]) -> bool {
    match pat.split_first() {
        None => nam.is_empty(),
        Some((&"**", rest)) => (0..=nam.len()).any(|skip| segments_match(rest, &nam[skip..])),
        Some((seg, rest)) => match nam.split_first() {
            Some((first, tail)) => segment_matches(seg, first) && segments_match(rest, tail),
            None => false,
        },
    }
}

/// One path segment against `*` / `?` wildcards.
fn segment_matches(pattern: &str, text: &str) -> bool {
    fn go(p: &[char], t: &[char]) -> bool {
        match p.split_first() {
            None => t.is_empty(),
            Some(('*', rest)) => (0..=t.len()).any(|skip| go(rest, &t[skip..])),
            Some(('?', rest)) => !t.is_empty() && go(rest, &t[1..]),
            Some((c, rest)) => t.first() == Some(c) && go(rest, &t[1..]),
        }
    }
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    go(&p, &t)
}

/// A branch the current one may have started from, with its distance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseCandidate {
    /// `main` or `origin/main`.
    pub name: String,
    /// A remote-tracking branch.
    pub remote: bool,
    /// Commits of HEAD not in the candidate: the commits made since the fork.
    pub ahead: u32,
    /// Commits of the candidate not in HEAD.
    pub behind: u32,
}

/// The base among the candidates: the one the current branch forked from
/// most recently, that is the fewest commits since the fork (`ahead`).
///
/// Ties (a feature branch has the same `ahead` against `main` and against a
/// `develop` that contains `main`) go to the candidate whose tip is nearest
/// the fork (fewest `behind`), then to a remote-tracking branch over a local
/// one (what is published is what a merge request targets), then to the
/// name, so the answer never depends on the order refs were listed in.
pub fn pick_base(candidates: &[BaseCandidate]) -> Option<&BaseCandidate> {
    candidates
        .iter()
        .min_by_key(|c| (c.ahead, c.behind, !c.remote, c.name.as_str()))
}

/// The answer of a conflict check.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictPrediction {
    pub status: PredictionStatus,
    /// Why the check did not run (`status` is `disabled`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// HEAD's commit, 40 hex digits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head: Option<String>,
    /// The base branch as named (`origin/main`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_ref: Option<String>,
    /// The base's commit, 40 hex digits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    /// Where the base came from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_source: Option<BaseSource>,
    /// The paths a merge would conflict on, sorted.
    #[serde(default)]
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PredictionStatus {
    /// Detection is off (globally, for this repository, or git is too old).
    #[default]
    Disabled,
    /// No branch the globs allow, or HEAD has no commit yet.
    NoBase,
    /// A merge would go through.
    Clean,
    /// A merge would conflict on `files`.
    Conflicts,
    /// A newer request for the same repository took over while this one
    /// waited: nothing was computed, ask again if still wanted.
    Superseded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BaseSource {
    /// Picked among the branches the globs allow.
    Globs,
    /// The target branch of the branch's known pull request.
    PullRequest,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pats(lines: &[&str]) -> Vec<String> {
        lines.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn the_pattern_table_examples() {
        let all = pats(&["**"]);
        for name in ["main", "origin/main", "origin/feature/x"] {
            assert!(base_allowed(&all, name), "{name}");
        }
        let one = pats(&["origin/main"]);
        assert!(base_allowed(&one, "origin/main"));
        assert!(!base_allowed(&one, "origin/main2"));
        assert!(!base_allowed(&one, "main"));
        assert!(!base_allowed(&one, "origin/feature/x"));

        let remote = pats(&["origin/**"]);
        assert!(base_allowed(&remote, "origin/main"));
        assert!(base_allowed(&remote, "origin/feature/deep/x"));
        assert!(!base_allowed(&remote, "upstream/main"));
        assert!(!base_allowed(&remote, "main"));

        let feature = pats(&["origin/feature/**"]);
        assert!(base_allowed(&feature, "origin/feature/login"));
        assert!(base_allowed(&feature, "origin/feature/a/b"));
        assert!(!base_allowed(&feature, "origin/main"));
        assert!(!base_allowed(&feature, "origin/features/x"));
    }

    #[test]
    fn an_exclusion_wins_and_alone_it_keeps_everything_else() {
        let p = pats(&["origin/**", "!origin/abc"]);
        assert!(base_allowed(&p, "origin/main"));
        assert!(!base_allowed(&p, "origin/abc"));
        assert!(base_allowed(&p, "origin/abcd"));
        let only_not = pats(&["!origin/abc"]);
        assert!(base_allowed(&only_not, "main"));
        assert!(!base_allowed(&only_not, "origin/abc"));
        // The exclusion also beats a later positive pattern.
        let late = pats(&["!origin/abc", "**"]);
        assert!(!base_allowed(&late, "origin/abc"));
        assert!(base_allowed(&late, "origin/other"));
    }

    #[test]
    fn single_star_and_question_stay_inside_a_segment() {
        let p = pats(&["origin/release-*"]);
        assert!(base_allowed(&p, "origin/release-1.2"));
        assert!(!base_allowed(&p, "origin/release-1/2"));
        let q = pats(&["origin/v?"]);
        assert!(base_allowed(&q, "origin/v1"));
        assert!(!base_allowed(&q, "origin/v10"));
        // `**` in the middle matches zero segments too.
        let mid = pats(&["origin/**/x"]);
        assert!(base_allowed(&mid, "origin/x"));
        assert!(base_allowed(&mid, "origin/a/b/x"));
    }

    #[test]
    fn an_empty_list_is_the_default_and_comments_are_dropped() {
        assert!(base_allowed(&[], "anything/at/all"));
        let parsed = parse_base_globs("# note\n\n  origin/main \n!origin/abc\n");
        assert_eq!(parsed, pats(&["origin/main", "!origin/abc"]));
        assert!(parse_base_globs("  \n").is_empty());
    }

    fn cand(name: &str, remote: bool, ahead: u32, behind: u32) -> BaseCandidate {
        BaseCandidate {
            name: name.into(),
            remote,
            ahead,
            behind,
        }
    }

    #[test]
    fn the_base_is_the_fewest_commits_since_the_fork() {
        // Forked from develop (2 commits since), main is further back.
        let c = [
            cand("origin/main", true, 5, 1),
            cand("origin/develop", true, 2, 3),
        ];
        assert_eq!(pick_base(&c).unwrap().name, "origin/develop");
        assert!(pick_base(&[]).is_none());
    }

    #[test]
    fn ties_go_to_the_nearest_tip_then_remote_then_name() {
        // Same `ahead`: main is the one HEAD forked from, develop moved on.
        let c = [
            cand("origin/develop", true, 2, 9),
            cand("origin/main", true, 2, 1),
        ];
        assert_eq!(pick_base(&c).unwrap().name, "origin/main");
        // A local `main` and its remote at the same place: the remote.
        let c = [cand("main", false, 2, 1), cand("origin/main", true, 2, 1)];
        assert_eq!(pick_base(&c).unwrap().name, "origin/main");
        // Identical in every respect: the name decides, in any input order.
        let a = [cand("origin/b", true, 1, 1), cand("origin/a", true, 1, 1)];
        let b = [cand("origin/a", true, 1, 1), cand("origin/b", true, 1, 1)];
        assert_eq!(pick_base(&a).unwrap().name, "origin/a");
        assert_eq!(pick_base(&b).unwrap().name, "origin/a");
    }

    #[test]
    fn a_prediction_round_trips_as_json() {
        let p = ConflictPrediction {
            status: PredictionStatus::Conflicts,
            base_ref: Some("origin/main".into()),
            files: vec!["a.txt".into()],
            ..Default::default()
        };
        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(json["status"], "conflicts");
        let back: ConflictPrediction = serde_json::from_value(json).unwrap();
        assert_eq!(back, p);
    }
}
