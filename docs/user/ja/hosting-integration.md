[English](../hosting-integration.md) · [Čeština](../cs/hosting-integration.md) · [Deutsch](../de/hosting-integration.md) · [Español](../es/hosting-integration.md) · [Français](../fr/hosting-integration.md) · [Italiano](../it/hosting-integration.md) · **日本語** · [한국어](../ko/hosting-integration.md) · [Nederlands](../nl/hosting-integration.md) · [Polski](../pl/hosting-integration.md) · [Português (Brasil)](../pt-BR/hosting-integration.md) · [Português (Portugal)](../pt-PT/hosting-integration.md) · [Русский](../ru/hosting-integration.md) · [Svenska](../sv/hosting-integration.md) · [Türkçe](../tr/hosting-integration.md) · [Українська](../uk/hosting-integration.md) · [简体中文](../zh-CN/hosting-integration.md)

# ホスティングと Issue トラッカーの連携

GitPoulpi は次の製品に接続できます。

- **ホスティング**（pull request、Issue、アカウントからのクローン、SSH 鍵）：GitHub、GitHub Enterprise、GitLab、GitLab Self-Managed、Azure DevOps、Azure DevOps Server、Bitbucket、Bitbucket Data Center、Gitea、Forgejo（Codeberg を含む）。**Gerrit Code Review** はレビュー専用のホスティングで、そのチェンジが pull request として扱われ、Issue はありません。
- **Issue トラッカー**（Issue のみ）：Jira Cloud、Jira Data Center、Trello、Linear、YouTrack（Cloud と Server）、Redmine。
- **ローカルの Markdown ノート**（Issue のみ、トークンもネットワークも不要）：Markdown ノートのフォルダー（たとえば Obsidian の vault 内のフォルダー）。[ローカル Issue と Obsidian](local-issues.md)を参照してください。

どの連携も**パーソナルアクセストークン**を使います。GitPoulpi のアカウントも OAuth サインインもありません。連携は任意で、**リポジトリごとにオプトイン**です。リポジトリをリンクするまで、何も取得されません。トークンは認証情報ヘルパーに保存され、`config.toml` には保存されません。通常の `git` 操作（fetch、pull、push）には連携は不要です。`git` がすでに持っている認証情報が使われます。

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

## 5. サイドバーの pull request と Issue

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

### 行に対する操作

- `Enter` またはダブルクリックで、その行をブラウザーで開きます。
- 右クリック、または `.` で、行のメニューを開きます。
  - **ブラウザーで開く**、**URL をコピー**、**番号をコピー**。
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
| `Enter` / ダブルクリック | 行をブラウザーで開く |
| `.` / 右クリック | 行のメニュー |
| `e` | 選択した pull request をホストで編集 |
| `n` | 新しい pull request（PULL REQUESTS）または新しい Issue（ISSUES） |
| `v` | 状態のドロップダウンを開く |
| `r` | 更新 |
| ヘッダーでの `Enter` / `Space` | セクションを折りたたむ／展開する |

### 再接続とオフライン

トークンがない、拒否された、またはスコープが足りないとき、セクションの最初の行に `再接続（トークン拒否）` と表示されます。クリックするか `Enter` を押すと、設定 > 連携でそのプロファイルが開きます。

ホストに接続できないときは、最後の一覧が画面に残り、ヘッダーに経過時間（`· 5m old`）が表示されます。最後の応答はディスク（`.gitpoulpi/hosting-cache.json`）にもキャッシュされるため、ネットワークが応答する前に一覧が表示されます。

## 5b. Issue ボード

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

カードには、番号、タイトル、担当者のイニシャル、ラベル、マイルストーンが表示されます。別の列にドラッグするか、`Shift+Left` / `Shift+Right`、または**ボード: カードの移動先...**（`m`）を使います。カードはすぐに移動し、約 150 ms 後にホストが処理中であることをスピナーが示します。ホストが拒否した場合はカードが元に戻り、ステータスバーが理由を伝えます。`Enter` はホスト上の Issue を開き（ローカル Issue ではノートを開き）、`e` は担当者、ラベル、マイルストーンを編集し、`n` は選択中の列に新規 Issue を作ります。`a`、`l`、`M`、`/` は担当者、ラベル、マイルストーン、テキストで絞り込み、`x` は絞り込みを解除し、`r` はホストを読み直します。サイドバーの一覧が更新されると、ボードも読み直されます。Radar にはボードはありません。

## 6. pull request を作成する

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

## 7. Issue を作成する

ISSUES セクションにプロバイダー**と**プロジェクトが設定されていると、そのヘッダーに `+`（*新規 Issue*）が表示されます。ほかには、セクション内での `n`、またはパレットコマンドの*ホスティング: 新規 Issue...*があります。

フォームには**タイトル**（必須）、**説明**（複数行）、**ラベル**（カンマ区切り、例：`bug, ui`。GitHub、GitLab、Gitea、Forgejo では、代わりにラベル、担当者、マイルストーンのボタンがピッカーを開きます）があります。Jira と Azure DevOps では**種別**のドロップダウンが追加され、Trello では**リスト**のドロップダウンが追加されます。Linear と YouTrack では**ステータス**のドロップダウン（Issue が始まるワークフローの状態、または `State` の値）が追加されます。Redmine ではプロジェクトのトラッカー（Bug、Feature、Support...）を選ぶ**種別**のドロップダウンが追加されます。Redmine にはラベルがないため、ラベルのボタンはなく、コマンドラインで指定したラベルは拒否されます。これらのドロップダウンの選択肢は、フォームを開いたときにホストから取得されます。

**Issue を作成**（または `Ctrl+S`）で、ホスト上に作成されます。`Esc` または**キャンセル**でフォームを閉じます。ホストが拒否した場合（トークンの拒否、スコープ不足、不明なプロジェクト、不明なラベル）、フォームは理由を表示し、入力した内容を保持します。成功すると、ステータスバーに新しい Issue の名前が表示され、ISSUES セクションが更新されます。

CLI：`gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`。

## 8. ホスト自身の Web ページで作成する

新規 Issue フォームと pull request 作成フォームには、右上に **<provider> で作成**ボタンがあります。キーは `Alt+W` です。ホストの Web フォームを、これまでに入力した内容を反映した状態でブラウザーに開きます。ダイアログは開いたままです。トークンは不要で、API 呼び出しも行わないため、API エラーの後でも使えます。

- GitHub には*ドラフト*用の URL パラメーターがないため、ドラフトのチェックは引き継がれません。GitLab では、タイトルに `Draft: ` のプレフィックスが付きます。
- GitLab はラベルを ID でしか受け付けないため、ラベルは説明の末尾に `/label ~"x"` のクイックアクションとして追加されます。
- 非常に長い説明は、URL が 6000 文字未満に収まるよう切り詰められ、`[...truncated: paste the rest]` という注記が付きます。
- Bitbucket Data Center には Web フォームのアドレスがないため、**<provider> で作成**ボタンはありません。

CLI：`gitpoulpi hosting issue create ... --web` と `gitpoulpi hosting pr --web` は、同じ URL を出力します。

## 9. ツールバーのチップ

ツールバーの端（既定では右端）のチップには、チェックアウト中のブランチのリクエストが表示されます：`GH #12`（GitHub）、`GL !53`（GitLab）、`AZ !12`（Azure DevOps）、`BB #12`（Bitbucket）。色は状態を表します：緑はオープン、グレーはドラフト、紫は merge 済み、赤はクローズ。

- ホバーすると、タイトル、状態、`source → target` が表示されます。
- クリックするか `Ctrl+G g` を押すと、ブラウザーでリクエストが開きます。
- 右クリックすると、**ブラウザーで開く**、**URL をコピー**、**更新**、**プロファイルを切り替え**が表示されます。

リクエストのない push 済みブランチには、薄い `+ PR` チップが表示されます。クリックすると、pull request を開始できます。一致するプロファイルがない、ブランチが push されていない、またはホストに接続できなかったときは、チップは表示されません。ウィンドウが狭いときは、ツールバーのボタンの場所を確保するため、チップは非表示になります。

## 10. 一覧が更新されるタイミング

一覧は次のタイミングで更新されます。

- リポジトリを開いたとき。
- チェックアウト中のブランチが変わったとき。
- push または fetch の後。
- *pull request と Issue の更新間隔（分）*（`general.hosting_refresh_minutes`、既定は 5、`0` でタイマーをオフ。設定 > 連携）ごと。
- 設定を閉じたとき。
- `r` を押したとき、*連携: Pull Requestと Issue を更新*を実行したとき、またはフィルターを変更したとき。

リクエストがウィンドウをブロックすることはありません。

ホストには、呼び出せる回数の制限があります。GitPoulpi は前回の応答の `ETag` を送り返すため、変更のない一覧はほとんど負荷がかかりません（GitHub では全くかかりません）。残りの呼び出し回数が少ないとホストが報告すると、ステータスバーでお知らせします。残りがなくなると、GitPoulpi は制限がリセットされるまでそのホストへの問い合わせを止め、一覧はそのままになります。

## 11. 用語

GitHub、Azure DevOps、Bitbucket は「pull request」と呼び、GitLab は「merge request」と呼びます。*レビュー依頼の用語*（`ui.pr_terminology`、設定 > 連携）で、すべてのラベル、メニュー、パレットコマンド、メッセージで使う語を選びます。

- **自動（ホストごと）**：リポジトリのリンク先プロバイダーに従います（何もリンクされていない間は Pull Request）。
- **Pull Request** または **Merge Request**：常にその語を使います。

変更はすぐに反映されます。CLI と MCP は、常に「pull/merge request」と表記します。このページでは「pull request」と表記します。

## 12. commit メッセージとブランチ名の参照

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

### Issue の参照を挿入する

commit コンポーザーでは、概要の右端にある `#▾` ボタンで、リンクされた Issue の絞り込み可能な一覧が開きます。現在のブランチ名が指す Issue が先頭に、*（ブランチから）*という印付きで表示されます。`Enter` またはクリックで、その参照（`ABC-123`、`#12`、`AB#12`）がカーソル位置に挿入されます。キーボード：コンポーザーで `Alt+R`。パレット：*Commit: Issue の参照を挿入...*。リンクされた Issue の一覧がないときは、ボタンはグレーアウトされ、*先にサイドバーで Issue の一覧をリンクしてください*と表示されます。GitPoulpi が自動で参照を追加することはありません。

## 13. SSH 鍵

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

## 13b. 履歴の書き換えのためのブランチ保護

履歴を書き換えるとき（[ワークベンチ](history-rewrite.md)）、GitPoulpi はホストに、どのブランチが保護されているかを問い合わせ、既定ではそれらを書き換えの対象外にします。問い合わせるのは、**連携した**リポジトリ（サイドバーのプルリクエストまたは Issue のセクション、または固定したプロファイル）だけで、そのプロファイルのトークンを使います。トークンが読み取れない場合は「不明」になるだけで、何も失敗しません。各プロバイダーが何を返し、トークンに何が必要かは次のとおりです。

| ホスト | 読み取るもの | トークンに必要なもの |
|---|---|---|
| GitHub（と Enterprise Server） | ブランチの `protected` フラグと、有効なブランチのルールセット（`~DEFAULT_BRANCH`、`~ALL`、`refs/heads/...` のパターン） | `repo`（公開リポジトリなら `public_repo`）を持つクラシックトークン、または Metadata（読み取り）を持つ細粒度トークン。トークンが読めないルールセットはその旨が表示され、クラシックのルールは引き続き考慮されます |
| GitLab（gitlab.com とセルフマネージド） | `protected_branches`（名前と `*` パターン） | `read_api` と、リポジトリの設定を見られるロール |
| Azure DevOps（Services と Server） | リポジトリで有効な、ブロックするブランチポリシー（`Exact`、`Prefix`、`DefaultBranch` のスコープ） | Code（読み取り）。「強制 push」の権限はセキュリティ設定であり、読み取りません |
| Bitbucket Cloud | push を止めるブランチ制限（`push`、`force`、`delete`） | `repository:admin`。ブランチングモデルへの制限は解決されません |
| Bitbucket Data Center | 強制 push を止めるブランチ権限（`fast-forward-only`、`pull-request-only`、`read-only`） | リポジトリの管理者 |

保護の設定ページ（と、既定のブランチ、リポジトリの削除の設定ページ）は、GitHub、GitLab、Azure DevOps、Bitbucket について、リポジトリが連携されていなくても、リモートの URL から組み立てられ、ホストが push を拒否したときに、3 つの対処とともに表示されます。

## 13c. Radar のアカウント全体の一覧

Radar は通常、クローンしたリポジトリを監視します。プロファイルは、自分の `[integrations.radar]` テーブルの 3 つのスイッチで、
クローンの有無を問わずホスト全体であなたに関わるものを一覧にすることもできます（すべて既定でオフ。
[設定リファレンス](config-reference.md#integrationsradarアカウント全体の一覧)を参照）。

- **レビュー依頼**：あなたのレビューを求めている未完了のリクエスト。
- **未完了のリクエスト**：自分の名前空間と、所属する各組織／グループ、または指定した名前空間の未完了のリクエストすべて。ホスト全体ではありません。
  プロファイルの**名前空間**（設定 > 連携 > Radar で 1 行に 1 項目、必要なら正規表現）は 3 つの一覧すべてを絞り込みます。
  行は、リポジトリの完全なパスと等しいか、`/` で区切られたパスのプレフィックスのときに一致します（`ptm` は `ptm/produits/x` を残し、`ptmx/y` は残しません）。
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

## 14. コマンドラインから

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
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

- **フォークからの pull request：** Azure DevOps と Bitbucket では、フォークからのリクエストをチェックアウトできません（[行に対する操作](#行に対する操作)を参照）。Azure DevOps では、フォークからの pull request の作成もできません。
- **Trello** は、ボードの最初の 1000 枚までのカードしか表示しません。リストのフィルターで絞り込んでください。**Jira**、**Linear**、**YouTrack** は、更新ごとに最大 500 件の Issue を一覧します。JQL の断片または YouTrack クエリで絞り込んでください。
- **Linear** は Issue ごとに担当者 1 人です。ピッカーで複数選ぶと、最初の 1 人が送られます。その「マイルストーン」はチームの**サイクル**です（プロジェクトのマイルストーンは使いません）。**YouTrack** では `Fix versions` フィールドです。バージョンには番号がないため、ピッカーは未リリースのものをプロジェクト内の位置で番号付けします。どちらのトラッカーにも pull request はありません。
- **Redmine** は Issue ごとに担当者 1 人（ピッカーの最初の 1 人）で、ラベルはなく、更新ごとに最大 500 件の Issue を一覧します。担当者はプロジェクトのメンバー（人とグループ）で、表示名で区別されます。カンバンボードの列は Issue のステータスで、カードを移動するとステータスが変わります。Redmine の既定のワークフロールールが適用されます（その遷移が許されない役割には拒否が返り、カードは元に戻ります）。Redmine の既定データでは「Resolved」はクローズ済みのステータスではなく、クローズの列は「Closed」と「Rejected」のステータスです。
- **Bitbucket Data Center** には、新しい pull request、ブランチ、commit のブラウザー用リンクがありません。pull request のページは通常どおり開きます。
- **Azure DevOps** は作成者のアバターを表示しません。「自分」と「レビュー依頼」のフィルターには、パーソナルアクセストークンを拒否することがあるエンドポイントが必要です。その場合、プロファイルはユーザー名なしで接続され、この 2 つのフィルターは使えません。無効なトークンはエラーではなくサインインページが返されます。GitPoulpi はそれをトークンの拒否として報告します。
- **Gitea と Forgejo：**ドラフトとは、タイトルが `WIP:` で始まるリクエストのことです（作成時にサーバーにドラフトのフラグがないため）。リクエストのレビュアーとして一覧されるのは、すでにレビューした全員で、レビュー依頼の取り下げでは保留中のものだけが外れます。フォークのローカルブランチからの pull request は、GitHub と同様に `refs/pull/<n>/head` 経由でチェックアウトされます。メンションはサーバーの `mentioned_by` フィルターで検索されます。サーバーはトークンのスコープを表示しないため、不足は呼び出しが拒否されて初めてわかります。
- **Issue ボード**は、すべての Issue の取得元でカードを移動できます。Jira では、列へ至るワークフローの遷移が実行されます。入力項目（たとえば解決状況）を求める遷移は Jira に拒否され、カードはそのメッセージとともに元に戻ります。Azure DevOps では、移動先の状態を持たないワークアイテムタイプのカードが、理由とともに元に戻ります。
- **Bitbucket：** アプリパスワードはなくなりました。API トークンを使ってください。リポジトリはワークスペースごとに一覧されます。
- **Jira Cloud：** クラシックな API トークンのみ動作します（スコープ付きトークンも OAuth も不可）。新しい Jira の Issue の説明は、プレーンテキストになります。
- pull request、ワークアイテム、Issue、カードの作成はホスト上で行われ、GitPoulpi からは**元に戻せません**。リクエストのチェックアウトは、通常どおり元に戻せます。
