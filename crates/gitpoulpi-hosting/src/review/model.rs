//! Host-agnostic shapes of an in-app code review.
//!
//! # How a position maps onto the diff
//!
//! The diff is not fetched from the host: GitPoulpi fetches the request's
//! head into `refs/gitpoulpi/pr/<n>` and diffs `merge-base(base, head)` to
//! that head with its own diff code. Every comment position below is
//! expressed in that diff's terms:
//!
//! * [`DiffSide::New`] and `line`: the 1-based line number in the file as
//!   it is at the head commit (the right-hand column of the diff);
//! * [`DiffSide::Old`] and `line`: the 1-based line number in the file as
//!   it is at the merge base (the left-hand column);
//! * an unchanged context line exists on both sides; either number
//!   addresses it (GitHub accepts the new side, GitLab wants both - see
//!   [`PendingComment::other_line`]);
//! * a thread whose code moved on since it was written is `outdated`: its
//!   `line` is `None` (nothing to draw inline) and `original_line` tells
//!   where it was, for a list under the file.

use serde::{Deserialize, Serialize};

use crate::model::{ChecksState, PullRequestSummary};

/// Which column of the diff a line number belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffSide {
    /// The file before the change (the merge base): deleted and context
    /// lines, numbered as in the old file.
    Old,
    /// The file after the change (the request's head): added and context
    /// lines, numbered as in the new file.
    #[default]
    New,
}

impl DiffSide {
    /// `old` / `new`, the spelling the CLI takes and prints.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Old => "old",
            Self::New => "new",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "old" | "left" | "base" => Some(Self::Old),
            "new" | "right" | "head" => Some(Self::New),
            _ => None,
        }
    }
}

/// What the host says about whether the request can be merged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeState {
    /// Nothing blocks it as far as the host says.
    Clean,
    /// The branches conflict.
    Conflicting,
    /// Mergeable in principle, but something else blocks it (required
    /// reviews, failing checks, a draft).
    Blocked,
    /// The host has not computed it yet, or does not say.
    #[default]
    Unknown,
}

/// One pull/merge request, read for review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequestDetail {
    #[serde(flatten)]
    pub summary: PullRequestSummary,
    /// The description, markdown as the host stores it.
    pub body: String,
    /// The head commit the review is about. Pass it back in
    /// [`ReviewSubmission::head_sha`] so a comment cannot land on a newer
    /// push than the one the reviewer read.
    pub head_sha: String,
    /// The base branch tip the host last compared against (the diff's
    /// real base is the merge base, which git computes locally).
    pub base_sha: String,
    pub merge_state: MergeState,
    pub checks: Option<ChecksState>,
    pub created_at: String,
    /// `None` when the host does not say.
    pub additions: Option<u32>,
    pub deletions: Option<u32>,
    pub changed_files: Option<u32>,
    /// What this client lets the reviewer do (so a UI can hide the rest).
    pub capabilities: ReviewCapabilities,
    /// The numbered patch sets of the request, oldest first (Gerrit; empty
    /// elsewhere). `head_sha` is the one of [`Self::shown_patch_set`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patch_sets: Vec<PatchSet>,
    /// The patch set this detail (and its files and threads) describes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shown_patch_set: Option<u32>,
    /// The labels the user may vote on, each with the values the server
    /// permits for this request (Gerrit; empty elsewhere).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vote_labels: Vec<VoteLabel>,
    /// The server says this request can be submitted (merged) right now
    /// (Gerrit; `false` elsewhere, where merging is not a review action).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub can_submit: bool,
    /// The user may abandon this request (Gerrit; `false` elsewhere).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub can_abandon: bool,
}

/// One patch set (revision) of a Gerrit change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchSet {
    pub number: u32,
    /// The commit of the patch set (what a review is sent on).
    pub sha: String,
    pub created_at: String,
}

/// One value a label can be voted: `-2`, `0`, `+1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteValue {
    pub value: i32,
    /// The server's words for it ("Looks good to me, approved").
    pub text: String,
}

/// A label the user may vote on (`Code-Review`, `Verified`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteLabel {
    pub name: String,
    /// Only the values the server permits for this request and user,
    /// lowest first.
    pub values: Vec<VoteValue>,
    /// The user's vote now.
    pub current: i32,
    /// What no vote reads as (`0`).
    pub default: i32,
}

/// A vote sent with a review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewVote {
    pub label: String,
    pub value: i32,
}

/// What a host and its edition let a review do. A choice that is `false`
/// is not offered: the UI hides it, and asking anyway is
/// [`crate::Error::Unsupported`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ReviewCapabilities {
    /// Any of the review operations at all.
    pub review: bool,
    pub approve: bool,
    /// GitHub has it; GitLab has no equivalent through its REST API.
    pub request_changes: bool,
    /// A comment covering several lines (`start_line`..`line`).
    pub line_ranges: bool,
    /// Threads can be resolved and reopened.
    pub resolve: bool,
}

impl ReviewCapabilities {
    /// Whether [`ReviewEvent`] `event` may be submitted.
    pub fn allows(&self, event: ReviewEvent) -> bool {
        self.review
            && match event {
                ReviewEvent::Comment => true,
                ReviewEvent::Approve => self.approve,
                ReviewEvent::RequestChanges => self.request_changes,
            }
    }
}

/// How a file changed in the request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Added,
    Modified,
    Removed,
    Renamed,
}

/// One changed file of the request, for the file list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewFile {
    /// The path at the head (the old path for a removed file).
    pub path: String,
    /// The path at the merge base, for a rename.
    pub previous_path: Option<String>,
    pub status: FileStatus,
    pub additions: u32,
    pub deletions: u32,
}

/// One comment: on a thread, or in the conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewComment {
    /// The host's id, as a string (a number on both hosts).
    pub id: String,
    pub author: String,
    /// Markdown, as written.
    pub body: String,
    pub created_at: String,
    /// The comment this one answers (conversation comments only; a
    /// thread's replies are ordered in `comments`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_url: Option<String>,
}

/// The verdict of a submitted review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewState {
    Approved,
    ChangesRequested,
    Commented,
    /// A verdict the author or a maintainer dismissed.
    Dismissed,
}

/// A review somebody submitted: the verdict and its summary (the line
/// comments are in the threads). Listed in the conversation next to the
/// plain comments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmittedReview {
    pub id: String,
    pub author: String,
    pub state: ReviewState,
    /// The summary; may be empty (an approval without words).
    pub body: String,
    /// Empty when the host does not date it (GitLab approvals).
    pub submitted_at: String,
}

/// A conversation attached to a place in the code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewThread {
    /// What `reply_to_thread` / `resolve_thread` take: GitHub's GraphQL
    /// node id, GitLab's discussion id. Opaque.
    pub id: String,
    /// The path at the head (at the merge base for a removed file).
    pub path: String,
    /// The line, on `side`, the thread sits on now; `None` once it is
    /// outdated (the code it was written against changed).
    pub line: Option<u32>,
    /// Where it was when written, kept for outdated threads.
    pub original_line: Option<u32>,
    pub side: DiffSide,
    /// The first line of a multi-line comment, on `side`.
    pub start_line: Option<u32>,
    pub resolved: bool,
    pub outdated: bool,
    /// Whether it can be resolved at all (a GitLab thread may not be).
    pub resolvable: bool,
    /// Oldest first; the first is the thread's opening comment.
    pub comments: Vec<ReviewComment>,
}

impl ReviewThread {
    /// The diff line to draw the thread under, or `None` when it has no
    /// current line (outdated).
    pub fn anchor(&self) -> Option<(DiffSide, u32)> {
        self.line.map(|line| (self.side, line))
    }
}

/// How a review concludes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewEvent {
    /// Just the comments, no verdict.
    Comment,
    Approve,
    RequestChanges,
}

impl ReviewEvent {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Comment => "comment",
            Self::Approve => "approve",
            Self::RequestChanges => "request-changes",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().replace('_', "-").as_str() {
            "comment" => Some(Self::Comment),
            "approve" => Some(Self::Approve),
            "request-changes" => Some(Self::RequestChanges),
            _ => None,
        }
    }
}

/// A comment on code that is held locally until the review is submitted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingComment {
    /// The path at the head.
    pub path: String,
    /// The path at the merge base, when the file was renamed (GitLab
    /// wants both; defaults to `path`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_path: Option<String>,
    /// The (last) line commented, on `side`.
    pub line: u32,
    pub side: DiffSide,
    /// The first line of a range, on `side`; `None` for one line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    /// For an unchanged context line, its number on the other side.
    /// GitLab needs both numbers to place such a comment; GitHub ignores
    /// it. The diff view knows both.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub other_line: Option<u32>,
    pub body: String,
}

/// Everything a review submission sends: one host round-trip set, however
/// many comments were collected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewSubmission {
    pub event: ReviewEvent,
    /// The review's summary; may be empty for an approval or when
    /// comments carry the message.
    pub body: String,
    pub comments: Vec<PendingComment>,
    /// The head the comments were written against; the host refuses (or
    /// marks them outdated) when the request moved. `None`: whatever is
    /// current.
    #[serde(default)]
    pub head_sha: Option<String>,
    /// Votes on labels (Gerrit's `Code-Review`, `Verified`); the event is
    /// ignored for the labels voted this way.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub votes: Vec<ReviewVote>,
}

/// What a submitted review reports back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewOutcome {
    pub event: ReviewEvent,
    /// How many line comments were posted.
    pub comments: usize,
    /// The host's id of the review when it has one (GitHub).
    pub review_id: Option<String>,
}
