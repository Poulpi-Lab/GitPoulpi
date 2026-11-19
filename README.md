**English** · [Čeština](docs/readme/README.cs.md) · [Deutsch](docs/readme/README.de.md) · [Español](docs/readme/README.es.md) · [Français](docs/readme/README.fr.md) · [Italiano](docs/readme/README.it.md) · [日本語](docs/readme/README.ja.md) · [한국어](docs/readme/README.ko.md) · [Nederlands](docs/readme/README.nl.md) · [Polski](docs/readme/README.pl.md) · [Português (Brasil)](docs/readme/README.pt-BR.md) · [Português (Portugal)](docs/readme/README.pt-PT.md) · [Русский](docs/readme/README.ru.md) · [Svenska](docs/readme/README.sv.md) · [Türkçe](docs/readme/README.tr.md) · [Українська](docs/readme/README.uk.md) · [简体中文](docs/readme/README.zh-CN.md)

# GitPoulpi

<p align="center"><img src="assets/logo/gitpoulpi-512.png" alt="The GitPoulpi octopus" width="128"></p>

**A Git client for the terminal that feels like a desktop app.** GitPoulpi
gives you a full commit graph you can click, drag and right-click, a
keyboard-first command palette that reaches every feature, integration with
code hosts (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, in
the cloud or self-hosted) and issue trackers (Jira, Trello, Linear, YouTrack,
Redmine, or a folder of Markdown notes), and the same operations as a
scriptable CLI and an MCP server, so an AI agent can drive it exactly like you
do. GitPoulpi is inspired by GitKraken, rewritten from scratch in Rust: native binaries, light on disk and in memory, no Electron, no account, no telemetry. It runs in your favorite terminal ([which one?](docs/user/terminals.md), [inside tmux or zellij?](docs/user/multiplexers.md)), because some of us live in there.

![GitPoulpi showing the commit graph with branches, tags and a stash, the Refs sidebar on the left, and the working changes with the commit form on the right](docs/user/images/hero.png)

- **Mouse and keyboard, equally.** Click, double-click, right-click and drag
  and drop refs onto commits; or press `Ctrl+P` and type what you want. Every
  mouse gesture has a keyboard twin, and every command is in the palette.
- **A real commit graph.** Branches, merges, tags, stashes and the working
  copy in one scrolling Log, drawn as pixels in terminals that support it and
  as characters everywhere else. Many dark and light color themes, author
  avatars, search.
- **Stage precisely.** Stage, unstage or discard a file, a folder, a hunk or a
  single line straight from the diff. Inline or side-by-side, with a minimap.
- **Edit history safely.** Reword or amend any commit (message, author,
  dates), interactively rebase with a review screen, rewrite many commits at
  once with a preview and one undo, cherry-pick several commits, fast-forward,
  create worktrees and patches.
- **Hosting and issue trackers built in.** Browse, check out and create pull
  and merge requests (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket)
  and review Gerrit changes. Browse and create issues, work items and cards
  (also Jira, Trello, Linear, YouTrack, Redmine and local Markdown notes), on a
  Kanban board if you like. Clone from your account and manage SSH keys.
- **CLI and MCP for agents.** `gitpoulpi <command> --json` covers everything
  the UI does, and `gitpoulpi-mcp` serves it as MCP tools. One background
  daemon per repository keeps every frontend live and in sync.
- **Optional AI, never required.** Off by default and no account needed: bring
  your own API key or a local agent CLI such as `claude -p`. Generate commit
  messages, explain changes, write pull request descriptions, split your
  working changes into several commits and draft history rewrite rules.
- **Git LFS, submodules, worktrees, patches, blame and file history, image
  diffs, a built-in editor.**

## Screenshots

| | |
|---|---|
| ![The Log graph with the Material color theme and row tint enabled](docs/user/images/theme-material.png) **The commit graph**, here in the Material theme with row tint. | ![A diff of a modified file with Stage Hunk and Discard Hunk buttons on each hunk](docs/user/images/diff.png) **The diff**, with stage and discard per hunk or per line. |
| ![The command palette filtered on the word stash](docs/user/images/palette.png) **The command palette** (`Ctrl+P`) reaches every command. | ![The commit composer proposing four commits for the working changes, ready to review](docs/user/images/composer.png) **The commit composer** reviews AI-proposed commits before anything is committed. |
| ![The right-click menu of a commit with cherry-pick, revert, reset, rebase, branch and tag actions](docs/user/images/context-menu.png) **Right-click** anything for its actions. | ![The AI page of the Settings with profiles, per-action instructions and the enable switch](docs/user/images/settings-ai.png) **Settings**, with search, including the optional AI page. |

More pictures, with explanations, are in the [user documentation](docs/user/README.md).

## Install

You need a system `git`; `git-lfs` is optional.

### Linux and macOS (`install.sh`)

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

### Homebrew (macOS, Linux)

```
brew install Poulpi-Lab/gitpoulpi/gitpoulpi
```

### apt (Debian, Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

### Scoop (Windows)

```
scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi
scoop install gitpoulpi
```

### Windows installer

Download `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` from the
[latest release](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest) and
run it. For administrators, the MSI:

```
msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn
```

### Docker

```
docker pull poulpilab/gitpoulpi
```

See [Running in Docker](docs/user/docker.md).

### cargo

```
cargo binstall gitpoulpi-cli          # prebuilt
cargo install --locked gitpoulpi-cli  # compiles
```

### Release archive

Download it from the
[latest release](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest),
unpack it and follow `INSTALL.txt` inside.

### From source

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Your own release archive, a smaller custom build and per-OS notes:
[docs/user/installation.md](docs/user/installation.md) and
[docs/user/custom-builds.md](docs/user/custom-builds.md).

## Quick start

```
cd path/to/your/repository
gitpoulpi
```

- `?` opens the help with every key and gesture, `Ctrl+P` the command palette,
  `Ctrl+,` the Settings, `q` quits.
- Click a commit to see its changes; click the top row of the Log (the working
  changes) to stage files and commit.
- In a terminal that supports it (kitty, Ghostty, WezTerm, Rio, iTerm2, and the sixel
  terminals Windows Terminal, foot and Tabby) the graph is drawn as pixels. In others
  (a plain Windows console, zellij, screen) it falls back to characters automatically.

The step-by-step version is in [Getting started](docs/user/getting-started.md),
and [the interface tour](docs/user/interface-tour.md) names every area of the
screen.

```
gitpoulpi --json status            # the CLI, scriptable
gitpoulpi --json log --limit 20
claude mcp add gitpoulpi -- gitpoulpi-mcp   # give an AI agent the same tools
```

AI is optional and off by default: turn on GitPoulpi's own AI in *Settings > AI*
([AI assistance](docs/user/ai.md)); let an external agent drive GitPoulpi with
[MCP and agents](docs/user/mcp-and-agents.md) (permissions, every tool) and the
agent skill [`skills/gitpoulpi/SKILL.md`](skills/gitpoulpi/SKILL.md).

## Documentation

The complete user documentation starts at **[docs/user/README.md](docs/user/README.md)**:

| | |
|---|---|
| [Getting started](docs/user/getting-started.md) | Install, open a repository, make a first commit |
| [Tour of the interface](docs/user/interface-tour.md) | Every area of the window, named |
| [Workspaces](docs/user/workspaces.md) | Named groups of repositories, their own tabs, a default commit profile |
| [Working with the Log](docs/user/log-and-history.md) | Graph, refs, branches, tags, stashes, rebase, cherry-pick |
| [Staging and committing](docs/user/staging-and-committing.md) | Status, diff, hunks and lines, the composer, the editor |
| [Resolving merge conflicts](docs/user/merge-conflicts.md) | The merge editor, picking lines and chunks, binary and submodule conflicts |
| [Worktrees](docs/user/worktrees.md) | Several branches checked out at once, in their own folders |
| [Submodules](docs/user/submodules.md) | The SUBMODULES section, updating, keeping in step with checkouts |
| [Reviewing a pull request](docs/user/review.md) | Read, comment and approve a pull or merge request without leaving the window |
| [Pull requests and issues](docs/user/pull-requests-and-issues.md) | The sidebar lists, the issue board, creating a request or an issue |
| [Conversations](docs/user/conversations.md) | Read and answer the discussion of an issue or request in the app |
| [Settings](docs/user/settings.md) | Every Settings page and option |
| [AI assistance](docs/user/ai.md) | Configure the built-in AI, every AI action, privacy |
| [Hosting and issue trackers](docs/user/hosting-integration.md) | GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine: tokens, linking a repository, pull and merge requests |
| [Local issues and Obsidian](docs/user/local-issues.md) | A folder of Markdown notes (or an Obsidian vault folder) as the issue source: no token, kanban Base |
| [CLI](docs/user/cli-reference.md) and [MCP](docs/user/mcp-and-agents.md) | Script it, or let an agent drive it (permissions, every tool) |
| [Keybindings](docs/user/keybindings.md) (and every palette command), [mouse](docs/user/mouse-and-gestures.md), [configuration](docs/user/config-reference.md), [LFS](docs/user/lfs.md), [undo](docs/user/undo-redo.md), [troubleshooting](docs/user/troubleshooting.md) | Reference and help |

Developers: [CLAUDE.md](CLAUDE.md) and `docs/architecture/overview.md` describe
the crates and the design; [CHANGELOG.md](CHANGELOG.md) lists what is in each
release.

## License

[MIT](LICENSE).

## Credits and third-party licenses

GitPoulpi is built on open-source libraries and uses color values from published themes. Their licenses and credits, theme authors included, are in [THIRD-PARTY-LICENSES.txt](THIRD-PARTY-LICENSES.txt). The file ships with every release; the About window and `gitpoulpi licenses` show it too.

## Trademarks

GitKraken is a trademark of its respective owner. GitPoulpi is an independent project, not affiliated with, endorsed or sponsored by it; the name is mentioned only to describe compatibility and inspiration.

All other product and service names mentioned in GitPoulpi and its documentation are trademarks of their respective owners, for example code hosts (GitHub, GitLab, Bitbucket, etc.), issue trackers (Jira, Trello, Linear, etc.) and the editors and tools GitPoulpi can open (Visual Studio Code, JetBrains IDEs, lazygit, etc.). They are used only to name what GitPoulpi works with; GitPoulpi is not affiliated with, endorsed or sponsored by any of them.
