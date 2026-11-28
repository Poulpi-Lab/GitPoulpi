//! The vocabulary the event loop and the Radar share: what an action
//! is, what the palette can ask, and what the event loop must do afterwards.
//! No hosting types, so the build without a provider compiles it too
//! (`radar_stub.rs`).

#[allow(unused_imports)]
pub(crate) use gitpoulpi_app::palette::vocab::radar::{Act, ColumnCmd, RadarCommand, SizedColumn};
use std::path::PathBuf;

use super::pending::Pending;

/// A part of the Radar that `Ctrl+N` badges can reach (the badge
/// overlay and the key handler share this vocabulary; the stub build has no
/// Radar, so it never produces one). `Heading` indexes the view's rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Spot {
    Filter,
    Scope,
    View,
    Who,
    Profile,
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
