#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoMenuAction {
    /// Focus its tab, or open it as one.
    Open,
    Close,
    CloseOthers,
    CloseToTheRight,
    ToggleFavorite,
    RemoveFromRecent,
    /// Forget the repository in every stored list (recent, favorites).
    RemoveFromList,
    CopyPath,
    OpenInFileManager,
    OpenInTerminal,
    /// Entry `n` of the "Open in" list (`tab_dir::entries()`), run on this
    /// repository: the "Open in" submenu's rows.
    OpenIn(usize),
    /// This repository's SSH & credentials page (it also shows a
    /// `core.sshCommand` pin and moves it into GitPoulpi).
    SshKey,
    /// The Sparse checkout dialog of this repository.
    Sparse,
    /// A transient "New tab" (the tab bar's `+`); the tab menu's first row.
    NewTab,
    /// The `+` menu's Radar row, offered while its tab is closed.
    OpenRadar,
    /// The `+` menu's Workspaces row, offered while its tab is closed.
    OpenWorkspaces,
    /// Add this repository to the active workspace.
    AddToWorkspace,
    /// Take this repository out of the active workspace (it stays a tab).
    RemoveFromWorkspace,
    /// A row of a workspace's (or a workspace repository's) menu on the Home
    /// tab's Workspaces page.
    Ws(WsMenu),
}

/// The rows of the Workspaces page's menus (`workspace_ui::menu`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsMenu {
    Open,
    Edit,
    Delete,
    FetchAll,
    PullAll,
    CloneMissing,
    OpenRepo,
    Locate,
    CloneRepo,
    RemoveRepo,
}
