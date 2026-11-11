[English](../config-reference.md) · [Čeština](../cs/config-reference.md) · [Deutsch](../de/config-reference.md) · [Español](../es/config-reference.md) · [Français](../fr/config-reference.md) · [Italiano](../it/config-reference.md) · **日本語** · [한국어](../ko/config-reference.md) · [Nederlands](../nl/config-reference.md) · [Polski](../pl/config-reference.md) · [Português (Brasil)](../pt-BR/config-reference.md) · [Português (Portugal)](../pt-PT/config-reference.md) · [Русский](../ru/config-reference.md) · [Svenska](../sv/config-reference.md) · [Türkçe](../tr/config-reference.md) · [Українська](../uk/config-reference.md) · [简体中文](../zh-CN/config-reference.md)

# 設定リファレンス

`config.toml` のすべてのキーについて、既定値と働きを説明します。ファイルを手で編集する必要はほとんどありません。[設定画面](settings.md)（`Ctrl+,`）が変更に合わせて書き込み、`gitpoulpi config set` でも書き込めます。

## 保存場所と値の決まり方

| | |
|---|---|
| ファイル | `%APPDATA%\GitPoulpi\config.toml`（Windows）、`~/.config/gitpoulpi/config.toml`（Linux）、`~/Library/Application Support/GitPoulpi/config.toml`（macOS）。`gitpoulpi config path` で表示できます。 |
| 別のフォルダー | `GITPOULPI_CONFIG_DIR=<dir>` で設定フォルダー全体（ファイル、`sessions/`、デーモンのポートファイル）を移せます。ポータブルインストール、テスト、自分の設定に触れさせたくないエージェントに使えます。 |
| 優先順位 | 組み込みの既定値、次に `config.toml`、次に `GITPOULPI_*` 変数、最後にコマンドラインフラグの順で、後のものが優先されます。誤った値（ファイル内の選択肢の綴り間違い、不正な変数）はエラーになり、黙って無視されることはありません。 |
| 変数 | キーを大文字にし、ドットをアンダースコアに替えます。`general.auto_prune` は `GITPOULPI_GENERAL_AUTO_PRUNE` です。GitPoulpi が実際に読むキーだけが対象です（下で「*未使用*」とされているキーや、`ui.log_columns_hidden` のようなリストは対象外です）。 |
| フラグ | `--backend git2\|system` は、そのコマンドが起動するデーモンについて `general.default_backend` を上書きします。 |
| トークンと鍵 | このファイルには決して保存されません。ホスティングのトークンと AI の API キーはシステムの認証情報ヘルパー（Git Credential Manager、libsecret / `pass`、キーチェーン）に保存され、ファイルには参照（`token_ref`）だけが残ります。ヘルパーがない場合、トークンの保存はメッセージとともに失敗し、トークンはそのセッションの間だけ保持されます。 |

```
gitpoulpi config path                          # the file, and whether it exists yet
gitpoulpi config get general.auto_stash        # the value, the layer it came from (default/file/env/flag), and whether anything reads it
gitpoulpi --json config get ui                 # a whole section
gitpoulpi config set general.auto_stash always # checked against the schema, then written
```

`config set` はファイルを疎な状態（誰かが設定したキーだけ）に保ち、指定されなかったキーはそのまま残しますが、コメントは消えます。リストとテーブル全体は拒否しますが、`keybindings.*` は例外です（複数のキー入力は空白で区切り、空にすると割り当てを解除します）。`[[profiles]]`、`[[integrations]]`、`[[ai.profiles]]` は設定画面か手作業で編集してください。

表中の **再起動が必要** は、値が起動時に読み込まれることを意味します。それ以外はすぐに反映されます（または次の更新、fetch、設定を閉じたときに反映されます）。

## `[general]`

| キー | 既定値 | 値と効果 |
|---|---|---|
| `general.default_backend` | `"git2"` | `git2`（libgit2。ステージ、merge、rebase、stash、ネットワーク操作は `git` プログラムにフォールバック）または `system`（インストール済みの `git` のみ）。実行中のデーモンは自分のバックエンドを保持するため、**再起動**してください（`gitpoulpi daemon restart`）。 |
| `general.auto_stash` | `"ask"` | ローカルの変更が checkout、pull、merge、rebase を妨げるとき：`ask`（TUI ではダイアログ。CLI と MCP は失敗して `--autostash` を案内）、`always`（stash、実行、pop）、`never`（拒否）。[自動 stash](log-and-history.md)を参照してください。 |
| `general.include_untracked_in_autostash` | `false` | 自動 stash で未追跡ファイルも対象にします。 |
| `general.initial_commits_in_graph` | `5000` | ログが最初に読み込む commit 数（最小 500）。大きくすると、巨大なリポジトリでメモリと描画時間が増えます。 |
| `general.lazy_load_commits_in_graph` | `true` | 末尾付近までスクロールすると、古い commit を読み込みます。オフの場合、ログは最初の件数で止まります。 |
| `general.commits_per_lazy_page` | `1000` | バックグラウンドで読み込む 1 ページあたりの commit 数（最小 50）。 |
| `general.max_commits_in_graph` | `100000` | 遅延読み込みが止まる位置（1 commit あたり約 300 バイト）。`0` は無制限です。 |
| `general.auto_fetch_interval_minutes` | `5` | N 分ごとにバックグラウンドで Fetch All を実行します（0〜60）。`0` はオフです。**再起動が必要。** |
| `general.auto_fetch_interval_unit` | `"auto"` | 設定が間隔を表示する単位：`minutes`、`hours`、`auto`。表示専用で、間隔は常に `auto_fetch_interval_minutes` 分です。`auto`（およびこのキーのない設定）は、分数を割り切れる最大の単位を選びます（60 は「1 時間」、45 は「45 分」）。合わない単位は無視されます。設定は両方のキーを書き込みます。古い GitPoulpi は単位を無視し、分数を読みます。 |
| `general.fetch_on_tab_open` | `true` | タブを開くときに Fetch All を実行します（GitPoulpi がそのリポジトリを 60 秒以内に fetch していた場合はスキップ）。 |
| `general.auto_prune` | `false` | fetch 時に、すでに存在しないリモート追跡ブランチも prune します。 |
| `general.auto_prune_tags` | `false` | fetch 時に、リモートで削除された tag もローカルから削除します（対象はリモートにあった tag だけで、push していない tag は残ります）。Git 2.41 以降。 |
| `general.hosting_refresh_minutes` | `5` | pull request / Issue の一覧とツールバーのチップの更新間隔です（0〜120）。`0` は要求時とイベント時のみです。 |
| `general.hosting_refresh_unit` | `"auto"` | 設定が更新間隔を表示する単位：`minutes`、`hours`、`auto`。`general.auto_fetch_interval_unit` と同じように読まれます。表示専用です。 |
| `general.keep_submodules_up_to_date` | `false` | checkout または pull の後、サブモジュールが合っていない場合に `git submodule update --init --recursive` を実行します。[サブモジュール](submodules.md)を参照してください。 |
| `general.default_branch_name` | `"main"` | Init で作成したリポジトリの最初のブランチ名です。空にすると git に任せます。既存のブランチは変更しません。 |
| `general.delete_orig_files_after_merge` | `false` | 取り込まれた merge の後で、バックアップの `*.orig` ファイルを削除します。 |
| `general.remember_tabs` | `true` | 前回のセッションのタブを開き直します。オフの場合、リポジトリの中で起動するとそのリポジトリだけを開き、リポジトリの外で起動するとホームだけを開きます。**再起動が必要。** |
| `general.radar_enabled` | `true` | [Radar](radar.md)：そのタブ、パレットのコマンド、`Alt+R`、バックグラウンドでの取得。オフにするとすべて非表示になり、そのための取得も行われません。すぐに反映されます。`radar` 機能のないビルドには表示されません。 |
| `general.startup_tab_in_repo` | `"current-repo"` | 起動したフォルダーがリポジトリの中にあるときに、最初に開くタブ：`"current-repo"`（そのリポジトリのタブ。なければ開きます）、`"last"`（前回終了時にアクティブだったタブ）、`"home"`、または `"radar"`。`--repo` で指定したリポジトリは常に自分のタブを開きます。`"last"` は `remember_tabs` がオフ、または記憶されたタブがなければホームになります。`"radar"` は Radar が無効またはビルドに含まれないときホームになります。起動時に一度だけ読み込まれ、反映には再起動が必要です。 |
| `general.startup_tab_outside_repo` | `"home"` | 起動したフォルダーがリポジトリではないとき（たとえばユーザーフォルダー）に、最初に開くタブ：`"home"`、`"last"`、または `"radar"`。`"last"` は `remember_tabs` がオフ、または記憶されたタブがなければホームになります。`"radar"` は Radar が無効またはビルドに含まれないときホームになります。起動時に一度だけ読み込まれ、反映には再起動が必要です。 |
| `general.startup_tab_launcher` | `"last"` | GitPoulpi が作成したデスクトップまたはスタートメニューのショートカット（`--launcher` を渡します）から起動したときに、フォルダーに関係なく最初に開くタブ：`"home"`、`"last"`、または `"radar"`。上の 2 つの設定より優先されます。`"last"` は `remember_tabs` がオフ、または記憶されたタブがなければホームになります。`"radar"` は Radar が無効またはビルドに含まれないときホームになります。起動時に一度だけ読み込まれ、反映には再起動が必要です。 |
| `general.group_worktree_tabs` | `true` | リポジトリにひも付いた worktree のタブを、そのリポジトリのタブのすぐ後ろに並べます：worktree を開いたとき、worktree のタブをドラッグして離したとき（グループの隣に戻ります。リポジトリのタブがグループの先頭で、worktree がその後に続きます）、この設定をオンにしたとき。オフの場合、タブは置いた場所のままです。 |
| `general.undo_depth` | `100` | 元に戻す操作でさかのぼれる操作の数です。`0` は無制限です。 |
| `general.conflict_detection` | `true` | 競合の事前検出：現在のブランチを派生元のブランチへ merge すると競合する場合に、ログにバッジを表示します。バックグラウンドで計算され（メモリ上の merge：ファイル、インデックス、ref は変わりません）、リポジトリごとにオフにできます（`[preferences] conflict_detection`）。比較するベースブランチには専用の一覧があります：[`[conflict_prevention]`](#conflict_preventionベースブランチの一覧)。[merge の競合を解決する](merge-conflicts.md#競合の予防)を参照してください。 |
| `general.dev_mode` | なし（自動） | ソースから GitPoulpi をビルドする人向けで、設定には表示されません。なしの場合：実行中の exe が `<checkout>/target/<profile>/` にあるとき（`target` の隣に `Cargo.toml` と `crates/gitpoulpi-tui` があるとき）にオンになります。`true` / `false` で強制できます。開発モードで使えるようになるのは `ui.launcher_build` だけです。 |
| `general.journal_outside` | `true` | シェルや AI エージェントがブランチ、tag、stash、HEAD に対して行った操作を、外部のものとして印を付けて元に戻す履歴の項目に記録し、`Ctrl+Z` で取り消せるようにします。ファイルの編集は記録されません。`undo_enabled` が必要です。[元に戻す・やり直す](undo-redo.md#gitpoulpi-の外で行われた変更)を参照してください。 |
| `general.undo_enabled` | `true` | `false`：元に戻すための記録を一切行わず（最速）、元に戻す履歴は消去され、元に戻す／やり直すボタンは表示されなくなります。[元に戻す・やり直す](undo-redo.md)を参照してください。 |
| `general.active_profile` | `""` | commit に使う `[[profiles]]` の ID です。空、または名前かメールアドレスが空のプロファイルの場合は、git の `user.name` / `user.email` を使います。 |
| `general.external_editor` | `""` | ファイルの「外部エディターで開く」のコマンドです。例：`code -r {file}`。空の場合：VS Code がインストールされていればそれ、なければシステムの既定。 |
| `general.external_diff_tool` | `""` | 「外部 diff ツールで開く」のコマンドです。`{local}`（commit 済み / HEAD の版）、`{remote}`（作業ツリー / インデックスの版）。空の場合：`git difftool --no-prompt`。 |
| `general.external_merge_tool` | `""` | 競合カード（バイナリ、LFS、または大きすぎるファイル）の「外部 merge ツールで開く」のコマンドです。`{base}`、`{local}`（A、現在）、`{remote}`（B、取り込み）、`{merged}`（ツールが結果を書き込むファイル。ツールが正常終了すると GitPoulpi がステージします）。空の場合：`git mergetool --no-prompt`。 |
| `general.file_explorer` | OS ごと | 「フォルダーで表示」のコマンドです。`{path}`（ファイル）、`{dir}`（そのフォルダー）。既定値：`explorer.exe /select,"{path}"`、`open -R "{path}"`、`xdg-open "{dir}"`。 |

3 つのコマンドテンプレートは、まず引数に分割され（`"..."` は 1 語としてまとめます）、その後でプレースホルダーが置き換えられるため、空白を含むパスも 1 つの引数のままです。シェルは介しません。

```toml
[[profiles]]                # commit identities (Settings > Profiles)
id = "work"
name = "Work"
author_name = "Jane Doe"
author_email = "jane@work.example"
```

## `[ui]`

**ログとグラフ**

| キー | 既定値 | 値と効果 |
|---|---|---|
| `ui.graph_renderer` | `"auto"` | `auto`（ターミナルが描画できる場合は画像、そうでなければ文字）、`image`、`text`。**再起動が必要。** |
| `ui.image_protocol` | `"auto"` | 画像（グラフ、アバター）をターミナルへ送る方法：`auto`、`kitty`、`iterm2`、`sixel`、`none`。`auto` は、kitty、Ghostty、および `kitty_graphics = true` の herdr では Kitty プロトコル、WezTerm、iTerm2、Rio、mintty では iTerm2、Windows Terminal、foot、Konsole、mlterm では sixel（アバターのみ）、tmux、zellij、screen と未知のターミナルでは none になります。**再起動が必要。** |
| `ui.theme` | `"system"` | 配色テーマです。組み込みのスラッグ（`gitpoulpi theme list` で一覧表示）、`<config>/themes/` 内のファイルのスラッグ、または `system`（起動時に検出したターミナルの明るい／暗い背景に従います。古い値 `default` も同じ意味です）。`config set` は未知の名前を拒否します。[テーマ](themes.md)を参照してください。 |
| `ui.graph_color_theme` | `"theme"` | レーンのパレット：`theme`（テーマに従う）または任意のレーンパレット名：`gitpoulpi`、`light`、`material`、`serie`、および他のテーマのパレット。 |
| `ui.graph_lane_background` | `"off"` | `off` または `row_tint`（グラフの背後にレーン色の淡い帯と、その端のバーを描きます）。**アプリ: グラフの行の色付けを切り替え**（`Ctrl+N Shift+T`）で切り替わり、**アプリ: グラフのカラーテーマを変更...**（`Ctrl+N Shift+Y`）で `ui.graph_color_theme` を選べます。 |
| `ui.graph_row_tint_strength` | `"medium"` | `strong`、`medium`、`subtle`（`row_tint` のときのみ）。 |
| `ui.toolbar_position` | `"bottom"` | `top`、`bottom`、`left`、`right`。`left` と `right` は縦の列として描画し、80 列未満のウィンドウでは `bottom` に戻ります。 |
| `ui.dialog_geometry` | `"session"` | `session`、`saved`、`never`：移動またはサイズ変更したダイアログ（タイトルバーか枠をドラッグ、または `Ctrl+N m`）がその位置を保つかどうか。すべてのダイアログに共通の設定で（各ダイアログがそれぞれの位置とサイズを保ちます）、GitPoulpi を閉じるまで（既定）、次回の起動まで保存（すべてのダイアログがデータフォルダーの 1 つの `dialog_geometry.txt` に入ります）、または保持しない（どのダイアログも既定の位置とサイズで開く）。 |
| `ui.toolbar_align` | `"start"` | `start`、`center`、`end`：ツールバー上でのボタンの位置（行では左・中央・右、列では上・中央・下）。 |
| `ui.toolbar_pr_pinned` | `true` | プルリクエストのボタンをツールバーの末端に固定します。`false` にすると他のボタンと一緒に並びます。 |
| `ui.statusbar_position` | `"bottom"` | `top`（リポジトリのタブの上）または `bottom`。 |
| `ui.tabs_position` | `"top"` | `top`、`bottom`（ウィンドウの最終行、ステータスバーの下）、`left`（左端の列。80 列未満では上に残ります）。 |
| `ui.graph_show_stashes` | `true` | ログに stash ごとの行を表示します。 |
| `ui.ref_connector` | `true` | 参照のピルからその commit へ細い線を引きます。 |
| `ui.column_resize` | `"line"` | Log、Radar、diff・ファイル・Blame の各ビューの commit 列で、列の境界をどうつかむか。`line`：各境界は 1 セル幅の隙間で、ヘッダーに小さな目印があります。ポインターが境界にある間（ヘッダー上でも表のどこでも）、アクセント色の細い線が全高にわたって描かれ、その線に沿ってどこからでもドラッグしてサイズを変えられます。`header`：隙間も目印もなく、境界の次のヘッダーセルの最初の文字が光り、そこからドラッグしてサイズを変えます（ヘッダーのみ）。キーボードのサイズ変更キーとパレットの項目は、どちらのモードでも同じです。 |
| `ui.ref_tree` | `true` | Refs サイドバーで、`/` を含むブランチ名とタグ名をフォルダーにまとめます（`y` で切り替え）。 |
| `ui.graph_ghost_ref` | `true` | Log の Ref 列は、自分の ref を持たないホバー中の commit に、列に余裕があれば、所属ブランチ（なければリモートブランチ）の名前を淡いピルで表示します。tag は仮表示されません。削除されたブランチは、それを merge した merge commit の件名から推定され、取り消し線付きで描かれます。ピルにホバーすると種類が表示されます。 |
| `ui.reveal_on_ref_click` | `true` | Refs ペインで参照（ブランチ、tag、リモートブランチ、stash）を 1 回クリックすると、ログがその commit までスクロールして行が点滅します。commit の選択は動きません。 |
| `ui.log_wip_counts_in_message` | `true` | 変更列が非表示のとき、WIP のメッセージに WIP の変更数を表示します。 |
| `ui.log_columns_layout` | `"fit"` | ログの列の並べ方。`fit`：表示中のすべての列がペインにぴったり収まり、`message` だけが伸縮します。ペインが狭いときは列ごと省かれます。`scroll`：`message` を含め列は幅を保ち、何も省かれず、ペインより広いときは表が横にスクロールします。 |
| `ui.log_column_widths_scroll` | 空 | `scroll` レイアウトの列幅（名前 -> セル数）。`message` も指定できます（`fit` レイアウト用の `ui.log_column_widths` には `message` を入れません）。レイアウトが `scroll` のときに列をドラッグ、サイズ変更、内容に合わせると書き込まれます。 |
| `ui.log_columns_hidden`、`ui.log_columns_order`、`ui.log_column_widths` | 空 | ログの列（`ref`、`graph`、`message`、`author`、`date`、`sha`、`changes`）です。列を非表示にしたり、ドラッグしたり、幅を変えたりすると書き込まれます。`message` は非表示にできません。 |
| `ui.commit_column_columns_visible`、`ui.commit_column_columns_order`、`ui.commit_column_columns_widths` | 空 | diff の横にある [commit 列](interface-tour.md#commit列)の欄で、ログの欄とは別に持ちます。表示する欄を名前で（`ref`、`graph`、`message`、`author`、`date`、`changes`、`sha`。空ならレーン、メッセージ、作成者、日付）、同じ名前で指定する左から右への順序（空なら今挙げた順。省いた名前はその順で後ろに続きます）、ドラッグした幅を欄ごとのセル数で（省略した欄は自動）保存します。欄を選んだり、移動したり、幅を変えたりすると書き込まれます。`message` は非表示にできません。これらのキーができる前に書かれたセッションファイルは、一度だけここに読み込まれます。 |
| `ui.sidebar_sections_hidden` | 空 | 非表示にした左ペインのセクション（`local`、`remote`、`worktrees`、`stashes`、`tags`、`pull_requests`、`issues`、`submodules`）で、ペインの右クリックメニューまたはパレットから書き込まれます。存在するセクションが少なくとも 1 つは必ず表示されます。 これは既定値です。各リポジトリは独自にセクションを非表示にでき（そのリポジトリの状態ファイルに保存）、このキーは独自の選択がないリポジトリに適用されます。*すべてのリポジトリに使う*がこのキーを書き込みます。 |
| `ui.sidebar_sections_scope` | `"repository"` | 左ペインのセクションを誰の設定にするか：`repository`（各リポジトリが独自の設定を持ち、`ui.sidebar_sections_hidden` が既定値になります）または `global`（すべてのリポジトリが `ui.sidebar_sections_hidden` を使い、変更もそこに書き込まれます）。切り替えても、どちらの設定も失われません。`global` の間も、リポジトリごとの設定は状態ファイルに残ります。設定：*左ペインのセクション*。 |
| `ui.language` | `"auto"` | インターフェースの言語：`auto`（システムのロケール、なければ英語）、`en`、`fr`、`es`、`pt-BR`、`pt-PT`、`it`、`de`、`zh-CN`、`ja`、`ko`、`ru`、`uk`、`pl`、`cs`、`nl`、`sv`、`tr`。**即時反映。** `GITPOULPI_LANG` が優先されます。このビルドに含まれていない言語は英語で表示されます。**アプリ: 言語を変更...**（`Ctrl+N g`）でも、ライブプレビュー付きで設定できます。CLI と MCP の出力は英語のままです。 |
| `ui.translate_git_terms` | `false` | `false`：commit、push、pull、stash、tag... はどの言語でも英語のままです。`true`：選んだ言語の git 用語を使います（日本語ではコミット、プッシュ、スタッシュなど）。**即時反映。** |
| `ui.git_terms_english` | `false` | ja、ko、zh-CN のみ：git 用語（stage、unstage、staged、unstaged...）を音訳ではなく英語で表示します。`ui.translate_git_terms` が `true` の間は効果がありません。**即時反映。** |
| `ui.mouse_enabled` | `true` | `false` にすると、マウス（とテキスト選択）をターミナルに任せます。**再起動が必要。** |
| `ui.mouse_scroll_lines` | `3` | ホイール 1 ノッチあたりのログの行数（1〜20）。**再起動が必要。** |
| `ui.kitty_keyboard` | `"auto"` | `auto` はターミナルに kitty キーボードプロトコルを要求します（`Ctrl+,`、`Ctrl+Enter`、`Ctrl+Shift+N` が届くようになります）。`off` は要求しません。**再起動が必要。** |
| `ui.logo_color` | `""` | 「GitPoulpi について」ウィンドウのロゴの配色で、スラッグ（`violet`、`orange`、`teal`、`ocean`、`magenta`、`candy`、`coral`、`green`、`cyan`、`indigo`、`lavender`、`sun`、`mint`、`slate`）で指定します。空または未知の値はビルドの既定（`lavender`）になります。ロゴをクリックするか、「GitPoulpi について」ウィンドウで色を選ぶと書き込まれます。 |

**アバター**（既定ではオフです。オンにすると、作者のメールアドレスの SHA-256 ハッシュが Gravatar に送られ、接続済みの GitHub / GitLab アカウントにも問い合わせます）

| キー | 既定値 | 値と効果 |
|---|---|---|
| `ui.avatars` | `"off"` | `graph`（画像グラフの commit の円の中）、`author`（作成者列）、`off`。古い `ui.show_avatars`（`true`）は、`ui.avatars` がない間は `author` として読まれます。 |
| `ui.avatar_sources` | `"gravatar_hosting"` | 検索の順序：`hosting_gravatar`、`gravatar_hosting`、`gravatar`、`hosting`。GitLab ホストには、接続済みのプロファイルを通してのみ問い合わせます。 |
| `ui.avatar_fallback` | `"identicon"` | `identicon`（ローカルで生成）または `initials`。画像プロトコルのないターミナルでは、常にイニシャルのバッジが使われます。 |

**diffとエディターの表示**

| キー | 既定値 | 値と効果 |
|---|---|---|
| `ui.diff_layout` | `"inline"` | `inline` または `split`（diff 内の `m`）。 |
| `ui.diff_hunks_only` | `true` | `false` にすると、ファイル全体を変更箇所付きで表示します（`H`）。 |
| `ui.diff_show_whitespace` | `true` | 空白のみの変更を表示します（`s`）。 |
| `ui.diff_word_wrap` | `false` | 長い diff の行を折り返します（`w`。設定 > diffとエディター）。 |
| `ui.minimap` | `false` | diffとエディターのスクロールバーの横にミニマップを表示します（`M`）。 |
| `ui.mermaid_detail` | `refs` | ログの Mermaid gitGraph としてコピーの詳細度：`structure`、`refs`、`subjects`、`full`（ポップアップでの選択が保存され、`log --format mermaid` の既定になります）。 |
| `ui.mermaid_orientation` | `lr` | そのグラフの方向：`lr`（左から右）または `tb`（上から下）。 |
| `ui.mermaid_fence` | `false` | コピーする Mermaid スクリプトを `mermaid` のコードフェンスで囲みます。 |
| `ui.mermaid_subject_width` | `40` | Mermaid の ID に入る commit の件名を、この文字数で切り詰めます（`subjects`、`full`）。 |
| `ui.color_depth` | `"auto"` | `auto`、`truecolor`、`256`、`16`：描画に使う色数です。`auto` は、Apple Terminal（256）、Linux コンソール（16）、`COLORTERM` のない `*-256color` ターミナル（256）を除いて truecolor です。即時反映。[テーマ](themes.md)を参照してください。 |
| `ui.input_color` | `"theme"` | テキスト入力欄の背景：`theme`（テーマ自身の色）、`gray`、`blue`、`slate`、`dark`、`custom`。 |
| `ui.input_color_custom` | `"#2a2a2a"` | `custom` の色（`#rrggbb`）。 |
| `ui.composer_files` | `"expanded"` | AI commit コンポーザーが、各提案のファイルを `expanded`（展開）または `collapsed`（折りたたみ）で開きます。 |
| `ui.blame_side` | `"right"` | Blame ビューで注釈列（アバター、作成者、経過時間、要約）を置く側：`right`（既定）または `left`。 |
| `ui.blame_width` | `46` | Blame の注釈列の幅（セル単位、24 から 90）。境界をドラッグするか、ビューで `<` / `>` を押します。新しい幅はここに保存されます。 |
| `ui.pr_terminology` | `"auto"` | `auto`（GitHub では Pull Request、GitLab では Merge Request）、`pull_request`、`merge_request`。 |
| `ui.hide_finished_pr_chip` | `true` | チェックアウト中のブランチの最新の pull request または merge request が、クローズまたはマージされたあとに、ツールバーのチップを非表示にします。代わりに薄い「+ PR」チップ（新しく作成）が表示され、ブランチに何もないときと同じ見た目になります。`false`：状態にかかわらず、常に最新のリクエストのチップを表示します。 |

diff の中で切り替えたトグルはここにすぐ保存され、その後に開くすべての diff で使われます。

**日付**：`ui.date_format`（`"YYYY-MM-DD"`。Blame / ファイル履歴の日付列）、`ui.date_time_format`（`"YYYY-MM-DD HH:mm"`。ログの日付列）、`ui.date_verbose_format`（`"D MMMM YYYY HH:mm"`。commit の詳細、Date のツールチップ、*コピー > 日付*）。トークンは `YYYY` `YY` `MMMM` `MMM` `MM` `M` `DD` `D` `dddd` `ddd`（曜日、フル / 省略形）`HH` `mm` `ss` で、それ以外の文字はそのままコピーされます。月名と曜日名は `ui.language` に従い、日付の数字の後ろに書く月は、その言語に属格がある場合（ロシア語、ウクライナ語、ポーランド語、チェコ語）は属格になります。`D MMMM YYYY` の並びは、日本語、中国語、韓国語では年が先頭になります（`2024年3月5日`）。`ui.date_locale`（既定は `"system"`、または `"english"`）は、言語にかかわらず名前を英語のままにします。

**エージェント、ターミナル、タブのフォルダー**

| キー | 既定値 | 値と効果 |
|---|---|---|
| `ui.allow_agent_view_control` | `true` | ウィンドウがエージェントからの `ui show` / `ui propose` を受け付けるかどうかです（設定 > エージェントと MCP > *AI エージェントによる表示の操作を許可*）。オフの場合、どのエージェントも届きません。[MCP とエージェント](mcp-and-agents.md)を参照してください。 |
| `ui.launch` | `"off"` | GitPoulpi のウィンドウが開いていないときに、エージェントがウィンドウを開いてよいかどうか：`off`、プリセット（`kitty`、`foot`、`alacritty`、`wezterm`、`wezterm-tab`、`ghostty`、`gnome-terminal`、`xdg-terminal-exec`、`windows-terminal`、`windows-terminal-tab`、`tmux`、`zellij`、`cmux`）、または `custom`。 |
| `ui.launch_command` | `""` | `custom` のテンプレート：`{repo}`、`{session}`、`{gitpoulpi}`。引数に分割され、シェルは介しません。 |
| `ui.tab_dir_follow` | `"buttons"` | `buttons`（タブバーに*終了して cd*を表示）または `auto`（終了のたびにシェルのフォルダーを変更し、新しいタブでは既定の*開く場所*のエントリを実行）。[タブのフォルダーを追従する](following-the-tab-folder.md)を参照してください。 |
| `ui.terminal_launch` | `"auto"` | 旧仕様です。下の説明と[開く場所](open-in.md)を参照してください。旧「ここでターミナル」がターミナルを開く方法：`auto`（`ui.launch` のプリセット、なければ `$TERMINAL`、`xdg-terminal-exec`、Windows Terminal、Terminal）、プリセット、または `custom`。 |
| `ui.terminal_launch_command` | `""` | `custom` のテンプレート。`{repo}` はフォルダーです。 |
| `ui.open_in` | *(なし)* | [開く場所](open-in.md)の一覧で、`label` と `command` を持つテーブルの配列 `[[ui.open_in]]` です。なしは「まだ設定されていない」ことを意味し、一覧は旧 `ui.terminal_launch*` キー（`auto` 以外の値は最初のエントリになります）、OS のファイルエクスプローラー、環境から検出したマルチプレクサとターミナル（herdr、tmux、zellij、WezTerm、kitty、Windows Terminal など）、`PATH` で見つかったエディターから組み立てられます。設定画面から最初に保存したときに一覧が書き込まれます。`open_in = []` は空の一覧です。`config set` では設定できません（ファイルを編集してください）。 |
| `ui.open_in_default` | `""` | 「既定の開く場所を実行」が実行するエントリのラベルです。空または未知の場合は最初のエントリになります。 |
| `ui.launch_terminal` | `"auto"` | デスクトップのショートカット（設定 > ランチャー）と、Windows での開き直しが GitPoulpi を起動するターミナルです。`auto` はインストール済みのものから選びます（Windows：Windows Terminal、なければ WezTerm、Alacritty、通常のコンソール。Linux：`xdg-terminal-exec`、なければ最初に見つかったターミナル。macOS：iTerm2、Ghostty、WezTerm、kitty のいずれかがあればそれ、なければ Terminal）。`default` は OS 自身の既定です。`windows-terminal`、`wezterm`、`alacritty`、`console`、`kitty`、`foot`、`ghostty`、`gnome-terminal`、`konsole`、`iterm`、`apple-terminal` のようなターミナル ID も指定できます。 |
| `ui.launcher_build` | `"current"` | 開発モードのときだけ有効です（`general.dev_mode` を参照。そのときだけ設定 > ランチャーに表示され、それ以外は無視されます）。`"current"`：ショートカットとコマンドエイリアスは、それらを作成したビルドを起動します。`"newest"`：ソースの `target/dist` と `target/release` のうち最も新しくビルドされたものを、クリックのたびに判定して起動します。どちらのビルドを起動しても、ショートカットは書き換えられません。 |
| `ui.command_alias` | `""` | ターミナルから GitPoulpi を起動するコマンドの名前です（例：`gp`。設定 > ランチャー、`gitpoulpi alias create`）。空：なし。英数字と `-`、`_`、`.` のみ。**作成：コマンドのエイリアス**ボタンが使う名前にすぎず、作成するまで何も存在しません。 |
| `ui.command_alias_add` | `""` | `gitpoulpi --add` を実行する 2つ目のコマンドの名前です（例：`gpa`）。すでに起動中の GitPoulpi に、現在のフォルダーを新しいタブとして開きます（設定 > ランチャー、`gitpoulpi alias create gpa --add`）。空：なし。名前の規則は同じで、**作成**ボタンで `ui.command_alias` と一緒に作られます。 |
| `ui.relaunch_in_terminal` | `true` | Windows のみ：exe がダブルクリックで専用のコンソールに起動された場合、GitPoulpi は `ui.launch_terminal` で開き直して終了します。オフの場合はそのコンソールにとどまります。 |
| `ui.reuse_window` | `true` | GitPoulpi のショートカット（`gitpoulpi launch`）をクリックすると、新しく起動せず、すでに開いている GitPoulpi のウィンドウを表示します。 オフ：クリックのたびに新しいウィンドウを起動します。 |

**`ui.open_in` のコマンド**は、まず引数に分割され（空白、`'...'`、`"..."`）、その後で各引数の中で変数が置き換えられるため、空白を含むフォルダーも 1 つの引数のままです。コマンド自身がシェルを起動しない限り（`sh -c '... "$GITPOULPI_CWD"'`）、シェルは介しません。変数：`${cwd}`（タブのフォルダー）、`${repo}`（リポジトリのルート）、`${name}`（フォルダー名）、`${branch}`（detached のときは空）、`${web_url}`（`origin` リモートが GitHub または GitLab のプロジェクトであればその Web ページ、そうでなければ空。これを使うエントリは、空の場合は実行を拒否します）、`${env:NAME}`（未設定なら空）。古い `{repo}` は今も `${cwd}` を意味します。それ以外の `${...}` はその名前を挙げたエラーになります。例：

```toml
[ui]
open_in_default = "Herdr: new tab"

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

旧仕様：`ui.terminal_launch` と `ui.terminal_launch_command` は今も読み込まれますが（`ui.open_in` がない間は一覧の元になります。`custom` は `{repo}` を `${cwd}` に書き換えた「Terminal（custom）」に、プリセット ID はそのプリセットになります）、`ui.open_in` に置き換えられています。

### 画像

| キー | 既定値 | 意味 |
|---|---|---|
| `ui.image_view` | `true` | 画像であるバイナリファイル（PNG、JPEG、GIF、WebP、BMP など）を、Diff ビューとファイルビューで画像として表示します。オフの場合は、ふつうのバイナリの通知になります。 |
| `ui.image_view_max_mib` | `32` | これ（MiB）より大きい画像はデコードされず、代わりに情報カードが表示されます。 |
| `ui.image_view_max_unit` | `"auto"` | 設定が上限を表示する単位：`mib`、`gib`（1024 MiB）、`auto`。表示専用で、上限は常に `image_view_max_mib` MiB（1〜1024）です。 |
| `ui.image_view_text_fallback` | `"halfblock"` | 画像プロトコルのないターミナルで：`halfblock`（色付きの半ブロック文字）または `card`（情報カードのみ）。 |
| `ui.image_view_pixelated_from` | `"2x"` | これ以上の拡大で、滑らかにせず四角いピクセルで表示します：`1x`、`2x`、`never`。 |
| `ui.image_view_sync` | `true` | 変更前と変更後のペインで、ズームとスクロール位置を共有します。 |
| `ui.image_editors` | *(なし)* | 「Edit in」の画像エディターで、`label` と `command` を持つテーブルの配列 `[[ui.image_editors]]` です。なしは「まだ設定されていない」ことを意味し、一覧はこのマシンで検出されたエディター（`PATH` 上のプログラム、Flatpak のインストール、Windows の既知のインストール先、macOS の `/Applications`）、続いて OS の既定アプリになります。設定画面から最初に保存したときに一覧が書き込まれます。`image_editors = []` は空の一覧です。`config set` では設定できません（ファイルを編集してください）。 |
| `ui.image_editor_default` | `""` | 「Edit in default」が実行するエディターのラベルです。空または未知の場合は最初のエントリになります。 |

**`ui.image_editors` のコマンド**は、まず引数に分割され（空白、`'...'`、`"..."`）、その後で各引数の中で変数が置き換えられるため、空白を含むパスも 1 つの引数のままです。シェルは介しません。変数：`${file}`（画像の絶対パス）、`${dir}`（そのフォルダー）、`${name}`（ファイル名）、`${repo}`（リポジトリのルート。一時コピーでは空）、`${env:NAME}`（未設定なら空）。それ以外の `${...}` はその名前を挙げたエラーになります。作業ツリーのファイルではない画像（commit 時点の版、インデックス、削除されたファイル）は**読み取り専用の一時コピー**として開かれるため、エディターがリポジトリに触れることはありません。コマンドラインを持たないエディター（Microsoft Store 版の Affinity Photo、一部の Photoshop のインストール）はプリセットに含まれません。カスタムのエントリを書いてください。例：

```toml
[ui]
image_editor_default = "GIMP"

[[ui.image_editors]]
label = "GIMP"
command = "gimp ${file}"

[[ui.image_editors]]
label = "Paint.NET"
command = "\"C:\\Program Files\\paint.net\\paintdotnet.exe\" ${file}"
```

**まだ使われていません**（受け付けられ、保存されますが、何も読み取りません）：`ui.graph_colorization`（設定から削除済みで、読み込み時には受け付けます）、`ui.date_word_format`（設定から削除済みで、読み込み時には受け付けます）。古い `ui.diff_view_mode` は一度だけ読まれ、`diff_layout` / `diff_hunks_only` になります。

## `[editor]`

| キー | 既定値 | 値と効果 |
|---|---|---|
| `editor.tab_size` | `4` | タブストップあたりの桁数（1〜8。エディターと diff）。 |
| `editor.eol` | `"auto"` | 組み込みエディターが書き込む改行コード：`auto`（ファイルのものを維持）、`lf`、`crlf`。 |
| `editor.syntax_highlighting` | `true` | エディターで言語ごとに色を付けます。 |
| `editor.line_numbers` | `true` | エディターと diff に行番号を表示します。 |
| `editor.follow_selection` | `true` | エディターを開いている間、右ペインで別のファイルを選ぶとそのファイルもエディターで開きます。未保存の変更がある場合は先に保存するかどうかを尋ねます。編集できないファイルは通常のビューで開きます。commit のファイルを選ぶと、エディターを離れてその diff が開きます。 |

## `[merge]`

merge 競合エディター（[merge の競合を解決する](merge-conflicts.md)）の設定です。設定 > diffとエディターに表示されます。

| キー | 既定値 | 意味 |
|---|---|---|
| `merge.side_by_side_min_width` | `100` | エディター領域の幅（桁数）がこの値以上のとき、A と B を左右に並べて表示します（長い行は横にスクロールします：Shift+ホイール、Shift+矢印）。これより狭い場合は、タブで一度に片側ずつ表示します。最小値は `40` で、`0` は常に左右に並べます（40 桁未満の通知は残ります）。エディターを開いている間は Refs と Status のパネルが隠れるため、エディター領域は通常ウィンドウ全体になります。 |

## `[git]`、`[credentials]`、`[ssh]`

| キー | 既定値 | 値と効果 |
|---|---|---|
| `git.pull_mode` | `"merge"` | Pull（ボタン、`Ctrl+G p`、`Git: Pull`）の動作：`merge`（可能なら fast-forward）、`ff_only`、`rebase`。常に git に明示的に渡されます。古い `fetch_all` は `merge` として読まれます。 |
| `git.push_behavior` | `"direct"` | `direct` は既定のリモートへ push し、`ask` は先にリモートのメニューを表示します。 |
| `git.merge_fast_forward` | `"never"` | 自分で開始した merge（ブランチメニュー、ドラッグ＆ドロップ、パレット、`gitpoulpi merge`）が、fast-forward できるブランチをどう扱うか：`never` は常に merge commit を作り（`--no-ff`）、`allowed` は可能なら fast-forward し（git の既定で、`merge.ff` が効きます）、`only` は fast-forward するか拒否します。リポジトリ自身の選択（設定 > Merge オプション、`repo-state.toml` の `merge_fast_forward`）が優先され、CLI の `--ff`、`--no-ff`、`--ff-only` はその両方に優先します。即時に反映されます。 |
| `git.default_encoding` | `"utf-8"` | 有効な UTF-8 **ではない**テキスト（ファイルの内容、blame、diff、`encoding` ヘッダーのない commit メッセージ）のエンコーディングです。任意の WHATWG ラベルを指定できます（`windows-1252`、`shift_jis` など）。表示のみで、ステージ時に再エンコードすることはありません。リポジトリごとに上書きできます。 |
| `git.lfs_enabled` | `true` | Git LFS を表示して操作します。リポジトリごとに上書きできます。[Git LFS](lfs.md)を参照してください。 |
| `git.lfs_autofix_filters` | `true` | LFS を使うリポジトリの pull または push の前に、LFS フィルター / pre-push フックがなければインストールします（`git lfs install --local`、このリポジトリのみ）。[Git LFS](lfs.md)を参照してください。 |
| `git.autocrlf`、`git.eol` | `"unset"`、`"native"` | *未使用*：リポジトリの `core.autocrlf` / `core.eol` は git 自身の設定で、設定 > Git とエンコードで編集します（または `gitpoulpi repo-config`）。 |
| `credentials.https_mode` | `"credential_helper"` | fetch、pull、push、clone の HTTPS 認証：`credential_helper`（git のもの）、`prompt`（ヘルパーを無視し、git がターミナルで尋ねるため、バックグラウンドのデーモンでは失敗します）、`none`（公開リモートのみ）。古い `encrypted_file` は `credential_helper` として読まれます。 |
| `ssh.key_type` | `"ed25519"` | *鍵ペアを生成*で作る種類：`ed25519`、`rsa`（4096 ビット）、`ecdsa`（NIST P-256）。ファイルは `gitpoulpi_<type>` になります。SSH 自体は常にお使いの ssh-agent と ssh の設定を使います。 |
| `ssh.private_key_path`、`ssh.public_key_path` | 未設定 | *鍵ペアを生成*が記録します。`ssh.private_key_path` が設定されていると、fetch、pull、push、clone は `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes` を付けて git を実行します。秘密鍵のパスが空の場合は `ssh.public_key_path` が同じ働きをします（ssh が対応するエージェントの鍵を選びます）。環境の `GIT_SSH_COMMAND` / `GIT_SSH`、または git 設定の `core.sshCommand`（リポジトリごとの固定、`gitpoulpi ssh-command set --key`）が優先されます。転送のたびに読み込まれるため、変更は次の転送から反映されます。 |

## `[[integrations]]`（ホスティングと Issue トラッカー）

設定 > 連携、または `gitpoulpi hosting integrations add` が書き込みます。すべての種類にクライアントがあります（パーソナルアクセストークンのみで、OAuth はありません）。

```toml
[[integrations]]
id = "corp"                         # what --id takes; the token is stored under integration:corp
kind = "gitlab_self_managed"        # github | github_enterprise | gitlab | gitlab_self_managed | ...
url = "https://gitlab.example.com"  # the API root is derived from it
token_ref = "integration:corp"      # a reference, never the token
name = "Corp GitLab"                # optional display name
account = "me@example.com"          # optional, see below
enabled = true                      # false: ignored for host matching
```

セルフホストのサーバーは平文の http でも構いません。`url = "http://localhost:3000"` は入力したとおりに保持されます（スキームとポートはすべてのリクエストに使われ、そのホストを指すリモートはこのプロファイルに一致します）。平文の http で送ったトークンは暗号化されずに流れるため、http はローカルまたは信頼できるネットワークでのみ使ってください。

`kind` には `azure_devops`、`azure_devops_server`、`bitbucket`、`bitbucket_data_center`、`jira_cloud`、`jira_data_center`、`trello`、`gitea`、`forgejo`、`linear`、`youtrack`、`redmine`、`gerrit` も指定できます（Gitea と Forgejo は 1 つのクライアントを共有し、Forgejo のプロファイルの既定は `https://codeberg.org` です）。Jira、Trello、Linear、YouTrack、Redmine は Issue トラッカーのみで、Gerrit はレビューのみです（そのチェンジが pull request で、Issue はありません）。Bitbucket Cloud には Issue トラッカーがありません。

`account` はトークンと対になる、秘密ではない ID で、ほとんどの種類では空で省略されます。`jira_cloud` と `bitbucket` では Atlassian アカウントのメールアドレス（Bitbucket のアカウントが空の場合は、ワークスペースまたはリポジトリのアクセストークンを意味し、Bearer として送られます）、`trello` では API キー、`gerrit` ではユーザー名です（トークンはその HTTP パスワードで、HTTP の Basic 認証として送られます）。秘密はあくまでトークンだけです。`azure_devops` では `url` は `https://dev.azure.com/<organization>` で、`https://<organization>.visualstudio.com` の URL は同じ組織として読まれます。`azure_devops_server` では、`https://tfs.example.com/tfs/DefaultCollection` のようなコレクションのルートです。

`user`（接続済みのアカウント）と `ssh`（プロファイルの鍵の選択）は自動で書き込まれます。トークンは `GITPOULPI_TOKEN_<ID>`（ID を大文字にし、英数字以外は `_` に）から取得し、なければ保存済みのトークン、それもなければ、そのホストを使うプロファイルが 1 つだけの場合に限り認証情報ヘルパーから取得します（GitHub と GitLab のプロファイルのみで、他の種類は認証情報ヘルパーを読みません）。詳細：[ホスティングと Issue トラッカーの連携](hosting-integration.md)。古い `[[hosting]]` テーブルは*使われません*。

`local_markdown`（Markdown ノートのフォルダーを Issue の取得元にするもの）では、`url` はフォルダー（各リポジトリからの相対パスまたは絶対パス）で、トークンはありません。省略可能な `[integrations.local]` テーブルで、`statuses`（ボードの列の順序。既定は `todo`、`doing`、`done`）、`done_status`（既定は `done`）、`reference_prefix`（`LOC` なら `LOC-12` と表示）、`me`（「自分に割り当て」が指す人）を設定します。[ローカル Issue と Obsidian](local-issues.md)を参照してください。

### `[integrations.radar]`（アカウント全体の一覧）

コードホストのプロファイル（GitHub、GitLab、Gitea、Forgejo、Azure DevOps、Bitbucket Data Center、Gerrit。Bitbucket Cloud を除く）の
省略可能なテーブルです。なければすべてのキーがオフです。`gitpoulpi hosting integrations radar` が書き込み、
`gitpoulpi hosting radar` と Radar が読み取り、設定 > 連携（接続済みのプロファイルの Radar ブロック）で編集できます。

```toml
[integrations.radar]
review_requests = true    # requests waiting for my review, every repository
open_requests = false     # every open request of my namespaces
include_drafts = false    # keep draft requests in these lists
assigned_issues = true    # open issues assigned to me, every repository
namespaces = ["acme"]     # orgs / groups to search for open requests
review_mode = "ask"       # ask | clone | api
```

| キー | 既定値 | 意味 |
|---|---|---|
| `review_requests` | `false` | リポジトリをクローンしているかどうかに関係なく、あなたのレビューを求めている未完了のリクエスト |
| `open_requests` | `false` | 自分の名前空間と、所属する各組織／グループ（または `namespaces`）の未完了のリクエストすべて。「トークンから見えるすべて」ではありません |
| `include_drafts` | `false` | これをオンにしない限り、ドラフトのリクエストはこれらの一覧から除外されます |
| `assigned_issues` | `false` | 自分に割り当てられた未完了の Issue（すべてのリポジトリ） |
| `namespaces` | `[]` | `open_requests` で検索する組織／グループ。空なら自分の名前空間と所属するすべて（GitHub：最大 12 件を検索、GitLab：10 グループと自分が所有する 10 プロジェクト、Gitea、Forgejo、Gerrit：10 件。Azure DevOps はプロジェクト名、Bitbucket Data Center はプロジェクトキー、Gerrit はプロジェクトパスのプレフィックスを指定） |
| `review_mode` | `"ask"` | リポジトリをクローンしていないリクエストをレビューするときの動作：`ask`、`clone`（クローンしてからレビュー）、`api`（ホストの API 経由でレビュー） |

回答はプロファイルごとに、`config.toml` の隣の `radar-account.json` に 5 分間キャッシュされます。このファイルはキャッシュなので、
削除しても問題ありません。

### トラストストア（`config.toml` にはありません）

各連携と API キー方式の AI プロファイルが何について信頼されたか（アドレス、アカウント ID、ピン留めした証明書の鍵、平文 http を許可したかどうか）は、ユーザー専用のプライベートデータフォルダーの `trust.toml`（`<data>/private/trust.toml`：Windows では `%APPDATA%\GitPoulpi\private\`、Linux では `~/.local/share/gitpoulpi/private/`）に保存されます。これは意図的に `config.toml` の一部にしていません。設定を編集できる人が、編集によって信頼を広げられてはならないからです。変更は `gitpoulpi hosting integrations trust` と `gitpoulpi ai trust` で行います（[セキュリティチェック](hosting-integration.md#セキュリティチェック)を参照）。ファイルは手作業ではなく、GitPoulpi が書き直します。

## `[mcp]`（AI エージェントに許可すること）

| キー | 既定値 | 値と効果 |
|---|---|---|
| `mcp.allow_mutations` | `false` | オフ：何かを変更する MCP ツールは拒否されます。読み取り、`ui_*`、提案は引き続き動作します。`true` にすると、エージェントがリポジトリを変更できます。 |
| `mcp.mutation_allowlist` | `[]` | 空でない場合：ここに挙げたツール名だけが変更を行えます（`["stage", "commit"]`）。 |
| `mcp.allow_ui` | `true` | `false`：ウィンドウを動かす `ui_*` ツールを拒否します（`ui_state` / `ui_viewers` は除く）。 |
| `mcp.ui_control` | `"ask"` | `ask`（ウィンドウに確認用のチップを表示）、`auto`（ユーザーが操作中でなければ適用）、`off`。サーバーコマンドの `--ui-control` でエージェントごとに上書きできます。`gitpoulpi ui show` にも適用されます。 |
| `mcp.allow_ui_propose` | `false` | `ui_propose_*` / `gitpoulpi ui propose` を許可します。ユーザー自身が確定する、入力済みのダイアログです。 |
| `mcp.allow_history_purge` | `false` | `config.toml` のみ（設定に行はありません）。`true` にすると、エージェント向けにツール `rewrite_purge` が一覧に表示されます（`allow_mutations` も必要です）。オフでは、エージェントにはこのツールがまったく見えません。消去は、履歴の書き換えの古い履歴と、その undo、reflog を完全に削除するためです。CLI と TUI は、この設定の制限を受けません。 |
| `mcp.enabled` | `true` | `false`：`gitpoulpi mcp serve` と `gitpoulpi-mcp` は、このキーを挙げたメッセージを出して起動を拒否します。設定 > エージェントと MCP > *MCP サーバー*。エージェントがサーバーを起動するときに読み込まれます。 |
| `mcp.bind` | `"127.0.0.1:8808"` | アドレスを指定しないときに `--http` が待ち受けるアドレスです（`--http ADDR` / `--bind` が優先）。`--allow-remote` がない限りループバックのみです。空、または古い `"stdio"` は既定値を意味します。設定 > エージェントと MCP > *MCP サーバーのポート*でポートを編集でき、ホストは維持されます。 |
| `mcp.http_auth` | `true` | `--http` は、`Authorization: Bearer <token>`（`gitpoulpi mcp token` のトークン）を持つリクエストにだけ応答します。`false` にするとチェックがなくなり、サーバーは stderr に警告を出し、`--allow-remote` は拒否されます。設定 > エージェントと MCP > *HTTP 認証*。 |
| `mcp.tls` | `"off"` | `off`、`self_signed`（プライベートデータフォルダーの下に一度だけ生成される証明書。起動時に SHA-256 フィンガープリントを出力）、`files`（`tls_cert` と `tls_key`）。TLS をオンにすると、サーバー自身が TLS を終端し、平文のリスナーは存在しません。設定 > エージェントと MCP > *TLS*。 |
| `mcp.tls_cert` | `""` | `tls = "files"` で使う PEM 形式の証明書チェーン。 |
| `mcp.tls_key` | `""` | `tls = "files"` で使う PEM 形式の秘密鍵。 |
| `mcp.behind_proxy` | `false` | エンドポイントの前段にあるトンネルまたはリバースプロキシが暗号化を担っていると、あなたが申告します。サーバーの内部からは確認できないため、TLS なしの `--allow-remote` にはこの設定が必要です。プロキシとサーバーの間は、そのネットワーク上で平文です。設定 > エージェントと MCP > *トンネルまたはリバースプロキシの背後*。 |
| `mcp.allow_sensitive_staging` | `false` | `config.toml` のみ。オフでは、MCP のステージツールは `allow_sensitive` を提供せず、フラグの付いたファイル（`[privacy]` を参照）を追加するステージは、エージェントにあなたへ尋ねるよう伝えるメッセージとともに拒否されます。オンでは、エージェントが `allow_sensitive` を渡せます。CLI と TUI はこの設定の制限を受けません。あなたに尋ねます。 |

`allow_mutations`、`mutation_allowlist`、`allow_ui` は MCP サーバーだけを制限します。CLI はユーザー自身のシェルなので、これらの対象ではありません。[MCP とエージェント](mcp-and-agents.md)を参照してください。

## `[ai]`（組み込み AI、既定ではオフ）

| キー | 既定値 | 値と効果 |
|---|---|---|
| `ai.enabled` | `false` | スイッチです。オフの場合、AI に関するものはどこにも表示されず、何も送信されません。 |
| `ai.default_profile` | `"default"` | アクションがプロファイルを指定しないときに使うプロファイルの ID です。 |
| `ai.global_instructions` | `""` | すべてのアクションのプロンプトに追加されるテキストです。`${repo}`、`${uilang}` などを使えます（[AI > プロンプト変数](ai.md#プロンプト変数)を参照）。アクションごとの指示とリポジトリの指示も同様です。 |
| `ai.actions.<action>.profile` | `""` | 1 つのアクションのプロファイルです。空は既定を使います。アクション：`commit_message`、`explain_changes`、`stash_message`、`pull_request_description`、`commit_composer`、`conflict_resolve`、`code_review`、`split_commit`、`rewrite_script`、`rewrite_rules`。ワークスペース（`workspaces.toml` の `[workspace.ai_instructions]` と `[workspace.ai_overrides.<action>]`）とリポジトリは、これらの指示に追加したり置き換えたりできます：[AI](ai.md#指示)を参照してください。 |
| `ai.actions.<action>.instructions` | `""` | そのアクションだけに追加されるテキストです。 |
| `ai.review.run_in_repository` | `"profile"` | `profile`、`yes`、`no`：コードレビューのドラフトのローカルエージェントが、その実行でリポジトリ内で作業するかどうか（AI プロファイルの `workdir` より優先）。リポジトリ固有の値も設定できます（`gitpoulpi ai review-context --repo`）。[AI > レビューのコンテキスト](ai.md#レビューのコンテキスト)を参照してください。 |
| `ai.review.include_claude_md` | `true` | コードレビューのドラフトにリポジトリの `CLAUDE.md` を渡します（ルート、リクエストの対象ブランチで読み取り。8 KB で切り詰め）。 |
| `ai.review.include_agents_md` | `true` | `AGENTS.md` についても同様です。 |
| `ai.review.extra_file` | `""` | コードレビューのドラフトに渡す追加ファイル1つ：リポジトリルートからの相対パス、絶対パス、またはコードホスト上のファイルへのリンク（一致する連携プロファイル経由で、なければ匿名で読み取り）。すべてのファイルの合計は 16 KB で切り詰められます。 |
| `ai.privacy.redact_secrets` | `true` | リクエスト内のスキャナーが認識するすべてのシークレット（トークン、秘密鍵のブロック、URL 内のパスワード、ランダムに見える `password = ...` の値）を、外に出る前に `[REDACTED:<rule>]` に置き換えます。オフの間、設定 > AI には赤い警告が表示されます。 |
| `ai.privacy.exclude_sensitive_files` | `true` | `privacy.sensitive_patterns` に一致するすべてのファイルの diff を、送信内容から除外します。リポジトリのフォルダーで動くローカルエージェントは、それでもファイルを自分で読めます：[AI > プライバシー](ai.md#プライバシー)を参照。 |
| `ai.limits.conflict_bytes` | `40000` | 「解決案を提案」が送信する 1 つの merge 競合（ベース、現在、受け入れ側のブロックの合計）の最大バイト数。超えると GitPoulpi は失敗せず、警告を出して、*そのまま送信*（このリクエストのみ）、*設定を開く*、*キャンセル*を提示します。競合が切り詰められることはありません。この上限は GitPoulpi 独自の安全策で、モデルの上限ではありません。モデルに余裕があるときは引き上げてください。1000 から 50000000 の範囲に収められ、設定 > AI > 上限で変更できます。以下の他の上限も同じ方式で、すべてそのグループにあります。 |
| `ai.limits.diff_bytes` | `60000` | commit メッセージ、説明、stash の名前、pull request の説明、commit プラン、commit の分割が持てる diff の最大バイト数、およびコードレビュー全体の diff の最大バイト数。超えても黙って切り詰めることはありません。TUI では*AI の上限を超えています*が*すべて送信*（このリクエストのみ、何も切り詰めない）、*切り詰めて送信*（末尾を除外、従来どおり）、*設定を開く*、*キャンセル*を提示します。CLI と MCP は切り詰めて `truncated` で報告します（`--send-all` ですべてを送信）。1000 から 50000000 の範囲です。 |
| `ai.limits.review_file_bytes` | `12000` | コードレビューでの 1 ファイルの diff の最大バイト数。超えると、同じ確認と同じ 2 つの回答になります。*切り詰めて送信*はそのファイルを、目に見える注記つきで短くします。範囲は同じです。 |
| `ai.limits.review_text_bytes` | `4000` | レビュー対象の pull request の説明の最大バイト数。超えると、*切り詰めて送信*は説明を `[text cut]` の注記つきで切り詰めます。範囲は同じです。 |
| `ai.limits.review_threads_bytes` | `6000` | すでにあるレビューコメントの一覧（モデルが繰り返さないように渡されます）の最大バイト数。超えると、*切り詰めて送信*は一覧を切り詰めます。範囲は同じです。 |
| `ai.limits.review_context_file_bytes` | `8192` | コードレビューに渡す 1 つのプロジェクトコンテキストファイル（`CLAUDE.md`、`AGENTS.md`、追加ファイル）の最大バイト数。超えると、*切り詰めて送信*はそのファイルを注記つきで短くします。範囲は同じです。 |
| `ai.limits.review_context_bytes` | `16384` | すべてのコンテキストファイルを合わせた最大バイト数。超えると、*切り詰めて送信*は上限をまたぐファイルを短くし、それ以降のファイルを除外します。範囲は同じです。 |
| `ai.sent_preview.<action>` | `true` | アクションごとに 1 つのキー（`commit_message`、`explain_changes`、`stash_message`、`pull_request_description`、`commit_composer`、`conflict_resolve`、`code_review`、`split_commit`、`rewrite_script`、`rewrite_rules`）：リクエストを送る前に「送信する内容」を表示します。ダイアログの「今後表示しない」が `false` を書き込み、設定 > AI > *送信前に内容を表示...* でオンに戻せます。 |

リポジトリ固有の `[preferences.ai_review]` テーブル（`repo-state.toml` 内、またはクローンしていないリポジトリでは `remote-reviews.toml` のエントリ）にも同じキーを指定でき（すべて省略可能）、加えて `no_extra_file = true` があります。これは `ai.review.extra_file` の内容にかかわらず、そのリポジトリでは追加ファイルを使いません。未設定のキー、未設定の `extra_file`、存在しない `no_extra_file` は全体の値を継承します。`extra_file` にファイルを設定するとスイッチが解除され、逆の場合も同様です。

```toml
[[ai.profiles]]
id = "default"
name = "Default"
kind = "open_ai"        # open_ai | anthropic | open_router | open_ai_compatible | local_agent
model = "gpt-5"         # empty = the kind's default
base_url = ""           # empty = the kind's default (Ollama: http://localhost:11434/v1)
token_ref = "ai:default"  # where the API key is stored; never the key
command = ""            # local_agent only: the exact program ("claude", an alias, a wrapper, a path)
harness = ""            # local_agent only: claude | codex | gemini | opencode | cursor-agent | copilot | pi | qwen | aider | goose | crush | amp | other; empty = inferred from `command`
args = []               # local_agent only, e.g. ["-p", "--model", "haiku"]
run_mode = "headless"   # local_agent only: headless (default) | terminal (the agent's interactive screen in a virtual terminal)
terminal_args = []      # local_agent only: the arguments of the terminal mode (args keeps the headless set)
workdir = "default"     # local_agent only: default | dedicated | repository (default = dedicated folder in terminal mode, GitPoulpi's folder in headless mode)
timeout_secs = 0        # local_agent only: seconds before the agent is stopped; 0 = 120 headless, 180 terminal

[ai.profiles.env]       # local_agent only, optional: variables set for that process only
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"   # ${env:NAME} = one of GitPoulpi's own variables
```

`harness` は、設定 > AI が提示する候補（ヘッドレス実行の引数、オプション、モデルのフラグと一覧、環境変数）を `command` とは独立に選びます。どちらのキーも、空の場合はファイルから省略されます。環境変数の値はログに記録されず、秘密のように見える場合は設定画面で隠されます。

設定 > AI には、`harness` / `command` / `args` / `env` のプリセット（一般的なエージェント CLI と、そのヘッドレス用フラグ、モデルのフラグ、役に立つ環境変数）と、`kind` / `base_url` / `model` のプリセット（ホスト型とローカルの HTTP プロバイダー）があります。[AI > AI を設定する](ai.md#ai-を設定する)を参照してください。

*Default* という名前のプロファイルは、なければ作成されます。古い `[[ai.presets]]` / `[ai.defaults]` のファイルは読み込み時に移行され、`[[ai.legacy_presets]]` に保持されます。そのほかすべて：[AI アシスタント](ai.md)。

## `[repos]`、`[session]`

| キー | 既定値 | 値と効果 |
|---|---|---|
| `repos.clone_root` | `""` | Clone ダイアログが開始する場所です。空の場合は `%USERPROFILE%\source\repos` または `~/src`。 |
| `repos.clone_start_source` | `"last"` | Clone ダイアログが最初に開くソースです。`last`（既定。空でも同じ）= 最後に閉じたときに表示していたソース、`url` = 常に「URL でクローン」、または連携プロファイルの id（`[[integrations]]` エントリの `id`）。なくなった、無効にした、または未接続のプロファイルは「URL でクローン」になります。最後のソース自体は状態であり、ここではなく設定フォルダーの `clone-source.toml` に保存されます。 |
| `repos.browse_root` | `""` | Browse が開く場所です。空の場合は最後に選んだフォルダー、なければホーム。 |
| `repos.show_hidden` | `false` | フォルダー選択で隠しフォルダーを表示します（`Alt+H` で切り替わり、保存されます）。 |
| `repos.show_siblings` | `false` | フォルダー選択の左のツリーに、開いている各階層のほかのフォルダーも、本来の階層のまま並べます（`Alt+S` で切り替わり、保存されます）。 |
| `session.per_repo_state_dir` | `"in_git_dir"` | GitPoulpi がリポジトリごとに覚えておくもの（ref のフィルター、折りたたみの状態、個人用のワークフローの上書き、ホスティングのキャッシュ）の置き場所：`in_git_dir` = `<git dir>/gitpoulpi`（すべての worktree で共有され、`git status` には現れません）、`in_worktree` = `<repo>/.gitpoulpi`（チームが `workflow.toml` を commit できる唯一の場所）、`user_data` = `<OS のデータフォルダー>/repos/<name>-<id>`（Windows では `%APPDATA%\GitPoulpi\data`、Linux では `$XDG_DATA_HOME/gitpoulpi`、macOS では `~/Library/Application Support/GitPoulpi`）。古い値 `".gitpoulpi"` は `in_worktree` と同じ意味です。変更しても何も元には戻りません：従来の `<repo>/.gitpoulpi` にあるデータは、初めて必要になったときに選んだ場所へ移されます。 |

## `[log]`（アプリケーションログ）

アクティビティログの「アプリケーション」タブに表示される内容と、「診断情報をコピー」が引用する内容です。詳細：[何かが失敗したとき](troubleshooting.md#何かが失敗したとき)。

| キー | 既定値 | 値と効果 |
|---|---|---|
| `log.level` | `"info"` | 記録される最も詳細な行：`error`、`warn`、`info`、`debug`、`trace`。アクティビティログのレベルボックスで変更できます（ここにも保存されます）。 |
| `log.retention_days` | `7` | 保存するログファイルの日数（当日を含む）：1〜3650（10 年。範囲外の値は丸められます）。設定でどの単位で表示しても、常に日数です。それより古いファイルは GitPoulpi の起動時に削除されます。1 日分のファイルは `log.max_file_mb` でも上限が決まるため、最悪の場合は約 `2 x max_file_mb x retention_days` です。 |
| `log.retention_unit` | `"auto"` | 設定が `log.retention_days` を表示する単位：`days`、`weeks`、`months`、`years`、`auto`。表示専用で、ファイルを保持する期間は変わりません。1 週間は 7 日、1 か月は 30 日（暦の月ではないため、いつ適用しても日数は同じです）、1 年は 365 日です。`auto`、およびこのキーができる前に書かれた設定は、日数を割り切れる最大の単位を選びます（14 日は「2 週間」、45 日は「45 日」）。日数を割り切れない単位（`retention_days = 10` に `weeks`）は無視され、日数が表示されます。設定は両方のキーを書き込みます。古い GitPoulpi は単位を無視し、日数を読みます。 |
| `log.max_file_mb` | `5` | 1 日分のファイルがこのサイズに達すると、1 つの `.old` バックアップにローテーションされます。そのため 1 日分がこの 2 倍を超えることはありません。 |
| `log.max_file_unit` | `"auto"` | 設定がサイズを表示する単位：`mib`、`gib`（1024 MiB）、`auto`。表示専用で、サイズは常に `max_file_mb` MiB（1〜1024）です。 |
| `log.timings` | `true` | アクティビティログの「Git 操作」タブに、各操作にかかった時間を表示します（`Push - 123 ms`：リポジトリを開く、更新、fetch、pull、push、commit、checkout、merge、rebase、stash、clone）。時間はウィンドウが計測し、リクエストからデーモンの応答までなので、IPC の往復も含みます。オフにすると、時間と、成功した操作のエントリを非表示にします。失敗は常に一覧に出ます。 |
| `log.commit_count` | `"system-git"` | ステータスバーの `読み込み済み / 総数` の総数を求める方法です。`"system-git"` は `git rev-list --count` を実行します（プロセス 1 つ。通常のマシンでは最速：git が commit-graph ファイルを読みます）。`"libgit2"` は libgit2 の revwalk でプロセス内で数え、何も起動しません：コマンドラインのプロセスがどれも起動に時間がかかるとき（たいていは、リアルタイムスキャンを行うウイルス対策ソフトがそれぞれを検査するため）に選びます。50,000 件の commit で、バックグラウンドで約 200 ms かかります（system-git では 12 ms）が、プロセスは起動しません。`"off"` は数えず、バーには読み込み済みの commit だけを表示します。すぐに反映されます。パスで絞り込んだ件数は、つねにシステムの git を使います。 |
| `log.extended` | `false` | 「拡張ログ」：「Git 操作」タブにバックグラウンドの操作（状態をポーリングする定期更新、自動 fetch）も所要時間つきで表示します。これらは常にアプリケーションログへ `debug` レベルで書き込まれます。 |
| `log.use_mailmap` | `true` | Log、blame、ファイル履歴で、各作成者をリポジトリの `.mailmap`（および `mailmap.file`）が指定する名前で表示します。デーモンが git に `%aN`/`%aE` を問い合わせるか（システム git バックエンド）、同じファイルを自分で適用するため（libgit2 バックエンド）、答えは同じです。保存済みの commit は変更されず、何も書き換えられません。Log を作成者で検索すると、表示されている名前に一致します。即時に反映され、Log は 2 秒間隔のポーリングで読み直します。 |

ファイルは OS の状態フォルダーにある `gitpoulpi-YYYY-MM-DD.log` です（Linux では `~/.local/state/gitpoulpi/logs`、Windows と macOS では `<cache>\logs`、`GITPOULPI_CONFIG_DIR` を使うときは `<config>/cache/logs`）。トークン、パスワード、`Authorization` ヘッダーは、何かが書き込まれる前にマスクされます。

## `[search]`（ワークスペース検索）

`gitpoulpi search`（とその MCP ツール）が対象とする範囲です。詳細：[CLI のワークスペースと検索](cli-reference.md#ワークスペースと検索)。

| キー | 既定値 | 値と効果 |
|---|---|---|
| `search.commit_limit` | `200000` | リポジトリごとに、検索インデックスがメッセージをメモリに保持する commit の数（新しいものから、すべてのブランチ、リモートブランチ、tag をまたぐ）。1 件あたり約 100 バイトにメッセージ分が加わります。これより長い履歴はそこで打ち切られ、結果は `partial` と示されます。`0` で commit 検索がオフになります（ブランチ、tag、stash、worktree は引き続き検索されます）。即時に読まれ、次の検索から変更が反映されます。 |

## `[rewrite]`（履歴の書き換え）

`gitpoulpi rewrite` と履歴書き換えワークベンチが保護するものです。詳細：[履歴の書き換え](history-rewrite.md)。

| キー | 既定値 | 値と効果 |
|---|---|---|
| `rewrite.protected_branches` | `["main", "master", "develop", "release/*"]` | すでに push 済みの場合に、書き換えが手を付けないブランチです（名前、または `*` が任意のテキストに一致するパターン）。実行で明示した場合（`--include-protected BRANCH`。push では `--include-all-protected`。ワークベンチではブランチごとのチェックボックス）は別です。リポジトリの `workflow.toml` が `protected` と指定した基点ブランチにも追加されます。書き換えの push も同じようにスキップします。これは GitPoulpi 独自の保護で、強制 push を受け付けるかどうかは、ホスト側のブランチ保護が決めます。即時に読み込まれ、変更後の次のプレビューから反映されます。 |
| `rewrite.external_scripts` | `["shell"]` | スクリプトルールが使える外部スクリプトの言語です。`"shell"`（bash または sh。PowerShell スクリプトでは pwsh または powershell）、`"python"`、`"lua"`（インストール済みの Lua。内蔵の Lua は常にオンで、記載は不要）のいずれかを指定します。**外部スクリプトはサンドボックスで保護されず、あなたの権限で動作します。** 一覧にない言語は、ワークベンチではグレー表示、CLI、MCP、デーモンでは拒否されます（`scripts_disabled`）。一覧にあっても、リポジトリや共有レシピから来た外部スクリプトは、あなたが信頼した後でなければ実行されません（`gitpoulpi rewrite script trust FILE`、またはワークベンチでの確認）。信頼はスクリプトの内容のハッシュに結び付くため、変更されたスクリプトは再度確認を求めます。Python をオンにするには `config.toml` を編集します（リストは `config set` では設定できません）：`[rewrite]` に `external_scripts = ["shell", "python"]` と書きます。即時に読み込まれます。 |
| `rewrite.script_timeout_ms` | `5000` | 外部スクリプトが 1 件の commit に答えるまでの制限時間です。超過するとプロセスツリー全体が終了され、書き換えは失敗します（`script_failed`、commit を明示）。何も動いていません。`0` は既定値として扱われます。即時に読み込まれます。 |
| `rewrite.script_timeout_unit` | `"auto"` | 設定が制限時間を表示する単位：`milliseconds`、`seconds`、`auto`。表示専用で、制限は常に `script_timeout_ms` ミリ秒です。設定で指定できるのは 1〜600000（10 分）で、`0` は引き続き既定値として扱われます。即時に読み込まれます。 |
| `rewrite.script_total_timeout_s` | `3600` | 外部スクリプトの 1 回の処理全体にかけられる制限時間です。`0` は既定値として扱われます。即時に読み込まれます。 |
| `rewrite.script_total_timeout_unit` | `"auto"` | 設定が制限時間を表示する単位：`seconds`、`minutes`、`hours`、`auto`。表示専用で、制限は常に `script_total_timeout_s` 秒です。設定で指定できるのは 1〜86400（1 日）で、`0` は引き続き既定値として扱われます。即時に読み込まれます。 |

## `[updates]`（新しいリリースの確認）

タイトルバーのバージョンチップが、新しいリリースについて行うことです。詳細：[更新](updates.md)。

| キー | 既定値 | 値と効果 |
|---|---|---|
| `updates.mode` | `"notify"` | `"notify"` は 1 日に 1 回、バックグラウンドで GitHub に新しいリリースがあるかを問い合わせ、あればバージョンチップを `vX.Y.Z が利用可能` にします。`"auto"` は、このビルドがリリースアーカイブから来たものであれば、更新のダウンロードとステージも自動で行います（再起動の前には確認します）。それ以外のビルドでは `"notify"` と同じです。`"off"` はリクエストを一切送りません。すぐに反映されます。 |
| `updates.channel` | `"stable"` | `"stable"` は正式リリースだけを対象にします。`"prerelease"` はリリース候補（`-rc.1`）も案内します。すぐに反映されます。 |
| `updates.repo` | `"Poulpi-Lab/GitPoulpi"` | リリースを確認する github.com 上の `owner/name` です。そのリポジトリが非公開の間は、github.com のトークン（設定 > 連携）が保存されていない限り GitHub は 404 を返し、確認は静かに失敗してアプリケーションログに 1 行だけ書き込まれます。 |

管理者は、マシン全体のポリシーファイルまたは Windows レジストリで、`mode`、`channel`（`release_channel` として）、`repo` のほか、いくつかの更新オプションを設定できます。ポリシーが設定したキーは `config.toml` より優先され、設定画面には**管理者が設定しています**と表示されます。ポリシーの `mode` は上限で、より低い値は選べますが、より高い値は選べません。[エンタープライズ配布](enterprise-distribution.md#ポリシーファイルとレジストリ)を参照してください。

## `[privacy]`（機密ファイル）

| キー | 既定値 | 値と効果 |
|---|---|---|
| `privacy.sensitive_patterns` | `[".env", ".env.*", "!.env.example", "!.env.sample", "!.env.template", "*.pem", "*.key", "id_rsa*", "id_ed25519*", "id_ecdsa*", "*.p12", "*.pfx", "*.jks", "*.keystore", "*.crt", "*.cer", "*.der", "*.kdbx"]` | gitignore 形式のパスパターンです。`/` のないパターンは任意の深さのファイル名に、`/` のあるパターンはパス全体に一致します（先頭の `/` はルートに固定します）。`*` と `?` は 1 つのフォルダー内にとどまり、`**` はフォルダーをまたぎます。`!` は先のパターンが一致したものを再び許可し、最後の一致が優先されます。大文字小文字は区別しません。ステージのガードと `ai.privacy.exclude_sensitive_files` が使います。 |
| `privacy.staging_guard` | `true` | パターンに一致するファイル、または認識されたシークレットを内容に含むファイルをステージすると、先に確認されます（TUI）。`--allow-sensitive` を付けない限り拒否されます（CLI）。`mcp.allow_sensitive_staging` がエージェントに `allow_sensitive` を渡すことを許している場合を除き、拒否されます（MCP）。テキストファイルは 1 つあたり 1 MiB まで、合計 32 MiB まで読み取ります。バイナリファイルは名前だけで照合されます。[ステージと commit](staging-and-committing.md#機密ファイル)を参照してください。 |

## `[conflict_prevention]`（ベースブランチの一覧）

競合の事前検出が、現在のブランチの派生元（ベース）として扱ってよいブランチは、パターンの一覧で決まります（[merge の競合を解決する](merge-conflicts.md#競合の予防)を参照）。一覧には名前を付けて共有できます。編集は 設定 > 競合の予防 で行います。

| キー | 既定値 | 値と効果 |
|---|---|---|
| `conflict_prevention.default_list` | `""` | 独自の一覧を持たないリポジトリが使う一覧の名前。空、またはその名前の一覧がない場合は、組み込みの `**`（すべてのブランチ）です。確認のたびに読み直されるため、変更は再起動なしで開いているリポジトリに反映されます。 |
| `conflict_prevention.lists` | なし | 名前付きの一覧。1 つにつき 1 つのテーブルです：`[conflict_prevention.lists.release]` に `patterns = ["origin/release/**", "!origin/release/old"]`。`patterns` が空ならすべてのブランチを許可します。一覧を編集すると、それに従うすべてのリポジトリに反映されます。 |

リポジトリがどの一覧に従うかは、その `repo-state.toml` にあります（下記）。優先順位：リポジトリ自身のパターン、従っている名前付きの一覧、`default_list`、`**` の順です。

## `[keybindings]`

TUI のショートカットを割り当て直します。設定 > キーボードとマウスで編集できます。

```toml
[keybindings.global]
git_prefix = "Ctrl+B"            # default Ctrl+G
nav_prefix = "Alt+n"             # default Ctrl+N
settings   = ["Ctrl+,", "F2"]    # a list = alternatives
tab_menu   = []                  # unbound

[keybindings.git_prefix]         # the key pressed after the git prefix
stage_file = "z"
```

`[keybindings.<context>] <action> = "<chord>"`：値はそのアクションの既定値を置き換え、行を削除すると既定値に戻ります。キー入力の書き方：`Ctrl+Shift+N`、`Alt+n`、`F2`、`PageDown`、`Space`、`,`、`P`（= `Shift+p`）。不正なエントリが何かを妨げることはありません。TUI はそれを無視して既定値を保ち、ステータスバーでそのことを知らせます。コンテキスト、アクション ID、既定値、競合のルール：[キー割り当て](keybindings.md)。

## リポジトリごとの設定：`.gitpoulpi/repo-state.toml`

リポジトリの中に保存されます。このフォルダーは自分自身を無視する（`*` だけの `.gitignore`）ため、変更として表示されることはありません。TUI と CLI が書き込み、手で編集することはほとんどありません。

| キー | 内容 | 設定方法 |
|---|---|---|
| `[preferences] encoding` | ここでの `git.default_encoding` を上書き | 設定 > Git とエンコード |
| `[preferences] lfs_enabled` | ここでの `git.lfs_enabled` を上書き | 設定 > Git とエンコード |
| `[preferences] profile` | ここで commit に署名する `[[profiles]]` の ID。なしの場合は `general.active_profile` | ツールバーのプロファイルのコンボボックス、**リポジトリ: プロファイルを変更...** |
| `[preferences] autocrlf`、`eol` | *まだ使われていません* | 設定 > Git とエンコード |
| `[preferences] push_after_commit` | commit のたびに、ブランチをアップストリームへ push します（なければオフ） | 設定 > リポジトリ別の設定 > commit |
| `[preferences] skip_hooks_by_default` | commit フォームの「フックをスキップ」が最初からオンになります（なければオフ） | 設定 > リポジトリ別の設定 > commit |
| `[preferences] squash_merge` | ローカルブランチの merge が squash になります：ステージだけで merge commit は作りません（なければオフ） | 設定 > リポジトリ別の設定 > merge オプション |
| `[preferences] template_apply`、`template_strip_comments` | 空の commit フォームにテンプレートを入れる。commit 時に `#` のコメント行を取り除く（どちらもなければオフ） | 設定 > リポジトリ別の設定 > メッセージテンプレート |
| `[preferences] conflict_detection` | このリポジトリの競合の事前検出。なければ `general.conflict_detection` に従います | 設定 > リポジトリ別の設定 > 競合の予防 |
| `[preferences] conflict_base_branches` | リポジトリ自身のベースブランチのパターン（1 行に 1 つ。**独自の一覧**として引き続き使われます） | 設定 > リポジトリ別の設定 > 競合の予防 |
| `[preferences] conflict_base_source` | ベースブランチの取得元：`default`（全体の既定の一覧）、`list`（`conflict_base_list`）、`own`（`conflict_base_branches`）。なければ、独自のパターンがあればそれ、なければ既定 | 設定 > リポジトリ別の設定 > 競合の予防 |
| `[preferences] conflict_base_list` | 取得元が `list` のときにこのリポジトリが従う `[conflict_prevention.lists.<name>]`。削除された一覧は既定に戻ります | 設定 > リポジトリ別の設定 > 競合の予防 |
| `commit-template.txt`（`repo-state.toml` の隣のファイル） | commit テンプレート。git の `commit.template` がこれを指します | 設定 > リポジトリ別の設定 > メッセージテンプレート |
| `[preferences.ai_instructions]` | `text`、`mode = "add" \| "replace"`：このリポジトリの AI への指示 | 設定 > リポジトリ別の設定 > AI |
| `[preferences.ai_overrides.<action>]` | ここでの 1 つのアクションの `profile`、`instructions` | 設定 > リポジトリ別の設定 > AI |
| `default_remote` | ワンクリックの Push/Pull/Fetch が使うリモート | リモートのメニュー、`gitpoulpi remote default <name>` |
| `hosting_integration` | ここに固定された `[[integrations]]` のプロファイル | `gitpoulpi hosting use --id` |
| `pulls_source`、`issues_source` | PR / Issue の一覧が読むプロバイダーのプロジェクトで、`{ integration, project, filter }` の形です。`filter` は省略可能で、プロバイダー固有の追加条件（Jira の JQL の断片、Azure の WIQL の条件、Trello のリスト名）を一覧のクエリに AND で加えます。GitHub と GitLab は無視します。 | サイドバーのセクション、`gitpoulpi hosting link` |
| `ref_visibility` | 非表示 / ソロ表示の参照 | Refs サイドバー、`gitpoulpi refs hide\|show\|solo` |
| `hidden_remotes` | グラフからまるごと非表示にしたリモート（すべてのブランチ。あとで fetch したものも） | リモートの目のアイコンとメニュー、`gitpoulpi refs hide --remote` |
| `folds`、`collapsed_*_ref_folders` | 折りたたんだサイドバーのセクションと、ブランチおよびタグのフォルダー。`[folds]` には、`status_folders`（Unstaged / Staged のファイルツリーの折りたたんだフォルダー）と `commit_folders`（commit の変更ファイルツリーの折りたたんだフォルダーで、すべての commit で共通）も含まれ、それぞれ最大 500 パスで、古いものから捨てられます | サイドバー、ファイルツリー |
| `pending_auto_stash` | GitPoulpi が作成し、まだ戻していない stash | デーモンが書き込みます。[自動 stash](log-and-history.md)を参照 |

## その他の環境変数

| 変数 | 効果 |
|---|---|
| `GITPOULPI_CONFIG_DIR` | 設定フォルダー（上記を参照）。 |
| `GITPOULPI_TOKEN_<ID>` | プロファイル `<ID>` のホスティングトークンで、保存済みのものより優先されます（CI や一度限りの利用向け）。 |
| `GITPOULPI_GIT` | 実行する `git` の実行ファイル（既定：`PATH` 上の `git`。Windows では Git for Windows の `cmd\git.exe` シムの背後にある本物の `mingw64\bin\git.exe` で、呼び出しごとのプロセスを 1 つ節約できます）。 |
| `GITPOULPI_LANG=fr` | `ui.language` より優先されるインターフェース言語（`ui.language` の任意の値。`qps` は翻訳者向けの疑似ロケール）。未知の値や空の値は無視されます。 |
| `GITPOULPI_TERMINAL_IMAGE=off` | 設定にかかわらず、画像をまったく使いません。 |
| `GITPOULPI_TERMINAL_CELL_PX=9x21`、`GITPOULPI_TERMINAL_SUPERSAMPLE=1..4` | 画像の解像度のヒント（セルのピクセルサイズ、オーバーサンプリング。既定は 2 で、`1` にすると CPU を節約できます）。 |
| `GITPOULPI_NO_BROWSER=1` | Web ブラウザーを起動しません（URL は表示されます）。 |
| `GITPOULPI_TIMING=1` | リクエストの所要時間を一時フォルダーの `gitpoulpi-timing.log` に書き込みます（性能レポート用）。 |
| `GITPOULPI_NO_EXIT_PAUSE=1` | エラーで TUI が終了した後に、Enter を待ちません。 |
