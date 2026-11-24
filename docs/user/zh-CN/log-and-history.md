[English](../log-and-history.md) · [Čeština](../cs/log-and-history.md) · [Deutsch](../de/log-and-history.md) · [Español](../es/log-and-history.md) · [Français](../fr/log-and-history.md) · [Italiano](../it/log-and-history.md) · [日本語](../ja/log-and-history.md) · [한국어](../ko/log-and-history.md) · [Nederlands](../nl/log-and-history.md) · [Polski](../pl/log-and-history.md) · [Português (Brasil)](../pt-BR/log-and-history.md) · [Português (Portugal)](../pt-PT/log-and-history.md) · [Русский](../ru/log-and-history.md) · [Svenska](../sv/log-and-history.md) · [Türkçe](../tr/log-and-history.md) · [Українська](../uk/log-and-history.md) · **简体中文**

# 使用 Log

Log 是 GitPoulpi 的中心：整个历史以图的形式呈现，每个分支、tag 和 stash 都在其上。本页介绍如何阅读它、在其中查找内容，以及你可以对 commit 和引用做的一切。各区域的名称见[界面导览](interface-tour.md)；确切的按键见[快捷键](keybindings.md)，鼠标手势见[鼠标与手势](mouse-and-gestures.md)。

![选中了一个 merge commit 的 Log：其详情和文件显示在右侧](../images/log-graph.png)

## 阅读图

每一行是一个 commit，最新的在前。圆点位于该 commit 的泳道上；线条把 commit 连到它的父 commit；merge 显示为两条线汇合。左侧的标签以泳道的颜色标出该 commit 上的分支和 tag，并用一条细线连到圆点。笔记本图标表示本地分支，地球表示远程分支，`✓` 表示你已检出的分支。`+N` 表示还有更多引用指向该 commit；悬停可列出它们，或使用下面的菜单。

第一行是**工作更改**（虚线圆圈：你未 commit 的工作，连到它所基于的 commit）。stash 显示为泳道上的小方块，位于它们所基于的 commit 旁边。

### 颜色、行着色和头像

泳道颜色来自主题。*设置 > 外观 > 图配色主题*可将任何主题与任何泳道配色组合，而命令面板的 *应用：更改图配色主题…* 选择器（`Ctrl+N Shift+T`）会实时预览每种配色；参见[主题](themes.md)。把*图泳道背景*设为**行着色**，会在图列后面绘制一条淡淡的泳道颜色色带，并在其边缘绘制一条细条。*行着色强度*（**强**、**中**（默认）、**弱**）决定色带的淡化程度。

| 使用行着色的 Serie 泳道配色 | 使用行着色的 Material 泳道配色 |
|---|---|
| ![Serie 配色主题下带行着色的 Log 图](../images/theme-serie.png) | ![Material 配色主题下带行着色的 Log 图](../images/theme-material.png) |

*作者头像*（图中圆点里或作者列中的图片）默认关闭。开启它会把每位作者邮箱地址的哈希发送给 Gravatar，并询问你的托管平台。同一页面上的*头像来源*和*头像回退*决定图片来自哪里，以及没有图片时绘制什么。

如果你的终端无法显示图片，图会用字符绘制。*图渲染器*和*图像协议*可强制使用其中一种；参见[故障排除](troubleshooting.md)。

所选或悬停行的高亮从 commit 的圆圈开始。其左侧的泳道保持正常背景。

### 列

右键点击标题以选择列：**引用**、**图**、**消息**（始终开启）、**作者**、**日期**、**更改**和 **SHA**。可选的**更改**列会为每个 commit 显示 `n files +added -removed`。拖动两个标题之间的边框可调整列宽；该选择会被记住，单位是终端单元格。默认的 `ui.column_resize = "line"` 下，边框是列末尾的那个单元格宽的间隙：表头行中有一个小刻度停在其中，指针在间隙上时（在表头或表格中的任意位置），会在表格全高范围内画出一条细的强调色线，从线上任意一点拖动都可调整宽度（在支持的终端中，指针会变成调整大小的指针）。设为 `"header"` 时没有刻度和线：边框之后的表头单元格的第一个字符会在悬停时亮起，拖动从它开始，只能在表头行进行（设置 > Log 与图 > 列宽调整）。两种模式下各行外观相同：间隙属于它左侧的那一列，所以选中行和行底色会穿过它。调整窗口大小或最大化时，引用、图等列保持原有宽度：只有**消息**列会变宽或变窄（窗口非常窄时，其他列会暂时让出空间，先从图开始，窗口恢复后再回到你设定的宽度）。把“图”列拖到最小宽度，即一个 commit 圆点，图就会变为紧凑模式：每个 commit 都在同一条泳道上，一个接一个，没有分支线和 merge 线。再宽一格，树就回来了。图中的 commit 圆圈里显示作者头像时（设置 > 外观 > 作者头像：图），圆圈占两个单元格，所以这一列会提前一格（三个单元格）停下，绝不会裁掉头像。

拖动表头时，会有一个名称的虚影跟随指针，并在该列将要落下的位置显示一条强调色竖条；松开时顺序才会改变，按 `Esc` 取消。右键点击打开的选择器是一个复选清单（`[x]` / `[ ]`），标题中带 ✕；也可以从命令面板打开（`视图：Log：选择要显示的列`），用方向键移动，用 `Space` / `Enter` 切换。

用键盘时，按 `Shift+C`（或命令面板的 `视图：Log 列：选择要移动或调整宽度的列`）选择一列，其表头单元格会亮起。然后 `Tab` / `Shift+Tab` 选择其他列，`<` / `>` 在已显示的列之间将其左移或右移一位（相当于拖动其表头），`]` / `[` 加宽或收窄它（相当于拖动其边界；最后一列移动的是它的左边缘），`0` 把所有宽度恢复为自动，`Esc` 或 `Enter` 退出。期间方向键仍用于在 commit 间移动。顺序和宽度会像拖动一样被记住，并且属于 Log 自己：diff 旁边的 commit 列有自己的另一份。

*按内容调整。*在某列的边界上**双击**，会把其左侧的列调整为其最宽的可见内容和表头所需的宽度（宽字符和 emoji 按其占用的单元格数计算）。在适应窗格布局中，Log 始终填满窗格，所以一列最多只能到窗格容得下的宽度，即**消息**列保留其份额、其他每列保留最小宽度之后所剩的空间：此时状态栏会显示 `作者需要 34 格，当前只容得下 20 格。`键盘上同样可行：选中一列（`Shift+C`、`Tab`）后，`=` 调整该列，`+` 调整所有列；命令面板有 `视图：Log 列：按内容调整该列宽度` 和 `视图：Log 列：按内容调整所有列宽度`；右键列选择器会在复选框下方列出 `按内容调整<列>宽度`（指针下的那一列）、`按内容调整所有列宽度` 和 `重置各列宽度`。**消息**是伸缩列，没有可供调整的自有宽度：它占用其他列留下的空间。

*横向滚动。*默认每列都适应窗格。在**横向滚动**布局中（选择器的 `横向滚动（各列保持宽度）` 复选框、设置 > Log 与图 > 列布局、命令面板的 `视图：Log：切换各列的横向滚动`，或选中某列时按 `s`），各列保持宽度，**消息**列也是（它在这里有自己的宽度，调整它会取其最宽的摘要），窗口变窄时不会让出任何空间，表格可以比窗格更宽。底部边框上会出现一个滚动条：拖动滑块，点按空白轨道可让滑块移到指针下，或使用两端的箭头。键盘上 `Shift+Left` / `Shift+Right` 每次滚动四格（命令面板有 `视图：Log：向左滚动各列` / `向右滚动各列`）；表格上的横向滚轮或 `Shift`+滚轮效果相同，选中某列（`Shift+C`、`Tab`）会把它滚动到可见位置。比窗格窄的表格会在右侧留出空白，选中行的色带一直延伸到边框。每种布局各自保存宽度（`ui.log_column_widths` 和 `ui.log_column_widths_scroll`）；在横向滚动布局中第一次拖动、按键或调整时，会把每一列按其当前宽度存下，因此只有你改动的那一列会变化。在适应窗格布局中，若按内容调整后放不进窗格，会给出提示并指向此布局。

*最后一列的把手。*最后一列也有把手，位于它自己的最后一个单元格（不是窗格的边框，后者仍用于调整窗格大小）：拖动时它按指针移动的距离变化，由**消息**列补上差值。**SHA** 默认宽 8 格（7 位数字加上那个把手单元格）。

![显示了“更改”列的 Log：每个 commit 列出其文件数以及添加和删除的行数](../images/log-changes-column.png)

### 作者与 `.mailmap`

当仓库有 `.mailmap`（git 的文件，为用过多个身份提交的人指定唯一的姓名和邮箱）时，Log 的**作者**列、commit 详情、Blame 和文件历史会像 `git log` 一样显示**规范**身份，commit 本身不会被改动。设置 `log.use_mailmap`（设置 > 日志与图，默认开启）可以关闭它，以显示存储的名字。按作者搜索时匹配的是显示出来的姓名和邮箱，所以搜索规范姓名就能找到这个人的每个 commit。历史工作台的**身份**标签页会列出历史中的姓名和邮箱，并替你写入 `.mailmap` 行，见 `history-rewrite.md`；在终端里：`gitpoulpi mailmap list|check|add`。

## 查找内容

- **搜索**（`f`、工具栏的**搜索**按钮，或命令面板的 *导航：搜索 Commit…*）：匹配整条 commit 消息、作者姓名和邮箱、哈希以及引用名称。它会将不匹配的内容变暗，并保持图的形状。`Enter` / `Shift+Enter` 跳到下一个 / 上一个匹配项。
- **作者筛选**（**作者**表头中的漏斗、Log 中的 `Shift+A`，或命令面板的 *Log：按作者筛选…*）：按 commit 的作者预先筛选 Log。表头下方会弹出一个窗口，里面有搜索框、一个**清除筛选**行（✕），以及已加载历史中每位作者各一行：头像（`ui.avatars` 未关闭时：终端能绘制图像且图片已就绪时是作者的图片，在此之前以及在其他终端上是首字母徽标）、姓名，后面是淡色的邮箱，按 commit 数量从多到少，再按姓名排序。输入时用命令面板的模糊匹配缩小列表；`Enter` 或点击会切换高亮的作者（打勾；可选任意多位），`Esc` 或点击别处会关闭弹出窗口并保留筛选。该筛选的行为与搜索完全相同：所选作者的 commit 保持高亮，其他所有行变暗，图的形状保持不变，不会移除或重新获取任何内容。搜索栏同时打开时两者都生效（commit 必须由所选作者提交**并且**匹配文本），`Enter` / `Shift+Enter` 会在这些命中项之间跳转。筛选开启时，漏斗会变成填充的强调色 `▼`，其工具提示列出所选作者；当作者列被隐藏或窄得放不下漏斗时，Log 的上边框会改为显示 *作者：Alice Example +1*（按键和命令面板仍然有效）。要取消它：用**清除筛选**行、右键点击亮起的漏斗、命令面板的 *Log：清除作者筛选*，或在 Log 中按 `Esc`（若有选区，`Esc` 先取消选区；在弹出窗口内，`Esc` 先清空搜索框，再关闭弹出窗口并保留筛选）。筛选开启期间，Up/Down（以及 PageUp/PageDown、Home/End）只会在高亮的 commit 之间移动选择，跳过变暗的行；点击仍可选中任意一行。作者是 Log 已加载的 commit 的作者（应用了 mailmap，与列中一致），该筛选按仓库记住，切换标签页和重启后依然有效。文本搜索不会被记住，筛选会。
- **引用过滤器**（侧栏中的 `/`）：按名称缩小分支、tag 和远程列表。
- **单独显示**（`o` 键，或某个引用、文件夹或远程的菜单）：只显示被单独显示的引用的 commit；单独显示多个则看它们的并集。`Shift+O` 停止单独显示。
- **隐藏**某个引用（通过它的菜单）可将其从图中移除：它的标签和只有它能到达的 commit 都会消失（已检出的 commit 始终保留，工作树行也是）。引用标题中的*已隐藏 n*会统计它们，**显示全部**（`Shift+H`）会把它们带回来。状态栏的总数统计的是可见的历史。

## 分支、tag 和 引用侧栏

双击本地分支或 tag 可将其检出。右键点击某个引用（在侧栏中，或 Log 里它的标签）可打开它的菜单。最有用的条目：

| 条目 | 作用 |
|---|---|
| 在图中定位 | 将 Log 滚动到该引用的 commit（必要时先加载更早的历史）并闪烁该行；选择保持不变。除非在设置中关闭了*点击引用时在图中定位*，否则点击引用的效果相同。被图隐藏（或被单独显示排除）的引用会改在状态栏中说明 |
| 检出 | 切换到该分支，或在该 tag 处分离 |
| Merge、Rebase 到 | 把它 merge 到已检出的分支，或把已检出的分支 rebase 到该引用上 |
| Fast-forward `<branch>` 到 `<current>` / `<current>` 到 `<branch>` | 在可 fast-forward 时前移分支；否则为灰色并给出原因（`diverged, not a fast-forward`） |
| 从…创建 worktree | 在它自己的标签页中打开一个关联的 worktree |
| Pull、Push、Fetch、<分支> 的远程… | 针对该分支的网络操作（<分支> 的远程… 会设置 upstream，并为每个远程提供 Push 和 也 Pull 复选框） |
| 在图中隐藏 / 显示、单独显示 | 仅限显示；它们隐藏或单独显示引用（隐藏还会去掉只有该引用能到达的 commit），而不触碰仓库 |
| 重命名、删除 | 删除会先询问；远程的对应分支也可以删除 |
| 复制分支名、复制远程上分支的链接 | 复制到剪贴板 |

**移动 tag。**把一个 tag 拖到另一个引用上（见[鼠标与手势](./mouse-and-gestures.md)中的*拖动引用*），或使用 `引用：将所选 Tag 移到所选 Commit…`。它保留自己的类型和注释，只有你本地的 tag 会移动（在你 push 之前，远程仍保留旧位置），一次撤销即可放回原处。CLI：`gitpoulpi tag move <name> <target> [--force-non-ff]`、`gitpoulpi tag merge-branch <name> <branch> [-m <message>]`。

**tag 在哪里。**一个 tag 可以在这台电脑上、在某个远程上，或两者都有，侧栏的 TAGS 行（以及 Log 中该 tag 的标签）会在名称后面用一个标记表示是哪种情况。把鼠标悬停在 tag 上，可以看到同样的文字说明。

| 标记 | 含义 |
|---|---|
| 向上箭头 | 仅在本地：没有任何远程有它。它的菜单提供*Push* |
| 左右箭头 | 本地和某个远程上都有，位于同一个 commit |
| 向下箭头（该行变暗） | 仅在某个远程上：本仓库中没有（从未 fetch，或已在本地删除）。它的菜单提供*从 `<remote>` Fetch `<tag>`* 和*在 `<remote>` 上删除 `<tag>`…*；命令面板中有 `Git：Fetch 所选 Tag` |
| 上下箭头 | 某个远程上也有，但位于不同的 commit（普通 push 会被拒绝）。它的菜单会为每个这样的远程提供两行：*采用 `<remote>` 的 `<tag>`*（本地 tag 移到远程的版本，依据上次 fetch，不联网；附注 tag 会变成同一个 tag 对象），以及红色的 *将 `<tag>` 强制 Push 到 `<remote>`*（会先询问；以上次 fetch 所见的内容作租约，所以此后已移动的远程会被拒绝并提示“fetch first”，不会覆盖任何内容）。各为一个撤销条目。命令面板中有 `Git：采用所选 Tag 的远程版本` 和 `Git：将所选 Tag 强制 Push 到其远程`（有多个远程有差异时用默认远程）；CLI：`tag take-remote`、`tag force-push` |

git 不为 tag 保存远程跟踪引用，所以 GitPoulpi 是根据你上一次 **fetch**（任何一次 fetch，包括自动 fetch）以及它自己的 tag push 和删除，得知每个远程上有什么的。浏览时不会访问网络；如果别人改动了某个远程上的 tag，请 fetch 来查看。同样的信息也在 `gitpoulpi tag list` 中（字段 `local`、`remote`、`differs_on`、`where`）。

**删除 tag。**当某个远程有该 tag 时，红色的**删除**会打开一个对话框，里面有三个带编号的行（`1`-`9`，或用上/下键和 `Enter`；`Esc` 取消），高亮的那一行会在下方自行说明：

1. **在本地删除**：只从本仓库移除。它立即执行，并且消息会说明远程仍有它。**下一次 fetch 会把它带回来**（git 会跟随远程的 tag），除非你也在远程上删除它。
2. **在 `<remote>` 上删除**：对所有人移除它（`git push <remote> --delete`）。它会要求一次点名该远程的确认。你本地的 tag 保留。这无法撤销。
3. **在本地和 `<remote>` 上都删除**：经过同样的确认后两处都删，这样 fetch 就无法把它带回来。**撤销只恢复本地 tag**（同一个 tag 对象）；之后它又变成仅在本地，*Push* 可以把它放回远程。

![删除 tag 对话框，有“仅在本地删除”“在 origin 上删除”“在本地和 origin 上都删除”三行，下方是 refs 窗格的 Tags 部分，每个 tag 都带有标记，表明它只在本地还是也在 origin 上](../images/tag-delete.png)

有多个远程时，每个远程各有自己的行。只在本地的 tag 保持普通确认；只在一个远程上的 tag 会直接进入该远程的确认。命令行：`gitpoulpi tag delete <name> [--local | --remote <r> | --both [--remote <r>]]`。

**Fetch 与 tag。**fetch 也会刷新每个远程上有什么（即那些标记），而 git 自己的 tag 跟随机制仍然适用：只在本地删除的 tag 会被再次 fetch 回来。被别人**在远程上删除**的 tag，在任何一次 fetch 之后都会从标记的远程一侧消失，但仍作为你的本地 tag 留在这里（现在是*仅在本地*），因为 git 默认不清理 tag。**设置 > 同步 > 自动清理 tags**（默认关闭；`gitpoulpi fetch --prune-tags`）也会删除这样的本地 tag，但只限曾经在远程上的：你从未 push 过的 tag，或你移动过的 tag，绝不会被碰。需要 git 2.41 或更新版本才能得知哪些被清理了。

**新建分支或 tag。**工具栏的**分支**和 **Tag** 按钮、`Ctrl+G b` 和 `Ctrl+G t`，或 commit 的*在此创建分支…*，会在引用列中目标 commit 所在行的内联框里询问名称。`Enter` 创建，`Esc` 取消，git 会拒绝的名称会以红色提示，且不关闭该框。

对于没有 ref 的 commit，把指针移到它的行上时，Ref 列也会显示一个淡色药丸，内含它所属分支（否则为远程分支）的名称，由历史推断得出；tag 绝不显示为虚影。合并后被删除的分支仍会以删除线显示，名称取自 merge commit 标题中的名称（`Merge branch 'x'`、`Merge pull request #N from owner/x`……）；squash merge、rebase 和快进合并不会留下这样的痕迹。悬停在药丸上会显示其类型。它不会创建任何东西：请使用上面的创建分支命令。设置 > 日志与图 中的 **在悬停的 commit 上显示虚影分支** 可将其关闭；Ref 列太窄的地方不会绘制。

**远程。**REMOTE 标题中的 `+` 添加一个；右键点击远程的名称可 fetch 它、复制其网页链接、将其设为默认远程、在图中隐藏它、编辑它的 URL 或移除它。**隐藏远程**（其行的 ⊙ 标记、菜单中的*在图中隐藏远程*，或命令面板中的 `远程：在图中隐藏远程…`）会隐藏它现有和之后获得的每个分支，只有它能到达的 commit 也会从 Log 中消失：用于浏览某一段历史，而不掺入无关的归档远程的历史；用*在图中显示远程*把它带回来。显示远程时，它那些被逐个隐藏的分支也会一并显示。被隐藏远程的分支若点击其 ⊙ 标记，状态栏会说明原因：请先显示该远程。

**Stash。**工具栏的 **Stash** 会 stash 所有内容（包括未跟踪文件）；`Ctrl+G m` 会先询问消息（开启 AI 时，在该提示中按 `Ctrl+G` 会给出建议）。右键点击 Log 或 STASHES 分区中的 stash 行，可使用 **Pop**、**Apply**（保留该 stash）或 **Drop**。

**Worktree。***从…创建 worktree*（在分支、tag 或 commit 上）会打开一个对话框，建议仓库旁边的一个文件夹、一个可选的新分支名，以及“将新 worktree 作为标签页打开”。worktree 出现在 WORKTREE 分区中；双击其中一个即可打开，当 worktree 被移除时，它的标签页会自动关闭。整个功能（在其他工具中打开、合并 worktree 的分支、移除）见 [Worktrees](worktrees.md)；同一窗格的 SUBMODULES 分区见 [Submodules](submodules.md)。

## 把引用拖到 commit 上

你可以按住某个分支（在侧栏中，或 Log 里它的标签）并拖动。一个幽灵标签会跟随指针，有效目标会亮起；松开即可得到菜单：

- 一个本地分支**拖到另一个本地分支上**：Merge、Rebase、交互式 rebase、Fast-forward；
- 一个本地分支**拖到远程分支上**：Push 到该远程；
- 一个远程分支**拖到本地分支上**：Merge、Rebase、Fast-forward、Pull。

`Esc` 取消拖动。键盘上的等价操作是命令面板命令 *引用：将所选引用放到已检出分支上…*。

**Merge 还是 fast-forward。**当目标没有自己的 commit（被拖动的分支只是领先于它）时，merge 本可以直接把目标向前移动。默认情况下（设置 > Git > **Merge fast-forward** = *never*，或仓库在其 Merge 选项页中的自有选择），菜单行显示 *将 x merge 到 y（merge commit）* 并生成真正的 merge commit，让历史保留该分支；想要单纯的移动时，请用单独的 *将 y fast-forward 到 x* 一行（想要一条直线时，请改用 *Rebase*）。选择 *allowed* 时，Merge 行显示 *（fast-forward）*，选择 *only* 时显示 *（仅 fast-forward）*，分支已分叉时会置灰并说明原因。分支自己的菜单中有同样的行和标签，`gitpoulpi merge --ff | --no-ff | --ff-only` 可按单次 merge 指定。

## commit 菜单

右键点击某个 commit（或在其上按 `.`）打开它的菜单：

![commit 菜单：cherry-pick、revert、reset、编辑消息、用 AI 解释、交互式 rebase、创建分支、tag、检出、worktree、补丁和复制操作](../images/context-menu.png)

| 条目 | 作用 |
|---|---|
| **Cherry-pick 到 `<branch>`** | 在已检出的分支上重放该 commit |
| **Revert commit…** | 创建一个撤销它的 commit。对 merge commit，会先弹出一个下拉框询问哪个父 commit 是主线（保留的一侧；父 1 是创建该 merge 时所在的分支），然后是通常的确认。撤销会恢复到之前的状态 |
| **将 `<branch>` reset 到此 commit** | 一个用于选择 reset 类型的子菜单；hard reset 会要求确认 |
| **编辑 commit 消息…** | 见下文 |
| **修订…** | 在一个对话框中修改 commit 的消息、作者姓名、邮箱和日期；见[修订 commit](#修订-commit消息作者日期) |
| **拆分 commit…** | 把该 commit 拆成多个；见[拆分 commit](#拆分-commit) |
| **拆分 commit（AI）…** | 同上，但分组由模型提议；仅在开启 [AI](ai.md) 时 |
| **解释更改（AI）** | 带有 AI 解释的弹出窗口；仅在开启 [AI](ai.md) 时 |
| **从此处交互式 rebase…** | 见下文 |
| **在此创建分支…**、**在此创建 tag…**、**在此创建附注 tag…** | 内联名称框 |
| **Checkout 此 commit**、**在此创建 worktree…** | 分离 HEAD，或创建 worktree |
| **从此 commit 创建补丁…**、**从此处到 `<branch>`…** | 写入 `.patch` 文件；*应用补丁*在引用的菜单中 |
| **复制 commit 在远程上的链接**、**复制** > | 复制到剪贴板（哈希、消息等） |

## 编辑 commit 消息（reword）

*编辑 commit 消息…*（或 `Ctrl+G e`）会打开完整的消息：带计数器的单行摘要、其下的描述、**取消**和**更新消息**。已检出分支的任何 commit 都可以 reword（HEAD 或祖先）；其上的 commit 会用各自的 tree、作者和日期重新创建，merge 仍是 merge，你 staged 和 unstaged 的更改不受影响。如果该 commit 已经 push 过，弹出窗口会用琥珀色警告你需要 force push（with lease），并要求再次确认。`Ctrl+Z`（工具栏的撤销）会撤回 reword；参见[撤销与重做](undo-redo.md)。状态栏也会点明之前的顶端。

不在已检出分支上的 commit（它位于另一个分支）无法在你当前所在的位置 reword 或修订，所以*编辑 commit 消息…*和*修订…*会改为打开**编辑其他分支上的 commit**，这是一个选项列表（`Up` / `Down` 或 `Ctrl+J` / `Ctrl+K` 移动，`Enter` 或点击选定，`Esc`、叉号或**取消（C）**关闭）：

- **检出 `<分支>` 并编辑（1）**：每个其历史包含该 commit 的本地分支一行（`1` 到 `8`，只有前八个，其余只计数）。分支按常规方式检出，然后在其上打开编辑。撤销就是这两个操作各自的撤销：`Ctrl+Z` 撤回编辑，检出仍然是一次检出。
- **用高级历史重写来编辑（R）**：在该 commit 上打开[历史重写工作台](history-rewrite.md)（仅限包含它的构建）。
- **在此创建分支…（B）**：仅当没有本地分支包含该 commit 时出现（比如它只在远程分支上）；打开名称框，在它处创建分支。

## 修订 commit（消息、作者、日期）

在 commit 菜单中选择 *修订…*（或按 `Ctrl+G Shift+E`，或在命令面板中选择 *Commit：修订…*）会打开一个对话框，包含该 commit 的各项属性：摘要、描述、作者姓名、作者邮箱和作者日期，每一项都已填入 commit 当前的值。修改任意项后按 **应用**；只会发送被修改的字段。日期的格式是 `YYYY-MM-DD HH:MM:SS +HH:MM`，在你编辑之前以 UTC 显示。提交者（committer）是你，时间是现在（与 reword 相同）。当前检出分支上的任何 commit 都可以修订，规则、已 push commit 的警告和第二次确认都与 *编辑 commit 消息…* 相同；其上的 commit 会以各自原有的作者和日期重新创建，你 staged 和 unstaged 的更改不受影响。`Ctrl+Z`（工具栏的撤销）可以撤回。在包含历史重写的构建中，对话框左下角的**深入修改历史…**（`Alt+H`）会关闭对话框而不应用任何内容，并打开[历史重写工作台](history-rewrite.md)，范围从这个 commit 到最新的 commit；如果你编辑过字段，它会先询问（`Esc` 确认）。`Ctrl+N` 会在对话框的每个字段和按钮上显示徽标；按下它会把光标放进该字段或按下该按钮（参见[徽标](keybindings.md#徽标)）。

无论从哪个入口进入，对话框都是完整的，每个字段都显示并可编辑；不同的只是光标的位置。在 commit 详情窗格（右侧）中，右键点击摘要、描述或作者区域，选择 *修订摘要…*、*修订描述…* 或 *修订作者…*；或者双击摘要、描述、作者或作者日期：对话框打开时光标就在该字段中。键盘等价操作在 Log 和详情窗格中都可用：`Ctrl+G Shift+E`（所有字段）、`Ctrl+G Shift+A`（作者），以及命令面板中的 *Commit：修订…*、*Commit：修订作者…*、*Commit：修订摘要…* 和 *Commit：修订描述…*。这两个快捷键都可以重新绑定（`git_prefix.amend`、`git_prefix.edit_author`；见[快捷键](keybindings.md)）。在脚本或代理中：`gitpoulpi amend <commit>`，配合 `--message`（或 `--summary` / `--description`）、`--author-name`、`--author-email`、`--author-date` 以及对应的 `--committer-*` 选项（MCP 工具名为 `amend`）。

## 拆分 commit

commit 菜单中的 *拆分 commit…*（或命令面板：*Git：拆分所选 Commit…*）会把一个 commit 拆成两个或更多，顺序从旧到新。已检出分支上任何非 merge 的 commit 都可以拆分（HEAD 或祖先，根 commit 也行）。拆分编辑器分为两个窗格：左边是**新的 commit**（各带一条消息），右边是该 commit 的**更改**，每个文件和 hunk 一行，每行都有一个 `[N v]` 字段，指明由哪个新 commit 接收它。

| 按键 | 鼠标 | 作用 |
|---|---|---|
| `Up` / `Down`（`k` / `j`） | 点击某一行 | 在更改之间移动（`Shift+Up` / `Shift+Down` 扩展选择） |
| `1`-`9` | 该行的 `[N v]` 字段，再选一项 | 把所选更改分配给该新 commit；`0` 表示暂不分配 |
| `Enter` / `m` | `[N v]` 字段 | 打开可接收所选更改的 commit 列表 |
| `n` | `+ 新建 commit` | 把所选更改放进一个新 commit |
| `v` | | **按行模式**：逐行拆分一个 hunk（拖过若干行可选中一个范围） |
| `Tab` | 点击某个窗格 | 在 commit 和更改之间切换 |
| `e` / `Enter`（在 commit 窗格中） | 铅笔图标 | 编辑该 commit 的消息 |
| `d` / `Delete` | 叉号 | 删除一个新 commit（它的更改变为未分配） |
| `Alt+Up` / `Alt+Down` | 箭头 | 上移或下移一个新 commit |
| `s` | 复选框 | 为新 commit 签名 |
| `Shift+R` | `重置` | 恢复到最初的方案 |
| `Ctrl+S` | `拆分为 N 个` | 拆分 |
| `Esc` | 叉号、`取消` | 离开；有未保存的编辑时，需再按一次确认 |

每处更改都必须恰好分配给一个 commit；编辑器会统计仍然*未分配*的数量，数量不为零时按钮一直不可用。二进制文件、只改模式的文件、在该 commit 中新增或删除的文件、子模块以及 LFS 文件，只能整体分配。第一个 commit 沿用原来的消息，其余每个新 commit 都要自己写一条消息。新 commit 保留原来的作者和日期；提交者是你，时间是现在。

**除分支外，你的仓库不会有任何变化。** 最后一个新 commit 的内容与原 commit 完全相同，因此它之上的 commit 会以相同的 tree、作者和消息重新创建（merge 仍是 merge），你已 stage 和未 stage 的更改都不受影响：工作区有改动也没关系，也不可能产生冲突。不会运行钩子。有 auto-stash 待处理时，拆分会被阻止。

**签名。** *签名 commit* 复选框的初始状态取自仓库的 `commit.gpgSign`。不勾选时，被重写的 commit（新的以及它们之上的）的签名会被丢弃，编辑器会在复选框下方说明。勾选时，所有这些 commit 都会签名；签名失败则不会改动任何内容。

**撤销。** 按一次 `Ctrl+Z`（工具栏的撤销）就能把原 commit 放回去，工作区有改动时也一样；见[撤销与重做](undo-redo.md)。已经 push 过的 commit，之后需要 force push（带 lease）。

**从交互式 rebase 进入。** rebase 评审界面的每一行都有一个小的 **拆分…** 按钮（在该行按 `t`）：它会为该 commit 打开同一个编辑器；拆分完成（或你取消）后，会回到评审界面，并显示新的历史。如果你已经编辑过方案，需要再按一次拆分来确认这些编辑会丢失。

脚本和代理可以使用 [CLI 参考](cli-reference.md)中的 `split-source` 和 `split-commit`；代理也可以为你打开已预填的这个编辑器供你审阅（`ui propose split`，见 [MCP 与代理](mcp-and-agents.md)）。

## 交互式 rebase

在已检出分支的某个祖先 commit 上选择**从此处交互式 rebase…**，或者把一个分支拖到另一个分支上并选择*交互式 rebase*。Log 会被一个审阅界面取代：每个 commit 一行，最新的在上，每行带有一个拖动手柄和一个动作下拉列表。

![交互式 rebase 界面：四个 commit，第二个设为 Squash，第三个设为 Drop，带有重置、取消 Rebase 和开始 Rebase 按钮](../images/rebase.png)

| 动作 | 按键 | 效果 |
|---|---|---|
| Pick | `p` | 保留该 commit |
| Reword | `r` | 保留它，但使用新消息 |
| Squash | `s` | 并入下面的 commit，合并两者的消息 |
| Fixup | `f` | 并入下面的 commit，丢弃它的消息 |
| Drop | `d` | 排除它 |
| Edit | `e` | 在该处停下，以便你 amend 它 |

拖动一行（或 `Ctrl+Up` / `Ctrl+Down`）可重新排序。`a` 预览 autosquash（`squash!` 和 `fixup!` 的 commit 会移到各自目标旁边）。在你按下**开始 Rebase**（`Ctrl+S`）之前什么都不会发生；**重置**会恢复原计划，**取消 Rebase**（`Esc`）则不做任何更改地离开。当某一步因冲突或 `edit` 而停下时，Log 上方的横幅会提供**继续**、**跳过**和**中止**。包含 merge commit 的范围无法进行交互式 rebase；菜单条目会说明原因。

## Cherry-pick 多个 commit

`Ctrl+点击` 切换某个 commit，`Shift+点击`（或 `Alt+点击`）选择一个范围，`Space` 切换当前的 commit，`Esc` 清除。右键点击已选中的 commit，选择 **将 N 个 commit cherry-pick 到 `<branch>`…**：会打开同样的审阅界面，具有同样的动作和重新排序，**开始 Cherry-pick** 会按顺序重放它们。

## 将 commit 复制为 Mermaid gitGraph

![Copy as Mermaid gitGraph 弹窗，位于三个选中的 commit 之上：范围、详细程度和方向的选项，围栏选项和实时预览](../images/log-mermaid.png)

在 Log 中选择 commit（`Ctrl+点击`、`Shift+点击`、`Space`；或者只把光标停在一个 commit 上），然后打开**复制为 Mermaid gitGraph…**：在 Log 中按 `Shift+Y`，用 commit 菜单的**复制**子菜单，或用面板中的 `Log：复制为 Mermaid gitGraph…`。弹窗会实时显示生成的 [Mermaid](https://mermaid.js.org/syntax/gitgraph.html) 脚本，可滚动；按 `Enter`（或点击**复制**按钮）会把恰好这段文本放到剪贴板。把它粘贴到 README、pull request 描述或 [mermaid.live](https://mermaid.live) 中即可。

弹窗有三个下拉框、一个复选框和预览区（`Tab` 在它们之间移动；每个下拉框用 `Enter`、`Space` 或 `Down` 打开）：

- **Commits**：所选内容，或者 - 当选中一个 commit 且 HEAD 是它的后代时 - **从所选内容到 HEAD**，即两者之间的那一段 commit（根据 Log 已加载的部分得出）。
- **详细程度**：`structure`（commit、分支和 merge；id 为短 SHA）、`refs`（把真实的分支名加为 Mermaid 分支，tag 写成 `tag:`，HEAD commit 写成 `type: HIGHLIGHT`）、`subjects`（commit id 变为 `<sha> <标题>`，经过转义并截断到 `ui.mermaid_subject_width` 个字符）和 `full`（在 id 中加入作者和 UTC 日期）。
- **方向**：从左到右（`gitGraph LR:`）或从上到下（`gitGraph TB:`）。
- **包含 ```mermaid 围栏**会用 Markdown 代码围栏包住脚本。

详细程度、方向和围栏会被记住（`ui.mermaid_detail`、`ui.mermaid_orientation`、`ui.mermaid_fence`）。同一个生成器也可从命令行运行：`gitpoulpi log --format mermaid [--detail structure|refs|subjects|full] [--orientation lr|tb] [--fence] [--range A..B | --limit N]`（见 [CLI 参考](cli-reference.md)）；MCP 服务器把它作为 `log` 工具的 `format` 参数提供。

Mermaid 的 `gitGraph` 是一种重放语言（commit、branch、checkout、merge），而不是图的描述，所以生成器做了几个值得了解的选择：

- 一个 commit 沿用它第一个被选中的父 commit 所在的分支；其他子 commit 会开一个新分支，并以其上的某个真实分支命名（在 `structure` 下为 `b1`、`b2`……）。第一个分支是 `main`（或通过 `mainBranchName` 头给出的真实名称），并跟随 HEAD 的 first-parent 链。
- merge 会接入另一个父 commit 的分支。如果该分支在那个父 commit 之后又往前走了，生成器会在该父 commit 处分出一个小的指针分支（`<branch>-<sha>`）并 merge 它，这样 merge 仍然指向正确的 commit。
- 不在所选内容中的父 commit 会被丢弃：一个没有任何被选中父 commit 的 commit 会自成一条线，所以选择中的空缺会显示为互不相连的线。
- octopus merge（三个或更多父 commit）会变成依次进行的 merge；中间的几个带有后缀 ` (octopus 1/2)`，最后一个才是真正的 commit。
- 不会检测 cherry-pick；标题中的双引号会变成撇号（Mermaid 无法转义它们）。

## 比较 commit

选择 Log 的两行或更多行：`Ctrl+点击` 切换一行，`Shift+点击`（或 `Alt+点击`、`Shift+Up/Down`、`Alt+Up/Down`）选择一个范围，`Space` 切换当前行。commit、**WIP** 行和 **stash** 行都可以选择。右侧窗格会变成 **比较** 窗格：

- 标题处有一个 **交换** 按钮，其下按 Log 的顺序（最新的在前）把每个选中的行列成两行的**卡片**：作者头像（按 Log 的头像设置：支持图像的终端中是图片，其他终端中是首字母）、姓名、多久以前、短 id 和摘要。**WIP** 行是“工作区 / 未提交的更改”，stash 显示 stash 图标、`stash@{n}` 及其消息。最旧的一端带 **起点** 标签，最新的一端带 **终点** 标签（交换会把它们对调）；选了三行或更多时，会有一条说明告诉你这些更改是合并比较的。卡片列表最多占窗格的三分之一，超出后滚动（鼠标滚轮、滚动条，或拖动下方的分隔条调整大小）；
- 点击卡片会把 Log 光标移到那一行，并在图中闪一下；选择保持不变，所以之后按 `Space` 就能把那一行从选择中去掉；
- 其下是常见的已更改文件列表（路径或树视图、筛选、排序），列出它们之间有差异的文件。点击文件或按 `Enter`，会在中央打开它的 diff，布局同样是内联或分栏，并在较新的一端提供 Blame、文件历史和文件视图。

![选中三个 Log 行：比较窗格把它们列为带头像和起点 / 终点标签的卡片，其下是已更改的文件](../images/compare-cards.png)

比较的对象取决于选择：

| 选择 | 比较的内容 |
|---|---|
| 两个 commit（或一个 stash 和一个 commit） | 按 Log 的顺序，较旧的与较新的比较，与点击顺序无关 |
| WIP 行和一个 commit 或 stash | 该 commit 与工作区：已跟踪文件的已暂存和未暂存更改（未跟踪的文件不属于 diff） |
| 三个或更多 commit | 它们合并后的更改：从最旧的那个之前，到最新的那个（merge 沿第一个父 commit 走） |

重命名或移动的文件显示为一个文件，diff 顶部带有旧名称和相似度。**交换**（`x`）会对调两行比较的两端；有 WIP 行时（它始终是较新的一端），或者范围有三行以上时，没有可交换的内容。与 WIP 行的比较会在你保存文件时跟随工作区变化。

键盘：`v`（或命令面板中的 **Log：比较所选内容**）把焦点移到比较的文件列表；`x`（或 **Log：交换比较的两端**）进行交换；`n`（或 **Log：转到下一个比较行**）是点击卡片的键盘对应操作：Log 光标跳到下一个选中的行，到末尾后回到开头。`Esc` 或在某一行上点击会回到单个 commit。**Cherry-pick** 项只统计所选的 commit；WIP 行和 stash 在那里会被忽略。

脚本和代理也能得到同样的列表：`gitpoulpi diff --from <rev> [--to <rev|worktree>] [--name-only]`（CLI 和 MCP 的 `diff` 工具；`--to` 默认为工作区）。

## 冲突和其他横幅

当 merge、rebase、cherry-pick 或 revert 因冲突而停下时，Log 上方会出现一条横幅，有冲突的文件在状态窗格中带有红色的 `!`。在 merge 编辑器中解决它们，然后使用横幅的 **继续** 或 **commit merge**，或用 **中止** 退出。参见[解决 merge 冲突](merge-conflicts.md)。

Log 上方的其他横幅：

| 横幅 | 按钮 | 命令面板 |
|---|---|---|
| *正在把 `<branch>` rebase 到 `<commit>` · 步骤/总数 · 停在 ...* | **继续**、**跳过**、**中止** | *Rebase：继续 / 跳过 / 中止* |
| *正在应用补丁（git am）· 补丁 n / m*（停下的 `git am`） | **解决冲突…**（某个补丁有冲突时）、**继续**、**跳过**、**中止**（会先询问） | *git am：继续 / 跳过此补丁 / 中止* |
| *残留的 index.lock（n 秒前）阻止了对此仓库的每一次更改*（某个 git 命令死掉并留下了 `.git/index.lock`） | **删除 index.lock**（会先询问；仅当文件至少已有 10 秒时才提供） | *仓库：删除过期的 index.lock* |
| *Auto-stash pending: ...* | 参见 [Auto-stash](#自动-stash) | *横幅：...* |

## Blame

Blame 显示整个文件，带行号和语法着色，并在代码**右侧**给出一列注释（`ui.blame_side = "left"` 将其移到左侧）。可从文件的栏（`Blame`）、文件菜单、命令面板或历史行（`b`）打开。从工作区打开时，尚未 commit 的行会单独显示（见色条）。

- **Hunk。** 最后一次由同一个 commit 修改的连续若干行是一个 hunk。它的第一行显示作者头像（终端能绘制图像时为图片，否则为首字母徽标，*作者头像* 关闭时不显示）、作者、距今多久和 commit 摘要；其余行只显示作者的颜色条。hunk 之间用细线分隔。当 hunk 的起点在屏幕上方时，它的注释会在第一个可见行重复出现。
- **色条。** 注释旁是作者颜色条和**年龄热度条**：新的行偏暖，旧的行偏冷，以文件自身最旧和最新的一行为基准（图例在标题里）。尚未 commit 的行以点线显示，并标注“Not committed yet”。
- **卡片。** 把指针在注释上停留片刻，或按 `i`，即可看到 hunk 的详情卡片：完整的 commit ID、作者和邮箱、日期、摘要及其操作。选中的 hunk 会着色。

  ![开启缩略图并在选中的 hunk 上打开卡片的 Blame](../images/blame-card.png)
- **操作**（卡片行、hunk 上的右键菜单、按键，以及命令面板中的 `Blame: ...`）：在 Log 中显示该 commit（`Enter`）、复制其 ID（`Shift+Y`）、显示该 commit 对此文件的 diff（`d`）、对该 commit 的父级做 Blame（`b`，会跟随重命名）、打开文件历史（`Shift+H`）。
- **移动。** `j` / `k`、PageUp/PageDown、Home、`Shift+G` 和滚轮与别处相同；`n` / `p` 下一个 / 上一个 hunk，`Shift+N` / `Shift+P` 同一 commit 的下一个 / 上一个 hunk，`g` 跳到某一行号。
- **较高的 hunk。** 三行及以上的 hunk 会在摘要下方显示 commit 的短 ID 和完整日期（淡色），让这一列不至于空着。
- **缩略图。** `ui.minimap` 开启时（`缩略` 按钮，`M`），文件的缩略图位于代码旁，用与色条相同的年龄热度着色；沿其点击或拖动即可滚动。
- **宽度。** 拖动注释列的边框，或按 `>` / `<`；宽度保存为 `ui.blame_width`。

## 文件历史

可从 diff 栏 `本文件 │ 全部` 选择器的 `本文件` 段、文件菜单或命令面板（`文件历史：...`）打开。中央分为两半：左侧是该文件的 commit，最新的在前，并跟随重命名；右侧是所选 commit 中该文件的 diff，具备 Diff 视图的所有模式（hunk / 整个文件、内联 / 并排、逐词 diff、缩略图、换行）。

- **列表。** 每个 commit 两行。第一行：变更字母（A 新增，M 修改，D 删除，R 重命名）、摘要，以及带颜色的 `+新增 -删除`。第二行为淡色：作者头像（与 Log 作者列按 *作者头像* 设置所显示的完全一致）、作者、相对日期、commit 短 ID，以及重命名该文件的 commit 上的 `旧名 -> 新名`。列表较窄时，依次省略重命名、ID、作者名的末尾，但绝不省略作者和日期。临近末尾时每次加载 100 条，期间会出现 `正在加载更多…` 一行。
- **移动。** `j` / `k`、方向键、PageUp/PageDown、Home / End、滚轮和点击；diff 立即跟随（diff 在后台获取，按行缓存，按住按键只会请求一个）。重命名之前的行按当时的文件名比较。
- **Diff / 文件 / Blame。** 视图栏自己的**文件视图**、**Diff 视图**和 **Blame** 按钮用来选择右侧为所选 commit 显示的内容：该 commit 的 diff、文件（该 commit 时的文件）或其 blame，全部在分栏内完成，列表保留在左侧，列选择器的 `本文件` 段保持高亮。所选模式在切换行时保持，右侧跟随选择。按键 `Alt+1` / `Alt+2` / `Alt+3`（在列表中也可用 `d` / `o` / `b`），两半都可使用，命令面板中的 `文件历史：...` 各行效果相同；`Esc` 返回 Diff。
- **列。** 该列表就是显示其**本文件**列表的 *commit 列*：隐藏、显示、尺寸和 `本文件 │ 全部` 选择器统一在 [Commit 列](interface-tour.md#commit-列)中介绍。点击 `全部` 会离开文件历史，并让视图停留在所选 commit 上。

  ![选中中间一个 commit 的文件历史：右侧是它的 diff](../images/file-history.png)

  ![右侧为 Blame 模式的文件历史，所选 commit 时文件的 blame](../images/file-history-blame.png)
- **比较两个 commit。** `Shift+Up` / `Shift+Down` 或 `Shift+点击` 选择一个范围；右侧显示两端之间该文件的 diff，各自带有自己的路径。`Esc` 放弃该范围。
- **焦点。** `Tab`（或点击）把键盘交给 diff，再交回；diff 保留自己的按键。
- **操作**（右键菜单、按键，以及命令面板中的 `文件历史：...`）：在 Log 中显示（`Enter`、双击；Log 会按需向前加载）、复制 ID（`y`）或摘要（`Shift+Y`）、显示 diff（`d`）、在右侧显示该 commit 时的文件（`o`）或其 blame（`b`）、与工作区比较（`c`，再按一次退出）、把文件恢复到该版本（`Shift+R`，需确认）。
- **撤销。** 恢复某个版本是一个写入日志的步骤：撤销（`Ctrl+Z`）会找回文件原来的字节。
- **分隔条。** 拖动两半之间的边框，或用 `Alt+Right` / `Alt+Left`，`Alt+0` 重置；宽度按会话记忆。

## Fetch、pull 和 push

- **Pull**（`Ctrl+G p`）按 *设置 > Git 与编码 > Pull 按钮* 所指定的方式 pull（`git.pull_mode`：merge、仅 fast-forward、rebase；按仓库的设置）。它的 `▾` 会一次性运行 *Fetch 全部*、*Pull（尽可能 fast-forward）*、*Pull（仅 fast-forward）* 或 *Pull（rebase）*，而不改变按钮。模式总是显式传给 git，所以你的 git 配置中的 `pull.rebase` 不会让你意外。
- **Pull 所有分支**（`Ctrl+G Ctrl+P`，Pull `▾` 的一行）会先从你的分支所跟踪的远程下载，然后一次性让每个本地分支都追上其 upstream。未检出的分支只会被 fast-forward（不触碰任何文件）；已分叉的分支保持原样并被报告。当前检出的分支按 Pull 按钮的设置来 pull，`▾` 中还有针对其他各模式的行（*Pull 所有分支（仅 fast-forward）*、*（rebase）*、*（可能时 fast-forward）*）。当某个落后的分支在另一个 worktree 中检出时，会先弹出对话框，询问是否也更新那些 worktree、跳过它们或取消。状态栏会汇总这次运行，活动日志每个分支一行；撤销会一步带回所有分支。`Ctrl+G p` 仍然是普通的 Pull。
- **Push**（`Ctrl+G Shift+P`）把已检出的分支发送到默认远程，第一次时设置上游。它的 `▾` 会一次性 push 到你选择的另一个远程。如果远程有你没有的 commit，对话框会提供 **Pull（merge），然后 push**、**Force push（with lease）…**（绝不使用裸的 `--force`）或 **取消**。
- `Ctrl+G f` 会 fetch 所有远程。在 *设置 > 同步* 中，*自动 Prune*决定 fetch 是否 prune，*自动 Fetch 间隔（分钟）*则在后台 fetch。

所有这些在 [CLI](cli-reference.md) 中也可用，即 `gitpoulpi fetch`、`pull` 和 `push`。

## 自动 stash

当未 commit 的更改阻止了 checkout、pull、merge 或 rebase 时，GitPoulpi 可以替你把它们移到一边：它 stash 这些更改，运行操作，然后 pop 该 stash。*设置 > 同步 > checkout、pull、merge 或 rebase 前自动 stash*（`general.auto_stash`）：

- **ask**（默认）：对话框提供*Stash 更改并继续*、*始终自动 stash*（同上，并设置 `general.auto_stash = "always"`，这样就不会再询问；设置 > 同步可改回）或*取消*。CLI 和 MCP 则会失败，并指出 `--autostash`。
- **always**：不询问直接完成。**never**：拒绝该操作。

*自动 stash 包含未跟踪文件*还会包含未跟踪文件。干净的工作区永远不会产生 stash。stash、操作和 pop 各自都可以撤销。

如果 stash 无法放回（它与结果冲突，或 merge 或 rebase 因自己的冲突而停下），它会留在 stash 列表中。**Log 上方的横幅**会显示 *Auto-stash pending: your changes from before the checkout are in stash@{n}*。它的按钮：

| 按钮 | 作用 |
|---|---|
| **解决冲突…** | 打开 merge 编辑器（仅在有冲突的 pop 之后） |
| **保留结果** | 保持工作区原样并丢弃 stash 副本（仅在有冲突的 pop 之后，且没有文件处于冲突状态时） |
| **立即 Pop**（pop 失败后为 **重试 Pop**） | 把 stash 放回 |
| **Show stash** | 显示该 stash 的更改 |
| **重新开始** | 见下文 |
| **Drop** | 丢弃该 stash（会先询问） |

每个按钮都有命令面板命令（*横幅：Pop 待处理的 auto-stash*、*横幅：重新开始…*，等等）。不会替你重试或丢弃任何东西。

**重新开始**会先询问。然后它会：

1. 把被 stash 文件的当前状态的副本保存在 `refs/gitpoulpi-backup/autostash-start-over-<time>` 下；
2. 仅丢弃这些文件上的更改（如果其他文件有可能丢失的本地更改，它会拒绝并点名这些文件）；
3. 通过撤销日志撤销执行了 auto-stash 的那个操作（所以该操作必须是最近的一个）；
4. 在原来的 commit 上把 stash pop 回来，并清除横幅。

如果某一步失败，它会在 stash 完好的情况下停止，并说明停在哪里。当横幅显示、且最近的操作正是执行了 auto-stash 的那个操作时，**撤销**（`Ctrl+Z`、`gitpoulpi undo`）在确认之后也能做同样的事。脚本使用 `gitpoulpi autostash start-over [--dry-run] [--yes]`（没有 `--yes` 时退出码为 3）。

**在 auto-stash 待处理期间，会移动历史的操作被拒绝：**检出分支或 commit、merge、rebase（包括交互式）、pull、fast-forward、cherry-pick、revert、reword、AI 撰写 commit 以及重做。窗口会把这些按钮和命令面板项置灰并给出原因。CLI 和 MCP 会得到 `PendingAutoStash` 错误（退出码 1）。其他一切仍然允许：读取、stage、丢弃、commit、分支、tag、远程、fetch、push、rebase 的继续 / 跳过 / 中止、冲突解决、`stash push`、`reset`，以及 `stash pop` / `apply` / `drop`。撤销同样允许。

当 auto-stash 在等待一个因冲突而停下的 merge 或 rebase 时，该操作一结束（在 GitPoulpi 中，或用普通 git 完成后刷新），它就会自动被 pop。
