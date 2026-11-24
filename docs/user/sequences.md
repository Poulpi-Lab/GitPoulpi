A **sequence** is a named list of commands that run one after the other: for
example *Stage all changes*, *AI: Generate commit message*, *Commit*, *Push*
as one **Backup** key. Each step is a command of the
[command palette](keybindings.md); a sequence is a palette row of its own
(`Sequence: <name>`), can be bound to a key, and is edited in
**Settings > Sequences** or in `config.toml`.

## The Sequences page

**Settings > Sequences** is a [list page](settings.md#lists): the sequences on
the left, the selected one's rows on the right. Every change is applied and
written at once, and the palette and the keymap follow without a restart.

```
 Sequences
+- Sequences -------------+  Editing sequence 1/2
| This installation       |  Name         Backup
| > Backup        Alt+B   |  Id           backup
|   Worktree and agent    |  Description  Stage everything, ...
| This repository         |  Scope        This installation
|   Release               |  Shortcut     [ Alt+B ] [ Clear ]
+-------------------------+  Steps        Stage all > AI: ... > Commit > Push
| [+ Add v] [Dup] [Remove]|              [ Edit steps (4) ] [ Run now ]
| [Up] [Down]             |
+-------------------------+
```

* **+ Add** offers **Empty sequence** and the three presets, **From preset >
  Backup**, **Compose and push** and **Worktree and agent**. A preset is a
  starting point: nothing is active until you add it, and what you add is a
  copy you can edit. Presets are named in the interface language when they
  are added.
* **Duplicate**, **Remove** (asks first), **Up** / **Down** or a drag reorder
  the list (`Insert`, `Ctrl+D`, `Delete`, `Alt+Up`, `Alt+Down`). The order of
  the list is the order in `config.toml`.
* A row with a problem is marked in red and says why (no steps, a step this
  version does not know, a `Never` pause that lacks the option it needs).
  A sequence with problems **can be saved**, unknown future steps are kept as
  they are, but its palette row stays grayed out until it is fixed.

### The rows of a sequence

| Row | What it does |
|---|---|
| **Name** | What the palette shows: `Sequence: <name>`. |
| **Id** | The short name used in `config.toml`, on the command line and as the shortcut's key (`a-z`, `0-9`, `_`, `-`). Renaming it moves the shortcut with it, in the same write. |
| **Description** | Optional, for yourself. |
| **Scope** | See below. |
| **Shortcut** | A button showing the key. `Enter` or a click opens the same key capture as **Keyboard & mouse**: conflicts and terminal warnings show while you press, and a key another action uses is refused the same way. **Clear** removes it. The key is saved as `[keybindings.sequence] <id>`. |
| **Steps** | The steps in order. **Edit steps** opens them. |
| **Run now** | Runs the sequence in the current window. |

### Scope

Each sequence has a scope, saved with it:

| Scope | Where it is saved | Who sees it |
|---|---|---|
| **This installation** (default) | `[[sequences]]` in `config.toml`, with this installation as its owner | Only this install of GitPoulpi. |
| **All installations on this computer** | `[[sequences]]` in `config.toml`, no owner | Every installation that shares the file. |
| **This repository only** | `.gitpoulpi/sequences.toml` in the current repository | Whoever has the repository. It only counts once you trust it (see below). |

Changing the scope moves the definition: choosing **This repository only**
writes it to the repository's file and removes it from `config.toml`, and
the reverse. A repository scope needs a repository to be open.

### Repository sequences and trust

The file `.gitpoulpi/sequences.toml` can be committed, so it can come from
other people, and a sequence runs commands on your machine. A repository's
sequences are therefore **not offered until you trust them**, the way the
commands of a committed workflow file are.

When a repository has such a file nobody trusted yet, a notice appears over
the Log: *This repository defines 2 sequences* with **Review**, **Trust** and
**Ignore**. It never blocks anything.

- **Review** lists every sequence with every step, its pause and its options,
  read-only. **Trust** there trusts what was listed; **Close** (or `Esc`)
  trusts nothing. If the list is longer than the window, Trust is not offered
  there (enlarge the window): you never trust what you could not read.
- **Trust** (the button, or the palette's *Sequence: Trust This Repository's
  Sequences*) trusts the file exactly as it is now.
- **Ignore** hides the notice until the file changes.
- From the keyboard: the palette has *Sequence: Review / Trust / Ignore This
  Repository's Sequences*.

What is recorded is a digest of the ids, the commands, the pauses and the
options, kept in the repository's local state (never in the repository). If
any of those change, for example after a pull, the notice comes back as *The
sequences of this repository changed* and they leave the palette and the keys
until you trust them again. Renaming a sequence or editing its description
changes nothing. A sequence you write yourself in Settings stays trusted when
the file was already trusted (or empty) before you edited it.

Once trusted, repository sequences appear as `Sequence: <name>` in the
palette like your own and can be given a shortcut in Settings. A sequence with
the same id as one of yours is a clash: yours wins and the other is reported
as a duplicate.

From the command line: `gitpoulpi sequence list` shows the file's sequences
with `trusted` and a `repo_trust` state (`untrusted`, `changed`, `trusted`);
`gitpoulpi sequence trust` (and `untrust`, `--repo PATH` for another
repository) is for a person to run after reading `sequence show`.
`sequence run` refuses an untrusted repository sequence. AI agents cannot
trust: there is no MCP tool for it.

## Steps

**Edit steps** replaces the list with the steps of the sequence (the title
says **Steps of <name>**) and the rows with the selected step's. **Back**
(the cross at the top of the rows) or `Esc` returns to the sequences.

* **+ Add** opens **Add a step**: every command a sequence may run, under its
  palette name (the AI ones with the sparkle), filtered as you type with the
  palette's matching. A step this version cannot run is grayed with the
  reason. `Enter` or a click adds it at the end; `Esc` or the cross closes
  the popup.
* **Duplicate**, **Remove**, **Up** / **Down** and drag work as for the
  sequences.
* A destructive command (delete a branch, discard changes...) is never
  offered: a sequence cannot run one.

### Pause

Each step has a **Pause**:

| Pause | Meaning |
|---|---|
| **Default** | The step stops only when it needs an answer, on its usual dialog (a branch name, a commit message that is still empty...). |
| **Always** | Stop before this step and ask to continue. |
| **Never** | Never stop. The step's options must give it everything it needs, so a step that asks for a name needs its **Name** option filled in. |

A step that always asks before it runs (a force push) does not offer
**Never**: the row says why. **Never** on the AI message step is allowed
as an explicit choice.

### Options

The rows under **Pause** are the step's options: a **Name** or **Message**
text, a **Branch**, a checkbox such as **Open in a new tab**, or a dropdown
filled from your configuration (a **Remote**, an **AI profile**, an **Open
in** entry). Texts accept the variables `${branch}`, `${date}`, `${time}` and
`${n}` (a number that makes the name unique). Clearing a text removes the
option.

## config.toml

```toml
[[sequences]]
id = "backup"
name = "Backup"
scope = "installation"
owner = "<this installation's id>"

[[sequences.steps]]
run = "stage_all"

[[sequences.steps]]
run = "ai_generate_commit_message"
pause = "always"

[[sequences.steps]]
run = "commit"

[[sequences.steps]]
run = "push"

[keybindings.sequence]
backup = "Alt+B"
```

Keys this version does not know, and steps it does not know, are kept when
the page saves.

## Running a sequence

A sequence is a named list of steps, each one a command of the palette (stage everything, write the commit message with the AI, commit, push...). Run it from the palette (`Sequence: <name>`), with the key you gave it in Settings > Keyboard & mouse (`[keybindings.sequence]` in `config.toml`), or from the command line (`gitpoulpi sequence run`).

While it runs, a strip over the top of the Log shows `<name> k/n`, the step, and what it waits for. The run also shows in the Jobs panel as `Sequence <name> k/n: <step>`. One sequence runs at a time; starting a second says which one is running.

**Where it stops to ask.** Each step has a `pause` setting (`default`, `always`, `never`):

| Step | Default | `pause = "never"` |
|---|---|---|
| AI commit message | the message lands in the message field; the strip says "Review the message, then Continue" | goes on at once |
| Commit | an empty message always stops: the field has the keyboard, and your commit (or Continue) goes on | not available when there is no message to commit |
| AI compose commits | the review opens; Apply goes on, closing it cancels | applies the proposal as it is |
| Create worktree | the dialog opens with the name filled in | creates it from `options.name` without the dialog |
| Push | when `git.push_behavior = ask` the "Push to..." chooser opens | pushes to `options.remote` |
| Force push with lease, staging files that look like secrets | always asks | not allowed |

`pause = "always"` on any step shows "Next: <step>" with **Continue** and **Cancel** before it runs. If the push is rejected because the branch diverged, the usual dialog opens and the strip says "Push rejected: choose in the dialog"; pulling then pushing (or a force push with lease) continues the run, closing the dialog cancels it.

**Continue** is the strip's button, `Sequence: Continue` in the palette, or the key bound to `sequence.continue`. **Cancel** is the strip's close cross, `Sequence: Cancel` (`sequence.cancel`), the Jobs panel, or `Esc` when nothing else wants it (an open dialog closes first, which also cancels). Canceling keeps the steps that already ran and says so: "<name> canceled after step k/n (<step>); earlier steps are kept".

**When a step fails** the run stops there: the status line names the sequence, the step and the error, the strip turns red with **Details** (the Activity Log entry, which lists the step's options) and a close cross, and the earlier steps stay done.

**Worktrees.** A create worktree step with `open_tab = true` moves the run to the new worktree's tab: the next steps (`tab_dir_terminal`, `tab_open_agent_here`, `tab_open_in`) act there. If another tab is on screen the strip says "waiting for tab <name>" with **Go there**. Closing that tab cancels the run. A run does not survive quitting GitPoulpi.

### An agent proposing a run

With *Let AI agents propose changes for me to confirm* on (`[mcp]
allow_ui_propose`), an agent can ask your window to run a sequence
(`gitpoulpi ui sequence run <id>`, MCP `ui_sequence_run`). A banner appears:
*✦ Proposed by <agent>: run sequence <name>* with **Run** and **Dismiss**.
Nothing runs until you press Run (click it, or `Ctrl+N` then `Enter`);
Dismiss, `Esc` (when nothing else wants it) or the palette's *Agent: Dismiss
Pending View* drops it. The run then goes exactly as if you had started it,
pausing where the sequence says. The proposal is refused when the sequence
does not exist, is invalid, or is a repository sequence you have not trusted.
