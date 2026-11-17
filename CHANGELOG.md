# Changelog

All notable changes to GitPoulpi are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/) (from 1.0.0 on, the CLI, config and
IPC surfaces are stable within a major version).

## [Unreleased]

## [1.0.0] - Unreleased

The first public release of GitPoulpi: a Git client for the terminal with a clickable commit graph, a keyboard-first command palette, and a CLI and MCP server that do everything the interface does. No Electron, no account.

### Highlights
- **Commit graph with mouse support:** right-click menus, drag and drop of refs onto commits, double-click, multi-select, and a graph drawn as real pictures in kitty, Ghostty, WezTerm, iTerm2 and sixel terminals (a character graph elsewhere).
- **Keyboard first:** every mouse action has a shortcut, every command can be rebound, and the command palette reaches 100% of the functionality.
- **CLI and MCP with the same surface:** `gitpoulpi` and `gitpoulpi-mcp` run one shared command layer, so an AI agent can do what you do. MCP is read-only until you set `[mcp] allow_mutations = true`.
- **One daemon per session with live sync:** a change made in the TUI, the CLI or by an agent shows up at once in the others.
- **A safety net:** undo and redo for destructive actions, every position a branch has had, and a list of lost commits.
- **Code hosts:** GitHub, GitLab (including self-managed), Gitea, Forgejo (including Codeberg), Azure DevOps, Bitbucket Cloud and Data Center, and Gerrit, with personal tokens.
- **Issue trackers:** Jira, Trello, Linear, YouTrack, Redmine and local Markdown notes, with an issue board.
- **Reviews and conversations:** read, comment and approve pull requests, merge requests and Gerrit changes without leaving the window.
- **AI, off by default and never needs an account:** commit messages, explanations, splitting commits, rewrite rules; OpenAI-compatible, Anthropic or a local agent CLI.
- **History rewrite workbench:** change authors, dates, messages, files, signatures and LFS usage across a whole history, with a preview and one undoable Apply.
- **Workflows, workspaces and the Radar:** customizable Git Flow, repository groups, and what waits for you across all of them.
- **17 languages, themes, and signed updates** on every install channel, on Windows, Linux and macOS.

### Added

#### Log & graph
- Lazy-loaded graph with author avatars (identicon fallback, Gravatar, GitHub and GitLab), stashes in the graph, a hidden-refs counter, and solo or hide per folder or remote.
- Ref pills with inline branch and tag naming, a merged ref and commit context menu, and a faded ghost branch label that creates a branch on click (`ui.graph_ghost_ref`).
- Interactive rebase (reorder, squash, fixup, reword, edit, autosquash), multi-commit cherry-pick, fast-forward, worktrees, patches, revert of a merge commit, and amend of message, author and dates.
- Split a commit into several, from the Log menu, the palette or the rebase popup, with an optional AI grouping.
- Log columns: configurable, with a **Scroll sideways** layout (`ui.log_columns_layout = "scroll"`), double-click on a boundary to fit a column, and a Changes column (files, +N -M).
- Search and filter, a global **Search** button for commits, files and branches, `.mailmap` support with an Identities tab (`log.use_mailmap`), and a Reflog view that loads on demand.
- Copy commits as a Mermaid gitGraph; the status bar shows `loaded / total` commits (`gitpoulpi log --count`).
- Tags in folders split on `/`, Refs filters, submodules in the Refs sidebar, and right-click on a left-pane header to show or hide sections.

#### Staging, diffs & files
- The working changes are one row in the Log, with Unstaged and Staged panes; stage, unstage and discard by file, folder, hunk or single line.
- Diff view: inline or split, hunks only or whole file, minimap, selectable text; File View, Blame, File history with the commit's diff, an image view, and a built-in editor with syntax highlighting.
- A compact commit column beside file views moves a file view to another commit (`Alt+C` hides it; shown from 140 terminal columns).
- A three-way merge conflict editor with output editing, find and non-text conflicts; a strip on the Log warns before a merge that would conflict, with **Open merge preview** and **Rebase onto base** (`gitpoulpi conflicts check`).
- Committing with files still in conflict opens a dialog that lists them instead of failing.
- Commit composer, `Ctrl+G Enter` to commit from anywhere, and repository settings for Sparse checkout, LFS, Commit (`--no-verify`), Merge options and Message template.
- Git LFS from the toolbar: install, track, checkout, locks, pointer awareness, and pre-flight checks before pull and push.
- Open in: files, folders and terminals in your tools.

#### Undo & safety
- Undo and redo (`Ctrl+Z`, `Ctrl+Y`) for discards, deletes, patches, staging and history operations, with a dated Undo history.
- The safety net keeps every position a branch has had, including moves made by other programs, and restores a single branch.
- A failed git operation shows what went wrong and a next step (Pull first, Stash and retry, Retry, Open Integrations), with Details and Copy. The CLI and MCP carry the same facts under `detail`.
- Network operations can be cancelled and clean up after themselves; a stale `index.lock` is detected and offered a fix. Tokens and passwords are masked everywhere.

#### History rewrite
- A recipe is a scope plus ordered rules: replace or anonymise authors, apply `.mailmap`, shift dates, change time zones, edit messages and trailers, search and replace in files, remove files or large files, move paths, convert line endings and encodings, migrate to or from LFS, remove signatures.
- Preview shows counts, what each rule did and which branches and tags would move. Apply is one undo entry; Push (force with lease) and Purge are separate steps.
- Script rules in Lua, shell or Python, trusted by content hash; a secret scanner for history.
- Available as `gitpoulpi rewrite` and as MCP tools. Agents can preview but never apply on their own.

#### Branching workflows
- A workflow describes base branches, topic types, name patterns and parents, with presets, detection and import. The `Flow` menu starts, finishes, syncs and promotes branches, and the sidebar groups branches by workflow. Also in the CLI and MCP.

#### Workspaces, Home & tabs
- Workspaces group repositories, remember which are open and can carry a default commit profile (`gitpoulpi workspace`).
- Home is one list (Favorites, Open, Recent, All) with one filter. Tabs can be reordered, and worktree tabs sit after their repository.
- The Radar (`Alt+R`) lists your pull requests, review requests, assigned issues and unfinished work across repositories, and can be turned off.
- `gp -a` (or `gitpoulpi --add [path]`) opens a folder as a new tab of the GitPoulpi that is already running.

#### Hosting & issues
- Clone from a connected account in the background; per-profile tokens and SSH keys, with a per-repository SSH key pin.
- Review a pull request, merge request or Gerrit change: description, conversation, files with comment threads, commits, approval and requested changes.
- Create pull requests and issues with assignees, reviewers, labels and milestones; a toolbar chip for the linked pull request (`ui.hide_finished_pr_chip`); issues from a per-repository source.

#### AI
- Providers: OpenAI, OpenRouter, OpenAI-compatible endpoints, Anthropic, or a local agent CLI such as `claude -p`.
- Commit messages, change explanations, stash messages, pull request descriptions, splitting working changes or a commit, and history rewrite rules.
- Global, per-action and per-repository instructions and profiles with their own environment variables.
- Input limits are visible, not silent: one setting per cap under `[ai.limits]`. An input over its limit opens *Over the AI limit* with Send all (this request only, still redacted), Send truncated, Open setting and Cancel; the CLI and MCP report `truncated` and accept `--send-all`.
- AI can be left out entirely of a custom build.

#### CLI & MCP
- `gitpoulpi` covers every operation of the UI, with `--json` output; the MCP server offers one tool per CLI command over stdio or HTTP, gated by the `[mcp]` table of `config.toml`.
- Background jobs are listed and cancellable (`gitpoulpi job list | cancel | wait`). Agents can ask the window to show a diff, a file at a position or a highlight.
- `gitpoulpi alias` and `gitpoulpi path` create aliases such as `gp` and put GitPoulpi on your PATH; they are not available to MCP agents.

#### Interface & languages
- 17 languages: Chinese (Simplified), Czech, Dutch, English, French, German, Italian, Japanese, Korean, Polish, Portuguese (Brazil and Portugal), Russian, Spanish, Swedish, Turkish and Ukrainian; git words can stay in English (`ui.git_terms_english`).
- Themes: GitPoulpi Dark and Light, GitHub and VS Code presets, imported terminal palettes, your own themes, and any lane palette with any theme.
- Settings with search, inline actions and master-detail lists; every palette command can get its own shortcut. The Activity Log lists the session's messages and the application log. **Licenses and credits** in About and `gitpoulpi licenses`.

#### Install & updates
- Windows installer (per user, with PATH and alias choices), `install.sh` for Linux and macOS, and packages for Chocolatey, Scoop, Homebrew, apt, MSI, cargo and Docker. `gitpoulpi uninstall` removes what was installed.
- Every release file is signed with the project's minisign key and an update installs only when the signature verifies. **Update now** stages the update, **Restart to update** swaps it in, **Revert** goes back; `updates.mode` is `notify`, `auto` or `off`.
- Security notices: the daily check reads the signed `security.json`, and a notice that names the running version turns the version chip red on every install channel.
- Custom builds choose languages, integrations, features and themes (`xtask build-custom`).

### Good to know
- **Requirements:** a system `git` 2.34 or newer on your `PATH`, and a terminal with mouse support. A Nerd Font or a font with emoji gives the icons. `git-lfs` 3.0 or newer and a credential helper are optional.
- **Configuration:** `config.toml` lives in `%APPDATA%\GitPoulpi` (Windows), `~/.config/gitpoulpi` (Linux) or `~/Library/Application Support/GitPoulpi` (macOS); `gitpoulpi config path` prints it.
- **AI agents are read-only by default:** `[mcp] allow_mutations` is `false` until you set it, and history purge needs `allow_history_purge`.
- **Reporting a vulnerability:** privately, as described in [SECURITY.md](SECURITY.md).
