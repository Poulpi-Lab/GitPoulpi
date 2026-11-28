//! The palette's hosting commands and what they need to be offered.
//!
//! One definition for every build: with the `hosting` feature off the stub
//! module reports every command unavailable, but the palette still names them.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it.

/// The palette's hosting commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HostingCommand {
    OpenLinkedPr,
    /// The checked-out branch's request, on the host's edit page.
    EditLinkedPr,
    RefreshHosting,
    CopyLinkedPrUrl,
    CheckoutPullRequest,
    OpenIssue,
    /// `#123` / `!45` in the selected commit's message.
    OpenCommitReference,
    /// The composer's `#` button: pick an issue reference to type into the
    /// commit message (the branch's issue first).
    InsertIssueRef,
    IntegrationsPreferences,
    UseProfile,
    ConnectIntegration,
    /// Settings > Integrations, on the Radar block of a profile.
    RadarOptions,
    /// The header's `+`.
    CreatePullRequest,
    /// The ISSUES header's `+`: the New issue form.
    CreateIssue,
    /// Puts the keyboard in the PULL REQUESTS search box.
    SearchPullRequests,
    /// Opens the PULL REQUESTS state dropdown.
    ChoosePullRequestFilter,
    SearchIssues,
    ChooseIssueFilter,
    /// The ISSUES section's extra query row: Jira JQL / Azure WIQL text, or
    /// Trello's lists dropdown.
    EditIssueQuery,
    /// The ISSUES section's provider dropdown.
    ChooseIssueProvider,
    /// The ISSUES section's repository dropdown.
    ChooseIssueRepository,
    /// Clears this repository's chosen issue source (`[issues_source]`).
    UnlinkIssueSource,
    /// The PULL REQUESTS section's provider dropdown.
    ChoosePullProvider,
    /// The PULL REQUESTS section's repository dropdown.
    ChoosePullRepository,
    /// Clears this repository's chosen pull-request source.
    UnlinkPullSource,
    /// Clears this repository's pinned integration profile.
    UnlinkProfile,
    /// Pick a request or an issue, then choose its assignees.
    EditAssignees,
    /// Pick a request, then choose its reviewers.
    EditReviewers,
    /// Pick a request or an issue, then choose its labels.
    EditLabels,
    /// Pick a request or an issue, then choose its milestone.
    SetMilestone,
    /// Pick a request, then review it in the app.
    ReviewPullRequest,
    /// Pick a local issue to close.
    CloseIssue,
    /// Pick a local issue to reopen.
    ReopenIssue,
    /// Pick a local issue to open in Obsidian.
    OpenInObsidian,
    /// Write `issues.base` next to the local notes.
    CreateObsidianBase,
    /// The Kanban board of the ISSUES source.
    OpenBoard,
    /// Pick an issue, then read its conversation in the app.
    OpenIssueConversation,
    /// Push `HEAD` to `refs/for/<branch>` to open a Gerrit change.
    PushForReview,
}

/// What the palette needs to know to gray a hosting command out --
/// plain data, so `PaletteContext` stays `Copy`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct HostingAvailability {
    /// A profile answers for this repository.
    pub(crate) linked: bool,
    /// The checked-out branch has a pull request.
    pub(crate) head_pr: bool,
    pub(crate) has_prs: bool,
    pub(crate) has_issues: bool,
    pub(crate) has_commit_references: bool,
    /// The ISSUES section is linked, so references mean something.
    pub(crate) issues_linked: bool,
    /// At least one hosting profile is configured.
    pub(crate) has_profiles: bool,
    /// A GitHub / GitLab profile an issue tracker can be picked from.
    pub(crate) has_issue_profiles: bool,
    /// A provider is chosen for the ISSUES section.
    pub(crate) issue_provider: bool,
    /// A repository is chosen for the ISSUES section.
    pub(crate) issue_repo: bool,
    /// The chosen ISSUES provider takes an extra query (Jira, Azure, Trello).
    pub(crate) issue_query: bool,
    /// This repository has an issue source saved (`[issues_source]`).
    pub(crate) issues_pinned: bool,
    /// A GitHub / GitLab profile a pull-request source can be picked from.
    pub(crate) has_pull_profiles: bool,
    /// A provider is chosen for the PULL REQUESTS section.
    pub(crate) pull_provider: bool,
    /// This repository has a pull-request source saved (or a profile pinned).
    pub(crate) pulls_pinned: bool,
    /// This repository has an integration profile pinned.
    pub(crate) profile_pinned: bool,
    /// A listed pull / merge request is on GitHub or GitLab (metadata can
    /// be edited).
    pub(crate) meta_prs: bool,
    /// A listed issue is on GitHub or GitLab.
    pub(crate) meta_issues: bool,
    /// This repository blocks assignment.
    pub(crate) assignment_blocked: bool,
    /// The ISSUES source is a local Markdown folder.
    pub(crate) local_issues: bool,
    /// The PULL REQUESTS section is linked to a Gerrit project.
    pub(crate) gerrit_pulls: bool,
}
