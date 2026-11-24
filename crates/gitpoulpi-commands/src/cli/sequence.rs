use clap::{Subcommand, ValueEnum};

/// The command-line spelling of `gitpoulpi_config::sequences::Scope`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SequenceScopeArg {
    /// Only this installation sees it (`config.toml`, owner = this installation).
    Installation,
    /// Every installation sharing this `config.toml`.
    Machine,
    /// The current repository only (`.gitpoulpi/sequences.toml`).
    Repo,
}

#[derive(Debug, Subcommand)]
pub enum SequenceAction {
    /// The sequences this installation sees (its own and the machine-wide
    /// ones in `config.toml`), each with its steps, its validation problems
    /// and whether it can run headless. The current repository's
    /// `.gitpoulpi/sequences.toml` is listed too, with `trusted` as the user
    /// decided (`sequence trust`) and a `repo_trust` summary: repository
    /// sequences are somebody else's commands, so they run only while the
    /// file is trusted exactly as it is now. Needs no daemon.
    List,
    /// One sequence with its steps and, for each step, what it does and
    /// whether it can run headless.
    Show {
        /// The sequence id (`gitpoulpi sequence list`).
        id: String,
    },
    /// Check a sequence against the step vocabulary: unknown or excluded
    /// steps, bad options, a forced pause. Without an id or a file, every
    /// visible sequence. Exit code 2 when anything has a problem.
    Validate {
        /// The sequence id.
        #[arg(conflicts_with = "file")]
        id: Option<String>,
        /// Check the sequences in this TOML file (`[[sequences]]`) instead.
        #[arg(long)]
        file: Option<String>,
    },
    /// Run a sequence headless through the daemon, one request per step with
    /// the same mapping as the matching command (`stage --all`, `commit`,
    /// `push`, ...). Everything is checked first and nothing runs when a
    /// step is unknown, valid only in the terminal UI (terminal, open in,
    /// open tab, agent here), or `force_push_with_lease` (never headless):
    /// exit code 2 with `problems`. A step that pauses (its pause is `always`,
    /// or `default` on a step that normally asks: the AI message review, a
    /// name dialog) needs `--yes`, else exit code 3 and nothing changed;
    /// `--yes` accepts the proposal as is (the AI message, the name
    /// template). A step that fails stops the run, exit code 2, the earlier
    /// steps stay done (each is undoable on its own).
    Run {
        /// The sequence id.
        id: String,
        /// Accept the steps that would pause, as proposed.
        #[arg(long)]
        yes: bool,
        /// Print the resolved plan (the command of each step, templates
        /// expanded) and change nothing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Add or replace sequences from a TOML file (`[[sequences]]` with `id`,
    /// `name`, `steps`, like `config.toml`; a file with one sequence at the
    /// top level works too). Each is validated first: one problem and
    /// nothing is written (exit code 2). The other keys of the file written
    /// to are kept.
    Add {
        /// The TOML file.
        #[arg(long)]
        file: String,
        /// Replace a sequence that has the same id in the same place.
        #[arg(long)]
        replace: bool,
        /// Where to save, overriding the file's own `scope` (default
        /// `installation`).
        #[arg(long, value_enum)]
        scope: Option<SequenceScopeArg>,
    },
    /// Remove a sequence from this installation's or the machine's config
    /// (or, with `--scope repo`, the current repository's file).
    Remove {
        /// The sequence id.
        id: String,
        /// Where to remove it from. Omitted: this installation's own, else
        /// the machine-wide one.
        #[arg(long, value_enum)]
        scope: Option<SequenceScopeArg>,
    },
    /// Trust the sequences of the repository's `.gitpoulpi/sequences.toml`
    /// as they are now (use `--repo PATH` for another repository): they then
    /// list, bind and run like your own. Any change of an id, a command, a
    /// pause or an option asks again; a new name does not. Prints the
    /// sequences and what they do first when run in a terminal is not
    /// needed: the JSON lists every step. Exit code 2 when the repository
    /// has no such file.
    Trust,
    /// Forget the trust given by `sequence trust`: the repository's
    /// sequences are listed again but never run.
    Untrust,
}
