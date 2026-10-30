[English](../README.md) · [Čeština](../cs/README.md) · [Deutsch](../de/README.md) · [Español](../es/README.md) · [Français](../fr/README.md) · [Italiano](../it/README.md) · [日本語](../ja/README.md) · [한국어](../ko/README.md) · [Nederlands](../nl/README.md) · [Polski](../pl/README.md) · [Português (Brasil)](../pt-BR/README.md) · [Português (Portugal)](../pt-PT/README.md) · [Русский](../ru/README.md) · [Svenska](../sv/README.md) · [Türkçe](../tr/README.md) · [Українська](../uk/README.md) · **简体中文**

# GitPoulpi 用户文档

<p align="center"><img src="../../../assets/logo/gitpoulpi-512.png" alt="GitPoulpi 章鱼" width="128"></p>

GitPoulpi 是一个终端 Git 客户端，拥有可点击的 commit 图、命令面板、托管平台和议题跟踪器集成、可选的 AI 辅助，以及能完成界面所做一切的 CLI 和 MCP 服务器。这些页面按阅读顺序排列：从顶部开始，需要时再跳到参考页面。

![GitPoulpi 主窗口](../images/hero.png)

## 从这里开始

1. **[安装](installation.md)**：你需要什么、发布压缩包、从源码构建。若想要更小的二进制文件，请参见 **[自定义构建](custom-builds.md)**。
2. **[快速上手](getting-started.md)**：你的头五分钟，从启动到第一次 commit。
3. **[界面导览](interface-tour.md)**：窗口中每个区域的名称，附图。

## 日常使用

4. **[使用 Log](log-and-history.md)**：阅读图、搜索、分支、tag、stash、worktree、拖动引用、rebase、cherry-pick、reword 和修订、横幅、pull 和 push、auto-stash。
5. **[stage 与 commit](staging-and-committing.md)**：工作更改、diff、stage hunk 和单行、撰写 commit、commit 撰写器、blame、文件历史和内置编辑器。
6. **[图像](image-view.md)**：Diff 视图和文件视图中的图像、修改前后、缩放、编辑方式。
7. **[解决 merge 冲突](merge-conflicts.md)**：merge 编辑器（A 和 B 并排显示在输出之上）、选取行和块、针对二进制、LFS、已删除、符号链接和子模块冲突的卡片，以及代理如何提出解决方案。
8. **[撤销与重做](undo-redo.md)**：`Ctrl+Z` / `Ctrl+Y`、哪些可以撤销哪些不能、何时拒绝或询问、可回溯多远、如何关闭。
9. **[Git LFS](lfs.md)**：大文件、跟踪、锁，以及未安装 `git-lfs` 时会发生什么。
10. **[子模块](submodules.md)**：子模块部分、更新它们，以及让它们与检出和 pull 保持同步。
11. **[主题](themes.md)**：内置主题、任意主题搭配任意泳道调色板、实时预览、你自己的主题文件、`gitpoulpi theme`。
12. **[工作空间](workspaces.md)**：有名称的仓库分组，各有自己的标签页和可选的默认提交配置文件、标签栏标记、工作空间标签页、全部 fetch、全部 pull、克隆缺失的仓库。
13. **[搜索](search.md)**：一个模糊查找器（`Ctrl+Alt+F`），覆盖工作空间或已打开标签页中所有仓库的分支、tag、worktree、stash、commit、pull/merge request 和议题。
14. **[Radar](radar.md)**：把工作空间（或你已打开的标签页）中的 pull/merge request、议题和进行中的工作汇总在一个列表中：筛选、固定、延后，以及每一行能做什么。在应用内阅读并评审这些请求，带评论讨论、待处理评审以及批准 / 请求更改：**[评审 pull request](review.md)**。
15. **[打开方式](open-in.md)**：标题栏中的列表，可在终端、多路复用器、编辑器或文件管理器中打开活动标签页的文件夹。
16. **[跟随活动标签页的文件夹](following-the-tab-folder.md)**：“退出并 cd 到此”，以及适用于 bash、zsh、fish 和 PowerShell 的 `gitpoulpi shell-init` 包装函数（`gp`）。
    哪个终端最适合 GitPoulpi：**[选择终端](terminals.md)**。

另见：**[分支工作流](branching-workflows.md)**，Git Flow、GitHub Flow、GitLab Flow 和 Trunk-based 预设，或你自己的基础分支、分支名称模式和主题类型（`gitpoulpi flow ...`）。

**[重写历史](history-rewrite.md)**：重写工作台（`Ctrl+G Shift+H`），可跨多个 commit 重写作者、日期、消息和文件内容，支持预览、Lua 脚本和一次撤销。

## 键盘与鼠标

17. **[快捷键](keybindings.md)**：按区域列出的每个按键，以及面板中的每条命令。在应用中，`?` 显示同一份列表，`Ctrl+P` 可找到任意命令。
18. **[鼠标与手势](mouse-and-gestures.md)**：点击、拖动、选择和复制、滚动条和悬停反馈。

## 设置与集成

19. **[设置](settings.md)**：设置界面的每个页面和选项、搜索框、配置，以及如何重新绑定快捷键。
20. **[AI 辅助](ai.md)**：GitPoulpi 自带的 AI（默认关闭）：本地代理（如 `claude -p`）、Anthropic、OpenAI、OpenRouter、Ollama 或 LM Studio；指令；commit 消息、解释、pull request 描述和 commit 撰写器；隐私与故障排除。
21. **[托管平台与议题跟踪器](hosting-integration.md)**：GitHub、GitLab、Gitea 和 Forgejo、Azure DevOps、Bitbucket、Gerrit 以及各议题跟踪器：令牌及其权限范围、连接、关联仓库、pull request 和议题、commit 消息中的引用、SSH 密钥、已知限制。另见 **[本地议题与 Obsidian](local-issues.md)**：以一个 Markdown 笔记文件夹作为议题来源，无需令牌，并可生成 Obsidian Base。

## 自动化

22. **[CLI 参考](cli-reference.md)**：每条 `gitpoulpi` 命令，脚本可使用 `--json`。
23. **[MCP 与代理](mcp-and-agents.md)**：让外部代理（Claude Code、Cursor、Codex 等）操控 GitPoulpi：注册服务器、`.mcp.json`、权限开关及其默认值、每个工具，以及代理如何在你的窗口中向你展示内容。现成的代理技能是 [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md)。

## 参考与帮助

24. **[配置参考](config-reference.md)**：每个 `config.toml` 键、其默认值和作用、各仓库设置、环境变量。
25. **[故障排除与常见问题](troubleshooting.md)**：图标、图的渲染、收不到的按键、卡住的守护进程、速度、LFS 指针。

## 截图

这些页面中的每张图片都是发布版本的真实截图，取自一个带有虚构作者的小型演示仓库。图片位于 [`images/`](../images/)。`scripts/docs-screenshots.sh` 会构建演示仓库并重新截图。截图在 `kitty` 终端中拍摄，该终端会把 commit 图和作者头像绘制为图片。没有图形协议的终端则显示字符图。

## 面向贡献者

[CLAUDE.md](../../../CLAUDE.md)、[架构概览](../../architecture/overview.md)和[路线图](../../roadmap.md)描述了设计和计划内容；[CHANGELOG.md](../../../CHANGELOG.md) 列出了每个版本的变更。

---

GitKraken 是其各自所有者的商标。GitPoulpi 是独立项目，与其所有者没有隶属关系，也未获得其认可或赞助；提及该名称仅用于说明兼容性和灵感来源。
