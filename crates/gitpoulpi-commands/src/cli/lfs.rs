use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum LfsAction {
    /// Is `git-lfs` installed, does this repository use LFS, which
    /// patterns are tracked, are the filters and the hook installed.
    /// Never fails because `git-lfs` is missing: the answer says so.
    Status,
    /// The LFS pointer files in the index (or at `--at`), with whether
    /// each real object is in the local LFS store.
    Files {
        /// Revision to list (default: the index).
        #[arg(long)]
        at: Option<String>,
    },
    /// Set up the LFS filters and hooks (`git lfs install`), for this
    /// repository only unless `--user`.
    Install {
        /// Install into the user's global git config instead of this
        /// repository.
        #[arg(long)]
        user: bool,
    },
    /// Route file patterns through LFS (`git lfs track`). Edits
    /// `.gitattributes` and leaves it unstaged.
    Track {
        #[arg(required = true)]
        patterns: Vec<String>,
        /// Make the files read-only until locked.
        #[arg(long)]
        lockable: bool,
        /// Also re-stage already-committed files that match, so their
        /// next commit stores pointers (history is not rewritten).
        #[arg(long)]
        renormalize: bool,
    },
    /// Stop routing patterns through LFS (`git lfs untrack`).
    Untrack {
        #[arg(required = true)]
        patterns: Vec<String>,
    },
    /// Delete old, already-pushed objects from the local LFS store.
    Prune,
    /// Download LFS objects into the local store (`git lfs fetch`),
    /// leaving the working tree alone.
    Fetch {
        /// Omitted: the repository's default remote (see `remote default`).
        #[arg(long)]
        remote: Option<String>,
        /// Every ref, not only the current one.
        #[arg(long)]
        all: bool,
    },
    /// Download LFS objects and replace the pointer files in the working
    /// tree with the real content (`git lfs pull`).
    Pull {
        /// Omitted: the repository's default remote.
        #[arg(long)]
        remote: Option<String>,
    },
    /// Write the real content of the LFS pointer files in the working
    /// tree from the local LFS store (`git lfs checkout`). Offline.
    Checkout {
        /// Only these paths (relative to the repository root); omitted:
        /// every pointer in the working tree.
        paths: Vec<String>,
    },
    /// The LFS file locks on the server: who holds which file. Needs a
    /// server with locking (GitHub, GitLab, Gitea, ...); uses the
    /// remote's own credentials.
    Locks {
        /// Omitted: the repository's default remote.
        #[arg(long)]
        remote: Option<String>,
    },
    /// Lock a file so nobody else can change it until you unlock it.
    Lock {
        path: String,
        /// Omitted: the repository's default remote.
        #[arg(long)]
        remote: Option<String>,
    },
    /// Release a lock. Someone else's lock needs `--force`, which breaks
    /// it.
    Unlock {
        path: String,
        /// Omitted: the repository's default remote.
        #[arg(long)]
        remote: Option<String>,
        /// Break a lock held by someone else.
        #[arg(long)]
        force: bool,
    },
    /// Upload LFS objects (`git lfs push`).
    Push {
        /// Omitted: the repository's default remote.
        #[arg(long)]
        remote: Option<String>,
        /// Every ref, not only the current branch.
        #[arg(long)]
        all: bool,
    },
}
