[English](../README.md) · [Čeština](../cs/README.md) · [Deutsch](../de/README.md) · [Español](../es/README.md) · [Français](../fr/README.md) · [Italiano](../it/README.md) · **日本語** · [한국어](../ko/README.md) · [Nederlands](../nl/README.md) · [Polski](../pl/README.md) · [Português (Brasil)](../pt-BR/README.md) · [Português (Portugal)](../pt-PT/README.md) · [Русский](../ru/README.md) · [Svenska](../sv/README.md) · [Türkçe](../tr/README.md) · [Українська](../uk/README.md) · [简体中文](../zh-CN/README.md)

# GitPoulpi ユーザードキュメント

<p align="center"><img src="../../../assets/logo/gitpoulpi-512.png" alt="GitPoulpi のタコ" width="128"></p>

GitPoulpi は、クリックできる commit グラフ、コマンドパレット、ホスティングと Issue トラッカーとの連携、
任意の AI 支援、そして UI のすべての操作ができる CLI と MCP サーバーを備えた、ターミナル向けの Git クライアントです。
これらのページは読む順に並んでいます。まず最初から読み、必要になったらリファレンスのページに移動してください。

![GitPoulpi のメインウィンドウ](../images/hero.png)

## はじめに

1. **[インストール](installation.md)**：必要なもの、リリースアーカイブ、
   パッケージマネージャー、ソースからのビルド。より小さなバイナリが必要な場合は
   **[カスタムビルド](custom-builds.md)** を参照してください。GitPoulpi が自分を
   最新に保つ仕組み（バージョンのチップ、更新パネル、元に戻す）は
   **[更新](updates.md)**、コンテナーで動かすには **[Docker で実行する](docker.md)**、
   管理者向けの社内フィード、ミラー、更新ポリシーは
   **[エンタープライズ配布](enterprise-distribution.md)** を参照してください。
2. **[はじめに](getting-started.md)**：起動から最初の commit までの、最初の 5 分間。
3. **[インターフェースツアー](interface-tour.md)**：ウィンドウのすべての領域の名前を、画像付きで説明します。

## 日常の使い方

4. **[ログを使う](log-and-history.md)**：グラフの読み方、検索、
   ブランチ、tag、stash、worktree、参照のドラッグ、rebase、cherry-pick、
   reword と amend、バナー、pull と push、auto-stash。
5. **[ステージと commit](staging-and-committing.md)**：作業中の変更、
   diff、ハンクや 1 行単位のステージ、commit の書き方、commit コンポーザー、
   blame、ファイル履歴、内蔵エディター。
6. **[画像](image-view.md)**：Diff ビューとファイルビューの画像、
   変更前と変更後、ズーム、編集に使う。
7. **[merge 競合の解決](merge-conflicts.md)**：merge エディター（A と B を
   出力の上に左右に並べて表示）、行やチャンクの選択、バイナリ、
   LFS、削除、シンボリックリンク、サブモジュールの競合用のカード、そしてエージェントが
   解決案を提案する方法。
8. **[元に戻す・やり直す](undo-redo.md)**：`Ctrl+Z` / `Ctrl+Y`、元に戻せるものと戻せないもの、
   拒否したり確認したりする場合、どこまで遡れるか、オフにする方法。
9. **[Git LFS](lfs.md)**：大きなファイル、追跡、ロック、そして
   `git-lfs` がインストールされていない場合の動作。
10. **[サブモジュール](submodules.md)**：サブモジュールセクション、更新方法、
    checkout や pull との同期の保ち方。
11. **[テーマ](themes.md)**：内蔵テーマ、任意のテーマと任意のレーンパレットの組み合わせ、
    ライブプレビュー、独自のテーマファイル、`gitpoulpi theme`。
12. **[ワークスペース](workspaces.md)**：リポジトリに名前を付けたグループ。専用のタブと、
    任意の既定 commit プロファイルを持てます。タブバーのチップ、ワークスペースのタブ、すべてFetch、すべてPull、見つからないものをクローン。
13. **[検索](search.md)**：ワークスペースまたは開いているタブの全リポジトリの
    ブランチ、tag、worktree、stash、commit、pull/merge request、Issue を横断する
    1 つのあいまい検索（`Ctrl+Alt+F`）。
14. **[Radar](radar.md)**：ワークスペース（または開いているタブ）の
    pull/merge request、Issue、作業中の変更を 1 つの一覧に：フィルター、固定、
    保留、各行でできること。
    アプリ内でこれらのリクエストを読み、コメントのスレッド、保留中のレビュー、承認 / 変更の要求とともにレビューする方法は、
    **[pull request / merge request のレビュー](review.md)** を参照してください。
15. **[開く場所](open-in.md)**：タイトルバーの一覧から、アクティブなタブの
    フォルダーをターミナル、マルチプレクサ、エディター、ファイルマネージャーで開きます。
16. **[アクティブなタブのフォルダーに追従](following-the-tab-folder.md)**：
    「終了して cd」と、bash、zsh、fish、PowerShell 用の
    `gitpoulpi shell-init` ラッパー（`gp`）。
    GitPoulpi を最もよく表示できるターミナル：**[ターミナルの選び方](terminals.md)**。
    tmux、zellij、screen、herdr、ターミナルのペインを通るもの：
    **[マルチプレクサ](multiplexers.md)**。

そのほか：**[1 つのブランチに複数のリモート](multiple-remotes.md)**：push 先と pull 元、リモートのダイアログ、
すべてまたは選択したリモートへの Push、動いたリモートの解決、競合後の続行、CLI、素の git のエイリアス。

**[ブランチワークフロー](branching-workflows.md)**。Git Flow、GitHub Flow、GitLab Flow、
トランクベースのプリセット、または独自のベースブランチ、ブランチ名のパターン、トピックの種類
（`gitpoulpi flow ...`）。

**[履歴の書き換え](history-rewrite.md)**：多数の commit にわたって作者、日付、メッセージ、
ファイルの内容を書き換えるワークベンチ（`Ctrl+G Shift+H`）。プレビュー、Lua スクリプト、
1 回の「元に戻す」に対応します。

## キーボードとマウス

17. **[キーバインド](keybindings.md)**：領域ごとのすべてのキーと、パレットのすべての
    コマンド。アプリ内では `?` で同じ一覧が表示され、`Ctrl+P` で任意の
    コマンドを探せます。
18. **[マウスとジェスチャー](mouse-and-gestures.md)**：クリック、ドラッグ、
    選択とコピー、スクロールバー、ホバー時の表示。

## 設定と連携

19. **[設定](settings.md)**：設定画面のすべてのページとオプション、
    検索ボックス、プロファイル、ショートカットの再割り当て方法。
20. **[AI アシスタント](ai.md)**：GitPoulpi 自身の AI（既定ではオフ）：`claude -p` のような
    ローカルエージェント、Anthropic、OpenAI、OpenRouter、Ollama、
    LM Studio、指示、commit メッセージ、説明、pull request の
    説明文と commit コンポーザー、プライバシーとトラブルシューティング。
21. **[ホスティングと Issue トラッカー](hosting-integration.md)**：GitHub、GitLab、Gitea と Forgejo、
    Azure DevOps、Bitbucket、Gerrit、各 Issue トラッカー：
    トークンとそのスコープ、
    接続、リポジトリの紐付け、pull request と Issue、
    commit メッセージ内の参照、SSH 鍵、既知の制限。あわせて
    **[ローカル Issue と Obsidian](local-issues.md)**：Markdown ノートのフォルダーを
    Issue の取得元にする方法。トークン不要で、Obsidian Base も使えます。

## 自動化

22. **[CLI リファレンス](cli-reference.md)**：すべての `gitpoulpi` コマンドと、
    スクリプト向けの `--json`。
23. **[MCP とエージェント](mcp-and-agents.md)**：外部のエージェント（Claude Code、
    Cursor、Codex など）に GitPoulpi を操作させる方法：サーバーの登録、`.mcp.json`、
    権限スイッチとその既定値、すべてのツール、そしてエージェントがウィンドウ内で
    何かを見せる方法。用意済みのエージェントスキルは
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md) です。

## リファレンスとヘルプ

24. **[設定リファレンス](config-reference.md)**：`config.toml` のすべてのキー、
    その既定値と効果、リポジトリごとの設定、環境変数。
25. **[トラブルシューティングと FAQ](troubleshooting.md)**：アイコン、グラフの描画、
    届かないキー、固まったデーモン、速度、LFS ポインター。

## スクリーンショット

これらのページのすべての画像は、作者名を架空にした小さなデモリポジトリで、リリースビルドを実際に
キャプチャしたものです。画像は [`images/`](../images/) にあります。
`scripts/docs-screenshots.sh` がデモリポジトリを作成し、撮り直します。
画像は `kitty` ターミナルで撮影しており、commit グラフと作者のアバターが画像として描画されます。
グラフィックスプロトコルのないターミナルでは、代わりに文字のグラフが表示されます。

## コントリビューター向け

[CLAUDE.md](../../../CLAUDE.md)、[アーキテクチャ概要](../../architecture/overview.md)、
[ロードマップ](../../roadmap.md)に設計と今後の予定が説明されています。
[CHANGELOG.md](../../../CHANGELOG.md) には各リリースの変更点が載っています。

---

GitKraken はそれぞれの所有者の商標です。GitPoulpi は独立したプロジェクトであり、その所有者とは提携しておらず、承認やスポンサーも受けていません。この名称は互換性とインスピレーションを説明する目的でのみ記載しています。
