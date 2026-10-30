[English](../following-the-tab-folder.md) · [Čeština](../cs/following-the-tab-folder.md) · [Deutsch](../de/following-the-tab-folder.md) · [Español](../es/following-the-tab-folder.md) · [Français](../fr/following-the-tab-folder.md) · [Italiano](../it/following-the-tab-folder.md) · [日本語](../ja/following-the-tab-folder.md) · [한국어](../ko/following-the-tab-folder.md) · [Nederlands](../nl/following-the-tab-folder.md) · [Polski](../pl/following-the-tab-folder.md) · [Português (Brasil)](../pt-BR/following-the-tab-folder.md) · [Português (Portugal)](../pt-PT/following-the-tab-folder.md) · [Русский](../ru/following-the-tab-folder.md) · [Svenska](../sv/following-the-tab-folder.md) · [Türkçe](../tr/following-the-tab-folder.md) · [Українська](../uk/following-the-tab-folder.md) · **简体中文**

# 跟随活动标签页的文件夹

GitPoulpi 可以把多个仓库作为标签页同时打开。启动它的那个 shell 会停留在原来的文件夹：程序无法更改其父 shell 的文件夹。GitPoulpi 提供两种方式，让你在活动标签页的文件夹中获得一个 shell：

- **退出并 cd 到此**：GitPoulpi 退出，你的 shell 切换到活动标签页的文件夹。这需要一个由 `gitpoulpi shell-init` 打印的小型 shell 函数。
- **打开方式**：在终端、多路复用器窗格、编辑器或文件管理器中打开活动标签页的文件夹。无需设置 shell。参见[打开方式](open-in.md)。

```
┌🐙 GitPoulpi vX.Y.Z -- /work/other-repo - Open in ▾ ── ⌕ 全局搜索 ⚙ × ┐
│ ⌂  main-repo x  other-repo x                   Quit & cd here           │
```

- **打开方式**位于标题栏中仓库路径之后。窗口太窄时会隐藏。
- **退出并 cd 到此**位于标签栏的右端。只有同时满足以下两点时才会显示：
  - GitPoulpi 是通过 shell 函数（见下文）启动的。
  - 活动标签页不是你启动 GitPoulpi 时所在的文件夹。
- 这些命令始终在面板（`Ctrl+P`）中：`标签页：退出并 cd 到此`、`标签页：打开方式…`（`Alt+O`）和 `标签页：用默认方式打开`（`Ctrl+Alt+O`）。没有 shell 函数时，`标签页：退出并 cd 到此`会告诉你如何设置。

正常退出（`q`）绝不会更改你 shell 的文件夹。只有**退出并 cd 到此**才会，除非你选择 `auto`（见下文）。

## 设置 shell 函数

`gitpoulpi shell-init <shell>` 会打印一个名为 `gp` 的函数。使用 `--name` 可选择其他名称。该函数运行 `gitpoulpi --cwd-file <temp file> ...`。当你用**退出并 cd 到此**退出时，GitPoulpi 会把文件夹写入该文件，函数随后切换到该文件夹。GitPoulpi 不会安装或编辑任何内容：你只需在 shell 的启动文件中加一行。

| Shell | 添加到 | 行 |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(gitpoulpi shell-init bash)"` |
| zsh | `~/.zshrc` | `eval "$(gitpoulpi shell-init zsh)"` |
| fish | `~/.config/fish/config.fish` | `gitpoulpi shell-init fish \| source` |
| PowerShell | 你的 `$PROFILE`（`notepad $PROFILE`） | `gitpoulpi shell-init pwsh \| Out-String \| Invoke-Expression` |

然后用 `gp` 而不是 `gitpoulpi` 启动 GitPoulpi。参数会被原样传递（`gp -s work`），退出状态就是 `gitpoulpi` 的退出状态。

**已有的 `gp` 绝不会在没有警告的情况下被替换。** 如果你的 shell 已经有名为 `gp` 的命令、别名或函数，脚本会打印警告且不定义任何内容。请选择另一个名称，例如 `gitpoulpi shell-init bash --name gpl`。在 PowerShell 中，该函数会替换内置的 `gp` 别名（`Get-ItemProperty`）以及指向 `gitpoulpi` 的别名。其他任何 `gp` 都会保留，并给出警告。

每条 `gitpoulpi` 命令都接受 `--cwd-file`，但只有 TUI 会写入该文件。

## 设置：跟随活动标签页的文件夹

设置 > 打开方式 > **跟随活动标签页的文件夹**（`ui.tab_dir_follow`）：

| 值 | 行为 |
|---|---|
| `buttons`（默认） | 如上所述的**退出并 cd 到此**和**打开方式**。 |
| `auto` | 没有**退出并 cd 到此**按钮。**打开方式**仍在。每次退出都会让你的 shell 切换到活动标签页的文件夹（前提是你通过 shell 函数启动了 GitPoulpi）。每次打开新标签页时，都会运行默认的**打开方式**条目。 |

在 `auto` 模式下，默认条目只对你自己打开的标签页运行：来自主页标签页、worktree、克隆或初始化。对于启动时恢复的标签页、第一个标签页，或切换到已经打开的标签页时，它不会运行。

旧的设置 `ui.terminal_launch` 和 `ui.terminal_launch_command` 已移入**打开方式**列表。参见[打开方式](open-in.md#旧版键)。
