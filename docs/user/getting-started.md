**English** · [Čeština](cs/getting-started.md) · [Deutsch](de/getting-started.md) · [Español](es/getting-started.md) · [Français](fr/getting-started.md) · [Italiano](it/getting-started.md) · [日本語](ja/getting-started.md) · [한국어](ko/getting-started.md) · [Nederlands](nl/getting-started.md) · [Polski](pl/getting-started.md) · [Português (Brasil)](pt-BR/getting-started.md) · [Português (Portugal)](pt-PT/getting-started.md) · [Русский](ru/getting-started.md) · [Svenska](sv/getting-started.md) · [Türkçe](tr/getting-started.md) · [Українська](uk/getting-started.md) · [简体中文](zh-CN/getting-started.md)

# Getting started

This page takes you from a fresh install to your first commit. The
[interface tour](interface-tour.md) names every area of the window.
[Settings](settings.md) lists every option.

## 1. Install and launch

Follow [Installation](installation.md). Then open a terminal in a Git
repository and run:

```
gitpoulpi
```

GitPoulpi opens the repository
that contains the current folder. Outside a repository, it opens the **Home**
tab, where you can browse, clone or initialize a repository (see
[step 7](#7-open-clone-and-initialize-repositories)).

The icons need a Nerd Font or a font with emoji. If you see boxes, see
[Troubleshooting](troubleshooting.md).

![The main window: the Refs sidebar on the left, the Log graph in the middle and the working changes with the commit form on the right](images/hero.png)

## 2. Three keys to learn everything else

| Key | Opens |
|---|---|
| `?` | The **help**: every key and mouse gesture. Type to filter it. `Ctrl+D` opens this documentation in your browser. |
| `Ctrl+P` | The **command palette**: type to find a command, `Enter` runs it. Each row shows its shortcut. |
| `Ctrl+,` | The **Settings**. The gear in the title bar, left of the ×, opens them too. |

You can do everything with the keyboard or with the mouse. The palette reaches
every command. See [Keybindings](keybindings.md) and
[Mouse and gestures](mouse-and-gestures.md).

![The command palette filtered on the word stage, listing staging commands with their shortcuts](images/palette.png)

## 3. Look at the history

The middle of the window is the **Log**: one row per commit, newest first. It
shows the branches and tags as colored pills, the graph, the message, the
author, the date and the short hash.

- Click a commit. Its details and changed files appear on the right.
- Click a changed file. Its diff opens in the middle. `Esc` closes it.
- `j` / `k` or the arrow keys move the selection. `f` searches the commits.
- Right-click a commit for its menu.

![The Log with a merge commit selected: its message, author and the file it adds on the right](images/log-graph.png)

## 4. Make a change and stage it

Edit a file in your editor. The **top row of the Log** now shows the working
changes: a dashed row with the number of changed files. Click it. The right
pane shows **Unstaged Files** and **Staged Files**.

- Hover a file and click its **Stage file** button. Or click **Stage all
  changes**.
- Click a file name to open its diff. There you can stage one **hunk**
  (**Stage hunk**) or one **line**.

See [Staging and committing](staging-and-committing.md).

![A diff of a modified file, with Stage hunk and Discard hunk buttons above each hunk](images/diff.png)

## 5. Commit

Type a summary, and optionally a description, in the commit form at the
bottom right. Then click the green **Commit changes to N files** button or press
`Ctrl+Enter` (or `Ctrl+G Enter`, from anywhere). Check **Amend previous commit** to add your changes to the last
commit instead.

If you turned on [AI](ai.md), the `✦` button next to the summary writes a
message from the staged changes. GitPoulpi never commits until you click
**Commit changes to N files**.

## 6. Share your work

The toolbar has **Pull** and **Push** buttons, each with a `▾` list. From the
keyboard, press `Ctrl+G` and then a letter: `Ctrl+G p` pulls,
`Ctrl+G Shift+P` pushes. After `Ctrl+G`, the status line lists the letters.

GitPoulpi uses the authentication your `git` already uses (credential helper,
ssh-agent). It needs no account of its own. To see and create pull requests,
connect a hosting account once. See
[Hosting and issue trackers](hosting-integration.md).

## 7. Open, clone and initialize repositories

Click the house icon at the left end of the tab row, or press `Ctrl+O`. The
**Home** tab lists your open, favorite and recent repositories, with
**Browse**, **Clone** and **Init** buttons. Each repository opens in its own
tab. `Ctrl+W` closes the current tab.

![The Home tab listing the open and recent repositories with Browse, Clone and Init buttons](images/home.png)

**Clone** asks for a URL and a folder, or lets you pick a repository from a
connected account (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit; see
[Clone from a connected account](hosting-integration.md#clone-from-a-connected-account)).
**Init** makes a folder into a new repository.
In all three dialogs (and in the folder picker they open), `Ctrl+N` shows a badge on each field and button: press the character to put the cursor in the field or press the button.

| ![The Clone dialog: where to clone to, the URL, the folder name, and the Cancel and Clone buttons](images/clone.png) | ![The Initialize a repository dialog: the folder, the name and the initial branch](images/init.png) |
|---|---|

### Sparse checkout

A big repository does not have to land on disk in full. In the Clone dialog,
check **Sparse checkout (only some directories)** and type the directories you
want, separated by spaces or commas (`src docs/guide`). Only those directories
and the files at the repository root are checked out; the whole history is
still downloaded, so the Log, blame and branches work as usual. At least one
directory is required while the box is checked.

![The Sparse checkout dialog with two directories typed, and the Cancel, Disable and Apply buttons](images/sparse-checkout.png)

Any repository can be switched later: open the command palette
(`Ctrl+P`) and run **Repository: Sparse Checkout...**, or use **Sparse
checkout...** in the repository tab's menu. The dialog lists the directories
currently checked out in one field: add or remove names there and press
**Apply** (`Enter`), or **Disable (full checkout)** to bring every file back.
While a working tree is sparse, the status line shows a small `sparse` badge. The same controls are a page of
*Settings > Sparse checkout* (a list of paths, one per line, with **Start**, **Stop** and **Refresh** buttons).

Notes:

- It is git's cone mode (directories, not patterns); `*`, `..` and absolute
  paths are refused.
- Changing the set is not recorded by Undo: nothing is lost, and **Disable**
  restores every file. Git can refuse to drop a directory that holds changes
  you have not committed (the error shows on the status line); commit or
  stash them first.
- The same thing from scripts and agents: `gitpoulpi clone <url> --sparse src
  docs` and `gitpoulpi sparse-checkout status | set | add | disable`
  (see the [CLI reference](cli-reference.md)); the MCP server has the same
  tools.

## Where next

- [Tour of the interface](interface-tour.md): the name of everything you see.
- [Working with the Log](log-and-history.md) and
  [Staging and committing](staging-and-committing.md): daily work.
- [Themes](themes.md) and [Settings](settings.md).
- [AI assistance](ai.md) and [Hosting and issue trackers](hosting-integration.md):
  the two optional integrations.
- [CLI reference](cli-reference.md) and [MCP and agents](mcp-and-agents.md):
  the same operations from scripts and agents.
- [Troubleshooting](troubleshooting.md).
