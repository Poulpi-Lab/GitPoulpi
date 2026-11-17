[English](../ai.md) · [Čeština](../cs/ai.md) · [Deutsch](../de/ai.md) · [Español](../es/ai.md) · [Français](../fr/ai.md) · [Italiano](../it/ai.md) · [日本語](../ja/ai.md) · [한국어](../ko/ai.md) · [Nederlands](../nl/ai.md) · [Polski](../pl/ai.md) · [Português (Brasil)](../pt-BR/ai.md) · [Português (Portugal)](../pt-PT/ai.md) · [Русский](../ru/ai.md) · [Svenska](../sv/ai.md) · [Türkçe](../tr/ai.md) · [Українська](../uk/ai.md) · **简体中文**

# AI 辅助

GitPoulpi 从不要求账号，并且 **AI 默认关闭**。开启 AI 后，它可以编写 commit 和 stash 消息、解释更改、编写 pull request 描述、建议 merge 冲突的解决方案，并把你的工作区更改拆分为多个 commit。模型由你自己决定：你自己的 API 密钥、本地服务器，或已安装的代理 CLI（例如 `claude -p`）。

每个 AI 控件都带有紫色的 `✦`。AI 关闭时：

- 不会向任何地方发送任何内容。
- 不显示任何 AI 控件：没有 `✦` 按钮，没有 `AI:` 命令面板命令，没有 AI 菜单项，也没有 AI 按键提示。AI 按键不起任何作用。
- 仓库的 AI 页面只显示“AI 已关闭（设置 > AI）”。
- MCP 服务器不会列出 `ai_*` 工具。`gitpoulpi ai ...` CLI 命令会以同样的消息拒绝执行。

**启用 AI** 立即生效，无需重启。

在 GitPoulpi 中使用 AI 有两种方式：

| | 内置 AI（本页） | 外部代理 |
|---|---|---|
| 是什么 | GitPoulpi 向模型请求 commit 消息、解释等 | Claude Code、Cursor、Codex 等把 GitPoulpi 当作工具来驱动 |
| 在何处设置 | 设置 > AI（`[ai]`） | 代理的 MCP 配置，限制在 `[mcp]` 中 |
| 指南 | 见下文 | [MCP 与代理](mcp-and-agents.md) |

## 配置 AI

1. `Ctrl+,` > **AI** > 勾选 **启用 AI**。
2. 配置是**配置（连接）**标签页上位于页面左侧的列表（终端较窄时叠在上方）：选中一个，即可在右侧编辑它（`Enter` 进入它的各行；命令面板的 **AI：编辑配置…** 会跳到这个列表）。**+ 添加** 可再创建一个，**创建副本** 复制所选配置（不含它的密钥），**删除** 会先询问。选择 **提供商** 并填写其字段：

   | 提供商 | 字段 | 默认值 |
   |---|---|---|
   | **本地代理**（无需密钥，本身也没有账号） | **代理类型**（它是哪种代理）、**命令**（确切的程序：位于你的 `PATH` 中、别名或包装脚本，或完整路径）、使其以非交互方式运行的 **参数**，以及可选的 **环境变量**。提示词通过其 stdin 传入，答案从 stdout 读取。 | Claude Code：`claude`，`-p --model haiku` |
   | **Anthropic** | **API 密钥**，可选的 **模型** 和 **基础 URL** | `claude-sonnet-5`，`https://api.anthropic.com/v1` |
   | **OpenAI** | **API 密钥**、模型、基础 URL | `gpt-5`，`https://api.openai.com/v1` |
   | **OpenRouter** | **API 密钥**、模型、基础 URL | `openai/gpt-5`，`https://openrouter.ai/api/v1` |
   | **OpenAI 兼容**（Ollama、LM Studio、vLLM、公司网关） | **基础 URL**（`.../v1`）、**模型**，可选的密钥 | `http://localhost:11434/v1`（Ollama；LM Studio 为 `http://localhost:1234/v1`），`llama3` |

   **预设**（你始终可以输入其他值）：

   ![本地代理配置的“代理类型”下拉列表，列出 Claude Code、OpenAI Codex CLI、Gemini CLI、opencode 以及其他代理 CLI](../images/settings-ai-presets.png)

   - **本地代理 > 代理类型** 是常见代理 CLI 的下拉列表，最常用的排在前面：Claude Code、OpenAI Codex CLI、Gemini CLI、opencode、Cursor Agent、GitHub Copilot CLI、Pi、Qwen Code、Aider、Goose、Crush、Amp，或 *其他（无建议）*。类型与 **命令** 是 **相互独立的**：它只决定你能得到哪些建议。选择一个类型会用无头调用（`claude -p`、`codex exec -`、`gemini -p ...` 等）预填 **命令**（当它仍是上一个类型的默认值时）和 **参数**。**命令** 是自由文本：写入 `/opt/bin/work-claude`、包装脚本或别名目标，类型及其选项、模型列表和变量都会保留。当 **代理类型** 为空时，GitPoulpi 会根据命令推断它。
   - **参数选项** 添加或移除适合 GitPoulpi 一次性使用的选项，每项附有一行理由：不使用工具、不保存会话、纯文本输出、低推理强度、只读模式、无权限提示、无更新检查或分析。各代理的列表不同，因为代理本身不同（只有部分代理能从命令行关闭工具；对于 opencode 则是一个环境变量）。**模型** 列出该代理的模型并写入正确的标志（Claude Code 为 `--model haiku`，Codex 为 `-m gpt-6-luna`，opencode 为 `--model provider/model`）；*（默认 - 无标志）* 会将其移除。Amp 没有模型标志。`aider` 从 `/dev/stdin` 读取提示词，因此只能在 Linux 和 macOS 上使用。
   - **环境变量** 只为该代理的进程设置，绝不会作用于 GitPoulpi 或其他配置。**添加环境变量** 会提供对该代理重要的、有文档说明的变量，并说明各自的作用（禁用遥测和更新检查、强度级别、配置目录、API 密钥、基础 URL 或本地主机等）；你也可以输入 `NAME` 或 `NAME=value`。之后每个变量都有自己的 **Env NAME** 行：它的值显示为一个按钮（按下后可输入新值，离开输入框时保存），旁边的 **[x]** 按钮会删除该变量。值可以是 `${env:NAME}`，以借用 GitPoulpi 自身的某个变量（对 API 密钥很方便，这样它们不会被写入 `config.toml`）。看起来像机密的值（名称以 `_KEY`、`_TOKEN`、`_SECRET` 等结尾，或带有知名密钥前缀）显示为圆点，需要重新输入。值绝不会被记录到日志。
   - **HTTP 提供商 > 提供商预设** 会为 OpenAI、Anthropic、OpenRouter、Google Gemini、xAI、DeepSeek、Mistral、Groq、Together、Fireworks、Cerebras、Perplexity、Moonshot、Qwen（DashScope）、Zhipu（GLM），以及本地服务器 Ollama、LM Studio、llama.cpp、vLLM、Jan 和 LocalAI 填充基础 URL 和一个模型。**模型** 是该提供商的模型下拉列表，也允许输入自由文本。

3. 按 **测试连接**：*上次测试* 显示 `connected: the model answered` 或 `failed: <reason>`。
4. 有多个配置时，请在底部选择 **默认配置**；每个操作仍可指定自己的配置。

![本地代理的 AI 页面“配置”区块的下半部分：运行模式、参数、模型、工作文件夹、超时（秒）、两个 Env 行、添加环境变量和测试连接](../images/settings-ai-agent.png)

**API 密钥** 在 **API 密钥** 行上按 `Enter` 输入，并存储在系统凭据助手（Git Credential Manager、libsecret / `pass`、钥匙串）中，名称为 `ai:<profile id>`，绝不会写入 `config.toml`。没有凭据助手时，GitPoulpi 会如实说明，而不是假装保存了它。*设置 > SSH 与凭据 > 忘记所有用户名和密码* 会将它们移除。

`config.toml` 中的等价写法（其余内容见[配置参考](config-reference.md#ai内置-ai默认关闭)）：

```toml
[ai]
enabled = true
default_profile = "claude-code"

[[ai.profiles]]
id = "claude-code"
name = "Claude Code"
kind = "local_agent"      # open_ai | anthropic | open_router | open_ai_compatible | local_agent
command = "claude"        # free: an alias, a wrapper script, an absolute path
harness = "claude"        # which agent it is; empty = inferred from `command`
args = ["-p", "--model", "haiku"]

[ai.profiles.env]         # this agent's process only; ${env:NAME} borrows ours
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"
```

## 运行模式：无头或虚拟终端

本地代理配置以两种方式之一运行其代理，由 **运行模式** 选择（Claude Code、OpenAI Codex CLI 和 Gemini CLI 会显示该项）：

- **无头（非交互）**，默认：代理的一次性模式（`claude -p`），提示词走 stdin，答案从 stdout 读取。
- **交互式（虚拟终端）**：GitPoulpi 在隐藏的终端中启动代理自己的界面，像人一样输入请求，等待答案，然后关闭代理。有些代理对无头模式和交互模式的计费或限制不同；具体适用哪种、提供商的条款对两者如何规定，均由提供商决定，请查看其说明。

每种模式各自保留自己的 **参数**、**参数选项** 和 **模型**：切换模式不会丢失任何内容，交互模式只提供在没有无头标志时有意义的选项。另有两行对两种模式都适用：

- **工作文件夹**：*专用文件夹* 是 GitPoulpi 拥有的空文件夹，因此代理看不到任何项目，会请你信任它一次。*当前仓库* 会给代理项目自身的上下文（例如它的 `CLAUDE.md`），并且每个仓库请求一次信任。*默认* 在交互模式下是专用文件夹，在无头模式下仍是 GitPoulpi 运行所在的位置，与以前一致。
- **超时（秒）**：代理最多可以花多长时间，超过后 GitPoulpi 会放弃并停止它。留空表示无头 120 秒、交互式 180 秒。

**首次运行的问题由你来回答。** 从未在某个文件夹中运行过的代理会先问一些问题（信任此文件夹、登录、选择主题）。GitPoulpi 绝不替你回答：请求会停止并给出一条消息，而 **打开代理…**（在配置上，或命令面板中的 *AI：打开代理*）会在可见的终端中、于它将使用的工作文件夹里打开一次代理。回答问题，关闭代理，然后重试；代理会记住这个回答。

对于 Claude Code，答案从代理的会话记录中读取（其确切文本，之后会删除）；对于 Codex 和 Gemini，则从请求要求输出的两条标记行之间的屏幕内容中读取。同时最多进行两个虚拟终端运行，请求期间按 Esc 会停止代理。

```toml
[[ai.profiles]]
id = "claude-terminal"
kind = "local_agent"
command = "claude"
harness = "claude"
args = ["-p"]                       # the headless set, kept
run_mode = "terminal"               # headless (default) | terminal
terminal_args = ["--effort", "low"] # the interactive set
workdir = "dedicated"               # default | dedicated | repository
timeout_secs = 180                  # 0 or absent: 120 headless, 180 terminal
```

## 指令

你添加到模型所接收内容中的文本。每个操作的内置提示词（它固定了答案的格式）始终排在最前面且不可编辑；随后是四个层级，从最通用到最具体，每个层级仅在非空时才会加入：

1. **全局指令**（每个操作）。编辑器中的 *快速插入* 快捷项可插入常用句子：*用 <语言> 回复*、*保持回复简洁*、*使用祈使语气*、*技术性语气*、*轻松语气*。
2. **工作空间指令**（每个操作，该[工作空间](workspaces.md#工作空间的-ai-指令)的每个仓库）：适用于所有操作，也可按操作单独设置。在工作空间的编辑表单中编辑。
3. **仓库指令**（每个操作，仅限此仓库），带有 *与全局合并*：**追加到全局**（置于其后）或 **替换全局**（取代它们）。
4. **操作指令**（*Commit 消息生成*、*解释更改*、*Stash 消息生成*、*Pull Request*、*Commit 撰写器*、*冲突解决*、*代码评审*），每个区块在自己的编辑器中都有自己的快捷项（例如 *约定式 commit 格式*、*限制 50 个字符*）和自己的 **配置** 下拉列表（`默认（<name>）` 表示跟随默认配置）。

在 *自定义指令* 行上按 `Enter` 会打开多行编辑器（`Ctrl+S` 保存，`Esc` 取消）。文本上方有一行 *快速插入*，放着该区块的快捷项：点击其中一个，即可在光标处插入它的句子。用键盘时，`Tab` 移到这一行，`Left`/`Right` 选择快捷项，`Enter` 或 `Space` 插入并回到文本（`Tab` 或 `Esc` 不插入直接返回）。句子已在指令中的快捷项会变暗，再次插入不会有任何效果。文本部分是完整的编辑器：点击放置光标，拖动或 `Shift`+方向键选择，`Ctrl+A` 全选，`Ctrl+C` / `Ctrl+X` 复制和剪切所选内容，`Ctrl`+方向键按词跳转，`Home` / `End`、`PageUp` / `PageDown` 和鼠标滚轮可在长文本中移动。页面本身只显示每条指令的预览。

![AI 页面：全局指令，然后是 Commit 消息生成和解释更改区块，带有配置下拉列表和自定义指令](../images/settings-ai.png)

**按仓库设置**：*设置 > 仓库专属设置 > AI*（在打开仓库时）保存仓库指令，并按操作保存可覆盖全局设置的配置和指令。空字段使用全局设置（在其中以灰色显示）；**重置为全局** 会移除覆盖。保存在仓库的 `.gitpoulpi/repo-state.toml` 中，CLI 也会遵循。

![仓库的 AI 页面：仓库指令及“与全局合并”下拉列表，然后是每个操作的配置和指令](../images/settings-repo-ai.png)

### 各层级如何组合

发送的顺序是：内置提示词、**全局**、**工作空间**、**仓库**，然后是该操作自己的指令。每个层级的模式决定它如何处理排在它前面的层级：

- **追加**（默认）：该层级的文本加在更通用的层级之后。
- **替换**：该层级的文本*取代*所有更通用的层级（全局，以及仓库替换时的工作空间）。没有文本的层级绝不会替换任何东西。

操作自己的指令遵循同样的思路：先是该操作的全局指令，然后是该操作的工作空间指令（追加在其后，或取代它们），而仓库对该操作自己的覆盖设置优先于两者。所用的工作空间是窗口当前的工作空间（当它包含该仓库时），否则是第一个包含该仓库的工作空间；在命令行中是 `GITPOULPI_WORKSPACE` 指定的那个，否则是第一个。

要逐层查看究竟发送了什么，请运行 `gitpoulpi ai prompt --action <action> --repo .`（`base`、`global`、`workspace`、`repository`、`task`，以及最终的 `system_prompt`）。它不调用任何模型，AI 关闭时也能用。可以在脚本中用 `gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode append|replace] --text ...`（或 `--clear`）设置工作空间的指令。

### 提示词变量

每段指令文本（全局、工作空间、仓库、按操作）都可以包含 `${name}` 变量。它们会在提示词发送之前被替换为各自的值（保存的文本仍保留 `${...}`）。不在表中的名称会按原样发送，编辑器会对此发出警告；要表示字面的 `${`，请写 `$${`。

| 变量 | 展开为 | 示例 |
|---|---|---|
| `${uilang}` | 界面语言，使用其英文名称 | `French` |
| `${repo}` | 仓库的文件夹名 | `gitpoulpi` |
| `${branch}` | 当前分支 | `main` |
| `${ref}` / `${commit}` | 该操作所针对的 commit 的短 sha（*解释*），否则为 HEAD | `4c0443d` |
| `${commit_subject}` | 该 commit 的标题行 | `feat: add search` |
| `${date}` | 本地日期，ISO | `2026-10-03` |
| `${time}` | 本地时间，`HH:MM` | `14:05` |
| `${datetime}` | 本地日期和时间 | `2026-10-03 14:05` |
| `${user}` / `${email}` | `git config user.name` / `user.email` | `Ada Lovelace` |
| `${remote}` | 默认远程（`origin`，否则取第一个） | `origin` |
| `${host}` | `github`、`gitlab`、`other`（其他主机）或 `none` | `github` |
| `${files}` | 该操作 diff 中的文件数 | `3` |
| `${action}` | 正在运行的操作 | `commit_message` |

在指令编辑器中：输入 `${`，弹出窗口会列出匹配的变量（`Up`/`Down` 选择，`Enter` 或 `Tab` 插入，`Esc` 关闭）；或者使用左下角的 **变量** 下拉列表（`Ctrl+T`，或命令面板中的 *AI：插入提示词变量*），它会在光标处插入。每一项都有说明，当你悬停或高亮它时，会显示在下拉列表中以及按钮上方的那一行。

## 它能做什么

| 操作 | 在窗口中 | CLI |
|---|---|---|
| **Commit 消息** | stage 更改，按摘要字段右侧的 `✦`、`Ctrl+G i`，或命令面板 `AI：生成 Commit 消息`。摘要和描述会被填好供你编辑；不会 commit 任何内容。 | `ai commit-message`（打印），`commit --ai`（commit） |
| **解释更改** | 工作区更改窗格右上角的 `✦`（`Ctrl+G w`：已选中的 commit，否则为工作区更改），或 commit 菜单中的 *解释更改（AI）*。弹出窗口：方向键滚动，`c` 复制，`Esc` 关闭。 | `ai explain [rev] [--wip]` |
| **Stash 消息** | `Ctrl+G m` 打开 stash 对话框；其 AI 按钮（对话框中的 `Ctrl+G`）会询问模型。命令面板 `AI：生成 Stash 消息` 两者兼做。 | `stash push --ai` |
| **Pull request 标题和描述** | 创建 pull request 表单中的 *生成标题和描述*（分支的 commit 和 diff 在本地读取）。 | |
| **Merge 冲突解决** | 在 merge 冲突编辑器中：`✦ 建议` 工具栏按钮、冲突的右键菜单 *✦ 建议解决方案（AI）*、`Alt+I`，或命令面板 `AI：建议 merge 冲突解决方案`。建议会作为一次编辑替换输出中 *当前冲突* 的区域（标记为已编辑，占一个撤销步骤）；在你保存之前不会保存或 stage 任何内容。 | `ai resolve-conflict <path> [--conflict N]`（打印） |
| **代码评审** | 在 [pull 或 merge request 的评审](review.md#-ai-评审草稿)中：`✦ 起草评审…` 按钮、`s`，或命令面板 `评审：✦ 起草评审…`（它会打开评审选项，`Enter` 运行）。模型读取请求的 diff、描述和已有评论（默认还有项目的 `CLAUDE.md` 和 `AGENTS.md`：见[评审上下文](#评审上下文)）并提出评论；你逐条接受、编辑或忽略。在你提交评审之前，不会有任何内容发送到平台。 | `hosting review suggest <n>`（输出 JSON，不发布任何内容），`ai review-context`（默认值） |
| **拆分 commit** | commit 的菜单 *拆分 commit（AI）…*，或命令面板 `AI：拆分所选 Commit…`。拆分编辑器会带着模型给出的分组打开；见下文。 | `ai split-plan <rev>`（打印） |
| **Worktree 名称** | 在[创建 worktree 对话框](worktrees.md#用-ai-命名)中：**✦ 用 AI 命名** 按钮或 `Alt+A` 会打开一个框，你在其中描述功能、缺陷或任务；模型按仓库自身的分支命名习惯为 worktree 及其分支命名，两者都会填入对话框的字段供你编辑。只发送你的描述和本地分支名称，因此没有“发送的内容”对话框。 | `ai worktree-name --describe "<task>"`（打印） |
| **托管平台回复**和**议题描述** | 每个会发往托管平台的 Markdown 文本框的 **AI 写作**按钮（`Alt+E`）：对话中的评论、回复或编辑，评审评论或摘要，新议题的描述。草稿会放进文本框；不会发布任何内容。见[托管平台上的写作辅助](#托管平台上的写作辅助)。 | `ai hosting-reply`（输出草稿） |
| **历史重写脚本** | 在[历史重写工作台](history-rewrite.md#用-ai-编写脚本)中：脚本编辑器的 `✦ 用 AI 编写…` 按钮，或命令面板 `重写：用 AI 编写脚本…`。对话框会如实显示将要发送的内容；脚本会带着标记出现在编辑器里，绝不会替你运行或应用。 | `ai rewrite-script "<instruction>" [--sample N]`（打印） |
| **历史重写规则** | 在[历史重写工作台](history-rewrite.md#用-ai-创建规则)中：`✦ 用 AI 创建规则…` 按钮、`Alt+I`，或命令面板 `重写：用 AI 创建规则…`。一个多行对话框会如实显示将要发送的内容；规则和设置会填入工作台，不会替你预览或应用任何内容。 | `ai rewrite-rules "<description>" [--context]`（打印） |
| **Commit 撰写器** | WIP 行菜单中的 *撰写 commit（AI）...*、`Ctrl+G C`，命令面板 `AI：根据工作区更改撰写 Commit…` | |

`Esc` 会取消正在运行的请求（本地代理的进程会被终止）。

### Merge 冲突解决

模型会收到文件的路径和语言、冲突的 base、当前（A）和传入（B）代码块（带有编辑器所显示的标签）、其前后各最多 20 行已 merge 的文本，以及你的指令（全局、仓库，以及设置 > AI 中的 *冲突解决* 指令）。它以代码块的形式回答该冲突的解决文本；如果答案带有冲突标记或闲聊而非代码，会被退回一次，之后拒绝。超过 40 KB（`ai.limits.conflict_bytes`，*设置 > AI > 限制*）的冲突会被拦下并给出警告，警告中写明它的大小和限制，并提供 *仍然发送*（仅限这一次请求，仍会经过隐私检查）、*打开设置* 和 *取消*；该限制是 GitPoulpi 自己的保护措施，所以当你的模型容量足够时可以调高。AI 关闭时这些控件会隐藏，建议绝不会保存、stage 或解决任何内容：请检查输出、进行编辑，然后照常保存。

![点击闪光按钮之后的 commit 表单：由模型写好的摘要和描述，可供编辑](../images/ai-commit-message.png)

![“解释工作区更改”弹出窗口：一段解释和一份需要再次核对的事项清单](../images/ai-explain.png)

### Commit 撰写器

模型把工作区更改拆分为多个 commit（每个有改动的文件恰好属于一个 commit；无效的计划会被退回一次，之后拒绝）。只要仍有文件存在冲突，就不会发出任何请求：一个对话框会列出这些文件并打开冲突解决视图，因为撰写器需要一个没有冲突的工作树。在 commit 任何内容之前，你可以先审阅该计划：

![撰写器评审：四个拟议的 commit 及其文件，以及重置、取消和创建 4 个 commit 按钮](../images/composer.png)

| 按键 | 鼠标 | 作用 |
|---|---|---|
| `Up` / `Down`（`k` / `j`） | 点击某一行 | 在 commit 和文件之间移动 |
| `Alt+Up` / `Alt+Down`（`K` / `J`），`m` | 箭头、拖放 | 移动一个 commit；把文件移到另一个 commit |
| `Right` / `Left`，`*` | 该行的箭头 | 展开 / 折叠一个 commit，或全部（`ui.composer_files` 设置初始状态） |
| `e` / `Enter` | 铅笔 | 编辑消息 |
| `d` / `Delete` | `drop` / 恢复 | 排除一个 commit（其文件仍保持未 commit） |
| `n` | `+ 单独 commit` | 把文件移入它自己的 commit |
| `r` | `重置` | 恢复为模型的计划 |
| `Ctrl+S` / `a` | `创建 N 个 commit` | 按顺序创建它们 |
| `Esc` | `取消` | 离开，不做任何操作 |

创建时会逐个 stage 每个 commit 的文件并依次 commit；工作区绝不会被改动。只 stage 了一部分的文件会被整体 commit。如果某一步失败，它会停止并说明已创建了多少个 commit。

### 拆分 commit

模型会读取该 commit 的更改（编号方式与[拆分编辑器](log-and-history.md#拆分-commit)一致），并提议拆成几个 commit，每个都带一条消息；它可以按 hunk 乃至单独的行来分组。如果答案漏掉了某处更改、把同一处分配了两次，或指向不存在的内容，就会被拒绝。无论如何，拆分编辑器都会打开：要么已填入模型的提议，要么在模型失败时用一条备注写明原因，让你手动拆分。在你按下 **拆分**（`Ctrl+S`）之前不会拆分任何内容；分组不合适时，可以先移动更改、重命名 commit 或再加一个。所用的配置和指令是设置 > AI 中 *拆分 commit* 的那一组。

### 历史重写脚本

你描述重写要做什么（“把所有在夜间创建的 commit 移到第二天早上”，“把我的旧邮箱换成新邮箱，只限 2024 年之前”），模型就为脚本规则编写一个 [Lua 脚本](history-rewrite.md#lua-脚本)，包含头部、`rewrite_commit(c, ctx)` 以及沙箱提供的辅助函数。守护进程会编译这份答案；编译不通过时，错误会退回给模型一次。最后留下的内容会显示在编辑器里：先阅读，再测试，然后预览。细节和对话框见[重写历史](history-rewrite.md#用-ai-编写脚本)。所用的配置和指令是设置 > AI 中 *历史重写脚本* 的那一组。

### 历史重写规则

你用自己的话描述历史应如何重写（“把我的旧邮箱 a@b.org 换成 c@d.org”，“从全部历史中移除 secrets.txt”），模型就会用工作台自己的词汇提出一个范围、一些设置和一组[重写规则](history-rewrite.md#用-ai-创建规则)。**它只做准备**：答案会填入工作台，不会预览也不会应用任何内容；无法使用的条目会连同原因一起列出，有效的条目则保留。这种方式绝不会创建脚本规则。可选地，还可以一并发送范围、分支和 tag 名称以及最近 commit 的作者（一个复选框，默认关闭，显示在“发送的内容”窗格中）。所用的配置和指令是设置 > AI 中 *历史重写规则* 的那一组。

### 托管平台上的写作辅助

*托管平台回复*起草评论、评论串中的回答、对某行代码的评论或评审摘要；*议题描述*起草新议题的描述。**AI 只写草稿。**文本会放进文本框（或由 CLI 输出），由你自己阅读、编辑并发送。这个功能没有任何代码路径可以写入托管平台：它不能发布、编辑、解决或删除任何内容，CLI 和 MCP 的回答中是 `"posted": false`，本地代理以只读方式运行（见下文）。文本只会通过文本框自己的**发送**（`Ctrl+Enter`）或**评论**按钮离开 GitPoulpi。

**在 Markdown 文本框中。**开启 AI 时，每个文本会发往托管平台的文本框（[对话视图](conversations.md)中的评论、回复或编辑，评审评论，“完成评审”摘要，新议题的描述和 pull request 描述）在其模式栏右端都有一个 **AI 写作**按钮（所有文本框中都是同一个键 `Alt+E`）。它会在栏下方打开一个面板：

- 一个字段，*我希望这条回答包含什么*，在使用时输入（`Enter` 起草）。它的占位文字给出了这类内容的例子（“表示同意，并请对方补充一个失败的测试”）。**最近**列表保留以前的指令（点击其中一条，或在字段中用 `Up` / `Down`；保存在 `ai-hints.toml` 中）；
- *改进我的草稿*，文本框中已有文本时会勾选：AI 从它出发，而不是写新的；
- *允许 AI 读取仓库*，除非你把默认值打开（见下文），否则**每次都不勾选**；
- **起草**、**停止**（也可用 `Esc`）和**关闭**，以及一行显示进度或错误。

通常的“发送的内容”对话框（`[ai.sent_preview] hosting_reply`、`issue_body`、`pull_request_description`）会在请求发出之前显示它。除 system 和 user 文本外，读取仓库时它还会显示**仓库上下文**列表：每个部分及其大小、总量与上限（*31.2 KB / 48 KB*）、为符合上限截掉了什么、因敏感而排除了什么，以及哪些部分是复用的。对本地代理，它会说明代理使用只读工具自行读取仓库，列出这些工具，给出工作文件夹，以及代理会话是新建还是继续；每个对话框都以*AI 只写草稿。在你亲自发送之前不会发布任何内容。*结尾。草稿会以一个撤销步骤替换文本，文本框切换到预览，让你发送前先读一遍；`Ctrl+Z` 会取回你的文本。pull request 描述根据分支的提交和 diff 撰写，接受同样的指令；那里的仓库选项会变灰（“不适用于此文本”）。

模型得到的内容：条目的标题、状态和描述，你要回答的评论串，近期评论（最新的在前，限制在 `ai.limits.hosting_reply_context_bytes` 之内，默认 49152），托管平台接受的标记格式（因此 Jira 的回答会以 Markdown 返回，再由 GitPoulpi 转换），你希望这条回答包含的内容，以及要改进你自己的文本时，你的草稿。其他人写的所有内容在请求中都标记为不受信任的数据，所以藏在评论里的指令不会被执行。超过上限时不会悄悄截断：会询问你，并先省略最旧的评论（你要回答的评论串会保留，最多占预算的一半）；CLI 在 `truncated` 中报告截断，`--send-all` 解除限制。

**你自己的基础提示词。**两个操作共同的起始指令可以编辑：*设置 > AI > 托管平台上的回复 > 基础提示词*（`ai.hosting_reply.base_prompt`）。留空则使用内置提示词，它要求只输出文本，使用托管平台的标记格式和对话的语言，除非你的提示另有说明。你的全局、工作区和仓库指令仍会附在其后，无论你在那里写什么，其他人的文本仍标记为不受信任。`ai prompt --action hosting_reply` 会输出各层。

#### 仓库访问

**默认关闭，每次使用时勾选。**读取仓库能让“这个崩溃是出在我们的代码里吗？”这类问题得到好得多的回答，但发送的内容也更多，所以这是每次都要做的选择：*允许 AI 读取仓库*在每个框中都从不勾选开始。`ai.hosting_reply.read_repository`（*默认允许 AI 读取仓库*，关闭）只会让框一开始就勾选；`--read-repo` 是 CLI 的勾选。在三种情况下该选项会变灰，激活时会给出原因：仓库**未克隆**（Radar 中另一个仓库的对话：没有可读取的内容）、所选本地代理**没有只读模式**可供 GitPoulpi 信任，或者文本**不是根据仓库撰写的**（pull request 描述）。只有你自己的勾选才会把它打开。

之后的行为取决于配置：

| 配置 | 由谁读取仓库 | 发送什么 |
|---|---|---|
| **API**（OpenAI 兼容、Anthropic） | GitPoulpi，通过守护进程，只读 | 由它构建并交给模型的上下文：仓库地图（顶层目录树和匹配文件周围的路径）、对话提到的文件（整个文件，或该行或符号周围 40 行的窗口）、对话中标识符和错误消息的搜索结果，pull request 则还有它的 diff。限制在 `ai.limits.hosting_reply_*` 之内（上下文 49152，文件 8192，地图 6144，每个词 20 条结果，8 个词），按 diff、文件、结果、地图的顺序截断并报告。看起来敏感的文件会被排除，其余内容中的密钥会先被遮蔽（见[隐私](#隐私)）。 |
| **本地代理**（Claude Code、Codex），无头 | 代理自己，在仓库文件夹中 | 不构建任何内容：代理在仓库中以**只读工具**启动，并读取它需要的内容。Claude Code 得到 `Read`、`Grep` 和 `Glob`，`Bash`、`Edit`、`Write`、`NotebookEdit`、`WebFetch` 和 `WebSearch` 被拒绝，且不加载任何 MCP 服务器（项目自己的服务器可能会发布内容）；Codex 在它的只读沙箱中运行。你环境中的托管平台凭据（`GH_TOKEN`、`GITHUB_TOKEN`、`GITLAB_TOKEN`、Jira、Linear、Azure DevOps 和 Bitbucket 的变量、`GIT_ASKPASS`、`SSH_AUTH_SOCK`）会从它的环境中移除，所以即使配置有误的代理也无法发布或推送。 |

GitPoulpi 没有确认其只读模式的代理（以及交互式终端运行模式下的任何代理）无法获准：该选项保持变灰，并显示*此代理没有 GitPoulpi 信任的只读模式。*

**缓存与刷新上下文。**收集到的内容会被复用，让下一次回复又快又省：

- API 配置构建的上下文按仓库保留 `ai.hosting_reply.context_ttl_minutes`（*仓库上下文保留时长*，默认 30）。它以仓库、其 `HEAD`、所选内容和上限为键，各部分逐个保留，所以同一仓库的下一个议题只重建它不共享的部分。在窗口中缓存保存在内存里（最近 8 份上下文）；CLI 把它们保存在其私有状态文件夹的文件中，所以第二次 `ai hosting-reply --read-repo` 会复用第一次构建的内容。
- 已检出分支上的新提交（`HEAD` 移动）、上限的更改或时间限制会让一切重新开始。**刷新上下文**同样如此，即读取仓库时面板中的按钮，或 CLI 上的 `--refresh-context`。框下面的一行说明上一份草稿做了什么：*已构建上下文：5 个部分，31.2 KB，HEAD a1b2c3d*、*已复用上下文（4/5 个部分，HEAD a1b2c3d）*、*服务商缓存：复用 12k 个 token*、*代理已读取仓库（HEAD a1b2c3d）*、*代理会话已开始*或*代理会话已继续*。
- **Anthropic 提示词缓存。**对 Anthropic 配置，请求发送时把基础提示词和指令作为一个缓存块，仓库上下文作为第二个，你的对话、提示和草稿放在它们之后，因此在同一上下文上的第二次回复会从服务商的缓存中读取，只需一小部分费用（后续追问还会缓存对话消息和最新的草稿，见下文）。回答中的缓存计数就是*服务商缓存：复用 N 个 token* 所显示的内容。OpenAI 兼容服务器得到同样稳定的请求顺序，能受益于它们的自动前缀缓存；GitPoulpi 不会额外发送任何内容。
- **代理保留它的会话。**在**在回复之间保留代理会话**（`ai.hosting_reply.reuse_agent_session`，默认开启）且勾选了*允许 AI 读取仓库*时，支持此功能的本地代理（目前是 Claude Code）会带着会话 id 启动，之后凭它恢复，所以它读到的仓库内容保留在自己的历史中，下一次回复启动更快。其他代理以一次性方式运行。会话按仓库、`HEAD` 和操作区分；新提交或**刷新上下文**会开始新会话；该选项关闭或未勾选该框时，每次回复都是什么也不保留的一次性运行。会话保存在代理自己的存储中，GitPoulpi 只保留它的 id（窗口中在内存里，CLI 则在仅用户可读的文件中）。

**追问：更短、换一条指令。**在同一个文本框里再问一次（“改短一点”“加一句感谢”）算作一次交流。恢复的 Claude Code 会话自己记得它。其他所有情况（API 配置、Codex 以及其他所有代理），GitPoulpi 会把它还给模型：请求带上第一次请求的对话消息，然后是此前的每条指令和回答它的草稿，作为正式的聊天轮次（对只读取一份文档的代理，则是提示词中的一份记录），最后只有新的内容：你的指令，以及在勾选了*改进我的草稿*时，框中现在的文本，包含你自己的修改。所以追问从不从零开始，也不会再次读取仓库。“发送内容”会完整显示这一切。这次交流和对话一样受 `ai.limits.hosting_reply_context_bytes` 限制：最旧的追问最先被舍去，对话消息和你最新的草稿始终保留（CLI 回答中的 `context.history_omitted` 会统计被舍去的轮次）。在一次交流中，仓库上下文始终是它第一次请求的那份，因此会被复用，服务商的缓存也能命中：对 Anthropic 配置，基础提示词、仓库上下文、对话消息和最新的草稿是被缓存的部分，OpenAI 兼容服务器则得到同样的消息顺序，以受益于它的前缀缓存。Codex 本可以用 `codex exec resume <id>` 恢复，但会话 id 由它自己选定，只在它的 JSON 事件中打印出来（而且 `--last` 会接上你自己的会话），所以 GitPoulpi 让它保持一次性运行，改为重新提供历史。这次交流与文本框同生共死：文本框关闭或文本发送后它就结束，**刷新上下文**也会丢弃它（下一次请求又是第一次请求）。只有得到回答的请求才会加入它；失败或被停止的不会。pull request 描述由 diff 写成，没有交流。

在 CLI 上每次调用都是独立的，所以用 `--continue` 接续这次交流：它会延续此仓库中针对同一目标（相同的 `--item`、`--reply-to`、`--path`/`--line` 或 `--issue-title`）的上一次 `ai hosting-reply`，数据来自私有状态文件夹中的一个小文件。不带 `--continue` 则开始新的交流，并替换已保留的那次。目标没有已保留的内容时使用 `--continue` 会报错。有先前轮次一起发送时，回答的 `context` 会带有 `history_turns`（以及 `history_omitted`）。

```
gitpoulpi ai hosting-reply --conversation conversation.json --hint "say I will look tomorrow"
gitpoulpi ai hosting-reply --conversation conversation.json --hint "shorter" --continue
gitpoulpi ai hosting-reply --conversation conversation.json --reply-to <thread id> --draft "thx will do"
gitpoulpi ai hosting-reply --conversation conversation.json --read-repo --hint "is this our bug?"
gitpoulpi ai hosting-reply --issue-title "Crash on empty config" --hint "steps to reproduce"
```

`--conversation` 接受文件、表示标准输入的 `-`，或 JSON 本身：即 `hosting issue comments --json` 或 `hosting review conversation --json` 输出的 JSON（带 `header` 和 `entries` 的较旧页面也接受），所以 `gitpoulpi hosting issue comments 7 --json | gitpoulpi ai hosting-reply --conversation -` 可以工作；`--item <ref>` 则自行获取对话（`7` 或 `#7` 为议题，`!7` 为 pull request）。配合 `--read-repo`，`--pr-base REV --pr-head REV` 会把 pull request 的 diff 放入上下文；回答中的 `context` 说明读取了哪些部分、是否复用、服务商的缓存 token 数以及代理会话。MCP 工具 `ai_hosting_reply` 是同一条命令，并且是只读的。

## 密钥去向

API 密钥（OpenAI、Anthropic、OpenRouter、与 OpenAI 兼容的服务器）会绑定到它被存入时的地址：配置的基础 URL 的源（origin），记录在仅限该用户访问的私有数据文件夹里的 `trust.toml` 中。如果基础 URL 之后指向了别处，请求会在发送任何内容之前被拒绝，密钥也留在原处。运行 `gitpoulpi ai trust <profile> --accept-origin <url>`（或在应用中接受）即可有意把密钥移到新地址。

- 纯 `http://` 在本机上是允许的（`localhost` 上的 Ollama、LM Studio）；在其他任何地方都需要 `gitpoulpi ai trust <profile> --allow-insecure-http on`。
- 证书不能链到已知机构的自托管服务器，只有在你用 `--accept-certificate <sha256>` 固定了密钥指纹时才会被接受；TLS 握手仍会证明服务器持有该密钥。
- 在此检查出现之前就已有密钥的配置，会在第一次使用时按其当前基础 URL 静默记录。
- 本地代理（`claude -p` 之类）没有密钥也没有 HTTP，所以无需绑定任何东西。

## 评审上下文

代码评审草稿（`✦`，或 `hosting review suggest`）会读取请求的 diff、标题、描述和已有评论。还可以给它项目自己的约定，放在 diff 之前的 **Project conventions** 块中，
本地代理也可以在仓库内工作。默认值如下：

| 设置 | 默认 | 含义 |
|---|---|---|
| `run_in_repository` | `profile` | `profile`：由 AI 配置的*工作文件夹*决定。`yes`：不论配置如何，评审时都在仓库中运行本地代理。`no`：不在其中运行（设为*仓库*的配置会改用专用文件夹）。HTTP 配置没有工作文件夹，所以对它们没有影响。 |
| `include_claude_md` | 开 | 将仓库的 `CLAUDE.md` 提供给模型。 |
| `include_agents_md` | 开 | 将仓库的 `AGENTS.md` 提供给模型。 |
| `extra_file` | 无 | 再多一个文件：相对于仓库根目录的路径、绝对路径，或链接。 |

它们像指令一样分层，最具体的逐字段获胜：全局 `[ai.review]` 表，然后是仓库自己的值（未设置的字段继承），最后是单次运行的选择。未克隆的仓库按
`host/project` 把自己的值保存在 `remote-reviews.toml` 中，与其评审草稿放在一起。

仓库还可以把全局的额外文件**关掉**：它的 `no_extra_file = true`（*仓库专属设置 > AI* 中的**无额外文件**复选框、✦ 评审选项对话框，以及 `ai review-context` 的 `--no-extra-file` 标志）表示“此仓库不使用额外文件”，不论 `[ai.review]` 指定了什么。**空的****额外文件**字段与此不同：它继承全局文件。仓库自己的文件与这个开关互斥（设置其一会清除另一个），若手工编辑的文件同时包含两者，则开关优先。仓库中旧的 `extra_file = ""` 仍表示没有。

文件的来源：

- `CLAUDE.md`、`AGENTS.md` 和相对路径的额外文件，都是在请求所针对的分支上从仓库的**根目录**读取，绝不从工作树读取（工作树可能在另一个分支上，或有本地改动）。
  有克隆时从请求的 base commit 读取；没有克隆时从托管平台的 API 读取。
- 绝对路径从这台机器读取。
- 指向代码托管平台上某文件的链接（GitHub `.../blob/<branch>/<path>`、GitLab `.../-/blob/<branch>/<path>`、Gitea 和 Forgejo `.../src/branch/<branch>/<path>`，或原始链接）
  通过主机匹配的集成配置读取，所以私有仓库可以用你已提供的令牌访问。没有配置匹配的链接，或任何其他地址，会匿名获取。
  含斜杠的分支名无法从链接读取：请用路径，或用不含斜杠的分支上的链接。
- 缺失的 `CLAUDE.md` 或 `AGENTS.md` 会被静默跳过；缺失的额外文件会被报告（CLI 的 JSON 中的 `context.problems`）。
- 每个文件在 **8 KB** 处截断，所有文件合计在 **16 KB** 处截断，模型读到的内容中会有明显的 `[CLAUDE.md cut: showing ... of ... bytes]` 提示。

本地代理在仓库中运行时，提示词会告诉它：检出的文件可能与请求的 head 不一致，以 diff 为准。

在评审视图中，`s` 会打开 **✦ 评审选项**对话框，并填入这些值：仓库的值优先于全局值。可以只为本次运行修改，或勾选*保存为此仓库的默认值*来保留
（见[评审选项](review.md#评审选项)）。默认值也可以在*设置 > AI*（*代码评审*块，四行）和*仓库专属：AI* 中编辑，在后者中每一行都可以*使用全局值*，
*重置为全局*按钮会丢弃仓库自己的值；设置搜索可以找到它们（`包含 AGENTS.md`、`额外文件`）。无法提供给模型的内容（缺失的额外文件）或被截短以符合限制的内容，会显示在草稿卡片上。

命令行示例：

```sh
gitpoulpi ai review-context                      # 全局默认值，以及评审实际得到的内容
gitpoulpi ai review-context --agents-md off      # 保存全局默认值
gitpoulpi --repo ~/work/app ai review-context --claude-md off --context-file docs/REVIEW.md
gitpoulpi ai review-context --project github.com/acme/app --context-file https://github.com/acme/rules/blob/main/review.md
gitpoulpi ai review-context --repo ~/work/app --reset   # 重新跟随全局默认值
gitpoulpi hosting review suggest 12 --claude-md off --context-file docs/REVIEW.md --run-in-repo no
```

`ai review-context` 不需要模型，AI 关闭时也能用。`ai prompt --action code_review` 会把实际生效的上下文输出为 `review_context`。
同样的四项设置在 `config.toml` 中是 `ai.review.*`（[参考](config-reference.md#ai内置-ai默认关闭)）。

## 输入限制

GitPoulpi 放进请求的每一项输入都有大小限制，它们是 `[ai.limits]` 下的设置（*设置 > AI > 限制*）。这些限制是 GitPoulpi 自己的保护措施（不同模型的上下文窗口各不相同）：模型容量足够时可以调高，想让请求保持精简时可以调低。

| 设置（`ai.limits.`） | 默认值 | 限制的内容 |
|---|---|---|
| `conflict_bytes` | 40000 | 一个 merge 冲突的 base、当前和传入代码块 |
| `diff_bytes` | 60000 | commit 消息、解释、stash 名称、pull request 描述、commit 计划或拆分所用的 diff；代码评审的整个 diff |
| `review_file_bytes` | 12000 | 代码评审中单个文件的 diff |
| `review_text_bytes` | 4000 | 被评审的 pull request 的描述 |
| `review_threads_bytes` | 6000 | 已存在的评审评论 |
| `review_context_file_bytes` | 8192 | 评审的单个项目上下文文件（`CLAUDE.md`、`AGENTS.md`、额外文件） |
| `review_context_bytes` | 16384 | 评审的所有上下文文件合计 |
| `hosting_reply_context_bytes` | 49152 | 交给草稿回复的条目和对话 |
| `hosting_reply_file_bytes` | 8192 | 交给草稿回复的单个仓库文件（AI 读取仓库时） |
| `hosting_reply_map_bytes` | 6144 | 交给草稿回复的仓库地图 |
| `hosting_reply_hits_per_term` | 20 | 每个词保留的搜索结果数（1 到 200） |
| `hosting_reply_terms` | 8 | 查找的搜索词数量（1 到 50） |

**不会悄悄截断任何内容。** 当某项输入超过其限制时，请求会暂停，*超出 AI 限制* 对话框会列出超出的部分：哪一部分（diff、某个文件、描述、某个上下文文件）、它的大小、限制和对应的设置。评审会在同一个对话框中列出所有超出的部分。各按钮及其按键：

- *全部发送*（`A`）：仅限这一次请求，不截断任何内容。它仍会经过隐私检查（密钥被替换、敏感文件被排除），并且在开启时还会经过随后出现的 *发送内容* 窗格。
- *发送截断版本*（`T`）：超出限制的部分会被截去，和以前一样；对话框的每一行都会说明省略了什么（diff 的末尾、最后的 hunk、放不下的文件）。
- *打开设置*（`O`）：取消请求，并在列出的第一个限制处打开 *设置 > AI > 限制*。焦点从这里开始：Enter 绝不会发送。
- *取消*（`C`、`Esc`、关闭叉号或点击窗口外部）：不发送任何内容。

冲突是例外：它绝不会被截断。*仍然发送* 会把它完整发送一次。

没有任何东西会自行发起 AI 请求，所以不存在能够询问的后台运行：每个请求都是你发起的。无法询问的运行（CLI 和 MCP）会像以前一样截断，并给出说明；见下文。

**在命令行中**，没有人能回答对话框，所以超过限制的输入会像以前一样被截断，结果会说明截掉了什么：`truncated: [{part, detail?, bytes, limit, setting}]`（仅在有内容被截掉时出现；`part` 是 `diff`、`split_hunks`、`review_diff`、`review_file`、`review_text`、`review_threads`、`context_file`、`context_total` 之一）。`--send-all`（别名 `--no-truncate`）则改为完整发送所有内容：可用于 `ai`（它的所有命令，包括 `ai resolve-conflict`，其中超过限制的冲突在其他情况下会被拒绝）、`commit --ai`、`stash push --ai` 和 `hosting review suggest`。MCP 工具有同样的 `send_all` 参数和同样的 `truncated` 字段。

## 隐私

- 在你按下按钮或运行命令之前，不会发送任何内容。没有后台 AI，没有遥测，没有 GitPoulpi 账号。
- 发送的内容：提示词（先是内置的，然后是你的指令）和你所询问的 diff；对于 pull request，还包括分支的 commit 标题。超过 60 KB（`ai.limits.diff_bytes`）的 diff 绝不会被悄悄截断：见[输入限制](#输入限制)。
- 历史重写脚本请求会发送你的指令和固定的脚本约定（脚本沙箱的 API）。最多 20 个 commit 的作者、日期和标题**仅在**你于对话框中勾选 *发送样本* 时才会一并发送（默认关闭；CLI 需要 `--sample N`）。文件内容和机密值绝不会发送。
- 它只会发送到为该操作选定的配置：该 API 主机，或你的本地代理命令的 stdin（之后由该代理自行决定做什么）。
- API 密钥由 `gitpoulpi` 进程自身使用，绝不会到达守护进程、窗口的其他客户端或 MCP 客户端。
- 生成的文本绝不会被自动 commit、push 或应用。

### 请求中会清除什么

每个请求，无论由哪个操作发出、由哪个提供方接收（托管的 API 或本地代理），都会先经过同一个清理步骤：

- **密钥会被遮蔽**（`[ai.privacy] redact_secrets`，默认开启）。带有已知前缀的令牌、私钥块、JWT、URL 中的密码，以及值看起来随机的 `password = value` 行，都会被替换为 `[REDACTED:<rule>]`。它并不穷尽：它能找出常见的形态（与 `rewrite scan-secrets` 用的是同一个扫描器）。
- **敏感文件会被排除**（`[ai.privacy] exclude_sensitive_files`，默认开启）。匹配 `[privacy] sensitive_patterns` 的文件（`.env` 和 `.env.*`，但 `.example`、`.sample` 和 `.template` 文件除外，以及 `*.pem`、`*.key`、SSH 私钥、`*.p12`、`*.pfx`、`*.jks`、`*.keystore`、`*.crt`、`*.cer`、`*.der`、`*.kdbx`）的 diff 不属于请求的一部分。点名这类文件的拆分片段或撰写器文件列表也会同样被裁掉，所以针对它的方案可能会因不完整而被拒绝：请自己 stage 或拆分该文件。

关闭其中任何一项都是你自己的选择，只要它处于关闭状态，设置 > AI 就会在它下面显示一行红字。

**发送之前，先看发送的内容。** 请求离开之前，TUI 会显示**发送的内容**对话框：清理之后的指令和请求（与历史重写对话框是同一个窗格，可滚动，`Alt+W` 显示或隐藏它）、有多少个密钥被替换以及哪些文件被排除（规则和遮蔽后的预览，例如 `ghp_****abcd`，绝不显示值）、一个**不再为<操作>显示**复选框，以及**发送** / **取消**（`Esc`、关闭叉号和点击窗口外部都会取消；在“发送”之前不会有任何内容离开）。该复选框会写入 `[ai.sent_preview] <action> = false`；设置 > AI > *发送前显示所发内容…* 为每个操作提供一个开关，并有**全部重新显示**。两个历史重写对话框本来就会显示它们的请求，所以没有开关。

**在命令行中**，每个 `ai ...` 的结果（以及 `hosting review suggest`）都带有 `privacy: {redactions: N, excluded_files: [...]}`，而 `--print-sent` 会打印清理后的 system 和 user 文本，形式为 `{sent: [{system, user}], privacy, dry_run: true}`，且不调用模型（它需要 `ai.enabled = true`，不需要密钥和网络）。

**一个限制。** 工作文件夹就是仓库的本地代理（`workdir = repository`，或评审时的*在仓库中运行代理*）可以自己读取文件：清理覆盖的是 GitPoulpi 发给它的内容，而不是它自己打开的内容。使用 `dedicated` 工作文件夹可以让它远离仓库。

## 故障排除

| 症状 | 解决办法 |
|---|---|
| 哪里都没有 `✦` | AI 已关闭：*设置 > AI > 启用 AI*。 |
| `✦` 是灰色的 | 没有 stage 任何内容（commit 消息）。 |
| *测试连接*：找不到命令 | 本地代理的 **命令** 不在 GitPoulpi 看到的 `PATH` 中：请在 `claude --version` 可用的环境中启动 GitPoulpi，或给出完整路径。 |
| 本地代理卡住或显示交互式界面 | 在 *无头* 运行模式下，把它的无头标志（Claude Code 为 `-p`）加入 **参数**，或重新选择其 **代理类型** 以恢复无头调用。交互式界面正是 *交互式（虚拟终端）* 运行模式的用途。 |
| "Use "Open the agent"..." | 代理问了一个首次运行的问题（信任该文件夹、登录）。使用配置上的 **打开代理…**，回答它，关闭代理，然后重试。 |
| 虚拟终端请求超时 | 消息末尾附有代理屏幕的最后几行：权限提示、登录或更新问题会显示在那里。用 **打开代理…** 回答它，或调大 **超时**。 |
| `failed:` 401 | 密钥错误，或是另一个提供商的密钥：请重新输入。 |
| `failed:` 404 | 那里不存在该 **模型**：清空它以使用默认值。 |
| `failed:` 连接错误（Ollama、LM Studio） | 服务器未运行，或 **基础 URL** 的端口有误或缺少 `/v1`。 |
| 输入密钥后显示 `(not set)` | 没有凭据助手：请配置一个（`git config --global credential.helper ...`）并重新输入。 |
| *超出 AI 限制* | 请求中的某项输入超过了 `[ai.limits]` 的某个限制：选择 *全部发送*、*发送截断版本*、*打开设置* 或 *取消*（见[输入限制](#输入限制)）。 |
| “冲突大于限制” | 冲突超过了 `ai.limits.conflict_bytes`（默认 40000）。*仍然发送* 会发送一次；*打开设置* 会前往 设置 > AI > 限制 中的该限制。 |
| 语言或风格不对 | 添加指令；编辑器中的“快速插入”快捷项能让这件事很快完成。 |
| 撰写器提示计划无效 | 模型漏掉或重复了某个文件：请重试，或使用更强的模型。 |

## 旧版配置文件

带有 `[[ai.presets]]` 和 `[ai.defaults]` 的旧版 `config.toml` 仍可加载：

- 每个操作的默认预设会变为它的 `[ai.actions.<action>]`（`system_prompt` 变为 `instructions`）。
- 每个旧预设都保留在 `[[ai.legacy_presets]]` 中。设置 > AI 在 *旧预设（改版之前的）* 下显示它们，并带有 *删除归档*。
- 仓库原来的 `[preferences.ai_presets]` 会变为仓库覆盖设置。

内置提示词现在始终排在最前面。如果旧提示词复制了内置提示词，请删除复制的部分，只保留你想添加的内容。
