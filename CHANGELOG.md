# Changelog

All notable changes to GitPoulpi are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/) (from 1.0.0 on, the CLI, config and
IPC surfaces are stable within a major version).

## [Unreleased]

### Changed
- The mouse pointer is the I-beam over the code of the Diff view (Inline and Split), the review's diff pane, the File View and Blame, since that text can be selected even though it is not editable. Line numbers, hunk buttons, scrollbars and menus keep their own pointer. One rule: wherever a drag can select text, the pointer is the I-beam.

## [1.2.0] - 2026-11-23

### Added
- **Pull all branches:** the Pull menu fetches, then brings every local branch that is behind its upstream up to date in one action (`Ctrl+G` then `Ctrl+P`, also in the palette). The checked-out branch is integrated like the Pull button, with rows for the other pull modes; the other branches are only fast-forwarded, and diverged ones are skipped and listed. When a behind branch is checked out in another worktree, GitPoulpi asks whether to update that worktree too. One undo restores every branch. CLI: `gitpoulpi pull --all-branches`.

### Changed
- A `Ctrl` key after a prefix (`Ctrl+G`, `Ctrl+N`) can now have its own binding; when it has none it still acts as the plain key. `Ctrl+G` then `Ctrl+P` is now **Pull all branches** (`Ctrl+G` then `p` stays Pull).
- A mouse click while a `Ctrl+G` or `Ctrl+N` chord waits for its second key cancels the chord, then acts as usual.

### Fixed
- Tags no longer show as being on a remote that was removed from the repository outside GitPoulpi (for example with `git remote remove`); the leftover tag record of that remote is ignored.
- Windows: a GitPoulpi pinned to the taskbar before 1.1.0 follows the octopus colour again.
- Windows: changing the octopus colour is fast again.

### Security
- The `cookie` dependency is updated to 0.18.3 (RUSTSEC-2026-0594).

## [1.1.1] - 2026-11-23

Fixes for several installations side by side on one machine, and two theme fixes.

### Fixed
- **Aliases stay with their installation:** an installation that is not the primary no longer picks up the shared `ui.command_alias` / `ui.command_alias_add` of another one, and `gitpoulpi alias create` records the name for the installation that made it. Aliases left by an older build go to the installation that claims them, never to whoever becomes the primary.
- **Primary handover without a gap:** becoming the primary or resigning renames this installation's shortcuts (Start menu, Desktop, app launcher entry) in place and moves its "Open in" folder-menu entry, so neither ever disappears. Its own command aliases are rewritten for the new role.
- **One folder-menu entry per installation (Windows):** the primary no longer shows two "Open in" entries, resigning leaves exactly one, and uninstalling the primary removes all of its entries.
- **Launcher renames:** changing the launcher name, or showing the version in it, renames the existing shortcuts and folder-menu entry in place, from Settings or `gitpoulpi installation set`.
- **Windows installer:** a package install names its shortcuts after its channel (for example `GitPoulpi (Chocolatey)`) from the start.
- **Settings > Launcher:** the primary checkbox follows a resign as soon as it finishes.
- **Themes reload live:** a running window applies a changed `config.toml` or active theme file within about a second, including `gitpoulpi config set ui.theme ...` from a shell.
- **`ui.border_focused` is used:** focused pane borders take this theme color (default: the accent), so a theme can frame the active pane in a color that would be unreadable as text.

## [1.1.0] - 2026-11-22

Several GitPoulpi installations can now live side by side on one machine (a package, a portable copy, a cargo build, a dev checkout), each with its own launchers, colour, aliases and background processes, so two versions run at once without getting in each other's way. GitPoulpi also ships as a single program.

### Highlights
- **Side-by-side installations:** every installation has its own launcher name (`GitPoulpi (Chocolatey)`, `GitPoulpi (dev)`, ...), octopus colour, command aliases, "Open in" folder-menu entry and daemons. Settings, repositories and tokens stay shared.
- **Environments:** sessions, windows, daemons and the undo history belong to an environment, by default one per installation, so two versions never take each other's windows or undo entries.
- **One program:** a single `gitpoulpi` executable is the terminal UI, the CLI and the MCP server, for smaller downloads and installs.

### Added
- **Settings > Launcher, "This installation":** the installation's kind, folder and environment, its launcher name (optionally with the version), its octopus colour, **Become the primary**, the environment, a folder-menu submenu, the other installations of the machine (with **Forget** for one whose folder is gone) and the launchers an older build left (**Remove legacy launchers**). Palette: **Launcher: Become the Primary Installation**, **Launcher: Remove Legacy Launchers**, **Launcher: Forget Gone Installations**.
- **The primary installation:** nobody is primary by default. The primary owns the plain `GitPoulpi` launcher name, the legacy shortcuts and the shared `ui.*` keys; resigning gives back the look you had before.
- **Folder menu per installation:** one "Open in" entry per installation on Windows, Linux and macOS, with the icon in the installation's colour. Installations that choose the same submenu name share it, and the last one to leave removes it.
- **Shortcuts per installation:** Windows shortcuts with their own icon and taskbar identity, Linux `.desktop` entries and macOS bundles per installation; changing the primary renames launchers in place, with no gap.
- **Command aliases per installation:** `gp` and `gpa` belong to the installation that made them (`gpd` and `gpda` are suggested for a dev checkout); creating an alias another installation owns is refused.
- **`gitpoulpi installation show` and `gitpoulpi installation set <field> <value>`:** which installation this is, its environment, its settings and the others on the machine. `GITPOULPI_INSTALL_ID` pins the identity and `GITPOULPI_ENV` forces the environment (scripts and tests).
- **Per-environment undo history:** each environment keeps its own journal and caches (hosting, AI); undoing an operation made in another environment is refused without touching the repository. Two windows of the same environment share their history.
- **MCP port check:** `gitpoulpi mcp serve --http` refuses to start on a port already in use and says so, instead of colliding with another installation's server. Settings > MCP has **Suggest a free port** (palette: **MCP: Suggest a free port**), saved per installation.
- `gitpoulpi mcp serve --bind <ADDR>`: the address for `--http` (same as `--http ADDR`).
- Hovering an author avatar in the graph shows the author.

### Changed
- **One program:** GitPoulpi ships a single `gitpoulpi` executable. Without a subcommand it opens the terminal UI; `gitpoulpi mcp serve` is the MCP server. `gitpoulpi-updater` stays a separate small program.
- `gitpoulpi-tui`, `gitpoulpi-mcp` and `gitpoulpi-launch` are now small relays that start `gitpoulpi` (existing MCP configs keep working; `gitpoulpi-mcp` stays for all of 1.x, `gitpoulpi-tui` will be removed in a later release).
- The `gp` alias and the shortcuts start `gitpoulpi`; ones written for `gitpoulpi-tui` are rewritten automatically.
- Smaller downloads and installs: the archive no longer carries three full copies of the program.
- **Updates stop only their own installation:** the updater closes the processes of the installation it updates and leaves the others running.
- **Uninstall removes only this installation:** `gitpoulpi uninstall` takes out this installation's aliases, PATH entries, shortcuts and folder-menu entries, never another's. Scoop and the MSI installer now run it on uninstall too.
- Writing installation settings keeps the formatting and comments of `config.toml`.

### Fixed
- The Safety panel's tree rows follow the folders-first setting.
- `gitpoulpi-launch --version` prints the version line.
- The palette shows every binding of the tab-switch commands, and the `Ctrl+Tab` hint names the terminals that keep that key for themselves (see Troubleshooting).
- The About window lists crate versions without build metadata.

### Upgrading from 1.0.0
- Launchers made by 1.0.0 under the plain name `GitPoulpi` are kept and listed in Settings > Launcher: choose **Become the primary** to keep that name for this installation, or **Remove legacy launchers**.
- The undo history of 1.0.0 is carried over to the installation's environment the first time it opens a repository.
- Nothing to change in your MCP client configuration: `gitpoulpi-mcp` still works.

### Dependencies
- `toml` 1.x, `mlua` 0.12, `thiserror` 2, `sha1` 0.11 and other crate updates; release workflow actions updated.

## [1.0.0] - 2026-11-18

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
