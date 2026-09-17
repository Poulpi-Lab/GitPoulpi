//! `gitpoulpi conflicts ...`: look at the unmerged files of a stopped
//! merge, rebase, cherry-pick or revert, resolve them, and abort the
//! operation. The agent recipe is in `skills/gitpoulpi/SKILL.md`.

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum ConflictsAction {
    /// The conflicted files, each with the text of its three sides. With
    /// `--names-only`: just the path, the kind of conflict and the stage
    /// oids (no blob is read; cheap on a big merge).
    List {
        #[arg(long)]
        names_only: bool,
    },
    /// One conflicted file as the merge editor sees it: the pane labels,
    /// the three sides and the numbered conflict regions (`--json` is the
    /// full document). Merged regions git settled by itself are not
    /// listed; they stay in the result of `resolve --both`/`--pick`.
    Show {
        path: String,
        /// Only conflict N (1-based).
        #[arg(long, value_name = "N")]
        conflict: Option<usize>,
    },
    /// Resolve one file and stage it (the file stops being conflicted).
    /// Exactly one of `--side/--ours/--theirs`, `--both`, `--pick`,
    /// `--content-file` or `--delete`. A and "ours" are the current
    /// side (`HEAD`; during a rebase, the branch being rebased onto),
    /// B and "theirs" the incoming side. Never edit conflict markers by
    /// hand: pick, or write the final text to a file.
    #[command(group(
        clap::ArgGroup::new("how")
            .required(true)
            .multiple(false)
            .args(["side", "ours", "theirs", "both", "pick", "content_file", "delete"])
    ))]
    Resolve {
        path: String,
        /// Take the whole file from one stage, no auto-merge.
        #[arg(long, value_enum)]
        side: Option<ConflictSideArg>,
        /// Same as `--side ours`.
        #[arg(long)]
        ours: bool,
        /// Same as `--side theirs`.
        #[arg(long)]
        theirs: bool,
        /// Keep both sides of every conflict (A then B), and git's own
        /// merge everywhere else.
        #[arg(long)]
        both: bool,
        /// With `--both`: B then A.
        #[arg(long, requires = "both")]
        theirs_first: bool,
        /// A choice per conflict number: `1=a,2=b,3=ab,4=ba,5=none`
        /// (a = ours, b = theirs, ab / ba = both in that order). Every
        /// conflict needs one, unless `--default` covers the rest.
        #[arg(long, value_name = "N=CHOICE,...")]
        pick: Option<String>,
        /// With `--pick`: the choice for every conflict not listed.
        #[arg(long = "default", value_name = "a|b", requires = "pick")]
        pick_default: Option<String>,
        /// The final file text, exactly (`-` reads standard input).
        #[arg(long, value_name = "FILE")]
        content_file: Option<String>,
        /// With `--content-file`: write the text in this encoding (a
        /// WHATWG label such as `windows-1252`) instead of UTF-8.
        #[arg(long, requires = "content_file")]
        encoding: Option<String>,
        /// Accept the deletion: remove the file (`git rm`).
        #[arg(long)]
        delete: bool,
        /// Write the working tree file but leave it conflicted (a plain
        /// save).
        #[arg(long)]
        no_stage: bool,
        /// The stage oids the decision was made against, `base,ours,theirs`
        /// (use `-` for a missing stage). The call is refused as `stale`
        /// when the index no longer holds exactly these. `--both` and
        /// `--pick` always check the stages they read.
        #[arg(long, value_name = "OID,OID,OID")]
        expect: Option<String>,
    },
    /// Put a file resolved during this operation back to conflicted
    /// (markers again). Refused when it is still conflicted or was not
    /// resolved while the operation ran.
    Unresolve { path: String },
    /// The merge, rebase, cherry-pick or revert in progress (`null`
    /// when none).
    State,
    /// Would a merge of HEAD into the branch it started from conflict, and
    /// on which files? Nothing is touched (an in-memory merge). The base is
    /// the target of the branch's known pull request, else the branch the
    /// Base branches patterns allow with the fewest commits since the fork.
    /// `{"status":"conflicts"|"clean"|"no-base"|"disabled", "base_ref", "files": [...]}`.
    /// Off when `general.conflict_detection` or the repository's switch is off.
    Check,
    /// Abort the operation in progress: HEAD, the index and the working
    /// tree return to before it. Throws away the resolutions made since.
    Abort,
    /// Continue the `git am` in progress: apply the patch it stopped on
    /// (resolve and stage its files first) and the rest of the mailbox.
    /// `{"outcome":"Completed"}` or `"Stopped"` (another patch conflicted).
    AmContinue,
    /// Skip the patch the `git am` in progress stopped on and go on with
    /// the next one.
    AmSkip,
    /// Abort the `git am` in progress: the branch returns to its tip from
    /// before the `am`, and the patches applied so far are dropped.
    AmAbort,
    /// Skip the cherry-pick or revert in progress (or the commit a rebase
    /// is stopped at): HEAD, the index and the working tree return to HEAD.
    /// The way out of one that became empty (the change is already
    /// applied). A merge has nothing to skip.
    Skip,
}

/// `gitpoulpi index-lock ...`: a stale `.git/index.lock`.
#[derive(Debug, Subcommand)]
pub enum IndexLockAction {
    /// The repository's `index.lock` (`null` when there is none): its
    /// path, age in seconds, and whether it is stale (older than 10 s).
    Show,
    /// Delete a stale `index.lock` (left by a git command that died).
    /// Refused while it is younger than 10 s or when there is none. Only
    /// the lock file is removed; check that no git command is running.
    Remove,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ConflictSideArg {
    Ours,
    Theirs,
    Base,
}
