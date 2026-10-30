[English](../local-issues.md) · [Čeština](../cs/local-issues.md) · [Deutsch](../de/local-issues.md) · [Español](../es/local-issues.md) · [Français](../fr/local-issues.md) · [Italiano](../it/local-issues.md) · **日本語** · [한국어](../ko/local-issues.md) · [Nederlands](../nl/local-issues.md) · [Polski](../pl/local-issues.md) · [Português (Brasil)](../pt-BR/local-issues.md) · [Português (Portugal)](../pt-PT/local-issues.md) · [Русский](../ru/local-issues.md) · [Svenska](../sv/local-issues.md) · [Türkçe](../tr/local-issues.md) · [Українська](../uk/local-issues.md) · [简体中文](../zh-CN/local-issues.md)

# ローカル Issue と Obsidian

Issue を Markdown ノートのフォルダーとして管理できます。アカウントもトークンもネットワークも不要です。フォルダーはリポジトリの中
（コードと一緒に commit する `docs/issues/`）でも、ディスク上のどこでも構いません。たとえば [Obsidian](https://obsidian.md) の
保管庫のサブフォルダーです。GitPoulpi はノートを読み書きします。Obsidian、任意のエディター、シェルも同時にノートを編集でき、
一覧はそれに追従します。

![サイドバーのローカル Markdown Issue と、1 つのノートの行メニュー：ノートを開く、クローズ、ラベルやマイルストーンの編集](../images/local-issues.png)

## 設定する

1. **設定 > 連携 > Issue トラッカー > Local Markdown > プロファイルを追加**。フォルダーを入力するか、**参照...**を押して選びます。
   相対パスのフォルダー（`docs/issues`）は、リンクされた各リポジトリを基準に読み取られ、絶対パス
   （`/home/me/Vault/Projects/Issues`）はどこでも同じ場所を指します。入力するトークンはありません。
   - CLI：`gitpoulpi hosting integrations add --kind local_markdown --path docs/issues`。
2. リポジトリで、サイドバーの **ISSUES** セクションを開き、プロバイダー **Local Markdown - <name>** を選びます
   （フォルダーも一緒に選ばれます。別のフォルダーを使いたい場合、リポジトリのドロップダウンにそのフォルダーと直下のサブフォルダーが並びます）。
   - CLI：`gitpoulpi hosting link issues --id <profile>`（プロファイルのフォルダーが `--project` の既定値です）。

自動でリンクされるものはありません。ほかの Issue の取得元と同じく、リポジトリごとに明示的にオンにします。

## ノート

Issue ごとに 1 つのノートがあり、名前は `<id>-<slug>.md`（`12-crash-on-empty-name.md`）で、フロントマターを持ちます。

```yaml
---
id: 12
title: Crash on empty name
status: todo
assignees: [kai]
labels: [bug]
milestone: v0.1
created: 2026-10-14
updated: 2026-10-14T09:30:00
---
Free Markdown. GitPoulpi never rewrites the body.
```

- 新しい Issue には `max(id) + 1` が付きます。
- GitPoulpi が書き込むのは、自分が管理するプロパティ（`title`、`status`、`assignees`、`labels`、`milestone`、`updated`）だけです。
  **それ以外のプロパティ、本文、改行コード、キーの順序は、バイト単位でそのまま保たれます。**書き込みはアトミックです
  （一時ファイルに書いてから名前を変更します）。
- 読み取りは寛容です：`id` のないノートはファイル名から取り、`status` のないノートは `todo` として扱い、フロントマターのないノートは
  最初の `# 見出し` からタイトルを取ります。Issue でないファイル（`readme.md`）は無視されます。
- ステータスは自由なテキストです。**オープン**とは「完了ステータスではない」という意味です。

## できること

| ISSUES セクションで | キーボード / パレット |
|---|---|
| 新規 Issue（`+`、または `n`）：タイトル、説明、ステータス、担当者、ラベル、マイルストーン | `ホスティング: 新規 Issue...` |
| 行メニューから担当者 / ラベル / マイルストーンを編集。フォルダーで既に使われているものから選ぶか、**新しいラベルやログイン名を入力**して `「...」を追加` の行を選びます | `ホスティング: ラベルを編集...` など |
| **Issue をクローズ** / **Issue を再オープン**（完了ステータス、または完了以外の最初のステータスにします） | `ローカル Issue: Issue をクローズ...` / `Issue を再オープン...` |
| **ノートを開く**（`.md` の既定のプログラムで開きます） | `連携: Issue を開く...` |
| **Obsidian で開く**（フォルダーが保管庫の中、つまり上位に `.obsidian/` フォルダーがある場合のみ） | `ローカル Issue: Obsidian で開く...` |
| フィルター：オープン / クローズ / すべて、「自分に割り当て」 | フィルターのドロップダウン |

「自分に割り当て」は、プロファイルの `me` の設定、なければ接続中の名前、それもなければプロファイル名と照合されます。

ブランチと commit は GitHub と同様に動作します：「Issue からブランチを作成」は `12-crash-on-empty-name` を作り、commit メッセージ内の
`#12` やブランチ名の `12-...` も認識されます。

GitPoulpi の外（Obsidian、シェル、`git pull`）で行った変更は、約 1 秒以内に反映されます。フォルダーのファイル名と更新時刻は、
描画スレッドではなく、バックグラウンドのスレッドで調べられます。

## 設定ファイル

`config.toml` の、プロファイルの隣に書きます（すべてのキーは省略可能です）。

```toml
[[integrations]]
id = "vault"
kind = "local_markdown"
url = "/home/me/Vault/Projects/Issues"
token_ref = "integration:vault"

[integrations.local]
statuses = ["todo", "doing", "review", "done"]   # the board's column order
done_status = "done"                            # closes an issue
reference_prefix = "LOC"                         # shown as LOC-12 instead of #12
me = "kai"                                       # who "assigned to me" means
```

ノートが使っていても `statuses` に載っていないステータスは、載っているものの後ろに並びます。`statuses` は
[ボード](#カンバンボード)の列の順序です（`gitpoulpi hosting issue options` で出力されます）。

## カンバンボード

![ノートフォルダーのボード：ステータスごとに 1 列、担当者、ラベル、マイルストーンを表示したカード](../images/issue-board.png)

1 人で作業している場合や、Issue をコードの隣で管理しているチームでは、フォルダーはカンバンボードにもなります。
**ISSUES ヘッダー > ボード**、またはパレットの **Issue: ボードを開く** で、ウィンドウの上に重ねて開き、`Esc` で閉じます。

- **列**はステータスです：`statuses` に載っているものがその順序で先に並び、続いてノートが使うほかのステータスが並びます。
  空の列も表示されます。各ヘッダーにはカード数が表示されます。
- **カード**には、Issue 番号、タイトル（2 行）、担当者のイニシャル、ラベル、マイルストーンが表示されます。
  すべての列が収まらないときは、ボードが横にスクロールします（下のバー、`Shift` とホイール、または矢印キー）。
- **カードを移動**するには、別の列にドラッグします（ポインターの下で列が光ります）。`Shift+Left` / `Shift+Right` を使うか、
  **ボード: カードの移動先...**（`m`）で列のドロップダウンから選びます。カードはすぐに移動し、ノートの `status` が
  [ノートの節](#ノート)のとおりに書き換えられ、ファイルのそれ以外の部分は変わりません。書き込みに失敗するとカードは元の位置に戻り、
  エラーがステータスバーに表示されます。
- カードを**開く**には `Enter` またはダブルクリックします（ノートは `.md` の既定のプログラムで開きます）。`e` は担当者、ラベル、
  マイルストーンを編集し、`.` または右クリックでカードのメニューを開き、`n` は選択中の列に新規 Issue を作ります
  （フォームを閉じるとボードに戻ります）。
- 担当者（`a`）、ラベル（`l`）、マイルストーン（`M`）、テキスト（`/`）で**絞り込み**ます。`x` で解除します。フィルターは
  ドロップダウンで、入力して絞り込み、`Enter` で選びます。
- **ライブ**：Obsidian やシェルから変更したノートのカードは、1 秒以内に移動します。

すべてのマウス操作に、キーボードのキーとパレットのコマンド（**ボード: ...**、[キーバインド](keybindings.md)のページに一覧があります）が
あります。

## Obsidian Bases

**パレット：`ローカル Issue: Obsidian Base を作成`**（CLI：`gitpoulpi hosting issue base`）は、ノートの隣に `issues.base` を 1 度だけ
書き込みます。*テーブル*ビューと、`status` でグループ化された*カンバン*ビュー（`statuses` の順）です。Obsidian で開くと、同じ Issue が
そこでもボードとして表示されます。GitPoulpi は既存の `.base` ファイル（ユーザーのものでも以前のものでも）を編集せず、
拒否してそう伝えます。形式は Obsidian の Bases の構文（<https://obsidian.md/help/bases/syntax>）に従います。
カンバンビューには新しい Obsidian が必要です。

## CLI と MCP

ほかのすべての Issue の取得元と同じコマンドが使えます：`hosting issues`、`hosting issue create [--list <status>]`、
`hosting issue edit <n> --add-label ...`、`hosting open issue <n>`、`hosting refs`。ノート専用のコマンドが 4 つあります：
`hosting issue close <n>`、`hosting issue reopen <n>`、`hosting issue move <n> --status <s>`、`hosting issue base`。
[CLI リファレンス](cli-reference.md)を参照してください。

## 制限

- 新しい**マイルストーン**はピッカーに入力できません（フォルダーで既に使われているものを選んでください）。ただし、ノートの
  `milestone:` には何でも書け、保存するとすぐに反映されます。
- フォルダー直下のノートだけが Issue です（サブフォルダーは対象外）。
- 削除はありません：ファイルマネージャーまたは Obsidian でノートを削除してください（その際のごみ箱はユーザー自身のものです）。
