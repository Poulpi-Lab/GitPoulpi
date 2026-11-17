[English](../pull-requests-and-issues.md) · [Čeština](../cs/pull-requests-and-issues.md) · [Deutsch](../de/pull-requests-and-issues.md) · [Español](../es/pull-requests-and-issues.md) · [Français](../fr/pull-requests-and-issues.md) · [Italiano](../it/pull-requests-and-issues.md) · **日本語** · [한국어](../ko/pull-requests-and-issues.md) · [Nederlands](../nl/pull-requests-and-issues.md) · [Polski](../pl/pull-requests-and-issues.md) · [Português (Brasil)](../pt-BR/pull-requests-and-issues.md) · [Português (Portugal)](../pt-PT/pull-requests-and-issues.md) · [Русский](../ru/pull-requests-and-issues.md) · [Svenska](../sv/pull-requests-and-issues.md) · [Türkçe](../tr/pull-requests-and-issues.md) · [Українська](../uk/pull-requests-and-issues.md) · [简体中文](../zh-CN/pull-requests-and-issues.md)

# pull request と Issue

サイドバー、ボード、ツールバーから pull / merge request と Issue を読み、作成し、管理します。先にアカウントを接続し、リポジトリをリンクしてください：[ホスティングと Issue トラッカーの連携](hosting-integration.md)を参照してください。1 件の Issue やリクエストについて語られたこと（コメント、レビュー、スレッド）は[会話](conversations.md)に、リクエストの閲覧とレビューは [pull request のレビュー](review.md)にあります。

## サイドバーの pull request と Issue

### PULL REQUESTS

セクションには、件数と `+`（pull request を作成）を持つヘッダー、プロバイダーとプロジェクトのドロップダウン、検索ボックス、状態のドロップダウン（*オープン*、*merge 済み / クローズ*、*すべて*）があります。その下に、それぞれ件数付きの 4 つのグループが並びます。

- **自分のpull request**：自分が作成者のもの。
- **自分に割り当て**。
- **自分のレビュー待ち**：自分がレビュー依頼を受けているもの。
- **すべてのpull request**：最初は折りたたまれています。

検索ボックスはすべてのグループを絞り込みます（タイトル、番号、作成者、ブランチ）。状態のドロップダウンを変更するたびに、ホストへ再問い合わせします。

行は状態のグリフで始まります：`●` オープン（緑）、`○` ドラフト、`◆` merge 済み（紫）、`✕` クローズ（赤）。チェックアウト中のブランチのリクエストは太字です。行は `#12  title`（GitHub、Bitbucket）または `!53  title`（GitLab、Azure DevOps）の形式です。

### ISSUES

セクションのタイトルにはプロバイダー名が付きます：`GITHUB ISSUES`、`JIRA ISSUES` など。Azure DevOps では `... WORK ITEMS`、Trello では `... CARDS` となります。ドロップダウンの下に、検索ボックス、状態のドロップダウン（*オープン*、*自分に割り当て*、*クローズ*、*すべて*）、そして Issue が並びます。Jira の行は `ABC-12  title` の形式で、サイドバーが十分に広いときは種別と状態が薄く表示されます。

![サイドバーの ISSUES セクション。Issue #7 の右クリックメニューが開き、先頭は会話を開く](../images/issues-sidebar.png)

### 行に対する操作

- `Enter` またはダブルクリックで、その行を開きます：pull request は[レビュー](review.md)で（GitHub、GitLab、Gitea、Forgejo、Gerrit）、Issue と Azure DevOps や Bitbucket のリクエストはウィンドウ内の[会話](conversations.md)として、それ以外はブラウザーで開きます。行メニューの**ブラウザーで開く**は、常にブラウザーを開きます。
- 右クリック、または `.` で、行のメニューを開きます。
  - **会話を開く**（Issue と、レビュー画面のないリクエスト）、**ブラウザーで開く**、**URL をコピー**、**番号をコピー**。
  - pull request の場合：**<provider> で編集**、**ソースブランチをチェックアウト**、**ソースブランチのcommitを表示**。フォークからのリクエストでは、チェックアウトは**pull requestをチェックアウト（フォーク）**となり、**表示**はグレーアウトされます。
  - Issue の場合：**Issue からブランチを作成**。`12-fix-the-thing`（Jira なら `ABC-123-fix-the-thing`）のような名前で、新しいブランチのプロンプトが開きます。
  - GitHub、GitLab、Gitea、Forgejo では、pull request の行に**レビュー...**もあります（行での **Enter** も同じ）。アプリ内のレビューです。[pull request / merge request のレビュー](review.md)を参照してください。
  - GitHub、GitLab、Gitea、Forgejo では、どちらの行にも**担当者を編集...**、**ラベルを編集...**、**マイルストーンを設定...**があります（pull request には**レビュアーを編集...**も）。[担当者、レビュアー、ラベル、マイルストーン](#担当者レビュアーラベルマイルストーン)を参照してください。
- **<provider> で編集**には、ほかにも 3 つの入口があります：ホバーまたは選択した pull request 行の右端の鉛筆 `✎`、選択した行での `e`、そしてパレットコマンドの*連携: 関連するPull Requestをホストで編集*（チェックアウト中のブランチのリクエスト用）です。
- ソースブランチがまだリポジトリにない場合、GitPoulpi はその旨を表示します。先に fetch してください（`Ctrl+G f`）。

**pull request のチェックアウト。** GitPoulpi はリクエストを、それを追跡するローカルブランチ `pr/<n>`（GitLab では `mr/<n>`）に fetch して、そのブランチに切り替えます。GitHub、GitLab、Gitea、Forgejo では、フォークでもこれが機能します（GitPoulpi がホストの特別な head 参照を fetch します）。Azure DevOps と Bitbucket は、リクエストのソースブランチを fetch します。フォークからのリクエストはこの 2 つでは拒否されます：フォークをリモートとして追加し、そのブランチを自分でチェックアウトしてください。元に戻す操作で、前のブランチに戻ります。CLI：`gitpoulpi hosting checkout-pr <n>`。

**コマンドラインからの（そしてエージェントによる）pull request のレビュー。** GitHub（Enterprise を含む）、GitLab（セルフマネージドを含む）、Gitea、Forgejo では、GitPoulpi を離れずにリクエストを読み取ってレビューできます：`hosting review show <n>`（ファイル、マージの状態、チェック）、`hosting review threads <n>`（レビューのスレッドと会話）、`hosting review comment`、`hosting review reply`、`hosting review resolve`、`hosting review submit <n> --event approve|request-changes|comment --comment path:line[:side]:text`。差分はホストではなくリポジトリから取得されます：`hosting review fetch <n>` が、リクエストの head を `refs/gitpoulpi/pr/<n>` に、ターゲットブランチを `refs/gitpoulpi/pr/<n>-base` に取得します（どちらもブランチではなく、何もチェックアウトされません）。行番号はその差分のものです：`new` は head 側の行、`old` はマージベース側の行です。GitLab には「変更を要求」のレビュー状態がないため、そこでは拒否され、何も送信されません。Gitea と Forgejo は 1 行へのコメントしか受け付けず（範囲は不可）、スレッドはファイルの 1 行に付いたコメントのまとまりです。Gitea はスレッドを解決できますが、Forgejo 11 はできません（ホストにそのためのエンドポイントがなく、呼び出しがその旨を伝えます）。返信用のエンドポイントがないサーバーでの返信は、同じ行への 1 コメントのレビューとして投稿されます。Gitea は自分のリクエストの承認や却下を拒否します。
クローンなしの場合：`review show`、`threads`、`comment`、`reply`、`resolve`、`submit`、`suggest` に `--project owner/name`（GitLab は `group/project`）を付けると、
現在のリポジトリではないプロジェクトをレビューできます。`hosting review diff <n> --project P` はホストの API から diff（ファイルごとのハンク）を、
`hosting file <path> --project P [--ref R]` は指定リビジョンのファイルを読み取ります（GitHub、GitLab、Gitea、Forgejo、Gerrit。Gerrit ではパッチセットの
diff がファイルごとに読み取られ、ファイルはブランチ、コミット ID の場合はそのコミットから取得されます）。ホストが diff に表示しないファイル（大きすぎる場合）には
`omitted` が付きます。
MCP ツールは同じコマンドです（`hosting_review_show`、`hosting_review_submit`、`hosting_review_diff`、
`hosting_file`...）。

### 担当者、レビュアー、ラベル、マイルストーン

GitHub（Enterprise を含む）、GitLab（セルフマネージドを含む）、Gitea、Forgejo では、GitPoulpi を離れずに、pull request や Issue の担当者、レビュアー、ラベル、マイルストーンを変更できます。ほかのプロバイダーでは、これらは表示されません。

- **行のメニュー**：**担当者を編集...**、**レビュアーを編集...**（pull request のみ）、**ラベルを編集...**、**マイルストーンを設定...**。
- **パレット**：*ホスティング: 担当者を編集...*、*ホスティング: レビュアーを編集...*、*ホスティング: ラベルを編集...*、*ホスティング: マイルストーンを設定...*は、いずれも最初にサイドバーのリクエストと Issue を一覧して選ばせます。[Radar](radar.md) では、同じ 4 つが選択した行に作用します。
- ピッカーはすぐに開き、プロジェクトの人やラベル（とアイテムの現在の状態）をホストから読み込みます。約 150 ms 後にスピナーの行が表示されます。入力で絞り込み（あいまい一致）、`Up` / `Down` で移動、`Space` でチェックの切り替え、`Enter` で適用、`Esc` でキャンセル、`Tab` でボタンに移動します。行のクリックでチェックされ、ホイールでスクロールします。人にはイニシャルのバッジ、ラベルにはその色が表示されます。
- マイルストーンはドロップダウンです（**マイルストーンなし**で解除できます）。
- **適用**の後にホストが更新され、一覧と Radar には、すぐに始まる次の更新の後に変更が反映されます。

作成フォーム（pull request、Issue）にも、同じ選択肢がボタンの並びとして表示されます：`担当者: なし`、`レビュアー: なし`、`ラベル: なし`、`マイルストーン: なし`。GitHub、GitLab、Gitea、Forgejo では、ラベルのボタンが Issue フォームの入力式のラベル欄に置き換わります。ホストが新しい pull request を受け付けても一部のラベルや人を拒否した場合は、ステータスバーがそう伝えます。

CLI：`gitpoulpi hosting members|labels|milestones`、`hosting pr-edit <n>`、`hosting issue edit <n>`、そして `hosting pr` と `hosting issue create` の `--assignee`、`--reviewer`、`--label`、`--milestone`。[CLI リファレンス](cli-reference.md)を参照してください。

### セクションのキー

| キー | 動作 |
|---|---|
| `Enter` / ダブルクリック | ホストに会話があれば GitPoulpi 内で会話を開く（GitHub、GitLab、Gitea、Gerrit の pull request ではレビューが開く）。なければブラウザー |
| `o` | 行をブラウザーで開く |
| `.` / 右クリック | 行のメニュー |
| `e` | 選択した pull request をホストで編集 |
| `n` | 新しい pull request（PULL REQUESTS）または新しい Issue（ISSUES） |
| `v` | 状態のドロップダウンを開く |
| `r` | 更新 |
| ヘッダーでの `Enter` / `Space` | セクションを折りたたむ／展開する |

### 再接続とオフライン

トークンがない、拒否された、またはスコープが足りないとき、セクションの最初の行に `再接続（トークン拒否）` と表示されます。クリックするか `Enter` を押すと、設定 > 連携でそのプロファイルが開きます。

ホストに接続できないときは、最後の一覧が画面に残り、ヘッダーに経過時間（`· 5m old`）が表示されます。最後の応答はディスク（`.gitpoulpi/hosting-cache.json`）にもキャッシュされるため、ネットワークが応答する前に一覧が表示されます。

## Issue ボード

ISSUES ヘッダーの**ボード**ボタン（パレット：**Issue: ボードを開く**）は、リンクされた取得元の Issue を、ウィンドウの上にカンバンボードとして表示します。列は取得元によって決まります。

| 取得元 | 列 | カードの移動 |
|---|---|---|
| ローカル Markdown ノート | ステータス（プロファイルの `statuses`、続いて使われているその他のもの） | ノートの `status` を書き換える |
| GitHub、GitLab、Gitea、Forgejo | **オープン**、選んだラベル、**クローズ** | ステータスのラベルを入れ替える。**クローズ**へは Issue をクローズし、再び別の列へは再オープンする |
| Trello | ボードのオープンなリストを左から右へ | カードをそのリストへ移動する |
| Redmine | Issue のステータス | Issue のステータスを変更する（Redmine のワークフロールールが適用される） |
| Jira（Cloud と Data Center） | プロジェクトのステータス。未着手が先、完了が最後 | その列へ至るワークフローの遷移を実行する（`GET`、続いて `POST .../transitions`）。該当する遷移がない場合、カードは元に戻り、ステータスバーに `no transition from X to Y` と表示される |
| Azure DevOps（Services と Server） | プロジェクトのワークアイテムタイプの状態を、状態カテゴリ別に | `System.State` を設定する。その列の状態を持たない種類のワークアイテムは、理由（`a Bug has no state Done`）とともに元に戻る |

GitHub または GitLab のプロジェクトでは、**列**（または `c`、**ボード: 列のラベルを設定...**）が、列にするラベルを順番に尋ねます（例：`status::todo, status::doing, status::review`）。プロジェクトのラベルのうち、ステータスに見えるものが提案されます。選択はリポジトリごとに保存されます。ラベルがない場合は、オープン / クローズのボードになります。カードを移動すると、指定したステータスのラベルだけが変わり、そのほかのラベルはそのままです。ホストのボードには、オープンな Issue と、最近クローズされた 60 件が一覧されます。

Jira の遷移が必須項目（たとえば解決状況）を求める場合、Jira に拒否され、カードは Jira 自身のメッセージとともに元に戻ります：その操作はサイトで行ってください。

同じボードはスクリプトやエージェントからも使えます：`gitpoulpi hosting board` は列とそのカードを出力し、`gitpoulpi hosting board move <card> <column>` はカードを 1 枚移動します（[CLI リファレンス](cli-reference.md)を参照）。

カードには、番号、タイトル、担当者のイニシャル、ラベル、マイルストーンが表示されます。別の列にドラッグするか、`Shift+Left` / `Shift+Right`、または**ボード: カードの移動先...**（`m`）を使います。カードはすぐに移動し、約 150 ms 後にホストが処理中であることをスピナーが示します。ホストが拒否した場合はカードが元に戻り、ステータスバーが理由を伝えます。`Enter` はアプリ内で Issue の[会話](conversations.md)を開き（ボードは閉じると戻ります）、`e` は担当者、ラベル、マイルストーンを編集し、`n` は選択中の列に新規 Issue を作ります。`a`、`l`、`M`、`/` は担当者、ラベル、マイルストーン、テキストで絞り込み、`x` は絞り込みを解除し、`r` はホストを読み直します。サイドバーの一覧が更新されると、ボードも読み直されます。Radar にはボードはありません。

## pull request を作成する

フォームは、PULL REQUESTS ヘッダーの `+`、セクション内での `n`、またはパレットコマンドの*Pull Request: Pull Request を作成...*で開きます。ブランチのメニューにも、**<branch> からpull requestを開始...**があります。

フォームの内容：

- pull request を持つ接続済みプロファイルごとに 1 つのタブ（タブストリップで `Left` / `Right`）。
- **元のリポジトリ / ブランチ**と**先のリポジトリ / ブランチ**のコンボボックス。長い一覧は検索できます。一覧にないブランチやリポジトリを入力することもできます。
- **タイトル**。先頭 commit の件名が入力されます。
- **説明**。複数行（`Enter` で改行、`Tab` でフィールドを離れる）。
- **担当者**、**レビュアー**、**ラベル**、**マイルストーン**のボタン（GitHub、GitLab、Gitea、Forgejo）。ピッカーを開きます。
- **ドラフトとして送信**。
- [AI](ai.md) がオンのとき：**タイトルと説明を生成**。

フォームは、チェックアウト中のブランチのプロファイル、リポジトリ、ブランチ（`feature -> main`）で開きます。ソースのリポジトリを別にすると、フォークからのリクエストになります。

**Pull Request を作成**（または `Ctrl+S`）で、ホスト上にリクエストが作成されます。結果はブラウザーで開き、ステータスバーにもその名前が表示されます。

## Issue を作成する

ISSUES セクションにプロバイダー**と**プロジェクトが設定されていると、そのヘッダーに `+`（*新規 Issue*）が表示されます。ほかには、セクション内での `n`、またはパレットコマンドの*ホスティング: 新規 Issue...*があります。

フォームには**タイトル**（必須）、**説明**（複数行）、**ラベル**（カンマ区切り、例：`bug, ui`。GitHub、GitLab、Gitea、Forgejo では、代わりにラベル、担当者、マイルストーンのボタンがピッカーを開きます）があります。Jira と Azure DevOps では**種別**のドロップダウンが追加され、Trello では**リスト**のドロップダウンが追加されます。Linear と YouTrack では**ステータス**のドロップダウン（Issue が始まるワークフローの状態、または `State` の値）が追加されます。Redmine ではプロジェクトのトラッカー（Bug、Feature、Support...）を選ぶ**種別**のドロップダウンが追加されます。Redmine にはラベルがないため、ラベルのボタンはなく、コマンドラインで指定したラベルは拒否されます。これらのドロップダウンの選択肢は、フォームを開いたときにホストから取得されます。

**Issue を作成**（または `Ctrl+S`）で、ホスト上に作成されます。`Esc` または**キャンセル**でフォームを閉じます。ホストが拒否した場合（トークンの拒否、スコープ不足、不明なプロジェクト、不明なラベル）、フォームは理由を表示し、入力した内容を保持します。成功すると、ステータスバーに新しい Issue の名前が表示され、ISSUES セクションが更新されます。

CLI：`gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`。

## ホスト自身の Web ページで作成する

新規 Issue フォームと pull request 作成フォームには、右上に **<provider> で作成**ボタンがあります。キーは `Alt+W` です。ホストの Web フォームを、これまでに入力した内容を反映した状態でブラウザーに開きます。ダイアログは開いたままです。トークンは不要で、API 呼び出しも行わないため、API エラーの後でも使えます。

- GitHub には*ドラフト*用の URL パラメーターがないため、ドラフトのチェックは引き継がれません。GitLab では、タイトルに `Draft: ` のプレフィックスが付きます。
- GitLab はラベルを ID でしか受け付けないため、ラベルは説明の末尾に `/label ~"x"` のクイックアクションとして追加されます。
- 非常に長い説明は、URL が 6000 文字未満に収まるよう切り詰められ、`[...truncated: paste the rest]` という注記が付きます。
- Bitbucket Data Center には Web フォームのアドレスがないため、**<provider> で作成**ボタンはありません。

CLI：`gitpoulpi hosting issue create ... --web` と `gitpoulpi hosting pr --web` は、同じ URL を出力します。

## ツールバーのチップ

ツールバーの端（既定では右端）のチップには、チェックアウト中のブランチのリクエストが表示されます：`GH #12`（GitHub）、`GL !53`（GitLab）、`AZ !12`（Azure DevOps）、`BB #12`（Bitbucket）。色は状態を表します：緑はオープン、グレーはドラフト、紫は merge 済み、赤はクローズ。

- ホバーすると、タイトル、状態、`source → target` が表示されます。
- クリックするか `Ctrl+G g` を押すと、ブラウザーでリクエストが開きます。
- 右クリックすると、**ブラウザーで開く**、**URL をコピー**、**更新**、**プロファイルを切り替え**が表示されます。

リクエストのない push 済みブランチには、薄い `+ PR` チップが表示されます。クリックすると、pull request を開始できます。一致するプロファイルがない、ブランチが push されていない、またはホストに接続できなかったときは、チップは表示されません。ウィンドウが狭いときは、ツールバーのボタンの場所を確保するため、チップは非表示になります。

## 一覧が更新されるタイミング

一覧は次のタイミングで更新されます。

- リポジトリを開いたとき。
- チェックアウト中のブランチが変わったとき。
- push または fetch の後。
- *pull request と Issue の更新間隔（分）*（`general.hosting_refresh_minutes`、既定は 5、`0` でタイマーをオフ。設定 > 連携）ごと。
- 設定を閉じたとき。
- `r` を押したとき、*連携: Pull Requestと Issue を更新*を実行したとき、またはフィルターを変更したとき。

リクエストがウィンドウをブロックすることはありません。

ホストには、呼び出せる回数の制限があります。GitPoulpi は前回の応答の `ETag` を送り返すため、変更のない一覧はほとんど負荷がかかりません（GitHub では全くかかりません）。残りの呼び出し回数が少ないとホストが報告すると、ステータスバーでお知らせします。残りがなくなると、GitPoulpi は制限がリセットされるまでそのホストへの問い合わせを止め、一覧はそのままになります。

## 用語

GitHub、Azure DevOps、Bitbucket は「pull request」と呼び、GitLab は「merge request」と呼びます。*レビュー依頼の用語*（`ui.pr_terminology`、設定 > 連携）で、すべてのラベル、メニュー、パレットコマンド、メッセージで使う語を選びます。

- **自動（ホストごと）**：リポジトリのリンク先プロバイダーに従います（何もリンクされていない間は Pull Request）。
- **Pull Request** または **Merge Request**：常にその語を使います。

変更はすぐに反映されます。CLI と MCP は、常に「pull/merge request」と表記します。このページでは「pull request」と表記します。

## commit メッセージとブランチ名の参照

リポジトリの Issue または pull request がリンクされていると、commit 詳細の commit メッセージ内の参照に下線が付きます。クリックすると、リンク先のページが開きます。キーボードでは、パレットコマンドの*連携: commitの参照を開く（#issue / !request）...*を使います（メッセージに複数の参照があるときはメニューが出ます）。何もリンクされていないリポジトリには、リンクはありません。

何が参照として扱われるかは、ISSUES セクションのプロバイダーによって決まります。

| ISSUES のリンク先 | メッセージ内 | ブランチ名内 |
|---|---|---|
| GitHub、GitLab、Gitea、Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` と `AB#123` | `123-fix-login` |
| Jira | `ABC-123`（`ABC` がリンクされたプロジェクトのキーである場合のみ）。`UTF-8`、`SHA-256`、`ISO-8601` のようなテキストは、リンクになりません。 | 名前のどこかにある `ABC-123`（`feature/ABC-123-x`）、大文字 |
| Linear | `ENG-123`（`ENG` がリンクされたチーム、またはプロファイルのチーム一覧にあるチームのキーである場合のみ）。`UTF-8` はリンクになりません。 | Issue からブランチを作成したときの Linear 自身の `branchName`。名前のどこかにある `ENG-123` も読み取られます |
| YouTrack | `ABC-123`（`ABC` がリンクされたプロジェクト、またはプロファイルのプロジェクト一覧にあるプロジェクトの短い名前である場合のみ） | 名前のどこかにある `ABC-123` |
| Redmine | `#123`（Redmine がリンクされた ISSUES の取得元である場合のみ。pull request の取得元は自身の記号を保ちます） | `123-fix-login` |
| Trello | `https://trello.com/c/<shortLink>` のアドレス。キャッシュ済みのカードに番号 12 のものがあるときだけ `#12` | `12-fix-login` |

| PULL REQUESTS のリンク先 | メッセージ内 |
|---|---|
| GitLab、Azure DevOps | `!45` |
| GitHub、Bitbucket、Gitea、Forgejo | `#45` |

両方の一覧が `#N` に一致する場合は、Issue が優先されます。CLI：`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`。

**会話の中で。** Issue やリクエストのテキストは、そのホストの文法で読み取られ、上の表より広い範囲を認識します：`owner/repo#7`、`!7`、`GH-7`、`@login`、commit ID、そして[相互参照](conversations.md#相互参照)に挙げた他の形式です。リポジトリのリンクされた取得元が先に照合されるので、GitHub のコメント内の Jira キーも、アプリ内でその Issue を開きます。`gitpoulpi hosting markdown --body TEXT --id I` は、テキストの中にアプリが見つけるものを表示します。

### Issue の参照を挿入する

commit コンポーザーでは、概要の右端にある `#▾` ボタンで、リンクされた Issue の絞り込み可能な一覧が開きます。現在のブランチ名が指す Issue が先頭に、*（ブランチから）*という印付きで表示されます。`Enter` またはクリックで、その参照（`ABC-123`、`#12`、`AB#12`）がカーソル位置に挿入されます。キーボード：コンポーザーで `Alt+R`。パレット：*Commit: Issue の参照を挿入...*。リンクされた Issue の一覧がないときは、ボタンはグレーアウトされ、*先にサイドバーで Issue の一覧をリンクしてください*と表示されます。GitPoulpi が自動で参照を追加することはありません。
