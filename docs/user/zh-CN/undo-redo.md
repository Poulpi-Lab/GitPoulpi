[English](../undo-redo.md) · [Čeština](../cs/undo-redo.md) · [Deutsch](../de/undo-redo.md) · [Español](../es/undo-redo.md) · [Français](../fr/undo-redo.md) · [Italiano](../it/undo-redo.md) · [日本語](../ja/undo-redo.md) · [한국어](../ko/undo-redo.md) · [Nederlands](../nl/undo-redo.md) · [Polski](../pl/undo-redo.md) · [Português (Brasil)](../pt-BR/undo-redo.md) · [Português (Portugal)](../pt-PT/undo-redo.md) · [Русский](../ru/undo-redo.md) · [Svenska](../sv/undo-redo.md) · [Türkçe](../tr/undo-redo.md) · [Українська](../uk/undo-redo.md) · **简体中文**

# 撤销与重做

**你总能退回去。** GitPoulpi 对你的仓库所做的每一项更改都会被记录（默认最多 100 项操作，`general.undo_depth`），分支曾经到过的每个位置也一样，包括其他程序造成的移动。如果你犯了错，可以用下面这些方式把工作找回来：

| 你想要 | 使用 |
|---|---|
| 撤回上一件事 | `Ctrl+Z`（重做：`Ctrl+Y` 或 `Ctrl+Shift+Z`），或工具栏的 **撤销** 和 **重做** 按钮 |
| 一次撤回多件事，或撤回很久以前的某一件事 | **撤销历史**（`Alt+Z`、撤销旁的箭头、`编辑：撤销历史…`） |
| 找回几步之前移除的分支、tag、stash、文件或 hunk，**而不撤销此后做的事** | 撤销历史中的 **仅恢复此项** |
| 把分支移回它原来的位置，即使是别的程序移动了它 | 分支的 **Reflog**（引用菜单 `Reflog…`） |
| 找出已经没有任何分支能到达的 commit | **丢失的工作**（`编辑：找回丢失的 Commits…`） |
| 撤回 shell 或 **AI 代理** 对分支、tag、stash 或 HEAD 所做的事 | 同一个 `Ctrl+Z`：这些更改也在历史里，标为 **外部**（见[在 GitPoulpi 之外所做的更改](#在-gitpoulpi-之外所做的更改)） |

每次执行会销毁内容且可以撤销的操作之后（丢弃、删除文件、删除分支或 tag、hard reset、丢弃 stash、移除远程），状态栏都会提示：“已完成：丢弃 - 按 Ctrl+Z 撤销”，其中的按键是你当前绑定的按键。撤销按钮的工具提示会说明你能回退多少项操作。

`Ctrl+Z` 撤回 GitPoulpi 对你的仓库所做的上一件事。`Ctrl+Y`（或 `Ctrl+Shift+Z`）把它恢复回来。工具栏有 **撤销** 和 **重做** 按钮，命令面板有 `编辑：撤销` 和 `编辑：重做`，每个按钮的工具提示会说明它将撤回什么（“撤销 Commit “fix typo” (a81b9e4)”）。

![全部 stage 之后的工具栏：撤销按钮已点亮，其工具提示点明了它将撤回的操作](../images/undo-toolbar.png)

重做从不重新执行操作。它恢复已记录的状态，所以被重做的 commit 就是原来的那个 commit，id 和时间戳都相同。

当文本框占用键盘时（commit 消息、过滤框、编辑器，编辑器自带文本撤销），`Ctrl+Z` 不起作用。

## 撤销历史

`Alt+Z`（可重新绑定）、撤销按钮旁的箭头或 `编辑：撤销历史…` 会打开一个面板，列出每一项已记录的操作，最新的在前：一个表示操作类型的图标（commit、分支、tag、stash、reset、merge 之类、丢弃、stage、配置、外部更改）、用你的语言写出的做了什么、多久以前，以及它的状态（**可单独恢复**、**被阻止**并附原因，或 **已撤销** 且可以重做）。当条目跨越多天时，每一天以一个细标题开头（今天、昨天，然后是日期）。标题会统计数量（“可撤销 12 项操作（最多 100 项）”）。

你已撤销的操作位于最新操作的**上方**，在一条分隔线之下（“已撤销 - 重做会带回它们”），显示为暗色，重做将带回的那一项紧挨分隔线。选中项从最新的操作开始，而不是它们。方向键移动（会跳过标题和分隔线），`Tab` 在三个视图之间切换，`Esc` 关闭。对选中的行：

- **撤销到这里**（`u`）：撤销该操作以及之后的所有操作，最新的先撤销，并先用一次确认列出它们。守护进程认为有风险的某一步，会在执行前再次询问。
- **仅恢复此项**（`r`）：带回该操作移除的内容，并保留此后做的一切。已删除的分支或 tag 会回到它原来的 commit（如果名称现在已被占用，会请你换一个，并预先填入一个空闲的名称）；被丢弃的 stash 会放回 stash 列表的最上面；被丢弃或被删除的文件，若此后没有改动就写回原处，否则写在它旁边，名为 `name.restored`（绝不覆盖任何东西）；已移除的远程、上游、ignore 行或 attributes 行会重新加上。reset、amend、rebase、merge 或 commit 会在旧的顶端创建分支 `restore/<branch>-<sha>`：你当前的分支和文件绝不会移动。确认对话框会准确显示它要做什么，而恢复本身是一项新的操作：`Ctrl+Z` 可以撤回它。
- 对已撤销的行可以 **重做到这里**（`y`）。
- **将 main 恢复到“reset”留下的位置**（`o`，或在面板之外使用 `Alt+Shift+Z` / `编辑：将分支恢复到被阻止操作完成后的位置`）：当另一个程序移动了分支并阻止了撤销时，reflog 仍然知道该操作把它留在了哪里。

右键点击某一行（或按 `.`）可以用菜单执行同样的操作。

![删除一个分支并做了两次 commit 之后的撤销历史：已删除分支的那一行被选中，可以单独恢复](../images/undo-history.png)

## Reflog

本地分支的引用菜单有 **Reflog…**；`编辑：Reflog…` 显示 HEAD 的 reflog，撤销历史里也有一个对应的标签页。它列出分支曾经到过的位置，最新的在前：短 SHA、记录的原因、多久以前、是谁，包括在 shell 中做的移动。列表旁边显示所选位置的消息和更改的文件。可用 **在 Log 中显示**（`s`）、**在此创建分支**（`b`）和 **将 `<branch>` 恢复到此位置**（`r`）。这个恢复是可以撤销的。当分支已检出，并且你有本地更改，而移动分支会覆盖它们时，会出现一个下拉框询问如何处理：保留我的更改、Soft、Mixed 或 Hard。Hard 会丢弃你对文件的更改，但会先把它们保存起来，所以 `Ctrl+Z` 仍能带回它们（界面用红色这样提示）。

Reflog 每次读取 200 个位置。更长的 reflog 会自动继续读取：把光标移到列表末尾附近（或滚动到那里）就会获取下一页，还有下一页时，最后一行显示 **… 加载更多**。**加载更多**（`m`，或命令面板中的 `Reflog：加载更多`）会立即获取，并在状态栏显示“已加载的位置：N”。使用 git reftable 格式的仓库的读取方式与其他仓库相同。

![main 的 Reflog：它到过的每个位置，最新的在前，右侧描述所选的位置](../images/reflog.png)

## 丢失的工作

`编辑：找回丢失的 Commits…`（Log 的 commit 菜单里也有）会找出没有任何分支、tag 或 stash 能到达的 commit：reflog 仍然记着的（已删除分支的 commit）、只有撤销固定点还保留着的，以及游离的 commit。列表会立即出现，较慢的游离 commit 扫描在转圈提示下逐步补全；它从不阻塞界面。

![丢失的工作：已删除分支的两个 commit（通过 reflog 找到）和游离的 commit](../images/lost-work.png)

**在此创建分支**（`b`）会以你选的名称找回一个 commit，**cherry-pick**（`c`）把它应用到当前分支，**在 Log 中显示**（`s`）适用于仍可到达的 commit。

## 从命令面板操作

这三个视图的每个行操作也都是命令面板中的一行，所以面板无需鼠标，也无需记住字母。面板打开时按 `Ctrl+P`：命令面板会在它上面打开，其中的 **撤销历史：**、**Reflog：** 和 **丢失的工作：** 行作用于当前屏幕上那个视图中所选的行。

| 视图 | 命令面板行 |
|---|---|
| 撤销历史 | `撤销历史：撤销到这里…`、`重做到这里…`、`仅恢复此项…`、`将分支恢复到原来的位置…`、`刷新`、`关闭` |
| Reflog | `Reflog：在 Log 中显示`、`在此创建分支…`、`恢复到此位置…`、`加载更多`、`刷新` |
| 丢失的工作 | `丢失的工作：在 Log 中显示`、`在此创建分支…`、`cherry-pick`、`重新扫描` |

对应视图未打开时，这些行是灰色的（并给出原因）；三个入口 `编辑：撤销历史…`、`编辑：Reflog…` 和 `编辑：找回丢失的 Commits…` 也会把已打开的面板切换到各自的视图。所选内容做不到的行，会在状态栏里说明原因，而不是什么都不做：“此操作已撤销：请用“重做到这里””、某个条目无法单独恢复的原因、“只有分支或 HEAD 可以移回”，等等。带 `…` 的行会打开与按钮相同的确认或名称提示。

## 哪些操作可以撤销

| 范围 | 操作 |
|---|---|
| commit | commit、amend、reword、AI commit 撰写器（它的所有 commit 算作一步） |
| 分支和 tag | 创建、删除、重命名、设置或取消上游、创建跟踪分支、创建/删除/注解 tag、fast-forward、把 pull request 检出为分支 |
| 切换位置 | 检出分支或 commit（包括分离 HEAD） |
| 历史 | merge（fast-forward 或 mergecommit）、rebase（普通和交互式，完成之后）、cherry-pick、revert、reset（soft、mixed 和 hard）、pull 的本地部分 |
| 工作更改 | stage 和 unstage（文件、文件夹、hunk、行、全部）、丢弃（文件、文件夹、hunk、行、文件检出）、应用补丁、**从磁盘删除文件**、**忽略**（文件、扩展名或文件夹：你所选 `.gitignore` 中的那一行），**视为二进制或文本**（`.gitattributes` 中的那一行） |
| stash | push、apply、pop、drop |
| 远程和配置 | 添加、移除、重命名远程，更改远程的 URL 或 push URL，Git LFS 的 track 和 untrack |
| 服务器上 | **删除远程上的分支或 tag**（从引用菜单、命令面板、`branch delete-remote`、`tag delete-remote`、`tag delete --both`，或会删除远程分支的工作流完成操作）：见下文 |

当 merge、cherry-pick 或 revert 因冲突而停下时，**保存并标记为已解决**、stage 和 unstage 同样可以撤销（每一项都先于其下面的一项被撤回），**中止 merge**（或 cherry-pick、revert）也可以撤销：撤销它会带回整个停下的操作，包括它的冲突（每个未解决文件的三个版本）、已解决并 staged 的文件、工作区中的冲突标记以及 merge 消息；重做则再次中止。一旦操作结束（已 commit 或已中止），stage 和解决就不再可撤销。*unstage*和*全部 unstage*从不触碰仍处于冲突状态的文件：它没有可撤回的 staged 更改。

丢弃或 hard reset 抛弃的所有内容都会先被保存，所以撤销时会带回完全相同的字节，包括未跟踪文件、可执行位和符号链接。撤销一次被丢弃的 stash，会带回同一个 stash，commit id 也相同（它出现在 stash 列表的最上面，即 `stash@{0}`，而不是原来的位置）。撤销移除远程的操作，会带回它的远程跟踪分支、它的 `origin/HEAD` 别名，以及曾经跟踪它的分支的上游设置。

### 删除远程上的分支或标签

删除之前，GitPoulpi 会向服务器询问该分支或 tag 当前所指的内容并记录下来：远程、名称和确切的 commit（对于带注解的 tag，则是 tag 对象本身，所以它的注解得以保留）。只有当服务器仍然保持这个值时，才会进行删除。

**撤销会把它推回去**，名称相同、commit 相同，绝不强制。因为这是对其他人也能看到的服务器的更改，撤销**总是先询问**并说明这一点（“撤销此操作会在 origin 上以 3f2a1c0… 重新创建分支 topic”）；该条目的步骤也会说明（`gitpoulpi undo --dry-run`）。它使用与 push 相同的凭据。远程跟踪分支会随之回来。**重做**会以同样的安全保证再次删除它。

在以下情况下，撤销会拒绝并且不更改任何东西：

- 该分支或 tag 此后在远程上**被重新创建**或被移动：它位于与记录不同的 commit，推送会覆盖别人的工作（拒绝消息会写出两个 commit）；
- 该 commit（或 tag 对象）**已不在你的仓库中**（已被垃圾回收）：无法完成撤销。

“在本地和远程上都删除”是**一个条目**：撤销会一起恢复本地和远程的分支或 tag，重做会再次把两者都删除。只删除远程分支会保留本地分支；撤销时同样如此。同时删除远程分支的工作流完成操作可以整体撤销。

## 哪些操作无法撤销，以及原因

- **服务器上的大多数操作**：push、push 标签、LFS push、加锁和解锁、打开 pull request。这些状态与其他人共享；撤销 push 就等于在他们那边 force-push 或删除，而 GitPoulpi 不会替你这么做。（删除远程分支或标签是例外：撤销只会放回服务器原来持有的内容，并且在那里有任何变化时拒绝。）撤销也绝不会因这类操作而被阻止。
- **Fetch**：它只移动远程跟踪引用，那些引用只是服务器的镜像。下一次 fetch 会再次把它们移动过去。
- **中止 rebase**：不提供撤销（rebase 会移动分支并保留自己的状态；中止本身就是回退的方法）。中止 **merge、cherry-pick 或 revert** *会*被记录，见上文。
- **创建 worktree**、**创建补丁文件**、克隆、初始化、托管操作（pull request、议题）和 AI 调用：它们不属于仓库历史的一部分。
- 从文件菜单**创建新的空文件**、merge 之后自动删除 `*.orig` 文件，以及在内置编辑器中保存文件，都不会被记录。更改 **SSH 命令**（`core.sshCommand`）或**稀疏检出**同样不会（不会丢失任何内容；*关闭* 会找回所有文件）。
- **写入 `.git/info/exclude` 或 `.git/info/attributes`**（忽略和视为二进制/文本列表中的“仅此克隆”选项）：这些文件位于工作区之外，因此不会被记录。
- **在 GitPoulpi 之外对工作区文件所做的修改**（编辑器、脚本、写代码的 AI 代理）绝不会被捕获：它们没有快照，所以在 shell 里执行 `git reset --hard` 或 `git checkout -- file` 会永久丢失未 commit 的修改。shell 的工作中*能*被捕获的，是存在于引用里的部分：见下一节。

## 在 GitPoulpi 之外所做的更改

当你和 AI 代理（或紧挨着 GitPoulpi 的 shell）一起工作时，大部分 git 工作都发生在窗口之外：commit、新分支和 worktree、删除分支、reset、rebase、stash。GitPoulpi 也看得到这些。守护进程保存着它上次看到的引用（本地分支、tag、HEAD、stash 列表、worktree 列表），每当仓库发生变化且不是 GitPoulpi 自己所为时，就在撤销历史里为**每一处连贯的更改记一个条目**，标为**外部**，附上 git 记录的原因（`commit: fix login`、`rebase (finish)`、`reset: moving to HEAD~1`、`checkout: moving from a to b`）和 git 记下的执行者名称。GitPoulpi 分不清是 shell 还是 AI 代理，也从不猜测；条目只说明它是在外部做的。

- **一连串操作算一个条目。** 两次查看之间做的二十个 commit 是一个条目，从第一个值到最后一个值；一次 rebase 是一个条目，在结束时记录（rebase 或 merge 停下来期间不记录任何内容）。
- **已删除的引用会被固定。** 消失的分支、tag 或 stash，会在任何东西能清理它之前，被保留在 `refs/gitpoulpi/undo/` 下它的旧 commit 上，所以 **仅恢复此项**（或 `Ctrl+Z`）能带回代理删除的分支，即使 reflog 已过期、已做过垃圾回收。
- **GitPoulpi 关闭期间。** 快照保存在日志旁边（`.git/gitpoulpi/refs-snapshot.json`），所以没有 GitPoulpi 运行时发生的事，会在下次启动时被发现，并标为“GitPoulpi 关闭期间”。
- **`Ctrl+Z` 对它们的作用和对任何条目一样**：引用移回去，已删除的分支被重新创建，已丢弃的 stash 被重新存下，外部的 checkout 会再次检出旧分支。已检出的分支像 `git reset --keep` 一样移回：文件跟着走，而当你的某个本地更改挡路时，它会**拒绝并且不做任何更改**。重做会重新应用。**仅恢复此项**同样适用：已删除的分支回来，之后的操作保留；被移动的分支会作为 `restore/<branch>-<sha>` 回来。
- **状态栏只提示一次**：当新的外部条目到达且窗口在最前面时，显示“已记录外部更改 - 按 Ctrl+Z 撤销”。
- 由于外部更改现在是最新的条目，旧的“被外部更改阻止”式拒绝基本不会再出现：你先撤销那个外部更改即可。

不会记录的内容：远程跟踪分支仅靠 fetch 产生的移动（它们只是镜像服务器）、GitPoulpi 自己的隐藏引用、没有任何分支变化的 worktree 添加或移除（它所携带分支的条目会写“已添加 worktree <path>”；撤销只移动引用，不会移除 worktree），以及文件内部的一切。`general.journal_outside = false`（设置 > 常规，“记录在 GitPoulpi 之外所做的更改”）会关掉它：不记录任何外部更改，外部所做的更改会像以前一样阻止最新条目。

## 撤销被拒绝时

在动手之前，GitPoulpi 会检查操作所触及的一切是否仍与操作结束时完全一致。如果不一致，它**不会更改任何东西**，并说明哪里变了：

- `main` 位于 3f2a1c0，而不是 Commit 留下的 a81b9e4（比如在 shell 里做了一次 commit）；
- 某个文件、索引、stash 或配置节在此之后被更改过；
- 正在进行 merge、rebase 或 cherry-pick；
- 该操作从未完成，因为运行期间 GitPoulpi 被停止了（该条目被标记为*已中断*，绝不会被自动撤销，`gitpoulpi undo --list` 会显示它原本要更改什么，供你手动修复）。

撤销严格遵循后进先出：被阻止的最新条目也会阻止更早的条目，因为在仍然生效的较新操作之下撤销较旧的操作，等于是在猜测。如果你自己把东西恢复原样，该条目就会重新有效。不触及某个条目所触及内容的更改（在另一个分支上用 shell commit、一次 fetch）不会阻止任何东西。

### 无法清除的被阻止条目：忘记

当最新条目永久卡住时（仓库在 GitPoulpi 之外发生了变化，或者该条目从未完成），它下面的任何条目也都无法撤销。撤销的工具提示和拒绝消息会这样说明，并指出出路：命令面板中的 **编辑：忘记被阻止的撤销条目**（它会先询问），或命令行中的 `gitpoulpi undo-forget`。忘记只会从日志中移除那一个条目；仓库绝不会被触碰，该条目也不能再被撤销或重做。它下面的条目随即成为最新条目。

有两种情况会自行解决：

- 你用普通 git 完成的有冲突的 merge 或 rebase：下次 GitPoulpi 读取仓库状态时，该条目就会关闭；
- 你用 **保存并标记为已解决** 解决的、有冲突的 stash pop 或 apply 的文件：只要已解决的文件仍处于 stage 状态且尚未 commit，撤销就会把冲突放回去。

## 确认

只有存在风险时，撤销才会询问：

- 你要撤回的 commit（或 merge、rebase、cherry-pick、revert）已经在该分支的上游上：撤销它会改写你已发布的历史，你将不得不 force-push 其结果；
- 你有未 commit 的更改，而撤销会移动工作区：你的更改会保留，但如果其中某个更改位于撤销必须重写的文件中，撤销就会停止，不更改任何东西。

- 该条目删除了远程上的分支或 tag：撤销它会**把它推回服务器**（见上文），所以总是询问，再次在那里删除它的重做也一样。

重做，以及仅涉及引用移动的每次撤销，从不询问：反方向只需按一个键。

## 能回退多远

`general.undo_depth`（设置 > 常规）是保留的操作数量。默认值为 **100**；`0` 表示不限制。调低它会立即忘掉最旧的条目。撤销之后开始新的操作，会丢弃你本可以重做的内容，与任何编辑器一样。

历史按 worktree 保存（关联的 worktree 有自己的历史），位于 git 目录中（`.git/gitpoulpi/journal.json`），所以关闭程序后依然存在，也绝不会成为 `git status` 或某次 commit 的一部分。

## 关闭撤销

`general.undo_enabled = false`（设置 > 常规，“启用撤销”）会关闭整个功能（连同外部更改的记录）。不会记录或保留任何内容，因此 stage、丢弃和 commit 会稍快一些。撤销和重做按钮会消失，`Ctrl+Z` 会提示“撤销已关闭”。

**关闭会清除历史**：在关闭期间，任何请求第一次到达守护进程时，日志文件和下文所述的隐藏引用都会被删除。重新开启后，从空历史开始。

## 存储了什么

为了确保撤销永远不会丢失数据，某个条目可能需要的每个 commit、标签对象、stash 或快照，只要该条目还在历史中，就会由 `refs/gitpoulpi/undo/` 下的一个隐藏引用保持存活。它们被排除在 GitPoulpi 自己的列表之外，但在 shell 中执行 `git log --all` 或 `git for-each-ref` 会显示它们，而 `git push --mirror` 会 push 它们（请改用 `git push --all`）。当条目从历史中滑出或撤销被关闭时，它们会被删除。除此之外，绝不会删除任何东西。

## 命令行和 MCP

```
gitpoulpi undo [--dry-run] [--expect <id>] [--yes]
gitpoulpi redo [--dry-run] [--expect <id>] [--yes]
gitpoulpi undo --list [--all]
gitpoulpi undo-forget [--dry-run] [--redo] [--expect <id>] [--yes]
```

- `--dry-run` 描述各个步骤，不更改任何东西。
- `--list` 打印日志，最新的在最后（`--all` 会加上不再可重做但仍保留的条目），并附上每个条目的 id 和状态。
- `--expect <id>` 使调用在该条目不再是最新条目时以*过期*为由拒绝：脚本（或第二个窗口）永远不会撤销它没有看过的东西。
- `undo-forget` 从日志中丢弃被阻止的最新条目（`--redo`：最新的重做条目）。当该条目没有被阻止时它会拒绝，没有 `--yes` 时以退出码 **3** 退出。对 MCP 而言它是变更操作，但不是破坏性的：仓库不会被触碰。
- 退出码 **0** 表示完成，**2** 表示被拒绝（JSON 会给出原因，例如已删除的远程分支被重新创建时为 `RemoteRefMoved`），**3** 表示有风险、需要 `--yes`（撤销删除远程分支或 tag，这会向服务器 push）。

不撤销之后的一切就回退：

```
gitpoulpi undo-restore <id> [--name N] [--dry-run]      （也可写作：undo restore <id>）
gitpoulpi reflog [<ref>] [--skip N] [--limit N]
gitpoulpi reflog-restore <ref> <oid> [--mode keep|soft|mixed|hard] [--dry-run]   （也可写作：reflog restore ...）
gitpoulpi lost-commits [--skip N] [--limit N] [--no-deep] [--budget-ms MS]
```

- `undo --list` 会为每个条目显示 **仅恢复此项** 能带回什么（`restore.items`），或为什么什么也带不回（`restore.not_restorable`），以及是谁做的：`"source": "gitpoulpi"`，或 `"source": "outside"` 并带有 `outside.message`（git 的原因）和 `outside.committer`，表示是 shell 或 AI 代理做出的更改。`undo`、`redo` 和 `undo-restore` 对待外部条目和其他条目一样。
- `undo-restore <id>` 带回那一个条目移除的内容（已删除的分支或 tag、已丢弃的 stash、已丢弃或已删除的文件、连同其远程跟踪分支和上游一起被移除的远程、ignore 行），并保留之后的每一项操作。它绝不覆盖：此后改过的文件，保留的版本会放在它旁边，名为 `<name>.restored`；分支名称已被占用会被拒绝（退出码 2）并建议一个空闲的名称，用 `--name` 传入；在 reset、amend 或 rebase 之后，它会在旧的顶端创建一个新分支，绝不移动你的分支。这个恢复是一个日志条目，所以 `gitpoulpi undo` 可以撤回它。
- `reflog` 列出 `HEAD` 或某个分支曾经在哪里，附带 git 记录的原因，包括 shell 做过的事。`reflog-restore` 把分支（或 `HEAD`）移到其中一个位置；已检出且有未 commit 更改的分支需要 `--mode`。它同样会写入日志。
- `lost-commits` 列出没有任何分支、tag、远程或 stash 能到达的 commit。
- 当 shell 中的移动阻止了最新条目时，拒绝信息会带上持有该条目所期望状态的 reflog 位置（`reason.RefMoved.restore`），把它传给 `reflog-restore` 即可。

有两个命令的存在，是为了让上述操作在每个界面上都可撤销：`gitpoulpi rm-file <paths>`（从工作区删除文件）和 `gitpoulpi ignore <pattern> [--file <path/.gitignore>]`（向 `.gitignore` 追加一行；该行已存在时什么也不会发生；撤销会把文件恢复原状，如果文件是该命令创建的则将其删除，如果文件在此之后被编辑过则拒绝）。

MCP 服务器为每个命令提供一个工具，所以 `undo`、`redo`、`rm-file` 和 `ignore` 也都是工具。`undo`、`redo` 和 `rm-file` 被归为破坏性（与 `reset` 一样受门控，参见 [MCP 与代理](mcp-and-agents.md)）；`ignore` 是普通的变更操作。日志是仓库中的一个文件，所以在 shell 中运行 `gitpoulpi undo` 会撤回窗口所做的事，窗口也可以撤销脚本所做的事。
