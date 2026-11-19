[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · **简体中文**

# 安装

GitPoulpi 是一个程序，`gitpoulpi`。不带子命令运行时它会打开终端界面；带子命令时它是命令行界面（脚本可使用 `--json`）；`gitpoulpi mcp serve` 是面向 AI 代理的 MCP 服务器（参见 [MCP 与代理](mcp-and-agents.md)）。压缩包里还有 `gitpoulpi-updater`，这是另一个负责更新安装的小程序（参见[更新](updates.md)）。

为了兼容，压缩包里还有三个只负责启动 `gitpoulpi` 的小型中继程序：`gitpoulpi-tui`（打开界面；将在后续版本中移除）、`gitpoulpi-mcp`（启动 MCP 服务器；它在整个 1.x 系列中都会保留，因此现有的代理配置可继续使用）和 `gitpoulpi-launch`（快捷方式用它来显示已打开的 GitPoulpi 窗口，即 `gitpoulpi launch`；在 Windows 上它没有控制台窗口，所以点击时不会闪出窗口）。它们会一起安装、更新和删除。

## 你需要什么

- `PATH` 中有**系统 `git`**。GitPoulpi 的大多数操作使用 libgit2，其余使用你的 `git`。
  需要 **2.34 或更高版本**。在 macOS 上，Apple Command Line Tools 自带的 `git` 和 Homebrew 的 `git` 都可以使用。
- 支持鼠标的**终端**。kitty、Ghostty、WezTerm、Rio 和 iTerm2，以及通过 sixel 的 Windows Terminal、foot 和 Tabby，会把 commit 图绘制为图片。其他终端（Windows 控制台、zellij、screen 以及大多数其他终端）显示字符图。参见[选择终端](terminals.md)和[多路复用器](multiplexers.md)。
- **Nerd Font** 或带 emoji 的字体，用于工具栏和侧边栏中的图标。
- 可选：**`git-lfs`** 3.0 或更高版本，用于 Git LFS 仓库。没有它时，LFS 文件会显示为小的指针文件，GitPoulpi 会告诉你原因。安装方法见 [Git LFS](lfs.md)。
- 可选：**credential helper**（Git Credential Manager、libsecret、`pass`、Keychain），用于保存托管平台令牌和 AI API 密钥。参见[托管平台与议题跟踪器](hosting-integration.md)。

## 从发布压缩包安装

发布压缩包为 `gitpoulpi-<version>-<target>.zip`（Windows）或 `.tar.gz`（Linux、macOS），旁边附有一个 `.sha256` 文件。请对照 `.sha256` 文件检查压缩包，解压后，把这些可执行文件（`gitpoulpi`、`gitpoulpi-updater` 以及中继程序 `gitpoulpi-tui`、`gitpoulpi-mcp` 和 `gitpoulpi-launch`）放进 `PATH` 中的某个文件夹，并附上 `.gitpoulpi-dist` 标记文件（请把 `gitpoulpi-updater` 和标记文件放在其他文件旁边：标记写的是 `channel=archive`，这样该构建才能自行更新，见[更新](updates.md)）。压缩包内的 `INSTALL.txt` 给出相同的步骤。

### 验证下载

每个发布文件都用项目的 [minisign](https://jedisct1.github.io/minisign/) 密钥签名，GitPoulpi 在安装更新之前会自己检查该签名。对于你手动下载的文件，旁边的 `.minisig` 就是签名。公钥（密钥 id 为 `B461FFCBF6F6A4F2`）作为 `packaging/minisign.pub` 提交在仓库中：

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

请阅读该命令打印出的受信任注释：它必须是 `file:<该文件的名称> version:<你期望的版本>`。有效但写明了另一个文件或另一个版本的签名，不是你这个文件的签名。

如果发布版本是带构建来源证明（仓库的 `ATTESTATIONS` 变量）构建的，GitHub CLI 可以检查文件是在哪里构建的：

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

`.sha256` 文件只能发现损坏的下载。签名才能证明该文件出自本项目。

## 使用安装程序

每个发布版本还为各个系统提供了安装程序。两者做出的选择相同（快捷方式、PATH、`gp` 之类的命令别名），都是调用已安装的 `gitpoulpi` 本身来完成，也就是“设置 > 启动器”背后的那段代码，所以安装程序设置的内容与设置界面显示的内容始终一致。

### Windows：`gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

第一个对话框会询问为谁安装：

| 选择 | 位置 | 管理员权限 | 对更新意味着什么 |
|---|---|---|---|
| **仅为我**（默认） | `%LOCALAPPDATA%\Programs\GitPoulpi` | 不需要 | GitPoulpi 替换自身的文件：在[更新面板](updates.md)中使用**更新并重启**，没有提示。 |
| **为所有用户** | `Program Files\GitPoulpi` | 安装时一次提示 | 每次更新都会请求管理员批准（Windows 提示），并且这台电脑上其他用户的 GitPoulpi 会话会先被关闭。 |

同样的选择也可以静默进行：`gitpoulpi-<version>-...-setup.exe /CURRENTUSER` 或 `/ALLUSERS`。

复选框：开始菜单快捷方式、桌面快捷方式、添加到 PATH。有一页会询问**命令别名**：默认为 `gp`，可填任意名称，留空则不创建（命令行中用 `/ALIAS=name` 或 `/ALIAS=`）。PowerShell 已经把 `gp` 用作 `Get-ItemProperty`，所以在那里请换一个名称，或直接使用 `gitpoulpi`。

安装程序会复制这些可执行文件（包括 `gitpoulpi-updater`，更新期间负责替换文件的小程序）和 `.gitpoulpi-dist` 标记文件。标记记录了 GitPoulpi 是怎样安装的（`inno-user` 或 `inno-machine`），更新面板把它显示为**安装方式**。再次运行安装程序会保留你的选择（范围、快捷方式、别名）。

卸载请在 Windows 设置 > 应用中进行：它会运行 `gitpoulpi uninstall`（别名、PATH 条目、快捷方式），并删除这些文件，包括更新程序留下的 `.old` 文件。你的设置和仓库会保留。

### Linux 和 macOS：`install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

它会选择适合你的系统和 CPU 的压缩包，检查其 `.sha256`，安装（通道为 `archive`：GitPoulpi 会原地自行更新）到 `~/.local/opt/gitpoulpi`，并把 `gitpoulpi` 和 `gitpoulpi-mcp`（中继程序）链接到 `~/.local/bin`。不需要 root 权限，也不会改动主目录之外的任何内容。不加 `--yes` 时，它会询问别名（默认 `gp`）、PATH 和快捷方式。

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

## 使用包管理器

下面每个通道都会在程序旁写入同样的 `.gitpoulpi-dist` 标记，只是通道名称各不相同。GitPoulpi 读取它来了解由谁负责更新：由包管理器负责，所以 GitPoulpi 绝不会在它背后替换文件。更新面板（点击标题栏中的版本号）随后会运行包管理器自己的命令，或者把它显示出来供你复制。见[更新](updates.md)。

| 通道 | 安装 | 说明 |
|---|---|---|
| **Chocolatey**（Windows） | `choco install gitpoulpi` | 为所有用户安装，所以更新会请求管理员批准。 |
| **Scoop**（Windows） | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi` 然后 `scoop install gitpoulpi` | 按用户安装，不需要管理员权限。 |
| **Homebrew**（macOS、Linux） | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | tap 是 `Poulpi-Lab/homebrew-gitpoulpi`。 |
| **apt**（Debian、Ubuntu） | 见 [apt 软件源](#aptdebian-和-ubuntu) | 已签名的软件源；每个发布版本还附带一个 `.deb`。 |
| **MSI**（Windows，受管理） | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | 面向管理员：所有用户，没有应用内更新。见[企业分发](enterprise-distribution.md#msi使用-intune-或组策略)。 |
| **cargo** | `cargo binstall gitpoulpi-cli` | 下载发布压缩包而不是编译。 |
| **cargo，来自 crates.io** | `cargo install --locked gitpoulpi-cli` | 需要编译；需要 Rust 工具链。安装 `gitpoulpi`，不带子命令运行时打开终端界面，同时提供 MCP 服务（`gitpoulpi mcp`）。 |
| **cargo，从源码** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | 需要编译；需要 Rust 工具链。 |
| **Docker** | `docker pull poulpilab/gitpoulpi` | 见[在 Docker 中运行](docker.md)。同一个镜像也在 `ghcr.io/poulpi-lab/gitpoulpi` 上。 |

上面的名称是发布流程所发布到的名称（见[发布](../../developer/releasing.md)）：尚未上线的通道会回答“not found”，而发布压缩包或安装程序始终可用。

包管理器自己的 PATH 处理会生效：Chocolatey、Scoop 和 Homebrew 会自己把程序放进 PATH，而且它们都不会创建 `gp` 别名（请运行 `gitpoulpi alias create gp`，或使用设置 > 启动器）。

### apt（Debian 和 Ubuntu）

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

该软件源支持 `amd64` 和 `arm64`，由属于本项目的密钥签名。程序安装到 `/usr/lib/gitpoulpi`，并在 `/usr/bin` 中建立链接。该软件包依赖 `git`，并推荐 `git-lfs`。没有该软件源时，可以用 `sudo apt install ./gitpoulpi_<version>_<arch>.deb` 安装某个发布版本的 `.deb`：此后 GitPoulpi 会告诉你自己去下载下一个 `.deb`，因为只有在 `/etc/apt/sources.list.d/gitpoulpi.*` 中找到该软件源时，它才会提供 `apt` 命令。

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

仓库根目录下的 `build-opti` 脚本会执行此构建，并打印可执行文件的大小：

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd or PowerShell)
```

若只想构建你用到的语言和功能，请参见[自定义构建](custom-builds.md)。

## 自行构建发布压缩包

```
cargo run -p xtask -- dist
```

这会构建 `dist` 配置，并写出 `target/dist/gitpoulpi-<version>-<target>.zip`（Windows）或 `.tar.gz`（其他系统）。压缩包内含这些可执行文件、`INSTALL.txt`、`README.md`、`LICENSE` 和 `CHANGELOG.md`。旁边还会写出一个 `.sha256` 文件。

## 更新

标题栏在应用名称和仓库路径之间显示正在运行的版本（`v1.0.0`）。已知有更新的发布版本（或更新正在进行）时，点击它会打开更新面板，里面有版本、GitPoulpi 的安装方式，以及面对新版本该怎么做；否则点击会打开“发行说明”标签页。`Ctrl+N` 再按 `u`，或命令面板（`Ctrl+P`）中的 **更新：显示更新**，总是打开更新面板，而 **帮助：发行说明** 总是打开更新日志。

GitPoulpi 每天在后台到发布其版本的地方（默认是 GitHub）检查是否有更新的版本（`updates.mode`，[设置 > 同步](settings.md#同步)）。有的话，标记会以强调色显示 `v0.2.0 可用`。

面板的做法取决于 GitPoulpi 的安装方式：发布压缩包或 Windows 安装程序会替换自身的文件，包管理器会运行它自己的命令，受管理的安装或容器安装会说明更新如何送达。用 `cargo build` 构建的版本从不更新。完整的说明，包括回退、`auto` 与 `notify` 的区别以及管理员可以锁定什么，见[更新](updates.md)。

## 检查是否正常工作

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi
```

`--version` 会打印版本以及构建所用的 commit。然后请按照[快速上手](getting-started.md)继续。

## 卸载

`gitpoulpi uninstall`（两个安装程序都会运行它）会移除 GitPoulpi 创建的命令别名、PATH 条目、快捷方式和文件管理器菜单项（Windows 上还有它们的 App Paths 键），然后移除发布版安装的程序文件；在没有 `.gitpoulpi-dist` 标记的构建（cargo 构建）上它会拒绝执行，`--keep-files` 则保留程序文件。MCP 代理无法使用它。不用安装程序时，直接删除程序文件即可。GitPoulpi 还会保留以下文件：

| 内容 | 位置 |
|---|---|
| 设置（`config.toml`）和会话 | 你操作系统的配置文件夹。可用 `GITPOULPI_CONFIG_DIR` 更改。 |
| 各仓库的状态 | 默认位于仓库 `.git` 文件夹内的 `gitpoulpi/`（`session.per_repo_state_dir` 可把它改到工作区的 `.gitpoulpi/` 或用户数据文件夹）。 |
| 缓存（头像） | 你操作系统的缓存文件夹。 |
| 令牌和 API 密钥 | 你的 credential helper。**设置 > SSH 与凭据 > 忘记所有用户名和密码**可将其删除。 |

确切路径参见[配置参考](config-reference.md)。
