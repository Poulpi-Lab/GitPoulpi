[English](../open-in.md) · [Čeština](../cs/open-in.md) · [Deutsch](../de/open-in.md) · [Español](../es/open-in.md) · [Français](../fr/open-in.md) · [Italiano](../it/open-in.md) · [日本語](../ja/open-in.md) · [한국어](../ko/open-in.md) · [Nederlands](../nl/open-in.md) · [Polski](../pl/open-in.md) · [Português (Brasil)](../pt-BR/open-in.md) · [Português (Portugal)](../pt-PT/open-in.md) · [Русский](../ru/open-in.md) · [Svenska](../sv/open-in.md) · [Türkçe](../tr/open-in.md) · [Українська](../uk/open-in.md) · **简体中文**

# 打开方式

标题栏中仓库路径旁边的 **打开方式 ▾** 按钮，可以在别处打开当前标签页的文件夹：你的终端、新的多路复用器标签页或窗格、编辑器、文件管理器。列表里有什么由你决定。没有任何硬编码：每个条目都是一个标签加一条命令。

## 使用方法

- **点击 打开方式 ▾**：会下拉出一个列表。点击条目即可运行。默认条目以 `•` 标记；最后一行 **配置…** 会打开[设置](settings.md#打开方式)中的“打开方式”页面。

  ![“打开方式”列表：终端、两个编辑器、一个 Git 工具和文件管理器，默认项带圆点标记，末尾是“配置…”](../images/open-in-menu.png)

- **键盘**：`Alt+O` 打开同一个列表（`Up`/`Down`、`Enter`、`Esc`；条目多于七个时，可输入文字搜索）。`Ctrl+Alt+O` 立即运行默认条目。命令面板（`Ctrl+P`）：`标签页：打开方式…` 和 `标签页：用默认方式打开`。即使标题栏放不下这个按钮，面板命令也照样可用。这两个键都可以重新绑定；参见[快捷键](keybindings.md)。
- **右键点击仓库标签页**：标签页菜单里有一个 **打开方式** 子菜单，条目相同。
- 列表会从本次会话中你最近使用的条目开始。
- 状态栏会立即显示“正在 ... 中打开”。如果程序在约 1.5 秒内失败（例如 tmux 没有服务器，或某个参数有误），其错误输出的第一行会显示在状态栏上。未安装的程序则会立即点名。

在你保存自己的列表之前，默认条目由检测结果决定：在 herdr 内是“Herdr: new tab”；在 Omarchy 上是 kitty 的新窗口（kitty 是它的终端）；其他情况是文件管理器。

该按钮会显示在每个仓库标签页上，包括 GitPoulpi 启动时所在的文件夹，也包括 `ui.tab_dir_follow = "auto"` 模式。在 `auto` 模式下，打开新标签页时默认条目还会自动运行（启动时恢复的标签页除外）。参见[跟随活动标签页的文件夹](following-the-tab-folder.md)。

## 条目的样子

在 `config.toml` 中，每个条目对应一个 `[[ui.open_in]]` 表：

```toml
[ui]
open_in_default = "Herdr: new tab"   # a label; empty means the first entry

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

你很少需要手写：设置页面可以编辑它。所有键都列在[配置参考](config-reference.md)中。

### 命令与变量

命令首先被拆分为参数（空格、`'...'` 和 `"..."` 会把词组合在一起），然后在每个参数内部替换变量。因此名称含空格的文件夹仍是同一个参数。除非你的命令自己启动 shell，否则**不涉及任何 shell**。

| 变量 | 替换为 |
|---|---|
| `${cwd}` | 标签页的文件夹。 |
| `${repo}` | 仓库根目录（对于 worktree 标签页，与 `${cwd}` 不同）。 |
| `${name}` | 文件夹的基本名称。 |
| `${branch}` | 已检出的分支；分离 HEAD 时为空。 |
| `${web_url}` | origin 远程仓库为 GitHub 或 GitLab 时的网页地址；否则为空。 |
| `${env:NAME}` | 环境变量 `NAME`；未设置时为空。 |

旧的 `{repo}` 仍然有效，含义是 `${cwd}`。任何其他 `${...}` 都是错误，并会指出变量名。

程序以标签页的文件夹作为工作目录启动，并获得三个环境变量，方便编写 shell 片段：

| 变量 | 值 |
|---|---|
| `GITPOULPI_CWD` | 标签页的文件夹。 |
| `GITPOULPI_REPO` | 仓库根目录。 |
| `GITPOULPI_BRANCH` | 分支，或为空。 |

```toml
[[ui.open_in]]
label = "Terminal here, then status"
command = "sh -c 'cd \"$GITPOULPI_CWD\" && git status; exec $SHELL'"
```

在 Windows 上，程序通过 `PATH` 和 `PATHEXT` 查找，因此 `code` 和 `cursor`（`.cmd` 文件）可以正常工作，也不会闪出控制台窗口。

## 预设

设置 > 打开方式 > **添加预设** 只列出适用于你操作系统的现成条目（Windows 的预设绝不会在 Linux 上显示，依此类推），并按类别排列：每一行以类别开头，如 `资源管理器：...`、`多路复用器：...`、`终端：...`、`编辑器：...`、`Git 工具：...`、`AI 代理：...`、`网页：...`。最上面的**已检测**是计算得出的默认列表中的条目。在每个类别内部，可以立即使用的预设排在最前（你正运行在该多路复用器或终端内，且程序在 `PATH` 上），然后是已安装的，最后是其余的。在组合框中输入即可搜索，例如 `lazygit` 或 `Git 工具：`。依赖某个条目之前，可以先用**在当前标签页测试**试一试。

![“打开方式”页面的“+ 添加”列表：先是已检测的条目，然后是资源管理器、多路复用器和其他类别，每个预设都以其类别为前缀](../images/settings-open-in-presets.png)

“在窗格中运行工具”类预设（`lazygit in Herdr tab`、`yazi in tmux window`、`Claude Code in Herdr split` 等）只有在宿主（herdr、tmux、zellij、WezTerm、kitty、`xdg-terminal-exec`、Windows Terminal）和该工具都已安装时才会提供。`$EDITOR in ...` 会在 herdr 或 tmux 中运行由你的 `EDITOR` 变量指定的编辑器（在 herdr 中回退到 `vi`）。

Herdr 的工具预设会先创建标签页或分屏，然后把命令键入其 shell，因此工具在该文件夹中启动，并且工具退出后窗格仍保持打开。`AI 代理` 类预设使用 `herdr agent start`，所以 herdr 会跟踪代理的状态。

| 类别 | 预设（`V` = 已通过运行或对照已安装的帮助验证） |
|---|---|
| 资源管理器 | Windows `explorer.exe` (V)、macOS Finder (V)、Linux `xdg-open` (V)、Nautilus、Dolphin、Thunar、Nemo、PCManFM；yazi、superfile、nnn、lf、ranger（在窗格/标签页/终端中） |
| 多路复用器 | herdr 标签页、工作区、向右/向下分屏 (V)；tmux 窗口、向右/向下分屏、分离会话 (V)；zellij 窗格、标签页、浮动；WezTerm 标签页、分屏、新窗口；kitty 标签页、分屏窗口、OS 窗口（V：参数取自帮助；需要 `allow_remote_control`）；screen；cmux；psmux |
| 终端 | `xdg-terminal-exec` (V)、kitty (V)、foot (V)、Alacritty（及 `alacritty msg`）、WezTerm、Ghostty（Linux、macOS）、GNOME Terminal、Ptyxis、Konsole、Xfce Terminal、Tilix、Terminator、MATE Terminal、iTerm2、Terminal.app、Warp；Windows：Windows Terminal（窗口、标签页、分屏，以及 PowerShell 7、Windows PowerShell、命令提示符、WSL 各一个标签页）、PowerShell 7 / Windows PowerShell / 命令提示符 / WSL 窗口、Git Bash |
| 编辑器 | VS Code (V)、VSCodium、Cursor、Windsurf、Zed、Sublime Text、Emacs、IntelliJ IDEA、RustRover、PyCharm、WebStorm、CLion、GoLand、Rider、PhpStorm、Android Studio、Visual Studio（Windows）；Neovim、Vim、Helix、`$EDITOR`（在窗格/标签页/终端中） |
| Git 工具 | gitk、`git gui`、gitg、Sublime Merge、GitKraken、GitHub Desktop；lazygit、tig、gitui（在窗格/标签页/终端中；herdr、tmux、kitty 和 `xdg-terminal-exec` 的预设已验证 V） |
| AI 代理 | Claude Code、Codex、Gemini CLI、opencode，在 herdr 标签页/分屏中（V，Claude 已验证）、tmux、zellij、WezTerm、kitty、新终端 |
| 网页 | `Repository page on the web`（`${web_url}`）、`gh repo view --web` (V)、`glab repo view --web` |

Windows 上的 WSL：`wsl.exe --cd ${cwd}` 原样接受 Windows 路径，而 Windows Terminal 的 `wsl.exe` 标签页会在该文件夹中启动，WSL 把它映射到 `/mnt/` 之下。

资源管理器、Windows Terminal、PowerShell、`cmd`、WSL、Warp、psmux、Visual Studio 和 Zed（Zed 仅限 Unix）只在它们存在的平台上出现；tmux、zellij、cmux、screen、kitty 远程控制以及由 `sh` 驱动的 herdr 预设仅限 Unix；herdr（普通预设）、WezTerm、Alacritty、VS Code、Cursor 和 JetBrains IDE 在各处都提供。

### 变量 `${web_url}`

仓库 `origin` 远程的网页地址，前提是它是 GitHub 或 GitLab 项目（github.com、gitlab.com，或在 `[[integrations]]` 中有条目的自托管主机）；否则为空，此时使用它的条目会停下并给出提示，而不是打开一个空地址。它是可选的：除了 `Repository page on the web`，没有任何预设需要它。

## 设置页面

设置 > **打开方式** 用来编辑列表。条目是左侧的一个[列表](settings.md#列表)（默认条目以 `★` 标记），所选条目的字段在右侧；`+ 添加`、`创建副本`、`删除`、`↑` / `↓` 以及拖动某一行都可用来管理这个列表。

| 行 | 作用 |
|---|---|
| **默认条目** | **用默认方式打开**所运行的条目。它在“打开方式”列表中以 `•` 标记。 |
| **标签** | 列表中显示的名称。唯一，不能为空。 |
| **命令** | 命令，以及用于插入的变量标签（`${cwd}`、`${repo}`、`${branch}`、`${name}`、`${web_url}`）。 |
| **在当前标签页测试** | 立即对活动标签页的文件夹运行该条目。 |
| **+ 添加 ▾** | 追加一个现成条目（已检测的排在最前），或选 **自定义**：追加一个空条目，随后编辑它的命令。 |
| **恢复检测到的默认值** | 用此电脑上检测到的条目替换列表，并清除默认项。 |

编辑会随你的操作即时保存。第一次编辑会把整个 `[[ui.open_in]]` 数组写入 `config.toml`；在此之前，列表是计算得出的（参见[旧版键](#旧版键)），文件中没有 `open_in` 键。设置搜索也能找到这个页面。

## 二进制文件：打开方式

二进制文件（PDF、Word 文档、电子表格、压缩包等）在 Diff 视图和文件视图中没有文本可显示。当操作系统知道该类型对应的应用时，视图会在通常的提示旁显示 **打开方式** 按钮。它的列表中，系统用来打开该类型的应用排在第一位并用圆点标记，随后是系统为该类型注册的所有其他应用（即系统自带的“打开方式”列表）。在 Linux 上，依据文件名的 MIME 类型，默认应用来自 `mimeapps.list`，其余来自 `.desktop` 文件；在 Windows 上依据注册表（用户的选择和为该扩展名注册的程序）；在 macOS 上只有默认应用。

`o` 运行默认应用，`Shift+O` 打开列表。两者都在命令面板中（*文件：用默认应用打开*、*文件：打开方式...*），**打开方式...** 也在状态窗格中文件的右键菜单和 commit 的文件列表中。查找在后台进行，每个扩展名只做一次：查找期间视图显示“正在查找应用...”。系统不知道任何应用的类型以及纯文本，保持原来的提示，也没有按钮。

![文件视图中的 PDF 及其“打开方式”列表：首先是以圆点标记的默认应用，然后是系统为该类型注册的其他应用](../images/open-with.png)

工作区中的文件按原样交给应用。commit（或索引）中的版本会写入一个 **只读临时副本**，并在状态栏给出提示，因此应用无法修改仓库。两种方式都不会改变仓库：没有日志条目，撤销不受影响。

## 旧版键

旧的键 `ui.terminal_launch` 和 `ui.terminal_launch_command` 仅在 `ui.open_in` 不存在时才会被读取。此时计算得出的列表是：

1. 你旧的终端选择（如果不是 `auto`；`custom` 变成“Terminal (custom)”，其中的 `{repo}` 改写为 `${cwd}`；预设 id 变成对应的预设）。`auto` 值变成第一个条目“Default terminal”。
2. 操作系统的文件资源管理器，
3. 从环境中检测到的多路复用器和终端，
4. 在 `PATH` 上找到的编辑器。

当你从设置中保存列表后，`ui.open_in` 接管，旧的键会被忽略。面板命令 **标签页：在终端中打开** 和 **标签页：在文件管理器中打开** 不使用这个列表。

## 故障排除

- *“program X was not found”*：该程序不在 GitPoulpi 看到的 `PATH` 上。请从已有该程序的 shell 中启动 GitPoulpi，或者写出完整路径。
- *tmux/zellij/herdr 条目立即失败*：它们与正在运行的服务器通信，所以只有 GitPoulpi 在其内部运行时才能工作。
- *kitty 标签页条目失败*：它需要在 `kitty.conf` 中设置 `allow_remote_control`。
- *Windows 资源管理器*：`explorer.exe` 即使成功也会以退出码 1 退出；GitPoulpi 把这视为成功。
