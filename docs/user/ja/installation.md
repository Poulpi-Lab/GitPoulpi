[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · **日本語** · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# インストール

GitPoulpi には 3 つの実行ファイルがあります。

| 実行ファイル | 内容 |
|---|---|
| `gitpoulpi-tui` | ターミナル UI。 |
| `gitpoulpi` | コマンドラインインターフェース（スクリプト向けの `--json`）。サブコマンドなしで実行すると UI が開きます。 |
| `gitpoulpi-mcp` | AI エージェント向けの MCP サーバー。[MCP とエージェント](mcp-and-agents.md)を参照してください。 |

アーカイブには、これらに加えて `gitpoulpi-launch` が入っています。これはショートカットが、すでに開いている GitPoulpi のウィンドウを表示するために実行する小さなプログラムです（`gitpoulpi launch`）。Windows ではコンソールウィンドウを持たないため、クリックしても一瞬表示されることはありません。ほかのファイルと一緒にインストール、更新、削除されます。

## 必要なもの

- `PATH` 上の**システムの `git`**。GitPoulpi はほとんどの操作に libgit2 を使い、
  残りにはシステムの `git` を使います。
  バージョン **2.34 以降**が必要です。macOS では Apple の Command Line Tools に含まれる `git` でも Homebrew の `git` でも動作します。
- マウスに対応した**ターミナル**。kitty、Ghostty、WezTerm、Rio、iTerm2、sixel 対応の Windows Terminal、foot、Tabby は
  commit グラフを画像として描画します。その他のターミナル（Windows コンソール、
  zellij、screen、ほかのほとんど）では文字のグラフが表示されます。
  [ターミナルの選び方](terminals.md)と[マルチプレクサ](multiplexers.md)を参照してください。
- ツールバーとサイドバーのアイコン用に、**Nerd Font** または絵文字を含むフォント。
- 任意：Git LFS リポジトリ用の **`git-lfs`** 3.0 以降。ない場合、
  LFS ファイルは小さなポインターファイルとして表示され、GitPoulpi がその理由を伝えます。
  インストール方法は [Git LFS](lfs.md) を参照してください。
- 任意：ホスティングのトークンと AI の API キーを保存するための **認証情報ヘルパー**
  （Git Credential Manager、libsecret、`pass`、Keychain）。
  [ホスティングと Issue トラッカー](hosting-integration.md)を参照してください。

## リリースアーカイブから

リリースアーカイブは `gitpoulpi-<version>-<target>.zip`（Windows）または
`.tar.gz`（Linux、macOS）で、隣に `.sha256` ファイルがあります。
アーカイブを `.sha256` ファイルで確認し、展開して、3 つの実行ファイルを
`PATH` 上のフォルダーに置いてください（`gitpoulpi-updater` と `.gitpoulpi-dist`
マーカーはその隣に残してください。マーカーには `channel=archive` と書かれていて、
これによりビルドが自己更新できます。[更新](updates.md)を参照）。
アーカイブ内の `INSTALL.txt` にも同じ手順があります。

### ダウンロードを検証する

すべてのリリースファイルには、プロジェクトの [minisign](https://jedisct1.github.io/minisign/) 鍵で署名が付いており、GitPoulpi は更新をインストールする前にその署名を自分で検証します。手動でダウンロードしたファイルの場合、その隣にある `.minisig` が署名です。公開鍵（鍵 ID `B461FFCBF6F6A4F2`）は、リポジトリの `packaging/minisign.pub` にコミットされています。

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

コマンドが出力する信頼済みコメントを確認してください。`file:<ファイル名> version:<期待するバージョン>` になっている必要があります。署名自体は有効でも、別のファイルや別のバージョンを指している場合、それはあなたのファイルの署名ではありません。

リリースがビルド来歴の証明（リポジトリの `ATTESTATIONS` 変数）付きでビルドされている場合は、GitHub CLI でファイルがどこでビルドされたかを確認できます。

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

`.sha256` ファイルで分かるのは、ダウンロードの破損だけです。そのファイルがプロジェクトのものであることを示すのは署名です。

## インストーラーで

各リリースには、OS ごとのインストーラーも付いています。どちらも、インストールされた `gitpoulpi` 自身（設定 > ランチャーの裏側にあるコード）を呼び出して同じ選択（ショートカット、PATH、`gp` のようなコマンドのエイリアス）を行うため、インストーラーの設定内容と設定画面の表示はつねに一致します。

### Windows：`gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

最初のダイアログで、インストールの対象を尋ねます。

| 選択 | 場所 | 管理者権限 | 更新への影響 |
|---|---|---|---|
| **自分のみ**（既定） | `%LOCALAPPDATA%\Programs\GitPoulpi` | 不要 | GitPoulpi が自分のファイルを置き換えます：[更新パネル](updates.md)の**更新して再起動**で、確認はありません。 |
| **全ユーザー** | `Program Files\GitPoulpi` | インストール時に 1 回確認 | 更新のたびに管理者の承認（Windows の確認）を求め、このコンピューター上の他のユーザーの GitPoulpi セッションを先に終了します。 |

同じ選択はサイレントでも指定できます：`gitpoulpi-<version>-...-setup.exe /CURRENTUSER` または `/ALLUSERS`。

チェックボックスは、スタートメニューのショートカット、デスクトップのショートカット、PATH への追加です。**コマンドのエイリアス**を尋ねるページがあり、既定は `gp`、好きな名前にでき、空にすればエイリアスなしになります（コマンドラインでは `/ALIAS=name` または `/ALIAS=`）。PowerShell では `gp` がすでに `Get-ItemProperty` に使われているため、別の名前にするか `gitpoulpi` を使ってください。

インストーラーは実行ファイル（更新中にファイルを入れ替える小さなプログラム `gitpoulpi-updater` を含む）と `.gitpoulpi-dist` マーカーをコピーします。マーカーには GitPoulpi のインストール方法（`inno-user` または `inno-machine`）が記録され、更新パネルが**インストール方法**として表示するのはこれです。セットアップをもう一度実行しても、選択内容（対象、ショートカット、エイリアス）は保持されます。

アンインストールは Windows の設定 > アプリから行います。`gitpoulpi uninstall`（エイリアス、PATH のエントリ、ショートカット）を実行してから、更新機能が残した `.old` ファイルを含めてファイルを削除します。設定とリポジトリは残ります。

### Linux と macOS：`install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

OS と CPU に合ったアーカイブを選び、`.sha256` を確認して、`~/.local/opt/gitpoulpi` にインストールし（チャンネルは `archive`：GitPoulpi はその場で自己更新します）、`gitpoulpi` と `gitpoulpi-mcp` を `~/.local/bin` にリンクします。root 権限は不要で、ホームの外には何も触れません。`--yes` を付けない場合は、エイリアス（既定は `gp`）、PATH、ショートカットについて尋ねます。

| フラグ | 効果 |
|---|---|
| `--prefix DIR` | プログラムファイルを `DIR`（絶対パス）にインストールします。 |
| `--version X.Y.Z` | 最新版ではなく、このリリースをインストールします。 |
| `--archive FILE` | ローカルのアーカイブを使います。`FILE.sha256` を隣に置く必要があります（`--no-verify` で省略可）。ネットワークは使いません。 |
| `--alias NAME` / `--no-alias` | `NAME` で GitPoulpi を起動できるようにします（`gitpoulpi alias create`）。または作成しません。 |
| `--no-path` | GitPoulpi を新しいシェルの PATH に追加しません（付けない場合は `gitpoulpi path add` を実行します）。 |
| `--desktop` | アプリケーションメニューとデスクトップのショートカットも作成します。 |
| `--yes` | 何も尋ねず、フラグと既定値を適用します（エイリアスは `gp`、PATH は追加する、ショートカットは作らない）。 |
| `--uninstall` | `gitpoulpi uninstall` を実行してから、リンクとフォルダーを削除します。 |

最新リリースの検索にはリポジトリが公開されている必要があります。非公開の場合は、ブラウザーでアーカイブをダウンロードして `--archive` を使ってください。

## パッケージマネージャーで

以下のどのチャンネルも、プログラムの隣に同じ `.gitpoulpi-dist` マーカーを、そのチャンネル固有の名前で書き込みます。GitPoulpi はこれを読んで、更新の担い手を知ります。担うのはパッケージマネージャーであり、GitPoulpi がその裏でファイルを入れ替えることはありません。更新パネル（タイトルバーのバージョンをクリック）は、パッケージマネージャー自身のコマンドを実行するか、コピーできるように表示します。[更新](updates.md)を参照してください。

| チャンネル | インストール | 備考 |
|---|---|---|
| **Chocolatey**（Windows） | `choco install gitpoulpi` | 全ユーザー向けにインストールされるため、更新には管理者の承認が必要です。 |
| **Scoop**（Windows） | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi` の後に `scoop install gitpoulpi` | ユーザーごと。管理者権限は不要です。 |
| **Homebrew**（macOS、Linux） | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | tap は `Poulpi-Lab/homebrew-gitpoulpi` です。 |
| **apt**（Debian、Ubuntu） | [apt リポジトリ](#aptdebian-と-ubuntu)を参照 | 署名付きのリポジトリ。すべてのリリースに `.deb` も添付されています。 |
| **MSI**（Windows、管理対象） | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | 管理者向け：全ユーザー、アプリ内更新なし。[エンタープライズ配布](enterprise-distribution.md#intune-またはグループポリシーでの-msi)を参照。 |
| **cargo** | `cargo binstall gitpoulpi-cli` | コンパイルせず、リリースアーカイブをダウンロードします。 |
| **cargo（crates.io から）** | `cargo install --locked gitpoulpi-cli` | コンパイルします。Rust ツールチェーンが必要です。`gitpoulpi` をインストールし、サブコマンドなしで実行すると端末 UI が開き、MCP（`gitpoulpi mcp`）も提供します。`gitpoulpi-tui` と `gitpoulpi-mcp` は、同じプログラムを単独のクレートにしたものです。 |
| **cargo（ソースから）** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | コンパイルします。Rust ツールチェーンが必要です。 |
| **Docker** | `docker pull poulpilab/gitpoulpi` | [Docker で実行する](docker.md)を参照。同じイメージは `ghcr.io/poulpi-lab/gitpoulpi` にもあります。 |

上の名前は、リリースの手順が公開先としているものです（[リリース](../../developer/releasing.md)を参照）。まだ公開されていないチャンネルは「見つかりません」と応答しますが、リリースアーカイブやインストーラーはつねに使えます。

パッケージマネージャー自身の PATH の扱いが適用されます。Chocolatey、Scoop、Homebrew はプログラムを自分で PATH に追加しますが、どれも `gp` エイリアスは作りません（`gitpoulpi alias create gp` を実行するか、設定 > ランチャーを使ってください）。

### apt（Debian と Ubuntu）

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

リポジトリは `amd64` と `arm64` に対応し、プロジェクト固有の鍵で署名されています。プログラムは `/usr/lib/gitpoulpi` に入り、`/usr/bin` にリンクが作られます。パッケージは `git` に依存し、`git-lfs` を推奨します。リポジトリを使わない場合は、リリースの `.deb` を `sudo apt install ./gitpoulpi_<version>_<arch>.deb` でインストールしてください。その場合、GitPoulpi は次の `.deb` を自分でダウンロードするよう案内します。`apt` コマンドを提示するのは、`/etc/apt/sources.list.d/gitpoulpi.*` にリポジトリがあるときだけだからです。

## ソースから

最近の安定版 Rust ツールチェーンが必要です（<https://rustup.rs>）。

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

実行ファイルは `target/release/` にあります。`release` プロファイルは
速くビルドできます。最小かつ最速の実行ファイルが必要な場合は `dist` プロファイルを使います。
ビルドに時間がかかります。

```
cargo build --profile dist
```

リポジトリのルートにある `build-opti` スクリプトは、3 つの実行ファイルに対してこのビルドを実行し、
そのサイズを表示します。

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd or PowerShell)
```

使う言語と機能だけをビルドするには、
[カスタムビルド](custom-builds.md)を参照してください。

## リリースアーカイブを自分でビルドする

```
cargo run -p xtask -- dist
```

これは `dist` プロファイルをビルドし、
`target/dist/gitpoulpi-<version>-<target>.zip`（Windows）または `.tar.gz`（その他の
システム）を書き出します。アーカイブには 3 つの実行ファイル、`INSTALL.txt`、
`README.md`、`LICENSE`、`CHANGELOG.md` が入ります。`.sha256` ファイルが
その隣に書き出されます。

## 更新

タイトルバーのアプリ名とリポジトリのパスの間に、実行中のバージョン（`v1.0.0`）が表示されます。新しいリリースが分かっているとき（または更新が進行中のとき）にこれをクリックすると、バージョン、GitPoulpi のインストール方法、新しいリリースへの対処を表示する更新パネルが開きます。それ以外のときにクリックすると、**リリースノート**タブが開きます。`Ctrl+N` の後に `u` を押すか、コマンドパレット（`Ctrl+P`）で**更新: 更新パネルを表示**を実行すると、常に更新パネルが開き、**ヘルプ: リリースノート**は常に変更履歴を開きます。

GitPoulpi は 1 日に 1 回、バックグラウンドで、リリースの公開先（既定では GitHub）に新しいリリースがあるかを確認します（`updates.mode`、[設定 > 同期](settings.md#同期)）。新しいリリースがあると、チップにアクセントカラーで `v0.2.0 が利用可能` と表示されます。

パネルの動作は GitPoulpi のインストール方法によって決まります。リリースアーカイブや Windows インストーラーは自分のファイルを置き換え、パッケージマネージャーは自分のコマンドを実行し、管理対象やコンテナーのインストールは、更新がどう届くかを案内します。`cargo build` で作ったビルドは更新されません。元に戻す操作、`auto` と `notify` の違い、管理者がロックできる内容を含む詳しい説明は、[更新](updates.md)にあります。

## 動作を確認する

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi
```

`--version` はバージョンと、ビルド元の commit を表示します。その後は
[はじめに](getting-started.md)に進んでください。

## アンインストール

`gitpoulpi uninstall`（どちらのインストーラーも実行します）は、GitPoulpi が作成したコマンドのエイリアス、PATH のエントリ、ショートカット、ファイルマネージャーのメニューのエントリ（Windows ではその App Paths キーも）を削除し、リリース版のインストールではプログラムファイルも削除します。`.gitpoulpi-dist` マーカーのないビルド（cargo のビルド）では実行を拒否し、`--keep-files` を付けるとプログラムファイルは残します。MCP エージェントからは使えません。インストーラーを使っていない場合は、3 つの実行ファイルを削除してください。GitPoulpi は次のファイルも保持しています。

| 内容 | 場所 |
|---|---|
| 設定（`config.toml`）とセッション | OS の設定フォルダー。`GITPOULPI_CONFIG_DIR` で変更できます。 |
| リポジトリごとの状態 | 既定では、各リポジトリの `.git` フォルダー内の `gitpoulpi/`（`session.per_repo_state_dir` で、作業ツリーの `.gitpoulpi/` またはユーザーデータフォルダーに移せます）。 |
| キャッシュ（アバター） | OS のキャッシュフォルダー。 |
| トークンと API キー | 認証情報ヘルパー。**設定 > SSH と認証情報 > すべてのユーザー名とパスワードを削除**で削除できます。 |

正確なパスは[設定リファレンス](config-reference.md)を参照してください。
