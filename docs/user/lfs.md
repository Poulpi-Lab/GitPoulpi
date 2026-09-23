**English** · [Čeština](cs/lfs.md) · [Deutsch](de/lfs.md) · [Español](es/lfs.md) · [Français](fr/lfs.md) · [Italiano](it/lfs.md) · [日本語](ja/lfs.md) · [한국어](ko/lfs.md) · [Nederlands](nl/lfs.md) · [Polski](pl/lfs.md) · [Português (Brasil)](pt-BR/lfs.md) · [Português (Portugal)](pt-PT/lfs.md) · [Русский](ru/lfs.md) · [Svenska](sv/lfs.md) · [Türkçe](tr/lfs.md) · [Українська](uk/lfs.md) · [简体中文](zh-CN/lfs.md)

# Git LFS

Git LFS ("Large File Storage") keeps big files (design files, videos, datasets) out of a repository's history: git stores a three-line *pointer* text instead, and the real content lives on the LFS server and in a local cache under `.git/lfs`.

GitPoulpi needs no account for any of this. LFS uses the credentials your remote already uses (your git credential helper, or the ssh-agent).

## What needs the `git-lfs` program, and what does not

GitPoulpi runs the standard `git-lfs` program for everything that converts or moves files (install, track, fetch, pull, push, locks).

Reading LFS *state* needs no program: whether a repository uses LFS, which files are pointers, how big they are, and whether the real object is already stored locally.

| | needs `git-lfs` |
|---|---|
| `gitpoulpi lfs status` (reports the program as missing rather than failing) | no |
| `gitpoulpi lfs files` | no |
| install / track / untrack / prune | yes |
| fetch / pull / push | yes |
| locks | yes |

## Installing `git-lfs`

- **Windows**: it ships with Git for Windows; or `winget install GitHub.GitLFS`.
- **Debian / Ubuntu**: `apt install git-lfs`.
- **macOS**: `brew install git-lfs`.
- Anything else: https://git-lfs.com

Version 3.0 or newer is required. After installing, GitPoulpi notices within a minute (or immediately on `lfs status`).

## When `git-lfs` is missing

GitPoulpi never crashes or exits over it:

- `gitpoulpi lfs status` succeeds and says `git-lfs: not installed`, with the install link.
- An operation that needs the program fails with a clear message that ends with the install link (`https://git-lfs.com`) and a non-zero exit status; over MCP the tool returns that message.
- A repository that uses LFS still opens; its large files simply show as pointers until the program is installed.

## Looking at LFS files

```
gitpoulpi lfs status            # program, patterns, filters, hook
gitpoulpi lfs files             # pointer files in the index, with size and local/not downloaded
gitpoulpi lfs files --at v1.0   # ... as of a revision
gitpoulpi show big.bin --at HEAD  # reports "Git LFS object, 12.3 MB, sha256:..." instead of pointer text
```

## Setting a repository up: install, track, untrack

```
gitpoulpi lfs install                 # filters + hooks for THIS repository (git lfs install --local)
gitpoulpi lfs install --user          # into your global git config instead (never done silently)
gitpoulpi lfs track '*.psd'           # route *.psd through LFS
gitpoulpi lfs track '*.psd' --lockable   # ... and make them read-only until locked
gitpoulpi lfs track '*.bin' --renormalize  # ... and re-stage already-committed *.bin as pointers
gitpoulpi lfs untrack '*.psd'
gitpoulpi lfs prune                   # delete old, already-pushed objects from the local store
```

- `install` writes into the repository's own `.git/config` and hooks by default, so your global git configuration is never touched unless you pass `--user`.
- `track` and `untrack` edit `.gitattributes` and leave it **unstaged**: it shows as an ordinary working-tree change that you commit yourself. Files added *after* tracking are stored as pointers by their next commit.
- Files that are **already committed** as plain content stay plain in history. `--renormalize` re-stages them so that your next commit replaces them with pointers. To move them in every past commit too, use the history rewrite rule *Migrate files to LFS* (see [History rewrite](history-rewrite.md#moving-files-into-or-out-of-git-lfs)); it is GitPoulpi's own, not `git lfs migrate`.
- Quote the pattern so your shell does not expand it.
- Through MCP these are ordinary mutations (`lfs_install`, `lfs_track`, `lfs_untrack`, `lfs_prune`), subject to the `[mcp]` gating in `config.toml`.

## Moving objects: fetch, pull, push

```
gitpoulpi lfs fetch [--remote origin] [--all]   # download objects into the local store; working tree untouched
gitpoulpi lfs pull  [--remote origin]           # fetch, then replace pointer files with the real content
gitpoulpi lfs push  [--remote origin] [--all]   # upload the current branch's objects (or every ref with --all)
```

- The remote defaults to the repository's default remote, like `fetch` and `push`.
- Credentials are your remote's usual ones (credential helper, ssh-agent). GitPoulpi never asks for an account of its own.
- `pull` installs the LFS filters for this repository if they are missing: without them git-lfs downloads the objects but skips replacing the pointers, and a later checkout would put pointers back.
- MCP treats `lfs push` like `push`: a destructive tool (it changes the remote).
- The window shows the progress of a transfer (files and bytes) on the status line. The CLI waits for the result.

### Checking out the content: `lfs checkout`

```
gitpoulpi lfs checkout [<path>...]   # offline: write real content over the pointer files, from the local store
```

It never downloads (that is `lfs pull`); objects missing from the local store stay pointers. Like `pull`, it installs the repository's LFS filters first when none are configured. Like every LFS operation, it is not undoable; see [Undo and redo](undo-redo.md). Through MCP it is `lfs_checkout` (a write, not destructive).

### The toolbar's LFS button, and pull / push

The toolbar has an **`LFS ▾`** button after **Pop** (hidden when `git.lfs_enabled` is off, and in a repository where LFS is neither initialized nor used by its `.gitattributes`: see [Settings > LFS](settings.md#repo-specific-lfs)). A click opens a dropdown: **Fetch all LFS files** (`lfs fetch --all`), **Check out all LFS files** (`lfs checkout`), **Pull all LFS files** (`lfs pull`), **Push all LFS files** (`lfs push --all`, every ref's objects) and **Prune local LFS store** (`lfs prune`). Each row is also a palette command (`LFS: Fetch All Objects`, `LFS: Checkout All Files`, `LFS: Pull Objects`, `LFS: Push All Objects`, `LFS: Prune Local Objects`). `LFS: Open Menu` opens the dropdown. Each one has a `Ctrl+G` chord: `l` for the menu, then `Shift+F`, `Shift+K`, `Shift+L`, `Shift+J` and `Shift+V` for the rows. You can change them in *Settings > Keyboard & mouse*. With `git-lfs` missing, every row leads to the install dialog.

![The LFS dropdown of the toolbar: fetch, checkout, pull and push all LFS files, and prune the local store](images/lfs-menu.png)

**Pull and push handle LFS by themselves.** In a repository that uses LFS, with `git-lfs` installed, a plain `pull` first checks that the repository's LFS filters are installed, and a plain `push` checks its `pre-push` hook. When one is missing, GitPoulpi runs `git lfs install --local` (this repository only) before the transfer, and the status line says "installed the LFS filters". Without the hook, a push would upload no LFS objects. Turn it off with `git.lfs_autofix_filters = false` (*Settings > Git & encoding > Install LFS filters before pull and push*). A plain `push` still uploads only the pushed branch's objects, never every ref's; prune is never implicit.

## File locks

Some files cannot be merged (a Photoshop file, a level, a binary asset). Git LFS *locks* let one person say "I am editing this": nobody else can lock it or push changes to it until it is unlocked.

```
gitpoulpi lfs locks  [--remote origin]              # who holds which file (yours are marked)
gitpoulpi lfs lock   assets/hero.psd [--remote origin]
gitpoulpi lfs unlock assets/hero.psd [--remote origin] [--force]
```

- Locking is a feature of the **LFS server** (GitHub, GitLab and Gitea have it). It is reached with the remote's own credentials; GitPoulpi has no account of its own.
- A server without locking makes `lock` fail with a message that says so (the typed `LfsLockingUnsupported`). `locks` on such a server lists nothing, which is what git-lfs itself reports.
- Tracking a pattern with `--lockable` (`lfs track '*.psd' --lockable`) makes those files read-only until you lock them.
- `unlock` of somebody else's lock is refused; `--force` breaks it, which is why MCP treats `lfs_unlock` as a destructive tool. `lfs_locks` is read-only.

## Cloning an LFS repository

`gitpoulpi clone <url>` (and the TUI's Clone dialog) checks the repository out *without* running the LFS filters, then, when the repository uses LFS and `git-lfs` is installed, downloads the objects with `git lfs pull` (installing the filters for that repository if none are configured) and shows it as its own phase, *Downloading LFS objects*. `--no-lfs` skips that step and leaves pointers.

A clone **never fails because of LFS**: with `git-lfs` missing, or a download that fails, the clone still succeeds and the large files stay as pointers; the repository's LFS status says why, and `gitpoulpi lfs pull` finishes the job once the problem is fixed.

## In the terminal UI

- **The `LFS` tag.** A file stored with Git LFS has a small `LFS` tag after its name in the Status pane's Unstaged and Staged lists and in a commit's changed-file list.
- **The diff card.** Opening an LFS file's diff shows a card instead of pointer text: `Git LFS object`, `1.2 MB -> 1.4 MB`, the short old and new oids, and whether the new object is stored locally or `not downloaded`. Press `r` for the raw pointer diff, `r` again to come back. Only file-level staging applies to an LFS file (there are no hunks or lines to stage).
- **The File View** of an LFS file says `Git LFS object, 12.3 MB, sha256:ab12cd34...` and where to get the content.
- **The banner.** For a repository that uses LFS while `git-lfs` is missing or older than 3.0, a banner in the Status pane says so. Click it (or use the palette entry *LFS: Open Install Page*) to open https://git-lfs.com. Nothing else changes: the window opens, diffs work, large files simply show as pointers.
- **The file menu and the palette.** Right-click a changed file in the Status pane: *Track with LFS: this file* / *this extension* for a plain file; *Lock file (LFS)*, *Unlock file (LFS)* or *Force unlock* for an LFS file, according to who holds its lock. A locked file's row says `locked by you` / `locked by <name>`. Everything is also a palette command (`Ctrl+P`, type `LFS`): initialize, track or untrack a pattern, fetch, pull or push objects, prune, show locks, open the install page.
- **A failed action explains itself.** If an LFS action fails because `git-lfs` is missing, a dialog carries the message and an *Open install page* button.
- **Settings.** *Settings > LFS* (per repository) shows whether LFS is initialized, lists the tracked patterns and has **Set up LFS here**. *Git & encoding* (per repository) has the **Git LFS** switch, a read-only **git-lfs program** row and an **Initialize LFS for this repository** button.
- **Git LFS** off (`git.lfs_enabled = false`, see the [configuration reference](config-reference.md)) hides all of this. git's own LFS filters still run.
- The LFS state is read shortly after the repository opens, after every change, on a manual refresh, when `.gitattributes` changes, and every ten seconds for a repository that uses LFS.

## Known limits

- A repository whose LFS patterns live only in a *nested* `.gitattributes` (not the root one, not `.git/info/attributes`) is detected as using LFS once it has an LFS object store (`.git/lfs`), not before.
- `git lfs migrate` itself is not used. Moving files into or out of LFS across history is the history rewrite's *Migrate files to LFS* / *Bring files back from LFS* rules: they work on the local clone, only the root `.gitattributes` is written, and the objects still have to be pushed afterwards (*Send LFS objects* in the result panel, or `lfs push --all`).
