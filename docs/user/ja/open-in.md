[English](../open-in.md) · [Čeština](../cs/open-in.md) · [Deutsch](../de/open-in.md) · [Español](../es/open-in.md) · [Français](../fr/open-in.md) · [Italiano](../it/open-in.md) · **日本語** · [한국어](../ko/open-in.md) · [Nederlands](../nl/open-in.md) · [Polski](../pl/open-in.md) · [Português (Brasil)](../pt-BR/open-in.md) · [Português (Portugal)](../pt-PT/open-in.md) · [Русский](../ru/open-in.md) · [Svenska](../sv/open-in.md) · [Türkçe](../tr/open-in.md) · [Українська](../uk/open-in.md) · [简体中文](../zh-CN/open-in.md)

# 開く場所

タイトルバーのリポジトリのパスの隣にある**開く場所 ▾**ボタンは、アクティブなタブのフォルダーを
別の場所で開きます。ターミナル、マルチプレクサの新しいタブやペイン、エディター、
ファイルマネージャーなどです。一覧に何を入れるかは自分で選びます。決め打ちのものは何もなく、
すべてのエントリはラベルとコマンドです。

## 使い方

- **開く場所 ▾ をクリック**：一覧がドロップダウンします。エントリをクリックすると実行されます。既定の
  エントリには `•` が付きます。最後の行の**設定...**は、
  [設定](settings.md#開く場所)の「開く場所」ページを開きます。

  ![開く場所の一覧：ターミナル、2 つのエディター、Git ツール、ファイルマネージャー。既定には黒丸が付き、末尾に設定...がある](../images/open-in-menu.png)

- **キーボード**：`Alt+O` で同じ一覧が開きます（`Up`/`Down`、`Enter`、`Esc`。
  エントリが 7 つを超える場合は、入力で検索）。`Ctrl+Alt+O` は既定のエントリを
  すぐに実行します。パレット（`Ctrl+P`）：`タブ: 開く先...` と `タブ: 既定の開く先で開く`。
  パレットのコマンドは、タイトルバーにボタンの場所がないときでも動作します。
  どちらのキーも再割り当てできます。[キーバインド](keybindings.md)を参照してください。
- **リポジトリのタブを右クリック**：タブメニューに、同じエントリを持つ**開く場所**
  サブメニューがあります。
- 一覧は、このセッションで最後に使ったエントリから始まります。
- ステータスバーにはすぐに「{label} で開いています...」と表示されます。プログラムが約 1.5 秒以内に失敗した場合
  （たとえばサーバーのない tmux や、不正なフラグ）、そのエラー出力の最初の行が
  ステータスバーに表示されます。インストールされていないプログラムは、すぐに名前が示されます。

自分の一覧を保存するまで、既定のエントリは検出されたものから選ばれます。herdr の中では「Herdr: new tab」、Omarchy では kitty の新しいウィンドウ（kitty がそのターミナルだからです）、それ以外ではファイルマネージャーです。

このボタンは、GitPoulpi を起動したフォルダーを含むすべてのリポジトリのタブに表示され、
`ui.tab_dir_follow = "auto"` モードでも表示されます。`auto` モードでは、新しいタブを開いたときに
既定のエントリが自動的に実行されます（起動時に復元されたタブでは実行されません）。
[アクティブなタブのフォルダーに追従](following-the-tab-folder.md)を参照してください。

## エントリの形

`config.toml` では、エントリごとに 1 つの `[[ui.open_in]]` テーブルになります。

```toml
[ui]
open_in_default = "Herdr: new tab"   # a label; empty means the first entry

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

手で書くことはほとんどありません。設定のページで編集できます。すべてのキーは
[設定リファレンス](config-reference.md)にあります。

### コマンドと変数

コマンドはまず引数に分割され（スペース、`'...'`、`"..."` で単語がまとまります）、
その後、各引数の中で変数が置き換えられます。そのため、名前にスペースを含むフォルダーも
1 つの引数のままです。コマンド自身がシェルを起動しない限り、**シェルは使われません**。

| 変数 | 置き換えられるもの |
|---|---|
| `${cwd}` | タブのフォルダー。 |
| `${repo}` | リポジトリのルート（worktree のタブでは `${cwd}` と異なります）。 |
| `${name}` | フォルダーのベース名。 |
| `${branch}` | checkout されているブランチ。detached の場合は空。 |
| `${web_url}` | origin リモートのウェブページ（GitHub または GitLab の場合）。それ以外は空。 |
| `${env:NAME}` | 環境変数 `NAME`。未設定の場合は空。 |

古い `{repo}` も引き続き動作し、`${cwd}` を意味します。それ以外の `${...}` は、
その変数名を示すエラーになります。

プログラムは、タブのフォルダーを作業ディレクトリとして起動され、3 つの環境変数を受け取ります。
シェルのスニペットで便利です。

| 変数 | 値 |
|---|---|
| `GITPOULPI_CWD` | タブのフォルダー。 |
| `GITPOULPI_REPO` | リポジトリのルート。 |
| `GITPOULPI_BRANCH` | ブランチ。なければ空。 |

```toml
[[ui.open_in]]
label = "Terminal here, then status"
command = "sh -c 'cd \"$GITPOULPI_CWD\" && git status; exec $SHELL'"
```

Windows では、プログラムは `PATH` と `PATHEXT` から探されるので、`code` や
`cursor`（`.cmd` ファイル）が動作し、コンソールウィンドウが一瞬表示されることもありません。

## プリセット

設定 > 開く場所 > **プリセットを追加**には、お使いのオペレーティングシステム向けのエントリだけが
カテゴリ別に一覧されます（Windows のプリセットは Linux には表示されません。ほかも同様です）。各行は
カテゴリで始まります：`エクスプローラー: ...`、`マルチプレクサ: ...`、`ターミナル: ...`、
`エディター: ...`、`Git ツール: ...`、`AI エージェント: ...`、`Web: ...`。先頭の**検出**は、
計算された既定の一覧のエントリです。各カテゴリの中では、すぐに使えるプリセット（そのマルチプレクサや
ターミナルの中で実行していて、プログラムが `PATH` にあるもの）が最初に、次にインストール済みのもの、
それ以外の順になります。コンボボックスに入力して検索できます。たとえば `lazygit` や `Git ツール:` です。
**現在のタブで試す**で、頼る前にエントリを試せます。

![開く場所ページの「+ 追加」のリスト：検出されたエントリが先頭、次にエクスプローラー、マルチプレクサなどのカテゴリ。各プリセットの先頭にカテゴリが付いている](../images/settings-open-in-presets.png)

「ペインでツールを実行する」プリセット（`lazygit in Herdr tab`、`yazi in tmux window`、
`Claude Code in Herdr split` など）は、ホスト（herdr、tmux、zellij、WezTerm、kitty、
`xdg-terminal-exec`、Windows Terminal）とツールの両方がインストールされている場合にのみ提示されます。
`$EDITOR in ...` は、`EDITOR` 変数で指定されたエディター（herdr では `vi` にフォールバック）を
herdr または tmux の中で実行します。

herdr のツールのプリセットは、タブまたは分割を作成してから、そのシェルにコマンドを入力するので、
ツールはそのフォルダーで起動し、終了後もペインは開いたままです。
`AI エージェント` のものは `herdr agent start` を使うので、herdr がエージェントの状態を追跡します。

| カテゴリ | プリセット（`V` = 実行して、またはインストール済みのヘルプと照らして検証済み） |
|---|---|
| エクスプローラー | Windows の `explorer.exe`（V）、macOS の Finder（V）、Linux の `xdg-open`（V）、Nautilus、Dolphin、Thunar、Nemo、PCManFM。ペイン/タブ/ターミナル内の yazi、superfile、nnn、lf、ranger |
| マルチプレクサ | herdr のタブ、ワークスペース、右/下への分割（V）。tmux のウィンドウ、右/下への分割、デタッチしたセッション（V）。zellij のペイン、タブ、フローティング。WezTerm のタブ、分割、新しいウィンドウ。kitty のタブ、分割ウィンドウ、OS ウィンドウ （V：フラグはヘルプから。`allow_remote_control` が必要）。screen。cmux。psmux |
| ターミナル | `xdg-terminal-exec`（V）、kitty（V）、foot（V）、Alacritty（+ `alacritty msg`）、WezTerm、Ghostty（Linux、macOS）、GNOME Terminal、Ptyxis、Konsole、Xfce Terminal、Tilix、Terminator、MATE Terminal、iTerm2、Terminal.app、Warp。Windows：Windows Terminal（ウィンドウ、タブ、分割、PowerShell 7、Windows PowerShell、コマンドプロンプト、WSL 用のタブ）、PowerShell 7 / Windows PowerShell / コマンドプロンプト / WSL のウィンドウ、Git Bash |
| エディター | VS Code（V）、VSCodium、Cursor、Windsurf、Zed、Sublime Text、Emacs、IntelliJ IDEA、RustRover、PyCharm、WebStorm、CLion、GoLand、Rider、PhpStorm、Android Studio、Visual Studio（Windows）。Neovim、Vim、Helix、ペイン/タブ/ターミナル内の `$EDITOR` |
| Git ツール | gitk、`git gui`、gitg、Sublime Merge、GitKraken、GitHub Desktop。ペイン/タブ/ターミナル内の lazygit、tig、gitui（herdr、tmux、kitty、`xdg-terminal-exec` のものは V） |
| AI エージェント | herdr のタブ/分割（V、Claude で）、tmux、zellij、WezTerm、kitty、新しいターミナルでの Claude Code、Codex、Gemini CLI、opencode |
| Web | `Repository page on the web`（`${web_url}`）、`gh repo view --web`（V）、`glab repo view --web` |

Windows の WSL：`wsl.exe --cd ${cwd}` は Windows のパスをそのまま受け付け、
Windows Terminal の `wsl.exe` タブはそのフォルダーで起動します。WSL はそれを `/mnt/` の下に対応付けます。

エクスプローラー、Windows Terminal、PowerShell、`cmd`、WSL、Warp、psmux、Visual Studio、
Zed（Zed は Unix 専用）は、存在する環境にのみ表示されます。tmux、zellij、cmux、
screen、kitty のリモート制御、`sh` で動く herdr のプリセットは Unix 専用です。
herdr（通常のプリセット）、WezTerm、Alacritty、VS Code、Cursor、JetBrains の IDE は
どこでも提示されます。

### 変数 `${web_url}`

リポジトリの `origin` リモートのウェブページです。GitHub または GitLab のプロジェクト
（github.com、gitlab.com、または `[[integrations]]` にエントリのあるセルフマネージドのホスト）の場合に限ります。
それ以外は空で、これを使うエントリは空のアドレスを開く代わりにメッセージを出して停止します。
これは任意です。`Repository page on the web` 以外に、これを必要とするプリセットはありません。

## 設定ページ

設定 > **開く場所**で一覧を編集します。エントリは左側の[リスト](settings.md#リスト)（既定のエントリには `★` が付きます）に並び、選択中のエントリのフィールドが右側に表示されます。リストの管理には、`+ 追加`、`複製`、`削除`、`↑` / `↓`、行のドラッグを使います。

| 行 | 内容 |
|---|---|
| **既定のエントリ** | **既定の開く先で開く**が実行するエントリ。「開く場所」の一覧で `•` が付きます。 |
| **ラベル** | 一覧に表示される名前。重複不可、空は不可。 |
| **コマンド** | コマンド。挿入できる変数のチップ（`${cwd}`、`${repo}`、`${branch}`、`${name}`、`${web_url}`）があります。 |
| **現在のタブで試す** | アクティブなタブのフォルダーで、今すぐエントリを実行します。 |
| **+ 追加 ▾** | 用意済みのエントリ（検出されたものが先頭）を追加します。**カスタム**を選ぶと空のエントリが追加され、続けてそのコマンドを編集します。 |
| **検出された既定値を復元** | 一覧をこのマシンで検出されたものに置き換え、既定を解除します。 |

編集は行うたびに保存されます。最初の編集で `[[ui.open_in]]` 配列全体が `config.toml` に書き込まれます。
それまでは一覧は計算されたもので（[従来のキー](#従来のキー)を参照）、ファイルに
`open_in` キーはありません。設定の検索でもこのページが見つかります。

## バイナリファイル：開く

バイナリファイル（PDF、Word 文書、表計算、アーカイブなど）は、Diff ビューやファイルビューで表示できるテキストがありません。オペレーティングシステムがその種類に対応するアプリを知っている場合、ビューにはいつもの通知の横に **開く** ボタンが表示されます。リストの先頭には、システムがその種類を開くアプリが丸印付きで並び、続いてシステムがその種類に登録している他のすべてのアプリ（システム自身の「アプリで開く」リスト）が並びます。Linux ではファイル名の MIME タイプから、既定のアプリは `mimeapps.list`、その他は `.desktop` ファイルから取得します。Windows ではレジストリ（ユーザーの選択と拡張子に登録されたプログラム）、macOS では既定のアプリのみです。

`o` は既定のアプリを起動し、`Shift+O` はリストを開きます。どちらもコマンドパレット（*ファイル: 既定のアプリで開く*、*ファイル: アプリで開く...*）にあり、**アプリで開く...** はステータスペインのファイルや、commit のファイル一覧の右クリックメニューにもあります。検索はバックグラウンドで拡張子ごとに一度だけ実行され、その間ビューには「アプリを検索中...」と表示されます。システムが対応アプリを知らない種類やプレーンテキストには、これまでの通知だけが表示され、ボタンはありません。

![ファイルビューの PDF と「Open in」の一覧：先頭に黒点つきの既定のアプリ、続いてシステムがその種類に登録している他のアプリ](../images/open-with.png)

作業ツリーのファイルはそのままアプリに渡されます。commit（またはインデックス）のバージョンは **読み取り専用の一時コピー** に書き出され、ステータスバーに通知が出るため、アプリがリポジトリを変更することはありません。どちらもリポジトリを変更しません。ジャーナルのエントリは作られず、元に戻すにも影響しません。

メニューの一覧は、サブメニューのように **アプリで開く...** 行と同じ高さでメニューの横に開きます（右に余白がないときは左）。マウスでもキーボードでも同じです。ファイルメニューから開く画像エディターの一覧（**編集...**）も同じです。

### Git LFS ファイル

Git LFS オブジェクトのカード（[Git LFS](lfs.md)を参照）には、テキストの下に同じ 2 つの操作 **既定のプログラムで開く** と **開く場所** があり、キー（`o`、`Shift+O`）とパレットの行も同じです。ポインターではなく実際のファイルを開きます。作業ツリーのファイルはそのまま、commit またはインデックスのバージョンはローカルの LFS ストアにあるオブジェクトの読み取り専用の一時コピーです。まだダウンロードされていないオブジェクトのカードにはボタンがなく、ステータス行に内容がまだダウンロードされていないと表示されます。ポインターしか持たない作業ツリーのファイルにも同じ拒否が働きます。カードでもファイルメニューでも、**既定のプログラムで開く**、**開く場所**、**編集...** はポインターのテキストを開かず、内容がダウンロードされていないと表示します（LFS ツールバーの pull か checkout の行でダウンロードしてから、やり直してください）。開く操作は読み取り専用で、ジャーナルのエントリはなく、元に戻すものもありません。

### Windows のブラウザー

Chrome と Edge は `--single-argument %1` を登録しており、スイッチ以降のすべてを、引用符も含めて 1 つのアドレスとして読みます。GitPoulpi は、そのようなプログラムにはパスを、Windows と同じく引用符なしでそのまま渡すので、パスに空白があるファイルも `http://"c/...` で失敗せずに開きます。そのほかのプログラムには、パスが 1 つの通常の引数として渡されます。

## 従来のキー

古いキー `ui.terminal_launch` と `ui.terminal_launch_command` は、`ui.open_in` がない間だけ
読み取られます。その場合、計算される一覧は次のとおりです。

1. 以前のターミナルの選択（`auto` 以外の場合。`custom` は
   `{repo}` を `${cwd}` に書き換えた「Terminal（custom）」になり、プリセットの id は
   そのプリセットになります）。`auto` の値は、最初のエントリ「Default terminal」になります。
2. OS のファイルエクスプローラー、
3. 環境から検出されたマルチプレクサとターミナル、
4. `PATH` 上で見つかったエディター。

設定から一覧を保存すると、`ui.open_in` が優先され、古いキーは無視されます。
パレットのコマンド**タブ: ターミナルで開く**と**タブ: ファイルマネージャーで開く**は
この一覧を使いません。

## トラブルシューティング

- *「program X was not found」*：そのプログラムが、GitPoulpi から見える `PATH` にありません。
  それがあるシェルから GitPoulpi を起動するか、フルパスを書いてください。
- *tmux/zellij/herdr のエントリがすぐに失敗する*：これらは実行中のサーバーと通信するため、
  GitPoulpi がその中で動いているときにしか動作しません。
- *kitty のタブのエントリが失敗する*：`kitty.conf` に `allow_remote_control` が必要です。
- *Windows のエクスプローラー*：`explorer.exe` は成功しても終了コード 1 で終了します。GitPoulpi は
  それを成功として扱います。
