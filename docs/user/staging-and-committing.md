**English** · [Čeština](cs/staging-and-committing.md) · [Deutsch](de/staging-and-committing.md) · [Español](es/staging-and-committing.md) · [Français](fr/staging-and-committing.md) · [Italiano](it/staging-and-committing.md) · [日本語](ja/staging-and-committing.md) · [한국어](ko/staging-and-committing.md) · [Nederlands](nl/staging-and-committing.md) · [Polski](pl/staging-and-committing.md) · [Português (Brasil)](pt-BR/staging-and-committing.md) · [Português (Portugal)](pt-PT/staging-and-committing.md) · [Русский](ru/staging-and-committing.md) · [Svenska](sv/staging-and-committing.md) · [Türkçe](tr/staging-and-committing.md) · [Українська](uk/staging-and-committing.md) · [简体中文](zh-CN/staging-and-committing.md)

# Staging and committing

How to see what you changed, choose exactly what goes into a commit, write the
commit, and look at files in more depth (blame, history, the built-in editor).
Terms such as *WIP row* and *hunk* are explained in the
[interface tour](interface-tour.md).

## The working changes

Uncommitted work is the **first row of the Log** (dashed circle, `// WIP`, with
`✎ n` modified and `+ n` added counters). Select it and the right pane shows
the working changes:

![The working changes: Unstaged and Staged files on the right, with the commit form below](images/hero.png)

- **Unstaged Files**: edited, added (untracked) and deleted files not yet in
  the next commit.
- **Staged Files**: what the next commit will contain.
- **Path** or **Tree** (`m`) lists files flat or in folders. **Collapse all**
  (`z`) folds the folders. **View all files** (`a`) shows the whole tree
  instead of only the changed files. Right-click the list's empty space for
  **Expand all** / **Collapse all**, a folder row for **Expand all subfolders** /
  **Collapse all subfolders**; keys `=`, `-`, `*`, `_` (see
  [Mouse and gestures](mouse-and-gestures.md)).
- The title says how many files changed on which branch.

### Stage, unstage, discard

| To | Do |
|---|---|
| Stage a file | Hover its row and click **Stage file**, or select it and press `Ctrl+Space`, or `Ctrl+G s` |
| Stage a folder | **Stage folder** on the folder row |
| Stage everything | **Stage all changes** in the section title, or `Ctrl+G a` |
| Unstage | The same buttons and keys in the Staged section (`Ctrl+G u`) |
| Discard a file's changes | Right-click the file > **Discard changes...** (asks first; `Ctrl+Z` brings the changes back) |

### Sensitive files

Staging a file, a folder or everything first checks what it would add. A file whose
name matches `[privacy] sensitive_patterns` (`.env` and `.env.*` except the
`.example`, `.sample` and `.template` ones, `*.pem`, `*.key`, SSH private keys,
`*.p12`, `*.pfx`, `*.jks`, `*.keystore`, `*.crt`, `*.cer`, `*.der`, `*.kdbx`), or
whose text holds a secret the scanner recognizes, is not staged. The **Sensitive
files** question lists each file with its reason (a pattern, or the kind of secret
with a masked preview, never the value) and offers **1. Stage anyway** or **2. Cancel,
stage nothing**. No answer is highlighted until you press an arrow key or a digit,
so a stray Enter answers nothing; `Esc`, the close cross and a click outside cancel.
A secret found in a file names its line (`config/app.env:12`). To look before you
decide, click a finding (or move over them with `Tab` / `Shift+Tab`, `Ctrl+J` /
`Ctrl+K`) and press the **Show in file** button, `S`, or the palette's **Sensitive
Files: Show in File**: the file opens in the File View scrolled to that line and
highlighting it, and the question comes back untouched when you close the File View
(`Esc`). Looking stages nothing.
It is the same question from the keyboard, a button, a menu, the palette or a
drag-and-drop, because they all send the same request. Canceling stages nothing,
not even the ordinary files of the same stage.

Text files up to 1 MiB each and 32 MiB in all are read; larger files and binary
files are only judged by name. `[privacy] staging_guard = false` turns the check off.
Staging one hunk or some lines is not checked: you are already working on that file.

Right-click a file for its menu: stage or unstage, discard, copy the path, file
history and blame, open in your external editor, diff tool or file manager (set
in [Settings > Diff & editor](settings.md#diff--editor)), and more.

## Ignoring files, and treating them as binary or text

Right-click a file > **Ignore** to ignore it or its extension, or a folder row > **Ignore all files in folder**. GitPoulpi first asks which file the rule goes into: the repository's `.gitignore` (the default), the `.gitignore` of each folder down to the file's own, or `.git/info/exclude` (this clone only, never committed). Files that do not exist yet are marked *new* and created when you confirm. The line is written relative to the chosen file, so `src/a/b.txt` becomes `/a/b.txt` in `src/.gitignore`. Move with the arrow keys, type to narrow the list, `Enter` writes, `Esc` cancels. `Ctrl+Z` undoes a write to a `.gitignore`; `.git/info/exclude` is outside the working tree and is not undoable.

![The list asking which file an ignore rule goes into: the repository's .gitignore, the .gitignore of the file's folder or .git/info/exclude, each with the line it would write](images/ignore-picker.png)

Right-click a file > **Treat as binary** (or **Treat as text** when git already diffs it as binary) writes a `.gitattributes` rule: `binary`, or `text diff`. Pick the file itself or every file with its extension, then the `.gitattributes` to write, with the same list as above (and `.git/info/attributes` for this clone only). A rule already written for the same pattern is replaced, never contradicted. The file's diff updates at once, and the status line says what was written where (and warns when another rule still overrides it). A folder row has **Treat files in folder as** > **Binary** or **Text**. The same rules are in the command palette (**Working Changes: Treat Selected File as Binary...**) and on the command line (`gitpoulpi treat binary|text <paths>`). The File View follows the rule too: a file you treat as Text shows its lines even when it holds a NUL byte, and a file git diffs as binary shows the binary notice.

## Menus, multi-select and conversion on the file lists

The file lists of the right pane (Unstaged, Staged, a commit's changed files and **View all files**) share the same menus and gestures. Right-click a file, a folder row or the empty part of a list, or press `.` (`Shift+F10`, the Menu key) on the focused row, or run **Files: Open Menu of the Row or Selection** from the palette.

- **A commit's file** (and a file in **View all files**): **Open Diff**, **Open file**, **File History**, **File Blame**, open in your editor, in the default program or in **Open with...**, **Show in folder**, **Edit file**, **Copy file path**, **Copy full file path**, **Ignore**, **Treat as binary** or **Treat as text**, **Convert**, **Restore file from this commit** (asks first; like `git checkout <commit> -- <file>` it also stages the file, and `Ctrl+Z` brings the previous file back) and **Delete file...**. A row about the working copy is grayed, with its reason, when the file is not in the working tree. **Diff against working tree** opens the diff of the commit's version against the file on disk. **Create patch from file changes...** writes a patch of that file's change in the commit (a selection and a folder row have it too; in **View all files** on the working changes it patches the file's working-tree changes).
- **A folder row**: **Open folder**, **Copy file path**, **Ignore all files in folder**, **Treat files in folder as**, **Convert**, **Create a file in this folder**.
- **The empty part of a list**: **Create file** and **Open repository folder**. The Unstaged list adds **Stage all changes** and **Discard all changes...**, the Staged list **Unstage all changes**.

![The right-click menu of a file of a commit: open the diff, file history and blame, open in an editor or the default program, copy paths, ignore, convert, restore from the commit and delete](images/commit-file-menu.png)

### Selecting several files

The gestures are the Log's. **Ctrl+click** adds a row to the selection or removes it. **Shift+click** (**Alt+click** in terminals that keep Shift+click) takes the range from the anchor. A plain click starts over and opens the diff as before. `Esc` clears the selection. From the keyboard, `Space` or `Insert` toggles the current row in every list (in Unstaged and Staged `Ctrl+Space` stages: the selection, or the current row) and `Shift+Up` / `Shift+Down` extend the range. A folder row selects every file below it, folded or not. Selected rows are highlighted and the status line counts them. `Ctrl+A` selects every row of the focused list.

Right-click a selected row (or press `.`) for the menu of the whole selection, headed **N files**: stage or unstage, discard, ignore, treat as binary or text, convert, copy the paths, restore from the commit, delete. Stage, unstage and restore are one action each; Discard and Delete ask first and say how many files. A right-click on a row outside the selection starts the selection over with that row.

![Four files selected in the Unstaged list, with the menu headed 4 files: stage, discard, ignore, treat as binary or text, convert, copy the paths and delete](images/file-pane-multiselect.png)

### Converting encoding, line endings and indentation

**Convert** is in the menu of a file, a folder or a selection. `gitpoulpi convert` does the same from the command line.

- **Encoding**: UTF-8, UTF-8 BOM, UTF-16 LE and BE, Windows-1252, ISO-8859-1, ISO-8859-15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5, EUC-KR.
- **Line endings**: LF, CRLF or CR.
- **Indentation**: leading whitespace as spaces or as tabs, with the width of a tab.

The files in the working tree are rewritten in the background with the same functions as the editor's status line. A file that is not valid UTF-8 and has no byte order mark is read as Windows-1252, as in the editor. Binary files, missing files and files the target encoding cannot hold are skipped and reported (*Converted 12 files, skipped 3 (3 binary)*), never damaged; a file that cannot be written raises a failure notification. The whole batch is one entry of the undo journal: one `Ctrl+Z` puts every file back. A file open in the editor blocks the conversion until it is closed.

## Reading a diff

Click a file name (or press `d` on it) and its diff replaces the Log in the
center. `Esc` or `q` brings the Log back.

![A diff with two hunks, each with Discard hunk and Stage hunk buttons](images/diff.png)

The bar above it switches the view:

- **Inline** shows the unified diff; **Split** puts before and after side by
  side (`m` cycles the layouts).
- **Hunks** (`Shift+H`) shows only the changed hunks with a line of context.
  Turn it off to see the whole file.
- **WS** (`s`) hides changes that differ only in whitespace.
- **Wrap** (`w`) wraps long lines instead of truncating them.
- **Map** (`Shift+M`) shows a minimap of the whole file beside the scrollbar.
  Click it to jump. Other occurrences of a selected word are tinted, and marked
  on the scrollbar.
- `n` and `p` (or the arrows on the bar) jump between hunks.
- **Auto** (the last button, `Shift+C`) is the encoding the file is read in. Click it (or press
  `Shift+C`, or run *Diff: Choose Encoding...* from the palette) and pick one from the list:
  **Auto-detect** first, then UTF-8, UTF-8 BOM, UTF-16 LE and BE, Windows-1252, ISO-8859-1,
  ISO-8859-15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5 and EUC-KR (type to
  search the list). The two sides of the file are read again, as exact bytes, in that encoding, so a
  Latin-1 file shows its accents and a UTF-16 file (which git calls binary) shows its lines. The
  choice is remembered for that file's path until you quit, in this view and in the
  [review](review.md). While an encoding is chosen the hunks are this view's own, so staging and
  discarding are off (pick **Auto-detect** to stage again). A slow read says *reading as ...* in the
  title after a moment.

![The same diff in the split layout, before and after side by side](images/diff-split.png)

You can select text with the mouse (drag, or double-click a word) and copy it
with `y` or `Ctrl+C`; only the file text is copied, never line numbers or `+`
and `-` markers.

### Stage a hunk or a single line

On a working-tree diff, each hunk header has **Stage hunk** and **Discard
hunk** (on the Staged side, **Unstage hunk**).

- `Space` stages the hunk under the cursor.
- To stage a few lines, press `v` to start a selection, move with `j`/`k`, and
  press `Enter`. Only *changed* lines count.
- `x` discards the hunk and `Shift+X` the selected lines. Both ask first, and
  `Ctrl+Z` brings the changes back.
- Right-click (or `.`) on a hunk for its menu: stage, unstage, discard, file
  history, blame, copy.

A commit's diff is read-only. A changed picture shows as a picture; see
[Pictures](image-view.md).

## Writing the commit

The commit form is at the bottom of the right pane, and its **Summary** is also
the field in the working-changes row of the Log: typing in either shows in the
other.

![The commit form filled in with an AI-written message: summary with a character counter, description, and the green commit button](images/ai-commit-message.png)

- **Summary** (the counter shows how many of 72 characters are left) and
  **Description**. `Enter` in the summary moves to the description, `Enter`
  there adds a line.
- **Commit changes to N files** (the green button) or `Ctrl+Enter` commits the staged
  changes. If nothing is staged, the button says so and does nothing.
- **Amend previous commit** (`Shift+A`) folds the staged changes and the new message
  into the last commit. A commit that is already pushed will then need a force
  push (with lease).
- **Skip hooks** (`Shift+H`, or **Commit: Toggle Skip Hooks** in the palette) commits
  without running the repository's `pre-commit` and `commit-msg` hooks. It is
  per commit; **Settings > Repo-specific settings > Commit** makes it start
  checked. Shown when the row has room.
- `c` in the Status pane moves the caret to the form. `Esc` leaves it.
- `Ctrl+G c` and `Ctrl+G Enter` (**Git: Commit...** and **Git: Commit Staged
  Changes Now** in the palette) are one command, the commit button from
  anywhere. With a summary in the form and something staged it commits right
  away, honoring **Amend previous commit** and **Skip hooks**, without
  moving the focus. Otherwise it commits nothing: it moves the caret into the
  form's summary (showing the pane if it was hidden or collapsed) and the
  status line says which shortcut commits (`Ctrl+Enter` by default). With
  nothing staged it says so. Nothing is typed in the status line. While an AI
  request is still running it commits nothing and says so (`Esc` cancels the
  request). `Ctrl+Z` takes the commit back like any other.
- Per repository, [Settings](settings.md#repo-specific-commit) can **push after
  each commit**, start the form with a **commit template** (and remove its
  `#` comments from the message) and make a merge **squash**: the form then
  opens with git's squash message.
- With a hosting integration linked, `Alt+R` in the form inserts an issue
  reference; see [Hosting and issue trackers](hosting-integration.md).

With [AI](ai.md) on, the **`✦`** button right of the summary writes a message
from the staged changes (`Ctrl+G i`), and **Compose commits with AI** splits
your work into several commits, below. The text goes into the form for you to
edit; nothing is committed until you press the commit button.

**Compose commits with AI** sits at the right end of the Amend row and shrinks
with the right pane instead of disappearing: the full label, then **Compose
with AI**, then the **`✦`** alone. It keeps its click, its tooltip and its
key (`Ctrl+G Shift+C`) at every width; the Amend and Skip hooks labels give way
to it, never the other way round.

## The commit composer: split your changes into commits

*(Needs AI turned on: see [AI assistance](ai.md).)* Press **Compose commits with
AI** (or `Ctrl+G C`, or right-click the working-changes row). The model proposes
several commits, each with its files; every changed file is in exactly one
commit, and an invalid plan is sent back to the model once. The proposal opens
as a review where nothing has been committed yet:

![The commit composer: four proposed commits, each expandable to its files, with up, down, edit and drop controls and Reset, Cancel and Create 4 commits buttons](images/composer.png)

| To | Do |
|---|---|
| Expand or collapse a commit | `Left` / `Right`, or click the chevron; `*` toggles all |
| Move a commit up or down | The `▲` `▼` on its row, or `Alt+Up` / `Alt+Down` (`K` / `J`) |
| Move a file to another commit | Drag it onto the commit (from the list, or from the working-changes list on the right); or `m` on the file and choose the commit from the dropdown; or `Alt+Up` / `Alt+Down` on the file |
| Put a file in a commit of its own | `n` on the file, or the `+ own commit` button |
| Reword a commit | The pencil, `e`, `Enter` or a double-click: edit the message, `Enter` keeps it, `Esc` discards |
| Leave a commit out | `drop` (`d` or `Delete`); its files stay uncommitted. `restore` brings it back |
| Start over | **Reset** (`r`) returns to the model's plan |
| Create them | **Create N commits** (`Ctrl+S`, `Ctrl+Enter` or `a`) |
| Leave | **Cancel** or `Esc`: nothing done |

Creating the commits unstages everything, then for each commit stages its files
and commits, so your working tree is never touched. If a step fails, it stops
and says how many commits were made. Whole files are staged: a file that was
staged only in part is committed whole. Choose which profile and instructions
the composer uses in *Settings > AI*; see [AI assistance](ai.md#the-commit-composer).

## File View, Blame and File History

The view bar has **File View** (the file's content), **Diff View**, **Blame** and the
`File │ All` selector of the [commit column](interface-tour.md#the-commit-column) (its `File` list is the File History); the same are in a file's right-click menu. They show the file
as of the commit you came from, or as uncommitted (`@ WIP`) when you opened it
from the working changes.

- **Blame** and the **File History** are full views of their own: see [Blame](log-and-history.md#blame) and
  [File History](log-and-history.md#file-history) in *Working with the Log*. `Enter` on a blame hunk or a history
  row goes to that commit in the Log.
- `y` or `Ctrl+C` copies the selected text (in Blame, the code only).
- `Shift+G` jumps to the last line. `q` closes the view.

## The built-in editor

**Edit** (`e`) opens the file's *working-directory copy* in a small editor
over the center pane: syntax colors, line numbers, undo and redo, find and a
status line (position, line ending, indentation, language). It always edits the
working copy, even when you opened it from an old commit's diff.

![The editor showing a Rust file with syntax highlighting and line numbers; the toolbar has Save, Undo, Redo and Find](images/editor.png)

| Key | Action |
|---|---|
| `Ctrl+S` | Save (the file is written to a temporary file and renamed, so it is never half-written; line endings, BOM and the final newline are preserved) |
| `Ctrl+Z`, `Ctrl+Y` | Undo, redo |
| `Ctrl+F`, `F3` | Find, find next |
| `Tab`, `Shift+Tab` | Indent, outdent (following the file's own style) |
| `Ctrl+A`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V` | Select all, copy, cut, paste |
| `Esc` | Close. With unsaved changes it asks **Save** (`s`), **Discard** (`d`) or **Cancel** (`c`) |

Binary files are refused. Tab size, line
endings, syntax highlighting, line numbers and word wrap are in *Settings >
Diff & editor*; see [Settings](settings.md#diff--editor).

With **Follow selection** checked (the box on the editor's bar, also *Editor: Toggle Follow Selection* in the palette and *Settings > Diff & editor*), selecting another file in the right pane while the editor is open opens that file's working copy in the editor instead of its diff or File View. With unsaved changes it first asks **Save**, **Discard** or **Cancel**; Cancel keeps you on the current file. A file that cannot be edited (binary, too large, deleted) opens in its usual view and the status line says why. Closing the editor lands on the selected file's view. On by default. This is for the working changes: picking a file of a commit while editing leaves the editor (asking to save first) and shows that commit's diff of the file, and the commit column, which the editor does not have, comes back with it.

The status line, on the right, shows the file's **encoding**, **line endings** and
**indentation**, then the cursor position and the syntax. The first three are
buttons: each opens a dropdown (arrows or the wheel, type to filter, `Enter` to
choose, `Esc` to close). The palette (`Ctrl+P`) has the same menus: *Editor: Change Encoding*, *Editor: Reopen with Encoding*, *Editor: Convert to Encoding*, *Editor: Convert Line Endings*, *Editor: Change Indentation*.

![The editor's encoding menu open under the UTF-8 chip of the bar: reopen the file with another encoding, or convert it](images/editor-encoding-menu.png)

![The editor's line-ending menu open under the LF chip: convert the file to LF, CRLF or CR](images/editor-eol-menu.png)

- **Encoding.** A file opens in the encoding it announces (a BOM: UTF-8 or
  UTF-16), else UTF-8, else Windows-1252, and the bar says which one it is.
  **Reopen with encoding** reads the file's bytes again with another encoding
  (it asks first when you have unsaved changes): use it when the guess was
  wrong. **Convert to encoding** keeps the text and changes the encoding the
  file is saved in (UTF-8, UTF-8 BOM, UTF-16 LE and BE, Windows-1252,
  ISO-8859-1 and -15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP,
  GBK, Big5, EUC-KR). Nothing is written until you save, and Undo takes it
  back. When the text has characters the target cannot hold, the status line
  says how many and where the first one is, and nothing is converted; a save
  refuses the same way, it never writes `?`.
- **Line endings.** Shows `LF`, `CRLF`, `CR`, or `Mixed` when the file uses
  several. The menu converts every line break of the file to LF, CRLF or CR in
  one undo step. New lines you type use the file's own ending. Your choice wins
  over the *Line endings* setting.
- **Indentation.** Shows `Spaces: 4` or `Tabs: 4`. The menu sets what `Tab` and
  the auto-indent insert (spaces or tabs), the size (1 to 8: the indent step
  and the width of a tab in this file), and converts the leading whitespace of
  every line to spaces or to tabs, in one undo step.

## Stash, patches and Git LFS

- **Stash** and **Pop** are in the toolbar; see
  [Working with the Log](log-and-history.md#branches-tags-and-the-refs-sidebar).
- **Create patch** and **Apply patch** are in the commit and ref menus; the CLI
  has `gitpoulpi patch`.
- Large files stored with Git LFS show as small pointer files until you pull
  their content; see [Git LFS](lfs.md).

## From the command line

Everything on this page is scriptable:

```
gitpoulpi --json status
gitpoulpi stage --file src/lib.rs
gitpoulpi stage --all --allow-sensitive   # after the refusal below
gitpoulpi --json diff
gitpoulpi commit -m "feat: add the helper"
gitpoulpi discard --file src/lib.rs --hunk 1
```

`gitpoulpi stage` refuses a sensitive file the same way: nothing is staged, the exit
code is 2 and `--json` prints `{error: "sensitive_files", files: [{path, reason}]}`
(a rule and a masked preview, never a value). `--allow-sensitive` is the retry. An
MCP agent is refused with the same answer and cannot pass `allow_sensitive` unless
`[mcp] allow_sensitive_staging = true`: it is told to ask you.

See the [CLI reference](cli-reference.md).
