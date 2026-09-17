use clap::Subcommand;

/// `gitpoulpi ui ...`. Every leaf is view-only (`policy.rs` lists them as
/// such for the MCP server), except `propose`, its own gated class.
#[derive(Debug, Subcommand)]
pub enum UiAction {
    /// List the GitPoulpi windows attached to this session.
    Viewers {
        /// Also list the windows of every other running session
        /// (read from `sessions/*.port`, loopback only).
        #[arg(long)]
        all_sessions: bool,
    },
    /// Read what a window currently shows (the most recently focused one,
    /// or `--viewer`).
    State {
        /// Also return the plain text of the main pane's visible rows
        /// (200 lines at most; secret fields stay masked).
        #[arg(long)]
        text: bool,
    },
    /// Move the user's view to something.
    Show {
        #[command(subcommand)]
        what: UiShowAction,
    },
    /// Tint a line range in the diff or file the window has open.
    Highlight {
        /// `A-B`, or `A:C-B:C` with columns (1-based).
        range: String,
        /// Do not scroll the range into view.
        #[arg(long)]
        no_scroll: bool,
    },
    /// Show a short toast message to the user.
    Notify {
        /// At most 200 characters; control characters are stripped.
        message: String,
    },
    /// Undo the agent's last view change (restore the previous view).
    Back,
    /// Propose a change for the user to confirm: a dialog opens pre-filled,
    /// marked "Proposed by", and nothing happens until the user presses its
    /// own confirm. Needs `[mcp] allow_ui_propose = true`. Follow the
    /// answer's `proposal` id in `ui state`'s `proposals`.
    Propose {
        #[command(subcommand)]
        what: UiProposeAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum UiProposeAction {
    /// Open the merge editor on a conflicted file with a resolution already
    /// in it, for the user to review and confirm with Save & Mark Resolved
    /// (their Ctrl+S is the only confirm; nothing is written or staged
    /// here). Give the choices as `--pick`, or the whole file as
    /// `--content-file`.
    #[command(group(
        clap::ArgGroup::new("how")
            .required(true)
            .multiple(false)
            .args(["pick", "content_file"])
    ))]
    Resolution {
        /// Repository-relative path of the conflicted file.
        path: String,
        /// A choice per conflict number: `1=a,2=b,3=ab,4=ba,5=none` (a =
        /// current, b = incoming, ab / ba = both in that order).
        #[arg(long, value_name = "N=CHOICE,...")]
        pick: Option<String>,
        /// With `--pick`: the choice for every conflict not listed.
        #[arg(long = "default", value_name = "a|b", requires = "pick")]
        pick_default: Option<String>,
        /// The final file text, exactly (`-` reads standard input).
        #[arg(long, value_name = "FILE")]
        content_file: Option<String>,
    },
    /// Open the Reword popup over a commit with a new message filled in.
    Reword {
        /// The commit (on the checked-out branch).
        rev: String,
        /// The proposed message: summary, blank line, body.
        #[arg(long, short)]
        message: String,
    },
    /// Open the split editor over a commit, filled with a plan, for the user
    /// to review, edit and apply; never splits anything itself.
    Split {
        /// The commit to split (on the checked-out branch).
        rev: String,
        /// The plan (JSON), from a file path or `-` for stdin, as
        /// `split-commit --plan` takes it.
        #[arg(long, value_name = "FILE")]
        plan: String,
    },
    #[cfg(feature = "history-rewrite")]
    /// Open the history rewrite workbench with a recipe already in it, for
    /// the user to preview and apply; never rewrites, purges or pushes
    /// anything itself. A recipe has at most 50 rules and no script, and
    /// secret values are dropped from it.
    Rewrite {
        /// The recipe (TOML or JSON, as `rewrite preview --recipe` takes
        /// it), from a file path or `-` for stdin.
        #[arg(long, value_name = "FILE")]
        recipe: String,
    },
    /// Fill the commit composer's Summary and Description; never commits.
    CommitMessage {
        summary: String,
        #[arg(long)]
        body: Option<String>,
    },
    /// Open the New Branch box with a name filled in.
    Branch {
        name: String,
        /// Where the branch starts (default: HEAD).
        #[arg(long)]
        start_point: Option<String>,
    },
    /// Open the New Tag box with a name (and, annotated, a message) filled in.
    Tag {
        name: String,
        /// The commit to tag (default: HEAD).
        #[arg(long)]
        target: Option<String>,
        /// Makes it an annotated tag with this message.
        #[arg(long, short)]
        message: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum UiShowAction {
    /// Switch to (or open) a repository tab.
    Tab {
        /// The repository to show; defaults to `--repo` / the current one.
        path: Option<String>,
    },
    /// Show Repository Management (the home screen).
    Home {
        /// Text for its search box.
        #[arg(long)]
        filter: Option<String>,
    },
    /// Select a commit in the Log.
    Commit {
        /// Any revision (branch, tag, `HEAD~2`, a sha).
        rev: String,
        /// More commits to select with it (at most 9).
        #[arg(long)]
        also: Vec<String>,
        /// Expand the commit's changed-files list.
        #[arg(long)]
        files: bool,
        /// Move the keyboard focus to this pane.
        #[arg(long, value_parser = ["refs", "log", "status"])]
        focus: Option<String>,
    },
    /// Open the Log find bar with a query (dims, never filters).
    Search { query: String },
    /// Select a ref (branch, tag, stash) in the sidebar; never checks it out.
    Ref {
        /// A full (`refs/heads/x`) or short (`main`, `origin/main`) name.
        name: String,
    },
    /// Select a pull request / merge request in the sidebar.
    Pr { number: u32 },
    /// Select an issue in the sidebar.
    Issue { number: u32 },
    /// Show the working-tree changes (the WIP row).
    Wip {
        /// Select this file's row.
        path: Option<String>,
        /// The staged side of `path`.
        #[arg(long)]
        staged: bool,
        /// Put the keyboard focus in the commit summary field (types nothing).
        #[arg(long)]
        composer: bool,
    },
    /// Open a file's diff, at a line.
    Diff {
        /// Repository-relative path, forward slashes.
        path: String,
        /// Diff of this commit against its parent (default: HEAD).
        #[arg(long, conflicts_with_all = ["unstaged", "staged"])]
        rev: Option<String>,
        /// The working tree against the index.
        #[arg(long, conflicts_with = "staged")]
        unstaged: bool,
        /// The index against HEAD.
        #[arg(long)]
        staged: bool,
        /// Scroll to this line (1-based).
        #[arg(long)]
        line: Option<u32>,
        /// Which side's numbering `--line` uses.
        #[arg(long, value_parser = ["old", "new"])]
        side: Option<String>,
        #[arg(long, conflicts_with = "inline")]
        split: bool,
        #[arg(long)]
        inline: bool,
        /// Show the whole file rather than only the hunks.
        #[arg(long, conflicts_with = "hunks")]
        whole_file: bool,
        #[arg(long)]
        hunks: bool,
        /// Tint a range: `A-B`, or `A:C-B:C`.
        #[arg(long)]
        highlight: Option<String>,
    },
    /// Open a file (content, blame or history) at a position.
    File {
        path: String,
        /// The file at this revision (default: HEAD).
        #[arg(long, conflicts_with_all = ["worktree", "index"])]
        rev: Option<String>,
        /// The file on disk.
        #[arg(long, conflicts_with = "index")]
        worktree: bool,
        /// The file as staged.
        #[arg(long)]
        index: bool,
        #[arg(long, conflicts_with = "history")]
        blame: bool,
        #[arg(long)]
        history: bool,
        #[arg(long)]
        line: Option<u32>,
        #[arg(long)]
        column: Option<u32>,
        /// Tint a range: `A-B`, or `A:C-B:C`.
        #[arg(long)]
        highlight: Option<String>,
    },
    /// Open the merge conflict editor on a conflicted file, at a conflict.
    /// View-only: nothing is picked and nothing is saved.
    Conflict {
        /// Repository-relative path of the conflicted file.
        path: String,
        /// Scroll to conflict N (1-based).
        #[arg(long, value_name = "N")]
        conflict: Option<u32>,
    },
    /// Open a Settings page and flash a setting; never edits a value.
    Settings {
        #[arg(value_parser = clap::builder::PossibleValuesParser::new(gitpoulpi_ipc::ui::SettingsPage::IDS))]
        page: Option<String>,
        /// A config key such as `ui.minimap`.
        #[arg(long)]
        setting: Option<String>,
    },
    /// Open the command palette with a query filled in; never runs it.
    Palette { query: Option<String> },
    /// Open the help overlay (the key bindings; named `keys` because clap
    /// reserves `help` as a subcommand name).
    Keys { topic: Option<String> },
}
