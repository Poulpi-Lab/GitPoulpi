//! How much history the Log asks the daemon for, from the user's config.

/// How much history the Log asks the daemon for. defaulted here;
/// the frontend's daemon link is the only consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogLoadSettings {
    /// How many commits to fetch for the graph up front ("Initial Commits in
    /// Graph").
    pub initial_commits: usize,
    /// "Lazy load commits in graph": additional commits are loaded
    /// when you reach the earliest commit in the graph. Off, you get
    /// [`initial_commits`] and nothing more.
    ///
    /// [`initial_commits`]: LogLoadSettings::initial_commits
    pub lazy_load: bool,
    /// How many commits one lazily-loaded page fetches. Only consulted
    /// when [`lazy_load`](LogLoadSettings::lazy_load) is on.
    pub page_commits: usize,
    /// The ceiling on how many commits the client will ever hold, or
    /// `None` for no ceiling. `general.max_commits_in_graph`.
    pub max_commits: Option<usize>,
}

// The numbers live in `gitpoulpi-config`, which the Preferences screen reads
// and writes, and are aliased here rather than restated so there is one
// default of each in the workspace.
impl LogLoadSettings {
    /// The "Initial commits in graph" default (5000, minimum 500): far
    /// more than any window shows, so scrolling always has somewhere to go,
    /// while staying bounded on huge repositories. 5000 `CommitNode`s are a few
    /// megabytes per session; later pages load as the scroll nears the end.
    pub const DEFAULT_INITIAL_COMMITS: usize =
        gitpoulpi_config::DEFAULT_INITIAL_COMMITS_IN_GRAPH as usize;

    /// The floor a configured value is clamped to: a window can be 200+ rows
    /// on a large display, and the initial page must still fill it without
    /// knowing the window size at request time.
    pub const MIN_INITIAL_COMMITS: usize = gitpoulpi_config::MIN_INITIAL_COMMITS_IN_GRAPH as usize;

    /// One lazily-loaded page; lives in `gitpoulpi-config` like the two above.
    pub const DEFAULT_PAGE_COMMITS: usize =
        gitpoulpi_config::DEFAULT_COMMITS_PER_LAZY_PAGE as usize;

    /// Lazy loading is **on** by default: history is
    /// effectively unbounded up to [`max_commits`](LogLoadSettings::max_commits)
    /// while only the initial page is paid for up front.
    pub const DEFAULT_LAZY_LOAD: bool = true;

    /// The shipped ceiling; see [`gitpoulpi_config::DEFAULT_MAX_COMMITS_IN_GRAPH`].
    pub const DEFAULT_MAX_COMMITS: usize = gitpoulpi_config::DEFAULT_MAX_COMMITS_IN_GRAPH as usize;

    /// Clamps a configured count into the supported range.
    pub fn with_initial_commits(count: usize) -> LogLoadSettings {
        LogLoadSettings {
            initial_commits: count.max(Self::MIN_INITIAL_COMMITS),
            ..LogLoadSettings::default()
        }
    }

    /// Everything this module needs from the user's config. The clamping is
    /// `gitpoulpi-config`'s, not a second copy.
    pub fn from_config(general: &gitpoulpi_config::GeneralConfig) -> LogLoadSettings {
        LogLoadSettings {
            initial_commits: general.initial_commits_in_graph_clamped() as usize,
            lazy_load: general.lazy_load_commits_in_graph,
            page_commits: general.commits_per_lazy_page_clamped() as usize,
            max_commits: general.max_commits_in_graph_effective().map(|n| n as usize),
        }
    }
}

impl Default for LogLoadSettings {
    fn default() -> LogLoadSettings {
        LogLoadSettings {
            initial_commits: Self::DEFAULT_INITIAL_COMMITS,
            lazy_load: Self::DEFAULT_LAZY_LOAD,
            page_commits: Self::DEFAULT_PAGE_COMMITS,
            max_commits: Some(Self::DEFAULT_MAX_COMMITS),
        }
    }
}
