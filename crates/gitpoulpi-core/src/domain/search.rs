//! The workspace search's vocabulary (the workspaces design):
//! what the commit-message read returns, what a search asks for and what a
//! hit carries. Pure data and the query prefix parser; the index that
//! answers lives in the daemon, the scorer in [`crate::fuzzy`].

use serde::{Deserialize, Serialize};

use super::Oid;

/// One commit's searchable text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitMessage {
    pub id: Oid,
    /// The author's name.
    pub author: String,
    /// The author's time, seconds since the epoch.
    pub time: i64,
    /// First paragraph with line breaks folded to spaces (git's `%s`).
    pub subject: String,
    /// Everything after the subject (git's `%b`), trailing newlines trimmed.
    pub body: String,
}

/// Splits a decoded commit message into git's subject (`%s`, the first
/// paragraph on one line) and body (`%b`, the rest, trimmed). One function for
/// both backends, so their answers cannot differ on an odd message.
pub fn split_message(message: &str) -> (String, String) {
    let normalized = message
        .trim_start_matches(['\n', '\r'])
        .replace("\r\n", "\n");
    let subject = super::commit_summary(&normalized);
    let body = match normalized.split_once("\n\n") {
        Some((_, rest)) => rest.trim().to_string(),
        None => String::new(),
    };
    (subject, body)
}

/// What `GitBackend::commit_messages` is asked.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitMessagesQuery {
    /// Tips a previous answer already covered (its [`CommitMessages::tips`]):
    /// commits reachable from them are not returned again. Empty = every
    /// commit.
    #[serde(default)]
    pub known_tips: Vec<Oid>,
    /// At most this many commits, newest first. `0` = no limit.
    #[serde(default)]
    pub limit: usize,
}

/// What `GitBackend::commit_messages` answers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitMessages {
    /// Commits reachable from the refs (branches, remote branches, tags and
    /// HEAD; never stashes or GitPoulpi's own refs) and not from
    /// `known_tips`, newest first.
    pub commits: Vec<CommitMessage>,
    /// The current tips (peeled commit ids, sorted, de-duplicated): what to
    /// pass as `known_tips` next time.
    pub tips: Vec<Oid>,
    /// True when a known tip is no longer reachable from any ref (a branch
    /// was reset, amended or deleted): the answer then holds *every* commit
    /// (`known_tips` was ignored) and replaces what the caller kept.
    pub replace: bool,
    /// True when `limit` cut the walk short.
    pub truncated: bool,
}

/// What a hit is. Serialized lower-case (`branch`, `commit`...) so a CLI flag
/// and the JSON spell it alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchKind {
    /// A repository of the scope (answered by the caller, not a daemon: a
    /// daemon index is per repository).
    Repo,
    /// A local or remote-tracking branch.
    Branch,
    Tag,
    Commit,
    Stash,
    Worktree,
    /// A pull or merge request, from the repository's hosting cache. Never
    /// answered by a daemon (it makes no network call and reads no cache).
    PullRequest,
    /// An issue, from the hosting cache. Same rule as `PullRequest`.
    Issue,
}

impl SearchKind {
    /// The kinds a daemon answers (git data).
    pub const ALL: [SearchKind; 6] = [
        SearchKind::Repo,
        SearchKind::Branch,
        SearchKind::Tag,
        SearchKind::Commit,
        SearchKind::Stash,
        SearchKind::Worktree,
    ];

    /// Every kind, the hosting ones last: what a window or the CLI offers.
    pub const EVERY: [SearchKind; 8] = [
        SearchKind::Repo,
        SearchKind::Branch,
        SearchKind::Tag,
        SearchKind::Commit,
        SearchKind::Stash,
        SearchKind::Worktree,
        SearchKind::PullRequest,
        SearchKind::Issue,
    ];

    /// Whether the kind comes from a hosting cache rather than from git.
    pub fn is_hosting(self) -> bool {
        matches!(self, Self::PullRequest | Self::Issue)
    }

    /// The one-letter query prefix (`b:`).
    pub fn prefix(self) -> char {
        match self {
            Self::Repo => 'r',
            Self::Branch => 'b',
            Self::Tag => 't',
            Self::Commit => 'c',
            Self::Stash => 's',
            Self::Worktree => 'w',
            Self::PullRequest => 'p',
            Self::Issue => 'i',
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Repo => "repo",
            Self::Branch => "branch",
            Self::Tag => "tag",
            Self::Commit => "commit",
            Self::Stash => "stash",
            Self::Worktree => "worktree",
            Self::PullRequest => "pull_request",
            Self::Issue => "issue",
        }
    }

    /// Parses `repo`, `branch`... (a CLI flag value), also the prefix letter.
    pub fn parse(word: &str) -> Option<Self> {
        let word = word.trim().to_lowercase();
        Self::EVERY.into_iter().find(|k| {
            k.name() == word
                || (word.len() == 1 && word.starts_with(k.prefix()))
                || (*k == Self::PullRequest && matches!(word.as_str(), "pr" | "mr"))
        })
    }
}

/// A query split into its kind prefix and the text to match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedQuery {
    /// The kinds the prefix narrowed to; `None` = the caller's own filter.
    pub kind: Option<SearchKind>,
    /// The text to match, trimmed (not lower-cased).
    pub text: String,
    /// `#` opened the query: pull/merge requests and issues, both.
    pub hosting_only: bool,
}

/// Splits a leading `b:` `t:` `c:` `s:` `w:` `r:` `p:` `i:` (also `pr:` and
/// `mr:`) off `query`; a leading `#` means "pull/merge requests and issues"
/// (`ParsedQuery::hosting_only`, the `#` dropped). Only that very start
/// counts (`fix: login` is a text, not a prefix); blanks around the text are
/// dropped.
pub fn parse_query(query: &str) -> ParsedQuery {
    let q = query.trim_start();
    if let Some(rest) = q.strip_prefix('#') {
        return ParsedQuery {
            kind: None,
            text: rest.trim().to_string(),
            hosting_only: true,
        };
    }
    if q.is_char_boundary(3)
        && (q[..q.len().min(3)].eq_ignore_ascii_case("pr:")
            || q[..q.len().min(3)].eq_ignore_ascii_case("mr:"))
    {
        return ParsedQuery {
            kind: Some(SearchKind::PullRequest),
            text: q[3..].trim().to_string(),
            hosting_only: false,
        };
    }
    let mut chars = q.chars();
    if let (Some(letter), Some(':')) = (chars.next(), chars.next()) {
        if let Some(kind) = SearchKind::EVERY
            .into_iter()
            .find(|k| letter.to_ascii_lowercase() == k.prefix())
        {
            return ParsedQuery {
                kind: Some(kind),
                text: chars.as_str().trim().to_string(),
                hosting_only: false,
            };
        }
    }
    ParsedQuery {
        kind: None,
        text: q.trim_end().to_string(),
        hosting_only: false,
    }
}

/// What a client needs to go to a hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SearchTarget {
    /// A branch or tag: its full ref name (`refs/heads/main`).
    Ref {
        name: String,
    },
    Commit {
        id: Oid,
    },
    /// `stash@{index}` and the stash commit, so a moved stack is noticed.
    Stash {
        index: u32,
        id: Option<Oid>,
    },
    Worktree {
        path: String,
    },
    Repo {
        path: String,
    },
    /// A pull/merge request or an issue of the hosting cache, by number.
    PullRequest {
        number: u64,
    },
    Issue {
        number: u64,
    },
}

/// One search result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchHit {
    pub kind: SearchKind,
    /// The matched text as shown (a branch's short name, a commit's subject).
    pub label: String,
    /// What follows it: the short id and author of a commit, a worktree's
    /// branch, a branch's upstream.
    pub detail: String,
    pub target: SearchTarget,
    /// Higher is better; comparable across repositories.
    pub score: i32,
    /// Characters of `label` to highlight (indices, ascending). Empty when
    /// the match was in the commit body or the empty query.
    pub positions: Vec<u32>,
    /// The repository root, filled by the caller that merges repositories
    /// (the CLI); a daemon answers for one repository and leaves it unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
}

/// The `Search` IPC request's arguments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchRequest {
    /// The raw query, prefix included.
    pub query: String,
    /// Kinds to search; empty = all. A prefix in `query` narrows further.
    #[serde(default)]
    pub kinds: Vec<SearchKind>,
    /// At most this many hits (best first). `0` = the daemon's default (50).
    #[serde(default)]
    pub limit: usize,
    /// Wait for the commit index to be complete instead of answering with
    /// what is indexed so far (`SearchHits::partial`). A one-shot CLI wants
    /// the full answer; a window typing a query does not.
    #[serde(default)]
    pub wait: bool,
}

/// The `Search` IPC answer.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchHits {
    pub hits: Vec<SearchHit>,
    /// True while the commit index of this repository is still being built
    /// (or was cut at `search.commit_limit`): the hits may be missing
    /// commits. Ask again shortly.
    pub partial: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_letter_prefix_narrows_and_is_stripped() {
        for (q, kind, text) in [
            ("b: main", SearchKind::Branch, "main"),
            ("T:v1", SearchKind::Tag, "v1"),
            ("  c:  fix login ", SearchKind::Commit, "fix login"),
            ("s: wip", SearchKind::Stash, "wip"),
            ("w: ../x", SearchKind::Worktree, "../x"),
            ("r: api", SearchKind::Repo, "api"),
            ("pr: fix", SearchKind::PullRequest, "fix"),
            ("MR:fix", SearchKind::PullRequest, "fix"),
            ("p: x", SearchKind::PullRequest, "x"),
            ("i: 12", SearchKind::Issue, "12"),
        ] {
            let p = parse_query(q);
            assert_eq!((p.kind, p.text.as_str()), (Some(kind), text), "{q}");
        }
    }

    #[test]
    fn a_hash_means_hosting_only_and_is_stripped() {
        let p = parse_query("  # 12");
        assert!(p.hosting_only);
        assert_eq!((p.kind, p.text.as_str()), (None, "12"));
        assert!(!parse_query("12").hosting_only);
    }

    #[test]
    fn anything_else_is_plain_text() {
        for q in ["fix: login", "x: y", "main", "bb: x", ":x", "b", "b:"] {
            let p = parse_query(q);
            if q == "b:" {
                assert_eq!((p.kind, p.text.as_str()), (Some(SearchKind::Branch), ""));
            } else {
                assert_eq!(p.kind, None, "{q}");
                assert_eq!(p.text, q.trim());
            }
        }
    }

    #[test]
    fn a_message_splits_into_subject_and_body_like_git() {
        assert_eq!(split_message("fix it\n"), ("fix it".into(), String::new()));
        assert_eq!(
            split_message("fix it\n\nbecause\nreasons\n\n\nSigned-off-by: x\n"),
            (
                "fix it".into(),
                "because\nreasons\n\n\nSigned-off-by: x".into()
            )
        );
        // A wrapped subject folds onto one line, like `%s`.
        assert_eq!(
            split_message("a long\nsubject\n\nbody"),
            ("a long subject".into(), "body".into())
        );
        assert_eq!(
            split_message("\n\nlate start"),
            ("late start".into(), String::new())
        );
        assert_eq!(
            split_message("crlf\r\n\r\nbody\r\n"),
            ("crlf".into(), "body".into())
        );
        assert_eq!(split_message(""), (String::new(), String::new()));
    }

    #[test]
    fn kinds_round_trip_through_their_names_and_letters() {
        for kind in SearchKind::EVERY {
            assert_eq!(SearchKind::parse(kind.name()), Some(kind));
            assert_eq!(SearchKind::parse(&kind.prefix().to_string()), Some(kind));
        }
        assert_eq!(SearchKind::parse("nope"), None);
        assert_eq!(
            serde_json::to_string(&SearchKind::Worktree).unwrap(),
            "\"worktree\""
        );
    }
}
