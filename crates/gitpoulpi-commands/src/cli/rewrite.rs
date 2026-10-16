use clap::{Args, Subcommand};

/// The encodings `--reencode`, `--from` and `--encoding-fallback` name.
const ENCODINGS: [&str; 6] = [
    "utf-8",
    "utf-8-bom",
    "utf-16le",
    "utf-16be",
    "windows-1252",
    "iso-8859-1",
];

/// Where the recipe of `rewrite preview` / `rewrite run` comes from: a file
/// or JSON (the same TOML / JSON the TUI workbench saves), and/or a few flags
/// for the common rules. Flag rules are added after the recipe's own rules.
#[derive(Debug, Clone, Default, Args)]
pub struct RewriteRecipeArgs {
    /// A recipe file (TOML or JSON, the TUI workbench's format), or `-` for
    /// JSON on standard input.
    #[arg(long)]
    pub recipe: Option<String>,
    /// A recipe as a JSON string (for callers that cannot write a file).
    #[arg(long)]
    pub recipe_json: Option<String>,
    /// A file with one secret per line (blank lines ignored). The secrets
    /// are added to the recipe's `content_replace` rules that search for
    /// secrets, so they never travel in the command line, a saved recipe or
    /// a log.
    #[arg(long)]
    pub secrets_file: Option<String>,
    /// Rewrite every local branch and tag instead of the current branch.
    #[arg(long, conflicts_with = "refs")]
    pub all_local: bool,
    /// Rewrite only these local branches or tags (repeatable).
    #[arg(long = "ref", value_name = "REF")]
    pub refs: Vec<String>,
    /// With the current branch: only the commits after this revision.
    #[arg(long, conflicts_with_all = ["all_local", "refs"])]
    pub since: Option<String>,
    /// Identity rule: match this author / committer email.
    #[arg(long)]
    pub match_email: Option<String>,
    /// Identity rule: match this author / committer name.
    #[arg(long)]
    pub match_name: Option<String>,
    /// Identity rule: the name to set (with `--set-email`, at least one).
    #[arg(long)]
    pub set_name: Option<String>,
    /// Identity rule: the email to set.
    #[arg(long)]
    pub set_email: Option<String>,
    /// Identity rule: `author`, `committer` or `both` (default).
    #[arg(long, value_parser = ["author", "committer", "both"])]
    pub who: Option<String>,
    /// Mailmap rule: rewrite authors and committers to what the worktree's
    /// `.mailmap` names them (the history ends up showing what `git log
    /// --use-mailmap` shows). `--who` picks author, committer or both. For
    /// a display-only fix without rewriting, use `mailmap add` instead.
    #[arg(long)]
    pub mailmap: bool,
    /// With `--mailmap`: apply this file instead of the worktree's `.mailmap`
    /// (relative to the repository root, or absolute).
    #[arg(long, value_name = "FILE", requires = "mailmap")]
    pub mailmap_file: Option<String>,
    /// Date rule: move every date by this many seconds (negative: earlier),
    /// keeping each commit's own time zone offset.
    #[arg(long, allow_hyphen_values = true)]
    pub shift_dates: Option<i64>,
    /// Message rule `FIND=WITH` (literal text, repeatable).
    #[arg(long, value_name = "FIND=WITH")]
    pub replace_message: Vec<String>,
    /// Trailer rule `KEY=VALUE`: add this trailer to every message (repeatable;
    /// a message that already has exactly that trailer is left alone).
    #[arg(long = "trailer-add", value_name = "KEY=VALUE")]
    pub trailer_add: Vec<String>,
    /// Trailer rule: remove every trailer with this key (repeatable).
    #[arg(long = "trailer-remove", value_name = "KEY")]
    pub trailer_remove: Vec<String>,
    /// Trailer rule `KEY:OLD=NEW`: replace literal text in the values of the
    /// trailers with this key, for instance `Signed-off-by:@old.org=@new.org`
    /// (repeatable).
    #[arg(long = "trailer-map", value_name = "KEY:OLD=NEW")]
    pub trailer_map: Vec<String>,
    /// With an identity rule: also map the identity trailers (`Name <email>`
    /// in `Signed-off-by`, `Co-authored-by`...) that match, not only the
    /// author and committer.
    #[arg(long)]
    pub map_trailers: bool,
    /// Replace abbreviated commit ids (7 or more hex characters) in messages
    /// by the new ids; ambiguous or unknown ones are left and counted.
    #[arg(long)]
    pub update_message_hashes: bool,
    /// Committer of rewritten commits: `keep` (default) or `me-now`.
    #[arg(long, value_parser = ["keep", "me-now"])]
    pub committer: Option<String>,
    /// What to do with the signatures of rewritten commits: `strip`,
    /// `resign-signed` or `resign-all`. Required when signed commits change.
    #[arg(long, value_parser = ["strip", "resign-signed", "resign-all"])]
    pub signatures: Option<String>,
    /// Signature rule: strip the signature of every commit in scope, even
    /// where nothing else changes. Those commits need no `--signatures`
    /// answer.
    #[arg(long)]
    pub remove_signatures: bool,
    /// Message rule: re-encode commit messages to UTF-8. A message with an
    /// `encoding` header (ISO-8859-1, Shift_JIS...) is decoded and the header
    /// goes; UTF-8 once read as Latin-1 or Windows-1252 ("RÃ©Ã©criture") is
    /// repaired only when the round trip is exact. What cannot be decided is
    /// left alone and counted (`encoding_left`).
    #[arg(long)]
    pub fix_message_encoding: bool,
    /// With `--fix-message-encoding`: also repair mangled author and
    /// committer names (the names of a commit whose header goes are always
    /// decoded with it).
    #[arg(long)]
    pub fix_name_encoding: bool,
    /// Also rewrite these protected branches that are already pushed
    /// (excluded by default; repeatable).
    #[arg(long = "include-protected", value_name = "BRANCH")]
    pub include_protected: Vec<String>,
    /// Content rule: remove files and folders matching this glob from every
    /// commit (repeatable; a pattern without a slash matches at any depth).
    #[arg(long = "remove-path", value_name = "GLOB")]
    pub remove_paths: Vec<String>,
    /// Content rule: remove every file larger than this many bytes from
    /// every commit.
    #[arg(long = "remove-blobs-over", value_name = "BYTES")]
    pub remove_blobs_over: Option<u64>,
    /// LFS rule: move the files matching this glob into Git LFS in every
    /// commit (repeatable; `.gitattributes`-style: no slash matches at any
    /// depth). Needs git-lfs. The content goes into the local LFS store and
    /// `.gitattributes` gets the `filter=lfs` line; push the objects after.
    #[arg(long = "lfs-import", value_name = "PATTERN")]
    pub lfs_import: Vec<String>,
    /// With `--lfs-import`: only files of at least this many bytes move.
    #[arg(long = "lfs-min-size", value_name = "BYTES", requires = "lfs_import")]
    pub lfs_min_size: Option<u64>,
    /// LFS rule: turn the LFS pointers matching this glob back into the real
    /// files in every commit (repeatable). The objects must be in the local
    /// LFS store (`lfs fetch` first); refused with the list otherwise.
    #[arg(long = "lfs-export", value_name = "PATTERN")]
    pub lfs_export: Vec<String>,
    /// Line-ending rule: every text file matching `--eol-paths` gets this
    /// ending (`lf` for Linux and macOS, `crlf` for Windows, `cr` for classic
    /// Mac); mixed endings are made uniform and binary files are skipped.
    #[arg(long, value_name = "ENDING", value_parser = ["lf", "crlf", "cr"], requires = "eol_paths")]
    pub eol: Option<String>,
    /// With `--eol`: the files it converts (globs, repeatable; a pattern
    /// without a slash matches at any depth).
    #[arg(long = "eol-paths", value_name = "GLOB", num_args = 1.., requires = "eol")]
    pub eol_paths: Vec<String>,
    /// With `--eol`: files it leaves alone (globs, repeatable).
    #[arg(long = "eol-exclude", value_name = "GLOB", num_args = 1.., requires = "eol")]
    pub eol_exclude: Vec<String>,
    /// With `--eol`: read each commit's root `.gitattributes`: `-text` and
    /// `binary` files are skipped, `eol=lf` and `eol=crlf` files keep their
    /// own ending.
    #[arg(long = "eol-follow-attributes", requires = "eol")]
    pub eol_follow_attributes: bool,
    /// File-mode rule: the executable bit (100755 becomes 100644) leaves the
    /// files matching these globs (repeatable). Content is untouched.
    #[arg(long = "strip-exec", value_name = "GLOB", num_args = 1..)]
    pub strip_exec: Vec<String>,
    /// With `--strip-exec`: files that keep their executable bit (globs).
    #[arg(long = "strip-exec-exclude", value_name = "GLOB", num_args = 1.., requires = "strip_exec")]
    pub strip_exec_exclude: Vec<String>,
    /// Encoding rule: the text files matching `--encoding-paths` are
    /// converted to this encoding (`utf-8`, `utf-8-bom`, `utf-16le`,
    /// `utf-16be`, `windows-1252`, `iso-8859-1`). Binary files, files the
    /// source encoding cannot read and characters the target cannot hold are
    /// left as they are and counted.
    #[arg(long, value_name = "TARGET", value_parser = clap::builder::PossibleValuesParser::new(ENCODINGS), requires = "encoding_paths")]
    pub reencode: Option<String>,
    /// With `--reencode`: the encoding the files are in. Omitted: detected
    /// per file (byte order mark, valid UTF-8, else `--encoding-fallback`).
    #[arg(long = "from", value_name = "SOURCE", value_parser = clap::builder::PossibleValuesParser::new(ENCODINGS), requires = "reencode")]
    pub reencode_from: Option<String>,
    /// With `--reencode`: what a file that is neither marked nor UTF-8 is
    /// read as when the source is detected (default `windows-1252`).
    #[arg(long = "encoding-fallback", value_name = "ENCODING", value_parser = clap::builder::PossibleValuesParser::new(ENCODINGS), requires = "reencode")]
    pub encoding_fallback: Option<String>,
    /// With `--reencode`: the files it converts (globs, repeatable).
    #[arg(long = "encoding-paths", value_name = "GLOB", num_args = 1.., requires = "reencode")]
    pub encoding_paths: Vec<String>,
    /// With `--reencode`: files it leaves alone (globs, repeatable).
    #[arg(long = "encoding-exclude", value_name = "GLOB", num_args = 1.., requires = "reencode")]
    pub encoding_exclude: Vec<String>,
    /// Content rule `FIND=WITH`: replace literal text in every text file
    /// (repeatable; binary files are skipped and counted). For secrets use a
    /// `content_replace` rule in the recipe with `--secrets-file`.
    #[arg(long = "replace-content", value_name = "FIND=WITH")]
    pub replace_content: Vec<String>,
    /// Path rule `FROM=TO`: rename a file or folder in every commit, the same
    /// blobs and trees at the new path (repeatable, in order, after the
    /// content rules). A commit where `TO` already exists is not changed: the
    /// preview is refused (exit code 2, `path_collision`) and lists the
    /// commits and paths.
    #[arg(long = "move-path", value_name = "FROM=TO")]
    pub move_paths: Vec<String>,
    /// Path rule: move the whole repository into this subdirectory in every
    /// commit (to merge repositories without path clashes).
    #[arg(long = "into-subdir", value_name = "DIR")]
    pub into_subdir: Option<String>,
    /// Path rule: keep only this subdirectory, which becomes the root; the
    /// commits that do not touch it are dropped and merges are simplified.
    /// Asked each time: give `--extract-mode`, else the exit code is 3 and
    /// nothing is previewed. The extraction runs before the other path flags.
    #[arg(long = "extract", value_name = "DIR")]
    pub extract: Option<String>,
    /// How `--extract` ends up: `clone` (a new local clone in `--target`, the
    /// original is not touched; every local branch and tag), `in-place` (this
    /// repository's history is rewritten, one undo entry) or `submodule` (the
    /// clone as in `clone`, and one new commit on the current branch replaces
    /// the folder by a submodule of it; this history is kept).
    #[arg(long = "extract-mode", value_name = "MODE", value_parser = ["clone", "in-place", "submodule"], requires = "extract")]
    pub extract_mode: Option<String>,
    /// The folder of the new repository for `--extract-mode clone` or
    /// `submodule`: it must not exist or be empty, and not be inside this
    /// repository. Used by `rewrite run`; `rewrite apply` takes its own.
    #[arg(long, value_name = "PATH")]
    pub target: Option<String>,
    /// Commits that a content rule leaves empty: `auto` drops them (default),
    /// `never` keeps them.
    #[arg(long, value_parser = ["auto", "never"])]
    pub prune_empty: Option<String>,
    /// Content rule: remove a secret that `rewrite scan-secrets` found, by
    /// its finding id (repeatable), or `all` for every finding in the scope.
    /// The daemon scans again and resolves the ids itself: the value never
    /// reaches the command line, a recipe or a log.
    #[arg(long = "remove-finding", value_name = "ID")]
    pub remove_findings: Vec<String>,
    /// Script rule: run this script file on every commit (repeatable, in
    /// order, after the other flags' rules). The file is read here and
    /// embedded in the recipe; see `rewrite script check` for the format. A
    /// `.lua` file runs in the embedded sandboxed Lua; a shell, Python or
    /// external Lua script is not sandboxed and needs trust (`--trust`, or
    /// `rewrite script trust FILE` before).
    #[arg(long = "script", value_name = "FILE")]
    pub scripts: Vec<String>,
    /// The language of every `--script`: `lua` (embedded, sandboxed), `shell`,
    /// `python` or `lua-external`. Default: by the file's extension (`.lua`;
    /// `.sh`, `.bash`, `.ps1`; `.py`), else `lua`. Python and external Lua
    /// must be switched on in `rewrite.external_scripts`.
    #[arg(long = "script-language", value_name = "LANGUAGE", value_parser = ["lua", "shell", "python", "lua-external"])]
    pub script_language: Option<String>,
    /// Trust the external scripts (shell, Python, external Lua) of this
    /// recipe, by their content: remembered for this repository, and asked
    /// again if a script changes. A person's decision: the MCP server
    /// refuses it, and an agent can only run scripts a person trusted.
    #[arg(long)]
    pub trust: bool,
    /// A script parameter `NAME=VALUE`, given to every `--script`
    /// (repeatable). The names are the ones the script's header declares.
    #[arg(long = "script-param", value_name = "NAME=VALUE")]
    pub script_params: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum RewriteScriptAction {
    /// Compile a Lua script and print its header (name, window, parameters),
    /// the functions it defines and every problem. Exit code 2 when it has
    /// problems. Reads only; nothing runs on a commit.
    ///
    /// For a shell, Python or external Lua script (not sandboxed) it also
    /// says whether the interpreter was found and the script is trusted, and
    /// for a trusted one does a dry handshake: it starts the script once with
    /// a made-up commit (`"dry_run": true`) and checks the answer. An
    /// untrusted script is never started.
    Check {
        /// The script file.
        file: String,
        /// A parameter `NAME=VALUE` (repeatable), checked against the header.
        #[arg(long = "param", value_name = "NAME=VALUE")]
        params: Vec<String>,
        /// `lua`, `shell`, `python` or `lua-external`; default by extension.
        #[arg(long = "language", value_name = "LANGUAGE", value_parser = ["lua", "shell", "python", "lua-external"])]
        language: Option<String>,
        /// Trust this script first (a person's decision, refused over MCP).
        #[arg(long)]
        trust: bool,
    },
    /// Run a script on some commits and print what it would change and what
    /// it logged, in history order with the state carried from one commit to
    /// the next. Reads only: nothing is written. Exit code 2 when the script
    /// fails (the answer names the commit and the reason).
    Test {
        /// The script file.
        file: String,
        /// A parameter `NAME=VALUE` (repeatable).
        #[arg(long = "param", value_name = "NAME=VALUE")]
        params: Vec<String>,
        /// A commit to run on (repeatable; a revision or an id).
        #[arg(long = "commit", value_name = "REV", required = true)]
        commits: Vec<String>,
        /// `lua`, `shell`, `python` or `lua-external`; default by extension.
        #[arg(long = "language", value_name = "LANGUAGE", value_parser = ["lua", "shell", "python", "lua-external"])]
        language: Option<String>,
        /// Trust this script first (a person's decision, refused over MCP).
        #[arg(long)]
        trust: bool,
    },
    /// Trust an external script (shell, Python, external Lua) by its content:
    /// the SHA-256 of the file is remembered for this repository, so the
    /// script may run (preview, run, test); a changed file asks again. These
    /// scripts are not sandboxed: they run with your rights, so read the
    /// file first. Only a person can do this: there is no MCP tool for it.
    Trust {
        /// The script file.
        file: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum RewriteAction {
    /// Which tools the rewrite features need (gpg / ssh-keygen / gpgsm for
    /// re-signing, script interpreters, git-lfs) and whether each is found.
    /// Reads only.
    Tools {
        /// Probe again instead of answering from the one-minute cache.
        #[arg(long)]
        fresh: bool,
    },
    /// Sign a dummy payload once with the repository's signing setup
    /// (`gpg.format`, `user.signingkey`, `gpg.*.program`) and verify it, as
    /// the smoke test before a run that re-signs (`--signatures
    /// resign-signed` or `resign-all`). Prints the program, whether it works
    /// and how long one signature took (multiply by the commits to re-sign
    /// for the estimate). May open a passphrase prompt. Exits 2 when it does
    /// not work, with the reason (`tool_missing` or `signing_failed`).
    CheckSigning,
    /// Scan the history for secrets: well-known token prefixes (GitHub,
    /// GitLab, Slack, AWS, Stripe, npm, Google, OpenAI, Anthropic), private
    /// key blocks, JWTs, passwords in URLs and `password = value` lines that
    /// look random. Prints each finding's rule, file, first and last commit,
    /// counts and a REDACTED preview (`ghp_****abcd`); the value is never
    /// printed, logged or stored. Not exhaustive. A secret that was pushed is
    /// leaked: rotate it first. Reads only.
    ScanSecrets {
        /// Scan every local branch and tag instead of the current branch.
        #[arg(long, conflicts_with = "refs")]
        all_local: bool,
        /// Scan only these local branches or tags (repeatable).
        #[arg(long = "ref", value_name = "REF")]
        refs: Vec<String>,
        /// With the current branch: only the commits after this revision.
        #[arg(long, conflicts_with_all = ["all_local", "refs"])]
        since: Option<String>,
    },
    /// Size analysis of the history, like `filter-repo --analyze`: the largest
    /// blobs with their path, size, first and last commit and whether they are
    /// still in HEAD, the biggest folders, the files deleted from HEAD that the
    /// history still holds, and the totals with what removing the history-only
    /// part would free (an estimate: after a purge and a gc only). Remove what
    /// it shows with `rewrite preview --remove-paths PATH` or
    /// `--remove-blobs-over BYTES`. Reads only.
    Analyze {
        /// Analyse every local branch and tag instead of the current branch.
        #[arg(long, conflicts_with = "refs")]
        all_local: bool,
        /// Analyse only these local branches or tags (repeatable).
        #[arg(long = "ref", value_name = "REF")]
        refs: Vec<String>,
        /// With the current branch: only the commits after this revision.
        #[arg(long, conflicts_with_all = ["all_local", "refs"])]
        since: Option<String>,
        /// Rows kept in each list (default 50, at most 5000).
        #[arg(long, default_value_t = 50)]
        top: u32,
    },
    /// Plan a history rewrite without changing anything: counts, the refs
    /// that would move, warnings and the first rows. Keeps the plan in the
    /// daemon as a run (`run`, `digest`) for `rewrite rows` and
    /// `rewrite apply`. The preview also lives on disk in the repository's
    /// git dir, so `rewrite apply` works from a later command, after a daemon
    /// restart or from another frontend, without starting a daemon; only
    /// `rewrite rows` needs the daemon that previewed (a running one, or use
    /// `--all` on the preview). Exit code 2 when the recipe is refused.
    Preview {
        #[command(flatten)]
        recipe: RewriteRecipeArgs,
        /// How many changed commits to print (default 50).
        #[arg(long, default_value_t = 50)]
        limit: usize,
        /// Print every changed commit.
        #[arg(long)]
        all: bool,
    },
    /// Page through the commits of a previewed run.
    Rows {
        /// The run id `rewrite preview` printed.
        #[arg(long)]
        run: String,
        /// `changed` (default), `dropped`, `pushed` or `all`.
        #[arg(long, value_parser = ["changed", "dropped", "pushed", "all"], default_value = "changed")]
        filter: String,
        /// Rows to skip.
        #[arg(long, default_value_t = 0)]
        offset: usize,
        /// Rows per page (at most 200).
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Apply a previewed run: moves the refs as ONE undo entry (`gitpoulpi
    /// undo` puts everything back). Never pushes. Exit code 2 when refused
    /// (the repository changed since the preview, wrong digest, ...), 3 when
    /// the plan moves refs that are on a remote and `--yes` is missing.
    Apply {
        /// The run id `rewrite preview` printed.
        #[arg(long)]
        run: String,
        /// The digest `rewrite preview` printed: the plan you reviewed.
        #[arg(long)]
        digest: String,
        /// Confirm moving refs that are already pushed. Without it such a
        /// plan changes nothing and the exit code is 3.
        #[arg(long)]
        yes: bool,
        /// The folder of the new repository when the plan extracts a
        /// subdirectory as a clone or a submodule (see `--extract-mode`).
        #[arg(long, value_name = "PATH")]
        target: Option<String>,
    },
    /// Preview and apply in one call, for scripts. Same refusals and exit
    /// codes as `preview` and `apply`; on exit code 3 the preview is
    /// discarded and nothing changed.
    Run {
        #[command(flatten)]
        recipe: RewriteRecipeArgs,
        /// Confirm moving refs that are already pushed.
        #[arg(long)]
        yes: bool,
    },
    /// What a content rule did to one file of one commit of a previewed run:
    /// the old and the new file diffed. The lines a secret rule removes are
    /// hidden. Reads only.
    Diff {
        /// The run id `rewrite preview` printed.
        #[arg(long)]
        run: String,
        /// The commit as it is before the rewrite (an id from the rows).
        #[arg(long)]
        commit: String,
        /// The file, relative to the repository root.
        #[arg(long)]
        path: String,
    },
    /// Scripts for history rewrites: check one, try it on a few commits, or
    /// trust an external one. Embedded Lua is sandboxed; shell, Python and
    /// external Lua scripts are not (see `rewrite.external_scripts`). Run one
    /// for real with `rewrite preview --script FILE`.
    Script {
        #[command(subcommand)]
        action: RewriteScriptAction,
    },
    /// Push the refs an applied run moved that were on a remote, each with
    /// `--force-with-lease=<ref>:<the id the remote had when you previewed>`
    /// (never a plain force), one `--atomic` push per remote where the server
    /// allows it (one push per ref otherwise, and the answer says so). Tags
    /// are included. Protected branches (`rewrite.protected_branches`) are
    /// left out unless listed with `--include-protected` or all of them with
    /// `--include-all-protected`. A ref whose remote moved since the preview
    /// is rejected for that ref (fetch and review, then run it again). Not
    /// undoable: the remote keeps the new history even if you `undo` the
    /// rewrite locally. Without `--yes` (or with `--dry-run`) nothing is
    /// sent: the answer lists what would be pushed, and the exit code is 3
    /// without `--yes`. Exit code 2 when the run is gone, 4 when a ref was
    /// rejected or not pushed.
    Push {
        /// The run id `rewrite apply` printed; `latest` (default) is the
        /// most recently applied run.
        #[arg(long, default_value = "latest")]
        run: String,
        /// Only the refs of this remote.
        #[arg(long)]
        remote: Option<String>,
        /// Push this protected branch too (repeatable).
        #[arg(long = "include-protected", value_name = "BRANCH")]
        include_protected: Vec<String>,
        /// Push every protected branch of the run too.
        #[arg(long)]
        include_all_protected: bool,
        /// Ask the remote what would happen (leases checked) and send nothing.
        #[arg(long)]
        dry_run: bool,
        /// One push per ref instead of one all-or-nothing `--atomic` push.
        #[arg(long)]
        no_atomic: bool,
        /// Go ahead and push. Without it nothing is sent (exit code 3).
        #[arg(long)]
        yes: bool,
    },
    /// The note to send to the people who have the old history: per pushed
    /// branch and tag the old and new tip and what to run (`git fetch`, then
    /// `git reset --hard origin/<branch>` or `git rebase --onto`). Reads only.
    Note {
        /// The run id; `latest` (default) is the most recently applied run.
        #[arg(long, default_value = "latest")]
        run: String,
    },
    /// The run's `commit-map` and `ref-map` (git-filter-repo's format: a
    /// header line, then `old new` per line; GitLab's repository cleanup
    /// takes the commit map). Prints their paths, or copies them to `--out`.
    Maps {
        /// The run id; `latest` (default) is the most recently applied run.
        #[arg(long, default_value = "latest")]
        run: String,
        /// Copy both files into this directory (created when missing).
        #[arg(long, value_name = "DIR")]
        out: Option<String>,
    },
    /// Delete the old history of a rewrite from this clone for good: drops
    /// the undo pins of the rewrite (and of any earlier operation whose pins
    /// also hold the old history), expires the reflog entries that name it,
    /// prunes the objects (`gc --prune=now`) and checks that they are gone.
    /// Undo is no longer possible for those operations (they stay in
    /// `undo --list` as purged). It cannot be stopped once it starts and it
    /// cannot be undone. Anything that still holds the old history (a branch
    /// outside the rewrite, a stash, another worktree's HEAD, a remote-tracking
    /// ref: fetch after the push) blocks it and nothing is changed. Nothing
    /// is sent to any remote; the answer lists the steps the host needs
    /// (GitHub, GitLab, Azure DevOps, Bitbucket). Without `--yes` nothing
    /// happens: the answer says what would be deleted and lost, and the exit
    /// code is 3. Exit code 2 when the entry is not a rewrite, 4 when
    /// something blocks it or objects remain. Over MCP only with
    /// `[mcp] allow_history_purge = true`.
    Purge {
        /// A history rewrite entry of the undo history (`undo --list`;
        /// repeatable). Default: the newest rewrite that is not purged.
        #[arg(long = "entry", value_name = "ID", conflicts_with_all = ["latest", "all"])]
        entries: Vec<u64>,
        /// The newest rewrite that is not purged yet (the default).
        #[arg(long, conflicts_with = "all")]
        latest: bool,
        /// Every history rewrite of the undo history.
        #[arg(long)]
        all: bool,
        /// Go ahead and delete. Without it nothing is changed (exit code 3).
        #[arg(long)]
        yes: bool,
    },
    /// Drop a previewed run without applying it. Discarding twice is fine.
    Discard {
        /// The run id `rewrite preview` printed.
        #[arg(long)]
        run: String,
    },
}
