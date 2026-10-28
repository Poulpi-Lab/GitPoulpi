**English** · [Čeština](cs/merge-conflicts.md) · [Deutsch](de/merge-conflicts.md) · [Español](es/merge-conflicts.md) · [Français](fr/merge-conflicts.md) · [Italiano](it/merge-conflicts.md) · [日本語](ja/merge-conflicts.md) · [한국어](ko/merge-conflicts.md) · [Nederlands](nl/merge-conflicts.md) · [Polski](pl/merge-conflicts.md) · [Português (Brasil)](pt-BR/merge-conflicts.md) · [Português (Portugal)](pt-PT/merge-conflicts.md) · [Русский](ru/merge-conflicts.md) · [Svenska](sv/merge-conflicts.md) · [Türkçe](tr/merge-conflicts.md) · [Українська](uk/merge-conflicts.md) · [简体中文](zh-CN/merge-conflicts.md)

# Resolving merge conflicts

When a merge, cherry-pick, revert or rebase stops on conflicts, the Log shows a
banner (*Merge in progress · feature into main · 2 conflicted files*) and the
conflicted files appear in the Status pane with a red `!`. Open one in the **merge
editor**: A (the current side) and B (the incoming side) side by side, with the
**Output**, the file you are writing, underneath. This page is the short version;
the full key list is in [Keybindings](keybindings.md).

## Opening it

- Enter, or a double click, on a `!` row of the Status pane; its right-click menu
  starts with **Open in merge editor**.
- The banner's **Resolve conflicts...** button (it opens the first conflicted file).
- The Diff view of a conflicted file: the **Merge Editor** button (`Shift+E`).
- The palette: **Conflicts: Open Merge Editor**.

Nothing opens by itself: a stopped operation only shows the banner and the `!`
rows.

![The Log of a repository stopped on a conflicting merge: the Merge in progress banner with Resolve conflicts and Abort merge, and two conflicted files marked with a red !](images/conflict-banner.png)

The editor takes the whole window width: the Refs sidebar and the
right panel are hidden while it is open and come back when you close it (`F9`
shows or hides them meanwhile). A and B are side by side whenever the editor is
at least `merge.side_by_side_min_width` columns wide (100 by default, Settings >
Diff & editor; `0` means always); narrower you get one side at a time behind tabs. Long
lines scroll sideways, A and B together (`Shift+←` / `Shift+→`, Shift+wheel or a
sideways wheel), they are never wrapped.

![The merge editor: A (current) and B (incoming) side by side over the Output, conflict 1 taken from A and conflict 2 from B, conflict 3 still to choose](images/merge-editor.png)

## Picking

Each conflict is a framed block in A and in B, with a checkbox on its header and a
`+` / `✓` in front of every line.

| To... | Key | Mouse |
|---|---|---|
| take the line under the cursor (focused side) | `Space` | click the line |
| take the whole chunk of the focused side | `Enter` | click the `[ ]` on its header |
| take A only / B only for this conflict | `a` / `b` | right-click the chunk |
| take both, A then B / B then A | `c` / `Shift+C` | right-click the chunk |
| take nothing | `x` | right-click the chunk |
| swap the order of two picked sides | `s` | click the `1st` / `2nd` badge |
| take all of A / all of B (every conflict) | `Shift+A` / `Shift+B` | toolbar |
| un-pick this conflict (back to unresolved) | `u` in A / B, `Alt+U` anywhere | right-click the chunk, **Unpick** |
| reset the whole file to its original conflicts (asks first when you typed in the Output; `Ctrl+Z` brings it back) | `Shift+U` in A / B, `Alt+R` anywhere | **Reset** next to Take all |
| next / previous conflict | `F7` / `Shift+F7`, `Alt+Down` / `Alt+Up` (`n` / `Shift+N` in A / B) | `▼` / `▲`, or click a mark on a scrollbar |
| a count repeats a motion | `3j`, `2k`, `2n`, `3N` (in A / B) | |
| next / previous conflicted file | `Alt+PageDown` / `Alt+PageUp` | `›` / `‹` in the header |
| show or hide the Base (common ancestor) pane | `Alt+D` (`d` in A / B) | `Base` |
| show the whole file in A and B (**Full file**) | `Alt+F` (`f` in A / B) | `Full file` |
| give A or B more width / back to equal widths | `Alt+Right` / `Alt+Left`, `Alt+0` | drag the divider; double-click it |
| scroll A / B / Base sideways | `Shift+←` / `Shift+→` (`Shift+H` / `Shift+L`) | Shift+wheel, sideways wheel |
| show or hide the Refs and Status panels | `F9` | |
| focus A, B, the Output | `F6` / `Shift+F6` | click |
| jump to a pane, on the conflict you are on | `Alt+1` A, `Alt+2` B, `Alt+3` Output, `Alt+4` Base; or `Ctrl+N` then `1` / `2` / `3` | |
| move to the neighboring pane (A, Base, B, Output), same conflict | `h` / `l` or `←` / `→` in A / B | |
| cycle the Output height | `z` | drag its title |
| undo / redo (picks and typing, one stack) | `Ctrl+Z` / `Ctrl+Y` | |
| write the file, keep it conflicted | `Alt+S` | **Save** |
| write the file and mark it resolved | `Ctrl+S` | **Save & mark resolved** |
| close | `Esc` | `✕` |

Every scrollbar (A / B / Base, and the Output) carries one mark per conflict:
orange while it still needs a choice, green once handled, blue for the current one.
A click on a mark jumps to that conflict (a mark under the scrollbar's thumb is
already on screen, so there the click drags the thumb).

**Full file** shows the whole file in A and B with every difference colored, not only the conflicts:

![Full file mode: the whole file in A and B, with the changes colored](images/merge-editor-full-file.png)

The Output is a real editor: click in it and type, select, cut and paste, find with
`Ctrl+F`. A pick on a conflict you edited by hand replaces your edit (`Ctrl+Z`
brings it back). Saving with a conflict that has nothing picked, or with a line
that still looks like a conflict marker, asks first.

### ✦ Suggest a resolution (AI)

With AI turned on (Settings > AI; see [AI assistance](ai.md)) the toolbar has a
**✦ Suggest** button, a conflict's right-click menu has *✦ Suggest resolution (AI)*, and
`Alt+I` and the palette (`AI: Suggest Merge Conflict Resolution`) do the same. It
works on the current conflict only and runs only when you ask. The answer replaces
that conflict's part of the Output as one edit, marked edited (`✎`), so `Ctrl+Z`
puts the conflict back as it was. It is never saved or staged for you. While it
runs the hint row shows the time spent; `Esc` cancels, and a failure is shown
there too. With AI off none of these controls exist.

After **Save & mark resolved** the file is staged and the conflicted-files list is
back; open the next file yourself. `Ctrl+Z` (or the Undo button)
takes the last "mark resolved" back. When no file is left, the banner offers
**Commit merge** (or **Continue** for a cherry-pick, a revert or a rebase) and
**Abort merge**, which asks first: it puts HEAD, the index and the working tree
back where they were.

A merge whose result equals HEAD (you took our side everywhere) has nothing
staged and no changed file, yet it can still be concluded: **Commit merge**
opens the commit prompt with the merge message, and the commit has both
parents. Likewise the composer's Commit button is enabled with nothing staged
while a merge is stopped.

An empty cherry-pick or revert (the change is already applied) cannot be
committed: git refuses it, and **Continue** says so and points to **Skip**.
**Skip** (banner button, palette entry *Git: Skip Cherry-pick or Revert*, CLI
`conflicts skip`) asks first, then ends the operation with nothing recorded
for it; HEAD, the index and the working tree are back at HEAD. Skip is not
undoable (nothing is lost that was not given up on purpose); a merge has
nothing to skip.

## Conflicts that are not text

A binary file, a Git LFS pointer, a file over 8 MiB, a file deleted on one side,
a leftover of a rename, a symlink, a mode change or a submodule has no lines to
pick. The editor shows a **card** with what each side holds and one button per
choice; choosing resolves and stages the file at once.

| Card | Choices |
|---|---|
| Binary file, LFS pointer, too large | **Keep A** (`a`), **Take B** (`b`), **Take base** (`o`, when there is one), **Open in merge tool** (`m`) |
| Deleted in one side, changed in the other | **Keep the file** (`k`), **Accept the deletion** (`Shift+D`) |
| Rename leftover (the path exists on one side only) | **Keep the file** (`k`), **Delete the file** (`Shift+D`) |
| Symlink, submodule, file mode | **Keep A**, **Take B**, **Take base** |

When the file is a picture (PNG, JPEG, GIF, WebP, BMP, ICO), the boxes also show
the A, B and Base pictures, fitted and read-only, in a window wide enough for
them; see [Pictures](image-view.md#in-the-merge-editor).

![The card of a binary conflict: what A, B and the base hold, and the Keep A, Take B, Take base and Open in merge tool buttons](images/merge-editor-card.png)

`Esc` closes the card without choosing. The external merge tool is
`general.external_merge_tool` in Settings > Diff & editor (`{base}`, `{local}` = A,
`{remote}` = B, `{merged}`; empty means `git mergetool`); the file is staged when
the tool exits successfully.

## Conflict prevention

GitPoulpi can tell you *before* you merge. When the branch you are on would conflict with the branch it started from
(its **base**), a strip appears on the Log's top border row: **Conflicts with origin/main in 2 files**, with a
**Show files** button. It takes the next free border row after the banners of operations in progress, so while one
of those is showing it may wait for room. Nothing is merged: the daemon does an in-memory merge of the two commits
(libgit2, or `git merge-tree --write-tree` with git 2.38 or later; an older git turns the feature off with one line
in the log), so your files, index and refs are untouched.

- **The base** is the target branch of the branch's pull request when GitPoulpi already knows it (the cache the
  Pull requests section keeps; no network call is made for this), else the branch among the **Base branches**
  patterns (Settings > Conflict prevention) with the fewest commits since the fork. A tie goes to the branch whose
  tip is nearest the fork, then to a remote-tracking branch over a local one, then to the name. The branch itself and
  its own remote copy are never the base.
- **The popup** (click the strip, or the palette's **Conflicts: Show Predicted Conflicts**, which works from the
  keyboard) lists the files, and has **Open merge preview** and **Rebase onto base**. They are the base
  branch's own menu rows (*Merge <base> into <branch>*, *Rebase <branch> onto <base>*) run for you: journaled, so `Ctrl+Z` takes them back, and a
  merge that does conflict stops in the merge editor like any other. Left and Right move between the buttons, Enter
  presses one, Esc closes.
- **When it runs**: in the background, about 2 seconds after the repository goes quiet (a commit, a checkout, a
  fetch that moved a branch, a rebase), once per `(HEAD, base)` pair. The strip hides as soon as HEAD moves and comes
  back, or not, with the new answer. A detached HEAD is not checked.
- **Off**: **Enable proactive conflict detection** in Settings > General, or for one repository on its **Conflict
  prevention** page. From a script, `gitpoulpi conflicts check` prints the same answer as JSON ([CLI
  reference](cli-reference.md)).

## Agents

An agent can show you a conflict (`gitpoulpi ui show conflict <path> [--conflict
N]`: the editor opens on it, nothing is picked) or **propose** a resolution
(`gitpoulpi ui propose resolution <path> --pick 1=a,2=ba` or `--content-file F`,
needing *Let AI agents propose changes for me to confirm*). A proposal opens the
editor with the picks (or the whole text) already in the Output and a purple
*✦ Proposed by claude-code* banner. Review it, change what you want and press
**Ctrl+S**: only that writes or stages anything. Closing the editor rejects the
proposal. See [MCP and agents](mcp-and-agents.md).

From a script, the same operations are `gitpoulpi conflicts list | show | resolve |
unresolve | state | abort` ([CLI reference](cli-reference.md)); `conflicts check` predicts them before a merge.
