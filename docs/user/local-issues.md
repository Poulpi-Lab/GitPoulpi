**English** · [Čeština](cs/local-issues.md) · [Deutsch](de/local-issues.md) · [Español](es/local-issues.md) · [Français](fr/local-issues.md) · [Italiano](it/local-issues.md) · [日本語](ja/local-issues.md) · [한국어](ko/local-issues.md) · [Nederlands](nl/local-issues.md) · [Polski](pl/local-issues.md) · [Português (Brasil)](pt-BR/local-issues.md) · [Português (Portugal)](pt-PT/local-issues.md) · [Русский](ru/local-issues.md) · [Svenska](sv/local-issues.md) · [Türkçe](tr/local-issues.md) · [Українська](uk/local-issues.md) · [简体中文](zh-CN/local-issues.md)

# Local issues and Obsidian

Keep your issues as a folder of Markdown notes: no account, no token, no network. The folder can be inside the
repository (`docs/issues/`, committed with the code) or anywhere on disk, for example a subfolder of an
[Obsidian](https://obsidian.md) vault. GitPoulpi reads and writes the notes. Obsidian, any editor or a shell can edit
them at the same time, and the lists follow.

![Local Markdown issues in the sidebar, with the row menu of one note: open the note, close it, edit its labels or milestone](images/local-issues.png)

## Set it up

1. **Settings > Integrations > Issue trackers > Local Markdown > Add profile.** Type the folder, or press
   **Browse...** and pick it. A relative folder (`docs/issues`) is read against each repository it is linked to; an
   absolute one (`/home/me/Vault/Projects/Issues`) is the same everywhere. There is no token to enter.
   - CLI: `gitpoulpi hosting integrations add --kind local_markdown --path docs/issues`.
2. In a repository, open the sidebar's **ISSUES** section, pick the provider **Local Markdown - <name>** (the folder
   is chosen with it; the repository dropdown lists the folder and its direct subfolders if you want another).
   - CLI: `gitpoulpi hosting link issues --id <profile>` (the profile's folder is the default `--project`).

Nothing is linked on its own: like every issue source it is opt-in per repository.

## Comments on a note

`Enter` on a note's row in the ISSUES section, or on its card, opens its conversation: the note's text, then the
comments. Comments are kept in the note file itself, in a final `## Comments` section, one `### <author> - <UTC time>`
entry each (an edited one adds `(edited <time>)`). Nothing above that heading is ever touched, so Obsidian and a shell
read and edit the same file. You can write, edit and delete the comments of the local author (the profile's
`local.me`, else the connected user, the name *Assigned to me* uses); nobody else's. The box is the usual Markdown
composer with a preview, and *Writing as <name> (local note)* names the author. Nothing goes over the network. A note with a `## Comments` heading you wrote by hand is read as such.

## The notes

One note per issue, named `<id>-<slug>.md` (`12-crash-on-empty-name.md`), with a frontmatter block:

```yaml
---
id: 12
title: Crash on empty name
status: todo
assignees: [kai]
labels: [bug]
milestone: v0.1
created: 2026-10-14
updated: 2026-10-14T09:30:00
---
Free Markdown. GitPoulpi never rewrites the body.
```

- New issues get `max(id) + 1`.
- GitPoulpi only writes the properties it owns (`title`, `status`, `assignees`, `labels`, `milestone`, `updated`).
  **Every other property, the body, the line endings and the order of the keys are kept byte for byte.** Writes are
  atomic (a temporary file, then a rename).
- Reading is forgiving: a note with no `id` takes it from its file name, a note with no `status` is `todo`, a note
  with no frontmatter takes its title from the first `# heading`. A file that is not an issue (`readme.md`) is ignored.
- A status is free text. **Open** means "not the done status".

### Comments in a note

A comment on a note is written into the note itself, in a last `## Comments` section (no network, and it reads fine in
Obsidian):

```markdown
(the note's text)

## Comments

### kai - 2026-11-15T10:00:00Z
First comment, **any Markdown**.

### ann - 2026-11-15T11:30:00Z (edited 2026-11-15T12:00:00Z)
Second one.
```

- An entry starts at a line that is exactly `### <author> - <UTC time>`, with ` (edited <UTC time>)` after an edit;
  its text runs to the next entry. The section is the last `## Comments` heading outside a code fence and runs to the
  end of the file, so write nothing after it.
- A comment line that would read as an entry or as the heading is stored with a leading backslash (`\### ...`, which
  Markdown shows as the same text) and read back without it.
- Posting adds to the end and leaves everything above byte for byte (line endings included); an edit or a delete
  rewrites only the section. A post also moves the note's `updated` (a note without frontmatter is given none).
- The author is the profile's local author (the "me" of the Local issues settings). You can edit or delete only your
  own comments; a comment's id is its time and author, and two posts in the same second are told apart by moving the
  second one on by a second.
- There are no replies, no internal comments and no event history for a note.

## What you can do

| In the ISSUES section | Keyboard / palette |
|---|---|
| New issue (the `+`, or `n`): title, description, status, assignees, labels, milestone | `Hosting: New Issue...` |
| Edit assignees / labels / milestone from the row menu. Pick from what the folder already uses, or **type a new label or login** and take the `Add "..."` row | `Hosting: Edit Labels...` and friends |
| **Close issue** / **Reopen issue** (sets the done status, or the first other one) | `Local Issues: Close Issue...` / `Reopen Issue...` |
| **Open note** (your default program for `.md`) | `Integrations: Open Issue...` |
| **Open in Obsidian** (only when the folder is inside a vault, i.e. a `.obsidian/` folder above it) | `Local Issues: Open in Obsidian...` |
| Filters: Open / Closed / All, "Assigned to me" | the filter dropdown |

"Assigned to me" matches the profile's `me` setting, else the connected name, else the profile name.

Branches and commits work as for GitHub: "Create branch from issue" gives `12-crash-on-empty-name`, and `#12` in a commit
message or `12-...` in a branch name is recognized.

Changes made outside GitPoulpi (Obsidian, a shell, a `git pull`) show up within about a second: the folder's file names
and modification times are looked at on a background thread, never on the render thread.

## Configuration file

In `config.toml`, next to the profile (every key is optional):

```toml
[[integrations]]
id = "vault"
kind = "local_markdown"
url = "/home/me/Vault/Projects/Issues"
token_ref = "integration:vault"

[integrations.local]
statuses = ["todo", "doing", "review", "done"]   # the board's column order
done_status = "done"                            # closes an issue
reference_prefix = "LOC"                         # shown as LOC-12 instead of #12
me = "kai"                                       # who "assigned to me" means
```

Statuses used by notes but not listed in `statuses` follow the listed ones. `statuses` is the order of the
[board](#the-kanban-board)'s columns (`gitpoulpi hosting issue options` prints it).

## The Kanban board

![The board of a notes folder: one column per status, the cards with their assignees, labels and milestone](images/issue-board.png)

If you work alone, or in a team that keeps its issues next to the code, the folder is also a Kanban board.
**ISSUES header > Board**, or the palette's **Issues: Open Board**, opens it over the window; `Esc` closes it.

- **Columns** are the statuses: the ones listed in `statuses` first, in that order, then any other status a note uses.
  An empty column is still shown. Each header carries its card count.
- **Cards** show the issue number, the title (two lines), the assignees as initials, the labels and the milestone.
  When the columns do not all fit, the board scrolls sideways (the bar under it, the wheel with `Shift`, or the arrows).
- **Move a card** by dragging it onto another column (the column lights up under the pointer), with
  `Shift+Left` / `Shift+Right`, or with **Board: Move Card To...** (`m`), a dropdown of the columns. The card moves at
  once; the note's `status` is rewritten as in [the notes section](#the-notes), nothing else in the file. If the write
  fails the card goes back and the error is on the status line.
- **Open** a card with `Enter` or a double click: the note's [conversation](conversations.md) opens in the window, with the note's text and its comments (the card menu's **Open note** opens the file in your default program for `.md`). `e` edits its
  assignees, labels or milestone, `.` or a right click opens the card menu, `n` starts a new issue in the selected
  column (the form comes back to the board when it closes).
- **Filter** by assignee (`a`), label (`l`), milestone (`M`) or text (`/`); `x` clears them. The filters are
  dropdowns: type to narrow, `Enter` picks.
- **Live**: a note changed from Obsidian or a shell moves its card within a second.

Every mouse gesture has a keyboard key and a palette command (**Board: ...**, listed in the
[keybindings](keybindings.md) page).

## Obsidian Bases

**Palette: `Local Issues: Create Obsidian Base`** (CLI: `gitpoulpi hosting issue base`) writes `issues.base` next to the
notes, once: a *table* view and a *kanban* view grouped by `status`, in the order of `statuses`. Open it in Obsidian to
see the same issues as a board there. GitPoulpi never edits an existing `.base` file (yours or a previous one): it
refuses and says so. The format follows Obsidian's Bases syntax
(<https://obsidian.md/help/bases/syntax>); the kanban view needs a recent Obsidian.

## CLI and MCP

The same commands as every issue source work on it: `hosting issues`, `hosting issue create [--list <status>]`,
`hosting issue edit <n> --add-label ...`, `hosting open issue <n>`, `hosting refs`. Four more are for notes:
`hosting issue close <n>`, `hosting issue reopen <n>`, `hosting issue move <n> --status <s>` and `hosting issue base`.
See the [CLI reference](cli-reference.md).

## Limits

- A new **milestone** cannot be typed in the picker (pick one the folder already uses), but `milestone:` in a note can
  be anything and shows up as soon as it is saved.
- Only the notes directly in the folder are issues (not subfolders).
- There is no delete: remove the note in your file manager or Obsidian (its trash is yours).
