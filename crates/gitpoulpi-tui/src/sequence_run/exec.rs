//! How a step is executed: the pure decisions.
//!
//! The command layer needs the window (`CommandCtx`), so the executing itself
//! lives in `event_loop/window/sequence.rs`. What does not need a window is
//! here: resolving a step id to its palette command, which kind of step it
//! is, which submitted action counts as its own, and the template variables.

use crate::daemon_link::Action;
use crate::palette::{command_id, PaletteCommand, PALETTE_COMMANDS};
use gitpoulpi_config::sequences::{expand_template, TemplateVars};
use gitpoulpi_config::AiAction;

/// The palette command of a step id, and its label in the UI language.
pub(crate) fn resolve(id: &str) -> (Option<PaletteCommand>, String) {
    match PALETTE_COMMANDS
        .iter()
        .find(|e| command_id(&e.command) == id)
    {
        Some(entry) => (Some(entry.command), entry.label.get().to_string()),
        None => (None, id.replace('_', " ")),
    }
}

/// What kind of step this is, i.e. how the glue runs and waits for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Plan {
    /// Stage everything.
    Stage,
    /// Commit (the composer's message, or `options.message`).
    Commit,
    /// Push the current branch.
    Push,
    /// Force push with lease (always asks).
    ForcePush,
    /// An AI request whose end the run waits for.
    Ai(AiAction),
    /// The AI composer's grouping into commits.
    Compose,
    /// The stash message prompt (a dialog the person answers).
    StashMessage,
    /// Create a worktree; `choose` starts from a branch of `options.branch`.
    Worktree { choose: bool },
    /// Open a worktree as a tab.
    OpenWorktreeInTab,
    /// Open the AI agent in the tab's folder.
    AgentHere,
    /// Open the tab's folder with an Open in entry, by label.
    OpenInLabel,
    /// Returns at once, nothing to wait for.
    Instant,
    /// Any other palette command: run it, the first action it submits is the
    /// step's.
    Action,
}

pub(crate) fn plan(id: &str) -> Plan {
    match id {
        "stage_all" => Plan::Stage,
        "commit" | "commit_now" => Plan::Commit,
        "push" => Plan::Push,
        "force_push_with_lease" => Plan::ForcePush,
        "ai_generate_commit_message" => Plan::Ai(AiAction::CommitMessage),
        "ai_compose_commits" => Plan::Compose,
        "ai_generate_stash_message" => Plan::StashMessage,
        "create_worktree_from_current_branch" => Plan::Worktree { choose: false },
        "create_worktree_choose_branch" => Plan::Worktree { choose: true },
        "open_worktree_in_tab" => Plan::OpenWorktreeInTab,
        "tab_open_agent_here" => Plan::AgentHere,
        "tab_open_in" => Plan::OpenInLabel,
        "tab_dir_terminal" | "tab_open_in_default" => Plan::Instant,
        _ => Plan::Action,
    }
}

/// Which submitted action counts as the step's own (so a stray stage click
/// during a pause cannot take the ticket of a commit).
pub(crate) fn accepts(plan: Plan) -> fn(&Action) -> bool {
    match plan {
        Plan::Stage => |a| matches!(a, Action::StageAll),
        Plan::Commit => |a| matches!(a, Action::Commit { .. }),
        Plan::Push | Plan::ForcePush => {
            |a| matches!(a, Action::Push { .. } | Action::PullThenPush { .. })
        }
        Plan::Compose => |a| matches!(a, Action::ComposeCommits { .. }),
        _ => |_| true,
    }
}

/// The variables of a template, from the repository's current branch.
pub(crate) fn expand(template: &str, branch: &str, exists: &dyn Fn(&str) -> bool) -> String {
    let now = gitpoulpi_core::time::unix_now();
    let secs = now.rem_euclid(86_400);
    let time = format!("{:02}-{:02}", secs / 3600, (secs % 3600) / 60);
    let date = gitpoulpi_core::time::civil_date(now);
    expand_template(
        template,
        &TemplateVars {
            branch,
            date: &date,
            time: &time,
        },
        exists,
    )
}
