# ja terminology and style (crates/gitpoulpi-i18n/locales/ja.toml)

Sources: git's own Japanese translation (git-l10n po/ja), Microsoft Japanese terminology and style guide, GitHub Desktop / VS Code / JetBrains / TortoiseGit / SourceTree usage. Follow this table for any new key.

## Two vocabularies

Default ("git" variant): git words stay English (or the usual katakana for stage). "Translate git terms" on ("local" variant): the Japanese word. Every git word is a `{$term}` token, never inline text, so both modes work. Where a concept has no token, write a pair `{ git = "..", local = ".." }` (worktree, revert, reset, checkout, clone, amend, pop/apply).

| Concept | git variant | local variant | Notes |
|---|---|---|---|
| commit (n./v.) | commit | コミット | |
| push / pull / fetch | push / pull / fetch | プッシュ / プル / フェッチ | |
| merge | merge | マージ | |
| rebase | rebase | リベース | |
| cherry-pick | cherry-pick | チェリーピック | |
| stash | stash | スタッシュ | toolbar button (width): 退避 |
| stash pop / apply / drop | Pop / Apply / Drop | 復元 / 適用 / 破棄 | pop = apply and remove; tokens `{$Pop}`, `{$Apply}` and `{$Stash_drop}` (git: pop / apply / drop) |
| tag | tag | タグ | |
| diff | diff | 差分 | |
| stage / unstage / staged / unstaged | ステージ / ステージ解除 / ステージ済み / 未ステージ | same | katakana accepted in git mode; English-forced option swaps to `stage` |
| pull request / merge request | same | プルリクエスト / マージリクエスト | follows the host; abbreviation PR / MR |
| pick / reword / squash / fixup / drop / edit | same | 採用 / メッセージ変更 / スカッシュ / fixup / 破棄 / 編集 | rebase todo actions |
| checkout, revert, reset | same | チェックアウト, リバート, リセット | no token: write a pair when inline |
| worktree | worktree | ワークツリー | git-l10n; "working tree" (the files) is 作業ツリー |
| fast-forward, upstream | same | 早送り, アップストリーム | |
| conflict, resolve | 競合, 解決 | same | never 衝突 / コンフリクト |
| blame | Blame | Blame | kept English (GitHub, VS Code ja); not ブレーム |
| hunk | ハンク | same | merge-editor "chunk" is チャンク |
| amend | amend / Amend | 修正 | label `直前のコミットを修正（Amend）` |
| prune | プルーン | same | |
| HEAD, SHA, LFS, ours/theirs/base, index | unchanged | unchanged except index インデックス | |
| repository, branch, remote | リポジトリ, ブランチ, リモート | same | never English |
| ref(s) | 参照 | same | `Refs` stays only as the pane / palette category name, `Ref` column header |
| settings / palette | 設定 / コマンドパレット | same | |
| issue | Issue | same | not イシュー or 課題 |
| avatar, minimap | アバター, ミニマップ | same | |
| Log (the commit graph pane) | ログ in prose, Log in palette entries | same | the application log is アクティビティログ |
| merge-editor panes | 現在 (A), 取り込み (B), ベース, 出力 | same | not Base / Output in prose |
| composer fields | 概要, 説明 | same | not Summary / Description |
| diff layout | インライン, 分割 | same | not 統合 / 左右並べ |
| undo safety net | 元に戻す履歴 (Undo history), これだけ復元 (Restore only this), ここまで元に戻す / ここまでやり直す (Undo / Redo up to here), 失われた作業 (Lost work), Reflog (kept English, like Blame) | same | tags `単独で復元可`, `ブロック中`, `取り消し済み`; the offer reads `{name} を {op} が残した位置に復元` |
| terminal-related | ターミナル, マルチプレクサ, 画像 (the terminal pictures), トゥルーカラー, 文字のグラフ (character graph) | same | page `terminals.md`: `（未確認）` for "unverified", `二次情報` for "secondary" source |

## Style

- Address: neutral polite (です・ます) sentences, no pronoun. Buttons and menu rows are noun or dictionary-form verb phrases (`保存`, `ブランチを作成`), never ます form.
- Punctuation: full-width （ ） ： 、 。 ？ in prose, including inside parentheses; the ASCII `'` apostrophe only where the catalog test demands it (it quotes names: `'{name}'`). The ellipsis is three ASCII dots `...` exactly where the English source has them (menus, buttons, palette names, placeholders, in-flight messages), never the real `…`; the user guide follows. A palette entry keeps the ASCII `カテゴリ: 操作` form (same categories as English: Git, Refs, ナビゲーション, 作業中の変更, 履歴, 表示...).
- Spacing: no space between Japanese and a `{$term}` token (`{$commit}を作成`, so the local variant never gets a stray space); one space around placeholders `{n}` / `{path}` and around Latin words in prose (`Status ペイン`, `{n} 個のファイル`).
- Long vowels follow Microsoft style: ブラウザー, エディター, フォルダー, ポインター, マネージャー; ディレクトリ, メモリ, ユーザー.
- Time: `2026年10月12日`, months `10月`, weekdays `月曜日`, relative `5 分前` / `3 か月前`.
- Keys stay `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`, `Tab`, `Space`, `Up/Down` in hints.
- Width: CJK is 2 cells; button and tab labels at most 4-5 characters (`チャンク`, `今すぐ復元` are already at the limit).
- Lint limits: placeholders named `{commit}`, `{tag}`, refs like `stash@{id}`, code identifiers (`git.pull_mode`, `diff.tool`, `{{merged}}`) and AI prompt chips are flagged by the local-mode lint and stay in the baseline `tests/git_vocabulary/ja.toml`.

## User guide (docs/user/ja)

- Labels in **bold** or `code` are quoted exactly as the catalog shows them in the default (git) variant, including the missing space next to a git word (`直前のcommitを修正（Amend）`, `diffとエディター`, `N ファイルの変更をcommit`). Running prose puts a space around Latin words (`commit を作成`).
- The keymap and palette tables of `keybindings.md` are generated from the catalog (labels of the `ja` catalog, git variant); the key column is the English one, except the few palette key descriptions (`ダブルクリック`, `競合したファイルで Enter`...), which stay Japanese. Regenerate both from a throwaway test that prints `with_lang(Lang::Ja, ...)`, never by hand.
- No pronoun: `あなた` becomes `ユーザー` (third person, "the user's window"), `自分` ("my pull requests") or is dropped. `お使いの` is fine for "your terminal / your git".
- `ターミナル` (never `端末`), `認証情報ヘルパー` (credential helper), `パーソナルアクセストークン`, `Issue トラッカー`, `エントリ` (no long vowel, as the catalog), `マルチプレクサ`, `ポインター`, `参照` for refs (prose), `ログ` for the Log pane (prose; `ログペイン` as the catalog), `名前変更` for rename, `早送り` for fast-forward, `競合` for merge conflicts but `衝突` for a clashing key binding.
- Key names stay English (`Up`, `Down`, `Ctrl`); the mouse part of a chord is Japanese (`Ctrl+クリック`). Half-width `:` and `()` next to Japanese text become `：` and `（）`, except inside a quoted label, where the catalog decides (palette categories use `: `, a few use `：`).
- The theme `Terminal (16 colours)` keeps the name the code gives it.

## Undo history labels and outside changes

- `[undo_label]` rows are noun-object plus dictionary-form verb (`{path} を破棄`, `{name} をチェックアウト`); git words are `{$term}` tokens (`{$stash}を{$pop}`), checkout / revert / reset stay katakana in both variants (as the menus). The reset modes `soft` / `mixed` / `hard` stay English (option names).
- Outside change (recorded from a shell or an AI agent): `外部` (tag), `GitPoulpi の外での変更`; reflog-style texts are translated (`チェックアウト: {from} から {to} へ移動`). Mainline of a merge revert: `メインライン`, parent `親`.
- `nr_index` avoids the word index / インデックス inline (the English-forced lint flags it): it says `{$stage}の状態`.

## Workspaces, search and Radar

- workspace: `ワークスペース` everywhere (UI, CLI help, guide); never a katakana variant such as ワークスペイス, and not the multiplexer meaning. Active: `有効`; missing repository: `見つかりません`; locate: `場所を指定...`; remove from a workspace: `除外`.
- Radar stays `Radar` (product name). Snooze: `保留` (state `保留中`), pin: `固定`, scope: `対象`, view: `表示`, show (the who filter): `絞り込み`, "Mine": `自分が作成`, WIP: `WIP`, draft: `ドラフト`.
- Issue stays `Issue` in every string (also the search kind and Radar headings), never `イシュー`; no plural.
- A word plus a git term uses a token (`すべて{$Fetch}`, `{name} 内をすべて{$fetch}`); a bare worktree in a plain string is a `{ git, local }` pair (`worktree` / `ワークツリー`).
- Status messages are impersonal past tense (`ワークスペース {name} を作成しました`, `{item} の保留を解除しました`); a `{age}` placeholder is already a duration without `前` (`5分`, `たった今`), so `note_offline` reads `オフライン（データの古さ：{age}）`.
- CLI help: コミット / スタッシュ / プルリクエスト / ワークスペース (katakana) as the rest of that file; `worktree` stays the command's name.

## Review, metadata, Board, local issues (round 4)

- Review (the in-app code review): `レビュー`; category of its palette rows `レビュー:`. Request (pull / merge request / Gerrit change, generic): `リクエスト`. Overview / Files / Commits pages: `概要` / `ファイル` / `{$Commits}`.
- Thread: `スレッド`; resolve `解決` (state `解決済み`), reopen `再開`, outdated `古い`, fold / unfold `折りたたみ／展開`. Pending comment (kept locally until the review is finished): `保留中のコメント` (GitHub ja says 保留中; the Radar snooze is the same word `保留`, the contexts never meet), pending review `保留中のレビュー`.
- Finish review `レビューを完了`, Submit review `レビューを送信`, Discard `破棄` / Keep `保持`. Verdict: `判定`; approve `承認`, request changes `変更を要求`, comment `コメント`, dismissed `却下`. Viewed (file): `閲覧済み`.
- ✦ AI review suggestion: `提案` (accept `採用`, edit `編集`, dismiss `却下`, accept / dismiss all `すべて採用` / `すべて却下`); severity `情報` / `警告` / `問題`; AI summary `AI の要約`. "Code review" (the AI action): `コードレビュー`. AI instructions: `AI の指示` (as the `指示` of Settings > AI); workspace level `ワークスペースの AI の指示`; the Combine list keeps the repository wording `全体と組み合わせる` / `全体に追加` / `全体を置き換え`.
- Gerrit: patch set `パッチセット`, vote `投票` (CLI help only), change `チェンジ` (help and docs), push for review `レビュー用に{$push}`.
- Metadata: assignee `担当者`, reviewer `レビュアー` (short, fits buttons; not レビュー担当者), label `ラベル`, milestone `マイルストーン`; "Edit ..." `...を編集`, "Set milestone" `マイルストーンを設定`; "No milestone" `マイルストーンなし`; Radar filter "Mentioning me" `自分への言及`.
- Diff encoding: `文字コード` (button width: `文字コード`, value `自動`; list entry `自動検出`), never エンコーディング / エンコード.
- Board (Kanban): `ボード`, column `列`, card `カード`; the host columns Open / Closed are `オープン` / `クローズ` (as the issue states); "Move Card To..." `カードの移動先...`. Palette category `ボード:`; `Issue: ボードを開く`.
- Local issues: `ローカル Issue`; notes folder `ノートフォルダー`; Obsidian vault `保管庫` (Obsidian's own ja UI word); `Base` (Obsidian file type) kept English: `Obsidian Base`; close / reopen an issue `Issue をクローズ` / `Issue を再オープン`; status (frontmatter) `ステータス`. Palette category `ローカル Issue:`.
- Chip texts sent to the model (`review_ai.chip_*_text`) stay English as everywhere; ghost label `{severity} - {range} 行目`.

## Branching workflows, Mermaid and Gerrit follow-ups (round 5)

- Workflow: `ワークフロー`; branching workflow `ブランチワークフロー`; the toolbar button and palette category stay `Flow` (product word, like Radar), also in `Flow メニュー`, `Flow ドロップダウン`. Git Flow, GitHub Flow, GitLab Flow keep their names; trunk-based is `トランクベース`; template `テンプレート`, preset `プリセット` (guide only), Custom `カスタム`.
- Base branch `ベースブランチ` (badge `[ベース]`, `[ベース、保護]`), protected `保護`, environment branch `環境ブランチ`, topic branch `トピックブランチ`, topic type / type `トピックの種類` / `種類`, parent branch `親` (`親ブランチ` in prose), "Other" folder `その他`, name pattern `名前パターン`, slug `スラッグ`, token `トークン`, regular expression `正規表現`, other names `別名`.
- Actions: start `開始`, finish `完了`, sync `同期`, promote `昇格`, set parent `親を設定`, continue / abort `続行` / `中止`, preview `プレビュー`, hook `フック` (`開始後` / `完了前`), finish by `完了方法`, finish target `完了先`, strategy `戦略` (`{$Merge}戦略`, as git's マージ戦略), "Merge into" `{$Merge}先`. fast-forward is `早送り` in both variants (as elsewhere).
- Host-driven flow: "Pick up" / "Take over" `引き継ぐ` / `引き継ぎ元` (not 取り込み, which is the merge editor's side B), bot `ボット`, wait `待機`, "Look up" `検索`, "Later" `後で`, "Stop waiting" `待機を中止`. Issue stays `Issue` (`新しい {label} Issue`).
- Mermaid gitGraph (script name kept Latin): popup title `Mermaid gitGraph としてコピー`; detail levels `構造` / `参照` / `件名` / `完全` (the CLI values `structure` / `refs` / `subjects` / `full` stay English in the guide), label `詳細度`, direction `方向` (`左から右` / `上から下`), fence `フェンス`, preview `プレビュー`. Palette entry `Log: Mermaid gitGraph としてコピー...`.
- Gerrit: submit `サブミット` (Gerrit's own word; not 送信, which is Submit review), abandon `放棄`, compare with another patch set `比較：ベース` / `比較：パッチセット {n}`.
