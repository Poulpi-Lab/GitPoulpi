[English](../docker.md) · [Čeština](../cs/docker.md) · [Deutsch](../de/docker.md) · [Español](../es/docker.md) · [Français](../fr/docker.md) · [Italiano](../it/docker.md) · **日本語** · [한국어](../ko/docker.md) · [Nederlands](../nl/docker.md) · [Polski](../pl/docker.md) · [Português (Brasil)](../pt-BR/docker.md) · [Português (Portugal)](../pt-PT/docker.md) · [Русский](../ru/docker.md) · [Svenska](../sv/docker.md) · [Türkçe](../tr/docker.md) · [Українська](../uk/docker.md) · [简体中文](../zh-CN/docker.md)

# Docker で実行する

`poulpilab/gitpoulpi` イメージには、公式リリースの実行ファイル（`gitpoulpi` と、中継プログラムの `gitpoulpi-tui`、`gitpoulpi-mcp`）が `debian:bookworm-slim` 上に入っており、`git`、`git-lfs`、`openssh-client`、`less`、`tini` も含まれます。`linux/amd64` と `linux/arm64` で動作します。Docker Hub で公開されています。

## UI を起動する

```
docker run -it --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD":/work \
  -v gitpoulpi-config:/config \
  poulpilab/gitpoulpi
```

- `-it` で UI にターミナルを与えます。これがないと、プログラムは描画する先がありません。
- `-v "$PWD":/work` は、現在のフォルダーを作業ディレクトリ（`/work`）としてマウントします。そこからリポジトリを開くか、`/work` の下に複数のリポジトリをマウントしてください。
- `-v gitpoulpi-config:/config` は名前付きボリュームで、実行をまたいで設定とセッションを保持します。
- `--user "$(id -u):$(id -g)"` はあなた自身として実行するので、GitPoulpi がリポジトリ内に作るファイルの所有者は `root` ではなくホストのあなたのままになります。イメージが書き込むフォルダーはすべて誰でも書き込めるため、どのユーザー ID でも動作します。

## ボリュームとパス

| パス | 内容 |
|---|---|
| `/work` | 作業ディレクトリ。リポジトリをここにバインドマウントします。 |
| `/config` | 設定（`config.toml`）、セッション、およびサブフォルダーとしてデータとキャッシュ（`GITPOULPI_CONFIG_DIR=/config`）。保持するには名前付きボリュームかバインドマウントを使います。 |
| `/data`、`/cache` | XDG 変数に従うツール向けに、これらもボリュームとして宣言されています（`XDG_DATA_HOME=/data`、`XDG_CACHE_HOME=/cache`）。 |
| `/home/gitpoulpi` | `HOME`。ボリュームではなく、コンテナーごとに新しくなります。 |

`/config` のボリュームがないと、`--rm` で実行するたびに既定の設定で始まります。自分の `config.toml` で始めるには、マウントします：`-v "$PWD/config.toml":/config/config.toml:ro`（読み取り専用なので、設定は保存できません）。

## 別のユーザーが所有するリポジトリ

git は、所有者が実行ユーザーと異なるリポジトリを拒否します（「dubious ownership」）。上のように `--user` を付ければ所有者は一致します。一致しない場合（別アカウントからのバインドマウント、Docker Desktop の共有など）は、次のようにしてすべてのディレクトリを信頼します。

```
docker run -it --rm -e GITPOULPI_SAFE_DIRECTORY='*' -v "$PWD":/work poulpilab/gitpoulpi
```

エントリポイントは、UI を起動する前に `git config --global --add safe.directory '*'` を実行します。コンテナー全体で git の保護を外すことになるため、信頼できるフォルダーでのみ使ってください。

## SSH

鍵をコンテナーにコピーする代わりに、SSH エージェントを転送します。

Linux：

```
docker run -it --rm \
  -v "$SSH_AUTH_SOCK":/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Docker Desktop（macOS、Windows）は、VM 内の固定パスにホストのエージェントを公開します。

```
docker run -it --rm \
  -v /run/host-services/ssh-auth.sock:/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

`--user` を付ける場合、ソケットはあなたのユーザー ID から読み取れる必要があります。自分のものであれば読み取れます。コンテナーには `known_hosts` がありません。ホストへの最初の接続でそのホスト鍵の承認を求められるか、`-v ~/.ssh/known_hosts:/home/gitpoulpi/.ssh/known_hosts:ro` で自分のファイルをマウントしてください。

## Git の ID と認証情報

- **ID と設定：**Git の設定を読み取り専用でマウントします：`-v ~/.gitconfig:/home/gitpoulpi/.gitconfig:ro`。または、`/config` にある GitPoulpi のプロファイル（設定 > プロファイル）でコミットの ID を設定します。
- **HTTPS の認証情報：**イメージにはキーチェーンのような認証情報ヘルパーがありません。認証情報ストアをマウントする（`-v ~/.git-credentials:/home/gitpoulpi/.git-credentials:ro`、マウントした `.gitconfig` に `credential.helper = store`）か、エージェントを転送した SSH のリモートを使ってください。
- **ホスティングのトークン**（プルリクエスト、issue）：認証情報ヘルパーがない場合、GitPoulpi はトークンをセッションの間だけメモリに保持します。継続して使うには、環境変数で `-e GITPOULPI_TOKEN_<ID>=...` と渡します。`<ID>` は連携の id を大文字にし、英数字以外を `_` にしたものです（[ホスティングと issue トラッカー](hosting-integration.md)を参照）。連携自体は、`/config` の `config.toml` にあります。

## ターミナル、色、マウス

色とマウスのサポートは、ターミナルエミュレーターによります。Docker は `-t` から `TERM` を引き継ぎます。色やトゥルーカラーのテーマがおかしいときは `-e TERM -e COLORTERM` を追加してください。たとえば `-e COLORTERM=truecolor` です。マウスのクリック、ドラッグ、ホイールは、通常どおりターミナル経由で届きます。commit グラフは文字で描かれます。画像（kitty、iTerm2、sixel）は、すべてのターミナルやマルチプレクサ経由で転送されるわけではありません。[ターミナルの選び方](terminals.md)を参照してください。ターミナルウィンドウのサイズを変えると UI のサイズも変わります：コンテナーはサイズ変更を受け取ります。

## CLI と MCP

エントリポイントは UI を起動します。ほかのプログラムは、エントリポイントを上書きすれば使えます。

```
# the scriptable CLI (add --json for scripts)
docker run --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi status

# the MCP server, over standard input and output, for an agent that launches it
docker run -i --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi mcp serve
```

`docker run` ごとに独立したコンテナーで、それぞれが自分の GitPoulpi バックグラウンドデーモンを持つため、CLI で行った変更は、別のコンテナーで動いている UI にはリアルタイムで反映されません。1 つの UI を CLI から操作するには、同じコンテナーで両方を動かします：UI を `--name gp` で起動し、`docker exec gp gitpoulpi ...` を使います。エージェントの場合は、`docker run -i --rm --entrypoint gitpoulpi ... mcp serve` を MCP サーバーのコマンドとして登録してください。[MCP とエージェント](mcp-and-agents.md)を参照してください。マウントした `config.toml` の MCP の権限スイッチは、通常どおり適用されます。

## タグと更新

| タグ | 意味 |
|---|---|
| `X.Y.Z` | 特定の 1 つのリリース。 |
| `X.Y` | そのマイナーバージョンの最新パッチ（安定版リリースのみ）。 |
| `latest` | 最新の安定版リリース。 |

プレリリースには、正確な `X.Y.Z-...` のタグだけが付きます。

イメージは置き換えるもので、その場で更新することはありません。コンテナー内の GitPoulpi はそれを知っています（インストール記録に「Docker イメージ」とあります）。[更新パネル](updates.md)にはバージョンとコマンドが表示され、何も置き換えません。更新するには：

```
docker pull poulpilab/gitpoulpi
```

その後、新しいコンテナーを起動します。設定は `/config` ボリュームに残ります。戻すには、古いタグを実行します：`docker pull poulpilab/gitpoulpi:<previous>`。スクリプトでバージョンを固定するには、`latest` の代わりに正確なタグを使ってください。組織は自社のレジストリにイメージをミラーできます。[エンタープライズ配布](enterprise-distribution.md#docker-レジストリのミラー)を参照してください。
