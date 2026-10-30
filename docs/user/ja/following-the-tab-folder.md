[English](../following-the-tab-folder.md) · [Čeština](../cs/following-the-tab-folder.md) · [Deutsch](../de/following-the-tab-folder.md) · [Español](../es/following-the-tab-folder.md) · [Français](../fr/following-the-tab-folder.md) · [Italiano](../it/following-the-tab-folder.md) · **日本語** · [한국어](../ko/following-the-tab-folder.md) · [Nederlands](../nl/following-the-tab-folder.md) · [Polski](../pl/following-the-tab-folder.md) · [Português (Brasil)](../pt-BR/following-the-tab-folder.md) · [Português (Portugal)](../pt-PT/following-the-tab-folder.md) · [Русский](../ru/following-the-tab-folder.md) · [Svenska](../sv/following-the-tab-folder.md) · [Türkçe](../tr/following-the-tab-folder.md) · [Українська](../uk/following-the-tab-folder.md) · [简体中文](../zh-CN/following-the-tab-folder.md)

# アクティブなタブのフォルダーに追従

GitPoulpi は複数のリポジトリをタブとして開けます。起動元のシェルは自分のフォルダーに残ります。
プログラムは親シェルのフォルダーを変更できないからです。GitPoulpi には、
アクティブなタブのフォルダーでシェルを使うための 2 つの方法があります。

- **終了して cd**：GitPoulpi が終了し、シェルがアクティブなタブのフォルダーに移動します。
  これには `gitpoulpi shell-init` が出力する小さなシェル関数が必要です。
- **開く場所**：アクティブなタブのフォルダーを、ターミナル、マルチプレクサの
  ペイン、エディター、ファイルマネージャーで開きます。シェルの設定は不要です。
  [開く場所](open-in.md)を参照してください。

```
┌🐙 GitPoulpi vX.Y.Z -- /work/other-repo - Open in ▾ ── ⌕ 全体検索 ⚙ × ┐
│ ⌂  main-repo x  other-repo x                   Quit & cd here           │
```

- **開く場所**はタイトルバーの、リポジトリのパスの後ろにあります。ウィンドウが
  狭すぎると非表示になります。
- **終了して cd**はタブバーの右端にあります。次の両方が成り立つときだけ表示されます。
  - GitPoulpi がシェル関数経由で起動された（下を参照）。
  - アクティブなタブが、GitPoulpi を起動したフォルダーではない。
- コマンドは常にパレット（`Ctrl+P`）にあります。`タブ: 終了してここへ cd`、
  `タブ: 開く先...`（`Alt+O`）、`タブ: 既定の開く先で開く`（`Ctrl+Alt+O`）です。
  シェル関数がない場合、`タブ: 終了してここへ cd` は設定方法を伝えます。

通常の終了（`q`）では、シェルのフォルダーは決して変わりません。
**終了して cd**だけが変えます。ただし `auto`（下を参照）を選んだ場合は別です。

## シェル関数の設定

`gitpoulpi shell-init <shell>` は `gp` という名前の関数を出力します。別の名前にするには `--name` を使います。
この関数は `gitpoulpi --cwd-file <temp file> ...` を実行します。
**終了して cd**で終了すると、GitPoulpi はそのファイルにフォルダーを書き込み、
関数がそのフォルダーに移動します。GitPoulpi は何もインストールも編集もしません。
シェルの起動ファイルに 1 行を追加するだけです。

| シェル | 追加先 | 行 |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(gitpoulpi shell-init bash)"` |
| zsh | `~/.zshrc` | `eval "$(gitpoulpi shell-init zsh)"` |
| fish | `~/.config/fish/config.fish` | `gitpoulpi shell-init fish \| source` |
| PowerShell | `$PROFILE`（`notepad $PROFILE`） | `gitpoulpi shell-init pwsh \| Out-String \| Invoke-Expression` |

その後は、`gitpoulpi` の代わりに `gp` で GitPoulpi を起動します。引数はそのまま渡され
（`gp -s work`）、終了ステータスは `gitpoulpi` の終了ステータスです。

**既存の `gp` は、警告なしに置き換えられることはありません。** シェルにすでに
`gp` という名前のコマンド、エイリアス、関数がある場合、スクリプトは警告を表示し、
何も定義しません。別の名前を選んでください。たとえば
`gitpoulpi shell-init bash --name gpl` です。PowerShell では、この関数は
組み込みの `gp` エイリアス（`Get-ItemProperty`）と、`gitpoulpi` を指すエイリアスを置き換えます。
それ以外の `gp` は、警告付きで残されます。

すべての `gitpoulpi` コマンドは `--cwd-file` を受け付けますが、ファイルに書き込むのは TUI だけです。

## 設定：アクティブなタブのフォルダーに追従

設定 > 開く場所 > **アクティブなタブのフォルダーに追従**（`ui.tab_dir_follow`）：

| 値 | 動作 |
|---|---|
| `buttons`（既定） | 上で説明した**終了して cd**と**開く場所**。 |
| `auto` | **終了して cd**ボタンはありません。**開く場所**は残ります。終了するたびに、シェルがアクティブなタブのフォルダーに移動します（シェル関数で GitPoulpi を起動した場合）。新しいタブを開くたびに、**開く場所**の既定のエントリが実行されます。 |

`auto` モードでは、既定のエントリは自分で開いたタブに対してのみ実行されます。
ホームタブ、worktree、クローン、初期化から開いた場合です。起動時に復元されたタブ、最初のタブ、
すでに開いているタブへの切り替えでは実行されません。

古い設定 `ui.terminal_launch` と `ui.terminal_launch_command` は、**開く場所**の一覧に
移されました。[開く場所](open-in.md#従来のキー)を参照してください。
