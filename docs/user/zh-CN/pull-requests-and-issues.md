[English](../pull-requests-and-issues.md) · [Čeština](../cs/pull-requests-and-issues.md) · [Deutsch](../de/pull-requests-and-issues.md) · [Español](../es/pull-requests-and-issues.md) · [Français](../fr/pull-requests-and-issues.md) · [Italiano](../it/pull-requests-and-issues.md) · [日本語](../ja/pull-requests-and-issues.md) · [한국어](../ko/pull-requests-and-issues.md) · [Nederlands](../nl/pull-requests-and-issues.md) · [Polski](../pl/pull-requests-and-issues.md) · [Português (Brasil)](../pt-BR/pull-requests-and-issues.md) · [Português (Portugal)](../pt-PT/pull-requests-and-issues.md) · [Русский](../ru/pull-requests-and-issues.md) · [Svenska](../sv/pull-requests-and-issues.md) · [Türkçe](../tr/pull-requests-and-issues.md) · [Українська](../uk/pull-requests-and-issues.md) · **简体中文**

# Pull request 和议题

在侧栏、看板和工具栏中读取、创建和管理 pull / merge request 与议题。请先连接账号并关联仓库：参见[托管平台与议题跟踪器集成](hosting-integration.md)。关于某个议题或请求的一切（评论、评审、评论串）见[对话](conversations.md)；阅读和评审请求见[评审 pull request](review.md)。

## 侧栏中的 pull request 和议题

### PULL REQUESTS

该分区有一个带计数和 `+`（创建 pull request）的标题、平台和项目下拉列表、一个搜索框和一个状态下拉列表（*打开*、*已 merge / 已关闭*、*全部*）。然后是四个分组，每个分组带有计数：

- **我的 pull request**：你是作者。
- **指派给我**。
- **等待我评审**：你是被请求的评审者。
- **所有 pull request**：起初是折叠的。

搜索框会过滤每个分组（标题、编号、作者、分支）。每次更改状态下拉列表，都会重新询问托管平台。

一行以状态符号开头：`●` 打开（绿色）、`○` 草稿、`◆` 已 merge（紫色）、`✕` 已关闭（红色）。已检出分支所对应的请求以粗体显示。行的形式为 `#12  title`（GitHub、Bitbucket）或 `!53  title`（GitLab、Azure DevOps）。

### 议题

分区标题会点明平台：`GITHUB 议题`、`JIRA 议题`，依此类推。Azure DevOps 为 `... 工作项`，Trello 为 `... 卡片`。下拉列表下方是：一个搜索框、状态下拉列表（*打开*、*指派给我*、*已关闭*、*全部*）和各个议题。Jira 的行形如 `ABC-12  title`，当侧栏足够宽时，类型和状态会以暗色显示。

### 对行的操作

- `Enter` 或双击会打开该行：pull request 在[评审](review.md)中打开（GitHub、GitLab、Gitea、Forgejo、Gerrit），议题以及 Azure DevOps 或 Bitbucket 上的请求在窗口中以[对话](conversations.md)打开，其他情况在浏览器中打开。行菜单的**在浏览器中打开**始终打开浏览器。
- 右键点击或按 `.` 会打开行菜单：
  - **打开对话**（议题，以及没有评审视图的请求）、**在浏览器中打开**、**复制 URL**、**复制编号**。
  - 对于 pull request：**在 <provider> 上编辑**、**检出源分支**、**查看源分支的 commit**。对于来自 fork 的请求，检出显示为 **Checkout pull request（fork）**，而**查看**为灰色。
  - 对于议题：**从议题创建分支**。它会打开新建分支的提示，带有诸如 `12-fix-the-thing` 的名称（Jira 则为 `ABC-123-fix-the-thing`）。
  - 在 GitHub、GitLab、Gitea 和 Forgejo 上，pull request 行还有**评审…**（在该行上按 **Enter** 也行）：应用内评审，见[评审 pull request](review.md)。
  - 在 GitHub、GitLab、Gitea 和 Forgejo 上，两种行还有**编辑指派人…**、**编辑标签…**和**设置里程碑…**（pull request 上另有**编辑评审者…**）：见[指派人、评审者、标签和里程碑](#指派人评审者标签和里程碑)。
- **在 <provider> 上编辑**还有另外三种进入方式：悬停或选中的 pull request 行最右端的铅笔 `✎`、所选行上的 `e`，以及命令面板命令 *集成：在托管平台上编辑关联的 pull request*（针对已检出分支的请求）。
- 如果源分支尚未在仓库中，GitPoulpi 会这样说明。请先 fetch（`Ctrl+G f`）。

**检出 pull request。**GitPoulpi 会把请求 fetch 到一个跟踪它的本地分支 `pr/<n>`（GitLab 上为 `mr/<n>`），并切换到该分支。在 GitHub、GitLab、Gitea 和 Forgejo 上，这对 fork 同样有效（GitPoulpi 会 fetch 托管平台的特殊 head 引用）。Azure DevOps 和 Bitbucket 会 fetch 请求的源分支。在那里，来自 fork 的请求会被拒绝：请把该 fork 添加为远程，并自行检出它的分支。撤销会回到之前的分支。CLI：`gitpoulpi hosting checkout-pr <n>`。

**通过命令行（以及代理）评审 pull request。**在 GitHub（包括 Enterprise）、GitLab（包括 self-managed）、Gitea 和 Forgejo 上，无需离开 GitPoulpi 就能读取并评审请求：`hosting review show <n>`（文件、合并状态、检查）、`hosting review threads <n>`（评审讨论和对话）、`hosting review comment`、`hosting review reply`、`hosting review resolve`，以及 `hosting review submit <n> --event approve|request-changes|comment --comment path:line[:side]:text`。diff 来自你的仓库，而不是托管平台：`hosting review fetch <n>` 会把请求的 head fetch 到 `refs/gitpoulpi/pr/<n>`，把它的目标分支 fetch 到 `refs/gitpoulpi/pr/<n>-base`（两者都不是分支，也不检出任何内容）。行号是该 diff 的行号：`new` 是 head 处的行，`old` 是 merge base 处的行。GitLab 没有“请求更改”这种评审状态：在那里会被拒绝，且不发送任何内容。Gitea 和 Forgejo 的评论只能针对一行（没有范围）；一个讨论就是文件某一行上的评论。Gitea 可以解决讨论，Forgejo 11 不行（该平台没有对应的端点：调用会如实说明）。在没有回复端点的服务器上，回复会作为同一行上的单条评论评审发布。Gitea 拒绝批准或驳回你自己的请求。无需克隆：给 `review show`、`threads`、`comment`、`reply`、`resolve`、`submit` 和 `suggest` 加上 `--project owner/name`（GitLab：`group/project`），即可评审一个不是当前仓库的项目；
`hosting review diff <n> --project P` 从托管平台的 API 读取 diff（按文件列出 hunk），`hosting file <path> --project P [--ref R]` 读取某个修订下的文件
（GitHub、GitLab、Gitea、Forgejo 和 Gerrit；在 Gerrit 上，补丁集的 diff 按文件逐个读取，文件来自分支，提交 id 则来自该提交）。diff 中托管平台没有显示的文件（太大）会标为 `omitted`。
MCP 工具与这些命令相同（`hosting_review_show`、`hosting_review_submit`、`hosting_review_diff`、`hosting_file`……）。

### 指派人、评审者、标签和里程碑

在 GitHub（包括 Enterprise）、GitLab（包括 self-managed）、Gitea 和 Forgejo 上，你可以在不离开 GitPoulpi 的情况下更改 pull request 或议题的指派人、评审者、标签和里程碑。其他提供方不显示这些内容。

- **行菜单**：**编辑指派人…**、**编辑评审者…**（仅 pull request）、**编辑标签…**、**设置里程碑…**。
- **命令面板**：*托管：编辑指派人…*、*托管：编辑评审者…*、*托管：编辑标签…*、*托管：设置里程碑…* 都会先列出侧栏中的请求和议题供你选择。在 [Radar](radar.md) 中，这四条作用于所选的行。
- 选择器会立即打开，并从平台加载项目的人员或标签（以及该条目当前的取值）。约 150 毫秒后会显示一行旋转指示符。输入以过滤（模糊匹配），`Up` / `Down` 移动，`Space` 勾选或取消勾选，`Enter` 应用，`Esc` 取消，`Tab` 到达按钮。点击某一行会勾选它，滚轮滚动。人员显示首字母徽标，标签显示其颜色。
- 里程碑是一个下拉列表（用**无里程碑**来清除它）。
- 在**应用**之后，平台即被更新，列表和 Radar 会在下一次刷新后显示更改，刷新会立即开始。

创建表单（pull request、议题）有同样的选项，以一排按钮的形式出现：`指派人：无`、`评审者：无`、`标签：无`、`里程碑：无`。在 GitHub、GitLab、Gitea 和 Forgejo 上，“标签”按钮取代议题表单中手动输入标签的字段。如果平台接受了新的 pull request，却拒绝其中某些标签或人员，状态栏会说明。

CLI：`gitpoulpi hosting members|labels|milestones`、`hosting pr-edit <n>`、`hosting issue edit <n>`，以及 `hosting pr` 和 `hosting issue create` 上的 `--assignee`、`--reviewer`、`--label`、`--milestone`；见 [CLI 参考](cli-reference.md)。

### 分区中的按键

| 按键 | 动作 |
|---|---|
| `Enter` / 双击 | 托管平台提供对话时在 GitPoulpi 中打开对话（GitHub、GitLab、Gitea 或 Gerrit 上的 pull request 会打开评审），否则打开浏览器 |
| `o` | 在浏览器中打开该行 |
| `.` / 右键点击 | 行菜单 |
| `e` | 在托管平台上编辑所选的 pull request |
| `n` | 新建 pull request（PULL REQUESTS）或新建议题（议题） |
| `v` | 打开状态下拉列表 |
| `r` | 刷新 |
| 标题上的 `Enter` / `Space` | 折叠或展开该分区 |

### 重新连接与离线

当令牌缺失、被拒绝或缺少某个权限范围时，分区的第一行显示 `重新连接（令牌被拒绝）`。点击它或按 `Enter`，即可在设置 > 集成中打开该配置。

当无法连接托管平台时，最后一次的列表仍会保留在屏幕上，标题会显示其时效（`· 5m old`）。最后一次的响应也缓存在磁盘上（`.gitpoulpi/hosting-cache.json`），所以列表会在网络响应之前就显示出来。

## 议题看板

议题标题上的**看板**按钮（面板：**议题：打开看板**）会把所关联来源的议题以 Kanban 看板的形式显示在窗口之上。列取决于来源：

| 来源 | 列 | 移动卡片 |
|---|---|---|
| 本地 Markdown 笔记 | 各状态（配置的 `statuses`，然后是在用的其他状态） | 重写笔记的 `status` |
| GitHub、GitLab、Gitea、Forgejo | **未关闭**，然后是你选择的标签，然后是**已关闭** | 换掉状态标签；移到**已关闭**会关闭议题，再移到某一列会重新打开它 |
| Trello | 看板上打开的列表，从左到右 | 把卡片移到该列表 |
| Redmine | 议题状态 | 更改议题的状态（适用 Redmine 的工作流规则） |
| Jira（Cloud 和 Data Center） | 项目的各状态，待办在前、完成在后 | 运行通向该列的工作流转换（`GET` 然后 `POST .../transitions`）；没有这样的转换时，卡片会回到原处，状态栏会显示 `no transition from X to Y` |
| Azure DevOps（Services 和 Server） | 项目各工作项类型的 state，按 state 类别排列 | 设置 `System.State`；类型没有该列所对应 state 的工作项会回到原处，并给出原因（`a Bug has no state Done`） |

在 GitHub 或 GitLab 项目上，**列**（或 `c`，或**看板：设置列标签…**）会询问作为列的标签，按顺序填写，例如 `status::todo, status::doing, status::review`。它会建议项目中看起来像状态的标签。这个选择按仓库保存。没有标签时，看板就是“未关闭 / 已关闭”。移动卡片只会改动你列出的状态标签：卡片的其他标签保持不变。平台看板会列出打开的议题和最近关闭的 60 个议题。

Jira 的转换如果要求必填字段（例如解决结果），会被 Jira 拒绝，卡片回到原处并显示 Jira 自己的消息：那种转换请在网站上完成。

脚本和代理也能使用同一个看板：`gitpoulpi hosting board` 输出各列及其卡片，`gitpoulpi hosting board move <card> <column>` 移动一张卡片（见 [CLI 参考](cli-reference.md)）。

卡片显示编号、标题、以首字母显示的指派人、标签和里程碑。把它拖到另一列，或用 `Shift+Left` / `Shift+Right`，或用**看板：将卡片移到…**（`m`）。它立即移动；约 150 毫秒后，一个旋转指示符表示平台仍在处理，如果平台拒绝，卡片会回到原处，状态栏会说明原因。`Enter` 在 GitPoulpi 中打开该议题的[对话](conversations.md)，关闭它后会回到看板；`e` 编辑指派人、标签和里程碑；`n` 在所选列中新建议题；`a`、`l`、`M` 和 `/` 分别按指派人、标签、里程碑和文本过滤，`x` 清除过滤；`r` 重新读取平台。当侧栏的列表刷新时，看板会重新读取。Radar 没有看板。

## 创建 pull request

用 PULL REQUESTS 标题中的 `+`、分区中的 `n`，或命令面板命令 *Pull request：创建 pull request…* 打开表单。分支菜单中也有 **从 <branch> 发起 pull request…**。

表单包含：

- 每个有 pull request 的已连接配置各占一个标签页（在标签条上用 `Left` / `Right`）。
- **源仓库 / 分支**和**目标仓库 / 分支**组合框。较长的列表可搜索。你可以输入列表中未显示的分支或仓库。
- **标题**，以顶端 commit 的主题填充。
- **描述**，可多行（`Enter` 换行，`Tab` 离开该字段）。
- **指派人**、**评审者**、**标签**和**里程碑**按钮（GitHub、GitLab、Gitea 和 Forgejo），用来打开选择器。
- **作为草稿提交**。
- 开启 [AI](ai.md) 后：**生成标题和描述**。

表单会以已检出分支的配置、仓库和分支打开（`feature -> main`）。不同的源仓库会产生来自 fork 的请求。

**创建 pull request**（或 `Ctrl+S`）会在托管平台上创建请求。结果会在浏览器中打开，状态栏会点明它。

## 创建议题

当议题分区既有平台**又有**项目时，它的标题上有一个 `+`（*新建议题*）。另外：分区中的 `n`，或命令面板命令 *托管：新建议题…*。

表单包含**标题**（必填）、**描述**（多行）和**标签**（以逗号分隔，例如 `bug, ui`；在 GitHub、GitLab、Gitea 和 Forgejo 上，改为由“标签”“指派人”和“里程碑”按钮打开选择器）。Jira 和 Azure DevOps 另有一个**类型**下拉列表。Trello 另有一个**列表**下拉列表。Linear 和 YouTrack 另有一个**状态**下拉列表（议题起始所处的工作流状态，或 `State` 的值）。Redmine 另有一个**类型**下拉列表，列出项目的 tracker（Bug、Feature、Support……）；它没有标签，所以没有“标签”按钮，在命令行上给出标签会被拒绝。表单打开时会从托管平台填充这些下拉列表。

**创建议题**（或 `Ctrl+S`）会在托管平台上创建它。`Esc` 或**取消**关闭表单。如果托管平台拒绝（令牌被拒绝、缺少权限范围、未知项目、未知标签），表单会显示原因并保留你输入的内容。成功时，状态栏会点明新议题，并刷新议题分区。

CLI：`gitpoulpi hosting issue create --title ... [--body ... | --body-file ...] [--label ...] [--type ...] [--list ...]`。

## 在托管平台自己的网页上创建

“新建议题”表单和“创建 pull request”表单的右上角都有一个 **在 <provider> 上创建** 按钮。快捷键是 `Alt+W`。它会在浏览器中打开托管平台的网页表单，并填入你目前已输入的内容。对话框保持打开。它不需要令牌，也不进行 API 调用，所以即使发生 API 错误之后也能使用。

- GitHub 没有表示*草稿*的 URL 参数，所以草稿复选框不会被带过去。GitLab 会得到 `Draft: ` 标题前缀。
- GitLab 只按 id 接受标签，所以它们会作为 `/label ~"x"` 快捷操作添加到描述末尾。
- 过长的描述会被截断，使 URL 保持在 6000 个字符以内，并附上 `[...truncated: paste the rest]` 的说明。
- Bitbucket Data Center 没有网页表单地址，所以没有 **在 <provider> 上创建** 按钮。

CLI：`gitpoulpi hosting issue create ... --web` 和 `gitpoulpi hosting pr --web` 会打印相同的 URL。

## 工具栏标记

在工具栏的末端（默认在右侧），一个标记显示已检出分支的请求：`GH #12`（GitHub）、`GL !53`（GitLab）、`AZ !12`（Azure DevOps）或 `BB #12`（Bitbucket）。它的颜色表示状态：绿色为打开，灰色为草稿，紫色为已 merge，红色为已关闭。

- 悬停可查看标题、状态和 `source → target`。
- 点击它，或按 `Ctrl+G g`，即可在浏览器中打开该请求。
- 右键点击它可使用 **在浏览器中打开**、**复制 URL**、**刷新** 和 **切换配置**。

已 push 但没有请求的分支会显示暗淡的 `+ PR` 标记。点击它即可发起 pull request。当没有匹配的配置、分支未 push，或无法连接托管平台时，没有标记。窗口较窄时，标记会被隐藏，以便为工具栏按钮留出空间。

## 列表何时刷新

列表会在以下时机刷新：

- 打开仓库时；
- 已检出分支改变时；
- push 或 fetch 之后；
- 每隔 *pull request 和议题刷新间隔（分钟）*（`general.hosting_refresh_minutes`，默认 5，`0` 关闭定时器；设置 > 集成）；
- 关闭设置时；
- 按 `r`、运行 *集成：刷新 pull request 和议题*，或更改过滤器时。

请求绝不会阻塞窗口。

托管平台会限制你可以进行的调用次数。GitPoulpi 会把上一次响应的 `ETag` 发回去，所以未变化的列表开销很小（在 GitHub 上没有开销）。当托管平台报告剩余调用次数很少时，状态栏会这样说明。当次数用完时，GitPoulpi 会停止向该托管平台发出请求，直到限额重置，列表则保持原样。

## 术语

GitHub、Azure DevOps 和 Bitbucket 称为“pull request”。GitLab 称为“merge request”。*评审请求术语*（`ui.pr_terminology`，位于设置 > 集成）决定每个标签、菜单、命令面板命令和消息所用的词：

- **自动（按托管平台）**：跟随仓库已关联的平台（尚未关联任何平台时用“Pull Request”）。
- **Pull Request** 或 **Merge Request**：始终使用该词。

更改立即生效。CLI 和 MCP 始终说“pull/merge request”。本页说“pull request”。

## commit 消息和分支名称中的引用

当仓库的议题或 pull request 已关联时，Commit 详情中 commit 消息里的引用会带下划线。点击会打开关联的页面。键盘：命令面板命令 *集成：打开 Commit 引用（#议题 / !请求）…*（当消息有多个引用时是一个菜单）。没有关联任何内容的仓库没有链接。

什么算引用，取决于议题分区的平台：

| 议题关联到 | 在消息中 | 在分支名称中 |
|---|---|---|
| GitHub、GitLab、Gitea、Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` 和 `AB#123` | `123-fix-login` |
| Jira | `ABC-123`，仅当 `ABC` 是所关联项目的键时。`UTF-8`、`SHA-256` 或 `ISO-8601` 之类的文本绝不会成为链接。 | 名称中任何位置的 `ABC-123`（`feature/ABC-123-x`），大写 |
| Linear | `ENG-123`，仅当 `ENG` 是所关联团队的键（或配置团队列表中某个团队的键）时。`UTF-8` 绝不会成为链接。 | 从议题创建分支时使用 Linear 自己的 `branchName`；名称中任何位置的 `ENG-123` 都会被读回 |
| YouTrack | `ABC-123`，仅当 `ABC` 是所关联项目（或配置项目列表中某个项目）的简称时 | 名称中任何位置的 `ABC-123` |
| Redmine | `#123`，仅当 Redmine 是所关联的议题来源时（pull request 来源保留它自己的符号） | `123-fix-login` |
| Trello | `https://trello.com/c/<shortLink>` 形式的地址；仅当已缓存的卡片编号为 12 时才识别 `#12` | `12-fix-login` |

| PULL REQUESTS 关联到 | 在消息中 |
|---|---|
| GitLab、Azure DevOps | `!45` |
| GitHub、Bitbucket、Gitea、Forgejo | `#45` |

当两个列表都匹配 `#N` 时，议题优先。CLI：`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`。

**在对话中。**议题或请求的文本按其托管平台的语法读取，这比上面的表格更宽：`owner/repo#7`、`!7`、`GH-7`、`@login`、提交 id，以及[交叉引用](conversations.md#交叉引用)中列出的其他形式。仓库已关联的来源优先匹配，所以 GitHub 评论中的 Jira 键仍会在应用内打开它的议题。`gitpoulpi hosting markdown --body TEXT --id I` 会显示应用在文本中找到了什么。

### 插入议题引用

在 commit 撰写器中，摘要最右端的 `#▾` 按钮会打开已关联议题的可过滤列表。当前分支所指的议题排在最前，标有*（来自分支）*。`Enter` 或点击会在光标处插入它的引用（`ABC-123`、`#12`、`AB#12`）。键盘：撰写器中的 `Alt+R`。命令面板：*Commit：插入议题引用…*。没有关联的议题列表时，按钮为灰色，并提示*请先在侧栏中关联议题列表*。GitPoulpi 绝不会自行添加引用。
