//! The shared command layer: clap's `Cli` tree, the IPC `Session` client,
//! and one `execute` per subcommand's `GitBackend` mapping.
//!
//! Extracted from `gitpoulpi-cli` so the MCP server (`gitpoulpi-mcp`) can
//! generate its tools from the same `Cli` tree and run them through the
//! same `execute`: no drift between what a human types and what an agent
//! calls, by construction.

#[cfg(all(
    feature = "hosting",
    not(any(
        feature = "github",
        feature = "gitlab",
        feature = "azure-devops",
        feature = "bitbucket",
        feature = "jira",
        feature = "trello",
        feature = "gitea",
        feature = "linear",
        feature = "youtrack",
        feature = "redmine",
        feature = "gerrit",
        feature = "local-issues"
    ))
))]
compile_error!(
    "the internal `hosting` feature needs a provider (`github`, `gitlab`, `azure-devops`, `bitbucket`, `jira`, `trello`, `gitea`, `linear`, `youtrack`, `redmine`, `gerrit` or `local-issues`); enable one of those instead"
);

#[cfg_attr(not(feature = "ai"), path = "ai_cmd_stub.rs")]
mod ai_cmd;
#[cfg(all(feature = "ai", feature = "hosting"))]
pub(crate) mod ai_conversation;
#[cfg(feature = "ai")]
pub(crate) mod ai_repo;
#[cfg(feature = "ai")]
pub(crate) mod ai_review_ctx;
pub mod alias_cmd;
pub mod build_features;
pub mod cli;
pub mod client;
pub mod commands;
pub mod config_cmd;
#[cfg(feature = "hosting")]
pub mod hosting_cmd;
pub mod installation_cmd;
pub mod job_cmd;
pub mod launch;
#[cfg(feature = "mcp")]
pub mod mcp_cmd;
pub mod repo_cmd;
#[cfg(feature = "global-search")]
pub mod search_cmd;
pub mod session_cmd;
pub mod shell_init;
pub mod theme_cmd;
mod ui_cmd;
mod ui_launch;
#[cfg(feature = "workspaces")]
pub mod workspace_cmd;

pub use client::{discover_repo, Session};
pub use commands::{execute, run, Emitted};
