[English](../hosting-integration.md) · [Čeština](../cs/hosting-integration.md) · [Deutsch](../de/hosting-integration.md) · [Español](../es/hosting-integration.md) · [Français](../fr/hosting-integration.md) · [Italiano](../it/hosting-integration.md) · **日本語** · [한국어](../ko/hosting-integration.md) · [Nederlands](../nl/hosting-integration.md) · [Polski](../pl/hosting-integration.md) · [Português (Brasil)](../pt-BR/hosting-integration.md) · [Português (Portugal)](../pt-PT/hosting-integration.md) · [Русский](../ru/hosting-integration.md) · [Svenska](../sv/hosting-integration.md) · [Türkçe](../tr/hosting-integration.md) · [Українська](../uk/hosting-integration.md) · [简体中文](../zh-CN/hosting-integration.md)

# ホスティングと Issue トラッカーの連携

GitPoulpi は次の製品に接続できます。

- **ホスティング**（pull request、Issue、アカウントからのクローン、SSH 鍵）：GitHub、GitHub Enterprise、GitLab、GitLab Self-Managed、Azure DevOps、Azure DevOps Server、Bitbucket、Bitbucket Data Center、Gitea、Forgejo（Codeberg を含む）。**Gerrit Code Review** はレビュー専用のホスティングで、そのチェンジが pull request として扱われ、Issue はありません。
- **Issue トラッカー**（Issue のみ）：Jira Cloud、Jira Data Center、Trello、Linear、YouTrack（Cloud と Server）、Redmine。
- **ローカルの Markdown ノート**（Issue のみ、トークンもネットワークも不要）：Markdown ノートのフォルダー（たとえば Obsidian の vault 内のフォルダー）。[ローカル Issue と Obsidian](local-issues.md)を参照してください。

どの連携も**パーソナルアクセストークン**を使います。GitPoulpi のアカウントも OAuth サインインもありません。連携は任意で、**リポジトリごとにオプトイン**です。リポジトリをリンクするまで、何も取得されません。トークンは認証情報ヘルパーに保存され、`config.toml` には保存されません。通常の `git` 操作（fetch、pull、push）には連携は不要です。`git` がすでに持っている認証情報が使われます。

**次に読むもの。** このページは設定の手順です：トークン、接続、リポジトリのリンク、SSH 鍵。アカウントを接続したら：

- サイドバーの PULL REQUESTS と ISSUES のセクション、Issue ボード、pull request や Issue の作成、ツールバーのチップ、commit メッセージ内の参照：**[pull request と Issue](pull-requests-and-issues.md)**。
- 1 件の Issue やリクエストのアプリ内ビュー（コメント、書き込み、投稿者名）：**[会話](conversations.md)**。
- コメントスレッドと保留中のレビューを使ったリクエストの閲覧とレビュー：**[pull request / merge request のレビュー](review.md)**。
- ワークスペース全体の pull request と Issue を 1 つの一覧で：**[Radar](radar.md)**。

## 1. ホストでトークンを作成する

接続フォームには*<host> でトークンを生成*というリンクがあります。適切なスコープがあらかじめ選択された、ホストのトークンページが開きます。手動で行う場合は次のとおりです。

| ホスト | 場所 | スコープ |
|---|---|---|
| GitHub、GitHub Enterprise | Settings > Developer settings > Personal access tokens (classic) > Generate new token | `repo`（リポジトリ、pull request、Issue）、`read:user`（自分が誰か）、`admin:public_key`（SSH 鍵の追加・削除のみ） |
| GitLab、GitLab Self-Managed | User settings > Access tokens > Add new token | `api`、`read_user` |
| Azure DevOps | User settings > Personal access tokens（`dev.azure.com/<organization>/_usersSettings/tokens`） | `Code (Read & write)`、`Work Items (Read & write)`。トークンは「All accessible organizations」ではなく、**その組織用に**作成してください。グローバルトークンは 2026-12-01 に使えなくなります。 |
| Azure DevOps Server | 自分のサーバーの同じページ（`https://<server>/<collection>/_usersSettings/tokens`） | 同じ 2 つのスコープ |
| Bitbucket | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token with scopes | `read:user:bitbucket`、`read:workspace:bitbucket`、`read:repository:bitbucket`、`write:repository:bitbucket`、`read:pullrequest:bitbucket`、`write:pullrequest:bitbucket`。SSH 鍵を扱う場合は、さらに `read:ssh-key:bitbucket`、`write:ssh-key:bitbucket`、`delete:ssh-key:bitbucket`。 |
| Bitbucket Data Center | プロフィール画像 > Manage account > HTTP access tokens | `Repository (Read & write)`、`Projects (Read)` |
| Gitea、Forgejo（Codeberg） | Settings > Applications > Generate New Token（`<host>/user/settings/applications`） | `user` の読み取り、`repository`、`issue`、`user` の読み書き（最後のものは SSH 鍵用）。Gitea は作成後にスコープを表示しないため、GitPoulpi は不足を事前に警告できません。拒否された呼び出しがその旨を伝えます。 |
| Linear | Settings > Account > Security & access > Personal API keys > New API key（`linear.app/settings/account/security`） | Read、Write、Create issues。キーはユーザー自身として動作します。 |
| YouTrack（Cloud と Server） | プロフィール > Account Security > New token（`<host>/users/me?tab=account-security`） | `YouTrack` スコープ。永続トークンはユーザー自身として動作します。 |
| Gerrit | Settings > HTTP Credentials > Generate new token（新しいサーバー。古いサーバーは Generate new password）（`<host>/settings/#HTTPCredentials`） | なし。トークンは **HTTP パスワード**で、ユーザー自身として動作します。ユーザー名も一緒に指定します。 |
| Redmine | My account > API access key > Show（`<host>/my/account`）。管理者が **Administration > Settings > API > Enable REST web service** をオンにしている必要があります。 | なし。キーはユーザー自身として動作します。 |
| Jira Cloud | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token（スコープなしのクラシックなもの） | なし。トークンはユーザー自身として動作します。 |
| Jira Data Center | Profile > Personal Access Tokens | なし。トークンはユーザー自身として動作します。 |
| Trello | まず `trello.com/apps/admin` で **API キー**を取得します（Power-Up を作成し、API キーを生成）。次に `trello.com/1/authorize?...&key=<your key>` で**トークン**を取得します。 | `read`、`write` |

トークンには「GitPoulpi」などの名前と有効期限を付けてください。トークンはコピーしておきます。ホストが表示するのは 1 回だけです。

## 2. 接続する

1. `Ctrl+,` を押して**連携**を選びます。
2. 左側で製品を選びます。製品は**ホスティング**と **Issue トラッカー**の 2 つのグループに分かれています。
3. **接続**を押します。同じ製品の 2 つ目のアカウントを追加するには、**+ プロファイル追加**を使います。

   ![連携ページ：左に製品の一覧、GitHub 用の接続ボタン](../images/settings-integrations.png)

4. フォームに入力し、トークンを貼り付けます。トークンは入力中は平文で表示されるため、周りに誰もいないことを確認してください。フォームには、必要なスコープが一覧されています。

   ![接続フォーム：パーソナルアクセストークンのフィールド、スコープ repo・read:user・admin:public_key、github.com でトークンを生成するリンク、接続とキャンセルのボタン](../images/settings-connect.png)

5. **接続**を押します。GitPoulpi はバックグラウンドでホストに対してトークンを確認します。ページには、名前とログイン名、緑色の*接続済み*、SSH 鍵のセクションを含むカードが表示されます。必要なスコープの一部が欠けているトークンでも接続はでき、足りないスコープは赤で一覧されます。

カードでは、**編集**でフォームを再び開きます。**切断**はトークンを削除し、プロファイルは残します。**削除**は、確認の後に両方を削除します。

連携ページのキーボード操作：`Tab` / `Shift+Tab` で製品一覧、プロファイルのストリップ、フォームの間を移動します。`a` でプロファイルを追加、`e` で編集、`d` または `Delete` で削除、`o` でトークンのページを開き、`Esc` で 1 段階戻ります。

### 各フォームで求められるもの

*トークンを生成*のリンクは、ホストまたは組織を入力した後でないと機能しません。

- **GitHub、GitLab：** トークンだけです。**GitHub Enterprise** と **GitLab Self-Managed** は、**ホストドメイン**（`gitlab.example.com`。`http://` と入力しない限り https になります）も求めます。ドットを含まない名前（Docker のサービスや LAN 内のマシン）には、ポートか `http://` が必要です：`gitea:3000`、`http://nas`。自前でホストするサーバーは平文の http でも構いません。例：`http://localhost:8080` のローカルコンテナ。平文の http で送ったトークンは暗号化されずに流れるため、http はローカルまたは信頼できるネットワークでのみ使ってください。
- **Azure DevOps：** **組織**（`dev.azure.com/<organization>` の名前）。`https://<org>.visualstudio.com` のアドレスも受け付けます。2 つの組織には 2 つのプロファイルが必要です。リポジトリは、リモートが指す組織のプロファイルを使います。**Azure DevOps Server** は**コレクション URL**（例：`tfs.corp/tfs/DefaultCollection`）を求めます。GitPoulpi は Azure DevOps ではシステムの認証情報ヘルパーを使いません。Git Credential Manager が保存しているのはサインイン用トークンで、パーソナルアクセストークンではないためです。
- **Bitbucket：** **Atlassian アカウントのメール**と API トークン。メールが空の場合、トークンは bearer トークンとして送信されます。ワークスペースまたはリポジトリのアクセストークンにはこちらを使います。GitPoulpi はトークンからワークスペースを見つけます。**Bitbucket Data Center：** **サーバー URL**と HTTP アクセストークン。
- **Gitea、Forgejo：** **ホストドメイン**とトークン。**Forgejo** は `https://codeberg.org` を初期値として始まります。セルフホストのサーバーでは置き換えてください。Gitea には公開の既定値はありません。ローカルのサーバーなら `http://localhost:3000` のアドレスも使えます。どちらの種類も 1 つのクライアントを共有します（Forgejo は Gitea のフォークで、同じ `/api/v1` を持ちます）。トークンは `Authorization: token <token>` で送られます。これらにはシステムの認証情報ヘルパーは使われません。
- **Jira Cloud：** **サイト URL**（`acme.atlassian.net`）、**Atlassian アカウントのメール**、クラシックな API トークン。スコープ付きの Jira トークンには対応していません。**Jira Data Center：** **サーバー URL**とパーソナルアクセストークン。
- **Trello：** **API キー**と**トークン**。API キーは公開情報なので、`config.toml` に（`account` として）保存されます。トークンは秘密であり、認証情報ヘルパーに保存されます。フォームには 2 つのリンクがあります。*Trello で API キーを取得*と、キーを入力した後に表示される*トークンを生成*です。キーとトークンはヘッダーで送信され、URL には含まれません。
- **Linear：** **API キー**のみです。Linear の公開ホストは 1 つなので、入力するアドレスはありません。キーは `Authorization` ヘッダーにそのまま送られます（Linear は個人用キーに `Bearer` を付けません）。commit メッセージの `ENG-12` のような参照で正しいページが開くようにするには、プロファイルの URL にワークスペースを保存してください（`https://linear.app/acme`。CLI からは `--url`）。Issue の行自体は常に正確なアドレスを持っています。
- **YouTrack：** **サイト URL**（Cloud なら `acme.youtrack.cloud`、それ以外は自分のサーバー。コンテキストパスがある場合は含めます。例：`host/youtrack`）と永続トークン（`Authorization: Bearer` で送信）。同じクライアントが Cloud と Server の両方に対応します（`<url>/api`）。
- **Gerrit：** **サーバー URL**（コンテキストパスがある場合は含めます）、**ユーザー名**、トークンとしての HTTP パスワード。これらは HTTP の Basic 認証で送られます。GitPoulpi はサーバーの `/a/` の REST API と通信し、各応答の先頭にある `)]}'` の行を無視します。ユーザー名は秘密ではないため `config.toml` に（`account` として）保存され、パスワードは認証情報ヘルパーに保存されます。リポジトリは PULL REQUESTS で Gerrit の*プロジェクト*にリンクされます（そのチェンジがそこに一覧表示されます）。Gerrit には Issue がなく、チェンジは push して作成します（[レビュー](review.md#gerrit)の*レビュー用の push*を参照）。
- **Redmine：** Redmine の**サイト URL**（サブパスがある場合は含めます。例：`host/redmine`）と API キー（`X-Redmine-API-Key` で送信）。Redmine が 403 または HTML のページを返す場合は、REST Web サービスがオフです（管理者に依頼するか、**Administration > Settings > API** でオンにしてください）。

Jira、Trello、Linear、YouTrack、Redmine は Issue トラッカー専用です。pull request、クローンするリポジトリ、SSH 鍵はありません。Bitbucket には Issue がありません（Atlassian が Bitbucket Issues を廃止しました）。代わりに、Bitbucket リポジトリの ISSUES セクションを Jira プロジェクトにリンクしてください。

## 3. トークンの保存場所

トークンは、システムの**認証情報ヘルパー**（Windows では Git Credential Manager、Linux では libsecret または `pass`、macOS では Keychain）に保存されます。実在するホストの認証情報と混同されない、専用のエントリが使われます。`config.toml` に書き込まれることも、表示されることもありません。

認証情報ヘルパーが設定されていない、またはトークンの保存を拒否された場合、GitPoulpi はトークンを**このセッションの間だけメモリ上に**保持し、その旨を表示します。再起動後は、もう一度接続してください。

GitPoulpi は、プロファイルのトークンを次の順序で探します。

1. 環境変数 `GITPOULPI_TOKEN_<PROFILE ID>`。保存せずにトークンを渡したいとき（CI など）に使います。
2. プロファイルに保存されたトークン（`token_ref`）。
3. GitHub と GitLab のプロファイルのみ：同じホストを使う有効なプロファイルが他にない場合、そのホストのシステム認証情報ヘルパー。

各プロファイルは、`config.toml` の `[[integrations]]` エントリです。[設定リファレンス](config-reference.md#integrationsホスティングと-issue-トラッカー)を参照してください。

## セキュリティチェック

保存されたトークンの安全性は、それを送る先のアドレスで決まります。そのため GitPoulpi は、連携ごとに何に接続されていたかを記憶し、それが変わると停止します。

- **アドレス。**連携を初めて使う（または接続する）ときに、そのアドレスが `trust.toml` に記録されます。保存先は `config.toml` ではなく、ユーザー専用のプライベートデータフォルダー（`<data>/private/`）です。設定を編集しても、信頼する範囲を広げることはできません。設定内のアドレスが記録されたものと異なる場合は、そこへ何も送られず、トークンも読み込まれません。同じアドレスの別表記（`HTTPS://Git.Example.com:443/`）は同じアドレスとして扱われ、紛らわしいホストは別のものとして扱われます。
- **アカウント。**トークンを接続した後、およびその後は 1 日に 1 回程度（`GITPOULPI_TOKEN_*` や認証情報ヘルパー由来のトークンでは、1 回の実行につき 1 回）、GitPoulpi はホストにトークンの持ち主を問い合わせ、ログインではなくアカウントの **id** を比較します。トークンが別の人のものになっていた場合は停止します。
- **証明書。**証明書が既知の認証局につながらないサーバー（プライベート CA、自己署名のサーバー）は、承認した鍵そのものに限って受け入れられ、TLS ハンドシェイクで、サーバーがその鍵を持っていることは引き続き確認されます。厳密なピン留めをオンにすると、証明書が公的なルートにつながっていても、鍵が一致する必要があります。その代わり、鍵のローテーションのたびに再度確認されるため、オプトインになっています。公開ホスト（github.com、gitlab.com、codeberg.org、bitbucket.org、dev.azure.com、`*.atlassian.net` など）には、証明書の記録を残しません。
- **平文 http。**ループバック以外の `http://` アドレスは、その連携で許可しない限り拒否されます。

すべてのリクエストは、連携自身のアドレス（公開ホストの場合は `api.github.com` などの API ホストも）に限定されます。それ以外へのリクエストは送信前に拒否され、別のアドレスへのリダイレクトには従いません。

**このチェックができる前に接続した連携**は、アップグレード後に初めて使われたときに、そのときの設定から静かに記録が作られます。現在の信頼のレベルはそのままで、何も尋ねられません。

### コマンドラインから

停止したコマンドは終了コード 2 で失敗し、何が変わったかを出力します（[CLI リファレンス](cli-reference.md)を参照）。`gitpoulpi hosting integrations trust <id>` は信頼済みの内容を表示します。`--accept-origin <url>`、`--accept-account <id>`、`--accept-certificate <sha256>`、`--strict-tls on|off`、`--allow-insecure-http on|off` は、それぞれ 1 つずつ、正確な値を指定して受け入れます。新しいアドレスを受け入れた後は、トークンをもう一度入力してください。切断（`hosting logout`）または連携の削除を行うと、そのトークン、信頼の記録、そこからキャッシュされた内容（クローンダイアログの保存済みリポジトリ一覧、Radar のアカウント一覧、そのホストのリモートレビューのドラフト）が削除されます。

エージェント（MCP）は何も受け入れられません。セキュリティチェックにはあなたが必要だと伝えられます。

### ターミナル UI では

停止したリクエストがあると、作業中の画面の上に**連携のセキュリティチェック**ダイアログが開きます。何が変わったか（古いアドレスと新しいアドレス、古いアカウントと新しいアカウントとその id、または古い証明書と新しい証明書。4 文字ずつに区切った SHA-256 フィンガープリント、サブジェクト、発行者、有効期間を載せた小さな表）と、トークンが送信されなかったことが表示されます。回答には番号が付いています。

1. **切断（保存済みトークンを削除）**は、保存済みのトークン、信頼の記録、その連携からキャッシュされた内容を削除します。
2. 新しいアドレス、アカウント、証明書を**信頼**します（暗号化されていないアドレスの場合は**平文 http を許可**）。
3. **キャンセル**は何も変更しません。

矢印キーか数字キーを押すまでどの回答も強調表示されないので、うっかり Enter を押しても何も受け入れられません。Esc と閉じる ✕ はキャンセルで、ダイアログの外側のクリックは何もしません。長い本文は、ホイール、Page Up/Page Down、スクロールバーでスクロールできます。複数のチェック（連携ごと、変更の種類ごとに 1 つ）はキューに入り、順番に尋ねられます。キャンセルしたチェックは、変更がない間は再び表示されません。コマンドパレットの**連携: セキュリティチェックを確認**で呼び戻せます。

新しい**アドレス**を信頼すると、連携のフォームがトークン欄にフォーカスした状態で開き、「トークンの再入力をおすすめします。」というヒントが表示されます。保存済みのトークンは、そこへは一度も送られていません。フォームでは、意図して変更したアドレスは、それ用に入力したトークンと一緒に受け入れられます。トークンがない場合、フォームは入力を求めます。

フォームには 2 つのスイッチもあります。**証明書を厳密にピン留め**（https アドレス用）は、接続時にサーバーが提示する証明書の鍵をピン留めします。**平文 http を許可（暗号化なし）**は、このマシンではない `http://` アドレスにだけ表示され、トークンが平文で送られることを示す赤い警告が付きます。接続済みのカードには、ピン留めしたフィンガープリントが表示されます。

### トークンにできることと、その期限

接続時には、ホストにトークンにできることも問い合わせます。GitHub（クラシックトークン）、GitLab、Bitbucket（スコープを返す場合）はスコープを報告します。カードには、黄色の「このトークンは GitPoulpi に必要な範囲を超える権限を持っています：」の行が表示され、GitPoulpi が使う範囲を超えるスコープが一覧されます。`delete_repo`、`admin:org`、`sudo` など機微なものは赤で表示され、有効期限のあるきめ細かいトークンの使用をおすすめするメッセージが付きます。クラシックの GitHub トークン（`ghp_...`）には、スコープが適切でもこのメッセージが表示されます。きめ細かい GitHub トークンは、権限を問い合わせることができないため、有効期限だけが表示されます。ホストが有効期限を返す場合、カードには期限の日付が表示されます。期限のないトークンには、リマインダーは表示されません。ほかの製品はスコープを報告せず、警告も出しません。同じ項目は、`hosting integrations login` と `test` の JSON（`excess_scopes`、`token_flavor`、`recommend_fine_grained`、`expires_at`）にもあります。接続時に読み取られるため、カードに表示されるのは、接続したセッションに限られます。

## 4. リポジトリをリンクする

リンクは明示的で、リポジトリごとに行います。プロファイルが接続されていると、Refs サイドバーの TAGS の後に **PULL REQUESTS** セクションと **ISSUES** セクションが表示されます。各セクションには、プロバイダーとプロジェクトの 2 つのドロップダウンがあります。

- プロバイダーのドロップダウンは、リポジトリのリモート（`origin` が先頭）に一致するアカウントを、`owner/name · origin · GitHub` のような行で**提案**します。1 つ選ぶと、プロバイダーとプロジェクトの両方が設定されます。
- 自動でリンクされるものはありません。Jira、Trello、Linear、YouTrack、Redmine は、それらを指すリモートがないため提案されません。自分で選んでください。
- ドロップダウンの先頭にある**なし（このリポジトリのリンクを解除）**は、セクションのリンクを解除します。自動で再リンクされることはありません。
- リンクは、リポジトリの `.gitpoulpi/repo-state.toml` ファイルに保存されます。

2 つのセクションは**別々に**リンクされます。たとえば、pull request は Azure DevOps から、Issue は Jira から取得できます。

| プロバイダー | PULL REQUESTS のプロジェクト | ISSUES のプロジェクト |
|---|---|---|
| GitHub、GitLab | `owner/name`、`group/sub/proj` | 同じ |
| Azure DevOps | `Project/Repo`（組織はプロファイルから決まる） | `Project`（ワークアイテム） |
| Gitea、Forgejo | `owner/name` | 同じ |
| Bitbucket | `workspace/slug`（Data Center：`PROJ/slug`） | なし：Jira をリンク |
| Jira | なし | プロジェクトキー（例：`ABC`） |
| Trello | なし | ボード |
| Linear | なし | チームキー（例：`ENG`。チームが一覧されます） |
| YouTrack | なし | プロジェクトの短い名前（例：`ABC`） |
| Gerrit | Gerrit のプロジェクト名（例：`team/tool`） | なし |
| Redmine | なし | プロジェクトの識別子（例：`bench`。プロジェクトのアドレスの最後の部分） |

ISSUES のプロジェクト一覧は検索できます。一覧にないプロジェクトを入力することもできます。

**追加クエリ。** Issue のソースには、プロジェクトのドロップダウンの下に任意のフィルター行があります。プロジェクトと組み合わせて（AND で）適用されます。

- Jira：**JQL** の断片（`component = UI AND labels = backend`）。
- Azure DevOps：**WIQL** の条件（`[System.AssignedTo] = @Me`）。
- YouTrack：**YouTrack クエリ**（`#Unresolved Type: Bug`、`tag: ui or tag: ux`）。括弧で囲まれて送られるため、`or` でプロジェクトの範囲が広がることはありません。独自の `sort by:` は、既定の `sort by: updated desc` を置き換えます。
- Redmine：なし（セクションの状態フィルターと「自分に割り当て」で十分です）。
- Linear：なし（セクションの状態フィルターと「自分に割り当て」で十分です）。
- Trello：表示する**リスト**にチェックを入れるドロップダウン。何もチェックしなければ、開いているすべてのリストが表示されます。
- GitHub と GitLab では無視されます。

`Enter` で入力したクエリを保持し、`Esc` で破棄します。パレット：*Issue：クエリ / リストを編集...*。

CLI からは：`gitpoulpi hosting link pulls --id <profile>`（または `issues`）、リンク解除は `--clear`、追加クエリは `--filter`。[CLI リファレンス](cli-reference.md#コマンド)を参照してください。

### 接続したアカウントからクローンする

接続したプロファイル（GitHub、GitLab、Gitea、Forgejo、Azure DevOps、Bitbucket、Gerrit）は、**クローン**ダイアログのソースにもなります（Home タブの**クローン**ボタン、またはそこで `c`）。各プロファイルは **URL でクローン**の下に独立したソースとして並びます（`GitHub · name`、`GitLab · name`）。GitHub と GitLab が混ざることはありません。1 つ選ぶとそのリポジトリが読み込まれ、所有者ごとにまとまります。最初に自分のアカウント、続いて組織またはグループごとに 1 つのセクション（GitLab のサブグループなら `quarry/tools`）で、それぞれ件数つきの見出しの下に並びます。鍵のマークは非公開リポジトリを示します。

![接続した GitHub アカウントのクローンダイアログ：所有者ごとのセクションと、非公開リポジトリの鍵のマーク](../images/clone-hosted.png)

検索ボックスはすべてのセクションにまたがって絞り込みます（名前、フルパス、説明）。一致のないセクションは消えます。行で `Enter` を押すと、そのリポジトリの HTTPS URL とフォルダー名がクローンの入力欄に入ります。

![検索で一覧を 1 つの組織に絞り込んだクローンダイアログ](../images/clone-hosted-search.png)

以前に開いたことのあるプロファイルの一覧は、ホストに再度問い合わせている間も、保存されたコピーからすぐに表示されます（ラベルの横に*更新中...*）。応答が届くと、新しいリポジトリが現れ、なくなったものは消え、ハイライトと選択中のリポジトリはそのままです。更新に失敗した場合は、保存済みの一覧が注記つきで残り、理由はアクティビティログにあります。

ホストがトークンを拒否すると、ダイアログは一覧の代わりに理由つきで*リポジトリの一覧を取得できませんでした*と表示します（保存されたコピーは削除されます）。ほかのソースは引き続き使えます。

## 5. SSH 鍵

### プロファイルごとの SSH 鍵

接続済みのプロファイルには、カードの下に **SSH 鍵**セクションがあります。鍵がない場合は、次のいずれかを選びます。

- **既定の鍵**：ssh-agent または `~/.ssh`。何も変わりません。
- **新しい鍵を生成**：任意のタイトル（既定は `GitPoulpi <host> <date>`）。**公開鍵を <host> に追加**にチェックを入れると、ホストにまだなければ GitPoulpi がアップロードします。鍵はパスフレーズなしで `~/.ssh/gitpoulpi_<profile id>_ed25519` と `.pub` に書き込まれます。`ssh-keygen` は不要です。既存のファイルが上書きされることはありません。
- **カスタム鍵**：パスを入力するか、秘密鍵ファイルを**参照**で選び、**この鍵を使用**を押します。

鍵が設定されていると、セクションには `title (SHA256:...)` と次のボタンが表示されます。

- **解除**：鍵を外します。GitPoulpi が追加した鍵は、ホスト上でも削除します。
- **削除**：同じことに加えて、鍵ファイルも削除します。両方のパスを一覧する確認が表示されます。

### 1 つのリポジトリ用の SSH 鍵

1 つのリポジトリで特定の鍵を使わせるには、タブのメニュー（`Ctrl+T`、またはタブを右クリック）を開いて **SSH と認証情報...**を選ぶか、パレットコマンドの*リポジトリ: SSH と認証情報...*を実行し、SSH の下で**鍵ファイル**（連携プロファイルの鍵なら**アカウントの鍵**）を選びます。この選択は GitPoulpi の専用フォルダーに保存され、リポジトリにも git の設定にも何も書き込まれません。[設定](settings.md#リポジトリ別ssh-と認証情報)を参照してください。

以前の「このリポジトリの SSH 鍵」の固定による独自の `core.sshCommand` がまだあるリポジトリでは、そのページに読み取り専用で表示され、**GitPoulpi に移す**が使えます。

CLI と MCP：`gitpoulpi repo-auth`（リポジトリごとの選択）と、`core.sshCommand` の固定そのものには `gitpoulpi ssh-command show | set --key <file> | clear`。

設定 > SSH と認証情報のグローバルな鍵（*秘密鍵のパス*、または*鍵ペアを生成*）は、独自の選択のないすべてのリポジトリに適用されます。リポジトリの鍵ファイルまたは固定が優先されます。

## 6. 履歴の書き換えのためのブランチ保護

履歴を書き換えるとき（[ワークベンチ](history-rewrite.md)）、GitPoulpi はホストに、どのブランチが保護されているかを問い合わせ、既定ではそれらを書き換えの対象外にします。問い合わせるのは、**連携した**リポジトリ（サイドバーのプルリクエストまたは Issue のセクション、または固定したプロファイル）だけで、そのプロファイルのトークンを使います。トークンが読み取れない場合は「不明」になるだけで、何も失敗しません。各プロバイダーが何を返し、トークンに何が必要かは次のとおりです。

| ホスト | 読み取るもの | トークンに必要なもの |
|---|---|---|
| GitHub（と Enterprise Server） | ブランチの `protected` フラグと、有効なブランチのルールセット（`~DEFAULT_BRANCH`、`~ALL`、`refs/heads/...` のパターン） | `repo`（公開リポジトリなら `public_repo`）を持つクラシックトークン、または Metadata（読み取り）を持つ細粒度トークン。トークンが読めないルールセットはその旨が表示され、クラシックのルールは引き続き考慮されます |
| GitLab（gitlab.com とセルフマネージド） | `protected_branches`（名前と `*` パターン） | `read_api` と、リポジトリの設定を見られるロール |
| Azure DevOps（Services と Server） | リポジトリで有効な、ブロックするブランチポリシー（`Exact`、`Prefix`、`DefaultBranch` のスコープ） | Code（読み取り）。「強制 push」の権限はセキュリティ設定であり、読み取りません |
| Bitbucket Cloud | push を止めるブランチ制限（`push`、`force`、`delete`） | `repository:admin`。ブランチングモデルへの制限は解決されません |
| Bitbucket Data Center | 強制 push を止めるブランチ権限（`fast-forward-only`、`pull-request-only`、`read-only`） | リポジトリの管理者 |

保護の設定ページ（と、既定のブランチ、リポジトリの削除の設定ページ）は、GitHub、GitLab、Azure DevOps、Bitbucket について、リポジトリが連携されていなくても、リモートの URL から組み立てられ、ホストが push を拒否したときに、3 つの対処とともに表示されます。

## 7. Radar のアカウント全体の一覧

Radar は通常、クローンしたリポジトリを監視します。プロファイルは、自分の `[integrations.radar]` テーブルの 3 つのスイッチで、
クローンの有無を問わずホスト全体であなたに関わるものを一覧にすることもできます（すべて既定でオフ。
[設定リファレンス](config-reference.md#integrationsradarアカウント全体の一覧)を参照）。

- **レビュー依頼**：あなたのレビューを求めている未完了のリクエスト。
- **未完了のリクエスト**：自分の名前空間と、所属する各組織／グループ、または指定した名前空間の未完了のリクエストすべて。ホスト全体ではありません。
  プロファイルの**名前空間**（設定 > 連携 > Radar で 1 行に 1 項目、必要なら正規表現）は 3 つの一覧すべてを絞り込みます。
  行は、リポジトリの完全なパスと等しいか、`/` で区切られたパスのプレフィックスのときに一致します（`orbit` は `orbit/products/x` を残し、`orbitx/y` は残しません）。
  ホストが返した行に適用され、ホストの検索が対応していればサーバー側でも適用されます（GitHub の `user:` / `repo:` 修飾子）。
- **割り当てられた Issue**：自分に割り当てられた未完了の Issue。

GitHub、GitLab（Self-Managed を含む）、Gitea、Forgejo、Azure DevOps（Services と Server）、Bitbucket Data Center、Gerrit が対応しています。
それぞれの問い合わせ方は次のとおりです。

| ホスト | リクエスト | Issue |
|---|---|---|
| Azure DevOps | 組織全体のアクティブな pull request（自分の待ち分は `searchCriteria.reviewerId`。すでに投票したリクエストは除外）。サーバーが組織レベルのルートを拒否した場合は、最初の 20 プロジェクトを1つずつ問い合わせます。`namespaces` はプロジェクト名 | 自分に割り当てられた、Done、Closed、Removed 以外のワークアイテム |
| Bitbucket Data Center | ダッシュボード：レビュー依頼にはまだ承認していない `REVIEWER` ロール、未完了のリクエストには `AUTHOR` と `PARTICIPANT`。`namespaces` はプロジェクトキー | なし（Data Center には Issue トラッカーがなく、注記でそう表示されます） |
| Gerrit | `is:open reviewer:self -owner:self -is:ignored`。未完了のリクエストは名前空間（プロジェクトパスのプレフィックス）ごとの `is:open projects:<prefix>`、指定がなければ自分のチェンジとレビューしたチェンジ。作業中（work-in-progress）のチェンジはドラフト | なし |

Bitbucket Cloud にはアカウント全体の行がありません：リポジトリをまたいでリクエストを一覧にするエンドポイントがなく、
リポジトリごとに問い合わせると呼び出しが多すぎるためです。クローンしたリポジトリは独自の一覧を保持し、
`gitpoulpi hosting radar --id <bitbucket>` もそう伝えます。Gerrit が返す上限より多くのチェンジがある場合、一覧は注記付きで切り詰められます。

ドラフトは、保持を指定しない限り除外されます。各一覧には上限があり、種類ごとに 300 行で、ほかに何件あったかが注記されます。
GitHub では検索 API が 1 分に 30 回しか許可しないため、1 ラウンドあたりの検索呼び出しは数回に抑えられます。トークンで読み取れない名前空間は
注記になり、ほかは読み込まれます。拒否されたトークンやレート制限は、そのプロファイルだけを失敗にします。回答は 5 分間再利用されます
（設定フォルダーの `radar-account.json`）。

設定 > 連携では、接続済みのコードホストのプロファイルの SSH 鍵の下に **Radar** ブロックがあります：一覧ごとに 1 つのチェックボックス
（*自分のレビュー待ちのリクエスト*、*未完了のすべてのリクエスト*、*ドラフトを含める*、*自分に割り当てられた課題*。最後のものは Issue のないホストでは表示されません）、
*名前空間*フィールド（1 行に 1 項目で、名前はそのグループまたは所有者とその配下すべてに一致します。*正規表現*にチェックを入れると 1 行ごとに正規表現になります。空なら自分と所属するすべて）、そして*クローンしていないリポジトリのリクエスト*の選択：
*確認する*、*クローンする*、*API 経由でレビュー*。変更するたびにすべて保存されます。アカウント全体の問い合わせがないホスト（Bitbucket Cloud）では、
代わりにそう伝える1行が表示されます。コマンドパレットからは *設定: プロファイルのレーダーオプション...* でこのブロックに移動でき、
設定の検索では各コントロールをラベルで見つけられます。

コマンドラインから：`gitpoulpi hosting integrations radar <id> --review-requests on` で一覧をオンにし、
`gitpoulpi hosting radar --json` で一覧を出力します（`--refresh` でキャッシュを使いません）。AI エージェントは、MCP で同じ 2 つのコマンドを使えます
（`hosting_radar`、`hosting_integrations_radar`）。

## 8. GitHub App の bot として動作する

人の名前を表示してはいけないプロジェクトでは、ホスト上のあらゆる操作を **GitHub App** を通じて行えます：すべてのコメント、レビュー、クローズや再オープン、ラベルの変更が `<app-slug>[bot]` として表示されます。これは 1 つのリポジトリ単位で機能し、他のリポジトリは自分のトークンのままです。

1. 組織用の GitHub App を作成し、**Issues** と **Pull requests** の権限を *Read & write* にして（さらに、pull request を一覧表示してレビューできるよう *Contents: read* も）、リポジトリにインストールします。
2. **インストールトークン**を標準出力に出力する小さなスクリプトを書きます（App の秘密鍵で有効期間の短い JWT に署名し、それを 1 時間有効なトークンと交換します）。GitPoulpi が鍵を目にすることはありません。
3. **設定 > リポジトリ別の設定 > SSH と認証情報**で、HTTPS の**トークンコマンド**を選び、**コマンド**にそのスクリプトを、**ホスト**にホスト（`github.com`）を入力し、**ホスティング API に使う（Issue、pull request）**にチェックを入れます。保存してコマンドを確認します（確認は必ず人が、その SHA-256 とともに行います）。コマンドラインからは：`gitpoulpi repo-auth set --https token-command --hosts github.com --use-for-hosting --yes --command ./app-token.sh`。

そのリポジトリで変わること：

- Issue と pull request の呼び出し（一覧、作成、コメント、レビュー、マージ、クローズ、ラベル）は、git 自身のサインインと同じ 30 秒の制限、メモリ内キャッシュ、伏せ字処理で、そのコマンドのトークンを使います。トークンの期限切れでホストが 401 を返すと、コマンドをもう一度だけ実行します。コマンドを実行できない場合、呼び出しは拒否されます：個人トークンにフォールバックすることはありません。
- 「自分」はありません：インストールトークンは自分が誰かを問い合わせられません。セクションのヘッダーにはプロファイル名に **(App bot)** が付き、「自分」に基づくフィルター（*自分に割り当て*、*自分の*、*自分のレビュー待ち*）には一致するものがありません。エラーとしては何も表示されません。
- **投稿者名。** GitPoulpi が投稿するメッセージはすべて、投稿される名前とともに表示されます（[会話](conversations.md#投稿者名)のヘッダーの下とそのコメントボックスの *my-app[bot] として投稿（GitHub App bot）*、CLI の回答の `posted_as`）。個人トークンならそれは自分のログイン名（`GET /user`）です。App トークンなら**投稿者名**の名前、たとえば `my-app[bot]` です：すべてのリポジトリ共通なら**設定 > 連携**（`general.hosting_post_as`。`gitpoulpi config set general.hosting_post_as my-app[bot]` でも可）、1 つのリポジトリだけなら**設定 > リポジトリ別の設定 > SSH と認証情報 > ホスティング**で指定でき、後者が優先されます。空欄の場合、GitPoulpi は GitHub に問い合わせ（GraphQL の `viewer { login }`）、それができなければ最初に投稿したコメントの作成者から名前を学習し、リポジトリごとに保持します。その名前は自分のコメントを見分ける手段でもあるので、編集と削除はそれらにだけ提供されます（App のコメントは bot アカウントからのものでなければなりません）。名前が分かるまでは、このインストールが投稿したコメントだけを変更できます。
- App に担当者を割り当てることはできません。この設定は、下のブロックされた割り当てと組み合わせてください。
- **ホスティング API に使う**のオン・オフの切り替えには、新しいコマンドと同じく人の操作が必要です。

このようにリポジトリのトークンを使うのは GitHub のプロファイルだけで、他のホストはプロファイル自身のトークンのままです。（GitLab のプロジェクトまたはグループのアクセストークンも bot です：それはプロファイルのトークンで、*プロジェクトの bot* と表示され、何も設定する必要はありません。）

### リポジトリで担当者の割り当てをブロックする

**設定 > リポジトリ別の設定 > SSH と認証情報**には**担当者の割り当て**の行があります（既定は**許可**、または**ブロック**。すぐに保存されます）。ブロック中は、誰が頼んでも、そのリポジトリの Issue と pull request で担当者の割り当てはできません：

- 担当者ピッカーは、新規 Issue と新規 pull request のフォームから外れます（理由を示すメモが付きます）。パレットの*ホスティング: 担当者を編集...*は理由つきでグレー表示になり、行メニューは開く代わりにその旨を表示します。
- すべてのクライアントが、何かを送る前に割り当てを拒否します。Issue や pull request の作成時に渡された担当者も同様です：CLI は `{"refused": {"reason": "assignment_blocked", ...}}` を終了コード 2 で返し、MCP 経由のエージェントにも同じ拒否が返ります。
- すでにいる担当者は表示されたままです（読み取り専用）。

`gitpoulpi hosting assignment` は設定を表示し、`--block` でオン、`--allow` で再びオフにします。再び許可できるのは人だけです（MCP サーバーからはできません）。

## 9. コマンドラインから

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
gitpoulpi hosting assignment --block
```

すべてのコマンドは [CLI リファレンス](cli-reference.md#コマンド)を参照してください。トークンを引数として受け付けることはなく（`hosting login` は、OS の資格情報ストアが保持するときは `stored: "os"` を、ストアが拒否してトークンがこのセッションの間だけメモリにあるときは `stored: "session"` と `store_error` を表示します）、MCP サーバーは `hosting login` を提供しないため、トークンがエージェントを通ることはありません。

## トラブルシューティング

| 症状 | 対処 |
|---|---|
| *接続*が認証エラーで失敗する | トークンが間違っている、期限切れ、または失効しています。新しく作成してください。 |
| スコープが赤で一覧されている | トークンにそのスコープがありません。接続は機能しますが、その機能は使えません。ホストでスコープを追加し、**編集**から新しいトークンを貼り付けてください。 |
| サイドバーに PULL REQUESTS セクションがない | 有効なプロファイルが接続されていません。接続してから、セクションのドロップダウンでリポジトリをリンクしてください。 |
| 一覧に*再接続*の行がある | トークンが失われた（認証情報ヘルパーなしでの再起動）か、期限切れです。行をクリックしてプロファイルを開いてください。 |
| セルフマネージドのインスタンスで失敗する | **ホストドメイン**にパスが含まれていないこと、そしてコンピューターがそのインスタンスの証明書を信頼していることを確認してください。 |

## 既知の制限

- **フォークからの pull request：** Azure DevOps と Bitbucket では、フォークからのリクエストをチェックアウトできません（[行に対する操作](pull-requests-and-issues.md#行に対する操作)を参照）。Azure DevOps では、フォークからの pull request の作成もできません。
- **Trello** は、ボードの最初の 1000 枚までのカードしか表示しません。リストのフィルターで絞り込んでください。**Jira**、**Linear**、**YouTrack** は、更新ごとに最大 500 件の Issue を一覧します。JQL の断片または YouTrack クエリで絞り込んでください。
- **Linear** は Issue ごとに担当者 1 人です。ピッカーで複数選ぶと、最初の 1 人が送られます。その「マイルストーン」はチームの**サイクル**です（プロジェクトのマイルストーンは使いません）。**YouTrack** では `Fix versions` フィールドです。バージョンには番号がないため、ピッカーは未リリースのものをプロジェクト内の位置で番号付けします。どちらのトラッカーにも pull request はありません。
- **Redmine** は Issue ごとに担当者 1 人（ピッカーの最初の 1 人）で、ラベルはなく、更新ごとに最大 500 件の Issue を一覧します。担当者はプロジェクトのメンバー（人とグループ）で、表示名で区別されます。カンバンボードの列は Issue のステータスで、カードを移動するとステータスが変わります。Redmine の既定のワークフロールールが適用されます（その遷移が許されない役割には拒否が返り、カードは元に戻ります）。Redmine の既定データでは「Resolved」はクローズ済みのステータスではなく、クローズの列は「Closed」と「Rejected」のステータスです。
- **Bitbucket Data Center** には、新しい pull request、ブランチ、commit のブラウザー用リンクがありません。pull request のページは通常どおり開きます。
- **Azure DevOps** は作成者のアバターを表示しません。「自分」と「レビュー依頼」のフィルターには、パーソナルアクセストークンを拒否することがあるエンドポイントが必要です。その場合、プロファイルはユーザー名なしで接続され、この 2 つのフィルターは使えません。無効なトークンはエラーではなくサインインページが返されます。GitPoulpi はそれをトークンの拒否として報告します。
- **Gitea と Forgejo：**ドラフトとは、タイトルが `WIP:` で始まるリクエストのことです（作成時にサーバーにドラフトのフラグがないため）。リクエストのレビュアーとして一覧されるのは、すでにレビューした全員で、レビュー依頼の取り下げでは保留中のものだけが外れます。フォークのローカルブランチからの pull request は、GitHub と同様に `refs/pull/<n>/head` 経由でチェックアウトされます。メンションはサーバーの `mentioned_by` フィルターで検索されます。サーバーはトークンのスコープを表示しないため、不足は呼び出しが拒否されて初めてわかります。
- **Issue ボード**は、すべての Issue の取得元でカードを移動できます。Jira では、列へ至るワークフローの遷移が実行されます。入力項目（たとえば解決状況）を求める遷移は Jira に拒否され、カードはそのメッセージとともに元に戻ります。Azure DevOps では、移動先の状態を持たないワークアイテムタイプのカードが、理由とともに元に戻ります。
- **Bitbucket：** アプリパスワードはなくなりました。API トークンを使ってください。リポジトリはワークスペースごとに一覧されます。
- **Jira Cloud：** クラシックな API トークンのみ動作します（スコープ付きトークンも OAuth も不可）。新しい Jira の Issue の説明は、プレーンテキストになります。
- **会話：** 最新の 300 件が先に読み込まれます（残りは**古いものを読み込む**）。GitPoulpi から投稿したコメント、編集、削除は外部への操作で、GitPoulpi からは元に戻せません。ホストごとにできないことは[表](conversations.md#ホストごとにできないこと)にあります。Gerrit ではメッセージもコメントも編集・削除できず、Gerrit のテキストはプレーンテキストです。
- pull request、ワークアイテム、Issue、カードの作成はホスト上で行われ、GitPoulpi からは**元に戻せません**。リクエストのチェックアウトは、通常どおり元に戻せます。
