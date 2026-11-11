use clap::{Parser, Subcommand};

mod conflicts;
mod flow;
mod help_lang;
#[cfg(feature = "hosting")]
mod hosting;
mod job;
#[cfg(feature = "lfs")]
mod lfs;
mod mailmap;
mod refs_misc;
mod repo_auth;
#[cfg(feature = "history-rewrite")]
mod rewrite;
mod ui;
mod workspace;

pub use conflicts::{ConflictSideArg, ConflictsAction, IndexLockAction};
pub use flow::{FlowAction, FlowConfigAction, FlowIssueAction, FlowResumeArgs, FlowRunArgs};
pub use help_lang::{
    catalog, help_keys, parse_localized, prescan_lang, translate_command, HelpKey, HEADING_KEYS,
};
#[cfg(all(feature = "board", feature = "hosting"))]
pub use hosting::BoardAction;
#[cfg(feature = "hosting")]
pub use hosting::{
    FileArgs, HostingAction, IntegrationsAction, IssueAction, IssueMetaArgs, LinkKind, OpenKind,
    RadarWhat, RemoteReviewArg, ReviewAction, ReviewEventArg, SshKeyAction, StateArg,
};
pub use job::JobAction;
#[cfg(feature = "lfs")]
pub use lfs::LfsAction;
pub use mailmap::MailmapAction;
#[cfg(feature = "ai")]
pub use refs_misc::AiCommand;
#[cfg(feature = "mcp")]
pub use refs_misc::McpAction;
#[cfg(feature = "submodule")]
pub use refs_misc::SubmoduleAction;
#[cfg(feature = "worktree")]
pub use refs_misc::WorktreeAction;
pub use refs_misc::{
    AliasAction, AutoStashAction, BranchAction, ConfigAction, DaemonAction, DesktopAction, OnOff,
    PatchAction, PathAction, PullModeArg, ReflogModeArg, RefsAction, RemoteAction, RepoAction,
    RepoConfigAction, ResetModeArg, ReviewFlags, RunInRepoArg, SessionAction, ShortcutAction,
    SparseAction, SshCommandAction, StashAction, TagAction, ThemeAction,
};
pub use repo_auth::RepoAuthAction;
#[cfg(feature = "history-rewrite")]
pub use rewrite::{RewriteAction, RewriteRecipeArgs, RewriteScriptAction};
pub use ui::{UiAction, UiProposeAction, UiShowAction};
#[cfg(feature = "global-search")]
pub use workspace::SearchKindArg;
#[cfg(feature = "workspaces")]
pub use workspace::{InstructionModeArg, WorkspaceAction};

#[derive(Debug, Parser)]
#[command(
    name = "gitpoulpi",
    version = gitpoulpi_ipc::VERSION_STRING,
    long_version = crate::build_features::long_version_static(),
    about = "Lightweight cross-platform TUI Git client, scriptable via CLI/MCP."
)]
pub struct Cli {
    /// Named session to target. Omitted, it is derived from the current
    /// repository's root (see `gitpoulpi_config::default_session_name`),
    /// so two repositories get two independent daemons while everything
    /// opened inside one repository shares one. Pass this to force two
    /// repositories onto one session, or to name a session explicitly.
    /// The value is sanitised before use - it becomes a filename.
    #[arg(long, short = 's', global = true)]
    pub session: Option<String>,

    /// Emit structured JSON instead of human-readable output.
    #[arg(long, global = true)]
    pub json: bool,

    /// Repository to act on. Defaults to the repository enclosing the
    /// current directory, so `gitpoulpi status` behaves like `git status`;
    /// an agent driving several repos passes this instead of chdir-ing.
    #[arg(long, global = true)]
    pub repo: Option<String>,

    /// Which `GitBackend` this command's daemon runs on, overriding
    /// `general.default_backend` in config.toml and
    /// `GITPOULPI_GENERAL_DEFAULT_BACKEND` (flags are the top of the
    /// precedence ladder - see docs/user/config-reference.md).
    ///
    /// It selects the backend for a daemon this command *starts*. A
    /// session whose daemon is already running keeps the backend that
    /// daemon was built with; `gitpoulpi` has no way to swap a live
    /// daemon's backend out from under the clients already subscribed to
    /// it, and pretending otherwise would be the same silent no-op this
    /// flag exists to avoid.
    #[arg(long, global = true, value_enum)]
    pub backend: Option<BackendArg>,

    /// File the TUI writes the active tab's folder to when it quits
    /// ("Quit and cd here", or always with `ui.tab_dir_follow = "auto"`).
    /// Meant for the wrapper `gitpoulpi shell-init` prints; ignored by every
    /// subcommand, since only the TUI has tabs.
    #[arg(long, global = true, value_name = "FILE")]
    pub cwd_file: Option<std::path::PathBuf>,

    /// Set by the desktop and Start menu shortcuts GitPoulpi writes: the tab
    /// the TUI opens on is then `general.startup_tab_launcher`. Ignored by
    /// every subcommand.
    #[arg(long, global = true, hide = true)]
    pub launcher: bool,

    /// Open a folder as a new tab of the GitPoulpi that is already running,
    /// instead of starting another one (`gp -a`, `gp --add ../other`). PATH
    /// defaults to the current directory and resolves to its enclosing
    /// repository. One running session: it gets the tab and its window comes
    /// forward when the desktop allows it. Several: you pick one from a
    /// numbered list (or pass `--session NAME`). None: GitPoulpi starts
    /// normally, on that folder. Takes no subcommand; with `--json` it prints
    /// one object (`action` is `joined` or `cancelled`), exit code 0 when the
    /// tab is open, 3 when a busy window offered it on its status line chip.
    #[arg(long, short = 'a', num_args = 0..=1, value_name = "PATH")]
    pub add: Option<Option<String>>,

    /// With `--add`: do not ask the window to bring its terminal forward.
    #[arg(long, requires = "add")]
    pub no_raise: bool,

    /// Language of the `--help` text (en, fr, es, pt-BR, pt-PT, it, de, zh-CN, ja, ko, ru, uk, pl, cs, nl, sv, tr).
    ///
    /// Only explanations are translated - command and flag names
    /// never change - and `--help` stays English unless this is given
    /// (the environment is deliberately ignored, so scripts get stable
    /// output). Example: `gitpoulpi -l fr --help`.
    #[arg(long, short = 'l', global = true, value_name = "LANG")]
    pub lang: Option<String>,

    /// No subcommand launches the TUI for --session (default behavior,
    /// like `gitpoulpi`/`gp` with no arguments opening the app).
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// The command-line spelling of `general.default_backend`. A separate
/// type from `gitpoulpi_config::DefaultBackend` only because deriving
/// clap's `ValueEnum` on it would put a CLI dependency into the config
/// crate; the conversion below is what keeps the two spellings identical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum BackendArg {
    Git2,
    System,
}

impl From<BackendArg> for gitpoulpi_config::DefaultBackend {
    fn from(arg: BackendArg) -> Self {
        match arg {
            BackendArg::Git2 => Self::Git2,
            BackendArg::System => Self::System,
        }
    }
}

/// Output formats of `log --format`.
#[cfg(feature = "mermaid")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum LogFormatArg {
    Mermaid,
}

/// `log --detail`: `gitpoulpi_core::domain::MermaidDetail`.
#[cfg(feature = "mermaid")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum MermaidDetailArg {
    Structure,
    Refs,
    Subjects,
    Full,
}

#[cfg(feature = "mermaid")]
impl From<MermaidDetailArg> for gitpoulpi_core::domain::MermaidDetail {
    fn from(arg: MermaidDetailArg) -> Self {
        match arg {
            MermaidDetailArg::Structure => Self::Structure,
            MermaidDetailArg::Refs => Self::Refs,
            MermaidDetailArg::Subjects => Self::Subjects,
            MermaidDetailArg::Full => Self::Full,
        }
    }
}

/// `log --orientation`.
#[cfg(feature = "mermaid")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum MermaidOrientationArg {
    Lr,
    Tb,
}

#[cfg(feature = "mermaid")]
impl From<MermaidOrientationArg> for gitpoulpi_core::domain::MermaidOrientation {
    fn from(arg: MermaidOrientationArg) -> Self {
        match arg {
            MermaidOrientationArg::Lr => Self::Lr,
            MermaidOrientationArg::Tb => Self::Tb,
        }
    }
}

/// The command-line spelling of a line break for `convert --eol`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum EolArg {
    Lf,
    Crlf,
    Cr,
}

impl From<EolArg> for gitpoulpi_core::textconv::Eol {
    fn from(arg: EolArg) -> Self {
        match arg {
            EolArg::Lf => Self::Lf,
            EolArg::Crlf => Self::Crlf,
            EolArg::Cr => Self::Cr,
        }
    }
}

/// The command-line spelling of `gitpoulpi_core::domain::Treat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum TreatArg {
    Binary,
    Text,
}

impl From<TreatArg> for gitpoulpi_core::domain::Treat {
    fn from(arg: TreatArg) -> Self {
        match arg {
            TreatArg::Binary => Self::Binary,
            TreatArg::Text => Self::Text,
        }
    }
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)] // parsed once per process
pub enum Commands {
    Status,
    Log {
        #[arg(long)]
        graph: bool,
        #[arg(long, short = 'n')]
        limit: Option<u32>,
        /// Drop the first N commits of the walk before applying
        /// `--limit`. With `--limit` this pages history: page `n` is
        /// `--skip <n*limit> --limit <limit>`, and the pages tile the
        /// walk exactly (contract-checked by
        /// `check_log_skip_pages_history`).
        #[arg(long, default_value_t = 0)]
        skip: u32,
        /// Only commits touching this path (repeatable) - `git log --
        /// <path>`. This is the File History view's query.
        #[arg(long)]
        path: Vec<String>,
        /// Print only how many commits the walk holds (`{"total": N}`),
        /// ignoring `--limit` and `--skip`: what the TUI's status line
        /// shows after the slash. Counts without loading any commit.
        #[arg(long)]
        count: bool,
        #[cfg(feature = "mermaid")]
        /// Render the commits as another format. `mermaid` prints a
        /// Mermaid `gitGraph` script (paste it into a Markdown file, a
        /// PR or mermaid.live); with `--json`, `{format, detail, commits,
        /// text}`. The commits are `--range`, else the first `--limit`
        /// (default 30) of the walk.
        #[arg(long, value_enum)]
        format: Option<LogFormatArg>,
        #[cfg(feature = "mermaid")]
        /// Mermaid detail level: `structure` (commits, branches, merges),
        /// `refs` (+ branch names, tags, HEAD), `subjects` (+ commit
        /// subjects) or `full` (+ author and date). Default: `ui.mermaid_detail`
        /// (`refs`).
        #[arg(long, value_enum, requires = "format")]
        detail: Option<MermaidDetailArg>,
        #[cfg(feature = "mermaid")]
        /// Mermaid graph direction. Default: `ui.mermaid_orientation` (`lr`).
        #[arg(long, value_enum, requires = "format")]
        orientation: Option<MermaidOrientationArg>,
        #[cfg(feature = "mermaid")]
        /// Wrap the Mermaid script in a ```mermaid fence.
        #[arg(long, requires = "format")]
        fence: bool,
        #[cfg(feature = "mermaid")]
        /// Mermaid: the commits `A..B` (reachable from B, not from A),
        /// like `git log A..B`; a bare `A` means `A..HEAD`.
        #[arg(long, requires = "format")]
        range: Option<String>,
        #[cfg(feature = "mermaid")]
        /// Mermaid: cut commit subjects to this many characters. Default:
        /// `ui.mermaid_subject_width` (40).
        #[arg(long, requires = "format")]
        subject_width: Option<usize>,
    },
    Diff {
        #[arg(long, conflicts_with_all = ["from", "to"])]
        staged: bool,
        #[arg(long)]
        path: Option<String>,
        /// Compare from this revision (older end) instead of the working
        /// tree against the index. Pair with `--to`.
        #[arg(long)]
        from: Option<String>,
        /// The other end of `--from`: a revision, or `worktree` (the default)
        /// for the working tree, staged and unstaged changes of tracked
        /// files. The Log's Compare Selection lists the same files.
        #[arg(long, requires = "from")]
        to: Option<String>,
        /// List only the changed files and their status, without hunks.
        #[arg(long)]
        name_only: bool,
    },
    Stage {
        #[arg(long)]
        file: Option<String>,
        #[arg(long)]
        folder: Option<String>,
        /// Index of a hunk within `--file`'s diff, as `gitpoulpi diff
        /// --json` numbers them.
        #[arg(long)]
        hunk: Option<u32>,
        /// Comma-separated line indices within `--hunk`.
        #[arg(long)]
        lines: Option<String>,
        #[arg(long)]
        all: bool,
        /// Stage files that look sensitive (a private key, a `.env`, a
        /// recognized token) too. Without it such a stage is refused and
        /// nothing is staged (`[privacy] staging_guard`).
        #[arg(long)]
        allow_sensitive: bool,
    },
    Unstage {
        #[arg(long)]
        file: Option<String>,
        #[arg(long)]
        folder: Option<String>,
        #[arg(long)]
        hunk: Option<u32>,
        #[arg(long)]
        lines: Option<String>,
        #[arg(long)]
        all: bool,
    },
    /// Throw away a file's *unstaged* changes in the working tree: one
    /// hunk (`--hunk`), some of its lines (`--hunk --lines`), or - with
    /// `--file` alone - every uncommitted change to the file, staged ones
    /// included (an untracked file is deleted). Hunks and lines are
    /// numbered as `gitpoulpi diff --json` numbers the unstaged diff.
    /// Everything it throws away is kept in a safety snapshot, so
    /// `gitpoulpi undo` brings it back (untracked files included). Not
    /// confirmed: this is the scripting surface.
    Discard {
        #[arg(long)]
        file: String,
        #[arg(long)]
        hunk: Option<u32>,
        /// Comma-separated line indices within `--hunk`.
        #[arg(long)]
        lines: Option<String>,
    },
    /// Delete files (or directories) from the working tree; the index is
    /// untouched, so a tracked file shows as a deleted change. Undoable
    /// with `gitpoulpi undo`, which restores the exact bytes.
    RmFile {
        /// Repo-relative paths to delete.
        #[arg(required = true)]
        paths: Vec<String>,
    },
    /// Append a pattern to a `.gitignore` (the repository's root one by
    /// default). Does nothing when the pattern is already a line of it.
    /// Undoable with `gitpoulpi undo`, which puts the file back as it was
    /// (or removes it when it did not exist).
    Ignore {
        /// The line to add, e.g. `*.log`, `/build` or `docs/tmp/`.
        pattern: String,
        /// Repo-relative `.gitignore` to append to (default `.gitignore`).
        #[arg(long)]
        file: Option<String>,
    },
    /// Make git diff files as binary or as text, by writing a
    /// `.gitattributes` rule (`binary`, or `text diff`) for the paths: the
    /// repository's root file by default, `--file` for a folder's own or
    /// `.git/info/attributes` (this clone only). A rule already naming the
    /// same pattern is rewritten, not contradicted. The answer says whether
    /// git now really treats the paths that way (`effective`). Undoable
    /// with `gitpoulpi undo`, except for `.git/info/attributes`.
    Treat {
        /// What git should treat the paths as.
        #[arg(value_enum)]
        kind: TreatArg,
        /// Repo-relative paths to treat (folders with `--folder`).
        #[arg(required = true)]
        paths: Vec<String>,
        /// Name every file with each path's extension (`*.ext`) instead of
        /// the file itself.
        #[arg(long, conflicts_with = "folder")]
        extension: bool,
        /// The paths are folders: the rule covers everything below them.
        #[arg(long)]
        folder: bool,
        /// Repo-relative `.gitattributes` to write, or
        /// `.git/info/attributes` (default: the root `.gitattributes`).
        #[arg(long)]
        file: Option<String>,
    },
    /// Rewrite the encoding, the line endings or the indentation of files in
    /// the working tree, with the functions the TUI's editor and its
    /// Convert menu use. Exactly one of `--encoding`, `--eol` and `--indent`.
    /// Binary files, missing files and files the target encoding cannot hold
    /// are skipped and reported, never damaged. One journal entry for the
    /// whole batch: `gitpoulpi undo` puts every file back.
    #[command(group(clap::ArgGroup::new("what").required(true).multiple(false).args(["encoding", "eol", "indent"])))]
    Convert {
        /// Repo-relative files to convert.
        #[arg(required = true)]
        paths: Vec<String>,
        /// The target encoding, as the editor names it (`UTF-8`, `UTF-8 BOM`,
        /// `UTF-16 LE`, `Windows-1252`, `ISO-8859-1`, `Shift_JIS`, ...).
        #[arg(long)]
        encoding: Option<String>,
        /// The target line break.
        #[arg(long, value_enum)]
        eol: Option<EolArg>,
        /// Leading whitespace as `spaces:<n>` or `tabs:<n>`, `<n>` being the
        /// width of a tab (1-8).
        #[arg(long)]
        indent: Option<String>,
    },
    /// A file's contents at a revision, the index, or the working tree.
    Show {
        /// Repo-relative path to read.
        path: String,
        /// Revision to read it at (default: HEAD). Mutually exclusive
        /// with --staged and --worktree.
        #[arg(long)]
        at: Option<String>,
        /// Read the staged content instead of a commit's.
        #[arg(long, conflicts_with_all = ["at", "worktree"])]
        staged: bool,
        /// Read the file as it is on disk instead of a commit's.
        #[arg(long, conflicts_with_all = ["at", "staged"])]
        worktree: bool,
        /// Write the file's exact bytes (no decoding, no JSON) to stdout,
        /// or to --output. For scripts and binary files such as images.
        /// Command line only: not available over MCP.
        #[arg(long)]
        raw: bool,
        /// With --raw: write the bytes to this file instead of stdout.
        #[arg(long, requires = "raw", value_name = "FILE")]
        output: Option<std::path::PathBuf>,
    },
    /// Per-line attribution for a file at a revision.
    Blame {
        path: String,
        /// Revision to blame (default: HEAD).
        #[arg(long)]
        at: Option<String>,
    },
    /// The commits that changed a file, newest first, following renames
    /// (the path at each commit, change kind and line counts).
    FileHistory {
        path: String,
        /// Revision to start from (default: HEAD).
        #[arg(long)]
        at: Option<String>,
        /// Entries to skip (page n is `--skip <n*limit> --limit <limit>`).
        #[arg(long, default_value_t = 0)]
        skip: usize,
        /// Entries per page; `more` in the answer says whether older ones exist.
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    Commit {
        /// The commit message. Required unless --ai writes it.
        #[arg(
            short = 'm',
            long,
            required_unless_present = "ai",
            conflicts_with = "ai"
        )]
        message: Option<String>,
        #[arg(long)]
        amend: bool,
        /// Skip the commit hooks (`pre-commit`, `commit-msg`), like
        /// `commit --no-verify`.
        #[arg(long)]
        no_verify: bool,
        /// Let the AI write the message from the staged diff, using the
        /// repository's Commit message instructions (Settings > AI). Needs
        /// `ai.enabled = true` and a configured profile.
        #[arg(long)]
        ai: bool,
        /// With `--ai`: send the whole diff even when it is over
        /// `ai.limits.diff_bytes`. Without it a larger diff is truncated and
        /// the answer says so (`truncated`).
        #[arg(long, alias = "no-truncate")]
        send_all: bool,
        /// Commit as this workspace's default profile (a workspace id; also
        /// `GITPOULPI_WORKSPACE`). Applies when the repository is in the
        /// workspace and the workspace names a profile that exists; else
        /// `general.active_profile`; else the identity of the repository's
        /// own git configuration. Without it, that last one is used.
        #[cfg(feature = "workspaces")]
        #[arg(long)]
        workspace: Option<String>,
    },
    /// AI helpers (off unless `ai.enabled = true`): print a generated
    /// commit message, or explain a commit or the working changes.
    #[cfg(feature = "ai")]
    Ai {
        /// Print what would be sent to the model (the system and user text
        /// after secrets are redacted and sensitive files left out) and
        /// send nothing. Needs `ai.enabled = true`, but no key and no
        /// network.
        #[arg(long, global = true)]
        print_sent: bool,
        /// Send every input whole, even over its `ai.limits.*` cap. Without
        /// it an input over a cap is truncated (a conflict is refused) and
        /// the answer says what was cut in `truncated: [{part, detail?,
        /// bytes, limit, setting}]`.
        #[arg(long, global = true, alias = "no-truncate")]
        send_all: bool,
        #[command(subcommand)]
        action: AiCommand,
    },
    Branch {
        #[command(subcommand)]
        action: BranchAction,
    },
    /// Linked worktrees: list them, add one on a branch or a commit.
    #[cfg(feature = "worktree")]
    Worktree {
        #[command(subcommand)]
        action: WorktreeAction,
    },
    /// Submodules: list them with their state, register (`init`) or check
    /// out (`update`) them.
    #[cfg(feature = "submodule")]
    Submodule {
        #[command(subcommand)]
        action: SubmoduleAction,
    },
    /// Write a patch file, or apply one to the working tree.
    Patch {
        #[command(subcommand)]
        action: PatchAction,
    },
    Checkout {
        target: String,
        /// Stash local changes that block this, run it, then pop the
        /// stash (what `general.auto_stash = "always"` does).
        #[arg(long)]
        autostash: bool,
    },
    Merge {
        r#ref: String,
        /// Squash the branch's commits: stage their changes and make no
        /// merge commit (`merge --squash`), so a `commit` follows. The
        /// Merge behavior setting's Squash mode. Not undoable: nothing moved,
        /// discard the staged changes to back out.
        #[arg(long, conflicts_with_all = ["ff", "no_ff", "ff_only"])]
        squash: bool,
        /// Fast-forward when the branch allows it, a merge commit otherwise
        /// (git's default). Overrides `git.merge_fast_forward` for this merge.
        #[arg(long, conflicts_with_all = ["no_ff", "ff_only"])]
        ff: bool,
        /// Always make a merge commit, even when a fast-forward is possible.
        /// Overrides `git.merge_fast_forward` for this merge.
        #[arg(long = "no-ff", conflicts_with_all = ["ff", "ff_only"])]
        no_ff: bool,
        /// Fast-forward only: refuse when the branch has
        /// diverged. Overrides `git.merge_fast_forward` for this merge.
        #[arg(long = "ff-only", conflicts_with_all = ["ff", "no_ff"])]
        ff_only: bool,
        /// Stash local changes that block this, run it, then pop the
        /// stash (what `general.auto_stash = "always"` does).
        #[arg(long)]
        autostash: bool,
    },
    Rebase {
        /// The branch/commit to rebase onto, or (with `--plan`) the
        /// upstream the plan's range starts after. Omitted with `--root`
        /// or one of `--continue`/`--skip`/`--abort`/`--status`.
        onto: Option<String>,
        /// Run `git rebase -i`. Without `--plan`, only `--autosquash`
        /// alone is meaningful (there is no interactive editor here);
        /// combine with `--plan` for an explicit todo list.
        #[arg(long)]
        interactive: bool,
        /// A JSON `Vec<RebaseStep>` plan (see `rebase-plan`), from a
        /// file path or `-` for stdin.
        #[arg(long)]
        plan: Option<String>,
        /// Rebase the whole history (`--root`) instead of a range after
        /// `onto`/an explicit upstream.
        #[arg(long)]
        root: bool,
        /// Replay onto this commit instead of the upstream itself
        /// (`git rebase --onto`).
        #[arg(long = "onto-commit")]
        onto_commit: Option<String>,
        /// Reorder `--plan`'s steps with `git rebase --autosquash`'s
        /// algorithm first; with no `--plan`, run a plain
        /// `git rebase -i --autosquash`.
        #[arg(long)]
        autosquash: bool,
        /// Resume a rebase stopped by a conflict or an `edit` step.
        #[arg(long = "continue")]
        continue_: bool,
        /// Skip the commit a rebase is stopped at.
        #[arg(long)]
        skip: bool,
        /// Abort a rebase in progress, restoring the branch and HEAD.
        #[arg(long)]
        abort: bool,
        /// Report the rebase currently in progress, if any.
        #[arg(long)]
        status: bool,
        /// Stash local changes that block this, run it, then pop the
        /// stash (what `general.auto_stash = "always"` does).
        #[arg(long)]
        autostash: bool,
    },
    /// The range an interactive rebase from `upstream` (or the whole
    /// history, with `--root`) would replay, oldest first - material to
    /// build a `rebase --plan` JSON file from.
    RebasePlan {
        upstream: Option<String>,
        #[arg(long)]
        root: bool,
    },
    CherryPick {
        /// Commits to apply, in order. Omitted when `--plan` is given.
        commits: Vec<String>,
        /// A JSON `Vec<RebaseStep>` (same shape as `rebase --plan`; see
        /// `rebase-plan`), from a file path or `-` for stdin: the steps
        /// are cherry-picked in list order. Only the `Pick` and `Drop`
        /// actions make sense here (`Drop` skips the commit); any other
        /// action is refused. Cannot be combined with positional commits.
        #[arg(long)]
        plan: Option<String>,
    },
    /// Undo a commit with a new commit that applies the inverse change.
    Revert {
        /// The commit to revert (any revision: a sha, `HEAD~2`, ...).
        commit: String,
        /// For a merge commit: the number (1-based) of the parent that
        /// stays the mainline, as `git revert -m`. `1` is the branch the
        /// merge was made on. Required to revert a merge commit.
        #[arg(short = 'm', long, value_name = "PARENT")]
        mainline: Option<u32>,
    },
    /// Apply the changes one commit made (even a lost one) to the working
    /// tree only: the index and HEAD stay. All or nothing: when a file does
    /// not apply cleanly nothing changes (`stash from-commit` keeps the
    /// changes as a stash instead). A merge commit is refused. `undo`
    /// takes it back.
    ApplyCommit {
        /// The commit whose changes to apply (any revision: a sha, `HEAD~2`, ...).
        commit: String,
    },
    /// Give a commit of the current branch a new message (its
    /// descendants are re-created on top; index and working tree are
    /// left alone). A pushed commit then needs a force push.
    Reword {
        /// The commit to edit (any revision: a sha, `HEAD~2`, ...).
        commit: String,
        /// The full new message (summary, blank line, body).
        #[arg(short = 'm', long)]
        message: String,
    },
    /// Change the properties of a commit on the current branch: message,
    /// author name, email and date, committer name, email and date (any
    /// subset; what you leave out is kept). Its descendants are
    /// re-created on top with their own authors; index and working tree
    /// are left alone. Undoable; a pushed commit then needs a force push.
    Amend {
        /// The commit to change (any revision: a sha, `HEAD~2`, ...).
        commit: String,
        /// The full new message (summary, blank line, body).
        #[arg(short = 'm', long)]
        message: Option<String>,
        /// Only the summary (first line): the body is kept.
        #[arg(long, conflicts_with = "message")]
        summary: Option<String>,
        /// Only the body (what follows the summary): the summary is kept.
        /// An empty value removes the body.
        #[arg(long, conflicts_with = "message")]
        description: Option<String>,
        /// The new author name.
        #[arg(long)]
        author_name: Option<String>,
        /// The new author email.
        #[arg(long)]
        author_email: Option<String>,
        /// Raw git date (`1700000000 +0100`) or ISO 8601
        /// (`2001-02-03T04:05:06+05:30`).
        #[arg(long)]
        author_date: Option<String>,
        /// The new committer name (default: yours).
        #[arg(long)]
        committer_name: Option<String>,
        /// The new committer email (default: yours).
        #[arg(long)]
        committer_email: Option<String>,
        /// Same formats as --author-date (default: now).
        #[arg(long)]
        committer_date: Option<String>,
    },
    /// The changes of a commit numbered for `split-commit`: every file with
    /// its hunks (`id`, lines) or, for files that can only be assigned
    /// whole (binary, mode-only, added, deleted, submodule, LFS), the
    /// reason. Read-only.
    SplitSource {
        /// The commit (any revision: a sha, `HEAD~2`, ...).
        commit: String,
    },
    /// Break a commit of the current branch into several, oldest first,
    /// following a JSON plan (see `split-source` for the numbering). The
    /// last new commit carries the original tree, so descendants and the
    /// working tree are unchanged; a pushed commit then needs a force
    /// push. One `undo` puts it back. Exit code 2 when refused.
    SplitCommit {
        /// The commit to split (any revision: a sha, `HEAD~2`, ...).
        commit: String,
        /// The plan (JSON), from a file path or `-` for stdin. A piece is
        /// `path` plus an optional `hunk` plus optional `lines` (indices into
        /// the hunk's `lines`); a piece with only a `path` is the whole file.
        #[arg(long)]
        plan: String,
        /// Put every change the plan leaves unassigned into one final
        /// commit with this message.
        #[arg(long, value_name = "MESSAGE")]
        rest_message: Option<String>,
        /// Sign the new commits (and the replayed descendants).
        #[arg(long)]
        sign: bool,
        /// Validate the plan and list what is unassigned; change nothing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Undo the latest journaled operation (commit, amend, checkout,
    /// branch create/delete). Refuses, changing nothing, if anything the
    /// operation touched has moved since. Exit code 2 when refused, 3 when
    /// it needs `--yes` (a risky undo).
    Undo {
        /// Show what would happen and change nothing.
        #[arg(long)]
        dry_run: bool,
        /// List the journal instead (newest last); `--all` adds the
        /// entries that are no longer redoable but still pinned.
        #[arg(long)]
        list: bool,
        /// With `--list`: include entries no longer redoable.
        #[arg(long, requires = "list")]
        all: bool,
        /// Only undo if this is still the latest entry (its id, as
        /// `--list` shows it); otherwise refused as stale.
        #[arg(long, conflicts_with = "list")]
        expect: Option<u64>,
        /// Go ahead when the undo is risky (the commit is already on its
        /// upstream, or working-tree files would be overwritten). Without
        /// it such an undo changes nothing and exits with code 3.
        #[arg(long, conflicts_with = "list")]
        yes: bool,
    },
    /// Drop the blocked top entry of the undo journal (an interrupted
    /// operation, or one the repository moved on from) so the entries
    /// under it can be undone. Never touches the repository, only the
    /// journal; the entry cannot be undone or redone afterwards. Exit
    /// code 2 when the top entry is not blocked, 3 without `--yes`.
    UndoForget {
        /// Show what would be dropped and change nothing.
        #[arg(long)]
        dry_run: bool,
        /// Forget the top redo entry instead of the top undo entry.
        #[arg(long)]
        redo: bool,
        /// Only forget if this is still the top entry (its id, as
        /// `undo --list` shows it); otherwise refused as stale.
        #[arg(long)]
        expect: Option<u64>,
        /// Confirm. Without it nothing changes and the exit code is 3.
        #[arg(long)]
        yes: bool,
    },
    /// "Restore only this": bring back what one earlier operation removed
    /// (a deleted branch or tag, a dropped stash, a discarded or deleted
    /// file, a removed remote, an ignore line) without undoing the
    /// operations after it. The restore is a new journal entry, so `undo`
    /// takes it back. Nothing is overwritten: a file that changed since gets
    /// the kept version next to it as `<name>.restored`; a reset, amend or
    /// rebase comes back as a new branch at the old tip. Also spelled
    /// `gitpoulpi undo restore <id>`. Exit code 2 when it cannot be done (the
    /// JSON names the reason).
    UndoRestore {
        /// The entry to restore, as `undo --list` shows its id.
        id: u64,
        /// Restore the branch or tag under this name (when its old name is
        /// taken, or for the branch a reset/amend/rebase restores).
        #[arg(long)]
        name: Option<String>,
        /// Show what would happen and change nothing.
        #[arg(long)]
        dry_run: bool,
    },
    /// List a reflog, newest first: where `HEAD` or a branch was before, with
    /// the date and git's reason, including moves made in a shell. A branch
    /// that was deleted has no reflog left. Also spelled `reflog restore`
    /// for `reflog-restore`.
    Reflog {
        /// `HEAD` (the default) or a branch (or a full ref name).
        #[arg(name = "ref")]
        reference: Option<String>,
        /// Entries to skip.
        #[arg(long, default_value_t = 0)]
        skip: usize,
        /// Entries to list.
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Move a branch (or `HEAD`) to a position of its reflog. A branch that
    /// is not checked out just moves. For the checked-out branch the working
    /// tree stays safe: with uncommitted changes it refuses unless `--mode` is
    /// given. One journal entry: `undo` takes it back. Also spelled
    /// `gitpoulpi reflog restore`.
    ReflogRestore {
        /// `HEAD` or a branch.
        #[arg(name = "ref")]
        reference: String,
        /// The commit to move it to (a reflog entry's `new`).
        oid: String,
        /// With uncommitted changes: `keep` moves and keeps them (refusing if
        /// one is in the way), `soft`, `mixed` and `hard` are the reset modes.
        #[arg(long, value_enum)]
        mode: Option<ReflogModeArg>,
        /// Show what would happen and change nothing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Commits no branch, tag, remote or stash reaches: what a deleted
    /// branch left behind (named by a reflog, or kept only by an undo pin)
    /// and, unless `--no-deep`, the dangling commits of the object database.
    /// Newest first. Create a branch at one to get it back.
    LostCommits {
        /// Commits to skip.
        #[arg(long, default_value_t = 0)]
        skip: usize,
        /// Commits to list.
        #[arg(long, default_value_t = 50)]
        limit: usize,
        /// Only commits a reflog or an undo pin still names (fast on any size).
        #[arg(long)]
        no_deep: bool,
        /// Stop scanning after this many milliseconds (0: no limit); the answer
        /// then says `partial`.
        #[arg(long, default_value_t = 8000)]
        budget_ms: u64,
    },
    /// Redo the operation `undo` just took back, to the exact state
    /// recorded (it never re-runs the operation). Exit code 2 when refused.
    Redo {
        /// Show what would happen and change nothing.
        #[arg(long)]
        dry_run: bool,
        /// Only redo if this is still the latest undone entry.
        #[arg(long)]
        expect: Option<u64>,
        /// Go ahead when the redo is risky; without it exit code 3.
        #[arg(long)]
        yes: bool,
    },
    Reset {
        target: String,
        #[arg(long, value_enum)]
        mode: ResetModeArg,
    },
    Stash {
        #[command(subcommand)]
        action: StashAction,
    },
    /// The pending auto-stash: GitPoulpi stashed local changes to run a
    /// checkout, merge, rebase or pull and has not put them back.
    Autostash {
        #[command(subcommand)]
        action: AutoStashAction,
    },
    Tag {
        #[command(subcommand)]
        action: TagAction,
    },
    Fetch {
        /// Omitted: the repository's default remote (see `remote default`).
        #[arg(long, conflicts_with = "all")]
        remote: Option<String>,
        /// Fetch every configured remote.
        #[arg(long)]
        all: bool,
        /// Drop remote-tracking branches the remote no longer has.
        #[arg(long)]
        prune: bool,
        /// Also delete the local tags the remote has deleted (only tags
        /// that were on it: a tag never pushed is left alone). Git's
        /// `fetch.pruneTags`, narrower; off unless asked.
        #[arg(long)]
        prune_tags: bool,
    },
    Pull {
        /// Pull the same-named branch from this remote instead of the
        /// branch's upstream.
        #[arg(long)]
        remote: Option<String>,
        /// How to integrate: `merge` (fast-forward if possible), `ff-only`
        /// or `rebase`. Always passed to git explicitly.
        #[arg(long, value_enum, default_value = "merge")]
        mode: PullModeArg,
        /// Stash local changes that block this, run it, then pop the
        /// stash (what `general.auto_stash = "always"` does).
        #[arg(long)]
        autostash: bool,
    },
    Push {
        /// Omitted: the repository's default remote (see `remote default`).
        #[arg(long)]
        remote: Option<String>,
        /// The local branch to push; the checked-out one when omitted.
        #[arg(long)]
        branch: Option<String>,
        /// Record `<remote>/<branch>` as the branch's upstream.
        #[arg(long, short = 'u')]
        set_upstream: bool,
        /// Overwrite the remote branch, only if it is where our last fetch
        /// saw it. There is no plain `--force`.
        #[arg(long)]
        force_with_lease: bool,
    },
    Remote {
        #[command(subcommand)]
        action: RemoteAction,
    },
    #[cfg(feature = "lfs")]
    /// Git LFS: large files stored outside the repository's history.
    /// Needs the `git-lfs` program for anything that moves or converts
    /// files; `status` and `files` work without it.
    Lfs {
        #[command(subcommand)]
        action: LfsAction,
    },
    /// The `.mailmap`: one canonical name and email for people who committed
    /// under several. The Log, blame and file history show the mapped names
    /// (`log.use_mailmap`) without touching a commit. `list` counts the
    /// identities of the history and shows which are mapped, `check` shows how
    /// some identities resolve, `add` merges identities into one by appending
    /// lines to the `.mailmap` (edited, not committed; undoable).
    Mailmap {
        #[command(subcommand)]
        action: MailmapAction,
    },
    #[cfg(feature = "history-rewrite")]
    /// Rewrite history in bulk (identities, dates, messages) from a recipe:
    /// `preview` plans it without changing anything, `apply` runs the plan
    /// you reviewed as ONE undo entry, `run` does both for scripts. Only
    /// `push` sends anything: it force-pushes the rewritten refs, each with
    /// a lease.
    Rewrite {
        #[command(subcommand)]
        action: RewriteAction,
    },
    /// Branching workflows (Git Flow, GitHub Flow, GitLab Flow, Trunk-based or
    /// your own): which branches are long-lived bases, which are topic
    /// branches of which type, their parents and how far they are from them.
    /// `status`, `templates`, `detect` and `config` read; `config init` and
    /// `import` print a workflow file and create it only with `--write`.
    /// `start`, `finish`, `sync`, `promote` and `set-parent` act: each prints
    /// the steps it will do (`--dry-run` prints them as JSON and stops),
    /// needs `--yes` to run, and is one undoable entry (a merge request or a
    /// push cannot be undone).
    Flow {
        #[command(subcommand)]
        action: FlowAction,
    },
    /// The repository-local `core.sshCommand` pin: which SSH key this
    /// repository uses. See `repo-auth` for the whole identity.
    SshCommand {
        #[command(subcommand)]
        action: SshCommandAction,
    },
    /// Which identity this repository authenticates with (HTTPS and SSH): the
    /// machine's own, a GitPoulpi account, or a key file. `show` and `test`
    /// read; `set` and `clear` are a person's decision.
    RepoAuth {
        #[command(subcommand)]
        action: RepoAuthAction,
    },
    /// The repository-local `core.autocrlf` and `core.eol` (what Settings >
    /// Git & Encoding edits). Undoable like any other operation.
    RepoConfig {
        #[command(subcommand)]
        action: RepoConfigAction,
    },
    /// Sparse checkout (cone mode): keep only some directories in the working
    /// tree of a big repository. `gitpoulpi clone --sparse` starts one.
    SparseCheckout {
        #[command(subcommand)]
        action: SparseAction,
    },
    Clone {
        url: String,
        #[arg(long)]
        dest: Option<String>,
        /// Check out this branch instead of the remote's default HEAD.
        #[arg(long)]
        branch: Option<String>,
        /// Name the new remote something other than `origin`.
        #[arg(long)]
        origin: Option<String>,
        /// Truncate history to this many commits (`git clone --depth`).
        #[arg(long)]
        depth: Option<u32>,
        /// Clone submodules too.
        #[arg(long)]
        recurse_submodules: bool,
        /// Sparse checkout: check out only these directories (plus the
        /// files at the root); the whole history is still downloaded.
        #[arg(long, num_args = 1.., value_name = "DIR")]
        sparse: Vec<String>,
        /// Leave Git LFS files as pointers instead of downloading their
        /// content after the clone.
        #[cfg(feature = "lfs")]
        #[arg(long)]
        no_lfs: bool,
    },
    /// Opening, inspecting and remembering repositories - the
    /// multi-repository surface. See `gitpoulpi repo --help`.
    Repo {
        #[command(subcommand)]
        action: RepoAction,
    },
    Conflicts {
        #[command(subcommand)]
        action: ConflictsAction,
    },
    /// A stale `.git/index.lock`, which makes every write fail: look at
    /// it, or remove it once it is old enough.
    IndexLock {
        #[command(subcommand)]
        action: IndexLockAction,
    },
    Refs {
        #[command(subcommand)]
        action: RefsAction,
    },
    #[cfg(feature = "hosting")]
    Hosting {
        #[command(subcommand)]
        action: HostingAction,
    },
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// The color themes: list them, or print one as a theme file.
    Theme {
        #[command(subcommand)]
        action: ThemeAction,
    },
    /// Prints the third-party license notices and credits built into this
    /// program (the same text as About > Licenses and the
    /// THIRD-PARTY-LICENSES.txt file of the release archive).
    Licenses,
    /// Shortcuts that start GitPoulpi from the OS launcher, in the terminal
    /// `ui.launch_terminal` names.
    Desktop {
        #[command(subcommand)]
        action: DesktopAction,
    },
    /// What the GitPoulpi shortcuts run: shows the window of a GitPoulpi that
    /// is already open (`ui.reuse_window`), else starts COMMAND (the terminal
    /// that runs GitPoulpi). Starts nothing and starts no daemon when it can
    /// show a window.
    Launch {
        /// Start a new window even when one is open.
        #[arg(long)]
        new_window: bool,
        /// The command to start when there is no window to show, after `--`.
        #[arg(
            trailing_var_arg = true,
            allow_hyphen_values = true,
            value_name = "COMMAND"
        )]
        command: Vec<String>,
    },
    /// The command alias (`gp` or any name) that starts GitPoulpi from a
    /// terminal: the scriptable twin of Settings > General > Launcher.
    Alias {
        #[command(subcommand)]
        action: AliasAction,
    },
    /// Puts GitPoulpi on the user PATH, for new terminals.
    Path {
        #[command(subcommand)]
        action: PathAction,
    },
    /// Removes what the installer and Settings made (command aliases, the
    /// PATH entries, the shortcuts) and, from a release install, the program
    /// files. Refuses on a build that is not a release install (no
    /// `.gitpoulpi-dist` marker next to the exe). Your config, sessions and
    /// repositories are never touched.
    Uninstall {
        /// Leave the program files in place (the Windows uninstaller deletes
        /// them itself, because a running exe cannot).
        #[arg(long)]
        keep_files: bool,
    },
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },
    /// The daemon's background jobs: every fetch, push, pull, clone and LFS
    /// transfer is one from the moment it is received. `list` shows them,
    /// `cancel` stops one, `wait` blocks until one ends. Ctrl+C on a command
    /// that started a job cancels that job.
    Job {
        #[command(subcommand)]
        action: JobAction,
    },
    #[cfg(feature = "workspaces")]
    /// Workspaces: named groups of repositories, optionally with a default
    /// commit profile. Stored in `workspaces.toml` beside `config.toml`,
    /// shared with the TUI.
    Workspace {
        #[command(subcommand)]
        action: WorkspaceAction,
    },
    #[cfg(feature = "global-search")]
    /// Fuzzy search over the branches, tags, worktrees, stashes and commit
    /// messages of the current repository, or of every repository of a
    /// workspace, plus the cached pull requests and issues. Hits come best
    /// first with the characters that matched. A prefix narrows the search:
    /// `b:` branches, `t:` tags, `c:` commits, `s:` stashes, `w:` worktrees,
    /// `pr:` pull requests, `i:` issues, `r:` repositories (workspace only).
    Search {
        /// What to look for, prefix included (`c: fix login`).
        query: String,
        /// Search every repository of this workspace (an id; also
        /// `GITPOULPI_WORKSPACE`). Without one, the current repository.
        #[cfg(feature = "workspaces")]
        #[arg(long)]
        workspace: Option<String>,
        /// Only these kinds (repeatable); default all. A prefix in the query
        /// narrows further.
        #[arg(long = "kind", value_enum)]
        kinds: Vec<SearchKindArg>,
        /// At most this many hits.
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// `gitpoulpi server ...` is a visible alias - the owner calls this
    /// group "the server" and types `gp server stop`.
    #[command(visible_alias = "server")]
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    #[cfg(feature = "mcp")]
    Mcp {
        #[command(subcommand)]
        action: McpAction,
    },
    /// Show things to the user in their open GitPoulpi window: a commit, a
    /// diff at a line, a file, a settings page. View-only: nothing here
    /// changes the repository or types into the window, except `ui propose`,
    /// which opens a dialog pre-filled for the user to confirm (off unless
    /// `[mcp] allow_ui_propose = true`). Needs a GitPoulpi TUI open on the
    /// session.
    Ui {
        /// Send to this window (`ui viewers` lists them) instead of letting
        /// the daemon choose.
        #[arg(long, global = true)]
        viewer: Option<String>,
        /// When no window is open anywhere, open one with the configured
        /// launch command (`[ui] launch` / `launch_command`; off by default)
        /// and wait up to 10 s for it to attach. Without this flag a window
        /// is only launched when `[ui] launch` is set AND `ui_control` is
        /// `auto`.
        #[arg(long, global = true)]
        launch: bool,
        #[command(subcommand)]
        action: UiAction,
    },
    /// Prints a shell completion script to stdout (bash, zsh, fish,
    /// powershell or elvish), e.g. `gitpoulpi completions bash >
    /// ~/.local/share/bash-completion/completions/gitpoulpi`.
    Completions {
        shell: clap_complete::Shell,
    },
    /// Prints a shell wrapper function (default name `gp`) that runs
    /// gitpoulpi and, when you quit with "Quit and cd here", leaves your
    /// shell in the active tab's folder. Add it to your shell's rc file:
    /// `eval "$(gitpoulpi shell-init bash)"`. Nothing is installed or edited.
    ShellInit {
        shell: crate::shell_init::ShellKind,
        /// The function's name. An existing command of that name is not
        /// overridden.
        #[arg(long, default_value = crate::shell_init::DEFAULT_NAME)]
        name: String,
    },
}
