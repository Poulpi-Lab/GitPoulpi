[English](../multiplexers.md) · **日本語**

# マルチプレクサとペイン

マルチプレクサは GitPoulpi とターミナルの間に入ります。セルを自分で描き直し、理解できるエスケープシーケンスだけを
転送します。そのため、ターミナル単体で動作すること（[ターミナルの選び方](terminals.md)）が、tmux、zellij、
screen、herdr の中では動作しないことがあります。このページでは、何が通るかと、追加する設定を説明します。結果は
ターミナルの表と同じ 2026年11月のテストによるものです：tmux 3.8（MSYS2：3.7c）、zellij 0.45.1 と 0.46.0、
GNU screen 5.0（macOS に入っているのは 4.00.03）、herdr 0.9.5、psmux 3.4.0。

GitPoulpi は、`TMUX`、`ZELLIJ`、`STY`、`TERM=screen*` または `tmux*`、`HERDR_*` からマルチプレクサを検出します。
その中では、下の表が GitPoulpi が使える経路を検出すると書いている場合を除き、画像は文字が既定になります。
`GITPOULPI_TERMINAL_IMAGE=off` はすべての画像をオフにし、`ui.image_protocol` はプロトコルを強制します。

## 早見表

| | 画像 | ポインターの形（OSC 22） | クリップボード（OSC 52） | キー |
|---|---|---|---|---|
| **tmux** | 背後のターミナルが対応していれば sixel（検出）。`allow-passthrough on` と `ui.image_protocol = "kitty"` で kitty の画像 | `allow-passthrough on` で kitty と foot では動作。Ghostty では動作しない | `set-clipboard on` で動作 | kitty キーボードなし。`Ctrl+Enter` と `Ctrl+,` は届かない |
| **zellij** | 文字 | なし | あり | kitty キーボードが動作 |
| **GNU screen** | 文字 | なし | なし | 従来のキー |
| **herdr** | kitty の画像（`kitty_graphics = true` で検出） | なし | あり | 未測定 |
| **psmux**（Windows） | 文字（設計上） | なし（Windows Terminal にはない） | システムのクリップボードが動作 | 未測定 |
| **MSYS2 の tmux**（Windows） | 文字（設計上） | なし | システムのクリップボードが動作 | 未測定。画面が乱れる |
| **WezTerm のペイン**、**Windows Terminal のペイン**、**kitty と Ghostty の分割**、**iTerm2 の分割ペイン** | ターミナル自身と同じ | ターミナルと同じ | ターミナルと同じ | ターミナルと同じ |
| **iTerm2 の `tmux -CC`** | 文字 | なし | なし（iTerm2 は既定で拒否） | 未測定 |

## tmux

**画像。**

- **Sixel** は tmux 自身が、対応するクライアント向けに再エンコードして描画します。GitPoulpi は起動時に tmux へ
  一度問い合わせ（`tmux display-message -p '#{client_termfeatures}'`、400 ms まで）、クライアントが sixel を
  挙げていれば sixel を使います。これは foot、WezTerm、Rio、Tabby の背後では当てはまり、kitty と Ghostty の背後では
  当てはまりません（tmux は「SIXEL IMAGE」という文字を出力します）。sixel 付きでビルドした tmux が必要です
  （Arch と Homebrew のものはそうです。`tmux -V` と sixel のテストで確認してください）。設定は不要です。
  tmux はデバイス属性の問い合わせに、背後に何があっても sixel に対応していると答えるため、GitPoulpi は
  代わりにクライアントの機能を尋ねます。
- **tmux の背後の iTerm2** は sixel に対応していると名乗りますが、未設定のピクセルを不透明に塗るため、ほとんどが
  透明なグラフは塗りつぶしの長方形になってしまいます。GitPoulpi は iTerm2 を認識して（`LC_TERMINAL`、
  `ITERM_SESSION_ID`、クライアントの種類）文字で描画します。
- **kitty の画像**は tmux の DCS パススルーを通ります（kitty、Ghostty。生の probe では iTerm2 と Rio でも確認）。
  `set -g allow-passthrough on`（tmux 3.3a 以降）**と**、GitPoulpi の `ui.image_protocol = "kitty"` が必要で、
  後者によりアップロードと削除が包まれます。tmux のオプションがないと、包みが捨てられてグラフは空白のままです。
- iTerm2 のインライン画像は tmux を通りますが、tmux はその位置を決められません。GitPoulpi はそこでは
  使いません。

**ポインターの形。** GitPoulpi は `OSC 22` をパススルー用に包みます。`allow-passthrough on` で kitty と foot では
ウィンドウに届きます。Ghostty は、包まれていると表示しません（矢印のままです）。WezTerm、Rio、Alacritty、Tabby、
iTerm2 には表示する `OSC 22` がありません。

**クリップボード。** `set -g set-clipboard on`。GitPoulpi はパススルー用に包んだ形も送ります（`allow-passthrough on`
で動作します）。kitty、Ghostty、Alacritty、Tabby の背後では届きます。iTerm2 は、クリップボードの許可をオンにする
まで拒否します。

**キー。** tmux は kitty キーボードの問い合わせに応答せず、`extended-keys` でも `Ctrl+Enter` や `Ctrl+,` は
通りません：kitty の中の tmux 3.8 で測定したところ、`extended-keys` がオフ、オン、always のどれでも、両方とも
ただの `Enter` と `,` として届きました。`Ctrl+Shift+Z` のような文字の組み合わせが届くのは、`extended-keys` が
オンまたは always で、**かつ** `extended-keys-format csi-u`（tmux 3.5 以降）で、**かつ**アプリケーションが拡張キーを
要求する場合だけです。GitPoulpi は要求しないので、この設定は何も変えません。tmux の中では `Alt` の別形を
使ってください
（[キーバインド](keybindings.md#ターミナルが送れないキーと-kitty-キーボードプロトコル)）。

**設定**（`~/.tmux.conf`）：

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later: pointer shapes, forced kitty pictures
```

**WSL の中の tmux。** 未測定です。Linux 版のビルドなので、外側のターミナルが Windows Terminal の場合は
上と同じ結果になると考えられます。

## zellij

画像：zellij は kitty プロトコルを受け付けますが、その Unicode プレースホルダーは受け付けず、sixel や iTerm2 の
画像も運ばないため、GitPoulpi は文字で描画します（意図的な拒否）。ポインターの形：握りつぶされるので、GitPoulpi は
送りません。クリップボード：`OSC 52` は転送されます（kitty、Ghostty、Alacritty、Tabby の背後で動作。iTerm2 と
Terminal.app は拒否または無視します）。キー：zellij は kitty キーボードの問い合わせに応答するので、すべての
キーが動作します。

zellij 0.46 は新しく始めたときに「Welcome to Zellij」のポップアップを開きます：`Esc` を押してください。終了した
セッションは名前が残り、同じ名前で次に起動すると、どうするかを尋ねられます。設定は不要です。

## GNU screen

何も通りません：画像と `OSC 22` は文字として出力されるか捨てられ、`OSC 52` は無視され、キーボードは従来のものです。
GitPoulpi は文字で描画します。screen 4.00.03（macOS に入っているバージョン）は、画面内の色と全角のグリフも
乱します。screen 5 は Linux でのみテストしました。使えるセッションには tmux か zellij を使ってください。

## herdr

herdr は **kitty の画像**を再出力するターミナルを内蔵しています。herdr の `config.toml` が
`kitty_graphics = true`（`[terminal]` の下、古いバージョンでは `[experimental]`）となっていて、herdr を囲む
ターミナルがそれを描画できる（kitty、Ghostty、WezTerm）場合、GitPoulpi はそれを使います：検出され、設定は
不要です。設定ファイルが別の場所にある場合（たとえば `HERDR_HOME` の下。GitPoulpi は読みません）は、
`ui.image_protocol = "kitty"` を設定してください。SSH 越し（`herdr --remote`）では GitPoulpi はローカルの
クライアントを見られないので、文字で描画します。
[トラブルシューティング](troubleshooting.md#グラフが画像ではなく文字で描かれるherdrtmuxそのほかのターミナル)を
参照してください。`OSC 22` は herdr 0.9.5 が転送しないので、GitPoulpi は送りません。`OSC 52` は kitty、Ghostty、
Alacritty、Tabby の背後で届きます。生の probe では、herdr は既定の設定でも kitty、Ghostty、iTerm2 で kitty の
画像を描画しました。キー：未測定です。

## psmux と MSYS2 の tmux（Windows）

- **psmux**（Rust、ConPTY、tmux のコマンド言語。`winget install psmux`）は `TMUX` を設定するため、GitPoulpi は
  そこでは設計上、文字で描画します。Windows Terminal の中では普通に動き、コピーはシステムのクリップボードで
  動作します。ポインターは変わりません（Windows Terminal には `OSC 22` がありません）。ほかのホストターミナルは
  測定していません。
- **MSYS2 または Cygwin の tmux** も `TMUX` を設定し（設計上、文字）、テストでは画面が乱れました：mintty でも
  Windows Terminal でも、古いフレームが重なって描かれました（`-u` の有無にかかわらず。ない場合は罫線が `qqqq` と
  出力されました）。MSYS2 の疑似端末の中のネイティブ Windows プログラムは、tmux によって正しく再描画されません。
  GitPoulpi では直せません：psmux、Windows Terminal のペイン、WezTerm のペインを使ってください。
- zellij と screen には Windows 向けのリリースがありません。herdr 0.9.5 はインストールしましたが実行していません。

## ターミナルのペイン

ターミナル自身に属するペインは、GitPoulpi にとってはマルチプレクサではありません。

- **WezTerm のペイン**（`wezterm cli split-pane`、`wezterm connect`、SSH ドメイン）：WezTerm がすべてを描画する
  ので、画像は動作し（ペイン内のグラフとアバター）、ポインターは WezTerm 自身と同じ動作になります（形は
  アップストリームの WezTerm PR #8092 を含むビルドで動作し、ペインの区切りはリサイズ用のポインターになります）。
  Linux と macOS では未測定です。
- **Windows Terminal のペイン**（`wt ; split-pane`）：ペイン内で sixel のグラフとアバターが動作し、コピーも動作し、
  ポインターの形はありません。
- **kitty、Ghostty、iTerm2 の分割**：ターミナルと同じネイティブの動作です。kitty と Ghostty は macOS では
  未測定です（GPU が必要）。

## iTerm2 の `tmux -CC`（コントロールモード）

iTerm2 は tmux のウィンドウごとにネイティブのウィンドウを開き、「[tmux bash]」というゲートウェイのウィンドウを
保持します。`TMUX` が設定されているため、GitPoulpi はネイティブのウィンドウでは文字で描画します。画像は
コントロール接続を通して iTerm2 に届きます（`OSC 1337` のテストで「Allow Terminal-Initiated Display?」ダイアログが
出ました）が、GitPoulpi はそこでは使いません。`ui.image_protocol = "kitty"` を強制した場合の kitty のペイロードは
ゲートウェイのウィンドウに出力されるので、`auto` のままにしてください。`OSC 22` と `OSC 52` は届きません。

## 知っておきたい設定

| 設定 | 用途 |
|---|---|
| `ui.image_protocol` | `kitty` は、kitty の画像を転送するマルチプレクサ（`allow-passthrough on` の tmux、GitPoulpi が読まない設定の herdr）を通して、kitty の画像を強制します。`none` は文字にします。 |
| `ui.graph_renderer` | `text` は常に正しい表示です。 |
| `ui.kitty_keyboard` | マルチプレクサがキーボードの問い合わせでおかしくなるときは `off`。 |

マルチプレクサの背後のターミナルについては[ターミナルの選び方](terminals.md)を、グラフが文字で描かれるときは
[トラブルシューティング](troubleshooting.md)を参照してください。
