[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · **日本語** · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# ターミナルの選び方

GitPoulpi はマウスに対応したどのターミナルでも動きますが、画像を描画でき、トゥルーカラーを表示でき、
キーボードのすべてのキーを送れるターミナルで最もよく動作します。このページでは、OS ごとにターミナルを順位付けし、
それぞれのターミナルで何が得られるか、どの設定を変えるかを説明します。内容は、2026年10月に各プロジェクトの
ドキュメント、変更履歴、Issue トラッカーで確認しました（[出典](#出典)を参照）。すべてのターミナルで
実際に試したわけではありません。**（未確認）** と書かれた項目は、一次情報で確認できませんでした。

## おすすめ

| OS | おすすめ（順位順） | 理由 |
|---|---|---|
| **Windows** | **WezTerm**、次に **Windows Terminal** | WezTerm はグラフとアバターを透過つきの画像として描画します。Windows Terminal は OS との統合が最も優れています（ランチャーの既定です）。アバターは sixel 画像、グラフは文字で描画します。 |
| **Linux**（X11、Wayland、Hyprland、Omarchy） | **kitty**、**Ghostty**、**WezTerm**、次に **foot** | kitty と Ghostty は画像とすべてのキーを備えます。WezTerm も画像を表示できます。foot はアバターを sixel 画像で描画し、高速ですべてのキーを送れます。Omarchy で画像を使うなら Ghostty か kitty を選んでください。 |
| **macOS** | **kitty**、**Ghostty**、**WezTerm**、次に **iTerm2** | 理由は同じです。Terminal.app は動作しますが、画像もトゥルーカラーもありません。 |

どの OS でも、**tmux、zellij、screen** はターミナルの画像プロトコルを GitPoulpi から隠すため、
GitPoulpi は文字で描画します（[マルチプレクサ](#マルチプレクサ)を参照）。

## GitPoulpi が使うもの

- **画像**（グラフ列、作成者のアバター、画像 diff とファイルビュー）：Unicode プレースホルダーを使う
  **kitty グラフィックスプロトコル**、**iTerm2 インライン画像**（`OSC 1337`）、**sixel** のいずれかで、
  この順に優先されます。検出は「拒否が既定」で、`TERM`、`TERM_PROGRAM`、`KITTY_WINDOW_ID`、
  `GHOSTTY_RESOURCES_DIR`、`WEZTERM_EXECUTABLE`、`WT_SESSION`、`KONSOLE_VERSION` とマルチプレクサの
  環境変数から判断し、キーボードは読み取りません（そのため、ターミナルにデバイス属性も問い合わせません）。
  kitty と Ghostty は kitty プロトコルを使います。WezTerm は（意図的に）
  iTerm2 プロトコルを使い、PNG の透過を合成します。iTerm2、Rio、mintty は不透明な背景つきの
  iTerm2 プロトコルを使います。Windows Terminal、foot、Konsole、mlterm は **sixel** を使います。
  どれも使えない場合、アバターはイニシャルのバッジになります。
- **Sixel** は**グラフ列**（レーン、ノード、`ui.avatars = "graph"` ならコミットの丸の中のアバター）に使われ、それ以外では**アバター**（ログ、commit の詳細、ファイル履歴、Blame、画像ビュー）に使われます。sixel にはアルファも ID も拡大縮小もないため、画像は `列数 x セル幅` x `行数 x セル高` ピクセルちょうどで、何かがある所だけを、背景を保った空のセルの上に描きます。エンコード済みの画像はキャッシュされ、レーン、スクロール位置、ズームが変わったときだけ再エンコードされ、下のセルが描き直されたときだけ再送されます。sixel には削除の手段がないので、GitPoulpi はそのセルの上に文字を書いて消します。sixel にはセルの実ピクセルサイズが必要です。Windows Terminal には起動時とズームの後に一度だけ尋ねます（`CSI 16 t`、約 10 ms で応答、応答がなくてもブロックしません）。他の環境ではウィンドウサイズか `GITPOULPI_TERMINAL_CELL_PX=10x20` から得ます。sixel を描画できるのに認識されないターミナル（`terminal.integrated.enableImages` を有効にした VS Code ターミナル、`-ti vt340` で起動した xterm）では `ui.image_protocol = "sixel"` が必要です。1.22 より前の Windows Terminal は sixel を描画しないので、そこでは `ui.image_protocol = "none"` に設定してください（グラフは文字のままです）。
- **トゥルーカラー**：テーマは RGB です。`ui.color_depth = "auto"` は、`COLORTERM=truecolor` のない
  `*-256color` の `TERM`（と Terminal.app）では 256 色に、Linux コンソールでは 16 色に下げます。
  kitty、WezTerm、Windows Terminal はトゥルーカラーとして扱われます。
- **マウス**：ボタン、ドラッグ、移動を伴う SGR マウスレポート（ホバーの表示、ドラッグ＆ドロップ、
  右クリック、ダブルクリック）。
- **キーボード**：**kitty キーボードプロトコル**（disambiguate フラグのみ）を、起動時に 100 ms の
  タイムアウトつきで要求します。これにより `Ctrl+,`、`Ctrl+Enter`、`Ctrl+Shift+N`、
  `Ctrl+Shift+Z` が届きます。ないと従来のキーのままになります（`Ctrl+Y` でやり直し、
  一部のショートカットは `Alt` の別形に置き換わります）。
  [キーバインド](keybindings.md#ターミナルが送れないキーと-kitty-キーボードプロトコル)を参照してください。
- **クリップボード**：コピー操作は `OSC 52` を書き込みます。
- **マウスポインターの形**（区切り線の上でのリサイズ用ポインター、クリックできる項目の上での手の形）：
  `OSC 22`。実装されている場合のおまけの機能です。
- **フォーカスイベント**は有効です（ウィンドウがフォーカスを失うとツールチップが閉じます）。
  GitPoulpi は、ブラケットペースト、OSC 8 ハイパーリンク、アンダーカールを要求しません。
  画像を使う経路に限り、各フレームを同期更新（モード 2026。知らないターミナルは無視します）で
  包み、ポップアップの上に画像が一瞬映り込まないようにしています。
- **絵文字と記号**：GitPoulpi は、既定が絵文字表示のグリフか、幅が 1 の BMP の図形だけを使います。
  ターミナルとフォントの間で異体字セレクターの扱いが食い違うためです。

## 機能の一覧表

「画像」は、そのターミナルで GitPoulpi が現在実際に描画するもの（**kitty**、**iTerm2**、**sixel** のいずれかのプロトコル、
あるいは**文字**）を指し、ターミナルにできることではありません。

| ターミナル | OS | 画像（グラフ、アバター、画像 diff） | トゥルーカラー | kitty キーボード（すべてのキー） | OSC 52 コピー | OSC 22 ポインター |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux、macOS | kitty | はい | はい | はい | はい |
| **Ghostty** (1.3) | Linux、macOS（Windows 版なし） | kitty | はい | はい | はい | はい（1.0 以降） |
| **WezTerm** | Windows、Linux、macOS | iTerm2、透過あり | はい | `enable_kitty_keyboard = true` のときだけ（既定は `false`） | はい（書き込みのみ） | いいえ（プルリクエストはまだオープン） |
| **foot** (1.28) | Linux（Wayland） | グラフもアバターも sixel（kitty グラフィックスなし） | はい | はい（1.10.3 以降） | はい（`security.osc52`） | はい（1.12 以降） |
| **Alacritty** (0.17) | すべて | 文字（本体に画像サポートなし） | はい | はい（0.13 以降） | はい（書き込み。貼り付けは既定でオフ） | いいえ（未実装） |
| **Konsole** | Linux | グラフもアバターも sixel | はい | 2026年に追加中（マージリクエスト !1188）。リリースは（未確認） | 書き込みのみ、2024 年半ばにマージ済み | （未確認） |
| **GNOME Terminal、Ptyxis**（VTE） | Linux | 文字 | はい | いいえ（リリース済みの VTE にない） | いいえ（VTE が無視する） | いいえ |
| **iTerm2** (3.7) | macOS | iTerm2、不透明な背景 | はい | はい（3.6.9 以降。Profiles > Keys を確認） | 既定でオフ：「Applications in terminal may access clipboard」を有効にする | （未確認） |
| **Terminal.app** | macOS | 文字 | いいえ（256 色） | いいえ | （未確認） | （未確認） |
| **Windows Terminal** (1.25) | Windows | グラフもアバターも sixel | はい | はい（安定版は 1.25 以降、2026年10月。Preview は 1.25 から、2026年3月） | はい（書き込み。フォーカス中のみ） | いいえ（未実装） |
| **Rio** | Windows、Linux、macOS | iTerm2、不透明な背景（kitty グラフィックスも話せる） | はい | はい（既定でオン） | はい | いいえ（未実装） |
| **Warp** | macOS、Linux、Windows | 文字（検出されない） | はい | はい | はい（シーケンスを受け付ける） | （未確認） |
| **VS Code ターミナル**（xterm.js） | すべて | 文字（sixel と iTerm2 画像は `terminal.integrated.enableImages` の裏にあります。`ui.image_protocol = "sixel"` で sixel を使えます） | はい | （未確認） | （未確認） | いいえ |
| **mintty**（Cygwin、Git Bash） | Windows | iTerm2、不透明な背景 | はい | （未確認。ドキュメントなし） | （未確認。ドキュメントなし） | はい |
| **Windows コンソール**（conhost、cmd、PowerShell のウィンドウ） | Windows | 文字 | 最近の Windows 10/11 のみ | 不要：修飾キーを自分で報告する | いいえ | いいえ |
| **ConEmu** | Windows | 文字 | （未確認） | （未確認） | （未確認） | （未確認） |
| **tmux、zellij、screen** | すべて | 文字（常に） | 設定が必要 | 設定が必要 | 設定が必要 | いいえ |

OSC 22 の「いいえ（未実装）」のセルは、プロジェクトの変更履歴とドキュメントに記載がないことと
ユーザーからの報告に基づくもので、テストしたものではありません。「ドキュメントなし」と読み替えてください。
絵文字と異体字セレクターの幅は、このページでは表にしていません。ターミナルとフォントごとに異なるためで、
GitPoulpi が VS16 のグリフを避けているのもそのためです（[GitPoulpi が使うもの](#gitpoulpi-が使うもの)を参照）。

## ターミナルごとの注意

### Windows

- **WezTerm.** Windows で最も優れた画像サポートです。`ui.image_protocol` は `auto` のままにします。
  `wezterm.lua` で、すべてのキーを使うために `config.enable_kitty_keyboard = true` を設定し、
  絵文字のフォールバックを持つフォントを指定してください。たとえば
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`。
  WezTerm の最後のタグ付き安定版は 20240203-110809-5046fc22（2024年2月）で、新しい修正は
  ナイトリービルドで提供されます。WezTerm にはまだ OSC 22 のポインター形状がありません。
- **Windows Terminal.** トゥルーカラー、マウス、OSC 52、きれいな絵文字に対応します。GitPoulpi は
  アバターを sixel 画像で描画し（Windows Terminal 1.22 以降）、グラフは文字です。
  *Settings > Profiles > Defaults > Appearance > Font face* で Nerd Font を選んでください。
  マウスがキャプチャされている間にテキストを選択するには `Shift` を押したままにします。
  ポインターの形は変わりません。1.25 より古いビルド（kitty キーボードプロトコルなし）では、
  `Alt` の別形を使ってください。
- **Rio** は不透明な画像の背景つきで動作します。検証は少なめです。
- **mintty**（Git Bash、MSYS2、Cygwin）は、iTerm2 プロトコルと不透明な背景で画像を描画します。
  検証は少なめです。
- **Windows コンソール、ConEmu、cmd**：避けてください。画像が正しく描画されないか、描画されません。
  exe をダブルクリックすると、選んだターミナルで GitPoulpi が開き直されます
  （*設定 > 一般 > ランチャー*）。

### Linux

- **kitty.** 設定なしですべて動きます。`Shift+クリック` は kitty 自身の操作なので、
  選択範囲の拡張には `Alt+クリック` を使ってください（[マウスとジェスチャー](mouse-and-gestures.md)）。
- **Ghostty.** 設定なしですべて動きます。アイコンが穴になって表示される場合は、`font-family` に
  絵文字フォントか Nerd Font を追加してください（キーを繰り返すとフォールバックが増えます）。
- **WezTerm.** iTerm2 プロトコルで画像を表示し、透過を合成します。`enable_kitty_keyboard` の
  注意は Windows と同じです。
- **foot.** 文字のグラフで、すべてのキーを送れて高速です。アバターは sixel 画像で、
  グラフは文字です。`foot.ini` に `font=JetBrainsMono Nerd Font:size=11` を設定してください。
- **Alacritty.** 文字のグラフで、kitty キーボードは 0.13 以降です。Alacritty には画像サポートが
  ありません。
- **Konsole、GNOME Terminal、Ptyxis（VTE）.** 文字と従来のキーで動作します。`Alt` の別形と、
  やり直しの `Ctrl+Y` を使ってください（新しい Konsole は kitty キーボードプロトコルに対応する
  かもしれません）。VTE ではコピー操作（`OSC 52`）がクリップボードに何も入れません。
  入力メソッド（IBus、Fcitx）が、ステージ切り替えの `Ctrl+Space` を奪うことがあります -
  **（未確認）**。*設定 > キーボードとマウス* で割り当てを変えてください。
- **Wayland、Hyprland、Omarchy.** マウスポインターのヒント（`OSC 22`）は kitty、Ghostty、
  foot で働きます。Omarchy のランチャーのエントリは `xdg-terminal-exec` を経由します。

### macOS

- **kitty、Ghostty、WezTerm**：Linux と同じです。
- **iTerm2.** 不透明な背景つきの画像です（背景色がテーマと違うと四角形が見えます。
  `ui.graph_renderer = "text"` で消えます）。コピーには
  *Settings > General > Selection > Applications in terminal may access clipboard* を、
  `Alt` のショートカットには *Profiles > Keys > Left Option key: Esc+* を有効にしてください。
- **Terminal.app.** 256 色のみです。GitPoulpi は自分で `ui.color_depth = 256` を選び、
  テーマはやや平板に見えます。画像はありません。`Alt` のショートカットには
  *Use Option as Meta key* をオンにしてください。

### VS Code と Warp

VS Code ターミナル（`TERM_PROGRAM=vscode`）と Warp は画像対応のターミナルとして認識されないため、
GitPoulpi は文字で描画します。どちらも使えます。VS Code では `terminal.integrated.enableImages` を有効にして
`ui.image_protocol = "sixel"` を設定すると、アバターが画像になります。Warp は kitty キーボードプロトコルへの対応を
明記していますが、VS Code ターミナルの対応は **（未確認）** です。

## マルチプレクサ

GitPoulpi は tmux、zellij、screen、herdr（`TMUX`、`ZELLIJ`、`STY`、`TERM=screen*` または
`tmux*`、`HERDR_*`）を検出し、**文字**で描画します。マルチプレクサが転送するのはセルであって、
画像のエスケープシーケンスではないからです。ほかの機能のために、`~/.tmux.conf` に次を書きます。

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g extended-keys on                # full chords (tmux 3.2 or later)
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later; only if you force pictures, see below
```

画像を転送するマルチプレクサ越しに画像を強制するには、`ui.image_protocol` を `kitty` または
`iterm2` にします。kitty、Ghostty、WezTerm の中で `kitty_graphics = true`
にした herdr は自動的に検出されます。詳しくは
[トラブルシューティング](troubleshooting.md#グラフが画像ではなく文字で描かれるherdrtmuxそのほかのターミナル)を参照してください。
`GITPOULPI_TERMINAL_IMAGE=off` ですべての画像をオフにできます。

tmux 自身は kitty キーボードプロトコルを転送しません。`extended-keys` を使うと、修飾キー付きのキーを
`CSI u` または `xterm` のシーケンスとして送ります。zellij 0.41 以降は、変更履歴によれば
kitty キーボードプロトコルを話し、OSC 52 を転送します。

## 機能が欠けたときの動作

| 欠けているもの | 代わりに得られるもの |
|---|---|
| 画像プロトコル | 文字のグラフ。アバターは、`ui.avatar_fallback` の設定にかかわらず、色付きのイニシャルのバッジ（1 文字）になります。文字のセルに押し込んだアイデンティコンは見分けがつかないためです。画像 diff とファイルビューは、色付きのハーフブロックか情報カードを使います。 |
| トゥルーカラー | 色は 256 色または 16 色のパレットに割り当てられます（`ui.color_depth`）。 |
| kitty キーボードプロトコル | 従来のキー。一部のショートカットは別のキーとして届きます。設定が警告するので、割り当てを変えるか `Alt` の別形を使います。 |
| OSC 52 | コピー操作はクリップボードに何も入れません。マウスで選択し（`Shift` を押したまま）、ターミナル自身のコピーを使ってください。 |
| OSC 22 | ポインターは変わりません。マウスの下の区切り線やボタンは引き続き点灯します。 |
| 絵文字フォント | アイコンの代わりに四角や穴が表示されます。グラフとテキストは動作します。Nerd Font か絵文字フォントをインストールし、フォールバックとして指定してください。 |

## 知っておきたい設定

| 設定 | 用途 |
|---|---|
| `ui.image_protocol` | `auto`（既定）、`kitty`、`iterm2`、`sixel`、`none`。画像があるはずの場所に文字化けが出る場合は `none` にします。再起動が必要です。 |
| `ui.graph_renderer` | `auto`、`image`、`text`。`text` は常に正しく表示されます。再起動が必要です。 |
| `ui.color_depth` | `auto`、`truecolor`、`256`、`16`。 |
| `ui.kitty_keyboard` | ターミナルやマルチプレクサの動作がおかしい場合に、`auto` または `off`。再起動が必要です。 |
| `ui.mouse_enabled` | オフにすると、マウスはターミナルのものになります。再起動が必要です。 |
| `ui.launch_terminal` | OS のショートカットが開くターミナル（[設定](settings.md)）。 |

あわせて[画像](image-view.md)、[キーバインド](keybindings.md)、
[トラブルシューティング](troubleshooting.md)も参照してください。

## 出典

2026年10月に確認しました。特記のないものは一次情報です。

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html)。kitty キーボードは [terminfo.dev matrix](https://terminfo.dev/extensions/kitty-keyboard-protocol) から（二次情報）
- Windows Terminal: [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/)（二次情報）, [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps)。OSC 52 は [Warp OSC 52 advisory write-up](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) から（二次情報）
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app（トゥルーカラーなし）: [Apple discussion](https://discussions.apple.com/thread/255315820)（二次情報）
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES)。zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- 比較サイト（二次情報。照合にのみ使用）: [Terminal Trove](https://terminaltrove.com/compare/terminals/)

未確認のもの：Konsole と mlterm の sixel（各プロジェクト自身の機能一覧によるもので、リリースでは再確認しておらず、どのターミナルでも実際のウィンドウでは試していません）。Konsole、iTerm2、Terminal.app、Warp の OSC 22。Terminal.app、VS Code、mintty の OSC 52。
VS Code、mintty、ConEmu の kitty キーボードプロトコル。ConEmu のすべてのセル。ターミナルごとの絵文字の幅。
Konsole が kitty キーボードプロトコルを搭載するリリースは確認できませんでした。
