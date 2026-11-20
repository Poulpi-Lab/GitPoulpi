[English](../mcp-and-agents.md) · [Čeština](../cs/mcp-and-agents.md) · [Deutsch](../de/mcp-and-agents.md) · [Español](../es/mcp-and-agents.md) · [Français](../fr/mcp-and-agents.md) · [Italiano](../it/mcp-and-agents.md) · [日本語](../ja/mcp-and-agents.md) · [한국어](../ko/mcp-and-agents.md) · [Nederlands](../nl/mcp-and-agents.md) · [Polski](../pl/mcp-and-agents.md) · [Português (Brasil)](../pt-BR/mcp-and-agents.md) · [Português (Portugal)](../pt-PT/mcp-and-agents.md) · [Русский](../ru/mcp-and-agents.md) · [Svenska](../sv/mcp-and-agents.md) · [Türkçe](../tr/mcp-and-agents.md) · [Українська](../uk/mcp-and-agents.md) · **简体中文**

# MCP 与代理

AI 代理（Claude Code、Cursor、Codex 等）可以通过它的 **MCP 服务器**或 **CLI** 来驱动 GitPoulpi。两者与窗口使用同一个后台守护进程，所以代理所做的一切会实时显示在你打开的 GitPoulpi 中，而且两者对每条命令运行的代码完全相同。

本页讲的是*外部*代理。GitPoulpi 自己的 AI 按钮（commit 消息、解释、commit 撰写器）是单独配置的：参见 [AI 辅助](ai.md)。

## 快速设置

1. 把 `gitpoulpi` 放到你的 `PATH` 上（[安装](installation.md)）。服务器就是 `gitpoulpi mcp serve`；压缩包里的 `gitpoulpi-mcp` 中继程序仍可作为别名使用，因此已有的、写着它的代理配置会继续运行。
2. 在你的代理中注册服务器。它使用 stdio：无需启动任何东西，也没有端口。
3. 可选地在 `config.toml` 的 `[mcp]` 表中选择代理可以做什么（见下文）。默认设置允许它读取仓库（在你设置 `allow_mutations = true` 之前，更改会被拒绝），仅在你确认之后才向你展示内容，并且从不预填对话框。

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project` 会改为把它写入仓库的 `.mcp.json`，这样打开该项目的每个人都会得到它。手动编写同一个文件，并带上可选的附加项：

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "command": "gitpoulpi",
      "args": ["mcp", "serve", "--stdio", "--ui-control", "auto"],
      "env": { "GITPOULPI_CONFIG_DIR": "/home/me/.config/gitpoulpi-agent" }
    }
  }
}
```

- `--ui-control auto|ask|off` 仅对这个代理覆盖 `[mcp] ui_control`。
- `GITPOULPI_CONFIG_DIR` 给代理一个它自己的 `config.toml`（它自己的 `[mcp]` 限制，无法访问你的设置）。省略则共用你的。
- `gitpoulpi --repo <path> mcp serve --stdio` 把服务器固定到一个仓库；否则工具会作用于服务器工作目录所在的仓库，或调用的 `repo` 参数所指的仓库。

**其他客户端**：任何能启动 stdio 服务器的 MCP 客户端。

| 客户端 | 位置 | 内容 |
|---|---|---|
| Cursor、Windsurf、Claude Desktop、VS Code | 客户端的 MCP JSON 文件 | 上面的 `mcpServers` 块（VS Code 把键命名为 `servers`） |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`、`command = "gitpoulpi"`、`args = ["mcp", "serve", "--stdio"]` |
| 其他任何客户端 | 它的文档 | 命令 `gitpoulpi mcp serve`（默认使用 stdio；`gitpoulpi-mcp` 是同一个服务器，供已经在使用它的配置使用） |

**通过 HTTP**（连接到某个 URL 的客户端）：

```
gitpoulpi mcp serve --http                   # [mcp] bind, default http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000    # or: --bind 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

无状态 JSON，相同的工具，相同的 `[mcp]` 门控。每个请求都需要 bearer 令牌（`[mcp] http_auth`，默认开启）：用 `gitpoulpi mcp token --show`（或 `--copy`、`--rotate`；令牌保存在私有数据文件夹下仅限该用户访问的文件里）打印它，并作为请求头交给客户端：

```
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp \
  --header "Authorization: Bearer <token>"
```

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "type": "http",
      "url": "http://127.0.0.1:8808/mcp",
      "headers": { "Authorization": "Bearer <token>" }
    }
  }
}
```

缺少令牌或令牌错误会得到 `401` 和 `WWW-Authenticate: Bearer`。更换后的令牌立即生效，无需重启服务器。关闭 `http_auth` 只适用于你信任本机每个账户的回环端点；此时服务器会在 stderr 上发出警告。

### 从另一台机器访问端点

最安全的做法根本不需要远程监听：保留回环地址，在运行代理的机器上通过 SSH 转发端口，`ssh -L 8808:127.0.0.1:8808 host`。这样就不涉及 `--allow-remote`。

`--allow-remote`（非回环地址）会被拒绝，除非已开启身份验证并且已有令牌，**并且**流量已加密。拒绝信息会指出缺少什么。有两种方式可以做到加密：

- **服务器中的 TLS**（`[mcp] tls = "self_signed"` 或 `"files"`）。服务器自己终结 TLS：它持有密钥、执行握手，只有完成的握手才会到达 HTTP 层；纯 HTTP 客户端会握手失败，旁边也不存在纯文本监听器。所以加密是服务器在每个连接上确立的事实，而不是一个假设。使用 `self_signed` 时，首次启动会在私有数据文件夹下生成一对密钥，并打印其 SHA-256 指纹（设置中也有）；客户端通过固定该指纹或证书文件来信任服务器。使用 `files` 时，把 `tls_cert` 和 `tls_key` 设为你的 PEM 文件，客户端则通过 CA 验证证书。客户端使用 bearer 令牌进行身份验证；不使用客户端证书。
- **隧道或反向代理**（`[mcp] behind_proxy = true`）。隧道代理或反向代理与任何本机或局域网客户端一样发起连接，其请求头可以被伪造，所以服务器从内部无法判断它是否存在。因此这是一个明确的确认（在设置中带红色警告显示），而不是自动检测。可能的话，请在同一台机器上运行隧道并保留回环地址。在代理之后使用非回环地址时，代理与服务器之间的这一跳在该网络上是明文。

**不使用 MCP** 时，有 shell 的代理使用 CLI：`gitpoulpi --json <command>`（参见 [CLI 参考](cli-reference.md)）。请给它现成的技能 [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md)，它说明了何时以及如何用 GitPoulpi 取代原始的 `git`。

## 权限

| 开关 | 默认值 | 效果 |
|---|---|---|
| `[mcp] enabled` | `true` | `false`：`gitpoulpi mcp serve`（及其别名 `gitpoulpi-mcp`）拒绝启动（设置 > 代理与 MCP > *MCP 服务器*）。在服务器启动时读取：已在运行的服务器沿用其设置。 |
| `[mcp] bind` | `"127.0.0.1:8808"` | 未给出地址时 `--http` 使用的地址（除非 `--allow-remote`，否则仅限回环；`--allow-remote` 需要身份验证和加密，见上文）。设置 > 代理与 MCP > *MCP 服务器端口* 编辑其端口并保留主机。 |
| `[mcp] http_auth` | `true` | `--http` 需要 `Authorization: Bearer <token>`（`gitpoulpi mcp token`）。设置 > 代理与 MCP > *HTTP 身份验证*。 |
| `[mcp] tls`、`tls_cert`、`tls_key` | `"off"` | `off`、`self_signed` 或 `files`（你的 PEM 路径）：服务器自己终结 TLS。设置 > 代理与 MCP > *TLS*。 |
| `[mcp] behind_proxy` | `false` | 你担保隧道或反向代理会加密流量，这样 `--allow-remote` 就可以在没有 TLS 的情况下运行。 |
| `[mcp] allow_mutations` | `false` | 关闭：每个会更改内容的工具都被拒绝；读取、`ui_*` 和提案仍可使用。`true` 允许代理 commit、push、reset…… |
| `[mcp] mutation_allowlist` | `[]` | 非空：只有这些工具可以更改内容，例如 `["stage", "unstage", "commit"]`。 |
| `[mcp] allow_ui` | `true` | `false`：代理不能移动你的窗口（`ui_state` / `ui_viewers` 仍会响应）。 |
| `[mcp] ui_control` | `"ask"` | `ask`：你的窗口中出现一个需要你确认的提示块；`auto`：除非你正忙，否则直接应用；`off`：拒绝。按代理设置：`--ui-control`。 |
| `[mcp] allow_ui_propose` | `false` | 允许代理打开由你确认的预填对话框（设置 > 代理与 MCP > *允许 AI 代理提出更改供我确认*）。 |
| `[mcp] allow_sensitive_staging` | `false` | 仅存在于 `config.toml`。关闭时，`stage` 工具不提供 `allow_sensitive`，会加入敏感文件（私钥、`.env`、可识别的令牌）的 stage 会被拒绝并返回 `{error: "sensitive_files", files}`：代理应当来问你。`true` 允许它传入 `allow_sensitive`。见 [stage 与 commit](staging-and-committing.md#敏感文件)。 |
| `[mcp] allow_history_purge` | `false` | 仅存在于 `config.toml`。`true` 会列出工具 `rewrite_purge`，它会永久删除历史重写留下的旧历史（以及它的撤销记录）。关闭：代理看不到它。仍需要 `allow_mutations`。 |
| `[ui] allow_agent_view_control` | `true` | 你窗口的总开关（设置 > 代理与 MCP > *允许 AI 代理控制视图*）。关闭：没有任何代理能触及你的窗口。 |
| `[ui] launch` | `"off"` | 在没有打开窗口时，代理是否可以打开 GitPoulpi 窗口（参见[下文](#没有打开窗口或窗口在另一个会话中)）。 |

实际含义：

- **开箱即用时代理是只读的**：它可以读取、在你的窗口中向你展示内容（`ui_*`），并且在你允许时，提出由你确认的更改。要让它更改仓库（commit、push、reset……），请设置 `allow_mutations = true`；若只想授予有限权限，请保持其为 `true` 并配合 `mutation_allowlist`。每个工具都带有 MCP 注解（`readOnlyHint`、`destructiveHint`、`openWorldHint`），所以像 Claude Code 这样的客户端在执行有风险的操作前仍会询问你。
- **冲突**：`conflicts_check`（merge 之前的预测）、`conflicts_list`、`conflicts_show` 和 `conflicts_state` 是读取；`conflicts_resolve`（它会丢弃未保留的一侧，`--delete` 会删除文件）和 `conflicts_abort`（丢弃整个 merge/rebase/cherry-pick）是破坏性的；`conflicts_unresolve` 是普通写入。被拒绝的调用（`stale`、`not_conflicted`、`pick_incomplete`……）会以退出码 2 回答 `{"refused": {...}}`，而不是失败。
- **议题和 pull request 的对话**：`hosting_issue_comments` 和 `hosting_review_conversation` 读取完整时间线（读取，开放世界：它们会调用托管平台）；`hosting_markdown` 清理一段文本并找出其中的引用，不调用任何外部服务（封闭世界）。发布（`hosting_issue_comment`、`hosting_review_comment`、`hosting_review_reply`）和编辑（`hosting_*_comment_edit`）是标记为开放世界的普通更改：它们会立即以令牌所属的身份发往托管平台（回答中的 `posted_as`；机器人令牌以机器人身份发布），因此由 `allow_mutations` 和 `mutation_allowlist` 决定代理是否可以这么做。删除评论（`hosting_*_comment_delete`）是破坏性的。发布、编辑或删除之后，守护进程会得到通知，该仓库已打开的窗口会刷新那一项。`ui_show_pr` / `ui_show_issue` 接受 `--view conversation`，打开对话而不是该行。
- **卡住的状态**：`conflicts_am_continue` 是写入，`conflicts_am_skip`、`conflicts_am_abort` 和 `index_lock_remove` 是破坏性的，`index_lock_show` 是读取。
- **历史重写**：`rewrite_tools`、`rewrite_preview`、`rewrite_rows`、`rewrite_analyze`（大小分析）、`rewrite_scan_secrets`（带脱敏预览的发现项，绝不含机密值）、`rewrite_script_check`、`rewrite_script_test` 和 `rewrite_discard` 是读取（预览不会改动任何内容；它只在守护进程和 git 目录的磁盘上保存一份规划，所以 `rewrite_apply` 在重启之后也能工作）。`rewrite_apply` 和 `rewrite_run` 是破坏性的：它们会把许多引用作为一条撤销记录移到重写后的 commit 上（`undo` 可还原），并且绝不会 push。方案用 `recipe_json`（或 `recipe`，一个路径）传入，机密用 `secrets_file` 传入，绝不要内联。shell、Python 和外部 Lua 脚本（`script_language`）没有沙箱隔离，**代理无法信任它们**：没有 `rewrite_script_trust` 工具，也没有 `trust` 参数，所以这类脚本只有在有人对那段确切的文本运行过 `gitpoulpi rewrite script trust FILE` 之后才会运行（脚本一旦改动就会再次被拒绝，`script_not_trusted`，哈希在 `refused.script` 中），并且只有在 `rewrite.external_scripts` 启用了其语言时才可用（`scripts_disabled`）。被拒绝时返回 `{"refused": {"reason", "message", "detail"}}`；规划要移动已 push 的引用时，在调用设置 `yes` 之前返回 `{"needs_confirmation": ...}`（退出码 3）- 请先向用户展示。
- **代理永远无法触及**：存储令牌（`hosting login`）、创建并上传 SSH 密钥、`config set`（它可能改写自己的限制），以及守护进程的生命周期。令牌绝不会展示给代理。
- **账户范围的列表**：`hosting_radar` 是读取（跨某个配置的所有仓库，等待你评审的请求、未关闭的请求、指派给你的议题；除非加 `refresh`，否则来自五分钟缓存）；
  `hosting_integrations_radar` 会更改某个配置的 `[integrations.radar]` 表，所以由 `allow_mutations` 管控（即使不带任何选项调用也是如此，那时它只输出设置）。
- **评审上下文**：`ai_review_context` 显示代码评审草稿除 diff 外还会得到什么（CLAUDE.md、AGENTS.md、额外文件、本地代理的运行位置）。
  带设置参数时它会保存一个默认值（全局的，或 `repo` 或 `project` 自己的），所以与其他更改设置的工具一样由 `allow_mutations` 管控；不带参数时只输出。
  与所有 `ai_*` 工具一样，只有在 `ai.enabled` 开启时才会列出。`hosting_review_suggest` 接受同样的参数，但只用于单次运行。
- **`[mcp]` 门控适用于 MCP 服务器。**CLI 是你自己的 shell，不受 `allow_mutations` 约束；只有 `ui_control` 和 `allow_ui_propose` 也适用于 `gitpoulpi ui ...`。
- 被拒绝的调用是普通的工具错误（`isError: true`，原因在 `structuredContent.error` 中），绝不会是静默的空操作。

```toml
[mcp]
allow_mutations = false        # true: the agent may change the repository
mutation_allowlist = []        # e.g. ["stage", "unstage", "commit", "branch_create"]
allow_ui = true
ui_control = "ask"             # "auto" lets the agent move your view without asking
allow_ui_propose = false       # true: the agent may pre-fill Reword / Split / commit / branch / tag dialogs
allow_sensitive_staging = false  # true: the agent may stage files the staging guard flags
allow_history_purge = false    # true: the agent sees rewrite_purge (deletes old history for good)
```

## 工具

每个 CLI 命令对应一个工具，以其路径加 `_` 命名（`branch create` 即 `branch_create`）。每个工具接受该命令的参数，外加 `repo`（仓库；默认：服务器的）和 `session`。成功调用的 `structuredContent` 与 `gitpoulpi --json <command>` 打印的内容完全一致。**类型**：*read* 绝不更改任何东西；*write* 和 *destructive* 是变更操作（受门控）；*network* 与远程或托管平台通信；*view* 只移动你的窗口；*propose* 打开一个由你确认的对话框。

<!-- mcp-tools:begin -->
| Tool | CLI | Kind |
|---|---|---|
| `ai_commit_message` | `gitpoulpi ai commit-message` | read, network |
| `ai_explain` | `gitpoulpi ai explain` | read, network |
| `ai_hosting_reply` | `gitpoulpi ai hosting-reply` | read, network |
| `ai_prompt` | `gitpoulpi ai prompt` | read |
| `ai_resolve_conflict` | `gitpoulpi ai resolve-conflict` | read, network |
| `ai_review_context` | `gitpoulpi ai review-context` | write |
| `ai_rewrite_rules` | `gitpoulpi ai rewrite-rules` | read, network |
| `ai_rewrite_script` | `gitpoulpi ai rewrite-script` | read, network |
| `ai_split_plan` | `gitpoulpi ai split-plan` | read, network |
| `ai_worktree_name` | `gitpoulpi ai worktree-name` | read, network |
| `alias_status` | `gitpoulpi alias status` | read |
| `amend` | `gitpoulpi amend` | destructive |
| `apply_commit` | `gitpoulpi apply-commit` | write |
| `autostash_start_over` | `gitpoulpi autostash start-over` | destructive |
| `blame` | `gitpoulpi blame` | read |
| `branch_create` | `gitpoulpi branch create` | write |
| `branch_delete` | `gitpoulpi branch delete` | destructive |
| `branch_delete_remote` | `gitpoulpi branch delete-remote` | destructive, network |
| `branch_fast_forward` | `gitpoulpi branch fast-forward` | write |
| `branch_list` | `gitpoulpi branch list` | read |
| `branch_rename` | `gitpoulpi branch rename` | write |
| `branch_set_upstream` | `gitpoulpi branch set-upstream` | write |
| `branch_track` | `gitpoulpi branch track` | write |
| `branch_unset_upstream` | `gitpoulpi branch unset-upstream` | write |
| `branch_upstream_add` | `gitpoulpi branch upstream add` | write |
| `branch_upstream_list` | `gitpoulpi branch upstream list` | read |
| `branch_upstream_primary` | `gitpoulpi branch upstream primary` | write |
| `branch_upstream_pull` | `gitpoulpi branch upstream pull` | write |
| `branch_upstream_remove` | `gitpoulpi branch upstream remove` | write |
| `checkout` | `gitpoulpi checkout` | write |
| `cherry_pick` | `gitpoulpi cherry-pick` | write |
| `clone` | `gitpoulpi clone` | write, network |
| `commit` | `gitpoulpi commit` | write |
| `config_get` | `gitpoulpi config get` | read |
| `config_path` | `gitpoulpi config path` | read |
| `conflicts_abort` | `gitpoulpi conflicts abort` | destructive |
| `conflicts_am_abort` | `gitpoulpi conflicts am-abort` | destructive |
| `conflicts_am_continue` | `gitpoulpi conflicts am-continue` | write |
| `conflicts_am_skip` | `gitpoulpi conflicts am-skip` | destructive |
| `conflicts_check` | `gitpoulpi conflicts check` | read |
| `conflicts_list` | `gitpoulpi conflicts list` | read |
| `conflicts_resolve` | `gitpoulpi conflicts resolve` | destructive |
| `conflicts_show` | `gitpoulpi conflicts show` | read |
| `conflicts_skip` | `gitpoulpi conflicts skip` | destructive |
| `conflicts_state` | `gitpoulpi conflicts state` | read |
| `conflicts_unresolve` | `gitpoulpi conflicts unresolve` | write |
| `convert` | `gitpoulpi convert` | write |
| `diff` | `gitpoulpi diff` | read |
| `discard` | `gitpoulpi discard` | destructive |
| `fetch` | `gitpoulpi fetch` | write, network |
| `file_history` | `gitpoulpi file-history` | read |
| `flow_config_init` | `gitpoulpi flow config init` | write |
| `flow_config_show` | `gitpoulpi flow config show` | read |
| `flow_detect` | `gitpoulpi flow detect` | read |
| `flow_finish` | `gitpoulpi flow finish` | destructive, network |
| `flow_import` | `gitpoulpi flow import` | write |
| `flow_issue_new` | `gitpoulpi flow issue new` | write, network |
| `flow_issue_wait` | `gitpoulpi flow issue wait` | write, network |
| `flow_pick_up` | `gitpoulpi flow pick-up` | write, network |
| `flow_promote` | `gitpoulpi flow promote` | destructive |
| `flow_set_parent` | `gitpoulpi flow set-parent` | write |
| `flow_start` | `gitpoulpi flow start` | write, network |
| `flow_status` | `gitpoulpi flow status` | read |
| `flow_sync` | `gitpoulpi flow sync` | destructive |
| `flow_templates` | `gitpoulpi flow templates` | read |
| `hosting_assignment` | `gitpoulpi hosting assignment` | write |
| `hosting_board_move` | `gitpoulpi hosting board move` | write, network |
| `hosting_board_show` | `gitpoulpi hosting board show` | read, network |
| `hosting_checkout_pr` | `gitpoulpi hosting checkout-pr` | write, network |
| `hosting_clone` | `gitpoulpi hosting clone` | write, network |
| `hosting_edit_url` | `gitpoulpi hosting edit-url` | read, network |
| `hosting_file` | `gitpoulpi hosting file` | read, network |
| `hosting_integrations_add` | `gitpoulpi hosting integrations add` | write |
| `hosting_integrations_list` | `gitpoulpi hosting integrations list` | read |
| `hosting_integrations_radar` | `gitpoulpi hosting integrations radar` | write |
| `hosting_integrations_remove` | `gitpoulpi hosting integrations remove` | destructive |
| `hosting_integrations_test` | `gitpoulpi hosting integrations test` | read, network |
| `hosting_issue_base` | `gitpoulpi hosting issue base` | write |
| `hosting_issue_close` | `gitpoulpi hosting issue close` | write |
| `hosting_issue_comment` | `gitpoulpi hosting issue comment` | write, network |
| `hosting_issue_comment_delete` | `gitpoulpi hosting issue comment-delete` | destructive, network |
| `hosting_issue_comment_edit` | `gitpoulpi hosting issue comment-edit` | write, network |
| `hosting_issue_comments` | `gitpoulpi hosting issue comments` | read, network |
| `hosting_issue_create` | `gitpoulpi hosting issue create` | write, network |
| `hosting_issue_edit` | `gitpoulpi hosting issue edit` | write, network |
| `hosting_issue_move` | `gitpoulpi hosting issue move` | write |
| `hosting_issue_options` | `gitpoulpi hosting issue options` | read, network |
| `hosting_issue_reopen` | `gitpoulpi hosting issue reopen` | write |
| `hosting_issues` | `gitpoulpi hosting issues` | read, network |
| `hosting_labels` | `gitpoulpi hosting labels` | read, network |
| `hosting_link` | `gitpoulpi hosting link` | write |
| `hosting_list` | `gitpoulpi hosting list` | read |
| `hosting_logout` | `gitpoulpi hosting logout` | destructive |
| `hosting_markdown` | `gitpoulpi hosting markdown` | read |
| `hosting_members` | `gitpoulpi hosting members` | read, network |
| `hosting_milestones` | `gitpoulpi hosting milestones` | read, network |
| `hosting_open` | `gitpoulpi hosting open` | read |
| `hosting_pr` | `gitpoulpi hosting pr` | write, network |
| `hosting_pr_edit` | `gitpoulpi hosting pr-edit` | write, network |
| `hosting_pr_for_branch` | `gitpoulpi hosting pr-for-branch` | read, network |
| `hosting_projects` | `gitpoulpi hosting projects` | read, network |
| `hosting_prs` | `gitpoulpi hosting prs` | read, network |
| `hosting_radar` | `gitpoulpi hosting radar` | read, network |
| `hosting_refs` | `gitpoulpi hosting refs` | read |
| `hosting_review_abandon` | `gitpoulpi hosting review abandon` | write, network |
| `hosting_review_comment` | `gitpoulpi hosting review comment` | write, network |
| `hosting_review_comment_delete` | `gitpoulpi hosting review comment-delete` | destructive, network |
| `hosting_review_comment_edit` | `gitpoulpi hosting review comment-edit` | write, network |
| `hosting_review_conversation` | `gitpoulpi hosting review conversation` | read, network |
| `hosting_review_diff` | `gitpoulpi hosting review diff` | read, network |
| `hosting_review_fetch` | `gitpoulpi hosting review fetch` | write, network |
| `hosting_review_push` | `gitpoulpi hosting review push` | write, network |
| `hosting_review_reply` | `gitpoulpi hosting review reply` | write, network |
| `hosting_review_resolve` | `gitpoulpi hosting review resolve` | write, network |
| `hosting_review_show` | `gitpoulpi hosting review show` | read, network |
| `hosting_review_submit` | `gitpoulpi hosting review submit` | write, network |
| `hosting_review_submit_change` | `gitpoulpi hosting review submit-change` | write, network |
| `hosting_review_suggest` | `gitpoulpi hosting review suggest` | read, network |
| `hosting_review_threads` | `gitpoulpi hosting review threads` | read, network |
| `hosting_ssh_key_delete` | `gitpoulpi hosting ssh-key delete` | destructive, network |
| `hosting_ssh_key_list` | `gitpoulpi hosting ssh-key list` | read, network |
| `hosting_use` | `gitpoulpi hosting use` | write |
| `ignore` | `gitpoulpi ignore` | write |
| `index_lock_remove` | `gitpoulpi index-lock remove` | destructive |
| `index_lock_show` | `gitpoulpi index-lock show` | read |
| `job_cancel` | `gitpoulpi job cancel` | write |
| `job_list` | `gitpoulpi job list` | read |
| `job_wait` | `gitpoulpi job wait` | read |
| `lfs_checkout` | `gitpoulpi lfs checkout` | write |
| `lfs_fetch` | `gitpoulpi lfs fetch` | write, network |
| `lfs_files` | `gitpoulpi lfs files` | read |
| `lfs_install` | `gitpoulpi lfs install` | write |
| `lfs_lock` | `gitpoulpi lfs lock` | write, network |
| `lfs_locks` | `gitpoulpi lfs locks` | read, network |
| `lfs_prune` | `gitpoulpi lfs prune` | destructive |
| `lfs_pull` | `gitpoulpi lfs pull` | write, network |
| `lfs_push` | `gitpoulpi lfs push` | destructive, network |
| `lfs_status` | `gitpoulpi lfs status` | read |
| `lfs_track` | `gitpoulpi lfs track` | write |
| `lfs_unlock` | `gitpoulpi lfs unlock` | destructive, network |
| `lfs_untrack` | `gitpoulpi lfs untrack` | write |
| `licenses` | `gitpoulpi licenses` | read |
| `log` | `gitpoulpi log` | read |
| `lost_commits` | `gitpoulpi lost-commits` | read |
| `mailmap_add` | `gitpoulpi mailmap add` | write |
| `mailmap_check` | `gitpoulpi mailmap check` | read |
| `mailmap_list` | `gitpoulpi mailmap list` | read |
| `merge` | `gitpoulpi merge` | write |
| `patch_apply` | `gitpoulpi patch apply` | write |
| `patch_create` | `gitpoulpi patch create` | write |
| `path_status` | `gitpoulpi path status` | read |
| `pull` | `gitpoulpi pull` | write, network |
| `push` | `gitpoulpi push` | destructive, network |
| `rebase` | `gitpoulpi rebase` | destructive |
| `rebase_plan` | `gitpoulpi rebase-plan` | read |
| `redo` | `gitpoulpi redo` | destructive |
| `reflog` | `gitpoulpi reflog` | read |
| `reflog_restore` | `gitpoulpi reflog-restore` | destructive |
| `refs_hide` | `gitpoulpi refs hide` | write |
| `refs_show` | `gitpoulpi refs show` | write |
| `refs_solo` | `gitpoulpi refs solo` | write |
| `remote_add` | `gitpoulpi remote add` | write |
| `remote_clear_push_url` | `gitpoulpi remote clear-push-url` | write |
| `remote_default` | `gitpoulpi remote default` | write |
| `remote_list` | `gitpoulpi remote list` | read |
| `remote_remove` | `gitpoulpi remote remove` | destructive |
| `remote_rename` | `gitpoulpi remote rename` | write |
| `remote_set_url` | `gitpoulpi remote set-url` | write |
| `remote_group_abort` | `gitpoulpi remote-group abort` | destructive |
| `remote_group_continue` | `gitpoulpi remote-group continue` | destructive, network |
| `remote_group_plan` | `gitpoulpi remote-group plan` | read, network |
| `remote_group_retry` | `gitpoulpi remote-group retry` | write, network |
| `remote_group_status` | `gitpoulpi remote-group status` | read |
| `repo_close` | `gitpoulpi repo close` | write |
| `repo_discover` | `gitpoulpi repo discover` | read |
| `repo_init` | `gitpoulpi repo init` | write |
| `repo_list` | `gitpoulpi repo list` | read |
| `repo_open` | `gitpoulpi repo open` | write |
| `repo_recent` | `gitpoulpi repo recent` | read |
| `repo_auth_show` | `gitpoulpi repo-auth show` | read |
| `repo_auth_test` | `gitpoulpi repo-auth test` | write, network |
| `repo_config_set` | `gitpoulpi repo-config set` | write |
| `repo_config_show` | `gitpoulpi repo-config show` | read |
| `repo_config_unset` | `gitpoulpi repo-config unset` | write |
| `reset` | `gitpoulpi reset` | destructive |
| `revert` | `gitpoulpi revert` | write |
| `reword` | `gitpoulpi reword` | destructive |
| `rewrite_analyze` | `gitpoulpi rewrite analyze` | read |
| `rewrite_apply` | `gitpoulpi rewrite apply` | destructive |
| `rewrite_check_signing` | `gitpoulpi rewrite check-signing` | read |
| `rewrite_diff` | `gitpoulpi rewrite diff` | read |
| `rewrite_discard` | `gitpoulpi rewrite discard` | read |
| `rewrite_maps` | `gitpoulpi rewrite maps` | write |
| `rewrite_note` | `gitpoulpi rewrite note` | read |
| `rewrite_preview` | `gitpoulpi rewrite preview` | read |
| `rewrite_purge` | `gitpoulpi rewrite purge` | destructive |
| `rewrite_push` | `gitpoulpi rewrite push` | destructive, network |
| `rewrite_rows` | `gitpoulpi rewrite rows` | read |
| `rewrite_run` | `gitpoulpi rewrite run` | destructive |
| `rewrite_scan_secrets` | `gitpoulpi rewrite scan-secrets` | read |
| `rewrite_script_check` | `gitpoulpi rewrite script check` | read |
| `rewrite_script_test` | `gitpoulpi rewrite script test` | read |
| `rewrite_tools` | `gitpoulpi rewrite tools` | read |
| `rm_file` | `gitpoulpi rm-file` | destructive |
| `search` | `gitpoulpi search` | read |
| `session_delete` | `gitpoulpi session delete` | destructive |
| `session_list` | `gitpoulpi session list` | read |
| `session_new` | `gitpoulpi session new` | write |
| `session_rename` | `gitpoulpi session rename` | write |
| `show` | `gitpoulpi show` | read |
| `sparse_checkout_add` | `gitpoulpi sparse-checkout add` | write |
| `sparse_checkout_disable` | `gitpoulpi sparse-checkout disable` | write |
| `sparse_checkout_set` | `gitpoulpi sparse-checkout set` | write |
| `sparse_checkout_status` | `gitpoulpi sparse-checkout status` | read |
| `split_commit` | `gitpoulpi split-commit` | destructive |
| `split_source` | `gitpoulpi split-source` | read |
| `ssh_command_clear` | `gitpoulpi ssh-command clear` | write |
| `ssh_command_set` | `gitpoulpi ssh-command set` | write |
| `ssh_command_show` | `gitpoulpi ssh-command show` | read |
| `stage` | `gitpoulpi stage` | write |
| `stash_apply` | `gitpoulpi stash apply` | write |
| `stash_drop` | `gitpoulpi stash drop` | destructive |
| `stash_from_commit` | `gitpoulpi stash from-commit` | write |
| `stash_list` | `gitpoulpi stash list` | read |
| `stash_pop` | `gitpoulpi stash pop` | write |
| `stash_push` | `gitpoulpi stash push` | write |
| `status` | `gitpoulpi status` | read |
| `submodule_init` | `gitpoulpi submodule init` | write |
| `submodule_list` | `gitpoulpi submodule list` | read |
| `submodule_update` | `gitpoulpi submodule update` | write, network |
| `tag_annotate` | `gitpoulpi tag annotate` | write |
| `tag_create` | `gitpoulpi tag create` | write |
| `tag_delete` | `gitpoulpi tag delete` | destructive, network |
| `tag_delete_remote` | `gitpoulpi tag delete-remote` | destructive, network |
| `tag_fetch` | `gitpoulpi tag fetch` | write, network |
| `tag_list` | `gitpoulpi tag list` | read |
| `tag_push` | `gitpoulpi tag push` | write, network |
| `theme_export` | `gitpoulpi theme export` | read |
| `theme_list` | `gitpoulpi theme list` | read |
| `treat` | `gitpoulpi treat` | write |
| `ui_back` | `gitpoulpi ui back` | view |
| `ui_highlight` | `gitpoulpi ui highlight` | view |
| `ui_notify` | `gitpoulpi ui notify` | view |
| `ui_propose_branch` | `gitpoulpi ui propose branch` | propose |
| `ui_propose_commit_message` | `gitpoulpi ui propose commit-message` | propose |
| `ui_propose_resolution` | `gitpoulpi ui propose resolution` | propose |
| `ui_propose_reword` | `gitpoulpi ui propose reword` | propose |
| `ui_propose_rewrite` | `gitpoulpi ui propose rewrite` | propose |
| `ui_propose_split` | `gitpoulpi ui propose split` | propose |
| `ui_propose_tag` | `gitpoulpi ui propose tag` | propose |
| `ui_show_commit` | `gitpoulpi ui show commit` | view |
| `ui_show_conflict` | `gitpoulpi ui show conflict` | view |
| `ui_show_diff` | `gitpoulpi ui show diff` | view |
| `ui_show_file` | `gitpoulpi ui show file` | view |
| `ui_show_home` | `gitpoulpi ui show home` | view |
| `ui_show_issue` | `gitpoulpi ui show issue` | view |
| `ui_show_keys` | `gitpoulpi ui show keys` | view |
| `ui_show_palette` | `gitpoulpi ui show palette` | view |
| `ui_show_pr` | `gitpoulpi ui show pr` | view |
| `ui_show_ref` | `gitpoulpi ui show ref` | view |
| `ui_show_search` | `gitpoulpi ui show search` | view |
| `ui_show_settings` | `gitpoulpi ui show settings` | view |
| `ui_show_tab` | `gitpoulpi ui show tab` | view |
| `ui_show_wip` | `gitpoulpi ui show wip` | view |
| `ui_state` | `gitpoulpi ui state` | view |
| `ui_viewers` | `gitpoulpi ui viewers` | view |
| `undo` | `gitpoulpi undo` | destructive |
| `undo_forget` | `gitpoulpi undo-forget` | write |
| `undo_restore` | `gitpoulpi undo-restore` | write |
| `unstage` | `gitpoulpi unstage` | write |
| `workspace_add` | `gitpoulpi workspace add` | write |
| `workspace_create` | `gitpoulpi workspace create` | write |
| `workspace_delete` | `gitpoulpi workspace delete` | destructive |
| `workspace_list` | `gitpoulpi workspace list` | read |
| `workspace_remove` | `gitpoulpi workspace remove` | write |
| `workspace_rename` | `gitpoulpi workspace rename` | write |
| `workspace_set_ai_instructions` | `gitpoulpi workspace set-ai-instructions` | write |
| `workspace_set_profile` | `gitpoulpi workspace set-profile` | write |
| `workspace_show` | `gitpoulpi workspace show` | read |
| `worktree_add` | `gitpoulpi worktree add` | write |
| `worktree_changes` | `gitpoulpi worktree changes` | read |
| `worktree_list` | `gitpoulpi worktree list` | read |
| `worktree_remove` | `gitpoulpi worktree remove` | destructive |

Not exposed as tools:

| CLI | Why |
|---|---|
| `gitpoulpi hosting integrations trust` | accepting a changed address, account or certificate is the user's decision; an agent is told the security check needs the user |
| `gitpoulpi ai trust` | binding an API key to another address is the user's decision |
| `gitpoulpi mcp serve` | the MCP server cannot spawn/serve a copy of itself |
| `gitpoulpi mcp token` | prints, copies and rotates the key that guards this very server: an agent must not read or replace it |
| `gitpoulpi completions` | shell completion generation, not a GitBackend operation |
| `gitpoulpi shell-init` | prints a wrapper function for the user's own shell, not a GitBackend operation |
| `gitpoulpi desktop shortcut status` | launcher shortcuts are the user's desktop, not a repository; they need the TUI's icon renderer |
| `gitpoulpi desktop shortcut install` | writes files into the user's launcher, Start menu and Desktop |
| `gitpoulpi desktop shortcut remove` | deletes files from the user's launcher, Start menu and Desktop |
| `gitpoulpi alias create` | writes a command into the user's PATH; an agent never changes the user's shell |
| `gitpoulpi alias remove` | deletes a command from the user's PATH; an agent never changes the user's shell |
| `gitpoulpi installation show` | describes this machine's GitPoulpi installations (folders, environments); local, like desktop and alias |
| `gitpoulpi installation set` | changes this installation's own names, colour, aliases and environment in the user's config |
| `gitpoulpi path add` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi path remove` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi uninstall` | removes GitPoulpi's own program files, aliases, PATH entries and shortcuts from the machine |
| `gitpoulpi launch` | starts a terminal window on the user's desktop; the launcher shortcuts' own entry point |
| `gitpoulpi daemon run` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon start` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon stop` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon restart` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon status` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi rewrite script trust` | trusting an unsandboxed script is a person's decision; an agent can only run scripts a person trusted |
| `gitpoulpi config set` | an agent must not be able to rewrite its own gating config |
| `gitpoulpi hosting ssh-key create` | writes a private key and uploads it; too dangerous to let an agent automate |
| `gitpoulpi hosting login` | takes a secret token on stdin; a token must not pass through an agent's tool-call transcript |
| `gitpoulpi repo-auth set` | which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token |
| `gitpoulpi repo-auth clear` | which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token |
| `gitpoulpi flow trust-hooks` | trusting the shell commands a repository commits is a person's decision; an agent can only run hooks a person trusted |
<!-- mcp-tools:end -->

### 分支工作流操作先出计划再执行

`flow_start`、`flow_finish`、`flow_sync`、`flow_promote` 和 `flow_set_parent`（[分支工作流](branching-workflows.md#操作)）在只有一次裸调用时绝不会运行。`dry_run: true` 会把计划（每一个 git 步骤和命令、警告、undo 无法撤回的内容、阻碍项）作为普通结果返回；`yes: true` 运行它；两者都不带时，调用会被拒绝并附带计划。工作流文件的 `[hooks]` 是来自团队共享文件的 shell 命令，所以只有在明确给出 `run_hooks: true` 时才会运行：计划会列出它们（`hooks`、`hooks_run`），以便你先把确切的命令展示给用户。因冲突而停下的 merge 会返回 `status: "conflicted"`（错误结果，退出码 2）；在冲突工具解决了文件之后，用 `continue_: true` 或 `abort: true` 再次调用同一个工具。创建 merge request（`merge-request` 模式下的 `finish`）无法撤销，计划中会说明这一点。

### 代理用纯 `git` 做的事也在撤销历史里

代理若用自己的 `git`（而不是通过这些工具）创建分支和 worktree、commit、reset 或丢弃 stash，这些事照样看得见：守护进程会把仓库的引用与它上次看到的做对比，把每一处变化记为一个 `undo --list` 条目，带有 `"source": "outside"`、git 记录的原因（`outside.message`）和 git 记下的提交者。`undo`、`redo` 和 `undo_restore` 工具对待这些条目和其他条目一样，所以代理（或用户，按 `Ctrl+Z`）可以带回被删除的分支。文件的修改绝不会被记录。参见[撤销与重做](undo-redo.md#在-gitpoulpi-之外所做的更改)。

## 向用户展示内容

代理可以直接指给你看，而不是用文字描述：`ui_show_commit`、`ui_show_diff`（某文件的某一行，内联或分栏）、`ui_show_file`（某一行处的内容、blame 或历史）、`ui_show_ref`、`ui_show_pr`、`ui_show_issue`、`ui_show_conflict`（在有冲突文件上打开 merge 编辑器，定位到某个冲突）、`ui_show_wip`（`composer` 把光标放到 commit 摘要中，不输入任何内容）、`ui_show_tab`、`ui_show_home`、`ui_show_search`、`ui_show_palette`（输入查询，但绝不运行）、`ui_show_keys`、`ui_show_settings`（一个页面和一项设置，绝不编辑）、`ui_highlight`（为打开的 diff 或文件的若干行着色）、`ui_notify`（一条 toast）和 `ui_back`（撤销代理的上一次视图更改）。`ui_state` 读取窗口显示的内容（`text: true` 会加上可见文本）；`ui_viewers` 列出窗口。

这些工具**绝不会更改仓库**，也不能键入、点击或编辑设置。修订和路径会在任何内容到达窗口之前被检查。

**用户看到什么、决定什么**：

- `ask`（默认），或者当你正在输入、对话框或设置已打开、你正在拖动、或你在最近 1.5 秒内按过键时：代理会立刻得到 `deferred`，你的状态栏会显示一个提示块 *“claude-code wants to show ...”*。`Ctrl+N Enter` 或点击它即可显示；`Esc` 或 `Ctrl+N Backspace` 将其忽略。它在 60 秒后过期。
- 已应用：视图移动，一条 4 秒的 toast 说明是谁移动的，标签页会出现紫色的 `✦`，直到你的下一次按键。`Ctrl+N Backspace`（命令面板 `代理：返回`）可以返回；窗口会记住十个视图。
- 绝不会取消隐藏或检出任何东西：被你的单独显示过滤器隐藏的 commit 会回答 `partial`（`hidden_by_filter`）；未关联到平台的 PR 或议题列表会回答 `hosting_not_linked`。

**回答**：`{"outcome": "applied" | "partial" | "deferred" | "refused" | "not_found" | "no_viewer" | "proposed", ...}`，应用时带有窗口的 `state`。CLI 退出码：0 表示 applied/partial/proposed，3 表示 deferred，其他情况为 1。调用有速率限制（突发 5 次，之后每秒一次）。

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "The bug is the early return above."
$ gitpoulpi --json ui back
```

### 提出更改（`ui_propose_*`）

设置 `allow_ui_propose = true` 后，代理可以打开你的某个对话框并**预填**内容，标有 *✦ Proposed by <agent>*。在你按下该对话框自己的确认按钮之前什么都不会发生；你可以先编辑或取消。

| 工具 | 打开 | 你通过以下方式确认 |
|---|---|---|
| `ui_propose_reword`（rev、message） | Reword 弹出窗口 | *更新消息*（已 push 的 commit 仍会询问是否 force push） |
| `ui_propose_commit_message`（summary、body） | 已填写的 commit 表单；绝不 commit | Commit 按钮 |
| `ui_propose_branch`（name、start point） | 内联的新建分支框 | `Enter` |
| `ui_propose_tag`（name、target、message） | 内联的新建 tag 框（有消息时为附注 tag） | `Enter` |
| `ui_propose_split`（rev、plan file） | 在该 commit 上打开拆分编辑器，内容由计划填入 | *拆分*（`Ctrl+S`）；`Esc` 即拒绝 |
| `ui_propose_rewrite`（方案文件） | 打开历史重写工作台，列表中已有方案的规则；不会预览或应用任何内容 | 你自己的*预览*，然后*应用*（`Ctrl+Enter`）；`Esc` 即拒绝。带脚本的方案会被拒绝，机密值会被丢弃 |
| `ui_propose_resolution`（path、pick 或 content file） | 在有冲突文件上打开 merge 编辑器，选择（或整段文本）已放在输出中 | *保存并标记为已解决*（`Ctrl+S`）；关闭即拒绝 |

一次只能有一个提案；当你在撰写器中有自己的草稿时，commit 消息提案会被拒绝；当 merge 编辑器中有你未保存的工作、或文件的 stage 阶段自代理读取后已改变时，解决方案提案会被拒绝。你在确认之前修改过的解决方案会被报告为 `accepted_edited`。回答中带有一个 `proposal` id；你的决定可在 `ui_state` 的 `proposals` 中看到（`offered`、`open`、`accepted`、`accepted_edited`、`rejected`、`expired`、`failed`）。

### 没有打开窗口，或窗口在另一个会话中

- 窗口属于它被启动时所在的会话。如果代理的会话中没有窗口，`ui show` 会在另一个会话中查找打开了该仓库的**一个**窗口并使用它（回答中的 `routed_session`）。如果有多个匹配：回答 `no_viewer` 并点名它们；请传入 `--session`。
- **为代理打开窗口**默认关闭。把 `[ui] launch`（设置 > 代理与 MCP > *为代理打开窗口*）设为终端或多路复用器预设，或 `custom` 命令；然后带 `launch: true` 的调用（CLI `gitpoulpi ui --launch show ...`）会打开一个并最多等待 10 秒。若 `ui_control = "auto"`，即使没有该标志也会打开；若为 `ask`，代理会得到 `no_viewer`，并应告知你。

| 预设 | 命令 |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}`（你的默认终端） |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux`（新窗口）、`zellij`（新窗格）、`cmux`（新工作区） | 在运行中的多路复用器内，位于仓库文件夹中 |
| `custom` | `[ui] launch_command`，例如 `my-term --cd {repo} {gitpoulpi} --session {session}`；拆分为参数，不经过 shell |

如果某个预设不适用于你的程序版本，请使用 `custom`。

## 错误

- 格式错误的调用（未知的工具或参数、缺少必需参数）是 JSON-RPC `-32602 invalid_params` 错误。
- 拒绝，或 git 或守护进程的错误（错误的修订、冲突等），是工具错误：`isError: true`，消息在 `content` 和 `structuredContent.error` 中。
