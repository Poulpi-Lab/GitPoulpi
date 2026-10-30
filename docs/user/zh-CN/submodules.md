[English](../submodules.md) · [Čeština](../cs/submodules.md) · [Deutsch](../de/submodules.md) · [Español](../es/submodules.md) · [Français](../fr/submodules.md) · [Italiano](../it/submodules.md) · [日本語](../ja/submodules.md) · [한국어](../ko/submodules.md) · [Nederlands](../nl/submodules.md) · [Polski](../pl/submodules.md) · [Português (Brasil)](../pt-BR/submodules.md) · [Português (Portugal)](../pt-PT/submodules.md) · [Русский](../ru/submodules.md) · [Svenska](../sv/submodules.md) · [Türkçe](../tr/submodules.md) · [Українська](../uk/submodules.md) · **简体中文**

# 子模块

子模块是嵌套在你的仓库内、固定在某个 commit 的另一个仓库。GitPoulpi 在引用侧栏中列出它们，可通过界面、CLI 或命令面板更新它们，并能让它们与你检出的 commit 保持同步。

## 子模块部分

当仓库有 `.gitmodules` 时，引用侧栏会出现**子模块**部分（位于其他部分下方；没有子模块时不显示）。每个子模块一行：一个状态标记、其路径和一个短 commit。

| 标记 | 状态 | 含义 |
|---|---|---|
| `✓` | 最新 | 已检出到仓库所记录的 commit |
| `↑` | 已修改 | 已检出到另一个 commit（它向前移动了，或者你切换了分支但没有更新它） |
| `○` | 未克隆 | 已在 `.gitmodules` 中声明并有记录，但其文件夹为空：更新它即可克隆 |
| `✗` | 有冲突 | 仓库在该子模块上存在 merge 冲突 |

| 操作 | 鼠标 | 键盘 |
|---|---|---|
| 把子模块作为独立标签页打开 | 双击该行 | 选中它，按 `Enter`（面板：`引用：在标签页中打开子模块`） |
| 菜单：**更新（必要时克隆）**、**初始化**、**在新标签页中打开** | 右键点击该行 | 选中它，按上下文菜单键（默认 `.`） |
| 更新所选子模块（需要时先克隆） | 菜单 | `Ctrl+P`：`引用：更新子模块（所选）` |
| 在 `.git/config` 中注册而不克隆 | 菜单 | `Ctrl+P`：`引用：初始化子模块（所选）` |
| 更新所有子模块 | | `Ctrl+P`：`Git：更新所有子模块` |
| 跳转到该部分 | 点击其标题 | `Ctrl+P`：`导航：聚焦引用 > 子模块` |

更新可能会从子模块的远程仓库获取，因此与其他传输一样，它遵循**设置 > SSH 与凭据 > HTTPS 凭据**。尚未克隆的子模块无法作为标签页打开；状态栏会说明这一点，并指向更新。

## 让子模块保持最新

**设置 > 同步 > 保持子模块最新**（`general.keep_submodules_up_to_date`，默认关闭）。开启后，每次成功的 **checkout** 或 **pull** 之后都会执行 `git submodule update --init --recursive`。仅当某个子模块与仓库现在所记录的内容不一致时才会运行，因此没有子模块的仓库不会变慢。如果该更新失败，checkout 或 pull 仍然已经完成，错误信息也会这样说明。

## 命令行

```
gitpoulpi submodule list                       # path, state, url (--json: the full list)
gitpoulpi submodule init [<path>...]           # register, do not clone
gitpoulpi submodule update [<path>...] [--init] [--recursive]
```

不带路径表示所有子模块。这三个命令同样是 MCP 工具（`submodule_list` 是读取；`init` 和 `update` 是变更操作，遵循 `[mcp]` 的 `allow_mutations`）。

## 注意事项

- 只列出顶层；子模块自己的子模块会在你把它作为标签页打开时显示（或使用 `--recursive` 更新它们）。
- 较新的 git 会拒绝通过 `file` 传输克隆*本地路径*子模块，除非 `protocol.file.allow` 允许。这只与 URL 为本地文件夹的子模块有关；如果你使用它们，请在全局 git 配置中设置。
