//! What this build was compiled with (custom builds), and the one filter
//! every registry (keymap, palette, Settings pages) applies to entries tagged
//! with a feature. Shared by every frontend: each forwards its cargo
//! features of the same names to this crate.
//!
//! A tag never removes an enum variant or a config key: a user's
//! `keybindings.toml` entry for an action this build lacks still parses, the
//! action is just not listed or run.

/// A feature a custom build can leave out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    Ai,
    Hosting,
    Mcp,
    Lfs,
    Worktree,
    Submodule,
    ImageView,
    Radar,
    Mermaid,
    Board,
    Review,
    Conversation,
    HistoryRewrite,
    GlobalSearch,
    Workspaces,
}

impl Feature {
    pub const ALL: [Feature; 15] = [
        Feature::Ai,
        Feature::Hosting,
        Feature::Mcp,
        Feature::Lfs,
        Feature::Worktree,
        Feature::Submodule,
        Feature::ImageView,
        Feature::Radar,
        Feature::Mermaid,
        Feature::Board,
        Feature::Review,
        Feature::Conversation,
        Feature::HistoryRewrite,
        Feature::GlobalSearch,
        Feature::Workspaces,
    ];

    /// Whether this binary was built with the feature.
    pub const fn built(self) -> bool {
        match self {
            Feature::Ai => AI,
            Feature::Hosting => HOSTING,
            Feature::Mcp => MCP,
            Feature::Lfs => LFS,
            Feature::Worktree => WORKTREE,
            Feature::Submodule => SUBMODULE,
            Feature::ImageView => IMAGE_VIEW,
            Feature::Radar => RADAR,
            Feature::Mermaid => MERMAID,
            Feature::Board => BOARD,
            Feature::Review => REVIEW,
            Feature::Conversation => CONVERSATION,
            Feature::HistoryRewrite => HISTORY_REWRITE,
            Feature::GlobalSearch => GLOBAL_SEARCH,
            Feature::Workspaces => WORKSPACES,
        }
    }

    /// The cargo feature name.
    pub const fn name(self) -> &'static str {
        match self {
            Feature::Ai => "ai",
            Feature::Hosting => "hosting",
            Feature::Mcp => "mcp",
            Feature::Lfs => "lfs",
            Feature::Worktree => "worktree",
            Feature::Submodule => "submodule",
            Feature::ImageView => "image-view",
            Feature::Radar => "radar",
            Feature::Mermaid => "mermaid",
            Feature::Board => "board",
            Feature::Review => "review",
            Feature::Conversation => "conversation",
            Feature::HistoryRewrite => "history-rewrite",
            Feature::GlobalSearch => "global-search",
            Feature::Workspaces => "workspaces",
        }
    }
}

pub const AI: bool = cfg!(feature = "ai");
pub const HOSTING: bool = cfg!(feature = "hosting");
pub const GITHUB: bool = cfg!(feature = "github");
pub const GITLAB: bool = cfg!(feature = "gitlab");
pub const AZURE_DEVOPS: bool = cfg!(feature = "azure-devops");
pub const BITBUCKET: bool = cfg!(feature = "bitbucket");
pub const JIRA: bool = cfg!(feature = "jira");
pub const TRELLO: bool = cfg!(feature = "trello");
pub const GITEA: bool = cfg!(feature = "gitea");
pub const LINEAR: bool = cfg!(feature = "linear");
pub const YOUTRACK: bool = cfg!(feature = "youtrack");
pub const REDMINE: bool = cfg!(feature = "redmine");
pub const GERRIT: bool = cfg!(feature = "gerrit");
pub const LOCAL_ISSUES: bool = cfg!(feature = "local-issues");
pub const MCP: bool = cfg!(feature = "mcp");
pub const LFS: bool = cfg!(feature = "lfs");
pub const WORKTREE: bool = cfg!(feature = "worktree");
pub const SUBMODULE: bool = cfg!(feature = "submodule");
pub const AVATAR_FETCH: bool = cfg!(feature = "avatar-fetch");
pub const IMAGE_VIEW: bool = cfg!(feature = "image-view");
pub const RADAR: bool = cfg!(all(feature = "radar", feature = "hosting"));
pub const MERMAID: bool = cfg!(feature = "mermaid");
pub const BOARD: bool = cfg!(all(feature = "board", feature = "hosting"));
pub const REVIEW: bool = cfg!(all(feature = "review", feature = "hosting"));
pub const CONVERSATION: bool = cfg!(all(feature = "conversation", feature = "hosting"));
pub const HISTORY_REWRITE: bool = cfg!(feature = "history-rewrite");
pub const GLOBAL_SEARCH: bool = cfg!(feature = "global-search");
pub const WORKSPACES: bool = cfg!(feature = "workspaces");
/// The embedded Lua runtime behind the history rewrite's Script rule.
pub const REWRITE_SCRIPTS: bool = cfg!(all(
    feature = "history-rewrite",
    feature = "rewrite-scripts"
));

/// Whether this build has the client of an API family.
pub const fn family_built(family: gitpoulpi_config::ApiFamily) -> bool {
    use gitpoulpi_config::ApiFamily as F;
    match family {
        F::Github => GITHUB,
        F::Gitlab => GITLAB,
        F::AzureDevops => AZURE_DEVOPS,
        F::Bitbucket => BITBUCKET,
        F::Jira => JIRA,
        F::Trello => TRELLO,
        F::Gitea => GITEA,
        F::Linear => LINEAR,
        F::YouTrack => YOUTRACK,
        F::Redmine => REDMINE,
        F::Gerrit => GERRIT,
        F::LocalMarkdown => LOCAL_ISSUES,
    }
}

/// Whether this build has a client for `kind` (a profile of a kind that is
/// not built stays in the config but is never used).
pub fn kind_built(kind: gitpoulpi_config::IntegrationKind) -> bool {
    gitpoulpi_config::IntegrationKind::IMPLEMENTED.contains(&kind) && family_built(kind.family())
}

/// The user-facing feature table, in `--version` order.
pub const FLAGS: [(&str, bool); 27] = [
    ("ai", AI),
    ("github", GITHUB),
    ("gitlab", GITLAB),
    ("azure-devops", AZURE_DEVOPS),
    ("bitbucket", BITBUCKET),
    ("jira", JIRA),
    ("trello", TRELLO),
    ("gitea", GITEA),
    ("linear", LINEAR),
    ("youtrack", YOUTRACK),
    ("redmine", REDMINE),
    ("gerrit", GERRIT),
    ("local-issues", LOCAL_ISSUES),
    ("mcp", MCP),
    ("lfs", LFS),
    ("worktree", WORKTREE),
    ("submodule", SUBMODULE),
    ("avatar-fetch", AVATAR_FETCH),
    ("image-view", IMAGE_VIEW),
    ("radar", RADAR),
    ("mermaid", MERMAID),
    ("board", BOARD),
    ("review", REVIEW),
    ("conversation", CONVERSATION),
    ("history-rewrite", HISTORY_REWRITE),
    ("global-search", GLOBAL_SEARCH),
    ("workspaces", WORKSPACES),
];

/// Whether something tagged `requires` is offered, given which features
/// `built` reports. Untagged is always offered. Registries call
/// [`offered`]; tests call this with a fake table.
pub fn offered_with(requires: Option<Feature>, built: impl Fn(Feature) -> bool) -> bool {
    requires.is_none_or(built)
}

/// [`offered_with`] for this build.
pub fn offered(requires: Option<Feature>) -> bool {
    offered_with(requires, |f| {
        f.built() && (f != Feature::Radar || radar_enabled())
    })
}

/// `general.radar_enabled`, published like the other live settings: the
/// one switch every Radar surface (tab, palette entries, keys, background
/// fetches) reads through [`offered`] / [`radar_enabled`].
///
/// A test publishes it for its own thread only ([`install_radar_enabled_local`])
/// so parallel tests never see each other's value.
static RADAR_ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

thread_local! {
    static RADAR_ENABLED_LOCAL: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

pub fn install_radar_enabled(on: bool) {
    RADAR_ENABLED.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// [`install_radar_enabled`] for the calling thread alone (unit tests).
pub fn install_radar_enabled_local(on: bool) {
    RADAR_ENABLED_LOCAL.with(|c| c.set(Some(on)));
}

/// Whether the user left the Radar on (the build having it is
/// [`Feature::built`]'s question).
pub fn radar_enabled() -> bool {
    RADAR_ENABLED_LOCAL
        .with(std::cell::Cell::get)
        .unwrap_or_else(|| RADAR_ENABLED.load(std::sync::atomic::Ordering::Relaxed))
}
