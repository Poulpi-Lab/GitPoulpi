[English](../keybindings.md) · [Čeština](../cs/keybindings.md) · [Deutsch](../de/keybindings.md) · [Español](../es/keybindings.md) · [Français](../fr/keybindings.md) · [Italiano](../it/keybindings.md) · [日本語](../ja/keybindings.md) · [한국어](../ko/keybindings.md) · [Nederlands](../nl/keybindings.md) · [Polski](../pl/keybindings.md) · [Português (Brasil)](../pt-BR/keybindings.md) · [Português (Portugal)](../pt-PT/keybindings.md) · [Русский](../ru/keybindings.md) · [Svenska](../sv/keybindings.md) · [Türkçe](../tr/keybindings.md) · [Українська](../uk/keybindings.md) · **简体中文**

# 键盘快捷键

本页列出**默认**按键。其中大多数都可以重新绑定。应用始终显示当前生效的按键：在命令面板、`?` 帮助、状态栏、上下文菜单和按钮提示中都是如此。当你更改某个按键后，这些位置会显示新的按键。关于鼠标，参见[鼠标与手势](mouse-and-gestures.md)。

## 重新绑定

打开**设置 > 键盘与鼠标**。该页面在[设置](settings.md#键盘与鼠标)中有说明。更改会立即生效，并保存在 `config.toml` 的 `[keybindings]` 表中。你也可以手动编辑该表：

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # tmux and GNU screen often use Ctrl+B and Ctrl+A
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

语法、冲突规则和警告说明见[配置参考](config-reference.md)中的 `[keybindings]` 一节。简而言之：

- 出错时会在状态栏提示，并保留默认按键。
- 同一上下文中的两个操作不能使用同一组合键，否则会被拒绝。
- 前缀键需要 `Ctrl` 或 `Alt`。
- 窗格组合键遮盖全局组合键时，只会给出警告。

下表列出了你可以重新绑定的所有按键。有些按键是固定的，无法重新绑定：列表移动键（`j` `k` `h` `l`、方向键、`PageUp` / `PageDown`、`Home` / `End`）、`Enter`、`Esc`、`Tab`、数字键（数量前缀）、文本编辑键，以及弹出窗口中的 `Enter` / `Tab` / 方向键。面板中有 `应用：键位绑定…` 和 `键位绑定：全部重置…`。

## 默认键位

下表列出了你可以重新绑定的每个按键，包括在 `[keybindings.<context>]` 中使用的 id 及其默认值。该表由应用生成（有测试保证其保持最新）。前缀键之后的按键与前缀一起显示（`Ctrl+G s`）。

<!-- keymap:begin -->
| 范围 | 操作 id | 作用 | 默认按键 |
|---|---|---|---|
| `global` | `git_prefix` | Git 操作前缀 | `Ctrl+G` |
| `global` | `nav_prefix` | 导航前缀 | `Ctrl+N` |
| `global` | `palette` | 命令面板 | `Ctrl+P` |
| `global` | `settings` | 设置 | `Ctrl+,` |
| `global` | `home` | 主页（仓库管理） | `Ctrl+O` |
| `global` | `clone` | 克隆仓库 | `Ctrl+Shift+N` |
| `global` | `new_tab` | 新标签页 | `Alt+T` |
| `global` | `workspace_switch` | 切换工作空间 | `Alt+W` |
| `global` | `radar` | 打开 Radar | `Alt+R` |
| `global` | `workspaces` | 打开工作空间 | `Alt+J` |
| `global` | `tab_menu` | 标签页菜单 | `Ctrl+T` |
| `global` | `tab_open_in_popup` | 打开方式（列表） | `Alt+O` |
| `global` | `tab_open_in_default` | 用默认方式打开 | `Ctrl+Alt+O` |
| `global` | `close_tab` | 关闭标签页 | `Ctrl+W` |
| `global` | `next_tab` | 下一个标签页 | `Ctrl+PageDown` |
| `global` | `prev_tab` | 上一个标签页 | `Ctrl+PageUp` |
| `global` | `help` | 帮助 | `?` |
| `global` | `quit` | 退出 | `q` |
| `global` | `refresh` | 刷新 | `r` |
| `global` | `refresh_all` | 全部刷新（fetch、pull request、议题） | `F5` |
| `global` | `find_in_log` | 搜索 Commit 图 | `f` |
| `global` | `workspace_search` | 在工作空间中搜索 | `Ctrl+Alt+F` |
| `global` | `select_tab` | 选择标签页 | `t` |
| `global` | `undo` | 撤销上一步操作 | `Ctrl+Z` |
| `global` | `redo` | 重做已撤销的操作 | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `global` | `undo_history` | 打开撤销历史 | `Alt+Z` |
| `global` | `commit_column` | 显示或隐藏 Commit 列 | `Alt+C` |
| `global` | `undo_offer` | 将分支恢复到被阻止操作完成后的位置 | `Alt+Shift+Z` |
| `global` | `context_menu` | 上下文菜单 | `.`, `Menu`, `Shift+F10` |
| `git_prefix` | `compose_commit` | 撰写 Commit 消息 | `Ctrl+G c` |
| `git_prefix` | `commit_now` | 立即 Commit staged 的更改 | `Ctrl+G Enter` |
| `git_prefix` | `ai_compose` | AI：将更改拆分为 Commit | `Ctrl+G Shift+C` |
| `git_prefix` | `ai_explain` | AI：解释更改 | `Ctrl+G w` |
| `git_prefix` | `stash_message` | 带消息 Stash | `Ctrl+G m` |
| `git_prefix` | `ai_commit_message` | AI：撰写 Commit 消息 | `Ctrl+G i` |
| `git_prefix` | `stage_all` | stage 所有更改 | `Ctrl+G a` |
| `git_prefix` | `discard_all` | 丢弃所有更改 | `Ctrl+G x` |
| `git_prefix` | `stage_file` | stage 所选文件 | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | unstage 所选文件 | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | 打开关联的 pull request | `Ctrl+G g` |
| `git_prefix` | `push_tags` | Push 所有 Tag | `Ctrl+G Shift+U` |
| `git_prefix` | `fetch_all` | Fetch 所有远程 | `Ctrl+G f` |
| `git_prefix` | `new_tag` | 新建 Tag | `Ctrl+G t` |
| `git_prefix` | `new_branch` | 新建分支 | `Ctrl+G b` |
| `git_prefix` | `stash_push` | Stash 更改 | `Ctrl+G Shift+T` |
| `git_prefix` | `cherry_pick` | Cherry-pick 所选 Commit | `Ctrl+G y` |
| `git_prefix` | `revert` | Revert 所选 Commit | `Ctrl+G v` |
| `git_prefix` | `edit_message` | 编辑 Commit 消息 | `Ctrl+G e` |
| `git_prefix` | `amend` | 修订所选 commit | `Ctrl+G Shift+E` |
| `git_prefix` | `edit_author` | 修订所选 commit 的作者 | `Ctrl+G Shift+A` |
| `git_prefix` | `reset_mixed` | Reset（Mixed）到所选 Commit | `Ctrl+G Shift+R` |
| `git_prefix` | `rebase_onto` | Rebase 到所选分支 | `Ctrl+G r` |
| `git_prefix` | `stash_pop` | Pop Stash | `Ctrl+G o` |
| `git_prefix` | `delete_ref` | 删除所选引用或 Drop Stash | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | 打开 LFS 菜单 | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | 打开 Flow 菜单 | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS：Fetch 所有对象 | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS：检出所有文件 | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS：Pull 对象 | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS：Push 所有对象 | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS：清理本地对象 | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | 打开历史重写工作台 | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | 折叠或恢复引用窗格 | `Ctrl+N h` |
| `nav_prefix` | `collapse_status` | 折叠或恢复状态窗格 | `Ctrl+N l` |
| `nav_prefix` | `settings` | 打开设置 | `Ctrl+N ,` |
| `nav_prefix` | `about` | 打开“关于” | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | 打开活动日志 | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | 打开全局活动日志 | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | 显示后台任务 | `Ctrl+N Shift+J` |
| `nav_prefix` | `updates` | 显示更新面板 | `Ctrl+N u` |
| `nav_prefix` | `help` | 切换帮助 | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | 选择标签页 | `Ctrl+N t` |
| `nav_prefix` | `change_theme` | 更改主题 | `Ctrl+N y` |
| `nav_prefix` | `change_language` | 更改语言 | `Ctrl+N g` |
| `nav_prefix` | `change_graph_color_theme` | 更改图配色主题 | `Ctrl+N Shift+Y` |
| `nav_prefix` | `toggle_graph_row_tints` | 切换图行着色 | `Ctrl+N Shift+T` |
| `nav_prefix` | `grow_section` | 增大引用分区 | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | 缩小引用分区 | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | 增大 Commit 消息区 | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | 缩小 Commit 消息区 | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | 加宽引用窗格 | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | 收窄引用窗格 | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | 加宽状态窗格 | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | 收窄状态窗格 | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | 代理：显示待处理的视图 | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | 代理：返回（或关闭待处理的视图） | `Ctrl+N Backspace` |
| `nav_prefix` | `arrange_dialog` | 调整当前对话框（移动、改变大小） | `Ctrl+N m` |
| `refs` | `filter` | 按名称筛选引用 | `/` |
| `refs` | `solo` | 单独显示所选项 | `o` |
| `refs` | `clear_solo` | 停止单独显示 | `Shift+O` |
| `refs` | `toggle_hierarchy` | 切换分支层级视图 | `y` |
| `refs` | `show_all` | 显示所有隐藏的引用 | `Shift+H` |
| `refs` | `reveal` | 在图中定位 | `g` |
| `refs` | `mark_drop` | 标记为拖放来源 | `m` |
| `refs` | `drop_marked` | 将标记的引用放到此处 | `Shift+M` |
| `refs` | `expand_all` | 展开所有文件夹 | `=` |
| `refs` | `collapse_all` | 折叠所有文件夹 | `-` |
| `refs` | `expand_below` | 展开该文件夹及其子文件夹 | `*`, `Shift+Right` |
| `refs` | `collapse_below` | 折叠该文件夹及其子文件夹 | `_`, `Shift+Left` |
| `log` | `toggle_file_mode` | 文件：路径或树 | `m` |
| `log` | `view_all_files` | 文件：查看所有文件 | `a` |
| `log` | `compare_selection` | 比较所选行 | `v` |
| `log` | `compare_swap` | 交换比较的两端 | `x` |
| `log` | `compare_next` | 转到下一个比较行 | `n` |
| `log` | `pick_column` | 选择要移动或调整宽度的列 | `Shift+C` |
| `log` | `scroll_left` | 向左滚动各列 | `Shift+Left` |
| `log` | `scroll_right` | 向右滚动各列 | `Shift+Right` |
| `log` | `copy_mermaid` | 将 Commits 复制为 Mermaid gitGraph | `Shift+Y` |
| `log_columns` | `next` | 选择下一列 | `Tab` |
| `log_columns` | `prev` | 选择上一列 | `Shift+BackTab` |
| `log_columns` | `move_left` | 将该列左移 | `<` |
| `log_columns` | `move_right` | 将该列右移 | `>` |
| `log_columns` | `wider` | 加宽该列 | `]` |
| `log_columns` | `narrower` | 收窄该列 | `[` |
| `log_columns` | `reset_width` | 重置各列宽度 | `0` |
| `log_columns` | `fit` | 按内容调整该列宽度 | `=` |
| `log_columns` | `fit_all` | 按内容调整所有列宽度 | `+` |
| `log_columns` | `layout` | 横向滚动或适应窗格 | `s` |
| `log_files` | `collapse_all` | 折叠或展开所有文件夹 | `z` |
| `log_files` | `filter` | 筛选已更改的文件 | `/` |
| `log_files` | `expand_all` | 展开所有文件夹 | `=` |
| `log_files` | `collapse_all_folders` | 折叠所有文件夹 | `-` |
| `log_files` | `expand_below` | 展开该文件夹及其子文件夹 | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | 折叠该文件夹及其子文件夹 | `_`, `Shift+Left` |
| `status` | `compose` | 聚焦 Commit 消息 | `c` |
| `status` | `toggle_file_mode` | 文件：路径或树 | `m` |
| `status` | `view_all_files` | 文件：查看所有文件 | `a` |
| `status` | `amend` | 切换修订 | `Shift+A` |
| `status` | `skip_hooks` | 切换“跳过钩子” | `Shift+H` |
| `status` | `collapse_all` | 折叠或展开所有文件夹 | `z` |
| `status` | `stage_toggle` | stage 或 unstage 所选行 | `Ctrl+Space` |
| `status` | `open_diff` | 打开所选文件的 Diff | `d` |
| `status` | `expand_all` | 展开所有文件夹 | `=` |
| `status` | `collapse_all_folders` | 折叠所有文件夹 | `-` |
| `status` | `expand_below` | 展开该文件夹及其子文件夹 | `*`, `Shift+Right` |
| `status` | `collapse_below` | 折叠该文件夹及其子文件夹 | `_`, `Shift+Left` |
| `home` | `favorite` | 收藏或取消收藏仓库 | `f`, `*` |
| `home` | `close_repo` | 关闭、移除或取消收藏该行 | `x`, `Delete` |
| `home` | `browse` | 浏览仓库 | `b` |
| `home` | `clone` | 克隆仓库 | `c` |
| `home` | `init` | 初始化仓库 | `i` |
| `home` | `collapse_all` | 折叠所有分区 | `-` |
| `home` | `expand_all` | 展开所有分区 | `+`, `=` |
| `home` | `search` | 搜索仓库 | `/` |
| `home` | `workspace_new` | 新建工作空间 | `n` |
| `home` | `workspace_edit` | 编辑工作空间 | `e` |
| `home` | `workspace_from_tabs` | 从打开的标签页新建工作空间 | `a` |
| `home` | `workspace_fetch` | Fetch 工作空间全部仓库 | `Shift+F` |
| `home` | `workspace_pull` | Pull 工作空间全部仓库 | `Shift+P` |
| `home` | `workspace_clone_missing` | 克隆缺失的仓库 | `m` |
| `home` | `workspace_locate` | 定位缺失的仓库 | `l` |
| `refs` | `hosting_filter` | pull request / 议题：选择筛选 | `v` |
| `refs` | `hosting_edit` | pull request：在托管平台上编辑 | `e` |
| `refs` | `new_pull_request` | pull request：创建 | `n` |
| `refs` | `add_remote` | 远程：添加远程 | `+` |
| `refs` | `remotes` | 分支：编辑其远程 | `u` |
| `diff` | `close` | 关闭 Diff | `q` |
| `diff` | `edit` | 在工作区中编辑 | `e` |
| `diff` | `lfs_raw` | LFS：卡片或原始指针 | `r` |
| `diff` | `copy` | 复制所选内容或行 | `y`, `Ctrl+C` |
| `diff` | `stage_hunk` | stage 或 unstage hunk | `Space` |
| `diff` | `stage_lines` | stage 或 unstage 所选行 | `Enter` |
| `diff` | `discard_hunk` | 丢弃 hunk | `x` |
| `diff` | `discard_lines` | 丢弃所选行 | `Shift+X` |
| `diff` | `select_lines` | 开始或取消行选择 | `v` |
| `diff` | `next_hunk` | 下一个 hunk | `n` |
| `diff` | `prev_hunk` | 上一个 hunk | `p` |
| `diff` | `toggle_layout` | 布局：统一或并排 | `m` |
| `diff` | `toggle_hunks_only` | 仅 hunk 或整个文件 | `Shift+H` |
| `diff` | `toggle_minimap` | 切换缩略图 | `Shift+M` |
| `diff` | `choose_encoding` | 选择编码 | `Shift+C` |
| `diff` | `toggle_wrap` | 切换自动换行 | `w` |
| `diff` | `toggle_whitespace` | 显示或隐藏仅空白字符的更改 | `s` |
| `diff` | `merge_editor` | 打开 merge 编辑器（冲突文件） | `Shift+E` |
| `file_insight` | `close` | 关闭文件视图 | `q` |
| `file_insight` | `edit` | 在工作区中编辑 | `e` |
| `file_insight` | `copy` | 复制所选内容或行 | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | 跳到最后一行 | `Shift+G` |
| `file_insight` | `open_commit` | 转到该行的 Commit | `Enter` |
| `history` | `show_in_log` | 在 Log 中显示 | `Enter` |
| `history` | `copy_sha` | 复制 commit 的 SHA | `y` |
| `history` | `copy_summary` | 复制摘要 | `Shift+Y` |
| `history` | `show_diff` | 显示此 commit 的 diff | `d`, `Alt+1` |
| `history` | `open_file` | 打开此 commit 时的文件 | `o`, `Alt+2` |
| `history` | `blame` | 在此 commit 处 Blame | `b`, `Alt+3` |
| `history` | `compare_working_tree` | 与工作区比较 | `c` |
| `history` | `restore` | 恢复为此版本 | `Shift+R` |
| `history` | `range_up` | 向上扩展选择 | `Shift+Up` |
| `history` | `range_down` | 向下扩展选择 | `Shift+Down` |
| `history` | `switch_focus` | 在列表和 diff 之间切换 | `Tab` |
| `history` | `widen_list` | 加宽列表 | `Alt+Right` |
| `history` | `narrow_list` | 收窄列表 | `Alt+Left` |
| `history` | `reset_split` | 重置列表宽度 | `Alt+0` |
| `commit_column` | `scroll_left` | 向左滚动列 | `Shift+Left` |
| `commit_column` | `scroll_right` | 向右滚动列 | `Shift+Right` |
| `commit_column` | `next_part` | 调整下一栏 | `Tab` |
| `commit_column` | `move_left` | 将该栏左移 | `<` |
| `commit_column` | `move_right` | 将该栏右移 | `>` |
| `commit_column` | `wider_part` | 加宽该栏 | `]` |
| `commit_column` | `narrower_part` | 收窄该栏 | `[` |
| `commit_column` | `reset_widths` | 重置各栏宽度 | `0` |
| `commit_column` | `fit` | 按内容调整该列宽度 | `=` |
| `commit_column` | `fit_all` | 按内容调整所有列宽度 | `+` |
| `file_insight` | `blame_card` | Blame：显示或隐藏详情卡片 | `i` |
| `file_insight` | `blame_next` | Blame：下一个 hunk | `n` |
| `file_insight` | `blame_prev` | Blame：上一个 hunk | `p` |
| `file_insight` | `blame_next_same` | Blame：同一 commit 的下一个 hunk | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame：同一 commit 的上一个 hunk | `Shift+P` |
| `file_insight` | `blame_goto` | Blame：跳到行 | `g` |
| `file_insight` | `blame_copy_sha` | Blame：复制 commit SHA | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame：显示该 commit 对此文件的 diff | `d` |
| `file_insight` | `blame_parent` | Blame：对该 commit 的父级做 Blame | `b` |
| `file_insight` | `blame_history` | Blame：打开文件历史 | `Shift+H` |
| `file_insight` | `blame_wider` | Blame：加宽注释列 | `>` |
| `file_insight` | `blame_narrower` | Blame：收窄注释列 | `<` |
| `image` | `zoom_in` | 放大图片 | `+`, `=` |
| `image` | `zoom_out` | 缩小图片 | `-` |
| `image` | `fit` | 使图片适应视图 | `0` |
| `image` | `actual_size` | 以实际大小显示图片 | `1` |
| `image` | `flip_side` | 显示另一侧（修改前 / 修改后） | `Tab` |
| `image` | `widen_before` | 加宽“修改前”窗格 | `Alt+Right` |
| `image` | `widen_after` | 加宽“修改后”窗格 | `Alt+Left` |
| `image` | `reset_split` | 重置前后分栏 | `Alt+0` |
| `image` | `edit_in` | 打开方式… | `Shift+O` |
| `image` | `edit_default` | 用默认应用打开 | `o` |
| `editor` | `save` | 保存 | `Ctrl+S` |
| `editor` | `undo` | 撤销 | `Ctrl+Z` |
| `editor` | `redo` | 重做 | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | 查找 | `Ctrl+F` |
| `editor` | `find_next` | 查找下一个（Shift：上一个） | `F3` |
| `editor` | `paste` | 粘贴 | `Ctrl+V` |
| `editor` | `toggle_minimap` | 切换缩略图 | `Alt+M` |
| `editor_prompt` | `save` | 未保存的更改：保存并关闭 | `s`, `Shift+S` |
| `editor_prompt` | `discard` | 未保存的更改：放弃并关闭 | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | 未保存的更改：继续编辑 | `c`, `Shift+C` |
| `merge` | `next_conflict` | 下一个冲突 | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | 上一个冲突 | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | 下一个冲突文件 | `Alt+PageDown` |
| `merge` | `prev_file` | 上一个冲突文件 | `Alt+PageUp` |
| `merge` | `next_pane` | 聚焦下一个窗格 | `F6` |
| `merge` | `prev_pane` | 聚焦上一个窗格 | `Shift+F6` |
| `merge` | `toggle_panes` | 显示或隐藏侧面板（引用和状态） | `F9` |
| `merge` | `toggle_base` | 显示或隐藏基础窗格 | `Alt+D` |
| `merge` | `toggle_full_file` | 切换整个文件的 diff | `Alt+F` |
| `merge` | `undo` | 撤销（选择和输入） | `Ctrl+Z` |
| `merge` | `redo` | 重做（选择和输入） | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | 加宽当前（A）窗格 | `Alt+Right` |
| `merge` | `widen_incoming` | 加宽传入（B）窗格 | `Alt+Left` |
| `merge` | `reset_panes` | 重置窗格大小 | `Alt+0` |
| `merge` | `save` | 保存但不标记为已解决 | `Alt+S` |
| `merge` | `save_resolve` | 保存并标记为已解决 | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ 建议解决方案（AI） | `Alt+I` |
| `merge` | `unpick` | 取消此冲突的选择（恢复为未解决） | `Alt+U` |
| `merge` | `reset_all` | 将整个文件重置为原始冲突 | `Alt+R` |
| `merge` | `focus_a` | 聚焦当前（A）窗格 | `Alt+1` |
| `merge` | `focus_b` | 聚焦传入（B）窗格 | `Alt+2` |
| `merge` | `focus_output` | 聚焦输出窗格 | `Alt+3` |
| `merge` | `focus_base` | 聚焦基础窗格 | `Alt+4` |
| `merge_sides` | `toggle_line` | 选取或取消选取该行 | `Space` |
| `merge_sides` | `toggle_chunk` | 选取或取消选取整个块 | `Enter` |
| `merge_sides` | `take_a` | 仅采用当前（A） | `a` |
| `merge_sides` | `take_b` | 仅采用传入（B） | `b` |
| `merge_sides` | `take_both` | 两者都采用，先 A 后 B | `c` |
| `merge_sides` | `take_both_reversed` | 两者都采用，先 B 后 A | `Shift+C` |
| `merge_sides` | `clear` | 此冲突都不采用 | `x` |
| `merge_sides` | `unpick` | 取消此冲突的选择（恢复为未解决） | `u` |
| `merge_sides` | `reset_all` | 将整个文件重置为原始冲突 | `Shift+U` |
| `merge_sides` | `swap_order` | 交换两侧的顺序 | `s` |
| `merge_sides` | `next_conflict` | 下一个冲突 | `n` |
| `merge_sides` | `prev_conflict` | 上一个冲突 | `Shift+N` |
| `merge_sides` | `take_all_a` | 全部采用当前（A） | `Shift+A` |
| `merge_sides` | `take_all_b` | 全部采用传入（B） | `Shift+B` |
| `merge_sides` | `toggle_base` | 显示或隐藏基础窗格 | `d` |
| `merge_sides` | `scroll_left` | 向左滚动窗格 | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | 向右滚动窗格 | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | 切换整个文件的 diff | `f` |
| `merge_sides` | `cycle_height` | 循环切换输出高度 | `z` |
| `merge_card` | `take_a` | 原样保留 A（当前） | `a` |
| `merge_card` | `take_b` | 原样采用 B（传入） | `b` |
| `merge_card` | `take_base` | 采用基础版本（共同祖先） | `o` |
| `merge_card` | `keep` | 保留剩下的文件 | `k` |
| `merge_card` | `delete` | 接受删除 | `Shift+D` |
| `merge_card` | `external_tool` | 在外部 merge 工具中打开 | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit（停下来修订） | `e` |
| `rebase` | `autosquash` | 切换 Autosquash | `a` |
| `rebase` | `start` | 开始计划 | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | 重置计划 | `Ctrl+R` |
| `rebase` | `split` | 拆分该行的 Commit | `t` |
| `rebase` | `move_up` | 上移该行 | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | 下移该行 | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | 创建 Commit | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | 切换按行选择 | `v` |
| `split` | `move_up` | 上移该 Commit | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | 下移该 Commit | `Alt+Down`, `Shift+J` |
| `split` | `reset` | 重置拆分 | `Shift+R` |
| `rewrite` | `preview` | 预览重写 | `F5` |
| `rewrite` | `apply` | 应用重写 | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | 取消预览 | `Ctrl+X` |
| `rewrite` | `add_rule` | 添加规则 | `a`, `Insert` |
| `rewrite` | `toggle_rule` | 启用或停用规则 | `Space` |
| `rewrite` | `move_up` | 上移规则 | `Alt+Up` |
| `rewrite` | `move_down` | 下移规则 | `Alt+Down` |
| `rewrite` | `delete_rule` | 删除规则 | `d`, `Delete` |
| `rewrite` | `restore_rule` | 恢复已删除的规则 | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | 复制规则 | `Ctrl+D` |
| `rewrite` | `next_tab` | 下一个预览标签页 | `Alt+Right` |
| `rewrite` | `prev_tab` | 上一个预览标签页 | `Alt+Left` |
| `rewrite` | `save_shared` | 将方案保存到仓库 | `Alt+S` |
| `rewrite` | `save_mine` | 仅为我保存方案 | `Alt+M` |
| `rewrite` | `load` | 加载方案 | `Alt+L` |
| `rewrite` | `name` | 重命名方案 | `F2` |
| `rewrite` | `protected_all` | 包含所有受保护分支 | `Alt+A` |
| `rewrite` | `protected_none` | 排除所有受保护分支 | `Alt+U` |
| `rewrite` | `check_tools` | 重新检查工具 | `F6` |
| `rewrite` | `analyze` | 分析历史大小 | `F7` |
| `rewrite` | `host_howto` | 说明主机的分支保护 | `Alt+H` |
| `rewrite` | `ai_rules` | 用 AI 创建重写规则 | `Alt+I` |
| `reword` | `apply` | 应用新消息 | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | 深入修改历史 | `Alt+H` |
| `composer` | `commit` | 从撰写器 Commit | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | 插入议题引用 | `Alt+R` |
| `md_editor` | `preview` | Markdown 编辑器：切换编写与预览 | `Alt+P` |
| `md_editor` | `split` | Markdown 编辑器：切换分栏视图 | `Alt+L` |
| `md_editor` | `submit` | Markdown 编辑器：发送文本 | `Ctrl+Enter`, `Ctrl+S`, `Alt+Enter` |
| `md_editor` | `ai` | Markdown 编辑器：AI 写作 | `Alt+E` |
| `dialog` | `browse` | 浏览文件夹 | `Ctrl+B` |
| `dialog` | `sources` | 转到克隆来源 | `Alt+S` |
| `dialog` | `submit` | 提交 pull request 表单 | `Ctrl+S` |
| `dialog` | `web` | 打开服务商的网页表单 | `Alt+W` |
| `folder_picker` | `choose` | 选择文件夹 | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | 新建文件夹 | `Alt+N` |
| `folder_picker` | `filter` | 筛选文件夹中的条目 | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | 显示或隐藏同级文件夹 | `Alt+S` |
| `folder_picker` | `toggle_hidden` | 显示或隐藏隐藏文件夹 | `Alt+H` |
| `preferences` | `search` | 搜索设置 | `Ctrl+F`, `/` |
| `preferences` | `list_add` | 添加列表项 | `Insert` |
| `preferences` | `list_duplicate` | 创建所选列表项的副本 | `Ctrl+D` |
| `preferences` | `list_remove` | 删除所选列表项 | `Delete` |
| `preferences` | `list_move_up` | 将所选列表项上移 | `Alt+Up` |
| `preferences` | `list_move_down` | 将所选列表项下移 | `Alt+Down` |
| `preferences` | `prev_tab` | 设置页的上一个标签页 | `Alt+Left` |
| `preferences` | `next_tab` | 设置页的下一个标签页 | `Alt+Right` |
| `ai_popup` | `copy` | 复制结果 | `c` |
| `activity_log` | `scroll_left` | 向左滚动活动日志 | `Left` |
| `activity_log` | `scroll_right` | 向右滚动活动日志 | `Right` |
| `integrations` | `add` | 添加配置 | `a` |
| `integrations` | `edit` | 编辑所选配置 | `e` |
| `integrations` | `open_token` | 打开令牌页面 | `o` |
| `integrations` | `remove` | 移除所选配置 | `d`, `Delete` |
| `prompt` | `ai_stash_message` | AI：生成 Stash 消息 | `Ctrl+G` |
<!-- keymap:end -->

### 终端无法发送的按键，以及 kitty 键盘协议

传统终端无法发送某些组合键：`Ctrl+,`、`Ctrl+Shift+N`、`Ctrl+Enter` 和 `Ctrl+<数字>`。它要么什么也不发送，要么发送另一个按键：例如，`Ctrl+Shift+N` 到达时变成 `Ctrl+N`，`Ctrl+I` 变成 `Tab`，`Ctrl+M` 变成 `Enter`。设置页面会对这类绑定给出警告（“此终端不发送 …”）。

支持 **kitty 键盘协议**的终端可以发送这些组合键。例如：kitty、WezTerm、Ghostty、foot、Alacritty 0.13 及更高版本，以及 zellij（tmux 不会应答：见[多路复用器](multiplexers.md#tmux)）。GitPoulpi 默认使用该协议：

- 启动时，GitPoulpi 会询问终端，并最多等待约 100 毫秒的回应。如果终端没有回应，则保留传统按键。
- 如果终端回应，GitPoulpi 会开启该协议，并在退出、失败或崩溃时将其关闭。
- 设置页面上的警告会跟随该协议：协议开启时，上述组合键即可发送。
- 将**设置 > 键盘与鼠标 > Kitty 键盘协议**（`ui.kitty_keyboard`）设为 `off` 后，永远不会更改终端的键盘模式。如果某个终端或多路复用器出现异常，请使用它。需要重启。

Windows 控制台自己会报告修饰键，不需要该协议。

## 命令面板（`Ctrl+P`）

面板列出了应用的**所有**命令，因此无需快捷键就能触达所有功能。完整列表见[下方](#所有面板命令)。

| 按键 | 操作 |
|---|---|
| `Ctrl+P` | 打开面板 |
| *（输入）* | 筛选列表。`类别：` 前缀之后的精确匹配排在最前，其次是标签中任意位置的匹配，最后是模糊匹配 |
| `Up` / `Down`, `Home` / `End`, `PageUp` / `PageDown` | 移动高亮（参见[可筛选的弹出列表](#可筛选的弹出列表)） |
| `Enter`，或双击 | 运行高亮的命令 |
| `Esc` | 关闭 |

`j` 和 `k` 会被输入到筛选框中，不会移动高亮。

每一行的右侧显示其快捷键。当前不适用的命令（例如没有选中任何引用）会留在列表中并显示为灰色，原因显示在列表下方。标签以类别开头，例如 `导航：`、`引用：`、`Git：`、`视图：`、`Diff：` 或 `应用：`。输入某个类别即可看到整个分组。

### 为任何命令设置快捷键

命令面板的每一行末尾都有一个按键按钮（`[k]`），在高亮行上按 `Alt+K` 效果相同：它会打开**设置 > 键盘与鼠标**，并按该命令的快捷键筛选，可直接重新绑定。没有专属快捷键的命令也能在这里获得一个：它列在**命令面板命令**下，默认未绑定，你为它指定的按键与在面板中选择它一样运行该命令（当前无法运行的命令会说明原因）。之后该行会显示你的按键。

在 `config.toml` 中，这对应 `[keybindings.palette]` 表，以命令命名（`toggle_minimap = "Ctrl+Alt+M"`）。与其他操作冲突的按键以红色显示，在其所在行按 `c` 会列出与之冲突的所有操作。

列表下方的提示栏会显示这个按键（默认 `Alt+K`）。它是操作 `palette.configure_shortcut`，可以像其他操作一样重新绑定。

命令面板和选择器（主题、语言、标签页、工作区搜索）可以像对话框一样移动和调整大小：拖动上边框，按 `Ctrl+N` 再按 `m`（与在所有对话框上一样），或按 `Alt+G`。`Alt+G` 是操作 `palette.arrange`（“对话框：移动或调整最前面的对话框的大小”）：一种额外的方式，在所有对话框和弹窗上都有效，用来对最前面的那个启动调整模式（面板打开时，面板中的 `对话框：移动或调整大小` 和 `对话框：重置位置和大小` 作用于面板自身）。方向键移动，`Shift`+方向键调整大小，`0` 重置，`Enter` 或 `Esc` 退出该模式，弹窗保持打开。位置是否保留由 `ui.dialog_geometry` 决定。

在这里 `Ctrl+N` 同样是导航前缀：它会在筛选框、列表第一行和关闭叉上显示徽标，其后的按键会按下该控件，或用 `m` 调整弹窗。它不再让高亮下移，`Ctrl+P` 也不会让它上移：对有筛选框的列表，只有方向键、`PageUp` / `PageDown` 和 `Home` / `End` 会移动。

### 所有面板命令

由应用生成（有测试保证其保持最新）。AI 命令仅在 AI 开启时出现；其余命令在不适用时显示为灰色，并给出原因。按键为默认值。

<!-- palette:begin -->
| 命令 | 默认按键 |
|---|---|
| 导航：聚焦引用窗格 | `Ctrl+N <badge>` |
| 导航：聚焦 Log 窗格 | `Ctrl+N <badge>` |
| 导航：聚焦状态窗格 | `Ctrl+N <badge>` |
| 导航：聚焦引用筛选 | `Ctrl+N <badge>` |
| 导航：聚焦引用 > 本地 | `Ctrl+N <badge>` |
| 导航：聚焦引用 > 远程 | `Ctrl+N <badge>` |
| 导航：聚焦引用 > Worktree | `Ctrl+N <badge>` |
| 导航：聚焦引用 > 子模块 | `Ctrl+N <badge>` |
| 导航：聚焦引用 > Stashes | `Ctrl+N <badge>` |
| 导航：聚焦引用 > Tags | `Ctrl+N <badge>` |
| 左侧窗格：显示/隐藏分区… |  |
| 左侧窗格：将分区恢复为默认 |  |
| 左侧窗格：将此分区设置用于所有仓库 |  |
| 左侧窗格：显示/隐藏本地分支 |  |
| 左侧窗格：显示/隐藏远程 |  |
| 左侧窗格：显示/隐藏 Worktree |  |
| 左侧窗格：显示 worktree 分区 |  |
| 左侧窗格：显示/隐藏 Stashes |  |
| 左侧窗格：显示/隐藏 Tags |  |
| 左侧窗格：显示/隐藏 Pull requests |  |
| 左侧窗格：显示/隐藏议题 |  |
| 左侧窗格：显示/隐藏子模块 |  |
| 导航：聚焦状态 > unstaged | `Ctrl+N <badge>` |
| 导航：聚焦状态 > staged | `Ctrl+N <badge>` |
| 导航：焦点切换到下一个窗格 | `Tab` |
| 导航：焦点切换到上一个窗格 | `Shift+Tab` |
| 导航：搜索 Commit… | `f` |
| 导航：折叠/展开引用窗格 | `Ctrl+N h` |
| 导航：加宽引用窗格 | `Ctrl+N Right` |
| 导航：收窄引用窗格 | `Ctrl+N Left` |
| 导航：加宽状态窗格 | `Ctrl+N Shift+Left` |
| 导航：收窄状态窗格 | `Ctrl+N Shift+Right` |
| 导航：折叠/展开状态窗格 | `Ctrl+N l` |
| 导航：增大引用分区 | `Ctrl+N +` |
| 导航：缩小引用分区 | `Ctrl+N -` |
| 导航：重置引用分区大小 |  |
| 导航：重置布局 |  |
| Commit 详情：增大消息区 | `Ctrl+N ]` |
| Commit 详情：缩小消息区 | `Ctrl+N [` |
| Commit 详情：重置消息区大小 |  |
| Commit 详情：消息向下滚动 |  |
| Commit 详情：消息向上滚动 |  |
| Commit 详情：打开消息中的链接… |  |
| 文件：切换 commit 文件排序（A-Z / Z-A） |  |
| 引用：检出所选引用 | `双击` |
| 引用：复制引用名称 |  |
| 引用：复制完整引用（refs/heads/...） |  |
| 引用：复制引用目标 SHA |  |
| 引用：将所选分支 Merge 到当前分支 |  |
| 引用：将当前分支 Rebase 到所选分支 | `Ctrl+G r` |
| 引用：将所选引用放到已检出分支上… |  |
| 引用：标记为拖放来源 | `m` |
| 引用：将标记的引用放到此处… | `M` |
| 引用：重命名所选分支… |  |
| 引用：将所选分支快进到当前分支 |  |
| 引用：将当前分支快进到所选分支 |  |
| 引用：从所选分支或 Tag 创建 worktree… |  |
| 引用：所选分支的 Reflog… |  |
| 引用：从所选分支发起 pull request… |  |
| 引用：在标签页中打开 worktree | `Enter / 双击` |
| 引用：worktree 的打开方式… |  |
| 引用：将 worktree 的分支 merge 到… |  |
| 引用：移除 worktree… |  |
| 引用：创建 worktree… | `+` |
| 引用：从当前分支创建 worktree |  |
| 引用：选择分支创建 worktree |  |
| 引用：更新子模块（所选） |  |
| 引用：初始化子模块（所选） |  |
| 引用：在标签页中打开子模块 | `Enter / 双击` |
| Git：更新所有子模块 |  |
| Git：在所选 Commit 处创建 worktree… |  |
| Git：从所选 Commit 创建补丁… |  |
| Git：从所选 Commit 到 HEAD 创建补丁… |  |
| Git：从工作区更改创建补丁… |  |
| 重写：清除旧历史… |  |
| Git：应用补丁… |  |
| 重写：历史工作台 | `Ctrl+G H` |
| 重写：替换作者… |  |
| 重写：匿名化作者… |  |
| 重写：将 .mailmap 应用于历史… |  |
| 重写：Merge 身份（.mailmap）… |  |
| 重写：平移 Commit 日期… |  |
| 重写：更改 Commit 时区… |  |
| 重写：把 Commits 限定在指定时段内… |  |
| 重写：将提交者日期设为作者日期 |  |
| 重写：在消息中查找替换… |  |
| 重写：删除消息中的行… |  |
| 重写：添加或删除尾部字段… |  |
| 重写：移除签名… |  |
| 重写：修复消息编码… |  |
| 重写：在文件中查找替换… |  |
| 重写：从历史中删除文件… |  |
| 重写：从历史中删除大文件… |  |
| 重写：移动或重命名路径… |  |
| 重写：将文件迁移到 LFS… |  |
| 重写：从 LFS 取回文件… |  |
| 重写：转换换行符… |  |
| 重写：移除可执行位… |  |
| 重写：转换文件编码… |  |
| 重写：运行脚本… |  |
| ✦ 重写：用 AI 编写脚本… |  |
| ✦ 重写：用 AI 创建规则… |  |
| 重写：从选中的 Commit 到 HEAD… |  |
| 重写：选中的 Commits… |  |
| LFS：为此仓库初始化 |  |
| LFS：跟踪模式… |  |
| LFS：取消跟踪模式… |  |
| LFS：fetch 对象 |  |
| LFS：打开菜单 | `Ctrl+G l` |
| LFS：Fetch 所有对象 | `Ctrl+G F` |
| LFS：检出所有文件 | `Ctrl+G K` |
| LFS：Push 所有对象 | `Ctrl+G J` |
| LFS：pull 对象 | `Ctrl+G L` |
| LFS：push 对象 |  |
| LFS：清理本地对象 | `Ctrl+G V` |
| LFS：显示锁 |  |
| LFS：打开安装页面 |  |
| LFS：在 diff 中切换原始指针 | `r` |
| LFS：跟踪所选文件 |  |
| LFS：跟踪所选文件的扩展名 |  |
| LFS：锁定所选文件 |  |
| LFS：解锁所选文件 |  |
| LFS：强制解锁所选文件 |  |
| 引用：删除所选分支或 Tag… | `Ctrl+G d` |
| 引用：从远程删除所选分支… |  |
| 引用：同时删除所选分支的本地和远程… |  |
| 引用：复制所选分支在远程上的链接 |  |
| Git：复制所选 Commit 在远程上的链接 |  |
| Git：在所选 Commit 处创建附注 Tag… |  |
| 引用：将所选 Tag Push 到默认远程 |  |
| 引用：从远程删除所选 Tag… |  |
| Git：Fetch 所选 Tag |  |
| 引用：为所选 Tag 添加注释… |  |
| 引用：切换在图中隐藏 |  |
| 引用：在图中定位 | `g` |
| 引用：切换所选引用的单独显示 | `o` |
| 引用：停止单独显示（清除所有单独显示的引用） | `O` |
| 引用：显示所有隐藏的引用 | `H` |
| 引用：切换所选文件夹/远程的隐藏 |  |
| 引用：切换所选文件夹/远程的单独显示 | `o` |
| 引用：按名称筛选… | `/` |
| 引用：清除名称筛选 | `Esc` |
| 引用：展开所有文件夹 | `=` |
| 引用：折叠所有文件夹 | `-` |
| 引用：展开所选文件夹及其子文件夹 | `*` |
| 引用：折叠所选文件夹及其子文件夹 | `_` |
| 引用：下一个分区 | `l` |
| 引用：上一个分区 | `h` |
| 引用：展开或折叠当前分区 | `Enter` |
| 引用：切换分支层级视图 | `y` |
| Git：stage 所选文件 | `Ctrl+G s` |
| Git：unstage 所选文件 | `Ctrl+G u` |
| Git：stage 所有更改 | `Ctrl+G a` |
| 工作区更改：丢弃所选文件… |  |
| 工作区更改：忽略所选文件 |  |
| 工作区更改：忽略所选文件的扩展名 |  |
| 工作区更改：将所选文件视为二进制… |  |
| 工作区更改：将所选文件视为文本… |  |
| 工作区更改：将所选文件的扩展名视为二进制… |  |
| 工作区更改：将所选文件的扩展名视为文本… |  |
| 工作区更改：Stash 所选文件 |  |
| 工作区更改：所选文件历史 |  |
| 工作区更改：所选文件 Blame |  |
| 工作区更改：在外部 diff 工具中打开所选文件 |  |
| 工作区更改：在外部编辑器中打开所选文件 |  |
| 工作区更改：用默认程序打开所选文件 |  |
| 工作区更改：在文件夹中显示所选文件 |  |
| 工作区更改：复制所选文件路径 |  |
| 工作区更改：从所选文件创建补丁… |  |
| 工作区更改：编辑所选文件 |  |
| 工作区更改：删除所选文件… |  |
| 工作区更改：打开所选文件夹菜单 |  |
| 工作区更改：创建文件 |  |
| 工作区更改：丢弃所有更改… | `Ctrl+G x` |
| Git：Commit… | `Ctrl+G c` |
| Git：立即 Commit staged 的更改 | `Ctrl+G Enter` |
| Git：编辑 WIP Commit 消息（在 Log 中） | `Log 中按 c` |
| Git：Fetch 所有远程 | `Ctrl+G f` |
| Git：Pull | `Ctrl+G p` |
| Git：Pull（可能时快进） |  |
| Git：Pull（仅快进） |  |
| Git：Pull（rebase） |  |
| Git：Pull 选项… |  |
| Git：Push | `Ctrl+G P` |
| Git：Push 到远程… |  |
| Git：Push 到所选远程… |  |
| Git：Push 到所有目标 |  |
| Git：从所有来源 Pull |  |
| Git：从一个远程 Pull… |  |
| Git：显示分组 Push |  |
| Git：继续分组 Push |  |
| Git：中止分组 Push |  |
| Git：重试失败的 push |  |
| Git：强制 Push（带 lease）… |  |
| Git：分支的远程… |  |
| 远程：添加远程… |  |
| 远程：移除远程… |  |
| 远程：编辑远程… |  |
| 远程：设置默认远程… |  |
| 远程：复制远程名称 |  |
| 远程：复制远程链接 |  |
| 远程：Fetch 远程… |  |
| 远程：所选远程的操作… | `.` |
| 仓库：稀疏检出… |  |
| 仓库：SSH 与凭据... |  |
| 仓库：测试连接 |  |
| 仓库：使用默认凭据 |  |
| Flow：打开菜单 | `Ctrl+G W` |
| Flow：开始分支… |  |
| Flow：新建议题… |  |
| Flow：接手托管平台分支… |  |
| Flow：完成当前分支… |  |
| Flow：与父分支同步… |  |
| Flow：提升… |  |
| Flow：设置父分支… |  |
| Flow：继续或中止已停止的操作… |  |
| Flow：切换分支分组 |  |
| Flow：工作流设置… |  |
| 远程：在图中隐藏远程… |  |
| 远程：在图中显示远程… |  |
| Git：在 HEAD 处新建分支… | `Ctrl+G b` |
| Git：在 HEAD 处新建 Tag… | `Ctrl+G t` |
| Git：Push 所有未 push 的 Tag | `Ctrl+G U` |
| Git：Stash 所有更改 | `Ctrl+G T` |
| Git：带消息 Stash 所有更改… | `Ctrl+G m` |
| Git：Stash Pop | `Ctrl+G o` |
| Git：Stash Apply |  |
| Git：Stash Drop… |  |
| Git：将所选 Commit Cherry-pick 到 HEAD | `Ctrl+G y` |
| Git：检出所选 Commit（detached HEAD） |  |
| Git：在所选 Commit 处创建分支… |  |
| Git：在所选 Commit 处创建 Tag… |  |
| Commit：复制消息 |  |
| Commit：复制作者姓名 |  |
| Commit：复制作者邮箱 |  |
| Commit：复制日期 |  |
| Commit：复制短 SHA |  |
| Commit：复制完整 SHA |  |
| Git：Revert 所选 Commit… | `Ctrl+G v` |
| Git：编辑 Commit 消息… | `Ctrl+G e` |
| Commit：修订… | `Ctrl+G E` |
| Commit：修订作者… | `Ctrl+G A` |
| Commit：修订摘要… |  |
| Commit：修订描述… |  |
| Git：拆分所选 Commit… |  |
| Git：从所选 Commit 交互式 Rebase… |  |
| Rebase：继续 |  |
| Rebase：跳过 |  |
| Rebase：中止 |  |
| git am：继续 |  |
| git am：跳过此补丁 |  |
| git am：中止 |  |
| 仓库：删除过期的 index.lock |  |
| 横幅：Pop 待处理的自动 stash |  |
| 横幅：完成待处理的自动 stash（保留结果） |  |
| 横幅：显示待处理的自动 stash |  |
| 冲突：显示预测的冲突 |  |
| 横幅：Drop 待处理的自动 stash… |  |
| 横幅：重新开始… |  |
| Git：将分支重置到所选 Commit（soft - 保留索引和工作区） |  |
| Git：将分支重置到所选 Commit（mixed - 保留工作区） | `Ctrl+G R` |
| Git：将分支重置到所选 Commit（hard - 丢弃一切）… |  |
| Git：用 Ours 解决冲突（此分支） |  |
| Git：用 Theirs 解决冲突（传入） |  |
| Git：用基础版本解决冲突（共同祖先） |  |
| 冲突：打开 merge 编辑器 | `在冲突文件上按 Enter` |
| Merge：下一个冲突 | `F7` |
| Merge：上一个冲突 | `Shift+F7` |
| Merge：下一个冲突文件 | `Alt+PageDown` |
| Merge：上一个冲突文件 | `Alt+PageUp` |
| Merge：采用 A 的块 |  |
| Merge：采用 B 的块 |  |
| Merge：此冲突采用当前 | `a` |
| Merge：此冲突采用传入 | `b` |
| Merge：两者都采用，先 A 后 B | `c` |
| Merge：两者都采用，先 B 后 A | `C` |
| Merge：此冲突采用基础版本 |  |
| Merge：清除此冲突 | `x` |
| Merge：交换顺序 | `s` |
| Merge：全部采用当前 | `A` |
| Merge：全部采用传入 | `B` |
| Merge：在输出中编辑此冲突 |  |
| Merge：显示或隐藏侧面板 | `F9` |
| Merge：向左滚动窗格 | `Shift+Left` |
| Merge：向右滚动窗格 | `Shift+Right` |
| Merge：切换基础窗格 | `Alt+D` |
| Merge：切换整个文件的 diff | `Alt+F` |
| Merge：撤销 | `Ctrl+Z` |
| Merge：重做 | `Ctrl+Y` |
| Merge：保存但不标记为已解决 | `Alt+S` |
| Merge：保存并标记为已解决 | `Ctrl+S` |
| Merge：将输出重置为 merge 结果 | `Alt+R` |
| Merge：循环切换输出高度 | `z` |
| Merge：加宽当前窗格 | `Alt+Right` |
| Merge：加宽传入窗格 | `Alt+Left` |
| Merge：重置窗格大小 | `Alt+0` |
| Merge：取消此冲突的选择 | `Alt+U` |
| Merge：聚焦当前（A）窗格 | `Alt+1` |
| Merge：聚焦传入（B）窗格 | `Alt+2` |
| Merge：聚焦输出窗格 | `Alt+3` |
| Merge：聚焦基础窗格 | `Alt+4` |
| Merge：移到左侧窗格（同一冲突） |  |
| Merge：移到右侧窗格（同一冲突） |  |
| Merge：聚焦下一个窗格 | `F6` |
| Merge：聚焦上一个窗格 | `Shift+F6` |
| Merge：从磁盘文件继续 |  |
| Merge：从 merge 结果重新开始 |  |
| Merge：取消解决并重新打开 |  |
| Merge：重新加载文件 |  |
| Merge：原样使用当前文件 | `a` |
| Merge：原样使用传入文件 | `b` |
| Merge：原样使用基础文件 | `o` |
| Merge：保留剩下的文件 | `k` |
| Merge：接受删除 | `D` |
| Merge：在外部 merge 工具中打开 | `m` |
| ✦ AI：建议 merge 冲突解决方案 | `Alt+I` |
| Merge：关闭 merge 编辑器 |  |
| Git：中止 Merge、Cherry-pick 或 revert |  |
| Git：跳过 Cherry-pick 或 revert |  |
| 仓库：打开或切换（仓库管理）… | `Ctrl+O` |
| 仓库：浏览仓库… | `b（管理标签页）` |
| 仓库：新建（初始化）… | `i（管理标签页）` |
| 仓库列表：打开所选 | `Enter` |
| 仓库列表：将所选加入或移出收藏 | `f` |
| 仓库列表：关闭或移除所选 | `x` |
| 仓库列表：所选仓库菜单… | `.` |
| 仓库列表：搜索 | `/ / Ctrl+F` |
| 仓库列表：折叠所有分区 | `-` |
| 仓库列表：展开所有分区 | `+` |
| 仓库列表：关闭所有标签页… |  |
| 仓库列表：将所选从列表中移除 |  |
| 仓库列表：移除所有最近项 |  |
| 工作空间：新建… | `n` |
| 工作空间：从打开的标签页新建… | `a` |
| 工作空间：切换… | `Alt+W` |
| 工作空间：编辑… | `e` |
| 工作空间：关闭（无工作空间） |  |
| 工作空间：管理… | `Alt+J` |
| 工作空间：关闭标签页 |  |
| 工作空间：定位缺失的仓库… | `l` |
| 工作空间：删除… |  |
| 工作空间：Fetch 全部 | `F` |
| 工作空间：Pull 全部 | `P` |
| 工作空间：克隆缺失的仓库 | `m` |
| 工作空间：将标签页添加到工作空间 |  |
| 工作空间：将标签页从工作空间移除 |  |
| Radar：打开 | `Alt+R` |
| Radar：刷新 | `F5` |
| Radar：搜索 | `/` |
| Radar：在浏览器中打开 | `o` |
| Radar：打开仓库 | `Enter` |
| Radar：Checkout 分支 | `c` |
| Radar：创建 worktree… | `t` |
| Radar：在仓库中显示 | `s` |
| Radar：开始处理议题… | `n` |
| Radar：固定或取消固定 | `p` |
| Radar：延后… | `z` |
| Radar：评审请求… | `e` |
| Radar：打开议题对话 | `i` |
| Radar：通过 API 评审 |  |
| Radar：克隆并评审… |  |
| Radar：编辑指派人… |  |
| Radar：编辑评审者… |  |
| Radar：编辑标签… |  |
| Radar：设置里程碑… |  |
| Radar：选择范围 | `w` |
| Radar：选择视图 | `v` |
| Radar：选择筛选 | `f` |
| Radar：显示未克隆的请求 | `a` |
| Radar：显示草稿 | `d` |
| Radar：启用账户范围的评审请求… | `T` |
| Radar：行菜单… | `.` |
| Radar：选择列 | `C` |
| Radar：加宽列 | `]` |
| Radar：收窄列 | `[` |
| Radar：重置列宽 | `0` |
| Radar：按内容调整该列宽度 | `=` |
| Radar：按内容调整所有列宽度 | `+` |
| Radar：返回主页 | `Esc` |
| Radar：关闭标签页 |  |
| 标签页：新标签页 | `Alt+T` |
| 标签页：新建… |  |
| 标签页：操作菜单… | `Ctrl+T` |
| 标签页：关闭其他标签页… |  |
| 标签页：关闭右侧标签页… |  |
| 标签页：加入或移出收藏 |  |
| 标签页：复制仓库路径 |  |
| 标签页：在文件管理器中打开 |  |
| 标签页：在终端中打开 |  |
| 标签页：打开方式… | `Alt+O` |
| 标签页：用默认方式打开 | `Ctrl+Alt+O` |
| 仓库：更改配置文件… |  |
| 帮助：关于 GitPoulpi | `Ctrl+N a` |
| 帮助：许可证与致谢 |  |
| 视图：活动日志 | `Ctrl+N A` |
| 视图：全局活动日志 | `Ctrl+N G` |
| 帮助：发行说明 |  |
| 更新：显示更新 | `Ctrl+N u` |
| 更新：打开安全公告 |  |
| 更新：复制更新命令 |  |
| 更新：在终端中运行更新命令 |  |
| 更新：立即检查更新 |  |
| 更新：下载 |  |
| 更新：立即更新 |  |
| 更新：重启以更新 |  |
| 更新：回退到上一个版本 |  |
| 更新：跳过此版本 |  |
| 发行说明：关闭标签页 |  |
| 任务：显示后台任务 | `Ctrl+N J` |
| 任务：取消所有可取消的任务 |  |
| 视图：复制诊断信息 |  |
| 活动日志：显示或隐藏“错误”级别 |  |
| 活动日志：显示或隐藏“警告”级别 |  |
| 活动日志：显示或隐藏“信息”级别 |  |
| 活动日志：显示或隐藏“调试”级别 |  |
| 活动日志：显示或隐藏“跟踪”级别 |  |
| 活动日志：显示所有级别 |  |
| 活动日志：不显示任何级别 |  |
| 活动日志：清空应用日志 |  |
| 活动日志：选择范围 |  |
| 活动日志：Git 操作 |  |
| 活动日志：应用日志 |  |
| 活动日志：显示此行对应的操作 |  |
| 活动日志：下一个失败的操作 |  |
| 活动日志：上一个失败的操作 |  |
| 活动日志：向左滚动 | `Left` |
| 活动日志：向右滚动 | `Right` |
| 活动日志：复制最近的错误 |  |
| 活动日志：执行建议的操作 |  |
| 活动日志：打开最近的错误 |  |
| 标签页：退出并 cd 到此处 |  |
| 仓库：克隆… | `Ctrl+Shift+N` |
| 仓库：下一个标签页 | `Ctrl+PageDown` |
| 仓库：上一个标签页 | `Ctrl+PageUp` |
| 标签页：左移 |  |
| 标签页：右移 |  |
| 仓库：关闭标签页 | `Ctrl+W` |
| 仓库：选择标签页… | `t` |
| 搜索：工作空间… | `Ctrl+Alt+F` |
| 编辑：撤销 | `Ctrl+Z` |
| 编辑：重做 | `Ctrl+Y` |
| 编辑：忘记被阻止的撤销条目 |  |
| 编辑：撤销历史… | `Alt+Z` |
| 编辑：Reflog… |  |
| 编辑：找回丢失的 Commits… |  |
| 编辑：将分支恢复到被阻止操作完成后的位置 | `Alt+Shift+Z` |
| 评审：显示概览 |  |
| 评审：显示文件 |  |
| 评审：显示 Commits |  |
| 评审：评论此行… | `c` |
| 评审：评论对话… | `C` |
| 评审：回复讨论… | `r` |
| 评审：解决或重新打开讨论 | `x` |
| 评审：折叠或展开讨论 | `z` |
| 评审：编辑待处理评论… | `e` |
| 评审：删除待处理评论 | `d` |
| 评审：将文件标记为已查看 | `v` |
| 评审：下一个文件 | `]` |
| 评审：上一个文件 | `[` |
| 评审：完成评审… | `f` |
| 评审：放弃待处理评审… |  |
| 评审：刷新 | `F5` |
| 评审：在浏览器中打开 | `o` |
| 评审：检出分支 | `b` |
| 评审：关闭 | `Esc` |
| 评审：编辑指派人… | `a` |
| 评审：编辑评审者… | `p` |
| 评审：编辑标签… | `l` |
| 评审：设置里程碑… | `m` |
| 评审：选择补丁集… | `P` |
| 评审：比较补丁集… | `D` |
| 评审：合入变更… | `U` |
| 评审：放弃变更… | `Q` |
| 评审：切换内联 / 分栏布局 | `V` |
| 评审：切换整个文件 / 仅 hunk | `H` |
| 评审：切换仅空白字符的更改 | `S` |
| 评审：切换自动换行 | `W` |
| 评审：下一个 hunk | `J` |
| 评审：上一个 hunk | `K` |
| 评审：选择编码… | `E` |
| 评审：切换缩略图 | `M` |
| 评审：向左滚动 diff | `Left` |
| 评审：向右滚动 diff | `Right` |
| 评审：复制所选文本或行 | `Ctrl+C` |
| 看板：将卡片移到… | `m` |
| 看板：将卡片左移 |  |
| 看板：将卡片右移 |  |
| 看板：打开卡片 | `Enter` |
| 看板：编辑指派人… |  |
| 看板：编辑标签… |  |
| 看板：设置里程碑… |  |
| 看板：在此列新建议题 | `n` |
| 看板：按指派人筛选… | `a` |
| 看板：按标签筛选… | `l` |
| 看板：按里程碑筛选… | `M` |
| 看板：按文本筛选 | `/` |
| 看板：清除筛选 | `x` |
| 看板：设置列标签… | `c` |
| 看板：刷新 | `r` |
| 看板：关闭 | `Esc` |
| 对话：返回 |  |
| 对话：刷新 | `F5` |
| 对话：加载更早的条目 | `m` |
| 对话：上一条目 | `Up` |
| 对话：下一条目 | `Down` |
| 对话：下一条未读条目 | `n` |
| 对话：回复条目 | `r` |
| 对话：引述条目 | `Q` |
| 对话：新评论 | `c` |
| 对话：条目的链接… | `l` |
| 对话：打开条目的图表 | `g` |
| 对话：复制条目链接 | `y` |
| 对话：在浏览器中打开 | `o` |
| 对话：编辑评论 | `e` |
| 对话：删除评论 | `d` |
| 对话：显示或隐藏事件 | `v` |
| 对话：切换编写与预览 |  |
| 对话：切换分栏视图 |  |
| 对话：发送评论 |  |
| 对话：切换内部备注 |  |
| 对话：关闭 | `Esc` |
| Markdown 编辑器：AI 写作 |  |
| Markdown 编辑器：刷新 AI 上下文 |  |
| 托管：打开议题对话… |  |
| 评审：✦ 起草评审… | `s` |
| 评审：接受建议 | `y` |
| 评审：编辑并接受建议… | `w` |
| 评审：忽略建议 | `n` |
| 评审：接受所有建议 | `A` |
| 评审：忽略所有建议 | `N` |
| 评审：在评审文本中使用 AI 摘要 | `i` |
| 评审：✦ 使用草稿… | `u` |
| 评审：编辑 ✦ 草稿文本… | `t` |
| 评审：丢弃 ✦ 草稿 | `N` |
| 撤销历史：撤销到这里… | `u` |
| 撤销历史：重做到这里… | `y` |
| 撤销历史：仅恢复此项… | `r` |
| 撤销历史：将分支恢复到原来的位置… | `o` |
| 撤销历史：刷新 |  |
| 撤销历史：关闭 | `Esc` |
| Reflog：在 Log 中显示 | `l` |
| Reflog：在此创建分支… | `b` |
| Reflog：在此创建 tag… | `t` |
| Reflog：将更改应用到工作区 | `w` |
| Reflog：将更改保存为 stash | `s` |
| Reflog：恢复到此位置… | `r` |
| Reflog：加载更多 | `m` |
| Reflog：刷新 |  |
| 丢失的工作：在此创建分支… | `b` |
| 丢失的工作：在此创建 tag… | `t` |
| 丢失的工作：Cherry-pick | `c` |
| 丢失的工作：将更改应用到工作区 | `w` |
| 丢失的工作：将更改保存为 stash | `s` |
| 丢失的工作：重新扫描 |  |
| Reflog / 丢失的工作：下一个窗格 | `Tab` |
| Reflog / 丢失的工作：路径或树形文件列表 | `v` |
| Reflog / 丢失的工作：重置窗格大小 |  |
| 对话框：移动或调整大小 | `Ctrl+N m` |
| 对话框：重置位置和大小 |  |
| 视图：刷新 | `r` |
| 视图：刷新（fetch、pull request、议题） | `F5` |
| 视图：切换帮助浮层 | `?` |
| 视图：切换引用列 |  |
| 视图：切换图列 |  |
| 视图：切换消息列 |  |
| 视图：切换作者列 |  |
| 视图：切换日期列 |  |
| 视图：切换更改列 |  |
| 视图：切换 SHA 列 |  |
| 视图：Log 列：选择要移动或调整宽度的列 | `C` |
| 视图：Log 列：选择下一列 | `Tab` |
| 视图：Log 列：选择上一列 | `Shift+BackTab` |
| 视图：Log 列：将该列左移 | `<` |
| 视图：Log 列：将该列右移 | `>` |
| 视图：Log 列：加宽该列 | `]` |
| 视图：Log 列：收窄该列 | `[` |
| 视图：Log 列：重置各列宽度 | `0` |
| 视图：Log 列：按内容调整该列宽度 | `=` |
| 视图：Log 列：按内容调整所有列宽度 | `+` |
| 视图：Log：向左滚动各列 | `Shift+Left` |
| 视图：Log：向右滚动各列 | `Shift+Right` |
| 视图：Log：切换各列的横向滚动 | `s` |
| 视图：Log：选择要显示的列 |  |
| Diff：打开所选文件的 diff | `Enter` |
| Diff：关闭 diff 视图 | `Esc` |
| Diff：下一个 hunk | `n` |
| Diff：上一个 hunk | `p` |
| Diff：stage 光标处的 hunk | `Space` |
| Diff：unstage 光标处的 hunk | `Space` |
| Diff：选择行（或清除选择） | `v` |
| Diff：stage 所选行 | `Enter` |
| Diff：unstage 所选行 | `Enter` |
| Diff：丢弃光标处的 hunk… | `x` |
| Diff：丢弃所选行… | `X` |
| 复制：所选文本（请先拖选文本） | `Ctrl+C` |
| 复制：光标所在行（或所选行） | `y` |
| Diff：复制光标处的 hunk |  |
| Diff：内联布局（+/- 在一列中） | `m` |
| Diff：分栏布局（前后并排） | `m` |
| Diff：切换仅 hunk（整个文件 / 仅更改） | `H` |
| 文件：切换路径/树 | `m` |
| 文件：切换查看所有文件 | `a` |
| Commit：切换“跳过钩子” | `H` |
| 文件：打开当前行或所选文件的菜单 | `.` |
| 文件：展开所有文件夹 | `=` |
| 文件：折叠所有文件夹 | `-` |
| 文件：展开所选文件夹及其子文件夹 | `*` |
| 文件：折叠所选文件夹及其子文件夹 | `_` |
| 文件：全选 |  |
| 文件：清除选择 |  |
| 文件：切换当前行的选择 |  |
| Log：复制为 Mermaid gitGraph… | `Y` |
| Log：比较所选内容 | `v` |
| Log：交换比较的两端 | `x` |
| Log：转到下一个比较行 | `n` |
| Diff：切换自动换行 | `w` |
| Diff：选择编码… | `C` |
| 视图：切换缩略图 | `M` |
| 视图：切换 Commit 列 | `Alt+C` |
| 视图：Commit 列：隐藏 |  |
| 视图：Commit 列：文件历史 |  |
| 视图：Commit 列：全部 Commits |  |
| 视图：聚焦 Commit 列 |  |
| 视图：加宽 Commit 列 |  |
| 视图：收窄 Commit 列 |  |
| 视图：Commit 列：向左滚动 | `Shift+Left` |
| 视图：Commit 列：向右滚动 | `Shift+Right` |
| 视图：Commit 列：调整下一栏 | `Tab` |
| 视图：Commit 列：将该栏左移 | `<` |
| 视图：Commit 列：将该栏右移 | `>` |
| 视图：Commit 列：加宽该栏 | `]` |
| 视图：Commit 列：收窄该栏 | `[` |
| 视图：Commit 列：重置各栏宽度 | `0` |
| 视图：Commit 列：按内容调整该列宽度 | `=` |
| 视图：Commit 列：按内容调整所有列宽度 | `+` |
| 视图：Commit 列：所选行的菜单 | `.` |
| 视图：Commit 列：选择显示的列 |  |
| 视图：Commit 列：切换引用列 |  |
| 视图：Commit 列：切换图列 |  |
| 视图：Commit 列：切换作者列 |  |
| 视图：Commit 列：切换日期列 |  |
| 视图：Commit 列：切换更改列 |  |
| 视图：Commit 列：切换 SHA 列 |  |
| 视图：Commit 列：在 Log 中显示所选行 |  |
| 视图：Commit 列：显示所选行中该文件的更改 |  |
| 视图：Commit 列：显示该文件在所选行时的版本 |  |
| 视图：Commit 列：在所选行处 Blame 该文件 |  |
| 编辑器：切换跟随选择 |  |
| 图片：放大 | `+` |
| 图片：缩小 | `-` |
| 图片：适应视图 | `0` |
| 图片：实际大小（1:1） | `1` |
| 图片：切换修改前 / 修改后 | `Tab` |
| 图片：加宽“修改前”窗格 | `Alt+Right` |
| 图片：加宽“修改后”窗格 | `Alt+Left` |
| 图片：重置窗格大小 | `Alt+0` |
| 图片：编辑方式… | `O` |
| 图片：用默认编辑器编辑 | `o` |
| 文件历史：在 Log 中显示 | `Enter` |
| 文件历史：复制 commit 的 SHA | `y` |
| 文件历史：复制摘要 | `Y` |
| 文件历史：显示此 commit 的 diff | `d` |
| 文件历史：打开此 commit 时的文件 | `o` |
| 文件历史：在此 commit 处 Blame | `b` |
| 文件历史：与工作区比较 | `c` |
| 文件历史：恢复为此版本 | `R` |
| 文件历史：向上扩展选择 | `Shift+Up` |
| 文件历史：向下扩展选择 | `Shift+Down` |
| 文件历史：在列表和 diff 之间切换 | `Tab` |
| 文件历史：加宽列表 | `Alt+Right` |
| 文件历史：收窄列表 | `Alt+Left` |
| 文件历史：重置列表宽度 | `Alt+0` |
| 文件：打开方式… | `O` |
| 文件：用默认应用打开 | `o` |
| Diff：切换仅空白字符的更改 | `s` |
| ✦ AI：生成 Commit 消息 | `Ctrl+G i` |
| ✦ AI：生成 Stash 消息 |  |
| ✦ AI：插入提示词变量 |  |
| ✦ AI：编辑配置… |  |
| ✦ AI：打开代理… |  |
| ✦ AI：解释所选 Commit | `Ctrl+G w` |
| ✦ AI：拆分所选 Commit… |  |
| ✦ AI：解释工作区更改 | `Ctrl+G w` |
| ✦ AI：根据工作区更改撰写 Commit… | `Ctrl+G C` |
| 撰写器：将文件移到 Commit… | `m` |
| Diff：Blame |  |
| Blame：在 Log 中显示 commit | `Enter` |
| Blame：复制 commit SHA | `Y` |
| Blame：显示该 commit 对此文件的 diff | `d` |
| Blame：对该 commit 的父级做 Blame | `b` |
| Blame：打开文件历史 | `H` |
| Blame：显示或隐藏详情卡片 | `i` |
| Blame：下一个 hunk | `n` |
| Blame：上一个 hunk | `p` |
| Blame：同一 commit 的下一个 hunk | `N` |
| Blame：同一 commit 的上一个 hunk | `P` |
| Blame：跳到行 | `g` |
| Blame：加宽注释列 | `>` |
| Blame：收窄注释列 | `<` |
| Diff：文件历史 |  |
| Diff：文件视图 |  |
| 文件：在工作区中编辑 | `e` |
| 编辑器：更改编码 |  |
| 编辑器：用编码重新打开 |  |
| 编辑器：转换为编码 |  |
| 编辑器：转换换行符 |  |
| 编辑器：更改缩进 |  |
| 应用：设置… | `Ctrl+, / Ctrl+N ,` |
| 应用：搜索设置… | `设置中按 /` |
| 应用：键位绑定… |  |
| 代理：显示待处理的视图 | `Ctrl+N Enter` |
| 代理：关闭待处理的视图 |  |
| 代理：返回 | `Ctrl+N Backspace` |
| 键位绑定：全部重置… |  |
| 集成：打开关联的 pull request | `Ctrl+G g` |
| 集成：在托管平台上编辑关联的 pull request | `e（在某一行上）` |
| 集成：复制关联的 pull request URL |  |
| 集成：刷新 pull request 和议题 |  |
| 集成：检出 pull request… |  |
| 集成：打开 commit 引用（#议题 / !请求）… |  |
| 集成：打开议题… |  |
| 托管：评审请求… |  |
| 托管：Push 以供评审… |  |
| 托管：编辑指派人… |  |
| 托管：编辑评审者… |  |
| 托管：编辑标签… |  |
| 托管：设置里程碑… |  |
| 本地议题：关闭议题… |  |
| 本地议题：重新打开议题… |  |
| 本地议题：在 Obsidian 中打开… |  |
| 议题：打开看板 |  |
| 本地议题：创建 Obsidian Base |  |
| Pull request：创建 pull request… | `n` |
| 托管：新建议题… | `n` |
| Pull request：搜索 | `在搜索行上按 Enter` |
| Pull request：选择筛选（打开 / 已 merge / 全部）… | `v` |
| Pull request：选择平台… | `在平台行上按 Enter` |
| Pull request：选择仓库… | `在仓库行上按 Enter` |
| Pull request：取消关联来源（无） |  |
| 议题：选择平台… | `在平台行上按 Enter` |
| 议题：取消关联来源（无） |  |
| 集成：取消关联配置（自动） |  |
| 议题：选择仓库… | `在仓库行上按 Enter` |
| 议题：搜索 | `在搜索行上按 Enter` |
| 议题：选择筛选… | `v` |
| 议题：编辑查询 / 列表… |  |
| Commit：插入议题引用… |  |
| 集成：为此仓库使用配置… |  |
| 集成：管理此仓库的配置 |  |
| 集成：连接集成… |  |
| 设置：配置的 Radar 选项… |  |
| 集成：查看安全检查 |  |
| 应用：退出 | `q` |
| 应用：主题… |  |
| 冲突：编辑基准分支列表… |  |
| 冲突：此仓库的基准分支… |  |
| 应用：更改主题… | `Ctrl+N y` |
| 应用：更改语言… | `Ctrl+N g` |
| 启动器：创建快捷方式 |  |
| 启动器：创建桌面快捷方式 |  |
| 启动器：移除快捷方式 |  |
| 启动器：创建命令别名 |  |
| 启动器：移除命令别名 |  |
| 启动器：添加到 PATH |  |
| 启动器：从 PATH 中移除 |  |
| 启动器：安装“在 GitPoulpi 中打开”菜单 |  |
| 启动器：移除“在 GitPoulpi 中打开”菜单 |  |
| 应用：更改图配色主题… | `Ctrl+N Y` |
| 应用：切换图行着色 | `Ctrl+N T` |
<!-- palette:end -->

## 在界面中移动

| 按键 | 操作 |
|---|---|
| `Tab` / `Shift+Tab` | 聚焦下一个 / 上一个窗格 |
| `Ctrl+N`，然后按徽标 | 聚焦窗口中的该部分，或按下最前面的对话框或弹窗中的该控件（参见[导航前缀](#导航前缀ctrln)） |
| `j` / `k`, `Up` / `Down` | 移动选择，在每个窗格中都适用 |
| `PageUp` / `PageDown`, `Home` / `End` | 移动一页；跳到第一行 / 最后一行 |
| 一个数字，然后按 `j` / `k` / `Up` / `Down` | 数量前缀：`4j` 移动四行。配合 `PageUp` / `PageDown` 时，数量表示页数。状态栏显示待执行的数量（`4…`）。`Esc` 或任何其他键会清除它。开头的 `0` 不会开始计数 |
| `h` / `l`, `Left` / `Right` | 引用窗格：切换分区。Log：离开（`h`）或进入（`l`）某个 commit 的已更改文件列表 |
| `?` | 打开或关闭帮助（参见[应用内帮助](#应用内帮助)） |
| `r` | 刷新 |
| `F5` | fetch 所有远程，重新加载 pull request 和议题，并刷新 |
| `q` | 退出 |
| `Ctrl+Z` | **撤销**上一次操作。参见[撤销与重做](undo-redo.md) |
| `Ctrl+Y`, `Ctrl+Shift+Z` | **重做**。`Ctrl+Shift+Z` 需要 kitty 键盘协议；`Ctrl+Y` 始终可用 |
| `.`, `Menu`, `Shift+F10` | 打开所选行的上下文菜单 |

### 文本框中的 Ctrl+Z / Ctrl+Y

当文本框获得键盘时，`Ctrl+Z` 和 `Ctrl+Y`（`Ctrl+Shift+Z` 也一样）作用于该文本框的文字，绝不作用于仓库：获得键盘的文本框总是优先，所以在 commit 消息中按 `Ctrl+Z` 不会撤销某个 commit。在其他任何地方，它们保持全局含义（上面的撤销 / 重做日志）。这适用于每个文本框和多行编辑器：commit 消息、设置中的字段和文本编辑器、pull request 和议题表单、AI 对话框，以及每个筛选框或搜索框。

- **输入**按词撤销（一串字母、一串空格、一串标点），停顿超过一秒的输入会开始新的一步。连续的退格键算一步。一次粘贴、一次剪切、`Ctrl+U` / `Ctrl+K`、一段插入的片段或一次接受的补全，各算一步。
- 撤销会恢复文字、光标和选区。撤销之后的新编辑会丢弃本可重做的内容。
- 每个文本框保留有限的历史（最近 200 步，约 1 MB 文字），当该文本框被外部替换（在选择器中选了文件夹、重新加载了表单）或另一个文本框获得键盘时，历史会被清除。
- 没有可撤销的内容时，`Ctrl+Z` 什么也不做：当文本框拥有键盘时，它绝不会落到仓库的撤销上。
- AI 设置的**指令编辑器**在保存和取消旁边还有**撤销**和**重做**按钮，没有可前往的步骤时它们是禁用的。
- 代码编辑器和 merge 编辑器保留它们自己的撤销（`编辑：撤销`、`Merge：撤销`），它也涵盖它们的选择。

## 可筛选的弹出列表

所有可筛选的弹出列表使用相同的按键：命令面板及其选择器（主题、语言、图的配色）、标签页选择器，以及每个带搜索行的下拉列表（设置中的选项、pull request 表单、议题引用选择器、**打开方式**列表）。输入文字即可筛选列表。以下按键移动高亮：

| 按键 | 操作 |
|---|---|
| `Up` / `Down`、`Ctrl+K` / `Ctrl+J` | 向上 / 向下移动一行 |
| `Home` / `End` | 第一行 / 最后一行 |
| `PageUp` / `PageDown` | 移动一个可见页 |
| `Enter` | 选择高亮的行 |
| `Esc` | 关闭 |

在这些弹出窗口中，`Home` 和 `End` 移动的是列表，而不是筛选框的光标，并且 `Ctrl+K` / `Ctrl+J` 也会移动列表，所以 `Ctrl+K` 不再删除到筛选框末尾（`Ctrl+U` 仍会清空它；在列表之外的普通文本框中，`Ctrl+K` 仍然删除到行尾）。`Ctrl+A` 和 `Ctrl+E` 仍会将光标移到开头和末尾。`Ctrl+J` / `Ctrl+K` 也会移动其他所有列表，无论有没有文本框（`Ctrl+N` 是导航前缀，`Ctrl+P` 是命令面板，所以它们不是列表按键）。单独的 `j` 和 `k` 会被输入到筛选框中；在没有文本框的列表中，`j` / `k` 是额外的下移 / 上移方式。

使用鼠标时：

- 滚轮**滚动**列表，不会移动高亮。
- 点击**选中**某一行。双击**选择**该行，效果与 `Enter` 相同。
- 比弹出窗口更高的列表会在右边缘显示滚动条，你可以拖动它。
- 没有搜索行的简短下拉列表（两三个选项）在第一次点击时就会选择。

**实时预览。**主题、图配色主题和语言选择器会立即在弹出窗口后面应用高亮的那一行。`Enter` 保留它。`Esc`，或点击弹出窗口之外，会恢复之前的值。

**主题筛选。**主题选择器在筛选框下方有一行：**全部**、**深色**、**浅色**和**彩色**。`Tab` / `Shift+Tab` 切换到下一个 / 上一个。`Alt+1` 到 `Alt+4` 直接选择其中一个。你也可以点击它们。

## 应用内帮助（`?`）

`?`、`Ctrl+N ?` 或面板中的 `视图：切换帮助浮层` 会打开帮助。它列出应用各个部分的按键和鼠标手势。它会在任何内容之上打开，包括设置界面。

| 按键 | 操作 |
|---|---|
| `?` | 关闭帮助（并清除筛选） |
| *（输入）* | 筛选：只保留包含该文本的分区。`j`、`k` 和 `q` 也会被输入 |
| `Backspace`, `Ctrl+U` | 删除最后一个字符 / 清除筛选 |
| `Esc` | 清除筛选；如果筛选已为空，则关闭帮助 |
| `Up` / `Down` | 滚动一行 |
| `PageUp` / `PageDown`, `Home` / `End` | 滚动一页；跳到顶部 / 底部 |
| `Ctrl+D`，或点击链接 | 在浏览器中打开 GitHub 上的这份文档 |

滚轮可以滚动帮助，你也可以拖动它的滚动条。点击帮助之外的区域会将其关闭。

## 导航前缀（`Ctrl+N`）

`Ctrl+N` 会等待再按一个键。状态栏显示它接受的按键。下一个按键会执行某个操作，或取消。

| 按键 | 操作 |
|---|---|
| `Ctrl+N`，然后按徽标 | 聚焦显示该徽标的窗口部分 |
| `Ctrl+N h` / `Ctrl+N l` | 折叠或恢复引用窗格 / 右侧窗格 |
| `Ctrl+N Left` / `Ctrl+N Right` | 将引用窗格与 Log 之间的分隔条向左 / 向右移动一档（已折叠的引用窗格会先恢复） |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | 将 Log 与右侧窗格之间的分隔条向左 / 向右移动一档 |
| `Ctrl+N +` / `Ctrl+N -` | 将当前引用分区增大 / 缩小一行 |
| `Ctrl+N ]` / `Ctrl+N [` | 将右侧窗格的 commit 消息块增大 / 缩小一行 |
| `Ctrl+N t` | 选择标签页（标签页选择器；单独按 `t` 效果相同） |
| `Ctrl+N y` / `Ctrl+N g` / `Ctrl+N Shift+Y` | 更改主题 / 语言 / 图的配色主题 |
| `Ctrl+N Shift+T` | 开启或关闭图的行着色 |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | 显示代理准备的视图 / 返回（参见 [MCP 与代理](mcp-and-agents.md)） |
| `Ctrl+N ,` | 打开设置 |
| `Ctrl+N ?` | 打开或关闭帮助 |
| `Esc`，或任何其他键 | 取消 |

### 徽标

前缀键等待期间，窗口中每个可以获得焦点的部分都会在其左上角边框上显示一个单字符徽标。按下该字符即可聚焦。徽标每次都会重新分配，按屏幕顺序排列：先是 `1` 到 `9`，然后是 `a`、`b`、`c`……所以请以屏幕上显示的徽标为准：同一部分并不总是得到相同的徽标。

带徽标的部分：引用筛选框和每个可见的引用分区、Log（或 Diff 视图）、打开时的文件视图、unstaged 和 staged 列表、commit 编辑区的摘要框，以及 commit 文件列表的控件。在主页标签页上，徽标分配给搜索框、四个分区和工具栏按钮（按钮的徽标会按下该按钮）。在 Radar 标签页上，徽标分配给筛选框、范围、视图和显示下拉框（下拉框的徽标会打开其列表）、刷新和返回按钮，以及屏幕上的每个分区标题。分区的徽标会把选中项移到其标题；随后按 `Enter` 或 `Space` 即可折叠。

在 Browse 文件夹选择器，以及克隆、初始化、创建 worktree、工作空间、稀疏检出、Flow、新建议题和拉取请求对话框中，徽标分配给字段、复选框和按钮（按钮的徽标会按下它，字段的徽标会把光标放进去；列表只有一个徽标，位于其第一行；Browse 文件夹选择器的“快速访问”列表和“文件夹”列表各有一个徽标，位于各自第一行旁，按下它会把键盘放进该列表而不选择任何一行）。在活动日志中，徽标分配给两个标签页、范围框和级别框、筛选框、级别复选框、按钮和列表的第一项。徽标的作用与点击该控件相同。在文件夹选择器中，`Ctrl+N` 用于启动徽标，所以它的新建文件夹键是 `Alt+N`。

在“修订”（以及“编辑 commit 消息”）对话框中，徽标分配给摘要、描述、作者姓名、邮箱和日期、**深入修改历史…**、**取消**、**应用** 和关闭叉；在改写历史工作台中，徽标分配给范围框及其复选框和引用按钮、方案名称、每条规则及其复选框和按钮、**+ 添加规则** 按钮、所选规则的字段、预览标签页、筛选框、受保护分支的行及其“全选” / “全不选”，以及“预览”、“应用”、“取消”、“保存”、“加载”和“用 AI 创建规则…”按钮（“机密”“身份”“大小”和结果页面会加上各自的按钮和行；“Commit”标签页的行是列表，所以没有）。徽标的作用与点击相同。下拉列表、问题、正在输入的字段、文件夹选择器、清除屏幕、脚本编辑器、AI 对话框、blob diff 或主机操作指南处于打开状态时，不显示任何徽标，“修订”对话框询问“放弃？”时也一样；`Esc` 会收起徽标而不关闭任何内容。

在设置中，徽标分配给搜索框、页面列表（当前页面上一个徽标）、子标签页、列表及其按钮、屏幕上页面的每个字段和关闭按钮 `✕`。徽标的作用与点击该控件相同：文本字段进入编辑，复选框被切换，下拉列表被打开，按钮被执行。正在字段中输入，或下拉列表、编辑器、搜索处于打开状态时，`Ctrl+N` 保持它自己的含义。

面板中有相同的操作，例如 `导航：聚焦引用 > 本地` 或 `导航：聚焦状态 > staged`。

## 仓库与标签页

| 按键 | 操作 |
|---|---|
| `Ctrl+O`，或点击 `⌂` | 转到主页标签页（在主页标签页上：聚焦搜索框；已打开 Radar 或发行说明时：先关闭它） |
| `Ctrl+Shift+N` | 打开主页标签页并显示克隆对话框 |
| `Ctrl+PageDown` / `Ctrl+PageUp` | 下一个 / 上一个标签页 |
| `t` | 从可筛选的列表中选择标签页 |
| `Ctrl+W` | 关闭当前标签页（会先确认；仓库不会被更改） |
| `Ctrl+T`，或右键点击标签页 | 标签页菜单（各行列在[标签行](interface-tour.md#标签行)中） |
| `Alt+O` / `Ctrl+Alt+O` | 打开标签页的文件夹：从**打开方式**列表中选择 / 使用默认项。参见[打开方式](open-in.md) |

### 主页标签页

`⌂` 标签页用于管理你的仓库。它有四个分区：**已打开的仓库**、**收藏**、**最近的仓库**和**所有仓库**。参见[界面导览](interface-tour.md#主页)。

| 按键 | 操作 |
|---|---|
| `j` / `k`、方向键、`PageUp` / `PageDown`、`Home` / `End` | 移动 |
| `Enter`，或双击 | 打开仓库（或切换到它的标签页）。在分区标题上：折叠或展开该分区 |
| `f` 或 `*`，或点击星标 | 添加到收藏 / 从收藏中移除 |
| `x`、`Delete`，或该行的按钮 | 关闭它的标签页（已打开），将其从列表中移除（最近），取消星标（收藏），忘记它（所有）。磁盘上的任何内容都不会被删除 |
| `.`、右键点击，或该行的 `⋮` | 该行的菜单 |
| `/` 或 `Ctrl+F` | 按名称、分支或路径搜索所有分区。`Enter` 打开第一个匹配项；`Esc` 先清除搜索，再离开搜索框 |
| `Space`, `Left` / `Right` | 折叠 / 展开当前分区 |
| `-` / `+` | 全部折叠 / 全部展开 |
| `b` / `c` / `i` | 浏览（打开文件夹）/ 克隆 / 初始化 |
| `Esc` | 清除搜索，或返回当前的仓库标签页 |
| `q` | 退出 |

`Ctrl+P`、`Ctrl+,`、`?` 和 `Ctrl+N` 在主页标签页上可用。`Ctrl+G`、`Ctrl+W` 和 `Ctrl+T` 需要仓库标签页：在主页标签页上它们只会显示一条消息。需要仓库的面板命令显示为灰色。

### 文件夹选择器

**浏览**（以及初始化、克隆和创建 worktree 对话框中的**浏览…**按钮）会打开文件夹选择器。顶部是路径框，左侧是位置和文件夹树，右侧是当前文件夹的内容。

| 按键 | 操作 |
|---|---|
| `Tab` / `Shift+Tab` | 路径框，然后是左列，再是内容区 |
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | 在获得焦点的列中移动。在路径框中：在补全项中移动 |
| 一个字母（内容区） | 跳到下一个以该字母开头的文件夹 |
| `Enter` / `Right` | 打开所选文件夹。在路径框中：转到输入的文件夹；若该文件夹已显示，则选择它 |
| `Backspace` / `Left` / `Alt+Up` | 上级文件夹（在路径框中，`Backspace` 删除一个字符） |
| `Tab`（路径框） | 接受唯一的补全项，或高亮的那一项 |
| `Alt+N` | 新建文件夹（`Ctrl+N` 用于启动徽标，参见[徽标](#徽标)） |
| `Alt+H` | 显示或隐藏隐藏文件夹 |
| `Ctrl+F`，或在路径框之外按 `/` | 边输入边筛选文件夹中的条目（不区分大小写；`..` 保留；标题显示“x / n”；`Enter` 跳到第一个匹配项，`Esc` 清除；切换文件夹时清空）。也可以用 `Tab` 或点击进入 |
| `Ctrl+S`、`Ctrl+Enter`，或确认按钮 | 选择所选的子文件夹，否则选择当前显示的文件夹 |
| `Esc` | 关闭补全项，或取消 |

当路径框显示新路径时，整个路径处于选中状态：你输入的第一个按键会替换它。

### 初始化和克隆对话框

| 按键 | 操作 |
|---|---|
| `Tab` / `Shift+Tab`, `Up` / `Down` | 下一个 / 上一个字段或按钮 |
| `Enter` | 按下获得焦点的按钮，或创建 / 克隆 |
| `Space` | 切换获得焦点的复选框，或按下获得焦点的按钮 |
| `Ctrl+B` | 浏览选择文件夹 |
| `j` / `k`、`g` / `G` | 在克隆对话框的来源列表和仓库列表中：向下 / 向上、第一个 / 最后一个（搜索框中无效） |
| `Esc` | 关闭。克隆过程中，克隆会在后台继续 |

在克隆对话框中，左列列出**通过 URL 克隆**，以及每个已连接的托管平台配置对应的一个来源（GitHub、GitLab、Gitea、Forgejo、Azure DevOps、Bitbucket、Gerrit）。用 `Up` / `Down` 或点击选择来源。然后在**仓库**搜索框中输入，用方向键、`PageUp` / `PageDown` 或滚轮在列表中移动，并用 `Enter` 或点击选择一行。各字段的说明参见[快速上手](getting-started.md)。

## 引用窗格

| 按键 | 操作 |
|---|---|
| `Enter` / `Space` | 展开或折叠当前分区或所选文件夹 |
| `h` / `l`, `Left` / `Right` | 切换分区 |
| `/` | 按名称筛选（`Enter` 应用，`Esc` 取消） |
| `y` | 开启或关闭分支层级视图：根据分支名中的 `/` 生成文件夹（本地和远程） |
| `o` | **单独显示**所选引用，或所选文件夹或远程下的所有引用。单独显示多个引用可查看它们的并集 |
| `Shift+O` | **停止单独显示**：清除所有单独显示 |
| `Shift+H` | **全部显示**：重新显示所有被隐藏的引用 |
| `g` | **在图中定位**：将 Log 滚动到所选引用（或 stash）的 commit 并让该行闪烁；选择不会移动 |
| `m` | 将所选引用**标记**为拖放来源（`Esc`，或在它上面再按 `m`，可清除标记） |
| `Shift+M` | 把标记的引用**放到**所选引用上：打开拖放时打开的菜单（Merge、Rebase、Push、Fast-forward……） |
| `+`（远程分区） | 添加远程 |
| `.`、`Menu`、`Shift+F10`，或右键点击 | 该行的菜单：分支、tag、远程名称或文件夹 |
| 在 WORKTREE 行上按 `Enter` | 在单独的标签页中打开该 worktree |
| `Ctrl+G d` | 删除所选分支或 tag（会先确认）。在远程行上：删除该远程分支 |

关于菜单的作用，参见[使用 Log](log-and-history.md)。

### Pull request 和议题

有匹配的集成时，引用窗格会有一个 pull request 分区（GitLab：merge request）。有已连接的托管平台配置时，它还有一个议题分区。参见[托管平台与议题跟踪器](hosting-integration.md#5-侧栏中的-pull-request-和议题)。

| 按键 | 操作 |
|---|---|
| `Enter` | 打开所选行：议题（以及 Azure DevOps 或 Bitbucket 上的请求）以[对话](hosting-integration.md#5c-对话)打开，有评审的托管平台上的 pull request 在[评审](review.md)中打开，其他情况在浏览器中打开。在 `重新连接` 行上：打开该配置的设置 |
| `Space` | 展开或折叠该分区 |
| `v` | 打开该分区的筛选。Pull request：打开、已 merge / 已关闭、全部。议题：打开、指派给我、已关闭、全部 |
| `n` | Pull request 分区：创建 pull request。议题分区：新建议题。（标题栏的 `+` 按钮） |
| `e` | Pull request 分区：在托管平台上编辑所选请求（该行的铅笔图标） |
| 在搜索行上按 `Enter` | 在搜索框中输入。`Enter`、`Tab` 或 `Esc` 离开搜索框；`Ctrl+U` 清除内容 |
| 在分组标题上按 `Enter` | 折叠或展开分组（*我的 pull request*、*指派给我*、*等待我评审*、*所有 pull request*） |
| 在筛选、提供方或仓库行上按 `Enter` | 打开该下拉列表。提供方列表以**无（取消关联此仓库）**结尾 |
| `.`，或右键点击 | 该行的菜单：打开对话、在浏览器中打开、复制 URL 或编号、检出源分支（pull request）；根据议题创建分支（议题） |

### 议题看板

议题标题上的**看板**（面板：**议题：打开看板**）会打开该议题来源的 Kanban 看板。见[看板](hosting-integration.md#5b-议题看板)和[本地议题](local-issues.md#kanban-看板)。下面的每个按键同时也是一条面板命令（**看板：…**）。

| 按键 | 动作 |
|---|---|
| 方向键 | 选择卡片（`Up`、`Down`）或相邻的列（`Left`、`Right`）；`Home`、`End`、`PageUp`、`PageDown` 在列内跳转 |
| `Shift+Left`、`Shift+Right` | 把所选卡片移到上一列或下一列 |
| `m` | **将卡片移到…**：列的下拉列表 |
| `Enter` | 打开议题的[对话](hosting-integration.md#5c-对话)（托管平台没有可读对话时打开它的页面） |
| `e`、`.` | 编辑指派人、标签或里程碑；卡片菜单 |
| `n` | 在所选列中新建议题 |
| `a`、`l`、`M`、`/` | 按指派人、标签、里程碑、文本筛选。`x` 清除筛选 |
| `c` | 作为 GitHub / GitLab 看板各列的标签 |
| `r` | 重新读取议题 |
| `Esc` | 关闭看板 |

鼠标：把卡片拖到另一列上即可移动，点击卡片选中它，双击打开它，右键点击打开卡片菜单，滚轮滚动一列，`Shift`+滚轮（或看板下方的滚动条）滚动各列。

### Markdown 文本框与对话视图

每个文本会发往托管平台的框（评论、回复、评审摘要、新议题或 pull request 的描述）都是同一个文本框。它的按键在上面的 `md_editor` 表中，可以重新绑定。

| 按键 | 操作 |
|---|---|
| `Alt+P` | 在**编写**与**预览**之间切换（从**分栏**切换时进入预览） |
| `Alt+L` | 打开**分栏**，或回到编写（需要 100 列宽的窗口） |
| `Ctrl+Enter`、`Ctrl+S`、`Alt+Enter` | 发送文本（表单自己的按钮作用相同） |
| `Alt+E` | **AI 写作**：打开或关闭模式栏下方的面板（仅在 AI 开启时） |
| `Alt+R` | 仅 commit 撰写器：插入议题引用 |
| `Alt+W` | 新建议题和 pull request 表单：在托管平台自己的网页上创建 |
| `Esc` | 在 **AI 写作**面板中：停止正在运行的草稿，否则关闭面板 |

在议题或请求的[对话视图](hosting-integration.md#5c-对话)中（命令面板行 *对话：…*；带按键的行在上面的面板列表中）：

| 按键 | 操作 |
|---|---|
| `r` | 回复所选条目（进入它的评论串，或在没有评论串的托管平台上带上它的引述文本） |
| `Q` | 在新评论中引述所选条目 |
| `c` | 新评论 |
| `e`、`d` | 编辑、删除你自己的评论 |
| `Alt+I` | 在评论框中：**内部备注**（GitLab、Redmine） |
| `l`、`y`、`o` | 该条目的链接、复制它的地址、在浏览器中打开它 |
| `n` | 下一条未读条目 |
| `v`、`m`、`F5` | 显示或隐藏事件、加载更早的条目、刷新 |
| `g` | 打开该条目的图表（`mermaid` 块） |
| `Alt+Left`、`Backspace` | 沿着从引用打开的条目栈返回 |
| `Tab` | 时间线、评论框、标题区按钮 |
| `q`、`Esc` | 关闭 |

在 Radar 中，`i` 会打开议题行的对话。

在侧栏各区的 pull request 或议题行上，`Enter` 打开对话，`o` 在浏览器中打开该行。

### Flow（分支工作流）

仓库设置了工作流后（见[分支工作流](branching-workflows.md)），工具栏会出现 **Flow** 按钮，侧栏也会按分支类型对 LOCAL 和 REMOTE 分组。菜单的每一行都是一条面板命令（**Flow：…**；**Flow：开始 Feature…** 和 **Flow：新建 Feature 议题…** 会为工作流的每个主题类型生成）。由托管平台驱动的行（*新建 <类型> 议题*、*接手托管平台创建的分支*）及其对话框、等待卡片和“分支已出现”卡片，见[分支工作流](branching-workflows.md#应用中由托管平台驱动的流程)；`Enter` 运行当前聚焦的按钮（*创建议题*、*接手*），`Esc` 停止等待或关闭。

| 按键 | 动作 |
|---|---|
| `Ctrl+G Shift+W` | 打开 Flow 菜单（即工具栏按钮） |
| `Enter`、`Tab`、`Shift+Tab` | 在“开始 / 完成 / 同步 / 提升 / 设置父分支”对话框中：预览或运行、下一字段、上一字段 |
| `Space` | 勾选复选框（*发布分支*、*运行钩子*、*同时更新堆叠在其上的分支*） |
| `Esc` | 关闭下拉列表，从预览返回表单，关闭对话框（运行期间无效） |
| 悬停在分支上 | 显示其类型、父分支，以及落后父分支多少 |

## Log 窗格

| 按键 | 操作 |
|---|---|
| `Enter` / `l` | 进入所选 commit 的已更改文件列表 |
| 在文件行上按 `Enter` / `l` | 在中央区域打开该文件的 diff |
| `Esc` / `h` | 返回 commit 列表 |
| `Right` | （带有多个引用的 commit）打开它的其他引用列表：`Up` / `Down` 移动，`Enter` 检出，`Esc` 关闭 |
| `f` | 打开搜索栏（参见[搜索](#搜索)）；它从来不是 `/`，所以 Log 的搜索没有变化 |
| `/` | 聚焦引用筛选框（引用窗格获得焦点时同一个键也能做到）；文本框获得焦点时则作为字符输入 |
| `c` | （有未 commit 的更改时）选中 WIP 行并将光标放入它的消息框 |
| 在 WIP 行上按 `Enter` | 将光标放入 WIP 行的消息框 |
| `.`, `Menu`, `Shift+F10` | commit 菜单。`Up` / `Down` 移动，`Right` / `l` / `Enter` 打开子菜单，`Left` / `h` / `Esc` 返回 |
| `m` / `a` | 右侧窗格的文件：`路径` 或 `树` / `查看所有文件` |
| `Shift+C` | 选择要移动或调整宽度的 Log 列（见下） |

选中一列后，`Tab` / `Shift+Tab` 选择其他列，`<` / `>` 移动它，`]` / `[` 加宽或收窄它，`0` 把所有宽度恢复为自动，`Esc` 或 `Enter` 退出（上方生成表格中的上下文 `log_columns`）。

WIP 行的消息框就是 commit 编辑区的摘要框。光标在其中时，所有按键都会被输入，但 `Ctrl` 组合键仍然有效。`Enter` 移到 commit 编辑区的描述。`Esc` 离开该框并保留文本。参见[stage 与 commit](staging-and-committing.md)。

### 选择多个 commit

| 按键或手势 | 操作 |
|---|---|
| `Space` | 将当前 commit 加入选择，或将其移除 |
| `Shift+Up` / `Shift+Down`、`Alt+Up` / `Alt+Down` | 扩展选择（终端自己占用 Shift 时用 Alt） |
| `Ctrl+点击` / `Shift+点击` 或 `Alt+点击` | 切换某个 commit / 选择一个范围 |
| `Esc` | 清除选择 |
| `Shift+Y` | [将所选内容复制为 Mermaid gitGraph](log-and-history.md#将-commit-复制为-mermaid-gitgraph)（带详细程度选项和实时预览的弹窗；`Enter` 复制） |

选中多个 commit 后，右键点击其中一个并选择**将 N 个 commit cherry-pick 到 `<branch>`…**。它会打开与交互式 rebase 相同的视图。

### 为分支或 tag 命名

`Ctrl+G b`、`Ctrl+G t`、工具栏的**分支**和 **Tag** 按钮，以及菜单中的**在此创建分支…** / **在此创建 tag…**，会在该 commit 所在行的文本框中询问名称。`Enter` 创建引用。`Esc` 取消。无效或已存在的名称会以红色显示，文本框保持打开。

当该行不在屏幕上时（打开了 diff、Log 被单独显示、Ref 列被隐藏），名称改为输入到一个带关闭叉号、**取消**和**创建**按钮的小边框框中。重命名分支直接在 Refs 窗格中它所在的行上输入，或在 Log 中它那一行的 Ref 单元格里输入；创建文件则在工作区更改列表中的文件夹下输入。所有位置按键相同：`Enter` 确认，`Esc` 取消。

状态栏中从不输入任何内容。需要更多空间的提示都是带真正按钮的对话框：stash 消息、附注 tag 的消息（`Ctrl+Enter` 或 `Ctrl+S` 应用，`Enter` 换行）、设置 upstream（可搜索的远程分支列表，外加“取消设置 upstream”按钮），以及 Git LFS 的跟踪和取消跟踪模式。从命令面板选择的数值或文本设置会打开设置页并定位到该行，可直接编辑；Refs 过滤（`/`）则输入到 Refs 窗格顶部的输入框中。

### 编辑 commit 消息

commit 菜单中的**编辑 commit 消息…**、`Ctrl+G e`，或面板中的 `Git：编辑 Commit 消息…`，会打开**改写 commit 消息**弹出窗口。它会改变什么，参见[使用 Log](log-and-history.md)。

| 按键 | 操作 |
|---|---|
| 在摘要中按 `Enter` | 转到描述 |
| 在描述中按 `Enter` | 换行。在第一行按 `Up` 返回摘要 |
| `Ctrl+Enter`、`Ctrl+S`，或**更新消息** | 保存新消息 |
| `Tab` / `Shift+Tab` | 在摘要、描述和按钮之间移动 |
| `Esc`，或**取消** | 关闭且不做更改 |

### 修订 commit

`Ctrl+G Shift+E`（所有字段）和 `Ctrl+G Shift+A`（作者）会打开修订对话框，commit 菜单、命令面板中的 `Commit：修订…` 各行以及 commit 详情窗格也一样；所有入口都在[修订 commit](log-and-history.md#修订-commit消息作者日期)中介绍。这两个键序都可以重新绑定：`git_prefix.amend` 和 `git_prefix.edit_author`。对话框中的按键：

| 按键 | 操作 |
|---|---|
| `Tab` / `Shift+Tab` | 在各字段和按钮之间移动 |
| 在单行字段中按 `Enter` | 下一个字段；在最后一个字段上则**应用** |
| `Up` / `Down` | 上一个 / 下一个字段（在描述中，从其第一行 / 最后一行开始） |
| `Ctrl+Enter`、`Ctrl+S`，或**应用** | 应用修改过的字段 |
| `Esc`，或**取消** | 关闭且不做更改 |
| `Alt+H`，或**深入修改历史…** | 关闭对话框，并从这个 commit 起打开历史重写工作台（仅限包含历史重写的构建）；如果你编辑过字段，会先询问（`Esc` 确认）。可重新绑定：`reword.deeper` |

### 创建 worktree

**从 `<ref>` 创建 worktree…**（分支和 tag 菜单）和**在此创建 worktree…**（commit 菜单）会打开一个对话框：文件夹、一个可选的新分支，以及**将新 worktree 作为标签页打开**。按键：`Tab` / `Shift+Tab` 移动，`Enter` 创建，`Space` 切换复选框，`Ctrl+B` 浏览，`Esc` 取消。

### 交互式 rebase

commit 菜单中的**从此处交互式 rebase…**，或 `Git：从所选 Commit 交互式 Rebase…`，会打开 rebase 视图。对多个 commit 的 cherry-pick 使用同一个视图。参见[使用 Log](log-and-history.md)。

| 按键 | 操作 |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | 设置该行的操作：Pick / Reword / Squash / Fixup / Drop / Edit（Fixup 和 Edit 仅用于 rebase） |
| `Space` | 打开该行的操作列表（`Up` / `Down` 选择，`Enter` 应用，`Esc` 关闭列表） |
| `Up` / `Down` | 移动选择 |
| `Ctrl+Up` / `Ctrl+Down`, `Alt+Up` / `Alt+Down` | 将所选行上移 / 下移 |
| `a` | 开启或关闭 autosquash 预览 |
| 在 Reword 行上按 `Enter` | 编辑该 commit 的新消息 |
| `Ctrl+S`、`Ctrl+Enter`，或**开始 Rebase** / **开始 Cherry-pick** | 运行计划 |
| `Ctrl+R`，或**重置** | 恢复为视图打开时的计划 |
| `Esc`，或**取消** | 关闭且不做更改 |

rebase 停下来时（遇到冲突或 Edit 步骤），Log 上方的横幅会提供**继续**、**跳过**和**中止**。面板中有 `Rebase：继续`、`Rebase：跳过` 和 `Rebase：中止`。

## 搜索

`f`、工具栏的**搜索**按钮，或 `导航：搜索 Commit…`，会在 Log 上方打开搜索栏。它搜索完整的 commit 消息、作者姓名和邮箱、SHA 以及引用名称。它不会隐藏行：不匹配的行会变暗。

| 按键 | 操作 |
|---|---|
| *（输入）* | 编辑搜索文本（所有按键都会被输入） |
| `Enter` / `Down`，或搜索栏的 `↑` | 下一个匹配项 |
| `Shift+Enter` / `Up`，或搜索栏的 `↓` | 上一个匹配项 |
| `Esc`，或搜索栏的 `✕` | 关闭搜索栏 |

## Diff 视图

已更改的文件会在中央区域打开，取代 Log。参见[stage 与 commit](staging-and-committing.md)。

| 按键 | 操作 |
|---|---|
| `Enter` / `l`，或双击（在文件行上） | 打开该文件的 diff |
| `Esc`、`q`，或标题中的 `✕` | 关闭 diff |
| `j` / `k`, `Up` / `Down` | 滚动 |
| `PageUp` / `PageDown`, `Home` / `End` | 滚动一页；跳到开头 / 结尾 |
| `h` / `l`, `Left` / `Right` | 横向滚动（自动换行时不可用） |
| `n` / `p` | 下一个 / 上一个 hunk |
| `m` | 内联或分栏布局 |
| `Shift+H` | 仅 hunk 或整个文件 |
| `w` | 开启或关闭自动换行 |
| `s` | 显示或隐藏仅空白字符的更改 |
| `Shift+M` | 开启或关闭缩略图 |
| `Shift+C` | 选择读取文件所用的编码（一个列表：先是“自动检测”） |
| `e` | 在内置编辑器中编辑文件（参见[编辑器](#编辑器)） |
| `r` | （Git LFS 文件）LFS 卡片或原始指针。参见 [Git LFS](lfs.md) |
| `Shift+E` | （有冲突的文件）打开 merge 编辑器。参见[解决 merge 冲突](merge-conflicts.md) |
| `y` / `Ctrl+C` | 复制所选文本。没有选区时：复制光标所在行，或所选行 |
| `.`，或在 hunk 上右键点击 | hunk 菜单 |

### 在 diff 中 stage 和丢弃

这些按键仅适用于工作区更改的 diff（从 unstaged 或 staged 列表打开）。丢弃仅适用于 unstaged 列表。

| 按键 | 操作 |
|---|---|
| `Space` | stage（unstaged）/ unstage（staged）光标所在的 hunk |
| `v` | 开始或放弃行选择；`j` / `k` 扩展选择 |
| `Enter` | stage / unstage 所选行，或光标所在行 |
| `x` | 丢弃光标所在的 hunk（会先确认：`y` / `Enter` 确认，`n` / `Esc` 取消） |
| `Shift+X` | 丢弃所选行，或光标所在行（会先确认） |

只有已更改的行才会计入：选区中的上下文行会被忽略。延伸到第二个 hunk 的选区只作用于第一个 hunk。

## 图片视图

Diff 视图或文件视图中的图片。按键在上表的 `image` 上下文中。参见[图片](image-view.md)。

## 文件视图、Blame 和文件历史

| 按键 | 操作 |
|---|---|
| `Esc` / `q` | 关闭（有文本选区时，`Esc` 先清除选区） |
| `j` / `k`, `Up` / `Down` | 移动选择 |
| `PageUp` / `PageDown` | 滚动一页 |
| `Home` / `End`, `Shift+G` | 第一行 / 最后一行 |
| `h` / `l`, `Left` / `Right` | 横向滚动文本 |
| `Enter` | （Blame、文件历史）在 Log 中转到该行的 commit |
| `e` | 在内置编辑器中编辑文件 |
| `y` / `Ctrl+C` | 复制所选文本。没有选区时：复制所选行（Blame：仅代码；文件历史：摘要） |

Blame 和文件历史有各自的按键，列在上面生成的表格中（上下文 `file_insight` 和 `history`），以及命令面板中的 `Blame: ...` / `文件历史：...`；全部都可以重新绑定。详情见 [Blame](log-and-history.md#blame) 和 [文件历史](log-and-history.md#文件历史)。在历史列表中，单个字母（`d` `o` `b` `c` `y`）作用于所选行，但带 Ctrl 的组合键（`Ctrl+C`、`Ctrl+P`、`Ctrl+G ...`）始终保持其全局含义。

## Commit 列

文件视图左侧的列（见[Commit 列](interface-tour.md#commit-列)）。`Alt+C` 在任何位置都能显示或隐藏它；其他按键在它获得键盘焦点后才有效（`Ctrl+N` 后按它的徽标，或 `视图：聚焦 Commit 列`）。

| 按键 | 操作 |
|---|---|
| `Up` / `Down`、`PageUp` / `PageDown`、`Home` / `End` | 选择 commit |
| `Enter` / `Right` | 转到所选 commit 的文件列表 |
| `Esc` | 把键盘还给视图 |
| `.`、`Menu`、`Shift+F10` | 所选行的菜单 |

它的其他按键（`Shift+Left` / `Shift+Right` 横向滚动，`Tab`、`]`、`[` 和 `0` 调整各栏宽度）列在上面生成的表格中，上下文为 `commit_column`。

## 编辑器

**编辑**，或在 diff 或文件视图中按 `e`，会在文件的工作目录副本上打开内置编辑器。参见[stage 与 commit](staging-and-committing.md)。

| 按键 | 操作 |
|---|---|
| *（输入）* | 在光标处插入（替换选区） |
| `Enter` | 换行，并保持当前行的缩进 |
| `Tab` / `Shift+Tab` | 缩进 / 取消缩进（若选中多行，则作用于所选行） |
| 方向键、`Home` / `End` | 移动。`Home` 先跳到第一个非空白字符，再跳到第 0 列。`Shift` 扩展选区 |
| `Ctrl+Left` / `Ctrl+Right` | 移动一个单词 |
| `Ctrl+Home` / `Ctrl+End` | 文件开头 / 结尾 |
| `PageUp` / `PageDown` | 上 / 下翻一页 |
| `Ctrl+A` | 全选 |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | 复制 / 剪切 / 粘贴 |
| `Ctrl+U` / `Ctrl+K` | 删除到行首 / 行尾 |
| `Ctrl+Z` | 撤销 |
| `Ctrl+Y` / `Ctrl+Shift+Z` | 重做 |
| `Ctrl+S` | 保存 |
| `Ctrl+F` / `F3` | 查找。在查找框中：`Enter` / `Down` / `F3` 下一个，`Shift+Enter` / `Up` 上一个，`Esc` 关闭 |
| `Alt+M` | 开启或关闭缩略图 (**Map**) |
| `Esc` | 关闭。有未保存的更改时会询问：**保存**（`S` / `Enter`）、**丢弃**（`D`）、**取消**（`C` / `Esc`） |

`Tab` 在编辑器中用于缩进，因此不会移动焦点。其他 `Ctrl` 组合键（`Ctrl+P`、`Ctrl+N`）仍然有效。

状态栏中的**编码**、**换行符**和**缩进**都是按钮：每个按钮打开一个下拉菜单（方向键或滚轮，
输入文字过滤，`Enter` 选择，`Esc` 关闭）。命令面板（`Ctrl+P`）可以打开相同的菜单：`编辑器：更改编码`、`编辑器：用编码重新打开`、`编辑器：转换为编码`、`编辑器：转换换行符`、`编辑器：更改缩进`。
详见 [stage 与 commit](staging-and-committing.md)。

## Git 操作（`Ctrl+G`）

`Ctrl+G` 会等待再按一个键：

| 按键 | 操作 |
|---|---|
| `s` / `u` | stage / unstage 所选文件 |
| `a` | stage 所有更改 |
| `x` | 丢弃所有更改（会先确认） |
| `c` | 打开 commit 编辑区 |
| `Enter` | 在任何位置立即用表单中的消息 commit 已 stage 的更改（见[stage 与 commit](staging-and-committing.md#编写-commit)） |
| `m` | 带消息 stash 所有更改，消息在一个小对话框中输入（可以为空）。开启 AI 时，其 AI 按钮（对话框中的 `Ctrl+G`）会撰写消息 |
| `Shift+T` | stash 所有更改（包括未跟踪的文件），不带消息 |
| `o` | 弹出一个 stash：Log 中选中的 stash 行，否则为最新的 stash |
| `p` | 按 Pull 按钮的设置执行 pull（`git.pull_mode`） |
| `Shift+P` | push 到默认远程（首次 push 时设置上游），或询问远程 |
| `Shift+U` | push 所有 tag |
| `f` | fetch 所有远程，并执行 prune |
| `g` | 在浏览器中打开当前检出分支的 pull request |
| `b` / `t` | 在 HEAD 处新建分支 / 新建 tag（参见[为分支或 tag 命名](#为分支或-tag-命名)） |
| `d` | 删除所选分支或 tag（会先确认） |
| `e` | 编辑所选 commit 的消息 |
| `Shift+E` / `Shift+A` | 修订所选 commit（所有字段）/ 它的作者（见[修订 commit](#修订-commit)） |
| `y` | cherry-pick 所选 commit |
| `v` | revert 所选 commit |
| `Shift+R` | 以 mixed 方式 reset 到所选 commit |
| `r` | rebase 到所选分支上 |
| `Shift+W` | 打开 Flow 菜单（有分支工作流的仓库） |
| `Shift+H` | 打开历史重写工作台（仅限包含历史重写的构建） |
| `l` | 打开 LFS 菜单。`Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V`：LFS fetch 全部 / checkout 全部 / pull / push 全部 / prune。参见 [Git LFS](lfs.md) |
| `i` | ✦ AI：为已 stage 的更改撰写 commit 消息 |
| `w` | ✦ AI：解释所选 commit 或工作区更改 |
| `Shift+C` | ✦ AI：建议如何将工作区更改拆分为多个 commit |

AI 按键仅在 AI 开启时有效（参见 [AI 辅助](ai.md)）。`Esc` 取消正在运行的 AI 请求。

## 右侧窗格（工作区更改与 commit 详情）

选中 WIP 行时，右侧窗格显示 **unstaged 的文件**和 **staged 的文件**列表以及 commit 编辑区。选中某个 commit 时，它显示 commit 详情及其文件。

| 按键 | 操作 |
|---|---|
| `j` / `k`, `Up` / `Down` | 移动。到达一个列表的末尾后，选择会继续进入另一个列表 |
| `h` / `l`, `Left` / `Right` | 转到 unstaged / staged 列表 |
| `Enter` | 折叠或展开当前列表 |
| `Space` / `Insert` | 将当前文件加入选择，或从中移除 |
| `Ctrl+A` | 选中列表中的所有文件 |
| `Ctrl+Space` | stage / unstage 所选文件，或折叠文件夹行 |
| `d` | 打开所选文件的 diff |
| `m` | `路径`（平铺列表）或 `树`（文件夹） |
| `a` | `查看所有文件`：整个文件树，而不仅是已更改的文件 |
| `z` | 折叠所有文件夹，或再次展开 |
| `/` | 按名称筛选 commit 的已更改文件 |
| `.`, `Menu`, `Shift+F10` | 该文件的菜单 |
| `c` | 聚焦 commit 编辑区的摘要框 |
| `Shift+A` | 开启或关闭**修订上一个 commit** |
| `Ctrl+N ]` / `Ctrl+N [` | 增大 / 缩小 commit 消息块 |

在 commit 编辑区中：

| 按键 | 操作 |
|---|---|
| `Enter` | 摘要：转到描述。描述：换行 |
| `Ctrl+Enter`, `Alt+Enter`, `Ctrl+J` | commit（`Ctrl+G Enter` 在任何位置都能做到） |
| `Alt+R` | 插入议题引用 |
| `Esc` | 离开编辑区 |

文件菜单包含：stage 或 unstage、丢弃更改、忽略、Stash 文件、文件历史、文件 Blame、在外部 diff 工具中打开、在外部编辑器中打开、用默认程序打开文件、在文件夹中显示、复制文件路径、从文件更改创建补丁、编辑文件和删除文件。对于有冲突的文件，它以**在 merge 编辑器中打开**和三个**用……解决**行开头。开启 Git LFS 时，还有 LFS 行。**丢弃更改**和**删除文件**会先确认。

## 窗口按钮

每个按钮都有对应的键盘操作：

| 按钮 | 按键 |
|---|---|
| 标题栏左侧的 🐙（关于） | 面板中的 `帮助：关于 GitPoulpi` |
| 标题栏中的**打开方式** | `Alt+O`, `Ctrl+Alt+O` |
| 标题栏右侧的 `×`（退出） | `q` |
| 标题栏中的版本号（发行说明） | 面板中的 `帮助：发行说明` |
| 标题栏中 `⚙` 左侧的 `⌕ 全局搜索` | `Ctrl+Alt+F` |
| 标题栏中的 `⚙`（设置） | `Ctrl+,` |
| 标签行中的工作空间标记 | `Alt+W` |
| 最后一个标签页之后的 `+` | `Alt+T` |
| **Radar** 标签页 | `Alt+R` |
| **工作空间**标签页 | `Alt+J` |
| 状态栏中的**活动** | `Ctrl+N Shift+A` |
| 工具栏：**撤销**、**重做** | `Ctrl+Z`, `Ctrl+Y` |
| 工具栏：**Pull** / 它的 `▾` | `Ctrl+G p` / `Git: Pull Options...` |
| 工具栏：**Push** / 它的 `▾` | `Ctrl+G Shift+P` / `Git: Push to Remote...` |
| 工具栏：**分支**、**Tag** | `Ctrl+G b`, `Ctrl+G t` |
| 工具栏：**Stash**、**弹出** | `Ctrl+G Shift+T`, `Ctrl+G o` |
| 工具栏：**LFS**（LFS 开启时） | `Ctrl+G l` |
| 工具栏：**Flow**（仓库有工作流时） | `Ctrl+G Shift+W` |
| 工具栏：**搜索** | `f` |
| 工具栏：关联的 pull request（`GH #12`、`GL !53`） | `Ctrl+G g` |
| 工具栏：配置列表（有两个或更多配置时） | `Repository: Change Profile...` |

工具栏在[界面导览](interface-tour.md#工具栏)中有说明。

## 设置

`Ctrl+,`、`Ctrl+N ,`、`⚙` 齿轮图标，或 `应用：设置（首选项）…` 会打开设置界面。

| 按键 | 操作 |
|---|---|
| `Tab` / `Shift+Tab`, `h` / `l`, `Left` / `Right` | 在页面列表和选项之间移动 |
| `j` / `k`, `Up` / `Down` | 移动选择 |
| `Enter` / `Space` | 打开页面、切换复选框、编辑字段、打开下拉列表或按下按钮 |
| `/`, `Ctrl+F` | 搜索所有选项 |
| `Alt+Left` / `Alt+Right` | 切换子标签页（常规 / 列表） |
| `Esc` / `q` | 关闭设置。正在编辑字段时，`Esc` 只取消编辑 |
| `?` | 在设置界面上打开帮助 |

参见[设置](settings.md)。

## 文本框

所有文本框使用相同的按键：筛选框、搜索框、命令面板、提示、commit 编辑区、WIP 行的文本框、对话框字段和设置字段。当文本框获得键盘时，这些按键的优先级高于窗格快捷键。

| 按键 | 操作 |
|---|---|
| 一个字符 | 在光标处输入（替换选区） |
| `Left` / `Right` | 移动一个字符（有选区时：跳到选区的开头 / 结尾） |
| `Ctrl+Left` / `Ctrl+Right` | 移动一个单词 |
| `Home` / `End` | 行首 / 行尾 |
| `Shift` + 上述任一移动键 | 扩大或缩小选区 |
| `Up` / `Down`（仅限描述） | 上一行 / 下一行 |
| `Ctrl+Home` / `Ctrl+End`（仅限描述） | 文本开头 / 结尾 |
| `Ctrl+A` | 全选 |
| `Backspace` / `Delete` | 删除光标之前 / 之后的字符，或删除选区 |
| `Ctrl+U` | 从光标处删除到行首（或删除选区） |
| `Ctrl+K` | 从光标处删除到行尾（或删除选区） |
| `Ctrl+C` / `Ctrl+X` | 复制 / 剪切选区（密码框中不可用） |
| 粘贴（终端自带的粘贴） | 在光标处插入，并替换选区 |

在 commit 描述中，`Enter` 输入换行，`Ctrl+Enter` commit。不同终端发送 `Ctrl+Enter` 的方式并不相同，因此 `Alt+Enter` 和 `Ctrl+J` 也可以 commit。

有些控制台会将 `Ctrl+字母` 作为控制字符发送（`Ctrl+A` 发送为 `0x01`）。GitPoulpi 两种形式都能识别。`AltGr` 字符（瑞士或法语键盘上的 `@`、`#`、`€`）可以正常输入。

## 工具提示

当指针在目标（Log 单元格、工具栏按钮、关联的 pull request）上停留约 0.6 秒时，会出现提示。按键、点击、滚动、调整大小、打开弹出窗口或菜单，或指针离开目标，都会将其隐藏。此后，只有指针再次移动时它才会重新出现。
