**English** · [Čeština](cs/worktrees.md) · [Deutsch](de/worktrees.md) · [Español](es/worktrees.md) · [Français](fr/worktrees.md) · [Italiano](it/worktrees.md) · [日本語](ja/worktrees.md) · [한국어](ko/worktrees.md) · [Nederlands](nl/worktrees.md) · [Polski](pl/worktrees.md) · [Português (Brasil)](pt-BR/worktrees.md) · [Português (Portugal)](pt-PT/worktrees.md) · [Русский](ru/worktrees.md) · [Svenska](sv/worktrees.md) · [Türkçe](tr/worktrees.md) · [Українська](uk/worktrees.md) · [简体中文](zh-CN/worktrees.md)

# Worktrees

A worktree is a second checked-out copy of the repository: another folder, on
another branch, sharing the same history. The WORKTREES section of the Refs
pane lists them; the main one is marked `(main)`.

## Showing the section

The section is hidden in a repository that has no linked worktree yet. To see
it anyway (it then reads "No worktree yet: + to create one"), use the palette
command **Sidebar: Show Worktrees Section**, tick **WORKTREES** in the
sidebar's show/hide list (right-click a section header), or run any of the
create commands below. GitPoulpi remembers it for that repository (see
**Sidebar sections apply to** in [Settings](settings.md)); another repository
without worktrees still starts with the section hidden.

## Creating a worktree

The **+** on the WORKTREES header (keyboard: `+` in the section; palette:
**Refs: Create Worktree...**) opens a menu:

- **From the current branch**: the branch is checked out in the main folder,
  so the dialog proposes a new branch on it (`<branch>-wt`).
- **From the selected commit**: grayed until a commit is selected in the Log.
  The worktree starts detached there unless you name a new branch.
- **Choose a branch...**: the dialog opens on its **Ref** list.

The palette has the same three (**Refs: Create Worktree from Current Branch**,
**Git: Create Worktree at Selected Commit**, **Refs: Create Worktree Choosing a
Branch**). The older entries still work: **Create worktree from...** in a
branch, tag or commit menu.

The dialog proposes everything and lets you change it:

| Field | Meaning |
|---|---|
| **Name** | The folder name is built from it. Proposed from the branch or the short id. |
| **Folder** | A sibling of the repository, `<repository>-<name>`. It follows the Name until you type in the Folder; **Browse** chooses where. |
| **Ref** | The branch or commit the worktree starts from (a list; type to search). Changing it proposes a new Name. |
| **New branch** | Empty checks the Ref's branch out there (or detaches on a commit). Required when the branch is checked out already. |
| **Open the new worktree as a tab** | On by default. |

A folder that is not empty, an invalid name or a branch that already exists is
reported on the dialog's own line, and **Create** stays grayed.

## The row's menu

Right-click a worktree (or select it and press the menu key, `Shift+F10` or
`.`):

- **Open in a new tab**: like a double click or `Enter`.
- **Open in**: the [Open in](open-in.md) entries whose **Can open a worktree**
  box is ticked (Settings > Open in; every entry is ticked by default). They
  run with the worktree's folder as `${cwd}`.
- **Merge <branch> into...**: pick the branch to merge into (the main
  worktree's branch by default). The ordinary merge runs in the worktree where
  that branch is checked out, with your merge options; a conflict is resolved
  there by the usual conflict flow. A branch checked out nowhere is refused
  with the reason. Not offered for a detached worktree.
- **Remove worktree...**: never the main one.

The palette has the same actions on the selected row (**Refs: Open Worktree
In...**, **Refs: Merge Worktree Branch Into...**, **Refs: Remove Worktree...**,
**Refs: Open Worktree in Tab**).

## Removing a worktree

A clean worktree gets a plain confirmation. If its branch is already merged
(into its upstream, the main worktree's branch or `main` / `master`) the box
**Also delete branch <b>** is ticked by default; an unmerged branch says it
stays. **Merge into... then remove** merges first and removes only once the
merge landed without conflicts.

With uncommitted changes (tracked or untracked) the dialog lists them
(count, scrollable) and offers:

- **Commit and remove**: a message field in the dialog; every change is staged
  (`.gitignore` respected) and committed on the worktree's branch, then the
  worktree is removed. A detached worktree names a branch for the commit in
  the same dialog.
- **Stash and remove**: every change, untracked files included, is stashed
  with an editable message (default "WIP on <folder>"), then the worktree is
  removed. The stash list is shared by all worktrees: the stash is in the main
  repository's STASHES afterwards, ready to pop.
- **Open to commit**: opens the worktree in a tab with the commit message
  focused; nothing is removed.
- **Remove anyway**: discards the changes after a second, explicit
  confirmation.

## Undo

Removing a worktree is not undoable: the folder is gone. What was saved
first stays: the commit is on its branch, the stash in the list. The branch
deletion that **Also delete branch** adds is the ordinary journaled one:
**Undo** brings the branch back (not the folder). Creating a worktree is not
part of the history either.

## From a script or an agent

`gitpoulpi worktree changes <path>` lists the uncommitted changes of a
worktree. `gitpoulpi worktree remove <path> [--force] [--commit-message M
[--commit-branch B]] [--stash-message M] [--delete-branch]` removes it; the
MCP tools are `worktree_changes` and `worktree_remove`. See the
[CLI reference](cli-reference.md).
