[English](../conversations.md) · [Čeština](../cs/conversations.md) · [Deutsch](../de/conversations.md) · [Español](../es/conversations.md) · [Français](../fr/conversations.md) · [Italiano](../it/conversations.md) · [日本語](../ja/conversations.md) · [한국어](../ko/conversations.md) · [Nederlands](../nl/conversations.md) · [Polski](../pl/conversations.md) · [Português (Brasil)](../pt-BR/conversations.md) · [Português (Portugal)](../pt-PT/conversations.md) · [Русский](../ru/conversations.md) · [Svenska](../sv/conversations.md) · [Türkçe](../tr/conversations.md) · [Українська](../uk/conversations.md) · **简体中文**

# 对话

议题、工作项、卡片或 pull / merge request 的对话视图。它需要账号和已关联的仓库：参见[托管平台与议题跟踪器集成](hosting-integration.md)；打开它的列表见 [Pull request 和议题](pull-requests-and-issues.md)；评审代码见[评审 pull request](review.md)。

**对话**是围绕某个议题、工作项、卡片、变更或 pull / merge request 说过的一切：描述、评论、评审、代码上的评论串，以及事件（标签、指派、状态更改、推送）。GitPoulpi 从托管平台读取它，格式化显示，并让你不离开窗口就能回答。

### 打开对话

| 来自 | 方式 |
|---|---|
| 议题行 | `Enter` 或双击；行菜单的**打开对话**；命令面板 *托管：打开议题对话…*（屏幕上的议题列表）。行菜单的**在浏览器中打开**仍然打开浏览器。 |
| Azure DevOps 或 Bitbucket 上的 PULL REQUESTS 行 | `Enter` 或**打开对话**。在 GitHub、GitLab、Gitea、Forgejo 和 Gerrit 上，该行打开[评审](review.md)，其“概览”页就是同一个对话（时间线、按键和下面的评论框）。 |
| [议题看板](pull-requests-and-issues.md#议题看板)的卡片 | `Enter` 或双击。 |
| [Radar](radar.md) | 议题或请求行上按 `i`、行菜单的**打开对话**，或 *Radar：打开议题对话*。 |
| 另一个对话中的引用 | 点击它（或按 `l`，再选择它）：已关联来源的 `#7`、`!12`、`ABC-12` 会在应用内打开，并压入“返回”栈。 |
| 代理 | `gitpoulpi ui show issue <n> --view conversation`（`ui show pr` 同样），见 [CLI 参考](cli-reference.md)。 |

**未克隆的仓库。**你没有克隆的仓库在 Radar 中的行也能打开其对话：这个视图只需要集成配置和项目，所以 Radar 把它托管在自己的窗口之上，按 `Esc` 会回到 Radar。需要克隆的部分会被省略：提交引用改为在托管平台上打开，而不是在 Log 中打开，“允许 AI 读取仓库”会变灰并给出原因（见 [AI 一章](ai.md#托管平台上的写作辅助)）。已克隆仓库的请求则有它的评审（`e`），Radar 的**打开对话**会说明这里没有可读的对话；已克隆仓库的议题会在该仓库的标签页中打开它。

托管平台没有可读对话的议题（Bitbucket 和 Gerrit 没有议题）仍像以前一样打开浏览器；不带 `conversation` 功能的构建在所有地方都是如此（[自定义构建](custom-builds.md)）。

### 视图

全窗口视图：标题区（状态、谁在何时打开的、标签、评论数、**显示事件**、**返回**、**刷新**、**在浏览器中打开**）、**以…身份发布**行、时间线，以及其下的评论框。

![议题 #7 的对话：带 #9 链接的描述、两条评论、由“显示事件”显示的一个标签事件，以及空的“新评论”文本框，其中有“编写”“预览”和“分栏”](../images/issue-conversation.png)

![merge request !58 的“概览”页：描述、一个未解决和一个已解决的代码评论串（带文件和行链接）、评论、所选评论串上的“回复”和“引述”，以及评论框](../images/pr-conversation.png)

- 时间线从旧到新，从最新的窗口开始（最多 300 条）。顶部的**加载更早的**（`m`，命令面板 *对话：加载更早的条目*）会读取更早的窗口，托管平台给出剩余数量时会一并显示。
- 评论显示作者、日期、`已编辑` 标记，机器人有 `机器人` 标记，私有备注有 `内部` 标记，以及所选条目允许的按钮：**回复**、**引述**、**编辑**、**删除**和**链接**。代码上的评论串显示 `path:line`、`已解决` 或 `未解决`、`已过时`，以及其中的评论。评审显示它的结论。
- 事件每条一行淡色文字（*carol 添加了标签 lfs*）。**显示事件**（`v`）可隐藏它们；该选择按仓库保存。没有事件历史的托管平台（YouTrack、Azure DevOps 工作项、本地笔记）没有这个复选框。
- 文本按 Markdown 渲染：标题、嵌套列表和任务列表、引用块、GitHub 风格的提示块、高亮代码、表格、链接，图片显示为 `[image: alt]`。以托管平台自有标记格式到达的文本会先被转换（见[标记转换](#标记转换)）。`mermaid` 代码块有一个**打开图表**按钮，在可滚动的查看器中显示图表源码（`Enter` 复制它）；命令面板有 *对话：打开条目的图表*。这需要同时带 `conversation` 和 `mermaid` 的构建。
- 在评审的概览中，代码讨论的 `路径:行号` 是通向代码的链接（`G`，*对话：打开讨论的代码*；见[评审](review.md)）；单独打开的对话（某个 issue、某个 Radar 行）没有可跳转的代码页。
- 视图每 60 秒以及按 `F5`（**刷新**）时重新读取。

**按键。**每个按键都有对应的按钮、菜单行或命令面板行（*对话：…*），`Ctrl+N` 会给按钮标上编号。

| 按键 | 操作 |
|---|---|
| `Up`、`Down`、`Ctrl+J`、`Ctrl+K`、`j`、`k` | 上一条 / 下一条 |
| `PageUp`、`PageDown`、`Home`、`End`、滚轮、滚动条 | 滚动 |
| `Tab`、`Shift+Tab` | 时间线、评论框，然后是标题区的按钮 |
| `r` | 回复：进入所选条目的评论串；托管平台没有评论串时，带上它的引述文本 |
| `Q` | 在新评论中引述所选条目 |
| `c` | 新评论 |
| `e`、`d` | 编辑、删除你自己的评论（删除之前会先询问） |
| `l`、`Enter` | 该条目的链接：`Enter` 跟随唯一的一个，或打开它们的菜单 |
| `y`、`o` | 复制该条目的地址、在浏览器中打开它 |
| `n` | 下一条未读条目 |
| `v`、`m`、`F5` | 显示或隐藏事件、加载更早的、刷新 |
| `Alt+Left`、`Backspace` | 沿“返回”栈返回 |
| `q`、`Esc` | 关闭（文本框中有文本时会先询问） |

### 写作

**新评论**（`c`，或用 `Tab` 切到文本框）、**回复**（`r`）、**引述**（`Q`）和**编辑**（`e`）使用同一个 Markdown 文本框：**编写**、**预览**和**分栏**（`Alt+P` 在编写与预览之间切换，`Alt+L` 打开分栏，需要 100 列宽的窗口）。`Ctrl+Enter`（也可用 `Ctrl+S` 和 `Alt+Enter`）或**发送**会发布，**取消**会丢弃文本框，开启 AI 时 `Alt+E` 打开 **AI 写作**（[AI 一章](ai.md#托管平台上的写作辅助)）。预览由与时间线相同的渲染器排版，引用也相同。托管平台不接受 Markdown 时，页脚会写明文本以何种标记发送（*以 Jira wiki markup 发送*、*以 Textile 发送*）。

- 评论会立即加入时间线，并在托管平台回答后就位。如果托管平台拒绝，该条目会消失，文本框会带着你的文本和原因重新打开。
- 你尚未发送的文本按条目保存（配置文件夹中的 `remote-conversations.toml`），下次打开该文本框时仍在；发送或丢弃它之后会被移除。关闭有文本的文本框时会先询问。
- 在评论串（评审评论串、GitLab 讨论、Linear 评论、Bitbucket 或 Azure DevOps 评论串）上**回复**会发布到该评论串中。托管平台只有扁平列表时，**回复**会新建一条评论，并引述所选条目（`> **author**`），与**引述**相同。
- **内部备注**（复选框，`Alt+I`）：在 GitLab 和 Redmine 上，该评论只对项目成员可见。它只在有这类备注的托管平台上显示，并且只用于新评论。
- 发布是对外的操作：不记入日志，**无法从 GitPoulpi 撤销**。没有连接时，文本框保留，**发送**会变灰并给出原因，标题区会说明显示的是已保存的副本。

**谁可以编辑或删除。**只有你自己的评论带有**编辑**和**删除**。托管平台自己给出的判断优先（GitHub 的 `viewerCanUpdate` 和 `viewerCanDelete`，其他平台是 `can_edit`），其次是你的登录名与评论作者比较，不区分大小写；App 机器人的评论还必须来自机器人账户。在得知 GitHub App 机器人的登录名之前，只有此安装发布的评论才算（仓库保留它们的 id）：不会根据名称去猜。在评论串中，**编辑**和**删除**作用于其中最后一条属于你的评论。不能编辑或删除的托管平台（见[表格](#各托管平台不能做什么)）会把这些按钮变灰并给出原因。

### 以谁的身份发布

标题区下面的一行说明你的消息以谁的身份发布，每个文本框里也有同样的一行：

| 行 | 含义 |
|---|---|
| *以 ann 身份发布（个人令牌）* | 该配置的令牌：你自己的登录名（`GET /user`）。 |
| *以 ci-bot 身份发布（仓库令牌命令）* | 托管 API 使用仓库的[令牌命令](hosting-integration.md#8-以-github-app-机器人身份操作)。 |
| *以 my-app[bot] 身份发布（GitHub App 机器人）* | 安装令牌。名称来自**发布身份**，否则来自托管平台。 |
| *以 GitHub App 机器人身份发布（尚不知道其名称：请在设置中设置“发布身份”）* | 没有配置名称，也还没有得知。 |
| *以 ... 身份发布（项目机器人）* | GitLab 项目或群组访问令牌。 |
| *以 kai 身份写入（本地笔记）* | Markdown 笔记文件夹：该配置的 `local.me`，否则是已连接的用户。 |
| *尚不知道你将以谁的身份发布。请在设置中设置“发布身份”。* | 什么也没能得知。 |

**发布身份**是一个名称：对 GitHub App 来说是 `my-app[bot]`。可在**设置 > 集成 > 发布身份**中为所有仓库设置（`general.hosting_post_as`），或在**设置 > 仓库专属设置 > SSH 与凭据 > 托管平台**中为某个仓库设置，后者优先。留空时，GitPoulpi 会询问 GitHub（GraphQL `viewer { login }`，安装令牌可能回答也可能不回答），失败时则从它发布的第一条评论的作者得知名称，并随仓库保存。个人令牌从不需要它。见[以 GitHub App 机器人身份操作](hosting-integration.md#8-以-github-app-机器人身份操作)。

### 交叉引用

对话中的引用带下划线，可以点击。仓库已关联的议题和 PULL REQUESTS 来源会优先匹配，所以已关联 Jira 项目的 `ABC-12` 即使出现在 GitHub 评论中也会在应用内打开；托管平台自有语法额外识别的内容，在链接确认之后打开其页面。仓库已克隆且 id 至少有 7 个字符时，提交 id 在 Log 中打开，否则在托管平台上打开。代码片段、围栏代码块、Markdown 链接、电子邮件地址或 URL 之内的任何内容都不是引用，`C#1`、`UTF-8`、`SHA-256` 和 `ISO-8601` 也不是。

| 托管平台 | 可识别的引用 |
|---|---|
| GitHub | `#7`、`GH-7`、`owner/repo#7`、`owner/repo@sha`、`@login`、提交 id |
| GitLab | `#7`、`!7`、`group/project#7`、`group/project!7`、`project#7`（同级项目）、`@user`、作为标记显示的 `~label` 和 `%milestone`、提交 id |
| Gitea、Forgejo | `#7`、`!7`、`owner/repo#7`、`owner/repo@sha`、`@user`、提交 id |
| Azure DevOps | `#7`、`!7`、`AB#7`、`@user` |
| Bitbucket | `@user`、`{uuid}` 提及、提交 id |
| Gerrit | `Change-Id` 值（`I` 加 40 位十六进制数字）、提交 id |
| Redmine | `#7`、`r123` 修订、`commit:abc1234` |
| Jira、Linear、YouTrack | `ABC-12`，仅限已关联来源的项目或团队 |
| Trello | 卡片地址（`trello.com/c/...`） |
| 本地笔记 | `[[note]]`、`[[note\|alias]]`，以及指向文件夹中另一篇笔记的相对 `.md` 链接 |

`gitpoulpi hosting markdown --body T --id I` 会输出应用如何处理一段文本，包括其中的引用。

### 标记转换

不使用 Markdown 的托管平台会双向转换，因此你在所有地方读写的都是 Markdown。转换器不理解的部分保持为可读文本。

- **Jira**（Cloud 和 Data Center）通过 API v2 以 wiki markup 存储评论。你读到的内容会转换为 Markdown；你写的内容会转换为 wiki markup，页脚会显示 *以 Jira wiki markup 发送*。
- **Redmine** 的文本格式是 Textile 或 Markdown，API 不会说明是哪一种：该配置的**服务器的文本格式**行（设置 > 集成，`markup`，默认 Textile）会告诉 GitPoulpi。Textile 在读取时转换为 Markdown，写入时再转换回 Textile。
- **Azure DevOps Server** 以 HTML 返回工作项评论，也以 HTML 接受：它们显示为由 HTML 转换而来的 Markdown，并以 HTML 写入。Azure DevOps Services 和 pull request 评论串接受 Markdown。
- **Gerrit** 是纯文本：不按 Markdown 渲染，你写的内容原样发送。
- 每个导入器都会先移除控制序列和转义序列，所以评论无法移动终端。

### 已保存的副本、加载更早的、未读

- 你打开的每个对话都会被保存（`<state dir>/conversations/<integration>/<project>/<kind>-<n>.json`，仅用户可读，不含令牌，每个文件最大 2 MB，保留最近的 50 个文件）。条目打开时会立即绘制它，并在后台刷新。无法连接托管平台时，已保存的副本留在屏幕上，显示 *离线：显示 3 分钟前保存的副本（原因）*，**发送**变灰。
- **未读：**你最后看到的内容按对话记住（保留最近的 200 个）。他人发布的更新条目会得到强调条和 `新` 标记，按 `n` 跳到下一条。你第一次打开的对话没有新内容。

### 各托管平台不能做什么

下面每个“否”都是真实的限制：控件会被隐藏，或变灰并给出原因，CLI 在发送任何内容之前就回答指明该托管平台的错误。

| 托管平台 | 条目 | 在评论串中回复 | 编辑和删除自己的 | 解决评论串 | 事件 | 内部备注 | 写入格式 |
|---|---|---|---|---|---|---|---|
| GitHub | 议题、pull request | pull request | 两者 | pull request | 是 | 否 | GitHub Markdown |
| GitLab | 议题、merge request | 是（讨论） | 两者 | merge request | 是 | 是 | GitLab Markdown |
| Gitea | 议题、pull request | pull request | 两者 | pull request | 是 | 否 | Markdown |
| Forgejo | 议题、pull request | pull request | 两者 | 否（托管平台没有对应端点） | 是 | 否 | Markdown |
| Azure DevOps | 工作项、pull request | pull request | 两者 | 否（显示状态，不能更改） | 仅 pull request | 否 | Markdown（Server 工作项为 HTML） |
| Bitbucket（Cloud、Data Center） | 仅 pull request | 是 | 两者 | 否（显示，不能更改） | 是 | 否 | Markdown |
| Gerrit | 仅变更 | 是 | 否（都不能编辑或删除） | 是 | 是 | 否 | 纯文本 |
| Jira | 议题 | 否（扁平列表） | 两者 | 否 | 是（变更日志） | 否（需要项目角色） | wiki markup |
| Linear | 议题 | 是（回复某条评论） | 两者 | 否 | 是 | 否 | Markdown |
| YouTrack | 议题 | 否 | 两者 | 否 | 否 | 否 | Markdown |
| Redmine | 议题 | 否 | 仅编辑（Redmine 5 及更高版本），从不能删除 | 否 | 是（日志） | 是（私有备注） | Markdown 或 Textile |
| Trello | 卡片 | 否 | 两者 | 否 | 是（移动） | 否 | Markdown |
| 本地笔记 | 笔记 | 否 | 本地作者自己的 | 否 | 否 | 否 | Markdown |

另外：没有哪个托管平台允许你从对话中在某一行上开始新的评论串（请在评审中进行），反应表情仅在 GitHub 上显示。超过托管平台允许长度的文本会被拒绝（GitHub 65,536 个字符，GitLab 1,000,000，Azure DevOps 150,000，Jira 32,767，Trello 16,384）：CLI 会先检查，应用内则显示托管平台自己的消息并保留你的文本。本地笔记把评论保存在笔记文件本身末尾的 `## Comments` 一节中，每条是一个 `### author - time` 条目，因此 Obsidian 和 shell 也能看到它们。

在命令行中，同样的对话是 `gitpoulpi hosting issue comments`、`issue comment`、`comment-edit` 和 `comment-delete`，以及 `hosting review conversation`（见 [CLI 参考](cli-reference.md)）；对代理来说，它们是同名的 MCP 工具（见 [MCP 与代理](mcp-and-agents.md)）。
