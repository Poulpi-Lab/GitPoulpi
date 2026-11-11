**English** · [Čeština](cs/undo-redo.md) · [Deutsch](de/undo-redo.md) · [Español](es/undo-redo.md) · [Français](fr/undo-redo.md) · [Italiano](it/undo-redo.md) · [日本語](ja/undo-redo.md) · [한국어](ko/undo-redo.md) · [Nederlands](nl/undo-redo.md) · [Polski](pl/undo-redo.md) · [Português (Brasil)](pt-BR/undo-redo.md) · [Português (Portugal)](pt-PT/undo-redo.md) · [Русский](ru/undo-redo.md) · [Svenska](sv/undo-redo.md) · [Türkçe](tr/undo-redo.md) · [Українська](uk/undo-redo.md) · [简体中文](zh-CN/undo-redo.md)

# Undo and redo

**You can always go back.** Every change GitPoulpi makes to your repository is
recorded (up to 100 actions by default, `general.undo_depth`), and so is every
position a branch has had, including moves made by other programs. If you make
a mistake, one of these brings the work back:

| You want to | Use |
|---|---|
| take back the last thing | `Ctrl+Z` (redo: `Ctrl+Y` or `Ctrl+Shift+Z`), or the toolbar **Undo** and **Redo** buttons |
| take back several things at once, or one thing from long ago | the **Undo history** (`Alt+Z`, the arrow beside Undo, `Edit: Undo History...`) |
| get back a branch, tag, stash, file or hunk you removed several actions ago, **without undoing what you did since** | **Restore only this** in the Undo history |
| move a branch back to where it was, even if another program moved it | the **Reflog** of the branch (ref menu `Reflog...`) |
| find commits no branch reaches any more | **Lost work** (`Edit: Recover Lost Commits...`) |
| take back what a shell or an **AI agent** did to branches, tags, stashes or HEAD | the same `Ctrl+Z`: those changes are in the history too, marked **outside** (see [Changes made outside GitPoulpi](#changes-made-outside-gitpoulpi)) |

After every action that destroys something and can be undone (discard, delete a
file, delete a branch or tag, hard reset, drop a stash, remove a remote), the
status line says so: "discard done - Ctrl+Z to undo", with the key as you have
it bound. The Undo tooltip says how many actions you can go back.

`Ctrl+Z` takes back the last thing GitPoulpi did to your repository. `Ctrl+Y`
(or `Ctrl+Shift+Z`) puts it back. The toolbar has **Undo** and **Redo** buttons,
the palette has `Edit: Undo` and `Edit: Redo`, and the tooltip of each
button says what it would take back ("Undo Commit \"fix typo\" (a81b9e4)").

![The toolbar after Stage all changes: Undo is lit and its tooltip names the operation it would take back](images/undo-toolbar.png)

Redo never runs the operation again. It restores the state that was recorded,
so a redone commit is the very same commit, with the same id and the same
timestamp.

`Ctrl+Z` is inert while a text field has the keyboard (the commit message, the
filter box, the editor, which keeps its own text undo).

## The Undo history

`Alt+Z` (rebindable), the arrow beside the Undo button or `Edit: Undo History...`
opens a panel with every recorded action, newest first: an icon for the kind
of action (commit, branch, tag, stash, reset, merge and friends, discard,
staging, configuration, outside change), what it did in your language, how long
ago, and its state (**restorable**, **blocked** with the reason, or **undone**
and ready to redo). When the entries span several days, a thin heading (Today,
Yesterday, then the date) starts each day. The header counts them ("12 actions
can be undone (up to 100)").

The actions you have undone sit **above** the newest action, under a divider
("undone - Redo brings them back"), dimmed, with the one Redo brings back next
nearest the divider. The selection starts on the newest action, not on them.
The arrow keys move (the headings and the divider are skipped), `1` / `2` / `3`
(or `Ctrl+Tab` / `Ctrl+Shift+Tab`) switch between the three views, `Esc` closes.
`Enter` never runs an action: every button has its own letter, shown in the hint
line under the buttons. For the selected row:

- **Undo up to here** (`u`): undoes that action and every newer one, newest
  first, after one confirmation that lists them all. A step the daemon finds
  risky asks again before it runs.
- **Restore only this** (`r`): brings back what that one action removed and
  keeps everything done after it. A deleted branch or tag comes back at the
  commit it had (if its name is taken now you are asked for another, with a
  free one filled in); a dropped stash is put back on top of the stash list; a
  discarded or deleted file is written back in place when it is untouched since,
  otherwise next to it as `name.restored` (nothing is ever overwritten); a
  removed remote, upstream, ignore line or attributes line is added again. A
  reset, amend, rebase, merge or commit gets a branch `restore/<branch>-<sha>`
  at the old tip - your current branch and files never move. The confirmation
  shows exactly what it will do, and the restore is a new action: `Ctrl+Z`
  takes it back.
- **Redo up to here** (`y`) on an undone row.
- **Restore main to where Commit left it** (`o`, or `Alt+Shift+Z` /
  `Edit: Restore the Branch to Where the Blocked Action Left It` from outside
  the panel): when another program moved the branch and blocked Undo, the
  reflog still knows where the action left it.

Right-click a row (or press `.`) for the same actions as a menu.

![The Undo history after deleting a branch and making two commits: the deleted branch's row is selected and can be restored on its own](images/undo-history.png)

## The Reflog

The ref menu of a local branch has **Reflog...**; `Edit: Reflog...` shows HEAD's,
and the Undo history has a tab for it. It lists the positions the branch has had,
newest first: short SHA, the reason recorded, how long ago, who - including
moves made in a shell. The selected position's message and changed files are
shown beside the list.

The reflog is read 200 positions at a time. A longer one keeps going on its own:
moving the cursor (or scrolling) near the end of the list fetches the next page,
and the last row reads **... Load more** while there is one. **Load more** (`m`,
or `Reflog: Load More` in the palette) fetches it at once and says "positions loaded: N" on the status line. Repositories using git's reftable format are read like any
other.

![The Reflog of main: every position it has had, newest first, with the selected one described on the right](images/reflog.png)

**Show in Log** (`l`), **Create branch here** (`b`), **Create tag here** (`t`),
**Apply to working tree** (`w`), **Save as stash** (`s`) (the last three work as
in Lost work below) and
**Restore `<branch>` to this position** (`r`). The restore is undoable. When the
branch is checked out and you have local changes it cannot move without
overwriting, a dropdown asks what to do with them: keep my changes, soft, mixed,
or hard. Hard discards the changes in your files but saves them first, so
`Ctrl+Z` still brings them back (it says so in red).

## Lost work

`Edit: Recover Lost Commits...` (also in the Log's commit menu) finds commits
that no branch, tag or stash reaches: those a reflog still names (a deleted
branch's commits), those only an undo pin keeps, and dangling ones. The list
appears at once and the slower scan for dangling commits fills in while a
spinner shows; it never blocks the interface.

![Lost work: the two commits of a deleted branch (found through the reflog) and dangling commits](images/lost-work.png)

**Create branch here** (`b`)
recovers a commit under a name you choose, **Create tag here** (`t`) pins it
with a tag (default name `restored-<short id>`), **Cherry-pick** (`c`) applies
it to the current branch. A lost commit is not in the Log, so this view has no
"Show in Log" (the Reflog view keeps it for the commits still reachable).

**Restore changes** puts what the commit changed back without committing it:
**Apply to working tree** (`w`) replays the commit's changes onto the files,
all or nothing (the index and HEAD stay; a merge commit is refused). When it
cannot apply cleanly, nothing is touched and the dialog offers **Save as stash**
(`s`), which keeps the same changes as a new stash instead; that action is also
always available on its own. Both are journaled: `Ctrl+Z` puts the files back,
or drops the stash again. The same actions are `gitpoulpi apply-commit <rev>`
and `gitpoulpi stash from-commit <rev> [-m M]`, and the Reflog view has them
too.

### Panes, hunks and the window

Reflog and Lost work show the commit list on the left and, on the right, the
selected commit's header, **every file it changed** and the selected file's
hunks (hunks only, wrapped to the pane). The file list is the one of the main
window's right pane: `Path` or `Tree` (`v`, or click the toggle), a folder
folds with `Enter`, `Left` or `Right`, a click or the arrows pick a file and
its hunks appear below (the daemon is asked off the interface thread; the last
few files are kept). The commit list, the file list and the hunks each have a
scrollbar you can press and drag.

- `Tab` / `Shift+Tab` walk the panes (list, files, hunks); the arrows, `PageUp`,
  `PageDown`, `Home` and `End` act on the pane that has the keyboard (its
  border is lit). `Enter` on a file moves to its hunks.
- **Dividers**: drag the one between the list and the right pane, or the one
  between the files and the hunks; double-click a divider to reset it. The
  keyboard: `Alt+Left` / `Alt+Right`, `Alt+Up` / `Alt+Down`, `Alt+0` resets both.
  The palette has `Reflog / Lost Work: Reset Pane Sizes`, `Next Pane` and
  `Path or Tree File List`.
- `Ctrl+N` shows its badges over the three view tabs and the three panes, like
  in the main window; a badge key jumps there.
- **Move and resize the dialog**: drag the title bar to move it, drag the left,
  right and bottom borders or a bottom corner (the `◢` grip) to resize it (never
  below 60 x 16, always on screen). The close cross keeps its click and letting
  go outside the dialog never closes it. The keyboard twin is `Ctrl+N` then `m`:
  the arrows move, `Shift`+arrows resize, `0` resets, `Enter` or `Esc` leave (the
  border says so while it is on). The palette has `Dialog: Move/resize` and
  `Dialog: Reset position and size`. Whether the place is kept is
  `ui.dialog_geometry` (Settings > Appearance > Layout): `session` (default),
  `saved` (restored at the next launch) or `never`.

## From the command palette

Every row action of the three views is also a command palette row, so the
panel needs no mouse and no letter to remember. Press `Ctrl+P` while the panel
is open: the palette opens over it and its **Undo History:**, **Reflog:** and
**Lost Work:** rows act on the selected row of the view that is on screen.

| View | Palette rows |
|---|---|
| Undo history | `Undo History: Undo Up to Here...`, `Redo Up to Here...`, `Restore Only This...`, `Restore the Branch to Where It Was...`, `Refresh`, `Close` |
| Reflog | `Reflog: Show in Log`, `Create Branch Here...`, `Create Tag Here...`, `Apply Changes to Working Tree`, `Save Changes as Stash`, `Restore to This Position...`, `Load More`, `Refresh` |
| Lost work | `Lost Work: Create Branch Here...`, `Create Tag Here...`, `Cherry-pick`, `Apply Changes to Working Tree`, `Save Changes as Stash`, `Scan Again` |

The rows are grayed out (with the reason) while their view is not open; the three
openers, `Edit: Undo History...`, `Edit: Reflog...` and `Edit: Recover Lost
Commits...`, also switch an open panel to their view. A row the selection
cannot do says why in the status line instead of doing nothing: "this action is
already undone: use Redo up to here", the reason an entry cannot be restored on
its own, "only a branch or HEAD can be moved back", and so on. The `...` rows
open the same confirmation or name prompt as the button.

## What can be undone

| Area | Operations |
|---|---|
| Commits | commit, amend, reword, the AI commit composer (all its commits as one step) |
| Branches and tags | create, delete, rename, set or unset the upstream, create a tracking branch, create / delete / annotate a tag, fast-forward, check a pull request out as a branch |
| Moving around | checkout of a branch or a commit (including a detached HEAD) |
| History | merge (fast-forward or merge commit), rebase (plain and interactive, once it has finished), cherry-pick, revert, reset (soft, mixed and hard), the local part of a pull |
| Working changes | stage and unstage (file, folder, hunk, lines, all), discard (file, folder, hunk, lines, a file checkout), apply a patch or a commit's changes (`apply-commit`), **delete a file from disk**, **ignore** (file, extension or folder: the line in the `.gitignore` you chose), **treat as binary or text** (the `.gitattributes` line) |
| Stashes | push, apply, pop, drop, save a commit's changes as a stash (`stash from-commit`) |
| Remotes and config | add, remove, rename, change the URL or push URL of a remote, Git LFS track and untrack |
| On a server | **deleting a branch or a tag on a remote** (from the ref's menu, the palette, `branch delete-remote`, `tag delete-remote`, `tag delete --both`, or a workflow finish that deletes the remote branch): see below |

While a merge, cherry-pick or revert is stopped on conflicts, **Save & Mark
Resolved**, staging and unstaging are undoable too (each one is taken back
before the one under it), and **Abort merge** (or cherry-pick, revert) is
undoable: undoing it brings back the whole stopped operation, with its
conflicts (the three versions of each unresolved file), the files already
resolved and staged, the conflict markers in the working tree and the merge
message; redo aborts again. Staging and resolutions stop being undoable once
the operation ends (committed or aborted). *Unstage* and *Unstage All* never
touch a file that is still conflicted: it has no staged change to take back.

Everything a discard or a hard reset throws away is kept first, so undoing it
brings back the exact bytes, untracked files, executable bits and symbolic
links included. Undoing a dropped stash gives back the same stash, with the same
commit id (it lands on top of the stash list, as `stash@{0}`, not at its old
position). Undoing the removal of a remote gives back its remote-tracking
branches, its `origin/HEAD` alias and the upstream settings of the branches that
tracked it.

### Deleting a branch or tag on a remote

Before the deletion GitPoulpi asks the server what the branch or tag holds
right now and records it: the remote, the name and the exact commit (for an
annotated tag, the tag object itself, so its annotation is kept). The deletion
is then made only if the server still holds that value.

**Undo pushes it back** under the same name, at the same commit, without ever
forcing. Because that is a change on a server other people see, Undo **always
asks first** and says so ("Undoing this re-creates branch topic on origin at
3f2a1c0..."); the entry's steps say it too (`gitpoulpi undo --dry-run`). It uses
the same credentials as a push. The remote-tracking branch comes back with it.
**Redo** deletes it again, with the same safety.

Undo refuses, changing nothing, when:

- the branch or tag was **created again on the remote** since, or moved: it is
  at another commit than the one recorded, and pushing would overwrite someone
  else's work (the refusal names both commits);
- the commit (or tag object) is **no longer in your repository** (it was
  garbage collected): the undo cannot be done.

"Delete locally and on the remote" is **one entry**: Undo restores the local and
the remote branch or tag together, Redo deletes both again. Deleting only the
remote branch leaves the local one alone; undoing it does the same. A workflow
finish that also deletes the remote branch is undoable as a whole.

## What cannot be undone, and why

- **Most things on a server**: push, pushing a tag, LFS push, lock and unlock,
  opening a pull request. That state is shared with other people; undoing a
  push would be a force-push or a delete on their side, and GitPoulpi will not
  do that for you. (Deleting a remote branch or tag is the exception: the undo
  only puts back what the server held, and refuses if anything changed there.)
  Undo never blocks because of such an operation either.
- **Fetch**: it only moves remote-tracking refs, which mirror the server. The
  next fetch would do it again.
- **Aborting a rebase**: undo is not offered for it (a rebase moves branches
  and keeps its own state; the abort is itself the way back). Aborting a
  **merge, cherry-pick or revert** *is* recorded, see above.
- **Creating a worktree**, **creating a patch file**, clone, init, hosting
  actions (pull requests, issues) and AI calls: they are not part of the
  repository's history.
- **Creating a new empty file** from the file menu, the automatic removal of
  `*.orig` files after a merge, and saving a file in the built-in editor are
  not recorded. Neither is changing the **SSH command** (`core.sshCommand`), nor
  changing the **sparse checkout** (nothing is lost; *Disable* brings every
  file back).
- **Writing to `.git/info/exclude` or `.git/info/attributes`** (the "this clone only" choice of the ignore and treat-as lists): those files are outside the working tree, so they are not recorded.
- **Edits to files in the working tree made outside GitPoulpi** (an editor, a
  script, an AI agent writing code) are never captured: no snapshot of them
  exists, so a `git reset --hard` or `git checkout -- file` done in a shell
  loses uncommitted edits for good. What *is* captured of a shell's work is
  what lives in refs: see the next section.

## Changes made outside GitPoulpi

When you work with an AI agent (or a shell next to GitPoulpi), most of the git
work happens outside the window: commits, new branches and worktrees, deleted
branches, resets, rebases, stashes. GitPoulpi sees these too. The daemon keeps
the last refs it saw (local branches, tags, HEAD, the stash list, the worktree
list) and, whenever the repository changed and it was not GitPoulpi's own doing,
records **one entry per coherent change** in the Undo history, marked
**outside**, with the reason git logged (`commit: fix login`, `rebase
(finish)`, `reset: moving to HEAD~1`, `checkout: moving from a to b`) and the
name git recorded for whoever did it. GitPoulpi cannot tell a shell from an AI
agent and never guesses; the entry says only that it was made outside.

- **A burst is one entry.** Twenty commits made between two looks are one
  entry from the first value to the last; a rebase is one entry, recorded when
  it ends (nothing is recorded while a rebase or merge is stopped).
- **Deleted refs are pinned.** A branch, tag or stash that vanished is kept
  alive under `refs/gitpoulpi/undo/` at its old commit before anything can
  prune it, so **Restore only this** (or `Ctrl+Z`) brings back the branch the
  agent deleted, even after the reflog and a garbage collection.
- **While GitPoulpi was closed.** The snapshot is saved next to the journal
  (`.git/gitpoulpi/refs-snapshot.json`), so what happened while no GitPoulpi
  ran is found at the next start and marked "while GitPoulpi was closed".
- **`Ctrl+Z` works on them like on any entry**: the refs move back, a deleted
  branch is recreated, a dropped stash is stored again, an outside checkout
  checks the old branch out again. A checked-out branch moves back like `git
  reset --keep`: files follow, and when one of your local changes is in the
  way it **refuses and changes nothing**. Redo re-applies it. **Restore only
  this** works too: a deleted branch comes back and the later actions stay; a
  moved branch comes back as `restore/<branch>-<sha>`.
- **The status line says so once**: "outside change recorded - Ctrl+Z to
  undo" when a new outside entry arrives and the window is in front.
- Because the outside change is now the newest entry, the old "blocked by an
  outside change" refusal mostly disappears: you undo the outside change first.

What is not recorded: fetch-only moves of remote-tracking branches (they mirror
the server), GitPoulpi's own hidden refs, a worktree that was added or removed
without any branch change (the entry of the branch it carries says "worktree
added <path>"; undoing moves refs only, it does not remove the worktree), and
anything inside files. `general.journal_outside = false` (Settings > General,
"Record changes made outside GitPoulpi") turns this off: nothing outside is
recorded and a change made outside blocks the newest entry, as before.

## When undo refuses

Before acting, GitPoulpi checks that everything the operation touched is still
exactly as the operation left it. If not, it **changes nothing** and says what
moved:

- `main` is at 3f2a1c0, not at a81b9e4 where Commit left it (a commit made in a
  shell, say);
- a file, the index, a stash or a config section was changed since;
- a merge, rebase or cherry-pick is in progress;
- the operation never completed because GitPoulpi was stopped while it ran (the
  entry is marked *interrupted*, is never undone automatically, and `gitpoulpi
  undo --list` shows what it was about to change so you can fix it by hand).

Undo is strictly last-in, first-out: a blocked newest entry blocks the older
ones too, because undoing an older operation underneath a newer one that is
still in effect would be guessing. If you put things back yourself, the entry
becomes valid again. Changes that touch nothing an entry touched (a shell
commit on another branch, a fetch) do not block anything.

### A blocked entry you cannot clear: Forget

When the newest entry is stuck for good (the repository moved on outside
GitPoulpi, or the entry never finished), nothing below it can be undone either.
The Undo tooltip and the refusal then say so and name the way out:
**Edit: Forget Blocked Undo Entry** in the palette (it asks first), or
`gitpoulpi undo-forget` on the command line. Forgetting only removes that one
entry from the journal; the repository is never touched, and the entry can no
longer be undone or redone. The entry under it then becomes the newest.

Two cases settle by themselves:

- a conflicted merge or rebase that you finish with plain git: the entry is
  closed the next time GitPoulpi reads the repository state;
- the files of a conflicted stash pop or apply that you resolve with **Save &
  Mark Resolved**: Undo puts the conflict back while the resolved file is still
  staged and not committed.

## Confirmations

Undo asks only when it is risky:

- the commit (or merge, rebase, cherry-pick, revert) you are taking back is
  already on the branch's upstream: undoing it rewrites history you published,
  and you would have to force-push the result;
- you have uncommitted changes and the undo moves the working tree: your
  changes are kept, but the undo stops, changing nothing, if one of them is in a
  file it has to rewrite.

- the entry deleted a branch or tag on a remote: undoing it **pushes it back to
  the server** (see above), so it always asks, and so does the redo that deletes
  it there again.

Redo, and every undo of a ref-only move, never asks: the other direction is one
key away.

## How far back

`general.undo_depth` (Settings > General) is the number of operations kept.
The default is **100**; `0` means unlimited. Lowering it forgets the oldest
entries at once. Starting a new operation after an undo discards what you could
have redone, as in any editor.

The history is kept per worktree (a linked worktree has its own), in the git
directory (`.git/gitpoulpi/journal.json`), so it survives closing the program
and is never part of `git status` or a commit.

## Turning it off

`general.undo_enabled = false` (Settings > General, "Undo enabled") turns the
whole feature off (and with it the recording of outside changes). Nothing is recorded or kept, so staging,
discarding and committing are a little faster. The Undo and Redo buttons
disappear and `Ctrl+Z` says "Undo is turned off".

**Turning it off erases the history**: the journal file and the hidden refs
described below are deleted, the first time any request reaches the daemon
while it is off. Turning it back on starts from an empty history.

## What is stored

To make sure an undo can never lose data, every commit, tag object, stash or
snapshot an entry might need is kept alive by a hidden ref under
`refs/gitpoulpi/undo/` for as long as the entry is in the history. They are
excluded from GitPoulpi's own listings, but a shell `git log --all` or
`git for-each-ref` shows them, and `git push --mirror` would push them (use
`git push --all` instead). They are deleted when their entry falls off the
history or when undo is turned off. Nothing else is ever deleted.

## From the command line and MCP

```
gitpoulpi undo [--dry-run] [--expect <id>] [--yes]
gitpoulpi redo [--dry-run] [--expect <id>] [--yes]
gitpoulpi undo --list [--all]
gitpoulpi undo-forget [--dry-run] [--redo] [--expect <id>] [--yes]
```

- `--dry-run` describes the steps and changes nothing.
- `--list` prints the journal, newest last (`--all` adds entries that are no
  longer redoable but still kept), with each entry's id and state.
- `--expect <id>` makes the call refuse as *stale* unless that entry is still
  the newest: a script (or a second window) never undoes something other than
  what it looked at.
- `undo-forget` drops the blocked newest entry from the journal (`--redo`: the
  newest redo entry). It refuses when that entry is not blocked, and exits **3**
  without `--yes`. It is a mutation for MCP, but not a destructive one: the
  repository is untouched.
- Exit code **0** done, **2** refused (the JSON names the reason, for example
  `RemoteRefMoved` when a deleted remote branch was created again), **3** risky
  and needs `--yes` (undoing the deletion of a remote branch or tag, which
  pushes to the server).

Going back without undoing everything after it:

```
gitpoulpi undo-restore <id> [--name N] [--dry-run]      (also: undo restore <id>)
gitpoulpi reflog [<ref>] [--skip N] [--limit N]
gitpoulpi reflog-restore <ref> <oid> [--mode keep|soft|mixed|hard] [--dry-run]   (also: reflog restore ...)
gitpoulpi lost-commits [--skip N] [--limit N] [--no-deep] [--budget-ms MS]
```

- `undo --list` shows, for every entry, what **restore only this** could bring
  back (`restore.items`) or why nothing (`restore.not_restorable`), and who made
  it: `"source": "gitpoulpi"`, or `"source": "outside"` with `outside.message`
  (git's reason) and `outside.committer` for a change made by a shell or an AI
  agent. `undo`, `redo` and `undo-restore` take outside entries like any other.
- `undo-restore <id>` brings back what that one entry removed (a deleted branch
  or tag, a dropped stash, a discarded or deleted file, a removed remote with
  its tracking branches and upstreams, an ignore line) and keeps every later
  operation. It never overwrites: a file that changed since gets the kept
  version next to it as `<name>.restored`; a taken branch name is a refusal
  (exit 2) that suggests a free one, give it with `--name`; after a reset,
  amend or rebase it creates a new branch at the old tip and never moves your
  branch. The restore is one journal entry, so `gitpoulpi undo` takes it back.
- `reflog` lists where `HEAD` or a branch was, with git's reason, including
  what a shell did. `reflog-restore` moves a branch (or `HEAD`) to one of those
  positions; a checked-out branch with uncommitted changes needs `--mode`.
  It is journaled too.
- `lost-commits` lists commits no branch, tag, remote or stash reaches. To
  get the work of one back without committing it: `apply-commit <rev>` (to the
  working tree, all or nothing) or `stash from-commit <rev>` (as a stash); both
  are journaled.
- When a shell move blocks the newest entry, the refusal carries the reflog
  position that holds what the entry expects (`reason.RefMoved.restore`), to
  be passed to `reflog-restore`.

Two commands exist so that the actions above are undoable from every surface:
`gitpoulpi rm-file <paths>` (delete files from the working tree) and
`gitpoulpi ignore <pattern> [--file <path/.gitignore>]` (append a line to a
`.gitignore`; nothing happens when the line is already there; undo puts the file
back as it was, or removes it if the command created it, and refuses if the
file was edited since).

The MCP server has one tool per command, so `undo`, `redo`, `rm-file` and
`ignore` are tools too. `undo`, `redo` and `rm-file` are classed as destructive
(gated like `reset`, see [MCP and agents](mcp-and-agents.md)); `ignore` is an
ordinary mutation. The journal is a file in the repository, so `gitpoulpi
undo` in a shell takes back what the window did, and the window can undo what a
script did.
