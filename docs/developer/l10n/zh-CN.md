# zh-CN terminology and style (crates/gitpoulpi-i18n/locales/zh-CN.toml)

Sources: git's own zh_CN translation (git-l10n), Microsoft zh-CN terminology, GitHub Desktop / VS Code / and TortoiseGit usage. Follow this table for any new key.

## Two vocabularies

Default ("git" variant): git words stay English. "Translate git terms" on ("local" variant): the Chinese word. Every git word is a `{$term}` token, never inline text, so both modes work. Where a concept has no token, write a pair `{ git = "..", local = ".." }`.

| Concept | git variant | local variant | Notes |
|---|---|---|---|
| commit (n./v.) | commit | 提交 | git-l10n |
| push / pull / fetch | push / pull / fetch | 推送 / 拉取 / 获取 | git-l10n |
| merge | merge | 合并 | |
| rebase | rebase | 变基 | git-l10n |
| cherry-pick | cherry-pick | 拣选 | git-l10n |
| stash | stash | 贮藏 | git-l10n; pop 弹出, apply 应用, drop 丢弃 |
| tag | tag | 标签 | |
| diff | diff | 差异 | |
| stage / unstage / staged / unstaged | same | 暂存 / 取消暂存 / 已暂存 / 未暂存 | |
| pull request / merge request | same | 拉取请求 / 合并请求 | follows the host; abbreviation PR / MR |
| squash / fixup / reword / pick / drop / edit | same | 压缩 / 修正 / 改写 / 选取 / 丢弃 / 编辑 | rebase todo actions |
| checkout, revert, reset | same | 检出, 还原, 重置 | no token: write a pair |
| worktree | worktree | 工作树 | git-l10n; "working tree" (the files) is 工作区 |
| fast-forward, upstream | same | 快进, 上游 | |
| blame | Blame | 追溯 | |
| hunk | hunk | 代码块 | merge-editor "chunk" is 块 |
| amend | amend | 修订 | |
| prune | Prune | 清理 | |
| HEAD, SHA, LFS, ours/theirs, index | unchanged | unchanged except index 索引 | |
| repository, branch, remote | 仓库, 分支, 远程 | same | never English |
| refs / settings / palette | 引用 / 设置 (never 首选项) / 命令面板 | same | |
| conflict, resolve | 冲突, 解决 | same | |
| issue, avatar, minimap | 议题, 头像, 缩略图 | same | |
| Log (the commit graph pane) | Log | same | kept English; 日志 is the application log (活动日志) |
| merge-editor panes | 当前 (A), 传入 (B), 基础, 输出 | same | not Base / Output |

"restore a file to a version" is 恢复, never 还原 (还原 = revert).

## Style

- Address: 你 (never 您); drop the pronoun when natural.
- Punctuation: full-width （ ） ： ， 。 ； ？ and “ ” quotes in sentences; an ASCII `'` only where the catalog test demands it. `…` for dialog-opening commands, never `...`.
- One space between CJK and a Latin word, number or placeholder (`在 Log 中`, `{n} 个文件`). No space between a CJK character and a `{$term}` in the local variant (`提交{$commit}` becomes `提交`): the same message is therefore usually a `git`/`local` pair, the git arm spaced, the local arm unspaced.
- Actions: verb first, no 请/将 in buttons (`保存`, `创建分支`); sentences may use 将 ... 到 ...; progress messages `正在 ...…`.
- Palette: `类别：动作` with a full-width colon; same categories as the English ones (Git, 引用, 远程, 仓库, 标签页, 视图, 应用, 设置...). Quote palette commands in Chinese, not English.
- Keys stay `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`, `Tab`, `Space` in hints (空格 only in prose).
- Dates: `2026年10月12日`, months `1月`, weekdays `周一`, relative `5 分钟前`.
- Width: CJK is 2 cells; buttons <= 4-5 characters.
- Lint limits: placeholders named `{commit}`, `{tag}`... and refs like `stash@{n}` are flagged by the local-mode lint and stay in the baseline `tests/git_vocabulary/zh-CN.toml`.

## User guide (`docs/user/zh-CN/`)

- Address 你, never 您. Mouse prose and chords use 点击 (`Ctrl+点击`, `Shift+点击`, `Alt+点击`), never 单击. The catalog itself mixes both: prefer 点击.
- The Log pane stays `Log`; 日志 is only the application log / Activity Log (活动日志) and the undo journal (撤销日志). Panes are 窗格 (引用窗格, 状态窗格, 右侧窗格), their parts 分区 (never 区段); the sidebar is 引用侧栏.
- Settings is 设置 everywhere in prose and headings (the repo-specific page is 仓库专属设置); the catalog no longer uses 首选项.
- Troubleshooting is 故障排除 (page title 故障排除与常见问题). Issue is 议题 (never "问题跟踪", never English `issue` in prose). Hosting is 托管平台, the page is 托管平台与议题跟踪器.
- A keypress sequence ("chord", e.g. `Ctrl+G` then `s`) is 键序; never 和弦 (that is the musical chord). A key combination is 组合键. A cryptographic or SSH key is 密钥, but a keyboard key is 按键 (`默认按键`, never `默认密钥`).
- worktree stays English in prose (git variant); 工作区 is the working tree (the files). Image is 图片 in the palette (`图片：放大`) and 图像 for the Settings page and its options (`设置 > 图像`).
- Full-width parentheses （ ） whenever the content or the neighbour is Chinese; ASCII quotes only inside code or data; “ ” elsewhere. Palette commands are quoted exactly with `…` (never `...`).
- The two generated tables of `keybindings.md` (`<!-- keymap:begin -->` and `<!-- palette:begin -->` blocks) are regenerated from the catalog (git vocabulary), one row per English row. Keep the markers. The `mcp-tools` block's tool names stay English, its "why" column is translated.

## Undo safety net (2026-10)

- Undo history = 撤销历史; Reflog stays `Reflog` (a git word, like Log); Lost work = 丢失的工作; "Restore only this" = 仅恢复此项 (恢复, never 还原); "Undo up to here" = 撤销到这里; the blocked-undo offer reads `将 {name} 恢复到“{op}”留下的位置` (quotes keep the Latin op word clean).
- Undo pin = 撤销固定点, dangling = 游离. `reset` stays `reset` in the git variant (pair with 重置); the `{op}` strings are verbs without a trailing 完成, the status line puts `已完成：` first so no CJK/Latin spacing breaks.
- Ellipsis: the English catalog now writes `...`; zh keeps `…` (dialog-opening rows gain it too: 删除 {name}…, 丢弃更改…).
- `应用：设置…` is now the generated palette row (the old 首选项 wording is gone from the catalog).

## Undo history labels and outside changes (2026-10)

- Outside (change made by a shell or an AI agent) = 外部 (tag), "在 GitPoulpi 之外" in prose; the page section is 在 GitPoulpi 之外所做的更改. AI agent = AI 代理.
- `[undo_label]` rows: verb first, same words as the rest of the catalog (Checkout / 检出, Revert / 还原, Reset / 重置, amend / 修订, hunk / 代码块, upstream / 上游, sparse checkout = 稀疏检出, patch = 补丁). Rows where a token meets a CJK character are `git`/`local` pairs (`{$Stage}所有更改`).
- `soft` / `mixed` / `hard` stay English in both variants (as the reset menu does). The reflog-style lines of outside entries (`checkout：从 a 切到 b`) translate the verb only.
- The git-vocabulary lint rejects both the word `index` and 索引 in a plain string that has no token: describe "only the index changed" with the `{$staged}` token (`nr_index`).
- Revert of a merge: 父 commit (`父{$commit}`) and 主线 (mainline).

## Workspaces, search, Radar (2026-10)

- **workspace = 工作空间** everywhere (never 工作区): 工作区 is already the working tree (the files), so the two must not collide. A multiplexer "workspace" (`launch_help`) is 工作空间 too. The tab-bar chip, Home pages (仓库页 / 工作空间页), palette category `工作空间：`.
- Radar, WIP, Reflog and the `+` "New tab" (新标签页) stay as proper names. Snooze = 延后 (unsnooze 取消延后), pin = 固定 (to top: 固定到顶部), Scope = 范围, "Show" filter = 显示, Review requested = 请求我评审, Needs my review = 等待我评审. Profile is 配置文件. Missing repository = 缺失, Locate = 定位.
- Strings that put a git-term token next to a CJK word (`全部{$fetch}`, `我的{$prs}`, `Checkout 分支`, `创建 worktree…`) are `git`/`local` pairs: git arm has the Latin word spaced, local arm unspaced. "X all" is `{$Fetch} 全部` (git) / `全部{$Fetch}` (local).
- Status messages are impersonal (`已固定 {item}`, `已取消延后 {item}`, `已切换到工作空间 {name}`); progress `正在 {$fetch} ... 中的全部仓库：{done}/{total}…`.
- Docs: the CLI section is headed 工作空间与搜索 and the config table `[search]`（工作空间搜索）, so the anchors are `#工作空间与搜索` and `#search工作空间搜索`.

## Review, board, local issues, new providers (2026-10, round 4)

- **review = 评审** (as in 请求我评审), "code review" = 代码评审, palette category `评审：`. *Request* (a pull/merge request, a Gerrit change, any host) = 请求 (the full 拉取请求 / 合并请求 stays the `{$pr}` token; the docs say once that 请求 is the generic word). Finish review = 完成评审, Submit review = 提交评审, Discard review = 放弃评审.
- **thread = 讨论** (GitLab's word, never 线程), resolve = 解决, reopen = 重新打开, outdated = 已过时, folded = 折叠. **pending comment = 待处理评论** (pending review = 待处理评审), "Add to review" = 加入评审, Post = 发布.
- Verdict = 结论: Approve = 批准, Request changes = 请求更改, Comment = 评论; badges 已批准 / 请求更改 / 已评论 / 已驳回. **Suggestion = 建议** (✦ AI 评审建议; Accept 接受, Dismiss 忽略, Accept all 全部接受), severity 提示 / 警告 / 问题; "Suggest review" = 建议评审.
- **patch set = 补丁集**, **vote = 投票** (Gerrit), "Check out" = 检出, Overview = 概览, Files = 文件, Commits stays the `{$Commits}` token.
- Metadata: **assignee = 指派人**, **reviewer = 评审者**, **label = 标签** (same word as tag; the context tells), **milestone = 里程碑**; chips are 指派人 / 评审者 / 标签 / 里程碑, palette `托管：编辑指派人…`.
- **encoding = 编码** (Diff view button 编码, value 自动 = Auto, 自动检测 = Auto-detect). Checks (CI) = 检查, "Mentioning me" = 提到我的.
- **board = 看板**, **column = 列**, **card = 卡片**, Open / Closed columns = 未关闭 / 已关闭 ("Open" as a state, not 打开), "Move Card To..." = 将卡片移到…, palette `看板：`; the board of local notes is the Kanban 看板 (Kanban stays Latin). **Notes folder = 笔记文件夹**, Obsidian vault = Obsidian 库, `Obsidian Base` stays Latin, palette `本地议题：`.
- Providers (Gitea, Forgejo, Codeberg, Linear, YouTrack, Redmine, Gerrit, Obsidian) stay as brand names; "User name" = 用户名.
- Chip sentences sent to a model (`chip_*_text` of `review_ai`) stay English (the model reads them).
- Layout: every new button label is at most 4 CJK characters (指派人, 评审者, 里程碑, 全部接受, 加入评审, 完成评审); `finish_with` keeps the count in full-width parentheses.
- Docs: `review.md` heading `✦ AI 评审建议` has the anchor `#-ai-评审建议` (the star is dropped by the slug, the dash stays); other anchors used: `#指令`, `#工作空间的-ai-指令`, `#指派人评审者标签和里程碑`, `#5b-议题看板`, `#kanban-看板`, `#笔记`.

## Branching workflows, Mermaid and Gerrit follow-ups (round 5)

- **workflow = 工作流** (the branching workflow = 分支工作流); `Flow` stays Latin (toolbar button, palette category `Flow：`, `Flow 菜单`, `Flow 下拉菜单`), like Log and Radar. Preset names stay Latin (Git Flow, GitHub Flow, GitLab Flow, Trunk-based, Custom). Settings page: 设置 > 仓库专属设置 > 工作流.
- **base branch = 基础分支** (badge `[基础]`, `[基础，受保护]`; "protected" 受保护), **topic type = 主题类型** and **topic branch = 主题分支** (never 话题), **parent = 父分支** (父 in tags and short labels), **environment branch = 环境分支**, **hook = 钩子**, **issue flow = 议题流程**, **finish target = 完成目标**, "Other" group = 其他.
- Actions: start 开始, finish 完成, sync 同步, promote 提升 (into an environment branch), set parent 设置父分支 (button `设父分支`, 4 characters), preview 预览, continue / abort 继续 / 中止, "Finish by" 完成方式, "Branch made by" 分支创建者. **pick up = 接手** (a branch the host or a bot made; never 取用), "Look up" 查找, "Later" 稍后, "Stop waiting" 停止等待, bot 机器人, "Wait for" 等待.
- `{label}` is a user-defined topic type name and stays as typed (`开始 {label}…`, `新建 {label} 议题`). Tokens (`{{slug}}`, `{{issue}}`...) stay literal; "token" in prose = 占位符; slug stays `Slug`; tracker key = 跟踪器 Key.
- A git token next to a Chinese character is a `git`/`local` pair, written once in git style (spaces) and unspaced in local (`{$Merge} 策略` / `合并策略`, `{$Merge} 时的 {$Tag}`); adjacent tokens (`{$merge} {$commits}`) lose their space in local mode. Docs keep the git words in Latin (`merge 策略`).
- **Mermaid** popup: title `复制为 Mermaid gitGraph` (script name Latin); detail levels 结构 / 引用 / 标题 / 完整 (the stored words `structure`, `refs`, `subjects`, `full` stay Latin in config and CLI); label 详细程度; Direction = 方向 (从左到右 / 从上到下); "fence" = 围栏; the commit subject is 标题.
- **Gerrit**: change = 变更 (docs and catalog; elsewhere 更改 is the generic "change"), **submit = 合入** (Gerrit's zh word; not 提交, which is commit and "Submit review" 提交评审), abandon = 放弃, "Compare: base" = 比较：基础, "Compare with" = 比较对象.
- Status lines stay impersonal (`已合入 {id}`, `已放弃 {id}`, `已接手 {name}`); the wait card counts in seconds (`已等待 {secs} 秒，{max} 秒后放弃`).
- Docs: `branching-workflows.md` heading anchors used by other pages: `#操作`, `#钩子`, `#名称模式`, `#由托管平台驱动的流程`, `#应用中由托管平台驱动的流程`; the Log page's `#将-commit-复制为-mermaid-gitgraph`.
