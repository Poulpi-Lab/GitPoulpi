use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{Oid, RefFilter};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogOptions {
    pub limit: Option<u32>,
    /// Drop the first `skip` commits of the walk before applying `limit`
    /// (`git log --skip=<n>`); zero starts at the top. Makes the log
    /// pageable: page `n` is `skip = n * page_size`.
    ///
    /// An offset rather than a "start after this commit" cursor: with
    /// `all_refs` there is no single ref to resume from, and a cursor would
    /// narrow the next page to one branch's ancestry.
    ///
    /// `#[serde(default)]` keeps an older client's JSON (no `skip` key)
    /// deserializing against a newer daemon.
    #[serde(default)]
    pub skip: u32,
    pub start_ref: Option<String>,
    /// When true, walk history from *every* ref (branches, remotes, tags) -
    /// `git log --all` rather than plain `git log`/`git log <ref>` --
    /// so the graph shows every branch's commits, not just the checked-out
    /// one's ancestry. Takes priority over `start_ref` when both are set.
    pub all_refs: bool,
    /// Limit history to commits touching at least one of these paths --
    /// `git log -- <path>...`. Empty means no filter.
    ///
    /// Renames are **not** followed here (`git log --follow` takes one
    /// pathspec only); `GitBackend::file_history` is the rename-following
    /// per-file history.
    pub paths: Vec<PathBuf>,
    /// Show each author as the repository's `.mailmap` names them (git's
    /// `%aN`/`%aE`, libgit2's `Mailmap`). The daemon sets it from
    /// `log.use_mailmap`; a client's value is not trusted.
    #[serde(default)]
    pub use_mailmap: bool,
    /// Refs the `all_refs` walk leaves out: commits only they reach vanish
    /// from the result. Empty (the default, and then absent from the wire)
    /// is exactly the plain all-refs walk. A non-empty filter always keeps
    /// `HEAD` in the walk (the working-tree row and the next commit refer
    /// to it), even when its branch is hidden. Ignored without `all_refs`.
    #[serde(default, skip_serializing_if = "RefFilter::is_empty")]
    pub hidden: RefFilter,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommitOptions {
    pub amend: bool,
    pub sign: bool,
    /// Skip the commit hooks (`--no-verify`: `pre-commit` and `commit-msg`).
    /// Without it both backends run the repository's commit hooks, the way
    /// the system `commit` does.
    #[serde(default)]
    pub no_verify: bool,
    /// Overrides the author identity the commit is made with --
    /// `general.active_profile` in `config.toml`, resolved by the TUI
    /// before this crosses the wire. `None` (the default) leaves both
    /// backends on whatever `git config user.*` says.
    #[serde(default)]
    pub author: Option<CommitAuthor>,
}

/// A commit's author identity, as `CommitOptions::author` overrides it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckoutTarget {
    Branch(String),
    Commit(Oid),
    File {
        path: String,
        source: Oid,
    },
    /// Several files from one commit, as one operation (one journal entry).
    Files {
        paths: Vec<String>,
        source: Oid,
    },
}

/// Whether a merge the user starts may fast-forward (`git.merge_fast_forward`
/// and the repository's own override).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeFastForward {
    /// Always make a merge commit (`--no-ff`), even when the branch could
    /// simply be fast-forwarded.
    #[default]
    Never,
    /// Fast-forward when possible, a merge commit otherwise (git's default,
    /// and `merge.ff` when the repository sets it).
    Allowed,
    /// Fast-forward only (`--ff-only`): a diverged branch is refused with
    /// the typed `NotFastForward`.
    Only,
}

impl MergeFastForward {
    pub const ALL: &'static [MergeFastForward] = &[Self::Never, Self::Allowed, Self::Only];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::Allowed => "allowed",
            Self::Only => "only",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|m| m.as_str() == text.trim())
    }

    /// The repository's own choice wins over the global one; a value that is
    /// not a policy (a hand-edited file) is ignored, never an error.
    pub fn resolve(repo: Option<&str>, global: Self) -> Self {
        repo.and_then(Self::parse).unwrap_or(global)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MergeOptions {
    /// The fast-forward policy of a user-started merge. `None` lets the
    /// daemon resolve it from the settings (repository override, then
    /// `git.merge_fast_forward`) into `no_ff` / `fast_forward_only`; a request
    /// that sets either flag itself is taken as it is.
    pub ff: Option<MergeFastForward>,
    pub fast_forward_only: bool,
    pub no_commit: bool,
    /// Always make a merge commit, even when a fast-forward is possible
    /// (`git merge --no-ff`).
    pub no_ff: bool,
    /// Squash the other branch's commits into one new commit on the
    /// checked-out branch (`git merge --squash`, then a commit). The
    /// commit's message is `message`, or a default naming the branch.
    pub squash: bool,
    /// The message of the merge commit or of the squash commit.
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeOutcome {
    FastForwarded(Oid),
    Merged(Oid),
    UpToDate,
    Conflicted(Vec<String>),
}

/// What [`crate::GitBackend::fast_forward`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FastForwardOutcome {
    /// The branch moved from `from` to `to`.
    FastForwarded { from: Oid, to: Oid },
    /// The branch already contained the target (it *is* the target, or
    /// is ahead of it): nothing moved.
    UpToDate,
}

/// Where a new linked worktree starts (see
/// [`crate::GitBackend::worktree_add`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorktreeStart {
    /// An existing *local* branch, by its short name. Without a new
    /// branch name it is checked out in the new worktree (git refuses a
    /// branch already checked out in another worktree); with one, the
    /// new branch starts at its tip.
    Branch(String),
    /// A commit. Without a new branch name the new worktree gets a
    /// detached HEAD there.
    Commit(Oid),
}

/// The arguments of [`crate::GitBackend::worktree_add`] besides the path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorktreeAddOptions {
    pub start: WorktreeStart,
    /// Create this local branch at `start` and check it out in the new
    /// worktree (`git worktree add -b`). A name already taken is an
    /// error, never a silent reuse.
    pub new_branch: Option<String>,
}

/// What [`crate::GitBackend::patch_create`] turns into a patch file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatchSource {
    /// One or more commits, given oldest-first. Each becomes its own
    /// `git format-patch -1`, concatenated into a single mbox file - so
    /// the result is exactly what applying them one at a time, in this
    /// order, would produce. A merge commit or an unknown oid refuses
    /// the whole request; nothing is written.
    Commits(Vec<Oid>),
    /// Every non-merge commit in `base..tip` (`git format-patch
    /// base..tip`), oldest-first -- merges in the range are skipped
    /// rather than refusing the whole range, since a merge itself never
    /// applies as a patch.
    Range { base: Oid, tip: Oid },
    /// The working tree's current changes against `HEAD`
    /// (`git diff HEAD --binary`), covering both staged and unstaged
    /// modifications to tracked files. `paths` empty means every
    /// changed file; otherwise only those paths.
    ///
    /// Untracked files are deliberately excluded: a working-tree patch
    /// is a diff against `HEAD`, and an untracked file has no `HEAD`
    /// side to diff against - `git diff` never lists it either. Stage
    /// the file first (or use `PatchSource::Commits`) to include it.
    WorkingTree { paths: Vec<String> },
    /// One commit restricted to `paths` (`git format-patch -1 -- paths`):
    /// the patch for just those files of a commit. A merge commit, an
    /// unknown oid, no paths, or paths the commit did not touch refuse.
    CommitPaths { commit: Oid, paths: Vec<String> },
}

/// What [`crate::GitBackend::patch_create`] wrote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchCreated {
    pub path: PathBuf,
    /// How many commits the patch carries. `0` for
    /// [`PatchSource::WorkingTree`], which is not commit-shaped.
    pub commits: usize,
    pub bytes: u64,
}

/// Why [`crate::GitBackend::apply_commit_to_worktree`] refused before
/// touching anything. A patch that merely does not apply is an ordinary
/// error, not one of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplyRefusal {
    /// A merge commit has no single set of changes to replay.
    MergeCommit,
}

/// What [`crate::GitBackend::patch_apply`] changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchApplied {
    /// The paths the patch touched, in the order `git apply` reported
    /// them.
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RebaseOptions {
    pub autosquash: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebaseStep {
    pub commit: Oid,
    pub action: RebaseAction,
    /// The new message for a [`RebaseAction::Reword`] step. Required
    /// (validated by [`validate_plan`]) when `action` is `Reword`; ignored
    /// for every other action.
    #[serde(default)]
    pub message: Option<String>,
}

/// Serializes as the variant name (`"Pick"`); deserializes from that, from
/// the lowercase todo-list names and from their one-letter abbreviations,
/// in any case (see [`RebaseAction::parse`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub enum RebaseAction {
    Pick,
    Squash,
    Fixup,
    Edit,
    Drop,
    Reword,
}

impl RebaseAction {
    /// `pick`/`p`, `reword`/`r`, `edit`/`e`, `squash`/`s`, `fixup`/`f`,
    /// `drop`/`d`, case-insensitively: the spelling of a rebase todo list.
    pub fn parse(text: &str) -> Option<Self> {
        Some(match text.to_ascii_lowercase().as_str() {
            "pick" | "p" => Self::Pick,
            "reword" | "r" => Self::Reword,
            "edit" | "e" => Self::Edit,
            "squash" | "s" => Self::Squash,
            "fixup" | "f" => Self::Fixup,
            "drop" | "d" => Self::Drop,
            _ => return None,
        })
    }
}

impl TryFrom<String> for RebaseAction {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        Self::parse(&text).ok_or_else(|| {
            format!(
                "unknown rebase action {text:?}, expected one of pick, reword, edit, squash, \
                 fixup, drop (or p, r, e, s, f, d)"
            )
        })
    }
}

/// A request to run `git rebase -i` with an explicit, pre-decided todo
/// list rather than an interactively edited one - see
/// `crate::GitBackend::rebase_interactive`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractiveRebase {
    /// The commit the range starts *after* - `None` means `--root`
    /// (rebase the whole history onto `onto`, or in place with no
    /// `onto`).
    pub upstream: Option<Oid>,
    /// Replay onto this commit instead of `upstream` (`git rebase
    /// --onto`). `None` keeps replaying onto `upstream` itself, i.e. a
    /// plain interactive rebase.
    pub onto: Option<Oid>,
    /// The plan, oldest first - must be exactly a permutation of the
    /// range `upstream..HEAD` (or `HEAD` when `upstream` is `None`); see
    /// [`validate_plan`].
    pub steps: Vec<RebaseStep>,
    /// Reorder `steps` with `git rebase --autosquash`'s algorithm before
    /// building the todo list (see [`autosquash_order`]). The backend
    /// itself always calls git with `--no-autosquash`, so this is the
    /// only way autosquash reordering happens.
    pub autosquash: bool,
    /// Cherry-pick mode: `steps` are arbitrary commits (not the range
    /// `upstream..HEAD`) to replay, in order, onto the current HEAD with
    /// the same pick/reword/squash/fixup/edit/drop actions. `upstream`
    /// and `onto` are ignored. Conflicts/stops are handled with the same
    /// rebase continue/skip/abort operations.
    #[serde(default)]
    pub cherry_pick: bool,
}

/// What [`crate::GitBackend::rebase_plan`] reports: the range a rebase
/// from `upstream` would replay, oldest first - the material a frontend
/// builds an [`InteractiveRebase`] plan from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebasePlanDraft {
    pub upstream: Option<Oid>,
    pub commits: Vec<PlanCommit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanCommit {
    pub id: Oid,
    pub summary: String,
    pub message: String,
}

/// The state of a rebase (interactive or plain) currently in progress in
/// the repository, as reported by [`crate::GitBackend::rebase_state`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebaseState {
    /// The branch being rebased, or `None` for a detached HEAD.
    pub head_name: Option<String>,
    pub onto: Oid,
    /// The commit the rebase is currently stopped at (conflict or `edit`
    /// stop), or `None` if it is not stopped (should not normally happen
    /// while `RebaseState` exists, but guards a race reading git's own
    /// state files).
    pub stopped_at: Option<Oid>,
    /// 1-based index of the step currently being applied.
    pub step: usize,
    pub total: usize,
    pub conflicted: bool,
    pub interactive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RebaseOutcome {
    Completed,
    Conflicted {
        paused_at: Oid,
        remaining: Vec<Oid>,
    },
    /// An `edit` step stopped the rebase deliberately - not a conflict.
    /// `at` is the commit now checked out; `remaining` are the steps not
    /// yet applied.
    Stopped {
        at: Oid,
        remaining: Vec<Oid>,
    },
}

/// Why [`crate::GitBackend::rebase_plan`], `rebase_interactive`,
/// `rebase_continue`, `rebase_skip` or `rebase_abort` refused, before
/// touching anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RebaseRefusal {
    /// A merge, rebase, cherry-pick, revert or bisect is already in
    /// progress.
    OperationInProgress,
    UncommittedChanges,
    /// HEAD is detached, or the range crosses onto a commit not on the
    /// current branch.
    NotOnCurrentBranch,
    /// The range `upstream..HEAD` contains a merge commit (more than one
    /// parent) - `RebaseInteractive` never linearises history.
    MergeInRange,
    /// `steps` is not exactly a permutation of the range being rebased.
    PlanMismatch,
    /// A `Squash` or `Fixup` step is the oldest step in the plan, so it
    /// has no predecessor to combine into.
    SquashWithoutPredecessor,
    /// A `Reword` step carries no (or an empty, after normalisation)
    /// message.
    EmptyRewordMessage,
    /// `rebase_continue`/`rebase_skip`/`rebase_abort` was called with no
    /// rebase in progress.
    NoRebaseInProgress,
}

/// Validates that `steps` is exactly a permutation of `range` (same set
/// of oids, no duplicates, no missing ones) and that every `Reword` step
/// carries a non-empty message and no `Squash`/`Fixup` step is first.
///
/// Pure and backend-agnostic so every backend refuses identically and it
/// can be unit tested without a repository.
pub fn validate_plan(
    range: &[Oid],
    steps: &[RebaseStep],
) -> std::result::Result<(), RebaseRefusal> {
    if steps.len() != range.len() {
        return Err(RebaseRefusal::PlanMismatch);
    }
    let mut wanted: Vec<Oid> = range.to_vec();
    wanted.sort_by_key(|o| o.to_hex());
    let mut got: Vec<Oid> = steps.iter().map(|s| s.commit).collect();
    got.sort_by_key(|o| o.to_hex());
    if wanted != got {
        return Err(RebaseRefusal::PlanMismatch);
    }
    if let Some(first) = steps.first() {
        if matches!(first.action, RebaseAction::Squash | RebaseAction::Fixup) {
            return Err(RebaseRefusal::SquashWithoutPredecessor);
        }
    }
    for step in steps {
        if step.action == RebaseAction::Reword
            && normalize_reword_message(step.message.as_deref().unwrap_or_default()).is_none()
        {
            return Err(RebaseRefusal::EmptyRewordMessage);
        }
    }
    Ok(())
}

/// Reorders `rows` the way `git rebase --autosquash` would: a row whose
/// summary starts with `squash! <target>` or `fixup! <target>` moves to
/// sit directly after the row whose summary (after stripping its own
/// `squash!`/`fixup!` prefixes) matches `<target>` - exactly, or as a
/// prefix of it, the way git itself matches - and its action becomes
/// `Squash`/`Fixup`. Rows with no match are left in place. Oldest first,
/// like the plan it reorders.
pub fn autosquash_order(rows: &mut Vec<(Oid, String, RebaseAction)>) {
    fn autosquash_kind(summary: &str) -> Option<(RebaseAction, &str)> {
        if let Some(rest) = summary.strip_prefix("squash! ") {
            Some((RebaseAction::Squash, rest))
        } else if let Some(rest) = summary.strip_prefix("fixup! ") {
            Some((RebaseAction::Fixup, rest))
        } else {
            None
        }
    }

    let original = rows.clone();
    let mut out: Vec<(Oid, String, RebaseAction)> = Vec::with_capacity(original.len());
    let mut consumed = vec![false; original.len()];

    for (i, (_, summary, _)) in original.iter().enumerate() {
        if consumed[i] {
            continue;
        }
        if autosquash_kind(summary).is_some() {
            continue;
        }
        out.push(original[i].clone());
        consumed[i] = true;
        // Any not-yet-placed row targeting this one, in original order.
        for (j, (_, other_summary, _)) in original.iter().enumerate() {
            if consumed[j] {
                continue;
            }
            if let Some((action, target)) = autosquash_kind(other_summary) {
                if summary.starts_with(target) {
                    out.push((original[j].0, original[j].1.clone(), action));
                    consumed[j] = true;
                }
            }
        }
    }
    // Anything left (a squash!/fixup! with no matching target) keeps its
    // original relative order at the end, unchanged (as `Pick`).
    for (i, row) in original.into_iter().enumerate() {
        if !consumed[i] {
            out.push(row);
        }
    }
    *rows = out;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CherryPickOptions {
    pub no_commit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CherryPickOutcome {
    Completed(Vec<Oid>),
    Conflicted { paused_at: Oid, remaining: Vec<Oid> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevertOutcome {
    Completed(Oid),
    Conflicted,
}

/// What [`crate::GitBackend::reword`] did.
///
/// `previous_head` is the tip the branch (or a detached HEAD) had before
/// the rewrite. Both backends also leave it in `ORIG_HEAD` and in the
/// branch's reflog, so "put it back" is `reset --soft <previous_head>`
/// whether or not the frontend kept this value around.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewordOutcome {
    /// The new commit carrying the new message (the rewritten `commit`).
    pub reworded: Oid,
    /// HEAD before the rewrite.
    pub previous_head: Oid,
    /// HEAD after it. Equal to `reworded` when the reworded commit was
    /// HEAD itself.
    pub new_head: Oid,
    /// How many descendants of the reworded commit were re-created on
    /// top of it (0 when it was HEAD).
    pub replayed: usize,
}

/// Why [`crate::GitBackend::reword`] refused, before touching anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RewordRefusal {
    /// The commit is not HEAD nor an ancestor of HEAD, so there is no
    /// current-branch history to rewrite it in.
    NotOnCurrentBranch,
    /// A merge, rebase, cherry-pick, revert or bisect is in progress.
    OperationInProgress,
    /// The new message is empty once trailing whitespace is removed.
    EmptyMessage,
    /// An amended name or email is empty, or holds `<`, `>`, a line break
    /// (or, for an email, a space).
    InvalidIdentity,
    /// An amended date is neither git's raw format nor ISO 8601.
    InvalidDate,
}

/// The message [`crate::GitBackend::reword`] stores for `message`, or
/// `None` when nothing is left.
///
/// One rule shared by both backends so the new commit's oid is the same
/// whichever wrote it: CRLF becomes LF, leading blank lines and trailing
/// whitespace go, and the message ends in exactly one `\n`, as
/// `git commit` leaves it.
pub fn normalize_reword_message(message: &str) -> Option<String> {
    let unix = message.replace("\r\n", "\n");
    let mut rest = unix.as_str();
    while let Some((line, after)) = rest.split_once('\n') {
        if !line.trim().is_empty() {
            break;
        }
        rest = after;
    }
    let trimmed = rest.trim_end();
    if trimmed.is_empty() {
        None
    } else {
        Some(format!("{trimmed}\n"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResetMode {
    Soft,
    Mixed,
    Hard,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FetchOptions {
    /// `--prune`: drop remote-tracking branches the remote no longer has.
    pub prune: bool,
    /// Also delete the **local** tags that were on a remote and that this
    /// fetch learned the remote has deleted (git's `fetch.pruneTags`, but
    /// narrower: a tag that was never on the remote - an unpushed one - is
    /// never touched, and neither is one that moved locally). Needs git
    /// 2.41+ to learn what was pruned; off by default, like git.
    pub prune_tags: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FetchOutcome {
    pub updated_refs: Vec<String>,
    /// Local tags deleted because `prune_tags` was on and the remote had
    /// deleted them.
    #[serde(default)]
    pub pruned_tags: Vec<String>,
}

/// How a pull integrates what it fetched.
///
/// Always passed to git *explicitly* (`--no-rebase`, `--ff-only`,
/// `--rebase`), so a `pull.rebase` or `pull.ff` in the user's own config
/// cannot change what the button says.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullMode {
    /// Fast-forward when possible, otherwise a merge commit
    /// (`git pull --no-rebase`).
    #[default]
    Merge,
    /// Fast-forward or fail (`git pull --ff-only`). A diverged branch is
    /// the typed [`crate::Error::NotFastForward`], never a merge.
    FastForwardOnly,
    /// Replay local commits on top of the upstream (`git pull --rebase`).
    Rebase,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PullOptions {
    pub mode: PullMode,
    /// Pull the same-named branch from this remote instead of the
    /// branch's configured upstream - what a branch with no upstream
    /// needs. `None` is plain `git pull`: the upstream.
    pub remote: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PullOutcome {
    UpToDate,
    FastForwarded(Oid),
    Merged(Oid),
    Conflicted,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PushOptions {
    /// Overwrite the remote branch, but only if it is still where our
    /// remote-tracking ref says it is: `--force-with-lease`. There is
    /// deliberately no way to ask for a bare `--force` - a force push
    /// that silently discards a colleague's commits pushed since our last
    /// fetch is the one push mistake nobody sees coming.
    ///
    /// `force` is accepted as a spelling on the wire for older clients,
    /// and means the same lease.
    #[serde(alias = "force")]
    pub force_with_lease: bool,
    /// `--set-upstream`: record `<remote>/<branch>` as the branch's
    /// upstream - what the first push of a new branch wants.
    pub set_upstream: bool,
    /// The local branch to push. `None` is the checked-out branch.
    pub branch: Option<String>,
}

/// Why a remote refused a push, when it is a refusal the user can act on
/// rather than an arbitrary failure (see [`crate::Error::PushRejected`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PushRejectReason {
    /// The remote branch has commits the local one does not - git's
    /// `non-fast-forward` / `fetch first`. The remote has diverged.
    NonFastForward,
    /// A `--force-with-lease` push found the remote branch somewhere
    /// other than where our remote-tracking ref said: someone pushed
    /// since our last fetch, and the lease protected their work.
    StaleLease,
}

/// A push the remote rejected, typed so a frontend can offer "pull
/// first / force push / cancel" instead of printing git's stderr.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushRejection {
    pub remote: String,
    pub branch: String,
    pub reason: PushRejectReason,
    /// The command and its stderr, for the activity log. Absent on the wire
    /// from a peer that predates it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<Box<crate::failure::ErrorDetail>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PushOutcome {
    pub pushed_refs: Vec<String>,
}

/// Which tags [`crate::GitBackend::tag_push`] should send.
///
/// `AllUnpushed` is a *selection* resolved inside the backend, not a list
/// the caller built by looping over `refs()`: it is one operation (one
/// result, one `RepoChanged`), one `git push` (hooks and credential
/// prompts run once), and the set is resolved **at push time** from the
/// mirror namespace that drew the markers, so a tag created after the
/// frontend last refreshed is included.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TagSelection {
    /// Exactly these tags, in this order. An unknown name is an error:
    /// "push v9" quietly pushing nothing is the worst possible answer.
    Named(Vec<String>),
    /// Every local tag no remote is known to have (see
    /// [`super::TagSync`]). An empty set is a *success* reporting no
    /// pushed refs - "everything is already pushed" is not a failure.
    AllUnpushed,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloneOptions {
    pub shallow_depth: Option<u32>,
    pub recurse_submodules: bool,
    /// Check out this branch instead of the remote's default HEAD --
    /// `git clone --branch <name>`. `None` keeps git's own default.
    #[serde(default)]
    pub branch: Option<String>,
    /// Name the new remote something other than `origin` --
    /// `git clone --origin <name>`.
    #[serde(default)]
    pub remote_name: Option<String>,
    /// Whether to download Git LFS objects after cloning (default: yes,
    /// when `git-lfs` is installed). A clone never fails because of LFS.
    #[serde(default)]
    pub lfs: super::LfsCloneMode,
    /// Sparse checkout in cone mode: only these top-level or nested
    /// directories (plus the files at the repository root) are checked out;
    /// the full history is still downloaded. `None` is a normal full clone.
    /// Normalised with [`normalize_sparse_dirs`] before use.
    #[serde(default)]
    pub sparse: Option<Vec<String>>,
}

/// What a repository's sparse checkout looks like right now
/// (`GitBackend::sparse_checkout_status`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SparseState {
    /// `core.sparseCheckout` is on: part of the tree is not materialised.
    pub enabled: bool,
    /// Cone mode (directories, the git default). A repository set up in
    /// non-cone mode with patterns elsewhere reports `false` and the raw
    /// patterns in `dirs`.
    pub cone: bool,
    /// The checked-out directories, `/`-separated and relative to the root.
    /// Empty when not enabled.
    pub dirs: Vec<String>,
}

/// Cleans the directories of a sparse-checkout request: backslashes become
/// `/`, surrounding and trailing slashes and a leading `./` go, duplicates
/// collapse (order kept). Refuses an empty list, an empty or absolute
/// path, `..`, and anything git would read as an option or a pattern.
pub fn normalize_sparse_dirs(dirs: &[String]) -> crate::Result<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    for raw in dirs {
        let mut d = raw.trim().replace('\\', "/");
        while let Some(rest) = d.strip_prefix("./") {
            d = rest.to_string();
        }
        let d = d.trim_matches('/').to_string();
        if d.is_empty() || d == "." {
            return Err(crate::Error::Backend(format!(
                "`{raw}` is not a directory of the repository"
            )));
        }
        if raw.trim().starts_with('/')
            || d.starts_with('-')
            || d.starts_with('!')
            || d.starts_with('#')
            || d.split('/').any(|c| c == ".." || c == "." || c.is_empty())
            || d.contains(['*', '?', '[', ':'])
            || d.chars().any(char::is_control)
        {
            return Err(crate::Error::Backend(format!(
                "`{raw}` is not a valid sparse-checkout directory (use a path relative to the repository root, without patterns)"
            )));
        }
        if !out.contains(&d) {
            out.push(d);
        }
    }
    if out.is_empty() {
        return Err(crate::Error::Backend(
            "a sparse checkout needs at least one directory".to_string(),
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod sparse_tests {
    use super::normalize_sparse_dirs;

    fn n(v: &[&str]) -> crate::Result<Vec<String>> {
        normalize_sparse_dirs(&v.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn cleans_slashes_dots_and_duplicates() {
        assert_eq!(
            n(&["./src/", "docs\\guide", " src "]).unwrap(),
            ["src", "docs/guide"]
        );
    }

    #[test]
    fn refuses_what_is_not_a_plain_relative_dir() {
        for bad in ["", "/", "/etc", "..", "a/../b", "-x", "!x", "a/*", "a:b"] {
            assert!(n(&[bad]).is_err(), "{bad:?} must be refused");
        }
        assert!(n(&[]).is_err());
    }
}

/// What `GitBackend::init` should make.
///
/// Deliberately tiny: anything else belongs in the user's own git
/// configuration, which `init` does not override.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitOptions {
    /// A repository with no working tree (`git init --bare`).
    #[serde(default)]
    pub bare: bool,
    /// The name of the branch HEAD points at before the first commit --
    /// `git init --initial-branch <name>`. `None` leaves git's own default
    /// (`init.defaultBranch`, or `master`).
    #[serde(default)]
    pub initial_branch: Option<String>,
}

/// How far along a `clone` is, pushed to the UI while the clone runs. The
/// phases mirror git's own progress vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClonePhase {
    /// The connection is open, nothing has been counted yet.
    Connecting,
    /// The remote is enumerating what it will send.
    Counting,
    /// The remote is compressing objects.
    Compressing,
    /// Objects are arriving.
    Receiving,
    /// Deltas are being applied locally.
    Resolving,
    /// The working tree is being written.
    CheckingOut,
    /// Git LFS objects are being downloaded after the clone.
    LfsObjects,
    /// The clone finished; `percent` is 100.
    Done,
}

impl ClonePhase {
    /// The label a progress bar shows. Plain ASCII words, so the value is
    /// safe in a terminal at any font (see the emoji-presentation rule in
    /// `docs/developer/where-we-are.md`).
    pub fn label(self) -> &'static str {
        match self {
            Self::Connecting => "Connecting",
            Self::Counting => "Counting objects",
            Self::Compressing => "Compressing objects",
            Self::Receiving => "Receiving objects",
            Self::Resolving => "Resolving deltas",
            Self::CheckingOut => "Checking out files",
            Self::LfsObjects => "Downloading LFS objects",
            Self::Done => "Done",
        }
    }
}

/// One progress report from an in-flight clone.
///
/// `percent` is always present and always 0..=100, so a progress bar is
/// drawable from this type alone. `current`/`total` carry the raw counts
/// when git reported them, for a `(450/1000)` suffix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloneProgress {
    pub phase: ClonePhase,
    pub percent: u8,
    pub current: Option<u64>,
    pub total: Option<u64>,
}

impl CloneProgress {
    pub fn new(phase: ClonePhase, percent: u8) -> Self {
        Self {
            phase,
            percent: percent.min(100),
            current: None,
            total: None,
        }
    }

    pub fn with_counts(mut self, current: u64, total: u64) -> Self {
        self.current = Some(current);
        self.total = Some(total);
        self
    }

    pub fn done() -> Self {
        Self::new(ClonePhase::Done, 100)
    }
}

/// The sink a backend reports clone progress to.
///
/// A plain callback rather than a channel, so `gitpoulpi-core` keeps no
/// I/O-framework dependency: the daemon wraps a `broadcast::Sender`, the
/// CLI wraps stderr, a test wraps a `Vec`.
pub type CloneProgressSink<'a> = &'a (dyn Fn(CloneProgress) + Send + Sync);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_merge_fast_forward_policy_defaults_to_never_and_the_repo_wins() {
        use MergeFastForward as P;
        assert_eq!(P::default(), P::Never, "the owner's expectation");
        // The repository's choice wins over the global one, either way.
        assert_eq!(P::resolve(Some("allowed"), P::Never), P::Allowed);
        assert_eq!(P::resolve(Some("never"), P::Allowed), P::Never);
        assert_eq!(P::resolve(Some("only"), P::Allowed), P::Only);
        // Nothing, or a value that is no policy: the global one.
        assert_eq!(P::resolve(None, P::Only), P::Only);
        assert_eq!(P::resolve(Some("sometimes"), P::Allowed), P::Allowed);
        assert_eq!(P::resolve(Some(""), P::Never), P::Never);
        for p in P::ALL {
            assert_eq!(P::parse(p.as_str()), Some(*p));
            assert_eq!(
                serde_json::to_string(p).unwrap(),
                format!("\"{}\"", p.as_str()),
                "the wire spelling is the config spelling"
            );
        }
    }

    fn oid(byte: u8) -> Oid {
        Oid([byte; 20])
    }

    #[test]
    fn validate_plan_accepts_any_permutation_of_the_range() {
        let range = [oid(1), oid(2), oid(3)];
        let steps = vec![
            RebaseStep {
                commit: oid(3),
                action: RebaseAction::Pick,
                message: None,
            },
            RebaseStep {
                commit: oid(1),
                action: RebaseAction::Pick,
                message: None,
            },
            RebaseStep {
                commit: oid(2),
                action: RebaseAction::Squash,
                message: None,
            },
        ];
        assert_eq!(validate_plan(&range, &steps), Ok(()));
    }

    #[test]
    fn validate_plan_rejects_a_dropped_or_duplicated_commit() {
        let range = [oid(1), oid(2)];
        let missing = vec![
            RebaseStep {
                commit: oid(1),
                action: RebaseAction::Pick,
                message: None,
            },
            RebaseStep {
                commit: oid(1),
                action: RebaseAction::Pick,
                message: None,
            },
        ];
        assert_eq!(
            validate_plan(&range, &missing),
            Err(RebaseRefusal::PlanMismatch)
        );
    }

    #[test]
    fn validate_plan_refuses_squash_or_fixup_as_the_oldest_step() {
        let range = [oid(1), oid(2)];
        let steps = vec![
            RebaseStep {
                commit: oid(1),
                action: RebaseAction::Fixup,
                message: None,
            },
            RebaseStep {
                commit: oid(2),
                action: RebaseAction::Pick,
                message: None,
            },
        ];
        assert_eq!(
            validate_plan(&range, &steps),
            Err(RebaseRefusal::SquashWithoutPredecessor)
        );
    }

    #[test]
    fn validate_plan_refuses_an_empty_reword_message() {
        let range = [oid(1)];
        let steps = vec![RebaseStep {
            commit: oid(1),
            action: RebaseAction::Reword,
            message: Some("  \r\n ".into()),
        }];
        assert_eq!(
            validate_plan(&range, &steps),
            Err(RebaseRefusal::EmptyRewordMessage)
        );
    }

    #[test]
    fn autosquash_order_moves_a_fixup_row_after_its_target_and_relabels_it() {
        let mut rows = vec![
            (oid(1), "base".to_string(), RebaseAction::Pick),
            (oid(2), "feature work".to_string(), RebaseAction::Pick),
            (
                oid(3),
                "fixup! feature work".to_string(),
                RebaseAction::Pick,
            ),
        ];
        autosquash_order(&mut rows);
        assert_eq!(
            rows,
            vec![
                (oid(1), "base".to_string(), RebaseAction::Pick),
                (oid(2), "feature work".to_string(), RebaseAction::Pick),
                (
                    oid(3),
                    "fixup! feature work".to_string(),
                    RebaseAction::Fixup
                ),
            ]
        );
    }

    #[test]
    fn autosquash_order_leaves_an_unmatched_squash_row_in_place() {
        let mut rows = vec![
            (
                oid(1),
                "squash! nothing matches".to_string(),
                RebaseAction::Pick,
            ),
            (oid(2), "unrelated".to_string(), RebaseAction::Pick),
        ];
        autosquash_order(&mut rows);
        assert_eq!(
            rows,
            vec![
                (oid(2), "unrelated".to_string(), RebaseAction::Pick),
                (
                    oid(1),
                    "squash! nothing matches".to_string(),
                    RebaseAction::Pick
                ),
            ]
        );
    }

    #[test]
    fn a_reword_message_is_normalized_the_way_git_commit_leaves_it() {
        assert_eq!(
            normalize_reword_message("  \r\n\nsubject\r\n\r\n  body  \r\n\n").as_deref(),
            Some("subject\n\n  body\n")
        );
        assert_eq!(
            normalize_reword_message("one line").as_deref(),
            Some("one line\n")
        );
        assert_eq!(normalize_reword_message(" \r\n\t\n  "), None);
        assert_eq!(normalize_reword_message(""), None);
    }
}

/// The namespace `GitBackend::fetch_refspec` may write into: refs that
/// are neither branches, tags nor remote-tracking refs, so a fetch there
/// never shows up in the user's ref lists and cannot move anything they
/// work on.
pub const FETCH_REFSPEC_NAMESPACE: &str = "refs/gitpoulpi/";

/// Validates a `fetch_refspec` spec (`[+]refs/<src>:refs/gitpoulpi/<dst>`)
/// and returns `(source, destination)`; `None` for anything else. Shared
/// by both backends so they refuse exactly the same specs: one ref per
/// spec, no wildcard, no option-looking or whitespace-carrying text, a
/// full source ref, and a destination strictly inside
/// [`FETCH_REFSPEC_NAMESPACE`].
pub fn parse_fetch_refspec(spec: &str) -> Option<(&str, &str)> {
    let spec = spec.strip_prefix('+').unwrap_or(spec);
    let (src, dst) = spec.split_once(':')?;
    let clean = |r: &str| {
        !r.is_empty()
            && !r.contains("..")
            && !r.contains("//")
            && !r.ends_with('/')
            && !r.ends_with(".lock")
            && !r.contains("@{")
            && !r
                .chars()
                .any(|c| c.is_control() || c.is_whitespace() || "~^:?*[\\".contains(c))
    };
    (src.starts_with("refs/")
        && clean(src)
        && dst.starts_with(FETCH_REFSPEC_NAMESPACE)
        && dst.len() > FETCH_REFSPEC_NAMESPACE.len()
        && clean(dst))
    .then_some((src, dst))
}

#[cfg(test)]
mod fetch_refspec_tests {
    use super::parse_fetch_refspec;

    #[test]
    fn only_namespaced_single_ref_specs_pass() {
        assert_eq!(
            parse_fetch_refspec("+refs/pull/7/head:refs/gitpoulpi/pr/7"),
            Some(("refs/pull/7/head", "refs/gitpoulpi/pr/7"))
        );
        for bad in [
            "refs/pull/7/head:refs/heads/x",
            "refs/pull/*/head:refs/gitpoulpi/pr/*",
            "refs/pull/7/head:refs/gitpoulpi/",
            "refs/pull/7/head",
            "--x:refs/gitpoulpi/a",
            "refs/a b:refs/gitpoulpi/a",
        ] {
            assert!(parse_fetch_refspec(bad).is_none(), "{bad}");
        }
    }
}
