//! The review view's command vocabulary, kept apart from the view so the
//! command palette (and a build without a hosting provider) can name the
//! commands without the view's types.
#![cfg_attr(not(all(feature = "hosting", feature = "review")), allow(dead_code))]

use gitpoulpi_i18n::msg::review_view as msg;

/// The three pages of the view (Alt+1/2/3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Tab {
    Overview,
    Files,
    Commits,
}

/// What the palette needs to know to gray a review command out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReviewAvail {
    pub(crate) tab: Tab,
    /// The host data is in.
    pub(crate) loaded: bool,
    /// The Files page shows a file whose diff is in.
    pub(crate) diff_ready: bool,
    /// The cursor is on a line a comment can start from.
    pub(crate) on_code: bool,
    pub(crate) on_thread: bool,
    pub(crate) can_resolve: bool,
    pub(crate) on_pending: bool,
    pub(crate) pending: usize,
    pub(crate) can_edit_meta: bool,
    /// The request has patch sets to choose from (Gerrit).
    pub(crate) patch_sets: bool,
    /// The server says the change can be submitted now (Gerrit).
    pub(crate) can_submit: bool,
    /// The user may abandon the change (Gerrit).
    pub(crate) can_abandon: bool,
    pub(crate) busy: bool,
    /// AI is on (Settings > AI).
    pub(crate) ai: bool,
    /// A suggestion run is in flight.
    pub(crate) ai_running: bool,
    /// The request and its code are loaded: a run can start.
    pub(crate) ai_ready: bool,
    /// How many suggestions wait for the reviewer.
    pub(crate) suggestions: usize,
    /// The cursor (Files) or the list (Overview) is on a suggestion.
    pub(crate) on_suggestion: bool,
    pub(crate) has_summary: bool,
    /// The Overview holds a ✦ review draft.
    pub(crate) has_draft: bool,
}

/// What the view can be asked to do. The palette lists one entry per value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReviewCmd {
    TabOverview,
    TabFiles,
    TabCommits,
    /// A comment on the cursor line or the selected range (pending).
    AddComment,
    /// A comment in the conversation (posted at once).
    ConversationComment,
    Reply,
    ToggleResolve,
    ToggleThread,
    EditPending,
    DeletePending,
    ToggleViewed,
    NextFile,
    PrevFile,
    FinishReview,
    DiscardReview,
    Refresh,
    OpenBrowser,
    Checkout,
    Close,
    EditAssignees,
    EditReviewers,
    EditLabels,
    SetMilestone,
    /// Choose the patch set to read (Gerrit).
    PickPatchSet,
    /// Choose the older patch set the diff starts from (Gerrit).
    ComparePatchSet,
    /// Merge the change on the server (Gerrit); asks first.
    SubmitChange,
    /// Abandon the change on the server (Gerrit); asks first.
    AbandonChange,
    /// ✦ Ask the AI for suggested comments (only on this explicit action).
    SuggestReview,
    AcceptSuggestion,
    /// Change a suggestion's words, then accept it.
    EditSuggestion,
    DismissSuggestion,
    AcceptAllSuggestions,
    DismissAllSuggestions,
    /// Put the AI's summary in the Finish review text.
    UseSummary,
    /// ✦ Use the review draft: accept its suggestions, open Finish review
    /// filled with its text and verdict.
    UseDraft,
    /// Change the draft's review text.
    EditDraftText,
    /// Throw the draft away (text, verdict and suggestions).
    DiscardDraft,
    /// The Diff view's display toggles, on the diff pane: Inline <-> Split.
    ToggleLayout,
    /// Only the hunks <-> the whole file.
    ToggleHunksOnly,
    /// Show or ignore whitespace-only changes.
    ToggleWhitespace,
    ToggleWrap,
    NextHunk,
    PrevHunk,
    /// The encoding dropdown of the diff pane's bar.
    ChooseEncoding,
    /// The minimap beside the diff (the `ui.minimap` setting).
    ToggleMinimap,
    /// Scroll the diff sideways.
    ScrollLeft,
    ScrollRight,
    /// Copy the selected text, or the line under the cursor.
    CopyText,
}

impl ReviewCmd {
    /// The commands that only exist while AI is on: the palette hides them.
    pub(crate) fn is_ai(self) -> bool {
        matches!(
            self,
            Self::SuggestReview
                | Self::AcceptSuggestion
                | Self::EditSuggestion
                | Self::DismissSuggestion
                | Self::AcceptAllSuggestions
                | Self::DismissAllSuggestions
                | Self::UseSummary
                | Self::UseDraft
                | Self::EditDraftText
                | Self::DiscardDraft
        )
    }

    /// Why the command does nothing right now (`None`: it can run).
    pub(crate) fn unavailable(self, avail: Option<ReviewAvail>) -> Option<&'static str> {
        let Some(a) = avail else {
            return Some(msg::WHY_NO_VIEW.get());
        };
        let files = a.tab == Tab::Files;
        match self {
            Self::TabOverview
            | Self::TabFiles
            | Self::TabCommits
            | Self::Refresh
            | Self::OpenBrowser
            | Self::Checkout
            | Self::Close => None,
            Self::ConversationComment => (!a.loaded).then_some(msg::NOT_LOADED.get()),
            Self::AddComment => {
                if !files {
                    Some(msg::WHY_NOT_ON_FILES.get())
                } else if !a.on_code {
                    Some(msg::WHY_NOT_ON_CODE.get())
                } else {
                    None
                }
            }
            Self::Reply | Self::ToggleThread => {
                if !files {
                    Some(msg::WHY_NOT_ON_FILES.get())
                } else if !a.on_thread {
                    Some(msg::WHY_NOT_ON_THREAD.get())
                } else {
                    None
                }
            }
            Self::ToggleResolve => {
                if !files {
                    Some(msg::WHY_NOT_ON_FILES.get())
                } else if !a.on_thread {
                    Some(msg::WHY_NOT_ON_THREAD.get())
                } else if !a.can_resolve {
                    Some(msg::WHY_NO_RESOLVE.get())
                } else {
                    None
                }
            }
            Self::EditPending | Self::DeletePending => {
                if !files {
                    Some(msg::WHY_NOT_ON_FILES.get())
                } else if !a.on_pending {
                    Some(msg::WHY_NOT_ON_PENDING.get())
                } else {
                    None
                }
            }
            Self::ToggleViewed | Self::NextFile | Self::PrevFile => {
                (!files).then_some(msg::WHY_NOT_ON_FILES.get())
            }
            Self::FinishReview => (!a.loaded).then_some(msg::NOT_LOADED.get()),
            Self::DiscardReview => (a.pending == 0).then_some(msg::WHY_NO_PENDING.get()),
            Self::EditAssignees | Self::EditReviewers | Self::EditLabels | Self::SetMilestone => {
                (!a.can_edit_meta).then_some(msg::WHY_NO_META.get())
            }
            Self::PickPatchSet | Self::ComparePatchSet => {
                (!a.patch_sets).then_some(msg::WHY_NO_PATCH_SETS.get())
            }
            Self::SubmitChange => (!a.can_submit).then_some(msg::WHY_CANNOT_SUBMIT.get()),
            Self::AbandonChange => (!a.can_abandon).then_some(msg::WHY_CANNOT_ABANDON.get()),
            Self::SuggestReview => {
                use gitpoulpi_i18n::msg::review_ai as ai;
                if !a.ai {
                    Some(ai::WHY_OFF.get())
                } else if a.ai_running {
                    Some(ai::WHY_BUSY.get())
                } else if !a.loaded {
                    Some(ai::WHY_NOT_LOADED.get())
                } else if !a.ai_ready {
                    Some(ai::WHY_NO_DIFF.get())
                } else {
                    None
                }
            }
            Self::AcceptSuggestion | Self::EditSuggestion | Self::DismissSuggestion => {
                use gitpoulpi_i18n::msg::review_ai as ai;
                if !a.ai {
                    Some(ai::WHY_OFF.get())
                } else if !a.on_suggestion {
                    Some(ai::WHY_NO_SUGGESTION.get())
                } else {
                    None
                }
            }
            Self::AcceptAllSuggestions | Self::DismissAllSuggestions => {
                use gitpoulpi_i18n::msg::review_ai as ai;
                if !a.ai {
                    Some(ai::WHY_OFF.get())
                } else if a.suggestions == 0 {
                    Some(ai::WHY_NO_SUGGESTION.get())
                } else {
                    None
                }
            }
            Self::ToggleLayout
            | Self::ToggleHunksOnly
            | Self::ToggleWhitespace
            | Self::ToggleWrap
            | Self::NextHunk
            | Self::PrevHunk
            | Self::ChooseEncoding
            | Self::ToggleMinimap
            | Self::ScrollLeft
            | Self::ScrollRight
            | Self::CopyText => (a.tab == Tab::Overview).then_some(msg::WHY_NOT_ON_DIFF.get()),
            Self::UseDraft | Self::EditDraftText | Self::DiscardDraft => {
                use gitpoulpi_i18n::msg::review_ai as ai;
                if !a.ai {
                    Some(ai::WHY_OFF.get())
                } else if !a.has_draft {
                    Some(ai::WHY_NO_DRAFT.get())
                } else if self == Self::UseDraft && !a.loaded {
                    Some(msg::NOT_LOADED.get())
                } else {
                    None
                }
            }
            Self::UseSummary => {
                use gitpoulpi_i18n::msg::review_ai as ai;
                if !a.ai {
                    Some(ai::WHY_OFF.get())
                } else if !a.loaded {
                    Some(msg::NOT_LOADED.get())
                } else if !a.has_summary {
                    Some(ai::WHY_NO_SUMMARY.get())
                } else {
                    None
                }
            }
        }
    }
}
