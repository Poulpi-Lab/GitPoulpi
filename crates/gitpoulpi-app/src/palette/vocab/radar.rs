/// The columns of the Radar's list that can be sized (`radar::Cell`'s sizable
/// ones; this file cannot name that type, the stub build has none).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizedColumn {
    Repo,
    Branches,
    Num,
    Author,
    Age,
}

/// What a row's menu, a key or a palette command asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
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
pub enum RadarCommand {
    Open,
    Refresh,
    Search,
    Act(Act),
    Scope,
    View,
    Filter,
    /// Opens the Profile combobox (`P`).
    Profile,
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
pub enum ColumnCmd {
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
