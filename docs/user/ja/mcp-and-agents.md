[English](../mcp-and-agents.md) · [Čeština](../cs/mcp-and-agents.md) · [Deutsch](../de/mcp-and-agents.md) · [Español](../es/mcp-and-agents.md) · [Français](../fr/mcp-and-agents.md) · [Italiano](../it/mcp-and-agents.md) · **日本語** · [한국어](../ko/mcp-and-agents.md) · [Nederlands](../nl/mcp-and-agents.md) · [Polski](../pl/mcp-and-agents.md) · [Português (Brasil)](../pt-BR/mcp-and-agents.md) · [Português (Portugal)](../pt-PT/mcp-and-agents.md) · [Русский](../ru/mcp-and-agents.md) · [Svenska](../sv/mcp-and-agents.md) · [Türkçe](../tr/mcp-and-agents.md) · [Українська](../uk/mcp-and-agents.md) · [简体中文](../zh-CN/mcp-and-agents.md)

# MCP とエージェント

AI エージェント（Claude Code、Cursor、Codex など）は、**MCP サーバー**または **CLI** を通じて GitPoulpi を操作できます。どちらもウィンドウと同じバックグラウンドデーモンを経由するため、エージェントの操作は、開いている GitPoulpi にリアルタイムで反映されます。また、どちらも各コマンドについてまったく同じコードを実行します。

このページは*外部の*エージェントについて説明します。GitPoulpi 自身の AI ボタン（commit メッセージ、説明、commit コンポーザー）は別に設定します。[AI アシスタント](ai.md)を参照してください。

## クイックセットアップ

1. `gitpoulpi` と `gitpoulpi-mcp` を `PATH` に置きます（[インストール](installation.md)）。
2. エージェントにサーバーを登録します。標準入出力（stdio）で通信するため、起動するものも、ポートもありません。
3. 必要に応じて、`config.toml` の `[mcp]` テーブル（後述）で、エージェントに許可することを選びます。既定では、エージェントはリポジトリを読み取れ（`allow_mutations = true` を設定するまで変更は拒否されます）、ユーザーへの表示は確認後にのみ行われ、ダイアログが入力済みになることはありません。

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project` を付けると、代わりにリポジトリの `.mcp.json` に書き込まれ、プロジェクトを開いた全員に適用されます。同じファイルを、任意の追加設定付きで手書きする場合：

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "command": "gitpoulpi",
      "args": ["mcp", "serve", "--stdio", "--ui-control", "auto"],
      "env": { "GITPOULPI_CONFIG_DIR": "/home/me/.config/gitpoulpi-agent" }
    }
  }
}
```

- `--ui-control auto|ask|off` は、このエージェントに限って `[mcp] ui_control` を上書きします。
- `GITPOULPI_CONFIG_DIR` は、エージェント専用の `config.toml` を与えます（専用の `[mcp]` の制限があり、ユーザーの設定にはアクセスできません）。省略すると、ユーザーの設定を共有します。
- `gitpoulpi --repo <path> mcp serve --stdio` は、サーバーを 1 つのリポジトリに固定します。そうしない場合、ツールはサーバーの作業ディレクトリを含むリポジトリ、または呼び出しの `repo` 引数に対して動作します。

**その他のクライアント**：stdio サーバーを起動できる MCP クライアントなら何でも使えます。

| クライアント | 場所 | 内容 |
|---|---|---|
| Cursor、Windsurf、Claude Desktop、VS Code | クライアントの MCP JSON ファイル | 上記の `mcpServers` ブロック（VS Code ではキー名が `servers`） |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`、`command = "gitpoulpi"`、`args = ["mcp", "serve", "--stdio"]` |
| その他 | そのドキュメント | コマンド `gitpoulpi-mcp`（単体のバイナリ。同じサーバーで、引数は不要） |

**HTTP 経由**（URL に接続するクライアント向け）：

```
gitpoulpi-mcp --http                         # [mcp] bind, default http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

ステートレスな JSON で、ツールも `[mcp]` の制限も同じです。**認証はない**ため、ループバックでしか待ち受けません。`--allow-remote` でこれを解除できますが、その場合はポートに到達できる人なら誰でも、設定が許可するすべてのツールを実行できます。信頼できるネットワークの内側、または認証付きプロキシの背後でのみ使ってください。

**MCP を使わない場合**、シェルを使えるエージェントは CLI を使います：`gitpoulpi --json <command>`（[CLI リファレンス](cli-reference.md)を参照）。素の `git` の代わりに、いつ・どのように GitPoulpi を使うかを記した既製のスキル [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md) を渡してください。

## 権限

| スイッチ | 既定値 | 効果 |
|---|---|---|
| `[mcp] enabled` | `true` | `false`：`gitpoulpi mcp serve` / `gitpoulpi-mcp` は起動を拒否します（設定 > エージェントと MCP > *MCP サーバー*）。サーバーの起動時に読み込まれ、起動中のサーバーは設定をそのまま保ちます。 |
| `[mcp] bind` | `"127.0.0.1:8808"` | アドレスが指定されないときに `--http` が使うアドレス（`--allow-remote` がなければループバックのみ）。設定 > エージェントと MCP > *MCP サーバーのポート*でポートを編集でき、ホストは維持されます。 |
| `[mcp] allow_mutations` | `false` | オフ：何かを変更するツールはすべて拒否されます。読み取り、`ui_*`、提案は引き続き動作します。`true` にすると、エージェントは commit、push、リセットなどができます。 |
| `[mcp] mutation_allowlist` | `[]` | 空でない場合：ここに挙げたツールだけが変更を行えます。例：`["stage", "unstage", "commit"]`。 |
| `[mcp] allow_ui` | `true` | `false`：エージェントはユーザーのウィンドウを動かせません（`ui_state` / `ui_viewers` は引き続き応答します）。 |
| `[mcp] ui_control` | `"ask"` | `ask`：ウィンドウにユーザーが確認するチップが出ます。`auto`：ユーザーが操作中でなければ適用されます。`off`：拒否されます。エージェントごと：`--ui-control`。 |
| `[mcp] allow_ui_propose` | `false` | エージェントが、ユーザーが確認する入力済みのダイアログを開けるようにします（設定 > エージェントと MCP > *AI エージェントの変更提案を確認できるようにする*）。 |
| `[mcp] allow_history_purge` | `false` | `config.toml` のみ。`true` にすると、履歴の書き換えの古い履歴（とその undo）を完全に削除するツール `rewrite_purge` が一覧に表示されます。オフ：エージェントにはこのツールが見えません。使うには `allow_mutations` も必要です。 |
| `[ui] allow_agent_view_control` | `true` | ウィンドウ側のマスタースイッチです（設定 > エージェントと MCP > *AI エージェントによる表示の操作を許可*）。オフ：どのエージェントもユーザーのウィンドウに到達できません。 |
| `[ui] launch` | `"off"` | ウィンドウが開いていないときに、エージェントが GitPoulpi のウィンドウを開いてよいか（[後述](#ウィンドウが開いていない場合または別のセッションにある場合)）。 |

実際には次のようになります。

- **初期状態では、エージェントは読み取り専用です。** 読み取り、ウィンドウへの表示（`ui_*`）、そして許可した場合はユーザーが確認する変更の提案ができます。リポジトリを変更させる（commit、push、リセットなど）には `allow_mutations = true` にします。範囲を絞って許可するには、`true` のまま `mutation_allowlist` を設定します。すべてのツールには MCP のアノテーション（`readOnlyHint`、`destructiveHint`、`openWorldHint`）が付いているため、Claude Code のようなクライアントは、危険なものについては事前にユーザーに確認します。
- **競合：** `conflicts_check`（merge の前に行う予測）、`conflicts_list`、`conflicts_show`、`conflicts_state` は読み取りです。`conflicts_resolve`（残さなかった側を破棄します。`--delete` はファイルを削除します）と `conflicts_abort`（merge／rebase／cherry-pick 全体を破棄します）は破壊的です。`conflicts_unresolve` は通常の書き込みです。拒否された呼び出し（`stale`、`not_conflicted`、`pick_incomplete` など）は、失敗する代わりに `{"refused": {...}}` と終了コード 2 で応答します。
- **行き詰まった状態：** `conflicts_am_continue` は書き込み、`conflicts_am_skip`、`conflicts_am_abort`、`index_lock_remove` は破壊的、`index_lock_show` は読み取りです。
- **履歴の書き換え：** `rewrite_tools`、`rewrite_preview`、`rewrite_rows`、`rewrite_script_check`、`rewrite_script_test`、`rewrite_discard`、`rewrite_analyze`（サイズ分析）、`rewrite_scan_secrets`（マスクしたプレビュー付きの検出結果で、値は決して含みません）、`rewrite_note` は読み取りです（プレビューは何も変更せず、計画をデーモンと git ディレクトリのディスクに保持するだけなので、`rewrite_apply` は再起動後でも動作します）。`rewrite_apply` と `rewrite_run` は破壊的です。書き換えた commit に向けて多数の参照を、元に戻す操作 1 回分として動かし（`undo` で元に戻ります）、push はしません。レシピは `recipe_json`（またはパスを渡す `recipe`）で、シークレットは `secrets_file` で渡し、インラインでは渡さないでください。拒否された場合は `{"refused": {"reason", "message", "detail"}}` で応答し、push 済みの参照を動かす計画は、呼び出しで `yes` を設定するまで `{"needs_confirmation": ...}`（終了コード 3）で応答します。先にユーザーに見せてください。シェル、Python、外部 Lua のスクリプト（`script_language`）はサンドボックスの外で動作し、**エージェントは信頼できません**。`rewrite_script_trust` ツールも `trust` 引数もないため、そのようなスクリプトが実行されるのは、人がそのテキストそのものに対して `gitpoulpi rewrite script trust FILE` を実行した後だけです（変更されたスクリプトは再び拒否され、`script_not_trusted` とハッシュが `refused.script` に入ります）。さらに `rewrite.external_scripts` でその言語がオンになっている必要があります（`scripts_disabled`）。`rewrite_maps` は書き込み、`rewrite_push` は破壊的でネットワークを使います（リース付きの強制 push で、元に戻せません）。`rewrite_purge` は破壊的で、`[mcp] allow_history_purge = true` のときだけ一覧に表示されます。
- **エージェントからは決して到達できないもの：** トークンの保存（`hosting login`）、SSH 鍵の作成とアップロード、`config set`（自分自身の制限を書き換えられてしまうため）、デーモンのライフサイクル。トークンがエージェントに表示されることはありません。
- **アカウント全体の一覧：** `hosting_radar` は読み取りです（プロファイルのすべてのリポジトリにまたがる、自分のレビュー待ちのリクエスト、
  未完了のリクエスト、割り当てられた Issue。`refresh` を指定しない限り 5 分間のキャッシュから返します）。`hosting_integrations_radar` は
  プロファイルの `[integrations.radar]` テーブルを変更するため、`allow_mutations` の制限を受けます（オプションなしで呼び出して
  設定を出力するだけの場合でも同様です）。
- **レビューのコンテキスト：** `ai_review_context` は、コードレビューのドラフトに差分以外に何が渡されるか（CLAUDE.md、AGENTS.md、追加ファイル、
  ローカルエージェントの実行場所）を表示します。設定の引数を付けると既定値（全体、または `repo` や `project` 固有）を保存するため、設定を変更する
  ほかのツールと同様に `allow_mutations` の制限を受けます。引数がなければ出力するだけです。すべての `ai_*` ツールと同様に、`ai.enabled` が
  オンの間だけ一覧に表示されます。`hosting_review_suggest` は同じ引数を受け取りますが、その実行にのみ有効です。
- **`[mcp]` の制限が適用されるのは MCP サーバーです。** CLI はユーザー自身のシェルであり、`allow_mutations` の制限を受けません。`gitpoulpi ui ...` にも適用されるのは、`ui_control` と `allow_ui_propose` だけです。
- 拒否された呼び出しは通常のツールエラー（`isError: true`、理由は `structuredContent.error`）であり、黙って何もしないことはありません。

```toml
[mcp]
allow_mutations = false        # true: the agent may change the repository
mutation_allowlist = []        # e.g. ["stage", "unstage", "commit", "branch_create"]
allow_ui = true
ui_control = "ask"             # "auto" lets the agent move your view without asking
allow_ui_propose = false       # true: the agent may pre-fill Reword / Split / commit / branch / tag dialogs
allow_history_purge = false    # true: the agent sees rewrite_purge (deletes old history for good)
```

## ツール

CLI コマンド 1 つにつき 1 つのツールがあり、名前はコマンドのパスを `_` でつないだものです（`branch create` は `branch_create`）。各ツールは、コマンドの引数に加えて `repo`（リポジトリ。既定はサーバーのもの）と `session` を受け取ります。呼び出しが成功したときの `structuredContent` は、`gitpoulpi --json <command>` が出力する内容そのものです。**Kind**（「Kind」列。サーバーと同じく英語のままです）：*read* は何も変更しません。*write* と *destructive* は変更を伴うもの（制限の対象）です。*network* はリモートまたはホストと通信します。*view* はユーザーのウィンドウを動かすだけです。*propose* はユーザーが確認するダイアログを開きます。

<!-- mcp-tools:begin -->
| Tool | CLI | Kind |
|---|---|---|
| `ai_commit_message` | `gitpoulpi ai commit-message` | read, network |
| `ai_explain` | `gitpoulpi ai explain` | read, network |
| `ai_prompt` | `gitpoulpi ai prompt` | read |
| `ai_resolve_conflict` | `gitpoulpi ai resolve-conflict` | read, network |
| `ai_review_context` | `gitpoulpi ai review-context` | write |
| `ai_rewrite_rules` | `gitpoulpi ai rewrite-rules` | read, network |
| `ai_rewrite_script` | `gitpoulpi ai rewrite-script` | read, network |
| `ai_split_plan` | `gitpoulpi ai split-plan` | read, network |
| `alias_status` | `gitpoulpi alias status` | read |
| `amend` | `gitpoulpi amend` | destructive |
| `autostash_start_over` | `gitpoulpi autostash start-over` | destructive |
| `blame` | `gitpoulpi blame` | read |
| `branch_create` | `gitpoulpi branch create` | write |
| `branch_delete` | `gitpoulpi branch delete` | destructive |
| `branch_delete_remote` | `gitpoulpi branch delete-remote` | destructive, network |
| `branch_fast_forward` | `gitpoulpi branch fast-forward` | write |
| `branch_list` | `gitpoulpi branch list` | read |
| `branch_rename` | `gitpoulpi branch rename` | write |
| `branch_set_upstream` | `gitpoulpi branch set-upstream` | write |
| `branch_track` | `gitpoulpi branch track` | write |
| `branch_unset_upstream` | `gitpoulpi branch unset-upstream` | write |
| `checkout` | `gitpoulpi checkout` | write |
| `cherry_pick` | `gitpoulpi cherry-pick` | write |
| `clone` | `gitpoulpi clone` | write, network |
| `commit` | `gitpoulpi commit` | write |
| `config_get` | `gitpoulpi config get` | read |
| `config_path` | `gitpoulpi config path` | read |
| `conflicts_abort` | `gitpoulpi conflicts abort` | destructive |
| `conflicts_am_abort` | `gitpoulpi conflicts am-abort` | destructive |
| `conflicts_am_continue` | `gitpoulpi conflicts am-continue` | write |
| `conflicts_am_skip` | `gitpoulpi conflicts am-skip` | destructive |
| `conflicts_check` | `gitpoulpi conflicts check` | read |
| `conflicts_list` | `gitpoulpi conflicts list` | read |
| `conflicts_resolve` | `gitpoulpi conflicts resolve` | destructive |
| `conflicts_show` | `gitpoulpi conflicts show` | read |
| `conflicts_skip` | `gitpoulpi conflicts skip` | destructive |
| `conflicts_state` | `gitpoulpi conflicts state` | read |
| `conflicts_unresolve` | `gitpoulpi conflicts unresolve` | write |
| `convert` | `gitpoulpi convert` | write |
| `diff` | `gitpoulpi diff` | read |
| `discard` | `gitpoulpi discard` | destructive |
| `fetch` | `gitpoulpi fetch` | write, network |
| `file_history` | `gitpoulpi file-history` | read |
| `flow_config_init` | `gitpoulpi flow config init` | write |
| `flow_config_show` | `gitpoulpi flow config show` | read |
| `flow_detect` | `gitpoulpi flow detect` | read |
| `flow_finish` | `gitpoulpi flow finish` | destructive, network |
| `flow_import` | `gitpoulpi flow import` | write |
| `flow_issue_new` | `gitpoulpi flow issue new` | write, network |
| `flow_issue_wait` | `gitpoulpi flow issue wait` | write, network |
| `flow_pick_up` | `gitpoulpi flow pick-up` | write, network |
| `flow_promote` | `gitpoulpi flow promote` | destructive |
| `flow_set_parent` | `gitpoulpi flow set-parent` | write |
| `flow_start` | `gitpoulpi flow start` | write, network |
| `flow_status` | `gitpoulpi flow status` | read |
| `flow_sync` | `gitpoulpi flow sync` | destructive |
| `flow_templates` | `gitpoulpi flow templates` | read |
| `hosting_board_move` | `gitpoulpi hosting board move` | write, network |
| `hosting_board_show` | `gitpoulpi hosting board show` | read, network |
| `hosting_checkout_pr` | `gitpoulpi hosting checkout-pr` | write, network |
| `hosting_clone` | `gitpoulpi hosting clone` | write, network |
| `hosting_edit_url` | `gitpoulpi hosting edit-url` | read, network |
| `hosting_file` | `gitpoulpi hosting file` | read, network |
| `hosting_integrations_add` | `gitpoulpi hosting integrations add` | write |
| `hosting_integrations_list` | `gitpoulpi hosting integrations list` | read |
| `hosting_integrations_radar` | `gitpoulpi hosting integrations radar` | write |
| `hosting_integrations_remove` | `gitpoulpi hosting integrations remove` | destructive |
| `hosting_integrations_test` | `gitpoulpi hosting integrations test` | read, network |
| `hosting_issue_base` | `gitpoulpi hosting issue base` | write |
| `hosting_issue_close` | `gitpoulpi hosting issue close` | write |
| `hosting_issue_create` | `gitpoulpi hosting issue create` | write, network |
| `hosting_issue_edit` | `gitpoulpi hosting issue edit` | write, network |
| `hosting_issue_move` | `gitpoulpi hosting issue move` | write |
| `hosting_issue_options` | `gitpoulpi hosting issue options` | read, network |
| `hosting_issue_reopen` | `gitpoulpi hosting issue reopen` | write |
| `hosting_issues` | `gitpoulpi hosting issues` | read, network |
| `hosting_labels` | `gitpoulpi hosting labels` | read, network |
| `hosting_link` | `gitpoulpi hosting link` | write |
| `hosting_list` | `gitpoulpi hosting list` | read |
| `hosting_logout` | `gitpoulpi hosting logout` | destructive |
| `hosting_members` | `gitpoulpi hosting members` | read, network |
| `hosting_milestones` | `gitpoulpi hosting milestones` | read, network |
| `hosting_open` | `gitpoulpi hosting open` | read |
| `hosting_pr` | `gitpoulpi hosting pr` | write, network |
| `hosting_pr_edit` | `gitpoulpi hosting pr-edit` | write, network |
| `hosting_pr_for_branch` | `gitpoulpi hosting pr-for-branch` | read, network |
| `hosting_projects` | `gitpoulpi hosting projects` | read, network |
| `hosting_prs` | `gitpoulpi hosting prs` | read, network |
| `hosting_radar` | `gitpoulpi hosting radar` | read, network |
| `hosting_refs` | `gitpoulpi hosting refs` | read |
| `hosting_review_abandon` | `gitpoulpi hosting review abandon` | write, network |
| `hosting_review_comment` | `gitpoulpi hosting review comment` | write, network |
| `hosting_review_diff` | `gitpoulpi hosting review diff` | read, network |
| `hosting_review_fetch` | `gitpoulpi hosting review fetch` | write, network |
| `hosting_review_push` | `gitpoulpi hosting review push` | write, network |
| `hosting_review_reply` | `gitpoulpi hosting review reply` | write, network |
| `hosting_review_resolve` | `gitpoulpi hosting review resolve` | write, network |
| `hosting_review_show` | `gitpoulpi hosting review show` | read, network |
| `hosting_review_submit` | `gitpoulpi hosting review submit` | write, network |
| `hosting_review_submit_change` | `gitpoulpi hosting review submit-change` | write, network |
| `hosting_review_suggest` | `gitpoulpi hosting review suggest` | read, network |
| `hosting_review_threads` | `gitpoulpi hosting review threads` | read, network |
| `hosting_ssh_key_delete` | `gitpoulpi hosting ssh-key delete` | destructive, network |
| `hosting_ssh_key_list` | `gitpoulpi hosting ssh-key list` | read, network |
| `hosting_use` | `gitpoulpi hosting use` | write |
| `ignore` | `gitpoulpi ignore` | write |
| `index_lock_remove` | `gitpoulpi index-lock remove` | destructive |
| `index_lock_show` | `gitpoulpi index-lock show` | read |
| `job_cancel` | `gitpoulpi job cancel` | write |
| `job_list` | `gitpoulpi job list` | read |
| `job_wait` | `gitpoulpi job wait` | read |
| `lfs_checkout` | `gitpoulpi lfs checkout` | write |
| `lfs_fetch` | `gitpoulpi lfs fetch` | write, network |
| `lfs_files` | `gitpoulpi lfs files` | read |
| `lfs_install` | `gitpoulpi lfs install` | write |
| `lfs_lock` | `gitpoulpi lfs lock` | write, network |
| `lfs_locks` | `gitpoulpi lfs locks` | read, network |
| `lfs_prune` | `gitpoulpi lfs prune` | destructive |
| `lfs_pull` | `gitpoulpi lfs pull` | write, network |
| `lfs_push` | `gitpoulpi lfs push` | destructive, network |
| `lfs_status` | `gitpoulpi lfs status` | read |
| `lfs_track` | `gitpoulpi lfs track` | write |
| `lfs_unlock` | `gitpoulpi lfs unlock` | destructive, network |
| `lfs_untrack` | `gitpoulpi lfs untrack` | write |
| `licenses` | `gitpoulpi licenses` | read |
| `log` | `gitpoulpi log` | read |
| `lost_commits` | `gitpoulpi lost-commits` | read |
| `mailmap_add` | `gitpoulpi mailmap add` | write |
| `mailmap_check` | `gitpoulpi mailmap check` | read |
| `mailmap_list` | `gitpoulpi mailmap list` | read |
| `merge` | `gitpoulpi merge` | write |
| `patch_apply` | `gitpoulpi patch apply` | write |
| `patch_create` | `gitpoulpi patch create` | write |
| `path_status` | `gitpoulpi path status` | read |
| `pull` | `gitpoulpi pull` | write, network |
| `push` | `gitpoulpi push` | destructive, network |
| `rebase` | `gitpoulpi rebase` | destructive |
| `rebase_plan` | `gitpoulpi rebase-plan` | read |
| `redo` | `gitpoulpi redo` | destructive |
| `reflog` | `gitpoulpi reflog` | read |
| `reflog_restore` | `gitpoulpi reflog-restore` | destructive |
| `refs_hide` | `gitpoulpi refs hide` | write |
| `refs_show` | `gitpoulpi refs show` | write |
| `refs_solo` | `gitpoulpi refs solo` | write |
| `remote_add` | `gitpoulpi remote add` | write |
| `remote_clear_push_url` | `gitpoulpi remote clear-push-url` | write |
| `remote_default` | `gitpoulpi remote default` | write |
| `remote_list` | `gitpoulpi remote list` | read |
| `remote_remove` | `gitpoulpi remote remove` | destructive |
| `remote_rename` | `gitpoulpi remote rename` | write |
| `remote_set_url` | `gitpoulpi remote set-url` | write |
| `repo_close` | `gitpoulpi repo close` | write |
| `repo_discover` | `gitpoulpi repo discover` | read |
| `repo_init` | `gitpoulpi repo init` | write |
| `repo_list` | `gitpoulpi repo list` | read |
| `repo_open` | `gitpoulpi repo open` | write |
| `repo_recent` | `gitpoulpi repo recent` | read |
| `repo_config_set` | `gitpoulpi repo-config set` | write |
| `repo_config_show` | `gitpoulpi repo-config show` | read |
| `repo_config_unset` | `gitpoulpi repo-config unset` | write |
| `reset` | `gitpoulpi reset` | destructive |
| `revert` | `gitpoulpi revert` | write |
| `reword` | `gitpoulpi reword` | destructive |
| `rewrite_analyze` | `gitpoulpi rewrite analyze` | read |
| `rewrite_apply` | `gitpoulpi rewrite apply` | destructive |
| `rewrite_check_signing` | `gitpoulpi rewrite check-signing` | read |
| `rewrite_diff` | `gitpoulpi rewrite diff` | read |
| `rewrite_discard` | `gitpoulpi rewrite discard` | read |
| `rewrite_maps` | `gitpoulpi rewrite maps` | write |
| `rewrite_note` | `gitpoulpi rewrite note` | read |
| `rewrite_preview` | `gitpoulpi rewrite preview` | read |
| `rewrite_purge` | `gitpoulpi rewrite purge` | destructive |
| `rewrite_push` | `gitpoulpi rewrite push` | destructive, network |
| `rewrite_rows` | `gitpoulpi rewrite rows` | read |
| `rewrite_run` | `gitpoulpi rewrite run` | destructive |
| `rewrite_scan_secrets` | `gitpoulpi rewrite scan-secrets` | read |
| `rewrite_script_check` | `gitpoulpi rewrite script check` | read |
| `rewrite_script_test` | `gitpoulpi rewrite script test` | read |
| `rewrite_tools` | `gitpoulpi rewrite tools` | read |
| `rm_file` | `gitpoulpi rm-file` | destructive |
| `search` | `gitpoulpi search` | read |
| `session_delete` | `gitpoulpi session delete` | destructive |
| `session_list` | `gitpoulpi session list` | read |
| `session_new` | `gitpoulpi session new` | write |
| `session_rename` | `gitpoulpi session rename` | write |
| `show` | `gitpoulpi show` | read |
| `sparse_checkout_add` | `gitpoulpi sparse-checkout add` | write |
| `sparse_checkout_disable` | `gitpoulpi sparse-checkout disable` | write |
| `sparse_checkout_set` | `gitpoulpi sparse-checkout set` | write |
| `sparse_checkout_status` | `gitpoulpi sparse-checkout status` | read |
| `split_commit` | `gitpoulpi split-commit` | destructive |
| `split_source` | `gitpoulpi split-source` | read |
| `ssh_command_clear` | `gitpoulpi ssh-command clear` | write |
| `ssh_command_set` | `gitpoulpi ssh-command set` | write |
| `ssh_command_show` | `gitpoulpi ssh-command show` | read |
| `stage` | `gitpoulpi stage` | write |
| `stash_apply` | `gitpoulpi stash apply` | write |
| `stash_drop` | `gitpoulpi stash drop` | destructive |
| `stash_list` | `gitpoulpi stash list` | read |
| `stash_pop` | `gitpoulpi stash pop` | write |
| `stash_push` | `gitpoulpi stash push` | write |
| `status` | `gitpoulpi status` | read |
| `submodule_init` | `gitpoulpi submodule init` | write |
| `submodule_list` | `gitpoulpi submodule list` | read |
| `submodule_update` | `gitpoulpi submodule update` | write, network |
| `tag_annotate` | `gitpoulpi tag annotate` | write |
| `tag_create` | `gitpoulpi tag create` | write |
| `tag_delete` | `gitpoulpi tag delete` | destructive, network |
| `tag_delete_remote` | `gitpoulpi tag delete-remote` | destructive, network |
| `tag_fetch` | `gitpoulpi tag fetch` | write, network |
| `tag_list` | `gitpoulpi tag list` | read |
| `tag_push` | `gitpoulpi tag push` | write, network |
| `theme_export` | `gitpoulpi theme export` | read |
| `theme_list` | `gitpoulpi theme list` | read |
| `treat` | `gitpoulpi treat` | write |
| `ui_back` | `gitpoulpi ui back` | view |
| `ui_highlight` | `gitpoulpi ui highlight` | view |
| `ui_notify` | `gitpoulpi ui notify` | view |
| `ui_propose_branch` | `gitpoulpi ui propose branch` | propose |
| `ui_propose_commit_message` | `gitpoulpi ui propose commit-message` | propose |
| `ui_propose_resolution` | `gitpoulpi ui propose resolution` | propose |
| `ui_propose_reword` | `gitpoulpi ui propose reword` | propose |
| `ui_propose_rewrite` | `gitpoulpi ui propose rewrite` | propose |
| `ui_propose_split` | `gitpoulpi ui propose split` | propose |
| `ui_propose_tag` | `gitpoulpi ui propose tag` | propose |
| `ui_show_commit` | `gitpoulpi ui show commit` | view |
| `ui_show_conflict` | `gitpoulpi ui show conflict` | view |
| `ui_show_diff` | `gitpoulpi ui show diff` | view |
| `ui_show_file` | `gitpoulpi ui show file` | view |
| `ui_show_home` | `gitpoulpi ui show home` | view |
| `ui_show_issue` | `gitpoulpi ui show issue` | view |
| `ui_show_keys` | `gitpoulpi ui show keys` | view |
| `ui_show_palette` | `gitpoulpi ui show palette` | view |
| `ui_show_pr` | `gitpoulpi ui show pr` | view |
| `ui_show_ref` | `gitpoulpi ui show ref` | view |
| `ui_show_search` | `gitpoulpi ui show search` | view |
| `ui_show_settings` | `gitpoulpi ui show settings` | view |
| `ui_show_tab` | `gitpoulpi ui show tab` | view |
| `ui_show_wip` | `gitpoulpi ui show wip` | view |
| `ui_state` | `gitpoulpi ui state` | view |
| `ui_viewers` | `gitpoulpi ui viewers` | view |
| `undo` | `gitpoulpi undo` | destructive |
| `undo_forget` | `gitpoulpi undo-forget` | write |
| `undo_restore` | `gitpoulpi undo-restore` | write |
| `unstage` | `gitpoulpi unstage` | write |
| `workspace_add` | `gitpoulpi workspace add` | write |
| `workspace_create` | `gitpoulpi workspace create` | write |
| `workspace_delete` | `gitpoulpi workspace delete` | destructive |
| `workspace_list` | `gitpoulpi workspace list` | read |
| `workspace_remove` | `gitpoulpi workspace remove` | write |
| `workspace_rename` | `gitpoulpi workspace rename` | write |
| `workspace_set_ai_instructions` | `gitpoulpi workspace set-ai-instructions` | write |
| `workspace_set_profile` | `gitpoulpi workspace set-profile` | write |
| `workspace_show` | `gitpoulpi workspace show` | read |
| `worktree_add` | `gitpoulpi worktree add` | write |
| `worktree_list` | `gitpoulpi worktree list` | read |

Not exposed as tools:

| CLI | Why |
|---|---|
| `gitpoulpi mcp serve` | the MCP server cannot spawn/serve a copy of itself |
| `gitpoulpi completions` | shell completion generation, not a GitBackend operation |
| `gitpoulpi shell-init` | prints a wrapper function for the user's own shell, not a GitBackend operation |
| `gitpoulpi desktop shortcut status` | launcher shortcuts are the user's desktop, not a repository; they need the TUI's icon renderer |
| `gitpoulpi desktop shortcut install` | writes files into the user's launcher, Start menu and Desktop |
| `gitpoulpi desktop shortcut remove` | deletes files from the user's launcher, Start menu and Desktop |
| `gitpoulpi alias create` | writes a command into the user's PATH; an agent never changes the user's shell |
| `gitpoulpi alias remove` | deletes a command from the user's PATH; an agent never changes the user's shell |
| `gitpoulpi path add` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi path remove` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi uninstall` | removes GitPoulpi's own program files, aliases, PATH entries and shortcuts from the machine |
| `gitpoulpi launch` | starts a terminal window on the user's desktop; the launcher shortcuts' own entry point |
| `gitpoulpi daemon run` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon start` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon stop` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon restart` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon status` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi rewrite script trust` | trusting an unsandboxed script is a person's decision; an agent can only run scripts a person trusted |
| `gitpoulpi config set` | an agent must not be able to rewrite its own gating config |
| `gitpoulpi hosting ssh-key create` | writes a private key and uploads it; too dangerous to let an agent automate |
| `gitpoulpi hosting login` | takes a secret token on stdin; a token must not pass through an agent's tool-call transcript |
<!-- mcp-tools:end -->

### ブランチワークフローのアクションは「プランしてから実行」

`flow_start`、`flow_finish`、`flow_sync`、`flow_promote`、`flow_set_parent`
（[ブランチワークフロー](branching-workflows.md#アクション)）は、単独の呼び出しでは決して実行されません。
`dry_run: true` はプラン（すべての git ステップとコマンド、警告、Undo で取り消せないもの、ブロッカー）を
通常の結果として返し、`yes: true` はそれを実行します。どちらもない場合、呼び出しは拒否され、プランが添えられます。
ワークフローファイルの `[hooks]` は、チームで共有するファイル由来のシェルコマンドなので、明示的な
`run_hooks: true` がある場合にのみ実行されます：プランにはフックが一覧されるので（`hooks`、`hooks_run`）、
先にユーザーに正確なコマンドを見せられます。競合で停止した merge は `status: "conflicted"`
（エラー結果、終了コード 2）を返します。競合ツールでファイルを解決した後、同じツールを `continue_: true`
または `abort: true` で呼び出します。マージリクエストを作成すること（`merge-request` モードの `finish`）は
元に戻せず、プランにもそう書かれます。

### 素の `git` での操作も元に戻す履歴に入る

エージェントが、これらのツールを使わず自分の `git` でブランチやワークツリーを作成したり、commit、reset、
stash の drop を行ったりしても、その操作は見えます。デーモンがリポジトリの参照を前回確認した内容と比較し、
変更ごとに `undo --list` の項目として、`"source": "outside"`、git の理由（`outside.message`）、
git が記録したコミッターとともに記録します。`undo`、`redo`、`undo_restore` ツールはこれらの項目をほかの項目と同様に扱うため、
エージェント（またはユーザーが `Ctrl+Z` で）、削除されたブランチを取り戻せます。
ファイルの編集は記録されません。[元に戻す・やり直す](undo-redo.md#gitpoulpi-の外で行われた変更)を参照してください。

## ユーザーに表示する

エージェントは、説明する代わりに、何かを指し示すことができます：`ui_show_commit`、`ui_show_diff`（行位置を指定したファイル。インラインまたは分割）、`ui_show_file`（行位置を指定した内容、blame、履歴）、`ui_show_ref`、`ui_show_pr`、`ui_show_issue`、`ui_show_conflict`（競合中のファイルで merge エディターを、指定した競合で開く）、`ui_show_wip`（`composer` は commit の概要にカーソルを置くだけで、何も入力しません）、`ui_show_tab`、`ui_show_home`、`ui_show_search`、`ui_show_palette`（クエリを入力するだけで、実行はしません）、`ui_show_keys`、`ui_show_settings`（ページと設定項目を示すだけで、編集はしません）、`ui_highlight`（開いている diff やファイルの行に色を付ける）、`ui_notify`（トースト通知）、`ui_back`（エージェントによる直前の表示変更を元に戻す）。`ui_state` はウィンドウの表示内容を読み取ります（`text: true` で表示中のテキストも含まれます）。`ui_viewers` はウィンドウを一覧します。

これらのツールは**リポジトリを一切変更せず**、入力、クリック、設定の編集もできません。リビジョンとパスは、ウィンドウに何かが届く前に検証されます。

**ユーザーに見えること、決められること：**

- `ask`（既定）の場合、またはユーザーが入力中、ダイアログや設定が開いている、ドラッグ中、直近 1.5 秒以内にキーを押した場合：エージェントにはすぐ `deferred` が返り、ステータスバーにチップ*「claude-code が...を表示しようとしています」*が出ます。`Ctrl+N Enter` またはクリックで表示し、`Esc` または `Ctrl+N Backspace` で却下します。60 秒で期限切れになります。
- 適用された場合：表示が移動し、誰が動かしたかを示すトーストが 4 秒間出て、タブには次のキー入力まで紫色の `✦` が付きます。`Ctrl+N Backspace`（パレットの `エージェント: 戻る`）で戻れます。ウィンドウは 10 個のビューを記憶します。
- 非表示のものが表示されたり、チェックアウトされたりすることはありません：ソロフィルターで隠されている commit には `partial`（`hidden_by_filter`）が返ります。プロバイダーにリンクされていない PR や Issue の一覧には `hosting_not_linked` が返ります。

**応答：** `{"outcome": "applied" | "partial" | "deferred" | "refused" |
"not_found" | "no_viewer" | "proposed", ...}`。適用された場合はウィンドウの `state` も含まれます。CLI の終了コード：0 は applied/partial/proposed、3 は deferred、1 はそれ以外。呼び出しにはレート制限があります（バースト 5 回、その後は 1 秒に 1 回）。

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "The bug is the early return above."
$ gitpoulpi --json ui back
```

### 変更を提案する（`ui_propose_*`）

`allow_ui_propose = true` にすると、エージェントはユーザーのダイアログを**入力済み**の状態で開き、*✦ Proposed by <agent>*という印を付けられます。そのダイアログ自身の確定を押すまで、何も起こりません。先に編集することも、キャンセルすることもできます。

| ツール | 開くもの | 確定の方法 |
|---|---|---|
| `ui_propose_reword`（rev、message） | メッセージ変更のポップアップ | *メッセージを更新*（push 済みの commit では、強制 push についても確認されます） |
| `ui_propose_commit_message`（summary、body） | 入力済みの commit フォーム。commit は決してしません | commit ボタン |
| `ui_propose_branch`（name、start point） | インラインの新規ブランチボックス | `Enter` |
| `ui_propose_tag`（name、target、message） | インラインの新規 tag ボックス（メッセージがあれば注釈付き） | `Enter` |
| `ui_propose_rewrite`（レシピファイル） | レシピのルールが入った履歴書き換えワークベンチ。プレビューも適用もされません | ご自身での *プレビュー* と *適用*（`Ctrl+Enter`）。`Esc` で却下。スクリプトを含むレシピは拒否され、シークレット値は取り除かれます |
| `ui_propose_split`（rev、計画ファイル） | commit の上に開く分割エディター（計画が入力済み） | *分割*（`Ctrl+S`）。`Esc` で却下 |
| `ui_propose_resolution`（path、pick または content file） | 競合中のファイルで merge エディターを開き、選択（またはテキスト全体）が出力にすでに入った状態にする | *保存して解決済みにする*（`Ctrl+S`）。閉じると却下になります |

同時に出せる提案は 1 つです。コンポーザーにユーザー自身の下書きがあるときは commit メッセージの提案は拒否され、merge エディターにユーザーの未保存の作業があるとき、またはエージェントが読み取った後にファイルのステージが変わったときは、解決案の提案が拒否されます。確定する前にユーザーが変更した解決案は、`accepted_edited` として報告されます。応答には `proposal` ID が含まれます。ユーザーの判断は、`ui_state` の `proposals`（`offered`、`open`、`accepted`、`accepted_edited`、`rejected`、`expired`、`failed`）に残ります。

### ウィンドウが開いていない場合、または別のセッションにある場合

- ウィンドウは、起動されたセッションに属します。エージェントのセッションにウィンドウがない場合、`ui show` は、そのリポジトリを開いている別のセッションの**ウィンドウを 1 つ**探して、それを使います（応答の `routed_session`）。複数が該当する場合は、それらの名前を挙げた `no_viewer` が返ります。`--session` を指定してください。
- **エージェント用にウィンドウを開く**機能は、既定ではオフです。`[ui] launch`（設定 > エージェントと MCP > *エージェント用ウィンドウを開く*）に、ターミナルまたはマルチプレクサのプリセット、あるいは `custom` コマンドを設定します。すると、`launch: true` 付きの呼び出し（CLI では `gitpoulpi ui --launch show ...`）がウィンドウを開き、最大 10 秒待ちます。`ui_control = "auto"` の場合は、フラグなしでも開かれます。`ask` の場合、エージェントには `no_viewer` が返るので、ユーザーに伝えるはずです。

| プリセット | コマンド |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}`（既定のターミナル） |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux`（新しいウィンドウ）、`zellij`（新しいペイン）、`cmux`（新しいワークスペース） | 起動中のマルチプレクサ内の、リポジトリのフォルダー |
| `custom` | `[ui] launch_command`。例：`my-term --cd {repo} {gitpoulpi} --session {session}`。引数に分割され、シェルは使いません |

プリセットがお使いのバージョンのプログラムで動作しない場合は、`custom` を使ってください。

## エラー

- 不正な呼び出し（不明なツールや引数、必須引数の欠落）は、JSON-RPC の `-32602 invalid_params` エラーになります。
- 拒否、または git やデーモンのエラー（不正なリビジョン、競合など）は、ツールエラーです：`isError: true` で、メッセージは `content` と `structuredContent.error` に入ります。
