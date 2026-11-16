[English](../staging-and-committing.md) · [Čeština](../cs/staging-and-committing.md) · [Deutsch](../de/staging-and-committing.md) · [Español](../es/staging-and-committing.md) · [Français](../fr/staging-and-committing.md) · [Italiano](../it/staging-and-committing.md) · [日本語](../ja/staging-and-committing.md) · [한국어](../ko/staging-and-committing.md) · [Nederlands](../nl/staging-and-committing.md) · [Polski](../pl/staging-and-committing.md) · [Português (Brasil)](../pt-BR/staging-and-committing.md) · [Português (Portugal)](../pt-PT/staging-and-committing.md) · [Русский](../ru/staging-and-committing.md) · [Svenska](../sv/staging-and-committing.md) · [Türkçe](../tr/staging-and-committing.md) · [Українська](../uk/staging-and-committing.md) · **简体中文**

# stage 与 commit

如何查看你做了哪些更改、精确选择哪些内容进入一次 commit、写好 commit，以及更深入地查看文件（blame、历史、内置编辑器）。*WIP 行*和 *hunk* 等术语在[界面导览](interface-tour.md)中有解释。

## 工作更改

未 commit 的工作是**Log 的第一行**（虚线圆圈，`// WIP`，带有 `✎ n` 已修改和 `+ n` 已添加的计数）。选中它，右侧窗格就会显示工作更改：

![工作更改：右侧是 unstaged 与 staged 的文件，下方是 commit 表单](../images/hero.png)

- **unstaged 的文件**：已编辑、已添加（未跟踪）和已删除、但尚未进入下一次 commit 的文件。
- **staged 的文件**：下一次 commit 将包含的内容。
- **重命名或移动的文件**在两个列表中都是一行，`新名 <- 旧名`（磁盘上已删除的文件与一个内容至少 50% 相同的新文件，即使尚未 stage 也算作一次移动）。stage、unstage 和丢弃会同时作用于两个名称：stage 会 stage 删除和新文件，所以 git 看到的是一次重命名，并且只算**一步**撤销（`Undo Stage rename old -> new`）。打开该行会显示相对旧名称的内容 diff，标题为 `旧名 -> 新名（N% 相似）`；没有任何修改的移动会显示**内容无更改**。commit 的文件也是如此，`gitpoulpi status` / `gitpoulpi diff` 会报告 `renamed_from`。
- **路径**或**树**（`m`）以扁平方式或按文件夹列出文件。**全部折叠**（`z`）折叠所有文件夹。**查看所有文件**（`a`）显示整棵树，而不只是已更改的文件。右键点击列表的空白处可选择**全部展开** / **全部折叠**，右键点击文件夹行可选择**展开所有子文件夹** / **折叠所有子文件夹**；按键 `=`、`-`、`*`、`_`（见[鼠标与手势](mouse-and-gestures.md)）。
- 标题会说明在哪个分支上有多少个文件发生了更改。

### stage、unstage、丢弃

| 想要 | 做法 |
|---|---|
| stage 一个文件 | 悬停在它的行上并点击 **stage 文件**，或选中它后按 `Ctrl+Space`，或 `Ctrl+G s` |
| stage 一个文件夹 | 文件夹行上的 **stage 文件夹** |
| stage 全部 | 分区标题中的 **全部 stage**，或 `Ctrl+G a` |
| unstage | 在 staged 分区中使用相同的按钮和按键（`Ctrl+G u`） |
| 丢弃一个文件的更改 | 右键点击文件 > **丢弃更改**（会先询问；`Ctrl+Z` 可带回这些更改） |

### 敏感文件

stage 一个文件、一个文件夹或全部内容之前，会先检查它将加入什么。文件名匹配 `[privacy] sensitive_patterns`（`.env` 和 `.env.*`，但 `.example`、`.sample` 和 `.template` 的除外，以及 `*.pem`、`*.key`、SSH 私钥、`*.p12`、`*.pfx`、`*.jks`、`*.keystore`、`*.crt`、`*.cer`、`*.der`、`*.kdbx`），或者文本中含有扫描器可识别的密钥的文件，不会被 stage。**敏感文件**问题会列出每个文件及其原因（一个模式，或带遮蔽预览的密钥种类，绝不显示值），并提供 **1. 仍然 stage** 或 **2. 取消，不 stage 任何文件**。在你按下方向键或数字键之前，没有任何答案被高亮，所以误按 Enter 不会作答；`Esc`、关闭叉号和点击窗口外部都会取消。在文件中发现的密钥会指出它所在的行（`config/app.env:12`）。想先看看再决定，可以点击某条发现（或用 `Tab` / `Shift+Tab`、`Ctrl+J` / `Ctrl+K` 在它们之间移动），再按**在文件中显示**按钮、`S`，或命令面板中的**敏感文件：在文件中显示**：文件会在文件视图中打开，滚动到该行并高亮它，关闭文件视图（`Esc`）后，询问原样回来。查看不会 stage 任何内容。无论来自键盘、按钮、菜单、命令面板还是拖放，都是同一个问题，因为它们发出的是同一个请求。取消不会 stage 任何内容，同一次 stage 中的普通文件也不会。

每个文本文件最多读取 1 MiB、总共 32 MiB；更大的文件和二进制文件只按名称判断。`[privacy] staging_guard = false` 会关闭该检查。stage 单个 hunk 或若干行不做检查：你已经在处理那个文件了。

右键点击文件可打开它的菜单：stage 或 unstage、丢弃、复制路径、文件历史和 blame、在外部编辑器、diff 工具或文件管理器中打开（在[设置 > Diff 与编辑器](settings.md#diff-与编辑器)中设置），等等。

## 忽略文件，以及将文件视为二进制或文本

右键点击文件 > **忽略**，可忽略该文件或其扩展名；在文件夹行上选择 **忽略文件夹中的所有文件**。GitPoulpi 会先询问规则写入哪个文件：仓库根目录的 `.gitignore`（默认）、从根目录到该文件所在文件夹每一级的 `.gitignore`，或 `.git/info/exclude`（仅此克隆，永远不会被 commit）。尚不存在的文件标为*新建*，确认时才会创建。写入的行相对于所选文件：`src/a/b.txt` 在 `src/.gitignore` 中写作 `/a/b.txt`。用方向键移动，输入文字可缩小列表，`Enter` 写入，`Esc` 取消。写入 `.gitignore` 可用 `Ctrl+Z` 撤销；`.git/info/exclude` 位于工作区之外，无法撤销。

![询问忽略规则写入哪个文件的列表：仓库的 .gitignore、该文件所在文件夹的 .gitignore 或 .git/info/exclude，每项都附有将写入的那一行](../images/ignore-picker.png)

右键点击文件 > **视为二进制**（git 已将其视为二进制时则是 **视为文本**），会写入一条 `.gitattributes` 规则：`binary` 或 `text diff`。先选择文件本身或所有同扩展名的文件，再用同样的列表选择要写入的 `.gitattributes`（仅此克隆则选 `.git/info/attributes`）。已为同一模式写过的规则会被替换，而不会互相矛盾。文件的 diff 会立即更新，状态栏会说明写入了什么、写到了哪里（若另一条规则仍然优先，还会给出警告）。文件夹行有 **将文件夹中的文件视为** > **二进制** / **文本**。命令面板（`工作区更改：将所选文件视为二进制…`）和命令行（`gitpoulpi treat binary|text <paths>`）中也有同样的功能。文件视图同样遵循这条规则：被视为文本的文件即使含有 NUL 字节，也会显示其各行；git 视为二进制的文件则显示二进制提示。

## 文件列表中的菜单、多选与转换

右侧窗格的文件列表（unstaged 的文件、staged 的文件、某个 commit 修改的文件，以及**查看所有文件**）使用相同的菜单和手势。在文件、文件夹行或列表的空白处右键点击，或在获得焦点的行上按 `.`（`Shift+F10`、菜单键），或在命令面板中运行 `文件：打开当前行或所选文件的菜单`。

- **commit 中的文件**（以及**查看所有文件**中的文件）：**打开 Diff**、**打开文件**、**文件历史**、**文件 Blame**、在编辑器、默认程序或 **打开方式…** 中打开、**在文件夹中显示**、**编辑文件**、**复制文件路径**、**复制文件完整路径**、**忽略**、**视为二进制** 或 **视为文本**、**转换**、**从此 commit 恢复文件**（会先询问；与 `git checkout <commit> -- <file>` 一样，它也会 stage 该文件，`Ctrl+Z` 可带回之前的文件）以及 **删除文件**。当文件不在工作区时，与工作副本有关的行会变灰并显示原因。**与工作目录的 Diff** 会打开该 commit 中的版本与磁盘上文件的 diff。**从文件更改创建补丁** 会为该文件在此 commit 中的更改生成补丁（选区和文件夹行也有此项；在工作更改的**查看所有文件**中，它为该文件在工作区中的更改生成补丁）。
- **文件夹行**：**打开文件夹**、**复制文件路径**、**忽略文件夹中的所有文件**、**将文件夹中的文件视为**、**转换**、**在此文件夹中创建文件**。
- **列表的空白处**：**创建文件** 和 **打开仓库文件夹**。unstaged 列表另有 **全部 stage** 和 **丢弃所有更改…**，staged 列表另有 **全部 unstage**。

![commit 中某个文件的右键菜单：打开 diff、文件历史和 blame，用编辑器或默认程序打开，复制路径，忽略，转换，从该 commit 恢复和删除](../images/commit-file-menu.png)

### 选择多个文件

手势与 Log 相同。**Ctrl+点击** 将一行加入选择或从选择中移除。**Shift+点击**（在会占用 Shift+点击的终端中用 **Alt+点击**）选取从锚点开始的范围。普通点击会重新开始选择，并像以前一样打开 diff。`Esc` 清除选择。在键盘上，`Space` 或 `Insert` 在任何列表中切换当前行（在 unstaged 和 staged 列表中，`Ctrl+Space` 会 stage 所选文件或当前行），`Shift+Up` / `Shift+Down` 扩展范围。文件夹行会选中其下的所有文件，无论是否折叠。选中的行会高亮，状态栏会显示数量。`Ctrl+A` 选中当前列表的所有行。

在选中的行上右键点击（或按 `.`）可打开整个选择的菜单，标题为 **N 个文件**：stage 或 unstage、丢弃、忽略、视为二进制或文本、转换、复制路径、从 commit 恢复、删除。stage、unstage 和恢复各算一次操作；丢弃和删除会先询问并说明文件数量。在选择之外的行上右键点击，会以该行重新开始选择。

![在 unstaged 列表中选中四个文件，并显示标题为“4 个文件”的菜单：stage、丢弃、忽略、视为二进制或文本、转换、复制路径和删除](../images/file-pane-multiselect.png)

### 转换编码、行尾和缩进

**转换** 位于文件、文件夹或选择的菜单中。命令行中的 `gitpoulpi convert` 作用相同。

- **编码**：UTF-8、UTF-8 BOM、UTF-16 LE 和 BE、Windows-1252、ISO-8859-1、ISO-8859-15、Windows-1250、Windows-1251、KOI8-R、Shift_JIS、EUC-JP、GBK、Big5、EUC-KR。
- **行尾**：LF、CRLF 或 CR。
- **缩进**：把行首空白转为空格或制表符，并指定制表符的宽度。

工作区中的文件会在后台用与编辑器状态栏相同的函数重写。不是有效 UTF-8 且没有字节顺序标记（BOM）的文件，会像在编辑器中一样按 Windows-1252 读取。二进制文件、缺失的文件以及目标编码无法表示的文件会被跳过并报告（*已转换 12 个文件，跳过 3 个（3 个二进制文件）*），绝不会被损坏；无法写入的文件会触发失败通知。整个批次是撤销日志中的一条记录：一次 `Ctrl+Z` 就能让所有文件恢复原样。在编辑器中打开的文件会阻止转换，直到关闭它。

## 阅读 diff

点击文件名（或在其上按 `d`），它的 diff 就会取代中间的 Log。`Esc` 或 `q` 带回 Log。

![带有两个 hunk 的 diff，每个 hunk 都有丢弃 Hunk 和 stage Hunk 按钮](../images/diff.png)

其上方的栏用来切换视图：

- **内联**显示统一 diff；**分栏**把前后内容并排放置（`m` 循环切换布局）。
- **Hunk**（`Shift+H`）只显示带一行上下文的已更改 hunk。关闭它即可看到整个文件。
- **WS**（`s`）隐藏仅空白字符不同的更改。
- **换行**（`w`）把长行折行显示，而不是截断。
- **缩略**（`Shift+M`）在滚动条旁显示整个文件的缩略图。点击它即可跳转。所选单词的其他出现位置会被着色，并标记在滚动条上。
- `n` 和 `p`（或栏上的箭头）在 hunk 之间跳转。
- **Auto**（最后一个按钮，`Shift+C`）是读取文件所用的编码。点击它（或按 `Shift+C`，或在命令面板中运行 *Diff：选择编码…*）并从列表中选一个：先是**自动检测**，然后是 UTF-8、UTF-8 BOM、UTF-16 LE 和 BE、Windows-1252、ISO-8859-1、ISO-8859-15、Windows-1250、Windows-1251、KOI8-R、Shift_JIS、EUC-JP、GBK、Big5 和 EUC-KR（输入可搜索列表）。文件的两侧会以精确的字节、按该编码重新读取，所以 Latin-1 文件能显示出重音字符，UTF-16 文件（git 视其为二进制）能显示出它的各行。这个选择会在你退出之前按该文件的路径记住，在此视图和[评审](review.md)中都有效。选择了编码时，hunk 是此视图自己的，所以 stage 和丢弃不可用（选择**自动检测**即可再次 stage）。读取较慢时，片刻之后标题中会显示*正在按 ... 读取*。

![同一个 diff 的分栏布局，前后并排](../images/diff-split.png)

你可以用鼠标选择文本（拖动，或双击一个单词），并用 `y` 或 `Ctrl+C` 复制；只会复制文件文本，绝不包括行号或 `+` 和 `-` 标记。

### stage 一个 hunk 或单独一行

在工作区 diff 上，每个 hunk 标题都有 **stage Hunk** 和 **丢弃 Hunk**（在 staged 一侧是 **unstage Hunk**）。

- `Space` stage 光标所在的 hunk。
- 要 stage 几行，按 `v` 开始选择，用 `j`/`k` 移动，然后按 `Enter`。只有*已更改*的行才算数。
- `x` 丢弃该 hunk，`Shift+X` 丢弃所选的行。两者都会先询问，`Ctrl+Z` 可带回这些更改。
- 在 hunk 上右键点击（或按 `.`）可打开它的菜单：stage、unstage、丢弃、文件历史、blame、复制。

commit 的 diff 是只读的。已更改的图片会显示为图片；参见[图片](image-view.md)。

## 编写 commit

commit 表单位于右侧窗格底部，它的 **摘要** 同时也是 Log 中工作更改行里的那个字段：在其中任何一处输入，另一处都会同步显示。

![填写了 AI 所写消息的 commit 表单：带字符计数的摘要、描述，以及绿色的 commit 按钮](../images/ai-commit-message.png)

- **摘要**（计数器显示 72 个字符还剩多少）和 **描述**。在摘要中按 `Enter` 会移到描述，在描述中按 `Enter` 会增加一行。
- **Commit N 个文件的更改**（绿色按钮）或 `Ctrl+Enter` commit staged 的更改。如果没有任何内容 staged，按钮会这样提示，并且什么也不做。
- **修订上一个 commit**（`Shift+A`）把 staged 的更改和新的消息并入上一次 commit。已经 push 过的 commit 之后需要 force push（with lease）。
- **跳过钩子**（`Shift+H`，或命令面板的 **Commit：切换“跳过钩子”**）会在 commit 时不运行仓库的 `pre-commit` 和 `commit-msg` 钩子。它是每次 commit 单独的选择；**设置 > 仓库专属设置 > Commit** 可让它默认勾选。这只适用于工作区更改：编辑时在右侧窗格选择某个 commit 的文件，会离开编辑器（先询问是否保存）并显示该 commit 中此文件的 diff，编辑器没有的 commit 列也会随之回来。在窄行上，它的标签会缩短为**无钩子**，再缩到只剩复选框（它的提示会说出名称）；它不会消失。
- 在状态窗格中按 `c` 会把光标移到表单。`Esc` 离开。
- `Ctrl+G c` 和 `Ctrl+G Enter`（命令面板中的 **Git：Commit…** 与 **Git：立即 Commit staged 的更改**）是同一个命令，即在任何位置都能按下的 commit 按钮。表单中有摘要且有内容已 staged 时，它会立即 commit，遵循 **修订上一个 commit** 和 **跳过钩子** 的设置，并且不移动焦点。否则它不会 commit 任何内容：它会把光标移到表单的摘要中（窗格被隐藏或折叠时会先显示出来），状态栏会提示用哪个快捷键来 commit（默认是 `Ctrl+Enter`）。没有任何内容 staged 时，它会说明这一点。状态栏中不会输入任何内容。AI 请求仍在运行时，它不会 commit 任何内容，并会说明这一点（`Esc` 取消请求）。`Ctrl+Z` 可以像撤回其他 commit 一样撤回它。
- 按仓库，[设置](settings.md#仓库专属commit)可以让你 **每次 commit 时 push**、让表单以 **消息模板** 开头（并把其中的 `#` 注释从消息里去掉），以及让 merge 改为 **squash**：此时表单会带着 git 的 squash 消息打开。
- 链接了托管集成时，在表单中按 `Alt+R` 会插入一个议题引用；参见[托管平台与议题跟踪器](hosting-integration.md)。

开启 [AI](ai.md) 后，摘要右侧的 **`✦`** 按钮会根据 staged 的更改撰写一条消息（`Ctrl+G i`），而**用 AI 撰写 commits**会把你的工作拆分成若干次 commit，见下文。文本会进入表单供你编辑；在你按下 commit 按钮之前，不会 commit 任何内容。

**用 AI 撰写 commits**位于“修订”行的右端，会随右侧窗格一起缩小而不是消失：先是完整标签，然后是 **AI 撰写**，最后只剩 **`✦`**。在任何宽度下它都保留自己的点击、提示和按键（`Ctrl+G Shift+C`）；让位的是“跳过钩子”的标签（先是**无钩子**，再只剩复选框），然后才轮到它自己缩小。

摘要行右端的 **`✦`**（生成消息）和 **`#`**（插入 issue 引用）按钮也同样缩小：窗格变窄时先去掉边距，再折叠为一个 **`⋯`** 按钮，其菜单列出这两项，所以哪一个都不会消失。

## commit 撰写器：把你的更改拆分成多次 commit

*（需要开启 AI：参见 [AI 辅助](ai.md)。）*按下**用 AI 撰写 commits**（或 `Ctrl+G C`，或右键点击工作更改行）。模型会提出若干次 commit，每次附带其文件；每个已更改的文件恰好属于一次 commit，无效的方案会被退回给模型一次。方案会作为审阅界面打开，此时尚未 commit 任何内容：

![commit 撰写器：四个建议的 commit，每个都可展开查看其文件，带有上移、下移、编辑和丢弃控件，以及重置、取消和创建 4 个 commit 按钮](../images/composer.png)

| 想要 | 做法 |
|---|---|
| 展开或折叠一次 commit | `Left` / `Right`，或点击箭头；`*` 切换全部 |
| 上移或下移一次 commit | 其行上的 `▲` `▼`，或 `Alt+Up` / `Alt+Down`（`K` / `J`） |
| 把文件移到另一次 commit | 把它拖到该 commit 上（从列表中，或从右侧的工作更改列表中）；或在文件上按 `m` 并从下拉列表中选择 commit；或在文件上按 `Alt+Up` / `Alt+Down` |
| 让一个文件单独成为一次 commit | 在文件上按 `n`，或点击 `+ 单独 commit` 按钮 |
| 改写一次 commit | 铅笔图标、`e`、`Enter` 或双击：编辑消息，`Enter` 保留，`Esc` 放弃 |
| 排除一次 commit | `drop`（`d` 或 `Delete`）；其文件保持未 commit。`恢复` 可将其带回 |
| 从头开始 | **重置**（`r`）回到模型的方案 |
| 创建它们 | **创建 N 个 commit**（`Ctrl+S`、`Ctrl+Enter` 或 `a`） |
| 离开 | **取消**或 `Esc`：什么也不做 |

创建 commit 时会先 unstage 所有内容，然后针对每次 commit stage 其文件并 commit，所以你的工作区从不会被触碰。如果某一步失败，它会停下并说明已经创建了多少次 commit。stage 的是整个文件：只 stage 了一部分的文件会被整个 commit。在 *设置 > AI* 中选择撰写器使用的配置和指令；参见 [AI 辅助](ai.md#commit-撰写器)。

## 文件视图、Blame 和文件历史

视图栏有 **文件视图**（文件内容）、**Diff 视图**、**Blame**，以及 [Commit 列](interface-tour.md#commit-列)的 `本文件 │ 全部` 选择器（它的 `本文件` 列表就是文件历史）；文件的右键菜单中也有同样的几项。它们显示的是你所来自的那次 commit 时的文件；如果是从工作更改中打开的，则显示未 commit 的状态（`@ WIP`）。

- **Blame** 和**文件历史**本身就是完整的视图：见 *使用 Log* 中的 [Blame](log-and-history.md#blame) 和 [文件历史](log-and-history.md#文件历史)。在 blame 的 hunk 或历史行上按 `Enter`，会跳到 Log 中的该 commit。
- `y` 或 `Ctrl+C` 复制所选文本（在 Blame 中只复制代码）。
- `Shift+G` 跳到最后一行。`q` 关闭视图。

## 内置编辑器

**编辑**（`e`）会在中间窗格之上的一个小编辑器中打开文件的*工作副本*：语法着色、行号、撤销和重做、查找，以及状态栏（位置、行尾、缩进、语言）。它始终编辑工作副本，即使你是从旧 commit 的 diff 中打开的。

![显示 Rust 文件的编辑器，带有语法高亮和行号；工具栏有保存、撤销、重做和查找](../images/editor.png)

| 按键 | 动作 |
|---|---|
| `Ctrl+S` | 保存（文件先写入临时文件再重命名，因此绝不会只写了一半；行尾、BOM 和末尾换行符都会保留） |
| `Ctrl+Z`、`Ctrl+Y` | 撤销、重做 |
| `Ctrl+F`、`F3` | 查找、查找下一个 |
| `Tab`、`Shift+Tab` | 缩进、取消缩进（遵循文件自身的风格） |
| `Ctrl+A`、`Ctrl+C`、`Ctrl+X`、`Ctrl+V` | 全选、复制、剪切、粘贴 |
| `Esc` | 关闭。有未保存的更改时会询问 **保存**（`s`）、**放弃**（`d`）或 **取消**（`c`） |

二进制文件会被拒绝。制表符宽度、换行符、语法着色、行号和自动换行在 *设置 > Diff 与编辑器* 中设置；参见[设置](settings.md#diff-与编辑器)。

勾选 **跟随选择**（编辑器栏上的复选框，命令面板中的 `编辑器：切换跟随选择`，以及 *设置 > Diff 与编辑器*）后，编辑器打开期间在右侧窗格中选择其他文件，会在编辑器中打开该文件的工作副本，而不是它的 diff 或文件视图。有未保存的更改时，会先询问 **保存**、**放弃** 或 **取消**；取消则停留在当前文件。无法编辑的文件（二进制、过大、已删除）会在其常规视图中打开，状态栏会说明原因。关闭编辑器后会回到所选文件的视图。默认勾选。

右侧的状态栏显示文件的**编码**、**换行符**和**缩进**，然后是光标位置和语法。前三项都是按钮：
每个按钮打开一个下拉菜单（方向键或滚轮，输入文字过滤，`Enter` 选择，`Esc` 关闭）。
命令面板（`Ctrl+P`）可以打开相同的菜单：`编辑器：更改编码`、`编辑器：用编码重新打开`、`编辑器：转换为编码`、`编辑器：转换换行符`、`编辑器：更改缩进`。

![在栏上 UTF-8 标签下展开的编辑器编码菜单：用另一种编码重新打开文件，或转换编码](../images/editor-encoding-menu.png)

![在 LF 标签下展开的编辑器换行符菜单：把文件转换为 LF、CRLF 或 CR](../images/editor-eol-menu.png)

- **编码。** 文件按自己声明的编码打开（BOM：UTF-8 或 UTF-16），否则按 UTF-8，再否则按
  Windows-1252，状态栏会告诉你是哪一种。**用编码重新打开**会用另一种编码重新读取文件的字节
  （有未保存的更改时会先询问）：猜错时使用。**转换为编码**保留文本，改变文件的保存编码
  （UTF-8、UTF-8 BOM、UTF-16 LE 和 BE、Windows-1252、ISO-8859-1 和 -15、Windows-1250、
  Windows-1251、KOI8-R、Shift_JIS、EUC-JP、GBK、Big5、EUC-KR）。保存之前不会写入任何内容，
  撤销可以恢复。如果文本中有目标编码无法表示的字符，状态栏会说明有多少个以及第一个在哪里，
  并且不会转换；保存时同样会拒绝，绝不会写入 `?`。
- **换行符。** 显示 `LF`、`CRLF`、`CR`，文件混用多种时显示 `混合`。菜单把文件中的所有换行
  一次性转换为 LF、CRLF 或 CR，只算一步撤销。你新输入的行使用文件自己的换行符。你的选择优先于
  *换行符* 设置。
- **缩进。** 显示 `空格：4` 或 `制表符：4`。菜单用来选择 `Tab` 和自动缩进插入什么（空格或制表符）、
  大小（1 到 8：缩进步长以及此文件中制表符的宽度），并把每一行开头的空白转换为空格或制表符，
  只算一步撤销。

## Stash、补丁和 Git LFS

- **Stash** 和 **弹出** 在工具栏中；参见[使用 Log](log-and-history.md#分支tag-和-引用侧栏)。
- **创建补丁** 和 **应用补丁** 在 commit 菜单和引用菜单中；CLI 有 `gitpoulpi patch`。
- 用 Git LFS 存储的大文件，在你 pull 其内容之前显示为小的指针文件；参见 [Git LFS](lfs.md)。

## 通过命令行

本页的一切都可以脚本化：

```
gitpoulpi --json status
gitpoulpi stage --file src/lib.rs
gitpoulpi stage --all --allow-sensitive   # 在下面的拒绝之后
gitpoulpi --json diff
gitpoulpi commit -m "feat: add the helper"
gitpoulpi discard --file src/lib.rs --hunk 1
```

`gitpoulpi stage` 以同样的方式拒绝敏感文件：不 stage 任何内容，退出码为 2，`--json` 输出 `{error: "sensitive_files", files: [{path, reason}]}`（一条规则和遮蔽后的预览，绝不输出值）。`--allow-sensitive` 就是重试。MCP 代理会得到同样的拒绝，并且除非 `[mcp] allow_sensitive_staging = true`，否则无法传入 `allow_sensitive`：它会被告知来问你。

参见 [CLI 参考](cli-reference.md)。
