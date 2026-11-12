[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · **简体中文**

# 选择终端

GitPoulpi 可以在任何支持鼠标的终端中运行，但在能绘制图片、显示真彩色并发送完整键盘按键的终端里，外观和手感最好。本页说明每个终端在每个操作系统上能给你什么，以及要更改哪项设置。

**如何阅读。** 这些表格来自 **2026 年 11 月**的一轮测试：每个终端都用原始转义序列驱动，在能运行 GitPoulpi 的地方还用 GitPoulpi 本身驱动，系统为 Linux、Windows 11 和 macOS 15。版本见[下文](#已测试的版本)。每个单元格都是以下之一：

- **可用**；
- **部分可用**（并说明缺什么）；
- **不可用**（并说明原因：*终端*、*操作系统层*，还是 *GitPoulpi*）；
- **不适用**（该终端在该操作系统上不存在）；
- **未测量**（并说明原因）。

多路复用器（tmux、zellij、screen、herdr、psmux、终端窗格）有自己的页面：[多路复用器](multiplexers.md)。

## 最佳选择

| 操作系统 | 最佳选择（按顺序） | 原因 |
|---|---|---|
| **Windows** | **Windows Terminal**，其次 **WezTerm** 和 **Rio** | Windows Terminal 把图和头像绘制为 sixel 图片，位置准确，并且是启动器的默认终端。WezTerm 和 Rio 把它们绘制为 iTerm2 图片，自 2026 年 11 月的 ConPTY 修复起位置准确（见 [Windows](#windows)）。 |
| **Linux**（X11、Wayland、Hyprland、Omarchy） | **kitty**、**Ghostty**、**WezTerm**、**foot**、**Rio** | kitty 和 Ghostty 提供图片、指针形状和完整键盘。foot 绘制 sixel 图片，并有完整键盘和指针形状。 |
| **macOS** | **iTerm2**，其次 **kitty** 和 **Ghostty** | iTerm2 3.7.7 通过 kitty 协议绘制图片。Terminal.app 可以用，但既没有图片，也没有真彩色。kitty 和 Ghostty 的表现与它们在 Linux 上一致（未在 macOS 上测量）。 |

在任何操作系统上，**screen** 和 **zellij** 都会向 GitPoulpi 隐藏终端的图片协议，所以它绘制的是字符；tmux 会透传 sixel。见[多路复用器](multiplexers.md)。

## GitPoulpi 用到什么

- **图片**（图列、作者头像、图像 diff 和文件视图）：带 Unicode 占位符的 **kitty 图形协议**、**iTerm2 内联图像**（`OSC 1337`）或 **sixel**。检测默认拒绝，依据 `TERM`、`TERM_PROGRAM`、`KITTY_WINDOW_ID`、`GHOSTTY_RESOURCES_DIR`、`WEZTERM_EXECUTABLE`、`WT_SESSION`、`KONSOLE_VERSION` 以及多路复用器的变量。当环境变量没有指明是哪个终端时（以 `-ti vt340` 启动的 xterm、开启图像的 VS Code 终端、任何通过 SSH 到达的终端，因为 `TERM_PROGRAM` 不会被转发），由终端自己的应答决定：GitPoulpi 在启动时询问一次（见下文“启动时的查询”），并采用终端的 XTVERSION 名称（kitty、Ghostty、WezTerm、iTerm2、mintty、foot），或设备属性中的 sixel。在多路复用器内部会忽略这些应答，因为不论 tmux 后面是什么，它都声称支持 sixel。
- **哪里用哪种协议。** kitty、Ghostty 和 iTerm2 3.7.7 及更高版本：kitty 协议。WezTerm 和 Rio：iTerm2 协议，混合 PNG 的透明度。mintty 和 3.7.7 之前的 iTerm2：iTerm2 协议，背景不透明。Windows Terminal、foot、Tabby、Konsole 和 mlterm：**sixel**。其他一切：字符，头像变成首字母徽章。`ui.image_protocol` 可强制指定一种。
- **iTerm2 3.7.7。** 每一张 `OSC 1337` 内联图像都会弹出一个模态的“Allow Terminal-Initiated Display?”对话框，在你勾选“Remember my choice”之前会阻塞窗口。通过 kitty 协议（包括 Unicode 占位符）发送的同样图片则不会提问就绘制，所以 GitPoulpi 在那里使用 kitty 协议。`ui.image_protocol = "iterm2"` 仍然会给出内联图像。
- **Sixel** 用于绘制**图列**（泳道、节点，以及设置 `ui.avatars = "graph"` 时 commit 圆圈内的头像），其他位置用于绘制**头像**（Log、commit 详情、文件历史、Blame、图像视图）。sixel 没有 alpha 通道、没有 ID，也不能缩放，所以图片恰好是 `列数 x 单元格宽度` 乘 `行数 x 单元格高度` 像素，只在有内容的地方绘制，位于保留背景的空白单元格之上；只有泳道、滚动位置或缩放改变时才重新编码。sixel 没有删除操作：GitPoulpi 通过在其单元格上写入文本来移除它。sixel 需要单元格的真实像素大小：Windows Terminal、Tabby 和 mintty 在启动时和缩放后应答一次 `CSI 16 t`（不应答也绝不阻塞）；其他终端取自窗口大小或 `GITPOULPI_TERMINAL_CELL_PX=10x20`。能绘制 sixel 但没有声明的终端需要设置 `ui.image_protocol = "sixel"`。1.22 之前的 Windows Terminal 不能绘制 sixel；请在其中设置 `ui.image_protocol = "none"`。
- **真彩色**：主题是 RGB。`ui.color_depth = "auto"` 在 `TERM` 为 `*-256color` 且没有 `COLORTERM=truecolor` 时（以及在没有真彩色的 Terminal.app 中）降到 256 色，在 Linux 控制台降到 16 色。
- **鼠标**：带按键、拖动和移动的 SGR 鼠标报告（悬停反馈、拖放、右键点击、双击）。
- **启动时的查询。** 四个问题在一次写入中发出：kitty 键盘标志（`CSI ? u`）、背景颜色（`OSC 11`，只用于 `ui.theme = system`）、终端的名称（XTVERSION，`CSI > 0 q`）和它的设备属性（`CSI c`，放在最后：每个终端都会应答它，所以它结束读取）。读取是一个带 100 ms 期限的 `poll` 循环，在键盘读取器启动之前完成。不作任何应答的终端只会让你等一次 100 ms，其余一切仍按环境变量所说的处理。仅限 Unix：在 Windows 上控制台自己报告按键，由环境变量决定。应答记录在活动日志的环境报告中（`start-up query:` 行）。
- **键盘**：启动时会请求 **kitty 键盘协议**（只用 disambiguate 标志）。它让 `Ctrl+,`、`Ctrl+Enter`、`Ctrl+Shift+N` 和 `Ctrl+Shift+Z` 能够送达。没有它时，保留传统按键（`Ctrl+Y` 重做，`Alt` 变体替代部分键序）。参见[键盘快捷键](keybindings.md#终端无法发送的按键以及-kitty-键盘协议)。
- **剪贴板**：每个复制操作都遵循同一个策略。在本地，写入 GitPoulpi 所在机器的**系统剪贴板**，并在已知终端会响应时同时向终端发送 `OSC 52`（kitty、WezTerm、Ghostty、foot、Alacritty、Windows Terminal、Rio、Tabby、iTerm2）；系统剪贴板失败时，`OSC 52` 也是后备方案。**通过 SSH，或没有显示器时**（Linux 控制台、容器），只发送 `OSC 52`，在 tmux 内部还会同时发送包装为透传的版本和原样版本。只有系统剪贴板接收了文本时，状态栏才会说“已复制”；发给终端的则说“已将……发送到终端剪贴板（OSC 52，未确认）”，因为没有任何东西能确认终端是否照办。活动日志的环境报告在 `clipboard:` 行中给出所走的途径。
- **鼠标指针形状**（分隔线上的调整大小指针、可点击项上的手形、文本框上的 I 形）：`OSC 22`，在终端实现了它的地方是额外的加分项。GitPoulpi 总是发送它（Linux 控制台以及 screen、zellij 和 herdr 之后除外，它们会吞掉或打印它；在 tmux 之后则包装为透传）。在 `TERM_PROGRAM=mintty` 下，它发送 mintty 自己的光标名称（`hand`、`ibeam`、`sizewe` ……），这是 mintty 唯一会读取的。**Rio 在任何操作系统上都没有实现 `OSC 22`**：它的源码解析该序列后将其丢弃，所以两边的任何设置都改变不了这一点。
- **文本插入符**：`ui.text_cursor = "terminal"`（默认）时，终端自己的光标会放在获得焦点的文本框的插入点处，呈闪烁的竖线（`DECSCUSR`），在其他位置隐藏；退出时恢复你自己的光标样式。`drawn` 则改为绘制 GitPoulpi 自己的插入符。
- **焦点事件**已启用（窗口失去焦点时工具提示会关闭）。**括号粘贴**在启动时请求，并在每一种退出方式上释放（Unix），所以多行粘贴作为一次粘贴到达：其中的换行不会提交字段，其中的字母也不会触发快捷键。在 Windows 上控制台没有粘贴括号；GitPoulpi 改用什么办法，见 [Windows](#windows)。GitPoulpi 不请求 OSC 8 超链接或 undercurl。仅在图片路径上，每一帧都包在同步更新里（模式 2026，不认识它的终端会忽略），这样图片不会在弹出窗口上方闪烁。
- **表情符号和符号**：GitPoulpi 只使用默认表情呈现的字形，或单宽度的 BMP 形状，因为终端和字体对变体选择符的处理并不一致。

## 能力表

行是终端，列是操作系统。单元格里的“GitPoulpi”表示在那里运行了 GitPoulpi 本身；“探测”表示只运行了原始转义序列。**不适用**的单元格：kitty 没有原生 Windows 版本（发布版本提供 Linux 和 macOS）；Ghostty 没有 Windows 版本（只有它的 libghostty 库是跨平台的）；foot 是 Wayland 终端（仅限 Linux）；xterm 是 X11 客户端（Linux，以及通过 XQuartz 的 macOS）；iTerm2 和 Terminal.app 仅限 macOS；Windows Terminal、conhost 和 mintty 仅限 Windows（mintty 也可用于 Cygwin 和 MSYS2）。

**未在 macOS 上测量：kitty、Ghostty、WezTerm、Rio 和 Warp。** 它们需要 GPU（Metal 或 OpenGL 3.3）。它们在 Linux 上的结果是同一程序行为的参照；依赖某个 macOS 单元格之前，请在带 GPU 的 Mac 上重测。

### 图片：GitPoulpi 选哪种协议，是否显示

| 终端 | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **可用**：kitty 协议（GitPoulpi） | 不适用 | 未测量（需要 GPU） |
| **Ghostty** | **可用**：kitty 协议（GitPoulpi） | 不适用 | 未测量（需要 GPU） |
| **WezTerm** | **可用**：带透明度的 iTerm2 协议（GitPoulpi）。占位符会被压扁，所以不使用 kitty 协议 | **可用**：带透明度的 iTerm2 协议，图和头像在各自的行上（官方 20240203 和带上游 PR #8092 的构建）。官方 20240203 在 GitPoulpi 于图片周围显示光标之前，会把图片放到错误的行上，见“ConPTY 的限制”。 | 未测量（需要 GPU） |
| **foot** | **可用**：带透明度的 sixel（GitPoulpi） | 不适用 | 不适用 |
| **Rio** | **可用**：带透明度的 iTerm2 协议（GitPoulpi） | **可用**：带透明度的 iTerm2 协议，在试过的每个字体大小下都精确对齐单元格（0.5.28；GitPoulpi）。sixel 和 kitty 都到不了（ConPTY 会丢弃它们），所以没有可选的 | 未测量（需要 GPU） |
| **Alacritty** | 文本：**可用**（终端不支持图像） | 文本：**部分可用**（没有图片；图像 diff 用方块） | 文本：**可用** |
| **Tabby** | **可用**：带透明度的 sixel（探测；未在 Linux 的 Tabby 中运行 GitPoulpi） | **可用**：sixel，位置对齐（GitPoulpi） | **可用**：sixel，保留透明度（GitPoulpi） |
| **Warp** | 未测量（它需要 XWayland，并且会错误解码合成的按键，所以什么也输入不了） | **不可用**（终端：没有 sixel、iTerm2 或 kitty 图像）；GitPoulpi 未测量（无法从 Warp 的输入框启动） | 未测量（需要 GPU） |
| **xterm** | 探测：用 `-ti vt340` 时 sixel **可用**；GitPoulpi 根据设备属性选用它（启动时的查询是在这次运行之后才有的），或设置 `ui.image_protocol = "sixel"` | 不适用 | 文本：**可用**；sixel 能绘制（探测），需要 `ui.image_protocol = "sixel"` |
| **iTerm2** | 不适用 | 不适用 | **可用**：3.7.7 及更高版本使用 kitty 协议，没有对话框（GitPoulpi）。旧版本得到背景不透明的内联图像。sixel 能绘制，但会把未设置的像素画成不透明，所以 `ui.image_protocol = "sixel"` 会得到一个实心矩形 |
| **Terminal.app** | 不适用 | 不适用 | 文本：**可用**（没有图片） |
| **Windows Terminal** | 不适用 | **可用**：sixel（GitPoulpi，1.25） | 不适用 |
| **conhost** | 不适用 | 文本：**部分可用**（字符图，半块字符的图像 diff） | 不适用 |
| **mintty** | 不适用 | **不可用**（ConPTY 之后的终端：根本不绘制任何图片，即使是在显示光标的情况下放置的普通 `OSC 1337` 图片也不行；图列保持空白；文本和命令面板正常） | 不适用 |

### 鼠标指针形状（OSC 22）

| 终端 | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **可用** | 不适用 | 未测量（需要 GPU） |
| **Ghostty** | **可用** | 不适用 | 未测量（需要 GPU） |
| **WezTerm** | **不可用**（终端：在测试的构建中未实现） | 官方：**不可用**（终端）。带上游 WezTerm PR #8092（它加入了指针形状）的构建：**可用**（手形、I 形、调整大小箭头） | 未测量（需要 GPU） |
| **foot** | **可用** | 不适用 | 不适用 |
| **Rio** | **不可用**（终端：解析后丢弃） | **不可用**（终端，原因相同；不是 ConPTY） | **不可用**（终端，同一份源码；只在 Linux 和 Windows 上运行过） |
| **Alacritty** | **不可用**（终端） | **不可用**（终端） | **不可用**（终端） |
| **Tabby** | **不可用**（终端：xterm.js） | **不可用**（终端） | **不可用**（终端） |
| **Warp** | 未测量 | **不可用**（终端） | 未测量 |
| **xterm** | **不可用**（终端：不认识 CSS 名称） | 不适用 | 未测量（指针在窗口外） |
| **iTerm2** | 不适用 | 不适用 | **不可用**（终端） |
| **Terminal.app** | 不适用 | 不适用 | **不可用**（终端） |
| **Windows Terminal** | 不适用 | **不可用**（终端） | 不适用 |
| **conhost** | 不适用 | **不可用**（终端） | 不适用 |
| **mintty** | 不适用 | 使用 mintty 自己的名称时**可用**；GitPoulpi 会发送它们 | 不适用 |

单元格写着“不可用”的地方，鼠标下的分隔线或按钮仍会亮起。

### 键盘协议

GitPoulpi 在 Linux 和 macOS 上请求 kitty 键盘协议。在 Windows 上不请求：控制台自己报告修饰键，下面的键序受限于终端留给自己的按键。

| 终端 | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **可用** | 不适用 | 未测量（需要 GPU） |
| **Ghostty** | **可用** | 不适用 | 未测量（需要 GPU） |
| **WezTerm** | **部分可用**：在 `wezterm.lua` 中设置 `enable_kitty_keyboard = true`（默认 `false`）之前没有应答 | **部分可用**：WezTerm 保留 `Ctrl+Shift+N`（打开它的窗口）；`Ctrl+,` 不会送达（官方构建）。PR #8092 构建两个键序都不保留，但 `Ctrl+,` 仍然打不开设置（通过 ConPTY 没有 kitty 键盘应答） | 未测量（需要 GPU） |
| **foot** | **可用** | 不适用 | 不适用 |
| **Rio** | **可用**（默认开启） | **部分可用**：Rio 保留 `Ctrl+Shift+N`（打开它的窗口） | 未测量（需要 GPU） |
| **Alacritty** | **可用**（0.13 及更高版本） | **部分可用**：`Ctrl+Shift+N` 以 `Ctrl+N` 到达，`Ctrl+,` 不会送达 | **可用** |
| **Tabby** | **不可用**（终端：没有应答；它自己保留 `Ctrl+Shift+N` 和 `Ctrl+,`） | **部分可用**：Tabby 保留 `Ctrl+Shift+N` 和 `Ctrl+,` | **不可用**（终端：没有应答）；有些键序没有送达 |
| **Warp** | 未测量 | 未测量 | 未测量（需要 GPU） |
| **xterm** | **不可用**（终端） | 不适用 | **不可用**（终端） |
| **iTerm2** | 不适用 | 不适用 | **可用**（3.7.7；若不行，请检查 *Profiles > Keys*） |
| **Terminal.app** | 不适用 | 不适用 | **不可用**（终端：没有应答；传统按键） |
| **Windows Terminal** | 不适用 | **部分可用**：WT 保留 `Ctrl+Shift+N`（新窗口）和 `Ctrl+,`（它自己的设置） | 不适用 |
| **conhost** | 不适用 | 未测量 | 不适用 |
| **mintty** | 不适用 | **部分可用**：两个键序都没有可见效果 | 不适用 |

键序没有送达时，请使用 `Alt` 变体（[键盘快捷键](keybindings.md#终端无法发送的按键以及-kitty-键盘协议)），或在 *设置 > 键盘与鼠标* 中重新绑定该键序。

### 剪贴板（复制操作）

有本地显示器时，复制总是使用系统剪贴板，所以这些单元格描述的是终端的 `OSC 52` 路径，它在 SSH、容器中以及作为后备时才有意义。每一次使用系统剪贴板的 GitPoulpi 复制测试都成功了。

| 终端 | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **可用** | 不适用 | 未测量（需要 GPU） |
| **Ghostty** | **可用**（在 tmux、zellij、herdr 之后见到）；单独使用：未测量（写入剪贴板需要键盘焦点，测试没有给它） | 不适用 | 未测量（需要 GPU） |
| **WezTerm** | **可用** | **可用** | 未测量（需要 GPU） |
| **foot** | 未测量（写入需要键盘焦点，测试没有给它） | 不适用 | 不适用 |
| **Rio** | 未测量（原因相同） | **可用** | 未测量（需要 GPU） |
| **Alacritty** | 未测量（原因相同） | **不可用**（默认配置下剪贴板没变；未深入调查） | **可用** |
| **Tabby** | **可用** | **可用** | **可用** |
| **Warp** | 未测量 | **不可用**（剪贴板没变） | 未测量（需要 GPU） |
| **xterm** | 未测量 | 不适用 | **不可用**（剪贴板没变） |
| **iTerm2** | 不适用 | 不适用 | 在开启 *Settings > General > Selection > Applications in terminal may access clipboard* 之前**不可用** |
| **Terminal.app** | 不适用 | 不适用 | **不可用**（终端） |
| **Windows Terminal** | 不适用 | **可用**（获得焦点时） | 不适用 |
| **conhost** | 不适用 | 未测量（没有运行探测） | 不适用 |
| **mintty** | 不适用 | **不可用**（默认设置下剪贴板没变） | 不适用 |

### 粘贴

在 Linux 和 macOS 上，GitPoulpi 请求括号粘贴，所以多行粘贴就是一次粘贴。这没有逐个终端运行过（每个终端都是**未测量**）。在 Windows 上，控制台没有粘贴括号，所以不预先询问的终端会把文本作为按键送出，每个换行都会变成一次 `Enter`。因此 GitPoulpi 把 30 ms 内相继到达的按键（中间有一个 `Enter`，或有八个及以上字符）当作一次粘贴。按住的按键和手动输入的“abc、`Enter`”不算粘贴。

| 终端 | Windows（把两行文本粘贴到新分支提示中） |
|---|---|
| **Windows Terminal** | **可用**：WT 先问“粘贴多行吗？”（它自己的警告），之后不会创建分支 |
| **WezTerm**、**Rio**、**Alacritty**、**Tabby**、**mintty** | **可用**（没有创建分支） |
| **Warp** | 未测量（无法在其中启动 GitPoulpi） |
| **conhost** | 未测量（测试发出的 `Ctrl+V` 和 `Shift+Insert` 没有粘贴进经典控制台） |

### 表情符号和图标宽度

| 终端 | Linux | Windows | macOS |
|---|---|---|---|
| **kitty**、**Ghostty**、**WezTerm**、**foot**、**Rio**、**Alacritty** | **可用**（宽单元格对齐；kitty 把齿轮 U+2699 画成空白，这是已知的字体问题） | WezTerm、Rio、Tabby、Windows Terminal：**可用**。Alacritty：**部分可用**（表情符号是单色轮廓） | Alacritty、Tabby：**可用**。其他：未测量（需要 GPU） |
| **Tabby** | **可用** | **可用** | **可用** |
| **Warp** | 未测量 | **可用** | 未测量 |
| **xterm** | **部分可用**（方框：字体问题） | 不适用 | **部分可用**（方框：字体问题） |
| **iTerm2**、**Terminal.app** | 不适用 | 不适用 | **可用** |
| **conhost** | 不适用 | **部分可用**：绘制彩色表情符号，宽单元格对齐 | 不适用 |
| **mintty** | 不适用 | **可用** | 不适用 |

### 已测试的版本

kitty 0.49.2（macOS 上的 0.50.0 无法启动）、Ghostty 1.4.0、Linux 上的 WezTerm 20260716 和 Windows 上的 20240203（最后一个带标签的发布版本）、foot 1.28.0、Rio 0.5.28（Linux、Windows）和 0.5.29（macOS）、Alacritty 0.17.0、Tabby 1.0.238、Warp 0.2026.11.04、xterm 411（Linux）和 378（XQuartz）、iTerm2 3.7.7、Terminal.app 455.1（macOS 15.8）、Windows Terminal 1.25.2733、mintty 3.8.3。终端更新很快：比这些更新的版本可能有所不同。

## 各终端详情

### Windows

- **Windows Terminal。** 启动器的默认终端，也是 Windows 上图片支持最好的：sixel 图和头像、OSC 52、良好的表情符号，并且会应答单元格大小查询。在 *设置 > 配置文件 > 默认值 > 外观 > 字体* 中选一款 Nerd Font。鼠标被捕获时，按住 `Shift` 可选择文本。它保留 `Ctrl+Shift+N` 和 `Ctrl+,`：请使用 `Alt` 变体或重新绑定。指针形状不会改变。
- **WezTerm。** 在 `wezterm.lua` 中设置 `config.enable_kitty_keyboard = true`（Linux 和 macOS 构建）和带表情符号回退的字体，例如 `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`。图片使用 iTerm2 协议，无需任何设置：最后一个带标签的发布版本 20240203-110809（2024 年 2 月）能把它们放在正确的行上（2026 年 11 月用当前的 GitPoulpi 复核过；在下面所述的 ConPTY 修复之前则不行）。上游 WezTerm PR #8092 加入了指针形状。
- **Rio**（MSI，需要 UAC）。图片（图、头像、图像 diff）使用带透明度的 iTerm2 协议，并且恰好落在它们的单元格上：在 0.5.28 上以默认字体以及 18 和 22.5 px 测量（单元格 8x16、11x21 和 13x26 像素，始终是整像素），一张 20x10 单元格和一张 4x27 单元格的图片都精确对齐单元格。无需任何设置。伪控制台会吞掉单元格大小查询，所以 GitPoulpi 假定 Rio 的单元格高度是宽度的两倍（这三种大小下都成立）；字体比例不同时，请设置 `GITPOULPI_TERMINAL_CELL_PX=WxH`（例如 `10x24`），以得到圆形的 commit 标记。Rio 在任何操作系统上都没有指针形状。
- **Tabby。** sixel 图和头像，复制和粘贴可用。Tabby 自己保留 `Ctrl+Shift+N` 和 `Ctrl+,`。
- **Alacritty。** 需要 OpenGL 3.3。字符图；默认配置下没有开启通过 `OSC 52` 的复制。
- **mintty**（Git Bash、MSYS2、Cygwin）。指针形状可用（mintty 自己的名称）。图片不显示（图列保持空白；即使由普通探测放置，mintty 3.8.3 在 ConPTY 之后也不绘制任何 `OSC 1337` 图片），并且那里它不应答单元格大小查询：请设置 `ui.graph_renderer = "text"`。
- **Warp。** 未用 GitPoulpi 测量：无法从 Warp 的输入框启动它。该终端没有 sixel、iTerm2 或 kitty 图像。
- **conhost、cmd、PowerShell 窗口。** 字符图，半块字符的图像 diff。默认情况下，从这样的窗口启动的 GitPoulpi 会在你选择的终端中重新打开自己；见“重新启动规则”。

**ConPTY 的限制。** Windows 让每个终端的程序都运行在伪控制台（ConPTY）之后。捆绑了较新版本的 Windows Terminal 和 WezTerm 构建会透传图片序列；Windows 内置的那个（Rio 和 mintty 使用），以及 WezTerm 20240203 捆绑的那个则不会。丢失的是：DCS（sixel）和 APC（kitty 图形）序列被丢弃，所以只有 iTerm2 图片（`OSC 1337`）能到达，这就是 Rio 和 WezTerm 使用它们的原因；`CSI 16 t`（单元格大小）只有 Windows Terminal 和 Tabby 会应答。已修复的是：`OSC 1337` 会立即穿过 ConPTY，并落在 ConPTY 最后绘制的光标处，而 ConPTY 只在光标可见时才绘制它。GitPoulpi 一直把光标隐藏，所以它的图片落在了过期的行上（图画在页脚上，滚动后屏幕撕裂）。现在它在 Windows 上写出每张图片时，会在同一次写入中围绕它显示光标，然后再隐藏；Rio 和 WezTerm 20240203 把每张图片都定位在它的单元格上（`pty_conpty_picture_anchor` 会把 ConPTY 之后的光标移动读回来）。在 Windows 之外或对 sixel 没有任何变化。

**Windows 粘贴。** 见上面的“粘贴”表：GitPoulpi 把一阵按键合并为一次粘贴，所以在测试过的终端中，多行粘贴绝不会提交提示。

**重新启动规则。** 双击 exe 会在它自己的经典控制台窗口（conhost）中启动。当 `ui.launch_terminal` 指定了另一个终端（默认：Windows Terminal）时，GitPoulpi 会在那里重新打开自己，第一个窗口随之关闭（`ui.relaunch_in_terminal`）。它**只在**独自处于经典控制台窗口中时才这样做。直接运行 exe 的终端（`wt.exe gitpoulpi.exe`、WezTerm、Rio、Alacritty `-e`、Tabby `run`）就是你想在的地方，不会重新启动任何东西；从 shell 中启动时它并不是独自一个，所以同样什么都不会发生。用 `ui.relaunch_in_terminal = false` 可关闭它。

### Linux

- **kitty。** 无需任何设置，一切正常。`Shift+点击` 是 kitty 自己的，所以请用 `Alt+点击` 扩展选择（[鼠标](mouse-and-gestures.md)）。
- **Ghostty。** 无需任何设置，一切正常。如果图标显示为空洞，请在 `font-family` 中加入表情符号字体或 Nerd Font（重复该键即可添加回退字体）。
- **WezTerm。** 通过 iTerm2 协议显示图片，混合透明度。设置 `enable_kitty_keyboard = true` 以获得完整键序。没有指针形状。
- **foot。** sixel 图和头像，指针形状，完整键盘，速度快。在 `foot.ini` 中设置 `font=JetBrainsMono Nerd Font:size=11`。通过 `OSC 52` 的复制由 `security.osc52` 控制。
- **Rio。** 通过 iTerm2 协议显示带透明度的图片，完整键盘。没有指针形状（Rio 会丢弃 `OSC 22`）。
- **Alacritty。** 字符图，自 0.13 起支持 kitty 键盘。不支持图像。
- **Tabby。** sixel 图片；没有 kitty 键盘，没有指针形状（Electron、xterm.js）。它保留 `Ctrl+Shift+N` 和 `Ctrl+,`。
- **xterm。** 用 `xterm -u8 -ti vt340` 启动以使用 sixel（绘制方框需要 UTF-8）。用方框代替表情符号：这是字体问题。
- **Warp。** 未测量：它需要 XWayland，并且在测试中错误解码了合成的按键。
- **Konsole、GNOME Terminal、Ptyxis（VTE）。** 不在 2026 年 11 月这一轮之内。使用字符和传统按键：请使用 `Alt` 变体，并用 `Ctrl+Y` 重做。VTE 忽略 `OSC 52`，所以通过 SSH 的复制到不了剪贴板。输入法（IBus、Fcitx）可能会抢占 `Ctrl+Space` 暂存切换键；请在 *设置 > 键盘与鼠标* 中重新绑定。Konsole 被识别为支持 sixel，依据是它自己的文档。
- **Wayland、Hyprland、Omarchy。** 鼠标指针提示（`OSC 22`）在 kitty、Ghostty 和 foot 中有效。Omarchy 的启动器条目通过 `xdg-terminal-exec` 运行。

### macOS

- **iTerm2。** 3.7.7 及更高版本通过 kitty 协议绘制图片，不会提问。有帮助的设置：
  - *Settings > General > Selection > Applications in terminal may access clipboard*：通过 `OSC 52` 复制所必需（否则 iTerm2 会显示“the terminal attempted to access the clipboard but it was denied”）。
  - *Profiles > Keys > Left Option key: Esc+*：`Alt` 键序所必需。
  - 如果你设置了 `ui.image_protocol = "iterm2"`（或使用早于 3.7.7 的版本），请在“Allow Terminal-Initiated Display?”对话框中勾选“Remember my choice”。此时图片带有不透明的背景（如果你的背景与主题不同，会看到一个矩形；`ui.graph_renderer = "text"` 可以去掉它）。
  - kitty 键盘协议默认开启（3.6.9 及更高版本）。
- **Terminal.app。** 只有 256 色：GitPoulpi 会自己选择 `ui.color_depth = 256`，主题看起来更平淡。没有图片，没有 kitty 键盘，也无法通过 `OSC 52` 访问剪贴板。要使用 `Alt` 键序，请勾选 *Use Option as Meta key*。
- **Alacritty、Tabby。** Alacritty：字符图，kitty 键盘，`OSC 52` 复制；要使用 `Alt` 键序，请设置 `option_as_alt = "Both"`。Tabby：sixel 图和头像。
- **Option 作为 Meta。** 除非终端把 Option 变成 Meta，否则它会组合字符（Option+B 输入积分号）：iTerm2 的 *Left Option key: Esc+*、Terminal.app 的 *Use Option as Meta key*、Alacritty 的 `option_as_alt`。GitPoulpi 的 `Alt` 键序需要它，而且在任何地方它都不是默认开启的。
- **kitty、Ghostty、WezTerm、Rio、Warp**：未在 macOS 上测量（它们需要 GPU）。预期与 Linux 行为相同。
- **xterm**（XQuartz）：`xterm -u8 -ti vt340`；sixel 需要 `ui.image_protocol = "sixel"`。

### VS Code 和 Warp

VS Code 终端（`TERM_PROGRAM=vscode`）不在这一轮测试之内。开启 `terminal.integrated.enableImages` 时，它会根据设备属性被识别，否则 GitPoulpi 绘制字符；`ui.image_protocol = "sixel"` 可强制显示图片。Warp：见上面各操作系统的说明。

## 多路复用器

tmux、zellij、screen、herdr、psmux 和终端自己的窗格位于 GitPoulpi 和终端之间，会改变能通过的内容。简而言之：tmux 透传 sixel，并在 `allow-passthrough on` 时透传指针形状和 kitty 图片；zellij 和 screen 既不透传图片也不透传指针形状；herdr 透传 kitty 图片。所有细节以及要添加的配置都在[多路复用器](multiplexers.md)中。

## 缺失时会怎样降级

| 缺失 | 你会得到什么 |
|---|---|
| 图片协议 | 字符图。头像变成带颜色的首字母徽章（一个字母），无论 `ui.avatar_fallback` 怎么设：挤进文本单元格里的 identicon 根本认不出来。图像 diff 和文件视图使用带颜色的半块字符，或信息卡。 |
| 真彩色 | 颜色被映射到 256 色或 16 色调色板（`ui.color_depth`）。 |
| Kitty 键盘协议 | 传统按键；部分键序会以另一个按键到达。设置中会给出警告，你可以重新绑定，或使用 `Alt` 变体。 |
| OSC 52 | 在本地没有变化（使用系统剪贴板）。通过 SSH 或没有显示器时，复制操作不会向剪贴板放入任何内容：用鼠标选择（按住 `Shift`），再使用终端自己的复制。 |
| OSC 22 | 指针不变；鼠标下的分隔线或按钮仍会高亮。 |
| 表情符号字体 | 图标位置出现方框或空洞；图和文字仍然正常。请安装 Nerd Font 或表情符号字体，并把它列为回退字体。 |

## 需要了解的设置

| 设置 | 用途 |
|---|---|
| `ui.image_protocol` | `auto`（默认）、`kitty`、`iterm2`、`sixel`、`none`。如果在应有图片的地方看到乱码，请设为 `none`。需要重启。 |
| `ui.graph_renderer` | `auto`、`image`、`text`。`text` 始终正确（在 ConPTY 的限制适用的地方使用它）。需要重启。 |
| `ui.color_depth` | `auto`、`truecolor`、`256`、`16`。 |
| `ui.kitty_keyboard` | `auto` 或 `off`，用于终端或多路复用器行为异常时。需要重启。 |
| `ui.text_cursor` | `terminal`（默认）或 `drawn`。 |
| `ui.mouse_enabled` | 关闭后鼠标交还给终端。需要重启。 |
| `ui.launch_terminal`、`ui.relaunch_in_terminal` | 系统快捷方式所打开的终端，以及双击是否在那里重新打开 GitPoulpi（[设置](settings.md)）。 |

另见[多路复用器](multiplexers.md)、[图片](image-view.md)、[键盘快捷键](keybindings.md)和[故障排除](troubleshooting.md)。

## 来源

文档于 2026 年 10 月核实，行为于 2026 年 11 月测试。除非注明，均为一手来源。

- kitty：[更新日志](https://sw.kovidgoyal.net/kitty/changelog/)、[指针形状](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty：[OSC 22](https://ghostty.org/docs/vt/osc/22)、[OSC 52](https://ghostty.org/docs/vt/osc/52)、[下载](https://ghostty.org/download)
- WezTerm：[enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html)、[转义序列](https://wezterm.org/escape-sequences.html)、[OSC 22 拉取请求](https://github.com/wezterm/wezterm/pull/6292)、[发布版本](https://github.com/wezterm/wezterm/releases)
- foot：[更新日志](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty：[更新日志](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md)、[发布版本](https://github.com/alacritty/alacritty/releases)
- Konsole：[OSC 52 合并请求](https://invent.kde.org/utilities/konsole/-/merge_requests/767)、[kitty 键盘合并请求](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE：[OSC 52 议题](https://gitlab.gnome.org/GNOME/vte/-/issues/2495)、[kitty 键盘议题](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2：[下载](https://iterm2.com/downloads.html)；kitty 键盘来自 [terminfo.dev 对照表](https://terminfo.dev/extensions/kitty-keyboard-protocol)（二手来源）
- Windows Terminal：[1.22 公告，sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release)、[Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/)（二手来源）、[v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio：[转义序列支持](https://rioterm.com/docs/escape-sequence-support)（没有 `OSC 22` 一行）、[功能](https://rioterm.com/docs/features)、[kitty 键盘](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp：[全屏应用](https://docs.warp.dev/terminal/more-features/full-screen-apps)；OSC 52 来自一篇 [Warp OSC 52 安全公告解读](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp)（二手来源）
- VS Code：[高级终端](https://code.visualstudio.com/docs/terminal/advanced)
- mintty：[控制序列](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md)、[技巧](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app（无真彩色）：[Apple 讨论区](https://discussions.apple.com/thread/255315820)（二手来源）
- tmux：[CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES)；zellij：[CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- 对比网站（二手来源，仅用于交叉核对）：[Terminal Trove](https://terminaltrove.com/compare/terminals/)
