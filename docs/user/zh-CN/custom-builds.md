[English](../custom-builds.md) · [Čeština](../cs/custom-builds.md) · [Deutsch](../de/custom-builds.md) · [Español](../es/custom-builds.md) · [Français](../fr/custom-builds.md) · [Italiano](../it/custom-builds.md) · [日本語](../ja/custom-builds.md) · [한국어](../ko/custom-builds.md) · [Nederlands](../nl/custom-builds.md) · [Polski](../pl/custom-builds.md) · [Português (Brasil)](../pt-BR/custom-builds.md) · [Português (Portugal)](../pt-PT/custom-builds.md) · [Русский](../ru/custom-builds.md) · [Svenska](../sv/custom-builds.md) · [Türkçe](../tr/custom-builds.md) · [Українська](../uk/custom-builds.md) · **简体中文**

# 自定义构建

适合想要更小二进制文件的人：一个交互式脚本会询问要包含哪些语言、托管平台集成、主题、徽标颜色和功能，然后只用优化配置构建这些内容。常规构建和正式发布保持不变（全部开启）。

```
./build-custom.sh                # Linux / macOS (scripts/build-custom.sh)
build-custom                     # Windows, cmd or PowerShell (scripts/build-custom.ps1)
```

输出位于 `target/custom/`，绝不会覆盖正式的 `target/dist/`。脚本会打印每个二进制文件的大小，并在 `target/dist/` 的构建存在时打印与它的差值。它还会在二进制文件旁写入 `BUILD-FEATURES.txt`。

## 你可以选择什么

| 选项 | 关闭后 |
|---|---|
| 语言 | 只有英语（始终构建）加上你勾选的语言。设置中的选择器只列出已构建的语言。 |
| 主题 | GitPoulpi Dark、GitPoulpi Light 和 Terminal 始终会构建（`system` 需要 Dark 和 Light）。其他所有主题按系列选择（`catppuccin`、`gruvbox`、`nightfox` 等；`--list-themes` 会打印它们）。未选中的主题不会被编译进去：它们会从设置的下拉框和 `gitpoulpi theme list` 中消失，配置中指定它们时会像未知主题一样回退，并给出常规警告。 |
| 徽标颜色 | “关于”窗口中的像素章鱼有 20 种配色（`--list-logo-colors`）。选择要编译进去的配色（至少一个）、默认配色（“关于”首先显示它；当存储的 `ui.logo_color` 未被编译进去时也会回退到它；它必须是已编译的配色之一；正式默认：lavender），以及 Windows 可执行文件图标的颜色（任意配色，默认：徽标默认值）。只有一种配色时，徽标点击后仍会旋转，但保持该颜色，也不会提供颜色列表（`c`、右键点击）。 |
| 集成（GitHub、GitLab、Gitea、Forgejo、Azure DevOps、Bitbucket、Gerrit、Jira、Trello、Linear、YouTrack、Redmine） | 全部关闭时：只能通过 URL 克隆；没有议题，没有 pull/merge request。 |
| AI | 没有 AI 按钮、命令，也没有 `ai` CLI/MCP 工具。 |
| MCP | 没有 `gitpoulpi-mcp` 中继程序，也没有 `gitpoulpi mcp` 命令。`ui` 代理命令保留。 |
| LFS、Worktree、子模块 | 对应的管理界面和命令都会消失；返回一个带类型的“未内置于此二进制文件”应答。 |
| 头像下载 | 仅显示首字母徽章，不进行网络获取。 |
| 图像预览 | 图像文件显示普通的二进制提示；**编辑方式**仍可使用。 |
| Radar | 没有 Radar 标签页、`Alt+R` 或命令面板命令；Radar 永远不会打开。需要一个集成（没有集成时它本来也不起作用）。 |
| Mermaid | 没有“复制为 Mermaid gitGraph”（Log 菜单、`Shift+Y`、命令面板），CLI/MCP 中也没有 `log --format mermaid`。 另外，[对话](conversations.md)中 `mermaid` 代码块下没有**打开图表**按钮（该块保持为代码），也没有“对话：打开条目的图表”命令面板行。 |
| Board | 没有 Kanban 看板视图（议题标题处的按钮、命令面板），也没有 `hosting board` 命令。需要一个集成。 |
| Conversation | 没有议题或请求的应用内[对话](conversations.md)视图：在议题行、看板卡片或 Radar 行上按 Enter 仍像以前一样打开浏览器，没有“打开对话”行，没有“对话：…”命令面板命令，对话中也没有**AI 写作**（评审、议题表单和 pull request 表单的文本框保留它）。CLI 和 MCP 的对话命令（`hosting issue comments`、`comment`、`comment-edit`、`comment-delete`、`hosting review conversation`）和 `ai hosting-reply` 保留。需要一个集成。 |
| Review | 评审的“概览”页就是对话视图，所以含 Review 的构建也包含对话视图，不论 *Conversation* 如何。没有应用内的代码评审视图（行菜单、在 pull request 行上按 Enter、命令面板、Radar 的 `Review`），也没有 AI 评审辅助。`hosting review` 的 CLI/MCP 命令保留（Gerrit 和代理会用到它们）。需要一个集成。 |
| 历史重写工作台（高级历史重写：作者、日期、消息、内容） | 没有历史重写工作台：没有 `重写：` 命令面板命令，没有工作台的组合键、按键或帮助部分，Log 菜单中没有“重写历史”行，修订对话框中没有“深入修改历史…”按钮，没有“用 AI 创建重写规则”或脚本 AI，同步设置页面上没有脚本限制，AI 设置中也没有历史重写 AI 区块。`rewrite` CLI/MCP 命令、`ai rewrite-script`、`ai rewrite-rules` 和 `ui propose rewrite` 都会从命令树中消失。`.mailmap` 身份（`mailmap`、`log.use_mailmap`）不属于它，会保留。 |
| 全局搜索 | 标题栏中没有全局搜索按钮，没有 `Ctrl+Alt+F`，没有 `搜索：工作空间…` 命令面板命令，也没有 `gitpoulpi search` CLI/MCP 命令。Log 的查找栏（`ui show search`）是另一个功能，会保留。 |
| 工作空间 | 窗口只使用默认的单一标签页列表（“无工作空间”）：没有工作空间标记，没有工作空间标签页，没有 `Alt+W` / `Alt+J`，没有 `工作空间：` 命令面板命令，标签页行中没有“添加到工作空间”，没有 `gitpoulpi workspace` CLI/MCP 命令，`commit` 和 `search` 上也没有 `--workspace` 标志。标签页和会话照常工作。其他构建写下的 `workspaces.toml` 既不会被读取，也不会被改动；工作空间的默认配置文件和 AI 指令不生效。Radar 和全局搜索覆盖已打开的标签页。 |
| 重写脚本（内嵌 Lua） | 历史重写中的 Script 规则会被以带类型的“此二进制文件未包含”应答拒绝，并且不会链接 Lua。历史重写工作台本身保留（它有自己的选项）。历史重写工作台关闭时此项没有意义，并会给出警告。 |

你的配置文件在各个构建之间共享：为未构建的功能所做的设置会被保留并忽略。

## 交互式

Bash 脚本使用 `whiptail` 或 `dialog` 的复选列表（三个界面，然后是摘要），两者都不存在时则使用编号提示。PowerShell 脚本在可用时使用 `Out-GridView`，否则使用提示。你之前的选择会预先勾选。

## 非交互式

任何参数都会跳过提问，直接进入 `cargo run -p xtask -- build-custom`：

```
./build-custom.sh --langs fr,de --integrations github --no-ai --no-mcp
./build-custom.sh --last --dry-run         # previous choices, print the cargo command only
.\build-custom -Langs fr -Integrations none -NoAi -NoLfs -NoWorktree -NoSubmodules
```

| 选项 | 含义 |
|---|---|
| `--langs fr,de` / `all` / `en` | 英语之外的语言。代码即语言环境文件名（`pt-br`、`zh-cn` 等）。 |
| `--integrations github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues` / `all` / `none` | 提供方：GitHub、GitLab（含自托管）、Azure DevOps（Services 和 Server）、Bitbucket（Cloud 和 Data Center）（托管平台），含 Codeberg 的 Gitea 和 Forgejo（`gitea`）（托管平台），Jira、Trello、Linear、YouTrack（Cloud 和 Server）、Redmine（议题跟踪器），Gerrit Code Review（`gerrit`，仅评审），本地 Markdown 议题（`local-issues`，一个文件夹或 Obsidian 库）。`azure-devops`、`bitbucket`、`gerrit`、`gitea`、`jira`、`linear`、`local-issues`、`redmine`、`trello`、`youtrack` 各自是独立的 cargo feature；大小见 [binary-size](../../developer/binary-size.md)。 |
| `--themes all` / `none` / `nord,catppuccin` | 要编译进去的主题系列（默认 `all`）。`--list-themes` 会打印这些系列。 |
| `--logo-colors all` / `violet,teal` | 要编译进去的徽标配色（默认 `all`，保持表中的顺序，即旋转顺序）。`--list-logo-colors` 会打印它们（`*` = 正式默认）。 |
| `--logo-default teal` | 默认配色；必须在已编译的配色之中（默认：lavender，否则为第一个已编译的）。 |
| `--icon-color sun` | Windows 可执行文件图标的颜色（默认：徽标默认值）。 |
| `--[no-]ai`、`--[no-]mcp`、`--[no-]lfs`、`--[no-]worktree`、`--[no-]submodules`、`--[no-]avatar-fetch`、`--[no-]image-view`、`--[no-]radar`、`--[no-]mermaid`、`--[no-]board`、`--[no-]review`、`--[no-]conversation` | 每项默认开启。 |
| `--[no-]history-rewrite` | 历史重写工作台（高级历史重写：作者、日期、消息、内容）。默认开启。 |
| `--[no-]global-search` | 全局搜索（标题栏按钮、`Ctrl+Alt+F`、`gitpoulpi search`）。默认开启。 |
| `--[no-]workspaces` | 工作空间（有名称的仓库分组）。默认开启；关闭后只剩默认的标签页列表。 |
| `--[no-]rewrite-scripts` | 用于重写的 Script 规则的内嵌 Lua。默认开启；历史重写工作台关闭时会被忽略并给出警告。 |
| `--save <name\|file>` | 把已确定的选择保存为已保存的选择（名称或文件路径）。 |
| `--preset <name\|file>` | 从已保存的选择开始（名称或文件路径）；其他选项无论在命令行的什么位置都会覆盖它。 |
| `--list-presets`, `--delete-preset <name>` | 列出已保存的选择及其文件夹 / 按名称删除其中一个。 |
| `--last` | 复用 `target/custom-build/last-choices.env`；其他选项会覆盖它。 |
| `--full` | 完全不做选择（全部开启）。 |
| `--profile custom\|dist\|release` | 默认 `custom`（采用 `dist` 的设置，输出目录独立）。 |
| `--dry-run` | 打印确切的 cargo 命令后停止。 |
| `-- <cargo args>` | 传给 cargo，例如 `--target x86_64-unknown-linux-musl`。 |

PowerShell 的名称：`-Langs -Integrations -Themes -LogoColors -LogoDefault -IconColor -NoAi -NoMcp -NoLfs -NoWorktree
-NoSubmodules -NoAvatarFetch -NoImageView -NoRadar -NoMermaid -NoBoard -NoReview -NoHistoryRewrite -NoGlobalSearch -NoWorkspaces -NoRewriteScripts -Last -Preset -Save -ListPresets -DeletePreset -Full -DryRun -Profile -CargoArgs`。

主题选择以环境变量 `GITPOULPI_THEMES`（`all`、`none` 或以逗号分隔的系列列表）传递给构建，由 `gitpoulpi-config` 的构建脚本读取，并在编译时丢弃其他调色板；如果你愿意，也可以为普通的 `cargo build` 自行设置它。`gitpoulpi --version` 会打印一行 `themes:`，列出已构建的集合。

徽标选择是环境变量 `GITPOULPI_LOGO_COLORS`（`all` 或以逗号分隔的 slug 列表）、`GITPOULPI_LOGO_DEFAULT` 和 `GITPOULPI_ICON_COLOR`，由 `gitpoulpi-tui` 的构建脚本（它把 `crates/gitpoulpi-logo-build/colourways.json` 过滤成“关于”窗口所嵌入的表）以及 `gitpoulpi-cli` 的构建脚本（图标）读取。颜色的来源仍是 `crates/gitpoulpi-logo-build/colourways.json`。

### Windows 可执行文件图标

当目标是 Windows 时，`gitpoulpi` 可执行文件会以章鱼作为图标，在构建时生成（无需 Python 或 ImageMagick），颜色为所选的图标颜色，尺寸从 16 到 128 px。它通过 `winresource` crate 嵌入，并需要平台的资源编译器（MSVC 上是 Windows SDK 的 `rc.exe`，GNU 上是 `windres`）。如果缺失，构建只会打印警告，可执行文件保留默认图标。在 GNU 工具链上，只有 `gitpoulpi` 带有该图标（否则资源会被链接两次）。面向 Linux 和 macOS 的构建会忽略以上所有内容，只保留颜色表。

## 已保存的选择

想在不重做全部选择的情况下重新构建新版本，可以把选择按名称保存，之后从它开始。与 `--preset` 一起给出的选项仍会覆盖已保存的值。

```
./build-custom.sh --no-ai --no-rewrite-scripts --save no-ai    # keep these choices as "no-ai"
./build-custom.sh --preset no-ai --dry-run                     # start from them later
./build-custom.sh --preset ~/dotfiles/gitpoulpi.env --langs fr # a file path works too
./build-custom.sh --list-presets
./build-custom.sh --delete-preset no-ai
.\build-custom -Preset no-ai -Save no-ai-v2                    # PowerShell
```

已保存的选择是简单的 `KEY=value` 文件（格式与 `last-choices.env` 相同，shell 可直接读取）。它们位于 GitPoulpi 配置目录下的 `custom-builds` 文件夹中，在 `target/` 之外，因此 `cargo clean` 不会删除它们：

- Linux：`$XDG_CONFIG_HOME/gitpoulpi/custom-builds/` （默认为 `~/.config/gitpoulpi/custom-builds/`）
- Windows：`%APPDATA%\GitPoulpi\config\custom-builds\`
- macOS：`~/Library/Application Support/GitPoulpi/custom-builds/`

`GITPOULPI_CONFIG_DIR` 会把该文件夹连同配置一起移动。`--preset` 和 `--save` 也接受文件路径（包含 `/` 或 `\`，或以 `.env` 结尾的值），便于把选择保存在 dotfiles 仓库中。

由其他版本的 GitPoulpi 写入的选择仍可加载。未知的键，或已不存在的语言、集成、主题、颜色，会被忽略并给出警告。文件中缺少的键（后来新增的功能）取默认值，后来新增的集成会被包含；两者都会在标准错误输出中提示。

存在已保存的选择时，交互模式会先提供“从以下开始”（上次的选择、已保存的选择，或全部开启），并在最后询问是否把当前选择按名称保存。

## 它能节省多少

在 Linux 上于 2026-10-29 测得，commit 为 `65999ef5c`，为 `gitpoulpi`（整个程序：界面、CLI 和 MCP 服务器）基于完整构建的大小，括号内为与完整构建的差值
（详情见 `docs/developer/binary-size.md`）。无论如何选择，中继程序 `gitpoulpi-tui`、`gitpoulpi-mcp` 和 `gitpoulpi-launch` 都远小于 1 MiB：

| 构建（除非另有说明，均基于 `full`） | `gitpoulpi` (MiB) |
|---|---|
| 完整（全部开启） | 64.6 |
| 仅英语 | 52.0 (-12.6) |
| 关闭 MCP | 61.6 (-3.0) |
| 关闭 AI | 62.6 (-2.0) |
| 关闭所有集成 | 56.3 (-8.2) |
| 关闭 AI + 集成 + 头像获取 | 52.2 (-12.3) |
| 关闭历史重写工作台（`--no-history-rewrite`） | 61.6 (-3.0) |
| 关闭重写脚本 | 63.1 (-1.5) |
| 关闭全局搜索 | 63.5 (-1.1) |
| 关闭工作空间 | 63.5 (-1.1) |
| 关闭 Radar | 63.4 (-1.2) |
| 关闭 Board | 63.6 (-1.0) |
| 关闭 Review | 63.2 (-1.4) |
| 关闭 Mermaid | 63.6 (-1.0) |
| 关闭 LFS | 63.5 (-1.1) |
| 关闭 Worktree | 63.7 (-0.9) |
| 关闭子模块 | 63.7 (-0.9) |
| 关闭头像获取 | 63.7 (-0.9) |
| 关闭图片查看 | 63.3 (-1.2) |
| 全部关闭，仅英语（保留 LFS） | 35.2 (-29.4) |
| **全部关闭，仅英语** | **34.9 (-29.7)** |

## 注意事项

- 自定义构建和任何 `dist` 构建一样慢（thin LTO，几分钟）。
- 它和 `build-opti` 一样，不使用 sccache 运行。
- 语言代码会与随附的语言环境进行核对；未知代码会被拒绝，并列出有效的代码。
