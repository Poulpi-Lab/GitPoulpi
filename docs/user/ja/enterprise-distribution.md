[English](../enterprise-distribution.md) · [Čeština](../cs/enterprise-distribution.md) · [Deutsch](../de/enterprise-distribution.md) · [Español](../es/enterprise-distribution.md) · [Français](../fr/enterprise-distribution.md) · [Italiano](../it/enterprise-distribution.md) · **日本語** · [한국어](../ko/enterprise-distribution.md) · [Nederlands](../nl/enterprise-distribution.md) · [Polski](../pl/enterprise-distribution.md) · [Português (Brasil)](../pt-BR/enterprise-distribution.md) · [Português (Portugal)](../pt-PT/enterprise-distribution.md) · [Русский](../ru/enterprise-distribution.md) · [Svenska](../sv/enterprise-distribution.md) · [Türkçe](../tr/enterprise-distribution.md) · [Українська](../uk/enterprise-distribution.md) · [简体中文](../zh-CN/enterprise-distribution.md)

# エンタープライズ配布

自社のサーバーから多数のコンピューターに GitPoulpi を展開する管理者向けのページです。社内のパッケージフィード、ミラー、マシン全体の更新ポリシー、既定の設定を扱います。先に[インストール](installation.md)と[更新](updates.md)をお読みください。

ここでのすべては次の 2 つの事実が土台になっています。

- すべてのインストールは、プログラムの隣にある `.gitpoulpi-dist` マーカー（`channel=...`）に**インストール方法**を記録します。GitPoulpi は、そのチャンネルが許す方法でしか自分を更新しません。パッケージマネージャーが管理するファイルは置き換えず、MSI には一切触れません。
- **ポリシー**（ファイル、Windows ではレジストリも）は管理者のものです。ユーザー自身の `config.toml` がポリシーを緩めることはできません。

## 非公開フィードでの Chocolatey

公開パッケージは GitHub からインストーラーをダウンロードします。閉じたネットワークでは、これを内部化してください。

1. パッケージを入手します（ライセンス版の Chocolatey ツールで `choco download gitpoulpi --internalize`、または `.nupkg` をダウンロードして展開）。
2. `tools\chocolateyinstall.ps1` の `url64bit` を、自社に置いた `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` のコピーに向け、`checksum64` はリリースの `.sha256` ファイルの先頭フィールド（またはコピーのチェックサム）と同じに保ちます。
3. `choco pack` を実行し、`.nupkg` を自社のフィード（Nexus、Artifactory、ProGet、ファイル共有）に push します。

パッケージパラメーターを付けてインストールします。

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| パラメーター | 効果 |
|---|---|
| `/Channel:choco-internal` | マーカーに `channel=choco-internal` を記録します。これでフィードが Chocolatey のコミュニティフィードではなく自社のものであることが GitPoulpi に伝わります。 |
| `/Feed:<url>` | フィードのアドレスを記録します（マーカーの `feed=`）。GitPoulpi はここで新しいバージョンを確認し、`choco upgrade gitpoulpi -y --source=<feed>` を実行します。 |

パラメーターなしの場合、チャンネルは `choco` で、確認先は Chocolatey のコミュニティフィードです。パッケージは全ユーザー向けに Program Files へインストールされ、`gp` エイリアスは作られません。

GitPoulpi の更新は、展開した内容に手を付けないよう `--params` なしで `choco upgrade` を実行します。そのため Chocolatey は、最初のインストールのパッケージパラメーターを記憶している必要があります（`useRememberedArgumentsForUpgrades`。Chocolatey 2.0 以降は既定でオン、古いバージョンでは `choco feature enable -n useRememberedArgumentsForUpgrades`）。そうでないと、アップグレードで通常の `choco` チャンネルが記録されてしまいます。ポリシーの `check_url`（後述）を使うと、マーカーの内容にかかわらず確認とアップグレードが自社のフィードを向き続け、マーカーの feed より優先されます。

**パッケージは `policy.toml` も `config.toml` も書き込みません。** 展開した内容は、インストール時も、毎回のアップグレードでも、そのまま残ります。[既定の config.toml を配布する](#既定の-configtoml-を配布する)を参照してください。

## 内部ソースでの winget

winget は、パッケージの入手元のソースを使って `winget upgrade --id GitPoulpi.GitPoulpi --exact` で GitPoulpi を更新します。内部ソースを使うには、Windows Package Manager の REST ソース（winget REST API の任意の実装）を用意し、リリースの 3 つのマニフェスト（`GitPoulpi.GitPoulpi.yaml`、`...installer.yaml`、`...locale.en-US.yaml`）を、`InstallerUrl` を自社のセットアッププログラムのコピーに向け、対応する `InstallerSha256` を付けて公開します。その後：

```
winget source add --name corp --arg https://winget.example/api --type Microsoft.Rest
winget install --id GitPoulpi.GitPoulpi --source corp
```

マニフェストのインストーラースイッチはそのまま残してください。`/CHANNEL=winget` でマーカーに `winget` と書かれ、`/ALIAS=` で `gp` エイリアスが省かれます。winget は何が新しいかを GitPoulpi に伝えないため、確認は引き続き GitHub にバージョンを問い合わせます。GitHub に接続できないネットワークでは、ポリシーの `check = "manifest"` で自社の `latest.json` に向けてください（[latest.json と download_base（リリースのミラー）](#latestjson-と-download_baseリリースのミラー)を参照）。

## プライベートバケットでの Scoop

公開バケットの `gitpoulpi.json`（マニフェスト）を、自社でホストする Git リポジトリの `bucket/` に置き、`url` をリリース zip の自社コピーに変更し、`hash` をその sha256 に設定します（コピーがリリースの名前に従うなら `autoupdate` は残せます）。その後：

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

マニフェストの `post_install` が、マーカーを `channel=scoop` で書き直します。以後、GitPoulpi は自分で `scoop update gitpoulpi` を実行し、Scoop の裏でファイルを入れ替えることはありません。Scoop はユーザーごとにインストールされ、管理者権限は不要です。

## ミラーでの apt

リポジトリは、HTTPS で提供される単純なディレクトリツリー（`dists/`、`pool/`、署名鍵の `gitpoulpi-archive-keyring.gpg` と `.asc`、サンプルの `gitpoulpi.sources`）です。`wget --mirror`、`rsync`、または自社のアーティファクト管理ツールでミラーし、社内で提供したうえで、各コンピューターに鍵とソースをインストールします。

```
sudo install -d /etc/apt/keyrings
sudo install -m 644 gitpoulpi-archive-keyring.gpg /etc/apt/keyrings/gitpoulpi.gpg
```

`/etc/apt/sources.list.d/gitpoulpi.sources`：

```
Types: deb
URIs: https://mirror.example/gitpoulpi/apt
Suites: stable
Components: main
Architectures: amd64 arm64
Signed-By: /etc/apt/keyrings/gitpoulpi.gpg
```

ファイル名は `gitpoulpi.sources`（または `gitpoulpi.list`）のままにしてください。GitPoulpi は `/etc/apt/sources.list.d/gitpoulpi.*` を見つけたときにだけ `apt` コマンドを表示します。プロジェクトの鍵を使ってください。ミラーはその鍵による署名のままで、`Signed-By` がこのリポジトリに限ってその 1 つの鍵を固定します。リポジトリは各パッケージの直近 5 バージョンを保持します。

**オフラインの `.deb`：**すべてのリリースに `gitpoulpi_<version>_<arch>.deb` が添付されています。コピーして `sudo apt install ./gitpoulpi_<version>_<arch>.deb` を実行してください。マーカーは `channel=apt` になります。ソースファイルがない場合、GitPoulpi は `apt` コマンドを表示する代わりに、次の `.deb` をダウンロードするようユーザーに伝えます。

## Intune またはグループポリシーでの MSI

`gitpoulpi-<version>-x86_64-pc-windows-msvc.msi` は全ユーザー向けにインストールされ、管理されることを前提にしています。GitPoulpi のパネルには「管理者が管理しています」（管理者の `message` 付き）としか表示されず、MSI インストールを更新することはありません。アップグレードは新しい MSI を展開して行います。メジャーアップグレードなので、古いバージョンは置き換えられます。

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="Updates are deployed by IT."
```

| プロパティ | 効果 |
|---|---|
| `INSTALLFOLDER` | インストール先フォルダー（既定は `Program Files\GitPoulpi`）。 |
| `ADDTOPATH` | `1`（既定）でフォルダーをマシンの PATH に追加、`0` で追加しません。 |
| `UPDATES_MODE` | ポリシーの `mode`：`off`、`notify`、`auto`。 |
| `UPDATES_PIN` | ポリシーの `pin`。たとえば `0.2` や `0.2.4`。 |
| `UPDATES_CHECK` | ポリシーの `check`：`github`、`nuget`、`manifest`、`none`。 |
| `UPDATES_CHECK_URL` | ポリシーの `check_url`。 |
| `UPDATES_MESSAGE` | ポリシーの `message`。更新パネルに表示されます。 |

プロパティは、設定したときだけ値を `HKLM\SOFTWARE\Policies\GitPoulpi\Updates` に書き込むため、未設定のものはレジストリ（とポリシーファイル）に手を付けません。**Intune：**上記のコマンドで、MSI を基幹業務アプリまたは Win32 アプリとして展開します。**グループポリシー：**ソフトウェアのインストールではプロパティを渡せないため、トランスフォーム（`.mst`）を使うか、グループポリシーの基本設定でレジストリ値を設定してください（後述）。

## 社内 tap での Homebrew

公開 tap の `gitpoulpi.rb` を、自社でホストする Git リポジトリの `Formula/` にコピーし、`url` の行をリリースアーカイブの自社コピーに変更し、各 `sha256`（各アーカイブの隣にある `.sha256` ファイルの先頭フィールド）を更新します。その後：

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

formula は `channel=homebrew` のマーカーをインストールします。GitPoulpi は自分で `brew upgrade gitpoulpi` を実行し、それが自社の tap を読みます。

## Docker レジストリのミラー

イメージ名は `poulpilab/gitpoulpi`（と `ghcr.io/poulpi-lab/gitpoulpi`）です。パネルは常にこの名前を表示するため、ミラーは透過的である必要があります。

- **プルスルーキャッシュ：**Docker Hub のプロキシとしてレジストリを動かし（`registry` イメージで `proxy.remoteurl: https://registry-1.docker.io`）、Docker デーモンの `daemon.json` に `"registry-mirrors": ["https://registry.example"]` を設定します。以後 `docker pull poulpilab/gitpoulpi` は自社のキャッシュを経由します。
- **コピー：**`docker pull poulpilab/gitpoulpi:<v>`、`docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`、`docker push registry.example/tools/gitpoulpi:<v>` を実行し、その名前で実行します。パネルには公開名が表示されたままですが、バージョンの目安としては問題ありません。

実行方法は [Docker で実行する](docker.md)を参照してください。

## ポリシーファイルとレジストリ

ポリシーは、コンピューター上で更新がどう動作するかを決め、ユーザーは変更できません。次の場所から読み込まれます。

| OS | ポリシーファイル |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows（追加） | レジストリキー `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`。下の各キーに対して同じ名前の `REG_SZ` 値を 1 つずつ（グループポリシー、Intune、MSI プロパティ用） |

ファイルにはテーブルが 1 つあります。

```toml
[updates]
mode = "notify"
release_channel = "stable"
pin = "1.4"
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
repo = "Poulpi-Lab/GitPoulpi"
download_base = "https://mirror.example/gitpoulpi/"
allow_self_update = false
message = "Updates are deployed by IT."
```

| キー | 値 | 効果 |
|---|---|---|
| `mode` | `off`、`notify`、`auto` | 上限です：`off` < `notify` < `auto`。ユーザーはより低い値を選べますが、より高い値は選べません。`off` は確認を隠し、リクエストを一切送りません。 |
| `release_channel` | `stable`、`prerelease` | 更新チャンネルを強制します。 |
| `pin` | `1.4` または `1.4.2` | 提示される最高のバージョン：`1.4` は任意の `1.4.x` を許可し、`1.4.2` は `1.4.2` までを許可します。 |
| `check` | `github`、`nuget`、`manifest`、`none` | 「最新リリースは何か」を問い合わせる先。インストールごとの既定は GitHub、Chocolatey のチャンネルでは Chocolatey フィードです。`none` は確認しません。 |
| `check_url` | URL | `nuget` では：フィード。`manifest` では：`latest.json` のアドレス。`github` では：GitHub Enterprise Server 用の GitHub API ベース（`repo` を追加）。 |
| `repo` | `owner/name` | `check = "github"` で確認するリポジトリ。 |
| `download_base` | URL | アーカイブとインストーラーのチャンネル向けの、リリースファイル（同じファイル名）のミラー。GitHub に加えて、このホストからのダウンロードも許可されます。 |
| `allow_self_update` | `true`、`false` | `false` にすると、「GitPoulpi のファイルを置き換える」と「パッケージマネージャーを実行する」のすべての操作が、注記またはコピー用のコマンドになります。既定は `true`。 |
| `message` | テキスト | 更新パネルに表示されます（「管理者からのメッセージ：...」）。 |

**優先順位**（強いものから）：レジストリ、ポリシーファイル、ユーザーの `config.toml` の `[updates]`、マーカーの `feed=`、組み込みの既定値。ポリシーが設定したキーは、設定画面で**管理者が設定しています**と表示され、そこでは編集できません。

**安全側に倒れる動作：**読み取れないポリシーファイルやレジストリ値（TOML の構文エラー、`mode = "sometimes"`、バージョンではない `pin`）は、`mode = notify` かつ `allow_self_update = false` として扱われ、アプリケーションログに 1 行が記録されます。タイプミスで、展開先のコンピューター群が勝手に更新を始めることはありません。未知のキーは警告付きで無視されます。Windows で管理者権限を持つステップは、マシンのポリシーだけを読み、ユーザーの設定は読みません。

### 例

更新をオフにする（ロック済みイメージ、エアギャップのネットワーク）：

```toml
[updates]
mode = "off"
```

展開先を 1.4 系に固定し、通知のみにして、問い合わせ先を示す：

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "Updates go through the software center. Ask the service desk."
```

社内の Chocolatey（NuGet）フィードを確認する：

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

Windows でレジストリを通じて同様のポリシーを設定する（PowerShell）：

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json と download_base（リリースのミラー）

リリースのファイル（アーカイブ、セットアップ、`.sha256` ファイル、`latest.json`、**すべての `.minisig` 署名ファイル**、`security.json`）を、名前を変えずに社内 Web サーバーの 1 つのフォルダーにコピーします。`latest.json` はすべてのリリースに添付されています。

```json
{
  "version": "1.4.2",
  "notes": "Markdown release notes",
  "assets": { "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "<sha256>" }
}
```

ポリシーは次のようになります。

```toml
[updates]
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
download_base = "https://mirror.example/gitpoulpi/"
```

アーカイブとインストーラーのチャンネルは、マニフェストを確認し、`download_base` からダウンロードします。検証は GitHub からの場合と同じです。ミラーはファイルをコピーするだけでよく、API は不要です。

署名はファイルとともに運ばれるため、ミラーが署名を弱めることはできません。

- `latest.json` が信頼されるのは、`latest.json.minisig` がその隣にあり、GitPoulpi に組み込まれたプロジェクトの鍵で検証できる場合だけです。署名のないマニフェストや、署名後に編集されたマニフェストは拒否されます（ダウンロードは、その中のハッシュと照合されます）。
- 各アーカイブは、同じホストの `<archive>.minisig` と一緒にダウンロードされ、その署名が検証でき、かつこのファイルとこのバージョンを指している場合にだけインストールされます。アーカイブはコピーしたが `.minisig` をコピーしていないミラーでは、更新が「署名なし」で失敗し、何も変更されません。
- `security.json`（セキュリティ通知。[更新](updates.md#セキュリティ通知)を参照）と `security.json.minisig` は、マニフェストの隣で探されます。これらをコピーしていないミラーでは、通知が表示されないだけです。

ファイルは変更せずにコピーしてください。署名は正確なバイト列を対象にしているためです。セキュリティ通知は、新しいビルドなしで最新リリースに対して再公開されるので、ミラーを更新するたびに、リリースのページから `latest.json`、`security.json` とそれらの `.minisig` ファイルを更新してください。

## 既定の config.toml を配布する

GitPoulpi にはマシン全体の `config.toml` はありません。設定は各ユーザーのフォルダー（`%APPDATA%\GitPoulpi\config.toml`、`~/.config/gitpoulpi/config.toml`、`~/Library/Application Support/GitPoulpi/config.toml`）にあります。新しいユーザーに既定値を渡すには、ユーザーが設定を持っていないときに、必要なキーだけを入れた最小限の `config.toml`（[設定リファレンス](config-reference.md)を参照）をそのフォルダーにコピーします。Windows ではログオンスクリプトまたは Intune のスクリプト、Linux では `/etc/skel/.config/gitpoulpi/`、macOS では初回ログイン時のスクリプトを使います。GitPoulpi のパッケージが設定を上書きすることは決してないため、管理は管理者に任されています。必ず守らせたい更新のルールはここに書かず、ユーザーが上書きできないポリシーを使ってください。ポータブルな設定やジョブごとの設定は、`GITPOULPI_CONFIG_DIR` でも選べます。
