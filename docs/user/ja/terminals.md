[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · **日本語** · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# ターミナルの選び方

GitPoulpi はマウスに対応したどのターミナルでも動きますが、画像を描画でき、トゥルーカラーを表示でき、
キーボードのすべてのキーを送れるターミナルで最もよく動作します。このページでは、OS ごとに
それぞれのターミナルで何が得られるか、どの設定を変えるかを説明します。

**読み方。** 表は **2026年11月** のテストの結果です。すべてのターミナルを、生のエスケープシーケンスで、
そして実行できるものでは GitPoulpi そのもので、Linux、Windows 11、macOS 15 の上で動かしました。バージョンは
[下](#テスト済みのバージョン)にあります。各セルは次のどれかです。

- **動作する**
- **一部動作**（何が欠けているか）
- **動作しない**（理由：*ターミナル*、*OS の層*、*GitPoulpi* のどれか）
- **n/a**（その OS にそのターミナルがない）
- **未測定**（理由）

マルチプレクサ（tmux、zellij、screen、herdr、psmux、ペイン）には専用のページがあります：
[マルチプレクサ](multiplexers.md)。

## おすすめ

| OS | おすすめ（順位順） | 理由 |
|---|---|---|
| **Windows** | **Windows Terminal**、次に **WezTerm** と **Rio** | Windows Terminal はグラフとアバターを sixel 画像として、正しい位置に描画し、ランチャーの既定です。WezTerm と Rio は iTerm2 画像として描画し、2026年11月の ConPTY の修正以降は正しい位置に置かれます（[Windows](#windows)を参照）。 |
| **Linux**（X11、Wayland、Hyprland、Omarchy） | **kitty**、**Ghostty**、**WezTerm**、**foot**、**Rio** | kitty と Ghostty は画像、ポインターの形、すべてのキーを備えます。foot は sixel 画像を描画し、すべてのキーとポインターの形に対応します。 |
| **macOS** | **iTerm2**、次に **kitty** と **Ghostty** | iTerm2 3.7.7 は kitty プロトコルで画像を描画します。Terminal.app は動作しますが、画像もトゥルーカラーもありません。kitty と Ghostty は Linux と同じ動作です（macOS では未測定）。 |

どの OS でも、**screen** と **zellij** はターミナルの画像プロトコルを GitPoulpi から隠すため、
GitPoulpi は文字で描画します。tmux は sixel を通します。[マルチプレクサ](multiplexers.md)を参照してください。

## GitPoulpi が使うもの

- **画像**（グラフ列、作成者のアバター、画像 diff とファイルビュー）：Unicode プレースホルダーを使う
  **kitty グラフィックスプロトコル**、**iTerm2 インライン画像**（`OSC 1337`）、**sixel** のいずれかです。
  検出は「拒否が既定」で、`TERM`、`TERM_PROGRAM`、`KITTY_WINDOW_ID`、`GHOSTTY_RESOURCES_DIR`、
  `WEZTERM_EXECUTABLE`、`WT_SESSION`、`KONSOLE_VERSION` とマルチプレクサの環境変数から判断します。
  環境変数がどのターミナルも示さない場合（`-ti vt340` で起動した xterm、画像をオンにした VS Code ターミナル、
  `TERM_PROGRAM` が転送されない SSH 越しのターミナル）は、ターミナル自身の応答で決めます。GitPoulpi は起動時に
  一度だけ問い合わせ（下の「起動時の問い合わせ」を参照）、ターミナルの XTVERSION の名前（kitty、Ghostty、
  WezTerm、iTerm2、mintty、foot）か、デバイス属性の sixel を使います。マルチプレクサの中では応答は無視されます。
  tmux は背後に何があっても sixel に対応していると答えるからです。
- **どこでどのプロトコルか。** kitty、Ghostty、iTerm2 3.7.7 以降：kitty プロトコル。WezTerm と Rio：PNG の透過を
  合成する iTerm2 プロトコル。mintty と 3.7.7 より古い iTerm2：不透明な背景つきの iTerm2 プロトコル。
  Windows Terminal、foot、Tabby、Konsole、mlterm：**sixel**。それ以外：文字で、アバターはイニシャルのバッジになります。
  `ui.image_protocol` でどれかを強制できます。
- **iTerm2 3.7.7。** `OSC 1337` のインライン画像はどれも、モーダルの「Allow Terminal-Initiated Display?」ダイアログを
  開き、「Remember my choice」にチェックを入れるまでウィンドウをブロックします。同じ画像を kitty プロトコル
  （Unicode プレースホルダーを含む）で送ると、質問なしで描画されるため、GitPoulpi はそこでは kitty プロトコルを
  使います。`ui.image_protocol = "iterm2"` にすると、引き続きインライン画像を使えます。
- **Sixel** は**グラフ列**（レーン、ノード、`ui.avatars = "graph"` ならコミットの丸の中のアバター）と、
  それ以外の場所の**アバター**（ログ、commit の詳細、ファイル履歴、Blame、画像ビュー）に描かれます。sixel には
  アルファも ID も拡大縮小もないため、画像は `列数 x セル幅` x `行数 x セル高` ピクセルちょうどで、何かがある所だけを、
  背景を保った空のセルの上に描きます。再エンコードされるのは、レーン、スクロール位置、ズームが変わったときだけです。
  sixel には削除の手段がないので、GitPoulpi はそのセルの上に文字を書いて消します。sixel にはセルの実ピクセルサイズが
  必要です。Windows Terminal、Tabby、mintty は、起動時とズームの後に一度だけ `CSI 16 t` に応答します（応答がなくても
  ブロックしません）。他の環境ではターミナルのウィンドウサイズか `GITPOULPI_TERMINAL_CELL_PX=10x20` から得ます。
  sixel を描画できるのに自分でそう名乗らないターミナルには `ui.image_protocol = "sixel"` が必要です。
  1.22 より前の Windows Terminal は sixel を描画しないので、そこでは `ui.image_protocol = "none"` に設定してください。
- **トゥルーカラー**：テーマは RGB です。`ui.color_depth = "auto"` は、`COLORTERM=truecolor` のない
  `*-256color` の `TERM`（と、トゥルーカラーのない Terminal.app）では 256 色に、Linux コンソールでは 16 色に下げます。
- **マウス**：ボタン、ドラッグ、移動を伴う SGR マウスレポート（ホバーの表示、ドラッグ＆ドロップ、
  右クリック、ダブルクリック）。
- **起動時の問い合わせ。** 4 つの質問を 1 回の書き込みで送ります。kitty キーボードのフラグ（`CSI ? u`）、背景色
  （`OSC 11`。`ui.theme = system` のときだけ使用）、ターミナルの名前（XTVERSION、`CSI > 0 q`）、デバイス属性
  （`CSI c`。最後に置くのは、どのターミナルも応答するので、これで読み取りが終わるからです）。読み取りは 100 ms の
  期限つきの `poll` ループ 1 つで、キーボードの読み取りが始まる前に終わります。何も応答しないターミナルでは、
  100 ms を一度使うだけで、あとは環境変数のとおりになります。Unix のみ：Windows ではコンソールがキーを自分で
  報告し、環境変数が決めます。応答は、アクティビティログの環境レポートの `start-up query:` 行にあります。
- **キーボード**：**kitty キーボードプロトコル**（disambiguate フラグのみ）を、起動時に要求します。
  これにより `Ctrl+,`、`Ctrl+Enter`、`Ctrl+Shift+N`、`Ctrl+Shift+Z` が届きます。ないと従来のキーのままになります
  （`Ctrl+Y` でやり直し、一部のショートカットは `Alt` の別形に置き換わります）。
  [キーバインド](keybindings.md#ターミナルが送れないキーと-kitty-キーボードプロトコル)を参照してください。
- **クリップボード**：すべてのコピー操作は 1 つの方針に従います。ローカルでは、GitPoulpi が動いているマシンの
  **システムのクリップボード**に加えて、ターミナルが対応していると分かっている場合（kitty、WezTerm、Ghostty、
  foot、Alacritty、Windows Terminal、Rio、Tabby、iTerm2）は `OSC 52` もターミナルへ送ります。システムの
  クリップボードが失敗したときも `OSC 52` がフォールバックです。**SSH 越しやディスプレイがない場合**（Linux コンソール、
  コンテナ）は `OSC 52` だけが送られ、tmux の中ではそのままの形とパススルー用に包んだ形の両方が送られます。
  ステータス行が「コピーしました」と表示するのは、システムのクリップボードがテキストを受け取ったときだけです。
  ターミナルへ送っただけの場合は「…をターミナルのクリップボードに送信しました（OSC 52、未確認）」と
  表示します。ターミナルが従ったかどうかを確認する手段がないためです。アクティビティログの環境レポートの
  `clipboard:` 行に経路が書かれます。
- **マウスポインターの形**（区切り線の上でのリサイズ用ポインター、クリックできる項目の上での手の形、テキスト欄の上での
  I ビーム）：`OSC 22`。ターミナルが実装していればおまけで効きます。GitPoulpi は常に送ります（Linux コンソールと、
  それを握りつぶすか文字として表示する screen、zellij、herdr の背後を除きます。tmux の背後ではパススルー用に包みます）。
  `TERM_PROGRAM=mintty` では、mintty 自身のカーソル名（`hand`、`ibeam`、`sizewe` など）を送ります。mintty が読むのは
  それだけだからです。**Rio はどの OS でも `OSC 22` を実装していません**：ソースがシーケンスを解析して捨てるので、
  どちら側の設定を変えても変わりません。
- **テキストのキャレット**：`ui.text_cursor = "terminal"`（既定）では、フォーカス中のテキスト欄のキャレット位置に
  ターミナル自身のカーソルが点滅するバーとして置かれ（`DECSCUSR`）、それ以外では非表示になります。終了時には
  あなたのカーソル形状に戻ります。`drawn` にすると、代わりに GitPoulpi 独自のキャレットを描きます。
- **フォーカスイベント**は有効です（ウィンドウがフォーカスを失うとツールチップが閉じます）。
  **ブラケットペースト**は起動時に要求され、終了するあらゆる経路で解除されます（Unix）。そのため、複数行の貼り付けは
  1 回の貼り付けとして届き、改行でフィールドが確定されることも、文字がショートカットとして動くこともありません。
  Windows のコンソールにはペーストのブラケットがありません。代わりに GitPoulpi が何をするかは
  [Windows](#windows)を参照してください。GitPoulpi は、OSC 8 ハイパーリンクとアンダーカールを要求しません。
  画像を使う経路に限り、各フレームを同期更新（モード 2026。知らないターミナルは無視します）で
  包み、ポップアップの上に画像が一瞬映り込まないようにしています。
- **絵文字と記号**：GitPoulpi は、既定が絵文字表示のグリフか、幅が 1 の BMP の図形だけを使います。
  ターミナルとフォントの間で異体字セレクターの扱いが食い違うためです。

## 一覧表

行はターミナル、列は OS です。セルの「GitPoulpi」は GitPoulpi 自体をそこで動かしたこと、「probe」は生のエスケープ
シーケンスだけを試したことを意味します。**n/a** のセル：kitty には Windows 用のビルドがありません（リリースは
Linux と macOS）。Ghostty には Windows 用のビルドがありません（クロスプラットフォームなのはライブラリの libghostty だけ）。
foot は Wayland のターミナルです（Linux のみ）。xterm は X11 のクライアントです（Linux と、XQuartz 経由の macOS）。
iTerm2 と Terminal.app は macOS のみ。Windows Terminal、conhost、mintty は Windows のみです（mintty は Cygwin と
MSYS2 でも動きます）。

**macOS で未測定：kitty、Ghostty、WezTerm、Rio、Warp。** GPU（Metal または OpenGL 3.3）が必要です。同じプログラムが
何をするかは Linux の結果が基準になります。macOS のセルを当てにする前に、GPU のある Mac で再測定してください。

### 画像：GitPoulpi がどのプロトコルを選び、表示されるか

| ターミナル | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **動作する**：kitty プロトコル（GitPoulpi） | n/a | 未測定（GPU が必要） |
| **Ghostty** | **動作する**：kitty プロトコル（GitPoulpi） | n/a | 未測定（GPU が必要） |
| **WezTerm** | **動作する**：透過つきの iTerm2 プロトコル（GitPoulpi）。プレースホルダーは潰れて描画されるため、kitty プロトコルは使いません | **動作する**：透過つきの iTerm2 プロトコルで、グラフとアバターが正しい行に出ます（公式 20240203 と、アップストリームの PR #8092 のビルド）。公式 20240203 は、GitPoulpi が画像の前後でカーソルを表示するようにする前は、画像が違う行に出ていました。「ConPTY の限界」を参照 | 未測定（GPU が必要） |
| **foot** | **動作する**：透過つきの sixel（GitPoulpi） | n/a | n/a |
| **Rio** | **動作する**：透過つきの iTerm2 プロトコル（GitPoulpi） | **動作する**：透過つきの iTerm2 プロトコルで、試したどのフォントサイズでもセル単位で正確（0.5.28、GitPoulpi）。sixel と kitty は届かない（ConPTY が落とす）ので、選ぶものはありません | 未測定（GPU が必要） |
| **Alacritty** | 文字：**動作する**（本体に画像サポートなし） | 文字：**一部動作**（画像なし。画像 diff はブロック表示） | 文字：**動作する** |
| **Tabby** | **動作する**：透過つきの sixel（probe。Linux の Tabby では GitPoulpi は未実行） | **動作する**：位置の合った sixel（GitPoulpi） | **動作する**：透過を保った sixel（GitPoulpi） |
| **Warp** | 未測定（XWayland が必要で、合成したキー入力を誤って解釈するため、何も入力できなかった） | **動作しない**（ターミナル：sixel、iTerm2、kitty の画像なし）。GitPoulpi は未測定（Warp の入力欄から起動できなかった） | 未測定（GPU が必要） |
| **xterm** | probe：`-ti vt340` で sixel は**動作する**。GitPoulpi はデバイス属性から選ぶ（起動時の問い合わせは実行後に追加）。または `ui.image_protocol = "sixel"` を設定 | n/a | 文字：**動作する**。sixel は描画される（probe）が、`ui.image_protocol = "sixel"` が必要 |
| **iTerm2** | n/a | n/a | **動作する**：3.7.7 以降は kitty プロトコルで、ダイアログなし（GitPoulpi）。古いバージョンは不透明な背景のインライン画像。sixel は描画されるが、未設定のピクセルを不透明に塗るため、`ui.image_protocol = "sixel"` では塗りつぶしの長方形になる |
| **Terminal.app** | n/a | n/a | 文字：**動作する**（画像なし） |
| **Windows Terminal** | n/a | **動作する**：sixel（GitPoulpi、1.25） | n/a |
| **conhost** | n/a | 文字：**一部動作**（文字のグラフ、半ブロックの画像 diff） | n/a |
| **mintty** | n/a | **動作しない**（ConPTY の背後のターミナル：カーソルを表示して置いた単純な `OSC 1337` の画像でも、画像がまったく描画されない。グラフ列は空白のまま。文字とパレットは問題なし） | n/a |

### マウスポインターの形（OSC 22）

| ターミナル | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **動作する** | n/a | 未測定（GPU が必要） |
| **Ghostty** | **動作する** | n/a | 未測定（GPU が必要） |
| **WezTerm** | **動作しない**（ターミナル：テストしたビルドでは未実装） | 公式：**動作しない**（ターミナル）。ポインターの形を追加するアップストリームの WezTerm PR #8092 のビルド：**動作する**（手、I ビーム、リサイズ矢印） | 未測定（GPU が必要） |
| **foot** | **動作する** | n/a | n/a |
| **Rio** | **動作しない**（ターミナル：解析して破棄） | **動作しない**（ターミナル、同じ理由。ConPTY のせいではない） | **動作しない**（ターミナル、同じソース。実行したのは Linux と Windows のみ） |
| **Alacritty** | **動作しない**（ターミナル） | **動作しない**（ターミナル） | **動作しない**（ターミナル） |
| **Tabby** | **動作しない**（ターミナル：xterm.js） | **動作しない**（ターミナル） | **動作しない**（ターミナル） |
| **Warp** | 未測定 | **動作しない**（ターミナル） | 未測定 |
| **xterm** | **動作しない**（ターミナル：CSS の名前を知らない） | n/a | 未測定（ポインターがウィンドウの外にあった） |
| **iTerm2** | n/a | n/a | **動作しない**（ターミナル） |
| **Terminal.app** | n/a | n/a | **動作しない**（ターミナル） |
| **Windows Terminal** | n/a | **動作しない**（ターミナル） | n/a |
| **conhost** | n/a | **動作しない**（ターミナル） | n/a |
| **mintty** | n/a | mintty 自身の名前で**動作する**。GitPoulpi がそれを送る | n/a |

セルが「動作しない」でも、マウスの下にある区切りやボタンは引き続き点灯します。

### キーボードプロトコル

GitPoulpi は Linux と macOS で kitty キーボードプロトコルを要求します。Windows では要求しません。コンソールが
修飾キーを自分で報告し、下のキーの組み合わせは、ターミナルが自分用に取っておくものによって制限されます。

| ターミナル | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **動作する** | n/a | 未測定（GPU が必要） |
| **Ghostty** | **動作する** | n/a | 未測定（GPU が必要） |
| **WezTerm** | **一部動作**：`wezterm.lua` で `enable_kitty_keyboard = true` にするまで応答なし（既定は `false`） | **一部動作**：WezTerm が `Ctrl+Shift+N`（自分のウィンドウを開く）を取る。`Ctrl+,` は届かない（公式ビルド）。PR #8092 のビルドはどちらの組み合わせも取らないが、`Ctrl+,` で設定が開かない（ConPTY 経由では kitty キーボードの応答がない） | 未測定（GPU が必要） |
| **foot** | **動作する** | n/a | n/a |
| **Rio** | **動作する**（既定でオン） | **一部動作**：Rio が `Ctrl+Shift+N`（自分のウィンドウを開く）を取る | 未測定（GPU が必要） |
| **Alacritty** | **動作する**（0.13 以降） | **一部動作**：`Ctrl+Shift+N` は `Ctrl+N` として届き、`Ctrl+,` は届かない | **動作する** |
| **Tabby** | **動作しない**（ターミナル：応答なし。`Ctrl+Shift+N` と `Ctrl+,` は自分用に取る） | **一部動作**：Tabby が `Ctrl+Shift+N` と `Ctrl+,` を取る | **動作しない**（ターミナル：応答なし）。届かない組み合わせがあった |
| **Warp** | 未測定 | 未測定 | 未測定（GPU が必要） |
| **xterm** | **動作しない**（ターミナル） | n/a | **動作しない**（ターミナル） |
| **iTerm2** | n/a | n/a | **動作する**（3.7.7。動作しない場合は *Profiles > Keys* を確認） |
| **Terminal.app** | n/a | n/a | **動作しない**（ターミナル：応答なし。従来のキー） |
| **Windows Terminal** | n/a | **一部動作**：WT が `Ctrl+Shift+N`（新しいウィンドウ）と `Ctrl+,`（自分の設定）を取る | n/a |
| **conhost** | n/a | 未測定 | n/a |
| **mintty** | n/a | **一部動作**：どちらの組み合わせも目に見える効果はなかった | n/a |

組み合わせが届かない場合は、`Alt` の別形を使うか
（[キーバインド](keybindings.md#ターミナルが送れないキーと-kitty-キーボードプロトコル)）、
*設定 > キーボードとマウス*でその組み合わせを割り当て直してください。

### クリップボード（コピー操作）

ローカルのディスプレイがあれば、コピーは常にシステムのクリップボードを使います。そのため、これらのセルは
ターミナルの `OSC 52` の経路を表しており、これは SSH 越し、コンテナ、フォールバックのときに意味があります。
システムのクリップボードを使った GitPoulpi のコピーのテストは、すべて動作しました。

| ターミナル | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **動作する** | n/a | 未測定（GPU が必要） |
| **Ghostty** | **動作する**（tmux、zellij、herdr の背後で確認）。単独では未測定（クリップボードの書き込みにはキーボードフォーカスが必要で、テストでは与えなかった） | n/a | 未測定（GPU が必要） |
| **WezTerm** | **動作する** | **動作する** | 未測定（GPU が必要） |
| **foot** | 未測定（書き込みにはキーボードフォーカスが必要で、テストでは与えなかった） | n/a | n/a |
| **Rio** | 未測定（同じ理由） | **動作する** | 未測定（GPU が必要） |
| **Alacritty** | 未測定（同じ理由） | **動作しない**（既定の設定ではクリップボードが変わらない。調査していない） | **動作する** |
| **Tabby** | **動作する** | **動作する** | **動作する** |
| **Warp** | 未測定 | **動作しない**（クリップボードが変わらない） | 未測定（GPU が必要） |
| **xterm** | 未測定 | n/a | **動作しない**（クリップボードが変わらない） |
| **iTerm2** | n/a | n/a | *Settings > General > Selection > Applications in terminal may access clipboard* をオンにするまで**動作しない** |
| **Terminal.app** | n/a | n/a | **動作しない**（ターミナル） |
| **Windows Terminal** | n/a | **動作する**（フォーカス中） | n/a |
| **conhost** | n/a | 未測定（probe は実行していない） | n/a |
| **mintty** | n/a | **動作しない**（既定の設定ではクリップボードが変わらない） | n/a |

### 貼り付け

Linux と macOS では、GitPoulpi はブラケットペーストを要求するので、複数行の貼り付けは 1 回の貼り付けになります。
これはターミナルごとには実行していません（ターミナルごとに**未測定**）。Windows のコンソールにはペーストの
ブラケットがないため、先に要求しないターミナルはテキストをキー入力として届け、改行ごとに `Enter` になってしまいます。
そこで GitPoulpi は、30 ms 以内に続くキー入力（内側の `Enter`、または 8 文字以上）を 1 回の貼り付けとして扱います。
押しっぱなしのキーや、手で打った「abc、`Enter`」は貼り付けではありません。

| ターミナル | Windows（新しいブランチの入力欄に 2 行を貼り付け） |
|---|---|
| **Windows Terminal** | **動作する**：WT が先に「複数行を貼り付けますか？」と尋ねる（WT 自身の警告）。その後、ブランチは作られない |
| **WezTerm**、**Rio**、**Alacritty**、**Tabby**、**mintty** | **動作する**（ブランチは作られない） |
| **Warp** | 未測定（GitPoulpi を Warp の中で起動できなかった） |
| **conhost** | 未測定（テストが送った `Ctrl+V` と `Shift+Insert` では、従来のコンソールに貼り付けられなかった） |

### 絵文字とアイコンの幅

| ターミナル | Linux | Windows | macOS |
|---|---|---|---|
| **kitty**、**Ghostty**、**WezTerm**、**foot**、**Rio**、**Alacritty** | **動作する**（全角セルが揃う。kitty は歯車 U+2699 を空白で描くが、これはフォントの問題として知られている） | WezTerm、Rio、Tabby、Windows Terminal：**動作する**。Alacritty：**一部動作**（絵文字が単色の輪郭になる） | Alacritty、Tabby：**動作する**。ほかは未測定（GPU が必要） |
| **Tabby** | **動作する** | **動作する** | **動作する** |
| **Warp** | 未測定 | **動作する** | 未測定 |
| **xterm** | **一部動作**（四角になる：フォントのせい） | n/a | **一部動作**（四角になる：フォントのせい） |
| **iTerm2**、**Terminal.app** | n/a | n/a | **動作する** |
| **conhost** | n/a | **一部動作**：カラー絵文字は描画され、全角セルは揃う | n/a |
| **mintty** | n/a | **動作する** | n/a |

### テスト済みのバージョン

kitty 0.49.2（macOS の 0.50.0 は起動しなかった）、Ghostty 1.4.0、WezTerm は Linux で 20260716、Windows で
20240203（最後にタグが付いたリリース）、foot 1.28.0、Rio 0.5.28（Linux、Windows）と 0.5.29（macOS）、
Alacritty 0.17.0、Tabby 1.0.238、Warp 0.2026.11.04、xterm 411（Linux）と 378（XQuartz）、iTerm2 3.7.7、
Terminal.app 455.1（macOS 15.8）、Windows Terminal 1.25.2733、mintty 3.8.3。ターミナルの変化は速いので、
これより新しいバージョンでは異なる場合があります。

## ターミナルごとの注意

### Windows

- **Windows Terminal。** ランチャーの既定で、Windows で最も画像サポートが優れています。sixel のグラフとアバター、
  OSC 52、良好な絵文字、セルサイズの問い合わせへの応答があります。*設定 > プロファイル > 既定値 > 外観 > フォント*
  で Nerd Font を選んでください。マウスが取られているときにテキストを選択するには `Shift` を押したままにします。
  `Ctrl+Shift+N` と `Ctrl+,` は WT が取るので、`Alt` の別形を使うか割り当て直してください。ポインターの形は
  変わりません。
- **WezTerm。** `wezterm.lua` で `config.enable_kitty_keyboard = true`（Linux と macOS のビルド）と、絵文字の
  フォールバックのあるフォントを設定してください。たとえば
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }` です。
  画像は iTerm2 プロトコルを使い、何も設定する必要はありません。最後にタグが付いたリリース 20240203-110809
  （2024年2月）は、画像を正しい行に置きます（2026年11月に現在の GitPoulpi で再確認。下で説明する ConPTY の修正の前は
  そうではありませんでした）。アップストリームの WezTerm PR #8092 でポインターの形が追加されます。
- **Rio**（MSI、UAC が必要）。画像（グラフ、アバター、画像 diff）は透過つきの iTerm2 プロトコルを使い、セルにぴったり
  載ります。0.5.28 の既定のフォントと 18 px、22.5 px（セルは 8x16、11x21、13x26 ピクセルで、常に整数ピクセル）で測定し、
  20x10 セルと 4x27 セルの画像がどちらもセル単位で正確でした。何も設定する必要はありません。疑似コンソールが
  セルサイズの問い合わせを握りつぶすため、GitPoulpi は Rio のセルを幅の 2 倍の高さと仮定します（3 つのサイズすべてで
  成り立ちます）。別の比率のフォントでは、commit のマーカーを丸くするために `GITPOULPI_TERMINAL_CELL_PX=WxH`
  （たとえば `10x24`）を設定してください。Rio はどの OS でもポインターの形に対応しません。
- **Tabby。** sixel のグラフとアバター、コピーと貼り付けが動作します。Tabby は `Ctrl+Shift+N` と `Ctrl+,` を
  自分用に取ります。
- **Alacritty。** OpenGL 3.3 が必要です。文字のグラフ。`OSC 52` によるコピーは、既定の設定ではオンではありませんでした。
- **mintty**（Git Bash、MSYS2、Cygwin）。ポインターの形は動作します（mintty 自身の名前）。画像は表示されず
  （グラフ列は空白のまま。mintty 3.8.3 は ConPTY の背後では、単純な probe で置いた `OSC 1337` の画像すら描画しません）、
  そこではセルサイズの問い合わせにも応答しません：`ui.graph_renderer = "text"` を設定してください。
- **Warp。** GitPoulpi では未測定です。Warp の入力欄から起動できませんでした。このターミナルには sixel、iTerm2、
  kitty の画像がありません。
- **conhost、cmd、PowerShell のウィンドウ。** 文字のグラフ、半ブロックの画像 diff。既定では、そのようなウィンドウから
  起動した GitPoulpi は、選んだターミナルで自分を開き直します。「開き直しの規則」を参照してください。

**ConPTY の限界。** Windows は、どのターミナルのプログラムも疑似コンソール（ConPTY）の背後で動かします。
最近のものを同梱する Windows Terminal と WezTerm のビルドは、画像のシーケンスを通します。Windows に入っているもの
（Rio と mintty が使います）と、WezTerm 20240203 に同梱されているものは通しません。失われるもの：DCS（sixel）と
APC（kitty グラフィックス）のシーケンスは落とされるため、届くのは iTerm2 の画像（`OSC 1337`）だけです。これが
Rio と WezTerm がそれを使う理由です。`CSI 16 t`（セルサイズ）に応答するのは Windows Terminal と Tabby だけです。
修正されたこと：`OSC 1337` は ConPTY を即座に通り、ConPTY が最後に描いたカーソルの位置に置かれますが、ConPTY は
カーソルが見えている間しかカーソルを描きません。GitPoulpi はカーソルを非表示にしているため、画像が古い行に置かれていました
（フッターの上にグラフが出る、スクロール後に画面が乱れる）。現在は Windows で、書き込む画像ごとに、同じ書き込みの中で
その前後にカーソルを表示し、再び隠します。Rio と WezTerm 20240203 は、すべての画像をそのセルに置きます
（`pty_conpty_picture_anchor` が ConPTY の背後でのカーソルの動きを読み戻します）。Windows 以外と sixel では
何も変わりません。

**Windows の貼り付け。** 上の「貼り付け」の表を参照してください。GitPoulpi は連続したキー入力を 1 回の貼り付けに
まとめるため、テストしたターミナルでは、複数行の貼り付けで入力欄が確定されることはありません。

**開き直しの規則。** exe をダブルクリックすると、専用の従来のコンソールウィンドウ（conhost）で起動します。
`ui.launch_terminal` が別のターミナル（既定：Windows Terminal）を指していれば、GitPoulpi はそこで自分を開き直し、
最初のウィンドウは閉じます（`ui.relaunch_in_terminal`）。これは、従来のコンソールウィンドウの中で自分だけのときに
**限って**行われます。exe を直接実行するターミナル（`wt.exe gitpoulpi.exe`、WezTerm、Rio、Alacritty の `-e`、
Tabby の `run`）は、あなたが居たい場所なので、何も開き直されません。シェルから起動した場合は自分だけではないので、
やはり何も起きません。`ui.relaunch_in_terminal = false` でオフにできます。

### Linux

- **kitty。** 何も設定せずにすべて動作します。`Shift+クリック` は kitty 自身のものなので、選択範囲を広げるには
  `Alt+クリック` を使います（[マウス](mouse-and-gestures.md)）。
- **Ghostty。** 何も設定せずにすべて動作します。アイコンが穴になる場合は、`font-family` に絵文字か Nerd Font を
  追加してください（キーを繰り返すとフォールバックを追加できます）。
- **WezTerm。** iTerm2 プロトコルで画像、透過は合成されます。すべてのキーの組み合わせには
  `enable_kitty_keyboard = true` を設定してください。ポインターの形はありません。
- **foot。** sixel のグラフとアバター、ポインターの形、すべてのキー、高速です。`foot.ini` に
  `font=JetBrainsMono Nerd Font:size=11` を設定してください。`OSC 52` によるコピーは `security.osc52` で決まります。
- **Rio。** 透過つきの iTerm2 プロトコルで画像、すべてのキー。ポインターの形はありません（Rio は `OSC 22` を捨てます）。
- **Alacritty。** 文字のグラフ、0.13 以降は kitty キーボード。画像サポートはありません。
- **Tabby。** sixel 画像。kitty キーボードもポインターの形もありません（Electron、xterm.js）。`Ctrl+Shift+N` と
  `Ctrl+,` は自分用に取ります。
- **xterm。** sixel には `xterm -u8 -ti vt340` で起動してください（罫線に UTF-8 が必要です）。絵文字が四角になる
  のはフォントのせいです。
- **Warp。** 未測定：XWayland が必要で、テストでは合成したキー入力を誤って解釈しました。
- **Konsole、GNOME Terminal、Ptyxis（VTE）。** 2026年11月のテストの対象外です。文字と従来のキーで動作します：
  `Alt` の別形と、やり直しに `Ctrl+Y` を使ってください。VTE は `OSC 52` を無視するため、SSH 越しのコピーは
  クリップボードに届きません。入力メソッド（IBus、Fcitx）が `Ctrl+Space` のステージ切り替えを横取りすることが
  あります。*設定 > キーボードとマウス*で割り当て直してください。Konsole は、自身のドキュメントに基づいて sixel の
  対応が認識されます。
- **Wayland、Hyprland、Omarchy。** ポインターのヒント（`OSC 22`）は kitty、Ghostty、foot で動作します。Omarchy の
  ランチャーの項目は `xdg-terminal-exec` を通ります。

### macOS

- **iTerm2。** 3.7.7 以降は、質問なしで kitty プロトコルによる画像を描画します。役立つ設定：
  - *Settings > General > Selection > Applications in terminal may access clipboard*：
    `OSC 52` によるコピーに必要です（ないと、iTerm2 は「the terminal attempted to access the clipboard but it was
    denied」と表示します）。
  - *Profiles > Keys > Left Option key: Esc+*：`Alt` の組み合わせに必要です。
  - `ui.image_protocol = "iterm2"` に設定した場合（または 3.7.7 より古いバージョンの場合）は、
    「Allow Terminal-Initiated Display?」ダイアログで「Remember my choice」にチェックを入れてください。
    その場合、画像は不透明な背景を持ちます（背景がテーマの色と違うと長方形が見えます。
    `ui.graph_renderer = "text"` で消えます）。
  - kitty キーボードプロトコルは既定でオンです（3.6.9 以降）。
- **Terminal.app。** 256 色のみ：GitPoulpi は自分で `ui.color_depth = 256` を選ぶので、テーマは平板に見えます。
  画像も kitty キーボードも、`OSC 52` によるクリップボードもありません。`Alt` の組み合わせには
  *Use Option as Meta key* にチェックを入れてください。
- **Alacritty、Tabby。** Alacritty：文字のグラフ、kitty キーボード、`OSC 52` コピー。`Alt` の組み合わせには
  `option_as_alt = "Both"` を設定してください。Tabby：sixel のグラフとアバター。
- **Option を Meta に。** ターミナルが Option を Meta に変えない限り、Option は文字を合成します（Option+B は積分記号を
  入力します）：iTerm2 は *Left Option key: Esc+*、Terminal.app は *Use Option as Meta key*、Alacritty は
  `option_as_alt`。GitPoulpi の `Alt` の組み合わせにはこれが必要ですが、既定でオンのものはありません。
- **kitty、Ghostty、WezTerm、Rio、Warp**：macOS では未測定です（GPU が必要）。Linux と同じ動作と考えてください。
- **xterm**（XQuartz）：`xterm -u8 -ti vt340`。sixel には `ui.image_protocol = "sixel"` が必要です。

### VS Code と Warp

VS Code ターミナル（`TERM_PROGRAM=vscode`）はテストの対象外です。`terminal.integrated.enableImages` がオンのときは
デバイス属性から認識され、そうでなければ GitPoulpi は文字で描画します。`ui.image_protocol = "sixel"` で画像を強制
できます。Warp：上の OS ごとの注意を参照してください。

## マルチプレクサ

tmux、zellij、screen、herdr、psmux、そしてターミナル自身のペインは、GitPoulpi とターミナルの間に入り、何が通るかを
変えます。要点：tmux は sixel を通し、`allow-passthrough on` ならポインターの形と kitty の画像も通します。zellij と
screen は画像もポインターの形も通しません。herdr は kitty の画像を通します。詳細と追加する設定は
[マルチプレクサ](multiplexers.md)にすべてあります。

## 機能が欠けたときの動作

| 欠けているもの | 代わりに得られるもの |
|---|---|
| 画像プロトコル | 文字のグラフ。`ui.avatar_fallback` の設定にかかわらず、アバターは色付きのイニシャルのバッジ（1 文字）になります：文字のセルに押し込んだアイデンティコンは判別できないためです。画像 diff とファイルビューは、色付きの半ブロックか情報カードを使います。 |
| トゥルーカラー | 色は 256 色または 16 色のパレットに割り当てられます（`ui.color_depth`）。 |
| kitty キーボードプロトコル | 従来のキー。一部の組み合わせは別のキーとして届きます。設定が警告するので、割り当て直すか `Alt` の別形を使います。 |
| OSC 52 | ローカルでは何も変わりません（システムのクリップボードを使います）。SSH 越しやディスプレイがない場合、コピー操作はクリップボードに何も置きません：マウスで選択して（`Shift` を押したまま）ターミナル自身のコピーを使ってください。 |
| OSC 22 | ポインターは変わりません。マウスの下にある区切りやボタンは引き続き点灯します。 |
| 絵文字フォント | アイコンの代わりに四角や穴が出ます。グラフとテキストは引き続き動作します。Nerd Font か絵文字フォントをインストールし、フォールバックとして指定してください。 |

## 知っておきたい設定

| 設定 | 用途 |
|---|---|
| `ui.image_protocol` | `auto`（既定）、`kitty`、`iterm2`、`sixel`、`none`。画像が出るはずの場所に文字化けが出る場合は `none` にします。再起動が必要です。 |
| `ui.graph_renderer` | `auto`、`image`、`text`。`text` は常に正しい表示です（ConPTY の限界が当てはまる所で使います）。再起動が必要です。 |
| `ui.color_depth` | `auto`、`truecolor`、`256`、`16`。 |
| `ui.kitty_keyboard` | ターミナルやマルチプレクサの動作がおかしいときは `auto` または `off`。再起動が必要です。 |
| `ui.text_cursor` | `terminal`（既定）または `drawn`。 |
| `ui.mouse_enabled` | オフにするとマウスをターミナルに任せます。再起動が必要です。 |
| `ui.launch_terminal`、`ui.relaunch_in_terminal` | OS のショートカットが開くターミナルと、ダブルクリックで GitPoulpi をそこで開き直すかどうか（[設定](settings.md)）。 |

[マルチプレクサ](multiplexers.md)、[画像](image-view.md)、[キーバインド](keybindings.md)、
[トラブルシューティング](troubleshooting.md)も参照してください。

## 出典

ドキュメントは 2026年10月に確認し、動作は 2026年11月にテストしました。特記がないものは一次情報です。

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/)、[pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22)、[OSC 52](https://ghostty.org/docs/vt/osc/52)、[download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html)、[escape sequences](https://wezterm.org/escape-sequences.html)、[OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292)、[releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md)、[releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767)、[kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495)、[kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html)。kitty キーボードは [terminfo.dev の一覧](https://terminfo.dev/extensions/kitty-keyboard-protocol)から（二次情報）
- Windows Terminal: [1.22 の発表、sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release)、[Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/)（二次情報）、[v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [escape sequence support](https://rioterm.com/docs/escape-sequence-support)（`OSC 22` の行なし）、[features](https://rioterm.com/docs/features)、[kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps)。OSC 52 は [Warp の OSC 52 に関する勧告の解説](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp)から（二次情報）
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md)、[tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app（トゥルーカラーなし）: [Apple のディスカッション](https://discussions.apple.com/thread/255315820)（二次情報）
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES)、zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- 比較サイト（二次情報。照合のためだけに使用）: [Terminal Trove](https://terminaltrove.com/compare/terminals/)
