[English](../multiple-remotes.md) · [Čeština](../cs/multiple-remotes.md) · [Deutsch](../de/multiple-remotes.md) · [Español](../es/multiple-remotes.md) · [Français](../fr/multiple-remotes.md) · [Italiano](../it/multiple-remotes.md) · [日本語](../ja/multiple-remotes.md) · [한국어](../ko/multiple-remotes.md) · [Nederlands](../nl/multiple-remotes.md) · [Polski](../pl/multiple-remotes.md) · [Português (Brasil)](../pt-BR/multiple-remotes.md) · [Português (Portugal)](../pt-PT/multiple-remotes.md) · [Русский](../ru/multiple-remotes.md) · [Svenska](../sv/multiple-remotes.md) · [Türkçe](../tr/multiple-remotes.md) · [Українська](../uk/multiple-remotes.md) · **简体中文**

# 一个分支对应多个远程

许多仓库不止存在于一个托管平台：一个主要的（`origin`），另一个平台上的镜像，一份备份。GitPoulpi 让一个分支一次点击就 push 到所有这些远程，还可以选择从多个远程 pull，而不改变普通 git 的行为。这里的一切都不需要账户，所有内容都保存在仓库自己的 `.git/config` 中，所以普通 git 和其他工具照常工作。

## Push 目标与 pull 来源

- **upstream** 是 git 自己的概念（`branch.<name>.remote` 和 `merge`）。它是该分支的*主*远程。GitPoulpi 绝不改变它的工作方式。
- **push 目标**是该分支还会 push 到的额外远程。Push 按钮会把分支发送到 upstream 和每个 push 目标。
- **pull 来源**是 pull 时也会从中获取 commit 的额外远程。除非你开启（**也 Pull** 复选框），否则它是关闭的：谁也不希望默认 merge 两个自己没有选择的远程。

目标可以在该远程上使用另一个分支名（`backup` 上是 `feat-x`）。

## “远程”对话框

右键点击分支，选择 **<分支> 的远程…**（Push 下拉菜单中也有，命令面板中是 `Git：分支的远程…`，也可以点击 Refs 区中该分支行上的 `⇉3` 徽标）。它列出仓库的每个远程：

| 列 | 含义 |
|---|---|
| 远程、分支 | 远程以及它上面的分支名。**在远程上重命名…** 会修改所选行的这个名称。 |
| Push | 勾选后，该远程成为此分支的 push 目标。 |
| 也 Pull | 勾选后，pull 时也会整合该远程。upstream 总是会被 pull。 |
| 状态 | 分支相对于它的位置：已同步、领先、落后、新建、无关。 |

标有“远程已不存在”的行，指的是该分支仍然提到、但仓库已不再有的远程；**移除** 会忘掉它。同一个对话框也用来设置 upstream（**Upstream** 列表），**取消设置 upstream** 会清除它。**应用** 会把所有内容作为一次可撤销的更改写入。

仓库也可以让每个新分支都带上同样的额外目标：**设置 > Git 与编码**（仓库专属页面）中的“新分支的额外 push 目标”（每个远程一个复选框）。它们会在分支首次获得 upstream 时复制到该分支；已存在的分支不受影响。

## Push

- **Push 按钮**：对有额外目标的分支，会 push 到所有目标。当每个远程都只是接收新 commit 时，会立即执行，进度视图里每个远程依次变成对勾。如果有事情需要决定（某个远程有你没有的 commit），则改为打开计划对话框。
- **Push ▾**（按钮旁边的箭头；当设置中的 **Push 按钮** 为 **询问** 时，按钮本身也是它）：
  - `Push 到全部（origin, gitlab, backup）`
  - `仅 Push 到 <远程>`，该分支的每个远程各一项
  - `Push 到所选远程…`：对任何分支都有，即使它只有一个远程
  - `<分支> 的远程…`
- 分支的右键菜单和命令面板（`Git：Push 到所有目标`、`Git：Push 到远程…`、`Git：Push 到所选远程…`）提供同样的操作，名称相同。

**Push 到所选远程…** 会打开仓库每个远程的列表，其中分支的 upstream 和目标已被勾选。它只对这一次 push 有效；勾选 **为此分支记住**，就会把这个选择保存为分支的 push 目标。

### 计划

分组 push 之前，GitPoulpi 会查看每个远程（一次快速查询，仅当该远程有差异时才从中下载），这样第三个远程上的问题会在第一次 push 开始之前就显示出来。对话框列出每个远程、分支相对于它的位置以及将要做的事：

| 情况 | 默认操作 | 另外提供 |
|---|---|---|
| 远程上没有该分支，或缺少一些 commit | Push | 跳过 |
| 远程更靠前（你没有可给它的新内容） | 从它 fast-forward | 跳过 |
| 远程不同，因为你 rebase 或 amend 了已 push 的内容 | 强制 push（lease） | Merge、Rebase、跳过 |
| 远程有别人的 commit | 从它 Merge（当 **Pull 按钮** 设为 rebase 时为 Rebase） | Rebase、强制 push（lease）、跳过 |
| 历史与你的没有任何共同之处 | 跳过 | 强制 push（lease） |
| 托管平台拒绝了该分支（受保护） | 跳过 | 无：强制 push 同样会被拒绝 |

每一行的操作是一个可以更改的列表（`Left`/`Right` 逐项切换，`Enter` 打开它，`s` 跳过该行）。merge 会显示是否会冲突以及在哪些文件中冲突。**强制 push** 总是要确认，会指明远程以及它将替换多少个 commit，总是使用 lease（绝不使用裸的 force），并且无法撤销：结果会说明哪个 commit 被替换，以及把它恢复回来的命令。**跳过有问题的** 会把所有有问题的行设为跳过。**运行**（`Ctrl+Enter`）开始执行。

merge、fast-forward 和 rebase 先按列表顺序执行，所以你的分支只会到达一次最终状态；然后 push 一次一个远程地发出，upstream 在最前。跨远程没有原子的“要么全部成功要么全部不做”：每个远程各有自己的结果，一个远程上的失败（网络、钩子）不会阻止其他远程。**重试失败项**（或 `Git：重试失败的 push`）会再次尝试这些远程。关闭进度视图绝不会停止运行；`Git：显示分组 Push` 可以把它调回来。

### 冲突与“继续”

如果运行中的某次 merge 或 rebase 因冲突而停止，对话框会关闭，由普通的 merge 横幅和冲突编辑器接手。Log 上的提示条写着 `分组 push 已暂停（gitlab）：请解决并完成冲突，然后点击“继续”。`，带有 **继续** 和 **中止分组** 按钮。

- 解决冲突并完成 merge 或 rebase（在 GitPoulpi 中或用普通 git）。
- 在完成之前，**继续** 是灰色的，而且**绝不会替你按下**：没有任何东西会自行继续。按下它（或运行 `Git：继续分组 Push`），运行就会从你的新尖端再次 push 到每个远程。rebase 之后，仍持有旧 commit 的远程会回到计划对话框（它现在需要强制 push），而不是被悄悄强制覆盖。
- **中止分组**（需确认；`Git：中止分组 Push`）会丢弃运行所启动的 merge 或 rebase，并忘掉这次运行。已完成的 push 保持不变。

提示条在 GitPoulpi 重启后依然存在，CLI 看到的是同一次已保存的运行。

## Pull

- **Pull** 按钮和以前一样 pull upstream，除非该分支有 pull 来源：此时它会依次 pull upstream 和每个来源，每个都使用设置中 **Pull 按钮** 指定的模式（merge、仅 fast-forward 或 rebase）。无法 fast-forward 的来源会像 upstream 一样被 merge（或 rebase）；在仅 fast-forward 模式下，需要更多操作的来源会先询问。
- 有来源时，**Pull ▾** 会增加 `从全部来源 Pull（origin + gitlab）`，并为该分支的每个远程增加 `仅从 <远程> Pull`：一次性 pull 该远程的分支，不管复选框怎么设置。
- 命令面板中有 `Git：从所有来源 Pull` 和 `Git：从一个远程 Pull…`。
- 因冲突而停止的 pull 会留下同样的提示条，标记为分组 pull。

## 设置

这四项都在 **设置 > Git 与编码** 中。

| 设置 | 默认 | 含义 |
|---|---|---|
| **Push 按钮** | 直接 | 直接是立即 push；询问是先打开 Push 菜单。 |
| **Pull 按钮** | merge | Pull 按钮的模式（merge、仅 fast-forward 或 rebase），也是 pull 来源的模式。 |
| **push 前检查每个远程** | 开 | 分组 push 之前先查看每个远程（CLI 遵循它；`--no-preflight` 为单次调用关闭它）。 |
| **新分支的额外 push 目标**（每个仓库） | 无 | 新分支还会 push 到的远程。 |

## 命令行

```
gitpoulpi branch upstream add gitlab --pull      # a push target that is also a pull source
gitpoulpi branch upstream list                    # what the branch is set up for
gitpoulpi push --all-targets                      # upstream plus every target
gitpoulpi push --to gitlab --to backup:feat-x     # exactly these, this once
gitpoulpi pull --all-sources                      # upstream plus every pull source
gitpoulpi remote-group plan                       # what a grouped push would do, nothing changed
gitpoulpi remote-group continue                   # after resolving and concluding a stop
```

当某个远程需要决定时，命令会用计划作答（退出码 3）并且不改动任何内容；用 `--resolve gitlab=merge` 回答它。每个标志见 [CLI 参考](cli-reference.md)。MCP 服务器有同样的工具。

## 配合普通 git

目标是普通的配置键，所以脚本或别名不需要 GitPoulpi 也能使用它们：

```
git config --get-all branch.feature.gitpoulpi-push-to     # one value per line: remote or remote:branch

# push the current branch to its upstream and to every target
git config alias.pushall '!b=$(git symbolic-ref --short HEAD); git push; for t in $(git config --get-all branch.$b.gitpoulpi-push-to); do r=${t%%:*}; d=${t#*:}; [ "$d" = "$t" ] && d=$b; git push "$r" "$b:$d"; done'
```

值是 `<remote>`（分支名相同）或 `<remote>:<branch>`，`d=${t#*:}` 这一行就是在区分它们。这个普通别名不会先查看各远程，不提供 merge，冲突之后也无法继续：这些正是对话框的用处。

## 须知

- 指向同一地址的两个远程会被发现（“gitlab 和 mirror push 到同一个 URL”）；默认会跳过第二个。
- 有受保护分支的托管平台会拒绝强制 push；该行会这样说明，并且只提供跳过。
