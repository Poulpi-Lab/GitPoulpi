# Changelog

All notable changes to GitPoulpi are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/) (0.x: the CLI, config and IPC
surfaces may still change between minor versions).

## [Unreleased]

### Added
- Log columns: a **Scroll sideways** layout (`ui.log_columns_layout = "scroll"`; the column chooser, Settings > Log & graph, the palette, or `s` while a column is picked). Columns keep their widths (Message too), nothing is given up in a narrow window, and the table scrolls sideways with `Shift+Left/Right`, the horizontal wheel or Shift+wheel, and a bar on the bottom border. Each layout keeps its own widths (`ui.log_column_widths_scroll`).
- Log columns: double-click a column's boundary to fit the column on its left to its content (keyboard: `=` on the picked column, `+` for all; palette and chooser rows too). The last column has a resize grip on its own last cell, and the SHA column is one cell wider by default so the grip never covers a digit.

### Changed
- The Radar opens with `Alt+R` by default (was `Alt+L`). Inside the merge editor and the commit message composer `Alt+R` keeps its own action (reset all, insert issue reference); the Radar stays reachable through the palette there. A custom `global.radar` binding is kept.

## [1.0.0] - 2026-10-29

The first stable release. It builds on the 0.1.0 release candidate (kept below) and adds history rewriting, branching workflows, workspaces, reviews, a safety net, many more hosts and 17 languages. The CLI, config and IPC surfaces are now stable.

### Highlights
- **History rewrite workbench:** change authors, dates, messages, files, signatures and LFS usage across a whole history, with a preview first, one undoable Apply, and a separate push and purge.
- **Branching workflows:** Git Flow made customizable, with presets, detection and a `Flow` menu to start, finish, sync and promote branches.
- **Workspaces and the Radar:** group repositories, and see what waits for you across all of them.
- **Reviews without leaving the window:** read, comment and approve pull requests, merge requests and Gerrit changes.
- **You can always go back:** undo and redo for destructive actions, plus a safety net of past branch positions and lost commits.
- **Failures explained:** a failed git operation says what went wrong and what to do next.
- **17 languages**, a dozen hosting and issue integrations, and an installer and self-update.
- **Breaking (safety):** `[mcp] allow_mutations` now defaults to `false`. AI agents are read-only until you set it to `true`.

### Log & history
- Split a commit into several, from the Log menu, the palette or the interactive rebase popup, with an optional AI grouping.
- Amend a commit's message, author and dates; revert a merge commit with a mainline parent; copy commits as a Mermaid gitGraph.
- The Reflog view loads more positions on demand.
- A comparison panel lists the selected rows as cards.
- The Log's Ref, Graph and Author columns keep their width when the window is resized; only Message flexes.
- The Tags section shows tags in folders split on `/`, and a Refs filter opens the folders that hold a match.
- A faded ghost branch label on a hovered or selected commit with no ref creates a branch there on click (`ui.graph_ghost_ref`).
- The commit graph is drawn with sixel pictures, avatars included, in Windows Terminal, foot and other sixel terminals.
- The status bar shows `loaded / total` commits; `gitpoulpi log --count` prints the total.
- The Log honours `.mailmap`, with an Identities tab to merge authors (`log.use_mailmap`).
- Right-click a header in the left pane to show or hide sections.

### Commit column & file views
- Beside the Diff, File View, Blame and Edit views, a compact list of the Log's commits lets you move a file view to another commit. Shown from 140 terminal columns; `Alt+C` hides and shows it, and its width is remembered.
- A fixed-width `File | All` selector chooses the file's history or the whole Log. Rows of the All list have the Log's tooltips and right-click menu.
- A header over the rows sizes the lanes, author and date by dragging; rows scroll sideways when they do not fit.
- File history opens as a split view with the commit's diff on the right.

### Commits & staging
- `Ctrl+G Enter` commits the staged changes from anywhere.
- Repository settings pages for Sparse checkout, LFS, Commit (`--no-verify`), Merge options and Message template.
- Conflict prevention: a strip on the Log warns before you merge when the branch would conflict with the branch it started from, with **Open merge preview** and **Rebase onto base**. `gitpoulpi conflicts check` prints the same answer.
- A merge conflict editor with a three-way view, output editing, find, and support for non-text conflicts.
- Git LFS from the toolbar: install, track, checkout, locks, and pre-flight checks before pull and push.
- Submodules in the Refs sidebar: list, init, update, and keep them in step with the commit you check out.
- `core.autocrlf` and `core.eol` write the repository's git config, undoably.

### Branching & workflows
- A branching workflow describes base branches, topic types, name patterns and parents. Presets, detection and import are included.
- The `Flow` menu starts, finishes, syncs and promotes branches; the sidebar groups branches by workflow.
- The same actions are available through the CLI and MCP.

### History rewrite
- A recipe is a scope plus ordered rules: replace or anonymise authors, apply `.mailmap`, shift dates, change time zones, edit messages and trailers, search and replace in files, remove files or large files, move paths, convert line endings and encodings, migrate to or from LFS, remove signatures.
- Preview shows counts, what each rule did and which branches and tags would move. Apply is one undo entry. Push (force with lease) and Purge are separate steps.
- Script rules in Lua, shell or Python, trusted by content hash.
- A secret scanner finds tokens and passwords in history.
- Create rules from a description with AI, or review a recipe proposed by an agent. Agents can preview but never apply on their own.
- Available as `gitpoulpi rewrite` and as MCP tools, running the same engine as the workbench.

### Undo & safety net
- Undo and redo (`Ctrl+Z`, `Ctrl+Y`) cover discards, deletes, patches, staging and history operations, with a dated Undo history.
- The safety net keeps every position a branch has had, including moves made by other programs, with restore of a single branch and a list of lost commits.
- A failed git operation shows a card with the reason and a next step (Pull first, Stash and retry, Retry, Open Integrations), a Details view and Copy for pasting to an AI. The CLI and MCP error results carry the same facts under `detail`.
- Network git operations can be cancelled and clean up after themselves; stuck states such as a stale `index.lock` are detected and offered a fix.
- Tokens and passwords are masked everywhere.

### Workspaces, Home & tabs
- Workspaces group repositories, remember which are open and can carry a default commit profile (`gitpoulpi workspace`).
- Home is one list: Favorites, Open, Recent and All, with one filter over all of it.
- The Radar is a tab of its own listing your pull requests, review requests, assigned issues and unfinished work across repositories. It can be turned off.
- A global **Search** button finds commits, files, branches and more, with prefixes to narrow it.
- Drag a repository tab to reorder it; worktree tabs sit right after their repository.
- `gp -a` (or `gitpoulpi --add [path]`) opens a folder as a new tab of the GitPoulpi that is already running.

### Hosting & issues
- Code hosts: GitHub, GitLab (including self-managed), Gitea, Forgejo (including Codeberg), Azure DevOps, Bitbucket Cloud and Data Center, and Gerrit.
- Issue trackers: Jira, Trello, Linear, YouTrack, Redmine, and local Markdown notes (Obsidian-friendly), with an issue board.
- Review a pull request, merge request or Gerrit change in the repository's window: description, conversation, files with comment threads, commits, comments, approval and requested changes.
- Create pull requests and issues with assignees, reviewers, labels and milestones.
- The toolbar chip of a pull request is hidden once it is closed or merged (`ui.hide_finished_pr_chip`).
- Clone from a connected account in the background; the new tab opens when it is done.

### AI
- Still off by default and never needs an account.
- Propose a split of your working changes or of an existing commit, write history rewrite rules and scripts, and explain changes in a read-only popup.
- Profiles with their own environment variables, edited inline.
- AI can be left out entirely of a custom build.

### CLI & MCP
- Every new feature has a CLI command and an MCP tool, including `rewrite`, `workspace`, `conflicts`, `job`, `amend`, `split-commit`, `convert` and `licenses`.
- MCP agents can ask the window to show a diff at a line, a file at a position, or a highlight.
- `gitpoulpi alias` and `gitpoulpi path` create command aliases such as `gp` and put GitPoulpi on your PATH. They are not available to MCP agents.
- Background jobs (fetch, push, pull, clone, LFS transfers, AI requests) are listed and cancellable with `gitpoulpi job list | cancel | wait`.

### Interface
- Settings: one value column per page, inline action buttons, master-detail lists with reordering, sub-tabs, search and a regrouped page order.
- Every palette command can get its own shortcut; the Keyboard & mouse page captures keys, resets and applies live.
- One way to close: every framed window and dialog has a `x`, and an outside click closes popups and panels.
- Themes: GitPoulpi Dark and Light, GitHub and VS Code presets, imports, any lane palette with any theme, live preview and your own themes.
- Dates use month and weekday names in the interface language (`Date/Time Locale = english` opts out).
- Image view with an image editors list; Open in for files, folders and terminals.
- The Activity Log (status bar, `View: Activity Log`) lists the session's messages and the application log, with a level box, a Global tab, retention up to a year and **Clear log**.
- **Licenses and credits** in About and `gitpoulpi licenses`.

### Languages
- 17 interface languages: Chinese (Simplified), Czech, Dutch, English, French, German, Italian, Japanese, Korean, Polish, Portuguese (Brazil and Portugal), Russian, Spanish, Swedish, Turkish and Ukrainian.
- Git words can stay in English (`ui.git_terms_english`), and the user guide is translated too.

### Platforms & installation
- Windows: an Inno Setup `...-setup.exe` (per user, with Start menu, Desktop, PATH and alias choices); the taskbar shows GitPoulpi's own icon.
- Linux and macOS: `install.sh` with `--alias`, `--no-path`, `--desktop`, `--yes`, `--archive` and `--uninstall`.
- `gitpoulpi uninstall` removes aliases, PATH entries, shortcuts and, for a release install, the files.
- Launcher shortcuts bring an already open GitPoulpi forward (Windows, WezTerm, Hyprland) instead of piling up terminals (`ui.reuse_window`).
- Self-update: **Update now** downloads the archive in the background, checks its `.sha256` and stages it; **Restart to update** swaps it in; **Revert** goes back. `updates.mode` is `notify`, `auto` or `off`, and the status bar shows the running version.
- Custom builds choose languages, integrations, features and themes (`xtask build-custom`). Radar, mermaid, board, review and AI are optional features.

### Performance
- Background jobs keep the interface responsive, and transfers can be cancelled.
- Spawn and content budgets are guarded by `xtask perf`, including a rewrite budget.
- The Log's commit count can use system git or libgit2 (`log.commit_count`).

## [0.1.0] - Release candidate

First public release candidate, version 0.1.0.

### Documentation
- README with screenshots, and a complete user guide in `docs/user/` (index,
  getting started, interface tour, Log, staging, Settings, AI, GitHub/GitLab,
  CLI, MCP, troubleshooting) with real captures of the release build.

### Log & graph
- Commit graph drawn as real pixels (Kitty and iTerm2 graphics protocols, with
  a character fallback), loaded lazily so large histories open fast.
- Author avatars (identicon fallback, Gravatar, GitHub and GitLab sources).
- Ref pills with inline branch and tag naming, drag and drop of refs onto
  commits with a menu, a merged ref and commit context menu.
- Multi-select, interactive rebase (reorder, squash, fixup, reword, edit,
  autosquash) and multi-commit cherry-pick in a review view.
- Stashes shown in the graph, hidden-refs counter with Show All, solo and hide
  per folder or remote, resizable Refs sidebar sections.
- Optional Changes column (files, +N -M per commit), search and filter.
- Reword any commit, fast-forward, worktrees, create and apply patches.

### Staging & WIP
- The working changes are one row in the Log, with Unstaged and Staged panes,
  per-file, per-folder and Stage All actions.
- Stage, unstage and discard by file, folder, hunk or single line, directly
  from the diff; optimistic updates with a "Staging..." indicator.
- Diff view: inline or split, hunks-only or whole file, minimap, selectable
  and copyable text, scrollbars with an overview ruler.
- File View, Blame and History in the centre pane; a built-in editor with
  syntax highlighting (Edit in Working Directory).
- Commit composer, file context menus with external-tool settings.
- Git LFS: pointer awareness, install, track and untrack, fetch, pull, push
  and clone with progress, file locks, and a banner when `git-lfs` is missing.

### Hosting
- GitHub and GitLab (including self-managed) integrations with per-profile
  tokens and SSH keys, and a per-repository SSH key pin.
- Clone from a connected account's repositories, with a folder-name preview.
- Pull and merge requests: sidebar groups with search and filter, linked-PR
  chip in the toolbar, Create Pull Request form (assignees, reviewers,
  fork-aware), Edit on GitHub or GitLab; issues from a per-repository source.
- PR or MR wording follows a setting.

### AI
- Off by default and never needs an account: OpenAI, OpenRouter,
  OpenAI-compatible endpoints, Anthropic, or a local agent CLI such as
  `claude -p`.
- Generate commit messages, explain changes, write stash messages and pull
  request descriptions, and propose a split of working changes into several
  commits (commit composer).
- Global, per-action and per-repository instructions and profiles.

### Settings
- Preferences and Integrations pages with search, real dropdowns, remembered
  section, and an Editor page (tab size, EOL, syntax, line numbers, wrap).
- Configurable columns, input colour, date formats, pull mode, push
  behaviour, auto-prune and remember-tabs.
- Repository management tab with favourites and recent repositories; tab
  context menus; Ctrl+N navigation prefix and a command palette that reaches
  every command.

### Performance
- Graph and refs paint first on open, hosting loads after the first paint.
- Native libgit2 staging and discard, scoped refreshes, Log and Refs caches,
  one git resolution at startup, spawn budgets guarded by `xtask perf`.
- Hunks load per file on demand; transfers share one fetch with the tag mirror.

### CLI & MCP
- `gitpoulpi` covers every operation of the UI (status, log, diff, commit,
  reword, branch, tag, stash, remote, fetch, pull, push, rebase, cherry-pick,
  worktree, patch, LFS, hosting, AI) with `--json` output.
- One background daemon per named session: changes made through the CLI or an
  agent show up live in the open TUI.
- `gitpoulpi-mcp` serves one MCP tool per CLI command over stdio, gated by the
  `[mcp]` table of `config.toml`.
- All three binaries report the same `--version`, including the build sha.
- `cargo run -p xtask -- dist` builds the release archive and its checksum.
