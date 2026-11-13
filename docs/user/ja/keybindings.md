[English](../keybindings.md) · [Čeština](../cs/keybindings.md) · [Deutsch](../de/keybindings.md) · [Español](../es/keybindings.md) · [Français](../fr/keybindings.md) · [Italiano](../it/keybindings.md) · **日本語** · [한국어](../ko/keybindings.md) · [Nederlands](../nl/keybindings.md) · [Polski](../pl/keybindings.md) · [Português (Brasil)](../pt-BR/keybindings.md) · [Português (Portugal)](../pt-PT/keybindings.md) · [Русский](../ru/keybindings.md) · [Svenska](../sv/keybindings.md) · [Türkçe](../tr/keybindings.md) · [Українська](../uk/keybindings.md) · [简体中文](../zh-CN/keybindings.md)

# キーバインド

このページには**既定**のキーを載せています。ほとんどのキーは再割り当てできます。アプリは、有効になっているキーを常に表示します。コマンドパレット、`?` のヘルプ、ステータスバー、コンテキストメニュー、ボタンのツールチップに表示されます。キーを変更すると、これらの場所には新しいキーが表示されます。マウスについては、[マウスとジェスチャー](mouse-and-gestures.md)を参照してください。

## 再割り当て

**設定 > キーボードとマウス**を開きます。このページについては[設定](settings.md#キーボードとマウス)で説明しています。変更はすぐに反映され、`config.toml` の `[keybindings]` テーブルに保存されます。このテーブルは手で編集することもできます。

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # tmux and GNU screen often use Ctrl+B and Ctrl+A
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

構文、競合のルール、警告については、[設定リファレンス](config-reference.md)の `[keybindings]` にあります。要点は次のとおりです。

- 誤りはステータスバーに報告され、既定のキーはそのまま残ります。
- 同じコンテキストで 1 つのキーに 2 つのアクションを割り当てることはできません。
- プレフィックスには `Ctrl` または `Alt` が必要です。
- グローバルなキーを隠してしまうペインのキーは、警告が出るだけです。

下の表には、再割り当てできるすべてのキーを載せています。一部のキーは固定されていて、再割り当てできません。リストの移動（`j` `k` `h` `l`、矢印キー、`PageUp` / `PageDown`、`Home` / `End`）、`Enter`、`Esc`、`Tab`、数字（回数プレフィックス）、テキスト編集のキー、そしてポップアップの `Enter` / `Tab` / 矢印キーです。パレットには `アプリ: キー割り当て...` と `キー割り当て: すべてリセット...` があります。

## 既定のキーバインド

再割り当てできるすべてのキーを、`[keybindings.<context>]` で使う ID と既定値とともに載せています。この表はアプリから生成されています（テストで最新に保っています）。プレフィックスの後に続くキーは、プレフィックス付きで表示しています（`Ctrl+G s`）。

<!-- keymap:begin -->
| コンテキスト | アクション ID | 内容 | 既定のキー |
|---|---|---|---|
| `global` | `git_prefix` | Git 操作のプレフィックス | `Ctrl+G` |
| `global` | `nav_prefix` | ナビゲーションのプレフィックス | `Ctrl+N` |
| `global` | `palette` | コマンドパレット | `Ctrl+P` |
| `global` | `settings` | 設定 | `Ctrl+,` |
| `global` | `home` | ホーム（リポジトリ管理） | `Ctrl+O` |
| `global` | `clone` | リポジトリをクローン | `Ctrl+Shift+N` |
| `global` | `new_tab` | 新しいタブ | `Alt+T` |
| `global` | `workspace_switch` | ワークスペースを切り替え | `Alt+W` |
| `global` | `radar` | Radar を開く | `Alt+R` |
| `global` | `workspaces` | ワークスペースを開く | `Alt+J` |
| `global` | `tab_menu` | タブメニュー | `Ctrl+T` |
| `global` | `tab_open_in_popup` | 開く先（一覧） | `Alt+O` |
| `global` | `tab_open_in_default` | 既定の開く先で開く | `Ctrl+Alt+O` |
| `global` | `close_tab` | タブを閉じる | `Ctrl+W` |
| `global` | `next_tab` | 次のタブ | `Ctrl+PageDown` |
| `global` | `prev_tab` | 前のタブ | `Ctrl+PageUp` |
| `global` | `help` | ヘルプ | `?` |
| `global` | `quit` | 終了 | `q` |
| `global` | `refresh` | 更新 | `r` |
| `global` | `refresh_all` | 更新（fetch、pull request、Issue） | `F5` |
| `global` | `find_in_log` | Commitグラフを検索 | `f` |
| `global` | `workspace_search` | ワークスペースを検索 | `Ctrl+Alt+F` |
| `global` | `select_tab` | タブを選択 | `t` |
| `global` | `undo` | 直前の操作を元に戻す | `Ctrl+Z` |
| `global` | `redo` | 元に戻した操作をやり直す | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `global` | `undo_history` | 元に戻す履歴を開く | `Alt+Z` |
| `global` | `commit_column` | Commit列の表示/非表示 | `Alt+C` |
| `global` | `undo_offer` | ブランチをブロックされた操作が残した位置に復元 | `Alt+Shift+Z` |
| `global` | `context_menu` | コンテキストメニュー | `.`, `Menu`, `Shift+F10` |
| `git_prefix` | `compose_commit` | Commitメッセージを作成 | `Ctrl+G c` |
| `git_prefix` | `commit_now` | ステージ済みの変更を今すぐCommit | `Ctrl+G Enter` |
| `git_prefix` | `ai_compose` | AI：変更をcommitに分割 | `Ctrl+G Shift+C` |
| `git_prefix` | `ai_explain` | AI：変更を説明 | `Ctrl+G w` |
| `git_prefix` | `stash_message` | メッセージ付きでStash | `Ctrl+G m` |
| `git_prefix` | `ai_commit_message` | AI：Commitメッセージを作成 | `Ctrl+G i` |
| `git_prefix` | `stage_all` | すべての変更をステージ | `Ctrl+G a` |
| `git_prefix` | `discard_all` | すべての変更を破棄 | `Ctrl+G x` |
| `git_prefix` | `stage_file` | 選択中のファイルをステージ | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | 選択中のファイルをステージ解除 | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | 関連するPull Requestを開く | `Ctrl+G g` |
| `git_prefix` | `push_tags` | すべてのTagをPush | `Ctrl+G Shift+U` |
| `git_prefix` | `fetch_all` | すべてのリモートをFetch | `Ctrl+G f` |
| `git_prefix` | `new_tag` | 新規Tag | `Ctrl+G t` |
| `git_prefix` | `new_branch` | 新規ブランチ | `Ctrl+G b` |
| `git_prefix` | `stash_push` | 変更をStash | `Ctrl+G Shift+T` |
| `git_prefix` | `cherry_pick` | 選択中のCommitをcherry-pick | `Ctrl+G y` |
| `git_prefix` | `revert` | 選択中のCommitをリバート | `Ctrl+G v` |
| `git_prefix` | `edit_message` | Commitメッセージを編集 | `Ctrl+G e` |
| `git_prefix` | `amend` | 選択したCommitを修正 | `Ctrl+G Shift+E` |
| `git_prefix` | `edit_author` | 選択したCommitの作成者を修正 | `Ctrl+G Shift+A` |
| `git_prefix` | `reset_mixed` | 選択中のCommitにリセット（Mixed） | `Ctrl+G Shift+R` |
| `git_prefix` | `rebase_onto` | 選択中のブランチにRebase | `Ctrl+G r` |
| `git_prefix` | `stash_pop` | Stashを Pop | `Ctrl+G o` |
| `git_prefix` | `delete_ref` | 選択中の参照を削除、またはStashをDrop | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | LFS メニューを開く | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | Flow メニューを開く | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS: すべてのオブジェクトをFetch | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS: すべてのファイルをチェックアウト | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS: オブジェクトをPull | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS: すべてのオブジェクトをPush | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS: ローカルのオブジェクトを整理 | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | 履歴書き換えワークベンチを開く | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | Refs ペインの折りたたみ／復元 | `Ctrl+N h` |
| `nav_prefix` | `collapse_status` | Status ペインの折りたたみ／復元 | `Ctrl+N l` |
| `nav_prefix` | `settings` | 設定を開く | `Ctrl+N ,` |
| `nav_prefix` | `about` | バージョン情報を開く | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | アクティビティログを開く | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | グローバルアクティビティログを開く | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | バックグラウンドタスクを表示 | `Ctrl+N Shift+J` |
| `nav_prefix` | `updates` | 更新パネルを表示 | `Ctrl+N u` |
| `nav_prefix` | `help` | ヘルプの表示切り替え | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | タブを選択 | `Ctrl+N t` |
| `nav_prefix` | `change_theme` | テーマを変更 | `Ctrl+N y` |
| `nav_prefix` | `change_language` | 言語を変更 | `Ctrl+N g` |
| `nav_prefix` | `change_graph_color_theme` | グラフのカラーテーマを変更 | `Ctrl+N Shift+Y` |
| `nav_prefix` | `toggle_graph_row_tints` | グラフの行の色付けを切り替え | `Ctrl+N Shift+T` |
| `nav_prefix` | `grow_section` | Refs のセクションを拡大 | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | Refs のセクションを縮小 | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | Commitメッセージ欄を拡大 | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | Commitメッセージ欄を縮小 | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | Refs ペインを広げる | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | Refs ペインを狭める | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | Status ペインを広げる | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | Status ペインを狭める | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | エージェント：保留中のビューを表示 | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | エージェント：戻る（保留中のビューを閉じる） | `Ctrl+N Backspace` |
| `nav_prefix` | `arrange_dialog` | 開いているダイアログを調整（移動、サイズ変更） | `Ctrl+N m` |
| `refs` | `filter` | 名前で参照を絞り込み | `/` |
| `refs` | `solo` | 選択項目をソロ表示 | `o` |
| `refs` | `clear_solo` | ソロ表示を解除 | `Shift+O` |
| `refs` | `toggle_hierarchy` | ブランチの階層表示を切り替え | `y` |
| `refs` | `show_all` | 非表示の参照をすべて表示 | `Shift+H` |
| `refs` | `reveal` | グラフで表示 | `g` |
| `refs` | `mark_drop` | ドロップ元としてマーク | `m` |
| `refs` | `drop_marked` | マークした参照をここにドロップ | `Shift+M` |
| `refs` | `expand_all` | すべてのフォルダーを展開 | `=` |
| `refs` | `collapse_all` | すべてのフォルダーを折りたたむ | `-` |
| `refs` | `expand_below` | フォルダーとそのサブフォルダーを展開 | `*`, `Shift+Right` |
| `refs` | `collapse_below` | フォルダーとそのサブフォルダーを折りたたむ | `_`, `Shift+Left` |
| `log` | `toggle_file_mode` | ファイル：パスまたはツリー | `m` |
| `log` | `view_all_files` | ファイル：すべてのファイルを表示 | `a` |
| `log` | `compare_selection` | 選択した行を比較 | `v` |
| `log` | `compare_swap` | 比較の両端を入れ替え | `x` |
| `log` | `compare_next` | 次の比較対象の行へ移動 | `n` |
| `log` | `pick_column` | 移動・幅変更する列を選ぶ | `Shift+C` |
| `log` | `scroll_left` | 列を左へスクロール | `Shift+Left` |
| `log` | `scroll_right` | 列を右へスクロール | `Shift+Right` |
| `log` | `copy_mermaid` | Commitsを Mermaid gitGraph としてコピー | `Shift+Y` |
| `log_columns` | `next` | 次の列を選ぶ | `Tab` |
| `log_columns` | `prev` | 前の列を選ぶ | `Shift+BackTab` |
| `log_columns` | `move_left` | 列を左へ移動 | `<` |
| `log_columns` | `move_right` | 列を右へ移動 | `>` |
| `log_columns` | `wider` | 列を広げる | `]` |
| `log_columns` | `narrower` | 列を狭める | `[` |
| `log_columns` | `reset_width` | 列の幅をすべてリセット | `0` |
| `log_columns` | `fit` | 列を内容に合わせる | `=` |
| `log_columns` | `fit_all` | 全列を内容に合わせる | `+` |
| `log_columns` | `layout` | 横スクロール／ペインに合わせる | `s` |
| `log_files` | `collapse_all` | すべてのフォルダーを折りたたむ／展開 | `z` |
| `log_files` | `filter` | 変更されたファイルを絞り込み | `/` |
| `log_files` | `expand_all` | すべてのフォルダーを展開 | `=` |
| `log_files` | `collapse_all_folders` | すべてのフォルダーを折りたたむ | `-` |
| `log_files` | `expand_below` | フォルダーとそのサブフォルダーを展開 | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | フォルダーとそのサブフォルダーを折りたたむ | `_`, `Shift+Left` |
| `status` | `compose` | Commitメッセージ欄にフォーカス | `c` |
| `status` | `toggle_file_mode` | ファイル：パスまたはツリー | `m` |
| `status` | `view_all_files` | ファイル：すべてのファイルを表示 | `a` |
| `status` | `amend` | Amend の切り替え | `Shift+A` |
| `status` | `skip_hooks` | フックのスキップを切り替え | `Shift+H` |
| `status` | `collapse_all` | すべてのフォルダーを折りたたむ／展開 | `z` |
| `status` | `stage_toggle` | 選択行のステージ／ステージ解除 | `Ctrl+Space` |
| `status` | `open_diff` | 選択中のファイルのDiffを開く | `d` |
| `status` | `expand_all` | すべてのフォルダーを展開 | `=` |
| `status` | `collapse_all_folders` | すべてのフォルダーを折りたたむ | `-` |
| `status` | `expand_below` | フォルダーとそのサブフォルダーを展開 | `*`, `Shift+Right` |
| `status` | `collapse_below` | フォルダーとそのサブフォルダーを折りたたむ | `_`, `Shift+Left` |
| `home` | `favorite` | リポジトリのお気に入りを切り替え | `f`, `*` |
| `home` | `close_repo` | 行を閉じる、削除、またはお気に入り解除 | `x`, `Delete` |
| `home` | `browse` | リポジトリを参照 | `b` |
| `home` | `clone` | リポジトリをクローン | `c` |
| `home` | `init` | リポジトリを初期化 | `i` |
| `home` | `collapse_all` | すべてのセクションを折りたたむ | `-` |
| `home` | `expand_all` | すべてのセクションを展開 | `+`, `=` |
| `home` | `search` | リポジトリを検索 | `/` |
| `home` | `workspace_new` | 新しいワークスペース | `n` |
| `home` | `workspace_edit` | ワークスペースを編集 | `e` |
| `home` | `workspace_from_tabs` | 開いているタブから新規ワークスペース | `a` |
| `home` | `workspace_fetch` | ワークスペース内をすべてFetch | `Shift+F` |
| `home` | `workspace_pull` | ワークスペース内をすべてPull | `Shift+P` |
| `home` | `workspace_clone_missing` | 見つからないリポジトリをクローン | `m` |
| `home` | `workspace_locate` | 見つからないリポジトリの場所を指定 | `l` |
| `refs` | `hosting_filter` | Pull Request / Issue：絞り込みを選択 | `v` |
| `refs` | `hosting_edit` | Pull Request：ホストで編集 | `e` |
| `refs` | `new_pull_request` | Pull Request：新規作成 | `n` |
| `refs` | `add_remote` | リモート：リモートを追加 | `+` |
| `diff` | `close` | Diffを閉じる | `q` |
| `diff` | `edit` | 作業ツリーで編集 | `e` |
| `diff` | `lfs_raw` | LFS：カードまたは生のポインター | `r` |
| `diff` | `copy` | 選択範囲または行をコピー | `y`, `Ctrl+C` |
| `diff` | `stage_hunk` | ハンクのステージ／ステージ解除 | `Space` |
| `diff` | `stage_lines` | 選択行のステージ／ステージ解除 | `Enter` |
| `diff` | `discard_hunk` | ハンクを破棄 | `x` |
| `diff` | `discard_lines` | 選択行を破棄 | `Shift+X` |
| `diff` | `select_lines` | 行選択の開始／解除 | `v` |
| `diff` | `next_hunk` | 次のハンク | `n` |
| `diff` | `prev_hunk` | 前のハンク | `p` |
| `diff` | `toggle_layout` | レイアウト：インラインまたは分割 | `m` |
| `diff` | `toggle_hunks_only` | ハンクのみまたはファイル全体 | `Shift+H` |
| `diff` | `toggle_minimap` | ミニマップの切り替え | `Shift+M` |
| `diff` | `choose_encoding` | 文字コードを選択 | `Shift+C` |
| `diff` | `toggle_wrap` | 折り返しの切り替え | `w` |
| `diff` | `toggle_whitespace` | 空白のみの変更の表示／非表示 | `s` |
| `diff` | `merge_editor` | mergeエディターを開く（競合したファイル） | `Shift+E` |
| `file_insight` | `close` | ファイルビューを閉じる | `q` |
| `file_insight` | `edit` | 作業ツリーで編集 | `e` |
| `file_insight` | `copy` | 選択範囲または行をコピー | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | 最後の行へ移動 | `Shift+G` |
| `file_insight` | `open_commit` | その行のCommitへ移動 | `Enter` |
| `history` | `show_in_log` | Log で表示 | `Enter` |
| `history` | `copy_sha` | commitの SHA をコピー | `y` |
| `history` | `copy_summary` | 要約をコピー | `Shift+Y` |
| `history` | `show_diff` | このcommitのdiffを表示 | `d`, `Alt+1` |
| `history` | `open_file` | このcommit時点のファイルを開く | `o`, `Alt+2` |
| `history` | `blame` | このcommitで Blame | `b`, `Alt+3` |
| `history` | `compare_working_tree` | 作業ツリーと比較 | `c` |
| `history` | `restore` | このバージョンを復元 | `Shift+R` |
| `history` | `range_up` | 選択を上へ拡張 | `Shift+Up` |
| `history` | `range_down` | 選択を下へ拡張 | `Shift+Down` |
| `history` | `switch_focus` | リストとdiffを切り替え | `Tab` |
| `history` | `widen_list` | リストを広げる | `Alt+Right` |
| `history` | `narrow_list` | リストを狭める | `Alt+Left` |
| `history` | `reset_split` | リストの幅をリセット | `Alt+0` |
| `commit_column` | `scroll_left` | 列を左へスクロール | `Shift+Left` |
| `commit_column` | `scroll_right` | 列を右へスクロール | `Shift+Right` |
| `commit_column` | `next_part` | 次の欄の幅を調整 | `Tab` |
| `commit_column` | `move_left` | 欄を左へ移動 | `<` |
| `commit_column` | `move_right` | 欄を右へ移動 | `>` |
| `commit_column` | `wider_part` | 欄を広げる | `]` |
| `commit_column` | `narrower_part` | 欄を狭める | `[` |
| `commit_column` | `reset_widths` | 各欄の幅をリセット | `0` |
| `commit_column` | `fit` | 列を内容に合わせる | `=` |
| `commit_column` | `fit_all` | 全列を内容に合わせる | `+` |
| `file_insight` | `blame_card` | Blame: 詳細カードを表示 / 非表示 | `i` |
| `file_insight` | `blame_next` | Blame: 次のハンク | `n` |
| `file_insight` | `blame_prev` | Blame: 前のハンク | `p` |
| `file_insight` | `blame_next_same` | Blame: 同じcommitの次のハンク | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame: 同じcommitの前のハンク | `Shift+P` |
| `file_insight` | `blame_goto` | Blame: 行へ移動 | `g` |
| `file_insight` | `blame_copy_sha` | Blame: commitの SHA をコピー | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame: このcommitのこのファイルのdiffを表示 | `d` |
| `file_insight` | `blame_parent` | Blame: このcommitの親で Blame | `b` |
| `file_insight` | `blame_history` | Blame: ファイル履歴を開く | `Shift+H` |
| `file_insight` | `blame_wider` | Blame: 注釈列を広げる | `>` |
| `file_insight` | `blame_narrower` | Blame: 注釈列を狭める | `<` |
| `image` | `zoom_in` | 画像を拡大 | `+`, `=` |
| `image` | `zoom_out` | 画像を縮小 | `-` |
| `image` | `fit` | 画像をビューに合わせる | `0` |
| `image` | `actual_size` | 画像を実寸で表示 | `1` |
| `image` | `flip_side` | 反対側を表示（変更前 / 変更後） | `Tab` |
| `image` | `widen_before` | 変更前ペインを広げる | `Alt+Right` |
| `image` | `widen_after` | 変更後ペインを広げる | `Alt+Left` |
| `image` | `reset_split` | 変更前 / 変更後の分割をリセット | `Alt+0` |
| `image` | `edit_in` | アプリで開く... | `Shift+O` |
| `image` | `edit_default` | 既定のアプリで開く | `o` |
| `editor` | `save` | 保存 | `Ctrl+S` |
| `editor` | `undo` | 元に戻す | `Ctrl+Z` |
| `editor` | `redo` | やり直す | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | 検索 | `Ctrl+F` |
| `editor` | `find_next` | 次を検索（Shift：前を検索） | `F3` |
| `editor` | `paste` | 貼り付け | `Ctrl+V` |
| `editor` | `toggle_minimap` | ミニマップの切り替え | `Alt+M` |
| `editor_prompt` | `save` | 未保存の変更：保存して閉じる | `s`, `Shift+S` |
| `editor_prompt` | `discard` | 未保存の変更：破棄して閉じる | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | 未保存の変更：編集を続ける | `c`, `Shift+C` |
| `merge` | `next_conflict` | 次の競合 | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | 前の競合 | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | 次の競合ファイル | `Alt+PageDown` |
| `merge` | `prev_file` | 前の競合ファイル | `Alt+PageUp` |
| `merge` | `next_pane` | 次のペインにフォーカス | `F6` |
| `merge` | `prev_pane` | 前のペインにフォーカス | `Shift+F6` |
| `merge` | `toggle_panes` | サイドパネル（Refs と Status）の表示／非表示 | `F9` |
| `merge` | `toggle_base` | ベースペインの表示／非表示 | `Alt+D` |
| `merge` | `toggle_full_file` | 全体のdiff表示を切り替え | `Alt+F` |
| `merge` | `undo` | 元に戻す（選択と入力） | `Ctrl+Z` |
| `merge` | `redo` | やり直す（選択と入力） | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | 現在（A）ペインを広げる | `Alt+Right` |
| `merge` | `widen_incoming` | 取り込み（B）ペインを広げる | `Alt+Left` |
| `merge` | `reset_panes` | ペインのサイズをリセット | `Alt+0` |
| `merge` | `save` | 解決済みにせず保存 | `Alt+S` |
| `merge` | `save_resolve` | 保存して解決済みにする | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ 解決案を提案（AI） | `Alt+I` |
| `merge` | `unpick` | この競合の選択を解除（未解決に戻す） | `Alt+U` |
| `merge` | `reset_all` | ファイル全体を元の競合にリセット | `Alt+R` |
| `merge` | `focus_a` | 現在（A）ペインにフォーカス | `Alt+1` |
| `merge` | `focus_b` | 取り込み（B）ペインにフォーカス | `Alt+2` |
| `merge` | `focus_output` | 出力ペインにフォーカス | `Alt+3` |
| `merge` | `focus_base` | ベースペインにフォーカス | `Alt+4` |
| `merge_sides` | `toggle_line` | 行の採用／解除 | `Space` |
| `merge_sides` | `toggle_chunk` | チャンク全体の採用／解除 | `Enter` |
| `merge_sides` | `take_a` | 現在（A）のみ採用 | `a` |
| `merge_sides` | `take_b` | 取り込み（B）のみ採用 | `b` |
| `merge_sides` | `take_both` | 両方採用（A、次に B） | `c` |
| `merge_sides` | `take_both_reversed` | 両方採用（B、次に A） | `Shift+C` |
| `merge_sides` | `clear` | この競合は何も採用しない | `x` |
| `merge_sides` | `unpick` | この競合の選択を解除（未解決に戻す） | `u` |
| `merge_sides` | `reset_all` | ファイル全体を元の競合にリセット | `Shift+U` |
| `merge_sides` | `swap_order` | 両側の順序を入れ替え | `s` |
| `merge_sides` | `next_conflict` | 次の競合 | `n` |
| `merge_sides` | `prev_conflict` | 前の競合 | `Shift+N` |
| `merge_sides` | `take_all_a` | 現在（A）をすべて採用 | `Shift+A` |
| `merge_sides` | `take_all_b` | 取り込み（B）をすべて採用 | `Shift+B` |
| `merge_sides` | `toggle_base` | ベースペインの表示／非表示 | `d` |
| `merge_sides` | `scroll_left` | ペインを左へスクロール | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | ペインを右へスクロール | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | 全体のdiff表示を切り替え | `f` |
| `merge_sides` | `cycle_height` | 出力の高さを切り替え | `z` |
| `merge_card` | `take_a` | A（現在）をそのまま残す | `a` |
| `merge_card` | `take_b` | B（取り込み）をそのまま採用 | `b` |
| `merge_card` | `take_base` | ベース（共通の祖先）を採用 | `o` |
| `merge_card` | `keep` | 残っているファイルを残す | `k` |
| `merge_card` | `delete` | 削除を受け入れる | `Shift+D` |
| `merge_card` | `external_tool` | 外部mergeツールで開く | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit（停止して Amend） | `e` |
| `rebase` | `autosquash` | Autosquash の切り替え | `a` |
| `rebase` | `start` | プランを開始 | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | プランをリセット | `Ctrl+R` |
| `rebase` | `split` | 選択行のcommitを分割 | `t` |
| `rebase` | `move_up` | 行を上へ移動 | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | 行を下へ移動 | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | Commitを作成 | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | 行単位の選択を切り替え | `v` |
| `split` | `move_up` | commitを上へ移動 | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | commitを下へ移動 | `Alt+Down`, `Shift+J` |
| `split` | `reset` | 分割をリセット | `Shift+R` |
| `rewrite` | `preview` | 書き換えをプレビュー | `F5` |
| `rewrite` | `apply` | 書き換えを適用 | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | プレビューを中止 | `Ctrl+X` |
| `rewrite` | `add_rule` | ルールを追加 | `a`, `Insert` |
| `rewrite` | `toggle_rule` | ルールのオン/オフを切り替え | `Space` |
| `rewrite` | `move_up` | ルールを上へ移動 | `Alt+Up` |
| `rewrite` | `move_down` | ルールを下へ移動 | `Alt+Down` |
| `rewrite` | `delete_rule` | ルールを削除 | `d`, `Delete` |
| `rewrite` | `restore_rule` | 削除したルールを復元 | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | ルールを複製 | `Ctrl+D` |
| `rewrite` | `next_tab` | 次のプレビュータブ | `Alt+Right` |
| `rewrite` | `prev_tab` | 前のプレビュータブ | `Alt+Left` |
| `rewrite` | `save_shared` | レシピをリポジトリに保存 | `Alt+S` |
| `rewrite` | `save_mine` | レシピを自分用に保存 | `Alt+M` |
| `rewrite` | `load` | レシピを読み込む | `Alt+L` |
| `rewrite` | `name` | レシピ名を変更 | `F2` |
| `rewrite` | `protected_all` | 保護ブランチをすべて含める | `Alt+A` |
| `rewrite` | `protected_none` | 保護ブランチをすべて除外 | `Alt+U` |
| `rewrite` | `check_tools` | ツールを再確認 | `F6` |
| `rewrite` | `analyze` | 履歴のサイズを分析 | `F7` |
| `rewrite` | `host_howto` | ホストのブランチ保護について説明 | `Alt+H` |
| `rewrite` | `ai_rules` | AI で書き換えルールを作成 | `Alt+I` |
| `reword` | `apply` | 新しいメッセージを適用 | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | 履歴をさらに深く変更 | `Alt+H` |
| `composer` | `commit` | コンポーザーからCommit | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | Issue の参照を挿入 | `Alt+R` |
| `dialog` | `browse` | フォルダーを参照 | `Ctrl+B` |
| `dialog` | `sources` | クローン元へ移動 | `Alt+S` |
| `dialog` | `submit` | Pull Requestフォームを送信 | `Ctrl+S` |
| `dialog` | `web` | プロバイダーの Web フォームを開く | `Alt+W` |
| `folder_picker` | `choose` | フォルダーを選択 | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | 新規フォルダー | `Alt+N` |
| `folder_picker` | `filter` | フォルダー内の項目を絞り込み | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | 兄弟フォルダーの表示／非表示 | `Alt+S` |
| `folder_picker` | `toggle_hidden` | 隠しフォルダーの表示／非表示 | `Alt+H` |
| `preferences` | `search` | 設定を検索 | `Ctrl+F`, `/` |
| `preferences` | `list_add` | リスト項目を追加 | `Insert` |
| `preferences` | `list_duplicate` | 選択したリスト項目を複製 | `Ctrl+D` |
| `preferences` | `list_remove` | 選択したリスト項目を削除 | `Delete` |
| `preferences` | `list_move_up` | 選択したリスト項目を上へ移動 | `Alt+Up` |
| `preferences` | `list_move_down` | 選択したリスト項目を下へ移動 | `Alt+Down` |
| `preferences` | `prev_tab` | 設定ページの前のタブ | `Alt+Left` |
| `preferences` | `next_tab` | 設定ページの次のタブ | `Alt+Right` |
| `ai_popup` | `copy` | 結果をコピー | `c` |
| `activity_log` | `scroll_left` | アクティビティログを左へスクロール | `Left` |
| `activity_log` | `scroll_right` | アクティビティログを右へスクロール | `Right` |
| `integrations` | `add` | プロファイルを追加 | `a` |
| `integrations` | `edit` | 選択中のプロファイルを編集 | `e` |
| `integrations` | `open_token` | トークンのページを開く | `o` |
| `integrations` | `remove` | 選択中のプロファイルを削除 | `d`, `Delete` |
| `prompt` | `ai_stash_message` | AI：Stashメッセージを生成 | `Ctrl+G` |
<!-- keymap:end -->

### ターミナルが送れないキーと kitty キーボードプロトコル

古いターミナルは、一部のキーの組み合わせを送れません。`Ctrl+,`、`Ctrl+Shift+N`、`Ctrl+Enter`、`Ctrl+<数字>` です。何も送られないか、別のキーとして届きます。たとえば、`Ctrl+Shift+N` は `Ctrl+N` として、`Ctrl+I` は `Tab` として、`Ctrl+M` は `Enter` として届きます。設定ページは、このような割り当てについて警告します（「このターミナルは ... を送信しません」）。

**kitty キーボードプロトコル**に対応したターミナルは、これらのキーの組み合わせを送れます。例：kitty、WezTerm、Ghostty、foot、Alacritty 0.13 以降、そして zellij です（tmux は応答しません：[マルチプレクサ](multiplexers.md#tmux)を参照）。GitPoulpi は既定でこのプロトコルを使います。

- 起動時に、GitPoulpi はターミナルに問い合わせ、返事を最大でおよそ 100 ms 待ちます。ターミナルが返事をしない場合は、従来のキーのままです。
- ターミナルが返事をすると、GitPoulpi はプロトコルをオンにします。終了時、失敗時、クラッシュ時には、再びオフにします。
- 設定ページの警告はこのプロトコルに従います。オンのときは、上記のキーの組み合わせは送信可能です。
- **設定 > キーボードとマウス > Kitty キーボードプロトコル**（`ui.kitty_keyboard`）を `off` にすると、ターミナルのキーボードモードを一切変更しません。ターミナルやマルチプレクサの動作がおかしいときに使ってください。再起動が必要です。

Windows のコンソールは修飾キーを自分で報告するので、このプロトコルは不要です。

## コマンドパレット（`Ctrl+P`）

パレットにはアプリの**すべての**コマンドが載っているので、ショートカットなしですべてに手が届きます。完全な一覧は[下](#パレットのすべてのコマンド)にあります。

| キー | 動作 |
|---|---|
| `Ctrl+P` | パレットを開く |
| *（入力）* | 一覧を絞り込む。`カテゴリ: ` プレフィックスの後の完全一致が最初に、次にラベルのどこかに一致するもの、その次にあいまい一致が来ます |
| `Up` / `Down`、`Home` / `End`、`PageUp` / `PageDown` | ハイライトを移動する（[絞り込みできるポップアップ一覧](#絞り込みできるポップアップ一覧)を参照） |
| `Enter`、またはダブルクリック | ハイライトされたコマンドを実行する |
| `Esc` | 閉じる |

`j` と `k` はフィルターに入力されます。ハイライトは移動しません。

各行の右側には、そのショートカットが表示されます。今は当てはまらないコマンド（たとえば、参照が選択されていない場合）は、一覧に残ったままグレー表示になり、その理由が一覧の下に表示されます。ラベルは、`ナビゲーション:`、`Refs:`、`Git:`、`表示:`、`Diff:`、`アプリ:` のようなカテゴリで始まります。カテゴリを入力すると、そのグループ全体が表示されます。

### どのコマンドにもキーを割り当てる

パレットの各行の末尾にはキーボタン（`[k]`）があり、強調表示された行で `Alt+K` を押しても同じです。そのコマンドのショートカットで絞り込んだ **設定 > キーボードとマウス** が開き、すぐに割り当て直せます。専用のショートカットがないコマンドもここで割り当てられます。**パレットのコマンド** に並び、既定では未割り当てで、割り当てたキーはパレットで選んだときと同じようにコマンドを実行します（今は実行できないコマンドは理由を表示します）。行にはそのキーが表示されます。

`config.toml` では `[keybindings.palette]` テーブルで、コマンド名で指定します（`toggle_minimap = "Ctrl+Alt+M"`）。他の操作と衝突するキーは赤で表示され、その行で `c` を押すと衝突するすべての操作が一覧されます。

リストの下のヒント欄にこのキー（既定は `Alt+K`）が表示されます。これは操作 `palette.configure_shortcut` なので、他の操作と同じように割り当て直せます。

パレットとピッカー（テーマ、言語、タブ、ワークスペース検索）は、ダイアログと同じように移動とサイズ変更ができます。上の枠をドラッグするか、（すべてのダイアログと同じく）`Ctrl+N` に続けて `m` を押すか、`Alt+G` を押します。`Alt+G` は操作 `palette.arrange`（「ダイアログ: 手前のダイアログを移動 / サイズ変更」）で、すべてのダイアログとポップアップの上で使える追加の方法です。手前にあるものの調整モードを始めます（パレットの `ダイアログ: 移動 / サイズ変更` 行と `ダイアログ: 位置とサイズをリセット` 行は、開いているパレット自身に作用します）。矢印で移動、`Shift`+矢印でサイズ変更、`0` でリセットし、`Enter` か `Esc` でこのモードを終了します（ポップアップは開いたままです）。位置を保つかどうかは `ui.dialog_geometry` が決めます。

ここでも `Ctrl+N` はナビゲーションのプレフィックスです。フィルターボックス、一覧の最初の行、閉じる ✕ にバッジが表示され、続けて押したキーでそのコントロールが押されるか、`m` でポップアップの調整が始まります。ハイライトを下へ移動することはなくなり、`Ctrl+P` も上へ移動しません。フィルター欄のある一覧を移動するのは、矢印、`PageUp` / `PageDown`、`Home` / `End` だけです。

### パレットのすべてのコマンド

アプリから生成されています（テストで最新に保っています）。AI のコマンドは AI がオンのときだけ表示されます。ほかのコマンドは、当てはまらないときは理由付きでグレー表示になります。キーは既定のものです。

<!-- palette:begin -->
| コマンド | 既定のキー |
|---|---|
| ナビゲーション: Refs ペインにフォーカス | `Ctrl+N <badge>` |
| ナビゲーション: ログペインにフォーカス | `Ctrl+N <badge>` |
| ナビゲーション: Status ペインにフォーカス | `Ctrl+N <badge>` |
| ナビゲーション: Refs フィルターにフォーカス | `Ctrl+N <badge>` |
| ナビゲーション: Refs > ローカルにフォーカス | `Ctrl+N <badge>` |
| ナビゲーション: Refs > リモートにフォーカス | `Ctrl+N <badge>` |
| ナビゲーション: Refs > Worktrees にフォーカス | `Ctrl+N <badge>` |
| ナビゲーション: Refs > サブモジュールにフォーカス | `Ctrl+N <badge>` |
| ナビゲーション: Refs > Stashesにフォーカス | `Ctrl+N <badge>` |
| ナビゲーション: Refs > Tagsにフォーカス | `Ctrl+N <badge>` |
| サイドバー: セクションの表示/非表示... |  |
| サイドバー: セクションを既定の状態に戻す |  |
| サイドバー: このセクション設定をすべてのリポジトリに使う |  |
| サイドバー: ローカルブランチの表示/非表示 |  |
| サイドバー: リモートの表示/非表示 |  |
| サイドバー: worktree の表示/非表示 |  |
| サイドバー: Stashesの表示/非表示 |  |
| サイドバー: Tagsの表示/非表示 |  |
| サイドバー: Pull requestの表示/非表示 |  |
| サイドバー: Issue の表示/非表示 |  |
| サイドバー: サブモジュールの表示/非表示 |  |
| ナビゲーション: Status > 未ステージにフォーカス | `Ctrl+N <badge>` |
| ナビゲーション: Status > ステージ済みにフォーカス | `Ctrl+N <badge>` |
| ナビゲーション: 次のペインにフォーカスを移動 | `Tab` |
| ナビゲーション: 前のペインにフォーカスを移動 | `Shift+Tab` |
| ナビゲーション: Commitを検索... | `f` |
| ナビゲーション: Refs ペインの折りたたみを切り替え | `Ctrl+N h` |
| ナビゲーション: Refs ペインを広げる | `Ctrl+N Right` |
| ナビゲーション: Refs ペインを狭める | `Ctrl+N Left` |
| ナビゲーション: Status ペインを広げる | `Ctrl+N Shift+Left` |
| ナビゲーション: Status ペインを狭める | `Ctrl+N Shift+Right` |
| ナビゲーション: Status ペインの折りたたみを切り替え | `Ctrl+N l` |
| ナビゲーション: Refs のセクションを拡大 | `Ctrl+N +` |
| ナビゲーション: Refs のセクションを縮小 | `Ctrl+N -` |
| ナビゲーション: Refs のセクションサイズをリセット |  |
| ナビゲーション: レイアウトをリセット |  |
| Commit詳細: メッセージを拡大 | `Ctrl+N ]` |
| Commit詳細: メッセージを縮小 | `Ctrl+N [` |
| Commit詳細: メッセージのサイズをリセット |  |
| Commit詳細: メッセージを下へスクロール |  |
| Commit詳細: メッセージを上へスクロール |  |
| Commit詳細: メッセージ内のリンクを開く... |  |
| ファイル: Commitのファイル並び順を切り替え（A-Z / Z-A） |  |
| Refs: 選択中の参照をチェックアウト | `ダブルクリック` |
| Refs: 参照名をコピー |  |
| Refs: 完全な参照をコピー（refs/heads/...） |  |
| Refs: 参照の対象 SHA をコピー |  |
| Refs: 選択中のブランチを現在のブランチにMerge |  |
| Refs: 現在のブランチを選択中のものにRebase | `Ctrl+G r` |
| Refs: 選択中の参照をチェックアウト中のブランチにドロップ... |  |
| Refs: ドロップ元としてマーク | `m` |
| Refs: マークした参照をここにドロップ... | `M` |
| Refs: 選択中のブランチの名前を変更... |  |
| Refs: 選択中のブランチを現在のものに早送り |  |
| Refs: 現在のブランチを選択中のものに早送り |  |
| Refs: 選択中のブランチまたはTagから worktree を作成... |  |
| Refs: 選択中のブランチの Reflog... |  |
| Refs: 選択中のブランチからPull Requestを開始... |  |
| Refs: Worktree をタブで開く | `Enter / ダブルクリック` |
| Refs: サブモジュールを更新（選択中） |  |
| Refs: サブモジュールを初期化（選択中） |  |
| Refs: サブモジュールをタブで開く | `Enter / ダブルクリック` |
| Git: すべてのサブモジュールを更新 |  |
| Git: 選択中のCommitに worktree を作成... |  |
| Git: 選択中のCommitからパッチを作成... |  |
| Git: 選択中のCommitから HEAD までパッチを作成... |  |
| Git: 作業ツリーの変更からパッチを作成... |  |
| 書き換え: 古い履歴を消去... |  |
| Git: パッチを適用... |  |
| 書き換え: 履歴ワークベンチ | `Ctrl+G H` |
| 書き換え: 作成者を置き換え... |  |
| 書き換え: 作成者を匿名化... |  |
| 書き換え: 履歴に .mailmap を適用... |  |
| 書き換え: 名義をmerge（.mailmap）... |  |
| 書き換え: Commitの日付をずらす... |  |
| 書き換え: Commitのタイムゾーンを変更... |  |
| 書き換え: Commitsを時間帯内に収める... |  |
| 書き換え: コミッターの日付を作成者の日付に合わせる |  |
| 書き換え: メッセージ内を検索して置換... |  |
| 書き換え: メッセージの行を削除... |  |
| 書き換え: トレーラーを追加/削除... |  |
| 書き換え: 署名を削除... |  |
| 書き換え: メッセージのエンコーディングを修正... |  |
| 書き換え: ファイル内を検索して置換... |  |
| 書き換え: 履歴からファイルを削除... |  |
| 書き換え: 履歴から大きなファイルを削除... |  |
| 書き換え: パスを移動/名前変更... |  |
| 書き換え: ファイルを LFS に移行... |  |
| 書き換え: LFS からファイルを戻す... |  |
| 書き換え: 改行コードを変換... |  |
| 書き換え: 実行ビットを削除... |  |
| 書き換え: ファイルのエンコーディングを変換... |  |
| 書き換え: スクリプトを実行... |  |
| ✦ 書き換え: AI でスクリプトを作成... |  |
| ✦ 書き換え: AI でルールを作成... |  |
| 書き換え: 選択中のCommitから HEAD まで... |  |
| 書き換え: 選択中のCommits... |  |
| LFS: このリポジトリで初期化 |  |
| LFS: パターンを追跡... |  |
| LFS: パターンの追跡を解除... |  |
| LFS: オブジェクトをFetch |  |
| LFS: メニューを開く | `Ctrl+G l` |
| LFS: すべてのオブジェクトをFetch | `Ctrl+G F` |
| LFS: すべてのファイルをチェックアウト | `Ctrl+G K` |
| LFS: すべてのオブジェクトをPush | `Ctrl+G J` |
| LFS: オブジェクトをPull | `Ctrl+G L` |
| LFS: オブジェクトをPush |  |
| LFS: ローカルのオブジェクトを整理 | `Ctrl+G V` |
| LFS: ロックを表示 |  |
| LFS: インストールページを開く |  |
| LFS: diffで生のポインターを切り替え | `r` |
| LFS: 選択中のファイルを追跡 |  |
| LFS: 選択中のファイルの拡張子を追跡 |  |
| LFS: 選択中のファイルをロック |  |
| LFS: 選択中のファイルのロックを解除 |  |
| LFS: 選択中のファイルのロックを強制解除 |  |
| Refs: 選択中のブランチまたはTagを削除... | `Ctrl+G d` |
| Refs: 選択中のブランチをリモートから削除... |  |
| Refs: 選択中のブランチをローカルとリモートの両方で削除... |  |
| Refs: リモートの選択中のブランチへのリンクをコピー |  |
| Git: リモートの選択中のCommitへのリンクをコピー |  |
| Git: 選択中のCommitに注釈付きTagを作成... |  |
| Refs: 選択中のTagを既定のリモートへPush |  |
| Refs: 選択中のTagをリモートから削除... |  |
| Git: 選択中のtagをfetch |  |
| Refs: 選択中のTagに注釈を付ける... |  |
| Refs: グラフでの非表示を切り替え |  |
| Refs: グラフで表示 | `g` |
| Refs: 選択中の参照のソロ表示を切り替え | `o` |
| Refs: ソロ表示を解除（ソロ表示中の参照をすべて解除） | `O` |
| Refs: 非表示の参照をすべて表示 | `H` |
| Refs: 選択中のフォルダー/リモートの非表示を切り替え |  |
| Refs: 選択中のフォルダー/リモートのソロ表示を切り替え | `o` |
| Refs: 名前で絞り込み... | `/` |
| Refs: 名前の絞り込みを解除 | `Esc` |
| Refs: すべてのフォルダーを展開 | `=` |
| Refs: すべてのフォルダーを折りたたむ | `-` |
| Refs: 選択中のフォルダーとそのサブフォルダーを展開 | `*` |
| Refs: 選択中のフォルダーとそのサブフォルダーを折りたたむ | `_` |
| Refs: 次のセクション | `l` |
| Refs: 前のセクション | `h` |
| Refs: 現在のセクションを展開／折りたたみ | `Enter` |
| Refs: ブランチの階層表示を切り替え | `y` |
| Git: 選択中のファイルをステージ | `Ctrl+G s` |
| Git: 選択中のファイルをステージ解除 | `Ctrl+G u` |
| Git: すべての変更をステージ | `Ctrl+G a` |
| 作業中の変更: 選択中のファイルを破棄... |  |
| 作業中の変更: 選択中のファイルを無視 |  |
| 作業中の変更: 選択中のファイルの拡張子を無視 |  |
| 作業中の変更: 選択中のファイルをバイナリとして扱う... |  |
| 作業中の変更: 選択中のファイルをテキストとして扱う... |  |
| 作業中の変更: 選択中のファイルの拡張子をバイナリとして扱う... |  |
| 作業中の変更: 選択中のファイルの拡張子をテキストとして扱う... |  |
| 作業中の変更: 選択中のファイルをStash |  |
| 作業中の変更: 選択中のファイルの履歴 |  |
| 作業中の変更: 選択中のファイルの Blame |  |
| 作業中の変更: 選択中のファイルを外部diffツールで開く |  |
| 作業中の変更: 選択中のファイルを外部エディターで開く |  |
| 作業中の変更: 選択中のファイルを既定のプログラムで開く |  |
| 作業中の変更: 選択中のファイルをフォルダーで表示 |  |
| 作業中の変更: 選択中のファイルのパスをコピー |  |
| 作業中の変更: 選択中のファイルからパッチを作成... |  |
| 作業中の変更: 選択中のファイルを編集 |  |
| 作業中の変更: 選択中のファイルを削除... |  |
| 作業中の変更: 選択中のフォルダーのメニューを開く |  |
| 作業中の変更: ファイルを作成 |  |
| 作業中の変更: すべての変更を破棄... | `Ctrl+G x` |
| Git: Commit... | `Ctrl+G c` |
| Git: ステージ済みの変更を今すぐCommit | `Ctrl+G Enter` |
| Git: WIP のcommitメッセージを編集（ログ内） | `ログ内で c` |
| Git: すべてのリモートをFetch | `Ctrl+G f` |
| Git: Pull | `Ctrl+G p` |
| Git: Pull（可能なら早送り） |  |
| Git: Pull（早送りのみ） |  |
| Git: Pull（rebase） |  |
| Git: Pullのオプション... |  |
| Git: Push | `Ctrl+G P` |
| Git: リモートへPush... |  |
| Git: 強制Push（リース付き）... |  |
| Git: アップストリームを設定... |  |
| リモート: リモートを追加... |  |
| リモート: リモートを削除... |  |
| リモート: リモートを編集... |  |
| リモート: 既定のリモートを設定... |  |
| リモート: リモート名をコピー |  |
| リモート: リモートへのリンクをコピー |  |
| リモート: リモートをFetch... |  |
| リモート: 選択中のリモートの操作... | `.` |
| リポジトリ: スパースチェックアウト... |  |
| リポジトリ: SSH と認証情報... |  |
| リポジトリ: 接続テスト |  |
| リポジトリ: 既定の認証情報を使う |  |
| Flow: メニューを開く | `Ctrl+G W` |
| Flow: ブランチを開始... |  |
| Flow: 新しい Issue... |  |
| Flow: ホストのブランチを引き継ぐ... |  |
| Flow: 現在のブランチを完了... |  |
| Flow: 親と同期... |  |
| Flow: 昇格... |  |
| Flow: 親を設定... |  |
| Flow: 停止した操作を続行または中止... |  |
| Flow: ブランチのグループ表示を切り替え |  |
| Flow: ワークフロー設定... |  |
| リモート: リモートをグラフで非表示... |  |
| リモート: リモートをグラフに表示... |  |
| Git: HEAD に新規ブランチ... | `Ctrl+G b` |
| Git: HEAD に新規Tag... | `Ctrl+G t` |
| Git: 未pushのTagをすべてPush | `Ctrl+G U` |
| Git: すべての変更をStash | `Ctrl+G T` |
| Git: メッセージ付きですべての変更をStash... | `Ctrl+G m` |
| Git: Stashを Pop | `Ctrl+G o` |
| Git: Stashを Apply |  |
| Git: StashをDrop... |  |
| Git: 選択中のCommitを HEAD にcherry-pick | `Ctrl+G y` |
| Git: 選択中のCommitをチェックアウト（HEAD を切り離し） |  |
| Git: 選択中のCommitにブランチを作成... |  |
| Git: 選択中のCommitにTagを作成... |  |
| Commit: メッセージをコピー |  |
| Commit: 作成者名をコピー |  |
| Commit: 作成者のメールアドレスをコピー |  |
| Commit: 日付をコピー |  |
| Commit: 短い SHA をコピー |  |
| Commit: 完全な SHA をコピー |  |
| Git: 選択中のCommitをリバート... | `Ctrl+G v` |
| Git: Commitメッセージを編集... | `Ctrl+G e` |
| Commit: 修正... | `Ctrl+G E` |
| Commit: 作成者を修正... | `Ctrl+G A` |
| Commit: 概要を修正... |  |
| Commit: 説明を修正... |  |
| Git: 選択中のCommitを分割... |  |
| Git: 選択中のCommitからインタラクティブにRebase... |  |
| Rebase: 続行 |  |
| Rebase: スキップ |  |
| Rebase: 中止 |  |
| git am: 続行 |  |
| git am: このパッチをスキップ |  |
| git am: 中止 |  |
| リポジトリ: 古い index.lock を削除 |  |
| バナー: 保留中の自動stashを Pop |  |
| バナー: 保留中の自動stashを完了（結果を保持） |  |
| バナー: 保留中の自動stashを表示 |  |
| 競合: 予測された競合を表示 |  |
| バナー: 保留中の自動stashをDrop... |  |
| バナー: やり直す... |  |
| Git: ブランチを選択中のCommitにリセット（soft - インデックスと作業ツリーを保持） |  |
| Git: ブランチを選択中のCommitにリセット（mixed - 作業ツリーを保持） | `Ctrl+G R` |
| Git: ブランチを選択中のCommitにリセット（hard - すべて破棄）... |  |
| Git: Ours（このブランチ）で競合を解決 |  |
| Git: Theirs（取り込み側）で競合を解決 |  |
| Git: ベース（共通の祖先）で競合を解決 |  |
| 競合: mergeエディターを開く | `競合ファイルで Enter` |
| Merge: 次の競合 | `F7` |
| Merge: 前の競合 | `Shift+F7` |
| Merge: 次の競合ファイル | `Alt+PageDown` |
| Merge: 前の競合ファイル | `Alt+PageUp` |
| Merge: A からチャンクを採用 |  |
| Merge: B からチャンクを採用 |  |
| Merge: この競合は現在を採用 | `a` |
| Merge: この競合は取り込みを採用 | `b` |
| Merge: 両方採用（A、次に B） | `c` |
| Merge: 両方採用（B、次に A） | `C` |
| Merge: この競合はベースを採用 |  |
| Merge: この競合をクリア | `x` |
| Merge: 順序を入れ替え | `s` |
| Merge: 現在をすべて採用 | `A` |
| Merge: 取り込みをすべて採用 | `B` |
| Merge: この競合を出力で編集 |  |
| Merge: サイドパネルの表示／非表示 | `F9` |
| Merge: ペインを左へスクロール | `Shift+Left` |
| Merge: ペインを右へスクロール | `Shift+Right` |
| Merge: ベースペインを切り替え | `Alt+D` |
| Merge: 全体のdiff表示を切り替え | `Alt+F` |
| Merge: 元に戻す | `Ctrl+Z` |
| Merge: やり直す | `Ctrl+Y` |
| Merge: 解決済みにせず保存 | `Alt+S` |
| Merge: 保存して解決済みにする | `Ctrl+S` |
| Merge: 出力をmerge結果に戻す | `Alt+R` |
| Merge: 出力の高さを切り替え | `z` |
| Merge: 現在のペインを広げる | `Alt+Right` |
| Merge: 取り込みのペインを広げる | `Alt+Left` |
| Merge: ペインのサイズをリセット | `Alt+0` |
| Merge: この競合の選択を解除 | `Alt+U` |
| Merge: 現在（A）ペインにフォーカス | `Alt+1` |
| Merge: 取り込み（B）ペインにフォーカス | `Alt+2` |
| Merge: 出力ペインにフォーカス | `Alt+3` |
| Merge: ベースペインにフォーカス | `Alt+4` |
| Merge: 左のペインへ移動（同じ競合） |  |
| Merge: 右のペインへ移動（同じ競合） |  |
| Merge: 次のペインにフォーカス | `F6` |
| Merge: 前のペインにフォーカス | `Shift+F6` |
| Merge: ディスク上のファイルから続ける |  |
| Merge: merge結果からやり直す |  |
| Merge: 解決を取り消して再度開く |  |
| Merge: ファイルを再読み込み |  |
| Merge: 現在のファイルをそのまま使用 | `a` |
| Merge: 取り込みのファイルをそのまま使用 | `b` |
| Merge: ベースのファイルをそのまま使用 | `o` |
| Merge: 残っているファイルを残す | `k` |
| Merge: 削除を受け入れる | `D` |
| Merge: 外部mergeツールで開く | `m` |
| ✦ AI: merge競合の解決案を提案 | `Alt+I` |
| Merge: mergeエディターを閉じる |  |
| Git: Merge、cherry-pick、リバートを中止 |  |
| Git: Cherry-pick、リバートをスキップ |  |
| リポジトリ: 開く／切り替え（リポジトリ管理）... | `Ctrl+O` |
| リポジトリ: リポジトリを参照... | `b（管理タブ）` |
| リポジトリ: 新規作成（初期化）... | `i（管理タブ）` |
| リポジトリ一覧: 選択中のものを開く | `Enter` |
| リポジトリ一覧: 選択中のもののお気に入りを切り替え | `f` |
| リポジトリ一覧: 選択中のものを閉じる／削除 | `x` |
| リポジトリ一覧: 選択中のリポジトリのメニュー... | `.` |
| リポジトリ一覧: 検索 | `/ / Ctrl+F` |
| リポジトリ一覧: すべてのセクションを折りたたむ | `-` |
| リポジトリ一覧: すべてのセクションを展開 | `+` |
| リポジトリ一覧: すべてのタブを閉じる... |  |
| リポジトリ一覧: 選択中のものを一覧から削除 |  |
| リポジトリ一覧: 最近の項目をすべて削除 |  |
| ワークスペース: 新規... | `n` |
| ワークスペース: 開いているタブから新規作成... | `a` |
| ワークスペース: 切り替え... | `Alt+W` |
| ワークスペース: 編集... | `e` |
| ワークスペース: 閉じる（ワークスペースなし） |  |
| ワークスペース: 管理... | `Alt+J` |
| ワークスペース: タブを閉じる |  |
| ワークスペース: 見つからないリポジトリの場所を指定... | `l` |
| ワークスペース: 削除... |  |
| ワークスペース: すべてFetch | `F` |
| ワークスペース: すべてPull | `P` |
| ワークスペース: 見つからないものをクローン | `m` |
| ワークスペース: タブをワークスペースに追加 |  |
| ワークスペース: タブをワークスペースから除外 |  |
| Radar: 開く | `Alt+R` |
| Radar: 更新 | `F5` |
| Radar: 検索 | `/` |
| Radar: ブラウザーで開く | `o` |
| Radar: リポジトリを開く | `Enter` |
| Radar: ブランチをチェックアウト | `c` |
| Radar: worktree を作成... | `t` |
| Radar: リポジトリで表示 | `s` |
| Radar: Issue の作業を開始... | `n` |
| Radar: 固定／固定解除 | `p` |
| Radar: 保留... | `z` |
| Radar: リクエストをレビュー... | `e` |
| Radar: API 経由でレビュー |  |
| Radar: クローンしてレビュー... |  |
| Radar: 担当者を編集... |  |
| Radar: レビュアーを編集... |  |
| Radar: ラベルを編集... |  |
| Radar: マイルストーンを設定... |  |
| Radar: 対象を選択 | `w` |
| Radar: 表示を選択 | `v` |
| Radar: 絞り込みを選択 | `f` |
| Radar: 未クローンのリクエストを表示 | `a` |
| Radar: ドラフトを表示 | `d` |
| Radar: アカウント全体のレビュー依頼を有効にする... | `T` |
| Radar: 行メニュー... | `.` |
| Radar: 列を選択 | `C` |
| Radar: 列を広げる | `]` |
| Radar: 列を狭める | `[` |
| Radar: 列の幅をリセット | `0` |
| Radar: 列を内容に合わせる | `=` |
| Radar: 全列を内容に合わせる | `+` |
| Radar: ホームに戻る | `Esc` |
| Radar: タブを閉じる |  |
| タブ: 新しいタブ | `Alt+T` |
| タブ: 新規... |  |
| タブ: 操作メニュー... | `Ctrl+T` |
| タブ: ほかのタブを閉じる... |  |
| タブ: 右側のタブを閉じる... |  |
| タブ: お気に入りに追加／削除 |  |
| タブ: リポジトリのパスをコピー |  |
| タブ: ファイルマネージャーで開く |  |
| タブ: ターミナルで開く |  |
| タブ: 開く先... | `Alt+O` |
| タブ: 既定の開く先で開く | `Ctrl+Alt+O` |
| リポジトリ: プロファイルを変更... |  |
| ヘルプ: GitPoulpi について | `Ctrl+N a` |
| ヘルプ: ライセンスとクレジット |  |
| 表示: アクティビティログ | `Ctrl+N A` |
| 表示: グローバルアクティビティログ | `Ctrl+N G` |
| ヘルプ: リリースノート |  |
| 更新: 更新パネルを表示 | `Ctrl+N u` |
| 更新: セキュリティ通知を開く |  |
| 更新: 更新コマンドをコピー |  |
| 更新: 更新コマンドをターミナルで実行 |  |
| 更新: 今すぐ更新を確認 |  |
| 更新: ダウンロード |  |
| 更新: 今すぐ更新 |  |
| 更新: 再起動して更新 |  |
| 更新: 前のバージョンに戻す |  |
| 更新: このバージョンをスキップ |  |
| リリースノート: タブを閉じる |  |
| タスク: バックグラウンドタスクを表示 | `Ctrl+N J` |
| タスク: キャンセルできるものをすべてキャンセル |  |
| 表示: 診断情報をコピー |  |
| アクティビティログ: 「エラー」レベルの表示/非表示を切り替え |  |
| アクティビティログ: 「警告」レベルの表示/非表示を切り替え |  |
| アクティビティログ: 「情報」レベルの表示/非表示を切り替え |  |
| アクティビティログ: 「デバッグ」レベルの表示/非表示を切り替え |  |
| アクティビティログ: 「トレース」レベルの表示/非表示を切り替え |  |
| アクティビティログ: すべてのレベルを表示 |  |
| アクティビティログ: どのレベルも表示しない |  |
| アクティビティログ: アプリケーションログを消去 |  |
| アクティビティログ: 対象範囲を選択 |  |
| アクティビティログ: Git 操作 |  |
| アクティビティログ: アプリケーションログ |  |
| アクティビティログ: この行の操作を表示 |  |
| アクティビティログ: 次の失敗した操作 |  |
| アクティビティログ: 前の失敗した操作 |  |
| アクティビティログ: 左へスクロール | `Left` |
| アクティビティログ: 右へスクロール | `Right` |
| アクティビティログ: 直前のエラーをコピー |  |
| アクティビティログ: 提案された操作を実行 |  |
| アクティビティログ: 直前のエラーを開く |  |
| タブ: 終了してここへ cd |  |
| リポジトリ: クローン... | `Ctrl+Shift+N` |
| リポジトリ: 次のタブ | `Ctrl+PageDown` |
| リポジトリ: 前のタブ | `Ctrl+PageUp` |
| タブ: 左へ移動 |  |
| タブ: 右へ移動 |  |
| リポジトリ: タブを閉じる | `Ctrl+W` |
| リポジトリ: タブを選択... | `t` |
| 検索: ワークスペース... | `Ctrl+Alt+F` |
| 編集: 元に戻す | `Ctrl+Z` |
| 編集: やり直す | `Ctrl+Y` |
| 編集: ブロックされた元に戻す項目を履歴から外す |  |
| 編集: 元に戻す履歴... | `Alt+Z` |
| 編集: Reflog... |  |
| 編集: 失われたCommitsを復旧... |  |
| 編集: ブランチをブロックされた操作が残した位置に復元 | `Alt+Shift+Z` |
| レビュー: 概要を表示 |  |
| レビュー: ファイルを表示 |  |
| レビュー: Commitsを表示 |  |
| レビュー: 行にコメント... | `c` |
| レビュー: 会話にコメント... | `C` |
| レビュー: スレッドに返信... | `r` |
| レビュー: スレッドを解決／再開 | `x` |
| レビュー: スレッドを折りたたみ／展開 | `z` |
| レビュー: 保留中のコメントを編集... | `e` |
| レビュー: 保留中のコメントを削除 | `d` |
| レビュー: ファイルを閲覧済みにする | `v` |
| レビュー: 次のファイル | `]` |
| レビュー: 前のファイル | `[` |
| レビュー: レビューを完了... | `f` |
| レビュー: 保留中のレビューを破棄... |  |
| レビュー: 更新 | `F5` |
| レビュー: ブラウザーで開く | `o` |
| レビュー: ブランチをチェックアウト | `b` |
| レビュー: 閉じる | `Esc` |
| レビュー: 担当者を編集... | `a` |
| レビュー: レビュアーを編集... | `p` |
| レビュー: ラベルを編集... | `l` |
| レビュー: マイルストーンを設定... | `m` |
| レビュー: パッチセットを選択... | `P` |
| レビュー: パッチセットを比較... | `D` |
| レビュー: チェンジをサブミット... | `U` |
| レビュー: チェンジを放棄... | `Q` |
| レビュー: レイアウトを切り替え（インライン／分割） | `V` |
| レビュー: ハンクのみを切り替え（ファイル全体／変更のみ） | `H` |
| レビュー: 空白のみの変更を切り替え | `S` |
| レビュー: 折り返しを切り替え | `W` |
| レビュー: 次のハンク | `J` |
| レビュー: 前のハンク | `K` |
| レビュー: 文字コードを選択... | `E` |
| レビュー: ミニマップを切り替え | `M` |
| レビュー: diffを左にスクロール | `Left` |
| レビュー: diffを右にスクロール | `Right` |
| レビュー: 選択したテキストまたは行をコピー | `Ctrl+C` |
| ボード: カードの移動先... | `m` |
| ボード: カードを左へ移動 |  |
| ボード: カードを右へ移動 |  |
| ボード: カードを開く | `Enter` |
| ボード: 担当者を編集... |  |
| ボード: ラベルを編集... |  |
| ボード: マイルストーンを設定... |  |
| ボード: この列に新規 Issue | `n` |
| ボード: 担当者で絞り込み... | `a` |
| ボード: ラベルで絞り込み... | `l` |
| ボード: マイルストーンで絞り込み... | `M` |
| ボード: テキストで絞り込み | `/` |
| ボード: 絞り込みを解除 | `x` |
| ボード: 列のラベルを設定... | `c` |
| ボード: 更新 | `r` |
| ボード: 閉じる | `Esc` |
| レビュー: ✦ レビューのドラフトを作成... | `s` |
| レビュー: 提案を採用 | `y` |
| レビュー: 提案を編集して採用... | `w` |
| レビュー: 提案を却下 | `n` |
| レビュー: すべての提案を採用 | `A` |
| レビュー: すべての提案を却下 | `N` |
| レビュー: AI の要約をレビュー本文に使用 | `i` |
| レビュー: ✦ ドラフトを使用... | `u` |
| レビュー: ✦ ドラフト本文を編集... | `t` |
| レビュー: ✦ ドラフトを破棄 | `N` |
| 元に戻す履歴: ここまで元に戻す... | `u` |
| 元に戻す履歴: ここまでやり直す... | `y` |
| 元に戻す履歴: これだけ復元... | `r` |
| 元に戻す履歴: ブランチを元の位置に復元... | `o` |
| 元に戻す履歴: 更新 |  |
| 元に戻す履歴: 閉じる | `Esc` |
| Reflog: Log で表示 | `l` |
| Reflog: ここにブランチを作成... | `b` |
| Reflog: ここにtagを作成... | `t` |
| Reflog: 変更を作業ツリーに適用 | `w` |
| Reflog: 変更をstashとして保存 | `s` |
| Reflog: この位置に復元... | `r` |
| Reflog: さらに読み込む | `m` |
| Reflog: 更新 |  |
| 失われた作業: ここにブランチを作成... | `b` |
| 失われた作業: ここにtagを作成... | `t` |
| 失われた作業: Cherry-pick | `c` |
| 失われた作業: 変更を作業ツリーに適用 | `w` |
| 失われた作業: 変更をstashとして保存 | `s` |
| 失われた作業: 再検索 |  |
| Reflog / 失われた作業: 次のペイン | `Tab` |
| Reflog / 失われた作業: パス一覧とツリーの切り替え | `v` |
| Reflog / 失われた作業: ペインのサイズをリセット |  |
| ダイアログ: 移動 / サイズ変更 | `Ctrl+N m` |
| ダイアログ: 位置とサイズをリセット |  |
| 表示: 更新 | `r` |
| 表示: 更新（fetch、pull request、Issue） | `F5` |
| 表示: ヘルプオーバーレイを切り替え | `?` |
| 表示: Ref 列を切り替え |  |
| 表示: グラフ列を切り替え |  |
| 表示: メッセージ列を切り替え |  |
| 表示: 作成者列を切り替え |  |
| 表示: 日付列を切り替え |  |
| 表示: 変更列を切り替え |  |
| 表示: SHA 列を切り替え |  |
| 表示: Log列: 移動・幅変更する列を選ぶ | `C` |
| 表示: Log列: 次の列を選ぶ | `Tab` |
| 表示: Log列: 前の列を選ぶ | `Shift+BackTab` |
| 表示: Log列: 列を左へ移動 | `<` |
| 表示: Log列: 列を右へ移動 | `>` |
| 表示: Log列: 列を広げる | `]` |
| 表示: Log列: 列を狭める | `[` |
| 表示: Log列: 列の幅をすべてリセット | `0` |
| 表示: Log列: 列を内容に合わせる | `=` |
| 表示: Log列: 全列を内容に合わせる | `+` |
| 表示: Log: 列を左へスクロール | `Shift+Left` |
| 表示: Log: 列を右へスクロール | `Shift+Right` |
| 表示: Log: 列の横スクロールを切り替え | `s` |
| 表示: Log: 表示する列を選択 |  |
| Diff: 選択中のファイルのdiffを開く | `Enter` |
| Diff: diffビューを閉じる | `Esc` |
| Diff: 次のハンク | `n` |
| Diff: 前のハンク | `p` |
| Diff: カーソル位置のハンクをステージ | `Space` |
| Diff: カーソル位置のハンクをステージ解除 | `Space` |
| Diff: 行を選択（または選択を解除） | `v` |
| Diff: 選択行をステージ | `Enter` |
| Diff: 選択行をステージ解除 | `Enter` |
| Diff: カーソル位置のハンクを破棄... | `x` |
| Diff: 選択行を破棄... | `X` |
| コピー: 選択したテキスト（先にテキストをドラッグ） | `Ctrl+C` |
| コピー: カーソル位置の行（または選択行） | `y` |
| Diff: カーソル位置のハンクをコピー |  |
| Diff: レイアウトをインライン（1 列に +/-） | `m` |
| Diff: レイアウトを分割（前後を並べる） | `m` |
| Diff: ハンクのみを切り替え（ファイル全体／変更のみ） | `H` |
| ファイル: パス/ツリーを切り替え | `m` |
| ファイル: すべてのファイルの表示を切り替え | `a` |
| Commit: フックのスキップを切り替え | `H` |
| ファイル: 行または選択範囲のメニューを開く | `.` |
| ファイル: すべてのフォルダーを展開 | `=` |
| ファイル: すべてのフォルダーを折りたたむ | `-` |
| ファイル: 選択中のフォルダーとそのサブフォルダーを展開 | `*` |
| ファイル: 選択中のフォルダーとそのサブフォルダーを折りたたむ | `_` |
| ファイル: すべて選択 |  |
| ファイル: 選択を解除 |  |
| ファイル: 現在の行の選択を切り替える |  |
| Log: Mermaid gitGraph としてコピー... | `Y` |
| Log: 選択を比較 | `v` |
| Log: 比較の両端を入れ替え | `x` |
| Log: 次の比較対象の行へ移動 | `n` |
| Diff: 折り返しを切り替え | `w` |
| Diff: 文字コードを選択... | `C` |
| 表示: ミニマップを切り替え | `M` |
| 表示: Commit列を切り替え | `Alt+C` |
| 表示: Commit列: 非表示 |  |
| 表示: Commit列: ファイル履歴 |  |
| 表示: Commit列: すべてのCommits |  |
| 表示: Commit列にフォーカス |  |
| 表示: Commit列を広げる |  |
| 表示: Commit列を狭める |  |
| 表示: Commit列: 左へスクロール | `Shift+Left` |
| 表示: Commit列: 右へスクロール | `Shift+Right` |
| 表示: Commit列: 次の欄の幅を調整 | `Tab` |
| 表示: Commit列: 欄を左へ移動 | `<` |
| 表示: Commit列: 欄を右へ移動 | `>` |
| 表示: Commit列: 欄を広げる | `]` |
| 表示: Commit列: 欄を狭める | `[` |
| 表示: Commit列: 各欄の幅をリセット | `0` |
| 表示: Commit列: 列を内容に合わせる | `=` |
| 表示: Commit列: 全列を内容に合わせる | `+` |
| 表示: Commit列: 選択行のメニュー | `.` |
| 表示: Commit列: 列を選択 |  |
| 表示: Commit列: Ref を切り替え |  |
| 表示: Commit列: グラフを切り替え |  |
| 表示: Commit列: 作成者を切り替え |  |
| 表示: Commit列: 日付を切り替え |  |
| 表示: Commit列: 変更を切り替え |  |
| 表示: Commit列: SHAを切り替え |  |
| 表示: Commit列: 選択行を Log で表示 |  |
| 表示: Commit列: 選択行でのファイルの変更を表示 |  |
| 表示: Commit列: 選択行の時点のファイルを表示 |  |
| 表示: Commit列: 選択行の時点でファイルを Blame |  |
| エディター: 選択に追従を切り替え |  |
| 画像: 拡大 | `+` |
| 画像: 縮小 | `-` |
| 画像: ビューに合わせる | `0` |
| 画像: 実寸（1:1） | `1` |
| 画像: 変更前 / 変更後を切り替え | `Tab` |
| 画像: 変更前ペインを広げる | `Alt+Right` |
| 画像: 変更後ペインを広げる | `Alt+Left` |
| 画像: ペインサイズをリセット | `Alt+0` |
| 画像: 編集に使う... | `O` |
| 画像: 既定のエディターで編集 | `o` |
| ファイル履歴: Log で表示 | `Enter` |
| ファイル履歴: commitの SHA をコピー | `y` |
| ファイル履歴: 要約をコピー | `Y` |
| ファイル履歴: このcommitのdiffを表示 | `d` |
| ファイル履歴: このcommit時点のファイルを開く | `o` |
| ファイル履歴: このcommitで Blame | `b` |
| ファイル履歴: 作業ツリーと比較 | `c` |
| ファイル履歴: このバージョンを復元 | `R` |
| ファイル履歴: 選択を上へ拡張 | `Shift+Up` |
| ファイル履歴: 選択を下へ拡張 | `Shift+Down` |
| ファイル履歴: リストとdiffを切り替え | `Tab` |
| ファイル履歴: リストを広げる | `Alt+Right` |
| ファイル履歴: リストを狭める | `Alt+Left` |
| ファイル履歴: リストの幅をリセット | `Alt+0` |
| ファイル: アプリで開く... | `O` |
| ファイル: 既定のアプリで開く | `o` |
| Diff: 空白のみの変更を切り替え | `s` |
| ✦ AI: Commitメッセージを生成 | `Ctrl+G i` |
| ✦ AI: Stashメッセージを生成 |  |
| ✦ AI: プロンプト変数を挿入 |  |
| ✦ AI: プロファイルを編集... |  |
| ✦ AI: エージェントを開く... |  |
| ✦ AI: 選択中のCommitを説明 | `Ctrl+G w` |
| ✦ AI: 選択中のCommitを分割... |  |
| ✦ AI: 作業中の変更を説明 | `Ctrl+G w` |
| ✦ AI: 作業中の変更からCommitを構成... | `Ctrl+G C` |
| コンポーザー: ファイルをCommitへ移動... | `m` |
| Diff: Blame |  |
| Blame: commitを Log で表示 | `Enter` |
| Blame: commitの SHA をコピー | `Y` |
| Blame: このcommitのこのファイルのdiffを表示 | `d` |
| Blame: このcommitの親で Blame | `b` |
| Blame: ファイル履歴を開く | `H` |
| Blame: 詳細カードを表示 / 非表示 | `i` |
| Blame: 次のハンク | `n` |
| Blame: 前のハンク | `p` |
| Blame: 同じcommitの次のハンク | `N` |
| Blame: 同じcommitの前のハンク | `P` |
| Blame: 行へ移動 | `g` |
| Blame: 注釈列を広げる | `>` |
| Blame: 注釈列を狭める | `<` |
| Diff: ファイル履歴 |  |
| Diff: ファイルビュー |  |
| ファイル: 作業ツリーで編集 | `e` |
| エディター: エンコードを変更 |  |
| エディター: エンコードを指定して開き直す |  |
| エディター: エンコードに変換 |  |
| エディター: 改行コードを変換 |  |
| エディター: インデントを変更 |  |
| アプリ: 設定... | `Ctrl+, / Ctrl+N ,` |
| アプリ: 設定を検索... | `設定内で /` |
| アプリ: キー割り当て... |  |
| エージェント: 保留中のビューを表示 | `Ctrl+N Enter` |
| エージェント: 保留中のビューを閉じる |  |
| エージェント: 戻る | `Ctrl+N Backspace` |
| キー割り当て: すべてリセット... |  |
| 連携: 関連するPull Requestを開く | `Ctrl+G g` |
| 連携: 関連するPull Requestをホストで編集 | `e（行の上で）` |
| 連携: 関連するPull Requestの URL をコピー |  |
| 連携: Pull Requestと Issue を更新 |  |
| 連携: Pull Requestをチェックアウト... |  |
| 連携: commitの参照を開く（#issue / !request）... |  |
| 連携: Issue を開く... |  |
| ホスティング: リクエストをレビュー... |  |
| ホスティング: レビュー用にPush... |  |
| ホスティング: 担当者を編集... |  |
| ホスティング: レビュアーを編集... |  |
| ホスティング: ラベルを編集... |  |
| ホスティング: マイルストーンを設定... |  |
| ローカル Issue: Issue をクローズ... |  |
| ローカル Issue: Issue を再オープン... |  |
| ローカル Issue: Obsidian で開く... |  |
| Issue: ボードを開く |  |
| ローカル Issue: Obsidian Base を作成 |  |
| Pull Request: Pull Requestを作成... | `n` |
| ホスティング: 新規 Issue... | `n` |
| Pull Request: 検索 | `検索行で Enter` |
| Pull Request: 絞り込みを選択（オープン / merge済み / すべて）... | `v` |
| Pull Request: プロバイダーを選択... | `プロバイダー行で Enter` |
| Pull Request: リポジトリを選択... | `リポジトリ行で Enter` |
| Pull Request: ソースのリンクを解除（なし） |  |
| Issue: プロバイダーを選択... | `プロバイダー行で Enter` |
| Issue: ソースのリンクを解除（なし） |  |
| 連携: プロファイルのリンクを解除（自動） |  |
| Issue: リポジトリを選択... | `リポジトリ行で Enter` |
| Issue: 検索 | `検索行で Enter` |
| Issue: 絞り込みを選択... | `v` |
| Issue: クエリ / リストを編集... |  |
| Commit: Issue の参照を挿入... |  |
| 連携: このリポジトリで使うプロファイルを選択... |  |
| 連携: このリポジトリのプロファイルを管理 |  |
| 連携: 連携を接続... |  |
| 設定: プロファイルのレーダーオプション... |  |
| 連携: セキュリティチェックを確認 |  |
| アプリ: 終了 | `q` |
| アプリ: テーマ... |  |
| 競合: ベースブランチの一覧を編集... |  |
| 競合: このリポジトリのベースブランチ... |  |
| アプリ: テーマを変更... | `Ctrl+N y` |
| アプリ: 言語を変更... | `Ctrl+N g` |
| ランチャー: ショートカットを作成 |  |
| ランチャー: デスクトップのショートカットを作成 |  |
| ランチャー: ショートカットを削除 |  |
| ランチャー: コマンドのエイリアスを作成 |  |
| ランチャー: コマンドのエイリアスを削除 |  |
| ランチャー: PATH に追加 |  |
| ランチャー: PATH から削除 |  |
| ランチャー: 「GitPoulpi で開く」メニューをインストール |  |
| ランチャー: 「GitPoulpi で開く」メニューを削除 |  |
| アプリ: グラフのカラーテーマを変更... | `Ctrl+N Y` |
| アプリ: グラフの行の色付けを切り替え | `Ctrl+N T` |
<!-- palette:end -->

## 移動

| キー | 動作 |
|---|---|
| `Tab` / `Shift+Tab` | 次 / 前のペインにフォーカスする |
| `Ctrl+N`、続けてバッジ | ウィンドウのその部分にフォーカスする（[ナビゲーションプレフィックス](#ナビゲーションプレフィックスctrln)を参照） |
| `j` / `k`、`Up` / `Down` | すべてのペインで選択を移動する |
| `PageUp` / `PageDown`、`Home` / `End` | 1 ページ移動する。最初 / 最後の行へ移動する |
| 数字、続けて `j` / `k` / `Up` / `Down` | 回数プレフィックス：`4j` は 4 行移動します。`PageUp` / `PageDown` では、回数はページ数になります。ステータスバーに保留中の回数が表示されます（`4...`）。`Esc` または他のキーで解除されます。先頭の `0` は回数の開始になりません |
| `h` / `l`、`Left` / `Right` | Refs ペイン：セクションを切り替える。ログ：commit の変更ファイル一覧から出る（`h`）/ 入る（`l`） |
| `?` | ヘルプを開く / 閉じる（[アプリ内ヘルプ](#アプリ内ヘルプ)を参照） |
| `r` | 更新 |
| `F5` | すべてのリモートを fetch し、pull request と Issue を再読み込みして、更新する |
| `q` | 終了 |
| `Ctrl+Z` | 直前の操作を**元に戻す**。[元に戻す・やり直す](undo-redo.md)を参照 |
| `Ctrl+Y`、`Ctrl+Shift+Z` | **やり直す**。`Ctrl+Shift+Z` には kitty キーボードプロトコルが必要です。`Ctrl+Y` は常に使えます |
| `.`、`Menu`、`Shift+F10` | 選択した行のコンテキストメニューを開く |

### テキストフィールドでの Ctrl+Z / Ctrl+Y

テキストフィールドがキーボードを使っている間、`Ctrl+Z` と `Ctrl+Y`（`Ctrl+Shift+Z` も）はそのフィールドのテキストに作用し、リポジトリには作用しません。キーボードを使っているフィールドが常に優先されるので、commit メッセージで `Ctrl+Z` を押しても commit を取り消すことはありません。それ以外の場所では、これまでどおりの全体の意味（上の元に戻す / やり直すの履歴）を持ちます。これは、すべてのテキストフィールドと複数行エディターに当てはまります：commit メッセージ、設定のフィールドとテキストエディター、pull request と Issue のフォーム、AI のダイアログ、すべてのフィルターと検索ボックスです。

- **入力**は 1 語ずつ取り消されます（文字の連続、空白の連続、記号の連続）。1 秒以上間が空いた入力は、新しいステップになります。続けた Backspace は 1 ステップです。貼り付け、切り取り、`Ctrl+U` / `Ctrl+K`、挿入したスニペット、受け入れた補完は、それぞれ 1 ステップです。
- 元に戻すと、テキスト、キャレット、選択範囲が戻ります。元に戻した後に新しく編集すると、やり直せたはずのものは破棄されます。
- 各フィールドは上限のある履歴（直近 200 ステップ、約 1 MB のテキスト）を保持し、フィールドが外から置き換えられたとき（ピッカーで選んだフォルダー、再読み込みしたフォーム）や、別のフィールドがキーボードを使い始めたときに、それを忘れます。
- 元に戻すものがなくなると、`Ctrl+Z` は何もしません。フィールドがキーボードを使っている間は、リポジトリの元に戻すに回ることはありません。
- AI 設定の**指示エディター**にも、保存とキャンセルの隣に**元に戻す**と**やり直す**のボタンがあり、移動できるステップがない間は無効になります。
- コードエディターとマージエディターは、それぞれ独自の元に戻す（`編集: 元に戻す`、`Merge: 元に戻す`）を持ち、選択も対象になります。

## 絞り込みできるポップアップ一覧

絞り込みできるポップアップ一覧は、すべて同じキーを使います。コマンドパレットとその選択画面（テーマ、言語、グラフの色）、タブの選択、検索行のあるすべてのドロップダウン（設定の選択肢、pull request フォーム、Issue 参照ピッカー、**開く場所**の一覧）です。入力すると一覧が絞り込まれます。次のキーでハイライトを移動します。

| キー | 動作 |
|---|---|
| `Up` / `Down`、`Ctrl+K` / `Ctrl+J` | 1 行上 / 下へ移動する |
| `Home` / `End` | 最初 / 最後の行 |
| `PageUp` / `PageDown` | 見えている 1 ページ分移動する |
| `Enter` | ハイライトされた行を選ぶ |
| `Esc` | 閉じる |

これらのポップアップでは、`Home` と `End` はフィルターフィールドのキャレットではなく、一覧を移動します。`Ctrl+K` / `Ctrl+J` も一覧を移動するため、`Ctrl+K` はフィルターを末尾まで削除しなくなりました（`Ctrl+U` は引き続きクリアします。一覧の外のふつうのテキスト欄では、`Ctrl+K` はこれまでどおり行末まで削除します）。`Ctrl+A` と `Ctrl+E` は、これまでどおりキャレットを先頭と末尾へ移動します。`Ctrl+J` / `Ctrl+K` は、テキスト欄の有無にかかわらず、ほかのすべての一覧も移動します（`Ctrl+N` はナビゲーションのプレフィックス、`Ctrl+P` はコマンドパレットなので、一覧のキーではありません）。ふつうの `j` と `k` はフィルターに入力されます。テキスト欄のない一覧では、`j` / `k` でも下 / 上へ移動できます。

マウスでは次のようになります。

- ホイールは一覧を**スクロール**します。ハイライトは移動しません。
- クリックは行を**選択**します。ダブルクリックは、`Enter` と同じく行を**選びます**。
- ポップアップより縦に長い一覧では、右端にスクロールバーが表示されます。ドラッグできます。
- 検索行のない短いドロップダウン（選択肢が 2 つか 3 つ）では、最初のクリックで選びます。

**ライブプレビュー。** テーマ、グラフカラーテーマ、言語のピッカーは、ハイライトされた行をポップアップの背後ですぐに反映します。`Enter` で確定します。`Esc`、またはポップアップの外をクリックすると、前の値に戻ります。

**テーマフィルター。** テーマピッカーには、フィルターフィールドの下に行があります。**すべて**、**ダーク**、**ライト**、**カラー**です。`Tab` / `Shift+Tab` で次 / 前に移動します。`Alt+1` から `Alt+4` で直接選べます。クリックでも選べます。

## アプリ内ヘルプ（`?`）

`?`、`Ctrl+N ?`、またはパレットの `表示: ヘルプオーバーレイを切り替え` でヘルプが開きます。アプリのあらゆる部分のキーとマウスジェスチャーが一覧になっています。設定画面を含め、どの画面の上にも開きます。

| キー | 動作 |
|---|---|
| `?` | ヘルプを閉じる（フィルターも消去する） |
| *（入力）* | フィルター：そのテキストを含むセクションだけを残します。`j`、`k`、`q` も入力されます |
| `Backspace`、`Ctrl+U` | 最後の 1 文字を削除する / フィルターを消去する |
| `Esc` | フィルターを消去する。空の場合は、ヘルプを閉じる |
| `Up` / `Down` | 1 行スクロールする |
| `PageUp` / `PageDown`、`Home` / `End` | 1 ページスクロールする。先頭 / 末尾へ移動する |
| `Ctrl+D`、またはリンクのクリック | このドキュメントを GitHub でブラウザーに開く |

ホイールでヘルプをスクロールでき、スクロールバーをドラッグすることもできます。ヘルプの外をクリックすると閉じます。

## ナビゲーションプレフィックス（`Ctrl+N`）

`Ctrl+N` は、もう 1 つのキーを待ちます。ステータスバーには、受け付けるキーが表示されます。次のキーでアクションを実行するか、キャンセルします。

| キー | 動作 |
|---|---|
| `Ctrl+N`、続けてバッジ | ウィンドウのこのバッジが表示されている部分にフォーカスする。手前のダイアログやポップアップなら、そのコントロールを押す |
| `Ctrl+N h` / `Ctrl+N l` | Refs ペイン / 右ペインを折りたたむ、または復元する |
| `Ctrl+N Left` / `Ctrl+N Right` | Refs ペインとログの間の区切りを 1 段階左 / 右へ動かす（折りたたまれた Refs ペインは先に戻ります） |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | ログと右ペインの間の区切りを 1 段階左 / 右へ動かす |
| `Ctrl+N +` / `Ctrl+N -` | アクティブな Refs セクションを 1 行ぶん拡大 / 縮小する |
| `Ctrl+N ]` / `Ctrl+N [` | 右ペインの commit メッセージ欄を 1 行ぶん拡大 / 縮小する |
| `Ctrl+N t` | タブを選ぶ（タブピッカー。`t` だけでも同じ） |
| `Ctrl+N y` / `Ctrl+N g` / `Ctrl+N Shift+Y` | テーマ / 言語 / グラフカラーテーマを変更する |
| `Ctrl+N Shift+T` | グラフの行の色付けをオン / オフにする |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | エージェントが用意したビューを表示する / 戻る（[MCP とエージェント](mcp-and-agents.md)を参照） |
| `Ctrl+N ,` | 設定を開く |
| `Ctrl+N ?` | ヘルプを開く / 閉じる |
| `Esc`、または他のキー | キャンセル |

### バッジ

プレフィックスが待機している間、フォーカスを受け取れるウィンドウの各部分には、左上の枠に 1 文字のバッジが表示されます。その文字を押すとフォーカスします。バッジはその都度、画面上の順序で割り当て直されます。まず `1` から `9`、次に `a`、`b`、`c`... です。そのため、画面上のバッジを読み取ってください。同じ部分が毎回同じバッジになるとは限りません。

バッジが付く部分：Refs フィルターと表示中の各 Refs セクション、ログ（または Diff ビュー）、開いているときのファイルビュー、未ステージとステージ済みの一覧、commit コンポーザーの概要フィールド、commit ファイル一覧のコントロール。ホームタブでは、バッジは検索フィールド、4 つのセクション、ツールバーのボタン（ボタンのバッジはそのボタンを押します）に付きます。Radar タブでは、フィルターフィールド、対象・表示・絞り込みのコンボボックス（コンボボックスのバッジは一覧を開きます）、更新と戻るのボタン、画面上の各セクション見出しに付きます。セクションのバッジは選択をその見出しに移します。続けて `Enter` または `Space` を押すと折りたたみます。

参照のフォルダーピッカー、およびクローン、初期化、worktree 作成、ワークスペース、スパースチェックアウト、Flow、新規 Issue とプルリクエストの各ダイアログでは、バッジはフィールド、チェックボックス、ボタンに付きます（ボタンのバッジはそのボタンを押し、フィールドのバッジはそこにカーソルを置きます。リストでは、最初の行に 1 つ付きます。参照のフォルダーピッカーには、クイックアクセスの一覧とフォルダーの一覧に 1 つずつバッジがあり、各一覧の最初の行の横に付いて、押すと行を選ばずにキーボードがその一覧に移ります）。アクティビティログでは、2 つのタブ、対象範囲とレベルのボックス、フィルター、レベルのチェックボックス、ボタン、一覧の最初の項目に付きます。バッジはそのコントロールをクリックしたのと同じ動作をします。フォルダーピッカーでは `Ctrl+N` がバッジを開始するため、新規フォルダーのキーは `Alt+N` です。

修正（およびコミットメッセージの編集）ダイアログでは、バッジは概要、説明、作成者の名前・メールアドレス・日付、**履歴をさらに深く変更...**、**キャンセル**、**適用**、閉じる ✕ に付きます。履歴書き換えワークベンチでは、対象範囲のボックスとそのチェックボックスおよび ref ボタン、レシピ名、各ルールとそのチェックボックスおよびボタン、**+ ルールを追加** ボタン、選択中のルールの項目、プレビューのタブ、フィルターボックス、保護ブランチの行とそのすべてチェック / すべて解除、そしてプレビュー、適用、キャンセル、保存、読み込み、AI でルールを作成のボタンに付きます（シークレット、名義、サイズ、結果の各ページは独自のボタンと行を加え、コミットタブの行は一覧なのでバッジはありません）。バッジはクリックしたのと同じ動作をします。ドロップダウン、質問、入力中のフィールド、フォルダーピッカー、消去画面、スクリプトエディター、AI ダイアログ、blob の diff、ホストの手順が開いている間、また修正ダイアログが「破棄しますか？」と尋ねている間は、バッジは付きません。`Esc` は何も閉じずにバッジを消します。

設定では、バッジは検索ボックス、ページの一覧（現在のページに 1 つ）、サブタブ、リストとそのボタン、画面上にあるページの各フィールド、閉じる `✕` に付きます。バッジはそのコントロールをクリックしたのと同じ動作をします。テキストフィールドは編集状態で開き、チェックボックスは切り替わり、ドロップダウンは開き、ボタンは実行されます。フィールドに入力している間や、ドロップダウン、エディター、検索が開いている間は、`Ctrl+N` は本来の意味のままです。

パレットにも同じアクションがあります。たとえば `ナビゲーション: Refs > ローカルにフォーカス` や `ナビゲーション: Status > ステージ済みにフォーカス` です。

## リポジトリとタブ

| キー | 動作 |
|---|---|
| `Ctrl+O`、または `⌂` のクリック | ホームタブへ移動する（ホームタブ上では、検索フィールドにフォーカスする。Radar またはリリースノートが開いているときは、先にそれを閉じる） |
| `Ctrl+Shift+N` | クローンダイアログ付きでホームタブを開く |
| `Ctrl+PageDown` / `Ctrl+PageUp` | 次 / 前のタブ |
| `t` | 絞り込みできる一覧からタブを選ぶ |
| `Ctrl+W` | 現在のタブを閉じる（先に確認します。リポジトリは変更されません） |
| `Ctrl+T`、またはタブの右クリック | タブメニュー（項目は[タブ行](interface-tour.md#タブ行)に一覧があります） |
| `Alt+O` / `Ctrl+Alt+O` | タブのフォルダーを開く：**開く場所**の一覧から選ぶ / 既定の項目を使う。[開く場所](open-in.md)を参照 |

### ホームタブ

`⌂` タブでリポジトリを管理します。**開いているリポジトリ**、**お気に入り**、**最近のリポジトリ**、**すべてのリポジトリ**の 4 つのセクションがあります。[インターフェースツアー](interface-tour.md#ホーム)を参照してください。

| キー | 動作 |
|---|---|
| `j` / `k`、矢印キー、`PageUp` / `PageDown`、`Home` / `End` | 移動する |
| `Enter`、またはダブルクリック | リポジトリを開く（またはそのタブに切り替える）。セクションヘッダー上では、折りたたむ / 展開する |
| `f` または `*`、または星のクリック | お気に入りに追加 / から削除する |
| `x`、`Delete`、または行のボタン | そのタブを閉じる（開いている）、一覧から削除する（最近）、星を外す（お気に入り）、忘れる（すべて）。ディスク上のものは何も削除されません |
| `.`、右クリック、または行の `⋮` | 行のメニュー |
| `/` または `Ctrl+F` | すべてのセクションを名前、ブランチ、パスで検索する。`Enter` で最初の一致を開き、`Esc` で検索を消去し、もう一度押すとフィールドから出ます |
| `Space`、`Left` / `Right` | 現在のセクションを折りたたむ / 展開する |
| `-` / `+` | すべて折りたたむ / すべて展開する |
| `b` / `c` / `i` | 参照（フォルダーを開く）/ クローン / 初期化 |
| `Esc` | 検索を消去する、またはアクティブなリポジトリタブへ戻る |
| `q` | 終了 |

`Ctrl+P`、`Ctrl+,`、`?`、`Ctrl+N` はホームタブで使えます。`Ctrl+G`、`Ctrl+W`、`Ctrl+T` にはリポジトリタブが必要です。ホームタブではメッセージが表示されるだけです。リポジトリが必要なパレットのコマンドはグレー表示になります。

### フォルダーピッカー

**参照**（および、初期化、クローン、worktree 作成の各ダイアログの**参照...**ボタン）でフォルダーピッカーが開きます。上にパスフィールド、左に場所とフォルダーツリー、右に現在のフォルダーの内容があります。

| キー | 動作 |
|---|---|
| `Tab` / `Shift+Tab` | パスフィールド、次に左の列、次に内容 |
| `Up` / `Down`、`PageUp` / `PageDown`、`Home` / `End` | フォーカスしている列で移動する。パスフィールドでは、補完候補の間を移動する |
| 文字（内容） | その文字で始まる次のフォルダーへ移動する |
| `Enter` / `Right` | 選択したフォルダーを開く。パスフィールドでは、入力したフォルダーへ移動する。すでに表示されているフォルダーなら、それを選ぶ |
| `Backspace` / `Left` / `Alt+Up` | 親フォルダー（パスフィールドでは、`Backspace` は 1 文字削除） |
| `Tab`（パスフィールド） | 唯一の補完候補、またはハイライトされた候補を受け入れる |
| `Alt+N` | 新規フォルダー（`Ctrl+N` はバッジを開始します。[バッジ](#バッジ)を参照） |
| `Alt+H` | 隠しフォルダーを表示 / 非表示にする |
| `Ctrl+F`、またはパスフィールドの外で `/` | 入力に合わせてフォルダーの項目を絞り込む（大文字と小文字を区別しません。`..` は残ります。ヘッダーに「x / n」と件数が出ます。`Enter` で最初の一致へ移動、`Esc` で消去。フォルダーを移ると空になります）。`Tab` やクリックでも移動できます |
| `Ctrl+S`、`Ctrl+Enter`、または確定ボタン | 選択したサブフォルダー、なければ表示中のフォルダーを選ぶ |
| `Esc` | 補完候補を閉じる、またはキャンセルする |

パスフィールドに新しいパスが表示されると、パス全体が選択されます。最初に入力したキーがそれを置き換えます。

### 初期化とクローンのダイアログ

| キー | 動作 |
|---|---|
| `Tab` / `Shift+Tab`、`Up` / `Down` | 次 / 前のフィールドまたはボタン |
| `Enter` | フォーカスしているボタンを押す、または作成 / クローンする |
| `Space` | フォーカスしているチェックボックスを切り替える、またはフォーカスしているボタンを押す |
| `Ctrl+B` | フォルダーを参照する |
| `j` / `k`、`g` / `G` | クローンダイアログのソース一覧とリポジトリ一覧で：下 / 上、最初 / 最後（検索フィールドの中を除く） |
| `Esc` | 閉じる。クローン中は、クローンはバックグラウンドで続きます |

クローンダイアログでは、左の列に **URL でクローン**と、接続済みのホスティングプロファイルごとに 1 つのソース（GitHub、GitLab、Gitea、Forgejo、Azure DevOps、Bitbucket、Gerrit）が並びます。`Up` / `Down` またはクリックでソースを選びます。次に、**リポジトリ**の検索フィールドに入力し、矢印キー、`PageUp` / `PageDown`、またはホイールで一覧を移動し、`Enter` またはクリックで行を選びます。各フィールドについては[はじめに](getting-started.md)を参照してください。

## Refs ペイン

| キー | 動作 |
|---|---|
| `Enter` / `Space` | アクティブなセクション、または選択したフォルダーを展開 / 折りたたむ |
| `h` / `l`、`Left` / `Right` | セクションを切り替える |
| `/` | 名前で絞り込む（`Enter` で適用、`Esc` でキャンセル） |
| `y` | ブランチ階層ビューのオン / オフ：ブランチ名の `/` からフォルダーを作る（ローカルとリモート） |
| `o` | 選択した参照、または選択したフォルダーやリモートの下のすべての参照を**ソロ表示**する。複数の参照をソロ表示すると、それらの和集合が見えます |
| `Shift+O` | **ソロ表示を解除**：すべてのソロ表示をクリアする |
| `Shift+H` | **すべて表示**：非表示にしたすべての参照を再び表示する |
| `g` | **グラフで表示**：ログを選択した参照（または stash）の commit までスクロールして、その行を点滅させる。選択は移動しません |
| `m` | 選択した参照をドロップ元として**マーク**する（`Esc`、またはその参照の上でもう一度 `m` でマークを解除） |
| `Shift+M` | マークした参照を選択中の参照に**ドロップ**する：ドラッグ＆ドロップで開くのと同じメニュー（Merge、Rebase、Push、早送り...）が開く |
| `+`（リモートセクション） | リモートを追加する |
| `.`、`Menu`、`Shift+F10`、または右クリック | 行のメニュー：ブランチ、tag、リモートの名前、またはフォルダー |
| WORKTREE の行で `Enter` | その worktree を専用のタブで開く |
| `Ctrl+G d` | 選択したブランチまたは tag を削除する（先に確認します）。リモートの行では、そのリモートブランチを削除する |

メニューの内容については、[ログを使う](log-and-history.md)を参照してください。

### Pull request と Issue

対応する連携があると、Refs ペインに pull request のセクションが表示されます（GitLab では merge request）。ホスティングのプロファイルが接続されていると、ISSUES セクションも表示されます。[ホスティングと Issue トラッカー](hosting-integration.md#5-サイドバーの-pull-request-と-issue)を参照してください。

| キー | 動作 |
|---|---|
| `Enter` | 選択した行をブラウザーで開く。`再接続` の行では、そのプロファイルの設定を開く |
| `Space` | セクションを展開 / 折りたたむ |
| `v` | セクションのフィルターを開く。pull request：オープン、merge 済み / クローズ、すべて。Issue：オープン、自分に割り当て、クローズ、すべて |
| `n` | pull request セクション：pull request を作成する。ISSUES セクション：新しい Issue（ヘッダーの `+` ボタン） |
| `e` | pull request セクション：選択したリクエストをホスト上で編集する（行の鉛筆） |
| 検索行で `Enter` | 検索フィールドに入力する。`Enter`、`Tab`、`Esc` で抜け、`Ctrl+U` で消去する |
| グループヘッダーで `Enter` | グループを折りたたむ / 展開する（*自分のpull request*、*自分に割り当て*、*自分のレビュー待ち*、*すべてのpull request*） |
| フィルター、プロバイダー、リポジトリの行で `Enter` | そのドロップダウンを開く。プロバイダーの一覧の末尾は**なし（このリポジトリのリンクを解除）** |
| `.`、または右クリック | 行のメニュー：ブラウザーで開く、URL または番号をコピー、ソースブランチをチェックアウト（pull request）、Issue からブランチを作成（Issue） |

### Issue ボード

ISSUES ヘッダーの**ボード**（パレット：**Issue: ボードを開く**）は、Issue の取得元のカンバンボードを開きます。[ボード](hosting-integration.md#5b-issue-ボード)と[ローカル Issue](local-issues.md#カンバンボード)を参照してください。以下のキーはすべてパレットのコマンド（**ボード: ...**）でもあります。

| キー | 動作 |
|---|---|
| 矢印 | カードを選択する（`Up`、`Down`）、または隣の列へ移動する（`Left`、`Right`）。`Home`、`End`、`PageUp`、`PageDown` は列内でジャンプする |
| `Shift+Left`、`Shift+Right` | 選択したカードを前または次の列へ移動する |
| `m` | **カードの移動先...**：列のドロップダウン |
| `Enter` | Issue を開く（ローカル Issue ではノート、それ以外ではホストのページ） |
| `e`、`.` | 担当者、ラベル、マイルストーンを編集する。カードのメニュー |
| `n` | 選択中の列に新規 Issue を作る |
| `a`、`l`、`M`、`/` | 担当者、ラベル、マイルストーン、テキストで絞り込む。`x` で絞り込みを解除する |
| `c` | GitHub / GitLab のボードの列になるラベル |
| `r` | Issue を読み直す |
| `Esc` | ボードを閉じる |

マウス：カードを別の列へドラッグすると移動し、クリックで選択、ダブルクリックで開き、右クリックでカードのメニュー、ホイールで列をスクロール、`Shift` とホイール（またはボードの下のバー）で列を横にスクロールします。

### Flow（ブランチワークフロー）

リポジトリにワークフローが設定されていると（[ブランチワークフロー](branching-workflows.md)を参照）、ツールバーに **Flow** ボタンが表示され、サイドバーは LOCAL と REMOTE をブランチの種類でグループ化します。メニューのすべての行はパレットコマンドです
（**Flow: ...**。**Flow: Feature を開始...** と **Flow: 新しい Feature Issue...** は、ワークフローのトピックの種類ごとに生成されます）。ホスト主導の行（*新しい {種類} Issue...*、*ホストが作成したブランチを引き継ぐ...*）とそのダイアログ、待機カード、
「ブランチが現れた」カードは、[ブランチワークフロー](branching-workflows.md#アプリでのホスト主導のフロー)で説明しています。
`Enter` はフォーカス中のボタン（*Issue を作成*、*引き継ぐ*）を実行し、`Esc` は待機を中止するか閉じます。

| キー | 動作 |
|---|---|
| `Ctrl+G Shift+W` | Flow メニューを開く（ツールバーのボタン） |
| `Enter`、`Tab`、`Shift+Tab` | 開始 / 完了 / 同期 / 昇格 / 親を設定のダイアログで：プレビューまたは実行、次のフィールド、前のフィールド |
| `Space` | チェックボックスを切り替える（*ブランチを公開*、*フックを実行*、*その上に積まれたブランチも更新*） |
| `Esc` | ドロップダウンを閉じる、プレビューからフォームに戻る、ダイアログを閉じる（実行中を除く） |
| ブランチにポインターを置く | その種類、親、親に対してどれだけ遅れているか |


## ログペイン

| キー | 動作 |
|---|---|
| `Enter` / `l` | 選択した commit の変更ファイル一覧に入る |
| ファイル行で `Enter` / `l` | そのファイルの diff を中央に開く |
| `Esc` / `h` | commit の一覧に戻る |
| `Right` | （複数の参照がある commit）ほかの参照の一覧を開く：`Up` / `Down` で移動、`Enter` でチェックアウト、`Esc` で閉じる |
| `f` | 検索バーを開く（[検索](#検索)を参照）。これが `/` だったことはないので、ログの検索は変わりません |
| `/` | Refs フィルターのフィールドにフォーカスする（Refs ペインにフォーカスがあるときも同じキー）。テキストフィールドにフォーカスがあるときは文字として入力される |
| `c` | （未 commit の変更がある場合）WIP 行を選択し、キャレットをそのメッセージフィールドに置く |
| WIP 行で `Enter` | キャレットを WIP 行のメッセージフィールドに置く |
| `.`、`Menu`、`Shift+F10` | commit メニュー。`Up` / `Down` で移動、`Right` / `l` / `Enter` でサブメニューを開く、`Left` / `h` / `Esc` で戻る |
| `m` / `a` | 右ペインのファイル：`パス`または`ツリー` / `すべてのファイルを表示` |
| `Shift+C` | 移動・幅変更する Log の列を選ぶ（下を参照） |

列を選んでいる間は、`Tab` / `Shift+Tab` で別の列を選び、`<` / `>` で移動し、`]` / `[` で広げる・狭め、`0` ですべての幅を自動に戻し、`Esc` か `Enter` で終了します（上の生成された表のコンテキスト `log_columns`）。

WIP 行のメッセージフィールドは、commit コンポーザーの概要です。キャレットがある間は、すべてのキーが入力されますが、`Ctrl` を使うキーは引き続き動作します。`Enter` でコンポーザーの説明に移動します。`Esc` でフィールドを離れ、テキストはそのまま残ります。[ステージと commit](staging-and-committing.md)を参照してください。

### 複数の commit を選択する

| キーまたは操作 | 動作 |
|---|---|
| `Space` | 現在の commit を選択に追加する、または外す |
| `Shift+Up` / `Shift+Down`、`Alt+Up` / `Alt+Down` | 選択を広げる（Shift をターミナル自身が使う場合は Alt） |
| `Ctrl+クリック` / `Shift+クリック` または `Alt+クリック` | commit を切り替える / 範囲を選択する |
| `Esc` | 選択を解除する |
| `Shift+Y` | [選択を Mermaid gitGraph としてコピーする](log-and-history.md#commit-を-mermaid-gitgraph-としてコピーする)（詳細度の選択とライブプレビュー付きのポップアップ。`Enter` でコピー） |

複数の commit を選択した状態で、そのうちの 1 つを右クリックし、**N 件の commit を `<branch>` に cherry-pick...** を選びます。インタラクティブ rebase と同じビューが開きます。

### ブランチまたは tag に名前を付ける

`Ctrl+G b`、`Ctrl+G t`、ツールバーの**ブランチ**と **Tag** ボタン、メニューの**ここにブランチを作成...** / **ここにtagを作成...** では、commit の行にあるテキストボックスで名前を尋ねます。`Enter` で参照を作成します。`Esc` でキャンセルします。無効な名前や既存の名前は赤で表示され、ボックスは開いたままです。

その行が画面に出ていない場合（diff を開いている、ログを単独表示にしている、Ref 列を非表示にしている）は、代わりに閉じる ✕、**キャンセル**、**作成**のボタンが付いた小さな枠付きボックスに名前を入力します。ブランチの名前変更は、Refs ペインのその行、またはログ行の Ref セルの上で入力します。ファイルの作成は、変更一覧のフォルダーの下で入力します。キーはどこでも同じです：`Enter` で確定、`Esc` でキャンセル。

ステータス行に入力することはありません。より広い場所が必要な入力は、本物のボタンが付いたダイアログになります：stash のメッセージ、注釈付き tag のメッセージ（`Ctrl+Enter` または `Ctrl+S` で適用、`Enter` で改行）、アップストリームの設定（リモートブランチの検索可能な一覧と**アップストリームを解除**ボタン）、Git LFS の追跡・追跡停止のパターン。パレットから選んだ数値やテキストの設定は、その行を編集できる状態で設定を開き、Refs の絞り込み（`/`）は Refs ペイン上部のボックスに入力します。

### commit メッセージを編集する

commit メニューの **commit メッセージを編集...**、`Ctrl+G e`、またはパレットの `Git: Commitメッセージを編集...` で、**Commitメッセージを変更**ポップアップが開きます。何が変わるかについては、[ログを使う](log-and-history.md)を参照してください。

| キー | 動作 |
|---|---|
| 概要で `Enter` | 説明に移動する |
| 説明で `Enter` | 改行する。最初の行で `Up` を押すと概要に戻る |
| `Ctrl+Enter`、`Ctrl+S`、または**メッセージを更新** | 新しいメッセージを保存する |
| `Tab` / `Shift+Tab` | 概要、説明、ボタンの間を移動する |
| `Esc`、または**キャンセル** | 変更せずに閉じる |

### commit を修正する

`Ctrl+G Shift+E`（すべての欄）と `Ctrl+G Shift+A`（作成者）で修正ダイアログが開きます。commit メニュー、パレットの `Commit: 修正...` の各行、commit の詳細ペインからも開きます。開き方はすべて [commit を修正する](log-and-history.md#commit-を修正するメッセージ作成者日付)で説明しています。どちらのキーも割り当て変更できます：`git_prefix.amend`、`git_prefix.edit_author`。ダイアログ内のキー：

| キー | 動作 |
|---|---|
| `Tab` / `Shift+Tab` | 欄とボタンの間を移動する |
| 1 行の欄で `Enter` | 次の欄へ移動する。最後の欄では**適用** |
| `Up` / `Down` | 前 / 次の欄へ移動する（説明では最初 / 最後の行から） |
| `Ctrl+Enter`、`Ctrl+S`、または**適用** | 変更した欄を適用する |
| `Esc`、または**キャンセル** | 変更せずに閉じる |
| `Alt+H`、または**履歴をさらに深く変更...** | ダイアログを閉じ、この commit から履歴書き換えワークベンチを開く（履歴書き換えを含むビルドのみ）。欄を編集していた場合は先に確認します（`Esc` で確定）。割り当て変更：`reword.deeper` |

### worktree を作成する

**worktree を作成**（ブランチと tag のメニュー）と**ここに worktree を作成...**（commit メニュー）でダイアログが開きます。フォルダー、任意の新しいブランチ、**新しい worktree をタブで開く**があります。キー：`Tab` / `Shift+Tab` で移動、`Enter` で作成、`Space` でチェックボックスを切り替え、`Ctrl+B` で参照、`Esc` でキャンセルします。

### インタラクティブ rebase

commit メニューの**ここからインタラクティブに rebase...**、または `Git: 選択中のCommitからインタラクティブにRebase...` で、rebase ビューが開きます。複数の commit の cherry-pick も同じビューを使います。[ログを使う](log-and-history.md)を参照してください。

| キー | 動作 |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | 行のアクションを設定する：Pick / Reword / Squash / Fixup / Drop / Edit（Fixup と Edit は rebase のときだけ） |
| `Space` | 行のアクション一覧を開く（`Up` / `Down` で選択、`Enter` で適用、`Esc` で一覧を閉じる） |
| `Up` / `Down` | 選択を移動する |
| `Ctrl+Up` / `Ctrl+Down`、`Alt+Up` / `Alt+Down` | 選択した行を上 / 下へ移動する |
| `a` | autosquash のプレビューをオン / オフにする |
| Reword の行で `Enter` | その commit の新しいメッセージを編集する |
| `Ctrl+S`、`Ctrl+Enter`、または **Rebaseを開始** / **Cherry-pickを開始** | 計画を実行する |
| `Ctrl+R`、または**リセット** | ビューを開いたときの計画に戻す |
| `Esc`、または**キャンセル** | 変更せずに閉じる |

rebase が止まったとき（競合時、または Edit の手順）は、ログの上のバナーに**続行**、**スキップ**、**中止**が表示されます。パレットには `Rebase: 続行`、`Rebase: スキップ`、`Rebase: 中止` があります。

## 検索

`f`、ツールバーの**検索**ボタン、または `ナビゲーション: Commitを検索...` で、ログの上に検索バーが開きます。commit メッセージ全文、作成者の名前とメールアドレス、SHA、参照名を検索します。行を隠すことはなく、一致しない行は暗く表示されます。

| キー | 動作 |
|---|---|
| *（入力）* | 検索テキストを編集する（すべてのキーが入力されます） |
| `Enter` / `Down`、またはバーの `↑` | 次の一致 |
| `Shift+Enter` / `Up`、またはバーの `↓` | 前の一致 |
| `Esc`、またはバーの `✕` | 検索バーを閉じる |

## Diff ビュー

変更されたファイルは、ログの代わりに中央に開きます。[ステージと commit](staging-and-committing.md)を参照してください。

| キー | 動作 |
|---|---|
| `Enter` / `l`、またはダブルクリック（ファイル行） | そのファイルの diff を開く |
| `Esc`、`q`、またはタイトルの `✕` | diff を閉じる |
| `j` / `k`、`Up` / `Down` | スクロールする |
| `PageUp` / `PageDown`、`Home` / `End` | 1 ページスクロールする。先頭 / 末尾へ移動する |
| `h` / `l`、`Left` / `Right` | 横にスクロールする（折り返し時は不可） |
| `n` / `p` | 次 / 前のハンク |
| `m` | インライン / 分割のレイアウト |
| `Shift+H` | ハンクのみ / ファイル全体 |
| `w` | 折り返しのオン / オフ |
| `s` | 空白のみの変更を表示 / 非表示にする |
| `Shift+M` | ミニマップのオン / オフ |
| `Shift+C` | ファイルを読み込む文字コードを選択する（一覧：先頭が自動検出） |
| `e` | 組み込みエディターでファイルを編集する（[エディター](#エディター)を参照） |
| `r` | （Git LFS のファイル）LFS カード、または生のポインター。[Git LFS](lfs.md)を参照 |
| `Shift+E` | （競合したファイル）mergeエディターを開く。[merge 競合の解決](merge-conflicts.md)を参照 |
| `y` / `Ctrl+C` | 選択したテキストをコピーする。選択がない場合は、カーソル位置の行、または選択した行 |
| `.`、またはハンクの右クリック | ハンクメニュー |

### diff からのステージと破棄

これらのキーは、作業中の変更の diff（未ステージまたはステージ済みの一覧から開いたもの）でのみ動作します。破棄は未ステージの一覧でのみ動作します。

| キー | 動作 |
|---|---|
| `Space` | カーソル位置のハンクをステージ（未ステージ）/ ステージ解除（ステージ済み）する |
| `v` | 行の選択を開始 / 解除する。`j` / `k` で広げる |
| `Enter` | 選択した行、またはカーソル位置の行をステージ / ステージ解除する |
| `x` | カーソル位置のハンクを破棄する（先に確認します：`y` / `Enter` で確定、`n` / `Esc` でキャンセル） |
| `Shift+X` | 選択した行、またはカーソル位置の行を破棄する（先に確認します） |

対象になるのは変更された行だけです。選択範囲に含まれる文脈の行は無視されます。2 つ目のハンクにまたがる選択は、最初のハンクにだけ作用します。

## 画像ビュー

Diff ビューまたはファイルビューの画像です。キーは上の表のコンテキスト `image` にあります。[画像](image-view.md)を参照してください。

## ファイルビュー、Blame、ファイル履歴

| キー | 動作 |
|---|---|
| `Esc` / `q` | 閉じる（テキストを選択しているときは、先に `Esc` で選択を解除） |
| `j` / `k`、`Up` / `Down` | 選択を移動する |
| `PageUp` / `PageDown` | 1 ページスクロールする |
| `Home` / `End`、`Shift+G` | 最初 / 最後の行 |
| `h` / `l`、`Left` / `Right` | テキストを横にスクロールする |
| `Enter` | （Blame、ファイル履歴）その行の commit にログで移動する |
| `e` | 組み込みエディターでファイルを編集する |
| `y` / `Ctrl+C` | 選択したテキストをコピーする。選択がない場合は、選択した行（Blame ではコードのみ、ファイル履歴では概要） |

Blame とファイル履歴には専用のキーがあり、上の生成された表（コンテキスト `file_insight` と `history`）とパレットの `Blame: ...` / `ファイル履歴: ...` に載っています。すべて割り当て直せます。詳細は [Blame](log-and-history.md#blame) と [ファイル履歴](log-and-history.md#ファイル履歴) を参照してください。履歴の一覧では単独の文字（`d` `o` `b` `c` `y`）が選択行に作用しますが、Ctrl を使う組み合わせ（`Ctrl+C`、`Ctrl+P`、`Ctrl+G ...`）は常にグローバルな意味を保ちます。

## Commit列

ファイルのビューの左にある列です（[Commit列](interface-tour.md#commit列)を参照）。`Alt+C` はどこからでも列の表示と非表示を切り替えます。ほかのキーは、列がキーボードを持っているときに使えます（`Ctrl+N` の後に列のバッジ、または `表示: Commit列にフォーカス`）。

| キー | 動作 |
|---|---|
| `Up` / `Down`、`PageUp` / `PageDown`、`Home` / `End` | commit を選ぶ |
| `Enter` / `Right` | 選んだ commit のファイル一覧へ進む |
| `Esc` | キーボードをビューに戻す |
| `.`、`Menu`、`Shift+F10` | 選んだ行のメニュー |

そのほかのキー（横スクロールの `Shift+Left` / `Shift+Right`、欄の幅を変える `Tab`、`]`、`[`、`0`）は、上の生成された表のコンテキスト `commit_column` にあります。

## エディター

**編集**、または diff やファイルビューでの `e` で、ファイルの作業ディレクトリ側のコピーに対して組み込みエディターが開きます。[ステージと commit](staging-and-committing.md)を参照してください。

| キー | 動作 |
|---|---|
| *（入力）* | キャレット位置に挿入する（選択範囲を置き換える） |
| `Enter` | 現在の行のインデントを引き継いで改行する |
| `Tab` / `Shift+Tab` | インデント / インデント解除（複数行を選択している場合は、その行） |
| 矢印キー、`Home` / `End` | 移動する。`Home` は最初の空白以外の文字へ、次に 0 桁目へ移動します。`Shift` で選択を広げます |
| `Ctrl+Left` / `Ctrl+Right` | 1 単語ぶん移動する |
| `Ctrl+Home` / `Ctrl+End` | ファイルの先頭 / 末尾 |
| `PageUp` / `PageDown` | 1 ページ上 / 下へ |
| `Ctrl+A` | すべて選択する |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | コピー / 切り取り / 貼り付け |
| `Ctrl+U` / `Ctrl+K` | 行頭 / 行末まで削除する |
| `Ctrl+Z` | 元に戻す |
| `Ctrl+Y` / `Ctrl+Shift+Z` | やり直す |
| `Ctrl+S` | 保存 |
| `Ctrl+F` / `F3` | 検索。検索フィールドでは、`Enter` / `Down` / `F3` で次、`Shift+Enter` / `Up` で前、`Esc` で閉じる |
| `Alt+M` | ミニマップのオン / オフ（**Map**） |
| `Esc` | 閉じる。未保存の変更がある場合は確認します：**保存**（`S` / `Enter`）、**破棄**（`D`）、**キャンセル**（`C` / `Esc`） |

エディターでは `Tab` がインデントになるため、フォーカスは移動しません。ほかの `Ctrl` を使うキー（`Ctrl+P`、`Ctrl+N`）は引き続き動作します。

ステータスバーの**エンコード**、**改行コード**、**インデント**はボタンです。それぞれドロップダウンを開きます
（矢印キーまたはホイール、入力で絞り込み、`Enter` で選択、`Esc` で閉じる）。パレット（`Ctrl+P`）からも同じメニューを開けます：*エディター: エンコードを変更*、*エディター: エンコードを指定して開き直す*、*エディター: エンコードに変換*、*エディター: 改行コードを変換*、*エディター: インデントを変更*。
詳しくは[ステージと commit](staging-and-committing.md)を参照してください。

## Git 操作（`Ctrl+G`）

`Ctrl+G` は、もう 1 つのキーを待ちます。

| キー | 動作 |
|---|---|
| `s` / `u` | 選択したファイルをステージ / ステージ解除する |
| `a` | すべての変更をステージする |
| `x` | すべての変更を破棄する（先に確認します） |
| `c` | commit コンポーザーを開く |
| `Enter` | ステージ済みの変更を、フォームのメッセージで今すぐ commit する。どこからでも使えます（[ステージと commit](staging-and-committing.md#commit-を書く)を参照） |
| `m` | すべての変更をメッセージ付きで stash する。メッセージは小さなダイアログで入力します（空でも構いません）。AI がオンなら、ダイアログの **AI メッセージ**ボタン（ダイアログ内の `Ctrl+G`）がメッセージを書きます |
| `Shift+T` | すべての変更（未追跡ファイルを含む）を、メッセージなしで stash する |
| `o` | stash を pop する：ログで選択している stash の行、なければ最新の stash |
| `p` | pull ボタンの設定どおりに pull する（`git.pull_mode`） |
| `Shift+P` | 既定のリモートへ push する（最初の push では upstream を設定）。または、リモートを尋ねる |
| `Shift+U` | すべての tag を push する |
| `f` | prune 付きですべてのリモートを fetch する |
| `g` | チェックアウト中のブランチの pull request をブラウザーで開く |
| `b` / `t` | HEAD に新しいブランチ / 新しい tag を作成する（[ブランチまたは tag に名前を付ける](#ブランチまたは-tag-に名前を付ける)を参照） |
| `d` | 選択したブランチまたは tag を削除する（先に確認します） |
| `e` | 選択した commit のメッセージを編集する |
| `Shift+E` / `Shift+A` | 選択した commit を修正する（すべての欄）/ その作成者を修正する（[commit を修正する](#commit-を修正する)を参照） |
| `y` | 選択した commit を cherry-pick する |
| `v` | 選択した commit を revert する |
| `Shift+R` | 選択した commit へ reset（mixed）する |
| `r` | 選択したブランチの上に rebase する |
| `Shift+W` | Flow メニューを開く（ブランチワークフローのあるリポジトリ） |
| `Shift+H` | 履歴書き換えワークベンチを開く（履歴書き換えを含むビルドのみ） |
| `l` | LFS メニューを開く。`Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V`：LFS の fetch all / checkout all / pull / push all / prune。[Git LFS](lfs.md)を参照 |
| `i` | ✦ AI：ステージ済みの変更の commit メッセージを書く |
| `w` | ✦ AI：選択した commit、または作業中の変更を説明する |
| `Shift+C` | ✦ AI：作業中の変更を複数の commit に分ける方法を提案する |

AI のキーは、AI がオンのときだけ動作します（[AI アシスタント](ai.md)を参照）。`Esc` で実行中の AI リクエストをキャンセルします。

## 右ペイン（作業中の変更と commit の詳細）

WIP 行を選択していると、右ペインには**未ステージ**と**ステージ済み**の一覧と commit コンポーザーが表示されます。commit を選択していると、commit の詳細とそのファイルが表示されます。

| キー | 動作 |
|---|---|
| `j` / `k`、`Up` / `Down` | 移動する。一方の一覧の末尾で、選択はもう一方へ続く |
| `h` / `l`、`Left` / `Right` | 未ステージ / ステージ済みの一覧へ移動する |
| `Enter` | アクティブな一覧を折りたたむ / 展開する |
| `Space` / `Insert` | 現在のファイルを選択に追加する、または選択から外す |
| `Ctrl+A` | リストのすべてのファイルを選択 |
| `Ctrl+Space` | 選択したファイルをステージ / ステージ解除する、またはフォルダー行を折りたたむ |
| `d` | 選択したファイルの diff を開く |
| `m` | `パス`（フラットな一覧）または`ツリー`（フォルダー） |
| `a` | `すべてのファイルを表示`：変更されたファイルだけでなくツリー全体 |
| `z` | すべてのフォルダーを折りたたむ、または再び展開する |
| `/` | commit の変更ファイルを名前で絞り込む |
| `.`、`Menu`、`Shift+F10` | ファイルのメニュー |
| `c` | commit コンポーザーの概要フィールドにフォーカスする |
| `Shift+A` | **直前のcommitを修正（Amend）**のオン / オフ |
| `Ctrl+N ]` / `Ctrl+N [` | commit メッセージ欄を拡大 / 縮小する |

commit コンポーザーでは次のとおりです。

| キー | 動作 |
|---|---|
| `Enter` | 概要：説明に移動する。説明：改行する |
| `Ctrl+Enter`、`Alt+Enter`、`Ctrl+J` | commit する（`Ctrl+G Enter` ならどこからでも） |
| `Alt+R` | Issue の参照を挿入する |
| `Esc` | コンポーザーを離れる |

ファイルメニューには、ステージ / ステージ解除、変更を破棄、無視、ファイルをStash、ファイル履歴、ファイル Blame、外部diffツールで開く、外部エディターで開く、既定のプログラムでファイルを開く、フォルダーで表示、ファイルパスをコピー、ファイルの変更からパッチを作成、ファイルを編集、ファイルを削除が含まれます。競合したファイルでは、**mergeエディターで開く**と 3 つの**解決**行（Ours、Theirs、Base）から始まります。Git LFS がオンの場合は、LFS の行があります。**変更を破棄**と**ファイルを削除**は先に確認します。

## ウィンドウのボタン

どのボタンにも、対応するキーボード操作があります。

| ボタン | キー |
|---|---|
| タイトルバー左端の 🐙（バージョン情報） | パレットの `ヘルプ: GitPoulpi について` |
| タイトルバーの**開く場所** | `Alt+O`、`Ctrl+Alt+O` |
| タイトルバー右端の `×`（終了） | `q` |
| タイトルバーのバージョン（リリースノート） | パレットの `ヘルプ: リリースノート` |
| タイトルバーの `⚙` の左にある `⌕ 全体検索` | `Ctrl+Alt+F` |
| タイトルバーの `⚙`（設定） | `Ctrl+,` |
| タブ行のワークスペースのチップ | `Alt+W` |
| 最後のタブの後の `+` | `Alt+T` |
| **Radar** タブ | `Alt+R` |
| **ワークスペース** タブ | `Alt+J` |
| ステータスバーの **アクティビティ** | `Ctrl+N Shift+A` |
| ツールバー：**元に戻す**、**やり直す** | `Ctrl+Z`、`Ctrl+Y` |
| ツールバー：**Pull** / その `▾` | `Ctrl+G p` / `Git: Pullのオプション...` |
| ツールバー：**Push** / その `▾` | `Ctrl+G Shift+P` / `Git: リモートへPush...` |
| ツールバー：**ブランチ**、**Tag** | `Ctrl+G b`、`Ctrl+G t` |
| ツールバー：**Stash**、**Pop** | `Ctrl+G Shift+T`、`Ctrl+G o` |
| ツールバー：**LFS**（LFS がオンのとき） | `Ctrl+G l` |
| ツールバー：**Flow**（リポジトリにワークフローがあるとき） | `Ctrl+G Shift+W` |
| ツールバー：**検索** | `f` |
| ツールバー：リンクされた pull request（`GH #12`、`GL !53`） | `Ctrl+G g` |
| ツールバー：プロファイルの一覧（プロファイルが 2 つ以上あるとき） | `リポジトリ: プロファイルを変更...` |

ツールバーについては、[インターフェースツアー](interface-tour.md#ツールバー)で説明しています。

## 設定

`Ctrl+,`、`Ctrl+N ,`、`⚙` の歯車、または `アプリ: 設定...` で設定画面が開きます。

| キー | 動作 |
|---|---|
| `Tab` / `Shift+Tab`、`h` / `l`、`Left` / `Right` | ページ一覧とオプションの間を移動する |
| `j` / `k`、`Up` / `Down` | 選択を移動する |
| `Enter` / `Space` | ページを開く、チェックボックスを切り替える、フィールドを編集する、ドロップダウンを開く、またはボタンを押す |
| `/`、`Ctrl+F` | すべてのオプションを検索する |
| `Alt+Left` / `Alt+Right` | サブタブ（一般 / 一覧）を切り替える |
| `Esc` / `q` | 設定を閉じる。フィールドを編集している間は、`Esc` は編集だけをキャンセルします |
| `?` | 設定画面の上にヘルプを開く |

[設定](settings.md)を参照してください。

## テキストフィールド

すべてのテキストフィールドは、同じキーを共有します。フィルター、検索フィールド、コマンドパレット、プロンプト、commit コンポーザー、WIP 行のフィールド、ダイアログのフィールド、設定のフィールドです。フィールドがキーボードを使っている間は、これらのキーがペインのショートカットより優先されます。

| キー | 動作 |
|---|---|
| 文字 | キャレット位置に入力する（選択範囲を置き換える） |
| `Left` / `Right` | 1 文字移動する（選択がある場合は、その先頭 / 末尾へ移動） |
| `Ctrl+Left` / `Ctrl+Right` | 1 単語ぶん移動する |
| `Home` / `End` | 行頭 / 行末 |
| `Shift` + 上記の移動 | 選択を広げる、または狭める |
| `Up` / `Down`（説明のみ） | 前 / 次の行 |
| `Ctrl+Home` / `Ctrl+End`（説明のみ） | テキストの先頭 / 末尾 |
| `Ctrl+A` | すべて選択する |
| `Backspace` / `Delete` | キャレットの前 / 後ろの文字、または選択範囲を削除する |
| `Ctrl+U` | キャレットから行頭まで（または選択範囲）を削除する |
| `Ctrl+K` | キャレットから行末まで（または選択範囲）を削除する |
| `Ctrl+C` / `Ctrl+X` | 選択範囲をコピー / 切り取りする（パスワードフィールドでは不可） |
| 貼り付け（ターミナル自身の貼り付け） | キャレット位置に挿入し、選択範囲を置き換える |

commit の説明では、`Enter` は改行を入力し、`Ctrl+Enter` は commit します。ターミナルによって `Ctrl+Enter` の送り方は異なるので、`Alt+Enter` と `Ctrl+J` でも commit できます。

一部のコンソールは `Ctrl+文字` を制御文字として送ります（`Ctrl+A` は `0x01`）。GitPoulpi はどちらの形式も理解します。`AltGr` の文字（スイスやフランスのキーボードの `@`、`#`、`€`）も入力できます。

## ツールチップ

ポインターがターゲット（ログのセル、ツールバーのボタン、リンクされた pull request）の上に約 0.6 秒とどまると、ツールチップが表示されます。キー操作、クリック、スクロール、リサイズ、ポップアップやメニューが開くこと、ポインターがターゲットから外れることで、ツールチップは消えます。その後は、ポインターが再び動いたときにだけ戻ります。
