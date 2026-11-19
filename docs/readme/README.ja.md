[English](../../README.md) · [Čeština](README.cs.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · **日本語** · [한국어](README.ko.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [简体中文](README.zh-CN.md)

# GitPoulpi

<p align="center"><img src="../../assets/logo/gitpoulpi-512.png" alt="GitPoulpi のタコ" width="128"></p>

**デスクトップアプリのように使える、ターミナル向けの Git クライアントです。**
GitPoulpi には、クリック、ドラッグ、右クリックができる本格的な commit グラフ、
あらゆる機能に届くキーボード中心のコマンドパレット、コードホスティング（GitHub、GitLab、Gitea、Forgejo、Azure DevOps、Bitbucket、Gerrit。クラウドもセルフマネージドも可）とイシュートラッカー（Jira、Trello、Linear、YouTrack、Redmine、Markdown ノートのフォルダー）との連携があります。
同じ操作はスクリプト可能な CLI と MCP サーバーとしても使えるので、AI エージェントがあなたと
まったく同じように操作できます。GitPoulpi は GitKraken にインスピレーションを得て、Rust でゼロから書き直したものです。
ネイティブのバイナリで、ディスクにもメモリにも軽く、Electron もアカウントもテレメトリーも不要です。お気に入りのターミナル（[どれ？](../../docs/user/ja/terminals.md)）で動きます。私たちの中には、そこに住んでいる人もいるので。

![ブランチ、tag、stash を含む commit グラフ、左側の Refs サイドバー、右側の作業中の変更と commit フォームを表示する GitPoulpi](../../docs/user/images/hero.png)

- **マウスとキーボードを同等に。** クリック、ダブルクリック、右クリック、ref の commit へのドラッグ＆ドロップができます。
  または `Ctrl+P` を押して、やりたいことを入力します。すべてのマウス操作にキーボードの対応操作があり、
  すべてのコマンドがパレットにあります。
- **本格的な commit グラフ。** ブランチ、merge、tag、stash、作業コピーを 1 つのスクロールする
  ログに表示します。対応するターミナルではピクセルで、それ以外ではどこでも文字で描画します。
  多数のダーク/ライトのカラーテーマ、作者のアバター、検索があります。
- **正確にステージ。** ファイル、フォルダー、ハンク、1 行単位で、diff から直接ステージ、
  ステージ解除、破棄ができます。インラインでも左右並べでも、ミニマップ付きです。
- **履歴を安全に編集。** 任意の commit のメッセージの書き換えや修正（メッセージ、作成者、日付）、
  確認画面付きの対話的な rebase、プレビュー付きで 1 回で取り消せる多数の commit の一括書き換え、
  複数の commit の cherry-pick、fast-forward、worktree とパッチの作成ができます。
- **ホスティングとイシュートラッカーを内蔵。** pull request と merge request（GitHub、GitLab、Gitea、Forgejo、Azure DevOps、Bitbucket）の閲覧、チェックアウト、作成と、Gerrit の change のレビューができます。Issue、ワークアイテム、カード（Jira、Trello、Linear、YouTrack、Redmine、ローカルの Markdown ノートも）の閲覧と作成ができ、カンバンボードでも扱えます。アカウントからのクローンと SSH 鍵の管理もできます。
- **エージェント向けの CLI と MCP。** `gitpoulpi <command> --json` は UI のすべての操作をカバーし、
  `gitpoulpi-mcp` はそれを MCP ツールとして提供します。リポジトリごとに 1 つのバックグラウンド
  デーモンが、すべてのフロントエンドをライブに同期した状態に保ちます。
- **AI は任意で、必須ではありません。** 既定ではオフで、アカウントも不要です。自分の API キーか、
  `claude -p` のようなローカルのエージェント CLI を使います。commit メッセージの生成、変更の説明、
  pull request の説明文の作成、作業中の変更の複数 commit への分割、履歴書き換えルールの下書きができます。
- **Git LFS、サブモジュール、worktree、パッチ、blame とファイル履歴、画像の diff、内蔵エディター。**

## スクリーンショット

| | |
|---|---|
| ![Material レーンパレットと行の色付けを有効にした ログのグラフ](../../docs/user/images/theme-material.png) **commit グラフ**。ここでは Material レーンパレットと行の色付けを使っています。 | ![各ハンクにハンクをステージとハンクを破棄のボタンがある、変更したファイルの diff](../../docs/user/images/diff.png) **diff**。ハンク単位または行単位でステージと破棄ができます。 |
| ![stash という語で絞り込んだコマンドパレット](../../docs/user/images/palette.png) **コマンドパレット**（`Ctrl+P`）から、すべてのコマンドに届きます。 | ![作業中の変更に対して 4 つの commit を提案し、確認できる状態の commit コンポーザー](../../docs/user/images/composer.png) **commit コンポーザー**は、AI が提案した commit を、何かを commit する前に確認できます。 |
| ![cherry-pick、revert、reset、rebase、ブランチ、tag の操作がある commit の右クリックメニュー](../../docs/user/images/context-menu.png) どこでも**右クリック**すると、その操作が表示されます。 | ![プロファイル、操作ごとの指示、有効化スイッチがある設定の AI ページ](../../docs/user/images/settings-ai.png) **設定**には検索があり、任意の AI ページもあります。 |

さらに多くの画像と説明は、[ユーザードキュメント](../../docs/user/ja/README.md)にあります。

## インストール

システムの `git` が必要です。`git-lfs` は任意です。

### Linux と macOS (`install.sh`)

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

### Homebrew (macOS、Linux)

```
brew install Poulpi-Lab/gitpoulpi/gitpoulpi
```

### apt (Debian、Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

### Scoop (Windows)

```
scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi
scoop install gitpoulpi
```

### Windows インストーラー

[最新リリース](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest)から
`gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` をダウンロードして実行します。
管理者向けには MSI があります。

```
msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn
```

### Docker

```
docker pull poulpilab/gitpoulpi
```

[Docker での実行](../user/ja/docker.md)を参照してください。

### cargo

```
cargo binstall gitpoulpi-cli          # prebuilt
cargo install --locked gitpoulpi-cli  # compiles
```

### リリースアーカイブ

[最新リリース](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest)からダウンロードし、
展開して、中の `INSTALL.txt` に従ってください。

### ソースから

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

自分でリリースアーカイブをビルドする方法、より小さなカスタムビルド、OS ごとの注意点：
[docs/user/ja/installation.md](../user/ja/installation.md) と
[docs/user/ja/custom-builds.md](../user/ja/custom-builds.md)。

## クイックスタート

```
cd path/to/your/repository
gitpoulpi
```

- `?` ですべてのキーと操作を載せたヘルプ、`Ctrl+P` でコマンドパレット、
  `Ctrl+,` で設定が開き、`q` で終了します。
- commit をクリックするとその変更が表示されます。ログの最上行（作業中の変更）をクリックすると、
  ファイルをステージして commit できます。
- 対応するターミナル（kitty、Ghostty、WezTerm、iTerm2、sixel 対応の Windows Terminal と foot）では、グラフはピクセルで描画されます。
  それ以外（素の Windows コンソール、tmux）では、自動的に文字描画に切り替わります。

手順を追った説明は[はじめに](../../docs/user/ja/getting-started.md)にあり、
[インターフェースツアー](../../docs/user/ja/interface-tour.md)では画面のすべての領域に名前を付けています。

```
gitpoulpi --json status            # the CLI, scriptable
gitpoulpi --json log --limit 20
claude mcp add gitpoulpi -- gitpoulpi-mcp   # give an AI agent the same tools
```

AI は任意で、既定ではオフです。GitPoulpi 自身の AI は *設定 > AI* でオンにします
（[AI アシスタント](../../docs/user/ja/ai.md)）。外部のエージェントに GitPoulpi を操作させるには
[MCP とエージェント](../../docs/user/ja/mcp-and-agents.md)（権限、すべてのツール）と、
エージェントスキル [`skills/gitpoulpi/SKILL.md`](../../skills/gitpoulpi/SKILL.md) を使います。

## ドキュメント

ユーザードキュメントの全体は **[docs/user/ja/README.md](../../docs/user/ja/README.md)** から始まります。

| | |
|---|---|
| [はじめに](../../docs/user/ja/getting-started.md) | インストール、リポジトリを開く、最初の commit |
| [インターフェースツアー](../../docs/user/ja/interface-tour.md) | ウィンドウのすべての領域の名前 |
| [ログを使う](../../docs/user/ja/log-and-history.md) | グラフ、refs、ブランチ、tag、stash、rebase、cherry-pick |
| [ステージと commit](../../docs/user/ja/staging-and-committing.md) | ステータス、diff、ハンクと行、コンポーザー、エディター |
| [設定](../../docs/user/ja/settings.md) | 設定のすべてのページとオプション |
| [AI アシスタント](../../docs/user/ja/ai.md) | 内蔵 AI の設定、すべての AI 操作、プライバシー |
| [ホスティングとイシュートラッカー](../../docs/user/ja/hosting-integration.md) | GitHub、GitLab、Gitea、Forgejo、Azure DevOps、Bitbucket、Gerrit、Jira、Trello、Linear、YouTrack、Redmine：トークン、リポジトリの紐付け、pull request と merge request |
| [CLI](../../docs/user/ja/cli-reference.md) と [MCP](../../docs/user/ja/mcp-and-agents.md) | スクリプトから使う、またはエージェントに操作させる（権限、すべてのツール） |
| [キーバインド](../../docs/user/ja/keybindings.md)（とパレットのすべてのコマンド）、[マウス](../../docs/user/ja/mouse-and-gestures.md)、[設定リファレンス](../../docs/user/ja/config-reference.md)、[LFS](../../docs/user/ja/lfs.md)、[元に戻す](../../docs/user/ja/undo-redo.md)、[トラブルシューティング](../../docs/user/ja/troubleshooting.md) | リファレンスとヘルプ |

開発者向け：[CLAUDE.md](../../CLAUDE.md) と `docs/architecture/overview.md` に crate と
設計が説明されています。[CHANGELOG.md](../../CHANGELOG.md) には各リリースの内容が載っています。

## ライセンス

[MIT](../../LICENSE)。

## クレジットとサードパーティのライセンス

GitPoulpi はオープンソースのライブラリの上に成り立っており、公開されているテーマの色の値を使っています。それらのライセンスとクレジット（テーマの作者を含む）は [THIRD-PARTY-LICENSES.txt](../../THIRD-PARTY-LICENSES.txt) にあります。このファイルはすべてのリリースに同梱され、「GitPoulpi について」ウィンドウと `gitpoulpi licenses` でも表示できます。

## 商標

GitKraken はそれぞれの所有者の商標です。GitPoulpi は独立したプロジェクトであり、その所有者とは提携しておらず、承認やスポンサーも受けていません。この名称は互換性とインスピレーションを説明する目的でのみ記載しています。
