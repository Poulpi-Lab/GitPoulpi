[English](../config-reference.md) · [Čeština](../cs/config-reference.md) · [Deutsch](../de/config-reference.md) · [Español](../es/config-reference.md) · [Français](../fr/config-reference.md) · [Italiano](../it/config-reference.md) · [日本語](../ja/config-reference.md) · [한국어](../ko/config-reference.md) · [Nederlands](../nl/config-reference.md) · [Polski](../pl/config-reference.md) · [Português (Brasil)](../pt-BR/config-reference.md) · [Português (Portugal)](../pt-PT/config-reference.md) · [Русский](../ru/config-reference.md) · [Svenska](../sv/config-reference.md) · [Türkçe](../tr/config-reference.md) · [Українська](../uk/config-reference.md) · **简体中文**

# 配置参考

`config.toml` 的每一个键、它的默认值以及作用。你很少需要手动编辑这个文件：[设置界面](settings.md)（`Ctrl+,`）会在你更改时写入它，`gitpoulpi config set` 也会。

## 文件位置，以及取值如何确定

| | |
|---|---|
| 文件 | `%APPDATA%\GitPoulpi\config.toml`（Windows）、`~/.config/gitpoulpi/config.toml`（Linux）、`~/Library/Application Support/GitPoulpi/config.toml`（macOS）。`gitpoulpi config path` 会打印它。 |
| 另一个文件夹 | `GITPOULPI_CONFIG_DIR=<dir>` 会移动整个配置文件夹（该文件、`sessions/`、守护进程的端口文件）：用于便携安装、测试，或让代理远离你自己的设置。 |
| 优先级 | 内置默认值，然后是 `config.toml`，然后是 `GITPOULPI_*` 变量，然后是命令行标志。优先级最高者生效。错误的值（文件中拼错的选项、错误的变量）会报错，绝不会被悄悄忽略。 |
| 变量 | 将键名转为大写并把点号换成下划线：`general.auto_prune` 对应 `GITPOULPI_GENERAL_AUTO_PRUNE`。仅适用于 GitPoulpi 实际读取的键（不包括下文标注为*未使用*的键，也不包括 `ui.log_columns_hidden` 之类的列表）。 |
| 标志 | `--backend git2\|system` 会覆盖该命令所启动的守护进程的 `general.default_backend`。 |
| 令牌和密钥 | 绝不在此文件中。托管平台令牌和 AI API 密钥会交给你系统的凭据助手（Git Credential Manager、libsecret / `pass`、钥匙串）；文件中只保存一个引用（`token_ref`）。没有凭据助手时，保存令牌会失败并给出提示，令牌仅在本次会话中有效。 |

```
gitpoulpi config path                          # the file, and whether it exists yet
gitpoulpi config get general.auto_stash        # the value, the layer it came from (default/file/env/flag), and whether anything reads it
gitpoulpi --json config get ui                 # a whole section
gitpoulpi config set general.auto_stash always # checked against the schema, then written
```

`config set` 会让文件保持精简（只保留有人设置过的键），并保留它未被要求处理的键，但会丢弃注释。它拒绝列表和整张表，`keybindings.*` 除外（多个组合键用空格分隔；留空表示解除绑定）。请在设置中或手动编辑 `[[profiles]]`、`[[integrations]]` 和 `[[ai.profiles]]`。

在表格中，**重启**表示该值在启动时读取；其余的都会立即生效（或在下一次刷新、fetch 或关闭设置时生效）。

## `[general]`

| 键 | 默认值 | 取值与作用 |
|---|---|---|
| `general.default_backend` | `"git2"` | `git2`（libgit2，在 stage、merge、rebase、stash 和网络操作上回退到 `git` 程序）或 `system`（仅使用已安装的 `git`）。正在运行的守护进程会保留它的后端：请**重启**它（`gitpoulpi daemon restart`）。 |
| `general.auto_stash` | `"ask"` | 当本地更改妨碍 checkout、pull、merge 或 rebase 时：`ask`（TUI 中弹出对话框；CLI 和 MCP 会失败并提示 `--autostash`）、`always`（stash、执行、pop）或 `never`（拒绝）。参见[自动 stash](log-and-history.md#自动-stash)。 |
| `general.include_untracked_in_autostash` | `false` | 自动 stash 也会包含未跟踪的文件。 |
| `general.initial_commits_in_graph` | `5000` | Log 首先加载的 commit 数（至少 500）。设得更高会在超大仓库上消耗更多内存和绘制时间。 |
| `general.lazy_load_commits_in_graph` | `true` | 滚动到接近末尾时加载更早的 commit。关闭后：Log 停在初始数量。 |
| `general.commits_per_lazy_page` | `1000` | 每个后台页面加载的 commit 数（至少 50）。 |
| `general.max_commits_in_graph` | `100000` | 延迟加载停止的位置（每个 commit 约 300 字节）；`0` = 不限制。 |
| `general.auto_fetch_interval_minutes` | `5` | 每 N 分钟在后台 Fetch All，范围 0 到 60；`0` = 关闭。**重启。** |
| `general.auto_fetch_interval_unit` | `"auto"` | 设置中显示该间隔所用的单位：`minutes`、`hours` 或 `auto`。仅影响显示：间隔始终是 `auto_fetch_interval_minutes` 分钟。`auto`（以及没有这个键的配置）会选出能整除分钟数的最大单位（60 显示为“1 小时”，45 显示为“45 分钟”）；不合适的单位会被忽略。设置会同时写入两个键；旧版 GitPoulpi 会忽略单位，仍然读取分钟数。 |
| `general.fetch_on_tab_open` | `true` | 打开标签页时 Fetch All（如果 GitPoulpi 在不到 60 秒前刚 fetch 过该仓库，则跳过）。 |
| `general.auto_prune` | `false` | fetch 时也会 prune 已不存在的远程跟踪分支。 |
| `general.auto_prune_tags` | `false` | fetch 时也会删除远程已经删除的本地 tag（只涉及曾存在于远程的 tag；你从未 push 过的 tag 会保留）。需要 Git 2.41+。 |
| `general.hosting_refresh_minutes` | `5` | pull request / 议题列表和工具栏标签的刷新间隔，范围 0 到 120；`0` = 仅按需刷新以及在事件发生时刷新。 |
| `general.hosting_refresh_unit` | `"auto"` | 设置中显示该刷新间隔所用的单位：`minutes`、`hours` 或 `auto`，读取方式与 `general.auto_fetch_interval_unit` 相同。仅影响显示。 |
| `general.keep_submodules_up_to_date` | `false` | 在 checkout 或 pull 之后，如果某个子模块不同步，则执行 `git submodule update --init --recursive`。参见[子模块](submodules.md)。 |
| `general.default_branch_name` | `"main"` | 通过初始化创建的仓库的第一个分支；留空则由 git 决定。不会重命名任何分支。 |
| `general.delete_orig_files_after_merge` | `false` | merge 成功完成后，删除 `*.orig` 备份文件。 |
| `general.remember_tabs` | `true` | 重新打开上次会话的标签页；关闭时，在仓库内启动只打开该仓库，在仓库外启动只打开主页。**重启。** |
| `general.radar_enabled` | `true` | [Radar](radar.md)：它的标签页、命令面板命令、`Alt+R` 以及后台获取。关闭后这些全部隐藏，也不会再为它获取任何数据。立即生效；不含 `radar` 功能的构建不显示此项。 |
| `general.startup_tab_in_repo` | `"current-repo"` | 启动所在的文件夹位于仓库内时，打开哪个标签页：`"current-repo"`（该仓库的标签页，必要时打开）、`"last"`（上次退出时处于活动状态的标签页）、`"home"` 或 `"radar"`。用 `--repo` 指定的仓库总是打开自己的标签页。`"last"`：`remember_tabs` 关闭或没有记住任何标签页时为主页；`"radar"`：Radar 关闭或构建中没有时为主页。仅在启动时读取；需重启生效。 |
| `general.startup_tab_outside_repo` | `"home"` | 启动所在的文件夹不是仓库（例如用户文件夹）时，打开哪个标签页：`"home"`、`"last"` 或 `"radar"`。`"last"`：`remember_tabs` 关闭或没有记住任何标签页时为主页；`"radar"`：Radar 关闭或构建中没有时为主页。仅在启动时读取；需重启生效。 |
| `general.startup_tab_launcher` | `"last"` | 通过 GitPoulpi 创建的桌面或开始菜单快捷方式（会传入 `--launcher`）启动时，无论文件夹是什么，打开哪个标签页：`"home"`、`"last"` 或 `"radar"`。优先于上面两项设置。`"last"`：`remember_tabs` 关闭或没有记住任何标签页时为主页；`"radar"`：Radar 关闭或构建中没有时为主页。仅在启动时读取；需重启生效。 |
| `general.group_worktree_tabs` | `true` | 让仓库的关联 worktree 的标签页紧跟在该仓库的标签页之后：worktree 打开时、worktree 标签页被拖走时（它会自动回到所属分组旁边；仓库标签页是分组的头，它的 worktree 跟在后面）以及开启该设置时。关闭则标签页停留在你放置的位置。 |
| `general.undo_depth` | `100` | 撤销最多可以回退多少次操作；`0` = 不限制。 |
| `general.conflict_detection` | `true` | 主动冲突检测：当把当前分支 merge 到它的来源分支会产生冲突时，在 Log 上显示一个徽标，在后台计算（内存中的 merge：你的文件、索引和 ref 都不会改变）。仓库可以为自己关闭它（`[preferences] conflict_detection`）。参见[解决 merge 冲突](merge-conflicts.md#冲突预防)。 |
| `general.dev_mode` | 不存在（自动） | 供从源码检出构建 GitPoulpi 的人使用；从不在设置中显示。不存在时：正在运行的可执行文件位于 `<checkout>/target/<profile>/` 中（`target` 旁边有 `Cargo.toml` 和 `crates/gitpoulpi-tui`）时开启。`true` / `false` 强制设定。开发模式只解锁 `ui.launcher_build`。 |
| `general.journal_outside` | `true` | 把 shell 或 AI 代理对分支、tag、stash 和 HEAD 所做的操作记为撤销历史中的条目，并标为外部，这样 `Ctrl+Z` 就能撤回它们。文件的修改绝不会被记录。需要 `undo_enabled`。参见[撤销与重做](undo-redo.md#在-gitpoulpi-之外所做的更改)。 |
| `general.undo_enabled` | `true` | `false`：完全不记录撤销信息（最快），撤销历史会被清除，撤销 / 重做按钮会消失。参见[撤销与重做](undo-redo.md)。 |
| `general.active_profile` | `""` | 用于创建 commit 的 `[[profiles]]` 身份的 id。留空，或所选配置的名称或邮箱为空时：使用 git 的 `user.name` / `user.email`。 |
| `general.external_editor` | `""` | 文件的*在编辑器中打开*所用的命令，例如 `code -r {file}`。留空：已安装 VS Code 时使用它，否则使用系统默认程序。 |
| `general.external_diff_tool` | `""` | *在外部 diff 工具中打开*所用的命令：`{local}`（已 commit / HEAD 版本）、`{remote}`（工作区 / 索引版本）。留空：`git difftool --no-prompt`。 |
| `general.external_merge_tool` | `""` | 冲突卡片（二进制、LFS 或过大的文件）上的*在外部 merge 工具中打开*所用的命令：`{base}`、`{local}`（A，当前）、`{remote}`（B，传入）和 `{merged}`（工具写入结果的文件；工具成功退出时 GitPoulpi 会将其 stage）。留空：`git mergetool --no-prompt`。 |
| `general.file_explorer` | 因操作系统而异 | *在文件夹中显示*所用的命令：`{path}`（文件）、`{dir}`（它所在的文件夹）。默认值：`explorer.exe /select,"{path}"`、`open -R "{path}"`、`xdg-open "{dir}"`。 |

这三个命令模板会先被拆分为参数（`"..."` 将一个词括在一起），然后再填入占位符，因此带空格的路径仍然是一个参数。不涉及 shell。

```toml
[[profiles]]                # commit identities (Settings > Profiles)
id = "work"
name = "Work"
author_name = "Jane Doe"
author_email = "jane@work.example"
```

## `[ui]`

**Log 与图**

| 键 | 默认值 | 取值与作用 |
|---|---|---|
| `ui.graph_renderer` | `"auto"` | `auto`（终端能绘制图片时用图片，否则用字符）、`image`、`text`。**重启。** |
| `ui.image_protocol` | `"auto"` | 图片（图、头像）如何传送到终端：`auto`、`kitty`、`iterm2`、`sixel`、`none`。`auto` = 在 kitty、Ghostty 以及设置了 `kitty_graphics = true` 的 herdr 中使用 Kitty 协议；在 WezTerm、iTerm2、Rio、mintty 中使用 iTerm2 协议；在 Windows Terminal、foot、Konsole、mlterm 中使用 sixel（仅头像）；在 tmux、zellij、screen 和未知终端中不使用。**重启。** |
| `ui.theme` | `"system"` | 配色主题：内置的 slug（`gitpoulpi theme list` 会列出全部）、`<config>/themes/` 中某个文件的 slug，或 `system`（跟随终端的浅色或深色背景，在启动时检测；旧值 `default` 含义相同）。`config set` 会拒绝未知的名称。参见[主题](themes.md)。 |
| `ui.graph_color_theme` | `"theme"` | 车道调色板：`theme`（跟随主题）或任意车道调色板名称：`gitpoulpi`、`light`、`material`、`serie`，以及其他主题的调色板。 |
| `ui.graph_lane_background` | `"off"` | `off` 或 `row_tint`（图后方有一条淡淡的车道颜色带，边缘还有一道竖条）。**应用：切换图行着色**（`Ctrl+N Shift+T`）可切换它；**应用：更改图配色主题…**（`Ctrl+N Shift+Y`）可选择 `ui.graph_color_theme`。 |
| `ui.graph_row_tint_strength` | `"medium"` | `strong`、`medium`、`subtle`（仅配合 `row_tint`）。 |
| `ui.toolbar_position` | `"bottom"` | `top`、`bottom`、`left`、`right`。`left` 和 `right` 绘制为一列；窗口窄于 80 列时回退为 `bottom`。 |
| `ui.toolbar_align` | `"start"` | `start`、`center`、`end`：按钮在工具栏上的位置（一行的左、中、右；一列的上、中、下）。 |
| `ui.toolbar_pr_pinned` | `true` | 拉取请求按钮固定在工具栏的最末端；`false` 则让它与其他按钮一起排列。 |
| `ui.statusbar_position` | `"bottom"` | `top`（仓库标签页上方）或 `bottom`。 |
| `ui.tabs_position` | `"top"` | `top`、`bottom`（窗口最后一行，位于状态栏下方）或 `left`（左边缘的一列；窄于 80 列时仍在顶部）。 |
| `ui.graph_show_stashes` | `true` | Log 中每个 stash 占一行。 |
| `ui.ref_connector` | `true` | 从引用标签到其 commit 的一条细线。 |
| `ui.column_resize` | `"line"` | 在 Log、Radar 以及 diff、文件和 Blame 视图的 commit 列中如何抓取列边界。`line`：每条边界是一个单元格宽的间隙，表头中带一个小刻度；指针位于边界上时（在表头或表格中的任意位置），会在全高范围内画出一条细的强调色线，从线上任意位置拖动都可调整宽度。`header`：没有间隙和刻度；边界之后的表头单元格的第一个字符会亮起，从那里拖动即可调整（仅限表头）。键盘调整键和命令面板中的各行在两种模式下相同。 |
| `ui.ref_tree` | `true` | 在引用侧栏中按 `/` 将分支和 tag 分组为文件夹（`y` 可切换）。 |
| `ui.graph_ghost_ref` | `true` | Log 的 Ref 列会为没有自身 ref 的悬停 commit，在列里有空间时，显示一个淡色药丸，内含它所属分支（否则为远程分支）的名称；绝不显示 tag。已删除的分支由合并它的 merge commit 的标题推断，并以删除线绘制。悬停在药丸上会显示其类型。 |
| `ui.reveal_on_ref_click` | `true` | 在引用窗格中点击某个引用（分支、tag、远程分支、stash）会将 Log 滚动到其 commit 并让该行闪烁；commit 的选择不会移动。 |
| `ui.log_wip_counts_in_message` | `true` | 更改列被隐藏时，在 WIP 消息中显示 WIP 的更改数量。 |
| `ui.log_columns_hidden`, `ui.log_columns_order`, `ui.log_column_widths` | 空 | Log 的列（`ref`、`graph`、`message`、`author`、`date`、`sha`、`changes`），在你隐藏、拖动或调整某一列时写入。`message` 不能被隐藏。 |
| `ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths` | 空 | diff 旁 [Commit 列](interface-tour.md#commit-列)自己的各栏，与 Log 的列相互独立：显示的栏按名称列出（`ref`、`graph`、`message`、`author`、`date`、`changes`、`sha`；为空表示泳道、消息、作者和日期），它们从左到右的顺序，用同样的名称表示（为空表示刚才列出的顺序；漏掉的名称按该顺序排在后面），以及拖动得到的各栏宽度（以单元格计，未列出的栏为自动宽度），在你选择、移动或调整各栏时写入。`message` 不能被隐藏。在这些键出现之前写入的会话文件会被读取一次并导入其中。 |
| `ui.sidebar_sections_hidden` | 空 | 你隐藏的左侧窗格分区（`local`、`remote`、`worktrees`、`stashes`、`tags`、`pull_requests`、`issues`、`submodules`），通过窗格的右键菜单或命令面板写入。至少始终会保留一个存在的分区。 这是默认值：每个仓库可以隐藏自己的分区（保存在该仓库的状态文件中），此键适用于没有自己选择的仓库；*用于所有仓库* 会写入它。 |
| `ui.sidebar_sections_scope` | `"repository"` | 左侧窗格分区由谁决定：`repository`（每个仓库各自保存，`ui.sidebar_sections_hidden` 是默认值）或 `global`（所有仓库都显示 `ui.sidebar_sections_hidden`，更改也写入它）。来回切换不会丢失任何一套：开启 `global` 期间，各仓库自己的选择仍保存在它们的状态文件中。设置：*左侧窗格分区*。 |
| `ui.language` | `"auto"` | 界面语言：`auto`（系统区域设置，否则为英语）、`en`、`fr`、`es`、`pt-BR`、`pt-PT`、`it`、`de`、`zh-CN`、`ja`、`ko`、`ru`、`uk`、`pl`、`cs`、`nl`、`sv`、`tr`。**即时生效。** `GITPOULPI_LANG` 会覆盖它。本构建中没有的语言会显示英语。**应用：更改语言…**（`Ctrl+N g`）也可以设置它，并带有实时预览。CLI 和 MCP 的输出始终是英语。 |
| `ui.translate_git_terms` | `false` | `false`：commit、push、pull、stash、tag…… 在所有语言中都保持英语。`true`：使用所选语言的 git 词汇（法语的 “remiser”“étiquette”“valider”……）。**即时生效。** |
| `ui.git_terms_english` | `false` | 仅限 ja、ko 和 zh-CN：显示英文的 git 词汇（stage、unstage、staged、unstaged……），而不是它们的音译或翻译。`ui.translate_git_terms` 为 `true` 时无效。**即时生效。** |
| `ui.mouse_enabled` | `true` | `false` 会把鼠标（以及文本选择）留给终端。**重启。** |
| `ui.mouse_scroll_lines` | `3` | 滚轮每格滚动的 Log 行数，1 到 20。**重启。** |
| `ui.kitty_keyboard` | `"auto"` | `auto` 会向终端请求 kitty 键盘协议（这样 `Ctrl+,`、`Ctrl+Enter`、`Ctrl+Shift+N` 才能送达）；`off` 从不请求。**重启。** |
| `ui.logo_color` | `""` | “关于”窗口中徽标的配色方案，一个 slug（`violet`、`orange`、`teal`、`ocean`、`magenta`、`candy`、`coral`、`green`、`cyan`、`indigo`、`lavender`、`sun`、`mint`、`slate`）；留空或未知表示使用构建的默认值（`lavender`）。在你点击徽标或在“关于”中选择颜色时写入。 |

**头像**（默认关闭；开启后会把作者邮箱的 SHA-256 哈希发送给 Gravatar，并向你已连接的 GitHub/GitLab 账号查询）

| 键 | 默认值 | 取值与作用 |
|---|---|---|
| `ui.avatars` | `"off"` | `graph`（在图片图的 commit 圆圈中）、`author`（在作者列中）、`off`。在 `ui.avatars` 不存在时，旧的 `ui.show_avatars`（`true`）仍按 `author` 读取。 |
| `ui.avatar_sources` | `"gravatar_hosting"` | 查询顺序：`hosting_gravatar`、`gravatar_hosting`、`gravatar`、`hosting`。GitLab 主机只会通过已连接的配置来查询。 |
| `ui.avatar_fallback` | `"identicon"` | `identicon`（在本地生成）或 `initials`。在没有图片协议的终端上，始终使用首字母徽章。 |

**Diff 与编辑器显示**

| 键 | 默认值 | 取值与作用 |
|---|---|---|
| `ui.diff_layout` | `"inline"` | `inline` 或 `split`（在 diff 中按 `m`）。 |
| `ui.diff_hunks_only` | `true` | `false` 会显示整个文件并在原位显示更改（`H`）。 |
| `ui.diff_show_whitespace` | `true` | 显示仅空白字符的更改（`s`）。 |
| `ui.diff_word_wrap` | `false` | 对过长的 diff 行自动换行（`w`；设置 > Diff 与编辑器）。 |
| `ui.minimap` | `false` | 在 diff 和编辑器的滚动条旁显示缩略图（`M`）。 |
| `ui.mermaid_detail` | `refs` | Log 的“复制为 Mermaid gitGraph”的详细程度：`structure`、`refs`、`subjects`、`full`（会从弹窗中记住；也是 `log --format mermaid` 的默认值）。 |
| `ui.mermaid_orientation` | `lr` | 该图的方向：`lr`（从左到右）或 `tb`（从上到下）。 |
| `ui.mermaid_fence` | `false` | 用 `mermaid` 代码围栏包住所复制的 Mermaid 脚本。 |
| `ui.mermaid_subject_width` | `40` | Mermaid id 中的 commit 标题会截断到这么多个字符（`subjects`、`full`）。 |
| `ui.color_depth` | `"auto"` | `auto`、`truecolor`、`256` 或 `16`：使用多少种颜色绘制。`auto` 为 truecolor，但 Apple Terminal（256）、Linux 控制台（16）以及没有 `COLORTERM` 的 `*-256color` 终端（256）除外。即时生效。参见[主题](themes.md)。 |
| `ui.input_color` | `"theme"` | 文本框背景：`theme`（主题自带的）、`gray`、`blue`、`slate`、`dark`、`custom`。 |
| `ui.input_color_custom` | `"#2a2a2a"` | `custom` 颜色，`#rrggbb`。 |
| `ui.composer_files` | `"expanded"` | AI commit 编辑区打开时，每个方案的文件为 `expanded`（展开）或 `collapsed`（折叠）。 |
| `ui.blame_side` | `"right"` | Blame 视图中注释列（头像、作者、年龄、摘要）所在的一侧：`right`（默认）或 `left`。 |
| `ui.blame_width` | `46` | Blame 注释列的宽度，以单元格计（24 到 90）。拖动其边框，或在视图中按 `<` / `>`；新宽度会保存在这里。 |
| `ui.pr_terminology` | `"auto"` | `auto`（GitHub 上称 Pull Request，GitLab 上称 Merge Request）、`pull_request`、`merge_request`。 |
| `ui.hide_finished_pr_chip` | `true` | `true`：已检出分支最新的 pull request 或 merge request 一旦关闭或 merge，就隐藏工具栏上显示它的徽章；改为显示暗淡的 `+ PR` 徽章（发起新的请求），就像该分支没有请求一样。`false`：徽章始终显示最新的请求，无论其状态如何。 |

你在 diff 中切换的各项设置会立即保存在此处，并用于之后打开的每一个 diff。

**日期**：`ui.date_format`（`"YYYY-MM-DD"`，Blame / 文件历史的日期列）、`ui.date_time_format`（`"YYYY-MM-DD HH:mm"`，Log 的日期列）和 `ui.date_verbose_format`（`"D MMMM YYYY HH:mm"`，commit 详情、日期提示以及*复制 > 日期*）。标记：`YYYY` `YY` `MMMM` `MMM` `MM` `M` `DD` `D` `dddd` `ddd`（星期，全称 / 缩写）`HH` `mm` `ss`；其他内容原样复制。月份和星期的名称跟随 `ui.language`；在日期数字之后书写的月份，在有属格形式的语言（俄语、乌克兰语、波兰语、捷克语）中使用属格。`D MMMM YYYY` 在日语、中文和韩语中按年份在前书写（`2024年3月5日`）。`ui.date_locale`（`"system"` 为默认值，或 `"english"`）会让名称无论使用何种语言都保持英语。

**代理、终端与标签页文件夹**

| 键 | 默认值 | 取值与作用 |
|---|---|---|
| `ui.allow_agent_view_control` | `true` | 窗口是否接受来自代理的 `ui show` / `ui propose`（设置 > 代理与 MCP > *允许 AI 代理控制视图*）。关闭后，没有任何代理能触及它。参见 [MCP 与代理](mcp-and-agents.md#向用户展示内容)。 |
| `ui.launch` | `"off"` | 在没有窗口打开时，代理是否可以打开 GitPoulpi 窗口：`off`、一个预设（`kitty`、`foot`、`alacritty`、`wezterm`、`wezterm-tab`、`ghostty`、`gnome-terminal`、`xdg-terminal-exec`、`windows-terminal`、`windows-terminal-tab`、`tmux`、`zellij`、`cmux`）或 `custom`。 |
| `ui.launch_command` | `""` | `custom` 模板：`{repo}`、`{session}`、`{gitpoulpi}`。会被拆分为参数，不涉及 shell。 |
| `ui.tab_dir_follow` | `"buttons"` | `buttons`（标签页行提供*退出并 cd 到此*）或 `auto`（每次退出都会更改 shell 的文件夹，新标签页会运行默认的*打开方式*条目）。参见[跟随标签页文件夹](following-the-tab-folder.md)。 |
| `ui.terminal_launch` | `"auto"` | 旧版，见下文以及[打开方式](open-in.md)。旧的*在此打开终端*如何打开终端：`auto`（`ui.launch` 预设，否则是 `$TERMINAL`、`xdg-terminal-exec`、Windows Terminal、Terminal）、一个预设，或 `custom`。 |
| `ui.terminal_launch_command` | `""` | `custom` 模板；`{repo}` 是文件夹。 |
| `ui.open_in` | *（不存在）* | [打开方式](open-in.md)列表，由表组成的数组 `[[ui.open_in]]`，含 `label` 和 `command`。不存在表示“尚未配置”：列表由旧的 `ui.terminal_launch*` 键（非 `auto` 的值会成为第一项）、操作系统的文件管理器、从环境中检测到的多路复用器和终端（herdr、tmux、zellij、WezTerm、kitty、Windows Terminal 等）以及 `PATH` 上找到的编辑器计算得出。首次从设置保存时会写入该列表。`open_in = []` 是空列表。无法用 `config set` 设置（请编辑文件）。 |
| `ui.open_in_default` | `""` | “用默认方式打开”所运行条目的标签；留空或未知表示第一项。 |
| `ui.launch_terminal` | `"auto"` | 桌面快捷方式（设置 > 常规 > 启动器）和 Windows 上的重新打开用来启动 GitPoulpi 的终端。`auto` 从已安装的终端中选择（Windows：Windows Terminal，否则依次为 WezTerm、Alacritty、普通控制台；Linux：`xdg-terminal-exec`，否则为找到的第一个终端；macOS：如果安装了 iTerm2、Ghostty、WezTerm 或 kitty 则使用它，否则使用 Terminal），`default` 为系统自身的默认终端，也可以是终端 ID，例如 `windows-terminal`、`wezterm`、`alacritty`、`console`、`kitty`、`foot`、`ghostty`、`gnome-terminal`、`konsole`、`iterm`、`apple-terminal`。 |
| `ui.launcher_build` | `"current"` | 仅限开发模式（见 `general.dev_mode`；只有这时才在设置 > 启动器中显示，否则会被忽略）。`"current"`：快捷方式和命令别名启动创建它们的那个构建。`"newest"`：它们启动检出目录中 `target/dist` 和 `target/release` 里最近构建的那一个，每次点击时决定，并且启动任何一个构建都不会改写它们。 |
| `ui.command_alias` | `""` | 在终端中启动 GitPoulpi 的命令名称，例如 `gp`（设置 > 常规 > 启动器，`gitpoulpi alias create`）。留空：无。只能使用字母、数字、`-`、`_` 和 `.`。它只是 **创建：命令别名** 按钮所用的名称；在你创建之前什么都不存在。 |
| `ui.command_alias_add` | `""` | 第二个命令的名称，例如 `gpa`，它运行 `gitpoulpi --add`：把当前文件夹作为新标签页在已运行的 GitPoulpi 中打开（设置 > 常规 > 启动器，`gitpoulpi alias create gpa --add`）。留空：无。名称规则相同；由 **创建** 按钮与 `ui.command_alias` 一起创建。 |
| `ui.relaunch_in_terminal` | `true` | 仅限 Windows：当 exe 通过双击在自己的控制台中启动时，GitPoulpi 会在 `ui.launch_terminal` 中重新打开并退出。关闭：留在该控制台中。 |
| `ui.reuse_window` | `true` | 点击 GitPoulpi 快捷方式（`gitpoulpi launch`）时，显示已打开的 GitPoulpi 的窗口，而不是再启动一个。 关闭：每次点击都会启动新窗口。 |

**`ui.open_in` 命令**会先被拆分为参数（空白、`'...'`、`"..."`），然后在每个参数内部替换变量，因此带空格的文件夹仍然是一个参数；除非命令自己启动了 shell（`sh -c '... "$GITPOULPI_CWD"'`），否则不涉及 shell。变量：`${cwd}`（标签页的文件夹）、`${repo}`（仓库根目录）、`${name}`（文件夹的基本名称）、`${branch}`（分离 HEAD 时为空）、`${web_url}`（`origin` 远程的网页，当它是 GitHub 或 GitLab 项目时；否则为空；使用它的条目在其为空时会拒绝运行）、`${env:NAME}`（未设置时为空）。旧的 `{repo}` 仍表示 `${cwd}`。任何其他 `${...}` 都是错误，并会指出其名称。示例：

```toml
[ui]
open_in_default = "Herdr: new tab"

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

旧版：`ui.terminal_launch` 和 `ui.terminal_launch_command` 仍会被读取（在 `ui.open_in` 不存在时，它们用于生成列表：`custom` 会变成 “Terminal (custom)”，其中的 `{repo}` 会被改写为 `${cwd}`，预设 id 会变成对应的预设），但已被 `ui.open_in` 取代。

### 图像

| 键 | 默认值 | 含义 |
|---|---|---|
| `ui.image_view` | `true` | 在 Diff 视图和文件视图中，将属于图像的二进制文件（PNG、JPEG、GIF、WebP、BMP 等）显示为图片。关闭：显示普通的二进制提示。 |
| `ui.image_view_max_mib` | `32` | 大于此值（MiB）的图像不会被解码；改为显示信息卡片。 |
| `ui.image_view_max_unit` | `"auto"` | 设置中显示该上限所用的单位：`mib`、`gib`（1024 MiB）或 `auto`。仅影响显示：上限始终是 `image_view_max_mib` MiB，范围 1 到 1024。 |
| `ui.image_view_text_fallback` | `"halfblock"` | 在没有图像协议的终端上：`halfblock`（彩色半块字符）或 `card`（仅信息卡片）。 |
| `ui.image_view_pixelated_from` | `"2x"` | 从该缩放比例起，放大时显示方形像素而不是平滑处理：`1x`、`2x`、`never`。 |
| `ui.image_view_sync` | `true` | 更改前和更改后的窗格共用同一个缩放比例和滚动位置。 |
| `ui.image_editors` | *（不存在）* | “编辑方式”图像编辑器，由表组成的数组 `[[ui.image_editors]]`，含 `label` 和 `command`。不存在表示“尚未配置”：列表是在这台机器上检测到的编辑器（`PATH` 上的程序、Flatpak 安装、Windows 上已知的安装文件夹、macOS 上的 `/Applications`），然后是操作系统的默认应用。首次从设置保存时会写入该列表。`image_editors = []` 是空列表。无法用 `config set` 设置（请编辑文件）。 |
| `ui.image_editor_default` | `""` | “用默认编辑器编辑”所运行编辑器的标签；留空或未知表示第一项。 |

**`ui.image_editors` 命令**会先被拆分为参数（空白、`'...'`、`"..."`），然后在每个参数内部替换变量，因此带空格的路径仍然是一个参数；不涉及 shell。变量：`${file}`（图像的绝对路径）、`${dir}`（它所在的文件夹）、`${name}`（它的文件名）、`${repo}`（仓库根目录；临时副本时为空）、`${env:NAME}`（未设置时为空）。任何其他 `${...}` 都是错误，并会指出其名称。不属于工作区文件的图像（某个 commit 的版本、索引、已删除的文件）会作为**只读临时副本**打开，因此编辑器无法触及仓库。没有命令行的编辑器（Microsoft Store 版的 Affinity Photo、某些 Photoshop 安装）不在预设中：请编写自定义条目。示例：

```toml
[ui]
image_editor_default = "GIMP"

[[ui.image_editors]]
label = "GIMP"
command = "gimp ${file}"

[[ui.image_editors]]
label = "Paint.NET"
command = "\"C:\\Program Files\\paint.net\\paintdotnet.exe\" ${file}"
```

**尚未使用**（会被接受和保存，但没有任何东西读取它们）：`ui.graph_colorization`（已从设置中移除；加载时仍被接受）、`ui.date_word_format`（已从设置中移除；加载时仍被接受）。旧的 `ui.diff_view_mode` 会被一次性读取为 `diff_layout` / `diff_hunks_only`。

## `[editor]`

| 键 | 默认值 | 取值与作用 |
|---|---|---|
| `editor.tab_size` | `4` | 每个制表位的列数，1 到 8（编辑器和 diff）。 |
| `editor.eol` | `"auto"` | 内置编辑器写入的行尾：`auto`（保持文件原有的）、`lf`、`crlf`。 |
| `editor.syntax_highlighting` | `true` | 编辑器中的语言配色。 |
| `editor.line_numbers` | `true` | 编辑器和 diff 中的行号。 |
| `editor.follow_selection` | `true` | 编辑器打开时，在右侧窗格中选择其他文件也会在编辑器中打开它，并会先询问是否保存未保存的更改。无法编辑的文件会在其常规视图中打开；某个 commit 的文件则会离开编辑器，显示其 diff。 |

## `[merge]`

merge 冲突编辑器（[解决 merge 冲突](merge-conflicts.md)）。设置 > Diff 与编辑器中会显示它。

| 键 | 默认值 | 含义 |
|---|---|---|
| `merge.side_by_side_min_width` | `100` | 编辑器区域的宽度（以列计），达到该宽度时 A 和 B 并排显示（长行可横向滚动：Shift+滚轮、Shift+方向键）。更窄时，一次只显示一侧，并用标签页切换。`40` 是最小值，`0` 表示始终并排显示（40 列以下的提示仍会保留）。编辑器打开时会隐藏引用面板和状态窗格，所以它的区域通常就是整个窗口。 |

## `[git]`、`[credentials]`、`[ssh]`

| 键 | 默认值 | 取值与作用 |
|---|---|---|
| `git.pull_mode` | `"merge"` | Pull（按钮、`Ctrl+G p`、`Git：Pull`）的行为：`merge`（可能时快进）、`ff_only`、`rebase`。始终显式传给 git。旧的 `fetch_all` 按 `merge` 读取。 |
| `git.push_behavior` | `"direct"` | `direct` push 到默认远程；`ask` 会先显示远程菜单。 |
| `git.default_encoding` | `"utf-8"` | **不是**有效 UTF-8 的文本（文件内容、blame、diff、没有 `encoding` 头的 commit 消息）所用的编码：任何 WHATWG 标签（`windows-1252`、`shift_jis` 等）。仅用于显示；stage 时绝不会重新编码。仓库可以覆盖它。 |
| `git.lfs_enabled` | `true` | 显示并驱动 Git LFS。仓库可以覆盖它。参见 [Git LFS](lfs.md)。 |
| `git.lfs_autofix_filters` | `true` | 在对使用 LFS 的仓库执行 pull 或 push 之前，如果缺少 LFS 过滤器 / pre-push 钩子，则安装它们（`git lfs install --local`，仅限此仓库）。参见 [Git LFS](lfs.md)。 |
| `git.autocrlf`, `git.eol` | `"unset"`, `"native"` | *未使用*：仓库的 `core.autocrlf` / `core.eol` 是 git 自己的设置，在设置 > Git 与编码（或 `gitpoulpi repo-config`）中编辑。 |
| `credentials.https_mode` | `"credential_helper"` | fetch、pull、push、clone 的 HTTPS 认证：`credential_helper`（git 的）、`prompt`（忽略凭据助手；git 在终端中询问，因此在后台守护进程中会失败）、`none`（仅限公开远程）。旧的 `encrypted_file` 值按 `credential_helper` 读取。 |
| `ssh.key_type` | `"ed25519"` | *生成密钥对*所生成的类型：`ed25519`、`rsa`（4096 位）、`ecdsa`（NIST P-256）；文件名为 `gitpoulpi_<type>`。SSH 本身始终使用你的 ssh-agent 和 ssh 配置。 |
| `ssh.private_key_path`, `ssh.public_key_path` | 未设置 | 由*生成密钥对*记录。设置了 `ssh.private_key_path` 时，fetch、pull、push 和 clone 会以 `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes` 运行 git；私钥路径为空时，`ssh.public_key_path` 起同样的作用（ssh 会选取匹配的 agent 密钥）。环境中的 `GIT_SSH_COMMAND`/`GIT_SSH`，或 git 配置中的 `core.sshCommand`（按仓库固定，`gitpoulpi ssh-command set --key`）优先。每次传输之前都会读取，所以更改会应用于下一次传输。 |

## `[[integrations]]`（托管平台与议题跟踪器）

由设置 > 集成或 `gitpoulpi hosting integrations add` 写入。每一种类型都有客户端（仅限个人令牌，不支持 OAuth）。

```toml
[[integrations]]
id = "corp"                         # what --id takes; the token is stored under integration:corp
kind = "gitlab_self_managed"        # github | github_enterprise | gitlab | gitlab_self_managed | ...
url = "https://gitlab.example.com"  # the API root is derived from it
token_ref = "integration:corp"      # a reference, never the token
name = "Corp GitLab"                # optional display name
account = "me@example.com"          # optional, see below
enabled = true                      # false: ignored for host matching
```

自托管服务器可以使用纯 http：`url = "http://localhost:3000"` 会按输入原样保留（协议和端口会用于每个请求，该主机上的远程也会匹配此配置）。通过纯 http 发送的令牌不加密传输，因此只在本地或可信网络中使用 http。

`kind` 还接受 `azure_devops`、`azure_devops_server`、`bitbucket`、`bitbucket_data_center`、`jira_cloud`、`jira_data_center`、`trello`、`gitea`、`forgejo`、`linear`、`youtrack`、`redmine` 和 `gerrit`（Gitea 和 Forgejo 共用一个客户端；Forgejo 配置默认使用 `https://codeberg.org`）。Jira、Trello、Linear、YouTrack 和 Redmine 仅是议题跟踪器；Gerrit 仅用于评审（它的变更就是 pull request，没有议题）。Bitbucket Cloud 没有议题跟踪器。

`account` 是令牌旁边的非机密身份信息，对大多数类型为空并省略：`jira_cloud` 和 `bitbucket` 的 Atlassian 账号邮箱（Bitbucket 的账号为空表示使用工作区或仓库访问令牌，以 Bearer 方式发送），`trello` 的 API 密钥，`gerrit` 的用户名（令牌是它的 HTTP 密码，以 HTTP basic 方式发送）。令牌本身仍是唯一的机密。对于 `azure_devops`，`url` 是 `https://dev.azure.com/<organization>`；`https://<organization>.visualstudio.com` 形式的 URL 会被读作同一个组织。对于 `azure_devops_server`，它是集合根，例如 `https://tfs.example.com/tfs/DefaultCollection`。

`user`（已连接的账号）和 `ssh`（该配置的密钥选择）由程序为你写入。令牌来自 `GITPOULPI_TOKEN_<ID>`（id 转为大写，非字母数字字符换成 `_`），其次是已保存的令牌，再其次是你的凭据助手为该主机提供的令牌（仅当只有一个配置使用该主机时；仅限 GitHub 和 GitLab 配置：其他类型从不读取凭据助手）。详情：[托管平台与议题跟踪器集成](hosting-integration.md)。旧的 `[[hosting]]` 表*未使用*。

对于 `local_markdown`（以一个 Markdown 笔记文件夹作为议题来源），`url` 是该文件夹，相对于每个仓库或为绝对路径，且没有令牌。它可选的 `[integrations.local]` 表设置 `statuses`（看板的列顺序，默认 `todo`、`doing`、`done`）、`done_status`（默认 `done`）、`reference_prefix`（`LOC` 会显示为 `LOC-12`）和 `me`（“指派给我”指的是谁）。见[本地议题与 Obsidian](local-issues.md)。

### `[integrations.radar]`（账户范围的列表）

代码托管平台配置的可选表（GitHub、GitLab、Gitea、Forgejo、Azure DevOps、Bitbucket Data Center、Gerrit；不含 Bitbucket Cloud）。省略表示所有键都关闭。
由 `gitpoulpi hosting integrations radar` 写入；由 `gitpoulpi hosting radar` 和 Radar 读取，并可在设置 > 集成中编辑（已连接配置的 Radar 块）。

```toml
[integrations.radar]
review_requests = true    # requests waiting for my review, every repository
open_requests = false     # every open request of my namespaces
include_drafts = false    # keep draft requests in these lists
assigned_issues = true    # open issues assigned to me, every repository
namespaces = ["acme"]     # orgs / groups to search for open requests
review_mode = "ask"       # ask | clone | api
```

| 键 | 默认值 | 含义 |
|---|---|---|
| `review_requests` | `false` | 请求你评审的未关闭请求，不论仓库是否已克隆 |
| `open_requests` | `false` | 你自己的命名空间以及你所属的每个组织或群组（或 `namespaces`）中所有未关闭的请求，绝不是“令牌能看到的一切” |
| `include_drafts` | `false` | 除非开启，否则这些列表不含草稿请求 |
| `assigned_issues` | `false` | 指派给你的未关闭议题，所有仓库 |
| `namespaces` | `[]` | 为 `open_requests` 搜索的组织 / 群组；留空表示你自己的命名空间加上你所属的每一个（GitHub：最多搜索 12 个；GitLab：10 个群组和 10 个自己的项目；Gitea、Forgejo 和 Gerrit：10 个；Azure DevOps 填项目名，Bitbucket Data Center 填项目键，Gerrit 填项目路径前缀） |
| `review_mode` | `"ask"` | 评审仓库未克隆的请求时怎么做：`ask`、`clone`（先克隆，再评审）或 `api`（通过托管平台的 API 评审） |

答复按配置缓存在 `config.toml` 旁边的 `radar-account.json` 中，保留五分钟；该文件只是缓存，可以放心删除。

## `[mcp]`（AI 代理可以做什么）

| 键 | 默认值 | 取值与作用 |
|---|---|---|
| `mcp.allow_mutations` | `false` | 关闭：会更改任何内容的 MCP 工具都会被拒绝；读取、`ui_*` 和提议仍可使用。`true` 允许代理更改仓库。 |
| `mcp.mutation_allowlist` | `[]` | 非空：只有这些工具名称可以更改内容（`["stage", "commit"]`）。 |
| `mcp.allow_ui` | `true` | `false`：拒绝会移动你窗口的 `ui_*` 工具（`ui_state` / `ui_viewers` 除外）。 |
| `mcp.ui_control` | `"ask"` | `ask`（窗口显示一个由你确认的标签）、`auto`（除非你正忙，否则直接应用）、`off`。服务器命令上的 `--ui-control` 会按代理覆盖它。也适用于 `gitpoulpi ui show`。 |
| `mcp.allow_ui_propose` | `false` | 允许 `ui_propose_*` / `gitpoulpi ui propose`：预先填好的对话框，由你自己确认。 |
| `mcp.allow_history_purge` | `false` | 仅存在于 `config.toml`（设置中没有对应行）。`true` 会向代理列出工具 `rewrite_purge`（仍需要 `allow_mutations`）。关闭时，代理根本看不到这个工具：清除会永久删除历史重写留下的旧历史、它的撤销记录及其 reflog。CLI 和 TUI 不受它限制。 |
| `mcp.enabled` | `true` | `false`：`gitpoulpi mcp serve` 和 `gitpoulpi-mcp` 拒绝启动，并给出点名此键的提示。设置 > 代理与 MCP > *MCP 服务器*。在代理启动服务器时读取。 |
| `mcp.bind` | `"127.0.0.1:8808"` | 未指定时 `--http` 监听的地址（`--http ADDR` / `--bind` 优先）。除非使用 `--allow-remote`，否则仅限回环地址。留空或旧的 `"stdio"` 表示默认值。设置 > 代理与 MCP > *MCP 服务器端口* 编辑端口并保留主机。 |

`allow_mutations`、`mutation_allowlist` 和 `allow_ui` 只限制 MCP 服务器；CLI 是你自己的 shell，不受它们限制。参见 [MCP 与代理](mcp-and-agents.md#权限)。

## `[ai]`（内置 AI，默认关闭）

| 键 | 默认值 | 取值与作用 |
|---|---|---|
| `ai.enabled` | `false` | 总开关。关闭时，任何地方都不会显示 AI 内容，也不会发送任何内容。 |
| `ai.default_profile` | `"default"` | 操作未指定配置时所使用的配置 id。 |
| `ai.global_instructions` | `""` | 添加到每个操作提示词中的文本。可以使用 `${repo}`、`${uilang}` 等（参见 [AI > 提示词变量](ai.md#提示词变量)）；按操作的指令和仓库指令同理。 |
| `ai.actions.<action>.profile` | `""` | 某个操作使用的配置；留空 = 默认配置。操作：`commit_message`、`explain_changes`、`stash_message`、`pull_request_description`、`commit_composer`、`conflict_resolve`、`code_review`、`split_commit`、`rewrite_script`、`rewrite_rules`。工作空间（`workspaces.toml`，`[workspace.ai_instructions]` 和 `[workspace.ai_overrides.<action>]`）和仓库都可以追加或替换这些指令：见 [AI 辅助](ai.md#指令)。 |
| `ai.actions.<action>.instructions` | `""` | 仅为该操作添加的文本。 |
| `ai.review.run_in_repository` | `"profile"` | `profile`、`yes` 或 `no`：代码评审草稿的本地代理本次运行是否在仓库内工作，覆盖 AI 配置的 `workdir`。仓库可以设置自己的值（`gitpoulpi ai review-context --repo`）。见 [AI > 评审上下文](ai.md#评审上下文)。 |
| `ai.review.include_claude_md` | `true` | 将仓库的 `CLAUDE.md` 提供给代码评审草稿（根目录，在请求所针对的分支上读取；在 8 KB 处截断）。 |
| `ai.review.include_agents_md` | `true` | `AGENTS.md` 同理。 |
| `ai.review.extra_file` | `""` | 代码评审草稿的另一个文件：相对于仓库根目录的路径、绝对路径，或指向代码托管平台上某文件的链接（通过匹配的集成配置读取，否则匿名读取）。所有文件合计在 16 KB 处截断。 |

仓库自己的 `[preferences.ai_review]` 表（在其 `repo-state.toml` 中；未克隆仓库的条目在 `remote-reviews.toml` 中）接受相同的键，全部可选，另加 `no_extra_file = true`：此仓库不使用额外文件，不论 `ai.review.extra_file` 是什么。未设置的键、未设置的 `extra_file` 和缺失的 `no_extra_file` 都继承全局值；把 `extra_file` 设为某个文件会清除该开关，反之亦然。

```toml
[[ai.profiles]]
id = "default"
name = "Default"
kind = "open_ai"        # open_ai | anthropic | open_router | open_ai_compatible | local_agent
model = "gpt-5"         # empty = the kind's default
base_url = ""           # empty = the kind's default (Ollama: http://localhost:11434/v1)
token_ref = "ai:default"  # where the API key is stored; never the key
command = ""            # local_agent only: the exact program ("claude", an alias, a wrapper, a path)
harness = ""            # local_agent only: claude | codex | gemini | opencode | cursor-agent | copilot | pi | qwen | aider | goose | crush | amp | other; empty = inferred from `command`
args = []               # local_agent only, e.g. ["-p", "--model", "haiku"]
run_mode = "headless"   # local_agent only: headless (default) | terminal (the agent's interactive screen in a virtual terminal)
terminal_args = []      # local_agent only: the arguments of the terminal mode (args keeps the headless set)
workdir = "default"     # local_agent only: default | dedicated | repository (default = dedicated folder in terminal mode, GitPoulpi's folder in headless mode)
timeout_secs = 0        # local_agent only: seconds before the agent is stopped; 0 = 120 headless, 180 terminal

[ai.profiles.env]       # local_agent only, optional: variables set for that process only
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"   # ${env:NAME} = one of GitPoulpi's own variables
```

`harness` 决定设置 > AI 提供的建议（无头参数、选项、模型标志和列表、环境变量），与 `command` 无关；两个键为空时都会从文件中省略。环境变量的值绝不会被记录到日志，当它们看起来像机密时，在设置中会被隐藏。

设置 > AI 为 `harness`/`command`/`args`/`env`（常见的代理 CLI，附带其无头标志、模型标志和有用的环境变量）以及 `kind`/`base_url`/`model`（托管和本地的 HTTP 提供方）提供了预设；参见 [AI > 配置](ai.md#配置-ai)。

没有配置时，会创建一个名为 *Default* 的配置。旧的 `[[ai.presets]]` / `[ai.defaults]` 文件会在加载时迁移，并保留在 `[[ai.legacy_presets]]` 中。其余内容：[AI 辅助](ai.md)。

## `[repos]`、`[session]`

| 键 | 默认值 | 取值与作用 |
|---|---|---|
| `repos.clone_root` | `""` | 克隆对话框的起始位置；留空 = `%USERPROFILE%\source\repos` 或 `~/src`。 |
| `repos.browse_root` | `""` | “浏览”打开的位置；留空 = 上次选择的文件夹，然后是主目录。 |
| `repos.show_hidden` | `false` | 文件夹选择器显示隐藏文件夹（`Alt+H` 切换并保存）。 |
| `repos.show_siblings` | `false` | 文件夹选择器的左侧树还会以真实层级列出每一级已展开目录中的其他文件夹（`Alt+S` 切换并保存）。 |
| `session.per_repo_state_dir` | `"in_git_dir"` | GitPoulpi 在哪里保存它对某个仓库的记忆（引用筛选、折叠状态、个人的工作流覆盖项、托管平台缓存）：`in_git_dir` = `<git 目录>/gitpoulpi`（所有 worktree 共用，永远不会出现在 `git status` 中），`in_worktree` = `<仓库>/.gitpoulpi`（团队提交 `workflow.toml` 的唯一位置），`user_data` = `<操作系统数据文件夹>/repos/<name>-<id>`（Windows 为 `%APPDATA%\GitPoulpi\data`，Linux 为 `$XDG_DATA_HOME/gitpoulpi`，macOS 为 `~/Library/Application Support/GitPoulpi`）。旧值 `".gitpoulpi"` 仍表示 `in_worktree`。更改它不会把数据搬回去：在旧的 `<仓库>/.gitpoulpi` 中找到的数据，会在第一次需要时被移到所选位置。 |

## `[log]`（应用程序日志）

活动日志“应用程序”标签页显示的内容，也是“复制诊断信息”引用的内容。详情：[出错时](troubleshooting.md#出错时)。

| 键 | 默认值 | 取值与作用 |
|---|---|---|
| `log.level` | `"info"` | 记录的最详细的行：`error`、`warn`、`info`、`debug` 或 `trace`。活动日志中的级别框会修改它（并保存到这里）。 |
| `log.retention_days` | `7` | 保留日志文件的天数，含当天：1 到 3650（十年）（更大或更小的值会被限制到该范围）。无论设置用什么单位来显示，它始终是一个天数。更早的文件会在 GitPoulpi 启动时删除。一天的文件还受 `log.max_file_mb` 限制，所以最坏情况约为 `2 x max_file_mb x retention_days`。 |
| `log.retention_unit` | `"auto"` | 设置中显示 `log.retention_days` 所用的单位：`days`、`weeks`、`months`、`years` 或 `auto`。仅影响显示，绝不会改变文件保留多久。一周是 7 天，一个月是 30 天（不是日历月，所以无论何时应用，天数都相同），一年是 365 天。`auto`（以及在这个键出现之前写成的配置）会选出能整除天数的最大单位（14 天显示为“2 周”，45 天显示为“45 天”）。不能整除天数的单位（`retention_days = 10` 配 `weeks`）会被忽略，仍显示天数。设置会同时写入两个键；旧版 GitPoulpi 会忽略单位，仍然读取天数。 |
| `log.max_file_mb` | `5` | 一天的文件达到这个大小时会轮转为一个 `.old` 备份，所以一天的日志占用不会超过它的两倍。 |
| `log.max_file_unit` | `"auto"` | 设置中显示该大小所用的单位：`mib`、`gib`（1024 MiB）或 `auto`。仅影响显示：大小始终是 `max_file_mb` MiB，范围 1 到 1024。 |
| `log.timings` | `true` | 活动日志的“Git 操作”标签页显示每个操作花了多长时间（`Push - 123 ms`：打开仓库、刷新、fetch、pull、push、commit、checkout、merge、rebase、stash、clone）。耗时由窗口测量，从发出请求到守护进程应答，因此包含 IPC 往返。关闭后会隐藏耗时以及成功操作的条目。失败始终会列出。 |
| `log.extended` | `false` | “扩展日志”：“Git 操作”标签页还会列出后台操作（轮询状态的定期刷新、自动 fetch）及其耗时。它们始终以 `debug` 级别写入应用程序日志。 |
| `log.commit_count` | `"system-git"` | 状态栏的 `已加载 / 总数` 如何得出总数。`"system-git"` 运行 `git rev-list --count`（一个进程，在普通机器上最快：git 会读取 commit-graph 文件）。`"libgit2"` 在进程内用 libgit2 的 revwalk 统计，不启动任何进程：当每个命令行进程启动都很慢时选它，典型原因是开启实时扫描的杀毒软件会检查每一个进程；对 50,000 个 commit，它在后台大约需要 200 毫秒，而不是 12 毫秒，但不会启动进程。`"off"` 从不统计：状态栏只显示已加载的 commit。立即生效。带路径筛选的统计始终使用 system git。 |
| `log.use_mailmap` | `true` | 在 Log、blame 和文件历史中，按仓库的 `.mailmap`（以及 `mailmap.file`）所指定的名字显示每位作者：守护进程向 git 请求 `%aN`/`%aE`（system 后端），或自行应用同样的文件（libgit2 后端），因此两者结果一致。已存储的 commit 不会改变，也不会重写任何内容。按作者搜索 Log 时匹配的是显示出来的名字。立即生效；Log 会在 2 秒的轮询内重新读取。 |

文件是操作系统状态文件夹中的 `gitpoulpi-YYYY-MM-DD.log`（Linux 上是 `~/.local/state/gitpoulpi/logs`；Windows 和 macOS 上是 `<cache>\logs`；使用 `GITPOULPI_CONFIG_DIR` 时是 `<config>/cache/logs`）。令牌、密码和 `Authorization` 头在写入任何内容之前都会被隐去。

## `[search]`（工作空间搜索）

`gitpoulpi search`（及其 MCP 工具）会查找的范围。详情：[CLI 中的工作空间与搜索](cli-reference.md#工作空间与搜索)。

| 键 | 默认值 | 取值与作用 |
|---|---|---|
| `search.commit_limit` | `200000` | 搜索索引在内存中为每个仓库保留其消息的 commit 数（从新到旧，覆盖所有分支、远程分支和 tag），每个约 100 字节加上消息本身。更长的历史会在此处截断，答复中会标明 `partial`。`0` 会关闭 commit 搜索（分支、tag、stash 和 worktree 仍会被搜索）。实时读取：下一次搜索就会遵循更改。 |

## `[rewrite]`（历史重写）

`gitpoulpi rewrite` 和历史重写工作台所保护的内容。详情：[重写历史](history-rewrite.md)。

| 键 | 默认值 | 取值与作用 |
|---|---|---|
| `rewrite.protected_branches` | `["main", "master", "develop", "release/*"]` | 已经 push 过的分支（名称，或用 `*` 匹配任意文本的模式）会被重写绕开，除非本次运行明确列出它们（`--include-protected BRANCH`；push 时用 `--include-all-protected`；工作台中每个分支一个复选框）。它们会与仓库的 `workflow.toml` 标记为 `protected` 的基础分支合并计算。重写之后的 push 同样会跳过它们。这是 GitPoulpi 自己的防护：强制 push 是否被接受，仍由托管平台的分支保护决定。立即生效：下一次预览就会遵循修改。 |
| `rewrite.external_scripts` | `["shell"]` | 脚本规则可以使用哪些外部脚本语言：`"shell"`（bash 或 sh；PowerShell 脚本用 pwsh 或 powershell）、`"python"`、`"lua"`（已安装的 Lua；内嵌的 Lua 始终可用，无需列出）的任意组合。**外部脚本没有沙箱隔离：它以你的权限运行。** 未列出的语言在工作台里是灰色的，CLI、MCP 和守护进程会拒绝它（`scripts_disabled`）。即使已列出，来自仓库或共享方案的外部脚本也要在你信任之后才会运行（`gitpoulpi rewrite script trust FILE`，或工作台里的提问）；信任以脚本内容的哈希为键，所以脚本一旦改动就会再次询问。要启用 Python，请编辑 `config.toml`（列表无法用 `config set` 设置）：在 `[rewrite]` 下写 `external_scripts = ["shell", "python"]`。立即生效。 |
| `rewrite.script_timeout_ms` | `5000` | 外部脚本回答一个 commit 的时限。超时后整个进程树会被终止，重写失败（`script_failed`，并指出该 commit）；此时没有任何东西被移动。`0` 视为默认值。立即生效。 |
| `rewrite.script_timeout_unit` | `"auto"` | 设置中显示该时限所用的单位：`milliseconds`、`seconds` 或 `auto`。仅影响显示：时限始终是 `script_timeout_ms` 毫秒。设置接受 1 到 600000（十分钟）；`0` 仍视为默认值。立即生效。 |
| `rewrite.script_total_timeout_s` | `3600` | 外部脚本一次完整遍历的总时限。`0` 视为默认值。立即生效。 |
| `rewrite.script_total_timeout_unit` | `"auto"` | 设置中显示该时限所用的单位：`seconds`、`minutes`、`hours` 或 `auto`。仅影响显示：时限始终是 `script_total_timeout_s` 秒。设置接受 1 到 86400（一天）；`0` 仍视为默认值。立即生效。 |

## `[updates]`（查找更新的发布版本）

标题栏的版本标记如何处理更新的发布版本。详情：[更新](installation.md#更新)。

| 键 | 默认值 | 取值与作用 |
|---|---|---|
| `updates.mode` | `"notify"` | `"notify"` 每天在后台向 GitHub 询问一次是否有更新的发布版本，并把版本标记变成 `vX.Y.Z 可用`。`"auto"` 在此构建来自发布压缩包时，还会自行下载并暂存更新（重启前仍会询问）；对其他任何构建，其行为与 `"notify"` 相同。`"off"` 从不发出请求。立即生效。 |
| `updates.channel` | `"stable"` | `"stable"` 只考虑正式发布的版本；`"prerelease"` 还会提供候选版本（`-rc.1`）。立即生效。 |
| `updates.repo` | `"Poulpi-Lab/GitPoulpi"` | 检查其发布版本的 github.com 上的 `owner/name`。在该仓库为私有期间，除非为 github.com 存储了令牌（设置 > 集成），否则 GitHub 会返回 404；没有令牌时检查会悄悄失败，并向应用程序日志写入一行。 |

## `[keybindings]`

重新绑定 TUI 的快捷键；设置 > 键盘与鼠标可编辑它。

```toml
[keybindings.global]
git_prefix = "Ctrl+B"            # default Ctrl+G
nav_prefix = "Alt+n"             # default Ctrl+N
settings   = ["Ctrl+,", "F2"]    # a list = alternatives
tab_menu   = []                  # unbound

[keybindings.git_prefix]         # the key pressed after the git prefix
stage_file = "z"
```

`[keybindings.<context>] <action> = "<chord>"`；设置的值会替换该操作的默认值，删除该行即可恢复默认值。组合键写法：`Ctrl+Shift+N`、`Alt+n`、`F2`、`PageDown`、`Space`、`,`、`P`（= `Shift+p`）。错误的条目绝不会阻塞任何东西：TUI 会忽略它，保留默认值，并在状态栏上说明。上下文、操作 id、默认值和冲突规则：[键盘快捷键](keybindings.md)。

## 按仓库的设置：`.gitpoulpi/repo-state.toml`

保存在仓库内部；该文件夹会忽略自身（一个内容为 `*` 的 `.gitignore`），所以它不会显示为更改。由 TUI 和 CLI 写入；你很少需要编辑它。

| 键 | 它是什么 | 设置方式 |
|---|---|---|
| `[preferences] encoding` | 在此仓库中覆盖 `git.default_encoding` | 设置 > Git 与编码 |
| `[preferences] lfs_enabled` | 在此仓库中覆盖 `git.lfs_enabled` | 设置 > Git 与编码 |
| `[preferences] profile` | 在此仓库中为 commit 署名的 `[[profiles]]` id；不存在表示使用 `general.active_profile` | 工具栏的配置下拉框，**仓库：更改配置文件…** |
| `[preferences] autocrlf`, `eol` | *尚未使用* | 设置 > Git 与编码 |
| `[preferences] push_after_commit` | 每次 commit 后把分支 push 到它的上游（不存在时为关闭） | 设置 > 仓库专属设置 > Commit |
| `[preferences] skip_hooks_by_default` | commit 表单的“跳过钩子”默认勾选（不存在时为关闭） | 设置 > 仓库专属设置 > Commit |
| `[preferences] squash_merge` | 本地分支的 merge 改为 squash：只暂存，不生成 merge commit（不存在时为关闭） | 设置 > 仓库专属设置 > Merge 选项 |
| `[preferences] template_apply`, `template_strip_comments` | 用模板预填空的 commit 表单；commit 时移除 `#` 注释行（不存在时两者都为关闭） | 设置 > 仓库专属设置 > 消息模板 |
| `[preferences] conflict_detection` | 此仓库的主动冲突检测；不存在表示跟随 `general.conflict_detection` | 设置 > 仓库专属设置 > 冲突预防 |
| `[preferences] conflict_base_branches` | “基础分支”模式，每行一个（不存在时为 `**`） | 设置 > 仓库专属设置 > 冲突预防 |
| `commit-template.txt`（`repo-state.toml` 旁边的文件） | commit 模板；git 的 `commit.template` 指向它 | 设置 > 仓库专属设置 > 消息模板 |
| `[preferences.ai_instructions]` | `text`，`mode = "add" \| "replace"`：此仓库的 AI 指令 | 设置 > 仓库专属设置 > AI |
| `[preferences.ai_overrides.<action>]` | 此处某个操作的 `profile`、`instructions` | 设置 > 仓库专属设置 > AI |
| `default_remote` | 一键 Push/Pull/Fetch 所使用的远程 | 远程菜单，`gitpoulpi remote default <name>` |
| `hosting_integration` | 固定在此处的 `[[integrations]]` 配置 | `gitpoulpi hosting use --id` |
| `pulls_source`, `issues_source` | PR / 议题列表所读取的提供方项目，形式为 `{ integration, project, filter }`。`filter` 是可选的：提供方原生的附加子句（Jira JQL 片段、Azure WIQL 条件、Trello 列表名称），以 AND 方式附加到列表查询上；GitHub 和 GitLab 会忽略它。 | 侧栏分区，`gitpoulpi hosting link` |
| `ref_visibility` | 隐藏 / 单独显示的引用 | 引用侧栏，`gitpoulpi refs hide\|show\|solo` |
| `folds`, `collapsed_*_ref_folders` | 折叠的侧栏分区以及分支和 tag 文件夹。`[folds]` 还包含 `status_folders`（unstaged / staged 文件树中折叠的文件夹）和 `commit_folders`（某个 commit 已更改文件树中折叠的文件夹，对每个 commit 都相同），各自最多 500 个路径，超出时先丢弃最旧的 | 侧栏、文件树 |
| `pending_auto_stash` | GitPoulpi 创建但尚未放回的 stash | 由守护进程写入；参见[自动 stash](log-and-history.md#自动-stash) |

## 其他环境变量

| 变量 | 作用 |
|---|---|
| `GITPOULPI_CONFIG_DIR` | 配置文件夹（见上文）。 |
| `GITPOULPI_TOKEN_<ID>` | 配置 `<ID>` 的托管平台令牌，优先于已保存的令牌（CI、一次性使用）。 |
| `GITPOULPI_GIT` | 要运行的 `git` 可执行文件（默认：`PATH` 上的 `git`；在 Windows 上是 Git for Windows 的 `cmd\git.exe` 垫片背后真正的 `mingw64\bin\git.exe`，这样每次调用可节省一个进程）。 |
| `GITPOULPI_LANG=fr` | 界面语言，优先于 `ui.language`（任何 `ui.language` 的值；`qps` 是供翻译人员使用的伪区域设置）。未知或空值会被忽略。 |
| `GITPOULPI_TERMINAL_IMAGE=off` | 无论配置如何，都完全不显示图片。 |
| `GITPOULPI_TERMINAL_CELL_PX=9x21`, `GITPOULPI_TERMINAL_SUPERSAMPLE=1..4` | 图片分辨率提示（单元格的像素尺寸；过采样，默认 2，`1` 可节省 CPU）。 |
| `GITPOULPI_NO_BROWSER=1` | 从不启动网页浏览器（仍会显示 URL）。 |
| `GITPOULPI_TIMING=1` | 将请求耗时写入临时文件夹中的 `gitpoulpi-timing.log`（用于性能报告）。 |
| `GITPOULPI_NO_EXIT_PAUSE=1` | TUI 因错误退出后，不等待按下 Enter。 |
