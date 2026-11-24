[English](../lfs.md) · [Čeština](../cs/lfs.md) · [Deutsch](../de/lfs.md) · [Español](../es/lfs.md) · [Français](../fr/lfs.md) · [Italiano](../it/lfs.md) · [日本語](../ja/lfs.md) · [한국어](../ko/lfs.md) · [Nederlands](../nl/lfs.md) · [Polski](../pl/lfs.md) · [Português (Brasil)](../pt-BR/lfs.md) · [Português (Portugal)](../pt-PT/lfs.md) · [Русский](../ru/lfs.md) · [Svenska](../sv/lfs.md) · [Türkçe](../tr/lfs.md) · [Українська](../uk/lfs.md) · **简体中文**

# Git LFS

Git LFS（“Large File Storage”，大文件存储）把大文件（设计文件、视频、数据集）挡在仓库历史之外：git 只存储一个三行的文本*指针*，真正的内容保存在 LFS 服务器上，以及 `.git/lfs` 下的本地缓存中。

这一切都不需要 GitPoulpi 账号。LFS 使用你的远程仓库本来就在用的凭据（git 凭据助手或 ssh-agent）。

## 哪些操作需要 `git-lfs` 程序，哪些不需要

凡是转换或移动文件的操作（install、track、fetch、pull、push、锁），GitPoulpi 都会运行标准的 `git-lfs` 程序。

读取 LFS *状态*则不需要该程序：仓库是否使用 LFS、哪些文件是指针、它们有多大，以及真正的对象是否已存储在本地。

| | 需要 `git-lfs` |
|---|---|
| `gitpoulpi lfs status`（程序缺失时会如实报告，而不是失败） | 否 |
| `gitpoulpi lfs files` | 否 |
| install / track / untrack / prune | 是 |
| fetch / pull / push | 是 |
| 锁 | 是 |

## 安装 `git-lfs`

- **Windows**：随 Git for Windows 附带；或者使用 `winget install GitHub.GitLFS`。
- **Debian / Ubuntu**：`apt install git-lfs`。
- **macOS**：`brew install git-lfs`。
- 其他系统：https://git-lfs.com

需要 3.0 或更高版本。安装之后，GitPoulpi 会在一分钟内发现它（执行 `lfs status` 时则立即发现）。

## `git-lfs` 缺失时

GitPoulpi 绝不会因此崩溃或退出：

- `gitpoulpi lfs status` 仍会成功，并显示 `git-lfs: not installed`，同时给出安装链接。
- 需要该程序的操作会失败，给出以安装链接（`https://git-lfs.com`）结尾的清晰提示，并返回非零退出状态；通过 MCP 时，工具会返回该提示。
- 使用 LFS 的仓库仍可正常打开；在安装程序之前，它的大文件只会显示为指针。

## 查看 LFS 文件

```
gitpoulpi lfs status            # program, patterns, filters, hook
gitpoulpi lfs files             # pointer files in the index, with size and local/not downloaded
gitpoulpi lfs files --at v1.0   # ... as of a revision
gitpoulpi show big.bin --at HEAD  # reports "Git LFS object, 12.3 MB, sha256:..." instead of pointer text
```

## 设置仓库：install、track、untrack

```
gitpoulpi lfs install                 # filters + hooks for THIS repository (git lfs install --local)
gitpoulpi lfs install --user          # into your global git config instead (never done silently)
gitpoulpi lfs track '*.psd'           # route *.psd through LFS
gitpoulpi lfs track '*.psd' --lockable   # ... and make them read-only until locked
gitpoulpi lfs track '*.bin' --renormalize  # ... and re-stage already-committed *.bin as pointers
gitpoulpi lfs untrack '*.psd'
gitpoulpi lfs prune                   # delete old, already-pushed objects from the local store
```

- `install` 默认写入仓库自己的 `.git/config` 和钩子，所以除非你传入 `--user`，否则绝不会改动你的全局 git 配置。
- `track` 和 `untrack` 会编辑 `.gitattributes`，并让它保持 **unstaged**：它会显示为一项普通的工作区更改，由你自行 commit。在跟踪*之后*添加的文件，会在下一次 commit 时以指针形式存储。
- **已经**以普通内容 commit 的文件在历史中仍保持普通内容。`--renormalize` 会重新 stage 它们，使你的下一次 commit 用指针替换它们。要在过去的每个 commit 中也把它们迁移过去，请使用历史重写规则*将文件迁移到 LFS*（见[重写历史](history-rewrite.md#将文件迁入或迁出-git-lfs)）；它是 GitPoulpi 自己的实现，不是 `git lfs migrate`。
- 请给模式加上引号，以免被 shell 展开。
- 通过 MCP 时，这些都是普通的变更操作（`lfs_install`、`lfs_track`、`lfs_untrack`、`lfs_prune`），受 `config.toml` 中 `[mcp]` 门控的约束。

## 传输对象：fetch、pull、push

```
gitpoulpi lfs fetch [--remote origin] [--all]   # download objects into the local store; working tree untouched
gitpoulpi lfs pull  [--remote origin]           # fetch, then replace pointer files with the real content
gitpoulpi lfs push  [--remote origin] [--all]   # upload the current branch's objects (or every ref with --all)
```

- 与 `fetch` 和 `push` 一样，远程仓库默认为仓库的默认远程。
- 凭据是你的远程仓库通常使用的那些（凭据助手、ssh-agent）。GitPoulpi 从不要求自己的账号。
- 如果缺少 LFS 过滤器，`pull` 会为此仓库安装它们：没有过滤器时，git-lfs 会下载对象但不替换指针，之后再次检出又会把指针放回来。
- MCP 把 `lfs push` 与 `push` 同样对待：属于破坏性工具（它会改变远程仓库）。
- 窗口会在状态栏上显示传输进度（文件数和字节数）。CLI 则等待结果。

### 检出内容：`lfs checkout`

```
gitpoulpi lfs checkout [<path>...]   # offline: write real content over the pointer files, from the local store
```

它从不下载（那是 `lfs pull` 的工作）；本地存储中缺失的对象仍保持为指针。与 `pull` 一样，如果没有配置任何 LFS 过滤器，它会先为仓库安装。与所有 LFS 操作一样，它不可撤销；参见[撤销与重做](undo-redo.md)。通过 MCP 时，它是 `lfs_checkout`（写入操作，非破坏性）。

### 工具栏的 LFS 按钮，以及 pull / push

工具栏在 **弹出** 之后有一个 **`LFS ▾`** 按钮（`git.lfs_enabled` 关闭时隐藏，在 LFS 既未初始化、其 `.gitattributes` 也没有使用 LFS 的仓库中同样隐藏：参见[设置 > LFS](settings.md#仓库专属lfs)）。点击会打开下拉菜单：**Fetch 所有 LFS 文件**（`lfs fetch --all`）、**检出所有 LFS 文件**（`lfs checkout`）、**Pull 所有 LFS 文件**（`lfs pull`）、**Push 所有 LFS 文件**（`lfs push --all`，即所有引用的对象）和**清理本地 LFS 存储**（`lfs prune`）。每一行也都是命令面板中的命令（`LFS：Fetch 所有对象`、`LFS：检出所有文件`、`LFS：pull 对象`、`LFS：Push 所有对象`、`LFS：清理本地对象`）。`LFS：打开菜单` 会打开下拉菜单。每项都有一个 `Ctrl+G` 键序：`l` 对应菜单，然后 `Shift+F`、`Shift+K`、`Shift+L`、`Shift+J` 和 `Shift+V` 对应各行。你可以在*设置 > 键盘与鼠标*中更改它们。缺少 `git-lfs` 时，每一行都会引导到安装对话框。

![工具栏的 LFS 下拉菜单：fetch、检出、pull 和 push 所有 LFS 文件，以及清理本地存储](../images/lfs-menu.png)

**Pull 和 push 会自行处理 LFS。**在使用 LFS 且已安装 `git-lfs` 的仓库中，普通的 `pull` 会先检查仓库的 LFS 过滤器是否已安装，普通的 `push` 则检查它的 `pre-push` 钩子。缺少其中之一时，GitPoulpi 会在传输之前运行 `git lfs install --local`（仅针对此仓库），状态栏会显示“已安装 LFS 过滤器”。没有这个钩子，push 将不会上传任何 LFS 对象。可以用 `git.lfs_autofix_filters = false` 关闭此行为（*设置 > Git 与编码 > 在 Pull 和 Push 之前安装 LFS 过滤器*）。普通的 `push` 仍然只上传被 push 分支的对象，绝不会上传所有引用的对象；清理（prune）从不会隐式执行。

## 文件锁

有些文件无法 merge（Photoshop 文件、关卡、二进制资源）。Git LFS 的*锁*让一个人可以声明“我正在编辑它”：在解锁之前，其他人既不能锁定它，也不能 push 对它的更改。

```
gitpoulpi lfs locks  [--remote origin]              # who holds which file (yours are marked)
gitpoulpi lfs lock   assets/hero.psd [--remote origin]
gitpoulpi lfs unlock assets/hero.psd [--remote origin] [--force]
```

- 加锁是 **LFS 服务器**的功能（GitHub、GitLab 和 Gitea 都支持）。它使用远程仓库自己的凭据访问；GitPoulpi 没有自己的账号。
- 不支持加锁的服务器会让 `lock` 失败，并给出相应说明（类型化错误 `LfsLockingUnsupported`）。在这样的服务器上，`locks` 不会列出任何内容，这也正是 git-lfs 本身的报告。
- 用 `--lockable` 跟踪某个模式（`lfs track '*.psd' --lockable`）会使这些文件在你加锁之前保持只读。
- 对他人的锁执行 `unlock` 会被拒绝；`--force` 会强行解除，所以 MCP 把 `lfs_unlock` 视为破坏性工具。`lfs_locks` 是只读的。

## 克隆 LFS 仓库

`gitpoulpi clone <url>`（以及 TUI 的克隆对话框）会在*不*运行 LFS 过滤器的情况下检出仓库；然后，当仓库使用 LFS 且已安装 `git-lfs` 时，用 `git lfs pull` 下载对象（如果没有配置任何过滤器，则先为该仓库安装），并把它作为独立阶段显示：*正在下载 LFS 对象*。`--no-lfs` 会跳过这一步，让文件保持为指针。

克隆**绝不会因 LFS 而失败**：即使 `git-lfs` 缺失或下载失败，克隆仍会成功，大文件保持为指针；仓库的 LFS 状态会说明原因，待问题解决后，`gitpoulpi lfs pull` 可以完成余下的工作。

## 在终端界面中

- **`LFS` 标记。**以 Git LFS 存储的文件，在状态窗格的 unstaged 与 staged 列表中，以及 commit 的变更文件列表中，名称后面都有一个小小的 `LFS` 标记。**仅 LFS**（焦点在状态窗格或 Log 时按 `o`、文件列表工具栏中的复选框，或命令面板中的 **文件：切换仅 LFS**）会把这些列表缩小为只含 LFS 文件。该复选框位于筛选行的右端（状态窗格：在工具栏中，窗口较窄时在其溢出菜单中）；在 commit 的文件列表中，窗格较窄时它会缩短为 `[ ] LFS`，而不是消失。只有当列出的文件中有 LFS 文件时才显示。在状态窗格和 commit 的变更文件中，当一个文件的路径在索引中是指针时，它就算 LFS 文件（每次刷新只调用一次批量的 `git`，没有 `filter=lfs` 属性的仓库则完全不调用）。在 Log 中开启**查看所有文件**时，会列出所选修订的整棵树，其中的 LFS 文件取自该修订自己的 blob（按修订缓存，在渲染线程之外读取，不需要 `git-lfs` 程序），因此在那里是 LFS、但不在索引中的文件也会被标记，并被该筛选保留。
- **diff 卡片。**打开 LFS 文件的 diff 时，显示的是卡片而不是指针文本：`Git LFS 对象`、`1.2 MB -> 1.4 MB`、新旧 oid 的短形式，以及新对象是已存储在本地还是`尚未下载`。按 `r` 查看原始指针 diff，再按 `r` 返回。LFS 文件只适用文件级 stage（没有可 stage 的 hunk 或行）。卡片下方的 **用默认程序打开** 和 **打开方式**（`o`、`Shift+O` 或命令面板）会打开真实文件，而不是指针：参见[打开方式](open-in.md#git-lfs-文件)。
- LFS 文件的**文件视图**会显示 `Git LFS 对象, 12.3 MB, sha256:ab12cd34...`，以及获取内容的方法。
- **横幅。**对于使用 LFS、但 `git-lfs` 缺失或低于 3.0 的仓库，状态窗格中会有一条横幅说明情况。点击它（或使用命令面板项 *LFS：打开安装页面*）即可打开 https://git-lfs.com。其他一切不变：窗口照常打开，diff 照常工作，大文件只是显示为指针。
- **文件菜单和命令面板。**在状态窗格中右键点击已更改的文件：对普通文件是*用 LFS 跟踪：此文件* / *此扩展名*；对 LFS 文件，则根据锁的持有者显示*锁定文件（LFS）*、*解锁文件（LFS）*或*强制解锁（LFS，他人的锁）*。已加锁文件的行会显示 `被 你 锁定` / `被 <name> 锁定`。所有这些也都是命令面板中的命令（`Ctrl+P`，输入 `LFS`）：初始化、跟踪或取消跟踪某个模式、fetch、pull 或 push 对象、清理、显示锁、打开安装页面。
- **失败的操作会自己解释原因。**如果某个 LFS 操作因 `git-lfs` 缺失而失败，对话框会带上相应提示和一个*打开安装页面*按钮。
- **设置。***设置 > LFS*（按仓库）显示 LFS 是否已初始化，列出被跟踪的模式，并有 **在此设置 LFS**。*Git 与编码*（按仓库）中有 **Git LFS** 开关、只读的 **git-lfs 程序**行，以及**为此仓库初始化 LFS**按钮。
- 关闭 **Git LFS**（`git.lfs_enabled = false`，参见[配置参考](config-reference.md)）会隐藏以上所有内容。git 自带的 LFS 过滤器仍会运行。
- LFS 状态会在仓库打开后不久、每次更改之后、手动刷新时、`.gitattributes` 更改时读取一次；对使用 LFS 的仓库，还会每十秒读取一次。

## 已知限制

- 如果仓库的 LFS 模式只存在于*嵌套*的 `.gitattributes` 中（不在根目录的 `.gitattributes`，也不在 `.git/info/attributes`），则要等它有了 LFS 对象存储（`.git/lfs`）之后，才会被识别为使用 LFS，在此之前不会。
- `git lfs migrate` 本身不会被使用。在整个历史中把文件迁入或迁出 LFS，由历史重写的*将文件迁移到 LFS* / *从 LFS 取回文件*规则完成：它们作用于本地克隆，只写根目录的 `.gitattributes`，之后仍需 push 这些对象（结果面板中的*发送 LFS 对象*，或 `lfs push --all`）。
