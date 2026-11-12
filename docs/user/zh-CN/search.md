[English](../search.md) · [Čeština](../cs/search.md) · [Deutsch](../de/search.md) · [Español](../es/search.md) · [Français](../fr/search.md) · [Italiano](../it/search.md) · [日本語](../ja/search.md) · [한국어](../ko/search.md) · [Nederlands](../nl/search.md) · [Polski](../pl/search.md) · [Português (Brasil)](../pt-BR/search.md) · [Português (Portugal)](../pt-PT/search.md) · [Русский](../ru/search.md) · [Svenska](../sv/search.md) · [Türkçe](../tr/search.md) · [Українська](../uk/search.md) · **简体中文**

# 搜索

`Ctrl+Alt+F`（或 `Ctrl+P`，再选 *搜索：工作空间…*）会打开一个模糊查找器，覆盖**你正在处理的所有仓库**：有当前[工作空间](workspaces.md)时是它的仓库，否则是已打开的标签页。**搜索**按钮（标题栏中齿轮左侧的放大镜及其标签）也能打开它。它可以找到：

- **仓库**（先按名称，再按路径），
- **分支**（本地和远程跟踪分支）和 **tag**，
- **worktree**（先按文件夹，再按路径和分支），
- **stash**（按消息），
- **commit**（按标题；只有当标题不匹配且你输入了三个或更多字符时，才会搜索 commit 的正文），
- **pull request 或 merge request** 和**议题**（按标题、`#编号`、作者和分支）。

![在两个已打开的仓库上弹出的搜索窗口：匹配“main”的分支、一个 stash 和一个 commit，每一行右侧是所属仓库](../images/workspace-search.png)

结果按类型分组，每组内最佳匹配在前。每一行有该类型的图标、高亮的匹配字符、一段变暗的详情（分支的上游、请求的作者和分支、stash 的位置），右侧是仓库名称，因此两个仓库都有 `main` 分支时也不会混淆。当某个仓库仍在为它的 commit 建立索引时，标题中会出现一个旋转图标（在大型仓库中第一次搜索可能要稍等片刻）；其他类型会立即出现，commit 则在准备好后加入。

## 前往结果

| 按键 | 作用 |
|---|---|
| `Up` / `Down`、`PageUp` / `PageDown`、`Home` / `End` | 在结果中移动（分组标题会被跳过）。 |
| `Enter` | 前往该结果：打开（或切换到）该仓库的标签页，然后显示该结果。 |
| `Esc` | 关闭搜索。 |
| `Alt+1` .. `Alt+8` | 开启或关闭某个类型（列表上方的标签，按其顺序）。`Alt+0` 全部开启。 |

“前往”的含义：

| 结果 | Enter |
|---|---|
| 分支或 tag | 在引用窗格中选中它，并在 Log 中选中它的 commit。 |
| commit | 在 Log 中选中它；当它比已加载的内容更早时，会先加载更多 Log。 |
| stash | 选中 STASHES 分区中它的行。 |
| worktree | 以标签页打开它。 |
| pull/merge request、议题 | 在侧栏中选中它的行（在该仓库的标签页中）。 |
| 仓库 | 切换到它的标签页（未打开时先打开）。 |

`Ctrl+N Backspace`（或命令面板中的 *返回*）会回到原来的位置，就像 AI 代理替你做的任何视图更改一样。前往某处不会检出、合并或更改任何内容。

用鼠标时：点击一行会选中它，双击前往，点击标签会开启或关闭该类型，滚轮滚动，点击窗口之外则关闭。

## 用前缀缩小范围

以前缀开头可只搜索一种类型，不受标签的开关影响：

| 前缀 | 类型 |
|---|---|
| `r:` | 仓库 |
| `b:` | 分支 |
| `t:` | tag |
| `w:` | worktree |
| `s:` | stash |
| `c:` | commit |
| `pr:`（或 `mr:`、`p:`） | pull 或 merge request |
| `i:` | 议题 |
| `#` | pull/merge request 和议题（`#12` 找到编号 12） |

`fix: login` 是普通查询，不是前缀：只有单个字母（或 `pr:` / `mr:`）加冒号，或开头的 `#`，才算前缀。

## 匹配方式

与命令面板相同的三个层级：你的文本原样匹配，然后是原样出现在任意位置，最后是其字符按顺序出现但不相邻。更靠前的匹配和更短的文本优先。不区分大小写。

## Pull request 和议题

它们来自 GitPoulpi 已为每个仓库加载的内容（侧栏中 PULL REQUESTS 和议题分区背后的列表），保存在仓库的 `.gitpoulpi/` 文件夹中一个小的缓存文件里。搜索**绝不会调用 GitHub 或 GitLab**：从未在窗口中加载过请求的仓库就没有可找的内容，你看到的内容与侧栏上一次刷新时一样新。没有关联账户时就没有可找的内容，也不需要配置任何东西。

## 在命令行中

`gitpoulpi search <query> [--workspace ID] [--kind K]... [--limit N]` 做同样的事并输出 JSON（参见 [CLI 参考](cli-reference.md#工作空间与搜索)）；MCP 服务器有一个 `search` 工具，因此 AI 代理能找到你能找到的东西。[设置](config-reference.md#search工作空间搜索)中的 `search.commit_limit` 限定每个仓库索引多少个 commit（`0` 会关闭 commit 搜索）。

## 更改按键

*搜索：工作空间…* 是一个命令面板命令，按键对应的是 `global.workspace_search` 动作：可在“设置 > 键盘与鼠标”中重新绑定，或在 `[keybindings.global]` 表中（`workspace_search = "Ctrl+Alt+F"`）。
