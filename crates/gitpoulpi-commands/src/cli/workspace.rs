#[cfg(feature = "workspaces")]
use clap::Subcommand;
#[cfg(any(feature = "workspaces", feature = "global-search"))]
use clap::ValueEnum;

#[cfg(feature = "workspaces")]
/// What `gitpoulpi workspace` does. The workspaces live in
/// `<config>/workspaces.toml`, shared by every session and by the TUI.
#[derive(Debug, Subcommand)]
pub enum WorkspaceAction {
    /// The workspaces, with their repositories and default profile.
    List,
    /// One workspace: its repositories (and whether each is on disk), its
    /// recorded origin URLs and its default profile.
    Show {
        /// The workspace id (the slug `workspace list` prints).
        id: String,
    },
    /// Creates a workspace and prints it, with its new id. Each `--add` is
    /// recorded with its `origin` URL.
    Create {
        /// What people call it. The id is a slug made from it.
        name: String,
        /// A repository to put in it (repeatable). Not `--repo`: that one picks the
        /// repository a command runs in.
        #[arg(long = "add", value_name = "PATH")]
        repos: Vec<String>,
        /// The default commit profile (a `[[profiles]]` id).
        #[arg(long)]
        profile: Option<String>,
        /// A theme accent name for its dot in the tab bar.
        #[arg(long)]
        color: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
    /// Changes a workspace's name; its id stays.
    Rename { id: String, name: String },
    /// Deletes a workspace. The repositories are not touched.
    Delete { id: String },
    /// Adds repositories to a workspace, recording each one's `origin` URL.
    Add {
        id: String,
        #[arg(required = true, value_name = "PATH")]
        paths: Vec<String>,
    },
    /// Takes repositories out of a workspace. Nothing on disk changes.
    Remove {
        id: String,
        #[arg(required = true, value_name = "PATH")]
        paths: Vec<String>,
    },
    /// Sets (or with `--none` clears) the profile commits are made as in the
    /// workspace's repositories.
    SetProfile {
        id: String,
        /// A `[[profiles]]` id. A name no profile has is accepted (the
        /// profile may come later); it just does not apply.
        #[arg(required_unless_present = "none", conflicts_with = "none")]
        profile: Option<String>,
        /// Clear the default profile.
        #[arg(long)]
        none: bool,
    },
    /// Sets (or with `--clear` removes) the AI instructions of a workspace:
    /// for every AI action, or for one with `--action`. They apply to every
    /// repository of the workspace, after the global instructions and before
    /// the repository's own. See `ai prompt` for the result.
    SetAiInstructions {
        id: String,
        /// Only for this AI action (`commit_message`, `code_review`, ...).
        /// Without it, the instructions apply to every action.
        #[arg(long, value_parser = ai_action_key)]
        action: Option<String>,
        /// How they combine with the global instructions: `append` adds
        /// after them, `replace` is used instead (and drops them).
        #[arg(long, value_enum, default_value = "append")]
        mode: InstructionModeArg,
        /// The instructions.
        #[arg(long, required_unless_present = "clear", conflicts_with = "clear")]
        text: Option<String>,
        /// Remove the instructions (of the action, or the general ones).
        #[arg(long)]
        clear: bool,
    },
}

/// How workspace instructions combine with the ones above them.
#[cfg(feature = "workspaces")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum InstructionModeArg {
    /// Added after the global instructions.
    Append,
    /// Used instead of the global instructions.
    Replace,
}

/// Checks an AI action key against the known ones.
#[cfg(feature = "workspaces")]
fn ai_action_key(value: &str) -> Result<String, String> {
    gitpoulpi_config::AiAction::from_key(value)
        .filter(|a| crate::build_features::ai_action_offered(*a))
        .map(|_| value.to_string())
        .ok_or_else(|| {
            format!(
                "unknown AI action `{value}` (one of: {})",
                gitpoulpi_config::AiAction::ALL
                    .iter()
                    .filter(|a| crate::build_features::ai_action_offered(**a))
                    .map(|a| a.key())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
}

/// The kinds `gitpoulpi search --kind` takes.
#[cfg(feature = "global-search")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SearchKindArg {
    Repo,
    Branch,
    Tag,
    Commit,
    Stash,
    Worktree,
    // Pull and merge requests, from the hosting cache.
    #[value(alias = "pull_request", alias = "pr", alias = "mr")]
    PullRequest,
    // Issues, from the hosting cache.
    Issue,
}

#[cfg(feature = "global-search")]
impl From<SearchKindArg> for gitpoulpi_core::domain::SearchKind {
    fn from(arg: SearchKindArg) -> Self {
        use gitpoulpi_core::domain::SearchKind as K;
        match arg {
            SearchKindArg::Repo => K::Repo,
            SearchKindArg::Branch => K::Branch,
            SearchKindArg::Tag => K::Tag,
            SearchKindArg::Commit => K::Commit,
            SearchKindArg::Stash => K::Stash,
            SearchKindArg::Worktree => K::Worktree,
            SearchKindArg::PullRequest => K::PullRequest,
            SearchKindArg::Issue => K::Issue,
        }
    }
}
