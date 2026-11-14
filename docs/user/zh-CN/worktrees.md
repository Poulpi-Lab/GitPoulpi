[English](../worktrees.md) · [Čeština](../cs/worktrees.md) · [Deutsch](../de/worktrees.md) · [Español](../es/worktrees.md) · [Français](../fr/worktrees.md) · [Italiano](../it/worktrees.md) · [日本語](../ja/worktrees.md) · [한국어](../ko/worktrees.md) · [Nederlands](../nl/worktrees.md) · [Polski](../pl/worktrees.md) · [Português (Brasil)](../pt-BR/worktrees.md) · [Português (Portugal)](../pt-PT/worktrees.md) · [Русский](../ru/worktrees.md) · [Svenska](../sv/worktrees.md) · [Türkçe](../tr/worktrees.md) · [Українська](../uk/worktrees.md) · **简体中文**

# Worktree

worktree 是仓库的第二份检出副本：另一个文件夹，位于另一个分支上，共用同一份历史。引用窗格的 **WORKTREE** 分区会列出它们；主 worktree 标有 `（主）`。

## 显示该分区

在还没有任何关联 worktree 的仓库中，该分区是隐藏的。若想仍然看到它（此时会显示“还没有 worktree：按 + 创建”），可以使用命令面板命令 **左侧窗格：显示 worktree 分区**，在侧栏的显示/隐藏列表中勾选 **WORKTREE**（右键点击某个分区标题），或者运行下面任何一个创建命令。GitPoulpi 会为该仓库记住这一选择（参见[设置](settings.md)中的 **左侧窗格分区**）；另一个没有 worktree 的仓库启动时仍然隐藏该分区。

## 创建 worktree

WORKTREE 标题上的 **+**（键盘：在该分区中按 `+`；命令面板：**引用：创建 worktree…**）会打开一个菜单：

- **从当前分支**：该分支已在主文件夹中检出，因此对话框会建议在它之上创建一个新分支（`<branch>-wt`）。
- **从所选 commit**：在 Log 中选中某个 commit 之前是灰色的。除非你指定新分支，否则 worktree 会在那里以 detached 状态开始。
- **选择分支…**：对话框打开时停在它的 **引用** 列表上。

命令面板里有同样的三项（**引用：从当前分支创建 worktree**、**Git：在所选 commit 处创建 worktree…**、**引用：选择分支创建 worktree**）。旧的入口仍然可用：分支、tag 或 commit 菜单中的 **从 … 创建 worktree…**。

对话框会给出所有建议，你可以修改：

| 字段 | 含义 |
|---|---|
| **名称** | 文件夹名由它生成。根据分支或短 id 给出建议。 |
| **文件夹** | 仓库的同级文件夹，`<repository>-<name>`。在你于“文件夹”中输入之前，它会跟随“名称”；**浏览…** 用来选择位置。 |
| **引用** | worktree 的起点分支或 commit（一个列表；输入即可搜索）。更改它会给出新的名称建议。 |
| **新分支** | 留空表示在那里检出“引用”所指的分支（对 commit 则是 detached）。当该分支已被检出时必填。 |
| **将新 worktree 作为标签页打开** | 默认开启。 |

非空的文件夹、无效的名称或已存在的分支，会在对话框自己的那一行中报告，同时 **创建** 保持灰色。

## 行菜单

右键点击某个 worktree（或选中它后按菜单键、`Shift+F10` 或 `.`）：

- **在新标签页中打开**：与双击或 `Enter` 相同。
- **打开方式**：勾选了 **可打开 worktree** 的[打开方式](open-in.md)条目（设置 > 打开方式；默认每个条目都已勾选）。它们以该 worktree 的文件夹作为 `${cwd}` 运行。
- **将 <branch> merge 到…**：选择要 merge 到的分支（默认是主 worktree 的分支）。普通的 merge 会在该分支被检出的那个 worktree 中运行，并使用你的 merge 选项；冲突由常规的冲突流程在那里解决。未在任何地方检出的分支会被拒绝，并说明原因。detached 的 worktree 不提供此项。
- **移除 worktree…**：主 worktree 永远不行。

命令面板对所选行有同样的操作（**引用：worktree 的打开方式…**、**引用：将 worktree 的分支 merge 到…**、**引用：移除 worktree…**、**引用：在标签页中打开 worktree**）。

## 移除 worktree

干净的 worktree 只需一次普通确认。如果它的分支已经 merge（到它的 upstream、主 worktree 的分支或 `main` / `master`），**同时删除分支 <b>** 这个复选框默认勾选；未 merge 的分支会说明它将保留。**Merge 到…后移除** 会先 merge，只有 merge 无冲突完成后才移除。

有未提交的更改（已跟踪或未跟踪）时，对话框会列出它们（带数量，可滚动），并提供：

- **Commit 并移除**：对话框中有一个消息字段；所有更改都会被 stage（遵守 `.gitignore`）并 commit 到该 worktree 的分支，然后移除 worktree。detached 的 worktree 在同一个对话框里为这个 commit 指定一个分支名。
- **Stash 并移除**：所有更改（包括未跟踪文件）都会用一条可编辑的消息 stash（默认“WIP on <folder>”），然后移除 worktree。stash 列表由所有 worktree 共用：之后这个 stash 就在主仓库的 STASHES 中，随时可以 pop。
- **打开以 commit**：在标签页中打开该 worktree，并聚焦到 commit 消息；不会移除任何内容。
- **仍然移除**：经过第二次明确确认后，丢弃这些更改。

## 撤销

移除 worktree 无法撤销：文件夹已经没了。事先保存的内容会保留：commit 在它的分支上，stash 在列表中。**同时删除分支** 所增加的分支删除是普通的、有日志的操作：**撤销** 会找回该分支（找不回文件夹）。创建 worktree 同样不属于历史的一部分。

## 来自脚本或代理

`gitpoulpi worktree changes <path>` 列出某个 worktree 的未提交更改。`gitpoulpi worktree remove <path> [--force] [--commit-message M [--commit-branch B]] [--stash-message M] [--delete-branch]` 用来移除它；对应的 MCP 工具是 `worktree_changes` 和 `worktree_remove`。参见 [CLI 参考](cli-reference.md)。
