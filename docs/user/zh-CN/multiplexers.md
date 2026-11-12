[English](../multiplexers.md) · **简体中文**

# 多路复用器和窗格

多路复用器位于 GitPoulpi 和你的终端之间：它自己重绘单元格，只转发它能理解的转义序列。所以单独在终端里可用的功能（[选择终端](terminals.md)），在 tmux、zellij、screen 或 herdr 里面未必可用。本页说明哪些内容能通过，以及要添加的配置。结果来自与终端表格相同的 2026 年 11 月测试：tmux 3.8（MSYS2：3.7c）、zellij 0.45.1 和 0.46.0、GNU screen 5.0（macOS 自带 4.00.03）、herdr 0.9.5、psmux 3.4.0。

GitPoulpi 通过 `TMUX`、`ZELLIJ`、`STY`、`TERM=screen*` 或 `tmux*` 以及 `HERDR_*` 检测多路复用器。在其中时，图片默认使用字符，除非下表说明 GitPoulpi 检测到了可用的路径。`GITPOULPI_TERMINAL_IMAGE=off` 会关闭所有图片，`ui.image_protocol` 可强制指定协议。

## 一览

| | 图片 | 指针形状（OSC 22） | 剪贴板（OSC 52） | 按键 |
|---|---|---|---|---|
| **tmux** | 背后的终端支持 sixel 时用 sixel（会检测）；用 kitty 图片需要 `allow-passthrough on` 和 `ui.image_protocol = "kitty"` | 在 kitty 和 foot 中，需要 `allow-passthrough on`；Ghostty 中没有 | 需要 `set-clipboard on` | 没有 kitty 键盘；`Ctrl+Enter` 和 `Ctrl+,` 永远送不到 |
| **zellij** | 字符 | 否 | 是 | kitty 键盘可用 |
| **GNU screen** | 字符 | 否 | 否 | 传统按键 |
| **herdr** | kitty 图片（设置 `kitty_graphics = true` 时会检测） | 否 | 是 | 未测量 |
| **psmux**（Windows） | 字符（有意如此） | 否（Windows Terminal 没有） | 系统剪贴板可用 | 未测量 |
| **MSYS2 tmux**（Windows） | 字符（有意如此） | 否 | 系统剪贴板可用 | 未测量；屏幕显示混乱 |
| **WezTerm 窗格**、**Windows Terminal 窗格**、**kitty 和 Ghostty 分屏**、**iTerm2 分屏窗格** | 与终端本身相同 | 与终端相同 | 与终端相同 | 与终端相同 |
| **iTerm2 `tmux -CC`** | 字符 | 否 | 否（iTerm2 默认拒绝） | 未测量 |

## tmux

**图片。**

- **Sixel** 由 tmux 自己绘制，为支持它的客户端重新编码。GitPoulpi 在启动时询问一次 tmux（`tmux display-message -p '#{client_termfeatures}'`，限时 400 ms），客户端列出 sixel 时就使用它。在 foot、WezTerm、Rio 和 Tabby 之后是这种情况；在 kitty 和 Ghostty 之后则不是（tmux 此时会打印“SIXEL IMAGE”文字）。需要带 sixel 支持构建的 tmux（Arch 和 Homebrew 的都有；用 `tmux -V` 和一个 sixel 测试检查）。无需设置。tmux 对设备属性查询的应答，不论背后是什么都声称支持 sixel，所以 GitPoulpi 改为询问客户端的功能。
- **tmux 之后的 iTerm2** 声明支持 sixel，但会把未设置的像素画成不透明，所以大部分透明的图会变成一个实心矩形。GitPoulpi 能识别 iTerm2（`LC_TERMINAL`、`ITERM_SESSION_ID`、客户端类型），并绘制字符。
- **kitty 图片**会通过 tmux 的 DCS 透传（kitty、Ghostty；在原始探测中 iTerm2 和 Rio 也可以）。它们需要 `set -g allow-passthrough on`（tmux 3.3a 或更高版本）**并且**在 GitPoulpi 中设置 `ui.image_protocol = "kitty"`，这样它才会把上传和删除包装起来。没有那个 tmux 选项，包装会被丢弃，图保持空白。
- iTerm2 内联图像能通过 tmux，但 tmux 无法定位它们：GitPoulpi 在那里从不使用它们。

**指针形状。** GitPoulpi 会把 `OSC 22` 包装为透传。在 kitty 和 foot 中，配合 `allow-passthrough on` 它能到达窗口。Ghostty 在被包装时不会显示它（箭头不变）。WezTerm、Rio、Alacritty、Tabby、iTerm2 没有可显示的 `OSC 22`。

**剪贴板。** `set -g set-clipboard on`。GitPoulpi 也会发送包装为透传的形式（配合 `allow-passthrough on` 可用）。在 kitty、Ghostty、Alacritty 和 Tabby 之后它能到达；iTerm2 在打开其剪贴板权限之前会拒绝。

**按键。** tmux 不应答 kitty 键盘查询，而且 `extended-keys` 也带不来 `Ctrl+Enter` 或 `Ctrl+,`：在 kitty 里的 tmux 3.8 上测量，无论 `extended-keys` 是 off、on 还是 always，两者都以普通的 `Enter` 和 `,` 到达。像 `Ctrl+Shift+Z` 这样的字母键序，只有在 `extended-keys` 为 on 或 always，**并且**设置了 `extended-keys-format csi-u`（tmux 3.5 或更高版本），**并且**应用程序请求扩展按键时才会到达；GitPoulpi 从不请求，所以这个设置在这里什么也改变不了。在 tmux 里请使用 `Alt` 变体（[键盘快捷键](keybindings.md#终端无法发送的按键以及-kitty-键盘协议)）。

**配置**（`~/.tmux.conf`）：

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later: pointer shapes, forced kitty pictures
```

**WSL 中的 tmux。** 未测量。它是 Linux 构建，所以预期与上面相同，外层终端为 Windows Terminal。

## zellij

图片：zellij 接受 kitty 协议但不接受它的 Unicode 占位符，也不承载 sixel 或 iTerm2 图像，所以 GitPoulpi 绘制字符（有意否决）。指针形状：被吞掉，所以 GitPoulpi 不发送它。剪贴板：`OSC 52` 会被转发（在 kitty、Ghostty、Alacritty、Tabby 之后可用；iTerm2 和 Terminal.app 会拒绝或忽略它）。按键：zellij 会应答 kitty 键盘查询，所以完整键盘可用。

zellij 0.46 在全新启动时会打开一个“Welcome to Zellij”弹窗：按 `Esc`。已退出的会话保留其名称，下次用同名启动时会询问怎么做。无需任何配置。

## GNU screen

什么都过不去：图片和 `OSC 22` 会被打印为字符或丢弃，`OSC 52` 被忽略，键盘是传统键盘。GitPoulpi 绘制字符。screen 4.00.03（macOS 自带的版本）还会让界面中的颜色和宽字形显示混乱；screen 5 只在 Linux 上测试过。要得到可用的会话，请使用 tmux 或 zellij。

## herdr

herdr 内嵌了一个会重新发出 **kitty 图片**的终端。当 herdr 的 `config.toml` 写着 `kitty_graphics = true`（在 `[terminal]` 下，旧版本中在 `[experimental]` 下），并且 herdr 外面的终端能绘制它们（kitty、Ghostty、WezTerm）时，GitPoulpi 就会使用它们：自动检测，无需设置。如果配置位于别处（例如在 `HERDR_HOME` 下，GitPoulpi 不会读取它），请设置 `ui.image_protocol = "kitty"`。通过 SSH（`herdr --remote`）时，GitPoulpi 看不到本地客户端，所以绘制字符；见[故障排除](troubleshooting.md#图由字符绘制而不是图片herdrtmux其他终端)。herdr 0.9.5 不转发 `OSC 22`，所以 GitPoulpi 不发送它。`OSC 52` 在 kitty、Ghostty、Alacritty 和 Tabby 之后能到达。在原始探测中，herdr 在 kitty、Ghostty 和 iTerm2 中绘制了 kitty 图片，使用默认配置时也是如此。按键：未测量。

## psmux 和 MSYS2 tmux（Windows）

- **psmux**（Rust、ConPTY、tmux 命令语言；`winget install psmux`）会设置 `TMUX`，所以 GitPoulpi 在那里有意绘制字符。在 Windows Terminal 内它运行正常，复制通过系统剪贴板可用；指针不会改变（Windows Terminal 没有 `OSC 22`）。其他宿主终端未测量。
- **来自 MSYS2 或 Cygwin 的 tmux** 同样会设置 `TMUX`（有意绘制字符），并且在测试中屏幕显示混乱：在 mintty 和 Windows Terminal 中，过期的帧相互叠画（无论有没有 `-u`；没有它时，方框绘制字符会被打印成 `qqqq`）。在 MSYS2 伪终端内的原生 Windows 程序，tmux 无法正确重绘。这在 GitPoulpi 中无法修复：请使用 psmux、Windows Terminal 窗格或 WezTerm 窗格。
- zellij 和 screen 没有 Windows 发布版本。herdr 0.9.5 已安装，但没有运行。

## 终端窗格

属于终端自己的窗格，对 GitPoulpi 来说不是多路复用器：

- **WezTerm 窗格**（`wezterm cli split-pane`、`wezterm connect`、SSH 域）：WezTerm 绘制一切，所以图片可用（窗格中的图和头像），指针的表现与 WezTerm 本身相同（带上游 WezTerm PR #8092 的构建中形状可用；窗格分隔线给出调整大小的指针）。未在 Linux 或 macOS 上测量。
- **Windows Terminal 窗格**（`wt ; split-pane`）：窗格中的 sixel 图和头像可用，复制可用，没有指针形状。
- **kitty、Ghostty 和 iTerm2 分屏**：原生，与终端相同。kitty 和 Ghostty 未在 macOS 上测量（它们需要 GPU）。

## iTerm2 `tmux -CC`（控制模式）

iTerm2 为每个 tmux 窗口打开一个原生窗口，并保留一个“[tmux bash]”网关窗口。`TMUX` 已设置，所以 GitPoulpi 在原生窗口中绘制字符。图像确实会通过控制连接到达 iTerm2（对一个 `OSC 1337` 测试出现了“Allow Terminal-Initiated Display?”对话框），但 GitPoulpi 在那里不使用它们。强制设置 `ui.image_protocol = "kitty"` 时，kitty 的负载会被打印在网关窗口中，所以请保持 `auto`。`OSC 22` 和 `OSC 52` 到不了。

## 需要了解的设置

| 设置 | 用途 |
|---|---|
| `ui.image_protocol` | `kitty` 会通过转发它们的多路复用器（带 `allow-passthrough on` 的 tmux，配置是 GitPoulpi 不读取的 herdr）强制使用 kitty 图片；`none` 为字符。 |
| `ui.graph_renderer` | `text` 始终正确。 |
| `ui.kitty_keyboard` | 多路复用器因键盘查询而行为异常时用 `off`。 |

多路复用器后面的终端见[选择终端](terminals.md)，图用字符绘制时见[故障排除](troubleshooting.md)。
