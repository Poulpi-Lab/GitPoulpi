[English](../branching-workflows.md) · [Čeština](../cs/branching-workflows.md) · [Deutsch](../de/branching-workflows.md) · [Español](../es/branching-workflows.md) · [Français](../fr/branching-workflows.md) · [Italiano](../it/branching-workflows.md) · **日本語** · [한국어](../ko/branching-workflows.md) · [Nederlands](../nl/branching-workflows.md) · [Polski](../pl/branching-workflows.md) · [Português (Brasil)](../pt-BR/branching-workflows.md) · [Português (Portugal)](../pt-PT/branching-workflows.md) · [Русский](../ru/branching-workflows.md) · [Svenska](../sv/branching-workflows.md) · [Türkçe](../tr/branching-workflows.md) · [Українська](../uk/branching-workflows.md) · [简体中文](../zh-CN/branching-workflows.md)

# ブランチワークフロー

ブランチワークフローは、チームがブランチをどう使うかを GitPoulpi に伝えます：どのブランチが
永続するか（`main`、`develop`、`production`）、どのブランチが短命のトピックブランチか
（`feature/...`、`hotfix/...`）、それらがどう命名され、それぞれがどこから始まりどこで終わるか。
Git Flow をカスタマイズ可能にしたもので、固定の名前、接頭辞や接尾辞、プレースホルダー付きの
パターンが使えるため、Issue からボットが作るブランチも、自分で作ったブランチと同じように
認識されます。

このページでは、モデル、ファイル、プリセット、検出とインポート、読み取り専用のコマンド、
ブランチを開始、完了、同期、昇格するアクション（CLI、MCP、アプリ）、そしてアプリの画面
（設定のエディター、グループ化されたサイドバー、`Flow` メニューとそのダイアログ）を扱います。

## 基本概念

- **ベースブランチ**は長期的に存在するブランチです。ベースには**親**を持たせられます
  （`develop` を `main` に取り込む）。トランクには親がありません。ベースには複数の候補名
  （`main` または `master`、`develop` または `dev`）を挙げられ、存在するほうが使われます。
- **トピックの種類**は、短命のブランチの種類です（feature、bugfix、release、hotfix、
  support...）。**名前パターン**、**親**（開始位置であり、完了時の取り込み先）、完了の方法を
  持ちます。
- すべてのブランチは**分類**されます：ベース（名前で判定）、ある種類のトピック（パターンで判定）、
  または未分類のいずれかです。
- **ブランチの親**は、次の順で決まります：GitPoulpi がブランチを作成したときに記録されたもの
  （`git config branch.<name>.gitpoulpi-parent`）、なければ Git Town の
  `git-town-branch.<name>.parent`（あれば）、それもなければその種類の親。

## 保存場所

| 内容 | 場所 | commit するか |
|---|---|---|
| 共有のワークフロー | リポジトリ内の `.gitpoulpi/workflow.toml` | はい（自分で）：GitPoulpi は求められたときだけ書き込みます |
| 自分用の調整 | `.gitpoulpi/repo-state.toml` の `[workflow]` | いいえ（このファイルは無視されます） |

状態ディレクトリは自分自身を無視します（`.gitpoulpi/.gitignore`）。ただし
`workflow.toml` は commit する前提です。有効なワークフローは、共有ファイルの上に自分の上書きを
重ねたものです：同じ名前のベース、または同じ ID のトピックは置き換えられ、新しいものは追加され、
`disable_bases` と `disable_topics` は共有のものを自分に対してだけ外します。

既存のファイルに上書き保存すると、このバージョンが知らないキー（より新しい GitPoulpi のもの、
チームメイトのもの）は保たれますが、コメントとキーの順序は保たれません。

## ファイル

```toml
template = "git-flow"          # git-flow | github-flow | gitlab-flow | trunk | custom (informative)

[[base]]
name = "main"
names = ["master"]             # other spellings of the same branch
protected = true               # never deleted, direct commits are suspect

[[base]]
name = "develop"
names = ["dev"]
parent = "main"
merge = "merge"                # merge | squash | rebase | fast-forward
tag_on_merge = false
# environment = true           # an environment branch (GitLab Flow): promoted downstream

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
patterns = ["features/{slug}"] # more names that also match; never used to create
# regex = "^feat-(?P<slug>.+)$"  # also matches; named groups become fields
parent = "develop"             # where it starts, and the default finish target
finish = [{ into = "develop", merge = "squash" }]
finish_mode = "merge-request"  # local-merge | merge-request | ask
delete_after_finish = true

[hooks]                        # opt-in shell commands, always shown before they run
after_start = []
before_finish = ["cargo test"]

[issue_flow]                   # host-driven flows, see below
mode = "local"                 # local | host-creates-branch | bot
```

`merge`、`finish`、`finish_mode`、`hooks` は下の[アクション](#アクション)を、
`issue_flow` は[ホスト主導のフロー](#ホスト主導のフロー)を動かします。

### 名前パターン

`{プレースホルダー}` を含むテキストです：

| プレースホルダー | 一致するもの | 生成元 |
|---|---|---|
| `{slug}` | 自由なテキスト。最後に置く場合は `/` を含められる | テキストを小文字にし、アクセントを外し、単語の間を `-` でつないだもの（最大 50 文字） |
| `{issue}` | 数字（`123`） | `123` または `#123` |
| `{issue_key}` | トラッカーのキー（`PROJ-123`） | 指定どおり |
| `{user}` | `/` を含まない名前 | スラッグ化したもの |
| `{date}` | `2026-10-16` または `20261016` | 指定どおり |
| `{version}` | `1.2.0`、`v1.2`、`1.2.0-rc1` | 指定どおり |
| `{n}` | 数字 | 指定どおり |

それ以外はリテラルなので、接頭辞や接尾辞は単なるテキストです：
`feature/{slug}`、`{user}/{issue}-{slug}-wip`、`bot/{issue_key}`。各プレースホルダーは 1 回だけ
使え、2 つのプレースホルダーの間にはテキストが必要です。パターンは有効なブランチ名
（`git check-ref-format` の規則）を生成しなければなりません：スペースなし、`~ ^ : ? * [ \` なし、
`..` なし、`.` で始まる要素や `.lock` で終わる要素なし。

`regex` は一致の判定専用です（両端にアンカーされます）。どのパターンでも表せない名前に使います。
名前付きグループはブランチのフィールドになります。

### どのトピックが採用されるか

1. ベースと同じ名前（その名前のいずれか）のブランチは、そのベースです。
2. そうでなければ、一致するすべてのトピックのすべてのパターンが候補になります：`pattern`、
   次に `patterns`、次に `regex` の順で、トピックごとにファイルの順に見ます。固定のリテラル文字が
   最も多いものが採用されます（`release/{version}` は `{user}/{slug}` に勝ち、それは単独の
   `{slug}` に勝ちます）。正規表現はリテラルの接頭辞で数えます。同点ならファイルの中で先のものです。
3. それでもなければ、ブランチは未分類です。

壊れたパターンや正規表現は警告として報告されてスキップされ、残りは動き続けます。

## テンプレート

| テンプレート | ベース | トピック |
|---|---|---|
| `git-flow` | `main` または `master`、`develop` または `dev`（親は main） | `feature/` と `bugfix/`（develop から）、`release/`（develop から、main と develop へ、tag 付き）、`hotfix/`（main から、main と develop へ、tag 付き）、`support/`（main から）。複数形（`features/`、`releases/`...）も一致します |
| `github-flow` | `main` | `feature/` または任意の名前。main から開始し、main へのマージリクエスト、squash、その後削除 |
| `gitlab-flow` | `main`、`pre-production` と `production`（環境ブランチ。main、pre-production、production の順につながる） | `feature/` または任意の名前（main から）。`release/{version}` は cherry-pick の対象 |
| `trunk` | `main` | main から開始する短命の `{user}/{slug}`。rebase |
| `custom` | なし | なし |

環境ブランチは `preprod`、`staging`、`prod` としても一致します。テンプレートは出発点です：
`flow config init` で出力し、ファイルを編集してください。

## 検出とインポート

`gitpoulpi flow detect` は、リポジトリにあるブランチに対してテンプレートを順位づけし、100 点満点の
スコアと、見つかった具体的な名前（`main` に対する `master`、`develop` に対する `dev`、`feature/` に
対する `features/`）を表示します。ベースが 60 点、テンプレート固有のパターンが認識する
その他のブランチの割合が 40 点で、別のテンプレートの長期ブランチ（GitHub Flow の候補にある
`develop` など）は 25 点が引かれます。

`gitpoulpi flow import --from git-flow` は、リポジトリ自身の `.git/config` から従来の git-flow の
設定（`gitflow.branch.master`、`gitflow.branch.develop`、
`gitflow.prefix.feature|bugfix|release|hotfix|support|versiontag`）を読み取ります。
`--from git-town` は `git-town.main-branch` と `git-town.perennial-branches` を読み取ります
（各ブランチの親 `git-town-branch.<name>.parent` は、記録された親として取り込まれます）。
何も黙って適用されることはありません：コマンドは TOML を出力し、`--write` は
`.gitpoulpi/workflow.toml` が存在しない場合にのみそれを作成します。

## コマンド

どれも `--json` を受け付けます。MCP サーバーはこれらをツールとして公開します（`flow_status`、
`flow_templates`、`flow_detect`、`flow_config_show` は読み取り。アクションは下で説明します）。

| コマンド | 内容 |
|---|---|
| `gitpoulpi flow status` | すべてのローカルブランチとリモートブランチ：種別（ベース、トピック、未分類）、トピックの種類とフィールド、親とその出どころ（`recorded`、`git_town`、`type`、`base`）、ローカルブランチについては親に対して `ahead`（先行）と `behind`（遅れ）が何 commit か。定義で見つかった警告、順位づけされた検出結果、インポートできる他のツールの設定も含みます。 |
| `gitpoulpi flow templates` | 上のプリセットを、完全な定義つきで表示します。リポジトリは不要です。 |
| `gitpoulpi flow detect` | 順位づけされたテンプレートと、見つかったベースとトピック。 |
| `gitpoulpi flow config [--show-effective]` | 共有ファイル（そのパス、存在するか、解析されたワークフロー、未知のキー）、または `--show-effective` を付けると、ブランチの分類に使われているワークフロー。`flow config show` も同じです。 |
| `gitpoulpi flow config init --template <t> [--write]` | テンプレートの TOML を出力します。`--write` は `.gitpoulpi/workflow.toml` を作成し、既存のものは上書きしません。 |
| `gitpoulpi flow import --from git-flow\|git-town [--write]` | 上で説明したとおりです。 |

先行・遅れの数は、1 回の呼び出しにつき最大 300 のローカルブランチについて計算されます。

読み取れないワークフローファイル（不正な TOML、型の誤った値）は、ファイル名を示すエラーになります。
黙って無視されることはありません。

## アクション

`start`、`finish`、`sync`、`promote`、`set-parent` はリポジトリに対して動作します。どれもまず
**プランを立てます**：行うすべての git ステップを、それぞれに相当する git コマンド、警告、
取り消せないもの、実行を妨げるものとともに一覧にします。確認するまで何も変わりません。
各実行は**元に戻す履歴の 1 エントリ**になります（`gitpoulpi undo` は、ブランチ、tag、削除した
ブランチ、記録された親、チェックアウトをまとめて取り消します）。

| コマンド | 内容 |
|---|---|
| `gitpoulpi flow start <type> [name=value ...] [--from <ref>] [--publish]` | 種類のパターンからブランチに名前を付けます（`flow start feature "Login form"` は `feature/login-form` を作り、`flow start release version=1.2.0` は `release/1.2.0` を作ります。単語だけの場合は `slug` で、`user` と `date` の既定は自分と今日です）。種類の親または `--from`（ブランチ、リモートブランチ、tag）から作成し、チェックアウトして親を記録します。`--publish` は `-u` 付きで push します（`undo` では取り消されません）。 |
| `gitpoulpi flow finish [<branch>]` | その種類の各 `finish` 先について順に：先をチェックアウトし、先の戦略でブランチを merge します（`merge`：常にマージコミット、`squash`：`--title` またはブランチ名から付けたタイトルの 1 commit、`rebase`：ブランチを先に rebase してから早送り、`fast-forward`）。先に tag が指定されていれば作成し（`tag = "v{version}"`、注釈付き）、続いて親をチェックアウトし、種類の設定に従ってローカルブランチを削除します（`--delete` / `--keep` で上書き。`--delete-remote` はリモートのブランチも削除しますが、元に戻せません）。`finish_mode = "merge-request"`（または `--mode merge-request`）の場合は、ブランチを push して、親へのプルリクエストまたはマージリクエストを作成するだけです（`--title`、`--draft`、`--assignee`、`--reviewer`、`--label`、`--milestone`、`--integration` を指定できます。元に戻せません。ローカルでは何もマージも削除もされません）。`--mode local-merge` は `merge-request` の種類を上書きし、`ask` の種類ではどちらかの指定が必要です。 |
| `gitpoulpi flow sync [<branch>] [--stack] [--rebase\|--merge]` | 親の commit をブランチに取り込みます：merge、またはブランチの戦略が `rebase`（または `--rebase`）の場合は rebase です。`--stack` を付けると、親がこのブランチであるブランチ（`set-parent` で記録したもの）も再帰的に、順に更新します。終了後は元のブランチに戻ります。 |
| `gitpoulpi flow promote <env-branch>` | GitLab Flow の環境ブランチ：その上流のブランチ（`main` を `pre-production` に、`pre-production` を `production` に）を 1 ステップで merge して戻ります。可能なら早送りします。 |
| `gitpoulpi flow set-parent <branch> <parent>` / `--clear` | `branch.<name>.gitpoulpi-parent` を書き込みます（または削除します）。循環は拒否されます。元に戻せます。 |

別の場所から開始したブランチは、そこへ完了されます：記録された親（`--from` または
`set-parent` による）がその種類のものと異なる場合、最初の完了先は記録された親になります。

### 確認、プラン、スクリプト化

| フラグ | 意味 |
|---|---|
| `--dry-run` | プランを JSON（`{"plan": ...}`）で出力して停止します。実行を妨げるものがあれば終了コード 1。 |
| `--yes` | 確認なしで実行します。 |
| どちらもなし | ターミナルでは、プランが表示されて確認されます。それ以外（スクリプト、`--json`、エージェント）では、プランが `{"needs_confirmation": {"plan": ...}}` として返り、終了コード 3 で何も変更されません。`--publish` やフックのない `start`、`set-parent`、`--abort` は確認不要です。 |
| `--run-hooks` / `--no-hooks` | [フック](#フック)を参照。 |
| `--autostash` | チェックアウトやマージを妨げるローカルの変更を一時退避し、後で元に戻します（`general.auto_stash` の設定に関係なく）。 |

実行できないプランは理由を示します：トピックが必要な場所にベースブランチがある、名前がすでに
存在する、フィールドが足りない（`release/{version}` には `version=...` が必要）、tag がすでに
ある、先のブランチがリモートより遅れている（`--allow-behind` で続行できます）、merge や rebase が
すでに進行中、ワークフローが設定されていない。

**自動 stash** の設定（`general.auto_stash`）では、チェックアウトやマージを妨げる作業中の変更は、
通常のチェックアウトとまったく同じようにアクションの前後で stash され、元に戻されます。`never` の
場合、アクションは何も変更する前に停止します。

### 競合

競合で停止した merge や rebase は通常の競合状態（競合バナー、`gitpoulpi conflicts ...`）を残し、
フローは記憶されます。終了コード 2、`status: "conflicted"`、パスは `conflicts` に入ります。
ファイルを解決して解決済みにしてから：

- `gitpoulpi flow finish --continue` は merge を commit し（rebase なら続行、squash ならそのメッセージで
  commit）、残りのステップを実行します。
- `gitpoulpi flow finish --abort` は merge や rebase を中止し、フローを開始したブランチに戻り、
  残りを忘れます。すでに完了したステップ（二重の完了の最初の先と、その tag）は残ります：`undo` で
  取り消せます。

失敗したフックで停止したフローも同じ方法で再開します（`--continue` はそのステップをもう一度
実行します）。競合した実行とその続きは、2 つの元に戻すエントリです。

### マージリクエスト

デーモンはブランチを push します。リクエストの作成にはホスティングのトークンが必要なため、CLI が
`gitpoulpi hosting pr` と同じコードで作成します（リモートに合う連携、なければホストの入力済み
Web ページを URL として出力）。結果には `merge_request`（求めた内容）と `opened`（ホストの応答）、
またはブランチは push されたがリクエストを作成できなかった場合は `open_error` が入ります。

### フック

ワークフローファイルの `[hooks]` は、シェルコマンド（`after_start`、`before_finish`）を並べます。
チームで共有するファイル由来なので、**求めない限り実行されません**：`--run-hooks`、または一覧表示の
後にターミナルが尋ねる質問への「はい」です。プランには常にフックが一覧され、実行されない場合は
`hooks_run: false` になります。フックは、リポジトリを作業ディレクトリとし、
`GITPOULPI_FLOW_ACTION` と `GITPOULPI_FLOW_BRANCH` を設定したシェルで実行されます。その正確なコマンド、
終了コード、すべての出力が結果に含まれます。ゼロ以外で終了したフックはそこでフローを止めます
（`before_finish` のフックは何かが変わる前に実行されるので単なるエラーです。`after_start` のフックの後は
ブランチが存在し、フローを続行も中止もできます）。`--no-hooks` は確認なしでスキップします。
フックは何でもできるため、`undo` が取り消すのは git の変更だけです。

**信頼。**コミットされた `[hooks]` セクションのコマンドは、リポジトリに push できる誰かに由来するため、リストのバージョンごとに 1 回、*あなた*の信頼も必要です。信頼はフックのハッシュでリポジトリごとに保持され（コマンド、フェーズ、順序のどれかが変わると再度確認されます）、実行の最初のステップの前と、続行するフローが先に進む前に、デーモンが確認します。そのため、拒否された実行では何も変わりません。自分用のオーバーライド（リポジトリの状態の `[workflow]`、設定ページ）のフックは自分のものなので、確認されません。

- **CLI：**`gitpoulpi flow trust-hooks` はコマンド、その `hash`、`trusted` かどうかを出力します。`gitpoulpi flow trust-hooks --hash <hash>` はそれらを信頼しますが、ディスク上のファイルがそのハッシュのままの場合に限ります。信頼されていないフックに `--run-hooks` を付けると、コマンドとこのコマンドラインを含む拒否（`{"refused": {"reason": "hooks_not_trusted", ...}}`、終了コード 2）が出力されます。「一緒に実行しますか？」と尋ねられるターミナルでは、信頼するかどうかも尋ねられます。
- **TUI：**プレビューにフックが一覧され、まだ信頼されていないと表示されます。アクションボタンを押すと、「このリポジトリのワークフローは、あなたのマシン上で次のコマンドを実行します。信頼しますか？」と尋ね、正確なコマンドがスクロールできるペイン（マウスホイール、ドラッグできるスクロールバー、PgUp/PgDn/Home/End）に表示され、回答は 2 つあります：**1 信頼して実行**と **2 フックなしで実行**。矢印キーか数字キーを押すまでどの回答も強調表示されません。Esc、キャンセル、閉じる ✕ は、回答せずに戻ります。信頼した後でファイルが変わった場合、タイトルに「信頼した後でフックが変更されました」と表示され、新しいリストの下に古いリストが表示されます。
- **エージェント：**フックを信頼するツールはありません。信頼されていないフックへの `run_hooks: true` は、コマンドとともに拒否され、人が `flow trust-hooks` を実行します。

### エージェント向け（MCP）

ツールは `flow_start`、`flow_finish`、`flow_sync`、`flow_promote`、`flow_set_parent` です
（finish、sync、promote は破壊的と注釈されているため、`allow_mutations` で制限されます）。
これらは「プランしてから実行」の形です：`dry_run: true` で呼び出すとプランが（通常の結果として）
返り、次に `yes: true` で呼び出すと実行されます。どちらもなしの呼び出しは拒否され、プランが添えられるので、
うっかり実行されることはありません。フックは明示的な `run_hooks: true` でのみ実行されます。
エージェントがユーザーにコマンドを先に見せられるよう、プランにはフックが一覧されます
（`hooks`、`hooks_run`）。コミットされたフックは人が信頼する必要があります（`flow trust-hooks`。ツールはありません）：
信頼されていないフックへの `run_hooks: true` は、コマンドを添えた `refused` を返し、何も実行しません。
競合の後は `continue_: true`（フラグは `--continue`）または `abort: true` です。

### 元に戻す

| アクション | `undo` の内容 |
|---|---|
| start | ブランチを削除し、その親を忘れ、前のブランチを再びチェックアウトします。 |
| finish | 動かしたすべてのブランチ、tag、削除したブランチ、チェックアウトを、1 ステップで元に戻します。push したブランチ、リモートでの削除、作成したマージリクエストは残ります（プランにそう書かれています）。 |
| sync | 更新したすべてのブランチを元に戻します（チェックアウト中のブランチはファイルも）。 |
| promote | 環境ブランチを元に戻します。 |
| set-parent | 以前に記録されていた親を復元します。 |

## ホスト主導のフロー

Issue から作業が始まるチーム向けです：Issue はテンプレートから書かれ、ホスト（GitLab、GitHub、
Gitea、Forgejo）またはボットがブランチとマージリクエストを作ります。GitPoulpi は Issue を作成し、
ホストが作るものを待ち、それを引き継ぎます：リモートのブランチを追跡するローカルブランチ、記録された
親（マージリクエストのターゲット）、チェックアウト。ISSUES セクションがトラッカーにリンクされている
必要があります（`gitpoulpi hosting link issues`）。ブランチとリクエストがどこにあるかは PULL REQUESTS の
リンクが示します（リンクされていなければ ISSUES のプロジェクト）。

```toml
[issue_flow]
mode = "bot"                         # local | host-creates-branch | bot
issue_title = "[{type}] {title}"     # {type} {title} {user} {date}
issue_labels = ["workflow::{type}"]  # a bot often triggers on a label
description_template = ".github/ISSUE_TEMPLATE/feature.md"   # a file of the repository
branch_from_issue = "{type}/{issue}-{slug}"   # what the host or bot names the branch
wait_for = "branch-or-mr"            # branch | mr | branch-or-mr
timeout_secs = 600                   # how long `wait` polls
merge_request = true                 # host-creates-branch: also a draft request
assignees = []
```

`branch_from_issue` の `{type}` はトピックの種類の ID で、そのほかは通常の
[名前パターン](#名前パターン)のトークンです（`{issue}`、`{slug}`、`{issue_key}`...）。ブランチは、
そのパターンに一致し、`{issue}` が 12 と等しいとき、Issue 12 のものになります
（`feature/12-login` は該当し、`feature/120-x` は該当しません）。

| モード | Issue の作成後 |
|---|---|
| `local` | ホストでは何もしません。`--pick-up` を付けると、その種類の `flow start` を Issue の番号とタイトルで実行します（通常の start と同じく元に戻せます）。 |
| `host-creates-branch` | GitPoulpi が種類の親から**ホスト上で**ブランチを作成し（GitLab：branches API、GitHub：`createLinkedBranch` なので Issue の Development に表示される、Gitea / Forgejo：branches API）、`Closes #12` と書いた**ドラフトの**リクエストを作成します。空のリクエストを拒否するホスト（GitHub：まだ commit がない）では、ブランチは残り、その旨が警告として表示されます。 |
| `bot` | 何もしません：ボット（webhook ハンドラー、CI ジョブ、GitHub App）がブランチとリクエストを作ります。`--wait` は `wait_for` が満たされるまでポーリングします。 |

`wait` が見るものは、どのプロバイダーでも次のとおりです：ホスト自身のリンク（GitLab の
`issues/:iid/related_merge_requests` と `closed_by`、GitHub のリンクされたブランチとタイムラインの
`connected` / `cross-referenced` のプルリクエスト）、名前が `branch_from_issue` に一致するブランチ、
元のブランチ名が一致する open のリクエスト。間隔を広げながらポーリングし（1、2、3、5、8 秒、その後は 10 秒ごと）、
`wait_for` が満たされるか `timeout_secs` が尽きるまで続けます。応答の後で遅い、または一時的に
到達できないホストは再試行されます。ホストが提供しない取得元（Gitea には Issue のリンクがない）は
スキップされ、すべての取得元が失敗した場合は最初のエラーが答えになります。

| コマンド | 内容 |
|---|---|
| `gitpoulpi flow issue new --type <t> --title "..." [--body T \| --body-file F] [--label L] [--assignee A] [--milestone M] [--wait] [--pick-up] [--timeout S] [--integration ID] [--dry-run]` | テンプレート（`issue_title`、`issue_labels`、説明ファイル、それに代わる `--body`）から Issue を作成し、モードに従って進めます。`--dry-run` は作成される内容を出力し（`{"plan": ...}`）、何も作成しません。`--pick-up` は `bot` モードでは先に待機します。終了コード 4：待機が時間切れ（Issue は存在します。`flow issue wait` で待機を続けられます）。 |
| `gitpoulpi flow issue wait <#12> [--type t] [--for branch\|mr\|branch-or-mr] [--timeout S] [--once] [--pick-up]` | Issue のためにホストが作ったものをポーリングします。ターミナルでは進捗を stderr に表示し、Ctrl+C で中止します。`--once` は 1 回だけ確認します。終了コード 4：まだ何もありません。 |
| `gitpoulpi flow pick-up <#12 \| !5 \| branch> [--remote R] [--parent B] [--no-fetch] [--dry-run]` | fetch し、`origin/<branch>` を追跡するローカルブランチを作成し、その親を記録してチェックアウトします。`#12` は Issue のブランチまたはリクエストをホストで検索し、`!5` はリクエスト、それ以外はブランチ名です。親はリクエストのターゲット（`--parent` で上書き）、なければ種類の親です。`start` と同じくローカルで元に戻せるため、`--yes` は不要です。`--dry-run` でも fetch は行われます（リモート追跡参照を動かすだけで、プランにはブランチを見る必要があるため）。 |

`pick-up` は元に戻す履歴の 1 エントリです：`gitpoulpi undo` はブランチを削除し、親を忘れ、前のブランチを
再びチェックアウトします（fetch は、どの fetch とも同じく残ります）。ほかのアクションと同じくプランが
立てられます：`--dry-run` はそのプラン（`create_branch`、`set_upstream`、`checkout`、`record_parent`）を出力し、
通常の実行を妨げるものも適用されます（作業中の変更は自動 stash の設定に従います）。リクエストは
`{"action": "pick_up", "branch": ..., "parent": ...}` です。

フローが作った Issue は**監視**されます（`.gitpoulpi/repo-state.toml` の `watched_issues`。commit
されません）：fetch のたびに、アプリはリモートブランチを `branch_from_issue` と照合し、監視中の
Issue のブランチが現れると知らせます。引き継ぎで解消されます。

### エージェント向け（ホスト主導）

ツールは `flow_issue_new`、`flow_issue_wait`、`flow_pick_up` です（書き込み、ネットワーク。
`allow_mutations` で制限されます）。`dry_run: true` の `flow_issue_new` はプランを返します。
`flow_issue_wait` は最大 `timeout` 秒ブロックするので、エージェントには短い値を与えて再度呼び出して
ください（`once: true` ならすぐに答えます）。どちらも `found`（`branches`、`requests`）と `timed_out` を
含む JSON を返します。

### アプリでのホスト主導のフロー

ワークフローとリンクされたトラッカーがあると、Flow メニューに*新しい {種類} Issue...*（種類ごとに 1 つ。
パレットは `Flow: 新しい {種類} Issue...` と `Flow: 新しい Issue...`）と
*ホストが作成したブランチを引き継ぐ...*（パレットは `Flow: ホストのブランチを引き継ぐ...`）が加わります。

- **新しい Issue** は種類（ドロップダウン）とタイトルを尋ね、説明の出どころと、ブランチを作る主体を
  表示します。*ブランチが現れたら引き継ぐ*は既定でオンです。*Issue を作成*で作成し、`bot` モードでは
  **待機カード**が表示されます：スピナー、待っている内容、経過時間と諦めるまでの時間、そして
  **待機を中止**（`Esc`）。中止しても Issue は失われません：監視されたままです。ブランチが現れると
  引き継がれ、ステータスバーがそのことを伝えます。
- **ブランチが現れた**は、fetch で監視中の Issue のブランチが届いた後に（ほかのダイアログが開いて
  いないときに）開くカードです：*引き継ぐ*で引き継ぎ、*後で*で閉じます（アプリを再起動するまで
  再び表示されません）。
- **引き継ぎ**は `#12`、`!5`、またはブランチ名を受け取り、引き継ぐ前に見つかったものを表示します。

すべてバックグラウンドで実行され、150 ms を超える呼び出しではスピナーが表示されます。

### テストベンチで試す

`scripts/fake-gitlab-bot.sh` は、Docker の GitLab（`scripts/test-bench.sh`）用の代役のボットです：
ラベル付きの Issue があるプロジェクトを監視し、`<type>/<iid>-<slug>`（commit 1 つ付き）と、
その Issue をクローズするドラフトのリクエストを作ります。
`cargo run -p xtask -- mock-host --flow-bare <bare repo>` は、同じものをオフラインで提供します
（実際のベアリポジトリの上の GitLab 形式の API と、ボット）。

## アプリ内

![Git Flow ワークフローのサイドバー：main と develop がベースブランチとして示され、feature ブランチは Feature フォルダーに、残りは「その他」の下に並ぶ](../images/workflow-sidebar.png)

![Start Feature：種類、スラッグ、開始位置、作成されるブランチ名](../images/workflow-start.png)

![設定 > リポジトリ別 > ワークフロー：テンプレート、検出、ベースブランチのエディター](../images/workflow-settings.png)

以下はすべて、ワークフローがあるリポジトリ（`.gitpoulpi/workflow.toml`、または自分の上書き）でのみ
表示されます。ワークフローがなければ何も変わりません：サイドバーは通常のツリーのままで、
`Flow` ボタンもありません。

### 設定 > ワークフロー

*設定 > リポジトリ別の設定 > ワークフロー*で定義を編集します。下書きに対して作業し、保存ボタンを
押すまで何も保存しません。

- **テンプレート**はプリセットのドロップダウンです。選ぶと、下のベースとトピックの種類がプリセットで
  置き換えられます（フックと Issue フローは残ります）。
- **ブランチから検出**は、このリポジトリのブランチ名を読み取り、最も近いテンプレートとそのスコア、
  見つかったベース名（`main` に対する `master`）、各トピックの種類が認識したブランチの数と例、
  すべてのテンプレートのスコアを表示します。**検出したワークフローを使用**で、それを下書きに
  読み込みます。
- リポジトリに別のツールの設定（Git Flow の `gitflow.*`、Git Town の `git-town.*`）があると、
  **インポート**が表示され、インポートした定義が下書きに読み込まれます。押すまで何も適用されません。
- **ベースブランチ**、**トピックの種類**、それぞれの**完了ルール**は、グループに分かれた 1 つの
  [リスト](settings.md#リスト)にまとまり、**ルール**タブに入っています。**一般**タブには残り（テンプレート、検出、フック、Issue フロー）が
  あります。項目を選ぶと、右側で編集できます（名前と別名、親、保護、merge 戦略、merge 時の tag、
  環境ブランチ。ラベル、名前パターン、ほかのパターン、正規表現、親、完了モード、完了後に削除）。
  **+ 追加 ▾** で追加する対象を選びます（完了ルールは、選択中のトピックの種類に属します）。追加、複製、
  削除、ドラッグは、ほかの設定のリストと同じように使え、項目はグループ内で移動します。
  **名前パターン**の行にはトークン
  （`{slug} {issue} {issue_key} {user} {date} {version} {n}`）が一覧され、その下のライブプレビューが、
  パターンに一致する既存のブランチと、作成される名前を表示します。解析できないパターンは、
  フィールドのすぐ下に赤で理由が表示されます（入力したものは失われません）。
- **フック**はシェルコマンドで、1 行に 1 つ（`開始後`、`完了前`）、複数行のエディターで編集します
  （`Ctrl+S` で保存、`Esc` でキャンセル）。お使いのマシンでコマンドを実行するものなので、ページにその旨が
  表示され、ダイアログでも実行前に表示されます。
- **Issue フロー**には、[ホスト主導のフロー](#ホスト主導のフロー)が使うフィールド（モード、Issue のタイトルと
  ラベル、ブランチのパターン、待機対象）があり、ワークフローファイルに保存されます。
- **問題**には、定義の誤り（存在しない親、壊れたパターン）が一覧されます。
- ページの下部に固定されたバーには、**● 未保存**（または**変更なし**）と表示されます。
  **保存先：.gitpoulpi/workflow.toml（共有）**は、下書きを共有ファイルに書き込みます（commit して
  ください）。このバージョンが知らないキーは保たれ、自分の上書きは取り込まれます。
  **自分だけに保存**は、共有ファイルとの差分だけを自分のリポジトリ状態
  （`.gitpoulpi/repo-state.toml`。commit されません）に保持します。**変更を破棄**は、保存済みの内容を
  読み直します。未保存の下書きがある状態で設定を閉じようとすると、先に確認されます。

### サイドバー

LOCAL と REMOTE はワークフローでグループ化されます：まずベースブランチが、それぞれ `[ベース]` の
バッジ付きで（保護されていれば `[ベース、保護]`）、次にトピックの種類ごとに、ラベルの名前とブランチ数
（`Feature (3)`）を付けたフォルダーが 1 つずつ、種類に属さないブランチは「その他」の下に並びます。
フォルダーは通常のフォルダーと同じように折りたためます。通常のツリーに戻すには、
*Flow > ブランチを通常のリストで表示*（Flow メニューの行）、またはパレットコマンド
`Flow: ブランチのグループ表示を切り替え` を使います。ブランチにポインターを置くと、その種類、親、
親に対してどれだけ遅れて（先行して）いるかが表示されます。

### Flow メニューとそのダイアログ

ツールバーの **Flow** ボタン（`Ctrl+G Shift+W`、パレットは `Flow: メニューを開く`）には、トピックの種類ごとの
*{種類} を開始...*、*現在のブランチを完了...*、*親と同期...*、*昇格...*（GitLab Flow の環境ブランチ）、
*親を設定...*、グループ化の切り替え、*ワークフロー設定...* が並びます。パレットにも同じコマンドが
あります（`Flow: Feature を開始...` は種類ごとに生成されます）。

- **開始**は、種類の名前パターンから作られるフォームです：トークンごとに 1 つのフィールドがあります。
  `{issue}` と `{issue_key}` にはリンクされたトラッカーの Issue のドロップダウンが付き、Issue の
  タイトルから `{slug}` も入力されます。`{version}` は最新のリリース tag の次のバージョンを提案し、
  `{user}` は有効なプロファイル、`{date}` は今日です。パターンから作られるブランチ名が 1 行で表示されます
  （作れない場合はその理由：無効な参照、すでに使われている名前）。開始元はドロップダウン（種類の親が先頭）で、
  *ブランチを公開*は新しいブランチを push します。
- **完了**、**同期**、**昇格**、**親を設定**は、必要なものを尋ねてから、デーモンが作る**プレビュー**
  （`gitpoulpi flow ... --dry-run` が出力するのと同じプラン）を表示します：すべてのステップを順に、
  警告（リモートより遅れている先、このブランチの上に積まれたブランチ）、そして**ブロッカー**
  （存在しない先、リモートより遅れている先）。ブロッカーは実行ボタンを無効にし、理由を示します。
  取り消せないステップ（push、リモートブランチの削除、マージリクエスト）には `!` と色が付き、
  「元に戻せない内容：...」という行が表示されます。実行されるフックは警告の下に一覧され、
  *フックを実行*のチェックボックス（既定はオフ）が付きます。種類の完了モードが `ask` の場合、完了は
  まず*完了方法*を尋ねます：ローカルで merge するか、プルリクエストまたはマージリクエストを作成するか。
- その後アクションはステップごとの印とともに実行され（150 ms 後にスピナー）、ダイアログは各ステップの
  結果を表示します。アクション全体が**元に戻す 1 エントリ**です：`Ctrl+Z` で start、finish、sync、promote が
  まとめて取り消されます（元に戻す履歴には `Flow: finish feature/login` のように表示されます）。push、
  リモートでの削除、マージリクエストは、プレビューで示したとおり残ります。
- **競合**はアクションを停止し、**続行**と**中止**が表示されます。ダイアログを閉じて通常の競合ツールで
  競合を解決し、パレットコマンド `Flow: 停止した操作を続行または中止...`（アクションが停止している間は
  Flow メニューの行でもあります）で開き直せます。続行は、競合しているファイルが残っている間は拒否され、
  2 つのボタンはそのままです。中止はアクションを開始したブランチに戻ります。すでに終えたステップは
  残ります（undo で取り消せます）。
- **マージリクエスト**による完了は、ブランチを push してから、通常の*プルリクエストを作成*フォーム
  （GitLab では*マージリクエストを作成*）を入力済みで開きます：ターゲットはブランチの親で、タイトルは
  ブランチ（と Issue）から作られます。メタデータのボタン（担当者、レビュアー、ラベル、マイルストーン）は
  通常のプルリクエストと同じように使えます。リモートに応答する連携がない場合は、代わりに比較 URL が
  ブラウザーで開かれます（ステータスバーにも表示されます）。
- ステップとブロッカーのテキストは CLI と同じく英語で、ダイアログ自身のテキストは翻訳されています。
  変更のある作業ツリーは `auto-stash` の設定に従います。

これらはすべてキーボードとパレットから操作でき、すべてのドロップダウンはコンボボックスです
（Enter で開き、入力で絞り込み、Enter で選択）。

ホスト主導の行（*新しい {種類} Issue...*、*ホストが作成したブランチを引き継ぐ...*）は
[前の節](#アプリでのホスト主導のフロー)で説明しています。
