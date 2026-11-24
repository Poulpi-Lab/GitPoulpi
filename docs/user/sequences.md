# Sequences

Named lists of palette commands you can run with one key.

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
