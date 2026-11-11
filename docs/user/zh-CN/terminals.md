[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · **简体中文**

# 选择终端

GitPoulpi 可以在任何支持鼠标的终端中运行，但在能绘制图片、显示真彩色并发送完整键盘按键的终端里，外观和手感最好。本页按操作系统给终端排名，说明每个终端能给你什么，以及要更改哪项设置。这些事实于 2026 年 10 月对照各项目的文档、更新日志或议题跟踪器核实过（见[来源](#来源)）；这里的内容并没有在每一个终端上测试过。凡标有 **（未经验证）** 的内容，都无法从一手来源确认。

## 最佳选择

| 操作系统 | 最佳选择（按顺序） | 原因 |
|---|---|---|
| **Windows** | **WezTerm**，其次 **Windows Terminal** | WezTerm 会把图和头像绘制为图片，并支持透明。Windows Terminal 与系统的集成最好（启动器的默认终端），它把头像绘制为 sixel 图片，图则用字符绘制。 |
| **Linux**（X11、Wayland、Hyprland、Omarchy） | **kitty**、**Ghostty**、**WezTerm**，其次 **foot** | kitty 和 Ghostty 提供图片和完整键盘。WezTerm 也提供图片。foot 把头像绘制为 sixel 图片，速度快，并有完整键盘。在 Omarchy 上，想要图片请选 Ghostty 或 kitty。 |
| **macOS** | **kitty**、**Ghostty**、**WezTerm**，其次 **iTerm2** | 理由相同。Terminal.app 可以用，但既没有图片，也没有真彩色。 |

在任何操作系统上，**tmux、zellij 和 screen** 都会向 GitPoulpi 隐藏终端的图片协议，所以它绘制的是字符（见[多路复用器](#多路复用器)）。

## GitPoulpi 用到什么

- **图片**（图列、作者头像、图像 diff 和文件视图）：带 Unicode 占位符的 **kitty 图形协议**，、**iTerm2 内联图像**（`OSC 1337`）或 **sixel**，按此优先顺序。检测默认拒绝，依据 `TERM`、`TERM_PROGRAM`、`KITTY_WINDOW_ID`、`GHOSTTY_RESOURCES_DIR`、`WEZTERM_EXECUTABLE`、`WT_SESSION`、`KONSOLE_VERSION` 以及多路复用器的变量，并且从不读取键盘（所以也不会向终端查询设备属性）。kitty 和 Ghostty 使用 kitty 协议。WezTerm 使用 iTerm2 协议（有意为之），并混合 PNG 的透明度。iTerm2、Rio 和 mintty 使用 iTerm2 协议，背景不透明。Windows Terminal、foot、Konsole 和 mlterm 使用 **sixel**。这些都不可用时，头像显示为首字母徽章。
- **Sixel** 用于绘制**图列**（泳道、节点，以及设置 `ui.avatars = "graph"` 时 commit 圆圈内的头像），其他位置用于绘制**头像**（Log、commit 详情、文件历史、Blame、图像视图）。sixel 没有 alpha 通道、没有 ID，也不能缩放，所以图片恰好是 `列数 x 单元格宽度` 乘 `行数 x 单元格高度` 像素，只在有内容的地方绘制，位于保留背景的空白单元格之上；编码后的图片会被缓存，只在泳道、滚动位置或缩放改变时重新编码，只在其下的单元格被重绘时重新发送。sixel 没有删除操作：GitPoulpi 通过在其单元格上写入文本来移除它。sixel 需要单元格的真实像素大小：启动时和缩放后会向 Windows Terminal 询问一次（`CSI 16 t`，约 10 毫秒内应答，不应答也绝不阻塞）；其他终端取自窗口大小或 `GITPOULPI_TERMINAL_CELL_PX=10x20`。能绘制 sixel 但未被识别的终端（开启了 `terminal.integrated.enableImages` 的 VS Code 终端，以 `-ti vt340` 启动的 xterm）需要设置 `ui.image_protocol = "sixel"`。1.22 之前的 Windows Terminal 不能绘制 sixel；请在其中设置 `ui.image_protocol = "none"`（图列此时保持字符）。
- **真彩色**：主题是 RGB。`ui.color_depth = "auto"` 在 `TERM` 为 `*-256color` 且没有 `COLORTERM=truecolor` 时（以及在 Terminal.app 中）降到 256 色，在 Linux 控制台降到 16 色。kitty、WezTerm 和 Windows Terminal 视为真彩色。
- **鼠标**：带按键、拖动和移动的 SGR 鼠标报告（悬停反馈、拖放、右键点击、双击）。
- **键盘**：启动时会以 100 ms 超时请求 **kitty 键盘协议**（只用 disambiguate 标志）。它让 `Ctrl+,`、`Ctrl+Enter`、`Ctrl+Shift+N` 和 `Ctrl+Shift+Z` 能够送达。没有它时，保留传统按键（`Ctrl+Y` 重做，`Alt` 变体替代部分键序）。参见[键盘快捷键](keybindings.md#终端无法发送的按键以及-kitty-键盘协议)。
- **剪贴板**：复制操作会写出 `OSC 52`。
- **鼠标指针形状**（分隔线上的调整大小指针、可点击项上的手形）：`OSC 22`，在支持的地方是额外的加分项。
- **焦点事件**已启用（窗口失去焦点时工具提示会关闭）。GitPoulpi 不请求括号粘贴、OSC 8 超链接或 undercurl。仅在图片路径上，每一帧都包在同步更新里（模式 2026，不认识它的终端会忽略），这样图片不会在弹出窗口上方闪烁。
- **表情符号和符号**：GitPoulpi 只使用默认表情呈现的字形，或单宽度的 BMP 形状，因为终端和字体对变体选择符的处理并不一致。

## 能力表

“图片”指 GitPoulpi 目前在该终端中实际绘制的内容（**kitty** 或 **iTerm2** 协议，或 **文本**），而不是终端本身能做到的。

| 终端 | 操作系统 | 图片（图、头像、图像 diff） | 真彩色 | Kitty 键盘（完整键序） | OSC 52 复制 | OSC 22 指针 |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux、macOS | kitty | 是 | 是 | 是 | 是 |
| **Ghostty** (1.3) | Linux、macOS（没有 Windows 版本） | kitty | 是 | 是 | 是 | 是，自 1.0 起 |
| **WezTerm** | Windows、Linux、macOS | iTerm2，支持透明 | 是 | 仅在 `enable_kitty_keyboard = true` 时（默认 `false`） | 是（仅写入） | 否（拉取请求仍未合并） |
| **foot** (1.28) | Linux (Wayland) | 图和头像都用 sixel（没有 kitty 图形） | 是 | 是，自 1.10.3 起 | 是（`security.osc52`） | 是，自 1.12 起 |
| **Alacritty** (0.17) | 全部 | 文本（上游不支持图像） | 是 | 是，自 0.13 起 | 是（写入；粘贴默认关闭） | 否（未实现） |
| **Konsole** | Linux | 图和头像都用 sixel | 是 | 2026 年正在加入（合并请求 !1188）；发布版本（未经验证） | 仅写入，2024 年中合并 | （未经验证） |
| **GNOME Terminal、Ptyxis**（VTE） | Linux | 文本 | 是 | 否（已发布的 VTE 中没有） | 否（VTE 忽略它） | 否 |
| **iTerm2** (3.7) | macOS | iTerm2，背景不透明 | 是 | 是，自 3.6.9 起（检查 Profiles > Keys） | 默认关闭：启用 “Applications in terminal may access clipboard” | （未经验证） |
| **Terminal.app** | macOS | 文本 | 否（256 色） | 否 | （未经验证） | （未经验证） |
| **Windows Terminal** (1.25) | Windows | 图和头像都用 sixel | 是 | 是，2026 年 10 月起稳定版 1.25 支持；预览版自 2026 年 3 月的 1.25 起 | 是（写入；仅在获得焦点时） | 否（未实现） |
| **Rio** | Windows、Linux、macOS | iTerm2，背景不透明（它也支持 kitty 图形） | 是 | 是（默认开启） | 是 | 否（未实现） |
| **Warp** | macOS、Linux、Windows | 文本（未被检测到） | 是 | 是 | 是（它遵循该序列） | （未经验证） |
| **VS Code 终端**（xterm.js） | 全部 | 文本（sixel 和 iTerm2 图像在 `terminal.integrated.enableImages` 之后可用；`ui.image_protocol = "sixel"` 会使用 sixel） | 是 | （未经验证） | （未经验证） | 否 |
| **mintty**（Cygwin、Git Bash） | Windows | iTerm2，背景不透明 | 是 | （未经验证；无文档说明） | （未经验证；无文档说明） | 是 |
| **Windows 控制台**（conhost、cmd、PowerShell 窗口） | Windows | 文本 | 仅较新的 Windows 10/11 | 不需要：它自己报告修饰键 | 否 | 否 |
| **ConEmu** | Windows | 文本 | （未经验证） | （未经验证） | （未经验证） | （未经验证） |
| **tmux、zellij、screen** | 全部 | 文本，始终如此 | 需要设置 | 需要设置 | 需要设置 | 否 |

OSC 22 一列中的“否（未实现）”，来自项目更新日志和文档中的缺失以及用户反馈，而不是测试结果；请把它们当作“无文档说明”。本页没有列出表情符号和变体选择符的宽度：它因终端和字体而异，这也是 GitPoulpi 避免使用 VS16 字形的原因（见 [GitPoulpi 用到什么](#gitpoulpi-用到什么)）。

## 各终端详情

### Windows

- **WezTerm。** Windows 上最好的图片支持。让 `ui.image_protocol` 保持 `auto`。在 `wezterm.lua` 中设置 `config.enable_kitty_keyboard = true` 以获得完整键序，并使用带表情符号回退的字体，例如 `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`。WezTerm 最后一个带标签的稳定版本是 20240203-110809-5046fc22（2024 年 2 月）；更新的修复发布在 nightly 中。WezTerm 还没有 OSC 22 指针形状。
- **Windows Terminal。** 真彩色、鼠标、OSC 52 和良好的表情符号支持。GitPoulpi 把头像绘制为 sixel 图片（需要 Windows Terminal 1.22 或更高版本）；图用字符。在 *设置 > 配置文件 > 默认值 > 外观 > 字体* 中选一款 Nerd Font。鼠标被捕获时，按住 `Shift` 可选择文本。指针形状不会改变。在早于 1.25 的版本（没有 kitty 键盘协议）上，请使用 `Alt` 变体。
- **Rio** 可以配合不透明的图片背景使用；测试较少。
- **mintty**（Git Bash、MSYS2、Cygwin）通过 iTerm2 协议绘制图片，背景不透明；测试较少。
- **Windows 控制台、ConEmu、cmd**：避免使用。图片绘制得很差，或根本不绘制。双击 exe 会在你选择的终端中重新打开 GitPoulpi（*设置 > 启动器*）。

### Linux

- **kitty。** 无需任何设置，一切正常。`Shift+点击` 是 kitty 自己的，所以请用 `Alt+点击` 扩展选择（[鼠标](mouse-and-gestures.md)）。
- **Ghostty。** 无需任何设置，一切正常。如果图标显示为空洞，请在 `font-family` 中加入表情符号字体或 Nerd Font（重复该键即可添加回退字体）。
- **WezTerm。** 通过 iTerm2 协议显示图片，混合透明度。`enable_kitty_keyboard` 的说明与 Windows 上相同。
- **foot。** 字符图、完整键盘、速度快。头像是 sixel 图片，图是字符。在 `foot.ini` 中设置 `font=JetBrainsMono Nerd Font:size=11`。
- **Alacritty。** 字符图，自 0.13 起支持 kitty 键盘。Alacritty 不支持图像。
- **Konsole、GNOME Terminal、Ptyxis（VTE）。** 使用字符和传统按键：请使用 `Alt` 变体，并用 `Ctrl+Y` 重做（较新的 Konsole 可能会加入 kitty 键盘协议）。VTE 的复制操作（`OSC 52`）不会向剪贴板放入任何内容。输入法（IBus、Fcitx）可能会抢占 `Ctrl+Space` 暂存切换键 - **（未经验证）**；请在 *设置 > 键盘与鼠标* 中重新绑定。
- **Wayland、Hyprland、Omarchy。** 鼠标指针提示（`OSC 22`）在 kitty、Ghostty 和 foot 中有效。Omarchy 的启动器条目通过 `xdg-terminal-exec` 运行。

### macOS

- **kitty、Ghostty、WezTerm**：与 Linux 相同。
- **iTerm2。** 图片的背景不透明（如果你的背景与主题不同，会看到一个矩形；`ui.graph_renderer = "text"` 可以去掉它）。复制需要启用 *Settings > General > Selection > Applications in terminal may access clipboard*，`Alt` 键序需要 *Profiles > Keys > Left Option key: Esc+*。
- **Terminal.app。** 只有 256 色：GitPoulpi 会自己选择 `ui.color_depth = 256`，主题看起来更平淡。没有图片。要使用 `Alt` 键序，请勾选 *Use Option as Meta key*。

### VS Code 和 Warp

VS Code 终端（`TERM_PROGRAM=vscode`）和 Warp 不会被识别为图片终端，所以 GitPoulpi 绘制字符。两者都可以使用。在 VS Code 中，开启 `terminal.integrated.enableImages` 并设置 `ui.image_protocol = "sixel"` 即可显示头像图片。Warp 的文档说明支持 kitty 键盘协议；VS Code 终端是否支持 **（未经验证）**。

## 多路复用器

GitPoulpi 会检测 tmux、zellij、screen 和 herdr（`TMUX`、`ZELLIJ`、`STY`、`TERM=screen*` 或 `tmux*`、`HERDR_*`），并绘制**字符**，因为多路复用器转发的是单元格，而不是图片转义序列。要使用其他功能，请在 `~/.tmux.conf` 中加入：

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g extended-keys on                # full chords (tmux 3.2 or later)
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later; only if you force pictures, see below
```

要通过会转发图片的多路复用器强制显示图片，请把 `ui.image_protocol` 设为 `kitty` 或 `iterm2`。在 kitty、Ghostty 或 WezTerm 中，开启了 `kitty_graphics = true` 的 herdr 会被自动检测到；参见[故障排除](troubleshooting.md#图由字符绘制而不是图片herdrtmux其他终端)。`GITPOULPI_TERMINAL_IMAGE=off` 会关闭所有图片。

tmux 本身不转发 kitty 键盘协议：`extended-keys` 让它以 `CSI u` 或 `xterm` 序列发送带修饰键的按键。根据其更新日志，zellij 0.41 及更高版本支持 kitty 键盘协议，并转发 OSC 52。

## 缺失时会怎样降级

| 缺失 | 你会得到什么 |
|---|---|
| 图片协议 | 字符图。头像变成带颜色的首字母徽章（一个字母），无论 `ui.avatar_fallback` 怎么设：挤进文本单元格里的 identicon 根本认不出来。图像 diff 和文件视图使用带颜色的半块字符，或信息卡。 |
| 真彩色 | 颜色被映射到 256 色或 16 色调色板（`ui.color_depth`）。 |
| Kitty 键盘协议 | 传统按键；部分键序会以另一个按键到达。设置中会给出警告，你可以重新绑定，或使用 `Alt` 变体。 |
| OSC 52 | 复制操作不会向剪贴板放入任何内容：用鼠标选择（按住 `Shift`），再使用终端自己的复制。 |
| OSC 22 | 指针不变；鼠标下的分隔线或按钮仍会高亮。 |
| 表情符号字体 | 图标位置出现方框或空洞；图和文字仍然正常。请安装 Nerd Font 或表情符号字体，并把它列为回退字体。 |

## 需要了解的设置

| 设置 | 用途 |
|---|---|
| `ui.image_protocol` | `auto`（默认）、`kitty`、`iterm2`、`sixel`、`none`。如果在应有图片的地方看到乱码，请设为 `none`。需要重启。 |
| `ui.graph_renderer` | `auto`、`image`、`text`。`text` 始终正确。需要重启。 |
| `ui.color_depth` | `auto`、`truecolor`、`256`、`16`。 |
| `ui.kitty_keyboard` | `auto` 或 `off`，用于终端或多路复用器行为异常时。需要重启。 |
| `ui.mouse_enabled` | 关闭后鼠标交还给终端。需要重启。 |
| `ui.launch_terminal` | 系统快捷方式所打开的终端（[设置](settings.md)）。 |

另见[图片](image-view.md)、[键盘快捷键](keybindings.md)和[故障排除](troubleshooting.md)。

## 来源

2026 年 10 月核实。除非注明，均为一手来源。

- kitty：[更新日志](https://sw.kovidgoyal.net/kitty/changelog/)、[指针形状](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty：[OSC 22](https://ghostty.org/docs/vt/osc/22)、[OSC 52](https://ghostty.org/docs/vt/osc/52)、[下载](https://ghostty.org/download)
- WezTerm：[enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html)、[转义序列](https://wezterm.org/escape-sequences.html)、[OSC 22 拉取请求](https://github.com/wezterm/wezterm/pull/6292)、[发布版本](https://github.com/wezterm/wezterm/releases)
- foot：[更新日志](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty：[更新日志](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md)、[发布版本](https://github.com/alacritty/alacritty/releases)
- Konsole：[OSC 52 合并请求](https://invent.kde.org/utilities/konsole/-/merge_requests/767)、[kitty 键盘合并请求](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE：[OSC 52 议题](https://gitlab.gnome.org/GNOME/vte/-/issues/2495)、[kitty 键盘议题](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2：[下载](https://iterm2.com/downloads.html)；kitty 键盘来自 [terminfo.dev 对照表](https://terminfo.dev/extensions/kitty-keyboard-protocol)（二手来源）
- Windows Terminal：[1.22 公告，sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release)、[Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/)（二手来源）、[v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio：[功能](https://rioterm.com/docs/features)、[kitty 键盘](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp：[全屏应用](https://docs.warp.dev/terminal/more-features/full-screen-apps)；OSC 52 来自一篇 [Warp OSC 52 安全公告解读](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp)（二手来源）
- VS Code：[高级终端](https://code.visualstudio.com/docs/terminal/advanced)
- mintty：[控制序列](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md)、[技巧](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app（无真彩色）：[Apple 讨论区](https://discussions.apple.com/thread/255315820)（二手来源）
- tmux：[CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES)；zellij：[CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- 对比网站（二手来源，仅用于交叉核对）：[Terminal Trove](https://terminaltrove.com/compare/terminals/)

仍未验证：Konsole 和 mlterm 中的 sixel（取自项目自己的功能列表，没有对照发布版本重新核实，也没有在任何终端的真实窗口中试过）；Konsole、iTerm2、Terminal.app 和 Warp 中的 OSC 22；Terminal.app、VS Code 和 mintty 中的 OSC 52；VS Code、mintty 和 ConEmu 中的 kitty 键盘协议；ConEmu 的每一个单元格；以及各终端的表情符号宽度。Konsole 在哪个版本中提供 kitty 键盘协议，也没有得到确认。
