[English](../lfs.md) · [Čeština](../cs/lfs.md) · [Deutsch](../de/lfs.md) · [Español](../es/lfs.md) · [Français](../fr/lfs.md) · [Italiano](../it/lfs.md) · **日本語** · [한국어](../ko/lfs.md) · [Nederlands](../nl/lfs.md) · [Polski](../pl/lfs.md) · [Português (Brasil)](../pt-BR/lfs.md) · [Português (Portugal)](../pt-PT/lfs.md) · [Русский](../ru/lfs.md) · [Svenska](../sv/lfs.md) · [Türkçe](../tr/lfs.md) · [Українська](../uk/lfs.md) · [简体中文](../zh-CN/lfs.md)

# Git LFS

Git LFS（Large File Storage）は、大きなファイル（デザインファイル、動画、データセット）をリポジトリの履歴の外に保ちます。git は 3 行のテキストの*ポインター*を保存し、実際の内容は LFS サーバーと、`.git/lfs` 配下のローカルキャッシュに置かれます。

GitPoulpi は、そのためのアカウントを一切必要としません。LFS は、リモートがすでに使っている認証情報（git の認証情報ヘルパー、または ssh-agent）を使います。

## `git-lfs` プログラムが必要なものと不要なもの

GitPoulpi は、ファイルを変換または移動するすべての処理（install、track、fetch、pull、push、ロック）に、標準の `git-lfs` プログラムを実行します。

LFS の*状態*を読み取るのにプログラムは不要です。リポジトリが LFS を使っているか、どのファイルがポインターか、そのサイズ、実際のオブジェクトがすでにローカルに保存されているかどうかが分かります。

| | `git-lfs` が必要か |
|---|---|
| `gitpoulpi lfs status`（プログラムがないことを、失敗ではなく報告します） | 不要 |
| `gitpoulpi lfs files` | 不要 |
| install / track / untrack / prune | 必要 |
| fetch / pull / push | 必要 |
| ロック | 必要 |

## `git-lfs` のインストール

- **Windows**：Git for Windows に同梱されています。または `winget install GitHub.GitLFS`。
- **Debian / Ubuntu**：`apt install git-lfs`。
- **macOS**：`brew install git-lfs`。
- それ以外：https://git-lfs.com

バージョン 3.0 以降が必要です。インストール後、GitPoulpi は 1 分以内に（`lfs status` では即座に）気付きます。

## `git-lfs` がない場合

GitPoulpi は、そのことでクラッシュしたり終了したりすることはありません。

- `gitpoulpi lfs status` は成功し、インストールのリンク付きで `git-lfs: not installed` と表示します。
- プログラムが必要な操作は、インストールのリンク（`https://git-lfs.com`）で終わる明確なメッセージとゼロ以外の終了ステータスで失敗します。MCP では、ツールがそのメッセージを返します。
- LFS を使うリポジトリも開けます。大きなファイルは、プログラムがインストールされるまでポインターとして表示されるだけです。

## LFS ファイルを見る

```
gitpoulpi lfs status            # program, patterns, filters, hook
gitpoulpi lfs files             # pointer files in the index, with size and local/not downloaded
gitpoulpi lfs files --at v1.0   # ... as of a revision
gitpoulpi show big.bin --at HEAD  # reports "Git LFS object, 12.3 MB, sha256:..." instead of pointer text
```

## リポジトリのセットアップ：install、track、untrack

```
gitpoulpi lfs install                 # filters + hooks for THIS repository (git lfs install --local)
gitpoulpi lfs install --user          # into your global git config instead (never done silently)
gitpoulpi lfs track '*.psd'           # route *.psd through LFS
gitpoulpi lfs track '*.psd' --lockable   # ... and make them read-only until locked
gitpoulpi lfs track '*.bin' --renormalize  # ... and re-stage already-committed *.bin as pointers
gitpoulpi lfs untrack '*.psd'
gitpoulpi lfs prune                   # delete old, already-pushed objects from the local store
```

- `install` は既定でリポジトリ自身の `.git/config` とフックに書き込むので、`--user` を渡さない限り、グローバルの git 設定には触れません。
- `track` と `untrack` は `.gitattributes` を編集し、**ステージしない**まま残します。通常の作業ツリーの変更として表示されるので、自分で commit します。track した*後に*追加されたファイルは、次の commit でポインターとして保存されます。
- すでに通常の内容として**commit 済み**のファイルは、履歴上は通常のままです。`--renormalize` はそれらを再ステージし、次の commit でポインターに置き換えます。過去のすべての commit でも移したい場合は、履歴の書き換えルール *ファイルを LFS に移行* を使ってください（[履歴の書き換え](history-rewrite.md#git-lfs-への移行と復帰)を参照）。これは `git lfs migrate` ではなく、GitPoulpi 独自のものです。
- パターンはシェルに展開されないよう、引用符で囲んでください。
- MCP では、これらは通常の変更操作（`lfs_install`、`lfs_track`、`lfs_untrack`、`lfs_prune`）で、`config.toml` の `[mcp]` による制限に従います。

## オブジェクトの移動：fetch、pull、push

```
gitpoulpi lfs fetch [--remote origin] [--all]   # download objects into the local store; working tree untouched
gitpoulpi lfs pull  [--remote origin]           # fetch, then replace pointer files with the real content
gitpoulpi lfs push  [--remote origin] [--all]   # upload the current branch's objects (or every ref with --all)
```

- リモートの既定は、`fetch` や `push` と同様に、リポジトリの既定のリモートです。
- 認証情報はリモートの通常のもの（認証情報ヘルパー、ssh-agent）です。GitPoulpi が独自のアカウントを求めることはありません。
- `pull` は、リポジトリの LFS フィルターがない場合にインストールします。フィルターがないと、git-lfs はオブジェクトをダウンロードしてもポインターの置き換えをスキップし、後の checkout でポインターが戻ってしまいます。
- MCP は `lfs push` を `push` と同様に扱います。破壊的なツールです（リモートを変更するため）。
- ウィンドウには、転送の進行状況（ファイル数とバイト数）がステータスバーに表示されます。CLI は結果を待ちます。

### 内容の checkout：`lfs checkout`

```
gitpoulpi lfs checkout [<path>...]   # offline: write real content over the pointer files, from the local store
```

ダウンロードは決して行いません（それは `lfs pull` です）。ローカルのストアにないオブジェクトはポインターのままです。`pull` と同様に、LFS フィルターが設定されていない場合は、先にリポジトリの LFS フィルターをインストールします。すべての LFS 操作と同様に、元に戻せません。[元に戻す・やり直す](undo-redo.md)を参照してください。MCP では `lfs_checkout`（書き込みで、破壊的ではありません）です。

### ツールバーの LFS ボタンと pull / push

ツールバーの **Pop** の後に **`LFS ▾`** ボタンがあります（`git.lfs_enabled` がオフの場合、および LFS が初期化されておらず `.gitattributes` でも使われていないリポジトリでは非表示。[設定 > LFS](settings.md#リポジトリ別lfs)を参照）。クリックするとドロップダウンが開きます：**すべての LFS ファイルを fetch**（`lfs fetch --all`）、**すべての LFS ファイルをチェックアウト**（`lfs checkout`）、**すべての LFS ファイルを pull**（`lfs pull`）、**すべての LFS ファイルを push**（`lfs push --all`、すべての参照のオブジェクト）、**ローカルの LFS ストアを整理**（`lfs prune`）。各行はパレットのコマンドでもあります（`LFS: すべてのオブジェクトをFetch`、`LFS: すべてのファイルをチェックアウト`、`LFS: オブジェクトをPull`、`LFS: すべてのオブジェクトをPush`、`LFS: ローカルのオブジェクトをプルーン`）。`LFS: メニューを開く`でドロップダウンが開きます。それぞれに `Ctrl+G` のコードがあります：メニューは `l`、続いて各行に `Shift+F`、`Shift+K`、`Shift+L`、`Shift+J`、`Shift+V`。*設定 > キーボードとマウス*で変更できます。`git-lfs` がない場合、すべての行はインストールのダイアログに進みます。

![ツールバーの LFS ドロップダウン：すべての LFS ファイルの fetch、checkout、pull、push と、ローカルストアの整理](../images/lfs-menu.png)

**pull と push は LFS を自動的に処理します。** LFS を使うリポジトリで `git-lfs` がインストールされている場合、通常の `pull` はまずリポジトリの LFS フィルターがインストールされているかを確認し、通常の `push` はその `pre-push` フックを確認します。どちらかがない場合、GitPoulpi は転送の前に `git lfs install --local`（このリポジトリのみ）を実行し、ステータスバーに「LFS フィルターをインストールしました」と表示します。フックがないと、push は LFS オブジェクトをアップロードしません。`git.lfs_autofix_filters = false`（*設定 > Git とエンコード > pull・push の前に LFS フィルターをインストール*）でオフにできます。通常の `push` が引き続きアップロードするのは push したブランチのオブジェクトだけで、すべての参照のものではありません。prune が暗黙に行われることはありません。

## ファイルのロック

merge できないファイル（Photoshop のファイル、レベル、バイナリアセット）があります。Git LFS の*ロック*を使うと、1 人が「これを編集中です」と宣言できます。ロックが解除されるまで、ほかの誰もそれをロックしたり、変更を push したりできません。

```
gitpoulpi lfs locks  [--remote origin]              # who holds which file (yours are marked)
gitpoulpi lfs lock   assets/hero.psd [--remote origin]
gitpoulpi lfs unlock assets/hero.psd [--remote origin] [--force]
```

- ロックは **LFS サーバー**の機能です（GitHub、GitLab、Gitea にあります）。リモート自身の認証情報で到達します。GitPoulpi に独自のアカウントはありません。
- ロックのないサーバーでは、`lock` はそのことを伝えるメッセージで失敗します（型付きの `LfsLockingUnsupported`）。そのようなサーバーでの `locks` は何も一覧しません。これは git-lfs 自身の報告と同じです。
- パターンを `--lockable` で track する（`lfs track '*.psd' --lockable`）と、それらのファイルはロックするまで読み取り専用になります。
- 他の人のロックの `unlock` は拒否されます。`--force` はそれを破るので、MCP は `lfs_unlock` を破壊的なツールとして扱います。`lfs_locks` は読み取り専用です。

## LFS リポジトリのクローン

`gitpoulpi clone <url>`（と TUI のクローンのダイアログ）は、LFS フィルターを実行*せずに*リポジトリを checkout し、その後、リポジトリが LFS を使っていて `git-lfs` がインストールされている場合は、`git lfs pull` でオブジェクトをダウンロードして（フィルターが設定されていなければそのリポジトリ用にインストールして）、独立したフェーズ、*LFS オブジェクトをダウンロードしています*として表示します。`--no-lfs` はこのステップを省略し、ポインターのままにします。

クローンは**LFS が原因で失敗することはありません**。`git-lfs` がない場合やダウンロードが失敗した場合でも、クローンは成功し、大きなファイルはポインターのままです。リポジトリの LFS の状態がその理由を伝え、問題が解決したら `gitpoulpi lfs pull` で仕上げられます。

## ターミナル UI で

- **`LFS` tag。** Git LFS で保存されたファイルには、ステータスペインの未ステージとステージ済みの一覧、および commit の変更ファイル一覧で、名前の後ろに小さな `LFS` tag が付きます。**LFS のみ**（ステータスペインまたは Log にフォーカスがあるときの `o`、ファイル一覧のツールバーのチェックボックス、またはパレットの **ファイル: LFS のみ表示を切り替え**）は、それらの一覧を LFS ファイルだけに絞り込みます。チェックボックスはフィルター行の右端にあります（ステータスペインではツールバー、狭いときはそのオーバーフローメニュー）。commit のファイル一覧では、狭いペインでも消えずに `[ ] LFS` に縮みます。表示されるのは、一覧にあるファイルに LFS ファイルがあるときだけです。ステータスペインと commit の変更ファイルでは、ファイルが LFS と見なされるのは、そのパスがインデックス内でポインターである場合です（更新ごとに `git` の呼び出しを 1 回にまとめ、`filter=lfs` 属性のないリポジトリでは何もしません）。Log で**すべてのファイルを表示**をオンにすると、選択中のリビジョンのツリー全体が一覧になり、LFS ファイルはそのリビジョン自身の blob から読み取られたもの（リビジョンごとにキャッシュし、描画スレッドの外で処理し、`git-lfs` プログラムは不要）になるので、そこでは LFS でもインデックスでは LFS でないファイルに tag が付き、フィルターでも残ります。
- **diff カード。** LFS ファイルの diff を開くと、ポインターのテキストではなくカードが表示されます：`Git LFS オブジェクト`、`1.2 MB -> 1.4 MB`、新旧の短い oid、そして新しいオブジェクトがローカルに保存されているか `未ダウンロード`か。`r` で生のポインターの diff、もう一度 `r` で戻ります。LFS ファイルには、ファイル単位のステージのみが適用されます（ステージするハンクも行もありません）。カードの下の **既定のプログラムで開く** と **開く場所**（`o`、`Shift+O`、またはパレット）は、ポインターではなく実際のファイルを開きます。[開く場所](open-in.md#git-lfs-ファイル)を参照してください。
- LFS ファイルの**ファイルビュー**には、`Git LFS オブジェクト, 12.3 MB, sha256:ab12cd34...` と、内容の入手方法が表示されます。
- **バナー。** `git-lfs` がない、または 3.0 より古い状態で LFS を使うリポジトリでは、ステータスペインのバナーがそう伝えます。クリックする（またはパレットの項目 *LFS: インストールページを開く* を使う）と https://git-lfs.com が開きます。それ以外は何も変わりません。ウィンドウは開き、diff は動作し、大きなファイルはポインターとして表示されるだけです。
- **ファイルメニューとパレット。** ステータスペインで変更されたファイルを右クリックします：通常のファイルには *LFS で追跡：このファイル* / *この拡張子*。LFS ファイルには、ロックを持つ人に応じて *ファイルをロック（LFS）*、*ファイルのロックを解除（LFS）*、*強制解除*。ロックされたファイルの行には `自分がロック中` / `<name> がロック中` と表示されます。すべてはパレットのコマンドでもあります（`Ctrl+P`、`LFS` と入力）：初期化、パターンの track / untrack、オブジェクトの fetch / pull / push、prune、ロックの表示、インストールページを開く。
- **失敗した操作は自ら説明します。** `git-lfs` がないために LFS の操作が失敗した場合、ダイアログにメッセージと*インストールページを開く*ボタンが表示されます。
- **設定。** *設定 > LFS*（リポジトリごと）には、LFS が初期化されているかどうかと追跡中のパターンが表示され、**ここで LFS をセットアップ**ボタンがあります。*Git とエンコード*（リポジトリごと）には、**Git LFS** スイッチ、読み取り専用の **git-lfs プログラム**の行、**このリポジトリの LFS を初期化**ボタンがあります。
- **Git LFS** をオフにする（`git.lfs_enabled = false`。[設定リファレンス](config-reference.md)を参照）と、これらはすべて非表示になります。git 自身の LFS フィルターは引き続き動作します。
- LFS の状態は、リポジトリを開いた直後、変更のたび、手動での更新時、`.gitattributes` が変わったとき、そして LFS を使うリポジトリでは 10 秒ごとに読み取られます。

## 既知の制限

- LFS のパターンが*入れ子の* `.gitattributes`（ルートのものでも `.git/info/attributes` でもない）にしかないリポジトリは、LFS のオブジェクトストア（`.git/lfs`）ができて初めて LFS を使っていると検出され、それより前には検出されません。
- `git lfs migrate` 自体は使いません。履歴全体でファイルを LFS に移す、または LFS から戻す処理は、履歴の書き換えのルール *ファイルを LFS に移行* / *LFS からファイルを戻す* が行います。これらはローカルのクローンに対して動作し、書き込まれるのはルートの `.gitattributes` だけで、オブジェクトはあとで push する必要があります（結果パネルの *LFS オブジェクトを送信*、または `lfs push --all`）。
