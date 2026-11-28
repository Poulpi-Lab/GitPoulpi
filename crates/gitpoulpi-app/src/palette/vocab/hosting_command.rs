//! The palette's hosting commands and what they need to be offered.
//!
//! One definition for every build: with the `hosting` feature off the stub
//! module reports every command unavailable, but the palette still names them.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it.

/// The palette's hosting commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostingCommand {
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
pub struct HostingAvailability {
    /// A profile answers for this repository.
    pub linked: bool,
    /// The checked-out branch has a pull request.
    pub head_pr: bool,
    pub has_prs: bool,
    pub has_issues: bool,
    pub has_commit_references: bool,
    /// The ISSUES section is linked, so references mean something.
    pub issues_linked: bool,
    /// At least one hosting profile is configured.
    pub has_profiles: bool,
    /// A GitHub / GitLab profile an issue tracker can be picked from.
    pub has_issue_profiles: bool,
    /// A provider is chosen for the ISSUES section.
    pub issue_provider: bool,
    /// A repository is chosen for the ISSUES section.
    pub issue_repo: bool,
    /// The chosen ISSUES provider takes an extra query (Jira, Azure, Trello).
    pub issue_query: bool,
    /// This repository has an issue source saved (`[issues_source]`).
    pub issues_pinned: bool,
    /// A GitHub / GitLab profile a pull-request source can be picked from.
    pub has_pull_profiles: bool,
    /// A provider is chosen for the PULL REQUESTS section.
    pub pull_provider: bool,
    /// This repository has a pull-request source saved (or a profile pinned).
    pub pulls_pinned: bool,
    /// This repository has an integration profile pinned.
    pub profile_pinned: bool,
    /// A listed pull / merge request is on GitHub or GitLab (metadata can
    /// be edited).
    pub meta_prs: bool,
    /// A listed issue is on GitHub or GitLab.
    pub meta_issues: bool,
    /// This repository blocks assignment.
    pub assignment_blocked: bool,
    /// The ISSUES source is a local Markdown folder.
    pub local_issues: bool,
    /// The PULL REQUESTS section is linked to a Gerrit project.
    pub gerrit_pulls: bool,
}

#[cfg(feature = "hosting")]
use gitpoulpi_i18n::msg;

#[cfg(feature = "hosting")]
fn not_linked() -> &'static str {
    msg::hosting_ui::WHY_NOT_LINKED.get()
}

/// Why `cmd` cannot run given `a`, or `None`.
#[cfg(feature = "hosting")]
pub fn command_unavailable_from(
    cmd: HostingCommand,
    a: HostingAvailability,
) -> Option<&'static str> {
    match cmd {
        HostingCommand::OpenLinkedPr
        | HostingCommand::EditLinkedPr
        | HostingCommand::CopyLinkedPrUrl => {
            if !a.linked {
                Some(not_linked())
            } else if !a.head_pr {
                Some(msg::hosting_ui::WHY_NO_HEAD_PR.get())
            } else {
                None
            }
        }
        HostingCommand::RefreshHosting
        | HostingCommand::SearchPullRequests
        | HostingCommand::ChoosePullRequestFilter => (!a.linked).then_some(not_linked()),
        HostingCommand::CreatePullRequest => {
            (!a.has_profiles).then_some(msg::hosting_ui::WHY_NO_INTEGRATION.get())
        }
        HostingCommand::CreateIssue => {
            if !a.issue_repo {
                Some(msg::hosting_ui::WHY_ISSUES_PICK_BOTH.get())
            } else {
                None
            }
        }
        HostingCommand::SearchIssues | HostingCommand::ChooseIssueFilter => {
            if !a.issue_repo {
                Some(msg::hosting_ui::WHY_ISSUES_PICK_REPO.get())
            } else {
                None
            }
        }
        HostingCommand::EditIssueQuery => {
            if !a.issue_repo {
                Some(msg::hosting_ui::WHY_ISSUES_PICK_REPO.get())
            } else if !a.issue_query {
                Some(msg::hosting_ui::WHY_NO_ISSUE_QUERY.get())
            } else {
                None
            }
        }
        HostingCommand::ChooseIssueProvider => {
            (!a.has_issue_profiles).then_some(msg::hosting_ui::WHY_NO_INTEGRATION.get())
        }
        HostingCommand::ChooseIssueRepository => {
            if !a.has_issue_profiles {
                Some(msg::hosting_ui::WHY_NO_INTEGRATION.get())
            } else if !a.issue_provider {
                Some(msg::hosting_ui::WHY_ISSUES_PICK_PROVIDER.get())
            } else {
                None
            }
        }
        HostingCommand::UnlinkIssueSource => {
            (!a.issues_pinned).then_some(msg::hosting_ui::WHY_NO_ISSUE_SOURCE.get())
        }
        HostingCommand::ChoosePullProvider => {
            (!a.has_pull_profiles).then_some(msg::hosting_ui::WHY_NO_INTEGRATION.get())
        }
        HostingCommand::ChoosePullRepository => {
            if !a.has_pull_profiles {
                Some(msg::hosting_ui::WHY_NO_INTEGRATION.get())
            } else if !a.pull_provider {
                Some(msg::hosting_ui::WHY_PULLS_PICK_PROVIDER.get())
            } else {
                None
            }
        }
        HostingCommand::UnlinkPullSource => {
            (!a.pulls_pinned).then_some(msg::hosting_ui::WHY_NO_PULL_SOURCE.get())
        }
        HostingCommand::UnlinkProfile => {
            (!a.profile_pinned).then_some(msg::hosting_ui::WHY_NO_PROFILE_PINNED.get())
        }
        HostingCommand::CheckoutPullRequest => {
            if !a.linked {
                Some(not_linked())
            } else if !a.has_prs {
                Some(msg::hosting_ui::WHY_NO_PRS.get())
            } else {
                None
            }
        }
        HostingCommand::OpenCommitReference => {
            (!a.has_commit_references).then_some(msg::hosting_ui::WHY_NO_COMMIT_REFS.get())
        }
        HostingCommand::InsertIssueRef => {
            (!a.issues_linked).then_some(msg::hosting_ui::WHY_NO_ISSUE_REF_SOURCE.get())
        }
        HostingCommand::OpenIssue => {
            if !a.has_issues {
                Some(msg::hosting_ui::WHY_NO_ISSUES.get())
            } else {
                None
            }
        }
        HostingCommand::UseProfile => {
            (!a.has_profiles).then_some(msg::hosting_ui::WHY_NO_HOSTING.get())
        }
        HostingCommand::EditReviewers => {
            (!a.meta_prs).then_some(msg::hosting_ui::WHY_NO_META_PRS.get())
        }
        HostingCommand::ReviewPullRequest => {
            (!a.meta_prs).then_some(msg::hosting_ui::WHY_NO_REVIEW.get())
        }
        HostingCommand::EditAssignees if a.assignment_blocked => {
            Some(msg::hosting_ui::WHY_ASSIGNMENT_BLOCKED.get())
        }
        HostingCommand::EditAssignees
        | HostingCommand::EditLabels
        | HostingCommand::SetMilestone => {
            (!a.meta_prs && !a.meta_issues).then_some(msg::hosting_ui::WHY_NO_META.get())
        }
        HostingCommand::CloseIssue
        | HostingCommand::ReopenIssue
        | HostingCommand::OpenInObsidian => {
            if !a.local_issues {
                Some(msg::local_issues::WHY_NOT_LOCAL.get())
            } else if !a.has_issues {
                Some(msg::hosting_ui::WHY_NO_ISSUES.get())
            } else {
                None
            }
        }
        HostingCommand::CreateObsidianBase => {
            (!a.local_issues).then_some(msg::local_issues::WHY_NOT_LOCAL.get())
        }
        HostingCommand::OpenIssueConversation => {
            (!a.has_issues).then_some(msg::hosting_ui::WHY_NO_ISSUES.get())
        }
        HostingCommand::OpenBoard => {
            (!a.issue_repo).then_some(msg::hosting_ui::WHY_ISSUES_PICK_REPO.get())
        }
        HostingCommand::PushForReview => {
            (!a.gerrit_pulls).then_some(msg::hosting_ui::WHY_NO_PUSH_REVIEW.get())
        }
        HostingCommand::IntegrationsPreferences
        | HostingCommand::ConnectIntegration
        | HostingCommand::RadarOptions => None,
    }
}

/// Always "unavailable": there is no provider in this build.
#[cfg(not(feature = "hosting"))]
pub fn command_unavailable_from(
    _cmd: HostingCommand,
    _a: HostingAvailability,
) -> Option<&'static str> {
    Some(gitpoulpi_i18n::msg::hosting_ui::WHY_NOT_BUILT.get())
}
