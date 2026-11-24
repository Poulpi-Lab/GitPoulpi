# Sequences

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
| **This repository only** | `.gitpoulpi/sequences.toml` in the current repository | Whoever has the repository. A repository sequence has no shortcut yet, and the repository file is only read once you trust it. |

Changing the scope moves the definition: choosing **This repository only**
writes it to the repository's file and removes it from `config.toml`, and
the reverse. A repository scope needs a repository to be open.

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

## Create and edit with AI

With the [AI turned on](ai.md#sequence-draft) (Settings > AI), two entries appear:
**Add > Create with AI...** (also the palette row `Sequence: Create with AI...`,
which opens Settings on this page with the dialog) and **Edit with AI...** next
to **Run now** for the selected sequence. Neither exists while the AI is off.

```
+- Create a sequence with AI ------------------------------- x -+
| Describe the sequence you want                                |
| +-----------------------------------------------------------+ |
| | stage everything, write the commit message with the AI,   | |
| | commit, then push                                         | |
| +-----------------------------------------------------------+ |
| Enter adds a line. Ctrl+Enter sends. Esc closes.              |
| [ ] Send context (remote names, AI profiles, Open in labels)  |
|    Off: only your description is sent.                        |
| The AI only proposes. Nothing is saved before you accept...   |
| [ Send ] [ Cancel ]                                           |
+---------------------------------------------------------------+
```

You write what you want in your own words; **Send** (or `Ctrl+Enter`, `Ctrl+S`)
asks the model, through the same profile, jobs indicator and `Esc` as every
other AI action (and the same "What is sent" question). The answer is a
**proposal** you read before anything happens:

* the name, id, description and the steps with their pause and options;
* for **Edit with AI**, a diff against the current sequence: `+` a step added,
  `-` a step removed, `~` a step whose pause or options changed, `~ Name: "a"
  -> "b"` for a field;
* **Left out**: a step the model proposed that is not allowed (unknown,
  destructive, a sequence inside a sequence, an impossible pause), with the
  reason; the valid steps are kept;
* the model's **notes**, and a **suggested shortcut** that is only shown, with
  whether the key is already taken. It is never bound: use the **Shortcut**
  button afterwards.

```
+- Proposed change ----------------------------------------- x -+
| + added   - removed   ~ changed                               |
| ~ Name: "Daily" -> "Daily 2"                                  |
| Steps (3)                                                     |
| + 1. Git: Fetch All Remotes                                   |
|   2. Git: Stage All Changes                                   |
| - Git: Commit...                                              |
| ~ 3. Git: Push: defaults -> Pause: Always                     |
| Left out                                                      |
| step 4 `reset_hard`: destructive steps are not allowed        |
| [ Accept ] [ Edit before saving ] [ Reject ] [ Back ]         |
+---------------------------------------------------------------+
```

* **Accept** writes it with one save (a new sequence, or the replacement of the
  edited one, which keeps its id, scope and key) and selects it in the list.
* **Edit before saving** does the same and opens its steps so you can tweak
  the rows (this page saves every edit at once, so the sequence is saved
  already).
* **Reject** writes nothing. **Back** returns to the description to send again.
* **Accept** is unavailable when there is nothing to accept: no usable step, a
  problem the page would flag, or an edit that changes nothing.

Nothing is written before **Accept**, and a sequence is never run from here.
Closing the dialog (the cross, `Esc`, a click outside) asks "Discard changes?"
while a description is typed or a proposal is open; closing Settings asks too.

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
