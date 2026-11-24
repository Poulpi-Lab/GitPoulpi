**English** · [Čeština](cs/mouse-and-gestures.md) · [Deutsch](de/mouse-and-gestures.md) · [Español](es/mouse-and-gestures.md) · [Français](fr/mouse-and-gestures.md) · [Italiano](it/mouse-and-gestures.md) · [日本語](ja/mouse-and-gestures.md) · [한국어](ko/mouse-and-gestures.md) · [Nederlands](nl/mouse-and-gestures.md) · [Polski](pl/mouse-and-gestures.md) · [Português (Brasil)](pt-BR/mouse-and-gestures.md) · [Português (Portugal)](pt-PT/mouse-and-gestures.md) · [Русский](ru/mouse-and-gestures.md) · [Svenska](sv/mouse-and-gestures.md) · [Türkçe](tr/mouse-and-gestures.md) · [Українська](uk/mouse-and-gestures.md) · [简体中文](zh-CN/mouse-and-gestures.md)

# Mouse and gestures

Everything you can do with the mouse also has a keyboard equivalent. See
[Keybindings](keybindings.md). The names of the areas are in the
[interface tour](interface-tour.md).

## General rules

- **Click** a row to select it. This works in every pane, list, menu and
  dialog.
- **Double-click** a row to open or run it. A double-click on a local branch,
  a tag or a ref pill checks it out. A double-click on a repository row opens
  the repository.
- **Right-click** a row for its menu. `.`, `Shift+F10` or the Menu key opens
  the same menu for the selected row.
- **The wheel** scrolls the area under the pointer. In a list, it moves the
  view only. The selection stays where it is, even when it scrolls out of
  view. Only a click or a key moves the selection.
- **Shift+wheel** scrolls sideways, in the diff, the File View and the editor
  when word wrap is off.
- **Hover** a button, a row or a scrollbar thumb to light it up. Many terminals
  cannot change the mouse pointer, so this light is often the only sign that
  something is clickable. Rest the pointer on a button or a Log cell for a
  tooltip.
- **Click outside** a text field to leave it. What you typed is kept. The
  Refs filter applies what you typed, as `Enter` does. The Log search bar
  closes, as `Esc` does.

## Closing windows and dialogs

Every framed window and dialog has a `✕` at the top right of its border, and the
same rules everywhere:

- **Click `✕`** to close it. `Esc` does the same, and closes the innermost thing
  first (an open dropdown, a text field, a confirmation) before the window.
- **Click outside** a dialog, a help or info panel or a popup to close it. An
  outside click never closes a full view (Settings, the Diff view, the File
  View, the editors, Review, Board): they have no outside.
- A click **inside** a panel never closes it.
- A mouse press (left or right) while a `Ctrl+G` or `Ctrl+N` chord waits for its second key cancels the chord like `Esc`, and then acts as usual (a click still selects, a right click still opens its menu).
- A form that holds text you typed (a new issue, a pull or merge request) ignores
  an outside click, so a stray click cannot lose it. `✕` and `Esc` ask
  **Discard changes?** first; **Keep editing** goes back. A form with nothing
  typed closes at once, and so does a single-line dialog.
- The command palette, the pickers (tab, theme, language), the small review,
  board, merge and composer dialogs and the confirmation prompt carry the `✕`
  too, and so do the Activity Log's clear confirmation, the Undo history /
  Reflog / Lost work confirmations and prompts (both also move by their title)
  and the blame card (its `✕` dismisses the card like `Esc`). Context menus and dropdowns have none: they close on `Esc` or a click
  outside.

## Lists in Settings

The lists of Settings (Profiles, AI profiles, Workflow, Open in, Images) take the
mouse like this. **Click a sub-tab** (General or the list) to switch to it; `Alt+Left` / `Alt+Right` does the same.

- **Click** a row to select it, **double-click** to go to its fields, **wheel** to
  scroll without selecting.
- **Press a row and drag** it up or down to reorder: a bar shows where it will
  land, release drops it, `Esc` cancels. The keyboard twin is `Alt+Up` /
  `Alt+Down`.
- **Click `+ Add`, `Duplicate`, `Remove`, `↑`, `↓`** under the list. Keys:
  `Insert`, `Ctrl+D`, `Delete`, `Alt+Up` / `Alt+Down`; see
  [Settings](settings.md#lists).

## Lists in popups

These rules apply to every list in a popup that has a filter field: the
command palette, its pickers (theme, language, graph colors), the tab picker,
searchable dropdowns, the issue-reference list and the help.

- **Click** a row to select it.
- **Double-click** a row to run or pick it, as `Enter` does.
- **The wheel** scrolls the list. It does not move the selection.
- When the list is longer than the popup, a **scrollbar** appears on its
  right edge. Drag the thumb or click the track.

A short dropdown without a filter field picks the row on the first click.
A context menu runs the row on the first click. The keys for these lists are
in [Keybindings](keybindings.md#filterable-popup-lists).

## Scrollbars

Every area that can scroll shows a thin scrollbar on its border when its
content does not fit: the Log, the Refs sections, the file lists, the diff,
the File View, the editor and popup lists.

- **Drag the thumb** to scroll.
- **Click the track** to bring the thumb under the pointer: the view jumps
  there in proportion, and keeping the button down keeps dragging.
- **Hold the button on an end arrow** to keep scrolling. It starts after a
  short delay. Move off the arrow to pause.
- A scrollbar moves the view only. It never moves the selection.

In the diff and the editor, the vertical scrollbar also shows the whole file
in small marks: green for added lines, red for removed lines, amber for
changed lines, `◆` for the cursor line and `▌` for each other place where the
selected text appears. The merge editor marks each conflict on its
scrollbars. Click a mark to go there.

## Failures and the Activity Log

- **Click the red error** in the status line to open the Activity Log on its
  entry; **click Copy / the suggested button** (Retry, Pull first, Stash and
  retry, Open Integrations) on the entry.
- **Click Activity** at the right end of the status line to open the Activity
  Log. Click a tab to switch, click a level checkbox (or **All** / **None**)
  to show or hide those lines, click a Repository entry to open its details,
  click **Clear log** on the Application tab (it asks first).
  Scroll long lines sideways with `Shift` + wheel or the horizontal scrollbar.
  See [The Activity Log](troubleshooting.md#the-activity-log).
- **Click `[x]`** on the jobs indicator at the right of the status line to cancel the running job, **click its label** to open the jobs panel. The **Activity** button at the bottom right of the Home tab (or `Ctrl+N Shift+G`) opens the Activity Log on All repositories (its Scope box, a dropdown under the tabs, picks a repository or a workspace).

## Resizing

- **Drag a divider** between the Refs sidebar, the Log and the right pane. The keyboard moves them a notch at a time: `Ctrl+N Left` / `Ctrl+N Right` move the Refs | Log divider, `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` the Log | right pane divider (the divider goes the way the arrow points).
  Drag it past the minimum width to collapse that pane. The keyboard's twins are `Ctrl+N Left` / `Ctrl+N Right` (the divider between the Refs pane and the Log) and `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` (the divider between the Log and the right pane), one notch each, the divider going the way the arrow points; the palette has them as `Navigation: Widen Refs Pane`, `... Narrow Refs Pane`, `... Widen Status Pane` and `... Narrow Status Pane`, and `Navigation: Toggle Refs Pane Collapsed` / `... Toggle Status Pane Collapsed`.
- **Drag the bottom border** of a Refs section to change its height.
- **Drag the bottom border** of the commit message block in the right pane.
- **Drag the border between two Log headers** (by default the gap, from the header or from anywhere down the table; Settings > Log & graph > Column resizing) to resize a column (the keyboard: `Shift+C` to pick a column, then `]` / `[`).
- **Double-click the border between two Log headers** to fit the column on its left to its content (the keyboard: `Shift+C` to pick a column, then `=`; `+` fits every column). The last column has a grip on its own last cell: drag it to resize that column, the Message column taking the difference.
- **Scroll the Log sideways** (the `scroll` column layout; Settings > Log & graph > Column layout, or the chooser's `Scroll sideways` checkbox): the horizontal wheel or `Shift`+wheel over the table, the bar on the bottom border (drag its thumb, press the bare track, click its end arrows), or `Shift+Left` / `Shift+Right`. In the default `fit` layout the columns fill the pane and there is nothing to scroll.
- **Scroll a list pane sideways** when a row is wider than the pane: a Refs section (a long branch, tag or stash name), the Unstaged and Staged lists, the commit's (or a comparison's) changed files and the commit details' info lines (id, author, authored date, counts). The horizontal wheel or `Shift`+wheel over the pane scrolls it; a bar appears on the pane's bottom border only while something overflows (drag its thumb, press the bare track, click its end arrows), so it never takes a row. From the keyboard, `Alt+Left` / `Alt+Right` scroll the pane that has it (the Log scrolls the commit details; inside the changed files, the files); the palette has `View: Scroll the Pane Left/Right`. The whole row scrolls (indent, status marker, chevron and name together); the pane's title, its border, the scrollbars and a hovered Status row's Stage/Unstage button stay put. A name is no longer cut with an ellipsis: its tail is one scroll away (a branch with extra push targets keeps its `⇉` badge in view instead, so its name still gives way). The offset is per pane, stops at the widest row in view and is forgotten when you switch repository. The commit message block wraps its text, so it has nothing to scroll. While a section is scrolled sideways its bottom border carries the bar, so resize that section by its keyboard commands or from a section that fits.
- **Drag a Log header** to move the column: a ghost of its name follows the pointer, the header it left is dimmed and an accent bar marks the slot where it would land. The columns move when you release; `Esc` cancels, and the pointer shows the open hand while you carry it (the keyboard: `Shift+C` to pick a column, then `<` / `>`).

GitPoulpi remembers every size. See
[Sizes are remembered](interface-tour.md#sizes-are-remembered).

## Title bar, tab row and toolbar

- **Click <img src="../../assets/logo/gitpoulpi.png" alt="GitPoulpi icon" height="16">** in the title bar to open **About**. In About, the octopus's
  pupils follow the pointer; click the logo to spin it to the next color,
  right-click it for the list of colors, click a link to open it.
- **Click Open in ▾** in the title bar for the list of programs. See
  [Open in](open-in.md).
- **Click the version** (`v1.0.0`, or `vX.Y.Z available`) in the title bar to open the Release notes tab.
- **Click the magnifier and `Global search`** in the title bar, just left of the gear, for the
  [global search](search.md).
- **Click the gear** in the title bar, just left of the ×, for the Settings.
- **Click ×** at the right end of the title bar to quit.
- **Click a tab** to switch to it. **Click its `x`** (shown on the active tab
  and on the one under the pointer) to close it. **Drag a tab** sideways to
  reorder it. **Right-click a tab** for the tab menu.
- **Middle-click a tab** to close it, like its `x`.
- **Click `‹` or `›`** at the ends of the tabs (shown when they do not all fit) to scroll the row by one tab; hold the button to keep scrolling, or use the wheel over the row. Dragging a tab onto an arrow scrolls the row too. The row also scrolls by itself to show the current tab whenever it changes.
- **Click the `+`** after the last tab for a menu: New tab (`Alt+T`) and, while their tabs are closed, Radar and Workspaces (`Alt+J`).
- **Click the house icon** for the Home tab.
- **Click the Radar or Workspaces tab** to open it; its `x`, a middle click or `Ctrl+W` closes it. **Click the workspace chip** (`Alt+W`) to switch workspace.
- **Click Quit & cd here** (shown at the right of the tabs when you started GitPoulpi through the `gp` shell wrapper) to quit and leave your shell in the active tab's folder. See [Following the active tab's folder](following-the-tab-folder.md).
- **Click a toolbar button** to run it. **Click `▾`** beside Pull, Push or
  LFS for its list. **Click `⋯`** for the buttons that did not fit.
- **Click the profile combobox** at the right end of the toolbar to choose the
  commit identity for this repository.

## Home tab

- **Click** a row to select it. **Double-click** it to open the repository.
- **Click the star** to add or remove a favorite.
- **Click Open, Switch, Close or Remove** on a row, or **`⋮`** for the row's
  menu. **Right-click** a row for the same menu.
- **Click a section title** to fold the section.
- **Click Browse, Clone, Init, Collapse all or Expand all** at the top, and
  **Close all tabs** or **Remove all** on a section title.

In the **Browse** folder picker: click a place or a folder to go there,
double-click a folder to open it, click the path field to type a path. Tick
**Show sibling folders** (`Alt+S`, remembered) and the left tree also lists the
other folders of every open level: the folders on the open path show `▾`,
the others `▸`, and a click on one goes there. In
**Clone** and **Init**: click a field to type in it, and click a button or a
check box to use it.

## Refs sidebar

- **Click** a row to select it. **Double-click** a local branch or a tag to
  check it out. **Double-click** a worktree or a submodule to open it in a
  tab. The **+** on the WORKTREES header creates a worktree; right-click a
  worktree for Open in a new tab, Open in, Merge and Remove ([Worktrees](worktrees.md)).
- **Hover** a row to show the icon at its left. **Click** it to hide the ref
  in the Log, or to show it again. While **Solo** is on, the same place solos
  the ref. On a folder row, the icon acts on every ref in the folder.
- **Click Stop Soloing** or **Show All** in the sidebar title to clear every
  solo or every hidden ref.
- **Right-click** a row for its menu: check out, merge, fast-forward, rebase,
  create a worktree, rename, delete, hide, solo, and more. A remote's own row
  has **Fetch**, **Edit**, **Remove** and other remote actions. A stash has
  **Pop**, **Apply** and **Drop**. A grayed row says why it cannot run.
- **Click `+`** on the REMOTE title to add a remote.
- **Click a folder row** to fold or unfold it.

### Pull requests and issues

| Gesture | Action |
|---|---|
| Click a row | Select it. |
| Double-click a row | Open it in the browser. |
| Right-click a row | Its menu: open, copy the URL or number, check out the source branch, create a branch from an issue. |
| Click `+` on the section title | Open the **Create Pull Request** form or the **New issue** form. |
| Click the filter row (`Open ▾`) | Choose which requests or issues to list. |
| Click a group title | Fold or unfold the group. |
| Click the provider or repository row (ISSUES) | Choose the issue source. **None (unlink this repository)** removes it. |
| Click the query row (ISSUES) | Type a JQL or WIQL query (Jira, Azure DevOps), or choose Trello lists. |
| Click the toolbar chip (`GH #12`, `GL !53`, ...) | Open the pull request in the browser. Right-click it for **Open in browser**, **Copy URL**, **Refresh** and **Switch profile**. |
| Click the dim `+ PR` chip | Start a pull request for the pushed branch. |

See [Hosting and issue trackers](hosting-integration.md).

## The Log

- **Click** a commit to select it. Its details appear in the right pane.
- **Ctrl+click** adds a commit to the selection or removes it.
  **Shift+click** selects a range (**Alt+click** in terminals that keep Shift+click
  for their own text selection, such as kitty and Windows Terminal). Right-click a selected commit to cherry-pick
  them all. The selection can include the WIP row and stashes; two or more
  rows turn the right pane into a comparison, see
  [Comparing commits](log-and-history.md#comparing-commits).
- **Click a card** of the comparison pane to move the Log cursor to that row;
  **click Swap** to trade the From and To ends. A plain click on a row leaves
  the comparison.
- **Click the WIP row** for the working changes. **Click its `// WIP` text**
  to type the commit summary there.
- **Hover** a cell for the full text: the message, the author and email, the
  full date, the full hash.
- **Hover a `+N` badge** for the other refs on the commit. Move into the popup
  to click a ref there. Double-click a ref in the popup to check it out.
- **Double-click a ref pill** to check it out.
- **Right-click a commit** for the commit menu. It has one submenu for each ref
  on the commit.
- **Right-click a ref pill** for one menu with the ref's actions and the
  commit's actions.
- **Click the funnel in the Author header** (or `Shift+A` in the Log, or the palette's `Log: Filter by Author...`) to choose which authors' commits stay lit; the rest dim like a search does. `Enter` or a click on a row toggles an author, the **Clear filter** row (✕) drops the filter, `Esc` empties the search text and then closes the popup, keeping the filter. `Esc` in the Log drops a lit filter. **Right-click the lit funnel** to clear it (the palette: `Log: Clear Author Filter`).
- **Right-click the header** to show or hide columns: a checklist (`[x]` / `[ ]`) with a ✕ in its title; click a row or use the arrows and `Space` / `Enter`, then close it with the ✕, `Esc` or a click away (the palette: `View: Log: Choose the Columns to Show`). Below the checkboxes: `Fit <column> to content` (the column right-clicked), `Fit all columns to content` and `Reset column widths`. The keyboard twin of the right-click: `Shift+C` to pick a column, then `.` (or `Shift+F10`) opens the same menu on it.
- **The Radar's columns** work the same way: drag the separator after the repository, `#`, author or age column to size it, **double-click it** to fit the column to its widest entry (`C` picks a column, then `=`; `+` fits every column), and **right-click a column title** (or `C`, then `.`) for `Fit <column> to content` and `Fit all columns to content`.
- **Click `↑`, `↓` or `✕`** in the search bar to go to the previous or next
  match, or to close the bar.
- **Click `#12`, `ABC-12` or `AB#12`** in a commit message, in the right pane,
  to open that issue or pull request in the browser.

### Left pane sections

**Folders** (branches and tags named `a/b/c`, the files of the right pane in
tree view) fold and unfold all at once: **right-click the header or the empty
space of a section** that has folders for **Expand all** / **Collapse all**
(and **Show/hide sections...**, which leads to the checklist below), and
**right-click a folder row** for **Expand all subfolders** / **Collapse all
subfolders** (the folder and everything under it). Keys, in the Refs pane, the
Unstaged and Staged lists and a commit's changed files alike: `=` expands all,
`-` collapses all, `*` (or `Shift+Right`) expands the folder under the cursor
and everything below it, `_` (or `Shift+Left`) collapses it. Palette: *Refs:
Expand All Folders*, *Refs: Collapse All Folders*, *Refs: Expand the Selected
Folder and Its Subfolders*, *Refs: Collapse the Selected Folder and Its
Subfolders* and the four *Files:* twins for the focused file list. While a name
filter is active (the Refs filter, a commit's *Filter files*), every folder
holding a match is shown open whatever you had folded, and your own folds are
neither changed nor saved: clear the filter and they come back. A fold you make
during the filter acts on what is shown and is forgotten with the filter.

**Right-click a section header** (a section without folders, or the empty part of the left pane) for a
checklist of its sections: Local, Remote, Worktrees, Stashes, Tags, Pull
requests, Issues and Submodules. A click unchecks or checks a section; the menu
stays open. A hidden section is not drawn and the keyboard skips it. The last
visible section cannot be unchecked. The choice is kept per repository and across restarts (Settings, *Left pane sections*, can make it global instead: one set for every repository, and then *Use for all repositories* is not offered); a repository with none of its own follows the default (the global `ui.sidebar_sections_hidden`). *Use for all repositories* makes the current choice the default, and *Reset to default* drops this repository's own choice (palette: *Left pane: Apply These Sections to All Repositories*). Palette: *Left pane: Show/Hide
Sections...*, one *Show/Hide* command per section, *Reset Sections to Default*.

### Dragging refs

Press on a branch, a remote branch or a tag in the Refs sidebar, or on a ref
pill in the Log, and drag it onto another ref. The status line says what you
carry. A target that can take the drop lights up. Release to get a menu. `Esc`
cancels the drag. The Log stays where it is while you carry: pressing a ref
only scrolls the Log to its commit when it turns out to be a click (the jump
happens when the button comes up), so you can scroll to the drop target first
and then pick the ref up.

| Drag | Menu |
|---|---|
| Local branch X onto local branch Y | **Merge X into Y**, **Rebase X onto Y**, **Interactive rebase X onto Y...**, **Fast-forward Y to X** |
| Local branch onto a remote branch | **Push X** to that remote |
| Remote branch onto local branch Y | **Merge**, **Rebase Y onto it**, **Fast-forward Y to it**, and **Pull** when it is the upstream of Y |
| Tag T onto any branch, remote branch or tag | **Move tag T here**; marked *(not a fast-forward: T would no longer be an ancestor)* when the target does not descend from the tag's commit (your choice); grayed *already there* when it is already on that commit |
| Tag T onto local branch Y | also **Merge tag T into Y**, and below a rule the rarer **Merge Y into tag T and move the tag** (a merge commit of the tag's commit and Y, detached, then the tag moves onto it) |

When the action changes a branch that is not checked out, GitPoulpi checks it
out first. If git refuses the checkout, nothing changes. Only tags can be
dropped on a tag, and a branch cannot be dropped on one.

Moving a tag keeps its kind: a lightweight tag stays lightweight, an annotated
tag is re-created annotated with its message and tagger line carried over
unchanged (a signed tag is refused, its signature would no longer match). Only
your local tag moves: a remote that has the tag keeps the old position, the tag
turns *differs* and the status line says so; push it yourself when you mean
it. The move and the merge-and-move are each one undo step (undo puts the old
tag object back). A merge into a tag that conflicts changes nothing. Palette:
`Refs: Move Selected Tag to Selected Commit...` moves the Refs pane's tag to
the commit selected in the Log (it asks first when that is not a
fast-forward); every other target goes through mark then drop below.

Keyboard: "mark then drop" does what a drag does, for any pair of refs. In
the Refs pane press `m` on a branch (palette: `Refs: Mark as Drop Source`).
The ref shows a `◆` and is underlined, also on its pills in the Log, and the
status line says where to go. Select another ref, in any section, and press
`Shift+M` (palette: `Refs: Drop Marked Ref Here...`): the same menu a drag
opens appears, and its rows run exactly as they do after a drag. The mark is
spent once the menu is open; `Esc`, or `m` on the marked ref, clears it
earlier. The palette command `Refs: Drop Selected Ref on Checked-Out
Branch...` opens the menu for the selected ref and the current branch.

## The right pane

### Working changes

- **Click** a file to open its diff. **Click the same file again** to go back
  to the Log. If the file is open in the editor with unsaved changes,
  GitPoulpi asks first.
- **Hover** a row to show its **Stage file** or **Unstage file** button. A
  folder row has **Stage folder** or **Unstage folder**.
- **Click Stage all changes** or **Unstage all changes** on a section title.
- **Click a section title** (`Unstaged Files`, `Staged Files`) to fold it.
- **Click Path or Tree** to choose a flat list or folders. **Click Collapse
  all** or **Expand all** to fold every folder.
- **Click `✗`** at the left of the title to discard all changes (asks
  first). **Click `✦`** at the right to explain the changes with AI.
- **Right-click a file** for its menu: stage, discard, ignore, LFS, stash
  the file, history, blame, open in an external program, copy the path,
  create a patch, edit, delete. A conflicted file starts with the
  **Resolve using ...** rows.
- **Right-click a folder row** for the folder menu: stage, discard, ignore,
  stash, create a patch, open the folder, create a file in it.
- **Right-click the empty part** of Unstaged Files for **Create file**.
- **Ignore**, **Treat as binary** and **Treat as text** (file and folder menus) first open a list of the `.gitignore` or `.gitattributes` files the rule can go into. Click a row, then press `Enter` (or use the arrow keys); `Esc` or a click outside cancels and writes nothing.
- **Ctrl+click** a file or folder row to add it to the selection or remove it, **Shift+click** (**Alt+click**) to take a range; the same in a commit's changed files and in View all files. A plain click starts over. **Right-click a selected row** for the menu of all selected files.
- **Right-click the empty part** of a list also offers **Open repository folder**; Unstaged adds **Stage all changes** and **Discard all changes...**, Staged adds **Unstage all changes**.
- **Convert** (file, folder and selection menus) rewrites the encoding, the line endings or the indentation of the files on disk; one `Ctrl+Z` undoes the batch.

### The commit form

- **Click** the Summary or the Description to type in it.
- **Click Amend previous commit** to turn amend on or off.
- **Click Commit changes to N files** to commit. When the button is grayed, a click says
  why (for example *Stage changes to commit*).
- **Click `✦`** at the right of the Summary to write the message with AI.
- **Click `#▾`** at the end of the Summary for the list of issues. Click one
  to insert its reference.
- **Click Compose commits with AI** to open the commit composer.

### Commit details

- **Click** a changed file to open its diff. **Click it again** to go back to
  the Log.
- **Drag** the bottom border of the message to resize it. The wheel over the
  message scrolls a long description.
- **Right-click** the summary, the description or the author block for *Amend
  summary...*, *Amend description...* or *Amend author...*; **double-click** one (or the authored-on date) to
  open the full Amend dialog with the caret in it. The keyboard equivalents are `Ctrl+G Shift+E` (all fields) and `Ctrl+G Shift+A` (the author); summary and description are in the palette.
- **Right-click** a changed file, a folder row, the empty part of the list or a selection for its menu (open, history, blame, open in a program, copy paths, ignore, treat as binary or text, convert, restore from the commit, delete). **Ctrl+click** and **Shift+click** select several files.

See [Staging and committing](staging-and-committing.md).

## The diff and the File View

- **Click a button** in the bar above the diff: File View, Diff View, Blame,
  the `File │ All` column selector, Edit, `↑` `↓`, Inline, Split, Hunks, WS, Wrap, Map, Encoding. A button that
  is on is highlighted. A click on `File` or `All` shows that list in the commit
  column; a click on the segment already selected hides it (see
  [The commit column](interface-tour.md#the-commit-column)).
- **Click `✕`** in the title to close the file.
- **Hover a hunk** to show **Stage hunk** or **Unstage hunk**, and
  **Discard hunk**.
- **Hover a changed line** to show its own **+ Stage**, **− Unstage** and
  **✗ Discard** buttons at the right end of the row.
- **Right-click a line** for the hunk menu: stage, unstage or discard the hunk
  or the selected lines, file history, blame, copy.
- **Click the minimap** to move the view to that line. Hold the button down
  and move to scroll.

Every discard asks first. When a discard deletes untracked files, the folders that become completely empty go with them (never one that still holds a tracked, ignored or hidden file, or one that was already empty); Undo brings the files back with their folders.

### Blame and File History

- **Blame.** Rest the pointer on an annotation to see the hunk card; click selects a hunk, right-click opens its
  menu. Drag the annotation column's border to resize it (saved). Click or drag on the minimap to scroll.
- **File History.** Click selects a commit; `Shift+click` selects a range to compare; double-click shows the commit
  in the Log; right-click opens the row menu; drag the border between the list and the diff to resize it; the ✕ on the list hides it (click the `›` strip to bring it back); click
  the bar's `File View` / `Diff View` / `Blame` to switch what the right side shows, and the `File │ All` selector to pick the list of the column. The wheel scrolls the half under
  the pointer. Each has a keyboard twin ([Blame](log-and-history.md#blame), [File History](log-and-history.md#file-history)).

### The commit column (All list)

- **Hover a row** for the Log's tooltip (the whole message, the author's name and email, the full date, by the part under the pointer).
- **Right-click a row** for its menu: show it in the Log, the commit's diff of the file, the file at that commit, its blame, then the Log's menu for the commit. The menu key does the same from the keyboard.
- **Drag a boundary of the header** (`ui.column_resize`: by default the one-cell separator between two parts, marked by a muted `│` in the header and drawn as an accent line down the column while the pointer is on it, a drag from anywhere along it resizing; with `header`, the first letter of the next name, lit while the pointer is on it) to size the part on its left: lanes, message, author, date, refs, changes or SHA. The cell right after the last part is the boundary of the last part.
- **Double-click a boundary of the header** to fit the part on its left to its content, as the Log's boundaries do: the width that shows its widest row whole, never wider than the column (the keyboard: click a row or `Ctrl+N` to give the column the keys, `Tab` to pick the part, then `=`; `+` fits every part; the palette has `Fit the Part to Its Content` and `Fit All Parts to Content`). The message keeps what the other parts leave.
- **Drag a name of the header** over another to move the part there: the same ghost, dimmed source and accent bar as the Log's header, and the part moves on release (`Esc` cancels; the keyboard's `<` / `>`, after `Tab`, do the same one place at a time).
- **Right-click the header** to choose which parts are shown (Ref, Graph, Author, Date, Changes, SHA; Message is always shown); the checklist has a ✕ in its title. Below it: `Fit <part> to content` (the part right-clicked) and `Fit all columns to content`. The palette has `Choose the Columns` (the menu from the keyboard, on the part `Tab` picked) and a toggle per part.
- **Shift+wheel** or a horizontal wheel scrolls the rows sideways when they are wider than the column.
- **Drag the right border** of the column to resize it. **Click the `›` strip** (or the ✕ on the column, or **click the selected segment** of the bar's `File │ All` selector) to bring it back or hide it.

### Selecting and copying text

You can select and copy the text of the diff, the File View, Blame and
File History.

- **Press and drag** to select text. The selection stays on the same text
  when you scroll.
- **Double-click** to select a word.
- **A plain click** does not select text. It moves the cursor or selects the
  row, and clears the text selection.
- **`y` or `Ctrl+C`** copies the selection. With no selection, they copy the
  current line.
- **Right-click** in the diff for **Copy selection**, **Copy line** and
  **Copy hunk**.

GitPoulpi copies only the text of the file. Line numbers, `+` and `-` marks,
hunk headers and the Blame columns are not copied. In Split, a selection stays
on the side where you started the drag.

In the diff, a drag also selects the lines it crosses, as `v` does. `Enter`
then stages those lines. `Esc` first clears the text selection, then the line
selection, then closes the diff.

Selecting text also tints every other place where that text appears in the
diff and the editor.

## Text fields

This applies to every text field: filters, the search bar, the palette, the
Summary and the Description, dialog fields and Settings values.

- **Click** to put the caret there.
- **Press and drag** to select text.
- **Double-click** to select a word.
- Type to replace the selection. `Backspace` or `Delete` removes it.

The keys for text fields are in [Keybindings](keybindings.md).

## The editor

- **Click** to put the caret there. **Shift+click** extends the selection.
- **Drag** to select. Drag past the top or bottom edge to scroll.
- **Double-click** to select a word.
- **The wheel** scrolls without moving the caret.
- **Click Save, Undo, Redo or Find** in the bar. In find mode, **`↑`** and
  **`↓`** go to the previous and next match.
- **Click `✕`** to close. GitPoulpi asks first when there are unsaved
  changes.

## Pictures

See [Pictures](image-view.md) for the mouse in the image view.

## Moving and resizing dialogs

Dialogs can be moved, and the larger windows resized:

- **Drag the title bar** to move a dialog (the close cross keeps its click). The command palette and the pickers have no separate title bar: drag the top border.
- **Drag the left, right or bottom border**, or a bottom corner (the `◢` grip), to resize a window. The dialog stays on screen and never gets smaller than its minimum (60 x 16 for the Undo history, smaller for the others). A resized window uses the room: text areas and lists grow, text wraps to the new width.
- The drag goes on when the pointer leaves the dialog, and releasing outside never closes it.
- **Drag the dividers** inside the Undo history's Reflog / Lost work view; double-click one to reset it.
- The keyboard twins are `Ctrl+N` then `m` (arrows move, `Shift`+arrows resize, `0` resets, `Enter` or `Esc` leave) and `Alt`+arrows for the dividers; the palette has `Dialog: Move or Resize` and `Dialog: Reset Position and Size`. They act on the dialog on top. `Ctrl+N` works the same over every surface: it shows a badge on the controls of the window, dialog or popup in front (fields, lists, tabs, buttons and the close cross), the key after it presses that control, `m` starts the arrange mode and `Esc` cancels; the status line names `<badge>`, `m=arrange` and `Esc`. That holds for the command palette, the pickers, the workspace search and the help overlay too, where `Ctrl+N` no longer moves down the list. `Alt+G` (`palette.arrange`) is one more way, valid over every dialog and popup, to arrange the one in front without the prefix.
- **One setting** (`ui.dialog_geometry`, Settings > Appearance > Layout) covers every dialog: each remembers its own place and size for this session, on disk for the next launch, or never.

| Dialog | Move | Resize |
|---|---|---|
| Undo history / Reflog / Lost work, Activity Log, jobs panel, folder picker | yes | yes |
| Command palette, tab / theme / language pickers, workspace search | yes | yes (the list grows with the height, the rows with the width) |
| Clone (the repository list), Pull request and New issue (the description), "What is sent" (the pane), Mermaid (the preview), review options (the fields, width only) | yes | yes |
| Settings long-text editors (templates, hooks, AI prompts), the workspace form's AI box | yes | yes (the text area) |
| History rewrite: the two AI helper dialogs | yes | yes (the description, the sent pane) |
| About, update panel, security check, predicted conflicts, "Over the AI limit", tag delete, sensitive files | yes | no (the size follows the text) |
| Init, Worktree, Sparse checkout, Flow dialog, issue-flow dialogs, commit message (reword) form, Workspace form, label / milestone pickers | yes | no |
| Confirmations (every "Are you sure?") | yes | no |
| Review and Board dialogs: their small dialogs (comment, finish, pick, columns) | yes | no |
| Keybindings page popups (reset confirmation, key capture), the AI answer popup | yes | no |
| Settings: the repository authentication results and confirmation | yes | no |
| History rewrite: purge, identity and question dialogs | yes | no |
| Move-a-file popup of the commit composer, merge notices | yes | no |

Not movable: the full-page screens (Settings, Help, the Review and Board pages, which already fill the window), context menus and dropdowns (they hang off what opened them), tooltips, and the small popups inside a page (the remotes form, the split-commit editor).

The pointer shows a move shape over the title bar and a resize shape over the borders and corners (terminals with `OSC 22`).

## Mouse pointer shape

Some terminals let GitPoulpi change the mouse pointer: a resize pointer over
dividers, over a column boundary of the Log, the Radar and the commit column
(also while dragging it), a hand over clickable items. This works in kitty,
foot, Ghostty, mintty and a WezTerm build with upstream PR #8092 (the sequence
`OSC 22`). It is sent only when the shape changes, inside tmux only wrapped for
the passthrough (`allow-passthrough on`; kitty and foot show it, Ghostty does
not), and never in screen, zellij or herdr. The default pointer is restored
when GitPoulpi exits. In other terminals (Rio, Alacritty, Tabby, iTerm2,
Terminal.app, Windows Terminal, xterm), the pointer does not change; see
[Choosing a terminal](terminals.md#mouse-pointer-shape-osc-22). The divider or the button
under the pointer lights up in every terminal.

The same kind of component gives the same pointer everywhere in the app: the buttons of a review shown over the Radar (tabs, the close cross, popup buttons) and the Radar's own Actions buttons show the hand like any other button.

- **Buttons are real buttons**, in Settings (Integrations: Edit, Disconnect,
  Remove, Browse, Delete, the snippet buttons of the instructions editor...)
  as in dialogs: padded labels on a filled background that brightens under the
  pointer, dimmed when disabled, never text in `[ ]`. The only brackets left
  are the checkbox marks `[x]` / `[ ]`, which are checkboxes and take the hand
  like any other control.
- **The pointing hand** is on controls: buttons, checkboxes and toggles,
  comboboxes (in Settings, the issues and pull request panes, the Radar,
  dialogs), tabs and their crosses, the `+` buttons, links and references
  (`#12`, `!45`), group and folder headers with their expand arrow, ref pills,
  scrollbars, and the status bar's Activity, jobs and version buttons. A
  checkbox or button inside a row keeps the hand.
- **The arrow** is on rows. A row lights as a whole when the pointer is on it
  (the same light in every list) and is not a control by itself: the rows of
  the Refs, Worktrees, Stashes and Submodules panes, the Log and the commit
  column (only a ref pill takes the hand), the gutters of a diff, the changed
  files, the pull request, issue and Radar items, and the rows of the lists in
  dialogs and panels (clone, folder picker, Workspaces, the board's cards, the
  Jobs panel, assignee, label and milestone pickers, rebase steps, review files
  and commits, menu and combobox choices).
- **The text cursor (I-beam)** is on input boxes, over the whole box: the search
  boxes (global search, Settings, the command palette, Home), the filters (Refs,
  files, Radar, Activity Log, the issues pane search), the text, number and
  area rows of Settings (also while you edit them), the fields of dialogs, the
  review comment boxes, the commit composer, the file editor's text and the merge
  editor's editable result (its read-only panes keep the arrow, since a drag
  there selects nothing). One rule covers read-only text too: wherever a mouse
  drag can select text, the pointer is the I-beam. That is the code of a diff
  (the Diff view, Inline and Split, and the review's diff pane), the File View
  and Blame's code. Their line numbers, markers and hunk headers keep the
  arrow, and a button on a line keeps the hand. The same goes for the
  conversation of an issue or a request (also the review's Overview and its
  draft block), the commit details' message, the Release notes, the Activity Log
  (both tabs), the help popup, the Markdown preview of the comment editor and the
  comments, pending notes and suggestions inside the review's diff: a drag selects
  the text (a plain click keeps its meaning: a link opens, a button acts, an Activity
  Log entry or a thread opens or folds), `Ctrl+C` copies it, `Esc` or a click drops
  it. A selection follows its text when you scroll. The commit message is also
  editable (double-click amends it), and the whole block lights lightly.
- **In the commit details**, the info lines are controls: click `commit: <id>`
  to copy the short id, double-click the author or the date line to amend it
  (the hand and a light mark them). The stats line only informs and keeps the
  arrow.

The resize pointers stay over dividers and column boundaries, and the open hand
shows only while you carry a ref.

## Copying to the clipboard

Copy actions put the text on the system clipboard of the machine GitPoulpi runs
on and, where the terminal is known to honor it, also send it to the terminal
(OSC 52). Over SSH, or with no display, only OSC 52 is sent: if your terminal
does not support it (iTerm2 until its clipboard permission is on, Terminal.app,
GNU screen), nothing is copied and the status line says the text was only sent
to the terminal. See [Choosing a terminal](terminals.md#clipboard-copy-actions).
