# Parity with the terminal UI

Generated from the shared registry (`gitpoulpi_app::registry::parity_rows`, every feature on) and
`desktop/parity/status.toml`. Do not edit by hand: run
`UPDATE_PARITY=1 cargo test -p gitpoulpi-desktop parity` from `desktop/` to rewrite it.
The branch merges when `parity_is_complete` passes (see `desktop/parity/README.md`).

**976 rows: 86 done, 8 terminal only, 882 to do.**

| Kind | Rows | Done | Terminal only | To do |
|---|---:|---:|---:|---:|
| action | 375 | 54 | 0 | 321 |
| palette command | 588 | 32 | 0 | 556 |
| family | 13 | 0 | 8 | 5 |

A palette command that is the only twin of a key action shares that action's row.

| Id | Context | Label | Default keys | Status |
|---|---|---|---|---|
| `global.git_prefix` | global | Git Actions Prefix | Ctrl+G | done |
| `global.nav_prefix` | global | Navigation Prefix | Ctrl+N | done |
| `global.palette` | global | Command Palette | Ctrl+P | done |
| `global.settings` | global | Settings | Ctrl+, / Ctrl+N , | to do |
| `global.home` | global | Home (Repository Management) | Ctrl+O | to do |
| `global.clone` | global | Clone Repository | Ctrl+Shift+N | to do |
| `global.new_tab` | global | New Tab | Alt+T | to do |
| `global.workspace_switch` | global | Switch Workspace | Alt+W | to do |
| `global.radar` | global | Open Radar | Alt+R | to do |
| `global.workspaces` | global | Open Workspaces | Alt+J | to do |
| `global.tab_menu` | global | Tab Menu | Ctrl+T | to do |
| `global.tab_open_in_popup` | global | Open In (list) | Alt+O | to do |
| `global.tab_open_in_default` | global | Open in Default | Ctrl+Alt+O | to do |
| `global.close_tab` | global | Close Tab | Ctrl+W | to do |
| `global.next_tab` | global | Next Tab | Ctrl+PageDown / Ctrl+Tab | to do |
| `global.prev_tab` | global | Previous Tab | Ctrl+PageUp / Ctrl+Shift+Tab | to do |
| `global.help` | global | Help | ? | done |
| `global.quit` | global | Quit | q | done |
| `global.refresh` | global | Refresh | r | done |
| `global.refresh_all` | global | Refresh (fetch, pull requests, issues) | F5 | to do |
| `global.find_in_log` | global | Search the Commit Graph | f | done |
| `global.workspace_search` | global | Search the Workspace | Ctrl+Alt+F | to do |
| `global.select_tab` | global | Select Tab | t / Ctrl+N Tab | to do |
| `global.undo` | global | Undo the Last Operation | Ctrl+Z | done |
| `global.redo` | global | Redo the Undone Operation | Ctrl+Y / Ctrl+Shift+Z | done |
| `global.undo_history` | global | Open the Undo History | Alt+Z | to do |
| `global.commit_column` | global | Show or Hide the Commit Column | Alt+C | to do |
| `global.undo_offer` | global | Restore the Branch to Where the Blocked Action Left It | Alt+Shift+Z | to do |
| `global.context_menu` | global | Context Menu | . / Menu / Shift+F10 | to do |
| `git_prefix.compose_commit` | git_prefix | Compose Commit Message | c / Ctrl+G c | done |
| `git_prefix.commit_now` | git_prefix | Commit Staged Changes Now | Enter / Ctrl+G Enter | done |
| `git_prefix.ai_compose` | git_prefix | AI: Split Changes into Commits | Shift+C | to do |
| `git_prefix.ai_explain` | git_prefix | AI: Explain Changes | w | to do |
| `git_prefix.stash_message` | git_prefix | Stash with Message | m / Ctrl+G m | to do |
| `git_prefix.ai_commit_message` | git_prefix | AI: Write Commit Message | i | to do |
| `git_prefix.stage_all` | git_prefix | Stage All Changes | a / Ctrl+G a | done |
| `git_prefix.discard_all` | git_prefix | Discard All Changes | x / Ctrl+G x | done |
| `git_prefix.stage_file` | git_prefix | Stage Selected File | s / Ctrl+G s | done |
| `git_prefix.unstage_file` | git_prefix | Unstage Selected File | u / Ctrl+G u | done |
| `git_prefix.pull` | git_prefix | Pull | p / Ctrl+G p | done |
| `git_prefix.pull_all_branches` | git_prefix | Pull All Branches | Ctrl+P / Ctrl+G Ctrl+P | to do |
| `git_prefix.push` | git_prefix | Push | Shift+P / Ctrl+G P | done |
| `git_prefix.open_linked_pr` | git_prefix | Open Linked Pull Request | g | to do |
| `git_prefix.push_tags` | git_prefix | Push All Tags | Shift+U / Ctrl+G U | to do |
| `git_prefix.fetch_all` | git_prefix | Fetch All Remotes | f / Ctrl+G f | done |
| `git_prefix.new_tag` | git_prefix | New Tag | t / Ctrl+G t | to do |
| `git_prefix.new_branch` | git_prefix | New Branch | b / Ctrl+G b | to do |
| `git_prefix.stash_push` | git_prefix | Stash Changes | Shift+T / Ctrl+G T | to do |
| `git_prefix.cherry_pick` | git_prefix | Cherry-Pick Selected Commit | y / Ctrl+G y | to do |
| `git_prefix.revert` | git_prefix | Revert Selected Commit | v / Ctrl+G v | to do |
| `git_prefix.edit_message` | git_prefix | Edit Commit Message | e / Ctrl+G e | to do |
| `git_prefix.amend` | git_prefix | Amend Selected Commit | Shift+E / Ctrl+G E | to do |
| `git_prefix.edit_author` | git_prefix | Amend Selected Commit Author | Shift+A / Ctrl+G A | to do |
| `git_prefix.reset_mixed` | git_prefix | Reset (Mixed) to Selected Commit | Shift+R / Ctrl+G R | to do |
| `git_prefix.rebase_onto` | git_prefix | Rebase onto Selected Branch | r / Ctrl+G r | to do |
| `git_prefix.stash_pop` | git_prefix | Pop Stash | o / Ctrl+G o | to do |
| `git_prefix.delete_ref` | git_prefix | Delete Selected Ref or Drop Stash | d / Ctrl+G d | to do |
| `git_prefix.lfs_menu` | git_prefix | Open the LFS Menu | l | to do |
| `git_prefix.flow_menu` | git_prefix | Open the Flow Menu | Shift+W | to do |
| `git_prefix.lfs_fetch_all` | git_prefix | LFS: Fetch All Objects | Shift+F | to do |
| `git_prefix.lfs_checkout` | git_prefix | LFS: Check Out All Files | Shift+K | to do |
| `git_prefix.lfs_pull` | git_prefix | LFS: Pull Objects | Shift+L | to do |
| `git_prefix.lfs_push_all` | git_prefix | LFS: Push All Objects | Shift+J | to do |
| `git_prefix.lfs_prune` | git_prefix | LFS: Prune Local Objects | Shift+V | to do |
| `git_prefix.rewrite_workbench` | git_prefix | Open the History Rewrite Workbench | Shift+H | to do |
| `nav_prefix.collapse_refs` | nav_prefix | Collapse or Restore Refs Pane | Shift+H / Ctrl+N H | done |
| `nav_prefix.collapse_status` | nav_prefix | Collapse or Restore Status Pane | Shift+L / Ctrl+N L | done |
| `nav_prefix.settings` | nav_prefix | Open Settings | , | to do |
| `nav_prefix.about` | nav_prefix | Open About | a | to do |
| `nav_prefix.activity_log` | nav_prefix | Open the Activity Log | Shift+A | to do |
| `nav_prefix.global_log` | nav_prefix | Open the global Activity Log | Shift+G | to do |
| `nav_prefix.jobs` | nav_prefix | Show the Background Jobs | Shift+J | to do |
| `nav_prefix.updates` | nav_prefix | Show the Update Panel | u | to do |
| `nav_prefix.help` | nav_prefix | Toggle Help | ? | done |
| `nav_prefix.select_tab` | nav_prefix | Select Tab | Tab | to do |
| `nav_prefix.change_theme` | nav_prefix | Change Theme | t | done |
| `nav_prefix.change_language` | nav_prefix | Change Language | l | done |
| `nav_prefix.change_graph_color_theme` | nav_prefix | Change Graph Color Theme | Shift+T | done |
| `nav_prefix.toggle_graph_row_tints` | nav_prefix | Toggle Graph Row Tints | Shift+Y | to do |
| `nav_prefix.grow_section` | nav_prefix | Grow Refs Section | + / = / Ctrl+N + | to do |
| `nav_prefix.shrink_section` | nav_prefix | Shrink Refs Section | - / Ctrl+N - | to do |
| `nav_prefix.grow_message` | nav_prefix | Grow Commit Message Block | ] / Ctrl+N ] | to do |
| `nav_prefix.shrink_message` | nav_prefix | Shrink Commit Message Block | [ / Ctrl+N [ | to do |
| `nav_prefix.refs_wider` | nav_prefix | Widen Refs Pane | Right / Ctrl+N Right | to do |
| `nav_prefix.refs_narrower` | nav_prefix | Narrow Refs Pane | Left / Ctrl+N Left | to do |
| `nav_prefix.status_wider` | nav_prefix | Widen Status Pane | Shift+Left / Ctrl+N Shift+Left | to do |
| `nav_prefix.status_narrower` | nav_prefix | Narrow Status Pane | Shift+Right / Ctrl+N Shift+Right | to do |
| `nav_prefix.agent_show` | nav_prefix | Agent: Show the Pending View | Enter / Ctrl+N Enter | to do |
| `nav_prefix.agent_back` | nav_prefix | Agent: Back (or Dismiss the Pending View) | Backspace | to do |
| `nav_prefix.arrange_dialog` | nav_prefix | Arrange the open dialog (move, resize) | m / Ctrl+N m | to do |
| `sequence.cancel` | sequence | Sequence: Cancel |  | to do |
| `sequence.continue` | sequence | Sequence: Continue |  | to do |
| `refs.filter` | refs | Filter Refs by Name | / | to do |
| `refs.solo` | refs | Solo the Selection | o | done |
| `refs.clear_solo` | refs | Stop Soloing | Shift+O | done |
| `refs.toggle_hierarchy` | refs | Toggle Branch Hierarchy View | y | to do |
| `refs.show_all` | refs | Show All Hidden Refs | Shift+H | done |
| `refs.reveal` | refs | Reveal in Graph | g | to do |
| `refs.mark_drop` | refs | Mark as Drop Source | m | to do |
| `refs.drop_marked` | refs | Drop the Marked Ref Here | Shift+M | to do |
| `refs.expand_all` | refs | Expand All Folders | = | done |
| `refs.collapse_all` | refs | Collapse All Folders | - | done |
| `refs.expand_below` | refs | Expand the Folder and Its Subfolders | * / Shift+Right | to do |
| `refs.collapse_below` | refs | Collapse the Folder and Its Subfolders | _ / Shift+Left | to do |
| `refs.scroll_left` | refs | Scroll the Pane Left | Alt+Left | to do |
| `refs.scroll_right` | refs | Scroll the Pane Right | Alt+Right | to do |
| `log.toggle_file_mode` | log | Files: Path or Tree | m | to do |
| `log.lfs_only` | log | Files: LFS Only | o | to do |
| `log.view_all_files` | log | Files: View All Files | a | to do |
| `log.compare_selection` | log | Compare the Selected Rows | v | to do |
| `log.compare_swap` | log | Swap the Comparison Ends | x | to do |
| `log.compare_next` | log | Go to the Next Compared Row | n | to do |
| `log.filter_author` | log | Filter the Log by Author | Shift+A | to do |
| `log.pick_column` | log | Pick a Column to Move or Size | Shift+C | to do |
| `log.scroll_left` | log | Scroll the Columns Left | Shift+Left | to do |
| `log.scroll_right` | log | Scroll the Columns Right | Shift+Right | to do |
| `log.details_scroll_left` | log | Scroll the Pane Left | Alt+Left | to do |
| `log.details_scroll_right` | log | Scroll the Pane Right | Alt+Right | to do |
| `log.copy_mermaid` | log | Copy Commits as a Mermaid gitGraph | Shift+Y | to do |
| `log_columns.next` | log_columns | Pick the Next Column | Tab | to do |
| `log_columns.prev` | log_columns | Pick the Previous Column | Shift+BackTab | to do |
| `log_columns.move_left` | log_columns | Move the Column Left | < | to do |
| `log_columns.move_right` | log_columns | Move the Column Right | > | to do |
| `log_columns.wider` | log_columns | Widen the Column | ] | to do |
| `log_columns.narrower` | log_columns | Narrow the Column | [ | to do |
| `log_columns.reset_width` | log_columns | Reset the Column Widths | 0 | done |
| `log_columns.fit` | log_columns | Fit the Column to Its Content | = | to do |
| `log_columns.fit_all` | log_columns | Fit All Columns to Content | + | done |
| `log_columns.layout` | log_columns | Scroll Sideways or Fit the Pane | s | to do |
| `log_files.collapse_all` | log_files | Collapse or Expand All Folders | z | to do |
| `log_files.filter` | log_files | Filter the Changed Files | / | to do |
| `log_files.expand_all` | log_files | Expand All Folders | = | to do |
| `log_files.collapse_all_folders` | log_files | Collapse All Folders | - | to do |
| `log_files.expand_below` | log_files | Expand the Folder and Its Subfolders | * / Shift+Right | to do |
| `log_files.collapse_below` | log_files | Collapse the Folder and Its Subfolders | _ / Shift+Left | to do |
| `log_files.scroll_left` | log_files | Scroll the Pane Left | Alt+Left | to do |
| `log_files.scroll_right` | log_files | Scroll the Pane Right | Alt+Right | to do |
| `status.compose` | status | Focus the Commit Message | c | done |
| `status.toggle_file_mode` | status | Files: Path or Tree | m | done |
| `status.lfs_only` | status | Files: LFS Only | o | to do |
| `status.view_all_files` | status | Files: View All Files | a | to do |
| `status.amend` | status | Toggle Amend | Shift+A | done |
| `status.skip_hooks` | status | Toggle Skip Hooks | Shift+H | done |
| `status.collapse_all` | status | Collapse or Expand All Folders | z | done |
| `status.stage_toggle` | status | Stage or Unstage the Selected Row | Ctrl+Space | done |
| `status.open_diff` | status | Open the Selected File's Diff | d | done |
| `status.expand_all` | status | Expand All Folders | = | done |
| `status.collapse_all_folders` | status | Collapse All Folders | - | done |
| `status.expand_below` | status | Expand the Folder and Its Subfolders | * / Shift+Right | done |
| `status.collapse_below` | status | Collapse the Folder and Its Subfolders | _ / Shift+Left | done |
| `status.scroll_left` | status | Scroll the Pane Left | Alt+Left | to do |
| `status.scroll_right` | status | Scroll the Pane Right | Alt+Right | to do |
| `home.favorite` | home | Star or Unstar the Repository | f / * | to do |
| `home.close_repo` | home | Close, Remove or Unstar the Row | x / Delete | to do |
| `home.copy_path` | home | Copy the Repository Path | y | to do |
| `home.open_in` | home | Open the Repository in the Default Tool | o | to do |
| `home.browse` | home | Browse for a Repository | b | to do |
| `home.clone` | home | Clone a Repository | c | to do |
| `home.init` | home | Initialize a Repository | i | to do |
| `home.collapse_all` | home | Collapse All Sections | - | to do |
| `home.expand_all` | home | Expand All Sections | + / = | to do |
| `home.search` | home | Search the Repositories | / | to do |
| `home.workspace_new` | home | New Workspace | n | to do |
| `home.workspace_edit` | home | Edit the Workspace | e | to do |
| `home.workspace_from_tabs` | home | New Workspace from the Open Tabs | a | to do |
| `home.workspace_fetch` | home | Fetch All in the Workspace | Shift+F | to do |
| `home.workspace_pull` | home | Pull All in the Workspace | Shift+P | to do |
| `home.workspace_clone_missing` | home | Clone the Missing Repositories | m | to do |
| `home.workspace_locate` | home | Locate the Missing Repository | l | to do |
| `refs.hosting_filter` | refs | Pull Requests / Issues: Choose the Filter | v | to do |
| `refs.hosting_edit` | refs | Pull Request: Edit on the Host | e | to do |
| `refs.new_pull_request` | refs | Pull Requests: Create One | n | to do |
| `refs.add_remote` | refs | Remotes: Add a Remote | + | to do |
| `refs.remotes` | refs | Branch: Edit its Remotes | u | to do |
| `diff.close` | diff | Close the Diff | q | done |
| `diff.edit` | diff | Edit in Working Directory | e | to do |
| `diff.lfs_raw` | diff | LFS: Card or Raw Pointer | r | to do |
| `diff.copy` | diff | Copy the Selection or Line | y / Ctrl+C | to do |
| `diff.stage_hunk` | diff | Stage or Unstage the Hunk | Space | done |
| `diff.stage_lines` | diff | Stage or Unstage the Selected Lines | Enter | done |
| `diff.discard_hunk` | diff | Discard the Hunk | x | done |
| `diff.discard_lines` | diff | Discard the Selected Lines | Shift+X | done |
| `diff.select_lines` | diff | Start or Drop a Line Selection | v | done |
| `diff.next_hunk` | diff | Next Hunk | n | done |
| `diff.prev_hunk` | diff | Previous Hunk | p | done |
| `diff.toggle_layout` | diff | Layout: Unified or Side by Side | m | done |
| `diff.toggle_hunks_only` | diff | Hunks Only or Whole File | Shift+H | to do |
| `diff.toggle_minimap` | diff | Toggle the Minimap | Shift+M | to do |
| `diff.choose_encoding` | diff | Choose the Encoding | Shift+C | to do |
| `diff.toggle_wrap` | diff | Toggle Word Wrap | w | done |
| `diff.toggle_whitespace` | diff | Show or Hide Whitespace-Only Changes | s | done |
| `diff.merge_editor` | diff | Open the Merge Editor (Conflicted File) | Shift+E | to do |
| `file_insight.close` | file_insight | Close the File View | q | to do |
| `file_insight.edit` | file_insight | Edit in Working Directory | e | to do |
| `file_insight.copy` | file_insight | Copy the Selection or Row | y / Ctrl+C | to do |
| `file_insight.last_row` | file_insight | Jump to the Last Row | Shift+G | to do |
| `file_insight.open_commit` | file_insight | Go to the Row's Commit | Enter | to do |
| `history.show_in_log` | history | Show in the Log | Enter | to do |
| `history.copy_sha` | history | Copy the commit SHA | y | to do |
| `history.copy_summary` | history | Copy the Summary | Shift+Y | to do |
| `history.show_diff` | history | Show the diff of This commit | d / Alt+1 | to do |
| `history.open_file` | history | Open the File at This commit | o / Alt+2 | to do |
| `history.blame` | history | Blame at This commit | b / Alt+3 | to do |
| `history.compare_working_tree` | history | Compare with the Working Tree | c | to do |
| `history.restore` | history | Restore This Version | Shift+R | to do |
| `history.range_up` | history | Extend the Selection Up | Shift+Up | to do |
| `history.range_down` | history | Extend the Selection Down | Shift+Down | to do |
| `history.switch_focus` | history | Switch Between the List and the Diff | Tab | to do |
| `history.widen_list` | history | Widen the List | Alt+Right | to do |
| `history.narrow_list` | history | Narrow the List | Alt+Left | to do |
| `history.reset_split` | history | Reset the List Width | Alt+0 | to do |
| `commit_column.scroll_left` | commit_column | Scroll the Column Left | Shift+Left | to do |
| `commit_column.scroll_right` | commit_column | Scroll the Column Right | Shift+Right | to do |
| `commit_column.next_part` | commit_column | Size the Next Part | Tab | to do |
| `commit_column.move_left` | commit_column | Move the Part Left | < | to do |
| `commit_column.move_right` | commit_column | Move the Part Right | > | to do |
| `commit_column.wider_part` | commit_column | Widen the Part | ] | to do |
| `commit_column.narrower_part` | commit_column | Narrow the Part | [ | to do |
| `commit_column.reset_widths` | commit_column | Reset the Part Widths | 0 | to do |
| `commit_column.fit` | commit_column | Fit the Part to Its Content | = | to do |
| `commit_column.fit_all` | commit_column | Fit All Parts to Content | + | to do |
| `file_insight.blame_card` | file_insight | Blame: Show or Hide the Detail Card | i | to do |
| `file_insight.blame_next` | file_insight | Blame: Next Hunk | n | to do |
| `file_insight.blame_prev` | file_insight | Blame: Previous Hunk | p | to do |
| `file_insight.blame_next_same` | file_insight | Blame: Next Hunk of the Same Commit | Shift+N | to do |
| `file_insight.blame_prev_same` | file_insight | Blame: Previous Hunk of the Same Commit | Shift+P | to do |
| `file_insight.blame_goto` | file_insight | Blame: Go to Line | g | to do |
| `file_insight.blame_copy_sha` | file_insight | Blame: Copy Commit SHA | Shift+Y | to do |
| `file_insight.blame_diff` | file_insight | Blame: Show the Commit's Diff for This File | d | to do |
| `file_insight.blame_parent` | file_insight | Blame: Blame at the Commit's Parent | b | to do |
| `file_insight.blame_history` | file_insight | Blame: Open File History | Shift+H | to do |
| `file_insight.blame_wider` | file_insight | Blame: Widen the Annotation Column | > | to do |
| `file_insight.blame_narrower` | file_insight | Blame: Narrow the Annotation Column | < | to do |
| `image.zoom_in` | image | Zoom the Image In | + / = | to do |
| `image.zoom_out` | image | Zoom the Image Out | - | to do |
| `image.fit` | image | Fit the Image to the View | 0 | to do |
| `image.actual_size` | image | Show the Image at Actual Size | 1 | to do |
| `image.flip_side` | image | Show the Other Side (Before / After) | Tab | to do |
| `image.widen_before` | image | Widen the Before Pane | Alt+Right | to do |
| `image.widen_after` | image | Widen the After Pane | Alt+Left | to do |
| `image.reset_split` | image | Reset the Before / After Split | Alt+0 | to do |
| `image.edit_in` | image | Open With... | Shift+O | to do |
| `image.edit_default` | image | Open in the Default App | o | to do |
| `editor.save` | editor | Save | Ctrl+S | to do |
| `editor.undo` | editor | Undo | Ctrl+Z | to do |
| `editor.redo` | editor | Redo | Ctrl+Y / Ctrl+Shift+Z | to do |
| `editor.find` | editor | Find | Ctrl+F | to do |
| `editor.find_next` | editor | Find Next (Shift: Previous) | F3 | to do |
| `editor.paste` | editor | Paste | Ctrl+V | to do |
| `editor.toggle_minimap` | editor | Toggle the Minimap | Alt+M | to do |
| `editor_prompt.save` | editor_prompt | Unsaved Changes: Save and Close | s / Shift+S | to do |
| `editor_prompt.discard` | editor_prompt | Unsaved Changes: Discard and Close | d / Shift+D | to do |
| `editor_prompt.cancel` | editor_prompt | Unsaved Changes: Keep Editing | c / Shift+C | to do |
| `merge.next_conflict` | merge | Next Conflict | F7 / Alt+Down | to do |
| `merge.prev_conflict` | merge | Previous Conflict | Shift+F7 / Alt+Up | to do |
| `merge.next_file` | merge | Next Conflicted File | Alt+PageDown | to do |
| `merge.prev_file` | merge | Previous Conflicted File | Alt+PageUp | to do |
| `merge.next_pane` | merge | Focus the Next Pane | F6 | to do |
| `merge.prev_pane` | merge | Focus the Previous Pane | Shift+F6 | to do |
| `merge.toggle_panes` | merge | Show or Hide the Side Panels (Refs and Status) | F9 | to do |
| `merge.toggle_base` | merge | Show or Hide the Base Pane | Alt+D | to do |
| `merge.toggle_full_file` | merge | Toggle Full File Diff | Alt+F | to do |
| `merge.undo` | merge | Undo (Picks and Typing) | Ctrl+Z | to do |
| `merge.redo` | merge | Redo (Picks and Typing) | Ctrl+Y / Ctrl+Shift+Z | to do |
| `merge.widen_current` | merge | Widen the Current (A) Pane | Alt+Right | to do |
| `merge.widen_incoming` | merge | Widen the Incoming (B) Pane | Alt+Left | to do |
| `merge.reset_panes` | merge | Reset the Pane Sizes | Alt+0 | to do |
| `merge.save` | merge | Save without Marking Resolved | Alt+S | to do |
| `merge.save_resolve` | merge | Save and Mark Resolved | Ctrl+S | to do |
| `merge.ai_suggest` | merge | ✦ Suggest a Resolution (AI) | Alt+I | to do |
| `merge.unpick` | merge | Unpick This Conflict (Back to Unresolved) | Alt+U | to do |
| `merge.reset_all` | merge | Reset the Whole File to the Original Conflicts | Alt+R | to do |
| `merge.focus_a` | merge | Focus the Current (A) Pane | Alt+1 | to do |
| `merge.focus_b` | merge | Focus the Incoming (B) Pane | Alt+2 | to do |
| `merge.focus_output` | merge | Focus the Output Pane | Alt+3 | to do |
| `merge.focus_base` | merge | Focus the Base Pane | Alt+4 | to do |
| `merge_sides.toggle_line` | merge_sides | Pick or Drop the Line | Space | to do |
| `merge_sides.toggle_chunk` | merge_sides | Pick or Drop the Whole Chunk | Enter | to do |
| `merge_sides.take_a` | merge_sides | Take Current (A) Only | a | to do |
| `merge_sides.take_b` | merge_sides | Take Incoming (B) Only | b | to do |
| `merge_sides.take_both` | merge_sides | Take Both, A Then B | c | to do |
| `merge_sides.take_both_reversed` | merge_sides | Take Both, B Then A | Shift+C | to do |
| `merge_sides.clear` | merge_sides | Take Nothing for This Conflict | x | to do |
| `merge_sides.unpick` | merge_sides | Unpick This Conflict (Back to Unresolved) | u | to do |
| `merge_sides.reset_all` | merge_sides | Reset the Whole File to the Original Conflicts | Shift+U | to do |
| `merge_sides.swap_order` | merge_sides | Swap the Order of the Two Sides | s | to do |
| `merge_sides.next_conflict` | merge_sides | Next Conflict | n | to do |
| `merge_sides.prev_conflict` | merge_sides | Previous Conflict | Shift+N | to do |
| `merge_sides.take_all_a` | merge_sides | Take All Current (A) | Shift+A | to do |
| `merge_sides.take_all_b` | merge_sides | Take All Incoming (B) | Shift+B | to do |
| `merge_sides.toggle_base` | merge_sides | Show or Hide the Base Pane | d | to do |
| `merge_sides.scroll_left` | merge_sides | Scroll the Panes Left | Shift+Left / Shift+H | to do |
| `merge_sides.scroll_right` | merge_sides | Scroll the Panes Right | Shift+Right / Shift+L | to do |
| `merge_sides.toggle_full_file` | merge_sides | Toggle Full File Diff | f | to do |
| `merge_sides.cycle_height` | merge_sides | Cycle the Output Height | z | to do |
| `merge_card.take_a` | merge_card | Keep A (Current) as Is | a | to do |
| `merge_card.take_b` | merge_card | Take B (Incoming) as Is | b | to do |
| `merge_card.take_base` | merge_card | Take the Base (Common Ancestor) | o | to do |
| `merge_card.keep` | merge_card | Keep the File That Is Left | k | to do |
| `merge_card.delete` | merge_card | Accept the Deletion | Shift+D | to do |
| `merge_card.external_tool` | merge_card | Open in the External Merge Tool | m | to do |
| `rebase.pick` | rebase | Pick | p | to do |
| `rebase.reword` | rebase | Reword | r | to do |
| `rebase.squash` | rebase | Squash | s | to do |
| `rebase.fixup` | rebase | Fixup | f | to do |
| `rebase.drop` | rebase | Drop | d | to do |
| `rebase.edit` | rebase | Edit (Stop to Amend) | e | to do |
| `rebase.autosquash` | rebase | Toggle Autosquash | a | to do |
| `rebase.start` | rebase | Start the Plan | Ctrl+S / Ctrl+Enter | to do |
| `rebase.reset` | rebase | Reset the Plan | Ctrl+R | to do |
| `rebase.split` | rebase | Split the Row's Commit | t | to do |
| `rebase.move_up` | rebase | Move the Row Up | Ctrl+Up / Alt+Up | to do |
| `rebase.move_down` | rebase | Move the Row Down | Ctrl+Down / Alt+Down | to do |
| `compose.apply` | compose | Create the Commits | Ctrl+S / Ctrl+Enter / Alt+Enter | to do |
| `split.line_mode` | split | Toggle Line Selection | v | to do |
| `split.move_up` | split | Move the Commit Up | Alt+Up / Shift+K | to do |
| `split.move_down` | split | Move the Commit Down | Alt+Down / Shift+J | to do |
| `split.reset` | split | Reset the Split | Shift+R | to do |
| `rewrite.preview` | rewrite | Preview the Rewrite | F5 | to do |
| `rewrite.apply` | rewrite | Apply the Rewrite | Ctrl+Enter / Ctrl+S | to do |
| `rewrite.cancel` | rewrite | Cancel the Preview | Ctrl+X | to do |
| `rewrite.add_rule` | rewrite | Add a Rule | a / Insert | to do |
| `rewrite.toggle_rule` | rewrite | Switch the Rule On or Off | Space | to do |
| `rewrite.move_up` | rewrite | Move the Rule Up | Alt+Up | to do |
| `rewrite.move_down` | rewrite | Move the Rule Down | Alt+Down | to do |
| `rewrite.delete_rule` | rewrite | Delete the Rule | d / Delete | to do |
| `rewrite.restore_rule` | rewrite | Restore the Deleted Rule | Ctrl+Z | to do |
| `rewrite.duplicate_rule` | rewrite | Duplicate the Rule | Ctrl+D | to do |
| `rewrite.next_tab` | rewrite | Next Preview Tab | Alt+Right | to do |
| `rewrite.prev_tab` | rewrite | Previous Preview Tab | Alt+Left | to do |
| `rewrite.save_shared` | rewrite | Save the Recipe to the Repository | Alt+S | to do |
| `rewrite.save_mine` | rewrite | Save the Recipe for Me Only | Alt+M | to do |
| `rewrite.load` | rewrite | Load a Recipe | Alt+L | to do |
| `rewrite.name` | rewrite | Rename the Recipe | F2 | to do |
| `rewrite.protected_all` | rewrite | Include All Protected Branches | Alt+A | to do |
| `rewrite.protected_none` | rewrite | Leave Out All Protected Branches | Alt+U | to do |
| `rewrite.check_tools` | rewrite | Check the Tools Again | F6 | to do |
| `rewrite.analyze` | rewrite | Analyse the History Sizes | F7 | to do |
| `rewrite.host_howto` | rewrite | Explain the Host's Branch Protection | Alt+H | to do |
| `rewrite.ai_rules` | rewrite | Create Rewrite Rules with AI | Alt+I | to do |
| `reword.apply` | reword | Apply the New Message | Ctrl+Enter / Ctrl+S | to do |
| `reword.deeper` | reword | Modify History Deeper | Alt+H | to do |
| `composer.commit` | composer | Commit from the Composer | Ctrl+Enter / Alt+Enter | done |
| `composer.insert_issue_ref` | composer | Insert an Issue Reference | Alt+R | to do |
| `md_editor.preview` | md_editor | Markdown Editor: Toggle Write and Preview | Alt+P | to do |
| `md_editor.split` | md_editor | Markdown Editor: Toggle the Split View | Alt+L | to do |
| `md_editor.submit` | md_editor | Markdown Editor: Send the Text | Ctrl+Enter / Ctrl+S / Alt+Enter | to do |
| `md_editor.ai` | md_editor | Markdown Editor: Write with AI | Alt+E | to do |
| `dialog.browse` | dialog | Browse for a Folder | Ctrl+B | to do |
| `dialog.sources` | dialog | Go to the Clone Sources | Alt+S | to do |
| `dialog.submit` | dialog | Submit the Pull Request Form | Ctrl+S | to do |
| `dialog.web` | dialog | Open the Provider's Web Form | Alt+W | to do |
| `folder_picker.choose` | folder_picker | Choose the Folder | Ctrl+S / Ctrl+Enter | to do |
| `folder_picker.new_folder` | folder_picker | New Folder | Alt+N | to do |
| `folder_picker.filter` | folder_picker | Filter the Folder's Entries | Ctrl+F / / | to do |
| `folder_picker.toggle_siblings` | folder_picker | Show or Hide Sibling Folders | Alt+S | to do |
| `folder_picker.toggle_hidden` | folder_picker | Show or Hide Hidden Folders | Alt+H | to do |
| `preferences.search` | preferences | Search the Settings | Ctrl+F / / | to do |
| `preferences.list_add` | preferences | Add a List Item | Insert | to do |
| `preferences.list_duplicate` | preferences | Duplicate the Selected List Item | Ctrl+D | to do |
| `preferences.list_remove` | preferences | Remove the Selected List Item | Delete | to do |
| `preferences.list_move_up` | preferences | Move the Selected List Item Up | Alt+Up | to do |
| `preferences.list_move_down` | preferences | Move the Selected List Item Down | Alt+Down | to do |
| `preferences.prev_tab` | preferences | Previous Tab of the Settings Page | Alt+Left | to do |
| `preferences.next_tab` | preferences | Next Tab of the Settings Page | Alt+Right | to do |
| `ai_popup.copy` | ai_popup | Copy the Result | c | to do |
| `activity_log.scroll_left` | activity_log | Scroll the Activity Log Left | Left | to do |
| `activity_log.scroll_right` | activity_log | Scroll the Activity Log Right | Right | to do |
| `integrations.add` | integrations | Add a Profile | a | to do |
| `integrations.edit` | integrations | Edit the Selected Profile | e | to do |
| `integrations.open_token` | integrations | Open the Token Page | o | to do |
| `integrations.remove` | integrations | Remove the Selected Profile | d / Delete | to do |
| `palette.configure_shortcut` | palette | Palette: Set the Highlighted Command's Shortcut | Alt+K | to do |
| `palette.arrange` | palette | Dialog: Move or Resize the Dialog in Front | Alt+G | to do |
| `prompt.ai_stash_message` | prompt | AI: Generate Stash Message | Ctrl+G | to do |
| `palette.focus_refs` | palette | Navigation: Focus Refs Pane | Ctrl+N <badge> | done |
| `palette.focus_log` | palette | Navigation: Focus Log Pane | Ctrl+N <badge> | done |
| `palette.focus_status` | palette | Navigation: Focus Status Pane | Ctrl+N <badge> | done |
| `palette.focus_refs_filter` | palette | Navigation: Focus Refs Filter | Ctrl+N <badge> | done |
| `palette.focus_refs_section_local` | palette | Navigation: Focus Refs > Local | Ctrl+N <badge> | to do |
| `palette.focus_refs_section_remote` | palette | Navigation: Focus Refs > Remote | Ctrl+N <badge> | to do |
| `palette.focus_refs_section_worktrees` | palette | Navigation: Focus Refs > Worktrees | Ctrl+N <badge> | to do |
| `palette.focus_refs_section_submodules` | palette | Navigation: Focus Refs > Submodules | Ctrl+N <badge> | to do |
| `palette.focus_refs_section_stashes` | palette | Navigation: Focus Refs > Stashes | Ctrl+N <badge> | to do |
| `palette.focus_refs_section_tags` | palette | Navigation: Focus Refs > Tags | Ctrl+N <badge> | to do |
| `palette.sidebar_sections_menu` | palette | Left pane: Show/Hide Sections... |  | to do |
| `palette.reset_sidebar_sections` | palette | Left pane: Reset Sections to Default |  | to do |
| `palette.use_sidebar_sections_for_all` | palette | Left pane: Apply These Sections to All Repositories |  | to do |
| `palette.toggle_sidebar_section_local` | palette | Left pane: Show/Hide Local Branches |  | to do |
| `palette.toggle_sidebar_section_remote` | palette | Left pane: Show/Hide Remotes |  | to do |
| `palette.toggle_sidebar_section_worktrees` | palette | Left pane: Show/Hide Worktrees |  | to do |
| `palette.show_worktrees_section` | palette | Sidebar: Show Worktrees Section |  | to do |
| `palette.toggle_sidebar_section_stashes` | palette | Left pane: Show/Hide Stashes |  | to do |
| `palette.toggle_sidebar_section_tags` | palette | Left pane: Show/Hide Tags |  | to do |
| `palette.toggle_sidebar_section_pull_requests` | palette | Left pane: Show/Hide Pull requests |  | to do |
| `palette.toggle_sidebar_section_issues` | palette | Left pane: Show/Hide Issues |  | to do |
| `palette.toggle_sidebar_section_submodules` | palette | Left pane: Show/Hide Submodules |  | to do |
| `palette.focus_status_section_unstaged` | palette | Navigation: Focus Status > Unstaged | Ctrl+N <badge> | done |
| `palette.focus_status_section_staged` | palette | Navigation: Focus Status > Staged | Ctrl+N <badge> | done |
| `palette.cycle_focus` | palette | Navigation: Cycle Focus to Next Pane | Tab | to do |
| `palette.cycle_focus_back` | palette | Navigation: Cycle Focus to Previous Pane | Shift+Tab | to do |
| `palette.clear_author_filter` | palette | Log: Clear Author Filter |  | to do |
| `palette.reset_refs_section_sizes` | palette | Navigation: Reset Refs Section Sizes |  | to do |
| `palette.reset_layout` | palette | Navigation: Reset Layout |  | to do |
| `palette.reset_commit_message_size` | palette | Commit Details: Reset Message Size |  | to do |
| `palette.scroll_commit_message_down` | palette | Commit Details: Scroll Message Down |  | to do |
| `palette.scroll_commit_message_up` | palette | Commit Details: Scroll Message Up |  | to do |
| `palette.open_message_link` | palette | Commit Details: Open Link in Message... |  | to do |
| `palette.toggle_commit_file_sort` | palette | Files: Toggle Commit File Sort (A-Z / Z-A) |  | to do |
| `palette.checkout_selected_ref` | palette | Refs: Check Out Selected Ref |  | to do |
| `palette.copy_ref_field_name` | palette | Refs: Copy Ref Name |  | to do |
| `palette.copy_ref_field_full_ref` | palette | Refs: Copy Full Ref (refs/heads/...) |  | to do |
| `palette.copy_ref_field_target_sha` | palette | Refs: Copy Ref Target SHA |  | to do |
| `palette.merge_selected_ref` | palette | Refs: Merge Selected Branch into Current |  | to do |
| `palette.drop_selected_ref_on_current` | palette | Refs: Drop Selected Ref on Checked-Out Branch... |  | to do |
| `palette.rename_selected_branch` | palette | Refs: Rename Selected Branch... |  | to do |
| `palette.fast_forward_selected_to_current` | palette | Refs: Fast-Forward Selected Branch to Current |  | to do |
| `palette.fast_forward_current_to_selected` | palette | Refs: Fast-Forward Current Branch to Selected |  | to do |
| `palette.create_worktree_from_selected_ref` | palette | Refs: Create Worktree from Selected Branch or Tag... |  | to do |
| `palette.reflog_selected_branch` | palette | Refs: Reflog of the Selected Branch... |  | to do |
| `palette.start_pull_request_from_selected_branch` | palette | Refs: Start a Pull Request from Selected Branch... |  | to do |
| `palette.open_worktree_in_tab` | palette | Refs: Open Worktree in Tab |  | to do |
| `palette.worktree_open_in_selected` | palette | Refs: Open Worktree In... |  | to do |
| `palette.worktree_merge_selected` | palette | Refs: Merge Worktree Branch Into... |  | to do |
| `palette.worktree_remove_selected` | palette | Refs: Remove Worktree... |  | to do |
| `palette.create_worktree_menu` | palette | Refs: Create Worktree... | + | to do |
| `palette.create_worktree_from_current_branch` | palette | Refs: Create Worktree from Current Branch |  | to do |
| `palette.create_worktree_choose_branch` | palette | Refs: Create Worktree Choosing a Branch |  | to do |
| `palette.submodule_update_selected` | palette | Refs: Submodule Update (Selected) |  | to do |
| `palette.submodule_init_selected` | palette | Refs: Submodule Init (Selected) |  | to do |
| `palette.submodule_open_selected` | palette | Refs: Submodule Open in Tab |  | to do |
| `palette.submodule_update_all` | palette | Git: Update All Submodules |  | to do |
| `palette.create_worktree_at_selected_commit` | palette | Git: Create Worktree at Selected Commit... |  | to do |
| `palette.create_patch_at_selected_commit` | palette | Git: Create Patch from Selected Commit... |  | to do |
| `palette.create_patch_from_selected_commit_to_head` | palette | Git: Create Patch from Selected Commit to HEAD... |  | to do |
| `palette.create_patch_from_wip` | palette | Git: Create Patch from Working Tree Changes... |  | to do |
| `palette.rewrite_purge` | palette | Rewrite: Purge Old History... |  | to do |
| `palette.apply_patch` | palette | Git: Apply Patch... |  | to do |
| `palette.rewrite_rule_identity` | palette | Rewrite: Replace an Author... |  | to do |
| `palette.rewrite_rule_anonymise` | palette | Rewrite: Anonymise Authors... |  | to do |
| `palette.rewrite_rule_mailmap` | palette | Rewrite: Apply .mailmap to History... |  | to do |
| `palette.rewrite_identities` | palette | Rewrite: Merge Identities (.mailmap)... |  | to do |
| `palette.rewrite_rule_date_shift` | palette | Rewrite: Shift Commit Dates... |  | to do |
| `palette.rewrite_rule_date_timezone` | palette | Rewrite: Change Commit Time Zone... |  | to do |
| `palette.rewrite_rule_date_window` | palette | Rewrite: Keep Commits Within Hours... |  | to do |
| `palette.rewrite_rule_committer_from_author` | palette | Rewrite: Set Committer Date to Author Date |  | to do |
| `palette.rewrite_rule_message_replace` | palette | Rewrite: Search and Replace in Messages... |  | to do |
| `palette.rewrite_rule_message_delete_lines` | palette | Rewrite: Delete Message Lines... |  | to do |
| `palette.rewrite_rule_trailer` | palette | Rewrite: Add or Remove Trailers... |  | to do |
| `palette.rewrite_rule_remove_signatures` | palette | Rewrite: Remove Signatures... |  | to do |
| `palette.rewrite_rule_message_encoding` | palette | Rewrite: Fix Message Encoding... |  | to do |
| `palette.rewrite_rule_content_replace` | palette | Rewrite: Search and Replace in Files... |  | to do |
| `palette.rewrite_rule_remove_paths` | palette | Rewrite: Remove Files from History... |  | to do |
| `palette.rewrite_rule_remove_blobs_over` | palette | Rewrite: Remove Large Files from History... |  | to do |
| `palette.rewrite_rule_move_paths` | palette | Rewrite: Move or Rename Paths... |  | to do |
| `palette.rewrite_rule_lfs_import` | palette | Rewrite: Migrate Files to LFS... |  | to do |
| `palette.rewrite_rule_lfs_export` | palette | Rewrite: Bring Files Back from LFS... |  | to do |
| `palette.rewrite_rule_line_endings` | palette | Rewrite: Convert Line Endings... |  | to do |
| `palette.rewrite_rule_strip_exec` | palette | Rewrite: Remove the Executable Bit... |  | to do |
| `palette.rewrite_rule_reencode` | palette | Rewrite: Convert File Encoding... |  | to do |
| `palette.rewrite_rule_script` | palette | Rewrite: Run a Script... |  | to do |
| `palette.rewrite_script_ai` | palette | Rewrite: Write a Script with AI... |  | to do |
| `palette.rewrite_rules_ai` | palette | Rewrite: Create Rules with AI... |  | to do |
| `palette.rewrite_from_selected_commit` | palette | Rewrite: From the Selected Commit to HEAD... |  | to do |
| `palette.rewrite_selected_commits` | palette | Rewrite: Selected Commits... |  | to do |
| `palette.lfs_initialize` | palette | LFS: Initialize for This Repository |  | to do |
| `palette.lfs_track_pattern` | palette | LFS: Track Pattern... |  | to do |
| `palette.lfs_untrack_pattern` | palette | LFS: Untrack Pattern... |  | to do |
| `palette.lfs_fetch` | palette | LFS: Fetch Objects |  | to do |
| `palette.lfs_push` | palette | LFS: Push Objects |  | to do |
| `palette.lfs_show_locks` | palette | LFS: Show Locks |  | to do |
| `palette.lfs_open_install_page` | palette | LFS: Open Install Page |  | to do |
| `palette.lfs_toggle_raw_pointer` | palette | LFS: Toggle Raw Pointer in Diff | r | to do |
| `palette.lfs_track_selected_file` | palette | LFS: Track Selected File |  | to do |
| `palette.lfs_track_selected_extension` | palette | LFS: Track Selected File's Extension |  | to do |
| `palette.lfs_lock_selected_file` | palette | LFS: Lock Selected File |  | to do |
| `palette.lfs_unlock_selected_file` | palette | LFS: Unlock Selected File |  | to do |
| `palette.lfs_force_unlock_selected_file` | palette | LFS: Force Unlock Selected File |  | to do |
| `palette.delete_selected_remote_branch` | palette | Refs: Delete Selected Branch from Its Remote... |  | to do |
| `palette.delete_selected_branch_and_remote` | palette | Refs: Delete Selected Branch Locally and on Its Remote... |  | to do |
| `palette.copy_selected_branch_link` | palette | Refs: Copy Link to Selected Branch on Remote |  | to do |
| `palette.copy_selected_commit_link` | palette | Git: Copy Link to Selected Commit on Remote |  | to do |
| `palette.create_annotated_tag_at_selected_commit` | palette | Git: Annotated Tag at Selected Commit... |  | to do |
| `palette.push_selected_tag` | palette | Refs: Push Selected Tag to Default Remote |  | to do |
| `palette.delete_selected_tag_from_remote` | palette | Refs: Delete Selected Tag from Its Remote... |  | to do |
| `palette.fetch_selected_tag` | palette | Git: Fetch Selected Tag |  | to do |
| `palette.take_remote_selected_tag` | palette | Git: Use the Remote's Version of the Selected Tag |  | to do |
| `palette.force_push_selected_tag` | palette | Git: Force Push the Selected Tag to Its Remote |  | to do |
| `palette.annotate_selected_tag` | palette | Refs: Annotate Selected Tag... |  | to do |
| `palette.move_selected_tag_to_selected_commit` | palette | Refs: Move Selected Tag to Selected Commit... |  | to do |
| `palette.toggle_hide_in_graph` | palette | Refs: Toggle Hide in Graph |  | done |
| `palette.toggle_solo` | palette | Refs: Toggle Solo on Selected Ref | o | done |
| `palette.toggle_hide_selected_folder` | palette | Refs: Toggle Hide on Selected Folder/Remote |  | done |
| `palette.toggle_solo_selected_folder` | palette | Refs: Toggle Solo on Selected Folder/Remote | o | done |
| `palette.filter_refs` | palette | Refs: Filter by Name... | / | to do |
| `palette.clear_refs_filter` | palette | Refs: Clear Name Filter | Esc | to do |
| `palette.next_ref_section` | palette | Refs: Next Section | l | to do |
| `palette.prev_ref_section` | palette | Refs: Previous Section | h | to do |
| `palette.toggle_section_collapse` | palette | Refs: Expand or Collapse Active Section | Enter | done |
| `palette.file_menu_discard` | palette | Working Changes: Discard Selected File... |  | done |
| `palette.file_menu_ignore_file` | palette | Working Changes: Ignore Selected File |  | to do |
| `palette.file_menu_ignore_extension` | palette | Working Changes: Ignore Selected File's Extension |  | to do |
| `palette.file_menu_treat_file_binary` | palette | Working Changes: Treat Selected File as Binary... |  | to do |
| `palette.file_menu_treat_file_text` | palette | Working Changes: Treat Selected File as Text... |  | to do |
| `palette.file_menu_treat_extension_binary` | palette | Working Changes: Treat Selected File's Extension as Binary... |  | to do |
| `palette.file_menu_treat_extension_text` | palette | Working Changes: Treat Selected File's Extension as Text... |  | to do |
| `palette.file_menu_stash_file` | palette | Working Changes: Stash Selected File |  | to do |
| `palette.file_menu_file_history` | palette | Working Changes: Selected File History |  | to do |
| `palette.file_menu_file_blame` | palette | Working Changes: Selected File Blame |  | to do |
| `palette.file_menu_external_diff` | palette | Working Changes: Open Selected File in External Diff Tool |  | to do |
| `palette.file_menu_open_in_editor` | palette | Working Changes: Open Selected File in External Editor |  | to do |
| `palette.file_menu_open_in_default` | palette | Working Changes: Open Selected File in Default Program |  | to do |
| `palette.file_menu_show_in_folder` | palette | Working Changes: Show Selected File in Folder |  | to do |
| `palette.file_menu_copy_path` | palette | Working Changes: Copy Selected File Path |  | to do |
| `palette.file_menu_create_patch` | palette | Working Changes: Create Patch from Selected File... |  | to do |
| `palette.file_menu_edit_file` | palette | Working Changes: Edit Selected File |  | to do |
| `palette.file_menu_delete_file` | palette | Working Changes: Delete Selected File... |  | to do |
| `palette.open_folder_menu` | palette | Working Changes: Open Selected Folder Menu |  | to do |
| `palette.create_file` | palette | Working Changes: Create File |  | to do |
| `palette.edit_wip_message` | palette | Git: Edit WIP Commit Message (in the Log) |  | to do |
| `palette.pull_with_mode_merge` | palette | Git: Pull (fast-forward if possible) |  | to do |
| `palette.pull_with_mode_ff_only` | palette | Git: Pull (fast-forward only) |  | to do |
| `palette.pull_with_mode_rebase` | palette | Git: Pull (rebase) |  | to do |
| `palette.pull_all_branches_with_mode_merge` | palette | Git: Pull All Branches (fast-forward if possible) |  | to do |
| `palette.pull_all_branches_with_mode_ff_only` | palette | Git: Pull All Branches (fast-forward only) |  | to do |
| `palette.pull_all_branches_with_mode_rebase` | palette | Git: Pull All Branches (rebase) |  | to do |
| `palette.choose_pull_button_action` | palette | Git: Pull Options... |  | to do |
| `palette.push_to_remote` | palette | Git: Push to Remote... |  | to do |
| `palette.push_to_selected_remotes` | palette | Git: Push to Selected Remotes... |  | to do |
| `palette.push_to_all_targets` | palette | Git: Push to All Targets |  | to do |
| `palette.pull_from_all_sources` | palette | Git: Pull from All Sources |  | to do |
| `palette.pull_from_one_remote` | palette | Git: Pull from One Remote... |  | to do |
| `palette.show_grouped_push` | palette | Git: Show Grouped Push |  | to do |
| `palette.continue_grouped_push` | palette | Git: Continue Grouped Push |  | to do |
| `palette.abort_grouped_push` | palette | Git: Abort Grouped Push |  | to do |
| `palette.retry_failed_pushes` | palette | Git: Retry Failed Pushes |  | to do |
| `palette.force_push_with_lease` | palette | Git: Force Push (with lease)... |  | to do |
| `palette.set_upstream` | palette | Git: Remotes of Branch... |  | to do |
| `palette.add_remote` | palette | Remote: Add Remote... |  | to do |
| `palette.remove_remote` | palette | Remote: Remove Remote... |  | to do |
| `palette.edit_remote` | palette | Remote: Edit Remote... |  | to do |
| `palette.set_default_remote` | palette | Remote: Set Default Remote... |  | to do |
| `palette.copy_remote_name` | palette | Remote: Copy Remote Name |  | to do |
| `palette.copy_remote_link` | palette | Remote: Copy Link to Remote |  | to do |
| `palette.fetch_remote` | palette | Remote: Fetch Remote... |  | to do |
| `palette.remote_actions` | palette | Remote: Actions for Selected Remote... | . | to do |
| `palette.sparse_checkout` | palette | Repository: Sparse Checkout... |  | to do |
| `palette.repo_auth_open` | palette | Repository: SSH & Credentials... |  | to do |
| `palette.repo_auth_test` | palette | Repository: Test Connection |  | to do |
| `palette.repo_auth_defaults` | palette | Repository: Use Default Credentials |  | to do |
| `palette.flow_start_any` | palette | Flow: Start Branch... |  | to do |
| `palette.flow_new_issue_any` | palette | Flow: New Issue... |  | to do |
| `palette.flow_pick_up` | palette | Flow: Pick Up Host Branch... |  | to do |
| `palette.flow_finish` | palette | Flow: Finish Current Branch... |  | to do |
| `palette.flow_sync` | palette | Flow: Sync with Parent... |  | to do |
| `palette.flow_promote` | palette | Flow: Promote... |  | to do |
| `palette.flow_set_parent` | palette | Flow: Set Parent... |  | to do |
| `palette.flow_resume` | palette | Flow: Continue or Abort Stopped Action... |  | to do |
| `palette.flow_toggle_grouping` | palette | Flow: Toggle Grouped Branches |  | to do |
| `palette.flow_settings` | palette | Flow: Workflow Settings... |  | to do |
| `palette.hide_remote_in_graph` | palette | Remote: Hide Remote in Graph... |  | to do |
| `palette.show_remote_in_graph` | palette | Remote: Show Remote in Graph... |  | to do |
| `palette.stash_apply` | palette | Git: Stash Apply |  | to do |
| `palette.stash_drop` | palette | Git: Stash Drop... |  | to do |
| `palette.checkout_selected_commit` | palette | Git: Check Out Selected Commit (detached HEAD) |  | to do |
| `palette.create_branch_at_selected_commit` | palette | Git: Create Branch at Selected Commit... |  | to do |
| `palette.create_tag_at_selected_commit` | palette | Git: Create Tag at Selected Commit... |  | to do |
| `palette.copy_commit_field_message` | palette | Commit: Copy Message |  | to do |
| `palette.copy_commit_field_author_name` | palette | Commit: Copy Author Name |  | to do |
| `palette.copy_commit_field_author_email` | palette | Commit: Copy Author Email |  | to do |
| `palette.copy_commit_field_date` | palette | Commit: Copy Date |  | to do |
| `palette.copy_commit_field_short_sha` | palette | Commit: Copy Short SHA |  | to do |
| `palette.copy_commit_field_long_sha` | palette | Commit: Copy Full SHA |  | to do |
| `palette.amend_selected_commit_summary` | palette | Commit: Amend Summary... |  | to do |
| `palette.amend_selected_commit_description` | palette | Commit: Amend Description... |  | to do |
| `palette.split_selected_commit` | palette | Git: Split Selected Commit... |  | to do |
| `palette.interactive_rebase_from_selected_commit` | palette | Git: Interactively Rebase from Selected Commit... |  | to do |
| `palette.rebase_continue` | palette | Rebase: Continue |  | to do |
| `palette.rebase_skip` | palette | Rebase: Skip |  | to do |
| `palette.rebase_abort` | palette | Rebase: Abort |  | to do |
| `palette.am_continue` | palette | git am: Continue |  | to do |
| `palette.am_skip` | palette | git am: Skip This Patch |  | to do |
| `palette.am_abort` | palette | git am: Abort |  | to do |
| `palette.index_lock_remove` | palette | Repository: Remove Stale index.lock |  | to do |
| `palette.auto_stash_pop` | palette | Banner: Pop Pending Auto-Stash |  | to do |
| `palette.auto_stash_finish` | palette | Banner: Keep the Result of the Pending Auto-stash |  | to do |
| `palette.auto_stash_show` | palette | Banner: Show Pending Auto-Stash |  | to do |
| `palette.conflicts_show` | palette | Conflicts: Show Predicted Conflicts |  | to do |
| `palette.auto_stash_drop` | palette | Banner: Drop Pending Auto-Stash... |  | to do |
| `palette.auto_stash_start_over` | palette | Banner: Start Over... |  | to do |
| `palette.reset_to_selected_commit_soft` | palette | Git: Reset Branch to Selected Commit (soft - keep index and working tree) |  | to do |
| `palette.reset_to_selected_commit_hard` | palette | Git: Reset Branch to Selected Commit (hard - discard everything)... |  | to do |
| `palette.resolve_conflict_with_ours` | palette | Git: Resolve Conflict Using Ours (this branch) |  | to do |
| `palette.resolve_conflict_with_theirs` | palette | Git: Resolve Conflict Using Theirs (incoming) |  | to do |
| `palette.resolve_conflict_with_base` | palette | Git: Resolve Conflict Using Base (common ancestor) |  | to do |
| `palette.open_merge_editor` | palette | Conflicts: Open Merge Editor |  | to do |
| `palette.merge_chunk_a` | palette | Merge: Take Chunk from A |  | to do |
| `palette.merge_chunk_b` | palette | Merge: Take Chunk from B |  | to do |
| `palette.merge_take_base` | palette | Merge: Take Base for This Conflict |  | to do |
| `palette.merge_edit_in_output` | palette | Merge: Edit This Conflict in Output |  | to do |
| `palette.merge_pane_left` | palette | Merge: Move to the Pane on the Left (Same Conflict) |  | to do |
| `palette.merge_pane_right` | palette | Merge: Move to the Pane on the Right (Same Conflict) |  | to do |
| `palette.merge_continue_from_disk` | palette | Merge: Continue from the File on Disk |  | to do |
| `palette.merge_start_over` | palette | Merge: Start Over from the Merge |  | to do |
| `palette.merge_unresolve` | palette | Merge: Unresolve and Reopen |  | to do |
| `palette.merge_reload` | palette | Merge: Reload the File |  | to do |
| `palette.merge_close` | palette | Merge: Close Merge Editor |  | to do |
| `palette.abort_operation` | palette | Git: Abort Merge, Cherry-Pick or Revert |  | to do |
| `palette.skip_operation` | palette | Git: Skip Cherry-Pick or Revert |  | to do |
| `palette.browse_repository` | palette | Repository: Browse for a Repository... |  | done |
| `palette.create_repository` | palette | Repository: Create New (Init)... |  | to do |
| `palette.management_open_selected` | palette | Repositories: Open Selected | Enter | to do |
| `palette.management_selected_menu` | palette | Repositories: Selected Repository Menu... | . | to do |
| `palette.management_search` | palette | Repositories: Search | / / Ctrl+F | to do |
| `palette.management_close_all_tabs` | palette | Repositories: Close All Tabs... |  | to do |
| `palette.management_remove_selected_from_list` | palette | Repositories: Remove Selected from List |  | to do |
| `palette.management_remove_all_recent` | palette | Repositories: Remove All Recent |  | to do |
| `palette.workspace_close` | palette | Workspace: Close (No Workspace) |  | to do |
| `palette.workspace_close_tab` | palette | Workspace: Close Tab |  | to do |
| `palette.workspace_delete` | palette | Workspace: Delete... |  | to do |
| `palette.workspace_add_tab` | palette | Workspace: Add Tab to Workspace |  | to do |
| `palette.workspace_remove_tab` | palette | Workspace: Remove Tab from Workspace |  | to do |
| `palette.radar_refresh` | palette | Radar: Refresh | r | to do |
| `palette.radar_search` | palette | Radar: Search | / | to do |
| `palette.radar_act_open_browser` | palette | Radar: Open in Browser | o | to do |
| `palette.radar_act_open_repo` | palette | Radar: Open Repository | Enter | to do |
| `palette.radar_act_checkout` | palette | Radar: Check Out Branch | c | to do |
| `palette.radar_act_worktree` | palette | Radar: Create Worktree... | t | to do |
| `palette.radar_act_show` | palette | Radar: Show in Repository | s | to do |
| `palette.radar_act_start_work` | palette | Radar: Start Work on Issue... | n | to do |
| `palette.radar_act_pin` | palette | Radar: Pin or Unpin | p | to do |
| `palette.radar_act_snooze` | palette | Radar: Snooze... | z | to do |
| `palette.radar_act_review` | palette | Radar: Review Request... | e | to do |
| `palette.radar_act_conversation` | palette | Radar: Open Issue Conversation | i | to do |
| `palette.radar_act_merge` | palette | Radar: Merge Request... | m | to do |
| `palette.radar_act_close_item` | palette | Radar: Close... | x | to do |
| `palette.radar_act_ready` | palette | Radar: Mark as Ready | y | to do |
| `palette.radar_act_clone_repo` | palette | Radar: Clone Repository... | g | to do |
| `palette.radar_act_review_api` | palette | Radar: Review Through the API |  | to do |
| `palette.radar_act_clone_review` | palette | Radar: Clone and Review... |  | to do |
| `palette.radar_act_edit_assignees` | palette | Radar: Edit Assignees... |  | to do |
| `palette.radar_act_edit_reviewers` | palette | Radar: Edit Reviewers... |  | to do |
| `palette.radar_act_edit_labels` | palette | Radar: Edit Labels... |  | to do |
| `palette.radar_act_set_milestone` | palette | Radar: Set Milestone... |  | to do |
| `palette.radar_scope` | palette | Radar: Choose Scope | w | to do |
| `palette.radar_view` | palette | Radar: Choose View | v | to do |
| `palette.radar_filter` | palette | Radar: Choose Filter | f | to do |
| `palette.radar_profile` | palette | Radar: Filter by Profile... | P | to do |
| `palette.radar_not_cloned` | palette | Radar: Show Not-Cloned Requests | a | to do |
| `palette.radar_drafts` | palette | Radar: Show Drafts | d | to do |
| `palette.radar_act_turn_on` | palette | Radar: Turn On Account-Wide Review Requests... | T | to do |
| `palette.radar_menu` | palette | Radar: Row Menu... | . | to do |
| `palette.radar_column_pick` | palette | Radar: Pick Column | C | to do |
| `palette.radar_column_wider` | palette | Radar: Widen Column | ] | to do |
| `palette.radar_column_narrower` | palette | Radar: Narrow Column | [ | to do |
| `palette.radar_column_reset` | palette | Radar: Reset Column Widths | 0 | to do |
| `palette.radar_column_fit` | palette | Radar: Fit Column to Its Content | = | to do |
| `palette.radar_column_fit_all` | palette | Radar: Fit All Columns to Content | + | to do |
| `palette.radar_back` | palette | Radar: Back to Home | Esc | to do |
| `palette.radar_close_tab` | palette | Radar: Close Tab |  | to do |
| `palette.open_new_menu` | palette | Tab: New... |  | to do |
| `palette.tab_menu_close_others` | palette | Tab: Close Other Tabs... |  | to do |
| `palette.tab_menu_close_to_the_right` | palette | Tab: Close Tabs to the Right... |  | to do |
| `palette.tab_menu_toggle_favorite` | palette | Tab: Add to or Remove from Favorites |  | to do |
| `palette.tab_menu_copy_path` | palette | Tab: Copy Repository Path |  | to do |
| `palette.tab_menu_open_in_file_manager` | palette | Tab: Open in File Manager |  | to do |
| `palette.tab_menu_open_in_terminal` | palette | Tab: Open in Terminal |  | to do |
| `palette.tab_dir_terminal` | palette | Tab: Terminal Here |  | to do |
| `palette.tab_open_agent_here` | palette | Tab: Open AI Agent Here |  | to do |
| `palette.change_profile` | palette | Repository: Change Profile... |  | to do |
| `palette.about_licenses` | palette | Help: Licenses and Credits |  | to do |
| `palette.updates_release_notes` | palette | Help: Release Notes |  | to do |
| `palette.updates_security_notice` | palette | Updates: Open Security Notice |  | to do |
| `palette.updates_copy_command` | palette | Updates: Copy Update Command |  | to do |
| `palette.updates_run_in_terminal` | palette | Updates: Run Update Command in a Terminal |  | to do |
| `palette.updates_check_now` | palette | Updates: Check for Updates Now |  | to do |
| `palette.updates_download` | palette | Updates: Download |  | to do |
| `palette.updates_update_now` | palette | Updates: Update Now |  | to do |
| `palette.updates_restart` | palette | Updates: Restart to Update |  | to do |
| `palette.updates_revert` | palette | Updates: Revert to the Previous Version |  | to do |
| `palette.updates_skip` | palette | Updates: Skip This Version |  | to do |
| `palette.updates_close_tab` | palette | Release Notes: Close Tab |  | to do |
| `palette.jobs_cancel_all` | palette | Jobs: Cancel All That Can Be Canceled |  | to do |
| `palette.copy_diagnostics` | palette | View: Copy Diagnostics |  | to do |
| `palette.activity_level_toggle_error` | palette | Activity Log: Show or Hide Level Error |  | to do |
| `palette.activity_level_toggle_warn` | palette | Activity Log: Show or Hide Level Warning |  | to do |
| `palette.activity_level_toggle_info` | palette | Activity Log: Show or Hide Level Info |  | to do |
| `palette.activity_level_toggle_debug` | palette | Activity Log: Show or Hide Level Debug |  | to do |
| `palette.activity_level_toggle_trace` | palette | Activity Log: Show or Hide Level Trace |  | to do |
| `palette.activity_level_all` | palette | Activity Log: Show All Levels |  | to do |
| `palette.activity_level_clear` | palette | Activity Log: Show No Level |  | to do |
| `palette.activity_level_clear_log` | palette | Activity Log: Clear the Application Log |  | to do |
| `palette.activity_level_scope` | palette | Activity Log: Choose the Scope |  | to do |
| `palette.activity_level_git_tab` | palette | Activity Log: Git Operations |  | to do |
| `palette.activity_level_app_tab` | palette | Activity Log: Application Log |  | to do |
| `palette.activity_level_show_operation` | palette | Activity Log: Show the Operation of This Line |  | to do |
| `palette.activity_level_next_problem` | palette | Activity Log: Next Failed Operation |  | to do |
| `palette.activity_level_prev_problem` | palette | Activity Log: Previous Failed Operation |  | to do |
| `palette.copy_last_error` | palette | Activity Log: Copy the Last Error |  | to do |
| `palette.run_last_error_remedy` | palette | Activity Log: Run the Suggested Action |  | to do |
| `palette.open_last_error` | palette | Activity Log: Open the Last Error |  | to do |
| `palette.tab_dir_quit_cd` | palette | Tab: Quit and cd Here |  | to do |
| `palette.move_repository_tab_false` | palette | Tab: Move Left |  | to do |
| `palette.move_repository_tab_true` | palette | Tab: Move Right |  | to do |
| `palette.undo_forget` | palette | Edit: Forget Blocked Undo Entry |  | to do |
| `palette.reflog_head` | palette | Edit: Reflog... |  | to do |
| `palette.lost_commits` | palette | Edit: Recover Lost Commits... |  | to do |
| `palette.undo_sequence` | palette | Edit: Undo Whole Sequence |  | to do |
| `palette.review_tab_overview` | palette | Review: Show Overview |  | to do |
| `palette.review_tab_files` | palette | Review: Show Files |  | to do |
| `palette.review_tab_commits` | palette | Review: Show Commits |  | to do |
| `palette.review_add_comment` | palette | Review: Comment on Line... | c | to do |
| `palette.review_conversation_comment` | palette | Review: Comment on Conversation... | C | to do |
| `palette.review_reply` | palette | Review: Reply to Thread... | r | to do |
| `palette.review_toggle_resolve` | palette | Review: Resolve or Reopen Thread | x | to do |
| `palette.review_back_to_comment` | palette | Review: Back to Comment in Overview | Esc | to do |
| `palette.review_toggle_thread` | palette | Review: Fold or Unfold Thread | z | to do |
| `palette.review_edit_pending` | palette | Review: Edit Pending Comment... | e | to do |
| `palette.review_delete_pending` | palette | Review: Delete Pending Comment | d | to do |
| `palette.review_toggle_viewed` | palette | Review: Mark File as Viewed | v | to do |
| `palette.review_next_file` | palette | Review: Next File | ] | to do |
| `palette.review_prev_file` | palette | Review: Previous File | [ | to do |
| `palette.review_finish_review` | palette | Review: Finish Review... | f | to do |
| `palette.review_discard_review` | palette | Review: Discard Pending Review... |  | to do |
| `palette.review_refresh` | palette | Review: Refresh | F5 | to do |
| `palette.review_open_browser` | palette | Review: Open in Browser | o | to do |
| `palette.review_checkout` | palette | Review: Check Out Branch | b | to do |
| `palette.review_close` | palette | Review: Close | Esc | to do |
| `palette.review_edit_assignees` | palette | Review: Edit Assignees... | a | to do |
| `palette.review_edit_reviewers` | palette | Review: Edit Reviewers... | p | to do |
| `palette.review_edit_labels` | palette | Review: Edit Labels... | l | to do |
| `palette.review_set_milestone` | palette | Review: Set Milestone... | m | to do |
| `palette.review_pick_patch_set` | palette | Review: Choose Patch Set... | P | to do |
| `palette.review_compare_patch_set` | palette | Review: Compare Patch Sets... | D | to do |
| `palette.review_submit_change` | palette | Review: Submit Change... | U | to do |
| `palette.review_abandon_change` | palette | Review: Abandon Change... | Q | to do |
| `palette.review_toggle_layout` | palette | Review: Toggle Inline / Split Layout | V | to do |
| `palette.review_toggle_hunks_only` | palette | Review: Toggle Whole File / Only the Hunks | H | to do |
| `palette.review_toggle_whitespace` | palette | Review: Toggle Whitespace-Only Changes | S | to do |
| `palette.review_toggle_wrap` | palette | Review: Toggle Word Wrap | W | to do |
| `palette.review_next_hunk` | palette | Review: Next Hunk | J | to do |
| `palette.review_prev_hunk` | palette | Review: Previous Hunk | K | to do |
| `palette.review_choose_encoding` | palette | Review: Choose Encoding... | E | to do |
| `palette.review_toggle_minimap` | palette | Review: Toggle Minimap | M | to do |
| `palette.review_scroll_left` | palette | Review: Scroll Diff Left | Left | to do |
| `palette.review_scroll_right` | palette | Review: Scroll Diff Right | Right | to do |
| `palette.review_copy_text` | palette | Review: Copy Selected Text or Line | Ctrl+C | to do |
| `palette.board_move_to` | palette | Board: Move Card To... | m | to do |
| `palette.board_move_left` | palette | Board: Move Card Left |  | to do |
| `palette.board_move_right` | palette | Board: Move Card Right |  | to do |
| `palette.board_open_card` | palette | Board: Open Card | Enter | to do |
| `palette.board_edit_assignees` | palette | Board: Edit Assignees... |  | to do |
| `palette.board_edit_labels` | palette | Board: Edit Labels... |  | to do |
| `palette.board_set_milestone` | palette | Board: Set Milestone... |  | to do |
| `palette.board_new_card` | palette | Board: New Issue in This Column | n | to do |
| `palette.board_filter_assignee` | palette | Board: Filter by Assignee... | a | to do |
| `palette.board_filter_label` | palette | Board: Filter by Label... | l | to do |
| `palette.board_filter_milestone` | palette | Board: Filter by Milestone... | M | to do |
| `palette.board_filter_text` | palette | Board: Filter by Text | / | to do |
| `palette.board_clear_filters` | palette | Board: Clear Filters | x | to do |
| `palette.board_set_column_labels` | palette | Board: Set Column Labels... | c | to do |
| `palette.board_refresh` | palette | Board: Refresh | r | to do |
| `palette.board_close` | palette | Board: Close | Esc | to do |
| `palette.item_back` | palette | Conversation: Back |  | to do |
| `palette.item_refresh` | palette | Conversation: Refresh | F5 | to do |
| `palette.item_load_older` | palette | Conversation: Load Older Entries | m | to do |
| `palette.item_prev_entry` | palette | Conversation: Previous Entry | Up | to do |
| `palette.item_next_entry` | palette | Conversation: Next Entry | Down | to do |
| `palette.item_next_unread` | palette | Conversation: Next Unread Entry | n | to do |
| `palette.item_reply` | palette | Conversation: Reply to Entry | r | to do |
| `palette.item_quote` | palette | Conversation: Quote Entry | Q | to do |
| `palette.item_new_comment` | palette | Conversation: New Comment | c | to do |
| `palette.item_links` | palette | Conversation: Links of Entry... | l | to do |
| `palette.item_open_diagram` | palette | Conversation: Open Diagram of Entry | g | to do |
| `palette.item_open_code` | palette | Conversation: Open Code of Thread | G | to do |
| `palette.item_copy_link` | palette | Conversation: Copy Link of Entry | y | to do |
| `palette.item_copy_text` | palette | Conversation: Copy Text | Ctrl+C | to do |
| `palette.item_open_browser` | palette | Conversation: Open in Browser | o | to do |
| `palette.item_edit` | palette | Conversation: Edit Comment | e | to do |
| `palette.item_delete` | palette | Conversation: Delete Comment | d | to do |
| `palette.item_toggle_events` | palette | Conversation: Show or Hide Events | v | to do |
| `palette.item_toggle_preview` | palette | Conversation: Toggle Write and Preview |  | to do |
| `palette.item_toggle_split` | palette | Conversation: Toggle the Split View |  | to do |
| `palette.item_send` | palette | Conversation: Send the Comment |  | to do |
| `palette.item_toggle_internal` | palette | Conversation: Toggle Internal Note |  | to do |
| `palette.item_close` | palette | Conversation: Close | Esc | to do |
| `palette.composer_write_with_ai` | palette | Markdown Editor: Write with AI |  | to do |
| `palette.composer_refresh_context` | palette | Markdown Editor: Refresh the AI Context |  | to do |
| `palette.hosting_open_issue_conversation` | palette | Hosting: Open Issue Conversation... |  | to do |
| `palette.review_suggest_review` | palette | Review: ✦ Draft Review... | s | to do |
| `palette.review_accept_suggestion` | palette | Review: Accept Suggestion | y | to do |
| `palette.review_edit_suggestion` | palette | Review: Edit and Accept Suggestion... | w | to do |
| `palette.review_dismiss_suggestion` | palette | Review: Dismiss Suggestion | n | to do |
| `palette.review_accept_all_suggestions` | palette | Review: Accept All Suggestions | A | to do |
| `palette.review_dismiss_all_suggestions` | palette | Review: Dismiss All Suggestions | N | to do |
| `palette.review_use_summary` | palette | Review: Use AI Summary in Review Text | i | to do |
| `palette.review_use_draft` | palette | Review: ✦ Use the Draft... | u | to do |
| `palette.review_edit_draft_text` | palette | Review: Edit the ✦ Draft Text... | t | to do |
| `palette.review_discard_draft` | palette | Review: Discard the ✦ Draft | N | to do |
| `palette.safety_undo_to` | palette | Undo History: Undo Up to Here... | u | to do |
| `palette.safety_redo_to` | palette | Undo History: Redo Up to Here... | y | to do |
| `palette.safety_undo_sequence` | palette | Undo History: Undo Whole Sequence | x | to do |
| `palette.safety_restore_only` | palette | Undo History: Restore Only This... | r | to do |
| `palette.safety_offer` | palette | Undo History: Restore the Branch to Where It Was... | o | to do |
| `palette.safety_history_refresh` | palette | Undo History: Refresh |  | to do |
| `palette.safety_close` | palette | Undo History: Close | Esc | to do |
| `palette.safety_reflog_show` | palette | Reflog: Show in Log | l | to do |
| `palette.safety_reflog_branch` | palette | Reflog: Create Branch Here... | b | to do |
| `palette.safety_reflog_tag` | palette | Reflog: Create tag Here... | t | to do |
| `palette.safety_reflog_apply` | palette | Reflog: Apply Changes to Working Tree | w | to do |
| `palette.safety_reflog_stash` | palette | Reflog: Save Changes as stash | s | to do |
| `palette.safety_reflog_restore` | palette | Reflog: Restore to This Position... | r | to do |
| `palette.safety_reflog_more` | palette | Reflog: Load More | m | to do |
| `palette.safety_reflog_refresh` | palette | Reflog: Refresh |  | to do |
| `palette.safety_lost_branch` | palette | Lost Work: Create Branch Here... | b | to do |
| `palette.safety_lost_tag` | palette | Lost Work: Create tag Here... | t | to do |
| `palette.safety_lost_pick` | palette | Lost Work: Cherry-pick | c | to do |
| `palette.safety_lost_apply` | palette | Lost Work: Apply Changes to Working Tree | w | to do |
| `palette.safety_lost_stash` | palette | Lost Work: Save Changes as stash | s | to do |
| `palette.safety_lost_refresh` | palette | Lost Work: Scan Again |  | to do |
| `palette.safety_next_pane` | palette | Reflog / Lost Work: Next Pane | Tab | to do |
| `palette.safety_path_tree` | palette | Reflog / Lost Work: Path or Tree File List | v | to do |
| `palette.safety_reset_panes` | palette | Reflog / Lost Work: Reset Pane Sizes |  | to do |
| `palette.safety_reset_dialog` | palette | Dialog: Reset Position and Size |  | to do |
| `palette.refresh_all` | palette | View: Refresh (fetch, pull requests, issues) | F5 | to do |
| `palette.toggle_column_ref` | palette | View: Toggle Ref Column |  | to do |
| `palette.toggle_column_graph` | palette | View: Toggle Graph Column |  | to do |
| `palette.toggle_column_message` | palette | View: Toggle Message Column |  | to do |
| `palette.toggle_column_author` | palette | View: Toggle Author Column |  | to do |
| `palette.toggle_column_date` | palette | View: Toggle Date Column |  | to do |
| `palette.toggle_column_changes` | palette | View: Toggle Changes Column |  | to do |
| `palette.toggle_column_sha` | palette | View: Toggle SHA Column |  | to do |
| `palette.log_column_chooser` | palette | View: Log: Choose the Columns to Show |  | to do |
| `palette.open_diff` | palette | Diff: Open Diff for Selected File | Enter | to do |
| `palette.close_diff` | palette | Diff: Close Diff View | Esc | done |
| `palette.next_hunk` | palette | Diff: Next Hunk | n | done |
| `palette.prev_hunk` | palette | Diff: Previous Hunk | p | done |
| `palette.stage_hunk` | palette | Diff: Stage Hunk Under Cursor | Space | done |
| `palette.unstage_hunk` | palette | Diff: Unstage Hunk Under Cursor | Space | done |
| `palette.select_lines` | palette | Diff: Select Lines (or clear the selection) | v | done |
| `palette.stage_lines` | palette | Diff: Stage Selected Lines | Enter | done |
| `palette.unstage_lines` | palette | Diff: Unstage Selected Lines | Enter | done |
| `palette.discard_hunk` | palette | Diff: Discard Hunk Under Cursor... | x | done |
| `palette.discard_lines` | palette | Diff: Discard Selected Lines... | X | done |
| `palette.copy_selected_text` | palette | Copy: Selected Text (drag over the text first) | Ctrl+C | to do |
| `palette.copy_line` | palette | Copy: Line Under the Cursor (or the selected lines) | y | to do |
| `palette.copy_hunk` | palette | Diff: Copy Hunk Under the Cursor |  | to do |
| `palette.diff_mode_inline` | palette | Diff: Layout Inline (+/- in one column) | m | done |
| `palette.diff_mode_split` | palette | Diff: Layout Split (before and after) | m | done |
| `palette.toggle_diff_hunks_only` | palette | Diff: Toggle Hunks Only (whole file / only the changes) | H | to do |
| `palette.files_open_menu` | palette | Files: Open Menu of the Row or Selection | . | to do |
| `palette.files_select_all` | palette | Files: Select All |  | done |
| `palette.files_clear_selection` | palette | Files: Clear Selection |  | done |
| `palette.files_toggle_selection` | palette | Files: Toggle Selection of the Current Row |  | done |
| `palette.toggle_word_wrap` | palette | Diff: Toggle Word Wrap | w | done |
| `palette.diff_choose_encoding` | palette | Diff: Choose Encoding... | C | to do |
| `palette.toggle_minimap` | palette | View: Toggle Minimap | M | to do |
| `palette.toggle_diff_intraline` | palette | Diff: Toggle Changed Words Highlight |  | to do |
| `palette.commit_column_set_off` | palette | View: Commit Column: Hide |  | to do |
| `palette.commit_column_set_file` | palette | View: Commit Column: File History |  | to do |
| `palette.commit_column_set_graph` | palette | View: Commit Column: All Commits |  | to do |
| `palette.commit_column_focus` | palette | View: Focus the Commit Column |  | to do |
| `palette.commit_column_widen` | palette | View: Widen the Commit Column |  | to do |
| `palette.commit_column_narrow` | palette | View: Narrow the Commit Column |  | to do |
| `palette.commit_column_menu` | palette | View: Commit Column: Menu of the Selected Row |  | to do |
| `palette.commit_column_columns_menu` | palette | View: Commit Column: Choose the Columns |  | to do |
| `palette.commit_column_toggle_shown_refs` | palette | View: Commit Column: Toggle Ref |  | to do |
| `palette.commit_column_toggle_shown_graph` | palette | View: Commit Column: Toggle Graph |  | to do |
| `palette.commit_column_toggle_shown_author` | palette | View: Commit Column: Toggle Author |  | to do |
| `palette.commit_column_toggle_shown_date` | palette | View: Commit Column: Toggle Date |  | to do |
| `palette.commit_column_toggle_shown_changes` | palette | View: Commit Column: Toggle Changes |  | to do |
| `palette.commit_column_toggle_shown_sha` | palette | View: Commit Column: Toggle SHA |  | to do |
| `palette.commit_column_show_show_in_log` | palette | View: Commit Column: Show the Selected Row in the Log |  | to do |
| `palette.commit_column_show_diff` | palette | View: Commit Column: Show the Changes of the File at the Selected Row |  | to do |
| `palette.commit_column_show_file` | palette | View: Commit Column: Show the File as It Was at the Selected Row |  | to do |
| `palette.commit_column_show_blame` | palette | View: Commit Column: Blame the File at the Selected Row |  | to do |
| `palette.editor_toggle_follow_selection` | palette | Editor: Toggle Follow Selection |  | to do |
| `palette.image_edit_in` | palette | Image: Edit In... | O | to do |
| `palette.image_edit_default` | palette | Image: Edit in Default Editor | o | to do |
| `palette.open_with` | palette | File: Open With... | O | to do |
| `palette.open_with_default` | palette | File: Open in Default App | o | to do |
| `palette.toggle_whitespace` | palette | Diff: Toggle Whitespace-Only Changes | s | done |
| `palette.ai_generate_commit_message` | palette | AI: Generate Commit Message | Ctrl+G i | to do |
| `palette.ai_generate_stash_message` | palette | AI: Generate Stash Message |  | to do |
| `palette.ai_insert_prompt_variable` | palette | AI: Insert Prompt Variable |  | to do |
| `palette.ai_edit_profile` | palette | AI: Edit Profile... |  | to do |
| `palette.ai_open_agent` | palette | AI: Open the Agent... |  | to do |
| `palette.ai_explain_selected_commit` | palette | AI: Explain Selected Commit | Ctrl+G w | to do |
| `palette.ai_split_selected_commit` | palette | AI: Split Selected Commit... |  | to do |
| `palette.ai_explain_wip` | palette | AI: Explain Working Changes | Ctrl+G w | to do |
| `palette.ai_compose_commits` | palette | AI: Compose Commits from Working Changes... | Ctrl+G C | to do |
| `palette.stage_guard_show_in_file` | palette | Sensitive Files: Show in File | S | to do |
| `palette.composer_move_file` | palette | Composer: Move File to Commit... | m | to do |
| `palette.blame` | palette | Diff: Blame |  | to do |
| `palette.history` | palette | Diff: File History |  | to do |
| `palette.file_view` | palette | Diff: File View |  | to do |
| `palette.edit_in_working_directory` | palette | File: Edit in Working Directory | e | to do |
| `palette.editor_menu_encoding` | palette | Editor: Change Encoding |  | to do |
| `palette.editor_menu_reopen` | palette | Editor: Reopen with Encoding |  | to do |
| `palette.editor_menu_convert` | palette | Editor: Convert to Encoding |  | to do |
| `palette.editor_menu_line_endings` | palette | Editor: Convert Line Endings |  | to do |
| `palette.editor_menu_indentation` | palette | Editor: Change Indentation |  | to do |
| `palette.sequence_create_ai` | palette | Sequence: Create with AI... |  | to do |
| `palette.search_settings` | palette | App: Search Settings... |  | to do |
| `palette.keybindings` | palette | App: Keybindings... |  | done |
| `palette.agent_dismiss_pending` | palette | Agent: Dismiss Pending View |  | to do |
| `palette.sequence_repo_review` | palette | Sequence: Review This Repository's Sequences |  | to do |
| `palette.sequence_repo_trust` | palette | Sequence: Trust This Repository's Sequences |  | to do |
| `palette.sequence_repo_ignore` | palette | Sequence: Ignore This Repository's Sequences |  | to do |
| `palette.keybindings_reset_all` | palette | Keybindings: Reset All... |  | to do |
| `palette.hosting_open_linked_pr` | palette | Integrations: Open Linked Pull Request | Ctrl+G g | to do |
| `palette.hosting_edit_linked_pr` | palette | Integrations: Edit Linked Pull Request on the Host |  | to do |
| `palette.hosting_copy_linked_pr_url` | palette | Integrations: Copy Linked Pull Request URL |  | to do |
| `palette.hosting_refresh_hosting` | palette | Integrations: Refresh Pull Requests and Issues |  | to do |
| `palette.hosting_checkout_pull_request` | palette | Integrations: Check Out Pull Request... |  | to do |
| `palette.hosting_open_commit_reference` | palette | Integrations: Open Commit Reference (#issue / !request)... |  | to do |
| `palette.hosting_open_issue` | palette | Integrations: Open Issue... |  | to do |
| `palette.hosting_review_pull_request` | palette | Hosting: Review Request... |  | to do |
| `palette.hosting_push_for_review` | palette | Hosting: Push for Review... |  | to do |
| `palette.hosting_edit_assignees` | palette | Hosting: Edit Assignees... |  | to do |
| `palette.hosting_edit_reviewers` | palette | Hosting: Edit Reviewers... |  | to do |
| `palette.hosting_edit_labels` | palette | Hosting: Edit Labels... |  | to do |
| `palette.hosting_set_milestone` | palette | Hosting: Set Milestone... |  | to do |
| `palette.hosting_close_issue` | palette | Local Issues: Close Issue... |  | to do |
| `palette.hosting_reopen_issue` | palette | Local Issues: Reopen Issue... |  | to do |
| `palette.hosting_open_in_obsidian` | palette | Local Issues: Open in Obsidian... |  | to do |
| `palette.hosting_open_board` | palette | Issues: Open Board |  | to do |
| `palette.hosting_create_obsidian_base` | palette | Local Issues: Create Obsidian Base |  | to do |
| `palette.hosting_create_pull_request` | palette | Pull Requests: Create Pull Request... | n | to do |
| `palette.hosting_create_issue` | palette | Hosting: New Issue... | n | to do |
| `palette.hosting_search_pull_requests` | palette | Pull Requests: Search |  | to do |
| `palette.hosting_choose_pull_request_filter` | palette | Pull Requests: Choose Filter (Open / Merged / All)... | v | to do |
| `palette.hosting_choose_pull_provider` | palette | Pull Requests: Choose Provider... |  | to do |
| `palette.hosting_choose_pull_repository` | palette | Pull Requests: Choose Repository... |  | to do |
| `palette.hosting_unlink_pull_source` | palette | Pull Requests: Unlink Source (None) |  | to do |
| `palette.hosting_choose_issue_provider` | palette | Issues: Choose Provider... |  | to do |
| `palette.hosting_unlink_issue_source` | palette | Issues: Unlink Source (None) |  | to do |
| `palette.hosting_unlink_profile` | palette | Integrations: Unlink Profile (Automatic) |  | to do |
| `palette.hosting_choose_issue_repository` | palette | Issues: Choose Repository... |  | to do |
| `palette.hosting_search_issues` | palette | Issues: Search |  | to do |
| `palette.hosting_choose_issue_filter` | palette | Issues: Choose Filter... | v | to do |
| `palette.hosting_edit_issue_query` | palette | Issues: Edit Query / Lists... |  | to do |
| `palette.hosting_insert_issue_ref` | palette | Commit: Insert Issue Reference... |  | to do |
| `palette.hosting_use_profile` | palette | Integrations: Use Profile for This Repository... |  | to do |
| `palette.hosting_integrations_preferences` | palette | Integrations: Manage This Repository's Profile |  | to do |
| `palette.hosting_connect_integration` | palette | Integrations: Connect Integration... |  | to do |
| `palette.hosting_radar_options` | palette | Settings: Radar Options of a Profile... |  | to do |
| `palette.trust_review` | palette | Integrations: Review security check |  | to do |
| `palette.theme` | palette | App: Theme... |  | done |
| `palette.conflict_lists` | palette | Conflicts: Edit Base Branch Lists... |  | to do |
| `palette.conflict_repo_page` | palette | Conflicts: Base Branches of This Repository... |  | to do |
| `palette.launcher_create` | palette | Launcher: Create Shortcut |  | to do |
| `palette.launcher_create_desktop` | palette | Launcher: Create Desktop Shortcut |  | to do |
| `palette.launcher_remove` | palette | Launcher: Remove Shortcuts |  | to do |
| `palette.launcher_alias_create` | palette | Launcher: Create Command Alias |  | to do |
| `palette.launcher_alias_remove` | palette | Launcher: Remove Command Alias |  | to do |
| `palette.launcher_path_add` | palette | Launcher: Add to PATH |  | to do |
| `palette.launcher_path_remove` | palette | Launcher: Remove from PATH |  | to do |
| `palette.launcher_folder_menu_install` | palette | Launcher: Install Open in GitPoulpi Menu |  | to do |
| `palette.launcher_folder_menu_remove` | palette | Launcher: Remove Open in GitPoulpi Menu |  | to do |
| `palette.launcher_become_primary` | palette | Launcher: Become the Primary Installation |  | to do |
| `palette.launcher_remove_legacy` | palette | Launcher: Remove Legacy Launchers |  | to do |
| `palette.launcher_forget_gone` | palette | Launcher: Forget Gone Installations |  | to do |
| `settings.*` | settings | Settings: <setting> |  | to do |
| `settings.ui.color_depth` | settings | Settings: ui color depth |  | terminal only: The colour depth of the terminal; a desktop window always has full colour. |
| `settings.ui.kitty_keyboard` | settings | Settings: ui kitty keyboard |  | terminal only: The kitty keyboard protocol is a way for a terminal to report keys. |
| `settings.ui.image_protocol` | settings | Settings: ui image protocol |  | terminal only: Which terminal graphics protocol draws inline images; a desktop window draws them itself. |
| `settings.ui.image_view_text_fallback` | settings | Settings: ui image view text fallback |  | terminal only: The text stand-in for images, for terminals that cannot draw them. |
| `settings.ui.text_cursor` | settings | Settings: ui text cursor |  | terminal only: The shape of the terminal's text cursor. |
| `settings.ui.mouse_enabled` | settings | Settings: ui mouse enabled |  | terminal only: Whether the terminal reports the mouse; a desktop window always has it. |
| `settings.ui.launch_terminal` | settings | Settings: ui launch terminal |  | terminal only: Which terminal the launcher starts the terminal UI in. |
| `settings.ui.relaunch_in_terminal` | settings | Settings: ui relaunch in terminal |  | terminal only: Restarting the terminal UI inside another terminal window. |
| `flow.start.*` | flow | Flow: Start <type> |  | to do |
| `flow.new_issue.*` | flow | Flow: New issue <type> |  | to do |
| `jobs.*` | jobs | Jobs: Cancel <job> |  | to do |
| `sequence.*` | sequence | Sequence: <name> |  | to do |
