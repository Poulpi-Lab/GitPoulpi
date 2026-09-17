use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum MailmapAction {
    /// List every name and email that authored or committed a commit of the
    /// scope, with how many commits each has as author and as committer,
    /// most commits first, and the identity the `.mailmap` already maps it
    /// to (`mapped_to`). Reads only.
    List {
        /// Every local branch and tag instead of the current branch.
        #[arg(long, conflicts_with = "refs")]
        all_local: bool,
        /// Only these local branches or tags (repeatable).
        #[arg(long = "ref", value_name = "REF")]
        refs: Vec<String>,
        /// Only the identities the `.mailmap` already maps.
        #[arg(long, conflicts_with = "unmapped")]
        mapped: bool,
        /// Only the identities the `.mailmap` does not map.
        #[arg(long)]
        unmapped: bool,
    },
    /// Show how identities (`Name <email>`, or `<email>`) appear through the
    /// repository's `.mailmap`: the canonical name and email, or the same
    /// identity when nothing maps it. Reads only.
    Check {
        /// The identities to resolve, each `Name <email>` or `<email>`.
        #[arg(required = true, value_name = "IDENTITY")]
        identities: Vec<String>,
    },
    /// Merge identities into one canonical identity by appending lines to
    /// the `.mailmap` (created when missing; a line already there is not
    /// repeated). The file is edited, not committed: commit it yourself. One
    /// undoable entry (`gitpoulpi undo` puts the previous file back). The Log
    /// shows the canonical identity at once (`log.use_mailmap`); the stored
    /// commits are not changed, so nothing needs a force push. To rewrite
    /// the commits themselves use a `mailmap` rule in `rewrite`.
    Add {
        /// The identity to keep, `Name <email>`.
        #[arg(long, value_name = "IDENTITY")]
        canonical: String,
        /// An identity to map to it, `Name <email>` (repeatable, at least one).
        #[arg(long = "from", required = true, value_name = "IDENTITY")]
        from: Vec<String>,
        /// The `.mailmap` to write, relative to the repository root (default:
        /// the root `.mailmap`).
        #[arg(long)]
        file: Option<String>,
    },
}
