//! The vocabulary the event loop and the Radar share: what an action
//! is, what the palette can ask, and what the event loop must do afterwards.
//! No hosting types, so the build without a provider compiles it too
//! (`radar_stub.rs`).

use std::path::PathBuf;

use super::pending::Pending;

/// The columns of the Radar's list that can be sized (`radar::Cell`'s sizable
/// ones; this file cannot name that type, the stub build has none).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SizedColumn {
    Repo,
    Branches,
    Num,
    Author,
    Age,
}

/// What a row's menu, a key or a palette command asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Act {
    OpenBrowser,
    OpenRepo,
    Checkout,
    Worktree,
    Show,
    StartWork,
    Pin,
    Snooze,
    Unsnooze,
    Link,
    Reconnect,
    /// Assignees, reviewers, labels, milestone: the metadata picker, in the
    /// repository's own window (GitHub, GitLab).
    EditAssignees,
    EditReviewers,
    EditLabels,
    SetMilestone,
    /// The in-app code review of a pull / merge request (GitHub, GitLab),
    /// opened in the repository's own window.
    Review,
    /// The same review, through the host's API (no clone), whatever the
    /// profile's review mode says.
    ReviewApi,
    /// The same review after cloning the repository (the Clone dialog opens
    /// prefilled).
    CloneReview,
    /// The conversation of an issue, read in the repository's own window.
    Conversation,
    /// Merges the request on the host (asks first, and how).
    Merge,
    /// Closes the request or issue on the host (asks first).
    CloseItem,
    /// Takes a draft request out of draft on the host.
    Ready,
    /// The Clone dialog on the row's repository (not cloned yet).
    CloneRepo,
    /// Turns on the account-wide review requests of a code host (the hint
    /// row's `[Turn on]`).
    TurnOn,
    /// Fits one sizable column to its content (the header menu's row).
    FitColumn(SizedColumn),
    /// Fits every sizable column to its content.
    FitAllColumns,
}

/// A palette command of the Radar (each is also a key or a click).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RadarCommand {
    Open,
    Refresh,
    Search,
    Act(Act),
    Scope,
    View,
    Filter,
    /// The `Not cloned` checkbox.
    NotCloned,
    /// The `Drafts` checkbox.
    Drafts,
    Menu,
    Back,
    /// Close the Radar's tab for this session (its `x`).
    CloseTab,
    /// Sizing the list's columns (the twins of dragging a separator).
    Column(ColumnCmd),
}

/// What the Radar's column keys and palette commands do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColumnCmd {
    /// Starts picking, or moves to the next sizable column.
    Pick,
    /// Widens or narrows the picked column (the repository's when none is).
    Wider,
    Narrower,
    /// Every column back to its automatic width.
    Reset,
    /// Fits the picked column to its content (the repository's when none is).
    Fit,
    /// Fits every sizable column to its content.
    FitAll,
}

/// A part of the Radar that `Ctrl+N` badges can reach (the badge
/// overlay and the key handler share this vocabulary; the stub build has no
/// Radar, so it never produces one). `Heading` indexes the view's rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Spot {
    Filter,
    Scope,
    View,
    Who,
    Refresh,
    Back,
    NotCloned,
    Drafts,
    TurnOn,
    Heading(usize),
    /// A control of the review choice dialog (its rows, then the checkbox).
    Choice(u8),
}

/// What the event loop must do after the Radar handled something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Outcome {
    None,
    /// A status line.
    Message(String),
    /// Back to the Home tab's repository list.
    Close,
    Quit,
    /// Text for the clipboard (a copied or cut selection of the filter).
    CopyText(String),
    /// Switch to the repository's tab (open it when it is not one), then run
    /// `Pending` in its window.
    OpenRepo(PathBuf, Option<Pending>),
    /// Settings, Integrations page, on this profile.
    Integrations(Option<String>),
    /// Settings, on the row of this config key (`repos.clone_root`).
    Setting(&'static str),
    /// Review a request through the host's API, in the Home window: the
    /// event loop builds the view (it knows whether AI is on) and hands it
    /// back to the Radar, which hosts it.
    ApiReview(Box<ReviewRequest>),
    /// Open the Clone dialog prefilled for a request; once the clone lands,
    /// its review opens in the new repository's tab.
    CloneReview(Box<ReviewRequest>),
    /// Read the conversation of an item whose repository is not cloned, in
    /// the Home window: the event loop builds the view and hands it back to
    /// the Radar, which hosts it (`item_host`).
    ApiConversation(Box<ConversationRequest>),
    /// Open the ordinary Clone dialog on the row's project; the clone lands
    /// like any other and the Radar finds it the next time it opens.
    CloneRepo(Box<CloneRequest>),
}

/// What the Clone action starts from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CloneRequest {
    /// The profile the row came from.
    pub(crate) integration: String,
    /// `owner/name` on the host.
    pub(crate) project: String,
    /// Where to clone from (the URL prefill).
    pub(crate) clone_url: String,
}

/// What a conversation opened from a Radar row starts from (plain text, so
/// the stub build compiles it too).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConversationRequest {
    /// The profile the row came from.
    pub(crate) integration: String,
    /// `owner/name` on the host.
    pub(crate) project: String,
    pub(crate) number: u64,
    /// A request (`!53`, `#7`) rather than an issue.
    pub(crate) pr: bool,
    pub(crate) label: String,
    pub(crate) title: String,
    pub(crate) web_url: String,
}

/// The request a review starts from (everything the Radar knows about it,
/// as plain text so the stub build compiles it too).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewRequest {
    /// The profile the row came from.
    pub(crate) integration: String,
    /// `owner/name` on the host.
    pub(crate) project: String,
    pub(crate) number: u64,
    /// `#12`, `!53`.
    pub(crate) label: String,
    pub(crate) title: String,
    pub(crate) web_url: String,
    pub(crate) source_branch: String,
    pub(crate) target_branch: String,
    /// Where to clone the project from (may be empty).
    pub(crate) clone_url: String,
}
