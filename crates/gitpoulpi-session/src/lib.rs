mod clone_source;
mod commit_template;
mod favorites;
mod global;
mod radar;
mod recent;
mod recipe_file;
mod remote_reviews;
mod repo_auth;
mod repo_state;
mod review_draft;
mod workflow_file;
mod workspaces;

pub use clone_source::CloneSource;
pub use commit_template::{
    join_template, own_template_path, read_template, split_template, template_path, write_template,
    TEMPLATE_FILE,
};
pub use favorites::{FavoriteRepo, FavoriteRepos};
pub use gitpoulpi_config::RepoAiReview;
pub use global::{
    linked_worktree_main, repo_path_key, same_repo_path, GlobalSession, PaneLayout, TabEntry,
    WorkspaceTabs,
};
pub use recent::{now_seconds, RecentRepo, RecentRepos, MAX_RECENT};
pub use recipe_file::{
    delete_recipe, list_recipes, load_recipe, my_recipes_dir, recipe_path, recipe_slug,
    save_recipe, shared_recipes_dir, RecipeEntry, RecipePlace, RECIPES_DIR,
};
pub use remote_reviews::RemoteReviews;
pub use repo_auth::{
    clear_repo_auth, load_repo_auth, repo_auth_key, save_repo_auth, RepoAuthRecord, RepoAuthStore,
    REPO_AUTH_FILE,
};
pub use repo_state::{
    conversation_key, resolve_default_remote, ConversationSeen, FoldState, HookLine, IssuesSource,
    LogColumnWidths, PendingAutoStash, PendingAutoStashState, RepoPreferences, RepoState,
    TrustedHooks, TrustedScript, WatchedIssue,
};
pub use review_draft::{AiDraft, AiItem, AiPlace, DraftComment, ReviewDraft};

pub use radar::{
    ItemKey, RadarColumns, RadarState, Snooze, SnoozeFor, KIND_ISSUE, KIND_PR, KIND_WIP,
};
pub use workflow_file::{
    load_shared, render_toml, save_shared, workflow_path_for, LoadedWorkflow, SharedWorkflow,
    WORKFLOW_FILE,
};
pub use workspaces::{slugify, Workspace, Workspaces};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("toml serialize error: {0}")]
    Serialize(#[from] toml::ser::Error),
}
