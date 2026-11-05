**English** · [Čeština](cs/history-rewrite.md) · [Deutsch](de/history-rewrite.md) · [Español](es/history-rewrite.md) · [Français](fr/history-rewrite.md) · [Italiano](it/history-rewrite.md) · [日本語](ja/history-rewrite.md) · [한국어](ko/history-rewrite.md) · [Nederlands](nl/history-rewrite.md) · [Polski](pl/history-rewrite.md) · [Português (Brasil)](pt-BR/history-rewrite.md) · [Português (Portugal)](pt-PT/history-rewrite.md) · [Русский](ru/history-rewrite.md) · [Svenska](sv/history-rewrite.md) · [Türkçe](tr/history-rewrite.md) · [Українська](uk/history-rewrite.md) · [简体中文](zh-CN/history-rewrite.md)

# History rewrite

Rewriting history changes commits that already exist: who wrote them and when,
their messages, the files they hold, their signatures. GitPoulpi does it with a
**recipe** (a scope and an ordered list of rules) and three steps that never skip
the middle one:

1. **Preview** computes the new history without touching anything: counts, what
   each rule did, the branches and tags that would move, one row per changed
   commit.
2. **Apply** moves your local branches and tags to it. It is one entry of the
   [undo history](undo-redo.md): **Undo** puts every moved ref (and the checked-out
   files) back.
3. **Push** (a force push with a lease) and **Purge** are separate, deliberate
   steps, because they cannot be undone: see [Pushing the result](#pushing-the-result)
   and [Deleting the old history](#deleting-the-old-history-purge).

The workbench, the palette, the `gitpoulpi rewrite` commands and the MCP tools
run the same engine, so a preview made in one is what another applies.
[From the command line](#from-the-command-line) lists the flags;
[Create rules with AI](#create-rules-with-ai) fills the workbench from a description;
[Proposals from AI agents](#proposals-from-ai-agents) says what an agent may and
may not do here.

**The rules, by what they change**

| Group | Rules | Section |
|---|---|---|
| Authors | Replace an author, Anonymise authors, Apply .mailmap | [Authors and the .mailmap](#authors-and-the-mailmap) |
| Dates | Shift dates, Change time zone, Keep within hours, Committer date = author date | [Dates](#dates) |
| Messages | Search and replace, Delete lines, Add or remove a trailer, Fix message encoding | [Messages](#messages) |
| File contents and paths | Search and replace in files, Remove files, Remove large files, Move or rename paths, Convert line endings, Remove the executable bit, Convert file encoding | [Files and paths](#files-and-paths) |
| LFS | Migrate files to LFS, Bring files back from LFS | [Moving files into or out of Git LFS](#moving-files-into-or-out-of-git-lfs) |
| Signatures | Remove signatures (and re-signing on Apply) | [Signatures](#signatures) |
| Script | Lua, shell, Python | [The Script rule](#the-script-rule) |

**Before you start**

- A preview changes nothing. Apply changes only your local refs and never
  pushes.
- Rewritten commits get new ids. Everyone who has the old history must be told:
  [Pushing the result](#pushing-the-result) writes the note.
- A secret that was pushed is leaked. **Rotate it first**, then rewrite,
  force-push and purge.
- Commits that stay as they are keep their ids and their signatures.

## The workbench

`Ctrl+G Shift+H` (or **Rewrite: History Workbench** in the palette) opens the
rewrite workbench over the Log. It has no toolbar button: advanced actions live
in the palette (every `Rewrite: ...` row opens it with that rule already added),
in the commit menu's **Rewrite history** submenu (*From this commit to HEAD...*,
*Selected commits...* when several are selected, *Open the workbench...*) and
behind the chord.

`Ctrl+N` shows a badge on each control of the workbench (scope, rules, fields, tabs, buttons); pressing a badge does what a click on that control does, and `Esc` takes the badges away (see [Badges](keybindings.md#badges)).

![The history rewrite workbench: the scope bar, a list of two Replace an author rules with the fields of the selected rule, the Create Rules with AI button, and the Preview summary saying how many commits change](./images/history-rewrite.png)

- **Scope bar:** which refs move (current branch, all local branches, chosen
  branches and tags, the selected commits), the *Tags* box, and, after *From
  this commit to HEAD...*, a box to start after that commit's parent.
- **Rules:** a numbered list, applied in order. The box switches a rule on or
  off, the arrows move it, the cross deletes it (*Restore the deleted rule* or
  `Ctrl+Z` brings it back). *Add rule* lists the kinds by group. A rule that
  needs a program you do not have stays greyed with the reason.
- **Rule settings:** a click on a text field edits it; a click on a field with
  several values opens its list; boxes flip.
- **Preview** (`F5`) computes the new history without touching anything and shows
  a progress bar. The tabs: **Summary** (counts, what each rule did, warnings),
  **Commits** (one row per changed commit, paged from the daemon; the filter
  lists changed, removed or pushed ones), **Branches and tags** (what moves, and
  the protected branches already pushed, each with a box), **Impact** (what
  rewriting pushed history means: a force push with lease, collaborators,
  signatures), **Secrets**, **Identities** and **Size** (see below).
- **Files of a commit:** `Enter` (or a double click) on a Commits row opens the
  files the content rules changed in it and, beside them, the diff of the
  selected one. The cross, `Esc` or a click outside closes it. Removed lines of a
  secret rule are hidden.
- **Apply** (`Ctrl+S` or `Ctrl+Enter`) is available once the preview matches
  the rules. Questions that have no default are asked each time: what happens to
  the signatures of signed commits, and how a folder is extracted. The result
  panel offers **Undo**, which puts every moved ref back.
- **Save / Load** keep a recipe in the repository (shared) or only for you;
  secret values are never saved.

Every button has a key: the list is in Settings > Keyboard & mouse (context *Rewrite*)
and in the guide `keybindings.md`.

### Authors and the .mailmap

- **Replace an author** matches an author or committer by name and/or email
  (a literal, or a regular expression) and sets a new name and/or email. *Apply
  to* picks the author, the committer or both; *Also the authors of tags* makes
  the annotated tags that are re-created follow. The original committer is kept
  unless the run says *me, now*.
- **Anonymise authors** replaces every identity by a numbered placeholder
  (`Contributor 1`, `contributor-1@anonymous.invalid`; both patterns are yours to
  change, with `{n}` for the number). The same person always gets the same number
  in one run; identities you list as *keep* are left as they are.
- **The Identities tab** (also **Rewrite: Merge Identities (.mailmap)...** in
  the palette) is the way to fix a person who committed under several names and
  emails **without rewriting anything**. It lists every name and email of the
  scope on the scope bar with how many commits each wrote and committed, and
  which ones the repository's `.mailmap` already maps (`mapped to Name <email>`).
  Tick the identities that are the same person (`Space`, or a click; *Check all*
  / *Uncheck all*), then **Merge into...** (`m`): a dialog asks which of them
  stays (the one with most commits is first) and **adds one line per other
  identity to `.mailmap`** (`Name <email> Old Name <old@email>`; a line already
  there is not repeated, the file is created when missing). Nothing is
  committed: commit `.mailmap` yourself to share it. The Log, blame and file
  history show the canonical name at once (setting `log.use_mailmap`, on by
  default; the stored commits never change, so there is nothing to force-push).
  The status line says what was written and offers **Undo** (`u`), which puts
  the previous `.mailmap` back (it is one entry of the undo history, like any
  other action). `r` reads the list again.
- **A `.mailmap` instead of a rewrite.** When an author rule changes commits
  that are already on a remote, the Summary says *A .mailmap fixes the display
  without rewriting. Use .mailmap instead?* with a **Use .mailmap** button
  (`m` on the Summary). It opens the Identities tab with the identities the
  rule matches ticked, and the identity the rule would set offered as the one to
  keep even though no commit carries it yet.
- **The `.mailmap` rule** (*Rewrite: Apply .mailmap to History...*, or
  `rewrite preview --mailmap`) does rewrite: authors and committers (your
  choice) become what the worktree's `.mailmap` names them, the result
  `git log --use-mailmap` shows. Use it when the history itself must carry the
  canonical names, for example before publishing it; it is a rewrite like any
  other (preview, Apply, force push).
- **Identity rules and trailers.** An identity rule can also change the people
  named in trailers: tick **Also map trailers** (`--map-trailers`). Every trailer
  whose value is exactly `Name <email>` and matches the rule's *Email is* /
  *Name is* is set to the new name and email, whatever the rule does to the
  author and committer. It is off by default: a `Signed-off-by` records who
  signed, so changing it is a decision of its own.

### Dates

The four date rules act on the author date, the committer date or both (*Dates*
in the rule's pane).

- **Shift dates** moves every date by an amount (`+2h`, `-1d`, `90m`, `1d 2h`). The time
  zone each commit was written in is kept.
- **Change time zone** writes the dates in another UTC offset. *Keep* says what
  stays: the **moment** (the same instant, the clock reads differently) or the
  **clock** (the same local time, the instant changes).
- **Keep within hours** moves each commit that falls outside the allowed hours
  (for example `Mon-Fri 18:00-23:30; Sat,Sun 10:00-20:00`) to the nearest
  allowed time, or only earlier, or only later (*Move to*). *Never earlier than
  its parent* keeps the order of the history; the *Seed* makes the small random
  spread repeatable: the same seed on the same history gives the same dates.
- **Committer date = author date** sets the committer (name, email and date) to
  the author's, which is what a `git commit --amend` or a rebase has undone.

Dates are rules like the others: preview, check the **Commits** tab (each row
shows the old and the new date), then apply. A commit's id depends on its dates,
so a date rule changes every commit after the first one it touches.

### Messages

- **Search and replace in messages** and **Delete message lines** take a literal
  text or a regular expression (*Regular expression*, *Case sensitive*); an empty
  replacement deletes. A message that is not valid UTF-8 is never touched by
  them: run **Fix message encoding** first if you want it edited.
- **Trailers.** *Add or remove a trailer* (*Rewrite: Add or Remove Trailers...*
  in the palette, or `--trailer-add`, `--trailer-remove`, `--trailer-map`) edits
  the `Key: value` lines at the end of a message, the ones `git
  interpret-trailers` knows (`Signed-off-by`, `Co-authored-by`, `Reviewed-by`...).
  The **Action** dropdown picks *Add* (the **Trailer** key, its **Value**, and
  **If it already exists**: *Add (unless identical)*, *Replace the existing
  ones*, or *Leave the commit alone*), *Remove* (every trailer of that key, or
  only those whose value contains **Value contains**) or *Change the value*
  (**Find in value** becomes **Replace with**, for instance an old email by a
  new one in `Signed-off-by`). Keys are matched without regard to case. Only the
  trailer lines change, the rest of the message stays exactly as it was, and a
  message that has no trailers yet gets a blank line and a block of its own. The
  Commits tab shows each change as `+Signed-off-by: ...` / `-Co-authored-by: ...`.
  The last paragraph of a message is its trailer block (never the title), so a
  line such as `Note: see the ticket` in the body is not a trailer unless it is
  alone in the last paragraph with other trailers.
- **Commit ids in messages.** **Update commit ids** (`--update-message-hashes`,
  off by default; at the end of the panes of the rules that edit messages) makes
  ids written in messages follow the rewrite: an abbreviated id of 7 or more hex
  characters (`This reverts commit 1a2b3c4`, `see 1a2b3c4d5e`) that names exactly
  one commit of the rewritten history is replaced by that commit's new id, cut to
  the same length. One that matches several commits, none, or a commit that the
  rewrite creates later is left as it is; the Summary says how many were
  replaced and how many were left.
- **Fix message encoding** (*Rewrite: Fix Message Encoding...*, or
  `--fix-message-encoding [--fix-name-encoding]`) re-encodes messages to UTF-8,
  two cases:
  - A commit with an `encoding` header (`ISO-8859-1`, `Windows-1252`,
    `Shift_JIS`, anything git and the encoding tables know) is decoded and
    written as UTF-8, and the header goes: the text reads the same everywhere,
    with no conversion. The author and committer names of that commit were in the
    same encoding and are decoded with it, so a commit is never left half
    converted.
  - **Mojibake**, UTF-8 that was once read as Latin-1 or Windows-1252
    (`RÃ©Ã©criture` for `Réécriture`), is repaired **only when the round trip is
    exact**: the text must encode back to bytes that are valid UTF-8, and mangling
    the result the same way must give the original. A message that merely
    contains `Ã©` next to real letters, or characters outside the old alphabets,
    does not pass and stays as it is.

  *Also fix names* (`--fix-name-encoding`) repairs mangled author and committer
  names the same way. What cannot be decided (an unknown encoding name, bytes
  that are not valid in the declared one, a message that is not UTF-8 and
  declares nothing, text that looks mangled but is not provably so) is **left
  alone and counted**: the Summary says *n messages were left as they are: their
  encoding cannot be decided*, and `counts.encoding_left` has the number. The
  Commits tab shows each fix with the old and new message and an *encoding* row
  (`ISO-8859-1` becomes `UTF-8`).

### Files and paths

- **File content rules** (*Search and replace in files*, *Remove files*,
  *Remove large files*): the pane checks each value as you type and shows the
  reason under the field (a pattern that does not compile, a bad path pattern, a
  size below one byte). Sizes take units (`500 KiB`, `5 MiB`, `1 GiB`). A path
  pattern without a slash matches at any depth. *Secret values* are typed in a
  masked field, used for the run only and never saved; the preview hides the
  lines that held them. *Empty commits* says whether a commit the rules left
  empty is removed (default) or kept. Binary files and files over the size limit
  are left as they are and counted in the Summary.
- **Moving paths.** *Move or rename paths* (*Rewrite: Move or Rename Paths...* in
  the palette) has three kinds, picked in the **Move** dropdown:
  - *Rename a path* (**From**, **To**; `--move-path FROM=TO`): the
    same files and folders appear at the new path in every commit that had the
    old one; nothing else changes, every file keeps its id. A commit where the
    new path already exists is never overwritten: the preview is refused and
    lists the commits and the paths that clash (pick a free name, or move the
    existing one first). Only exact paths, no wildcards.
  - *Move everything into a folder* (**Folder**; `--into-subdir DIR`):
    every path of every commit gets the prefix. Useful to merge repositories
    without name clashes.
  - *Extract a folder as the root* (**Folder**; `--extract DIR`): the folder becomes
    the root, commits that do not touch it are dropped (a commit that creates
    it has no parent), merges that no longer join anything are simplified, and
    branches and tags that have nothing of the folder are deleted. You are asked
    **every time** how it ends:
    - *Into a new clone, opened in a new tab* (`--extract-mode clone`): a copy
      of this repository is made in a folder you pick (the new repository
      gets a folder named after the extracted one inside it), rewritten
      there, and cleaned so that it holds only the new history, every local
      branch and tag that has the folder. This repository is not touched. The
      new repository has no remote (the original is not its upstream).
    - *In place: this repository's history is rewritten* (`--extract-mode
      in-place`): the usual rewrite, preview, Apply, undo. Branches without
      the folder are deleted (undo brings them back); the checked-out branch
      must have it.
    - *As a submodule of this repository* (`--extract-mode submodule`): the new
      repository is made as in the clone case, then **one new commit** on the
      current branch replaces the folder by a submodule of it (`.gitmodules`
      and the link), the folder stays as a checked-out submodule. This
      repository's history is **not** rewritten: its old commits keep the
      folder's files, so nothing that points at them (tags, other clones)
      breaks. The URL in `.gitmodules` is the local path of the new
      repository: change it (`git config -f .gitmodules submodule.<folder>.url`)
      once it lives on a host. Edited or untracked files in the folder refuse
      it. Undo puts the plain folder back.
    The Summary says that the preview shows the new repository's history and
    which folder it goes to. Rules that come after an extraction (a rename, a
    removal) apply to the extracted tree; for a clone or a submodule the
    extraction must be the first path rule. Path rules run after the content
    rules, so a removal glob names the paths as they were.

#### Line endings, the executable bit and file encodings

Three rules in the **Content** group of the Add menu (or the palette rows
*Rewrite: Convert Line Endings...*, *Remove the Executable Bit...* and
*Rewrite: Convert File Encoding...*). All three take **Files** (globs; a pattern
without a slash matches at any depth; empty means every file) and **Except
files**. They change only what the globs name, in every commit of the scope.
Binary files (a NUL byte in the first 8 KiB) are never touched: the Summary
counts them as *left as they are*.

- **Convert line endings** (`--eol lf|crlf|cr --eol-paths GLOB...
  [--eol-exclude GLOB...]`). *Convert to*: LF (Linux, macOS), CRLF (Windows) or
  CR (classic Mac). Every line ending of a matching text file becomes that one,
  so a file with mixed endings comes out uniform; files already right keep their
  version untouched. *Follow .gitattributes* (off by default,
  `--eol-follow-attributes`) reads each commit's root `.gitattributes`: files
  marked `-text` or `binary` are skipped and files marked `eol=lf` or
  `eol=crlf` keep that ending instead of the one you chose.
- **Remove the executable bit** (`--strip-exec GLOB... [--strip-exec-exclude
  GLOB...]`). Matching files go from mode 100755 to 100644; their content is
  not touched. At least one file pattern is required: removing every
  executable would break your scripts.
- **Convert file encoding** (`--reencode TARGET [--from SOURCE]
  [--encoding-fallback ENC] --encoding-paths GLOB... [--encoding-exclude
  GLOB...]`). *Convert to*: UTF-8 (the default), UTF-8 with BOM, UTF-16 LE or
  BE (written with a byte order mark), Windows-1252 or ISO-8859-1. *Files are
  in*: by default each file's encoding is detected (a byte order mark, else
  valid UTF-8, else the *Fallback*, Windows-1252 unless you change it); name
  the encoding to force it, but a wrong choice gives wrong text (UTF-8 read as
  Latin-1 doubles every accent). A file the source encoding cannot read, or
  that holds a character the target cannot store (a euro sign in ISO-8859-1),
  stays exactly as it is and is counted in the Summary.

The rules run in the order of the list, on the same file: converting a file to
UTF-16 and then asking for line endings leaves it alone (it is no longer
plain text). Applying is one entry in the undo list like any rewrite.

### Finding secrets

- **Finding secrets:** `gitpoulpi rewrite scan-secrets` (and the MCP tool
  `rewrite_scan_secrets`) reads the history once and lists token prefixes
  (GitHub, GitLab, Slack, AWS, Stripe, npm, Google, OpenAI, Anthropic), private
  keys, JWTs, passwords in URLs and random-looking `password = value` lines,
  each with its file, first and last commit and a redacted preview such as
  `ghp_****abcd`. It is not exhaustive, and the value is never shown, logged or
  saved. `--remove-finding ID` turns a finding into a rule that removes it from
  every commit. A secret that was pushed is leaked: rotate it first, then
  rewrite, force-push and purge.
- **The Secrets tab** does the same from the workbench, for the scope on the
  scope bar: **Scan for secrets** (`s`) runs the scan as a job (the progress bar
  is labelled *Scanning*, **Cancel** or `Esc` stops it). Each finding is a row
  with its kind, the redacted preview, the file and how many others hold it;
  under the list: the first and last commit it was seen in and how many commits
  and file versions hold it. The tab says how many binary and oversized files
  were not read, and that the scan is not exhaustive. All findings start
  ticked; **Remove from history** (`r`) turns the ticked ones into one rule
  that names them by id (never by value), which you preview and apply like any
  other. Findings go stale after a rewrite or a scope change: the tab says so,
  scan again. The workbench never holds the value of a secret, so it cannot
  show it.

### Finding what weighs

- **Finding what weighs:** `gitpoulpi rewrite analyze [--top N]` (and the MCP
  tool `rewrite_analyze`) is the size analysis, like `filter-repo --analyze`:
  the largest files with their path, size, first and last commit and whether
  they are still in HEAD, the biggest folders, the files deleted from HEAD that
  the history still stores, and the totals. It reads sizes and paths only, never
  file content, and changes nothing.
- **The Size tab** does the same from the workbench (`F7`, or *Analyse the
  History Sizes* in the palette, opens it and runs): **Analyse sizes** (`s`) is
  a job (*Analysing*, **Cancel** or `Esc` stops it). **Show** picks the list
  (`v`): *Files* (the largest unique file versions, each marked *in HEAD*, *old
  version* of a file that is still there, or *deleted*), *Folders* (everything
  ever stored below each) or *Deleted* (paths gone from HEAD that the history
  keeps). **Sort** (`o`) is by size, by path or deleted first. Tick rows
  (`Space`, `Alt+A` / `Alt+U` for all / none): the tab says how much they hold,
  honestly as "up to about", because the space comes back only after the
  rewrite, a purge of the old objects and a `gc`. **Remove from history** (`r`)
  turns the ticked paths into a path rule (special characters escaped, a path
  already in a rule is not added twice); **Remove every file over ...** (`b`,
  Files list) writes a size rule just under the smallest ticked file. Preview and
  apply them like any other rule. The lists show the 200 largest rows; the CLI
  says how many more there are and `--top` lists them. Rows go stale after a
  rewrite or a scope change: analyse again.

### Signatures

- **Signatures:** a rewritten commit cannot keep its signature (it signed the
  old bytes), so when signed commits are rewritten you are asked, every time:
  *Remove the signatures* (the count is shown first), *Re-sign only the commits
  that were signed*, or *Re-sign every rewritten commit*. There is no default.
  Commits that stay as they are keep theirs. Re-signing uses your own setup,
  the way a signed commit does: `gpg.format` (`openpgp`, `ssh` or `x509`),
  `user.signingkey` and `gpg.program` / `gpg.ssh.program` / `gpg.x509.program`.
  The fourth row of the question, **Check signing** (`4`), signs a dummy
  payload once and verifies it: it tells you how long one signature takes and
  estimates the run for the number of commits asked about, or why it does not
  work (key missing, agent locked, program not installed, with how to install
  it). The preview does the same test before it reads the history and refuses
  with that reason, so a broken setup never leaves a half-signed rewrite. The
  commits are signed **during the preview** (their ids contain the signature),
  so a preview that re-signs takes the signing time, the Summary says *Commits
  signed again: n, in 12.5 s*, and a passphrase prompt may open then: unlock
  your agent first (Check signing does exactly that). **Cancel** stops between
  two signatures and leaves nothing behind. Only commit signatures are
  re-created; a signed tag still loses its signature.
- **The *Remove signatures* rule** (Add menu, group *Signatures*; *Rewrite:
  Remove Signatures...* in the palette; `--remove-signatures`) strips the
  signature of every commit in scope, even where nothing else changes, and
  asks nothing about re-signing. From the CLI: `rewrite check-signing` and
  `--signatures strip|resign-signed|resign-all`.

### Undoing a rewrite

**Undo** (the result panel's button, the toolbar's *Undo* with `Ctrl+Z` in the Log, or `gitpoulpi undo`) puts every
moved branch, tag and note back, and the checked-out files with them, from the
old commits that the rewrite pins. It asks first when a rewritten branch is
already on its upstream (undoing locally does not undo a push) and when you have
uncommitted changes: those are kept, and a file you changed that the old
history would overwrite stops the undo instead (files move the way
`reset --keep` moves them). Undo stops being
possible for a rewrite once you [purge](#deleting-the-old-history-purge) its old
history; the history list then marks it *purged*.

## Moving files into or out of Git LFS

Two rules in the *LFS* group of the Add menu rewrite the history so large files
live in [Git LFS](lfs.md) from the start (or leave it). They need the
`git-lfs` program: without it the rules are greyed with the install hint and a
preview is refused. They are our own engine, not `git lfs migrate`, so you
get the preview, the scope, the undo and the push guard of any other rule.

- **Migrate files to LFS** (*Rewrite: Migrate Files to LFS...*, or
  `--lfs-import PATTERN` with `--lfs-min-size BYTES`). Every file that matches
  the patterns (`*.psd`, `assets/**`; a pattern without a slash matches at any
  depth) and is at least *Only files of at least* big (empty: every match)
  becomes an LFS pointer in every commit that holds it. Its content goes into
  the local LFS store (`.git/lfs/objects`, once per distinct content) when you
  Apply, and the root `.gitattributes` gets `<pattern> filter=lfs diff=lfs
  merge=lfs -text` in every rewritten commit that holds a pointer (created when
  there is none, the other lines untouched, never duplicated). A file that is
  already a pointer is left alone. The Summary says *3 files (12 MiB) move to
  LFS. LFS objects to write: 3.* and the Commits tab lists *moves big.psd to
  LFS* per commit.
- **Bring files back from LFS** (*Rewrite: Bring Files Back from LFS...*, or
  `--lfs-export PATTERN`). Pointers that match become the real files again, and
  the `.gitattributes` lines for those patterns are removed. The objects must be
  in the local LFS store: when some are not, the preview is refused with the
  list and a **Download LFS objects** button that fetches them from `origin`;
  preview again afterwards. An import followed by an export of the same
  patterns gives back the very same commits.

After Apply, the checked-out files are the real content (not pointers) and
**Undo** puts the old history back. The result panel has **Send LFS objects**
(`l`): the same as `LFS: Push All Objects`, so the new history can be pushed
without a later push stalling on missing objects (a plain push of the rewritten
branches also uploads them through the pre-push hook that Apply installs). The
note for collaborators says that they need git-lfs and `git lfs pull`.

Good to know: Apply installs the repository's LFS filters (`git lfs install
--local`) when they are missing, because git needs them to compare the real
files with the pointers. Only the root `.gitattributes` is written, never a
nested one. A file *below* the minimum size that matches a pattern stays a plain
file, but git will want to turn it into a pointer at its next add, so `git
status` shows it as modified until you commit it. The import streams large
files, the export holds one file in memory at a time.

## Pushing the result

Applying a rewrite never pushes. Rewritten branches and tags that were on a
remote need a force push, and the push never forces blindly: each ref is sent
with a lease on the id the remote had when you previewed (not what your
tracking branch says now), so a colleague's push in between is rejected for that
ref instead of overwritten, and refs go in one all-or-nothing (`--atomic`)
push where the server allows it. From the CLI:

```
gitpoulpi rewrite push --dry-run        # asks the remote, sends nothing
gitpoulpi rewrite push --yes            # sends it
gitpoulpi rewrite note                  # what the others must run
gitpoulpi rewrite maps --out ./maps     # commit-map and ref-map
```

Protected branches (setting `rewrite.protected_branches`, default `main`,
`master`, `develop`, `release/*`) are left alone when pushed, at the rewrite and
again at the push, unless you name them (`--include-protected main`) or include
all of them. The host's own branch protection still decides whether it accepts
a force push; its refusal is shown per ref.

In the workbench, the result panel after an Apply does all of this:

- **Send the rewritten refs to the remote** lists the refs the rewrite moved
  that a remote has, each with a box (`Space`). Protected branches start
  unticked, with *protected* next to them: tick them one by one, or use *Check
  all* / *Uncheck all* (`Alt+A` / `Alt+U`). **Push rewritten branches** (`p`)
  sends the ticked ones; the answer is one row per ref (*pushed*, *already up
  to date*, *rejected: the remote moved since the preview*, *rejected:
  protected by the host*, *skipped: protected branch*, *not pushed*). A
  rejected row has **Fetch and review** (`f`): the workbench closes and the
  remotes are fetched, so the Log shows what the others pushed. Nothing is
  overwritten: the lease protects their work, and you preview again before
  trying again.
- **Copy the note** (`c`) puts the note for collaborators on the clipboard, in
  the interface language around the untranslated git commands.
- **Open the commit map** (`m`) shows where the commit map and the ref map are;
  a click on a path (or `1` / `2`) copies it.

A push cannot be undone: if you `undo` the rewrite afterwards, only your
repository changes, and the undo says so before it asks. Send the note to
everyone who has the old history: it lists the old and new tip of each branch
and tag and the commands (`git fetch`, then `git reset --hard origin/<branch>`
or `git rebase --onto`). The two maps are kept for 24 hours with the run; copy
them out if GitLab's repository cleanup or a script needs them.

## When the host protects a branch

Your host may protect a branch on its own (GitHub branch protection and
rulesets, GitLab protected branches, Azure DevOps branch policies, Bitbucket
branch restrictions) and refuse the force push a rewrite needs. When the
repository is **linked** to its host (the sidebar's pull requests section,
see [Hosting integration](hosting-integration.md)), the workbench asks the
host which branches it protects, with your own token:

- Those branches are **left out of the rewrite by default**, exactly like the
  protected branches of `rewrite.protected_branches` and the workflow: the
  Refs tab lists them with a box each and the *Check all* / *Uncheck all*
  buttons, and says why: *protected on GitHub*.
- If the token cannot read the protection (it lacks the scope, or you are not
  an administrator), the workbench says *Host protection unknown* and treats
  nothing as protected on the host's word. The rewrite is never blocked by
  that. A repository that is not linked is never asked.

When a protected branch is in the way (the preview was refused because every
branch is protected, or the host rejected the push: *rejected: protected by
the host*), the **How to get through** button (`Alt+H`) explains the three ways
out, with the exact settings page of your provider and repository (select a
link and copy it with `c`, or open it with `o`). GitPoulpi never does any of
them for you:

1. **Temporarily lift the protection** in the host's settings, push, then put
   it back.
2. **Push the rewrite under a new branch name**, then make that branch the
   default branch (or rename), and delete the old one once nobody uses it.
3. **As the last resort, delete and recreate the remote repository** and push
   everything. This loses the pull / merge requests and their reviews, the
   issues, the forks' link to the old history, stars and watchers, the CI
   settings, secrets and variables, and the webhooks and deploy keys.

From the CLI the same text comes back as `protected_by_host` in the JSON of
`rewrite preview` / `rewrite run` (when the host protects a branch of the
scope), of a refused `rewrite push` and of `rewrite note`: `provider`,
`branches`, `ways` (each with its `steps` and `links`), `recreate_loses` and the
whole explanation as `text`.

## Deleting the old history (purge)

After a rewrite the old commits are still in your repository: the undo of the
rewrite keeps them (that is what makes it undoable), and so do the reflogs. If
you rewrote to get rid of a secret, that secret is still in the `.git` folder.
Purge deletes the old history for good:

```
gitpoulpi rewrite purge                 # asks: lists what is deleted and lost (exit code 3)
gitpoulpi rewrite purge --yes           # does it
```

It drops the undo pins of the rewrite, expires the reflog entries that name the
old commits, runs `git gc --prune=now` and then checks that every old commit
and file version is really gone. The confirmation says before what you lose:
**undo is no longer possible** for the rewrite (and for any earlier operation
whose undo also needed the old commits); they stay in the history list, marked
purged. A purge cannot be stopped once it starts and cannot be undone. It does
not touch a remote.

Anything that still holds the old history blocks the purge, and then nothing is
changed. The list names each one with what to do: a branch or tag that was not
part of the rewrite, a stash on the old history (drop it, or keep it and accept
that the old history stays), another worktree whose HEAD is on it, and the
remote-tracking branches that still point at the old history (push the
rewritten branches, then fetch). Your stash list and the reflogs of other
branches are never touched.

A secret that was pushed is leaked: **rotate it first**. After the purge, the
answer lists what the host still needs, per remote: GitHub keeps old commits
reachable in cached views and pull request refs (contact GitHub Support, or
delete and recreate the repository), GitLab has a repository cleanup that takes
the `commit-map` (`gitpoulpi rewrite maps --out DIR`) plus housekeeping, Azure
DevOps and Bitbucket need a support request or a new repository, and everyone
with a clone or a fork still has the old history.

### The purge screen

In the workbench, the result panel of a rewrite has a **Purge old history...**
button (key `e`), and the palette has **Rewrite: Purge Old History...** (it
works on the newest rewrite that is not purged yet, also after you closed the
workbench). The screen lists what the purge deletes and what you lose (the
operations whose undo goes with it; the ones marked *also loses its undo* are
not rewrites but needed the old commits), then what still holds the old history,
one row each:

- a remote-tracking branch: **Fetch** updates them from the remotes;
- a stash: **Drop** asks first, because a stash is work you never committed;
- a branch, tag, worktree or other ref: **Copy name**, then deal with it where
  it lives (rewrite or delete the branch, check out a rewritten branch in the
  worktree).

The check runs again after each fix, and **Check again** (`r`) runs it on
demand. **Purge** (`p`) is available only when nothing is left. It asks a last
question, numbered and with no default (`1` purges for good, `2` leaves
everything as it was), then runs; the screen shows the step it is in and says
it cannot be stopped. The outcome says the old history is gone from this
clone, or how many old objects are still there. Below it, the steps to take on
each remote (a link can be copied). The cross, `Esc` and a click outside close
the screen, except while the purge runs.

Over MCP the purge tool does not exist unless you set `allow_history_purge =
true` in the `[mcp]` table of `config.toml`.

## The Script rule

*Add rule > Script* (or **Rewrite: Run a Script...** in the palette) adds a
Lua script rule and opens its editor over the workbench; `Enter` on the rule's
*Script* line opens it again. The editor starts from a small template.

- **Language** (a row of the rule's pane): *Lua (built in)*, *Shell script*,
  *Python* or *Lua (installed)*. Anything but the built-in Lua is an
  [external script](#external-scripts-shell-python-lua): not sandboxed, and a
  language that is off in the settings or whose interpreter is missing is
  greyed in the list with the reason beside it (picking it says what to do).
  Changing the language of a script you have not written yet swaps the
  template.
- **Script** is a text editor with line numbers. What you type is the rule at
  once; there is nothing to confirm.
- Under it the **check** line says `OK: rewrite_commit found`, or lists the
  problems with their line (`script:3: ...`; the number is red in the margin).
  It runs a moment after you stop typing, never while a key is held.
- **Parameters** has one row per `--! param` line of the script's header, with
  its label next to the field: text, a number, or a box for true/false. A value
  you clear goes back to the header's default; a text parameter with no default
  must be filled in before Preview.
- **File** (a path inside the repository, `.gitpoulpi/rewrite/script.lua` to
  start) with **Load from file...** (replaces the text) and **Save to file...**.
- **Test on selection** runs the script on the commits you selected in the Log,
  or on the 20 newest when none is selected, and writes nothing. The table lists
  each commit (short id), every field it would change as `old -> new`, and the
  script's `log(...)` lines. **Cancel** stops waiting. If the script fails, the
  commit and the reason are shown.
- The cross, a click outside the editor and `Esc` close it; `Tab` and
  `Shift+Tab` move between the editor, the parameters, the file, the buttons and
  the results; `F5` tests.

Preview refuses a script that does not compile: the Preview pane says
*a script failed* with the line (or, for a failure on a commit, its short id and
the reason). In a build without scripts the kind stays in the Add menu, greyed.

## Write a script with AI

With AI turned on ([AI assistance](ai.md)) the Script editor has a `✦ Write with AI...`
button, and the palette has `✦ Rewrite: Write a Script with AI...`, which opens the
workbench on a new Script rule with the dialog open. Nothing is sent before you press
**Send**: the dialog shows, as you type, the exact text that will go out.

- **What should the script do?** Your own words: *move every commit made at night to the
  next morning*, *replace my old email by the new one, only before 2024*.
- **Send a sample of N commits** (off): when ticked, the author, date and subject of the
  commits "Test on selection" would use (your Log selection, else the newest 20) are added so
  the model sees what your data looks like. Never file contents or the commit bodies.
  With the box off, no commit data leaves the machine.
- **What is sent** lists the instructions (the fixed script contract, then your AI
  instructions) and your request, and which profile receives them. `Tab` moves to it, the
  arrows scroll it.
- **Send** (`Tab` to it, `Enter`). While the model works the dialog waits, and `Esc` cancels
  the request. If it fails, the dialog stays with the reason so you can try again.

The answer is checked by the same daemon check as a typed script; a compile error is sent back
to the model once. The script then **replaces the text in the editor** (the dialog says so when
the editor holds something other than the starter) and the editor's title reads
*✦ written by AI: read it before Preview* until you change the text. It is never tested,
previewed or applied for you: read it, press **Test on selection**, then Preview as for any
script. The script runs in the same sandbox as one you wrote (no files, no clock, no network).

From the command line: `gitpoulpi ai rewrite-script "<instruction>" [--sample N]` prints
the script and the result of `rewrite script check` on it as JSON and runs nothing (exit 2 when it
still does not compile). The *History rewrite script* profile of Settings > AI is used.

## Create rules with AI

With AI turned on ([AI assistance](ai.md)) the workbench has a `✦ Create Rules with AI...`
button next to Save and Load, the key `Alt+I` (*Create Rewrite Rules with AI* in Settings >
Keyboard & mouse, context *Rewrite*), and the palette has `✦ Rewrite: Create Rules with AI...`. With AI
off, or in a build without AI, the button and the palette row are not there. You describe in your
own words how the history must be rewritten (*replace my old email a@b.org with c@d.org*, *remove
secrets.txt from all history*, *shift my dates by two hours*) and the model proposes the rules and
settings, which **fill the workbench**. Nothing is sent before you press **Send**: the dialog shows,
as you type, the exact text that will go out.

- **How should the history be rewritten?** A multi-line field: `Enter` is a new line,
  `Ctrl+Enter` or `Ctrl+S` sends, `Esc` and the cross cancel.
- **Send the scope, the branch and tag names and the authors of the history** (off): when ticked,
  the scope on the scope bar, the branch and tag names and the distinct authors (name and email) of
  the newest commits the Log shows (at most 20 commits) are added, so the model can name your
  email exactly (the command line's `ai rewrite-rules --context` reads the newest 200 commits
  instead). Never file contents or commit messages. With the box off only your description
  and the fixed instructions leave the machine.
- **Replace the current rules** (only when the workbench already holds rules; off): by default the
  new rules are added after the ones you have; ticked, they replace them (the scope and settings
  the answer states are applied either way).
- **What is sent** is a read-only pane on a slightly darker background, apart from the fields above
  it, and **collapsed** when the dialog opens (the dialog is then short). Click its header, press
  `Alt+W`, or `Tab` to it and press `Enter` to show it: it lists the instructions (the rule
  vocabulary, then your AI instructions) and your request, and has its own scrollbar (drag it, use
  the wheel over the pane, or the arrows, `PageUp`, `PageDown`, `Home` and `End` while it has the
  focus). The profile that receives them is on the line above. The dialog grows to fit the pane,
  within the screen. The description field gets a scrollbar of its own when the text outgrows it.

The model may use every rule kind of this page except **Script** (code: *Write a script with AI*
shows the script for review) and the secret-values form of *Search and replace in files* (secret
values are never handed to a model: use the Secrets tab). It may set the scope to the current
branch, every local branch and tag, or named branches and tags that exist, and the options *Tags*,
*committer*, *empty commits* and *abbreviated ids in messages*. What the workbench cannot express
(a date limit such as *since March*, a commit range) is not guessed: the model says so in a
remark. What happens to signatures is never decided by the model: you are asked at Apply.

**The AI only prepares rules and settings.** It never previews, applies or runs anything: the
answer is copied into the workbench like a recipe you loaded, the preview on screen goes out of
date, and Preview and Apply (with their questions and confirmations) are yours, as always.
Every rule is parsed and checked on its own by the check the editor and the engine use; a rule
that fails is left out. When something was left out, or the model made a remark, a list shows it
after the rules are in (*Left out (the valid rules were kept)*, each with its position, name and
reason) and `Enter` or `Esc` closes it. Read the rules, edit them, then Preview.

From the command line: `gitpoulpi ai rewrite-rules "<description>" [--context]` prints the proposal
as JSON (`recipe`, which `rewrite preview --recipe-json` reads, `problems`, `notes`, and
`applied: false`) and runs nothing (exit 2 when something was left out). The MCP tool is
`ai_rewrite_rules`. The *History rewrite rules* profile of Settings > AI is used.

## Lua scripts

A script rule runs your function on every commit. It is plain Lua 5.4 in a
sandbox: no files, no clock, no processes, no modules, no `pcall`; the same
script on the same history always gives the same result, and a call that runs
past 10 million instructions, 64 MiB or 5 seconds stops with an error that names
the commit.

```lua
--! name: Replace an email
--! param old_email: text "Email to replace"
--! param new_email: text "New email"
function rewrite_commit(c, ctx)
  if c.author.email == ctx.params.old_email then
    return { author = { email = ctx.params.new_email } }
  end
end
```

`c` holds `id`, `parents`, `tree`, `author` and `committer` (`name`, `email`,
`time`, `offset_minutes`), `message` and `signed`. `ctx` holds `index`, `total`,
`params`, `state` (return `state = {...}` to carry data to the next commit) and
`window.before` / `window.after` (set `--! window: N`). Return only what changes:
`author`, `committer`, `message`. Helpers: `regex_replace`, `regex_match`,
`stable_hash`, `rand`, `weekday`, `hour`, `log`, and for messages `trailers(message)`
(a list of `{key, value}`), `set_trailer(message, key, value)` (replaces the
trailers of that key; a fourth argument `"add"` or `"skip"` changes that),
`remove_trailer(message, key)`, and `at("2024-03-05 18:30", offset_minutes)`
(a date in an offset as Unix time).

From the command line: `gitpoulpi rewrite script check my.lua` compiles it,
`gitpoulpi rewrite script test my.lua --commit HEAD --param old_email=a@b` shows
what it would change without writing anything, and
`gitpoulpi rewrite preview --script my.lua --script-param old_email=a@b` plans
the whole rewrite.

## External scripts (shell, Python, Lua)

The same contract as a Lua script, run by a program on your machine: *Shell
script* (bash or sh; PowerShell on Windows, or any `.ps1`), *Python* and
*Lua (installed)* (a `lua` of your own, not the built-in one; it has no JSON
library, so the template reads the line with a pattern). **They are not
sandboxed: an external script runs with your rights** and can read or change
anything you can. The rule's pane and the editor's title say so.

How it talks: GitPoulpi starts the script once per run and sends it one JSON
request per line on its standard input (the fields of a commit, `params`, the
`state` returned by the last answer); it answers with one JSON line on standard
output, the same fields as a Lua script returns (`author`, `committer`,
`message`, `state`, `log`), or `{"error": "text"}` to give up. Print
diagnostics on standard error: they appear as the commit's `log` lines. The
header lines start with `#!` for shell and Python (`#! name: ...`,
`#! param old: text "Old"`; a shebang line stays a shebang) and with `--!` for
Lua.

```sh
#!/bin/sh
#! name: Uppercase author names
while IFS= read -r line; do
  name=$(printf '%s' "$line" | sed -n 's/.*"author":{[^}]*"name":"\([^"]*\)".*/\1/p' | tr 'a-z' 'A-Z')
  printf '{"author":{"name":"%s"}}\n' "$name"
done
```

**What it gets.** An empty environment except `PATH`, `HOME` / `USERPROFILE`,
`SYSTEMROOT`, `TEMP` / `TMP` and `LANG`; a fresh temporary folder as its working
folder (holding its own copy of the script); not the repository's path.
**Time limits:** `rewrite.script_timeout_ms` (5000) for each answer and
`rewrite.script_total_timeout_s` (3600) for the whole pass. When it is
exceeded, crashes, prints something that is not JSON or the job is canceled,
GitPoulpi kills the script and everything it started, and the rewrite stops
with the commit and the reason; nothing has moved.

**Which languages exist.** `rewrite.external_scripts` in `config.toml` lists
them: `["shell"]` by default; add `"python"` and `"lua"` yourself in
`config.toml` (`[rewrite]`, `external_scripts = ["shell", "python"]`; a list
cannot be set with `gitpoulpi config set`). A
language that is not listed is greyed out in the workbench and refused by the
CLI and MCP (`scripts_disabled`); a listed one whose interpreter cannot be
found is greyed with the install hint (`tool_missing`).

**Trust.** Even when a language is on, a script runs only after you trusted
its exact text. A script you type in the workbench is yours and runs at once;
one that comes with a shared recipe or a file of the repository is not:
Preview and *Test on selection* stop and ask *Trust this script?* with two
numbered answers and none preselected (`1` trust it and go on, `2` do not,
`Esc` leaves it). Trusting remembers the SHA-256 of the text for this
repository, so a changed script asks again, and a cloned repository cannot
run code on you by itself. From the command line, read the file, then
`gitpoulpi rewrite script trust FILE` (or `--trust` on `preview`, `run`,
`script check` and `script test`, which trusts and goes on). An agent over MCP
cannot trust anything: it can run only a script you trusted before.

**Check.** The check line under the editor says *Not trusted yet* for an
untrusted script (nothing was started); for a trusted one GitPoulpi starts it
once with a made-up commit (`"dry_run": true`) and checks that the answer
follows the contract. `gitpoulpi rewrite script check FILE` does the same and
reports `trust`, `hash` and `interpreter`.

## From the command line

`gitpoulpi rewrite` (and the MCP tools of the same names) runs the same engine,
for scripts and agents. A **recipe** is a TOML or JSON file, the one the
workbench saves (`--recipe FILE`, or `-` for standard input), and flags add rules
after the recipe's own.

```
gitpoulpi rewrite preview --recipe fix.toml --all     # plan, changes nothing
gitpoulpi rewrite apply --run ID --digest D           # run the plan you reviewed
gitpoulpi rewrite run --fix-message-encoding --signatures strip --yes
gitpoulpi undo                                        # puts everything back
```

- Rules as flags: `--set-name` / `--set-email` with `--match-email` (identity),
  `--mailmap`, `--shift-dates`, `--replace-message FIND=WITH`, `--trailer-add`,
  `--trailer-remove`, `--trailer-map`, `--fix-message-encoding`
  (`--fix-name-encoding`), `--remove-path`, `--remove-blobs-over`,
  `--replace-content`, `--eol` (`--eol-paths`), `--strip-exec`, `--reencode`
  (`--encoding-paths`), `--move-path`, `--into-subdir`, `--extract`,
  `--lfs-import`, `--lfs-export`, `--script`, `--remove-signatures`,
  `--remove-finding`. The scope is the current branch, `--all-local`, or
  `--ref R` (repeatable), from `--since REV`.
- **Questions with no default** are flags: `--signatures
  strip|resign-signed|resign-all` when signed commits change, and
  `--extract-mode` for an extraction. Without them the answer is a refusal or a
  `needs_choice` (exit code 3), never a guess.
- **Exit codes:** 0 done; 1 an error (unreadable recipe, bad flags); 2 refused,
  with the reason (`stale`, `signature_choice_needed`, `dirty_paths_touched`,
  `empty_scope`, ...); 3 a question to answer, or pushed refs that need `--yes`.
- `preview` keeps the plan in the repository, so `apply` works from a later
  command, after a daemon restart, or from another frontend; a plan that no
  longer matches its digest is `stale`.

The [CLI reference](cli-reference.md) has every flag, and
[MCP and agents](mcp-and-agents.md) says which tools an agent gets (the purge
tool does not exist unless you turn it on, and an agent cannot trust a script).

## Proposals from AI agents

An agent can **suggest** a rewrite, never run one. `gitpoulpi ui propose rewrite
--recipe FILE|-` (the MCP tool `ui_propose_rewrite`, off until `[mcp]
allow_ui_propose = true`) opens the workbench in your running window with the
recipe's rules already in the list and a purple banner, *✦ Proposed by
<agent>: a history rewrite (3 rules) · Ctrl+Enter accepts · Esc rejects*.

- **Nothing runs.** No preview, no apply, no push, no purge: the proposal only
  fills the rules. You read them, edit them if you like, **Preview**, and apply
  with your own key. Closing the workbench rejects the proposal.
- **What the agent may not put in it:** a script (it would run when previewed),
  more than 50 rules, secret values (they are removed from the recipe before it
  is sent). Such a proposal is refused with the reason.
- The agent can read the outcome in `ui state` (`accepted` when you applied the
  rules as proposed, `accepted_edited` when you changed them first, `rejected`
  when you closed it). One proposal at a time; it is refused while another
  dialog of yours is open.

## Tools

Most of a history rewrite needs nothing beyond Git: changing authors and
committers, applying a `.mailmap`, anonymising, shifting or windowing dates,
rewriting messages and trailers, replacing content, removing paths or large
blobs and moving paths all run inside GitPoulpi.

**Lua scripts are embedded.** The interpreter is built into GitPoulpi, so a
Lua script needs no tool and no setup. Shell, Python and installed-Lua scripts
need their interpreter (see the table below) and are described under
[External scripts](#external-scripts-shell-python-lua).

A few features use an external program. GitPoulpi checks for it (the program
is found on your `PATH`, on Windows also through `PATHEXT` and Git for
Windows' own `usr/bin`, and must answer `--version` within 2 seconds); the
answer is cached for a minute and "Check again" asks again. A rule that needs
a missing program stays in the list, greyed, with the reason and how to
install it, and Preview refuses before anything runs.

| Feature | Tool | Notes |
|---|---|---|
| Re-sign commits | `gpg` (OpenPGP), `ssh-keygen` (`gpg.format = ssh`) or `gpgsm` (`gpg.format = x509`) | SSH and X.509 signing also need `user.signingkey`. `gpg.program`, `gpg.ssh.program` and `gpg.x509.program` are honored. |
| LFS migration (the two LFS rules) | `git-lfs` 3.0 or newer | The same check as the LFS pane. |
| Python scripts | `python` 3.8 or newer (`py -3` on Windows) | Off by default: add `"python"` to `rewrite.external_scripts`. |
| Lua (installed) scripts | `lua5.4`, `lua54`, `lua` or `luajit` | Off by default: add `"lua"` to `rewrite.external_scripts`. |
| Shell scripts | `bash`, `pwsh` (PowerShell) or `sh` | On by default (`"shell"`). A PowerShell script (a `.ps1` file, a `pwsh` shebang, or any shell script on Windows) needs `pwsh` or `powershell`; a POSIX one needs `bash` or `sh` (on Windows, Git for Windows provides them). |

Install hints, Windows first:

- Windows: Git for Windows bundles `gpg`, `ssh-keygen`, `bash`, `sh` and
  Git LFS. Otherwise `winget install GnuPG.GnuPG`, Settings > Optional
  features > OpenSSH Client, `winget install GitHub.GitLFS`,
  `winget install Python.Python.3.12`, `winget install DEVCOM.Lua`,
  `winget install Microsoft.PowerShell`.
- Linux: the distribution's packages (`gnupg`, `openssh`, `git-lfs`,
  `python3`, `lua5.4`, `bash`, `powershell`).
- macOS: `brew install gnupg git-lfs python lua powershell`; `ssh-keygen` and
  `sh` are built in.
