**English** · [Čeština](cs/README.md) · [Deutsch](de/README.md) · [Español](es/README.md) · [Français](fr/README.md) · [Italiano](it/README.md) · [日本語](ja/README.md) · [한국어](ko/README.md) · [Nederlands](nl/README.md) · [Polski](pl/README.md) · [Português (Brasil)](pt-BR/README.md) · [Português (Portugal)](pt-PT/README.md) · [Русский](ru/README.md) · [Svenska](sv/README.md) · [Türkçe](tr/README.md) · [Українська](uk/README.md) · [简体中文](zh-CN/README.md)

# GitPoulpi user documentation

<p align="center"><img src="../../assets/logo/gitpoulpi-512.png" alt="The GitPoulpi octopus" width="128"></p>

GitPoulpi is a Git client for the terminal with a clickable commit graph, a
command palette, hosting and issue-tracker integration, optional AI help, and a
CLI and MCP server that do everything the UI does. These pages are in reading order:
start at the top, and jump to the reference pages when you need them.

![The GitPoulpi main window](images/hero.png)

## Start here

1. **[Installation](installation.md)**: what you need, release archives,
   package managers, building from source. For a smaller binary, see
   **[Custom builds](custom-builds.md)**. How GitPoulpi keeps itself current
   (the version chip, the update panel, revert): **[Updates](updates.md)**.
   To run it in a container: **[Running in Docker](docker.md)**. For
   administrators, internal feeds, mirrors and the update policy:
   **[Enterprise distribution](enterprise-distribution.md)**.
2. **[Getting started](getting-started.md)**: your first five minutes, from
   launching to your first commit.
3. **[Tour of the interface](interface-tour.md)**: the name of every area of the
   window, with pictures.

## Everyday use

4. **[Working with the Log](log-and-history.md)**: reading the graph, search,
   branches, tags, stashes, worktrees, dragging refs, rebase, cherry-pick,
   reword and amend, banners, pull and push, auto-stash.
5. **[Staging and committing](staging-and-committing.md)**: the working changes,
   diffs, staging hunks and single lines, writing commits, the commit composer,
   blame, file history and the built-in editor.
6. **[Pictures](image-view.md)**: images in the Diff view and the File View,
   before and after, zoom, Edit in.
7. **[Resolving merge conflicts](merge-conflicts.md)**: the merge editor (A and B
   side by side over the Output), picking lines and chunks, cards for binary,
   LFS, deleted, symlink and submodule conflicts, and how an agent proposes a
   resolution.
8. **[Undo and redo](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, what can and cannot
   be undone, when it refuses or asks, how far back it goes, turning it off.
9. **[Git LFS](lfs.md)**: large files, tracking, locks, and what happens without
   `git-lfs` installed.
10. **[Submodules](submodules.md)**: the SUBMODULES section, updating them, and
    keeping them in step with checkouts and pulls.
11. **[Themes](themes.md)**: the built-in themes, any theme with any lane
    palette, live preview, your own theme files, `gitpoulpi theme`.
12. **[Workspaces](workspaces.md)**: named groups of repositories with their own
    tabs and an optional default commit profile, the tab-bar chip, the Workspaces tab, Fetch all, Pull all, Clone missing.
13. **[Search](search.md)**: one fuzzy finder (`Ctrl+Alt+F`) over the branches,
    tags, worktrees, stashes, commits, pull/merge requests and issues of every
    repository of the workspace or the open tabs.
14. **[The Radar](radar.md)**: the pull/merge requests, issues and work in
    progress of a workspace (or your open tabs) in one list: filters, pins,
    snoozes, and what each row can do.
    Reading and reviewing one of those requests in the app, with comment
    threads, a pending review and Approve / Request changes:
    **[Reviewing a pull request](review.md)**.
15. **[Open in](open-in.md)**: the title bar's list that opens the active tab's
    folder in a terminal, multiplexer, editor or file manager.
16. **[Following the active tab's folder](following-the-tab-folder.md)**:
    "Quit and cd here", and the `gitpoulpi shell-init` wrapper (`gp`) for bash,
    zsh, fish and PowerShell.
    Which terminal shows GitPoulpi best: **[Choosing a terminal](terminals.md)**.
    What passes through tmux, zellij, screen, herdr and terminal panes:
    **[Multiplexers](multiplexers.md)**.

Also: **[Worktrees](worktrees.md)**: creating, opening, merging and removing linked worktrees.

Also: **[Several remotes for one branch](multiple-remotes.md)**: push targets and pull
sources, the Remotes dialog, Push to all or to selected remotes, resolving a remote that
moved, Continue after a conflict, the CLI and a plain-git alias.

**[Branching workflows](branching-workflows.md)**, the Git Flow, GitHub
Flow, GitLab Flow and Trunk-based presets or your own bases, branch name patterns
and topic types (`gitpoulpi flow ...`).

**[History rewrite](history-rewrite.md)**: the workbench (`Ctrl+G Shift+H`) that rewrites
authors, dates, messages and file contents across many commits, with a preview, Lua
scripts and one undo.

## Keyboard and mouse

17. **[Keybindings](keybindings.md)**: every key, by area, and every palette
    command. In the app, `?` shows the same list and `Ctrl+P` finds any
    command.
18. **[Mouse and gestures](mouse-and-gestures.md)**: clicking, dragging,
    selecting and copying, scrollbars and hover feedback.

## Settings and integrations

19. **[Settings](settings.md)**: every page and option of the Settings screen,
    the search box, profiles, and how to rebind shortcuts.
20. **[AI assistance](ai.md)**: GitPoulpi's own AI (off by default): a local
    agent such as `claude -p`, Anthropic, OpenAI, OpenRouter, Ollama or
    LM Studio; instructions; commit messages, explanations, pull request
    descriptions and the commit composer; privacy and troubleshooting.
21. **[Hosting and issue trackers](hosting-integration.md)**: GitHub, GitLab, Gitea and Forgejo,
    Azure DevOps, Bitbucket, Gerrit and the issue trackers:
    tokens and their scopes,
    connecting, linking a repository, pull requests and issues, references in
    commit messages, SSH keys, known limits. Also
    **[Local issues and Obsidian](local-issues.md)**: a folder of Markdown
    notes as the issue source, no token, with an Obsidian Base.

## Automation

22. **[CLI reference](cli-reference.md)**: every `gitpoulpi` command, with
    `--json` for scripts.
23. **[MCP and agents](mcp-and-agents.md)**: let an external agent (Claude Code,
    Cursor, Codex, ...) drive GitPoulpi: registering the server, `.mcp.json`,
    the permission switches and their defaults, every tool, and how an agent
    shows you things in your window. The ready-made agent skill is
    [`skills/gitpoulpi/SKILL.md`](../../skills/gitpoulpi/SKILL.md).

## Reference and help

24. **[Configuration reference](config-reference.md)**: every `config.toml` key,
    its default and effect, the per-repository settings, environment variables.
25. **[Troubleshooting and FAQ](troubleshooting.md)**: icons, graph rendering,
    keys that do not arrive, stuck daemons, speed, LFS pointers.

## Screenshots

Every picture in these pages is a real capture of a release build, on a small
demo repository with made-up authors. The pictures are in [`images/`](images/).
`scripts/docs-screenshots.sh` builds the demo repositories and takes them again.
They are taken in the `kitty` terminal, which draws the commit graph and the
author avatars as pictures. Terminals without a graphics protocol show a
character graph instead.

## For contributors

[CLAUDE.md](../../CLAUDE.md), the [architecture overview](../architecture/overview.md)
and the [roadmap](../roadmap.md) describe the design and what is planned;
[CHANGELOG.md](../../CHANGELOG.md) lists what changed in each release.

---

GitKraken is a trademark of its respective owner. GitPoulpi is an independent project, not affiliated with, endorsed or sponsored by it; the name is mentioned only to describe compatibility and inspiration.
