use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum BranchAction {
    Create {
        name: String,
        at: Option<String>,
    },
    /// Create a local branch tracking a remote-tracking branch and check
    /// it out. `<upstream>` is spelled as the refs listing spells it
    /// (`origin/feature/login`); `--name` defaults to the branch part.
    Track {
        upstream: String,
        #[arg(long)]
        name: Option<String>,
    },
    Delete {
        name: String,
    },
    /// Delete a branch from a remote, keeping any local branch of that
    /// name.
    DeleteRemote {
        name: String,
        #[arg(long, default_value = "origin")]
        remote: String,
    },
    Rename {
        old_name: String,
        new_name: String,
    },
    /// Make `<branch>` track `<upstream>` (`origin/main`).
    SetUpstream {
        branch: String,
        upstream: String,
    },
    /// Remove `<branch>`'s upstream.
    UnsetUpstream {
        branch: String,
    },
    /// Move `<branch>` forward to `<target>`, only if that is a
    /// fast-forward (diverged history is refused, nothing moves). The
    /// checked-out branch takes its working tree along, like `git merge
    /// --ff-only`; any other branch just has its ref moved.
    FastForward {
        branch: String,
        /// Any revision (default: HEAD).
        target: Option<String>,
    },
    List,
    /// A branch's several remotes: the primary upstream plus extra push
    /// targets and opt-in pull sources, kept in its own config section.
    Upstream {
        #[command(subcommand)]
        action: UpstreamAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum UpstreamAction {
    /// The branch's primary upstream, its extra push targets and pull
    /// sources, each with where it stands (ahead, behind, relation) from
    /// the remote-tracking refs as they are. Reads only, no network.
    List {
        /// The branch (default: the current one).
        #[arg(long)]
        branch: Option<String>,
    },
    /// Add `<remote>[:<dst>]` as an extra push target (`<dst>` is the
    /// branch name on that remote, default the same name). Undoable.
    Add {
        /// `<remote>` or `<remote>:<dst>`.
        target: String,
        /// The branch (default: the current one).
        #[arg(long)]
        branch: Option<String>,
        /// Also make it a pull source.
        #[arg(long)]
        pull: bool,
    },
    /// Remove every extra push target and pull source naming `<remote>`.
    /// The primary upstream is left alone (use `unset-upstream`). Undoable.
    Remove {
        remote: String,
        /// The branch (default: the current one).
        #[arg(long)]
        branch: Option<String>,
    },
    /// Turn `<remote>[:<dst>]` on or off as a pull source: a plain pull
    /// then integrates it too.
    Pull {
        /// `<remote>` or `<remote>:<branch>`.
        target: String,
        /// Make it a pull source.
        #[arg(long, conflicts_with = "off", required_unless_present = "off")]
        on: bool,
        /// Stop pulling from it.
        #[arg(long)]
        off: bool,
        /// The branch (default: the current one).
        #[arg(long)]
        branch: Option<String>,
    },
    /// Set the primary upstream (`origin/main`): the same request as
    /// `branch set-upstream`.
    Primary {
        upstream: String,
        /// The branch (default: the current one).
        #[arg(long)]
        branch: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum RemoteGroupAction {
    /// Where the current branch (or `--branch`) stands against each of its
    /// remotes: primary upstream, extra push targets, pull sources. Reads
    /// only, no network.
    Status {
        /// The branch (default: the current one).
        #[arg(long)]
        branch: Option<String>,
    },
    /// What a grouped push (or, with `--pull`, a grouped pull) would do,
    /// remote by remote: where each stands, the suggested action and the
    /// others allowed. Asks the remotes (`ls-remote`, a fetch of the ones
    /// whose tip is unknown) unless `--no-preflight`; changes nothing.
    Plan {
        /// The branch (default: the current one).
        #[arg(long)]
        branch: Option<String>,
        /// Plan a push to exactly these remotes (`<remote>` or
        /// `<remote>:<dst>`, repeatable) instead of the configured targets.
        #[arg(long = "to", conflicts_with = "pull")]
        to: Vec<String>,
        /// Plan the grouped pull (the primary and the pull sources).
        #[arg(long)]
        pull: bool,
        /// Do not ask the remotes: use the remote-tracking refs as they are.
        #[arg(long)]
        no_preflight: bool,
    },
    /// Resume the saved grouped run once its merge or rebase is concluded
    /// (never automatic). Answers a `needs_resolution` with `--resolve`.
    Continue {
        /// The saved run's id, to be sure it is still the one meant.
        #[arg(long)]
        run: Option<String>,
        /// `REMOTE=push|ff|merge|rebase|force|skip` (repeatable): the
        /// decision for a remote the previous answer asked about.
        #[arg(long = "resolve")]
        resolve: Vec<String>,
        /// Confirm the force pushes named by `--resolve`.
        #[arg(long)]
        yes: bool,
        /// The branch (default: the current one); only read with `--resolve`.
        #[arg(long)]
        branch: Option<String>,
    },
    /// Drop the saved grouped run: aborts the merge or rebase it started.
    /// Pushes already done stay.
    Abort {
        /// The saved run's id, to be sure it is still the one meant.
        #[arg(long)]
        run: Option<String>,
    },
    /// Run again the targets of the saved run that failed, were canceled
    /// mid-push or were never attempted.
    Retry {
        /// The saved run's id, to be sure it is still the one meant.
        #[arg(long)]
        run: Option<String>,
    },
}

#[cfg(feature = "worktree")]
#[derive(Debug, Subcommand)]
pub enum WorktreeAction {
    /// Every worktree of the repository, the main one first.
    List,
    /// Add a linked worktree at `<path>`. `<start>` naming a local branch
    /// checks that branch out there (or starts `--branch` at it); any
    /// other revision gives a detached HEAD (or starts `--branch` there).
    Add {
        path: String,
        /// A local branch, or any revision.
        start: String,
        /// Create this new branch at `<start>` and check it out there.
        #[arg(long, short = 'b')]
        branch: Option<String>,
    },
    /// The uncommitted changes of the worktree at `<path>` (tracked and
    /// untracked, not ignored); an empty list means it is clean.
    Changes { path: String },
    /// Remove the linked worktree at `<path>`. A worktree with uncommitted
    /// changes is refused unless `--force` (they are lost), `--commit-message`
    /// (commit every change on its branch first; a detached worktree also
    /// needs `--commit-branch`) or `--stash-message` (stash every change,
    /// untracked included, first; the stash is shared by all worktrees).
    /// `--delete-branch` also deletes its branch (journaled: undo brings the
    /// branch back; the folder stays gone). The removal itself is not undoable.
    Remove {
        path: String,
        /// Remove even with uncommitted changes; they are lost.
        #[arg(long)]
        force: bool,
        /// Commit every change with this message before removing.
        #[arg(long)]
        commit_message: Option<String>,
        /// The branch to create for that commit when the worktree is detached.
        #[arg(long)]
        commit_branch: Option<String>,
        /// Stash every change with this message before removing.
        #[arg(long)]
        stash_message: Option<String>,
        /// Also delete the worktree's branch after removing it.
        #[arg(long)]
        delete_branch: bool,
    },
}

#[cfg(feature = "submodule")]
#[derive(Debug, Subcommand)]
pub enum SubmoduleAction {
    /// Every submodule with its state: `uninitialized` (not cloned),
    /// `up_to_date`, `modified` (checked out at another commit than the
    /// recorded one) or `conflicted`.
    List,
    /// Register submodules in `.git/config` without cloning them
    /// (`git submodule init`). No paths: all of them.
    Init {
        /// Repo-relative submodule paths.
        paths: Vec<String>,
    },
    /// Check submodules out at the commits the repository records
    /// (`git submodule update`). No paths: all of them.
    Update {
        /// Repo-relative submodule paths.
        paths: Vec<String>,
        /// Clone and register the ones that are not yet (`--init`).
        #[arg(long)]
        init: bool,
        /// Also update the submodules' own submodules.
        #[arg(long)]
        recursive: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum PatchAction {
    /// Write a patch file. `<revs>` are one or more commits (oldest-first)
    /// to turn into a patch each; combine with `--range a..b` instead to
    /// cover every non-merge commit in a range; combine with `--wip`
    /// instead to patch the working tree's current changes (staged and
    /// unstaged), optionally limited to `<paths>`.
    Create {
        /// Commit revisions (oldest-first) with no `--range`/`--wip`;
        /// with `--wip`, paths to limit the patch to (empty means every
        /// changed file); ignored with `--range`.
        revs: Vec<String>,
        #[arg(long, value_name = "BASE..TIP", conflicts_with_all = ["wip"])]
        range: Option<String>,
        #[arg(long, conflicts_with_all = ["range"])]
        wip: bool,
        #[arg(long, short = 'o')]
        output: String,
    },
    /// Apply a patch file to the working tree.
    Apply { path: String },
}

/// A switch spelled out, for flags that must be able to say "off".
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OnOff {
    On,
    Off,
}

/// Where the local agent of a review runs (`[ai.review] run_in_repository`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum RunInRepoArg {
    /// Wherever the AI profile's `workdir` says.
    Profile,
    /// In the repository.
    Yes,
    /// Not in the repository.
    No,
}

impl RunInRepoArg {
    pub fn to_config(self) -> gitpoulpi_config::RunInRepo {
        match self {
            Self::Profile => gitpoulpi_config::RunInRepo::Profile,
            Self::Yes => gitpoulpi_config::RunInRepo::Yes,
            Self::No => gitpoulpi_config::RunInRepo::No,
        }
    }
}

/// The review-context flags of `hosting review suggest` and
/// `ai review-context`, as the choices of one run (or one repository).
#[derive(Debug, Clone, Default)]
pub struct ReviewFlags {
    pub run_in_repo: Option<RunInRepoArg>,
    pub claude_md: Option<OnOff>,
    pub agents_md: Option<OnOff>,
    pub context_file: Option<String>,
    pub no_context_file: bool,
    /// `Some(On)`: this repository (or project) has no extra file, whatever
    /// the global value says; `Some(Off)`: it follows the global one again.
    pub no_extra_file: Option<OnOff>,
}

impl ReviewFlags {
    /// What the flags say; `None` for a field they leave alone.
    pub fn values(&self) -> gitpoulpi_config::RepoAiReview {
        gitpoulpi_config::RepoAiReview {
            run_in_repository: self.run_in_repo.map(RunInRepoArg::to_config),
            include_claude_md: self.claude_md.map(|v| v == OnOff::On),
            include_agents_md: self.agents_md.map(|v| v == OnOff::On),
            extra_file: if self.no_context_file {
                Some(String::new())
            } else {
                self.context_file.clone()
            },
            no_extra_file: self.no_extra_file.map(|v| v == OnOff::On),
        }
    }
}

#[cfg(feature = "ai")]
#[derive(Debug, Subcommand)]
pub enum AiCommand {
    /// Show or set what a ✦ code review draft is given besides the diff
    /// (`[ai.review]`): the repository's CLAUDE.md and AGENTS.md, one more
    /// file (a path or a link), and whether a local agent runs in the
    /// repository. With neither `--repo` nor `--project` it is the global
    /// default; `--repo PATH` is one cloned repository's own value,
    /// `--project HOST/PROJECT` the one of a repository that is not cloned.
    /// Without a setting flag it prints the values (`effective` is what a
    /// review gets); with one it saves it and prints the result. Needs no
    /// model and works with AI turned off.
    ReviewContext {
        /// A repository that is not cloned, as `HOST/PROJECT`
        /// (`github.com/owner/name`).
        #[arg(long, value_name = "HOST/PROJECT")]
        project: Option<String>,
        /// Run a local agent in the repository (`yes`), not there (`no`) or
        /// as its AI profile says (`profile`). Not for `--project`.
        #[arg(long, value_enum)]
        run_in_repo: Option<RunInRepoArg>,
        /// Give the model the repository's CLAUDE.md.
        #[arg(long, value_enum)]
        claude_md: Option<OnOff>,
        /// Give the model the repository's AGENTS.md.
        #[arg(long, value_enum)]
        agents_md: Option<OnOff>,
        /// One more file: a path relative to the repository root, an
        /// absolute path, or a link to a file of a code host.
        #[arg(long, value_name = "PATH|URL", conflicts_with = "no_context_file")]
        context_file: Option<String>,
        /// No extra file.
        #[arg(long)]
        no_context_file: bool,
        /// No extra file for the repository or project, whatever the global
        /// value says (`on`, the default when given alone); `off` makes it
        /// follow the global value again.
        #[arg(
            long,
            value_enum,
            num_args = 0..=1,
            default_missing_value = "on",
            conflicts_with_all = ["context_file", "no_context_file"]
        )]
        no_extra_file: Option<OnOff>,
        /// With `--repo` or `--project`: forget the repository's own values
        /// (it follows the global ones again) before applying the flags.
        #[arg(long)]
        reset: bool,
    },
    /// Print the commit message the AI proposes for the staged changes
    /// (or every change when nothing is staged). Commits nothing.
    CommitMessage,
    /// Explain a commit (default HEAD) or, with --wip, the working changes.
    Explain {
        rev: Option<String>,
        #[arg(long, conflicts_with = "rev")]
        wip: bool,
    },
    /// Print the resolution the AI suggests for ONE merge conflict of a
    /// conflicted file (`--conflict N`, 1-based; optional when the file has
    /// a single conflict). Prints only: nothing is written or staged, so
    /// the caller decides (`conflicts resolve --content-file` applies a
    /// whole-file result). Needs `ai.enabled = true`.
    ResolveConflict {
        /// The conflicted file, relative to the repository root.
        path: String,
        /// Which conflict of the file, 1-based.
        #[arg(long)]
        conflict: Option<usize>,
    },
    /// Print a plan the AI proposes for splitting a commit into several
    /// commits, in the JSON form `split-commit --plan` accepts. Applies
    /// nothing: review it, then pass it to `split-commit`. Needs
    /// `ai.enabled = true`.
    SplitPlan {
        /// The commit to split (any revision: a sha, `HEAD~2`, ...).
        rev: String,
    },
    #[cfg(feature = "history-rewrite")]
    /// Print a Lua script the AI writes for a history rewrite from your
    /// instruction ("move every commit made at night to the next morning"),
    /// with the result of `rewrite script check` on it. Runs nothing and
    /// changes nothing: read the script, then `rewrite script test` and
    /// `rewrite preview --script`. Sends only your instruction, plus, with
    /// `--sample N`, the author, date and subject of N commits (never file
    /// contents). Needs `ai.enabled = true`.
    RewriteScript {
        /// What the script must do, in your own words.
        instruction: String,
        /// Also send the author, date and subject of the newest N commits
        /// (at most 50) so the model sees what the data looks like. Off by
        /// default: without it no commit data leaves the machine.
        #[arg(long)]
        sample: Option<usize>,
    },
    #[cfg(feature = "history-rewrite")]
    /// Print the rules and settings the AI proposes for a history rewrite from
    /// your description ("replace my old email a@b with c@d on every commit",
    /// "remove secrets.txt from all history"), as the JSON recipe
    /// `rewrite preview --recipe-json` reads (save the `recipe` field to a file for `--recipe`), plus what it left out and why.
    /// Previews nothing and applies nothing: read the rules, then run
    /// `rewrite preview` and `rewrite apply` yourself. Sends only your
    /// description, plus, with `--context`, the current branch, the branch
    /// and tag names and the authors of the newest 200 commits (never file
    /// contents). Needs `ai.enabled = true`.
    RewriteRules {
        /// How the history must be rewritten, in your own words.
        description: String,
        /// Also send the branch and tag names and the authors of the newest
        /// 200 commits (at most 100 branch and tag names and 100 authors) so
        /// the model can name them exactly.
        /// Off by default: without it nothing but your description leaves
        /// the machine.
        #[arg(long)]
        context: bool,
    },
    #[cfg(feature = "worktree")]
    /// Print the worktree name and branch name the AI proposes for a task
    /// ("fix the crash when the config is empty"): `{ name, worktree,
    /// branch }`, where `worktree` is the name the Create worktree dialog
    /// would fill (`worktree-<name>`) and `branch` follows the repository's
    /// own branch naming. Creates nothing: pass them to `worktree add`.
    /// Sends only your description and the local branch names. Needs
    /// `ai.enabled = true`.
    WorktreeName {
        /// The feature, bug or task, in your own words.
        #[arg(long, value_name = "TEXT")]
        describe: String,
    },
    #[cfg(feature = "hosting")]
    /// Print a draft the AI writes for a conversation on a code host: a
    /// comment, an answer in a thread (`--reply-to`), a comment on a line of
    /// code (`--path`), a review summary (`--review-summary`), or the
    /// description of a new issue (`--issue-title`). The conversation is read
    /// from `--conversation` (the JSON `hosting issue comments --json` or
    /// `hosting review conversation --json` prints, so
    /// `hosting issue comments 7 --json | ai hosting-reply --conversation -`
    /// works) or fetched by `--item`; the AI sees the item, the thread, the recent
    /// comments within `ai.limits.hosting_reply_context_bytes` and your
    /// `--hint`. **It only drafts: the answer is `{ draft, posted: false, ... }`
    /// and nothing is posted, edited or deleted**; posting is a separate
    /// command you run yourself. The base prompt is
    /// `ai.hosting_reply.base_prompt` (edit it in Settings > AI). Needs
    /// `ai.enabled = true`.
    HostingReply {
        /// The conversation as JSON, as a `hosting` conversation command
        /// prints it: a file, `-` for standard input, or the JSON itself. Not
        /// for `--issue-title`.
        #[arg(
            long,
            value_name = "FILE|-|JSON",
            required_unless_present_any = ["issue_title", "item"],
            conflicts_with_all = ["issue_title", "item"]
        )]
        conversation: Option<String>,
        /// The issue or pull request to answer, read from the host itself in
        /// the repository's linked source (`7` or `#7` an issue, `!7` a pull
        /// request): the same conversation `hosting issue comments 7 --json`
        /// prints, without the system events. Instead of `--conversation`.
        #[arg(long, value_name = "REF", conflicts_with = "issue_title")]
        item: Option<String>,
        /// Answer in the thread with this id (a code thread, a comment or a
        /// discussion).
        #[arg(
            long,
            value_name = "ID",
            conflicts_with_all = ["path", "review_summary", "issue_title"]
        )]
        reply_to: Option<String>,
        /// Draft a new review comment on this file of the pull request.
        #[arg(long, conflicts_with_all = ["review_summary", "issue_title"])]
        path: Option<String>,
        /// With `--path`: the line.
        #[arg(long, requires = "path")]
        line: Option<u32>,
        /// Draft the summary of the review being submitted.
        #[arg(long, conflicts_with = "issue_title")]
        review_summary: bool,
        /// Draft the description of a new issue with this title (no
        /// conversation).
        #[arg(long, value_name = "TITLE")]
        issue_title: Option<String>,
        /// What you want in this answer ("say I will look at it tomorrow").
        #[arg(long, default_value = "")]
        hint: String,
        /// Your own text to improve instead of writing a new one.
        #[arg(long)]
        draft: Option<String>,
        /// The markup the host takes: `gfm`, `gitlab_md`, `gitea_md`,
        /// `common_mark`, `azure_md`, `bitbucket_md`, `html`, `jira_wiki`,
        /// `textile`, `youtrack_md`, `trello_md`, `plain_text`,
        /// `obsidian_md`. Default: the item's own.
        #[arg(long)]
        flavor: Option<String>,
        /// Let the AI read the repository for this draft (off unless given or
        /// `ai.hosting_reply.read_repository` is true). A local agent reads it
        /// itself with read-only tools in the repository folder; an API profile
        /// gets a context GitPoulpi builds (the files the conversation names,
        /// search hits, the map) within `ai.limits.hosting_reply_*`. The AI
        /// only drafts either way; nothing is posted.
        #[arg(long)]
        read_repo: bool,
        /// Continue the exchange of the last `hosting-reply` for this same
        /// target (same `--item`, `--reply-to`, `--path`/`--line` or
        /// `--issue-title`): the earlier hints and drafts of it go back to the
        /// model as earlier turns, so "make it shorter" works. Not needed for
        /// a local Claude agent whose session is kept (it remembers). Without
        /// the flag a new exchange starts. Kept in the state folder.
        #[arg(long = "continue")]
        continue_exchange: bool,
        /// Forget the context and the agent session kept from earlier drafts
        /// of this repository and read it again (with `--read-repo`).
        #[arg(long, requires = "read_repo")]
        refresh_context: bool,
        /// With `--read-repo`: the base revision of the pull request, to put
        /// its diff (against the merge base) in the context.
        #[arg(long, requires = "read_repo", value_name = "REV")]
        pr_base: Option<String>,
        /// With `--read-repo`: the head revision of the pull request.
        #[arg(long, requires = "pr_base", value_name = "REV")]
        pr_head: Option<String>,
    },
    /// Print the system prompt an AI action sends for this repository, level
    /// by level: the built-in base, the global instructions, the workspace's
    /// (the one in `GITPOULPI_WORKSPACE` if it holds the repository, else the
    /// first that does), the repository's, then the action's own. Calls no
    /// model and works with AI turned off.
    Prompt {
        /// The AI action (`commit_message`, `code_review`, ...).
        #[arg(long)]
        action: String,
    },
    /// Binds a profile's API key to the address it was stored for: shows
    /// what is trusted, or accepts the profile's current address (after
    /// its base URL changed), a self-hosted server's certificate, or plain
    /// http. Only a person runs this (it is not an agent tool).
    Trust {
        /// The AI profile id.
        profile: String,
        /// Accept the profile's current base URL (spelled exactly as the
        /// refusal printed it).
        #[arg(long, value_name = "URL")]
        accept_origin: Option<String>,
        /// Accept the certificate with this key fingerprint (SHA-256, hex).
        #[arg(long, value_name = "SHA256")]
        accept_certificate: Option<String>,
        /// Allow a plain `http://` base URL that is not on this machine.
        #[arg(long, value_enum)]
        allow_insecure_http: Option<OnOff>,
    },
}

#[derive(Debug, Subcommand)]
pub enum StashAction {
    Push {
        #[arg(long, conflicts_with = "ai")]
        message: Option<String>,
        /// Let the AI name the stash from the working changes (needs
        /// `ai.enabled = true`, see `docs/user/ai.md`).
        #[arg(long)]
        ai: bool,
        /// With `--ai`: send the whole diff even when it is over
        /// `ai.limits.diff_bytes`. Without it a larger diff is truncated and
        /// the answer says so (`truncated`).
        #[arg(long, alias = "no-truncate")]
        send_all: bool,
    },
    Pop {
        id: Option<String>,
    },
    Apply {
        id: Option<String>,
    },
    Drop {
        id: Option<String>,
    },
    /// Save the changes one commit made (even a lost one) as a new stash,
    /// without touching the working tree. `undo` drops it again.
    FromCommit {
        /// The commit whose changes to keep (any revision: a sha, `HEAD~2`, ...).
        commit: String,
        /// The stash message; omitted, one naming the commit.
        #[arg(short = 'm', long)]
        message: Option<String>,
    },
    List,
}

#[derive(Debug, Subcommand)]
pub enum AutoStashAction {
    /// Start over: discard the working-tree changes on the stash's files
    /// (a copy is kept under `refs/gitpoulpi-backup/`), undo the operation
    /// that auto-stashed (pull, checkout, merge, rebase), then pop the
    /// stash on the original commit. Stops with the stash intact if any
    /// step fails. Exit code 1 when refused, 3 when it needs `--yes`.
    StartOver {
        /// Show what would happen and change nothing.
        #[arg(long)]
        dry_run: bool,
        /// Go ahead; without it nothing changes and the exit code is 3.
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum TagAction {
    Create {
        name: String,
        at: Option<String>,
        /// Make an *annotated* tag with this message instead of a
        /// lightweight one - `git tag -a -m`.
        #[arg(short, long)]
        message: Option<String>,
    },
    /// Delete a tag. By default only the local one (`--local`): when a
    /// remote has the tag too, the next `fetch` brings it back, so the
    /// answer says so. `--remote <R>` deletes it on that remote only
    /// (outward-facing: other people lose it), `--both` deletes it here
    /// and on the remote that has it (`--both --remote <R>` names one).
    Delete {
        name: String,
        /// Delete only the local tag (the default).
        #[arg(long, conflicts_with_all = ["remote", "both"])]
        local: bool,
        /// Delete on this remote (alone, unless `--both`).
        #[arg(long)]
        remote: Option<String>,
        /// Delete here and on the remote that has the tag.
        #[arg(long)]
        both: bool,
    },
    /// Fetch one tag from a remote into the local tags - what a
    /// `remote`-only tag in `tag list` needs. Never overwrites a local
    /// tag at another commit.
    Fetch {
        name: String,
        /// Omitted: the one remote that has the tag.
        #[arg(long)]
        remote: Option<String>,
    },
    /// Replace an existing tag with an annotated one, in place. The tag
    /// keeps the commit it already points at.
    Annotate {
        name: String,
        #[arg(short, long)]
        message: String,
    },
    /// Use the remote's version of a tag that `differs` (here and on a
    /// remote at another commit): the local tag moves to what the remote
    /// has, as known from the last fetch or push (no network). An annotated
    /// tag becomes that same tag object. Undoable.
    TakeRemote {
        name: String,
        /// Omitted: the one remote the tag differs on.
        #[arg(long)]
        remote: Option<String>,
    },
    /// Force-push the local tag to a remote that has it at another commit
    /// (a tag that `differs`), leased on the value the last fetch saw: a
    /// remote that moved since is refused ("fetch first") and nothing is
    /// overwritten. Outward-facing: without `--yes` nothing changes and the
    /// exit code is 3. Undoable (the old value is pushed back, with a lease).
    ForcePush {
        name: String,
        /// Omitted: the one remote the tag differs on.
        #[arg(long)]
        remote: Option<String>,
        /// Go ahead; without it nothing changes and the exit code is 3.
        #[arg(long)]
        yes: bool,
    },
    /// Send tags to a remote. With no names, sends every tag no remote
    /// is known to have - one push, not one per tag.
    Push {
        /// Tags to send. Omit to push every unpushed tag.
        names: Vec<String>,
        #[arg(long, default_value = "origin")]
        remote: String,
    },
    /// Delete a tag from a remote, keeping the local one.
    DeleteRemote {
        name: String,
        #[arg(long, default_value = "origin")]
        remote: String,
    },
    /// Every tag with where it lives: `local` (here), `remote` (the
    /// remotes that have it), `differs_on` (remotes holding it at another
    /// commit) and `where`: `local`, `remote` (not fetched or deleted
    /// here), `both` or `differs`. Learned from the last fetch or push,
    /// not from the network.
    List {
        /// Only the tags no remote is known to have.
        #[arg(long)]
        unpushed: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum RemoteAction {
    Add {
        name: String,
        url: String,
        /// A separate URL to push to.
        #[arg(long)]
        push_url: Option<String>,
    },
    Remove {
        name: String,
    },
    List,
    Rename {
        old_name: String,
        new_name: String,
    },
    /// Set the fetch URL, or with `--push` the separate push URL.
    SetUrl {
        name: String,
        url: String,
        #[arg(long)]
        push: bool,
    },
    /// Remove the separate push URL, so pushes use the fetch URL again.
    ClearPushUrl {
        name: String,
    },
    /// Show the repository's default remote, or set it with a name.
    Default {
        name: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum RepoConfigAction {
    /// Show the repository-local value of one key (`core.autocrlf` or
    /// `core.eol`), or of both; null means unset (the global value applies).
    Show {
        /// `core.autocrlf` or `core.eol`; both when omitted.
        key: Option<String>,
    },
    /// Set a key: `core.autocrlf` takes true | input | false, `core.eol`
    /// takes lf | crlf | native.
    Set { key: String, value: String },
    /// Remove the repository-local value, so the global one applies again.
    Unset { key: String },
}

#[derive(Debug, Subcommand)]
pub enum SparseAction {
    /// Whether the working tree is a sparse checkout and which directories
    /// it shows: `{ "enabled", "cone", "dirs" }`.
    Status,
    /// Show exactly these directories (plus the files at the repository
    /// root); every other directory leaves the working tree. Turns sparse
    /// checkout on when it is off. History is untouched.
    Set {
        /// Directories relative to the repository root, e.g. `src docs/guide`.
        #[arg(required = true, num_args = 1..)]
        dirs: Vec<String>,
    },
    /// Show these directories too, keeping the ones already shown.
    Add {
        #[arg(required = true, num_args = 1..)]
        dirs: Vec<String>,
    },
    /// Check every file out again (a full checkout).
    Disable,
}

#[derive(Debug, Subcommand)]
pub enum SshCommandAction {
    /// Show the repository's pinned `core.sshCommand` (null when unset).
    Show,
    /// Pin the repository to a private key, or to a literal command.
    Set {
        /// Private key file: pins `ssh -i "<key>" -o IdentitiesOnly=yes`.
        #[arg(long, conflicts_with = "command", required_unless_present = "command")]
        key: Option<String>,
        /// A literal `core.sshCommand` value.
        #[arg(long)]
        command: Option<String>,
    },
    /// Remove the pin, so the repository uses the ambient ssh setup again.
    Clear,
}

/// `pull --all-branches --worktrees`.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum PullWorktreesArg {
    Skip,
    Update,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum PullModeArg {
    Merge,
    FfOnly,
    Rebase,
}

#[derive(Debug, Subcommand)]
pub enum RefsAction {
    /// Hide refs from the commit graph (the TUI's "Hide"): the per-repo
    /// display filter in `repo-state.toml`, which an open TUI follows live.
    /// Names are spelled as the refs listing spells them (`feature/x`,
    /// `origin/main`, a tag name).
    ///
    /// `--remote <name>...` hides a whole remote (every branch of it, the
    /// ones fetched later too): the way to look at one history without the
    /// history of a separate archive remote.
    Hide {
        /// Refs to hide.
        #[arg(required_unless_present_any = ["clear", "remote"])]
        names: Vec<String>,
        /// Remotes to hide as a whole (names as `remote list` spells them).
        #[arg(long, num_args = 1..)]
        remote: Vec<String>,
        /// Un-hide every hidden ref and remote (in addition to any named).
        #[arg(long)]
        clear: bool,
    },
    /// Show refs again: removes them from the hidden and solo sets.
    ///
    /// `--remote <name>...` shows a whole remote again, with every branch
    /// of it that was hidden one by one.
    Show {
        /// Refs to show again.
        #[arg(required_unless_present_any = ["clear", "remote"])]
        names: Vec<String>,
        /// Remotes to show again.
        #[arg(long, num_args = 1..)]
        remote: Vec<String>,
        /// Reset the whole filter: nothing hidden (remotes included),
        /// nothing soloed.
        #[arg(long)]
        clear: bool,
    },
    /// Show only these refs (and what they reach) in the commit graph.
    Solo {
        /// Refs to solo.
        #[arg(required_unless_present = "clear")]
        names: Vec<String>,
        /// Un-solo every soloed ref (in addition to any named).
        #[arg(long)]
        clear: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum RepoAction {
    /// Is this path in a git repository, and where is its root?
    /// Goes through `GitBackend::discover`, so it answers exactly what
    /// the TUI's open-repository flow would.
    Discover {
        /// Defaults to the current directory.
        path: Option<String>,
    },
    /// Creates a repository at `path` and opens it as a tab of this
    /// session. Goes through `GitBackend::init`, so it produces exactly
    /// the repository the TUI's Create form would.
    Init {
        path: String,
        /// The branch the first commit lands on. Defaults to
        /// `general.default_branch_name`; empty leaves git's own.
        #[arg(long)]
        initial_branch: Option<String>,
        /// A repository with no working tree.
        #[arg(long)]
        bare: bool,
    },
    /// Opens a repository as a tab of this session, and records it in
    /// the recent list. Prints what was opened; it does not launch a UI.
    Open { path: String },
    /// Closes a repository's tab in this session. The repository itself
    /// is untouched.
    Close { path: String },
    /// The tabs open in this session, active one flagged.
    List,
    /// The recently-opened repositories, newest first.
    Recent {
        /// Drop entries whose directory is gone from disk.
        #[arg(long)]
        prune: bool,
        /// Forget one entry.
        #[arg(long)]
        forget: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    Get { key: String },
    Set { key: String, value: String },
    Path,
}

#[derive(Debug, Subcommand)]
pub enum ThemeAction {
    /// Lists every theme: the built-in presets and the files of the
    /// config directory's `themes/` folder (slug, label, source, light,
    /// lane palette). `ui.theme` takes any of these slugs.
    List,
    /// Prints a complete theme file for a theme on stdout, to start a
    /// user theme from (save it as `<config>/themes/<name>.toml`).
    Export {
        /// The theme's slug (see `theme list`).
        slug: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum DesktopAction {
    /// Lists, creates or removes the launcher, Start menu and Desktop
    /// shortcuts.
    Shortcut {
        #[command(subcommand)]
        action: ShortcutAction,
    },
    /// The "Open in GitPoulpi" entry of the file manager's right-click menu:
    /// File Explorer (Windows), GNOME Files, Dolphin, Nemo, Thunar, Caja and
    /// PCManFM (Linux), Finder (macOS).
    FolderMenu {
        #[command(subcommand)]
        action: FolderMenuAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum FolderMenuAction {
    /// Shows each file manager of this system: whether it is found and
    /// whether the entry is not_installed, installed, outdated or foreign (a
    /// file that is not GitPoulpi's).
    Status,
    /// Adds the entry to every file manager found here (a foreign file is
    /// never overwritten).
    Install,
    /// Removes exactly what `install` wrote, from every file manager.
    Remove,
}

#[derive(Debug, Subcommand)]
pub enum ShortcutAction {
    /// Shows each shortcut this system offers: not_installed, installed,
    /// outdated or foreign (a file that is not GitPoulpi's), and the
    /// terminal in use.
    Status,
    /// Creates or updates a shortcut (default: the OS launcher).
    Install {
        /// `launcher`, `desktop` or `startmenu`.
        #[arg(long = "where")]
        place: Option<String>,
    },
    /// Removes the shortcuts GitPoulpi made (default: all); a file that is
    /// not GitPoulpi's is never touched.
    Remove {
        /// `launcher`, `desktop` or `startmenu`.
        #[arg(long = "where")]
        place: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum AliasAction {
    /// Shows the command aliases GitPoulpi made (and what each runs) and the
    /// names in `ui.command_alias` and `ui.command_alias_add`.
    Status,
    /// Makes `<name>` (for example `gp`) start GitPoulpi: a link on Linux and
    /// macOS, a `.cmd` file on Windows, in a per-user bin folder. A file that
    /// is not GitPoulpi's is never replaced. Put the folder on PATH with
    /// `path add`.
    Create {
        /// The command name: letters, digits, `-`, `_` and `.` only.
        name: String,
        /// Make the add variant (for example `gpa`): it runs `gitpoulpi
        /// --add`, which opens the current folder as a new tab of the
        /// GitPoulpi that is already running.
        #[arg(long)]
        add: bool,
    },
    /// Removes the command aliases GitPoulpi made (all, or only `<name>`);
    /// a file that is not GitPoulpi's is never touched.
    Remove {
        /// The alias to remove (default: every alias GitPoulpi made).
        name: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum InstallationAction {
    /// Shows which GitPoulpi installation this is (its id, kind, folder and
    /// environment), whether it is the primary, its own settings and the
    /// other installations of this machine.
    Show,
    /// Sets one of this installation's own settings: `name`,
    /// `name_with_version`, `logo_color`, `command_alias`,
    /// `command_alias_add`, `folder_menu_submenu`, `env` or `mcp_bind`. An
    /// empty value clears it. The primary's colour and aliases are also
    /// written to the shared `ui.*` keys that older builds read.
    Set {
        /// The field to set.
        field: String,
        /// The new value (empty clears it).
        value: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum PathAction {
    /// Shows whether the folder GitPoulpi lives in and the alias folder are on
    /// the user PATH, and what GitPoulpi added.
    Status,
    /// Puts the folder GitPoulpi lives in and the alias folder on the user
    /// PATH for new terminals: a marked block
    /// in the shell startup files (Linux, macOS) or the user PATH of the
    /// registry (Windows). Folders already on PATH are left alone.
    Add,
    /// Takes out exactly what `path add` put there.
    Remove,
}

#[derive(Debug, Subcommand)]
pub enum SessionAction {
    List,
    New { name: String },
    Delete { name: String },
    Rename { old_name: String, new_name: String },
}

#[derive(Debug, Subcommand)]
pub enum DaemonAction {
    /// Runs this session's daemon in the foreground (blocks; Ctrl-C to
    /// stop, or a `daemon stop`/`daemon restart` sent from elsewhere).
    /// This is what `daemon start` spawns detached, and what a dedicated
    /// terminal can also run directly.
    Run,
    /// Starts this session's daemon as a detached background process (or
    /// reports it as already running). `--backend` pins the backend the
    /// spawned process is told to use; omitted, the spawned process
    /// resolves `general.default_backend` itself.
    Start,
    /// Asks this session's daemon to shut down gracefully, falling back to
    /// killing its recorded PID only if it doesn't answer in time and only
    /// if it is a standalone daemon (never one embedded in a running TUI).
    Stop {
        /// Stop every session's daemon that `session list` reports a port
        /// file for, not just this one.
        #[arg(long)]
        all: bool,
    },
    /// `daemon stop` followed by `daemon start`.
    Restart,
    /// Whether this session's daemon is listening, and if so its port,
    /// PID, uptime, backend and build version.
    Status,
}

#[derive(Debug, Subcommand)]
pub enum McpAction {
    Serve {
        #[arg(long)]
        stdio: bool,
        /// Listen address for streamable HTTP, e.g. 127.0.0.1:8808
        /// (endpoint `/mcp`); without an address, `[mcp] bind` (default
        /// 127.0.0.1:8808). Requests need the bearer token (`mcp token`)
        /// unless `[mcp] http_auth` is off. Loopback only unless
        /// `--allow-remote`.
        #[arg(long, value_name = "ADDR", num_args = 0..=1, default_missing_value = "")]
        http: Option<String>,
        /// Address for `--http` (same as `--http ADDR`; wins over it).
        #[arg(long, requires = "http")]
        bind: Option<String>,
        /// Allow `--http` on a non-loopback address. Refused unless
        /// `[mcp] http_auth` is on with a token in place and the traffic is
        /// encrypted (`[mcp] tls`, or `behind_proxy` for a tunnel or reverse
        /// proxy you vouch for); `ssh -L` to a loopback endpoint is safer.
        #[arg(long, requires = "http")]
        allow_remote: bool,
        /// How far the agent may move the user's window with the `ui_*`
        /// tools: `ask` (default: the window shows a chip the user
        /// confirms), `auto` or `off`. Overrides `[mcp] ui_control`.
        #[arg(long, value_parser = ["auto", "ask", "off"])]
        ui_control: Option<String>,
    },
    /// The bearer token of the HTTP endpoint (`mcp serve --http`): shows it
    /// masked as JSON `{token_hint, path, rotated}`, creating it on first
    /// use. A client sends it as `Authorization: Bearer <token>`. Not an MCP
    /// tool: an agent never reads or rotates its own key
    Token {
        /// Print the full token too, for you to paste into a client's
        /// configuration
        #[arg(long)]
        show: bool,
        /// Copy the full token to the clipboard through the terminal
        /// (OSC 52), without printing it
        #[arg(long)]
        copy: bool,
        /// Replace the token with a new one; clients holding the old one are
        /// refused at once
        #[arg(long)]
        rotate: bool,
    },
}

/// How `reflog-restore` treats a checked-out branch with uncommitted changes.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ReflogModeArg {
    /// Move the branch and keep the local changes (refuses if one is in the way).
    Keep,
    Soft,
    Mixed,
    Hard,
}

impl ReflogModeArg {
    pub fn as_str(self) -> &'static str {
        match self {
            ReflogModeArg::Keep => "keep",
            ReflogModeArg::Soft => "soft",
            ReflogModeArg::Mixed => "mixed",
            ReflogModeArg::Hard => "hard",
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ResetModeArg {
    Soft,
    Mixed,
    Hard,
}
