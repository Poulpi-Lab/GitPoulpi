[English](../hosting-integration.md) · [Čeština](../cs/hosting-integration.md) · [Deutsch](../de/hosting-integration.md) · [Español](../es/hosting-integration.md) · [Français](../fr/hosting-integration.md) · [Italiano](../it/hosting-integration.md) · [日本語](../ja/hosting-integration.md) · [한국어](../ko/hosting-integration.md) · [Nederlands](../nl/hosting-integration.md) · [Polski](../pl/hosting-integration.md) · [Português (Brasil)](../pt-BR/hosting-integration.md) · [Português (Portugal)](../pt-PT/hosting-integration.md) · [Русский](../ru/hosting-integration.md) · [Svenska](../sv/hosting-integration.md) · [Türkçe](../tr/hosting-integration.md) · [Українська](../uk/hosting-integration.md) · **简体中文**

# 托管平台与议题跟踪器集成

GitPoulpi 可以连接以下产品：

- **托管平台**（pull request、议题、从你的账号克隆、SSH 密钥）：GitHub、GitHub Enterprise、GitLab、GitLab Self-Managed、Azure DevOps、Azure DevOps Server、Bitbucket、Bitbucket Data Center、Gitea 和 Forgejo（包括 Codeberg）。**Gerrit Code Review** 是仅用于评审的托管平台：它的变更就是 pull request，它没有议题。
- **议题跟踪器**（仅议题）：Jira Cloud、Jira Data Center、Trello、Linear、YouTrack（Cloud 和 Server）和 Redmine。
- **本地 Markdown 笔记**（仅议题，无需令牌，无需网络）：一个存放 Markdown 笔记的文件夹，例如 Obsidian 库中的文件夹，参见[本地议题与 Obsidian](local-issues.md)。

每个集成都使用**个人令牌**。没有 GitPoulpi 账号，也没有 OAuth 登录。集成是可选的，并且**按仓库选择启用**：在你关联仓库之前，不会获取任何内容。你的令牌保存在凭据助手中，绝不会写入 `config.toml`。普通的 `git` 操作（fetch、pull、push）不需要集成：它们使用你的 `git` 已有的凭据。

**接下来去哪里。**本页是设置：令牌、连接、关联仓库、SSH 密钥。账号连接之后：

- 侧栏的 PULL REQUESTS 和 ISSUES 分区、议题看板、创建 pull request 或议题、工具栏标记，以及 commit 消息中的引用：**[Pull request 和议题](pull-requests-and-issues.md)**；
- 在应用内查看某个议题或请求，包括它的评论、写作和发布身份：**[对话](conversations.md)**；
- 用评论串和待提交评审来阅读和评审请求：**[评审 pull request](review.md)**；
- 整个工作区的 pull request 和议题合在一个列表中：**[Radar](radar.md)**。

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

## 安全检查

已保存的令牌，其安全程度取决于它被发往的地址。因此 GitPoulpi 会为每个集成记住它当初连接的是什么，并在这些内容发生变化时停下：

- **地址。** 某个集成第一次被使用（或被连接）时，它的地址会被记录在 `trust.toml` 中，该文件位于仅限该用户访问的私有数据文件夹（`<data>/private/`），而不是 `config.toml`：编辑配置永远无法扩大被信任的范围。如果配置中的地址不再是所记录的地址，就不会向它发送任何内容，甚至不会读取令牌。同一个地址的不同写法（`HTTPS://Git.Example.com:443/`）是同一个地址；外观相似的主机则不是。
- **账户。** 连接令牌之后，以及此后大约每天一次（对来自 `GITPOULPI_TOKEN_*` 或凭据助手的令牌，则是每次运行一次），GitPoulpi 会向托管平台询问该令牌属于谁，并比较账户 **id**，而不是登录名。现在属于别人的令牌会被拦下。
- **证书。** 证书不能链到已知机构的服务器（私有 CA、自签名服务器），只有对你接受过的那个确切密钥才会被接受，而且 TLS 握手仍会证明服务器持有该密钥。开启严格固定后，即使证书链到公共根证书，密钥也必须匹配；代价是每次轮换密钥都会再次询问，所以它是可选的。公共主机（github.com、gitlab.com、codeberg.org、bitbucket.org、dev.azure.com、`*.atlassian.net` 等）不保留证书记录。
- **明文 http。** 非回环的 `http://` 地址会被拒绝，除非你为该集成允许了它。

每个请求还会被限定在该集成自己的地址（对公共主机，则是它的 API 主机，例如 `api.github.com`）：发往其他任何地方的请求都会在发送之前被拒绝，重定向到另一个地址时也不会跟随。

**在此检查出现之前就已连接的集成**会在升级后第一次使用时，按当时的配置静默得到它们的记录：它们保持如今的信任级别，不会有任何询问。

### 在命令行中

被拦下的命令会以退出码 2 失败，并打印发生了什么变化（见 [CLI 参考](cli-reference.md)）。`gitpoulpi hosting integrations trust <id>` 会显示被信任的内容；`--accept-origin <url>`、`--accept-account <id>`、`--accept-certificate <sha256>`、`--strict-tls on|off` 和 `--allow-insecure-http on|off` 各接受一样东西，并写明确切的值。接受新地址之后，请再次输入令牌。断开连接（`hosting logout`）或移除某个集成，会删除它的令牌、它的信任记录以及从它缓存的内容（克隆对话框已保存的仓库列表、Radar 账户列表，以及其托管平台的远程评审草稿）。

代理（MCP）无法接受任何东西：它会被告知该安全检查需要你。

### 在终端界面中

被拦下的请求会在你正在做的事情之上打开**集成安全检查**对话框。它会说明发生了什么变化（旧地址和新地址、带 id 的旧账户和新账户，或者以一个小表格显示的旧证书和新证书，其中有按四位一组分隔的 SHA-256 指纹、主体、颁发者和有效期），并说明令牌未发送。答案带有编号：

1. **断开连接**会删除已保存的令牌、信任记录以及从该集成缓存的内容。
2. **信任**新地址、新账户或新证书（对未加密的地址：**允许明文 http**）。
3. **取消**不更改任何内容。

在你按下方向键或数字键之前，没有任何答案被高亮，所以误按 Enter 不会接受任何东西。Esc 和关闭叉号会取消；点击对话框外部没有任何作用。较长的正文可以用滚轮、Page Up/Page Down 或滚动条滚动。多项检查会排队（每个集成、每种变化各一项），依次询问。被取消的检查在没有变化时不会再次弹出；命令面板中的**集成：查看安全检查**会把它带回来。

在你信任新**地址**之后，该集成的表单会打开，焦点在令牌字段上，并提示“建议重新输入令牌。”：已保存的令牌从未被发往那里。在表单中，有意更改的地址会连同为它输入的令牌一起被接受；没有令牌时，表单会要求输入。

表单还有两个开关。**严格证书固定**（用于 https 地址）会在你连接时固定服务器出示的证书密钥。**允许明文 http（未加密）**只为不是本机的 `http://` 地址提供，并带红色警告说明令牌以明文传输。已连接的卡片会显示被固定的指纹。

### 令牌能做什么，以及到什么时候

连接时还会向托管平台询问该令牌能做什么。GitHub（经典令牌）、GitLab 和 Bitbucket（在它发送权限范围时）会报告它们的权限范围：卡片会显示一行黄色的“此令牌的权限超出 GitPoulpi 所需：”，列出 GitPoulpi 用不到的权限范围，其中敏感的（如 `delete_repo`、`admin:org`、`sudo`）以红色显示，并建议“建议使用带到期日期的细粒度令牌”。经典 GitHub 令牌（`ghp_...`）即使权限范围正好，也会得到这条建议。细粒度的 GitHub 令牌无法查询其权限，所以只显示它的到期时间。当托管平台告知到期日期时，卡片会显示它。没有到期时间的令牌不显示提醒，其他产品不报告权限范围，也不发出警告。同样的字段也在 `hosting integrations login` 和 `test` 的 JSON 中（`excess_scopes`、`token_flavor`、`recommend_fine_grained`、`expires_at`）。它们是在连接时读取的，所以卡片只为发起连接的那次会话显示它们。

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

## 5. SSH 密钥

### 每个配置的 SSH 密钥

已连接的配置在其卡片下方有一个 **SSH 密钥** 区域。没有密钥时，请选择其一：

- **默认密钥**：你的 ssh-agent 或 `~/.ssh`。不做任何更改。
- **生成新密钥**：一个可选的标题（默认为 `GitPoulpi <host> <date>`）。勾选 **将公钥添加到 <host>** 后，如果托管平台上还没有它，GitPoulpi 会上传。密钥以无口令形式写入 `~/.ssh/gitpoulpi_<profile id>_ed25519` 和 `.pub`。不需要 `ssh-keygen`。已有文件绝不会被覆盖。
- **自定义密钥**：输入路径或**浏览**一个私钥文件，然后按 **使用此密钥**。

附加了密钥之后，该区域会显示 `title (SHA256:...)`，并带有：

- **移除**：分离该密钥，如果是 GitPoulpi 添加的，还会在托管平台上删除它。
- **删除**：同上，并且还会删除密钥文件，删除前的确认会列出两个路径。

### 单个仓库的 SSH 密钥

要让某个仓库使用某个特定的密钥，请打开标签页菜单（`Ctrl+T`，或右键点击标签页）并选择 **SSH 与凭据…**，或者运行命令面板命令 *仓库：SSH 与凭据…*，然后在 SSH 下选择 **密钥文件**（或选 **账户密钥**，使用某个集成配置的密钥）。这个选择保存在 GitPoulpi 的私有文件夹中：不会向仓库或 git 的配置写入任何内容。参见[设置](settings.md#仓库专属ssh-与凭据)。

仍带有自己的 `core.sshCommand` 的仓库（来自以前的“此仓库的 SSH 密钥”固定）会在该页面中以只读方式显示它，并带有 **移入 GitPoulpi** 按钮。

CLI 和 MCP：`gitpoulpi repo-auth`（针对单个仓库的选择），而对于 `core.sshCommand` 固定本身，则是 `gitpoulpi ssh-command show | set --key <file> | clear`。

设置 > SSH 与凭据中的全局密钥（*私钥路径*，或*生成密钥对*）适用于每个没有自己选择的仓库。仓库自己的密钥文件或固定项优先于它。

## 6. 分支保护，用于历史重写

当你重写历史（[工作台](history-rewrite.md)）时，GitPoulpi 会向主机询问它保护了哪些分支，这些分支默认不参与重写。它只会针对你**已关联**的仓库（侧边栏的 pull request 或议题区域，或固定的配置）提出询问，使用该配置的令牌；令牌读不到时只会得到“未知”，不会有任何失败。各个平台的回答内容，以及令牌需要什么：

| 主机 | 读取什么 | 令牌需要 |
|---|---|---|
| GitHub（及 Enterprise Server） | 分支的 `protected` 标志，以及生效的分支规则集（`~DEFAULT_BRANCH`、`~ALL` 和 `refs/heads/...` 模式） | 带 `repo` 的经典令牌（公开仓库用 `public_repo`），或带 Metadata（只读）的细粒度令牌。令牌读不到的规则集会明确说明，经典规则仍然有效 |
| GitLab（gitlab.com 和自托管） | `protected_branches`（名称和 `*` 模式） | `read_api`，以及可以查看仓库设置的角色 |
| Azure DevOps（Services 和 Server） | 仓库中已启用且会阻止推送的分支策略（`Exact`、`Prefix` 和 `DefaultBranch` 范围） | Code（只读）。“force push”权限是安全设置，不会被读取 |
| Bitbucket Cloud | 会阻止推送的分支限制（`push`、`force`、`delete`） | `repository:admin`。针对分支模型的限制不会被解析 |
| Bitbucket Data Center | 会阻止 force push 的分支权限（`fast-forward-only`、`pull-request-only`、`read-only`） | 仓库管理员 |

保护设置页面（以及默认分支和删除仓库的页面）的地址，对 GitHub、GitLab、Azure DevOps 和 Bitbucket 都根据仓库的远程 URL 生成，即使仓库没有关联也一样，并会在主机拒绝 push 时连同三条出路一起显示。

## 7. Radar 中的账户范围列表

Radar 通常监视你已克隆的仓库。配置还可以列出整个托管平台上与你有关的内容，无论是否已克隆，只需在其 `[integrations.radar]` 表中打开三个开关
（默认全部关闭；见[配置参考](config-reference.md#integrationsradar账户范围的列表)）：

- **评审请求**：请求你评审的未关闭请求；
- **未关闭的请求**：你自己的命名空间以及你所属的每个组织或群组（或你指定的命名空间）中所有未关闭的请求。绝不是整个托管平台；
  配置的**命名空间**（在设置 > 集成 > Radar 中每行一个，也可以是正则表达式）会限制这三个列表：一行与仓库的完整路径相等，或是以 `/` 结尾的路径前缀时即匹配（`orbit` 保留 `orbit/products/x`，不保留 `orbitx/y`），它会应用到托管平台返回的行上，并在托管平台的搜索允许时也在服务器端应用（GitHub 的 `user:` / `repo:` 限定符）；
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
*指派给我的议题*，最后一项在没有议题的托管平台上不显示）、一个*命名空间*字段（每行一项：名称匹配该组或所有者及其下的全部内容；勾选*正则表达式*后每行是一个正则表达式；留空表示你自己的以及你所属的每一个），以及选项*未克隆仓库的请求*：
*询问*、*克隆它*或*通过 API 评审*。你每改动一处就会保存。没有账户范围查询的托管平台（Bitbucket Cloud）会改为显示一行说明。
命令面板用*设置：配置的 Radar 选项…*到达该块，设置搜索则按标签找到每个控件。

命令行：`gitpoulpi hosting integrations radar <id> --review-requests on` 打开一个列表，`gitpoulpi hosting radar --json` 输出这些列表（`--refresh` 跳过缓存）。
AI 代理通过 MCP 也有同样的两个命令（`hosting_radar`、`hosting_integrations_radar`）。

## 8. 以 GitHub App 机器人身份操作

不能显示某个人名字的项目，可以通过 **GitHub App** 在托管平台上完成所有操作：每条评论、评审、关闭或重新打开以及标签更改，随后都显示为 `<app-slug>[bot]`。这一做法一次只针对一个仓库，你的其他仓库继续使用你自己的令牌。

1. 为组织创建一个 GitHub App，将 **Issues** 和 **Pull requests** 权限设为 *Read & write*（以及 *Contents: read*，以便列出和评审 pull request），并把它安装到该仓库。
2. 写一个小脚本，在其标准输出上打印**安装令牌**（App 的私钥签发一个短期 JWT，再用它换取有效期一小时的令牌）。GitPoulpi 绝不会看到私钥。
3. 在**设置 > 仓库专属设置 > SSH 与凭据**中，于 HTTPS 下选择**令牌命令**，把脚本填入**命令**，把主机（`github.com`）填入**主机**，并勾选**用于托管平台 API（议题、pull request）**。保存并确认该命令（始终由人确认，并附带其 SHA-256）。在命令行中：`gitpoulpi repo-auth set --https token-command --hosts github.com --use-for-hosting --yes --command ./app-token.sh`。

该仓库会发生的变化：

- 它的议题和 pull request 调用（列出、创建、评论、评审、合并、关闭、标签）使用该命令的令牌，30 秒限制、内存缓存和脱敏与 git 自己的登录相同。令牌过期而托管平台回答 401 时，会再询问该命令一次。如果命令无法运行，调用会被拒绝：个人令牌绝不会作为后备。
- 没有“我”：安装令牌无法询问自己是谁。区块标题会给该配置标上 **(App 机器人)**，基于“我”的过滤器（*指派给我*、*我的*、*等待我评审*）没有可匹配的内容。不会显示为错误。
- **发布身份。**GitPoulpi 发布的每条消息都会显示它所用的名称（[对话](conversations.md#以谁的身份发布)标题区下方及其评论框中的 *以 my-app[bot] 身份发布（GitHub App 机器人）*；CLI 回答中的 `posted_as`）。使用个人令牌时那是你的登录名（`GET /user`）。使用 App 令牌时它是**发布身份**名称，例如 `my-app[bot]`：可在**设置 > 集成**中为所有仓库设置（`general.hosting_post_as`，也可用 `gitpoulpi config set general.hosting_post_as my-app[bot]`），或在**设置 > 仓库专属设置 > SSH 与凭据 > 托管平台**中为某个仓库设置，后者优先。留空时，GitPoulpi 会询问 GitHub（GraphQL `viewer { login }`），否则从它发布的第一条评论的作者得知名称，并随仓库保存。该名称也是识别你自己评论的依据，所以只有这些评论才提供编辑和删除（App 的评论还必须来自机器人账户）；在得知它之前，只有此安装发布的评论可以更改。
- App 不能被指派。请把此设置与下面的禁止指派结合使用。
- 开启或关闭**用于托管平台 API** 需要由人操作，与新命令相同。

只有 GitHub 配置会这样使用仓库的令牌；其他托管平台保留配置自己的令牌。（GitLab 项目或群组访问令牌同样是机器人：它是该配置的令牌，显示为 *项目机器人*，无需设置任何内容。）

### 在仓库中禁止指派

**设置 > 仓库专属设置 > SSH 与凭据**有一个**指派**行（默认**允许**，或**禁止**；立即保存）。禁止时，无论谁提出，该仓库的议题和 pull request 中都无法指派：

- 新建议题和新建 pull request 的表单中不再有指派人选择器（并带有说明原因的备注），命令面板的 *托管：编辑指派人…* 变灰并给出原因，行菜单会说明原因而不是打开它；
- 每个客户端都会在发送任何内容之前拒绝指派，包括创建议题或 pull request 时传入的指派人：CLI 回答 `{"refused": {"reason": "assignment_blocked", ...}}`，退出码为 2，通过 MCP 的代理得到同样的拒绝；
- 已有的指派人仍然可见（只读）。

`gitpoulpi hosting assignment` 显示该设置，`--block` 打开它，`--allow` 再关闭；只有人可以重新允许（MCP 服务器不能）。

## 9. 通过命令行

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
gitpoulpi hosting assignment --block
```

每个命令参见 [CLI 参考](cli-reference.md#命令)。令牌绝不会被当作参数接受，而且 MCP 服务器不提供 `hosting login`，所以令牌绝不会经过代理。操作系统凭据存储保存了令牌时，`hosting login` 会输出 `stored: "os"`；存储拒绝了它、令牌只在内存中保留到本次会话结束时，则输出 `stored: "session"` 加上 `store_error`。

## 故障排除

| 症状 | 解决办法 |
|---|---|
| *连接*因认证错误而失败 | 令牌错误、已过期或已被撤销。请创建新的令牌。 |
| 某个权限范围以红色列出 | 令牌没有它。连接可以工作，但该功能不行。请在托管平台上添加该权限范围，然后**编辑**并粘贴新令牌。 |
| 侧栏没有 PULL REQUESTS 分区 | 没有已启用并已连接的配置。请连接一个，然后在分区的下拉列表中关联仓库。 |
| 列表中有一行*重新连接* | 令牌丢失了（没有凭据助手时重启）或已过期。点击该行以打开配置。 |
| 自托管实例失败 | 请检查**主机域名**不含路径，并且你的计算机信任该实例的证书。 |

## 已知限制

- **来自 fork 的 pull request：**Azure DevOps 和 Bitbucket 无法检出来自 fork 的请求（参见[对行的操作](pull-requests-and-issues.md#对行的操作)）。Azure DevOps 无法从 fork 创建 pull request。
- **Trello** 最多显示看板的前 1000 张卡片。请使用列表过滤器缩小范围。**Jira**、**Linear** 和 **YouTrack** 每次刷新最多列出 500 个议题。请使用 JQL 片段或 YouTrack 查询缩小范围。
- **Linear** 每个议题只接受一位指派人：如果你在选择器中选了多个，只会发送第一个。它的“里程碑”是团队的 **cycle**（不使用项目里程碑）。**YouTrack** 的“里程碑”是 `Fix versions` 字段；版本没有编号，所以选择器按未发布版本在项目中的位置给它们编号。这两个跟踪器都没有 pull request。
- **Redmine** 每个议题只接受一位指派人（选择器中的第一个），没有标签，每次刷新最多列出 500 个议题。指派人是项目的成员（人员和群组），以显示名称区分。Kanban 看板的列就是议题状态，移动卡片会更改状态；适用 Redmine 默认的工作流规则（没有权限做该转换的角色会收到拒绝，卡片随即弹回）。在 Redmine 的默认数据中，“Resolved”不是已关闭状态：“已关闭”列是“Closed”和“Rejected”状态。
- **Bitbucket Data Center** 没有指向新 pull request、分支或 commit 的浏览器链接。pull request 页面可以正常打开。
- **Azure DevOps** 不显示作者头像。“我的”和“请求我评审”过滤器需要一个可能拒绝个人访问令牌的端点。在这种情况下，配置会在没有用户名的情况下连接，这两个过滤器不可用。无效的令牌会得到登录页面而不是错误；GitPoulpi 将其报告为令牌被拒绝。
- **Gitea 和 Forgejo：**草稿就是标题以 `WIP:` 开头的请求（服务器在创建时没有草稿标志）。请求会把所有已评审过它的人列为评审者，而撤回评审请求只会移除待处理的那些。来自 fork 本地分支的 pull request 通过 `refs/pull/<n>/head` 检出，与 GitHub 相同。提及通过服务器的 `mentioned_by` 过滤器搜索。服务器不显示令牌的权限范围，所以缺失的权限范围只有在调用被拒绝时才会发现。
- **议题看板**在每个议题来源上都能移动卡片。在 Jira 上，移动会运行通向该列的工作流转换；要求填写字段（例如解决结果）的转换会被 Jira 拒绝，卡片带着它的消息弹回原处。在 Azure DevOps 上，没有目标 state 的工作项类型会弹回原处并给出原因。
- **Bitbucket：**应用密码已不存在；请使用 API 令牌。仓库按工作区逐个列出。
- **Jira Cloud：**只有经典 API 令牌有效（不支持带权限范围的令牌，也不支持 OAuth）。新的 Jira 议题得到的是纯文本描述。
- **对话：**先加载最新的 300 条（其余用**加载更早的**）；从 GitPoulpi 发布的评论、编辑或删除是对外的操作，无法从它撤销；各托管平台不能做的事见[表格](conversations.md#各托管平台不能做什么)。在 Gerrit 上，消息或评论不能编辑或删除，Gerrit 的文本是纯文本。
- 创建 pull request、工作项、议题或卡片发生在托管平台上，**无法**在 GitPoulpi 中撤销。检出请求则可以正常撤销。
