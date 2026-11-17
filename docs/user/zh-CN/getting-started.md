[English](../getting-started.md) · [Čeština](../cs/getting-started.md) · [Deutsch](../de/getting-started.md) · [Español](../es/getting-started.md) · [Français](../fr/getting-started.md) · [Italiano](../it/getting-started.md) · [日本語](../ja/getting-started.md) · [한국어](../ko/getting-started.md) · [Nederlands](../nl/getting-started.md) · [Polski](../pl/getting-started.md) · [Português (Brasil)](../pt-BR/getting-started.md) · [Português (Portugal)](../pt-PT/getting-started.md) · [Русский](../ru/getting-started.md) · [Svenska](../sv/getting-started.md) · [Türkçe](../tr/getting-started.md) · [Українська](../uk/getting-started.md) · **简体中文**

# 快速上手

本页带你从全新安装一路走到第一次 commit。[界面导览](interface-tour.md)为窗口的每个区域命名。[设置](settings.md)列出了每个选项。

## 1. 安装并启动

按照[安装](installation.md)操作。然后在 Git 仓库中打开终端并运行：

```
gitpoulpi-tui
```

不带子命令的 `gitpoulpi` 效果相同。GitPoulpi 会打开包含当前文件夹的仓库。如果不在仓库内，它会打开**主页**标签页，你可以在那里浏览、克隆或初始化仓库（参见[第 7 步](#7-打开克隆和初始化仓库)）。

图标需要 Nerd Font 或带 emoji 的字体。如果你看到的是方框，请参见[故障排除](troubleshooting.md)。

![主窗口：左侧是引用侧栏，中间是 Log 图，右侧是工作更改和 commit 表单](../images/hero.png)

## 2. 学会三个键，其余皆通

| 按键 | 打开 |
|---|---|
| `?` | **帮助**：每个按键和鼠标手势。输入文字即可筛选。`Ctrl+D` 会在浏览器中打开本文档。 |
| `Ctrl+P` | **命令面板**：输入以查找命令，`Enter` 运行。每一行都显示其快捷键。 |
| `Ctrl+,` | **设置**。标题栏中 × 左侧的齿轮同样可以打开。 |

你可以全程使用键盘或鼠标。面板可以触达每条命令。参见[快捷键](keybindings.md)和[鼠标与手势](mouse-and-gestures.md)。

![按 stage 一词筛选的命令面板，列出 stage 命令及其快捷键](../images/palette.png)

## 3. 查看历史

窗口中间是 **Log**：每个 commit 一行，最新的在最上面。它以彩色标记显示分支和 tag，同时显示图、消息、作者、日期和短哈希。

- 点击一个 commit。其详情和更改的文件会出现在右侧。
- 点击一个更改的文件。其 diff 会在中间打开。`Esc` 将其关闭。
- `j` / `k` 或方向键移动选择。`f` 搜索 commit。
- 右键点击 commit 可打开其菜单。

![选中一个 merge commit 的 Log：右侧是它的消息、作者以及它新增的两个文件](../images/log-graph.png)

## 4. 修改并 stage

在编辑器中编辑一个文件。**Log 的顶行**现在会显示工作更改：一条虚线行，带有已更改文件的数量。点击它。右侧窗格会显示 **unstaged 的文件**和 **staged 的文件**。

- 悬停在一个文件上并点击它的 **stage 文件**按钮。或者点击**全部 stage**。
- 点击文件名打开它的 diff。在那里你可以 stage 一个 **hunk**（**stage Hunk**）或一个**行**。

参见[stage 与 commit](staging-and-committing.md)。

![已修改文件的 diff，每个 hunk 上方都有 stage Hunk 和丢弃 Hunk 按钮](../images/diff.png)

## 5. commit

在右下角的 commit 表单中输入摘要，也可以再写描述。然后点击绿色的 **Commit N 个文件的更改**按钮或按 `Ctrl+Enter`（也可以在任何位置按 `Ctrl+G Enter`）。勾选**修订上一个 commit**，可把你的更改加入上一个 commit，而不是新建 commit。

如果你开启了 [AI](ai.md)，摘要旁边的 `✦` 按钮会根据已 stage 的更改撰写消息。在你点击**Commit N 个文件的更改**之前，GitPoulpi 绝不会 commit。

## 6. 分享你的工作

工具栏有 **Pull** 和 **Push** 按钮，各带一个 `▾` 列表。在键盘上，按 `Ctrl+G` 再按一个字母：`Ctrl+G p` 执行 pull，`Ctrl+G Shift+P` 执行 push。按下 `Ctrl+G` 后，状态栏会列出各个字母。

GitPoulpi 使用你的 `git` 已在使用的认证方式（credential helper、ssh-agent），自身不需要账号。若要查看和创建 pull request，请先连接一次托管平台账号。参见[托管平台与议题跟踪器](hosting-integration.md)。

## 7. 打开、克隆和初始化仓库

点击标签行最左端的房子图标，或按 `Ctrl+O`。**主页**标签页会列出你已打开、收藏和最近使用的仓库，并带有**浏览**、**克隆**和**初始化**按钮。每个仓库在各自的标签页中打开。`Ctrl+W` 关闭当前标签页。

![主页标签页列出已打开和最近使用的仓库，并带有浏览、克隆和初始化按钮](../images/home.png)

**克隆**会询问 URL 和文件夹，或让你从已连接的账户（GitHub、GitLab、Gitea、Forgejo、Azure DevOps、Bitbucket、Gerrit）中选择仓库（参见[从已连接的账户克隆](hosting-integration.md#从已连接的账户克隆)）。**初始化**会把一个文件夹变成新仓库。
在这三个对话框中（以及它们打开的文件夹选择器中），`Ctrl+N` 会在每个字段和按钮上显示一个徽标：按下该字符即可把光标放进该字段，或按下该按钮。

| ![克隆对话框：克隆到哪里、URL、文件夹名，以及取消和克隆按钮](../images/clone.png) | ![初始化仓库对话框：文件夹、名称和初始分支](../images/init.png) |
|---|---|

### 稀疏检出

大型仓库不必完整落到磁盘上。在“克隆”对话框中勾选 **稀疏检出（仅部分目录）**，并输入你想要的目录，用空格或逗号分隔（`src docs/guide`）。只有这些目录和仓库根目录下的文件会被检出；完整历史仍会下载，因此 Log、blame 和分支照常可用。勾选期间至少需要一个目录。

![输入了两个目录的“稀疏检出”对话框，以及“取消”“关闭”“应用”按钮](../images/sparse-checkout.png)

任何仓库之后都可以切换：打开命令面板（`Ctrl+P`）并运行 **仓库：稀疏检出…**，或使用仓库标签页菜单中的 **稀疏检出…**。对话框在一个字段里列出当前已检出的目录：在那里添加或删除名称，然后按 **应用**（`Enter`），或按 **关闭（完整检出）** 找回所有文件。工作区处于稀疏状态时，状态栏会显示一个小小的 `稀疏` 徽标。同样的控件也是 *设置 > 稀疏检出* 中的一页（一个每行一个路径的列表，带有 **开始**、**停止** 和 **刷新** 按钮）。

说明：

- 这是 git 的 cone 模式（按目录，而不是按模式）；`*`、`..` 和绝对路径会被拒绝。
- 撤销不会记录目录集合的更改：不会丢失任何内容，**关闭** 会恢复所有文件。如果某个目录里有尚未 commit 的更改，git 可能拒绝将其移出（错误显示在状态栏）；请先 commit 或 stash。
- 脚本和代理也能做同样的事：`gitpoulpi clone <url> --sparse src docs` 以及 `gitpoulpi sparse-checkout status | set | add | disable`（参见 [CLI 参考](cli-reference.md)）；MCP 服务器提供同样的工具。

## 下一步

- [界面导览](interface-tour.md)：你所见一切的名称。
- [使用 Log](log-and-history.md)和[stage 与 commit](staging-and-committing.md)：日常工作。
- [主题](themes.md)和[设置](settings.md)。
- [AI 辅助](ai.md)和[托管平台与议题跟踪器](hosting-integration.md)：两个可选的集成。
- [CLI 参考](cli-reference.md)和[MCP 与代理](mcp-and-agents.md)：用脚本和代理完成同样的操作。
- [故障排除](troubleshooting.md)。
