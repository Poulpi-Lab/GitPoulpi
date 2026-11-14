[English](../worktrees.md) · [Čeština](../cs/worktrees.md) · [Deutsch](../de/worktrees.md) · [Español](../es/worktrees.md) · [Français](../fr/worktrees.md) · [Italiano](../it/worktrees.md) · **日本語** · [한국어](../ko/worktrees.md) · [Nederlands](../nl/worktrees.md) · [Polski](../pl/worktrees.md) · [Português (Brasil)](../pt-BR/worktrees.md) · [Português (Portugal)](../pt-PT/worktrees.md) · [Русский](../ru/worktrees.md) · [Svenska](../sv/worktrees.md) · [Türkçe](../tr/worktrees.md) · [Українська](../uk/worktrees.md) · [简体中文](../zh-CN/worktrees.md)

# worktree

worktree は、リポジトリのもう 1 つのチェックアウト済みコピーです。別のフォルダーで別のブランチを開きながら、
同じ履歴を共有します。Refs ペインの **WORKTREE** セクションに一覧が出ます。メインのものには `（メイン）` が付きます。

## セクションを表示する

リンクされた worktree がまだないリポジトリでは、セクションは非表示です。それでも表示するには
（「worktree はまだありません：+ で作成」と出ます）、パレットのコマンド
**サイドバー: worktree セクションを表示** を使うか、サイドバーの表示/非表示の一覧で **WORKTREE** にチェックを付けるか
（セクションの見出しを右クリック）、下の作成コマンドのどれかを実行します。GitPoulpi はこれをそのリポジトリ
ごとに記憶します（[設定](settings.md)の **左ペインのセクション** を参照）。worktree のない別のリポジトリでは、
引き続きセクションは非表示で始まります。

## worktree を作成する

WORKTREE 見出しの **+**（キーボード：セクション内で `+`、パレット：**Refs: worktree を作成...**）でメニューが開きます。

- **現在のブランチから**：そのブランチはメインのフォルダーでチェックアウト済みなので、ダイアログはその上に
  新しいブランチ（`<ブランチ>-wt`）を提案します。
- **選択中のcommitから**：Log で commit を選ぶまでグレーアウトします。新しいブランチを指定しない限り、
  worktree はそこで HEAD を切り離して始まります。
- **ブランチを選択...**：ダイアログが **起点** の一覧に合わせて開きます。

パレットにも同じ 3 つがあります（**Refs: 現在のブランチから worktree を作成**、
**Git: 選択中のCommitに worktree を作成...**、**Refs: ブランチを選んで worktree を作成**）。従来の項目も使えます。
ブランチ、tag、commit のメニューの **<名前> から worktree を作成...** です。

ダイアログはすべてを提案し、変更もできます。

| フィールド | 意味 |
|---|---|
| **名前** | フォルダー名はこれから付きます。ブランチまたは短い ID から提案されます。 |
| **フォルダー** | リポジトリと同じ階層の `<リポジトリ>-<名前>`。フォルダー欄に入力するまでは名前に追従します。**参照...** で場所を選びます。 |
| **起点** | worktree の開始点となるブランチまたは commit（一覧。入力で検索）。変更すると新しい名前が提案されます。 |
| **新規ブランチ** | 空欄なら、起点のブランチをそこでチェックアウトします（commit の場合は HEAD を切り離します）。そのブランチがすでにチェックアウト済みなら必須です。 |
| **新しい worktree をタブで開く** | 既定でオンです。 |

空でないフォルダー、無効な名前、すでに存在するブランチは、ダイアログ自身の行に表示され、**作成** はグレーのままです。

## 行のメニュー

worktree を右クリックします（または選択してメニューキー、`Shift+F10`、`.`）。

- **新しいタブで開く**：ダブルクリックや `Enter` と同じです。
- **開く場所**：**ワークツリーを開ける** にチェックが付いた[開く場所](open-in.md)の項目です（設定 > 開く場所。
  既定ではすべての項目にチェックが付いています）。worktree のフォルダーを `${cwd}` として実行します。
- **<ブランチ> を別のブランチにmerge...**：merge 先のブランチを選びます（既定はメインの worktree のブランチ）。
  通常の merge が、そのブランチをチェックアウトしている worktree で、あなたの merge オプションで実行されます。
  競合は、そこで通常の競合の流れで解決します。どこにもチェックアウトされていないブランチは、理由付きで拒否されます。
  HEAD が切り離された worktree には表示されません。
- **worktree を削除...**：メインのものは削除できません。

パレットにも、選択中の行に対する同じ操作があります（**Refs: worktree を開く場所...**、
**Refs: worktree のブランチを別のブランチにmerge...**、**Refs: worktree を削除...**、
**Refs: Worktree をタブで開く**）。

## worktree を削除する

変更のない worktree では、通常の確認だけです。そのブランチがすでに merge 済み
（その upstream、メインの worktree のブランチ、または `main` / `master` に）なら、チェックボックス
**ブランチ <b> も削除** が既定でオンになります。未 merge のブランチには、残ることが表示されます。
**mergeしてから削除...** は先に merge し、競合なく merge できた場合にだけ削除します。

未 commit の変更（追跡対象または未追跡）があると、ダイアログがそれを一覧表示し（件数付き、スクロール可）、次を提供します。

- **commitして削除**：ダイアログ内のメッセージ欄に入力します。すべての変更がステージされ
  （`.gitignore` は尊重されます）、その worktree のブランチに commit されてから、worktree が削除されます。
  HEAD が切り離された worktree では、同じダイアログで commit 用のブランチ名を指定します。
- **stashして削除**：未追跡ファイルを含むすべての変更を、編集できるメッセージ（既定は「{名前} の作業中 (WIP)」）で
  stash してから、worktree を削除します。stash の一覧はすべての worktree で共有されます。つまり、その stash は
  あとでメインのリポジトリの STASHES に入り、pop できる状態になります。
- **開いてcommit**：worktree をタブで開き、commit メッセージにフォーカスします。何も削除されません。
- **それでも削除**：2 回目の明示的な確認のあとで、変更を破棄します。

## 元に戻す

worktree の削除は undo できません。フォルダーがなくなるためです。先に保存したものは残ります。commit はそのブランチに、
stash は一覧に残ります。**ブランチ <b> も削除** が加えるブランチの削除は通常のジャーナル対象のもので、
**元に戻す** でブランチが戻ります（フォルダーは戻りません）。worktree の作成も履歴には含まれません。

## スクリプトやエージェントから

`gitpoulpi worktree changes <path>` は worktree の未 commit の変更を一覧表示します。
`gitpoulpi worktree remove <path> [--force] [--commit-message M
[--commit-branch B]] [--stash-message M] [--delete-branch]` は worktree を削除します。MCP ツールは
`worktree_changes` と `worktree_remove` です。[CLI リファレンス](cli-reference.md)を参照してください。
