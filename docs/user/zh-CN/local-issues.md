[English](../local-issues.md) · [Čeština](../cs/local-issues.md) · [Deutsch](../de/local-issues.md) · [Español](../es/local-issues.md) · [Français](../fr/local-issues.md) · [Italiano](../it/local-issues.md) · [日本語](../ja/local-issues.md) · [한국어](../ko/local-issues.md) · [Nederlands](../nl/local-issues.md) · [Polski](../pl/local-issues.md) · [Português (Brasil)](../pt-BR/local-issues.md) · [Português (Portugal)](../pt-PT/local-issues.md) · [Русский](../ru/local-issues.md) · [Svenska](../sv/local-issues.md) · [Türkçe](../tr/local-issues.md) · [Українська](../uk/local-issues.md) · **简体中文**

# 本地议题与 Obsidian

把你的议题保存为一个 Markdown 笔记文件夹：无需账户、无需令牌、无需联网。这个文件夹可以放在仓库内（`docs/issues/`，与代码一起 commit），
也可以在磁盘上的任何位置，例如 [Obsidian](https://obsidian.md) 库的某个子文件夹。GitPoulpi 读写这些笔记。
Obsidian、任何编辑器或 shell 都可以同时编辑它们，列表会随之更新。

![侧栏中的本地 Markdown 议题，以及其中一条笔记的行菜单：打开笔记、关闭它、编辑其标签或里程碑](../images/local-issues.png)

## 设置

1. **设置 > 集成 > 议题跟踪器 > 本地 Markdown > 添加配置文件。**输入文件夹，或点击**浏览…**并选择它。
   相对路径的文件夹（`docs/issues`）会相对于它所关联的每个仓库来解析；绝对路径（`/home/me/Vault/Projects/Issues`）则在任何地方都一样。无需输入令牌。
   - CLI：`gitpoulpi hosting integrations add --kind local_markdown --path docs/issues`。
2. 在某个仓库中，打开侧栏的**议题**区域，选择提供方 **本地 Markdown - <名称>**（文件夹随之选定；如果想用别的文件夹，
   仓库下拉列表会列出该文件夹及其直接子文件夹）。
   - CLI：`gitpoulpi hosting link issues --id <profile>`（配置文件的文件夹就是默认的 `--project`）。

不会自动关联任何内容：与所有议题来源一样，按仓库自行选择开启。

## 笔记评论

在议题区中笔记所在的行上，或在它的卡片上按 `Enter`，会打开它的对话：笔记的文本，然后是评论。评论保存在笔记文件本身中，位于最后的 `## Comments` 一节，每条是一个 `### <author> - <UTC time>` 条目（编辑过的会加上 `(edited <time>)`）。该标题之上的任何内容都不会被改动，所以 Obsidian 和 shell 读写的是同一个文件。你可以写、编辑和删除本地作者的评论（配置的 `local.me`，否则是已连接的用户，也就是*指派给我*所用的名称）；不能动别人的。输入框是带预览的常规 Markdown 文本框，*以 <name> 身份写入（本地笔记）* 指明作者。不会经过网络。你手写的带有 `## Comments` 标题的笔记，会被当作评论来读取。

## 笔记

每个议题一条笔记，命名为 `<id>-<slug>.md`（`12-crash-on-empty-name.md`），带有一个 frontmatter 块：

```yaml
---
id: 12
title: Crash on empty name
status: todo
assignees: [kai]
labels: [bug]
milestone: v0.1
created: 2026-10-14
updated: 2026-10-14T09:30:00
---
Free Markdown. GitPoulpi never rewrites the body.
```

- 新议题的 id 为 `max(id) + 1`。
- GitPoulpi 只写它自己拥有的属性（`title`、`status`、`assignees`、`labels`、`milestone`、`updated`）。
  **其他所有属性、正文、换行符和键的顺序都逐字节保留。**写入是原子的（先写临时文件，再重命名）。
- 读取很宽容：没有 `id` 的笔记从文件名取得 id，没有 `status` 的笔记为 `todo`，没有 frontmatter 的笔记以第一个 `# 标题` 作为标题。
  不是议题的文件（`readme.md`）会被忽略。
- 状态是自由文本。**未关闭**的意思是“不是完成状态”。

### 笔记中的评论

对笔记的评论会写进笔记本身，放在最后的 `## Comments` 一节中（不走网络，在 Obsidian 中也能正常阅读）：

```markdown
(the note's text)

## Comments

### kai - 2026-11-15T10:00:00Z
First comment, **any Markdown**.

### ann - 2026-11-15T11:30:00Z (edited 2026-11-15T12:00:00Z)
Second one.
```

- 条目从恰好是 `### <author> - <UTC time>` 的一行开始，编辑后在其后加上 ` (edited <UTC time>)`；它的文本一直延续到下一个条目。该节是代码围栏之外的最后一个 `## Comments` 标题，一直延伸到文件末尾，所以不要在它之后写任何内容。
- 看起来会被读成条目或标题的评论行，存储时前面会加一个反斜杠（`\### ...`，Markdown 显示的文字相同），读取时去掉它。
- 发布会追加到末尾，其上的所有内容逐字节保持不变（包括换行符）；编辑或删除只重写该节。发布还会更新笔记的 `updated`（没有 frontmatter 的笔记不会得到它）。
- 作者是配置的本地作者（“本地议题”设置中的“我”）。你只能编辑或删除自己的评论；评论的 id 是它的时间加作者，同一秒内的两次发布会把第二条顺延一秒来区分。
- 笔记没有回复、没有内部评论，也没有事件历史。

## 你可以做什么

| 在议题区域中 | 键盘 / 面板 |
|---|---|
| 新建议题（`+`，或 `n`）：标题、描述、状态、指派人、标签、里程碑 | `托管：新建议题…` |
| 从行菜单编辑指派人 / 标签 / 里程碑。从文件夹已在使用的内容中选择，或**输入一个新的标签或登录名**并选择 `添加“…”` 这一行 | `托管：编辑标签…` 及其同类项 |
| **关闭议题** / **重新打开议题**（设为完成状态，或第一个其他状态） | `本地议题：关闭议题…` / `重新打开议题…` |
| **打开笔记**（用你默认的 `.md` 程序） | `集成：打开议题…` |
| **在 Obsidian 中打开**（仅当文件夹位于某个库内，即其上层有 `.obsidian/` 文件夹时） | `本地议题：在 Obsidian 中打开…` |
| 筛选：未关闭 / 已关闭 / 全部、“指派给我” | 筛选下拉列表 |

“指派给我”匹配配置文件的 `me` 设置，否则匹配已连接的名称，再否则匹配配置文件名称。

分支和 commit 的用法与 GitHub 相同：“从议题创建分支”得到 `12-crash-on-empty-name`，commit 消息中的 `#12` 或分支名中的 `12-...` 会被识别。

在 GitPoulpi 之外所做的更改（Obsidian、shell、`git pull`）大约一秒内就会显示出来：文件夹的文件名和修改时间是在后台线程中查看的，绝不在渲染线程中。

## 配置文件

在 `config.toml` 中，紧挨着配置文件（每个键都是可选的）：

```toml
[[integrations]]
id = "vault"
kind = "local_markdown"
url = "/home/me/Vault/Projects/Issues"
token_ref = "integration:vault"

[integrations.local]
statuses = ["todo", "doing", "review", "done"]   # 看板的列顺序
done_status = "done"                            # 关闭议题
reference_prefix = "LOC"                         # 显示为 LOC-12 而不是 #12
me = "kai"                                       # “指派给我”指的是谁
```

笔记中用到但未列在 `statuses` 中的状态排在已列出的之后。`statuses` 是[看板](#kanban-看板)各列的顺序（`gitpoulpi hosting issue options` 会输出它）。

## Kanban 看板

![笔记文件夹的看板：每个状态一列，卡片带有其指派人、标签和里程碑](../images/issue-board.png)

如果你独自工作，或在把议题放在代码旁边的团队中工作，这个文件夹同时也是一块 Kanban 看板。
**议题标题栏 > 看板**，或面板中的**议题：打开看板**，会把它打开在窗口之上；按 `Esc` 关闭。

- **列**就是状态：先是 `statuses` 中列出的，按其顺序，然后是笔记用到的其他任何状态。空列也会显示。每个列标题带有其卡片数。
- **卡片**显示议题编号、标题（两行）、以首字母显示的指派人、标签和里程碑。当列放不下时，看板可横向滚动（其下的滚动条、`Shift` 加滚轮，或方向键）。
- **移动卡片**：把它拖到另一列上（该列会在指针下亮起），或用 `Shift+Left` / `Shift+Right`，或用**看板：将卡片移到…**（`m`），一个列的下拉列表。
  卡片立即移动；笔记的 `status` 按[笔记一节](#笔记)所述重写，文件中的其他内容不变。如果写入失败，卡片会回到原处，错误显示在状态栏上。
- 用 `Enter` 或双击**打开**卡片：笔记的[对话](conversations.md)会在窗口中打开，显示笔记的文本及其评论（卡片菜单的**打开笔记**会在你默认的 `.md` 程序中打开该文件）。`e` 编辑其指派人、标签或里程碑，`.` 或右键打开卡片菜单，
  `n` 在所选列中新建议题（表单关闭后会回到看板）。
- 按指派人（`a`）、标签（`l`）、里程碑（`M`）或文本（`/`）**筛选**；`x` 清除筛选。筛选是下拉列表：输入以缩小范围，`Enter` 选定。
- **实时**：在 Obsidian 或 shell 中修改的笔记，其卡片会在一秒内移动。

每个鼠标手势都有键盘按键和面板命令（**看板：…**，列在[快捷键](keybindings.md)页面中）。

## Obsidian Bases

**面板：`本地议题：创建 Obsidian Base`**（CLI：`gitpoulpi hosting issue base`）会在笔记旁写入一次 `issues.base`：
一个*表格*视图和一个按 `status` 分组、顺序与 `statuses` 一致的 *kanban* 视图。在 Obsidian 中打开它，就能在那里以看板的形式看到同样的议题。
GitPoulpi 绝不编辑已有的 `.base` 文件（无论是你的还是之前生成的）：它会拒绝并说明。格式遵循 Obsidian 的 Bases 语法
（<https://obsidian.md/help/bases/syntax>）；kanban 视图需要较新的 Obsidian。

## CLI 与 MCP

与每个议题来源相同的命令也适用于它：`hosting issues`、`hosting issue create [--list <status>]`、
`hosting issue edit <n> --add-label ...`、`hosting open issue <n>`、`hosting refs`。另有四条专用于笔记：
`hosting issue close <n>`、`hosting issue reopen <n>`、`hosting issue move <n> --status <s>` 和 `hosting issue base`。
见 [CLI 参考](cli-reference.md)。

## 限制

- 不能在选择器中输入新的**里程碑**（请选择文件夹已在使用的），但笔记中的 `milestone:` 可以是任意内容，保存后立即显示。
- 只有直接位于文件夹中的笔记才是议题（子文件夹中的不算）。
- 没有删除：请在文件管理器或 Obsidian 中删除笔记（其回收站由你自己管理）。
