**English** · [Čeština](cs/keybindings.md) · [Deutsch](de/keybindings.md) · [Español](es/keybindings.md) · [Français](fr/keybindings.md) · [Italiano](it/keybindings.md) · [日本語](ja/keybindings.md) · [한국어](ko/keybindings.md) · [Nederlands](nl/keybindings.md) · [Polski](pl/keybindings.md) · [Português (Brasil)](pt-BR/keybindings.md) · [Português (Portugal)](pt-PT/keybindings.md) · [Русский](ru/keybindings.md) · [Svenska](sv/keybindings.md) · [Türkçe](tr/keybindings.md) · [Українська](uk/keybindings.md) · [简体中文](zh-CN/keybindings.md)

# Keybindings

This page lists the **default** keys. You can rebind most of them. The app
always shows the keys in effect: in the command palette, the `?` help, the
status lines, the context menus and the button tooltips. When you move a key, these
places show the new key. For the mouse, see
[Mouse and gestures](mouse-and-gestures.md).

## Rebinding

Open **Settings > Keyboard & mouse**. The page is described in
[Settings](settings.md#keyboard--mouse). Changes apply at once and are saved
in the `[keybindings]` table of `config.toml`. You can also edit that table by
hand:

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # tmux and GNU screen often use Ctrl+B and Ctrl+A
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

The syntax, the conflict rules and the warnings are in
[Configuration reference](config-reference.md) under `[keybindings]`. In short:

- A mistake is reported on the status line, and the default key stays.
- Two actions on one chord in the same context are refused.
- A prefix needs `Ctrl` or `Alt`.
- A pane chord that hides a global chord only gives a warning.

The table below lists every key you can rebind. Some keys are fixed and cannot
be rebound: list motion (`j` `k` `h` `l`, the arrows, `PageUp` / `PageDown`,
`Home` / `End`), `Enter`, `Esc`, `Tab`, the digits (the count prefix), the
text-editing keys, and the `Enter` / `Tab` / arrow keys of popups. The palette
has `App: Keybindings...` and `Keybindings: Reset All...`.

## Default keybindings

Every key you can rebind, with the id to use in `[keybindings.<context>]` and
its default. The table is generated from the app (a test keeps it current).
Keys after a prefix are shown with the prefix (`Ctrl+G s`).

<!-- keymap:begin -->
| Context | Action id | What it does | Default keys |
|---|---|---|---|
| `global` | `git_prefix` | Git Actions Prefix | `Ctrl+G` |
| `global` | `nav_prefix` | Navigation Prefix | `Ctrl+N` |
| `global` | `palette` | Command Palette | `Ctrl+P` |
| `global` | `settings` | Settings | `Ctrl+,` |
| `global` | `home` | Home (Repository Management) | `Ctrl+O` |
| `global` | `clone` | Clone Repository | `Ctrl+Shift+N` |
| `global` | `new_tab` | New Tab | `Alt+T` |
| `global` | `workspace_switch` | Switch Workspace | `Alt+W` |
| `global` | `radar` | Open Radar | `Alt+R` |
| `global` | `workspaces` | Open Workspaces | `Alt+J` |
| `global` | `tab_menu` | Tab Menu | `Ctrl+T` |
| `global` | `tab_open_in_popup` | Open In (list) | `Alt+O` |
| `global` | `tab_open_in_default` | Open in Default | `Ctrl+Alt+O` |
| `global` | `close_tab` | Close Tab | `Ctrl+W` |
| `global` | `next_tab` | Next Tab | `Ctrl+PageDown` |
| `global` | `prev_tab` | Previous Tab | `Ctrl+PageUp` |
| `global` | `help` | Help | `?` |
| `global` | `quit` | Quit | `q` |
| `global` | `refresh` | Refresh | `r` |
| `global` | `refresh_all` | Refresh (fetch, pull requests, issues) | `F5` |
| `global` | `find_in_log` | Search the Commit Graph | `f` |
| `global` | `workspace_search` | Search the Workspace | `Ctrl+Alt+F` |
| `global` | `select_tab` | Select Tab | `t` |
| `global` | `undo` | Undo the Last Operation | `Ctrl+Z` |
| `global` | `redo` | Redo the Undone Operation | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `global` | `undo_history` | Open the Undo History | `Alt+Z` |
| `global` | `commit_column` | Show or Hide the Commit Column | `Alt+C` |
| `global` | `undo_offer` | Restore the Branch to Where the Blocked Action Left It | `Alt+Shift+Z` |
| `global` | `context_menu` | Context Menu | `.`, `Menu`, `Shift+F10` |
| `git_prefix` | `compose_commit` | Compose Commit Message | `Ctrl+G c` |
| `git_prefix` | `commit_now` | Commit Staged Changes Now | `Ctrl+G Enter` |
| `git_prefix` | `ai_compose` | AI: Split Changes into Commits | `Ctrl+G Shift+C` |
| `git_prefix` | `ai_explain` | AI: Explain Changes | `Ctrl+G w` |
| `git_prefix` | `stash_message` | Stash with Message | `Ctrl+G m` |
| `git_prefix` | `ai_commit_message` | AI: Write Commit Message | `Ctrl+G i` |
| `git_prefix` | `stage_all` | Stage All Changes | `Ctrl+G a` |
| `git_prefix` | `discard_all` | Discard All Changes | `Ctrl+G x` |
| `git_prefix` | `stage_file` | Stage Selected File | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | Unstage Selected File | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | Open Linked Pull Request | `Ctrl+G g` |
| `git_prefix` | `push_tags` | Push All Tags | `Ctrl+G Shift+U` |
| `git_prefix` | `fetch_all` | Fetch All Remotes | `Ctrl+G f` |
| `git_prefix` | `new_tag` | New Tag | `Ctrl+G t` |
| `git_prefix` | `new_branch` | New Branch | `Ctrl+G b` |
| `git_prefix` | `stash_push` | Stash Changes | `Ctrl+G Shift+T` |
| `git_prefix` | `cherry_pick` | Cherry-Pick Selected Commit | `Ctrl+G y` |
| `git_prefix` | `revert` | Revert Selected Commit | `Ctrl+G v` |
| `git_prefix` | `edit_message` | Edit Commit Message | `Ctrl+G e` |
| `git_prefix` | `amend` | Amend Selected Commit | `Ctrl+G Shift+E` |
| `git_prefix` | `edit_author` | Amend Selected Commit Author | `Ctrl+G Shift+A` |
| `git_prefix` | `reset_mixed` | Reset (Mixed) to Selected Commit | `Ctrl+G Shift+R` |
| `git_prefix` | `rebase_onto` | Rebase onto Selected Branch | `Ctrl+G r` |
| `git_prefix` | `stash_pop` | Pop Stash | `Ctrl+G o` |
| `git_prefix` | `delete_ref` | Delete Selected Ref or Drop Stash | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | Open the LFS Menu | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | Open the Flow Menu | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS: Fetch All Objects | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS: Check Out All Files | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS: Pull Objects | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS: Push All Objects | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS: Prune Local Objects | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | Open the History Rewrite Workbench | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | Collapse or Restore Refs Pane | `Ctrl+N h` |
| `nav_prefix` | `collapse_status` | Collapse or Restore Status Pane | `Ctrl+N l` |
| `nav_prefix` | `settings` | Open Settings | `Ctrl+N ,` |
| `nav_prefix` | `about` | Open About | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | Open the Activity Log | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | Open the global Activity Log | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | Show the Background Jobs | `Ctrl+N Shift+J` |
| `nav_prefix` | `updates` | Show the Update Panel | `Ctrl+N u` |
| `nav_prefix` | `help` | Toggle Help | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | Select Tab | `Ctrl+N t` |
| `nav_prefix` | `change_theme` | Change Theme | `Ctrl+N y` |
| `nav_prefix` | `change_language` | Change Language | `Ctrl+N g` |
| `nav_prefix` | `change_graph_color_theme` | Change Graph Color Theme | `Ctrl+N Shift+Y` |
| `nav_prefix` | `toggle_graph_row_tints` | Toggle Graph Row Tints | `Ctrl+N Shift+T` |
| `nav_prefix` | `grow_section` | Grow Refs Section | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | Shrink Refs Section | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | Grow Commit Message Block | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | Shrink Commit Message Block | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | Widen Refs Pane | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | Narrow Refs Pane | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | Widen Status Pane | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | Narrow Status Pane | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | Agent: Show the Pending View | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | Agent: Back (or Dismiss the Pending View) | `Ctrl+N Backspace` |
| `refs` | `filter` | Filter Refs by Name | `/` |
| `refs` | `solo` | Solo the Selection | `o` |
| `refs` | `clear_solo` | Stop Soloing | `Shift+O` |
| `refs` | `toggle_hierarchy` | Toggle Branch Hierarchy View | `y` |
| `refs` | `show_all` | Show All Hidden Refs | `Shift+H` |
| `refs` | `reveal` | Reveal in Graph | `g` |
| `refs` | `mark_drop` | Mark as Drop Source | `m` |
| `refs` | `drop_marked` | Drop the Marked Ref Here | `Shift+M` |
| `refs` | `expand_all` | Expand All Folders | `=` |
| `refs` | `collapse_all` | Collapse All Folders | `-` |
| `refs` | `expand_below` | Expand the Folder and Its Subfolders | `*`, `Shift+Right` |
| `refs` | `collapse_below` | Collapse the Folder and Its Subfolders | `_`, `Shift+Left` |
| `log` | `toggle_file_mode` | Files: Path or Tree | `m` |
| `log` | `view_all_files` | Files: View All Files | `a` |
| `log` | `compare_selection` | Compare the Selected Rows | `v` |
| `log` | `compare_swap` | Swap the Comparison Ends | `x` |
| `log` | `compare_next` | Go to the Next Compared Row | `n` |
| `log` | `pick_column` | Pick a Column to Move or Size | `Shift+C` |
| `log` | `scroll_left` | Scroll the Columns Left | `Shift+Left` |
| `log` | `scroll_right` | Scroll the Columns Right | `Shift+Right` |
| `log` | `copy_mermaid` | Copy Commits as a Mermaid gitGraph | `Shift+Y` |
| `log_columns` | `next` | Pick the Next Column | `Tab` |
| `log_columns` | `prev` | Pick the Previous Column | `Shift+BackTab` |
| `log_columns` | `move_left` | Move the Column Left | `<` |
| `log_columns` | `move_right` | Move the Column Right | `>` |
| `log_columns` | `wider` | Widen the Column | `]` |
| `log_columns` | `narrower` | Narrow the Column | `[` |
| `log_columns` | `reset_width` | Reset the Column Widths | `0` |
| `log_columns` | `fit` | Fit the Column to Its Content | `=` |
| `log_columns` | `fit_all` | Fit All Columns to Content | `+` |
| `log_columns` | `layout` | Scroll Sideways or Fit the Pane | `s` |
| `log_files` | `collapse_all` | Collapse or Expand All Folders | `z` |
| `log_files` | `filter` | Filter the Changed Files | `/` |
| `log_files` | `expand_all` | Expand All Folders | `=` |
| `log_files` | `collapse_all_folders` | Collapse All Folders | `-` |
| `log_files` | `expand_below` | Expand the Folder and Its Subfolders | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | Collapse the Folder and Its Subfolders | `_`, `Shift+Left` |
| `status` | `compose` | Focus the Commit Message | `c` |
| `status` | `toggle_file_mode` | Files: Path or Tree | `m` |
| `status` | `view_all_files` | Files: View All Files | `a` |
| `status` | `amend` | Toggle Amend | `Shift+A` |
| `status` | `skip_hooks` | Toggle Skip Hooks | `Shift+H` |
| `status` | `collapse_all` | Collapse or Expand All Folders | `z` |
| `status` | `stage_toggle` | Stage or Unstage the Selected Row | `Ctrl+Space` |
| `status` | `open_diff` | Open the Selected File's Diff | `d` |
| `status` | `expand_all` | Expand All Folders | `=` |
| `status` | `collapse_all_folders` | Collapse All Folders | `-` |
| `status` | `expand_below` | Expand the Folder and Its Subfolders | `*`, `Shift+Right` |
| `status` | `collapse_below` | Collapse the Folder and Its Subfolders | `_`, `Shift+Left` |
| `home` | `favorite` | Star or Unstar the Repository | `f`, `*` |
| `home` | `close_repo` | Close, Remove or Unstar the Row | `x`, `Delete` |
| `home` | `browse` | Browse for a Repository | `b` |
| `home` | `clone` | Clone a Repository | `c` |
| `home` | `init` | Initialize a Repository | `i` |
| `home` | `collapse_all` | Collapse All Sections | `-` |
| `home` | `expand_all` | Expand All Sections | `+`, `=` |
| `home` | `search` | Search the Repositories | `/` |
| `home` | `workspace_new` | New Workspace | `n` |
| `home` | `workspace_edit` | Edit the Workspace | `e` |
| `home` | `workspace_from_tabs` | New Workspace from the Open Tabs | `a` |
| `home` | `workspace_fetch` | Fetch All in the Workspace | `Shift+F` |
| `home` | `workspace_pull` | Pull All in the Workspace | `Shift+P` |
| `home` | `workspace_clone_missing` | Clone the Missing Repositories | `m` |
| `home` | `workspace_locate` | Locate the Missing Repository | `l` |
| `refs` | `hosting_filter` | Pull Requests / Issues: Choose the Filter | `v` |
| `refs` | `hosting_edit` | Pull Request: Edit on the Host | `e` |
| `refs` | `new_pull_request` | Pull Requests: Create One | `n` |
| `refs` | `add_remote` | Remotes: Add a Remote | `+` |
| `diff` | `close` | Close the Diff | `q` |
| `diff` | `edit` | Edit in Working Directory | `e` |
| `diff` | `lfs_raw` | LFS: Card or Raw Pointer | `r` |
| `diff` | `copy` | Copy the Selection or Line | `y`, `Ctrl+C` |
| `diff` | `stage_hunk` | Stage or Unstage the Hunk | `Space` |
| `diff` | `stage_lines` | Stage or Unstage the Selected Lines | `Enter` |
| `diff` | `discard_hunk` | Discard the Hunk | `x` |
| `diff` | `discard_lines` | Discard the Selected Lines | `Shift+X` |
| `diff` | `select_lines` | Start or Drop a Line Selection | `v` |
| `diff` | `next_hunk` | Next Hunk | `n` |
| `diff` | `prev_hunk` | Previous Hunk | `p` |
| `diff` | `toggle_layout` | Layout: Unified or Side by Side | `m` |
| `diff` | `toggle_hunks_only` | Hunks Only or Whole File | `Shift+H` |
| `diff` | `toggle_minimap` | Toggle the Minimap | `Shift+M` |
| `diff` | `choose_encoding` | Choose the Encoding | `Shift+C` |
| `diff` | `toggle_wrap` | Toggle Word Wrap | `w` |
| `diff` | `toggle_whitespace` | Show or Hide Whitespace-Only Changes | `s` |
| `diff` | `merge_editor` | Open the Merge Editor (Conflicted File) | `Shift+E` |
| `file_insight` | `close` | Close the File View | `q` |
| `file_insight` | `edit` | Edit in Working Directory | `e` |
| `file_insight` | `copy` | Copy the Selection or Row | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | Jump to the Last Row | `Shift+G` |
| `file_insight` | `open_commit` | Go to the Row's Commit | `Enter` |
| `history` | `show_in_log` | Show in the Log | `Enter` |
| `history` | `copy_sha` | Copy the commit SHA | `y` |
| `history` | `copy_summary` | Copy the Summary | `Shift+Y` |
| `history` | `show_diff` | Show the diff of This commit | `d`, `Alt+1` |
| `history` | `open_file` | Open the File at This commit | `o`, `Alt+2` |
| `history` | `blame` | Blame at This commit | `b`, `Alt+3` |
| `history` | `compare_working_tree` | Compare with the Working Tree | `c` |
| `history` | `restore` | Restore This Version | `Shift+R` |
| `history` | `range_up` | Extend the Selection Up | `Shift+Up` |
| `history` | `range_down` | Extend the Selection Down | `Shift+Down` |
| `history` | `switch_focus` | Switch Between the List and the Diff | `Tab` |
| `history` | `widen_list` | Widen the List | `Alt+Right` |
| `history` | `narrow_list` | Narrow the List | `Alt+Left` |
| `history` | `reset_split` | Reset the List Width | `Alt+0` |
| `commit_column` | `scroll_left` | Scroll the Column Left | `Shift+Left` |
| `commit_column` | `scroll_right` | Scroll the Column Right | `Shift+Right` |
| `commit_column` | `next_part` | Size the Next Part | `Tab` |
| `commit_column` | `move_left` | Move the Part Left | `<` |
| `commit_column` | `move_right` | Move the Part Right | `>` |
| `commit_column` | `wider_part` | Widen the Part | `]` |
| `commit_column` | `narrower_part` | Narrow the Part | `[` |
| `commit_column` | `reset_widths` | Reset the Part Widths | `0` |
| `file_insight` | `blame_card` | Blame: Show or Hide the Detail Card | `i` |
| `file_insight` | `blame_next` | Blame: Next Hunk | `n` |
| `file_insight` | `blame_prev` | Blame: Previous Hunk | `p` |
| `file_insight` | `blame_next_same` | Blame: Next Hunk of the Same Commit | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame: Previous Hunk of the Same Commit | `Shift+P` |
| `file_insight` | `blame_goto` | Blame: Go to Line | `g` |
| `file_insight` | `blame_copy_sha` | Blame: Copy Commit SHA | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame: Show the Commit's Diff for This File | `d` |
| `file_insight` | `blame_parent` | Blame: Blame at the Commit's Parent | `b` |
| `file_insight` | `blame_history` | Blame: Open File History | `Shift+H` |
| `file_insight` | `blame_wider` | Blame: Widen the Annotation Column | `>` |
| `file_insight` | `blame_narrower` | Blame: Narrow the Annotation Column | `<` |
| `image` | `zoom_in` | Zoom the Image In | `+`, `=` |
| `image` | `zoom_out` | Zoom the Image Out | `-` |
| `image` | `fit` | Fit the Image to the View | `0` |
| `image` | `actual_size` | Show the Image at Actual Size | `1` |
| `image` | `flip_side` | Show the Other Side (Before / After) | `Tab` |
| `image` | `widen_before` | Widen the Before Pane | `Alt+Right` |
| `image` | `widen_after` | Widen the After Pane | `Alt+Left` |
| `image` | `reset_split` | Reset the Before / After Split | `Alt+0` |
| `image` | `edit_in` | Open With... | `Shift+O` |
| `image` | `edit_default` | Open in the Default App | `o` |
| `editor` | `save` | Save | `Ctrl+S` |
| `editor` | `undo` | Undo | `Ctrl+Z` |
| `editor` | `redo` | Redo | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | Find | `Ctrl+F` |
| `editor` | `find_next` | Find Next (Shift: Previous) | `F3` |
| `editor` | `paste` | Paste | `Ctrl+V` |
| `editor` | `toggle_minimap` | Toggle the Minimap | `Alt+M` |
| `editor_prompt` | `save` | Unsaved Changes: Save and Close | `s`, `Shift+S` |
| `editor_prompt` | `discard` | Unsaved Changes: Discard and Close | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | Unsaved Changes: Keep Editing | `c`, `Shift+C` |
| `merge` | `next_conflict` | Next Conflict | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | Previous Conflict | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | Next Conflicted File | `Alt+PageDown` |
| `merge` | `prev_file` | Previous Conflicted File | `Alt+PageUp` |
| `merge` | `next_pane` | Focus the Next Pane | `F6` |
| `merge` | `prev_pane` | Focus the Previous Pane | `Shift+F6` |
| `merge` | `toggle_panes` | Show or Hide the Side Panels (Refs and Status) | `F9` |
| `merge` | `toggle_base` | Show or Hide the Base Pane | `Alt+D` |
| `merge` | `toggle_full_file` | Toggle Full File Diff | `Alt+F` |
| `merge` | `undo` | Undo (Picks and Typing) | `Ctrl+Z` |
| `merge` | `redo` | Redo (Picks and Typing) | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | Widen the Current (A) Pane | `Alt+Right` |
| `merge` | `widen_incoming` | Widen the Incoming (B) Pane | `Alt+Left` |
| `merge` | `reset_panes` | Reset the Pane Sizes | `Alt+0` |
| `merge` | `save` | Save without Marking Resolved | `Alt+S` |
| `merge` | `save_resolve` | Save and Mark Resolved | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ Suggest a Resolution (AI) | `Alt+I` |
| `merge` | `unpick` | Unpick This Conflict (Back to Unresolved) | `Alt+U` |
| `merge` | `reset_all` | Reset the Whole File to the Original Conflicts | `Alt+R` |
| `merge` | `focus_a` | Focus the Current (A) Pane | `Alt+1` |
| `merge` | `focus_b` | Focus the Incoming (B) Pane | `Alt+2` |
| `merge` | `focus_output` | Focus the Output Pane | `Alt+3` |
| `merge` | `focus_base` | Focus the Base Pane | `Alt+4` |
| `merge_sides` | `toggle_line` | Pick or Drop the Line | `Space` |
| `merge_sides` | `toggle_chunk` | Pick or Drop the Whole Chunk | `Enter` |
| `merge_sides` | `take_a` | Take Current (A) Only | `a` |
| `merge_sides` | `take_b` | Take Incoming (B) Only | `b` |
| `merge_sides` | `take_both` | Take Both, A Then B | `c` |
| `merge_sides` | `take_both_reversed` | Take Both, B Then A | `Shift+C` |
| `merge_sides` | `clear` | Take Nothing for This Conflict | `x` |
| `merge_sides` | `unpick` | Unpick This Conflict (Back to Unresolved) | `u` |
| `merge_sides` | `reset_all` | Reset the Whole File to the Original Conflicts | `Shift+U` |
| `merge_sides` | `swap_order` | Swap the Order of the Two Sides | `s` |
| `merge_sides` | `next_conflict` | Next Conflict | `n` |
| `merge_sides` | `prev_conflict` | Previous Conflict | `Shift+N` |
| `merge_sides` | `take_all_a` | Take All Current (A) | `Shift+A` |
| `merge_sides` | `take_all_b` | Take All Incoming (B) | `Shift+B` |
| `merge_sides` | `toggle_base` | Show or Hide the Base Pane | `d` |
| `merge_sides` | `scroll_left` | Scroll the Panes Left | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | Scroll the Panes Right | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | Toggle Full File Diff | `f` |
| `merge_sides` | `cycle_height` | Cycle the Output Height | `z` |
| `merge_card` | `take_a` | Keep A (Current) as Is | `a` |
| `merge_card` | `take_b` | Take B (Incoming) as Is | `b` |
| `merge_card` | `take_base` | Take the Base (Common Ancestor) | `o` |
| `merge_card` | `keep` | Keep the File That Is Left | `k` |
| `merge_card` | `delete` | Accept the Deletion | `Shift+D` |
| `merge_card` | `external_tool` | Open in the External Merge Tool | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit (Stop to Amend) | `e` |
| `rebase` | `autosquash` | Toggle Autosquash | `a` |
| `rebase` | `start` | Start the Plan | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | Reset the Plan | `Ctrl+R` |
| `rebase` | `split` | Split the Row's Commit | `t` |
| `rebase` | `move_up` | Move the Row Up | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | Move the Row Down | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | Create the Commits | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | Toggle Line Selection | `v` |
| `split` | `move_up` | Move the Commit Up | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | Move the Commit Down | `Alt+Down`, `Shift+J` |
| `split` | `reset` | Reset the Split | `Shift+R` |
| `rewrite` | `preview` | Preview the Rewrite | `F5` |
| `rewrite` | `apply` | Apply the Rewrite | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | Cancel the Preview | `Ctrl+X` |
| `rewrite` | `add_rule` | Add a Rule | `a`, `Insert` |
| `rewrite` | `toggle_rule` | Switch the Rule On or Off | `Space` |
| `rewrite` | `move_up` | Move the Rule Up | `Alt+Up` |
| `rewrite` | `move_down` | Move the Rule Down | `Alt+Down` |
| `rewrite` | `delete_rule` | Delete the Rule | `d`, `Delete` |
| `rewrite` | `restore_rule` | Restore the Deleted Rule | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | Duplicate the Rule | `Ctrl+D` |
| `rewrite` | `next_tab` | Next Preview Tab | `Alt+Right` |
| `rewrite` | `prev_tab` | Previous Preview Tab | `Alt+Left` |
| `rewrite` | `save_shared` | Save the Recipe to the Repository | `Alt+S` |
| `rewrite` | `save_mine` | Save the Recipe for Me Only | `Alt+M` |
| `rewrite` | `load` | Load a Recipe | `Alt+L` |
| `rewrite` | `name` | Rename the Recipe | `F2` |
| `rewrite` | `protected_all` | Include All Protected Branches | `Alt+A` |
| `rewrite` | `protected_none` | Leave Out All Protected Branches | `Alt+U` |
| `rewrite` | `check_tools` | Check the Tools Again | `F6` |
| `rewrite` | `analyze` | Analyse the History Sizes | `F7` |
| `rewrite` | `host_howto` | Explain the Host's Branch Protection | `Alt+H` |
| `rewrite` | `ai_rules` | Create Rewrite Rules with AI | `Alt+I` |
| `reword` | `apply` | Apply the New Message | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | Modify History Deeper | `Alt+H` |
| `composer` | `commit` | Commit from the Composer | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | Insert an Issue Reference | `Alt+R` |
| `dialog` | `browse` | Browse for a Folder | `Ctrl+B` |
| `dialog` | `sources` | Go to the Clone Sources | `Alt+S` |
| `dialog` | `submit` | Submit the Pull Request Form | `Ctrl+S` |
| `dialog` | `web` | Open the Provider's Web Form | `Alt+W` |
| `folder_picker` | `choose` | Choose the Folder | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | New Folder | `Alt+N` |
| `folder_picker` | `filter` | Filter the Folder's Entries | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | Show or Hide Sibling Folders | `Alt+S` |
| `folder_picker` | `toggle_hidden` | Show or Hide Hidden Folders | `Alt+H` |
| `preferences` | `search` | Search the Settings | `Ctrl+F`, `/` |
| `preferences` | `list_add` | Add a List Item | `Insert` |
| `preferences` | `list_duplicate` | Duplicate the Selected List Item | `Ctrl+D` |
| `preferences` | `list_remove` | Remove the Selected List Item | `Delete` |
| `preferences` | `list_move_up` | Move the Selected List Item Up | `Alt+Up` |
| `preferences` | `list_move_down` | Move the Selected List Item Down | `Alt+Down` |
| `preferences` | `prev_tab` | Previous Tab of the Settings Page | `Alt+Left` |
| `preferences` | `next_tab` | Next Tab of the Settings Page | `Alt+Right` |
| `ai_popup` | `copy` | Copy the Result | `c` |
| `integrations` | `add` | Add a Profile | `a` |
| `integrations` | `edit` | Edit the Selected Profile | `e` |
| `integrations` | `open_token` | Open the Token Page | `o` |
| `integrations` | `remove` | Remove the Selected Profile | `d`, `Delete` |
| `prompt` | `ai_stash_message` | AI: Generate Stash Message | `Ctrl+G` |
<!-- keymap:end -->

### Keys the terminal cannot send, and the kitty keyboard protocol

A legacy terminal cannot send some chords: `Ctrl+,`, `Ctrl+Shift+N`,
`Ctrl+Enter` and `Ctrl+<digit>`. It sends nothing, or another key: for
example, `Ctrl+Shift+N` arrives as `Ctrl+N`, `Ctrl+I` as `Tab` and `Ctrl+M` as
`Enter`. The Settings page warns about such a binding ("this terminal does not
send ...").

Terminals with the **kitty keyboard protocol** can send these chords. Examples:
kitty, WezTerm, Ghostty, foot, Alacritty 0.13 and later, and tmux or herdr when
they pass the protocol through. GitPoulpi uses the protocol by default:

- At start-up, GitPoulpi asks the terminal and waits at most about 100 ms for
  the answer. If the terminal does not answer, the legacy keys stay.
- If the terminal answers, GitPoulpi turns the protocol on. It turns it off
  again when it exits, fails or crashes.
- The warnings on the Settings page follow the protocol: when it is on, the
  chords above are sendable.
- **Settings > Keyboard & mouse > Kitty keyboard protocol** (`ui.kitty_keyboard`)
  set to `off` never changes the terminal's keyboard mode. Use it if a terminal
  or a multiplexer misbehaves. A restart is needed.

Windows consoles report the modifiers themselves and do not need the protocol.

## Command palette (`Ctrl+P`)

The palette lists **every** command of the app, so it reaches everything
without a shortcut. The full list is [below](#every-palette-command).

| Key | Action |
|---|---|
| `Ctrl+P` | Open the palette |
| *(type)* | Filter the list. An exact match after the `Category: ` prefix comes first, then a match anywhere in the label, then a fuzzy match |
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J`, `Home` / `End`, `PageUp` / `PageDown` | Move the highlight (see [Filterable popup lists](#filterable-popup-lists)) |
| `Enter`, or a double-click | Run the highlighted command |
| `Esc` | Close |

`j` and `k` are typed into the filter. They do not move the highlight.

Each row shows its shortcut on the right. A command that does not apply now
(for example, no ref selected) stays in the list, grayed out, and the reason
shows under the list. Labels start with a category, such as `Navigation:`,
`Refs:`, `Git:`, `View:`, `Diff:` or `App:`. Type a category to see its whole
group.

### A key for any command

Every palette row ends with a key button (`[k]`), and `Alt+K` does the same on the highlighted row: it opens **Settings > Keyboard & mouse** filtered on that command's shortcut, ready to rebind. A command that has no shortcut of its own gets one here: it is listed under **Palette commands**, unbound by default, and a key you give it runs the command exactly as choosing it in the palette does (a command that cannot run now says why). The row then shows your key.

In `config.toml` these are the `[keybindings.palette]` table, named after the command (`toggle_minimap = "Ctrl+Alt+M"`). A key that clashes with another action is shown in red, and `c` on its row lists every action it clashes with.

The hint band under the list names this key (`Alt+K` by default). It is the action `palette.configure_shortcut`, so it can be rebound like any other.

### Every palette command

Generated from the app (a test keeps it current). AI commands only appear
while AI is on; the others are grayed, with the reason, when they do not apply.
Keys are the defaults.

Settings has rows of its own that are not listed here: one `Settings: Toggle <setting>` row per on/off setting, `Settings: <setting>...` per setting that opens a picker or a field, and `Settings: <setting>: <choice>` per choice. They are generated from the Settings screen, so the list always matches it; type `settings` in the palette to see them.

<!-- palette:begin -->
| Command | Default key |
|---|---|
| Navigation: Focus Refs Pane | `Ctrl+N <badge>` |
| Navigation: Focus Log Pane | `Ctrl+N <badge>` |
| Navigation: Focus Status Pane | `Ctrl+N <badge>` |
| Navigation: Focus Refs Filter | `Ctrl+N <badge>` |
| Navigation: Focus Refs > Local | `Ctrl+N <badge>` |
| Navigation: Focus Refs > Remote | `Ctrl+N <badge>` |
| Navigation: Focus Refs > Worktrees | `Ctrl+N <badge>` |
| Navigation: Focus Refs > Submodules | `Ctrl+N <badge>` |
| Navigation: Focus Refs > Stashes | `Ctrl+N <badge>` |
| Navigation: Focus Refs > Tags | `Ctrl+N <badge>` |
| Left pane: Show/Hide Sections... |  |
| Left pane: Reset Sections to Default |  |
| Left pane: Apply These Sections to All Repositories |  |
| Left pane: Show/Hide Local Branches |  |
| Left pane: Show/Hide Remotes |  |
| Left pane: Show/Hide Worktrees |  |
| Left pane: Show/Hide Stashes |  |
| Left pane: Show/Hide Tags |  |
| Left pane: Show/Hide Pull requests |  |
| Left pane: Show/Hide Issues |  |
| Left pane: Show/Hide Submodules |  |
| Navigation: Focus Status > Unstaged | `Ctrl+N <badge>` |
| Navigation: Focus Status > Staged | `Ctrl+N <badge>` |
| Navigation: Cycle Focus to Next Pane | `Tab` |
| Navigation: Cycle Focus to Previous Pane | `Shift+Tab` |
| Navigation: Search Commits... | `f` |
| Navigation: Toggle Refs Pane Collapsed | `Ctrl+N h` |
| Navigation: Widen Refs Pane | `Ctrl+N Right` |
| Navigation: Narrow Refs Pane | `Ctrl+N Left` |
| Navigation: Widen Status Pane | `Ctrl+N Shift+Left` |
| Navigation: Narrow Status Pane | `Ctrl+N Shift+Right` |
| Navigation: Toggle Status Pane Collapsed | `Ctrl+N l` |
| Navigation: Grow Refs Section | `Ctrl+N +` |
| Navigation: Shrink Refs Section | `Ctrl+N -` |
| Navigation: Reset Refs Section Sizes |  |
| Navigation: Reset Layout |  |
| Commit Details: Grow Message | `Ctrl+N ]` |
| Commit Details: Shrink Message | `Ctrl+N [` |
| Commit Details: Reset Message Size |  |
| Commit Details: Scroll Message Down |  |
| Commit Details: Scroll Message Up |  |
| Files: Toggle Commit File Sort (A-Z / Z-A) |  |
| Refs: Check Out Selected Ref | `double-click` |
| Refs: Copy Ref Name |  |
| Refs: Copy Full Ref (refs/heads/...) |  |
| Refs: Copy Ref Target SHA |  |
| Refs: Merge Selected Branch into Current |  |
| Refs: Rebase Current Branch onto Selected | `Ctrl+G r` |
| Refs: Drop Selected Ref on Checked-Out Branch... |  |
| Refs: Mark as Drop Source | `m` |
| Refs: Drop Marked Ref Here... | `M` |
| Refs: Rename Selected Branch... |  |
| Refs: Fast-Forward Selected Branch to Current |  |
| Refs: Fast-Forward Current Branch to Selected |  |
| Refs: Create Worktree from Selected Branch or Tag... |  |
| Refs: Reflog of the Selected Branch... |  |
| Refs: Start a Pull Request from Selected Branch... |  |
| Refs: Open Worktree in Tab | `Enter / double-click` |
| Refs: Submodule Update (Selected) |  |
| Refs: Submodule Init (Selected) |  |
| Refs: Submodule Open in Tab | `Enter / double-click` |
| Git: Update All Submodules |  |
| Git: Create Worktree at Selected Commit... |  |
| Git: Create Patch from Selected Commit... |  |
| Git: Create Patch from Selected Commit to HEAD... |  |
| Git: Create Patch from Working Tree Changes... |  |
| Rewrite: Purge Old History... |  |
| Git: Apply Patch... |  |
| Rewrite: History Workbench | `Ctrl+G H` |
| Rewrite: Replace an Author... |  |
| Rewrite: Anonymise Authors... |  |
| Rewrite: Apply .mailmap to History... |  |
| Rewrite: Merge Identities (.mailmap)... |  |
| Rewrite: Shift Commit Dates... |  |
| Rewrite: Change Commit Time Zone... |  |
| Rewrite: Keep Commits Within Hours... |  |
| Rewrite: Set Committer Date to Author Date |  |
| Rewrite: Search and Replace in Messages... |  |
| Rewrite: Delete Message Lines... |  |
| Rewrite: Add or Remove Trailers... |  |
| Rewrite: Remove Signatures... |  |
| Rewrite: Fix Message Encoding... |  |
| Rewrite: Search and Replace in Files... |  |
| Rewrite: Remove Files from History... |  |
| Rewrite: Remove Large Files from History... |  |
| Rewrite: Move or Rename Paths... |  |
| Rewrite: Migrate Files to LFS... |  |
| Rewrite: Bring Files Back from LFS... |  |
| Rewrite: Convert Line Endings... |  |
| Rewrite: Remove the Executable Bit... |  |
| Rewrite: Convert File Encoding... |  |
| Rewrite: Run a Script... |  |
| ✦ Rewrite: Write a Script with AI... |  |
| ✦ Rewrite: Create Rules with AI... |  |
| Rewrite: From the Selected Commit to HEAD... |  |
| Rewrite: Selected Commits... |  |
| LFS: Initialize for This Repository |  |
| LFS: Track Pattern... |  |
| LFS: Untrack Pattern... |  |
| LFS: Fetch Objects |  |
| LFS: Open Menu | `Ctrl+G l` |
| LFS: Fetch All Objects | `Ctrl+G F` |
| LFS: Check Out All Files | `Ctrl+G K` |
| LFS: Push All Objects | `Ctrl+G J` |
| LFS: Pull Objects | `Ctrl+G L` |
| LFS: Push Objects |  |
| LFS: Prune Local Objects | `Ctrl+G V` |
| LFS: Show Locks |  |
| LFS: Open Install Page |  |
| LFS: Toggle Raw Pointer in Diff | `r` |
| LFS: Track Selected File |  |
| LFS: Track Selected File's Extension |  |
| LFS: Lock Selected File |  |
| LFS: Unlock Selected File |  |
| LFS: Force Unlock Selected File |  |
| Refs: Delete Selected Branch or Tag... | `Ctrl+G d` |
| Refs: Delete Selected Branch from Its Remote... |  |
| Refs: Delete Selected Branch Locally and on Its Remote... |  |
| Refs: Copy Link to Selected Branch on Remote |  |
| Git: Copy Link to Selected Commit on Remote |  |
| Git: Annotated Tag at Selected Commit... |  |
| Refs: Push Selected Tag to Default Remote |  |
| Refs: Delete Selected Tag from Its Remote... |  |
| Git: Fetch Selected Tag |  |
| Refs: Annotate Selected Tag... |  |
| Refs: Toggle Hide in Graph |  |
| Refs: Reveal in Graph | `g` |
| Refs: Toggle Solo on Selected Ref | `o` |
| Refs: Stop Soloing (clear every soloed ref) | `O` |
| Refs: Show All Hidden Refs | `H` |
| Refs: Toggle Hide on Selected Folder/Remote |  |
| Refs: Toggle Solo on Selected Folder/Remote | `o` |
| Refs: Filter by Name... | `/` |
| Refs: Clear Name Filter | `Esc` |
| Refs: Expand All Folders | `=` |
| Refs: Collapse All Folders | `-` |
| Refs: Expand the Selected Folder and Its Subfolders | `*` |
| Refs: Collapse the Selected Folder and Its Subfolders | `_` |
| Refs: Next Section | `l` |
| Refs: Previous Section | `h` |
| Refs: Expand or Collapse Active Section | `Enter` |
| Refs: Toggle Branch Hierarchy View | `y` |
| Git: Stage Selected File | `Ctrl+G s` |
| Git: Unstage Selected File | `Ctrl+G u` |
| Git: Stage All Changes | `Ctrl+G a` |
| Working Changes: Discard Selected File... |  |
| Working Changes: Ignore Selected File |  |
| Working Changes: Ignore Selected File's Extension |  |
| Working Changes: Treat Selected File as Binary... |  |
| Working Changes: Treat Selected File as Text... |  |
| Working Changes: Treat Selected File's Extension as Binary... |  |
| Working Changes: Treat Selected File's Extension as Text... |  |
| Working Changes: Stash Selected File |  |
| Working Changes: Selected File History |  |
| Working Changes: Selected File Blame |  |
| Working Changes: Open Selected File in External Diff Tool |  |
| Working Changes: Open Selected File in External Editor |  |
| Working Changes: Open Selected File in Default Program |  |
| Working Changes: Show Selected File in Folder |  |
| Working Changes: Copy Selected File Path |  |
| Working Changes: Create Patch from Selected File... |  |
| Working Changes: Edit Selected File |  |
| Working Changes: Delete Selected File... |  |
| Working Changes: Open Selected Folder Menu |  |
| Working Changes: Create File |  |
| Working Changes: Discard All Changes... | `Ctrl+G x` |
| Git: Commit... | `Ctrl+G c` |
| Git: Commit Staged Changes Now | `Ctrl+G Enter` |
| Git: Edit WIP Commit Message (in the Log) | `c in Log` |
| Git: Fetch All Remotes | `Ctrl+G f` |
| Git: Pull | `Ctrl+G p` |
| Git: Pull (fast-forward if possible) |  |
| Git: Pull (fast-forward only) |  |
| Git: Pull (rebase) |  |
| Git: Pull Options... |  |
| Git: Push | `Ctrl+G P` |
| Git: Push to Remote... |  |
| Git: Force Push (with lease)... |  |
| Git: Set Upstream... |  |
| Remote: Add Remote... |  |
| Remote: Remove Remote... |  |
| Remote: Edit Remote... |  |
| Remote: Set Default Remote... |  |
| Remote: Copy Remote Name |  |
| Remote: Copy Link to Remote |  |
| Remote: Fetch Remote... |  |
| Remote: Actions for Selected Remote... | `.` |
| Repository: SSH Key for This Repository... |  |
| Repository: Sparse Checkout... |  |
| Flow: Open Menu | `Ctrl+G W` |
| Flow: Start Branch... |  |
| Flow: New Issue... |  |
| Flow: Pick Up Host Branch... |  |
| Flow: Finish Current Branch... |  |
| Flow: Sync with Parent... |  |
| Flow: Promote... |  |
| Flow: Set Parent... |  |
| Flow: Continue or Abort Stopped Action... |  |
| Flow: Toggle Grouped Branches |  |
| Flow: Workflow Settings... |  |
| Remote: Hide Remote in Graph... |  |
| Remote: Show Remote in Graph... |  |
| Git: New Branch at HEAD... | `Ctrl+G b` |
| Git: New Tag at HEAD... | `Ctrl+G t` |
| Git: Push All Unpushed Tags | `Ctrl+G U` |
| Git: Stash All Changes | `Ctrl+G T` |
| Git: Stash All Changes with Message... | `Ctrl+G m` |
| Git: Stash Pop | `Ctrl+G o` |
| Git: Stash Apply |  |
| Git: Stash Drop... |  |
| Git: Cherry-Pick Selected Commit onto HEAD | `Ctrl+G y` |
| Git: Check Out Selected Commit (detached HEAD) |  |
| Git: Create Branch at Selected Commit... |  |
| Git: Create Tag at Selected Commit... |  |
| Commit: Copy Message |  |
| Commit: Copy Author Name |  |
| Commit: Copy Author Email |  |
| Commit: Copy Date |  |
| Commit: Copy Short SHA |  |
| Commit: Copy Full SHA |  |
| Git: Revert Selected Commit... | `Ctrl+G v` |
| Git: Edit Commit Message... | `Ctrl+G e` |
| Commit: Amend... | `Ctrl+G E` |
| Commit: Amend Author... | `Ctrl+G A` |
| Commit: Amend Summary... |  |
| Commit: Amend Description... |  |
| Git: Split Selected Commit... |  |
| Git: Interactively Rebase from Selected Commit... |  |
| Rebase: Continue |  |
| Rebase: Skip |  |
| Rebase: Abort |  |
| git am: Continue |  |
| git am: Skip This Patch |  |
| git am: Abort |  |
| Repository: Remove Stale index.lock |  |
| Banner: Pop Pending Auto-Stash |  |
| Banner: Keep the Result of the Pending Auto-stash |  |
| Banner: Show Pending Auto-Stash |  |
| Conflicts: Show Predicted Conflicts |  |
| Banner: Drop Pending Auto-Stash... |  |
| Banner: Start Over... |  |
| Git: Reset Branch to Selected Commit (soft - keep index and working tree) |  |
| Git: Reset Branch to Selected Commit (mixed - keep working tree) | `Ctrl+G R` |
| Git: Reset Branch to Selected Commit (hard - discard everything)... |  |
| Git: Resolve Conflict Using Ours (this branch) |  |
| Git: Resolve Conflict Using Theirs (incoming) |  |
| Git: Resolve Conflict Using Base (common ancestor) |  |
| Conflicts: Open Merge Editor | `Enter on a conflicted file` |
| Merge: Next Conflict | `F7` |
| Merge: Previous Conflict | `Shift+F7` |
| Merge: Next Conflicted File | `Alt+PageDown` |
| Merge: Previous Conflicted File | `Alt+PageUp` |
| Merge: Take Chunk from A |  |
| Merge: Take Chunk from B |  |
| Merge: Take Current for This Conflict | `a` |
| Merge: Take Incoming for This Conflict | `b` |
| Merge: Take Both, A Then B | `c` |
| Merge: Take Both, B Then A | `C` |
| Merge: Take Base for This Conflict |  |
| Merge: Clear This Conflict | `x` |
| Merge: Swap Order | `s` |
| Merge: Take All Current | `A` |
| Merge: Take All Incoming | `B` |
| Merge: Edit This Conflict in Output |  |
| Merge: Show or Hide the Side Panels | `F9` |
| Merge: Scroll the Panes Left | `Shift+Left` |
| Merge: Scroll the Panes Right | `Shift+Right` |
| Merge: Toggle Base Pane | `Alt+D` |
| Merge: Toggle Full File Diff | `Alt+F` |
| Merge: Undo | `Ctrl+Z` |
| Merge: Redo | `Ctrl+Y` |
| Merge: Save without Marking Resolved | `Alt+S` |
| Merge: Save and Mark Resolved | `Ctrl+S` |
| Merge: Reset Output to Merge Result | `Alt+R` |
| Merge: Cycle Output Height | `z` |
| Merge: Widen Current Pane | `Alt+Right` |
| Merge: Widen Incoming Pane | `Alt+Left` |
| Merge: Reset Pane Sizes | `Alt+0` |
| Merge: Unpick This Conflict | `Alt+U` |
| Merge: Focus the Current (A) Pane | `Alt+1` |
| Merge: Focus the Incoming (B) Pane | `Alt+2` |
| Merge: Focus the Output Pane | `Alt+3` |
| Merge: Focus the Base Pane | `Alt+4` |
| Merge: Move to the Pane on the Left (Same Conflict) |  |
| Merge: Move to the Pane on the Right (Same Conflict) |  |
| Merge: Focus Next Pane | `F6` |
| Merge: Focus Previous Pane | `Shift+F6` |
| Merge: Continue from the File on Disk |  |
| Merge: Start Over from the Merge |  |
| Merge: Unresolve and Reopen |  |
| Merge: Reload the File |  |
| Merge: Use Current File as Is | `a` |
| Merge: Use Incoming File as Is | `b` |
| Merge: Use Base File as Is | `o` |
| Merge: Keep the File That Is Left | `k` |
| Merge: Accept the Deletion | `D` |
| Merge: Open in External Merge Tool | `m` |
| ✦ AI: Suggest Merge Conflict Resolution | `Alt+I` |
| Merge: Close Merge Editor |  |
| Git: Abort Merge, Cherry-Pick or Revert |  |
| Git: Skip Cherry-Pick or Revert |  |
| Repository: Open or Switch (Repository Management)... | `Ctrl+O` |
| Repository: Browse for a Repository... | `b (management tab)` |
| Repository: Create New (Init)... | `i (management tab)` |
| Repositories: Open Selected | `Enter` |
| Repositories: Add or Remove Selected from Favorites | `f` |
| Repositories: Close or Remove Selected | `x` |
| Repositories: Selected Repository Menu... | `.` |
| Repositories: Search | `/ / Ctrl+F` |
| Repositories: Collapse All Sections | `-` |
| Repositories: Expand All Sections | `+` |
| Repositories: Close All Tabs... |  |
| Repositories: Remove Selected from List |  |
| Repositories: Remove All Recent |  |
| Workspace: New... | `n` |
| Workspace: New from Open Tabs... | `a` |
| Workspace: Switch... | `Alt+W` |
| Workspace: Edit... | `e` |
| Workspace: Close (No Workspace) |  |
| Workspace: Manage... | `Alt+J` |
| Workspace: Close Tab |  |
| Workspace: Locate Missing Repository... | `l` |
| Workspace: Delete... |  |
| Workspace: Fetch All | `F` |
| Workspace: Pull All | `P` |
| Workspace: Clone Missing | `m` |
| Workspace: Add Tab to Workspace |  |
| Workspace: Remove Tab from Workspace |  |
| Radar: Open | `Alt+R` |
| Radar: Refresh | `r` |
| Radar: Search | `/` |
| Radar: Open in Browser | `o` |
| Radar: Open Repository | `Enter` |
| Radar: Check Out Branch | `c` |
| Radar: Create Worktree... | `t` |
| Radar: Show in Repository | `s` |
| Radar: Start Work on Issue... | `n` |
| Radar: Pin or Unpin | `p` |
| Radar: Snooze... | `z` |
| Radar: Review Request... | `e` |
| Radar: Review Through the API |  |
| Radar: Clone and Review... |  |
| Radar: Edit Assignees... |  |
| Radar: Edit Reviewers... |  |
| Radar: Edit Labels... |  |
| Radar: Set Milestone... |  |
| Radar: Choose Scope | `w` |
| Radar: Choose View | `v` |
| Radar: Choose Filter | `f` |
| Radar: Show Not-Cloned Requests | `a` |
| Radar: Show Drafts | `d` |
| Radar: Turn On Account-Wide Review Requests... | `T` |
| Radar: Row Menu... | `.` |
| Radar: Pick Column | `C` |
| Radar: Widen Column | `]` |
| Radar: Narrow Column | `[` |
| Radar: Reset Column Widths | `0` |
| Radar: Back to Home | `Esc` |
| Radar: Close Tab |  |
| Tab: New Tab | `Alt+T` |
| Tab: New... |  |
| Tab: Actions Menu... | `Ctrl+T` |
| Tab: Close Other Tabs... |  |
| Tab: Close Tabs to the Right... |  |
| Tab: Add to or Remove from Favorites |  |
| Tab: Copy Repository Path |  |
| Tab: Open in File Manager |  |
| Tab: Open in Terminal |  |
| Tab: Open In... | `Alt+O` |
| Tab: Open in Default | `Ctrl+Alt+O` |
| Repository: Change Profile... |  |
| Help: About GitPoulpi | `Ctrl+N a` |
| Help: Licenses and Credits |  |
| View: Activity Log | `Ctrl+N A` |
| View: Global Activity Log | `Ctrl+N G` |
| Help: Release Notes |  |
| Updates: Show Update | `Ctrl+N u` |
| Updates: Copy Update Command |  |
| Updates: Run Update Command in a Terminal |  |
| Updates: Check for Updates Now |  |
| Updates: Download |  |
| Updates: Update Now |  |
| Updates: Restart to Update |  |
| Updates: Revert to the Previous Version |  |
| Updates: Skip This Version |  |
| Release Notes: Close Tab |  |
| Jobs: Show Background Jobs | `Ctrl+N J` |
| Jobs: Cancel All That Can Be Canceled |  |
| View: Copy Diagnostics |  |
| Activity Log: Show or Hide Level Error |  |
| Activity Log: Show or Hide Level Warning |  |
| Activity Log: Show or Hide Level Info |  |
| Activity Log: Show or Hide Level Debug |  |
| Activity Log: Show or Hide Level Trace |  |
| Activity Log: Show All Levels |  |
| Activity Log: Show No Level |  |
| Activity Log: Clear the Application Log |  |
| Activity Log: Choose the Scope |  |
| Activity Log: Git Operations |  |
| Activity Log: Application Log |  |
| Activity Log: Copy the Last Error |  |
| Activity Log: Run the Suggested Action |  |
| Activity Log: Open the Last Error |  |
| Tab: Quit and cd Here |  |
| Repository: Clone... | `Ctrl+Shift+N` |
| Repository: Next Tab | `Ctrl+PageDown` |
| Repository: Previous Tab | `Ctrl+PageUp` |
| Tab: Move Left |  |
| Tab: Move Right |  |
| Repository: Close Tab | `Ctrl+W` |
| Repository: Select Tab... | `t` |
| Search: Workspace... | `Ctrl+Alt+F` |
| Edit: Undo | `Ctrl+Z` |
| Edit: Redo | `Ctrl+Y` |
| Edit: Forget Blocked Undo Entry |  |
| Edit: Undo History... | `Alt+Z` |
| Edit: Reflog... |  |
| Edit: Recover Lost Commits... |  |
| Edit: Restore the Branch to Where the Blocked Action Left It | `Alt+Shift+Z` |
| Review: Show Overview |  |
| Review: Show Files |  |
| Review: Show Commits |  |
| Review: Comment on Line... | `c` |
| Review: Comment on Conversation... | `C` |
| Review: Reply to Thread... | `r` |
| Review: Resolve or Reopen Thread | `x` |
| Review: Fold or Unfold Thread | `z` |
| Review: Edit Pending Comment... | `e` |
| Review: Delete Pending Comment | `d` |
| Review: Mark File as Viewed | `v` |
| Review: Next File | `]` |
| Review: Previous File | `[` |
| Review: Finish Review... | `f` |
| Review: Discard Pending Review... |  |
| Review: Refresh | `F5` |
| Review: Open in Browser | `o` |
| Review: Check Out Branch | `b` |
| Review: Close | `Esc` |
| Review: Edit Assignees... | `a` |
| Review: Edit Reviewers... | `p` |
| Review: Edit Labels... | `l` |
| Review: Set Milestone... | `m` |
| Review: Choose Patch Set... | `P` |
| Review: Compare Patch Sets... | `D` |
| Review: Submit Change... | `U` |
| Review: Abandon Change... | `Q` |
| Review: Toggle Inline / Split Layout | `V` |
| Review: Toggle Whole File / Only the Hunks | `H` |
| Review: Toggle Whitespace-Only Changes | `S` |
| Review: Toggle Word Wrap | `W` |
| Review: Next Hunk | `J` |
| Review: Previous Hunk | `K` |
| Review: Choose Encoding... | `E` |
| Review: Toggle Minimap | `M` |
| Review: Scroll Diff Left | `Left` |
| Review: Scroll Diff Right | `Right` |
| Review: Copy Selected Text or Line | `Ctrl+C` |
| Board: Move Card To... | `m` |
| Board: Move Card Left |  |
| Board: Move Card Right |  |
| Board: Open Card | `Enter` |
| Board: Edit Assignees... |  |
| Board: Edit Labels... |  |
| Board: Set Milestone... |  |
| Board: New Issue in This Column | `n` |
| Board: Filter by Assignee... | `a` |
| Board: Filter by Label... | `l` |
| Board: Filter by Milestone... | `M` |
| Board: Filter by Text | `/` |
| Board: Clear Filters | `x` |
| Board: Set Column Labels... | `c` |
| Board: Refresh | `r` |
| Board: Close | `Esc` |
| Review: ✦ Draft Review... | `s` |
| Review: Accept Suggestion | `y` |
| Review: Edit and Accept Suggestion... | `w` |
| Review: Dismiss Suggestion | `n` |
| Review: Accept All Suggestions | `A` |
| Review: Dismiss All Suggestions | `N` |
| Review: Use AI Summary in Review Text | `i` |
| Review: ✦ Use the Draft... | `u` |
| Review: Edit the ✦ Draft Text... | `t` |
| Review: Discard the ✦ Draft | `N` |
| Undo History: Undo Up to Here... | `u` |
| Undo History: Redo Up to Here... | `y` |
| Undo History: Restore Only This... | `r` |
| Undo History: Restore the Branch to Where It Was... | `o` |
| Undo History: Refresh |  |
| Undo History: Close | `Esc` |
| Reflog: Show in Log | `s` |
| Reflog: Create Branch Here... | `b` |
| Reflog: Restore to This Position... | `r` |
| Reflog: Load More | `m` |
| Reflog: Refresh |  |
| Lost Work: Show in Log | `s` |
| Lost Work: Create Branch Here... | `b` |
| Lost Work: Cherry-pick | `c` |
| Lost Work: Scan Again |  |
| View: Refresh | `r` |
| View: Refresh (fetch, pull requests, issues) | `F5` |
| View: Toggle Help Overlay | `?` |
| View: Toggle Ref Column |  |
| View: Toggle Graph Column |  |
| View: Toggle Message Column |  |
| View: Toggle Author Column |  |
| View: Toggle Date Column |  |
| View: Toggle Changes Column |  |
| View: Toggle SHA Column |  |
| View: Log Column: Pick a Column to Move or Size | `C` |
| View: Log Column: Pick the Next Column | `Tab` |
| View: Log Column: Pick the Previous Column | `Shift+BackTab` |
| View: Log Column: Move the Column Left | `<` |
| View: Log Column: Move the Column Right | `>` |
| View: Log Column: Widen the Column | `]` |
| View: Log Column: Narrow the Column | `[` |
| View: Log Column: Reset the Column Widths | `0` |
| View: Log Column: Fit the Column to Its Content | `=` |
| View: Log Column: Fit All Columns to Content | `+` |
| View: Log: Scroll the Columns Left | `Shift+Left` |
| View: Log: Scroll the Columns Right | `Shift+Right` |
| View: Log: Toggle Scroll Sideways for the Columns | `s` |
| View: Log: Choose the Columns to Show |  |
| Diff: Open Diff for Selected File | `Enter` |
| Diff: Close Diff View | `Esc` |
| Diff: Next Hunk | `n` |
| Diff: Previous Hunk | `p` |
| Diff: Stage Hunk Under Cursor | `Space` |
| Diff: Unstage Hunk Under Cursor | `Space` |
| Diff: Select Lines (or clear the selection) | `v` |
| Diff: Stage Selected Lines | `Enter` |
| Diff: Unstage Selected Lines | `Enter` |
| Diff: Discard Hunk Under Cursor... | `x` |
| Diff: Discard Selected Lines... | `X` |
| Copy: Selected Text (drag over the text first) | `Ctrl+C` |
| Copy: Line Under the Cursor (or the selected lines) | `y` |
| Diff: Copy Hunk Under the Cursor |  |
| Diff: Layout Inline (+/- in one column) | `m` |
| Diff: Layout Split (before and after) | `m` |
| Diff: Toggle Hunks Only (whole file / only the changes) | `H` |
| Files: Toggle Path/Tree | `m` |
| Files: Toggle View All Files | `a` |
| Commit: Toggle Skip Hooks | `H` |
| Files: Open Menu of the Row or Selection | `.` |
| Files: Expand All Folders | `=` |
| Files: Collapse All Folders | `-` |
| Files: Expand the Selected Folder and Its Subfolders | `*` |
| Files: Collapse the Selected Folder and Its Subfolders | `_` |
| Files: Select All |  |
| Files: Clear Selection |  |
| Files: Toggle Selection of the Current Row |  |
| Log: Copy as Mermaid gitGraph... | `Y` |
| Log: Compare Selection | `v` |
| Log: Swap Comparison Ends | `x` |
| Log: Go to Next Compared Row | `n` |
| Diff: Toggle Word Wrap | `w` |
| Diff: Choose Encoding... | `C` |
| View: Toggle Minimap | `M` |
| View: Toggle Commit Column | `Alt+C` |
| View: Commit Column: Hide |  |
| View: Commit Column: File History |  |
| View: Commit Column: All Commits |  |
| View: Focus the Commit Column |  |
| View: Widen the Commit Column |  |
| View: Narrow the Commit Column |  |
| View: Commit Column: Scroll Left | `Shift+Left` |
| View: Commit Column: Scroll Right | `Shift+Right` |
| View: Commit Column: Size the Next Part | `Tab` |
| View: Commit Column: Move the Part Left | `<` |
| View: Commit Column: Move the Part Right | `>` |
| View: Commit Column: Widen the Part | `]` |
| View: Commit Column: Narrow the Part | `[` |
| View: Commit Column: Reset the Part Widths | `0` |
| View: Commit Column: Menu of the Selected Row | `.` |
| View: Commit Column: Choose the Columns |  |
| View: Commit Column: Toggle Ref |  |
| View: Commit Column: Toggle Graph |  |
| View: Commit Column: Toggle Author |  |
| View: Commit Column: Toggle Date |  |
| View: Commit Column: Toggle Changes |  |
| View: Commit Column: Toggle SHA |  |
| View: Commit Column: Show the Selected Row in the Log |  |
| View: Commit Column: Show the Changes of the File at the Selected Row |  |
| View: Commit Column: Show the File as It Was at the Selected Row |  |
| View: Commit Column: Blame the File at the Selected Row |  |
| Editor: Toggle Follow Selection |  |
| Image: Zoom In | `+` |
| Image: Zoom Out | `-` |
| Image: Fit to View | `0` |
| Image: Actual Size (1:1) | `1` |
| Image: Flip Before / After | `Tab` |
| Image: Widen Before Pane | `Alt+Right` |
| Image: Widen After Pane | `Alt+Left` |
| Image: Reset Pane Sizes | `Alt+0` |
| Image: Edit In... | `O` |
| Image: Edit in Default Editor | `o` |
| File History: Show in the Log | `Enter` |
| File History: Copy the commit SHA | `y` |
| File History: Copy the Summary | `Y` |
| File History: Show the diff of This commit | `d` |
| File History: Open the File at This commit | `o` |
| File History: Blame at This commit | `b` |
| File History: Compare with the Working Tree | `c` |
| File History: Restore This Version | `R` |
| File History: Extend the Selection Up | `Shift+Up` |
| File History: Extend the Selection Down | `Shift+Down` |
| File History: Switch Between the List and the Diff | `Tab` |
| File History: Widen the List | `Alt+Right` |
| File History: Narrow the List | `Alt+Left` |
| File History: Reset the List Width | `Alt+0` |
| File: Open With... | `O` |
| File: Open in Default App | `o` |
| Diff: Toggle Whitespace-Only Changes | `s` |
| ✦ AI: Generate Commit Message | `Ctrl+G i` |
| ✦ AI: Generate Stash Message |  |
| ✦ AI: Insert Prompt Variable |  |
| ✦ AI: Edit Profile... |  |
| ✦ AI: Open the Agent... |  |
| ✦ AI: Explain Selected Commit | `Ctrl+G w` |
| ✦ AI: Split Selected Commit... |  |
| ✦ AI: Explain Working Changes | `Ctrl+G w` |
| ✦ AI: Compose Commits from Working Changes... | `Ctrl+G C` |
| Composer: Move File to Commit... | `m` |
| Diff: Blame |  |
| Blame: Show Commit in Log | `Enter` |
| Blame: Copy Commit SHA | `Y` |
| Blame: Show the Commit's Diff for This File | `d` |
| Blame: Blame at the Commit's Parent | `b` |
| Blame: Open File History | `H` |
| Blame: Show or Hide the Detail Card | `i` |
| Blame: Next Hunk | `n` |
| Blame: Previous Hunk | `p` |
| Blame: Next Hunk of the Same Commit | `N` |
| Blame: Previous Hunk of the Same Commit | `P` |
| Blame: Go to Line | `g` |
| Blame: Widen the Annotation Column | `>` |
| Blame: Narrow the Annotation Column | `<` |
| Diff: File History |  |
| Diff: File View |  |
| File: Edit in Working Directory | `e` |
| Editor: Change Encoding |  |
| Editor: Reopen with Encoding |  |
| Editor: Convert to Encoding |  |
| Editor: Convert Line Endings |  |
| Editor: Change Indentation |  |
| App: Settings... | `Ctrl+, / Ctrl+N ,` |
| App: Search Settings... | `/ in Settings` |
| App: Keybindings... |  |
| Agent: Show Pending View | `Ctrl+N Enter` |
| Agent: Dismiss Pending View |  |
| Agent: Back | `Ctrl+N Backspace` |
| Keybindings: Reset All... |  |
| Integrations: Open Linked Pull Request | `Ctrl+G g` |
| Integrations: Edit Linked Pull Request on the Host | `e (on a row)` |
| Integrations: Copy Linked Pull Request URL |  |
| Integrations: Refresh Pull Requests and Issues |  |
| Integrations: Check Out Pull Request... |  |
| Integrations: Open Commit Reference (#issue / !request)... |  |
| Integrations: Open Issue... |  |
| Hosting: Review Request... |  |
| Hosting: Push for Review... |  |
| Hosting: Edit Assignees... |  |
| Hosting: Edit Reviewers... |  |
| Hosting: Edit Labels... |  |
| Hosting: Set Milestone... |  |
| Local Issues: Close Issue... |  |
| Local Issues: Reopen Issue... |  |
| Local Issues: Open in Obsidian... |  |
| Issues: Open Board |  |
| Local Issues: Create Obsidian Base |  |
| Pull Requests: Create Pull Request... | `n` |
| Hosting: New Issue... | `n` |
| Pull Requests: Search | `Enter on the search row` |
| Pull Requests: Choose Filter (Open / Merged / All)... | `v` |
| Pull Requests: Choose Provider... | `Enter on the provider row` |
| Pull Requests: Choose Repository... | `Enter on the repository row` |
| Pull Requests: Unlink Source (None) |  |
| Issues: Choose Provider... | `Enter on the provider row` |
| Issues: Unlink Source (None) |  |
| Integrations: Unlink Profile (Automatic) |  |
| Issues: Choose Repository... | `Enter on the repository row` |
| Issues: Search | `Enter on the search row` |
| Issues: Choose Filter... | `v` |
| Issues: Edit Query / Lists... |  |
| Commit: Insert Issue Reference... |  |
| Integrations: Use Profile for This Repository... |  |
| Integrations: Manage This Repository's Profile |  |
| Integrations: Connect Integration... |  |
| Settings: Radar Options of a Profile... |  |
| App: Quit | `q` |
| App: Theme... |  |
| App: Change Theme... | `Ctrl+N y` |
| App: Change Language... | `Ctrl+N g` |
| Launcher: Create Shortcut |  |
| Launcher: Create Desktop Shortcut |  |
| Launcher: Remove Shortcuts |  |
| Launcher: Create Command Alias |  |
| Launcher: Remove Command Alias |  |
| Launcher: Add to PATH |  |
| Launcher: Remove from PATH |  |
| App: Change Graph Color Theme... | `Ctrl+N Y` |
| App: Toggle Graph Row Tints | `Ctrl+N T` |
<!-- palette:end -->

## Moving around

| Key | Action |
|---|---|
| `Tab` / `Shift+Tab` | Focus the next / previous pane |
| `Ctrl+N`, then a badge | Focus that part of the window (see [Navigation prefix](#navigation-prefix-ctrln)) |
| `j` / `k`, `Up` / `Down` | Move the selection, in every pane |
| `PageUp` / `PageDown`, `Home` / `End` | Move one page; go to the first / last row |
| a number, then `j` / `k` / `Up` / `Down` | Count prefix: `4j` moves four rows. With `PageUp` / `PageDown`, the count is a number of pages. The status line shows the pending count (`4…`). `Esc` or any other key clears it. A leading `0` never starts a count |
| `h` / `l`, `Left` / `Right` | Refs pane: switch section. Log: leave (`h`) or enter (`l`) the changed-files list of a commit |
| `?` | Open or close the help (see [In-app help](#in-app-help-)) |
| `r` | Refresh |
| `F5` | Fetch every remote, reload the pull requests and issues, and refresh |
| `q` | Quit |
| `Ctrl+Z` | **Undo** the last operation. See [Undo and redo](undo-redo.md) |
| `Ctrl+Y`, `Ctrl+Shift+Z` | **Redo**. `Ctrl+Shift+Z` needs the kitty keyboard protocol; `Ctrl+Y` always works |
| `.`, `Menu`, `Shift+F10` | Open the context menu of the selected row |

`Ctrl+Z` and `Ctrl+Y` do nothing while a text field has the keyboard. The
editor has its own text undo.

## Filterable popup lists

All filterable popup lists use the same keys: the command palette and its
pickers (theme, language, graph colors), the tab picker, and every dropdown
with a search line (Settings choices, the pull request form, the issue
reference picker, the **Open in** list). Type to filter the list. These keys
move the highlight:

| Key | Action |
|---|---|
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J` | Move one row |
| `Home` / `End` | First / last row |
| `PageUp` / `PageDown` | Move one visible page |
| `Enter` | Choose the highlighted row |
| `Esc` | Close |

In these popups, `Home`, `End` and `Ctrl+K` move the list, not the caret of the
filter field. `Ctrl+A` and `Ctrl+E` still move the caret to the start and end.
Plain `j` and `k` are typed into the filter.

With the mouse:

- The wheel **scrolls** the list. It does not move the highlight.
- A click **selects** a row. A double-click **chooses** it, like `Enter`.
- A list taller than its popup shows a scrollbar on its right edge. You can
  drag it.
- A short dropdown without a search line (two or three choices) chooses on the
  first click.

**Live preview.** The theme, graph color theme and language pickers apply the
highlighted row at once, behind the popup. `Enter` keeps it. `Esc`, or a click
outside the popup, restores the previous value.

**Theme filter.** The theme picker has a row under the filter field: **All**,
**Dark**, **Light** and **Colored**. `Tab` / `Shift+Tab` go to the next /
previous one. `Alt+1` to `Alt+4` choose one directly. You can also click them.

## In-app help (`?`)

`?`, `Ctrl+N ?` or the palette's `View: Toggle Help Overlay` opens the help.
It lists the keys and mouse gestures of every part of the app. It opens over
anything, the Settings screen included.

| Key | Action |
|---|---|
| `?` | Close the help (and clear the filter) |
| *(type)* | Filter: keep only the sections that contain the text. `j`, `k` and `q` are typed too |
| `Backspace`, `Ctrl+U` | Delete the last character / clear the filter |
| `Esc` | Clear the filter; if it is empty, close the help |
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J` | Scroll one line |
| `PageUp` / `PageDown`, `Home` / `End` | Scroll one page; go to the top / bottom |
| `Ctrl+D`, or a click on the link | Open this documentation on GitHub in the browser |

The wheel scrolls the help, and you can drag its scrollbar. A click outside
the help closes it.

## Navigation prefix (`Ctrl+N`)

`Ctrl+N` waits for one more key. The status line shows the keys it accepts. The
next key runs an action, or cancels.

| Key | Action |
|---|---|
| `Ctrl+N` then a badge | Focus the part of the window that shows this badge |
| `Ctrl+N h` / `Ctrl+N l` | Collapse or restore the Refs pane / the right pane |
| `Ctrl+N Left` / `Ctrl+N Right` | Move the divider between the Refs pane and the Log one notch left / right (a collapsed Refs pane comes back first) |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | Move the divider between the Log and the right pane one notch left / right |
| `Ctrl+N +` / `Ctrl+N -` | Grow / shrink the active Refs section by one row |
| `Ctrl+N ]` / `Ctrl+N [` | Grow / shrink the commit message block of the right pane by one row |
| `Ctrl+N t` | Choose a tab (the tab picker; `t` alone does the same) |
| `Ctrl+N y` / `Ctrl+N g` / `Ctrl+N Shift+Y` | Change the theme / the language / the graph color theme |
| `Ctrl+N Shift+T` | Turn the graph row tints on or off |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | Show the view an agent prepared / go back (see [MCP and agents](mcp-and-agents.md)) |
| `Ctrl+N ,` | Open Settings |
| `Ctrl+N ?` | Open or close the help |
| `Esc`, or any other key | Cancel |

### Badges

While the prefix waits, each part of the window that can take the focus shows
a one-character badge on its top-left border. Press that character to focus
it. Badges are given out again each time, in screen order: `1` to `9` first,
then `a`, `b`, `c`... So read the badge on the screen: a part does not always
get the same badge.

Parts with a badge: the Refs filter and each visible Refs section, the Log (or
the Diff view), the File View when it is open, the Unstaged and Staged lists,
the commit composer's summary field, and the controls of the commit file list.
On the Home tab, the badges are the search field, the four sections and the toolbar buttons (a button's badge presses it). On the Radar tab,
they are the filter field, the Scope, View and Show comboboxes (a combobox's badge
opens its list), the Refresh and Back buttons, and each section heading on screen.
A section's badge moves the selection to its heading; `Enter` or `Space` then folds
it.

In the Browse folder picker and the Clone, Init, Create worktree, Workspace, Sparse checkout, Flow, New issue and pull request dialogs, the badges are the fields, checkboxes and buttons (a button's badge presses it, a field's puts the cursor in it; in a list, one badge is on its first row; the Browse folder picker has one badge for the Quick access list and one for the Folders list, beside each list's first row, and pressing one puts the keyboard in that list without choosing a row). In the Activity Log, they are the two tabs, the Scope and Level boxes, the filter, the level checkboxes, the buttons and the first entry of the list. A badge does what a click on that control does. In the folder picker, `Ctrl+N` starts the badges, so its New folder key is `Alt+N`.

In the Amend (and Edit commit message) dialog, the badges are the summary, the description, the author's name, email and date, **Modify History Deeper...**, **Cancel**, **Apply** and the close cross; in the history rewrite workbench they are the scope box, its checkboxes and refs button, the recipe name, each rule, its checkbox and buttons, the **+ Add rule** button, the fields of the selected rule, the preview tabs, the filter box, the protected-branch rows and their Check all / Uncheck all, and the Preview, Apply, Cancel, Save, Load and Create Rules with AI buttons (the Secrets, Identities, Size and result pages add their own buttons and rows; the Commits tab's rows, being a list, have none). A badge does what a click does. Nothing is badged while a dropdown, a question, a field being typed into, the folder picker, the purge screen, the script editor, the AI dialogs, the blob diff or the host how-to is open, nor while the Amend dialog asks "discard?"; `Esc` takes the badges away without closing anything.

In Settings, the badges are the search box, the page list (one badge, on the current page), the sub-tabs, the list and its buttons, each field of the page that is on screen and the close cross. A badge does what a click on that control does: a text field opens for editing, a checkbox is ticked, a dropdown opens, a button runs. `Ctrl+N` keeps its own meaning while a field is being typed into or a dropdown, an editor or the search is open.

The palette has the same actions, for example
`Navigation: Focus Refs > Local` or `Navigation: Focus Status > Staged`.

## Repositories and tabs

| Key | Action |
|---|---|
| `Ctrl+O`, or a click on `⌂` | Go to the Home tab (on the Home tab: focus the search field; with the Radar or the release notes open: close it first) |
| `Ctrl+Shift+N` | Open the Home tab with the Clone dialog |
| `Ctrl+PageDown` / `Ctrl+PageUp` | Next / previous tab |
| `t` | Choose a tab from a filterable list |
| `Ctrl+W` | Close the current tab (asks first; the repository is not changed) |
| `Ctrl+T`, or a right-click on a tab | The tab menu (its rows are listed in [Tab row](interface-tour.md#tab-row)) |
| `Alt+O` / `Ctrl+Alt+O` | Open the tab's folder: choose from the **Open in** list / use the default entry. See [Open in](open-in.md) |

### The Home tab

The `⌂` tab manages your repositories. It has four sections: **Open
repositories**, **Favorites**, **Recent repositories** and **All
repositories**. See [Tour of the interface](interface-tour.md#home).

| Key | Action |
|---|---|
| `j` / `k`, arrows, `PageUp` / `PageDown`, `Home` / `End` | Move |
| `Enter`, or a double-click | Open the repository (or switch to its tab). On a section header: fold or unfold it |
| `f` or `*`, or a click on the star | Add to / remove from favorites |
| `x`, `Delete`, or the row's button | Close its tab (Open), remove it from the list (Recent), remove the star (Favorites), forget it (All). Nothing on disk is deleted |
| `.`, a right-click, or the row's `⋮` | The row's menu |
| `/` or `Ctrl+F` | Search all sections by name, branch or path. `Enter` opens the first match; `Esc` clears the search, then leaves the field |
| `Space`, `Left` / `Right` | Fold / unfold the current section |
| `-` / `+` | Collapse all / Expand all |
| `b` / `c` / `i` | Browse (open a folder) / Clone / Init |
| `Esc` | Clear the search, or go back to the active repository tab |
| `q` | Quit |

`Ctrl+P`, `Ctrl+,`, `?` and `Ctrl+N` work on the Home tab. `Ctrl+G`, `Ctrl+W`
and `Ctrl+T` need a repository tab: on the Home tab they only show a message.
Palette commands that need a repository are grayed out.

### The folder picker

**Browse** (and the **Browse...** button of the Init, Clone and Create
worktree dialogs) opens the folder picker. It has a path field at the top, the
places and the folder tree on the left, and the content of the current folder
on the right.

| Key | Action |
|---|---|
| `Tab` / `Shift+Tab` | Path field, then the left column, then the content |
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Move in the focused column. In the path field: move through the completions |
| a letter (content) | Go to the next folder that starts with this letter |
| `Enter` / `Right` | Open the selected folder. In the path field: go to the typed folder; on the folder already shown, choose it |
| `Backspace` / `Left` / `Alt+Up` | Parent folder (in the path field, `Backspace` deletes a character) |
| `Tab` (path field) | Accept the only completion, or the highlighted one |
| `Alt+N` | New folder (`Ctrl+N` starts the badges, see [Badges](#badges)) |
| `Alt+H` | Show or hide hidden folders |
| `Ctrl+F`, or `/` outside the path field | Filter the folder's entries as you type (case-insensitive; `..` stays; the header counts "x of n"; `Enter` jumps to the first match, `Esc` clears; it empties when you change folder). Also reached with `Tab` or a click |
| `Ctrl+S`, `Ctrl+Enter`, or the confirm button | Choose the selected subfolder, or else the folder shown |
| `Esc` | Close the completions, or cancel |

When the path field shows a new path, the whole path is selected: the first
key you type replaces it.

### Init and Clone dialogs

| Key | Action |
|---|---|
| `Tab` / `Shift+Tab`, `Up` / `Down` | Next / previous field or button |
| `Enter` | Press the focused button, or create / clone |
| `Space` | Toggle the focused checkbox, or press the focused button |
| `Ctrl+B` | Browse for the folder |
| `j` / `k`, `g` / `G` | In the Clone dialog's sources list and repository list: down / up, first / last (not in the search field) |
| `Esc` | Close. During a clone, the clone goes on in the background |

In the Clone dialog, the left column lists **Clone with URL** and one source
per connected hosting profile (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit).
Choose a source with `Up` / `Down` or a click. Then type in the **Repository**
search field, move in the list with the arrows, `PageUp` / `PageDown` or the
wheel, and choose a row with `Enter` or a click. See
[Getting started](getting-started.md) for the fields.

## Refs pane

| Key | Action |
|---|---|
| `Enter` / `Space` | Expand or collapse the active section, or the selected folder |
| `h` / `l`, `Left` / `Right` | Switch section |
| `/` | Filter by name (`Enter` applies, `Esc` cancels) |
| `y` | Branch hierarchy view on or off: folders from the `/` in branch and tag names (LOCAL, REMOTE and TAGS) |
| `o` | **Solo** the selected ref, or every ref under the selected folder or remote. Solo several refs to see their union |
| `Shift+O` | **Stop Soloing**: clear every solo |
| `Shift+H` | **Show All**: show every hidden ref again |
| `g` | **Reveal in graph**: scroll the Log to the selected ref's (or stash's) commit and flash its row; the selection does not move |
| `m` | **Mark** the selected ref as a drop source (`Esc`, or `m` on it again, clears the mark) |
| `Shift+M` | **Drop the marked ref** on the selected ref: opens the menu a drag-and-drop opens (Merge, Rebase, Push, Fast-forward...) |
| `+` (REMOTE section) | Add a remote |
| `.`, `Menu`, `Shift+F10`, or a right-click | The row's menu: a branch, a tag, a remote's name, or a folder |
| `Enter` on a WORKTREES row | Open that worktree in its own tab |
| `Ctrl+G d` | Delete the selected branch or tag (asks first). On a REMOTE row: delete that remote branch |

For what the menus do, see [Working with the Log](log-and-history.md).

### Pull requests and issues

With a matching integration, the Refs pane has a pull requests section (GitLab:
merge requests). With a connected hosting profile, it also has an ISSUES
section. See [Hosting and issue trackers](hosting-integration.md#5-pull-requests-and-issues-in-the-sidebar).

| Key | Action |
|---|---|
| `Enter` | Open the selected row in the browser. On a `Reconnect` row: open Settings on that profile |
| `Space` | Expand or collapse the section |
| `v` | Open the section's filter. Pull requests: Open, Merged / closed, All. Issues: Open, Assigned to me, Closed, All |
| `n` | Pull requests section: create a pull request. ISSUES section: new issue. (The header's `+` button) |
| `e` | Pull requests section: edit the selected request on the host (the row's pencil) |
| `Enter` on the search row | Type in the search field. `Enter`, `Tab` or `Esc` leaves it; `Ctrl+U` clears it |
| `Enter` on a group header | Fold or unfold the group (*My Pull Requests*, *Assigned To Me*, *Awaiting My Review*, *All Pull Requests*) |
| `Enter` on the filter, provider or repository row | Open that dropdown. The provider list ends with **None (unlink this repository)** |
| `.`, or a right-click | The row's menu: open in browser, copy URL or number, check out the source branch (pull requests); create a branch from the issue (issues) |

### The issue board

**Board** on the ISSUES header (palette: **Issues: Open Board**) opens the Kanban board of the issue source. See
[the board](hosting-integration.md#5b-the-issue-board) and [local issues](local-issues.md#the-kanban-board). Every key
below is also a palette command (**Board: ...**).

| Key | Action |
|---|---|
| Arrows | Select a card (`Up`, `Down`) or the neighbouring column (`Left`, `Right`); `Home`, `End`, `PageUp`, `PageDown` jump in the column |
| `Shift+Left`, `Shift+Right` | Move the selected card to the previous or next column |
| `m` | **Move Card To...**: a dropdown of the columns |
| `Enter` | Open the issue (the note, for a local issue; the host's page otherwise) |
| `e`, `.` | Edit assignees, labels or milestone; the card menu |
| `n` | New issue in the selected column |
| `a`, `l`, `M`, `/` | Filter by assignee, label, milestone, text. `x` clears the filters |
| `c` | The labels that are the columns of a GitHub / GitLab board |
| `r` | Read the issues again |
| `Esc` | Close the board |

Mouse: drag a card onto another column to move it, click a card to select it, double-click to open it, right-click for
the card menu, wheel to scroll a column, `Shift`+wheel (or the bar under the board) to scroll the columns.

### Flow (branching workflows)

With a workflow set up for the repository (see [branching workflows](branching-workflows.md)) the toolbar has a
**Flow** button, and the sidebar groups LOCAL and REMOTE by branch type. Every row of the menu is a palette command
(**Flow: ...**; **Flow: Start Feature...** and **Flow: New Feature Issue...** are generated for each topic type of the
workflow). The host-driven rows (*New <Type> issue*, *Pick up a host-made branch*) and their dialogs, the waiting card and the
"a branch appeared" card are described in [branching workflows](branching-workflows.md#host-driven-flows-in-the-app);
`Enter` runs the focused button (*Create issue*, *Pick up*), `Esc` stops waiting or closes.

| Key | Action |
|---|---|
| `Ctrl+G Shift+W` | Open the Flow menu (the toolbar button) |
| `Enter`, `Tab`, `Shift+Tab` | In the Start / Finish / Sync / Promote / Set parent dialog: preview or run, next field, previous field |
| `Space` | Check a checkbox (*Publish*, *Run the hooks*, *Also update the branches stacked on it*) |
| `Esc` | Close a dropdown, go back from a preview to the form, close the dialog (not while a run is going) |
| Hover a branch | Its type, parent, and how far behind the parent it is |

## Log pane

| Key | Action |
|---|---|
| `Enter` / `l` | Enter the selected commit's changed-files list |
| `Enter` / `l` on a file row | Open the file's diff in the center |
| `Esc` / `h` | Go back to the commits |
| `Right` | (commit with more than one ref) Open the list of its other refs: `Up` / `Down` move, `Enter` checks out, `Esc` closes |
| `f` | Open the search bar (see [Search](#search)) |
| `c` | (with uncommitted changes) Select the WIP row and put the caret in its message field |
| `Enter` on the WIP row | Put the caret in the WIP row's message field |
| `.`, `Menu`, `Shift+F10` | The commit menu. `Up` / `Down` move, `Right` / `l` / `Enter` open a submenu, `Left` / `h` / `Esc` go back |
| `m` / `a` | Files of the right pane: `Path` or `Tree` / `View all files` |
| `Shift+C` | Pick a Log column to move or size (see below) |

With a column picked, `Tab` / `Shift+Tab` pick another, `<` / `>` move it, `]` / `[` widen or narrow it, `0` puts every width back to automatic and `Esc` or `Enter` leaves (context `log_columns` in the generated table above).

The WIP row's message field is the commit composer's summary. While it has
the caret, every key is typed, but `Ctrl` chords still work. `Enter` moves to
the composer's description. `Esc` leaves the field and keeps the text. See
[Staging and committing](staging-and-committing.md).

### Selecting several commits

| Key or gesture | Action |
|---|---|
| `Space` | Add the current commit to the selection, or remove it |
| `Shift+Up` / `Shift+Down`, `Alt+Up` / `Alt+Down` | Extend the selection (Alt for terminals that keep Shift for themselves) |
| `Ctrl+click` / `Shift+click` or `Alt+click` | Toggle a commit / select a range |
| `Esc` | Clear the selection |
| `Shift+Y` | [Copy the selection as a Mermaid gitGraph](log-and-history.md#copying-commits-as-a-mermaid-gitgraph) (a popup with detail levels and a live preview; `Enter` copies) |

With several commits selected, right-click one of them and choose **Cherry
pick N commits onto `<branch>`...**. It opens the same view as an interactive
rebase.

### Naming a branch or tag

`Ctrl+G b`, `Ctrl+G t`, the toolbar's **Branch** and **Tag** buttons and the
menus' **Create branch here...** / **Create tag here...** ask for the name in
a text box on the commit's row. `Enter` creates the ref. `Esc` cancels. An
invalid or existing name is shown in red, and the box stays open.

### Editing a commit message

**Edit commit message...** in the commit menu, `Ctrl+G e`, or the palette's
`Git: Edit Commit Message...` opens the **Reword commit message** popup. See
[Working with the Log](log-and-history.md) for what it changes.

| Key | Action |
|---|---|
| `Enter` in the summary | Go to the description |
| `Enter` in the description | New line. `Up` on the first line goes back to the summary |
| `Ctrl+Enter`, `Ctrl+S`, or **Update Message** | Save the new message |
| `Tab` / `Shift+Tab` | Move between the summary, the description and the buttons |
| `Esc`, or **Cancel** | Close without a change |

### Amending a commit

`Ctrl+G Shift+E` (all fields) and `Ctrl+G Shift+A` (the author) open the Amend dialog, and so do the commit menu, the palette's `Commit: Amend...` rows and the commit details pane; every way in is described in [Amending a commit](log-and-history.md#amending-a-commit-message-author-dates). Both chords are rebindable: `git_prefix.amend` and `git_prefix.edit_author`. Keys inside the dialog:

| Key | Action |
|---|---|
| `Tab` / `Shift+Tab` | Move between the fields and the buttons |
| `Enter` in a one-line field | Next field; after the last one, **Apply** |
| `Up` / `Down` | Previous / next field (in the description, from its first / last line) |
| `Ctrl+Enter`, `Ctrl+S`, or **Apply** | Apply the changed fields |
| `Esc`, or **Cancel** | Close without a change |
| `Alt+H`, or **Modify History Deeper...** | Close the dialog and open the history rewrite workbench from this commit (builds with history rewrite only); asks first if you edited a field (`Esc` confirms). Rebindable: `reword.deeper` |

### Create worktree

**Create worktree from `<ref>`...** (branch and tag menus) and **Create
worktree here...** (commit menu) open a dialog: the folder, an optional new
branch, and **Open the new worktree as a tab**. Keys: `Tab` / `Shift+Tab`
move, `Enter` creates, `Space` toggles the checkbox, `Ctrl+B` browses,
`Esc` cancels.

### Interactive rebase

**Interactively rebase from here...** in the commit menu, or
`Git: Interactively Rebase from Selected Commit...`, opens the rebase view. A
cherry-pick of several commits uses the same view. See
[Working with the Log](log-and-history.md).

| Key | Action |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | Set the row's action: Pick / Reword / Squash / Fixup / Drop / Edit (Fixup and Edit only in a rebase) |
| `Space` | Open the row's action list (`Up` / `Down` choose, `Enter` applies, `Esc` closes the list) |
| `Up` / `Down` | Move the selection |
| `Ctrl+Up` / `Ctrl+Down`, `Alt+Up` / `Alt+Down` | Move the selected row up / down |
| `a` | Autosquash preview on or off |
| `Enter` on a Reword row | Edit that commit's new message |
| `Ctrl+S`, `Ctrl+Enter`, or **Start Rebase** / **Start Cherry Pick** | Run the plan |
| `Ctrl+R`, or **Reset** | Restore the plan as it was when the view opened |
| `Esc`, or **Cancel** | Close without a change |

When a rebase stops (on a conflict or an Edit step), a banner above the Log
offers **Continue**, **Skip** and **Abort**. The palette has
`Rebase: Continue`, `Rebase: Skip` and `Rebase: Abort`.

## Search

`f`, the toolbar's **Search** button, or `Navigation: Search Commits...` opens
the search bar over the Log. It searches the full commit message, the author
name and email, the SHA and the ref names. It does not hide rows: rows that do
not match are dimmed.

| Key | Action |
|---|---|
| *(type)* | Edit the search text (every key is typed) |
| `Enter` / `Down`, or the bar's `↑` | Next match |
| `Shift+Enter` / `Up`, or the bar's `↓` | Previous match |
| `Esc`, or the bar's `✕` | Close the search bar |

## Diff view

A changed file opens in the center, in place of the Log. See
[Staging and committing](staging-and-committing.md).

| Key | Action |
|---|---|
| `Enter` / `l`, or a double-click (on a file row) | Open the file's diff |
| `Esc`, `q`, or the `✕` in the title | Close the diff |
| `j` / `k`, `Up` / `Down` | Scroll |
| `PageUp` / `PageDown`, `Home` / `End` | Scroll one page; go to the start / end |
| `h` / `l`, `Left` / `Right` | Scroll sideways (not with word wrap) |
| `n` / `p` | Next / previous hunk |
| `m` | Inline or split layout |
| `Shift+H` | Hunks only or whole file |
| `w` | Word wrap on or off |
| `s` | Show or hide whitespace-only changes |
| `Shift+M` | Minimap on or off |
| `Shift+C` | Choose the encoding the file is read in (a list: Auto-detect first) |
| `e` | Edit the file in the built-in editor (see [Editor](#editor)) |
| `r` | (Git LFS file) The LFS card or the raw pointer. See [Git LFS](lfs.md) |
| `Shift+E` | (conflicted file) Open the merge editor. See [Resolving merge conflicts](merge-conflicts.md) |
| `y` / `Ctrl+C` | Copy the selected text. Without a selection: the line under the cursor, or the selected lines |
| `.`, or a right-click on a hunk | The hunk menu |

### Staging and discarding from the diff

These keys work on a diff of the working changes only (opened from the
Unstaged or Staged list). Discarding works on the Unstaged list only.

| Key | Action |
|---|---|
| `Space` | Stage (Unstaged) / unstage (Staged) the hunk under the cursor |
| `v` | Start or drop a line selection; `j` / `k` extend it |
| `Enter` | Stage / unstage the selected lines, or the line under the cursor |
| `x` | Discard the hunk under the cursor (asks first: `y` / `Enter` confirm, `n` / `Esc` cancel) |
| `Shift+X` | Discard the selected lines, or the line under the cursor (asks first) |

Only changed lines count: context lines in a selection are ignored. A
selection that reaches into a second hunk acts on the first hunk only.

## Image view

A picture in the Diff view or the File View. The keys are in the context
`image` of the table above. See [Pictures](image-view.md).

## File View, Blame and File History

| Key | Action |
|---|---|
| `Esc` / `q` | Close (with a text selection, `Esc` clears it first) |
| `j` / `k`, `Up` / `Down` | Move the selection |
| `PageUp` / `PageDown` | Scroll one page |
| `Home` / `End`, `Shift+G` | First / last line |
| `h` / `l`, `Left` / `Right` | Scroll the text sideways |
| `Enter` | (Blame, File History) Go to the row's commit in the Log |
| `e` | Edit the file in the built-in editor |
| `y` / `Ctrl+C` | Copy the selected text. Without a selection: the selected row (Blame: the code only; File History: the summary) |

Blame and File History have their own keys, listed in the generated table
above (contexts `file_insight` and `history`) and as `Blame: ...` /
`File History: ...` in the palette; all can be rebound. Details in
[Blame](log-and-history.md#blame) and [File History](log-and-history.md#file-history).
In the File History list the bare letters (`d` `o` `b` `c` `y`) act on the
selected row, but a Ctrl chord (`Ctrl+C`, `Ctrl+P`, `Ctrl+G ...`) always keeps
its global meaning.

## Commit column

The column left of a file's view (see [The commit column](interface-tour.md#the-commit-column)). `Alt+C` shows or hides it from anywhere; the other keys work once it has the keyboard (`Ctrl+N` then its badge, or `View: Focus the Commit Column`).

| Key | Action |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Pick a commit |
| `Enter` / `Right` | Go on to the picked commit's file list |
| `Esc` | Give the keyboard back to the view |
| `.`, `Menu`, `Shift+F10` | The picked row's menu |

Its other keys (`Shift+Left` / `Shift+Right` to scroll sideways, `Tab`, `]`, `[` and `0` to size the parts) are in the generated table above, context `commit_column`.

## Editor

**Edit**, or `e` in a diff or a File View, opens the built-in editor on the
working-directory copy of the file. See
[Staging and committing](staging-and-committing.md).

| Key | Action |
|---|---|
| *(type)* | Insert at the caret (replaces the selection) |
| `Enter` | New line, with the indentation of the current line |
| `Tab` / `Shift+Tab` | Indent / outdent (the selected lines, if several) |
| arrows, `Home` / `End` | Move. `Home` goes to the first non-blank character, then to column 0. `Shift` extends the selection |
| `Ctrl+Left` / `Ctrl+Right` | Move one word |
| `Ctrl+Home` / `Ctrl+End` | Start / end of the file |
| `PageUp` / `PageDown` | One page up / down |
| `Ctrl+A` | Select all |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Copy / cut / paste |
| `Ctrl+U` / `Ctrl+K` | Delete to the start / end of the line |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Redo |
| `Ctrl+S` | Save |
| `Ctrl+F` / `F3` | Find. In the find field: `Enter` / `Down` / `F3` next, `Shift+Enter` / `Up` previous, `Esc` closes |
| `Alt+M` | Minimap on or off (**Map**) |
| `Esc` | Close. With unsaved changes it asks: **Save** (`S` / `Enter`), **Discard** (`D`), **Cancel** (`C` / `Esc`) |

`Tab` indents in the editor, so it does not move the focus. Other `Ctrl` chords
(`Ctrl+P`, `Ctrl+N`) still work.

The status line's **encoding**, **line endings** and **indentation** are buttons:
each opens a dropdown (arrows or the wheel, type to filter, `Enter` to choose,
`Esc` to close). The palette (`Ctrl+P`) opens the same menus: *Editor: Change Encoding*, *Editor: Reopen with Encoding*, *Editor: Convert to Encoding*, *Editor: Convert Line Endings*, *Editor: Change Indentation*. Details
in [Staging and committing](staging-and-committing.md).

## Git actions (`Ctrl+G`)

`Ctrl+G` waits for one more key:

| Key | Action |
|---|---|
| `s` / `u` | Stage / unstage the selected file |
| `a` | Stage all changes |
| `x` | Discard all changes (asks first) |
| `c` | Open the commit composer |
| `Enter` | Commit the staged changes now, with the form's message, from anywhere (see [Staging and committing](staging-and-committing.md#writing-the-commit)) |
| `m` | Stash all changes with a message (empty is allowed). With AI on, `Ctrl+G` in this prompt writes the message |
| `Shift+T` | Stash all changes (untracked files included), without a message |
| `o` | Pop a stash: the stash row selected in the Log, or else the latest stash |
| `p` | Pull, the way the Pull button is set (`git.pull_mode`) |
| `Shift+P` | Push to the default remote (sets the upstream on the first push), or ask for the remote |
| `Shift+U` | Push all tags |
| `f` | Fetch every remote, with prune |
| `g` | Open the pull request of the checked-out branch in the browser |
| `b` / `t` | New branch / new tag at HEAD (see [Naming a branch or tag](#naming-a-branch-or-tag)) |
| `d` | Delete the selected branch or tag (asks first) |
| `e` | Edit the selected commit's message |
| `Shift+E` / `Shift+A` | Amend the selected commit (all fields) / its author (see [Amending a commit](#amending-a-commit)) |
| `y` | Cherry-pick the selected commit |
| `v` | Revert the selected commit |
| `Shift+R` | Reset (mixed) to the selected commit |
| `r` | Rebase onto the selected branch |
| `Shift+W` | Open the Flow menu (a repository with a branching workflow) |
| `Shift+H` | Open the history rewrite workbench (builds with history rewrite only) |
| `l` | Open the LFS menu. `Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V`: LFS fetch all / checkout all / pull / push all / prune. See [Git LFS](lfs.md) |
| `i` | ✦ AI: write the commit message for the staged changes |
| `w` | ✦ AI: explain the selected commit, or the working changes |
| `Shift+C` | ✦ AI: propose how to split the working changes into commits |

The AI keys work only while AI is on (see [AI assistance](ai.md)). `Esc`
cancels a running AI request.

## Right pane (working changes and commit details)

With the WIP row selected, the right pane shows the **Unstaged** and
**Staged** lists and the commit composer. With a commit selected, it shows the
commit details and its files.

| Key | Action |
|---|---|
| `j` / `k`, `Up` / `Down` | Move. At the end of one list, the selection goes on into the other |
| `h` / `l`, `Left` / `Right` | Go to the Unstaged / Staged list |
| `Enter` | Collapse or expand the active list |
| `Space` / `Insert` | Add the current file to the selection, or remove it |
| `Ctrl+A` | Select every file of the list |
| `Ctrl+Space` | Stage / unstage the selected file, or fold a folder row |
| `d` | Open the selected file's diff |
| `m` | `Path` (flat list) or `Tree` (folders) |
| `a` | `View all files`: the whole tree, not only the changed files |
| `z` | Collapse all folders, or expand them again |
| `/` | Filter a commit's changed files by name |
| `.`, `Menu`, `Shift+F10` | The file's menu |
| `c` | Focus the commit composer's summary field |
| `Shift+A` | **Amend previous commit** on or off |
| `Ctrl+N ]` / `Ctrl+N [` | Grow / shrink the commit message block |

In the commit composer:

| Key | Action |
|---|---|
| `Enter` | Summary: go to the description. Description: new line |
| `Ctrl+Enter`, `Alt+Enter`, `Ctrl+J` | Commit (`Ctrl+G Enter` does it from anywhere) |
| `Alt+R` | Insert an issue reference |
| `Esc` | Leave the composer |

The file menu holds: Stage or Unstage, Discard changes, Ignore, Stash file,
File History, File Blame, Open in external diff tool, Open in external editor,
Open file in default program, Show in folder, Copy file path, Create patch
from file changes, Edit file and Delete file. On a conflicted file it starts
with **Open in Merge Editor** and the three **Resolve using** rows. With Git
LFS on, it has LFS rows. **Discard changes** and **Delete file** ask first.

## Window buttons

Every button has a keyboard twin:

| Button | Key |
|---|---|
| 🐙 at the left of the title bar (About) | `Help: About GitPoulpi` in the palette |
| **Open in** in the title bar | `Alt+O`, `Ctrl+Alt+O` |
| `×` at the right of the title bar (quit) | `q` |
| The version in the title bar (Release notes) | `Help: Release Notes` in the palette |
| `⌕ Global search` in the title bar, left of the `⚙` | `Ctrl+Alt+F` |
| `⚙` in the title bar (Settings) | `Ctrl+,` |
| The workspace chip in the tab row | `Alt+W` |
| `+` after the last tab | `Alt+T` |
| The **Radar** tab | `Alt+R` |
| The **Workspaces** tab | `Alt+J` |
| **Activity** in the status line | `Ctrl+N Shift+A` |
| Toolbar: **Undo**, **Redo** | `Ctrl+Z`, `Ctrl+Y` |
| Toolbar: **Pull** / its `▾` | `Ctrl+G p` / `Git: Pull Options...` |
| Toolbar: **Push** / its `▾` | `Ctrl+G Shift+P` / `Git: Push to Remote...` |
| Toolbar: **Branch**, **Tag** | `Ctrl+G b`, `Ctrl+G t` |
| Toolbar: **Stash**, **Pop** | `Ctrl+G Shift+T`, `Ctrl+G o` |
| Toolbar: **LFS** (when LFS is on) | `Ctrl+G l` |
| Toolbar: **Flow** (when the repository has a workflow) | `Ctrl+G Shift+W` |
| Toolbar: **Search** | `f` |
| Toolbar: the linked pull request (`GH #12`, `GL !53`) | `Ctrl+G g` |
| Toolbar: the profile list (with two or more profiles) | `Repository: Change Profile...` |

The toolbar is described in [Tour of the interface](interface-tour.md#toolbar).

## Settings

`Ctrl+,`, `Ctrl+N ,`, the `⚙` gear, or `App: Settings...` opens
the Settings screen.

| Key | Action |
|---|---|
| `Tab` / `Shift+Tab`, `h` / `l`, `Left` / `Right` | Move between the page list and the options |
| `j` / `k`, `Up` / `Down` | Move the selection |
| `Enter` / `Space` | Open a page, toggle a checkbox, edit a field, open a dropdown, or press a button |
| `/`, `Ctrl+F` | Search all options |
| `Alt+Left` / `Alt+Right` | Switch sub-tab (General / the list) |
| `Esc` / `q` | Close Settings. While a field is edited, `Esc` cancels the edit only |
| `?` | Open the help over the Settings screen |

See [Settings](settings.md).

## Text fields

All text fields share the same keys: filters, search fields, the command
palette, prompts, the commit composer, the WIP row's field, dialog fields and
Settings fields. While a field has the keyboard, these keys win over the pane
shortcuts.

| Key | Action |
|---|---|
| a character | Type it at the caret (replaces the selection) |
| `Left` / `Right` | Move one character (with a selection: go to its start / end) |
| `Ctrl+Left` / `Ctrl+Right` | Move one word |
| `Home` / `End` | Start / end of the line |
| `Shift` + a move above | Extend or reduce the selection |
| `Up` / `Down` (description only) | Previous / next line |
| `Ctrl+Home` / `Ctrl+End` (description only) | Start / end of the text |
| `Ctrl+A` | Select all |
| `Backspace` / `Delete` | Delete the character before / after the caret, or the selection |
| `Ctrl+U` | Delete from the caret to the start of the line (or the selection) |
| `Ctrl+K` | Delete from the caret to the end of the line (or the selection) |
| `Ctrl+C` / `Ctrl+X` | Copy / cut the selection (not in a password field) |
| paste (the terminal's own paste) | Insert at the caret, replacing the selection |

In the commit description, `Enter` types a new line and `Ctrl+Enter` commits.
Terminals do not all send `Ctrl+Enter` the same way, so `Alt+Enter` and
`Ctrl+J` commit too.

Some consoles send `Ctrl+letter` as a control character (`Ctrl+A` as `0x01`).
GitPoulpi understands both forms. `AltGr` characters (`@`, `#`, `€` on Swiss
or French keyboards) are typed, and they reach every key of a list or a view too (`[`, `]`, `\`, `{`), even though Windows reports `AltGr` as `Ctrl+Alt`.

## Tooltips

A tooltip appears when the pointer stays about 0.6 s on a target (a Log cell,
a toolbar button, the linked pull request). A key press, a click, a scroll, a
resize, an open popup or menu, or the pointer leaving the target hides it.
After that, it comes back only when the pointer moves again.
