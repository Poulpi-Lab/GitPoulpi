[English](../merge-conflicts.md) · [Čeština](../cs/merge-conflicts.md) · [Deutsch](../de/merge-conflicts.md) · [Español](../es/merge-conflicts.md) · [Français](../fr/merge-conflicts.md) · [Italiano](../it/merge-conflicts.md) · [日本語](../ja/merge-conflicts.md) · [한국어](../ko/merge-conflicts.md) · [Nederlands](../nl/merge-conflicts.md) · [Polski](../pl/merge-conflicts.md) · [Português (Brasil)](../pt-BR/merge-conflicts.md) · [Português (Portugal)](../pt-PT/merge-conflicts.md) · [Русский](../ru/merge-conflicts.md) · [Svenska](../sv/merge-conflicts.md) · [Türkçe](../tr/merge-conflicts.md) · [Українська](../uk/merge-conflicts.md) · **简体中文**

# 解决 merge 冲突

当 merge、cherry-pick、revert 或 rebase 因冲突而停下时，Log 会显示一条横幅（*merge 进行中 · 将 feature merge 到 main · 2 个有冲突的文件*），有冲突的文件会以红色的 `!` 出现在状态窗格中。在 **merge 编辑器** 中打开其中一个：A（当前一侧）和 B（传入一侧）并排显示，下方是 **输出**，也就是你正在编写的文件。本页是精简版；完整的按键列表见[快捷键](keybindings.md)。

## 打开它

- 在状态窗格的 `!` 行上按 `Enter` 或双击；它的右键菜单以 **在 merge 编辑器中打开** 开头。
- 横幅上的 **解决冲突…** 按钮（它会打开第一个有冲突的文件）。
- 有冲突文件的 Diff 视图：**merge 编辑器** 按钮（`Shift+E`）。
- 命令面板：**冲突：打开 merge 编辑器**。

不会自动打开任何东西：停下的操作只会显示横幅和 `!` 行。

![停在冲突 merge 上的仓库的 Log：带有“解决冲突”和“中止”的 merge 进行中横幅，以及两个以红色 ! 标记的有冲突文件](../images/conflict-banner.png)

编辑器占满整个窗口宽度：打开期间 引用侧栏和右侧窗格会被隐藏，关闭后再回来（期间可用 `F9` 显示或隐藏它们）。只要编辑器至少有 `merge.side_by_side_min_width` 列宽（默认 100，设置 > Diff 与编辑器；`0` 表示始终如此），A 和 B 就并排显示；更窄时，一次只显示一侧，用标签页切换。长行可以横向滚动，A 和 B 一起滚动（`Shift+←` / `Shift+→`、Shift+滚轮或横向滚轮），它们绝不会被折行。

![merge 编辑器：A（当前）和 B（传入）并排位于输出之上，冲突 1 取自 A，冲突 2 取自 B，冲突 3 仍待选择](../images/merge-editor.png)

## 选择

每个冲突在 A 和 B 中都是一个带框的块，标题上有一个复选框，每一行前面有一个 `+` / `✓`。

| 想要…… | 按键 | 鼠标 |
|---|---|---|
| 采用光标所在的行（当前聚焦的一侧） | `Space` | 点击该行 |
| 采用当前聚焦一侧的整个块 | `Enter` | 点击其标题上的 `[ ]` |
| 此冲突只采用 A / 只采用 B | `a` / `b` | 右键点击该块 |
| 两者都采用，先 A 后 B / 先 B 后 A | `c` / `Shift+C` | 右键点击该块 |
| 都不采用 | `x` | 右键点击该块 |
| 交换两个已选侧的顺序 | `s` | 点击 `1st` / `2nd` 徽标 |
| 全部采用 A / 全部采用 B（每个冲突） | `Shift+A` / `Shift+B` | 工具栏 |
| 取消此冲突的选择（回到未解决） | 在 A / B 中按 `u`，任何位置按 `Alt+U` | 右键点击该块，**取消选择** |
| 把整个文件重置为原始冲突（当你在输出中输入过内容时会先询问；`Ctrl+Z` 可带回） | 在 A / B 中按 `Shift+U`，任何位置按 `Alt+R` | “全部采用”旁的 **重置** |
| 下一个 / 上一个冲突 | `F7` / `Shift+F7`、`Alt+Down` / `Alt+Up`（在 A / B 中为 `n` / `Shift+N`） | `▼` / `▲`，或点击滚动条上的标记 |
| 数字重复一次移动 | `3j`、`2k`、`2n`、`3N`（在 A / B 中） | |
| 下一个 / 上一个有冲突的文件 | `Alt+PageDown` / `Alt+PageUp` | 标题中的 `›` / `‹` |
| 显示或隐藏 Base（共同祖先）窗格 | `Alt+D`（在 A / B 中为 `d`） | `基础` |
| 在 A 和 B 中显示整个文件（**整个文件**） | `Alt+F`（在 A / B 中为 `f`） | `整个文件` |
| 给 A 或 B 更多宽度 / 恢复等宽 | `Alt+Right` / `Alt+Left`、`Alt+0` | 拖动分隔条；双击它 |
| 横向滚动 A / B / Base | `Shift+←` / `Shift+→`（`Shift+H` / `Shift+L`） | Shift+滚轮、横向滚轮 |
| 显示或隐藏 引用和状态窗格 | `F9` | |
| 聚焦 A、B、输出 | `F6` / `Shift+F6` | 点击 |
| 跳到某个窗格，停留在当前所在的冲突 | `Alt+1` A、`Alt+2` B、`Alt+3` 输出、`Alt+4` Base；或 `Ctrl+N` 然后 `1` / `2` / `3` | |
| 移到相邻窗格（A、Base、B、输出），同一冲突 | 在 A / B 中按 `h` / `l` 或 `←` / `→` | |
| 循环切换输出的高度 | `z` | 拖动它的标题 |
| 撤销 / 重做（选择和输入，同一个栈） | `Ctrl+Z` / `Ctrl+Y` | |
| 写入文件，保持冲突状态 | `Alt+S` | **保存** |
| 写入文件并标记为已解决 | `Ctrl+S` | **保存并标记为已解决** |
| 关闭 | `Esc` | `✕` |

每个滚动条（A / B / Base 以及输出）上，每个冲突都有一个标记：仍需选择时为橙色，处理完毕后为绿色，当前冲突为蓝色。点击标记会跳转到该冲突（位于滚动条滑块下方的标记已经在屏幕上，所以在那里点击会拖动滑块）。

**整个文件** 在 A 和 B 中显示整个文件，并为每一处差异着色，而不只是冲突：

![整个文件模式：A 和 B 中的整个文件，更改处已着色](../images/merge-editor-full-file.png)

输出是一个真正的编辑器：点击其中即可输入、选择、剪切和粘贴，并用 `Ctrl+F` 查找。对你手动编辑过的冲突进行选择，会替换你的编辑（`Ctrl+Z` 可带回）。如果保存时有冲突尚未选择，或有一行看起来仍像冲突标记，会先询问。

### ✦ 建议解决方案（AI）

开启 AI 后（设置 > AI；参见 [AI 辅助](ai.md)），工具栏有一个 **✦ 建议** 按钮，冲突的右键菜单有 *✦ 建议解决方案（AI）*，`Alt+I` 和命令面板（`AI：建议 merge 冲突解决方案`）也能做同样的事。它只处理当前冲突，并且只在你要求时运行。答案会作为一次编辑替换输出中该冲突的部分，并标记为已编辑（`✎`），所以 `Ctrl+Z` 会把冲突恢复原样。它绝不会替你保存或 stage。运行期间提示行会显示已用时间；`Esc` 取消，失败也会显示在那里。关闭 AI 后，这些控件都不存在。

**保存并标记为已解决** 之后，文件会被 stage，有冲突文件的列表会重新出现；请自己打开下一个文件。`Ctrl+Z`（或撤销按钮）会撤回上一次“标记为已解决”。当没有文件剩下时，横幅会提供 **commit merge**（cherry-pick、revert 或 rebase 则为 **继续**）和 **中止 merge**，后者会先询问：它会把 HEAD、索引和工作区恢复到原来的状态。

如果 merge 的结果与 HEAD 相同（你在所有地方都采用了我方），那么没有任何东西被 stage，也没有已更改的文件，但仍然可以完成它：**commit merge** 会打开带有 merge 消息的 commit 提示，而且该 commit 有两个父 commit。同样，在 merge 停下期间，撰写器的 commit 按钮即使没有 stage 任何内容也是启用的。

空的 cherry-pick 或 revert（更改已经应用过）无法 commit：git 会拒绝，**继续** 会这样说明，并指向 **跳过**。**跳过**（横幅按钮、命令面板项 *Git：跳过 cherry-pick 或 revert*、CLI `conflicts skip`）会先询问，然后结束该操作，不为它记录任何内容；HEAD、索引和工作区回到 HEAD。跳过不可撤销（除了你有意放弃的东西，没有任何东西会丢失）；merge 没有可跳过的内容。

## 非文本冲突

二进制文件、Git LFS 指针、超过 8 MiB 的文件、一侧已删除的文件、重命名的遗留、符号链接、模式更改或子模块没有可供选择的行。编辑器会显示一张**卡片**，说明双方各持有什么，并为每个选择提供一个按钮；选择之后会立即解决并 stage 该文件。

| 卡片 | 选择 |
|---|---|
| 二进制文件、LFS 指针、过大 | **保留 A**（`a`）、**采用 B**（`b`）、**采用基础版本**（`o`，有的话）、**在 merge 工具中打开**（`m`） |
| 一侧已删除，另一侧已更改 | **保留文件**（`k`）、**接受删除**（`Shift+D`） |
| 重命名遗留（路径只存在于一侧） | **保留文件**（`k`）、**删除文件**（`Shift+D`） |
| 符号链接、子模块、文件模式 | **保留 A**、**采用 B**、**采用基础版本** |

当文件是图片（PNG、JPEG、GIF、WebP、BMP、ICO）时，这些框中还会显示 A、B 和 Base 的图片，按窗口宽度适配且只读；参见[图片](image-view.md#在-merge-编辑器中)。

![二进制冲突的卡片：A、B 和基础版本各持有什么，以及保留 A、采用 B、采用基础版本和在 merge 工具中打开按钮](../images/merge-editor-card.png)

`Esc` 关闭卡片而不做选择。外部 merge 工具是设置 > Diff 与编辑器中的 `general.external_merge_tool`（`{base}`、`{local}` = A、`{remote}` = B、`{merged}`；为空表示 `git mergetool`）；当工具成功退出时，文件会被 stage。

## 冲突预防

GitPoulpi 可以在你 merge *之前*就告诉你。当你所在的分支会与它的来源分支（它的**基础分支**）产生冲突时，Log 顶部边框行上会出现一条提示：**与 origin/main 在 2 个文件中冲突**，并带有一个 **显示文件** 按钮。它占用进行中操作的横幅之后的下一个空闲边框行，所以当那些横幅显示时，它可能要等到有空位。不会真的 merge：守护进程在内存中对这两个 commit 做一次 merge（用 libgit2，或在 git 2.38 及以上用 `git merge-tree --write-tree`；更旧的 git 会让此功能关闭，并在日志里记一行），所以你的文件、索引和 ref 都不会被改动。

- **基础分支**：当 GitPoulpi 已经知道该分支的 pull request 的目标分支时（“Pull requests”区域保存的缓存；为此不会发起任何网络调用），就是那个目标分支；否则是符合 **基础分支** 模式（设置 > 冲突预防）的分支中、自分叉以来 commit 最少的那个。并列时，选择其末端离分叉点最近的分支，然后是远程跟踪分支优先于本地分支，最后按名称。分支本身及其自己的远程副本永远不会是基础分支。
- **弹出窗口**（点击该提示，或用命令面板的 **冲突：显示预测的冲突**，它可以用键盘操作）列出这些文件，并有 **打开 merge 预览** 和 **Rebase 到基础分支**。它们是基础分支自己的菜单项（*将 <base> merge 到 <branch>*、*将 <branch> rebase 到 <base>*）替你执行：会记入撤销日志，所以 `Ctrl+Z` 可以撤回，确实有冲突的 merge 会像其他 merge 一样停在 merge 编辑器里。左右键在按钮之间移动，Enter 按下其中一个，Esc 关闭。
- **何时运行**：在后台，仓库安静下来（一次 commit、一次 checkout、一次移动了分支的 fetch、一次 rebase）约 2 秒后，每个 `(HEAD, 基础分支)` 组合运行一次。HEAD 一移动，提示就会隐藏，随后带着新的结果重新出现，或不再出现。分离的 HEAD 不会被检查。
- **基础分支列表**：仓库的基础分支来自哪里，是它的 **冲突预防** 页中的一个选择。**默认** 沿用全局默认（内置的 `**`，即所有分支，直到你在 设置 > 冲突预防 中选择另一个列表）；**列表：<名称>** 沿用你的某个命名列表；**自有列表** 是仓库自己的列表。列表是共享的：编辑其中一个，每个沿用它的仓库都会在下次检查时采用更改，无需重启。**另存为列表…** 把仓库自己的模式变成一个命名列表，**恢复默认** 丢弃仓库的选择。删除仓库所沿用的列表会让它改用默认列表。
- **关闭**：设置 > 冲突预防 中的 **启用主动冲突检测**，或在某个仓库自己的 **冲突预防** 页中关闭。在脚本中，`gitpoulpi conflicts check` 会以 JSON 打印同样的结果（[CLI 参考](cli-reference.md)）。

## 代理

代理可以向你展示冲突（`gitpoulpi ui show conflict <path> [--conflict N]`：编辑器会在它上面打开，不选择任何内容），也可以**提出**解决方案（`gitpoulpi ui propose resolution <path> --pick 1=a,2=ba` 或 `--content-file F`，需要 *允许 AI 代理提出更改供我确认*）。提案会打开编辑器，选择（或整段文本）已经放在输出中，并带有紫色的 *✦ Proposed by claude-code* 横幅。请审阅它，修改你想改的地方，然后按 **Ctrl+S**：只有这样才会写入或 stage 任何内容。关闭编辑器即拒绝该提案。参见 [MCP 与代理](mcp-and-agents.md)。

在脚本中，相同的操作是 `gitpoulpi conflicts list | show | resolve | unresolve | state | abort`（[CLI 参考](cli-reference.md)）；`conflicts check` 可在 merge 之前预测冲突。
