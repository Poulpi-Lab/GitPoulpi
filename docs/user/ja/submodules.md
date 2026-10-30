[English](../submodules.md) · [Čeština](../cs/submodules.md) · [Deutsch](../de/submodules.md) · [Español](../es/submodules.md) · [Français](../fr/submodules.md) · [Italiano](../it/submodules.md) · **日本語** · [한국어](../ko/submodules.md) · [Nederlands](../nl/submodules.md) · [Polski](../pl/submodules.md) · [Português (Brasil)](../pt-BR/submodules.md) · [Português (Portugal)](../pt-PT/submodules.md) · [Русский](../ru/submodules.md) · [Svenska](../sv/submodules.md) · [Türkçe](../tr/submodules.md) · [Українська](../uk/submodules.md) · [简体中文](../zh-CN/submodules.md)

# サブモジュール

サブモジュールは、固定された commit で、自分のリポジトリの中に入れ子になった別のリポジトリです。GitPoulpi は Refs サイドバーにそれらを一覧し、UI、CLI、コマンドパレットから更新し、checkout した commit と同期させておくこともできます。

## サブモジュールセクション

リポジトリに `.gitmodules` があると、Refs サイドバーに**サブモジュール**セクションが加わります（ほかのセクションの下に表示され、ない場合は表示されません）。サブモジュールごとに 1 行で、状態の印、パス、短い commit が表示されます。

| 印 | 状態 | 意味 |
|---|---|---|
| `✓` | 最新 | リポジトリが記録している commit で checkout されている |
| `↑` | 変更あり | 別の commit で checkout されている（先に進んだ、または更新せずにブランチを切り替えた） |
| `○` | 未クローン | `.gitmodules` に宣言され記録されているが、フォルダーが空：更新するとクローンされる |
| `✗` | 競合 | リポジトリのサブモジュールに merge 競合がある |

| 操作 | マウス | キーボード |
|---|---|---|
| サブモジュールを専用のタブで開く | 行をダブルクリック | 選択して `Enter`（パレット：`Refs: サブモジュールをタブで開く`） |
| メニュー：**更新**、**初期化**、**新しいタブで開く** | 行を右クリック | 選択して、コンテキストメニューのキー（既定は `.`） |
| 選択したサブモジュールを更新（必要なら先にクローン） | メニュー | `Ctrl+P`：`Refs: サブモジュールを更新（選択中）` |
| クローンせずに `.git/config` に登録 | メニュー | `Ctrl+P`：`Refs: サブモジュールを初期化（選択中）` |
| すべてのサブモジュールを更新 | | `Ctrl+P`：`Git: すべてのサブモジュールを更新` |
| セクションへ移動 | ヘッダーをクリック | `Ctrl+P`：`ナビゲーション: Refs > サブモジュールにフォーカス` |

更新はサブモジュールのリモートから fetch することがあるため、ほかの転送と同様に**設定 > SSH と認証情報 > HTTPS 認証情報**に従います。クローンされていないサブモジュールにはタブで開くものがありません。ステータスバーがそう伝え、更新を案内します。

## サブモジュールを最新に保つ

**設定 > 同期 > サブモジュールを最新に保つ**（`general.keep_submodules_up_to_date`、既定はオフ）。オンにすると、**checkout** または **pull** が成功した後に `git submodule update --init --recursive` が実行されます。これはサブモジュールがリポジトリの現在の記録とずれている場合にのみ実行されるため、サブモジュールのないリポジトリが遅くなることはありません。その更新が失敗しても、checkout や pull は行われており、エラーがそのことを伝えます。

## コマンドラインから

```
gitpoulpi submodule list                       # path, state, url (--json: the full list)
gitpoulpi submodule init [<path>...]           # register, do not clone
gitpoulpi submodule update [<path>...] [--init] [--recursive]
```

パスを指定しない場合は、すべてのサブモジュールが対象です。同じ 3 つが MCP ツールにもなっています（`submodule_list` は読み取り、`init` と `update` は変更操作で、`[mcp]` の `allow_mutations` に従います）。

## 注意

- 最上位のものだけが一覧されます。サブモジュール自身のサブモジュールは、それをタブで開いたときに表示されます（または `--recursive` で更新します）。
- 新しい git は、`protocol.file.allow` が許可しない限り、*ローカルパス*のサブモジュールを `file` トランスポート経由でクローンすることを拒否します。これは URL がローカルフォルダーのサブモジュールにのみ関係します。そのようなサブモジュールを使う場合は、グローバルの git 設定で設定してください。
