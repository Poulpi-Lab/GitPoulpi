[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · **简体中文**

# 安装

GitPoulpi 有三个可执行文件：

| 可执行文件 | 说明 |
|---|---|
| `gitpoulpi-tui` | 终端界面。 |
| `gitpoulpi` | 命令行界面（脚本可使用 `--json`）。不带子命令时，它会打开界面。 |
| `gitpoulpi-mcp` | 面向 AI 代理的 MCP 服务器。参见 [MCP 与代理](mcp-and-agents.md)。 |

除这些文件外，压缩包里还有 `gitpoulpi-launch`，这是快捷方式用来显示已打开的 GitPoulpi 窗口的小程序（`gitpoulpi launch`）。在 Windows 上它没有控制台窗口，所以点击时不会闪出窗口。它与其他文件一起安装、更新和删除。

## 你需要什么

- `PATH` 中有**系统 `git`**。GitPoulpi 的大多数操作使用 libgit2，其余使用你的 `git`。
  需要 **2.34 或更高版本**。在 macOS 上，Apple Command Line Tools 自带的 `git` 和 Homebrew 的 `git` 都可以使用。
- 支持鼠标的**终端**。kitty、Ghostty、WezTerm 和 iTerm2，以及通过 sixel 的 Windows Terminal 和 foot，会把 commit 图绘制为图片。其他终端（Windows 控制台、tmux 以及大多数其他终端）显示字符图。参见[选择终端](terminals.md)。
- **Nerd Font** 或带 emoji 的字体，用于工具栏和侧边栏中的图标。
- 可选：**`git-lfs`** 3.0 或更高版本，用于 Git LFS 仓库。没有它时，LFS 文件会显示为小的指针文件，GitPoulpi 会告诉你原因。安装方法见 [Git LFS](lfs.md)。
- 可选：**credential helper**（Git Credential Manager、libsecret、`pass`、Keychain），用于保存托管平台令牌和 AI API 密钥。参见[托管平台与议题跟踪器](hosting-integration.md)。

## 从发布压缩包安装

发布压缩包为 `gitpoulpi-<version>-<target>.zip`（Windows）或 `.tar.gz`（Linux、macOS），旁边附有一个 `.sha256` 文件。请对照 `.sha256` 文件检查压缩包，解压后，把三个可执行文件放进 `PATH` 中的某个文件夹。压缩包内的 `INSTALL.txt` 给出相同的步骤。

## 使用安装程序

每个发布版本还为各个系统提供了安装程序。两者做出的选择相同（快捷方式、PATH、`gp` 之类的命令别名），都是调用已安装的 `gitpoulpi` 本身来完成，也就是“设置 > 常规 > 启动器”背后的那段代码，所以安装程序设置的内容与设置界面显示的内容始终一致。

### Windows：`gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

它为当前用户安装，不弹出管理员提示，安装到 `%LOCALAPPDATA%\Programs\GitPoulpi`（第一个对话框可以改为为所有用户安装，并可更改安装文件夹）。复选框：开始菜单快捷方式、桌面快捷方式、添加到 PATH。有一页会询问**命令别名**：默认为 `gp`，可填任意名称，留空则不创建。PowerShell 已经把 `gp` 用作 `Get-ItemProperty`，所以在那里请换一个名称，或直接使用 `gitpoulpi`。

安装程序会复制三个可执行文件和 `.gitpoulpi-dist` 标记文件，因此[立即更新](#原地更新)的行为与 zip 完全相同。按用户安装的目录可写，所以能自行更新；装在 Program Files 的所有用户安装需要提升权限才能被替换，请改为用新的安装程序重新安装。卸载请在 Windows 设置 > 应用中进行：它会运行 `gitpoulpi uninstall`（别名、PATH 条目、快捷方式），并删除这些文件，包括更新程序留下的 `.old` 文件。你的设置和仓库会保留。

### Linux 和 macOS：`install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

它会选择适合你的系统和 CPU 的压缩包，检查其 `.sha256`，安装到 `~/.local/opt/gitpoulpi`，并把 `gitpoulpi` 和 `gitpoulpi-mcp` 链接到 `~/.local/bin`。不需要 root 权限，也不会改动主目录之外的任何内容。不加 `--yes` 时，它会询问别名（默认 `gp`）、PATH 和快捷方式。

| 参数 | 作用 |
|---|---|
| `--prefix DIR` | 把程序文件安装到 `DIR`（绝对路径）。 |
| `--version X.Y.Z` | 安装指定版本，而不是最新版本。 |
| `--archive FILE` | 使用本地压缩包；`FILE.sha256` 必须放在它旁边（`--no-verify` 可跳过检查）。不使用网络。 |
| `--alias NAME` / `--no-alias` | 让 `NAME` 启动 GitPoulpi（`gitpoulpi alias create`），或不创建别名。 |
| `--no-path` | 不把 GitPoulpi 添加到新 shell 的 PATH（否则会执行 `gitpoulpi path add`）。 |
| `--desktop` | 同时创建应用程序菜单和桌面快捷方式。 |
| `--yes` | 从不询问：按参数和默认值执行（别名 `gp`、添加 PATH、不创建快捷方式）。 |
| `--uninstall` | 运行 `gitpoulpi uninstall`，然后删除链接和文件夹。 |

查询最新发布版本要求仓库是公开的；私有仓库请在浏览器中下载压缩包，再使用 `--archive`。

## 从源码安装

你需要较新的稳定版 Rust 工具链（<https://rustup.rs>）。

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

可执行文件位于 `target/release/`。`release` 配置构建很快。若要获得最小、最快的可执行文件，请使用 `dist` 配置，它的构建时间更长：

```
cargo build --profile dist
```

仓库根目录下的 `build-opti` 脚本会为这三个可执行文件执行此构建，并打印它们的大小：

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd or PowerShell)
```

若只想构建你用到的语言和功能，请参见[自定义构建](custom-builds.md)。

## 自行构建发布压缩包

```
cargo run -p xtask -- dist
```

这会构建 `dist` 配置，并写出 `target/dist/gitpoulpi-<version>-<target>.zip`（Windows）或 `.tar.gz`（其他系统）。压缩包内含三个可执行文件、`INSTALL.txt`、`README.md`、`LICENSE` 和 `CHANGELOG.md`。旁边还会写出一个 `.sha256` 文件。

## 更新

标题栏在应用名称和仓库路径之间显示正在运行的版本（`v1.0.0`）。点击它，或在命令面板（`Ctrl+P`）中运行 **帮助：发行说明**，会打开“发行说明”标签页，显示更新日志，并滚动到当前运行的版本。

GitPoulpi 每天在后台向 GitHub 询问一次是否有更新的发布版本（`updates.mode`，[设置 > 同步](settings.md#同步)）。有的话，标题栏里的标记会以强调色显示 `v0.2.0 可用`，点击它会打开“发行说明”标签页，并把该版本的说明放在最上面。标签页有三个按钮，每个都有对应的按键和命令面板命令：

- **下载**（`d`，*更新：下载*）在浏览器中打开发布页面。
- **跳过此版本**（`s`，*更新：跳过此版本*）隐藏提示，直到出现更新的版本。
- **立即检查**（`r`，*更新：立即检查更新*）立刻重新询问一次。

`updates.mode = "off"` 完全不发出请求。没有网络连接时，或者仓库为私有且没有存储 github.com 令牌时，检查会悄悄失败（应用日志里记一行，界面上没有任何提示）。`GITPOULPI_UPDATE_API` 可让检查指向另一个 API 根地址（测试用）。

### 原地更新

通过发布压缩包安装的版本可以替换自身。压缩包里可执行文件旁带有一个很小的 `.gitpoulpi-dist` 文件；只有当该文件位于可执行文件旁边（且写明了它所属的平台）时，正在运行的版本才算“可更新”。用 `cargo` 构建的版本、复制到别处的自定义构建，或丢掉了该文件的软件包，都不会自行更新：它的“发行说明”标签页会说明这一点，且只提供**下载**。

对于可更新的版本，标签页还会增加：

- **立即更新**（`u`，*更新：立即更新*）在后台下载压缩包及其 `.sha256`（状态栏指示器里的一个任务，带百分比，和其他任务一样可以取消），校验哈希，解压，并把每个新文件作为 `<name>.new` 放在旧文件旁边。正在运行的内容不会被动到。在 `auto` 模式下，一旦检查发现更新的版本，这一步会自动完成。
- **重启以更新**（`R`，*更新：重启以更新*；下载就绪后，状态栏标记也会显示同样的文字）始终由你来操作，绝不会自动进行。它会停止该会话的独立守护进程（内嵌的守护进程随窗口一起结束），替换文件，并以相同的参数和会话启动新版本。
- **回退到上一个版本**（`v`，*更新：回退到上一个版本*）把之前的文件换回来并重启。上一个版本的文件会以 `<name>.old` 保留，直到新版本运行满一分钟后才被删除，所以这个按钮是留给第一次运行的。它不属于 git 的撤销日志：它改动的是 GitPoulpi 自己的文件，而不是某个仓库。

文件如何被替换：在 Linux 和 macOS 上，每个新文件都以重命名的方式覆盖旧文件（原子操作；正在运行的程序在退出前一直使用旧副本）。在 Windows 上，正在运行的 `.exe` 无法被覆盖，所以旧文件被重命名为 `<name>.old`，再把新文件移入；仍在运行的 `.old` 会在之后的某次启动时被删除。替换要么全部完成，要么全都不做：如果另一个程序占用着其中一个可执行文件（例如运行着 `gitpoulpi-mcp` 的 MCP 客户端），GitPoulpi 会指出是哪个文件，不改动任何内容，并保留已下载的文件以便再试。

以下情况会被拒绝，并各有提示：发布版本没有 `.sha256` 文件、哈希不匹配、压缩包文件名不是适用于当前平台的名称（`gitpoulpi-<version>-<target>.<zip|tar.gz>`）、压缩包中的标记指向另一个版本或平台，以及指向这些主机之外的任何下载链接：GitHub 的 `github.com`、`api.github.com`、`objects.githubusercontent.com`、`release-assets.githubusercontent.com` 和 `github-releases.githubusercontent.com`（首次请求和每一次重定向；重定向到其他任何地方都会中止下载）。私有仓库的压缩包需要浏览器会话，因此在仓库公开之前，那里的下载会失败。

## 检查是否正常工作

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version` 会打印版本以及构建所用的 commit。然后请按照[快速上手](getting-started.md)继续。

## 卸载

`gitpoulpi uninstall`（两个安装程序都会运行它）会移除 GitPoulpi 创建的命令别名、PATH 条目和快捷方式，然后移除发布版安装的程序文件；在没有 `.gitpoulpi-dist` 标记的构建（cargo 构建）上它会拒绝执行，`--keep-files` 则保留程序文件。MCP 代理无法使用它。不用安装程序时，直接删除这三个可执行文件即可。GitPoulpi 还会保留以下文件：

| 内容 | 位置 |
|---|---|
| 设置（`config.toml`）和会话 | 你操作系统的配置文件夹。可用 `GITPOULPI_CONFIG_DIR` 更改。 |
| 各仓库的状态 | 默认位于仓库 `.git` 文件夹内的 `gitpoulpi/`（`session.per_repo_state_dir` 可把它改到工作区的 `.gitpoulpi/` 或用户数据文件夹）。 |
| 缓存（头像） | 你操作系统的缓存文件夹。 |
| 令牌和 API 密钥 | 你的 credential helper。**设置 > SSH 与凭据 > 忘记所有用户名和密码**可将其删除。 |

确切路径参见[配置参考](config-reference.md)。
