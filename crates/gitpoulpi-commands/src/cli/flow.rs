use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum FlowAction {
    /// Every branch of the repository classified by its branching workflow:
    /// base or topic (with its type), parent, and how many commits it is
    /// ahead of and behind that parent. Without a workflow file every branch
    /// is unclassified; `flow detect` proposes one.
    Status,
    /// The built-in workflow templates (Git Flow, GitHub Flow, GitLab Flow,
    /// Trunk-based, Custom) with their base branches and topic types.
    Templates,
    /// Which template fits this repository's branches, ranked by score, with
    /// the concrete base names found (`master` for `main`, `dev` for
    /// `develop`).
    Detect,
    /// The repository's workflow: the shared `.gitpoulpi/workflow.toml`, or
    /// with `--show-effective` that file with the per-user override applied.
    /// `config init` writes a template.
    Config {
        /// Print the effective workflow (shared file plus the per-user
        /// override) instead of the shared file.
        #[arg(long)]
        show_effective: bool,
        #[command(subcommand)]
        action: Option<FlowConfigAction>,
    },
    /// Start a topic branch of a workflow type: names it from the type's
    /// pattern (`feature/{slug}`, `release/{version}`, ...), creates it from
    /// the type's parent (or `--from`), checks it out and records the parent.
    /// Prints the steps it will do; it runs them only with `--yes` (or after
    /// the interactive question on a terminal). Undo takes it all back.
    Start {
        /// The branch type (`feature`, `hotfix`, `release`, ...): see
        /// `flow config --show-effective`.
        r#type: String,
        /// Values for the pattern's tokens as `name=value` (`slug=login`,
        /// `version=1.2.0`, `issue=42`); a bare word is the `slug`. `user`
        /// and `date` default to you and today.
        fields: Vec<String>,
        /// Start from this branch, remote branch or tag instead of the
        /// type's parent.
        #[arg(long)]
        from: Option<String>,
        /// Push the new branch and set its upstream.
        #[arg(long)]
        publish: bool,
        #[command(flatten)]
        run: FlowRunArgs,
    },
    /// Finish a topic branch: for each finish target of its type, in order,
    /// merge it (merge commit, squash, rebase or fast-forward), tag when the
    /// type says so, then delete the branch and go back to its parent. With
    /// the `merge-request` finish mode it pushes the branch and opens the
    /// pull/merge request instead (not undoable; nothing is merged or
    /// deleted locally). A merge that stops on conflicts leaves the normal
    /// conflict state: resolve it, then `--continue` (or `--abort`). Prints
    /// the plan and needs `--yes`; exit code 3 without it.
    Finish {
        /// The branch to finish. Defaults to the checked-out one.
        branch: Option<String>,
        /// `local-merge` or `merge-request` (needed when the type asks).
        #[arg(long, value_parser = ["local-merge", "merge-request"])]
        mode: Option<String>,
        /// The merge request title or the squash commit message.
        #[arg(long)]
        title: Option<String>,
        /// Delete the local branch afterwards (default: the type's setting).
        #[arg(long, conflicts_with = "keep")]
        delete: bool,
        /// Keep the local branch (override the type's `delete_after_finish`).
        #[arg(long)]
        keep: bool,
        /// Also delete the branch on its remote (needs the local delete;
        /// not undoable).
        #[arg(long)]
        delete_remote: bool,
        /// The remote to push to (default `origin`).
        #[arg(long)]
        remote: Option<String>,
        /// Continue although the target is behind its remote.
        #[arg(long)]
        allow_behind: bool,
        /// Merge-request mode: open it as a draft.
        #[arg(long)]
        draft: bool,
        /// Merge-request mode: the description.
        #[arg(long)]
        body: Option<String>,
        /// Merge-request mode: someone to assign; repeat or comma-separate.
        #[arg(long = "assignee", value_delimiter = ',')]
        assignees: Vec<String>,
        /// Merge-request mode: someone to ask for a review.
        #[arg(long = "reviewer", value_delimiter = ',')]
        reviewers: Vec<String>,
        /// Merge-request mode: a label to apply.
        #[arg(long = "label", value_delimiter = ',')]
        labels: Vec<String>,
        /// Merge-request mode: the milestone, by title.
        #[arg(long)]
        milestone: Option<String>,
        /// Merge-request mode: which hosting integration to use.
        #[arg(long = "integration")]
        integration: Option<String>,
        #[command(flatten)]
        run: FlowRunArgs,
        #[command(flatten)]
        resume: FlowResumeArgs,
    },
    /// Bring a branch up to date with its parent (merge or rebase, by the
    /// branch's configured strategy), and with `--stack` the branches built
    /// on it, in order. Prints the plan and needs `--yes`; exit code 3
    /// without it.
    Sync {
        /// The branch to update. Defaults to the checked-out one.
        branch: Option<String>,
        /// Also update the branches whose parent is this one, recursively.
        #[arg(long)]
        stack: bool,
        /// Rebase instead of merging.
        #[arg(long, conflicts_with = "merge")]
        rebase: bool,
        /// Merge instead of rebasing.
        #[arg(long)]
        merge: bool,
        #[command(flatten)]
        run: FlowRunArgs,
        #[command(flatten)]
        resume: FlowResumeArgs,
    },
    /// Promote an environment branch (GitLab Flow): merge the branch it is
    /// downstream of into it, one step. Prints the plan and needs `--yes`;
    /// exit code 3 without it.
    Promote {
        /// The environment branch to update (`pre-production`, `production`).
        branch: String,
        /// Continue although the branch is behind its remote.
        #[arg(long)]
        allow_behind: bool,
        #[command(flatten)]
        run: FlowRunArgs,
        #[command(flatten)]
        resume: FlowResumeArgs,
    },
    /// Record which branch a branch started from and is finished into
    /// (`branch.<name>.gitpoulpi-parent`), or forget it with `--clear`.
    /// Undoable.
    SetParent {
        /// The branch.
        branch: String,
        /// Its parent (an existing local branch).
        parent: Option<String>,
        /// Forget the recorded parent.
        #[arg(long, conflicts_with = "parent")]
        clear: bool,
        #[command(flatten)]
        run: FlowRunArgs,
    },
    /// Host-driven flows: make an issue from the workflow's template and let
    /// the host or a bot make the branch and the merge request, then pick
    /// them up. `issue new` creates the issue; `issue wait` polls for what
    /// the host made.
    Issue {
        #[command(subcommand)]
        action: FlowIssueAction,
    },
    /// Take over a branch the host or a bot made: fetches, creates the local
    /// branch tracking `origin/<branch>`, records its parent (the merge
    /// request's target), checks it out. One undoable entry. The target is
    /// `#12` (an issue: its branch or merge request is looked up on the
    /// host), `!5` (a merge request) or a branch name.
    PickUp {
        /// `#12` for an issue, `!5` for a merge request, or a branch name.
        target: String,
        /// The remote to fetch from and track (default `origin`).
        #[arg(long)]
        remote: Option<String>,
        /// The branch the merge request targets, recorded as the parent
        /// (default: the merge request's target, else the type's parent).
        #[arg(long)]
        parent: Option<String>,
        /// Do not fetch first (the branch must be known already).
        #[arg(long)]
        no_fetch: bool,
        /// Which hosting integration to ask.
        #[arg(long = "integration")]
        integration: Option<String>,
        #[command(flatten)]
        run: FlowRunArgs,
    },
    /// Review and trust the workflow hooks the repository commits (the
    /// `[hooks]` of `.gitpoulpi/workflow.toml`): shell commands that would run
    /// on your machine. Without `--hash` it only prints `{hash, hooks,
    /// trusted}`; with the hash it remembers the trust, but only when the
    /// file on disk still has exactly that hash, so what you read is what is
    /// trusted. A changed file asks again. Hooks of your own override never
    /// need this. Only a person can do this: there is no MCP tool for it.
    TrustHooks {
        /// The hash printed by `flow trust-hooks` (or by a refused
        /// `--run-hooks`): trust the hooks with that hash.
        #[arg(long)]
        hash: Option<String>,
    },
    /// Turn another tool's branching settings found in the repository
    /// (`git flow init`'s `gitflow.*`, Git Town's `git-town.*`) into a
    /// workflow. Prints the TOML; changes nothing unless `--write`, which
    /// only creates `.gitpoulpi/workflow.toml` when it does not exist yet.
    Import {
        /// Which tool's configuration to read.
        #[arg(long, value_parser = ["git-flow", "git-town"])]
        from: String,
        /// Create `.gitpoulpi/workflow.toml` (never overwrites).
        #[arg(long)]
        write: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum FlowIssueAction {
    /// Create an issue of a branch type on the linked tracker from the
    /// workflow's `[issue_flow]` (title and label templates, description
    /// file) and drive what follows by its mode: `host-creates-branch` makes
    /// the branch (and a draft merge request that closes the issue) on the
    /// host, `bot` waits for the one a bot makes (`--wait`), `local` makes
    /// nothing. `--pick-up` then takes the branch over. `--dry-run` prints
    /// what would be created and changes nothing.
    New {
        /// The branch type (`feature`, `bugfix`, ...): see `flow config
        /// --show-effective`.
        #[arg(long = "type")]
        r#type: String,
        /// The issue's title (the title template wraps it).
        #[arg(long)]
        title: String,
        /// The description, instead of the template file's text.
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        /// A file with the description, instead of the template file.
        #[arg(long)]
        body_file: Option<std::path::PathBuf>,
        /// More labels besides the workflow's.
        #[arg(long = "label", value_delimiter = ',')]
        labels: Vec<String>,
        /// More people to assign besides the workflow's.
        #[arg(long = "assignee", value_delimiter = ',')]
        assignees: Vec<String>,
        /// The milestone, by title.
        #[arg(long)]
        milestone: Option<String>,
        /// Bot mode: wait for the branch or merge request the bot makes.
        #[arg(long)]
        wait: bool,
        /// Then pick the branch up (waits first in bot mode; in local mode it
        /// starts the branch from the issue).
        #[arg(long)]
        pick_up: bool,
        /// How long to wait, in seconds (default: the workflow's, else 600).
        #[arg(long)]
        timeout: Option<u64>,
        /// Which hosting integration to use.
        #[arg(long = "integration")]
        integration: Option<String>,
        #[command(flatten)]
        run: FlowRunArgs,
    },
    /// Wait for what the host or a bot made for an issue: polls the host
    /// (its own links, branches matching `branch_from_issue`) with a growing
    /// pause until the workflow's `wait_for` is satisfied or the timeout
    /// ends (exit code 4). Ctrl+C cancels. `--pick-up` then takes the branch
    /// over.
    Wait {
        /// The issue: `#12` or `12`.
        reference: String,
        /// Only this branch type's pattern (default: every type).
        #[arg(long = "type")]
        r#type: Option<String>,
        /// What to wait for: `branch`, `mr` or `branch-or-mr` (default: the
        /// workflow's).
        #[arg(long = "for", value_parser = ["branch", "mr", "branch-or-mr"])]
        wait_for: Option<String>,
        /// How long to wait, in seconds (default: the workflow's, else 600).
        #[arg(long)]
        timeout: Option<u64>,
        /// Look once and answer, do not wait.
        #[arg(long)]
        once: bool,
        /// Then pick the branch up.
        #[arg(long)]
        pick_up: bool,
        /// Which hosting integration to use.
        #[arg(long = "integration")]
        integration: Option<String>,
        #[command(flatten)]
        run: FlowRunArgs,
    },
}

#[derive(Debug, Subcommand)]
pub enum FlowConfigAction {
    /// Print the shared workflow file (or, with `--show-effective`, the
    /// effective workflow). Same as `flow config`.
    Show {
        /// Print the effective workflow instead of the shared file.
        #[arg(long)]
        show_effective: bool,
    },
    /// Print the TOML of a template. With `--write`, create
    /// `.gitpoulpi/workflow.toml` from it, only when that file does not
    /// exist yet.
    Init {
        /// The template to start from.
        #[arg(long, value_parser = ["git-flow", "github-flow", "gitlab-flow", "trunk", "custom"])]
        template: String,
        /// Create `.gitpoulpi/workflow.toml` (never overwrites).
        #[arg(long)]
        write: bool,
    },
}

/// How a workflow action is confirmed and what it may run.
#[derive(Debug, Clone, Default, Args)]
pub struct FlowRunArgs {
    /// Print the plan (every step, blockers, warnings) as JSON and change
    /// nothing.
    #[arg(long)]
    pub dry_run: bool,
    /// Run the plan without asking. Without it a terminal is asked, anything
    /// else prints the plan and exits with code 3.
    #[arg(long)]
    pub yes: bool,
    /// Run the hooks the workflow configures for this action (shell commands
    /// from the workflow file, shown with their output). Never run unless
    /// asked for; a terminal is asked when this is absent.
    #[arg(long, conflicts_with = "no_hooks")]
    pub run_hooks: bool,
    /// Skip the configured hooks without asking.
    #[arg(long)]
    pub no_hooks: bool,
    /// Stash local changes that block a checkout or merge and put them back
    /// afterwards, whatever `general.auto_stash` says.
    #[arg(long)]
    pub autostash: bool,
}

/// Resuming a flow a conflict (or a failed hook) stopped.
#[derive(Debug, Clone, Default, Args)]
pub struct FlowResumeArgs {
    /// Continue after the conflict was resolved: commits the merge or continues
    /// the rebase, then runs the remaining steps. After a failed hook, runs
    /// that step again.
    #[arg(long = "continue", conflicts_with = "abort")]
    pub continue_: bool,
    /// Give up on the stopped action: abort the merge or rebase in progress and
    /// return to the branch it started on. Steps that already completed stay
    /// (take them back with `undo`).
    #[arg(long)]
    pub abort: bool,
}
