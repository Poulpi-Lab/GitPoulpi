[English](../../README.md) · [Čeština](README.cs.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · **简体中文**

# GitPoulpi

<p align="center"><img src="../../assets/logo/gitpoulpi-512.png" alt="GitPoulpi 章鱼" width="128"></p>

**一个用起来像桌面应用的终端 Git 客户端。** GitPoulpi 为您提供一张可以点击、拖动和右键操作的完整 commit 图、一个以键盘为先且能触达所有功能的命令面板、与代码托管平台（GitHub、GitLab、Gitea、Forgejo、Azure DevOps、Bitbucket、Gerrit，云端或自托管）和议题跟踪器（Jira、Trello、Linear、YouTrack、Redmine，或一个 Markdown 笔记文件夹）的集成，以及同样的操作所对应的可脚本化 CLI 和 MCP 服务器，因此 AI 代理可以像您一样操控它。GitPoulpi 受 GitKraken 启发，并用 Rust 从头重写：原生二进制文件，磁盘和内存占用都很小，没有 Electron，无需账号，无遥测。它运行在您喜欢的终端里（[哪一个？](../../docs/user/zh-CN/terminals.md)），因为我们中有些人就住在里面。

![GitPoulpi 显示带有分支、tag 和一个 stash 的 commit 图，左侧是引用侧边栏，右侧是工作更改和 commit 表单](../../docs/user/images/hero.png)

- **鼠标与键盘，同等对待。** 点击、双击、右键，并把引用拖放到 commit 上；或者按 `Ctrl+P` 输入您想要的操作。每个鼠标手势都有对应的键盘操作，每条命令都在面板中。
- **真正的 commit 图。** 分支、merge、tag、stash 和工作副本都在同一个可滚动的 Log 中，在支持的终端里以像素绘制，在其他地方则以字符绘制。提供多种深色和浅色配色主题、作者头像和搜索。
- **精确 stage。** 直接在 diff 中对文件、文件夹、hunk 或单行进行 stage、unstage 或丢弃。支持内联或并排显示，并带有缩略图。
- **安全地编辑历史。** 改写或修订任意 commit（消息、作者、日期），通过审阅界面进行交互式 rebase，一次重写多个 commit（先预览，一次撤销即可还原），cherry-pick 多个 commit，快进，创建 worktree 和补丁。
- **内置托管平台和议题跟踪器。** 浏览、检出和创建 pull request 与 merge request（GitHub、GitLab、Gitea、Forgejo、Azure DevOps、Bitbucket），并审阅 Gerrit 的 change。浏览和创建议题、工作项和卡片（还支持 Jira、Trello、Linear、YouTrack、Redmine 和本地 Markdown 笔记），也可以用看板（Kanban）查看。从您的账号克隆并管理 SSH 密钥。
- **面向代理的 CLI 和 MCP。** `gitpoulpi <command> --json` 涵盖界面所做的一切，`gitpoulpi mcp serve` 则将其作为 MCP 工具提供。每个仓库一个后台守护进程，让所有前端保持实时同步。
- **可选的 AI，绝非必需。** 默认关闭，无需账号：使用您自己的 API 密钥或本地代理 CLI（例如 `claude -p`）。可以生成 commit 消息、解释更改、撰写 pull request 描述，把您的工作更改拆分为多个 commit，以及起草历史重写规则。
- **Git LFS、子模块、worktree、补丁、blame 和文件历史、图像 diff、内置编辑器。**

## 截图

| | |
|---|---|
| ![采用 Material 泳道配色并启用行着色的 Log 图](../../docs/user/images/theme-material.png) **commit 图**，此处为 Material 泳道配色加行着色。 | ![已修改文件的 diff，每个 hunk 上都有 stage Hunk 和丢弃 Hunk 按钮](../../docs/user/images/diff.png) **diff**，可按 hunk 或按行 stage 和丢弃。 |
| ![按 stash 一词筛选的命令面板](../../docs/user/images/palette.png) **命令面板**（`Ctrl+P`）可触达每条命令。 | ![commit 撰写器为工作更改提出四个 commit，等待审阅](../../docs/user/images/composer.png) **commit 撰写器**会在 commit 任何内容之前，先让您审阅 AI 提出的 commit。 |
| ![commit 的右键菜单，含 cherry-pick、revert、reset、rebase、分支和 tag 操作](../../docs/user/images/context-menu.png) **右键点击**任何内容即可看到其操作。 | ![设置中的 AI 页面，含配置、各操作的指令和启用开关](../../docs/user/images/settings-ai.png) **设置**，带搜索，包括可选的 AI 页面。 |

更多带说明的图片见[用户文档](../../docs/user/zh-CN/README.md)。

## 安装

您需要系统自带的 `git`；`git-lfs` 是可选的。

### Linux 和 macOS（`install.sh`）

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

### Homebrew（macOS、Linux）

```
brew install Poulpi-Lab/gitpoulpi/gitpoulpi
```

### apt（Debian、Ubuntu）

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

### Scoop（Windows）

```
scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi
scoop install gitpoulpi
```

### Windows 安装程序

从[最新发行版](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest)下载
`gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` 并运行。
管理员可以使用 MSI：

```
msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn
```

### Docker

```
docker pull poulpilab/gitpoulpi
```

参见[在 Docker 中运行](../user/zh-CN/docker.md)。

### cargo

```
cargo binstall gitpoulpi-cli          # prebuilt
cargo install --locked gitpoulpi-cli  # compiles
```

### 发布压缩包

从[最新发行版](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest)下载，
解压后按其中的 `INSTALL.txt` 操作。

### 从源码安装

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

自行构建发布压缩包、更小的自定义构建以及各操作系统的说明：
[docs/user/zh-CN/installation.md](../user/zh-CN/installation.md) 和
[docs/user/zh-CN/custom-builds.md](../user/zh-CN/custom-builds.md)。

## 快速开始

```
cd path/to/your/repository
gitpoulpi
```

- `?` 打开列出所有按键和手势的帮助，`Ctrl+P` 打开命令面板，`Ctrl+,` 打开设置，`q` 退出。
- 点击一个 commit 查看它的更改；点击 Log 的顶行（工作更改）来 stage 文件并 commit。
- 在支持的终端（kitty、Ghostty、WezTerm、iTerm2、通过 sixel 的 Windows Terminal 和 foot）中，图以像素绘制。在其他终端（普通的 Windows 控制台、tmux）中，会自动回退为字符。

分步说明见[快速上手](../../docs/user/zh-CN/getting-started.md)，[界面导览](../../docs/user/zh-CN/interface-tour.md)则为屏幕上的每个区域命名。

```
gitpoulpi --json status            # the CLI, scriptable
gitpoulpi --json log --limit 20
claude mcp add gitpoulpi -- gitpoulpi mcp serve   # give an AI agent the same tools
```

AI 是可选的，默认关闭：在 *设置 > AI* 中开启 GitPoulpi 自带的 AI（[AI 辅助](../../docs/user/zh-CN/ai.md)）；让外部代理通过 [MCP 与代理](../../docs/user/zh-CN/mcp-and-agents.md)（权限、所有工具）以及代理技能 [`skills/gitpoulpi/SKILL.md`](../../skills/gitpoulpi/SKILL.md) 来操控 GitPoulpi。

## 文档

完整的用户文档从 **[docs/user/zh-CN/README.md](../../docs/user/zh-CN/README.md)** 开始：

| | |
|---|---|
| [快速上手](../../docs/user/zh-CN/getting-started.md) | 安装、打开仓库、完成第一次 commit |
| [界面导览](../../docs/user/zh-CN/interface-tour.md) | 窗口的每个区域，逐一命名 |
| [使用 Log](../../docs/user/zh-CN/log-and-history.md) | 图、引用、分支、tag、stash、rebase、cherry-pick |
| [stage 与 commit](../../docs/user/zh-CN/staging-and-committing.md) | 状态、diff、hunk 和行、撰写器、编辑器 |
| [设置](../../docs/user/zh-CN/settings.md) | 每个设置页面和选项 |
| [AI 辅助](../../docs/user/zh-CN/ai.md) | 配置内置 AI、每个 AI 操作、隐私 |
| [托管平台与议题跟踪器](../../docs/user/zh-CN/hosting-integration.md) | GitHub、GitLab、Gitea、Forgejo、Azure DevOps、Bitbucket、Gerrit、Jira、Trello、Linear、YouTrack、Redmine：令牌、关联仓库、pull request 和 merge request |
| [CLI](../../docs/user/zh-CN/cli-reference.md) 和 [MCP](../../docs/user/zh-CN/mcp-and-agents.md) | 用脚本操控，或让代理来操控（权限、所有工具） |
| [快捷键](../../docs/user/zh-CN/keybindings.md)（以及面板中的每条命令）、[鼠标](../../docs/user/zh-CN/mouse-and-gestures.md)、[配置](../../docs/user/zh-CN/config-reference.md)、[LFS](../../docs/user/zh-CN/lfs.md)、[撤销](../../docs/user/zh-CN/undo-redo.md)、[故障排除](../../docs/user/zh-CN/troubleshooting.md) | 参考与帮助 |

开发者：[CLAUDE.md](../../CLAUDE.md) 和 `docs/architecture/overview.md` 描述了各个 crate 和设计；[CHANGELOG.md](../../CHANGELOG.md) 列出了每个版本的内容。

## 许可证

[MIT](../../LICENSE)。

## 致谢与第三方许可证

GitPoulpi 基于开源库构建，并使用已发布主题的颜色值。它们的许可证和致谢（包括主题作者）见 [THIRD-PARTY-LICENSES.txt](../../THIRD-PARTY-LICENSES.txt)。该文件随每个版本一同发布，“关于”窗口和 `gitpoulpi licenses` 也会显示它。

## 商标

GitKraken 是其各自所有者的商标。GitPoulpi 是独立项目，与其所有者没有隶属关系，也未获得其认可或赞助；提及该名称仅用于说明兼容性和灵感来源。
