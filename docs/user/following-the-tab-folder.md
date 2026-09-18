**English** · [Čeština](cs/following-the-tab-folder.md) · [Deutsch](de/following-the-tab-folder.md) · [Español](es/following-the-tab-folder.md) · [Français](fr/following-the-tab-folder.md) · [Italiano](it/following-the-tab-folder.md) · [日本語](ja/following-the-tab-folder.md) · [한국어](ko/following-the-tab-folder.md) · [Nederlands](nl/following-the-tab-folder.md) · [Polski](pl/following-the-tab-folder.md) · [Português (Brasil)](pt-BR/following-the-tab-folder.md) · [Português (Portugal)](pt-PT/following-the-tab-folder.md) · [Русский](ru/following-the-tab-folder.md) · [Svenska](sv/following-the-tab-folder.md) · [Türkçe](tr/following-the-tab-folder.md) · [Українська](uk/following-the-tab-folder.md) · [简体中文](zh-CN/following-the-tab-folder.md)

# Following the active tab's folder

GitPoulpi can have several repositories open as tabs. The shell you started it
from stays in its folder: a program cannot change the folder of its parent
shell. GitPoulpi offers two ways to get a shell in the active tab's folder:

- **Quit & cd here**: GitPoulpi quits, and your shell changes to the folder of
  the active tab. This needs a small shell function, printed by
  `gitpoulpi shell-init`.
- **Open in**: opens the folder of the active tab in a terminal, a multiplexer
  pane, an editor or the file manager. No shell setup is needed. See
  [Open in](open-in.md).

```
┌🐙 GitPoulpi vX.Y.Z -- /work/other-repo - Open in ▾ ── ⌕ Global search ⚙ × ┐
│ ⌂  main-repo x  other-repo x                   Quit & cd here           │
```

- **Open in** is in the title bar, after the repository path. It is hidden
  when the window is too narrow.
- **Quit & cd here** is at the right end of the tab bar. It shows only when
  both are true:
  - GitPoulpi was started through the shell function (below).
  - The active tab is not the folder you started GitPoulpi in.
- The commands are always in the palette (`Ctrl+P`): `Tab: Quit and cd here`,
  `Tab: Open in…` (`Alt+O`) and `Tab: Open in default` (`Ctrl+Alt+O`).
  Without the shell function, `Tab: Quit and cd here` tells you how to set it
  up.

A normal quit (`q`) never changes the folder of your shell. Only
**Quit & cd here** does, unless you choose `auto` (below).

## Setting up the shell function

`gitpoulpi shell-init <shell>` prints a function named `gp`. Use `--name` to
choose another name. The function runs `gitpoulpi --cwd-file <temp file> ...`.
When you quit with **Quit & cd here**, GitPoulpi writes the folder into that
file, and the function changes to that folder. GitPoulpi does not install or
edit anything: you add one line to the start-up file of your shell.

| Shell | Add this to | Line |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(gitpoulpi shell-init bash)"` |
| zsh | `~/.zshrc` | `eval "$(gitpoulpi shell-init zsh)"` |
| fish | `~/.config/fish/config.fish` | `gitpoulpi shell-init fish \| source` |
| PowerShell | your `$PROFILE` (`notepad $PROFILE`) | `gitpoulpi shell-init pwsh \| Out-String \| Invoke-Expression` |

Then start GitPoulpi with `gp` instead of `gitpoulpi`. Arguments are passed on
(`gp -s work`), and the exit status is the exit status of `gitpoulpi`.

**An existing `gp` is never replaced without a warning.** If your shell already
has a command, alias or function named `gp`, the script prints a warning and
defines nothing. Choose another name, for example
`gitpoulpi shell-init bash --name gpl`. In PowerShell, the function replaces
the built-in `gp` alias (`Get-ItemProperty`) and an alias that points at
`gitpoulpi`. Any other `gp` is kept, with a warning.

Every `gitpoulpi` command accepts `--cwd-file`, but only the TUI writes the
file.

## Setting: Follow the active tab's folder

Settings > Open in > **Follow the active tab's folder** (`ui.tab_dir_follow`):

| Value | Behavior |
|---|---|
| `buttons` (default) | **Quit & cd here** and **Open in** as described above. |
| `auto` | No **Quit & cd here** button. **Open in** stays. Every quit changes your shell to the active tab's folder (when you started GitPoulpi with the shell function). The default **Open in** entry runs each time you open a new tab. |

In `auto` mode, the default entry runs only for tabs that you open yourself:
from the Home tab, a worktree, a clone or an init. It does not run for the
tabs restored at start-up, for the first tab, or when you switch to a tab that
is already open.

The older settings `ui.terminal_launch` and `ui.terminal_launch_command` are
moved into the **Open in** list. See
[Open in](open-in.md#legacy-keys).
