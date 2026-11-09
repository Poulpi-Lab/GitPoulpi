use clap::Subcommand;

use super::{OnOff, RunInRepoArg};

#[derive(Debug, Subcommand)]
pub enum HostingAction {
    /// The configured integrations, without their tokens.
    List,
    /// Every project the integration's token can see.
    Projects {
        /// Which integration (`[[integrations]] id`). Optional when only
        /// one is configured.
        #[arg(long)]
        id: Option<String>,
        /// Case-insensitive substring over the project path.
        #[arg(long)]
        filter: Option<String>,
    },
    /// Clones a project from `projects` by its full path, resolving the
    /// clone URL through the host rather than making the user paste one.
    Clone {
        /// The project's full path, as `hosting projects` prints it.
        project: String,
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        dest: Option<String>,
        /// Clone over HTTPS rather than SSH.
        #[arg(long)]
        https: bool,
    },
    /// Generates an ed25519 key pair and uploads the public half.
    SshKey {
        #[command(subcommand)]
        action: SshKeyAction,
    },
    /// Manages the configured integration profiles (`[[integrations]]`).
    Integrations {
        #[command(subcommand)]
        action: IntegrationsAction,
    },
    /// Validates a token and stores it under the profile's `token_ref`,
    /// caching who it belongs to. The token is read from stdin
    /// (`--token-stdin`), never from an argument, so it stays out of shell
    /// history. Not exposed over MCP: a secret must not pass through an
    /// agent's tool-call transcript.
    Login {
        /// Which integration. Optional when only one is configured.
        #[arg(long)]
        id: Option<String>,
        /// Read the token from stdin (required; one line, trailing newline
        /// ignored).
        #[arg(long)]
        token_stdin: bool,
    },
    /// Deletes the profile's stored token and forgets the connected user;
    /// the profile itself stays.
    Logout {
        #[arg(long)]
        id: Option<String>,
    },
    /// Pull/merge requests of the current repository's project.
    Prs {
        /// `open` (default), `closed` (includes merged) or `all`.
        #[arg(long, value_enum, default_value_t = StateArg::Open)]
        state: StateArg,
        /// Only those you opened.
        #[arg(long)]
        mine: bool,
        /// Only those awaiting your review.
        #[arg(long)]
        review_requested: bool,
        /// Which integration. Without it the repository's linked pull/merge
        /// request source is used (see `hosting link`); an unlinked
        /// repository fetches nothing. With it, the project this
        /// repository's remote names on that profile.
        #[arg(long)]
        id: Option<String>,
    },
    /// Checks out a pull/merge request, including one from a fork: fetches
    /// its head (`refs/pull/<n>/head`, `refs/merge-requests/<n>/head`) into
    /// the local branch `pr/<n>` (`mr/<n>` on GitLab), tracking it, and
    /// switches to it.
    CheckoutPr {
        /// The PR number (GitHub) or MR `iid` (GitLab).
        number: u64,
        /// The remote the request lives on. Defaults to the one the
        /// project was resolved from, else the repository's default.
        #[arg(long)]
        remote: Option<String>,
        /// Which integration (see `hosting prs`).
        #[arg(long)]
        id: Option<String>,
        /// Only fetch and create/update the branch; stay where you are.
        #[arg(long)]
        no_checkout: bool,
    },
    /// The pull/merge request that best represents a branch (open, then
    /// draft, then merged, then closed; newest first).
    PrForBranch {
        /// Defaults to the checked-out branch.
        branch: Option<String>,
        #[arg(long)]
        id: Option<String>,
    },
    /// Issues of the current repository's project.
    Issues {
        #[arg(long, value_enum, default_value_t = StateArg::Open)]
        state: StateArg,
        /// Only those assigned to you.
        #[arg(long)]
        assigned: bool,
        #[arg(long)]
        id: Option<String>,
    },
    #[cfg(feature = "board")]
    /// The issue board of the current repository's linked project: its
    /// columns with their cards, the same columns as the TUI board (a
    /// status, a list or a label per column). Without a subcommand, the
    /// same as `board show`.
    Board {
        #[command(subcommand)]
        action: Option<BoardAction>,
        /// Which integration (see `hosting issues`).
        #[arg(long)]
        id: Option<String>,
    },
    /// Creates issues on the current repository's linked project.
    Issue {
        #[command(subcommand)]
        action: IssueAction,
    },
    /// Prints the web URL of a pull request or issue of the current
    /// repository's project. Launches the system browser only with
    /// `--browser`.
    Open {
        #[arg(value_enum)]
        kind: OpenKind,
        /// A number (`12`, `#12`, `!12`), or an issue's own reference
        /// (`AB#12`, `ABC-12`, a Trello short link).
        reference: String,
        #[arg(long)]
        id: Option<String>,
        /// Also open the URL in the system browser.
        #[arg(long)]
        browser: bool,
    },
    /// The issue and pull request references a commit message or a branch
    /// name holds, read through this repository's links (an unlinked
    /// repository has none): `[{text, kind, reference, url}]`.
    Refs {
        /// Scan this text.
        #[arg(long, conflicts_with_all = ["commit", "branch"])]
        message: Option<String>,
        /// Scan this commit's message.
        #[arg(long, conflicts_with = "branch")]
        commit: Option<String>,
        /// Read the issue a branch name names (`ABC-123-fix`, `123-fix`).
        #[arg(long)]
        branch: Option<String>,
    },
    /// Prints the URL where a pull/merge request is edited (GitLab's
    /// `.../edit` view; GitHub edits in place on the request's page).
    /// With a number, that request; otherwise the one for `--branch`
    /// (default: the checked-out branch). Follows the repository's
    /// pull/merge request link like `prs`. Launches the browser only
    /// with `--open`.
    EditUrl {
        /// The request's number (`53` for `!53` / `#53`).
        #[arg(conflicts_with = "branch")]
        number: Option<u64>,
        /// Find the request by its source branch instead.
        #[arg(long)]
        branch: Option<String>,
        #[arg(long)]
        id: Option<String>,
        /// Also open the URL in the system browser.
        #[arg(long)]
        open: bool,
    },
    /// Links this repository's pull/merge request list or issue list to a
    /// provider profile and one of its repositories, or unlinks it (with
    /// no flag, shows the link). Nothing is fetched by `prs` / `issues`
    /// until a source is linked; the same link the TUI's sidebar sections
    /// save (`pulls_source` / `issues_source` in `.gitpoulpi/repo-state.toml`).
    Link {
        /// Which list.
        #[arg(value_enum)]
        kind: LinkKind,
        /// The profile to link. With no `--project`, the repository its
        /// remotes name on that profile's host is used.
        #[arg(long)]
        id: Option<String>,
        /// The provider's repository (`owner/name`, `group/sub/proj`).
        #[arg(long, requires = "id")]
        project: Option<String>,
        /// Narrows the issue list with the provider's own syntax (a Jira
        /// JQL clause, an Azure WIQL condition, Trello list names).
        #[arg(long, requires = "id", conflicts_with = "clear")]
        filter: Option<String>,
        /// Unlinks the list: nothing is fetched until it is linked again.
        #[arg(long, conflicts_with_all = ["id", "project"])]
        clear: bool,
    },
    /// Pins which integration profile answers for the current repository
    /// (or, with neither flag, shows what is in force).
    Use {
        /// The profile to pin.
        #[arg(long)]
        id: Option<String>,
        /// Removes the pin, going back to resolving by remote host.
        #[arg(long, conflicts_with = "id")]
        clear: bool,
    },
    /// Opens a pull request (GitHub, Gitea, Forgejo, Bitbucket, Azure DevOps) or merge request
    /// (GitLab) from a branch of the current repository.
    Pr {
        /// The branch to open it from. Defaults to the checked-out
        /// branch.
        #[arg(long)]
        source: Option<String>,
        /// The branch to open it against. Defaults to the remote's
        /// default branch when it can be told, else `main`/`master`
        /// when one of those exists.
        #[arg(long)]
        target: Option<String>,
        /// Defaults to the source branch's tip commit's subject line.
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        draft: bool,
        /// Which integration (`[[integrations]] id`). Optional when only
        /// one is configured.
        #[arg(long)]
        id: Option<String>,
        /// Print the compare/new-request URL instead of calling the
        /// host's API. Also what happens automatically with no
        /// integration configured for this remote, or no token for it.
        #[arg(long)]
        web: bool,
        /// Someone to assign (GitHub, GitLab, Gitea, Forgejo); repeat the flag or
        /// comma-separate. See `hosting members`.
        #[arg(long = "assignee", value_delimiter = ',')]
        assignees: Vec<String>,
        /// Someone to ask for a review (GitHub, GitLab, Gitea, Forgejo); repeat the flag or
        /// comma-separate. See `hosting members`.
        #[arg(long = "reviewer", value_delimiter = ',')]
        reviewers: Vec<String>,
        /// A label to apply (GitHub, GitLab, Gitea, Forgejo); repeat the flag or
        /// comma-separate. See `hosting labels`.
        #[arg(long = "label", value_delimiter = ',')]
        labels: Vec<String>,
        /// The milestone, by title (GitHub, GitLab, Gitea, Forgejo). See `hosting
        /// milestones`.
        #[arg(long)]
        milestone: Option<String>,
    },
    /// Changes who a pull/merge request is assigned to and who reviews
    /// it, its labels and its milestone (GitHub, GitLab, Gitea, Forgejo). Outward action:
    /// it writes to the host. Prints the metadata as it is afterwards.
    PrEdit {
        /// The PR number (GitHub) or MR `iid` (GitLab).
        number: u64,
        #[command(flatten)]
        meta: IssueMetaArgs,
        /// Someone to ask for a review; repeat the flag or comma-separate.
        #[arg(long, value_delimiter = ',')]
        add_reviewer: Vec<String>,
        /// Someone whose review request to withdraw; repeat the flag or
        /// comma-separate.
        #[arg(long, value_delimiter = ',')]
        remove_reviewer: Vec<String>,
        /// Which integration (see `hosting prs`).
        #[arg(long)]
        id: Option<String>,
    },
    /// The people who can be assigned or asked to review in the project
    /// (GitHub, GitLab, Gitea, Forgejo). Read-only.
    Members {
        /// Narrows the list to logins (and, on GitLab, names) holding this
        /// text.
        #[arg(long)]
        query: Option<String>,
        /// Which list's project: the pull/merge requests' (default) or
        /// the issues'.
        #[arg(long, value_enum, default_value_t = LinkKind::Pulls)]
        source: LinkKind,
        /// Which integration.
        #[arg(long)]
        id: Option<String>,
    },
    /// The labels of the project (GitHub, GitLab, Gitea, Forgejo). Read-only.
    Labels {
        /// Narrows the list to names holding this text.
        #[arg(long)]
        query: Option<String>,
        /// Which list's project: the pull/merge requests' (default) or
        /// the issues'.
        #[arg(long, value_enum, default_value_t = LinkKind::Pulls)]
        source: LinkKind,
        /// Which integration.
        #[arg(long)]
        id: Option<String>,
    },
    /// The open milestones of the project (GitHub, GitLab, Gitea, Forgejo). Read-only.
    Milestones {
        /// Narrows the list to titles holding this text.
        #[arg(long)]
        query: Option<String>,
        /// Which list's project: the pull/merge requests' (default) or
        /// the issues'.
        #[arg(long, value_enum, default_value_t = LinkKind::Pulls)]
        source: LinkKind,
        /// Which integration.
        #[arg(long)]
        id: Option<String>,
    },
    /// The account-wide lists for the Radar: the requests waiting for your
    /// review, every open request of your namespaces and the issues assigned
    /// to you, across all repositories of each profile (GitHub, GitLab,
    /// Gitea, Forgejo, Azure DevOps, Bitbucket Data Center, Gerrit), cloned
    /// or not. Without `--what`, each profile's
    /// `[integrations.radar]` settings decide (see `hosting integrations
    /// radar`); the answer is reused for five minutes unless `--refresh`.
    /// Read-only.
    Radar {
        /// Which profile; repeat the flag for several. Without it, every
        /// enabled profile of a supported kind.
        #[arg(long = "id")]
        ids: Vec<String>,
        /// What to list, whatever the profile's settings say: `review`
        /// (waiting for your review), `open` (every open request), `issues`
        /// (assigned to you) or `all`.
        #[arg(long, value_enum)]
        what: Option<RadarWhat>,
        /// Keep draft requests too.
        #[arg(long)]
        include_drafts: bool,
        /// Ask the host even when a recent answer is cached.
        #[arg(long)]
        refresh: bool,
    },
    /// Reviews a pull/merge request in place (GitHub, GitLab, Gitea, Forgejo): read it, its
    /// threads and its files, comment, reply, resolve, submit a review, and
    /// fetch its head for the diff. Same commands for a person, a script and
    /// an AI agent.
    Review {
        #[command(subcommand)]
        action: ReviewAction,
    },
    /// Reads one file of a project at a revision from the host's API, with
    /// no clone: text as is, other bytes as base64 (`encoding`). Exit 1 and
    /// an error when the file does not exist at that revision.
    File(FileArgs),
}

/// The arguments of `hosting file` (their own struct: a flat variant per
/// command overflows the debug-build stack of clap's derive).
#[derive(Debug, clap::Args)]
pub struct FileArgs {
    /// The path inside the project, like `CLAUDE.md` or `src/main.rs`.
    pub path: String,
    /// The project: `owner/name` on GitHub, `group/sub/project` on GitLab.
    #[arg(long)]
    pub project: String,
    /// The revision: a branch, tag or commit. Defaults to the project's
    /// default branch.
    #[arg(long = "ref")]
    pub rev: Option<String>,
    /// Which integration (see `hosting list`); optional when only one is configured.
    #[arg(long)]
    pub id: Option<String>,
}

/// The metadata flags `pr-edit` and `issue edit` share.
#[derive(Debug, clap::Args)]
pub struct IssueMetaArgs {
    /// Someone to assign; repeat the flag or comma-separate.
    #[arg(long, value_delimiter = ',')]
    pub add_assignee: Vec<String>,
    /// Someone to unassign; repeat the flag or comma-separate.
    #[arg(long, value_delimiter = ',')]
    pub remove_assignee: Vec<String>,
    /// A label to add; repeat the flag or comma-separate.
    #[arg(long, value_delimiter = ',')]
    pub add_label: Vec<String>,
    /// A label to remove; repeat the flag or comma-separate.
    #[arg(long, value_delimiter = ',')]
    pub remove_label: Vec<String>,
    /// Sets the milestone, by title (see `hosting milestones`).
    #[arg(long, conflicts_with = "no_milestone")]
    pub milestone: Option<String>,
    /// Clears the milestone.
    #[arg(long)]
    pub no_milestone: bool,
}

/// What `hosting review` does.
#[derive(Debug, Subcommand)]
pub enum ReviewAction {
    /// Reads one pull/merge request for review: description, head and base
    /// commits, merge state, checks, what the host lets a review do
    /// (`capabilities`) and its changed files. The diff itself comes from
    /// the repository (`review fetch`, then diff `refs/gitpoulpi/pr/<n>`).
    Show {
        /// The PR number (GitHub) or MR `iid` (GitLab), the change number (Gerrit).
        number: u64,
        /// The patch set to read (Gerrit; default: the newest).
        #[arg(long)]
        patch_set: Option<u32>,
        /// List the files as the difference from this older patch set
        /// instead of from the target branch (Gerrit).
        #[arg(long)]
        compare_with: Option<u32>,
        /// Review a project that is not the current repository (no clone
        /// needed): `owner/name` on GitHub, `group/sub/project` on GitLab.
        /// `--id` then names the integration, if several are configured.
        #[arg(long)]
        project: Option<String>,
        /// Which integration (see `hosting prs`).
        #[arg(long)]
        id: Option<String>,
    },
    /// The diff of a pull/merge request, file by file with its hunks
    /// (`files[].diff.hunks`). In a repository it fetches the request's refs
    /// and diffs them locally; with `--api`, or with `--project` (no
    /// repository), it reads the diff from the host's API instead, which needs
    /// no clone. A file whose patch the host left out (too big) has
    /// `omitted` set and no hunks.
    Diff {
        /// The PR number (GitHub) or MR `iid` (GitLab).
        number: u64,
        /// Read the diff from the host's API even inside a clone.
        #[arg(long)]
        api: bool,
        /// Only this file's diff.
        #[arg(long)]
        file: Option<String>,
        /// The remote the request lives on (local diff only). Defaults to the
        /// one the project was resolved from, else the repository's default.
        #[arg(long)]
        remote: Option<String>,
        /// Review a project that is not the current repository (no clone
        /// needed): `owner/name` on GitHub, `group/sub/project` on GitLab.
        /// `--id` then names the integration, if several are configured.
        #[arg(long)]
        project: Option<String>,
        /// Which integration (see `hosting prs`).
        #[arg(long)]
        id: Option<String>,
    },
    /// The review threads of a pull/merge request (resolved ones too, each
    /// with its path, line, side, outdated flag and comments) and its
    /// conversation comments. Lines are in the diff of `refs/gitpoulpi/pr/<n>`:
    /// `new` is the line at the head, `old` the line at the merge base.
    Threads {
        /// The PR number (GitHub) or MR `iid` (GitLab).
        number: u64,
        /// Leave out the resolved threads.
        #[arg(long)]
        unresolved: bool,
        /// The patch set to read (Gerrit; default: the newest).
        #[arg(long)]
        patch_set: Option<u32>,
        /// Review a project that is not the current repository (no clone
        /// needed): `owner/name` on GitHub, `group/sub/project` on GitLab.
        /// `--id` then names the integration, if several are configured.
        #[arg(long)]
        project: Option<String>,
        /// Which integration (see `hosting prs`).
        #[arg(long)]
        id: Option<String>,
    },
    /// Posts a comment in the conversation of a pull/merge request (not on
    /// code). Outward action: it is posted at once.
    Comment {
        /// The PR number (GitHub) or MR `iid` (GitLab).
        number: u64,
        /// The comment (markdown).
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read the comment from this file (`-` reads stdin).
        #[arg(long)]
        body_file: Option<std::path::PathBuf>,
        /// Review a project that is not the current repository (no clone
        /// needed): `owner/name` on GitHub, `group/sub/project` on GitLab.
        /// `--id` then names the integration, if several are configured.
        #[arg(long)]
        project: Option<String>,
        /// Which integration (see `hosting prs`).
        #[arg(long)]
        id: Option<String>,
    },
    /// Answers a review thread (its id from `review threads`). Outward action:
    /// it is posted at once.
    Reply {
        /// The PR number (GitHub) or MR `iid` (GitLab).
        number: u64,
        /// The thread's id, as `review threads` prints it.
        thread: String,
        /// The reply (markdown).
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read the reply from this file (`-` reads stdin).
        #[arg(long)]
        body_file: Option<std::path::PathBuf>,
        /// Review a project that is not the current repository (no clone
        /// needed): `owner/name` on GitHub, `group/sub/project` on GitLab.
        /// `--id` then names the integration, if several are configured.
        #[arg(long)]
        project: Option<String>,
        /// Which integration (see `hosting prs`).
        #[arg(long)]
        id: Option<String>,
    },
    /// Resolves a review thread, or reopens it with `--unresolve`.
    Resolve {
        /// The PR number (GitHub) or MR `iid` (GitLab).
        number: u64,
        /// The thread's id, as `review threads` prints it.
        thread: String,
        /// Reopen the thread instead.
        #[arg(long)]
        unresolve: bool,
        /// Review a project that is not the current repository (no clone
        /// needed): `owner/name` on GitHub, `group/sub/project` on GitLab.
        /// `--id` then names the integration, if several are configured.
        #[arg(long)]
        project: Option<String>,
        /// Which integration (see `hosting prs`).
        #[arg(long)]
        id: Option<String>,
    },
    /// Submits a review: every `--comment` and the verdict in one go.
    /// `request-changes` is not offered by GitLab (it has no such review state:
    /// it is refused there, with nothing sent).
    Submit {
        /// The PR number (GitHub) or MR `iid` (GitLab).
        number: u64,
        /// The verdict.
        #[arg(long, value_enum)]
        event: ReviewEventArg,
        /// The review's summary (markdown). Required for `request-changes`
        /// and `comment` unless a `--comment` carries the message.
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read the summary from this file (`-` reads stdin).
        #[arg(long)]
        body_file: Option<std::path::PathBuf>,
        /// A comment on code, `path:line[:side]:text`; repeat the flag.
        /// `line` is `N` or a range `N-M` (GitHub; GitLab keeps the last
        /// line), `side` is `new` (the line at the head, the default) or
        /// `old` (the line at the merge base).
        #[arg(long = "comment")]
        comments: Vec<String>,
        /// A vote on a label, `Label=value` (Gerrit: `Code-Review=+1`,
        /// `Verified=-1`); repeat the flag. Only the values the server
        /// permits for the change are accepted (see `review show`,
        /// `vote_labels`).
        #[arg(long = "vote")]
        votes: Vec<String>,
        /// The patch set to read (Gerrit; default: the newest).
        #[arg(long)]
        patch_set: Option<u32>,
        /// Review a project that is not the current repository (no clone
        /// needed): `owner/name` on GitHub, `group/sub/project` on GitLab.
        /// `--id` then names the integration, if several are configured.
        #[arg(long)]
        project: Option<String>,
        /// Which integration (see `hosting prs`).
        #[arg(long)]
        id: Option<String>,
    },
    /// Fetches a pull/merge request's head into `refs/gitpoulpi/pr/<n>`
    /// and its target branch into `refs/gitpoulpi/pr/<n>-base`, to review
    /// the diff between them. Neither is a branch, nothing is checked out.
    Fetch {
        /// The PR number (GitHub) or MR `iid` (GitLab).
        number: u64,
        /// The remote the request lives on. Defaults to the one the
        /// project was resolved from, else the repository's default.
        #[arg(long)]
        remote: Option<String>,
        /// The patch set to read (Gerrit; default: the newest).
        #[arg(long)]
        patch_set: Option<u32>,
        /// Also fetch this older patch set into
        /// `refs/gitpoulpi/pr/<n>-ps<k>`, to diff the two (Gerrit).
        #[arg(long)]
        compare_with: Option<u32>,
        /// Which integration (see `hosting prs`).
        #[arg(long)]
        id: Option<String>,
    },
    /// Submits (merges) a Gerrit change, when the server says it can be
    /// (`review show`: `can_submit`). Outward action on the host: not
    /// undoable from GitPoulpi.
    SubmitChange {
        /// The change number.
        number: u64,
        /// Which integration (see `hosting prs`).
        #[arg(long)]
        id: Option<String>,
    },
    /// Abandons a Gerrit change, with an optional message for its owner and
    /// reviewers. Outward action on the host: not undoable from GitPoulpi
    /// (restore the change in Gerrit).
    Abandon {
        /// The change number.
        number: u64,
        /// A message posted with the abandon.
        #[arg(long)]
        message: Option<String>,
        /// Which integration (see `hosting prs`).
        #[arg(long)]
        id: Option<String>,
    },
    /// Pushes `HEAD` to `refs/for/<branch>` to open a Gerrit change (or a
    /// new patch set of one). Outward action: it pushes. Not undoable from
    /// GitPoulpi: abandon the change in Gerrit.
    Push {
        /// The target branch on the server (`main`, `master`).
        #[arg(long)]
        branch: String,
        /// A topic to group changes under (Gerrit's `%topic=`).
        #[arg(long)]
        topic: Option<String>,
        /// The git remote of the Gerrit project. Defaults to the one the
        /// project was resolved from, else the repository's default.
        #[arg(long)]
        remote: Option<String>,
        /// Which integration (see `hosting prs`).
        #[arg(long)]
        id: Option<String>,
    },
    /// Asks the AI to review a pull/merge request and prints its suggested
    /// comments (path, line, side, severity, text) and an overall summary as
    /// JSON. Posts nothing: pass the ones you keep to `review submit
    /// --comment`. Fetches the request's refs like `review fetch`. Needs
    /// `ai.enabled = true`; the `code_review` instructions of the global,
    /// workspace and repository levels apply (see `ai prompt`).
    Suggest {
        /// The PR number (GitHub) or MR `iid` (GitLab).
        number: u64,
        /// The remote the request lives on. Defaults to the one the
        /// project was resolved from, else the repository's default.
        #[arg(long)]
        remote: Option<String>,
        /// Review a project that is not the current repository (no clone
        /// needed): `owner/name` on GitHub, `group/sub/project` on GitLab.
        /// `--id` then names the integration, if several are configured.
        #[arg(long)]
        project: Option<String>,
        /// Which integration (see `hosting prs`).
        #[arg(long)]
        id: Option<String>,
        /// Run a local agent in the repository (`yes`), not there (`no`) or
        /// as its AI profile says (`profile`), for this run. Not with
        /// `--project` (there is no checkout).
        #[arg(long, value_enum)]
        run_in_repo: Option<RunInRepoArg>,
        /// Give the model the repository's CLAUDE.md (read at the branch the
        /// request targets), for this run.
        #[arg(long, value_enum)]
        claude_md: Option<OnOff>,
        /// Give the model the repository's AGENTS.md, for this run.
        #[arg(long, value_enum)]
        agents_md: Option<OnOff>,
        /// One more file for this run: a path relative to the repository
        /// root, an absolute path, or a link to a file of a code host.
        #[arg(long, value_name = "PATH|URL", conflicts_with = "no_context_file")]
        context_file: Option<String>,
        /// No extra file for this run, whatever the defaults say.
        #[arg(long)]
        no_context_file: bool,
        /// Send every input whole, even over its `ai.limits.*` cap (the
        /// diff, a file's diff, the description, the existing comments, the
        /// context files). Without it an input over a cap is truncated and
        /// the answer says what was cut in `truncated`.
        #[arg(long, alias = "no-truncate")]
        send_all: bool,
    },
}

impl ReviewAction {
    /// The `--project` the action was given, if any: it then runs against
    /// the host alone, with no repository.
    pub fn project(&self) -> Option<&str> {
        match self {
            ReviewAction::Show { project, .. }
            | ReviewAction::Diff { project, .. }
            | ReviewAction::Threads { project, .. }
            | ReviewAction::Comment { project, .. }
            | ReviewAction::Reply { project, .. }
            | ReviewAction::Resolve { project, .. }
            | ReviewAction::Submit { project, .. }
            | ReviewAction::Suggest { project, .. } => project.as_deref(),
            ReviewAction::Fetch { .. }
            | ReviewAction::Push { .. }
            | ReviewAction::SubmitChange { .. }
            | ReviewAction::Abandon { .. } => None,
        }
    }
}

/// The verdicts `hosting review submit --event` takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ReviewEventArg {
    /// Comments only, no verdict.
    Comment,
    Approve,
    /// Not offered by GitLab.
    RequestChanges,
}

/// Which states a `hosting prs`/`hosting issues` query asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum StateArg {
    Open,
    Closed,
    All,
}

/// What `hosting radar --what` lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum RadarWhat {
    /// Requests waiting for your review.
    Review,
    /// Every open request of your namespaces.
    Open,
    /// Issues assigned to you.
    Issues,
    /// All three.
    All,
}

/// What to do with a request whose repository is not cloned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum RemoteReviewArg {
    /// Ask each time.
    Ask,
    /// Clone it, then review there.
    Clone,
    /// Review through the host's API, without a clone.
    Api,
}

/// Which per-repository list `hosting link` acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum LinkKind {
    /// Pull / merge requests (`pulls_source`).
    Pulls,
    /// Issues (`issues_source`).
    Issues,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OpenKind {
    Pr,
    Issue,
}

#[derive(Debug, Subcommand)]
pub enum IntegrationsAction {
    /// The configured profiles, without their tokens.
    List,
    /// Adds a profile of any kind that has a client.
    Add {
        /// `github`, `github_enterprise`, `gitlab`, `gitlab_self_managed`,
        /// `azure_devops`, `azure_devops_server`, `bitbucket`, `jira_cloud`,
        /// `jira_data_center`, `trello`, `gitea`, `forgejo`, `linear`, `youtrack`,
        /// `redmine`, `gerrit` or `local_markdown`.
        #[arg(long)]
        kind: String,
        /// The instance's web root. Required for the self-managed kinds
        /// and Jira.
        #[arg(long)]
        url: Option<String>,
        /// The Azure DevOps organization (`azure_devops`), a shortcut for
        /// `--url https://dev.azure.com/<org>`.
        #[arg(long)]
        org: Option<String>,
        /// The account email (`jira_cloud`, `bitbucket`), the API key
        /// (`trello`) or the user name (`gerrit`). Not a secret: the token goes in with `hosting login`.
        #[arg(long)]
        account: Option<String>,
        /// The notes folder (`local_markdown`): relative to each repository
        /// (`docs/issues`) or absolute (an Obsidian vault subfolder).
        #[arg(long)]
        path: Option<String>,
        /// Defaults to the kind's name (`gitlab`, `gitlab-2`, ...).
        #[arg(long)]
        id: Option<String>,
        /// Display name.
        #[arg(long)]
        name: Option<String>,
    },
    /// Removes a profile and deletes its stored token.
    Remove { id: String },
    /// Checks the profile's token against its host: user, scopes, expiry.
    Test {
        #[arg(long)]
        id: Option<String>,
    },
    /// Reviews the security check of a profile and accepts what it stopped:
    /// a new address, another account, a new certificate, plain http. Only a
    /// person runs this (it is not an agent tool). With no option it prints
    /// what is trusted and what is stopped.
    Trust {
        /// The profile.
        id: String,
        /// Accept the profile's current address (it must be spelled exactly
        /// as the refusal printed it).
        #[arg(long, value_name = "URL")]
        accept_origin: Option<String>,
        /// Accept this account id as the owner of the token.
        #[arg(long, value_name = "ID")]
        accept_account: Option<String>,
        /// Accept the certificate with this key fingerprint (SHA-256, hex),
        /// exactly as the refusal printed it.
        #[arg(long, value_name = "SHA256")]
        accept_certificate: Option<String>,
        /// Hold the pinned certificate strictly: the key must match even
        /// when the certificate chains to a public root.
        #[arg(long, value_enum)]
        strict_tls: Option<OnOff>,
        /// Allow a plain `http://` address that is not on this machine.
        #[arg(long, value_enum)]
        allow_insecure_http: Option<OnOff>,
    },
    /// Shows or changes a profile's Radar settings (`[integrations.radar]`):
    /// which account-wide lists `hosting radar` and the Radar fetch, the
    /// namespaces to search and what a review of a request that is not
    /// cloned does. With no option it only prints the current settings.
    Radar {
        /// The profile.
        id: String,
        /// Requests waiting for your review, every repository.
        #[arg(long, value_enum)]
        review_requests: Option<OnOff>,
        /// Every open request of your namespaces.
        #[arg(long, value_enum)]
        open_requests: Option<OnOff>,
        /// Open issues assigned to you, every repository.
        #[arg(long, value_enum)]
        assigned_issues: Option<OnOff>,
        /// Keep draft requests in the account lists.
        #[arg(long, value_enum)]
        include_drafts: Option<OnOff>,
        /// A namespace (org, group) to search for open requests; repeat the
        /// flag or comma-separate. Replaces the saved list.
        #[arg(long = "namespace", value_delimiter = ',')]
        namespaces: Vec<String>,
        /// Forget the saved namespaces: your own plus every membership.
        #[arg(long, conflicts_with = "namespaces")]
        clear_namespaces: bool,
        /// What reviewing a request that is not cloned does.
        #[arg(long, value_enum)]
        review_mode: Option<RemoteReviewArg>,
    },
}

#[cfg(feature = "board")]
#[derive(Debug, Subcommand)]
pub enum BoardAction {
    /// The board's columns, each with its cards. Read-only.
    Show {
        /// Which integration (see `hosting issues`).
        #[arg(long)]
        id: Option<String>,
    },
    /// Moves a card to a column: a status change, a workflow transition
    /// (Jira), a work item state (Azure DevOps), a list (Trello) or a label
    /// swap (GitHub, GitLab, Gitea, Forgejo). A move the provider has no
    /// path for is refused with the reason. Outward action: it writes to
    /// the host.
    Move {
        /// The card: its number or its own reference (`12`, `#12`,
        /// `ABC-12`, `AB#12`).
        reference: String,
        /// The column's name, as `hosting board` prints it.
        column: String,
        /// Which integration (see `hosting issues`).
        #[arg(long)]
        id: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum IssueAction {
    /// Opens an issue on the project the repository's ISSUES source is
    /// linked to (`hosting link issues`), or the one `--id` resolves from
    /// the remote. Outward action: it posts to the host.
    Create {
        /// The issue title.
        #[arg(long)]
        title: String,
        /// The issue description.
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read the description from this file (`-` reads stdin).
        #[arg(long)]
        body_file: Option<std::path::PathBuf>,
        /// A label to apply; repeat the flag or comma-separate.
        #[arg(long = "label", value_delimiter = ',')]
        labels: Vec<String>,
        /// The issue type, by name or id (Jira, Azure DevOps; see
        /// `hosting issue options`). Defaults to the provider's own pick.
        #[arg(long = "type")]
        issue_type: Option<String>,
        /// The list the card goes in, by name or id (Trello; see
        /// `hosting issue options`). Defaults to the first open list.
        #[arg(long)]
        list: Option<String>,
        /// Which integration (see `hosting issues`).
        #[arg(long)]
        id: Option<String>,
        /// Print the host's own prefilled new-issue page instead of
        /// calling its API (needs no token).
        #[arg(long)]
        web: bool,
        /// Someone to assign (GitHub, GitLab, Gitea, Forgejo); repeat the flag or
        /// comma-separate. See `hosting members`.
        #[arg(long = "assignee", value_delimiter = ',')]
        assignees: Vec<String>,
        /// The milestone, by title (GitHub, GitLab, Gitea, Forgejo). See `hosting
        /// milestones`.
        #[arg(long)]
        milestone: Option<String>,
    },
    /// Changes an issue's assignees, labels and milestone (GitHub, GitLab,
    /// Gitea, Forgejo). Outward action: it writes to the host. Prints the
    /// metadata as it is afterwards.
    Edit {
        /// The issue number (GitHub) or `iid` (GitLab).
        number: u64,
        #[command(flatten)]
        meta: IssueMetaArgs,
        /// Which integration (see `hosting issues`).
        #[arg(long)]
        id: Option<String>,
    },
    /// The issue types and lists the linked project offers to
    /// `issue create --type` / `--list`. Read-only.
    Options {
        /// Which integration (see `hosting issues`).
        #[arg(long)]
        id: Option<String>,
    },
    /// Closes an issue of a local Markdown folder (its status becomes the
    /// done status) or of a Redmine project (the first closed status).
    /// Writes the note or the issue.
    Close {
        /// The issue number.
        number: u64,
        /// Which integration (see `hosting issues`).
        #[arg(long)]
        id: Option<String>,
    },
    /// Reopens an issue of a local Markdown folder (its status becomes the
    /// first one that is not the done status) or of a Redmine project (the
    /// first open status). Writes the note or the issue.
    Reopen {
        /// The issue number.
        number: u64,
        /// Which integration (see `hosting issues`).
        #[arg(long)]
        id: Option<String>,
    },
    /// Moves an issue of a local Markdown folder or a Redmine project to a
    /// status (a Redmine status name, e.g. `In Progress`). Writes the note
    /// or the issue.
    Move {
        /// The issue number.
        number: u64,
        /// The status (`todo`, `doing`, ...).
        #[arg(long)]
        status: String,
        /// Which integration (see `hosting issues`).
        #[arg(long)]
        id: Option<String>,
    },
    /// Writes `issues.base` (an Obsidian Base with a table and a kanban
    /// view grouped by status) next to the notes of a local Markdown
    /// folder. Never overwrites an existing file.
    Base {
        /// Which integration (see `hosting issues`).
        #[arg(long)]
        id: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum SshKeyAction {
    /// The public keys already on the account.
    List {
        #[arg(long)]
        id: Option<String>,
    },
    /// Generates a new pair and uploads the public half. Refuses to
    /// overwrite an existing key file.
    Create {
        #[arg(long)]
        id: Option<String>,
        /// Where to write the private key. Defaults to
        /// `~/.ssh/id_ed25519`.
        #[arg(long)]
        path: Option<String>,
        /// The name the key appears under on the host. Defaults to
        /// `gitpoulpi@<hostname>`.
        #[arg(long)]
        title: Option<String>,
        /// Generate the pair but do not send it anywhere.
        #[arg(long)]
        no_upload: bool,
    },
    /// Removes a public key from the account on the host (by the id
    /// `ssh-key list` reports). Local key files are left alone.
    Delete {
        /// The host's id of the key.
        key_id: String,
        /// Which integration.
        #[arg(long)]
        id: Option<String>,
    },
}
