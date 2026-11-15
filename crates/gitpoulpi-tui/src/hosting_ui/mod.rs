//! Hosting in the window: the PULL REQUESTS and ISSUES sidebar sections,
//! the toolbar's linked-PR chip, their context menus and the palette
//! commands -- everything the user sees of `hosting_task`.
//!
//! `HostingUi` is the one value `event_loop` holds. It owns the worker,
//! the refresh policy, the current snapshot and the open menu, and answers
//! every question the loop has (how many rows, what to paint, what a
//! click means) so the loop itself stays thin: it forwards events and acts
//! on the returned [`HostingEffect`], the few outcomes that need state
//! only the loop owns (checking a branch out, opening Preferences).
//!
//! Every clickable registers a `HitTarget::Hosting(HostingHit)` from the
//! same walk that paints it, with the strong `plain_button_style(true)`
//! hover, and has a keyboard equivalent (see `docs/user/keybindings.md`).
//!
//! Layout: this file holds the types and the `HostingUi` struct; `sync`
//! the lifecycle (worker, refresh, applying a snapshot); `rows` the rows
//! and painting; `picker` reveal, the source dropdowns and the search box;
//! `actions` menus, row activation and the palette commands.

use super::*;

use gitpoulpi_i18n::msg;

use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::Arc;

use gitpoulpi_config::Integration;
use gitpoulpi_core::domain::Remote;
use gitpoulpi_hosting::{
    IssueState, IssueSummary, PullRequestState, PullRequestSummary, ResolveRemote,
};

use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::context_menu::Entry;
use crate::hosting_task::{
    age_label, cache_path, load_cache, refs_signature, save_cache, start_projects, unix_now,
    DefaultFactory, HostingRequest, HostingSnapshot, HostingWorker, IssueView, PrView, Problem,
    ProjectsReceiver, ProjectsState, RefreshPolicy,
};

/// What [`HostingUi::reveal`] found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RevealOutcome {
    /// The item's row index in the section.
    Row(usize),
    /// The repository is not linked to a provider for this list.
    NotLinked,
    /// Linked, but the item is not in the list (or the search hides it).
    NotListed,
}

/// Which of the two lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HostingList {
    Pulls,
    Issues,
}

impl HostingList {
    pub(crate) fn section(self) -> SidebarSection {
        match self {
            HostingList::Pulls => SidebarSection::PullRequests,
            HostingList::Issues => SidebarSection::Issues,
        }
    }

    pub(crate) fn from_section(section: SidebarSection) -> Option<Self> {
        match section {
            SidebarSection::PullRequests => Some(HostingList::Pulls),
            SidebarSection::Issues => Some(HostingList::Issues),
            _ => None,
        }
    }
}

/// Set when an issue was just created through the New issue form: the
/// form lives in `repo_dialogs`, which has no handle on the hosting UI,
/// so the next [`HostingUi::poll`] sees this and re-fetches.
static ISSUES_REFRESH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(crate) fn request_issues_refresh() {
    ISSUES_REFRESH.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// Counts the issue lists the sidebar has read (a refresh landed). The open
/// Kanban board watches it, so a folder edit or the host's refresh cadence
/// moves the board too.
///
/// In unit tests the counter is **thread-local**: tests run in parallel in
/// one process, and a bump from one test (the sidebar tests, the board test
/// that bumps it on purpose) made another test's board start a second load
/// mid-test, which read the fixture before the test edited it
/// (`an_empty_column_is_still_shown` failed once in a full run).
#[cfg(not(test))]
static ISSUES_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
#[cfg(test)]
thread_local! {
    static ISSUES_GENERATION: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg_attr(not(feature = "board"), allow(dead_code))]
pub(crate) fn issues_generation() -> u64 {
    #[cfg(not(test))]
    {
        ISSUES_GENERATION.load(std::sync::atomic::Ordering::SeqCst)
    }
    #[cfg(test)]
    {
        ISSUES_GENERATION.with(|c| c.get())
    }
}

/// A refresh landed.
pub(crate) fn bump_issues_generation() {
    #[cfg(not(test))]
    ISSUES_GENERATION.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    #[cfg(test)]
    ISSUES_GENERATION.with(|c| c.set(c.get() + 1));
}

/// What a landed refresh does, for the board's tests.
#[cfg(test)]
#[cfg_attr(not(feature = "board"), allow(dead_code))]
pub(crate) fn bump_issues_generation_for_tests() {
    bump_issues_generation();
}

/// A click target inside the hosting UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HostingHit {
    /// A row of a sidebar list (index into `HostingUi::rows`).
    Row(HostingList, usize),
    /// The `+` button on the PULL REQUESTS header.
    NewPr,
    /// The `+` button on the ISSUES header (once a source is chosen).
    NewIssue,
    /// The Board button on the ISSUES header (once a source is chosen).
    Board,
    /// The pencil at the right end of a pull request row (index into
    /// `HostingUi::rows`): "Edit on GitHub / GitLab".
    EditRow(usize),
    /// One row of the open hosting menu.
    MenuItem(usize),
    /// One row of the open combobox popup (index in its filtered list).
    PickerItem(usize),
    /// The toolbar's `GL !53` chip.
    LinkedPrChip,
    /// The toolbar's dim `+ PR` chip.
    NewPrChip,
}

/// The four collapsible groups of the PULL REQUESTS section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PrGroup {
    Mine,
    Assigned,
    Awaiting,
    All,
}

impl PrGroup {
    /// The key a folded group is remembered under (`fold_state`).
    pub(crate) fn key(self) -> &'static str {
        match self {
            PrGroup::Mine => "mine",
            PrGroup::Assigned => "assigned",
            PrGroup::Awaiting => "awaiting",
            PrGroup::All => "all",
        }
    }

    pub(crate) const ALL: [PrGroup; 4] = [
        PrGroup::Mine,
        PrGroup::Assigned,
        PrGroup::Awaiting,
        PrGroup::All,
    ];

    /// The group's header text; the pull / merge request word follows the host.
    pub(crate) fn label(self) -> &'static str {
        match self {
            PrGroup::Mine => msg::hosting_ui::GROUP_MINE.get(),
            PrGroup::Assigned => msg::hosting_ui::GROUP_ASSIGNED.get(),
            PrGroup::Awaiting => msg::hosting_ui::GROUP_AWAITING.get(),
            PrGroup::All => msg::hosting_ui::GROUP_ALL.get(),
        }
    }

    /// Whether `pr` belongs to the group, for the signed-in user `me`.
    fn holds(self, pr: &PullRequestSummary, me: &str) -> bool {
        let is_me = |login: &String| !me.is_empty() && login.eq_ignore_ascii_case(me);
        match self {
            PrGroup::All => true,
            PrGroup::Mine => !me.is_empty() && pr.author.eq_ignore_ascii_case(me),
            PrGroup::Assigned => pr.assignees.iter().any(is_me),
            PrGroup::Awaiting => pr.review_requested || pr.reviewers.iter().any(is_me),
        }
    }
}

/// One row of a sidebar list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HostingRow {
    /// "Reconnect" - the token is missing, rejected or short of scope.
    Reconnect,
    /// An inert line (an error, or "offline").
    Notice(String),
    /// The search box.
    Search,
    /// The state-filter combobox.
    Filter,
    /// A group's header (chevron, name, count).
    Group(PrGroup),
    /// A pull request under a group: `(group, index into the snapshot)`.
    Pull(PrGroup, usize),
    /// The issue provider combobox (GitHub / GitLab profiles).
    IssueProvider,
    /// The provider's repository combobox.
    IssueRepo,
    Issue(usize),
    /// The pull-request provider combobox (GitHub / GitLab profiles).
    PullProvider,
    /// The pull-request provider's repository combobox.
    PullRepo,
    /// The optional extra query of the issue source: a JQL / WIQL text field
    /// (Jira, Azure) or the Trello lists dropdown.
    IssueQuery,
}

impl HostingRow {
    /// A row that is a control, not an item: one click uses it.
    pub(crate) fn is_control(&self) -> bool {
        !matches!(
            self,
            HostingRow::Notice(_) | HostingRow::Pull(..) | HostingRow::Issue(_)
        )
    }
}

/// What a menu entry or a palette command does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HostingAction {
    OpenUrl(String),
    CopyText {
        what: String,
        text: String,
    },
    CheckoutBranch {
        branch: String,
        remote: String,
    },
    ViewBranch {
        branch: String,
        remote: String,
    },
    /// A pull request from a fork: fetch its head ref into a tracking
    /// local branch and check that out.
    CheckoutPullRequestHead {
        number: u64,
    },
    NewBranchFromIssue(String),
    StartPullRequest(String),
    Refresh,
    UseProfile(String),
    AutomaticProfile,
    /// Clear the chosen issue source.
    UnlinkIssues,
    /// Clear the chosen pull-request source (and any pinned profile).
    UnlinkPulls,
    Preferences(Option<String>),
    /// Close (`closed`) or reopen a local Markdown issue.
    LocalState {
        number: u64,
        closed: bool,
    },
    /// Open a local note (its `file://` URL) in Obsidian.
    OpenObsidian(String),
    /// Edit one field (assignees, reviewers, labels, milestone) of a
    /// request (`pr`) or an issue, in the metadata picker.
    EditMeta {
        field: crate::meta_picker::MetaField,
        pr: bool,
        number: u64,
    },
    /// Open the in-app review of a request (GitHub, GitLab).
    Review {
        number: u64,
    },
    /// Open the conversation of an issue (`pr: false`) or of a request on a
    /// host with no review view.
    Conversation {
        number: u64,
        pr: bool,
    },
    /// Push `HEAD` to `refs/for/<branch>` (Gerrit).
    PushForReview {
        branch: String,
    },
}

/// What the event loop must do after `HostingUi` handled something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HostingEffect {
    None,
    Message(String),
    /// Open Preferences on the Integrations page, on this profile.
    Preferences(Option<String>),
    /// Open Preferences on the Radar block of the first connected code host.
    RadarOptions,
    /// Check out the source branch of a pull request.
    Checkout {
        branch: String,
        remote: String,
    },
    /// Fetch a pull request's head ref into a local branch and check it
    /// out (the fork case, where no remote branch exists).
    CheckoutPullRequestHead {
        remote: String,
        remote_ref: String,
        branch: String,
    },
    /// Show the source branch's tip in the Log.
    ViewBranch {
        branch: String,
        remote: String,
    },
    /// Open the new-branch prompt with this name filled in.
    NewBranch(String),
    /// The existing Start-a-pull-request flow, for this local branch.
    StartPullRequest(String),
    /// The Create Pull Request form (the header's `+`), for the
    /// checked-out branch.
    CreatePullRequest,
    /// The New issue form (the ISSUES header's `+`), for the linked source.
    NewIssue(Box<crate::issue_form::IssueSeed>),
    /// The in-app review of a listed request.
    Review(Box<crate::review_view::ReviewTarget>),
    /// The Kanban board of the ISSUES source.
    Board(Box<crate::board_view::BoardTarget>),
    /// The conversation of an issue or a request.
    Item(Box<crate::item_view::ItemTarget>),
    /// Push `HEAD` to `refs/for/<branch>` of `remote` (Gerrit).
    PushForReview {
        remote: String,
        branch: String,
    },
    /// The metadata picker for one field of a listed request or issue.
    EditMeta(
        Box<crate::meta_picker::MetaTarget>,
        crate::meta_picker::MetaField,
    ),
    /// The composer's "Insert issue reference" picker chose this reference:
    /// type it at the caret of the field last focused.
    InsertIssueRef(String),
    /// Put the sidebar's keyboard focus on this list (a search box or a
    /// dropdown was opened from the palette).
    Focus(HostingList),
}

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

/// Marks a provider-dropdown row that proposes a whole `(profile,
/// repository)` pair taken from a remote.
const SUGGEST_PREFIX: &str = "@";

/// The provider dropdown's "clear the choice" row (its value is empty).
fn unlink_label() -> &'static str {
    msg::hosting_ui::UNLINK_OPTION.get()
}

fn not_linked() -> &'static str {
    msg::hosting_ui::WHY_NOT_LINKED.get()
}

/// Why `cmd` cannot run given `a`, or `None`.
pub(crate) fn command_unavailable_from(
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

/// What the toolbar needs to draw the chip.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ToolbarChip {
    /// The label, without padding: `GL !53`, `+ PR`.
    pub(crate) text: String,
    /// Resting colors; hovered, the strong plain-button fill takes over.
    pub(crate) fg: Color,
    pub(crate) bg: Option<Color>,
    pub(crate) hit: HostingHit,
}

fn state_color(state: PullRequestState) -> Color {
    match state {
        PullRequestState::Open => th().refs.local,
        PullRequestState::Draft => th().ui.muted,
        PullRequestState::Merged => th().hosting.merged,
        PullRequestState::Closed => th().status.danger,
    }
}

fn state_glyph(state: PullRequestState) -> &'static str {
    match state {
        PullRequestState::Open => "\u{25CF}",
        PullRequestState::Draft => "\u{25CB}",
        PullRequestState::Merged => "\u{25C6}",
        PullRequestState::Closed => "\u{2715}",
    }
}

fn state_word(state: PullRequestState) -> &'static str {
    match state {
        PullRequestState::Open => msg::hosting_ui::STATE_OPEN.get(),
        PullRequestState::Draft => msg::hosting_ui::STATE_DRAFT.get(),
        PullRequestState::Merged => msg::hosting_ui::STATE_MERGED.get(),
        PullRequestState::Closed => msg::hosting_ui::STATE_CLOSED.get(),
    }
}

/// `GH` / `GL` / `AZ` / `BB` for the profile's provider family (the chip's
/// tag; the same letters `gitpoulpi_hosting::FamilySpec::tag` carries).
pub(crate) fn provider_tag(kind: Option<gitpoulpi_config::IntegrationKind>) -> &'static str {
    use gitpoulpi_config::ApiFamily;
    match kind.map(|k| k.family()) {
        Some(ApiFamily::Github) => "GH",
        Some(ApiFamily::Gitlab) => "GL",
        Some(ApiFamily::AzureDevops) => "AZ",
        Some(ApiFamily::Bitbucket) => "BB",
        Some(ApiFamily::Jira) => "JI",
        Some(ApiFamily::Trello) => "TR",
        Some(ApiFamily::Gitea) => "GT",
        Some(ApiFamily::Linear) => "LN",
        Some(ApiFamily::YouTrack) => "YT",
        Some(ApiFamily::Redmine) => "RM",
        Some(ApiFamily::Gerrit) => "GR",
        Some(ApiFamily::LocalMarkdown) => "MD",
        None => msg::hosting_ui::PR_ABBR.get(),
    }
}

/// The branch name "Create branch from issue" offers for `issue` of a
/// provider of `family` (plan D7): Jira, Linear and YouTrack name it from
/// their key (`ABC-123-fix-login`, which they link back; Linear's own
/// `branchName` wins when it sent one); Azure, GitHub, GitLab
/// and Trello from the number (`123-fix-login`, Trello's `idShort`).
pub(crate) fn issue_branch_for(
    family: Option<gitpoulpi_config::ApiFamily>,
    issue: &IssueSummary,
) -> String {
    // Linear proposes its own name (`branchName`); it links the branch back.
    if let Some(name) = issue.branch_name.as_deref().filter(|n| !n.is_empty()) {
        return name.to_string();
    }
    use gitpoulpi_config::ApiFamily as F;
    if matches!(family, Some(F::Jira | F::Linear | F::YouTrack)) && !issue.reference.is_empty() {
        let slug = issue_branch_name(0, &issue.title);
        let slug = slug.strip_prefix("0-").unwrap_or("");
        return if slug.is_empty() {
            issue.reference.clone()
        } else {
            format!("{}-{slug}", issue.reference)
        };
    }
    issue_branch_name(issue.number, &issue.title)
}

/// `12-fix-the-thing`: the branch name "Create branch from issue" offers.
pub(crate) fn issue_branch_name(number: u64, title: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = true;
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    let slug: String = slug.trim_matches('-').chars().take(40).collect();
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        number.to_string()
    } else {
        format!("{number}-{slug}")
    }
}

/// The ref name a pull request's source branch can be checked out /
/// shown as: a local branch of that name if there is one, else the
/// remote-tracking one from the profile's remote. `None` when neither
/// exists (not fetched yet).
pub(crate) fn branch_target(
    refs: &[gitpoulpi_core::domain::Ref],
    remote: &str,
    branch: &str,
) -> Option<String> {
    let local = refs
        .iter()
        .any(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == branch));
    if local {
        return Some(branch.to_string());
    }
    refs.iter()
        .any(|r| {
            matches!(&r.kind, RefKind::RemoteBranch { remote: rm, name }
                if rm == remote && name == branch)
        })
        .then(|| format!("{remote}/{branch}"))
}

/// What the last repository refresh said, as far as hosting cares.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Inputs {
    remotes: Vec<ResolveRemote>,
    head_branch: Option<String>,
    /// The checked-out branch has an upstream: it was pushed.
    head_pushed: bool,
}

/// Render-loop ticks a freshly opened window lets pass after its first
/// refresh before the first hosting (network) request, so the graph and
/// the refs are on screen first.
const OPEN_TICKS_BEFORE_REQUEST: u8 = 2;

/// A hosting request in flight for longer than this shows "Loading..." in
/// the sections it feeds (maintainer rule 9: feedback after about 150 ms).
pub(crate) const LOADING_AFTER: Duration = Duration::from_millis(150);

/// Which dropdown a picker is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PickerTarget {
    PrFilter,
    IssueFilter,
    IssueProvider,
    IssueRepo,
    PullProvider,
    PullRepo,
    /// Trello: which lists of the board the ISSUES section shows.
    IssueLists,
    /// The composer's `#` button: the reference to type into the message.
    IssueRef,
}

impl PickerTarget {
    fn list(self) -> HostingList {
        match self {
            PickerTarget::PrFilter | PickerTarget::PullProvider | PickerTarget::PullRepo => {
                HostingList::Pulls
            }
            _ => HostingList::Issues,
        }
    }

    /// The sidebar row the dropdown hangs from.
    /// `None` for the composer's picker, which hangs from its own button.
    fn row(self) -> Option<HostingRow> {
        Some(match self {
            PickerTarget::PrFilter | PickerTarget::IssueFilter => HostingRow::Filter,
            PickerTarget::IssueProvider => HostingRow::IssueProvider,
            PickerTarget::IssueRepo => HostingRow::IssueRepo,
            PickerTarget::PullProvider => HostingRow::PullProvider,
            PickerTarget::PullRepo => HostingRow::PullRepo,
            PickerTarget::IssueLists => HostingRow::IssueQuery,
            PickerTarget::IssueRef => return None,
        })
    }
}

/// The open dropdown of a sidebar row.
pub(crate) struct HostingPicker {
    target: PickerTarget,
    combo: Combo,
    /// Where the popup hangs when its row is not on screen.
    fallback: Rect,
}

/// The host's page to edit a pull / merge request: GitLab has a dedicated
/// edit view (`.../merge_requests/53/edit`); GitHub edits the title and
/// description in place on the request's own page.
pub(crate) fn edit_url(kind: Option<gitpoulpi_config::IntegrationKind>, web_url: &str) -> String {
    gitpoulpi_hosting::pull_request_edit_url(kind, web_url)
}

/// `GitHub` / `GitLab` / `Azure DevOps` / ... for a profile kind (product
/// names, not translated).
pub(crate) fn provider_name(kind: gitpoulpi_config::IntegrationKind) -> &'static str {
    use gitpoulpi_config::ApiFamily;
    match kind.family() {
        ApiFamily::Github => "GitHub",
        ApiFamily::Gitlab => "GitLab",
        ApiFamily::AzureDevops => "Azure DevOps",
        ApiFamily::Bitbucket => "Bitbucket",
        ApiFamily::Jira => "Jira",
        ApiFamily::Trello => "Trello",
        ApiFamily::Gitea => kind.label(),
        ApiFamily::Linear => "Linear",
        ApiFamily::YouTrack => "YouTrack",
        ApiFamily::Redmine => "Redmine",
        ApiFamily::Gerrit => "Gerrit",
        ApiFamily::LocalMarkdown => "Local Markdown", // i18n-english
    }
}

/// Whether the ISSUES section of a provider of `family` takes an extra
/// provider-native query (`IssuesSource::filter`): a JQL fragment (Jira), a
/// WIQL condition (Azure), Trello list names, a YouTrack query.
pub(crate) fn takes_issue_query(family: gitpoulpi_config::ApiFamily) -> bool {
    use gitpoulpi_config::ApiFamily;
    matches!(
        family,
        ApiFamily::Jira | ApiFamily::AzureDevops | ApiFamily::Trello | ApiFamily::YouTrack
    )
}

/// Whether `pr` matches what was typed in the search box.
fn pr_matches(pr: &PullRequestSummary, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    q.is_empty()
        || [&pr.title, &pr.iid_label, &pr.author, &pr.source_branch]
            .iter()
            .any(|f| f.to_lowercase().contains(&q))
}

fn issue_matches(issue: &IssueSummary, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    q.is_empty()
        || [&issue.title, &issue.iid_label, &issue.author]
            .iter()
            .any(|f| f.to_lowercase().contains(&q))
        || issue.labels.iter().any(|l| l.to_lowercase().contains(&q))
}

pub(crate) struct HostingUi {
    repo_root: PathBuf,
    worker: Option<HostingWorker>,
    policy: RefreshPolicy,
    integrations: fn() -> Vec<Integration>,
    override_id: Option<String>,
    /// The provider and project the ISSUES section reads from, when the
    /// user picked them (`RepoState::issues_source`).
    issues_source: Option<gitpoulpi_session::IssuesSource>,
    /// The provider and repository the PULL REQUESTS section reads from
    /// (`RepoState::pulls_source`); `None` is unlinked.
    pulls_source: Option<gitpoulpi_session::IssuesSource>,
    inputs: Option<Inputs>,
    snapshot: Option<HostingSnapshot>,
    /// False for a snapshot read from disk and not yet confirmed live.
    fresh: bool,
    pr_view: PrView,
    issue_view: IssueView,
    cache: Option<PathBuf>,
    pub(crate) menu: Option<ContextMenu<HostingAction>>,
    /// The open dropdown (filter, provider, repository).
    pub(crate) picker: Option<HostingPicker>,
    /// What was typed in each list's search box.
    pr_query: String,
    issue_query: String,
    /// The search box that has the keyboard.
    pub(crate) editing: Option<HostingList>,
    /// The issue-query row (a JQL / WIQL field) is what `editing` edits; its
    /// text so far is `query_draft`, saved when the field is left.
    editing_query: bool,
    query_draft: String,
    /// What `Ctrl+Z` / `Ctrl+Y` walk in whichever of these boxes has the keyboard.
    query_history: crate::text_history::QueryHistory,
    /// The Trello lists of the linked board, once asked: `(profile, board)`
    /// and the list names (`None` while they load or when the call failed).
    lists: Option<(String, String, Option<Vec<String>>)>,
    lists_rx: Option<(String, String, crate::hosting_task::OptionsReceiver)>,
    /// Folded PULL REQUESTS groups ("All" starts folded, as it
    /// holds long lists).
    collapsed: HashSet<PrGroup>,
    /// Where each visible row was painted last frame (dropdown anchors).
    row_rects: RefCell<Vec<(HostingList, usize, Rect)>>,
    /// Provider project lists, by profile id.
    projects: std::collections::HashMap<String, ProjectsState>,
    projects_rx: Vec<ProjectsReceiver>,
    /// Something the status line should say (a failed project listing).
    notice: Option<String>,
    /// The last rate-limit note shown, so it is said once per change.
    last_rate_note: Option<String>,
    /// `#123` / `!45` link prefixes, cached against the inputs they come from.
    token_bases: RefCell<Option<(String, crate::issue_tokens::TokenBases)>>,
    /// The references of the selected commit's message, for the palette's
    /// "Open Commit Reference" (set every frame by the event loop).
    commit_references: Vec<(String, String)>,
    /// Builds the clients that list projects; `None` in tests without one.
    factory: Option<Arc<dyn crate::hosting_task::ClientFactory>>,
    /// Profiles offered by "Use integration profile...", refreshed with
    /// every request.
    profiles: Vec<(String, String)>,
    /// Writes to the local Markdown folder in flight.
    local_jobs: Vec<local::LocalJob>,
    /// The linked local folder's change watch.
    local_watch: local::LocalWatch,
    /// A request has been sent and not answered.
    in_flight: bool,
    /// The first request has been sent (the open gate is spent).
    opened_once: bool,
    /// When the request in flight was sent (drives the "Loading..." row).
    in_flight_since: Option<Instant>,
    /// The clock, replaced in tests.
    now: fn() -> i64,
    /// Production only: the first request waits until the first graph
    /// frame is on screen (see [`OPEN_TICKS_BEFORE_REQUEST`]).
    defer_open: bool,
    /// `Some(ticks seen)` while the first request is held back.
    open_gate: Option<u8>,
}

mod actions;
mod board;
mod item;
mod local;
mod meta;
mod picker;
mod push_review;
mod review;
mod rewrite_host;
mod rows;
mod sync;

#[cfg(test)]
#[cfg(all(feature = "github", feature = "gitlab"))]
mod tests;
