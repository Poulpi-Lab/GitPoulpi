[English](../custom-builds.md) · [Čeština](../cs/custom-builds.md) · [Deutsch](../de/custom-builds.md) · [Español](../es/custom-builds.md) · [Français](../fr/custom-builds.md) · [Italiano](../it/custom-builds.md) · **日本語** · [한국어](../ko/custom-builds.md) · [Nederlands](../nl/custom-builds.md) · [Polski](../pl/custom-builds.md) · [Português (Brasil)](../pt-BR/custom-builds.md) · [Português (Portugal)](../pt-PT/custom-builds.md) · [Русский](../ru/custom-builds.md) · [Svenska](../sv/custom-builds.md) · [Türkçe](../tr/custom-builds.md) · [Українська](../uk/custom-builds.md) · [简体中文](../zh-CN/custom-builds.md)

# カスタムビルド

より小さなバイナリが欲しい人向けです。対話型スクリプトが、含める言語、ホスティング連携、テーマ、ロゴの色、機能を尋ね、
最適化プロファイルでそれらだけをビルドします。通常のビルドと公式リリースは
変わりません（すべてオン）。

```
./build-custom.sh                # Linux / macOS (scripts/build-custom.sh)
build-custom                     # Windows, cmd or PowerShell (scripts/build-custom.ps1)
```

出力は `target/custom/` に置かれ、公式の `target/dist/` を上書きすることはありません。スクリプトは
各バイナリのサイズと、そのビルドが存在する場合は `target/dist/` との差を表示します。また、バイナリの隣に
`BUILD-FEATURES.txt` を書き出します。

## 選べるもの

| 選択 | オフにした場合 |
|---|---|
| 言語 | 英語（常にビルドされます）と、チェックを入れたものだけ。設定の選択肢には、ビルドされた言語だけが表示されます。 |
| テーマ | GitPoulpi Dark、GitPoulpi Light、Terminal は常にビルドされます（`system` には Dark と Light が必要です）。それ以外のテーマはファミリー単位（`catppuccin`、`gruvbox`、`nightfox` など。`--list-themes` で一覧表示）で選びます。選ばれなかったテーマはコンパイルされません。設定のコンボと `gitpoulpi theme list` に表示されず、そのテーマを指定した設定は、未知のテーマと同様に通常の警告とともにフォールバックします。 |
| ロゴの色 | 「GitPoulpi について」ウィンドウのピクセルのタコには 20 色のバリエーションがあります（`--list-logo-colors`）。コンパイルに含めるもの（少なくとも 1 つ）、既定の色（そのウィンドウが最初に表示する色。コンパイルされていない保存済みの `ui.logo_color` のフォールバック先でもあります。コンパイルされた色のどれかである必要があります。公式の既定：lavender）、Windows 実行ファイルのアイコンの色（任意のバリエーション。既定：ロゴの既定）を選びます。バリエーションが 1 つだけの場合、ロゴはクリックで回転しますが色は変わらず、色の一覧（`c`、右クリック）は表示されません。 |
| 連携（GitHub、GitLab、Gitea、Forgejo、Azure DevOps、Bitbucket、Gerrit、Jira、Trello、Linear、YouTrack、Redmine） | 何も選ばない場合：URL によるクローンのみ。Issue も pull/merge request もありません。 |
| AI | AI のボタン、コマンド、`ai` の CLI/MCP ツールがありません。 |
| MCP | `gitpoulpi-mcp` バイナリも `gitpoulpi mcp` コマンドもありません。`ui` のエージェントコマンドは残ります。 |
| LFS、Worktree、サブモジュール | 対応する管理画面とコマンドがなくなり、「このバイナリには含まれていません」という型付きの応答が返されます。 |
| アバターのダウンロード | イニシャルのバッジのみで、ネットワーク取得はありません。 |
| 画像プレビュー | 画像ファイルには通常のバイナリ通知が表示されます。**編集に使う**は引き続き動作します。 |
| Radar | Radar タブ、`Alt+R`、ホームのキー、パレットのコマンドがなくなり、Radar は開きません。連携が必要です（連携がなければもともと機能しません）。 |
| Mermaid | Mermaid gitGraph としてコピー（ログのメニュー、`Shift+Y`、パレット）がなくなり、CLI/MCP に `log --format mermaid` もありません。また、[会話](conversations.md)の `mermaid` ブロックの下の**図を開く**ボタン（ブロックはコードのまま残ります）と、パレットの「会話: 項目の図を開く」の行もなくなります。 |
| Conversation | Issue や request のアプリ内[会話](conversations.md)表示がなくなります：Issue の行、ボードのカード、Radar の行での Enter はこれまでどおりブラウザーを開き、「会話を開く」の行も「会話: ...」のパレットコマンドもなく、会話内の「AI で作成」もありません（レビュー、Issue フォーム、pull request フォームの入力欄にあるものは残ります）。CLI と MCP の会話コマンド（`hosting issue comments`、`comment`、`comment-edit`、`comment-delete`、`hosting review conversation`）と `ai hosting-reply` は残ります。連携が必要です。 |
| Board | かんばんボードのビュー（ISSUES ヘッダーのボタン、パレット）と `hosting board` コマンドがなくなります。連携が必要です。 |
| Review | レビューの概要ページは会話表示なので、Review を含むビルドには、*Conversation* の設定にかかわらず会話表示も含まれます。アプリ内のコードレビュー表示（行メニュー、pull request の行での Enter、パレット、Radar の `Review`）と AI のレビュー支援がなくなります。CLI/MCP の `hosting review` コマンドは残ります（Gerrit とエージェントが使います）。連携が必要です。 |
| 履歴ワークベンチ（高度な履歴の書き換え：作者、日付、メッセージ、内容） | 履歴書き換えワークベンチがなくなります。`書き換え:` のパレットコマンド、ワークベンチのコード、キー、ヘルプのセクション、ログメニューの「履歴を書き換え」の行、修正ダイアログの「履歴をさらに深く変更...」ボタン、「AI でルールを作成」とスクリプトの AI、同期設定ページのスクリプトの上限、AI 設定の履歴書き換え用 AI ブロックがすべてなくなります。`rewrite` の CLI/MCP コマンド、`ai rewrite-script`、`ai rewrite-rules`、`ui propose rewrite` はコマンドツリーから消えます。`.mailmap` の ID（`mailmap`、`log.use_mailmap`）はこれに含まれず、残ります。 |
| 全体検索 | タイトルバーの全体検索ボタン、`Ctrl+Alt+F`、`検索: ワークスペース...` のパレットコマンド、`gitpoulpi search` の CLI/MCP コマンドがなくなります。ログの検索バー（`ui show search`）は別の機能で、残ります。 |
| ワークスペース | ウィンドウは既定のタブ一覧（「ワークスペースなし」）だけで動きます。ワークスペースのチップ、ワークスペースのタブ、`Alt+W` / `Alt+J`、`ワークスペース:` のパレットコマンド、「ワークスペースに追加」のタブ行、`gitpoulpi workspace` の CLI/MCP コマンド、`commit` と `search` の `--workspace` フラグがなくなります。タブとセッションは通常どおり動作します。ほかのビルドが書いた `workspaces.toml` は読まれず、変更もされません。ワークスペースの既定のプロファイルと AI 指示は適用されません。Radar と全体検索は開いているタブを対象にします。 |
| 書き換えスクリプト（組み込み Lua） | 履歴書き換えの Script ルールは、型付きの「このバイナリには組み込まれていません」という応答で拒否され、Lua はリンクされません。履歴ワークベンチ自体は残ります（独立した選択肢です）。履歴ワークベンチがオフのときは意味がなく、警告が出ます。 |

設定ファイルはビルド間で共有されます。ビルドされていないものの設定は、保持されて無視されます。

## 対話型

Bash スクリプトは `whiptail` または `dialog` のチェックリスト（3 画面と、その後の概要）を使い、
どちらもない場合は番号付きのプロンプトを使います。PowerShell スクリプトは、利用できれば
`Out-GridView` を使い、なければプロンプトを使います。前回の選択はあらかじめチェックされています。

## 非対話型

引数を 1 つでも指定すると質問は省略され、そのまま
`cargo run -p xtask -- build-custom` が実行されます。

```
./build-custom.sh --langs fr,de --integrations github --no-ai --no-mcp
./build-custom.sh --last --dry-run         # previous choices, print the cargo command only
.\build-custom -Langs fr -Integrations none -NoAi -NoLfs -NoWorktree -NoSubmodules
```

| オプション | 意味 |
|---|---|
| `--langs fr,de` / `all` / `en` | 英語以外の言語。コードはロケールのファイル名です（`pt-br`、`zh-cn` など）。 |
| `--integrations github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues` / `all` / `none` | プロバイダー：GitHub、GitLab（セルフマネージドを含む）、Azure DevOps（Services と Server）、Bitbucket（Cloud と Data Center）（ホスティング）、Gitea と Forgejo（Codeberg を含む）（`gitea`）（ホスティング）、Jira、Trello、Linear、YouTrack（Cloud と Server）、Redmine（Issue トラッカー）、Gerrit Code Review（`gerrit`。レビューのみ）、ローカル Markdown Issue（`local-issues`。フォルダーまたは Obsidian の保管庫）。`azure-devops`、`bitbucket`、`gerrit`、`gitea`、`jira`、`linear`、`local-issues`、`redmine`、`trello`、`youtrack` はそれぞれ独自の cargo feature です。サイズは [binary-size](../../developer/binary-size.md) にあります。 |
| `--themes all` / `none` / `nord,catppuccin` | コンパイルに含めるテーマファミリー（既定は `all`）。`--list-themes` でファミリーを表示します。 |
| `--logo-colors all` / `violet,teal` | コンパイルに含めるロゴのバリエーション（既定は `all`。表の順序のまま保持され、これが回転の順序です）。`--list-logo-colors` で一覧を表示します（`*` = 公式の既定）。 |
| `--logo-default teal` | 既定のバリエーション。コンパイルされたものの中から選ぶ必要があります（既定：lavender、なければ最初にコンパイルされたもの）。 |
| `--icon-color sun` | Windows 実行ファイルのアイコンの色（既定：ロゴの既定）。 |
| `--[no-]ai`、`--[no-]mcp`、`--[no-]lfs`、`--[no-]worktree`、`--[no-]submodules`、`--[no-]avatar-fetch`、`--[no-]image-view`、`--[no-]radar`、`--[no-]mermaid`、`--[no-]board`、`--[no-]review`、`--[no-]conversation` | それぞれ既定はオンです。 |
| `--[no-]history-rewrite` | 履歴ワークベンチ（高度な履歴の書き換え：作者、日付、メッセージ、内容）。既定はオン。 |
| `--[no-]global-search` | 全体検索（タイトルバーのボタン、`Ctrl+Alt+F`、`gitpoulpi search`）。既定はオン。 |
| `--[no-]workspaces` | ワークスペース（名前付きのリポジトリのグループ）。既定はオン。オフにすると既定のタブ一覧だけが残ります。 |
| `--[no-]rewrite-scripts` | 書き換えの Script ルール用の組み込み Lua。既定はオン。履歴ワークベンチがオフのときは、警告を出して無視されます。 |
| `--save <name\|file>` | 確定した選択を保存した選択として保存します（名前またはファイルパス）。 |
| `--preset <name\|file>` | 保存した選択から始めます（名前またはファイルパス）。ほかのオプションは、行のどこにあってもそれを上書きします。 |
| `--list-presets`, `--delete-preset <name>` | 保存した選択とそのフォルダーを表示 / 名前を指定して 1 つ削除します。 |
| `--last` | `target/custom-build/last-choices.env` を再利用します。ほかのオプションはそれを上書きします。 |
| `--full` | 選択なし（すべてオン）。 |
| `--profile custom\|dist\|release` | 既定は `custom`（`dist` の設定で、出力ディレクトリは別）。 |
| `--dry-run` | 実際の cargo コマンドを表示して停止します。 |
| `-- <cargo args>` | cargo に渡されます。例：`--target x86_64-unknown-linux-musl`。 |

PowerShell の名前：`-Langs -Integrations -Themes -LogoColors -LogoDefault -IconColor -NoAi -NoMcp -NoLfs -NoWorktree
-NoSubmodules -NoAvatarFetch -NoImageView -NoRadar -NoMermaid -NoBoard -NoReview -NoHistoryRewrite -NoGlobalSearch -NoWorkspaces -NoRewriteScripts -Last -Preset -Save -ListPresets -DeletePreset -Full -DryRun -Profile -CargoArgs`。

テーマの選択は、環境変数 `GITPOULPI_THEMES`（`all`、`none`、またはファミリーのカンマ区切りリスト）として
ビルドに渡されます。これを `gitpoulpi-config` のビルドスクリプトが読み取り、コンパイル時にそれ以外のパレットを
除外します。通常の `cargo build` でも、自分で設定してもかまいません。`gitpoulpi
--version` は、ビルドされたセットを示す `themes:` の行を表示します。

ロゴの選択は、環境変数 `GITPOULPI_LOGO_COLORS`
（`all` またはスラッグのカンマ区切りリスト）、`GITPOULPI_LOGO_DEFAULT`、
`GITPOULPI_ICON_COLOR` で、`gitpoulpi-tui` のビルドスクリプト
（`crates/gitpoulpi-logo-build/colourways.json` をフィルターして「GitPoulpi について」ウィンドウが埋め込む表にします）と、
3 つのバイナリ crate のビルドスクリプト（アイコン）が読み取ります。色の元データは
`crates/gitpoulpi-logo-build/colourways.json` のままです。

### Windows 実行ファイルのアイコン

ターゲットが Windows の場合、`gitpoulpi`、`gitpoulpi-tui`、`gitpoulpi-mcp` の
実行ファイルには、選んだアイコンの色で 16 から 128 px のサイズのタコがアイコンとして付きます。
ビルド時に生成されます（Python も ImageMagick も不要）。
`winresource` crate で埋め込まれ、プラットフォームのリソースコンパイラ（MSVC では Windows SDK の
`rc.exe`、GNU では `windres`）が必要です。ない場合、ビルドは警告を表示するだけで、
実行ファイルは既定のアイコンのままになります。GNU ツールチェーンでは `gitpoulpi` だけがアイコンを持ちます
（そうしないとリソースが 2 回リンクされるため）。Linux と macOS 向けのビルドでは、色の表を除き、
これらはすべて無視されます。

## 保存した選択

選択をすべてやり直さずに新しいバージョンをビルドし直すには、選択に名前を付けて保存し、後でそこから始めます。`--preset` と一緒に指定したオプションは、保存された値を引き続き上書きします。

```
./build-custom.sh --no-ai --no-rewrite-scripts --save no-ai    # keep these choices as "no-ai"
./build-custom.sh --preset no-ai --dry-run                     # start from them later
./build-custom.sh --preset ~/dotfiles/gitpoulpi.env --langs fr # a file path works too
./build-custom.sh --list-presets
./build-custom.sh --delete-preset no-ai
.\build-custom -Preset no-ai -Save no-ai-v2                    # PowerShell
```

保存した選択は、単純な `KEY=value` ファイル（`last-choices.env` と同じ形式で、シェルから読めます）です。GitPoulpi の設定ディレクトリの `custom-builds` フォルダー（`target/` の外）に置かれるため、`cargo clean` では削除されません。

- Linux: `$XDG_CONFIG_HOME/gitpoulpi/custom-builds/` (`~/.config/gitpoulpi/custom-builds/` by default)
- Windows: `%APPDATA%\GitPoulpi\config\custom-builds\`
- macOS: `~/Library/Application Support/GitPoulpi/custom-builds/`

`GITPOULPI_CONFIG_DIR` を使うと、このフォルダーも設定と一緒に移動します。`--preset` と `--save` はファイルパス（`/` または `\` を含む値、もしくは `.env` で終わる値）も受け付けるので、選択を dotfiles リポジトリで管理できます。

ほかのバージョンの GitPoulpi が書いた選択も読み込めます。未知のキーや、すでに存在しない言語・連携・テーマ・色は、警告を出して無視されます。ファイルにないキー（後から追加された機能）は既定値になり、後から追加された連携は含められます。どちらも標準エラー出力に報告されます。

保存した選択がある場合、対話型では最初に「開始元」（前回の選択、保存した選択、またはすべてオン）を尋ね、最後に現在の選択を名前を付けて保存するかどうかを尋ねます。

## 節約できるサイズ

2026-10-29 にコミット `65999ef5c` のフルビルドを Linux で計測（括弧内はフルビルドとの差）
（詳細は `docs/developer/binary-size.md`）：

| ビルド（特記なき限り `full` から） | `gitpoulpi` (MiB) | `gitpoulpi-tui` (MiB) | `gitpoulpi-mcp` (MiB) |
|---|---|---|---|
| フル（すべてオン） | 64.6 | 50.5 | 31.7 |
| 英語のみ | 52.0 (-12.6) | 40.2 (-10.3) | 31.7 (0.0) |
| MCP オフ | 61.6 (-3.0) | 49.6 (-0.8) | ビルドされない |
| AI オフ | 62.6 (-2.0) | 48.6 (-1.8) | 30.5 (-1.3) |
| 連携をすべてオフ | 56.3 (-8.2) | 43.7 (-6.7) | 26.7 (-5.0) |
| AI + 連携 + アバター取得をオフ | 52.2 (-12.3) | 39.9 (-10.6) | 23.0 (-8.8) |
| 履歴ワークベンチオフ (`--no-history-rewrite`) | 61.6 (-3.0) | 48.9 (-1.5) | 28.5 (-3.2) |
| 書き換えスクリプトオフ | 63.1 (-1.5) | 49.0 (-1.5) | 31.1 (-0.7) |
| 全体検索オフ | 63.5 (-1.1) | 49.5 (-1.0) | 31.6 (-0.1) |
| ワークスペースオフ | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.2) |
| Radar オフ | 63.4 (-1.2) | 49.3 (-1.2) | 31.7 (0.0) |
| Board オフ | 63.6 (-1.0) | 49.5 (-1.0) | 31.7 (0.0) |
| Review オフ | 63.2 (-1.4) | 49.0 (-1.4) | 31.7 (0.0) |
| Mermaid オフ | 63.6 (-1.0) | 49.5 (-0.9) | 31.6 (-0.1) |
| LFS オフ | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.3) |
| Worktree オフ | 63.7 (-0.9) | 49.6 (-0.8) | 31.7 (0.0) |
| サブモジュールオフ | 63.7 (-0.9) | 49.6 (-0.9) | 31.6 (-0.1) |
| アバター取得オフ | 63.7 (-0.9) | 49.6 (-0.9) | 31.7 (0.0) |
| 画像プレビューオフ | 63.3 (-1.2) | 49.2 (-1.3) | 31.7 (0.0) |
| すべてオフ、英語のみ（LFS は残す） | 35.2 (-29.4) | 28.9 (-21.5) | ビルドされない |
| **すべてオフ、英語のみ** | **34.9 (-29.7)** | **28.8 (-21.6)** | ビルドされない |

## 注意

- カスタムビルドは、ほかの `dist` ビルドと同様に遅いです（thin LTO、数分）。
- `build-opti` と同様に、sccache なしで実行されます。
- 言語コードは同梱のロケールと照合されます。未知のコードは、
  有効なコードの一覧とともに拒否されます。
