[English](../branching-workflows.md) · [Čeština](../cs/branching-workflows.md) · [Deutsch](../de/branching-workflows.md) · [Español](../es/branching-workflows.md) · [Français](../fr/branching-workflows.md) · [Italiano](../it/branching-workflows.md) · [日本語](../ja/branching-workflows.md) · [한국어](../ko/branching-workflows.md) · [Nederlands](../nl/branching-workflows.md) · [Polski](../pl/branching-workflows.md) · [Português (Brasil)](../pt-BR/branching-workflows.md) · [Português (Portugal)](../pt-PT/branching-workflows.md) · [Русский](../ru/branching-workflows.md) · [Svenska](../sv/branching-workflows.md) · [Türkçe](../tr/branching-workflows.md) · [Українська](../uk/branching-workflows.md) · **简体中文**

# 分支工作流

分支工作流告诉 GitPoulpi 你的团队如何使用分支：哪些分支长期存在（`main`、`develop`、`production`），哪些是短期的主题分支（`feature/...`、`hotfix/...`），它们如何命名，以及每一个从哪里开始、到哪里结束。它是可自定义的 Git Flow：可以用固定名称、前缀和后缀，也可以用带占位符的模式，这样机器人根据议题创建的分支会和你自己的分支一样被识别。

本页涵盖：模型、文件、预设、检测与导入、只读命令、开始、完成、同步和提升分支的操作（CLI、MCP 和应用），以及应用中的各个界面：设置编辑器、分组的侧栏、`Flow` 菜单及其对话框。

## 概念

- **基础分支**是长期存在的。基础分支可以有**父分支**（`develop` 合入 `main`）；主干没有。一个基础分支可以列出多个候选名称（`main` 或 `master`，`develop` 或 `dev`）：哪个存在就用哪个。
- **主题类型**是一类短期分支（feature、bugfix、release、hotfix、support……）。它有一个**名称模式**、一个**父分支**（从哪里开始，以及完成时合入哪里）和一种完成方式。
- 每个分支都会被**分类**：基础分支（按名称）、某个类型的主题分支（按模式），或未分类。
- **分支的父分支**依次为：GitPoulpi 创建该分支时记录的那个（`git config branch.<name>.gitpoulpi-parent`），否则是存在时 Git Town 的 `git-town-branch.<name>.parent`，否则是其类型的父分支。

## 存放位置

| 内容 | 位置 | 是否 commit |
|---|---|---|
| 共享的工作流 | 仓库中的 `.gitpoulpi/workflow.toml` | 是，由你来做：GitPoulpi 只在你要求时才写入它 |
| 你自己的调整 | `.gitpoulpi/repo-state.toml` 中的 `[workflow]` | 否（该文件被忽略） |

状态目录会忽略自身（`.gitpoulpi/.gitignore`），`workflow.toml` 除外，它本来就是要 commit 的。生效的工作流是共享文件再加上你的覆盖项：同名的基础分支或同 id 的主题类型会被替换；新的会被添加；`disable_bases` 和 `disable_topics` 只为你自己移除共享的条目。

覆盖保存已有文件时，会保留此版本不认识的键（更新版本的 GitPoulpi 或队友写入的），但不保留注释和键的顺序。

## 文件

```toml
template = "git-flow"          # git-flow | github-flow | gitlab-flow | trunk | custom（仅供参考）

[[base]]
name = "main"
names = ["master"]             # 同一分支的其他写法
protected = true               # 永不删除，直接 commit 会被视为可疑

[[base]]
name = "develop"
names = ["dev"]
parent = "main"
merge = "merge"                # merge | squash | rebase | fast-forward
tag_on_merge = false
# environment = true           # 环境分支（GitLab Flow）：向下游提升

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
patterns = ["features/{slug}"] # 同样匹配的更多名称；绝不用于创建
# regex = "^feat-(?P<slug>.+)$"  # 同样匹配；命名分组会成为字段
parent = "develop"             # 从哪里开始，也是默认的完成目标
finish = [{ into = "develop", merge = "squash" }]
finish_mode = "merge-request"  # local-merge | merge-request | ask
delete_after_finish = true

[hooks]                        # 需主动启用的 shell 命令，运行前总会显示
after_start = []
before_finish = ["cargo test"]

[issue_flow]                   # 由托管平台驱动的流程，见下文
mode = "local"                 # local | host-creates-branch | bot
```

`merge`、`finish`、`finish_mode` 和 `hooks` 驱动下面的[操作](#操作)；`issue_flow` 驱动[由托管平台驱动的流程](#由托管平台驱动的流程)。

### 名称模式

带 `{占位符}` 的文本：

| 占位符 | 匹配 | 渲染来源 |
|---|---|---|
| `{slug}` | 自由文本；位于最后时可以包含 `/` | 该文本，转为小写、折叠重音符号、词之间用 `-`，最多 50 个字符 |
| `{issue}` | 数字（`123`） | `123` 或 `#123` |
| `{issue_key}` | 跟踪器中的 Key（`PROJ-123`） | 原样 |
| `{user}` | 不含 `/` 的名称 | 转为 slug |
| `{date}` | `2026-10-16` 或 `20261016` | 原样 |
| `{version}` | `1.2.0`、`v1.2`、`1.2.0-rc1` | 原样 |
| `{n}` | 数字 | 原样 |

其他任何内容都是字面文本，所以前缀和后缀就是普通文字：`feature/{slug}`、`{user}/{issue}-{slug}-wip`、`bot/{issue_key}`。每个占位符只能出现一次，两个占位符之间需要有文字。模式必须能产生有效的分支名（`git check-ref-format` 的规则）：没有空格，没有 `~ ^ : ? * [ \`，没有 `..`，没有以 `.` 开头或以 `.lock` 结尾的组成部分。

`regex` 只用于匹配（两端都是锚定的）；用它来表示任何模式都无法表达的名称。命名分组会成为该分支的字段。

### 哪个主题胜出

1. 名称与某个基础分支相同（它的任一名称）的分支就是该基础分支。
2. 否则，每个主题类型中每一个匹配的模式都是候选：先 `pattern`，再 `patterns`，再 `regex`，按文件中的顺序逐个主题类型处理。固定的字面字符最多的那个胜出（`release/{version}` 胜过 `{user}/{slug}`，后者又胜过单独的 `{slug}`）；regex 按其字面前缀计算。平局时取文件中靠前的。
3. 否则该分支为未分类。

损坏的模式或 regex 会作为警告报告并被跳过；其余部分继续工作。

## 模板

| 模板 | 基础分支 | 主题类型 |
|---|---|---|
| `git-flow` | `main` 或 `master`；`develop` 或 `dev`（父分支为 main） | `feature/` 和 `bugfix/`（从 develop 开始），`release/`（从 develop 开始，合入 main 和 develop，并打 tag），`hotfix/`（从 main 开始，合入 main 和 develop，并打 tag），`support/`（从 main 开始）；复数写法（`features/`、`releases/`……）也会匹配 |
| `github-flow` | `main` | `feature/` 或任意名称，从 main 开始，向 main 发起 merge request，squash，之后删除 |
| `gitlab-flow` | `main`；`pre-production` 和 `production`（环境分支，依次为 main、pre-production、production） | `feature/` 或任意名称，从 main 开始；`release/{version}` 作为 cherry-pick 目标 |
| `trunk` | `main` | 短期的 `{user}/{slug}`，从 main 开始，rebase |
| `custom` | 无 | 无 |

环境分支也会匹配 `preprod`、`staging` 和 `prod`。模板只是一个起点：用 `flow config init` 输出它，再编辑文件。

## 检测与导入

`gitpoulpi flow detect` 会针对仓库现有的分支给各模板排序，给出满分 100 的得分和找到的具体名称（`main` 对应的 `master`，`develop` 对应的 `dev`，`feature/` 对应的 `features/`）。基础分支占 60 分，模板的特定模式所识别的其他分支所占比例占 40 分，另一个模板的长期分支（GitHub Flow 候选中出现的 `develop`）会扣 25 分。

`gitpoulpi flow import --from git-flow` 会从仓库自己的 `.git/config` 读取经典 git-flow 设置（`gitflow.branch.master`、`gitflow.branch.develop`、`gitflow.prefix.feature|bugfix|release|hotfix|support|versiontag`）；`--from git-town` 读取 `git-town.main-branch` 和 `git-town.perennial-branches`（每个分支的父分支 `git-town-branch.<name>.parent` 会被当作已记录的父分支接收）。不会悄悄应用任何东西：命令输出 TOML，只有 `--write` 才会创建 `.gitpoulpi/workflow.toml`，且仅在它不存在时。

## 命令

它们都接受 `--json`；MCP 服务器把它们作为工具提供（`flow_status`、`flow_templates`、`flow_detect`、`flow_config_show` 为只读；操作见下文）。

| 命令 | 作用 |
|---|---|
| `gitpoulpi flow status` | 每个本地和远程分支：种类（基础分支、主题分支、未分类）、主题类型和字段、父分支及其来源（`recorded`、`git_town`、`type`、`base`），对本地分支还有它领先（`ahead`）和落后（`behind`）父分支多少个 commit。还有定义中发现的警告、排序后的检测结果，以及可以导入的其他工具的设置。 |
| `gitpoulpi flow templates` | 上述预设及其完整定义。不需要仓库。 |
| `gitpoulpi flow detect` | 排序后的模板，以及找到的基础分支和主题类型。 |
| `gitpoulpi flow config [--show-effective]` | 共享文件（其路径、是否存在、解析后的工作流、未知的键），或带 `--show-effective` 时为你的分支所用的工作流。`flow config show` 与之相同。 |
| `gitpoulpi flow config init --template <t> [--write]` | 输出某个模板的 TOML；`--write` 会创建 `.gitpoulpi/workflow.toml`，且拒绝覆盖已有的文件。 |
| `gitpoulpi flow import --from git-flow\|git-town [--write]` | 如上所述。 |

每次调用最多为 300 个本地分支计算领先/落后数量。

无法读取的工作流文件（TOML 有误、值的类型不对）是一个会指出该文件的错误；它绝不会被悄悄忽略。

## 操作

`start`、`finish`、`sync`、`promote` 和 `set-parent` 会作用于仓库。每一个都先**制定计划**：列出它将执行的每个 git 步骤及其对应的 git 命令、警告、无法撤销的内容，以及任何阻碍。你确认之前不会有任何更改。每次运行是**撤销历史中的一条记录**（`gitpoulpi undo` 会一并撤回分支、tag、被删除的分支、已记录的父分支和检出）。

| 命令 | 作用 |
|---|---|
| `gitpoulpi flow start <type> [name=value ...] [--from <ref>] [--publish]` | 按类型的模式为分支命名（`flow start feature "Login form"` 创建 `feature/login-form`；`flow start release version=1.2.0` 创建 `release/1.2.0`；单独的词就是 `slug`；`user` 和 `date` 默认为你和今天），从类型的父分支或 `--from`（分支、远程分支或 tag）创建它，检出并记录它的父分支。`--publish` 会用 `-u` push 它（`undo` 不会撤回）。 |
| `gitpoulpi flow finish [<branch>]` | 对该类型的每个 `finish` 目标依次：检出目标，按目标的策略 merge 该分支（`merge`：总是产生 merge commit；`squash`：一个 commit，标题为 `--title` 或取自分支名；`rebase`：把分支 rebase 到目标上，然后 fast-forward；`fast-forward`），目标带有 tag 时创建该 tag（`tag = "v{version}"`，附注 tag），然后检出父分支，并在类型要求时删除本地分支（`--delete` / `--keep` 可覆盖；`--delete-remote` 还会删除远程上的分支，无法撤销）。在 `finish_mode = "merge-request"`（或 `--mode merge-request`）下，它只会 push 分支并向父分支创建 pull 或 merge request，可带 `--title`、`--draft`、`--assignee`、`--reviewer`、`--label`、`--milestone` 和 `--integration`（无法撤销；本地不会合并或删除任何内容）。`--mode local-merge` 会覆盖 `merge-request` 类型，而设为 `ask` 的类型则需要在两者中择一。 |
| `gitpoulpi flow sync [<branch>] [--stack] [--rebase\|--merge]` | 把父分支的 commit 带入该分支：merge，或在该分支的策略为 `rebase`（或带 `--rebase`）时 rebase。带 `--stack` 时还会依次递归处理父分支为此分支的分支（用 `set-parent` 记录的）。结束后回到你原来所在的分支。 |
| `gitpoulpi flow promote <env-branch>` | GitLab Flow 的环境分支：把它所处下游的那个分支（`main` 到 `pre-production`，`pre-production` 到 `production`）merge 进来，一步完成，然后返回。能 fast-forward 时就 fast-forward。 |
| `gitpoulpi flow set-parent <branch> <parent>` / `--clear` | 写入（或移除）`branch.<name>.gitpoulpi-parent`。拒绝循环。可撤销。 |

从别处开始的分支就在别处完成：当已记录的父分支（来自 `--from` 或 `set-parent`）与类型的不同时，第一个完成目标就是已记录的父分支。

### 确认、计划、脚本化

| 选项 | 含义 |
|---|---|
| `--dry-run` | 以 JSON（`{"plan": ...}`）输出计划并停止。有阻碍时退出码为 1。 |
| `--yes` | 不询问直接运行。 |
| 都不带 | 在终端上：显示计划并询问你。否则（脚本、`--json`、代理）计划会以 `{"needs_confirmation": {"plan": ...}}` 返回，退出码为 3，且不更改任何内容。不带 `--publish` 或钩子的 `start`、`set-parent` 和 `--abort` 不需要确认。 |
| `--run-hooks` / `--no-hooks` | 见[钩子](#钩子)。 |
| `--autostash` | 把妨碍 checkout 或 merge 的本地更改 stash 起来并在之后放回，无论 `general.auto_stash` 如何设置。 |

被阻碍的计划会指明原因：需要主题分支的地方给了基础分支、名称已存在、缺少字段（`release/{version}` 需要 `version=...`）、tag 已存在、目标分支落后于其远程（`--allow-behind` 仍然继续）、已有 merge 或 rebase 在进行、工作流未配置。

在**自动 stash** 设置（`general.auto_stash`）下，妨碍 checkout 或 merge 的工作区更改会在操作前后被 stash 并恢复，与普通 checkout 完全一样；设为 `never` 时，操作会在改动任何内容之前停下。

### 冲突

因冲突而停下的 merge 或 rebase 会留下常规的冲突状态（冲突横幅、`gitpoulpi conflicts ...`），并且该流程会被记住。退出码 2，`status: "conflicted"`，路径在 `conflicts` 中。解决文件并标记为已解决，然后：

- `gitpoulpi flow finish --continue` 提交 merge（或继续 rebase，或带着其消息提交 squash）并运行其余步骤；
- `gitpoulpi flow finish --abort` 中止 merge 或 rebase，回到该流程开始时所在的分支，并忘记其余部分。已完成的步骤（双重完成的第一个目标及其 tag）会保留：`undo` 可以撤回它们。

因钩子失败而停下的流程用同样的方式恢复（`--continue` 会重新运行该步骤）。发生冲突的那次运行及其后续是两条撤销记录。

### Merge request

守护进程会 push 分支；创建请求需要你的托管平台令牌，所以由 CLI 用与 `gitpoulpi hosting pr` 相同的代码来做（与远程匹配的集成，否则是托管平台预填好的网页，以 URL 形式输出）。结果带有 `merge_request`（所请求的内容）和 `opened`（托管平台的答复），或者在分支已 push 但请求无法创建时带有 `open_error`。

### 钩子

工作流文件中的 `[hooks]` 列出 shell 命令（`after_start`、`before_finish`）。它们来自你的团队共享的文件，所以它们**除非你要求，否则绝不运行**：用 `--run-hooks`，或对终端在列出它们之后询问的问题回答是。计划总会列出它们，不会运行时为 `hooks_run: false`。钩子在 shell 中运行，以仓库为工作目录，并设置了 `GITPOULPI_FLOW_ACTION` / `GITPOULPI_FLOW_BRANCH`；它的确切命令、退出码和完整输出都在结果里。退出码非零的钩子会让流程停在那里（`before_finish` 钩子在任何更改发生前运行，所以是一个普通错误；`after_start` 钩子之后分支已经存在，流程可以继续或中止）。`--no-hooks` 不经询问直接跳过它们。钩子可以做任何事，所以 `undo` 只会撤回 git 的更改。

**信任。** 已提交的 `[hooks]` 部分里的命令来自任何能 push 到该仓库的人，所以它们还需要*你*的信任，每个版本的列表信任一次。信任按仓库保存，依据是钩子的哈希（任何命令、阶段或顺序的变化都会再次询问），并由守护进程在一次运行的第一步之前、以及在继续某个中断的流程之前检查，所以被拒绝的运行不会改动任何内容。你自己的覆盖设置里的钩子（你的仓库状态中的 `[workflow]`、设置页面）归你所有，从不询问。

- **CLI：** `gitpoulpi flow trust-hooks` 会打印这些命令、它们的 `hash` 以及是否 `trusted`；`gitpoulpi flow trust-hooks --hash <hash>` 会信任它们，并且仅当磁盘上的文件仍是该哈希时才生效。对未受信任的钩子使用 `--run-hooks` 会打印拒绝信息（`{"refused": {"reason": "hooks_not_trusted", ...}}`，退出码 2），其中带有这些命令和那条命令行。被问到“Run them too?”的终端，也会被问是否同时信任它们。
- **TUI：** 预览会列出这些钩子，并说明它们尚未受信任。随后按下操作按钮会询问“此仓库的工作流会在你的电脑上运行这些命令。要信任它们吗？”，并在一个可滚动的窗格中显示确切的命令（鼠标滚轮、可拖动的滚动条、PgUp/PgDn/Home/End），有两个答案：**1 信任并运行**和 **2 不带钩子运行**。在你按下方向键或数字键之前，没有任何选项被高亮；Esc、取消或关闭叉号会不作答直接返回。如果该文件在你信任之后发生了变化，标题会显示“钩子自你信任后已发生变化”，旧的列表显示在新的列表下方。
- **代理：** 没有信任钩子的工具。对未受信任的钩子使用 `run_hooks: true` 会被拒绝并附上这些命令，需要由人来运行 `flow trust-hooks`。

### 面向代理（MCP）

工具为 `flow_start`、`flow_finish`、`flow_sync`、`flow_promote` 和 `flow_set_parent`（finish、sync 和 promote 被标注为破坏性，所以由 `allow_mutations` 把关）。它们是先出计划再执行：带 `dry_run: true` 调用以获得计划（普通结果），然后带 `yes: true` 再次调用以运行它。两者都不带时调用会被拒绝并附带计划，所以不会意外运行。钩子只有在明确给出 `run_hooks: true` 时才会运行；计划会列出它们（`hooks`、`hooks_run`），以便代理先把命令展示给用户。已提交的钩子必须由人来信任（`flow trust-hooks`，它没有对应的工具）：对未受信任的钩子使用 `run_hooks: true` 会返回 `refused` 并附上这些命令，不会运行任何内容。发生冲突后，用 `continue_: true`（对应的选项是 `--continue`）或 `abort: true`。

### 撤销

| 操作 | `undo` 会做什么 |
|---|---|
| start | 删除该分支，忘记其父分支，重新检出之前的分支。 |
| finish | 一步恢复它移动过的每个分支、tag、被删除的分支和检出。已 push 的分支、远程删除和已创建的 merge request 会保留（计划里会说明）。 |
| sync | 把每个被更新的分支放回去，对已检出的那个也包括其文件。 |
| promote | 把环境分支放回去。 |
| set-parent | 恢复先前记录的父分支。 |

## 由托管平台驱动的流程

适用于由议题开启工作的团队：议题根据模板写成，托管平台（GitLab、GitHub、Gitea、Forgejo）或机器人创建分支和 merge request。GitPoulpi 创建议题，等待托管平台创建的内容，然后接手：一个跟踪远程分支的本地分支，记录其父分支（merge request 的目标），并检出。它需要 ISSUES 分区已关联到某个跟踪器（`gitpoulpi hosting link issues`）；PULL REQUESTS 的关联指明分支和请求所在的位置（未关联时为 ISSUES 的项目）。

```toml
[issue_flow]
mode = "bot"                         # local | host-creates-branch | bot
issue_title = "[{type}] {title}"     # {type} {title} {user} {date}
issue_labels = ["workflow::{type}"]  # 机器人通常由标签触发
description_template = ".github/ISSUE_TEMPLATE/feature.md"   # 仓库中的一个文件
branch_from_issue = "{type}/{issue}-{slug}"   # 托管平台或机器人如何给分支命名
wait_for = "branch-or-mr"            # branch | mr | branch-or-mr
timeout_secs = 600                   # `wait` 轮询多久
merge_request = true                 # host-creates-branch：同时创建草稿请求
assignees = []
```

`branch_from_issue` 中的 `{type}` 是主题类型的 id；其余是常用的[名称模式](#名称模式)占位符（`{issue}`、`{slug}`、`{issue_key}`……）。当分支匹配该模式且 `{issue}` 等于 12 时，它就属于议题 12（`feature/12-login` 属于，`feature/120-x` 不属于）。

| 模式 | 议题创建之后 |
|---|---|
| `local` | 托管平台上什么都不做。带 `--pick-up` 时，用议题编号和标题对该类型执行一次 `flow start`（和任何 start 一样可撤销）。 |
| `host-creates-branch` | GitPoulpi 从类型的父分支在**托管平台上**创建分支（GitLab：分支 API；GitHub：`createLinkedBranch`，所以议题会在 Development 下显示它；Gitea/Forgejo：分支 API），并创建一个写有 `Closes #12` 的**草稿**请求。拒绝空请求的托管平台（GitHub：还没有 commit）会留下该分支，并以警告说明。 |
| `bot` | 什么都不做：由机器人（webhook 处理程序、CI 作业、GitHub App）创建分支和请求。`--wait` 会轮询直到满足 `wait_for`。 |

对每个提供方，`wait` 会查看这些内容：托管平台自己的链接（GitLab 的 `issues/:iid/related_merge_requests` 和 `closed_by`；GitHub 的关联分支，以及时间线中的 `connected` / `cross-referenced` pull request）、名称匹配 `branch_from_issue` 的分支，以及源分支名称匹配的未关闭请求。它以逐渐增大的间隔轮询（1、2、3、5、8 秒，之后每 10 秒），直到满足 `wait_for` 或 `timeout_secs` 用尽；对在给出答复后变慢或短暂无法访问的托管平台会重试。托管平台不提供的来源（Gitea 没有议题链接）会被跳过；所有来源都失败时，第一个错误就是答复。

| 命令 | 作用 |
|---|---|
| `gitpoulpi flow issue new --type <t> --title "..." [--body T \| --body-file F] [--label L] [--assignee A] [--milestone M] [--wait] [--pick-up] [--timeout S] [--integration ID] [--dry-run]` | 根据模板（`issue_title`、`issue_labels`、描述文件，`--body` 可取代它）创建议题，然后按模式继续。`--dry-run` 输出将要创建的内容（`{"plan": ...}`）而不创建任何东西。`--pick-up` 在 `bot` 模式下会先等待。退出码 4：等待用尽（议题已存在；`flow issue wait` 会继续等待）。 |
| `gitpoulpi flow issue wait <#12> [--type t] [--for branch\|mr\|branch-or-mr] [--timeout S] [--once] [--pick-up]` | 轮询托管平台为某个议题创建了什么，在终端上进度显示在 stderr；Ctrl+C 取消。`--once` 只查看一次。退出码 4：暂时没有结果。 |
| `gitpoulpi flow pick-up <#12 \| !5 \| branch> [--remote R] [--parent B] [--no-fetch] [--dry-run]` | fetch，创建跟踪 `origin/<branch>` 的本地分支，记录其父分支并检出。`#12` 会在托管平台上查找该议题的分支或请求，`!5` 是一个请求，其他任何内容都是分支名。父分支是请求的目标（`--parent` 可覆盖），否则是该类型的父分支。不需要 `--yes`：它是本地的且可撤销，和 `start` 一样。`--dry-run` 仍然会 fetch（它只移动远程跟踪引用，而计划需要看到该分支）。 |

`pick-up` 是撤销历史中的一条记录：`gitpoulpi undo` 会删除该分支，忘记父分支，并重新检出之前的分支（fetch 会保留，和每次 fetch 一样）。它像其他操作一样先制定计划：`--dry-run` 输出它的计划（`create_branch`、`set_upstream`、`checkout`、`record_parent`），常见的阻碍同样适用（工作区更改遵循自动 stash 设置）。请求是 `{"action": "pick_up", "branch": ..., "parent": ...}`。

由流程创建的议题会被**监视**（`.gitpoulpi/repo-state.toml` 中的 `watched_issues`，绝不 commit）：每次 fetch 之后，应用会把远程分支与 `branch_from_issue` 对照，并在被监视议题的分支出现时告诉你。接手即可了结它。

### 面向代理（由托管平台驱动）

工具为 `flow_issue_new`、`flow_issue_wait` 和 `flow_pick_up`（write、network；由 `allow_mutations` 把关）。带 `dry_run: true` 的 `flow_issue_new` 会返回计划；`flow_issue_wait` 最多阻塞 `timeout` 秒，所以给代理一个较短的值并再次调用（`once: true` 会立即答复）。两者都返回带有 `found`（`branches`、`requests`）和 `timed_out` 的 JSON。

### 应用中由托管平台驱动的流程

有工作流并关联了跟踪器时，Flow 菜单会多出*新建 <类型> 议题…*（每个类型一行；面板为 `Flow：新建 <类型> 议题…`，以及 `Flow：新建议题…`）和*接手托管平台创建的分支…*（面板为 `Flow：接手托管平台分支…`）。

- **新建议题**会询问类型（下拉列表）和标题；它显示描述来自哪里以及由谁来创建分支。*分支出现后接手*默认开启。*创建议题*会创建它，然后在 `bot` 模式下显示**等待卡片**：一个旋转指示符、它在等待什么、已用时间和放弃的时间，以及**停止等待**（`Esc`）。停止等待不会丢失议题：它仍被监视。分支出现时它会被接手，状态栏会说明。
- **分支已出现**是一张卡片，在 fetch 带来某个被监视议题的分支后打开（没有其他对话框打开时）：*接手*会接管它，*稍后*会关闭它（在应用重启之前不会再次提供）。
- **接手**接受 `#12`、`!5` 或分支名，并在接手之前显示它找到了什么。

一切都在后台运行；超过 150 毫秒的调用会显示旋转指示符。

### 在试验台上试用

`scripts/fake-gitlab-bot.sh` 是 Docker GitLab（`scripts/test-bench.sh`）的一个替身机器人：它监视项目中带某个标签的议题，创建 `<type>/<iid>-<slug>`（带一个 commit）和一个会关闭该议题的草稿请求。`cargo run -p xtask -- mock-host --flow-bare <bare repo>` 以离线方式提供同样的东西（在真实裸仓库之上的 GitLab 形式的 API，连同机器人）。

## 在应用中

![带有 Git Flow 工作流的侧栏：main 和 develop 标为基础分支，feature 分支在 Feature 文件夹中，其余在 Other 下](../images/workflow-sidebar.png)

![开始 Feature：类型、slug、起点以及它将创建的分支名](../images/workflow-start.png)

![设置 > 仓库专属设置 > 工作流：模板、检测和基础分支编辑器](../images/workflow-settings.png)

下面的一切只出现在有工作流的仓库中（一个 `.gitpoulpi/workflow.toml`，或你自己的覆盖项）。没有工作流时什么都不会变：侧栏就是普通的树，也没有 `Flow` 按钮。

### 设置 > 工作流

*设置 > 仓库专属设置 > 工作流*用来编辑定义。它操作的是草稿，在你点击保存按钮之前不会保存任何内容。

- **模板**是预设的下拉列表；选择一个会用该预设替换下面的基础分支和主题类型（你的钩子和议题流程保持不变）。
- **根据分支检测**会读取此仓库的分支名，显示最接近的模板及其得分、找到的基础分支名（`main` 对应的 `master`）、每个主题类型识别了多少个分支（附示例），以及每个模板的得分。**使用检测到的工作流**会把它载入草稿。
- 当仓库中有其他工具的配置（Git Flow 的 `gitflow.*`、Git Town 的 `git-town.*`）时，会提供**导入**，并把导入的定义载入草稿。在你点击它之前不会应用任何内容。
- **基础分支**、**主题类型**和它们的**完成规则**组成**规则**标签页上一个分组的[列表](settings.md#列表)；**常规**标签页包含其余内容（模板、检测、钩子、议题流程）。选中一项，即可在右侧编辑它（名称和其他名称、父分支、受保护、merge 策略、merge 时的 tag、环境分支；显示名称、名称模式、其他模式、正则表达式、父分支、完成模式、完成后删除）。**+ 添加 ▾** 选择要添加什么（完成规则属于所选的主题类型）；添加、创建副本、删除和拖动与其他设置列表一样，各项只能在自己的分组内移动。**名称模式**行会列出占位符（`{slug} {issue} {issue_key} {user} {date} {version} {n}`）；其下有实时预览，显示该模式匹配的现有分支以及它将创建的名称，无法解析的模式会用红色在输入框正下方说明原因（已输入的内容不会丢失）。
- **钩子**是 shell 命令，每行一条（`开始之后`、`完成之前`），在多行编辑器中编辑（`Ctrl+S` 保存，`Esc` 取消）。它们会在你的机器上运行命令：页面会如此说明，对话框也会在它们运行前显示它们。
- **议题流程**包含[由托管平台驱动的流程](#由托管平台驱动的流程)所用的字段（模式、议题标题和标签、分支模式、等待什么），存放在工作流文件中。
- **问题**列出定义中的错误（不存在的父分支、损坏的模式）。
- 页面底部固定的一栏会显示 **● 未保存**（或 **无更改**）。**保存到共享文件**会把草稿写入 `.gitpoulpi/workflow.toml`（请 commit 它），保留此版本不认识的键，并把你自己的覆盖项并入其中。**仅为我保存**只把与共享文件的差异保存在你的仓库状态（`.gitpoulpi/repo-state.toml`，不 commit）中。**放弃更改**会重新读取已保存的内容。草稿未保存时关闭设置，会先询问。

### 侧栏

LOCAL 和 REMOTE 按工作流分组：基础分支在最前，各带一个 `[基础]` 标记（受保护时为 `[基础，受保护]`），然后是每个主题类型一个文件夹，以其显示名称命名并带有分支数量（`Feature (3)`），最后是没有类型认领的分支，放在 `其他` 下。文件夹像任何文件夹一样折叠；用*按类型分组显示分支*（Flow 菜单中的行）或面板命令 `Flow：切换分支分组` 可以回到普通的树。把指针停在分支上会显示它的类型、父分支以及落后（和领先）父分支多少。

### Flow 菜单及其对话框

工具栏的 **Flow** 按钮（`Ctrl+G Shift+W`，面板 `Flow：打开菜单`）会列出每个主题类型的*开始 <类型>…*、*完成当前分支…*、*与父分支同步…*、*提升…*（GitLab Flow 的环境分支）、*设置父分支…*、分组开关和*工作流设置…*。面板有同样的命令（`Flow：开始 Feature…` 会为每个类型生成）。

- **开始**是根据类型的名称模式生成的表单：每个占位符一个字段。`{issue}` 和 `{issue_key}` 带有所关联跟踪器的议题下拉列表，它还会用议题的标题填充 `{slug}`；`{version}` 会建议你最高 release tag 之后的版本；`{user}` 是你当前的配置文件；`{date}` 是今天。一行文字会显示该模式生成的分支名（或为什么不能：无效的引用、名称已被占用）。起点是一个下拉列表（类型的父分支在最前），*发布分支*会 push 新分支。
- **完成**、**同步**、**提升**和**设置父分支**会询问它们所需的内容，然后显示守护进程算出的**预览**（与 `gitpoulpi flow ... --dry-run` 输出的是同一个计划）：按顺序列出每一步、警告（目标落后于其远程、堆叠在此分支上的分支），以及**阻碍**（不存在的目标、落后于其远程的目标），阻碍会让运行按钮失效并说明原因。无法撤回的步骤（push、远程分支删除、merge request）会标以 `!` 并着色，另有一行“撤销不会收回以下内容：……”。会运行的钩子会列在一条警告之下，并带有*运行钩子*复选框（默认关闭）。当类型的完成模式为 `ask` 时，完成会先询问*完成方式*：在本地 merge，或创建 pull 或 merge request。
- 随后操作开始运行，每一步有一个标记（150 毫秒后出现旋转指示符），对话框会显示每一步的进展。整个操作是**一条撤销记录**：`Ctrl+Z` 会一次撤回 start、finish、sync 或 promote（撤销历史显示为 `Flow finish feature/login`）；push、远程删除和 merge request 会保留，正如预览所说。
- **冲突**会让操作停下，并出现**继续**和**中止**。可以关闭对话框，用常规的冲突工具解决冲突，然后用面板命令 `Flow：继续或中止已停止的操作…`（操作停止期间也是 Flow 菜单中的一行）重新打开；仍有文件处于冲突时继续会被拒绝，并保留两个按钮。中止会回到该操作开始时所在的分支；已完成的步骤会保留（undo 可以撤回）。
- 通过 **merge request** 完成时，会 push 分支，然后打开常规的*创建 Pull Request*表单（GitLab 上为*创建 Merge Request*）并预填：目标是该分支的父分支，标题取自分支（和议题）。元数据按钮（指派人、评审者、标签、里程碑）和任何 pull request 中一样工作。当没有集成能对应该远程时，会改为在浏览器中打开比较 URL（并显示在状态栏上）。
- 步骤和阻碍的文字是英文的，与 CLI 一样；对话框自己的文字是翻译过的。有更改的工作区遵循 `auto-stash` 设置。

所有这些都可以通过键盘和面板访问，每个下拉列表都是组合框（`Enter` 打开，输入筛选，`Enter` 选取）。

由托管平台驱动的行（*新建 <类型> 议题…*、*接手托管平台创建的分支…*）见[上文](#应用中由托管平台驱动的流程)。
