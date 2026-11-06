[English](../hosting-integration.md) · [Čeština](../cs/hosting-integration.md) · [Deutsch](../de/hosting-integration.md) · [Español](../es/hosting-integration.md) · [Français](../fr/hosting-integration.md) · [Italiano](../it/hosting-integration.md) · [日本語](../ja/hosting-integration.md) · [한국어](../ko/hosting-integration.md) · [Nederlands](../nl/hosting-integration.md) · [Polski](../pl/hosting-integration.md) · [Português (Brasil)](../pt-BR/hosting-integration.md) · [Português (Portugal)](../pt-PT/hosting-integration.md) · [Русский](../ru/hosting-integration.md) · [Svenska](../sv/hosting-integration.md) · [Türkçe](../tr/hosting-integration.md) · [Українська](../uk/hosting-integration.md) · **简体中文**

# 托管平台与议题跟踪器集成

GitPoulpi 可以连接以下产品：

- **托管平台**（pull request、议题、从你的账号克隆、SSH 密钥）：GitHub、GitHub Enterprise、GitLab、GitLab Self-Managed、Azure DevOps、Azure DevOps Server、Bitbucket、Bitbucket Data Center、Gitea 和 Forgejo（包括 Codeberg）。**Gerrit Code Review** 是仅用于评审的托管平台：它的变更就是 pull request，它没有议题。
- **议题跟踪器**（仅议题）：Jira Cloud、Jira Data Center、Trello、Linear、YouTrack（Cloud 和 Server）和 Redmine。
- **本地 Markdown 笔记**（仅议题，无需令牌，无需网络）：一个存放 Markdown 笔记的文件夹，例如 Obsidian 库中的文件夹，参见[本地议题与 Obsidian](local-issues.md)。

每个集成都使用**个人令牌**。没有 GitPoulpi 账号，也没有 OAuth 登录。集成是可选的，并且**按仓库选择启用**：在你关联仓库之前，不会获取任何内容。你的令牌保存在凭据助手中，绝不会写入 `config.toml`。普通的 `git` 操作（fetch、pull、push）不需要集成：它们使用你的 `git` 已有的凭据。

## 1. 在托管平台上创建令牌

连接表单中有一个*在 <host> 上生成令牌*的链接。它会打开该托管平台的令牌页面，并已选好正确的权限范围。若要手动操作：

| 托管平台 | 位置 | 权限范围 |
|---|---|---|
| GitHub、GitHub Enterprise | Settings > Developer settings > Personal access tokens (classic) > Generate new token | `repo`（仓库、pull request、议题）、`read:user`（你是谁）、`admin:public_key`（仅用于添加或删除 SSH 密钥） |
| GitLab、GitLab Self-Managed | User settings > Access tokens > Add new token | `api`、`read_user` |
| Azure DevOps | User settings > Personal access tokens，位于 `dev.azure.com/<organization>/_usersSettings/tokens` | `Code (Read & write)`、`Work Items (Read & write)`。请**为该组织**创建令牌，而不是为“All accessible organizations”：全局令牌将于 2026-12-01 失效。 |
| Azure DevOps Server | 你服务器上的同一页面（`https://<server>/<collection>/_usersSettings/tokens`） | 同样的两个权限范围 |
| Bitbucket | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token with scopes | `read:user:bitbucket`、`read:workspace:bitbucket`、`read:repository:bitbucket`、`write:repository:bitbucket`、`read:pullrequest:bitbucket`、`write:pullrequest:bitbucket`。对于 SSH 密钥，另需 `read:ssh-key:bitbucket`、`write:ssh-key:bitbucket`、`delete:ssh-key:bitbucket`。 |
| Bitbucket Data Center | 头像 > Manage account > HTTP access tokens | `Repository (Read & write)`、`Projects (Read)` |
| Gitea、Forgejo（Codeberg） | Settings > Applications > Generate New Token，位于 `<host>/user/settings/applications` | 读取 `user`；读写 `repository`、`issue` 和 `user`（最后一个用于 SSH 密钥）。Gitea 事后不显示权限范围，所以 GitPoulpi 无法对缺失的权限范围发出警告：被拒绝的调用会如实说明。 |
| Linear | Settings > Account > Security & access > Personal API keys > New API key（`linear.app/settings/account/security`） | 读取、写入、创建议题。该密钥以你的身份行事。 |
| YouTrack（Cloud 和 Server） | 你的个人资料 > Account Security > New token，位于 `<host>/users/me?tab=account-security` | `YouTrack` 权限范围。永久令牌以你的身份行事。 |
| Gerrit | Settings > HTTP Credentials > Generate new token（较新的服务器；较旧的：Generate new password），位于 `<host>/settings/#HTTPCredentials` | 无。令牌就是 **HTTP 密码**，以你的身份行事；用户名与它一起填写。 |
| Redmine | My account > API access key > Show，位于 `<host>/my/account`。管理员必须已开启 **Administration > Settings > API > Enable REST web service**。 | 无。该密钥以你的身份行事。 |
| Jira Cloud | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token（经典类型，不带权限范围） | 无。令牌以你的身份行事。 |
| Jira Data Center | Profile > Personal Access Tokens | 无。令牌以你的身份行事。 |
| Trello | 首先从 `trello.com/apps/admin` 获取 **API 密钥**（创建一个 Power-Up，然后生成 API 密钥）。然后从 `trello.com/1/authorize?...&key=<your key>` 获取**令牌**。 | `read`、`write` |

给令牌起一个名字，例如“GitPoulpi”，并设置有效期。复制令牌：托管平台只会显示一次。

## 2. 连接

1. 按 `Ctrl+,` 并选择 **集成**。
2. 在左侧选择产品。产品分为两组：**托管平台**和**议题跟踪器**。
3. 按 **连接**。若要为同一产品添加第二个账号，请使用 **+ 添加配置**。

   ![“集成”页面：左侧是各产品，GitHub 带有“连接”按钮](../images/settings-integrations.png)

4. 填写表单并粘贴令牌。输入时令牌以明文显示，所以请确保没有人在看。表单会列出所需的权限范围。

   ![连接表单：个人访问令牌字段、权限范围 repo、read:user 和 admin:public_key、在 github.com 上生成令牌的链接，以及“连接”和“取消”按钮](../images/settings-connect.png)

5. 按 **连接**。GitPoulpi 会在后台向托管平台校验令牌。随后页面会显示一张卡片，包含你的姓名和登录名、绿色的*已连接*，以及 SSH 密钥区域。缺少某个所需权限范围的令牌仍可连接，缺失的权限范围会以红色列出。

卡片上：**编辑**会再次打开表单。**断开连接**会删除令牌并保留配置。**移除**会在确认后同时删除两者。

“集成”页面上的键盘操作：`Tab` / `Shift+Tab` 在产品列表、配置条和表单之间移动。`a` 添加配置，`e` 编辑，`d` 或 `Delete` 移除，`o` 打开令牌页面，`Esc` 后退一步。

### 每个表单要求填写什么

只有在填写了主机或组织之后，*生成令牌*链接才会生效。

- **GitHub、GitLab：**只需令牌。**GitHub Enterprise** 和 **GitLab Self-Managed** 还要求填写**主机域名**（`gitlab.example.com`；输入 `http://` 才使用 http，否则默认使用 https）。不带点的名称（Docker 服务或局域网中的机器）需要加上端口或 `http://`：`gitea:3000`、`http://nas`。自托管服务器可以是纯 http，例如本地容器 `http://localhost:8080`。通过纯 http 发送的令牌不加密传输，因此只在本地或可信网络中使用 http。
- **Azure DevOps：**填写**组织**，即 `dev.azure.com/<organization>` 中的名称。也接受 `https://<org>.visualstudio.com` 形式的地址。两个组织需要两个配置。仓库使用其远程所指组织的配置。**Azure DevOps Server** 要求填写**集合 URL**，例如 `tfs.corp/tfs/DefaultCollection`。GitPoulpi 对 Azure DevOps 不使用系统凭据助手：Git Credential Manager 在那里保存的是登录令牌，而不是个人访问令牌。
- **Bitbucket：**你的 **Atlassian 账户邮箱**和一个 API 令牌。如果邮箱为空，令牌会作为 bearer 令牌发送：用于工作区或仓库访问令牌。GitPoulpi 会根据令牌找到你的工作区。**Bitbucket Data Center：**填写**服务器 URL**和一个 HTTP 访问令牌。
- **Gitea、Forgejo：**填写**主机域名**和令牌。**Forgejo** 默认从 `https://codeberg.org` 开始：自托管服务器请替换它。Gitea 没有公开的默认值。`http://localhost:3000` 这样的地址可用于本地服务器。两种类型共用一个客户端（Forgejo 是 Gitea 的分支，使用相同的 `/api/v1`）；令牌以 `Authorization: token <token>` 发送。它们不使用系统凭据助手。
- **Jira Cloud：**填写**站点 URL**（`acme.atlassian.net`）、你的 **Atlassian 账户邮箱**和一个经典 API 令牌。不支持带权限范围的 Jira 令牌。**Jira Data Center：**填写**服务器 URL**和一个个人访问令牌。
- **Trello：**填写 **API 密钥**和**令牌**。API 密钥是公开的，所以保存在 `config.toml` 中（作为 `account`）。令牌是机密，进入凭据助手。表单有两个链接：*在 Trello 上获取 API 密钥*，以及在你输入密钥之后的*生成令牌*。密钥和令牌通过请求头发送，绝不放在 URL 中。

- **Linear：**只需 **API 密钥**：Linear 只有一个公开主机，所以没有地址可填。密钥原样放在 `Authorization` 请求头中（Linear 的个人密钥不带 `Bearer`）。若要让 commit 消息中的 `ENG-12` 引用打开正确的页面，请把配置的 URL 与你的工作区一起保存（`https://linear.app/acme`，在 CLI 中用 `--url`）；议题行本身始终带有确切的地址。
- **YouTrack：**填写**站点 URL**（Cloud 用 `acme.youtrack.cloud`，否则是你自己的服务器；如果有上下文路径要带上，例如 `host/youtrack`）和一个永久令牌，以 `Authorization: Bearer` 发送。同一个客户端同时服务 Cloud 和 Server（`<url>/api`）。
- **Gerrit：**填写**服务器 URL**（如果有上下文路径要带上）、你的**用户名**，并把 HTTP 密码作为令牌。它们以 HTTP 基本认证传送；GitPoulpi 使用服务器的 `/a/` REST API，并忽略每个响应开头的 `)]}'` 行。用户名不是机密，保存在 `config.toml` 中（作为 `account`），密码进入凭据助手。仓库在 PULL REQUESTS 下关联到一个 Gerrit *项目*（它的变更列在那里）；Gerrit 没有议题，变更通过 push 创建（见[评审](review.md#gerrit)中的*推送以供评审*）。
- **Redmine：**填写你的 Redmine 的**站点 URL**（如果有子路径要带上，例如 `host/redmine`）和你的 API 密钥，以 `X-Redmine-API-Key` 发送。如果 Redmine 返回 403 或一个 HTML 页面，说明 REST 网络服务未开启（请联系管理员，或在 **Administration > Settings > API** 中开启）。

Jira、Trello、Linear、YouTrack 和 Redmine 只是议题跟踪器：没有 pull request，没有可克隆的仓库，也没有 SSH 密钥。Bitbucket 没有议题（Atlassian 已停用 Bitbucket Issues）。请改为把 Bitbucket 仓库的“议题”分区关联到某个 Jira 项目。

## 3. 令牌保存在哪里

令牌保存在你系统的**凭据助手**中（Windows 上是 Git Credential Manager，Linux 上是 libsecret 或 `pass`，macOS 上是钥匙串）。它使用一个私有条目，不会与真实主机的凭据混淆。它绝不会写入 `config.toml`，也绝不会被打印。

如果没有配置凭据助手，或者它拒绝了令牌，GitPoulpi 会**仅在本次会话中把令牌保存在内存里**，并告知你。重启之后需要重新连接。

GitPoulpi 按以下顺序查找配置的令牌：

1. 环境变量 `GITPOULPI_TOKEN_<PROFILE ID>`。用它来提供令牌而不存储它，例如在 CI 中。
2. 为该配置存储的令牌（`token_ref`）。
3. 仅限 GitHub 和 GitLab 配置：该主机的系统凭据助手，前提是没有其他已启用的配置使用同一主机。

每个配置是 `config.toml` 中的一个 `[[integrations]]` 条目。参见[配置参考](config-reference.md#integrations托管平台与议题跟踪器)。

## 4. 关联仓库

关联是显式的，并且按仓库进行。连接了配置之后，引用侧栏在 TAGS 之后会显示 **PULL REQUESTS** 分区和 **议题** 分区。每个分区有两个下拉列表：平台和项目。

- 平台下拉列表会**建议**与仓库远程匹配的账号（`origin` 在前），行形如 `owner/name · origin · GitHub`。一次选择同时设定平台和项目。
- 不会自动关联任何东西。Jira、Trello、Linear、YouTrack 和 Redmine 永远不会被建议，因为没有任何远程指向它们：请自己选择。
- 下拉列表顶部的 **无（取消关联此仓库）** 会取消该分区的关联。不会有任何东西自行再次关联它。
- 关联保存在仓库的 `.gitpoulpi/repo-state.toml` 文件中。

这两个分区是**分别**关联的。例如，pull request 可以来自 Azure DevOps，而议题来自 Jira。

| 平台 | PULL REQUESTS 项目 | 议题项目 |
|---|---|---|
| GitHub、GitLab | `owner/name`、`group/sub/proj` | 相同 |
| Azure DevOps | `Project/Repo`（组织来自配置） | `Project`（工作项） |
| Gitea、Forgejo | `owner/name` | 相同 |
| Bitbucket | `workspace/slug`（Data Center：`PROJ/slug`） | 无：关联 Jira |
| Jira | 无 | 项目键，例如 `ABC` |
| Trello | 无 | 一个看板 |
| Linear | 无 | 团队键，例如 `ENG`（会列出各团队） |
| YouTrack | 无 | 项目简称，例如 `ABC` |
| Gerrit | Gerrit 项目名，例如 `team/tool` | 无 |
| Redmine | 无 | 项目标识符，例如 `bench`（项目地址的最后一段） |

议题项目列表是可搜索的。你也可以输入列表中未显示的项目。

**额外查询。**议题来源在项目下拉列表下方有一个可选的过滤行。它与项目取 AND：

- Jira：一段 **JQL** 片段（`component = UI AND labels = backend`）。
- Azure DevOps：一个 **WIQL** 条件（`[System.AssignedTo] = @Me`）。
- YouTrack：一个 **YouTrack 查询**（`#Unresolved Type: Bug`、`tag: ui or tag: ux`）。它会被括号包起来，所以 `or` 不会扩大项目范围。你自己的 `sort by:` 会取代默认的 `sort by: updated desc`。
- Redmine：无（分区的状态和“指派给我”过滤器已经够用）。
- Linear：无（分区的状态和“指派给我”过滤器已经够用）。
- Trello：一个下拉列表，你在其中勾选要显示的**列表**。不勾选则显示所有打开的列表。
- GitHub 和 GitLab 忽略它。

`Enter` 保留所输入的查询，`Esc` 丢弃它。命令面板：*议题：编辑查询 / 列表…*。

在 CLI 中：`gitpoulpi hosting link pulls --id <profile>`（或 `issues`），`--clear` 取消关联，`--filter` 用于额外查询。参见 [CLI 参考](cli-reference.md#命令)。

### 从已连接的账户克隆

已连接的配置文件（GitHub、GitLab、Gitea、Forgejo、Azure DevOps、Bitbucket、Gerrit）也是**克隆**对话框中的一个来源（主页标签页的**克隆**按钮，或在其中按 `c`）。每个配置文件都作为独立的来源列在**通过 URL 克隆**下（`GitHub · name`、`GitLab · name`）；GitHub 和 GitLab 绝不会混在一起。选择一个后会加载它的仓库，并按所有者分组：先是你自己的账户，然后是每个组织或群组一个分区（GitLab 子群组如 `quarry/tools`），每个分区的标题处带有数量。锁图标表示私有仓库。

![已连接 GitHub 账户的克隆对话框：每个所有者一个分区，私有仓库带锁图标](../images/clone-hosted.png)

搜索框会在所有分区中筛选（名称、完整路径和描述）；没有匹配的分区会消失。在某一行上按 `Enter` 会用该仓库的 HTTPS URL 和文件夹名填好克隆。

![搜索把列表缩小到一个组织的克隆对话框](../images/clone-hosted-search.png)

你之前打开过的配置，其列表会立即从已保存的副本显示，同时重新向托管平台请求（标签旁显示*正在刷新…*）；答复到达后，新仓库会出现，已不存在的仓库会消失，高亮和已选中的仓库保持原位。如果刷新失败，已保存的列表会保留并带一条提示，原因见活动日志。

如果托管平台拒绝令牌，对话框会在列表的位置显示*无法列出仓库*及原因（并删除已保存的副本）；其他来源仍可使用。

## 5. 侧栏中的 pull request 和议题

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

- `Enter` 或双击会在浏览器中打开该行。
- 右键点击或按 `.` 会打开行菜单：
  - **在浏览器中打开**、**复制 URL**、**复制编号**。
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
| `Enter` / 双击 | 在浏览器中打开该行 |
| `.` / 右键点击 | 行菜单 |
| `e` | 在托管平台上编辑所选的 pull request |
| `n` | 新建 pull request（PULL REQUESTS）或新建议题（议题） |
| `v` | 打开状态下拉列表 |
| `r` | 刷新 |
| 标题上的 `Enter` / `Space` | 折叠或展开该分区 |

### 重新连接与离线

当令牌缺失、被拒绝或缺少某个权限范围时，分区的第一行显示 `重新连接（令牌被拒绝）`。点击它或按 `Enter`，即可在设置 > 集成中打开该配置。

当无法连接托管平台时，最后一次的列表仍会保留在屏幕上，标题会显示其时效（`· 5m old`）。最后一次的响应也缓存在磁盘上（`.gitpoulpi/hosting-cache.json`），所以列表会在网络响应之前就显示出来。

## 5b. 议题看板

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

卡片显示编号、标题、以首字母显示的指派人、标签和里程碑。把它拖到另一列，或用 `Shift+Left` / `Shift+Right`，或用**看板：将卡片移到…**（`m`）。它立即移动；约 150 毫秒后，一个旋转指示符表示平台仍在处理，如果平台拒绝，卡片会回到原处，状态栏会说明原因。`Enter` 在平台上打开该议题（本地议题则打开笔记）；`e` 编辑指派人、标签和里程碑；`n` 在所选列中新建议题；`a`、`l`、`M` 和 `/` 分别按指派人、标签、里程碑和文本过滤，`x` 清除过滤；`r` 重新读取平台。当侧栏的列表刷新时，看板会重新读取。Radar 没有看板。

## 6. 创建 pull request

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

## 7. 创建议题

当议题分区既有平台**又有**项目时，它的标题上有一个 `+`（*新建议题*）。另外：分区中的 `n`，或命令面板命令 *托管：新建议题…*。

表单包含**标题**（必填）、**描述**（多行）和**标签**（以逗号分隔，例如 `bug, ui`；在 GitHub、GitLab、Gitea 和 Forgejo 上，改为由“标签”“指派人”和“里程碑”按钮打开选择器）。Jira 和 Azure DevOps 另有一个**类型**下拉列表。Trello 另有一个**列表**下拉列表。Linear 和 YouTrack 另有一个**状态**下拉列表（议题起始所处的工作流状态，或 `State` 的值）。Redmine 另有一个**类型**下拉列表，列出项目的 tracker（Bug、Feature、Support……）；它没有标签，所以没有“标签”按钮，在命令行上给出标签会被拒绝。表单打开时会从托管平台填充这些下拉列表。

**创建议题**（或 `Ctrl+S`）会在托管平台上创建它。`Esc` 或**取消**关闭表单。如果托管平台拒绝（令牌被拒绝、缺少权限范围、未知项目、未知标签），表单会显示原因并保留你输入的内容。成功时，状态栏会点明新议题，并刷新议题分区。

CLI：`gitpoulpi hosting issue create --title ... [--body ... | --body-file ...] [--label ...] [--type ...] [--list ...]`。

## 8. 在托管平台自己的网页上创建

“新建议题”表单和“创建 pull request”表单的右上角都有一个 **在 <provider> 上创建** 按钮。快捷键是 `Alt+W`。它会在浏览器中打开托管平台的网页表单，并填入你目前已输入的内容。对话框保持打开。它不需要令牌，也不进行 API 调用，所以即使发生 API 错误之后也能使用。

- GitHub 没有表示*草稿*的 URL 参数，所以草稿复选框不会被带过去。GitLab 会得到 `Draft: ` 标题前缀。
- GitLab 只按 id 接受标签，所以它们会作为 `/label ~"x"` 快捷操作添加到描述末尾。
- 过长的描述会被截断，使 URL 保持在 6000 个字符以内，并附上 `[...truncated: paste the rest]` 的说明。
- Bitbucket Data Center 没有网页表单地址，所以没有 **在 <provider> 上创建** 按钮。

CLI：`gitpoulpi hosting issue create ... --web` 和 `gitpoulpi hosting pr --web` 会打印相同的 URL。

## 9. 工具栏标记

在工具栏的末端（默认在右侧），一个标记显示已检出分支的请求：`GH #12`（GitHub）、`GL !53`（GitLab）、`AZ !12`（Azure DevOps）或 `BB #12`（Bitbucket）。它的颜色表示状态：绿色为打开，灰色为草稿，紫色为已 merge，红色为已关闭。

- 悬停可查看标题、状态和 `source → target`。
- 点击它，或按 `Ctrl+G g`，即可在浏览器中打开该请求。
- 右键点击它可使用 **在浏览器中打开**、**复制 URL**、**刷新** 和 **切换配置**。

已 push 但没有请求的分支会显示暗淡的 `+ PR` 标记。点击它即可发起 pull request。当没有匹配的配置、分支未 push，或无法连接托管平台时，没有标记。窗口较窄时，标记会被隐藏，以便为工具栏按钮留出空间。

## 10. 列表何时刷新

列表会在以下时机刷新：

- 打开仓库时；
- 已检出分支改变时；
- push 或 fetch 之后；
- 每隔 *pull request 和议题刷新间隔（分钟）*（`general.hosting_refresh_minutes`，默认 5，`0` 关闭定时器；设置 > 集成）；
- 关闭设置时；
- 按 `r`、运行 *集成：刷新 pull request 和议题*，或更改过滤器时。

请求绝不会阻塞窗口。

托管平台会限制你可以进行的调用次数。GitPoulpi 会把上一次响应的 `ETag` 发回去，所以未变化的列表开销很小（在 GitHub 上没有开销）。当托管平台报告剩余调用次数很少时，状态栏会这样说明。当次数用完时，GitPoulpi 会停止向该托管平台发出请求，直到限额重置，列表则保持原样。

## 11. 术语

GitHub、Azure DevOps 和 Bitbucket 称为“pull request”。GitLab 称为“merge request”。*评审请求术语*（`ui.pr_terminology`，位于设置 > 集成）决定每个标签、菜单、命令面板命令和消息所用的词：

- **自动（按托管平台）**：跟随仓库已关联的平台（尚未关联任何平台时用“Pull Request”）。
- **Pull Request** 或 **Merge Request**：始终使用该词。

更改立即生效。CLI 和 MCP 始终说“pull/merge request”。本页说“pull request”。

## 12. commit 消息和分支名称中的引用

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

### 插入议题引用

在 commit 撰写器中，摘要最右端的 `#▾` 按钮会打开已关联议题的可过滤列表。当前分支所指的议题排在最前，标有*（来自分支）*。`Enter` 或点击会在光标处插入它的引用（`ABC-123`、`#12`、`AB#12`）。键盘：撰写器中的 `Alt+R`。命令面板：*Commit：插入议题引用…*。没有关联的议题列表时，按钮为灰色，并提示*请先在侧栏中关联议题列表*。GitPoulpi 绝不会自行添加引用。

## 13. SSH 密钥

### 每个配置的 SSH 密钥

已连接的配置在其卡片下方有一个 **SSH 密钥** 区域。没有密钥时，请选择其一：

- **默认密钥**：你的 ssh-agent 或 `~/.ssh`。不做任何更改。
- **生成新密钥**：一个可选的标题（默认为 `GitPoulpi <host> <date>`）。勾选 **将公钥添加到 <host>** 后，如果托管平台上还没有它，GitPoulpi 会上传。密钥以无口令形式写入 `~/.ssh/gitpoulpi_<profile id>_ed25519` 和 `.pub`。不需要 `ssh-keygen`。已有文件绝不会被覆盖。
- **自定义密钥**：输入路径或**浏览**一个私钥文件，然后按 **使用此密钥**。

附加了密钥之后，该区域会显示 `title (SHA256:...)`，并带有：

- **移除**：分离该密钥，如果是 GitPoulpi 添加的，还会在托管平台上删除它。
- **删除**：同上，并且还会删除密钥文件，删除前的确认会列出两个路径。

### 单个仓库的 SSH 密钥

要让某个仓库使用某个配置的密钥，GitPoulpi 会在该仓库中把 `core.sshCommand` 设置为 `ssh -i "<key>" -o IdentitiesOnly=yes`。这绝不会自动完成。打开标签页菜单（`Ctrl+T`，或右键点击标签页）并选择 **SSH 密钥…**，或者运行命令面板命令 *仓库：此仓库的 SSH 密钥…*。菜单会显示当前的固定、每个拥有私钥的配置，以及 **清除固定**。

CLI 和 MCP：`gitpoulpi ssh-command show | set --key <file> | clear`。

设置 > SSH 与凭据中的全局密钥（*私钥路径*，或*生成密钥对*）适用于每个没有固定的仓库。仓库的固定优先于它。

## 13b. 分支保护，用于历史重写

当你重写历史（[工作台](history-rewrite.md)）时，GitPoulpi 会向主机询问它保护了哪些分支，这些分支默认不参与重写。它只会针对你**已关联**的仓库（侧边栏的 pull request 或议题区域，或固定的配置）提出询问，使用该配置的令牌；令牌读不到时只会得到“未知”，不会有任何失败。各个平台的回答内容，以及令牌需要什么：

| 主机 | 读取什么 | 令牌需要 |
|---|---|---|
| GitHub（及 Enterprise Server） | 分支的 `protected` 标志，以及生效的分支规则集（`~DEFAULT_BRANCH`、`~ALL` 和 `refs/heads/...` 模式） | 带 `repo` 的经典令牌（公开仓库用 `public_repo`），或带 Metadata（只读）的细粒度令牌。令牌读不到的规则集会明确说明，经典规则仍然有效 |
| GitLab（gitlab.com 和自托管） | `protected_branches`（名称和 `*` 模式） | `read_api`，以及可以查看仓库设置的角色 |
| Azure DevOps（Services 和 Server） | 仓库中已启用且会阻止推送的分支策略（`Exact`、`Prefix` 和 `DefaultBranch` 范围） | Code（只读）。“force push”权限是安全设置，不会被读取 |
| Bitbucket Cloud | 会阻止推送的分支限制（`push`、`force`、`delete`） | `repository:admin`。针对分支模型的限制不会被解析 |
| Bitbucket Data Center | 会阻止 force push 的分支权限（`fast-forward-only`、`pull-request-only`、`read-only`） | 仓库管理员 |

保护设置页面（以及默认分支和删除仓库的页面）的地址，对 GitHub、GitLab、Azure DevOps 和 Bitbucket 都根据仓库的远程 URL 生成，即使仓库没有关联也一样，并会在主机拒绝 push 时连同三条出路一起显示。

## 13c. Radar 中的账户范围列表

Radar 通常监视你已克隆的仓库。配置还可以列出整个托管平台上与你有关的内容，无论是否已克隆，只需在其 `[integrations.radar]` 表中打开三个开关
（默认全部关闭；见[配置参考](config-reference.md#integrationsradar账户范围的列表)）：

- **评审请求**：请求你评审的未关闭请求；
- **未关闭的请求**：你自己的命名空间以及你所属的每个组织或群组（或你指定的命名空间）中所有未关闭的请求。绝不是整个托管平台；
- **指派的议题**：指派给你的未关闭议题。

GitHub、GitLab（包括 self-managed）、Gitea、Forgejo、Azure DevOps（Services 和 Server）、Bitbucket Data Center 和 Gerrit 会应答。各自的查询方式：

| 主机 | 请求 | 议题 |
|---|---|---|
| Azure DevOps | 整个组织的活动 pull request（等待你的用 `searchCriteria.reviewerId`；你已投过票的请求会被排除）；服务器拒绝组织级路径时，改为逐个查询前 20 个项目。`namespaces` 是项目名 | 指派给你且不是 Done、Closed 或 Removed 的工作项 |
| Bitbucket Data Center | 你的仪表板：评审请求用尚未批准的 `REVIEWER` 角色，未关闭的请求用 `AUTHOR` 和 `PARTICIPANT`；`namespaces` 是项目键 | 无（Data Center 没有议题跟踪器；会有一条说明） |
| Gerrit | `is:open reviewer:self -owner:self -is:ignored`；未关闭的请求按命名空间（项目路径前缀）用 `is:open projects:<prefix>`，未指定时为你自己的和你评审过的变更；进行中（work-in-progress）的变更算草稿 | 无 |

Bitbucket Cloud 没有账户范围的行：它没有跨仓库列出请求的端点，而逐个询问每个仓库调用次数太多。它已克隆的仓库保留各自的列表，
`gitpoulpi hosting radar --id <bitbucket>` 会这样说明。Gerrit 的变更多于其返回上限时，列表会标为已截断并附一条说明。

除非你选择保留，否则不含草稿。每个列表都有上限：每种类型 300 行，并附一条说明还有多少行；在 GitHub 上，每一轮只做少量搜索调用，因为其搜索 API 每分钟只允许 30 次。
令牌无法读取的命名空间会变成一条说明，其他的仍会加载；令牌被拒绝或触发速率限制只会使该配置失败。答复会复用五分钟（配置文件夹中的 `radar-account.json`）。

在设置 > 集成中，已连接的代码托管平台配置在其 SSH 密钥之下有一个 **Radar** 块：每个列表一个复选框（*等待我评审的请求*、*所有未关闭的请求*、*包含草稿*、
*指派给我的议题*，最后一项在没有议题的托管平台上不显示）、一个*命名空间*字段（逗号分隔；留空表示你自己的以及你所属的每一个），以及选项*未克隆仓库的请求*：
*询问*、*克隆它*或*通过 API 评审*。你每改动一处就会保存。没有账户范围查询的托管平台（Bitbucket Cloud）会改为显示一行说明。
命令面板用*设置：配置的 Radar 选项…*到达该块，设置搜索则按标签找到每个控件。

命令行：`gitpoulpi hosting integrations radar <id> --review-requests on` 打开一个列表，`gitpoulpi hosting radar --json` 输出这些列表（`--refresh` 跳过缓存）。
AI 代理通过 MCP 也有同样的两个命令（`hosting_radar`、`hosting_integrations_radar`）。

## 14. 通过命令行

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
```

每个命令参见 [CLI 参考](cli-reference.md#命令)。令牌绝不会被当作参数接受，而且 MCP 服务器不提供 `hosting login`，所以令牌绝不会经过代理。

## 故障排除

| 症状 | 解决办法 |
|---|---|
| *连接*因认证错误而失败 | 令牌错误、已过期或已被撤销。请创建新的令牌。 |
| 某个权限范围以红色列出 | 令牌没有它。连接可以工作，但该功能不行。请在托管平台上添加该权限范围，然后**编辑**并粘贴新令牌。 |
| 侧栏没有 PULL REQUESTS 分区 | 没有已启用并已连接的配置。请连接一个，然后在分区的下拉列表中关联仓库。 |
| 列表中有一行*重新连接* | 令牌丢失了（没有凭据助手时重启）或已过期。点击该行以打开配置。 |
| 自托管实例失败 | 请检查**主机域名**不含路径，并且你的计算机信任该实例的证书。 |

## 已知限制

- **来自 fork 的 pull request：**Azure DevOps 和 Bitbucket 无法检出来自 fork 的请求（参见[对行的操作](#对行的操作)）。Azure DevOps 无法从 fork 创建 pull request。
- **Trello** 最多显示看板的前 1000 张卡片。请使用列表过滤器缩小范围。**Jira**、**Linear** 和 **YouTrack** 每次刷新最多列出 500 个议题。请使用 JQL 片段或 YouTrack 查询缩小范围。
- **Linear** 每个议题只接受一位指派人：如果你在选择器中选了多个，只会发送第一个。它的“里程碑”是团队的 **cycle**（不使用项目里程碑）。**YouTrack** 的“里程碑”是 `Fix versions` 字段；版本没有编号，所以选择器按未发布版本在项目中的位置给它们编号。这两个跟踪器都没有 pull request。
- **Redmine** 每个议题只接受一位指派人（选择器中的第一个），没有标签，每次刷新最多列出 500 个议题。指派人是项目的成员（人员和群组），以显示名称区分。Kanban 看板的列就是议题状态，移动卡片会更改状态；适用 Redmine 默认的工作流规则（没有权限做该转换的角色会收到拒绝，卡片随即弹回）。在 Redmine 的默认数据中，“Resolved”不是已关闭状态：“已关闭”列是“Closed”和“Rejected”状态。
- **Bitbucket Data Center** 没有指向新 pull request、分支或 commit 的浏览器链接。pull request 页面可以正常打开。
- **Azure DevOps** 不显示作者头像。“我的”和“请求我评审”过滤器需要一个可能拒绝个人访问令牌的端点。在这种情况下，配置会在没有用户名的情况下连接，这两个过滤器不可用。无效的令牌会得到登录页面而不是错误；GitPoulpi 将其报告为令牌被拒绝。
- **Gitea 和 Forgejo：**草稿就是标题以 `WIP:` 开头的请求（服务器在创建时没有草稿标志）。请求会把所有已评审过它的人列为评审者，而撤回评审请求只会移除待处理的那些。来自 fork 本地分支的 pull request 通过 `refs/pull/<n>/head` 检出，与 GitHub 相同。提及通过服务器的 `mentioned_by` 过滤器搜索。服务器不显示令牌的权限范围，所以缺失的权限范围只有在调用被拒绝时才会发现。
- **议题看板**在每个议题来源上都能移动卡片。在 Jira 上，移动会运行通向该列的工作流转换；要求填写字段（例如解决结果）的转换会被 Jira 拒绝，卡片带着它的消息弹回原处。在 Azure DevOps 上，没有目标 state 的工作项类型会弹回原处并给出原因。
- **Bitbucket：**应用密码已不存在；请使用 API 令牌。仓库按工作区逐个列出。
- **Jira Cloud：**只有经典 API 令牌有效（不支持带权限范围的令牌，也不支持 OAuth）。新的 Jira 议题得到的是纯文本描述。
- 创建 pull request、工作项、议题或卡片发生在托管平台上，**无法**在 GitPoulpi 中撤销。检出请求则可以正常撤销。
